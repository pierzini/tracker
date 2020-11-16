#[macro_use]
extern crate lazy_static;

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub mod browser;
pub mod console;
pub mod elastic;
pub mod utils;
pub mod log;

pub trait JsonDumper {
    fn dump(&mut self) -> Option<Vec<serde_json::Value>>;
}

impl JsonDumper for browser::BrowserHistControl {

    fn dump(&mut self) -> Option<Vec<serde_json::Value>> {
        return match self.update() {
            Ok(n) => {
                if n == 0 {
                    return None;
                }
                let mut json_records = Vec::with_capacity(n);

                let history = self.history().clone();
                self.clear();

                for entry in history {
                    let mut json_value = serde_json::Map::new();
                    json_value.insert(
                        "@timestamp".to_string(),
                        serde_json::json!(entry.timestamp)
                    );
                    json_value.insert(
                        "url.full".to_string(),
                        serde_json::json!(entry.url)
                    );
                    json_value.insert(
                        "url.visit_count".to_string(),
                        serde_json::json!(entry.visit_count)
                    );
                    let json_value = serde_json::to_value(json_value).unwrap();

                    log::log_info(
                        &format!("dumped browser history: {}\t{}",
                            entry.timestamp, entry.url)
                    );
                    
                    json_records.push(json_value);

                }
                
                Some(json_records)
            }
            Err(_) => {
                // todo: log error (problem)
                None
            },
        };
    }
}

impl JsonDumper for console::HistState {

    fn dump(&mut self) -> Option<Vec<serde_json::Value>> {
        return match self.update() {
            Ok(n) => {

                if n == 0 {
                    return None;
                }

                let mut json_records = Vec::with_capacity(n);

                let history = self.history().clone();
                self.clear();

                for entry in &history {
                    let mut json_value = serde_json::Map::new();
                    json_value.insert(
                        "@timestamp".to_string(),
                        serde_json::json!(entry.timestamp)
                    );
                    json_value.insert(
                        "user.name".to_string(),
                        serde_json::json!(entry.user)
                    );
                    json_value.insert(
                        "process.command_line".to_string(),
                        serde_json::json!(entry.cmd),
                    );
                    json_value.insert(
                        "process.exit_code".to_string(),
                        serde_json::json!(entry.status),
                    );
                    json_value.insert(
                        "process.output".to_string(),
                        serde_json::json!(entry.output),
                    );

                    let json_value = serde_json::to_value(json_value).unwrap();

                    log::log_info(
                        &format!("dumped console history: {}\t{}",
                            entry.timestamp, entry.cmd)
                    );

                    json_records.push(json_value);
                }
                Some(json_records)
            }
            Err(_) => {
                // todo: log error (problem)
                None
            },
        };
    }
}

enum Message {
    Terminate,
}

pub struct Runner {
    workers: Vec<Option<thread::JoinHandle<()>>>,
    sender: mpsc::Sender<Message>,
    receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
}

impl Runner {
    pub fn new() -> Runner {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let workers = Vec::new();
        Runner {
            workers,
            sender,
            receiver,
        }
    }

    pub fn start_loop<F>(&mut self, mut job: F)
    where
        F: FnMut() + Send + 'static,
    {
        let id = self.workers.len();
        let rx = Arc::clone(&self.receiver);
        let thread = thread::spawn(move || loop {
            job();
            if let Ok(Message::Terminate) = rx.lock().unwrap().try_recv() {
                println!("[*] terminating thread ID {}", id);
                break;
            }
        });

        self.workers.push(Some(thread));
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(t) = worker.take() {
                t.join().unwrap();
            }
        }
    }
}
