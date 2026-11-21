//! The web app's rules are JavaScript, and its regular expressions and strings have their own
//! idea of a word, a space, a line and a length. These are those ideas, so a rule ported from
//! the TypeScript reads the same text the same way: a regex is written here exactly as it is
//! written there, and `Re::new` turns it into one that matches what JavaScript's would.
//!
//! What differs from JavaScript and is left: under `i`, `k` and `s` also match the Kelvin sign
//! and the long s.

use fancy_regex::{Regex, RegexBuilder};

/// One match and its groups.
pub type Captures<'t> = fancy_regex::Captures<'t, str>;

/// What JavaScript's `\s` and `trim` call white space.
pub fn is_space(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}

const SPACE: &str = r"\t\n\x0B\x0C\r \x{A0}\x{1680}\x{2000}-\x{200A}\x{2028}\x{2029}\x{202F}\x{205F}\x{3000}\x{FEFF}";
const WORD: &str = "0-9A-Za-z_";
const DIGIT: &str = "0-9";
const LINE_END: &str = r"\n\r\x{2028}\x{2029}";

/// A JavaScript regex source as one this engine reads the same way: `\s`, `\w`, `\d` and `\b`
/// in JavaScript's ASCII sense, `.` stopping at every line end, `^` and `$` at the ends of the
/// text (or of a line under `m`), and a `[` inside a class as the character it is there.
fn translate(source: &str, flags: &str) -> String {
    let multi = flags.contains('m');
    let mut out = String::with_capacity(source.len() + 16);
    if flags.contains('i') {
        out.push_str("(?i)");
    }
    let boundary = format!("(?:(?<=[{WORD}])(?![{WORD}])|(?<![{WORD}])(?=[{WORD}]))");
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
                'b' if !in_class => out.push_str(&boundary),
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
            '^' if multi => out.push_str(&format!(r"(?:\A|(?<=[{LINE_END}]))")),
            '^' => out.push_str(r"\A"),
            '$' if multi => out.push_str(&format!(r"(?:\z|(?=[{LINE_END}]))")),
            '$' => out.push_str(r"\z"),
            _ => out.push(c),
        }
    }
    out
}

/// One of the web app's regular expressions.
pub struct Re(Regex);

impl Re {
    /// `source` and `flags` as the TypeScript writes them between and after the slashes. `g` is
    /// not a property of the regex here: the caller picks `replace` or `replace_all`.
    pub fn new(source: &str, flags: &str) -> Re {
        let pattern = translate(source, flags);
        // A long answer may be scanned line by line; the default budget is for short inputs.
        let re = RegexBuilder::new(&pattern).backtrack_limit(50_000_000).build();
        Re(re.unwrap_or_else(|e| panic!("the regex /{source}/{flags} does not compile as {pattern}: {e}")))
    }

    pub fn test(&self, s: &str) -> bool {
        self.0.is_match(s).unwrap_or(false)
    }

    /// The first match and its groups, as `exec` returns them.
    pub fn exec<'t>(&self, s: &'t str) -> Option<Captures<'t>> {
        self.0.captures(s).ok().flatten()
    }

    /// The byte ranges of every match, in order, as a `g` regex walks them.
    pub fn matches(&self, s: &str) -> Vec<(usize, usize)> {
        self.0.find_iter(s).map_while(Result::ok).map(|m| (m.start(), m.end())).collect()
    }

    /// `s.replace(re, with)` for a regex without `g`: the first match only.
    pub fn replace(&self, s: &str, with: impl Fn(&Captures) -> String) -> String {
        match self.exec(s) {
            Some(caps) => {
                let m = caps.get(0).expect("a match has a whole");
                format!("{}{}{}", &s[..m.start()], with(&caps), &s[m.end()..])
            }
            None => s.to_string(),
        }
    }

    /// `s.replace(re, with)` for a regex with `g`: every match.
    pub fn replace_all(&self, s: &str, with: impl Fn(&Captures) -> String) -> String {
        let mut out = String::with_capacity(s.len());
        let mut last = 0;
        for caps in self.0.captures_iter(s).map_while(Result::ok) {
            let m = caps.get(0).expect("a match has a whole");
            out.push_str(&s[last..m.start()]);
            out.push_str(&with(&caps));
            last = m.end();
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
    caps.get(n).map_or("", |m| m.as_str())
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
