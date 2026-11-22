//! How the window answers the pointer and the hands, where the web window's stylesheet says so:
//! what a control does while the pointer rests on it. Every look here was read off the web
//! window with the pointer resting on the control (`tools/scenes/*-hover-*.mjs`), and every
//! pointer shape off its computed `cursor`.
//!
//! The web window's `:hover` rules have no transition unless one says so, so the window's root
//! lands every hover at once both ways (`hover_motion_within` in `main.rs`): Rune's own hover
//! overlay would otherwise fade out over a tenth of a second.

use std::time::Duration;

use rune::{Color, El, TextStyle, col, el, row, text};

use crate::State;
use crate::theme::*;

/// A control the web window does something with that this window draws and does not do yet:
/// it still answers the pointer as the web window's does, its look and its pointer, by taking
/// the pointer itself, which in Rune an element with nothing to press does not. A press on it
/// does nothing and goes nowhere else, as one on a button goes to the button.
pub fn answers(e: El) -> El {
    e.on_hover(|_: &mut State| {})
}

/// What the pointer is on, where the window has to know.
#[derive(Default)]
pub struct Pointed {
    /// The control under it, for the few whose look under the pointer Rune's own hover cannot
    /// draw: one that gives up a fill and a blur for another, say. Such a control says so with
    /// `pointed`, and its builder reads it back with `is`.
    key: Option<&'static str>,
    /// The Inbox's row the pointer moved onto, as `(product, id)`, which Enter, E and L act on
    /// in place of the picked row while it is there (`pointed` in `renderer/src/App.tsx`).
    /// Moving onto a row is what takes it, never a row moving under a still pointer, and the
    /// keys that walk the list take it back (`takesTheListBack` in `renderer/src/keys.ts`).
    pub row: Option<(String, String)>,
    /// The control whose key hint is drawn beside it, once the pointer has waited on it.
    pub hint: Option<Hint>,
}

impl Pointed {
    pub fn is(&self, key: &str) -> bool {
        self.key == Some(key)
    }
    pub fn is_row(&self, product: &str, id: &str) -> bool {
        self.row.as_ref().is_some_and(|(p, i)| p == product && i == id)
    }
}

/* -------------------------------- key hints -------------------------------- */

/// How long the pointer stays on a control before its plate is drawn, every control its own
/// wait and nothing remembered between two of them (`HINT_WAIT` in `renderer/src/hint-timing.ts`).
const HINT_WAIT: Duration = Duration::from_millis(700);
/// The plate's distance from its control, and from the window's edge (`HINT_GAP`, `HINT_MARGIN`).
const HINT_GAP: f32 = 7.0;
const HINT_MARGIN: f32 = 8.0;

/// Which edge of its control a plate lines up with (`data-hint-align`): the left, or the words
/// inside it when it says which (`data-hint-text`); the right, for one in the window's right
/// corner; or its middle, for a small square button.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Align {
    Left,
    Right,
    Center,
}

/// A control's hint: its name in `HINTS`, the layout key the plate is hung from, how it lines
/// up, and the key of the words inside the control it lines its first key up with.
#[derive(Clone, PartialEq, Debug)]
pub struct Hint {
    pub id: &'static str,
    pub key: String,
    pub align: Align,
    pub words: Option<String>,
}

/// What each hint says, line by line: the key, and what it does (`HINTS` in
/// `renderer/src/hint-plate.ts`), for the controls this window draws.
fn hint_lines(id: &str) -> &'static [(&'static str, &'static str)] {
    match id {
        "new-task" => &[("N", "New thread")],
        "state-tab" => &[("⇥", "Next tab")],
        "row" => &[("↵", "Open task"), ("E", "Mark done"), ("L", "Schedule for later")],
        "cloud" => &[("⌘C", "Cloud on or off")],
        "done" => &[("E", "Mark done")],
        "back" => &[("esc", "Go back one level")],
        "inbox" => &[("esc", "Inbox")],
        "previous-thread" => &[("K", "Previous thread")],
        "next-thread" => &[("J", "Next thread")],
        "thread-files" => &[("", "Files in this thread")],
        "runs-on" => &[("", "Where this thread runs")],
        "more" => &[("", "More actions")],
        _ => &[],
    }
}

/// One cap per key, a word that names one key staying whole (`capsOf` in `shared/key-caps.mts`).
fn caps_of(key: &str) -> Vec<String> {
    if key.is_empty() {
        return Vec::new();
    }
    if ["esc", "Ctrl", "Alt", "Shift", "Enter", "Tab", "Backspace"].contains(&key) {
        return vec![key.to_string()];
    }
    key.chars().map(|c| c.to_string()).collect()
}

/// The control `e`, carrying the hint `id` (`data-hint`): the pointer waiting on it 700 ms
/// draws its plate beside it (`plate`), and the plate goes the moment the pointer leaves. The
/// control is laid out under `key`, which the plate is hung from. The wait is a layer around
/// the control, so a control that already follows the pointer keeps doing so.
pub fn hinted(e: El, id: &'static str, key: impl Into<String>, align: Align, words: Option<String>) -> El {
    let hint = Hint { id, key: key.into(), align, words };
    let control = e.key(hint.key.clone());
    // A column, so a control as wide as a list is as wide inside it as it was.
    col().hover_delays(HINT_WAIT, Duration::ZERO).on_hover_change(move |s: &mut State, inside, tx| {
        if inside {
            s.pointed.hint = Some(hint.clone());
        } else if s.pointed.hint.as_ref().is_some_and(|h| h.key == hint.key) {
            s.pointed.hint = None;
        } else {
            tx.live_only();
        }
    })
    .child(control)
}

/// The plate of the hint the pointer has waited on (`HintPlate`, `.hint-plate` in
/// `renderer/src/styles.css`): under its control, lined up with it, each line the keys in caps
/// and what they do, on the window's glass. It rises above a control too near the window's foot,
/// keeps inside the window, and takes no pointer. None is drawn while something holds the keys
/// whole (the command bar, the New thread card, the Later picker, the reply box: `hintsOn`).
pub fn plate(s: &State) -> Option<El> {
    let hint = s.pointed.hint.as_ref()?;
    if s.palette.open || s.compose.card || s.later.on.is_some() || s.replying {
        return None;
    }
    let lines = hint_lines(hint.id);
    let [x, y, w, h] = rune::memo::window_layout(&hint.key)?;
    if lines.is_empty() || (w <= 0.0 && h <= 0.0) {
        return None;
    }
    let (cap_font, word_font, what_font) = (TextStyle::new(11.5, 600.0).lh(19.0), TextStyle::new(11.5 * 0.86, 600.0).tracking(0.0989), sans(12.0, 400.0));
    let wide = |s: &str, font: TextStyle| rune::text::measure(s, &[], font, None).0;
    // Each cap: its glyph in 6 points of room either side inside its edge, 19 points wide at
    // least; ⇥ with its name beside it, the way the walkthrough's cap reads "⇥ Tab".
    let cap_w = |glyph: &str| {
        let named = if glyph == "⇥" { 5.0 + wide("Tab", word_font) } else { 0.0 };
        (wide(glyph, cap_font) + named + 14.0).max(19.0)
    };
    let caps_w = |key: &str| {
        let caps = caps_of(key);
        caps.iter().map(|c| cap_w(c)).sum::<f32>() + 3.0 * caps.len().saturating_sub(1) as f32
    };
    let caps_col = lines.iter().map(|(key, _)| caps_w(key)).fold(0.0, f32::max);
    let words_col = lines.iter().map(|(_, what)| wide(what, what_font)).fold(0.0, f32::max);
    let line_h = |key: &str| if key.is_empty() { 15.5 } else if key == "⇥" { 21.5 } else { 21.0 };
    // A plate of words alone is five points wider: its line's ten point gap, on a grid whose
    // own gap is nought, puts half of itself after the words' column (CSS subgrid).
    let plate_w = 2.0 + 20.0 + if caps_col > 0.0 { caps_col + 10.0 } else { 5.0 } + words_col;
    let plate_h = 2.0 + 14.0 + lines.iter().map(|(key, _)| line_h(key)).sum::<f32>() + 4.0 * (lines.len() - 1) as f32;
    // Where it goes (`placeHint`): under the control, or above it when there is no room under.
    let (vw, vh) = s.size;
    let rose = y + h + HINT_GAP + plate_h > vh - HINT_MARGIN;
    let top = if rose { y - HINT_GAP - plate_h } else { y + h + HINT_GAP };
    let wanted = match hint.align {
        Align::Right => x + w - plate_w,
        Align::Center => x + w / 2.0 - plate_w / 2.0,
        // Its first key under the words' first letter: their left, less the plate's own
        // padding and edge.
        Align::Left => hint.words.as_deref().and_then(rune::memo::window_layout).map_or(x, |[wx, ..]| wx - 11.0),
    };
    // `Math.round`, which takes a half up.
    let left = (wanted.min(vw - plate_w - HINT_MARGIN).max(HINT_MARGIN) + 0.5).floor();
    let top = (top + 0.5).floor();
    let cap = |glyph: &str| {
        let mut c = row().h(if glyph == "⇥" { 21.5 } else { 21.0 }).min_w(19.0).px(6.0).radius(6.0).border(1.0, rule()).border_layout().justify_center().items_center().child(text(glyph).font(cap_font).color(muted()));
        if glyph == "⇥" {
            c = c.child(text("Tab").font(word_font).color(dim()).ml(5.0));
        }
        c
    };
    let line = |(key, what): &(&str, &str)| {
        let words = pen(text(*what).font(what_font).color(dim()), 12.0);
        if key.is_empty() {
            return row().h(line_h(key)).items_center().child(words);
        }
        row().h(line_h(key)).items_center().gap(10.0).child(row().w(caps_col).gap(3.0).children(caps_of(key).iter().map(|c| cap(c)))).child(words)
    };
    Some(
        col()
            .key("hint-plate")
            .absolute(left, top)
            .size(plate_w, plate_h)
            .py(7.0)
            .px(10.0)
            .gap(4.0)
            .radius(6.0)
            .backdrop_blur(18.0)
            .fill(white(0.07))
            .border(1.0, rule())
            .border_layout()
            .inert()
            .a11y_hidden()
            .children(lines.iter().map(line)),
    )
}

/// Keeps `key` as what the pointer is on while it is on `e`, its children included.
pub fn pointed(e: El, key: &'static str) -> El {
    e.on_hover_change(move |s: &mut State, inside, tx| {
        if inside {
            s.pointed.key = Some(key);
        } else if s.pointed.is(key) {
            s.pointed.key = None;
        } else {
            tx.live_only();
        }
    })
}

/// An Inbox row that takes the keys from the pointer moving onto it, and gives them back when
/// the pointer leaves it (`onMouseMove`, `onMouseLeave` in `renderer/src/components/List.tsx`).
pub fn pointed_row(e: El, product: &str, id: &str) -> El {
    let (p, i) = (product.to_string(), id.to_string());
    let (q, j) = (p.clone(), i.clone());
    e.on_pointer_move_local(move |s: &mut State, _, tx| {
        if s.pointed.is_row(&p, &i) {
            tx.live_only();
        } else {
            s.pointed.row = Some((p.clone(), i.clone()));
        }
    })
    .on_hover_change(move |s: &mut State, inside, tx| {
        if !inside && s.pointed.is_row(&q, &j) {
            s.pointed.row = None;
        } else {
            tx.live_only();
        }
    })
}

/// A control's one point edge drawn as a layer of its own, over the edge it has, point for point.
/// `laid_out` is whether the control's edge takes room in its layout (`border_layout`), which
/// puts the control's inside, where this layer is pinned, a point in from it.
pub fn edge(radius: f32, color: Color, laid_out: bool) -> El {
    let inset = if laid_out { -1.0 } else { 0.0 };
    el().pin(Some(inset), Some(inset), Some(inset), Some(inset)).radius(radius).border(1.0, color)
}

/// The edge a framed control turns while the pointer is on it (`border-color` under `:hover`),
/// laid over the edge it has at rest: the control is a `hover_group`. Opaque over opaque, the
/// lit edge is what shows.
pub fn edge_lit(radius: f32, color: Color, laid_out: bool) -> El {
    edge(radius, color, laid_out).group_opacity(0.0, 1.0)
}

/// The same for a translucent edge, which laid over another would add up to more than either:
/// the control's own edge is left clear, and the edge at rest gives way to the lit one.
pub fn edges(radius: f32, rest: Color, lit: Color, laid_out: bool) -> [El; 2] {
    [edge(radius, rest, laid_out).group_opacity(1.0, 0.0), edge(radius, lit, laid_out).group_opacity(0.0, 1.0)]
}

/// A framed button of a conversation's head strip (`.ts-done`, `.ts-more`: Done, Reopen, the
/// Files button, the place button, ⋯): under the pointer its ground is `--wash-strong`, its edge
/// `--line-strong` and its words and marks the text's own colour, which they take with
/// `group_color(ink())` (`threads/summary.css`).
pub fn strip_button(e: El, laid_out: bool) -> El {
    e.hover_group().hover(wash_strong()).cursor(rune::Cursor::Pointer).child(edge_lit(6.0, rule(), laid_out))
}

/// `--wash`, `--film-strong` and `--wash-strong`: what a control's ground turns under the
/// pointer, by how much it stands out at rest.
pub fn wash() -> Color {
    white(0.04)
}
pub fn film() -> Color {
    white(0.07)
}
pub fn wash_strong() -> Color {
    white(0.08)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use rune::{Cursor, Key, KeyEvent, Mods, Runtime, TitlebarAction};
    use serde_json::{Value, json};

    use crate::{State, View, chrome};

    /// A window on an invented inbox of four threads an agent finished, all four needing the
    /// person, at 1280 by 760, opened with `args` as the command line would open it.
    fn window(name: &str, args: &[&str]) -> Runtime<State> {
        let now = 1_790_000_000_000.0_f64;
        let min = 60_000.0;
        let items: Vec<Value> = (0..4)
            .map(|i| {
                let (asked, answered) = (now - (90.0 + 7.0 * i as f64) * min, now - (4.0 + 6.0 * i as f64) * min);
                json!({
                    "id": format!("w-feel{i}"), "product": "storefront", "productName": "Storefront",
                    "status": "done", "kind": "directive", "title": format!("Invented thread number {i}"),
                    "body": "Look into it.", "result": "Done, and tested. Merge it?", "labels": ["founder"],
                    "priority": 5, "epoch": 2, "claim": null, "createdAt": asked, "updatedAt": answered,
                    "wrote": {
                        "title": { "ts": asked, "source": "founder" }, "body": { "ts": asked, "source": "founder" },
                        "result": { "ts": answered, "source": "agent" }, "status": { "ts": answered, "source": "agent" },
                    },
                })
            })
            .collect();
        let fixture = json!({
            "now": now,
            "snapshot": {
                "products": [{ "slug": "storefront", "name": "Storefront", "dir": "/fixtures/storefront" }],
                "items": items, "approvals": [],
                "supervisor": { "paused": false, "pausedProducts": [], "running": [], "stalled": [], "queued": [], "scheduled": [], "productOrder": ["storefront"], "hiddenProducts": [], "personalProducts": [], "capacity": 0 },
                "restartNeeded": null, "config": {},
            },
            "local": {},
        });
        let path = std::env::temp_dir().join(format!("mercury-feel-{name}-{}.json", std::process::id()));
        std::fs::write(&path, fixture.to_string()).expect("the invented inbox");
        let mut all = vec!["--fixture".to_string(), path.to_string_lossy().into_owned()];
        all.extend(args.iter().map(|a| a.to_string()));
        let s = crate::init_from(&all);
        let _ = std::fs::remove_file(&path);
        let text = rune::text::TextSystem::new(crate::FONTS, ("Geist", "Geist Mono, ui-monospace"));
        let mut rt = Runtime::with_text(s, crate::view, (1280.0, 760.0), rune::Env::new(true, 2.0), text);
        rt.poll = Some(crate::poll);
        rt.on_key = Some(crate::keys::pressed);
        rt.frame(0.0);
        rt.frame(1.0 / 60.0);
        rt
    }

    /// `seconds` of frames at 60 a second.
    fn run(rt: &mut Runtime<State>, seconds: f64) {
        for _ in 0..(seconds * 60.0).round() as usize {
            rt.frame(1.0 / 60.0);
        }
    }

    fn press(rt: &mut Runtime<State>, x: f32, y: f32) -> Option<TitlebarAction> {
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        let action = rt.take_titlebar_action();
        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
        action
    }

    fn key(rt: &mut Runtime<State>, key: Key) {
        rt.key(KeyEvent { key, mods: Mods::default() });
        rt.frame(1.0 / 60.0);
    }

    #[test]
    fn the_rail_opens_once_the_pointer_has_rested_on_it_and_folds_when_it_leaves() {
        let mut rt = window("rail", &[]);
        assert_eq!(rt.state.lists.needs.len(), 4, "the invented inbox needs the person four times");
        // Onto Demands, which opens nothing of its own: the pointing hand, and the rail still
        // folded until 140 ms have gone by.
        rt.pointer_move(41.0, 244.0);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.cursor(), Cursor::Pointer);
        run(&mut rt, 0.1);
        assert_eq!(rt.state.rail.width(rt.state.at), 38.0);
        // Then it eases open over 0.24 s, the card moving over with it.
        run(&mut rt, 0.4);
        assert_eq!(rt.state.rail.width(rt.state.at), 202.0);
        assert_eq!(chrome::card_left(&rt.state), 246.0);
        // Off it and onto the card: it folds at once, over 0.2 s.
        rt.pointer_move(700.0, 400.0);
        run(&mut rt, 0.3);
        assert_eq!(rt.state.rail.width(rt.state.at), 38.0);
        assert_eq!(chrome::card_left(&rt.state), 82.0);
    }

    #[test]
    fn a_key_hint_comes_700_ms_after_the_pointer_and_goes_with_it_or_with_a_key() {
        let mut rt = window("hint", &["--view", "inbox"]);
        // New thread, in the Inbox's strip.
        rt.pointer_move(1126.0, 71.0);
        rt.frame(1.0 / 60.0);
        run(&mut rt, 0.6);
        assert!(rt.state.pointed.hint.is_none(), "not before 700 ms");
        run(&mut rt, 0.2);
        assert_eq!(rt.state.pointed.hint.as_ref().map(|h| h.id), Some("new-task"));
        // Hung 7 points under New thread, its left edge on New thread's (`placeHint`): the web
        // window's plate is 118.4 by 37 at 1065, 95, and `find` gives its middle.
        let at = rt.find("hint-plate").expect("its plate is drawn");
        assert!((at.0 - (1065.0 + 118.4 / 2.0)).abs() < 1.0 && (at.1 - (95.0 + 37.0 / 2.0)).abs() < 1.0, "the plate stands at {at:?}");
        rt.pointer_move(640.0, 600.0);
        rt.frame(1.0 / 60.0);
        assert!(rt.state.pointed.hint.is_none(), "it goes the moment the pointer leaves");
        // Back on a row, and then a key: the plate goes.
        rt.pointer_move(400.0, 221.0);
        run(&mut rt, 0.8);
        assert_eq!(rt.state.pointed.hint.as_ref().map(|h| h.id), Some("row"));
        key(&mut rt, Key::Down);
        assert!(rt.state.pointed.hint.is_none(), "a key takes it away");
    }

    #[test]
    fn the_row_the_pointer_moves_onto_takes_enter_until_the_keys_walk_the_list() {
        // The rows stand 54 points apart from 194; the third is under the pointer.
        let mut rt = window("row", &["--view", "inbox"]);
        let third = rt.state.lists.needs[2].id.clone();
        rt.pointer_move(600.0, 194.0 + 2.0 * 54.0 + 27.0);
        rt.frame(1.0 / 60.0);
        key(&mut rt, Key::Enter);
        assert!(matches!(&rt.state.view, View::Thread { id, .. } if *id == third), "Enter opens the row under the pointer");

        let mut rt = window("row-keys", &["--view", "inbox"]);
        let second = rt.state.lists.needs[1].id.clone();
        rt.pointer_move(600.0, 194.0 + 2.0 * 54.0 + 27.0);
        rt.frame(1.0 / 60.0);
        key(&mut rt, Key::Text("j".into()));
        key(&mut rt, Key::Enter);
        assert!(matches!(&rt.state.view, View::Thread { id, .. } if *id == second), "J takes the list back: Enter opens the row the keys walked to");
    }

    #[test]
    fn command_bracket_goes_back_where_backs_title_says() {
        let mut rt = window("back", &["--view", "inbox"]);
        assert_eq!(super::super::chrome::back_title(&rt.state), "Back to Home (⌘[)");
        rt.key(KeyEvent { key: Key::Command('['), mods: Mods { cmd: true, ..Mods::default() } });
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.view, View::Home);
        let mut rt = window("back-thread", &["--thread"]);
        assert_eq!(super::super::chrome::back_title(&rt.state), "Back to Needs you (⌘[)");
        rt.key(KeyEvent { key: Key::Command('['), mods: Mods { cmd: true, ..Mods::default() } });
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.view, View::Inbox, "a conversation opened from nowhere goes back to the Inbox");
    }

    #[test]
    fn the_bands_the_web_window_drags_move_the_window_and_their_buttons_keep_their_presses() {
        let mut rt = window("drag", &["--view", "inbox"]);
        assert_eq!(press(&mut rt, 640.0, 20.0), Some(TitlebarAction::Drag), "the band over the card");
        assert_eq!(press(&mut rt, 640.0, 60.0), Some(TitlebarAction::Drag), "the Inbox's strip between its title and its buttons");
        assert_eq!(press(&mut rt, 167.0, 71.5), None, "Forward, with nowhere to go, still keeps its press");
        assert_eq!(press(&mut rt, 41.0, 300.0), None, "the rail is never part of it");
        assert_eq!(press(&mut rt, 1126.0, 71.0), None, "New thread keeps its press");
        assert!(rt.state.compose.card, "and opens the card");
        // A double press zooms the window as the system says.
        rt.state.compose.card = false;
        rt.frame(1.0 / 60.0);
        assert_eq!([press(&mut rt, 640.0, 20.0), press(&mut rt, 640.0, 20.0)], [Some(TitlebarAction::Drag), Some(TitlebarAction::DoubleClick)]);

        let mut rt = window("drag-thread", &["--thread"]);
        assert_eq!(press(&mut rt, 92.0, 68.0), Some(TitlebarAction::Drag), "the conversation's strip, left of its arrows");
        assert_eq!(press(&mut rt, 900.0, 68.0), None, "the middle, where the crumb and the name stand");
        assert_eq!(press(&mut rt, 900.0, 46.0), Some(TitlebarAction::Drag), "the strip over the middle");
        assert!(matches!(rt.state.view, View::Thread { .. }));
        assert_eq!(press(&mut rt, 115.0, 67.5), None, "the Inbox button keeps its press");
        assert_eq!(rt.state.view, View::Inbox, "and goes to the Inbox");
    }

    /// The web window's word for a pointer (`cursor`), as Rune names it.
    fn shape(css: &str) -> Option<Cursor> {
        Some(match css {
            "pointer" => Cursor::Pointer,
            "text" => Cursor::Text,
            "default" => Cursor::Default,
            "zoom-in" => Cursor::ZoomIn,
            "zoom-out" => Cursor::ZoomOut,
            "not-allowed" => Cursor::NotAllowed,
            "grab" => Cursor::Grab,
            "grabbing" => Cursor::Grabbing,
            "move" => Cursor::Move,
            "crosshair" => Cursor::Crosshair,
            "col-resize" => Cursor::ResizeColumn,
            "row-resize" => Cursor::ResizeRow,
            _ => return None,
        })
    }

    /// The command line a scene's picture is drawn with, by its name, as the notes draw them.
    fn view_of(scene: &str) -> Vec<String> {
        if let Ok(own) = std::env::var("MERCURY_FEEL_ARGS") {
            return own.split_whitespace().map(str::to_string).collect();
        }
        let name = scene.rsplit('/').next().unwrap_or(scene);
        let words: &[&str] = match name {
            n if n.starts_with("thread") => &["--thread"],
            n if n.starts_with("inbox-later") => &["--view", "inbox", "--tab", "later"],
            n if n.starts_with("inbox-progress") => &["--view", "inbox", "--tab", "progress"],
            n if n.starts_with("inbox-done") => &["--view", "inbox", "--tab", "done"],
            n if n.starts_with("inbox") || n.starts_with("compose-card") || n.starts_with("palette") => &["--view", "inbox"],
            _ => &[],
        };
        words.iter().map(|w| w.to_string()).collect()
    }

    /// Walks the pointer over a scene the reference tool photographed (`tools/reference.mjs`, whose
    /// dump says which shape the web window's pointer takes at the middle of each thing on
    /// screen) and holds each point's shape here against it. A tool, not a test of the build: it
    /// reads the pictures' folder, which is never committed.
    ///
    ///   MERCURY_FEEL=shots/thread cargo test pointer -- --ignored --nocapture
    #[test]
    #[ignore]
    fn the_pointer_takes_the_web_windows_shape_everywhere() {
        let Ok(scene) = std::env::var("MERCURY_FEEL") else { return };
        let dump: Value = serde_json::from_str(&std::fs::read_to_string(format!("{scene}-dump.json")).expect("the scene's dump")).expect("a dump");
        let (w, h) = (dump["viewport"][0].as_f64().unwrap_or(1280.0) as f32, dump["viewport"][1].as_f64().unwrap_or(760.0) as f32);
        let mut args = vec!["--fixture".to_string(), format!("{scene}-fixture.json")];
        args.extend(view_of(&scene));
        let mut s = crate::init_from(&args);
        s.size = (w, h);
        // Every picture read where it is asked for, as a still picture reads them, so the page
        // stands as it does in the scene's picture.
        s.pictures = crate::pictures::Pictures::new(2.0, true, false);
        let text = rune::text::TextSystem::new(crate::FONTS, ("Geist", "Geist Mono, ui-monospace"));
        let mut rt = rune::Runtime::with_text(s, crate::view, (w, h), rune::Env::new(true, 2.0), text);
        rt.frame(0.0);
        rt.update(|s, tx| crate::press_what_the_picture_pressed(s, tx));
        rt.frame(1.0 / 60.0);
        let mut wrong = Vec::new();
        let mut kinds: BTreeMap<String, (usize, usize, Vec<String>)> = BTreeMap::new();
        for p in dump["pointers"].as_array().expect("the web window's pointers") {
            let (x, y) = (p["x"].as_f64().unwrap_or(0.0) as f32, p["y"].as_f64().unwrap_or(0.0) as f32);
            let css = p["cursor"].as_str().unwrap_or("default");
            let what = format!("{}.{} {:?}", p["tag"].as_str().unwrap_or(""), p["cls"].as_str().unwrap_or(""), p["label"].as_str().unwrap_or(""));
            let Some(want) = shape(css) else {
                println!("{x},{y}: the web window's {css} has no shape here ({what})");
                continue;
            };
            // Each point is a pointer that just arrived, so nothing waiting on a resting pointer
            // (the rail opening, a hint) comes due between two of them.
            rt.pointer_move(x, y);
            rt.frame(1.0 / 60.0);
            let got = rt.cursor();
            rt.pointer_leave();
            rt.frame(1.0 / 60.0);
            if p["resting"].as_bool() == Some(true) {
                println!("where the scene's pointer rests, {x},{y}: the web window's {css} over {what}, here {got:?}");
            }
            let kind = kinds.entry(css.to_string()).or_default();
            kind.1 += 1;
            if got == want {
                kind.0 += 1;
                if !kind.2.contains(&what) {
                    kind.2.push(what);
                }
            } else {
                wrong.push(format!("{x},{y}: the web window's {css} over {what}, here {got:?}"));
            }
        }
        for (css, (same, all, over)) in &kinds {
            println!("{css}: {same} of {all}");
            if css != "default" {
                for what in over {
                    println!("    {what}");
                }
            }
        }
        for line in &wrong {
            println!("DIFFERS {line}");
        }
        assert!(wrong.is_empty(), "{} points take another shape here", wrong.len());
    }
}
