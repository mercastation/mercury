//! Finding a thread by what it says: which threads a typed query keeps, in which order, and
//! the line each kept thread shows for it. This is `renderer/src/search.ts`, with the one
//! matching rule it shares with the engine's own search (`shared/word-match.mts`).
//!
//! Every thread is read, finished ones included, over its name, title, body, result, latest
//! checkpoint and the person's reply. Every typed word must appear, each at the start of a
//! word, and only the last may be a prefix, since it is the one still being typed. The typed
//! words in a row outrank the same words scattered, a title that has them outranks prose, and
//! nothing is dropped for ranking low.
//!
//! One person, with no team: a conversation with a teammate is found in the web window by its
//! people's names and by what was said in it (`renderer/src/threads/search.ts`), and neither
//! is read here.
//!
//! What differs from JavaScript and is left: a place in a text is counted in characters where
//! JavaScript counts UTF-16 units, so a sentence cut to its budget keeps a little more when
//! it holds characters outside the basic plane; and a capital whose small form is two
//! characters (the dotted İ) is matched as it is written.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use super::format::search_text;
use super::js::{self, re};
use super::kinds::is_cause;
use super::list::row_summary;
use crate::data::Item;

/// The room a row's line has (`SUMMARY_BUDGET` in `renderer/src/list-rules.ts`).
pub const SUMMARY_BUDGET: usize = 112;

/* ------------------------------- the one matching rule ------------------------------- */

/// A text as it is matched: small letters, and curly quotes and dashes as a keyboard types
/// them (`fold` in `shared/word-match.mts`). It keeps one character for each character, because
/// a place found in the folded text is then cut out of the text as written.
pub fn fold(text: &str) -> String {
    let low = text.to_lowercase();
    let low = if low.chars().count() == text.chars().count() {
        low
    } else {
        // A capital that becomes two characters would move every place after it.
        text.chars()
            .map(|c| {
                let mut small = c.to_lowercase();
                match (small.next(), small.next()) {
                    (Some(one), None) => one,
                    _ => c,
                }
            })
            .collect()
    };
    low.chars()
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' | '\u{2bc}' => '\'',
            '\u{201c}' | '\u{201d}' => '"',
            '\u{2013}' | '\u{2014}' => '-',
            other => other,
        })
        .collect()
}

/// A text with its camelCase names also written split ("toKebab" adds "to Kebab") and its
/// hyphenated words joined ("sign-up" adds "signup"), for a haystack that is matched and never
/// cut (`withHumps`). Read before folding, while a capital still says where a word starts.
pub fn with_humps(raw: &str) -> String {
    let letters: Vec<char> = raw.chars().collect();
    let (mut split, mut joined) = (String::with_capacity(raw.len() + 8), String::with_capacity(raw.len()));
    for (at, c) in letters.iter().enumerate() {
        let before = at.checked_sub(1).map(|b| letters[b]);
        if c.is_ascii_uppercase() && before.is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit()) {
            split.push(' ');
        }
        split.push(*c);
        let between = before.is_some_and(|b| b.is_ascii_alphanumeric()) && letters.get(at + 1).is_some_and(char::is_ascii_alphanumeric);
        if !(*c == '-' && between) {
            joined.push(*c);
        }
    }
    let mut out = raw.to_string();
    for extra in [split, joined] {
        if extra != raw {
            out.push_str(" . ");
            out.push_str(&extra);
        }
    }
    out
}

/// What counts as part of a word once a text is folded (`isWordChar`). An accented letter does
/// not, as in the web window.
fn is_word_char(c: u8) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit()
}

/// Where `needle` starts a word in the folded `hay`, at or after byte `from`; `whole` also
/// wants the word to end there (`wordStart`). A byte of a longer character is never a word
/// character, which is what the rule says of the character itself.
pub fn word_start(hay: &str, needle: &str, from: usize, whole: bool) -> Option<usize> {
    if needle.is_empty() || from > hay.len() {
        return None;
    }
    let bytes = hay.as_bytes();
    let mut at = from + hay[from..].find(needle)?;
    loop {
        let starts_word = at == 0 || !is_word_char(bytes[at - 1]);
        let ends_word = !whole || !bytes.get(at + needle.len()).is_some_and(|c| is_word_char(*c));
        if starts_word && ends_word {
            return Some(at);
        }
        // On from the next character, as `indexOf(needle, at + 1)` goes on.
        let next = at + hay[at..].chars().next().map_or(1, char::len_utf8);
        at = next + hay.get(next..)?.find(needle)?;
    }
}

/// The typed words. They decide what is kept; the phrase decides what comes first and what
/// lights as one run (`Query` in `search.ts`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query {
    pub terms: Vec<String>,
    /// The typed words as one string when there are several, empty for one word.
    pub phrase: String,
    /// The query was wrapped in double quotes, so only the phrase will do.
    pub quoted: bool,
}

// A thread's id, alone or after the app's own name as a branch has it (`ticketQuery` in
// `renderer/src/ticket-references.ts`; the name is `nameSlug` in `shared/product-name.mjs`).
re!(TICKET, r"^(?:mercury\/)?(w-[a-f0-9]{1,10})(-[a-z0-9]+(?:-[a-z0-9]+)*)?$", "i");

/// The id a query names, or the start of one. Words after the id count only once it is whole.
fn ticket_query(value: &str) -> Option<String> {
    let caps = TICKET.exec(js::trim(value))?;
    let id = caps.get(1)?;
    if caps.get(2).is_some_and(|rest| !rest.is_empty()) && id.len() != 12 {
        return None;
    }
    Some(id.to_lowercase())
}

/// Reads what was typed (`parseQuery`).
pub fn parse_query(query: &str) -> Query {
    let folded = fold(query);
    let folded = js::trim(&folded);
    let quoted = folded.chars().count() > 2 && folded.starts_with('"') && folded.ends_with('"');
    let inner = js::trim(if quoted { &folded[1..folded.len() - 1] } else { folded });
    let asked = ticket_query(inner).unwrap_or_else(|| inner.to_string());
    let terms: Vec<String> = js::words(&asked.replace('"', " ")).into_iter().map(str::to_string).collect();
    let phrase = if terms.len() > 1 { terms.join(" ") } else { String::new() };
    let quoted = quoted && !terms.is_empty();
    Query { terms, phrase, quoted }
}

/// Only the last term may be a prefix. Earlier terms must be whole words, so "to" in "how to
/// work" does not light "tonight" (`termAt`).
fn term_at(hay: &str, terms: &[String], i: usize, from: usize) -> Option<usize> {
    word_start(hay, &terms[i], from, i + 1 < terms.len())
}

fn has_every(hay: &str, terms: &[String]) -> bool {
    (0..terms.len()).all(|i| term_at(hay, terms, i, 0).is_some())
}

/* --------------------------------- what is searched --------------------------------- */

/// Every run of white space as one space, so a phrase is found across a line that wraps.
fn squash(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_space = false;
    for c in text.chars() {
        if js::is_space(c) {
            if !in_space {
                out.push(' ');
            }
            in_space = true;
        } else {
            out.push(c);
            in_space = false;
        }
    }
    out
}

/// One sentence as written, for printing, and folded flat, for matching.
#[derive(Clone, Debug)]
struct Line {
    raw: String,
    folded: String,
}

/// Prose cut where a sentence ends: after a full stop, a question mark or an exclamation mark
/// with white space behind it (`splitSentences`).
fn split_lines(text: &str) -> Vec<Line> {
    let letters: Vec<(usize, char)> = text.char_indices().collect();
    let mut cuts = Vec::new();
    let (mut k, mut from) = (0, 0);
    while k < letters.len() {
        let ends_a_sentence = k > 0 && matches!(letters[k - 1].1, '.' | '!' | '?') && js::is_space(letters[k].1);
        if !ends_a_sentence {
            k += 1;
            continue;
        }
        cuts.push(&text[from..letters[k].0]);
        while k < letters.len() && js::is_space(letters[k].1) {
            k += 1;
        }
        from = letters.get(k).map_or(text.len(), |l| l.0);
    }
    cuts.push(&text[from..]);
    cuts.into_iter().map(js::trim).filter(|s| !s.is_empty()).map(|raw| Line { raw: raw.to_string(), folded: squash(&fold(raw)) }).collect()
}

/// What one thread is searched over (`Hay`). The title is kept apart because a title hit ranks
/// higher. `all` is only tested, never cut, so it can carry the split and joined spellings.
/// The project's name is left out: it would match every thread of that project.
#[derive(Clone, Debug)]
pub struct Hay {
    title: String,
    all: String,
    body: Vec<Line>,
    result: Vec<Line>,
    note: Vec<Line>,
    answer: Vec<Line>,
}

fn haystack(i: &Item) -> Hay {
    // The list draws the name with the title under it, so both are searched. Fields are
    // joined with " . " so no phrase is found straddling the seam between two of them.
    let name = [i.label.as_deref(), i.title.as_deref()].into_iter().flatten().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" . ");
    let title = squash(&fold(&with_humps(&name)));
    // A cause row's body is its lead's brief, not something anyone said.
    let body = if is_cause(i) { String::new() } else { search_text(i.body.as_deref().unwrap_or("")) };
    let result = search_text(i.result.as_deref().unwrap_or(""));
    let note = search_text(i.text("note").unwrap_or(""));
    let answer = search_text(i.text("answer").unwrap_or(""));
    let prose = [&body, &result, &note, &answer].into_iter().filter(|s| !s.is_empty()).map(String::as_str).collect::<Vec<_>>().join(" . ");
    let all = format!("{} . {} . {}", fold(&i.id), title, squash(&fold(&with_humps(&prose))));
    Hay { title, all, body: split_lines(&body), result: split_lines(&result), note: split_lines(&note), answer: split_lines(&answer) }
}

/// What every thread is searched over, kept from one keystroke to the next: taking the
/// markdown out and cutting the prose into sentences is the slow part, and a thread's words do
/// not change while the person types (the `HAY` cache in `search.ts`, which keeps it for as
/// long as the page holds the thread).
#[derive(Default)]
pub struct Index {
    kept: HashMap<String, (u64, Hay)>,
}

/// What tells one state of a thread's words from the next without reading them again: when it
/// last changed and how long each searched field is.
fn stamp(i: &Item) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    i.product.hash(&mut h);
    i.updated_at.unwrap_or(0.0).to_bits().hash(&mut h);
    i.kind.hash(&mut h);
    for field in [i.label.as_deref(), i.title.as_deref(), i.body.as_deref(), i.result.as_deref(), i.text("note"), i.text("answer")] {
        field.map(str::len).hash(&mut h);
    }
    h.finish()
}

impl Index {
    /// Reads the threads that changed since they were last read, and lets go of the ones gone.
    pub fn read(&mut self, items: &[Item]) {
        let here: HashSet<&str> = items.iter().map(|i| i.id.as_str()).collect();
        self.kept.retain(|id, _| here.contains(id.as_str()));
        for i in items {
            let now = stamp(i);
            if self.kept.get(&i.id).is_none_or(|(was, _)| *was != now) {
                self.kept.insert(i.id.clone(), (now, haystack(i)));
            }
        }
    }

    /// A thread's haystack: the one kept when it still stands, else read now.
    fn of(&self, i: &Item) -> Cow<'_, Hay> {
        match self.kept.get(&i.id) {
            Some((was, hay)) if *was == stamp(i) => Cow::Borrowed(hay),
            _ => Cow::Owned(haystack(i)),
        }
    }
}

/* ------------------------------------ the search ------------------------------------ */

/// Higher sorts first. At 2 and above the thread says what was typed; at 1 it only contains
/// the words (`score`).
///
///   6  the thread's id, exactly
///   5  an id that starts with the query
///   4  the phrase, in the title
///   3  the phrase, anywhere else
///   2  every word, all of them in the title
///   1  every word, somewhere
///   0  not a match
fn score(i: &Item, hay: &Hay, q: &Query) -> u8 {
    if q.terms.is_empty() {
        return 0;
    }
    let id = fold(&i.id);
    if let Some(asked) = (q.terms.len() == 1).then(|| ticket_query(&q.terms[0])).flatten().filter(|asked| id.starts_with(asked.as_str())) {
        return if id == asked { 6 } else { 5 };
    }
    if !q.phrase.is_empty() {
        if word_start(&hay.title, &q.phrase, 0, false).is_some() {
            return 4;
        }
        if word_start(&hay.all, &q.phrase, 0, false).is_some() {
            return 3;
        }
        // The query asked for the phrase and only that.
        if q.quoted {
            return 0;
        }
    }
    if has_every(&hay.title, &q.terms) {
        return 2;
    }
    if has_every(&hay.all, &q.terms) {
        return 1;
    }
    0
}

/// A thread a query kept, and the sentence the words were found in, ready for its row.
#[derive(Clone, Debug)]
pub struct Hit<'a> {
    pub item: &'a Item,
    pub summary: String,
}

/// The threads a query keeps: best first, then newest. With nothing typed, every thread,
/// newest first (`searchItems`).
pub fn search_items<'a>(index: &Index, items: &'a [Item], query: &str) -> Vec<Hit<'a>> {
    let q = parse_query(query);
    let newest_first = |a: &&Item, b: &&Item| b.updated_at.unwrap_or(0.0).total_cmp(&a.updated_at.unwrap_or(0.0));
    if q.terms.is_empty() {
        let mut all: Vec<&Item> = items.iter().collect();
        all.sort_by(newest_first);
        return all.into_iter().map(|item| Hit { item, summary: summary(item, &index.of(item), &[], "") }).collect();
    }
    let mut kept: Vec<(u8, &Item)> = items.iter().map(|item| (score(item, &index.of(item), &q), item)).filter(|(n, _)| *n > 0).collect();
    kept.sort_by(|a, b| b.0.cmp(&a.0).then(newest_first(&a.1, &b.1)));
    kept.into_iter().map(|(_, item)| Hit { item, summary: summary(item, &index.of(item), &q.terms, &q.phrase) }).collect()
}

/// The sentence the words were found in (`hitSummary`). On a finished thread the result is
/// asked first, as its row does at rest. The checkpoint comes after body and result, and the
/// person's reply last. The phrase is asked of every field before any single word is asked of
/// one. A title-only hit keeps the ordinary line, since the title already shows the match.
fn summary(i: &Item, hay: &Hay, terms: &[String], phrase: &str) -> String {
    if let Some(asked) = (terms.len() == 1).then(|| ticket_query(&terms[0])).flatten() {
        if fold(&i.id).starts_with(asked.as_str()) {
            return i.id.clone();
        }
    }
    let finished = i.status == "done";
    let ordered = if finished { [&hay.result, &hay.body, &hay.note] } else { [&hay.body, &hay.result, &hay.note] };
    let passes: &[bool] = if phrase.is_empty() { &[false] } else { &[true, false] };
    for phrase_only in passes {
        for sentences in ordered.into_iter().chain([&hay.answer]) {
            let sentence = pick_sentence(sentences, terms, phrase, SUMMARY_BUDGET, *phrase_only);
            if !sentence.is_empty() {
                return sentence;
            }
        }
    }
    // The row the app makes itself is not a thread and is never searched, so no first line of
    // its own is handed over.
    row_summary(i, None)
}

/// The first sentence with the phrase, else the first with a word, cut around the match
/// rather than from the left, so the match still shows at the end of a long sentence.
/// `phrase_only` lets the summary look for the phrase in every field before any field answers
/// with a single word (`pickSentence`).
fn pick_sentence(lines: &[Line], terms: &[String], phrase: &str, budget: usize, phrase_only: bool) -> String {
    // With nothing typed, skip walking every sentence of every thread.
    if phrase.is_empty() && terms.is_empty() {
        return String::new();
    }
    if !phrase.is_empty() {
        // Matched on the folded sentence so a wrapped phrase is found. The cut is placed from
        // a single word, whose place carries to the sentence as written.
        if let Some(line) = lines.iter().find(|line| word_start(&line.folded, phrase, 0, false).is_some()) {
            return window_around(&line.raw, first_hit(line, terms).unwrap_or(0), budget);
        }
    }
    if phrase_only {
        return String::new();
    }
    lines.iter().find_map(|line| first_hit(line, terms).map(|hit| window_around(&line.raw, hit, budget))).unwrap_or_default()
}

/// Where the cut is placed, in characters: at the longest term, not the leftmost, which is
/// usually a short word like "to". Each term keeps the whole-word rule its place in the query
/// gives it (`firstHit`).
fn first_hit(line: &Line, terms: &[String]) -> Option<usize> {
    let mut order: Vec<usize> = (0..terms.len()).collect();
    order.sort_by(|a, b| terms[*b].chars().count().cmp(&terms[*a].chars().count()));
    // A place in the folded sentence is the same place in the written one up to the first run
    // of two or more spaces, which folding flat shortens.
    order.into_iter().find_map(|i| term_at(&line.folded, terms, i, 0)).map(|byte| line.folded[..byte].chars().count())
}

/// A budget-sized piece with the match inside it, cut on spaces and marked with an ellipsis on
/// whichever side was cut (`windowAround`). `hit` and `budget` count characters.
fn window_around(sentence: &str, hit: usize, budget: usize) -> String {
    let letters: Vec<char> = sentence.chars().collect();
    if letters.len() <= budget {
        return sentence.to_string();
    }
    // Enough of a run-up that the match reads in context.
    let lead = 24;
    let mut start = hit.saturating_sub(lead);
    if start > 0 {
        if let Some(space) = (start..letters.len()).find(|at| letters[*at] == ' ').filter(|space| *space < hit) {
            start = space + 1;
        }
    }
    let mut end = letters.len().min(start + budget);
    if end < letters.len() {
        if let Some(space) = (0..=end).rev().find(|at| letters[*at] == ' ').filter(|space| *space > hit) {
            end = space;
        }
    }
    let piece: String = letters[start.min(end)..end].iter().collect();
    format!("{}{}{}", if start > 0 { "…" } else { "" }, js::trim(&piece), if end < letters.len() { "…" } else { "" })
}

/* ----------------------------- the match, as a row draws it ----------------------------- */

/// A piece of a line, and whether it is what was typed.
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub text: String,
    pub hit: bool,
}

/// A line cut into the pieces that match and the pieces between, so a row can draw the match
/// without reading the query again. Nothing matching gives one unmarked piece (`splitHits`).
pub fn split_hits(text: &str, terms: &[String], phrase: &str) -> Vec<Run> {
    if text.is_empty() {
        return Vec::new();
    }
    let whole = || vec![Run { text: text.to_string(), hit: false }];
    let wanted: Vec<String> = terms.iter().map(|t| fold(t)).filter(|t| !t.is_empty()).collect();
    if wanted.is_empty() {
        return whole();
    }
    let lower = fold(text);
    // Every place a term or the phrase stands, merged, so "how to work" lights as one run and
    // terms that overlap do not nest.
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut push = |needle: &str, find: &dyn Fn(usize) -> Option<usize>| {
        let mut at = find(0);
        while let Some(found) = at {
            spans.push((found, found + needle.len()));
            at = find(found + lower[found..].chars().next().map_or(1, char::len_utf8));
        }
    };
    for (i, term) in wanted.iter().enumerate() {
        push(term, &|from| term_at(&lower, &wanted, i, from));
    }
    if !phrase.is_empty() {
        push(phrase, &|from| word_start(&lower, phrase, from, false));
    }
    if spans.is_empty() {
        return whole();
    }
    spans.sort();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in spans {
        match merged.last_mut() {
            Some(last) if start <= last.1 => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }
    // A place in the folded line is a count of characters into the line as written.
    let written: Vec<usize> = text.char_indices().map(|(at, _)| at).chain([text.len()]).collect();
    let place = |byte: usize| written[lower[..byte].chars().count()];
    let mut runs = Vec::new();
    let mut at = 0;
    for (start, end) in merged {
        let (start, end) = (place(start), place(end));
        if start > at {
            runs.push(Run { text: text[at..start].to_string(), hit: false });
        }
        runs.push(Run { text: text[start..end].to_string(), hit: true });
        at = end;
    }
    if at < text.len() {
        runs.push(Run { text: text[at..].to_string(), hit: false });
    }
    runs
}

/// A found line with the typed words marked (`markedWords` in `renderer/src/found-inside.ts`).
pub fn marked_words(text: &str, query: &str) -> Vec<Run> {
    let q = parse_query(query);
    split_hits(text, &q.terms, &q.phrase)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn item(v: Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    fn terms(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| w.to_string()).collect()
    }

    // What `search.ts` hands its own tests, each asked of one thread read on the spot:
    // `searchScore`, `matches`, `hitSummary` and `matchedSentence`.
    fn search_score(i: &Item, query: &str) -> u8 {
        score(i, &haystack(i), &parse_query(query))
    }
    fn matches(i: &Item, terms: &[String]) -> bool {
        let wanted: Vec<String> = terms.iter().map(|t| fold(t)).collect();
        !wanted.is_empty() && has_every(&haystack(i).all, &wanted)
    }
    fn hit_summary(i: &Item, terms: &[String], phrase: &str) -> String {
        summary(i, &haystack(i), terms, phrase)
    }
    fn matched_sentence(text: &str, terms: &[String], phrase: &str) -> String {
        pick_sentence(&split_lines(text), terms, phrase, SUMMARY_BUDGET, false)
    }

    /// The ids a query keeps, in order.
    fn found(items: &[Item], query: &str) -> Vec<String> {
        let mut index = Index::default();
        index.read(items);
        search_items(&index, items, query).into_iter().map(|h| h.item.id.clone()).collect()
    }

    fn hits(runs: &[Run]) -> Vec<&str> {
        runs.iter().filter(|r| r.hit).map(|r| r.text.as_str()).collect()
    }

    /// A word in the title, the same word only in the body, only in a finished thread's
    /// result, and a thread that has nothing to do with it.
    fn store() -> Vec<Item> {
        vec![
            item(json!({ "id": "a", "title": "The refund rate is half a percent", "body": "The number came off the ledger this morning.", "status": "open", "updatedAt": 300 })),
            item(json!({ "id": "b", "title": "Not counting the five: they are all ours",
                "body": "They were all our own orders. The March refund report now opens with a box that says so.", "status": "open", "updatedAt": 200 })),
            item(json!({ "id": "c", "title": "Sales walk: nothing filed", "body": "Walked the funnel.",
                "result": "The one real leak, the refund rate climbing after the first order, is the third option.", "status": "done", "updatedAt": 100 })),
            item(json!({ "id": "d", "title": "Pick a dark palette", "body": "Five of them, drawn in the real app.", "status": "open", "updatedAt": 400 })),
        ]
    }

    #[test]
    fn a_query_is_its_words_in_small_letters() {
        assert_eq!(parse_query("  Refund   Rate ").terms, ["refund", "rate"]);
        assert_eq!(parse_query("  Refund   Rate ").phrase, "refund rate");
        assert_eq!(parse_query("refund").phrase, "");
        assert!(parse_query("   ").terms.is_empty());
        // In double quotes only the phrase will do, and curly quotes are quotes too.
        assert!(parse_query("\"refund rate\"").quoted && parse_query("\u{201c}refund rate\u{201d}").quoted);
        assert!(!parse_query("\"\"").quoted);
        // A thread's id is asked for by itself, with or without the branch's name before it.
        assert_eq!(parse_query("mercury/w-0123abcdef-fix-the-export").terms, ["w-0123abcdef"]);
        assert_eq!(parse_query("W-01").terms, ["w-01"]);
    }

    #[test]
    fn the_title_the_body_and_a_finished_threads_result_are_all_read() {
        let all = store();
        assert!(matches(&all[0], &terms(&["refund"])));
        assert!(matches(&all[1], &terms(&["refund"])));
        assert!(matches(&all[2], &terms(&["refund"])) && all[2].status == "done");
        assert!(matches(&all[1], &terms(&["MARCH"])));
        // Through the markdown, so a word in bold still matches.
        assert!(matches(&item(json!({ "id": "x", "body": "The **refund** rate." })), &terms(&["refund"])));
        // Every word, not any: a second word narrows.
        assert!(matches(&all[0], &terms(&["refund", "percent"])));
        assert!(!matches(&all[0], &terms(&["refund", "kangaroo"])));
        assert!(!matches(&all[0], &[]));
    }

    #[test]
    fn every_project_and_every_status_is_kept_newest_first() {
        let all = store();
        assert_eq!(found(&all, "refund"), ["a", "b", "c"]);
        assert_eq!(found(&all, ""), ["d", "a", "b", "c"]);
        assert_eq!(found(&all, "refund march"), ["b"]);
        // With nothing typed a thread says its ordinary line, not a sentence it was found by.
        let mut index = Index::default();
        index.read(&all);
        let untyped = search_items(&index, &all, "");
        assert_eq!(untyped.iter().find(|h| h.item.id == "a").unwrap().summary, "The number came off the ledger this morning.");
    }

    #[test]
    fn a_kept_thread_says_the_sentence_the_word_was_found_in() {
        let all = store();
        assert_eq!(hit_summary(&all[1], &terms(&["refund"]), ""), "The March refund report now opens with a box that says so.");
        // A finished thread's result is asked first, as its row does at rest.
        assert!(hit_summary(&all[2], &terms(&["refund"]), "").contains("the refund rate climbing after the first order"));
        // A hit in the title alone keeps the ordinary line: the title already shows the match.
        let titled = item(json!({ "id": "e", "title": "Kangaroo", "body": "Nothing about that here.", "status": "open" }));
        assert_eq!(hit_summary(&titled, &terms(&["kangaroo"]), ""), "Nothing about that here.");
        // An id asked for is answered with the id.
        let by_id = item(json!({ "id": "w-0123abcdef", "title": "Fix the export", "body": "It drops a column.", "status": "open" }));
        assert_eq!(hit_summary(&by_id, &terms(&["w-0123"]), ""), "w-0123abcdef");
    }

    #[test]
    fn the_sentence_with_the_word_is_picked_and_cut_around_it() {
        assert_eq!(matched_sentence("One. Two. The word is refund. Four.", &terms(&["refund"]), ""), "The word is refund.");
        assert_eq!(matched_sentence("Nothing here.", &terms(&["refund"]), ""), "");
        // Cut from the left, the very word that kept the thread would be the part cut off.
        let long = format!("{}refund came at the end.", "padding words here ".repeat(12));
        let out = matched_sentence(&long, &terms(&["refund"]), "");
        assert!(out.to_lowercase().contains("refund") && out.starts_with('…'));
        assert!(out.chars().count() <= 120);
        // The sentence with the whole phrase, not the first with a stray word of it.
        let sentence = matched_sentence("To be clear, this is about something else. The How to work section is long.", &terms(&["how", "to", "work"]), "how to work");
        assert!(sentence.contains("How to work section"));
    }

    #[test]
    fn a_line_is_cut_into_what_matches_and_what_does_not() {
        let run = |text: &str, hit: bool| Run { text: text.to_string(), hit };
        assert_eq!(split_hits("the refund rate", &terms(&["refund"]), ""), [run("the ", false), run("refund", true), run(" rate", false)]);
        assert_eq!(hits(&split_hits("refund and refund", &terms(&["refund"]), "")).len(), 2);
        assert_eq!(split_hits("Refund", &terms(&["refund"]), ""), [run("Refund", true)]);
        // Two terms where one holds the other light as one run.
        assert_eq!(split_hits("refund", &terms(&["ref", "refund"]), ""), [run("refund", true)]);
        assert_eq!(split_hits("nothing here", &terms(&["refund"]), ""), [run("nothing here", false)]);
        // No character is lost: a row prints these runs and nothing else.
        let text = "A refund, then another Refund, then none.";
        assert_eq!(split_hits(text, &terms(&["refund", "none"]), "").iter().map(|r| r.text.as_str()).collect::<String>(), text);
        // A letter that takes more room written than folded keeps its place.
        assert_eq!(split_hits("\u{201c}Açaí\u{201d} refund", &terms(&["refund"]), ""), [run("\u{201c}Açaí\u{201d} ", false), run("refund", true)]);
        assert_eq!(marked_words("Onboarding checklist copy", "check"), [run("Onboarding ", false), run("check", true), run("list copy", false)]);
    }

    /// One that says the phrase in its title, one in its body, one with the three words
    /// scattered through prose, and one that only has them inside other words.
    fn phrases() -> Vec<Item> {
        vec![
            item(json!({ "id": "title", "status": "done", "updatedAt": 100, "title": "How to Work is merged and the app already has it", "result": "The app started at one today, after all of it." })),
            item(json!({ "id": "body", "status": "open", "updatedAt": 200, "title": "Measure the brief handed over this session",
                "body": "The How to work section is twenty thousand characters, measured again from scratch." })),
            item(json!({ "id": "scattered", "status": "open", "updatedAt": 300, "title": "Right now we only have one special theme",
                "body": "How many variants should there be, and do we have to ship them, and does the work land this week?" })),
            item(json!({ "id": "inside", "status": "open", "updatedAt": 400, "title": "After my last customer call I concluded something",
                "body": "I ran into it again and then I stopped. Showhow the paperwork lands." })),
        ]
    }

    #[test]
    fn the_typed_phrase_outranks_its_words_and_drops_nothing() {
        let all = phrases();
        let order = found(&all, "how to work");
        assert_eq!(order[0], "title");
        assert_eq!(order[1], "body");
        assert!(order.contains(&"scattered".to_string()));
        // In quotes, only the threads that say it.
        let mut quoted = found(&all, "\"how to work\"");
        quoted.sort();
        assert_eq!(quoted, ["body", "title"]);
        // One word has no phrase, and is ordered newest first as ever.
        assert_eq!(found(&all, "theme"), ["scattered"]);
        assert_eq!(found(&all, ""), ["inside", "scattered", "body", "title"]);
        // Words in the title are not the phrase: such a thread ranks under every one that says it.
        let loose = item(json!({ "id": "loose", "status": "open", "updatedAt": 900, "title": "I want to kick off a film, a small demo of how it works", "body": "Nothing else here." }));
        assert_eq!(search_score(&loose, "how to work"), 2);
        assert!(search_score(&all[0], "how to work") > 2 && search_score(&all[1], "how to work") > 2);
        assert_eq!(search_score(&loose, "film"), 2);
        assert_eq!(search_score(&loose, "nothing"), 1);
        // A line break inside the phrase does not hide it: prose is markdown, and markdown wraps.
        let wrapped = item(json!({ "id": "wrapped", "status": "open", "updatedAt": 500, "title": "The brief", "body": "Measure the How to\nwork section and say what you find." }));
        assert!(search_score(&wrapped, "how to work") >= 3);
        assert_eq!(found(&[wrapped], "\"how to work\""), ["wrapped"]);
    }

    #[test]
    fn a_word_is_found_at_the_start_of_a_word() {
        let all = phrases();
        let (body, inside) = (&all[1], &all[3]);
        // Not "to" inside "into", "customer" or "stopped", nor "how" inside "showhow".
        assert!(!matches(inside, &terms(&["to"])));
        assert!(!matches(inside, &terms(&["how"])));
        assert!(found(&[inside.clone()], "work").is_empty());
        // A word not finished yet is still found.
        assert!(matches(body, &terms(&["meas"])) && matches(body, &terms(&["charact"])));
        // Only the last word may be half typed: every word before it is a whole word.
        let tonight = item(json!({ "id": "t", "status": "open", "updatedAt": 1, "title": "Your disk tonight", "body": "A worker never ends its turn with a build still running." }));
        assert!(!split_hits("Your disk tonight", &terms(&["to", "work"]), "").iter().any(|r| r.hit));
        assert_eq!(hits(&split_hits("A worker never ends", &terms(&["to", "work"]), "")), ["work"]);
        assert!(!matches(&tonight, &terms(&["to", "work"])));
        assert!(matches(body, &terms(&["section", "charact"])));
        assert!(!matches(body, &terms(&["charact", "section"])));
        // A bracket, a slash and a dash are not letters, so a word starts after one.
        let marks = item(json!({ "id": "p", "status": "open", "updatedAt": 1, "title": "(idle-time) and main/store", "body": "" }));
        assert!(matches(&marks, &terms(&["time"])) && matches(&marks, &terms(&["store"])));
        assert!(!split_hits("I ran into it and then I stopped", &terms(&["to"]), "").iter().any(|r| r.hit));
        assert_eq!(hits(&split_hits("the refund rate", &terms(&["refu"]), "")), ["refu"]);
        assert_eq!(hits(&split_hits("The How to work section", &terms(&["how", "to", "work"]), "how to work")), ["How to work"]);
    }

    #[test]
    fn an_apostrophe_is_an_apostrophe_and_a_heading_is_text() {
        let curly = item(json!({ "id": "c", "status": "open", "updatedAt": 1, "title": "There\u{2019}s something wrong with the search box", "body": "" }));
        assert_eq!(found(&[curly], "there's"), ["c"]);
        let straight = item(json!({ "id": "s", "status": "open", "updatedAt": 1, "title": "There's something wrong", "body": "" }));
        assert_eq!(found(&[straight], "there\u{2019}s"), ["s"]);
        let quoted = item(json!({ "id": "q", "status": "open", "updatedAt": 1, "title": "They said \u{201c}how to work\u{201d} again", "body": "" }));
        assert_eq!(search_score(&quoted, "how to work"), 4);
        // A heading is a landmark, and a landmark is what a person half remembers and types.
        let headed = item(json!({ "id": "h", "status": "open", "updatedAt": 1, "title": "Fix the two approved defects",
            "body": "Do not touch the other lanes.\n\n## Where the code is and how to work\n\nThe app is elsewhere." }));
        assert!(search_score(&headed, "how to work") >= 3);
        let mut index = Index::default();
        index.read(std::slice::from_ref(&headed));
        assert!(search_items(&index, std::slice::from_ref(&headed), "how to work")[0].summary.contains("how to work"));
    }

    #[test]
    fn a_word_inside_a_camelcase_name_finds_its_thread() {
        let kebab = item(json!({ "id": "w-1", "title": "toKebab function and tests", "body": "Write toKebab(str) in index.js.", "updatedAt": 1 }));
        let into = item(json!({ "id": "w-2", "title": "Fold the list into one", "body": "Nothing else.", "updatedAt": 2 }));
        assert_eq!(found(&[kebab.clone(), into.clone()], "kebab"), ["w-1"]);
        // A hump counts as a title hit, as a plain word would.
        assert_eq!(search_score(&kebab, "kebab"), 2);
        assert_eq!(search_score(&kebab, "kebab function"), 4);
        assert_eq!(search_score(&kebab, "tokebab"), 2);
        assert_eq!(search_score(&kebab, "to"), 2);
        assert_eq!(search_score(&item(json!({ "id": "w-4", "title": "Helpers", "body": "Add clampNum to index.js.", "updatedAt": 4 })), "num"), 1);
        // A small letter in the middle of a word still starts nothing.
        assert_eq!(search_score(&into, "to"), 0);
        assert_eq!(search_score(&kebab, "ebab"), 0);
        assert_eq!(with_humps("sign-up toKebab"), "sign-up toKebab . sign-up to Kebab . signup toKebab");
        assert_eq!(with_humps("plain words"), "plain words");
    }

    #[test]
    fn a_thread_is_found_by_its_id_before_anything_that_only_mentions_it() {
        let named = item(json!({ "id": "w-0123abcdef", "title": "Fix the export", "body": "It drops a column.", "status": "open", "updatedAt": 1 }));
        let mentions = item(json!({ "id": "w-fedcba9876", "title": "Waits on w-0123abcdef", "body": "", "status": "open", "updatedAt": 2 }));
        assert_eq!(search_score(&named, "w-0123abcdef"), 6);
        assert_eq!(search_score(&named, "w-0123"), 5);
        assert_eq!(found(&[mentions, named], "w-0123abcdef"), ["w-0123abcdef", "w-fedcba9876"]);
    }

    #[test]
    fn what_was_read_is_kept_until_the_thread_changes() {
        let mut all = store();
        let mut index = Index::default();
        index.read(&all);
        assert_eq!(index.kept.len(), 4);
        // New words on a thread are read at the next keystroke, and a thread gone is let go.
        all[3].body = Some("A refund palette, of all things.".to_string());
        all[3].updated_at = Some(500.0);
        all.remove(2);
        assert_eq!(found(&all, "refund"), ["a", "d", "b"]);
        index.read(&all);
        assert_eq!(index.kept.len(), 3);
        // A thread the index has not read yet is read on the spot rather than missed.
        let late = item(json!({ "id": "z", "title": "A late refund", "status": "open", "updatedAt": 600 }));
        all.push(late);
        assert_eq!(search_items(&index, &all, "refund")[0].item.id, "z");
    }
}
