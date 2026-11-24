//! The code a thread's run changed, and every rule the web window's view of it goes by: what the
//! strip's button says (`renderer/src/change-figures.ts`), the tree of files, the words that
//! changed, the lines between two hunks (`renderer/src/code-artifact.ts`), what a key means
//! inside a change (`renderer/src/code-keys.ts`), what a copy takes (`renderer/src/code-copy.ts`),
//! how a typed line goes back into its file (`renderer/src/code-edit.ts`), which step's file a
//! change holds (`fileInChange` in `shared/work-lines.mts`), and where the engine finds one of its
//! files (`findFile` in `main/code-file.mts`). Each with the web's own test cases.
//!
//! Places inside a line are counted in characters, where the web counts UTF-16 units: they
//! differ only past the Basic Multilingual Plane, which source code rarely holds, and only cut a
//! line where both agree it changes.

use serde_json::Value;

/* ------------------------------ the shapes ------------------------------ */

/// What a row of a hunk is: unchanged (`=`), added (`+`) or removed (`-`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Same,
    Added,
    Removed,
}

/// One line of a change.
pub type Row = (Mark, String);

/// A run of lines the agent touched, with the number each row had in the old file and the new
/// one, when git gave them (`RowNums`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hunk {
    pub rows: Vec<Row>,
    pub nums: Option<Vec<(Option<u32>, Option<u32>)>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChangedFile {
    pub path: String,
    /// Where the run wrote it, when the change says.
    pub abs: Option<String>,
    pub hunks: Vec<Hunk>,
    pub plus: u64,
    pub minus: u64,
}

/// A run's change as the engine hands it over (`SavedChange`), or its summary, whose files have
/// no hunks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Change {
    pub files: Vec<ChangedFile>,
    pub plus: u64,
    pub minus: u64,
    pub started_at: Option<f64>,
    pub ended_at: Option<f64>,
    /// Where the folder stood when the run ended, or when the pane read it.
    pub committed: Option<bool>,
    pub uncommitted: Option<bool>,
    /// Whether Git there can name an author for a commit (`knowsYou`, the ship question's).
    pub knows_you: Option<bool>,
    /// The folders the run worked in, where a file of it is looked for.
    pub roots: Vec<String>,
}

fn count(v: Option<&Value>) -> u64 {
    v.and_then(Value::as_f64).filter(|n| n.is_finite() && *n > 0.0).map_or(0, |n| n as u64)
}

impl Change {
    /// The change out of the engine's answer (`change`), or nothing when it is not one.
    pub fn from_value(v: &Value) -> Option<Change> {
        let files = v.get("files")?.as_array()?;
        let mark = |m: &str| match m {
            "+" => Some(Mark::Added),
            "-" => Some(Mark::Removed),
            "=" => Some(Mark::Same),
            _ => None,
        };
        let files = files
            .iter()
            .filter_map(|f| {
                let path = f.get("path")?.as_str()?.to_string();
                let hunks = f.get("hunks").and_then(Value::as_array).map_or_else(Vec::new, |hs| {
                    hs.iter()
                        .map(|h| {
                            let rows: Vec<Row> = h.get("rows").and_then(Value::as_array).map_or_else(Vec::new, |rows| {
                                rows.iter().filter_map(|r| Some((mark(r.get(0)?.as_str()?)?, r.get(1).and_then(Value::as_str).unwrap_or("").to_string()))).collect()
                            });
                            let num = |n: Option<&Value>| n.and_then(Value::as_f64).map(|n| n as u32);
                            let nums = h.get("nums").and_then(Value::as_array).map(|ns| ns.iter().map(|p| (num(p.get(0)), num(p.get(1)))).collect::<Vec<_>>()).filter(|ns| ns.len() == rows.len());
                            Hunk { rows, nums }
                        })
                        .collect()
                });
                Some(ChangedFile { path, abs: f.get("abs").and_then(Value::as_str).filter(|a| !a.is_empty()).map(str::to_string), hunks, plus: count(f.get("plus")), minus: count(f.get("minus")) })
            })
            .collect();
        let at = |k: &str| v.get(k).and_then(Value::as_f64).filter(|t| *t > 0.0);
        Some(Change {
            files,
            plus: count(v.get("plus")),
            minus: count(v.get("minus")),
            started_at: at("startedAt"),
            ended_at: at("endedAt"),
            committed: v.get("committed").and_then(Value::as_bool),
            uncommitted: v.get("uncommitted").and_then(Value::as_bool),
            knows_you: v.get("knowsYou").and_then(Value::as_bool),
            roots: v.get("roots").and_then(Value::as_array).map(|r| r.iter().filter_map(Value::as_str).filter(|r| !r.is_empty()).map(str::to_string).collect()).unwrap_or_default(),
        })
    }
}

/// Where one thread's change is, inside its project (`changePathFor`).
pub fn change_path_for(item: &str) -> String {
    format!("runs/{item}/the-change-it-made.change")
}

/// Whether a path is a run's own change, which the pane names "The change" rather than by the
/// folder the app keeps it in (`DocPane.tsx`).
pub fn is_a_runs_change(path: &str) -> bool {
    let Some(rest) = path.strip_suffix("/the-change-it-made.change") else { return false };
    let mut parts = rest.rsplit('/');
    let (Some(id), Some(runs)) = (parts.next(), parts.next()) else { return false };
    runs == "runs" && id.strip_prefix("w-").is_some_and(|hex| !hex.is_empty() && hex.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()))
}

/* ------------------------------ the figures on the button ------------------------------ */

/// What one run did, as the strip's button says it (`ChangeFigures`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Figures {
    pub files: usize,
    pub plus: u64,
    pub minus: u64,
}

/// The figures out of a change, or nothing when there is nothing to say: no file, or no line
/// added or removed (`figuresFrom`).
pub fn figures_from(change: &Change) -> Option<Figures> {
    let files = change.files.len();
    if files == 0 || (change.plus == 0 && change.minus == 0) {
        return None;
    }
    Some(Figures { files, plus: change.plus, minus: change.minus })
}

/// A figure with thousands separated, because `+2,199` is read and `+2199` counted (`figure`).
pub fn figure(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// "+2,199" and "−48", with a side that is zero left out and the minus U+2212 (`signed`): true
/// for the added side.
pub fn signed(plus: u64, minus: u64) -> Vec<(bool, String)> {
    let mut out = Vec::new();
    if plus > 0 {
        out.push((true, format!("+{}", figure(plus))));
    }
    if minus > 0 {
        out.push((false, format!("−{}", figure(minus))));
    }
    out
}

/// "1 file", "18 files" (`fileCount`).
pub fn file_count(n: usize) -> String {
    if n == 1 { "1 file".to_string() } else { format!("{n} files") }
}

/// How long the figures of a finished run go on being read: its folder is read back after its
/// answer is written (`FOLDER_SETTLES_MS` in `renderer/src/ship-offer.ts`).
pub const FOLDER_SETTLES_MS: f64 = 120_000.0;
/// How often they are read while a run is up, or while its folder may still move.
pub const FIGURES_AGAIN_MS: f64 = 4_000.0;

/// Whether a finished run's folder may still move: its answer was written less than two minutes
/// ago (`folderMayStillMove`).
pub fn folder_may_still_move(result_at: f64, now: f64) -> bool {
    result_at > 0.0 && now - result_at < FOLDER_SETTLES_MS
}

/// Whether the button's figures are asked for now (`Focus.tsx`): on a new reading of the thread
/// (`fresh`: another thread, its row changed, or a run started on it); and while its folder may
/// still move (a run up, or its answer written under two minutes ago), every four seconds after
/// the last ask, and while a run is up on every change the engine pushes too, the beat the
/// chips on a run's steps are read on (`ItemThread.tsx`). A thread with neither is asked once.
pub fn asks_for_the_figures(fresh: bool, run_up: bool, result_at: f64, asked_at: Option<f64>, pushed: bool, now: f64) -> bool {
    let settling = run_up || folder_may_still_move(result_at, now);
    let due = asked_at.is_some_and(|at| now - at >= FIGURES_AGAIN_MS);
    fresh || (settling && (due || (pushed && run_up)))
}

/* ------------------------------ the tree's head ------------------------------ */

/// The whole change in one line: "11 files +287 −74", both sides always and no separators
/// (`changeSummary`).
pub fn change_summary(change: &Change) -> String {
    format!("{} +{} −{}", file_count(change.files.len()), change.plus, change.minus)
}

/// "not committed" when the run's work reached no commit and is still loose (`standingLabel`).
pub fn standing_label(change: &Change) -> &'static str {
    if change.committed == Some(false) && change.uncommitted == Some(true) { "not committed" } else { "" }
}

/// How long the run took, or nothing when it is not known (`tookLabel`).
pub fn took_label(change: &Change) -> String {
    let (Some(from), Some(to)) = (change.started_at, change.ended_at) else { return String::new() };
    if from <= 0.0 || to <= 0.0 || to <= from {
        return String::new();
    }
    let mins = ((to - from) / 60_000.0).round() as i64;
    match mins {
        m if m < 1 => "under a minute".to_string(),
        1 => "1 minute".to_string(),
        m if m < 60 => format!("{m} minutes"),
        m => {
            let hours = (m as f64 / 60.0).round() as i64;
            if hours == 1 { "1 hour".to_string() } else { format!("{hours} hours") }
        }
    }
}

/* ------------------------------ the tree ------------------------------ */

/// A row of the tree: a folder (a chain of single-child folders as one) or a file.
#[derive(Clone, Debug, PartialEq)]
pub enum TreeRow {
    Folder { label: String, depth: usize, key: String, files: usize, plus: u64, minus: u64 },
    File { label: String, depth: usize, path: String, plus: u64, minus: u64 },
}

impl TreeRow {
    pub fn depth(&self) -> usize {
        match self {
            TreeRow::Folder { depth, .. } | TreeRow::File { depth, .. } => *depth,
        }
    }
}

#[derive(Default)]
struct Node {
    name: String,
    children: Vec<Node>,
    /// The file this leaf is, by its place in the change.
    file: Option<usize>,
}

impl Node {
    fn child(&mut self, name: &str) -> &mut Node {
        // A name taken twice is the one node, as a Map keeps it.
        let at = match self.children.iter().position(|c| c.name == name && c.file.is_none()) {
            Some(at) => at,
            None => {
                self.children.push(Node { name: name.to_string(), ..Node::default() });
                self.children.len() - 1
            }
        };
        &mut self.children[at]
    }
    fn files_under(&self) -> Vec<usize> {
        match self.file {
            Some(f) => vec![f],
            None => self.children.iter().flat_map(Node::files_under).collect(),
        }
    }
}

/// `String.localeCompare` as a file browser sorts: letters by their lower case first, then the
/// case, then the characters themselves.
fn by_name(a: &str, b: &str) -> std::cmp::Ordering {
    a.to_lowercase().cmp(&b.to_lowercase()).then_with(|| b.cmp(a))
}

/// The changed files as a tree: folders before files at every level, each sorted by name, a
/// chain of folders with one child each one row (`treeRows`).
pub fn tree_rows(files: &[ChangedFile]) -> Vec<TreeRow> {
    let mut root = Node::default();
    for (i, file) in files.iter().enumerate() {
        let parts: Vec<&str> = file.path.split('/').filter(|p| !p.is_empty()).collect();
        let mut at = &mut root;
        for part in parts.iter().take(parts.len().saturating_sub(1)) {
            at = at.child(part);
        }
        let leaf = parts.last().copied().unwrap_or(file.path.as_str());
        // A leaf takes the place of one of its name, as `children.set` replaces it.
        at.children.retain(|c| !(c.name == leaf && c.file.is_some()));
        at.children.push(Node { name: leaf.to_string(), children: Vec::new(), file: Some(i) });
    }
    let mut out = Vec::new();
    fn walk(node: &Node, depth: usize, prefix: &str, files: &[ChangedFile], out: &mut Vec<TreeRow>) {
        let mut folders: Vec<&Node> = node.children.iter().filter(|k| k.file.is_none()).collect();
        let mut leaves: Vec<&Node> = node.children.iter().filter(|k| k.file.is_some()).collect();
        folders.sort_by(|a, b| by_name(&a.name, &b.name));
        leaves.sort_by(|a, b| by_name(&a.name, &b.name));
        for folder in folders {
            let mut label = folder.name.clone();
            let mut cur = folder;
            while cur.children.len() == 1 && cur.children[0].file.is_none() {
                cur = &cur.children[0];
                label = format!("{label}/{}", cur.name);
            }
            let under = cur.files_under();
            let key = format!("{prefix}{label}/");
            out.push(TreeRow::Folder { label, depth, key: key.clone(), files: under.len(), plus: under.iter().map(|f| files[*f].plus).sum(), minus: under.iter().map(|f| files[*f].minus).sum() });
            walk(cur, depth + 1, &key, files, out);
        }
        for leaf in leaves {
            let f = &files[leaf.file.unwrap_or(0)];
            out.push(TreeRow::File { label: leaf.name.clone(), depth, path: f.path.clone(), plus: f.plus, minus: f.minus });
        }
    }
    walk(&root, 0, "", files, &mut out);
    out
}

/// The rows left once everything under a shut folder is hidden (`visibleRows`).
pub fn visible_rows(rows: &[TreeRow], shut: &std::collections::HashSet<String>) -> Vec<TreeRow> {
    let mut out = Vec::new();
    let mut hiding: Option<usize> = None;
    for row in rows {
        if hiding.is_some_and(|d| row.depth() > d) {
            continue;
        }
        hiding = None;
        out.push(row.clone());
        if let TreeRow::Folder { key, depth, .. } = row
            && shut.contains(key)
        {
            hiding = Some(*depth);
        }
    }
    out
}

/// The files in the tree's order, so the column reads as the tree lists (`filesInTreeOrder`).
pub fn files_in_tree_order(files: &[ChangedFile]) -> Vec<ChangedFile> {
    let order: Vec<String> = tree_rows(files).into_iter().filter_map(|r| if let TreeRow::File { path, .. } = r { Some(path) } else { None }).collect();
    let rank = |p: &str| order.iter().position(|o| o == p).unwrap_or(0);
    let mut out = files.to_vec();
    out.sort_by_key(|f| rank(&f.path));
    out
}

/// Where an arrow, Home or End on the tree goes, as a path, or nowhere (`stepInTree`): `shown`
/// is the files the tree draws, `all` every file. A file hidden under a shut folder steps to
/// the nearest shown one.
pub fn step_in_tree(key: &str, path: Option<&str>, shown: &[String], all: &[String]) -> Option<String> {
    if shown.is_empty() || !matches!(key, "ArrowDown" | "ArrowUp" | "Home" | "End") {
        return None;
    }
    let here = path.and_then(|p| shown.iter().position(|s| s == p));
    if here.is_some() || key == "Home" || key == "End" {
        let at = here.unwrap_or(0);
        let to = walk_files(key, at, shown.len())?;
        return if here == Some(to) { None } else { Some(shown[to].clone()) };
    }
    let rank = |p: &str| all.iter().position(|a| a == p).map_or(-1, |i| i as i64);
    let mine = path.map_or(-1, rank);
    if key == "ArrowDown" { shown.iter().find(|p| rank(p) > mine).cloned() } else { shown.iter().rev().find(|p| rank(p) < mine).cloned() }
}

/// Which file an arrow, Home or End lands on, stopping at the ends (`walkFiles`).
pub fn walk_files(key: &str, at: usize, count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    let to = |n: i64| n.clamp(0, count as i64 - 1) as usize;
    match key {
        "ArrowDown" => Some(to(at as i64 + 1)),
        "ArrowUp" => Some(to(at as i64 - 1)),
        "Home" => Some(0),
        "End" => Some(count - 1),
        _ => None,
    }
}

/* ------------------------------ the words that changed ------------------------------ */

/// The part of each line the other does not share, between their common start and end, in
/// characters, or nothing when the lines are the same or more than six tenths differs
/// (`changedSpan`).
pub fn changed_span(a: &str, b: &str) -> Option<((usize, usize), (usize, usize))> {
    if a == b {
        return None;
    }
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let max = a.len().min(b.len());
    let mut p = 0;
    while p < max && a[p] == b[p] {
        p += 1;
    }
    let mut s = 0;
    while s < max - p && a[a.len() - 1 - s] == b[b.len() - 1 - s] {
        s += 1;
    }
    let minus = (p, a.len() - s);
    let plus = (p, b.len() - s);
    let share = ((minus.1 - minus.0) as f64 / a.len().max(1) as f64).max((plus.1 - plus.0) as f64 / b.len().max(1) as f64);
    if share > 0.6 { None } else { Some((minus, plus)) }
}

/// The changed span of each row of a hunk: a run of removed rows paired in order with the run of
/// added rows right after it, nothing across an unchanged row (`wordChanges`).
pub fn word_changes(rows: &[Row]) -> Vec<Option<(usize, usize)>> {
    let mut out = vec![None; rows.len()];
    let mut i = 0;
    while i < rows.len() {
        if rows[i].0 != Mark::Removed {
            i += 1;
            continue;
        }
        let minus_at = i;
        while i < rows.len() && rows[i].0 == Mark::Removed {
            i += 1;
        }
        let plus_at = i;
        while i < rows.len() && rows[i].0 == Mark::Added {
            i += 1;
        }
        for k in 0..(plus_at - minus_at).min(i - plus_at) {
            if let Some((minus, plus)) = changed_span(&rows[minus_at + k].1, &rows[plus_at + k].1) {
                out[minus_at + k] = Some(minus);
                out[plus_at + k] = Some(plus);
            }
        }
    }
    out
}

/* ------------------------------ the lines between two hunks ------------------------------ */

fn last_new(h: &Hunk) -> Option<(u32, &str)> {
    let nums = h.nums.as_ref()?;
    nums.iter().zip(&h.rows).rev().find_map(|((_, n), (_, t))| n.map(|n| (n, t.as_str())))
}
fn first_new(h: &Hunk) -> Option<(u32, &str)> {
    let nums = h.nums.as_ref()?;
    nums.iter().zip(&h.rows).find_map(|((_, n), (_, t))| n.map(|n| (n, t.as_str())))
}

/// How many lines of the new file lie between two hunks, or nothing when either has no numbers
/// or the two touch (`linesSkipped`).
pub fn lines_skipped(before: Option<&Hunk>, after: Option<&Hunk>) -> Option<u32> {
    let (end, start) = (last_new(before?)?.0, first_new(after?)?.0);
    let gap = start as i64 - end as i64 - 1;
    (gap > 0).then_some(gap as u32)
}

/// What a gap stood for, read from the file as it is now and numbered from `from`, or why it
/// cannot be placed (`hiddenLines`).
pub fn hidden_lines(file_text: &str, before: &Hunk, after: &Hunk) -> Result<(u32, Vec<String>), &'static str> {
    let (Some(end), Some(start)) = (last_new(before), first_new(after)) else {
        return Err("This change does not say which lines it skipped.");
    };
    let lines: Vec<&str> = file_text.split('\n').collect();
    let at = |n: u32| (n as usize).checked_sub(1).and_then(|i| lines.get(i)).copied();
    if at(end.0) != Some(end.1) || at(start.0) != Some(start.1) {
        return Err("This file has changed since, so the lines in between could not be placed. Open it in your editor to read them.");
    }
    let to = (end.0 as usize).max(start.0 as usize - 1);
    Ok((end.0 + 1, lines[end.0 as usize..to].iter().map(|l| l.to_string()).collect()))
}

/* ------------------------------ the keys ------------------------------ */

/// The keys a change open beside the conversation answers itself, kept from the app's own: the
/// arrows, the page keys, Home and End, J and K (which do nothing at all), ] and [
/// (`CHANGE_OWNS`, `changeOwnsKey`). Named as the web names them.
pub fn change_owns_key(key: &str) -> bool {
    let key = if key.chars().count() == 1 { key.to_lowercase() } else { key.to_string() };
    matches!(key.as_str(), "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | "PageUp" | "PageDown" | "Home" | "End" | "j" | "k" | "]" | "[")
}

/// Where ] and [ go: the start of the next change below the top of the column, or the one
/// before, or nowhere at either end, four points of slack keeping a change at the top from
/// being its own next (`nextChange`).
pub fn next_change(starts: &[f32], top: f32, down: bool) -> Option<f32> {
    const SLACK: f32 = 4.0;
    if down { starts.iter().copied().find(|s| *s > top + SLACK) } else { starts.iter().rev().copied().find(|s| *s < top - SLACK) }
}

/// Where the column scrolls after one key, or nothing for a key it does not answer: a row, a
/// screenful less two rows, the ends, never past them (`codeScroll`). `row` is a row's height,
/// `view` the column's, `top` where it is, `max` how far it goes.
pub fn code_scroll(key: &str, row: f32, view: f32, top: f32, max: f32, meta: bool) -> Option<f32> {
    let max = max.max(0.0);
    let clamp = |n: f32| n.round().clamp(0.0, max);
    let row = if row > 0.0 { row } else { 19.0 };
    let page = row.max(view - row * 2.0);
    match key {
        "ArrowDown" => Some(if meta { max } else { clamp(top + row) }),
        "ArrowUp" => Some(if meta { 0.0 } else { clamp(top - row) }),
        "PageDown" => Some(clamp(top + page)),
        "PageUp" => Some(clamp(top - page)),
        "Home" => Some(0.0),
        "End" => Some(max),
        _ => None,
    }
}

/// Which file is at the top of the column: the last whose block has come within twelve points
/// of the top, or the first (`fileOnScreen`). `offsets` are where each file's block starts.
pub fn file_on_screen(offsets: &[f32], top: f32) -> usize {
    let mut at = 0;
    for (i, o) in offsets.iter().enumerate() {
        if *o - top <= 12.0 {
            at = i;
        }
    }
    at
}

/// What an arrow does to a caret in a line, or nothing where the line itself moves it: off a
/// line's front onto the end of the one above, off its end onto the front of the one below, up
/// and down at the same column (`caretStep`). `-1` or `1` and the column, `usize::MAX` the end.
pub fn caret_step(key: &str, col: usize, len: usize, shift: bool, meta: bool) -> Option<(i32, usize)> {
    if shift || meta {
        return None;
    }
    match key {
        "ArrowUp" => Some((-1, col)),
        "ArrowDown" => Some((1, col)),
        "ArrowLeft" if col == 0 => Some((-1, usize::MAX)),
        "ArrowRight" if col >= len => Some((1, 0)),
        _ => None,
    }
}

/// Where the moving end of a selection goes after a shifted key, across lines and stopping at
/// both ends (`extendHead`). `lens` is every drawn line's length.
///
/// Ported and held to the web's cases, and not called: Rune, as pinned, has no way for a
/// window to set a selection that spans two texts (`Tx::select_text` takes one), so a shifted
/// arrow extends only inside the line being typed in. See `docs/parity-code-changes.md`, G4.
#[allow(dead_code)]
pub fn extend_head(key: &str, head: (usize, usize), lens: &[usize]) -> Option<(usize, usize)> {
    if lens.is_empty() {
        return None;
    }
    let last = lens.len() - 1;
    let line = head.0.min(last);
    let at = |n: usize, col: usize| (n, col.min(lens[n]));
    match key {
        "ArrowDown" => Some(if line >= last { at(line, lens[line]) } else { at(line + 1, head.1) }),
        "ArrowUp" => Some(if line == 0 { at(line, 0) } else { at(line - 1, head.1) }),
        "ArrowRight" if head.1 >= lens[line] => (line < last).then(|| at(line + 1, 0)),
        "ArrowRight" => Some(at(line, head.1 + 1)),
        "ArrowLeft" if head.1 == 0 => (line > 0).then(|| at(line - 1, lens[line - 1])),
        "ArrowLeft" => Some(at(line, head.1 - 1)),
        "Home" => Some(at(line, 0)),
        "End" => Some(at(line, lens[line])),
        _ => None,
    }
}

/// Whether a key changes the code, which is refused while a selection spans lines
/// (`editsAcrossLines`): Backspace, Delete, Return and any one character, never under a modifier.
pub fn edits_across_lines(key: &str, meta: bool, ctrl: bool, alt: bool) -> bool {
    if meta || ctrl || alt {
        return false;
    }
    matches!(key, "Backspace" | "Delete" | "Enter") || key.chars().count() == 1
}

/* ------------------------------ copying ------------------------------ */

/// Which of the rows a selection touches go on the clipboard, or nothing to leave a copy inside
/// one row as it is: the removed rows are left out unless they are all that was selected
/// (`linesToCopy`).
pub fn lines_to_copy(removed: &[bool]) -> Option<Vec<usize>> {
    if removed.len() < 2 {
        return None;
    }
    let live: Vec<usize> = removed.iter().enumerate().filter(|(_, r)| !**r).map(|(i, _)| i).collect();
    Some(if live.is_empty() { (0..removed.len()).collect() } else { live })
}

/// The text of a run of lines, the first clipped from `from` and the last to `to` characters
/// (`copiedText`).
pub fn copied_text(lines: &[String], from: i64, to: i64) -> String {
    let take = |s: &str, a: i64, b: i64| -> String {
        let n = s.chars().count() as i64;
        let a = a.clamp(0, n);
        let b = b.clamp(a, n);
        s.chars().skip(a as usize).take((b - a) as usize).collect()
    };
    match lines.len() {
        0 => String::new(),
        1 => take(&lines[0], from, to),
        n => {
            let mut out = vec![take(&lines[0], from, i64::MAX)];
            out.extend(lines[1..n - 1].iter().cloned());
            out.push(take(&lines[n - 1], 0, to));
            out.join("\n")
        }
    }
}

/* ------------------------------ typing into a change ------------------------------ */

/// The lines a hunk left in the file, what it added and kept (`postImage`).
pub fn post_image(rows: &[Row]) -> Vec<String> {
    rows.iter().filter(|(m, _)| *m != Mark::Removed).map(|(_, t)| t.clone()).collect()
}

/// Which rows of a hunk take typing: every one that is in the file now (`editableRows`).
pub fn editable_rows(rows: &[Row]) -> Vec<usize> {
    rows.iter().enumerate().filter(|(_, (m, _))| *m != Mark::Removed).map(|(i, _)| i).collect()
}

/// One hunk's typed lines: the rows it was drawn from and the typed text by row.
pub struct Splice<'a> {
    pub rows: &'a [Row],
    pub edits: &'a std::collections::BTreeMap<usize, String>,
}

/// The typed lines put back into the file's own bytes, or why not, with nothing written: each
/// hunk's lines found exactly as the run left them (the first place a repeated block sits),
/// applied from the foot of the file up, its last line break kept (`applySplices`). Gives the
/// new text and how many lines were typed.
pub fn apply_splices(file_text: &str, splices: &[Splice]) -> Result<(String, usize), &'static str> {
    let mut lines: Vec<String> = file_text.split('\n').map(str::to_string).collect();
    let trailing = lines.len() > 1 && lines.last().is_some_and(String::is_empty);
    if trailing {
        lines.pop();
    }
    let live: Vec<&Splice> = splices.iter().filter(|s| !s.edits.is_empty()).collect();
    if live.is_empty() {
        return Ok((file_text.to_string(), 0));
    }
    let mut placed: Vec<(usize, Vec<String>, Vec<String>)> = Vec::new();
    let mut changed = 0;
    for s in live {
        let before = post_image(s.rows);
        if before.is_empty() {
            return Err("Those lines were removed by the run, so there is nothing in the file to save them onto.");
        }
        let after: Vec<String> = s.rows.iter().enumerate().filter(|(_, (m, _))| *m != Mark::Removed).map(|(i, (_, t))| s.edits.get(&i).cloned().unwrap_or_else(|| t.clone())).collect();
        let at = (0..=lines.len().saturating_sub(before.len())).find(|i| before.len() <= lines.len() && lines[*i..*i + before.len()] == before[..]);
        let Some(at) = at else {
            return Err("The file has changed since this run, so those lines are not in it any more. Nothing was saved.");
        };
        placed.push((at, before, after));
        changed += s.edits.len();
    }
    placed.sort_by(|a, b| b.0.cmp(&a.0));
    for (at, before, after) in placed {
        if lines.get(at..at + before.len()) != Some(&before[..]) {
            return Err("Two of those edits overlap in the file, so nothing was saved.");
        }
        lines.splice(at..at + before.len(), after);
    }
    let mut text = lines.join("\n");
    if trailing {
        text.push('\n');
    }
    Ok((text, changed))
}

/// What a save says once it worked: the file's own name and how many lines (`savedLine`).
pub fn saved_line(path: &str, lines: usize) -> String {
    let name = path.rsplit('/').next().filter(|n| !n.is_empty()).unwrap_or(path);
    format!("Saved {name}, {lines} line{}.", if lines == 1 { "" } else { "s" })
}

/* ------------------------------ a step's file ------------------------------ */

/// The tools whose file a step shows as a chip into the change: those the engine reads a change
/// from (`CHANGES_A_FILE`).
pub fn changes_a_file(tool: &str) -> bool {
    matches!(tool, "Edit" | "Write" | "MultiEdit" | "NotebookEdit")
}

/// Which file of a change a step's path is: the whole path, or the change's key as its end at a
/// folder's edge, the longest match, `/private/tmp` read as `/tmp` (`fileInChange`).
pub fn file_in_change<'a>(paths: impl IntoIterator<Item = &'a str>, raw: &str) -> Option<String> {
    let tmp = |s: &str| match s.strip_prefix("/private/tmp/") {
        Some(rest) => format!("/tmp/{rest}"),
        None => s.to_string(),
    };
    let p = tmp(raw.trim());
    if p.is_empty() {
        return None;
    }
    let mut best: Option<&str> = None;
    for key in paths {
        let k = key.trim();
        if k.is_empty() {
            continue;
        }
        let hit = k == p || p.ends_with(&format!("/{k}")) || tmp(k) == p;
        if hit && best.is_none_or(|b| k.len() > b.len()) {
            best = Some(key);
        }
    }
    best.map(str::to_string)
}

/* ------------------------------ where a file of the change is ------------------------------ */

/// The most of one file the engine reads or writes for a change (`MAX_CODE_BYTES`).
pub const MAX_CODE_BYTES: u64 = 800_000;

/// Where one file of a change is on this Mac, as the engine finds it: where the run wrote it,
/// the path itself when it is whole, then under each of the change's roots and the project's,
/// the first that is a file (`findFile`). `is_file` asks the disk.
pub fn find_file(change: &Change, product_roots: &[String], file: &str, is_file: impl Fn(&str) -> bool) -> Result<String, &'static str> {
    if file.is_empty() {
        return Err("No file was named.");
    }
    let Some(entry) = change.files.iter().find(|f| f.path == file) else {
        return Err("That file is not part of this change.");
    };
    let mut tries: Vec<String> = Vec::new();
    tries.extend(entry.abs.clone());
    if file.starts_with('/') {
        tries.push(file.to_string());
    }
    for root in change.roots.iter().chain(product_roots) {
        tries.push(format!("{}/{}", root.trim_end_matches('/'), file));
    }
    tries.into_iter().find(|t| is_file(t)).ok_or("That file could not be found on this machine, so it cannot be edited here.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn file(path: &str, plus: u64, minus: u64) -> ChangedFile {
        ChangedFile { path: path.into(), plus, minus, ..ChangedFile::default() }
    }
    fn rows(list: &[(&str, &str)]) -> Vec<Row> {
        list.iter().map(|(m, t)| (match *m { "+" => Mark::Added, "-" => Mark::Removed, _ => Mark::Same }, t.to_string())).collect()
    }
    const ELEVEN: [&str; 11] = [
        "renderer/src/App.tsx", "renderer/src/components/Rail.tsx", "main/main.mts", "renderer/src/components/Browser.tsx", "renderer/src/main.tsx", "renderer/src/components/Focus.tsx",
        "renderer/src/components/Palette.tsx", "main/ipc.mts", "main/supervisor.mts", "renderer/src/format.ts", "renderer/src/types.ts",
    ];
    fn eleven() -> Vec<ChangedFile> {
        ELEVEN.iter().map(|p| file(p, 3, 1)).collect()
    }

    #[test]
    fn a_change_is_read_off_the_engines_answer() {
        let v = json!({ "files": [{ "path": "a/b.ts", "abs": "/w/a/b.ts", "plus": 2, "minus": 1, "hunks": [{ "rows": [["=", "x"], ["-", "y"], ["+", "z"], ["+", "w"]], "nums": [[1, 1], [2, null], [null, 2], [null, 3]] }] }], "plus": 2, "minus": 1, "startedAt": 5.0, "endedAt": 9.0, "committed": false, "uncommitted": true, "roots": ["/w"] });
        let c = Change::from_value(&v).expect("a change");
        assert_eq!(c.files[0].hunks[0].rows[1], (Mark::Removed, "y".to_string()));
        assert_eq!(c.files[0].hunks[0].nums.as_ref().unwrap()[1], (Some(2), None));
        assert_eq!((c.plus, c.minus, c.committed, c.uncommitted), (2, 1, Some(false), Some(true)));
        assert_eq!(c.roots, vec!["/w".to_string()]);
        // A summary has no rows; a file is still a file.
        let s = Change::from_value(&json!({ "files": [{ "path": "a.ts", "plus": 1, "minus": 0 }], "plus": 1, "minus": 0 })).unwrap();
        assert!(s.files[0].hunks.is_empty());
        assert!(Change::from_value(&json!("runs/w-1/the-change-it-made.change")).is_none());
        assert!(Change::from_value(&json!({})).is_none());
    }

    #[test]
    fn opens_a_change_and_nothing_else() {
        assert!(is_a_runs_change("runs/w-c1d09f0638/the-change-it-made.change"));
        assert!(is_a_runs_change("/Users/you/store/p/runs/w-c1d09f0638/the-change-it-made.change"));
        assert!(!is_a_runs_change("runs/w-readme0/the-change-it-made.change"));
        assert!(!is_a_runs_change("work/w-c1d0/the-change-it-made.change"));
        assert_eq!(change_path_for("w-c1d09f0638"), "runs/w-c1d09f0638/the-change-it-made.change");
    }

    #[test]
    fn the_figures_read_off_a_saved_change_or_say_nothing() {
        let c = Change { files: vec![file("a.ts", 2199, 48)], plus: 2199, minus: 48, ..Change::default() };
        assert_eq!(figures_from(&c), Some(Figures { files: 1, plus: 2199, minus: 48 }));
        assert_eq!(figures_from(&Change::default()), None);
        assert_eq!(figures_from(&Change { files: vec![file("a.ts", 0, 0)], ..Change::default() }), None);
        assert_eq!(figure(2199), "2,199");
        assert_eq!(figure(48), "48");
        assert_eq!(figure(1_234_567), "1,234,567");
        assert_eq!(signed(276, 0), vec![(true, "+276".to_string())]);
        assert_eq!(signed(0, 15), vec![(false, "−15".to_string())]);
        assert_eq!(signed(2199, 48), vec![(true, "+2,199".to_string()), (false, "−48".to_string())]);
        assert_eq!(signed(0, 48)[0].1.chars().next(), Some('\u{2212}'));
        assert_eq!(file_count(1), "1 file");
        assert_eq!(file_count(18), "18 files");
    }

    #[test]
    fn a_finished_runs_figures_are_read_for_two_minutes() {
        assert!(folder_may_still_move(1_000.0, 1_000.0 + 119_000.0));
        assert!(!folder_may_still_move(1_000.0, 1_000.0 + 120_000.0));
        assert!(!folder_may_still_move(0.0, 5.0));
    }

    #[test]
    fn the_figures_are_asked_for_on_a_new_reading_and_every_four_seconds_while_the_folder_moves() {
        let now = 1_000_000.0;
        // A new reading always asks; a thread with no run and an old answer is asked once.
        assert!(asks_for_the_figures(true, false, 0.0, Some(now - 60_000.0), false, now));
        assert!(!asks_for_the_figures(false, false, 0.0, Some(now - 60_000.0), false, now));
        assert!(!asks_for_the_figures(false, false, now - 300_000.0, Some(now - 60_000.0), true, now));
        // While a run is up: four seconds after the last ask, and not before, unless pushed.
        assert!(asks_for_the_figures(false, true, 0.0, Some(now - 4_000.0), false, now));
        assert!(!asks_for_the_figures(false, true, 0.0, Some(now - 3_999.0), false, now));
        assert!(asks_for_the_figures(false, true, 0.0, Some(now - 1_000.0), true, now));
        // For two minutes after the answer was written: every four seconds, a push not hurrying it.
        assert!(asks_for_the_figures(false, false, now - 60_000.0, Some(now - 4_000.0), false, now));
        assert!(!asks_for_the_figures(false, false, now - 60_000.0, Some(now - 1_000.0), true, now));
        assert!(!asks_for_the_figures(false, false, now - 120_000.0, Some(now - 4_000.0), false, now));
    }

    #[test]
    fn the_changed_files_are_a_tree() {
        let rows = tree_rows(&eleven());
        let folders: Vec<&str> = rows.iter().filter_map(|r| if let TreeRow::Folder { label, .. } = r { Some(label.as_str()) } else { None }).collect();
        assert!(folders.contains(&"renderer/src") && !folders.contains(&"renderer"));
        assert_eq!(rows.len(), 14);
        assert_eq!(folders.len(), 3);
        assert_eq!(rows.iter().map(TreeRow::depth).max(), Some(2));
        let top: Vec<String> = rows.iter().filter(|r| r.depth() == 0).map(|r| match r { TreeRow::Folder { label, .. } | TreeRow::File { label, .. } => label.clone() }).collect();
        assert_eq!(top, vec!["main", "renderer/src"]);
        let main: Vec<String> = rows.iter().filter_map(|r| match r { TreeRow::File { label, path, .. } if path.starts_with("main/") => Some(label.clone()), _ => None }).collect();
        assert_eq!(main, vec!["ipc.mts", "main.mts", "supervisor.mts"]);
        let src = rows.iter().find_map(|r| match r { TreeRow::Folder { label, files, plus, minus, .. } if label == "renderer/src" => Some((*files, *plus, *minus)), _ => None });
        assert_eq!(src, Some((8, 24, 8)));
        let shut = ["renderer/src/".to_string()].into_iter().collect();
        let shown = visible_rows(&rows, &shut);
        let paths: Vec<&str> = shown.iter().filter_map(|r| if let TreeRow::File { path, .. } = r { Some(path.as_str()) } else { None }).collect();
        assert!(shown.iter().any(|r| matches!(r, TreeRow::Folder { label, .. } if label == "renderer/src")));
        assert!(!paths.contains(&"renderer/src/App.tsx") && paths.contains(&"main/ipc.mts"));
        // The column reads in the tree's order.
        let in_column: Vec<String> = files_in_tree_order(&eleven()).into_iter().map(|f| f.path).collect();
        let in_tree: Vec<String> = rows.iter().filter_map(|r| if let TreeRow::File { path, .. } = r { Some(path.clone()) } else { None }).collect();
        assert_eq!(in_column, in_tree);
    }

    #[test]
    fn the_words_on_the_change() {
        let c = Change { files: eleven(), plus: 287, minus: 74, ..Change::default() };
        assert_eq!(change_summary(&c), "11 files +287 −74");
        let run = |a: f64, b: f64| took_label(&Change { started_at: Some(a), ended_at: Some(b), ..Change::default() });
        assert_eq!(took_label(&Change::default()), "");
        assert_eq!(run(0.0, 60_000.0), "");
        assert_eq!(run(1_785_706_116_205.0, 1_785_706_116_205.0 + 2_227_279.0), "37 minutes");
        assert_eq!(run(1.0, 1.0 + 20_000.0), "under a minute");
        assert_eq!(run(1.0, 1.0 + 60_000.0), "1 minute");
        assert_eq!(run(1.0, 1.0 + 70.0 * 60_000.0), "1 hour");
        assert_eq!(run(1.0, 1.0 + 150.0 * 60_000.0), "3 hours");
        assert_eq!(standing_label(&Change { committed: Some(false), uncommitted: Some(true), ..Change::default() }), "not committed");
        assert_eq!(standing_label(&Change { committed: Some(true), uncommitted: Some(true), ..Change::default() }), "");
        assert_eq!(standing_label(&Change::default()), "");
    }

    #[test]
    fn the_words_that_changed_are_marked() {
        let old = "import { listSharedProjects, joinSharedProject, markShared, makeDirect } from './projects.mjs';";
        let new = "import { listSharedProjects, joinSharedProject, markShared, makeDirect, ensurePersonalProject } from './projects.mjs';";
        let slice = |s: &str, (a, b): (usize, usize)| s.chars().skip(a).take(b - a).collect::<String>();
        let (minus, plus) = changed_span(old, new).unwrap();
        assert_eq!(slice(new, plus), ", ensurePersonalProject");
        assert_eq!(minus.0, minus.1);
        let (minus, plus) = changed_span("const MAX_ROWS = 600;", "const MAX_ROWS = 1200;").unwrap();
        assert_eq!(slice("const MAX_ROWS = 600;", minus), "6");
        assert_eq!(slice("const MAX_ROWS = 1200;", plus), "12");
        assert!(changed_span("a = 1;", "a = 1;").is_none());
        assert!(changed_span("return foo(bar);", "const x = await y.z();").is_none());
        let w = word_changes(&rows(&[("=", "a"), ("-", old), ("+", new), ("=", "b")]));
        assert!(w[0].is_none() && w[3].is_none());
        assert_eq!(slice(new, w[2].unwrap()), ", ensurePersonalProject");
        assert_eq!(w[1].unwrap().0, w[1].unwrap().1);
        let w = word_changes(&rows(&[("-", "let a = 1;"), ("+", "let a = 2;"), ("+", "let b = 3;")]));
        assert!(w[1].is_some() && w[2].is_none());
        assert_eq!(word_changes(&rows(&[("-", "let a = 1;"), ("=", "x"), ("+", "let a = 2;")])), vec![None, None, None]);
    }

    fn numbered(list: &[(&str, &str)], nums: &[(Option<u32>, Option<u32>)]) -> Hunk {
        Hunk { rows: rows(list), nums: Some(nums.to_vec()) }
    }

    #[test]
    fn the_hidden_lines_open_in_place() {
        let file: String = (1..=12).map(|i| format!("line {i}\n")).collect();
        let before = numbered(&[("=", "line 2"), ("+", "line 3"), ("=", "line 4")], &[(Some(2), Some(2)), (None, Some(3)), (Some(3), Some(4))]);
        let after = numbered(&[("=", "line 9"), ("-", "old 10"), ("+", "line 10")], &[(Some(8), Some(9)), (Some(9), None), (None, Some(10))]);
        assert_eq!(hidden_lines(&file, &before, &after), Ok((5, vec!["line 5".into(), "line 6".into(), "line 7".into(), "line 8".into()])));
        let moved = file.replace("line 9\n", "line 9 edited later\n");
        assert!(hidden_lines(&moved, &before, &after).unwrap_err().contains("changed since"));
        let bare = |h: &Hunk| Hunk { rows: h.rows.clone(), nums: None };
        assert!(hidden_lines(&file, &bare(&before), &bare(&after)).is_err());
        let next = numbered(&[("=", "line 5")], &[(Some(4), Some(5))]);
        assert_eq!(hidden_lines(&file, &before, &next), Ok((5, vec![])));
    }

    #[test]
    fn the_gap_counts_the_new_files_lines_not_drawn() {
        // `@@ -12,4 +14,5 @@` then `@@ -40,2 +43,2 @@`, as git-change.mts numbers them.
        let a = numbered(&[("=", "keep"), ("-", "gone"), ("+", "added"), ("+", "more"), ("=", "tail")], &[(Some(12), Some(14)), (Some(13), None), (None, Some(15)), (None, Some(16)), (Some(14), Some(17))]);
        let b = numbered(&[("=", "far"), ("-", "out"), ("+", "away")], &[(Some(40), Some(43)), (Some(41), None), (None, Some(44))]);
        assert_eq!(lines_skipped(Some(&a), Some(&b)), Some(25));
        let y = numbered(&[("-", "a"), ("+", "b")], &[(Some(1), None), (None, Some(1))]);
        let z = numbered(&[("-", "c"), ("+", "d")], &[(Some(2), None), (None, Some(2))]);
        assert_eq!(lines_skipped(Some(&y), Some(&z)), None);
        let loose = Hunk { rows: rows(&[("+", "a")]), nums: None };
        assert_eq!(lines_skipped(Some(&loose), Some(&loose)), None);
        assert_eq!(lines_skipped(None, None), None);
    }

    #[test]
    fn the_keys_a_change_answers() {
        for k in ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "PageUp", "PageDown", "Home", "End", "j", "k", "J", "K", "]", "["] {
            assert!(change_owns_key(k), "{k}");
        }
        for k in ["Escape", "e", "r", "c", "z", "Enter", "1", "9"] {
            assert!(!change_owns_key(k), "{k}");
        }
        let starts = [0.0, 400.0, 900.0, 2000.0];
        assert_eq!(next_change(&starts, 0.0, true), Some(400.0));
        assert_eq!(next_change(&starts, 500.0, true), Some(900.0));
        assert_eq!(next_change(&starts, 400.0, true), Some(900.0));
        assert_eq!(next_change(&starts, 402.0, true), Some(900.0));
        assert_eq!(next_change(&starts, 900.0, false), Some(400.0));
        assert_eq!(next_change(&starts, 950.0, false), Some(900.0));
        assert_eq!(next_change(&starts, 2000.0, true), None);
        assert_eq!(next_change(&starts, 0.0, false), None);
        assert_eq!(next_change(&[], 0.0, true), None);
    }

    #[test]
    fn the_arrows_scroll_the_code() {
        let s = |k: &str, top: f32| code_scroll(k, 20.0, 800.0, top, 5000.0, false);
        assert_eq!(s("ArrowDown", 0.0), Some(20.0));
        assert_eq!(s("ArrowUp", 100.0), Some(80.0));
        assert_eq!(s("PageDown", 0.0), Some(760.0));
        assert_eq!(s("PageUp", 1000.0), Some(240.0));
        assert_eq!(s("Home", 900.0), Some(0.0));
        assert_eq!(s("End", 0.0), Some(5000.0));
        assert_eq!(code_scroll("ArrowDown", 20.0, 800.0, 0.0, 5000.0, true), Some(5000.0));
        assert_eq!(code_scroll("ArrowUp", 20.0, 800.0, 900.0, 5000.0, true), Some(0.0));
        assert_eq!(s("ArrowUp", 0.0), Some(0.0));
        assert_eq!(s("ArrowDown", 5000.0), Some(5000.0));
        assert_eq!(s("PageDown", 4990.0), Some(5000.0));
        assert_eq!(code_scroll("PageDown", 20.0, 800.0, 0.0, 0.0, false), Some(0.0));
        assert_eq!(code_scroll("End", 20.0, 800.0, 0.0, -4.0, false), Some(0.0));
        assert_eq!(code_scroll("ArrowDown", 0.0, 800.0, 0.0, 5000.0, false), Some(19.0));
        for k in ["j", "ArrowLeft", "Escape"] {
            assert_eq!(s(k, 0.0), None);
        }
    }

    #[test]
    fn the_arrows_walk_the_files_and_stop_at_the_ends() {
        assert_eq!(walk_files("ArrowDown", 3, 18), Some(4));
        assert_eq!(walk_files("ArrowUp", 3, 18), Some(2));
        for k in ["j", "k", "J", "K"] {
            assert_eq!(walk_files(k, 3, 18), None);
        }
        assert_eq!(walk_files("ArrowDown", 17, 18), Some(17));
        assert_eq!(walk_files("ArrowUp", 0, 18), Some(0));
        assert_eq!(walk_files("Home", 9, 18), Some(0));
        assert_eq!(walk_files("End", 9, 18), Some(17));
        assert_eq!(walk_files("ArrowDown", 0, 0), None);
        let all: Vec<String> = ["a/one.ts", "a/two.ts", "b/three.ts", "b/four.ts", "c/five.ts"].map(String::from).to_vec();
        assert_eq!(step_in_tree("ArrowDown", Some("a/one.ts"), &all, &all).as_deref(), Some("a/two.ts"));
        assert_eq!(step_in_tree("ArrowUp", Some("a/two.ts"), &all, &all).as_deref(), Some("a/one.ts"));
        assert_eq!(step_in_tree("Home", Some("b/four.ts"), &all, &all).as_deref(), Some("a/one.ts"));
        assert_eq!(step_in_tree("End", Some("a/one.ts"), &all, &all).as_deref(), Some("c/five.ts"));
        let shown: Vec<String> = ["a/one.ts", "a/two.ts", "c/five.ts"].map(String::from).to_vec();
        assert_eq!(step_in_tree("ArrowDown", Some("a/two.ts"), &shown, &all).as_deref(), Some("c/five.ts"));
        assert_eq!(step_in_tree("ArrowDown", Some("b/three.ts"), &shown, &all).as_deref(), Some("c/five.ts"));
        assert_eq!(step_in_tree("ArrowUp", Some("b/three.ts"), &shown, &all).as_deref(), Some("a/two.ts"));
        assert_eq!(step_in_tree("ArrowUp", Some("a/one.ts"), &all, &all), None);
        assert_eq!(step_in_tree("ArrowDown", Some("c/five.ts"), &all, &all), None);
        assert_eq!(step_in_tree("j", Some("a/one.ts"), &all, &all), None);
        assert_eq!(step_in_tree("ArrowDown", Some("a/one.ts"), &[], &all), None);
    }

    #[test]
    fn the_file_at_the_top_of_the_column() {
        let offsets = [0.0, 400.0, 1200.0, 3000.0];
        assert_eq!(file_on_screen(&offsets, 0.0), 0);
        assert_eq!(file_on_screen(&offsets, 500.0), 1);
        assert_eq!(file_on_screen(&offsets, 1300.0), 2);
        assert_eq!(file_on_screen(&offsets, 9000.0), 3);
        assert_eq!(file_on_screen(&offsets, 380.0), 0);
        assert_eq!(file_on_screen(&offsets, 389.0), 1);
        assert_eq!(file_on_screen(&offsets, -50.0), 0);
        assert_eq!(file_on_screen(&[], 100.0), 0);
        assert_eq!(file_on_screen(&[0.0, f32::INFINITY, 1200.0], 1300.0), 2);
    }

    #[test]
    fn the_caret_crosses_lines() {
        assert_eq!(caret_step("ArrowDown", 4, 20, false, false), Some((1, 4)));
        assert_eq!(caret_step("ArrowUp", 4, 20, false, false), Some((-1, 4)));
        assert_eq!(caret_step("ArrowLeft", 0, 20, false, false), Some((-1, usize::MAX)));
        assert_eq!(caret_step("ArrowRight", 20, 20, false, false), Some((1, 0)));
        assert_eq!(caret_step("ArrowLeft", 4, 20, false, false), None);
        assert_eq!(caret_step("ArrowRight", 4, 20, false, false), None);
        assert_eq!(caret_step("ArrowDown", 4, 20, true, false), None);
        assert_eq!(caret_step("ArrowLeft", 0, 20, true, false), None);
        assert_eq!(caret_step("ArrowUp", 4, 20, false, true), None);
        assert_eq!(caret_step("ArrowRight", 0, 0, false, false), Some((1, 0)));
        assert_eq!(caret_step("ArrowLeft", 0, 0, false, false), Some((-1, usize::MAX)));
    }

    #[test]
    fn a_selection_is_extended_across_lines() {
        let lens = [10, 4, 20, 7];
        assert_eq!(extend_head("ArrowDown", (0, 3), &lens), Some((1, 3)));
        assert_eq!(extend_head("ArrowUp", (2, 5), &lens), Some((1, 4)));
        assert_eq!(extend_head("ArrowDown", (0, 9), &lens), Some((1, 4)));
        assert_eq!(extend_head("ArrowDown", (3, 2), &lens), Some((3, 7)));
        assert_eq!(extend_head("ArrowUp", (0, 5), &lens), Some((0, 0)));
        assert_eq!(extend_head("ArrowRight", (0, 10), &lens), Some((1, 0)));
        assert_eq!(extend_head("ArrowLeft", (1, 0), &lens), Some((0, 10)));
        assert_eq!(extend_head("ArrowRight", (2, 4), &lens), Some((2, 5)));
        assert_eq!(extend_head("ArrowLeft", (2, 4), &lens), Some((2, 3)));
        assert_eq!(extend_head("Home", (2, 11), &lens), Some((2, 0)));
        assert_eq!(extend_head("End", (2, 3), &lens), Some((2, 20)));
        assert_eq!(extend_head("PageDown", (1, 0), &lens), None);
        assert_eq!(extend_head("a", (1, 0), &lens), None);
        assert_eq!(extend_head("ArrowDown", (0, 0), &[]), None);
    }

    #[test]
    fn an_edit_across_lines_is_refused() {
        assert!(edits_across_lines("Backspace", false, false, false) && edits_across_lines("a", false, false, false));
        assert!(edits_across_lines("Delete", false, false, false) && edits_across_lines("Enter", false, false, false));
        for k in ["ArrowDown", "ArrowLeft", "Shift", "Tab", "Escape", "PageDown", "Home"] {
            assert!(!edits_across_lines(k, false, false, false), "{k}");
        }
        assert!(!edits_across_lines("c", true, false, false) && !edits_across_lines("s", true, false, false));
        assert!(edits_across_lines("é", false, false, false));
        assert!(!edits_across_lines("MediaPlayPause", false, false, false));
    }

    #[test]
    fn copying_code_out_of_a_change() {
        let l = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(copied_text(&l(&["const a = 1;", "const b = 2;", "const c = 3;"]), 0, 12), "const a = 1;\nconst b = 2;\nconst c = 3;");
        assert_eq!(copied_text(&l(&["const a = 1;", "const b = 2;"]), 6, 12), "a = 1;\nconst b = 2;");
        assert_eq!(copied_text(&l(&["const a = 1;", "const b = 2;"]), 0, 5), "const a = 1;\nconst");
        assert_eq!(copied_text(&l(&["const alpha = 1;"]), 6, 11), "alpha");
        assert_eq!(copied_text(&l(&["abc", "de"]), 99, 99), "\nde");
        assert_eq!(copied_text(&l(&["abc", "de"]), -5, -5), "abc\n");
        assert_eq!(copied_text(&[], 0, 0), "");
        assert_eq!(copied_text(&l(&["a", "", "b"]), 0, 1), "a\n\nb");
        assert_eq!(lines_to_copy(&[true, false]), Some(vec![1]));
        assert_eq!(lines_to_copy(&[false, true]), Some(vec![0]));
        assert_eq!(lines_to_copy(&[false]), None);
        assert_eq!(lines_to_copy(&[true]), None);
        assert_eq!(lines_to_copy(&[]), None);
        assert_eq!(lines_to_copy(&[false, true, true, false, false]), Some(vec![0, 3, 4]));
        assert_eq!(lines_to_copy(&[true, true]), Some(vec![0, 1]));
    }

    const FILE: &str = "// a greeting\nexport function greet(name) {\n  const greeting = \"hello\";\n  return `${greeting}, ${name}`;\n}\n\nexport const NAME = \"world\";\n";

    fn greet() -> Vec<Row> {
        rows(&[("=", "export function greet(name) {"), ("-", "  const greeting = \"hi\";"), ("+", "  const greeting = \"hello\";"), ("=", "  return `${greeting}, ${name}`;")])
    }
    fn edits(list: &[(usize, &str)]) -> std::collections::BTreeMap<usize, String> {
        list.iter().map(|(i, t)| (*i, t.to_string())).collect()
    }

    #[test]
    fn typing_into_a_change() {
        assert_eq!(post_image(&greet()), vec!["export function greet(name) {", "  const greeting = \"hello\";", "  return `${greeting}, ${name}`;"]);
        assert_eq!(editable_rows(&greet()), vec![0, 2, 3]);
        let howdy = edits(&[(2, "  const greeting = \"howdy\";")]);
        let rows = greet();
        assert_eq!(apply_splices(FILE, &[Splice { rows: &rows, edits: &howdy }]), Ok((FILE.replace("\"hello\"", "\"howdy\""), 1)));
        let (text, _) = apply_splices(&FILE[..FILE.len() - 1], &[Splice { rows: &rows, edits: &howdy }]).unwrap();
        assert!(!text.ends_with('\n'));
        let moved = FILE.replace("  const greeting = \"hello\";", "  const greeting = getGreeting();");
        assert!(apply_splices(&moved, &[Splice { rows: &rows, edits: &howdy }]).unwrap_err().contains("not in it any more"));
        let none = edits(&[]);
        assert_eq!(apply_splices(FILE, &[Splice { rows: &rows, edits: &none }]), Ok((FILE.to_string(), 0)));
        let a = self::rows(&[("=", "export function greet(name) {"), ("+", "  const greeting = \"hello\";")]);
        let b = self::rows(&[("=", ""), ("+", "export const NAME = \"world\";")]);
        let (ea, eb) = (edits(&[(1, "  const greeting = \"howdy\";")]), edits(&[(1, "export const NAME = \"earth\";")]));
        let (text, n) = apply_splices(FILE, &[Splice { rows: &a, edits: &ea }, Splice { rows: &b, edits: &eb }]).unwrap();
        assert!(text.contains("\"howdy\"") && text.contains("\"earth\""));
        assert_eq!(n, 2);
        let gone = self::rows(&[("-", "gone")]);
        let back = edits(&[(0, "back")]);
        assert!(apply_splices(FILE, &[Splice { rows: &gone, edits: &back }]).is_err());
        let twice = self::rows(&[("=", "a"), ("+", "x"), ("=", "b")]);
        let y = edits(&[(1, "y")]);
        assert_eq!(apply_splices("a\nx\nb\na\nx\nb\n", &[Splice { rows: &twice, edits: &y }]), Ok(("a\ny\nb\na\nx\nb\n".to_string(), 1)));
        assert_eq!(saved_line("renderer/src/api.ts", 1), "Saved api.ts, 1 line.");
        assert_eq!(saved_line("renderer/src/api.ts", 3), "Saved api.ts, 3 lines.");
    }

    #[test]
    fn a_steps_file_is_a_chip_only_when_the_change_holds_it() {
        // The four tools the engine reads a change from, and nothing else (`changedFile`).
        for tool in ["Edit", "Write", "MultiEdit", "NotebookEdit"] {
            assert!(changes_a_file(tool), "{tool}");
        }
        for tool in ["Read", "Bash", "Grep", "mcp__mercury__update_work_item"] {
            assert!(!changes_a_file(tool), "{tool}");
        }
        const ROOT: &str = "/Users/you/Desktop/dev/zero";
        let paths = ["renderer/src/api.ts", "src/api.ts", "main/ipc.mts"];
        assert_eq!(file_in_change(paths, &format!("{ROOT}/renderer/src/api.ts")).as_deref(), Some("renderer/src/api.ts"));
        assert_eq!(file_in_change(["src/api.ts"], &format!("{ROOT}/src/api.ts")).as_deref(), Some("src/api.ts"));
        assert_eq!(file_in_change(["/tmp/w/a.ts"], "/private/tmp/w/a.ts").as_deref(), Some("/tmp/w/a.ts"));
        assert_eq!(file_in_change(["/private/tmp/w/a.ts"], "/tmp/w/a.ts").as_deref(), Some("/private/tmp/w/a.ts"));
        assert_eq!(file_in_change(paths, &format!("{ROOT}/renderer/src/notes.ts")), None);
        assert_eq!(file_in_change(["pp.tsx"], &format!("{ROOT}/renderer/src/App.tsx")), None);
        assert_eq!(file_in_change([], &format!("{ROOT}/renderer/src/App.tsx")), None);
        assert_eq!(file_in_change(paths, ""), None);
    }

    #[test]
    fn a_file_of_the_change_is_found_as_the_engine_finds_it() {
        let change = Change { files: vec![ChangedFile { path: "renderer/src/api.ts".into(), abs: Some("/work/checkout/renderer/src/api.ts".into()), ..ChangedFile::default() }, file("src/b.ts", 1, 0)], roots: vec!["/work/checkout".into()], ..Change::default() };
        let on_disk = |p: &str| p == "/work/checkout/renderer/src/api.ts" || p == "/docs/src/b.ts";
        assert_eq!(find_file(&change, &[], "renderer/src/api.ts", on_disk), Ok("/work/checkout/renderer/src/api.ts".to_string()));
        assert_eq!(find_file(&change, &["/docs".to_string()], "src/b.ts", on_disk), Ok("/docs/src/b.ts".to_string()));
        assert!(find_file(&change, &[], "src/b.ts", on_disk).unwrap_err().contains("could not be found"));
        assert_eq!(find_file(&change, &[], "../../../etc/hosts", on_disk), Err("That file is not part of this change."));
    }
}
