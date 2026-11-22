//! A line of source split into the kinds its colours go by, as a script opened big draws it
//! (`tokenize` in `renderer/src/code-artifact.ts`, which `CodeFile.tsx` draws a line at a time).
//!
//! Deliberately a lexer for one line and not a parser, as the web window's is: anything with
//! state across lines (a block comment, a template literal) would be guessing. A line that
//! opens `/*` and never closes it keeps its ink.
//!
//! Places are counted in characters here, where the web window counts UTF-16 units. They only
//! cut the line where a kind changes, so the pieces are the same.

/// What a piece of a line is, and so its colour (`.t-com` and the rest in `styles.css`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Ink: what is none of the rest.
    Plain,
    Comment,
    Text,
    Keyword,
    Number,
    Type,
    /// A function run or defined.
    Run,
    /// A name being made.
    Made,
    /// A constant in capitals.
    Constant,
}

const KEYWORDS: [&str; 55] = [
    "const", "let", "var", "function", "return", "if", "else", "for", "while", "of", "in", "import", "from", "export", "default", "class", "extends", "new", "await", "async", "try", "catch",
    "finally", "throw", "typeof", "instanceof", "delete", "void", "type", "interface", "enum", "as", "null", "undefined", "true", "false", "this", "switch", "case", "break", "continue", "do",
    "yield", "static", "public", "private", "def", "elif", "lambda", "None", "True", "False", "pass", "raise", "with",
];
/// The words after which the next name, or every name in the next pattern, is being made.
const DECLARES: [&str; 3] = ["const", "let", "var"];
/// The words after which the next name is a function being defined.
const NAMES_A_FUNCTION: [&str; 2] = ["function", "def"];
/// Built-in types, which are lower case and so would otherwise be ink.
const BUILTIN_TYPES: [&str; 9] = ["number", "string", "boolean", "bigint", "symbol", "object", "any", "unknown", "never"];

/// Whether the text right after a name runs it: `(`, `?.(`, or a type argument written against
/// the name and closed before a `(`, as in `useState<T>(` (`runsAt`). A comparison is not one:
/// `a < b` has a space, and `&&`, `||` or a bracket inside the angle brackets is an expression.
fn runs_at(after: &[char]) -> bool {
    let mut k = 0;
    while k < after.len() && after[k].is_whitespace() {
        k += 1;
    }
    if after[k..].starts_with(&['?', '.', '(']) || after.get(k) == Some(&'(') {
        return true;
    }
    if after.first() != Some(&'<') {
        return false;
    }
    let mut depth = 0;
    for j in 0..after.len().min(120) {
        match after[j] {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return after.get(j + 1) == Some(&'(');
                }
            }
            '(' | ')' | ';' => return false,
            c @ ('&' | '|') if after.get(j + 1) == Some(&c) => return false,
            _ => {}
        }
    }
    false
}

fn starts_word(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '$'
}
fn in_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '$'
}

/// One line of source, as the pieces its colours go by (`tokenize`).
pub fn tokenize(line: &str) -> Vec<(Kind, String)> {
    let chars: Vec<char> = line.chars().collect();
    let mut out: Vec<(Kind, String)> = Vec::new();
    let push = |out: &mut Vec<(Kind, String)>, kind: Kind, s: &[char]| {
        if !s.is_empty() {
            out.push((kind, s.iter().collect()));
        }
    };
    // The inside of a `/** */` block is the one cross-line state that announces itself on every
    // line: a leading `*` then a space, a `/` or nothing.
    let lead = chars.iter().take_while(|c| c.is_whitespace()).count();
    if chars.get(lead) == Some(&'*') && chars.get(lead + 1).is_none_or(|c| c.is_whitespace() || *c == '/') {
        push(&mut out, Kind::Plain, &chars[..lead]);
        push(&mut out, Kind::Comment, &chars[lead..]);
        return out;
    }
    let mut plain: Vec<char> = Vec::new();
    // What the last keyword promised about the next name: one being made, a function being
    // defined, or (inside `[ ]` or `{ }` after `const`) a pattern of them, `depth` deep.
    let (mut declaring, mut naming, mut depth) = (false, false, 0usize);
    let mut i = 0;
    while i < chars.len() {
        let rest = &chars[i..];
        let flush = |out: &mut Vec<(Kind, String)>, plain: &mut Vec<char>| {
            push(out, Kind::Plain, plain);
            plain.clear();
        };
        if rest.starts_with(&['/', '/']) || rest.starts_with(&['/', '*']) || (rest[0] == '#' && rest.get(1).is_some_and(|c| *c == '!' || c.is_whitespace())) {
            flush(&mut out, &mut plain);
            push(&mut out, Kind::Comment, rest);
            return out;
        }
        let quote = rest[0];
        if quote == '"' || quote == '\'' || quote == '`' {
            let mut j = 1;
            while j < rest.len() {
                if rest[j] == '\\' {
                    j += 2;
                    continue;
                }
                if rest[j] == quote {
                    j += 1;
                    break;
                }
                j += 1;
            }
            let j = j.min(rest.len());
            flush(&mut out, &mut plain);
            push(&mut out, Kind::Text, &rest[..j]);
            i += j;
            continue;
        }
        if starts_word(rest[0]) {
            let len = rest.iter().take_while(|c| in_word(**c)).count();
            let w: String = rest[..len].iter().collect();
            let after = &rest[len..];
            flush(&mut out, &mut plain);
            let kind = if KEYWORDS.contains(&w.as_str()) {
                declaring = DECLARES.contains(&w.as_str());
                naming = NAMES_A_FUNCTION.contains(&w.as_str());
                Kind::Keyword
            } else if naming {
                naming = false;
                Kind::Run
            } else if declaring {
                declaring = false;
                Kind::Made
            } else if depth > 0 {
                // In a pattern, `a: renamed` makes `renamed`; `a` is the key it reads.
                let key = after.iter().find(|c| !c.is_whitespace()) == Some(&':');
                if key { Kind::Plain } else { Kind::Made }
            } else if BUILTIN_TYPES.contains(&w.as_str()) {
                Kind::Type
            } else if w.len() > 1 && w.chars().next().is_some_and(|c| c.is_ascii_uppercase()) && w.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
                Kind::Constant
            } else if w.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                Kind::Type
            } else if runs_at(after) {
                Kind::Run
            } else {
                Kind::Plain
            };
            push(&mut out, kind, &rest[..len]);
            i += len;
            continue;
        }
        if rest[0].is_ascii_digit() {
            let len = rest.iter().take_while(|c| c.is_ascii_hexdigit() || matches!(c, '_' | '.' | 'x' | 'X')).count();
            flush(&mut out, &mut plain);
            push(&mut out, Kind::Number, &rest[..len]);
            i += len;
            continue;
        }
        if rest.starts_with(&['=', '>']) {
            flush(&mut out, &mut plain);
            push(&mut out, Kind::Keyword, &rest[..2]);
            i += 2;
            declaring = false;
            naming = false;
            continue;
        }
        let ch = rest[0];
        if declaring && (ch == '[' || ch == '{') {
            declaring = false;
            depth = 1;
        } else if depth > 0 && (ch == '[' || ch == '{') {
            depth += 1;
        } else if depth > 0 && (ch == ']' || ch == '}') {
            depth -= 1;
        } else if !ch.is_whitespace() && ch != '*' {
            declaring = false;
            naming = false;
        }
        plain.push(ch);
        i += 1;
    }
    push(&mut out, Kind::Plain, &plain);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(line: &str) -> Vec<String> {
        tokenize(line)
            .into_iter()
            .filter(|(k, _)| *k != Kind::Plain)
            .map(|(k, s)| {
                let tag = match k {
                    Kind::Comment => "com",
                    Kind::Text => "str",
                    Kind::Keyword => "kw",
                    Kind::Number => "num",
                    Kind::Type => "typ",
                    Kind::Run => "fn",
                    Kind::Made => "def",
                    Kind::Constant => "con",
                    Kind::Plain => "",
                };
                format!("{tag}:{s}")
            })
            .collect()
    }
    fn kind_of(line: &str, word: &str) -> Option<Kind> {
        tokenize(line).into_iter().find(|(_, s)| s == word).map(|(k, _)| k).filter(|k| *k != Kind::Plain)
    }

    // tests/function-calls-and-new-names-are-coloured.test.mts
    #[test]
    fn a_function_being_run_or_defined_is_coloured() {
        assert_eq!(kind_of("const plan = fillNext(files, drawn)", "fillNext"), Some(Kind::Run));
        assert_eq!(kind_of("files.findIndex((f) => f.path === startAt)", "findIndex"), Some(Kind::Run));
        assert_eq!(kind_of("useState<ReadonlySet<number>>(() => {", "useState"), Some(Kind::Run));
        assert_eq!(kind_of("setDrawn ((n) => n + 1)", "setDrawn"), Some(Kind::Run));
        assert_eq!(kinds("export function hunkView(rows) {"), ["kw:export", "kw:function", "fn:hunkView"]);
        assert_eq!(kinds("def run(self, n):"), ["kw:def", "fn:run"]);
        assert_eq!(kinds("if (drawn >= files.length) return;"), ["kw:if", "kw:return"]);
        assert_eq!(kind_of("while (i < n) {", "while"), Some(Kind::Keyword));
        assert_eq!(kind_of("if (a < b && c > (d)) x();", "a"), None);
        assert_eq!(kind_of("if (a < b && c > (d)) x();", "c"), None);
        assert_eq!(kind_of("files.findIndex((f) => f.path === startAt)", "path"), None);
        assert_eq!(kind_of("if (drawn.size >= files.length) return;", "size"), None);
    }

    #[test]
    fn a_name_being_made_is_coloured() {
        assert_eq!(kind_of("const plan = fillNext(files)", "plan"), Some(Kind::Made));
        assert_eq!(kind_of("  let state = { ...EMPTY };", "state"), Some(Kind::Made));
        assert_eq!(kind_of("var n = 1", "n"), Some(Kind::Made));
        assert_eq!(kinds("const [drawn, setDrawn] = useState<ReadonlySet<number>>(() => {"), ["kw:const", "def:drawn", "def:setDrawn", "fn:useState", "typ:ReadonlySet", "typ:number", "kw:=>"]);
        assert_eq!(kinds("const { a: renamed, b } = x;"), ["kw:const", "def:renamed", "def:b"]);
        assert_eq!(kind_of("plan = fillNext(files)", "plan"), None);
        assert_eq!(kind_of("const plan = files", "files"), None);
    }

    #[test]
    fn constants_types_comments_and_words() {
        assert_eq!(kind_of("fillNext(files, drawn, BATCH_ROWS)", "BATCH_ROWS"), Some(Kind::Constant));
        assert_eq!(kind_of("(h: number) => void", "number"), Some(Kind::Type));
        assert_eq!(kind_of("let s: string = \"\"", "string"), Some(Kind::Type));
        assert_eq!(kind_of("function id<T>(x: T): T", "T"), Some(Kind::Type));
        assert_eq!(kinds("   * A Z ON A TASK SHOWS IN ITS THREAD (w-c78d1e1607). One"), ["com:* A Z ON A TASK SHOWS IN ITS THREAD (w-c78d1e1607). One"]);
        assert_eq!(kinds("import { cents } from './money';"), ["kw:import", "kw:from", "str:'./money'"]);
        assert_eq!(kinds("return Math.round(line.amount * 100) / 100 // the cent"), ["kw:return", "typ:Math", "fn:round", "num:100", "num:100", "com:// the cent"]);
        assert_eq!(kinds("#!/bin/sh"), ["com:#!/bin/sh"]);
        // Every character comes back, in order.
        let line = "  const é = `a\\`b` + 0x1F; /* x";
        assert_eq!(tokenize(line).into_iter().map(|(_, s)| s).collect::<String>(), line);
    }
}
