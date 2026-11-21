//! Where the conversations come from. Either a fixture file (the snapshot a reference picture
//! was taken with) or the engine's own door: `main/serve.mts`, the same channels the web window
//! calls, on 127.0.0.1 with a key.
//!
//!   mercury --fixture shots/home.json
//!   mercury --door http://127.0.0.1:7777 --token <key>      (or MERCURY_DOOR and MERCURY_TOKEN)
//!
//! With the door, a thread keeps the event stream open and asks for the snapshot again
//! whenever the engine says something changed. Nothing here writes: phase 1 only reads.

use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::data::{Fixture, Snapshot};

#[derive(Clone, Debug)]
pub enum Source {
    /// No data: the screens draw empty.
    Nothing,
    Fixture(String),
    Door { url: String, token: String },
}

/// What the door delivered last, waiting for the window to pick it up.
#[derive(Default)]
pub struct Arrived {
    pub snapshot: Option<Snapshot>,
    pub error: Option<String>,
}

pub type Mailbox = Arc<Mutex<Arrived>>;

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

pub fn from_args(args: &[String]) -> Source {
    if let Some(path) = flag(args, "--fixture") {
        return Source::Fixture(path);
    }
    let url = flag(args, "--door").or_else(|| std::env::var("MERCURY_DOOR").ok());
    let token = flag(args, "--token").or_else(|| std::env::var("MERCURY_TOKEN").ok());
    match (url, token) {
        (Some(url), Some(token)) => Source::Door { url: url.trim_end_matches('/').to_string(), token },
        _ => Source::Nothing,
    }
}

/// A fixture is read at once, before the first frame, so a headless picture has its data.
pub fn read_fixture(path: &str) -> Result<Fixture, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))
}

/// One read of the engine's snapshot. A headless picture calls it before its first frame.
pub fn fetch_snapshot(url: &str, token: &str) -> Result<Snapshot, String> {
    let res = ureq::post(&format!("{url}/api/zero%3Asnapshot"))
        .set("x-agentbox-token", token)
        .set("content-type", "application/json")
        .timeout(Duration::from_secs(20))
        .send_string("null")
        .map_err(|e| e.to_string())?;
    let text = res.into_string().map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

/// Keeps `mailbox` current from the door and calls `wake` each time something lands.
pub fn follow_door(url: String, token: String, mailbox: Mailbox, wake: impl Fn() + Send + 'static) {
    std::thread::spawn(move || {
        loop {
            match fetch_snapshot(&url, &token) {
                Ok(snapshot) => {
                    let mut m = mailbox.lock().unwrap();
                    m.snapshot = Some(snapshot);
                    m.error = None;
                }
                Err(e) => mailbox.lock().unwrap().error = Some(e),
            }
            wake();
            // The stream says when to ask again. A line of data is a push; anything else is a
            // heartbeat. If the stream cannot open or drops, wait a moment and start over.
            let stream = ureq::get(&format!("{url}/events?token={token}&device=mercury-desktop")).call();
            let Ok(stream) = stream else {
                std::thread::sleep(Duration::from_secs(2));
                continue;
            };
            let mut lines = BufReader::new(stream.into_reader()).lines();
            loop {
                match lines.next() {
                    Some(Ok(line)) if line.starts_with("data:") => break,
                    Some(Ok(_)) => continue,
                    _ => {
                        std::thread::sleep(Duration::from_secs(1));
                        break;
                    }
                }
            }
            // Pushes come in bursts while an agent works; one read a quarter second later covers them.
            std::thread::sleep(Duration::from_millis(250));
        }
    });
}
