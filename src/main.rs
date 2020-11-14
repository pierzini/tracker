use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use tracker::browser::*;
use tracker::console::*;
use tracker::*;

fn main() {
    /* parse args */
    let args = clap::App::new("tracker")
        .version("1")
        .author("Pier Paolo Zini")
        .arg(
            clap::Arg::with_name("host")
                .long("host")
                .short("h")
                .number_of_values(1)
                .default_value("localhost"),
        )
        .arg(
            clap::Arg::with_name("port")
                .long("port")
                .short("p")
                .number_of_values(1)
                .default_value("9200"),
        )
        .arg(
            clap::Arg::with_name("index")
                .long("index")
                .short("i")
                .number_of_values(1)
                .default_value("tracker"),
        )
        .arg(
            clap::Arg::with_name("configuration")
                .short("f")
                .long("file")
                .number_of_values(1)
                .conflicts_with_all(&["host", "port", "index"]),
        )
        .get_matches();

    /* get ElasticSearch configuration and client */
    let es_config = if args.is_present("configuration") {
        let es_config_file = args.value_of("configuration").unwrap();
        elastic::ESConfig::from_file(&es_config_file).unwrap_or_else(|err| {
            let errmsg = err.to_string();
            eprintln!(
                "failed to read ES configuration file {}: {}",
                &es_config_file,
                errmsg.to_string()
            );
            std::process::exit(1);
        })
    } else {
        let host = args.value_of("host").unwrap().to_string();
        let port = args.value_of("port").unwrap().to_string();
        let index = args.value_of("index").unwrap().to_string();
        elastic::ESConfig::new(&host, &port, &index)
    };
    let es_index = Arc::new(Mutex::new(elastic::ESIndex::new(es_config)));

    let mut firefox_history = BrowserHistControl::new(Browser::Firefox, BrowserHistFrom::Now)
        .unwrap_or_else(|err| {
            eprintln!("browser database not founded: {}", err);
            std::process::exit(1);
        });

    let pid = std::process::id();
    let mut shell_hist = HistState::new(pid).unwrap_or_else(|err| {
        eprintln!("error: {}", err);
        std::process::exit(1);
    });

    let mut runner = Runner::new();

    let t_es_index = Arc::clone(&es_index);
    runner.start_loop("browser_hist_dump", move || {
        if let Some(records) = firefox_history.dump() {
            t_es_index.lock().unwrap().bulk_import(records).unwrap();
        }
        thread::sleep(time::Duration::from_millis(500));
    });

    let t_es_index = Arc::clone(&es_index);
    runner.start_loop("shell_hist_dump", move || {
        if let Some(records) = shell_hist.dump() {
            t_es_index.lock().unwrap().bulk_import(records).unwrap();
        }
        thread::sleep(time::Duration::from_millis(500));
    });

    let mut shell = start_console().unwrap_or_else(|err| {
        eprintln!("error: {}", err);
        std::process::exit(1);
    });
    shell.wait().expect("error: failed to wait shell");
    println!("exit...");
}
