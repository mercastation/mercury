//! A sheet (`.csv`, `.tsv`) opened beside a conversation or big over the window, worked as the
//! web window's spreadsheet is (`TableView` in `renderer/src/components/TableView.tsx`): a press
//! picks a cell, a drag or Shift a block, a row number rows and a header its column; Enter, F2
//! or a double press edits; a tag or a date cell opens its list or its calendar; Space opens the
//! whole row; search, Filter and Sort narrow and order the rows; each column has its menu and
//! each cell its own; undo, the spreadsheet clipboard; and every change is written back to the
//! file in the shape it came in, through the app (`zero:write-doc`), 400 ms after the last.
//!
//! What a sheet is and how it changes are the web's own rules, ported with their cases
//! (`rules/table_file.rs`, `rules/table_edit.rs`); this is the hands, and `table/draw.rs` the look.
//! Each check of the port is a line of `docs/parity-tables.md`.

mod draw;

pub use draw::{layer, view};

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use rune::edit::Key;
use rune::{KeyEvent, ScrollPos, TextEdit, Tx};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::State;
use crate::rules::js;
use crate::rules::table_edit::{self as edit, At, Block, Edges, Looks, Pick, Shape, Step};
use crate::rules::table_file::{self as file, Kind, Sort, Table, Tone};

/// Where a sheet is open: in the pane beside a conversation, or big over the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Where {
    Pane,
    Big,
}

impl Where {
    /// A key of one of this sheet's elements: the two can be open at once, the big one over the
    /// pane.
    pub fn key(self, what: &str) -> String {
        format!("tbl-{what}-{}", self.tail())
    }
    fn tail(self) -> &'static str {
        match self {
            Where::Pane => "pane",
            Where::Big => "big",
        }
    }
    /// How wide long words make a column (`columnWidths`' cap: 300 full screen, 220 beside).
    fn cap(self) -> f32 {
        match self {
            Where::Pane => 220.0,
            Where::Big => 300.0,
        }
    }
}

/// `.tbl-td`'s height; with a column wrapped every row is three lines tall.
pub const ROW_H: f32 = 46.0;
pub const ROW_WRAPPED: f32 = 86.0;
/// `.tbl-rn`'s width, wider from a thousand rows, for a four-figure number.
pub const RN_W: f32 = 44.0;
pub const RN_W_WIDE: f32 = 58.0;
/// The header row, its lines over and under it included.
pub const HEAD_H: f32 = 41.0;
/// The narrowest a hand can make a column.
pub const MIN_W: f32 = 64.0;
/// A change is written this long after the last one.
pub const SAVE_MS: f64 = 400.0;
/// How many steps undo walks.
const UNDO_DEPTH: usize = 200;

/// What one sheet keeps between openings, by `product/path` (`agentbox.table-view`,
/// `agentbox.table-widths`, `agentbox.table-looks`): the sort and the filters by column name, so
/// a column added or taken away does not move one onto another, the widths a hand set, and what
/// was chosen that a sheet cannot hold.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Kept {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<KeptSort>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub picked: BTreeMap<String, Vec<String>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub widths: BTreeMap<String, f32>,
    #[serde(skip_serializing_if = "no_looks")]
    pub looks: Looks,
}

fn no_looks(l: &Looks) -> bool {
    *l == Looks::default()
}

/// A sort kept by its column's name and its direction, as the web keeps it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KeptSort {
    pub name: String,
    pub dir: String,
}

/// One step of the table, which undo walks back to (`Doc`). `filter`: the change this step made
/// to the filter (a tag it renamed or deleted was filtered on), before and after.
#[derive(Clone)]
struct Doc {
    table: Table,
    order: Rc<Vec<usize>>,
    looks: Looks,
    filter: Option<(Filters, Filters)>,
}

type Filters = BTreeMap<usize, Vec<String>>;

/// A cell being typed into (`Editing`): its words, whether a key opened it, and the list or
/// calendar it pops when it is a tag or a date.
pub struct Editing {
    pub r: usize,
    pub c: usize,
    pub draft: TextEdit,
    pub typed: bool,
    pub pop: Option<Pop>,
    /// The box has had the keys; losing them now ends the edit, as the web's `onBlur` does.
    held: bool,
}

pub enum Pop {
    Tag(TagPick),
    Date(DatePick),
}

/// A tag cell's list (`TagPicker`): what is typed, the lit row (`None` stands for the cell's own
/// tag, found again once a rename has drawn the list again), and one tag's page while it is open.
pub struct TagPick {
    pub find: TextEdit,
    pub hi: Option<usize>,
    pub page: Option<(String, TextEdit)>,
    held: bool,
}

/// A date cell's calendar (`DatePicker`): what is typed, and the month shown.
pub struct DatePick {
    pub words: TextEdit,
    pub month: (i64, i64),
    held: bool,
}

/// A menu at a point (`FloatMenu`): a header's or a cell's, at window points; `page_type` the
/// column menu's page of types; `lit` the row the keys are on.
pub struct Float {
    pub x: f32,
    pub y: f32,
    pub col: Option<usize>,
    pub page_type: bool,
    pub lit: Option<usize>,
}

/// One of the bar's two menus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bar {
    Filter,
    Sort,
}

/// Where a row or a column picked up would land (`Drop`): the gap, by its place, and where to
/// draw it, in the grid's points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dropping {
    pub cols: bool,
    pub place: usize,
    pub at: f32,
}

/// What a press on the grid is doing while the button is down (`drag`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Dragging {
    Cells,
    Rows,
    Cols,
    MoveRows,
    MoveCols,
}

/// A save that did not happen: the reason, and whether it is because the file moved on.
#[derive(Clone, Debug, PartialEq)]
pub struct Problem {
    pub error: String,
    pub stale: bool,
}

/// One field of the row opened whole while it is being written or picked in (`Field`).
pub struct Field {
    pub c: usize,
    pub draft: TextEdit,
    /// Its tag list or calendar, open under it.
    pub pick: Option<Pop>,
    held: bool,
}

/// One sheet open.
pub struct Sheet {
    pub at: Where,
    pub product: String,
    /// The path as it was pressed, which the app finds again to write it (`src`).
    pub src: String,
    /// The thread the big view was opened from, which a save names too.
    pub thread: Option<String>,
    /// Where it is on this Mac, and its name.
    pub found: String,
    pub name: String,
    /// What it is kept under (`storeKey`: `product/path`).
    store: String,
    /// Its words as read, and how it is written back.
    text: Rc<str>,
    shape: Shape,
    /// The file's time on disk when it was read, or the one the last save left (`useTableSave`).
    pub mtime: f64,
    /// Whether it can be changed here (`onSave` given).
    pub editable: bool,
    /// Each column's width from the file as read, by name, and the column of words that takes
    /// what is spare (`natural`, measured once from the whole file).
    pub(crate) natural: (HashMap<String, f32>, Option<String>),
    doc: Doc,
    past: Vec<Doc>,
    ahead: Vec<Doc>,
    /// The search box, and the words the rows were last narrowed by.
    pub search: TextEdit,
    searched: String,
    pub picked: Filters,
    pub sort: Option<Sort>,
    pub pick: Option<Pick>,
    pub editing: Option<Editing>,
    /// The row opened whole, by its place in the view, and its fields being written.
    pub sheet: Option<usize>,
    pub fields: Option<Field>,
    pub float: Option<Float>,
    pub bar: Option<Bar>,
    /// The Filter menu's box, and the bar menu's lit row.
    pub filter_find: TextEdit,
    pub bar_lit: Option<usize>,
    /// A header being renamed, and its box.
    pub renaming: Option<usize>,
    pub rename: TextEdit,
    rename_held: bool,
    pub problem: Option<Problem>,
    /// The widths a hand set, by column name.
    pub sized: BTreeMap<String, f32>,
    /// A grip being dragged: the column's name and its width when the drag began.
    gripping: Option<(String, f32)>,
    /// The keys are somewhere outside the table: the pick goes quiet (`holding`).
    pub holding: bool,
    /// The cell whose words shown whole were put away with Escape (`shut`).
    pub shut: Option<At>,
    pub dropping: Option<Dropping>,
    dragging: Option<Dragging>,
    pressed_pick: Option<Pick>,
    opens_on_click: Option<At>,
    /// What waits to be written, and when; a save on its way.
    unsaved: Option<String>,
    due: Option<f64>,
    saving: bool,
}

/// The sheets open, and what the app answered their saves.
#[derive(Default)]
pub struct Sheets {
    pub pane: Option<Sheet>,
    pub big: Option<Sheet>,
    /// A box for a sheet that has just gone, so a field's edit still has somewhere to land.
    spare: TextEdit,
    landed: Landed,
    /// In a test: saves go to nobody, what they would send is kept, and the app's answer is
    /// `answer` (or that it was written).
    #[cfg(test)]
    pub test_door: bool,
    #[cfg(test)]
    pub wrote: Vec<Value>,
    #[cfg(test)]
    pub answer: Option<Value>,
}

/// The answers to saves, landing from the thread that asked: where, the text it wrote, and
/// what the app said.
type Landed = Arc<Mutex<Vec<(Where, String, String, Value)>>>;

impl Sheets {
    pub fn get(&self, w: Where) -> Option<&Sheet> {
        match w {
            Where::Pane => self.pane.as_ref(),
            Where::Big => self.big.as_ref(),
        }
    }
    pub fn get_mut(&mut self, w: Where) -> Option<&mut Sheet> {
        match w {
            Where::Pane => self.pane.as_mut(),
            Where::Big => self.big.as_mut(),
        }
    }
    fn slot(&mut self, w: Where) -> &mut Option<Sheet> {
        match w {
            Where::Pane => &mut self.pane,
            Where::Big => &mut self.big,
        }
    }
}

thread_local! {
    /// Where each sheet's scroll viewport stands, as it last drew (`on_scroll_offset`).
    static SCROLLED: RefCell<HashMap<Where, Scrolled>> = RefCell::new(HashMap::new());
}

/// Where a sheet's viewport is scrolled to, how far it can go and how much it shows.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Scrolled {
    pub offset: [f32; 2],
    pub max: [f32; 2],
    pub viewport: [f32; 2],
}

pub(crate) fn heard(w: Where, at: ScrollPos) {
    SCROLLED.with(|s| s.borrow_mut().insert(w, Scrolled { offset: at.offset, max: at.max, viewport: at.viewport }));
}

pub(crate) fn scrolled(w: Where) -> Scrolled {
    SCROLLED.with(|s| s.borrow().get(&w).copied()).unwrap_or_default()
}

/* ------------------------------ a sheet read ------------------------------ */

impl Sheet {
    /// A sheet read from `text`, with what was kept for it applied (`readView`, `readWidths`,
    /// `readLooks`).
    fn new(at: Where, product: &str, src: &str, thread: Option<String>, found: &str, text: Rc<str>, mtime: f64, editable: bool, kept: &Kept) -> Sheet {
        let path = found;
        let base = file::read_table(&text, path);
        let shape = edit::shape_of(&text, path);
        let looks = kept.looks.clone();
        let table = edit::with_looks(&base, &looks);
        let named = |name: &str| table.columns.iter().position(|c| c.name == name);
        let picked: Filters = kept.picked.iter().filter(|(_, v)| !v.is_empty()).filter_map(|(n, v)| named(n).map(|c| (c, v.clone()))).collect();
        let sort = kept.sort.as_ref().and_then(|s| named(&s.name).map(|col| Sort { col, up: s.dir != "desc" }));
        let order = Rc::new(view_order(&table, "", &picked, sort));
        let name = found.rsplit('/').next().unwrap_or(found).to_string();
        let (px, wide) = draw::column_widths(&table, at.cap(), None);
        let natural = (table.columns.iter().zip(px).map(|(c, w)| (c.name.clone(), w)).collect(), wide.map(|i| table.columns[i].name.clone()));
        Sheet {
            at,
            product: product.to_string(),
            src: src.to_string(),
            thread,
            found: found.to_string(),
            name,
            store: format!("{product}/{src}"),
            text,
            shape,
            mtime,
            editable,
            doc: Doc { table, order, looks, filter: None },
            natural,
            past: Vec::new(),
            ahead: Vec::new(),
            search: TextEdit::new(""),
            searched: String::new(),
            picked,
            sort,
            pick: None,
            editing: None,
            sheet: None,
            fields: None,
            float: None,
            bar: None,
            filter_find: TextEdit::new(""),
            bar_lit: None,
            renaming: None,
            rename: TextEdit::new(""),
            rename_held: false,
            problem: None,
            sized: kept.widths.clone(),
            gripping: None,
            holding: true,
            shut: None,
            dropping: None,
            dragging: None,
            pressed_pick: None,
            opens_on_click: None,
            unsaved: None,
            due: None,
            saving: false,
        }
    }

    pub fn table(&self) -> &Table {
        &self.doc.table
    }
    pub fn order(&self) -> &[usize] {
        &self.doc.order
    }
    /// The first column a hand can pick: a ranked file's rank is the row number.
    pub fn first(&self) -> usize {
        usize::from(self.doc.table.ranked)
    }
    pub fn last(&self) -> usize {
        self.doc.table.columns.len().saturating_sub(1)
    }
    pub fn edges(&self) -> Edges {
        Edges { rows: self.doc.order.len(), first: self.first(), last: self.last() }
    }
    /// Every row is three lines tall while a column of words wraps.
    pub fn row_h(&self) -> f32 {
        if self.doc.table.columns.iter().any(wraps) { ROW_WRAPPED } else { ROW_H }
    }
    /// Room for a four-figure row number.
    pub fn rn_w(&self) -> f32 {
        if self.doc.table.rows.len() >= 1000 { RN_W_WIDE } else { RN_W }
    }
    pub fn sel(&self) -> Option<Block> {
        self.pick.as_ref().map(edit::box_of)
    }
    pub fn whole_rows(&self) -> bool {
        self.sel().is_some_and(|b| b.c0 <= self.first() && b.c1 >= self.last())
    }
    pub fn whole_cols(&self) -> bool {
        let n = self.doc.order.len();
        n > 0 && self.sel().is_some_and(|b| b.r0 == 0 && b.r1 == n - 1)
    }
    /// The row of the file at a place in the view.
    pub fn row_at(&self, r: usize) -> Option<&file::Cells> {
        self.doc.order.get(r).and_then(|i| self.doc.table.rows.get(*i)).map(|r| r.as_ref())
    }
    pub fn value(&self, at: At) -> &str {
        self.row_at(at.r).map_or("", |row| row.at(at.c))
    }
    /// The single cell picked, when only one is.
    pub fn one(&self) -> Option<At> {
        self.pick.filter(|p| p.a == p.b).map(|p| p.a)
    }
    /// Something of the table's own stands nearer than its search: a pick, an edit, the row
    /// opened, a menu or a rename (`owns`).
    fn owns(&self) -> bool {
        self.pick.is_some() || self.editing.is_some() || self.sheet.is_some() || self.float.is_some() || self.renaming.is_some() || self.bar.is_some()
    }
}

/// A column set to wrap and still a column of words (`wraps`).
pub fn wraps(c: &file::Column) -> bool {
    c.wrap && wordy(c.kind)
}

fn wordy(kind: Kind) -> bool {
    matches!(kind, Kind::Text | Kind::Name | Kind::Link | Kind::Email)
}

/// The view's order of the file's rows (`viewOrder`).
fn view_order(table: &Table, query: &str, picked: &Filters, sort: Option<Sort>) -> Vec<usize> {
    file::sort_rows(table, file::filter_rows(table, query, picked), sort)
}

/// What a tag or a date cell pops (`popOf`).
fn pop_of(kind: Option<Kind>) -> Option<PopKind> {
    match kind {
        Some(Kind::Tag) => Some(PopKind::Tag),
        Some(Kind::Date) => Some(PopKind::Date),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PopKind {
    Tag,
    Date,
}

/// A date typed in words, written the way the column writes its dates; anything else as typed
/// (`asDate`).
fn as_date(value: &str, was: &str, cells: &[String], now: f64) -> String {
    let day = if js::trim(value) == js::trim(was) { f64::NAN } else { file::day_typed(value, now) };
    if day.is_finite() { file::date_words(day, cells) } else { value.to_string() }
}

fn column_cells(t: &Table, c: usize) -> Vec<String> {
    t.rows.iter().map(|r| r.at(c).to_string()).collect()
}

/* ------------------------------ opening, keeping, following ------------------------------ */

/// The sheet open in the pane, made again when the pane's file is another or was read again
/// with other words; and the same for the big view.
pub fn sync(s: &mut State) {
    let pane = s.beside().and_then(|d| d.table_read().map(|(found, text, mtime)| (d.product.clone(), d.src.clone(), found.to_string(), text, mtime, d.thread.clone())));
    let roots = pane.as_ref().map(|p| crate::pane::project_roots(s, &p.0)).unwrap_or_default();
    match pane {
        Some((product, src, found, text, mtime, _thread)) => {
            // Beside a conversation it is written only inside the project's own folders
            // (`paneCanSave`).
            let editable = crate::rules::pane::pane_can_save(Some(&found), &roots);
            land(s, Where::Pane, &product, &src, None, &found, text, mtime, editable);
        }
        None => close(s, Where::Pane),
    }
    let big = s.viewer.open.as_ref().and_then(|o| o.table_read().map(|(product, src, found, text, mtime, thread)| (product, src, found.to_string(), text, mtime, thread)));
    match big {
        Some((product, src, found, text, mtime, thread)) => land(s, Where::Big, &product, &src, thread, &found, text, mtime, true),
        None => close(s, Where::Big),
    }
}

/// A sheet for these words at `w`: the one open kept when it is the same file with the same
/// words, its time moved on when the disk now holds what it last wrote, else read again.
#[allow(clippy::too_many_arguments)]
fn land(s: &mut State, w: Where, product: &str, src: &str, thread: Option<String>, found: &str, text: Rc<str>, mtime: f64, editable: bool) {
    let same_file = s.tables.get(w).is_some_and(|sh| sh.product == product && sh.src == src && sh.found == found);
    if same_file {
        let sh = s.tables.get_mut(w).expect("the sheet open");
        if Rc::ptr_eq(&sh.text, &text) || *sh.text == *text {
            sh.text = text;
            return;
        }
        // Read again after its own save: the words on disk are the ones it wrote.
        if edit::write_table(&sh.doc.table, &sh.shape) == *text {
            sh.text = text;
            sh.mtime = sh.mtime.max(mtime);
            return;
        }
        // Read again with other words: a new table, and what waited to be written is tried
        // with the time it had, which the app refuses as the file moved on.
        flush(s, w, false);
        let kept = s.picks.get().tables.get(&format!("{product}/{src}")).cloned().unwrap_or_default();
        let old = s.tables.slot(w).take().expect("the sheet open");
        let mut next = Sheet::new(w, product, src, thread, found, text, mtime, editable, &kept);
        // The search, the filter and the sort stay with the file through a reload.
        next.search = old.search;
        next.searched = old.searched;
        next.picked = old.picked;
        next.sort = old.sort;
        next.saving = old.saving;
        next.doc.order = Rc::new(view_order(&next.doc.table, &next.searched, &next.picked, next.sort));
        *s.tables.slot(w) = Some(next);
        return;
    }
    close(s, w);
    let kept = s.picks.get().tables.get(&format!("{product}/{src}")).cloned().unwrap_or_default();
    *s.tables.slot(w) = Some(Sheet::new(w, product, src, thread, found, text, mtime, editable, &kept));
}

/// The sheet at `w` put down: what waits to be written is written (`flush` on unmount), and a
/// refusal then is said in a line, there being no table left to say it.
fn close(s: &mut State, w: Where) {
    if s.tables.get(w).is_none() {
        return;
    }
    flush(s, w, false);
    *s.tables.slot(w) = None;
}

/// Keeps the sort and the filters by column name (`keepView`).
fn keep_view(s: &mut State, w: Where) {
    let Some(sh) = s.tables.get(w) else { return };
    let cols = &sh.doc.table.columns;
    let picked: BTreeMap<String, Vec<String>> = sh.picked.iter().filter(|(_, v)| !v.is_empty()).filter_map(|(c, v)| cols.get(*c).map(|col| (col.name.clone(), v.clone()))).collect();
    let sort = sh.sort.and_then(|so| cols.get(so.col).map(|c| KeptSort { name: c.name.clone(), dir: if so.up { "asc" } else { "desc" }.into() }));
    let store = sh.store.clone();
    s.picks.change(|p| {
        let kept = p.tables.entry(store.clone()).or_default();
        kept.picked = picked;
        kept.sort = sort;
        if *kept == Kept::default() {
            p.tables.remove(&store);
        }
    });
}

fn keep_widths(s: &mut State, w: Where) {
    let Some(sh) = s.tables.get(w) else { return };
    let (store, widths) = (sh.store.clone(), sh.sized.clone());
    s.picks.change(|p| p.tables.entry(store).or_default().widths = widths);
}

fn keep_looks(s: &mut State, w: Where) {
    let Some(sh) = s.tables.get(w) else { return };
    let (store, looks) = (sh.store.clone(), sh.doc.looks.clone());
    s.picks.change(|p| p.tables.entry(store).or_default().looks = looks);
}

/* ------------------------------ changes and undo ------------------------------ */

fn sheet(s: &mut State, w: Where) -> Option<&mut Sheet> {
    s.tables.get_mut(w)
}

/// One change made (`commit`): kept for undo, its looks kept, and written in 400 ms when it
/// changed the file.
fn commit(s: &mut State, w: Where, given: Doc) {
    let now = crate::clock();
    let Some(sh) = sheet(s, w) else { return };
    let was = sh.doc.clone();
    if given.table.same(&was.table) && Rc::ptr_eq(&given.order, &was.order) && given.looks == was.looks {
        return;
    }
    let changed = edit::file_changed(&given.table, &was.table);
    // A tag added now keeps the colour it was drawn in; read again, the guess would deal it another.
    let next = if changed { Doc { looks: edit::pin_tones(&given.table, &given.looks), ..given } } else { given };
    sh.past.push(was.clone());
    if sh.past.len() > UNDO_DEPTH {
        sh.past.remove(0);
    }
    sh.ahead.clear();
    let looks_moved = next.looks != was.looks;
    sh.doc = next;
    if changed && sh.editable {
        sh.unsaved = Some(edit::write_table(&sh.doc.table, &sh.shape));
        sh.due = Some(now + SAVE_MS);
    }
    if looks_moved {
        keep_looks(s, w);
    }
}

/// Undo, or redo (`travel`).
fn travel(s: &mut State, w: Where, back: bool) {
    let now = crate::clock();
    let Some(sh) = sheet(s, w) else { return };
    let next = if back { sh.past.pop() } else { sh.ahead.pop() };
    let Some(next) = next else { return };
    let was = std::mem::replace(&mut sh.doc, next.clone());
    if back { sh.ahead.push(was.clone()) } else { sh.past.push(was.clone()) }
    sh.pick = sh.pick.and_then(|p| clamp_pick(p, &next));
    // A step that changed the filter: the view is worked out again from the filter it had.
    let filter = if back { was.filter.as_ref().map(|f| f.0.clone()) } else { next.filter.as_ref().map(|f| f.1.clone()) };
    let changed = edit::file_changed(&next.table, &was.table);
    if changed && sh.editable {
        sh.unsaved = Some(edit::write_table(&next.table, &sh.shape));
        sh.due = Some(now + SAVE_MS);
    }
    let looks_moved = next.looks != was.looks;
    if let Some(filter) = filter {
        set_view(s, w, None, Some(filter), None, false);
    } else {
        // Undoing a column moved takes the sort and the filters back with their columns.
        let at = |i: usize| was.table.columns.get(i).and_then(|c| next.table.columns.iter().position(|n| n.name == c.name)).unwrap_or(i);
        if let Some(sh) = sheet(s, w) {
            sh.sort = sh.sort.map(|so| Sort { col: at(so.col), ..so });
            sh.picked = std::mem::take(&mut sh.picked).into_iter().map(|(k, v)| (at(k), v)).collect();
        }
        keep_view(s, w);
    }
    if looks_moved {
        keep_looks(s, w);
    }
}

fn clamp_pick(p: Pick, d: &Doc) -> Option<Pick> {
    if d.order.is_empty() {
        return None;
    }
    let fit = |a: At| At { r: a.r.min(d.order.len() - 1), c: a.c.min(d.table.columns.len().saturating_sub(1)) };
    Some(Pick { a: fit(p.a), b: fit(p.b), rows: p.rows })
}

pub fn undo(s: &mut State, w: Where) {
    travel(s, w, true);
}
pub fn redo(s: &mut State, w: Where) {
    travel(s, w, false);
}

/// The search, the filters or the sort changed (`setQuery`, `setPicked`, `setSort` and the
/// effect under them): the order is worked out again, the rows go back to the top and the pick,
/// an edit and the row opened are let go, the keys staying with the table. `quiet` keeps the
/// view where it is, for a change that only renumbers them.
fn set_view(s: &mut State, w: Where, query: Option<String>, picked: Option<Filters>, sort: Option<Option<Sort>>, quiet: bool) {
    let Some(sh) = sheet(s, w) else { return };
    if let Some(q) = query {
        sh.searched = q;
    }
    if let Some(p) = picked {
        sh.picked = p;
    }
    if let Some(so) = sort {
        sh.sort = so;
    }
    if !quiet {
        sh.doc.order = Rc::new(view_order(&sh.doc.table, &sh.searched, &sh.picked, sh.sort));
        sh.pick = None;
        sh.editing = None;
        sh.sheet = None;
        sh.fields = None;
        to_top(w);
    }
    keep_view(s, w);
}

/// Asks the viewport to go back to its top on the next frame (`scrollTo({ top: 0 })`).
fn to_top(w: Where) {
    SCROLL_ASKED.with(|a| a.borrow_mut().push((w, None, Some(0.0))));
}

thread_local! {
    /// Scrolls asked for by a change, run by the next handler or poll that has a transaction.
    static SCROLL_ASKED: RefCell<Vec<(Where, Option<f32>, Option<f32>)>> = const { RefCell::new(Vec::new()) };
}

/// The scrolls asked for since the last transaction, run now.
fn scroll_asked(tx: &mut Tx) {
    let asked: Vec<(Where, Option<f32>, Option<f32>)> = SCROLL_ASKED.with(|a| a.borrow_mut().drain(..).collect());
    for (w, x, y) in asked {
        let at = scrolled(w);
        tx.scroll_to_instant(w.key("scroll"), x.unwrap_or(at.offset[0]).max(0.0), y.unwrap_or(at.offset[1]).max(0.0));
    }
}

/// Cells written, by their place in the view; rows written past the last are added (`writeAt`).
fn write_at(s: &mut State, w: Where, cells: Vec<edit::Write>) {
    let Some(sh) = s.tables.get(w) else { return };
    let d = sh.doc.clone();
    let (first, last) = (sh.first(), sh.last());
    let mut t = d.table.clone();
    let mut o: Vec<usize> = d.order.as_ref().clone();
    let deepest = cells.iter().map(|c| c.r as i64).max().unwrap_or(-1);
    let beyond = deepest - (o.len() as i64 - 1);
    if beyond > 0 {
        let start = t.rows.len();
        t = edit::add_rows(&t, start, beyond as usize);
        o.extend((0..beyond as usize).map(|k| d.table.rows.len() + k));
    }
    let writes: Vec<edit::Cell> = cells.into_iter().filter(|c| c.c >= first && c.c <= last && c.r < o.len()).map(|c| edit::Cell { i: o[c.r], c: c.c, v: c.v }).filter(|c| t.cell(c.i, c.c) != c.v).collect();
    if writes.is_empty() && beyond <= 0 {
        return;
    }
    let table = edit::write_cells(&t, &writes);
    commit(s, w, Doc { table, order: Rc::new(o), looks: d.looks, filter: None });
}

/// Backspace, Delete, Clear: the picked cells emptied (`clearPicked`).
pub fn clear_picked(s: &mut State, w: Where) {
    let Some(sh) = s.tables.get(w) else { return };
    let Some(b) = sh.sel().filter(|_| sh.editable) else { return };
    let first = sh.first();
    let cells = (b.r0..=b.r1).flat_map(|r| (b.c0.max(first)..=b.c1).map(move |c| edit::Write { r, c, v: String::new() })).collect();
    write_at(s, w, cells);
}

/// The picked cells' words, row by row (`pickedGrid`).
fn picked_grid(sh: &Sheet) -> Vec<Vec<String>> {
    let Some(b) = sh.sel() else { return Vec::new() };
    let first = sh.first();
    (b.r0..=b.r1).filter_map(|r| sh.row_at(r)).map(|row| (b.c0.max(first)..=b.c1).map(|c| row.at(c).to_string()).collect()).collect()
}

/// What was copied, written from the pick's top left, and the cells it filled picked (`paste`).
pub fn paste(s: &mut State, w: Where, text: &str) {
    let Some(sh) = s.tables.get(w) else { return };
    let Some(b) = sh.sel().filter(|_| sh.editable) else { return };
    let (first, last) = (sh.first(), sh.last());
    let values = edit::from_clipboard(text);
    let c0 = b.c0.max(first);
    let cells = edit::paste_into(&values, Block { c0, ..b }, last);
    if cells.is_empty() {
        return;
    }
    let (r1, c1) = (cells.iter().map(|c| c.r).max().unwrap_or(b.r0), cells.iter().map(|c| c.c).max().unwrap_or(c0));
    write_at(s, w, cells);
    if let Some(sh) = sheet(s, w) {
        sh.pick = Some(Pick { a: At { r: b.r0, c: c0 }, b: At { r: r1, c: c1 }, rows: false });
    }
}

/// A blank row put in at a place in the view, and with `edit` its first cell opened (`addRowAt`).
/// In the file, a row at the foot goes last; one among rows goes just after the row above it in
/// the view, so a sorted view does not scatter them.
pub fn add_row_at(s: &mut State, w: Where, place: usize, open: bool, tx: &mut Tx) {
    let Some(sh) = s.tables.get(w) else { return };
    let d = sh.doc.clone();
    let first = sh.first();
    let at = if place >= d.order.len() { d.table.rows.len() } else if place == 0 { d.order[0] } else { d.order[place - 1] + 1 };
    let table = edit::add_rows(&d.table, at, 1);
    let order = edit::order_after_add(&d.order, at, 1, place);
    let pop = pop_of(d.table.columns.get(first).map(|c| c.kind));
    commit(s, w, Doc { table, order: Rc::new(order), looks: d.looks, filter: None });
    go_to(s, w, Some(Pick::one(At { r: place, c: first })), tx);
    if open {
        begin_edit(s, w, At { r: place, c: first }, String::new(), false, pop, tx);
    }
}

/// The picked rows taken out of the file; the pick stays on the row that comes into their place
/// (`dropPicked`).
pub fn drop_picked(s: &mut State, w: Where) {
    let Some(sh) = s.tables.get(w) else { return };
    let Some(b) = sh.sel().filter(|_| sh.editable) else { return };
    let d = sh.doc.clone();
    let keep_c = sh.pick.map_or(sh.first(), |p| p.a.c);
    let gone: HashSet<usize> = d.order[b.r0..=b.r1.min(d.order.len() - 1)].iter().copied().collect();
    let table = edit::drop_rows(&d.table, &gone);
    let order = edit::order_after_drop(&d.order, &gone);
    let rows = order.len();
    commit(s, w, Doc { table, order: Rc::new(order), looks: edit::pin_tones(&d.table, &d.looks), filter: None });
    if let Some(sh) = sheet(s, w) {
        sh.pick = (rows > 0).then(|| Pick::one(At { r: b.r0.min(rows - 1), c: keep_c }));
    }
}

/// A change to the columns; the pick goes to the top of the column `at` names, so typing never
/// lands elsewhere (`changeColumns`).
fn change_columns(s: &mut State, w: Where, t: Table, at: Option<usize>, looks: Option<Looks>) {
    let Some(sh) = s.tables.get(w) else { return };
    let d = sh.doc.clone();
    if !t.same(&d.table) || looks.as_ref().is_some_and(|l| *l != d.looks) {
        commit(s, w, Doc { table: t.clone(), order: d.order.clone(), looks: looks.unwrap_or(d.looks), filter: None });
    }
    let first = usize::from(t.ranked);
    let rows = d.order.len();
    if let Some(sh) = sheet(s, w) {
        sh.pick = at.filter(|_| rows > 0).map(|c| Pick::one(At { r: 0, c: c.clamp(first, t.columns.len().saturating_sub(1)) }));
    }
}

/// A column shown as another type (`setKind`); a name column is never made text, which it is.
pub fn set_kind(s: &mut State, w: Where, col: usize, kind: Kind) {
    let Some(sh) = s.tables.get(w) else { return };
    let Some(c) = sh.doc.table.columns.get(col) else { return };
    if c.kind == Kind::Name && kind == Kind::Text {
        return;
    }
    let looks = edit::look_kind(&sh.doc.looks, &c.name, kind);
    let t = edit::with_looks(&sh.doc.table, &looks);
    change_columns(s, w, t, Some(col), Some(looks));
}

/// A filter on a tag that was renamed follows the new name; one on a tag deleted lets go
/// (`refiltered`).
fn refiltered(picked: &Filters, col: usize, from: &str, to: Option<&str>) -> Option<(Filters, Filters)> {
    let on = picked.get(&col).cloned().unwrap_or_default();
    let key = js::trim(from).to_lowercase();
    if !on.iter().any(|v| v.to_lowercase() == key) {
        return None;
    }
    let mut kept: Vec<String> = on.into_iter().filter(|v| v.to_lowercase() != key).collect();
    if let Some(to) = to
        && !kept.iter().any(|v| v.to_lowercase() == to.to_lowercase())
    {
        kept.push(to.to_string());
    }
    let mut after = picked.clone();
    after.insert(col, kept);
    Some((picked.clone(), after))
}

/// A tag renamed in every cell that holds it (`tagHands.rename`).
pub fn rename_tag(s: &mut State, w: Where, col: usize, from: &str, to: &str) {
    let Some(sh) = s.tables.get(w) else { return };
    let d = sh.doc.clone();
    let Some(c) = d.table.columns.get(col).cloned() else { return };
    let t = edit::rename_tag(&d.table, col, from, to);
    if t.same(&d.table) {
        return;
    }
    let merges = c.values.iter().any(|v| v.to_lowercase() == js::trim(to).to_lowercase() && v.to_lowercase() != js::trim(from).to_lowercase());
    let written = t.columns[col].values.iter().find(|v| v.to_lowercase() == js::trim(to).to_lowercase()).cloned().unwrap_or_else(|| js::trim(to).to_string());
    let filter = refiltered(&sh.picked, col, from, Some(&written));
    let looks = if merges { d.looks.clone() } else { edit::look_rename_tag(&d.looks, &c.name, from, to) };
    commit(s, w, Doc { table: t, order: d.order.clone(), looks, filter: filter.clone() });
    // The same rows stay in view, so the tag list stays open.
    if let Some((_, after)) = filter {
        set_view(s, w, None, Some(after), None, true);
    }
}

/// A tag given another colour (`tagHands.tone`).
pub fn tone_tag(s: &mut State, w: Where, col: usize, tag: &str, tone: Tone) {
    let Some(sh) = s.tables.get(w) else { return };
    let d = sh.doc.clone();
    let Some(c) = d.table.columns.get(col) else { return };
    if c.tones.get(&js::trim(tag).to_lowercase()) == Some(&tone) {
        return;
    }
    let looks = edit::look_tone(&d.looks, &c.name, tag, tone);
    let table = edit::with_looks(&d.table, &looks);
    commit(s, w, Doc { table, order: d.order.clone(), looks, filter: None });
}

/// A tag taken out of the column and out of every cell (`tagHands.drop`).
pub fn drop_tag(s: &mut State, w: Where, col: usize, tag: &str) {
    let Some(sh) = s.tables.get(w) else { return };
    let d = sh.doc.clone();
    let filter = refiltered(&sh.picked, col, tag, None);
    let table = edit::drop_tag(&d.table, col, tag);
    commit(s, w, Doc { table, order: d.order.clone(), looks: d.looks.clone(), filter: filter.clone() });
    if let Some((_, after)) = filter {
        set_view(s, w, None, Some(after), None, false);
    }
}

/// How many rows hold a tag (`tagHands.count`).
pub fn tag_count(sh: &Sheet, col: usize, tag: &str) -> usize {
    let key = js::trim(tag).to_lowercase();
    sh.doc.table.rows.iter().filter(|r| js::trim(r.at(col)).to_lowercase() == key).count()
}

/// Whole rows picked moved so they stand before the row at `place` in the view; only in the
/// file's own order, as under a sort the sort decides (`moveRowsTo`).
pub fn move_rows_to(s: &mut State, w: Where, place: usize) {
    let Some(sh) = s.tables.get(w) else { return };
    let Some(b) = sh.sel() else { return };
    let d = sh.doc.clone();
    if sh.sort.is_some() || place > d.order.len() || (place >= b.r0 && place <= b.r1 + 1) {
        return;
    }
    let (first, last, row_h) = (sh.first(), sh.last(), sh.row_h());
    let before = if place >= d.order.len() { d.table.rows.len() } else { d.order[place] };
    let (t, to) = edit::move_rows(&d.table, &d.order[b.r0..=b.r1], before);
    if t.same(&d.table) {
        return;
    }
    let mut order: Vec<usize> = d.order.iter().map(|i| to[*i]).collect();
    order.sort_unstable();
    let r0 = order.iter().position(|i| *i == to[d.order[b.r0]]).unwrap_or(0);
    commit(s, w, Doc { table: t, order: Rc::new(order), looks: edit::pin_tones(&d.table, &d.looks), filter: None });
    if let Some(sh) = sheet(s, w) {
        // Not followed sideways: the pick's far end would scroll the table to its last column.
        sh.pick = Some(Pick { a: At { r: r0, c: first }, b: At { r: r0 + b.r1 - b.r0, c: last }, rows: true });
    }
    keep_row_in_view(w, r0, row_h);
}

fn keep_row_in_view(w: Where, r: usize, row_h: f32) {
    let at = scrolled(w);
    let top = HEAD_H + r as f32 * row_h;
    let y = if top < at.offset[1] + HEAD_H { Some(top - HEAD_H) } else if top + row_h > at.offset[1] + at.viewport[1] { Some(top + row_h - at.viewport[1]) } else { None };
    if y.is_some() {
        SCROLL_ASKED.with(|a| a.borrow_mut().push((w, None, y)));
    }
}

/// Columns `c0..=c1` moved before the column at `place`; the sort and the filters follow their
/// columns (`moveColsTo`).
pub fn move_cols_to(s: &mut State, w: Where, c0: usize, c1: usize, place: usize, tx: &mut Tx) {
    let Some(sh) = s.tables.get(w) else { return };
    let d = sh.doc.clone();
    let (t, to) = edit::move_columns(&d.table, c0, c1 - c0 + 1, place);
    if t.same(&d.table) {
        return;
    }
    commit(s, w, Doc { table: t, order: d.order.clone(), looks: d.looks.clone(), filter: None });
    let Some(sh) = sheet(s, w) else { return };
    sh.sort = sh.sort.map(|so| Sort { col: to[so.col], ..so });
    sh.picked = std::mem::take(&mut sh.picked).into_iter().map(|(k, v)| (to[k], v)).collect();
    let rows = sh.doc.order.len();
    if rows > 0 {
        sh.pick = Some(Pick { a: At { r: 0, c: to[c0] }, b: At { r: rows - 1, c: to[c1] }, rows: false });
    }
    keep_view(s, w);
    focus_keys(s, w, tx);
    // Sideways only: a column moved left must not land under the frozen first column.
    if let Some(sh) = s.tables.get(w) {
        let at = scrolled(w);
        let (x, cw) = draw::column_x(s, sh, to[c0]);
        let stuck = stuck_width(s, sh, to[c0]);
        let x_to = if x < at.offset[0] + stuck { Some((x - stuck).max(0.0)) } else if x + cw > at.offset[0] + at.viewport[0] { Some(x + cw - at.viewport[0]) } else { None };
        if x_to.is_some() {
            SCROLL_ASKED.with(|a| a.borrow_mut().push((w, x_to, None)));
        }
    }
}

/// How much of the left edge the row numbers and the first column cover, past the first
/// column (`stuckWidth`).
fn stuck_width(s: &State, sh: &Sheet, c: usize) -> f32 {
    if c == sh.first() { sh.rn_w() } else { sh.rn_w() + draw::column_x(s, sh, sh.first()).1 }
}

/// Wrap text, for a column of words: on, each row shows three lines of it (`wrapRow`).
pub fn toggle_wrap(s: &mut State, w: Where, col: usize) {
    let Some(sh) = s.tables.get(w) else { return };
    let d = sh.doc.clone();
    let Some(c) = d.table.columns.get(col).filter(|c| wordy(c.kind)) else { return };
    let looks = edit::look_wrap(&d.looks, &c.name, !c.wrap);
    let table = edit::with_looks(&d.table, &looks);
    commit(s, w, Doc { table, order: d.order.clone(), looks, filter: None });
}

/// A column fitted to what it holds, with some slack for the font (`fitColumn`).
pub fn fit_column(s: &mut State, w: Where, col: usize) {
    let Some(sh) = sheet(s, w) else { return };
    let Some(name) = sh.doc.table.columns.get(col).map(|c| c.name.clone()) else { return };
    let natural = draw::column_widths(&sh.doc.table, 640.0, Some(col));
    let wide = natural.0.get(col).copied().unwrap_or(160.0) + 12.0;
    sh.sized.insert(name, wide);
    keep_widths(s, w);
}

/// A header's grip dragged: the column as wide as the pointer leaves it, kept once let go.
pub fn grip(s: &mut State, w: Where, col: usize, phase: rune::Phase, dx: f32, from: f32) {
    let Some(sh) = sheet(s, w) else { return };
    let Some(name) = sh.doc.table.columns.get(col).map(|c| c.name.clone()) else { return };
    match phase {
        rune::Phase::Down => sh.gripping = Some((name, from)),
        rune::Phase::Move => {
            if let Some((name, from)) = sh.gripping.clone() {
                sh.sized.insert(name, (from + dx).round().max(MIN_W));
            }
        }
        rune::Phase::Up => {
            if let Some((name, from)) = sh.gripping.take() {
                sh.sized.insert(name, (from + dx).round().max(MIN_W));
                keep_widths(s, w);
            }
        }
    }
}

/// The sort the Sort menu and a header's menu set (`sortBy`): a column not sorted by sorts
/// upward, the one sorted by turns downward, then off.
pub fn sort_by(s: &mut State, w: Where, col: usize) {
    let Some(sh) = s.tables.get(w) else { return };
    let next = match sh.sort {
        Some(so) if so.col == col && so.up => Some(Sort { col, up: false }),
        Some(so) if so.col == col => None,
        _ => Some(Sort { col, up: true }),
    };
    set_view(s, w, None, None, Some(next), false);
}

pub fn set_sort(s: &mut State, w: Where, sort: Option<Sort>) {
    set_view(s, w, None, None, Some(sort), false);
}

/// A value of the Filter menu ticked or unticked (`toggle`).
pub fn toggle_filter(s: &mut State, w: Where, col: usize, value: &str) {
    let Some(sh) = s.tables.get(w) else { return };
    let mut picked = sh.picked.clone();
    let now = picked.entry(col).or_default();
    if let Some(at) = now.iter().position(|v| v == value) {
        now.remove(at);
    } else {
        now.push(value.to_string());
    }
    set_view(s, w, None, Some(picked), None, false);
}

pub fn clear_filters(s: &mut State, w: Where) {
    set_view(s, w, None, Some(Filters::new()), None, false);
}

/// What is typed in the search box narrows the rows.
pub fn searched(s: &mut State, w: Where) {
    let Some(sh) = s.tables.get(w) else { return };
    if sh.search.text == sh.searched {
        return;
    }
    let q = sh.search.text.clone();
    set_view(s, w, Some(q), None, None, false);
}

/* ------------------------------ the pick and the keys ------------------------------ */

/// The keys to the table (`focusKeys`): no box of its own has them.
pub fn focus_keys(s: &mut State, w: Where, tx: &mut Tx) {
    let Some(sh) = sheet(s, w) else { return };
    sh.search.focused = false;
    sh.filter_find.focused = false;
    sh.rename.focused = false;
    // The table has the keys from this very frame, its pick lit.
    sh.holding = false;
    tx.focus(w.key("scroll"), false);
}

/// A pick made, and its moving end kept in view (`goTo` and the follow effect).
fn go_to(s: &mut State, w: Where, pick: Option<Pick>, _tx: &mut Tx) {
    let Some(sh) = sheet(s, w) else { return };
    sh.pick = pick;
    let Some(p) = pick else { return };
    let row_h = sh.row_h();
    keep_row_in_view(w, p.b.r, row_h);
    let Some(sh) = s.tables.get(w) else { return };
    let at = scrolled(w);
    let (x, cw) = draw::column_x(s, sh, p.b.c);
    let stuck = stuck_width(s, sh, p.b.c);
    let room = at.viewport[0] - stuck;
    // A cell wider than the view shows its start, where its words begin, not its far end.
    let x_to = if x < at.offset[0] + stuck || (cw > room && x > at.offset[0] + stuck) {
        Some((x - stuck).max(0.0))
    } else if x + cw > at.offset[0] + at.viewport[0] {
        Some(x + cw - at.viewport[0])
    } else {
        None
    };
    if p.b.c != sh.first() && x_to.is_some() {
        SCROLL_ASKED.with(|a| a.borrow_mut().push((w, x_to, None)));
    }
}

/// A cell opened for typing (`beginEdit`).
fn begin_edit(s: &mut State, w: Where, at: At, draft: String, typed: bool, pop: Option<PopKind>, tx: &mut Tx) {
    let now = s.now;
    let Some(sh) = sheet(s, w) else { return };
    let was = sh.value(at).to_string();
    let mut words = TextEdit::new(draft);
    words.anchor = words.text.len();
    words.focus = words.text.len();
    let pop = pop.map(|p| match p {
        PopKind::Tag => {
            let mut find = TextEdit::new(if typed { words.text.clone() } else { String::new() });
            find.anchor = find.text.len();
            find.focus = find.text.len();
            find.focused = true;
            Pop::Tag(TagPick { find, hi: None, page: None, held: false })
        }
        PopKind::Date => Pop::Date(DatePick::new(&was, now)),
    });
    words.focused = pop.is_none();
    sh.search.focused = false;
    sh.editing = Some(Editing { r: at.r, c: at.c, draft: words, typed, pop, held: false });
    let _ = tx;
}

impl DatePick {
    fn new(value: &str, now: f64) -> DatePick {
        let mut words = TextEdit::new(value);
        // Selected whole, so typing replaces it (`onFocus` selects).
        words.anchor = 0;
        words.focus = words.text.len();
        words.focused = true;
        let day = file::day_typed(value, now);
        let shown = if day.is_finite() { crate::rules::calendar::Day((day / 86_400_000.0).floor() as i64) } else { crate::rules::calendar::Day::of(now) };
        let (y, m, _) = shown.ymd();
        DatePick { words, month: (y, m), held: false }
    }
}

/// A cell edited, or read where it cannot be (`startEdit`): a ranked file's rank never is, nor a
/// tag cell of a table that cannot be changed.
pub fn start_edit(s: &mut State, w: Where, at: At, typed: Option<String>, tx: &mut Tx) {
    let Some(sh) = s.tables.get(w) else { return };
    let Some(column) = sh.doc.table.columns.get(at.c) else { return };
    if sh.row_at(at.r).is_none() || (sh.doc.table.ranked && at.c == 0) {
        return;
    }
    if !sh.editable && column.kind == Kind::Tag {
        return;
    }
    let pop = if sh.editable { pop_of(Some(column.kind)) } else { None };
    let draft = typed.clone().unwrap_or_else(|| sh.value(at).to_string());
    if let Some(sh) = sheet(s, w) {
        sh.float = None;
    }
    go_to(s, w, Some(Pick::one(at)), tx);
    begin_edit(s, w, at, draft, typed.is_some(), pop, tx);
}

/// How an edit ends and where the pick goes after it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Then {
    Stay,
    Down,
    Right,
    Left,
}

/// An edit ended (`endEdit`): its words written with a number or a day as the column writes
/// them, unless dropped (`None`), the pick moved on, and the keys back to the table.
pub fn end_edit(s: &mut State, w: Where, value: Option<String>, then: Then, tx: &mut Tx) {
    let now = s.now;
    let Some(sh) = sheet(s, w) else { return };
    let Some(e) = sh.editing.take() else { return };
    if let Some(value) = value.filter(|_| sh.editable) {
        let t = &sh.doc.table;
        let cells = column_cells(t, e.c);
        let was = sh.value(At { r: e.r, c: e.c }).to_string();
        let v = match t.columns.get(e.c).map(|c| c.kind) {
            Some(Kind::Number) => edit::like_its_column(&value, &cells),
            Some(Kind::Date) => as_date(&value, &was, &cells, now),
            _ => value,
        };
        write_at(s, w, vec![edit::Write { r: e.r, c: e.c, v }]);
    }
    if then != Then::Stay {
        let Some(sh) = s.tables.get(w) else { return };
        let step = match then {
            Then::Down => Step::Down,
            Then::Right => Step::Right,
            _ => Step::Left,
        };
        let to = edit::moved(At { r: e.r, c: e.c }, step, sh.edges(), false, 10);
        go_to(s, w, Some(Pick::one(to)), tx);
    }
    focus_keys(s, w, tx);
}

/// The words of an edit open, as they stand.
fn draft_of(e: &Editing) -> String {
    match &e.pop {
        Some(Pop::Date(d)) => d.words.text.clone(),
        _ => e.draft.text.clone(),
    }
}

/// An edit that a press elsewhere ends (`onDown`, `onContext`): its words kept, a tag's list
/// closed with nothing picked.
fn end_by_press(s: &mut State, w: Where, tx: &mut Tx) {
    let Some(e) = s.tables.get(w).and_then(|sh| sh.editing.as_ref()) else { return };
    let value = if matches!(e.pop, Some(Pop::Tag(_))) { None } else { Some(draft_of(e)) };
    end_edit(s, w, value, Then::Stay, tx);
}

/// The keys while the table holds them (`onKeys` and `onRoot`). False lets a key go on to the
/// page: a letter is the app's (R replies, E closes), and Escape with nothing picked.
pub fn grid_key(s: &mut State, w: Where, e: &KeyEvent, tx: &mut Tx) -> bool {
    // A box inside the table has the keys: they are its own.
    if tx.focused() != Some(w.key("scroll").as_str()) {
        return false;
    }
    if s.tables.get(w).is_some_and(|sh| sh.float.is_some()) {
        return float_key(s, w, e, tx).unwrap_or(false);
    }
    let Some(sh) = s.tables.get(w) else { return false };
    let (meta, shift, alt) = (e.mods.cmd || e.mods.ctrl, e.mods.shift, e.mods.alt);
    let editable = sh.editable;
    let (first, sel, pick) = (sh.first(), sh.sel(), sh.pick);
    let (whole_rows, whole_cols, rows) = (sh.whole_rows(), sh.whole_cols(), sh.doc.order.len());
    // Escape puts down rows or columns picked up to move them.
    if e.key == Key::Escape && matches!(sh.dragging, Some(Dragging::MoveRows | Dragging::MoveCols)) {
        if let Some(sh) = sheet(s, w) {
            sh.dragging = None;
            sh.pressed_pick = None;
            sh.dropping = None;
        }
        return true;
    }
    let arrow = match e.key {
        Key::Up => Some(Step::Up),
        Key::Down => Some(Step::Down),
        Key::Left => Some(Step::Left),
        Key::Right => Some(Step::Right),
        Key::Home => Some(Step::Home),
        Key::End => Some(Step::End),
        Key::PageUp => Some(Step::PageUp),
        Key::PageDown => Some(Step::PageDown),
        _ => None,
    };
    // ⌥ and an arrow moves whole rows or columns picked, as the menus' Move does; on one cell
    // it does nothing.
    if alt && !meta && editable && matches!(e.key, Key::Up | Key::Down | Key::Left | Key::Right) {
        if let Some(b) = sel {
            match e.key {
                Key::Up if whole_rows && b.r0 > 0 => move_rows_to(s, w, b.r0 - 1),
                Key::Down if whole_rows => move_rows_to(s, w, b.r1 + 2),
                Key::Left if whole_cols && b.c0 >= first && b.c0 > 0 => move_cols_to(s, w, b.c0, b.c1, b.c0 - 1, tx),
                Key::Right if whole_cols && b.c0 >= first => move_cols_to(s, w, b.c0, b.c1, b.c1 + 2, tx),
                _ => {}
            }
        }
        return true;
    }
    if let Key::Command(c) = e.key {
        match c {
            'z' => {
                if shift { redo(s, w) } else { undo(s, w) }
                return true;
            }
            'y' => {
                redo(s, w);
                return true;
            }
            'a' => {
                if rows > 0
                    && let Some(sh) = sheet(s, w)
                {
                    let last = sh.last();
                    sh.pick = Some(Pick { a: At { r: 0, c: first }, b: At { r: rows - 1, c: last }, rows: false });
                }
                return true;
            }
            'c' | 'x' => {
                if let Some(sh) = s.tables.get(w)
                    && sh.sel().is_some()
                {
                    tx.copy(edit::to_clipboard(&picked_grid(sh)));
                    if c == 'x' && editable {
                        clear_picked(s, w);
                    }
                }
                return true;
            }
            'v' => {
                if sel.is_some() {
                    tx.read_clipboard(move |s: &mut State, content: &rune::ClipboardContent, _tx: &mut Tx| paste(s, w, content.text()));
                }
                return true;
            }
            // Fill down: the top row of the block, copied into every row picked below it.
            'd' if editable => {
                if let (Some(b), Some(sh)) = (sel, s.tables.get(w)) {
                    let top: Vec<String> = sh.row_at(b.r0).map(|r| r.cells.clone()).unwrap_or_default();
                    let cells = (b.r0 + 1..=b.r1).flat_map(|r| (b.c0.max(first)..=b.c1).map(|c| edit::Write { r, c, v: top.get(c).cloned().unwrap_or_default() }).collect::<Vec<_>>()).collect();
                    write_at(s, w, cells);
                }
                return true;
            }
            _ => return false,
        }
    }
    let Some(pick) = pick else {
        if (arrow.is_some() || matches!(e.key, Key::Tab | Key::Enter)) && rows > 0 {
            go_to(s, w, Some(Pick::one(At { r: 0, c: first })), tx);
            return true;
        }
        return false;
    };
    match &e.key {
        Key::Escape => {
            let peeked = s.tables.get(w).is_some_and(|sh| draw::peek(s, sh).is_some());
            let Some(sh) = sheet(s, w) else { return false };
            if sel.is_some_and(|b| edit::cells_in(&b) > 1) {
                sh.pick = Some(Pick::one(pick.a));
            } else if peeked {
                sh.shut = Some(pick.a);
            } else {
                // With nothing picked the table keeps the keys, so undo still works and the next
                // Escape goes on to close what it is in.
                sh.pick = None;
            }
            true
        }
        _ if arrow.is_some() => {
            let step = arrow.expect("an arrow");
            let page = (((scrolled(w).viewport[1]).max(400.0) / s.tables.get(w).map_or(ROW_H, Sheet::row_h)).floor() as usize).saturating_sub(1).max(1);
            let Some(sh) = s.tables.get(w) else { return false };
            let edges = sh.edges();
            let next = if shift { Pick { b: edit::moved(pick.b, step, edges, meta, page), ..pick } } else { Pick::one(edit::moved(pick.a, step, edges, meta, page)) };
            go_to(s, w, Some(next), tx);
            true
        }
        Key::Tab => {
            let Some(sh) = s.tables.get(w) else { return false };
            let to = edit::moved(pick.a, if shift { Step::Left } else { Step::Right }, sh.edges(), false, 10);
            go_to(s, w, Some(Pick::one(to)), tx);
            true
        }
        Key::Enter => {
            if shift && editable {
                add_row_at(s, w, pick.a.r + 1, true, tx);
            } else {
                start_edit(s, w, pick.a, None, tx);
            }
            true
        }
        Key::Function(2) => {
            start_edit(s, w, pick.a, None, tx);
            true
        }
        Key::Text(t) if t == " " && !meta => {
            if let Some(sh) = sheet(s, w) {
                sh.sheet = Some(pick.a.r);
            }
            open_row(s, w, pick.a.r, tx);
            true
        }
        Key::Backspace | Key::Delete => {
            clear_picked(s, w);
            true
        }
        // A letter is the app's, not the cell's: Enter, F2 or a double press edits.
        _ => false,
    }
}

/// The keys the window has before the page while a sheet is open (`keydown` listeners on the
/// window in `TableView`): an open menu's arrows, Enter and Escape, and Escape clearing what the
/// search box holds before it closes what the table is in. None when the sheets have no say.
pub fn key(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> Option<bool> {
    for w in [Where::Big, Where::Pane] {
        let Some(sh) = s.tables.get(w) else { continue };
        if w == Where::Pane && s.viewer.open.is_some() {
            continue;
        }
        if sh.float.is_some() {
            return float_key(s, w, e, tx);
        }
        if sh.bar.is_some() {
            return bar_key(s, w, e, tx);
        }
        // With words in the search box, Escape clears them before it closes the view the table
        // is in, but not while a pick, an edit or a menu is the nearer thing to close, and not
        // while the keys are in a box outside the table.
        if e.key == Key::Escape && !sh.search.text.is_empty() && !sh.owns() {
            let outside = tx.focused().is_some_and(|f| !f.starts_with("tbl-") && (s.reply.focused || s.compose.words.focused));
            if !outside {
                if let Some(sh) = sheet(s, w) {
                    sh.search = TextEdit::new("");
                }
                searched(s, w);
                return Some(true);
            }
        }
    }
    None
}

/// The keys of a menu at a point (`FloatMenu`): ↑ and ↓ walk its rows around both ends past the
/// ones that are off, Enter presses the one reached, Escape closes it; every other key waits.
fn float_key(s: &mut State, w: Where, e: &KeyEvent, tx: &mut Tx) -> Option<bool> {
    let rows = draw::float_rows(s, w);
    let sh = sheet(s, w)?;
    let f = sh.float.as_mut()?;
    match e.key {
        Key::Escape => {
            sh.float = None;
            Some(true)
        }
        Key::Up | Key::Down => {
            let live: Vec<usize> = rows.iter().enumerate().filter(|(_, r)| !r.off).map(|(i, _)| i).collect();
            if live.is_empty() {
                return Some(false);
            }
            let down = e.key == Key::Down;
            let now = f.lit.and_then(|l| live.iter().position(|i| *i == l));
            let next = match now {
                None => if down { 0 } else { live.len() - 1 },
                Some(at) => (at + if down { 1 } else { live.len() - 1 }) % live.len(),
            };
            f.lit = Some(live[next]);
            Some(true)
        }
        Key::Enter => {
            let Some(lit) = f.lit else { return Some(false) };
            if let Some(r) = rows.get(lit).filter(|r| !r.off) {
                let act = r.act.clone();
                draw::press_row(s, w, act, tx);
            }
            Some(true)
        }
        _ => Some(false),
    }
}

/// The keys of a bar menu: Escape closes it and the keys go back to the table (`keysBack`).
fn bar_key(s: &mut State, w: Where, e: &KeyEvent, tx: &mut Tx) -> Option<bool> {
    if e.key != Key::Escape {
        return None;
    }
    let sh = sheet(s, w)?;
    sh.bar = None;
    sh.filter_find = TextEdit::new("");
    let picked = sh.pick.is_some();
    if picked {
        focus_keys(s, w, tx);
    } else {
        tx.focus(w.key("scroll"), false);
    }
    Some(true)
}

/// The row opened whole (`setSheet`), the keys in it.
pub fn open_row(s: &mut State, w: Where, r: usize, tx: &mut Tx) {
    let Some(sh) = sheet(s, w) else { return };
    sh.sheet = Some(r);
    sh.fields = None;
    tx.focus(w.key("sheet"), false);
}

/* ------------------------------ the pointer ------------------------------ */

/// What a point of the grid is: a cell, a row number or a header.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hit {
    Cell(At),
    Row(usize),
    Col(usize),
}

/// The pick a press on `hit` makes, from the cell the pick began at with `extend` (`pickFor`).
fn pick_for(sh: &Sheet, hit: Hit, extend: bool) -> Pick {
    let (first, last) = (sh.first(), sh.last());
    let rows = sh.doc.order.len();
    let from = |at: At| if extend { sh.pick.map_or(at, |p| p.a) } else { at };
    match hit {
        Hit::Row(r) => {
            let a = from(At { r, c: first });
            Pick { a: At { r: a.r, c: first }, b: At { r, c: last }, rows: true }
        }
        Hit::Col(c) => {
            let a = from(At { r: 0, c });
            Pick { a: At { r: 0, c: a.c }, b: At { r: rows.saturating_sub(1), c }, rows: false }
        }
        Hit::Cell(at) => Pick { a: from(at), b: at, rows: false },
    }
}

/// A press on the grid at a point of it (`onDown`): it picks, as everywhere in a spreadsheet,
/// stretches with Shift, picks up whole rows or columns already picked, opens a tag or a date
/// cell once it ends where it began, and with ⌘ opens a cell's link.
pub fn pressed_at(s: &mut State, w: Where, hit: Hit, link: Option<String>, tx: &mut Tx) {
    let mods = tx.mods();
    let Some(sh) = s.tables.get(w) else { return };
    let was_open = sh.editing.as_ref().is_some_and(|e| matches!(hit, Hit::Cell(at) if at.r == e.r && at.c == e.c));
    if sh.editing.is_some() {
        end_by_press(s, w, tx);
    }
    let Some(sh) = s.tables.get(w) else { return };
    if matches!(hit, Hit::Col(_)) && sh.doc.order.is_empty() {
        return;
    }
    let plain = !mods.shift && !mods.cmd && !mods.ctrl;
    let sel = sh.sel();
    let (editable, sorted, whole_rows, whole_cols, first) = (sh.editable, sh.sort.is_some(), sh.whole_rows(), sh.whole_cols(), sh.first());
    // A press on rows or columns already picked picks them up to move them, as Sheets does.
    if let (Some(b), true, true) = (sel, editable, plain) {
        if let Hit::Row(r) = hit
            && whole_rows
            && !sorted
            && r >= b.r0
            && r <= b.r1
        {
            let p = pick_for(sh, hit, false);
            if let Some(sh) = sheet(s, w) {
                sh.dragging = Some(Dragging::MoveRows);
                sh.pressed_pick = Some(p);
            }
            focus_keys(s, w, tx);
            return;
        }
        if let Hit::Col(c) = hit
            && whole_cols
            && b.c0 >= first
            && c >= b.c0
            && c <= b.c1
        {
            let p = pick_for(sh, hit, false);
            if let Some(sh) = sheet(s, w) {
                sh.dragging = Some(Dragging::MoveCols);
                sh.pressed_pick = Some(p);
            }
            focus_keys(s, w, tx);
            return;
        }
    }
    // A link opens with ⌘ held, or from the menu, so a double press to edit never leaves it.
    if !mods.shift && (mods.cmd || mods.ctrl)
        && let Some(href) = link
    {
        crate::files::open_with_system(&href);
    }
    let p = pick_for(sh, hit, mods.shift);
    let kind = match hit {
        Hit::Cell(at) => sh.doc.table.columns.get(at.c).map(|c| c.kind),
        _ => None,
    };
    let opens = match hit {
        Hit::Cell(at) if editable && plain && !was_open && matches!(kind, Some(Kind::Tag | Kind::Date)) => Some(at),
        _ => None,
    };
    let Some(sh) = sheet(s, w) else { return };
    sh.pick = Some(p);
    sh.dragging = Some(match hit {
        Hit::Cell(_) => Dragging::Cells,
        Hit::Row(_) => Dragging::Rows,
        Hit::Col(_) => Dragging::Cols,
    });
    sh.opens_on_click = opens;
    sh.float = None;
    focus_keys(s, w, tx);
}

/// The pointer moving with the button down (`move`): the block stretches to where it is, or
/// the line shows where what was picked up would land.
pub fn dragged_to(s: &mut State, w: Where, hit: Option<Hit>, drop: Option<Dropping>) {
    let Some(sh) = sheet(s, w) else { return };
    sh.opens_on_click = None;
    let Some(dragging) = sh.dragging else { return };
    if matches!(dragging, Dragging::MoveRows | Dragging::MoveCols) {
        sh.dropping = drop;
        return;
    }
    let (Some(hit), Some(p)) = (hit, sh.pick) else { return };
    let last = sh.last();
    let b = match (dragging, hit) {
        (Dragging::Rows, Hit::Row(r) | Hit::Cell(At { r, .. })) => At { r, c: last },
        (Dragging::Cols, Hit::Col(c) | Hit::Cell(At { c, .. })) => At { r: p.b.r, c },
        (Dragging::Cells, Hit::Cell(at)) => at,
        (Dragging::Cells, Hit::Row(r)) => At { r, c: p.b.c },
        _ => return,
    };
    if b != p.b {
        sh.pick = Some(Pick { b, ..p });
    }
}

/// The button up (`up`): what was picked up lands, or the press picks what it was on; a tag or
/// a date cell pressed once opens.
pub fn released(s: &mut State, w: Where, cancelled: bool, tx: &mut Tx) {
    let Some(sh) = sheet(s, w) else { return };
    let was = sh.dragging.take();
    let drop = sh.dropping.take();
    let pressed = sh.pressed_pick.take();
    let opens = sh.opens_on_click.take();
    if cancelled {
        return;
    }
    let sel = sh.sel();
    if let Some(d) = drop {
        match was {
            Some(Dragging::MoveRows) if !d.cols => move_rows_to(s, w, d.place),
            Some(Dragging::MoveCols) if d.cols => {
                if let Some(b) = sel {
                    move_cols_to(s, w, b.c0, b.c1, d.place, tx);
                }
            }
            _ => {}
        }
    } else if let Some(p) = pressed {
        sh.pick = Some(p);
    }
    let Some(sh) = s.tables.get(w) else { return };
    if let (Some(at), Some(p)) = (opens, sh.pick)
        && p.a == at
        && p.b == at
    {
        start_edit(s, w, at, None, tx);
    }
}

/// A right press (`onContext`): a header's menu, or the cell's at the pointer, what is under it
/// picked first unless it is in the block already.
pub fn context_at(s: &mut State, w: Where, hit: Hit, x: f32, y: f32, tx: &mut Tx) {
    end_by_press(s, w, tx);
    let Some(sh) = s.tables.get(w) else { return };
    if let Hit::Col(c) = hit {
        if let Some(sh) = sheet(s, w) {
            sh.float = Some(Float { x, y, col: Some(c), page_type: false, lit: None });
        }
        return;
    }
    let first = sh.first();
    let inside = sh.sel().is_some_and(|b| match hit {
        Hit::Row(r) => edit::in_box(&b, r, first),
        Hit::Cell(at) => edit::in_box(&b, at.r, at.c),
        Hit::Col(_) => false,
    });
    if !inside {
        let p = pick_for(sh, hit, false);
        if let Some(sh) = sheet(s, w) {
            sh.pick = Some(p);
        }
    }
    focus_keys(s, w, tx);
    if let Some(sh) = sheet(s, w) {
        sh.float = Some(Float { x, y, col: None, page_type: false, lit: None });
    }
}

/// A double press (`onDouble`): a cell edits, a row number opens its row.
pub fn double_at(s: &mut State, w: Where, hit: Hit, tx: &mut Tx) {
    match hit {
        Hit::Cell(at) => start_edit(s, w, at, None, tx),
        Hit::Row(r) => open_row(s, w, r, tx),
        Hit::Col(_) => {}
    }
}

/* ------------------------------ saving ------------------------------ */

/// The text waiting is written now (`flush`), `force` writing it whatever the disk holds now
/// (Save mine). Off the main thread, through the app's door; what it answers lands in `tend`.
pub fn flush(s: &mut State, w: Where, force: bool) {
    let Some(sh) = sheet(s, w) else { return };
    sh.due = None;
    let Some(text) = sh.unsaved.take() else { return };
    let payload = save_payload(sh, &text, force);
    let found = sh.found.clone();
    sh.saving = true;
    let landed = s.tables.landed.clone();
    // A test writes to nobody: what would go to the app is kept, and the answer is the one the
    // test set, else that it was written.
    #[cfg(test)]
    if s.tables.test_door {
        let answer = s.tables.answer.clone().unwrap_or_else(|| serde_json::json!({ "ok": true, "mtime": crate::clock() }));
        s.tables.wrote.push(payload);
        landed.lock().unwrap().push((w, text, found, answer));
        return;
    }
    let crate::source::Source::Door(door) = &s.source else {
        // A picture of the web window has no app to write to, and says so as the web's bridge
        // does when there is none behind it; `MERCURY_WROTE=<file>` keeps there what it would
        // have sent, to be held to what the web window's page sent for the same steps.
        if let Some(file) = std::env::var_os("MERCURY_WROTE") {
            let mut all: Vec<Value> = std::fs::read_to_string(&file).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
            all.push(payload);
            let _ = std::fs::write(&file, Value::Array(all).to_string());
            landed.lock().unwrap().push((w, text, found, serde_json::json!({ "ok": true, "mtime": crate::clock() })));
            return;
        }
        landed.lock().unwrap().push((w, text, found, serde_json::json!({ "ok": false, "error": "quit and reopen Mercury to save this file" })));
        return;
    };
    let Some((url, token)) = door.now() else {
        landed.lock().unwrap().push((w, text, found, serde_json::json!({ "ok": false, "error": "Mercury is not running, so this was not saved." })));
        return;
    };
    std::thread::spawn(move || {
        let answer = crate::source::call(&url, &token, "zero:write-doc", &payload).unwrap_or_else(|e| serde_json::json!({ "ok": false, "error": e }));
        landed.lock().unwrap().push((w, text, found, answer));
    });
}

/// What a save sends (`api.writeDoc`): the file as pressed, the words, and the time it had when
/// read (none to write it whatever the disk holds); from the big view, the thread too.
fn save_payload(sh: &Sheet, text: &str, force: bool) -> Value {
    let mut p = serde_json::json!({ "product": sh.product, "src": sh.src, "text": text, "mtime": if force { 0.0 } else { sh.mtime } });
    if let Some(thread) = &sh.thread {
        p["thread"] = Value::String(thread.clone());
    }
    p
}

/// What the app answered a save: the new time on disk, or why not (`flush`'s answer). The edit
/// a refusal was about waits again, so Try again and Save mine have it.
fn saved(s: &mut State, w: Where, text: String, found: String, answer: Value, tx: &Tx) {
    let ok = answer.get("ok").and_then(Value::as_bool).unwrap_or(false);
    let here = s.tables.get(w).is_some_and(|sh| sh.found == found);
    if !here {
        if !ok {
            let name = found.rsplit('/').next().unwrap_or(&found).to_string();
            let why = answer.get("error").and_then(Value::as_str).unwrap_or("");
            s.tell(js::trim(&format!("Your last edit to {name} was not saved. {why}")), tx);
        }
        return;
    }
    let Some(sh) = sheet(s, w) else { return };
    sh.saving = false;
    if ok {
        sh.mtime = answer.get("mtime").and_then(Value::as_f64).unwrap_or(0.0);
        sh.problem = None;
        return;
    }
    sh.unsaved.get_or_insert(text);
    let stale = answer.get("stale").and_then(Value::as_bool).unwrap_or(false);
    let error = if stale { "This file changed on disk since it opened, so your edit is not saved yet.".to_string() } else { answer.get("error").and_then(Value::as_str).unwrap_or("That could not be saved.").to_string() };
    sh.problem = Some(Problem { error, stale });
}

/// Load the new one: what waited is let go and the file read again (`onReload`).
pub fn reload(s: &mut State, w: Where) {
    let Some(sh) = sheet(s, w) else { return };
    sh.unsaved = None;
    sh.due = None;
    sh.problem = None;
    match w {
        Where::Pane => s.reread_beside(),
        Where::Big => s.reread_big(),
    }
    sync(s);
    // Read again even when the words are the same: the time on disk is the new one.
    if let Some(sh) = sheet(s, w) {
        sh.past.clear();
        sh.ahead.clear();
    }
}

/* ------------------------------ what the window tends ------------------------------ */

/// What the window tends while a sheet is open: the sheets kept in step with what is open, a
/// save due, the answers to saves, where the keys are, a box losing them, and scrolls asked for.
pub fn tend(s: &mut State, now: f64, tx: &mut Tx) -> bool {
    sync(s);
    let mut changed = draw::logos_landed();
    let landed: Vec<(Where, String, String, Value)> = s.tables.landed.lock().unwrap().drain(..).collect();
    for (w, text, found, answer) in landed {
        saved(s, w, text, found, answer, tx);
        changed = true;
    }
    for w in [Where::Pane, Where::Big] {
        let Some(sh) = s.tables.get(w) else { continue };
        if sh.due.is_some_and(|due| now >= due) {
            flush(s, w, false);
            changed = true;
        }
        let Some(sh) = s.tables.get(w) else { continue };
        if let Some(due) = sh.due {
            tx.wake_after(std::time::Duration::from_millis((due - now).max(0.0) as u64 + 1));
        }
        if sh.saving {
            tx.wake_after(std::time::Duration::from_millis(30));
        }
        changed |= tend_focus(s, w, tx);
    }
    scroll_asked(tx);
    changed
}

/// Where the keys are: a pick goes quiet when they leave the table (`holding`); a box of the
/// table that had them and lost them ends what it was for, as the web's `onBlur` does.
fn tend_focus(s: &mut State, w: Where, tx: &mut Tx) -> bool {
    let focused = tx.focused().map(str::to_string);
    let mine = |k: &str| focused.as_deref() == Some(w.key(k).as_str());
    let inside = focused.as_deref().is_some_and(|f| f.starts_with("tbl-") && f.ends_with(w.tail()));
    let Some(sh) = sheet(s, w) else { return false };
    let mut changed = false;
    if sh.holding == inside {
        sh.holding = !inside;
        changed = true;
    }
    // The cell's box.
    let editor = sh.editing.as_ref().map(|e| match &e.pop {
        None => ("edit", e.held),
        Some(Pop::Tag(t)) => (if t.page.is_some() { "tagname" } else { "tagfind" }, t.held || e.held),
        Some(Pop::Date(d)) => ("date", d.held),
    });
    if let Some((k, held)) = editor {
        let has = mine(k);
        if has && !held {
            if let Some(e) = sh.editing.as_mut() {
                e.held = true;
                match &mut e.pop {
                    Some(Pop::Tag(t)) => t.held = true,
                    Some(Pop::Date(d)) => d.held = true,
                    None => {}
                }
            }
        } else if !has && held && !mine("scroll") {
            // Left for somewhere else: the edit ends as the box's blur ends it.
            let e = sh.editing.as_ref().expect("an edit");
            let value = match &e.pop {
                None => sh.editable.then(|| e.draft.text.clone()),
                Some(Pop::Tag(_)) => None,
                Some(Pop::Date(d)) => {
                    let was = sh.value(At { r: e.r, c: e.c });
                    (js::trim(&d.words.text) != js::trim(was)).then(|| js::trim(&d.words.text).to_string())
                }
            };
            end_edit(s, w, value, Then::Stay, tx);
            changed = true;
        }
    }
    let Some(sh) = sheet(s, w) else { return changed };
    // The header's rename box: leaving it keeps the name.
    if sh.renaming.is_some() {
        if mine("rename") {
            sh.rename_held = true;
        } else if sh.rename_held {
            finish_rename(s, w, true, tx);
            changed = true;
        }
    }
    let Some(sh) = sheet(s, w) else { return changed };
    // A field of the row opened whole: leaving it writes it.
    if let Some(f) = sh.fields.as_ref() {
        let key = format!("field{}", f.c);
        if mine(&key) {
            if let Some(f) = sh.fields.as_mut() {
                f.held = true;
            }
        } else if f.held && f.pick.is_none() {
            let (c, v) = (f.c, f.draft.text.clone());
            sh.fields = None;
            write_field(s, w, c, v);
            changed = true;
        }
    }
    changed
}

/// A field of the row opened whole written (`onWrite`).
pub fn write_field(s: &mut State, w: Where, c: usize, v: String) {
    let Some(sh) = s.tables.get(w) else { return };
    let Some(r) = sh.sheet else { return };
    if sh.value(At { r, c }) == v {
        return;
    }
    write_at(s, w, vec![edit::Write { r, c, v }]);
}

/// The header's rename box closed: the column renamed, its looks following, unless dropped.
pub fn finish_rename(s: &mut State, w: Where, keep: bool, tx: &mut Tx) {
    let Some(sh) = sheet(s, w) else { return };
    let Some(i) = sh.renaming.take() else { return };
    sh.rename_held = false;
    let to = sh.rename.text.clone();
    sh.rename = TextEdit::new("");
    let d = sh.doc.clone();
    if keep {
        let t = edit::rename_column(&d.table, i, &to);
        let looks = edit::look_rename_column(&d.looks, &d.table.columns[i].name, &t.columns[i].name);
        change_columns(s, w, t, Some(i), Some(looks));
    } else {
        change_columns(s, w, d.table.clone(), Some(i), None);
    }
    focus_keys(s, w, tx);
}

/// A header made a box to rename its column, all its name selected.
pub fn begin_rename(s: &mut State, w: Where, col: usize) {
    let Some(sh) = sheet(s, w) else { return };
    let Some(name) = sh.doc.table.columns.get(col).map(|c| c.name.clone()) else { return };
    let mut rename = TextEdit::new(name);
    rename.anchor = 0;
    rename.focus = rename.text.len();
    rename.focused = true;
    sh.rename = rename;
    sh.rename_held = false;
    sh.renaming = Some(col);
}

/* ------------------------------ for the pane and the big view ------------------------------ */

/// The box a field's edits land in: the sheet's, or a spare one once it has gone.
pub(crate) fn edit_of(s: &mut State, w: Where, which: Box_) -> &mut TextEdit {
    let Sheets { pane, big, spare, .. } = &mut s.tables;
    let Some(sh) = (match w {
        Where::Pane => pane.as_mut(),
        Where::Big => big.as_mut(),
    }) else {
        return spare;
    };
    let found: Option<&mut TextEdit> = match which {
        Box_::Search => Some(&mut sh.search),
        Box_::FilterFind => Some(&mut sh.filter_find),
        Box_::Rename => Some(&mut sh.rename),
        Box_::Edit => sh.editing.as_mut().map(|e| &mut e.draft),
        Box_::TagFind => match sh.editing.as_mut().and_then(|e| e.pop.as_mut()) {
            Some(Pop::Tag(t)) => Some(&mut t.find),
            _ => None,
        },
        Box_::TagName => match sh.editing.as_mut().and_then(|e| e.pop.as_mut()) {
            Some(Pop::Tag(TagPick { page: Some((_, name)), .. })) => Some(name),
            _ => None,
        },
        Box_::Date => match sh.editing.as_mut().and_then(|e| e.pop.as_mut()) {
            Some(Pop::Date(d)) => Some(&mut d.words),
            _ => None,
        },
        Box_::Field => match sh.fields.as_mut() {
            Some(Field { pick: Some(Pop::Tag(t)), .. }) => Some(&mut t.find),
            Some(Field { pick: Some(Pop::Date(d)), .. }) => Some(&mut d.words),
            Some(Field { draft, .. }) => Some(draft),
            None => None,
        },
        Box_::FieldName => match sh.fields.as_mut().and_then(|f| f.pick.as_mut()) {
            Some(Pop::Tag(TagPick { page: Some((_, name)), .. })) => Some(name),
            _ => None,
        },
    };
    found.unwrap_or(spare)
}

/// Which of a sheet's boxes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Box_ {
    Search,
    FilterFind,
    Rename,
    Edit,
    TagFind,
    TagName,
    Date,
    Field,
    FieldName,
}

#[cfg(test)]
mod tests;
