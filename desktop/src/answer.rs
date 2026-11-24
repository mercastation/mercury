//! Answering a thread and stopping its agent, as the web window does both
//! (`docs/parity-answering.md` lists every line): the Stop that takes Send's place while a run
//! is under way, what the person sent drawn in the conversation from the press on, Z's pile and
//! the question it asks past half a minute, the lines a Z leaves in a thread, the question an
//! option that throws work away asks first, the once-ever warning before a waiting agent is
//! closed or put off, the person following a thread they stopped or stayed on, and the line over
//! the reply box of the thread a send moved them to. The rules are `rules/answering.rs`.

use std::collections::HashSet;

use rune::{El, TextStyle, Tx, el, row, text};

use crate::data::Item;
use crate::rules::answering as rules;
use crate::rules::format::Choice;
use crate::talk::{self, Sending};
use crate::theme::*;
use crate::write::{self, Mark, Undo};
use crate::{State, View};

/// Something the person sent, drawn in its conversation from the press until its written copy
/// has stood in its place a while (`sending`, `PendingSaid` in `renderer/src/App.tsx`).
#[derive(Clone, Debug)]
pub struct Drawn {
    pub id: u64,
    /// The thread, as `product/id`.
    pub thread: String,
    /// The press, on the clock the ledger is stamped by.
    pub at: f64,
    pub text: String,
    /// Inside the three seconds Z still takes it back whole.
    pub held: bool,
    /// When the engine wrote it, on the same clock: the drawn copy goes `LANDED_MS` after.
    pub landed: Option<f64>,
}

/// A Z pressed on a thread, kept as the web window keeps its own (`zero.undoMarks.v1`, the
/// newest 300): drawn in that thread as one of the person's own actions.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UndoMark {
    pub thread: String,
    pub from: f64,
    pub at: f64,
    pub words: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choice: Option<String>,
}

/// The question an option that throws the change away asks first, on one thread: the option it
/// would send, and which of its two answers the arrows are on.
#[derive(Clone, Debug, PartialEq)]
pub struct Discard {
    pub thread: String,
    pub choice: Choice,
    pub lit: Option<usize>,
}

#[derive(Default)]
pub struct Answering {
    pub sending: Vec<Drawn>,
    next: u64,
    /// The pile entry Z last asked about past its half minute, and when, as `(entry's stamp,
    /// asked at)`: a second Z inside eight seconds does it.
    pub asked: Option<(f64, f64)>,
    /// The entry whose writes are on their way: done, its lines are said; refused, it goes
    /// back on the pile.
    pub undoing: Option<Undo>,
    pub marks: Vec<UndoMark>,
    /// The thread the person stopped or stayed on after a reply: leaving it goes to Needs you
    /// with its row picked (`following`, `wayOut`).
    pub following: Option<(String, String)>,
    pub discard: Option<Discard>,
    /// Threads whose waiting message was sent now, until nothing waits on them.
    pub cutting: HashSet<String>,
    /// The thread the app last put the person on by itself, and when on the window's clock: a
    /// number pressed within 0.8 s of that was meant for the thread before (`pickLandsHere`).
    pub arrived: Option<(String, f64)>,
    /// The option strip's top while the pointer is on it, as the pointer last found it, and the
    /// top it was last seen at with when it moved there under a pointer that had not moved, on
    /// the window's clock (`stripUnderThePointer`): noticed as the page is drawn.
    pub strip: Option<f32>,
    pub strip_moved: std::cell::Cell<Option<(f32, f64)>>,
    /// Where the pointer last was over the conversation, in window points, and where it was
    /// when the app last moved the person onto a thread (`pointer`, `arrived.from`).
    pub pointer: Option<(f32, f32)>,
    pub arrived_from: Option<(f32, f32)>,
    /// The option the pointer is on, whose words show whole over the strip when they are cut
    /// short (`peekOption`).
    pub peek: Option<usize>,
    /// Whether the once-ever warning about a waiting agent has been learned (`zero.stoppedWarned`):
    /// once E went ahead on the thread it warned of, or once L's picker said it.
    pub warned: bool,
    /// The thread E's last warning was about, `product:id` (`heldFor`): the next E goes ahead
    /// there, and anywhere else is warned about again.
    pub warned_of: Option<String>,
    /// When the last Z went through, by the window's clock: a second Z within the web window's
    /// burst of it is the same press said twice (`rules::e_and_z::z_said_twice`).
    pub z_at: Option<f64>,
    /// The replies sent and not yet written, by thread, written down so a restart cannot lose
    /// them (`markReplyInFlight`), and the looks for them a restart left owed.
    pub in_flight: std::collections::BTreeMap<String, Vec<crate::drafts::InFlight>>,
    /// Which files the boxes held when they were last written down.
    pub files_print: String,
    /// The box's caret is owed to the box at the next turn: a thread opened with a draft in its
    /// box opens the box on it.
    pub focus_reply: bool,
    /// The threads an answer was held or written on, out of Needs you until the engine's lists
    /// say so themselves, by id: true once the write has landed, and let go with the next
    /// snapshot after that.
    pub holding: std::collections::HashMap<String, bool>,
    /// The line over the thread a send moved the person to, while it stands: its own clock,
    /// which stops while the pointer is on it (`SentLine`).
    pub sent_line: Option<SentClock>,
    /// What the screen is, and when it last changed by itself rather than by E, J or K, and when
    /// one of those last moved it, on the window's clock (`screenChangedAt`, `movedByKeyAt`).
    pub screen: String,
    pub screen_changed_at: Option<f64>,
    pub moved_by_key_at: Option<f64>,
    /// When a letter was last typed into the box from the page, and E or L held a moment to see
    /// whether a word follows (`typedIntoBoxAt`, `waitingKey`): the letter, its thread, and when.
    pub typed_at: Option<f64>,
    pub waiting_key: Option<(String, String, f64)>,
    /// What the looks for replies a restart may have lost found, by thread: the stamps that
    /// landed, and the replies never found, to go back in their boxes.
    #[allow(clippy::type_complexity)]
    pub unsent: std::sync::Arc<std::sync::Mutex<Vec<(String, Vec<f64>, Vec<crate::drafts::InFlight>)>>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SentClock {
    /// The thread it stands on, as `product/id`, and the line it says, by when that went up.
    pub on: String,
    pub toast: f64,
    /// What is left of its five seconds, and the window's clock when that was last counted.
    pub left: f64,
    pub counted: f64,
    pub hovered: bool,
}

/// The kept marks: the newest 300.
const MAX_UNDO_MARKS: usize = 300;

/// Keys go into the box for a second after the last one typed into it from the page, and E or
/// L wait this long for a letter after them (`STILL_TYPING_MS`, `WORD_FOLLOWS_MS`).
const STILL_TYPING_MS: f64 = 1000.0;
const WORD_FOLLOWS_MS: f64 = 350.0;

fn key(item: &Item) -> String {
    format!("{}/{}", item.product, item.id)
}

impl Answering {
    /// Draws what was just sent, and returns its number.
    pub fn draw(&mut self, thread: &str, at: f64, text: &str) -> u64 {
        self.next += 1;
        self.sending.push(Drawn { id: self.next, thread: thread.to_string(), at, text: text.to_string(), held: true, landed: None });
        self.next
    }
    pub fn let_go(&mut self, id: u64) {
        self.sending.retain(|d| d.id != id);
    }
    fn drawn(&mut self, id: u64) -> Option<&mut Drawn> {
        self.sending.iter_mut().find(|d| d.id == id)
    }
    /// What the thread's conversation places among its lines, each as its head says it.
    pub fn sent_on(&self, thread: &str, waits_on_the_step: bool) -> Vec<talk::Sent> {
        let cutting = self.cutting.contains(thread);
        self.sending
            .iter()
            .filter(|d| d.thread == thread)
            .map(|d| talk::Sent {
                at: d.at,
                text: d.text.clone(),
                sending: if d.held {
                    Sending::Held
                } else if cutting {
                    Sending::SendingNow
                } else if waits_on_the_step {
                    Sending::WaitingOnTheStep
                } else {
                    Sending::OnItsWay
                },
            })
            .collect()
    }
    pub fn undone_on(&self, thread: &str) -> Vec<talk::Undid> {
        self.marks.iter().filter(|m| m.thread == thread).map(|m| talk::Undid { from: m.from, at: m.at, words: m.words.clone(), choice: m.choice.clone() }).collect()
    }
    pub fn leave_mark(&mut self, thread: &str, mark: &Mark, from: f64, at: f64) {
        self.marks.push(UndoMark { thread: thread.to_string(), from, at, words: mark.words.clone(), choice: mark.choice.clone() });
        let over = self.marks.len().saturating_sub(MAX_UNDO_MARKS);
        self.marks.drain(..over);
    }
}

impl State {
    /* ------------------------------------- stopping ------------------------------------- */

    /// Whether a run is on the thread right now (`working`).
    pub fn run_on(&self, item: &Item) -> bool {
        self.snap.supervisor.running.iter().any(|r| r.item_id == item.id && r.product == item.product)
    }

    /// Whether the thread can be stopped now: In progress by the list's own facts.
    pub fn stoppable_now(&self, item: &Item) -> bool {
        let starting = self.snap.supervisor.rest.get("starting").and_then(serde_json::Value::as_array).is_some_and(|s| s.iter().any(|id| id.as_str() == Some(&item.id)));
        let live = self.run_on(item) || starting;
        let listed = |field: &str| self.snap.supervisor.rest.get(field);
        let stalled = listed("stalled").and_then(serde_json::Value::as_array).is_some_and(|s| s.iter().any(|id| id.as_str() == Some(&item.id)))
            || listed("signInNeeded").and_then(serde_json::Value::as_object).is_some_and(|m| m.contains_key(&item.id));
        let snoozed = self.local.snooze.get(&item.id).copied().unwrap_or(0.0);
        rules::stoppable(item, live, stalled, snoozed, self.now)
    }

    /// Whether the open box puts Stop where Send is: the thread is under way and the box holds
    /// no words and no files (`stop && !hasDraft`).
    pub fn stop_in_the_box(&self, item: &Item) -> bool {
        self.reply.text.trim().is_empty() && self.reply_files.open.is_empty() && self.stoppable_now(item)
    }

    /// Stops the thread's agent: what is held is written first, so a reply still in its three
    /// seconds lands before the stop and does not reopen the row after it; then the stop, which
    /// parks the row in Needs you. The person stays on the thread, following it, and the line
    /// is said once the engine has it (`stopAgent`).
    pub fn stop_agent(&mut self, id: &str, product: &str) {
        let Some(item) = self.item(id, product).cloned() else {
            return;
        };
        if !self.stoppable_now(&item) {
            return;
        }
        self.write_what_waits();
        let running = self.run_on(&item);
        if matches!(&self.view, View::Thread { id: open, product: of } if open == id && of == product) {
            self.answering.following = Some((id.to_string(), product.to_string()));
        }
        let (channel, payload) = rules::stop(&item);
        self.send(channel, payload, write::Sent::Stop { thread: key(&item), running });
    }

    /// Send now, on a message waiting on the step its Claude Code run is in.
    pub fn send_now(&mut self, item: &Item) {
        self.answering.cutting.insert(key(item));
        let payload = serde_json::json!({ "product": item.product, "id": item.id });
        self.send("zero:send-now", payload, write::Sent::SendNow { thread: key(item) });
    }

    /* ---------------------------------- what was sent ---------------------------------- */

    /// The three seconds are over and a reply to a run that was up goes to it: its head says
    /// it is on its way, and Z can no longer take it back. What Z does instead goes on the pile
    /// now, and comes off again if the hand-over fails.
    pub(crate) fn hand_over(&mut self, answer: &write::Answer, thread: &(String, String)) {
        if let Some(drawn) = self.answering.drawn(answer.drawn) {
            drawn.held = false;
        }
        let (id, product) = thread.clone();
        let mut gone = Undo::plain(rules::ALREADY_WITH_THE_AGENT.to_string(), rules::ALREADY_UNDOES.to_string(), Vec::new(), write::Sent::TakenBack { thread: format!("{product}/{id}") }, thread.clone(), true);
        gone.back = None;
        self.undos.push(gone);
    }

    /// The written copy of something sent is in: the drawn one stands `LANDED_MS` more.
    pub(crate) fn drawn_landed(&mut self, id: u64) {
        let at = self.moment();
        if let Some(drawn) = self.answering.drawn(id) {
            drawn.landed = Some(at);
        }
    }

    /// Lets go of drawn copies whose written copy has long stood in their place, and of a cut
    /// nothing waits on any more. True when something went.
    pub(crate) fn tend_answering(&mut self, tx: &mut Tx) -> bool {
        let now = self.moment();
        let before = self.answering.sending.len();
        self.answering.sending.retain(|d| d.landed.is_none_or(|at| now - at < rules::LANDED_MS));
        if let Some(soonest) = self.answering.sending.iter().filter_map(|d| d.landed).map(|at| at + rules::LANDED_MS - now).reduce(f64::min) {
            tx.wake_after(std::time::Duration::from_millis(soonest.max(0.0) as u64 + 1));
        }
        let waiting: HashSet<String> = self.answering.sending.iter().filter(|d| !d.held).map(|d| d.thread.clone()).collect();
        self.answering.cutting.retain(|t| waiting.contains(t));
        before != self.answering.sending.len() || self.tend_sent_line(tx)
    }

    /// The thread the line about a send stands over: the open one, when the send was to another
    /// thread and moved the person here (`saysWhereItWent`).
    fn sent_line_on(&self) -> Option<String> {
        let toast = self.toast.as_ref()?;
        toast.sent.as_ref()?;
        let (id, product) = toast.goes.as_ref()?;
        match &self.view {
            View::Thread { id: here, product: of } if here != id || of != product => Some(format!("{of}/{here}")),
            _ => None,
        }
    }

    /// The line's own clock: five seconds, stopped while the pointer is on it, and the line about
    /// the send held for as long as it stands. Leaving the thread it stands on takes it down.
    fn tend_sent_line(&mut self, tx: &mut Tx) -> bool {
        let at = self.at;
        let on = self.sent_line_on();
        let since = self.toast.as_ref().map_or(0.0, |t| t.since);
        match (on, self.answering.sent_line.clone()) {
            (Some(on), clock) => {
                let mut clock = clock.filter(|c| c.on == on && c.toast == since).unwrap_or(SentClock { on, toast: since, left: rules::SENT_LINE_MS, counted: at, hovered: false });
                if !clock.hovered {
                    clock.left -= (at - clock.counted).max(0.0) * 1000.0;
                }
                clock.counted = at;
                if clock.left <= 0.0 {
                    self.toast = None;
                    self.answering.sent_line = None;
                    return true;
                }
                if let Some(toast) = self.toast.as_mut() {
                    toast.until = toast.until.max(at + 1.0);
                }
                self.answering.sent_line = Some(clock);
                self.wakes.every(tx, "a sent line's ring", std::time::Duration::from_millis(33));
                true
            }
            (None, Some(was)) => {
                // Left the thread it stood on, while the same line was up: it goes.
                if self.toast.as_ref().is_some_and(|t| t.since == was.toast) {
                    self.toast = None;
                }
                self.answering.sent_line = None;
                true
            }
            (None, None) => false,
        }
    }

    /// Whether a reply that waits on the step its agent is in can be sent now: a run is up on
    /// the thread, on Claude Code (`cutsARunningStep`).
    pub fn waits_on_the_step(&self, item: &Item) -> bool {
        let thread_on = crate::rules::reply_pick::running_engine(&self.snap, &item.id);
        self.snap.supervisor.running.iter().any(|r| {
            let on = r.rest.get("engine").and_then(serde_json::Value::as_str).map(str::to_string).or_else(|| thread_on.clone());
            r.item_id == item.id && r.product == item.product && on.as_deref().unwrap_or("claude") == "claude"
        })
    }

    /* -------------------------------- a reply on its way -------------------------------- */

    /// Writes a reply down as on its way, so a restart cannot lose it, and returns its stamp
    /// (`markReplyInFlight`). Pictures past the budget are the first thing given up: the words
    /// are what must survive.
    pub(crate) fn mark_in_flight(&mut self, thread: &str, words: &str, body: &str, files: &[crate::attach::Attached]) -> f64 {
        let list = self.answering.in_flight.entry(thread.to_string()).or_default();
        let mut at = write::wall_clock();
        while list.iter().any(|r| r.at == at) {
            at += 1.0;
        }
        let (files, _) = crate::drafts::within_budget(files.iter().map(crate::drafts::KeptFile::of).collect());
        list.push(crate::drafts::InFlight { words: words.to_string(), body: body.to_string(), at, files });
        at
    }

    /// The reply stamped `at` was written, refused back into the box, or taken back (`replyLanded`).
    pub(crate) fn in_flight_landed(&mut self, thread: &str, at: f64) {
        if let Some(list) = self.answering.in_flight.get_mut(thread) {
            list.retain(|r| r.at != at);
            if list.is_empty() {
                self.answering.in_flight.remove(thread);
            }
        }
    }

    /// At launch, the replies the last window wrote down and never saw land are looked for in
    /// their threads, off the main thread: three times, at once and then thirty and ninety
    /// seconds on, after the hold and five seconds more, so a write still on its way can land
    /// first. One never found goes back in its box (`bringBackWhenSure`).
    pub(crate) fn look_for_unsent(&mut self, wake: rune::Waker) {
        let crate::source::Source::Door(door) = self.source.clone() else { return };
        let owed: Vec<(String, Vec<crate::drafts::InFlight>)> = self.answering.in_flight.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        if owed.is_empty() {
            return;
        }
        let found = self.answering.unsent.clone();
        std::thread::spawn(move || {
            for (thread, replies) in owed {
                let Some((product, id)) = thread.split_once('/').map(|(p, i)| (p.to_string(), i.to_string())) else { continue };
                let newest = replies.iter().map(|r| r.at).fold(0.0, f64::max);
                let wait = newest + crate::rules::writes::UNDO_GRACE_MS + 5000.0 - write::wall_clock();
                if wait > 0.0 {
                    std::thread::sleep(std::time::Duration::from_millis(wait as u64));
                }
                let mut left = replies;
                for (n, pause) in crate::drafts::UNSENT_CHECKS_MS.iter().enumerate() {
                    if *pause > 0.0 {
                        std::thread::sleep(std::time::Duration::from_millis(*pause as u64));
                    }
                    let read = door.now().and_then(|(url, token)| crate::source::call(&url, &token, "zero:item-history", &serde_json::json!({ "product": product, "id": id })).ok());
                    let answers: Option<Vec<String>> = read.as_ref().filter(|r| r.get("ok") != Some(&serde_json::Value::Bool(false))).map(|r| {
                        r.get("lines").and_then(serde_json::Value::as_array).into_iter().flatten().filter_map(|l| l.get("patch").and_then(|p| p.get("answer")).and_then(serde_json::Value::as_str).map(str::to_string)).collect()
                    });
                    let last = n + 1 == crate::drafts::UNSENT_CHECKS_MS.len();
                    let Some(answers) = answers else {
                        if last {
                            found.lock().unwrap().push((thread.clone(), left.iter().map(|r| r.at).collect(), left.clone()));
                        }
                        continue;
                    };
                    let (landed, still): (Vec<_>, Vec<_>) = left.into_iter().partition(|r| crate::drafts::landed_in(r, &answers));
                    found.lock().unwrap().push((thread.clone(), landed.iter().map(|r| r.at).collect(), if last { still.clone() } else { Vec::new() }));
                    left = still;
                    if left.is_empty() || last {
                        break;
                    }
                }
                wake();
            }
        });
    }

    /// What the looks found: landed replies let go; those never found back in their boxes,
    /// after what each box holds, and a line naming the thread.
    pub(crate) fn tend_unsent(&mut self, tx: &mut Tx) -> bool {
        let found: Vec<_> = std::mem::take(&mut *self.answering.unsent.lock().unwrap());
        let mut changed = false;
        for (thread, done, back) in found {
            let still: Vec<f64> = self.answering.in_flight.get(&thread).map(|l| l.iter().map(|r| r.at).collect()).unwrap_or_default();
            for at in done {
                self.in_flight_landed(&thread, at);
            }
            // Only what is still written down: a send made since is this window's own.
            let back: Vec<_> = back.into_iter().filter(|r| still.contains(&r.at)).collect();
            if back.is_empty() {
                continue;
            }
            for r in &back {
                self.in_flight_landed(&thread, r.at);
                let files = r.files.iter().filter_map(crate::drafts::attached).collect();
                self.put_back_failed(&thread, &r.words, files);
            }
            if let Some((product, id)) = thread.split_once('/') {
                self.say_about(&crate::drafts::back_in_the_box(back.len()), (id.to_string(), product.to_string()), tx);
            }
            changed = true;
        }
        changed
    }

    /* ------------------------------------- Z's pile ------------------------------------- */

    /// Z with nothing held: the newest thing on the pile, at once while it is under half a
    /// minute old; past that Z names it and only a second Z inside eight seconds does it. Its
    /// writes go out, and its lines are said once the engine has them (`undo_landed`).
    pub(crate) fn undo_from_the_pile(&mut self, tx: &mut Tx) {
        let now = write::wall_clock();
        let Some(top) = self.undos.last() else {
            self.answering.asked = None;
            self.say(crate::rules::writes::NOTHING_TO_UNDO.to_string(), false, tx);
            return;
        };
        let answering_it = self.answering.asked.is_some_and(|(entry, at)| entry == top.at && rules::ask_still_stands(at, now));
        if !rules::instant(top.at, now) && !answering_it {
            self.answering.asked = Some((top.at, now));
            let ask = rules::undo_ask(&top.undoes);
            self.say(ask, false, tx);
            return;
        }
        self.answering.asked = None;
        let Some(last) = self.undos.pop() else { return };
        // A thread sent from the box is taken back its own way (`compose.rs`).
        if matches!(last.sent, write::Sent::Withdraw { .. }) {
            return self.withdraw(last, tx);
        }
        // A reply already with its agent has nothing to write: Z goes back to it.
        if last.writes.is_empty() {
            self.last_toast = None;
            let (id, product) = last.thread.clone();
            self.say(last.said, false, tx);
            self.open_thread(&id, &product);
            return;
        }
        let writes = last.writes.clone();
        let sent = last.sent.clone();
        let mut last = last;
        // Stamped as it starts, so the line it leaves spans its own writes.
        last.at = write::wall_clock();
        self.answering.undoing = Some(last);
        if !self.send_all(writes, sent.clone()) {
            // Not sent, so not undone: Z still has it.
            if let Some(back) = self.answering.undoing.take() {
                self.undos.push(back);
            }
        } else {
            // What Z puts back on the Files place is drawn as the key is pressed (`actNow`).
            crate::files_page::undoing(self, &sent);
        }
    }

    /// What the engine said to a Z's writes. Done: the thread's line, the words given back to
    /// the box when it holds nothing newer, the line about it, and the thread opened when it
    /// comes back in front of the person. Refused: back on the pile, and why.
    pub(crate) fn undo_landed(&mut self, landed: write::Landed, tx: &mut Tx) {
        let Some(mut done) = self.answering.undoing.take() else {
            if let Some(why) = landed.refusal() {
                self.say(rules::could_not_undo(&why), false, tx);
            }
            return;
        };
        if let Some(why) = landed.refusal() {
            self.say(rules::could_not_undo(&why), false, tx);
            self.undos.push(done);
            return;
        }
        let (id, product) = done.thread.clone();
        let thread = format!("{product}/{id}");
        if let Some(mark) = &done.mark {
            self.answering.leave_mark(&thread, mark, done.at, write::wall_clock());
            for also in &done.also {
                self.answering.leave_mark(also, mark, done.at, write::wall_clock());
            }
        }
        let back = done.back.take().map(|back| self.give_back(&thread, back));
        if back.is_some() && self.answering.following.as_ref().is_some_and(|(i, p)| *i == id && *p == product) {
            self.answering.following = None;
        }
        self.last_toast = None;
        let said = if back == Some(true) { format!("{}{}", done.said, rules::BACK_IN_THE_BOX) } else { done.said.clone() };
        self.say(said, false, tx);
        self.refresh();
        // Words given back are on no other screen, so their thread opens wherever the person is;
        // otherwise what Z put back opens when the entry says so.
        if back == Some(true) {
            self.open_thread(&id, &product);
            self.open_reply(tx);
        } else if done.opens {
            self.open_thread(&id, &product);
        }
    }

    /// A reply's words, model and files back in its box, unless the box holds something newer
    /// (`restoreDraft`). True when they went back.
    pub(crate) fn give_back(&mut self, thread: &str, back: write::GivenBack) -> bool {
        let here = matches!(&self.view, View::Thread { id, product } if format!("{product}/{id}") == thread);
        let holds = if here { !self.reply.text.trim().is_empty() || !self.reply_files.open.is_empty() } else { self.drafts.get(thread).is_some_and(|w| !w.trim().is_empty()) || self.reply_files.holds(thread) };
        if holds || (back.words.trim().is_empty() && back.files.is_empty()) {
            return false;
        }
        if here {
            self.reply = rune::TextEdit::new(back.words.trim().to_string());
        } else if !back.words.trim().is_empty() {
            self.drafts.insert(thread.to_string(), back.words.trim().to_string());
        }
        self.reply_files.put_back(thread, here, back.files);
        if let Some(pick) = back.pick {
            self.reply_picks.insert(thread.to_string(), pick);
        }
        true
    }

    /// A reply that did not get through comes back after whatever the box holds now, a blank
    /// line between, with its files after the box's (`restoreFailedDraft`).
    pub(crate) fn put_back_failed(&mut self, thread: &str, words: &str, files: Vec<crate::attach::Attached>) {
        let here = matches!(&self.view, View::Thread { id, product } if format!("{product}/{id}") == thread);
        let join = |was: &str| [was.trim_end(), words.trim()].into_iter().filter(|w| !w.is_empty()).collect::<Vec<_>>().join("\n\n");
        if here {
            let joined = join(&self.reply.text);
            self.reply = rune::TextEdit::new(joined);
        } else {
            let joined = join(self.drafts.get(thread).map_or("", String::as_str));
            if !joined.is_empty() {
                self.drafts.insert(thread.to_string(), joined);
            }
        }
        self.reply_files.append(thread, here, files);
    }

    /* ------------------------------ where the person goes ------------------------------ */

    /// Leaving a thread the person follows goes to Needs you, its row picked, where a stopped
    /// thread or one answered and stayed on lands (`wayOut`). True when that is where it went.
    pub(crate) fn leave_followed(&mut self) -> bool {
        let View::Thread { id, product } = self.view.clone() else {
            return false;
        };
        if self.answering.following.take().is_none_or(|(i, p)| i != id || p != product) {
            return false;
        }
        self.leave_thread();
        self.go(View::Inbox);
        self.show_tab(crate::data::Tab::Needs);
        if let Some(at) = self.lists.needs.iter().position(|r| r.id == id && r.product == product) {
            self.picked = at;
        }
        true
    }

    /// Whether a reply to the open thread moves the person on to the next one waiting: they
    /// opened it from Needs you on the Inbox, it is in that list, and they did not come to it
    /// from Home or back to it with O (`replyMovesOn`).
    pub(crate) fn reply_moves_on(&self, item: &Item) -> bool {
        let from_needs_you = matches!(self.history.last(), Some(View::Inbox)) && self.inbox_tab == crate::data::Tab::Needs;
        let listed = self.lists.needs.iter().any(|r| r.id == item.id && r.product == item.product);
        let from_home = matches!(self.history.last(), Some(View::Home));
        rules::reply_moves_on(from_needs_you, listed, from_home, self.came_back.contains(&key(item)))
    }

    /// On to the thread now in the answered one's place in Needs you, or the one above it when
    /// it was the last; with none left, to the Inbox (`leaveResolved`, `nextAfterAdvance`). The
    /// app moved the person, so a number pressed in the next moment was not meant for it.
    pub(crate) fn next_after(&self, item: &Item) -> Option<(String, String)> {
        let rows = &self.lists.needs;
        rows.iter().position(|r| r.id == item.id && r.product == item.product).and_then(|at| {
            let rest: Vec<&crate::data::Row> = rows.iter().filter(|r| !(r.id == item.id && r.product == item.product)).collect();
            rest.get(at.min(rest.len().saturating_sub(1))).map(|r| (r.id.clone(), r.product.clone()))
        })
    }

    /// Moves on to `next`, found before the answered thread left the list.
    pub(crate) fn move_on_to(&mut self, next: Option<(String, String)>, at: f64) {
        match next {
            Some((id, product)) => {
                self.step_to_thread(&id, &product);
                self.answering.arrived = Some((format!("{product}/{id}"), at));
                self.answering.arrived_from = self.answering.pointer;
            }
            None => self.leave_thread(),
        }
    }

    /// A press on one of the open thread's options. Not when the options moved under a still
    /// pointer a moment ago, nor on a thread the app just put under the pointer: the press was
    /// aimed at what stood there before, and a line says so (`pickByClick`, `clickAimedHere`).
    pub(crate) fn click_option(&mut self, at: usize, tx: &mut Tx) {
        if self.answering.strip_moved.get().is_some_and(|(_, when)| (self.at - when) * 1000.0 < rules::PICK_SETTLES_MS) {
            self.answering.strip_moved.set(None);
            self.tell(rules::THE_OPTIONS_MOVED, tx);
            return;
        }
        let here = self.open_item().map(|i| format!("{}/{}", i.product, i.id));
        if let Some((on, when)) = self.answering.arrived.clone().filter(|(on, _)| Some(on) == here.as_ref()) {
            let since = (self.at - when) * 1000.0;
            let still = match (self.answering.arrived_from, self.answering.pointer) {
                (Some((x0, y0)), Some((x, y))) => since < rules::STILL_POINTER_MS && (x - x0).hypot(y - y0) <= rules::MOVED_PX,
                _ => false,
            };
            if since < rules::PICK_SETTLES_MS || still {
                let _ = on;
                self.answering.arrived_from = None;
                self.tell(rules::CLICK_FOR_THE_THREAD_YOU_LEFT, tx);
                return;
            }
        }
        self.pick_option(at, tx);
    }

    /* --------------------------- the question before a discard --------------------------- */

    /// An option that throws the change away is not sent: the question stands where the options
    /// stood, nothing lit, on the open thread (`pickOption`, `discardAsk`).
    pub(crate) fn ask_before_discarding(&mut self, item: &Item, choice: Choice) {
        self.option = None;
        // One question at a time: a close's goes (`closeAskWhy`).
        self.leave_the_close_ask();
        self.answering.discard = Some(Discard { thread: key(item), choice, lit: None });
    }

    /// The question up on the open thread, if any.
    pub fn discard_asked(&self) -> Option<&Discard> {
        let here = match &self.view {
            View::Thread { id, product } => format!("{product}/{id}"),
            _ => return None,
        };
        self.answering.discard.as_ref().filter(|d| d.thread == here)
    }

    /// Its answer: 1 sends the option, confirmed; 2 keeps the change, the offer standing.
    pub(crate) fn answer_discard(&mut self, n: usize, tx: &mut Tx) {
        let Some(asked) = self.answering.discard.take() else { return };
        if n != 1 {
            return;
        }
        let Some(item) = self.open_item().cloned() else { return };
        self.send_pick(&item, &asked.choice, tx);
    }

    /// The question's keys while it is up: the numbers answer it, the arrows walk it (down from
    /// none lights the first, up from the first lights none), Enter answers the lit one and does
    /// nothing with none lit, Escape keeps the thread as it was. Every other key keeps its
    /// meaning (`closeAskOwnsKey`).
    pub(crate) fn discard_key(&mut self, e: &rune::KeyEvent, tx: &mut Tx) -> bool {
        use crate::closing::AskKey;
        let Some(asked) = self.discard_asked().cloned() else { return false };
        // A number the question does not have answers nothing, as in the web window.
        match crate::closing::ask_key(e, rules::discard_options().len(), asked.lit) {
            AskKey::NotMine => return false,
            AskKey::Keep => self.answering.discard = None,
            AskKey::Lit(lit) => self.set_discard_lit(lit),
            AskKey::Answer(n) => self.answer_discard(n, tx),
            AskKey::Nothing => {}
        }
        true
    }

    fn set_discard_lit(&mut self, lit: Option<usize>) {
        if let Some(asked) = self.answering.discard.as_mut() {
            asked.lit = lit;
        }
    }

    /* ------------------------------- typing on an open thread ------------------------------- */

    /// What a letter typed on an open thread with no box under the keys means
    /// (`keyOnAnOpenThread`): the thread's own keys (J K R N Z O G S, a number while options
    /// stand) are its shortcuts; E and L are too, unless the screen has just changed, when they
    /// wait a moment for a letter after them, or go to the box when the app put the person here;
    /// any other letter, and every letter for a second after one went there, goes into the box.
    /// True when the key was taken here.
    pub(crate) fn letter_on_an_open_thread(&mut self, e: &rune::KeyEvent, tx: &mut Tx) -> bool {
        let rune::edit::Key::Text(typed) = &e.key else { return false };
        let Some(item) = self.open_item().cloned() else { return false };
        let thread = key(&item);
        let printable = typed.chars().count() == 1 && typed != " ";
        // A letter after E or L held: both go into the box, a word being typed.
        if let Some((held, on, _)) = self.answering.waiting_key.clone() {
            if tx.key_repeat() && held.eq_ignore_ascii_case(typed) {
                return true;
            }
            self.answering.waiting_key = None;
            if typed.chars().count() == 1 && !tx.key_repeat() && on == thread {
                self.type_into_the_box(&format!("{held}{typed}"), tx);
                return true;
            }
        }
        let still_typing = self.answering.typed_at.is_some_and(|at| (self.at - at) * 1000.0 < STILL_TYPING_MS);
        if still_typing && typed.chars().count() == 1 {
            self.type_into_the_box(typed, tx);
            return true;
        }
        if !printable {
            return false;
        }
        let letter = typed.to_lowercase();
        if ["j", "k", "r", "n", "z", "o", "g", "s"].contains(&letter.as_str()) {
            return false;
        }
        let just_changed = self.answering.screen_changed_at.is_some_and(|at| (self.at - at) * 1000.0 < rules::PICK_SETTLES_MS);
        let moved_you = self.answering.arrived.as_ref().is_some_and(|(on, at)| *on == thread && (self.at - at) * 1000.0 < rules::PICK_SETTLES_MS);
        if letter == "e" || letter == "l" {
            if !just_changed {
                return false;
            }
            if moved_you {
                self.type_into_the_box(typed, tx);
            } else {
                self.answering.waiting_key = Some((typed.clone(), thread, self.at));
                tx.wake_after(std::time::Duration::from_millis(WORD_FOLLOWS_MS as u64 + 1));
            }
            return true;
        }
        let has_options = !crate::ship::offer_now(self, &item).is_empty();
        if letter.len() == 1 && letter.chars().all(|c| ('1'..='9').contains(&c)) && has_options {
            return false;
        }
        self.type_into_the_box(typed, tx);
        true
    }

    /// Words typed on the page go after the box's words, and the box opens with the caret at their
    /// end (`typeIntoTheBox`).
    fn type_into_the_box(&mut self, words: &str, tx: &mut Tx) {
        self.answering.typed_at = Some(self.at);
        let text = format!("{}{words}", self.reply.text);
        self.reply = rune::TextEdit::new(text);
        self.open_reply(tx);
    }

    /// E or L held a moment for a word, with none after it: it does what it does
    /// (`actAfterTheWait`). And the screen's changes are noticed for the next key.
    pub(crate) fn tend_keys(&mut self, tx: &mut Tx) {
        let screen = format!("{:?}|{}|{}|{}|{:?}", self.view, self.later.on.is_some(), self.palette.open, self.compose.card, self.settings.open);
        if screen != self.answering.screen {
            self.answering.screen = screen;
            if self.answering.moved_by_key_at.is_none_or(|at| (self.at - at) * 1000.0 > 1500.0) {
                self.answering.screen_changed_at = Some(self.at);
            }
        }
        let Some((held, on, at)) = self.answering.waiting_key.clone() else { return };
        if (self.at - at) * 1000.0 < WORD_FOLLOWS_MS {
            return;
        }
        self.answering.waiting_key = None;
        if self.open_item().is_none_or(|item| key(item) != on) {
            return;
        }
        if held.eq_ignore_ascii_case("e") {
            crate::keys::close_under_the_keys(self, tx);
        } else if let Some(item) = self.open_item().cloned() {
            self.open_later(&item.id, &item.product, tx);
        }
    }

    /* ------------------------------ the waiting agent's warning ------------------------------ */

    /// The first E or L ever on a thread whose agent waits on the person: E is held and the
    /// line says what it costs, L opens its picker with the sentence inside. E's lesson is
    /// learned once the press after it goes ahead on the thread it warned of; E on another such
    /// thread in between is held and warned about too (`pressAgainGoesAhead`). L's is learned as
    /// it is said. Once learned, never again. True when E was held.
    pub(crate) fn warns_first(&mut self, item: &Item, closing: bool, tx: &mut Tx) -> bool {
        if self.answering.warned || !rules::stopped_waiting(item) {
            return false;
        }
        if closing && crate::rules::e_and_z::press_again_goes_ahead(self.answering.warned_of.as_deref(), &item.product, &item.id) {
            self.answering.warned_of = None;
            self.answering.warned = true;
            self.keep_drafts();
            return false;
        }
        if closing {
            self.answering.warned_of = Some(format!("{}:{}", item.product, item.id));
            let since = tx.clock();
            let (line, undo) = crate::rules::compose::toast_parts(&rules::stopped_warning(true));
            self.toast = Some(write::Toast { line, undo, since, until: since + crate::rules::compose::TOAST_GOES_MS / 1000.0, goes: Some((item.id.clone(), item.product.clone())), sent: None, warns: true });
            return true;
        }
        self.answering.warned = true;
        self.keep_drafts();
        self.later.warning = Some(rules::stopped_warning(false));
        false
    }
}

/// The window is going: what waits its three seconds is written now, on this thread, before it
/// goes, as the web window writes what it holds when its page goes (`pagehide`, `onWriteHeld`);
/// a reply written so lands, and every box is written down.
pub fn on_close(s: &mut State, _tx: &mut Tx) {
    if let (Some(held), crate::source::Source::Door(door)) = (s.held.take(), &s.source) {
        if let Some((url, token)) = door.now() {
            let mut writes = vec![(held.channel, held.payload.clone())];
            writes.extend(held.then.clone());
            let ok = writes.iter().all(|(channel, payload)| crate::source::call_waiting(&url, &token, channel, payload, std::time::Duration::from_secs(5)).is_ok());
            if let (true, write::Sent::Reply { thread, answer, .. }) = (ok, &held.sent) {
                let (thread, at) = (thread.clone(), answer.in_flight);
                s.in_flight_landed(&thread, at);
            }
        }
    }
    s.keep_drafts();
}

/// What a picture's fixture says was done before it was taken: a reply sent from the open
/// thread's box (`replySent`), the three seconds over (`handedOver`), and keys pressed after
/// (`keysAfter`, each a letter or a key's name).
pub fn picture(s: &mut State, tx: &mut Tx) {
    // Opened from Needs you on the Inbox (`from`), which a reply then moves on from.
    if s.fixture.remove("from").is_some_and(|f| f == "needs") {
        s.history.push(View::Inbox);
        s.inbox_tab = crate::data::Tab::Needs;
    }
    if let Some(words) = s.fixture.remove("replySent").and_then(|w| w.as_str().map(str::to_string)) {
        s.open_reply(tx);
        s.reply = rune::TextEdit::new(words);
        s.send_reply(tx);
    }
    if s.fixture.remove("handedOver").is_some() {
        let sent = s.held.as_ref().map(|h| h.sent.clone());
        s.write_what_waits();
        // And refused by the engine, in its own words (`refusedWith`).
        if let (Some(sent), Some(why)) = (sent, s.fixture.remove("refusedWith").and_then(|w| w.as_str().map(str::to_string))) {
            s.landed(write::Landed { sent, answer: Err(why), snapshot: None }, tx);
        }
    }
    if let Some(keys) = s.fixture.remove("keysAfter").and_then(|k| k.as_array().cloned()) {
        for key in keys.iter().filter_map(serde_json::Value::as_str) {
            let key = match key {
                "Escape" => rune::edit::Key::Escape,
                "Enter" => rune::edit::Key::Enter,
                "ArrowDown" => rune::edit::Key::Down,
                "ArrowUp" => rune::edit::Key::Up,
                letter => rune::edit::Key::Text(letter.to_string()),
            };
            crate::keys::pressed(s, &rune::KeyEvent { key, mods: rune::Mods::default() }, tx);
        }
    }
}

/// Where the page's own controls stood as a menu of the reply box was drawn over them: the
/// agent's options, and Stop. A press outside the menu that lands on one of them goes on to it,
/// as the web window's press does after the menu closes.
#[derive(Clone, Debug, Default)]
pub struct PageMarks {
    options: Vec<[f32; 4]>,
    stop: Option<[f32; 4]>,
}

/// Read as the page is drawn.
pub fn page_marks() -> PageMarks {
    let options = (0..).map_while(|at| rune::memo::window_layout(&format!("option-{at}"))).collect();
    PageMarks { options, stop: rune::memo::window_layout("dock-stop") }
}

impl State {
    /// A press outside an open menu of the box, after the menu closed: on an option it picks it,
    /// on Stop it stops. True when it landed on one.
    pub(crate) fn press_lands_on_the_page(&mut self, at: [f32; 2], marks: &PageMarks, tx: &mut Tx) -> bool {
        let inside = |[x, y, w, h]: [f32; 4]| at[0] >= x && at[0] < x + w && at[1] >= y && at[1] < y + h;
        if let Some(option) = marks.options.iter().position(|rect| inside(*rect)) {
            self.click_option(option, tx);
            return true;
        }
        if marks.stop.is_some_and(inside) {
            if let Some(item) = self.open_item().cloned() {
                self.stop_agent(&item.id, &item.product);
            }
            return true;
        }
        false
    }
}

/* ---------------------------------------- drawn ---------------------------------------- */

/// Stop, where Send is while the box holds nothing and the thread is under way (`.dock-stop`):
/// a filled square of nine points and the word, in the dim ink inside the strong line; under the
/// pointer the text's own ink on the wash inside the faint edge, and 0.88 as bright while
/// pressed. A narrow box keeps the square alone in 32 points.
pub fn stop_button(s: &State, item: &Item, compact: bool) -> El {
    let running = s.run_on(item);
    // `fill: currentColor`: the dim ink at rest, the text's own under the pointer. The mark is
    // nine of a twelve-unit box drawn nine points wide, so the square is 6.75 points.
    let inset = Some(1.125);
    let square = el().size(9.0, 9.0).child(el().pin(inset, inset, inset, inset).fill(dim()).group_opacity(1.0, 0.0)).child(el().pin(inset, inset, inset, inset).fill(ink()).group_opacity(0.0, 1.0));
    let mut button = row()
        .key("dock-stop")
        .h(32.0)
        .items_center()
        .radius(6.0)
        .border(1.0, rule())
        .border_layout()
        .hover_group()
        .hover(crate::feel::wash())
        .cursor(rune::Cursor::Pointer)
        .label(if running { "Stop this agent" } else { "Stop this task" })
        .help(rules::stop_title(running))
        .child(crate::feel::edge_lit(6.0, muted(), true));
    button = if compact {
        button.w(32.0).justify_center().child(square)
    } else {
        button.px(12.0).gap(7.0).child(square).child(pen(label("Stop", 13.0, 500.0, dim()).group_color(ink()), 13.0))
    };
    let (id, product) = (item.id.clone(), item.product.clone());
    // Pressed, 0.88 as bright (`.dock-stop:active { filter: brightness(.88) }`).
    button.press(rgba(0, 0, 0, 0.12)).on_press_tx(move |s: &mut State, tx| {
        // The caret goes back in the box, as the web window's press puts it there.
        s.reply.focused = true;
        tx.focus("reply", false);
        s.stop_agent(&id, &product);
    })
}

/// What a sent message's head says in place of its time, and whether Send now follows it.
pub fn head_words(sending: Sending) -> (&'static str, bool) {
    match sending {
        Sending::Held => ("Sending… press Z to undo", false),
        Sending::OnItsWay => ("Sending…", false),
        Sending::WaitingOnTheStep => ("Waiting for its current step", true),
        Sending::SendingNow => ("Sending now…", false),
    }
}

/// Send now, after "Waiting for its current step" (`.msg-now`): underlined, in the dim ink, the
/// text's own under the pointer.
pub fn send_now() -> El {
    let words = text("Send now").font(TextStyle::new(12.5, 400.0).lh(20.0)).color(dim()).group_color(ink()).decorations(vec![rune::Span::new(0, "Send now".len()).underline(dim())]);
    // A message waiting is only ever drawn in the open conversation.
    row().key("send-now").hover_group().cursor(rune::Cursor::Pointer).child(words).on_press(|s: &mut State| {
        if let Some(item) = s.open_item().cloned() {
            s.send_now(&item);
        }
    })
}

/// The ring of the line about a send, run out by a share in 60 steps: what is left of its circle
/// from the top, clockwise (`.sent-line-ring`, r 8.25, a stroke of 1.5 on the line's track).
fn ring(left: f32) -> &'static rune::Icon {
    static FRAMES: std::sync::OnceLock<Vec<rune::Icon>> = std::sync::OnceLock::new();
    let frames = FRAMES.get_or_init(|| {
        (0..=60)
            .map(|step| {
                let share = step as f32 / 60.0;
                let track = r##"<circle cx="10" cy="10" r="8.25" fill="none" stroke="#464443" stroke-width="1.5"/>"##;
                let arc = if share >= 1.0 {
                    r#"<circle cx="10" cy="10" r="8.25" fill="none" stroke="currentColor" stroke-width="1.5"/>"#.to_string()
                } else if share <= 0.0 {
                    String::new()
                } else {
                    let angle = share * std::f32::consts::TAU;
                    let (x, y) = (10.0 + 8.25 * angle.sin(), 10.0 - 8.25 * angle.cos());
                    format!(r#"<path d="M10 1.75A8.25 8.25 0 {} 1 {x:.3} {y:.3}" fill="none" stroke="currentColor" stroke-width="1.5"/>"#, i32::from(share > 0.5))
                };
                rune::Icon { name: Box::leak(format!("m-sent-ring-{step}").into_boxed_str()), view: 20.0, body: Box::leak(format!("{track}{arc}").into_boxed_str()) }
            })
            .collect()
    });
    &frames[((left.clamp(0.0, 1.0) * 60.0).round() as usize).min(60)]
}

static SENT_X: rune::Icon = rune::Icon { name: "m-sent-x", view: 8.0, body: r#"<path d="M1 1 L7 7 M7 1 L1 7" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>"# };

/// The line in the strip over the thread a send moved the person to (`.ts-top > .sent-line`):
/// a dot of the accent, "Sent to" and the thread it went to, cut short; O Back and Z Undo as
/// keys; and the ring that runs out over five seconds and closes it, × in it to close it now.
/// The pointer on it stops its clock.
pub fn sent_line(s: &State) -> Option<El> {
    let toast = s.toast.as_ref()?;
    let clock = s.answering.sent_line.as_ref().filter(|c| Some(&c.on) == s.sent_line_on().as_ref())?;
    let says = toast.sent.as_deref()?;
    let (id, product) = toast.goes.as_ref()?;
    let title = s.snap.items.iter().find(|i| &i.id == id && &i.product == product).map(crate::rules::title::row_title).unwrap_or_else(|| "the thread you left".to_string());
    let key = |cap: &str, word: &str| {
        row().items_center().gap(6.0).hover_group().cursor(rune::Cursor::Pointer)
            .child(row().h(17.0).px(4.0).items_center().border(1.0, rule()).border_layout().child(text(cap).font(TextStyle::new(10.5, 600.0).lh(15.0)).color(muted()).group_color(ink())).child(crate::feel::edge_lit(0.0, muted(), true)))
            .child(pen(text(word).font(TextStyle::new(12.5, 400.0).lh(20.0)).color(muted()).group_color(ink()), 12.5))
    };
    let back = key("O", "Back").on_press(|s: &mut State| {
        s.open_what_the_line_names();
    });
    let undo = toast.undo.then(|| {
        key("Z", "Undo").on_press_tx(|s: &mut State, tx| {
            s.toast = None;
            s.undo(tx);
        })
    });
    let left = (clock.left / rules::SENT_LINE_MS) as f32;
    let close = el()
        .size(24.0, 24.0)
        .ml(2.0)
        .hover_group()
        .cursor(rune::Cursor::Pointer)
        .label("Close")
        .child(icon_at(ring(left), 20.0, 2.0, 2.0).color(muted()))
        .child(icon_at(&SENT_X, 8.0, 8.0, 8.0).color(muted()).group_color(ink()))
        .on_press(|s: &mut State| {
            s.toast = None;
        });
    let said = row().shrink().items_center().child(pen(text(says).font(TextStyle::new(12.5, 400.0).lh(20.0)).color(dim()), 12.5)).child(row().shrink().child(pen(text(format!(" {title}")).font(TextStyle::new(12.5, 400.0).lh(20.0)).color(ink()).truncate(), 12.5)));
    Some(
        row()
            .key("sent-line")
            .max_w(460.0)
            .min_w(0.0)
            .h(26.0)
            .items_center()
            .gap(10.0)
            .pt(2.0)
            .pr(4.0)
            .pb(2.0)
            .pl(12.0)
            .border(1.0, edge())
            .border_layout()
            .on_hover_change(|s: &mut State, over, _| {
                if let Some(clock) = s.answering.sent_line.as_mut() {
                    clock.hovered = over;
                }
            })
            .child(el().size(6.0, 6.0).radius(3.0).fill(accent()))
            .child(said)
            .child(row().gap(14.0).items_center().child(back).children(undo))
            .child(close),
    )
}

/// An icon `size` points wide, pinned at `(x, y)` inside its box.
fn icon_at(i: &'static rune::Icon, size: f32, x: f32, y: f32) -> El {
    rune::icon(i, size).pin(Some(x), Some(y), None, None)
}

/// The question before throwing the change away, where the options stood (`OptionsOffer` with
/// `discardAsk`): its heading, and its two answers numbered and pressable.
pub fn discard_strip(s: &State, column: f32) -> Option<El> {
    let asked = s.discard_asked()?;
    // The strip the close question stands in too (`closing.rs`).
    Some(crate::closing::ask_strip("discard", &crate::rules::close_asks::discard_ask(), asked.lit, None, column, |s, n, tx| s.answer_discard(n, tx)))
}

#[cfg(test)]
mod tests {
    use rune::{Key, KeyEvent, Mods, Runtime};
    use serde_json::{Value, json};

    use super::*;
    use crate::feel::tests::{key, press, run, window, window_dressed};
    use crate::rules::writes::ANSWER;
    use crate::write::{Landed, Sent};

    /// The invented inbox's first thread with an agent on it, its reply box opened.
    fn working(name: &str) -> Runtime<State> {
        let mut rt = window_dressed(name, &["--thread", "w-feel0"], "/fixtures/storefront", "Done.", |fixture| {
            let now = fixture["now"].as_f64().unwrap();
            fixture["snapshot"]["items"][0]["status"] = json!("claimed");
            fixture["snapshot"]["supervisor"]["running"] = json!([{ "itemId": "w-feel0", "product": "storefront", "startedAt": now - 300_000.0, "engine": "claude-code", "tail": [], "activity": [] }]);
        });
        key(&mut rt, Key::Text("r".into()));
        rt
    }

    /// The first thread, waiting on the person with three options, one of which throws the
    /// change away.
    fn asking(name: &str, args: &[&str]) -> Runtime<State> {
        window_dressed(name, args, "/fixtures/storefront", "Which way?\n\n1. Ship it\n2. Discard the change\n3. Ask me first", |fixture| {
            for item in fixture["snapshot"]["items"].as_array_mut().unwrap() {
                item["status"] = json!("blocked");
            }
        })
    }

    fn cmd(key: Key) -> KeyEvent {
        KeyEvent { key, mods: Mods { cmd: true, ..Mods::default() } }
    }

    fn typed(rt: &mut Runtime<State>, words: &str) {
        rt.update(|s, _| s.reply.insert(words));
        rt.frame(1.0 / 60.0);
    }

    fn sent_line(rt: &Runtime<State>) -> Option<String> {
        rt.state.toast.as_ref().map(|t| t.line.clone())
    }

    /// What the engine says to what is on its way, for a window that has nobody to send to.
    fn lands(rt: &mut Runtime<State>, sent: Sent, answer: Result<Value, String>) {
        rt.update(|s, tx| s.landed(Landed { sent, answer, snapshot: None }, tx));
        rt.frame(1.0 / 60.0);
    }

    #[test]
    fn stop_stands_where_send_is_while_the_box_is_empty_and_writes_the_stop_after_what_is_held() {
        let mut rt = working("stop");
        assert!(rt.find("dock-stop").is_some(), "Stop in Send's place");
        assert!(rt.find(crate::reply::SEND).is_none());
        // The first thing typed makes it Send again, and emptying the box makes it Stop.
        typed(&mut rt, "w");
        assert!(rt.find("dock-stop").is_none() && rt.find(crate::reply::SEND).is_some());
        rt.update(|s, _| s.reply = rune::TextEdit::new(""));
        rt.frame(1.0 / 60.0);
        let (x, y) = rt.find("dock-stop").expect("Stop again");
        // A press stops it, the caret back in the box.
        press(&mut rt, x, y);
        assert_eq!(rt.state.tried, vec![(rules::STOP, json!({ "product": "storefront", "id": "w-feel0" }))]);
        assert_eq!(rt.state.answering.following, Some(("w-feel0".to_string(), "storefront".to_string())));
        // Said once the engine has it, by whether a run was up.
        lands(&mut rt, Sent::Stop { thread: "storefront/w-feel0".to_string(), running: true }, Ok(json!({ "id": "w-feel0", "status": "blocked" })));
        assert_eq!(sent_line(&rt).as_deref(), Some("Agent stopped. Back in your inbox. Reply to redirect it."));
        // Nothing went on Z's pile.
        assert!(rt.state.undos.is_empty());
    }

    #[test]
    fn the_command_bar_stops_a_thread_under_way_and_says_whether_anything_had_started() {
        let mut rt = working("stop-bar");
        key(&mut rt, Key::Escape);
        rt.key(cmd(Key::Command('k')));
        rt.frame(1.0 / 60.0);
        rt.update(|s, _| s.palette.words = rune::TextEdit::new("stop"));
        rt.frame(1.0 / 60.0);
        assert!(rt.elements().iter().any(|e| e.text.as_deref() == Some("Stop Agent (Moves to Inbox)")), "the row, with a run up");
        rt.key(KeyEvent { key: Key::Enter, mods: Mods::default() });
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.tried.last(), Some(&(rules::STOP, json!({ "product": "storefront", "id": "w-feel0" }))));
        // With nothing started on it, the row says so in its hint.
        assert_eq!(crate::rules::palette::stop_row(false, ("w".into(), "p".into())).hint.as_deref(), Some("nothing has started on it yet"));
    }

    #[test]
    fn a_reply_still_held_is_written_before_the_stop() {
        let mut rt = working("stop-held");
        typed(&mut rt, "Also check the refunds.");
        rt.key(cmd(Key::Enter));
        rt.frame(1.0 / 60.0);
        assert!(rt.state.held.is_some(), "held its three seconds");
        rt.update(|s, _| s.stop_agent("w-feel0", "storefront"));
        let channels: Vec<&str> = rt.state.tried.iter().map(|(c, _)| *c).collect();
        assert_eq!(channels, [ANSWER, rules::STOP]);
        // A reply to a run carries no status: the run is going.
        assert_eq!(rt.state.tried[0].1, json!({ "product": "storefront", "id": "w-feel0", "answer": "Also check the refunds.", "permissionMode": null }));
    }

    #[test]
    fn a_reply_to_a_run_is_drawn_at_once_says_where_it_is_and_cannot_be_taken_back_once_handed_over() {
        let mut rt = working("sending-run");
        typed(&mut rt, "Also check the refunds.");
        rt.key(cmd(Key::Enter));
        rt.frame(1.0 / 60.0);
        // On the page from the press, its head saying Z still takes it back, and no line in the corner.
        let heads = |rt: &Runtime<State>, said: &str| rt.elements().iter().any(|e| e.text.as_deref() == Some(said));
        assert!(heads(&rt, "Sending… press Z to undo"));
        assert!(rt.elements().iter().any(|e| e.text.as_deref() == Some("Also check the refunds.")));
        assert!(rt.state.toast.is_none());
        // Three seconds on it is handed over: waiting on the step a Claude Code run is in.
        rt.update(|s, _| s.write_what_waits());
        rt.frame(1.0 / 60.0);
        assert!(heads(&rt, "Waiting for its current step") && heads(&rt, "Send now"));
        // Send now cuts the step.
        let (x, y) = rt.find("send-now").expect("Send now");
        press(&mut rt, x, y);
        assert_eq!(rt.state.tried.last().map(|(c, _)| *c), Some("zero:send-now"));
        assert!(heads(&rt, "Sending now…"));
        // Z cannot take it back: it goes back to the conversation, and writes nothing.
        let before = rt.state.tried.len();
        key(&mut rt, Key::Escape);
        key(&mut rt, Key::Text("z".into()));
        assert_eq!(sent_line(&rt).as_deref(), Some(rules::ALREADY_WITH_THE_AGENT));
        assert_eq!(rt.state.tried.len(), before);
    }

    #[test]
    fn a_reply_to_a_thread_at_rest_is_held_then_written_and_z_takes_it_back_and_stops_its_run() {
        let mut rt = window("reply-z", &["--thread"]);
        rt.state.pretend_sent = true;
        key(&mut rt, Key::Text("r".into()));
        typed(&mut rt, "Go on.");
        rt.key(cmd(Key::Enter));
        rt.frame(1.0 / 60.0);
        let item = rt.state.open_item().cloned().unwrap();
        let thread = format!("{}/{}", item.product, item.id);
        // Said with the thread named, and the box folded.
        assert_eq!(sent_line(&rt).as_deref(), Some("Sent → Storefront"));
        assert!(rt.state.toast.as_ref().is_some_and(|t| t.undo && t.goes.is_some()));
        assert!(!rt.state.replying);
        // Out of Needs you while it is held.
        assert!(!rt.state.lists.needs.iter().any(|r| r.id == item.id));
        assert!(rt.state.lists.progress.iter().any(|r| r.id == item.id));
        // Written: the answer, which opens the finished thread.
        let held = rt.state.held.take().unwrap();
        let Sent::Reply { answer, words, pick, files, .. } = held.sent.clone() else { panic!("a reply") };
        assert_eq!(held.payload, json!({ "product": "storefront", "id": item.id, "answer": "Go on.", "status": "open", "permissionMode": null }));
        rt.update(|s, _| s.held = Some(held));
        rt.update(|s, _| s.write_what_waits());
        lands(&mut rt, Sent::Reply { thread: thread.clone(), words, pick, files, answer }, Ok(json!({ "id": item.id })));
        assert_eq!(rt.state.undos.last().map(|u| u.undoes.as_str()), Some(rules::REPLY_UNDOES));
        // Z: the run it started stopped, then the answer withdrawn with the status it found.
        rt.state.tried.clear();
        key(&mut rt, Key::Text("z".into()));
        assert_eq!(rt.state.tried, rules::withdraw(&item, true));
        // Once the engine has it: the thread's line, the words back in the box, the line said.
        lands(&mut rt, Sent::TakenBack { thread: thread.clone() }, Ok(json!({ "id": item.id })));
        assert_eq!(rt.state.answering.marks.last().map(|m| m.words.as_str()), Some(rules::UNDID_REPLY));
        assert_eq!(rt.state.reply.text, "Go on.");
        assert!(rt.state.replying);
        assert_eq!(sent_line(&rt).as_deref(), Some("Reply withdrawn, back in your inbox · your message is back in the reply box"));
        // The thread draws the Z as the person's own line.
        assert!(rt.elements().iter().any(|e| e.text.as_deref() == Some(rules::UNDID_REPLY)));
    }

    #[test]
    fn z_past_half_a_minute_asks_first_and_any_other_key_says_no() {
        let mut rt = window("z-ask", &["--thread"]);
        rt.state.pretend_sent = true;
        let item = rt.state.open_item().cloned().unwrap();
        let mut old = write::Undo::plain(rules::REPLY_WITHDRAWN.to_string(), rules::REPLY_UNDOES.to_string(), rules::withdraw(&item, true), Sent::TakenBack { thread: "storefront/x".to_string() }, (item.id.clone(), item.product.clone()), false);
        old.at = write::wall_clock() - 60_000.0;
        rt.state.undos.push(old);
        key(&mut rt, Key::Text("z".into()));
        assert_eq!(sent_line(&rt).as_deref(), Some("Press Z again to take back that reply and stop the agent."));
        assert!(rt.state.tried.is_empty() && rt.state.undos.len() == 1);
        // Another key answers no; the next Z asks again.
        key(&mut rt, Key::Down);
        assert!(rt.state.answering.asked.is_none());
        run(&mut rt, 0.3);
        key(&mut rt, Key::Text("z".into()));
        assert!(rt.state.tried.is_empty() && rt.state.answering.asked.is_some());
        // A Z straight after it is the same press said twice, and answers nothing
        // (`tests/e-and-z-always-say-what-they-did.test.mts`: two quick Zs are one Z).
        key(&mut rt, Key::Text("z".into()));
        assert!(rt.state.tried.is_empty() && rt.state.answering.asked.is_some(), "still asking");
        // A second Z inside eight seconds, and past that moment, does it.
        run(&mut rt, 0.3);
        key(&mut rt, Key::Text("z".into()));
        assert_eq!(rt.state.tried, rules::withdraw(&item, true));
        // A refusal puts it back on the pile and says why.
        lands(&mut rt, Sent::TakenBack { thread: "storefront/x".to_string() }, Err("connection refused".to_string()));
        assert_eq!(sent_line(&rt).as_deref(), Some("Could not undo that. connection refused"));
        assert_eq!(rt.state.undos.len(), 1);
        // With nothing at all on the pile, Z says so.
        rt.state.undos.clear();
        run(&mut rt, 0.3);
        key(&mut rt, Key::Text("z".into()));
        assert_eq!(sent_line(&rt).as_deref(), Some("Nothing to undo yet."));
    }

    #[test]
    fn a_picked_option_is_the_line_its_written_copy_will_be_and_z_takes_it_back() {
        let mut rt = asking("pick", &["--thread"]);
        rt.state.pretend_sent = true;
        key(&mut rt, Key::Text("1".into()));
        let item = rt.state.open_item().cloned().unwrap();
        // Drawn at once as the pick it is, and said as what went where.
        assert!(rt.elements().iter().any(|e| e.text.as_deref() == Some("Picked")));
        assert!(rt.elements().iter().any(|e| e.text.as_deref() == Some(" Ship it")));
        assert_eq!(sent_line(&rt).as_deref(), Some(format!("Sent “Ship it” to {}", item.title.clone().unwrap()).as_str()));
        let held = rt.state.held.as_ref().unwrap();
        assert_eq!(held.payload, json!({ "product": "storefront", "id": item.id, "answer": "Option 1: Ship it", "status": "open" }));
        let Sent::Pick { answer, .. } = held.sent.clone() else { panic!("a pick") };
        rt.update(|s, _| s.write_what_waits());
        lands(&mut rt, Sent::Pick { thread: format!("storefront/{}", item.id), answer }, Ok(json!({ "id": item.id })));
        rt.state.tried.clear();
        key(&mut rt, Key::Text("z".into()));
        assert_eq!(rt.state.tried, rules::withdraw(&item, false));
        lands(&mut rt, Sent::TakenBack { thread: format!("storefront/{}", item.id) }, Ok(json!({})));
        assert_eq!(sent_line(&rt).as_deref(), Some("Option 1 withdrawn, back in your inbox"));
        let mark = rt.state.answering.marks.last().unwrap();
        assert_eq!((mark.words.as_str(), mark.choice.as_deref()), (rules::UNDID_PICKING, Some("Ship it")));
    }

    #[test]
    fn z_inside_the_three_seconds_takes_a_pick_back_whole_and_leaves_its_line() {
        let mut rt = asking("pick-grace", &["--thread"]);
        key(&mut rt, Key::Text("1".into()));
        key(&mut rt, Key::Text("z".into()));
        assert!(rt.state.held.is_none() && rt.state.tried.is_empty());
        assert!(rt.state.answering.sending.is_empty(), "its drawn copy went with it");
        assert_eq!(sent_line(&rt).as_deref(), Some("Undone. Nothing changed."));
        assert_eq!(rt.state.answering.marks.last().map(|m| m.words.as_str()), Some(rules::UNDID_PICKING));
    }

    #[test]
    fn an_option_that_throws_the_change_away_asks_first() {
        let mut rt = asking("discard", &["--thread"]);
        key(&mut rt, Key::Text("2".into()));
        assert!(rt.state.held.is_none(), "nothing sent");
        assert!(rt.state.discard_asked().is_some());
        assert!(rt.elements().iter().any(|e| e.text.as_deref() == Some("Discard the change?")));
        // Escape keeps the thread as it was.
        key(&mut rt, Key::Escape);
        assert!(rt.state.discard_asked().is_none() && matches!(rt.state.view, View::Thread { .. }));
        // Asked again: down lights the first answer, up from it lights none, Enter with none does nothing.
        key(&mut rt, Key::Text("2".into()));
        key(&mut rt, Key::Down);
        assert_eq!(rt.state.discard_asked().and_then(|d| d.lit), Some(0));
        key(&mut rt, Key::Up);
        assert_eq!(rt.state.discard_asked().and_then(|d| d.lit), None);
        key(&mut rt, Key::Enter);
        assert!(rt.state.discard_asked().is_some() && rt.state.held.is_none());
        // 2 keeps the change; 1 sends the option, confirmed.
        key(&mut rt, Key::Text("2".into()));
        assert!(rt.state.discard_asked().is_none() && rt.state.held.is_none());
        key(&mut rt, Key::Text("2".into()));
        key(&mut rt, Key::Text("1".into()));
        assert_eq!(rt.state.held.as_ref().map(|h| h.payload["answer"].clone()), Some(json!("Option 2: Discard the change")));
    }

    #[test]
    fn the_first_close_ever_of_a_waiting_agent_is_held_and_says_what_it_costs() {
        let mut rt = asking("warned", &["--thread"]);
        // Past the moment in which E, just after the screen changed, waits for a word.
        run(&mut rt, 1.0);
        key(&mut rt, Key::Text("e".into()));
        assert!(rt.state.held.is_none(), "the press did not land");
        assert_eq!(sent_line(&rt).as_deref(), Some("This agent has answered and waits on you, so marking it done leaves it stuck. Press again to go ahead."));
        assert!(rt.state.toast.as_ref().is_some_and(|t| t.warns), "and its line is read whole, not cut");
        // Not learned for being said: only once a press went ahead on the thread it was about.
        assert!(!rt.state.answering.warned);
        key(&mut rt, Key::Text("e".into()));
        assert!(rt.state.held.as_ref().is_some_and(|h| h.closes.is_some()), "the second press closes it");
        assert!(rt.state.answering.warned, "and now it is learned");
    }

    // `tests/e-and-z-always-say-what-they-did.test.mts`: E warned that one thread's agent waited,
    // and the next E closed a different thread with no warning.
    #[test]
    fn the_press_after_the_warning_goes_ahead_only_on_the_thread_it_warned_about() {
        let mut rt = asking("warned-other", &["--thread"]);
        run(&mut rt, 1.0);
        let (id, product) = rt.state.open_item().map(|i| (i.id.clone(), i.product.clone())).unwrap();
        key(&mut rt, Key::Text("e".into()));
        assert!(rt.state.held.is_none(), "held and warned");
        assert_eq!(rt.state.answering.warned_of, Some(format!("{product}:{id}")));
        // The warning was about another thread: this one's E is held and warned about too.
        rt.state.answering.warned_of = Some(format!("{product}:w-another"));
        key(&mut rt, Key::Text("e".into()));
        assert!(rt.state.held.is_none(), "not closed on the strength of another thread's warning");
        assert!(!rt.state.answering.warned);
        assert_eq!(rt.state.answering.warned_of, Some(format!("{product}:{id}")), "it now warns of this one");
        key(&mut rt, Key::Text("e".into()));
        assert!(rt.state.held.as_ref().is_some_and(|h| h.closes.is_some()), "and goes ahead on it");
    }

    #[test]
    fn the_warning_given_on_l_is_the_same_once_and_opens_the_picker_with_it() {
        let mut rt = asking("warned-later", &["--thread"]);
        run(&mut rt, 1.0);
        key(&mut rt, Key::Text("l".into()));
        assert!(rt.state.later.on.is_some());
        assert_eq!(rt.state.later.warning.as_deref(), Some("This agent has answered and waits on you, so putting it off leaves it stuck."));
        key(&mut rt, Key::Escape);
        run(&mut rt, 1.0);
        // Said once for both keys.
        key(&mut rt, Key::Text("e".into()));
        assert!(rt.state.held.is_some());
    }

    #[test]
    fn a_reply_from_needs_you_moves_on_and_one_from_elsewhere_stays_and_follows_the_thread() {
        let mut rt = asking("moves-on", &["--view", "inbox"]);
        key(&mut rt, Key::Enter);
        let first = rt.state.open_item().map(|i| i.id.clone()).unwrap();
        key(&mut rt, Key::Text("r".into()));
        typed(&mut rt, "Go on.");
        rt.key(cmd(Key::Enter));
        rt.frame(1.0 / 60.0);
        let now = rt.state.open_item().map(|i| i.id.clone()).unwrap();
        assert_ne!(now, first, "on to the next thread waiting");
        // The number pressed in the next moment was meant for the thread before.
        key(&mut rt, Key::Text("1".into()));
        assert_eq!(sent_line(&rt).as_deref(), Some(rules::AIMED_AT_THE_THREAD_YOU_LEFT));
        // Opened from Home, a reply stays, and Escape then goes to Needs you.
        let mut rt = window("stays", &[]);
        let row = rt.state.lists.needs[1].clone();
        rt.update(|s, _| s.open_thread(&row.id, &row.product));
        key(&mut rt, Key::Text("r".into()));
        typed(&mut rt, "Go on.");
        rt.key(cmd(Key::Enter));
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.open_item().map(|i| i.id.clone()), Some(row.id.clone()));
        assert_eq!(rt.state.answering.following, Some((row.id.clone(), row.product.clone())));
        key(&mut rt, Key::Escape);
        assert_eq!((rt.state.view.clone(), rt.state.inbox_tab), (View::Inbox, crate::data::Tab::Needs));
    }

    #[test]
    fn a_letter_typed_on_an_open_thread_goes_into_its_box_and_its_own_keys_stay_its_own() {
        let mut rt = asking("typed", &["--thread"]);
        run(&mut rt, 1.0);
        key(&mut rt, Key::Text("W".into()));
        assert!(rt.state.replying, "the box opened");
        assert_eq!(rt.state.reply.text, "W");
        // The thread's own keys are not typed: 1 picks while options stand.
        let mut rt = asking("typed-keys", &["--thread"]);
        run(&mut rt, 1.0);
        key(&mut rt, Key::Text("1".into()));
        assert!(rt.state.held.is_some() && rt.state.reply.text.is_empty());
        // Right after the screen changed, E waits a moment: a letter after it types both.
        let mut rt = asking("typed-e", &["--thread"]);
        key(&mut rt, Key::Text("e".into()));
        assert!(rt.state.held.is_none() && rt.state.answering.waiting_key.is_some());
        key(&mut rt, Key::Text("x".into()));
        assert_eq!(rt.state.reply.text, "ex");
        // With none after it, it does what it does once the moment is over.
        let mut rt = asking("typed-e-alone", &["--thread"]);
        rt.state.answering.warned = true;
        key(&mut rt, Key::Text("e".into()));
        run(&mut rt, 0.5);
        assert!(rt.state.held.as_ref().is_some_and(|h| h.closes.is_some()), "E closed it after the wait");
    }

    #[test]
    fn a_press_on_nothing_folds_an_empty_box_and_a_draft_holds_it_open() {
        let mut rt = window("fold-away", &["--thread"]);
        run(&mut rt, 1.0);
        key(&mut rt, Key::Text("r".into()));
        assert!(rt.state.replying);
        // Far right of the column, on the page and on nothing.
        press(&mut rt, 1180.0, 300.0);
        assert!(!rt.state.replying, "folded");
        key(&mut rt, Key::Text("r".into()));
        typed(&mut rt, "Half a thought");
        press(&mut rt, 1180.0, 300.0);
        assert!(rt.state.replying, "a draft holds it open");
    }

    #[test]
    fn a_click_on_a_thread_the_app_just_put_under_a_still_pointer_is_not_taken() {
        let mut rt = asking("click-guard", &["--thread"]);
        let thread = rt.state.open_item().map(|i| format!("{}/{}", i.product, i.id)).unwrap();
        // The app put the person here a moment ago, the pointer where it was then.
        rt.state.answering.pointer = Some((600.0, 600.0));
        rt.update(|s, _| {
            s.answering.arrived = Some((thread.clone(), s.at));
            s.answering.arrived_from = Some((600.0, 600.0));
        });
        rt.update(|s, tx| s.click_option(0, tx));
        assert_eq!(sent_line(&rt).as_deref(), Some(rules::CLICK_FOR_THE_THREAD_YOU_LEFT));
        assert!(rt.state.held.is_none());
        // Inside the next 0.8 s it is still not taken; past them, the next click is the person's own.
        rt.update(|s, tx| s.click_option(0, tx));
        assert!(rt.state.held.is_none());
        run(&mut rt, 1.0);
        rt.update(|s, tx| s.click_option(0, tx));
        assert!(rt.state.held.is_some());
    }

    #[test]
    fn a_reply_a_restart_lost_comes_back_to_its_box_and_one_that_landed_is_let_go() {
        let mut rt = window("unsent", &[]);
        let row = rt.state.lists.needs[0].clone();
        let thread = format!("{}/{}", row.product, row.id);
        let lost = crate::drafts::InFlight { words: "Never landed".to_string(), body: "Never landed".to_string(), at: 1.0, files: Vec::new() };
        let landed = crate::drafts::InFlight { words: "Go on.".to_string(), body: "Go on.".to_string(), at: 2.0, files: Vec::new() };
        rt.state.answering.in_flight.insert(thread.clone(), vec![lost.clone(), landed]);
        // The looks found the second in the thread and never the first.
        rt.state.answering.unsent.lock().unwrap().push((thread.clone(), vec![2.0], vec![lost]));
        rt.update(|s, tx| {
            s.tend_unsent(tx);
        });
        assert!(rt.state.answering.in_flight.is_empty(), "both let go");
        assert_eq!(rt.state.drafts.get(&thread).map(String::as_str), Some("Never landed"));
        assert_eq!(sent_line(&rt).as_deref(), Some("Your reply didn’t send. It’s back in the box."));
        assert!(rt.state.toast.as_ref().is_some_and(|t| t.goes == Some((row.id.clone(), row.product.clone()))));
    }

    #[test]
    fn a_reply_sent_is_written_down_until_it_lands() {
        let mut rt = window("in-flight", &["--thread"]);
        key(&mut rt, Key::Text("r".into()));
        typed(&mut rt, "Go on.");
        rt.key(cmd(Key::Enter));
        rt.frame(1.0 / 60.0);
        let thread = rt.state.open_item().map(|i| format!("{}/{}", i.product, i.id)).unwrap();
        assert_eq!(rt.state.answering.in_flight.get(&thread).map(|l| l[0].words.clone()), Some("Go on.".to_string()));
        // Taken back inside the hold, it is no longer on its way.
        key(&mut rt, Key::Text("z".into()));
        assert!(rt.state.answering.in_flight.is_empty());
    }

    #[test]
    fn a_refused_reply_comes_back_after_what_the_box_holds_now() {
        let mut rt = window("refused", &["--thread"]);
        key(&mut rt, Key::Text("r".into()));
        typed(&mut rt, "First thought.");
        rt.key(cmd(Key::Enter));
        rt.frame(1.0 / 60.0);
        let held = rt.state.held.take().unwrap();
        key(&mut rt, Key::Text("r".into()));
        typed(&mut rt, "Second thought.");
        lands(&mut rt, held.sent, Err("connection refused".to_string()));
        assert_eq!(rt.state.reply.text, "Second thought.\n\nFirst thought.");
        assert!(rt.state.replying && rt.state.answering.sending.is_empty());
        assert_eq!(sent_line(&rt).as_deref(), Some("connection refused"));
    }

    #[test]
    fn drawn_copies_let_go_once_their_written_copy_has_stood_a_while() {
        let mut rt = window("landed", &["--thread"]);
        let thread = rt.state.open_item().map(|i| format!("{}/{}", i.product, i.id)).unwrap();
        let at = rt.state.now;
        let id = rt.state.answering.draw(&thread, at, "Go on.");
        rt.update(|s, _| s.drawn_landed(id));
        run(&mut rt, 0.1);
        assert_eq!(rt.state.answering.sending.len(), 1);
        // The fixture's clock stands still; the window's own moves on.
        rt.update(|s, _| s.now = at + rules::LANDED_MS + 1.0);
        rt.update(|s, tx| {
            s.tend_answering(tx);
        });
        assert!(rt.state.answering.sending.is_empty());
    }
}
