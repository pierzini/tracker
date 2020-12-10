use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use tracker::browser::*;
use tracker::cli::*;
use tracker::console::*;
use tracker::elastic::*;
use tracker::utils::*;
use tracker::*;

fn main() {
    let cli = Cli::new().unwrap_or_else(|e| {
        eprintln!("[*] ERR: {}", e.to_string());
        std::process::exit(1);
    });

    let username = Arc::new(Mutex::new(whoami()));
    let ipaddr = Arc::new(Mutex::new(ip_get_addr(&cli.interface).to_string()));

    let es_config = ESConfig::new(cli.host, cli.port, &cli.index);
    let es_client = ESClient::new(es_config);

    // browser history control
    let mut b_history = None;
    if !cli.browser.is_none() {
        b_history = Some(
            BrowserHistControl::new(cli.browser.unwrap(), BrowserHistFrom::Now).unwrap_or_else(
                |e| {
                    eprintln!("[*] ERR: {}", e.to_string());
                    std::process::exit(1);
                },
            ),
        );
        println!("[*] Browser history db founded correctly.")
    }

    // shell history control
    let mut c_history = ConsoleHistControl::new();

    // start bash
    let pid = std::process::id();
    println!("[*] Starting /bin/bash.");
    println!("ATTENTION: Output is redirected.");
    let console = start_console(pid, &mut c_history).unwrap_or_else(|err| {
        eprintln!("[*] ERR: {}.", err.to_string());
        std::process::exit(1);
    });

    // threads: dump browser and console history
    let mut runner = Runner::new();
    let es_client = Arc::new(Mutex::new(es_client));
    if !b_history.is_none() {
        let async_esclient = Arc::clone(&es_client);
        let async_username = Arc::clone(&username);
        let async_ipaddr = Arc::clone(&ipaddr);
        let mut dumper = b_history.unwrap();
        runner.start_loop(move || {
            if let Some(mut records) = dumper.dump() {
                update_records(&mut records, &async_username.lock().unwrap(), &async_ipaddr.lock().unwrap());
                let _ = async_esclient.lock().unwrap().bulk_import(records);
            }
            thread::sleep(time::Duration::from_millis(500));
        });
    }

    let async_esclient = Arc::clone(&es_client);
    let async_username = Arc::clone(&username);
    let async_ipaddr = Arc::clone(&ipaddr);
    runner.start_loop(move || {
        if let Some(mut records) = c_history.dump() {
            update_records(&mut records, &async_username.lock().unwrap(), &async_ipaddr.lock().unwrap());
            let _ = async_esclient.lock().unwrap().bulk_import(records);
        }
        thread::sleep(time::Duration::from_millis(500));
    });

    // main thread: wait shell
    console.join().expect("[*] ERR: failed to wait console");
    println!("[*] Exit...");
}


fn update_records(records: &mut Vec<serde_json::value::Value>, username: &str, ip: &str) {
    for record in records {
        match record {
            serde_json::value::Value::Object(o) => {
                o.insert(
                    "user.name".to_string(),
                    serde_json::json!(username),
                );
                o.insert(
                    "host.ip".to_string(),
                    serde_json::json!(ip)
                );
            },
            _ => {}
        }
    }
}
