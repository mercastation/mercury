//! What a close asks first (`renderer/src/close-asks-first.ts`, `markDone` and `answerCloseAsk`
//! in `renderer/src/App.tsx`, `closeAskStrip` in `Focus.tsx`, `askHere` in `ItemThread.tsx`):
//! a close on a thread whose work never reached main asks whether the agent should ship it, and
//! one on a thread with filed tasks still waiting for a yes asks what happens to them. The
//! question stands in the options' strip, as the question before a discard does, and this file
//! holds what the three share: the strip, and the keys that answer it. Its words are
//! `rules/close_asks.rs`. `docs/parity-ship-and-close.md`, B5.

use rune::edit::Key;
use rune::{Cursor, El, KeyEvent, TextStyle, Tx, el, row, text};
use serde_json::json;

use crate::data::Item;
use crate::made::{self, Made};
use crate::rules::close_asks::{self as rules, Act, CloseAsk};
use crate::rules::filed_in_thread::{self, FiledInThread};
use crate::rules::{answering, palette, shipping, writes};
use crate::theme::*;
use crate::{State, View, write};

/// Why the open thread's close is asking (`closeAskWhy`, less the discard, which keeps its own
/// place in `answer.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    Filed,
    Unshipped,
}

/// The close question, while one is up.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Closing {
    /// The thread whose close is asking, as `product/id`, and why (`closeAskOn`).
    pub on: Option<(String, Why)>,
    /// The answer the arrows are on (`optionSel`).
    pub lit: Option<usize>,
    /// Bumped on every raise, so asking again brings it back onto the screen (`closeAskRaised`).
    pub raised: u32,
    /// Owed a scroll onto the screen once it is drawn.
    pub bring: bool,
    /// Owed the conversation unfolded, once the thread it was asked on has landed: landing on a
    /// thread puts back only what was unfolded when it was last left.
    pub unfold: bool,
    /// The pointer is on the head of the app's ship question (`askPeek`, `ship.rs`).
    pub head_peek: bool,
    /// Where the page last drew the question, noticed as it is drawn: at the foot (`Some(None)`),
    /// or at its task on an older turn, in the conversation's row of that key.
    pub drawn: std::cell::RefCell<Option<Option<String>>>,
}

fn key_of(item: &Item) -> String {
    format!("{}/{}", item.product, item.id)
}

/* ------------------------------ what every such question shares ------------------------------ */

/// The question at the head of the strip, inside its frame (`.opt-head.opt-head-ask`): 12.5
/// points in the text's own case and ink, two lines at most.
pub fn ask_head(ask: &str) -> El {
    el().key("ask-head").pt(11.0).px(16.0).pb(7.0).child(pen(text(ask).font(TextStyle::new(12.5, 600.0).lh(17.5)).color(ink()).wrap().max_lines(2), 12.5))
}

/// The tasks a question is about, named so one filed far up the thread can be found: a dot and
/// its title, each opening it (`.opt-named`).
fn named_rows(named: &[Made]) -> El {
    rune::col().mb(6.0).children(named.iter().map(|r| {
        let (id, product) = (r.id.clone(), r.product.clone());
        row()
            .min_h(30.0)
            .px(16.0)
            .gap(10.0)
            .items_center()
            .hover(crate::feel::wash())
            .cursor(Cursor::Pointer)
            .role("button")
            .label(format!("Open {}", r.title))
            .help(format!("Open {}", r.title))
            .on_press(move |s: &mut State| s.open_filed(&id, &product))
            .child(el().size(6.0, 6.0).radius(3.0).fill(accent()))
            .child(row().grow(1.0).min_w(0.0).child(pen(text(r.title.as_str()).font(TextStyle::new(14.0, 400.0).lh(kept(22.4))).color(ink()).truncate(), 14.0)))
    }))
}

/// A question that stands where the options stood (`OptionsOffer` with a close question): its
/// head, the tasks it names when asked to, and its answers numbered and pressable, the one the
/// arrows are on lit and saying what Enter does, the recommended one saying so. `key` names the
/// strip and its rows (`{key}-ask`, `{key}-{at}`); `press` answers it with an option's number.
pub fn ask_strip(key: &str, ask: &CloseAsk, lit: Option<usize>, named: Option<&[Made]>, column: f32, press: fn(&mut State, usize, &mut Tx)) -> El {
    let count = ask.options.len();
    let rows = ask.options.iter().enumerate().map(|(at, o)| {
        let c = &o.choice;
        let lit = lit == Some(at);
        let tone = if lit { ink() } else { dim() };
        // The number in its cap (`.opt-key`): 11 points in a frame of the strong line, six
        // points either side.
        let number = row().h(18.95).min_w(20.6).px(6.0).items_center().justify_center().radius(6.0).border(1.0, if lit { muted() } else { rule() }).border_layout().mt(1.5).child(text(format!("{}", c.n)).font(TextStyle::new(11.0, 400.0).lh(15.95)).color(tone));
        let last = at + 1 == count;
        let n = c.n as usize;
        let mut line = row()
            .key(format!("{key}-{at}"))
            .items_start()
            .min_h(if last { 39.45 } else { 35.45 })
            .pt(7.0)
            .pb(if last { 11.0 } else { 7.0 })
            .px(16.0)
            .gap(10.0)
            .cursor(Cursor::Pointer)
            .on_press_tx(move |s: &mut State, tx| press(s, n, tx))
            .child(number)
            .child(pen(text(c.text.as_str()).font(TextStyle::new(14.5, 400.0).lh(21.025)).color(tone).group_color(ink()), 14.5));
        line = if lit { line.fill(white(0.08)) } else { line.hover_group().hover(crate::feel::wash()) };
        let aside = if lit { Some("↵ send") } else if c.recommended { Some("recommended") } else { None };
        match aside {
            Some(said) => line.child(el().pl(8.0).mt(3.5).child(pen(text(said).font(TextStyle::new(11.5, 400.0).lh(16.675)).color(muted()), 11.5))),
            None => line,
        }
    });
    let mut strip = rune::col().key(format!("{key}-ask")).w(column).radius(10.0).clip_rounded(10.0).border(1.0, edge()).border_layout().child(ask_head(&ask.ask));
    if let Some(named) = named.filter(|n| !n.is_empty()) {
        strip = strip.child(named_rows(named));
    }
    strip.children(rows)
}

/// What a key means to a question that is up (`closeAskOwnsKey` and the branch under it).
#[derive(Debug, PartialEq)]
pub enum AskKey {
    /// Escape: the thread is kept as it was.
    Keep,
    /// An arrow moved the light.
    Lit(Option<usize>),
    /// A number or Enter on the lit one: answer with this number.
    Answer(usize),
    /// Taken and doing nothing: Enter with nothing lit, a number the question does not have.
    Nothing,
    /// Not one of its keys: the key keeps its usual meaning.
    NotMine,
}

/// The browser's name for a key, as the rule reads it.
fn key_name(key: &Key) -> Option<String> {
    Some(match key {
        Key::Escape => "Escape".to_string(),
        Key::Enter => "Enter".to_string(),
        Key::Up => "ArrowUp".to_string(),
        Key::Down => "ArrowDown".to_string(),
        Key::Text(t) => t.clone(),
        _ => return None,
    })
}

/// What `e` does to a question of `count` answers with `lit` lit: the numbers answer it, the
/// arrows walk it (down from none lights the first and stops at the last, up from the first
/// lights none), Enter answers the lit one, Escape keeps the thread open.
pub fn ask_key(e: &KeyEvent, count: usize, lit: Option<usize>) -> AskKey {
    let Some(name) = key_name(&e.key).filter(|n| rules::close_ask_owns_key(n)) else { return AskKey::NotMine };
    match name.as_str() {
        "Escape" => AskKey::Keep,
        "ArrowDown" => AskKey::Lit(Some(lit.map_or(0, |at| (at + 1).min(count.saturating_sub(1))))),
        "ArrowUp" => AskKey::Lit(lit.and_then(|at| at.checked_sub(1))),
        "Enter" => lit.map_or(AskKey::Nothing, |at| AskKey::Answer(at + 1)),
        digit => match digit.parse::<usize>() {
            Ok(n) if n <= count => AskKey::Answer(n),
            _ => AskKey::Nothing,
        },
    }
}

/* ------------------------------ when a close asks ------------------------------ */

impl State {
    /// The close question up on the open thread, if any, and its words
    /// (`askingToClose`).
    pub fn close_asked(&self) -> Option<(Why, CloseAsk)> {
        let item = self.open_item()?;
        let (on, why) = self.closing.on.as_ref()?;
        if *on != key_of(item) {
            return None;
        }
        let ask = match why {
            Why::Unshipped => rules::unshipped_ask(),
            Why::Filed => rules::close_ask(made::waiting_filed(self, item).len())?,
        };
        Some((*why, ask))
    }

    /// Before a close is held: whether it asks first instead (`markDone`). Work that never
    /// reached main asks unless the close says it is meant (`unshipped_ok`); filed tasks still
    /// waiting ask unless the close is `force`d. From a list, the thread opens and asks there.
    pub(crate) fn close_asks_first(&mut self, item: &Item, unshipped_ok: bool, force: bool) -> bool {
        let unshipped = !unshipped_ok && shipping::is_not_shipped(item);
        if !unshipped && (force || made::waiting_filed(self, item).is_empty()) {
            return false;
        }
        let here = self.open_item().is_some_and(|open| open.id == item.id && open.product == item.product);
        if !here {
            match self.view {
                View::Thread { .. } => self.step_to_thread(&item.id, &item.product),
                _ => self.open_thread(&item.id, &item.product),
            }
        }
        let why = if unshipped { Why::Unshipped } else { Why::Filed };
        // One question at a time: the discard question goes, and nothing is lit.
        self.answering.discard = None;
        self.option = None;
        // Asking about filed tasks unfolds the conversation, so the turn that filed them is on
        // the page with the question under it (`setEarlierOpen(true); setWhole(true)`).
        self.closing = Closing { on: Some((key_of(item), why)), lit: None, raised: self.closing.raised + 1, bring: true, unfold: why == Why::Filed, ..Closing::default() };
        true
    }

    /// The close that answers a question, or Dismiss on a filed row: the warning before closing a
    /// waiting agent still comes first, and only work off main can still ask
    /// (`markDone(item, { force: true, … })`).
    pub(crate) fn close_forced(&mut self, id: &str, product: &str, unshipped_ok: bool, tx: &mut Tx) {
        let Some(item) = self.snap.items.iter().find(|i| i.id == id && i.product == product).cloned() else { return };
        if self.warns_first(&item, true, tx) || self.close_asks_first(&item, unshipped_ok, true) {
            return;
        }
        self.close_unasked(id, product, unshipped_ok, tx);
    }

    /// Leaving the thread drops its question.
    pub fn leave_the_close_ask(&mut self) {
        self.closing.on = None;
        self.closing.lit = None;
        self.closing.bring = false;
        self.closing.unfold = false;
        self.closing.head_peek = false;
    }

    /// The question answered with option `n` (`answerCloseAsk`). A number it does not have
    /// answers nothing and leaves it up.
    pub(crate) fn answer_close(&mut self, n: usize, tx: &mut Tx) {
        let Some((_, asked)) = self.close_asked() else { return };
        let Some(option) = asked.options.iter().find(|o| o.choice.n as usize == n) else { return };
        let Some(item) = self.open_item().cloned() else { return };
        let waiting = made::waiting_filed(self, &item);
        self.leave_the_close_ask();
        self.option = None;
        match option.act {
            // Keeping it open, and deciding each one, are the whole of their own answer.
            Act::Keep | Act::Decide | Act::Discard => {}
            Act::Ship => {
                if let Some(words) = shipping::ship_it_ask(&item) {
                    self.reply_as_said(&item, words, tx);
                }
            }
            // The close again, meant: it may still ask about the filed tasks.
            Act::Close => self.close(&item.id, &item.product, true, tx),
            Act::Start | Act::Drop => {
                // One at a time through the same presses the rows carry, each writing the one
                // before it, and then the close for real.
                for kid in &waiting {
                    if option.act == Act::Start {
                        self.approve_as(kid, palette::Approval::Run, tx);
                    } else {
                        self.close_forced(&kid.id, &kid.product, false, tx);
                    }
                }
                self.close_forced(&item.id, &item.product, true, tx);
            }
        }
    }

    /// The question's keys while it is up; every other key keeps its meaning. True when taken.
    pub(crate) fn close_key(&mut self, e: &KeyEvent, tx: &mut Tx) -> bool {
        let Some((_, asked)) = self.close_asked() else { return false };
        match ask_key(e, asked.options.len(), self.closing.lit) {
            AskKey::NotMine => return false,
            AskKey::Keep => self.leave_the_close_ask(),
            AskKey::Lit(lit) => self.closing.lit = lit,
            AskKey::Answer(n) => self.answer_close(n, tx),
            AskKey::Nothing => {}
        }
        true
    }

    /// "Have the agent ship it": words sent to the thread as a reply is (`answerWith(item,
    /// words)`), held three seconds, with no priority and no mode with them, and the reply box
    /// left as it was, since nothing was typed in it. This is `hold_reply` without the box.
    fn reply_as_said(&mut self, item: &Item, words: String, tx: &mut Tx) {
        let thread = key_of(item);
        let running = self.run_on(item);
        let payload = if running { json!({ "product": item.product, "id": item.id, "answer": words }) } else { writes::reply(item, &words) };
        let drawn = self.answering.draw(&thread, self.moment(), &words);
        let around = if running { answering::Around { after: Vec::new(), undo_first: Vec::new() } } else { answering::around_a_reply(item, self.moment()) };
        let mut undo = around.undo_first;
        undo.extend(answering::withdraw(item, true));
        let moves_on = !running && self.reply_moves_on(item);
        let next = moves_on.then(|| self.next_after(item)).flatten();
        if !moves_on {
            self.answering.following = Some((item.id.clone(), item.product.clone()));
        }
        let answer = write::Answer {
            drawn,
            running,
            undo,
            undo_said: answering::REPLY_WITHDRAWN.to_string(),
            undoes: answering::REPLY_UNDOES.to_string(),
            mark: write::Mark::of(answering::UNDID_REPLY, None),
            moved_on: moves_on,
            in_flight: 0.0,
        };
        self.hold(
            write::Held {
                channel: writes::ANSWER,
                payload,
                sent: write::Sent::Reply { thread, words, pick: None, files: Vec::new(), answer },
                said: writes::said_sent(&self.project_name(item)),
                at: tx.clock() + writes::UNDO_GRACE_MS / 1000.0,
                thread: (item.id.clone(), item.product.clone()),
                from_inside: true,
                closes: None,
                then: around.after,
                mark: (!running).then(|| write::Mark::of(answering::UNDID_REPLY, None)),
            },
            tx,
        );
        if moves_on {
            let now = tx.clock();
            self.move_on_to(next, now);
        }
    }
}

/* ------------------------------ where it stands ------------------------------ */

/// Answers the open thread's close question with option `n`.
fn press_close(s: &mut State, n: usize, tx: &mut Tx) {
    s.answer_close(n, tx);
}

/// The unshipped question, at the conversation's foot where the discard question stands
/// (`closeAskStrip(false)`).
pub fn unshipped_strip(s: &State, column: f32) -> Option<El> {
    let (why, ask) = s.close_asked()?;
    (why == Why::Unshipped).then(|| {
        *s.closing.drawn.borrow_mut() = Some(None);
        ask_strip("close", &ask, s.closing.lit, None, column, press_close)
    })
}

/// The filed question, under the list of the tasks it is about (`askHere`): `at_task` names the
/// conversation's row of an older turn it stands in, rather than the foot.
pub fn filed_strip(s: &State, column: f32, at_task: Option<&str>) -> Option<El> {
    let (why, ask) = s.close_asked()?;
    (why == Why::Filed).then(|| {
        *s.closing.drawn.borrow_mut() = Some(at_task.map(str::to_string));
        ask_strip("close", &ask, s.closing.lit, None, column, press_close)
    })
}

/// Under a file that takes the whole card, where no conversation is on the screen, the question
/// stands over the reply box and names the tasks it is about itself (`fullScreenDoc &&
/// closeAskStrip(true)`); the question before a discard stands there too.
pub fn full_screen_strip(s: &State, column: f32) -> Option<El> {
    if let Some(asked) = s.discard_asked() {
        return Some(ask_strip("discard", &rules::discard_ask(), asked.lit, None, column, |s, n, tx| s.answer_discard(n, tx)));
    }
    let (why, ask) = s.close_asked()?;
    let item = s.open_item()?;
    let named: Vec<Made> = made::rows(s, item).into_iter().filter(|r| r.approve).collect();
    Some(ask_strip("close", &ask, s.closing.lit, (why == Why::Filed).then_some(named.as_slice()), column, press_close))
}

/// Whether the filed question is up on `item`, and the turn whose list it stands under: `None`
/// when it is not up, `Some(None)` at the foot (`askTurnOf`).
pub fn asks_at_turn(s: &State, item: &Item, made: &FiledInThread<Made>) -> Option<Option<usize>> {
    let (why, _) = s.close_asked()?;
    if why != Why::Filed || s.open_item().is_none_or(|open| open.id != item.id || open.product != item.product) {
        return None;
    }
    Some(filed_in_thread::ask_turn_of(made, |r| r.approve))
}

/// Unfolds the conversation a filed question was raised on, once its thread has landed, and
/// brings a question just raised onto the screen once it is drawn, at the foot or far up the
/// conversation (`bringIntoView`). Asked each turn, after the landing; true when it changed
/// anything.
pub fn bring_it(s: &mut State, tx: &mut Tx) -> bool {
    if !s.closing.bring || s.close_asked().is_none() {
        return false;
    }
    if s.closing.unfold {
        s.closing.unfold = false;
        if let Some(thread) = s.closing.on.as_ref().map(|(on, _)| on.clone()) {
            s.unfold_the_conversation(&thread);
        }
        return true;
    }
    let Some(at_task) = s.closing.drawn.borrow().clone() else { return false };
    s.closing.bring = false;
    // At its task, far up the conversation, its middle on the page's; at the foot, brought to
    // the nearest edge (`block: 'center'`, `block: 'nearest'`).
    if let Some(row) = at_task {
        let page = s.end_the_landing();
        tx.scroll_key_to(format!("{row}/{AT_MIDDLE}"), page / 2.0);
        tx.wake_after(std::time::Duration::ZERO);
    } else {
        tx.scroll_into_view("close-ask");
    }
    true
}

/// The close question standing under its task's list on an older turn (`.close-ask-at-task`),
/// and a point with no size halfway down it, which the page centres.
pub const AT_TASK: &str = "close-at-task";
pub const AT_MIDDLE: &str = "close-at-task-middle";

/// The question at its task, with the point the page centres halfway down it.
pub fn at_task(strip: El) -> El {
    let middle = rune::col().pin(Some(0.0), Some(0.0), None, Some(0.0)).w(0.0).justify_center().child(el().key(AT_MIDDLE).size(0.0, 0.0));
    rune::col().key(AT_TASK).child(middle).child(strip)
}

/// A picture taken after a close says so in its fixture (`closePressed`: `true`, as E pressed
/// on the open thread), and the same close is asked here.
pub fn picture(s: &mut State, tx: &mut Tx) {
    if s.fixture.remove("closePressed").is_some() {
        s.close_thread(false, tx);
    }
}

#[cfg(test)]
mod tests {
    use rune::{Key, KeyEvent, Mods, Runtime};
    use serde_json::json;

    use super::*;
    use crate::feel::tests::{key, press, window_dressed};
    use crate::rules::writes::ANSWER;

    /// The invented inbox's first thread, finished by its agent, with `note` the app's last word
    /// on it and `kids` the tasks it filed (each `(id, title)`), opened.
    fn closing(name: &str, note: Option<&str>, kids: &[(&str, &str)]) -> Runtime<State> {
        let note = note.map(str::to_string);
        let kids: Vec<(String, String)> = kids.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect();
        let mut rt = window_dressed(name, &["--thread", "w-feel0"], "/fixtures/storefront", "Done.", move |fixture| {
            let now = fixture["now"].as_f64().unwrap();
            let items = fixture["snapshot"]["items"].as_array_mut().unwrap();
            if let Some(note) = &note {
                items[0]["note"] = json!(note);
                items[0]["wrote"]["note"] = json!({ "ts": now - 1_000.0, "source": "system" });
                items[0]["wrote"]["result"] = json!({ "ts": now - 60_000.0, "source": "agent" });
            }
            for (n, (id, title)) in kids.iter().enumerate() {
                items.push(json!({ "id": id, "product": "storefront", "title": title, "status": "open", "kind": "task", "labels": [], "parent": "w-feel0", "createdAt": now - 30_000.0 + n as f64, "updatedAt": now - 30_000.0 }));
            }
        });
        rt.state.pretend_sent = true;
        // Past the moment E waits out after the screen changed, and past the once-ever warning.
        rt.state.answering.warned = true;
        crate::feel::tests::run(&mut rt, 1.0);
        rt
    }

    const FAILED: &str = "It did not ship, and it went back to its agent: the tests on main are red";

    fn says(rt: &Runtime<State>, words: &str) -> bool {
        rt.elements().iter().any(|e| e.text.as_deref() == Some(words))
    }

    fn e(rt: &mut Runtime<State>) {
        key(rt, Key::Text("e".into()));
    }

    #[test]
    fn a_close_on_work_that_never_reached_main_asks_first() {
        let mut rt = closing("close-unshipped", Some(FAILED), &[]);
        e(&mut rt);
        assert!(rt.state.held.is_none(), "nothing closed");
        assert_eq!(rt.state.close_asked().map(|(w, _)| w), Some(Why::Unshipped));
        for words in ["This thread's work never reached main.", "Have the agent ship it", "Close it anyway", "Keep this thread open", "recommended"] {
            assert!(says(&rt, words), "{words:?}");
        }
        // Escape keeps the thread open and the question goes.
        key(&mut rt, Key::Escape);
        assert!(rt.state.close_asked().is_none() && matches!(rt.state.view, View::Thread { .. }));
    }

    #[test]
    fn have_the_agent_ship_it_sends_the_ask_as_a_reply_with_nothing_else() {
        let mut rt = closing("close-ship", Some(FAILED), &[]);
        rt.update(|s, _| s.reply = rune::TextEdit::new("a draft of mine"));
        e(&mut rt);
        key(&mut rt, Key::Text("1".into()));
        assert!(rt.state.close_asked().is_none());
        rt.update(|s, _| s.write_what_waits());
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-feel0", "answer": "Fix what stopped the ship, then ship it.", "status": "open" }))]);
        assert_eq!(rt.state.reply.text, "a draft of mine", "the box is as it was");
    }

    #[test]
    fn close_it_anyway_closes_it_meant() {
        let mut rt = closing("close-anyway", Some(FAILED), &[]);
        e(&mut rt);
        key(&mut rt, Key::Down);
        key(&mut rt, Key::Down);
        assert_eq!(rt.state.closing.lit, Some(1));
        key(&mut rt, Key::Enter);
        rt.update(|s, _| s.write_what_waits());
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-feel0", "status": "done", "closeUnshipped": true }))]);
    }

    #[test]
    fn a_close_with_a_filed_task_waiting_asks_what_happens_to_it() {
        let mut rt = closing("close-filed", None, &[("w-kid1", "Say why Send is off")]);
        e(&mut rt);
        assert!(rt.state.held.is_none());
        for words in ["Before this thread closes, what happens to the task it filed?", "Start the task in a new thread, then close this one", "Close both without starting the task", "Keep this thread open"] {
            assert!(says(&rt, words), "{words:?}");
        }
        // The conversation unfolds, so the turn that filed it is on the page.
        crate::feel::tests::run(&mut rt, 0.5);
        assert!(rt.find("thread-whole").is_some(), "unfolded");
        // Under the list of what it filed.
        let (_, list) = rt.find("made-list").expect("the list");
        let (_, ask) = rt.find("close-ask").expect("the question");
        assert!(list < ask);
        // A number it does not have answers nothing; Enter with nothing lit does nothing.
        key(&mut rt, Key::Text("4".into()));
        key(&mut rt, Key::Enter);
        assert!(rt.state.close_asked().is_some() && rt.state.held.is_none());
        // Every other key keeps its meaning: E asks again.
        key(&mut rt, Key::Down);
        key(&mut rt, Key::Up);
        assert_eq!(rt.state.closing.lit, None);
        let raised = rt.state.closing.raised;
        e(&mut rt);
        assert_eq!(rt.state.closing.raised, raised + 1);
    }

    #[test]
    fn under_a_file_over_the_whole_card_the_question_stands_over_the_reply_box() {
        let mut rt = closing("close-full", None, &[("w-kid1", "Say why Send is off")]);
        assert!(full_screen_strip(&rt.state, 700.0).is_none());
        e(&mut rt);
        assert!(full_screen_strip(&rt.state, 700.0).is_some());
    }

    #[test]
    fn starting_the_task_approves_it_then_closes_the_thread() {
        let mut rt = closing("close-start", None, &[("w-kid1", "Say why Send is off")]);
        e(&mut rt);
        key(&mut rt, Key::Text("1".into()));
        rt.update(|s, _| s.write_what_waits());
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-kid1", "answer": "Approved. Run it." })), (ANSWER, json!({ "product": "storefront", "id": "w-feel0", "status": "done", "closeUnshipped": true }))]);
    }

    #[test]
    fn closing_both_closes_the_task_then_the_thread() {
        let mut rt = closing("close-drop", None, &[("w-kid1", "Say why Send is off")]);
        e(&mut rt);
        key(&mut rt, Key::Text("2".into()));
        rt.update(|s, _| s.write_what_waits());
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-kid1", "status": "done" })), (ANSWER, json!({ "product": "storefront", "id": "w-feel0", "status": "done", "closeUnshipped": true }))]);
    }

    #[test]
    fn two_waiting_recommend_keeping_the_thread_to_decide_each_one() {
        let mut rt = closing("close-two", None, &[("w-kid1", "Say why Send is off"), ("w-kid2", "Rename the usage page")]);
        e(&mut rt);
        assert!(says(&rt, "Before this thread closes, what happens to the two tasks it filed?"));
        key(&mut rt, Key::Text("3".into()));
        assert!(rt.state.close_asked().is_none() && rt.state.held.is_none() && rt.state.tried.is_empty());
    }

    #[test]
    fn a_press_answers_it_and_leaving_the_thread_drops_it() {
        let mut rt = closing("close-press", Some(FAILED), &[]);
        e(&mut rt);
        let (x, y) = rt.find("close-2").expect("Keep this thread open");
        press(&mut rt, x, y);
        assert!(rt.state.close_asked().is_none() && rt.state.held.is_none());
        e(&mut rt);
        rt.key(KeyEvent { key: Key::Escape, mods: Mods::default() });
        rt.frame(1.0 / 60.0);
        assert!(rt.state.close_asked().is_none());
        // The keys a question takes are only these.
        assert_eq!(ask_key(&KeyEvent { key: Key::Text("j".into()), mods: Mods::default() }, 3, None), AskKey::NotMine);
        assert_eq!(ask_key(&KeyEvent { key: Key::Down, mods: Mods::default() }, 3, Some(2)), AskKey::Lit(Some(2)));
    }

    #[test]
    fn a_close_from_the_list_opens_the_thread_and_asks_there() {
        let mut rt = closing("close-list", Some(FAILED), &[]);
        key(&mut rt, Key::Escape);
        assert!(!matches!(rt.state.view, View::Thread { .. }));
        rt.update(|s, tx| s.close("w-feel0", "storefront", false, tx));
        rt.frame(1.0 / 60.0);
        assert!(matches!(&rt.state.view, View::Thread { id, .. } if id == "w-feel0"));
        assert_eq!(rt.state.close_asked().map(|(w, _)| w), Some(Why::Unshipped));
    }
}
