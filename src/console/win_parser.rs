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
