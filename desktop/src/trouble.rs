//! The row that says tasks have not been able to run, opened and put away
//! (`renderer/src/trouble-row.ts`, and its uses in `App.tsx` and `components/Focus.tsx`).
//!
//! Opened, it is a conversation's page with no conversation behind it: the strip with its name,
//! Done and the dots, and under it, bare, which tasks are stopped and what each one waits on,
//! with no reply box (`made` in `Focus.tsx`). The rules are `rules/trouble_row.rs`.
//!
//! E on it, Done in its strip, or Mark done or Approve and run in the command bar, put it away
//! and stop nothing and start nothing: it comes back by itself the moment something new gets
//! stuck, since what is kept is when the pile it was put away against began
//! (`closeTroubleRow`). The web window keeps that in its own storage (`zero.troubleClosed`), so
//! both windows read it from there, this one through the app's door (`zero:window-keeps`), and
//! this one asks the web window to write it (`zero:trouble-closed-here`, a channel of the
//! desktop app's door alone), three seconds after the press as every close is, and again with
//! what it was before when Z takes it back. Until the web window's record says what this window
//! wrote, this window holds it itself, so the row does not come back in between.

use rune::{El, ScrollSpec, Tx, col, el};
use serde_json::{Value, json};

use crate::data::{Item, Local};
use crate::files::Files;
use crate::markdown;
use crate::rules::trouble_row::TROUBLE_ID;
use crate::rules::writes;
use crate::{State, View, write};

/// The desktop app's door channel that writes, in the web window, when the pile put away began.
pub const CLOSED_HERE: &str = "zero:trouble-closed-here";

/// What is said once it is put away, and what Z then says and asks (`closeTroubleRow`).
pub const DONE_SAID: &str = "Done: what is not running";
pub const BACK_SAID: &str = "Back in your inbox: what is not running";
pub const BACK_UNDOES: &str = "put the stopped tasks back in your inbox";

/// What this window wrote of the put-away, until the web window's own record says the same.
#[derive(Default)]
pub struct Kept {
    wanted: Option<f64>,
}

/// What the web window keeps, with this window's own put-away laid over it until the two agree.
pub fn lay_over(kept: &mut Kept, local: &mut Local) {
    match kept.wanted {
        Some(at) if local.trouble_closed == at => kept.wanted = None,
        Some(at) => local.trouble_closed = at,
        None => {}
    }
}

/// What is written: the pile begun at `since` is put away, or none is when it is 0.
pub fn closed(since: f64) -> (&'static str, Value) {
    (CLOSED_HERE, json!({ "since": since }))
}

fn holds(s: &mut State, at: f64) {
    s.trouble.wanted = Some(at);
    s.local.trouble_closed = at;
    s.refresh();
}

impl State {
    /// Puts the row away: out of the lists now, written in three seconds unless Z comes first.
    /// From inside it, on to the next thread that needs the person.
    pub(crate) fn put_the_trouble_away(&mut self, tx: &mut Tx) {
        // Only while there is a pile to put away (`if (since)`).
        let Some(since) = self.snap.supervisor.rest.get("spawnTrouble").and_then(|t| t.get("since")).and_then(Value::as_f64).filter(|since| *since != 0.0) else {
            return;
        };
        let was = self.local.trouble_closed;
        let inside = matches!(&self.view, View::Thread { id, product } if id == TROUBLE_ID && product.is_empty());
        let at = self.lists.needs.iter().position(|r| r.id == TROUBLE_ID && r.product.is_empty());
        let next = at.and_then(|at| self.lists.needs.get(at + 1).or_else(|| at.checked_sub(1).and_then(|a| self.lists.needs.get(a)))).map(|r| (r.id.clone(), r.product.clone()));
        let (channel, payload) = closed(since);
        self.hold(
            write::Held {
                channel,
                payload,
                sent: write::Sent::Trouble { closed: since, was, undo: false },
                said: DONE_SAID.to_string(),
                at: tx.clock() + writes::UNDO_GRACE_MS / 1000.0,
                thread: (TROUBLE_ID.to_string(), String::new()),
                from_inside: inside,
                closes: None,
                then: Vec::new(),
                mark: None,
            },
            tx,
        );
        holds(self, since);
        if inside {
            let now = tx.clock();
            self.move_on_to(next, now);
        }
    }

    /// Z inside the three seconds: nothing was written, and the row is back as it was.
    pub(crate) fn trouble_taken_back(&mut self, was: f64) {
        self.trouble.wanted = None;
        self.local.trouble_closed = was;
    }
}

/// What the web window said to the put-away, or to Z taking it back.
pub fn landed(s: &mut State, landed: write::Landed, tx: &mut Tx) {
    let write::Sent::Trouble { closed: at, was, undo } = landed.sent.clone() else {
        return;
    };
    if undo {
        // Z's own line, or its refusal, are the pile's (`undo_landed`).
        if landed.refusal().is_none() {
            holds(s, at);
        }
        return s.undo_landed(landed, tx);
    }
    if let Some(why) = landed.refusal() {
        s.trouble.wanted = None;
        s.local.trouble_closed = was;
        s.refresh();
        return s.tell(&refused(&why), tx);
    }
    let (channel, payload) = closed(was);
    s.undos.push(write::Undo::plain(BACK_SAID.to_string(), BACK_UNDOES.to_string(), vec![(channel, payload)], write::Sent::Trouble { closed: was, was: at, undo: true }, (TROUBLE_ID.to_string(), String::new()), false));
}

/// What is said when the web window did not take it. A door that has no such channel is an app
/// started before it had one.
pub fn refused(why: &str) -> String {
    if why.contains("404") { "Not done: restart Mercury first, so it can take that from this window.".to_string() } else { format!("Not done: {why}") }
}

/// The opened row: what it says, bare, where a conversation would be, the column's measure
/// handed over by the conversation's page (`.outcome`: 28 points over it, 8 under).
pub fn page(s: &State, item: &Item, (page_w, page_h): (f32, f32), (size, column, indent, right): (f32, f32, f32, f32)) -> El {
    let files = Files::of(s, item);
    let (said, top, bottom) = markdown::stack(markdown::blocks(item.body.as_deref().unwrap_or(""), size, column, &files), false);
    let (body, _, under) = markdown::stack(vec![markdown::block(top.max(28.0), bottom.max(8.0), said)], false);
    col().w(page_w).h(page_h).child(
        col().key("trouble-page").grow(1.0).min_h(0.0).scroll(ScrollSpec { stick_bottom: false }).child(el().pt(28.0 + top.max(28.0)).pl(indent).pr(right).pb(40.0 + under).child(body.key("trouble-said").w(column).selection_group(true))),
    )
}

#[cfg(test)]
mod tests {
    use rune::{Key, Runtime};
    use serde_json::json;

    use super::*;
    use crate::State;
    use crate::feel::tests::{key, run, window_dressed};
    use crate::write::{Landed, Sent};

    const MIN: f64 = 60_000.0;

    /// The invented inbox with three tasks stopped since an hour and a half ago, two of them
    /// on a signed-out login, its row the first in Needs you; `args` as the command line has them.
    fn stuck(name: &str, args: &[&str]) -> Runtime<State> {
        window_dressed(name, args, "/fixtures/storefront", "Done, and tested. Merge it?", |fixture| {
            let now = fixture["now"].as_f64().unwrap();
            fixture["snapshot"]["supervisor"]["spawnTrouble"] = json!({
                "since": now - 95.0 * MIN, "cause": "mixed", "message": "", "remedy": "", "retryAt": 0, "count": 3, "resetsAt": "6pm",
                "byCause": [
                    { "cause": "signed-out", "count": 2, "what": "2 stopped by a signed-out Claude login", "next": "Type /login in a terminal and they pick straight up.", "act": "Sign in", "hers": true },
                    { "cause": "at-limit", "count": 1, "what": "1 waiting on your Claude limit", "next": "They start again on their own at 6pm.", "act": null, "hers": false },
                ],
                "ids": ["w-feel1", "w-feel2", "w-gone"],
                "stopped": [{ "id": "w-feel1", "cause": "signed-out" }, { "id": "w-gone", "cause": "signed-out" }, { "id": "w-feel2", "cause": "at-limit" }],
            });
        })
    }

    fn since(rt: &Runtime<State>) -> f64 {
        rt.state.snap.supervisor.rest["spawnTrouble"]["since"].as_f64().unwrap()
    }

    fn said(rt: &Runtime<State>) -> Vec<String> {
        rt.elements().iter().filter_map(|e| e.text.clone()).collect()
    }

    #[test]
    fn opening_the_row_says_which_tasks_are_stopped_and_what_each_waits_on_with_no_reply_box() {
        let mut rt = stuck("trouble-open", &["--thread", "trouble"]);
        run(&mut rt, 0.1);
        assert!(matches!(&rt.state.view, View::Thread { id, product } if id == TROUBLE_ID && product.is_empty()));
        let words = said(&rt);
        assert!(!words.iter().any(|t| t == "This conversation is not here any more."));
        for line in ["2 need a login · 1 back at 6pm", "2 stopped by a signed-out Claude login.", "Type /login in a terminal and they pick straight up.", "Invented thread number 1 (Storefront)", "A task that is no longer in your inbox", "1 waiting on your Claude limit.", "Invented thread number 2 (Storefront)"] {
            assert!(words.iter().any(|t| t.contains(line)), "{line} in {words:?}");
        }
        // Its name is the strip's, and nothing on the page asks for a reply.
        assert!(words.iter().any(|t| t == "3 tasks have not been able to run"));
        assert!(rt.find("reply").is_none());
        // R opens nothing, and L says there is nothing to put off.
        key(&mut rt, Key::Text("r".into()));
        assert!(!rt.state.replying);
        key(&mut rt, Key::Text("l".into()));
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Nothing to put off here. Press E to close it until something else stops."));
    }

    #[test]
    fn e_puts_it_away_writes_it_to_the_web_window_three_seconds_later_and_z_puts_it_back() {
        let mut rt = stuck("trouble-e", &["--view", "inbox"]);
        rt.state.pretend_sent = true;
        run(&mut rt, 0.1);
        assert_eq!(rt.state.lists.needs[0].id, TROUBLE_ID);
        let since = since(&rt);
        key(&mut rt, Key::Text("e".into()));
        // Out of the list at once, the line said, and nothing written yet.
        assert!(!rt.state.lists.needs.iter().any(|r| r.id == TROUBLE_ID));
        assert_eq!(rt.state.toast.as_ref().map(|t| (t.line.as_str(), t.undo)), Some((DONE_SAID, true)));
        assert!(rt.state.tried.is_empty());
        // Z inside the three seconds: back, and nothing ever written.
        key(&mut rt, Key::Text("z".into()));
        assert!(rt.state.lists.needs.iter().any(|r| r.id == TROUBLE_ID));
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Undone. Still open: what is not running"));
        assert!(rt.state.tried.is_empty());
        // Again, and let the three seconds pass: the web window is asked to keep it.
        key(&mut rt, Key::Text("e".into()));
        run(&mut rt, 3.2);
        assert_eq!(rt.state.tried, vec![(CLOSED_HERE, json!({ "since": since }))]);
        rt.update(|s, tx| s.landed(Landed { sent: Sent::Trouble { closed: since, was: 0.0, undo: false }, answer: Ok(json!({ "ok": true })), snapshot: None }, tx));
        // The web window's record still says nothing: this window holds it until it agrees.
        rt.update(|s, _| s.keep(Local::default()));
        assert!(!rt.state.lists.needs.iter().any(|r| r.id == TROUBLE_ID));
        rt.update(|s, _| s.keep(Local { trouble_closed: since, ..Local::default() }));
        assert!(rt.state.trouble.wanted.is_none());
        // Z on the pile writes what it was, and says the web window's line once it is taken.
        rt.state.tried.clear();
        key(&mut rt, Key::Text("z".into()));
        assert_eq!(rt.state.tried, vec![(CLOSED_HERE, json!({ "since": 0.0 }))]);
        rt.update(|s, tx| s.landed(Landed { sent: Sent::Trouble { closed: 0.0, was: since, undo: true }, answer: Ok(json!({ "ok": true })), snapshot: None }, tx));
        assert!(rt.state.lists.needs.iter().any(|r| r.id == TROUBLE_ID));
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some(BACK_SAID));
    }

    #[test]
    fn done_in_its_strip_puts_it_away_and_goes_on_to_the_next_thread() {
        let mut rt = stuck("trouble-done", &["--thread", "trouble"]);
        rt.state.pretend_sent = true;
        run(&mut rt, 0.1);
        rt.update(|s, tx| s.close_thread(false, tx));
        assert!(matches!(&rt.state.view, View::Thread { id, .. } if id != TROUBLE_ID), "on to the next thread");
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some(DONE_SAID));
        // A put-away the web window did not take puts the row back, and says why.
        run(&mut rt, 3.2);
        let since = since(&rt);
        rt.update(|s, tx| s.landed(Landed { sent: Sent::Trouble { closed: since, was: 0.0, undo: false }, answer: Err("http://127.0.0.1:1/api/zero%3Atrouble-closed-here: status code 404".into()), snapshot: None }, tx));
        assert!(rt.state.lists.needs.iter().any(|r| r.id == TROUBLE_ID));
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Not done: restart Mercury first, so it can take that from this window."));
        assert!(rt.state.undos.is_empty());
    }

    #[test]
    fn it_comes_back_by_itself_when_a_new_pile_begins() {
        let mut rt = stuck("trouble-back", &["--view", "inbox"]);
        let since = since(&rt);
        rt.update(|s, _| s.keep(Local { trouble_closed: since, ..Local::default() }));
        rt.update(|s, _| s.refresh());
        assert!(!rt.state.lists.needs.iter().any(|r| r.id == TROUBLE_ID));
        rt.state.snap.supervisor.rest.get_mut("spawnTrouble").unwrap()["since"] = json!(since + 60_000.0);
        rt.update(|s, _| s.refresh());
        assert_eq!(rt.state.lists.needs[0].id, TROUBLE_ID);
    }
}
