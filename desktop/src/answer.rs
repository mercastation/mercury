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
    /// The option strip's top while the pointer is on it, and when it last moved under a still
    /// pointer, on the window's clock (`stripUnderThePointer`).
    pub strip: Option<f32>,
    pub strip_moved_at: Option<f64>,
    /// Whether the once-ever warning about a waiting agent has been given (`zero.stoppedWarned`).
    pub warned: bool,
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
}

/// The kept marks: the newest 300.
const MAX_UNDO_MARKS: usize = 300;

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
        before != self.answering.sending.len()
    }

    /// Whether a reply that waits on the step its agent is in can be sent now: a run is up on
    /// the thread, on Claude Code (`cutsARunningStep`).
    pub fn waits_on_the_step(&self, item: &Item) -> bool {
        self.snap.supervisor.running.iter().any(|r| r.item_id == item.id && r.product == item.product && r.rest.get("engine").and_then(serde_json::Value::as_str).is_none_or(|e| e == "claude-code" || e == "claude"))
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
        if !self.send_all(writes, sent) {
            // Not sent, so not undone: Z still has it.
            if let Some(back) = self.answering.undoing.take() {
                self.undos.push(back);
            }
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
            }
            None => self.leave_thread(),
        }
    }

    /* --------------------------- the question before a discard --------------------------- */

    /// An option that throws the change away is not sent: the question stands where the options
    /// stood, nothing lit, on the open thread (`pickOption`, `discardAsk`).
    pub(crate) fn ask_before_discarding(&mut self, item: &Item, choice: Choice) {
        self.option = None;
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
        use rune::edit::Key;
        let Some(asked) = self.discard_asked().cloned() else { return false };
        let count = rules::discard_options().len();
        match &e.key {
            Key::Escape => self.answering.discard = None,
            Key::Down => self.set_discard_lit(Some(asked.lit.map_or(0, |at| (at + 1).min(count - 1)))),
            Key::Up => self.set_discard_lit(asked.lit.and_then(|at| at.checked_sub(1))),
            Key::Enter => {
                if let Some(at) = asked.lit {
                    self.answer_discard(at + 1, tx);
                }
            }
            Key::Text(t) if t.len() == 1 && t.chars().all(|c| ('1'..='9').contains(&c)) => {
                let n: usize = t.parse().unwrap_or(0);
                if n <= count {
                    self.answer_discard(n, tx);
                } else {
                    // A number the question does not have answers nothing, as in the web window.
                    self.answering.discard = Some(asked);
                }
            }
            _ => return false,
        }
        true
    }

    fn set_discard_lit(&mut self, lit: Option<usize>) {
        if let Some(asked) = self.answering.discard.as_mut() {
            asked.lit = lit;
        }
    }

    /* ------------------------------ the waiting agent's warning ------------------------------ */

    /// The first E or L ever on a thread whose agent waits on the person: E is held and the
    /// line says what it costs, L opens its picker with the sentence inside. Once said, never
    /// again. True when E was held.
    pub(crate) fn warns_first(&mut self, item: &Item, closing: bool, tx: &mut Tx) -> bool {
        if self.answering.warned || !rules::stopped_waiting(item) {
            return false;
        }
        self.answering.warned = true;
        self.keep_drafts();
        if closing {
            let since = tx.clock();
            let (line, undo) = crate::rules::compose::toast_parts(&rules::stopped_warning(true));
            self.toast = Some(write::Toast { line, undo, since, until: since + crate::rules::compose::TOAST_GOES_MS / 1000.0, goes: Some((item.id.clone(), item.product.clone())), sent: None });
            return true;
        }
        self.later.warning = Some(rules::stopped_warning(false));
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

/// The line over the reply box of the thread a send moved the person to (`SentLine`): drawn
/// once its place on the page is measured (`docs/parity-answering.md`, P9).
pub fn sent_line(_s: &State) -> Option<El> {
    None
}

/// The question before throwing the change away, where the options stood (`OptionsOffer` with
/// `discardAsk`): its heading, and its two answers numbered and pressable.
pub fn discard_strip(s: &State, column: f32) -> Option<El> {
    let asked = s.discard_asked()?;
    let options = rules::discard_options();
    let rows = options.iter().enumerate().map(|(at, c)| {
        let lit = asked.lit == Some(at);
        let tone = if lit { ink() } else { dim() };
        let number = row().size(20.0, 20.0).radius(6.0).center().border(1.0, if lit { muted() } else { rule() }).child(text(format!("{}", c.n)).font(mono(11.0, 11.0)).color(tone));
        let mut line = row()
            .key(format!("discard-{at}"))
            .pt(if at == 0 { 7.0 } else { 3.0 })
            .pb(if at + 1 == options.len() { 7.0 } else { 3.0 })
            .pl(14.0)
            .pr(44.0)
            .gap(10.0)
            .cursor(rune::Cursor::Pointer)
            .on_press_tx(move |s: &mut State, tx| s.answer_discard(at + 1, tx))
            .child(number)
            .child(pen(text(c.text.as_str()).font(TextStyle::new(14.5, 400.0).lh(20.0)).color(tone).group_color(ink()), 14.5));
        line = if lit { line.fill(white(0.08)) } else { line.hover_group().hover(crate::feel::wash()) };
        let aside = if lit { Some("↵ send") } else if c.recommended { Some("recommended") } else { None };
        match aside {
            Some(said) => line.child(el().pl(8.0).mt(1.0).child(pen(text(said).font(TextStyle::new(11.5, 400.0).lh(20.0)).color(muted()), 11.5))),
            None => line,
        }
    });
    let head = el().pb(8.0).child(pen(text(rules::DISCARD_ASK).font(TextStyle::new(14.5, 500.0).lh(20.0)).color(ink()), 14.5));
    Some(rune::col().w(column).child(head).child(rune::col().w(column).radius(10.0).clip_rounded(10.0).border(1.0, edge()).border_layout().children(rows)))
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
        key(&mut rt, Key::Text("z".into()));
        assert!(rt.state.tried.is_empty());
        // A second Z inside eight seconds does it.
        key(&mut rt, Key::Text("z".into()));
        assert_eq!(rt.state.tried, rules::withdraw(&item, true));
        // A refusal puts it back on the pile and says why.
        lands(&mut rt, Sent::TakenBack { thread: "storefront/x".to_string() }, Err("connection refused".to_string()));
        assert_eq!(sent_line(&rt).as_deref(), Some("Could not undo that. connection refused"));
        assert_eq!(rt.state.undos.len(), 1);
        // With nothing at all on the pile, Z says so.
        rt.state.undos.clear();
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
        assert!(rt.elements().iter().any(|e| e.text.as_deref() == Some("You picked option 1")));
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
        key(&mut rt, Key::Text("e".into()));
        assert!(rt.state.held.is_none(), "the press did not land");
        assert_eq!(sent_line(&rt).as_deref(), Some("This agent is waiting on your answer, and marking it done leaves it stuck. Press again to go ahead."));
        assert!(rt.state.answering.warned);
        key(&mut rt, Key::Text("e".into()));
        assert!(rt.state.held.as_ref().is_some_and(|h| h.closes.is_some()), "the second press closes it");
    }

    #[test]
    fn the_warning_given_on_l_is_the_same_once_and_opens_the_picker_with_it() {
        let mut rt = asking("warned-later", &["--thread"]);
        key(&mut rt, Key::Text("l".into()));
        assert!(rt.state.later.on.is_some());
        assert_eq!(rt.state.later.warning.as_deref(), Some("This agent is waiting on your answer, and putting it off leaves it stuck."));
        key(&mut rt, Key::Escape);
        // Said once for both keys.
        key(&mut rt, Key::Text("e".into()));
        assert!(rt.state.held.is_some());
    }

    #[test]
    fn a_reply_from_needs_you_moves_on_and_one_from_elsewhere_stays_and_follows_the_thread() {
        let mut rt = window("moves-on", &["--view", "inbox"]);
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
