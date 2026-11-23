//! What a person does to a table once it is drawn: cells picked, typed over, copied and pasted
//! the way a spreadsheet does it, rows and columns added, moved and taken away, and the file
//! written back in the shape it came in. A port of `renderer/src/table-edit.ts`, with its tests'
//! cases; `table.rs` is the hands.

use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;

use super::js::{self, re};
use super::table_file::{self as file, Cells, Column, Kind, Row, Table, Tone};

/// A cell, by its place in the view (`r`) and its column.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct At {
    pub r: usize,
    pub c: usize,
}

/// `a` is where the pick began and stays; `b` is the end that moves. Rows are places in the
/// view, not in the file. `rows`: picked by the row numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pick {
    pub a: At,
    pub b: At,
    pub rows: bool,
}

impl Pick {
    pub fn one(at: At) -> Pick {
        Pick { a: at, b: at, rows: false }
    }
}

/// The block a pick covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Block {
    pub r0: usize,
    pub r1: usize,
    pub c0: usize,
    pub c1: usize,
}

/// `boxOf`.
pub fn box_of(p: &Pick) -> Block {
    Block { r0: p.a.r.min(p.b.r), r1: p.a.r.max(p.b.r), c0: p.a.c.min(p.b.c), c1: p.a.c.max(p.b.c) }
}

/// `inBox`.
pub fn in_box(b: &Block, r: usize, c: usize) -> bool {
    r >= b.r0 && r <= b.r1 && c >= b.c0 && c <= b.c1
}

/// `cellsIn`.
pub fn cells_in(b: &Block) -> usize {
    (b.r1 - b.r0 + 1) * (b.c1 - b.c0 + 1)
}

/// The table's edges: the rows in view and the columns drawn (a ranked file's rank is the row
/// number, not a column).
#[derive(Clone, Copy, Debug)]
pub struct Edges {
    pub rows: usize,
    pub first: usize,
    pub last: usize,
}

/// A key that moves the pick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
}

/// Where an arrow, Tab, Home, End or a page key takes the moving end of the pick (`moved`).
pub fn moved(at: At, key: Step, e: Edges, jump: bool, page: usize) -> At {
    let (r, c) = (at.r as i64, at.c as i64);
    let (rows, first, last) = (e.rows as i64, e.first as i64, e.last as i64);
    let page = page as i64;
    let (r, c) = match key {
        Step::Up => (if jump { 0 } else { r - 1 }, c),
        Step::Down => (if jump { rows - 1 } else { r + 1 }, c),
        Step::Left => (r, if jump { first } else { c - 1 }),
        Step::Right => (r, if jump { last } else { c + 1 }),
        Step::Home => (if jump { 0 } else { r }, first),
        Step::End => (if jump { rows - 1 } else { r }, last),
        Step::PageUp => (r - page, c),
        Step::PageDown => (r + page, c),
    };
    At { r: r.clamp(0, (rows - 1).max(0)) as usize, c: c.clamp(first, last.max(first)) as usize }
}

/* ---- The clipboard, as Google Sheets, Excel, Airtable and Numbers write it. ---- */

re!(NEEDS_QUOTES, "[\t\n\r\"]", "");

/// Cells as tab separated lines; a cell holding a tab, a line break or a quote is quoted
/// (`toClipboard`).
pub fn to_clipboard(grid: &[Vec<String>]) -> String {
    grid.iter().map(|r| r.iter().map(|v| if NEEDS_QUOTES.test(v) { format!("\"{}\"", v.replace('"', "\"\"")) } else { v.clone() }).collect::<Vec<_>>().join("\t")).collect::<Vec<_>>().join("\n")
}

/// What was copied, as rows of cells; one value stays one value, whatever it holds
/// (`fromClipboard`).
pub fn from_clipboard(text: &str) -> Vec<Vec<String>> {
    let t = text.replace("\r\n", "\n").replace('\r', "\n");
    let t = t.strip_suffix('\n').unwrap_or(&t);
    if !t.contains('\t') && !t.contains('\n') {
        return vec![vec![t.to_string()]];
    }
    if !t.contains('\t') && !t.starts_with('"') {
        return t.split('\n').map(|l| vec![js::trim(l).to_string()]).collect();
    }
    let rows = file::parse_delimited(t, '\t');
    if rows.is_empty() { vec![vec![String::new()]] } else { rows }
}

/// One cell written: its row in the view and its column.
#[derive(Clone, Debug, PartialEq)]
pub struct Write {
    pub r: usize,
    pub c: usize,
    pub v: String,
}

/// The cells a paste writes, from the top left of the pick (`pasteInto`): one value fills the
/// whole pick; a block that fits the pick a whole number of times is repeated across it.
/// Columns past the last are left off; rows past the end are written, and the caller adds the
/// rows they need.
pub fn paste_into(values: &[Vec<String>], b: Block, last_col: usize) -> Vec<Write> {
    let h = values.len().max(1);
    let w = values.iter().map(Vec::len).max().unwrap_or(0).max(1);
    let (box_h, box_w) = (b.r1 - b.r0 + 1, b.c1 - b.c0 + 1);
    let tiles = (box_h % h == 0 && box_w % w == 0) || (h == 1 && w == 1);
    let rows = if tiles { box_h.max(h) } else { h };
    let cols = if tiles { box_w.max(w) } else { w };
    let mut out = Vec::new();
    for i in 0..rows {
        for j in 0..cols {
            let c = b.c0 + j;
            if c > last_col {
                break;
            }
            let v = values.get(i % h).and_then(|r| r.get(j % w)).cloned().unwrap_or_default();
            out.push(Write { r: b.r0 + i, c, v });
        }
    }
    out
}

/* ---- The table changed: every function returns a new table and leaves the old one. ---- */

/// A tag column learns a value it has not seen, in the colour its word means or the next in
/// turn; none when it has nothing to learn (`learnTag`). A tag deleted and typed again keeps
/// the colour it was given.
pub fn learn_tag(col: &Column, value: &str) -> Option<Column> {
    let v = js::trim(value);
    let key = v.to_lowercase();
    if col.kind != Kind::Tag || v.is_empty() || col.values.iter().any(|x| x.to_lowercase() == key) {
        return None;
    }
    let mut next = col.clone();
    next.values.push(v.to_string());
    let tone = col.tones.get(&key).copied().unwrap_or_else(|| new_tone(col, v));
    next.tones.insert(key, tone);
    Some(next)
}

/// A new tag's colour: green or red when its word means done or lost; else the colour its word
/// means if no other tag has it, or the first in turn no tag has (`newTone`).
pub fn new_tone(col: &Column, value: &str) -> Tone {
    let used: HashSet<Option<Tone>> = col.values.iter().map(|x| col.tones.get(&x.to_lowercase()).copied()).collect();
    let meant: Vec<Tone> = (0..4).map(|p| file::tag_tone(value, p, &col.name)).collect();
    if meant.iter().all(|t| *t == meant[0]) && (meant[0] == Tone::Green || meant[0] == Tone::Rose || !used.contains(&Some(meant[0]))) {
        return meant[0];
    }
    let turns = [Tone::Blue, Tone::Purple, Tone::Amber, Tone::Teal, Tone::Grey];
    turns.iter().copied().find(|t| !used.contains(&Some(*t))).unwrap_or(turns[col.values.len() % turns.len()])
}

/// A cell written by its row in the file.
#[derive(Clone, Debug)]
pub struct Cell {
    pub i: usize,
    pub c: usize,
    pub v: String,
}

/// Cells written, by their row in the file; only the rows touched are made again
/// (`writeCells`).
pub fn write_cells(table: &Table, writes: &[Cell]) -> Table {
    if writes.is_empty() {
        return table.clone();
    }
    let mut rows: Vec<Row> = table.rows.as_ref().clone();
    let mut fresh: HashSet<usize> = HashSet::new();
    let mut columns: Option<Vec<Column>> = None;
    for Cell { i, c, v } in writes {
        let width = columns.as_ref().map_or(table.columns.len(), Vec::len);
        if *i >= rows.len() || *c >= width {
            continue;
        }
        if fresh.insert(*i) {
            rows[*i] = Cells::new(rows[*i].cells.clone());
        }
        let row = Rc::get_mut(&mut rows[*i]).expect("a row made again just now");
        row.cells[*c] = v.clone();
        let col = columns.as_ref().map_or(&table.columns[*c], |cs| &cs[*c]);
        if col.kind == Kind::Tag
            && let Some(learnt) = learn_tag(col, v)
        {
            columns.get_or_insert_with(|| table.columns.as_ref().clone())[*c] = learnt;
        }
    }
    Table { rows: Rc::new(rows), columns: columns.map_or_else(|| table.columns.clone(), Rc::new), ..table.clone() }
}

/// A ranked file whose ranks run 1, 2, 3 in file order keeps them running after rows come and
/// go; ranks in any other order are the file's own and stay (`countsUp`).
fn counts_up(t: &Table) -> bool {
    t.ranked
        && t.rows.iter().enumerate().all(|(i, r)| {
            let v = js::trim(r.at(0));
            v.strip_suffix('.').unwrap_or(v) == (i + 1).to_string()
        })
}

fn renumbered(rows: Vec<Row>) -> Vec<Row> {
    rows.into_iter()
        .enumerate()
        .map(|(i, r)| {
            if r.at(0) == (i + 1).to_string() {
                r
            } else {
                let mut cells = r.cells.clone();
                if cells.is_empty() {
                    cells.push(String::new());
                }
                cells[0] = (i + 1).to_string();
                Cells::new(cells)
            }
        })
        .collect()
}

/// Blank rows put in at `at` in the file (`addRows`).
pub fn add_rows(table: &Table, at: usize, n: usize) -> Table {
    let width = table.columns.len();
    let mut rows: Vec<Row> = table.rows.as_ref().clone();
    let at = at.min(rows.len());
    rows.splice(at..at, (0..n).map(|_| Cells::new(vec![String::new(); width])));
    Table { rows: Rc::new(if counts_up(table) { renumbered(rows) } else { rows }), ..table.clone() }
}

/// Rows taken out of the file (`dropRows`).
pub fn drop_rows(table: &Table, gone: &HashSet<usize>) -> Table {
    let rows: Vec<Row> = table.rows.iter().enumerate().filter(|(i, _)| !gone.contains(i)).map(|(_, r)| r.clone()).collect();
    Table { rows: Rc::new(if counts_up(table) { renumbered(rows) } else { rows }), ..table.clone() }
}

/// The view's order after rows were taken out of the file: theirs gone, the rest renumbered
/// (`orderAfterDrop`).
pub fn order_after_drop(order: &[usize], gone: &HashSet<usize>) -> Vec<usize> {
    let mut sorted: Vec<usize> = gone.iter().copied().collect();
    sorted.sort_unstable();
    order.iter().filter(|i| !gone.contains(i)).map(|i| i - sorted.partition_point(|g| g < i)).collect()
}

/// The view's order after `n` rows were put in the file at `at`, shown from place `place` in the
/// view (`orderAfterAdd`).
pub fn order_after_add(order: &[usize], at: usize, n: usize, place: usize) -> Vec<usize> {
    let mut out: Vec<usize> = order.iter().map(|i| if *i >= at { i + n } else { *i }).collect();
    let place = place.min(out.len());
    out.splice(place..place, (0..n).map(|k| at + k));
    out
}

fn unique_name(columns: &[Column], want: &str, skip: Option<usize>) -> String {
    let base = if js::trim(want).is_empty() { format!("Column {}", columns.len() + 1) } else { js::trim(want).to_string() };
    let taken: HashSet<String> = columns.iter().enumerate().filter(|(i, _)| Some(*i) != skip).map(|(_, c)| c.name.to_lowercase()).collect();
    let mut name = base.clone();
    let mut n = 2;
    while taken.contains(&name.to_lowercase()) {
        name = format!("{base} {n}");
        n += 1;
    }
    name
}

/// An empty text column put in at `at` (`addColumn`).
pub fn add_column(table: &Table, at: usize, want: &str) -> Table {
    let wanted = if want.is_empty() { format!("Column {}", table.columns.len() + 1) } else { want.to_string() };
    let name = unique_name(&table.columns, &wanted, None);
    let mut columns = table.columns.as_ref().clone();
    let at = at.min(columns.len());
    columns.insert(at, Column { name: name.clone(), head: name, kind: Kind::Text, values: Vec::new(), tones: BTreeMap::new(), wrap: false });
    let rows: Vec<Row> = table
        .rows
        .iter()
        .map(|r| {
            let mut cells = r.cells.clone();
            cells.insert(at.min(cells.len()), String::new());
            Cells::new(cells)
        })
        .collect();
    let logo_col = table.logo_col.map(|l| if l >= at { l + 1 } else { l });
    Table { columns: Rc::new(columns), rows: Rc::new(rows), logo_col, ranked: table.ranked }
}

/// A column taken away; never the last one, nor a ranked file's rank (`dropColumn`).
pub fn drop_column(table: &Table, at: usize) -> Table {
    if table.columns.len() <= 1 || (table.ranked && at == 0) {
        return table.clone();
    }
    let columns: Vec<Column> = table.columns.iter().enumerate().filter(|(i, _)| *i != at).map(|(_, c)| c.clone()).collect();
    let rows: Vec<Row> = table.rows.iter().map(|r| Cells::new(r.cells.iter().enumerate().filter(|(i, _)| *i != at).map(|(_, v)| v.clone()).collect())).collect();
    let logo_col = match table.logo_col {
        Some(l) if l == at => None,
        Some(l) if l > at => Some(l - 1),
        other => other,
    };
    Table { columns: Rc::new(columns), rows: Rc::new(rows), logo_col, ranked: table.ranked }
}

/// A column renamed, the file's header with it (`renameColumn`).
pub fn rename_column(table: &Table, at: usize, want: &str) -> Table {
    let Some(col) = table.columns.get(at) else { return table.clone() };
    if js::trim(want).is_empty() || js::trim(want) == col.name {
        return table.clone();
    }
    let name = unique_name(&table.columns, want, Some(at));
    let mut columns = table.columns.as_ref().clone();
    columns[at] = Column { name: name.clone(), head: name, ..col.clone() };
    Table { columns: Rc::new(columns), ..table.clone() }
}

/// A column read as another kind, by the person's choice rather than the guess
/// (`showColumnAs`); as tags it learns the values in it.
pub fn show_column_as(table: &Table, at: usize, kind: Kind) -> Table {
    let Some(col) = table.columns.get(at) else { return table.clone() };
    if col.kind == kind {
        return table.clone();
    }
    let mut next = Column { kind, values: Vec::new(), tones: BTreeMap::new(), ..col.clone() };
    if kind == Kind::Tag {
        for r in table.rows.iter() {
            if let Some(learnt) = learn_tag(&next, r.at(at)) {
                next = learnt;
            }
        }
    }
    let mut columns = table.columns.as_ref().clone();
    columns[at] = next;
    Table { columns: Rc::new(columns), ..table.clone() }
}

fn same_tag(a: &str, b: &str) -> bool {
    js::trim(a).to_lowercase() == js::trim(b).to_lowercase()
}

/// A tag renamed in every cell that holds it; a name another tag has already merges the two
/// (`renameTag`).
pub fn rename_tag(table: &Table, at: usize, from: &str, want: &str) -> Table {
    let Some(col) = table.columns.get(at) else { return table.clone() };
    let to = js::trim(want);
    if col.kind != Kind::Tag || to.is_empty() || to == js::trim(from) {
        return table.clone();
    }
    let into = col.values.iter().find(|v| same_tag(v, to) && !same_tag(v, from)).cloned();
    let written = into.clone().unwrap_or_else(|| to.to_string());
    let rows: Vec<Row> = table
        .rows
        .iter()
        .map(|r| {
            if same_tag(r.at(at), from) {
                let mut cells = r.cells.clone();
                cells[at] = written.clone();
                Cells::new(cells)
            } else {
                r.clone()
            }
        })
        .collect();
    let mut tones = col.tones.clone();
    let was = tones.remove(&js::trim(from).to_lowercase());
    if into.is_none()
        && let Some(was) = was
    {
        tones.insert(to.to_lowercase(), was);
    }
    let values: Vec<String> = if into.is_some() { col.values.iter().filter(|v| !same_tag(v, from)).cloned().collect() } else { col.values.iter().map(|v| if same_tag(v, from) { to.to_string() } else { v.clone() }).collect() };
    let mut columns = table.columns.as_ref().clone();
    columns[at] = Column { values, tones, ..col.clone() };
    Table { rows: Rc::new(rows), columns: Rc::new(columns), ..table.clone() }
}

/// A tag taken out of the column, and out of every cell that holds it (`dropTag`).
pub fn drop_tag(table: &Table, at: usize, value: &str) -> Table {
    let Some(col) = table.columns.get(at) else { return table.clone() };
    if col.kind != Kind::Tag {
        return table.clone();
    }
    let rows: Vec<Row> = table
        .rows
        .iter()
        .map(|r| {
            if same_tag(r.at(at), value) {
                let mut cells = r.cells.clone();
                cells[at] = String::new();
                Cells::new(cells)
            } else {
                r.clone()
            }
        })
        .collect();
    let mut columns = table.columns.as_ref().clone();
    columns[at] = Column { values: col.values.iter().filter(|v| !same_tag(v, value)).cloned().collect(), ..col.clone() };
    Table { rows: Rc::new(rows), columns: Rc::new(columns), ..table.clone() }
}

/* ---- What a person chose that a sheet cannot hold: a column's type, its tags' colours and
whether it wraps. Kept on this computer by column name, beside the widths. ---- */

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Looks {
    #[serde(default)]
    pub kinds: BTreeMap<String, Kind>,
    #[serde(default)]
    pub tones: BTreeMap<String, BTreeMap<String, Tone>>,
    #[serde(default)]
    pub wrap: BTreeMap<String, bool>,
}

/// The table as the person chose to see it (`withLooks`). Nothing chosen leaves it as read.
pub fn with_looks(table: &Table, looks: &Looks) -> Table {
    let mut t = table.clone();
    for i in 0..t.columns.len() {
        let c = &t.columns[i];
        if let Some(kind) = looks.kinds.get(&c.name)
            && *kind != c.kind
            && !(t.ranked && i == 0)
        {
            t = show_column_as(&t, i, *kind);
        }
    }
    let wrap_of = |c: &Column| looks.wrap.get(&c.name).copied().unwrap_or(false);
    if !t.columns.iter().any(|c| (c.kind == Kind::Tag && looks.tones.contains_key(&c.name)) || wrap_of(c) != c.wrap) {
        return t;
    }
    let columns: Vec<Column> = t
        .columns
        .iter()
        .map(|c| {
            let mut next = c.clone();
            if c.kind == Kind::Tag
                && let Some(kept) = looks.tones.get(&c.name)
            {
                next.tones.extend(kept.iter().map(|(k, v)| (k.clone(), *v)));
            }
            next.wrap = wrap_of(c);
            next
        })
        .collect();
    Table { columns: Rc::new(columns), ..t }
}

/// `lookWrap`.
pub fn look_wrap(looks: &Looks, column: &str, on: bool) -> Looks {
    let mut next = looks.clone();
    next.wrap.remove(column);
    if on {
        next.wrap.insert(column.to_string(), true);
    }
    next
}

/// Every tag's colour as drawn now, kept: the guess deals colours out in order of first row,
/// so moving rows would deal them again (`pinTones`).
pub fn pin_tones(table: &Table, looks: &Looks) -> Looks {
    let mut next = looks.clone();
    for c in table.columns.iter().filter(|c| c.kind == Kind::Tag) {
        let kept = next.tones.get(&c.name).cloned().unwrap_or_default();
        let fresh: Vec<(String, Tone)> = c.tones.iter().filter(|(k, _)| !kept.contains_key(*k)).map(|(k, v)| (k.clone(), *v)).collect();
        if !fresh.is_empty() {
            let mut set = kept;
            set.extend(fresh);
            next.tones.insert(c.name.clone(), set);
        }
    }
    next
}

/// `lookKind`.
pub fn look_kind(looks: &Looks, column: &str, kind: Kind) -> Looks {
    let mut next = looks.clone();
    next.kinds.insert(column.to_string(), kind);
    next
}

/// `lookTone`.
pub fn look_tone(looks: &Looks, column: &str, tag: &str, tone: Tone) -> Looks {
    let mut next = looks.clone();
    next.tones.entry(column.to_string()).or_default().insert(js::trim(tag).to_lowercase(), tone);
    next
}

/// The choices made for a column follow it when it is renamed (`lookRenameColumn`).
pub fn look_rename_column(looks: &Looks, from: &str, to: &str) -> Looks {
    if from == to || (!looks.kinds.contains_key(from) && !looks.tones.contains_key(from) && !looks.wrap.contains_key(from)) {
        return looks.clone();
    }
    let mut next = looks.clone();
    if let Some(k) = next.kinds.remove(from) {
        next.kinds.insert(to.to_string(), k);
    }
    if let Some(t) = next.tones.remove(from) {
        next.tones.insert(to.to_string(), t);
    }
    if let Some(w) = next.wrap.remove(from) {
        next.wrap.insert(to.to_string(), w);
    }
    next
}

/// A tag's chosen colour follows it when it is renamed (`lookRenameTag`).
pub fn look_rename_tag(looks: &Looks, column: &str, from: &str, to: &str) -> Looks {
    let key = js::trim(from).to_lowercase();
    let Some(tone) = looks.tones.get(column).and_then(|set| set.get(&key)).copied() else { return looks.clone() };
    let mut next = looks.clone();
    let set = next.tones.get_mut(column).expect("the column's colours");
    set.remove(&key);
    set.insert(js::trim(to).to_lowercase(), tone);
    next
}

/* ---- Rows and columns moved. `to[i]` is where what stood at `i` stands now. ---- */

/// The new order of places when those in `moving` go, in their order, just before place
/// `before`; none when nothing moves (`movedPlaces`).
fn moved_places(length: usize, moving: &[usize], before: usize) -> Option<Vec<usize>> {
    let mut going: Vec<usize> = moving.iter().copied().filter(|i| *i < length).collect();
    going.sort_unstable();
    going.dedup();
    if going.is_empty() {
        return None;
    }
    let set: HashSet<usize> = going.iter().copied().collect();
    let stay: Vec<usize> = (0..length).filter(|i| !set.contains(i)).collect();
    let at = stay.iter().filter(|i| **i < before).count();
    let places: Vec<usize> = stay[..at].iter().chain(going.iter()).chain(stay[at..].iter()).copied().collect();
    (!places.iter().enumerate().all(|(i, p)| *p == i)).then_some(places)
}

fn where_each_went(places: &[usize]) -> Vec<usize> {
    let mut to = places.to_vec();
    for (now, old) in places.iter().enumerate() {
        to[*old] = now;
    }
    to
}

/// Rows of the file put just before the row at `before` (the row count puts them last); they
/// need not stand together (`moveRows`).
pub fn move_rows(table: &Table, rows: &[usize], before: usize) -> (Table, Vec<usize>) {
    let Some(places) = moved_places(table.rows.len(), rows, before) else { return (table.clone(), (0..table.rows.len()).collect()) };
    let list: Vec<Row> = places.iter().map(|i| table.rows[*i].clone()).collect();
    let next = Table { rows: Rc::new(if counts_up(table) { renumbered(list) } else { list }), ..table.clone() };
    (next, where_each_went(&places))
}

/// Columns that stand together, put just before the column at `before`; a ranked file's rank
/// stays first (`moveColumns`).
pub fn move_columns(table: &Table, from: usize, count: usize, before: usize) -> (Table, Vec<usize>) {
    let first = usize::from(table.ranked);
    let places = if from < first { None } else { moved_places(table.columns.len(), &(from..from + count).collect::<Vec<_>>(), before.max(first)) };
    let Some(places) = places else { return (table.clone(), (0..table.columns.len()).collect()) };
    let to = where_each_went(&places);
    let next = Table {
        columns: Rc::new(places.iter().map(|i| table.columns[*i].clone()).collect()),
        rows: Rc::new(table.rows.iter().map(|r| Cells::new(places.iter().map(|i| r.at(*i).to_string()).collect())).collect()),
        logo_col: table.logo_col.map(|l| to[l]),
        ranked: table.ranked,
    };
    (next, to)
}

/// The columns the Filter menu offers, with the values to tick: every tag column, and any other
/// column whose values repeat from a short list, in A to Z order as first written; `''` stands
/// for the empty cells (`filterChoices`).
pub fn filter_choices(table: &Table, most: usize) -> Vec<(usize, Vec<String>)> {
    let mut out = Vec::new();
    for (col, c) in table.columns.iter().enumerate() {
        if table.ranked && col == 0 {
            continue;
        }
        let blank = table.rows.iter().any(|r| js::trim(r.at(col)).is_empty());
        if c.kind == Kind::Tag {
            if !c.values.is_empty() {
                let mut values = c.values.clone();
                if blank {
                    values.push(String::new());
                }
                out.push((col, values));
            }
            continue;
        }
        if matches!(c.kind, Kind::Number | Kind::Name) {
            continue;
        }
        let mut seen: Vec<(String, String)> = Vec::new();
        let mut filled = 0;
        let mut too_many = false;
        for r in table.rows.iter() {
            let v = js::trim(r.at(col));
            if v.is_empty() {
                continue;
            }
            filled += 1;
            let key = v.to_lowercase();
            if !seen.iter().any(|(k, _)| *k == key) {
                seen.push((key, v.to_string()));
            }
            if seen.len() > most {
                too_many = true;
                break;
            }
        }
        if too_many || seen.len() < 2 || seen.len() as f64 > filled as f64 * 0.6 {
            continue;
        }
        let mut values: Vec<String> = seen.into_iter().map(|(_, v)| v).collect();
        values.sort_by(|a, b| file::locale_compare(a, b));
        if blank {
            values.push(String::new());
        }
        out.push((col, values));
    }
    out
}

re!(PLAIN_NUMBER, r"^(-?)(\d+)(\.\d+)?$", "");
re!(GROUPED_BIG, r"\d{4,}|\d{1,3},\d{3}", "");
re!(GROUPED, r"\d,\d{3}", "");

/// A plain number typed into a column of amounts, written the way the column writes them
/// (`likeItsColumn`): "7200" in a column of "$1,200" is "$7,200". Anything else typed is left
/// exactly as it was.
pub fn like_its_column(value: &str, column: &[String]) -> String {
    let v = js::trim(value);
    let Some(m) = PLAIN_NUMBER.exec(v) else { return value.to_string() };
    let filled: Vec<&str> = column.iter().map(|x| js::trim(x)).filter(|x| x.chars().any(|c| c.is_ascii_digit())).collect();
    if filled.is_empty() {
        return value.to_string();
    }
    let most = |test: &dyn Fn(&str) -> bool| filled.iter().filter(|x| test(x)).count() as f64 / filled.len() as f64 >= 0.8;
    let sign = ["$", "€", "£"].into_iter().find(|c| most(&|x: &str| x.strip_prefix('-').unwrap_or(x).starts_with(c))).unwrap_or("");
    let big: Vec<&&str> = filled.iter().filter(|x| GROUPED_BIG.test(x)).collect();
    let grouped = !big.is_empty() && big.iter().filter(|x| GROUPED.test(x)).count() as f64 / big.len() as f64 >= 0.8;
    let digits = js::group(&m, 2);
    let whole = if grouped {
        let mut out = String::new();
        for (k, ch) in digits.chars().enumerate() {
            if k > 0 && (digits.len() - k) % 3 == 0 {
                out.push(',');
            }
            out.push(ch);
        }
        out
    } else {
        digits.to_string()
    };
    format!("{}{sign}{whole}{}", js::group(&m, 1), js::group(&m, 3))
}

/* ---- Back to the file. ---- */

/// How the file was written, so a save writes it the same way (`Shape`).
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    pub sep: char,
    pub eol: &'static str,
    pub markdown: bool,
    pub bom: bool,
}

/// `shapeOf`.
pub fn shape_of(text: &str, path: &str) -> Shape {
    Shape { sep: file::sep_for(text, path), eol: if text.contains("\r\n") { "\r\n" } else { "\n" }, markdown: file::is_markdown_table(text), bom: text.starts_with('\u{feff}') }
}

re!(BREAKS, "[\"\r\n]", "");
re!(HEAD_SEPS, "[,;\t]", "");

// The separator is read off the header line, so a header holding any of the three is quoted or
// a semicolon file would come back as a comma one.
fn csv_cell(v: &str, sep: char, head: bool) -> String {
    if v.contains(sep) || BREAKS.test(v) || v != js::trim(v) || (head && HEAD_SEPS.test(v)) { format!("\"{}\"", v.replace('"', "\"\"")) } else { v.to_string() }
}

fn md_cell(v: &str) -> String {
    v.replace("\r\n", " ").replace('\n', " ").replace('|', "\\|")
}

/// The table as the text of its file: the headers as the file had them, every row, the same
/// separator and line ends (`writeTable`).
pub fn write_table(table: &Table, shape: &Shape) -> String {
    let head: Vec<String> = table.columns.iter().map(|c| c.head.clone()).collect();
    let lines: Vec<String> = if shape.markdown {
        let line = |cells: &[String]| format!("| {} |", cells.iter().map(|c| md_cell(c)).collect::<Vec<_>>().join(" | "));
        let rule = format!("|{}|", head.iter().map(|_| " --- ").collect::<Vec<_>>().join("|"));
        std::iter::once(line(&head)).chain(std::iter::once(rule)).chain(table.rows.iter().map(|r| line(&r.cells))).collect()
    } else {
        let sep = shape.sep.to_string();
        std::iter::once(head.iter().map(|v| csv_cell(v, shape.sep, true)).collect::<Vec<_>>().join(&sep)).chain(table.rows.iter().map(|r| r.cells.iter().map(|v| csv_cell(v, shape.sep, false)).collect::<Vec<_>>().join(&sep))).collect()
    };
    format!("{}{}{}", if shape.bom { "\u{feff}" } else { "" }, lines.join(shape.eol), shape.eol)
}

/// Only a change to the cells or the headers is a change to the file; a colour or a type is not
/// (`fileChanged`).
pub fn file_changed(a: &Table, b: &Table) -> bool {
    !Rc::ptr_eq(&a.rows, &b.rows) || a.columns.len() != b.columns.len() || a.columns.iter().zip(b.columns.iter()).any(|(x, y)| x.head != y.head)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::table_file::read_table;

    const LEADS: &str = "Company,Status,Note,Website\r\nRamp,Meeting,Asked for a demo,ramp.com\r\nMercury,Replied,\"Replied, wants pricing\",mercury.com\r\nDeel,New,\"Intro through an \"\"investor\"\"\",deel.com\r\nPlaid,New,\"Two lines\nof note\",plaid.com\r\n";
    fn back(text: &str, path: &str) -> String {
        write_table(&read_table(text, path), &shape_of(text, path))
    }
    fn rows(t: &Table) -> Vec<Vec<String>> {
        t.rows.iter().map(|r| r.cells.clone()).collect()
    }
    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }
    fn firsts(t: &Table) -> Vec<String> {
        t.rows.iter().map(|r| r.at(0).to_string()).collect()
    }

    // tests/a-table-is-edited-like-a-spreadsheet.test.mts
    #[test]
    fn a_save_writes_the_file_back_the_way_it_came() {
        assert_eq!(back(LEADS, "leads.csv"), LEADS);
        let tsv = "Name\tStatus\nAda\tNew\nBo\tWon\n";
        assert_eq!(back(tsv, "x.tsv"), tsv);
        let semi = "\"Company, Inc\";Status\nAcme;New\nBeta;Won\n";
        assert_eq!(back(semi, "leads.csv"), semi);
        let bom = "\u{feff}Name,Status\nAda,New\nBo,Won\n";
        assert_eq!(back(bom, "leads.csv"), bom);
        let md = "| Name | Status |\n|---|---|\n| Ada | New |\n| Bo \\| Co | Won |\n";
        let out = back(md, "leads.csv");
        assert_eq!(out.split('\n').next(), Some("| Name | Status |"));
        assert_eq!(rows(&read_table(&out, "x.csv")), [s(&["Ada", "New"]), s(&["Bo | Co", "Won"])]);
        let dup = "Email,Email\na@x.co,b@x.co\nc@x.co,d@x.co\n";
        assert_eq!(read_table(dup, "x.csv").columns.iter().map(|c| c.name.clone()).collect::<Vec<_>>(), ["Email", "Email 2"]);
        assert_eq!(back(dup, "leads.csv"), dup);
        let t = read_table(LEADS, "leads.csv");
        let out = write_table(&write_cells(&t, &[Cell { i: 0, c: 2, v: "Said \"soon\", then left".into() }, Cell { i: 1, c: 2, v: "plain".into() }]), &shape_of(LEADS, "leads.csv"));
        assert!(out.contains("Ramp,Meeting,\"Said \"\"soon\"\", then left\",ramp.com"));
        assert!(out.contains("Mercury,Replied,plain,mercury.com"));
        assert_eq!(read_table(&out, "leads.csv").cell(0, 2), "Said \"soon\", then left");
    }

    #[test]
    fn a_cell_typed_over() {
        let t = read_table(LEADS, "leads.csv");
        let next = write_cells(&t, &[Cell { i: 1, c: 1, v: "Won".into() }]);
        assert_eq!(next.cell(1, 1), "Won");
        assert_eq!(t.cell(1, 1), "Replied");
        assert!(Rc::ptr_eq(&next.rows[0], &t.rows[0]));
        let next = write_cells(&t, &[Cell { i: 0, c: 1, v: "Lost".into() }]);
        assert!(next.columns[1].values.contains(&"Lost".to_string()));
        assert_eq!(next.columns[1].tones.get("lost"), Some(&Tone::Rose));
        assert!(learn_tag(&t.columns[1], "meeting").is_none());
        assert!(learn_tag(&t.columns[1], "  ").is_none());
        assert_eq!(rows(&write_cells(&t, &[Cell { i: 0, c: 9, v: "x".into() }, Cell { i: 99, c: 0, v: "x".into() }])), rows(&t));
    }

    #[test]
    fn a_pick_moves_the_way_arrows_move_in_a_spreadsheet() {
        let e = Edges { rows: 5, first: 0, last: 3 };
        let at = |r, c| At { r, c };
        assert_eq!(moved(at(0, 0), Step::Down, e, false, 10), at(1, 0));
        assert_eq!(moved(at(0, 0), Step::Up, e, false, 10), at(0, 0));
        assert_eq!(moved(at(4, 3), Step::Right, e, false, 10), at(4, 3));
        assert_eq!(moved(at(2, 1), Step::Down, e, true, 10), at(4, 1));
        assert_eq!(moved(at(2, 1), Step::Left, e, true, 10), at(2, 0));
        assert_eq!(moved(at(2, 1), Step::End, e, true, 10), at(4, 3));
        let ranked = Edges { rows: 3, first: 1, last: 2 };
        assert_eq!(moved(at(0, 1), Step::Left, ranked, false, 10), at(0, 1));
        assert_eq!(moved(at(0, 2), Step::Home, ranked, false, 10), at(0, 1));
        assert_eq!(moved(at(0, 0), Step::PageDown, e, false, 3), at(3, 0));
        assert_eq!(moved(at(1, 0), Step::PageUp, e, false, 3), at(0, 0));
        let b = box_of(&Pick { a: at(3, 2), b: at(1, 0), rows: false });
        assert_eq!(b, Block { r0: 1, r1: 3, c0: 0, c1: 2 });
        assert!(in_box(&b, 2, 1) && !in_box(&b, 0, 1) && !in_box(&b, 2, 3));
    }

    #[test]
    fn copy_and_paste_speak_the_spreadsheet_clipboard() {
        assert_eq!(to_clipboard(&[s(&["a", "b"]), s(&["c", "d"])]), "a\tb\nc\td");
        assert_eq!(to_clipboard(&[s(&["two\nlines", "say \"hi\""])]), "\"two\nlines\"\t\"say \"\"hi\"\"\"");
        let grid = vec![s(&["two\nlines", "say \"hi\""]), s(&["", "x"])];
        assert_eq!(from_clipboard(&to_clipboard(&grid)), grid);
        assert_eq!(from_clipboard("Replied, wants pricing"), [s(&["Replied, wants pricing"])]);
        assert_eq!(from_clipboard("Won\n"), [s(&["Won"])]);
        assert_eq!(from_clipboard("Ramp\r\nMercury\r\nDeel"), [s(&["Ramp"]), s(&["Mercury"]), s(&["Deel"])]);
        let w = |r, c, v: &str| Write { r, c, v: v.into() };
        assert_eq!(paste_into(&[s(&["Won"])], Block { r0: 1, r1: 2, c0: 1, c1: 2 }, 5), [w(1, 1, "Won"), w(1, 2, "Won"), w(2, 1, "Won"), w(2, 2, "Won")]);
        assert_eq!(paste_into(&[s(&["a", "b"]), s(&["c", "d"])], Block { r0: 3, r1: 3, c0: 0, c1: 0 }, 5), [w(3, 0, "a"), w(3, 1, "b"), w(4, 0, "c"), w(4, 1, "d")]);
        assert_eq!(paste_into(&[s(&["a"]), s(&["b"])], Block { r0: 0, r1: 3, c0: 0, c1: 0 }, 5).iter().map(|x| x.v.clone()).collect::<Vec<_>>(), ["a", "b", "a", "b"]);
        assert_eq!(paste_into(&[s(&["a", "b", "c"])], Block { r0: 0, r1: 0, c0: 2, c1: 2 }, 3).iter().map(|x| x.c).collect::<Vec<_>>(), [2, 3]);
    }

    #[test]
    fn rows_and_columns_added_and_taken_away() {
        let t = read_table(LEADS, "leads.csv");
        let next = add_rows(&t, 1, 1);
        assert_eq!(next.rows.len(), 5);
        assert_eq!(next.rows[1].cells, s(&["", "", "", ""]));
        assert!(Rc::ptr_eq(&next.rows[2], &t.rows[1]));
        let ranked = read_table("#,Candidate,Stage\n1,Ada,Offer\n2,Bo,Screening\n3,Cy,Applied\n", "x.csv");
        assert_eq!(add_rows(&ranked, 1, 1).rows.iter().map(|r| format!("{}{}", r.at(0), r.at(1))).collect::<Vec<_>>(), ["1Ada", "2", "3Bo", "4Cy"]);
        assert_eq!(drop_rows(&ranked, &HashSet::from([0])).rows.iter().map(|r| format!("{}{}", r.at(0), r.at(1))).collect::<Vec<_>>(), ["1Bo", "2Cy"]);
        let own = read_table("#,Candidate\n3,Ada\n1,Bo\n2,Cy\n", "x.csv");
        assert_eq!(firsts(&add_rows(&own, 3, 1)), ["3", "1", "2", ""]);
        assert_eq!(order_after_drop(&[3, 0, 2, 1], &HashSet::from([0, 2])), [1, 0]);
        assert_eq!(firsts(&drop_rows(&t, &HashSet::from([0, 2]))), ["Mercury", "Plaid"]);
        assert_eq!(order_after_add(&[3, 0, 2, 1], 2, 1, 1), [4, 2, 0, 3, 1]);
        let next = add_column(&t, 1, "");
        assert_eq!(next.columns[1].name, "Column 5");
        assert_eq!(next.rows[0].cells, s(&["Ramp", "", "Meeting", "Asked for a demo", "ramp.com"]));
        assert_eq!(next.logo_col, t.logo_col.map(|l| l + 1));
        assert_eq!(drop_column(&t, 2).columns.iter().map(|c| c.name.clone()).collect::<Vec<_>>(), ["Company", "Status", "Website"]);
        let one = read_table("Name\nAda\n", "x.csv");
        assert!(drop_column(&one, 0).same(&one));
        let ranked = read_table("#,Name,Fit\n1,Ada,High\n2,Bo,Low\n", "x.csv");
        assert!(drop_column(&ranked, 0).same(&ranked));
        assert_eq!(rename_column(&t, 2, "Status").columns[2].name, "Status 2");
        assert_eq!(write_table(&rename_column(&t, 2, "Why"), &shape_of(LEADS, "x.csv")).split("\r\n").next(), Some("Company,Status,Why,Website"));
        let tags = show_column_as(&t, 3, Kind::Tag);
        assert_eq!(tags.columns[3].values, s(&["ramp.com", "mercury.com", "deel.com", "plaid.com"]));
        assert!(show_column_as(&tags, 3, Kind::Text).columns[3].values.is_empty());
    }

    #[test]
    fn the_filter_menu_offers_any_column_with_a_short_list_of_values() {
        let cities: String = ["New York", "London", "New York", "Berlin", "London", "new york"].iter().enumerate().map(|(i, c)| format!("Co {i},{},{c},{},note {i}\n", if i % 2 == 1 { "New" } else { "Won" }, i * 10)).collect();
        let t = read_table(&format!("Company,Status,Based in,Employees,Note\n{cities}"), "x.csv");
        let offered: BTreeMap<String, Vec<String>> = filter_choices(&t, 40).into_iter().map(|(c, v)| (t.columns[c].name.clone(), v)).collect();
        assert_eq!(offered["Status"], s(&["Won", "New"]));
        assert_eq!(offered["Based in"], s(&["Berlin", "London", "New York"]));
        assert!(!offered.contains_key("Company") && !offered.contains_key("Employees") && !offered.contains_key("Note"));
        let gaps = read_table("Name,Status\nAda,Won\nBo,\nCy,Won\nDi,New\nEd,\n", "x.csv");
        let status = filter_choices(&gaps, 40).into_iter().find(|(c, _)| *c == 1).unwrap();
        assert_eq!(status.1, s(&["Won", "New", ""]));
        let found = file::filter_rows(&gaps, "", &BTreeMap::from([(1, vec![String::new()])]));
        assert_eq!(found.iter().map(|i| gaps.cell(*i, 0).to_string()).collect::<Vec<_>>(), ["Bo", "Ed"]);
        let many: String = (0..120).map(|i| format!("P{i},Town {}\n", i % 60)).collect();
        let many = read_table(&format!("Name,City\n{many}"), "x.csv");
        assert!(filter_choices(&many, 40).is_empty());
        assert_eq!(filter_choices(&many, 80).iter().map(|(c, _)| *c).collect::<Vec<_>>(), [1]);
        // As `localeCompare` orders them, figures one by one.
        assert_eq!(filter_choices(&many, 80)[0].1[..3], s(&["Town 0", "Town 1", "Town 10"]));
    }

    #[test]
    fn a_number_typed_into_a_column_of_amounts_is_written_the_columns_way() {
        let mrr = s(&["$1,200", "$3,450", "$980", "$12,000"]);
        assert_eq!(like_its_column("7200", &mrr), "$7,200");
        assert_eq!(like_its_column("1234567.5", &mrr), "$1,234,567.5");
        assert_eq!(like_its_column("-40", &mrr), "-$40");
        assert_eq!(like_its_column("$7,200", &mrr), "$7,200");
        assert_eq!(like_its_column("7.2k", &mrr), "7.2k");
        assert_eq!(like_its_column("n/a", &mrr), "n/a");
        assert_eq!(like_its_column("7200", &s(&["12", "40", "1200", "4000"])), "7200");
    }

    // tests/a-tables-tags-types-rows-and-columns-can-all-be-changed.test.mts
    const PLAN: &str = "Goal,Launch,Status\nStable cloud,2026-11-12,At risk\nPhone app,2026-11-12,Hello\nTesters,2026-11-17,hello\nVideo,2026-11-17,Done\n";
    fn plan() -> Table {
        read_table(PLAN, "plan.csv")
    }
    fn statuses(t: &Table) -> Vec<String> {
        t.rows.iter().map(|r| r.at(2).to_string()).collect()
    }

    #[test]
    fn a_tag_is_renamed_merged_or_deleted_in_every_cell_that_holds_it() {
        let t = plan();
        let tone = t.columns[2].tones["hello"];
        let next = rename_tag(&t, 2, "Hello", "Blocked on Apple");
        assert_eq!(statuses(&next), s(&["At risk", "Blocked on Apple", "Blocked on Apple", "Done"]));
        assert_eq!(next.columns[2].values, s(&["At risk", "Blocked on Apple", "Done"]));
        assert_eq!(next.columns[2].tones["blocked on apple"], tone);
        assert!(!next.columns[2].tones.contains_key("hello"));
        assert_eq!(statuses(&t), s(&["At risk", "Hello", "hello", "Done"]));
        let next = rename_tag(&plan(), 2, "Hello", "done");
        assert_eq!(statuses(&next), s(&["At risk", "Done", "Done", "Done"]));
        assert_eq!(next.columns[2].values, s(&["At risk", "Done"]));
        let next = rename_tag(&plan(), 2, "Hello", "HELLO");
        assert_eq!(statuses(&next), s(&["At risk", "HELLO", "HELLO", "Done"]));
        assert_eq!(next.columns[2].values, s(&["At risk", "HELLO", "Done"]));
        assert!(rename_tag(&t, 2, "Hello", "  ").same(&t));
        assert!(rename_tag(&t, 2, "Hello", "Hello").same(&t));
        assert!(rename_tag(&t, 0, "Video", "Film").same(&t));
        let next = drop_tag(&plan(), 2, "hello");
        assert_eq!(statuses(&next), s(&["At risk", "", "", "Done"]));
        assert_eq!(next.columns[2].values, s(&["At risk", "Done"]));
        assert!(write_table(&next, &shape_of(PLAN, "plan.csv")).contains("Phone app,2026-11-12,\n"));
        let t = with_looks(&plan(), &look_tone(&Looks::default(), "Status", "Hello", Tone::Teal));
        let gone = drop_tag(&t, 2, "Hello");
        let back = write_cells(&gone, &[Cell { i: 1, c: 2, v: "Hello".into() }]);
        assert!(back.columns[2].values.contains(&"Hello".to_string()));
        assert_eq!(back.columns[2].tones["hello"], Tone::Teal);
        let mut col = plan().columns[2].clone();
        col.tones.insert("later".into(), Tone::Green);
        let learnt = learn_tag(&col, "Later").unwrap();
        assert!(learnt.values.contains(&"Later".to_string()));
        assert_eq!(learnt.tones["later"], Tone::Green);
    }

    #[test]
    fn a_colour_and_a_type_the_person_picked_come_back() {
        let t = with_looks(&plan(), &look_tone(&Looks::default(), "Status", "At risk", Tone::Amber));
        assert_eq!(t.columns[2].tones["at risk"], Tone::Amber);
        assert_eq!(t.columns[2].tones["done"], Tone::Green);
        assert_eq!(plan().columns[1].kind, Kind::Date);
        let as_tags = with_looks(&plan(), &look_kind(&Looks::default(), "Launch", Kind::Tag));
        assert_eq!(as_tags.columns[1].kind, Kind::Tag);
        assert_eq!(as_tags.columns[1].values, s(&["2026-11-12", "2026-11-17"]));
        assert_eq!(with_looks(&plan(), &look_kind(&Looks::default(), "Status", Kind::Text)).columns[2].kind, Kind::Text);
        let words = read_table("Goal,When\nA,after the cloud\nB,once testers sign\n", "x.csv");
        assert_eq!(words.columns[1].kind, Kind::Text);
        assert_eq!(with_looks(&words, &look_kind(&Looks::default(), "When", Kind::Date)).columns[1].kind, Kind::Date);
        let looks = look_tone(&look_kind(&Looks::default(), "Status", Kind::Tag), "Status", "Hello", Tone::Teal);
        let moved = look_rename_column(&looks, "Status", "State");
        assert_eq!(moved.kinds, BTreeMap::from([("State".to_string(), Kind::Tag)]));
        assert_eq!(moved.tones, BTreeMap::from([("State".to_string(), BTreeMap::from([("hello".to_string(), Tone::Teal)]))]));
        assert_eq!(look_rename_column(&looks, "Goal", "Aim"), looks);
        assert_eq!(look_rename_tag(&moved, "State", "Hello", "Waiting").tones["State"], BTreeMap::from([("waiting".to_string(), Tone::Teal)]));
        assert_eq!(look_rename_tag(&moved, "State", "Done", "Shipped"), moved);
        let src = "Company,Owner\nA,Jordan\nB,Sam\nC,Jordan\nD,Sam\n";
        let t = read_table(src, "l.csv");
        let before = t.columns[1].tones.clone();
        let (moved, _) = move_rows(&t, &[1, 3], 0);
        let fresh = read_table(&write_table(&moved, &shape_of(src, "l.csv")), "l.csv");
        assert_ne!(fresh.columns[1].tones, before);
        assert_eq!(with_looks(&fresh, &pin_tones(&t, &Looks::default())).columns[1].tones, before);
        let chosen = look_tone(&Looks::default(), "Owner", "Sam", Tone::Teal);
        assert_eq!(pin_tones(&t, &chosen).tones["Owner"]["sam"], Tone::Teal);
        assert_eq!(pin_tones(&read_table("A,B\n1,2\n", "x.csv"), &Looks::default()), Looks::default());
        let t = read_table("Lead,Stage\nA,Intro call\nB,Scoping\nC,Intro call\n", "x.csv");
        let col = &t.columns[1];
        let used: HashSet<Tone> = col.tones.values().copied().collect();
        assert!(!used.contains(&new_tone(col, "Negotiating")));
        assert_eq!(new_tone(col, "Won"), Tone::Green);
        let leads = read_table("Lead,Status\nA,Replied\nB,Won\nC,Replied\n", "x.csv");
        let leads = &leads.columns[1];
        assert_eq!(leads.tones["replied"], Tone::Purple);
        assert_ne!(new_tone(leads, "Negotiating"), Tone::Purple);
        assert_eq!(new_tone(leads, "Lost"), Tone::Rose);
        let t = read_table("Goal,Note\nA,a long note\nB,another\n", "x.csv");
        let on = look_wrap(&Looks::default(), "Note", true);
        assert!(with_looks(&t, &on).columns[1].wrap);
        assert!(!with_looks(&t, &on).columns[0].wrap);
        assert_eq!(look_rename_column(&on, "Note", "Notes").wrap, BTreeMap::from([("Notes".to_string(), true)]));
        let off = look_wrap(&on, "Note", false);
        assert!(off.wrap.is_empty());
        assert!(!with_looks(&with_looks(&t, &on), &off).columns[1].wrap);
        let t = plan();
        assert!(with_looks(&t, &Looks::default()).same(&t));
    }

    #[test]
    fn rows_and_columns_move_and_the_file_is_written_in_the_new_order() {
        let t = plan();
        let names = |x: &Table| firsts(x);
        assert_eq!(names(&move_rows(&t, &[2], 0).0), s(&["Testers", "Stable cloud", "Phone app", "Video"]));
        assert_eq!(names(&move_rows(&t, &[0], 2).0), s(&["Phone app", "Stable cloud", "Testers", "Video"]));
        assert_eq!(names(&move_rows(&t, &[0, 1], 4).0), s(&["Testers", "Video", "Stable cloud", "Phone app"]));
        assert_eq!(move_rows(&t, &[0, 1], 4).1, [2, 3, 0, 1]);
        assert_eq!(names(&move_rows(&plan(), &[3, 1], 0).0), s(&["Phone app", "Video", "Stable cloud", "Testers"]));
        assert!(move_rows(&t, &[1, 2], 1).0.same(&t));
        assert!(move_rows(&t, &[1, 2], 2).0.same(&t));
        assert!(move_rows(&t, &[1, 2], 3).0.same(&t));
        assert!(move_rows(&t, &[], 0).0.same(&t));
        let r = read_table("#,Name\n1,Ada\n2,Bo\n3,Cy\n", "r.csv");
        assert_eq!(rows(&move_rows(&r, &[2], 0).0), [s(&["1", "Cy"]), s(&["2", "Ada"]), s(&["3", "Bo"])]);
        let src = "Company,Status,Website\nRamp,New,ramp.com\nDeel,Won,deel.com\n";
        let t = read_table(src, "l.csv");
        assert_eq!(t.logo_col, Some(2));
        let (next, to) = move_columns(&t, 2, 1, 1);
        assert_eq!(next.columns.iter().map(|c| c.name.clone()).collect::<Vec<_>>(), ["Company", "Website", "Status"]);
        assert_eq!(next.logo_col, Some(1));
        assert_eq!(to, [0, 2, 1]);
        assert_eq!(write_table(&next, &shape_of(src, "l.csv")), "Company,Website,Status\nRamp,ramp.com,New\nDeel,deel.com,Won\n");
        assert_eq!(move_columns(&t, 0, 1, 3).0.columns.iter().map(|c| c.name.clone()).collect::<Vec<_>>(), ["Status", "Website", "Company"]);
        let r = read_table("#,Name,Score\n1,Ada,3\n2,Bo,4\n", "r.csv");
        assert_eq!(move_columns(&r, 2, 1, 0).0.columns.iter().map(|c| c.name.clone()).collect::<Vec<_>>(), ["#", "Score", "Name"]);
        assert!(move_columns(&r, 0, 1, 3).0.same(&r));
        assert!(move_columns(&r, 1, 1, 2).0.same(&r));
    }

    #[test]
    fn only_a_change_to_cells_or_headers_is_a_change_to_the_file() {
        let t = plan();
        assert!(!file_changed(&t, &with_looks(&t, &look_tone(&Looks::default(), "Status", "Done", Tone::Teal))));
        assert!(file_changed(&t, &write_cells(&t, &[Cell { i: 0, c: 0, v: "x".into() }])));
        assert!(file_changed(&t, &rename_column(&t, 0, "Aim")));
    }
}
