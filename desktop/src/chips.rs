//! The reply box's priority chip and its mode chip, and the menu each opens, as the web window's
//! box draws them (`PriorityPicker` and `ModePicker` in `DockComposer`,
//! `renderer/src/components/Focus.tsx`; the menu is `.prio-menu` in `renderer/src/styles.css`,
//! risen eight points over its chip from its left edge). The rules are `rules/reply_chips.rs`.
//!
//! A pick is kept by the open box, not written: the next reply carries it (`said`). The box
//! folding lets both picks go, as the web window's box is made anew each time it opens, and an
//! opening box reads the thread's own mode afresh.
//!
//! The priority's menu opens with the keys on the level shown; the pointer and the arrows walk
//! it around both ends, Enter picks, and Escape closes it and leaves the keys on the chip, where
//! an arrow, Enter or Space opens it again. The mode's menu has no row under the keys: a press
//! picks, Enter or Space on it closes it as a press on its chip does, and Escape closes it and
//! puts the caret back in the words, as a pick does. Shift+Tab in the words moves the mode round
//! the agent's ring, and says so. Any other key goes on to the page, as the web window lets keys
//! through while the reply box is up and the caret is not in it (`keyWhileABoxIsOpen`).

use rune::edit::Key;
use rune::{Drag, El, KeyEvent, Phase, ScrollSpec, Tx, col, el, row, text};

use crate::compose::level_mark;
use crate::data::Item;
use crate::rules::compose::{self as compose_rules, Hang, LEVELS, Level, level_word};
use crate::rules::reply_chips::{self as rules, Said};
use crate::theme::*;
use crate::{State, chrome, home, icons};

/// The two chips, which the menus rise out of.
pub const PRIORITY: &str = "reply-priority";
pub const MODE: &str = "reply-mode";
/// The open menu's rows, whose width decides where it fits.
const MENU: &str = "reply-chip-menu";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chip {
    Priority,
    Mode,
}

impl Chip {
    fn key(self) -> &'static str {
        match self {
            Chip::Priority => PRIORITY,
            Chip::Mode => MODE,
        }
    }
}

/// The menu that is open, and the row the keys are on (the priority's; the mode's has none).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Open {
    pub chip: Chip,
    pub lit: Option<usize>,
}

/// What the open reply box holds of its two chips.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chips {
    /// The box these are of, as `product/id`, while it is open.
    thread: Option<String>,
    /// The level picked; none while untouched, when the chip shows the thread's own (`prio`).
    pub level: Option<Level>,
    /// What the next reply may do: the thread's own mode when the box opened, then whatever is
    /// picked; none is the project's (`mode`).
    pub mode: Option<String>,
    pub menu: Option<Open>,
    /// The chip the keys were left on by Escape or a pick made with them.
    pub on_chip: Option<Chip>,
}

fn key_of(item: &Item) -> String {
    format!("{}/{}", item.product, item.id)
}

/// The box opened, or folded, or another thread's is open: the chips start again on that box,
/// with its thread's own mode (`useState(item.answerMode ?? null)`, keyed by the thread).
pub fn tend(s: &mut State) {
    let open = if s.replying { s.open_item().map(|item| (key_of(item), item.text("answerMode").map(str::to_string))) } else { None };
    match open {
        Some((thread, mode)) if s.chips.thread.as_deref() != Some(thread.as_str()) => s.chips = Chips { thread: Some(thread), mode, ..Chips::default() },
        None if s.chips.thread.is_some() => s.chips = Chips::default(),
        _ => {}
    }
}

/// What a reply to `item` carries of the chips: their picks while its box is the open one, else
/// the thread's own mode and no priority.
pub fn said(s: &State, item: &Item) -> Said {
    if s.chips.thread.as_deref() == Some(key_of(item).as_str()) {
        return Said { level: s.chips.level, mode: s.chips.mode.clone() };
    }
    Said::untouched(item)
}

/// Whether the thread's next reply reaches Claude Code, whose modes the chip offers.
fn claude(s: &State, item: &Item) -> bool {
    rules::claude_code(crate::rules::reply_pick::running_engine(&s.snap, &item.id).as_deref())
}

/// What the next reply runs as, by the mode's value.
fn running(s: &State, item: &Item) -> String {
    let mode = if s.chips.thread.as_deref() == Some(key_of(item).as_str()) { s.chips.mode.clone() } else { item.text("answerMode").map(str::to_string) };
    rules::running(mode.as_deref(), claude(s, item), rules::project_mode(&s.snap, item))
}

/// The level the chip shows: the one picked, else the thread's own.
fn shown_level(s: &State, item: &Item) -> Level {
    s.chips.level.filter(|_| s.chips.thread.as_deref() == Some(key_of(item).as_str())).unwrap_or_else(|| rules::shown_level(item))
}

/// Whether the box offers a mode at all: not on a terminal agent's row, whose message goes
/// straight into it (`canSetMode`).
pub fn offers_a_mode(item: &Item) -> bool {
    item.rest.get("agent").is_none_or(serde_json::Value::is_null)
}

/// A chip of the box (`.tc-chip`), brighter inside a stronger edge while its menu is open
/// (`.tc-chip.open`), titled as the web window's.
fn chip(s: &State, which: Chip, kids: Vec<El>, title: &str) -> El {
    let open = s.chips.menu.is_some_and(|m| m.chip == which);
    let c = home::chip(kids).key(which.key()).cursor(rune::Cursor::Pointer).help(title.to_string()).on_background_press_tx(move |s: &mut State, tx| toggle(s, which, tx));
    if open { c.border(1.0, rule()) } else { c }
}

fn word(said: &str, lit: bool) -> El {
    pen(label(said, 13.0, 400.0, if lit { ink() } else { dim() }).group_color(ink()), 13.0)
}

/// The priority chip: the level's bars and its word (`PriorityPicker variant="chip"`).
pub fn priority_chip(s: &State, item: &Item) -> El {
    let level = shown_level(s, item);
    let open = s.chips.menu.is_some_and(|m| m.chip == Chip::Priority);
    chip(s, Chip::Priority, vec![level_mark(Some(level)), word(level_word(level), open)], "Priority").label(format!("Priority: {}", level_word(level)))
}

/// The mode chip: its shield and the mode the next reply runs in, title cased (`ModePicker chip`).
pub fn mode_chip(s: &State, item: &Item) -> El {
    let open = s.chips.menu.is_some_and(|m| m.chip == Chip::Mode);
    let said = rules::chip_word(&running(s, item), claude(s, item));
    chip(s, Chip::Mode, vec![icon_mark(), word(&said, open)], "What this message may do")
}

fn icon_mark() -> El {
    rune::icon(&icons::MODE_MARK, 14.0).color(muted())
}

/// A press on a chip: its menu opens, or closes when it is the one open (`setOpen((o) => !o)`).
/// Opening it takes the keys out of the words, as the web window's chip takes the focus, and
/// closes whatever else stands open over the conversation.
pub fn toggle(s: &mut State, which: Chip, tx: &mut Tx) {
    tend(s);
    if s.chips.menu.take().is_some_and(|m| m.chip == which) {
        s.chips.on_chip = Some(which);
        return;
    }
    let Some(item) = s.open_item().cloned() else {
        return;
    };
    s.close_strip();
    s.reply_menu = None;
    s.reply.focused = false;
    tx.blur();
    let lit = match which {
        Chip::Priority => Some(LEVELS.iter().position(|l| *l == shown_level(s, &item)).unwrap_or(0)),
        Chip::Mode => None,
    };
    s.chips.on_chip = Some(which);
    s.chips.menu = Some(Open { chip: which, lit });
}

/// The rows of the open menu, as the words they say and whether each is what the box is on.
fn rows(s: &State, item: &Item, which: Chip) -> Vec<(String, bool)> {
    match which {
        Chip::Priority => {
            let shown = shown_level(s, item);
            LEVELS.iter().map(|l| (level_word(*l).to_string(), *l == shown)).collect()
        }
        Chip::Mode => {
            let now = running(s, item);
            rules::rows(claude(s, item)).iter().map(|(value, label)| (label.to_string(), *value == now)).collect()
        }
    }
}

/// A row picked, by a press (`keys` false) or Enter. The priority's pick is kept and the keys
/// stay on its chip after Enter, while a press leaves them on nothing, as the pressed row goes
/// with its menu. The mode's is kept, said in the strip, and puts the caret back in the words,
/// which it empties only of a slash query (`pickMode`).
pub fn pick(s: &mut State, which: Chip, at: usize, keys: bool, tx: &mut Tx) {
    let Some(item) = s.open_item().cloned() else {
        return;
    };
    s.chips.menu = None;
    match which {
        Chip::Priority => {
            let Some(level) = LEVELS.get(at).copied() else {
                return;
            };
            s.chips.level = Some(level);
            s.chips.on_chip = keys.then_some(Chip::Priority);
        }
        Chip::Mode => {
            let claude = claude(s, &item);
            let Some((mode, _)) = rules::rows(claude).get(at).copied() else {
                return;
            };
            set_mode(s, &item, mode, tx);
            if rules::clears_the_words(&s.reply.text) {
                s.reply = rune::TextEdit::new("");
            }
            s.chips.on_chip = None;
            s.reply.focused = true;
        }
    }
}

/// The box's mode set, and said (`setMode`, `announce`).
fn set_mode(s: &mut State, item: &Item, mode: &str, tx: &Tx) {
    let claude = claude(s, item);
    s.chips.mode = Some(mode.to_string());
    let line = rules::announce(Some(mode), claude, rules::project_mode(&s.snap, item));
    s.tell(&line, tx);
}

/// Shift+Tab in the words: the next mode round the agent's ring, said in the strip.
pub fn shift_tab(s: &mut State, tx: &mut Tx) -> bool {
    tend(s);
    let Some(item) = s.open_item().cloned() else {
        return false;
    };
    if !offers_a_mode(&item) {
        return false;
    }
    let next = rules::next(&running(s, &item), claude(s, &item));
    set_mode(s, &item, next, tx);
    true
}

/// The keys while a chip's menu is open or the keys were left on a chip. `None` when they are
/// not the chips', and `Some(false)` for a key that goes on to the page.
pub fn key(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> Option<bool> {
    if s.chips.menu.is_none() && s.chips.on_chip.is_none() {
        return None;
    }
    if e.mods.cmd || e.mods.ctrl || e.mods.alt {
        return Some(false);
    }
    let space = matches!(&e.key, Key::Text(t) if t == " ");
    let Some(open) = s.chips.menu else {
        // On a chip with its menu shut: Enter and Space press it, and on the priority's the
        // arrows open it too (`onKeyDown`). Any other key goes on to the page.
        let on = s.chips.on_chip.take()?;
        if space || e.key == Key::Enter || (on == Chip::Priority && matches!(e.key, Key::Up | Key::Down)) {
            toggle(s, on, tx);
            return Some(true);
        }
        return Some(false);
    };
    match (open.chip, &e.key) {
        (Chip::Priority, Key::Escape) => {
            s.chips.menu = None;
            s.chips.on_chip = Some(Chip::Priority);
        }
        (Chip::Priority, Key::Down | Key::Up) => {
            let at = compose_rules::entry_step(open.lit.unwrap_or(0), LEVELS.len(), e.key == Key::Down);
            s.chips.menu = Some(Open { lit: Some(at), ..open });
        }
        (Chip::Priority, Key::Enter) => pick(s, Chip::Priority, open.lit.unwrap_or(0), true, tx),
        // The mode's chip keeps the keys: Enter and Space press it, which closes its menu.
        (Chip::Mode, Key::Enter) => {
            s.chips.menu = None;
            s.chips.on_chip = Some(Chip::Mode);
        }
        (Chip::Mode, _) if space => {
            s.chips.menu = None;
            s.chips.on_chip = Some(Chip::Mode);
        }
        (Chip::Mode, Key::Escape) => {
            s.chips.menu = None;
            s.chips.on_chip = None;
            s.reply.focused = true;
        }
        _ => return Some(false),
    }
    Some(true)
}

/// The keys go back to the words, and the chips let go of them.
pub fn let_go(s: &mut State) {
    s.chips.menu = None;
    s.chips.on_chip = None;
}

/// A press outside the open menu, on its way down, with where the box's controls stood: the
/// menu closes, as the web window's `pointerdown` listener closes it, and the press goes on to
/// what it was aimed at in the box. Its own chip only closes it; the other opens its own; the
/// model's opens the model menu; the words take the caret; Send sends. Anywhere else it only closes.
fn pressed_away(s: &mut State, at: [f32; 2], marks: &[(&'static str, [f32; 4])], tx: &mut Tx) {
    let inside = |key: &str| marks.iter().any(|(mark, [x, y, w, h])| *mark == key && at[0] >= *x && at[0] < x + w && at[1] >= *y && at[1] < y + h);
    let Some(open) = s.chips.menu.take() else {
        return;
    };
    s.chips.on_chip = None;
    if inside(open.chip.key()) {
        s.chips.on_chip = Some(open.chip);
        return;
    }
    for other in [Chip::Priority, Chip::Mode] {
        if inside(other.key()) {
            return toggle(s, other, tx);
        }
    }
    if inside(crate::reply::CHIP) {
        return s.toggle_reply_menu(tx);
    }
    if inside(crate::reply::SEND) {
        return s.send_reply(tx);
    }
    if inside(crate::reply::FIELD) {
        s.reply.focused = true;
    }
}

/// One row of the menu (`.prio-menu-row`): 29 points, its mark for a level, its words, the row
/// under the keys washed, the one the box is on in the text's ink and heavy.
fn menu_row(which: Chip, at: usize, words: &str, on: bool, lit: bool) -> El {
    let mut line = row().key(format!("chip-row-{at}")).h(29.0).px(9.0).gap(9.0).items_center().radius(6.0).cursor(rune::Cursor::Pointer);
    if lit {
        line = line.fill(white(0.08));
    }
    if which == Chip::Priority {
        line = line.child(level_mark(LEVELS.get(at).copied()));
    }
    let tone = if on || lit { ink() } else { dim() };
    let said = text(words.to_string()).font(sans(13.0, if on { 600.0 } else { 400.0 })).color(tone);
    line = line.child(row().grow(1.0).min_w(0.0).child(pen(said, 13.0)));
    // A press picks; the pointer moving onto a priority row puts the keys on it (`onPointerEnter`).
    let line = line.on_press_tx(move |s: &mut State, tx| pick(s, which, at, false, tx));
    if which == Chip::Priority {
        line.on_pointer_move_local(move |s: &mut State, _, tx: &mut Tx| {
            match s.chips.menu {
                Some(open) if open.lit != Some(at) => s.chips.menu = Some(Open { lit: Some(at), ..open }),
                _ => tx.live_only(),
            }
        })
    } else {
        line
    }
}

/// The open menu over the page, risen eight points over its chip from its left edge
/// (`.dock-card .compose-word-wrap .prio-menu`), kept inside the window (`useKeepInWindow`), and
/// seen only inside the page. Under it lies a layer the size of the window, and a press there
/// closes it on the way down (`pressed_away`).
pub fn layer(s: &State) -> Option<El> {
    let open = s.chips.menu?;
    let item = s.open_item()?;
    let anchor = rune::memo::window_layout(open.chip.key())?;
    let rows = rows(s, item, open.chip);
    let count = rows.len();
    // Four points inside a one point edge, a point between rows (`padding: 4px; gap: 1px`).
    let tall = 2.0 + 8.0 + count as f32 * 29.0 + count.saturating_sub(1) as f32;
    // At least 168 points with its edge (`min-width: 168px`), wider for longer words.
    let wide = rune::memo::layout_size(MENU).map_or(168.0, |[w, _]| w + 2.0);
    let (w, h) = s.size;
    let left = chrome::card_left(s);
    let fit = compose_rules::fit_menu(anchor, Hang { below: false, gap: 8.0, dx: 0.0 }, (wide, tall), h, (left, w - chrome::CARD[2]));
    let list = col().key(MENU).min_w(166.0).px(4.0).py(4.0).gap(1.0).items_stretch().instant_changes().children(rows.iter().enumerate().map(|(at, (words, on))| menu_row(open.chip, at, words, *on, open.lit == Some(at))));
    let shell = col().radius(10.0).fill(rgb(54, 52, 49)).border(1.0, rule()).border_layout().box_shadow(shadow(0.0, 24.0, 70.0, 0.0, rgba(0, 0, 0, 0.55), false));
    let shell = match fit.max_height {
        Some(most) => shell.h(most).child(col().grow(1.0).min_h(0.0).scroll(ScrollSpec { stick_bottom: false }).child(list)),
        None => shell.child(list),
    };
    // It comes in as the web window's does, fading in as it rises its last four points from
    // 98% of its size (`prio-in`, 0.13 s on `cubic-bezier(0.2, 0.8, 0.3, 1)`, the curve the
    // box's other menus stand a spring for), and goes at once.
    let comes_in = rune::Transition::offset(0.0, 4.0).combined(rune::Transition::scale(0.98)).combined(rune::Transition::opacity());
    let shell = shell
        .animated_changes()
        .motion(rune::Spring::NONE)
        .transition(rune::Transition::asymmetric(comes_in, rune::Transition::none()).springs(rune::SpringPair::new(rune::Spring::new(0.028, -0.66), rune::Spring::NONE)))
        .role("listbox")
        .label(match open.chip {
            Chip::Priority => "Priority",
            Chip::Mode => "What this message may do",
        });
    let marks: Vec<(&'static str, [f32; 4])> = [PRIORITY, MODE, crate::reply::CHIP, crate::reply::SEND, crate::reply::FIELD].into_iter().filter_map(|key| rune::memo::window_layout(key).map(|rect| (key, rect))).collect();
    let away = el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).on_drag(move |s: &mut State, d: &mut Drag, tx: &mut Tx| {
        if d.phase == Phase::Down {
            pressed_away(s, d.window_point(), &marks, tx);
        }
    });
    let page_top = chrome::page_top(&s.view);
    let page = el().pin(Some(left), Some(page_top), Some(chrome::CARD[2]), Some(chrome::CARD[3])).clip_corners([0.0, 0.0, 6.0, 6.0]).child(shell.absolute(fit.left - left, fit.top - page_top));
    Some(el().absolute(0.0, 0.0).size(w, h).child(away).child(page))
}

/// What a picture taken with a chip's menu open says (`actions`: `reply-priority`,
/// `reply-mode`), and one taken after picks were made in them (`replyPicked`, the level's id and
/// the mode's value), made here the way the scene made them: each chip pressed, its row pressed.
pub fn picture(s: &mut State, which: &str, tx: &mut Tx) -> bool {
    tend(s);
    match which {
        "reply-priority" => toggle(s, Chip::Priority, tx),
        "reply-mode" => toggle(s, Chip::Mode, tx),
        _ => return false,
    }
    true
}

/* ------------------------- the choices' little arrow ------------------------- */

/// Whether this thread's current round of choices was folded away (`choicesFolded`).
pub fn choices_folded(s: &State, item: &Item) -> bool {
    let key = rules::fold_key(item);
    s.picks.get().choices_folded.iter().any(|k| *k == key)
}

/// The little arrow in the choices' corner, or the sign on the folded box: folds this round of
/// choices away, or brings it back, kept from one window to the next (`foldChoices`). The keys
/// pick any choice either way.
pub fn toggle_choices(s: &mut State) {
    let Some(key) = s.open_item().map(rules::fold_key) else {
        return;
    };
    s.picks.change(|picks| picks.choices_folded = rules::toggle_fold(&picks.choices_folded, &key));
}

/// With the choices folded, the sign on the folded reply box that they are there: how many, and
/// an arrow to bring them back, after R's cap (`ChoicesShow`, `.one-choices-show`). Its press is
/// its own and does not open the box.
pub fn choices_sign(count: usize) -> El {
    row()
        .ml(-4.0)
        .mr(8.0)
        .px(6.0)
        .py(3.0)
        .gap(3.0)
        .items_center()
        .radius(6.0)
        .hover_group()
        .hover(crate::feel::wash())
        .cursor(rune::Cursor::Pointer)
        .label(format!("Show the {count} choices"))
        .child(text(count.to_string()).font(mono(11.0, 14.0)).color(muted()).group_color(ink()))
        .child(rune::icon(&icons::UP, 13.0).color(muted()).group_color(ink()))
        .on_background_press_tx(|s: &mut State, _| toggle_choices(s))
}

#[cfg(test)]
mod tests {
    use rune::{Cursor, Key, KeyEvent, Mods, Runtime};
    use serde_json::{Value, json};

    use super::{Chip, MODE, PRIORITY};
    use crate::State;
    use crate::feel::tests::{key, press, window_dressed};

    /// The invented inbox's first thread, on `engine`, every project running in `permission`,
    /// its reply box opened with R.
    fn replying(name: &str, engine: &str, permission: &str) -> Runtime<State> {
        let mut rt = window_dressed(name, &["--thread", "w-feel0"], "/fixtures/storefront", "Done.", |fixture| {
            fixture["snapshot"]["config"] = json!({ "permission": { "": permission } });
            fixture["snapshot"]["engines"] = json!({ "workspace": "claude", "choices": [{ "id": "claude", "label": "Claude Code" }, { "id": "codex", "label": "Codex" }], "byItem": { "w-feel0": engine } });
        });
        key(&mut rt, Key::Text("r".into()));
        assert!(rt.state.replying && rt.state.reply.focused);
        rt
    }

    fn said(rt: &Runtime<State>, words: &str) -> bool {
        rt.elements().iter().any(|e| e.text.as_deref() == Some(words))
    }

    fn line(rt: &Runtime<State>) -> Option<String> {
        rt.state.toast.as_ref().map(|t| t.line.clone())
    }

    fn shift_tab(rt: &mut Runtime<State>) {
        rt.key(KeyEvent { key: Key::Tab, mods: Mods { shift: true, ..Mods::default() } });
        rt.frame(1.0 / 60.0);
    }

    fn send(rt: &mut Runtime<State>, words: &str) -> Value {
        rt.update(|s, _| s.reply.insert(words));
        rt.key(KeyEvent { key: Key::Enter, mods: Mods { cmd: true, ..Mods::default() } });
        rt.frame(1.0 / 60.0);
        rt.state.held.as_ref().map(|h| h.payload.clone()).expect("the reply is held its three seconds")
    }

    #[test]
    fn the_priority_menu_opens_on_the_level_shown_and_the_keys_walk_and_pick() {
        let mut rt = replying("chips-priority", "claude", "auto");
        let (x, y) = rt.find(PRIORITY).expect("the priority chip");
        rt.pointer_move(x, y);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.cursor(), Cursor::Pointer);
        press(&mut rt, x, y);
        let open = rt.state.chips.menu.expect("its menu");
        // The invented thread is untagged, so it shows Medium, the third of four, lit.
        assert_eq!((open.chip, open.lit), (Chip::Priority, Some(2)));
        assert!(!rt.state.reply.focused, "the keys left the words for the menu");
        assert!(said(&rt, "Urgent") && said(&rt, "Low"));
        key(&mut rt, Key::Down);
        key(&mut rt, Key::Down);
        assert_eq!(rt.state.chips.menu.and_then(|m| m.lit), Some(0), "round the end");
        key(&mut rt, Key::Up);
        key(&mut rt, Key::Enter);
        assert!(rt.state.chips.menu.is_none());
        assert_eq!(rt.state.chips.level, Some(crate::rules::compose::Level::Low));
        assert!(said(&rt, "Low"), "the chip says the pick");
        // The keys stay on the chip, where an arrow opens it again; Escape closes it there.
        key(&mut rt, Key::Down);
        assert_eq!(rt.state.chips.menu.map(|m| m.lit), Some(Some(3)), "on the level picked");
        key(&mut rt, Key::Escape);
        assert!(rt.state.chips.menu.is_none() && rt.state.replying);
        // Nothing was written: the pick waits for the reply.
        assert!(rt.state.held.is_none());
    }

    #[test]
    fn the_mode_menu_lists_the_agents_own_modes_and_a_pick_is_said_and_puts_the_caret_back() {
        let mut rt = replying("chips-mode", "claude", "bypassPermissions");
        assert!(said(&rt, "Bypass Permissions"), "the project's own mode, title cased");
        let (x, y) = rt.find(MODE).expect("the mode chip");
        press(&mut rt, x, y);
        assert_eq!(rt.state.chips.menu.map(|m| (m.chip, m.lit)), Some((Chip::Mode, None)), "no row under the keys");
        for words in ["Manual", "Accept edits", "Plan", "Auto", "Don't Ask"] {
            assert!(said(&rt, words), "{words}");
        }
        let (px, py) = rt.find("chip-row-2").expect("Plan");
        press(&mut rt, px, py);
        assert_eq!(rt.state.chips.mode.as_deref(), Some("plan"));
        assert_eq!(line(&rt).as_deref(), Some("Permissions · plan mode on for this thread"));
        assert!(rt.state.reply.focused, "the caret is back in the words");
        assert!(said(&rt, "Plan"));
        // Shift+Tab goes on round Claude Code's ring, never into Bypass.
        shift_tab(&mut rt);
        assert_eq!(rt.state.chips.mode.as_deref(), Some("auto"));
        assert_eq!(line(&rt).as_deref(), Some("Permissions · auto mode on for this thread"));
        shift_tab(&mut rt);
        assert_eq!(rt.state.chips.mode.as_deref(), Some("default"));
        assert!(rt.state.reply.focused && rt.state.reply.text.is_empty(), "Shift+Tab types nothing and keeps the caret");
        // Escape out of the menu puts the caret back too.
        rt.update(|s, tx| super::toggle(s, Chip::Mode, tx));
        rt.frame(1.0 / 60.0);
        key(&mut rt, Key::Escape);
        assert!(rt.state.chips.menu.is_none() && rt.state.reply.focused && rt.state.replying);
    }

    #[test]
    fn a_thread_on_codex_is_offered_codexs_three_and_shift_tab_walks_them() {
        let mut rt = replying("chips-codex", "codex", "bypassPermissions");
        assert!(said(&rt, "Auto"), "Codex runs Auto with nothing picked");
        rt.update(|s, tx| super::toggle(s, Chip::Mode, tx));
        rt.frame(1.0 / 60.0);
        assert!(said(&rt, "Read only") && said(&rt, "Full access") && !said(&rt, "Manual"));
        key(&mut rt, Key::Escape);
        shift_tab(&mut rt);
        assert_eq!(rt.state.chips.mode.as_deref(), Some("full-access"));
        assert_eq!(line(&rt).as_deref(), Some("Permissions · full access on for this thread"));
    }

    #[test]
    fn a_reply_carries_the_priority_only_once_picked_and_always_what_it_may_do() {
        // Untouched: the thread's own mode, none here, and no priority.
        let mut rt = replying("chips-send-untouched", "claude", "auto");
        let payload = send(&mut rt, "Go on.");
        assert_eq!(payload, json!({ "product": "storefront", "id": "w-feel0", "answer": "Go on.", "status": "open", "permissionMode": null }));
        // Picked: both ride the reply.
        let mut rt = replying("chips-send-picked", "claude", "auto");
        rt.update(|s, tx| super::toggle(s, Chip::Priority, tx));
        rt.update(|s, tx| super::pick(s, Chip::Priority, 1, false, tx));
        rt.update(|s, tx| super::toggle(s, Chip::Mode, tx));
        rt.update(|s, tx| super::pick(s, Chip::Mode, 1, false, tx));
        rt.frame(1.0 / 60.0);
        let payload = send(&mut rt, "Go on.");
        assert_eq!(payload, json!({ "product": "storefront", "id": "w-feel0", "answer": "Go on.", "status": "open", "priority": 7.0, "permissionMode": "acceptEdits" }));
        // The box folded with the send, and opened again it starts over: the thread's own again.
        assert!(!rt.state.replying);
        key(&mut rt, Key::Text("z".into()));
        assert!(rt.state.replying, "Z puts the words back in the box");
        rt.frame(1.0 / 60.0);
        assert_eq!((rt.state.chips.level, rt.state.chips.mode.as_deref()), (None, None), "the picks do not come back with the words");
    }

    #[test]
    fn the_little_arrow_folds_this_rounds_choices_and_the_sign_on_the_box_brings_them_back() {
        let mut rt = window_dressed("chips-choices", &["--thread", "w-feel0"], "/fixtures/storefront", "Fixed. Merge it?\n\n## Options\n\n1. Merge it\n2. Wait for review", |fixture| {
            fixture["snapshot"]["items"][0]["status"] = json!("blocked");
        });
        // The strip's first row, by its key: the answer above says the same words.
        let strip = |rt: &Runtime<State>| rt.find("option-0").is_some();
        assert!(strip(&rt), "the choices over the box");
        let (x, y) = rt.find("choices-fold").expect("the little arrow");
        rt.pointer_move(x, y);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.cursor(), Cursor::Pointer);
        press(&mut rt, x, y);
        assert!(!strip(&rt), "folded away");
        let round = crate::rules::reply_chips::fold_key(rt.state.open_item().unwrap());
        assert_eq!(rt.state.picks.get().choices_folded, vec![round]);
        // The keys still reach them: one picked out with an arrow brings the strip back.
        key(&mut rt, Key::Down);
        assert!(strip(&rt) && rt.state.option == Some(0));
        key(&mut rt, Key::Up);
        assert!(!strip(&rt));
        // The sign at the folded box's end counts them, and its press unfolds them without
        // opening the box.
        let sign = rt.elements().iter().find(|e| e.text.as_deref() == Some("2") && e.rect[1] > 650.0).map(|e| (e.rect[0] + 2.0, e.rect[1] + 2.0));
        let (sx, sy) = sign.expect("the sign counts two");
        press(&mut rt, sx, sy);
        assert!(strip(&rt) && !rt.state.replying);
        assert!(rt.state.picks.get().choices_folded.is_empty());
    }

    #[test]
    fn folding_the_box_lets_the_picks_go_and_a_press_outside_closes_a_menu() {
        let mut rt = replying("chips-fold", "claude", "auto");
        rt.update(|s, tx| super::toggle(s, Chip::Priority, tx));
        rt.update(|s, tx| super::pick(s, Chip::Priority, 0, false, tx));
        rt.frame(1.0 / 60.0);
        assert!(said(&rt, "Urgent"));
        // The menu of the other chip; a press on the words closes it and the caret goes there.
        rt.update(|s, tx| super::toggle(s, Chip::Mode, tx));
        rt.frame(1.0 / 60.0);
        let (x, y) = rt.find("reply").expect("the words");
        press(&mut rt, x, y);
        assert!(rt.state.chips.menu.is_none() && rt.state.reply.focused);
        // A press on one chip with the other's menu open opens its own.
        rt.update(|s, tx| super::toggle(s, Chip::Mode, tx));
        rt.frame(1.0 / 60.0);
        let (px, py) = rt.find(PRIORITY).unwrap();
        press(&mut rt, px, py);
        assert_eq!(rt.state.chips.menu.map(|m| m.chip), Some(Chip::Priority));
        key(&mut rt, Key::Escape);
        key(&mut rt, Key::Escape);
        assert!(!rt.state.replying, "the second Escape folds the empty box");
        key(&mut rt, Key::Text("r".into()));
        assert_eq!(rt.state.chips.level, None);
        assert!(said(&rt, "Medium"), "the thread's own level again");
    }
}

pub fn picture_picked(s: &mut State, picked: &serde_json::Value, tx: &mut Tx) {
    tend(s);
    let Some(item) = s.open_item().cloned() else {
        return;
    };
    if let Some(level) = picked.get("priority").and_then(serde_json::Value::as_str) {
        if let Some(at) = LEVELS.iter().position(|l| level_word(*l).eq_ignore_ascii_case(level)) {
            toggle(s, Chip::Priority, tx);
            pick(s, Chip::Priority, at, false, tx);
        }
    }
    if let Some(mode) = picked.get("mode").and_then(serde_json::Value::as_str) {
        if let Some(at) = rules::rows(claude(s, &item)).iter().position(|(v, _)| *v == mode) {
            toggle(s, Chip::Mode, tx);
            pick(s, Chip::Mode, at, false, tx);
        }
    }
}
