//! Whether this window is the one in front, said to the desktop app.
//!
//! The app decides whether to put up a banner, or its island in the notch, from whether the
//! person is at it, and it went by its own window's focus alone: while the person worked here it
//! took them to be in another app. So this window says so each time it gains or loses focus, on
//! a channel only the desktop app's own door answers (`main/local-door.mts`), and again every beat
//! while it stays in front. The app stops believing a word three beats old, so a window that hung
//! stops counting; one that quits or crashes hangs up the door's event stream, and is forgotten
//! then (`main/native-window.mts`).

use std::sync::OnceLock;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::time::Duration;

use rune::Tx;
use serde_json::json;

use crate::State;
use crate::source::{self, Door};

/// Where this window says whether it is in front: `{ "focused": true }`.
pub const SAY_WHERE: &str = "zero:native-window";

/// How often that is said again while in front: `NATIVE_BEAT_MS` in `shared/notify-rules.mts`.
const BEAT: Duration = Duration::from_secs(5);

/// The thread that says it, told of each change of focus.
static FOCUS: OnceLock<Sender<bool>> = OnceLock::new();

/// `App::on_focus`: the window gained or lost focus.
pub fn focus_changed(_s: &mut State, focused: bool, _tx: &mut Tx) {
    if let Some(tell) = FOCUS.get() {
        let _ = tell.send(focused);
    }
}

/// Says where the person is to the app behind `door`, from now on and off the main thread.
pub fn tell_the_app(door: Door) {
    let (tell, heard) = mpsc::channel();
    if FOCUS.set(tell).is_err() {
        return;
    }
    std::thread::spawn(move || {
        let mut focused = false;
        loop {
            let got = heard.recv_timeout(BEAT);
            if got == Err(RecvTimeoutError::Disconnected) {
                return;
            }
            let Some(say) = word(&mut focused, got) else { continue };
            // A word that does not arrive is said again at the next beat, or was a window leaving
            // the front, which the app stops believing three beats on.
            if let Some((url, token)) = door.now() {
                let _ = source::call_waiting(&url, &token, SAY_WHERE, &json!({ "focused": say }), Duration::from_secs(2));
            }
        }
    });
}

/// What to say after waiting a beat for news of the window's focus: the news when there is some,
/// the same again when the beat passed in front, and nothing when it passed elsewhere.
fn word(focused: &mut bool, got: Result<bool, RecvTimeoutError>) -> Option<bool> {
    match got {
        Ok(now) => {
            *focused = now;
            Some(now)
        }
        Err(RecvTimeoutError::Timeout) => focused.then_some(true),
        Err(RecvTimeoutError::Disconnected) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_change_of_focus_is_said_at_once_and_a_beat_in_front_says_it_again() {
        let mut focused = false;
        assert_eq!(word(&mut focused, Err(RecvTimeoutError::Timeout)), None);
        assert_eq!(word(&mut focused, Ok(true)), Some(true));
        assert_eq!(word(&mut focused, Err(RecvTimeoutError::Timeout)), Some(true));
        assert_eq!(word(&mut focused, Err(RecvTimeoutError::Timeout)), Some(true));
        assert_eq!(word(&mut focused, Ok(false)), Some(false));
        // Elsewhere a beat says nothing: only a window in front has a word to keep fresh.
        assert_eq!(word(&mut focused, Err(RecvTimeoutError::Timeout)), None);
        assert_eq!(word(&mut focused, Err(RecvTimeoutError::Disconnected)), None);
    }
}
