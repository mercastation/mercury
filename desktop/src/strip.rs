//! What a press in the head strip over a conversation opens, and what a pick there writes: the
//! thread's own menu under the dots (`ThreadMenuList` in `renderer/src/threads/ThreadMenu.tsx`)
//! and the panel that says where it runs (`PlaceButton.tsx`), drawn from the web window's own
//! measurements of both (`.th-menu.ts-more-menu` and `.th-pop.th-cloud-pop.ts-place-pop`).
//!
//! The menu offers what this window does of the web's rows, in the web's order: Later (L), which
//! opens the picker, and Priority, whose four levels take the menu's place and write the one
//! picked at once (`retag`). Done and Reopen are buttons of their own beside the dots, as in the
//! web window, whose menu never lists them either (`splitDone`). The panel picks this Mac or the
//! cloud and writes it at once, and stays open on the pick until the engine says something new
//! about the thread (`afterPick`).
//!
//! One of the two is open at a time, on one thread: leaving the thread closes it, as both are
//! the strip's own state in the web window. Escape closes either, before anything else. A press
//! anywhere outside the strip's group of buttons closes it on the way down, as the web window's
//! `pointerdown` does, and a press on one of those buttons goes on to it. In the menu the arrows
//! walk its rows around both ends and Enter or Space presses the one lit; every other key goes on
//! to the page, as the web window's capture listener lets it.

use rune::edit::Key;
use rune::{Drag, El, KeyEvent, Phase, TextCase, TextStyle, Tx, col, el, icon, row, text};
use serde_json::Value;

use crate::compose::level_mark;
use crate::data::{Item, Priority};
use crate::rules::compose::{self as rules, Level};
use crate::rules::place::{self, Act, Place};
use crate::rules::writes;
use crate::theme::*;
use crate::{State, View, chrome, icons, write};

/// The dots and the place button, which what they open hangs from.
pub const MORE_BUTTON: &str = "strip-more";
pub const PLACE_BUTTON: &str = "strip-place";
/// The strip's group of buttons (`.ts-more-wrap`): a press on them is theirs, not an outside one.
pub const GROUP: &str = "strip-actions";
/// The menu's rows, whose width decides where its left edge falls.
const MENU: &str = "strip-menu";
/// What the dots say over a thread whose menu holds nothing this window does.
pub const NOT_HERE: &str = "Open terminal and Rename are in Mercury's own window.";

/// What of the strip is open.
#[derive(Clone, Debug, PartialEq)]
pub enum Over {
    /// The thread's menu: `levels` turned to the four levels, `lit` the row the keys are on.
    More { lit: usize, levels: bool },
    /// The panel that says where it runs.
    Place,
}

/// A place picked in the panel, drawn until the engine says anything new about the thread or it
/// gives up waiting (`picked`, `PICK_SHOWS_MS`): a pick the engine refused then shows as refused.
struct Picked {
    thread: String,
    act: Act,
    /// What the engine said of the thread's place when it was picked (`said`).
    said: String,
    /// When it stops being drawn, on the window's clock, in seconds.
    until: f64,
}

#[derive(Default)]
pub struct Strip {
    /// What is open, and on which thread (`product/id`).
    open: Option<(String, Over)>,
    picked: Option<Picked>,
}

/// One row of the thread's menu.
#[derive(Clone, Debug, PartialEq)]
pub enum Row {
    /// Later (L): the picker opens on the thread.
    Later,
    /// Priority, with the level the thread runs at: the menu turns to the four levels.
    Priority(Level),
    /// A row of the web window's that this window does not do, as a picture of the web window
    /// drew it (`actionRows` in a fixture): drawn as it was, and a press on it does nothing, as
    /// the command bar draws the web's commands it does not have.
    Pictured { label: String, key: String, mark: String },
}

fn thread_key(item: &Item) -> String {
    format!("{}/{}", item.product, item.id)
}

/// The rows this window offers for a thread, in the web window's order (`threadMenuRows`):
/// Later and Priority while it is open (`canFinish`), and Priority only on a thread of work,
/// not an outside agent's or a conversation between people.
pub fn rows_for(s: &State, item: &Item) -> Vec<Row> {
    let level = Priority::of(item.priority);
    if let Some(pictured) = s.fixture.get("actionRows").and_then(Value::as_array) {
        let text = |row: &Value, field: &str| row.get(field).and_then(Value::as_str).unwrap_or("").to_string();
        return pictured
            .iter()
            .map(|r| match text(r, "label").as_str() {
                "Later" => Row::Later,
                "Priority" => Row::Priority(level),
                _ => Row::Pictured { label: text(r, "label"), key: text(r, "key"), mark: text(r, "mark") },
            })
            .collect();
    }
    if item.status == "done" || item.truthy("agent") {
        return Vec::new();
    }
    let direct = s.snap.products.iter().any(|p| p.slug == item.product && rules::is_direct(p));
    let mut rows = vec![Row::Later];
    if !direct {
        rows.push(Row::Priority(level));
    }
    rows
}

/// The facts the panel is drawn from: the engine's, or what a pick leads to while it is drawn.
fn place_facts(s: &State, item: &Item, place: Place) -> place::Facts {
    let facts = place::facts(&s.snap, item, place);
    match &s.strip.picked {
        Some(p) if p.thread == thread_key(item) => place::after_pick(&facts, p.act),
        _ => facts,
    }
}

/// What the place button and its panel show for a thread the cloud could take, or nothing.
pub fn place_view(s: &State, item: &Item) -> Option<place::View> {
    let at = place::thread_place(&s.snap, item)?;
    Some(place::view(&place_facts(s, item, at)))
}

impl State {
    /// What of the strip is open over the conversation on screen.
    pub fn strip_open(&self) -> Option<&Over> {
        let (thread, over) = self.strip.open.as_ref()?;
        let item = self.open_item()?;
        (*thread == thread_key(item)).then_some(over)
    }

    /// Closes the menu or the panel, whichever is open.
    pub fn close_strip(&mut self) {
        self.strip.open = None;
    }

    /// Closes everything that stands open over a conversation: the strip's menu or panel, and
    /// the reply box's model menu. One thing stands over the page at a time.
    pub fn close_thread_menus(&mut self) {
        self.close_strip();
        self.close_reply_menu();
    }

    /// The conversation is left: what its strip had open closes, and a place picked in its panel
    /// is no longer drawn over the engine's, as the web window's strip is made anew for the next
    /// thread (`key` on `ThreadMenu` in `Focus.tsx`).
    pub fn leave_the_strip(&mut self) {
        self.close_thread_menus();
        self.strip.picked = None;
    }

    fn open_over(&mut self, over: Over) {
        let Some(item) = self.open_item() else {
            return;
        };
        let thread = thread_key(item);
        self.close_reply_menu();
        self.close_files();
        self.strip.open = Some((thread, over));
    }

    /// A press on the dots: the menu opens with the keys on its first row, or closes when it is
    /// open (`setOpen((o) => !o)`, and the first row taking the focus). On a thread where none of
    /// the web window's rows is one this window does (a finished one: the terminal, renaming), a
    /// line says where they are rather than open an empty menu.
    pub fn toggle_more(&mut self, tx: &mut Tx) {
        if matches!(self.strip_open(), Some(Over::More { .. })) {
            self.close_strip();
        } else if self.open_item().is_some_and(|item| rows_for(self, item).is_empty()) {
            self.close_thread_menus();
            self.tell(NOT_HERE, tx);
        } else {
            self.open_over(Over::More { lit: 0, levels: false });
        }
    }

    /// A press on the place button: the panel opens, or closes when it is open.
    pub fn toggle_place(&mut self) {
        if matches!(self.strip_open(), Some(Over::Place)) {
            self.close_strip();
        } else {
            self.open_over(Over::Place);
        }
    }

    /// The rows of the menu as it stands: the thread's, or its four levels.
    fn strip_rows(&self) -> Vec<Row> {
        let Some(item) = self.open_item() else {
            return Vec::new();
        };
        match self.strip_open() {
            Some(Over::More { levels: true, .. }) => rules::LEVELS.into_iter().map(Row::Priority).collect(),
            _ => rows_for(self, item),
        }
    }

    /// The pointer moved onto a row of the menu, or the arrows did.
    pub fn light_strip_row(&mut self, at: usize) {
        if let Some((_, Over::More { lit, .. })) = &mut self.strip.open {
            *lit = at;
        }
    }

    /// Enter on a row of the menu, or a press on it (`pick`). Priority turns the menu to the four
    /// levels with the keys on the first; a level closes it and writes the level, unless it is
    /// the one the thread is at; Later closes it and opens the picker.
    pub fn press_strip_row(&mut self, at: usize, tx: &mut Tx) {
        let Some(Over::More { levels, .. }) = self.strip_open().cloned() else {
            return;
        };
        let Some(row) = self.strip_rows().into_iter().nth(at) else {
            return;
        };
        let Some(item) = self.open_item().cloned() else {
            return;
        };
        match (levels, row) {
            (false, Row::Priority(_)) => self.strip.open = Some((thread_key(&item), Over::More { lit: 0, levels: true })),
            (true, Row::Priority(level)) => {
                self.close_strip();
                if level != Priority::of(item.priority) {
                    self.retag(&item, level);
                }
            }
            (_, Row::Later) => {
                self.close_strip();
                self.open_later(&item.id, &item.product, tx);
            }
            (_, Row::Pictured { .. }) => {}
        }
    }

    /// Writes a thread's level at once, as the web window's menu does, and says it once the
    /// engine has it (`retag`: `api.answer` with the level and no words, which starts nothing).
    fn retag(&mut self, item: &Item, level: Level) {
        self.write_what_waits();
        let payload = serde_json::json!({ "product": item.product, "id": item.id, "priority": rules::sent_priority(Some(level)) });
        let said = format!("Priority: {}", rules::level_word(level).to_lowercase());
        self.send(writes::ANSWER, payload, write::Sent::Priority { thread: thread_key(item), said });
    }

    /// A press on one of the panel's two places (`pick`): written at once, and the panel stays
    /// open on the pick, drawn until the engine says something new or four seconds pass.
    pub fn pick_place(&mut self, act: Act, tx: &mut Tx) {
        let Some(item) = self.open_item().cloned() else {
            return;
        };
        let Some(at) = place::thread_place(&self.snap, &item) else {
            return;
        };
        let said = place::said(&place::facts(&self.snap, &item, at));
        self.strip.picked = Some(Picked { thread: thread_key(&item), act, said, until: tx.clock() + place::PICK_SHOWS_MS / 1000.0 });
        self.write_what_waits();
        let writes = place::writes(&self.snap, &item, act);
        self.send_all(writes, write::Sent::Place { thread: thread_key(&item), moving: act == Act::Move });
    }

    /// What the engine said to a pick of a place. A move the cloud takes none of is said in
    /// the web window's words; anything else the panel shows (`moveToCloud` with `said` false).
    pub(crate) fn place_landed(&mut self, moving: bool, answer: &Value, tx: &mut Tx) {
        if moving && place::moving(answer, 1) == 0 {
            self.tell(place::CLOUD_CANNOT_TAKE, tx);
        }
    }

    /// Lets go of a pick once the engine has said something new about its thread, or once its
    /// time is up; and wakes the window when that time comes. True when the strip changed.
    pub(crate) fn tend_strip(&mut self, at: f64, tx: &mut Tx) -> bool {
        let Some(picked) = &self.strip.picked else {
            return false;
        };
        let now_said = self.snap.items.iter().find(|i| thread_key(i) == picked.thread).and_then(|item| place::thread_place(&self.snap, item).map(|p| place::said(&place::facts(&self.snap, item, p))));
        if at >= picked.until || now_said.as_deref() != Some(picked.said.as_str()) {
            self.strip.picked = None;
            return true;
        }
        tx.wake_after(std::time::Duration::from_secs_f64((picked.until - at).max(0.0)));
        false
    }

    /// What a picture of the web window had open in the strip (`actions` in its fixture): the
    /// same button pressed, and the menu turned to the levels for `priority`.
    pub(crate) fn picture_strip(&mut self, which: &str, tx: &mut Tx) {
        match which {
            "place" => self.toggle_place(),
            "menu" | "priority" => {
                self.toggle_more(tx);
                if which == "priority" {
                    let at = self.strip_rows().iter().position(|r| matches!(r, Row::Priority(_)));
                    if let Some(Over::More { levels, lit }) = self.strip.open.as_mut().map(|(_, over)| over) {
                        if at.is_some() {
                            *levels = true;
                            *lit = 0;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/* -------------------------------- drawing -------------------------------- */

/// The mark a row of the menu wears, by the web's own drawing of it (`MARKS`).
fn mark_of(row: &Row) -> El {
    let stroke = |glyph: &'static rune::Icon| icon(glyph, 16.0).color(muted());
    match row {
        Row::Later => stroke(&icons::MENU_LATER),
        // The row's mark is always the middle level, whatever the thread is at.
        Row::Priority(_) => level_mark(Some(Level::Medium)),
        Row::Pictured { mark, .. } => {
            if mark.contains("rect x=\"3\" y=\"4\"") {
                stroke(&icons::MENU_TERMINAL)
            } else if mark.contains("M4 20h4") {
                stroke(&icons::MENU_RENAME)
            } else if mark.contains("M3 12h13") {
                stroke(&icons::MENU_MOVE)
            } else if mark.contains("M8 3h8") {
                stroke(&icons::PIN)
            } else if mark.contains("circle cx=\"12\"") {
                stroke(&icons::MENU_LATER)
            } else {
                el().size(16.0, 16.0)
            }
        }
    }
}

/// One row (`.th-menu .row-i.ts-more-row`): its mark in a square of twenty points, its words,
/// and its key or its detail at the right; `lit` the row the keys or the pointer are on. A level
/// of the four wears a tick when the thread is at it.
fn menu_row(at: usize, lit: bool, lead: El, words: &str, aside: Option<El>) -> El {
    let tone = if lit { ink() } else { dim() };
    let mut line = row()
        .key(format!("strip-row-{at}"))
        .h(34.0)
        .items_center()
        .gap(10.0)
        .px(10.0)
        .radius(6.0)
        .cursor(rune::Cursor::Pointer)
        .on_pointer_move(move |s: &mut State, _| s.light_strip_row(at))
        .on_background_press_tx(move |s: &mut State, tx| s.press_strip_row(at, tx))
        .child(row().size(20.0, 20.0).center().child(lead))
        .child(row().grow(1.0).min_w(0.0).child(pen(label(words, 13.5, 400.0, tone), 13.5)));
    if lit {
        line = line.fill(white(0.07));
    }
    match aside {
        Some(aside) => line.child(aside),
        None => line,
    }
}

/// A row's key or detail (`.ts-more-menu kbd`): small, in the code face, eighteen points off.
fn kbd(words: &str) -> El {
    el().ml(18.0).child(pen(text(words).font(mono(11.0, 14.0)).color(muted()), 11.0))
}

/// The menu's rows as they stand.
fn more_rows(s: &State, item: &Item, levels: bool, lit: usize) -> Vec<El> {
    if levels {
        let at = Priority::of(item.priority);
        return rules::LEVELS
            .into_iter()
            .enumerate()
            .map(|(n, level)| menu_row(n, lit == n, level_mark(Some(level)), rules::level_word(level), (level == at).then(|| icon(&icons::MENU_TICK, 16.0).color(muted()))))
            .collect();
    }
    rows_for(s, item)
        .iter()
        .enumerate()
        .map(|(n, r)| {
            let (words, aside) = match r {
                Row::Later => ("Later".to_string(), Some("L".to_string())),
                Row::Priority(level) => ("Priority".to_string(), Some(rules::level_word(*level).to_string())),
                Row::Pictured { label, key, .. } => (label.clone(), Some(key.clone()).filter(|k| !k.is_empty())),
            };
            menu_row(n, lit == n, mark_of(r), &words, aside.as_deref().map(kbd))
        })
        .collect()
}

/// A popover's shell (`.th-menu`, `.th-pop`): the pop's fill inside the strong hairline, rounded
/// ten points, lifted by its shadow. No rounded clip on it: Rune's takes the element's own
/// shadow with it, and nothing inside reaches its corners.
fn shell(e: El) -> El {
    e.radius(10.0).fill(rgb(54, 52, 49)).border(1.0, rule()).border_layout().box_shadow(shadow(0.0, 18.0, 50.0, 0.0, rgba(0, 0, 0, 0.5), false))
}

/// The menu under the dots (`.th-menu.ts-more-menu`): at least 264 points wide, its right edge on
/// the dots' and its top six points under them.
fn more_menu(s: &State, item: &Item, levels: bool, lit: usize) -> Option<El> {
    let [x, y, w, h] = rune::memo::window_layout(MORE_BUTTON)?;
    let wide = rune::memo::layout_size(MENU).map_or(264.0, |[wide, _]| (wide + 2.0).max(264.0));
    let rows = col().key(MENU).min_w(262.0).items_stretch().gap(1.0).padding(5.0).children(more_rows(s, item, levels, lit));
    Some(shell(col()).absolute(x + w - wide, y + h + 6.0).min_w(264.0).child(rows).role("menu").label(if levels { "Priority" } else { "Thread actions" }))
}

/// One of the panel's two places (`.th-pop .opts > button`): its mark and its word, framed;
/// the one picked filled and brighter (`.on.picked`), and one that cannot be picked faded
/// (`:disabled`).
fn place_choice(cloud: bool, on: bool, act: Option<Act>) -> El {
    let tone = if on { ink() } else { dim() };
    let mut button = row()
        .h(26.0)
        .px(9.0)
        .gap(7.0)
        .items_center()
        .radius(6.0)
        .border(1.0, if on { rule() } else { edge() })
        .border_layout()
        .child(icon(if cloud { &icons::PLACE_CLOUD } else { &icons::PLACE_MAC }, 16.0).color(tone))
        .child(pen(label(if cloud { "Cloud" } else { "This Mac" }, 12.5, 400.0, tone), 12.5));
    if on {
        button = button.fill(rgba(239, 235, 231, 0.1));
    }
    match act {
        Some(act) if !on => button.cursor(rune::Cursor::Pointer).on_background_press_tx(move |s: &mut State, tx| s.pick_place(act, tx)),
        // The place it is on is a button that does nothing when pressed, as in the web window.
        _ if on => button,
        _ => button.opacity(0.4),
    }
}

/// The panel's width, and what is left of it for what a line says: its edge, sixteen points
/// each side, the label's column of 92 and the ten between (`grid-template-columns: 92px 1fr`).
const PANEL: f32 = 440.0;
const SAYS: f32 = PANEL - 2.0 - 32.0 - 92.0 - 10.0;

/// A line of the panel (`.th-pop .line`): its label in a column 92 points wide, then what it
/// says, ten points on, wrapped to the column's width.
fn panel_line(name: El, what: El) -> El {
    row().px(16.0).gap(10.0).child(el().w(92.0).child(name)).child(col().w(SAYS).child(what))
}

/// The small capitals of a label (`.th-pop .lab`).
fn panel_label(words: &str, line: f32) -> El {
    pen(text(words).font(mono(10.5, line).tracking(0.84)).text_case(TextCase::Uppercase).color(muted()), 10.5)
}

/// The faint words under a line (`.th-cloud-note`).
fn panel_note(words: &str) -> El {
    pen(text(words).font(TextStyle::new(12.5, 400.0).lh(18.75)).color(muted()).wrap().w(SAYS), 12.5)
}

/// The panel under the place button (`.th-pop.th-cloud-pop.ts-place-pop`): 440 points wide, its
/// right edge on the button's and its top eight points under it. Where it runs, the two places,
/// why, and what the cloud handed it back for.
fn place_panel(v: &place::View) -> Option<El> {
    let [x, y, w, h] = rune::memo::window_layout(PLACE_BUTTON)?;
    let opts = row().gap(5.0).child(place_choice(true, v.on_cloud, v.pick_cloud)).child(place_choice(false, !v.on_cloud, v.pick_mac));
    let mut panel = shell(col())
        .absolute(x + w - PANEL, y + h + 8.0)
        .w(PANEL)
        .py(8.0)
        .role("dialog")
        .label("Where this thread runs")
        .child(panel_line(panel_label("Runs on", 26.0), opts).py(7.0).items_start())
        .child(panel_line(el(), panel_note(&v.note)).pb(7.0));
    if let Some(needs) = &v.needs {
        panel = panel
            .child(el().h(1.0).my(6.0).fill(edge()))
            .child(panel_line(panel_label("Needs", 14.0), label(needs, 13.0, 400.0, ink())).py(7.0).items_center())
            .child(panel_line(el(), panel_note("The cloud doesn’t have it, so its cloud runs stopped to ask for it and it moved here.")).pb(7.0));
    }
    Some(panel)
}

/// What stands open over the strip, with the layer around the strip's group of buttons that
/// closes it on a press's way down: a press on the group goes on to its buttons.
pub fn layer(s: &State) -> Option<El> {
    let View::Thread { id, product } = &s.view else {
        return None;
    };
    let item = s.snap.items.iter().find(|i| &i.id == id && &i.product == product)?;
    let open = match s.strip_open()? {
        Over::More { levels, lit } => more_menu(s, item, *levels, *lit)?,
        Over::Place => place_panel(&place_view(s, item)?)?,
    };
    let (ww, wh) = s.size;
    let [gx, gy, gw, gh] = rune::memo::window_layout(GROUP).unwrap_or([chrome::CARD[0], chrome::CARD[1], 0.0, 0.0]);
    let (bottom, right) = (gy + gh, gx + gw);
    let away = [[0.0, 0.0, ww, gy], [0.0, bottom, ww, wh - bottom], [0.0, gy, gx, gh], [right, gy, ww - right, gh]].map(|[ax, ay, aw, ah]| {
        el().absolute(ax, ay).size(aw.max(0.0), ah.max(0.0)).on_drag(|s: &mut State, d: &mut Drag, _| {
            if d.phase == Phase::Down {
                s.close_strip();
            }
        })
    });
    Some(el().absolute(0.0, 0.0).size(ww, wh).children(away).child(open))
}

/// The keys while the strip has something open. Escape closes it; in the menu the arrows walk
/// its rows around both ends, Home and End go to the first and the last, Enter and Space press
/// the one lit, and Tab closes it. `None` with nothing open, and `Some(false)` for a key that
/// goes on to the page.
pub fn key(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> Option<bool> {
    let over = s.strip_open()?.clone();
    if e.mods.cmd || e.mods.ctrl || e.mods.alt {
        return Some(false);
    }
    if e.key == Key::Escape {
        s.close_strip();
        return Some(true);
    }
    let Over::More { lit, .. } = over else {
        return Some(false);
    };
    let len = s.strip_rows().len();
    match &e.key {
        Key::Down | Key::Up if len > 0 => s.light_strip_row(rules::entry_step(lit, len, e.key == Key::Down)),
        Key::Home => s.light_strip_row(0),
        Key::End => s.light_strip_row(len.saturating_sub(1)),
        Key::Enter => s.press_strip_row(lit, tx),
        Key::Text(t) if t == " " => s.press_strip_row(lit, tx),
        Key::Tab => {
            s.close_strip();
            return Some(false);
        }
        _ => return Some(false),
    }
    Some(true)
}
