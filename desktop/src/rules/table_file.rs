//! A CSV or TSV read into a table: its columns, what each one holds, and the colour of every
//! tag. A port of `renderer/src/table-file.ts`, regex for regex, with its tests'
//! cases; `table.rs` draws it and `table_edit.rs` changes it.

use std::cell::OnceCell;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::rc::Rc;

use super::calendar::Day;
use super::js::{self, re};

/// What a column holds (`ColumnKind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Name,
    Tag,
    Number,
    Date,
    Link,
    Email,
    Text,
}

/// A tag's colour (`Tone`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    Green,
    Blue,
    Purple,
    Amber,
    Rose,
    Teal,
    Grey,
}

impl Tone {
    /// The swatches a tag's page offers, in their order (`TONES` in `TableView.tsx`).
    pub const ALL: [Tone; 7] = [Tone::Grey, Tone::Blue, Tone::Purple, Tone::Teal, Tone::Green, Tone::Amber, Tone::Rose];
    pub fn word(self) -> &'static str {
        match self {
            Tone::Green => "green",
            Tone::Blue => "blue",
            Tone::Purple => "purple",
            Tone::Amber => "amber",
            Tone::Rose => "rose",
            Tone::Teal => "teal",
            Tone::Grey => "grey",
        }
    }
}

/// One column. `head` is the header as the file has it; `name` is unique. `values` are a tag
/// column's values as first written; `tones` is keyed by their lower case.
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub name: String,
    pub head: String,
    pub kind: Kind,
    pub values: Vec<String>,
    pub tones: BTreeMap<String, Tone>,
    pub wrap: bool,
}

impl Column {
    /// A tag's colour, grey for one the column does not know.
    pub fn tone(&self, value: &str) -> Tone {
        self.tones.get(&js::trim(value).to_lowercase()).copied().unwrap_or(Tone::Grey)
    }
}

/// One row's cells, with its words lower-cased once for the search, as the web keeps them in a
/// `WeakMap` so a letter typed reads one string a row.
#[derive(Debug, Default)]
pub struct Cells {
    pub cells: Vec<String>,
    lowered: OnceCell<String>,
}

impl Cells {
    pub fn new(cells: Vec<String>) -> Rc<Cells> {
        Rc::new(Cells { cells, lowered: OnceCell::new() })
    }
    fn lowered(&self) -> &str {
        self.lowered.get_or_init(|| self.cells.join("\u{1}").to_lowercase())
    }
    /// A cell, or no words past the row's end.
    pub fn at(&self, c: usize) -> &str {
        self.cells.get(c).map_or("", String::as_str)
    }
}

impl std::ops::Deref for Cells {
    type Target = Vec<String>;
    fn deref(&self) -> &Vec<String> {
        &self.cells
    }
}

pub type Row = Rc<Cells>;

/// A sheet read (`Table`). `ranked`: the first column is the file's own rank, which stands in
/// for the row number. Cloning shares the rows and the columns, as the web's spread does; two
/// tables are the same one when both are shared (`same`).
#[derive(Clone, Debug)]
pub struct Table {
    pub columns: Rc<Vec<Column>>,
    pub rows: Rc<Vec<Row>>,
    pub logo_col: Option<usize>,
    pub ranked: bool,
}

impl Table {
    pub fn empty() -> Table {
        Table { columns: Rc::new(Vec::new()), rows: Rc::new(Vec::new()), logo_col: None, ranked: false }
    }
    /// The same table, as `===` tells it: nothing in it was made again.
    pub fn same(&self, other: &Table) -> bool {
        Rc::ptr_eq(&self.columns, &other.columns) && Rc::ptr_eq(&self.rows, &other.rows) && self.logo_col == other.logo_col && self.ranked == other.ranked
    }
    /// A cell's words, or none.
    pub fn cell(&self, i: usize, c: usize) -> &str {
        self.rows.get(i).map_or("", |r| r.at(c))
    }
}

/// A sort: a column and its direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Sort {
    pub col: usize,
    pub up: bool,
}

/// Whether a path is a sheet (`isTablePath`): `.csv` or `.tsv`, a query or a fragment after it
/// ignored.
pub fn is_table_path(path: &str) -> bool {
    let bare = path.split(['?', '#']).next().unwrap_or("").to_lowercase();
    bare.ends_with(".csv") || bare.ends_with(".tsv")
}

/// RFC 4180, with the separator given: quotes hold separators, doubled quotes and line breaks
/// (`parseDelimited`).
pub fn parse_delimited(text: &str, sep: char) -> Vec<Vec<String>> {
    let src = text.strip_prefix('\u{feff}').unwrap_or(text);
    let chars: Vec<char> = src.chars().collect();
    let (mut rows, mut row, mut cell, mut quoted): (Vec<Vec<String>>, Vec<String>, String, bool) = (Vec::new(), Vec::new(), String::new(), false);
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if quoted {
            if ch == '"' {
                if chars.get(i + 1) == Some(&'"') {
                    cell.push('"');
                    i += 1;
                } else {
                    quoted = false;
                }
            } else {
                cell.push(ch);
            }
            i += 1;
            continue;
        }
        // `a, "1,200"`: the space a person types after a separator does not stop a quote opening.
        if ch == '"' && js::trim(&cell).is_empty() {
            quoted = true;
            cell.clear();
        } else if ch == sep {
            row.push(js::trim(&cell).to_string());
            cell.clear();
        } else if ch == '\n' || ch == '\r' {
            if ch == '\r' && chars.get(i + 1) == Some(&'\n') {
                i += 1;
            }
            row.push(js::trim(&cell).to_string());
            rows.push(std::mem::take(&mut row));
            cell.clear();
        } else {
            cell.push(ch);
        }
        i += 1;
    }
    if !cell.is_empty() || !row.is_empty() {
        row.push(js::trim(&cell).to_string());
        rows.push(row);
    }
    rows.into_iter().filter(|r| r.iter().any(|c| !c.is_empty())).collect()
}

fn first_line(text: &str) -> &str {
    match text.find(['\n', '\r']) {
        // `\r?\n`: a lone `\r` does not end the first line.
        Some(at) if text[at..].starts_with('\n') || text[at..].starts_with("\r\n") => &text[..at],
        Some(_) => text.split('\n').next().unwrap_or(text).trim_end_matches('\r'),
        None => text,
    }
}

re!(QUOTED, r#""(?:[^"]|"")*""#, "g");

/// The separator a sheet uses (`sepFor`): a tab for `.tsv`; else whichever of comma, tab and
/// semicolon its first line holds most of, counting only those outside quotes.
pub fn sep_for(text: &str, path: &str) -> char {
    if path.split(['?', '#']).next().unwrap_or("").to_lowercase().ends_with(".tsv") {
        return '\t';
    }
    let head = QUOTED.strip_all(first_line(text));
    let mut best = (',', head.split(',').count());
    for sep in ['\t', ';'] {
        let n = head.split(sep).count();
        if n > best.1 {
            best = (sep, n);
        }
    }
    best.0
}

re!(MD_TABLE, r"^\s*\|.*\|\s*$", "");
re!(MD_ROW, r"^\s*\|", "");
re!(MD_RULE, r"^\s*\|[\s|:-]+\|\s*$", "");
re!(MD_LEAD, r"^\|", "");
re!(MD_TAIL, r"(?<!\\)\|$", "");
re!(MD_CUT, r"(?<!\\)\|", "");
re!(MD_PIPE, r"\\\|", "g");

/// A markdown table saved under a `.csv` name: `| a | b |` lines and a `|---|` rule.
pub fn is_markdown_table(text: &str) -> bool {
    MD_TABLE.test(first_line(text.strip_prefix('\u{feff}').unwrap_or(text)))
}

fn parse_markdown_table(text: &str) -> Vec<Vec<String>> {
    let src = text.strip_prefix('\u{feff}').unwrap_or(text);
    src.split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .filter(|l| MD_ROW.test(l) && !MD_RULE.test(l))
        .map(|l| {
            let l = MD_TAIL.strip(&MD_LEAD.strip(js::trim(l)));
            // `\|` is a pipe inside a cell, not the edge of one.
            MD_CUT.split(&l).into_iter().map(|c| MD_PIPE.replace_all(js::trim(c), |_| "|".to_string())).collect()
        })
        .collect()
}

re!(URLISH, r"^(https?:\/\/)?(www\.)?[a-z0-9-]+(\.[a-z0-9-]+)*\.[a-z]{2,}(:\d+)?(\/\S*)?$", "i");
re!(EMAIL, r"^[^\s@]+@[^\s@]+\.[a-z]{2,}$", "i");
re!(NUMBER, r"^[-+]?[$€£]?\s?\d[\d,]*(\.\d+)?\s?(%|k|m|bn|b|t|tn)?$", "i");
re!(MONTH, r"\b(jan|feb|mar|apr|may|jun|jul|aug|sep|sept|oct|nov|dec)[a-z]*\b", "i");
// A cell that says there is nothing here, which a number or date column can hold.
re!(NOTHING, "^(n\\/?a|none|null|nil|tbd|unknown|-|\u{2013}|\u{2014}|\\?)$", "i");
re!(SOCIAL, r"(^|\.)(linkedin\.com|twitter\.com|x\.com|github\.com|facebook\.com|instagram\.com|youtube\.com|crunchbase\.com|angel\.co|wellfound\.com)$", "i");
// A header that names a choice from a short list.
re!(TAG_HEADER, r"\b(status|stage|category|type|tier|priority|segment|industry|tags?|label|fit|source|plan|state|kind|round|channel|owner|sector|vertical|risk|level|outcome|result|reply|response|replied|sentiment|intent|verdict|decision)\b", "i");
// A header that names something each row has its own of, even when values repeat.
re!(TEXT_HEADER, r"\b(name|company|investor|firm|fund|person|contact|candidate|employer|school|account|city|location|base|based|country|address|region|hq|headquarters|title|role|note|notes|description|summary|why|reason|comment|email|phone|mobile|tel|zip|postal|postcode|id|website|url|domain|linkedin)\b", "i");
// A number that is a label, not an amount: never added, sorted or lined up right.
re!(LABEL_NUMBER, r"\b(phone|mobile|tel|zip|postal|postcode|id|year)\b", "i");
// "Revenue per year" is an amount, though it names a year.
re!(AMOUNT_HEADER, r"\b(revenue|arr|mrr|amount|price|cost|salary|spend|budget|funding|raised|valuation|per|count|size)\b", "i");
re!(DATE_HEADER, r"\b(date|day|when|time|updated|created|last|next|due|deadline|founded)\b", "i");

fn share(values: &[&str], test: impl Fn(&str) -> bool) -> f64 {
    if values.is_empty() { 0.0 } else { values.iter().filter(|v| test(v)).count() as f64 / values.len() as f64 }
}

re!(AMOUNT_LEAD, r"^[~≈+]|^(about|approx\.?|circa|ca\.?)\s*", "i");
re!(AMOUNT_TAIL, r"\+$", "");
re!(AMOUNT_MARKS, r"[$€£¥\s]", "g");
re!(AMOUNT, r"^([-+]?[\d.,]+)(%|k|mm|m|bn|b|tn|t)?$", "i");
re!(EUROPEAN, r"^\d{1,3}(\.\d{3})+(,\d+)?$|^\d+,\d{1,2}$", "");
re!(HAS_DIGIT, r"\d", "");
re!(ISO_START, r"^\d{4}-\d{1,2}-\d{1,2}", "");
re!(MONEY, r"[$€£kmbt]", "i");
re!(RANGE, "^(.+?)\\s*(?:-|\u{2013}|\u{2014}|\\bto\\b)\\s*(.+)$", "i");
re!(RANGE_SCALE, r"(k|mm|m|bn|b|tn|t)\+?$", "i");
re!(SCALED, r"[kmbt]\+?$", "i");

fn scale(of: &str) -> f64 {
    match of.to_lowercase().as_str() {
        "k" => 1e3,
        "m" | "mm" => 1e6,
        "b" | "bn" => 1e9,
        "t" | "tn" => 1e12,
        _ => 1.0,
    }
}

/// An amount as an agent writes one: "$1.2M", "~$1M", "€80.000", "1.250.000,50" (`oneAmount`).
fn one_amount(raw: &str) -> f64 {
    // "$7B+" is at least $7B.
    let s = AMOUNT_MARKS.strip_all(&AMOUNT_TAIL.strip(&AMOUNT_LEAD.strip(js::trim(raw))));
    let Some(m) = AMOUNT.exec(&s) else { return f64::NAN };
    let mut n = js::group(&m, 1).to_string();
    if EUROPEAN.test(&n) {
        n = n.replace('.', "").replacen(',', ".", 1);
    } else {
        n = n.replace(',', "");
    }
    js_number(&n).map_or(f64::NAN, |v| v * scale(js::group(&m, 2)))
}

/// `Number(s)` for what `oneAmount` hands it: digits, one point, a sign; anything else is NaN.
fn js_number(s: &str) -> Option<f64> {
    if s.is_empty() || s == "+" || s == "-" || s.matches('.').count() > 1 {
        return if s.is_empty() { Some(0.0) } else { None };
    }
    let body = s.strip_prefix(['+', '-']).unwrap_or(s);
    if body.is_empty() || body == "." || !body.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    s.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// What a cell is worth as an amount, or NaN (`amountOf`). A range counts its low end, and only
/// where it is plainly money, so 2026-10-01 is not one.
pub fn amount_of(v: &str) -> f64 {
    let s = js::trim(v);
    if !HAS_DIGIT.test(s) || ISO_START.test(s) {
        return f64::NAN;
    }
    let range = if MONEY.test(s) { RANGE.exec(s) } else { None };
    let Some(range) = range else { return one_amount(s) };
    // "$12.4-13.2B": the low end shares the high end's B.
    let (low, high) = (js::group(&range, 1), js::group(&range, 2));
    let scale = RANGE_SCALE.exec(js::trim(high)).map(|m| js::group(&m, 1).to_string()).unwrap_or_default();
    let low = if SCALED.test(js::trim(low)) { low.to_string() } else { format!("{low}{scale}") };
    if one_amount(&low).is_finite() { one_amount(&low) } else { one_amount(s) }
}

const DAY_MS: f64 = 86_400_000.0;

/// The UTC midnight of a day the calendar has, or NaN (`realDay`): 2026-13-40 is not rolled over
/// into February.
fn real_day(y: i64, m: i64, d: i64) -> f64 {
    Day::real(y, m - 1, d).map_or(f64::NAN, |day| day.0 as f64 * DAY_MS)
}

/// The day a UTC midnight is.
fn day_of_utc(ms: f64) -> Day {
    Day((ms / DAY_MS).floor() as i64)
}

re!(ISO_DAY, r"^(\d{4})-(\d{1,2})-(\d{1,2})", "");
re!(US_DAY, r"^(\d{1,2})[/.](\d{1,2})[/.](\d{2,4})$", "");
re!(HAS_YEAR, r"\b\d{4}\b", "");

/// When a cell is, as the UTC midnight of its day, or NaN (`dateOf`). A bare number is no date.
/// The rules here ask `date_of_in` with the year read once; this is the web's own, for its tests.
#[cfg(test)]
pub fn date_of(v: &str) -> f64 {
    date_of_in(v, this_year())
}

fn this_year() -> i64 {
    Day::of(crate::clock()).ymd().0
}

fn date_of_in(v: &str, year: i64) -> f64 {
    let s = js::trim(v);
    if s.is_empty() || NUMBER.test(s) {
        return f64::NAN;
    }
    if let Some(m) = ISO_DAY.exec(s) {
        let n = |k: usize| js::group(&m, k).parse::<i64>().unwrap_or(0);
        return real_day(n(1), n(2), n(3));
    }
    if let Some(m) = US_DAY.exec(s) {
        let n = |k: usize| js::group(&m, k).parse::<i64>().unwrap_or(0);
        let y = if js::group(&m, 3).len() == 2 { 2000 + n(3) } else { n(3) };
        // 25/03/2025 can only be day first; 03/04/2025 is read the American way.
        let (month, day) = if n(1) > 12 { (n(2), n(1)) } else { (n(1), n(2)) };
        return real_day(y, month, day);
    }
    if !MONTH.test(s) {
        return f64::NAN;
    }
    // "Jan 5" has no year, and Date.parse would put it in 2001.
    let words = if HAS_YEAR.test(s) { s.to_string() } else { format!("{s} {year}") };
    // Date.parse gives local midnight and the ISO path UTC midnight; one day must be one number.
    legacy_date(&words).map_or(f64::NAN, |day| day.0 as f64 * DAY_MS)
}

/// The day `Date.parse` reads in words, as V8's legacy parser reads them (`dateparser-inl.h`),
/// taken as `new Date(t)` on this machine's clock: a month named by its first three letters, the
/// numbers around it a day and a year, a time and a zone after them, and any word that is none
/// of those allowed only before the first number.
fn legacy_date(s: &str) -> Option<Day> {
    #[derive(Debug, PartialEq)]
    enum Tok {
        Num(i64, usize),
        Word(String),
        Sym(char),
        Space,
    }
    let chars: Vec<char> = s.chars().collect();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_ascii_digit() {
            let start = i;
            let mut n: i64 = 0;
            while i < chars.len() && chars[i].is_ascii_digit() {
                n = n.saturating_mul(10).saturating_add(i64::from(chars[i] as u8 - b'0'));
                i += 1;
            }
            toks.push(Tok::Num(n, i - start));
        } else if c == '(' {
            // Parenthesised text is skipped.
            let mut depth = 0;
            while i < chars.len() {
                if chars[i] == '(' {
                    depth += 1;
                } else if chars[i] == ')' {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                i += 1;
            }
            toks.push(Tok::Space);
        } else if js::is_space(c) {
            while i < chars.len() && js::is_space(chars[i]) {
                i += 1;
            }
            toks.push(Tok::Space);
        } else if c.is_alphabetic() {
            let start = i;
            while i < chars.len() && chars[i].is_alphabetic() {
                i += 1;
            }
            toks.push(Tok::Word(chars[start..i].iter().collect::<String>().to_lowercase()));
        } else {
            toks.push(Tok::Sym(c));
            i += 1;
        }
    }
    const MONTHS: [&str; 12] = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
    enum Word {
        Month(i64),
        AmPm(i64),
        Zone(i64),
        Other,
    }
    // Words are told by their first three letters; only a month may be longer.
    let keyword = |w: &str| {
        let pre: String = w.chars().take(3).collect();
        if let Some(m) = MONTHS.iter().position(|m| *m == pre) {
            return Word::Month(m as i64 + 1);
        }
        if w.chars().count() > 3 {
            return Word::Other;
        }
        match w {
            "am" => Word::AmPm(0),
            "pm" => Word::AmPm(12),
            "ut" | "utc" | "z" | "gmt" => Word::Zone(0),
            "cdt" => Word::Zone(-5 * 60),
            "cst" => Word::Zone(-6 * 60),
            "edt" => Word::Zone(-4 * 60),
            "est" => Word::Zone(-5 * 60),
            "mdt" => Word::Zone(-6 * 60),
            "mst" => Word::Zone(-7 * 60),
            "pdt" => Word::Zone(-7 * 60),
            "pst" => Word::Zone(-8 * 60),
            _ => Word::Other,
        }
    };
    let (mut day, mut named): (Vec<i64>, Option<i64>) = (Vec::new(), None);
    let mut time: Vec<i64> = Vec::new();
    let mut time_final = false;
    let mut hour_offset: Option<i64> = None;
    let mut zone: Option<i64> = None;
    let (mut zone_sign, mut zone_hour, mut zone_minute): (Option<i64>, Option<i64>, Option<i64>) = (None, None, None);
    let mut has_number = false;
    let peek = |k: usize| toks.get(k);
    let mut k = 0;
    // The ES5 form's year at the start, which the legacy reading carries on from.
    if let Some(Tok::Num(n, 4)) = peek(0) {
        if !matches!(peek(1), Some(Tok::Sym('-'))) {
            day.push(*n);
            has_number = true;
            k = 1;
        }
    }
    let time_expects = |time: &Vec<i64>, n: i64| match time.len() {
        1 => (0..60).contains(&n),
        2 => (0..60).contains(&n),
        3 => (0..1000).contains(&n),
        _ => false,
    };
    while k < toks.len() {
        match &toks[k] {
            Tok::Num(n, len) => {
                let n = *n;
                has_number = true;
                k += 1;
                if matches!(peek(k), Some(Tok::Sym(':'))) {
                    k += 1;
                    if matches!(peek(k), Some(Tok::Sym(':'))) {
                        k += 1;
                        if !time.is_empty() {
                            return None;
                        }
                        time.extend([n, 0]);
                    } else {
                        if time_final || time.len() >= 4 {
                            return None;
                        }
                        time.push(n);
                        if matches!(peek(k), Some(Tok::Sym('.'))) {
                            k += 1;
                        }
                    }
                } else if matches!(peek(k), Some(Tok::Sym('.'))) && time_expects(&time, n) {
                    k += 1;
                    time.push(n);
                    let Some(Tok::Num(ms, digits)) = peek(k) else { return None };
                    let ms = match digits {
                        1 => ms * 100,
                        2 => ms * 10,
                        3 => *ms,
                        _ => ms / 10_i64.pow(*digits as u32 - 3),
                    };
                    k += 1;
                    time.push(ms);
                    time_final = true;
                } else if zone_sign.is_some() && zone_minute.is_none() && zone_hour.is_some() && *len <= 2 {
                    zone_minute = Some(n);
                } else if time_expects(&time, n) {
                    time.push(n);
                    time_final = true;
                    // Only the end, a space, Z or a sign may follow a time made whole.
                    match peek(k) {
                        None | Some(Tok::Space) | Some(Tok::Sym('+' | '-')) => {}
                        Some(Tok::Word(w)) if w == "z" => {}
                        _ => return None,
                    }
                } else {
                    if day.len() >= 3 {
                        return None;
                    }
                    day.push(n);
                    if matches!(peek(k), Some(Tok::Sym('-'))) {
                        k += 1;
                    }
                }
            }
            Tok::Word(w) => {
                k += 1;
                match keyword(w) {
                    Word::AmPm(off) if !time.is_empty() => hour_offset = Some(off),
                    Word::Month(m) => {
                        named = Some(m);
                        if matches!(peek(k), Some(Tok::Sym('-'))) {
                            k += 1;
                        }
                    }
                    Word::Zone(off) if has_number => zone = Some(off),
                    _ => {
                        // A word that is none of those is allowed only before the first number,
                        // and never written against it.
                        if has_number || matches!(peek(k), Some(Tok::Num(..))) {
                            return None;
                        }
                    }
                }
            }
            Tok::Sym(c @ ('+' | '-')) if zone == Some(0) || !time.is_empty() => {
                k += 1;
                zone_sign = Some(if *c == '-' { -1 } else { 1 });
                let (mut n, mut len) = (0, 0);
                if let Some(Tok::Num(v, l)) = peek(k) {
                    n = *v;
                    len = *l;
                    k += 1;
                }
                has_number = true;
                if matches!(peek(k), Some(Tok::Sym(':'))) {
                    zone_hour = Some(n);
                    zone_minute = None;
                } else if len == 1 || len == 2 {
                    zone_hour = Some(n);
                    zone_minute = Some(0);
                } else if len == 3 || len == 4 {
                    zone_hour = Some(n / 100);
                    zone_minute = Some(n % 100);
                } else {
                    return None;
                }
            }
            Tok::Sym('+' | '-' | ')') if has_number => return None,
            _ => k += 1,
        }
    }
    // The day: a named month with the numbers around it, or three numbers.
    if day.is_empty() {
        return None;
    }
    let given = day.len();
    while day.len() < 3 {
        day.push(1);
    }
    let is_day = |n: i64| (1..=31).contains(&n);
    let (mut year, month, d) = match named {
        None if given == 3 && !is_day(day[0]) => (day[0], day[1], day[2]),
        None => (if given == 3 { day[2] } else { 0 }, day[0], day[1]),
        Some(m) if given == 1 => (0, m, day[0]),
        Some(m) if !is_day(day[0]) => (day[0], m, day[1]),
        Some(m) => (day[1], m, day[0]),
    };
    if (0..=49).contains(&year) {
        year += 2000;
    } else if (50..=99).contains(&year) {
        year += 1900;
    }
    if !(1..=12).contains(&month) || !is_day(d) {
        return None;
    }
    while time.len() < 4 {
        time.push(0);
    }
    let (mut hour, minute, second, ms) = (time[0], time[1], time[2], time[3]);
    if let Some(off) = hour_offset {
        if !(0..=12).contains(&hour) {
            return None;
        }
        hour = hour % 12 + off;
    }
    let fits = (0..24).contains(&hour) && (0..60).contains(&minute) && (0..60).contains(&second) && (0..1000).contains(&ms);
    if !fits && !(hour == 24 && minute == 0 && second == 0 && ms == 0) {
        return None;
    }
    // `MakeDay` lets a day past the month's end run into the next.
    let days = Day::first_of(year, month - 1).0 + d - 1;
    let wall = (days * 86_400 + hour * 3600 + minute * 60 + second) as f64 * 1000.0 + ms as f64;
    let offset = match (zone_sign, zone_hour, zone) {
        (Some(sign), Some(h), _) => Some(sign * (h * 60 + zone_minute.unwrap_or(0))),
        (None, _, Some(off)) => Some(off),
        _ => None,
    };
    Some(match offset {
        Some(minutes) => Day::of(wall - minutes as f64 * 60_000.0),
        // No zone: the clock on this machine, whose day is the one written (24:00 the next).
        None => Day(days + i64::from(hour == 24)),
    })
}

/// A day as a person types one into a date cell (`dayTyped`): what `dateOf` reads, and today,
/// tomorrow and yesterday; "12/3" is this year's.
pub fn day_typed(v: &str, now: f64) -> f64 {
    let today = Day::of(now);
    let near = match js::trim(v).to_lowercase().as_str() {
        "yesterday" => Some(-1),
        "today" => Some(0),
        "tomorrow" => Some(1),
        _ => None,
    };
    if let Some(n) = near {
        let (y, m, d) = today.ymd();
        return (Day::first_of(y, m).0 + d - 1 + n) as f64 * DAY_MS;
    }
    let year = today.ymd().0;
    let t = js::trim(v);
    if let Some((a, b)) = t.split_once('/')
        && (1..=2).contains(&a.len())
        && (1..=2).contains(&b.len())
        && a.chars().chain(b.chars()).all(|c| c.is_ascii_digit())
    {
        return date_of_in(&format!("{a}/{b}/{year}"), year);
    }
    date_of_in(v, year)
}

const MONTH_NAMES: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const DAY_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
/// The months as the calendar's head names them (`toLocaleDateString` with `month: 'long'`).
pub const MONTHS_LONG: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

re!(ISO_WHOLE, r"^\d{4}-\d{1,2}-\d{1,2}$", "");
re!(SLASH_WHOLE, r"^\d{1,2}\/\d{1,2}\/\d{4}$", "");
re!(WEEKDAY, r"\b(mon|tue|wed|thu|fri|sat|sun)[a-z]*\b", "i");

/// A day picked from the calendar, written the way the column already writes its dates
/// (`dateWords`): ISO, `m/d/yyyy`, or words with a weekday and a year where most of it has them.
pub fn date_words(day: f64, column: &[String]) -> String {
    let at = day_of_utc(day);
    let (y, m, n) = at.ymd();
    let year_now = this_year();
    let filled: Vec<&str> = column.iter().map(|v| js::trim(v)).filter(|v| date_of_in(v, year_now).is_finite()).collect();
    let most = |re: &js::Re| !filled.is_empty() && filled.iter().filter(|v| re.test(v)).count() as f64 / filled.len() as f64 >= 0.6;
    if most(&ISO_WHOLE) {
        return format!("{y}-{:02}-{n:02}", m + 1);
    }
    if most(&SLASH_WHOLE) {
        return format!("{}/{n}/{y}", m + 1);
    }
    // "Mon Nov 9" in a column of those; a year only where the column writes one.
    let weekday = if most(&WEEKDAY) { format!("{} ", DAY_NAMES[at.weekday() as usize]) } else { String::new() };
    let year = if filled.is_empty() || most(&HAS_YEAR) { format!(", {y}") } else { String::new() };
    format!("{weekday}{} {n}{year}", MONTH_NAMES[m as usize])
}

// "Cloud Thu Nov 12" names a launch and its day; picking a new day would lose the name.
re!(DATE_PARTS, r"\b(jan|feb|mar|apr|may|jun|jul|aug|sep|sept|oct|nov|dec|mon|tue|tues|wed|thu|thur|thurs|fri|sat|sun)[a-z]*\b|\d+(st|nd|rd|th)?\b|\b(of|at|am|pm)\b|(?<=\d)t(?=\d)|(?<=\d)z\b|[\s,./:+-]", "gi");
re!(A_LETTER, r"[a-z]", "i");

fn only_a_date(v: &str) -> bool {
    !A_LETTER.test(&DATE_PARTS.strip_all(v))
}

/// Where a cell goes when opened, and the words it shows.
#[derive(Clone, Debug, PartialEq)]
pub struct CellLink {
    pub text: String,
    pub href: String,
}

// One level of brackets inside the address, as a Wikipedia page has.
re!(MD_LINK, r"^\[([^\]]*\S[^\]]*)\]\(((?:[^()\s]|\([^()\s]*\))+)\)$", "");
re!(MAILTO, r"^mailto:", "i");
re!(HTTP, r"^https?:\/\/", "i");
re!(SPACE, r"\s", "");

/// A cell that is wholly `[text](url)`, as an agent writes a link, or none (`markdownLink`).
/// `javascript:` and every other scheme stay words.
pub fn markdown_link(value: &str) -> Option<CellLink> {
    let m = MD_LINK.exec(js::trim(value))?;
    let to = js::group(&m, 2);
    let text = js::trim(js::group(&m, 1)).to_string();
    if MAILTO.test(to) {
        return EMAIL.test(&to[7..]).then(|| CellLink { text, href: to.to_string() });
    }
    let fine = if HTTP.test(to) { !SPACE.test(to) } else { URLISH.test(to) };
    fine.then(|| CellLink { text, href: href_of(to) })
}

/// What a cell reads as: a markdown link's text, or the cell (`shownText`).
pub fn shown_text(value: &str) -> String {
    markdown_link(value).map_or_else(|| js::trim(value).to_string(), |l| l.text)
}

/// What a column holds, by its name and its values (`kindOf`).
fn kind_of(name: &str, values: &[&str], first: bool) -> Kind {
    if first {
        return Kind::Name;
    }
    let year = this_year();
    let filled: Vec<&str> = values.iter().map(|v| js::trim(v)).filter(|v| !v.is_empty() && !NOTHING.test(v)).collect();
    if filled.is_empty() {
        return Kind::Text;
    }
    if share(&filled, |v| markdown_link(v).is_some()) >= 0.8 {
        return Kind::Link;
    }
    if share(&filled, |v| EMAIL.test(v)) >= 0.8 {
        return Kind::Email;
    }
    if share(&filled, |v| URLISH.test(v) && !NUMBER.test(v)) >= 0.8 {
        return Kind::Link;
    }
    if !(LABEL_NUMBER.test(name) && !AMOUNT_HEADER.test(name)) && share(&filled, |v| NUMBER.test(v) || amount_of(v).is_finite()) >= 0.9 {
        return Kind::Number;
    }
    // Names with a day in them read as words, not as dates and not as tags.
    let dated = |v: &str| date_of_in(v, year).is_finite();
    if share(&filled, dated) >= 0.8 && share(&filled, only_a_date) < 0.8 {
        return Kind::Text;
    }
    if share(&filled, |v| dated(v) && only_a_date(v)) >= 0.8 && (DATE_HEADER.test(name) || filled.iter().any(|v| MONTH.test(v) || v.contains('/') || ISO_YEAR.test(v))) {
        return Kind::Date;
    }
    let short = filled.iter().all(|v| js::len(v) <= 28 && js::words(v).len() <= 4);
    let mut distinct: Vec<String> = filled.iter().map(|v| v.to_lowercase()).collect();
    distinct.sort();
    distinct.dedup();
    let distinct = distinct.len();
    if short && distinct <= 30 && TAG_HEADER.test(name) {
        return Kind::Tag;
    }
    if short && distinct <= 12 && !TEXT_HEADER.test(name) && distinct < filled.len() && distinct as f64 / filled.len() as f64 <= 0.6 {
        return Kind::Tag;
    }
    Kind::Text
}
re!(ISO_YEAR, r"^\d{4}-", "");

// The words a pipeline uses keep the colour they mean wherever they appear.
re!(MEANT_GREY, r"^(new|to ?do|not started|none|n\/a|unknown|backlog|idle|cold|untouched|not contacted)$", "i");
re!(MEANT_GREEN, r"^(done|won|closed won|meeting|booked|meeting booked|meeting set|yes|y|true|active|live|qualified|hired|signed|customer|paid|approved|shipped|complete|completed|hot|good fit|strong fit|great fit|accepted|interviewing|offer|committed|funded|wired|positive|healthy|good|on track)$", "i");
re!(MEANT_ROSE, r"^(lost|closed lost|no|n|false|rejected|blocked|churned|failed|declined|bounced|unsubscribed|disqualified|cancelled|canceled|not a fit|no fit|poor fit|bad fit|passed|pass|dead|spam|archived|do not contact|dnc|inactive|expired|at risk|critical|severe|off track)$", "i");
re!(MEANT_NEUTRAL, r"^(neutral|mixed)$", "i");
re!(MEANT_DEAL, r"^(soft circle|soft circled|soft commit|term sheet|diligence|in diligence|due diligence)$", "i");
re!(MEANT_BLUE, r"^(contacted|sent|emailed|in progress|doing|reached out|outreach|open|warm|called|messaged|applied|screening)$", "i");
re!(MEANT_AMBER, r"^(researching|research|pending|waiting|maybe|on hold|review|in review|follow up|follow-up|later|nurture|nurturing|paused|snoozed|needs review)$", "i");
re!(MEANT_PURPLE, r"^(replied|interested|responded|engaged|demo|demo booked|proposal|negotiating|negotiation|intro|introduced)$", "i");
// High and Low mean opposite things under Risk and under Fit.
re!(BAD_WHEN_HIGH, r"\b(risk|churn|severity|urgency|cost|effort|difficulty|blocker)\b", "i");
// Under these a yes is the bad news.
re!(BAD_WHEN_YES, r"\b(bounced?|unsubscribed?|churn\w*|risk|blocked|spam|lost|do not contact|dnc|opted out|complain\w*)\b", "i");
re!(OUTCOME_HEADER, r"\b(status|stage|outcome|result|reply|response|replied|sentiment|intent|verdict|decision|fit|state|health|risk|level|priority|lead|deal|pipeline|progress)\b", "i");
re!(BAD_WORDS, r"\b(not|no|never|un(?:qualified|interested|responsive|reachable|subscribed|successful|fit|happy|willing)|wrong|negative|ghost\w*|reject\w*|declin\w*|bounc\w*|churn\w*|lost|lose|dead|cancel\w*|fail\w*|block\w*|spam|bad|weak|poor|pass(?:ed)? on|disqualif\w*|dropped|killed)\b", "i");
re!(LATER_WORDS, r"\b(not now|later|next quarter|next year|bad timing|timing|follow.?up|on hold|maybe)\b", "i");
re!(GOOD_WORDS, r"\b(won|win|signed|booked|good|strong|great|excellent|qualified|hired|accepted|closed won|converted|success\w*|passed)\b", "i");
re!(LEVEL, r"^(very high|high|medium|med|mid|low|very low)$", "i");
re!(YES, r"^(yes|y|true)$", "i");
re!(NO, r"^(no|n|false)$", "i");
re!(PLAIN_MARKS, r"[\s_-]+", "g");

/// A tag's colour: the one its word means, or the next in turn by its place in its column
/// (`tagTone`). A word nothing knows never takes green, rose or teal.
pub fn tag_tone(value: &str, place: usize, header: &str) -> Tone {
    let v = js::trim(value);
    if let Some(m) = LEVEL.exec(v) {
        let level = js::group(&m, 1).to_lowercase();
        let high = level.contains("high");
        let mid = !high && !level.contains("low");
        if mid {
            return Tone::Amber;
        }
        return if BAD_WHEN_HIGH.test(header) { if high { Tone::Rose } else { Tone::Green } } else if high { Tone::Green } else { Tone::Grey };
    }
    if BAD_WHEN_YES.test(header) {
        if YES.test(v) {
            return Tone::Rose;
        }
        if NO.test(v) {
            return Tone::Green;
        }
    }
    let plain = js::trim(&PLAIN_MARKS.replace_all(v, |_| " ".to_string())).to_string();
    let meant: [(&js::Re, Tone); 8] = [
        (&MEANT_GREY, Tone::Grey),
        (&MEANT_GREEN, Tone::Green),
        (&MEANT_ROSE, Tone::Rose),
        (&MEANT_NEUTRAL, Tone::Grey),
        (&MEANT_DEAL, Tone::Purple),
        (&MEANT_BLUE, Tone::Blue),
        (&MEANT_AMBER, Tone::Amber),
        (&MEANT_PURPLE, Tone::Purple),
    ];
    for (re, tone) in meant {
        if re.test(v) || re.test(&plain) {
            return tone;
        }
    }
    if header.is_empty() || OUTCOME_HEADER.test(header) {
        if LATER_WORDS.test(&plain) {
            return Tone::Amber;
        }
        if BAD_WORDS.test(&plain) {
            return Tone::Rose;
        }
        if GOOD_WORDS.test(&plain) {
            return Tone::Green;
        }
    }
    [Tone::Blue, Tone::Purple, Tone::Amber, Tone::Grey][place % 4]
}

/// A sheet's words as a table (`readTable`).
pub fn read_table(text: &str, path: &str) -> Table {
    if is_markdown_table(text) { table_from_grid(parse_markdown_table(text)) } else { table_from_grid(parse_delimited(text, sep_for(text, path))) }
}

re!(RANK_HEADER, r"^(#|no\.?|n°|rank|ranking|position|pos\.?)$", "i");
re!(RANK, r"^\d{1,4}\.?$", "");

/// A header row and its rows, as a table (`tableFromGrid`).
pub fn table_from_grid(grid: Vec<Vec<String>>) -> Table {
    let Some(head) = grid.first().cloned() else { return Table::empty() };
    // A loop, not the largest of all the rows at once, as the web's comment says.
    let mut width = head.len();
    for r in grid.iter().skip(1) {
        width = width.max(r.len());
    }
    let rows: Vec<Vec<String>> = grid.into_iter().skip(1).map(|r| (0..width).map(|i| r.get(i).cloned().unwrap_or_default()).collect()).collect();
    // A column with no header and nothing in it is a trailing separator, not data.
    let keep: Vec<usize> = (0..width).filter(|i| !js::trim(head.get(*i).map_or("", String::as_str)).is_empty() || rows.iter().any(|r| !js::trim(&r[*i]).is_empty())).collect();
    let rows: Vec<Vec<String>> = rows.into_iter().map(|r| keep.iter().map(|i| r[*i].clone()).collect()).collect();
    let mut seen: Vec<String> = Vec::new();
    let ranked = keep.len() > 1 && RANK_HEADER.test(js::trim(head.get(keep[0]).map_or("", String::as_str))) && rows.iter().all(|r| RANK.test(js::trim(&r[0])));
    let name_at = usize::from(ranked);
    let columns: Vec<Column> = keep
        .iter()
        .enumerate()
        .map(|(i, src)| {
            let header = head.get(*src).cloned().unwrap_or_default();
            let base = if js::trim(&header).is_empty() { format!("Column {}", i + 1) } else { js::trim(&header).to_string() };
            let mut name = base.clone();
            // "Email, Email, Email 2" must not end with two columns called Email 2.
            let mut n = 2;
            while seen.contains(&name.to_lowercase()) {
                name = format!("{base} {n}");
                n += 1;
            }
            seen.push(name.to_lowercase());
            let values: Vec<&str> = rows.iter().map(|r| r[i].as_str()).collect();
            let kind = if ranked && i == 0 { Kind::Number } else { kind_of(&name, &values, i == name_at) };
            let mut tones = BTreeMap::new();
            let mut shown: Vec<String> = Vec::new();
            if kind == Kind::Tag {
                for v in values.iter().map(|x| js::trim(x)).filter(|v| !v.is_empty()) {
                    let key = v.to_lowercase();
                    if tones.contains_key(&key) {
                        continue;
                    }
                    tones.insert(key, tag_tone(v, shown.len(), &name));
                    shown.push(v.to_string());
                }
            }
            Column { name, head: header, kind, values: shown, tones, wrap: false }
        })
        .collect();
    let logo_col = logo_column(&columns, &rows);
    Table { columns: Rc::new(columns), rows: Rc::new(rows.into_iter().map(Cells::new).collect()), logo_col, ranked }
}

/// The address a cell holds, whether bare or as `[text](url)`.
fn address_of(raw: &str) -> String {
    markdown_link(raw).map_or_else(|| js::trim(raw).to_string(), |l| l.href)
}

re!(SCHEME, r"^https?:\/\/", "i");
re!(WWW, r"^www\.", "i");
re!(HOST_END, r"[/:?#]", "");

fn host_of(raw: &str) -> String {
    let s = WWW.strip(&SCHEME.strip(js::trim(raw)));
    HOST_END.split(&s)[0].to_lowercase()
}

re!(NOT_OWN, r"linkedin|twitter|github|crunchbase", "i");
re!(SITE_NAME, r"website|domain|site|homepage", "i");

/// The column whose sites are each row's own (`logoColumn`): named for a website, or links that
/// are not all one network's profile pages.
fn logo_column(columns: &[Column], rows: &[Vec<String>]) -> Option<usize> {
    let links: Vec<usize> = columns.iter().enumerate().filter(|(_, c)| c.kind == Kind::Link && !NOT_OWN.test(&c.name)).map(|(i, _)| i).collect();
    let own = |i: usize| {
        let hosts: Vec<String> = rows.iter().map(|r| address_of(&r[i])).filter(|v| URLISH.test(v)).map(|v| host_of(&v)).collect();
        let hosts: Vec<&str> = hosts.iter().map(String::as_str).collect();
        !hosts.is_empty() && share(&hosts, |h| !SOCIAL.test(h)) >= 0.8
    };
    links.iter().copied().find(|i| SITE_NAME.test(&columns[*i].name) && own(*i)).or_else(|| links.iter().copied().find(|i| own(*i)))
}

/// The host a row's logo comes from, or none (`logoFor`).
pub fn logo_for(table: &Table, row: &Cells) -> Option<String> {
    let at = table.logo_col?;
    let raw = address_of(row.at(at));
    if raw.is_empty() || !URLISH.test(&raw) {
        return None;
    }
    let host = host_of(&raw);
    (!host.is_empty() && !SOCIAL.test(&host)).then_some(host)
}

/// A link cell's address, with its scheme (`hrefOf`).
pub fn href_of(value: &str) -> String {
    let v = js::trim(value);
    if HTTP.test(v) { v.to_string() } else { format!("https://{v}") }
}

re!(TRAILING_SLASH, r"\/$", "");

/// A link cell as it is read: no scheme, no www, no trailing slash (`shownLink`).
pub fn shown_link(value: &str) -> String {
    TRAILING_SLASH.strip(&WWW.strip(&SCHEME.strip(js::trim(value))))
}

/// The one link column a row's name opens, or none when there are none or several
/// (`nameLinkColumn`).
pub fn name_link_column(table: &Table) -> Option<usize> {
    let links: Vec<usize> = table.columns.iter().enumerate().filter(|(_, c)| c.kind == Kind::Link).map(|(i, _)| i).collect();
    (links.len() == 1).then(|| links[0])
}

re!(BARE_HTTP, r"^https?:\/\/\S+$", "i");

/// Where a cell goes when it is opened, and the words it shows, or none (`cellLink`).
pub fn cell_link(table: &Table, row: &Cells, col: usize) -> Option<CellLink> {
    let v = js::trim(row.at(col));
    if v.is_empty() {
        return None;
    }
    if let Some(own) = markdown_link(v) {
        return Some(own);
    }
    match table.columns.get(col).map(|c| c.kind) {
        Some(Kind::Link) => (URLISH.test(v) && !NUMBER.test(v)).then(|| CellLink { text: shown_link(v), href: href_of(v) }),
        Some(Kind::Email) => EMAIL.test(v).then(|| CellLink { text: v.to_string(), href: format!("mailto:{v}") }),
        Some(Kind::Name) => {
            if BARE_HTTP.test(v) && URLISH.test(v) {
                return Some(CellLink { text: shown_link(v), href: v.to_string() });
            }
            let at = name_link_column(table)?;
            cell_link(table, row, at).map(|to| CellLink { text: v.to_string(), href: to.href })
        }
        _ => None,
    }
}

/// The rows, by their place in the file, with the words searched for in any cell and, per
/// column, only the values ticked (`filterRows`).
pub fn filter_rows(table: &Table, query: &str, picked: &BTreeMap<usize, Vec<String>>) -> Vec<usize> {
    let q = js::trim(query).to_lowercase();
    let wants: Vec<(usize, Vec<String>)> = picked.iter().filter(|(_, v)| !v.is_empty()).map(|(k, v)| (*k, v.iter().map(|x| x.to_lowercase()).collect())).collect();
    (0..table.rows.len())
        .filter(|&i| {
            let r = &table.rows[i];
            (q.is_empty() || r.lowered().contains(&q)) && wants.iter().all(|(c, set)| set.contains(&js::trim(r.at(*c)).to_lowercase()))
        })
        .collect()
}

/// Two words in the order a person reads them (`Intl.Collator` with `numeric` and `sensitivity:
/// 'base'`): figures read as the numbers they are, capitals and accents aside, spaces and marks
/// before figures and figures before letters, as the root collation orders them.
pub fn collate(a: &str, b: &str) -> Ordering {
    order_words(a, b, true)
}

/// `a.localeCompare(b)`: the same order with figures read one by one, and a lower case before
/// its capital where nothing else tells two apart.
pub fn locale_compare(a: &str, b: &str) -> Ordering {
    order_words(a, b, false).then_with(|| b.cmp(a))
}

fn order_words(a: &str, b: &str, numeric: bool) -> Ordering {
    fn class(c: char) -> u8 {
        if c.is_ascii_digit() {
            1
        } else if c.is_alphabetic() {
            2
        } else {
            0
        }
    }
    let fold = |s: &str| -> Vec<char> { s.chars().flat_map(char::to_lowercase).filter(|c| !('\u{300}'..='\u{36f}').contains(c)).map(super::files_shown::bare_letter).collect() };
    let (x, y) = (fold(a), fold(b));
    let (mut i, mut j) = (0, 0);
    while i < x.len() && j < y.len() {
        let (c, d) = (x[i], y[j]);
        if numeric && c.is_ascii_digit() && d.is_ascii_digit() {
            let run =|v: &[char], mut k: usize| {
                let start = k;
                while k < v.len() && v[k].is_ascii_digit() {
                    k += 1;
                }
                (v[start..k].iter().collect::<String>(), k)
            };
            let ((m, ni), (n, nj)) = (run(&x, i), run(&y, j));
            let (m, n) = (m.trim_start_matches('0'), n.trim_start_matches('0'));
            let by = m.len().cmp(&n.len()).then_with(|| m.cmp(n));
            if by != Ordering::Equal {
                return by;
            }
            (i, j) = (ni, nj);
            continue;
        }
        let by = class(c).cmp(&class(d)).then_with(|| c.cmp(&d));
        if by != Ordering::Equal {
            return by;
        }
        i += 1;
        j += 1;
    }
    (x.len() - i).cmp(&(y.len() - j))
}

/// The rows given, in the order a sort puts them (`sortRows`): numbers by amount, days by day,
/// the rest as words are read. Blank, N/A and anything that does not read as the column's kind
/// go last, either way; rows that tie keep their order.
pub fn sort_rows(table: &Table, rows: Vec<usize>, sort: Option<Sort>) -> Vec<usize> {
    let Some(Sort { col, up }) = sort else { return rows };
    let kind = table.columns.get(col).map(|c| c.kind);
    let year = this_year();
    let value_of = |v: &str| match kind {
        Some(Kind::Number) => Some(amount_of(v)),
        Some(Kind::Date) => Some(date_of_in(v, year)),
        _ => None,
    };
    struct Keyed {
        i: usize,
        v: String,
        n: f64,
        empty: bool,
    }
    let mut keyed: Vec<Keyed> = rows
        .into_iter()
        .map(|i| {
            let v = shown_text(table.cell(i, col));
            let n = if v.is_empty() { f64::NAN } else { value_of(&v).unwrap_or(f64::NAN) };
            let valued = matches!(kind, Some(Kind::Number | Kind::Date));
            let empty = v.is_empty() || NOTHING.test(&v) || (valued && !n.is_finite());
            Keyed { i, v, n, empty }
        })
        .collect();
    keyed.sort_by(|a, b| {
        if a.empty || b.empty {
            return if a.empty == b.empty { Ordering::Equal } else if a.empty { Ordering::Greater } else { Ordering::Less };
        }
        let by = if matches!(kind, Some(Kind::Number | Kind::Date)) { a.n.partial_cmp(&b.n).unwrap_or(Ordering::Equal) } else { collate(&a.v, &b.v) };
        if up { by } else { by.reverse() }
    });
    keyed.into_iter().map(|k| k.i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(t: &Table) -> Vec<String> {
        t.columns.iter().map(|c| c.name.clone()).collect()
    }
    fn kinds(text: &str) -> Vec<Kind> {
        read_table(text, "x.csv").columns.iter().map(|c| c.kind).collect()
    }
    fn first_cells(t: &Table, order: &[usize]) -> Vec<String> {
        order.iter().map(|i| t.cell(*i, 0).to_string()).collect()
    }
    fn utc(y: i64, m: i64, d: i64) -> f64 {
        (Day::first_of(y, m).0 + d - 1) as f64 * DAY_MS
    }
    fn rows(t: &Table) -> Vec<Vec<String>> {
        t.rows.iter().map(|r| r.cells.clone()).collect()
    }

    const LEADS: &str = "Company,Category,Based in,Note,Status,Website,People\r\nRamp,Spend,New York,Asked for a demo last week,Meeting,ramp.com,1200\r\nMercury,Banking,San Francisco,\"Replied, wants pricing\",Replied,https://mercury.com/,800\r\nPlaid,Payments,San Francisco,Opened the email twice,Contacted,plaid.com,1100\r\nBrex,Spend,San Francisco,No contact yet,New,www.brex.com,1000\r\nDeel,Payroll,San Francisco,\"Intro through an \"\"investor\"\"\",New,deel.com,4000\r\nUnit,Banking,New York,No contact yet,New,unit.co,150\r\n";

    // tests/a-csv-an-agent-writes-is-drawn-as-a-table.test.mts
    #[test]
    fn which_files_are_tables() {
        for p in ["leads/fintech.csv", "export.TSV", "export.csv?v=2", "export.csv#row-3"] {
            assert!(is_table_path(p), "{p}");
        }
        for p in ["notes.md", "data.json", "sheet.xlsx", "csv", "my.csv.bak", ""] {
            assert!(!is_table_path(p), "{p}");
        }
    }

    #[test]
    fn the_text_is_split_as_a_spreadsheet_splits_it() {
        assert_eq!(parse_delimited("a,b\n\"x, y\",\"he said \"\"hi\"\"\"\n\"two\nlines\",z\n", ','), vec![vec!["a", "b"], vec!["x, y", "he said \"hi\""], vec!["two\nlines", "z"]]);
        assert_eq!(parse_delimited("\u{feff}a,b\r\n1,2\r\n", ','), vec![vec!["a", "b"], vec!["1", "2"]]);
        assert_eq!(names(&read_table("name\tstatus\nA\tNew\nB\tNew\n", "x.csv")), ["name", "status"]);
    }

    #[test]
    fn each_column_is_told_what_it_holds() {
        let t = read_table(LEADS, "fintech-leads.csv");
        let kind = |n: &str| t.columns.iter().find(|c| c.name == n).map(|c| c.kind);
        assert_eq!(t.rows.len(), 6);
        assert_eq!(t.cell(1, 3), "Replied, wants pricing");
        assert_eq!(t.cell(4, 3), "Intro through an \"investor\"");
        assert_eq!((kind("Category"), kind("Status")), (Some(Kind::Tag), Some(Kind::Tag)));
        assert_eq!((kind("Note"), kind("Company"), kind("Based in")), (Some(Kind::Text), Some(Kind::Name), Some(Kind::Text)));
        assert_eq!((kind("People"), kind("Website")), (Some(Kind::Number), Some(Kind::Link)));
        assert_eq!(logo_for(&t, &t.rows[0]).as_deref(), Some("ramp.com"));
        assert_eq!(logo_for(&t, &t.rows[1]).as_deref(), Some("mercury.com"));
        assert_eq!(logo_for(&t, &t.rows[3]).as_deref(), Some("brex.com"));
        let plain = read_table("Name,Status\nA,New\nB,New\nC,Won\n", "x.csv");
        assert_eq!(logo_for(&plain, &plain.rows[0]), None);
        assert_eq!(read_table("Name,Why\nA,raised a round in march\nB,hired a new head of sales\nC,opened an office in london\n", "x.csv").columns[1].kind, Kind::Text);
        assert_eq!(read_table("Name,Stage\nA,Seed\nB,Series A\nC,Series B\n", "x.csv").columns[1].kind, Kind::Tag);
        let u = read_table("Name,,Status\nA\nB,x,New\n", "x.csv");
        assert_eq!(u.rows[0].cells, ["A", "", ""]);
        assert_eq!(u.columns[1].name, "Column 2");
        assert!(read_table("", "x.csv").rows.is_empty() && read_table("", "x.csv").columns.is_empty());
    }

    #[test]
    fn files_the_way_agents_really_write_them() {
        let t = read_table("Company, Website, Status, ARR\nAcme, acme.com, New, \"$1,200,000\"\nBeta, beta.io, New, \"$90,000\"\n", "x.csv");
        assert_eq!(t.columns.len(), 4);
        assert_eq!(t.cell(0, 3), "$1,200,000");
        assert_eq!(names(&read_table("Name;Status\nA;New\nB;Won\n", "x.csv")), ["Name", "Status"]);
        let md = read_table("| Company | Website |\n|---|---|\n| Ramp | ramp.com |\n| Plaid | plaid.com |\n", "x.csv");
        assert_eq!(names(&md), ["Company", "Website"]);
        assert_eq!(rows(&md), [["Ramp", "ramp.com"], ["Plaid", "plaid.com"]]);
        assert_eq!(names(&read_table("A,B,\n1,x,\n2,y,\n", "x.csv")), ["A", "B"]);
        assert_eq!(names(&read_table("Name,Email,Email\nA,a@x.com,b@x.com\n", "x.csv")), ["Name", "Email", "Email 2"]);
        let t = read_table("Name,ARR\nA,$5k\nB,N/A\nC,-$5k\nD,$0\nE,\u{2014}\n", "x.csv");
        assert_eq!(t.columns[1].kind, Kind::Number);
        assert_eq!(first_cells(&t, &sort_rows(&t, (0..5).collect(), Some(Sort { col: 1, up: true }))), ["C", "D", "A", "B", "E"]);
        let t = read_table("Name,Last touch\nA,Mar 3 2026\nB,Dec 1 2025\nC,Mar 3 2026\nD,1/15/2026\n", "x.csv");
        assert_eq!(t.columns[1].kind, Kind::Date);
        assert_eq!(first_cells(&t, &sort_rows(&t, (0..4).collect(), Some(Sort { col: 1, up: true }))), ["B", "D", "A", "C"]);
        let many: String = (0..40).map(|i| format!("R{i},Stage {}\n", i % 20)).collect();
        assert_eq!(kinds(&format!("Name,Status\n{many}"))[1], Kind::Tag);
        assert_eq!(kinds("Name,Investor,Phone,Zip\nA,Sequoia,+1 415 555 0100,94107\nB,Sequoia,415-555-0101,94107\nC,Accel,(415) 555 0102,10001\nD,Accel,4155550103,10001\n"), [Kind::Name, Kind::Text, Kind::Text, Kind::Text]);
        let t = read_table("Name,LinkedIn,Status\nAna,https://linkedin.com/in/ana,New\nBo,https://linkedin.com/in/bo,New\n", "x.csv");
        assert_eq!(logo_for(&t, &t.rows[0]), None);
        let t = read_table("Name,Status\nA,New\nB,new\nC,NEW\nD,Won\n", "x.csv");
        assert_eq!(t.columns[1].values, ["New", "Won"]);
        assert_eq!(first_cells(&t, &filter_rows(&t, "", &BTreeMap::from([(1, vec!["New".to_string()])]))), ["A", "B", "C"]);
    }

    #[test]
    fn what_a_second_model_found() {
        let big: String = std::iter::once("Name,Status\n".to_string()).chain((0..150_000).map(|i| format!("R{i},{}\n", if i % 2 == 1 { "New" } else { "Won" }))).collect();
        let t = read_table(&big, "x.csv");
        assert_eq!(t.rows.len(), 150_000);
        assert_eq!(names(&t), ["Name", "Status"]);
        assert_eq!(names(&read_table("\"Company, Inc\";Status\nAcme;Won\nBeta;Lost\n", "x.csv")), ["Company, Inc", "Status"]);
        assert_eq!(names(&read_table("\"A; B\",C\n1,2\n", "x.csv")), ["A; B", "C"]);
        for v in ["Unassigned", "Under review", "Unicorn"] {
            assert_ne!(tag_tone(v, 0, ""), Tone::Rose, "{v}");
        }
        for v in ["Unqualified", "Uninterested", "Unresponsive", "Unsubscribed"] {
            assert_eq!(tag_tone(v, 0, ""), Tone::Rose, "{v}");
        }
    }

    #[test]
    fn tag_colours() {
        for i in 0..12 {
            let t = tag_tone(&format!("Thing {i}"), i, "");
            assert!(!matches!(t, Tone::Green | Tone::Teal | Tone::Rose), "{t:?}");
        }
        assert_eq!(tag_tone("TRUE", 0, "Churn risk"), Tone::Rose);
        assert_eq!(tag_tone("FALSE", 0, "Churn risk"), Tone::Green);
        assert_eq!(tag_tone("Critical", 0, "Risk level"), Tone::Rose);
        assert_eq!(tag_tone("Passed phone screen", 0, "Stage"), Tone::Green);
        for v in ["Non-profit", "Customer Success", "Win-back"] {
            assert!(!matches!(tag_tone(v, 0, "Industry"), Tone::Green | Tone::Rose), "{v}");
        }
        assert_eq!(tag_tone("Committed", 0, "Status"), Tone::Green);
        assert_eq!(tag_tone("Soft circle", 0, "Status"), Tone::Purple);
        assert_eq!(tag_tone("Positive", 0, "Sentiment"), Tone::Green);
        assert_eq!(tag_tone("Healthy", 0, "Health"), Tone::Green);
        assert_eq!(tag_tone("At risk", 0, "Health"), Tone::Rose);
        for v in ["Not a fit", "Passed", "FALSE", "Declined"] {
            assert_eq!(tag_tone(v, 0, ""), Tone::Rose, "{v}");
        }
        assert_eq!(tag_tone("TRUE", 3, ""), Tone::Green);
        assert_eq!(tag_tone("Nurture", 0, ""), Tone::Amber);
        assert_eq!(tag_tone("High", 0, "Churn risk"), Tone::Rose);
        assert_eq!(tag_tone("Low", 0, "Churn risk"), Tone::Green);
        assert_eq!(tag_tone("High", 0, "Fit"), Tone::Green);
        assert_eq!(tag_tone("Medium", 0, "Fit"), Tone::Amber);
        for v in ["Unqualified", "Wrong person", "Not interested", "Closed - Lost", "Negative", "Ghosted", "Bad", "Weak"] {
            assert_eq!(tag_tone(v, 4, ""), Tone::Rose, "{v}");
        }
        assert_eq!(tag_tone("Not now", 4, ""), Tone::Amber);
        for v in ["Closed-Won", "Strong", "Good"] {
            assert_eq!(tag_tone(v, 1, ""), Tone::Green, "{v}");
        }
        assert_eq!(tag_tone("TRUE", 0, "Bounced"), Tone::Rose);
        assert_eq!(tag_tone("FALSE", 0, "Bounced"), Tone::Green);
        assert_eq!(tag_tone("Yes", 0, "Unsubscribed"), Tone::Rose);
        assert_eq!(tag_tone("Yes", 0, "Warm intro?"), Tone::Green);
        let outcomes = ["Won", "Lost", "Ghosted", "Not now", "Won", "Bad timing", "Lost", "Signed", "Passed", "Won", "Pilot", "Churned"];
        let body: String = outcomes.iter().enumerate().map(|(i, r)| format!("C{i},{r},{}\n", ["Yes", "No", "Later"][i % 3])).collect();
        assert_eq!(kinds(&format!("Name,Outcome,Reply\n{body}")), [Kind::Name, Kind::Tag, Kind::Tag]);
        assert_eq!(tag_tone("Meeting", 0, ""), Tone::Green);
        assert_eq!(tag_tone("New", 3, ""), Tone::Grey);
        assert_eq!(tag_tone("Contacted", 0, ""), Tone::Blue);
        assert_eq!(tag_tone("Researching", 0, ""), Tone::Amber);
        assert_eq!(tag_tone("Replied", 0, ""), Tone::Purple);
        assert_ne!(tag_tone("Spend", 0, ""), tag_tone("Banking", 1, ""));
        let t = read_table(LEADS, "x.csv");
        let cat = t.columns.iter().find(|c| c.name == "Category").unwrap();
        assert_eq!(cat.values, ["Spend", "Banking", "Payments", "Payroll"]);
        let tones: std::collections::HashSet<Tone> = cat.values.iter().map(|v| cat.tone(v)).collect();
        assert_eq!(tones.len(), 4);
    }

    #[test]
    fn what_the_last_power_user_loop_found() {
        let t = read_table("Name,Signed\nA,25/03/2025\nB,2025-11-01\nC,14/03/2025\nD,2026-01-02\n", "x.csv");
        assert_eq!(t.columns[1].kind, Kind::Date);
        assert_eq!(first_cells(&t, &sort_rows(&t, (0..4).collect(), Some(Sort { col: 1, up: true }))), ["C", "A", "B", "D"]);
        let y = read_table("Name,Next call\nA,Dec 12\nB,Jan 5\n", "x.csv");
        assert_eq!(day_of_utc(date_of("Jan 5")).ymd().0, this_year());
        assert_eq!(first_cells(&y, &sort_rows(&y, vec![0, 1], Some(Sort { col: 1, up: true }))), ["B", "A"]);
        let t = read_table("Investor,Check size\nA,$250k-$1M\nB,~$1M\nC,TBD\nD,$1.5M\nE,$500K\n", "x.csv");
        assert_eq!(t.columns[1].kind, Kind::Number);
        assert_eq!(first_cells(&t, &sort_rows(&t, (0..5).collect(), Some(Sort { col: 1, up: true }))), ["A", "E", "B", "D", "C"]);
        let eu = read_table("Name;Salary\nA;€80.000\nB;€120.000\nC;€9.500,50\n", "x.csv");
        assert_eq!(eu.columns[1].kind, Kind::Number);
        assert_eq!(first_cells(&eu, &sort_rows(&eu, (0..3).collect(), Some(Sort { col: 1, up: true }))), ["C", "A", "B"]);
        let t = read_table("| Company | Stage | Website |\n|---|---|---|\n| Pipe \\| Inc | New | pipe.com |\n| Ramp | Won | ramp.com |\n", "x.csv");
        assert_eq!(names(&t), ["Company", "Stage", "Website"]);
        assert_eq!(t.cell(0, 0), "Pipe | Inc");
        let ns = names(&read_table("Name,Email,Email,Email 2\nA,a,b,c\n", "x.csv"));
        let unique: std::collections::HashSet<&String> = ns.iter().collect();
        assert_eq!(unique.len(), ns.len());
    }

    #[test]
    fn searching_filtering_and_sorting() {
        let t = read_table(LEADS, "x.csv");
        assert_eq!(first_cells(&t, &filter_rows(&t, "francisco", &BTreeMap::new())), ["Mercury", "Plaid", "Brex", "Deel"]);
        let status = t.columns.iter().position(|c| c.name == "Status").unwrap();
        assert_eq!(first_cells(&t, &filter_rows(&t, "", &BTreeMap::from([(status, vec!["New".to_string()])]))), ["Brex", "Deel", "Unit"]);
        let people = t.columns.iter().position(|c| c.name == "People").unwrap();
        assert_eq!(first_cells(&t, &sort_rows(&t, (0..6).collect(), Some(Sort { col: people, up: true }))), ["Unit", "Mercury", "Brex", "Plaid", "Ramp", "Deel"]);
        assert_eq!(first_cells(&t, &sort_rows(&t, (0..6).collect(), Some(Sort { col: 0, up: false })))[0], "Unit");
        assert_eq!(sort_rows(&t, vec![3, 1], None), [3, 1]);
    }

    // tests/a-tables-tags-types-rows-and-columns-can-all-be-changed.test.mts
    #[test]
    fn a_day_picked_is_written_the_way_the_column_writes_dates() {
        let day = utc(2026, 10, 12);
        let col = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(date_words(day, &col(&["2026-11-01", "2026-10-30", ""])), "2026-11-12");
        assert_eq!(date_words(day, &col(&["11/1/2026", "10/30/2026"])), "11/12/2026");
        assert_eq!(date_words(day, &col(&["Nov 1, 2026", "Oct 30, 2026"])), "Nov 12, 2026");
        assert_eq!(date_words(day, &[]), "Nov 12, 2026");
        assert_eq!(date_words(day, &col(&["Mon Nov 9", "Sat Nov 7", ""])), "Thu Nov 12");
        assert_eq!(date_words(day, &col(&["Cloud Thu Nov 12", "Multiplayer Tue Nov 17"])), "Thu Nov 12");
        assert_eq!(date_words(day, &col(&["Nov 9", "Oct 30"])), "Nov 12");
        assert_eq!(date_words(day, &col(&["Mon, Nov 9, 2026"])), "Thu Nov 12, 2026");
    }

    #[test]
    fn today_tomorrow_and_yesterday_are_days() {
        let now = Day::real(2026, 10, 7).unwrap().at(15, 0);
        assert_eq!(day_typed("tomorrow", now), utc(2026, 10, 8));
        assert_eq!(day_typed(" Today ", now), utc(2026, 10, 7));
        assert_eq!(day_typed("yesterday", now), utc(2026, 10, 6));
        assert!(day_typed("after the demo", now).is_nan());
        assert_eq!(day_typed("12/3", now), utc(2026, 11, 3));
        assert!(day_typed("2/30", now).is_nan());
        assert_eq!(day_typed("Nov 30, 2026", now), utc(2026, 10, 30));
    }

    #[test]
    fn a_column_of_names_with_a_day_in_them_is_not_dates() {
        assert_eq!(read_table("Goal,Launch\nA,Cloud Thu Nov 12\nB,Multiplayer Tue Nov 17\nC,Cloud Thu Nov 12\nD,Both\n", "x.csv").columns[1].kind, Kind::Text);
        assert_eq!(read_table("Goal,By\nA,Mon Nov 9\nB,Sat Nov 7\nC,Thu Nov 12\n", "x.csv").columns[1].kind, Kind::Date);
        assert_eq!(read_table("Goal,Due\nA,2026-11-12T10:30:00Z\nB,2026-11-13\n", "x.csv").columns[1].kind, Kind::Date);
        for v in ["2026-13-40", "2/30/2026", "2026-02-29", "13/13/2026"] {
            assert!(date_of(v).is_nan(), "{v}");
        }
        assert_eq!(date_of("2028-02-29"), utc(2028, 1, 29));
        assert_eq!(date_of("25/03/2025"), utc(2025, 2, 25));
        let day = utc(2026, 10, 12);
        for v in ["2026-11-12", "11/12/2026", "Nov 12, 2026", "November 12 2026", "Thu Nov 12 2026"] {
            assert_eq!(date_of(v), day, "{v}");
        }
        assert_eq!(date_of(&date_words(day, &[])), day);
        // Words after a figure are no day, as V8 reads them; a weekday before one is.
        assert!(date_of("Nov 12th 2026").is_nan());
        assert_eq!(date_of("Mon, Nov 9, 2026"), utc(2026, 10, 9));
        assert_eq!(date_of("12 November 2026"), day);
        assert_eq!(date_of("2026 Nov 12"), day);
    }

    // tests/a-link-in-a-table-cell-opens-from-its-text.test.mts
    #[test]
    fn a_cell_written_text_and_url_reads_as_its_text_and_address() {
        assert_eq!(markdown_link("[Watch](https://www.example.com/@a/video/1)"), Some(CellLink { text: "Watch".into(), href: "https://www.example.com/@a/video/1".into() }));
        assert_eq!(markdown_link("  [Acme](acme.com) "), Some(CellLink { text: "Acme".into(), href: "https://acme.com".into() }));
        assert_eq!(markdown_link("[Paris](https://en.wikipedia.org/wiki/Paris_(disambiguation))").map(|l| l.href).as_deref(), Some("https://en.wikipedia.org/wiki/Paris_(disambiguation)"));
        assert_eq!(shown_text("[Watch](https://x.com/a)"), "Watch");
        for v in ["see [Watch](https://x.com/a) later", "[Watch]", "[Watch](not a link)", "[Run](javascript:alert(1))", "[](https://x.com)"] {
            assert_eq!(markdown_link(v), None, "{v}");
        }
        assert_eq!(shown_text("[draft] notes"), "[draft] notes");
        let g = |rows: &[&[&str]]| table_from_grid(rows.iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect());
        let t = g(&[&["Video", "Notes"], &["[Founder skit](https://example.com/@a/video/1)", "[Script](https://docs.example.com/s)"], &["Plain", "Nothing here"]]);
        assert_eq!(cell_link(&t, &t.rows[0], 0).map(|l| l.text).as_deref(), Some("Founder skit"));
        assert_eq!(cell_link(&t, &t.rows[0], 1).map(|l| l.href).as_deref(), Some("https://docs.example.com/s"));
        assert_eq!(cell_link(&t, &t.rows[1], 0), None);
        assert_eq!(cell_link(&t, &t.rows[1], 1), None);
        let t = g(&[&["Name", "Video"], &["a", "[Zebra](https://z.com)"], &["b", "[Apple](https://a.com)"], &["c", "[Mango](https://m.com)"]]);
        assert_eq!(t.columns[1].kind, Kind::Link);
        assert_eq!(first_cells(&t, &sort_rows(&t, vec![0, 1, 2], Some(Sort { col: 1, up: true }))), ["b", "c", "a"]);
        let t = g(&[&["Company", "Website"], &["Acme", "[acme.com](https://acme.com)"], &["Globex", "[globex.io](https://globex.io)"]]);
        assert_eq!(logo_for(&t, &t.rows[0]).as_deref(), Some("acme.com"));
    }

    #[test]
    fn a_rows_name_opens_its_one_link() {
        let videos = "Example,Link,Format,Views\n@maker: a skit about founders,https://www.example.com/@maker/video/111,Skit,100\n@builder: three agents at once,https://www.example.com/@builder/video/222,Parallel,200\n@coder: a dating skit,,Skit,300";
        let t = read_table(videos, "viral.csv");
        assert_eq!(cell_link(&t, &t.rows[0], 0), Some(CellLink { text: "@maker: a skit about founders".into(), href: "https://www.example.com/@maker/video/111".into() }));
        assert_eq!(cell_link(&t, &t.rows[1], 0).map(|l| l.href).as_deref(), Some("https://www.example.com/@builder/video/222"));
        assert_eq!(cell_link(&t, &t.rows[2], 0), None);
        let t = read_table("Company,Website,LinkedIn\nAcme,acme.com,https://linkedin.com/company/acme\nGlobex,globex.io,https://linkedin.com/company/globex", "leads.csv");
        assert_eq!((t.columns[1].kind, t.columns[2].kind), (Kind::Link, Kind::Link));
        assert_eq!(cell_link(&t, &t.rows[0], 0), None);
        let t = read_table("Name,Stage\nAcme,Won\nGlobex,Lost\nInitech,Won", "deals.csv");
        assert_eq!(cell_link(&t, &t.rows[0], 0), None);
        let t = read_table("Example,Link\n[Own](https://own.example.com),https://other.example.com\nB,https://b.example.com", "v.csv");
        assert_eq!(cell_link(&t, &t.rows[0], 0).map(|l| l.href).as_deref(), Some("https://own.example.com"));
        let t = read_table("Video,Views\nhttps://www.example.com/@a/video/1,100\nhttps://www.example.com/@b/video/2,200", "v.csv");
        assert_eq!(cell_link(&t, &t.rows[0], 0), Some(CellLink { text: "example.com/@a/video/1".into(), href: "https://www.example.com/@a/video/1".into() }));
        let t = read_table("#,Example,Link\n1,First,https://a.example.com\n2,Second,https://b.example.com", "v.csv");
        assert!(t.ranked);
        assert_eq!(cell_link(&t, &t.rows[0], 1).map(|l| l.href).as_deref(), Some("https://a.example.com"));
    }

    #[test]
    fn words_are_ordered_as_a_person_reads_them() {
        assert_eq!(collate("shot-2", "shot-10"), Ordering::Less);
        assert_eq!(collate("Apple", "apple"), Ordering::Equal);
        assert_eq!(collate("Élan", "elan"), Ordering::Equal);
        assert_eq!(collate("a b", "ab"), Ordering::Less);
        assert_eq!(collate("9", "a"), Ordering::Less);
        assert_eq!(collate("Zed", "alpha"), Ordering::Greater);
        assert!(amount_of("$12.4-13.2B") > 1.2e10 && amount_of("2026-10-01").is_nan());
        assert_eq!(amount_of("about 5k"), 5000.0);
        assert_eq!(amount_of("$7B+"), 7e9);
    }
}
