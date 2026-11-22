//! Where the conversations come from. Either a fixture file (the snapshot a reference picture
//! was taken with) or the engine's own door: `main/serve.mts`, the same channels the web window
//! calls, on 127.0.0.1 with a key.
//!
//!   mercury --fixture shots/home-fixture.json
//!   mercury --door http://127.0.0.1:7777 --token <key>      (or MERCURY_DOOR and MERCURY_TOKEN)
//!   mercury --door-file <path>                              (a JSON file with `port` and `token`)
//!
//! With the door, one thread listens to the engine's event stream and another asks for the
//! snapshot again when the stream says something changed. The same stream carries the desktop
//! app's word to come forward at a thread (`front.rs`). It also asks unprompted: every five
//! seconds of an engine that has shown it tells, and every two of one that has not, because
//! not every engine that opens this door pushes. The desktop app's door moves each time the
//! app starts, and its note is read again when the one in hand stops answering. What the
//! window writes goes out through `write.rs`; nothing here does.

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
    Door(Door),
}

/// The engine's door: where it is now and, for the desktop app's own, the note that says so.
/// The app opens a new door, on another port and with another key, each time it starts, so an
/// address kept from before stops answering. Whoever asks takes the address from here, and
/// `look_again` reads the note for the door that answers now. A window opened before the app
/// has no address yet and finds one the same way.
#[derive(Clone, Debug, Default)]
pub struct Door {
    at: Arc<Mutex<Option<(String, String)>>>,
    note: Option<String>,
    /// Set by whoever just wrote through the door: the follower reads again at once.
    soon: Arc<AtomicBool>,
}

impl Door {
    /// A door named on the command line: it is where it is.
    fn fixed(url: String, token: String) -> Door {
        Door { at: Arc::new(Mutex::new(Some((url, token)))), note: None, soon: Arc::default() }
    }
    /// A door a running app leaves a note about: `{ "port": 49216, "token": "…" }`.
    fn noted(path: String) -> Door {
        let door = Door { at: Arc::default(), note: Some(path), soon: Arc::default() };
        door.look_again();
        door
    }
    /// Asks the follower to read the engine again now, not at its next turn: something this
    /// window wrote has changed what the lists say.
    pub fn ask_soon(&self) {
        self.soon.store(true, Ordering::Relaxed);
    }
    /// The address and the key to ask with now, when there is a door at all.
    pub fn now(&self) -> Option<(String, String)> {
        self.at.lock().unwrap().clone()
    }
    /// The note this door was found by, when it was: what the window keeps for itself sits
    /// in the same folder.
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
    /// Reads the note again. True when it names another door than the one in hand.
    pub fn look_again(&self) -> bool {
        let Some(read) = self.note.as_deref().and_then(read_note) else {
            return false;
        };
        let mut at = self.at.lock().unwrap();
        if at.as_ref() == Some(&read) {
            return false;
        }
        *at = Some(read);
        true
    }
}

fn read_note(path: &str) -> Option<(String, String)> {
    let note: Value = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    let port = note.get("port")?.as_u64()?;
    Some((format!("http://127.0.0.1:{port}"), note.get("token")?.as_str()?.to_string()))
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
/// The same, of an engine that says when its inbox changes. It still asks, and twice as often
/// as the web window does behind the same pushes, because the engine does not say everything:
/// its store hears of a change from a watch on its folders, which misses some (two new threads
/// of three went unsaid on a scratch engine, 2026-11-21), and what the web window keeps for
/// itself (which threads were read in it) is never pushed at all.
const ASK_WHEN_TOLD: Duration = Duration::from_secs(5);

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

pub fn from_args(args: &[String]) -> Source {
    if let Some(path) = flag(args, "--fixture") {
        return Source::Fixture(path);
    }
    if let Some(path) = flag(args, "--door-file") {
        return Source::Door(Door::noted(path));
    }
    let url = flag(args, "--door").or_else(|| std::env::var("MERCURY_DOOR").ok());
    let token = flag(args, "--token").or_else(|| std::env::var("MERCURY_TOKEN").ok());
    match (url, token) {
        (Some(url), Some(token)) => Source::Door(Door::fixed(url.trim_end_matches('/').to_string(), token)),
        // Told nothing, look for the running app's own door (`main/local-door.mts` leaves the
        // note), and keep looking: the app may start after this window does.
        _ => running_app_note().map_or(Source::Nothing, |path| Source::Door(Door::noted(path))),
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
    call_waiting(url, token, channel, payload, Duration::from_secs(20))
}

/// `call`, waiting `wait` for the answer.
pub fn call_waiting(url: &str, token: &str, channel: &str, payload: &Value, wait: Duration) -> Result<Value, String> {
    let channel = channel.replace(':', "%3A");
    let asked = ureq::post(&format!("{url}/api/{channel}"))
        .set("x-agentbox-token", token)
        .set("content-type", "application/json")
        .timeout(wait)
        .send_string(&payload.to_string());
    let res = match asked {
        Ok(res) => res,
        // A handler that threw answers 500 with its reason in `error` (`main/serve.mts`), and
        // that reason is what the person is told: the web window's bridge throws it as it is.
        Err(ureq::Error::Status(500, res)) => {
            let text = res.into_string().unwrap_or_default();
            let why = serde_json::from_str::<Value>(&text).ok().and_then(|v| v.get("error").and_then(Value::as_str).map(str::to_string));
            return Err(why.unwrap_or_else(|| "Mercury could not do that.".to_string()));
        }
        Err(e) => return Err(e.to_string()),
    };
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
pub fn follow_door(door: Door, mailbox: Mailbox, wake: impl Fn() + Send + 'static) {
    let told = Arc::new(AtomicBool::new(true));
    // Whether the stream in hand has ever said anything. An engine that tells is asked far
    // less often; one that only keeps its stream open is asked as before.
    let tells = Arc::new(AtomicBool::new(false));
    // The engine's pushes. A line of data is one; anything else is a heartbeat. When the stream
    // cannot open or drops, wait a moment and start over, at whatever door there is by then:
    // the asking below carries on meanwhile.
    {
        let (door, told, tells) = (door.clone(), told.clone(), tells.clone());
        std::thread::spawn(move || {
            loop {
                let stream = door.now().and_then(|(url, token)| ureq::get(&format!("{url}/events?token={token}&device=mercury-desktop")).call().ok());
                if let Some(stream) = stream {
                    for line in BufReader::new(stream.into_reader()).lines() {
                        match line {
                            Ok(line) if line.starts_with("data:") => {
                                tells.store(true, Ordering::Relaxed);
                                // A press on the app's banner or tray, for this window: no news of the inbox.
                                match crate::front::asked_to_open(&line) {
                                    Some(opening) => crate::front::come_forward(opening),
                                    None => told.store(true, Ordering::Relaxed),
                                }
                            }
                            Ok(_) => {}
                            Err(_) => break,
                        }
                    }
                }
                // The next engine at this door has to show that it tells, too.
                tells.store(false, Ordering::Relaxed);
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
            let asked_for = door.soon.swap(false, Ordering::Relaxed);
            let every = if tells.load(Ordering::Relaxed) { ASK_WHEN_TOLD } else { ASK_EVERY };
            if !told.swap(false, Ordering::Relaxed) && !asked_for && asked.elapsed() < every {
                continue;
            }
            asked = Instant::now();
            let news = match door.now() {
                None => Err("the app has not opened its door yet".to_string()),
                Some((url, token)) => call(&url, &token, "zero:snapshot", &Value::Null).and_then(|whole| {
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
                }),
            };
            // A door that does not answer may be one the app has since replaced: restarted, it
            // opens another. With a new one in hand, ask it at once, as a new engine.
            if news.is_err() && door.look_again() {
                has_a_window = true;
                seen = None;
                told.store(true, Ordering::Relaxed);
                continue;
            }
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
