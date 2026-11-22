//! Later: the picker L opens over the page (`renderer/src/components/Snooze.tsx` and the
//! `SchedulePicker` under it). A box to type a time in words, with what it read beside it, over
//! the moments it offers. What is typed decides; with nothing typed, the lit row does.

use rune::edit::Key;
use rune::text::Span;
use rune::{El, KeyEvent, TextEdit, Tx, col, el, row, text, text_input};

use crate::data::Item;
use crate::rules::{later as rules, list, page};
use crate::theme::*;
use crate::State;

/// The picker's own state. It is always there; `on` says whether it is open, and on what.
pub struct Picker {
    /// The thread being put off, as `(id, product)`.
    pub on: Option<(String, String)>,
    pub words: TextEdit,
    /// The row the arrows are on.
    pub selected: usize,
    /// Enter was pressed on words that are not a time, and nothing was typed since.
    pub refused: bool,
}

impl Picker {
    pub fn closed() -> Picker {
        Picker { on: None, words: TextEdit::new(""), selected: 0, refused: false }
    }
    /// Open on a thread, or closed again: empty, with its first row lit.
    pub fn open_on(thread: Option<(String, String)>, words: &str) -> Picker {
        let mut words = TextEdit::new(words);
        words.focused = thread.is_some();
        Picker { on: thread, words, selected: 0, refused: false }
    }
}

/// One row of the picker: its name, the moment it means, and that moment beside it.
pub struct Choice {
    pub name: &'static str,
    /// 0 is "Now": the thread comes back instead of being put off again.
    pub ts: f64,
    pub hint: String,
}

/// When a thread that was put off comes back (`returnAt` in `renderer/src/App.tsx`).
pub fn return_at(item: &Item) -> f64 {
    list::deferred_until(item).max(list::hidden_until(item, 0.0))
}

/// The rows offered for a thread. One already put off gets "Now" first, so L then Enter is
/// the way back to the inbox.
pub fn choices(item: &Item, now: f64) -> Vec<Choice> {
    let mut rows = Vec::new();
    if return_at(item) > now {
        rows.push(Choice { name: "Now (back to inbox)", ts: 0.0, hint: String::new() });
    }
    rows.extend(rules::presets(now).into_iter().map(|(name, ts)| Choice { name, ts, hint: page::day_and_time(ts, now).to_uppercase() }));
    rows
}

/// The keys the box does not keep for typing: the arrows walk the rows, and anything typed
/// takes back a refusal. Escape and Return are the picker's own (`modal`, `on_submit`).
fn in_the_box(s: &mut State, e: &KeyEvent, _: &mut Tx) -> bool {
    match &e.key {
        Key::Down | Key::Up => {
            let rows = s.later_choices().len();
            let at = s.later.selected;
            s.later.selected = if e.key == Key::Down { (at + 1).min(rows.saturating_sub(1)) } else { at.saturating_sub(1) };
            true
        }
        Key::Text(_) | Key::Backspace | Key::Delete => {
            s.later.refused = false;
            false
        }
        _ => false,
    }
}

/// The same keys when the box has lost the caret: the picker still answers them.
pub fn pressed(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> bool {
    match &e.key {
        Key::Escape => s.close_later(),
        Key::Enter => s.pick_later(None),
        _ => return in_the_box(s, e, tx),
    }
    true
}

fn choice_row(at: usize, choice: &Choice, lit: bool) -> El {
    let mut line = row()
        .key(format!("later-{at}"))
        .h(34.0)
        .px(18.0)
        .gap(16.0)
        .justify_between()
        // The two stand on one baseline (`align-items: baseline`), which puts the smaller
        // words a point and a half lower in the row.
        .child(pen(label(choice.name, 14.0, 400.0, ink()), 14.0).mt(8.0))
        .child(pen(label(&choice.hint, 12.5, 400.0, muted()), 12.5).mt(9.5));
    // The lit row is the command bar's: a wash, with no accent down its left edge any more
    // (`.palette-item.selected`).
    if lit {
        line = line.fill(white(0.04));
    }
    // The pointer lights the row it moves over, never the one it happened to be resting on
    // when the picker opened, and takes the pointing hand over it (`.palette-item`).
    line.cursor(rune::Cursor::Pointer).on_pointer_move(move |s: &mut State, _| s.later.selected = at).on_press(move |s: &mut State| s.pick_later(Some(at)))
}

/// The picker, over the whole window, when it is open.
pub fn view(s: &State) -> Option<El> {
    let (id, product) = s.later.on.as_ref()?;
    let item = s.snap.items.iter().find(|i| &i.id == id && &i.product == product)?;
    let (w, h) = s.size;
    let width = 620.0_f32.min(0.92 * w);
    let typed = s.later.words.text.as_str();
    let read = rules::parse_when(typed, s.now).map(|(ts, _)| page::day_and_time(ts, s.now).to_uppercase());
    let preview = rules::preview_line(typed, read.as_deref(), s.later.refused, rules::HINT_WHEN_REFUSED);

    let field = text_input(&s.later.words, sans(16.0, 400.0))
        .key("later")
        .single_line()
        .grow(1.0)
        .color(ink())
        .placeholder(rules::PLACEHOLDER, rgb(117, 117, 117))
        .caret(ink(), rgba(110, 165, 255, 0.3))
        .on_edit(|s: &mut State| &mut s.later.words)
        .on_submit(|s: &mut State, _| s.pick_later(None))
        .on_input_key(in_the_box);
    // The whole line is the box to type in (`input.snooze-input`): the I-beam over it, and a press
    // anywhere on it puts the caret there.
    let mut ask = row()
        .h(49.5)
        .px(18.0)
        .items_center()
        .border_sides([0.0, 0.0, 1.0, 0.0], edge())
        .border_layout()
        .cursor(rune::Cursor::Text)
        .on_press_tx(|s: &mut State, tx| {
            s.later.words.focused = true;
            tx.focus("later", false);
        })
        .child(pen(field, 16.0));
    if let Some((line, was_read)) = preview {
        // What it read stands upright; "not a time" leans, fainter until Enter was refused.
        let colour = if was_read || s.later.refused { dim() } else { muted() };
        let mut words = text(line.as_str()).font(mono(12.0, 15.5)).color(colour);
        if !was_read {
            words = words.spans(vec![Span::new(0, line.len()).italic()]);
        }
        ask = ask.child(el().pin(None, Some(17.0), Some(18.0), None).child(words));
    }

    // With words in the box no row is lit: what is typed decides.
    let rows = choices(item, s.now);
    let list = col().py(6.0).children(rows.iter().enumerate().map(|(at, choice)| choice_row(at, choice, typed.is_empty() && at == s.later.selected)));

    // Not clipped to its corners: Rune's clip takes the card's own shadow with it, and nothing
    // inside reaches the corners (the list stands 6 points in from the foot) to need one.
    let card = col()
        .absolute((w - width) / 2.0, 0.12 * h)
        .w(width)
        .radius(10.0)
        .fill(rgb(47, 45, 44))
        .border(1.0, rule())
        .border_layout()
        .box_shadow(shadow(0.0, 24.0, 70.0, 0.0, rgba(0, 0, 0, 0.55), false))
        .modal_tx(true, |s: &mut State, _| s.close_later())
        .child(el().pt(14.0).px(18.0).pb(2.0).child(pen(label("Later", 15.5, 600.0, ink()), 15.5)))
        // A long name takes the lines it needs, as a block of text does in the web window.
        .child(el().px(18.0).pb(10.0).child(pen(label(&crate::rules::title::row_title(item), 13.0, 400.0, dim()).wrap().max_w(width - 38.0), 13.0)))
        .child(ask)
        .child(list);
    // `--scrim` over the whole window, as this look has it.
    Some(el().absolute(0.0, 0.0).size(w, h).fill(rgba(0, 0, 0, 0.72)).child(card))
}
