//! The code a thread's run changed, as the web window shows it: the button in the head strip
//! with the figures (`codeButton` in `renderer/src/components/Focus.tsx`, `ThreadMenu.tsx`), and
//! the change itself in the pane beside the conversation (`CodeArtifact` in
//! `renderer/src/components/CodeArtifact.tsx`, drawn by `DocPane.tsx`): the files as a tree, or
//! a picker over the column in a pane of 800 points or less, and one running column of every
//! file, each under a head that stays at the top while its file is in view. The rules are
//! `rules/change.rs`; `docs/parity-code-changes.md` lists every behaviour and where it stands.
//!
//! What is read: the change without its rows, for the button and the chips on a run's steps,
//! when the conversation opens, as its row changes, every four seconds while a run is up and for
//! two minutes after one ended; the whole change once each time it opens; a file's own lines
//! when a gap between two hunks is pressed, off this Mac's disk as the window reads every file.
//! What is written: a typed line, on ⌘S, through the engine (`zero:code-file`,
//! `zero:save-code-file`), as the web window writes it.
//!
//! The rows are built only where they can be seen (`El::virtual_rows`), a list for each file,
//! which is what the web window's first screenful, its idle batches and its skipped slices are
//! there for: a change of thousands of rows opens at once.

use std::borrow::Cow;
use std::cell::Cell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;

use rune::{Cursor, El, GrowthSpec, Icon, KeyEvent, Latest, PopupMenu, ScrollSpec, Span, TextEdit, TextStyle, Tx, Waker, col, el, icon, row, text, text_input};
use serde_json::{Value, json};

use crate::rules::change::{self as rules, Change, ChangedFile, Mark, TreeRow};
use crate::rules::code::{self, Kind as Tok};
use crate::theme::*;
use crate::{State, View};

/* ------------------------------ looks ------------------------------ */

/// `--code-ground`: under the change, its head and every file's head.
pub fn ground() -> rune::Color {
    rgb(34, 32, 31)
}
/// `--code-plus` and `--code-minus`.
pub fn plus() -> rune::Color {
    rgb(123, 201, 142)
}
pub fn minus() -> rune::Color {
    rgb(226, 143, 130)
}
/// `--code-plus-bed`, `--code-minus-bed`: under an added and a removed line.
fn plus_bed() -> rune::Color {
    rgba(123, 201, 142, 0.13)
}
fn minus_bed() -> rune::Color {
    rgba(226, 143, 130, 0.12)
}
/// The words that changed: the line's own colour at 32% (`color-mix(… 32%, transparent)`).
fn plus_words() -> rune::Color {
    rgba(123, 201, 142, 0.32)
}
fn minus_words() -> rune::Color {
    rgba(226, 143, 130, 0.32)
}
/// `--code-live-bed`: under the line being typed in.
fn live_bed() -> rune::Color {
    rgba(255, 255, 255, 0.055)
}

/// A row of code: 12 points of mono on lines of 1.55 (`.code-row`), as Chromium keeps 18.6.
fn row_tall() -> f32 {
    kept(18.6)
}
fn code_font() -> TextStyle {
    mono(12.0, row_tall())
}
/// A character of it across.
const ADVANCE: f32 = 7.2;
/// The gutter: the number (44, ten before the mark) and the mark (22), and the air after a line.
const NUM_W: f32 = 44.0;
const MARK_W: f32 = 22.0;
const LINE_PAD: f32 = 14.0;
/// A file's head (`.code-file-head`: nine over and under 16 points of name, and its line).
const HEAD_H: f32 = 35.0;
/// The tree beside the column (`.code-tree`).
const TREE_W: f32 = 232.0;
/// Up to this pane width the tree is a picker (`@container artifact (max-width: 800px)`).
const NARROW: f32 = 800.0;

static CODE_MARK: Icon = Icon {
    name: "chg-code",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"><path d="m8.5 7-5 5 5 5"/><path d="m15.5 7 5 5-5 5"/></g>"#,
};
/// A file head's chevron and the picker's (`M4 6l4 4 4-4`).
static FOLD_MARK: Icon = Icon {
    name: "chg-fold",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6l4 4 4-4"/></g>"#,
};
/// A folder's chevron in the tree (`M6 4l4 4-4 4`), turned down while it is open.
static FOLDER_MARK: Icon = Icon {
    name: "chg-folder",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M6 4l4 4-4 4"/></g>"#,
};
/// The page mark a step's file chip wears (`.did-file-mark`: nine by eleven, a corner folded).
/// Drawn in a square of eleven, a point in from either side.
static PAGE_MARK: Icon = Icon {
    name: "chg-page",
    view: 11.0,
    body: r#"<path fill="currentColor" d="M2.5 0H7.12L10 2.86V9.5A1.5 1.5 0 0 1 8.5 11H2.5A1.5 1.5 0 0 1 1 9.5V1.5A1.5 1.5 0 0 1 2.5 0Z"/>"#,
};

/* ------------------------------ what is kept ------------------------------ */

/// The lines typed into one change, by file, hunk and row.
type Typed = BTreeMap<String, BTreeMap<usize, BTreeMap<usize, String>>>;

/// A change as drawn: its files in the tree's order, the tree, and each hunk's changed words.
#[derive(Clone)]
pub struct Shown {
    pub change: Change,
    pub files: Vec<ChangedFile>,
    tree: Vec<TreeRow>,
    words: Vec<Vec<Vec<Option<(usize, usize)>>>>,
}

impl Shown {
    fn new(change: Change) -> Shown {
        let files = rules::files_in_tree_order(&change.files);
        let tree = rules::tree_rows(&files);
        let words = files.iter().map(|f| f.hunks.iter().map(|h| rules::word_changes(&h.rows)).collect()).collect();
        Shown { change, files, tree, words }
    }
}

/// One opening of a change: on its way, read, or refused.
enum Opened {
    Reading,
    Read(Rc<Shown>),
    Failed(String),
}

/// One thing in a file's running list: a gap between two hunks, a line a gap opened onto, or a
/// row of a hunk.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Entry {
    Gap { hunk: usize, lines: u32 },
    Opened { hunk: usize, at: usize },
    Row { hunk: usize, row: usize },
}

/// What the open change shows, from its opening until it closes.
pub struct Here {
    key: String,
    product: String,
    src: String,
    /// The tree's folders shut (`shut`), the gaps opened (`gaps`), by file path and the hunk under
    /// each, numbered from where it starts.
    shut: HashSet<String>,
    gaps: HashMap<(String, usize), (u32, Vec<String>)>,
    /// Each file's running list.
    entries: Vec<Rc<Vec<Entry>>>,
    /// The file at the top of the column, by its place in the tree's order (`at`).
    pub at: usize,
    /// A jump landing on a file, and until when it may take (twenty frames, as the web window's).
    holding: Option<(usize, f64)>,
    /// The keys walk the tree, after a press on one of its rows (the focus on the tree).
    pub tree_keys: bool,
    /// The file to open at, from a step's chip, until the change has been read.
    start_at: Option<String>,
    /// Where each file's block and the column were drawn, in window points.
    tops: Vec<f32>,
    body_top: f32,
    /// The line being typed in, by file, hunk and row, and its words.
    editing: Option<(usize, usize, usize)>,
    pub edit: TextEdit,
    /// Where the pointer last moved over a line, in the line's own points, for where a press puts
    /// the caret.
    pointer: (f32, f32),
}

/// The code of the threads: what has been read and what is kept while the window runs.
#[derive(Default)]
pub struct Changes {
    /// Each thread's change without its rows, by `product/id`, as last answered.
    summaries: HashMap<String, Option<Rc<Change>>>,
    /// What the open thread's summary was last asked at: the thread, its row's change, its run,
    /// and when.
    asked: Option<(String, f64, Option<f64>, f64)>,
    summary_read: Latest<(String, Result<Value, String>)>,
    /// Each change opened, by `product\nsrc`.
    opened: HashMap<String, Opened>,
    whole_read: Latest<(String, Result<Value, String>)>,
    /// Kept while the window runs, as the web window keeps them while it runs (`foldsFor`,
    /// `KEPT`): the files folded and the lines typed in each change.
    folds: HashMap<String, HashSet<String>>,
    typed: HashMap<String, Typed>,
    /// A save on its way, and what each file it saved came to.
    saves: Latest<(String, Vec<Result<(String, Vec<(usize, usize, String)>, String), String>>)>,
    pub here: Option<Here>,
    /// The marks over a change given the whole workspace: when the pointer last moved, on the
    /// window's clock, and whether it is on them or the corner they live in.
    pub chrome_at: f64,
    pub reaching: bool,
    spare: TextEdit,
}

impl Changes {
    pub fn set_waker(&self, wake: Waker) {
        self.summary_read.set_waker(wake.clone());
        self.whole_read.set_waker(wake.clone());
        self.saves.set_waker(wake);
    }
    /// The editing line's words, for its field.
    fn edit(&mut self) -> &mut TextEdit {
        match self.here.as_mut() {
            Some(here) => &mut here.edit,
            // The field is drawn only while a change is open; a late edit lands nowhere.
            None => &mut self.spare,
        }
    }
}

thread_local! {
    /// Where the column is scrolled, as it last said, and how far it shows.
    static COLUMN: Cell<(f32, f32, f32)> = const { Cell::new((0.0, 0.0, 0.0)) };
}
#[cfg(test)]
thread_local! {
    /// How many times the change has been drawn, for a measurement.
    static VIEWS: Cell<usize> = const { Cell::new(0) };
}
const BODY: &str = "chg-body";
const TREE: &str = "chg-tree";
const PICKER: &str = "chg-picker";
const EDIT: &str = "chg-edit";

fn key_of(product: &str, src: &str) -> String {
    format!("{product}\n{src}")
}

fn door(s: &State) -> Option<(String, String)> {
    match &s.source {
        crate::source::Source::Door(door) => door.now(),
        _ => None,
    }
}

/// The project's own folders a file of a change may be in (`artifactRoots`).
fn product_roots(s: &State, product: &str) -> Vec<String> {
    let Some(p) = s.snap.products.iter().find(|p| p.slug == product) else { return Vec::new() };
    ["repoPath", "dir"].iter().filter_map(|f| p.rest.get(*f).and_then(Value::as_str)).filter(|d| !d.is_empty()).map(str::to_string).collect()
}

/* ------------------------------ the summary: the button and the chips ------------------------------ */

/// The open thread's change without its rows, when the engine has answered with one.
pub fn summary(s: &State, item: &crate::data::Item) -> Option<Rc<Change>> {
    s.changes.summaries.get(&format!("{}/{}", item.product, item.id)).cloned().flatten()
}

/// Whether a thread is one of the person's own agent sessions, which no run of ours changed.
fn an_agent(item: &crate::data::Item) -> bool {
    item.rest.get("agent").is_some_and(|a| !a.is_null() && a != &Value::Bool(false))
}

/// When the run on a thread started, while one is up.
fn run_started(s: &State, item: &crate::data::Item) -> Option<f64> {
    s.snap.supervisor.running.iter().find(|r| r.item_id == item.id && r.product == item.product).map(|r| r.rest.get("startedAt").and_then(Value::as_f64).unwrap_or(0.0))
}

/// The answer to a summary landing.
fn summary_from(answer: &Value) -> Option<Rc<Change>> {
    (answer.get("ok") == Some(&Value::Bool(true))).then(|| answer.get("change").and_then(Change::from_value)).flatten().map(Rc::new)
}

/// Asks for the open thread's summary when it is owed (`Focus.tsx`'s effect and its timer,
/// `ItemThread.tsx`'s beat): when the thread opens, its row changes or a run starts on it; every
/// four seconds while a run is up or its folder may still move; and on the engine's word while
/// a run is up. A picture's fixture answers it at once.
fn ask_for_the_summary(s: &mut State, pushed: bool, now: f64, tx: &mut Tx) -> bool {
    let Some(item) = s.open_item().cloned() else { return false };
    if an_agent(&item) {
        return false;
    }
    let key = format!("{}/{}", item.product, item.id);
    let changed = item.updated_at.unwrap_or(0.0);
    let run = run_started(s, &item);
    let fresh = s.changes.asked.as_ref().is_none_or(|(k, c, r, _)| *k != key || *c != changed || *r != run);
    // A picture is answered by its fixture, the shape the page was answered.
    if let Some(answer) = s.fixture.get("threads").and_then(|t| t.get(&key)).and_then(|t| t.get("zero:code-change")).cloned() {
        if fresh {
            s.changes.asked = Some((key.clone(), changed, run, now));
            s.changes.summaries.insert(key, summary_from(&answer));
            return true;
        }
        return false;
    }
    let settling = run.is_some() || rules::folder_may_still_move(item.wrote_at("result"), now);
    let due = s.changes.asked.as_ref().is_some_and(|(_, _, _, at)| now - at >= rules::FIGURES_AGAIN_MS);
    if settling {
        tx.wake_after(std::time::Duration::from_millis(rules::FIGURES_AGAIN_MS as u64));
    }
    if !(fresh || (settling && (due || (pushed && run.is_some())))) {
        return false;
    }
    let Some((url, token)) = door(s) else { return false };
    // A new reading of the thread starts with nothing drawn until its answer, as the web
    // window's `setFigures(null)` does.
    if fresh {
        s.changes.summaries.remove(&key);
    }
    s.changes.asked = Some((key.clone(), changed, run, now));
    let ask = json!({ "product": item.product, "src": rules::change_path_for(&item.id), "summary": true });
    s.changes.summary_read.request((url, token, key, ask), |(url, token, key, ask)| (key, crate::source::call(&url, &token, "zero:code-change", &ask)));
    fresh
}

/* ------------------------------ the button ------------------------------ */

/// The strip's button, for a thread whose run left a change with something in it: the code
/// mark and the figures, lit while the change is open beside the thread; a press opens it at
/// its top, or puts it away when it is open (`openChange`).
pub fn button(s: &State, item: &crate::data::Item) -> Option<El> {
    let change = summary(s, item)?;
    let figures = rules::figures_from(&change)?;
    let src = rules::change_path_for(&item.id);
    let open = s.beside().is_some_and(|d| d.src == src);
    let said = rules::signed(figures.plus, figures.minus).into_iter().map(|(added, words)| text(words).font(TextStyle::new(12.5, 560.0).mono().lh(16.0).tracking(-0.1875)).color(if added { plus() } else { minus() }));
    let button = row().key("thread-code").h(32.0).pl(9.0).pr(12.0).gap(8.0).items_center().radius(6.0).border(1.0, rule()).border_layout();
    // Pressed while open (`aria-pressed`): the text's ink and the strong wash, pointer or not.
    let button = if open { button.fill(crate::feel::wash_strong()) } else { button };
    Some(
        crate::feel::strip_button(button, true)
            .child(icon(&CODE_MARK, 16.0).color(if open { ink() } else { dim() }).group_color(ink()))
            .child(row().gap(6.0).items_center().children(said))
            .label("View code changes")
            .on_press(move |s: &mut State| {
                crate::feel::pressed(s);
                if s.beside().is_some_and(|d| d.src == src) {
                    s.close_beside();
                } else {
                    s.open_change(&src, None);
                }
            }),
    )
}

/* ------------------------------ opening and reading ------------------------------ */

impl State {
    /// Opens a thread's change beside the conversation, at a file when a step's chip names one
    /// (`onOpenDoc(changePathFor(id), path)`). The change already open is not opened again: the
    /// chip only moves it to the file, as a press of the same chip again does.
    pub fn open_change(&mut self, src: &str, at: Option<&str>) {
        if !self.beside().is_some_and(|d| d.src == src) {
            self.open_beside(src, false);
        }
        if let (Some(here), Some(at)) = (self.changes.here.as_mut(), at) {
            here.start_at = Some(at.to_string());
        }
    }
    /// A change opening in the pane: read again from the engine, its view begun afresh, its folds
    /// and typed lines kept from before (`DocPane`'s effect, `CodeArtifact`'s mount).
    pub(crate) fn change_opened(&mut self, product: &str, src: &str) {
        let key = key_of(product, src);
        self.changes.here = Some(Here {
            key: key.clone(),
            product: product.to_string(),
            src: src.to_string(),
            shut: HashSet::new(),
            gaps: HashMap::new(),
            entries: Vec::new(),
            at: 0,
            holding: None,
            tree_keys: false,
            start_at: None,
            tops: Vec::new(),
            body_top: 0.0,
            editing: None,
            edit: TextEdit::new(""),
            pointer: (0.0, 0.0),
        });
        self.changes.reaching = false;
        // A picture's fixture says what the engine answered, or nothing yet (`null`).
        if let Some(fixed) = self.fixture.get("codeChanges").and_then(|c| c.get(src)).cloned() {
            let opened = if fixed.is_null() { Opened::Reading } else { opened_from(&fixed) };
            self.changes.opened.insert(key, opened);
            self.lay_out_the_change();
            return;
        }
        match door(self) {
            Some((url, token)) => {
                self.changes.opened.insert(key.clone(), Opened::Reading);
                let ask = json!({ "product": product, "src": src });
                self.changes.whole_read.request((url, token, key, ask), |(url, token, key, ask)| (key, crate::source::call(&url, &token, "zero:code-change", &ask)));
            }
            None => {
                self.changes.opened.insert(key, Opened::Failed("That change could not be read.".to_string()));
            }
        }
    }
    /// The open change as drawn, once read.
    fn shown(&self) -> Option<Rc<Shown>> {
        let here = self.changes.here.as_ref()?;
        match self.changes.opened.get(&here.key)? {
            Opened::Read(shown) => Some(shown.clone()),
            _ => None,
        }
    }
    /// Each file's running list, again: after the change is read, a gap opens or a file folds.
    fn lay_out_the_change(&mut self) {
        let Some(shown) = self.shown() else { return };
        let Some(here) = self.changes.here.as_mut() else { return };
        here.entries = shown.files.iter().map(|f| Rc::new(entries_of(f, &here.gaps))).collect();
        here.tops = vec![f32::INFINITY; shown.files.len()];
    }
}

/// What the engine's answer to the whole change opens onto.
fn opened_from(answer: &Value) -> Opened {
    if answer.get("ok") == Some(&Value::Bool(true))
        && let Some(change) = answer.get("change").and_then(Change::from_value)
    {
        return Opened::Read(Rc::new(Shown::new(change)));
    }
    Opened::Failed(answer.get("error").and_then(Value::as_str).filter(|e| !e.is_empty()).unwrap_or("That change could not be read.").to_string())
}

/// A file's running list: each hunk's rows, and between two hunks the gap, or the lines it was
/// opened onto.
fn entries_of(file: &ChangedFile, gaps: &HashMap<(String, usize), (u32, Vec<String>)>) -> Vec<Entry> {
    let mut out = Vec::new();
    for (h, hunk) in file.hunks.iter().enumerate() {
        if h > 0 {
            match gaps.get(&(file.path.clone(), h)) {
                Some((_, lines)) => out.extend((0..lines.len()).map(|at| Entry::Opened { hunk: h, at })),
                None => {
                    if let Some(lines) = rules::lines_skipped(file.hunks.get(h - 1), Some(hunk)) {
                        out.push(Entry::Gap { hunk: h, lines });
                    }
                }
            }
        }
        out.extend((0..hunk.rows.len()).map(|row| Entry::Row { hunk: h, row }));
    }
    out
}

/// What the window tends for the code: the open thread's summary, the change read landing, a
/// file to open at once it has, the line typed in let go when its field lost the keys, and what
/// a save came to.
pub fn tend(s: &mut State, now: f64, pushed: bool, tx: &mut Tx) -> bool {
    let mut changed = false;
    if matches!(s.view, View::Thread { .. }) {
        changed |= ask_for_the_summary(s, pushed, now, tx);
    }
    if let Some((key, answer)) = s.changes.summary_read.take() {
        if let Ok(answer) = answer {
            s.changes.summaries.insert(key, summary_from(&answer));
        }
        changed = true;
    }
    if let Some((key, answer)) = s.changes.whole_read.take() {
        let opened = match answer {
            Ok(answer) => opened_from(&answer),
            Err(why) => Opened::Failed(why),
        };
        s.changes.opened.insert(key, opened);
        s.lay_out_the_change();
        changed = true;
    }
    // The change left the pane: what it showed goes with it.
    let still = s.changes.here.as_ref().is_some_and(|h| s.beside().is_some_and(|d| h.key == key_of(&d.product, &d.src)));
    if s.changes.here.is_some() && !still {
        s.changes.here = None;
        changed = true;
    }
    // A step's chip asked for a file: it is jumped to once the change is read.
    if let (Some(shown), Some(at)) = (s.shown(), s.changes.here.as_ref().and_then(|h| h.start_at.clone())) {
        if let Some(i) = shown.files.iter().position(|f| f.path == at) {
            jump(s, i, tx);
        }
        if let Some(here) = s.changes.here.as_mut() {
            here.start_at = None;
        }
        changed = true;
    }
    // A line whose field lost the keys is let go, its words kept.
    if s.changes.here.as_ref().is_some_and(|h| h.editing.is_some() && !h.edit.focused) {
        stop_editing(s);
        changed = true;
    }
    if let Some((key, done)) = s.changes.saves.take() {
        saved(s, &key, done, tx);
        changed = true;
    }
    changed
}

/* ------------------------------ the view ------------------------------ */

/// The change in the pane's card, under its head: the skeleton while it is read, the reason
/// when it cannot be, else the files and the column. `w` by `h`.
pub fn view(s: &State, w: f32, h: f32) -> El {
    #[cfg(test)]
    VIEWS.with(|v| v.set(v.get() + 1));
    let Some(here) = s.changes.here.as_ref() else { return skeleton(w, h) };
    match s.changes.opened.get(&here.key) {
        Some(Opened::Read(shown)) => artifact(s, here, shown, w, h),
        Some(Opened::Failed(why)) => col().w(w).padding(40.0).child(text(why.as_str()).font(TextStyle::new(13.5, 400.0).lh(20.0)).color(muted()).wrap()),
        _ => skeleton(w, h),
    }
}

/// The place a document holds while it arrives (`PreviewLoading`): a title bar, two lines, a
/// block, two lines, breathing over 2.4 seconds, still under Reduce motion.
fn skeleton(w: f32, h: f32) -> El {
    use crate::rules::motion::eased_stops;
    let pad = (0.05 * w).clamp(24.0, 56.0);
    let inner = w - 2.0 * pad;
    let bar = |wide: f32, tall: f32| el().w(inner * wide).h(tall).radius(3.0).fill(rgba(239, 235, 231, 0.07));
    static STOPS: std::sync::OnceLock<Vec<(f32, f32)>> = std::sync::OnceLock::new();
    let stops = STOPS.get_or_init(|| eased_stops(&[(0.0, 0.65), (0.5, 1.0), (1.0, 0.65)], rune::Easing::EASE_IN_OUT, 24));
    let block = (0.2 * h).clamp(80.0, 180.0);
    let sheet = col()
        .w(w)
        .padding(pad)
        .child(bar(0.48, 20.0).mb(24.0))
        .child(bar(0.9, 9.0).mb(12.0))
        .child(bar(0.64, 9.0).mb(12.0))
        .child(bar(1.0, block).my(28.0).opacity(0.55))
        .child(bar(0.9, 9.0).mb(12.0))
        .child(bar(0.64, 9.0).mb(12.0))
        .cycle(rune::Cycle { period: 2.4, phase: 0.0, stops, still: 1.0, fps: 30.0 });
    el().size(w, h).fill(rgb(40, 38, 37)).clip_rounded(0.0).child(sheet)
}

/// The files and the column side by side, or the picker over the column in a pane of 800
/// points or less (the pane is the card and its twelve points of air either side).
fn artifact(s: &State, here: &Here, shown: &Rc<Shown>, w: f32, h: f32) -> El {
    if w + 24.0 <= NARROW {
        let picker_h = 8.0 + 36.0 + 8.0;
        col().size(w, h).child(picker(s, here, shown, w)).child(column(s, here, shown, w, h - picker_h))
    } else {
        row().size(w, h).child(tree(s, here, shown, h)).child(column(s, here, shown, w - TREE_W, h))
    }
}

/// The lines typed into the open change.
fn typed_here<'a>(s: &'a State, here: &Here) -> Option<&'a Typed> {
    s.changes.typed.get(&here.key)
}
fn unsaved(s: &State, here: &Here, path: &str) -> bool {
    typed_here(s, here).and_then(|t| t.get(path)).is_some_and(|byh| byh.values().any(|rows| !rows.is_empty()))
}
fn folded(s: &State, here: &Here, path: &str) -> bool {
    s.changes.folds.get(&here.key).is_some_and(|f| f.contains(path))
}

/// "+5" in green and "−7" in red, six apart, each only when not zero (`Counts`).
fn counts(plus_n: u64, minus_n: u64, line: f32) -> El {
    let font = mono(10.5, line);
    row().gap(6.0).items_center().children((plus_n > 0).then(|| text(format!("+{plus_n}")).font(font).color(plus()))).children((minus_n > 0).then(|| text(format!("−{minus_n}")).font(font).color(minus())))
}

/// The tree: its head and its rows, which scroll on their own (`.code-tree`).
fn tree(s: &State, here: &Here, shown: &Rc<Shown>, h: f32) -> El {
    let change = &shown.change;
    let small = sans(11.5, 400.0);
    let mut words = vec![text(rules::change_summary(change)).font(small).color(dim())];
    let stands = rules::standing_label(change);
    if !stands.is_empty() {
        words.push(text(stands).font(small).color(muted()));
    }
    let took = rules::took_label(change);
    if !took.is_empty() {
        words.push(text(format!("in {took}")).font(small).color(muted()));
    }
    let head = row().w(TREE_W - 1.0).pt(4.0).pr(12.0).pb(10.0).pl(18.0).flex_wrap().gaps(2.0, 8.0).children(words);
    let shown_rows = rules::visible_rows(&shown.tree, &here.shut);
    let current = shown.files.get(here.at).map(|f| f.path.as_str());
    let rows = shown_rows.iter().map(|r| tree_row(s, here, r, current));
    let list = col().key(TREE).w(TREE_W - 1.0).grow(1.0).min_h(0.0).scroll(ScrollSpec { stick_bottom: false }).child(col().w(TREE_W - 1.0).pb(8.0).children(rows));
    col().w(TREE_W).h(h).border_sides([0.0, 1.0, 0.0, 0.0], edge()).border_layout().child(head).child(list)
}

/// A row of the tree: a folder, which shuts and opens what is under it, or a file, which the
/// column jumps to; washed under the pointer, at once both ways.
fn tree_row(s: &State, here: &Here, r: &TreeRow, current: Option<&str>) -> El {
    match r {
        TreeRow::Folder { label, depth, key, files, .. } => {
            let open = !here.shut.contains(key);
            let key = key.clone();
            row()
                .w(TREE_W - 1.0)
                .h(26.0)
                .pl(12.0 + 13.0 * *depth as f32)
                .pr(12.0)
                .gap(8.0)
                .items_center()
                .hover(crate::feel::wash())
                .cursor(Cursor::Pointer)
                .child(icon(&FOLDER_MARK, 13.0).color(muted()).rotate(if open { std::f32::consts::FRAC_PI_2 } else { 0.0 }))
                .child(row().grow(1.0).min_w(0.0).child(text(label.as_str()).font(sans(12.5, 400.0)).color(muted()).truncate()))
                .child(text(files.to_string()).font(mono(10.5, 13.5)).color(muted()))
                .on_press(move |s: &mut State| {
                    if let Some(here) = s.changes.here.as_mut() {
                        if !here.shut.remove(&key) {
                            here.shut.insert(key.clone());
                        }
                        here.tree_keys = true;
                        here.holding = None;
                    }
                })
        }
        TreeRow::File { label, depth, path, plus: p, minus: m } => {
            let at = current == Some(path.as_str());
            let fold = folded(s, here, path);
            let line = kept(1.65 * 12.5);
            let name = text(label.as_str()).font(sans(12.5, 400.0).lh(line)).color(if at { ink() } else { dim() }).truncate().selectable(true).cursor(Cursor::Pointer);
            let end = if unsaved(s, here, path) {
                text("•").font(mono(14.0, 14.0)).color(plus()).help("Not saved yet. Press ⌘S.")
            } else {
                counts(*p, *m, kept(1.65 * 10.5))
            };
            let target = path.clone();
            let mut r = row()
                .key(format!("t:{path}"))
                .w(TREE_W - 1.0)
                .h(5.0 + line + 5.0)
                .pl(12.0 + 13.0 * *depth as f32)
                .pr(12.0)
                .gap(8.0)
                .items_center()
                .help(path.as_str())
                .cursor(Cursor::Pointer)
                .child(row().grow(1.0).min_w(0.0).opacity(if fold { 0.45 } else { 1.0 }).child(name))
                .child(end.opacity(if fold { 0.45 } else { 1.0 }))
                .on_press_tx(move |s: &mut State, tx| {
                    let Some(i) = s.shown().and_then(|sh| sh.files.iter().position(|f| f.path == target)) else { return };
                    if let Some(here) = s.changes.here.as_mut() {
                        here.tree_keys = true;
                    }
                    jump(s, i, tx);
                });
            // The file at the top of the column: the strong wash and the selection's bar down its
            // left edge (`.code-file.is-at`).
            if at {
                r = r.fill(crate::feel::wash_strong()).child(el().absolute(0.0, 0.0).size(2.0, 5.0 + line + 5.0).fill(accent()));
            } else {
                r = r.hover(crate::feel::wash());
            }
            r
        }
    }
}

/// The picker that stands for the tree in a narrow pane (`.code-file-pick`): the file at the
/// top of the column in mono, its chevron twelve in from its edge; a press opens the system's
/// menu of every file, each with " · Unsaved" when it has unsaved lines, and a pick jumps to it.
fn picker(s: &State, here: &Here, shown: &Rc<Shown>, w: f32) -> El {
    let current = shown.files.get(here.at).or(shown.files.first());
    let said = current.map(|f| format!("{}{}", f.path, if unsaved(s, here, &f.path) { " · Unsaved" } else { "" })).unwrap_or_default();
    let field = row()
        .key(PICKER)
        .w(w - 24.0)
        .h(36.0)
        .pl(8.0)
        .pr(34.0)
        .items_center()
        .radius(6.0)
        .fill(ground())
        .border(1.0, edge())
        .border_layout()
        .label("File to review")
        .child(row().grow(1.0).min_w(0.0).child(text(said).font(mono(12.0, 18.0)).color(dim()).truncate()))
        .child(el().absolute(w - 24.0 - 12.0 - 13.0, 11.5).child(icon(&FOLD_MARK, 13.0).color(muted())))
        .on_press_tx(|s: &mut State, tx| {
            let (Some(shown), Some(here)) = (s.shown(), s.changes.here.as_ref()) else { return };
            let options: Vec<(String, String)> = shown.files.iter().map(|f| (f.path.clone(), format!("{}{}", f.path, if unsaved(s, here, &f.path) { " · Unsaved" } else { "" }))).collect();
            let on = shown.files.get(here.at).map(|f| f.path.clone()).unwrap_or_default();
            tx.popup_menu(PopupMenu::picker(options, &on), PICKER, |s: &mut State, picked: Option<&str>, tx: &mut Tx| {
                let Some(path) = picked else { return };
                if let Some(i) = s.shown().and_then(|sh| sh.files.iter().position(|f| f.path == path)) {
                    jump(s, i, tx);
                }
            });
        });
    row().w(w).px(12.0).py(8.0).child(field)
}

/// The running column: every file's block, the rows built only where they can be seen, the
/// words selectable across lines and copied as the web window copies them.
fn column(s: &State, here: &Here, shown: &Rc<Shown>, w: f32, h: f32) -> El {
    let blocks = shown.files.iter().enumerate().map(|(f, file)| block(s, here, shown, f, file, w));
    let lines = col().w(w).children(blocks);
    col().key(BODY)
        .w(w)
        .h(h.max(1.0))
        .scroll(ScrollSpec { stick_bottom: false })
        .on_scroll_offset(|at| COLUMN.with(|c| c.set((at.offset[1], at.viewport[1], at.max[1]))))
        .on_drawn_layout_tx(|s: &mut State, r, tx| {
            if let Some(here) = s.changes.here.as_mut() {
                here.body_top = r[1];
            }
            tx.live_only();
        })
        .on_wheel(|s: &mut State, _, _| {
            if let Some(here) = s.changes.here.as_mut() {
                here.holding = None;
            }
            false
        })
        .on_background_press(|s: &mut State| {
            if let Some(here) = s.changes.here.as_mut() {
                here.tree_keys = false;
                here.holding = None;
            }
        })
        .selection_group(true)
        .selection_copy(copy)
        // A selection reaching past the line being typed in takes the keys from it, or the next
        // keystroke would land in one line while many are selected.
        .on_selection_change(|s: &mut State, change: &rune::SelectionChange, tx: &mut Tx| {
            if change.selection().is_some_and(|sel| lines_selected(sel) > 1) && s.changes.here.as_ref().is_some_and(|h| h.editing.is_some()) {
                note_edit(s);
                stop_editing(s);
                tx.blur();
            }
        })
        .child(lines)
}

/// What ⌘C takes from a selection across the change's lines: the lines joined by newlines, the
/// first and last clipped where it starts and ends, the removed lines left out unless they are
/// all that was selected (`linesToCopy`, `copiedText`). Inside one line, the line's own copy.
fn copy(sel: &rune::TextSelection) -> Option<String> {
    let touched: Vec<&rune::SelectedText> = sel.texts().iter().filter(|t| t.is_selected() && t.key().is_some_and(|k| k.starts_with("l:") || k.starts_with("o:"))).collect();
    let removed: Vec<bool> = touched.iter().map(|t| t.key().is_some_and(|k| k.ends_with(":-"))).collect();
    let plan = rules::lines_to_copy(&removed)?;
    // Each text hands over its selected part, already clipped where the selection starts and
    // ends, so the parts are copied whole.
    let parts: Vec<String> = plan.into_iter().map(|i| touched[i].text().to_string()).collect();
    Some(rules::copied_text(&parts, 0, i64::MAX))
}

/// How many of the change's lines a selection takes.
fn lines_selected(sel: &rune::TextSelection) -> usize {
    sel.texts().iter().filter(|t| t.is_selected() && t.key().is_some_and(|k| k.starts_with("l:") || k.starts_with("o:"))).count()
}

/// One file's block: its head, which stays at the top while the file is in view, and its rows.
fn block(s: &State, here: &Here, shown: &Rc<Shown>, f: usize, file: &ChangedFile, w: f32) -> El {
    let fold = folded(s, here, &file.path);
    let head = file_head(s, here, f, file, fold, w).sticky_top(0.0);
    let mut block = col().key(format!("chg-file-{f}")).w(w).pb(10.0).child(head).on_drawn_layout_tx(move |s: &mut State, r, tx| section_moved(s, f, r[1], tx));
    if !fold {
        let count = here.entries.get(f).map_or(0, |e| e.len());
        let cols = ((w - NUM_W - MARK_W - LINE_PAD) / ADVANCE).floor().max(1.0) as usize;
        let shown = shown.clone();
        // The keys change only when the change, a gap or the file's list does: a commit then
        // skips asking each of thousands of rows for its key.
        let version = (here.key.clone(), f, here.entries.get(f).map_or(0, |e| Rc::as_ptr(e) as usize), count);
        let rows = col()
            .key(format!("chg-rows-{f}"))
            .w(w)
            .growth(GrowthSpec { spring: rune::Spring::NONE, ..GrowthSpec::default() })
            .virtual_rows_version(version)
            .virtual_rows(
                count,
                row_tall(),
                move |s: &State, i| Cow::Owned(s.changes.here.as_ref().and_then(|h| h.entries.get(f)).and_then(|e| e.get(i)).map_or_else(|| format!("x{f}-{i}"), |e| entry_key(f, e))),
                move |s: &State, i| entry_view(s, &shown, f, i, w, cols),
            );
        block = block.child(rows);
    }
    block
}

fn entry_key(f: usize, e: &Entry) -> String {
    match e {
        Entry::Gap { hunk, .. } => format!("g{f}-{hunk}"),
        Entry::Opened { hunk, at } => format!("o{f}-{hunk}-{at}"),
        Entry::Row { hunk, row } => format!("r{f}-{hunk}-{row}"),
    }
}

/// A file's head (`.code-file-head`): its chevron, turned a quarter back while folded, its name,
/// its folder, its counts and, while it has unsaved lines, "Unsaved". A press folds or opens
/// it; folding the file whose head is stuck at the top keeps the head where it was.
fn file_head(s: &State, here: &Here, f: usize, file: &ChangedFile, fold: bool, w: f32) -> El {
    let at = here.at == f;
    let (folder, name) = match file.path.rsplit_once('/') {
        Some((folder, name)) => (folder.to_string(), name.to_string()),
        None => (String::new(), file.path.clone()),
    };
    let path = file.path.clone();
    let mut head = row()
        .key(format!("chg-head-{f}"))
        .w(w)
        .h(HEAD_H)
        .pl(16.0)
        .pr(14.0)
        .gap(10.0)
        .items_center()
        .fill(ground())
        .border_sides([0.0, 0.0, 1.0, 0.0], if fold { rgba(0, 0, 0, 0.0) } else { edge() })
        .border_layout()
        .cursor(Cursor::Pointer)
        .help(if fold { "Show this file" } else { "Fold this file" })
        .child(
            icon(&FOLD_MARK, 13.0)
                .color(muted())
                .ml(-4.0)
                .rotate(if fold { -std::f32::consts::FRAC_PI_2 } else { 0.0 })
                .rotate_motion(rune::Curve::tween(0.12, rune::Easing::EASE)),
        )
        .child(text(name).font(sans(12.5, 600.0)).color(if at { ink() } else { dim() }))
        .child(row().shrink().min_w(0.0).child(text(folder).font(sans(11.5, 400.0)).color(muted()).truncate()))
        .child(counts(file.plus, file.minus, 13.5));
    if unsaved(s, here, &file.path) {
        head = head.child(el().grow(1.0)).child(
            row().gap(6.0).items_center().help("Not saved yet. Press ⌘S.").child(el().size(6.0, 6.0).radius(3.0).fill(ink())).child(text("Unsaved").font(sans(11.0, 400.0).tracking(0.22)).color(ink())),
        );
    }
    head.on_press_tx(move |s: &mut State, tx| fold_file(s, f, &path, tx))
}

/// Folds a file shut from its head, or opens it again (`foldFile`).
fn fold_file(s: &mut State, f: usize, path: &str, tx: &mut Tx) {
    let Some(key) = s.changes.here.as_ref().map(|h| h.key.clone()) else { return };
    if let Some(here) = s.changes.here.as_mut() {
        here.tree_keys = false;
        here.holding = None;
        // A head stuck at the top stays where it is as its file shuts under it.
        if here.tops.get(f).is_some_and(|t| *t < here.body_top) {
            tx.keep_on_screen(format!("chg-head-{f}"));
        }
    }
    if s.changes.here.as_ref().is_some_and(|h| h.editing.is_some_and(|(ef, ..)| ef == f)) {
        stop_editing(s);
    }
    let folds = s.changes.folds.entry(key).or_default();
    if !folds.remove(path) {
        folds.insert(path.to_string());
    }
}

/// One entry of a file's list, built only while it can be seen.
fn entry_view(s: &State, shown: &Rc<Shown>, f: usize, i: usize, w: f32, cols: usize) -> El {
    let Some(here) = s.changes.here.as_ref() else { return el() };
    let Some(file) = shown.files.get(f) else { return el() };
    let Some(entries) = here.entries.get(f) else { return el() };
    let Some(entry) = entries.get(i).copied() else { return el() };
    // What stands over it in its hunk's block: the block's eight, and the hunk's own two over
    // its first row; the hunk's two under its last.
    let first_in_block = i == 0 || !matches!((entries[i - 1], entry), (Entry::Gap { .. } | Entry::Opened { .. }, _) | (Entry::Row { .. }, Entry::Row { .. }));
    let first_in_block = first_in_block && !matches!((i.checked_sub(1).map(|p| entries[p]), entry), (Some(Entry::Opened { hunk: a, .. }), Entry::Opened { hunk: b, .. }) if a == b);
    match entry {
        // The block's eight over it and the line's own two either side, as padding: a virtual
        // row's own margins are not laid out.
        Entry::Gap { hunk, lines } => col().w(w).pt(10.0).pb(2.0).child(gap_line(f, hunk, lines, w)),
        Entry::Opened { hunk, at } => {
            let (from, lines) = here.gaps.get(&(file.path.clone(), hunk)).cloned().unwrap_or_default();
            let line = lines.get(at).cloned().unwrap_or_default();
            let top = if at == 0 { 8.0 } else { 0.0 };
            code_row(Mark::Same, Some(from + at as u32), &line, None, format!("o:{f}:{hunk}:{at}:="), w).pt(top)
        }
        Entry::Row { hunk, row: r } => {
            let h = &file.hunks[hunk];
            let (mark, said) = (&h.rows[r].0, &h.rows[r].1);
            let typed = typed_here(s, here).and_then(|t| t.get(&file.path)).and_then(|byh| byh.get(&hunk)).and_then(|rows| rows.get(&r));
            let line = if *mark == Mark::Removed { said.clone() } else { typed.cloned().unwrap_or_else(|| said.clone()) };
            // An edited line is no longer the line its changed words were found in.
            let span = if line == *said { shown.words.get(f).and_then(|w| w.get(hunk)).and_then(|w| w.get(r)).copied().flatten() } else { None };
            let num = h.nums.as_ref().and_then(|n| n.get(r)).and_then(|(old, new)| if *mark == Mark::Removed { *old } else { *new });
            let tag = match mark {
                Mark::Removed => "-",
                Mark::Added => "+",
                Mark::Same => "=",
            };
            let editing = here.editing == Some((f, hunk, r));
            let el = if editing { editing_row(here, *mark, num, w) } else { code_row(*mark, num, &line, span, format!("l:{f}:{hunk}:{r}:{tag}"), w) };
            let el = if *mark != Mark::Removed && !editing { editable(el, f, hunk, r, &line, cols) } else { el };
            let before_gap = i > 0 && matches!(entries[i - 1], Entry::Gap { .. } | Entry::Opened { .. });
            let top = if r == 0 { if before_gap { 2.0 } else if first_in_block { 10.0 } else { 2.0 } } else { 0.0 };
            let bottom = if r + 1 == h.rows.len() { 2.0 } else { 0.0 };
            // The hunk's own top, two points over its first row, is where ] and [ land it.
            let el = if r == 0 { col().w(w).pt(top - 2.0).child(col().key(format!("h{f}-{hunk}")).w(w).pt(2.0).child(el)) } else { el };
            // The hunk's two under its last row lie outside its bed.
            if bottom > 0.0 { col().w(w).pb(bottom).child(el) } else { el }
        }
    }
}

/// The colours of a line, and its changed words on the stronger bed, as byte ranges.
fn spans_of(line: &str, span: Option<(usize, usize)>, mark: Mark) -> Vec<Span> {
    let words_bg = if mark == Mark::Removed { minus_words() } else { plus_words() };
    let (from, to) = span.filter(|(a, b)| a < b).unwrap_or((usize::MAX, usize::MAX));
    let mut out = Vec::new();
    let (mut byte, mut ch) = (0usize, 0usize);
    for (kind, piece) in code::tokenize(line) {
        let colour = crate::viewer::colour_of(kind);
        // Cut where the changed words start and end, as `marked` does.
        let chars: Vec<(usize, char)> = piece.char_indices().collect();
        let n = chars.len();
        let cuts = [0, from.saturating_sub(ch).min(n), to.saturating_sub(ch).min(n), n];
        for k in 0..3 {
            let (a, b) = (cuts[k].max(if k == 0 { 0 } else { cuts[k - 1] }), cuts[k + 1].max(cuts[k]));
            if a >= b {
                continue;
            }
            let start = byte + chars[a].0;
            let end = byte + chars.get(b).map_or(piece.len(), |c| c.0);
            let mut sp = Span::new(start, end);
            if let Some(c) = colour {
                sp = sp.color(c);
            }
            if kind == Tok::Comment {
                sp = sp.italic();
            }
            if k == 1 && from != usize::MAX {
                sp = sp.bg(words_bg, 0.0);
            }
            if sp.color.is_some() || sp.bg.is_some() || sp.italic {
                out.push(sp);
            }
        }
        byte += piece.len();
        ch += n;
    }
    out
}

/// A row of code: its number in the file, its mark, and its line on the bed of its kind
/// (`.code-row`). The line's words can be selected; the gutter's cannot.
fn code_row(mark: Mark, num: Option<u32>, line: &str, span: Option<(usize, usize)>, key: String, w: f32) -> El {
    let bed = match mark {
        Mark::Added => plus_bed(),
        Mark::Removed => minus_bed(),
        Mark::Same => rgba(0, 0, 0, 0.0),
    };
    let (sign, sign_colour) = match mark {
        Mark::Added => ("+", plus()),
        Mark::Removed => ("−", minus()),
        Mark::Same => ("", muted()),
    };
    let quiet = mark == Mark::Removed || key.starts_with("o:");
    let words = text(line.to_string()).font(code_font()).color(ink()).spans(spans_of(line, span, mark)).span_radius(2.0).wrap_anywhere().key(key);
    row()
        .w(w)
        .items_start()
        .fill(bed)
        .child(row().w(NUM_W).pr(10.0).justify_end().child(pen(text(num.map(|n| n.to_string()).unwrap_or_default()).font(code_font().features("'tnum' 1")).color(muted()), 12.0)))
        .child(row().w(MARK_W).justify_center().child(text(sign).font(code_font()).color(sign_colour)))
        // A line that takes typing is the I-beam across its whole width (`.cr-live .code-line {
        // cursor: text }`); a removed one only over its words, as Rune finds selectable words by
        // their box.
        .child(row().grow(1.0).min_w(0.0).pr(LINE_PAD).child(if quiet { pen(readable(words), 12.0).min_w(0.0) } else { pen(readable(words), 12.0).grow(1.0).min_w(0.0) }))
}

/// A line in the file now takes typing as an editor's does: a press on its words puts the caret
/// where it landed (`contentEditable` on `.cr-live .code-line`).
fn editable(row_el: El, f: usize, hunk: usize, r: usize, line: &str, cols: usize) -> El {
    let line = line.to_string();
    row_el
        .on_pointer_move_local(|s: &mut State, p, tx| {
            // In the line's own points, past the gutter.
            if let Some(here) = s.changes.here.as_mut() {
                here.pointer = (p.x - NUM_W - MARK_W, p.y);
            }
            tx.live_only();
        })
        .on_press_tx(move |s: &mut State, tx| {
            let Some((x, y)) = s.changes.here.as_ref().map(|h| h.pointer) else { return };
            // A press in the margin is a selection's, never a caret's.
            if x < 0.0 {
                if let Some(here) = s.changes.here.as_mut() {
                    here.tree_keys = false;
                    here.holding = None;
                }
                return;
            }
            let col = caret_at(&line, cols, x, y);
            start_editing(s, f, hunk, r, col, tx);
        })
}

/// Where a press lands in a line `cols` characters wide, as the line wraps (`pre-wrap` with
/// `overflow-wrap: anywhere`): after a space that ends a visual line, or anywhere in a word too
/// long for one. In characters.
pub fn caret_at(line: &str, cols: usize, x: f32, y: f32) -> usize {
    let chars: Vec<char> = line.chars().collect();
    let mut starts = vec![0usize];
    let mut start = 0;
    while chars.len() - start > cols && cols > 0 {
        let window = &chars[start..(start + cols + 1).min(chars.len())];
        let next = match window.iter().rposition(|c| *c == ' ' || *c == '-') {
            Some(at) if at > 0 => start + at + 1,
            _ => start + cols,
        };
        start = next;
        starts.push(start);
    }
    let visual = ((y / row_tall()).floor().max(0.0) as usize).min(starts.len() - 1);
    let end = starts.get(visual + 1).copied().unwrap_or(chars.len());
    let col = (x / ADVANCE).round().max(0.0) as usize;
    (starts[visual] + col).min(if visual + 1 < starts.len() { end.saturating_sub(1).max(starts[visual]) } else { end })
}

/// The row being typed in: its line a field on the faint bed, the caret in the added green.
fn editing_row(here: &Here, mark: Mark, num: Option<u32>, w: f32) -> El {
    let bed = if mark == Mark::Added { plus_bed() } else { rgba(0, 0, 0, 0.0) };
    let (sign, sign_colour) = if mark == Mark::Added { ("+", plus()) } else { ("", muted()) };
    let field = text_input(&here.edit, code_font())
        .key(EDIT)
        .wrap_anywhere()
        .color(ink())
        .caret(plus(), rgba(110, 165, 255, 0.3))
        .spans(spans_of(&here.edit.text, None, mark))
        .grow(1.0)
        .min_w(0.0)
        .on_edit(|s: &mut State| s.changes.edit())
        .on_input_change(|s: &mut State, _| note_edit(s))
        .on_input_paste(|s: &mut State, pasted, _| {
            // A row is one line of its file: what is pasted comes in as plain words, its line
            // breaks turned to spaces.
            let flat: String = pasted.text().split(['\r', '\n']).filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" ");
            s.changes.edit().insert(&flat);
            note_edit(s);
            true
        })
        .on_input_key(in_the_line);
    row()
        .w(w)
        .items_start()
        .fill(bed)
        .child(row().w(NUM_W).pr(10.0).justify_end().child(pen(text(num.map(|n| n.to_string()).unwrap_or_default()).font(code_font().features("'tnum' 1")).color(muted()), 12.0)))
        .child(row().w(MARK_W).justify_center().child(text(sign).font(code_font()).color(sign_colour)))
        .child(row().grow(1.0).min_w(0.0).pr(LINE_PAD).fill(live_bed()).cursor(Cursor::Text).child(pen(field, 12.0)))
}

/// Puts the caret in a line, `col` characters in.
fn start_editing(s: &mut State, f: usize, hunk: usize, r: usize, col: usize, tx: &mut Tx) {
    let Some(shown) = s.shown() else { return };
    let Some(here) = s.changes.here.as_ref() else { return };
    let Some(file) = shown.files.get(f) else { return };
    let said = file.hunks.get(hunk).and_then(|h| h.rows.get(r)).map(|(_, t)| t.clone()).unwrap_or_default();
    let line = typed_here(s, here).and_then(|t| t.get(&file.path)).and_then(|b| b.get(&hunk)).and_then(|rows| rows.get(&r)).cloned().unwrap_or(said);
    let byte = line.char_indices().nth(col).map_or(line.len(), |(b, _)| b);
    let Some(here) = s.changes.here.as_mut() else { return };
    here.tree_keys = false;
    here.holding = None;
    here.editing = Some((f, hunk, r));
    here.edit = TextEdit::new(line);
    here.edit.set_caret(byte);
    here.edit.focused = true;
    tx.focus(EDIT, false);
}

/// The line's words are kept as they are typed, never on disk unsaved (`noteEdit`).
fn note_edit(s: &mut State) {
    let Some(shown) = s.shown() else { return };
    let Some(here) = s.changes.here.as_ref() else { return };
    let Some((f, hunk, r)) = here.editing else { return };
    let (key, words) = (here.key.clone(), here.edit.text.clone());
    let Some(path) = shown.files.get(f).map(|f| f.path.clone()) else { return };
    s.changes.typed.entry(key).or_default().entry(path).or_default().entry(hunk).or_default().insert(r, words);
}

/// The line lets go of the keys, its words kept.
fn stop_editing(s: &mut State) {
    if let Some(here) = s.changes.here.as_mut() {
        here.editing = None;
        here.edit.focused = false;
    }
}

/// Every line that takes typing, in the order drawn: the rows in the file now, across hunks and
/// files, folded files left out.
fn editable_lines(s: &State, shown: &Shown, here: &Here) -> Vec<(usize, usize, usize)> {
    let mut out = Vec::new();
    for (f, file) in shown.files.iter().enumerate() {
        if folded(s, here, &file.path) {
            continue;
        }
        for (h, hunk) in file.hunks.iter().enumerate() {
            for r in rules::editable_rows(&hunk.rows) {
                out.push((f, h, r));
            }
        }
    }
    out
}

/// The keys inside the line being typed in (`CodeLine`'s `onKeyDown` and the pane's own): Return
/// refused, Escape lets go of the line, ⌘S saves, and an arrow off the line's edge moves the
/// caret to the line above or below (`caretStep`).
fn in_the_line(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> bool {
    use rune::Key;
    // While a selection takes more than one line, nothing typed changes the code: an edit
    // there would change one line of several (`editsAcrossLines`). The selection stays for ⌘C.
    let typed = match &e.key {
        Key::Backspace => "Backspace".to_string(),
        Key::Delete => "Delete".to_string(),
        Key::Enter => "Enter".to_string(),
        Key::Text(t) => t.clone(),
        _ => String::new(),
    };
    if tx.text_selection().is_some_and(|sel| lines_selected(sel) > 1) && rules::edits_across_lines(&typed, e.mods.cmd, e.mods.ctrl, e.mods.alt) {
        return true;
    }
    match &e.key {
        Key::Enter if !e.mods.cmd => return true,
        Key::Escape => {
            stop_editing(s);
            tx.blur();
            return true;
        }
        Key::Command('s') if e.mods.cmd => {
            stop_editing(s);
            tx.blur();
            save(s, tx);
            return true;
        }
        _ => {}
    }
    let name = match e.key {
        Key::Up => "ArrowUp",
        Key::Down => "ArrowDown",
        Key::Left => "ArrowLeft",
        Key::Right => "ArrowRight",
        _ => return false,
    };
    let Some(here) = s.changes.here.as_ref() else { return false };
    let text = &here.edit.text;
    let col = text[..here.edit.focus.min(text.len())].chars().count();
    let Some((dir, to_col)) = rules::caret_step(name, col, text.chars().count(), e.mods.shift, e.mods.cmd) else { return false };
    let Some(shown) = s.shown() else { return false };
    let lines = editable_lines(s, &shown, here);
    let Some(at) = here.editing.and_then(|ed| lines.iter().position(|l| *l == ed)) else { return false };
    let next = if dir > 0 { lines.get(at + 1) } else { at.checked_sub(1).and_then(|a| lines.get(a)) };
    let Some(&(f, hunk, r)) = next else { return matches!(e.key, Key::Up | Key::Down) };
    note_edit(s);
    start_editing(s, f, hunk, r, to_col, tx);
    tx.scroll_into_view(format!("r{f}-{hunk}-{r}"));
    true
}

/// The quiet line between two hunks that do not touch (`.code-skip`): its count between two
/// hairlines, brighter under the pointer; a press reads the file and opens the lines in place.
fn gap_line(f: usize, hunk: usize, lines: u32, w: f32) -> El {
    let hair = |grow: bool| {
        let line = |c: rune::Color| el().h(1.0).fill(c);
        let pair = el().h(1.0).child(line(edge()).w_full().absolute(0.0, 0.0).group_opacity(1.0, 0.0)).child(line(rule()).w_full().absolute(0.0, 0.0).group_opacity(0.0, 1.0));
        if grow { pair.grow(1.0) } else { pair.w(44.0) }
    };
    row()
        .w(w)
        .pt(3.0)
        .pb(3.0)
        .pr(14.0)
        .gap(10.0)
        .items_center()
        .hover_group()
        .cursor(Cursor::Pointer)
        .help("Show these lines")
        .child(hair(false))
        .child(text(format!("{lines} lines not shown")).font(mono(11.0, 14.0)).color(muted()).group_color(dim()))
        .child(hair(true))
        .on_press_tx(move |s: &mut State, tx| open_gap(s, f, hunk, tx))
}

/// Opens a gap: the file as it is on disk now, the lines it stood for read and placed, or the
/// reason they cannot be (`openGap`, `hiddenLines`).
fn open_gap(s: &mut State, f: usize, hunk: usize, tx: &mut Tx) {
    let Some(shown) = s.shown() else { return };
    let Some(here) = s.changes.here.as_ref() else { return };
    let Some(file) = shown.files.get(f) else { return };
    let roots = product_roots(s, &here.product);
    let read = rules::find_file(&shown.change, &roots, &file.path, |p| std::path::Path::new(p).is_file()).and_then(read_code_file);
    let placed = read.map_err(str::to_string).and_then(|text| rules::hidden_lines(&text, &file.hunks[hunk - 1], &file.hunks[hunk]).map_err(str::to_string));
    match placed {
        Ok(lines) => {
            if let Some(here) = s.changes.here.as_mut() {
                here.gaps.insert((file.path.clone(), hunk), lines);
                here.tree_keys = false;
            }
            s.lay_out_the_change();
        }
        Err(why) => s.tell(&why, tx),
    }
}

/// One file of a change, read as the engine reads it for the pane (`readCodeFile`).
fn read_code_file(path: String) -> Result<String, &'static str> {
    let meta = std::fs::metadata(&path).map_err(|_| "That file is no longer there.")?;
    if meta.len() > rules::MAX_CODE_BYTES {
        return Err("That file is too big to edit here.");
    }
    std::fs::read_to_string(&path).map_err(|_| "That file is no longer there.")
}

/* ------------------------------ where the column is ------------------------------ */

/// A file's block moved: where it now stands says which file is at the top of the column, which
/// the tree, the picker and the open mark follow, and the tree keeps that file's row in view
/// (`fileOnScreen`). While a jump lands, it waits for the file it asked for.
fn section_moved(s: &mut State, f: usize, top: f32, tx: &mut Tx) {
    let Some(here) = s.changes.here.as_mut() else { return };
    if here.tops.len() <= f {
        here.tops.resize(f + 1, f32::INFINITY);
    }
    here.tops[f] = top;
    let offsets: Vec<f32> = here.tops.iter().map(|t| t - here.body_top).collect();
    if let Some((want, until)) = here.holding {
        let landed = offsets.get(want).is_some_and(|o| o.abs() <= 1.0);
        if landed || s.at > until {
            here.holding = None;
        } else {
            tx.live_only();
            return;
        }
    }
    let at = rules::file_on_screen(&offsets, 0.0);
    if at == here.at {
        tx.live_only();
        return;
    }
    here.at = at;
    tx.scroll_into_view_in(TREE, format!("t:{}", s.shown().and_then(|sh| sh.files.get(at).map(|f| f.path.clone())).unwrap_or_default()));
}

/// Jumps the column to a file: its block at the top, held there while the rows under it settle,
/// let go by the person's own wheel, press or key (`goTo`).
fn jump(s: &mut State, f: usize, tx: &mut Tx) {
    let Some(path) = s.shown().and_then(|sh| sh.files.get(f).map(|f| f.path.clone())) else { return };
    let until = s.at + 20.0 / 60.0;
    let Some(here) = s.changes.here.as_mut() else { return };
    here.at = f;
    // Twenty frames, on the window's own clock, as the web window's `put` gives up after.
    here.holding = Some((f, until));
    tx.scroll_key_to(format!("chg-file-{f}"), 0.0);
    tx.scroll_into_view_in(TREE, format!("t:{path}"));
}

/// The file at the top of the column now, for the pane's open mark.
pub fn current_file(s: &State) -> Option<(String, String)> {
    let shown = s.shown()?;
    let here = s.changes.here.as_ref()?;
    let file = shown.files.get(here.at).or(shown.files.first())?;
    let roots = product_roots(s, &here.product);
    let found = rules::find_file(&shown.change, &roots, &file.path, |p| std::path::Path::new(p).is_file()).ok();
    Some((file.path.clone(), found.unwrap_or_default()))
}

/// The open mark over a change: the file at the top of the column, in the app that owns it
/// (`openArtifactExternally({ src: fileAt })`); a file nowhere on this Mac says so.
pub fn open_outside(s: &mut State, tx: &mut Tx) {
    let Some((path, found)) = current_file(s) else { return };
    if found.is_empty() {
        let project = s.changes.here.as_ref().and_then(|h| s.snap.products.iter().find(|p| p.slug == h.product)).map(|p| p.name.clone()).unwrap_or_else(|| "this project".to_string());
        s.tell(&format!("{path} is not in {project}. Nothing was opened."), tx);
        return;
    }
    crate::files::open_with_system(&found);
}

/* ------------------------------ the keys ------------------------------ */

/// The keys while a change is open beside the conversation, before the conversation's own:
/// ⌘S saves; the keys the change answers (`CHANGE_OWNS`) are its own and never the page's: ] and
/// [ go to the next change and the one before, the arrows walk the tree after a press on it and
/// otherwise scroll the column with the page keys, Home and End, and J and K do nothing.
pub fn key(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> Option<bool> {
    use rune::Key;
    s.shown()?;
    if !s.beside().is_some_and(|d| d.kind() == crate::rules::pane::Kind::Change) {
        return None;
    }
    if let Some(here) = s.changes.here.as_mut() {
        here.holding = None;
    }
    if e.mods.cmd && e.key == Key::Command('s') {
        save(s, tx);
        return Some(true);
    }
    if e.mods.ctrl || e.mods.alt {
        return None;
    }
    let name: String = match &e.key {
        Key::Up => "ArrowUp".into(),
        Key::Down => "ArrowDown".into(),
        Key::Left => "ArrowLeft".into(),
        Key::Right => "ArrowRight".into(),
        Key::PageUp => "PageUp".into(),
        Key::PageDown => "PageDown".into(),
        Key::Home => "Home".into(),
        Key::End => "End".into(),
        Key::Text(t) if !e.mods.cmd => t.clone(),
        _ => return None,
    };
    if !rules::change_owns_key(&name) {
        return None;
    }
    let shown = s.shown()?;
    let (top, view, max) = COLUMN.with(Cell::get);
    if (name == "]" || name == "[") && !e.mods.cmd {
        let starts = hunk_starts(s, &shown, top);
        if let Some(to) = rules::next_change(&starts.iter().map(|(_, y)| *y).collect::<Vec<_>>(), top, name == "]")
            && let Some((key, _)) = starts.iter().find(|(_, y)| *y == to)
        {
            tx.scroll_key_to(key.clone(), HEAD_H);
        }
        return Some(true);
    }
    let tree_keys = s.changes.here.as_ref().is_some_and(|h| h.tree_keys);
    if tree_keys && !e.mods.cmd {
        let here = s.changes.here.as_ref()?;
        let shown_rows: Vec<String> = rules::visible_rows(&shown.tree, &here.shut).into_iter().filter_map(|r| if let TreeRow::File { path, .. } = r { Some(path) } else { None }).collect();
        let all: Vec<String> = shown.files.iter().map(|f| f.path.clone()).collect();
        let current = shown.files.get(here.at).map(|f| f.path.as_str());
        if let Some(to) = rules::step_in_tree(&name, current, &shown_rows, &all)
            && let Some(i) = all.iter().position(|p| *p == to)
        {
            jump(s, i, tx);
            return Some(true);
        }
    }
    if let Some(to) = rules::code_scroll(&name, 19.0, view, top, max, e.mods.cmd) {
        tx.scroll_to_y(BODY, to);
    }
    // J, K, ← and → are the change's, and do nothing.
    Some(true)
}

/// Where each hunk starts in the column, less the head that stays over it, by the key ] and [
/// land on: off the last frame where its rows were drawn, else counted from its file's block.
fn hunk_starts(s: &State, shown: &Shown, top: f32) -> Vec<(String, f32)> {
    let Some(here) = s.changes.here.as_ref() else { return Vec::new() };
    let body = rune::memo::window_layout(BODY).map_or(here.body_top, |r| r[1]);
    let mut out = Vec::new();
    for (f, file) in shown.files.iter().enumerate() {
        if folded(s, here, &file.path) {
            continue;
        }
        let block = here.tops.get(f).copied().filter(|t| t.is_finite()).map(|t| t - body + top);
        let mut counted = HEAD_H;
        let entries = here.entries.get(f).cloned().unwrap_or_default();
        for (i, e) in entries.iter().enumerate() {
            if let Entry::Row { hunk, row: 0 } = e {
                let key = format!("h{f}-{hunk}");
                let at = rune::memo::window_layout(&key).map(|r| r[1] - body + top).or_else(|| block.map(|b| b + counted + if i == 0 { 8.0 } else { 0.0 }));
                if let Some(at) = at {
                    out.push((key, at - HEAD_H));
                }
            }
            counted += match e {
                Entry::Gap { .. } => 24.0 + 8.0,
                Entry::Opened { at: 0, .. } => row_tall() + 8.0,
                Entry::Row { row: 0, .. } => row_tall() + 10.0,
                _ => row_tall(),
            };
        }
    }
    out
}

/* ------------------------------ saving ------------------------------ */

/// ⌘S: every file with lines typed into it, one at a time, read as it is now, its typed lines put
/// back where the run left them and written, each reported in one line (`saveAll`). Off the main
/// thread, through the engine, which alone writes a project's files.
fn save(s: &mut State, tx: &mut Tx) {
    let Some(shown) = s.shown() else { return };
    let Some(here) = s.changes.here.as_ref() else { return };
    let key = here.key.clone();
    let Some(typed) = s.changes.typed.get(&key).cloned() else { return };
    let dirty: Vec<(String, Vec<(usize, Vec<rules::Row>, BTreeMap<usize, String>)>)> = typed
        .iter()
        .filter(|(_, byh)| byh.values().any(|r| !r.is_empty()))
        .filter_map(|(path, byh)| {
            let file = shown.files.iter().find(|f| &f.path == path)?;
            Some((path.clone(), byh.iter().filter(|(_, r)| !r.is_empty()).filter_map(|(h, r)| Some((*h, file.hunks.get(*h)?.rows.clone(), r.clone()))).collect()))
        })
        .collect();
    if dirty.is_empty() || s.changes.saves.is_pending() {
        return;
    }
    let Some((url, token)) = door(s) else {
        s.tell("quit and reopen Mercury to save this file", tx);
        return;
    };
    let (product, src) = (here.product.clone(), here.src.clone());
    s.changes.saves.request((url, token, key, product, src, dirty), |(url, token, key, product, src, dirty)| {
        let mut done = Vec::new();
        for (path, hunks) in dirty {
            let read = crate::source::call(&url, &token, "zero:code-file", &json!({ "product": product, "src": src, "path": path }));
            let answer = match read {
                Ok(a) if a.get("ok") == Some(&Value::Bool(true)) && a.get("text").is_some_and(Value::is_string) => a,
                Ok(a) => {
                    done.push(Err(a.get("error").and_then(Value::as_str).map_or_else(|| format!("{path} could not be opened, so nothing was saved."), str::to_string)));
                    continue;
                }
                Err(why) => {
                    done.push(Err(why));
                    continue;
                }
            };
            let text = answer["text"].as_str().unwrap_or_default();
            let mtime = answer.get("mtime").and_then(Value::as_f64).unwrap_or(0.0);
            let splices: Vec<rules::Splice> = hunks.iter().map(|(_, rows, edits)| rules::Splice { rows, edits }).collect();
            let (spliced, lines) = match rules::apply_splices(text, &splices) {
                Ok(out) => out,
                Err(why) => {
                    done.push(Err(why.to_string()));
                    continue;
                }
            };
            let typed: Vec<(usize, usize, String)> = hunks.iter().flat_map(|(h, _, edits)| edits.iter().map(move |(r, t)| (*h, *r, t.clone()))).collect();
            if spliced == text {
                done.push(Ok((path, typed, String::new())));
                continue;
            }
            match crate::source::call(&url, &token, "zero:save-code-file", &json!({ "product": product, "src": src, "path": path, "text": spliced, "mtime": mtime })) {
                Ok(a) if a.get("ok") == Some(&Value::Bool(true)) => {
                    let said = rules::saved_line(&path, lines);
                    done.push(Ok((path, typed, said)));
                }
                Ok(a) => done.push(Err(a.get("error").and_then(Value::as_str).map_or_else(|| format!("{path} could not be saved."), str::to_string))),
                Err(why) => done.push(Err(why)),
            }
        }
        (key, done)
    });
}

/// What a save came to: each refusal said, the saved files' rows taking their saved lines and
/// their typed lines let go, and the saved files named in one line.
fn saved(s: &mut State, key: &str, done: Vec<Result<(String, Vec<(usize, usize, String)>, String), String>>, tx: &mut Tx) {
    let mut said = Vec::new();
    for result in done {
        match result {
            Err(why) => s.tell(&why, tx),
            Ok((path, typed, words)) => {
                if let Some(Opened::Read(shown)) = s.changes.opened.get_mut(key) {
                    let shown = Rc::make_mut(shown);
                    if let Some(file) = shown.files.iter_mut().find(|f| f.path == path) {
                        for (h, r, t) in &typed {
                            if let Some(row) = file.hunks.get_mut(*h).and_then(|h| h.rows.get_mut(*r)) {
                                row.1 = t.clone();
                            }
                        }
                    }
                }
                if let Some(t) = s.changes.typed.get_mut(key) {
                    t.remove(&path);
                }
                if !words.is_empty() {
                    said.push(words);
                }
            }
        }
    }
    if !said.is_empty() {
        s.tell(&said.join(" "), tx);
    }
}

/* ------------------------------ a step's file ------------------------------ */

/// The file a run's step changed as a chip into the change, when the thread's change holds it
/// (`.did-file`): the page mark and the path the line showed, quiet until the pointer is on it;
/// a press opens the change at that file. `raw` is the path the tool wrote, `shown` what the
/// line says of it.
pub fn chip(s: &State, item: &crate::data::Item, raw: &str, shown: &str, font: TextStyle) -> Option<El> {
    let change = summary(s, item)?;
    let held = rules::file_in_change(change.files.iter().map(|f| f.path.as_str()), raw)?;
    let src = rules::change_path_for(&item.id);
    let words = if shown.is_empty() { held.clone() } else { shown.to_string() };
    let target = held.clone();
    Some(
        row()
            .min_w(0.0)
            .shrink()
            .gap(7.0)
            .items_center()
            .px(6.0)
            .py(1.0)
            .radius(6.0)
            .border(1.0, rgba(0, 0, 0, 0.0))
            .border_layout()
            .hover_group()
            .hover(crate::feel::wash())
            .cursor(Cursor::Pointer)
            .help(format!("Open {held} in the code"))
            .child(el().absolute(0.0, 0.0).w_full().h_full().radius(6.0).border(1.0, edge()).group_opacity(0.0, 1.0))
            .child(icon(&PAGE_MARK, 11.0).mx(-1.0).color(muted()).group_color(dim()))
            // Chromium sets these words half a point higher on their line of 19.2 than Rune does.
            .child(row().min_w(0.0).shrink().child(text(words).font(font).color(muted()).group_color(ink()).truncate().mt(-0.5).mb(0.5)))
            .on_press(move |s: &mut State| s.open_change(&src, Some(&target))),
    )
}

/* ------------------------------ the whole workspace ------------------------------ */

/// The marks over a change given the whole workspace, or in a card too narrow for a side
/// (`.workspace-artifact-header .doc-marks` while `[data-artifact-layout="focus"]`): fixed 52
/// points from the window's top and 26 from its right, on the surface's glass with a line and a
/// shadow, shown while the pointer has moved in the last 1.8 seconds or rests on them or the
/// corner they live in (`chromeIsUp`, `CHROME_HOLD`, `CHROME_REACH`), coming and going over 0.16
/// seconds. The frame's corners turned inward put the change back beside the conversation, or
/// put it down where there is no room beside; close does the same.
///
/// Over a change they lie under the change's own sticky file head, as the web window paints
/// them (the head's `z-index: 2` against the bar's 1): woken, only the last eight points of their
/// frame show, past the column's edge, and a press on them lands on the head, which folds its
/// file. The window draws them under the page for that; Escape is the way back, as there.
fn focus_marks(s: &State) -> El {
    let (ww, _) = s.size;
    let up = s.changes.reaching || s.at - s.changes.chrome_at < 1.8;
    let room = crate::rules::pane::artifact_placement(s.page_size().0) == crate::rules::pane::Placement::Beside;
    let mark = |glyph: &'static rune::Icon, help: &'static str, key: &'static str| {
        row().key(key).size(28.0, 28.0).radius(3.0).center().hover_group().cursor(Cursor::Pointer).help(help).child(icon(glyph, 16.0).color(dim()).group_color(ink()))
    };
    let marks = row()
        .gap(6.0)
        .padding(2.0)
        .radius(6.0)
        .fill(rgba(40, 38, 37, 0.86))
        .border(1.0, edge())
        .border_layout()
        .box_shadow(shadow(0.0, 10.0, 32.0, 0.0, rgba(0, 0, 0, 0.38), false))
        .child(mark(&crate::icons::PANE_BESIDE, "Move it to the side · Esc closes it", "pane-full").label("Exit Focus").on_press(move |s: &mut State| {
            if room {
                s.pane.full = false;
            } else {
                s.close_beside();
            }
        }))
        .child(mark(&crate::icons::FILE_OUTSIDE, "Open this file in your editor", "pane-outside").on_press_tx(|s: &mut State, tx| open_outside(s, tx)))
        .child(mark(&crate::icons::FILE_CLOSE, "Exit Focus", "pane-close").on_press(|s: &mut State| s.close_or_put_back()))
        .on_hover_change(|s: &mut State, inside, _| s.changes.reaching = inside);
    let wide = 2.0 + 3.0 * 28.0 + 2.0 * 6.0 + 2.0 + 2.0;
    let marks = marks.absolute(ww - 26.0 - wide, 52.0).opacity(if up { 1.0 } else { 0.0 }).offset(0.0, if up { 0.0 } else { -4.0 }).opacity_motion(rune::Curve::tween(0.16, rune::Easing::EASE)).offset_motion(rune::Curve::tween(0.16, rune::Easing::EASE));
    // The corner that wakes them: 260 by 110, the pointer on it reaching for them.
    let reach = el().absolute(ww - 260.0, 0.0).size(260.0, 110.0).on_hover_change(|s: &mut State, inside, _| s.changes.reaching = inside);
    el().absolute(0.0, 0.0).size(ww, s.size.1).child(reach).child(if up { marks } else { marks.inert() })
}

/// The window with the marks over a change that takes the workspace, and the pointer moving
/// anywhere over it waking them; any other window as it is.
pub fn over_the_workspace(s: &State, root: El) -> El {
    let takes = crate::pane::takes_the_card(s) && s.beside().is_some_and(|d| d.kind() == crate::rules::pane::Kind::Change);
    if !takes {
        return root;
    }
    root.on_pointer_move_local(|s: &mut State, _, tx| pointer_moved(s, tx)).child(focus_marks(s))
}

/// A pointer moving anywhere over the window while the change takes the workspace wakes its marks.
fn pointer_moved(s: &mut State, tx: &mut Tx) {
    let was = s.at - s.changes.chrome_at < 1.8;
    s.changes.chrome_at = s.at;
    tx.wake_after(std::time::Duration::from_millis(1810));
    if was {
        tx.live_only();
    }
}

impl State {
    /// Close over a change given the whole workspace puts it back beside the conversation; on a
    /// card with no room beside, or beside, it puts it down (`closeArtifact`).
    pub fn close_or_put_back(&mut self) {
        if self.pane.full {
            self.pane.full = false;
        } else {
            self.close_beside();
        }
    }
}

/* ------------------------------ the command bar ------------------------------ */

/// Whether the open thread's page draws the code button, which the command bar's "View code
/// changes" follows (`onShown({ change: !!figures })`).
pub fn has_a_change(s: &State, item: &crate::data::Item) -> bool {
    summary(s, item).is_some_and(|c| rules::figures_from(&c).is_some())
}

/* ------------------------------ a picture ------------------------------ */

/// What a picture of the web window had done to the change open beside its conversation: the
/// file it jumped to (`changeAt`), the gaps it opened (`changeGaps`, `[path, hunk]`), the files
/// it folded (`changeFolded`), the folders it shut (`changeShut`), the lines typed
/// (`changeTyped`, `[path, hunk, row, words]`) and whether it took the whole workspace
/// (`docFull`).
pub fn pictured(s: &mut State) {
    // The figures the page was answered with, there from the picture's first frame.
    let mut tx = Tx::default();
    ask_for_the_summary(s, false, s.now, &mut tx);
    let Some(here) = s.changes.here.as_ref() else { return };
    let key = here.key.clone();
    if let Some(folds) = s.fixture.remove("changeFolded").and_then(|v| v.as_array().cloned()) {
        s.changes.folds.insert(key.clone(), folds.iter().filter_map(Value::as_str).map(str::to_string).collect());
    }
    if let Some(shut) = s.fixture.remove("changeShut").and_then(|v| v.as_array().cloned())
        && let Some(here) = s.changes.here.as_mut()
    {
        here.shut = shut.iter().filter_map(Value::as_str).map(str::to_string).collect();
    }
    if let Some(typed) = s.fixture.remove("changeTyped").and_then(|v| v.as_array().cloned()) {
        for t in typed {
            let (Some(path), Some(h), Some(r), Some(words)) = (t.get(0).and_then(Value::as_str), t.get(1).and_then(Value::as_u64), t.get(2).and_then(Value::as_u64), t.get(3).and_then(Value::as_str)) else { continue };
            s.changes.typed.entry(key.clone()).or_default().entry(path.to_string()).or_default().entry(h as usize).or_default().insert(r as usize, words.to_string());
        }
    }
    if s.fixture.remove("docFull").is_some() {
        s.pane.full = true;
    }
    let gaps = s.fixture.remove("changeGaps").and_then(|v| v.as_array().cloned()).unwrap_or_default();
    if !gaps.is_empty() {
        for g in gaps {
            let (Some(path), Some(h)) = (g.get(0).and_then(Value::as_str), g.get(1).and_then(Value::as_u64)) else { continue };
            if let Some(f) = s.shown().and_then(|sh| sh.files.iter().position(|f| f.path == path)) {
                open_gap(s, f, h as usize, &mut tx);
            }
        }
    }
    if let Some(at) = s.fixture.remove("changeAt").and_then(|v| v.as_str().map(str::to_string))
        && let Some(here) = s.changes.here.as_mut()
    {
        here.start_at = Some(at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feel::tests::{key, press, rect_of, run, window_dressed};
    use rune::{Key, Mods, Runtime};

    /// `src/a.ts` as it stands after the invented run: twelve lines, the second and the eleventh
    /// changed by it, so its two hunks have six lines between them.
    const A_TS: &str = "const a = 1;\nlet b = 3;\nexport { a, b };\n// four\n// five\n// six\n// seven\n// eight\n// nine\nconst c = 1;\nlet d = 5;\nexport { c, d };\n";

    /// The invented change, its files under `repo`: a deleted note, `src/a.ts` with two hunks,
    /// and a new `src/b.ts`. The tree lists them in that order.
    fn the_change(repo: &str) -> Value {
        json!({
            "files": [
                { "path": "src/a.ts", "abs": format!("{repo}/src/a.ts"), "plus": 2, "minus": 2, "hunks": [
                    { "rows": [["=", "const a = 1;"], ["-", "let b = 2;"], ["+", "let b = 3;"], ["=", "export { a, b };"]], "nums": [[1, 1], [2, null], [null, 2], [3, 3]] },
                    { "rows": [["=", "const c = 1;"], ["-", "let d = 4;"], ["+", "let d = 5;"], ["=", "export { c, d };"]], "nums": [[10, 10], [11, null], [null, 11], [12, 12]] },
                ] },
                { "path": "src/b.ts", "abs": format!("{repo}/src/b.ts"), "plus": 2, "minus": 0, "hunks": [
                    { "rows": [["+", "export const e = 1;"], ["+", "export const f = 2;"]], "nums": [[null, 1], [null, 2]] },
                ] },
                { "path": "docs/gone.md", "plus": 0, "minus": 1, "hunks": [
                    { "rows": [["-", "# Gone"]], "nums": [[1, null]] },
                ] },
            ],
            "roots": [repo], "plus": 4, "minus": 3, "startedAt": 1_000.0, "endedAt": 1_000.0 + 6.0 * 60_000.0, "committed": false, "uncommitted": true,
        })
    }

    /// A window on a conversation whose run changed code, `w` by `h`, the files planted. Every
    /// thread of the invented inbox carries the same change, so whichever opens has it.
    /// `summary` is what the engine answers for the figures.
    fn with_a_change(name: &str, size: (f32, f32), summary: Option<Value>) -> (Runtime<State>, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("mercury-change-{name}-{}", std::process::id()));
        let repo = dir.join("code");
        std::fs::create_dir_all(repo.join("src")).expect("an invented checkout");
        std::fs::write(repo.join("src/a.ts"), A_TS).expect("an invented file");
        let repo = repo.to_string_lossy().to_string();
        let change = the_change(&repo);
        let summary = summary.unwrap_or_else(|| {
            let mut s = change.clone();
            for f in s["files"].as_array_mut().expect("files") {
                f.as_object_mut().expect("a file").remove("hunks");
            }
            json!({ "ok": true, "change": s })
        });
        let mut rt = window_dressed(name, &["--thread"], &dir.to_string_lossy(), "Done, and tested.", |f| {
            f["snapshot"]["products"][0]["repoPath"] = json!(repo);
            let mut threads = serde_json::Map::new();
            let mut wholes = serde_json::Map::new();
            for i in 0..4 {
                threads.insert(format!("storefront/w-feel{i}"), json!({ "zero:code-change": summary }));
                wholes.insert(rules::change_path_for(&format!("w-feel{i}")), json!({ "ok": true, "change": change }));
            }
            f["threads"] = Value::Object(threads);
            f["codeChanges"] = Value::Object(wholes);
        });
        rt.set_viewport(size, 2.0);
        run(&mut rt, 0.1);
        (rt, dir)
    }

    fn middle(rt: &Runtime<State>, key: &str) -> (f32, f32) {
        let [x, y, w, h] = rect_of(rt, key).unwrap_or_else(|| panic!("{key} is drawn"));
        (x + w / 2.0, y + h / 2.0)
    }

    fn open_it(rt: &mut Runtime<State>) {
        let (x, y) = middle(rt, "thread-code");
        press(rt, x, y);
        run(rt, 0.1);
    }

    fn the_open_change(rt: &Runtime<State>) -> Option<String> {
        rt.state.beside().filter(|d| d.kind() == crate::rules::pane::Kind::Change).map(|d| d.src.clone())
    }

    fn typed(rt: &mut Runtime<State>, words: &str) {
        for c in words.chars() {
            rt.key(rune::KeyEvent { key: Key::Text(c.to_string()), mods: Mods::default() });
        }
        rt.frame(1.0 / 60.0);
    }

    #[test]
    fn the_button_says_the_figures_opens_the_change_and_puts_it_away() {
        let (mut rt, dir) = with_a_change("button", (1280.0, 760.0), None);
        let thread = match &rt.state.view {
            View::Thread { id, .. } => id.clone(),
            _ => panic!("a conversation is open"),
        };
        assert!(rect_of(&rt, "thread-code").is_some(), "a thread whose run changed code has the button");
        open_it(&mut rt);
        assert_eq!(the_open_change(&rt).as_deref(), Some(rules::change_path_for(&thread).as_str()));
        assert!(rt.state.shown().is_some(), "the whole change is read when it opens");
        // Opened from the button it stands at its top, on its first file.
        assert_eq!(rt.state.changes.here.as_ref().map(|h| h.at), Some(0));
        // Pressed while open, it puts the change away.
        open_it(&mut rt);
        assert!(the_open_change(&rt).is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_change_with_nothing_in_it_draws_no_button() {
        let (rt, dir) = with_a_change("no-button", (1280.0, 760.0), Some(json!({ "ok": true, "change": { "files": [], "plus": 0, "minus": 0 } })));
        assert!(rect_of(&rt, "thread-code").is_none());
        let _ = std::fs::remove_dir_all(dir);
        let (rt, dir) = with_a_change("no-change", (1280.0, 760.0), Some(json!({ "ok": false, "error": "no change was recorded for this thread" })));
        assert!(rect_of(&rt, "thread-code").is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_narrow_pane_has_the_picker_and_a_wide_one_the_tree() {
        let (mut rt, dir) = with_a_change("narrow", (1280.0, 760.0), None);
        open_it(&mut rt);
        assert!(rect_of(&rt, PICKER).is_some() && rect_of(&rt, TREE).is_none(), "a pane of 776 points has the picker");
        let _ = std::fs::remove_dir_all(dir);
        let (mut rt, dir) = with_a_change("wide", (1600.0, 1000.0), None);
        open_it(&mut rt);
        assert!(rect_of(&rt, TREE).is_some() && rect_of(&rt, PICKER).is_none(), "a pane of 1077 points has the tree");
        // Folders before files, a chain of single folders as one, sorted: docs, then src.
        let shown = rt.state.shown().expect("read");
        assert_eq!(shown.files.iter().map(|f| f.path.as_str()).collect::<Vec<_>>(), ["docs/gone.md", "src/a.ts", "src/b.ts"]);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_file_folds_from_its_head_and_stays_folded_while_the_window_runs() {
        let (mut rt, dir) = with_a_change("fold", (1600.0, 1000.0), None);
        open_it(&mut rt);
        assert!(rect_of(&rt, "chg-rows-1").is_some(), "src/a.ts shows its rows");
        let (x, y) = middle(&rt, "chg-head-1");
        press(&mut rt, x, y);
        assert!(rect_of(&rt, "chg-rows-1").is_none(), "folded, it draws no code");
        // Put away and opened again, the fold is kept.
        open_it(&mut rt);
        open_it(&mut rt);
        assert!(rect_of(&rt, "chg-head-1").is_some() && rect_of(&rt, "chg-rows-1").is_none());
        let (x, y) = middle(&rt, "chg-head-1");
        press(&mut rt, x, y);
        assert!(rect_of(&rt, "chg-rows-1").is_some());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_gap_opens_onto_the_files_own_lines() {
        let (mut rt, dir) = with_a_change("gap", (1600.0, 1000.0), None);
        open_it(&mut rt);
        let (x, y) = middle(&rt, "g1-1");
        press(&mut rt, x, y);
        run(&mut rt, 0.05);
        let here = rt.state.changes.here.as_ref().expect("open");
        assert_eq!(here.gaps.get(&("src/a.ts".to_string(), 1)), Some(&(4, ["// four", "// five", "// six", "// seven", "// eight", "// nine"].map(String::from).to_vec())));
        assert!(rect_of(&rt, "o1-1-0").is_some(), "the lines are drawn in place");
        // A file moved on since the change is refused, and says so.
        std::fs::write(dir.join("code/src/a.ts"), A_TS.replace("const c = 1;", "const c = 2;")).expect("rewritten");
        rt.update(|s, tx| {
            if let Some(here) = s.changes.here.as_mut() {
                here.gaps.clear();
            }
            s.lay_out_the_change();
            open_gap(s, 1, 1, tx);
        });
        rt.frame(1.0 / 60.0);
        assert!(rt.state.changes.here.as_ref().is_some_and(|h| h.gaps.is_empty()));
        assert!(rt.state.toast.as_ref().is_some_and(|t| t.line.contains("changed since")));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn its_keys_are_its_own_j_and_k_do_nothing_and_escape_puts_it_down() {
        // A short window, so the column has somewhere to go.
        let (mut rt, dir) = with_a_change("keys", (1280.0, 520.0), None);
        let thread = rt.state.view.clone();
        open_it(&mut rt);
        key(&mut rt, Key::Text("j".into()));
        key(&mut rt, Key::Text("k".into()));
        assert_eq!(rt.state.view, thread, "J and K do not go to another thread over a change");
        assert!(the_open_change(&rt).is_some());
        // ] lands on the next change below the top of the column.
        let shown = rt.state.shown().expect("read");
        let starts = hunk_starts(&rt.state, &shown, 0.0);
        key(&mut rt, Key::Text("]".into()));
        run(&mut rt, 0.1);
        let (top, _, _) = COLUMN.with(Cell::get);
        assert_eq!(top, starts[0].1, "the column moved to the next change, under its file's head");
        key(&mut rt, Key::Text("]".into()));
        run(&mut rt, 0.1);
        assert_eq!(COLUMN.with(Cell::get).0, starts[1].1, "and the one after");
        key(&mut rt, Key::Text("[".into()));
        run(&mut rt, 0.1);
        // The arrows scroll the column, a row at a time.
        let before = COLUMN.with(Cell::get).0;
        key(&mut rt, Key::Down);
        run(&mut rt, 0.05);
        assert!(COLUMN.with(Cell::get).0 >= before);
        key(&mut rt, Key::Escape);
        assert!(the_open_change(&rt).is_none(), "Escape puts the change down");
        assert_eq!(rt.state.view, thread, "and the next leaves the thread");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_press_on_a_tree_row_jumps_and_the_arrows_then_walk_the_files() {
        let (mut rt, dir) = with_a_change("tree", (1600.0, 1000.0), None);
        open_it(&mut rt);
        let (x, y) = middle(&rt, "t:src/b.ts");
        press(&mut rt, x, y);
        run(&mut rt, 0.1);
        let here = rt.state.changes.here.as_ref().expect("open");
        assert_eq!((here.at, here.tree_keys), (2, true));
        key(&mut rt, Key::Up);
        assert_eq!(rt.state.changes.here.as_ref().map(|h| h.at), Some(1));
        key(&mut rt, Key::Home);
        assert_eq!(rt.state.changes.here.as_ref().map(|h| h.at), Some(0));
        // A folder shut hides its files and moves nothing in the column.
        let (x, y) = middle(&rt, "t:src/a.ts");
        let at = rt.state.changes.here.as_ref().map(|h| h.at);
        let folder = rt.elements().into_iter().find(|e| e.rect[1] < y - 20.0 && e.rect[1] > y - 40.0 && e.rect[0] < x).map(|e| e.rect);
        if let Some([fx, fy, fw, fh]) = folder {
            press(&mut rt, fx + fw / 2.0, fy + fh / 2.0);
        }
        assert!(rt.state.changes.here.as_ref().is_some_and(|h| h.shut.contains("src/")));
        assert!(rect_of(&rt, "t:src/a.ts").is_none());
        assert_eq!(rt.state.changes.here.as_ref().map(|h| h.at), at);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_copy_across_lines_leaves_the_removed_line_out() {
        let (mut rt, dir) = with_a_change("copy", (1600.0, 1000.0), None);
        open_it(&mut rt);
        let [x0, y0, _, _] = rect_of(&rt, "l:1:0:0:=").expect("the first line");
        let [x1, y1, w1, _] = rect_of(&rt, "l:1:0:3:=").expect("the fourth line");
        rt.pointer_move(x0 + 1.0, y0 + 5.0);
        rt.pointer_down(x0 + 1.0, y0 + 5.0);
        for k in 1..=6 {
            let t = k as f32 / 6.0;
            rt.pointer_move(x0 + 1.0 + t * (x1 + w1 - x0 - 2.0), y0 + 5.0 + t * (y1 - y0));
            rt.frame(1.0 / 60.0);
        }
        rt.pointer_up(x1 + w1 - 1.0, y1 + 5.0);
        rt.frame(1.0 / 60.0);
        rt.copy_request = None;
        rt.key(rune::KeyEvent { key: Key::Command('c'), mods: Mods { cmd: true, ..Mods::default() } });
        rt.frame(1.0 / 60.0);
        let copied = rt.copy_request.as_ref().expect("⌘C copies").content().text().to_string();
        assert_eq!(copied, "const a = 1;\nlet b = 3;\nexport { a, b };", "no gutter, and the removed line left out");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_line_takes_typing_keeps_it_and_says_unsaved() {
        let (mut rt, dir) = with_a_change("typing", (1600.0, 1000.0), None);
        open_it(&mut rt);
        let [x, y, _, _] = rect_of(&rt, "l:1:0:2:+").expect("the added line");
        // A press after the third character puts the caret there.
        press(&mut rt, x + 3.0 * ADVANCE + 1.0, y + 5.0);
        run(&mut rt, 0.05);
        assert_eq!(rt.state.changes.here.as_ref().and_then(|h| h.editing), Some((1, 0, 2)));
        assert!(rt.has_focus());
        typed(&mut rt, "x");
        assert_eq!(rt.state.changes.here.as_ref().map(|h| h.edit.text.clone()).as_deref(), Some("letx b = 3;"));
        // Return is refused: a row is one line.
        key(&mut rt, Key::Enter);
        assert!(!rt.state.changes.here.as_ref().unwrap().edit.text.contains('\n'));
        // ↓ moves the caret to the next line in the file, the removed one skipped.
        key(&mut rt, Key::Down);
        assert_eq!(rt.state.changes.here.as_ref().and_then(|h| h.editing), Some((1, 0, 3)));
        // Escape lets go of the line, the words kept and the file marked unsaved.
        key(&mut rt, Key::Escape);
        run(&mut rt, 0.05);
        assert!(rt.state.changes.here.as_ref().is_some_and(|h| h.editing.is_none()));
        assert!(the_open_change(&rt).is_some(), "the first Escape is the line's");
        let here = rt.state.changes.here.as_ref().unwrap();
        assert!(unsaved(&rt.state, here, "src/a.ts"));
        assert!(rt.elements().iter().any(|e| e.text.as_deref() == Some("Unsaved")));
        // Put away and opened again, the typed words are still there, never on disk.
        open_it(&mut rt);
        open_it(&mut rt);
        assert!(rt.elements().iter().any(|e| e.key.as_deref() == Some("l:1:0:2:+") && e.text.as_deref() == Some("letx b = 3;")));
        assert_eq!(std::fs::read_to_string(dir.join("code/src/a.ts")).unwrap(), A_TS);
        // ⌘S with no engine behind the window writes nothing and says so.
        rt.key(rune::KeyEvent { key: Key::Command('s'), mods: Mods { cmd: true, ..Mods::default() } });
        rt.frame(1.0 / 60.0);
        assert!(rt.state.toast.as_ref().is_some_and(|t| t.line.contains("save this file")));
        assert_eq!(std::fs::read_to_string(dir.join("code/src/a.ts")).unwrap(), A_TS);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_steps_file_is_a_chip_only_when_the_change_holds_it() {
        let (rt, dir) = with_a_change("chip", (1280.0, 760.0), None);
        let item = rt.state.open_item().cloned().expect("open");
        let font = mono(12.0, 19.2);
        assert!(chip(&rt.state, &item, &format!("{}/code/src/a.ts", dir.to_string_lossy()), "…/src/a.ts", font).is_some());
        assert!(chip(&rt.state, &item, "/elsewhere/notes.ts", "notes.ts", font).is_none());
        assert!(chip(&rt.state, &item, "", "", font).is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_chip_opens_the_change_at_its_file_every_press() {
        let (mut rt, dir) = with_a_change("chip-opens", (1600.0, 1000.0), None);
        let View::Thread { id, .. } = rt.state.view.clone() else { panic!("a conversation") };
        let src = rules::change_path_for(&id);
        rt.update(|s, _| s.open_change(&src, Some("src/b.ts")));
        run(&mut rt, 0.2);
        assert_eq!(rt.state.changes.here.as_ref().map(|h| h.at), Some(2));
        // Walked away and the same chip pressed again, it lands again.
        rt.update(|s, tx| jump(s, 0, tx));
        run(&mut rt, 0.2);
        let src = rules::change_path_for(&id);
        rt.update(|s, _| s.open_change(&src, Some("src/b.ts")));
        run(&mut rt, 0.2);
        assert_eq!(rt.state.changes.here.as_ref().map(|h| h.at), Some(2));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_corners_give_it_the_whole_workspace_and_escape_puts_it_back_beside() {
        let (mut rt, dir) = with_a_change("full", (1280.0, 760.0), None);
        open_it(&mut rt);
        let (x, y) = middle(&rt, "pane-full");
        press(&mut rt, x, y);
        assert!(rt.state.pane.full);
        assert_eq!(rt.state.pane_placement(), Some(crate::rules::pane::Placement::Focus));
        assert!(rect_of(&rt, TREE).is_some(), "the whole card is wide enough for the tree");
        key(&mut rt, Key::Escape);
        assert!(!rt.state.pane.full && the_open_change(&rt).is_some(), "Escape puts it back beside");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_command_bar_offers_view_code_changes_on_a_thread_with_one() {
        let (mut rt, dir) = with_a_change("bar", (1280.0, 760.0), None);
        rt.key(rune::KeyEvent { key: Key::Command('k'), mods: Mods { cmd: true, ..Mods::default() } });
        rt.frame(1.0 / 60.0);
        typed(&mut rt, "diff");
        assert!(rt.elements().iter().any(|e| e.text.as_deref() == Some("View code changes")));
        key(&mut rt, Key::Enter);
        run(&mut rt, 0.1);
        assert!(the_open_change(&rt).is_some());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_change_that_cannot_be_read_says_why_and_one_on_its_way_holds_its_place() {
        let (mut rt, dir) = with_a_change("refused", (1280.0, 760.0), None);
        rt.update(|s, _| {
            for v in s.fixture.get_mut("codeChanges").and_then(Value::as_object_mut).expect("answers").values_mut() {
                *v = json!({ "ok": false, "error": "that change could not be read (an invented reason)" });
            }
        });
        open_it(&mut rt);
        assert!(rt.elements().iter().any(|e| e.text.as_deref() == Some("that change could not be read (an invented reason)")));
        open_it(&mut rt);
        rt.update(|s, _| {
            for v in s.fixture.get_mut("codeChanges").and_then(Value::as_object_mut).expect("answers").values_mut() {
                *v = Value::Null;
            }
        });
        open_it(&mut rt);
        assert!(rt.state.shown().is_none() && the_open_change(&rt).is_some(), "on its way, the skeleton holds its place");
        let _ = std::fs::remove_dir_all(dir);
    }

    /// How long a large change takes to open from the button and to scroll, on an invented one
    /// of `MERCURY_BIG_CHANGE` rows (twenty thousand, the most the engine keeps of a file times
    /// one, across eighty files, some lines two thousand characters long): a measurement, run with
    /// `MERCURY_BIG_CHANGE=20000 cargo test --release -- --ignored --nocapture a_big_change_opens`.
    #[test]
    #[ignore]
    fn a_big_change_opens_in() {
        let Ok(rows) = std::env::var("MERCURY_BIG_CHANGE").map(|n| n.parse::<usize>().unwrap_or(20_000)) else { return };
        let files = 80;
        let per = rows / files;
        let big: Vec<Value> = (0..files)
            .map(|f| {
                let hunks: Vec<Value> = (0..per / 25)
                    .map(|h| {
                        let start = 1 + h * 40;
                        let rows: Vec<Value> = (0..25).map(|r| {
                            let mark = match r % 5 { 1 => "-", 2 | 3 => "+", _ => "=" };
                            let line = if r == 12 && h % 4 == 0 { format!("  const long_{f}_{h} = '{}';", "x".repeat(2000)) } else { format!("  const value_{r} = compute(input_{r}, {h}); // invented line {r} of file {f}") };
                            json!([mark, line])
                        }).collect();
                        let nums: Vec<Value> = (0..25).map(|r| json!([start + r, start + r])).collect();
                        json!({ "rows": rows, "nums": nums })
                    })
                    .collect();
                json!({ "path": format!("pkg{}/src/file_{f}.ts", f % 7), "plus": per * 2 / 5, "minus": per / 5, "hunks": hunks })
            })
            .collect();
        let change = json!({ "files": big, "plus": rows * 2 / 5, "minus": rows / 5 });
        let (mut rt, dir) = with_a_change("big", (1280.0, 760.0), None);
        rt.update(move |s, _| {
            for v in s.fixture.get_mut("codeChanges").and_then(Value::as_object_mut).expect("answers").values_mut() {
                *v = json!({ "ok": true, "change": change });
            }
        });
        let (x, y) = middle(&rt, "thread-code");
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        let started = std::time::Instant::now();
        rt.frame(1.0 / 60.0);
        rt.frame(1.0 / 60.0);
        let opened = started.elapsed();
        assert!(rt.state.shown().is_some_and(|s| s.files.len() == files));
        let still = std::time::Instant::now();
        let views = VIEWS.with(Cell::get);
        for _ in 0..10 {
            rt.frame(1.0 / 60.0);
        }
        println!("a frame at rest: {:?}, the change drawn again {} times", still.elapsed() / 10, VIEWS.with(Cell::get) - views);
        let mut frames = Vec::new();
        for _ in 0..60 {
            let at = std::time::Instant::now();
            rt.wheel(900.0, 400.0, -120.0);
            rt.frame(1.0 / 60.0);
            frames.push(at.elapsed());
        }
        frames.sort();
        let jump = std::time::Instant::now();
        rt.update(|s, tx| jump_to_the_last(s, tx));
        rt.frame(1.0 / 60.0);
        rt.frame(1.0 / 60.0);
        println!("{rows} rows in {files} files: opened in {opened:?}; a frame scrolling: half under {:?}, the slowest {:?}; to the last file in {:?}", frames[30], frames[59], jump.elapsed());
        let _ = std::fs::remove_dir_all(dir);
    }

    fn jump_to_the_last(s: &mut State, tx: &mut Tx) {
        let last = s.shown().map_or(0, |sh| sh.files.len() - 1);
        jump(s, last, tx);
    }

    #[test]
    fn a_press_lands_where_the_line_wraps() {
        // 116 columns, as a line of the column holds with the change over the whole workspace at
        // 1280 by 760, where the web window breaks this line before "charged".
        let line = "  it('adds the lines after rounding each one, so the receipt a customer sees always matches what the card is charged', () => {";
        let cols = ((1176.0 - 24.0 - TREE_W - NUM_W - MARK_W - LINE_PAD) / ADVANCE).floor() as usize;
        assert_eq!(cols, 116);
        assert_eq!(caret_at(line, cols, 0.0, 5.0), 0);
        assert_eq!(caret_at(line, cols, ADVANCE * 4.0, 5.0), 4);
        // The second visual line starts after the space that ends the first.
        let second = caret_at(line, cols, 0.0, row_tall() + 2.0);
        assert_eq!(&line[second..second + 7], "charged");
        // Past the end of the last visual line is the line's end.
        assert_eq!(caret_at(line, cols, 900.0, row_tall() + 2.0), line.chars().count());
        // Past the end of an earlier one is its last character, not the next line.
        assert!(caret_at(line, cols, 900.0, 5.0) < second);
        // At 93 columns, beside the conversation at 1280 by 760, before "what".
        assert_eq!(&line[caret_at(line, 93, 0.0, row_tall() + 2.0)..][..4], "what");
        // A word longer than the line breaks anywhere.
        let long = "x".repeat(250);
        assert_eq!(caret_at(&long, 100, 0.0, row_tall() * 2.0 + 1.0), 200);
    }

    #[test]
    fn the_colours_and_the_changed_words_are_byte_ranges_of_the_line() {
        let spans = spans_of("const x = 600;", Some((10, 12)), Mark::Added);
        let bg: Vec<(u32, u32)> = spans.iter().filter(|s| s.bg.is_some()).map(|s| (s.start, s.end)).collect();
        assert_eq!(bg, vec![(10, 12)]);
        // The number keeps its colour on both sides of the cut.
        let coloured: Vec<(u32, u32)> = spans.iter().filter(|s| s.color.is_some()).map(|s| (s.start, s.end)).collect();
        assert!(coloured.contains(&(0, 5)) && coloured.contains(&(10, 12)) && coloured.contains(&(12, 13)));
        // A comment leans.
        assert!(spans_of("// why", None, Mark::Same).iter().all(|s| s.italic));
        // Words past the first byte of a wide character stay on character edges.
        let wide = spans_of("é = 'ü';", Some((5, 6)), Mark::Removed);
        assert!(wide.iter().any(|s| s.bg.is_some() && (s.start, s.end) == (6, 8)));
    }
}
