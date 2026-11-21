//! A thread's ledger, read as a conversation: who said what, in order, and which answer the
//! page leads with.
//!
//! The engine keeps every write to a thread as one line (`zero:item-history`), and the web
//! window turns those lines into sentences (`renderer/src/thread-history.ts`), lifts the
//! agent's last word out to the foot (`item-thread.ts`) and folds the rest behind one line
//! (`one-thing-at-a-time.ts`). This is the same reading for the lines a plain thread has: the
//! ask, a run picking it up, checkpoints, replies, picks, the result, renames, and being
//! closed, handed back or sent back. What a run did step by step (`zero:session-trace`),
//! notes on files, teammates and snoozes are not read yet.

use serde::Deserialize;
use serde_json::{Map, Value};

/// One write to the thread (`LedgerLine`).
#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Line {
    pub ts: f64,
    /// `founder` for the person, `agent`, or `system` for the app itself.
    pub source: String,
    pub patch: Option<Map<String, Value>>,
    pub claim: Option<Value>,
    pub heartbeat: bool,
    pub release: bool,
}

/// What `zero:item-history` answers.
#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct History {
    pub lines: Vec<Line>,
    pub error: Option<String>,
}

/// What the engine answers about one thread, each answer under its channel's name. A fixture
/// carries them under `threads["<product>/<id>"]`; `docs/thread-data.md` says what each is.
#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Answers {
    #[serde(rename = "zero:item-history")]
    pub history: History,
}

/// Which field of the row a message is.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Field {
    Body,
    Answer,
    Result,
    Note,
}

/// Something somebody said, at reading size.
#[derive(Clone, Debug)]
pub struct Message {
    pub at: f64,
    pub mine: bool,
    pub text: String,
    pub field: Field,
}

/// Something that happened, as one quiet line: a run picking the thread up, a rename, a pick.
#[derive(Clone, Debug)]
pub struct Step {
    pub at: f64,
    pub mine: bool,
    pub verb: String,
    pub words: String,
    /// A picked option answers the agent as a reply does.
    answers: bool,
}

#[derive(Clone, Debug)]
pub enum Said {
    Message(Message),
    Step(Step),
}

impl Said {
    fn at(&self) -> f64 {
        match self {
            Said::Message(m) => m.at,
            Said::Step(s) => s.at,
        }
    }
    fn message(&self) -> Option<&Message> {
        match self {
            Said::Message(m) => Some(m),
            Said::Step(_) => None,
        }
    }
    /// The person's own: their message, or something they did.
    fn mine(&self) -> bool {
        match self {
            Said::Message(m) => m.mine,
            Said::Step(s) => s.mine,
        }
    }
}

/// The conversation in order, and the agent's last word when nothing was said after it.
#[derive(Default, Clone, Debug)]
pub struct Talk {
    pub said: Vec<Said>,
    pub outcome: Option<Message>,
    /// What the person did after that last word, drawn under it.
    pub after: Vec<Step>,
}

fn words(v: Option<&Value>) -> Option<String> {
    let clean = v.and_then(Value::as_str).unwrap_or("").trim();
    (!clean.is_empty() && clean != "(withdrawn)").then(|| clean.to_string())
}

/// "Option 2: Ship it" is a pick, drawn as what the person did and not as a message of theirs.
fn picked(answer: &str) -> Option<(String, String)> {
    let rest = answer.strip_prefix("Option ")?;
    let (n, text) = rest.split_once(": ")?;
    if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) || text.contains('\n') {
        return None;
    }
    let text = match text.to_lowercase().find("(recommended)") {
        Some(at) => format!("{}{}", text[..at].trim_end(), &text[at + "(recommended)".len()..]),
        None => text.to_string(),
    };
    Some((format!("You picked option {n}"), text.trim().to_string()))
}

/// The ledger's lines, oldest first, as what was said and done (`threadEvents`).
fn events(lines: &[Line]) -> Vec<Said> {
    let step = |at: f64, mine: bool, verb: &str, words: String| Said::Step(Step { at, mine, verb: verb.to_string(), words, answers: false });
    let mut out: Vec<Said> = Vec::new();
    let mut title: Option<String> = None;
    let (mut first, mut opened) = (true, false);
    for (n, line) in lines.iter().enumerate() {
        let (at, mine) = (line.ts, line.source == "founder");
        let Some(patch) = line.patch.as_ref() else {
            if line.claim.is_some() && !line.heartbeat && !line.release {
                out.push(step(at, false, "An agent picked it up", String::new()));
            }
            continue;
        };
        // A comment or a note on a file is neither an ask nor an answer, and is not drawn yet.
        if patch.contains_key("comment") || patch.contains_key("artifact") || line.heartbeat || line.release {
            continue;
        }
        if line.claim.is_some() {
            out.push(step(at, false, "An agent picked it up", String::new()));
            continue;
        }
        let body = words(patch.get("body"));
        let speaks = patch.contains_key("answer") || words(patch.get("result")).is_some() || words(patch.get("note")).is_some();
        // The row's birth. The app stamps a new row a moment before the person's own line
        // lands with the same title, and then that second line is the birth.
        if first && patch.contains_key("title") {
            let name = patch.get("title").and_then(Value::as_str).unwrap_or("").to_string();
            let next = lines.get(n + 1);
            let theirs = line.source == "system"
                && body.is_none()
                && next.is_some_and(|l| l.source == "founder" && (l.ts - line.ts).abs() <= 60_000.0 && l.patch.as_ref().and_then(|p| p.get("title")).and_then(Value::as_str) == Some(name.as_str()));
            if theirs {
                continue;
            }
            first = false;
            title = Some(name);
            match body {
                Some(text) => {
                    opened = true;
                    out.push(Said::Message(Message { at, mine, text, field: Field::Body }));
                }
                None => out.push(step(at, mine, if mine { "You opened this" } else { "An agent opened this" }, String::new())),
            }
            continue;
        }
        if patch.is_empty() {
            continue;
        }
        if let (Some(text), false) = (&body, speaks) {
            if !opened {
                opened = true;
                if let Some(name) = patch.get("title").and_then(Value::as_str) {
                    title = Some(name.to_string());
                }
                out.push(Said::Message(Message { at, mine, text: text.clone(), field: Field::Body }));
                continue;
            }
            // The ask, rewritten by the side that wrote it. The other side's never lands.
            if let Some(Said::Message(ask)) = out.iter_mut().find(|e| e.message().is_some_and(|m| m.field == Field::Body))
                && ask.mine == mine
            {
                ask.text = text.clone();
            }
            if !patch.contains_key("title") {
                continue;
            }
        }
        if patch.contains_key("answer") {
            // Words the app handed over from a teammate are theirs, never drawn as the person's.
            let handed = line.source == "system" && patch.get("from").is_some();
            match words(patch.get("answer")) {
                Some(text) if handed => out.push(Said::Message(Message { at, mine: false, text, field: Field::Answer })),
                Some(text) => match picked(&text) {
                    Some((verb, option)) => out.push(Said::Step(Step { at, mine: true, verb, words: option, answers: true })),
                    None => out.push(Said::Message(Message { at, mine: true, text, field: Field::Answer })),
                },
                None => out.push(step(at, true, "You withdrew your reply", String::new())),
            }
            continue;
        }
        if let Some(text) = words(patch.get("result")) {
            out.push(Said::Message(Message { at, mine: false, text, field: Field::Result }));
            continue;
        }
        if let Some(text) = words(patch.get("note")) {
            out.push(Said::Message(Message { at, mine: false, text, field: Field::Note }));
            continue;
        }
        if let Some(name) = patch.get("title").and_then(Value::as_str).filter(|name| title.as_deref() != Some(*name)) {
            let was = title.replace(name.to_string()).filter(|w| !w.is_empty()).map(|w| format!("was \"{w}\"")).unwrap_or_default();
            out.push(step(at, mine, if mine { "You renamed it" } else { "It renamed itself" }, was));
            continue;
        }
        let after_result = || out.last().and_then(Said::message).is_some_and(|m| m.field == Field::Result);
        match patch.get("status").and_then(Value::as_str) {
            Some("done") => out.push(step(at, mine, if mine { "You marked it done" } else { "It closed this" }, String::new())),
            // The app parking a thread after delivering its answer is delivery, not a question,
            // and the person has no control that blocks one.
            Some("blocked") if !mine && !(line.source == "system" && after_result()) => out.push(step(at, false, "It stopped and asked you", String::new())),
            Some("open") if !(line.source == "system" && after_result()) => {
                let verb = if mine { "You sent it back" } else { "It handed this back" };
                // One press can write the reopening twice.
                let twice = matches!(out.last(), Some(Said::Step(last)) if last.verb == verb && (at - last.at).abs() < 1000.0);
                if !twice {
                    out.push(step(at, mine, verb, String::new()));
                }
            }
            _ => {}
        }
    }
    out
}

impl Talk {
    /// The conversation a ledger holds (`itemThread`): in the order things happened, the ask
    /// first, and the agent's last word held out of the stream when it is still the last word.
    pub fn read(lines: &[Line]) -> Talk {
        let mut said = events(lines);
        said.sort_by(|a, b| a.at().total_cmp(&b.at()));
        if let Some(ask) = said.iter().position(|e| e.message().is_some_and(|m| m.field == Field::Body)) {
            let ask = said.remove(ask);
            said.insert(0, ask);
        }
        let mut answer: Option<usize> = None;
        let mut spoke: Option<usize> = None;
        for (i, e) in said.iter().enumerate() {
            match e {
                Said::Step(s) if s.answers => answer = None,
                Said::Step(_) => {}
                Said::Message(m) => {
                    spoke = Some(i);
                    answer = (!m.mine && matches!(m.field, Field::Result | Field::Note)).then_some(i);
                }
            }
        }
        let Some(at) = answer.filter(|a| Some(*a) == spoke) else {
            return Talk { said, outcome: None, after: Vec::new() };
        };
        let mut after = Vec::new();
        let mut kept = Vec::new();
        let mut outcome = None;
        for (i, e) in said.into_iter().enumerate() {
            match e {
                Said::Message(m) if i == at => outcome = Some(m),
                Said::Step(s) if i > at && s.mine => after.push(s),
                other => kept.push(other),
            }
        }
        Talk { said: kept, outcome, after }
    }

    /// The newest answer: the one lifted to the foot, or else the last thing said that was not
    /// the person's.
    pub fn newest(&self) -> Option<&Message> {
        self.outcome.as_ref().or_else(|| self.said.iter().rev().filter_map(Said::message).find(|m| !m.mine && !m.text.trim().is_empty()))
    }

    fn messages(said: &[Said]) -> usize {
        said.iter().filter(|e| e.message().is_some()).count()
    }
}

/// How one thread's page is laid out (`ItemThread` under "one thing at a time"): the newest
/// answer alone under a line that unfolds the rest, or the conversation from some point on.
pub struct Page<'a> {
    /// The line under the title that folds and unfolds, when there is one.
    pub toggle: Option<String>,
    /// The newest answer, drawn alone.
    pub alone: Option<&'a Message>,
    /// Otherwise the conversation from here on, with the outcome under it.
    pub said: &'a [Said],
}

impl Talk {
    /// `working` is whether an agent is on the thread right now; `unfolded` whether the person
    /// asked for the earlier messages; `imported` whether it was brought in from another app.
    pub fn page(&self, working: bool, unfolded: bool, imported: bool) -> Page<'_> {
        let newest = self.newest();
        // Where the page starts when the newest answer has been answered since, or while an
        // agent works: at that answer, so the question stays in view over what followed.
        let stands = if working {
            let yours = self.said.iter().rposition(Said::mine);
            Some(yours.and_then(|y| self.said[..y].iter().rposition(|e| e.message().is_some_and(|m| !m.mine && !m.text.trim().is_empty()))).unwrap_or(0))
        } else {
            newest.filter(|n| self.said.iter().any(|e| e.at() > n.at && e.mine())).and_then(|n| self.said.iter().position(|e| e.message().is_some_and(|m| !m.mine && m.at == n.at)))
        };
        let alone = newest.is_some() && !working && stands.is_none();
        let behind = match stands {
            Some(from) => Talk::messages(&self.said[..from]),
            None if alone => match &self.outcome {
                Some(_) => Talk::messages(&self.said),
                None => Talk::messages(&self.said).saturating_sub(1),
            },
            None => 0,
        };
        // A short history stays in place, unless the page leads with a fresh answer.
        let short = !unfolded && !alone && !imported && behind > 0 && behind < 3;
        let steps = !short && alone && (self.said.iter().any(|e| e.message().is_none()) || !self.after.is_empty());
        let toggle = if !short && behind > 0 {
            Some(if unfolded { "Hide earlier messages".to_string() } else { format!("Show {behind} earlier {}", if behind == 1 { "message" } else { "messages" }) })
        } else if steps {
            Some(if unfolded { "Hide what the agent did" } else { "Show what the agent did" }.to_string())
        } else {
            None
        };
        if alone && !unfolded && !short {
            return Page { toggle, alone: newest, said: &[] };
        }
        let from = if short || unfolded { 0 } else { stands.unwrap_or(0) };
        Page { toggle, alone: None, said: &self.said[from..] }
    }
}

/// Which of the person's messages open a turn of theirs and which close one, for the rules
/// drawn over and under it (`herTurnStarts` and `herTurnEnds` in `her-turns.ts`).
pub fn turns(said: &[Said]) -> Vec<(bool, bool)> {
    let mut last_mine: Option<bool> = None;
    said.iter()
        .enumerate()
        .map(|(n, e)| {
            let Some(m) = e.message() else { return (false, false) };
            let starts = m.mine && last_mine != Some(true) && n > 0;
            last_mine = Some(m.mine);
            if !m.mine {
                return (false, false);
            }
            // The turn ends where the agent speaks next; with only steps after, it is on it.
            let ends = match said[n + 1..].iter().find_map(Said::message) {
                Some(next) => !next.mine,
                None => n < said.len() - 1,
            };
            (starts, ends)
        })
        .collect()
}

/// An answer that opens with a line that is bold and nothing else wears it as its headline
/// (`splitHeadline`); one with a link in it stays in the answer, where links are drawn.
pub fn headline(text: &str) -> (Option<&str>, &str) {
    let trimmed = text.trim_start();
    let Some(rest) = trimmed.strip_prefix("**") else { return (None, text) };
    let Some(close) = rest.find("**") else { return (None, text) };
    let (bold, after) = (&rest[..close], &rest[close + 2..]);
    let line_end = after.find('\n').unwrap_or(after.len());
    if bold.is_empty() || bold.contains(['*', '\n']) || !after[..line_end].trim_matches([' ', '\t']).is_empty() || (bold.contains("](") && bold.contains(')')) {
        return (None, text);
    }
    (Some(bold.trim()), after[line_end..].trim_start())
}
