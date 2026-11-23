//! Where a thread stands, around its conversation and on its row: the facts the line at a
//! conversation's foot is read from (`factsFor` and what `App.tsx` adds for the open thread), the
//! bar over a stopped conversation (`.stalled-bar` in `Focus.tsx`), Try again on the line, on
//! that bar, after a row's Status word (`TryAgainButton`) and in the command bar, what a press of
//! it writes and says (`tryAgain` in `App.tsx`), and the card that says why the cloud has not got
//! a thread, under a row's Status word or over the working line's place (`.th-why-card`).
//!
//! The line itself is drawn with the conversation (`thread.rs`), from `rules::live::foot`.

use std::sync::mpsc::{Receiver, Sender, channel};

use rune::{El, TextStyle, Tx, Waker, col, el, icon, row, text};
use serde_json::Value;

use crate::data::Item;
use crate::rules::{kinds, list, live};
use crate::source::{self, Source};
use crate::theme::*;
use crate::{State, icons};

/* ----------------------------------- the open thread ----------------------------------- */

/// What the supervisor and this window say about the thread on screen (`factsFor`, with what
/// `App.tsx` adds for the conversation: whether it is In progress by the tab's own rule, when it
/// comes back from Later, the answer held for its undo counted as queued; and the narrower
/// `stalled` the conversation is handed, which leaves out a thread asking who should take it and
/// one signed out).
pub fn facts(s: &State, item: &Item) -> live::Facts {
    let supervisor = &s.snap.supervisor;
    let id = item.id.as_str();
    let mut f = live::Facts::of(supervisor, id, s.now);
    // An approval in the spool names it, or a sign-in link it gave waits on the person.
    let approvals = s.snap.rest.get("approvals").and_then(Value::as_array);
    f.asking = approvals.is_some_and(|a| a.iter().any(|a| a.get("item").and_then(Value::as_str) == Some(id))) || kinds::open_sign_in(item);
    let listed = |name: &str| supervisor.rest.get(name).and_then(Value::as_array).is_some_and(|ids| ids.iter().any(|v| v.as_str() == Some(id)));
    // What the lists count as stopped: the supervisor's, and what waits on a sign-in.
    let stopped = f.stalled || f.sign_in_needed.is_some();
    f.stalled = f.stalled && !kinds::asks_who(item) && f.sign_in_needed.is_none();
    let starting = listed("starting");
    let live = f.session.is_some() || starting;
    // A thread being started again runs now, whatever moment it had (`dueAt`, `hiddenAt`).
    let due = if starting { 0.0 } else { list::deferred_until(item) };
    let hidden = if starting { 0.0 } else { list::hidden_until(item, s.local.snooze.get(id).copied().unwrap_or(0.0)) };
    f.in_progress = list::belongs_in_progress(item, &list::Facts { deferred_until: due, now: s.now, live, shipping: f.shipping, stopped, ..list::Facts::default() });
    let back = due.max(hidden);
    f.scheduled_until = if back > s.now { back } else { 0.0 };
    // An answer held for its undo is already on its way to an agent (`queuedWithHeld`).
    if let (Some(held), Some(queued)) = (&s.held, f.queued.as_mut())
        && matches!(held.sent, crate::write::Sent::Reply { .. } | crate::write::Sent::Pick { .. } | crate::write::Sent::Carry { .. })
        && held.thread.0 == item.id
        && !queued.iter().any(|q| q == id)
    {
        queued.push(id.to_string());
    }
    f
}

/// Whether the command bar offers Try again on the thread under the keys (`itemCommands` in
/// `App.tsx`): on the fleet's own facts, In progress by the rule that says a thread can be
/// stopped (`stoppableNow`), with no moment it is put off to.
pub fn offered_on_the_bar(s: &State, item: &Item) -> bool {
    let mut f = facts(s, item);
    f.stalled = live::Facts::of(&s.snap.supervisor, &item.id, s.now).stalled;
    f.in_progress = f.in_progress && item.status != "done" && !item.truthy("agent");
    f.scheduled_until = 0.0;
    live::offers_try_again(item, &f)
}

/// The bar over a stopped conversation (`.stalled-bar`): over the title, as wide as the column,
/// 4 points under the page's top (the 18 under it are its block's), on the raised ground with the
/// card's corner. It says the agent
/// stopped, and that the person's answer still waits when it does; at its end "try again →" in
/// the accent. Its edge comes up in the faint ink under the pointer, and a press anywhere on it
/// is Try again.
pub fn stalled_bar(item: &Item, column: f32) -> El {
    let words = if item.live_answer().is_some() { "The agent stopped without finishing this. Your answer is still waiting." } else { "The agent stopped without finishing this." };
    // The two sit on one baseline: the smaller line's box starts half a point lower.
    let label = row().grow(1.0).min_w(0.0).child(text(words).font(TextStyle::new(14.0, 400.0).lh(22.4)).color(dim()).wrap());
    let go = el().mt(0.5).child(text("try again →").font(TextStyle::new(13.5, 600.0).lh(21.6)).color(accent()));
    let (id, product) = (item.id.clone(), item.product.clone());
    row()
        .key("stalled-bar")
        .w(column)
        .px(15.0)
        .py(11.0)
        .gap(10.0)
        .items_start()
        .radius(6.0)
        .fill(rgb(47, 45, 44))
        .hover_group()
        .cursor(rune::Cursor::Pointer)
        .role("button")
        .child(crate::feel::edge_lit(6.0, muted(), false))
        .child(label)
        .child(go)
        .on_press_tx(move |s: &mut State, tx| s.try_again(&id, &product, tx))
}

/// Try again on the line at the foot (`.live-retry`): a quiet button 24 high, 9 in on each side
/// and 4 more after what is before it, a hairline edge on the card's corner, its words 12.5
/// points in dim ink; under the pointer the text's own ink on the strong wash inside the strong
/// edge, and the pointing hand.
pub fn retry_button(id: &str, product: &str) -> El {
    let (id, product) = (id.to_string(), product.to_string());
    row()
        .key("live-retry")
        .h(24.0)
        .ml(4.0)
        .px(10.0)
        .items_center()
        .radius(6.0)
        .border(1.0, edge())
        .hover(crate::feel::wash_strong())
        .hover_group()
        .cursor(rune::Cursor::Pointer)
        .role("button")
        .label("Try again")
        .child(crate::feel::edge_lit(6.0, rule(), false))
        .child(pen(text("Try again").font(TextStyle::new(12.5, 400.0).lh(20.0)).color(dim()).group_color(ink()), 12.5))
        .on_press_tx(move |s: &mut State, tx| s.try_again(&id, &product, tx))
}

/// Try again after a row's Status word (`TryAgainButton`): an 18-point square 2.5 after the
/// word, the arrow 11 points in dim ink, always drawn; under the pointer the text's ink on the
/// strong wash and the pointing hand, its title "Try again". A press tries again and does not
/// open the row.
pub fn row_retry(id: &str, product: &str) -> El {
    let (id, product) = (id.to_string(), product.to_string());
    row()
        .key(format!("retry-{product}-{id}"))
        .size(18.0, 18.0)
        .ml(2.5)
        .center()
        .radius(6.0)
        .hover(crate::feel::wash_strong())
        .hover_group()
        .cursor(rune::Cursor::Pointer)
        .role("button")
        .label("Try again")
        .help("Try again")
        .child(icon(&icons::TRY_AGAIN, 11.0).color(dim()).group_color(ink()))
        .on_press_tx(move |s: &mut State, tx| s.try_again(&id, &product, tx))
}

/* -------------------------------------- try again -------------------------------------- */

/// What Try again's answers land in, off the main thread.
pub struct Retries {
    tx: Sender<Result<Value, String>>,
    rx: Receiver<Result<Value, String>>,
    waker: Option<Waker>,
}

impl Default for Retries {
    fn default() -> Self {
        let (tx, rx) = channel();
        Retries { tx, rx, waker: None }
    }
}

impl Retries {
    pub fn wake_with(&mut self, waker: Waker) {
        self.waker = Some(waker);
    }
}

impl State {
    /// Try again on the thread `product/id` (`tryAgain` in `App.tsx`): Resume on that one thread,
    /// which ends a rest, a hold and a delivery mark and starts it now or as soon as an agent is
    /// free (`zero:resume-items`). The person stays where they are; the lists are read again at
    /// once, and a line says what came back, whatever it was. Not on an imported terminal
    /// session, nor on the rows the app makes itself.
    pub fn try_again(&mut self, id: &str, product: &str, tx: &mut Tx) {
        let Some(item) = self.snap.items.iter().find(|i| i.id == id && i.product == product) else {
            return;
        };
        if !live::can_try_again(item) {
            return;
        }
        let door = match &self.source {
            Source::Door(door) => door.clone(),
            _ => return self.tell(&live::try_again_words(None), tx),
        };
        let Some((url, token)) = door.now() else {
            return self.tell(&live::try_again_words(None), tx);
        };
        let (answers, waker, payload) = (self.retries.tx.clone(), self.retries.waker.clone(), live::try_again(id));
        std::thread::spawn(move || {
            let answer = source::call(&url, &token, live::TRY_AGAIN, &payload);
            // What was started changes the lists: read them again at once (`refresh`).
            door.ask_soon();
            let _ = answers.send(answer);
            if let Some(wake) = waker {
                wake();
            }
        });
    }
}

/// The repeating tasks' rules (`zero:repeats`), for the schedule a working run of one says under
/// its title (`repeatsLine`). The web window reads them when it opens and as threads come and go;
/// this one when a run of a repeating task is open and the snapshot's threads changed in number
/// since the last read.
#[derive(Default)]
pub struct Repeats {
    pub rules: Vec<Value>,
    asked: Option<usize>,
    reading: rune::Latest<Result<Value, String>>,
}

impl Repeats {
    pub fn wake_with(&self, waker: Waker) {
        self.reading.set_waker(waker);
    }
}

/// "Repeats daily at 9:00am" for the open thread, when it is a run of a repeating task whose
/// rule has been read.
pub fn repeats_of(s: &State, item: &Item) -> Option<String> {
    live::rule_for(item, &s.repeats.rules).map(live::repeats_line)
}

/// What Try again's presses came back with since the last look, each said in a line, and the
/// repeating tasks' rules, read when a run of one is open.
pub fn tend(s: &mut State, tx: &mut Tx) -> bool {
    // A picture taken just after Try again was pressed on the line says so (`pressed`).
    if s.fixture.get("pressed").and_then(Value::as_str) == Some(".live-retry")
        && let Some((id, product)) = s.open_item().map(|i| (i.id.clone(), i.product.clone()))
    {
        s.fixture.remove("pressed");
        s.try_again(&id, &product, tx);
    }
    let landed: Vec<Result<Value, String>> = s.retries.rx.try_iter().collect();
    for answer in &landed {
        s.tell(&live::try_again_words(Some(answer)), tx);
    }
    let mut changed = !landed.is_empty();
    if let Some(Ok(rules)) = s.repeats.reading.take() {
        s.repeats.rules = rules.as_array().cloned().unwrap_or_default();
        changed = true;
    }
    let a_run = matches!(&s.view, crate::View::Thread { id, product } if s.snap.items.iter().any(|i| &i.id == id && &i.product == product && kinds::rule_id_of(i).is_some()));
    if a_run && s.repeats.asked != Some(s.snap.items.len()) {
        // A picture's fixture says what the app answered.
        if let Some(rules) = s.fixture.get("repeats").and_then(Value::as_array) {
            s.repeats.rules = rules.clone();
            s.repeats.asked = Some(s.snap.items.len());
            changed = true;
        } else if let Source::Door(door) = &s.source
            && let Some((url, token)) = door.now()
        {
            s.repeats.asked = Some(s.snap.items.len());
            s.repeats.reading.request((url, token), |(url, token)| source::call(&url, &token, "zero:repeats", &Value::Null));
        }
    }
    // While the open conversation's line counts something, it is drawn again every 15 seconds.
    if let crate::View::Thread { id, product } = &s.view
        && let Some(item) = s.snap.items.iter().find(|i| &i.id == id && &i.product == product)
        && live::counting(&facts(s, item))
    {
        s.wakes.every(tx, "the line's count", std::time::Duration::from_millis(live::TICK_MS as u64));
    }
    changed
}

/* ------------------------------------- the why card ------------------------------------- */

/// The card saying why the cloud has not got a thread (`.th-why-card`): 320 points wide, 11 by
/// 14 inside, on the pop-up's ground inside the strong edge with the large shadow and a corner of
/// 10; its head in the text's ink, weight 500, three points over its body in dim ink, 13 points
/// on a 1.45 line.
pub fn why_card(head: &str, body: &str) -> El {
    col().w(320.0)
        .pt(12.0)
        .pb(13.0)
        .px(15.0)
        .radius(10.0)
        .fill(rgb(54, 52, 49))
        .border(1.0, rule())
        .box_shadow(shadow(0.0, 18.0, 50.0, 0.0, rgba(0, 0, 0, 0.45), false))
        .a11y_hidden()
        .child(pen(text(head).font(WHY_HEAD).color(ink()).wrap(), 13.0))
        .child(pen(text(body).font(WHY_BODY).color(dim()).wrap(), 13.0).mt(3.0))
}

const WHY_HEAD: TextStyle = TextStyle::new(13.0, 500.0).lh(18.85);
const WHY_BODY: TextStyle = TextStyle::new(13.0, 400.0).lh(18.85);

/// How tall that card stands: its words wrapped in its 290 points, and what is round them.
fn why_card_height(head: &str, body: &str) -> f32 {
    let tall = |words: &str, font: TextStyle| rune::text::measure(words, &[], font, Some(290.0)).1;
    1.0 + 11.0 + tall(head, WHY_HEAD) + 3.0 + tall(body, WHY_BODY) + 12.0 + 1.0
}

/// Where a card is up: the element keyed `at` the pointer rests on, and what its card says.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WhyUp {
    pub at: String,
    pub head: String,
    pub body: String,
    /// Under the element (a row's Status word), or over it (the working line's place, as the
    /// reply box sits right under that line).
    pub over: bool,
    /// The element whose foot is as far as the card shows: the web window's rows paint over a
    /// card hanging from the row above them, so under a row with another after it only the
    /// card's top shows, down to that row's foot.
    pub cut_at: Option<String>,
}

/// Has `e` put its card up while the pointer rests on it, and take it down when the pointer
/// leaves (`.has-card:hover .th-why-card { display: block }`), keyed `at`. `cut_at` is what the
/// card shows no further than (see `WhyUp`).
pub fn with_card(e: El, at: String, (head, body): (String, String), over: bool, cut_at: Option<String>) -> El {
    let up = WhyUp { at: at.clone(), head, body, over, cut_at };
    e.key(at.clone()).on_hover_change(move |s: &mut State, inside, _| {
        if inside {
            s.why_up = Some(up.clone());
        } else if s.why_up.as_ref().is_some_and(|w| w.at == at) {
            s.why_up = None;
        }
    })
}

/// The card that is up, over everything, where its element was last laid out: under it 10
/// points down, or over it 10 points up, lined up with its left.
pub fn why_layer(s: &State) -> Option<El> {
    let up = s.why_up.as_ref()?;
    let [x, y, _, h] = rune::memo::window_layout(&up.at)?;
    let card = why_card(&up.head, &up.body);
    let tall = why_card_height(&up.head, &up.body);
    if up.over {
        return Some(el().absolute(x, y - 10.0 - tall).child(card));
    }
    let top = y + h + 10.0;
    // Shown only down to the foot of the row it hangs from, where the next row paints over it.
    let shown = up.cut_at.as_deref().and_then(rune::memo::window_layout).map_or(tall + 60.0, |[_, ry, _, rh]| (ry + rh - top).max(0.0));
    Some(el().absolute(x, top).size(320.0, shown.min(tall + 60.0)).clip_rounded(0.0).child(card))
}

#[cfg(test)]
mod tests {
    use rune::Runtime;
    use serde_json::json;

    use crate::State;
    use crate::feel::tests::{press, run, window_dressed};

    /// The invented inbox's first thread, opened as `fleet` leaves it: open, the person's reply
    /// waiting on an agent, nothing on it.
    fn stuck(name: &str, fleet: serde_json::Value) -> Runtime<State> {
        window_dressed(name, &["--thread", "w-feel0"], "/fixtures/storefront", "Done, and tested. Merge it?", move |fixture| {
            let now = fixture["now"].as_f64().unwrap();
            let item = &mut fixture["snapshot"]["items"][0];
            item["status"] = json!("open");
            item["answer"] = json!("Go on.");
            item["wrote"]["answer"] = json!({ "ts": now - 60_000.0, "source": "founder" });
            for (k, v) in fleet.as_object().unwrap() {
                fixture["snapshot"]["supervisor"][k] = v.clone();
            }
        })
    }

    fn texts(rt: &Runtime<State>) -> Vec<String> {
        rt.elements().iter().filter_map(|e| e.text.clone()).collect()
    }

    #[test]
    fn a_thread_nothing_will_start_says_so_at_its_foot_and_offers_try_again() {
        let mut rt = stuck("foot-idle", json!({ "queued": [] }));
        run(&mut rt, 0.1);
        let said = texts(&rt);
        assert!(said.iter().any(|t| t == "Nothing running"), "{said:?}");
        let (x, y) = rt.find("live-retry").expect("Try again on the line");
        press(&mut rt, x, y);
        run(&mut rt, 0.1);
        // A picture has no app behind it, and says so, as the web window with no bridge does.
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Try again did not reach the app."));
    }

    #[test]
    fn a_stopped_thread_draws_its_bar_over_the_title_and_no_line_at_its_foot() {
        let mut rt = stuck("foot-stalled", json!({ "queued": [], "stalled": ["w-feel0"] }));
        run(&mut rt, 0.1);
        let said = texts(&rt);
        assert!(said.iter().any(|t| t == "The agent stopped without finishing this. Your answer is still waiting."), "{said:?}");
        assert!(said.iter().any(|t| t == "try again →"));
        assert!(!said.iter().any(|t| t == "Nothing running"));
        assert!(rt.find("live-retry").is_none());
        let (x, y) = rt.find("stalled-bar").expect("the bar");
        press(&mut rt, x, y);
        run(&mut rt, 0.1);
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Try again did not reach the app."));
    }

    #[test]
    fn a_stuck_rows_mark_tries_again_without_opening_the_row_and_the_bar_offers_it_too() {
        let mut rt = window_dressed("foot-row", &["--view", "inbox", "--tab", "progress"], "/fixtures/storefront", "Done, and tested. Merge it?", |fixture| {
            let now = fixture["now"].as_f64().unwrap();
            let item = &mut fixture["snapshot"]["items"][0];
            item["status"] = json!("open");
            item["answer"] = json!("Go on.");
            item["wrote"]["answer"] = json!({ "ts": now - 60_000.0, "source": "founder" });
        });
        run(&mut rt, 0.1);
        let row = rt.state.lists.in_progress_tab().first().map(|r| (r.retry, r.status_title.clone()));
        assert_eq!(row, Some((true, Some("No agent is on this, and none is waiting to start.".to_string()))));
        let (x, y) = rt.find("retry-storefront-w-feel0").expect("the mark after the word");
        press(&mut rt, x, y);
        run(&mut rt, 0.1);
        assert!(matches!(rt.state.view, crate::View::Inbox), "the row did not open");
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Try again did not reach the app."));
        // ⌘K over the row offers it after Reply.
        rt.key(rune::KeyEvent { key: rune::Key::Command('k'), mods: rune::Mods { cmd: true, ..rune::Mods::default() } });
        run(&mut rt, 0.1);
        assert!(texts(&rt).iter().any(|t| t == "Try again"), "{:?}", texts(&rt));
    }

    /// `queuedWithHeld`: a reply held for its three seconds is already on its way to an agent.
    #[test]
    fn a_reply_held_for_its_undo_reads_as_queued() {
        let mut rt = stuck("foot-held", json!({ "queued": [] }));
        run(&mut rt, 0.1);
        assert!(texts(&rt).iter().any(|t| t == "Nothing running"));
        rt.key(rune::KeyEvent { key: rune::Key::Text("r".into()), mods: rune::Mods::default() });
        run(&mut rt, 0.1);
        rt.state.reply = rune::TextEdit::new("Carry on with the second one.");
        rt.key(rune::KeyEvent { key: rune::Key::Enter, mods: rune::Mods { cmd: true, ..rune::Mods::default() } });
        run(&mut rt, 0.2);
        assert!(rt.state.held.is_some(), "held for its undo");
        assert!(texts(&rt).iter().any(|t| t == "Queued"), "{:?}", texts(&rt));
    }

    /// `workingContext`'s schedule: a run of a repeating task says when it repeats.
    #[test]
    fn a_working_run_of_a_repeating_task_says_its_schedule_under_the_title() {
        let mut rt = window_dressed("foot-repeats", &["--thread", "w-feel0"], "/fixtures/storefront", "Done, and tested. Merge it?", |fixture| {
            let now = fixture["now"].as_f64().unwrap();
            let item = &mut fixture["snapshot"]["items"][0];
            item["status"] = json!("claimed");
            item["labels"] = json!(["founder", "repeat:r-1"]);
            item["body"] = json!("Send the weekly numbers");
            fixture["snapshot"]["supervisor"]["running"] = json!([{ "itemId": "w-feel0", "product": "storefront", "startedAt": now - 60_000.0, "tail": [] }]);
            fixture["repeats"] = json!([{ "id": "r-1", "product": "storefront", "every": "weekday", "at": "09:00" }]);
        });
        run(&mut rt, 0.2);
        assert!(texts(&rt).iter().any(|t| t == "You asked: Send the weekly numbers. Repeats weekdays at 9:00am."), "{:?}", texts(&rt));
    }

    #[test]
    fn each_still_state_says_its_own_words_and_only_the_stuck_ones_offer_try_again() {
        for (fleet, words, retry) in [
            (json!({ "queued": ["w-feel0"] }), "Queued", false),
            (json!({ "queued": ["w-feel0"], "runNow": ["w-feel0"] }), "Up next", false),
            (json!({ "queued": [], "paused": true }), "Paused", false),
            (json!({ "queued": [], "shipping": ["w-feel0"] }), "Shipping", false),
            (json!({ "queued": ["w-feel0"], "cannotStart": { "w-feel0": { "why": "asleep", "tool": "Claude Code" } } }), "This computer looks asleep to the app, so nothing starts until it wakes.", false),
            (json!({ "queued": [], "signInNeeded": { "w-feel0": "Claude Code" } }), "Claude Code is signed out. Open a terminal, run claude, and type /login, and this starts on its own.", false),
            (json!({ "queued": [], "atLimit": { "w-feel0": { "tool": "Claude Code", "until": 0 } } }), "Claude Code is at its usage limit, so this starts on its own when it resets.", true),
        ] {
            let mut rt = stuck("foot-states", fleet);
            run(&mut rt, 0.1);
            let said = texts(&rt);
            assert!(said.iter().any(|t| t == words), "{words}: {said:?}");
            assert_eq!(rt.find("live-retry").is_some(), retry, "{words}");
        }
    }
}
