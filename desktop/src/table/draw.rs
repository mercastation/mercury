//! How a sheet looks and where a press lands on it (`TableView`'s markup and
//! `components/table-view.css`, the menus' `.when-menu`): the bar, the grid with its header and
//! row numbers standing still while the rest scrolls, a cell's box, a tag's list, a date's
//! calendar, a cell's words shown whole, the row opened whole, and the menus.

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;

use rune::edit::Key;
use rune::{Cursor, Drag, El, GrowthSpec, KeyEvent, Phase, Pixels, TextCase, TextEdit, TextStyle, Tx, col, el, icon, row, text, text_input};

use super::*;
use crate::theme::*;

/* ------------------------------ colours and marks ------------------------------ */

/// `--tbl-line`, the lines between cells.
fn line() -> rune::Color {
    white(0.09)
}
/// `--surface`: the ground the header, the row numbers and the first column stand on.
pub(crate) fn ground() -> rune::Color {
    rgb(40, 38, 37)
}
/// `--pop`, what a menu floats on.
fn pop() -> rune::Color {
    rgb(54, 52, 49)
}
/// `--bg-selected`, `--bg-hover`, `--bg-raised`.
fn selected() -> rune::Color {
    rgb(55, 53, 52)
}
fn hovered() -> rune::Color {
    rgb(46, 44, 43)
}
fn raised() -> rune::Color {
    rgb(47, 45, 44)
}
/// `--tbl-at`, the ring on the cell the keys act on, and the tint on the block over the ground.
fn at_ring() -> rune::Color {
    rgba(110, 150, 225, 0.7)
}
fn tinted() -> rune::Color {
    // `rgba(110, 150, 225, 0.1)` over `--surface`, as the browser lays one over the other.
    rgb(47, 49, 56)
}
/// `--shadow-lg`.
fn lifted() -> rune::Shadow {
    shadow(0.0, 24.0, 70.0, 0.0, rgba(0, 0, 0, 0.55), false)
}

/// A tag's ground, ink and dot, in the dark theme's colours.
pub(crate) fn tone_colours(tone: Tone) -> (rune::Color, rune::Color, rune::Color) {
    match tone {
        Tone::Green => (rgba(58, 165, 100, 0.16), rgb(0x93, 0xd8, 0xac), rgb(0x3a, 0xa5, 0x64)),
        Tone::Blue => (rgba(76, 127, 214, 0.18), rgb(0xab, 0xc5, 0xf4), rgb(0x4c, 0x7f, 0xd6)),
        Tone::Purple => (rgba(125, 99, 212, 0.2), rgb(0xc8, 0xb8, 0xf3), rgb(0x7d, 0x63, 0xd4)),
        Tone::Amber => (rgba(217, 149, 43, 0.17), rgb(0xed, 0xc5, 0x83), rgb(0xd9, 0x95, 0x2b)),
        Tone::Rose => (rgba(212, 98, 127, 0.18), rgb(0xf1, 0xa8, 0xbb), rgb(0xd4, 0x62, 0x7f)),
        Tone::Teal => (rgba(51, 160, 138, 0.17), rgb(0x8b, 0xd6, 0xc5), rgb(0x33, 0xa0, 0x8a)),
        Tone::Grey => (white(0.08), rgb(0xc9, 0xcc, 0xd2), rgb(0x8d, 0x92, 0x9b)),
    }
}

/// A tag (`TagChip`): its dot and its words in its colour, as the column first wrote it.
pub(crate) fn chip(column: &file::Column, value: &str) -> El {
    let key = js::trim(value).to_lowercase();
    let shown = column.values.iter().find(|x| x.to_lowercase() == key).cloned().unwrap_or_else(|| js::trim(value).to_string());
    chip_in(column.tone(value), &shown)
}

fn chip_in(tone: Tone, words: &str) -> El {
    let (bg, ink_c, dot) = tone_colours(tone);
    row().pl(8.0).pr(9.0).py(2.0).gap(6.0).items_center().fill(bg).child(el().size(6.0, 6.0).fill(dot)).child(text(words).font(TextStyle::new(13.0, 400.0).lh(18.0)).color(ink_c))
}

macro_rules! glyph {
    ($name:ident, $id:literal, $width:literal, $body:literal) => {
        pub static $name: rune::Icon = rune::Icon {
            name: $id,
            view: 16.0,
            body: concat!(r#"<g fill="none" stroke="currentColor" stroke-width=""#, $width, r#"" stroke-linecap="round" stroke-linejoin="round">"#, $body, "</g>"),
        };
    };
}

// The web's own `glyph(...)` paths (`OPEN`, `CARET`, the calendar's and the row's arrows, close).
glyph!(OPEN, "m-tbl-open", "1.4", r#"<path d="M9.5 3H13v3.5M13 3L8.5 7.5M6.5 13H3V9.5M3 13l4.5-4.5"/>"#);
glyph!(CARET, "m-tbl-caret", "1.4", r#"<path d="M5 6.5l3 3 3-3"/>"#);
glyph!(BACK, "m-tbl-back", "1.4", r#"<path d="M10 4L6 8l4 4"/>"#);
glyph!(BACK_BOLD, "m-tbl-back-bold", "1.6", r#"<path d="M10 4L6 8l4 4"/>"#);
glyph!(NEXT, "m-tbl-next", "1.4", r#"<path d="M6 4l4 4-4 4"/>"#);
glyph!(ABOVE, "m-tbl-above", "1.4", r#"<path d="M4 10l4-4 4 4"/>"#);
glyph!(BELOW, "m-tbl-below", "1.4", r#"<path d="M4 6l4 4 4-4"/>"#);
glyph!(CLOSE, "m-tbl-close", "1.4", r#"<path d="M4 4l8 8M12 4l-8 8"/>"#);
pub static MORE: rune::Icon = rune::Icon { name: "m-tbl-more", view: 16.0, body: r#"<g fill="currentColor"><circle cx="3.5" cy="8" r="1.1"/><circle cx="8" cy="8" r="1.1"/><circle cx="12.5" cy="8" r="1.1"/></g>"# };

/// What a column holds, drawn small before its name, as Notion marks a property (`KindIcon`).
pub(crate) fn kind_mark(kind: Kind) -> &'static rune::Icon {
    match kind {
        Kind::Name | Kind::Text => &crate::icons::TABLE_WORDS,
        Kind::Tag => &crate::icons::TABLE_TAG,
        Kind::Number => &crate::icons::TABLE_NUMBER,
        Kind::Date => &crate::icons::TABLE_DATE,
        Kind::Link => &crate::icons::TABLE_LINK,
        Kind::Email => &crate::icons::TABLE_EMAIL,
    }
}

/// The types a column can be shown as (`TYPES`).
pub(crate) const TYPES: [(Kind, &str); 6] = [(Kind::Text, "Text"), (Kind::Tag, "Tags"), (Kind::Number, "Number"), (Kind::Date, "Date"), (Kind::Link, "Link"), (Kind::Email, "Email")];

fn type_name(kind: Kind) -> &'static str {
    TYPES.iter().find(|(k, _)| *k == kind).map_or("Text", |(_, n)| n)
}

/// How a sort reads for a column (`sortWords`).
pub(crate) fn sort_words(kind: Kind, up: bool) -> &'static str {
    match (kind, up) {
        (Kind::Number, true) => "1 to 9",
        (Kind::Number, false) => "9 to 1",
        (Kind::Date, true) => "Oldest first",
        (Kind::Date, false) => "Newest first",
        (_, true) => "A to Z",
        (_, false) => "Z to A",
    }
}

/// A count as the web's `toLocaleString` writes it: 1,234.
pub(crate) fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (k, ch) in digits.chars().enumerate() {
        if k > 0 && (digits.len() - k) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// "1 row", "1,234 rows" (`rowsWord`).
pub(crate) fn rows_word(n: usize) -> String {
    format!("{} {}", thousands(n), if n == 1 { "row" } else { "rows" })
}

fn measure(s: &str, style: TextStyle) -> f32 {
    rune::text::measure(s, &[], style, None).0
}

fn head_font() -> TextStyle {
    mono(10.5, 10.5).tracking(0.945)
}

fn cell_font() -> TextStyle {
    TextStyle::new(14.0, 400.0).lh(18.0)
}

/* ------------------------------ widths ------------------------------ */

/// Each column's width from the whole file (`columnWidths`): its longest words up to `cap`,
/// or its widest tag, or its header, with a logo's room and the cell's edges; and the column of
/// longest words, which takes what is spare. `only` measures one column.
pub(crate) fn column_widths(table: &Table, cap: f32, only: Option<usize>) -> (Vec<f32>, Option<usize>) {
    let rows = table.rows.len();
    let step = (rows / 3000).max(1);
    let longest: Vec<usize> = (0..table.columns.len())
        .map(|i| {
            if only.is_some_and(|o| o != i) {
                return 0;
            }
            (0..rows).step_by(step).map(|r| js::len(table.cell(r, i))).max().unwrap_or(0)
        })
        .collect();
    let px = table
        .columns
        .iter()
        .enumerate()
        .map(|(i, c)| {
            if only.is_some_and(|o| o != i) {
                return 0.0;
            }
            let mut w: f32 = 0.0;
            if c.kind == Kind::Tag {
                for v in &c.values {
                    w = w.max(measure(v, TextStyle::new(13.0, 400.0)) + 32.0);
                }
            } else {
                for r in (0..rows).step_by(step) {
                    let v = table.cell(r, i);
                    // Only words near the longest can be the widest; measuring each would be slow.
                    if js::len(v) as f32 >= longest[i] as f32 * 0.7 {
                        let shown = if c.kind == Kind::Link { file::cell_link(table, &table.rows[r], i).map_or_else(|| v.to_string(), |l| l.text) } else { file::shown_text(v) };
                        w = w.max(cap.min(measure(&shown, TextStyle::new(14.0, 400.0))));
                    }
                }
            }
            let head = js::len(&c.name) as f32 * 7.6 + 20.0 + 24.0;
            let logo = if c.kind == Kind::Name && table.logo_col.is_some() { 28.0 } else { 0.0 };
            (head.max(w + logo) + 34.0).ceil()
        })
        .collect();
    let texts: Vec<usize> = table.columns.iter().enumerate().filter(|(_, c)| c.kind == Kind::Text).map(|(i, _)| i).collect();
    let wide = texts.iter().copied().reduce(|best, i| if longest[i] > longest[best] { i } else { best });
    (px, wide)
}

/// What a header's own words ask for (its max-content): its edges, its mark, its name in spaced
/// capitals, the arrow of a sort and the caret of its menu.
fn head_content(c: &file::Column, sorted: bool) -> f32 {
    let name = measure(&c.name.to_uppercase(), head_font());
    let arrow = if sorted { 6.0 + measure("↑", head_font()) } else { 0.0 };
    32.0 + 13.0 + 7.0 + name + arrow + 6.0 + 18.0 + 1.0
}

/// The drawn columns' widths (`tracks`, laid out as the browser lays out `minmax(…, …fr)` in a
/// grid as wide as its words and at least the view): a width a hand set; the longest column of
/// text, or else the last, taking what is spare and at least its base; the rest as measured.
/// A ranked file's rank is the row number and takes none.
pub(crate) fn widths(s: &State, sh: &Sheet) -> Vec<f32> {
    let room = scrolled(sh.at).viewport[0].max(1.0);
    let room = if room <= 1.0 { s.tables_room(sh.at) } else { room };
    let table = sh.table();
    let n = table.columns.len();
    let shown: Vec<usize> = (0..n).filter(|i| !(table.ranked && *i == 0)).collect();
    let natural = |c: usize| sh.natural.0.get(&table.columns[c].name).copied().unwrap_or(160.0);
    let wide = sh.natural.1.as_ref().and_then(|name| shown.iter().copied().find(|c| table.columns[*c].name == *name)).filter(|c| !sh.sized.contains_key(&table.columns[*c].name));
    let mut out = vec![0.0; n];
    let mut fixed = 0.0;
    for &c in &shown {
        let w = sh.sized.get(&table.columns[c].name).copied().unwrap_or_else(|| natural(c));
        out[c] = w;
        fixed += w;
    }
    let flex = wide.or_else(|| shown.last().copied().filter(|c| !sh.sized.contains_key(&table.columns[*c].name)));
    if let Some(f) = flex {
        let w = natural(f);
        let sorted = sh.sort.is_some_and(|so| so.col == f);
        let base = if Some(f) == wide { w.min(180.0) } else { w };
        let content = base.max(head_content(&table.columns[f], sorted));
        fixed -= out[f];
        let avail = room - 1.0 - sh.rn_w();
        out[f] = if fixed + content < avail { avail - fixed } else { content };
    }
    out
}

/// Where a column starts in the grid and how wide it is, the grid's own left line and the row
/// numbers before it.
pub(crate) fn column_x(s: &State, sh: &Sheet, c: usize) -> (f32, f32) {
    let ws = widths(s, sh);
    let x = 1.0 + sh.rn_w() + ws.iter().take(c).sum::<f32>();
    (x, ws.get(c).copied().unwrap_or(0.0))
}

impl State {
    /// The width a sheet's viewport has before it has been drawn once.
    fn tables_room(&self, w: Where) -> f32 {
        ROOM.with(|r| r.borrow().get(&w).copied()).unwrap_or(600.0)
    }
}

thread_local! {
    /// The width each sheet's viewport is laid out at, from its last view.
    static ROOM: RefCell<HashMap<Where, f32>> = RefCell::new(HashMap::new());
    /// Where each sheet's viewport stands in the window, as it last laid out.
    static VIEWPORT: RefCell<HashMap<Where, [f32; 4]>> = RefCell::new(HashMap::new());
    /// The favicons read, by host: a picture, or none for a site the service does not know.
    static LOGOS: RefCell<HashMap<String, Option<Pixels>>> = RefCell::new(HashMap::new());
    static LOGOS_LANDED: std::sync::Arc<std::sync::Mutex<Vec<(String, Option<(u32, u32, Vec<u8>)>)>>> = Default::default();
}

/* ------------------------------ where a press lands ------------------------------ */

/// What is under a point of the grid, in the grid's own points: the header and the row numbers
/// and the first column where they stand while the rest scrolls under them (`atOf`).
pub(crate) fn hit_at(s: &State, sh: &Sheet, x: f32, y: f32) -> Option<Hit> {
    let at = scrolled(sh.at);
    let (sx, sy) = (at.offset[0], at.offset[1]);
    let (vx, vy) = (x - sx, y - sy);
    let ws = widths(s, sh);
    let rn = sh.rn_w();
    let first = sh.first();
    let col_at = || -> Option<usize> {
        // The first column stands right of the row numbers once the rest scrolls under it.
        let first_left = (1.0 + rn - sx).max(rn);
        if vx >= first_left && vx < first_left + ws[first] {
            return Some(first);
        }
        let mut left = 1.0 + rn;
        for (c, w) in ws.iter().enumerate() {
            if *w > 0.0 && c != first && x >= left && x < left + w {
                return Some(c);
            }
            left += w;
        }
        None
    };
    let rn_left = (1.0 - sx).max(0.0);
    if vy < HEAD_H {
        if vx >= rn_left && vx < rn_left + rn {
            return None;
        }
        return col_at().map(Hit::Col);
    }
    let r = ((y - HEAD_H) / sh.row_h()).floor();
    if r < 0.0 || r as usize >= sh.order().len() {
        return None;
    }
    let r = r as usize;
    if vx >= rn_left && vx < rn_left + rn {
        return Some(Hit::Row(r));
    }
    col_at().map(|c| Hit::Cell(At { r, c }))
}

/// Where rows or columns picked up would land under the pointer (`dropAt`): the gap between two
/// and where to draw it.
fn drop_at(s: &State, sh: &Sheet, x: f32, y: f32, cols: bool) -> Option<Dropping> {
    let rows = sh.order().len();
    let row_h = sh.row_h();
    let ws = widths(s, sh);
    let (first, last) = (sh.first(), sh.last());
    if !cols {
        let at = scrolled(sh.at);
        if y - at.offset[1] < HEAD_H {
            return None;
        }
        let r = ((y - HEAD_H) / row_h).floor().max(0.0) as usize;
        if r >= rows {
            return Some(Dropping { cols, place: rows, at: HEAD_H + rows as f32 * row_h });
        }
        let mid = HEAD_H + r as f32 * row_h + row_h / 2.0;
        let place = if y > mid { r + 1 } else { r };
        return Some(Dropping { cols, place, at: HEAD_H + place as f32 * row_h });
    }
    let hit = hit_at(s, sh, x, y.max(scrolled(sh.at).offset[1]));
    let c = match hit {
        Some(Hit::Col(c) | Hit::Cell(At { c, .. })) => c,
        _ => {
            let end = 1.0 + sh.rn_w() + ws.iter().sum::<f32>();
            return (x > end).then_some(Dropping { cols, place: last + 1, at: end });
        }
    };
    let (left, w) = column_x(s, sh, c);
    let after = x > left + w / 2.0;
    let place = if after { c + 1 } else { c }.max(first);
    let (lx, lw) = column_x(s, sh, place.min(last));
    Some(Dropping { cols, place, at: lx + if place > last { lw } else { 0.0 } })
}

/// The pointer's gestures on the grid, read as the web's one listener reads them.
fn grid_drag(s: &mut State, w: Where, d: &mut Drag, tx: &mut Tx) {
    gesture(s, w, d.phase, d.x, d.y, d.clicks, d.cancelled(), tx);
}

/// A gesture begun on a link's words in a cell, which take the pointing hand and so the
/// pointer: handed on to the grid at the same point of it (`d.window_point()` less where the
/// grid stands in the window, as it last laid out, and how far it is scrolled).
fn link_drag(s: &mut State, w: Where, d: &mut Drag, tx: &mut Tx) {
    let [wx, wy] = d.window_point();
    let [vx, vy, _, _] = VIEWPORT.with(|v| v.borrow().get(&w).copied()).unwrap_or_default();
    let at = scrolled(w);
    gesture(s, w, d.phase, wx - vx + at.offset[0], wy - vy + at.offset[1], d.clicks, d.cancelled(), tx);
}

#[allow(clippy::too_many_arguments)]
fn gesture(s: &mut State, w: Where, phase: Phase, x: f32, y: f32, clicks: u32, cancelled: bool, tx: &mut Tx) {
    let Some(sh) = s.tables.get(w) else { return };
    let d = Point { x, y, clicks, cancelled };
    let hit = hit_at(s, sh, d.x, d.y);
    match phase {
        Phase::Down => {
            let link = match hit {
                Some(Hit::Cell(at)) => sh.row_at(at.r).and_then(|row| file::cell_link(sh.table(), row, at.c)).map(|l| l.href),
                _ => None,
            };
            match hit {
                Some(hit) => pressed_at(s, w, hit, link, tx),
                None => {
                    // The corner over the row numbers picks every cell.
                    let at = scrolled(w);
                    if d.y - at.offset[1] < HEAD_H && !sh.order().is_empty() {
                        let (first, last, n) = (sh.first(), sh.last(), sh.order().len());
                        if let Some(sh) = s.tables.get_mut(w) {
                            sh.pick = Some(Pick { a: At { r: 0, c: first }, b: At { r: n - 1, c: last }, rows: false });
                        }
                        focus_keys(s, w, tx);
                    }
                }
            }
        }
        Phase::Move => {
            let moving = matches!(sh.dragging, Some(super::Dragging::MoveRows | super::Dragging::MoveCols));
            let cols = matches!(sh.dragging, Some(super::Dragging::MoveCols));
            let drop = if moving { drop_at(s, sh, d.x, d.y, cols) } else { None };
            // Near the foot or under the header the table scrolls under the pointer.
            let at = scrolled(w);
            let vy = d.y - at.offset[1];
            let vx = d.x - at.offset[0];
            let mut to = (at.offset[0], at.offset[1]);
            if vy > at.viewport[1] - 24.0 {
                to.1 = (to.1 + 18.0).min(at.max[1]);
            } else if vy < HEAD_H + 8.0 {
                to.1 = (to.1 - 18.0).max(0.0);
            }
            if cols {
                if vx > at.viewport[0] - 24.0 {
                    to.0 = (to.0 + 18.0).min(at.max[0]);
                } else if vx < 24.0 {
                    to.0 = (to.0 - 18.0).max(0.0);
                }
            }
            if sh.dragging.is_some() && (to.0 != at.offset[0] || to.1 != at.offset[1]) {
                tx.scroll_to_instant(w.key("scroll"), to.0, to.1);
            }
            dragged_to(s, w, hit, drop);
        }
        Phase::Up => {
            released(s, w, d.cancelled, tx);
            if d.clicks == 2
                && !d.cancelled
                && let Some(hit) = hit
            {
                double_at(s, w, hit, tx);
            }
        }
    }
}

/// Where a gesture is on the grid, in the grid's own points.
struct Point {
    x: f32,
    y: f32,
    clicks: u32,
    cancelled: bool,
}

/* ------------------------------ the view ------------------------------ */

/// The sheet open at `w` in a box `width` by `height`: beside a conversation (`.tbl-pane`) or
/// big (`.tbl-full`).
pub fn view(s: &State, w: Where, (width, height): (f32, f32)) -> El {
    let Some(sh) = s.tables.get(w) else { return el().size(width, height) };
    let (pad_x, pad_top) = match w {
        Where::Pane => (28.0, 4.0),
        Where::Big => (48.0, 8.0),
    };
    let inner = (width - 2.0 * pad_x).max(1.0);
    if sh.table().columns.is_empty() {
        return col().size(width, height).pt(pad_top).px(pad_x).child(text("This table has no rows yet.").font(TextStyle::new(13.0, 400.0).lh(16.0)).color(muted()).ml(16.0).mt(24.0));
    }
    ROOM.with(|r| r.borrow_mut().insert(w, inner));
    let bar = bar(s, sh, inner);
    let frame_h = (height - pad_top - 68.0).max(1.0);
    let frame = el().w(inner).h(frame_h).child(grid(s, sh, inner, frame_h)).children(sheet_panel(s, sh, inner, frame_h));
    col().size(width, height).pt(pad_top).px(pad_x).child(bar).child(frame)
}

/// A control of the bar (`.tbl-btn`): its mark and its words, lit while on or open.
fn bar_button(glyph: &'static rune::Icon, words: &str, on: bool, key: String) -> El {
    let (fill, tone) = if on { (Some(hovered()), ink()) } else { (None, dim()) };
    let b = row().key(key).h(34.0).px(12.0).gap(6.0).items_center().hover_group().hover(hovered()).cursor(Cursor::Pointer).child(icon(glyph, 14.0).color(tone).group_color(ink())).child(label(words, 13.0, 400.0, tone).group_color(ink()));
    match fill {
        Some(f) => b.fill(f),
        None => b,
    }
}

/// The bar over the table (`.tbl-bar`): search, Filter, Sort, and how many rows, or what went
/// wrong with the last save.
fn bar(_s: &State, sh: &Sheet, inner: f32) -> El {
    let w = sh.at;
    let total = sh.table().rows.len();
    let search = text_input(&sh.search, sans(13.0, 400.0))
        .key(w.key("search"))
        .single_line()
        .grow(1.0)
        .color(ink())
        .placeholder(format!("Search {}", rows_word(total)), muted())
        .caret(ink(), rgba(110, 165, 255, 0.3))
        .on_edit(move |s: &mut State| super::edit_of(s, w, Box_::Search))
        .on_input_change(move |s: &mut State, _| super::searched(s, w))
        .on_input_key(move |s: &mut State, e: &KeyEvent, tx: &mut Tx| {
            // Escape with nothing typed goes on to what the table is in.
            if e.key == Key::Escape && s.tables.get(w).is_some_and(|sh| sh.search.text.is_empty()) {
                if let Some(sh) = s.tables.get_mut(w) {
                    sh.search.focused = false;
                }
                tx.focus(w.key("scroll"), false);
                return crate::keys::pressed(s, e, tx);
            }
            false
        });
    let find = row()
        .w(280.0_f32.min(inner).max(140.0))
        .h(34.0)
        .px(10.0)
        .gap(8.0)
        .items_center()
        .border(1.0, rule())
        .border_layout()
        // A label round its box: a press anywhere puts the caret in it, the I-beam only over it.
        .cursor(Cursor::Default)
        .child(icon(&crate::icons::TABLE_SEARCH, 14.0).color(muted()))
        .child(search)
        .on_press(move |s: &mut State| {
            if let Some(sh) = s.tables.get_mut(w) {
                sh.search.focused = true;
            }
        });
    let mut bar = row().w(inner).pt(18.0).pb(16.0).gap(6.0).items_center().child(find);
    let choices = edit::filter_choices(sh.table(), 40);
    if !choices.is_empty() {
        let named: Vec<&str> = sh.picked.iter().filter(|(_, v)| !v.is_empty()).filter_map(|(c, _)| sh.table().columns.get(*c).map(|c| c.name.as_str())).collect();
        let words = match named.len() {
            0 => "Filter".to_string(),
            1 | 2 => format!("Filtered by {}", named.join(" and ")),
            n => format!("Filtered by {} and {} and {} more", named[0], named[1], n - 2),
        };
        let open = sh.bar == Some(Bar::Filter);
        bar = bar.child(bar_button(&crate::icons::TABLE_FILTER, &words, !named.is_empty() || open, w.key("filter")).on_press_tx(move |s: &mut State, tx| toggle_bar(s, w, Bar::Filter, tx)));
    }
    let sorted = sh.sort.and_then(|so| sh.table().columns.get(so.col).map(|c| (c, so.up)));
    let words = match sorted {
        Some((c, up)) => {
            let said = sort_words(c.kind, up).to_lowercase();
            let said = match said.as_str() {
                "a to z" => "A to Z".to_string(),
                "z to a" => "Z to A".to_string(),
                _ => said,
            };
            format!("Sorted by {}, {said}", c.name)
        }
        None => "Sort".to_string(),
    };
    let open = sh.bar == Some(Bar::Sort);
    bar = bar.child(bar_button(&crate::icons::TABLE_SORT, &words, sorted.is_some() || open, w.key("sort")).on_press_tx(move |s: &mut State, tx| toggle_bar(s, w, Bar::Sort, tx)));
    bar = bar.child(el().grow(1.0));
    match &sh.problem {
        Some(p) => bar.child(problem_line(p, w)),
        None => bar.child(text(count_words(sh)).font(mono(11.0, 14.0).tracking(0.66)).text_case(TextCase::Uppercase).color(muted())),
    }
}

/// What the count says (`.tbl-count`).
pub(crate) fn count_words(sh: &Sheet) -> String {
    let total = sh.table().rows.len();
    let shown = sh.order().len();
    let count = if shown == total { rows_word(total) } else { format!("{} of {}", thousands(shown), rows_word(total)) };
    let picked = match sh.sel() {
        Some(b) if edit::cells_in(&b) > 1 => {
            if sh.whole_rows() { format!("{} picked · ", rows_word(b.r1 - b.r0 + 1)) } else { format!("{} cells picked · ", thousands(edit::cells_in(&b))) }
        }
        _ => String::new(),
    };
    let cols = sh.table().columns.len();
    let columns = if sh.at == Where::Big { format!(" · {cols} {}", if cols == 1 { "column" } else { "columns" }) } else { String::new() };
    format!("{picked}{count}{columns}")
}

/// A save that did not happen (`.tbl-problem`): a rose dot, the reason and what can be done.
fn problem_line(p: &Problem, w: Where) -> El {
    let button = |words: &str| row().h(28.0).px(9.0).items_center().hover(hovered()).cursor(Cursor::Pointer).child(label(words, 12.5, 400.0, ink()));
    let mut line = row().gap(4.0).items_center().min_w(0.0).shrink().child(el().size(6.0, 6.0).mr(4.0).fill(tone_colours(Tone::Rose).2)).child(text(p.error.as_str()).font(TextStyle::new(12.5, 400.0).lh(16.0)).color(dim()).truncate());
    if p.stale {
        line = line.child(button("Save mine").on_press(move |s: &mut State| super::flush(s, w, true))).child(button("Load the new one").on_press(move |s: &mut State| super::reload(s, w)));
    } else {
        line = line.child(button("Try again").on_press(move |s: &mut State| super::flush(s, w, false)));
    }
    line.role("alert")
}

fn toggle_bar(s: &mut State, w: Where, which: Bar, tx: &mut Tx) {
    let Some(sh) = s.tables.get_mut(w) else { return };
    sh.bar = if sh.bar == Some(which) { None } else { Some(which) };
    sh.bar_lit = None;
    sh.filter_find = TextEdit::new("");
    sh.float = None;
    // The box to find a value takes the keys as it opens (`autoFocus`).
    if sh.bar == Some(Bar::Filter) && filter_values(sh) > 12 {
        sh.filter_find.focused = true;
    } else if sh.bar.is_none() {
        super::focus_keys(s, w, tx);
    }
}

fn filter_values(sh: &Sheet) -> usize {
    edit::filter_choices(sh.table(), 40).iter().map(|(_, v)| v.len()).sum()
}

/* ------------------------------ the grid ------------------------------ */

/// The grid in its scrolling viewport (`.tbl-scroll`, `.tbl-grid`): the header standing at the
/// top, the row numbers and the first column at the left, only the rows in view built, and over
/// them what a cell has open.
fn grid(s: &State, sh: &Sheet, inner: f32, frame_h: f32) -> El {
    let w = sh.at;
    let ws = widths(s, sh);
    let rn = sh.rn_w();
    let row_h = sh.row_h();
    let grid_w = (1.0 + rn + ws.iter().sum::<f32>()).max(inner);
    let rows = sh.order().len();
    let first = sh.first();
    let stuck_at = rn + ws.get(first).copied().unwrap_or(160.0);
    let mut content = col().w(grid_w).border_sides([0.0, 0.0, 0.0, 1.0], line()).border_layout().child(header(s, sh, &ws, stuck_at));
    let version = (std::rc::Rc::as_ptr(&sh.doc.order) as usize, std::rc::Rc::as_ptr(&sh.doc.table.rows) as usize, rows);
    let list = col()
        .key(w.key("rows"))
        .growth(GrowthSpec { spring: rune::Spring::NONE, ..Default::default() })
        .virtual_rows(rows, row_h, |_: &State, i| Cow::Owned(i.to_string()), move |s: &State, i| s.tables.get(w).map_or_else(el, |sh| data_row(s, sh, i)))
        .virtual_rows_version(version);
    content = content.child(list);
    if sh.editable {
        content = content.child(new_row(sh, grid_w));
    }
    if rows == 0 {
        let words = if sh.table().rows.is_empty() { "This table has no rows yet." } else { "No rows match." };
        content = content.child(row().px(16.0).py(24.0).sticky_left(0.0).child(text(words).font(TextStyle::new(13.0, 400.0).lh(16.0)).color(muted())));
    }
    content = content.children(drop_line(sh, grid_w)).children(editor(s, sh)).children(peek_box(s, sh)).on_drag(move |s: &mut State, d: &mut Drag, tx| grid_drag(s, w, d, tx)).on_secondary_press_local(move |s: &mut State, p: rune::PointerEvent, tx| {
        let Some(sh) = s.tables.get(w) else { return };
        if let Some(hit) = hit_at(s, sh, p.x, p.y) {
            let at = p.window();
            context_at(s, w, hit, at.x, at.y, tx);
        }
    });
    let moving = matches!(sh.dragging, Some(super::Dragging::MoveRows | super::Dragging::MoveCols)) && sh.dropping.is_some();
    let viewport = el()
        .key(w.key("scroll"))
        .size(inner, frame_h)
        .scroll_xy()
        .on_scroll_offset(move |at| super::heard(w, at))
        .on_layout(move |_: &mut State, rect| VIEWPORT.with(|v| _ = v.borrow_mut().insert(w, rect)))
        .focusable(true)
        .on_key(move |s: &mut State, e: &KeyEvent, tx: &mut Tx| super::grid_key(s, w, e, tx))
        .child(content);
    // The keys are kept on the viewport itself, which a press anywhere in it gives them to.
    if moving { viewport.cursor(Cursor::Grabbing) } else { viewport }
}

/// The header row (`.tbl-th`): standing at the top, the corner and the first column at the left.
fn header(_s: &State, sh: &Sheet, ws: &[f32], stuck_at: f32) -> El {
    let w = sh.at;
    let rn = sh.rn_w();
    let first = sh.first();
    let table = sh.table();
    let all = sh.sel().is_some() && sh.whole_rows() && sh.whole_cols();
    let corner_tone = if all { tinted() } else { ground() };
    let corner = if table.ranked {
        // A ranked file's `#`, which sorts by its rank.
        let sorted = sh.sort.is_some_and(|so| so.col == 0);
        row().w(rn).h(HEAD_H).pr(12.0).justify_end().items_center().fill(ground()).border_sides([1.0, 1.0, 1.0, 0.0], line()).border_layout().sticky_left(0.0).hover_group().cursor(Cursor::Default).help(table.columns[0].name.clone()).child(text("#").font(head_font()).color(if sorted { ink() } else { muted() }).group_color(ink())).on_press(move |s: &mut State| super::sort_by(s, w, 0))
    } else {
        row().w(rn).h(HEAD_H).fill(corner_tone).border_sides([1.0, 1.0, 1.0, 0.0], line()).border_layout().sticky_left(0.0)
    };
    let mut head = row().sticky_top(0.0).child(corner);
    let whole_cols = sh.whole_cols();
    let sel = sh.sel();
    let editable = sh.editable;
    for (i, c) in table.columns.iter().enumerate() {
        if table.ranked && i == 0 {
            continue;
        }
        let sorted = sh.sort.filter(|so| so.col == i);
        let on = whole_cols && sel.is_some_and(|b| i >= b.c0 && i <= b.c1);
        let tone = if sorted.is_some() || on { ink() } else { muted() };
        let mut cell = row().w(ws[i]).h(HEAD_H).px(16.0).gap(6.0).items_center().fill(if on { tinted() } else { ground() }).border_sides([1.0, 1.0, 1.0, 0.0], line()).border_layout().hover_group();
        if c.kind == Kind::Number {
            cell = cell.justify_end();
        }
        if i == first {
            cell = cell.sticky_left(sh.rn_w());
        }
        if sh.renaming == Some(i) {
            let field = text_input(&sh.rename, head_font())
                .key(w.key("rename"))
                .single_line()
                .grow(1.0)
                .color(ink())
                .caret(ink(), rgba(110, 165, 255, 0.3))
                .on_edit(move |s: &mut State| super::edit_of(s, w, Box_::Rename))
                .on_input_key(move |s: &mut State, e: &KeyEvent, tx: &mut Tx| match e.key {
                    Key::Enter => {
                        super::finish_rename(s, w, true, tx);
                        true
                    }
                    Key::Escape => {
                        super::finish_rename(s, w, false, tx);
                        true
                    }
                    _ => false,
                });
            cell = cell.child(row().grow(1.0).min_w(0.0).h(25.0).mx(-6.0).px(6.0).items_center().fill(ground()).border(1.0, rule()).border_layout().child(field));
        } else {
            let mut words = row().min_w(0.0).shrink().items_center().help(type_name(c.kind)).child(icon(kind_mark(c.kind), 13.0).color(muted()).mr(7.0)).child(text(c.name.as_str()).font(head_font()).text_case(TextCase::Uppercase).color(tone).group_color(ink()).truncate());
            // A wide column scrolled half under the frozen ones keeps its name beside them.
            if i != first {
                words = words.sticky_left(stuck_at + 16.0);
            }
            cell = cell.child(words);
            if let Some(so) = sorted {
                cell = cell.child(text(if so.up { "↑" } else { "↓" }).font(head_font()).color(ink()));
            }
        }
        // Its menu's caret, by its name, out of sight until the header is under the pointer.
        let caret = row()
            .key(format!("{}-{i}", w.key("caret")))
            .size(20.0, 20.0)
            .ml(-2.0)
            .center()
            .group_opacity(0.0, 1.0)
            .hover(hovered())
            .hover_group()
            .cursor(Cursor::Pointer)
            .label(format!("{} column", c.name))
            .child(icon(&CARET, 14.0).color(muted()).group_color(ink()))
            .on_drag(move |s: &mut State, d: &mut Drag, tx: &mut Tx| {
                if d.phase != Phase::Up || d.cancelled() || d.x < 0.0 || d.y < 0.0 || d.x > d.w || d.y > d.h {
                    return;
                }
                super::end_by_press(s, w, tx);
                let [px, py] = d.window_point();
                let (left, bottom) = (px - d.x, py - d.y + d.h);
                if let Some(sh) = s.tables.get_mut(w) {
                    sh.float = Some(Float { x: left, y: bottom + 4.0, col: Some(i), page_type: false, lit: None });
                    sh.bar = None;
                }
            });
        cell = cell.child(caret);
        // Its right edge widens or narrows it; a double press fits it to what it holds.
        let from = ws[i];
        let resize = el()
            .absolute(ws[i] - 4.0, 0.0)
            .size(8.0, HEAD_H)
            .hover_group()
            .cursor(Cursor::ResizeColumn)
            .child(el().absolute(3.0, 0.0).size(2.0, HEAD_H).fill(rule()).group_opacity(0.0, 1.0))
            .on_drag(move |s: &mut State, d: &mut Drag, _| {
                super::grip(s, w, i, d.phase, d.dx, from);
                if d.phase == Phase::Up && d.clicks == 2 && d.dx.abs() < 1.0 {
                    super::fit_column(s, w, i);
                }
            });
        let cell = cell.child(resize);
        let _ = editable;
        head = head.child(cell);
    }
    head
}

/// One row of the grid at a place in the view (`Row`).
fn data_row(s: &State, sh: &Sheet, r: usize) -> El {
    let w = sh.at;
    let Some(cells) = sh.row_at(r) else { return el() };
    let table = sh.table();
    let row_h = sh.row_h();
    let ws = widths(s, sh);
    let first = sh.first();
    let sel = sh.sel();
    let on = sel.is_some_and(|b| r >= b.r0 && r <= b.r1);
    let whole = on && sh.whole_rows();
    let at = sh.pick.filter(|p| p.a.r == r).map(|p| p.a.c);
    let number = if table.ranked { cells.at(0).to_string() } else { (r + 1).to_string() };
    let movable = sh.editable && sh.sort.is_none() && whole;
    let rn = row()
        .w(sh.rn_w())
        .h(row_h)
        .pr(12.0)
        .justify_end()
        .items_center()
        .fill(if whole { tinted() } else { ground() })
        .border_sides([0.0, 1.0, 1.0, 0.0], line())
        .border_layout()
        .sticky_left(0.0)
        .cursor(if movable { Cursor::Grab } else { Cursor::Default })
        .child(text(number).font(figures(12.5, 400.0)).color(if whole { ink() } else { white(0.36) }));
    let mut line_el = row().h(row_h).hover_group().child(rn);
    for (i, c) in table.columns.iter().enumerate() {
        if table.ranked && i == 0 {
            continue;
        }
        let picked = on && sel.is_some_and(|b| i >= b.c0 && i <= b.c1);
        let mut cell = row().w(ws[i]).h(row_h).px(16.0).items_center().border_sides([0.0, 1.0, 1.0, 0.0], line()).border_layout();
        if picked {
            cell = cell.fill(tinted());
        } else if i == first {
            cell = cell.fill(ground());
        }
        if i == first {
            cell = cell.sticky_left(sh.rn_w());
        }
        if c.kind == Kind::Number {
            cell = cell.justify_end();
        }
        if at == Some(i) {
            cell = cell.box_shadow(if sh.holding { shadow(0.0, 0.0, 0.0, 1.0, rule(), true) } else { shadow(0.0, 0.0, 0.0, 2.0, at_ring(), true) });
        }
        let link = file::cell_link(table, cells, i);
        if let Some(title) = hover_text(c.kind, cells.at(i), link.as_ref()) {
            cell = cell.help(title);
        }
        cell = cell.child(cell_body(s, sh, cells, i, ws[i]));
        // The way to open a row sits at the end of its first cell while the row is under the pointer.
        if i == first {
            cell = cell.child(
                row()
                    .absolute(ws[i] - 6.0 - 24.0, (row_h - 24.0) / 2.0)
                    .size(24.0, 24.0)
                    .center()
                    .fill(ground())
                    .border(1.0, line())
                    .border_layout()
                    .group_opacity(0.0, 1.0)
                    .hover_group()
                    .cursor(Cursor::Pointer)
                    .help("Open this row (Space)")
                    .label("Open this row")
                    .child(icon(&OPEN, 13.0).color(muted()).group_color(ink()))
                    .on_press_tx(move |s: &mut State, tx| {
                        let Some(sh) = s.tables.get(w) else { return };
                        let p = super::pick_for(sh, Hit::Row(r), false);
                        if let Some(sh) = s.tables.get_mut(w) {
                            sh.pick = Some(p);
                        }
                        super::open_row(s, w, r, tx);
                    }),
            );
        }
        line_el = line_el.child(cell);
    }
    line_el
}

/// What a cell's hover says (`hoverText`): a link's address, a name, or text over 40 characters.
fn hover_text(kind: Kind, v: &str, link: Option<&file::CellLink>) -> Option<String> {
    if let Some(l) = link {
        let bare = l.href.strip_prefix("mailto:").unwrap_or(&l.href).to_string();
        return Some(if matches!(kind, Kind::Link | Kind::Email) || l.text == file::shown_link(&l.href) { bare } else { format!("{}\n{}", l.text, file::shown_link(&l.href)) });
    }
    (kind == Kind::Name || (kind == Kind::Text && js::len(v) > 40)).then(|| js::trim(v).to_string()).filter(|t| !t.is_empty())
}

/// A cell's words as they are drawn (`CellBody`): a name with its logo, a tag, a link, a number
/// to the right, a date dimmer, three lines of a wrapped column.
fn cell_body(s: &State, sh: &Sheet, cells: &file::Cells, c: usize, wide: f32) -> El {
    let table = sh.table();
    let column = &table.columns[c];
    let v = js::trim(cells.at(c));
    if v.is_empty() {
        return el();
    }
    let link = file::cell_link(table, cells, c);
    let wrapped = super::wraps(column);
    let words_of = |shown: &str, color: rune::Color, link: bool| {
        let mut t = text(shown.replace('\n', " ")).font(if wrapped { TextStyle::new(14.0, 400.0).lh(20.0) } else { cell_font() }).color(color);
        t = if wrapped { t.wrap().max_lines(3) } else { t.truncate() };
        let words = row().min_w(0.0).shrink().child(t);
        if link {
            // Underlined under the pointer, three points under the words, which take the pointing
            // hand as a link's do (a press still picks the cell: ⌘ opens it).
            let w = sh.at;
            words.hover_group().cursor(Cursor::Pointer).child(el().absolute(0.0, 17.0).h(1.0).w_full().fill(if column.kind == Kind::Name { dim() } else { ink() }).group_opacity(0.0, 1.0)).on_drag(move |s: &mut State, d: &mut Drag, tx: &mut Tx| link_drag(s, w, d, tx))
        } else {
            words
        }
    };
    match column.kind {
        Kind::Name => {
            let mut out = row().min_w(0.0).shrink().gap(10.0).items_center();
            if table.logo_col.is_some() {
                out = out.child(logo(s, file::logo_for(table, cells), link.as_ref().map_or(v, |l| l.text.as_str())));
            }
            let _ = s;
            out.child(words_of(link.as_ref().map_or(v, |l| l.text.as_str()), ink(), link.is_some()))
        }
        Kind::Tag if link.is_none() => chip(column, v),
        Kind::Number => row().child(text(v).font(figures(14.0, 400.0).lh(18.0)).color(ink())),
        Kind::Date => row().min_w(0.0).shrink().child(text(v).font(figures(14.0, 400.0).lh(18.0)).color(dim()).truncate()),
        _ => match &link {
            Some(l) => words_of(&l.text, dim(), true),
            None => words_of(&file::shown_text(v), ink(), false),
        },
    }
    .max_w((wide - 32.0).max(0.0))
}

/// A row's logo (`Logo`): its site's favicon, or the first letter of its name on a small tile
/// where the site is not known to the service or not read yet.
fn logo(s: &State, host: Option<String>, name: &str) -> El {
    let letter = || {
        let first = js::trim(name).chars().next().map_or("?".to_string(), |c| c.to_uppercase().collect());
        row().size(18.0, 18.0).center().fill(raised()).child(text(first).font(sans(10.5, 600.0)).color(dim()))
    };
    let Some(host) = host else { return letter() };
    let got = LOGOS.with(|l| l.borrow().get(&host).cloned());
    match got {
        Some(Some(pixels)) => el().size(18.0, 18.0).pixels(&pixels),
        Some(None) => letter(),
        None => {
            ask_logo(s, &host);
            letter()
        }
    }
}

/// A favicon asked of Google's service rather than of the site, so the company never sees a
/// request from someone reading a list they are on; a site it does not know comes back as its
/// 16-point globe, which is no logo. Only a window on the app asks: a picture has no network.
fn ask_logo(s: &State, host: &str) {
    if !matches!(s.source, crate::source::Source::Door(_)) || cfg!(test) {
        LOGOS.with(|l| l.borrow_mut().insert(host.to_string(), None));
        return;
    }
    LOGOS.with(|l| l.borrow_mut().insert(host.to_string(), None));
    let landed = LOGOS_LANDED.with(|l| l.clone());
    let host = host.to_string();
    std::thread::spawn(move || {
        // A host is letters, figures, dots and hyphens (`hostOf`); anything else asks nothing.
        if !host.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') {
            landed.lock().unwrap().push((host, None));
            return;
        }
        let url = format!("https://www.google.com/s2/favicons?domain={host}&sz=64");
        let read = std::process::Command::new("/usr/bin/curl").args(["-s", "-f", "-m", "8", "--max-filesize", "400000", &url]).output().ok().filter(|o| o.status.success()).map(|o| o.stdout);
        let picture = read.and_then(|bytes| image::load_from_memory(&bytes).ok()).map(|i| i.to_rgba8()).filter(|i| i.width() > 16).map(|i| (i.width(), i.height(), i.into_raw()));
        landed.lock().unwrap().push((host, picture));
    });
}

/// The favicons that came back since the last look; true when one did.
pub(crate) fn logos_landed() -> bool {
    let landed: Vec<(String, Option<(u32, u32, Vec<u8>)>)> = LOGOS_LANDED.with(|l| l.lock().unwrap().drain(..).collect());
    let any = !landed.is_empty();
    for (host, picture) in landed {
        let pixels = picture.map(|(w, h, rgba)| {
            let p = Pixels::new();
            p.set(w, h, rgba);
            p
        });
        LOGOS.with(|l| l.borrow_mut().insert(host, pixels));
    }
    any
}

/// `+ New row` at the foot (`.tbl-add`).
fn new_row(sh: &Sheet, grid_w: f32) -> El {
    let w = sh.at;
    row()
        .w(grid_w)
        .h(42.0)
        .items_center()
        .border_sides([0.0, 1.0, 1.0, 0.0], line())
        .border_layout()
        .hover_group()
        .hover(hovered())
        .cursor(Cursor::Pointer)
        .help("New row (Shift+Enter)")
        .child(row().sticky_left(0.0).items_center().child(row().w(44.0).pr(12.0).justify_end().child(label("+", 15.0, 400.0, muted()).group_color(ink()))).child(label("New row", 13.0, 400.0, muted()).group_color(ink())))
        .on_press_tx(move |s: &mut State, tx| {
            let n = s.tables.get(w).map_or(0, |sh| sh.order().len());
            super::add_row_at(s, w, n, true, tx);
        })
}

/// Where rows or columns picked up would land (`.tbl-drop`).
fn drop_line(sh: &Sheet, grid_w: f32) -> Option<El> {
    let d = sh.dropping?;
    let grid_h = HEAD_H + sh.order().len() as f32 * sh.row_h();
    Some(if d.cols { el().absolute(d.at - 1.0 - EDGE, 0.0).size(2.0, grid_h).fill(at_ring()) } else { el().absolute(-EDGE, d.at - 1.0).size(grid_w, 2.0).fill(at_ring()) })
}

/* ------------------------------ a cell's box, its words whole ------------------------------ */

/// Where a cell's box or its words shown whole sit (`place`, `peek`): over the cell, never over
/// the row numbers and the first column, as wide as the cell or wider for long words, inside the
/// view; near the view's foot rising from the cell's bottom.
struct Placed {
    left: f32,
    top: f32,
    w: f32,
    up: bool,
}

fn placed(s: &State, sh: &Sheet, at: At, want: f32, tall: f32) -> Placed {
    let view = scrolled(sh.at);
    let (sx, sy, vw, vh) = (view.offset[0], view.offset[1], view.viewport[0].max(1.0), view.viewport[1].max(1.0));
    let (x, cw) = column_x(s, sh, at.c);
    let from = sx + super::stuck_width(s, sh, at.c);
    let cell_left = if at.c == sh.first() { from } else { x };
    let w = cw.min(200.0).max(want.min(sx + vw - from - 8.0));
    let left = from.max(cell_left.min(sx + vw - w));
    let top = HEAD_H + at.r as f32 * sh.row_h();
    let below = sy + vh - (top + sh.row_h());
    let up = below < tall && top - sy > below;
    Placed { left, top, w, up }
}

/// The box a cell is typed into (`.tbl-edit`), or its list or calendar (`.tbl-pop`).
fn editor(s: &State, sh: &Sheet) -> Option<El> {
    let w = sh.at;
    let e = sh.editing.as_ref()?;
    let at = At { r: e.r, c: e.c };
    let row_h = sh.row_h();
    let column = sh.table().columns.get(e.c)?;
    if let Some(opened) = &e.pop {
        let tall = if matches!(opened, Pop::Date(_)) { 340.0 } else { 300.0 };
        let p = placed(s, sh, at, 260.0, tall);
        let body = match opened {
            Pop::Tag(t) => tag_list(sh, column, sh.value(at), t, PickFor::Cell, e.c, p.w - 10.0),
            Pop::Date(d) => calendar(s, sh, sh.value(at), d, PickFor::Cell, e.c),
        };
        let shell = col().w(p.w).padding(4.0).fill(pop()).border(1.0, rule()).border_layout().box_shadow(lifted()).child(body);
        let top = if p.up { None } else { Some(p.top + row_h - 1.0) };
        return Some(match top {
            Some(top) => shell.absolute(p.left - EDGE, top),
            // Rising: its foot one point under the cell's top line.
            None => shell.absolute(p.left - EDGE, (p.top + 1.0 - pop_height(s, sh, opened)).max(0.0)),
        });
    }
    let roomy = js::len(&e.draft.text) > 32 || e.draft.text.contains('\n');
    let p = placed(s, sh, at, if roomy { 380.0 } else { 0.0 }, 300.0);
    let ring = if sh.editable { shadow(0.0, 0.0, 0.0, 2.0, at_ring(), true) } else { shadow(0.0, 0.0, 0.0, 1.0, rule(), true) };
    let body: El = if sh.editable {
        text_input(&e.draft, TextStyle::new(14.0, 400.0).lh(20.0))
            .key(w.key("edit"))
            .w_full()
            .color(ink())
            .caret(ink(), rgba(110, 165, 255, 0.3))
            .on_edit(move |s: &mut State| super::edit_of(s, w, Box_::Edit))
            .on_input_key(move |s: &mut State, e: &KeyEvent, tx: &mut Tx| edit_key(s, w, e, tx))
            .on_input_paste(move |s: &mut State, content: &rune::ClipboardContent, tx: &mut Tx| {
                // Rows copied from a spreadsheet into an empty cell go across the table.
                let words = content.text();
                let empty = s.tables.get(w).and_then(|sh| sh.editing.as_ref()).is_some_and(|e| e.draft.text.is_empty());
                let trimmed = words.strip_suffix("\r\n").or_else(|| words.strip_suffix('\n')).unwrap_or(words);
                if !empty || !(trimmed.contains('\t') || trimmed.contains('\n')) {
                    return false;
                }
                super::end_edit(s, w, None, Then::Stay, tx);
                super::paste(s, w, words);
                true
            })
    } else {
        // Read where it cannot be changed: its words, to select and copy.
        readable(text(e.draft.text.as_str()).font(TextStyle::new(14.0, 400.0).lh(20.0)).color(ink()).wrap()).w_full()
    };
    let shell = col().key(w.key("edit-box")).w(p.w).min_h(46.0).max_h(320.0).fill(ground()).box_shadow(ring).box_shadow(lifted()).child(el().w_full().shrink().min_h(0.0).scroll(rune::ScrollSpec { stick_bottom: false }).child(el().w_full().px(16.0).py(13.0).child(body)));
    let shell = if sh.editable { shell } else { shell.focusable(true).key(w.key("edit")).on_key(move |s: &mut State, e: &KeyEvent, tx: &mut Tx| edit_key(s, w, e, tx)) };
    Some(shell.absolute(p.left - EDGE, p.top))
}

/// The grid's own line on its left, inside which what is placed over the grid is measured.
const EDGE: f32 = 1.0;

/// How tall a cell's list or calendar stands, to rise from the cell's top.
fn pop_height(s: &State, sh: &Sheet, pop: &Pop) -> f32 {
    let _ = s;
    let key = match pop {
        Pop::Tag(_) => "pop-tag",
        Pop::Date(_) => "pop-date",
    };
    rune::memo::layout_size(&sh.at.key(key)).map_or(if matches!(pop, Pop::Date(_)) { 330.0 } else { 160.0 }, |[_, h]| h + 10.0)
}

/// The keys of a cell's box (`GrowingText`'s): Enter keeps the words and goes down, ⌘Enter keeps
/// them and stays, Shift or ⌥ Enter is a new line, Tab and Shift Tab keep them and go right or
/// left, Escape drops them.
fn edit_key(s: &mut State, w: Where, e: &KeyEvent, tx: &mut Tx) -> bool {
    let Some(sh) = s.tables.get(w) else { return false };
    let Some(ed) = sh.editing.as_ref() else { return false };
    let value = sh.editable.then(|| ed.draft.text.clone());
    match e.key {
        Key::Escape => super::end_edit(s, w, None, Then::Stay, tx),
        Key::Enter if !e.mods.shift && !e.mods.alt => super::end_edit(s, w, value, if e.mods.cmd || e.mods.ctrl { Then::Stay } else { Then::Down }, tx),
        Key::Tab => super::end_edit(s, w, value, if e.mods.shift { Then::Left } else { Then::Right }, tx),
        _ => return false,
    }
    true
}

/// A single picked cell whose words are cut short, shown whole over it (`peek`): where, and the
/// words.
pub(crate) struct Peek {
    pub left: f32,
    pub top: f32,
    pub w: f32,
    pub text: String,
    pub up: bool,
    pub most: f32,
}

pub(crate) fn peek(s: &State, sh: &Sheet) -> Option<Peek> {
    let one = sh.one()?;
    let kind = sh.table().columns.get(one.c)?.kind;
    if sh.editing.is_some() || sh.sheet.is_some() || sh.float.is_some() || sh.dropping.is_some() || kind == Kind::Tag || sh.shut == Some(one) {
        return None;
    }
    let row = sh.row_at(one.r)?;
    let shown = match file::cell_link(sh.table(), row, one.c) {
        Some(l) => l.text,
        None => file::shown_text(row.at(one.c)),
    };
    if shown.is_empty() {
        return None;
    }
    let (_, cw) = column_x(s, sh, one.c);
    let logo = if kind == Kind::Name && sh.table().logo_col.is_some() { 28.0 } else { 0.0 };
    let room = cw - 32.0 - logo;
    let cut = if super::wraps(&sh.table().columns[one.c]) {
        // Three lines of the column's width.
        let lines = rune::text::measure(&shown, &[], TextStyle::new(14.0, 400.0).lh(20.0), Some(room.max(1.0))).1 / 20.0;
        lines > 3.05
    } else {
        measure(&shown.replace('\n', " "), cell_font()) > room + 1.0
    };
    if !cut {
        return None;
    }
    let view = scrolled(sh.at);
    let (sx, sy, vw, vh) = (view.offset[0], view.offset[1], view.viewport[0].max(1.0), view.viewport[1].max(1.0));
    let (x, _) = column_x(s, sh, one.c);
    let from = sx + super::stuck_width(s, sh, one.c);
    let cell_left = if one.c == sh.first() { from } else { x };
    // The editor's width, so pressing Enter does not move the words.
    let w = cw.min(200.0).max(cw.max(380.0).min(sx + vw - from - 8.0));
    let left = from.max(cell_left.min(sx + vw - w));
    let row_h = sh.row_h();
    let top = HEAD_H + one.r as f32 * row_h;
    let below = sy + vh - top - 8.0;
    let above = top + row_h - sy - HEAD_H - 8.0;
    let up = below < 200.0 && above > below;
    let most = row_h.max(320.0_f32.min(if up { above } else { below }));
    Some(Peek { left, top, w, text: shown, up, most })
}

/// A cut-off cell's words, picked, whole over it in the editor's place and look (`.tbl-peek`):
/// read only, a press on it going to the cell under it, a double press editing.
fn peek_box(s: &State, sh: &Sheet) -> Option<El> {
    if sh.holding {
        return None;
    }
    let w = sh.at;
    let p = peek(s, sh)?;
    let one = sh.one()?;
    let row_h = sh.row_h();
    let words = readable(text(p.text.as_str()).font(TextStyle::new(14.0, 400.0).lh(20.0)).color(ink()).wrap()).w_full();
    let mut b = col()
        .key(w.key("peek"))
        .w(p.w)
        .min_h(row_h)
        .max_h(p.most)
        .fill(ground())
        .box_shadow(shadow(0.0, 0.0, 0.0, 2.0, at_ring(), true))
        .box_shadow(lifted())
        .cursor(Cursor::Text)
        .child(el().w_full().shrink().min_h(0.0).scroll(rune::ScrollSpec { stick_bottom: false }).child(el().w_full().px(16.0).py(13.0).child(words)));
    if sh.editable {
        b = b.help("Double-click or press Enter to edit");
    }
    let b = b.on_drag(move |s: &mut State, d: &mut Drag, tx: &mut Tx| {
        // A press that selected no words goes to the cell under it, so the box never stands in
        // the way of the next row.
        if d.phase == Phase::Up && !d.cancelled() && d.dx.abs() < 2.0 && d.dy.abs() < 2.0 {
            if d.clicks == 2 {
                return super::start_edit(s, w, one, None, tx);
            }
            let Some(sh) = s.tables.get(w) else { return };
            let [_, py] = d.window_point();
            let [_, ty] = d.window_point();
            let _ = (py, ty);
            let (gx, gy) = (d.x + sh_left(sh, s), d.y + sh_top(sh, s, one));
            if let Some(Hit::Cell(at)) = hit_at(s, sh, gx, gy)
                && at != one
            {
                let p = Pick::one(at);
                if let Some(sh) = s.tables.get_mut(w) {
                    sh.pick = Some(p);
                }
                super::focus_keys(s, w, tx);
            }
        }
    });
    let top = if p.up { (p.top + row_h - p.most.min(peek_height(sh))).max(0.0) } else { p.top };
    Some(b.absolute(p.left - EDGE, top))
}

/// The peek's own height as last laid out, for one that rises from its cell's foot.
fn peek_height(sh: &Sheet) -> f32 {
    rune::memo::layout_size(&sh.at.key("peek")).map_or(sh.row_h(), |[_, h]| h)
}

fn sh_left(sh: &Sheet, s: &State) -> f32 {
    peek(s, sh).map_or(0.0, |p| p.left)
}

fn sh_top(sh: &Sheet, s: &State, _one: At) -> f32 {
    peek(s, sh).map_or(0.0, |p| if p.up { (p.top + sh.row_h() - p.most.min(peek_height(sh))).max(0.0) } else { p.top })
}

/* ------------------------------ a tag's list, a day's calendar ------------------------------ */

/// Whose list or calendar: the cell being edited, or a field of the row opened whole.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum PickFor {
    Cell,
    Field,
}

/// The rows of a tag's list (`items`): its tags narrowed by what is typed, Add for a new one,
/// Clear for a cell that has one.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Item {
    Tag(String),
    Add(String),
    Clear,
}

pub(crate) fn tag_items(column: &file::Column, value: &str, find: &str) -> Vec<Item> {
    let q = js::trim(find).to_lowercase();
    let mut out: Vec<Item> = column.values.iter().filter(|v| q.is_empty() || v.to_lowercase().contains(&q)).map(|v| Item::Tag(v.clone())).collect();
    let exact = column.values.iter().any(|v| v.to_lowercase() == q);
    if !q.is_empty() && !exact {
        out.push(Item::Add(js::trim(find).to_string()));
    }
    if !js::trim(value).is_empty() && q.is_empty() {
        out.push(Item::Clear);
    }
    out
}

/// The lit row: the one the keys or the pointer moved to, else the cell's own tag.
pub(crate) fn tag_lit(items: &[Item], value: &str, hi: Option<usize>) -> usize {
    let own = items.iter().position(|it| matches!(it, Item::Tag(v) if v.to_lowercase() == js::trim(value).to_lowercase()));
    match hi {
        None => own.unwrap_or(0),
        Some(h) => h.min(items.len().saturating_sub(1)),
    }
}

/// A row of a menu or a list (`.when-row`): six points round, as every menu row is, inside
/// a square menu.
fn menu_row(words: El, lit: bool) -> El {
    let r = row().w_full().px(9.0).py(6.0).gap(9.0).items_center().radius(6.0).cursor(Cursor::Pointer).child(words);
    if lit { r.fill(selected()) } else { r.hover(selected()) }
}

/// A box to type in at a menu's head (`.when-type`), its edge the faint ink while it has the
/// keys, which it takes as it opens (`autoFocus`).
fn type_row(field: El) -> El {
    row().mt(6.0).mx(5.0).mb(3.0).h(30.0).px(9.0).items_center().radius(6.0).fill(raised()).border(1.0, muted()).border_layout().cursor(Cursor::Text).child(field)
}

fn head_line(words: &str) -> El {
    row().px(9.0).pt(7.0).pb(3.0).child(text(words.to_uppercase()).font(sans(11.0, 600.0).tracking(0.66)).color(muted()))
}

fn separator() -> El {
    el().h(1.0).mx(9.0).my(5.0).fill(edge())
}

/// A tag cell's list (`TagPicker`), or its tag's own page (`TagEditor`).
fn tag_list(sh: &Sheet, column: &file::Column, value: &str, t: &TagPick, whose: PickFor, c: usize, wide: f32) -> El {
    let w = sh.at;
    let (find_box, name_box) = match whose {
        PickFor::Cell => (Box_::TagFind, Box_::TagName),
        PickFor::Field => (Box_::Field, Box_::FieldName),
    };
    if let Some((tag, name)) = &t.page {
        // One tag's name, colour and a way to delete it, in place of the list.
        let tone = column.tone(tag);
        let tag_c = tag.clone();
        let field = text_input(name, sans(12.5, 400.0))
            .key(w.key("tagname"))
            .single_line()
            .grow(1.0)
            .color(ink())
            .caret(ink(), rgba(110, 165, 255, 0.3))
            .on_edit(move |s: &mut State| super::edit_of(s, w, name_box))
            .on_input_key(move |s: &mut State, e: &KeyEvent, tx: &mut Tx| match e.key {
                Key::Enter | Key::Escape => {
                    tag_page_back(s, w, whose, c, tx);
                    true
                }
                _ => true.then_some(false).unwrap_or(false),
            });
        let back = row().size(28.0, 28.0).center().hover(hovered()).cursor(Cursor::Pointer).help("Back (esc)").label("Back to the tags").child(icon(&BACK, 14.0).color(dim())).on_press_tx(move |s: &mut State, tx| tag_page_back(s, w, whose, c, tx));
        let head = row().mt(6.0).mr(5.0).mb(3.0).ml(2.0).gap(2.0).items_center().child(back).child(row().grow(1.0).h(30.0).px(9.0).items_center().fill(raised()).border(1.0, edge()).border_layout().child(field));
        let swatches = row().gap(4.0).px(9.0).pt(4.0).pb(6.0).children(Tone::ALL.iter().map(|tn| {
            let (_, _, dot) = tone_colours(*tn);
            let tn = *tn;
            let tag = tag_c.clone();
            let ring = shadow(0.0, 0.0, 0.0, 1.5, dot, true);
            let sw = row().size(26.0, 26.0).center().hover_group().cursor(Cursor::Pointer).help(js::upper_first(tn.word())).label(tn.word()).child(el().size(10.0, 10.0).fill(dot)).on_press(move |s: &mut State| super::tone_tag(s, w, c, &tag, tn));
            // Its ring under the pointer, and on the tag's own colour.
            if tn == tone { sw.box_shadow(ring) } else { sw.child(el().absolute(0.0, 0.0).size(26.0, 26.0).box_shadow(ring).group_opacity(0.0, 1.0)) }
        }));
        let count = super::tag_count(sh, c, tag);
        let tag_d = tag.clone();
        let delete = menu_row(row().grow(1.0).child(label("Delete tag", 13.0, 400.0, tone_colours(Tone::Rose).1)), false).child(text(rows_word(count)).font(mono(11.0, 14.0)).color(muted())).on_press_tx(move |s: &mut State, tx| {
            super::drop_tag(s, w, c, &tag_d);
            close_tag_page(s, w, whose, tx);
        });
        return col().child(head).child(head_line("Colour")).child(swatches).child(separator()).child(delete);
    }
    let items = tag_items(column, value, &t.find.text);
    let lit = tag_lit(&items, value, t.hi);
    let next = super::edit::new_tone(column, js::trim(&t.find.text));
    let field = text_input(&t.find, sans(12.5, 400.0))
        .key(w.key("tagfind"))
        .single_line()
        .grow(1.0)
        .color(ink())
        .placeholder("Find or add a tag", muted())
        .caret(ink(), rgba(110, 165, 255, 0.3))
        .on_edit(move |s: &mut State| super::edit_of(s, w, find_box))
        .on_input_change(move |s: &mut State, _| set_tag_hi(s, w, whose, Some(0)))
        .on_input_key(move |s: &mut State, e: &KeyEvent, tx: &mut Tx| tag_key(s, w, whose, c, e, tx));
    let mut list = col().gap(1.0);
    for (n, it) in items.iter().enumerate() {
        let is_lit = n == lit;
        let own = matches!(it, Item::Tag(v) if v.to_lowercase() == js::trim(value).to_lowercase());
        let body = match it {
            Item::Tag(v) => {
                let c = chip(column, v);
                if own { c.box_shadow(shadow(0.0, 0.0, 0.0, 1.0, tone_colours(column.tone(v)).1, true)) } else { c }
            }
            Item::Add(v) => row().gap(9.0).items_center().child(label("Add", 13.0, 400.0, muted())).child(chip_in(next, v)),
            Item::Clear => label("Clear", 13.0, 400.0, muted()),
        };
        let it_c = it.clone();
        let r = menu_row(row().grow(1.0).pr(25.0).child(body), is_lit)
            .on_pointer_move(move |s: &mut State, _| {
                if lit != n {
                    set_tag_hi(s, w, whose, Some(n));
                }
            })
            .on_press_tx(move |s: &mut State, tx| take_tag(s, w, whose, c, &it_c, Then::Stay, tx));
        // Each tag's own menu waits at the end of its row, shown on the lit one (`.tbl-pick-more`).
        let mut line = el().w(wide).child(r);
        if let (Item::Tag(v), true) = (it, sh.editable) {
            let v = v.clone();
            let more = row()
                .absolute(wide - 4.0 - 24.0, 5.0)
                .size(24.0, 24.0)
                .center()
                .radius(6.0)
                .hover(hovered())
                .hover_group()
                .cursor(Cursor::Pointer)
                .help("Rename, colour or delete")
                .label(format!("Edit {v}"))
                .opacity(if is_lit { 1.0 } else { 0.0 })
                .child(icon(&MORE, 14.0).color(dim()).group_color(ink()))
                .on_press(move |s: &mut State| open_tag_page(s, w, whose, &v));
            line = line.child(more);
        }
        list = list.child(line);
    }
    if items.is_empty() {
        list = list.child(row().px(9.0).pt(2.0).pb(6.0).child(text("No tags yet. Type one to add it.").font(sans(12.0, 400.0)).color(muted())));
    }
    let list = col().mt(2.0).max_h(240.0).scroll(rune::ScrollSpec { stick_bottom: false }).child(list);
    col().key(w.key("pop-tag")).child(type_row(field)).child(list)
}

fn tag_pick_mut(s: &mut State, w: Where, whose: PickFor) -> Option<&mut TagPick> {
    let sh = s.tables.get_mut(w)?;
    let pop = match whose {
        PickFor::Cell => sh.editing.as_mut()?.pop.as_mut()?,
        PickFor::Field => sh.fields.as_mut()?.pick.as_mut()?,
    };
    match pop {
        Pop::Tag(t) => Some(t),
        _ => None,
    }
}

fn set_tag_hi(s: &mut State, w: Where, whose: PickFor, hi: Option<usize>) {
    if let Some(t) = tag_pick_mut(s, w, whose) {
        t.hi = hi;
    }
}

fn open_tag_page(s: &mut State, w: Where, whose: PickFor, tag: &str) {
    if let Some(t) = tag_pick_mut(s, w, whose) {
        let mut name = TextEdit::new(tag);
        name.anchor = 0;
        name.focus = name.text.len();
        name.focused = true;
        t.find.focused = false;
        t.page = Some((tag.to_string(), name));
    }
}

/// Back from a tag's page: what was typed renames the tag, and the list comes back on it.
fn tag_page_back(s: &mut State, w: Where, whose: PickFor, c: usize, tx: &mut Tx) {
    let Some(t) = tag_pick_mut(s, w, whose) else { return };
    let Some((tag, name)) = t.page.take() else { return };
    let to = js::trim(&name.text).to_string();
    t.hi = None;
    t.find.focused = true;
    if !to.is_empty() && to != tag {
        super::rename_tag(s, w, c, &tag, &to);
    }
    let _ = tx;
}

fn close_tag_page(s: &mut State, w: Where, whose: PickFor, _tx: &mut Tx) {
    if let Some(t) = tag_pick_mut(s, w, whose) {
        t.page = None;
        t.hi = None;
        t.find.focused = true;
    }
}

/// A row of the list taken: its tag written, Add's new one, or the cell cleared.
fn take_tag(s: &mut State, w: Where, whose: PickFor, c: usize, it: &Item, then: Then, tx: &mut Tx) {
    let v = match it {
        Item::Tag(v) | Item::Add(v) => v.clone(),
        Item::Clear => String::new(),
    };
    match whose {
        PickFor::Cell => super::end_edit(s, w, Some(v), then, tx),
        PickFor::Field => {
            if let Some(sh) = s.tables.get_mut(w) {
                sh.fields = None;
            }
            super::write_field(s, w, c, v);
            tx.focus(w.key("sheet"), false);
        }
    }
}

/// The keys of a tag's list (`TagPicker`'s): ↑ ↓ move the light, Enter takes it, Tab takes it
/// while something is typed (else keeps the cell) and moves on, Escape closes.
fn tag_key(s: &mut State, w: Where, whose: PickFor, c: usize, e: &KeyEvent, tx: &mut Tx) -> bool {
    let Some(sh) = s.tables.get(w) else { return false };
    let Some(column) = sh.table().columns.get(c).cloned() else { return false };
    let value = match whose {
        PickFor::Cell => sh.editing.as_ref().map(|e| sh.value(At { r: e.r, c: e.c }).to_string()).unwrap_or_default(),
        PickFor::Field => sh.sheet.map(|r| sh.value(At { r, c }).to_string()).unwrap_or_default(),
    };
    let Some(t) = tag_pick_mut(s, w, whose) else { return false };
    let items = tag_items(&column, &value, &t.find.text);
    let now = tag_lit(&items, &value, t.hi);
    let find = js::trim(&t.find.text).to_string();
    match e.key {
        Key::Down => t.hi = Some((now + 1).min(items.len().saturating_sub(1))),
        Key::Up => t.hi = Some(now.saturating_sub(1)),
        Key::Enter => match items.get(now) {
            Some(it) => take_tag(s, w, whose, c, it, Then::Stay, tx),
            None => close_pick(s, w, whose, tx),
        },
        Key::Tab => {
            let then = if e.mods.shift { Then::Left } else { Then::Right };
            if find.is_empty() {
                take_tag(s, w, whose, c, &Item::Tag(value.clone()).clone_if_some(&value), then, tx);
            } else if let Some(it) = items.get(now) {
                take_tag(s, w, whose, c, it, then, tx);
            } else {
                close_pick(s, w, whose, tx);
            }
        }
        Key::Escape => close_pick(s, w, whose, tx),
        _ => return false,
    }
    true
}

impl Item {
    /// The cell's own value kept as it is (`onPick(value, …)`): empty stays empty.
    fn clone_if_some(&self, value: &str) -> Item {
        if js::trim(value).is_empty() { Item::Clear } else { self.clone() }
    }
}

/// A list or a calendar closed with nothing taken (`onClose`).
fn close_pick(s: &mut State, w: Where, whose: PickFor, tx: &mut Tx) {
    match whose {
        PickFor::Cell => super::end_edit(s, w, None, Then::Stay, tx),
        PickFor::Field => {
            if let Some(f) = s.tables.get_mut(w).and_then(|sh| sh.fields.as_mut()) {
                f.pick = None;
            }
            tx.focus(w.key("sheet"), false);
        }
    }
}

fn date_pick_mut(s: &mut State, w: Where, whose: PickFor) -> Option<&mut DatePick> {
    let sh = s.tables.get_mut(w)?;
    let pop = match whose {
        PickFor::Cell => sh.editing.as_mut()?.pop.as_mut()?,
        PickFor::Field => sh.fields.as_mut()?.pick.as_mut()?,
    };
    match pop {
        Pop::Date(d) => Some(d),
        _ => None,
    }
}

const DAY_MS: f64 = 86_400_000.0;

/// A date cell's calendar (`DatePicker`): a box to type a day in words, the month with its two
/// arrows, the days, and Today and Clear.
fn calendar(s: &State, sh: &Sheet, value: &str, d: &DatePick, whose: PickFor, c: usize) -> El {
    use crate::rules::calendar::Day;
    let w = sh.at;
    let words_box = match whose {
        PickFor::Cell => Box_::Date,
        PickFor::Field => Box_::Field,
    };
    let typed = file::day_typed(&d.words.text, s.now);
    let today = Day::of(s.now);
    let (y, m) = d.month;
    let first = Day::first_of(y, m);
    let lead = first.weekday();
    let in_month = crate::rules::calendar::days_in(y, m);
    let weeks = ((lead + in_month) as f32 / 7.0).ceil() as i64;
    let field = text_input(&d.words, sans(12.5, 400.0))
        .key(w.key("date"))
        .single_line()
        .grow(1.0)
        .color(ink())
        .placeholder("Type a date", muted())
        .caret(ink(), rgba(110, 165, 255, 0.3))
        .on_edit(move |s: &mut State| super::edit_of(s, w, words_box))
        .on_input_change(move |s: &mut State, _| follow_typed(s, w, whose))
        .on_input_key(move |s: &mut State, e: &KeyEvent, tx: &mut Tx| date_key(s, w, whose, c, e, tx));
    let arrow = |glyph: &'static rune::Icon, label_words: &'static str, by: i64| row().size(28.0, 28.0).center().hover(hovered()).cursor(Cursor::Pointer).label(label_words).child(icon(glyph, 14.0).color(dim())).on_press(move |s: &mut State| {
        if let Some(d) = date_pick_mut(s, w, whose) {
            let (y, m, _) = Day::first_of(d.month.0, d.month.1 + by).ymd();
            d.month = (y, m);
        }
    });
    let head = row().pt(4.0).pr(4.0).pb(2.0).pl(9.0).gap(2.0).items_center().child(row().grow(1.0).child(label(&format!("{} {y}", file::MONTHS_LONG[m as usize]), 13.0, 400.0, ink()))).child(arrow(&BACK, "Month before", -1)).child(arrow(&NEXT, "Month after", 1));
    let cell_w: f32 = 260.0 - 8.0 - 2.0 - 10.0;
    // Seven equal shares of the row, one point between (`repeat(7, 1fr)`, `gap: 1px`).
    let day_w = (cell_w - 6.0) / 7.0;
    let mut grid = col().px(5.0).pb(4.0).gap(1.0);
    let mut letters = row().gap(1.0);
    for l in ["S", "M", "T", "W", "T", "F", "S"] {
        letters = letters.child(row().w(day_w).h(24.0).center().child(text(l).font(mono(10.5, 13.0)).color(muted())));
    }
    grid = grid.child(letters);
    let picked = typed.is_finite().then(|| Day((typed / DAY_MS).floor() as i64));
    for week in 0..weeks {
        let mut line = row().gap(1.0);
        for k in 0..7 {
            let day = Day(first.0 + week * 7 + k - lead);
            let (_, dm, dd) = day.ymd();
            let out = dm != m;
            let on = picked == Some(day);
            let is_today = day == today;
            let tone = if on { rgb(255, 255, 255) } else if out || !is_today { if out { muted() } else { dim() } } else { ink() };
            let mut b = row().w(day_w).h(30.0).center().radius(6.0).cursor(Cursor::Pointer).child(text(dd.to_string()).font(figures(12.5, 400.0)).color(tone));
            if out {
                b = b.opacity(0.55);
            }
            b = if on { b.fill(at_ring()) } else { b.hover(selected()) };
            if is_today && !on {
                b = b.box_shadow(shadow(0.0, 0.0, 0.0, 1.0, rule(), true));
            }
            let ms = day.0 as f64 * DAY_MS;
            b = b.on_press_tx(move |s: &mut State, tx| pick_day(s, w, whose, c, Some(ms), tx));
            line = line.child(b);
        }
        grid = grid.child(line);
    }
    let today_ms = Day::first_of(today.ymd().0, today.ymd().1).0 as f64 * DAY_MS + (today.ymd().2 - 1) as f64 * DAY_MS;
    let foot_row = |words: El| row().px(9.0).py(6.0).items_center().radius(6.0).hover(selected()).cursor(Cursor::Pointer).child(words);
    let mut foot = row().gap(2.0).pt(3.0).mt(2.0).border_sides([1.0, 0.0, 0.0, 0.0], edge()).border_layout().child(foot_row(label("Today", 13.0, 400.0, dim())).on_press_tx(move |s: &mut State, tx| pick_day(s, w, whose, c, Some(today_ms), tx)));
    if !js::trim(value).is_empty() {
        foot = foot.child(foot_row(label("Clear", 13.0, 400.0, muted())).on_press_tx(move |s: &mut State, tx| pick_day(s, w, whose, c, None, tx)));
    }
    col().key(w.key("pop-date")).child(type_row(field)).child(head).child(grid).child(foot)
}

/// The calendar turns to the day typed, when what is typed is one.
fn follow_typed(s: &mut State, w: Where, whose: PickFor) {
    let now = s.now;
    if let Some(d) = date_pick_mut(s, w, whose) {
        let day = file::day_typed(&d.words.text, now);
        if day.is_finite() {
            let (y, m, _) = crate::rules::calendar::Day((day / DAY_MS).floor() as i64).ymd();
            d.month = (y, m);
        }
    }
}

/// A day picked, or the cell cleared (`pickDay`, Clear): written as the column writes its days.
fn pick_day(s: &mut State, w: Where, whose: PickFor, c: usize, day: Option<f64>, tx: &mut Tx) {
    let Some(sh) = s.tables.get(w) else { return };
    let cells = super::column_cells(sh.table(), c);
    let v = day.map(|d| file::date_words(d, &cells)).unwrap_or_default();
    match whose {
        PickFor::Cell => super::end_edit(s, w, Some(v), Then::Stay, tx),
        PickFor::Field => {
            if let Some(sh) = s.tables.get_mut(w) {
                sh.fields = None;
            }
            super::write_field(s, w, c, v);
            tx.focus(w.key("sheet"), false);
        }
    }
}

/// The calendar's keys (`DatePicker`'s): Enter writes what is typed and goes down, ↑ and ↓ move
/// the day by one, Tab writes it and moves on, Escape closes.
fn date_key(s: &mut State, w: Where, whose: PickFor, c: usize, e: &KeyEvent, tx: &mut Tx) -> bool {
    let now = s.now;
    match e.key {
        Key::Enter | Key::Tab => {
            let typed = date_pick_mut(s, w, whose).map(|d| js::trim(&d.words.text).to_string()).unwrap_or_default();
            let then = if e.key == Key::Enter { Then::Down } else if e.mods.shift { Then::Left } else { Then::Right };
            match whose {
                PickFor::Cell => super::end_edit(s, w, Some(typed), then, tx),
                PickFor::Field => {
                    let Some(sh) = s.tables.get(w) else { return true };
                    let was = sh.sheet.map(|r| sh.value(At { r, c }).to_string()).unwrap_or_default();
                    let cells = super::column_cells(sh.table(), c);
                    if let Some(sh) = s.tables.get_mut(w) {
                        sh.fields = None;
                    }
                    super::write_field(s, w, c, super::as_date(&typed, &was, &cells, now));
                    tx.focus(w.key("sheet"), false);
                }
            }
        }
        Key::Up | Key::Down => {
            let Some(sh) = s.tables.get(w) else { return true };
            let cells = super::column_cells(sh.table(), c);
            let Some(d) = date_pick_mut(s, w, whose) else { return true };
            let typed = file::day_typed(&d.words.text, now);
            let today = crate::rules::calendar::Day::of(now);
            let from = if typed.is_finite() { typed } else { today.0 as f64 * DAY_MS };
            let next = from + if e.key == Key::Down { DAY_MS } else { -DAY_MS };
            let words = file::date_words(next, &cells);
            d.words = TextEdit::new(words.clone());
            d.words.anchor = words.len();
            d.words.focus = words.len();
            d.words.focused = true;
            follow_typed(s, w, whose);
        }
        Key::Escape => close_pick(s, w, whose, tx),
        _ => return false,
    }
    true
}

/* ------------------------------ the row opened whole ------------------------------ */

/// The row opened whole (`RowSheet`), at the table's right.
fn sheet_panel(s: &State, sh: &Sheet, inner: f32, frame_h: f32) -> Option<El> {
    let w = sh.at;
    let r = sh.sheet?;
    let cells = sh.row_at(r)?;
    let table = sh.table();
    let count = sh.order().len();
    let name_at = usize::from(table.ranked);
    let link = file::cell_link(table, cells, name_at);
    let name = file::shown_text(cells.at(name_at));
    let wide = 460.0_f32.min(inner * 0.92);
    let step = |glyph: &'static rune::Icon, words: &'static str, title: &'static str, off: bool, by: isize| {
        let b = row().size(30.0, 30.0).center().label(words).help(title).child(icon(glyph, 14.0).color(dim()));
        if off { b.opacity(0.35) } else { b.hover(hovered()).cursor(Cursor::Pointer).on_press_tx(move |s: &mut State, tx| step_row(s, w, by, tx)) }
    };
    let title = match &link {
        Some(l) => l.text.clone(),
        None if name.is_empty() => "Untitled".to_string(),
        None => name.clone(),
    };
    let head = row()
        .pt(12.0)
        .pr(12.0)
        .pb(12.0)
        .pl(24.0)
        .gap(2.0)
        .items_center()
        .border_sides([0.0, 0.0, 1.0, 0.0], line())
        .border_layout()
        .child(row().grow(1.0).min_w(0.0).child(text(title).font(sans(16.0, 600.0)).color(ink()).truncate()))
        .child(text(format!("{} of {}", r + 1, count)).font(mono(11.0, 14.0).tracking(0.66)).text_case(TextCase::Uppercase).color(muted()).mx(8.0))
        .child(step(&ABOVE, "Row above", "Row above (↑)", r == 0, -1))
        .child(step(&BELOW, "Row below", "Row below (↓)", r + 1 >= count, 1))
        .child(row().size(30.0, 30.0).center().hover(hovered()).cursor(Cursor::Pointer).label("Close").help("Close (esc)").child(icon(&CLOSE, 14.0).color(dim())).on_press_tx(move |s: &mut State, tx| close_sheet(s, w, tx)));
    let from = sh.pick.map(|p| p.a.c);
    let mut body = col().px(24.0).pt(10.0).pb(28.0);
    for (i, c) in table.columns.iter().enumerate() {
        let tone = if Some(i) == from { ink() } else { muted() };
        let name_line = row().items_center().child(icon(kind_mark(c.kind), 13.0).color(muted()).mr(7.0)).child(text(c.name.as_str()).font(head_font()).text_case(TextCase::Uppercase).color(tone));
        let field = field_el(s, sh, i, cells.at(i), sh.editable && !(table.ranked && i == 0));
        body = body.child(col().key(format!("{}-{i}", w.key("field"))).py(10.0).gap(5.0).child(name_line).child(field));
    }
    let panel = col()
        .key(w.key("sheet"))
        .absolute(inner - wide, 0.0)
        .size(wide, frame_h)
        .fill(ground())
        .border_sides([0.0, 0.0, 0.0, 1.0], rule())
        .border_layout()
        .box_shadow(lifted())
        .focusable(true)
        .on_key(move |s: &mut State, e: &KeyEvent, tx: &mut Tx| sheet_key(s, w, e, tx))
        .child(head)
        .child(el().key(w.key("sheet-body")).w_full().grow(1.0).min_h(0.0).scroll(rune::ScrollSpec { stick_bottom: false }).child(body))
        .role("dialog")
        .label(if name.is_empty() { "Row".to_string() } else { name });
    Some(panel)
}

/// One field of the row opened whole (`Field`).
fn field_el(s: &State, sh: &Sheet, c: usize, value: &str, editable: bool) -> El {
    let w = sh.at;
    let column = &sh.table().columns[c];
    let open = sh.fields.as_ref().filter(|f| f.c == c);
    let none = |words: &str| text(words.to_string()).font(sans(13.0, 400.0)).color(muted());
    if column.kind == Kind::Date && editable {
        let button = row().py(4.0).cursor(Cursor::Pointer).child(if js::trim(value).is_empty() { none("Pick a date") } else { text(js::trim(value).to_string()).font(TextStyle::new(14.0, 400.0).lh(20.0)).color(ink()) }).on_press_tx(move |s: &mut State, tx| toggle_field_pick(s, w, c, PopKind::Date, tx));
        let mut f = col().child(button);
        if let Some(Field { pick: Some(Pop::Date(d)), .. }) = open {
            f = f.child(col().mt(4.0).w(260.0).padding(4.0).fill(pop()).border(1.0, rule()).border_layout().box_shadow(lifted()).child(calendar(s, sh, value, d, PickFor::Field, c)));
        }
        return f;
    }
    if column.kind == Kind::Tag {
        let face = if js::trim(value).is_empty() { none(if editable { "Pick a tag" } else { "" }) } else { chip(column, value) };
        let mut button = row().py(4.0).child(face);
        if editable {
            button = button.cursor(Cursor::Pointer).on_press_tx(move |s: &mut State, tx| toggle_field_pick(s, w, c, PopKind::Tag, tx));
        }
        let mut f = col().child(button);
        if let Some(Field { pick: Some(Pop::Tag(t)), .. }) = open {
            f = f.child(col().mt(4.0).w(260.0).padding(4.0).fill(pop()).border(1.0, rule()).border_layout().box_shadow(lifted()).child(tag_list(sh, column, value, t, PickFor::Field, c, 250.0)));
        }
        return f;
    }
    if !editable {
        return readable(if value.is_empty() { none("Empty") } else { text(value.to_string()).font(TextStyle::new(14.0, 400.0).lh(20.0)).color(ink()).wrap() });
    }
    let draft = match open {
        Some(f) if f.pick.is_none() => f.draft.clone(),
        _ => TextEdit::new(value),
    };
    let field = text_input(&draft, TextStyle::new(14.0, 400.0).lh(20.0))
        .key(w.key(&format!("field{c}")))
        .w_full()
        .color(ink())
        .caret(ink(), rgba(110, 165, 255, 0.3))
        .on_edit(move |s: &mut State| {
            // The field opens for writing as the keys come to it.
            if let Some(sh) = s.tables.get_mut(w)
                && sh.fields.as_ref().is_none_or(|f| f.c != c || f.pick.is_some())
            {
                let value = sh.sheet.map(|r| sh.value(At { r, c }).to_string()).unwrap_or_default();
                sh.fields = Some(Field { c, draft: TextEdit::new(value), pick: None, held: false });
            }
            super::edit_of(s, w, Box_::Field)
        })
        .on_input_key(move |s: &mut State, e: &KeyEvent, tx: &mut Tx| match e.key {
            // Escape drops the typing, as it does in the grid.
            Key::Escape => {
                if let Some(sh) = s.tables.get_mut(w) {
                    sh.fields = None;
                }
                tx.focus(w.key("sheet"), false);
                true
            }
            Key::Enter if e.mods.cmd || e.mods.ctrl => {
                tx.focus(w.key("sheet"), false);
                true
            }
            _ => false,
        });
    let focused = open.is_some_and(|f| f.pick.is_none() && f.draft.focused);
    // Its edge shows under the pointer, and stronger while it has the keys (`.tbl-field-box`).
    let frame = el().absolute(0.0, 0.0).w_full().h_full().border(1.0, line()).group_opacity(0.0, 1.0);
    // 32 points tall for a line, as the web's box sizes itself to its words (`GrowingText`).
    let b = row().w_full().ml(-9.0).mr(-9.0).px(8.0).py(5.0).max_h(360.0).hover_group().border(1.0, if focused { rule() } else { rgba(0, 0, 0, 0.0) }).border_layout().cursor(Cursor::Text).child(field);
    if focused { b } else { b.child(frame) }
}

fn toggle_field_pick(s: &mut State, w: Where, c: usize, kind: PopKind, tx: &mut Tx) {
    let now = s.now;
    let Some(sh) = s.tables.get_mut(w) else { return };
    let open = sh.fields.as_ref().is_some_and(|f| f.c == c && f.pick.is_some());
    if open {
        sh.fields = None;
        tx.focus(w.key("sheet"), false);
        return;
    }
    let value = sh.sheet.map(|r| sh.value(At { r, c }).to_string()).unwrap_or_default();
    let pick = match kind {
        PopKind::Tag => {
            let mut find = TextEdit::new("");
            find.focused = true;
            Pop::Tag(TagPick { find, hi: None, page: None, held: false })
        }
        PopKind::Date => Pop::Date(DatePick::new(&value, now)),
    };
    sh.fields = Some(Field { c, draft: TextEdit::new(value), pick: Some(pick), held: false });
}

fn close_sheet(s: &mut State, w: Where, tx: &mut Tx) {
    if let Some(sh) = s.tables.get_mut(w) {
        sh.sheet = None;
        sh.fields = None;
    }
    super::focus_keys(s, w, tx);
}

/// The row above or below opened, the pick following it.
fn step_row(s: &mut State, w: Where, by: isize, tx: &mut Tx) {
    let Some(sh) = s.tables.get(w) else { return };
    let Some(r) = sh.sheet else { return };
    let n = sh.order().len();
    let to = (r as isize + by).clamp(0, n.saturating_sub(1) as isize) as usize;
    let c = sh.pick.map_or(sh.first(), |p| p.a.c);
    super::go_to(s, w, Some(Pick::one(At { r: to, c })), tx);
    if let Some(sh) = s.tables.get_mut(w) {
        sh.sheet = Some(to);
        sh.fields = None;
    }
}

/// The keys of the row opened whole (`RowSheet`'s): Escape closes it, ↑ and ↓ step rows, ⌘Z
/// and ⌘⇧Z take back and do again; a field's own keys stay the field's.
fn sheet_key(s: &mut State, w: Where, e: &KeyEvent, tx: &mut Tx) -> bool {
    if e.key == Key::Escape && tx.focused() == Some(w.key("sheet").as_str()) {
        close_sheet(s, w, tx);
        return true;
    }
    if tx.focused() != Some(w.key("sheet").as_str()) {
        return false;
    }
    match e.key {
        Key::Command('z') => {
            if e.mods.shift { super::redo(s, w) } else { super::undo(s, w) }
            true
        }
        Key::Command('y') => {
            super::redo(s, w);
            true
        }
        Key::Up | Key::Down => {
            step_row(s, w, if e.key == Key::Down { 1 } else { -1 }, tx);
            true
        }
        _ => false,
    }
}

/* ------------------------------ the menus ------------------------------ */

/// What a row of a menu does.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Act {
    SortCol(bool),
    StopSort,
    Rename,
    TypePage,
    TypeBack,
    SetType(Kind),
    InsertLeft,
    InsertRight,
    MoveLeft,
    MoveRight,
    Fit,
    Wrap,
    DeleteColumn,
    OpenRow,
    EditCell,
    Copy,
    Paste,
    OpenLink(String),
    InsertAbove,
    InsertBelow,
    MoveUp,
    MoveDown,
    Clear,
    DeleteRows,
}

/// A row of a menu at a point (`MenuRow`), or a line between rows, or a page's head.
#[derive(Clone, Debug)]
pub(crate) struct MenuLine {
    pub words: String,
    pub hint: Option<String>,
    pub act: Act,
    pub off: bool,
    pub sep: bool,
    pub ticked: bool,
    pub mark: Option<Kind>,
    pub head: bool,
}

fn line_of(words: &str, hint: Option<&str>, act: Act, off: bool) -> MenuLine {
    MenuLine { words: words.to_string(), hint: hint.map(str::to_string), act, off, sep: false, ticked: false, mark: None, head: false }
}

fn sep_line() -> MenuLine {
    MenuLine { words: String::new(), hint: None, act: Act::Copy, off: true, sep: true, ticked: false, mark: None, head: false }
}

/// The rows of the menu at a point open on the sheet at `w`, in order, lines between them
/// included (as rows that are off, which the keys pass over).
pub(crate) fn float_rows(s: &State, w: Where) -> Vec<MenuLine> {
    let Some(sh) = s.tables.get(w) else { return Vec::new() };
    let Some(f) = sh.float.as_ref() else { return Vec::new() };
    let table = sh.table();
    let (first, last) = (sh.first(), sh.last());
    let mut out = Vec::new();
    if let Some(col) = f.col {
        let Some(c) = table.columns.get(col) else { return out };
        if f.page_type {
            let mut back = line_of(&c.name, None, Act::TypeBack, false);
            back.head = true;
            out.push(back);
            for (kind, words) in TYPES {
                let on = (if c.kind == Kind::Name { Kind::Text } else { c.kind }) == kind;
                let mut l = line_of(words, on.then_some("✓"), Act::SetType(kind), false);
                l.ticked = on;
                l.mark = Some(kind);
                out.push(l);
            }
            return out;
        }
        out.push(line_of(&format!("Sort {}", sort_words(c.kind, true)), None, Act::SortCol(true), false));
        out.push(line_of(&format!("Sort {}", sort_words(c.kind, false)), None, Act::SortCol(false), false));
        if sh.sort.is_some_and(|so| so.col == col) {
            out.push(line_of("Stop sorting", None, Act::StopSort, false));
        }
        let wrap = wordy(c.kind).then(|| line_of("Wrap text", c.wrap.then_some("✓"), Act::Wrap, false));
        if sh.editable {
            out.push(sep_line());
            out.push(line_of("Rename", None, Act::Rename, false));
            if !(table.ranked && col == 0) {
                out.push(line_of("Type", Some(&format!("{} ›", type_name(c.kind))), Act::TypePage, false));
            }
            out.push(line_of("Insert column left", None, Act::InsertLeft, false));
            out.push(line_of("Insert column right", None, Act::InsertRight, false));
            out.push(line_of("Move left", Some("⌥←"), Act::MoveLeft, col <= first));
            out.push(line_of("Move right", Some("⌥→"), Act::MoveRight, col >= last));
            out.push(line_of("Fit to its contents", None, Act::Fit, false));
            out.extend(wrap);
            out.push(sep_line());
            out.push(line_of("Delete column", None, Act::DeleteColumn, table.columns.len() - first <= 1));
        } else {
            out.push(line_of("Fit to its contents", None, Act::Fit, false));
            out.extend(wrap);
        }
        return out;
    }
    let pick = sh.pick;
    out.push(line_of("Open row", Some("Space"), Act::OpenRow, false));
    if pick.is_some() {
        out.push(line_of(if sh.editable { "Edit cell" } else { "Read cell" }, Some("↵"), Act::EditCell, false));
    }
    out.push(line_of("Copy", Some("⌘C"), Act::Copy, false));
    if sh.editable {
        out.push(line_of("Paste", Some("⌘V"), Act::Paste, false));
    }
    if let Some(p) = pick
        && let Some(row) = sh.row_at(p.a.r)
        && let Some(l) = file::cell_link(table, row, p.a.c)
    {
        let words = if l.href.starts_with("mailto:") { "Write an email" } else { "Open link" };
        out.push(line_of(words, Some("⌘click"), Act::OpenLink(l.href), false));
    }
    if let (true, Some(b)) = (sh.editable, sh.sel()) {
        let n = b.r1 - b.r0 + 1;
        let sorted = sh.sort.is_some();
        out.push(sep_line());
        out.push(line_of("Insert row above", None, Act::InsertAbove, false));
        out.push(line_of("Insert row below", Some("⇧↵"), Act::InsertBelow, false));
        out.push(line_of("Move up", Some(if sorted { "Sorted" } else { "⌥↑" }), Act::MoveUp, sorted || b.r0 == 0));
        out.push(line_of("Move down", Some(if sorted { "Sorted" } else { "⌥↓" }), Act::MoveDown, sorted || b.r1 + 1 >= sh.order().len()));
        out.push(line_of("Clear", Some("⌫"), Act::Clear, false));
        out.push(sep_line());
        out.push(line_of(&if n > 1 { format!("Delete {n} rows") } else { "Delete row".to_string() }, None, Act::DeleteRows, false));
    }
    out
}

/// A menu row pressed: what it does, then the keys back to the table unless it opened a box.
pub(crate) fn press_row(s: &mut State, w: Where, act: Act, tx: &mut Tx) {
    let Some(sh) = s.tables.get(w) else { return };
    let col = sh.float.as_ref().and_then(|f| f.col);
    let first = sh.first();
    let sel = sh.sel();
    let pick = sh.pick;
    let close = |s: &mut State| {
        if let Some(sh) = s.tables.get_mut(w) {
            sh.float = None;
        }
    };
    match act {
        Act::TypePage | Act::TypeBack => {
            if let Some(f) = s.tables.get_mut(w).and_then(|sh| sh.float.as_mut()) {
                f.page_type = act == Act::TypePage;
                f.lit = None;
            }
            return;
        }
        Act::SortCol(up) => {
            close(s);
            if let Some(c) = col {
                super::set_sort(s, w, Some(Sort { col: c, up }));
            }
        }
        Act::StopSort => {
            close(s);
            super::set_sort(s, w, None);
        }
        Act::Rename => {
            close(s);
            if let Some(c) = col {
                super::begin_rename(s, w, c);
            }
            return;
        }
        Act::SetType(kind) => {
            close(s);
            if let Some(c) = col {
                super::set_kind(s, w, c, kind);
            }
        }
        Act::InsertLeft | Act::InsertRight => {
            close(s);
            if let Some(c) = col {
                let at = if act == Act::InsertLeft { c.max(first) } else { c + 1 };
                let t = edit::add_column(s.tables.get(w).map_or(&Table::empty(), |sh| sh.table()), at, "");
                super::change_columns(s, w, t, Some(at), None);
                super::begin_rename(s, w, at);
                return;
            }
        }
        Act::MoveLeft | Act::MoveRight => {
            close(s);
            if let Some(c) = col {
                super::move_cols_to(s, w, c, c, if act == Act::MoveLeft { c.saturating_sub(1) } else { c + 2 }, tx);
            }
        }
        Act::Fit => {
            close(s);
            if let Some(c) = col {
                super::fit_column(s, w, c);
            }
        }
        Act::Wrap => {
            close(s);
            if let Some(c) = col {
                super::toggle_wrap(s, w, c);
            }
        }
        Act::DeleteColumn => {
            close(s);
            if let Some(c) = col {
                let t = edit::drop_column(s.tables.get(w).map_or(&Table::empty(), |sh| sh.table()), c);
                super::change_columns(s, w, t, None, None);
            }
        }
        Act::OpenRow => {
            close(s);
            if let Some(p) = pick {
                super::open_row(s, w, p.a.r, tx);
                return;
            }
        }
        Act::EditCell => {
            close(s);
            if let Some(p) = pick {
                super::start_edit(s, w, p.a, None, tx);
                return;
            }
        }
        Act::Copy => {
            close(s);
            if let Some(sh) = s.tables.get(w)
                && sh.sel().is_some()
            {
                tx.copy(edit::to_clipboard(&super::picked_grid(sh)));
            }
        }
        Act::Paste => {
            close(s);
            tx.read_clipboard(move |s: &mut State, content: &rune::ClipboardContent, tx: &mut Tx| {
                let words = content.text();
                if words.is_empty() {
                    if let Some(sh) = s.tables.get_mut(w) {
                        sh.problem = Some(Problem { error: "Press ⌘V to paste here.".to_string(), stale: false });
                    }
                    let _ = tx;
                    return;
                }
                super::paste(s, w, words);
            });
        }
        Act::OpenLink(href) => {
            close(s);
            crate::files::open_with_system(&href);
        }
        Act::InsertAbove => {
            close(s);
            if let Some(b) = sel {
                super::add_row_at(s, w, b.r0, false, tx);
            }
        }
        Act::InsertBelow => {
            close(s);
            if let Some(b) = sel {
                super::add_row_at(s, w, b.r1 + 1, false, tx);
            }
        }
        Act::MoveUp => {
            close(s);
            if let Some(b) = sel.filter(|b| b.r0 > 0) {
                super::move_rows_to(s, w, b.r0 - 1);
            }
        }
        Act::MoveDown => {
            close(s);
            if let Some(b) = sel {
                super::move_rows_to(s, w, b.r1 + 2);
            }
        }
        Act::Clear => {
            close(s);
            super::clear_picked(s, w);
        }
        Act::DeleteRows => {
            close(s);
            super::drop_picked(s, w);
        }
    }
    let picked = s.tables.get(w).is_some_and(|sh| sh.pick.is_some());
    if picked {
        super::focus_keys(s, w, tx);
    } else {
        tx.focus(w.key("scroll"), false);
    }
}

/// A menu's shell (`.when-menu.tbl-menu`): the pop's grey inside the strong line, square, lifted.
fn menu_shell(wide: f32) -> El {
    col().w(wide).padding(4.0).gap(1.0).fill(pop()).border(1.0, rule()).border_layout().box_shadow(lifted())
}

/// The menus over the window: a bar menu under its button, and a menu at a point.
pub fn layer(s: &State) -> Option<El> {
    let w = if s.viewer.open.is_some() { Where::Big } else { Where::Pane };
    let sh = s.tables.get(w)?;
    if let Some(f) = &sh.float {
        return Some(float_menu(s, sh, f));
    }
    let which = sh.bar?;
    let key = match which {
        Bar::Filter => w.key("filter"),
        Bar::Sort => w.key("sort"),
    };
    let [x, y, _, h] = rune::memo::window_layout(&key)?;
    let body = match which {
        Bar::Filter => filter_menu(sh),
        Bar::Sort => sort_menu(sh),
    };
    let tall = (s.size.1 * 0.7).min(480.0);
    let menu = menu_shell(240.0).key(w.key("bar-menu")).max_h(tall).child(el().w_full().shrink().min_h(0.0).scroll(rune::ScrollSpec { stick_bottom: false }).child(body));
    // A press anywhere else closes it.
    let away = el().absolute(0.0, 0.0).size(s.size.0, s.size.1).on_press(move |s: &mut State| {
        if let Some(sh) = s.tables.get_mut(w) {
            sh.bar = None;
        }
    });
    Some(el().absolute(0.0, 0.0).size(s.size.0, s.size.1).child(away).child(menu.absolute(x, y + h + 8.0).role("menu")))
}

/// The Filter menu (`FilterMenu`).
fn filter_menu(sh: &Sheet) -> El {
    let w = sh.at;
    let table = sh.table();
    let choices = edit::filter_choices(table, 40);
    let many = choices.iter().map(|(_, v)| v.len()).sum::<usize>() > 12;
    let q = js::trim(&sh.filter_find.text).to_lowercase();
    let filtering = sh.picked.values().any(|v| !v.is_empty());
    let mut out = col().gap(1.0);
    if many {
        let field = text_input(&sh.filter_find, sans(12.5, 400.0)).key(w.key("filterfind")).single_line().grow(1.0).color(ink()).placeholder("Find a value", muted()).caret(ink(), rgba(110, 165, 255, 0.3)).on_edit(move |s: &mut State| super::edit_of(s, w, Box_::FilterFind));
        out = out.child(type_row(field));
    }
    if filtering {
        out = out.child(menu_row(label("Show every row", 13.0, 400.0, muted()), false).on_press(move |s: &mut State| super::clear_filters(s, w)));
    }
    for (col, all) in choices {
        let column = &table.columns[col];
        let values: Vec<&String> = all.iter().filter(|v| q.is_empty() || v.to_lowercase().contains(&q)).collect();
        if values.is_empty() {
            continue;
        }
        let mut counts: HashMap<String, usize> = HashMap::new();
        for r in table.rows.iter() {
            *counts.entry(js::trim(r.at(col)).to_lowercase()).or_default() += 1;
        }
        // A column's heading and values are one group, the menu's gap only between groups.
        let mut group = rune::col().w_full().child(head_line(&column.name));
        for v in values {
            let on = sh.picked.get(&col).is_some_and(|p| p.contains(v));
            let tick = if on { el().size(12.0, 12.0).fill(ink()).box_shadow(shadow(0.0, 0.0, 0.0, 2.0, pop(), true)) } else { el().size(12.0, 12.0).border(1.0, rule()).border_layout() };
            let face = if v.is_empty() { label("Empty", 13.0, 400.0, muted()) } else if column.kind == Kind::Tag { chip_in(column.tone(v), v) } else { label(v, 13.0, if on { 600.0 } else { 400.0 }, if on { ink() } else { dim() }) };
            let n = counts.get(&v.to_lowercase()).copied().unwrap_or(0);
            let value = v.clone();
            // The count stands at the row's right end (`.tbl-filter-n { margin-left: auto }`).
            let line = row().w_full().px(9.0).py(6.0).gap(9.0).items_center().radius(6.0).hover(selected()).cursor(Cursor::Pointer).child(tick).child(face).child(el().grow(1.0)).child(text(thousands(n)).font(figures(11.0, 400.0).mono()).color(muted()));
            group = group.child(line.on_press(move |s: &mut State| super::toggle_filter(s, w, col, &value)));
        }
        out = out.child(group);
    }
    out
}

/// The Sort menu.
fn sort_menu(sh: &Sheet) -> El {
    let w = sh.at;
    let mut out = col().gap(1.0);
    let none = sh.sort.is_none();
    out = out.child(menu_row(label("As the file has it", 13.0, if none { 600.0 } else { 400.0 }, if none { ink() } else { dim() }), false).on_press(move |s: &mut State| {
        super::set_sort(s, w, None);
        if let Some(sh) = s.tables.get_mut(w) {
            sh.bar = None;
        }
    }));
    for (i, c) in sh.table().columns.iter().enumerate() {
        let on = sh.sort.filter(|so| so.col == i);
        let mut r = row().grow(1.0).gap(9.0).items_center().child(row().grow(1.0).child(label(&c.name, 13.0, if on.is_some() { 600.0 } else { 400.0 }, if on.is_some() { ink() } else { dim() })));
        if let Some(so) = on {
            r = r.child(text(sort_words(c.kind, so.up)).font(mono(11.0, 14.0)).color(muted()));
        }
        let was = sh.sort.map(|so| so.col);
        out = out.child(menu_row(r, false).on_press(move |s: &mut State| {
            super::sort_by(s, w, i);
            if was != Some(i)
                && let Some(sh) = s.tables.get_mut(w)
            {
                sh.bar = None;
            }
        }));
    }
    out
}

/// A menu at a point (`FloatMenu`): kept eight points inside the window, a press anywhere else
/// closing it.
fn float_menu(s: &State, sh: &Sheet, f: &Float) -> El {
    let w = sh.at;
    let rows = float_rows(s, w);
    let mut menu = menu_shell(220.0).key(w.key("float"));
    for (n, line) in rows.iter().enumerate() {
        if line.sep {
            menu = menu.child(separator());
            continue;
        }
        let lit = f.lit == Some(n);
        let tone = if line.off { dim() } else if lit || line.ticked { ink() } else { dim() };
        let mut words = row().grow(1.0).gap(9.0).items_center();
        if line.head {
            words = words.gap(4.0).child(icon(&BACK_BOLD, 12.0).color(muted())).child(text(line.words.to_uppercase()).font(sans(11.0, 600.0).tracking(0.66)).color(muted()));
            let act = line.act.clone();
            menu = menu.child(row().w_full().px(9.0).pt(7.0).pb(3.0).items_center().hover_group().cursor(Cursor::Pointer).child(words).on_press_tx(move |s: &mut State, tx| press_row(s, w, act.clone(), tx)));
            continue;
        }
        if let Some(kind) = line.mark {
            words = words.child(icon(kind_mark(kind), 13.0).color(muted()));
        }
        words = words.child(row().grow(1.0).child(label(&line.words, 13.0, if line.ticked { 600.0 } else { 400.0 }, tone)));
        if let Some(h) = &line.hint {
            words = words.child(text(h.as_str()).font(mono(11.0, 14.0)).color(muted()));
        }
        let mut r = row().w_full().px(9.0).py(6.0).items_center().radius(6.0).child(words);
        if line.off {
            r = r.opacity(0.4);
        } else {
            let act = line.act.clone();
            r = r.cursor(Cursor::Pointer).on_pointer_move(move |s: &mut State, _| {
                if let Some(f) = s.tables.get_mut(w).and_then(|sh| sh.float.as_mut()) {
                    f.lit = Some(n);
                }
            });
            r = if lit { r.fill(selected()) } else { r.hover(selected()) };
            r = r.on_press_tx(move |s: &mut State, tx| press_row(s, w, act.clone(), tx));
        }
        menu = menu.child(r);
    }
    let [mw, mh] = rune::memo::layout_size(&w.key("float")).unwrap_or([220.0, 200.0]);
    let (ww, wh) = s.size;
    let x = f.x.min(ww - mw - 8.0).max(8.0);
    let y = f.y.min(wh - mh - 8.0).max(8.0);
    let away = el().absolute(0.0, 0.0).size(ww, wh).on_press(move |s: &mut State| {
        if let Some(sh) = s.tables.get_mut(w) {
            sh.float = None;
        }
    }).on_secondary_press(move |s: &mut State, _, _| {
        if let Some(sh) = s.tables.get_mut(w) {
            sh.float = None;
        }
    });
    el().absolute(0.0, 0.0).size(ww, wh).child(away).child(menu.absolute(x, y).role("menu"))
}
