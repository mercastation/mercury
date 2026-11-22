//! The web app's rules are JavaScript, and its regular expressions and strings have their own
//! idea of a word, a space, a line and a length. These are those ideas, so a rule ported from
//! the TypeScript reads the same text the same way: a regex is written here exactly as it is
//! written there, and `Re::new` turns it into one that matches what JavaScript's would.
//!
//! What differs from JavaScript and is left: under `i`, `k` and `s` also match the Kelvin sign
//! and the long s.

/// What JavaScript's `\s` and `trim` call white space.
pub fn is_space(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}

const SPACE: &str = r"\t\n\x0B\x0C\r \x{A0}\x{1680}\x{2000}-\x{200A}\x{2028}\x{2029}\x{202F}\x{205F}\x{3000}\x{FEFF}";
const WORD: &str = "0-9A-Za-z_";
const DIGIT: &str = "0-9";
const LINE_END: &str = r"\n\r\x{2028}\x{2029}";
/// The line ends JavaScript knows besides `\n`. A text with none of them has the lines the
/// plain engine's own multi-line mode sees.
const OTHER_LINE_ENDS: [char; 3] = ['\r', '\u{2028}', '\u{2029}'];

/// How a translated regex says the things the two engines say differently.
#[derive(Clone, Copy, PartialEq)]
enum Dialect {
    /// The plain engine, which is fast and has an ASCII word boundary of its own, but cannot
    /// look around: `^` and `$` under `m` mean a line as it sees one, ended by `\n` alone.
    Plain,
    /// The engine that looks ahead and behind, where a word boundary and a line's ends under
    /// `m` are spelled out as looks, the second for every line end JavaScript knows.
    Looking,
}

/// A JavaScript regex source as one this engine reads the same way: `\s`, `\w`, `\d` and `\b`
/// in JavaScript's ASCII sense, `.` stopping at every line end, `^` and `$` at the ends of the
/// text (or of a line under `m`), and a `[` inside a class as the character it is there.
fn translate(source: &str, flags: &str, dialect: Dialect) -> String {
    let multi = flags.contains('m');
    let mut out = String::with_capacity(source.len() + 16);
    if flags.contains('i') {
        out.push_str("(?i)");
    }
    let chars: Vec<char> = source.chars().collect();
    let mut in_class = false;
    let mut k = 0;
    while k < chars.len() {
        let c = chars[k];
        k += 1;
        if c == '\\' {
            let n = chars[k];
            k += 1;
            let class = |set: &str, negated: bool, out: &mut String| {
                assert!(!(in_class && negated), "a negated class inside a class in {source}");
                if in_class {
                    out.push_str(set);
                } else {
                    out.push_str(if negated { "[^" } else { "[" });
                    out.push_str(set);
                    out.push(']');
                }
            };
            match n {
                's' => class(SPACE, false, &mut out),
                'S' => class(SPACE, true, &mut out),
                'w' => class(WORD, false, &mut out),
                'W' => class(WORD, true, &mut out),
                'd' => class(DIGIT, false, &mut out),
                'D' => class(DIGIT, true, &mut out),
                'b' if !in_class && dialect == Dialect::Plain => out.push_str(r"(?-u:\b)"),
                'b' if !in_class => out.push_str(&format!("(?:(?<=[{WORD}])(?![{WORD}])|(?<![{WORD}])(?=[{WORD}]))")),
                'u' => {
                    let hex: String = if chars[k] == '{' {
                        let end = k + chars[k..].iter().position(|c| *c == '}').expect("an unclosed \\u{");
                        let hex = chars[k + 1..end].iter().collect();
                        k = end + 1;
                        hex
                    } else {
                        k += 4;
                        chars[k - 4..k].iter().collect()
                    };
                    out.push_str(&format!(r"\x{{{hex}}}"));
                }
                '/' => out.push('/'),
                _ => {
                    out.push('\\');
                    out.push(n);
                }
            }
            continue;
        }
        if in_class {
            match c {
                ']' => {
                    in_class = false;
                    out.push(']');
                }
                '[' | '&' | '~' => {
                    out.push('\\');
                    out.push(c);
                }
                // Two hyphens in a row would subtract one class from another here.
                '-' if chars.get(k) == Some(&'-') => out.push_str(r"\-"),
                _ => out.push(c),
            }
            continue;
        }
        match c {
            '[' => {
                in_class = true;
                out.push('[');
                if chars.get(k) == Some(&'^') {
                    out.push('^');
                    k += 1;
                }
            }
            '.' if flags.contains('s') => out.push_str("(?s:.)"),
            '.' => out.push_str(&format!("[^{LINE_END}]")),
            '^' if multi && dialect == Dialect::Plain => out.push_str("(?m:^)"),
            '^' if multi => out.push_str(&format!(r"(?:\A|(?<=[{LINE_END}]))")),
            '^' => out.push_str(r"\A"),
            '$' if multi && dialect == Dialect::Plain => out.push_str("(?m:$)"),
            '$' if multi => out.push_str(&format!(r"(?:\z|(?=[{LINE_END}]))")),
            '$' => out.push_str(r"\z"),
            _ => out.push(c),
        }
    }
    out
}

/// Whether a JavaScript regex looks ahead or behind, or back at a group it already matched.
fn looks_around(source: &str) -> bool {
    let back = |w: &[u8]| w[0] == b'\\' && w[1].is_ascii_digit() && w[1] != b'0';
    ["(?=", "(?!", "(?<=", "(?<!"].iter().any(|look| source.contains(look)) || source.replace(r"\\", "").as_bytes().windows(2).any(back)
}

/// One match and its groups.
pub struct Captures<'t> {
    text: &'t str,
    spans: Vec<Option<(usize, usize)>>,
}

impl<'t> Captures<'t> {
    /// Group `n`, or None where JavaScript would hand over `undefined`. Group 0 is the match.
    pub fn get(&self, n: usize) -> Option<&'t str> {
        self.spans.get(n).copied().flatten().map(|(a, b)| &self.text[a..b])
    }
    fn whole(&self) -> (usize, usize) {
        self.spans[0].expect("a match has a whole")
    }
    /// Where group `n` sits in the text, in bytes: `m.index` for group 0.
    pub fn span(&self, n: usize) -> Option<(usize, usize)> {
        self.spans.get(n).copied().flatten()
    }
}

/// One of the web app's regular expressions. Most are matched by the plain engine. One that
/// looks around is matched by the slower engine that can; and one that only asks for lines
/// (`m`) is matched by the plain engine on a text whose every line ends in `\n`, which is
/// nearly every text, and by the slower one on the rest.
pub struct Re {
    plain: Option<regex::Regex>,
    plain_needs_newline_only: bool,
    looking: Option<fancy_regex::Regex>,
}

impl Re {
    /// `source` and `flags` as the TypeScript writes them between and after the slashes. `g` is
    /// not a property of the regex here: the caller picks `replace` or `replace_all`.
    pub fn new(source: &str, flags: &str) -> Re {
        let (looks, lines) = (looks_around(source), flags.contains('m'));
        let plain = (!looks).then(|| {
            let pattern = translate(source, flags, Dialect::Plain);
            regex::Regex::new(&pattern).unwrap_or_else(|e| panic!("the regex /{source}/{flags} does not compile as {pattern}: {e}"))
        });
        let looking = (looks || lines).then(|| {
            let pattern = translate(source, flags, Dialect::Looking);
            // A long answer may be scanned line by line; the default budget is for short inputs.
            let re = fancy_regex::RegexBuilder::new(&pattern).backtrack_limit(50_000_000).build();
            re.unwrap_or_else(|e| panic!("the regex /{source}/{flags} does not compile as {pattern}: {e}"))
        });
        Re { plain, plain_needs_newline_only: lines, looking }
    }

    /// The plain engine, where it reads `s` as JavaScript would.
    fn plain_for(&self, s: &str) -> Option<&regex::Regex> {
        self.plain.as_ref().filter(|_| !(self.plain_needs_newline_only && s.contains(OTHER_LINE_ENDS)))
    }

    /// The first match at or after byte `from`, with its groups.
    fn exec_from<'t>(&self, s: &'t str, from: usize) -> Option<Captures<'t>> {
        let spans = match self.plain_for(s) {
            Some(re) => re.captures_at(s, from).map(|c| c.iter().map(|g| g.map(|m| (m.start(), m.end()))).collect()),
            None => {
                let looking = self.looking.as_ref().expect("a regex has one engine or the other");
                looking.captures_from_pos(s, from).ok().flatten().map(|c| c.iter().map(|g| g.map(|m| (m.start(), m.end()))).collect())
            }
        };
        spans.map(|spans| Captures { text: s, spans })
    }

    pub fn test(&self, s: &str) -> bool {
        match self.plain_for(s) {
            Some(re) => re.is_match(s),
            None => self.looking.as_ref().is_some_and(|re| re.is_match(s).unwrap_or(false)),
        }
    }

    /// The first match and its groups, as `exec` returns them.
    pub fn exec<'t>(&self, s: &'t str) -> Option<Captures<'t>> {
        self.exec_from(s, 0)
    }

    /// Every match in order, as a `g` regex walks them. None of these regexes matches nothing,
    /// so each search starts where the last match ended.
    fn all<'t>(&self, s: &'t str) -> Vec<Captures<'t>> {
        let mut out = Vec::new();
        let mut from = 0;
        while let Some(caps) = self.exec_from(s, from) {
            let (start, end) = caps.whole();
            // A match of nothing would never move on; step over one character, as `g` does.
            from = if end > start { end } else { end + s[end..].chars().next().map_or(1, char::len_utf8) };
            out.push(caps);
            if from > s.len() {
                break;
            }
        }
        out
    }

    /// `s.matchAll(re)`: every match with its groups, in order.
    pub fn each<'t>(&self, s: &'t str) -> Vec<Captures<'t>> {
        self.all(s)
    }

    /// The byte ranges of every match, in order.
    pub fn matches(&self, s: &str) -> Vec<(usize, usize)> {
        self.all(s).iter().map(Captures::whole).collect()
    }

    /// `s.replace(re, with)` for a regex without `g`: the first match only.
    pub fn replace(&self, s: &str, with: impl Fn(&Captures) -> String) -> String {
        match self.exec(s) {
            Some(caps) => {
                let (start, end) = caps.whole();
                format!("{}{}{}", &s[..start], with(&caps), &s[end..])
            }
            None => s.to_string(),
        }
    }

    /// `s.replace(re, with)` for a regex with `g`: every match.
    pub fn replace_all(&self, s: &str, with: impl Fn(&Captures) -> String) -> String {
        let mut out = String::with_capacity(s.len());
        let mut last = 0;
        for caps in self.all(s) {
            let (start, end) = caps.whole();
            out.push_str(&s[last..start]);
            out.push_str(&with(&caps));
            last = end;
        }
        out.push_str(&s[last..]);
        out
    }

    /// `s.replace(re, '')`, first match only.
    pub fn strip(&self, s: &str) -> String {
        self.replace(s, |_| String::new())
    }

    /// `s.replace(re, '')` with `g`.
    pub fn strip_all(&self, s: &str) -> String {
        self.replace_all(s, |_| String::new())
    }

    /// `s.split(re)`, for a regex that never matches nothing.
    pub fn split<'t>(&self, s: &'t str) -> Vec<&'t str> {
        let mut out = Vec::new();
        let mut last = 0;
        for (start, end) in self.matches(s) {
            out.push(&s[last..start]);
            last = end;
        }
        out.push(&s[last..]);
        out
    }
}

/// A group of a match as JavaScript hands it over: `undefined` reads as no text.
pub fn group<'t>(caps: &Captures<'t>, n: usize) -> &'t str {
    caps.get(n).unwrap_or("")
}

/// Names one of the web app's regular expressions, compiled the first time it is asked.
macro_rules! re {
    ($name:ident, $source:expr, $flags:expr) => {
        static $name: std::sync::LazyLock<$crate::rules::js::Re> = std::sync::LazyLock::new(|| $crate::rules::js::Re::new($source, $flags));
    };
}
pub(crate) use re;

/// `s.trim()`.
pub fn trim(s: &str) -> &str {
    s.trim_matches(is_space)
}

/// `s.trimEnd()`.
pub fn trim_end(s: &str) -> &str {
    s.trim_end_matches(is_space)
}

/// `s.split(/\s+/).filter(Boolean)`.
pub fn words(s: &str) -> Vec<&str> {
    s.split(is_space).filter(|w| !w.is_empty()).collect()
}

/// `s.length`: JavaScript counts in UTF-16 units, so a character outside the basic plane is two.
pub fn len(s: &str) -> usize {
    s.chars().map(char::len_utf16).sum()
}

/// The byte where the UTF-16 unit `n` starts, or the end of `s`. A unit in the middle of a
/// pair counts as the start of its character: JavaScript would cut the character in half.
fn byte_at(s: &str, n: usize) -> usize {
    let mut units = 0;
    for (at, c) in s.char_indices() {
        if units + c.len_utf16() > n {
            return at;
        }
        units += c.len_utf16();
    }
    s.len()
}

/// `s.slice(start, end)`, both counted the way `len` counts.
pub fn slice(s: &str, start: usize, end: usize) -> &str {
    let (a, b) = (byte_at(s, start), byte_at(s, end));
    if a < b { &s[a..b] } else { "" }
}

/// `s.lastIndexOf(needle)` in UTF-16 units.
pub fn last_index_of(s: &str, needle: &str) -> Option<usize> {
    s.rfind(needle).map(|at| len(&s[..at]))
}

/// `s[0].toUpperCase() + s.slice(1)`. JavaScript upper-cases one UTF-16 unit, so a first
/// character outside the basic plane is left as it is.
pub fn upper_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.len_utf16() == 1 => c.to_uppercase().chain(chars).collect(),
        _ => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_word_ends_where_javascript_says_it_does() {
        // An accented letter is not a word character to JavaScript, so "it" ends before it.
        assert!(Re::new(r"\bit\b", "").test("say ité now"));
        assert!(!Re::new(r"\bit\b", "").test("say item now"));
        assert!(Re::new(r"^\w+$", "").test("slot_race9"));
        assert!(!Re::new(r"^\w+$", "").test("ação"));
    }

    #[test]
    fn the_ends_of_the_text_and_of_a_line() {
        assert!(!Re::new(r"a$", "").test("a\nb"));
        assert!(Re::new(r"a$", "m").test("a\nb"));
        assert!(Re::new(r"^b", "m").test("a\r\nb"));
        assert!(!Re::new(r"^b", "").test("a\nb"));
        // A dot stops at a carriage return too.
        assert_eq!(group(&Re::new(r"^(.*)", "").exec("one\r\ntwo").unwrap(), 1), "one");
    }

    #[test]
    fn a_class_reads_as_javascript_reads_it() {
        assert!(Re::new(r"[([]recommended", "i").test("so [Recommended"));
        assert!(Re::new(r"^[\s:\u2014\u2013-]*x$", "").test(" \u{2014}-: x"));
        assert!(Re::new(r"\p{Pd}x", "u").test("\u{2014}x"));
        assert!(Re::new(r"^\s$", "").test("\u{feff}"));
        assert!(!Re::new(r"^\s$", "").test("\u{85}"));
    }

    #[test]
    fn looking_ahead_behind_and_back() {
        assert!(Re::new(r"(?<!`)!\[", "").test("see ![shot](a.png)"));
        assert!(!Re::new(r"(?<!`)!\[", "").test("see `![shot](a.png)`"));
        assert!(Re::new(r"^(`?)x\1$", "").test("`x`"));
        assert!(!Re::new(r"^(`?)x\1$", "").test("`x"));
    }

    #[test]
    fn a_line_ends_at_every_line_end_javascript_knows() {
        // The same regex, on a text the plain engine reads and on one only the slower engine does.
        let heading = Re::new(r"^#{1,4}\s.*$", "gm");
        assert_eq!(heading.strip_all("one\n## two\nthree"), "one\n\nthree");
        assert_eq!(heading.strip_all("one\r\n## two\r\nthree"), "one\r\n\r\nthree");
        assert_eq!(heading.strip_all("one\u{2028}## two\u{2028}three"), "one\u{2028}\u{2028}three");
        assert!(looks_around(r"a(?=b)") && looks_around(r"(`?)x\1") && !looks_around(r"a\\1") && !looks_around(r"(?:a|b)\d"));
    }

    #[test]
    fn replacing_and_splitting() {
        let re = Re::new(r"\s*,\s*", "");
        assert_eq!(re.strip("a , b , c"), "ab , c");
        assert_eq!(re.strip_all("a , b , c"), "abc");
        assert_eq!(re.split("a , b , c"), ["a", "b", "c"]);
        assert_eq!(re.split("abc"), ["abc"]);
    }

    #[test]
    fn lengths_are_counted_in_utf16_units() {
        assert_eq!(len("a\u{1F600}b"), 4);
        assert_eq!(slice("a\u{1F600}b", 1, 3), "\u{1F600}");
        assert_eq!(slice("ação", 0, 2), "aç");
        assert_eq!(last_index_of("é. b. c", ". "), Some(4));
        assert_eq!(upper_first("école"), "École");
        assert_eq!(trim("\u{feff} a \u{a0}"), "a");
        assert_eq!(words(" a\u{a0}b\n c "), ["a", "b", "c"]);
    }
}
