use std::fmt;
use std::fs::read_to_string;
use std::net::IpAddr;
use std::path::Path;

use clap;
use regex;

use crate::browser::Browser;

#[allow(dead_code)]
const TRACKER_CONF: &str = "/etc/tracker.conf";  // todo: use this as default cfg file

#[derive(Clone, Debug)]
pub struct CliError(String);

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CliError {}


#[derive(Clone, Debug)]
pub struct Cli {
    pub host: IpAddr,
    pub port: u64,
    pub index: String,
    pub browser: Browser,
}

impl Cli {
    pub fn new() -> Result<Cli, CliError> {
        let args = clap::App::new("tracker")
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
                    .number_of_values(1),
            )
            .arg(
                clap::Arg::with_name("browser")
                    .long("browser")
                    .short("b")
                    .number_of_values(1)
                    .default_value("firefox")
                    .validator(|arg| {
                        if !arg.eq("firefox") && !arg.eq("firefox-esr") && !arg.eq("chrome") {
                            return Err(format!(
                                "browser {} not valid. Valid browser are firefox, chrome or firefox-esr",
                                arg
                            ));
                        } else {
                            return Ok(())
                        }
                    }),
            )
            .arg(
                clap::Arg::with_name("cfgfile")
                    .long("file")
                    .short("f")
                    .number_of_values(1)
                    .conflicts_with_all(&["host", "port", "index", "browser"]),
            )
            .get_matches();

        // load from cfg file
        if args.is_present("cfgfile") {
            let cfgfile = args.value_of("cfgfile").unwrap();
            return load_cfg_file(cfgfile);
        }

        // get host
        let host: IpAddr;
        let s_host = args.value_of("host").unwrap();
        match check_host(s_host) {
            Some(addr) => host = addr,
            None => return Err(CliError("host not valid.".to_string())),
        }

        // get port
        let port: u64;
        let s_port = args.value_of("port").unwrap();
        match check_port(s_port) {
            Some(p) => port = p,
            None => return Err(CliError("port not valid.".to_string())),
        }

        // get idx
        let index: String;
        match args.value_of("index") {
            Some(idx) => index = idx.to_owned(),
            None => return Err(CliError("plase specify an index name.".to_string())),
        }

        // get browser
        let browser: Browser;
        let s_browser = args.value_of("browser").unwrap();
        match check_browser(s_browser) {
            Some(b) => browser = b,
            None => return Err(CliError("browser not valid.".to_string())),
        }

        Ok(Cli {
            host,
            port,
            index,
            browser,
        })
    }
}

fn check_host(host: &str) -> Option<IpAddr> {
    if host.eq("localhost") {
        return Some("127.0.0.1".parse::<IpAddr>().unwrap());
    }
    return match host.parse::<IpAddr>() {
        Ok(addr) => Some(addr),
        Err(_) => None
    }
}

fn check_port(port: &str) -> Option<u64> {
    return match port.parse::<u64>() {
        Ok(p) => Some(p),
        Err(_) => None
    }
}

fn check_browser(browser: &str) -> Option<Browser> {
    return match browser {
        "chrome" => Some(Browser::Chrome),
        "firefox" => Some(Browser::Firefox),
        "firefox-esr" => unimplemented!(),
        _ => None,
    }
}

fn load_host(line: &str) -> Option<IpAddr> {
    let re = regex::Regex::new(r#"^host:\s+([.0-9a-z]+)$"#).unwrap();
    if let Some(host) = re.captures(line) {
        return check_host(&host[1]);
    }

    None
}

fn load_port(line: &str) -> Option<u64> {
    let re = regex::Regex::new(r#"^port:\s+(\d+)$"#).unwrap();
    if let Some(port) = re.captures(line) {
        return check_port(&port[1]);
    }

    None
}

fn load_index(line: &str) -> Option<String> {
    let re = regex::Regex::new(r#"^index:\s+(.*)$"#).unwrap();
    return match re.captures(line) {
        Some(index) => Some(index[1].to_string()),
        None => None,
    }
}

fn load_browser(line: &str) -> Option<Browser> {
    let re = regex::Regex::new(r#"^browser:\s+(.*)$"#).unwrap();
    if let Some(browser) = re.captures(line) {
        return check_browser(&browser[1]);
    }

    None
}


fn load_cfg_file<P: AsRef<Path>>(filepath: P) -> Result<Cli, CliError> {
    let mut host: Option<IpAddr> = None;
    let mut port: Option<u64> = None;
    let mut index: Option<String> = None;
    let mut browser: Option<Browser> = None;
    let errmsg = "failed to read cfg file:";

    if ! filepath.as_ref().is_file() {
        return Err(CliError(format!(
            "path {} does not contain a file.", filepath.as_ref().display()))
        );
    }

    let contents: String;
    match read_to_string(&filepath) {
        Ok(c) => contents = c,
        Err(err) => return Err(CliError(format!(
            "failed to read file {}: {}.", filepath.as_ref().display(), err.to_string()
        )))
    }

    for (pos, line) in contents.lines().enumerate() {
        if line == "" {
            continue;
        }

        else if line.starts_with("#") {
            continue;
        }

        else if line.starts_with("host:") {
            match load_host(line) {
                Some(h) => host = Some(h),
                None => return Err(CliError(format!(
                    "{} bad host at position {}: {}", errmsg, pos, line
                ))),
            }
        }

        else if line.starts_with("port:") {
            match load_port(line) {
                Some(p) => port = Some(p),
                None => return Err(CliError(format!(
                    "{} bad port at position {}: {}", errmsg, pos, line
                ))),
            }
        }

        else if line.starts_with("index:") {
            match load_index(line) {
                Some(idx) => index = Some(idx),
                None => return Err(CliError(format!(
                    "{} bad index at position {}: {}", errmsg, pos, line
                ))),
            }
        }

        else if line.starts_with("browser:") {
            match load_browser(line) {
                Some(b) => browser = Some(b),
                None => return Err(CliError(format!(
                    "{} bad browser at position {}: {}", errmsg, pos, line
                ))),
            }
        }

        else {
            return Err(CliError(format!(
                "bad line at position {}: {}", pos, line
            )))
        }
    }

    if host.is_none() {
        // return Err(CliError(format!("{} host not present", errmsg)))
        host = Some("127.0.0.1".parse::<IpAddr>().unwrap());
    }

    if port.is_none() {
        // return Err(CliError(format!("{} port not present", errmsg)))
        port = Some(9200);
    }

    if index.is_none() {
        return Err(CliError(format!("{} index not present", errmsg)))
    }

    if browser.is_none() {
        browser = Some(Browser::Firefox);
        // return Err(CliError(format!("{} browser not present", errmsg)))
    }

    Ok(Cli {
        host: host.unwrap(),
        port: port.unwrap(),
        index: index.unwrap(),
        browser: browser.unwrap(),
    })
}
