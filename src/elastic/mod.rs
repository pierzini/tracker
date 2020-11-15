use std::path::Path;

use elasticsearch::http::request::JsonBody;
use elasticsearch::http::transport::Transport;
use elasticsearch::{BulkParts, Elasticsearch};
use tokio;

use regex::Regex;

mod parser;

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
            "[*] ElastcSearch configuration: host {}, port {}, index {}",
            host, port, index
        );
        
        ESConfig {
            host: host.to_string(),
            port: port.to_string(),
            index: index.to_string(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(
        filepath: P,
    ) -> Result<ESConfig, Box<dyn std::error::Error>> {
        parser::parse_cfg_file(filepath)
    }
}

impl ESIndex {
    pub fn new(
        config: ESConfig,
    ) -> Result<ESIndex, Box<dyn std::error::Error>> {
        /* todo: check if is a valid ES connection */
        let url = format!("http://{}:{}", &config.host, &config.port);
        let transport = Transport::single_node(&url)?;
        let client = Elasticsearch::new(transport);
        Ok(ESIndex { config, client })
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
