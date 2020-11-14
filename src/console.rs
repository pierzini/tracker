use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use crate::utils::*;

#[cfg(not(target_os = "windows"))]
const TRACKER_LOGS_DIR: &str = "~/.tracker/histlogs";
#[cfg(target_os = "windows")]
const TRACKER_LOGS_DIR: &str = "~\\AppData\\Local\\tracker\\histlogs";

#[cfg(not(target_os = "windows"))]
const TRACKER_INIT: &str = "~/.tracker/startup-files/tracker.rc";
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
        let histfile = format!("{}/hist.{}.log", TRACKER_LOGS_DIR, pid.to_string());
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

        let mut contents: String = String::new();
        self.offset = file_read_from(&self.histfile, self.offset, &mut contents)?;

        let history: Vec<HistEntry> = parser::parse_histfile(&contents)?;
        let n = history.len();

        self.history.extend(history);
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
    println!("start console with init file: {}", init_file.display());
    let console = if cfg!(target_os = "windows") {
        Command::new(SHELL)
            .args(&[
                "-noexit",
                "-command",
                &format!(". {}", init_file.as_path().to_str().unwrap()),
            ])
            .spawn()
            .expect("failed to run console")
    } else {
        Command::new(SHELL)
            .args(&["--rcfile", init_file.as_path().to_str().unwrap()])
            .spawn()
            .expect("failed to run console")
    };

    Ok(console)
}

#[cfg(not(target_os = "windows"))]
mod parser {
    use super::*;

    use regex::Regex;
    use std::fs::File;
    use std::io::Read;

    #[derive(Debug, Clone)]
    struct ParsingError(String);
    impl std::fmt::Display for ParsingError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for ParsingError {}

    lazy_static! {
        static ref RE: Regex = Regex::new(
            r#"status="(\d+)"\tuser="(.*)"\ttimestamp="(\d+)"\tcmd="(.*)"\toutfile="(.*)""#
        )
        .unwrap();
    }

    fn read_cmd_output<P: AsRef<Path>>(filepath: P) -> Result<String, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        let mut file = File::open(&filepath)?;
        file.read_to_end(&mut buf).unwrap();
        let output = String::from_utf8_lossy(&buf).to_string();
        Ok(output)
    }

    fn parse_record(line: &str) -> Result<HistEntry, Box<dyn std::error::Error>> {
        let captures: regex::Captures;

        match RE.captures(line) {
            Some(c) => {
                captures = c;
            }
            None => {
                let errmsg = "failed to match record fields";
                return Err(Box::new(ParsingError(errmsg.to_string())));
            }
        }

        let status: u64;
        match captures[1].parse::<u64>() {
            Ok(s) => status = s,
            Err(err) => {
                let errmgs = format!("failed to parse command exit status: {}", err.to_string());
                return Err(Box::new(ParsingError(errmgs)));
            }
        }

        let user: String = captures[2].to_string();

        let timestamp: u64;
        match captures[3].parse::<u64>() {
            Ok(t) => timestamp = t,
            Err(err) => {
                let errmgs = format!("failed to parse command timestamp: {}", err.to_string());
                return Err(Box::new(ParsingError(errmgs)));
            }
        }

        let cmd: String = captures[4].to_string();

        let outfile: String = captures[5].to_string();
        let output: String;
        match read_cmd_output(&outfile) {
            Ok(out) => output = out,
            Err(err) => {
                let errmsg = format!("failed to parse command output: {}", err.to_string());
                return Err(Box::new(ParsingError(errmsg)));
            }
        }

        Ok(HistEntry {
            user,
            status,
            timestamp,
            cmd,
            output,
        })
    }
    /* TODO: accept (seeked) file instead of contents */
    pub fn parse_histfile(contents: &str) -> Result<Vec<HistEntry>, Box<dyn std::error::Error>> {
        let mut records = vec![];
        for (pos, line) in contents.lines().enumerate() {
            match parse_record(line) {
                Ok(record) => records.push(record),
                Err(err) => {
                    let errmsg = format!(
                        "failed to parse line {} at position {}. Error: {}",
                        &line,
                        pos + 1,
                        err.to_string()
                    );
                    /* todo: log error */
                    return Err(Box::new(ParsingError(errmsg)));
                }
            }
        }

        Ok(records)
    }
}

#[cfg(target_os = "windows")]
mod parser {
    use super::*;

    use regex::Regex;
    use std::fs::File;
    use std::io::Read;
    use std::iter::Iterator;

    use chrono::NaiveDateTime;

    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^PS.*>(.*)$"#).unwrap();
    }

    pub fn parse_record(s_record: &str) -> HistEntry {
        let mut lines = s_record
            .lines()
            .filter(|&line| line != "**********************");

        let s_timestamp = lines.next().expect("failed to get record timestamp");

        /* TODO: adjust windows timestamp */
        let timestamp = NaiveDateTime::parse_from_str(&s_timestamp, "%Y%m%d%H%M%S")
            .expect(&format!(
                "failed to parse record timestamp: {}",
                s_timestamp
            ))
            .timestamp() as u64;

        let cmdline = lines.next().expect("failed to get command record line");
        let captures = RE.captures(&cmdline).expect(&format!(
            "failed to match regex record command: {}",
            &cmdline
        ));
        let cmd = captures[1].to_string();

        let output = lines.collect::<Vec<&str>>().join("\n");

        HistEntry {
            timestamp,
            cmd,
            output,
        }
    }

    pub fn parse_histfile(contents: &str) -> Vec<HistEntry> {
        let mut records = vec![];

        if contents == "" {
            return records;
        }

        let mut s_records = contents.split("Command start time: ");

        /* remove header */
        s_records
            .next()
            .expect("failed to remove header (windows parser)");

        for s_record in s_records {
            let record = parse_record(&s_record);
            records.push(record);
        }

        records
    }
}
