use crate::utils::*;

use std::fmt;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

#[cfg(not(target_os = "windows"))]
#[path = "linux_parser.rs"]
mod parser;

#[cfg(target_os = "windows")]
#[path = "win_parser.rs"]
mod parser;

#[cfg(not(target_os = "windows"))]
const TRACKER_LOGS_DIR: &str = "~/.tracker/histlogs";
#[cfg(target_os = "windows")]
const TRACKER_LOGS_DIR: &str = "~\\AppData\\Local\\tracker\\histlogs";

#[cfg(not(target_os = "windows"))]
const TRACKER_INIT: &str = "~/.tracker/.tracker.rc";

#[cfg(target_os = "windows")]
const TRACKER_INIT: &str = "~\\AppData\\Local\\tracker\\startup-files\\tracker.ps1";

#[cfg(not(target_os = "windows"))]
const SHELL: &str = "/bin/bash";
#[cfg(target_os = "windows")]
const SHELL: &str = "powershell";

#[derive(Clone, Debug)]
pub struct ConsoleError(String);

impl fmt::Display for ConsoleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ConsoleError {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConsoleHistEntry {
    pub timestamp: u64,
    #[cfg(not(target_os = "windows"))]
    pub user: String,
    pub cmd: String,
    #[cfg(not(target_os = "windows"))]
    pub status: u64,
    pub output: String,
}

#[derive(Clone, Debug)]
pub struct ConsoleHistControl {
    id: Option<u32>,                /* history control id, None if isn't initialized */
    history: Vec<ConsoleHistEntry>, /* history themself */
    histfile: PathBuf,              /* where the history is stored */
    offset: u64,                    /* location within this history */
    length: u64,                    /* length of the histfile */
}

impl ConsoleHistControl {
    pub fn new() -> ConsoleHistControl {
        ConsoleHistControl {
            id: None,
            history: Vec::new(),
            histfile: PathBuf::new(),
            offset: 0,
            length: 0,
        }
    }

    pub fn init(&mut self, id: u32) -> Result<(), ConsoleError> {
        self.reset();

        let histfile = if cfg!(not(target_os = "windows")) {
            format!("{}/hist.{}.log", TRACKER_LOGS_DIR, id)
        } else {
            format!("{}/history.log", TRACKER_LOGS_DIR)
        };

        let histfile = path_expand(&histfile).map_err(|err| {
            return ConsoleError(format!("histfile error: {}", err.to_string()));
        })?;

        let length: u64;
        match std::fs::metadata(&histfile) {
            Ok(metadata) => length = metadata.len(),
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    length = 0;
                } else {
                    return Err(ConsoleError(format!("histfile error: {}", err.to_string())));
                }
            }
        };

        self.id = Some(id);
        self.histfile = histfile;
        self.offset = 0;
        self.length = length;

        Ok(())
    }

    pub fn update(&mut self) -> Result<usize, ConsoleError> {
        if self.id.is_none() {
            return Err(ConsoleError(
                "history control not ready: call .init() before.".to_owned(),
            ));
        }

        self.length = std::fs::metadata(&self.histfile)
            .map_err(|err| ConsoleError(format!("histfile error: {}", err.to_string())))?
            .len();

        if self.length == self.offset {
            return Ok(0);
        }

        let contents = file_read_from(&self.histfile, self.offset)
            .map_err(|err| ConsoleError(format!("failed to read histfile: {}", err.to_string())))?;

        let history = parser::parse_histfile_contents(&contents).map_err(|err| {
            ConsoleError(format!("failed to parse histfile: {}", err.to_string()))
        })?;

        let n = history.len();
        self.history.extend(history);
        self.offset = self.length;

        Ok(n)
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }

    pub fn reset(&mut self) {
        self.clear();
        self.id = None;
        self.offset = 0;
    }

    pub fn history(&self) -> &Vec<ConsoleHistEntry> {
        &self.history
    }
}

/**
 * Start a shell (bash/powershell) with commands history readable by `ctrl`.
 */
pub fn attach_console(
    id: u32,
    ctrl: std::sync::Arc<std::sync::Mutex<ConsoleHistControl>>,
) -> Result<(), ConsoleError> {
    /* init history control */
    ctrl.lock().unwrap().init(id).map_err(|err| {
        ConsoleError(format!(
            "failed to init history control: {}",
            err.to_string()
        ))
    })?;

    /* start new shell */
    let init_file = path_expand(TRACKER_INIT).map_err(|err| {
        ConsoleError(format!(
            "problem with console init file: {}",
            err.to_string()
        ))
    })?;

    if !init_file.as_path().exists() {
        return Err(ConsoleError(format!(
            "console init file {} not founded",
            init_file.display()
        )));
    }

    println!(
        "[*] starting {} with init file: {}",
        SHELL,
        init_file.display()
    );

    let args = if cfg!(target_os = "windows") {
        vec![
            "-noexit".to_owned(),
            "-command".to_owned(),
            format!(". {}", init_file.as_path().to_str().unwrap()),
        ]
    } else {
        vec![
            "--rcfile".to_owned(),
            init_file.as_path().to_str().unwrap().to_owned(),
        ]
    };

    Command::new(SHELL)
        .args(&args)
        .env("TRACKER_ID", id.to_string())
        .spawn()
        .map_err(|err| ConsoleError(format!("failed to run {}: {}", SHELL, err.to_string())))?
        .wait()
        .map_err(|err| ConsoleError(format!("failed to wait {}: {}", SHELL, err.to_string())))?;
    Ok(())
}
