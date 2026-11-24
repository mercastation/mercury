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

mod answer;
mod attach;
mod change;
mod chips;
mod chrome;
mod closing;
mod cloud_need;
mod compose;
mod data;
mod drafts;
mod feel;
mod files;
mod film;
mod files_home;
mod files_page;
mod foot;
mod front;
mod head;
mod home;
mod icons;
mod inbox;
mod keys;
mod later;
mod logos;
mod made;
mod markdown;
mod need;
mod new_project;
mod palette;
mod pane;
mod picks;
mod pictures;
mod plan_bar;
mod reads;
mod reply;
mod rules;
mod settings;
mod ship;
mod sky;
mod source;
mod strip;
mod table;
mod talk;
mod theme;
mod thread;
mod trouble;
mod usage;
mod viewer;
mod wake;
mod walk;
mod window;
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
    /// The Files place: All files, or one project's page (`files_page.rs`).
    Files { project: Option<String> },
}

/// Both skies for one card size, painted off the main thread, so changing screen never waits.
struct Painted {
    home: (u32, u32, Vec<u8>),
    /// The Inbox's band alone, turned over (`inbox::band`).
    inbox: (u32, u32, Vec<u8>),
    /// Each as glass over it shows it, blurred here and kept small (`sky::soft`).
    home_soft: (u32, u32, Vec<u8>),
    inbox_soft: (u32, u32, Vec<u8>),
    points: (f32, f32),
}

pub struct State {
    /// The window, in points.
    pub size: (f32, f32),
    /// Home's sky, and the band of the Inbox's that shows behind its head, turned over
    /// (`inbox::band`).
    pub sky_home: Pixels,
    pub sky_inbox: Pixels,
    /// The same two as the glass over them shows them: blurred when they were painted, so no
    /// pane of glass has to ask Rune to blur what is behind it (`chrome::sky_through`).
    pub sky_home_soft: Pixels,
    pub sky_inbox_soft: Pixels,
    /// The card size the skies on screen were painted for.
    pub sky_size: (f32, f32),
    /// When Home's sky is let go, on the window's clock, once Home is no longer the page shown
    /// (`tend_the_home_sky`).
    home_sky_goes: Option<f64>,
    painting: Latest<Painted>,
    /// The window's repeating timers, each asked of Rune once (`wake.rs`).
    pub wakes: wake::Wakes,
    pub snap: Snapshot,
    /// What the screens draw, worked out from `snap` whenever it or the minute changes.
    pub lists: Lists,
    pub local: Local,
    /// The row of tasks that could not run, put away here and not yet in the web window's record.
    pub trouble: trouble::Kept,
    /// The bar that says agents cannot start until a plan is signed in (`plan_bar.rs`).
    pub plan: plan_bar::Bar,
    /// The rest of a fixture file, for the screen that knows what it is (a thread's words).
    pub fixture: Map<String, Value>,
    /// The clock the rows read, in milliseconds. A fixture brings its own.
    pub now: f64,
    fixed_clock: bool,
    /// The window's own clock when it last looked, in seconds: what a line's time left is
    /// drawn against.
    pub at: f64,
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
    /// The threads marked unread in this window: when, and when each had been read before,
    /// held the same way until the web window's record says unread too (`unsaw`).
    unread_here: HashMap<String, (f64, Option<f64>)>,
    /// The open conversation's reply box: open, or folded to its one line.
    pub replying: bool,
    /// What is written in it. Words written in one conversation wait in `drafts` while
    /// another is open.
    pub reply: TextEdit,
    drafts: HashMap<String, String>,
    /// The files in it, and in the boxes of the conversations left, the same way (`attach.rs`).
    pub reply_files: attach::ReplyFiles,
    /// Which of the open conversation's options the arrows are on, by its place in the list.
    pub option: Option<usize>,
    /// The model and level picked in each conversation's reply box, by `product/id`, kept beside
    /// its words until the next reply carries them (`reply.rs`), and that box's menu while open.
    pub reply_picks: HashMap<String, rules::reply_pick::ReplyPick>,
    pub reply_menu: Option<compose::Menu>,
    /// The open reply box's priority and mode, and the menu either has open (`chips.rs`).
    pub chips: chips::Chips,
    /// The menu or the panel open in the strip over a conversation (`strip.rs`).
    pub strip: strip::Strip,
    /// What the Inbox's strip has open: View and filters, the cloud's menu, ⌘C's question (`head.rs`).
    pub head: head::Head,
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
    /// Where each page was opened from, which is where Escape and a conversation's own way
    /// back lead (`leave_thread`).
    pub history: Vec<View>,
    /// The trail Back and Forward walk on the inbox page (`walk.rs`).
    walk: walk::Walk,
    pub source: Source,
    mailbox: Mailbox,
    /// Why the engine could not be read, when it could not.
    pub error: Option<String>,
    /// What this window is sending the engine, and what came back.
    outbox: write::Outbox,
    /// One line about something sent that the engine did not take, until the next thing is done.
    pub refused: Option<String>,
    /// What was done and waits three seconds to be written, and the pile of what Z still takes
    /// back once written, newest last (`undoStack`).
    held: Option<write::Held>,
    pub(crate) undos: Vec<write::Undo>,
    /// What answering a thread and stopping its agent keep (`answer.rs`).
    pub answering: answer::Answering,
    /// The threads opened with O from the line about them: a reply there keeps the person on
    /// them (`openedWithO`).
    pub(crate) came_back: HashSet<String>,
    /// Where the reply boxes' drafts are written down, across a restart (`drafts.rs`).
    pub(crate) kept_drafts: drafts::Store,
    /// What a test's window would have written, in order.
    #[cfg(test)]
    pub(crate) tried: Vec<writes::Write>,
    #[cfg(test)]
    pub(crate) pretend_sent: bool,
    /// What the last thing sent was filed under, for a test to hand it the engine's answer.
    #[cfg(test)]
    pub(crate) last_sent: Option<write::Sent>,
    /// The line about it.
    pub toast: Option<write::Toast>,
    /// The thread the last line about one named, and when that line went up on the window's
    /// clock: O still opens it for a while after the line has gone (`opensTheLastToast`).
    pub last_toast: Option<((String, String), f64)>,
    /// The threads closed here whose close the engine has not told back yet: out of the lists
    /// from the moment E is pressed.
    hidden: HashSet<String>,
    /// The Later picker L opens.
    pub later: later::Picker,
    /// The card of the sign-in the open thread's agent waits on, and what it holds (`need.rs`).
    pub sign_in: need::SignIn,
    /// The card of a ship that could not push for want of a GitHub sign-in here (`need.rs`).
    pub github: need::GitHub,
    /// The cards a cloud run's lines drew, the keys saved from here and the files held
    /// (`cloud_need.rs`).
    pub cloud_needs: cloud_need::Needs,
    /// What a close on the open thread is asking first (`closing.rs`).
    pub closing: closing::Closing,
    /// The command bar ⌘K opens (`palette.rs`).
    pub palette: palette::Bar,
    /// The box a thread is started in, and what it holds.
    pub compose: compose::Composer,
    /// The rail, folded or opened by the pointer resting on it (`chrome.rs`).
    pub rail: chrome::Rail,
    /// The card of the agents' limits its usage row opens (`usage.rs`).
    pub usage_card: usage::Card,
    /// The control the pointer is on, for a look under it Rune's own hover cannot draw (`feel.rs`).
    pub pointed: feel::Pointed,
    /// The models this Mac's coding agents offer, as the engine's settings say; the ones the
    /// app was built with until it has.
    pub models: rules::compose::Models,
    reading_models: Latest<Result<Value, String>>,
    /// What was last picked in that box, kept from one window to the next.
    pub picks: picks::Kept,
    /// Where the window stood on the desktop and how large, kept the same way (`window.rs`).
    pub window: window::Kept,
    /// The threads sent from here and taken back with Z, and when: a send taken back is closed
    /// on the ledger, and is not finished work (`withdrawnSend` in
    /// `renderer/src/withdraw-send.ts`).
    withdrawn: HashMap<String, f64>,
    /// A file opened big over the whole window (`viewer.rs`).
    pub viewer: viewer::Viewer,
    /// What agents did in their project's browser, for the open conversation (`film.rs`).
    pub films: film::Films,
    /// A file opened beside the conversation (`pane.rs`).
    pub pane: pane::Pane,
    /// The code the threads' runs changed: the strip's figures and a change open in the pane
    /// (`change.rs`).
    pub changes: change::Changes,
    /// A sheet open beside the conversation or big, worked as a spreadsheet (`table.rs`).
    pub tables: table::Sheets,
    /// The Files place's list and how it is narrowed (`files_page.rs`).
    pub files: files_page::Place,
    /// Settings, a screen over everything while it is open (`settings.rs`).
    pub settings: settings::Screen,
    /// What Try again's presses came back with, the card saying why the cloud has not got a
    /// thread while one is up, and the repeating tasks' rules (`foot.rs`).
    pub retries: foot::Retries,
    pub why_up: Option<foot::WhyUp>,
    pub repeats: foot::Repeats,
}

impl State {
    pub fn go(&mut self, to: View) {
        if self.view != to {
            self.close_head();
            self.put_the_reply_away();
            self.history.push(std::mem::replace(&mut self.view, to));
            self.take_the_reply_out();
            self.home_sky_for_home();
        }
    }
    pub fn back(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.close_head();
            self.put_the_reply_away();
            self.view = prev;
            self.take_the_reply_out();
            self.home_sky_for_home();
        }
    }
    /// The open conversation's row, as the engine last told it.
    pub fn open_item(&self) -> Option<&data::Item> {
        let View::Thread { id, product } = &self.view else {
            return None;
        };
        self.snap.items.iter().find(|i| &i.id == id && &i.product == product)
    }
    /// Leaving a conversation folds its reply box, drops the arrow off its options and closes
    /// its list of files and whatever its strip had open. Words written in the box, and the
    /// model picked in it, wait for the next visit.
    fn put_the_reply_away(&mut self) {
        self.leave_the_reading();
        self.leave_the_strip();
        self.leave_the_sign_in();
        self.leave_the_github_card();
        self.leave_the_cloud_cards();
        // A file open big belongs to the page it was opened over, and one beside a conversation
        // to the conversation, which keeps it for when it is opened again.
        self.viewer.open = None;
        self.leave_the_pane();
        if let View::Thread { id, product } = &self.view {
            let key = format!("{product}/{id}");
            self.reply_files.put_away(&key);
            // Words a caught send left in the box were no longer its draft, and go with it.
            if self.reply.text.trim().is_empty() || cloud_need::unkept(self, &key, &self.reply.text) {
                self.drafts.remove(&key);
            } else {
                self.drafts.insert(key, self.reply.text.clone());
            }
        }
        self.replying = false;
        self.reply = TextEdit::new("");
        self.option = None;
        self.refused = None;
        // The question before a discard belongs to the thread it was asked on, and so does the
        // one a close asks first.
        self.answering.discard = None;
        self.leave_the_close_ask();
    }
    fn take_the_reply_out(&mut self) {
        if let View::Thread { id, product } = &self.view {
            let key = format!("{product}/{id}");
            self.reply_files.take_out(&key);
            if let Some(words) = self.drafts.get(&key) {
                self.reply = TextEdit::new(words.clone());
            }
            // A draft is a conversation in progress: the thread opens with its box open on it,
            // the caret at its end (`hasDraft` in `renderer/src/App.tsx`).
            if !self.reply.text.trim().is_empty() || !self.reply_files.open.is_empty() {
                self.replying = true;
                self.reply.focused = true;
                self.answering.focus_reply = true;
            }
        }
        self.come_back_to_the_pane();
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
    /// Folds the reply box back to its one line, keeping what was written, and its model menu
    /// with it.
    pub fn fold_reply(&mut self) {
        // A folded box is drawn from its draft, which words a caught send left are not.
        if let Some(item) = self.open_item()
            && cloud_need::unkept(self, &format!("{}/{}", item.product, item.id), &self.reply.text)
        {
            self.reply = TextEdit::new("");
            self.cloud_needs.unkept = None;
        }
        self.reply_menu = None;
        self.replying = false;
        self.reply.focused = false;
    }
    /// One write to the engine. A fixture has nobody to send to, and says nothing of it.
    fn send(&mut self, channel: &'static str, payload: Value, sent: write::Sent) -> bool {
        self.send_all(vec![(channel, payload)], sent)
    }
    /// Several writes that are one thing done, sent in order.
    fn send_all(&mut self, writes: Vec<writes::Write>, sent: write::Sent) -> bool {
        self.send_steps(writes.into_iter().map(write::Step::Write).collect(), sent)
    }
    /// The same, where a write may be made of what the engine answered the one before it.
    pub(crate) fn send_steps(&mut self, steps: Vec<write::Step>, sent: write::Sent) -> bool {
        self.refused = None;
        // A test reads what would have gone, in order, from a window that has nobody to send to.
        #[cfg(test)]
        {
            self.tried.extend(steps.iter().filter_map(|step| match step {
                write::Step::Write(write) | write::Step::Linked(write) => Some(write.clone()),
                write::Step::Save { .. } => None,
            }));
            // And one that stands in for a door says it went, its answer handed in by the test.
            self.last_sent = Some(sent.clone());
            if self.pretend_sent {
                return true;
            }
        }
        let Source::Door(door) = &self.source else {
            return false;
        };
        if self.outbox.send_in_order(door, steps, sent) {
            return true;
        }
        self.refused = Some("Mercury is not running, so this was not sent.".to_string());
        false
    }
    /// One line about what was just done, for as long as the web window keeps its own up.
    fn say(&mut self, line: String, undo: bool, tx: &Tx) {
        let since = tx.clock();
        self.toast = Some(write::Toast { line, undo, since, until: since + writes::TOAST_MS / 1000.0, goes: None, sent: None, warns: false });
    }
    /// A line about one thread, as `(id, product)`: it stays up longer, names the thread, and
    /// O opens it. The promise of Z on the line's end becomes the key drawn beside it.
    fn say_about(&mut self, line: &str, goes: (String, String), tx: &Tx) {
        let (line, undo) = rules::compose::toast_parts(line);
        let since = tx.clock();
        self.last_toast = Some((goes.clone(), since));
        self.toast = Some(write::Toast { line, undo, since, until: since + rules::compose::TOAST_GOES_MS / 1000.0, goes: Some(goes), sent: None, warns: false });
    }
    /// A line that only tells: nothing was done, so there is nothing for Z.
    pub fn tell(&mut self, line: &str, tx: &Tx) {
        self.say(line.to_string(), false, tx);
    }
    /// Asks the desktop app for one of its own things, its Settings or the restart onto a new
    /// version (`front::ask_the_app`). A window with no app behind it says `refused` at once.
    pub fn ask_the_app(&mut self, channel: &'static str, refused: &'static str, tx: &Tx) {
        match &self.source {
            Source::Door(door) => front::ask_the_app(door, channel, refused),
            _ => self.tell(refused, tx),
        }
    }
    /// Does something now and writes it in three seconds, unless Z comes first
    /// (`deferCommit` in `renderer/src/App.tsx`). One thing waits at a time: whatever was
    /// waiting before is written at once.
    fn hold(&mut self, held: write::Held, tx: &Tx) {
        self.write_what_waits();
        // An answer names its thread, for O, and the line goes over the reply box of the thread
        // it moved the person to; one handed to a run already up says nothing, as its message
        // on the page says it (`answerWith`). Anything else is a plain line.
        match &held.sent {
            write::Sent::Reply { answer, .. } if answer.running => {}
            write::Sent::Reply { .. } | write::Sent::Pick { .. } => {
                let (line, undo) = rules::compose::toast_parts(&format!("{} · press Z to undo", held.said));
                let since = tx.clock();
                let goes = held.thread.clone();
                self.last_toast = Some((goes.clone(), since));
                let says = rules::answering::sent_line_says(&held.said).to_string();
                self.toast = Some(write::Toast { line, undo, since, until: since + rules::compose::TOAST_GOES_MS / 1000.0, goes: Some(goes), sent: Some(says), warns: false });
                // Out of Needs you now, In progress while it is held: the write can wait, the
                // lists cannot.
                self.answering.holding.insert(held.thread.0.clone(), false);
                self.refresh();
            }
            _ => self.say(held.said.clone(), true, tx),
        }
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
        if let Some(item) = held.closes.as_ref() {
            let title = writes::clip_to_sentence(item.title.as_deref().unwrap_or(""), 48);
            let thread = format!("{}/{}", held.thread.1, held.thread.0);
            let mut undo = write::Undo::plain(format!("Reopened: {title}"), format!("reopen “{title}”"), vec![(writes::ANSWER, writes::reopen_as_it_was(item))], write::Sent::TakenBack { thread }, held.thread.clone(), true);
            undo.mark = Some(write::Mark::of(rules::answering::UNDID_CLOSING, None));
            self.undos.push(undo);
        }
        // A reply to a run already up is handed over now, and Z no longer takes it back.
        if let write::Sent::Reply { answer, .. } = &held.sent {
            if answer.running {
                let answer = answer.clone();
                self.hand_over(&answer, &held.thread);
            }
        }
        // The stopped-tasks row leaves the lists as its put-away is written, not before.
        if let write::Sent::Trouble { closed, undo: false, .. } = &held.sent {
            trouble::written(self, *closed);
        }
        let mut writes = vec![(held.channel, held.payload)];
        writes.extend(held.then);
        self.send_all(writes, held.sent);
    }
    /// Z: takes back what is still waiting, whole; else the last thing this window wrote that
    /// can be taken back, a close or a thread sent.
    pub fn undo(&mut self, tx: &mut Tx) {
        if let Some(held) = self.held.take() {
            // What is taken back is no longer somewhere for O to go.
            self.last_toast = None;
            self.answering.asked = None;
            if let Some(item) = &held.closes {
                self.hidden.remove(&item.id);
            }
            if let write::Sent::Trouble { was, .. } = held.sent {
                self.trouble_taken_back(was);
            }
            let (id, product) = held.thread.clone();
            self.answering.holding.remove(&id);
            let thread = format!("{product}/{id}");
            if let write::Sent::Reply { answer, .. } = &held.sent {
                self.in_flight_landed(&thread, answer.in_flight);
            }
            let (back, drawn) = match &held.sent {
                write::Sent::Reply { words, pick, files, answer, .. } => (Some(write::GivenBack { words: words.clone(), pick: pick.clone(), files: files.clone() }), Some(answer.drawn)),
                write::Sent::Pick { answer, .. } => (None, Some(answer.drawn)),
                _ => (None, None),
            };
            // The copy drawn in the conversation goes with it.
            if let Some(drawn) = drawn {
                self.answering.let_go(drawn);
            }
            // Nothing reached the ledger, so this line is the only trace in the thread.
            if let Some(mark) = &held.mark {
                let now = self.moment();
                self.answering.leave_mark(&thread, mark, now, now);
            }
            self.refresh();
            // Back on the thread it was done from, when it was done from inside one; the words
            // go back in its box unless the box holds something newer (`wordsBackInTheBox`).
            let restored = back.map(|back| self.give_back(&thread, back)).unwrap_or(false);
            if held.from_inside || restored {
                self.step_to_thread(&id, &product);
            }
            if restored {
                self.open_reply(tx);
            }
            self.say(writes::said_undone(&held.said, restored), false, tx);
            return;
        }
        self.undo_from_the_pile(tx);
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
        // It stands over the page alone.
        self.close_thread_menus();
        self.later = later::Picker::open_on(Some((id.to_string(), product.to_string())), "");
        // The first L ever on a thread whose agent waits on the person says what it costs,
        // inside the picker (`openSnooze`).
        if let Some(item) = self.item(id, product).cloned() {
            self.warns_first(&item, false, tx);
        }
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
        // A stop is said once the engine has it; a reply or a pick written goes on Z's pile, its
        // drawn copy standing a while longer (`answer.rs`).
        match &landed.sent {
            write::Sent::Stop { running, .. } => {
                self.say(rules::answering::stopped_line(*running).to_string(), false, tx);
                self.refresh();
                return;
            }
            write::Sent::SendNow { .. } => return,
            write::Sent::Reply { answer, words, pick, files, .. } => {
                self.drawn_landed(answer.drawn);
                self.in_flight_landed(landed.sent.thread(), answer.in_flight);
                if let Some(held) = self.answering.holding.get_mut(&id) {
                    *held = true;
                }
                if !answer.running {
                    let mut undo = write::Undo::plain(answer.undo_said.clone(), answer.undoes.clone(), answer.undo.clone(), write::Sent::TakenBack { thread: landed.sent.thread().to_string() }, (id.clone(), product.clone()), false);
                    undo.mark = Some(answer.mark.clone());
                    undo.back = Some(write::GivenBack { words: words.clone(), pick: pick.clone(), files: files.clone() });
                    self.undos.push(undo);
                }
                self.refresh();
                return;
            }
            write::Sent::Pick { answer, .. } => {
                self.drawn_landed(answer.drawn);
                if let Some(held) = self.answering.holding.get_mut(&id) {
                    *held = true;
                }
                let mut undo = write::Undo::plain(answer.undo_said.clone(), answer.undoes.clone(), answer.undo.clone(), write::Sent::TakenBack { thread: landed.sent.thread().to_string() }, (id.clone(), product.clone()), true);
                undo.mark = Some(answer.mark.clone());
                self.undos.push(undo);
                self.refresh();
                return;
            }
            _ => {}
        }
        // A level picked in the strip's menu is said once it stands (`retag`); a place picked in
        // its panel says nothing the panel does not show (`strip.rs`).
        match &landed.sent {
            write::Sent::Priority { said, .. } => {
                let said = said.clone();
                self.say(said, false, tx);
                self.refresh();
                return;
            }
            write::Sent::Place { moving, .. } => {
                let answer = landed.answer.as_ref().cloned().unwrap_or(Value::Null);
                self.place_landed(*moving, &answer, tx);
                return;
            }
            _ => {}
        }
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
                let mut undo = write::Undo::plain(writes::BACK_IN_YOUR_INBOX.to_string(), "undo moving it to Later".to_string(), vec![undo], write::Sent::TakenBack { thread }, (id.clone(), product.clone()), true);
                undo.mark = Some(write::Mark::of("Undid moving it to Later", None));
                (said, undo)
            }
            write::Sent::Back { thread, said, undo } => {
                let started = said == "Started";
                let mut undo = write::Undo::plain(writes::RETURNED_TO_LATER.to_string(), "put it back in Later".to_string(), undo, write::Sent::TakenBack { thread }, (id.clone(), product.clone()), false);
                undo.mark = Some(write::Mark::of(if started { "Undid starting it" } else { "Undid bringing it back" }, None));
                (said.to_string(), undo)
            }
            _ => return,
        };
        self.undos.push(undo);
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
    /// empties and folds at once; Z, or a refusal, puts the words back. A model picked in the
    /// box goes with them, written before the words (`answerItem` in `main/store.mts`), and the
    /// box lets go of it as it lets go of the words (`clearDraft`). Files in the box are saved
    /// into the project first, and the reply waits for them (`carry` in `attach.rs`).
    pub fn send_reply(&mut self, tx: &mut Tx) {
        let words = self.reply.text.trim().to_string();
        let Some(item) = self.open_item().cloned() else {
            return;
        };
        if !self.reply_files.open.is_empty() {
            return self.carry(&item, words, tx);
        }
        if words.is_empty() {
            return;
        }
        // A key or a code typed for what a cloud run asked is saved or sent as that, never as
        // words (`sendReply` in `Focus.tsx`).
        if self.catch_the_reply(&item, &words, tx) {
            return;
        }
        let pick = self.reply_pick_to_send(&item);
        let said = chips::said(self, &item);
        self.hold_reply(&item, words, "", pick, said, Vec::new(), tx);
    }
    /// The reply to `item` held its three seconds and then written: the words, and the links to
    /// the files it carries under them after a blank line (`[words, attachedMd].join('\n\n')`),
    /// with the priority picked on its chip and what it may do (`said`, as they stood at Send).
    /// That thread's box lets go of its words, files and pick, open or not, since the person
    /// may have moved on while the files were saved.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn hold_reply(&mut self, item: &data::Item, words: String, links: &str, pick: Option<rules::reply_pick::ReplyPick>, said: rules::reply_chips::Said, files: Vec<attach::Attached>, tx: &mut Tx) {
        let thread = format!("{}/{}", item.product, item.id);
        let message = [words.as_str(), links].into_iter().filter(|part| !part.is_empty()).collect::<Vec<_>>().join("\n\n");
        // A reply to a run already up goes to that run: it opens nothing and lifts nothing, and
        // once handed over nothing takes it back (`isRunning` in `answerWith`).
        let running = self.run_on(item);
        let mut payload = if running { serde_json::json!({ "product": item.product, "id": item.id, "answer": message }) } else { writes::reply(item, &message) };
        if let Some(fields) = payload.as_object_mut() {
            fields.extend(said.fields());
            if let Some(pick) = &pick {
                fields.extend(rules::reply_pick::answer_fields(pick));
            }
        }
        let here = self.open_item().is_some_and(|open| open.id == item.id && open.product == item.product);
        self.drafts.remove(&thread);
        self.reply_picks.remove(&thread);
        self.reply_files.clear(&thread, here);
        if here {
            self.reply_menu = None;
            self.reply = TextEdit::new("");
            self.replying = false;
        }
        // On the page from the press on, as the person's own message, saying it is on its way;
        // and written down until it lands, so a restart cannot lose it (`markReplyInFlight`).
        let drawn = self.answering.draw(&thread, self.moment(), &message);
        let in_flight = self.mark_in_flight(&thread, &words, &message, &files);
        let around = if running { rules::answering::Around { after: Vec::new(), undo_first: Vec::new() } } else { rules::answering::around_a_reply(item, self.moment()) };
        let mut undo = around.undo_first;
        undo.extend(rules::answering::withdraw(item, true));
        // Where the person goes: on to the next thread waiting while Needs you is being cleared,
        // else they stay, following the thread (`stay`, `replyMovesOn`).
        let moves_on = here && !running && self.reply_moves_on(item);
        let next = moves_on.then(|| self.next_after(item)).flatten();
        if here && !moves_on {
            self.answering.following = Some((item.id.clone(), item.product.clone()));
        }
        let answer = write::Answer {
            drawn,
            running,
            undo,
            undo_said: rules::answering::REPLY_WITHDRAWN.to_string(),
            undoes: rules::answering::REPLY_UNDOES.to_string(),
            mark: write::Mark::of(rules::answering::UNDID_REPLY, None),
            moved_on: moves_on,
            in_flight,
        };
        self.hold(
            write::Held {
                channel: writes::ANSWER,
                payload,
                sent: write::Sent::Reply { thread, words, pick, files, answer },
                said: writes::said_sent(&self.project_name(item)),
                at: tx.clock() + writes::UNDO_GRACE_MS / 1000.0,
                thread: (item.id.clone(), item.product.clone()),
                from_inside: true,
                closes: None,
                then: around.after,
                // Taken back inside the three seconds, a reply to a run leaves no line in the
                // thread; one to a thread at rest does (`undid` in `deferCommit`).
                mark: (!running).then(|| write::Mark::of(rules::answering::UNDID_REPLY, None)),
            },
            tx,
        );
        if moves_on {
            let now = tx.clock();
            self.move_on_to(next, now);
        }
    }
    /// Picks one of the options the open thread offers, by its place in the list: most are sent
    /// as a reply, and a few close the thread or open the box instead (`rules/writes.rs`).
    pub fn pick_option(&mut self, at: usize, tx: &mut Tx) {
        let Some(item) = self.open_item().cloned() else {
            return;
        };
        let Some(choice) = ship::offer_now(self, &item).into_iter().nth(at) else {
            return;
        };
        self.option = None;
        match writes::pick(&choice) {
            writes::Pick::Reply(_) => self.send_pick(&item, &choice, tx),
            // The agent's own "Close this task", picked, is its answer: no warning stands before it.
            writes::Pick::Close => self.close_now(&item.id, &item.product, false, tx),
            writes::Pick::CloseUnshipped => self.close_now(&item.id, &item.product, true, tx),
            // Throwing the work away asks first, where the options stood (`discardAsk`).
            writes::Pick::AsksFirst => self.ask_before_discarding(&item, choice),
            writes::Pick::NeedsWords(words) => {
                if self.reply.text.trim().is_empty() {
                    self.reply = TextEdit::new(words);
                }
                self.open_reply(tx);
            }
        }
    }
    /// A picked option sent as a reply: "Option 2: …", drawn at once as the line its written
    /// copy will be, held three seconds, and once written on Z's pile. It moves the person on
    /// unless its words promise more or a run is up (`pickOption`, `pickStaysPut`).
    pub(crate) fn send_pick(&mut self, item: &data::Item, choice: &rules::format::Choice, tx: &mut Tx) {
        let thread = format!("{}/{}", item.product, item.id);
        let words = writes::option_words(choice);
        let drawn = self.answering.draw(&thread, self.moment(), &words);
        let live = self.run_on(item) || self.snap.supervisor.rest.get("starting").and_then(Value::as_array).is_some_and(|s| s.iter().any(|id| id.as_str() == Some(&item.id)));
        let here = self.open_item().is_some_and(|open| open.id == item.id && open.product == item.product);
        let moves_on = here && self.reply_moves_on(item) && !rules::answering::pick_stays_put(&choice.text, live);
        let next = moves_on.then(|| self.next_after(item)).flatten();
        let option = rules::format::without_recommended(&choice.text);
        let mark = write::Mark::of(rules::answering::UNDID_PICKING, Some(option));
        let answer = write::Answer {
            drawn,
            running: false,
            undo: rules::answering::withdraw(item, false),
            undo_said: rules::answering::pick_withdrawn(choice.n),
            undoes: rules::answering::pick_undoes(choice.n),
            mark: mark.clone(),
            moved_on: moves_on,
            in_flight: 0.0,
        };
        self.hold(
            write::Held {
                channel: writes::ANSWER,
                payload: writes::reply(item, &words),
                sent: write::Sent::Pick { thread, answer },
                said: rules::answering::pick_sent_words(&choice.text, &rules::title::row_title(item)),
                at: tx.clock() + writes::UNDO_GRACE_MS / 1000.0,
                thread: (item.id.clone(), item.product.clone()),
                from_inside: true,
                closes: None,
                then: Vec::new(),
                mark: Some(mark),
            },
            tx,
        );
        if moves_on {
            let now = tx.clock();
            self.move_on_to(next, now);
        }
    }
    /// Marks a thread done (E, or the button). It leaves the lists at once and is written in
    /// three seconds; from inside it, the person goes on to the next thread that needs them,
    /// as J would take them. The engine refuses a close that would strand work off main unless
    /// `unshipped` says it is meant, and its refusal is said in its own words. The first close
    /// ever of a thread whose agent waits on the person is held, and a line says what it costs
    /// (`nudgeOnStopped`).
    pub fn close(&mut self, id: &str, product: &str, unshipped: bool, tx: &mut Tx) {
        // The row the app makes itself is put away by its own route (`trouble.rs`).
        if id == rules::trouble_row::TROUBLE_ID && product.is_empty() {
            return self.put_the_trouble_away(tx);
        }
        let Some(item) = self.snap.items.iter().find(|i| i.id == id && i.product == product).cloned() else {
            return;
        };
        if self.warns_first(&item, true, tx) {
            return;
        }
        self.close_now(id, product, unshipped, tx);
    }
    /// The close itself, with no warning before it, unless it asks first: work that never
    /// reached main, or filed tasks still waiting for a yes (`closing.rs`).
    pub(crate) fn close_now(&mut self, id: &str, product: &str, unshipped: bool, tx: &mut Tx) {
        let Some(item) = self.snap.items.iter().find(|i| i.id == id && i.product == product).cloned() else {
            return;
        };
        // Already closed by the person: there is nothing left to mark. A thread its agent
        // finished still needs them, and E is how they say they have read it.
        if item.status == "done" && item.wrote("status").is_some_and(|w| w.source == "founder") {
            return;
        }
        if self.close_asks_first(&item, unshipped, false) {
            return;
        }
        self.close_unasked(id, product, unshipped, tx);
    }
    /// The close, held three seconds, asking nothing.
    pub(crate) fn close_unasked(&mut self, id: &str, product: &str, unshipped: bool, tx: &mut Tx) {
        let Some(item) = self.snap.items.iter().find(|i| i.id == id && i.product == product).cloned() else {
            return;
        };
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
                then: Vec::new(),
                mark: Some(write::Mark::of(rules::answering::UNDID_CLOSING, None)),
            },
            tx,
        );
        if inside {
            let now = tx.clock();
            self.move_on_to(next, now);
        }
    }
    /// The open thread, closed.
    pub fn close_thread(&mut self, unshipped: bool, tx: &mut Tx) {
        if let View::Thread { id, product } = self.view.clone() {
            self.close(&id, &product, unshipped, tx);
        }
    }
    /// Opens a finished thread again (the Reopen button), as the command bar's Reopen does: the
    /// thread open again with nothing said, written at once, and Z marks it done again
    /// (`onReopenDone` in `renderer/src/components/Focus.tsx`, `reopenDone` in `App.tsx`). It is
    /// not `zero:reopen`, which sends a blocked thread back to its agent.
    pub fn reopen_thread(&mut self) {
        let Some(item) = self.open_item().cloned() else {
            return;
        };
        self.reopen_done(&item);
    }
    /// What the engine said to something sent. A refusal is said in one line, and refused
    /// words go back where the person can send them again.
    fn landed(&mut self, landed: write::Landed, tx: &mut Tx) {
        // A new thread, and one taken back, are said of whether they stood or not; a write of
        // the cloud's is about no thread (`head.rs`).
        match &landed.sent {
            write::Sent::Thread { .. } => return self.thread_landed(landed, tx),
            write::Sent::Withdraw { .. } => return self.withdraw_landed(landed, tx),
            write::Sent::Cloud | write::Sent::Moved { .. } | write::Sent::SignIn => return self.cloud_landed(landed, tx),
            write::Sent::Carry { .. } => return self.carried(landed, tx),
            write::Sent::Palette(_) => return palette::landed(self, landed, tx),
            // A sign-in card's writes are the card's own to say (`need.rs`).
            write::Sent::SignInSaid { .. } | write::Sent::SignInKeep { .. } => return self.sign_in_landed(landed),
            write::Sent::GitHubBegin { .. } | write::Sent::GitHubToken { .. } => return self.github_landed(landed, tx),
            write::Sent::CloudNeed(_) => return self.cloud_need_landed(landed, tx),
            // So are the presses on the list of the threads a thread filed (`made.rs`).
            write::Sent::Filed(_) => return made::landed(self, landed, tx),
            // The row of tasks that could not run, and a project made (`trouble.rs`, `new_project.rs`).
            write::Sent::Trouble { .. } => return trouble::landed(self, landed, tx),
            write::Sent::Project { .. } => return new_project::landed(self, landed, tx),
            // The Files place's changes, New, the Trash, and Z's writes of them (`files_page.rs`).
            write::Sent::Files(_) => return files_page::landed(self, landed, tx),
            // Z's own writes: said once the engine has them, or put back on the pile.
            write::Sent::TakenBack { .. } => return self.undo_landed(landed, tx),
            _ => {}
        }
        let Some(why) = landed.refusal() else {
            self.took(landed, tx);
            return;
        };
        // A close the engine did not make puts the row back in its list, and Z no longer has it.
        if let write::Sent::Close { thread } = &landed.sent {
            if let Some((product, id)) = thread.split_once('/') {
                self.hidden.remove(id);
                let (id, product) = (id.to_string(), product.to_string());
                self.undos.retain(|u| !(u.thread == (id.clone(), product.clone()) && u.mark.as_ref().is_some_and(|m| m.words == rules::answering::UNDID_CLOSING)));
                self.refresh();
            }
        }
        // A reply that did not get through: its drawn copy goes, the words and files come back
        // after whatever the box holds now, and the box opens where the person still is, with
        // the engine's words; elsewhere a line names the thread (`putBack`, `restoreFailedDraft`).
        // The model it carried does not come back.
        if let Some((_, id)) = landed.sent.thread().split_once('/') {
            self.answering.holding.remove(id);
        }
        if let write::Sent::Reply { thread, words, files, answer, .. } = &landed.sent {
            self.answering.let_go(answer.drawn);
            self.in_flight_landed(thread, answer.in_flight);
            if answer.running {
                // Its hand-over never happened, so Z goes back to whatever came before it.
                if let Some(at) = self.undos.iter().rposition(|u| u.writes.is_empty() && format!("{}/{}", u.thread.1, u.thread.0) == *thread) {
                    self.undos.remove(at);
                }
            }
            let here = matches!(&self.view, View::Thread { id, product } if &format!("{product}/{id}") == thread);
            self.put_back_failed(thread, words, files.clone());
            self.refresh();
            if here {
                self.replying = true;
                self.reply.focused = true;
                tx.focus_when_shown("reply", false);
                let said = if why.trim().is_empty() { rules::answering::COULD_NOT_SEND.to_string() } else { why };
                self.say(said, false, tx);
            } else if let Some((product, id)) = thread.split_once('/') {
                self.say_about(rules::answering::DID_NOT_REACH, (id.to_string(), product.to_string()), tx);
            }
            return;
        }
        if let write::Sent::Pick { answer, .. } = &landed.sent {
            self.answering.let_go(answer.drawn);
            self.refresh();
        }
        self.refused = Some(match landed.sent {
            write::Sent::Reply { .. } => return,
            write::Sent::Pick { .. } => format!("Could not send that option: {why}"),
            write::Sent::Stop { .. } => format!("Could not stop it: {why}"),
            write::Sent::SendNow { .. } => {
                self.answering.cutting.remove(landed.sent.thread());
                return;
            }
            write::Sent::Close { .. } => format!("Could not mark it done: {why}"),
            write::Sent::Reopen { .. } => format!("Could not reopen it: {why}"),
            write::Sent::Later { .. } => format!("Could not move it to Later: {why}"),
            write::Sent::Back { .. } => format!("Could not bring it back: {why}"),
            write::Sent::TakenBack { .. } => format!("Could not undo that: {why}"),
            write::Sent::Priority { .. } => format!("Could not change its priority: {why}"),
            write::Sent::Place { .. } => format!("Could not change where it runs: {why}"),
            // These went their own way above.
            write::Sent::Thread { .. }
            | write::Sent::Withdraw { .. }
            | write::Sent::Carry { .. }
            | write::Sent::Cloud
            | write::Sent::Moved { .. }
            | write::Sent::SignIn
            | write::Sent::Palette(_)
            | write::Sent::SignInSaid { .. }
            | write::Sent::SignInKeep { .. }
            | write::Sent::GitHubBegin { .. }
            | write::Sent::GitHubToken { .. }
            | write::Sent::Filed(_)
            | write::Sent::Trouble { .. }
            | write::Sent::Project { .. }
            | write::Sent::CloudNeed(_)
            | write::Sent::Files(_) => return,
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
        // Another thread is no longer the one followed (`stillFollowing`).
        if self.answering.following.as_ref().is_some_and(|(i, p)| i != id || p != product) {
            self.answering.following = None;
        }
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
        self.unread_here.remove(id);
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
    /// Counts a thread as unread again (`markUnseen` in `App.tsx`): out of the seen, and its
    /// moment of reading set to none. The web window is asked to do the same by whoever calls
    /// this (`zero:mark-unread-here`), and until its record says so this window holds the mark.
    pub(crate) fn unsaw(&mut self, id: &str) {
        self.opened.remove(id);
        self.unread_here.insert(id.to_string(), (clock(), self.local.seen_at.get(id).copied()));
        self.local.seen_at.insert(id.to_string(), 0.0);
        self.local.seen.retain(|seen| seen != id);
        self.refresh();
    }
    /// The web window did not take a mark of unread: the thread is read as it was.
    pub(crate) fn unsaw_refused(&mut self, id: &str) {
        let Some((_, was)) = self.unread_here.remove(id) else { return };
        match was {
            Some(at) => {
                self.local.seen_at.insert(id.to_string(), at);
                if !self.local.seen.iter().any(|seen| seen == id) {
                    self.local.seen.push(id.to_string());
                }
            }
            None => {
                self.local.seen_at.remove(id);
            }
        }
        self.refresh();
    }
    /// What the web window keeps, with the threads opened here since laid over it, and the ones
    /// marked unread here. Once its own record of a thread is as new as this window's, this
    /// window's is let go.
    pub(crate) fn keep(&mut self, mut local: Local) {
        self.opened.retain(|id, at| local.seen_at.get(id).is_none_or(|theirs| theirs < at));
        for (id, at) in &self.opened {
            local.seen_at.insert(id.clone(), *at);
            if !local.seen.iter().any(|seen| seen == id) {
                local.seen.push(id.clone());
            }
        }
        // A mark of unread stands while the web window's record is an older reading: it is let
        // go once that says unread too, or says the thread was opened there since.
        self.unread_here.retain(|id, (at, _)| local.seen_at.get(id).is_none_or(|theirs| *theirs != 0.0 && theirs < at));
        for id in self.unread_here.keys() {
            local.seen_at.insert(id.clone(), 0.0);
            local.seen.retain(|seen| seen != id);
        }
        // And the row of tasks that could not run, put away or back here (`trouble.rs`).
        trouble::lay_over(&mut self.trouble, &mut local);
        self.local = local;
    }
    /// The rows J and K walk and Enter opens: the open tab's on the Inbox page.
    fn rows_under_the_keys(&self) -> usize {
        self.lists.under(self.inbox_tab).len()
    }
    /// The card's box, beside the rail as wide as it is now.
    pub fn card_size(&self) -> (f32, f32) {
        ((self.size.0 - chrome::card_left(self) - chrome::CARD[2]).max(1.0), (self.size.1 - chrome::CARD[1] - chrome::CARD[3]).max(1.0))
    }
    /// The page's box inside the window: under the card's top strip.
    pub fn page_size(&self) -> (f32, f32) {
        ((self.size.0 - chrome::card_left(self) - chrome::CARD[2]).max(1.0), (self.size.1 - chrome::page_top(&self.view) - plan_bar::height(self) - chrome::CARD[3]).max(1.0))
    }
    fn refresh(&mut self) {
        // An answer held for Z, or written and not yet read back, has already gone as far as the
        // lists are concerned: out of Needs you, and In progress as a run being started
        // (`holdRow`, `startingIds`).
        let holding: Vec<String> = self.answering.holding.keys().cloned().collect();
        self.lists = if holding.is_empty() {
            rules::lists_sorted(&self.snap, self.now, &self.local, self.picks.get().display.sort)
        } else {
            let mut snap = self.snap.clone();
            for field in ["starting", "queued"] {
                let list = snap.supervisor.rest.entry(field.to_string()).or_insert_with(|| Value::Array(Vec::new()));
                if let Value::Array(ids) = list {
                    ids.extend(holding.iter().map(|id| Value::String(id.clone())));
                }
            }
            let mut lists = rules::lists_sorted(&snap, self.now, &self.local, self.picks.get().display.sort);
            lists.needs.retain(|r| !holding.contains(&r.id));
            lists
        };
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
        // A thread taken back with Z is closed on the ledger and is not in Done, for as long as
        // that close is the last thing written to its status.
        if !self.withdrawn.is_empty() {
            let (snap, withdrawn) = (&self.snap, &self.withdrawn);
            let taken_back = |id: &str| withdrawn.get(id).is_some_and(|at| snap.item(id).is_some_and(|i| i.status == "done" && i.wrote_at("status") <= *at));
            self.lists.done.retain(|r| !taken_back(&r.id));
        }
        // Then the View and filters menu's filters, over what is left (`head.rs`).
        rules::display::show(&mut self.lists, &self.picks.get().display, self.now);
        self.tend_head();
    }
    /// Asks the engine which models its coding agents offer, off the main thread: the answer
    /// lands in `poll`. The web window reads them with its settings, at launch and again when
    /// the engine says a list changed; this one asks at launch and each time the model menu
    /// opens.
    fn read_the_models(&mut self) {
        let Source::Door(door) = &self.source else {
            return;
        };
        if let Some((url, token)) = door.now() {
            self.reading_models.request((url, token), |(url, token)| source::call(&url, &token, "zero:settings-read", &serde_json::json!({ "search": false })));
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
        // Home's sky is hung only where it is kept: on Home, or within its ten seconds after.
        if self.home_sky_is_kept() {
            self.sky_home.set(p.home.0, p.home.1, p.home.2);
            self.sky_home_soft.set(p.home_soft.0, p.home_soft.1, p.home_soft.2);
        }
        self.sky_inbox.set(p.inbox.0, p.inbox.1, p.inbox.2);
        self.sky_inbox_soft.set(p.inbox_soft.0, p.inbox_soft.1, p.inbox_soft.2);
        self.sky_size = p.points;
    }
    /// Whether Home's sky is held now: Home is the page shown, or was not long ago.
    fn home_sky_is_kept(&self) -> bool {
        self.view == View::Home || self.sky_home.size().0 > 0
    }
    /// Home's sky, painted before Home is drawn when it was let go: the page a person comes to
    /// has its sky in its first frame, as it always had.
    fn home_sky_for_home(&mut self) {
        if self.view != View::Home {
            return;
        }
        self.home_sky_goes = None;
        if self.sky_home.size().0 == 0 {
            let (home, soft) = paint_home_sky(chrome::folded_card_size(self));
            self.sky_home.set(home.0, home.1, home.2);
            self.sky_home_soft.set(soft.0, soft.1, soft.2);
        }
    }
    /// Lets Home's sky go once Home has not been the page for `HOME_SKY_KEPT` seconds, and
    /// paints it for a Home that has none, however Home came to be the page. True when what is
    /// drawn changed.
    fn tend_the_home_sky(&mut self, tx: &mut Tx) -> bool {
        if self.view == View::Home {
            let missing = self.sky_home.size().0 == 0;
            self.home_sky_for_home();
            return missing;
        }
        if self.sky_home.size().0 == 0 {
            self.home_sky_goes = None;
            return false;
        }
        let now = tx.clock();
        match self.home_sky_goes {
            // Asked for once, as it is set: Rune gives every wake it is asked for a turn.
            None => {
                self.home_sky_goes = Some(now + HOME_SKY_KEPT);
                tx.wake_at(now + HOME_SKY_KEPT);
            }
            // A moment due within a thousandth of a second has come (`wake.rs`).
            Some(goes) if now >= goes - 0.001 => {
                self.sky_home.set(0, 0, Vec::new());
                self.sky_home_soft.set(0, 0, Vec::new());
                self.home_sky_goes = None;
            }
            Some(_) => {}
        }
        // Nothing on a page that is not Home draws Home's sky.
        false
    }
}

fn clock() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0.0, |d| d.as_millis() as f64)
}

/// Home's sky for a card of `points`, and the same as the glass over it shows it.
fn paint_home_sky(points: (f32, f32)) -> ((u32, u32, Vec<u8>), (u32, u32, Vec<u8>)) {
    let (w, h) = (points.0.floor() as f64, points.1.floor() as f64);
    // Dots from 40 points down, fully in 300 points later (`SkyBackdrop` in App.tsx).
    let home = sky::draw(&sky::ember(), w, h, 40.0, 300.0, 0.0, sky::PER_POINT);
    // What Home's glass has behind it, blurred as the glass blurs it.
    let soft = sky::saturated(sky::soft(&home, theme::GLASS_BLUR * sky::PER_POINT as f32, chrome::SOFT_HOME), theme::GLASS_SATURATES);
    (home, soft)
}

fn paint_skies(points: (f32, f32)) -> Painted {
    let (w, h) = (points.0.floor() as f64, points.1.floor() as f64);
    let (home, home_soft) = paint_home_sky(points);
    // The Inbox's: a band 190 points tall at the card's foot (`InboxSky`, shape "head"), of
    // which only the rows behind the head are kept.
    let start = (h - 190.0).round();
    let inbox = inbox::band(sky::draw(&sky::ember(), w, h, start, 120.0, start - 60.0, sky::PER_POINT));
    // What the strip's buttons have behind them, blurred as each blurs it.
    let inbox_soft = sky::soft(&inbox::band_under_the_strip(&inbox), inbox::STRIP_BLUR * sky::PER_POINT as f32, chrome::SOFT_STRIP);
    Painted { home, inbox, home_soft, inbox_soft, points }
}

/// How long Home's sky is kept after the person leaves Home, in case they come straight back.
/// Past it the sky is let go, and painted again when Home is next shown: 8 ms for a window of
/// 1920 by 1050 points, 20 with its glass's copy, where keeping it held 32 MB all day in a
/// window that showed a conversation.
const HOME_SKY_KEPT: f64 = 10.0;

const DEFAULT_SIZE: (f32, f32) = (1280.0, 760.0);

/// The store of pictures for this run. A still picture reads each file where it is asked
/// for, since it cannot wait; a window, or a run of frames, reads them off the main thread.
fn pictures_for(never: bool) -> pictures::Pictures {
    let run = rune::tooling::headless_run();
    pictures::Pictures::new(run.map_or(2.0, |r| r.scale()), run.is_some_and(|r| r.frames() == 0), never)
}

fn init() -> State {
    init_from(rune::tooling::extra_args())
}

/// Where this window's frame on the desktop is kept: beside the note about the app's door, or
/// where the command line says (`window::file`).
fn window_file(args: &[String], source: &Source) -> Option<std::path::PathBuf> {
    window::file(args, match source {
        Source::Door(door) => door.note(),
        _ => None,
    })
}

/// The state a window opens on, from its command line; a test opens one on a fixture this way.
fn init_from(args: &[String]) -> State {
    let source = source::from_args(args);
    // What was last picked in the box a thread is started in, from beside the door's note. A
    // fixture's picks are its own, read below, and no file is touched for one.
    let picks = match &source {
        Source::Door(door) => picks::Kept::at(picks::file(args, door.note())),
        _ => picks::Kept::default(),
    };
    // And what the reply boxes held, the same way.
    let (kept_drafts, drafts_kept, draft_files) = drafts::Store::at(match &source {
        Source::Door(door) => drafts::file(args, door.note()),
        _ => drafts::file(args, None),
    });
    // And where the window stood on the desktop.
    let stood = window::Kept::at(window_file(args, &source));
    // A headless picture has its size before the first frame; a window reports its own.
    let size = rune::tooling::headless_run().map_or(DEFAULT_SIZE, |run| run.size());
    let mut s = State {
        size,
        sky_home: Pixels::new(),
        sky_inbox: Pixels::new(),
        sky_home_soft: Pixels::new(),
        sky_inbox_soft: Pixels::new(),
        sky_size: (0.0, 0.0),
        home_sky_goes: None,
        // A sky is painted again once the card has held still (`STILL`), never on each step of
        // a window being dragged larger: every step painted two full skies off the main thread
        // and hung them, which took a window from 0.4 GB to 1.6 GB while it was resized.
        painting: Latest::default().debounce(chrome::STILL),
        wakes: wake::Wakes::default(),
        snap: Snapshot::default(),
        lists: Lists::default(),
        local: Local::default(),
        trouble: trouble::Kept::default(),
        plan: plan_bar::Bar::default(),
        fixture: Map::new(),
        now: clock(),
        fixed_clock: false,
        at: 0.0,
        view: match args.iter().position(|a| a == "--view").and_then(|i| args.get(i + 1)).map(String::as_str) {
            Some("inbox") => View::Inbox,
            Some("files") => View::Files { project: None },
            _ => View::Home,
        },
        // `--tab later` opens the Inbox on that tab, for a picture of it.
        inbox_tab: Tab::from_args(args),
        picked: 0,
        home_pick: None,
        opened: HashMap::new(),
        unread_here: HashMap::new(),
        replying: false,
        reply: TextEdit::new(""),
        drafts: HashMap::new(),
        reply_files: attach::ReplyFiles::default(),
        option: None,
        reply_picks: HashMap::new(),
        reply_menu: None,
        chips: chips::Chips::default(),
        strip: strip::Strip::default(),
        head: head::Head::default(),
        reading: thread::Reading::default(),
        threads: HashMap::new(),
        asking: Latest::default(),
        reads: reads::Reads::default(),
        pictures: pictures_for(false),
        history: Vec::new(),
        walk: walk::Walk::default(),
        source,
        mailbox: Mailbox::default(),
        error: None,
        outbox: write::Outbox::default(),
        refused: None,
        held: None,
        undos: Vec::new(),
        answering: answer::Answering::default(),
        came_back: HashSet::new(),
        kept_drafts,
        #[cfg(test)]
        tried: Vec::new(),
        #[cfg(test)]
        pretend_sent: false,
        #[cfg(test)]
        last_sent: None,
        toast: None,
        last_toast: None,
        hidden: HashSet::new(),
        later: later::Picker::closed(),
        sign_in: need::SignIn::default(),
        github: need::GitHub::default(),
        cloud_needs: cloud_need::Needs::default(),
        closing: closing::Closing::default(),
        palette: palette::Bar::closed(),
        compose: compose::Composer::default(),
        rail: chrome::Rail::default(),
        usage_card: usage::Card::default(),
        pointed: feel::Pointed::default(),
        models: rules::compose::Models::built(),
        reading_models: Latest::default(),
        picks,
        window: stood,
        withdrawn: HashMap::new(),
        viewer: viewer::Viewer::default(),
        films: film::Films::default(),
        pane: pane::Pane::default(),
        changes: change::Changes::default(),
        tables: table::Sheets::default(),
        files: files_page::Place::default(),
        settings: settings::Screen::default(),
        retries: foot::Retries::default(),
        why_up: None,
        repeats: foot::Repeats::default(),
    };
    // The first skies and a fixture are in place before the first frame, so a picture is whole.
    let first = paint_skies(s.card_size());
    s.hang(first);
    s.take_up_drafts(drafts_kept, draft_files);
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
                // What the picture's page had been told besides the snapshot: the models its
                // settings listed (`settings`), and what was last picked in the box (`picks`).
                if let Some(settings) = s.fixture.get("settings") {
                    s.models = rules::compose::Models::from_settings(settings);
                }
                if let Some(kept) = s.fixture.get("picks").and_then(|kept| serde_json::from_value(kept.clone()).ok()) {
                    s.picks = picks::Kept::only_here(kept);
                }
                // And the Inbox's display it was drawn on (`display`), kept nowhere but here.
                if let Some(shown) = s.fixture.get("display").cloned() {
                    s.picks.change(|p| p.display = rules::display::Display::from(shown));
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
                if let Ok(settings) = source::call(&url, &token, "zero:settings-read", &serde_json::json!({ "search": false })) {
                    s.models = rules::compose::Models::from_settings(&settings);
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
    files_page::pictured(&mut s);
    // A picture of the list taken after its first thread was opened and left says so.
    if s.fixture.remove("lastThread").is_some() {
        walk::pictured_after_a_thread(&mut s);
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
            walk::opened_from_the_list(&mut s);
        }
        // A picture taken with the reply box open, or with the arrow on an option, says so in
        // its fixture (`replying` is what was typed, `option` the option's number).
        if let Some(words) = s.fixture.get("replying").and_then(Value::as_str) {
            s.reply = TextEdit::new(words);
            s.reply.focused = true;
            s.replying = true;
            // And the files in it (`replyFiles`), and whether it had been folded over them.
            s.picture_attached(attach::Holder::Reply);
            if s.fixture.get("folded") == Some(&Value::Bool(true)) {
                s.fold_reply();
            }
        }
        s.option = s.fixture.get("option").and_then(Value::as_u64).and_then(|n| (n as usize).checked_sub(1));
        // And one taken with the Later picker open (`later` is what was typed into it,
        // `laterRefused` that Enter was pressed on words it could not read).
        if let (Some(words), View::Thread { id, product }) = (s.fixture.get("later").and_then(Value::as_str), &s.view) {
            s.later = later::Picker::open_on(Some((id.clone(), product.clone())), words);
            s.later.refused = s.fixture.get("laterRefused").and_then(Value::as_bool).unwrap_or(false);
        }
    }
    // And one taken after a sign-in card was pressed or typed into (`signInPressed`).
    need::picture(&mut s);
    need::picture_github(&mut s);
    cloud_need::picture(&mut s);
    // And what the page was told of this computer's plans (`zero:engine-setup`).
    plan_bar::picture(&mut s);
    // A picture taken with the command bar open (`palette` is what was typed into it).
    palette::picture(&mut s);
    // Nor can it wait for a conversation behind the door to be read.
    if rune::tooling::headless_run().is_some()
        && let Some(ask) = s.read_owed(false)
        && let (key, Ok(answer)) = reads::read(ask)
    {
        s.threads.insert(key, answer);
    }
    // A picture taken with words typed into the box a thread is started in says so in its
    // fixture (`composing`): the same words, with the caret after them. And one taken with the
    // New thread card up (`card`) has the box there.
    if let Some(words) = s.fixture.get("composing").and_then(Value::as_str) {
        s.compose.words = TextEdit::new(words);
    }
    if s.fixture.contains_key("composing") || s.fixture.get("card") == Some(&Value::Bool(true)) {
        // With a menu open the keyboard is in the menu, and the words have no caret.
        s.compose.words.focused = !s.fixture.contains_key("menu");
        s.compose.card = s.fixture.get("card") == Some(&Value::Bool(true));
    }
    // And one taken with files in that box says which (`attached`, `pasted`).
    s.picture_attached(attach::Holder::Compose);
    // And one of Settings says its page and what its reads were answered (`settingsPage`).
    settings::picture(&mut s);
    s
}

/// A picture taken just after E says so in its fixture (`closed`): the same key is pressed
/// here once the window has its clock, on the open thread or on the Inbox's picked row. One
/// taken just after a thread was sent hands over what the page was answered (`sent`).
fn press_what_the_picture_pressed(s: &mut State, tx: &mut Tx) {
    // E, as many times as the picture's page had it pressed (`true` is once).
    if let Some(closed) = s.fixture.remove("closed") {
        for _ in 0..closed.as_u64().unwrap_or(1) {
            keys::close_under_the_keys(s, tx);
        }
    }
    if let Some(sent) = s.fixture.remove("sent") {
        s.picture_sent(&sent, tx);
    }
    // And one taken between Send and the answer says what was sent (`sending`): the same words
    // are sent here, to nobody, which is as far as a picture's send ever gets.
    if let Some(words) = s.fixture.remove("sending") {
        s.compose.words = TextEdit::new(words.as_str().unwrap_or_default());
        s.send_thread(tx);
    }
    // And one taken a moment after a reply was sent from the open thread's box (`replySent`),
    // and after the next keys pressed there (`answer::picture`).
    // A close asked first (`closePressed`), before the keys pressed after it.
    crate::closing::picture(s, tx);
    crate::answer::picture(s, tx);
    // And a press on the threads it filed (`filedPressed`).
    crate::made::picture(s, tx);
    // One taken with a menu of that box open says which (`menu`).
    if let Some(which) = s.fixture.remove("menu") {
        s.picture_menu(which.as_str().unwrap_or_default(), tx);
    }
    // And one taken with a conversation's list of files open (`filesOpen`), its working line
    // open onto its steps (`liveOpen`), or a file open big.
    if s.fixture.remove("filesOpen").is_some() {
        s.open_files();
    }
    s.picture_live();
    // And one taken a moment after a line found from the command bar was pressed (`foundLine`).
    s.picture_found_line();
    // A file beside the conversation, before one opened big over it.
    pane::pictured(s);
    change::pictured(s);
    viewer::pictured(s);
    // And one taken after a model was picked in the reply box (`replyPick`), and after picks in
    // its priority and mode chips (`replyPicked`).
    if let (Some(pick), Some(item)) = (s.fixture.remove("replyPick"), s.open_item().cloned()) {
        let text = |key: &str| pick.get(key).and_then(Value::as_str).map(str::to_string);
        s.reply_picks.insert(format!("{}/{}", item.product, item.id), rules::reply_pick::ReplyPick { model: text("model"), effort: text("effort"), engine: text("engine") });
    }
    if let Some(picked) = s.fixture.remove("replyPicked") {
        chips::picture_picked(s, &picked, tx);
    }
    // And one taken with the agent's choices folded away by their little arrow (`choicesFolded`).
    if s.fixture.remove("choicesFolded").is_some() {
        chips::toggle_choices(s);
    }
    // And one taken with something of the strip open, the reply box's model menu, or the menu
    // of one of its other chips (`actions`).
    if let Some(which) = s.fixture.remove("actions") {
        match which.as_str().unwrap_or_default() {
            "model" => s.picture_reply_menu(tx),
            which if chips::picture(s, which, tx) => {}
            which => s.picture_strip(which, tx),
        }
    }
    // And one of the Inbox with a menu of its strip open, or ⌘C's question (`head.rs`).
    s.picture_head();
}

fn setup(s: &mut State, wake: Waker) {
    s.painting.set_waker(wake.clone());
    s.asking.set_waker(wake.clone());
    s.reading_models.set_waker(wake.clone());
    s.head.set_waker(wake.clone());
    s.palette.set_waker(wake.clone());
    s.read_the_models();
    s.outbox.wake_with(wake.clone());
    s.compose.staging.wake_with(wake.clone());
    s.settings.wake_with(wake.clone());
    s.pictures.set_waker(wake.clone());
    s.files.set_waker(wake.clone());
    s.films.set_waker(wake.clone());
    // The replies the last window never saw land are looked for (`answer.rs`).
    s.look_for_unsent(wake.clone());
    s.retries.wake_with(wake.clone());
    s.repeats.wake_with(wake.clone());
    s.changes.set_waker(wake.clone());
    // Whether a plan can run an agent, asked as the window opens (`plan_bar.rs`).
    s.plan.set_waker(wake.clone());
    plan_bar::check(s);
    if let Source::Door(door) = s.source.clone() {
        front::tell_the_app(door.clone());
        source::follow_door(door, s.mailbox.clone(), move || wake());
    }
}

fn poll(s: &mut State, at: f64, tx: &mut Tx) -> bool {
    let mut changed = false;
    s.at = at;
    if let Some(p) = s.painting.take() {
        s.hang(p);
        changed = true;
    }
    // Home's sky is held while Home is the page and ten seconds after, and no longer.
    changed |= s.tend_the_home_sky(tx);
    // The reply box's chips follow the box: opened, they start on its thread's own mode.
    chips::tend(s);
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
    // The bar along the foot of a line that names a thread runs out with the time left to
    // open it, so the line is drawn again while it is up.
    if s.toast.as_ref().is_some_and(|t| t.goes.is_some()) {
        s.wakes.every(tx, "a line's bar", std::time::Duration::from_millis(33));
        changed = true;
    }
    changed |= s.say_if_not_taken(at, tx);
    // A place picked in the strip's panel is drawn until the engine has its own say.
    changed |= s.tend_strip(at, tx);
    if let Some(Ok(settings)) = s.reading_models.take() {
        s.models = rules::compose::Models::from_settings(&settings);
        changed = true;
    }
    // Agentbox Cloud's plan, for the cloud menu's month (`head.rs`).
    changed |= s.plan_landed();
    // And whether a plan can run an agent at all (`plan_bar.rs`).
    changed |= plan_bar::poll(s);
    let mut stale = false;
    let mut arrived = None;
    {
        let mut m = s.mailbox.lock().unwrap();
        if let Some(snapshot) = m.snapshot.take() {
            s.snap = snapshot;
            stale = true;
            // An answer written before this reading is in it: the lists say where it is now.
            s.answering.holding.retain(|_, landed| !*landed);
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
        s.wakes.every(tx, "the clock", std::time::Duration::from_secs(20));
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
    // A thread moved from the command bar opens as its copy once the lists have it.
    changed |= palette::tend(s);
    // What the bar's words found inside threads, once the app has said.
    changed |= palette::found_landed(s);
    // What was sent and has long been written lets go of its drawn copy; what the boxes hold is
    // written down; and a thread opened onto a draft gets its caret (`answer.rs`, `drafts.rs`).
    changed |= s.tend_answering(tx);
    changed |= s.tend_unsent(tx);
    s.tend_keys(tx);
    s.keep_drafts();
    if std::mem::take(&mut s.answering.focus_reply) && s.replying {
        tx.focus_when_shown("reply", false);
    }
    // A picture the engine has turned for the box a thread is started in.
    changed |= s.land_staged();
    // The Files place's list, read again as the store changes, and a tile's file opening.
    changed |= files_page::tend(s, clock(), stale, tx);
    // What Settings' reads and writes were answered.
    changed |= settings::tend(s, tx);
    // The file beside a conversation: opened by the newest answer, and read again as it changes.
    changed |= pane::tend(s, clock(), tx);
    // The code a thread changed: its figures read, the change read, a save landing.
    changed |= change::tend(s, clock(), stale, tx);
    // A sheet beside it or big: its saves, where its keys are, its logos (`table.rs`).
    changed |= table::tend(s, clock(), tx);
    changed |= chrome::rail_moves(s, at, tx);
    changed |= walk::tend(s, at, tx);
    changed |= usage::tend(s, at, tx);
    changed |= feel::hint_due(s, at);
    // What Try again came back with, and the line at a conversation's foot kept counting.
    changed |= foot::tend(s, tx);
    // What a cloud run's cards ask the engine of the files they name, and the keys they hand on.
    changed |= cloud_need::tend(s, tx);
    // The open conversation's film of its browser, read as often as it can change.
    changed |= film::tend(s, tx);
    // A line found from the command bar, looked for and landed on; before the landing, which it
    // ends once it is found.
    changed |= s.tend_the_found_line(tx);
    // Where the person was in the conversation they left, and where they land in this one.
    changed |= s.tend_the_reading(tx);
    // A close's question just raised, brought onto the screen once it is drawn.
    changed |= closing::bring_it(s, tx);
    if let Some((key, answer)) = s.asking.take() {
        // Whether it came back is said under the working line's steps (`thread.rs`).
        s.read_came_back(&key, answer.is_ok());
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
        s.wakes.every(tx, "the open conversation", std::time::Duration::from_millis(reads::AGAIN_MS as u64));
    }
    // The open conversation's pictures: what was read lands, another conversation's are let
    // go, and while any is on the page the disk is looked at again every two seconds.
    let open = match &s.view {
        View::Thread { id, product } => Some(format!("{product}/{id}")),
        View::Files { .. } => Some("files".to_string()),
        _ => None,
    };
    let (landed, again) = s.pictures.tend(open.as_deref(), clock());
    if again {
        s.wakes.every(tx, "the pictures", std::time::Duration::from_millis(pictures::AGAIN_MS as u64));
    }
    changed || stale || landed
}

fn resized(s: &mut State, [_, _, w, h]: [f32; 4]) {
    if (s.size.0 - w).abs() < 0.5 && (s.size.1 - h).abs() < 0.5 {
        return;
    }
    s.size = (w, h);
    // A key hint goes when the window is resized, as the web window's does.
    s.pointed.hint = None;
    let points = chrome::folded_card_size(s);
    s.painting.request(points, paint_skies);
}

fn view(s: &State, cx: &mut Cx<State>) -> El {
    let (w, h) = s.size;
    // A picture's copy is made for this screen's pixels.
    s.pictures.set_scale(cx.env().scale);
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
        // And what the pointer changes comes and goes at once, as the web window's `:hover`
        // rules do with no transition of their own; Rune's own fades out over a tenth of a
        // second. A control the web eases says so on itself (`feel.rs`).
        .hover_motion_within(rune::Anim::INSTANT, rune::Anim::INSTANT)
        .on_layout(resized)
        .child(chrome::backdrop(w, h));
    // Settings is a screen over everything: the rail, the card and the page under it are not
    // drawn while it is up (`.workspace-settings`), only its own, the line about the last thing
    // done and the band that moves the window.
    if s.settings.open {
        return root.children(settings::view(s)).children(chrome::toast_corner(s)).children(chrome::drag_band(s));
    }
    let page = match &s.view {
        View::Home => home::view(s),
        View::Inbox => inbox::view(s),
        // With a file beside it, the conversation narrowed and the file's pane on its right.
        View::Thread { id, product } => pane::page(s, thread::view(s, id, product, cx.is_reduced_motion()), |wide| thread::dock_alone(s, id, product, wide)),
        View::Files { .. } => files_page::view(s),
    };
    // A file that takes the whole card takes the strip's place too (`pane.rs`).
    let (strip, top) = if pane::takes_the_card(s) { (pane::strip(s), chrome::CARD[1]) } else { (chrome::back_forward(s), chrome::page_top(&s.view)) };
    let root = root.child(chrome::card(s));
    // The marks over a change that takes the workspace, woken by the pointer moving, lie under
    // the page, as the web window's lie under the change's own sticky heads (`change.rs`).
    let root = change::over_the_workspace(s, root);
    // The bar that says agents cannot start until a plan is signed in stands first on the page,
    // which is laid out under it (`plan_bar.rs`).
    let bar = plan_bar::view(s, top);
    let top = top + plan_bar::height(s);
    let root = root
        .child(chrome::rail(s, cx.is_reduced_motion()))
        .child(strip)
        .children(bar)
        // The page scrolls inside the card, under its top strip.
        .child(
            el().pin(Some(chrome::card_left(s)), Some(top), Some(chrome::CARD[2]), Some(chrome::CARD[3]))
                .clip_corners([0.0, 0.0, 6.0, 6.0])
                .scroll(ScrollSpec { stick_bottom: false })
                .child(page),
        );
    // The card of the agents' limits, risen from the rail's usage row over the page.
    let root = root.children(usage::layer(s));
    // The list a conversation's Files button opens, over the page and under everything else, and
    // what the rest of its strip opens; over the Inbox, what its strip opens.
    let root = root.children(thread::files_list(s)).children(strip::layer(s)).children(head::layer(s));
    // A menu of Files' home: a file's, a folder's or New's (`files_home.rs`).
    let root = root.children(files_home::layer(s));
    // The card saying why the cloud has not got a thread, while the pointer rests on its word.
    let root = root.children(foot::why_layer(s));
    // The box a thread is started in: its open menu over the page it hangs in, or the New
    // thread card over the whole page, under the line about the last thing done.
    let root = root.children(compose::layers(s));
    // The menu of the reply box's priority or mode chip, over the page.
    let root = root.children(chips::layer(s));
    let root = match chrome::toast_card(s) {
        Some(toast) => root.child(toast),
        None => root,
    };
    // The Later picker stands over everything, the line about the last thing done included.
    let root = match later::view(s) {
        Some(picker) => root.child(picker),
        None => root,
    };
    // A file open big covers all of that, the line about the last thing done included, as the
    // web window's view covers its own notices.
    let root = root.children(viewer::layer(s)).children(film::layer(s));
    // A sheet's menus, over it wherever it is open.
    let root = root.children(table::layer(s));
    // And the command bar, which never stands with it: opening one closes the other.
    let root = match palette::view(s) {
        Some(bar) => root.child(bar),
        None => root,
    };
    // ⌘C's question before the cloud goes off, over everything as the command bar is.
    let root = root.children(head::off_ask(s));
    let root = match chrome::notice(s) {
        Some(notice) => root.child(notice),
        None => root,
    };
    // The band that moves the window, and the key hint of the control the pointer has waited
    // on, over everything (`chrome.rs`, `feel.rs`).
    root.children(chrome::drag_band(s)).children(feel::plate(s))
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

/// Starts the program again with macOS's allocator told to favour space over speed
/// (`MallocSpaceEfficient=1`), unless it was started so already.
///
/// The allocator otherwise keeps up to 220 MB of the large blocks a program frees, mapped and
/// counted against it, and returns them only when the whole system runs short. The window frees
/// a great deal in bursts (a picture is read whole to be shrunk to the size it is drawn at, a
/// sky is painted over, a conversation's logs are parsed), so a conversation unfolded onto fifty
/// pictures stood at 600 MB where 315 MB was in use. The setting is read once, as a program
/// starts, from its environment and nowhere else, which is why the program starts itself again:
/// the same file, the same words, under the same number, before anything else has run. Asking
/// the allocator for the blocks back later (`malloc_zone_pressure_relief`) gives nothing on a
/// current macOS.
#[cfg(target_os = "macos")]
fn start_again_frugal() {
    use std::os::unix::process::CommandExt;
    const SETTING: &str = "MallocSpaceEfficient";
    if std::env::var_os(SETTING).is_some() {
        return;
    }
    let mut words = std::env::args_os();
    let (Some(name), Ok(file)) = (words.next(), std::env::current_exe()) else { return };
    // This only comes back when it could not start: the window then opens as it is.
    let _ = std::process::Command::new(file).arg0(name).args(words).env(SETTING, "1").exec();
}

#[cfg(not(target_os = "macos"))]
fn start_again_frugal() {}

fn main() {
    start_again_frugal();
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
    // Where the window stood when it was last open, which it opens at again (`window.rs`). A
    // headless picture has the size it was asked for.
    let args = rune::tooling::extra_args();
    let stood = match rune::tooling::headless_run() {
        None => window::Kept::at(window_file(args, &source::from_args(args))).get(),
        Some(_) => None,
    };
    // The web app sets `-webkit-font-smoothing: antialiased`; `Off` is that rendering.
    // What waits its three seconds is written before the window goes, and the boxes are written
    // down (`pagehide` in `renderer/src/App.tsx`).
    let mut app = App::new(spec)
        .font_smoothing(FontSmoothing::Off)
        .on_focus(focus_changed)
        .on_close(closing)
        .on_rest(rested)
        .on_window(move |made| window::opened(made, stood));
    if let Some(frame) = stood {
        app = app.window_options(frame.options()).maximized(frame.maximized);
    }
    // The desktop app's banner and tray bring this window forward from another thread.
    front::reach(app.handle());
    app.run();
}

/// The window's frame is kept at the three moments it can have changed and nothing is being
/// drawn for it: as the keys leave it, as it closes (an install closes it to replace it), and
/// as it rests (ten seconds after its last frame, or covered, hidden or in the Dock).
fn focus_changed(s: &mut State, focused: bool, tx: &mut Tx) {
    if !focused {
        window::keep(s);
    }
    front::focus_changed(s, focused, tx);
}

fn closing(s: &mut State, tx: &mut Tx) {
    // First, since what waits to be sent may take its seconds to go.
    window::keep(s);
    answer::on_close(s, tx);
}

fn rested(s: &mut State, _tx: &mut Tx) -> bool {
    window::keep(s);
    false
}
