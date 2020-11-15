use std::io::prelude::*;
use std::fs;

enum LogLevel {
    Info,
    Error
}

static LOGFILE: &str = "/tmp/tracker.log";

fn log_msg(level: LogLevel, msg: &str) {
    let msg = match level {
        LogLevel::Info => format!("INFO: {}", msg),
        LogLevel::Error => format!("ERROR: {}", msg),
    };
    if let Ok(mut file) = fs::File::create(LOGFILE) {
        writeln!(&mut file, "{}", msg).unwrap();
    }
}

pub fn log_error(msg: &str) {
    log_msg(LogLevel::Error, &msg);
}

pub fn log_info(msg: &str) {
    log_msg(LogLevel::Info, &msg);
}