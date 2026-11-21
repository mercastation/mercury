//! Where the conversations come from. Either a fixture file (the snapshot a reference picture
//! was taken with) or the engine's own door: `main/serve.mts`, the same channels the web window
//! calls, on 127.0.0.1 with a key.
//!
//!   mercury --fixture shots/home-fixture.json
//!   mercury --door http://127.0.0.1:7777 --token <key>      (or MERCURY_DOOR and MERCURY_TOKEN)
//!   mercury --door-file <path>                              (a JSON file with `port` and `token`)
//!
//! With the door, one thread listens to the engine's event stream and another asks for the
//! snapshot again when the stream says something changed, and every two seconds besides,
//! because not every engine that opens this door pushes. Nothing here writes: phase 1 only reads.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::data::{Fixture, Local, Snapshot};

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
    /// What the web window keeps for itself, when the door can say.
    pub local: Option<Local>,
    pub error: Option<String>,
}

pub type Mailbox = Arc<Mutex<Arrived>>;

/// How long the window goes without asking when nothing has told it to.
const ASK_EVERY: Duration = Duration::from_secs(2);

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

/// A door a running app left a note about: `{ "port": 49216, "token": "…" }`.
fn door_file(path: &str) -> Option<Source> {
    let note: Value = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    let port = note.get("port")?.as_u64()?;
    let token = note.get("token")?.as_str()?.to_string();
    Some(Source::Door { url: format!("http://127.0.0.1:{port}"), token })
}

pub fn from_args(args: &[String]) -> Source {
    if let Some(path) = flag(args, "--fixture") {
        return Source::Fixture(path);
    }
    if let Some(door) = flag(args, "--door-file").and_then(|path| door_file(&path)) {
        return door;
    }
    let url = flag(args, "--door").or_else(|| std::env::var("MERCURY_DOOR").ok());
    let token = flag(args, "--token").or_else(|| std::env::var("MERCURY_TOKEN").ok());
    match (url, token) {
        (Some(url), Some(token)) => Source::Door { url: url.trim_end_matches('/').to_string(), token },
        // Told nothing, look for the running app's own door (`main/local-door.mts` leaves the note).
        _ => running_app_note().and_then(|path| door_file(&path)).unwrap_or(Source::Nothing),
    }
}

/// Where the desktop app leaves its door's note: its own folder under Application Support.
fn running_app_note() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    Some(format!("{home}/Library/Application Support/Mercury/local-door.json"))
}

/// A fixture is read at once, before the first frame, so a headless picture has its data.
pub fn read_fixture(path: &str) -> Result<Fixture, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))
}

/// One call to a channel of the engine, as the web window's bridge makes it.
pub fn call(url: &str, token: &str, channel: &str, payload: &Value) -> Result<Value, String> {
    let channel = channel.replace(':', "%3A");
    let res = ureq::post(&format!("{url}/api/{channel}"))
        .set("x-agentbox-token", token)
        .set("content-type", "application/json")
        .timeout(Duration::from_secs(20))
        .send_string(&payload.to_string())
        .map_err(|e| e.to_string())?;
    let text = res.into_string().map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

/// One read of the engine's snapshot. A headless picture calls it before its first frame.
pub fn fetch_snapshot(url: &str, token: &str) -> Result<Snapshot, String> {
    serde_json::from_value(call(url, token, "zero:snapshot", &Value::Null)?).map_err(|e| e.to_string())
}

/// What the web window keeps for itself and decides its lists by: what counts as delivered and
/// as read (`zero:window-keeps`, a channel of the desktop app's local door alone). None from an
/// engine with no window, or a window that did not answer.
pub fn fetch_keeps(url: &str, token: &str) -> Result<Option<Local>, String> {
    let kept = call(url, token, "zero:window-keeps", &Value::Null)?;
    Ok(serde_json::from_value(kept).ok())
}

/// What the screens draw from, boiled down to a number: a snapshot that differs only in parts
/// no screen reads (usage figures, the token of the answer itself) is not news.
fn fingerprint(snapshot: &Value, keeps: &Option<Local>) -> u64 {
    let mut h = DefaultHasher::new();
    for part in ["items", "supervisor", "products", "approvals", "engines"] {
        snapshot.get(part).map(Value::to_string).hash(&mut h);
    }
    // Of the cloud only what its button wears: the rest is hours and readings that tick over.
    crate::rules::cloud::button(snapshot.get("cloud"), keeps.as_ref().map_or(&[], |local| local.cloud_broken_seen.as_slice())).hash(&mut h);
    if let Some(local) = keeps {
        (local.delivered_through.to_bits(), local.unread_since.to_bits()).hash(&mut h);
        let mut seen: Vec<(&String, u64)> = local.seen_at.iter().map(|(id, at)| (id, at.to_bits())).collect();
        seen.sort();
        seen.hash(&mut h);
    }
    h.finish()
}

/// Keeps `mailbox` current from the door and calls `wake` each time something new lands.
pub fn follow_door(url: String, token: String, mailbox: Mailbox, wake: impl Fn() + Send + 'static) {
    let told = Arc::new(AtomicBool::new(true));
    // The engine's pushes. A line of data is one; anything else is a heartbeat. When the stream
    // cannot open or drops, wait a moment and start over: the asking below carries on meanwhile.
    {
        let (url, token, told) = (url.clone(), token.clone(), told.clone());
        std::thread::spawn(move || {
            loop {
                if let Ok(stream) = ureq::get(&format!("{url}/events?token={token}&device=mercury-desktop")).call() {
                    for line in BufReader::new(stream.into_reader()).lines() {
                        match line {
                            Ok(line) if line.starts_with("data:") => told.store(true, Ordering::Relaxed),
                            Ok(_) => {}
                            Err(_) => break,
                        }
                    }
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        });
    }
    std::thread::spawn(move || {
        let mut seen = None;
        let mut asked = Instant::now() - ASK_EVERY;
        // Only the desktop app's door has a window to ask; one refusal settles it for this run.
        let mut has_a_window = true;
        loop {
            // Pushes come in bursts while an agent works; a short pause lets one read cover them.
            std::thread::sleep(Duration::from_millis(200));
            if !told.swap(false, Ordering::Relaxed) && asked.elapsed() < ASK_EVERY {
                continue;
            }
            asked = Instant::now();
            let news = call(&url, &token, "zero:snapshot", &Value::Null).and_then(|whole| {
                let keeps = if has_a_window {
                    fetch_keeps(&url, &token).unwrap_or_else(|refused| {
                        has_a_window = !refused.contains("404");
                        None
                    })
                } else {
                    None
                };
                let print = fingerprint(&whole, &keeps);
                if seen == Some(print) {
                    return Ok(None);
                }
                seen = Some(print);
                serde_json::from_value::<Snapshot>(whole).map(|snapshot| Some((snapshot, keeps))).map_err(|e| e.to_string())
            });
            let mut m = mailbox.lock().unwrap();
            match news {
                Ok(None) if m.error.is_none() => continue,
                Ok(None) => m.error = None,
                Ok(Some((snapshot, keeps))) => {
                    m.snapshot = Some(snapshot);
                    if keeps.is_some() {
                        m.local = keeps;
                    }
                    m.error = None;
                }
                Err(e) if m.error.as_deref() == Some(e.as_str()) => continue,
                Err(e) => m.error = Some(e),
            }
            drop(m);
            wake();
        }
    });
}
