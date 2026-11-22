//! Sending. Each thing this window writes is one call to the engine on a channel the web
//! window calls too (`rules/writes.rs` says which, and with what), made off the main thread;
//! what the engine said is handed back in `poll`. Nothing is sent from a fixture.

use std::sync::mpsc::{Receiver, Sender, channel};

use rune::Waker;
use serde_json::Value;

use crate::data::Item;
use crate::source::{self, Door};

/// What was sent, kept until the engine answers, so a refusal can be put right on screen.
#[derive(Clone, Debug, PartialEq)]
pub enum Sent {
    /// Words from the reply box of the thread `product/id`: they go back in the box if the
    /// engine does not take them.
    Reply { thread: String, words: String },
    /// A picked option of that thread.
    Pick { thread: String },
    Close { thread: String },
    Reopen { thread: String },
}

/// Something done and not yet written (`deferCommit` in `renderer/src/App.tsx`). It shows at
/// once, is written when its time comes, and until then Z takes it back whole: nothing was
/// written, so nothing has to be unwritten.
pub struct Held {
    pub channel: &'static str,
    pub payload: Value,
    pub sent: Sent,
    /// What the line about it says.
    pub said: String,
    /// When it is written, in seconds on the window's clock.
    pub at: f64,
    /// The thread it was done to, as `(id, product)`: where Z goes back.
    pub thread: (String, String),
    /// Done from inside that conversation, which Z then opens again.
    pub from_inside: bool,
    /// The row as it was, when this closes it: what undoing the close needs once it is written.
    pub closes: Option<Item>,
}

/// The one thing Z still takes back after it was written: a close, by opening the thread as
/// it was before.
pub struct Undo {
    pub said: String,
    pub payload: Value,
    pub thread: (String, String),
}

/// One line about what was just done, with Z beside it while Z can take it back.
pub struct Toast {
    pub line: String,
    pub undo: bool,
    /// When it goes, in seconds on the window's clock.
    pub until: f64,
}

/// One answer from the engine: its own, or why it could not be reached.
pub struct Landed {
    pub sent: Sent,
    pub answer: Result<Value, String>,
}

impl Landed {
    /// Why the write did not happen, when it did not: the door failed, or the engine refused
    /// (`{ ok: false, error }`, as `main/ipc.mts` answers a write it turns down).
    pub fn refusal(&self) -> Option<String> {
        match &self.answer {
            Err(e) => Some(e.clone()),
            Ok(v) if v.get("ok") == Some(&Value::Bool(false)) => {
                Some(v.get("error").or_else(|| v.get("why")).and_then(Value::as_str).unwrap_or("Mercury did not take it.").to_string())
            }
            Ok(_) => None,
        }
    }
}

pub struct Outbox {
    tx: Sender<Landed>,
    rx: Receiver<Landed>,
    waker: Option<Waker>,
}

impl Default for Outbox {
    fn default() -> Self {
        let (tx, rx) = channel();
        Outbox { tx, rx, waker: None }
    }
}

impl Outbox {
    /// The window's waker, so an answer is drawn when it lands and not at the next frame.
    pub fn wake_with(&mut self, waker: Waker) {
        self.waker = Some(waker);
    }

    /// Sends, and says whether there was a door to send through.
    pub fn send(&self, door: &Door, channel: &'static str, payload: Value, sent: Sent) -> bool {
        let Some((url, token)) = door.now() else {
            return false;
        };
        let (tx, waker, door) = (self.tx.clone(), self.waker.clone(), door.clone());
        std::thread::spawn(move || {
            let answer = source::call(&url, &token, channel, &payload);
            // What was written changes the lists: read them again at once, not at the next turn.
            door.ask_soon();
            let _ = tx.send(Landed { sent, answer });
            if let Some(wake) = waker {
                wake();
            }
        });
        true
    }

    /// The answers that came since the last look.
    pub fn landed(&self) -> Vec<Landed> {
        self.rx.try_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn landed(answer: Result<Value, String>) -> Landed {
        Landed { sent: Sent::Close { thread: "shop/w-1".to_string() }, answer }
    }

    #[test]
    fn a_write_is_refused_when_the_door_fails_or_the_engine_says_no() {
        assert_eq!(landed(Ok(json!({ "ok": true }))).refusal(), None);
        // A row the engine wrote comes back whole, with no `ok` at all.
        assert_eq!(landed(Ok(json!({ "id": "w-1", "status": "done" }))).refusal(), None);
        assert_eq!(landed(Ok(json!({ "ok": false, "error": "Its work never reached main." }))).refusal().as_deref(), Some("Its work never reached main."));
        assert_eq!(landed(Ok(json!({ "ok": false }))).refusal().as_deref(), Some("Mercury did not take it."));
        assert_eq!(landed(Err("connection refused".to_string())).refusal().as_deref(), Some("connection refused"));
    }
}
