use super::*;
use regex::Regex;
use std::fs::File;

#[derive(Clone, Debug)]
pub enum ErrorKind {
    BadLine(usize, String),
    OutfileNotAvailable(String, String),
}

#[derive(Clone, Debug)]
pub struct ParsingError {
    // todo: add filename: String,
    kind: ErrorKind,
}

impl ParsingError {
    pub fn new(kind: ErrorKind) -> ParsingError {
        ParsingError { kind }
    }
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let errmsg = match &self.kind {
            ErrorKind::BadLine(pos, line) => {
                format!("bad line at position {} : {}.", pos, line)
            },
            ErrorKind::OutfileNotAvailable(outfile, err) => {
                format!("output file {} not available: {}.", outfile, err)
            }
        };
        write!(f, "{}", errmsg)
    }
}

impl std::error::Error for ParsingError {}

type Result<T> = std::result::Result<T, ParsingError>;

lazy_static! {
    static ref RE: Regex =
        Regex::new(r#"status="(\d+)"\tuser="(.*)"\ttimestamp="(\d+)"\tcmd="(.*)"\toutfile="(.*)""#)
            .unwrap();
}

fn parse_line(n: usize, line: &str) -> Result<ConsoleHistEntry> {
    let captures: regex::Captures;
    match RE.captures(line) {
        Some(c) => captures = c,
        None => return Err(
            ParsingError::new(ErrorKind::BadLine(n, line.to_owned()))
        ),
    }

    let status = captures[1].parse::<u64>().unwrap();
    let user = captures[2].to_string();
    let timestamp = captures[3].parse::<u64>().unwrap();
    let cmd = captures[4].to_string();
    let outfile = captures[5].to_string();

    let output: String;
    match File::open(&outfile) {
        Ok(mut file) => {
            let mut buf = Vec::new();
            match file.read_to_end(&mut buf) {
                Ok(_) => output = String::from_utf8_lossy(&buf).to_string(),
                Err(err) => {
                    return Err(ParsingError::new(ErrorKind::OutfileNotAvailable(
                        outfile,
                        err.to_string(),
                    )))
                }
            }
        }
        Err(err) => {
            return Err(ParsingError::new(ErrorKind::OutfileNotAvailable(
                outfile,
                err.to_string(),
            )))
        }
    };

    Ok(ConsoleHistEntry {
        user,
        status,
        timestamp,
        cmd,
        output,
    })
}

pub fn parse_histfile_contents(contents: &str) -> Result<Vec<ConsoleHistEntry>> {
    let mut records: Vec<ConsoleHistEntry> = Vec::new();
    for (pos, line) in contents.lines().enumerate() {
        let record = parse_line(pos, line)?;
        records.push(record);
    }
    Ok(records)
}

#[cfg(test)]
mod test {
    // ...
}
