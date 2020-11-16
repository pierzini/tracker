use super::*;

use std::io::prelude::*;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
enum ErrorKind {
    FieldNotPresent(String),
    BadLine(usize, String),
    BadField(usize, String, String),
}

#[derive(Debug, Clone)]
struct ParsingError {
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
            ErrorKind::FieldNotPresent(field) => format!(
                "{} not present", field
            ),
            ErrorKind::BadLine(pos, line) => format!(
                "bad line at position {}: {}", pos, line
            ),
            ErrorKind::BadField(pos, field, value) => format!(
                "failed to parse {} at position {}: {}", field, pos, value
            ),
        };
        write!(f, "failed to read configuration file: {}", errmsg)
    }
}

impl std::error::Error for ParsingError {}


fn parse_host(line: &str) -> Result<String, ParsingError> {
    let re = Regex::new(r#"^host:\s+([.0-9]+)$"#).unwrap();
    if re.is_match(line) {

    }

    Ok(String::new())
}

pub fn parse_cfg_file<P: AsRef<Path>>(
    file: P,
) -> Result<ESConfig, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut host = String::new();
    let mut port = String::new();
    let mut index = String::new();

    for (pos, line) in contents.lines().enumerate() {
            /* empty line */
            if line == "" {
                continue;
            }

            /* comment */
            if line.starts_with("#") {
                continue;
            }

            /* host */
            else if line.starts_with("host:") {
                match Regex::new(r#"^host:\s+([.0-9]+)$"#)
                    .unwrap()
                    .captures(line)
                {
                    Some(h) => {
                        match h[1].to_string().parse::<std::net::IpAddr>() {
                            Ok(addr) => host = addr.to_string(),
                            Err(err) =>  return Err(Box::new(ParsingError::new(
                                            ErrorKind::BadField(
                                                pos,
                                                "host".to_string(),
                                                err.to_string()
                                            )
                                    )
                                )),
                        }
                    },
                    None => return Err(Box::new(ParsingError::new(
                                ErrorKind::BadField(
                                    pos,
                                    "host".to_string(),
                                    line.to_string()
                                )
                            )
                        )),
                };
            }

            /* port */
            else if line.starts_with("port:") {
                /* todo: port parsing, set number of digit to 4 */
                match Regex::new(r#"^port:\s+(\d+)$"#).unwrap().captures(line) {
                    Some(p) => port = p[1].to_string(),
                    None =>  return Err(Box::new(ParsingError::new(
                        ErrorKind::BadField(
                            pos,
                            "port".to_string(),
                            line.to_string())
                    ))),
                };
            }
            
            /* index */
            else if line.starts_with("index:") {
                /* todo: index parsing, check if index is a valid index */
                match Regex::new(r#"^index:\s+(.*)$"#).unwrap().captures(line) {
                    Some(i) => index = i[1].to_string(),
                    None => return Err(Box::new(ParsingError::new(
                        ErrorKind::BadField(
                            pos,
                            "index".to_string(),
                            line.to_string()
                        )
                    )))
                };
            }
            
            /* bad line, return error */
            else {
                return Err(Box::new(ParsingError::new(
                    ErrorKind::BadLine(pos, line.to_string())
                )));
            }
        }

    if host.len() == 0 {
        return Err(Box::new(ParsingError::new(
            ErrorKind::FieldNotPresent("host".to_string())
        )));
    }
 
    if port.len() == 0 {
        return Err(Box::new(ParsingError::new(
            ErrorKind::FieldNotPresent("port".to_string())
        )));
    }

    if index.len() == 0 {
        return Err(Box::new(ParsingError::new(
            ErrorKind::FieldNotPresent("index".to_string())
        )));
    }

    Ok(ESConfig::new(
        &host,
        &port,
        &index,
    ))
}