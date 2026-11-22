//! What an open conversation asks the engine for, and how often (`ItemThread.tsx`, through
//! `sessionTrace` in `renderer/src/api.ts`).
//!
//! One read is the thread's lines (`zero:item-history`) and its runs' logs
//! (`zero:session-trace`), the two together and one read at a time. The logs are asked for
//! with the stamp of each one already held (`have`), so a log that has not changed comes back
//! as a word and one that grew as what it grew by; `rules::trace::Reader` puts them back
//! together, and a read it cannot put together is asked again with no stamps. Once the
//! conversation's middle has been asked for, every log is read whole, once.
//!
//! A conversation is read when it opens, when its row changes, and when a run starts on it.
//! While a run is up it is read again on every change the engine pushes and every eight
//! seconds besides, as the web window does: the push covers a line the moment it is written,
//! the timer what no push says. A finished conversation is not read again until it changes.
//! All of it is off the main thread.

use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

use crate::rules::trace;
use crate::source;

/// The backstop while a run is up (`REFRESH_MS` in `ItemThread.tsx`).
pub const AGAIN_MS: f64 = 8_000.0;

/// What one read brought back, for the open conversation's place in `State::threads`.
pub type Answer = (String, Result<Value, String>);

#[derive(Default)]
pub struct Reads {
    /// The logs held of each thread read lately, shared with the reads on their way.
    reader: Arc<Mutex<trace::Reader>>,
    /// The thread last read, and what it was read at: its row's last change and the run on it.
    last: Option<(String, f64, Option<f64>)>,
    /// When the open thread was last read, for the backstop.
    at: f64,
    /// A read is owed: something changed while another read was on its way.
    owed: bool,
}

/// One read to make: where, which thread, and whether every log is to be read whole.
pub struct Ask {
    url: String,
    token: String,
    key: String,
    product: String,
    id: String,
    every: bool,
    reader: Arc<Mutex<trace::Reader>>,
}

impl Reads {
    /// The read the open thread is owed now, if any: `changed` is its row's last change, `run`
    /// when the run on it started, `pushed` whether the engine said something changed since the
    /// last turn, `every` whether every log is wanted whole and not yet read so, and `busy`
    /// whether a read is still on its way, which this one then waits for.
    #[allow(clippy::too_many_arguments)]
    pub fn owed(&mut self, door: (String, String), product: &str, id: &str, changed: f64, run: Option<f64>, pushed: bool, every: bool, busy: bool, now: f64) -> Option<Ask> {
        let key = format!("{product}/{id}");
        let fresh = self.last.as_ref().is_none_or(|(k, c, r)| *k != key || *c != changed || *r != run);
        let live = run.is_some() && (pushed || now - self.at >= AGAIN_MS);
        if fresh || live || every {
            self.owed = true;
        }
        if !self.owed || busy {
            return None;
        }
        self.owed = false;
        self.at = now;
        self.last = Some((key.clone(), changed, run));
        let (url, token) = door;
        Some(Ask { url, token, key, product: product.to_string(), id: id.to_string(), every, reader: self.reader.clone() })
    }

    /// A read that failed: the thread is read again at its next turn.
    pub fn failed(&mut self) {
        self.last = None;
    }
}

/// One channel's answer, and how many bytes it came to.
fn call(ask: &Ask, channel: &str, payload: &Value) -> Result<(Value, usize), String> {
    let answer = source::call(&ask.url, &ask.token, channel, payload)?;
    let bytes = answer.to_string().len();
    Ok((answer, bytes))
}

/// The read itself, on a worker. `MERCURY_READS=1` prints what each one asked and how much
/// came back.
pub fn read(ask: Ask) -> Answer {
    let key = ask.key.clone();
    let result = (|| {
        let thread = json!({ "product": ask.product, "id": ask.id });
        let (history, said) = call(&ask, "zero:item-history", &thread)?;
        let have = ask.reader.lock().unwrap().have(&ask.key);
        let mut with = thread.clone();
        if let Some(have) = &have {
            with["have"] = have.clone();
        }
        let (logs, mut bytes) = call(&ask, "zero:session-trace", &with)?;
        let mut whole_again = false;
        let taken = ask.reader.lock().unwrap().take(&ask.key, &logs);
        let sessions = match taken {
            Some(sessions) => sessions,
            // What could not be put together is asked again with no stamps.
            None => {
                whole_again = true;
                let (logs, more) = call(&ask, "zero:session-trace", &thread)?;
                bytes += more;
                ask.reader.lock().unwrap().take(&ask.key, &logs).unwrap_or_default()
            }
        };
        let mut answer = json!({
            "zero:item-history": history,
            "zero:session-trace": { "ok": true, "sessions": sessions },
        });
        if ask.every {
            let (every, more) = call(&ask, "zero:session-trace", &json!({ "product": ask.product, "id": ask.id, "whole": true }))?;
            bytes += more;
            answer["zero:session-trace/whole"] = every;
        }
        if std::env::var("MERCURY_READS").is_ok_and(|v| v == "1") {
            let lines = answer["zero:item-history"]["lines"].as_array().map_or(0, Vec::len);
            let logs = answer["zero:session-trace"]["sessions"].as_array().map_or(0, Vec::len);
            eprintln!("read {}: {lines} lines in {said} bytes, {logs} logs in {bytes} bytes{}{}{}", ask.key, if have.is_some() { ", stamps sent" } else { "" }, if whole_again { ", asked again whole" } else { "" }, if ask.every { ", every log whole" } else { "" });
        }
        Ok(answer)
    })();
    (key, result)
}

/// A read landing on what the window holds of the thread: the whole logs read once stay
/// until a newer whole read replaces them.
pub fn landed(held: Option<&Value>, mut answer: Value) -> Value {
    if answer.get("zero:session-trace/whole").is_none()
        && let Some(every) = held.and_then(|h| h.get("zero:session-trace/whole"))
    {
        answer["zero:session-trace/whole"] = every.clone();
    }
    answer
}

#[cfg(test)]
mod tests {
    use super::*;

    fn door() -> (String, String) {
        ("http://127.0.0.1:1".to_string(), "k".to_string())
    }

    #[test]
    fn a_finished_thread_is_read_once_and_again_only_when_it_changes() {
        let mut reads = Reads::default();
        assert!(reads.owed(door(), "p", "w-1", 10.0, None, false, false, false, 0.0).is_some());
        assert!(reads.owed(door(), "p", "w-1", 10.0, None, true, false, false, 60_000.0).is_none());
        assert!(reads.owed(door(), "p", "w-1", 11.0, None, false, false, false, 61_000.0).is_some());
        // Another thread is read when it opens.
        assert!(reads.owed(door(), "p", "w-2", 11.0, None, false, false, false, 61_500.0).is_some());
    }

    #[test]
    fn a_thread_with_a_run_up_is_read_on_every_push_and_every_eight_seconds() {
        let mut reads = Reads::default();
        assert!(reads.owed(door(), "p", "w-1", 10.0, Some(5.0), false, false, false, 0.0).is_some());
        assert!(reads.owed(door(), "p", "w-1", 10.0, Some(5.0), false, false, false, 7_000.0).is_none());
        assert!(reads.owed(door(), "p", "w-1", 10.0, Some(5.0), true, false, false, 7_500.0).is_some());
        assert!(reads.owed(door(), "p", "w-1", 10.0, Some(5.0), false, false, false, 15_600.0).is_some());
    }

    #[test]
    fn one_read_at_a_time_and_none_lost_meanwhile() {
        let mut reads = Reads::default();
        assert!(reads.owed(door(), "p", "w-1", 10.0, Some(5.0), false, false, false, 0.0).is_some());
        // A push while the read is on its way is owed, and asked once it lands.
        assert!(reads.owed(door(), "p", "w-1", 10.0, Some(5.0), true, false, true, 1_000.0).is_none());
        assert!(reads.owed(door(), "p", "w-1", 10.0, Some(5.0), false, false, false, 1_500.0).is_some());
        assert!(reads.owed(door(), "p", "w-1", 10.0, Some(5.0), false, false, false, 2_000.0).is_none());
    }

    #[test]
    fn the_whole_logs_stay_under_the_reads_after_them() {
        let held = json!({ "zero:session-trace/whole": { "sessions": [{ "startedAt": 1, "text": "all of it" }] } });
        let regular = json!({ "zero:item-history": { "lines": [] }, "zero:session-trace": { "sessions": [] } });
        assert_eq!(landed(Some(&held), regular)["zero:session-trace/whole"]["sessions"][0]["text"], "all of it");
    }
}
