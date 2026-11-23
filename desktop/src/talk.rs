//! A thread's ledger and the logs its runs left, read as a conversation: who said what, what
//! each run did between the messages, in the order it happened, and which answer the page
//! leads with.
//!
//! The engine keeps every write to a thread as one line (`zero:item-history`), and the web
//! window turns those lines into sentences (`renderer/src/thread-history.ts`). Each run that
//! picked the thread up left a log (`zero:session-trace`), and the web window hangs each under
//! the pickup that started it (`withRuns`) and pours it out as the run's own sentences and the
//! tools it ran, each at the moment it happened (`itemThread` in `item-thread.ts`). It says one
//! answer once, lifts the agent's last word out to the foot, keeps the opening and the end of a
//! long conversation with a line for the middle, and folds the rest behind one line
//! (`one-thing-at-a-time.ts`). This is the same reading. Notes on files, teammates, snoozes and
//! what the person sent that the ledger does not have yet are not read yet.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use serde_json::{Map, Value};

use crate::rules::js::{self, group, re};
use crate::rules::notes::TraceSession;
use crate::rules::trace::{self, Kind};
use crate::rules::work;

/// One write to the thread (`LedgerLine`).
#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Line {
    pub ts: f64,
    /// `founder` for the person, `agent`, or `system` for the app itself.
    pub source: String,
    /// Whose line it is, on a Mercury with a team: the person's id.
    pub by: Option<String>,
    pub patch: Option<Map<String, Value>>,
    pub claim: Option<Value>,
    pub heartbeat: bool,
    pub release: bool,
    /// The line's own name, which a message is found and landed on by (`LedgerLine.uid`).
    pub uid: Option<String>,
}

/// What `zero:item-history` answers.
#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct History {
    pub lines: Vec<Line>,
    pub error: Option<String>,
}

/// What `zero:session-trace` answers: each run's log.
#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Traces {
    pub sessions: Vec<TraceSession>,
}

/// What the engine answers about one thread, each answer under its channel's name. A fixture
/// carries them under `threads["<product>/<id>"]`; `docs/thread-data.md` says what each is.
#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Answers {
    #[serde(rename = "zero:item-history")]
    pub history: History,
    #[serde(rename = "zero:session-trace")]
    pub traces: Traces,
    /// Every log whole, read once the conversation's middle was asked for. A fixture's picture
    /// was taken with the regular answer standing for both.
    #[serde(rename = "zero:session-trace/whole")]
    pub every: Option<Traces>,
}

impl Answers {
    /// Each run's log as the page draws it: the whole copy where one was read, with the regular
    /// read laid over it (`withEveryRun`).
    pub fn sessions(&self) -> Vec<TraceSession> {
        match &self.every {
            Some(every) => trace::with_every_run(&every.sessions, &self.traces.sessions),
            None => self.traces.sessions.clone(),
        }
    }
}

/// Which field of the row a message is, or the run it was typed into.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Field {
    Body,
    Answer,
    Result,
    Note,
    /// Something the agent said in a run's log.
    Run,
}

/// Something somebody said, at reading size.
#[derive(Clone, Debug)]
pub struct Message {
    pub at: f64,
    pub mine: bool,
    pub text: String,
    pub field: Field,
    /// The same run still talking after the line above it: drawn with no name and no time.
    pub same: bool,
    /// The first thing after a long conversation's missing middle, which gets its name back.
    pub resumed: bool,
    /// The person's words sent this moment and not yet in the ledger, drawn from the press on:
    /// what its head says in place of the time (`PendingSaid`).
    pub sending: Option<Sending>,
    /// The uid of the ledger line it came off, when it came off one that has one (`tagWriter` in
    /// `renderer/src/thread-history.ts`).
    pub uid: Option<String>,
}

impl Message {
    fn new(at: f64, mine: bool, text: String, field: Field) -> Message {
        Message { at, mine, text, field, same: false, resumed: false, sending: None, uid: None }
    }
    /// What a found line names this message by: its line's uid, or when it was said
    /// (`data-line` in `Thread.tsx`).
    pub fn line_name(&self) -> String {
        crate::rules::found::line_name(self.uid.as_deref(), self.at)
    }
}

/// Gives the messages from `from` on the uid of the line they came off, where they have none.
fn tag_line(out: &mut [Said], from: usize, uid: &Option<String>) {
    let Some(uid) = uid else { return };
    for e in out.iter_mut().skip(from) {
        if let Said::Message(m) = e
            && m.uid.is_none()
        {
            m.uid = Some(uid.clone());
        }
    }
}

/// Where something the person sent is, as its head says it (`headFacts` in
/// `renderer/src/components/Thread.tsx`): held three seconds and still taken back by Z, then on
/// its way to the agent, or waiting on the step a Claude Code run is in, with Send now beside
/// it, and sent now once that was pressed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Sending {
    Held,
    OnItsWay,
    WaitingOnTheStep,
    SendingNow,
}

/// Something the person just sent, as `Talk::read_with` places it: when it was pressed, what it
/// said ("Option 2: …" for a pick) and where it is.
#[derive(Clone, Debug, PartialEq)]
pub struct Sent {
    pub at: f64,
    pub text: String,
    pub sending: Sending,
}

/// A Z pressed on this thread, as one line of the person's own: `from` and `at` are when it
/// started and finished, and the ledger lines it wrote between them fold under it (`UndoMark`).
#[derive(Clone, Debug, PartialEq)]
pub struct Undid {
    pub from: f64,
    pub at: f64,
    pub words: String,
    pub choice: Option<String>,
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
    /// A run picking the thread up, which its log stands in for when there is one.
    pickup: bool,
}

/// One tool a run ran, as its quiet line (`AgentWork` from `workLine` in `item-thread.ts`).
#[derive(Clone, Debug, PartialEq)]
pub struct Work {
    pub at: f64,
    pub verb: String,
    pub subject: String,
    /// The whole command or path, when the line's words shortened it.
    pub full: String,
    /// What came back, as the log kept it, and how many lines it had.
    pub output: String,
    pub lines: usize,
    pub failed: bool,
    /// How many more a run past its cap ran, which are not shown (`capRuns`).
    pub more: usize,
    /// The path an editing tool wrote, whole, which a chip opens in the thread's change
    /// (`changedFile` in `shared/work-lines.mts`); empty for anything else.
    pub file: String,
}

#[derive(Clone, Debug)]
pub enum Said {
    Message(Message),
    Step(Step),
    Work(Work),
}

impl Said {
    fn at(&self) -> f64 {
        match self {
            Said::Message(m) => m.at,
            Said::Step(s) => s.at,
            Said::Work(w) => w.at,
        }
    }
    pub fn message(&self) -> Option<&Message> {
        match self {
            Said::Message(m) => Some(m),
            _ => None,
        }
    }
    /// The person's own: their message, or something they did.
    fn mine(&self) -> bool {
        match self {
            Said::Message(m) => m.mine,
            Said::Step(s) => s.mine,
            Said::Work(_) => false,
        }
    }
    /// What the web window counts as a message: anything not a work line (`kind !== 'work'`).
    fn is_message(&self) -> bool {
        matches!(self, Said::Message(_))
    }
}

/// The conversation in order, and the agent's last word when nothing was said after it.
#[derive(Default, Clone, Debug)]
pub struct Talk {
    pub said: Vec<Said>,
    pub outcome: Option<Message>,
    /// What the person did after that last word, drawn under it.
    pub after: Vec<Step>,
    /// How many messages of a long conversation's middle are left out until asked for.
    pub omitted: usize,
    /// While an agent works, what it ran since anybody last said anything, oldest first: taken
    /// out of the conversation for the line at its foot, which unfolds onto it.
    pub trailing: Vec<Work>,
}

/// The sentence a running agent is typing this second, off the snapshot (`LiveSaying`).
#[derive(Clone, Debug, PartialEq)]
pub struct Saying {
    pub text: String,
    pub at: f64,
    /// The run typing it, by when that run started.
    pub run: f64,
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
    let step = |at: f64, mine: bool, verb: &str, words: String| Said::Step(Step { at, mine, verb: verb.to_string(), words, answers: false, pickup: false });
    let pickup = |at: f64| Said::Step(Step { at, mine: false, verb: "An agent picked it up".to_string(), words: String::new(), answers: false, pickup: true });
    let mut out: Vec<Said> = Vec::new();
    let mut title: Option<String> = None;
    let (mut first, mut opened) = (true, false);
    // Each line's messages are told the line's uid when the next line begins, and once more
    // after the last (`tagWriter`).
    let (mut tag_from, mut tag_uid): (usize, Option<String>) = (0, None);
    for (n, line) in lines.iter().enumerate() {
        tag_line(&mut out, tag_from, &tag_uid);
        tag_from = out.len();
        tag_uid = line.uid.clone().filter(|uid| !uid.is_empty());
        let (at, mine) = (line.ts, line.source == "founder");
        let Some(patch) = line.patch.as_ref() else {
            if line.claim.is_some() && !line.heartbeat && !line.release {
                out.push(pickup(at));
            }
            continue;
        };
        // A comment or a note on a file is neither an ask nor an answer, and is not drawn yet.
        if patch.contains_key("comment") || patch.contains_key("artifact") || line.heartbeat || line.release {
            continue;
        }
        if line.claim.is_some() {
            out.push(pickup(at));
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
                    out.push(Said::Message(Message::new(at, mine, text, Field::Body)));
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
                out.push(Said::Message(Message::new(at, mine, text.clone(), Field::Body)));
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
                Some(text) if handed => out.push(Said::Message(Message::new(at, false, text, Field::Answer))),
                Some(text) => match picked(&text) {
                    Some((verb, option)) => out.push(Said::Step(Step { at, mine: true, verb, words: option, answers: true, pickup: false })),
                    None => out.push(Said::Message(Message::new(at, true, text, Field::Answer))),
                },
                None => out.push(step(at, true, "You withdrew your reply", String::new())),
            }
            continue;
        }
        if let Some(text) = words(patch.get("result")) {
            out.push(Said::Message(Message::new(at, false, text, Field::Result)));
            continue;
        }
        if let Some(text) = words(patch.get("note")) {
            out.push(Said::Message(Message::new(at, false, text, Field::Note)));
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
    tag_line(&mut out, tag_from, &tag_uid);
    out
}

/* ------------------------------------- a run's lines ------------------------------------- */

// `[Read] strategy/art-style.md`: how the supervisor writes a tool call into a log.
re!(TOOL, r"^\[([^\]]+)\]\s*(.*)$", "");
// A row's id is how the store addresses it, never what a person reads.
re!(ROW_ID, r"^w-[0-9a-f]{10}$", "i");
// The change the supervisor saves for the code viewer is not a file the agent wrote.
re!(THE_CHANGE, r"(?:^|\/)runs\/[^/]+\/the-change-it-made\.change$", "");

/// One tool line of a log, as the quiet line the thread draws (`workLine`): what a shell
/// command did in words when it can be read, else the command; a path from its end.
fn work_line(at: f64, name: &str, hint: &str) -> Work {
    let plain = if name == "Bash" { work::command_work(hint, "", "") } else { None };
    let whole = js::trim(hint);
    let subject = match &plain {
        Some(p) => p.subject.clone(),
        None if ROW_ID.test(whole) => String::new(),
        None if name == "Bash" => work::plain_command(hint),
        None => work::short_path(hint, "", ""),
    };
    // The whole string travels whenever the words replaced it, so it stays one press away.
    let full = if plain.is_some() || (!subject.is_empty() && subject != whole) { whole.to_string() } else { String::new() };
    let verb = plain.map(|p| p.verb).unwrap_or_else(|| work::work_verb(name));
    // A log names an editing tool's file as its hint, whole.
    let file = if crate::rules::change::changes_a_file(name) { whole.to_string() } else { String::new() };
    Work { at, verb, subject, full, output: String::new(), lines: 0, failed: false, more: 0, file }
}

/// A cloud run handing its thread back to the Mac, drawn as the move.
fn moved(at: f64) -> Work {
    Work { at, verb: "moved".to_string(), subject: "to this Mac".to_string(), full: String::new(), output: String::new(), lines: 0, failed: false, more: 0, file: String::new() }
}

thread_local! {
    /// Each run's lines as last read, by its start, with how long its log was then. A log only
    /// grows, so while a run is going only its own log is read again; the web window keeps the
    /// same per log (`notesOf` in `thread-history.ts`). One reading per run, a hundred runs at most.
    static RUNS_READ: std::cell::RefCell<HashMap<u64, (usize, std::rc::Rc<Vec<Said>>)>> = std::cell::RefCell::new(HashMap::new());
}

/// `run_nodes`, read again only when the run's log has grown.
fn run_nodes_once(session: &TraceSession) -> std::rc::Rc<Vec<Said>> {
    let (key, length) = (session.started_at.to_bits(), session.text.len());
    if let Some(read) = RUNS_READ.with(|memo| memo.borrow().get(&key).filter(|(l, _)| *l == length).map(|(_, read)| read.clone())) {
        return read;
    }
    let read = std::rc::Rc::new(run_nodes(session));
    RUNS_READ.with(|memo| {
        let mut memo = memo.borrow_mut();
        if memo.len() >= 100 && !memo.contains_key(&key) {
            memo.clear();
        }
        memo.insert(key, (length, read.clone()));
    });
    read
}

/// One run, as the lines the thread draws for it (`runNodes`): its sentences are messages, and
/// the tools it ran the quiet lines under them. The store's own bookkeeping is left out.
pub fn run_nodes(session: &TraceSession) -> Vec<Said> {
    let mut out: Vec<Said> = Vec::new();
    for line in trace::trace_lines(session) {
        if trace::hands_back(&line.text) {
            if !matches!(out.last(), Some(Said::Work(w)) if w.verb == "moved") {
                out.push(Said::Work(moved(line.at)));
            }
            continue;
        }
        if line.kind == Kind::Tool {
            let parts = TOOL.exec(&line.text);
            let (name, hint) = parts.as_ref().map_or(("", ""), |p| (group(p, 1), group(p, 2)));
            if parts.is_some() && work::is_bookkeeping(name) {
                continue;
            }
            if name == "Write" && THE_CHANGE.test(js::trim(hint)) {
                continue;
            }
            let mut w = if parts.is_some() { work_line(line.at, name, hint) } else { work_line(line.at, "", &line.text) };
            if let Some(output) = line.output {
                w.output = output;
                w.lines = line.output_lines.unwrap_or(0);
                w.failed = line.failed;
            }
            out.push(Said::Work(w));
            continue;
        }
        out.push(Said::Message(Message::new(line.at, false, line.text, Field::Run)));
    }
    out
}

/// Each pickup's runs (`withRuns` in `thread-history.ts`). A worker claims its thread as its
/// first action, so the claim lands after its run began, sometimes by an hour when the Mac
/// slept first: each pickup takes the latest run that had already started, oldest pickup
/// first. Runs older than every pickup hang under the first one rather than vanish, and a
/// pickup with none left keeps its sentence.
fn runs_by_pickup(events: &[Said], sessions: &[TraceSession]) -> HashMap<usize, Vec<TraceSession>> {
    let pickups: Vec<(usize, f64)> = events.iter().enumerate().filter_map(|(i, e)| matches!(e, Said::Step(s) if s.pickup).then(|| (i, e.at()))).collect();
    let mut ordered: Vec<&TraceSession> = sessions.iter().filter(|s| !s.text.is_empty()).collect();
    ordered.sort_by(|a, b| a.started_at.total_cmp(&b.started_at));
    let mut mine: HashMap<usize, Vec<TraceSession>> = HashMap::new();
    if pickups.is_empty() || ordered.is_empty() {
        return mine;
    }
    let mut taken = vec![false; ordered.len()];
    for (event, at) in &pickups {
        if let Some(i) = (0..ordered.len()).rev().find(|i| !taken[*i] && ordered[*i].started_at <= *at) {
            taken[i] = true;
            mine.insert(*event, vec![ordered[i].clone()]);
        }
    }
    let mut orphans: Vec<TraceSession> = ordered.iter().zip(&taken).filter(|(_, t)| !**t).map(|(s, _)| (*s).clone()).collect();
    if !orphans.is_empty() {
        orphans.extend(mine.remove(&pickups[0].0).unwrap_or_default());
        orphans.sort_by(|a, b| a.started_at.total_cmp(&b.started_at));
        mine.insert(pickups[0].0, orphans);
    }
    mine
}

/* ------------------------------------ one thing said once ------------------------------------ */

/// One node with the run it came from, before everything is put in order (`Placed`).
struct Placed {
    node: Said,
    run: Option<f64>,
    /// A ledger result or checkpoint of the agent's: an answer.
    answer: bool,
    /// Something the person did that answered the agent (a pick).
    answers: bool,
}

/// Something said, with words in it (`isSaid`).
fn said_text(p: &Placed) -> Option<&Message> {
    p.node.message().filter(|m| !js::trim(&m.text).is_empty())
}

re!(SPACES, r"\s+", "g");
re!(BOLD, r"\*\*", "g");
re!(ENDING, r"[.?!:]+$", "");
re!(LEADING_SPACE, r"^\s+", "");

/// One message flattened, so two copies written through different surfaces compare equal.
fn flat(text: &str) -> String {
    js::trim(&SPACES.replace_all(text, |_| " ".to_string())).to_lowercase()
}

/// The first line with its bold off, the sentence read twice (`opening`).
fn opening(text: &str) -> String {
    let line = js::trim(text.split('\n').find(|l| !js::trim(l).is_empty()).unwrap_or(""));
    ENDING.strip(&flat(&BOLD.strip_all(line)))
}

/// Below this a message is too short for one found inside another to mean anything.
const ENOUGH: usize = 40;

/// The same message with its first line taken off, or None when it is only that line.
fn without_opening(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let first = lines.iter().position(|l| !js::trim(l).is_empty())?;
    let rest = LEADING_SPACE.strip(&lines[first + 1..].join("\n"));
    (!js::trim(&rest).is_empty()).then_some(rest)
}

/// One answer typed twice: the same opening, or the row's answer whole inside the typed copy
/// with nothing under it but that copy's own first line (`oneAnswerTwice`).
fn one_answer_twice(answer: &str, typed: &str) -> bool {
    if opening(answer) == opening(typed) {
        return true;
    }
    let words = flat(answer);
    let Some(rest) = without_opening(typed) else { return false };
    if words.is_empty() || js::len(&words) < ENOUGH {
        return false;
    }
    js::trim(&flat(&rest).replacen(&words, "", 1)).is_empty()
}

/// An earlier write of a field goes when a later write of it says the same words and the
/// person has not spoken in between (`supersededFields`).
fn superseded_fields(placed: &[Placed]) -> HashSet<usize> {
    let mut gone = HashSet::new();
    let mut seen: HashMap<String, usize> = HashMap::new();
    for (i, p) in placed.iter().enumerate() {
        let Some(m) = said_text(p) else { continue };
        if m.mine {
            seen.clear();
            continue;
        }
        if !p.answer {
            continue;
        }
        let key = format!("{:?}\n{}", m.field, flat(&m.text));
        if let Some(before) = seen.insert(key, i) {
            gone.insert(before);
        }
    }
    gone
}

/// Two repeats folded and nothing else (`collapseRepeats`): the same words again, and a run's
/// closing message that types again the answer it wrote to the row. Two messages that merely
/// open alike both stay, the second without the line already on the screen above it. The
/// person's own words are never folded away.
fn collapse_repeats(placed: Vec<Placed>) -> Vec<Placed> {
    // The last thing each run said: its answer sits there, and only that line of a run may go.
    let mut last_said: HashMap<u64, usize> = HashMap::new();
    for (i, p) in placed.iter().enumerate() {
        if let (Some(run), Some(_)) = (p.run, said_text(p)) {
            last_said.insert(run.to_bits(), i);
        }
    }
    let mut gone = superseded_fields(&placed);
    let mut headless: HashMap<usize, String> = HashMap::new();
    let mut prev: Option<usize> = None;
    for i in 0..placed.len() {
        let Some(here) = said_text(&placed[i]) else { continue };
        if gone.contains(&i) {
            continue;
        }
        let last = prev.and_then(|p| said_text(&placed[p]).map(|m| (p, m)));
        let Some((at, last)) = last.filter(|(_, m)| !m.mine && !here.mine) else {
            prev = Some(i);
            continue;
        };
        let (a, b) = (flat(&last.text), flat(&here.text));
        if a.is_empty() || b.is_empty() {
            prev = Some(i);
            continue;
        }
        // Word for word: the row's own field wins over a copy typed into a run; otherwise the
        // later one, so a live conversation still ends where it ended.
        if a == b {
            if placed[at].answer && !placed[i].answer {
                gone.insert(i);
            } else {
                gone.insert(at);
                prev = Some(i);
            }
            continue;
        }
        let closing_here = placed[i].run.is_some_and(|r| last_said.get(&r.to_bits()) == Some(&i)) && placed[at].answer;
        let closing_last = placed[at].run.is_some_and(|r| last_said.get(&r.to_bits()) == Some(&at)) && placed[i].answer;
        if closing_here && one_answer_twice(&last.text, &here.text) {
            gone.insert(i);
            continue;
        }
        if closing_last && one_answer_twice(&here.text, &last.text) {
            gone.insert(at);
            prev = Some(i);
            continue;
        }
        // One holds the other whole: the fuller text stays.
        if b.contains(&a) && js::len(&a) >= ENOUGH {
            gone.insert(at);
            prev = Some(i);
            continue;
        }
        if a.contains(&b) && js::len(&b) >= ENOUGH {
            gone.insert(i);
            continue;
        }
        if opening(&last.text) != opening(&here.text) {
            prev = Some(i);
            continue;
        }
        if let Some(rest) = without_opening(&here.text) {
            headless.insert(i, rest);
        }
        prev = Some(i);
    }
    placed
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !gone.contains(i))
        .map(|(i, mut p)| {
            if let (Some(rest), Said::Message(m)) = (headless.remove(&i), &mut p.node) {
                m.text = rest;
            }
            p
        })
        .collect()
}

/* ------------------------------------- a long conversation ------------------------------------- */

/// The opening a long conversation keeps, and how many messages it keeps in all (`threadWindow`).
const OPENING: usize = 3;
const KEEP: usize = 40;
/// The most work lines one unbroken run shows, and the whole thread (`capRuns`).
const WORK_RUN: usize = 20;
const WORK_TOTAL: usize = 200;

/// Messages, not blocks: a run that talked three times between its tools is one message drawn
/// as three (`saidCount`).
fn said_count(said: &[Said]) -> usize {
    said.iter().filter(|e| e.message().is_some_and(|m| !m.text.is_empty() && !m.same)).count()
}

/// A run past its cap says how many it left out on its last line, and past the whole thread's
/// cap the oldest work lines go and every message stays (`capRuns`).
fn cap_runs(said: Vec<Said>) -> Vec<Said> {
    let mut out: Vec<Said> = Vec::new();
    let (mut in_run, mut dropped) = (0, 0);
    let note = |out: &mut Vec<Said>, dropped: usize| {
        if let Some(Said::Work(w)) = out.last_mut() {
            w.more = dropped;
        }
    };
    for e in said {
        if e.is_message() {
            if dropped > 0 {
                note(&mut out, dropped);
                dropped = 0;
            }
            in_run = 0;
            out.push(e);
            continue;
        }
        in_run += 1;
        if in_run > WORK_RUN {
            dropped += 1;
            continue;
        }
        out.push(e);
    }
    if dropped > 0 {
        note(&mut out, dropped);
    }
    let work = out.iter().filter(|e| !e.is_message()).count();
    if work <= WORK_TOTAL {
        return out;
    }
    let mut over = work - WORK_TOTAL;
    out.into_iter()
        .filter(|e| {
            let goes = over > 0 && !e.is_message() && !matches!(e, Said::Work(w) if w.more > 0);
            if goes {
                over -= 1;
            }
            !goes
        })
        .collect()
}

/// The opening and the end of a long conversation, every message once its middle was asked
/// for (`threadWindow`). It counts messages only, so tool calls never push a message out.
fn thread_window(said: Vec<Said>, whole: bool) -> Vec<Said> {
    let all: Vec<Said> = said.into_iter().filter(|e| e.message().is_none_or(|m| !m.text.is_empty())).collect();
    let said_at: Vec<usize> = all.iter().enumerate().filter(|(_, e)| e.is_message()).map(|(i, _)| i).collect();
    if whole || said_at.len() <= KEEP {
        return cap_runs(all);
    }
    let (from, to) = (said_at[OPENING], said_at[said_at.len() - (KEEP - OPENING)]);
    let mut kept: Vec<Said> = Vec::new();
    for (i, mut e) in all.into_iter().enumerate() {
        if i >= from && i < to {
            continue;
        }
        // The first block after the cut continues something hidden, so it gets its name back.
        if i == to
            && let Said::Message(m) = &mut e
            && m.same
        {
            m.resumed = true;
        }
        kept.push(e);
    }
    cap_runs(kept)
}

/// Where the line for the missing middle stands: after the last thing the opening kept
/// (`gapIndex`), or None when nothing is missing.
pub fn gap_index(said: &[Said], omitted: usize) -> Option<usize> {
    if omitted == 0 {
        return None;
    }
    let mut blocks = 0;
    for (i, e) in said.iter().enumerate() {
        if !e.is_message() {
            continue;
        }
        blocks += 1;
        if blocks == OPENING + 1 {
            return i.checked_sub(1);
        }
    }
    said.len().checked_sub(1)
}

impl Talk {
    /// The conversation a ledger and its runs' logs hold (`itemThread`): in the order things
    /// happened, the ask first, the agent's last word held out of the stream while it is still
    /// the last word, and the middle of a long one left out unless `whole`. `saying` is the
    /// sentence a running agent is typing, which the log has not caught yet.
    /// Kept for a reader that places nothing sent and no Z (the tests, the conversation's steps).
    #[allow(dead_code)]
    pub fn read(lines: &[Line], sessions: &[TraceSession], saying: Option<&Saying>, whole: bool) -> Talk {
        Talk::read_with(lines, sessions, saying, whole, &[], &[])
    }

    /// The same, with what the person has just sent and the ledger does not have yet (`sent`),
    /// and the Zs pressed on the thread (`undone`), each placed where the web window places them
    /// (`pending` and `undone` in `itemThread`).
    pub fn read_with(lines: &[Line], sessions: &[TraceSession], saying: Option<&Saying>, whole: bool, sent: &[Sent], undone: &[Undid]) -> Talk {
        let events = events(lines);
        let runs = runs_by_pickup(&events, sessions);
        let mut placed: Vec<Placed> = Vec::new();
        let place = |node: Said, run: Option<f64>| Placed { node, run, answer: false, answers: false };
        // A run nothing claimed for (a person's own chat) is placed by its own times.
        if !events.iter().any(|e| matches!(e, Said::Step(s) if s.pickup)) {
            for session in sessions.iter().filter(|s| !s.text.is_empty()) {
                placed.extend(run_nodes_once(session).iter().cloned().map(|node| place(node, Some(session.started_at))));
            }
        }
        for (i, event) in events.into_iter().enumerate() {
            match event {
                // A pickup is its run: what the run said and did stands in for the sentence.
                Said::Step(s) if s.pickup => match runs.get(&i) {
                    Some(runs) => {
                        for run in runs {
                            placed.extend(run_nodes_once(run).iter().cloned().map(|node| place(node, Some(run.started_at))));
                        }
                    }
                    None => placed.push(place(Said::Step(s), None)),
                },
                Said::Message(mut m) => {
                    let answer = !m.mine && matches!(m.field, Field::Result | Field::Note);
                    // A cloud run's hand-back line is the app's: drawn as the move, once.
                    if answer && trace::hands_back(&m.text) {
                        let rest = trace::without_hand_back(&m.text);
                        if rest.is_empty() {
                            let just_before = placed.iter().any(|p| matches!(&p.node, Said::Work(w) if w.verb == "moved" && w.at <= m.at && m.at - w.at < 600_000.0));
                            if !just_before {
                                placed.push(place(Said::Work(moved(m.at)), None));
                            }
                            continue;
                        }
                        m.text = rest;
                    }
                    placed.push(Placed { node: Said::Message(m), run: None, answer, answers: false });
                }
                Said::Step(s) => {
                    let answers = s.answers;
                    placed.push(Placed { node: Said::Step(s), run: None, answer: false, answers });
                }
                work @ Said::Work(_) => placed.push(place(work, None)),
            }
        }
        // A Z is one thing the person did, so it is one line: the lines its own undo wrote,
        // stamped while it ran, fold under it (a withdrawn reply, a thread sent back).
        for mark in undone {
            placed.retain(|p| !matches!(&p.node, Said::Step(s) if s.mine && !s.answers && s.at >= mark.from && s.at <= mark.at));
            placed.push(place(Said::Step(Step { at: mark.at, mine: true, verb: mark.words.clone(), words: mark.choice.clone().unwrap_or_default(), answers: false, pickup: false }), None));
        }
        // The sentence being typed right now, unless the log already has the same words.
        if let Some(saying) = saying {
            let now = trace::said(&trace::without_hand_back(&saying.text));
            let kept = trace::as_traced(&now);
            let already = placed.iter().any(|p| p.node.message().is_some_and(|m| !m.mine && [&now, &kept].contains(&&trace::said(&m.text))));
            if !now.is_empty() && !already {
                placed.push(Placed { node: Said::Message(Message::new(saying.at, false, now, Field::Run)), run: Some(saying.run), answer: false, answers: false });
            }
        }
        // And what the person has just sent, which the ledger does not have yet. One written
        // copy cancels one drawn copy, oldest first, and only a copy written at or after the
        // press: "yes" sent twice into the same silence shows twice. A pick is drawn as the line
        // its written copy will be, so nothing moves when the write lands.
        let mut spent: Vec<usize> = Vec::new();
        let mut mine: Vec<&Sent> = sent.iter().collect();
        mine.sort_by(|a, b| a.at.total_cmp(&b.at));
        for sent in mine {
            let words = trace::said(&sent.text);
            if words.is_empty() {
                continue;
            }
            if let Some((verb, option)) = picked(&words) {
                let landed = placed.iter().enumerate().position(|(n, p)| !spent.contains(&n) && matches!(&p.node, Said::Step(s) if s.mine && s.answers && s.at >= sent.at && s.verb == verb && s.words == option));
                match landed {
                    Some(n) => spent.push(n),
                    None => placed.push(Placed { node: Said::Step(Step { at: sent.at, mine: true, verb, words: option, answers: true, pickup: false }), run: None, answer: false, answers: true }),
                }
                continue;
            }
            let landed = placed.iter().enumerate().position(|(n, p)| !spent.contains(&n) && p.node.message().is_some_and(|m| m.mine && m.at >= sent.at && trace::said(&m.text) == words));
            match landed {
                Some(n) => spent.push(n),
                None => {
                    let mut message = Message::new(sent.at, true, sent.text.clone(), Field::Answer);
                    message.sending = Some(sent.sending);
                    placed.push(Placed { node: Said::Message(message), run: None, answer: false, answers: false });
                }
            }
        }
        // Tool-call scaffolding off every message before two of them are compared.
        for p in &mut placed {
            if let Said::Message(m) = &mut p.node {
                m.text = work::unleaked(&m.text);
            }
        }
        // In the order it happened; two things stamped the same moment keep the order they came in.
        placed.sort_by(|a, b| a.node.at().total_cmp(&b.node.at()));
        if let Some(ask) = placed.iter().position(|p| p.node.message().is_some_and(|m| m.field == Field::Body)) {
            let ask = placed.remove(ask);
            placed.insert(0, ask);
        }
        let once = collapse_repeats(placed);

        let mut said: Vec<Said> = Vec::new();
        let mut spoke: Option<usize> = None;
        let mut answer: Option<usize> = None;
        let mut prev_run: Option<Option<f64>> = None;
        for p in once {
            match p.node {
                Said::Message(mut m) => {
                    m.same = p.run.is_some() && prev_run == Some(p.run);
                    prev_run = Some(p.run);
                    spoke = Some(said.len());
                    // A run's own narration moves the last word past an answer without wiping
                    // it; only a ledger message that is no answer does.
                    if p.answer {
                        answer = Some(said.len());
                    } else if p.run.is_none() {
                        answer = None;
                    }
                    said.push(Said::Message(m));
                }
                other => {
                    if p.answers {
                        answer = None;
                    }
                    said.push(other);
                }
            }
        }
        // Counted before the answer is lifted: it is still a message on the thread.
        let spoken = said_count(&said);
        let mut outcome = None;
        let mut after = Vec::new();
        if let Some(at) = answer.filter(|a| Some(*a) == spoke) {
            // What the person did after the answer goes under it.
            let mut kept = Vec::new();
            for (i, e) in said.into_iter().enumerate() {
                match e {
                    Said::Message(m) if i == at => outcome = Some(m),
                    Said::Step(s) if i > at && s.mine => after.push(s),
                    other => kept.push(other),
                }
            }
            said = kept;
        }
        let said = thread_window(said, whole);
        let shown = said_count(&said) + usize::from(outcome.is_some());
        Talk { said, outcome, after, omitted: spoken.saturating_sub(shown), trailing: Vec::new() }
    }

    /// The newest answer: the one lifted to the foot, or else the last thing said that was not
    /// the person's.
    pub fn newest(&self) -> Option<&Message> {
        self.outcome.as_ref().or_else(|| self.said.iter().rev().filter_map(Said::message).find(|m| !m.mine && !m.text.trim().is_empty()))
    }

    fn messages(said: &[Said]) -> usize {
        said.iter().filter(|e| e.is_message()).count()
    }

    /// While an agent is on the thread, the work since anybody last said anything is what the
    /// line at the foot stands for, so it leaves the conversation until the run ends
    /// (`withoutTrailingWork`), and is kept for that line to unfold onto. What the person did
    /// stays.
    pub fn without_trailing_work(&mut self) {
        while self.said.last().is_some_and(|e| matches!(e, Said::Work(_)) || matches!(e, Said::Step(s) if !s.mine)) {
            if let Some(Said::Work(work)) = self.said.pop() {
                self.trailing.insert(0, work);
            }
        }
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
    /// The line for a long conversation's missing middle: after which of `said`, and how many.
    pub gap: Option<(usize, usize)>,
    /// How many messages the conversation drawn leaves out, as `ItemThread` tells `Thread`
    /// (`omitted={from >= 0 ? 0 : omitted}`): none when it starts part way down.
    pub omitted: usize,
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
        // Starting inside the opening a long conversation keeps would cut off its gap line.
        let gap = gap_index(&self.said, self.omitted);
        let start = stands.filter(|s| gap.is_none_or(|g| *s > g));
        let alone = newest.is_some() && !working && stands.is_none();
        let behind = match start {
            Some(from) => Talk::messages(&self.said[..from]) + self.omitted,
            None if alone => {
                self.omitted
                    + match &self.outcome {
                        Some(_) => Talk::messages(&self.said),
                        None => Talk::messages(&self.said).saturating_sub(1),
                    }
            }
            None => 0,
        };
        // A short history stays in place, unless the page leads with a fresh answer.
        let short = !unfolded && !alone && !imported && behind > 0 && behind < 3;
        let steps = !short && alone && (self.said.iter().any(|e| !e.is_message()) || !self.after.is_empty());
        let toggle = if !short && behind > 0 {
            Some(if unfolded { "Hide earlier messages".to_string() } else { format!("Show {behind} earlier {}", if behind == 1 { "message" } else { "messages" }) })
        } else if steps {
            Some(if unfolded { "Hide what the agent did" } else { "Show what the agent did" }.to_string())
        } else {
            None
        };
        if alone && !unfolded && !short {
            return Page { toggle, alone: newest, said: &[], gap: None, omitted: 0 };
        }
        match (short || unfolded, start) {
            (false, Some(from)) => Page { toggle, alone: None, said: &self.said[from..], gap: None, omitted: 0 },
            _ => Page { toggle, alone: None, said: &self.said, gap: gap.map(|g| (g, self.omitted)), omitted: self.omitted },
        }
    }
}

/// Which of the person's messages open a turn of theirs and which close one, for the rules
/// drawn over and under it (`herTurnStarts` and `herTurnEnds` in `her-turns.ts`). The cut
/// middle of a long conversation counts as a break.
pub fn turns(said: &[Said], cut: Option<usize>) -> Vec<(bool, bool)> {
    let mut last_mine: Option<bool> = None;
    said.iter()
        .enumerate()
        .map(|(n, e)| {
            if n > 0 && Some(n - 1) == cut {
                last_mine = None;
            }
            let Some(m) = e.message() else { return (false, false) };
            let starts = m.mine && last_mine != Some(true) && n > 0;
            last_mine = Some(m.mine);
            if !m.mine {
                return (false, false);
            }
            // The turn ends where the agent speaks next; with only steps after, it is on it.
            let mut ends = n < said.len() - 1;
            for (k, next) in said.iter().enumerate().skip(n + 1) {
                if Some(k - 1) == cut {
                    ends = true;
                    break;
                }
                if let Some(next) = next.message() {
                    ends = !next.mine;
                    break;
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn line(v: Value) -> Line {
        serde_json::from_value(v).unwrap()
    }

    fn run(started_at: f64, body: &[&str]) -> TraceSession {
        TraceSession { started_at, text: body.join("\n") }
    }

    fn at(iso: &str) -> f64 {
        crate::rules::notes::iso_ms(iso).unwrap()
    }

    fn texts(said: &[Said]) -> Vec<String> {
        said.iter()
            .map(|e| match e {
                Said::Message(m) => format!("{}{}", if m.same { "…" } else { "" }, m.text),
                Said::Step(s) => format!("[{}]", s.verb),
                Said::Work(w) => format!("<{} {}>", w.verb, w.subject).replace(" >", ">"),
            })
            .collect()
    }

    /// A found line is landed on by its ledger line's uid (`tagWriter`, `data-line`): each
    /// message carries the uid of the line it came off, the ask keeps its own when it is
    /// rewritten, and a line with none is named by when it was said.
    #[test]
    fn each_message_carries_the_uid_of_the_line_it_came_off() {
        let lines: Vec<Line> = [
            json!({ "ts": 1000, "source": "founder", "uid": "l-ask", "patch": { "title": "Checkout fails", "body": "Checkout fails for some carts." } }),
            json!({ "ts": 2000, "source": "agent", "uid": "l-found", "patch": { "result": "Found it: tax rounding." } }),
            json!({ "ts": 3000, "source": "founder", "uid": "l-edit", "patch": { "body": "Checkout fails for carts over $1,000." } }),
            json!({ "ts": 4000, "source": "founder", "patch": { "answer": "Ship the fix." } }),
            json!({ "ts": 5000, "source": "agent", "uid": "", "patch": { "result": "Shipped." } }),
        ]
        .into_iter()
        .map(line)
        .collect();
        let talk = Talk::read(&lines, &[], None, false);
        let mut named: Vec<(String, String)> = talk.said.iter().filter_map(Said::message).map(|m| (m.text.clone(), m.line_name())).collect();
        named.extend(talk.outcome.iter().map(|m| (m.text.clone(), m.line_name())));
        assert_eq!(
            named,
            [
                ("Checkout fails for carts over $1,000.".to_string(), "l-ask".to_string()),
                ("Found it: tax rounding.".to_string(), "l-found".to_string()),
                ("Ship the fix.".to_string(), "at-4000".to_string()),
                ("Shipped.".to_string(), "at-5000".to_string()),
            ]
        );
    }

    /// `tests/a-thread-reads-in-plain-words.test.mts`: the first six lines of a real thread's
    /// own log. The store's bookkeeping drops out and the work around it stays.
    #[test]
    fn the_stores_bookkeeping_drops_out_of_the_conversation() {
        let nodes = run_nodes(&run(at("2026-08-12T21:05:00Z"), &[
            "21:05:03  I'll start by claiming the item and reading the project state.",
            "21:05:04  [ToolSearch] ",
            "21:05:06  [Bash] ls",
            "21:05:10  [mcp__agentbox__claim_work_item] w-0bd0d8b2ef",
            "21:05:11  [Bash] cat STATE.md",
            "21:05:40  [mcp__agentbox__update_work_item] w-0bd0d8b2ef",
        ]));
        assert_eq!(texts(&nodes), ["I'll start by claiming the item and reading the project state.", "<listed ./>", "<read STATE.md>"]);
        // The exact command is still one press away.
        let [Said::Work(line)] = &run_nodes(&run(0.0, &["21:05:06  [Bash] cat README.md; echo ---; cat package.json"]))[..] else { panic!() };
        assert_eq!((line.verb.as_str(), line.subject.as_str(), line.full.as_str()), ("read", "README.md and package.json", "cat README.md; echo ---; cat package.json"));
    }

    /// `tests/one-door-one-list.test.mts`: a run hangs under the latest pickup it had already
    /// started by, one whose machine slept an hour first included, and a claim with no log
    /// keeps its sentence.
    #[test]
    fn a_run_hangs_under_the_pickup_that_started_it() {
        let t = |hhmm: &str| at(&format!("2026-08-14T{hhmm}:00Z"));
        let claim = |ts: f64| json!({ "ts": ts, "source": "agent", "claim": { "holder": "mcp-1", "leaseUntil": ts + 300_000.0 }, "patch": { "status": "claimed" } });
        let lines: Vec<Line> = [
            json!({ "ts": t("08:00"), "source": "agent", "patch": { "title": "the row", "status": "open", "body": "the ask" } }),
            claim(t("08:23")),
            claim(t("09:56")),
            claim(t("13:07")),
            claim(t("13:12")),
            claim(t("14:39")),
        ]
        .into_iter()
        .map(line)
        .collect();
        let trace = |started: f64, said: &str| {
            let l = rune::time::LocalTime::at_offset(started as i64 + 1000, 0);
            run(started, &["# a task", "", &format!("{:02}:{:02}:{:02}  {said}", l.hour, l.minute, l.second), "09:00:04  [Bash] git status"])
        };
        let sessions = [
            trace(t("08:22") + 50_000.0, "Now the pure event mapping."),
            trace(t("09:55") + 42_000.0, "Now the renderer seam."),
            // Its session began before the machine slept, and the claim came an hour later.
            trace(t("12:15"), "Now the wiring in Focus.tsx."),
            trace(t("14:38") + 48_000.0, "Now the page she will actually open."),
        ];
        let talk = Talk::read(&lines, &sessions, None, false);
        let said: Vec<String> = texts(&talk.said).into_iter().filter(|t| !t.starts_with('<')).collect();
        assert_eq!(said, ["the ask", "Now the pure event mapping.", "Now the renderer seam.", "Now the wiring in Focus.tsx.", "[An agent picked it up]", "Now the page she will actually open."]);
        // It never reaches forward for a run that had not started yet.
        let late: Vec<Said> = events(&[line(claim(t("08:00"))), line(claim(t("10:00")))]);
        let matched = runs_by_pickup(&late, &[trace(t("09:00"), "the 9am run")]);
        assert_eq!((matched.get(&0).is_none(), matched.get(&1).map(|r| r[0].started_at)), (true, Some(t("09:00"))));
        // And loses no run: one older than every pickup still lands, under the first.
        let one: Vec<Said> = events(&[line(claim(t("10:00")))]);
        let matched = runs_by_pickup(&one, &[trace(t("08:00"), "the run with no claim of its own"), trace(t("09:59"), "the run that claimed")]);
        assert_eq!(matched[&0].iter().map(|r| r.started_at).collect::<Vec<_>>(), [t("08:00"), t("09:59")]);
    }

    fn checkout() -> (Vec<Line>, Vec<TraceSession>) {
        let asked = at("2026-11-20T21:00:00Z");
        let lines: Vec<Line> = [
            json!({ "ts": asked, "source": "founder", "patch": { "title": "Checkout fails", "body": "Find out why.", "status": "open" } }),
            json!({ "ts": asked + 60_000.0, "source": "agent", "claim": { "holder": "mcp-1" }, "patch": { "status": "claimed" } }),
            json!({ "ts": asked + 600_000.0, "source": "agent", "patch": { "status": "done", "result": "**Found it: tax rounding breaks on carts over $1,000.**\n\nThe fix is on main." } }),
        ]
        .into_iter()
        .map(line)
        .collect();
        let sessions = vec![run(asked + 50_000.0, &[
            "21:00:53  [mcp__mercury__claim_work_item] w-1",
            "21:00:59  Reading the checkout code.",
            "21:01:02  [Read] /Users/you/dev/storefront/src/checkout/tax.ts",
            "21:01:10  [Bash] npm test -- tax",
            "#id t1",
            "#out t1 2 error",
            "#| FAIL tax.test.ts",
            "#|   ✕ rounds to the cent",
            "21:04:00  The tax service returns four decimals now.",
            "21:09:58  [mcp__mercury__update_work_item] w-1",
            "21:10:03  **Found it: tax rounding breaks on carts over $1,000.**",
            "",
            "The fix is on main, the same words as the row.",
            "",
            "21:10:04  == RESULT (success · 9 turns) ==",
            "**Found it: tax rounding breaks on carts over $1,000.**",
            "",
            "# exited (0) 2026-11-20T21:10:06.000Z",
        ])];
        (lines, sessions)
    }

    /// `tests/one-thing-said-once.test.mts`: a run's closing message that types again the
    /// answer it wrote to the row is drawn once, as the row's.
    #[test]
    fn a_runs_lines_land_among_the_messages_and_one_answer_is_said_once() {
        let (lines, sessions) = checkout();
        let talk = Talk::read(&lines, &sessions, None, false);
        assert_eq!(texts(&talk.said), ["Find out why.", "Reading the checkout code.", "<read …/src/checkout/tax.ts>", "<ran the tests>", "…The tax service returns four decimals now."]);
        let Said::Work(test) = &talk.said[3] else { panic!() };
        assert!(test.failed && test.output == "FAIL tax.test.ts\n  ✕ rounds to the cent" && test.lines == 2);
        assert_eq!(talk.outcome.as_ref().map(|m| m.field), Some(Field::Result));
        // The page that leads with it counts what each run said too.
        assert_eq!(talk.page(false, false, false).toggle.as_deref(), Some("Show 3 earlier messages"));
    }

    /// A log read as a live run writes it, one line more each time: what is already drawn never
    /// changes, and only the last thing can still be growing (a sentence running onto another
    /// line, a command's output arriving).
    #[test]
    fn a_log_that_grows_only_ever_adds_at_its_foot() {
        let (_, sessions) = checkout();
        let whole = &sessions[0];
        let all: Vec<&str> = whole.text.split('\n').collect();
        let full = run_nodes(whole);
        let same = |a: &Said, b: &Said| match (a, b) {
            (Said::Message(x), Said::Message(y)) => y.text.starts_with(&x.text),
            (Said::Work(x), Said::Work(y)) => (x.verb.as_str(), x.subject.as_str()) == (y.verb.as_str(), y.subject.as_str()) && y.output.starts_with(&x.output),
            _ => false,
        };
        for k in 1..=all.len() {
            let part = run_nodes(&TraceSession { started_at: whole.started_at, text: all[..k].join("\n") });
            assert!(part.len() <= full.len(), "{k} lines");
            for (n, node) in part.iter().enumerate() {
                assert!(same(node, &full[n]), "{k} lines, node {n}");
                if n + 1 < part.len()
                    && let (Said::Message(a), Said::Message(b)) = (node, &full[n])
                {
                    assert_eq!(a.text, b.text, "{k} lines: only the last can still be growing");
                }
            }
        }
    }

    /// `tests/the-thinking-mark-is-the-lowest-thing.test.mts`: while a run is up, the work since
    /// anybody last spoke is the live line's to say, and comes back once the run ends.
    #[test]
    fn the_work_since_anybody_spoke_leaves_while_a_run_is_up() {
        let (lines, sessions) = checkout();
        let mut talk = Talk::read(&lines[..2], &[run(sessions[0].started_at, &sessions[0].text.lines().take(5).collect::<Vec<_>>())], None, false);
        assert_eq!(texts(&talk.said), ["Find out why.", "Reading the checkout code.", "<read …/src/checkout/tax.ts>", "<ran the tests>"]);
        talk.without_trailing_work();
        assert_eq!(texts(&talk.said), ["Find out why.", "Reading the checkout code."]);
    }

    /// `tests/an-answer-with-an-options-heading-is-drawn-once-while-the-run-finishes.test.mts`:
    /// the sentence being typed is placed by when it started, and not drawn again once the log
    /// carries the same words.
    #[test]
    fn the_sentence_being_typed_is_drawn_once() {
        let (lines, sessions) = checkout();
        let typing = Saying { text: "The tax service returns four decimals now.".to_string(), at: at("2026-11-20T21:04:00Z"), run: sessions[0].started_at };
        let talk = Talk::read(&lines[..2], &sessions, Some(&typing), false);
        assert_eq!(talk.said.iter().filter(|e| e.message().is_some_and(|m| m.text.contains("four decimals"))).count(), 1);
        let fresh = Saying { text: "Writing it up now. ".to_string(), at: at("2026-11-20T21:05:00Z"), run: sessions[0].started_at };
        let talk = Talk::read(&lines[..2], &sessions[..0], Some(&fresh), false);
        assert_eq!(texts(&talk.said), ["Find out why.", "[An agent picked it up]", "Writing it up now."]);
    }

    /// `tests/a-long-thread-shows-every-run-when-you-open-the-middle.test.mts`: a long
    /// conversation keeps its opening and its end with the middle counted, and asked for, whole.
    #[test]
    fn a_long_conversation_keeps_its_opening_and_its_end() {
        let start = at("2026-11-20T08:00:00Z");
        // Fifty runs that each said one thing, placed by their own times (nothing claimed for them).
        let sessions: Vec<TraceSession> = (0..50)
            .map(|n| {
                let begun = start + f64::from(n) * 60_000.0;
                let l = rune::time::LocalTime::at_offset(begun as i64 + 1000, 0);
                TraceSession { started_at: begun, text: format!("{:02}:{:02}:{:02}  Run {n} said out loud.", l.hour, l.minute, l.second) }
            })
            .collect();
        let lines = vec![line(json!({ "ts": start - 1000.0, "source": "founder", "patch": { "title": "t", "body": "the ask" } }))];
        let talk = Talk::read(&lines, &sessions, None, false);
        assert_eq!(talk.omitted, 51 - KEEP);
        assert_eq!(said_count(&talk.said), KEEP);
        assert_eq!(gap_index(&talk.said, talk.omitted), Some(2));
        let whole = Talk::read(&lines, &sessions, None, true);
        assert_eq!((whole.omitted, said_count(&whole.said)), (0, 51));
        // One run's fifty sentences are one message: the window keeps its blocks to forty, and
        // there is no middle of messages to count, as in the web window.
        let one = [TraceSession { started_at: start, text: sessions.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join("\n") }];
        let talk = Talk::read(&lines, &one, None, false);
        assert_eq!((talk.omitted, talk.said.len()), (0, KEEP));
        assert!(talk.said.iter().filter_map(Said::message).any(|m| m.resumed));
    }

    #[test]
    fn a_run_past_its_cap_says_how_many_it_left_out() {
        let work = |n: usize| (0..n).map(|k| Said::Work(Work { at: k as f64, verb: "ran".into(), subject: String::new(), full: String::new(), output: String::new(), lines: 0, failed: false, more: 0, file: String::new() }));
        let capped = cap_runs(work(23).collect());
        assert_eq!(capped.len(), WORK_RUN);
        assert!(matches!(capped.last(), Some(Said::Work(w)) if w.more == 3));
        assert_eq!(work::run_overflow(3), "and 3 more things it ran, not shown");
    }

    #[test]
    fn a_turn_of_the_persons_is_framed_across_the_cut() {
        let msg = |mine: bool| Said::Message(Message::new(0.0, mine, "x".into(), Field::Answer));
        let said = [msg(false), msg(true), msg(true), msg(false)];
        assert_eq!(turns(&said, None), [(false, false), (true, false), (false, true), (false, false)]);
        assert_eq!(turns(&said, Some(1)), [(false, false), (true, true), (true, true), (false, false)]);
    }
}
