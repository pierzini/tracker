use std::fs::read_to_string;
use std::path::Path;

use elasticsearch::http::request::JsonBody;
use elasticsearch::http::transport::Transport;
use elasticsearch::{BulkParts, Elasticsearch};
use tokio;

use regex::Regex;

#[derive(Debug, Clone)]
struct ParsingError(String);
impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ParsingError {}

pub struct ESConfig {
    host: String,
    port: String,
    index: String,
}

pub struct ESIndex {
    config: ESConfig,
    client: Elasticsearch,
}

impl ESConfig {
    pub fn new(host: &str, port: &str, index: &str) -> ESConfig {
        /* todo: only if verbose */
        println!(
            "ES configuration: host {}, port {}, index {}",
            host, port, index
        );
        ESConfig {
            host: host.to_string(),
            port: port.to_string(),
            index: index.to_string(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(filename: P) -> Result<ESConfig, Box<dyn std::error::Error>> {
        let errmsg = "bad line at position";
        let (mut host, mut port, mut index) = (None, None, None);

        /* file doesn't exists, return error */
        let contents = read_to_string(filename)?;

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
                /* todo: host parsing */
                host = match Regex::new(r#"^host:\s+([a-zA-z.0-9]+)$"#)
                    .unwrap()
                    .captures(line)
                {
                    Some(h) => Some(h[1].to_string()),
                    None => {
                        return Err(Box::new(ParsingError(format!(
                            "{} {}: {}",
                            errmsg,
                            pos + 1,
                            line
                        ))))
                    }
                };
            }
            /* port */
            else if line.starts_with("port:") {
                /* todo: port parsing, set number of digit to 4 */
                port = match Regex::new(r#"^port:\s+(\d+)$"#).unwrap().captures(line) {
                    Some(p) => Some(p[1].to_string()),
                    None => {
                        return Err(Box::new(ParsingError(format!(
                            "{} {}: {}",
                            errmsg,
                            pos + 1,
                            line
                        ))))
                    }
                };
            }
            /* index */
            else if line.starts_with("index:") {
                /* todo: index parsing, check if index is a valid index */
                index = match Regex::new(r#"^index:\s+(.*)$"#).unwrap().captures(line) {
                    Some(i) => Some(i[1].to_string()),
                    None => {
                        return Err(Box::new(ParsingError(format!(
                            "{} {}: {}",
                            errmsg,
                            pos + 1,
                            line
                        ))))
                    }
                };
            }
            /* bad line, return error */
            else {
                return Err(Box::new(ParsingError(format!(
                    "{} {}: {}",
                    errmsg, pos, line
                ))));
            }
        }

        if host.is_none() {
            return Err(Box::new(ParsingError(String::from("host not present"))));
        }

        if port.is_none() {
            return Err(Box::new(ParsingError(String::from("port not present"))));
        }

        if index.is_none() {
            return Err(Box::new(ParsingError(String::from("index not present"))));
        }

        Ok(ESConfig::new(
            &host.unwrap(),
            &port.unwrap(),
            &index.unwrap(),
        ))
    }
}

impl ESIndex {
    pub fn new(config: ESConfig) -> ESIndex {
        /* todo: check if is a valid ES connection */
        let url = format!("http://{}:{}", &config.host, &config.port);
        let transport = Transport::single_node(&url).expect("failed to get ES transport");
        let client = Elasticsearch::new(transport);
        ESIndex { config, client }
    }

    #[tokio::main]
    pub async fn bulk_import(
        &self,
        records: Vec<serde_json::Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        /* prepare records */
        let mut body: Vec<JsonBody<_>> = vec![];
        for record in records {
            body.push(serde_json::json!({"index": {}}).into());
            body.push(record.into());
        }

        let response = self
            .client
            .bulk(BulkParts::Index(&self.config.index))
            .body(body)
            .send()
            .await?;

        return match response.status_code().is_success() {
            true => Ok(()),
            false => {
                // todo: return error here!
                Ok(())
            }
        };
    }
}
