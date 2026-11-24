//! The threads a thread filed, drawn in its conversation on the turn that filed them
//! (`renderer/src/components/ThreadsMade.tsx`, `threads-made.css`, placed by `ItemThread.tsx`):
//! one row per thread, a proposal leading with the accent dot and carrying its answers (Dismiss,
//! Do here, New thread), any other row the tab it sits in, and over two or more proposals
//! Dismiss all and Start all. Its rules are `rules/threads_made.rs` and
//! `rules/filed_in_thread.rs`; what each press writes is `resolve`, `doItHere`, `markDone` and
//! `answerAllFiled` in `renderer/src/App.tsx`. `docs/parity-ship-and-close.md`, B6.

use rune::{Cursor, El, TextStyle, Tx, col, el, row, text};
use serde_json::{Value, json};

use crate::data::{Item, Tab};
use crate::markdown::{self, Block};
use crate::rules::filed_in_thread::{self, FiledInThread, Spoken};
use crate::rules::threads_made::{self as rules, All, State as Where};
use crate::rules::writes::{ANSWER, Write};
use crate::rules::{answering, palette, title};
use crate::talk::{Said, Talk};
use crate::theme::*;
use crate::write::{self, Landed, Sent};
use crate::State;
#[cfg(test)]
use crate::View;

/// One thread this thread filed, as its row draws it (`MadeRow`), and when it was filed.
#[derive(Clone, Debug, PartialEq)]
pub struct Made {
    pub id: String,
    pub product: String,
    pub title: String,
    pub state: Option<Where>,
    /// Waiting on a press from the person (`approvableFiled`).
    pub approve: bool,
    pub at: f64,
}

/// What a press on the list wrote, kept until the engine answers.
#[derive(Clone, Debug, PartialEq)]
pub enum Wrote {
    /// A proposal handed back to the thread that filed it, as `(id, product)`, and its name.
    Here { child: (String, String), title: String },
    /// Every proposal of a list started or dismissed, and what one Z writes to take it back.
    All { act: All, kids: Vec<(String, String)>, undo: Vec<Write> },
}

/// Where a thread sits, as the tabs list it: Needs you wins, then In progress, Later and Done
/// (`tabState`). A thread the View and filters menu holds back is still in its tab.
pub fn state_of(s: &State, id: &str, product: &str) -> Option<Where> {
    let lists = &s.lists;
    if lists.holds(Tab::Needs, id, product) {
        Some(Where::Waiting)
    } else if lists.holds(Tab::Progress, id, product) {
        Some(Where::Running)
    } else if lists.holds(Tab::Later, id, product) {
        Some(Where::Scheduled)
    } else if lists.holds(Tab::Done, id, product) {
        Some(Where::Done)
    } else {
        None
    }
}

/// The threads `item` filed, as their rows say them (`filed={threadsMade(items, focused).map(…)}`).
pub fn rows(s: &State, item: &Item) -> Vec<Made> {
    rules::threads_made(&s.snap.items, item)
        .into_iter()
        .map(|i| {
            let state = state_of(s, &i.id, &i.product);
            Made { id: i.id.clone(), product: i.product.clone(), title: title::row_title(i), state, approve: rules::approvable_filed(i, state), at: i.created_at.unwrap_or(0.0) }
        })
        .collect()
}

/// The tasks `item` filed that still wait for a yes (`waitingFiled`).
pub fn waiting_filed(s: &State, item: &Item) -> Vec<Item> {
    rules::threads_made(&s.snap.items, item).into_iter().filter(|i| rules::approvable_filed(i, state_of(s, &i.id, &i.product))).cloned().collect()
}

/// The conversation as the turn rule reads it: what was said, by whom, and the lines of what
/// happened, which no list ever hangs on.
fn spoken(talk: &Talk) -> Vec<Spoken> {
    talk.said
        .iter()
        .map(|e| match e {
            Said::Message(m) => Spoken { at: m.at, work: false, mine: m.mine },
            Said::Step(st) => Spoken { at: st.at, work: true, mine: st.mine },
            Said::Work(w) => Spoken { at: w.at, work: true, mine: false },
        })
        .collect()
}

/// Each filed thread on the turn that filed it, the newest turn's at the foot (`filedOnTurn`).
pub fn on_page(s: &State, item: &Item, talk: &Talk) -> FiledInThread<Made> {
    filed_in_thread::filed_on_turn(&spoken(talk), &rows(s, item), |r| r.at)
}

/* ------------------------------ the list ------------------------------ */

/// The mono capitals of the list's label, its state words and its answers.
fn caps(words: &str, size: f32, tracking: f32) -> El {
    text(words.to_uppercase()).font(mono(size, 14.0).tracking(tracking))
}

/// One of a proposal's answers, or of the count line's (`.made-acts button`): 26 high, nine in on
/// either side, the words in the mono capitals, washed under the pointer.
fn answer(key: String, words: &str, tone: rune::Color, over: rune::Color, inked: bool, label: String, help: String) -> El {
    let said = pen(caps(words, 11.0, 0.66).color(tone), 11.0);
    let said = if inked { said.group_color(ink()) } else { said };
    row().key(key).h(26.0).px(9.0).items_center().radius(6.0).hover_group().hover(over).cursor(Cursor::Pointer).role("button").label(label).help(help).child(said)
}

/// The list itself (`ThreadsMade`): nothing when there is nothing to show.
pub fn list(rows: &[Made], column: f32) -> Option<El> {
    let can_press = |r: &Made| r.approve;
    let asks: Vec<&Made> = rows.iter().filter(|r| can_press(r)).collect();
    let waiting = !asks.is_empty();
    let shown = rules::made_shown(rows, can_press, false);
    if shown.is_empty() {
        return None;
    }
    let heading = if waiting { rules::waiting_words(asks.len()) } else { rules::FILED_HERE.to_string() };
    let ids: Vec<(String, String)> = asks.iter().map(|r| (r.id.clone(), r.product.clone())).collect();
    let first = asks.first().map_or(String::new(), |r| r.id.clone());
    let mut label = row().pr(11.0).mb(8.0).items_center().child(pen(caps(&heading, 11.0, 0.88).color(muted()), 11.0));
    // Over two or more proposals, the count line answers for the lot (`.made-all`).
    if asks.len() > 1 {
        let n = asks.len();
        let (dismiss_ids, start_ids) = (ids.clone(), ids.clone());
        let dismiss = answer(format!("made-all-dismiss-{first}"), rules::DISMISS_ALL, muted(), crate::feel::wash_strong(), true, format!("{} {n}", rules::DISMISS_ALL), format!("{} {n}: none of them will run", rules::DISMISS_ALL))
            .on_press(move |s: &mut State| s.answer_all_filed(&dismiss_ids, All::Dismiss));
        let start = answer(format!("made-all-start-{first}"), rules::START_ALL, dim(), crate::feel::wash_strong(), true, format!("{} {n} as new threads", rules::START_ALL), format!("{} {n} as new threads", rules::START_ALL))
            .on_press(move |s: &mut State| s.answer_all_filed(&start_ids, All::Start));
        // Ending where the rows' answers end, pulled into the line's own height (`.made-all`).
        label = label.child(el().grow(1.0)).child(row().gap(2.0).mr(-2.0).my(-6.0).child(dismiss).child(start));
    }
    let rows = shown.iter().enumerate().map(|(at, r)| made_row(r, can_press(r), waiting, at > 0));
    Some(col().key(format!("made-{}", shown[0].id)).w(column).child(label).child(col().key("made-list").radius(6.0).border(1.0, edge()).border_layout().children(rows)))
}

/// One row (`.made-row`): the press that opens the thread fills it, a proposal's answers stand
/// where a state word would.
fn made_row(r: &Made, press: bool, waiting: bool, ruled: bool) -> El {
    let quiet = !press && waiting;
    let title_ink = if quiet { muted() } else { ink() };
    let title = text(r.title.as_str()).font(TextStyle::new(14.0, 400.0).lh(kept(22.4))).color(title_ink).truncate();
    let title = if quiet { title.group_color(dim()) } else { title };
    let (id, product) = (r.id.clone(), r.product.clone());
    let mut open = row()
        .key(format!("made-open-{}", r.id))
        .grow(1.0)
        .min_w(0.0)
        .min_h(40.0)
        .pl(14.0)
        .gap(14.0)
        .items_center()
        .radius(6.0)
        .cursor(Cursor::Pointer)
        .role("button")
        .label(r.title.as_str())
        .on_press(move |s: &mut State| s.open_filed(&id, &product));
    if press {
        open = open.child(el().size(6.0, 6.0).mr(-4.0).radius(3.0).fill(accent()));
    }
    open = open.child(row().grow(1.0).min_w(0.0).child(pen(title, 14.0)));
    if !press {
        let tone = if quiet || matches!(r.state, Some(Where::Done | Where::Scheduled)) { muted() } else { dim() };
        open = open.child(pen(caps(rules::state_word(r.state), 11.0, 0.88).color(tone), 11.0));
    }
    let mut line = row().key(format!("made-row-{}", r.id)).min_h(40.0).pr(10.0).items_center().hover_group().hover(crate::feel::wash()).child(open);
    if ruled {
        line = line.border_sides([1.0, 0.0, 0.0, 0.0], edge()).border_layout();
    }
    if !press {
        return line;
    }
    // Named with the thread each one is about (`aria-label`), the main one at the right edge.
    let kid = (r.id.clone(), r.product.clone());
    let (a, b, c) = (kid.clone(), kid.clone(), kid);
    let dismiss = answer(format!("made-dismiss-{}", r.id), rules::DISMISS, muted(), crate::feel::wash_strong(), true, format!("{}: {}", rules::DISMISS, r.title), format!("{}: it will not run", rules::DISMISS))
        .on_press_tx(move |s: &mut State, tx| s.dismiss_filed(&a.0, &a.1, tx));
    let here = answer(format!("made-here-{}", r.id), rules::DO_HERE, dim(), crate::feel::wash_strong(), true, format!("Do in this thread: {}", r.title), format!("Do in this thread: {}", r.title))
        .on_press_tx(move |s: &mut State, tx| s.do_filed_here(&b.0, &b.1, tx));
    let approve = answer(format!("made-approve-{}", r.id), rules::START_NEW, accent(), rgba(238, 96, 24, 0.14), false, format!("Start new thread: {}", r.title), format!("Start new thread: {}", r.title))
        .on_press_tx(move |s: &mut State, tx| s.approve_filed(&c.0, &c.1, tx));
    line.child(row().gap(2.0).ml(12.0).mr(-2.0).child(dismiss).child(here).child(approve))
}

/* ------------------------------ where it stands ------------------------------ */

/// What the turn that ends at `end`, an index into the conversation `talk`, carries under its
/// last word: its list, 12 points under it, and the close question about its tasks under that
/// (`tail`, `.thread-block .made`, `.close-ask-at-task`). `row` is the key of the
/// conversation's row it is drawn in.
pub fn tail(s: &State, talk: &Talk, end: usize, column: f32, row: &str) -> Option<El> {
    let item = s.open_item()?;
    let made = on_page(s, item, talk);
    let rows = made.on_turn.get(&end)?;
    let list = list(rows, column)?;
    let mut out = col().pt(12.0).child(list);
    if crate::closing::asks_at_turn(s, item, &made) == Some(Some(end))
        && let Some(strip) = crate::closing::filed_strip(s, column, Some(row))
    {
        out = out.child(crate::closing::at_task(strip).mt(12.0));
    }
    Some(out)
}

/// What stands at the conversation's foot (`madeList(made.atFoot)` and `askHere`): the list of
/// what the newest turn filed, 22 points over it and 6 under, and the close question about the
/// filed tasks when it stands there. `conversation` says the page draws its turns, so a question
/// about an older turn stands at that turn instead; `from` is where the page starts in them.
pub fn foot(s: &State, item: &Item, talk: &Talk, conversation: bool, from: usize, column: f32) -> Vec<Block> {
    let made = on_page(s, item, talk);
    let mut blocks = Vec::new();
    if let Some(list) = list(&made.at_foot, column) {
        blocks.push(markdown::block(22.0, 6.0, list));
    }
    let here = match crate::closing::asks_at_turn(s, item, &made) {
        None => false,
        Some(None) => true,
        // An older turn's question stands at its turn, unless the page starts after it.
        Some(Some(turn)) => !conversation || turn < from,
    };
    if here && let Some(strip) = crate::closing::filed_strip(s, column, None) {
        blocks.push(markdown::block(12.0, 6.0, strip));
    }
    blocks
}

/* ------------------------------ what its presses do ------------------------------ */

impl State {
    fn filed_item(&self, id: &str, product: &str) -> Option<Item> {
        self.snap.items.iter().find(|i| i.id == id && i.product == product).cloned()
    }

    /// A row's title: that thread, from this one (`onOpenItem`).
    pub(crate) fn open_filed(&mut self, id: &str, product: &str) {
        if self.filed_item(id, product).is_some() {
            self.step_to_thread(id, product);
        }
    }

    /// New thread: the proposal approved as its own thread, the person staying on this one
    /// (`resolve(i, { stay: true })`, which for a proposal is "Approved. Run it.").
    pub(crate) fn approve_filed(&mut self, id: &str, product: &str, tx: &mut Tx) {
        if let Some(kid) = self.filed_item(id, product) {
            self.approve_as(&kid, palette::Approval::Run, tx);
        }
    }

    /// Dismiss: the proposal closed as its own row closes it, with its undo, the person staying
    /// (`markDone(i, { stay: true, force: true })`).
    pub(crate) fn dismiss_filed(&mut self, id: &str, product: &str, tx: &mut Tx) {
        self.close_forced(id, product, false, tx);
    }

    /// Do here: the proposal handed back to this thread as the person's message, and closed,
    /// parent first, held three seconds (`doItHere`).
    pub(crate) fn do_filed_here(&mut self, id: &str, product: &str, tx: &mut Tx) {
        let Some(child) = self.filed_item(id, product) else { return };
        let parent = child.text("parent").and_then(|p| self.filed_item(p, product));
        let Some(mut writes) = rules::do_it_here_writes(&child, parent.as_ref()) else { return };
        let Some(parent) = parent else { return };
        let name = writes_title(&child);
        let (channel, payload) = writes.remove(0);
        let said = format!("Doing it in this thread: {name}");
        let child_at = (child.id.clone(), child.product.clone());
        self.hold(
            write::Held {
                channel,
                payload,
                sent: Sent::Filed(Wrote::Here { child: child_at.clone(), title: name }),
                said: said.clone(),
                at: tx.clock() + crate::rules::writes::UNDO_GRACE_MS / 1000.0,
                thread: child_at.clone(),
                from_inside: false,
                closes: None,
                then: writes,
                mark: Some(write::Mark::of(UNDID_HERE, None)),
            },
            tx,
        );
        // Out of the lists now, as an answer held is; the line names the thread it went to.
        self.answering.holding.insert(child_at.0, false);
        self.refresh();
        let (line, undo) = crate::rules::compose::toast_parts(&format!("{said} · press Z to undo"));
        let since = tx.clock();
        let goes = (parent.id.clone(), parent.product.clone());
        self.last_toast = Some((goes.clone(), since));
        self.toast = Some(write::Toast { line, undo, since, until: since + crate::rules::compose::TOAST_GOES_MS / 1000.0, goes: Some(goes), sent: None, warns: false });
    }

    /// Start all and Dismiss all: every proposal of the list answered at once, what was held
    /// written first, and one Z for the lot (`answerAllFiled`).
    pub(crate) fn answer_all_filed(&mut self, kids: &[(String, String)], act: All) {
        let found: Vec<Item> = kids.iter().filter_map(|(id, product)| self.filed_item(id, product)).collect();
        if found.is_empty() {
            return;
        }
        self.write_what_waits();
        let (writes, sent) = all_writes(&found, act);
        self.send_all(writes, sent);
    }
}

/// What Start all or Dismiss all writes, and what it is filed under until the engine answers:
/// undoing a start stops whatever already started, then takes the approvals back.
fn all_writes(found: &[Item], act: All) -> (Vec<Write>, Sent) {
    let refs: Vec<&Item> = found.iter().collect();
    let (writes, mut undo) = rules::answer_all_writes(&refs, act);
    if act == All::Start {
        let mut stops: Vec<Write> = found.iter().map(|k| (answering::STOP, json!({ "product": k.product, "id": k.id }))).collect();
        stops.append(&mut undo);
        undo = stops;
    }
    let kids = found.iter().map(|k| (k.id.clone(), k.product.clone())).collect();
    (writes, Sent::Filed(Wrote::All { act, kids, undo }))
}

/// The thread's name as a line about it says it (`clipToSentence(rowTitle(child), TOAST_TITLE)`).
fn writes_title(item: &Item) -> String {
    crate::rules::writes::clip_to_sentence(&title::row_title(item), 48)
}

/// What a Z inside the three seconds leaves on the proposal.
const UNDID_HERE: &str = "Undid doing it here";

/// What the engine said to the list's writes.
pub fn landed(s: &mut State, landed: Landed, tx: &mut Tx) {
    let Sent::Filed(wrote) = &landed.sent else { return };
    if let Some(why) = landed.refusal() {
        if let Wrote::Here { child, .. } = wrote {
            s.answering.holding.remove(&child.0);
        }
        s.refresh();
        s.refused = Some(why);
        return;
    }
    match wrote.clone() {
        // Z after it reopens the proposal, and says plainly that the message already went.
        Wrote::Here { child, title } => {
            if let Some(held) = s.answering.holding.get_mut(&child.0) {
                *held = true;
            }
            let thread = format!("{}/{}", child.1, child.0);
            let mut undo = write::Undo::plain(
                format!("Back as its own thread: {title}. The message already went."),
                format!("reopen “{title}”"),
                vec![(ANSWER, json!({ "product": child.1, "id": child.0, "status": "open" }))],
                Sent::TakenBack { thread },
                child,
                true,
            );
            undo.mark = Some(write::Mark::of(UNDID_HERE, None));
            s.undos.push(undo);
            s.refresh();
        }
        Wrote::All { act, kids, undo } => {
            let n = kids.len();
            let start = act == All::Start;
            let Some(first) = kids.first().cloned() else { return };
            let mut entry = write::Undo::plain(
                if start { format!("{n} approvals withdrawn") } else { format!("Reopened {n} proposals") },
                if start { format!("take back those {n} approvals and stop their agents") } else { format!("reopen those {n} proposals") },
                undo,
                Sent::TakenBack { thread: format!("{}/{}", first.1, first.0) },
                first,
                false,
            );
            entry.mark = Some(write::Mark::of(if start { palette::APPROVAL_UNDID } else { answering::UNDID_CLOSING }, None));
            entry.also = kids.iter().skip(1).map(|(id, product)| format!("{product}/{id}")).collect();
            s.undos.push(entry);
            s.say(format!("{} · press Z to undo", if start { format!("Started {n} threads") } else { format!("Dismissed {n} proposals") }), false, tx);
            s.refresh();
        }
    }
}

/// A picture taken after one of the list's presses says so in its fixture (`filedPressed`: the
/// press's key), and the same press is made here, with nobody to send to.
pub fn picture(s: &mut State, tx: &mut Tx) {
    let Some(pressed) = s.fixture.remove("filedPressed").and_then(|p| p.as_array().cloned()) else { return };
    for key in pressed.iter().filter_map(Value::as_str) {
        let Some(item) = s.open_item().cloned() else { return };
        let made = rows(s, &item);
        let find = |prefix: &str| key.strip_prefix(prefix).and_then(|id| made.iter().find(|r| r.id == id)).map(|r| (r.id.clone(), r.product.clone()));
        if let Some((id, product)) = find("made-approve-") {
            s.approve_filed(&id, &product, tx);
        } else if let Some((id, product)) = find("made-here-") {
            s.do_filed_here(&id, &product, tx);
        } else if let Some((id, product)) = find("made-dismiss-") {
            s.dismiss_filed(&id, &product, tx);
        }
    }
}

#[cfg(test)]
mod tests {
    use rune::{Key, Runtime};
    use serde_json::json;

    use super::*;
    use crate::feel::tests::{key, press, window_dressed};

    /// The invented inbox's first thread, whose agent filed `kids` (each `(id, title, more)`),
    /// opened; whatever is sent is taken as sent.
    pub(crate) fn filing(name: &str, kids: &[(&str, &str, Value)]) -> Runtime<State> {
        let kids: Vec<(String, String, Value)> = kids.iter().map(|(a, b, c)| (a.to_string(), b.to_string(), c.clone())).collect();
        let mut rt = window_dressed(name, &["--thread", "w-feel0"], "/fixtures/storefront", "Done.", move |fixture| {
            let now = fixture["now"].as_f64().unwrap();
            let items = fixture["snapshot"]["items"].as_array_mut().unwrap();
            for (n, (id, title, more)) in kids.iter().enumerate() {
                let mut kid = json!({ "id": id, "product": "storefront", "title": title, "status": "open", "kind": "task", "labels": [], "parent": "w-feel0", "createdAt": now - 60_000.0 + n as f64, "updatedAt": now - 60_000.0, "body": format!("{title}, as filed.") });
                kid.as_object_mut().unwrap().extend(more.as_object().cloned().unwrap_or_default());
                items.push(kid);
            }
        });
        rt.state.pretend_sent = true;
        rt
    }

    fn says(rt: &Runtime<State>, words: &str) -> bool {
        rt.elements().iter().any(|e| e.text.as_deref() == Some(words))
    }

    fn press_on(rt: &mut Runtime<State>, key: &str) {
        let (x, y) = rt.find(key).unwrap_or_else(|| panic!("{key} is not on the page"));
        press(rt, x, y);
    }

    /// The engine's answer to what the list sent: `sent` as the window files it.
    fn lands(rt: &mut Runtime<State>, sent: Sent, answer: Result<Value, String>) {
        rt.update(|s, tx| s.landed(Landed { sent, answer, snapshot: None }, tx));
        rt.frame(1.0 / 60.0);
    }

    fn kids_of(rt: &Runtime<State>, ids: &[&str]) -> Vec<Item> {
        ids.iter().map(|id| rt.state.snap.items.iter().find(|i| i.id == *id).cloned().unwrap()).collect()
    }

    #[test]
    fn a_thread_lists_what_it_filed_with_a_proposals_answers_beside_it() {
        let rt = filing("made-list", &[("w-kid1", "Say why Send is off", json!({})), ("w-kid2", "Keep a pasted command exactly as typed", json!({ "status": "claimed" }))]);
        for words in ["1 PROPOSAL WAITING ON YOU", "Say why Send is off", "Keep a pasted command exactly as typed", "DISMISS", "DO HERE", "NEW THREAD"] {
            assert!(says(&rt, words), "{words:?} is not on the page");
        }
        // One proposal: no answer for the lot. The proposal first, then the one under way.
        assert!(rt.find("made-all-start-w-kid1").is_none());
        let (top, under) = (rt.find("made-row-w-kid1").unwrap().1, rt.find("made-row-w-kid2").unwrap().1);
        assert!(top < under);
        assert!(rt.find("made-approve-w-kid2").is_none());
    }

    #[test]
    fn with_nothing_waiting_it_is_filed_from_this_thread_with_each_ones_tab() {
        let rt = filing("made-quiet", &[("w-kid1", "Say why Send is off", json!({ "status": "done" }))]);
        assert!(says(&rt, "FILED FROM THIS THREAD") && says(&rt, "DONE"));
        assert!(rt.find("made-approve-w-kid1").is_none());
    }

    #[test]
    fn new_thread_approves_it_as_its_own_thread_and_the_person_stays() {
        let mut rt = filing("made-approve", &[("w-kid1", "Say why Send is off", json!({}))]);
        press_on(&mut rt, "made-approve-w-kid1");
        assert!(matches!(&rt.state.view, View::Thread { id, .. } if id == "w-feel0"));
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Running → Say why Send is off"));
        rt.update(|s, _| s.write_what_waits());
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-kid1", "answer": "Approved. Run it." }))]);
    }

    #[test]
    fn do_here_hands_it_back_parent_first_and_z_after_reopens_it() {
        let mut rt = filing("made-here", &[("w-kid1", "Say why Send is off", json!({}))]);
        press_on(&mut rt, "made-here-w-kid1");
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Doing it in this thread: Say why Send is off"));
        assert!(rt.state.tried.is_empty(), "held three seconds");
        rt.update(|s, _| s.write_what_waits());
        // The parent's agent had finished, so the message wakes it as a reply does.
        let words = "Do this here, in this thread, not as a thread of its own: “Say why Send is off”\n\n, as filed.";
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-feel0", "answer": words, "status": "open" })), (ANSWER, json!({ "product": "storefront", "id": "w-kid1", "status": "done" }))]);
        let sent = Sent::Filed(Wrote::Here { child: ("w-kid1".to_string(), "storefront".to_string()), title: "Say why Send is off".to_string() });
        // Refused, the proposal is back in its list and the line says why.
        lands(&mut rt, sent.clone(), Ok(json!({ "ok": false, "error": "That thread is closed." })));
        assert_eq!(rt.state.refused.as_deref(), Some("That thread is closed."));
        assert!(rt.state.undos.is_empty());
        lands(&mut rt, sent, Ok(json!({ "ok": true })));
        let undo = rt.state.undos.last().expect("on Z's pile");
        assert_eq!(undo.said, "Back as its own thread: Say why Send is off. The message already went.");
        assert_eq!(undo.writes, vec![(ANSWER, json!({ "product": "storefront", "id": "w-kid1", "status": "open" }))]);
    }

    #[test]
    fn dismiss_closes_it_as_its_own_row_closes_and_the_person_stays() {
        let mut rt = filing("made-dismiss", &[("w-kid1", "Say why Send is off", json!({}))]);
        press_on(&mut rt, "made-dismiss-w-kid1");
        assert!(matches!(&rt.state.view, View::Thread { id, .. } if id == "w-feel0"));
        rt.update(|s, _| s.write_what_waits());
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-kid1", "status": "done" }))]);
    }

    #[test]
    fn start_all_and_dismiss_all_answer_the_lot_at_once_and_one_z_takes_it_back() {
        let kids = [("w-kid1", "Say why Send is off", json!({})), ("w-kid2", "Keep a pasted command exactly as typed", json!({})), ("w-kid3", "Rename the usage page", json!({ "status": "claimed" }))];
        let mut rt = filing("made-all", &kids);
        assert!(says(&rt, "2 PROPOSALS WAITING ON YOU") && says(&rt, "START ALL") && says(&rt, "DISMISS ALL"));
        press_on(&mut rt, "made-all-start-w-kid1");
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-kid1", "answer": "Approved. Run it." })), (ANSWER, json!({ "product": "storefront", "id": "w-kid2", "answer": "Approved. Run it." }))]);
        let sent = all_writes(&kids_of(&rt, &["w-kid1", "w-kid2"]), All::Start).1;
        lands(&mut rt, sent, Ok(json!({ "ok": true })));
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Started 2 threads · press Z to undo"));
        let undo = rt.state.undos.last().unwrap();
        assert_eq!(undo.said, "2 approvals withdrawn");
        assert_eq!(undo.writes, vec![
            (answering::STOP, json!({ "product": "storefront", "id": "w-kid1" })),
            (answering::STOP, json!({ "product": "storefront", "id": "w-kid2" })),
            (ANSWER, json!({ "product": "storefront", "id": "w-kid1", "answer": "(withdrawn)", "status": "open" })),
            (ANSWER, json!({ "product": "storefront", "id": "w-kid2", "answer": "(withdrawn)", "status": "open" })),
        ]);
        // Z takes the lot back, and each proposal says so.
        rt.state.tried.clear();
        key(&mut rt, Key::Text("z".into()));
        assert_eq!(rt.state.tried.len(), 4);
        let mut rt = filing("made-all-dismiss", &kids);
        press_on(&mut rt, "made-all-dismiss-w-kid1");
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-kid1", "status": "done" })), (ANSWER, json!({ "product": "storefront", "id": "w-kid2", "status": "done" }))]);
        let sent = all_writes(&kids_of(&rt, &["w-kid1", "w-kid2"]), All::Dismiss).1;
        lands(&mut rt, sent, Ok(json!({ "ok": true })));
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Dismissed 2 proposals · press Z to undo"));
        assert_eq!(rt.state.undos.last().unwrap().said, "Reopened 2 proposals");
    }

    #[test]
    fn a_rows_title_opens_that_thread() {
        let mut rt = filing("made-open", &[("w-kid1", "Say why Send is off", json!({}))]);
        press_on(&mut rt, "made-open-w-kid1");
        assert!(matches!(&rt.state.view, View::Thread { id, .. } if id == "w-kid1"));
    }
}
