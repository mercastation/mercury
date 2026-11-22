//! The command bar: what ⌘K opens over the page (`PaletteBox` in
//! `renderer/src/components/Palette.tsx`, `.palette-k` in `renderer/src/styles.css`). A box to
//! search threads or type a command in, over the rows that answer it: the arrows and the
//! pointer move the lit row, Enter or a press runs it, Escape or a press outside closes it.
//!
//! Which rows there are is `rules/palette.rs`; here they are drawn and carried out. A row is
//! there only when this window does what it says, so the list is much shorter than the web's.

use std::cell::RefCell;

use rune::edit::Key;
use rune::text::Span;
use rune::{Color, Cursor, El, KeyEvent, ScrollSpec, TextEdit, TextStyle, Tx, col, el, icon, row, text, text_input};
use serde_json::{Map, Value};

use crate::data::Item;
use crate::rules::palette::{self as rules, Does};
use crate::rules::{js, search, writes};
use crate::theme::*;
use crate::{State, View, icons, write};

/// HOOK: New thread. The web's bar has it second, after New project, and N does the same
/// (`onNewThread` in `Palette.tsx`). The box a thread is started in is being built apart
/// (`w-4c6fedcd6c-compose`); once it is, this answers with what opens it, with the caret in it,
/// and the row is drawn. Until then it answers nothing and the row is not drawn, because a row
/// is never drawn that would do nothing.
fn starts_a_thread() -> Option<fn(&mut State, &mut Tx)> {
    None
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
}

impl Bar {
    pub fn closed() -> Bar {
        Bar { open: false, words: TextEdit::new(""), lit: None, pointer: None, index: RefCell::new(search::Index::default()), pictured: None }
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

/// A row of the web window's bar, as a fixture carries it.
#[derive(Clone)]
enum Pictured {
    Heading(String),
    /// Where it listed a thread.
    Thread,
    Command { glyph: String, label: String, hint: Option<String>, cap: Option<String> },
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
        Some(Pictured::Command { glyph: words("glyph").unwrap_or_default(), label: words("label")?, hint: words("hint"), cap: words("key") })
    }
}

/// One row as it is drawn, and what running it does.
#[derive(Clone)]
struct Line {
    key: String,
    /// The heading over it, when a section starts at it.
    heading: Option<String>,
    glyph: String,
    label: String,
    hint: Option<String>,
    /// The key that does the same, drawn in place of a hint.
    cap: Option<String>,
    sub: Option<String>,
    done: bool,
    /// What pressing it does. A command the web window's picture had does nothing.
    does: Option<Does>,
}

impl Line {
    fn of(row: rules::Row, key: String, heading: Option<String>) -> Line {
        Line {
            key,
            heading,
            glyph: rules::glyph_for(&row.id).to_string(),
            label: row.label,
            // A key says more than a hint, and a row has room for one of them.
            hint: row.hint.filter(|_| row.key.is_none()),
            cap: row.key.map(str::to_string),
            sub: row.sub,
            done: row.done,
            does: Some(row.does),
        }
    }
}

/// The web window's rows as a fixture carries them, with this window's own threads where it
/// listed threads. A thread it found that the web did not list goes at the end, so the
/// picture shows it too.
fn pictured_lines(pictured: &[Pictured], threads: Vec<rules::Row>) -> Vec<Line> {
    let keys = rules::row_keys(&threads);
    let mut found = threads.into_iter().zip(keys);
    let (mut lines, mut heading) = (Vec::new(), None);
    for (at, row) in pictured.iter().enumerate() {
        match row {
            Pictured::Heading(words) => heading = Some(words.clone()),
            Pictured::Thread => {
                if let Some((thread, key)) = found.next() {
                    lines.push(Line::of(thread, key, heading.take()));
                }
            }
            Pictured::Command { glyph, label, hint, cap } => lines.push(Line {
                key: format!("pictured-{at}"),
                heading: heading.take(),
                glyph: glyph.clone(),
                label: label.clone(),
                hint: hint.clone(),
                cap: cap.clone(),
                sub: None,
                done: false,
                does: None,
            }),
        }
    }
    lines.extend(found.map(|(thread, key)| Line::of(thread, key, None)));
    lines
}

/// The thread the keys are on, and whether it is the conversation on screen: the open
/// conversation, or the Inbox's picked row (`focused ?? pointed` in `App.tsx`). Home has none,
/// and neither has a row the app made itself, which is no thread.
fn under_the_keys(s: &State) -> Option<(&Item, bool)> {
    match &s.view {
        View::Thread { .. } => s.open_item().map(|item| (item, true)),
        View::Inbox => s.lists.under(s.inbox_tab).get(s.picked).and_then(|r| s.item(&r.id, &r.product)).map(|item| (item, false)),
        View::Home => None,
    }
}

/// The rows the bar lists for what is typed, in the order drawn.
fn lines(s: &State) -> Vec<Line> {
    let query = s.palette.words.text.as_str();
    let threads = {
        let mut index = s.palette.index.borrow_mut();
        index.read(&s.snap.items);
        rules::threads_found(&index, &s.snap, query, s.now)
    };
    if let Some(pictured) = &s.palette.pictured {
        return pictured_lines(pictured, threads);
    }
    let under = under_the_keys(s);
    let commands = rules::commands(under.map(|(item, _)| item), under.is_some_and(|(_, open)| open), s.now, starts_a_thread().is_some());
    let listed = rules::listed(commands, threads, query);
    let keys = rules::row_keys(&listed.rows);
    let headings: Vec<Option<String>> = (0..listed.rows.len()).map(|at| listed.label_over(at).map(str::to_string)).collect();
    listed.rows.into_iter().zip(keys).zip(headings).map(|((row, key), heading)| Line::of(row, key, heading)).collect()
}

/// Which row is lit: the one picked under these same words, else the first (`keptPick`).
fn lit(s: &State, lines: &[Line]) -> Option<usize> {
    let keys: Vec<String> = lines.iter().map(|line| line.key.clone()).collect();
    let picked = s.palette.lit.as_ref().filter(|(under, _)| *under == s.palette.words.text).map(|(_, key)| key.as_str());
    rules::kept_pick(picked, &keys)
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
        // What a thread's row says of when it comes back is counted from this minute.
        if !self.fixed_clock {
            self.now = crate::clock();
        }
        // Fresh each time, as the web window mounts its box anew: empty, the first row lit, the
        // list at its top. Rune would otherwise carry out again the last request to bring a lit
        // row into view, which outlives the list it was made for.
        self.palette = Bar { open: true, index: std::mem::take(&mut self.palette.index), ..Bar::closed() };
        self.palette.words.focused = true;
        tx.focus_when_shown("palette", false);
        tx.scroll_to_y("palette-list", 0.0);
    }

    pub fn close_palette(&mut self) {
        self.palette = Bar { index: std::mem::take(&mut self.palette.index), ..Bar::closed() };
    }

    /// Opens a finished thread again: what the Reopen button over a conversation writes
    /// (`reopen_thread`), for the thread the keys are on.
    fn reopen(&mut self, id: &str, product: &str) {
        if self.open_item().is_some_and(|item| item.id == id && item.product == product) {
            return self.reopen_thread();
        }
        let Some(item) = self.item(id, product).cloned() else {
            return;
        };
        self.write_what_waits();
        let thread = format!("{}/{}", item.product, item.id);
        self.send(writes::REOPEN, writes::reopen(&item), write::Sent::Reopen { thread });
    }
}

/// Runs a row: the bar closes first, then the row does what it says, each through what the
/// key or the button for it already does.
fn run(s: &mut State, does: Does, tx: &mut Tx) {
    s.close_palette();
    match does {
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
        Does::Reopen((id, product)) => s.reopen(&id, &product),
        Does::Later((id, product)) => s.open_later(&id, &product, tx),
        Does::Back((id, product)) => s.bring_back(&id, &product),
    }
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

/// `--agent-ink`: the placeholder, an unlit row's mark, the headings and the foot.
fn quiet() -> Color {
    rgb(127, 121, 117)
}

/// The small capitals of the mono face: Esc, a heading, the foot. The line is as tall as the
/// type, and the letters stand a tenth or so of it apart.
fn capitals(size: f32, apart: f32) -> TextStyle {
    TextStyle::new(size, 500.0).mono().lh(size).tracking(apart)
}

const WIDTH: f32 = 620.0;
const BOX: f32 = 56.0;
const ROW: f32 = 38.0;
const FOOT: f32 = 30.5;
/// What the list grows to before it scrolls (`.palette-list`'s `max-height`).
const LIST_CAP: f32 = 320.0;

/// The mark at the head of a row, in the middle of its 18 points.
fn glyph(mark: &str, lit: bool) -> El {
    row().w(18.0).justify_center().child(pen(text(mark).font(mono(13.0, 13.0)).color(if lit { accent() } else { quiet() }), 13.0))
}

/// A key at a row's end (`.palette-key`).
fn key_cap(cap: &str) -> El {
    row().min_w(22.0).h(20.0).px(5.0).center().radius(6.0).fill(page()).border(1.0, edge()).border_layout().child(text(cap).font(mono(11.5, 15.0)).color(dim()))
}

/// A heading over a section. The first thing in the list stands closer to its top.
fn heading(words: &str, first: bool) -> El {
    el().pt(if first { 8.0 } else { 14.0 }).px(18.0).pb(6.0).child(text(words.to_uppercase()).font(capitals(9.5, 1.14)).color(quiet()))
}

/// How tall a row is: a thread with a line to say stands on two.
fn height(line: &Line) -> f32 {
    if line.sub.is_some() { 51.5 } else { ROW }
}

fn line_row(line: &Line, lit: bool, query: &str) -> El {
    // A finished thread steps back; the highlight behind a lit one does not (`.palette-done > *`).
    let fade = |e: El| if line.done { e.opacity(0.5) } else { e };
    let name = pen(label(&line.label, 14.5, 400.0, if lit { ink() } else { dim() }).truncate(), 14.5);
    let hint = line.hint.as_deref().map(|hint| pen(label(hint, 12.5, 400.0, muted()), 12.5));
    let mut e = match &line.sub {
        // A thread found: its title, then the line it was found by with the typed words in
        // ink, and where it is and how old beside them. A long title gives way, never the
        // project's name.
        Some(sub) => {
            let (mut said, mut marks) = (String::new(), Vec::new());
            for run in search::marked_words(sub, js::trim(query)) {
                let from = said.len();
                said.push_str(&run.text);
                if run.hit {
                    marks.push(Span::new(from, said.len()).weight(600.0).color(ink()));
                }
            }
            let found_by = pen(text(said).font(sans(12.0, 400.0)).color(muted()).spans(marks).truncate(), 12.0);
            let mut e = row().min_h(50.0).py(7.0).px(18.0).gap(12.0).items_start().child(fade(el().pt(3.0).child(glyph(&line.glyph, lit)))).child(fade(col().grow(1.0).min_w(0.0).gap(3.0).child(name).child(found_by)));
            if let Some(hint) = hint {
                e = e.child(fade(el().pt(2.0).child(hint)));
            }
            e
        }
        None => {
            let mut e = row().h(ROW).px(18.0).gap(12.0).items_center().child(fade(glyph(&line.glyph, lit))).child(fade(row().grow(1.0).min_w(0.0).child(name)));
            match (&line.cap, hint) {
                (Some(cap), _) => e = e.child(fade(key_cap(cap))),
                (None, Some(hint)) => e = e.child(fade(hint)),
                _ => {}
            }
            e
        }
    };
    if lit {
        e = e.fill(white(0.04)).child(el().pin(Some(0.0), Some(0.0), None, Some(0.0)).w(2.0).fill(accent()));
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

    let field = text_input(&s.palette.words, sans(17.0, 400.0))
        .key("palette")
        .single_line()
        .grow(1.0)
        .color(ink())
        .placeholder(rules::PLACEHOLDER, quiet())
        .caret(accent(), rgba(110, 165, 255, 0.3))
        .on_edit(|s: &mut State| &mut s.palette.words)
        .on_submit(run_the_lit_row)
        .on_input_key(in_the_box);
    let esc = row()
        .px(6.0)
        .py(4.0)
        .radius(6.0)
        .border(1.0, edge())
        .border_layout()
        .cursor(Cursor::Pointer)
        .child(text("ESC").font(capitals(10.0, 1.0)).color(muted()))
        .on_press(|s: &mut State| s.close_palette());
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
        rows = rows.child(line_row(line, lit == Some(at), query));
        tall += height(line);
    }
    // A column: a plain box is a row, which stretches the rows to its own height, and then
    // there is nothing left over to scroll to.
    let list = col().key("palette-list").h(tall.min(LIST_CAP)).scroll(ScrollSpec { stick_bottom: false }).child(rows);

    // What the keys do, said once at the foot.
    let keys = |keys: &str, does: &str| text(format!("{keys} {does}")).font(capitals(9.5, 1.14)).color(quiet()).spans(vec![Span::new(0, keys.len()).color(muted())]);
    let foot = row().px(18.0).py(10.0).gap(18.0).border_sides([1.0, 0.0, 0.0, 0.0], edge()).border_layout().child(keys("↑ ↓", "MOVE")).child(keys("↵", "SELECT"));

    // The box stands where a command bar stands: its middle three eighths of the way down the
    // window, at every window size (`place-items: center` over a quarter of the height held
    // back under it, in `.modal-backdrop.palette-backdrop`).
    // No rounded clip: Rune's takes the card's own shadow with it, and nothing inside reaches
    // a corner (the list scrolls inside its own bounds).
    let card_tall = 2.0 + BOX + tall.min(LIST_CAP) + FOOT;
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
        .child(list)
        .child(foot);
    // `--scrim` over the whole window, as this look has it.
    Some(el().absolute(0.0, 0.0).size(w, h).fill(rgba(0, 0, 0, 0.72)).child(card))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn found(id: &str, title: &str) -> rules::Row {
        let item: Item = serde_json::from_value(json!({ "id": id, "product": "shop", "status": "open", "title": title, "updatedAt": 1 })).unwrap();
        rules::thread_row(&item, title.to_string(), "Storefront", 2.0)
    }

    #[test]
    fn a_picture_draws_the_webs_commands_and_this_windows_threads_in_their_places() {
        let rows = json!([
            { "glyph": "↻", "label": "Check for updates", "hint": "look now", "key": "" },
            { "section": "Threads" },
            { "thread": true },
            { "thread": true },
            { "glyph": "", "label": "" },
        ]);
        let pictured: Vec<Pictured> = rows.as_array().unwrap().iter().filter_map(Pictured::read).collect();
        // A row with no name is no row.
        assert_eq!(pictured.len(), 4);
        let lines = pictured_lines(&pictured, vec![found("w-1", "Checkout fails"), found("w-2", "Onboarding checklist copy"), found("w-3", "Not listed by the web")]);
        let said: Vec<(&str, Option<&str>)> = lines.iter().map(|l| (l.label.as_str(), l.heading.as_deref())).collect();
        assert_eq!(said, [("Check for updates", None), ("Checkout fails", Some("Threads")), ("Onboarding checklist copy", None), ("Not listed by the web", None)]);
        // The web's command does nothing here, and the threads open as they would anywhere.
        assert!(lines[0].does.is_none() && lines[0].cap.is_none() && lines[0].hint.as_deref() == Some("look now"));
        assert_eq!(lines[1].does, Some(Does::Open(("w-1".to_string(), "shop".to_string()))));
        assert_eq!((lines[1].glyph.as_str(), lines[1].hint.as_deref()), ("›", Some("Storefront · now")));
        // With fewer threads found than the web listed, the places left over stay empty.
        assert_eq!(pictured_lines(&pictured, vec![found("w-1", "Checkout fails")]).len(), 2);
    }
}
