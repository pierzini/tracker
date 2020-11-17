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

    let es_config = ESConfig::new(cli.host, cli.port, &cli.index);
    let es_client = ESClient::new(es_config);

    // browser history control
    let mut b_history =
        BrowserHistControl::new(cli.browser, BrowserHistFrom::Now).unwrap_or_else(|e| {
            eprintln!("ERR: {}", e.to_string());
            std::process::exit(1);
        });
    // shell history control
    let mut c_history = ConsoleHistControl::new();

    // start bash
    let pid = std::process::id();
    let console = start_console(pid, &mut c_history).unwrap_or_else(|err| {
        eprintln!("ERR: {}", err.to_string());
        std::process::exit(1);
    });

    // threads: dump browser and console history
    let mut runner = Runner::new();
    let es_client = Arc::new(Mutex::new(es_client));
    let async_esclient = Arc::clone(&es_client);
    runner.start_loop(move || {
        if let Some(records) = b_history.dump() {
            let _ = async_esclient.lock().unwrap().bulk_import(records);
        }
        thread::sleep(time::Duration::from_millis(500));
    });
    let async_esclient = Arc::clone(&es_client);
    runner.start_loop(move || {
        if let Some(records) = c_history.dump() {
            let _ = async_esclient.lock().unwrap().bulk_import(records);
        }
        thread::sleep(time::Duration::from_millis(500));
    });

    // main thread: wait shell
    console.join().expect("ERR: failed to wait console");
    println!("exit...");
}
