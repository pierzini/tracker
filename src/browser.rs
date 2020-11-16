use std::fmt;
use std::fs::remove_file;
use std::path::PathBuf;

use crate::utils::*;

#[cfg(target_os = "macos")]
static FIREFOX_DB: &str =
    "~/Library/Application Support/Firefox/Profiles/*default-release/places.sqlite";
#[cfg(target_os = "linux")]
static FIREFOX_DB: &str = "~/.mozilla/firefox/*default-release/places.sqlite";
#[cfg(target_os = "windows")]
static FIREFOX_DB: &str =
    "~\\AppData\\Roaming\\Mozilla\\Firefox\\Profiles\\*default-release\\places.sqlite";
static FIREFOX_QUERY: &str = "\
SELECT CAST(strftime('%s', last_visit_date/1000000, 'unixepoch') AS INTEGER) as
    timestamp,
    url,
    visit_count
FROM moz_places
WHERE last_visit_date IS NOT NULL AND timestamp > ?
ORDER BY timestamp;";

#[cfg(target_os = "macos")]
static CHROME_DB: &str = "~/Library/Application Support/Google/Chrome/Default/History";
#[cfg(target_os = "linux")]
static CHROME_DB: &str = "~/.config/google-chrome/Default/History";
#[cfg(target_os = "windows")]
static CHROME_DB: &str = "";
static CHROME_QUERY: &str = "\
SELECT CAST(strftime('%s', last_visit_time/1000000 - 11644473600, 'unixepoch') AS INTEGER) as
    timestamp,
    url,
    visit_count
FROM urls
WHERE last_visit_time IS NOT NULL AND timestamp > ?
ORDER BY timestamp;";

#[derive(Clone, Debug)]
pub struct BrowserError(String);

impl fmt::Display for BrowserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for BrowserError {}

#[derive(Debug, Clone, serde::Serialize)]
pub enum Browser {
    Firefox,
    FirefoxEsr,
    Chrome,
    // others...
}

pub enum BrowserHistFrom {
    Start,
    Now,
    Timestamp(u64),
}

#[derive(Debug, Clone)]
pub struct BrowserHistEntry {
    pub timestamp: u64,
    pub url: String,
    pub visit_count: usize,
}

#[derive(Clone, Debug)]
pub struct BrowserHistControl {
    history: Vec<BrowserHistEntry>, /* history themself */
    browser: Browser,               /* browser of the managed history */
    database: PathBuf,              /* database of the managed history */
    raw_query: String,              /* raw query of the managed history */
    l_timestamp: u64,               /* last entry timestamp of the managed history */
}

impl BrowserHistControl {
    pub fn new(
        browser: Browser,
        from: BrowserHistFrom,
    ) -> Result<BrowserHistControl, BrowserError> {
        let (database, raw_query) = match browser {
            Browser::Firefox => {
                let database = FIREFOX_DB;
                let query = FIREFOX_QUERY;
                (database, query)
            }
            Browser::FirefoxEsr => unimplemented!(),
            Browser::Chrome => {
                let database = CHROME_DB;
                let query = CHROME_QUERY;
                (database, query)
            }
        };

        let database = path_expand_and_resolve(database).map_err(|err| {
            BrowserError(format!(
                "browser db not founded: {}. Check if {:?} is installed",
                err.to_string(),
                browser
            ))
        })?;

        // todo: only verbose
        println!("[*] browser db founded: {}", &database.display());

        let raw_query = raw_query.to_string();
        let l_timestamp = 0;
        let history = Vec::new();
        let mut ctrl = BrowserHistControl {
            history,
            browser,
            database,
            raw_query,
            l_timestamp,
        };
        ctrl.reset(from);
        Ok(ctrl)
    }

    pub fn update(&mut self) -> Result<usize, BrowserError> {
        let mut history = Vec::new();

        let database = file_copy(&self.database).map_err(|err| {
            BrowserError(format!(
                "failed to copy browser db: {}. Check if {:?} is installed.",
                err.to_string(),
                &self.browser
            ))
        })?;

        let con = sqlite::open(&database).map_err(|err| {
            BrowserError(format!("failed to open browser db: {}", err.to_string()))
        })?;

        let mut stmt = con.prepare(&self.raw_query).unwrap();
        stmt.bind(1, self.l_timestamp.to_string().as_str()).unwrap();

        loop {
            match stmt.next().unwrap() {
                sqlite::State::Row => {
                    let timestamp = stmt.read::<String>(0).unwrap().parse::<u64>().unwrap();
                    let url = stmt.read::<String>(1).unwrap();
                    let visit_count = stmt.read::<String>(2).unwrap().parse::<usize>().unwrap();
                    history.push(BrowserHistEntry {
                        timestamp,
                        url,
                        visit_count,
                    });
                }
                sqlite::State::Done => break,
            }
        }

        remove_file(&database).map_err(|err| {
            BrowserError(format!(
                "failed to remove browser db copy: {}",
                err.to_string()
            ))
        })?;

        let n = history.len();
        if n != 0 {
            self.l_timestamp = history.last().unwrap().timestamp;
            self.history.extend(history);
        }

        Ok(n)
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }

    pub fn reset(&mut self, from: BrowserHistFrom) {
        self.clear();
        self.l_timestamp = match from {
            BrowserHistFrom::Start => 0,
            BrowserHistFrom::Now => timestamp_now(),
            BrowserHistFrom::Timestamp(t) => t,
        }
    }

    pub fn history(&self) -> &Vec<BrowserHistEntry> {
        &self.history
    }

    pub fn browser(&self) -> &Browser {
        &self.browser
    }
}
