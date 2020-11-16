use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use tracker::browser::*;
use tracker::cli::*;
use tracker::console::*;
use tracker::elastic::*;
use tracker::*;

fn main() {
    let cli = Cli::new().unwrap_or_else(|e| {
        eprintln!("ERR: {}", e.to_string());
        std::process::exit(1);
    });

    let es_client = ESClient::from(cli.host, cli.port, cli.index.as_str());
    let es_client = Arc::new(Mutex::new(es_client));

    let mut b_history =
        BrowserHistControl::new(cli.browser, BrowserHistFrom::Now).unwrap_or_else(|e| {
            eprintln!("ERR: {}", e.to_string());
            std::process::exit(1);
        });

    let pid = std::process::id();
    let c_history = ConsoleHistControl::new();
    let c_history = Arc::new(Mutex::new(c_history));

    let mut runner = Runner::new();
    let async_esclient = Arc::clone(&es_client);
    runner.start_loop(move || {
        if let Some(records) = b_history.dump() {
            let _ = async_esclient.lock().unwrap().bulk_import(records);
        }

        thread::sleep(time::Duration::from_millis(500));
    });

    let async_esclient = Arc::clone(&es_client);
    let async_chistory = Arc::clone(&c_history);
    runner.start_loop(move || {
        if let Some(records) = async_chistory.lock().unwrap().dump() {
            let _ = async_esclient.lock().unwrap().bulk_import(records);
        }

        thread::sleep(time::Duration::from_millis(500));
    });

    attach_console(pid, Arc::clone(&c_history)).unwrap_or_else(|err| {
        eprintln!("ERR: {}", err.to_string());
        std::process::exit(1);
    });
    println!("exit...");
}
