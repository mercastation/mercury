//! Sending. Each thing this window writes is one call to the engine on a channel the web
//! window calls too (`rules/writes.rs` says which, and with what), made off the main thread;
//! what the engine said is handed back in `poll`. Nothing is sent from a fixture.

use std::sync::mpsc::{Receiver, Sender, channel};

use rune::Waker;
use serde_json::Value;

use crate::data::Item;
use crate::rules::compose::{Level, Pick};
use crate::rules::writes::Write;
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
    /// That thread put off until later. Once the engine has it, the line `said` goes up and
    /// Z can still write `undo`.
    Later { thread: String, said: String, undo: Write },
    /// That thread brought back from Later, the same way.
    Back { thread: String, said: &'static str, undo: Vec<Write> },
    /// Z, taking one of those two back.
    TakenBack { thread: String },
    /// A new thread for the project `product`, with what the box held: it all goes back in the
    /// box if the engine does not take it, and Z hands it back after it did.
    Thread { product: String, draft: Draft },
    /// A thread sent from here, taken back with Z, with its name for the line that says so.
    Withdraw { thread: String, title: String, draft: Draft },
}

impl Sent {
    /// The thread it was done to, as `product/id`. A new thread has none until the engine
    /// has made it.
    pub fn thread(&self) -> &str {
        match self {
            Sent::Reply { thread, .. }
            | Sent::Pick { thread }
            | Sent::Close { thread }
            | Sent::Reopen { thread }
            | Sent::Later { thread, .. }
            | Sent::Back { thread, .. }
            | Sent::TakenBack { thread }
            | Sent::Withdraw { thread, .. } => thread,
            Sent::Thread { .. } => "",
        }
    }
}

/// What the box to start a thread in held when Send was pressed: the words, the files, and the
/// level, the model and Later picked on it, which are as much a part of the message
/// (`ComposeDraft` in `renderer/src/drafts.ts`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Draft {
    pub words: String,
    pub level: Option<Level>,
    pub pick: Option<Pick>,
    pub later: bool,
    pub files: Vec<crate::attach::Attached>,
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

/// What Z still takes back after it was written: a close, by opening the thread as it was
/// before; a thread put off or brought back, by writing its moments as they were; a thread
/// sent from the box, by closing it, which stops its worker (`noteNewTask` in
/// `renderer/src/App.tsx`). `sent` is what the engine's answer to that is filed under.
pub struct Undo {
    pub said: String,
    pub writes: Vec<Write>,
    pub sent: Sent,
    pub thread: (String, String),
    /// Whether Z then opens that thread: it does when the thread comes back in front of the
    /// person, so the key is seen to have done something.
    pub opens: bool,
}

/// One line about what was just done, with Z beside it while Z can take it back.
pub struct Toast {
    pub line: String,
    pub undo: bool,
    /// When it went up and when it goes, in seconds on the window's clock.
    pub since: f64,
    pub until: f64,
    /// The thread it is about, as `(id, product)`: the line is then a card that names it, O
    /// opens it, and a bar along its foot runs out with the time left to (`ToastGoes` in
    /// `renderer/src/toast-parts.ts`).
    pub goes: Option<(String, String)>,
}

/// One answer from the engine: its own, or why it could not be reached.
pub struct Landed {
    pub sent: Sent,
    pub answer: Result<Value, String>,
    /// The engine's lists read again once it had answered a new thread, so the line about that
    /// thread is said of what the engine then says (`refresh` before `sentLine` in `App.tsx`).
    pub snapshot: Option<Value>,
}

/// Why a write did not happen, when it did not: the door failed, or the engine refused
/// (`{ ok: false, error }`, as `main/ipc.mts` answers a write it turns down).
fn refusal_in(answer: &Result<Value, String>) -> Option<String> {
    match answer {
        Err(e) => Some(e.clone()),
        Ok(v) if v.get("ok") == Some(&Value::Bool(false)) => {
            Some(v.get("error").or_else(|| v.get("why")).and_then(Value::as_str).unwrap_or("Mercury did not take it.").to_string())
        }
        Ok(_) => None,
    }
}

impl Landed {
    pub fn refusal(&self) -> Option<String> {
        refusal_in(&self.answer)
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

    /// Sends the writes that are one thing done, each after the one before it was taken, and
    /// says whether there was a door to send through. The first the engine refuses stops the
    /// rest, and its refusal is the answer; otherwise the answer is the last one's.
    pub fn send_in_order(&self, door: &Door, writes: Vec<Write>, sent: Sent) -> bool {
        let Some((url, token)) = door.now() else {
            return false;
        };
        let (tx, waker, door) = (self.tx.clone(), self.waker.clone(), door.clone());
        std::thread::spawn(move || {
            let mut answer = Ok(Value::Null);
            for (channel, payload) in &writes {
                answer = source::call(&url, &token, channel, payload);
                if refusal_in(&answer).is_some() {
                    break;
                }
            }
            let made = refusal_in(&answer).is_none() && matches!(sent, Sent::Thread { .. });
            let snapshot = if made { source::call(&url, &token, "zero:snapshot", &Value::Null).ok() } else { None };
            // What was written changes the lists: read them again at once, not at the next turn.
            door.ask_soon();
            let _ = tx.send(Landed { sent, answer, snapshot });
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
        Landed { sent: Sent::Close { thread: "shop/w-1".to_string() }, answer, snapshot: None }
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
