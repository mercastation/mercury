//! The command bar: what ⌘K opens over the page (`PaletteBox` in
//! `renderer/src/components/Palette.tsx`, `.palette-k` in `renderer/src/styles.css`). A box to
//! search threads or type a command in, over the rows that answer it: the arrows and the
//! pointer move the lit row, Enter or a press runs it, Escape or a press outside closes it.
//!
//! Which rows there are is `rules/palette.rs`; here they are drawn and carried out, each through
//! what this window's own key, button or page already does, or the web row's own write. A row
//! whose doing needs a screen this window does not have is not offered
//! (`docs/parity-command-bar.md`).

use std::cell::RefCell;

use rune::edit::Key;
use rune::text::Span;
use rune::{Color, Cursor, El, KeyEvent, Latest, ScrollSpec, TextEdit, TextStyle, Tx, Waker, col, el, icon, row, text, text_input};
use serde_json::{Map, Value};

use crate::data::Item;
use crate::rules::found::{self, Group};
use crate::rules::palette::{self as rules, Does};
use crate::rules::place::Act;
use crate::rules::{js, search, writes};
use crate::source::Source;
use crate::theme::*;
use crate::{State, View, front, icons, source, write};

mod marks;

/// New thread. The web's bar has it second, after New project, and N does the same
/// (`onNewThread` in `Palette.tsx`): it opens the New thread card over the page, Home included,
/// with the caret in its words (`compose.rs`). A row that would do nothing is never drawn, so
/// this answers what the row does, or nothing to leave it out.
fn starts_a_thread() -> Option<fn(&mut State, &mut Tx)> {
    Some(State::open_card)
}

/// The bar's own state. It is always there; `open` says whether it is on screen.
pub struct Bar {
    pub open: bool,
    pub words: TextEdit,
    /// The row the arrows or the pointer lit, by its key, and the words it was lit under.
    /// New words light the first row again; the same words keep the same row lit, wherever a
    /// list that changed under it has moved it (`pickedKey` in `Palette.tsx`).
    lit: Option<(String, String)>,
    /// Where the pointer last moved over the rows, since the bar opened.
    pointer: Option<(f32, f32)>,
    /// What every thread is searched over, kept between keystrokes.
    index: RefCell<search::Index>,
    /// The web window's own rows, when a fixture brings them for a picture.
    pictured: Option<Vec<Pictured>>,
    /// A thread moved to another project from the open conversation: its copy, opened once the
    /// lists have it, if they do within ten seconds (`opensWhenListed`).
    opens_when_listed: Option<((String, String), f64)>,
    /// What the app last found inside threads for the words, as rows, which stay up until the
    /// next answer lands (`inside` in `PaletteBox`).
    inside: Vec<Group>,
    /// The asking, kept from one opening of the bar to the next, as its waker is.
    finder: Finder,
}

/// The app asked what the words find inside threads, off the main thread: 90 ms after the words
/// last changed, an answer to older words dropped however late it lands (`Latest`); and asked
/// to read the threads ahead as the bar opens, its answer never waited for.
pub struct Finder {
    asking: Latest<(String, Result<Value, String>)>,
    warming: Latest<()>,
}

impl Default for Finder {
    fn default() -> Finder {
        Finder { asking: Latest::default().debounce(std::time::Duration::from_millis(found::ASK_AFTER_MS)), warming: Latest::default() }
    }
}

impl Finder {
    pub fn set_waker(&self, wake: Waker) {
        self.asking.set_waker(wake);
    }
    /// Asks what `query` finds, through `call`, once the words have rested; any answer still to
    /// come for older words is dropped.
    fn ask(&self, query: String, payload: Value, call: impl FnOnce(Value) -> Result<Value, String> + Send + 'static) {
        self.asking.request((query, payload), move |(query, payload)| (query, call(payload)));
    }
    /// Nothing more is asked, and nothing asked lands.
    fn stop(&self) {
        self.asking.cancel();
    }
    fn take(&self) -> Option<(String, Result<Value, String>)> {
        self.asking.take()
    }
    /// Has the app read the threads ahead; what it answers is not waited for, and lands nowhere.
    fn warm(&self, call: impl FnOnce() + Send + 'static) {
        self.warming.request_opt((), move |()| {
            call();
            None
        });
    }
}

impl Bar {
    pub fn closed() -> Bar {
        Bar {
            open: false,
            words: TextEdit::new(""),
            lit: None,
            pointer: None,
            index: RefCell::new(search::Index::default()),
            pictured: None,
            opens_when_listed: None,
            inside: Vec::new(),
            finder: Finder::default(),
        }
    }
    pub fn set_waker(&self, wake: Waker) {
        self.finder.set_waker(wake);
    }

    /// A picture taken with the bar open says so in its fixture: `palette` is what was typed
    /// into it. The web's bar lists commands this window cannot do yet, so a picture of the two
    /// is only like for like when this one draws the commands that one had: the fixture carries
    /// the web's rows as its page drew them (`paletteRows`, read off the page by the scene).
    /// Its commands are drawn as they were and do nothing; where it listed a thread, this
    /// window lists the next thread its own search found for the same words.
    pub fn picture(&mut self, fixture: &Map<String, Value>) {
        let Some(words) = fixture.get("palette").and_then(Value::as_str) else {
            return;
        };
        self.open = true;
        self.words = TextEdit::new(words);
        self.words.focused = true;
        self.pictured = fixture.get("paletteRows").and_then(Value::as_array).map(|rows| rows.iter().filter_map(Pictured::read).collect());
    }

    fn light(&mut self, key: &str) {
        self.lit = Some((self.words.text.clone(), key.to_string()));
    }
}

/// A picture taken with the bar open (`Bar::picture`), and what the app found inside threads
/// for its words, as the fixture carries the answer (`zero:find-in-threads`), as though it had
/// just landed.
pub fn picture(s: &mut State) {
    s.palette.picture(&s.fixture);
    if !s.palette.open {
        return;
    }
    if let Some(answer) = s.fixture.get(found::FIND) {
        s.palette.inside = found::groups(&s.palette.words.text, &found::Answer::read(answer), &s.snap);
    }
    settle_lit(s);
}

/// A row of the web window's bar, as a fixture carries it.
#[derive(Clone)]
enum Pictured {
    Heading(String),
    /// Where it listed a thread.
    Thread,
    /// Where it listed something found inside a thread.
    Found,
    /// A command, with the web's name for the drawing at its head.
    Command { icon: String, label: String, hint: Option<String>, cap: Option<String> },
}

impl Pictured {
    fn read(row: &Value) -> Option<Pictured> {
        let words = |field: &str| row.get(field).and_then(Value::as_str).filter(|s| !s.is_empty()).map(str::to_string);
        if let Some(heading) = words("section") {
            return Some(Pictured::Heading(heading));
        }
        if row.get("thread").and_then(Value::as_bool) == Some(true) {
            return Some(Pictured::Thread);
        }
        if row.get("found").and_then(Value::as_bool) == Some(true) {
            return Some(Pictured::Found);
        }
        Some(Pictured::Command { icon: words("icon").unwrap_or_else(|| rules::PLAIN_ICON.to_string()), label: words("label")?, hint: words("hint"), cap: words("key") })
    }
}

/// One row as it is drawn, and what running it does.
#[derive(Clone)]
struct Line {
    key: String,
    /// The heading over it, when a section starts at it.
    heading: Option<String>,
    /// The web's name for the drawing at its head (`iconFor`).
    icon: String,
    label: String,
    hint: Option<String>,
    /// The key that does the same, drawn in place of a hint.
    cap: Option<String>,
    sub: Option<String>,
    done: bool,
    /// Something found inside a thread: what it says, the typed words lit in it, over who, where
    /// and when (`c.found` in `PaletteBox`).
    found: Option<found::Found>,
    /// What pressing it does. A command the web window's picture had does nothing.
    does: Option<Does>,
}

impl Line {
    fn of(row: rules::Row, key: String, heading: Option<String>) -> Line {
        Line {
            key,
            heading,
            icon: rules::icon_for(&row.id, &row.label).to_string(),
            label: row.label,
            // A key says more than a hint, and a row has room for one of them.
            hint: row.hint.filter(|_| row.key.is_none()),
            cap: row.key.map(str::to_string),
            sub: row.sub,
            done: row.done,
            found: None,
            does: Some(row.does),
        }
    }
    /// A thing found inside a thread, with no hint or key, never dimmed, its drawing by its id
    /// (`insideForPalette`).
    fn found(row: found::Found, key: String, heading: Option<String>) -> Line {
        Line {
            key,
            heading,
            icon: rules::icon_for(&row.id, &row.text).to_string(),
            label: row.text.clone(),
            hint: None,
            cap: None,
            sub: None,
            done: false,
            does: Some(Does::Found(row.open.clone())),
            found: Some(row),
        }
    }
}

/// The rows of what was found inside threads, each group's first under its heading, after
/// every other row (`labels` in `PaletteBox`).
fn found_lines(groups: &[Group]) -> Vec<Line> {
    groups.iter().flat_map(|g| g.rows.iter().enumerate().map(move |(at, row)| Line::found(row.clone(), row.id.clone(), (at == 0).then(|| g.label.to_string())))).collect()
}

/// The web window's rows as a fixture carries them, with this window's own threads where it
/// listed threads. A thread it found that the web did not list goes at the end, so the
/// picture shows it too. A command this window has too, by its words and its drawing, is its
/// own row, which does what it says; one it does not have is drawn as it was and does nothing.
fn pictured_lines(pictured: &[Pictured], threads: Vec<rules::Row>, own: Vec<Line>) -> Vec<Line> {
    let keys = rules::row_keys(&threads);
    let mut found = threads.into_iter().zip(keys);
    // What this window found inside threads goes where the web's found things went, in order.
    let (inside, own): (Vec<Line>, Vec<Line>) = own.into_iter().partition(|l| l.found.is_some());
    let mut inside = inside.into_iter();
    let mut own: Vec<Option<Line>> = own.into_iter().map(Some).collect();
    let (mut lines, mut heading) = (Vec::new(), None);
    for (at, row) in pictured.iter().enumerate() {
        match row {
            Pictured::Heading(words) => heading = Some(words.clone()),
            Pictured::Thread => {
                if let Some((thread, key)) = found.next() {
                    lines.push(Line::of(thread, key, heading.take()));
                }
            }
            Pictured::Found => {
                if let Some(line) = inside.next() {
                    lines.push(Line { heading: heading.take(), ..line });
                }
            }
            Pictured::Command { icon, label, hint, cap } => {
                let mine = own.iter_mut().find(|l| l.as_ref().is_some_and(|l| l.sub.is_none() && &l.label == label && &l.icon == icon)).and_then(Option::take);
                lines.push(match mine {
                    Some(line) => Line { heading: heading.take(), ..line },
                    None => Line {
                        key: format!("pictured-{at}"),
                        heading: heading.take(),
                        icon: icon.clone(),
                        label: label.clone(),
                        hint: hint.clone(),
                        cap: cap.clone(),
                        sub: None,
                        done: false,
                        found: None,
                        does: None,
                    },
                });
            }
        }
    }
    lines.extend(found.map(|(thread, key)| Line::of(thread, key, None)));
    lines.extend(inside);
    lines
}

/// The thread the keys are on, and whether it is the conversation on screen: the open
/// conversation, or the Inbox's row under the pointer, else its picked row, the one E acts on
/// (`focused ?? pointed` in `App.tsx`). Home has none. The row the app makes itself when tasks
/// have not been able to run is in no ledger, and is read off the lists (`trouble.rs`).
fn under_the_keys(s: &State) -> Option<(&Item, bool)> {
    let made = |id: &str, product: &str| s.lists.trouble.as_ref().filter(|t| t.id == id && t.product == product);
    match &s.view {
        View::Thread { id, product } => s.open_item().or_else(|| made(id, product)).map(|item| (item, true)),
        View::Inbox => crate::keys::picked_row(s).and_then(|(id, product)| s.item(&id, &product).or_else(|| made(&id, &product))).map(|item| (item, false)),
        View::Home | View::Files { .. } => None,
    }
}

/// This window's own rows for what is typed, in the order drawn, whatever a picture's fixture
/// says the web window drew.
fn own_lines(s: &State, threads: Vec<rules::Row>) -> Vec<Line> {
    let query = s.palette.words.text.as_str();
    let projects = s.files.projects();
    let facts = rules::Facts {
        snap: &s.snap,
        local: &s.local,
        now: s.now,
        models: &s.models,
        file_projects: &projects,
        key_hints: s.picks.get().key_hints.unwrap_or(true),
        starts_threads: starts_a_thread().is_some(),
    };
    let under = under_the_keys(s);
    let mut commands = rules::commands(&facts, under);
    // Try again, after Reply, on a thread nothing will start without the person (`foot.rs`).
    if let Some((item, _)) = under.filter(|(item, _)| crate::foot::offered_on_the_bar(s, item)) {
        rules::with_try_again(&mut commands, (item.id.clone(), item.product.clone()));
    }
    // Stop, on a thread under way: after the thread's rows, before Open and the places.
    if let Some((item, _)) = under.filter(|(item, _)| s.stoppable_now(item)) {
        let at = commands.iter().position(|r| r.id == "open" || r.id == "view-home").unwrap_or(commands.len());
        commands.insert(at, rules::stop_row(s.run_on(item), (item.id.clone(), item.product.clone())));
    }
    // The code its run changed, on the conversation whose page draws the button for it.
    if let Some((item, true)) = under
        && crate::change::has_a_change(s, item)
    {
        rules::with_code_row(&mut commands, (item.id.clone(), item.product.clone()));
    }
    let listed = rules::listed(commands, threads, query);
    let headings: Vec<Option<String>> = (0..listed.rows.len()).map(|at| listed.label_over(at).map(str::to_string)).collect();
    let mut ids: Vec<String> = listed.rows.iter().map(|row| row.id.clone()).collect();
    let mut lines: Vec<Line> = listed.rows.into_iter().zip(headings).map(|(row, heading)| Line::of(row, String::new(), heading)).collect();
    // What was found inside threads, after everything else.
    let inside = found_lines(&s.palette.inside);
    ids.extend(inside.iter().map(|line| line.key.clone()));
    lines.extend(inside);
    // One key each, over the whole list, as the web keys it (`commandKeys(filtered)`).
    for (line, key) in lines.iter_mut().zip(rules::keys_of(ids.iter().map(String::as_str))) {
        line.key = key;
    }
    lines
}

/// The threads the words find, from the search kept between keystrokes.
fn threads(s: &State) -> Vec<rules::Row> {
    let mut index = s.palette.index.borrow_mut();
    index.read(&s.snap.items);
    rules::threads_found(&index, &s.snap, &s.palette.words.text, s.now)
}

/// The rows the bar lists for what is typed, in the order drawn.
fn lines(s: &State) -> Vec<Line> {
    let own = own_lines(s, threads(s));
    match &s.palette.pictured {
        Some(pictured) => pictured_lines(pictured, threads(s), own),
        None => own,
    }
}

/// Which row is lit: the one picked under these same words, else the first (`keptPick`).
fn lit(s: &State, lines: &[Line]) -> Option<usize> {
    let keys: Vec<String> = lines.iter().map(|line| line.key.clone()).collect();
    let picked = s.palette.lit.as_ref().filter(|(under, _)| *under == s.palette.words.text).map(|(_, key)| key.as_str());
    rules::kept_pick(picked, &keys)
}

/// With no row picked under these words, the first row there is is picked, and then kept by its
/// key as rows come in under it: as the bar opens, as the words change, and when what was found
/// inside threads lands, which lights its first row when nothing else was listed
/// (`if (pickedKey === null && keys.length) setPickedKey(keys[0])`).
fn settle_lit(s: &mut State) {
    if s.palette.lit.as_ref().is_some_and(|(under, _)| *under == s.palette.words.text) {
        return;
    }
    if let Some(first) = lines(s).first() {
        let key = first.key.clone();
        s.palette.light(&key);
    }
}

/// The door, when this window is on the app's: who is asked.
fn door(s: &State) -> Option<(String, String)> {
    match &s.source {
        Source::Door(door) => door.now(),
        _ => None,
    }
}

/* ------------------------------------ what it does ------------------------------------ */

impl State {
    /// ⌘K: opens the bar, or closes it when it is open (`setModal` in `App.tsx`'s keydown).
    pub fn toggle_palette(&mut self, tx: &mut Tx) {
        if self.palette.open { self.close_palette() } else { self.open_palette(tx) }
    }

    /// Opens the bar empty, with the caret in its box. One thing stands over the page at a
    /// time, as in the web window, where the bar, the Later picker and the reply box are one
    /// `modal`: the picker closes and the reply box folds, keeping its words.
    pub fn open_palette(&mut self, tx: &mut Tx) {
        self.close_later();
        self.fold_reply();
        self.close_strip();
        self.palette.finder.stop();
        // What a thread's row says of when it comes back is counted from this minute.
        if !self.fixed_clock {
            self.now = crate::clock();
        }
        // Fresh each time, as the web window mounts its box anew: empty, the first row lit, the
        // list at its top. Rune would otherwise carry out again the last request to bring a lit
        // row into view, which outlives the list it was made for.
        self.palette = Bar {
            open: true,
            index: std::mem::take(&mut self.palette.index),
            opens_when_listed: self.palette.opens_when_listed.take(),
            finder: std::mem::take(&mut self.palette.finder),
            ..Bar::closed()
        };
        self.palette.words.focused = true;
        tx.focus_when_shown("palette", false);
        tx.scroll_to_y("palette-list", 0.0);
        settle_lit(self);
        // The app reads the threads ahead, so the first words typed are answered at once
        // (`warmFindInThreads` as the bar opens).
        if let Some((url, token)) = door(self) {
            self.palette.finder.warm(move || {
                let _ = source::call(&url, &token, found::WARM, &Value::Null);
            });
        }
    }

    pub fn close_palette(&mut self) {
        // Nothing asked for its words lands once it is closed (the box unmounts).
        self.palette.finder.stop();
        self.palette = Bar {
            index: std::mem::take(&mut self.palette.index),
            opens_when_listed: self.palette.opens_when_listed.take(),
            finder: std::mem::take(&mut self.palette.finder),
            ..Bar::closed()
        };
    }

    /// The words in the box changed: the first row is lit under them, and 90 ms after they
    /// last change the app is asked what they find inside threads; words blank once trimmed
    /// take what was found away at once and ask nothing (`useEffect(…, [query])`).
    pub fn palette_typed(&mut self) {
        settle_lit(self);
        let words = self.palette.words.text.clone();
        let Some(payload) = found::ask(&words) else {
            self.palette.finder.stop();
            self.palette.inside.clear();
            return;
        };
        if let Some((url, token)) = door(self) {
            self.palette.finder.ask(js::trim(&words).to_string(), payload, move |payload| source::call(&url, &token, found::FIND, &payload));
        }
    }

    /// What the app found for words asked: the rows it makes stand until the next answer, and
    /// a call that failed is said nowhere and leaves the rows as they were.
    fn found_inside(&mut self, query: &str, answer: Result<Value, String>) -> bool {
        let Ok(answer) = answer else { return false };
        if !self.palette.open {
            return false;
        }
        self.palette.inside = found::groups(query, &found::Answer::read(&answer), &self.snap);
        settle_lit(self);
        true
    }

    /// Reopen, from the bar and from the Reopen button: the thread opened again with nothing
    /// said, written at once, and Z marks it done again (`reopenDone`).
    pub(crate) fn reopen_done(&mut self, item: &Item) {
        self.write_what_waits();
        let (said, again) = rules::reopened_said(item);
        let (channel, payload) = rules::reopen_done(item);
        self.send(channel, payload, write::Sent::Palette(Wrote::Reopened { thread: thread_of(item), said, again }));
    }

    /// Approve: the agent's recommended option, the go-ahead on a review, or the word to run a
    /// proposal, held three seconds as an answer is, and from inside the thread the person goes
    /// on to the next that needs them (`resolve`, through `deferCommit` and `leaveResolved`).
    fn approve(&mut self, item: &Item, tx: &mut Tx) {
        let Some(approval) = rules::approval(item) else {
            return self.close(&item.id, &item.product, false, tx);
        };
        self.approve_as(item, approval, tx);
    }

    /// The approval itself, as `resolve` makes it: also New thread beside a thread another
    /// filed, from which the person stays on the thread they pressed it from (`made.rs`).
    pub(crate) fn approve_as(&mut self, item: &Item, approval: rules::Approval, tx: &mut Tx) {
        let thread = thread_of(item);
        let inside = self.is_open(&thread);
        let next = next_after(self, &thread);
        // An answer as a picked option is: drawn from the press on, and once written on Z's
        // pile, where Z stops the agent and takes the approval back (`approveWith`'s `pushUndo`).
        let key = format!("{}/{}", item.product, item.id);
        let drawn = self.answering.draw(&key, self.moment(), &approval.answer());
        let mark = write::Mark::of(rules::APPROVAL_UNDID, None);
        let answer = write::Answer {
            drawn,
            running: false,
            undo: rules::approval_withdrawn(item),
            undo_said: rules::APPROVAL_WITHDRAWN.to_string(),
            undoes: rules::APPROVAL_UNDOES.to_string(),
            mark: mark.clone(),
            moved_on: inside,
            in_flight: 0.0,
        };
        self.hold(
            write::Held {
                channel: writes::ANSWER,
                payload: rules::approve(item, &approval),
                sent: write::Sent::Pick { thread: key, answer },
                said: approval.said(item, &self.project_name(item)),
                at: tx.clock() + writes::UNDO_GRACE_MS / 1000.0,
                thread: thread_of(item),
                from_inside: inside,
                closes: None,
                then: Vec::new(),
                mark: Some(mark),
            },
            tx,
        );
        if inside {
            self.advance(next);
        }
    }

    /// Whether the thread `(id, product)` is the conversation on screen, whether or not the
    /// engine still lists it.
    fn is_open(&self, (id, product): &(String, String)) -> bool {
        matches!(&self.view, View::Thread { id: open, product: of } if open == id && of == product)
    }

    /// From inside a thread just taken out of Needs you, on to `next`, or back where it was
    /// opened from (`advanceAfter`, as a close does).
    fn advance(&mut self, next: Option<(String, String)>) {
        match next {
            Some((id, product)) => self.step_to_thread(&id, &product),
            None => self.leave_thread(),
        }
    }

    /// Pins a thread, or lets it go: shown at once, written after, and said (`pinThread` from
    /// the menu). Letting it go forgets a place it was dragged to as well.
    fn pin(&mut self, item: &Item, pin: bool, tx: &Tx) {
        if let Some(row) = self.snap.items.iter_mut().find(|i| i.id == item.id && i.product == item.product) {
            row.rest.insert("pinned".to_string(), Value::Bool(pin));
            if !pin {
                row.rest.remove("place");
            }
        }
        self.refresh();
        self.write_what_waits();
        self.send_all(rules::pin(item, pin), write::Sent::Palette(Wrote::Quiet));
        self.tell(if pin { "Pinned to the top of your list" } else { "Unpinned" }, tx);
    }

    /// Where it runs, as the place button's panel picks it, said here since no panel shows it
    /// (`placeThread` with `said`).
    fn place(&mut self, item: &Item, act: Act, tx: &Tx) {
        self.write_what_waits();
        let writes = crate::rules::place::writes(&self.snap, item, act);
        match act {
            Act::Move => {
                self.send_all(writes, write::Sent::Palette(Wrote::Moving));
            }
            Act::Keep(keep) => {
                self.send_all(writes, write::Sent::Palette(Wrote::Quiet));
                self.tell(if keep { "This thread now runs only on this Mac" } else { "This thread can run on the cloud again" }, tx);
            }
            Act::CloudOn => {
                self.send_all(writes, write::Sent::Palette(Wrote::Quiet));
            }
        }
    }

    /// One write a row makes, on its own channel, its answer said in `landed` below.
    fn palette_write(&mut self, write: writes::Write, wrote: Wrote) {
        self.write_what_waits();
        self.send(write.0, write.1, write::Sent::Palette(wrote));
    }

    /// The sidebar's Move to Applications (`api.updateMove` from `SidebarMove`): the bar's own
    /// write, with nothing said of its answer, since a move that worked relaunches the app and
    /// one that did not is said by the card it was pressed on (`moveError`).
    pub(crate) fn move_to_applications(&mut self) {
        self.palette_write(("zero:update-move", Value::Null), Wrote::Quiet);
    }
}

fn thread_of(item: &Item) -> (String, String) {
    (item.id.clone(), item.product.clone())
}

/// The thread after this one in Needs you, or the one before, while it is still in the list.
fn next_after(s: &State, (id, product): &(String, String)) -> Option<(String, String)> {
    let needs = &s.lists.needs;
    let at = needs.iter().position(|r| &r.id == id && &r.product == product)?;
    needs.get(at + 1).or_else(|| at.checked_sub(1).and_then(|a| needs.get(a))).map(|r| (r.id.clone(), r.product.clone()))
}

/// What a row of the bar wrote, kept until the engine answers, for what is then said and done.
#[derive(Clone, Debug, PartialEq)]
pub enum Wrote {
    /// Nothing is said of it but a refusal.
    Quiet,
    /// Reopened: the line to say, and the one Z says when it marks it done again.
    Reopened { thread: (String, String), said: String, again: String },
    RunNow { thread: (String, String) },
    /// Marked unread: held here until the web window, which keeps what has been read, says so too.
    Unread { thread: (String, String) },
    /// An agent put back on it; `next` is the thread after it in Needs you as it was pressed,
    /// where the person goes on to from inside it once the agent is on it.
    Resume { thread: (String, String), next: Option<(String, String)> },
    /// Its next model: the line to say once it stands.
    Model { said: String },
    /// Moved to the project `to`, called `name`; `from` is the one it was in.
    Move { thread: (String, String), to: String, name: String, from: String },
    SendBack { thread: (String, String), project: String, next: Option<(String, String)> },
    Redeliver { thread: (String, String), project: String },
    /// Its run carried to the cloud.
    Moving,
    /// A project put first, by its name.
    Ranked { name: String },
    ResumeStopped,
    UpdateMove,
    UpdateCheck,
}

/// Runs a row: the bar closes first, then the row does what it says, each through what the
/// key or the button for it already does, or the web row's own write. The one row that does
/// nothing leaves the bar open, as the web's does: its press is a no-op that does not close it.
fn run(s: &mut State, does: Does, tx: &mut Tx) {
    if does == Does::Nothing {
        return;
    }
    s.close_palette();
    let item = |s: &State, (id, product): &(String, String)| s.item(id, product).cloned();
    match does {
        Does::NewProject => s.new_project(tx),
        Does::NewThread => {
            if let Some(start) = starts_a_thread() {
                start(s, tx);
            }
        }
        Does::Home => s.go(View::Home),
        // Out of an open conversation, as the rail's own Inbox button goes (`openView`).
        Does::Tab(tab) => {
            s.go(View::Inbox);
            s.show_tab(tab);
        }
        Does::Open((id, product)) => s.open_thread(&id, &product),
        Does::Reply((id, product)) => {
            if !s.open_item().is_some_and(|item| item.id == id && item.product == product) {
                s.open_thread(&id, &product);
            }
            s.open_reply(tx);
        }
        Does::Done((id, product)) => s.close(&id, &product, false, tx),
        Does::Reopen(thread) => {
            if let Some(item) = item(s, &thread) {
                s.reopen_done(&item);
            }
        }
        Does::Later((id, product)) => s.open_later(&id, &product, tx),
        Does::Back((id, product)) => s.bring_back(&id, &product),
        Does::Files(project) => s.open_files_place(project),
        Does::TryAgain((id, product)) => s.try_again(&id, &product, tx),
        Does::Approve(thread) => {
            if let Some(item) = item(s, &thread) {
                s.approve(&item, tx);
            }
        }
        Does::RunNow(thread) => {
            if let Some(item) = item(s, &thread) {
                s.palette_write(rules::run_now(&item), Wrote::RunNow { thread });
            }
        }
        Does::Resume(thread) => {
            if let Some(item) = item(s, &thread) {
                let next = next_after(s, &thread);
                s.palette_write(rules::resume(&item), Wrote::Resume { thread, next });
            }
        }
        Does::MarkRead((id, product)) => s.saw(&id, &product),
        // Unread here at once, and in the web window, which keeps what has been read, once it
        // has been asked: nothing is said unless it would not (`markUnseen`).
        Does::MarkUnread(thread) => {
            if let Some(item) = item(s, &thread) {
                s.unsaw(&item.id);
                s.palette_write(rules::mark_unread(&item), Wrote::Unread { thread });
            }
        }
        Does::Pin(thread, pin) => {
            if let Some(item) = item(s, &thread) {
                s.pin(&item, pin, tx);
            }
        }
        Does::Place(thread, act) => {
            if let Some(item) = item(s, &thread) {
                s.place(&item, act, tx);
            }
        }
        Does::Model(thread, pick) => {
            if let Some(item) = item(s, &thread) {
                let said = format!("{} picks this up next", crate::rules::compose::model_label(&pick, &s.models));
                s.palette_write(rules::hand_to_model(&item, &pick), Wrote::Model { said });
            }
        }
        Does::Move(thread, to) => {
            if let Some(item) = item(s, &thread) {
                let name = s.snap.products.iter().find(|p| p.slug == to).map_or_else(|| to.clone(), |p| p.name.clone());
                let from = s.project_name(&item);
                s.palette_write(rules::move_item(&item, &to), Wrote::Move { thread, to, name, from });
            }
        }
        Does::CannotMove(why) => s.tell(&why, tx),
        Does::SendBack(thread) => {
            if let Some(item) = item(s, &thread) {
                let project = s.project_name(&item);
                let next = next_after(s, &thread);
                s.palette_write(rules::send_back(&item), Wrote::SendBack { thread, project, next });
            }
        }
        Does::Redeliver(thread) => {
            if let Some(item) = item(s, &thread) {
                let project = s.project_name(&item);
                s.palette_write(rules::redeliver(&item), Wrote::Redeliver { thread, project });
            }
        }
        // Every level is written, the one it is at included (`retag` from the bar).
        Does::Priority(thread, level) => {
            if let Some(item) = item(s, &thread) {
                s.retag(&item, level);
            }
        }
        // A page this window draws opens here; one it does not is the web window's, which comes
        // forward on it, as this window's own Settings sends it there.
        Does::Settings(page) => {
            if page.native() {
                crate::settings::open(s, page, tx);
            } else {
                crate::settings::bridge(s, page, tx);
            }
        }
        Does::RankFirst(slug) => {
            let name = s.snap.products.iter().find(|p| p.slug == slug).map_or_else(|| slug.clone(), |p| p.name.clone());
            let write = rules::rank_first(&s.snap, &slug);
            s.palette_write(write, Wrote::Ranked { name });
        }
        // Off asks first, as ⌘C does (`cloudOffAsks`); on is written and says nothing.
        Does::Cloud(mode) => {
            if mode == crate::rules::cloud::Mode::Mac {
                s.head.asking_off = true;
            } else {
                s.pick_cloud(crate::rules::cloud::set_mode(mode));
            }
        }
        Does::InstallUpdate => s.ask_the_app(front::INSTALL_UPDATE, "Mercury could not restart onto the new version. Try again in a moment.", tx),
        Does::MoveToApplications => s.palette_write(("zero:update-move", Value::Null), Wrote::UpdateMove),
        // It says it is looking at once, and what it found once the engine has said.
        Does::CheckForUpdates => {
            s.tell("Looking for a new version.", tx);
            s.palette_write(("zero:update-check", Value::Null), Wrote::UpdateCheck);
        }
        Does::Pause(paused) => s.palette_write(rules::pause(paused), Wrote::Quiet),
        Does::ResumeStopped => s.palette_write(rules::resume_stopped(), Wrote::ResumeStopped),
        // A plate left over from before the switch is let go, and so is the row the pointer
        // was keeping for the keys (`setHints`).
        Does::KeyHints(on) => {
            s.picks.change(|p| p.key_hints = Some(on));
            if !on {
                s.pointed.hint = None;
                s.pointed.row = None;
            }
        }
        Does::Stop((id, product)) => s.stop_agent(&id, &product),
        Does::Nothing => {}
        // It opens the change, and never closes one already open (`setOpenDoc`).
        Does::Code((id, _)) => s.open_change(&crate::rules::change::change_path_for(&id), None),
        Does::Found(open) => open_found(s, open),
    }
}

/// Something found inside a thread, pressed: the thread opens and is counted as read; a line
/// is then looked for and landed on, and a file opens beside the conversation when it is one
/// that opens there and the row is not an outside agent's, else big over it (`openInside`).
fn open_found(s: &mut State, open: found::Open) {
    match open {
        found::Open::Line { thread: (id, product), uid } => {
            s.open_thread(&id, &product);
            s.reveal_line(&id, &product, &uid);
        }
        found::Open::File { thread: (id, product), src, outside } => {
            s.open_thread(&id, &product);
            if !outside && crate::rules::pane::where_a_file_opens(&src) == Some(crate::rules::pane::Opens::Beside) {
                s.open_beside(&src, false);
            } else {
                s.open_big(&src, None);
            }
        }
    }
}

/// What the engine said to a row's write: said in the web window's words, and what then
/// follows done (`App.tsx`'s handlers for each).
pub(crate) fn landed(s: &mut State, landed: write::Landed, tx: &mut Tx) {
    let refused = landed.refusal();
    let write::Sent::Palette(wrote) = landed.sent else {
        return;
    };
    // A mark of unread the app did not take is taken back here too, whatever kept it from
    // taking it; one it took says nothing, as the web's row says nothing.
    if let Wrote::Unread { thread } = &wrote {
        if let Some(why) = refused {
            s.unsaw_refused(&thread.0);
            s.tell(&rules::unread_refused(&why), tx);
        }
        return;
    }
    let answer = match landed.answer {
        Ok(answer) => answer,
        Err(why) => {
            s.refused = Some(format!("Could not do that: {why}"));
            return;
        }
    };
    let refusal = || answer.get("ok").filter(|ok| **ok == Value::Bool(false)).map(|_| answer.get("error").and_then(Value::as_str).unwrap_or("Mercury did not take it.").to_string());
    match wrote {
        Wrote::Quiet => {
            if let Some(why) = refusal() {
                s.refused = Some(format!("Could not do that: {why}"));
            }
        }
        Wrote::Reopened { thread, said, again } => {
            if let Some(why) = refusal() {
                return s.tell(&format!("Not reopened: {why}"), tx);
            }
            // On Z's pile, with what Z asks once it is half a minute old (`reopenDone`'s `undoes`).
            let undoes = format!("mark “{}” done again", again.strip_prefix("Done again: ").unwrap_or(&again));
            s.undos.push(write::Undo::plain(again, undoes, vec![rules::done_again(&thread)], write::Sent::Palette(Wrote::Quiet), thread, true));
            let (line, undo) = crate::rules::compose::toast_parts(&said);
            s.say(line, undo, tx);
        }
        // Answered above, before the engine's words were read.
        Wrote::Unread { .. } => {}
        Wrote::RunNow { thread } => {
            let said = rules::run_now_said(&answer);
            if answer.get("ok") == Some(&Value::Bool(true)) { s.say_about(said, thread, tx) } else { s.tell(said, tx) }
        }
        Wrote::Resume { thread, next } => {
            let (said, names, started) = rules::resumed_said(&answer);
            if names { s.say_about(&said, thread.clone(), tx) } else { s.tell(&said, tx) }
            // Put back to work, it has left Needs you, and from inside it the person goes on.
            if started && s.is_open(&thread) {
                s.advance(next);
            }
        }
        Wrote::Model { said } => match refusal() {
            Some(why) => s.refused = Some(format!("Could not change its model: {why}")),
            None => s.tell(&said, tx),
        },
        Wrote::Move { thread, to, name, from } => {
            let made = answer.get("item").filter(|m| m.is_object());
            let (Some(made), None) = (made, refusal()) else {
                let why = answer.get("error").and_then(Value::as_str).unwrap_or("That thread could not be moved.");
                return s.tell(why, tx);
            };
            let copy = (made.get("id").and_then(Value::as_str).unwrap_or("").to_string(), made.get("product").and_then(Value::as_str).unwrap_or(&to).to_string());
            // Z puts it back where it was: the copy is closed and the thread opened again.
            s.undos.push(write::Undo::plain(format!("Back in {from}"), format!("put it back in {from}"), rules::move_back(made, &to, &thread), write::Sent::Palette(Wrote::Quiet), thread.clone(), true));
            // Left on the old row, now closed, the person would reopen it and have two.
            if s.is_open(&thread) {
                s.palette.opens_when_listed = Some((copy.clone(), crate::clock()));
            }
            s.say_about(&format!("Moved to {name} · press Z to undo"), copy, tx);
        }
        Wrote::SendBack { thread, project, next } => {
            s.say_about(&rules::sent_back_said(&project), thread.clone(), tx);
            // Back in the queue is out of the Inbox, so from inside it the person goes on.
            if s.is_open(&thread) {
                s.advance(next);
            }
        }
        Wrote::Redeliver { thread, project } => match rules::redeliver_said(&answer, &project) {
            Some(said) => s.say_about(&said, thread, tx),
            None => s.tell("Nothing here to restart", tx),
        },
        Wrote::Moving => s.tell(&crate::rules::cloud::moved_line(crate::rules::place::moving(&answer, 1)), tx),
        Wrote::Ranked { name } => match refusal() {
            Some(why) => s.refused = Some(format!("Could not change the order: {why}")),
            None => s.tell(&format!("{name} is now first"), tx),
        },
        Wrote::ResumeStopped => s.tell(&rules::resume_stopped_said(&answer), tx),
        // A move that worked relaunches the app; one that did not says its own reason.
        Wrote::UpdateMove => {
            if answer.get("moved") != Some(&Value::Bool(true)) && let Some(why) = answer.get("error").and_then(Value::as_str).filter(|e| !e.is_empty()) {
                s.tell(why, tx);
            }
        }
        Wrote::UpdateCheck => {
            let said = crate::rules::settings::update_look(Some(&answer), false).sentence;
            s.tell(&said, tx);
        }
    }
}

/// The poll's part: what the app found inside threads for the words, once it lands. True when
/// the rows changed.
pub(crate) fn found_landed(s: &mut State) -> bool {
    match s.palette.finder.take() {
        Some((query, answer)) => s.found_inside(&query, answer),
        None => false,
    }
}

/// The poll's part: a moved thread's copy opens once the lists have it, within ten seconds
/// (`listedNow`). True when it opened.
pub(crate) fn tend(s: &mut State) -> bool {
    let Some(((id, product), at)) = s.palette.opens_when_listed.clone() else {
        return false;
    };
    if crate::clock() - at > 10_000.0 {
        s.palette.opens_when_listed = None;
        return false;
    }
    let lists = &s.lists;
    let listed = lists.needs.iter().chain(&lists.progress).chain(&lists.later).chain(&lists.done).any(|r| r.id == id && r.product == product);
    if !listed {
        return false;
    }
    s.palette.opens_when_listed = None;
    s.open_thread(&id, &product);
    true
}

/// Enter: runs the lit row. With no row there is nothing to run.
fn run_the_lit_row(s: &mut State, tx: &mut Tx) {
    let lines = lines(s);
    if let Some(does) = lit(s, &lines).and_then(|at| lines[at].does.clone()) {
        run(s, does, tx);
    }
}

/// The keys the box does not keep for typing: the arrows move the lit row, stopping at either
/// end, and only a key brings the row it lands on into view, since a pointer is already over
/// its own. Escape and Return are the bar's (`modal_tx`, `on_submit`).
fn in_the_box(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> bool {
    if !matches!(e.key, Key::Down | Key::Up) {
        return false;
    }
    let lines = lines(s);
    if let Some(at) = lit(s, &lines) {
        let to = if e.key == Key::Down { (at + 1).min(lines.len() - 1) } else { at.saturating_sub(1) };
        s.palette.light(&lines[to].key);
        tx.scroll_into_view_in("palette-list", format!("palette-{}", lines[to].key));
    }
    true
}

/// The same keys when the box has lost the caret: the bar still answers them.
fn pressed(s: &mut State, e: &KeyEvent, tx: &mut Tx) {
    match &e.key {
        Key::Escape => s.close_palette(),
        Key::Enter => run_the_lit_row(s, tx),
        _ => {
            in_the_box(s, e, tx);
        }
    }
}

/// What the bar takes of the keys before the page gets any: ⌘K from anywhere, and every key
/// that is no chord while it is open. Control K does what ⌘K does, as the web window's keydown
/// handler has it (`metaKey || ctrlKey` in `renderer/src/App.tsx`).
pub fn took(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> bool {
    let k_with_control = e.mods.ctrl && !e.mods.alt && matches!(&e.key, Key::Text(t) if t.eq_ignore_ascii_case("k"));
    if e.is_command('k') || k_with_control {
        s.toggle_palette(tx);
        return true;
    }
    if !s.palette.open || e.mods.cmd || e.mods.ctrl || e.mods.alt {
        return false;
    }
    pressed(s, e, tx);
    true
}

/* ------------------------------------ how it is drawn ------------------------------------ */

/// `--agent-ink`: the placeholder and the headings.
fn quiet() -> Color {
    rgb(127, 121, 117)
}

/// The small capitals of the mono face: a heading. The line is as tall as the type, and the
/// letters stand a tenth or so of it apart.
fn capitals(size: f32, apart: f32) -> TextStyle {
    TextStyle::new(size, 500.0).mono().lh(size).tracking(apart)
}

const WIDTH: f32 = 620.0;
const BOX: f32 = 56.0;
const ROW: f32 = 38.0;
/// What the list grows to before it scrolls (`.palette-list`'s `max-height`).
const LIST_CAP: f32 = 320.0;

/// The drawing at the head of a row, 16 points in the middle of its 18 (`.palette-glyph`):
/// in the faint ink, and in the text's own on the lit row.
fn glyph(name: &str, lit: bool) -> El {
    row().size(18.0, 18.0).center().child(icon(marks::mark(name), 16.0).color(if lit { ink() } else { muted() }))
}

/// A key at a row's end (`.palette-key`), and Esc in the box, which wears the same cap.
fn key_cap(cap: &str) -> El {
    row().min_w(22.0).h(20.0).px(5.0).center().radius(6.0).fill(page()).border(1.0, edge()).border_layout().child(pen(text(cap).font(mono(11.5, 15.0)).color(dim()), 11.5))
}

/// A heading over a section. The first thing in the list stands closer to its top.
fn heading(words: &str, first: bool) -> El {
    el().pt(if first { 8.0 } else { 14.0 }).px(18.0).pb(6.0).child(text(words.to_uppercase()).font(capitals(9.5, 1.14)).color(quiet()))
}

/// How tall a row is: a thread with a line to say stands on two, and so does a thing found
/// inside one (`:is(.palette-found, .palette-two)`).
fn height(line: &Line) -> f32 {
    if line.sub.is_some() || line.found.is_some() { 51.5 } else { ROW }
}

/// `text` with the typed words lit in it, in the text's ink and weight 600 (`markedWords`, `.hit`).
fn marked(text_said: &str, query: &str, font: TextStyle, color: Color) -> El {
    let (mut said, mut marks) = (String::new(), Vec::new());
    for run in search::marked_words(text_said, js::trim(query)) {
        let from = said.len();
        said.push_str(&run.text);
        if run.hit {
            marks.push(Span::new(from, said.len()).weight(600.0).color(ink()));
        }
    }
    text(said).font(font).color(color).spans(marks).truncate()
}

fn line_row(line: &Line, lit: bool, query: &str, now: f64) -> El {
    // A finished thread steps back; the highlight behind a lit one does not (`.palette-done > *`).
    let fade = |e: El| if line.done { e.opacity(0.5) } else { e };
    let name = pen(label(&line.label, 14.5, 400.0, if lit { ink() } else { dim() }).truncate(), 14.5);
    let hint = line.hint.as_deref().map(|hint| pen(label(hint, 12.5, 400.0, muted()), 12.5));
    let mut e = match (&line.found, &line.sub) {
        // Found inside a thread: what it says with the typed words lit, a file's name before
        // the line that matched in it, and under that who, where and when, each one line cut
        // short; no hint, never dimmed (`.palette-found`).
        (Some(found), _) => {
            let words = pen(marked(&found.text, query, sans(14.5, 400.0), if lit { ink() } else { dim() }), 14.5);
            let said = match &found.file {
                Some(name) => row().min_w(0.0).child(el().pr(8.0).child(pen(label(name, 14.5, 500.0, ink()), 14.5))).child(row().grow(1.0).min_w(0.0).child(words)),
                None => words,
            };
            let under = pen(text(found::under(found, now)).font(sans(12.0, 400.0)).color(muted()).truncate(), 12.0);
            row().min_h(50.0).py(7.0).px(18.0).gap(12.0).items_start().child(glyph(&line.icon, lit)).child(col().grow(1.0).min_w(0.0).gap(3.0).child(said).child(under))
        }
        // A thread found: its title, then the line it was found by with the typed words in
        // ink, and where it is and how old beside them. A long title gives way, never the
        // project's name.
        (None, Some(sub)) => {
            let found_by = pen(marked(sub, query, sans(12.0, 400.0), muted()), 12.0);
            // The drawing stands at the top of the two lines, level with the title's.
            let mut e = row().min_h(50.0).py(7.0).px(18.0).gap(12.0).items_start().child(fade(glyph(&line.icon, lit))).child(fade(col().grow(1.0).min_w(0.0).gap(3.0).child(name).child(found_by)));
            if let Some(hint) = hint {
                e = e.child(fade(el().pt(2.0).child(hint)));
            }
            e
        }
        (None, None) => {
            let mut e = row().h(ROW).px(18.0).gap(12.0).items_center().child(fade(glyph(&line.icon, lit))).child(fade(row().grow(1.0).min_w(0.0).child(name)));
            match (&line.cap, hint) {
                (Some(cap), _) => e = e.child(fade(key_cap(cap))),
                (None, Some(hint)) => e = e.child(fade(hint)),
                _ => {}
            }
            e
        }
    };
    // The lit row is a wash, with no accent down its left edge any more (`.palette-item.selected`).
    if lit {
        e = e.fill(white(0.04));
    }
    let (key, does) = (line.key.clone(), line.does.clone());
    e.key(format!("palette-{}", line.key))
        .cursor(Cursor::Pointer)
        // The pointer lights the row it moves over, never the one it happened to be resting
        // on when the list moved under it: a move to where the pointer already was is the
        // list moving, not a person (`pointerMoved` in `renderer/src/palette-pick.ts`).
        .on_pointer_move(move |s: &mut State, at| {
            let moved = s.palette.pointer != Some((at.x, at.y));
            s.palette.pointer = Some((at.x, at.y));
            if moved {
                s.palette.light(&key);
            }
        })
        .on_press_tx(move |s: &mut State, tx| {
            if let Some(does) = does.clone() {
                run(s, does, tx);
            }
        })
}

/// The bar, over the whole window, while it is open.
pub fn view(s: &State) -> Option<El> {
    if !s.palette.open {
        return None;
    }
    let (w, h) = s.size;
    let width = WIDTH.min(0.92 * w);
    let query = s.palette.words.text.as_str();
    let lines = lines(s);
    let lit = lit(s, &lines);
    // A found thing's age is read off the clock as the row is drawn (`ago(c.found.at)`), the
    // picture's own clock in a picture.
    let now = if s.fixed_clock { s.now } else { crate::clock() };

    let field = text_input(&s.palette.words, sans(17.0, 400.0))
        .key("palette")
        .single_line()
        .grow(1.0)
        .color(ink())
        .placeholder(rules::PLACEHOLDER, quiet())
        .caret(accent(), rgba(110, 165, 255, 0.3))
        .on_edit(|s: &mut State| &mut s.palette.words)
        .on_input_change(|s: &mut State, _| s.palette_typed())
        .on_submit(run_the_lit_row)
        .on_input_key(in_the_box);
    // Esc is a key's cap, as the rows' keys are, its word as the key is named (`.palette-esc`).
    let esc = key_cap("esc").cursor(Cursor::Pointer).on_press(|s: &mut State| s.close_palette());
    let ask = row()
        .h(BOX)
        .pl(18.0)
        .pr(16.0)
        .gap(12.0)
        .items_center()
        .border_sides([0.0, 0.0, 1.0, 0.0], edge())
        .border_layout()
        .child(icon(&icons::BAR_SEARCH, 18.0).color(muted()))
        .child(pen(field, 17.0))
        .child(esc);

    // The rows, six points clear of the list's top and foot. With nothing to list, one line
    // says so where the first row would have been.
    // A scroller lays its content out at the content's own width, so the rows are told the
    // card's, or the lit row's wash would stop where its words do.
    let mut rows = col().w(width - 2.0).py(6.0);
    let mut tall = 12.0;
    if lines.is_empty() {
        rows = rows.child(el().py(8.0).px(18.0).child(pen(label(&rules::empty_line(query), 14.0, 400.0, muted()), 14.0)));
        tall += 34.0;
    }
    for (at, line) in lines.iter().enumerate() {
        if let Some(words) = &line.heading {
            rows = rows.child(heading(words, at == 0));
            tall += if at == 0 { 23.5 } else { 29.5 };
        }
        rows = rows.child(line_row(line, lit == Some(at), query, now));
        tall += height(line);
    }
    // A column: a plain box is a row, which stretches the rows to its own height, and then
    // there is nothing left over to scroll to.
    let list = col().key("palette-list").h(tall.min(LIST_CAP)).scroll(ScrollSpec { stick_bottom: false }).child(rows);

    // The box stands where a command bar stands: its middle three eighths of the way down the
    // window, at every window size (`place-items: center` over a quarter of the height held
    // back under it, in `.modal-backdrop.palette-backdrop`). It ends with the list: the foot
    // that said what the arrows and Return do is gone (`.palette-foot`).
    // No rounded clip: Rune's takes the card's own shadow with it, and nothing inside reaches
    // a corner (the list scrolls inside its own bounds).
    let card_tall = 2.0 + BOX + tall.min(LIST_CAP);
    let card = col()
        .absolute((w - width) / 2.0, (0.75 * h - card_tall) / 2.0)
        .w(width)
        .radius(10.0)
        .fill(rgb(38, 36, 34))
        .border(1.0, white(0.08))
        .border_layout()
        .box_shadow(shadow(0.0, 24.0, 70.0, 0.0, rgba(0, 0, 0, 0.55), false))
        .modal_tx(true, |s: &mut State, _| s.close_palette())
        .child(ask)
        .child(list);
    // `--scrim` over the whole window, as this look has it.
    Some(el().absolute(0.0, 0.0).size(w, h).fill(rgba(0, 0, 0, 0.72)).child(card))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A row as the web's picture records it and as this window draws it, for the two to be held
    /// side by side: a heading, a thread, or a command's drawing, words and key or hint.
    fn said_web(row: &Pictured) -> String {
        match row {
            Pictured::Heading(words) => format!("## {words}"),
            Pictured::Thread => "(a thread)".to_string(),
            Pictured::Found => "(found inside)".to_string(),
            Pictured::Command { icon, label, hint, cap } => format!("{icon} · {label} · {}", cap.clone().or_else(|| hint.clone()).unwrap_or_default()),
        }
    }

    fn said_own(lines: &[Line]) -> Vec<String> {
        let mut out = Vec::new();
        for line in lines {
            if let Some(words) = &line.heading {
                out.push(format!("## {words}"));
            }
            out.push(if line.key.starts_with("thread-") { "(a thread)".to_string() } else if line.found.is_some() { "(found inside)".to_string() } else { format!("{} · {} · {}", line.icon, line.label, line.cap.clone().or_else(|| line.hint.clone()).unwrap_or_default()) });
        }
        out
    }

    /// The two lists aligned on what they share, in order: ` ` both, `-` the web's alone, `+`
    /// this window's alone.
    fn aligned(web: &[String], own: &[String]) -> Vec<(char, String)> {
        let (n, m) = (web.len(), own.len());
        let mut best = vec![vec![0usize; m + 1]; n + 1];
        for i in (0..n).rev() {
            for j in (0..m).rev() {
                best[i][j] = if web[i] == own[j] { best[i + 1][j + 1] + 1 } else { best[i + 1][j].max(best[i][j + 1]) };
            }
        }
        let (mut i, mut j, mut out) = (0, 0, Vec::new());
        while i < n || j < m {
            if i < n && j < m && web[i] == own[j] {
                out.push((' ', web[i].clone()));
                (i, j) = (i + 1, j + 1);
            } else if j < m && (i == n || best[i][j + 1] >= best[i + 1][j]) {
                out.push(('+', own[j].clone()));
                j += 1;
            } else {
                out.push(('-', web[i].clone()));
                i += 1;
            }
        }
        out
    }

    /// `MERCURY_PALETTE="<fixture> [--view inbox|--thread|--tab progress]; <next>…" cargo test
    /// the_rows_are_the_webs -- --ignored --nocapture`: this window's own rows for a picture's
    /// inbox and words beside the rows the web window drew (`paletteRows`), aligned row by row.
    #[test]
    #[ignore]
    fn the_rows_are_the_webs() {
        let spec = std::env::var("MERCURY_PALETTE").expect("MERCURY_PALETTE names the fixtures");
        for one in spec.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            let mut words = one.split_whitespace().map(str::to_string);
            let path = words.next().unwrap();
            let args: Vec<String> = ["--fixture".to_string(), path.clone()].into_iter().chain(words).collect();
            let s = crate::init_from(&args);
            let web: Vec<String> = s.palette.pictured.as_ref().expect("the fixture has the web's rows").iter().map(said_web).collect();
            let own = said_own(&own_lines(&s, threads(&s)));
            let rows = aligned(&web, &own);
            let both = rows.iter().filter(|(mark, _)| *mark == ' ').count();
            println!("\n{path}: \"{}\": the web's {} rows, this window's {}, {both} the same in the same order", s.palette.words.text, web.len(), own.len());
            for (mark, row) in rows {
                println!("  {mark} {row}");
            }
        }
    }

    fn found(id: &str, title: &str) -> rules::Row {
        let item: Item = serde_json::from_value(json!({ "id": id, "product": "shop", "status": "open", "title": title, "updatedAt": 1 })).unwrap();
        rules::thread_row(&item, title.to_string(), "Storefront", 2.0)
    }

    /// `markUnseen` in `App.tsx`, from a window that does not keep what has been read: the mark
    /// stands here until the web window's own record says unread too.
    #[test]
    fn a_thread_marked_unread_here_stays_unread_until_the_web_window_says_so_too() {
        // Answered long ago, and read a moment after.
        let (answered, read) = (1_700_000_000_000.0, 1_700_000_100_000.0);
        let fixture = json!({
            "now": answered + 3_600_000.0,
            "snapshot": {
                "products": [{ "slug": "shop", "name": "Storefront" }],
                "items": [{ "id": "w-1", "product": "shop", "status": "open", "kind": "question", "title": "Checkout fails", "result": "Which one?", "createdAt": answered - 60_000.0,
                    "wrote": { "result": { "ts": answered, "source": "agent" } } }],
                "approvals": [],
                "supervisor": { "running": [], "stalled": [], "queued": [], "productOrder": ["shop"] },
            },
            "local": { "seenAt": { "w-1": read }, "seen": ["w-1"] },
        });
        let path = std::env::temp_dir().join(format!("mercury-palette-unread-{}.json", std::process::id()));
        std::fs::write(&path, fixture.to_string()).expect("the invented inbox");
        let mut s = crate::init_from(&["--fixture".to_string(), path.to_string_lossy().into_owned()]);
        let _ = std::fs::remove_file(&path);
        let thread = s.item("w-1", "shop").cloned().unwrap();
        let unread = |s: &State| rules::is_unread(&thread, &s.local);
        let theirs = |at: Option<f64>| crate::data::Local { seen_at: at.map(|at| ("w-1".to_string(), at)).into_iter().collect(), seen: vec!["w-1".to_string()], ..Default::default() };
        assert!(!unread(&s));
        s.unsaw("w-1");
        assert!(unread(&s) && !s.local.seen.iter().any(|id| id == "w-1"));
        // What the web window keeps still says read, as it was before it was asked: the mark stands.
        s.keep(theirs(Some(read)));
        assert!(unread(&s));
        // It says unread too, and from then on its word alone decides: opened there, it is read.
        s.keep(theirs(Some(0.0)));
        assert!(unread(&s));
        s.keep(theirs(Some(read + 5_000.0)));
        assert!(!unread(&s));
        // Marked again and opened in the web window before it took the mark: its reading is the newer.
        s.unsaw("w-1");
        s.keep(theirs(Some(crate::write::wall_clock() + 60_000.0)));
        assert!(!unread(&s));
        // A mark the app would not take leaves the thread read as it was.
        s.keep(theirs(Some(read)));
        s.unsaw("w-1");
        assert!(unread(&s));
        s.unsaw_refused("w-1");
        assert!(!unread(&s));
        assert_eq!(s.local.seen_at.get("w-1"), Some(&read));
        // And opening it here reads it, whatever was marked.
        s.unsaw("w-1");
        s.open_thread("w-1", "shop");
        s.keep(theirs(Some(read)));
        assert!(!unread(&s));
    }

    /// An inbox of three threads, the second an outside agent's, opened on a fixture.
    fn found_inbox(name: &str) -> State {
        let now = 1_790_000_000_000.0_f64;
        let item = |id: &str, title: &str| json!({ "id": id, "product": "shop", "status": "open", "kind": "directive", "title": title, "body": title, "labels": ["founder"], "createdAt": now - 3_600_000.0, "updatedAt": now - 600_000.0 });
        let mut outside = item("agent:7", "Storefront agent");
        outside["agent"] = json!({ "pid": 7 });
        let fixture = json!({
            "now": now,
            "snapshot": {
                "products": [{ "slug": "shop", "name": "Storefront", "dir": "/fixtures/shop" }],
                "items": [item("w-1", "Checkout fails"), outside, item("w-3", "Onboarding checklist copy")],
                "approvals": [],
                "supervisor": { "running": [], "stalled": [], "queued": [], "productOrder": ["shop"] },
            },
            "local": {},
        });
        let path = std::env::temp_dir().join(format!("mercury-palette-{name}-{}.json", std::process::id()));
        std::fs::write(&path, fixture.to_string()).expect("the invented inbox");
        let s = crate::init_from(&["--fixture".to_string(), path.to_string_lossy().into_owned(), "--view".to_string(), "inbox".to_string()]);
        let _ = std::fs::remove_file(&path);
        s
    }

    /// What the finder answers for "refund": two lines in Checkout fails and one in a thread this
    /// window does not list, a report found by its name and a note by what it says.
    fn refunds(now: f64) -> Value {
        let at = |m: f64| now - m * 60_000.0;
        let w = |id: &str| json!({ "kind": "thread", "product": "shop", "id": id, "ownerId": null, "threadId": null, "updatedAt": 1 });
        let with = |base: Value, more: Value| {
            let mut base = base.as_object().unwrap().clone();
            base.extend(more.as_object().unwrap().clone());
            Value::Object(base)
        };
        json!({
            "ok": true,
            "lines": [
                with(w("w-1"), json!({ "uid": "u1", "by": null, "source": "agent", "at": at(30.0), "comment": false, "text": "The 37 refunds are queued behind a note to support." })),
                with(w("w-9"), json!({ "uid": "u2", "by": null, "source": "agent", "at": at(5.0), "comment": false, "text": "Refund it." })),
                with(w("w-1"), json!({ "uid": "u3", "by": null, "source": "founder", "at": at(10.0), "comment": false, "text": "Tell support before the refunds go out." })),
            ],
            "files": [
                with(w("w-1"), json!({ "how": "name", "src": "/fixtures/shop/refunds.png", "name": "refunds.png", "by": null, "at": at(20.0), "text": "" })),
                with(w("w-3"), json!({ "how": "contents", "src": "/fixtures/shop/notes.md", "name": "notes.md", "by": null, "at": at(40.0), "text": "Refunds go out on Friday." })),
                with(w("agent:7"), json!({ "how": "name", "src": "/fixtures/shop/refund-plan.md", "name": "refund-plan.md", "by": null, "at": at(50.0), "text": "" })),
            ],
            "comments": [],
            "total": { "lines": 3, "files": 3, "comments": 0 },
        })
    }

    fn typed(s: &mut State, words: &str) {
        s.palette.words = TextEdit::new(words);
        s.palette_typed();
    }

    fn drawn(s: &State) -> Vec<String> {
        lines(s).iter().flat_map(|l| l.heading.iter().map(|h| format!("## {h}")).chain([format!("{} {}", l.icon, l.label)]).collect::<Vec<_>>()).collect()
    }

    /// `PaletteBox`'s found-inside effect and `insideForPalette`: the rows after every other row,
    /// Lines then Files under their headings, their drawings by their ids, and kept until the
    /// next answer lands.
    #[test]
    fn what_is_found_inside_threads_comes_after_every_other_row_under_its_headings() {
        let mut s = found_inbox("found-rows");
        let mut tx = Tx::default();
        s.open_palette(&mut tx);
        typed(&mut s, "refund");
        // Nothing is shown of it until the app answers.
        assert!(drawn(&s).iter().all(|r| !r.starts_with("## Lines")));
        assert!(s.found_inside("refund", Ok(refunds(s.now))));
        assert_eq!(
            drawn(&s),
            [
                "## Lines",
                "text Tell support before the refunds go out.",
                "text The 37 refunds are queued behind a note to support.",
                "## Files",
                "file refunds.png",
                "file Refunds go out on Friday.",
                "file refund-plan.md",
            ]
        );
        let all = lines(&s);
        assert_eq!(all[0].key, "line-thread-shop-w-1-u3");
        assert_eq!(all[3].found.as_ref().and_then(|f| f.file.as_deref()), Some("notes.md"));
        assert!(all.iter().all(|l| l.hint.is_none() && l.cap.is_none() && !l.done));
        assert_eq!(found::under(all[1].found.as_ref().unwrap(), s.now), "Agent · Checkout fails · 30m");
        // New words keep the last answer's rows until the next lands; a failed call leaves them.
        typed(&mut s, "refunds");
        assert_eq!(lines(&s).len(), 5);
        assert!(!s.found_inside("refunds", Err("the door is shut".into())));
        assert_eq!(lines(&s).len(), 5);
        // An answer the app could not make leaves none, and the line saying nothing matches.
        let none = json!({ "ok": false, "error": "no", "lines": [], "files": [], "comments": [], "total": { "lines": 0, "files": 0, "comments": 0 } });
        s.found_inside("refunds", Ok(none));
        assert!(lines(&s).is_empty());
        // Words cleared take them away at once.
        s.found_inside("refund", Ok(refunds(s.now)));
        typed(&mut s, "  ");
        assert!(lines(&s).iter().all(|l| l.found.is_none()));
        // And a closed bar lands nothing.
        s.close_palette();
        assert!(!s.found_inside("refund", Ok(refunds(s.now))));
    }

    /// `pickedKey` and `keptPick`: with nothing else listed for the words, the first found row is
    /// lit as it lands; a row already lit stays lit as they come in under it; new words light the
    /// first row; and the arrows step over the headings.
    #[test]
    fn the_lit_row_is_kept_as_found_rows_come_in_and_the_arrows_step_over_their_headings() {
        let mut s = found_inbox("found-lit");
        let mut tx = Tx::default();
        s.open_palette(&mut tx);
        typed(&mut s, "refund");
        assert!(lines(&s).is_empty());
        s.found_inside("refund", Ok(refunds(s.now)));
        let at = |s: &State| lit(s, &lines(s)).map(|at| lines(s)[at].key.clone());
        assert_eq!(at(&s).as_deref(), Some("line-thread-shop-w-1-u3"));
        assert_eq!(s.palette.lit.as_ref().map(|(_, key)| key.as_str()), Some("line-thread-shop-w-1-u3"));
        // Down twice: the second line, then the first file, past the Files heading.
        let down = KeyEvent { key: Key::Down, mods: Default::default() };
        in_the_box(&mut s, &down, &mut tx);
        in_the_box(&mut s, &down, &mut tx);
        assert_eq!(at(&s).as_deref(), Some("file-thread-shop-w-1-/fixtures/shop/refunds.png"));
        // A new answer to the same words keeps that row lit wherever it lands.
        let mut again = refunds(s.now);
        again["files"].as_array_mut().unwrap().swap(0, 1);
        s.found_inside("refund", Ok(again));
        assert_eq!(at(&s).as_deref(), Some("file-thread-shop-w-1-/fixtures/shop/refunds.png"));
        // Words that list a thread light it, and the found rows coming in under it leave it lit.
        typed(&mut s, "checkout");
        let first = lines(&s)[0].key.clone();
        assert_eq!(at(&s), Some(first.clone()));
        s.found_inside("checkout", Ok(refunds(s.now)));
        assert_eq!(at(&s), Some(first));
    }

    /// `openInside`: the bar closes, the thread opens and is read, and a line is looked for; a
    /// file opens beside the conversation when it is one that does, else big, and an outside
    /// agent's big.
    #[test]
    fn a_found_row_opens_its_thread_at_the_line_or_with_its_file_beside_it_or_big() {
        let mut s = found_inbox("found-run");
        let mut tx = Tx::default();
        s.open_palette(&mut tx);
        typed(&mut s, "refund");
        s.found_inside("refund", Ok(refunds(s.now)));
        let does = |s: &State, key: &str| lines(s).into_iter().find(|l| l.key == key).and_then(|l| l.does).unwrap();
        let line = does(&s, "line-thread-shop-w-1-u3");
        run(&mut s, line, &mut tx);
        assert!(!s.palette.open, "the bar closed");
        assert_eq!(s.view, View::Thread { id: "w-1".into(), product: "shop".into() });
        assert!(s.local.seen.iter().any(|id| id == "w-1"), "counted as read");
        let r = s.reading_reveal().expect("the line is looked for");
        assert_eq!(r, ("shop/w-1".to_string(), "u3".to_string()));
        // A note opens beside its conversation.
        s.open_palette(&mut tx);
        typed(&mut s, "refund");
        s.found_inside("refund", Ok(refunds(s.now)));
        let note = does(&s, "file-thread-shop-w-3-/fixtures/shop/notes.md");
        run(&mut s, note, &mut tx);
        assert_eq!(s.view, View::Thread { id: "w-3".into(), product: "shop".into() });
        assert_eq!(s.beside().map(|d| d.src.clone()).as_deref(), Some("/fixtures/shop/notes.md"));
        assert!(s.viewer.open.is_none());
        // A picture opens big over it.
        s.open_palette(&mut tx);
        typed(&mut s, "refund");
        s.found_inside("refund", Ok(refunds(s.now)));
        let picture = does(&s, "file-thread-shop-w-1-/fixtures/shop/refunds.png");
        run(&mut s, picture, &mut tx);
        assert_eq!(s.view, View::Thread { id: "w-1".into(), product: "shop".into() });
        assert!(s.beside().is_none());
        assert!(s.viewer.open.is_some(), "big over the conversation");
        // A note on an outside agent's row does not open beside it.
        s.open_palette(&mut tx);
        typed(&mut s, "refund");
        s.found_inside("refund", Ok(refunds(s.now)));
        let outside = does(&s, "file-thread-shop-agent:7-/fixtures/shop/refund-plan.md");
        assert_eq!(outside, Does::Found(found::Open::File { thread: ("agent:7".into(), "shop".into()), src: "/fixtures/shop/refund-plan.md".into(), outside: true }));
        run(&mut s, outside, &mut tx);
        assert_eq!(s.view, View::Thread { id: "agent:7".into(), product: "shop".into() });
        assert!(s.beside().is_none());
    }

    /// The asking itself: 90 ms after the words last changed, once for a run of keys, and an
    /// answer to older words dropped however late it lands (`setTimeout(…, 90)` and `live`).
    #[test]
    fn the_app_is_asked_ninety_ms_after_the_words_rest_and_an_answer_to_older_words_is_dropped() {
        use std::sync::{Arc, Mutex};
        use std::time::{Duration, Instant};
        let finder = Finder::default();
        let asked: Arc<Mutex<Vec<(String, Instant)>>> = Arc::default();
        let ask = |words: &str, slow: u64| {
            let (asked, words) = (asked.clone(), words.to_string());
            let payload = found::ask(&words).unwrap();
            finder.ask(words.clone(), payload, move |p| {
                asked.lock().unwrap().push((p["query"].as_str().unwrap().to_string(), Instant::now()));
                std::thread::sleep(Duration::from_millis(slow));
                Ok(json!({ "for": p["query"] }))
            });
        };
        let typed_at = Instant::now();
        for words in ["r", "re", "ref", "refund"] {
            ask(words, 0);
        }
        std::thread::sleep(Duration::from_millis(40));
        assert!(asked.lock().unwrap().is_empty(), "not before the words have rested 90 ms");
        let landed = (0..100).find_map(|_| {
            std::thread::sleep(Duration::from_millis(10));
            finder.take()
        });
        let calls = asked.lock().unwrap().clone();
        assert_eq!(calls.iter().map(|(w, _)| w.as_str()).collect::<Vec<_>>(), ["refund"], "once, for the words as they rested");
        assert!(calls[0].1.duration_since(typed_at) >= Duration::from_millis(90));
        assert_eq!(landed.map(|(q, a)| (q, a.unwrap())), Some(("refund".to_string(), json!({ "for": "refund" }))));
        // A slow answer for "refu" is still out when "refund" is asked: it never lands.
        asked.lock().unwrap().clear();
        ask("refu", 300);
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(asked.lock().unwrap().len(), 1, "the first is under way");
        ask("refund", 0);
        let mut got = Vec::new();
        for _ in 0..60 {
            std::thread::sleep(Duration::from_millis(10));
            got.extend(finder.take().map(|(q, _)| q));
        }
        assert_eq!(got, ["refund"]);
        // Stopped, nothing asked lands.
        ask("refunds", 0);
        finder.stop();
        std::thread::sleep(Duration::from_millis(200));
        assert!(finder.take().is_none());
    }

    #[test]
    fn a_picture_draws_the_webs_commands_and_this_windows_threads_in_their_places() {
        let rows = json!([
            { "icon": "refresh", "label": "Check for updates", "hint": "look now", "key": "" },
            { "section": "Threads" },
            { "thread": true },
            { "thread": true },
            { "icon": "", "label": "" },
        ]);
        let pictured: Vec<Pictured> = rows.as_array().unwrap().iter().filter_map(Pictured::read).collect();
        // A row with no name is no row.
        assert_eq!(pictured.len(), 4);
        let lines = pictured_lines(&pictured, vec![found("w-1", "Checkout fails"), found("w-2", "Onboarding checklist copy"), found("w-3", "Not listed by the web")], Vec::new());
        let said: Vec<(&str, Option<&str>)> = lines.iter().map(|l| (l.label.as_str(), l.heading.as_deref())).collect();
        assert_eq!(said, [("Check for updates", None), ("Checkout fails", Some("Threads")), ("Onboarding checklist copy", None), ("Not listed by the web", None)]);
        // The web's command does nothing here, and the threads open as they would anywhere.
        assert!(lines[0].does.is_none() && lines[0].cap.is_none() && lines[0].hint.as_deref() == Some("look now"));
        assert_eq!(lines[1].does, Some(Does::Open(("w-1".to_string(), "shop".to_string()))));
        assert_eq!((lines[0].icon.as_str(), lines[1].icon.as_str(), lines[1].hint.as_deref()), ("refresh", "thread", Some("Storefront · now")));
        // With fewer threads found than the web listed, the places left over stay empty.
        assert_eq!(pictured_lines(&pictured, vec![found("w-1", "Checkout fails")], Vec::new()).len(), 2);
        // A command this window has too, by its words and drawing, is its own row and does what it says.
        let own = rules::update_rows(&crate::data::Snapshot::default()).into_iter().map(|row| Line::of(row, "update-check".to_string(), None)).collect();
        let lines = pictured_lines(&pictured, Vec::new(), own);
        assert_eq!((lines[0].does.clone(), lines[0].hint.as_deref(), lines[0].key.as_str()), (Some(Does::CheckForUpdates), Some("look for one now"), "update-check"));
    }
}
