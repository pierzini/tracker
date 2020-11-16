use std::fmt;
use std::net::IpAddr;

use elasticsearch::http::request::JsonBody;
use elasticsearch::http::transport::Transport;
use elasticsearch::{BulkParts, Elasticsearch};
use tokio;

#[derive(Clone, Debug)]
pub struct ESError(String);

impl fmt::Display for ESError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ElasticSearch error: {}", self.0)
    }
}

impl std::error::Error for ESError {}

#[derive(Debug)]
pub struct ESConfig {
    host: IpAddr,
    port: u64,
    index: String,
}

impl ESConfig {
    pub fn new(host: IpAddr, port: u64, index: &str) -> ESConfig {
        ESConfig {
            host,
            port,
            index: index.to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct ESClient {
    config: ESConfig,      /* ES configuration */
    client: Elasticsearch, /* the client themself */
}

impl ESClient {
    pub fn new(config: ESConfig) -> ESClient {
        let url = format!("http://{}:{}", &config.host.to_string(), &config.port);
        let transport = Transport::single_node(&url).unwrap();
        let client = Elasticsearch::new(transport);
        ESClient { config, client }
    }

    pub fn from(host: IpAddr, port: u64, index: &str) -> ESClient {
        let config = ESConfig::new(host, port, index);
        ESClient::new(config)
    }

    #[tokio::main]
    pub async fn bulk_import(&self, records: Vec<serde_json::Value>) -> Result<(), ESError> {
        let mut body: Vec<JsonBody<_>> = vec![];
        for record in records {
            body.push(serde_json::json!({"index": {}}).into());
            body.push(record.into());
        }

        return match self
            .client
            .bulk(BulkParts::Index(&self.config.index))
            .body(body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status_code().is_success() {
                    Ok(())
                } else {
                    Err(ESError(format!("records not updated: status code: {}", response.status_code())))
                }
            }
            Err(err) => Err(ESError(format!("records not updated: error: {}", err.to_string()))),
        };
    }
}
