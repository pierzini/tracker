use crate::utils::*;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::{Child, Command};

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
const TRACKER_INIT: &str = "~/.tracker/startup-files/.tracker.rc";
#[cfg(target_os = "windows")]
const TRACKER_INIT: &str = "~\\AppData\\Local\\tracker\\startup-files\\tracker.ps1";

#[cfg(not(target_os = "windows"))]
const SHELL: &str = "/bin/bash";
#[cfg(target_os = "windows")]
const SHELL: &str = "powershell";

#[derive(Debug, Clone, serde::Serialize)]
pub struct HistEntry {
    pub timestamp: u64,
    #[cfg(not(target_os = "windows"))]
    pub user: String,
    pub cmd: String,
    #[cfg(not(target_os = "windows"))]
    pub status: u64,
    pub output: String,
}

pub struct HistState {
    history: Vec<HistEntry>, /* history themself */
    histfile: PathBuf,       /* where the history is stored */
    offset: u64,             /* location within this history */
    length: u64,             /* length of the histfile */
}

impl HistState {
    pub fn new(pid: u32) -> Result<HistState, Box<dyn std::error::Error>> {
        let histfile = format!(
            "{}/hist.{}.log", TRACKER_LOGS_DIR, pid.to_string()
        );
        
        let histfile = path_expand(&histfile)?;
        let offset = 0;
        let length: u64;
        match std::fs::metadata(&histfile) {
            Ok(metadata) => length = metadata.len(),
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    length = 0;
                } else {
                    return Err(Box::new(err));
                }
            }
        };
        let history = Vec::new();
        Ok(HistState {
            histfile,
            offset,
            history,
            length,
        })
    }

    pub fn update(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        self.length = std::fs::metadata(&self.histfile)?.len();

        if self.length == self.offset {
            return Ok(0);
        }

        let contents = file_read_from(&self.histfile, self.offset)?;
        let history = parser::parse_histfile_contents(&contents)?;
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
        self.offset = 0;
    }

    pub fn history(&self) -> &Vec<HistEntry> {
        &self.history
    }
}

/**
 * Start a shell (bash/powershell) with commands history readable by `HistState`.
 */
pub fn start_console() -> Result<Child, Box<dyn std::error::Error>> {
    let init_file = path_expand(TRACKER_INIT)?;
    if !init_file.as_path().exists() {
        return Err(Box::new(
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("init file {} not founded.", init_file.display())
            )
        ));
    }
    /* todo: only verbose */
    println!("[*] starting {} with init file: {}", SHELL, init_file.display());
    
    let console = if cfg!(target_os = "windows") {
        Command::new(SHELL)
            .args(&[
                "-noexit",
                "-command",
                &format!(". {}", init_file.as_path().to_str().unwrap()),
            ])
            .spawn()?
    } else {
        Command::new(SHELL)
            .args(&["--rcfile", init_file.as_path().to_str().unwrap()])
            .spawn()?
    };

    Ok(console)
}
