//! Whether this window is the one in front, said to the desktop app; and the app's word to come
//! to the front, at a thread.
//!
//! The app decides whether to put up a banner, or its island in the notch, from whether the
//! person is at it, and it went by its own window's focus alone: while the person worked here it
//! took them to be in another app. So this window says where it is each time it gains or loses
//! focus, on a channel only the desktop app's own door answers (`main/local-door.mts`), and again
//! every beat, in front or not. The app stops believing a word three beats old. Each beat is
//! asked of this window's own thread, so a window that hangs stops beating; one that quits or
//! crashes hangs up the door's event stream, and is forgotten then (`main/native-window.mts`).
//!
//! A press on the app's banner, or on its island or corner tag, comes back on that event stream
//! (`OPEN_HERE`) while the app hears this window beat: the window comes forward and opens the
//! thread, as a press on its row would.

use std::sync::OnceLock;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::time::Duration;

use rune::{AppHandle, Tx};
use serde_json::{Value, json};

use crate::State;
use crate::data::Snapshot;
use crate::source::{self, Door};

/// Where this window says whether it is in front: `{ "focused": true }`.
pub const SAY_WHERE: &str = "zero:native-window";

/// What the app pushes on the door's event stream when its banner or its tray is pressed:
/// `{ "id", "product" }` of the thread to open here, or nulls to come forward as it is
/// (`OPEN_IN_NATIVE` in `main/native-window.mts`).
pub const OPEN_HERE: &str = "zero:open-in-native";

/// How often that is said again: `NATIVE_BEAT_MS` in `shared/notify-rules.mts`.
const BEAT: Duration = Duration::from_secs(5);

/// What the thread that speaks to the app hears: a change of focus, or this window's own thread
/// answering a beat.
enum Heard {
    Focus(bool),
    Beat,
}

#[derive(Debug, PartialEq)]
enum Next {
    Say(bool),
    AskForABeat,
    Wait,
    Stop,
}

/// What the speaking thread keeps from one beat to the next.
#[derive(Default)]
struct Speaker {
    focused: bool,
    /// A beat asked of this window's own thread and not answered yet.
    asked: bool,
}

impl Speaker {
    /// What to do after waiting a beat for news: a change of focus is said at once; a beat that
    /// passed quietly is asked of this window's own thread, unless it has not answered the last
    /// one; and its answer says where the window is, in front or not.
    fn next(&mut self, got: Result<Heard, RecvTimeoutError>) -> Next {
        match got {
            Ok(Heard::Focus(now)) => {
                self.focused = now;
                Next::Say(now)
            }
            Ok(Heard::Beat) => {
                self.asked = false;
                Next::Say(self.focused)
            }
            Err(RecvTimeoutError::Timeout) if self.asked => Next::Wait,
            Err(RecvTimeoutError::Timeout) => {
                self.asked = true;
                Next::AskForABeat
            }
            Err(RecvTimeoutError::Disconnected) => Next::Stop,
        }
    }
}

/// The thread that says it, told of each change of focus.
static TELL: OnceLock<Sender<Heard>> = OnceLock::new();
/// This window, for the threads that ask it for a beat or bring it forward.
static WINDOW: OnceLock<AppHandle<State>> = OnceLock::new();

/// `main`, before the window runs.
pub fn reach(window: AppHandle<State>) {
    let _ = WINDOW.set(window);
}

/// `App::on_focus`: the window gained or lost focus.
pub fn focus_changed(_s: &mut State, focused: bool, _tx: &mut Tx) {
    if let Some(tell) = TELL.get() {
        let _ = tell.send(Heard::Focus(focused));
    }
}

/// Says where the person is to the app behind `door`, from now on and off the main thread.
pub fn tell_the_app(door: Door) {
    let (tell, heard) = mpsc::channel();
    if TELL.set(tell.clone()).is_err() {
        return;
    }
    std::thread::spawn(move || {
        let mut speaker = Speaker::default();
        loop {
            match speaker.next(heard.recv_timeout(BEAT)) {
                // A word that does not arrive is said again at the next beat.
                Next::Say(said) => {
                    if let Some((url, token)) = door.now() {
                        let _ = source::call_waiting(&url, &token, SAY_WHERE, &json!({ "focused": said }), Duration::from_secs(2));
                    }
                }
                Next::AskForABeat => speaker.asked = ask_for_a_beat(&tell),
                Next::Wait => {}
                Next::Stop => return,
            }
        }
    });
}

/// Asks this window's own thread for a beat, behind whatever it is doing.
fn ask_for_a_beat(tell: &Sender<Heard>) -> bool {
    let Some(window) = WINDOW.get() else {
        return false;
    };
    let tell = tell.clone();
    // Nothing on screen changes, so nothing is drawn again.
    window.update_if(move |_, _| {
        let _ = tell.send(Heard::Beat);
        false
    });
    true
}

/// What a push of `OPEN_HERE` names.
#[derive(Debug, PartialEq, Default)]
pub struct Opening {
    id: Option<String>,
    product: Option<String>,
}

/// A line of the door's event stream that asks this window to come forward, with what it names.
/// None for every other push.
pub fn asked_to_open(line: &str) -> Option<Opening> {
    let push: Value = serde_json::from_str(line.strip_prefix("data:")?.trim()).ok()?;
    if push.get("channel")?.as_str()? != OPEN_HERE {
        return None;
    }
    let named = push.get("args").and_then(|args| args.get(0));
    let text = |key: &str| named.and_then(|n| n.get(key)).and_then(Value::as_str).filter(|s| !s.is_empty()).map(str::to_string);
    Some(Opening { id: text("id"), product: text("product") })
}

impl Opening {
    /// The thread to open, as this window's snapshot has it: none when the press named none, or
    /// one that has gone since, which the web window does not open either.
    fn thread(&self, snap: &Snapshot) -> Option<(String, String)> {
        let id = self.id.as_deref()?;
        let item = snap.items.iter().find(|i| i.id == id && self.product.as_deref().is_none_or(|p| p == i.product))?;
        Some((item.id.clone(), item.product.clone()))
    }
}

/// Opens what the app asked for and brings the window forward, from the thread that follows the
/// door. The conversation opens in the same turn of the window's loop that brings it forward.
pub fn come_forward(opening: Opening) {
    let Some(window) = WINDOW.get() else {
        return;
    };
    window.update(move |s, _| {
        if let Some((id, product)) = opening.thread(&s.snap) {
            s.open_thread(&id, &product);
        }
    });
    window.open();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quiet() -> Result<Heard, RecvTimeoutError> {
        Err(RecvTimeoutError::Timeout)
    }

    #[test]
    fn a_change_of_focus_is_said_at_once_and_every_beat_says_where_the_window_is_once_its_own_thread_answers() {
        let mut s = Speaker::default();
        assert_eq!(s.next(Ok(Heard::Focus(true))), Next::Say(true));
        assert_eq!(s.next(quiet()), Next::AskForABeat);
        assert_eq!(s.next(Ok(Heard::Beat)), Next::Say(true));
        assert_eq!(s.next(Ok(Heard::Focus(false))), Next::Say(false));
        // Elsewhere it beats too: the app sends a press only to a window it has heard lately.
        assert_eq!(s.next(quiet()), Next::AskForABeat);
        assert_eq!(s.next(Ok(Heard::Beat)), Next::Say(false));
        assert_eq!(s.next(Err(RecvTimeoutError::Disconnected)), Next::Stop);
    }

    #[test]
    fn a_window_whose_own_thread_does_not_answer_stops_beating_until_it_does() {
        let mut s = Speaker::default();
        s.next(Ok(Heard::Focus(true)));
        assert_eq!(s.next(quiet()), Next::AskForABeat);
        // Hung: the beats that pass ask nothing more and say nothing, so the app stops believing
        // the word three beats on.
        assert_eq!(s.next(quiet()), Next::Wait);
        assert_eq!(s.next(quiet()), Next::Wait);
        // Its thread answers at last, and the beats go on.
        assert_eq!(s.next(Ok(Heard::Beat)), Next::Say(true));
        assert_eq!(s.next(quiet()), Next::AskForABeat);
    }

    #[test]
    fn a_push_to_come_forward_names_the_thread_and_no_other_push_is_one() {
        let line = r#"data: {"channel":"zero:open-in-native","args":[{"id":"w-1","product":"lantern"}]}"#;
        assert_eq!(asked_to_open(line), Some(Opening { id: Some("w-1".into()), product: Some("lantern".into()) }));
        // The tray knows the thread and not its project.
        let line = r#"data: {"channel":"zero:open-in-native","args":[{"id":"w-1","product":null}]}"#;
        assert_eq!(asked_to_open(line), Some(Opening { id: Some("w-1".into()), product: None }));
        // The island itself, pressed: forward as it is.
        for line in [r#"data: {"channel":"zero:open-in-native","args":[{"id":null,"product":null}]}"#, r#"data: {"channel":"zero:open-in-native","args":[]}"#] {
            assert_eq!(asked_to_open(line), Some(Opening::default()), "{line}");
        }
        for line in [r#"data: {"channel":"zero:changed","args":[]}"#, "data: not json", ": beat", "retry: 1000", ""] {
            assert_eq!(asked_to_open(line), None, "{line}");
        }
    }

    #[test]
    fn the_thread_opened_is_the_one_the_snapshot_has_by_that_id_and_project() {
        let snap: Snapshot = serde_json::from_value(json!({ "items": [
            { "id": "w-1", "product": "lantern", "status": "blocked" },
            { "id": "w-2", "product": "harbor", "status": "done" },
        ] }))
        .unwrap();
        let named = |id: Option<&str>, product: Option<&str>| Opening { id: id.map(str::to_string), product: product.map(str::to_string) };
        let thread = |id: &str, product: &str| Some((id.to_string(), product.to_string()));
        assert_eq!(named(Some("w-1"), Some("lantern")).thread(&snap), thread("w-1", "lantern"));
        // Its project is taken from the snapshot when the press did not know it.
        assert_eq!(named(Some("w-2"), None).thread(&snap), thread("w-2", "harbor"));
        // A thread gone since, or under another project, is not opened; nor is anything when
        // the press named no thread. The window still comes forward.
        assert_eq!(named(Some("w-9"), None).thread(&snap), None);
        assert_eq!(named(Some("w-1"), Some("harbor")).thread(&snap), None);
        assert_eq!(named(None, None).thread(&snap), None);
    }
}
