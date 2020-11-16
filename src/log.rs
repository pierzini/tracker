use crate::utils::timestamp_now;

use chrono::NaiveDateTime;

use std::io::prelude::*;
use std::fs;

enum LogLevel {
    Info,
    Error
}

const LOGFILE: &str = "/tmp/tracker.log";

fn log_msg(level: LogLevel, msg: &str) {
    let now = timestamp_now() as i64;
    let now = NaiveDateTime::from_timestamp(now, 0);

    let msg = match level {
        LogLevel::Info => format!("{}:INFO: {}", now, msg),
        LogLevel::Error => format!("{}:ERROR: {}", now, msg),
    };
    if let Ok(mut file) = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(LOGFILE) {
        writeln!(&mut file, "{}", msg).expect("failed to write log");
    }

}

pub fn log_error(msg: &str) {
    log_msg(LogLevel::Error, &msg);
}

pub fn log_info(msg: &str) {
    log_msg(LogLevel::Info, &msg);
}
