//! Mercury's desktop window, drawn natively. Phase 1: it reads. Home, the Inbox and one
//! conversation, from the same snapshot the web window gets, and kept current by the engine.
//!
//!   cargo run -- --door http://127.0.0.1:7777 --token <key>          # live, from the engine
//!   cargo run -- --fixture shots/home-fixture.json --snap shots/home.png --size 1280x760 --scale 2
//!   cargo run -- --fixture shots/inbox-fixture.json --view inbox --snap shots/inbox.png --size 1280x760 --scale 2
//!   cargo run -- --fixture shots/inbox-later-kinds-fixture.json --view inbox --tab later --snap shots/inbox-later-kinds.png --size 1280x760 --scale 2
//!   cargo run -- --fixture shots/thread-fixture.json --thread --snap shots/thread.png --size 1280x760 --scale 2
//!
//! Every size, colour, border and shadow was read off the running renderer, so a screen here
//! can be laid over the web app's and compared pixel by pixel (`tools/`).

#![forbid(unsafe_code)]

mod chrome;
mod data;
mod files;
mod home;
mod icons;
mod inbox;
mod keys;
mod later;
mod markdown;
mod palette;
mod pictures;
mod reads;
mod rules;
mod sky;
mod source;
mod talk;
mod theme;
mod thread;
mod write;

use std::collections::{HashMap, HashSet};

use rune::{App, AppSpec, Cx, El, FontSmoothing, Latest, Pixels, ScrollSpec, TextEdit, Tx, Waker, el};
use serde_json::{Map, Value};

use data::{Lists, Local, Snapshot, Tab};
use rules::writes;
use source::{Mailbox, Source};

/// Which screen the card is showing.
#[derive(Clone, PartialEq, Debug)]
pub enum View {
    Home,
    Inbox,
    Thread { id: String, product: String },
}

/// Both skies for one card size, painted off the main thread, so changing screen never waits.
struct Painted {
    home: (u32, u32, Vec<u8>),
    inbox: (u32, u32, Vec<u8>),
    points: (f32, f32),
}

pub struct State {
    /// The window, in points.
    pub size: (f32, f32),
    /// Home's sky, and the Inbox's (drawn at the card's foot; `inbox.rs` turns it over).
    pub sky_home: Pixels,
    pub sky_inbox: Pixels,
    /// The card size the skies on screen were painted for.
    pub sky_size: (f32, f32),
    painting: Latest<Painted>,
    pub snap: Snapshot,
    /// What the screens draw, worked out from `snap` whenever it or the minute changes.
    pub lists: Lists,
    pub local: Local,
    /// The rest of a fixture file, for the screen that knows what it is (a thread's words).
    pub fixture: Map<String, Value>,
    /// The clock the rows read, in milliseconds. A fixture brings its own.
    pub now: f64,
    fixed_clock: bool,
    pub view: View,
    /// The Inbox's open tab.
    pub inbox_tab: Tab,
    /// The row of the open tab the keys act on: J and K move it, Enter opens it.
    pub picked: usize,
    /// The row of Home's Needs you the keys or the pointer are on, when either is.
    pub home_pick: Option<usize>,
    /// The threads opened in this window and when, until the web window's own record of what
    /// has been read says the same (`saw`).
    opened: HashMap<String, f64>,
    /// The open conversation's reply box: open, or folded to its one line.
    pub replying: bool,
    /// What is written in it. Words written in one conversation wait in `drafts` while
    /// another is open.
    pub reply: TextEdit,
    drafts: HashMap<String, String>,
    /// Which of the open conversation's options the arrows are on, by its place in the list.
    pub option: Option<usize>,
    /// What the person has unfolded in the conversations they opened.
    pub reading: thread::Reading,
    /// What the engine answered about each conversation opened so far, by `product/id` and
    /// then by channel, the shape a fixture's `threads` has (`docs/thread-data.md`).
    pub threads: HashMap<String, Value>,
    /// A conversation being asked for, and when it is owed a read (`reads.rs`).
    asking: Latest<reads::Answer>,
    reads: reads::Reads,
    /// The pictures the open conversation names, read off the disk (`pictures.rs`).
    pub pictures: pictures::Pictures,
    pub history: Vec<View>,
    pub source: Source,
    mailbox: Mailbox,
    /// Why the engine could not be read, when it could not.
    pub error: Option<String>,
    /// What this window is sending the engine, and what came back.
    outbox: write::Outbox,
    /// One line about something sent that the engine did not take, until the next thing is done.
    pub refused: Option<String>,
    /// What was done and waits three seconds to be written, and the last close written.
    held: Option<write::Held>,
    last: Option<write::Undo>,
    /// The line about it.
    pub toast: Option<write::Toast>,
    /// The threads closed here whose close the engine has not told back yet: out of the lists
    /// from the moment E is pressed.
    hidden: HashSet<String>,
    /// The Later picker L opens.
    pub later: later::Picker,
    /// The command bar ⌘K opens (`palette.rs`).
    pub palette: palette::Bar,
}

impl State {
    pub fn go(&mut self, to: View) {
        if self.view != to {
            self.put_the_reply_away();
            self.history.push(std::mem::replace(&mut self.view, to));
            self.take_the_reply_out();
        }
    }
    pub fn back(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.put_the_reply_away();
            self.view = prev;
            self.take_the_reply_out();
        }
    }
    /// The open conversation's row, as the engine last told it.
    pub fn open_item(&self) -> Option<&data::Item> {
        let View::Thread { id, product } = &self.view else {
            return None;
        };
        self.snap.items.iter().find(|i| &i.id == id && &i.product == product)
    }
    /// Leaving a conversation folds its reply box and drops the arrow off its options. Words
    /// written in the box wait for the next visit.
    fn put_the_reply_away(&mut self) {
        if let View::Thread { id, product } = &self.view {
            let key = format!("{product}/{id}");
            if self.reply.text.trim().is_empty() {
                self.drafts.remove(&key);
            } else {
                self.drafts.insert(key, self.reply.text.clone());
            }
        }
        self.replying = false;
        self.reply = TextEdit::new("");
        self.option = None;
        self.refused = None;
    }
    fn take_the_reply_out(&mut self) {
        if let View::Thread { id, product } = &self.view {
            if let Some(words) = self.drafts.get(&format!("{product}/{id}")) {
                self.reply = TextEdit::new(words.clone());
            }
        }
    }
    /// Opens the reply box with the caret in it (R, or a press on the box).
    pub fn open_reply(&mut self, tx: &mut Tx) {
        if self.open_item().is_none() {
            return;
        }
        self.replying = true;
        self.option = None;
        self.reply.focused = true;
        tx.focus_when_shown("reply", false);
    }
    /// Folds the reply box back to its one line, keeping what was written.
    pub fn fold_reply(&mut self) {
        self.replying = false;
        self.reply.focused = false;
    }
    /// One write to the engine. A fixture has nobody to send to, and says nothing of it.
    fn send(&mut self, channel: &'static str, payload: Value, sent: write::Sent) -> bool {
        self.send_all(vec![(channel, payload)], sent)
    }
    /// Several writes that are one thing done, sent in order.
    fn send_all(&mut self, writes: Vec<writes::Write>, sent: write::Sent) -> bool {
        self.refused = None;
        let Source::Door(door) = &self.source else {
            return false;
        };
        if self.outbox.send_in_order(door, writes, sent) {
            return true;
        }
        self.refused = Some("Mercury is not running, so this was not sent.".to_string());
        false
    }
    /// One line about what was just done, for as long as the web window keeps its own up.
    fn say(&mut self, line: String, undo: bool, tx: &Tx) {
        self.toast = Some(write::Toast { line, undo, until: tx.clock() + writes::TOAST_MS / 1000.0 });
    }
    /// A line that only tells: nothing was done, so there is nothing for Z.
    pub fn tell(&mut self, line: &str, tx: &Tx) {
        self.say(line.to_string(), false, tx);
    }
    /// Does something now and writes it in three seconds, unless Z comes first
    /// (`deferCommit` in `renderer/src/App.tsx`). One thing waits at a time: whatever was
    /// waiting before is written at once.
    fn hold(&mut self, held: write::Held, tx: &Tx) {
        self.write_what_waits();
        self.say(held.said.clone(), true, tx);
        if let Some(item) = &held.closes {
            // Out of the lists now; the write can wait, the list cannot.
            self.hidden.insert(item.id.clone());
            self.refresh();
        }
        self.held = Some(held);
    }
    /// Writes what is waiting: its time came, or something else was done after it. A close can
    /// still be taken back afterwards, by opening the thread as it was.
    fn write_what_waits(&mut self) {
        let Some(held) = self.held.take() else {
            return;
        };
        self.last = held.closes.as_ref().map(|item| write::Undo {
            said: format!("Reopened: {}", writes::clip_to_sentence(item.title.as_deref().unwrap_or(""), 48)),
            writes: vec![(writes::ANSWER, writes::reopen_as_it_was(item))],
            sent: write::Sent::Reopen { thread: format!("{}/{}", held.thread.1, held.thread.0) },
            thread: held.thread.clone(),
            opens: true,
        });
        self.send(held.channel, held.payload, held.sent);
    }
    /// Z: takes back what is still waiting, whole; else the last close this window wrote.
    pub fn undo(&mut self, tx: &mut Tx) {
        if let Some(held) = self.held.take() {
            if let Some(item) = &held.closes {
                self.hidden.remove(&item.id);
                self.refresh();
            }
            let (id, product) = held.thread.clone();
            let words = match &held.sent {
                write::Sent::Reply { words, .. } => Some(words.clone()),
                _ => None,
            };
            // Back on the thread it was done from, when it was done from inside one.
            if held.from_inside || words.is_some() {
                self.step_to_thread(&id, &product);
            }
            if let Some(words) = &words {
                self.reply = TextEdit::new(words.clone());
                self.open_reply(tx);
            }
            self.say(writes::said_undone(&held.said, words.is_some()), false, tx);
            return;
        }
        if let Some(last) = self.last.take() {
            let (id, product) = last.thread.clone();
            if self.send_all(last.writes, last.sent) {
                self.say(last.said, false, tx);
                // What it puts back in front of the person is opened, so Z is seen to have
                // done something (`brings` in `renderer/src/App.tsx`).
                if last.opens {
                    self.open_thread(&id, &product);
                }
            }
            return;
        }
        self.say(writes::NOTHING_TO_UNDO.to_string(), false, tx);
    }
    /// The clock for something done this instant. A fixture has its own.
    fn moment(&self) -> f64 {
        if self.fixed_clock { self.now } else { clock() }
    }
    fn item(&self, id: &str, product: &str) -> Option<&data::Item> {
        self.snap.items.iter().find(|i| i.id == id && i.product == product)
    }
    /// The rows the open Later picker offers.
    pub fn later_choices(&self) -> Vec<later::Choice> {
        let on = self.later.on.as_ref().and_then(|(id, product)| self.item(id, product));
        on.map(|item| later::choices(item, self.now)).unwrap_or_default()
    }
    /// L: opens the Later picker on a thread (`openSnooze` in `renderer/src/App.tsx`).
    pub fn open_later(&mut self, id: &str, product: &str, tx: &mut Tx) {
        // A row the app made itself has no ledger to keep a moment in.
        if product.is_empty() {
            self.say("Nothing to put off here. Press E to close it until something else stops.".to_string(), false, tx);
            return;
        }
        if self.item(id, product).is_none() {
            return;
        }
        // The moments it offers are counted from this minute, not from when the lists were last read.
        if !self.fixed_clock {
            self.now = clock();
        }
        self.later = later::Picker::open_on(Some((id.to_string(), product.to_string())), "");
        tx.focus_when_shown("later", false);
    }
    pub fn close_later(&mut self) {
        self.later = later::Picker::closed();
    }
    /// Enter in the Later picker, or a press on one of its rows (`at`). Typed words decide;
    /// with none, the lit row does; words that are not a time are refused and said so.
    pub fn pick_later(&mut self, at: Option<usize>) {
        let Some((id, product)) = self.later.on.clone() else {
            return;
        };
        let Some(item) = self.item(&id, &product).cloned() else {
            self.close_later();
            return;
        };
        let now = self.moment();
        let rows = later::choices(&item, now);
        let row = |at: usize| rows.get(at).map(|choice| (choice.ts, choice.name.to_string()));
        let picked = match at {
            Some(at) => row(at),
            None => {
                let read = rules::later::parse_when(&self.later.words.text, now);
                match rules::later::enter_means(&self.later.words.text, read.is_some()) {
                    rules::later::Enter::Typed => read,
                    rules::later::Enter::Selected => row(self.later.selected),
                    rules::later::Enter::Refuse => {
                        self.later.refused = true;
                        return;
                    }
                }
            }
        };
        let Some((until, back)) = picked else {
            return;
        };
        self.close_later();
        // "Now" on a thread already put off wakes it instead of timing it again.
        if until == 0.0 {
            self.bring_back(&id, &product);
            return;
        }
        self.write_what_waits();
        let (channel, payload) = writes::put_off(&item, until);
        let sent = write::Sent::Later {
            thread: format!("{product}/{id}"),
            said: writes::said_later(&rules::title::row_title(&item), &back),
            undo: writes::put_off(&item, item.number("snoozedUntil").unwrap_or(0.0)),
        };
        self.send(channel, payload, sent);
    }
    /// Brings a thread back from Later: E on its row there, or "Now" in the picker (`unsnooze`).
    pub fn bring_back(&mut self, id: &str, product: &str) {
        let Some(item) = self.item(id, product).cloned() else {
            return;
        };
        self.write_what_waits();
        let back = writes::bring_back(&item, self.moment());
        self.send_all(back.writes, write::Sent::Back { thread: format!("{product}/{id}"), said: back.said, undo: back.undo });
    }
    /// What a write the engine took still has to do on screen. Putting a thread off and
    /// bringing one back are said only now, as the web window says them only once the write
    /// stands: a line promising a row had gone, over a row still there, is worse than a wait.
    fn took(&mut self, landed: write::Landed, tx: &mut Tx) {
        let Some((product, id)) = landed.sent.thread().split_once('/').map(|(product, id)| (product.to_string(), id.to_string())) else {
            return;
        };
        let (said, undo) = match landed.sent {
            write::Sent::Later { thread, said, undo } => {
                // The engine answers with the row as it now stands. Its moment is laid over
                // the row in hand, so the lists move now and not at the next reading.
                if let (Ok(row), Some(item)) = (&landed.answer, self.snap.items.iter_mut().find(|i| i.id == id && i.product == product)) {
                    if let Some(until) = row.get("snoozedUntil") {
                        item.rest.insert("snoozedUntil".to_string(), until.clone());
                    }
                    if let (Some(wrote), Some(mine)) = (row.get("wrote").and_then(|w| w.get("snoozedUntil")), item.rest.get_mut("wrote").and_then(Value::as_object_mut)) {
                        mine.insert("snoozedUntil".to_string(), wrote.clone());
                    }
                }
                let undo = write::Undo {
                    said: writes::BACK_IN_YOUR_INBOX.to_string(),
                    writes: vec![undo],
                    sent: write::Sent::TakenBack { thread },
                    thread: (id.clone(), product.clone()),
                    opens: true,
                };
                (said, undo)
            }
            write::Sent::Back { thread, said, undo } => {
                let undo = write::Undo { said: writes::RETURNED_TO_LATER.to_string(), writes: undo, sent: write::Sent::TakenBack { thread }, thread: (id.clone(), product.clone()), opens: false };
                (said.to_string(), undo)
            }
            _ => return,
        };
        self.last = Some(undo);
        self.say(said, true, tx);
        self.refresh();
        // The thread has left the list the person was reading it from, so they are back on it.
        if matches!(&self.view, View::Thread { id: open, product: of } if *open == id && *of == product) {
            self.leave_thread();
        }
    }
    /// Whether an answer to this thread is waiting to be written: its options are spent on
    /// screen from the moment one is picked.
    pub fn answer_waits(&self, item: &data::Item) -> bool {
        self.held.as_ref().is_some_and(|h| h.closes.is_none() && h.thread.0 == item.id && h.thread.1 == item.product)
    }
    /// The project's name, as a line about a thread says where it went.
    fn project_name(&self, item: &data::Item) -> String {
        item.product_name.clone().or_else(|| self.snap.products.iter().find(|p| p.slug == item.product).map(|p| p.name.clone())).unwrap_or_else(|| item.product.clone())
    }
    /// Sends what is in the reply box to the thread's agent (Send, or ⌘Return). The box
    /// empties and folds at once; Z, or a refusal, puts the words back.
    pub fn send_reply(&mut self, tx: &mut Tx) {
        let words = self.reply.text.trim().to_string();
        let Some(item) = self.open_item().cloned() else {
            return;
        };
        if words.is_empty() {
            return;
        }
        let thread = format!("{}/{}", item.product, item.id);
        self.drafts.remove(&thread);
        self.reply = TextEdit::new("");
        self.replying = false;
        self.hold(
            write::Held {
                channel: writes::ANSWER,
                payload: writes::reply(&item, &words),
                sent: write::Sent::Reply { thread, words },
                said: writes::said_sent(&self.project_name(&item)),
                at: tx.clock() + writes::UNDO_GRACE_MS / 1000.0,
                thread: (item.id.clone(), item.product.clone()),
                from_inside: true,
                closes: None,
            },
            tx,
        );
    }
    /// Picks one of the options the open thread offers, by its place in the list: most are sent
    /// as a reply, and a few close the thread or open the box instead (`rules/writes.rs`).
    pub fn pick_option(&mut self, at: usize, tx: &mut Tx) {
        let Some(item) = self.open_item().cloned() else {
            return;
        };
        let Some(choice) = thread::live_options(&item).into_iter().nth(at) else {
            return;
        };
        let thread = format!("{}/{}", item.product, item.id);
        self.option = None;
        match writes::pick(&choice) {
            writes::Pick::Reply(words) => self.hold(
                write::Held {
                    channel: writes::ANSWER,
                    payload: writes::reply(&item, &words),
                    sent: write::Sent::Pick { thread },
                    said: writes::said_sent(&self.project_name(&item)),
                    at: tx.clock() + writes::UNDO_GRACE_MS / 1000.0,
                    thread: (item.id.clone(), item.product.clone()),
                    from_inside: true,
                    closes: None,
                },
                tx,
            ),
            writes::Pick::Close => self.close(&item.id, &item.product, false, tx),
            writes::Pick::CloseUnshipped => self.close(&item.id, &item.product, true, tx),
            // The web window asks before throwing work away. This window has no such question
            // yet, so it sends nothing rather than discard on one key.
            writes::Pick::AsksFirst => self.refused = Some("Throwing the work away asks first. Pick it in Mercury's own window.".to_string()),
            writes::Pick::NeedsWords(words) => {
                if self.reply.text.trim().is_empty() {
                    self.reply = TextEdit::new(words);
                }
                self.open_reply(tx);
            }
        }
    }
    /// Marks a thread done (E, or the button). It leaves the lists at once and is written in
    /// three seconds; from inside it, the person goes on to the next thread that needs them,
    /// as J would take them. The engine refuses a close that would strand work off main unless
    /// `unshipped` says it is meant, and its refusal is said in its own words.
    pub fn close(&mut self, id: &str, product: &str, unshipped: bool, tx: &mut Tx) {
        let Some(item) = self.snap.items.iter().find(|i| i.id == id && i.product == product).cloned() else {
            return;
        };
        // Already closed by the person: there is nothing left to mark. A thread its agent
        // finished still needs them, and E is how they say they have read it.
        if item.status == "done" && item.wrote("status").is_some_and(|w| w.source == "founder") {
            return;
        }
        let inside = matches!(&self.view, View::Thread { id: open, product: of } if open == id && of == product);
        // Looked up before the row is taken out of the list.
        let next = self.lists.needs.iter().position(|r| r.id == id && r.product == product).and_then(|at| self.lists.needs.get(at + 1).or_else(|| at.checked_sub(1).and_then(|a| self.lists.needs.get(a)))).map(|r| (r.id.clone(), r.product.clone()));
        self.hold(
            write::Held {
                channel: writes::ANSWER,
                payload: writes::close(&item, unshipped),
                sent: write::Sent::Close { thread: format!("{product}/{id}") },
                said: writes::said_done(item.title.as_deref().unwrap_or("")),
                at: tx.clock() + writes::UNDO_GRACE_MS / 1000.0,
                thread: (id.to_string(), product.to_string()),
                from_inside: inside,
                closes: Some(item),
            },
            tx,
        );
        if inside {
            match next {
                Some((id, product)) => self.step_to_thread(&id, &product),
                None => self.leave_thread(),
            }
        }
    }
    /// The open thread, closed.
    pub fn close_thread(&mut self, unshipped: bool, tx: &mut Tx) {
        if let View::Thread { id, product } = self.view.clone() {
            self.close(&id, &product, unshipped, tx);
        }
    }
    /// Opens a finished thread again (the Reopen button). Written at once: it starts work
    /// nobody has to wait three seconds for.
    pub fn reopen_thread(&mut self) {
        let Some(item) = self.open_item().cloned() else {
            return;
        };
        self.write_what_waits();
        let thread = format!("{}/{}", item.product, item.id);
        self.send(writes::REOPEN, writes::reopen(&item), write::Sent::Reopen { thread });
    }
    /// What the engine said to something sent. A refusal is said in one line, and refused
    /// words go back where the person can send them again.
    fn landed(&mut self, landed: write::Landed, tx: &mut Tx) {
        let Some(why) = landed.refusal() else {
            self.took(landed, tx);
            return;
        };
        // A close the engine did not make puts the row back in its list.
        if let write::Sent::Close { thread } = &landed.sent {
            if let Some((_, id)) = thread.split_once('/') {
                self.hidden.remove(id);
                self.last = None;
                self.refresh();
            }
        }
        if let write::Sent::Reply { thread, words } = &landed.sent {
            let here = matches!(&self.view, View::Thread { id, product } if &format!("{product}/{id}") == thread);
            if here && self.reply.text.trim().is_empty() {
                self.reply = TextEdit::new(words.clone());
                self.replying = true;
            } else {
                self.drafts.entry(thread.clone()).or_insert_with(|| words.clone());
            }
        }
        self.refused = Some(match landed.sent {
            write::Sent::Reply { .. } => format!("Could not send: {why} Your message is back in the reply box."),
            write::Sent::Pick { .. } => format!("Could not send that option: {why}"),
            write::Sent::Close { .. } => format!("Could not mark it done: {why}"),
            write::Sent::Reopen { .. } => format!("Could not reopen it: {why}"),
            write::Sent::Later { .. } => format!("Could not move it to Later: {why}"),
            write::Sent::Back { .. } => format!("Could not bring it back: {why}"),
            write::Sent::TakenBack { .. } => format!("Could not undo that: {why}"),
        });
    }
    /// Out of a conversation: to where the person came from, and one opened from nowhere came
    /// from the Inbox.
    pub fn leave_thread(&mut self) {
        if self.history.is_empty() { self.go(View::Inbox) } else { self.back() }
    }
    /// Another tab of the Inbox, with its first row picked (`setSelected(0)` in `App.tsx`).
    pub fn show_tab(&mut self, tab: Tab) {
        self.inbox_tab = tab;
        self.picked = 0;
    }
    /// Opens a conversation, which reads it (`setFocused` and `markSeen` in `App.tsx`).
    pub fn open_thread(&mut self, id: &str, product: &str) {
        self.go(View::Thread { id: id.to_string(), product: product.to_string() });
        self.saw(id, product);
    }
    /// From one conversation straight to another: Escape still goes back to where the first
    /// was opened from, not through every thread walked on the way.
    pub fn step_to_thread(&mut self, id: &str, product: &str) {
        self.put_the_reply_away();
        self.view = View::Thread { id: id.to_string(), product: product.to_string() };
        self.take_the_reply_out();
        self.saw(id, product);
    }
    /// Counts a thread as read from now. Only the web window keeps what has been read, in its
    /// own storage, so it is told the way a phone tells it (`zero:phone-seen`, which the app
    /// passes on to its window) and what it then keeps comes back with its other keeps. Until
    /// it does, this window holds the moment itself, so the dot does not come back in between.
    fn saw(&mut self, id: &str, product: &str) {
        let at = clock();
        self.opened.insert(id.to_string(), at);
        self.local.seen_at.insert(id.to_string(), at);
        if !self.local.seen.iter().any(|seen| seen == id) {
            self.local.seen.push(id.to_string());
        }
        self.refresh();
        if let Source::Door(door) = &self.source {
            if let Some((url, token)) = door.now() {
                let payload = serde_json::json!({ "product": product, "id": id, "at": at });
                // Nothing waits on the answer: a refusal leaves the thread unread in the web
                // window, and the next opening says it again.
                std::thread::spawn(move || {
                    let _ = source::call(&url, &token, "zero:phone-seen", &payload);
                });
            }
        }
    }
    /// What the web window keeps, with the threads opened here since laid over it. Once its
    /// own record of a thread is as new as this window's, this window's is let go.
    fn keep(&mut self, mut local: Local) {
        self.opened.retain(|id, at| local.seen_at.get(id).is_none_or(|theirs| theirs < at));
        for (id, at) in &self.opened {
            local.seen_at.insert(id.clone(), *at);
            if !local.seen.iter().any(|seen| seen == id) {
                local.seen.push(id.clone());
            }
        }
        self.local = local;
    }
    /// The rows J and K walk and Enter opens: the open tab's on the Inbox page.
    fn rows_under_the_keys(&self) -> usize {
        self.lists.under(self.inbox_tab).len()
    }
    pub fn card_size(&self) -> (f32, f32) {
        ((self.size.0 - chrome::CARD[0] - chrome::CARD[2]).max(1.0), (self.size.1 - chrome::CARD[1] - chrome::CARD[3]).max(1.0))
    }
    /// The page's box inside the window: under the card's top strip.
    pub fn page_size(&self) -> (f32, f32) {
        ((self.size.0 - chrome::CARD[0] - chrome::CARD[2]).max(1.0), (self.size.1 - chrome::page_top(&self.view) - chrome::CARD[3]).max(1.0))
    }
    fn refresh(&mut self) {
        self.lists = self.snap.lists(self.now, &self.local);
        // A thread closed here stays out of the lists until the engine tells the close back:
        // done, and by the person's hand. An agent's own "done" is what Needs you shows them.
        let snap = &self.snap;
        let closed_by_them = |i: &data::Item| i.status == "done" && i.wrote("status").is_some_and(|w| w.source == "founder");
        self.hidden.retain(|id| snap.items.iter().any(|i| &i.id == id && !closed_by_them(i)));
        if !self.hidden.is_empty() {
            let hidden = &self.hidden;
            for list in [&mut self.lists.needs, &mut self.lists.progress, &mut self.lists.later] {
                list.retain(|r| !hidden.contains(&r.id));
            }
        }
    }
    /// The read the open conversation is owed now, if any: its lines and its runs' logs, when it
    /// opens or changes, and while a run is up on every push (`pushed`) and every eight seconds.
    fn read_owed(&mut self, pushed: bool) -> Option<reads::Ask> {
        let (View::Thread { id, product }, Source::Door(door)) = (&self.view, &self.source) else {
            return None;
        };
        // With no door open yet there is nobody to ask; the next snapshot brings this back.
        let door = door.now()?;
        let key = format!("{product}/{id}");
        let changed = self.snap.items.iter().find(|i| &i.id == id && &i.product == product).and_then(|i| i.updated_at).unwrap_or(0.0);
        let run = self.snap.supervisor.running.iter().find(|r| &r.item_id == id && &r.product == product).map(|r| r.rest.get("startedAt").and_then(Value::as_f64).unwrap_or(0.0));
        // Once its middle is asked for, every log is read whole, once.
        let every = self.reading.whole(&key) && self.threads.get(&key).is_none_or(|t| t.get("zero:session-trace/whole").is_none());
        self.reads.owed(door, product, id, changed, run, pushed, every, self.asking.is_pending(), clock())
    }
    /// Asks for what the open conversation is owed, off the main thread: it lands in `poll`.
    fn ask_for_the_open_thread(&mut self, pushed: bool) {
        if let Some(ask) = self.read_owed(pushed) {
            self.asking.request(ask, reads::read);
        }
    }
    fn hang(&mut self, p: Painted) {
        self.sky_home.set(p.home.0, p.home.1, p.home.2);
        self.sky_inbox.set(p.inbox.0, p.inbox.1, p.inbox.2);
        self.sky_size = p.points;
    }
}

fn clock() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0.0, |d| d.as_millis() as f64)
}

fn paint_skies(points: (f32, f32)) -> Painted {
    let (w, h) = (points.0.floor() as f64, points.1.floor() as f64);
    let art = sky::ember();
    // Home's: dots from 40 points down, fully in 300 points later (`SkyBackdrop` in App.tsx).
    let home = sky::draw(&art, w, h, 40.0, 300.0, 0.0, 2);
    // The Inbox's: a band 190 points tall at the card's foot (`InboxSky`, shape "head").
    let start = (h - 190.0).round();
    let inbox = sky::draw(&art, w, h, start, 120.0, start - 60.0, 2);
    Painted { home, inbox, points }
}

const DEFAULT_SIZE: (f32, f32) = (1280.0, 760.0);

/// The store of pictures for this run. A still picture reads each file where it is asked
/// for, since it cannot wait; a window, or a run of frames, reads them off the main thread.
fn pictures_for(never: bool) -> pictures::Pictures {
    let run = rune::tooling::headless_run();
    pictures::Pictures::new(run.map_or(2.0, |r| r.scale()), run.is_some_and(|r| r.frames() == 0), never)
}

fn init() -> State {
    let args = rune::tooling::extra_args();
    let source = source::from_args(args);
    // A headless picture has its size before the first frame; a window reports its own.
    let size = rune::tooling::headless_run().map_or(DEFAULT_SIZE, |run| run.size());
    let mut s = State {
        size,
        sky_home: Pixels::new(),
        sky_inbox: Pixels::new(),
        sky_size: (0.0, 0.0),
        painting: Latest::default(),
        snap: Snapshot::default(),
        lists: Lists::default(),
        local: Local::default(),
        fixture: Map::new(),
        now: clock(),
        fixed_clock: false,
        view: match args.iter().position(|a| a == "--view").and_then(|i| args.get(i + 1)).map(String::as_str) {
            Some("inbox") => View::Inbox,
            _ => View::Home,
        },
        // `--tab later` opens the Inbox on that tab, for a picture of it.
        inbox_tab: Tab::from_args(args),
        picked: 0,
        home_pick: None,
        opened: HashMap::new(),
        replying: false,
        reply: TextEdit::new(""),
        drafts: HashMap::new(),
        option: None,
        reading: thread::Reading::default(),
        threads: HashMap::new(),
        asking: Latest::default(),
        reads: reads::Reads::default(),
        pictures: pictures_for(false),
        history: Vec::new(),
        source,
        mailbox: Mailbox::default(),
        error: None,
        outbox: write::Outbox::default(),
        refused: None,
        held: None,
        last: None,
        toast: None,
        hidden: HashSet::new(),
        later: later::Picker::closed(),
        palette: palette::Bar::closed(),
    };
    // The first skies and a fixture are in place before the first frame, so a picture is whole.
    let first = paint_skies(s.card_size());
    s.hang(first);
    if let Source::Fixture(path) = &s.source {
        match source::read_fixture(path) {
            Ok(fixture) => {
                s.snap = fixture.snapshot;
                s.local = fixture.local;
                s.fixture = fixture.rest;
                // A page photographed before its pictures came draws none of them either.
                if s.fixture.get("arriving").and_then(Value::as_bool) == Some(true) {
                    s.pictures = pictures_for(true);
                }
                if let Some(now) = fixture.now {
                    s.now = now;
                    s.fixed_clock = true;
                }
            }
            Err(e) => s.error = Some(e),
        }
    }
    // A headless picture cannot wait for a thread to bring the engine's answer, so it asks once here.
    if let (Source::Door(door), true) = (&s.source, rune::tooling::headless_run().is_some()) {
        match door.now() {
            Some((url, token)) => {
                match source::fetch_snapshot(&url, &token) {
                    Ok(snapshot) => s.snap = snapshot,
                    Err(e) => s.error = Some(e),
                }
                if let Ok(Some(local)) = source::fetch_keeps(&url, &token) {
                    s.local = local;
                }
            }
            None => s.error = Some("the app has not opened its door yet".to_string()),
        }
    }
    s.refresh();
    // A window opened straight onto the Inbox came there from Home, as far as Back is concerned.
    if s.view == View::Inbox {
        s.history.push(View::Home);
    }
    // `--thread` opens the first thread that needs the person, for a picture of it; `--thread <id>` that one.
    if let Some(at) = args.iter().position(|a| a == "--thread") {
        // Or the one its fixture says was open, for a picture of a thread elsewhere than first in Needs you.
        let pictured = s.fixture.get("thread").and_then(Value::as_str).map(str::to_string);
        let wanted = args.get(at + 1).filter(|a| !a.starts_with("--")).or(pictured.as_ref());
        let row = match wanted {
            Some(id) => s.lists.needs.iter().chain(&s.lists.progress).chain(&s.lists.later).chain(&s.lists.done).find(|r| &r.id == id),
            None => s.lists.needs.first(),
        };
        if let Some(r) = row {
            s.view = View::Thread { id: r.id.clone(), product: r.product.clone() };
        }
        // A picture taken with the reply box open, or with the arrow on an option, says so in
        // its fixture (`replying` is what was typed, `option` the option's number).
        if let Some(words) = s.fixture.get("replying").and_then(Value::as_str) {
            s.reply = TextEdit::new(words);
            s.reply.focused = true;
            s.replying = true;
        }
        s.option = s.fixture.get("option").and_then(Value::as_u64).and_then(|n| (n as usize).checked_sub(1));
        // And one taken with the Later picker open (`later` is what was typed into it,
        // `laterRefused` that Enter was pressed on words it could not read).
        if let (Some(words), View::Thread { id, product }) = (s.fixture.get("later").and_then(Value::as_str), &s.view) {
            s.later = later::Picker::open_on(Some((id.clone(), product.clone())), words);
            s.later.refused = s.fixture.get("laterRefused").and_then(Value::as_bool).unwrap_or(false);
        }
    }
    // A picture taken with the command bar open (`palette` is what was typed into it).
    s.palette.picture(&s.fixture);
    // Nor can it wait for a conversation behind the door to be read.
    if rune::tooling::headless_run().is_some()
        && let Some(ask) = s.read_owed(false)
        && let (key, Ok(answer)) = reads::read(ask)
    {
        s.threads.insert(key, answer);
    }
    s
}

/// A picture taken just after E says so in its fixture (`closed`): the same key is pressed
/// here once the window has its clock, on the open thread or on the Inbox's picked row.
fn press_what_the_picture_pressed(s: &mut State, tx: &mut Tx) {
    if s.fixture.remove("closed").is_some() {
        keys::close_under_the_keys(s, tx);
    }
}

fn setup(s: &mut State, wake: Waker) {
    s.painting.set_waker(wake.clone());
    s.asking.set_waker(wake.clone());
    s.outbox.wake_with(wake.clone());
    s.pictures.set_waker(wake.clone());
    if let Source::Door(door) = s.source.clone() {
        source::follow_door(door, s.mailbox.clone(), move || wake());
    }
}

fn poll(s: &mut State, at: f64, tx: &mut Tx) -> bool {
    let mut changed = false;
    if let Some(p) = s.painting.take() {
        s.hang(p);
        changed = true;
    }
    press_what_the_picture_pressed(s, tx);
    // What waited its three seconds is written, and a line that has been up long enough goes.
    if s.held.as_ref().is_some_and(|h| at >= h.at) {
        s.write_what_waits();
        changed = true;
    }
    if s.toast.as_ref().is_some_and(|t| at >= t.until) {
        s.toast = None;
        changed = true;
    }
    for due in [s.held.as_ref().map(|h| h.at), s.toast.as_ref().map(|t| t.until)].into_iter().flatten() {
        tx.wake_after(std::time::Duration::from_secs_f64((due - at).max(0.0)));
    }
    let mut stale = false;
    let mut arrived = None;
    {
        let mut m = s.mailbox.lock().unwrap();
        if let Some(snapshot) = m.snapshot.take() {
            s.snap = snapshot;
            stale = true;
        }
        if let Some(local) = m.local.take() {
            arrived = Some(local);
            stale = true;
        }
        if m.error != s.error && matches!(s.source, Source::Door(_)) {
            s.error = m.error.clone();
            changed = true;
        }
    }
    if let Some(local) = arrived {
        s.keep(local);
    }
    if !s.fixed_clock {
        // Ages are told to the minute, so the clock only has to turn that often.
        let now = clock();
        stale |= (now / 60_000.0).floor() != (s.now / 60_000.0).floor();
        s.now = now;
        tx.wake_after(std::time::Duration::from_secs(20));
    }
    if stale {
        s.refresh();
        // A list that lost rows keeps a row under the keys.
        s.picked = s.picked.min(s.rows_under_the_keys().saturating_sub(1));
    }
    for landed in s.outbox.landed() {
        s.landed(landed, tx);
        changed = true;
    }
    if let Some((key, answer)) = s.asking.take() {
        match answer {
            Ok(answer) => {
                let answer = reads::landed(s.threads.get(&key), answer);
                s.threads.insert(key, answer);
            }
            // Asked again at its next turn; until then what was read last is shown.
            Err(_) => s.reads.failed(),
        }
        changed = true;
    }
    s.ask_for_the_open_thread(stale);
    // While a run is up on the open conversation, it is read again every eight seconds at least.
    if matches!(&s.view, View::Thread { id, product } if s.snap.supervisor.running.iter().any(|r| &r.item_id == id && &r.product == product)) {
        tx.wake_after(std::time::Duration::from_millis(reads::AGAIN_MS as u64));
    }
    // The open conversation's pictures: what was read lands, another conversation's are let
    // go, and while any is on the page the disk is looked at again every two seconds.
    let open = match &s.view {
        View::Thread { id, product } => Some(format!("{product}/{id}")),
        _ => None,
    };
    let (landed, again) = s.pictures.tend(open.as_deref(), clock());
    if again {
        tx.wake_after(std::time::Duration::from_millis(pictures::AGAIN_MS as u64));
    }
    changed || stale || landed
}

fn resized(s: &mut State, [_, _, w, h]: [f32; 4]) {
    if (s.size.0 - w).abs() < 0.5 && (s.size.1 - h).abs() < 0.5 {
        return;
    }
    s.size = (w, h);
    let points = s.card_size();
    s.painting.request(points, paint_skies);
}

fn view(s: &State, cx: &mut Cx<State>) -> El {
    let (w, h) = s.size;
    // A picture's copy is made for this screen's pixels.
    s.pictures.set_scale(cx.env().scale);
    let page = match &s.view {
        View::Home => home::view(s),
        View::Inbox => inbox::view(s),
        View::Thread { id, product } => thread::view(s, id, product),
    };
    let root = el().w_full()
        .h_full()
        .fill(theme::page())
        // A press changes the page at once, as in the web window, whose pages, tabs, rows and
        // rail have no transition of their own. Rune would otherwise carry what two pages share
        // from its old place to its new one on a spring, fade the rest across and blend each
        // colour over a few frames: a table's headings slid sideways under rows dissolving into
        // others, and the highlight trailed the press. Both settings reach the whole window.
        .instant_changes()
        .motion(rune::Spring::NONE)
        .on_layout(resized)
        .child(chrome::backdrop(w, h))
        .child(chrome::card(s))
        .child(chrome::rail(s))
        .child(chrome::back_forward(s))
        // The page scrolls inside the card, under its top strip.
        .child(
            el().pin(Some(chrome::CARD[0]), Some(chrome::page_top(&s.view)), Some(chrome::CARD[2]), Some(chrome::CARD[3]))
                .clip_corners([0.0, 0.0, 6.0, 6.0])
                .scroll(ScrollSpec { stick_bottom: false })
                .child(page),
        );
    let root = match chrome::toast_card(s) {
        Some(toast) => root.child(toast),
        None => root,
    };
    // The Later picker stands over everything, the line about the last thing done included.
    let root = match later::view(s) {
        Some(picker) => root.child(picker),
        None => root,
    };
    // And the command bar, which never stands with it: opening one closes the other.
    let root = match palette::view(s) {
        Some(bar) => root.child(bar),
        None => root,
    };
    match chrome::notice(s) {
        Some(notice) => root.child(notice),
        None => root,
    }
}

static FONTS: &[&[u8]] = &[include_bytes!("../assets/fonts/Geist-Variable.ttf"), include_bytes!("../assets/fonts/GeistMono-Variable.ttf")];

/// `--watch-door <seconds>`: follow the engine with no window at all, and print one line each
/// time its conversations change. A check that the live path works, for a script or an agent.
fn watch_door(args: &[String]) -> bool {
    let Some(seconds) = args.iter().position(|a| a == "--watch-door").and_then(|i| args.get(i + 1)).and_then(|s| s.parse::<u64>().ok()) else {
        return false;
    };
    let Source::Door(door) = source::from_args(args) else {
        eprintln!("--watch-door needs a door: --door and --token, or --door-file");
        std::process::exit(2);
    };
    let mailbox = Mailbox::default();
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    source::follow_door(door.clone(), mailbox.clone(), move || {
        let _ = tx.send(());
    });
    let until = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
    // The newest thread is read the way an open conversation with a run up is: on every change
    // the engine pushes and every eight seconds, its logs asked for by the stamps already held.
    let (mut reads, mut newest) = (reads::Reads::default(), None);
    while let Some(left) = until.checked_duration_since(std::time::Instant::now()) {
        let pushed = rx.recv_timeout(left.min(std::time::Duration::from_millis(reads::AGAIN_MS as u64))).is_ok();
        let mut m = mailbox.lock().unwrap();
        if let Some(snapshot) = m.snapshot.take() {
            let lists = snapshot.lists(clock(), &Local::default());
            println!("changed: {} threads, {} need you, {} in progress", snapshot.items.len(), lists.needs.len(), lists.progress.len());
            newest = snapshot.items.last().map(|i| (i.product.clone(), i.id.clone(), i.updated_at.unwrap_or(0.0)));
        }
        if let Some(e) = m.error.take() {
            println!("could not read the engine: {e}");
        }
        drop(m);
        if let (Some((product, id, changed)), Some(at)) = (&newest, door.now())
            && let Some(ask) = reads.owed(at, product, id, *changed, Some(0.0), pushed, false, false, clock())
        {
            let (_, answer) = reads::read(ask);
            match answer {
                Ok(read) => {
                    let lines = read["zero:item-history"]["lines"].as_array().map_or(0, Vec::len);
                    let logs = read["zero:session-trace"]["sessions"].as_array().map_or(0, Vec::len);
                    println!("the newest thread's conversation: {lines} lines, {logs} logs");
                }
                Err(e) => {
                    reads.failed();
                    println!("the newest thread's conversation could not be read: {e}");
                }
            }
        }
    }
    true
}

fn main() {
    // `--rows <out.json>` writes the lists a fixture gives and opens no window.
    if let Some(written) = rules::rows_from_args(&std::env::args().collect::<Vec<_>>()) {
        return written.unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1)
        });
    }
    if watch_door(rune::tooling::extra_args()) {
        return;
    }
    // The web app's own lists: Geist Mono has no ⌘, which the system's monospace then draws.
    let spec = AppSpec::<State> {
        fonts: FONTS,
        families: ("Geist", "Geist Mono, ui-monospace"),
        setup: Some(setup),
        poll: Some(poll),
        on_key: Some(keys::pressed),
        min_size: Some((720.0, 520.0)),
        ..AppSpec::new("Mercury", DEFAULT_SIZE, init, view)
    };
    // The web app sets `-webkit-font-smoothing: antialiased`; `Off` is that rendering.
    App::new(spec).font_smoothing(FontSmoothing::Off).run();
}
