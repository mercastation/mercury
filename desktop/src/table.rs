//! A sheet opened in the pane beside a conversation, drawn as its table (`TableView` in the
//! pane's mode, `renderer/src/table-file.ts` for what each column holds and the colour of every
//! tag, `components/table-view.css` for the look): the bar with its search, Filter, Sort and how
//! many rows, then the grid, each row numbered, its first column the row's name, figures to the
//! right, a short list of words as coloured tags.
//!
//! Read only: the web window's table is a sheet to edit, whose search, filter, sort, column
//! menus and New row this one draws and does not do yet.

use rune::{El, TextCase, TextStyle, col, el, icon, row, text};

use crate::State;
use crate::theme::*;

/// What a column holds (`ColumnKind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holds {
    Name,
    Tag,
    Number,
    Date,
    Link,
    Email,
    Words,
}

/// A tag's colour (`Tone`). The web window's teal is given by none of its words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Green,
    Blue,
    Purple,
    Amber,
    Rose,
    Grey,
}

pub struct Column {
    pub name: String,
    pub holds: Holds,
    /// A tag column's colours, by each value in lower case.
    tones: Vec<(String, Tone)>,
}

/// A sheet read into its columns and rows (`Table`).
pub struct Grid {
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<String>>,
}

/// RFC 4180 with the separator given: quotes hold separators, doubled quotes and line breaks
/// (`parseDelimited`).
pub fn parse_delimited(text: &str, sep: char) -> Vec<Vec<String>> {
    let src = text.strip_prefix('\u{feff}').unwrap_or(text);
    let (mut rows, mut row, mut cell, mut quoted) = (Vec::new(), Vec::new(), String::new(), false);
    let mut chars = src.chars().peekable();
    while let Some(ch) = chars.next() {
        if quoted {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    cell.push('"');
                    chars.next();
                } else {
                    quoted = false;
                }
            } else {
                cell.push(ch);
            }
            continue;
        }
        if ch == '"' && cell.trim().is_empty() {
            quoted = true;
            cell.clear();
        } else if ch == sep {
            row.push(cell.trim().to_string());
            cell.clear();
        } else if ch == '\n' || ch == '\r' {
            if ch == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            row.push(cell.trim().to_string());
            rows.push(std::mem::take(&mut row));
            cell.clear();
        } else {
            cell.push(ch);
        }
    }
    if !cell.is_empty() || !row.is_empty() {
        row.push(cell.trim().to_string());
        rows.push(row);
    }
    rows.into_iter().filter(|r: &Vec<String>| r.iter().any(|c| !c.is_empty())).collect()
}

/// The separator a sheet uses: a tab for `.tsv`, else whichever of comma, tab and semicolon the
/// first line holds most of outside quotes (`sepFor`).
fn sep_for(text: &str, path: &str) -> char {
    if path.split(['?', '#']).next().unwrap_or("").to_ascii_lowercase().ends_with(".tsv") {
        return '\t';
    }
    let first = text.lines().next().unwrap_or("");
    let mut head = String::new();
    let mut quoted = false;
    for ch in first.chars() {
        if ch == '"' {
            quoted = !quoted;
        } else if !quoted {
            head.push(ch);
        }
    }
    let mut best = (',', head.split(',').count());
    for sep in ['\t', ';'] {
        let n = head.split(sep).count();
        if n > best.1 {
            best = (sep, n);
        }
    }
    best.0
}

fn is_number(v: &str) -> bool {
    let s = v.trim();
    let s = s.strip_prefix(['-', '+']).unwrap_or(s);
    let s = s.strip_prefix(['$', '€', '£']).unwrap_or(s).trim_start();
    let lower = s.to_ascii_lowercase();
    let s = ["%", "k", "m", "bn", "b", "tn", "t"].iter().find_map(|end| lower.strip_suffix(end)).unwrap_or(&lower).trim_end();
    let (whole, part) = s.split_once('.').map_or((s, None), |(a, b)| (a, Some(b)));
    !whole.is_empty() && whole.starts_with(|c: char| c.is_ascii_digit()) && whole.chars().all(|c| c.is_ascii_digit() || c == ',') && part.is_none_or(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

fn is_email(v: &str) -> bool {
    let Some((who, at)) = v.split_once('@') else { return false };
    !who.is_empty() && !who.contains(char::is_whitespace) && at.contains('.') && !at.contains(['@', ' ']) && at.rsplit('.').next().is_some_and(|tld| tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic()))
}

fn is_address(v: &str) -> bool {
    let s = v.strip_prefix("http://").or_else(|| v.strip_prefix("https://")).unwrap_or(v);
    let host = s.split(['/', '?', '#']).next().unwrap_or("");
    let host = host.split(':').next().unwrap_or("");
    !s.contains(char::is_whitespace) && host.contains('.') && host.rsplit('.').next().is_some_and(|tld| tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic())) && host.split('.').all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
}

fn is_nothing(v: &str) -> bool {
    matches!(v.to_ascii_lowercase().as_str(), "n/a" | "na" | "none" | "null" | "nil" | "tbd" | "unknown" | "-" | "\u{2013}" | "\u{2014}" | "?")
}

fn has_word(name: &str, words: &[&str]) -> bool {
    name.to_lowercase().split(|c: char| !c.is_alphanumeric()).any(|w| words.contains(&w))
}

const TAG_HEADER: &[&str] = &["status", "stage", "category", "type", "tier", "priority", "segment", "industry", "tag", "tags", "label", "fit", "source", "plan", "state", "kind", "round", "channel", "owner", "sector", "vertical", "risk", "level", "outcome", "result", "reply", "response", "replied", "sentiment", "intent", "verdict", "decision"];
const TEXT_HEADER: &[&str] = &["name", "company", "investor", "firm", "fund", "person", "contact", "candidate", "employer", "school", "account", "city", "location", "base", "based", "country", "address", "region", "hq", "headquarters", "title", "role", "note", "notes", "description", "summary", "why", "reason", "comment", "email", "phone", "mobile", "tel", "zip", "postal", "postcode", "id", "website", "url", "domain", "linkedin"];
const LABEL_NUMBER: &[&str] = &["phone", "mobile", "tel", "zip", "postal", "postcode", "id", "year"];
const AMOUNT_HEADER: &[&str] = &["revenue", "arr", "mrr", "amount", "price", "cost", "salary", "spend", "budget", "funding", "raised", "valuation", "per", "count", "size"];

fn share(values: &[&str], test: impl Fn(&str) -> bool) -> f32 {
    if values.is_empty() { 0.0 } else { values.iter().filter(|v| test(v)).count() as f32 / values.len() as f32 }
}

/// What a column holds, by its name and its values (`kindOf`). A date is read where it is a day
/// written the way a person writes one; this window reads the ISO day only.
fn holds_of(name: &str, values: &[String], first: bool) -> Holds {
    if first {
        return Holds::Name;
    }
    let filled: Vec<&str> = values.iter().map(|v| v.trim()).filter(|v| !v.is_empty() && !is_nothing(v)).collect();
    if filled.is_empty() {
        return Holds::Words;
    }
    if share(&filled, is_email) >= 0.8 {
        return Holds::Email;
    }
    if share(&filled, |v| is_address(v) && !is_number(v)) >= 0.8 {
        return Holds::Link;
    }
    if !(has_word(name, LABEL_NUMBER) && !has_word(name, AMOUNT_HEADER)) && share(&filled, is_number) >= 0.9 {
        return Holds::Number;
    }
    let iso_day = |v: &str| v.len() == 10 && v.as_bytes()[4] == b'-' && v.as_bytes()[7] == b'-' && v.chars().filter(char::is_ascii_digit).count() == 8;
    if share(&filled, iso_day) >= 0.8 {
        return Holds::Date;
    }
    let short = filled.iter().all(|v| v.chars().count() <= 28 && v.split_whitespace().count() <= 4);
    let mut distinct: Vec<String> = filled.iter().map(|v| v.to_lowercase()).collect();
    distinct.sort();
    distinct.dedup();
    let distinct = distinct.len();
    if short && distinct <= 30 && has_word(name, TAG_HEADER) {
        return Holds::Tag;
    }
    if short && distinct <= 12 && !has_word(name, TEXT_HEADER) && distinct < filled.len() && distinct as f32 / filled.len() as f32 <= 0.6 {
        return Holds::Tag;
    }
    Holds::Words
}

/// A tag's colour: the one its word means, or the next in turn by its place in its column
/// (`tagTone`, its common words).
pub fn tone_of(value: &str, place: usize, header: &str) -> Tone {
    let v = value.trim().to_lowercase();
    let bad_when_high = has_word(header, &["risk", "churn", "severity", "urgency", "cost", "effort", "difficulty", "blocker"]);
    match v.as_str() {
        "very high" | "high" => return if bad_when_high { Tone::Rose } else { Tone::Green },
        "medium" | "med" | "mid" => return Tone::Amber,
        "low" | "very low" => return if bad_when_high { Tone::Green } else { Tone::Grey },
        _ => {}
    }
    if has_word(header, &["bounced", "bounce", "unsubscribed", "unsubscribe", "risk", "blocked", "spam", "lost", "dnc"]) {
        if matches!(v.as_str(), "yes" | "y" | "true") {
            return Tone::Rose;
        }
        if matches!(v.as_str(), "no" | "n" | "false") {
            return Tone::Green;
        }
    }
    let plain = v.replace(['_', '-'], " ").split_whitespace().collect::<Vec<_>>().join(" ");
    let meant: &[(&[&str], Tone)] = &[
        (&["new", "todo", "to do", "not started", "none", "n/a", "unknown", "backlog", "idle", "cold", "untouched", "not contacted"], Tone::Grey),
        (&["done", "won", "closed won", "meeting", "booked", "meeting booked", "meeting set", "yes", "y", "true", "active", "live", "qualified", "hired", "signed", "customer", "paid", "approved", "shipped", "complete", "completed", "hot", "good fit", "strong fit", "great fit", "accepted", "interviewing", "offer", "committed", "funded", "wired", "positive", "healthy", "good", "on track"], Tone::Green),
        (&["lost", "closed lost", "no", "n", "false", "rejected", "blocked", "churned", "failed", "declined", "bounced", "unsubscribed", "disqualified", "cancelled", "canceled", "not a fit", "no fit", "poor fit", "bad fit", "passed", "pass", "dead", "spam", "archived", "do not contact", "dnc", "inactive", "expired", "at risk", "critical", "severe", "off track"], Tone::Rose),
        (&["neutral", "mixed"], Tone::Grey),
        (&["soft circle", "soft circled", "soft commit", "term sheet", "diligence", "in diligence", "due diligence"], Tone::Purple),
        (&["contacted", "sent", "emailed", "in progress", "doing", "reached out", "outreach", "open", "warm", "called", "messaged", "applied", "screening"], Tone::Blue),
        (&["researching", "research", "pending", "waiting", "maybe", "on hold", "review", "in review", "follow up", "later", "nurture", "nurturing", "paused", "snoozed", "needs review"], Tone::Amber),
        (&["replied", "interested", "responded", "engaged", "demo", "demo booked", "proposal", "negotiating", "negotiation", "intro", "introduced"], Tone::Purple),
    ];
    for (words, tone) in meant {
        if words.contains(&v.as_str()) || words.contains(&plain.as_str()) {
            return *tone;
        }
    }
    [Tone::Blue, Tone::Purple, Tone::Amber, Tone::Grey][place % 4]
}

impl Grid {
    /// A sheet's words as a table (`readTable`, `tableFromGrid`): its header row names the
    /// columns, each told what it holds, and a tag column's every value its colour.
    pub fn read(text: &str, path: &str) -> Grid {
        let src = text.strip_prefix('\u{feff}').unwrap_or(text);
        let markdown = src.lines().next().is_some_and(|l| l.trim().starts_with('|') && l.trim().ends_with('|'));
        let grid: Vec<Vec<String>> = if markdown {
            src.lines()
                .filter(|l| l.trim_start().starts_with('|') && !l.trim().trim_matches('|').chars().all(|c| matches!(c, ' ' | '|' | ':' | '-')))
                .map(|l| l.trim().trim_start_matches('|').trim_end_matches('|').split('|').map(|c| c.trim().to_string()).collect())
                .collect()
        } else {
            parse_delimited(src, sep_for(src, path))
        };
        let Some(head) = grid.first() else { return Grid { columns: Vec::new(), rows: Vec::new() } };
        let width = grid.iter().map(Vec::len).max().unwrap_or(0);
        let rows: Vec<Vec<String>> = grid[1..].iter().map(|r| (0..width).map(|i| r.get(i).cloned().unwrap_or_default()).collect()).collect();
        // A column with no header and nothing in it is a trailing separator, not data.
        let keep: Vec<usize> = (0..width).filter(|i| head.get(*i).is_some_and(|h| !h.trim().is_empty()) || rows.iter().any(|r| !r[*i].trim().is_empty())).collect();
        let rows: Vec<Vec<String>> = rows.into_iter().map(|r| keep.iter().map(|i| r[*i].clone()).collect()).collect();
        let mut seen: Vec<String> = Vec::new();
        let columns = keep
            .iter()
            .enumerate()
            .map(|(i, src)| {
                let base = head.get(*src).map(|h| h.trim().to_string()).filter(|h| !h.is_empty()).unwrap_or_else(|| format!("Column {}", i + 1));
                let mut name = base.clone();
                let mut n = 2;
                while seen.contains(&name.to_lowercase()) {
                    name = format!("{base} {n}");
                    n += 1;
                }
                seen.push(name.to_lowercase());
                let values: Vec<String> = rows.iter().map(|r| r[i].clone()).collect();
                let holds = holds_of(&name, &values, i == 0);
                let mut tones: Vec<(String, Tone)> = Vec::new();
                if holds == Holds::Tag {
                    for v in values.iter().map(|v| v.trim()).filter(|v| !v.is_empty()) {
                        let key = v.to_lowercase();
                        if !tones.iter().any(|(k, _)| *k == key) {
                            let tone = tone_of(v, tones.len(), &name);
                            tones.push((key, tone));
                        }
                    }
                }
                Column { name, holds, tones }
            })
            .collect();
        Grid { columns, rows }
    }
}

fn measure(s: &str, style: TextStyle) -> f32 {
    rune::text::measure(s, &[], style, None).0
}

/// Each column's width from the whole file (`columnWidths` with the pane's cap of 220, and
/// `tracks`): its longest words or its widest tag, or its header, with the cell's edges; the
/// column of longest words takes what is spare, else the last one does.
fn widths(grid: &Grid, room: f32) -> Vec<f32> {
    let cap = 220.0;
    let mut px: Vec<f32> = grid
        .columns
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let longest = grid.rows.iter().map(|r| r[i].chars().count()).max().unwrap_or(0);
            let w = if c.holds == Holds::Tag {
                grid.rows.iter().map(|r| measure(r[i].trim(), TextStyle::new(13.0, 400.0)) + 32.0).fold(0.0, f32::max)
            } else {
                grid.rows.iter().filter(|r| r[i].chars().count() as f32 >= longest as f32 * 0.7).map(|r| measure(r[i].trim(), TextStyle::new(14.0, 400.0)).min(cap)).fold(0.0, f32::max)
            };
            let head = c.name.chars().count() as f32 * 7.6 + 20.0 + 24.0;
            (head.max(w) + 34.0).ceil()
        })
        .collect();
    let texts: Vec<usize> = (0..grid.columns.len()).filter(|i| grid.columns[*i].holds == Holds::Words).collect();
    let longest = |i: usize| grid.rows.iter().map(|r| r[i].chars().count()).max().unwrap_or(0);
    let wide = texts.iter().copied().reduce(|best, i| if longest(i) > longest(best) { i } else { best });
    let fills = wide.or(px.len().checked_sub(1));
    let spare = room - 44.0 - px.iter().sum::<f32>();
    if let (Some(at), true) = (fills, spare > 0.0) {
        px[at] += spare;
    }
    px
}

fn tone_colours(tone: Tone) -> (rune::Color, rune::Color, rune::Color) {
    match tone {
        Tone::Green => (rgba(58, 165, 100, 0.16), rgb(0x93, 0xd8, 0xac), rgb(0x3a, 0xa5, 0x64)),
        Tone::Blue => (rgba(76, 127, 214, 0.18), rgb(0xab, 0xc5, 0xf4), rgb(0x4c, 0x7f, 0xd6)),
        Tone::Purple => (rgba(125, 99, 212, 0.2), rgb(0xc8, 0xb8, 0xf3), rgb(0x7d, 0x63, 0xd4)),
        Tone::Amber => (rgba(217, 149, 43, 0.17), rgb(0xed, 0xc5, 0x83), rgb(0xd9, 0x95, 0x2b)),
        Tone::Rose => (rgba(212, 98, 127, 0.18), rgb(0xf1, 0xa8, 0xbb), rgb(0xd4, 0x62, 0x7f)),
        Tone::Grey => (white(0.08), rgb(0xc9, 0xcc, 0xd2), rgb(0x8d, 0x92, 0x9b)),
    }
}

/// The lines between cells (`--tbl-line`) and the ground the row's number and name stand on.
fn line() -> rune::Color {
    white(0.09)
}
fn ground() -> rune::Color {
    rgb(40, 38, 37)
}

fn holds_mark(holds: Holds) -> &'static rune::Icon {
    match holds {
        Holds::Name | Holds::Words => &crate::icons::TABLE_WORDS,
        Holds::Tag => &crate::icons::TABLE_TAG,
        Holds::Number => &crate::icons::TABLE_NUMBER,
        Holds::Date => &crate::icons::TABLE_DATE,
        Holds::Link => &crate::icons::TABLE_LINK,
        Holds::Email => &crate::icons::TABLE_EMAIL,
    }
}

/// A cell: the edge on its right and under it, its words cut short with dots.
fn cell(w: f32, h: f32) -> El {
    row().w(w).h(h).px(16.0).items_center().border_sides([0.0, 1.0, 1.0, 0.0], line()).border_layout()
}

/// What the sheet's search, Filter and Sort say when pressed.
pub const TOOLS_NOT_HERE: &str = "A table is searched, filtered and sorted in Mercury's own window.";

/// The sheet in a box `w` by `h` (`.tbl.tbl-pane`).
pub fn view(s: &State, grid: &Grid, path: &str, (w, h): (f32, f32)) -> El {
    let _ = s;
    let inner = (w - 56.0).max(1.0);
    let rows = grid.rows.len();
    // Searching, filtering and sorting a sheet are the web window's: a press says so, rather
    // than a control that answers the pointer and goes nowhere.
    let control = |glyph: &'static rune::Icon, words: &str| {
        row().h(34.0).px(12.0).gap(6.0).items_center().hover_group().hover(white(0.06)).cursor(rune::Cursor::Pointer).child(icon(glyph, 14.0).color(dim()).group_color(ink())).child(label(words, 13.0, 400.0, dim()).group_color(ink())).on_press_tx(|s: &mut State, tx| s.tell(TOOLS_NOT_HERE, tx))
    };
    let search = row()
        .w(280.0_f32.min(inner).max(140.0))
        .h(34.0)
        .px(10.0)
        .gap(8.0)
        .items_center()
        .border(1.0, rule())
        .border_layout()
        .cursor(rune::Cursor::Text)
        .child(icon(&crate::icons::TABLE_SEARCH, 14.0).color(muted()))
        .child(label(&format!("Search {rows} row{}", if rows == 1 { "" } else { "s" }), 13.0, 400.0, muted()));
    let count = text(format!("{rows} row{}", if rows == 1 { "" } else { "s" })).font(mono(11.0, 14.0).tracking(0.66)).text_case(TextCase::Uppercase).color(muted());
    let bar = row().w(inner).pt(18.0).pb(16.0).gap(6.0).items_center().child(search.on_press_tx(|s: &mut State, tx| s.tell(TOOLS_NOT_HERE, tx))).child(control(&crate::icons::TABLE_FILTER, "Filter")).child(control(&crate::icons::TABLE_SORT, "Sort")).child(el().grow(1.0)).child(count);
    let px = widths(grid, inner - 1.0);
    let head_font = mono(10.5, 10.5).tracking(0.945);
    let mut header = row().child(row().w(44.0).h(41.0).fill(ground()).border_sides([1.0, 1.0, 1.0, 0.0], line()).border_layout());
    for (c, w) in grid.columns.iter().zip(&px) {
        header = header.child(
            row().w(*w).h(41.0).px(16.0).gap(6.0).items_center().fill(ground()).border_sides([1.0, 1.0, 1.0, 0.0], line()).border_layout().child(
                row().min_w(0.0).shrink().items_center().child(icon(holds_mark(c.holds), 13.0).color(muted()).mr(7.0)).child(text(c.name.as_str()).font(head_font).text_case(TextCase::Uppercase).color(muted()).truncate()),
            ),
        );
    }
    let body = grid.rows.iter().enumerate().map(|(r, cells)| {
        let mut line_el = row().child(row().w(44.0).h(46.0).pr(12.0).justify_end().items_center().fill(ground()).border_sides([0.0, 1.0, 1.0, 0.0], line()).border_layout().child(text((r + 1).to_string()).font(figures(12.5, 400.0)).color(white(0.36))));
        for (i, (c, w)) in grid.columns.iter().zip(&px).enumerate() {
            let v = cells[i].trim();
            let mut box_ = cell(*w, 46.0);
            box_ = match c.holds {
                Holds::Name => box_.fill(ground()).child(row().min_w(0.0).shrink().child(text(v).font(TextStyle::new(14.0, 400.0).lh(18.0)).color(ink()).truncate())),
                Holds::Number => box_.justify_end().child(text(v).font(figures(14.0, 400.0).lh(18.0)).color(ink())),
                Holds::Date => box_.child(text(v).font(figures(14.0, 400.0).lh(18.0)).color(dim())),
                Holds::Tag if !v.is_empty() => {
                    let tone = c.tones.iter().find(|(k, _)| *k == v.to_lowercase()).map_or(Tone::Grey, |(_, t)| *t);
                    let (bg, ink_c, dot) = tone_colours(tone);
                    box_.child(row().pl(8.0).pr(9.0).py(2.0).gap(6.0).items_center().fill(bg).child(el().size(6.0, 6.0).fill(dot)).child(text(v).font(TextStyle::new(13.0, 400.0).lh(18.0)).color(ink_c)))
                }
                _ => box_.child(row().min_w(0.0).shrink().child(text(v).font(TextStyle::new(14.0, 400.0).lh(18.0)).color(ink()).truncate())),
            };
            line_el = line_el.child(box_);
        }
        line_el
    });
    let sheet = col().border_sides([0.0, 0.0, 0.0, 1.0], line()).border_layout().child(header).children(body);
    let _ = path;
    col().w(w).h(h).pt(4.0).px(28.0).child(bar).child(el().w(inner).grow(1.0).min_h(0.0).child(el().w(inner).h_full().scroll_xy().child(sheet)))
}

#[cfg(test)]
mod tests {
    use super::*;

    // tests of renderer/src/table-file.ts: quotes, doubled quotes, a semicolon file.
    #[test]
    fn a_sheet_is_read_as_a_spreadsheet_reads_it() {
        assert_eq!(parse_delimited("a,b\n\"1,200\",\"say \"\"hi\"\"\"\n", ','), vec![vec!["a", "b"], vec!["1,200", "say \"hi\""]]);
        assert_eq!(sep_for("\"Company, Inc\";Status\nx;y", "a.csv"), ';');
        assert_eq!(sep_for("a\tb", "a.tsv"), '\t');
        assert_eq!(parse_delimited("a,b\n\n,\n1,2", ','), vec![vec!["a", "b"], vec!["1", "2"]]);
    }

    #[test]
    fn each_column_is_told_what_it_holds_and_each_tag_its_colour() {
        let g = Grid::read("order,total,tax,failed,refund\n1001,1049.99,87.12,yes,8.40\n1002,412.00,34.18,no,\n1003,5902.10,489.87,yes,47.22\n1004,1004.99,83.41,yes,8.04\n", "orders.csv");
        let holds: Vec<Holds> = g.columns.iter().map(|c| c.holds).collect();
        assert_eq!(holds, [Holds::Name, Holds::Number, Holds::Number, Holds::Tag, Holds::Number]);
        assert_eq!(tone_of("yes", 0, "failed"), Tone::Green);
        assert_eq!(tone_of("no", 1, "failed"), Tone::Rose);
        assert_eq!(tone_of("High", 0, "Risk"), Tone::Rose);
        assert_eq!(tone_of("High", 0, "Fit"), Tone::Green);
        assert_eq!(tone_of("Plum", 2, "Colour"), Tone::Amber);
        assert!(is_email("a@example.com") && !is_email("a@b"));
        assert!(is_number("$1,200.50") && is_number("12%") && !is_number("12a"));
    }
}
