//! What an agent's runs did, read off the logs they left: the run's own sentences as its steps,
//! and the counts and the time under them (`renderer/src/notes.ts`, word for word).
//!
//! The supervisor writes one log per run under `<product>/sessions/<thread id>/<startedAt>.log`,
//! every line `HH:MM:SS  <what happened>` on a clock that is UTC and carries no date. A line is
//! the agent saying something in plain words, a tool it ran (`[Read] notes.md`), a stderr chunk,
//! the result block, or the exit. Nothing here is summarised or guessed: if a run said little,
//! its notes say little.
//!
//! The web window reads a thread's runs through this (`withRuns` in `thread-history.ts` keeps
//! each run's whole log as `raw`), and draws them through `trace.rs` and `work.rs`, which take
//! the date a line's clock is on from here (`moment_of`). The diary itself, its one-line
//! summaries (`closed_line`, `tally_line`) and the live step (`live_step`) are drawn by no
//! screen of the web window today; they are kept, with their tests, as the web keeps them, for
//! the screen that draws them.
#![allow(dead_code)]

use std::collections::HashSet;

use rune::time::LocalTime;
use serde::{Deserialize, Serialize};

use super::calendar::Day;
use super::js::{self, group, re};

const DAY_MS: f64 = 86_400_000.0;

/// One run's log, as the engine hands it over (`TraceSession`).
#[derive(Deserialize, Serialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TraceSession {
    pub started_at: f64,
    pub text: String,
}

/// One thing the agent said, with the moment it said it on the person's clock.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    /// "5:16pm".
    pub time: String,
    pub text: String,
    /// The tools that ran under this step, before the agent said anything else.
    pub actions: usize,
    /// The last thing a failed run has to say for itself.
    pub failed: bool,
}

/// Every run on a thread, read as one diary in the order it happened.
#[derive(Clone, Debug, PartialEq)]
pub struct Notes {
    pub steps: Vec<Step>,
    /// Every tool call in the runs.
    pub actions: usize,
    /// Files written or changed, and threads filed. Counted, never guessed.
    pub files: usize,
    pub filed: usize,
    pub started_at: f64,
    /// When the last run ended, or None while one is still going.
    pub ended_at: Option<f64>,
    /// Time worked: each run's own life, added up, which is not the time the thread existed.
    pub worked: f64,
    /// Set while a run is still going, so a live line can add the time since it started.
    pub open_since: Option<f64>,
    pub failed: bool,
    /// What went wrong, in words rather than an error code.
    pub failure: Option<String>,
    pub sessions: usize,
    /// Every log, joined.
    pub raw: String,
}

re!(LINE, r"^(\d\d):(\d\d):(\d\d) {2}(.*)$", "");
re!(EXIT, r"^# exited \((-?\d+)\)(?:\s+(\S+))?", "");
re!(PARAGRAPH, r"\n\s*\n", "");
re!(MARKS, r"[\s*_`#]+", "g");
re!(NOT_YOUR_LIMIT, r"not your usage limit", "i");
re!(LIMITING, r"temporarily limiting requests", "i");
re!(LIMIT, r"usage limit|rate.?limit|quota", "i");
re!(RESETS, r"resets? at ([^.\n]+)", "i");
re!(SLEPT, r"went to sleep mid-response", "i");
re!(CREDIT, r"credit balance|insufficient", "i");
re!(API_ERROR, r"^API Error:\s*", "i");
re!(ERROR_WORD, r"ERROR", "");
re!(ERROR_ANY_CASE, r"error", "i");
re!(FLAVOUR, r"\(([^)]*)\)", "");
re!(WROTE_A_FILE, r"^(Write|Edit|NotebookEdit)$", "");
re!(FILED, r"create_work_item", "");
re!(NOT_A_STEP, r"^(tool:|stderr:|done:|session exited|fast exit)", "");

/// "5:16pm", as the person reads a clock (`clock`).
fn clock(h: u32, m: u32) -> String {
    let hour = if h.is_multiple_of(12) { 12 } else { h % 12 };
    format!("{hour}:{m:02}{}", if h < 12 { "am" } else { "pm" })
}

/// The moment a log's `HH:MM:SS` names (`momentOf`). The clock is UTC and has no date, so the
/// run's own start lends the date, and the day rolls on when a run crosses midnight UTC. A line
/// is held against the start's whole second, because the stamp has none smaller: a run spawned
/// at 18:30:28.097 whose line reads 18:30:28 did not write it the next day.
pub fn moment_of(started_at: f64, h: u32, m: u32, s: u32) -> f64 {
    let day = (started_at / DAY_MS).floor() * DAY_MS;
    let at = day + f64::from(h) * 3_600_000.0 + f64::from(m) * 60_000.0 + f64::from(s) * 1000.0;
    if at < (started_at / 1000.0).floor() * 1000.0 { at + DAY_MS } else { at }
}

/// The same moment in the time zone the person is sitting in (`hers`).
fn hers(at: f64) -> String {
    let t = LocalTime::at(at as i64);
    clock(t.hour, t.minute)
}

/// The three numbers of a stamped line.
fn stamp(caps: &js::Captures) -> (u32, u32, u32) {
    let n = |i| group(caps, i).parse::<u32>().unwrap_or(0);
    (n(1), n(2), n(3))
}

/// `Date.parse` of the moment an exit line carries, which the supervisor writes with
/// `toISOString()`: `2026-08-12T17:20:14.000Z`. A moment with an offset reads too; anything
/// else is not a moment, as `Date.parse` says of a word.
pub fn iso_ms(s: &str) -> Option<f64> {
    let b = s.as_bytes();
    let num = |from: usize, to: usize| -> Option<i64> { s.get(from..to).filter(|d| d.bytes().all(|c| c.is_ascii_digit())).and_then(|d| d.parse().ok()) };
    if b.len() < 19 || b[4] != b'-' || b[7] != b'-' || !matches!(b[10], b'T' | b't') || b[13] != b':' || b[16] != b':' {
        return None;
    }
    let (year, month, day, hour, minute, second) = (num(0, 4)?, num(5, 7)?, num(8, 10)?, num(11, 13)?, num(14, 16)?, num(17, 19)?);
    let mut at = 19;
    let mut millis = 0.0;
    if b.get(at) == Some(&b'.') {
        let end = at + 1 + b[at + 1..].iter().take_while(|c| c.is_ascii_digit()).count();
        let digits = &s[at + 1..end];
        if digits.is_empty() {
            return None;
        }
        millis = (format!("0.{digits}").parse::<f64>().ok()? * 1000.0).floor();
        at = end;
    }
    let offset = match &s[at..] {
        "Z" | "z" => 0,
        rest if rest.len() == 6 && matches!(rest.as_bytes()[0], b'+' | b'-') && rest.as_bytes()[3] == b':' => {
            let (h, m) = (num(at + 1, at + 3)?, num(at + 4, at + 6)?);
            let sign = if rest.starts_with('-') { -1 } else { 1 };
            sign * (h * 60 + m)
        }
        _ => return None,
    };
    if !(1..=12).contains(&month) || hour > 24 || minute > 59 || second > 59 {
        return None;
    }
    let date = Day::real(year, month - 1, day)?;
    let ms = (date.0 * 86_400 + hour * 3600 + minute * 60 + second - offset * 60) as f64 * 1000.0 + millis;
    Some(ms)
}

/// A step is one sentence the person can read, not a wall (`firstParagraph`): its first
/// paragraph, and past 300 characters, cut at the last sentence that ends after 120.
pub fn first_paragraph(text: &str) -> String {
    let para = js::trim(PARAGRAPH.split(text)[0]);
    if js::len(para) <= 300 {
        return para.to_string();
    }
    let cut = js::slice(para, 0, 300);
    let stop = [". ", "? ", "! "].iter().filter_map(|end| js::last_index_of(cut, end)).max();
    let kept = match stop {
        Some(stop) if stop > 120 => js::slice(cut, 0, stop + 1),
        _ => cut,
    };
    format!("{}…", js::trim(kept))
}

/// The sign-off and the result are the same words, never the same length: compared on
/// whichever opening is shorter, with enough of it to be sure (`sameOpening`).
fn same_opening(a: &str, b: &str) -> bool {
    let norm = |s: &str| js::trim(&MARKS.replace_all(s, |_| " ".to_string())).to_lowercase();
    let (x, y) = (norm(a), norm(b));
    let width = js::len(&x).min(js::len(&y)).min(60);
    width >= 25 && js::slice(&x, 0, width) == js::slice(&y, 0, width)
}

/// The engine's servers turning a request away, which is not the person's limit
/// (`SERVER_BUSY` in `shared/spawn-trouble.mts`).
pub fn server_busy(text: &str) -> bool {
    NOT_YOUR_LIMIT.test(text) || LIMITING.test(text)
}

/// Claude's own failure text, said the way the person would say it (`humanFailure`). Anything
/// not met before keeps the words it came in, less the "API Error:" label: a failure nobody
/// recognised is never softened into a friendlier one.
pub fn human_failure(text: &str) -> String {
    let raw = js::trim(text);
    if raw.is_empty() {
        return "The run stopped without saying why.".to_string();
    }
    if server_busy(raw) {
        return "Claude's servers turned the request away for a moment. This is not your plan's limit.".to_string();
    }
    if LIMIT.test(raw) {
        let reset = RESETS.exec(raw).and_then(|c| c.get(1)).map(|r| format!(", and resets at {}", js::trim(r))).unwrap_or_default();
        return format!("Claude turned the session away: the usage limit is used up{reset}.");
    }
    if SLEPT.test(raw) {
        return "Your computer went to sleep mid-response, so the agent was cut off part-way.".to_string();
    }
    if CREDIT.test(raw) {
        return "Claude refused the session: the account is out of credit.".to_string();
    }
    let rest = API_ERROR.strip(raw);
    js::slice(rest.split('\n').next().unwrap_or(""), 0, 300).to_string()
}

/// One run, read (`readSession`).
struct Session {
    steps: Vec<Step>,
    actions: usize,
    files: HashSet<String>,
    filed: usize,
    failed: bool,
    ended_at: Option<f64>,
}

fn read_session(session: &TraceSession) -> Session {
    let mut steps: Vec<Step> = Vec::new();
    let mut files = HashSet::new();
    let (mut actions, mut filed) = (0, 0);
    // The step the lines below belong to, by its place in `steps`.
    let mut current: Option<usize> = None;
    let (mut in_result, mut result_text, mut result_failed) = (false, String::new(), false);
    let mut exit_code: Option<i64> = None;
    let mut ended_at: Option<f64> = None;
    // The moment the run reported its result, for a run that never got an exit line: a killed
    // worker leaves its RESULT and nothing after it, and it is over all the same.
    let mut said_at: Option<f64> = None;
    // A stderr chunk's lines after its first carry no prefix.
    let mut in_stderr = false;

    for line in session.text.split('\n') {
        let Some(stamped) = LINE.exec(line) else {
            if line.starts_with("stderr:") {
                in_stderr = true;
                continue;
            }
            if in_stderr {
                if !js::trim(line).is_empty() && !line.starts_with('#') {
                    continue;
                }
                in_stderr = false;
            }
            if let Some(exit) = EXIT.exec(line) {
                exit_code = group(&exit, 1).parse().ok();
                if let Some(at) = exit.get(2).and_then(iso_ms) {
                    ended_at = Some(at);
                }
                current = None;
                continue;
            }
            if line.starts_with('#') {
                continue;
            }
            if in_result {
                result_text.push_str(line);
                result_text.push('\n');
                continue;
            }
            // A step that ran onto more lines. Only prose continues a step; once an action has
            // come between, there is nothing to continue.
            if let (Some(at), false) = (current, js::trim(line).is_empty()) {
                steps[at].text.push('\n');
                steps[at].text.push_str(line);
            }
            continue;
        };
        let body = group(&stamped, 4);
        let (h, m, s) = stamp(&stamped);
        in_stderr = body.starts_with("stderr:");
        if body.starts_with("== RESULT") {
            in_result = true;
            said_at = Some(moment_of(session.started_at, h, m, s));
            let flavour = FLAVOUR.exec(body).and_then(|c| c.get(1)).unwrap_or("");
            result_failed = ERROR_WORD.test(body) || ERROR_ANY_CASE.test(flavour);
            current = None;
            continue;
        }
        in_result = false;
        if body.starts_with('[') {
            let close = body.find(']').filter(|c| *c > 0);
            let tool = close.map_or("", |c| &body[1..c]);
            let hint = close.map_or("", |c| js::trim(&body[c + 1..]));
            actions += 1;
            if let Some(at) = current {
                steps[at].actions += 1;
            }
            if WROTE_A_FILE.test(tool) && !hint.is_empty() {
                files.insert(hint.to_string());
            }
            if FILED.test(tool) {
                filed += 1;
            }
            continue;
        }
        if body.starts_with("stderr:") || js::trim(body).is_empty() {
            continue;
        }
        steps.push(Step { time: hers(moment_of(session.started_at, h, m, s)), text: body.to_string(), actions: 0, failed: false });
        current = Some(steps.len() - 1);
    }

    // The sign-off is the result, already on the screen above the notes.
    if steps.last().is_some_and(|last| !result_text.is_empty() && same_opening(&last.text, &result_text)) {
        steps.pop();
    }
    for step in &mut steps {
        step.text = first_paragraph(&step.text);
    }
    // A run that said its result is over, whether or not anything wrote its exit line.
    if ended_at.is_none() {
        ended_at = said_at;
    }
    let failed = result_failed || exit_code.is_some_and(|c| c != 0);
    if failed {
        let time = ended_at.filter(|e| *e != 0.0).map(hers).unwrap_or_default();
        steps.push(Step { time, text: human_failure(&result_text), actions: 0, failed: true });
    }
    Session { steps, actions, files, filed, failed, ended_at }
}

/// Every run on a thread, as one diary in the order it happened (`readNotes`). A continuation
/// is the next few lines of the same story, and only the last run's fate is the thread's: a
/// first attempt that died and a second that finished is a finished thread.
pub fn read_notes(sessions: &[TraceSession]) -> Option<Notes> {
    let mut ordered: Vec<&TraceSession> = sessions.iter().filter(|s| !s.text.is_empty()).collect();
    ordered.sort_by(|a, b| a.started_at.total_cmp(&b.started_at));
    let first = ordered.first()?.started_at;
    let mut steps = Vec::new();
    let mut files = HashSet::new();
    let (mut actions, mut filed, mut worked) = (0, 0, 0.0);
    let (mut failed, mut failure, mut ended_at, mut open_since) = (false, None, None, None);
    for session in &ordered {
        let read = read_session(session);
        failed = read.failed;
        failure = if read.failed { read.steps.last().map(|s| s.text.clone()) } else { None };
        steps.extend(read.steps);
        files.extend(read.files);
        actions += read.actions;
        filed += read.filed;
        ended_at = read.ended_at;
        match read.ended_at.filter(|e| *e != 0.0) {
            Some(end) => {
                worked += (end - session.started_at).max(0.0);
                open_since = None;
            }
            None => open_since = Some(session.started_at),
        }
    }
    Some(Notes {
        steps,
        actions,
        files: files.len(),
        filed,
        started_at: first,
        ended_at,
        worked,
        open_since,
        failed,
        failure,
        sessions: ordered.len(),
        raw: ordered.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join("\n"),
    })
}

fn plural(n: usize, one: &str) -> String {
    format!("{n} {one}{}", if n == 1 { "" } else { "s" })
}

/// "5 minutes", "3 seconds", "1h 12m" (`spanLabel`). Never rounded up: 90 seconds is a minute.
pub fn span_label(ms: f64) -> String {
    let seconds = (ms / 1000.0 + 0.5).floor().max(0.0) as u64;
    if seconds < 60 {
        return plural(seconds as usize, "second");
    }
    let minutes = seconds / 60;
    if minutes < 60 {
        return plural(minutes as usize, "minute");
    }
    format!("{}h {}m", minutes / 60, minutes % 60)
}

/// How long the agents were awake on this thread (`workedFor`).
pub fn worked_for(notes: &Notes, now: f64) -> f64 {
    notes.worked + notes.open_since.filter(|s| *s != 0.0).map_or(0.0, |since| (now - since).max(0.0))
}

/// The one line the notes wear when closed (`closedLine`): what the runs did, or that they
/// did not finish. A run that narrated nothing counts what it can instead of saying "0 steps".
pub fn closed_line(notes: &Notes, now: f64) -> String {
    let span = span_label(worked_for(notes, now));
    if notes.failed {
        return format!("what happened · stopped after {span}");
    }
    match notes.steps.len() {
        0 => format!("what it did · {} in {span}, none of them said out loud", plural(notes.actions, "action")),
        count => format!("what it did · {} in {span}", plural(count, "step")),
    }
}

/// The line under the opened notes (`tallyLine`): only what was counted, and a zero not said.
pub fn tally_line(notes: &Notes, now: f64) -> String {
    let mut parts = vec![span_label(worked_for(notes, now))];
    if !notes.steps.is_empty() {
        parts.push(plural(notes.steps.len(), "step"));
    }
    // A run that never got a tool call away read nothing and changed nothing.
    parts.push(if notes.actions > 0 { plural(notes.actions, "action") } else { "nothing read, nothing changed".to_string() });
    if notes.files > 0 {
        parts.push(format!("{} written", plural(notes.files, "file")));
    }
    if notes.filed > 0 {
        parts.push(format!("{} filed", plural(notes.filed, "task")));
    }
    if notes.sessions > 1 {
        parts.push(plural(notes.sessions, "session"));
    }
    parts.join(" · ")
}

/// What the agent is doing right now, out of the live tail the supervisor streams (`liveStep`):
/// the newest of its own sentences, never a summarised tool call.
pub fn live_step(tail: &[String]) -> Option<String> {
    tail.iter().rev().map(|line| js::trim(line)).find(|line| !line.is_empty() && !NOT_A_STEP.test(line)).map(first_paragraph)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: f64 = 3_600_000.0;

    fn at(iso: &str) -> f64 {
        iso_ms(iso).unwrap()
    }

    /// The same moment on the person's clock, whatever zone the suite runs in.
    fn her_clock(iso: &str) -> String {
        hers(at(iso))
    }

    fn trace(started_at: f64, body: &[&str]) -> TraceSession {
        let spawned = LocalTime::at_offset(started_at as i64, 0);
        let iso = format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.000Z", spawned.year, spawned.month, spawned.day, spawned.hour, spawned.minute, spawned.second);
        TraceSession { started_at, text: format!("# a task\n# w-x · spawned {iso}\n\n{}", body.join("\n")) }
    }

    fn texts(notes: &Notes) -> Vec<&str> {
        notes.steps.iter().map(|s| s.text.as_str()).collect()
    }

    fn diary() -> Notes {
        read_notes(&[trace(at("2026-08-12T17:16:00Z"), &[
            "17:16:04  Took the job. Reading the scope document your answer was about.",
            "17:16:06  [mcp__agentbox__claim_work_item] w-6d21470b94",
            "17:16:31  [Read] /Users/you/Desktop/dev/zero/CLAUDE.md",
            "17:17:02  Checked two claims against the real code instead of trusting the session before it.",
            "17:17:14  [Bash] grep -rn skill main/ renderer/src",
            "17:18:41  [mcp__agentbox__create_work_item] agentbox",
            "17:18:52  [mcp__agentbox__create_work_item] agentbox",
            "17:19:03  [Write] /Users/you/Zero/agentbox/STATE.md",
            "17:20:11  Closed this row with the result above.",
            "",
            "17:20:12  == RESULT (success · 27 turns) ==",
            "Done. Your answer is enacted and the row is closed.",
            "",
            "# exited (0) 2026-08-12T17:20:14.000Z",
        ])])
        .unwrap()
    }

    #[test]
    fn the_diary_keeps_the_sentences_and_counts_the_actions() {
        let notes = diary();
        assert_eq!(texts(&notes), [
            "Took the job. Reading the scope document your answer was about.",
            "Checked two claims against the real code instead of trusting the session before it.",
            "Closed this row with the result above.",
        ]);
        assert_eq!(notes.actions, 6);
        // What it can prove and nothing else.
        assert_eq!((notes.filed, notes.files, notes.failed), (2, 1, false));
        // Each action under the step it happened during.
        assert_eq!(notes.steps.iter().map(|s| s.actions).collect::<Vec<_>>(), [2, 4, 0]);
    }

    #[test]
    fn the_clock_is_read_the_way_she_reads_it_out_of_a_trace_written_in_utc() {
        let notes = diary();
        assert_eq!(notes.steps[0].time, her_clock("2026-08-12T17:16:04Z"));
        assert!(!notes.steps[0].time.contains("17:16"));
        // A run that crosses midnight in UTC keeps going forwards.
        let overnight = read_notes(&[trace(at("2026-08-12T23:50:00Z"), &["23:55:00  Late.", "00:20:00  Later."])]).unwrap();
        assert_eq!(overnight.steps.iter().map(|s| s.time.clone()).collect::<Vec<_>>(), [her_clock("2026-08-12T23:55:00Z"), her_clock("2026-08-13T00:20:00Z")]);
    }

    #[test]
    fn how_long_it_took_is_measured_not_rounded_up() {
        let notes = diary();
        assert_eq!(closed_line(&notes, 0.0), "what it did · 3 steps in 4 minutes");
        assert_eq!(tally_line(&notes, 0.0), "4 minutes · 3 steps · 6 actions · 1 file written · 2 tasks filed");
    }

    #[test]
    fn the_sign_off_that_only_repeats_the_result_is_dropped() {
        let notes = read_notes(&[trace(at("2026-08-12T10:00:00Z"), &[
            "10:00:05  Starting on the frames.",
            "10:00:09  [Bash] git status",
            "10:02:00  **Built and on `zero/w-000e11c85b`**, unmerged.",
            "Some more of the same long message.",
            "",
            "10:02:01  == RESULT (success · 9 turns) ==",
            "**Built and on `zero/w-000e11c85b`**, unmerged.",
            "",
            "# exited (0) 2026-08-12T10:02:03.000Z",
        ])])
        .unwrap();
        assert_eq!(texts(&notes), ["Starting on the frames."]);
    }

    #[test]
    fn a_step_that_ran_onto_a_second_line_carries_it() {
        let notes = read_notes(&[trace(at("2026-08-12T10:00:00Z"), &[
            "10:00:05  Found it: the transform is applied in both places,",
            "so only one of them can keep writing it.",
            "10:00:09  [Edit] src/tilt.ts",
            "# exited (0) 2026-08-12T10:00:20.000Z",
        ])])
        .unwrap();
        assert_eq!(notes.steps[0].text, "Found it: the transform is applied in both places,\nso only one of them can keep writing it.");
    }

    #[test]
    fn a_run_that_was_turned_away_says_so_in_words() {
        let notes = read_notes(&[trace(at("2026-08-12T14:10:00Z"), &[
            "14:10:02  Started on your answer.",
            "",
            "14:10:03  == RESULT (success ERROR · 1 turns) ==",
            "API Error: Claude AI usage limit reached. Your weekly limit resets at 4pm.",
            "",
            "# exited (1) 2026-08-12T14:10:03.000Z",
        ])])
        .unwrap();
        assert!(notes.failed);
        let last = notes.steps.last().unwrap();
        assert!(last.failed);
        assert_eq!(last.text, "Claude turned the session away: the usage limit is used up, and resets at 4pm.");
        // It leads with what happened rather than what it did.
        assert_eq!(closed_line(&notes, 0.0), "what happened · stopped after 3 seconds");
        // And never softens a failure it does not recognise into a friendlier lie.
        assert_eq!(human_failure("API Error: 529 overloaded_error"), "529 overloaded_error");
        assert_eq!(human_failure(""), "The run stopped without saying why.");
    }

    #[test]
    fn a_run_ends_at_the_result_it_reported_when_nothing_wrote_its_exit_line() {
        let started = at("2026-08-14T19:39:13Z");
        let killed = read_notes(&[trace(started, &["19:39:20  Reading the branch.", "", "19:41:40  == RESULT (error_during_execution ERROR · 28 turns) =="])]).unwrap();
        assert_eq!(killed.open_since, None);
        assert_eq!(killed.ended_at, Some(at("2026-08-14T19:41:40Z")));
        assert_eq!(closed_line(&killed, started + 8.0 * HOUR), "what happened · stopped after 2 minutes");
    }

    #[test]
    fn every_run_is_one_diary_and_the_last_runs_fate_is_the_threads() {
        let start = at("2026-08-12T09:00:00Z");
        let notes = read_notes(&[
            trace(start + HOUR, &["10:00:01  Picking it back up after the limit reset.", "# exited (0) 2026-08-12T10:05:00.000Z"]),
            trace(start, &["09:00:01  Started on your answer.", "09:00:02  == RESULT (success ERROR · 1 turns) ==", "API Error: Claude AI usage limit reached.", "# exited (1) 2026-08-12T09:00:02.000Z"]),
        ])
        .unwrap();
        assert_eq!(notes.sessions, 2);
        assert!(!notes.failed);
        assert_eq!(texts(&notes), ["Started on your answer.", "Claude turned the session away: the usage limit is used up.", "Picking it back up after the limit reset."]);
    }

    #[test]
    fn no_run_has_nothing_to_draw() {
        assert_eq!(read_notes(&[]), None);
    }

    #[test]
    fn the_step_it_is_on_is_the_newest_thing_it_said_not_the_newest_thing_it_ran() {
        let tail = |lines: &[&str]| lines.iter().map(|l| l.to_string()).collect::<Vec<_>>();
        assert_eq!(live_step(&tail(&["tool: Read", "Found it: the transform is in both places.", "tool: Edit", "tool: Bash"])).as_deref(), Some("Found it: the transform is in both places."));
        assert_eq!(live_step(&tail(&["tool: Read", "tool: Grep"])), None);
        assert_eq!(live_step(&[]), None);
    }

    #[test]
    fn a_span_is_never_rounded_up() {
        assert_eq!(span_label(3_000.0), "3 seconds");
        assert_eq!(span_label(90_000.0), "1 minute");
        assert_eq!(span_label(5.0 * 60_000.0), "5 minutes");
        assert_eq!(span_label(72.0 * 60_000.0), "1h 12m");
    }

    #[test]
    fn time_worked_is_the_time_the_agents_were_awake_not_the_time_the_row_existed() {
        let start = at("2026-08-10T09:00:00Z");
        let notes = read_notes(&[
            TraceSession { started_at: start, text: "09:00:01  First pass.\n# exited (0) 2026-08-10T09:04:00.000Z".into() },
            TraceSession { started_at: start + 24.0 * HOUR, text: "09:00:01  Picked it back up a day later.\n# exited (0) 2026-08-11T09:06:00.000Z".into() },
        ])
        .unwrap();
        assert_eq!(closed_line(&notes, 0.0), "what it did · 2 steps in 10 minutes");
    }

    #[test]
    fn a_silent_run_says_what_it_did_rather_than_announcing_zero() {
        let notes = read_notes(&[TraceSession { started_at: at("2026-08-12T09:00:00Z"), text: ["09:00:01  [Bash] ls", "09:00:04  [Read] STATE.md", "# exited (0) 2026-08-12T09:03:00.000Z"].join("\n") }]).unwrap();
        assert!(notes.steps.is_empty());
        assert_eq!(closed_line(&notes, 0.0), "what it did · 2 actions in 3 minutes, none of them said out loud");
    }

    #[test]
    fn a_run_that_got_nowhere_read_nothing_rather_than_counting_zero() {
        let notes = read_notes(&[TraceSession {
            started_at: at("2026-08-12T14:10:00Z"),
            text: ["14:10:02  Started on your answer.", "14:10:03  == RESULT (success ERROR · 1 turns) ==", "API Error: Claude AI usage limit reached.", "# exited (1) 2026-08-12T14:10:03.000Z"].join("\n"),
        }])
        .unwrap();
        assert_eq!(tally_line(&notes, 0.0), "3 seconds · 2 steps · nothing read, nothing changed");
    }

    /// `tests/the-server-saying-not-your-usage-limit-is-not-a-limit.test.mts`: the servers
    /// turning a request away is never the person's plan, and a real limit still is one.
    #[test]
    fn the_server_saying_not_your_usage_limit_is_not_a_limit() {
        let busy = "Server is temporarily limiting requests (not your usage limit)";
        assert!(!human_failure(&format!("API Error: {busy}")).contains("used up"));
        assert!(human_failure("API Error: Claude AI usage limit reached.").contains("used up"));
    }

    #[test]
    fn a_log_written_at_the_start_s_own_second_stays_on_its_day() {
        // Spawned 97 ms into the second its only line names.
        let started = at("2026-10-04T18:30:28.097Z");
        assert_eq!(moment_of(started, 18, 30, 28), at("2026-10-04T18:30:28Z"));
        assert_eq!(moment_of(started, 18, 30, 27), at("2026-10-05T18:30:27Z"));
    }

    #[test]
    fn a_long_step_keeps_its_first_paragraph_and_ends_on_a_sentence() {
        assert_eq!(first_paragraph("One.\n\nTwo."), "One.");
        let long = format!("{}. {}", "a".repeat(150), "b".repeat(200));
        assert_eq!(first_paragraph(&long), format!("{}.…", "a".repeat(150)));
        let unbroken = "c".repeat(400);
        assert_eq!(first_paragraph(&unbroken), format!("{}…", "c".repeat(300)));
    }

    #[test]
    fn an_exit_line_s_moment_is_read_as_date_parse_reads_it() {
        assert_eq!(iso_ms("1970-01-01T00:00:01.500Z"), Some(1500.0));
        assert_eq!(iso_ms("2026-08-12T17:20:14+02:00"), Some(at("2026-08-12T15:20:14Z")));
        assert_eq!(iso_ms("stopped"), None);
        assert_eq!(iso_ms("2026-02-30T00:00:00Z"), None);
    }
}
