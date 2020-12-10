use std::net::IpAddr;
use std::process::Command;
use regex::Regex;
use std::fs::{copy, File};
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use dirs;
use glob;


pub fn whoami() -> String {
    // TODO: add windows funcionality
    let whoami = Command::new("whoami").output().expect("failed to get username").stdout;
    let whoami = String::from_utf8(whoami).unwrap();
    let whoami = whoami.trim_end_matches("\n");
    whoami.to_string()
}

pub fn ip_get_addr(interface: &str) -> IpAddr {
    // TODO: add Windows funcionality
    let re = Regex::new(r#"inet\s([0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3})"#).unwrap();
    let ipaddr = if cfg!(target_os = "linux") {
        Command::new("ip").args(&["a", "s", interface]).output()
    } else if cfg!(target_os = "macos") {
        Command::new("ifconfig").arg(interface).output()
    } else {
        unimplemented!()
    };
    let ipaddr = ipaddr.expect("failed to get ip address").stdout;
    let ipaddr = String::from_utf8(ipaddr).unwrap();
    let ipaddr = re.captures(&ipaddr).expect("failed to retrieve IP addr")[1].to_string();
    let ipaddr = ipaddr.parse::<IpAddr>().unwrap();
    ipaddr
}

/**
 * Get current timestamp as seconds.
 */
pub fn timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/**
 * Read file `filepath` from position `from`
 * and return contents readed.
 * Return some io::Error if present.
 */
pub fn file_read_from<P: AsRef<Path>>(
    filepath: P,
    from: u64,
) -> io::Result<String> {
    let mut contents = String::new();

    let mut file = File::open(&filepath)?;
    let len = file.metadata().unwrap().len();
    if len < from {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Cannot read file {} from position {}: file lenght is {}",
                &filepath.as_ref().display(),
                from,
                len
            ),
        ));
    }
    file.seek(io::SeekFrom::Start(from)).unwrap();
    file.read_to_string(& mut contents)? as u64;
    Ok(contents)
}

/**
 * Copy file `filepath` to a temporary file and return path ot the copy.
 * Return some io::Error if present.
 */
pub fn file_copy<P: AsRef<Path>>(filepath: P) -> io::Result<PathBuf> {
    if !filepath.as_ref().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} isn't a file", filepath.as_ref().display()),
        ));
    }
    let tmpfile = tempfile::NamedTempFile::new()?.path().to_path_buf();
    copy(filepath, &tmpfile)?;
    Ok(tmpfile)
}

/**
 * Expand path: replace character "~" in `path` with current home directory.
 * Return an io::Error if cannot find home directory.
 */
pub fn path_expand<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    if !path.as_ref().to_str().unwrap().contains("~") {
        return Ok(path.as_ref().to_path_buf());
    }
    return match dirs::home_dir() {
        Some(home) => {
            let home = home.to_str().unwrap();
            let expanded = path.as_ref().to_str().unwrap().replace("~", home);
            Ok(PathBuf::from(expanded))
        }
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Home directory not founded, check if $HOME is set. Path {} not expanded.",
                path.as_ref().display()
            ),
        )),
    };
}

/**
 *  Resolve path: replace '*' in `path` with first file founded and return it.
 *  Return an Error if there's a problem with path parsing.
 */
pub fn path_resolve<P: AsRef<Path>>(path: P) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut paths = glob::glob(&path.as_ref().to_str().unwrap())?.filter_map(|p| {
        return match p {
            Ok(entry) => Some(entry),
            Err(_) => None,
        };
    });
    return match paths.next() {
        Some(entry) => Ok(entry),
        None => Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            format!("No file founded in {}", path.as_ref().display()),
        ))),
    };
}

/**
 * Expand and resolve path `path` and return it.
 */
pub fn path_expand_and_resolve(path: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let expanded = path_expand(path)?;
    let resolved = path_resolve(&expanded)?;
    Ok(resolved)
}
