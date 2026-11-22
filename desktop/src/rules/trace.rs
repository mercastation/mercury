//! A run's log, as the lines the conversation draws: what the agent said, each tool it ran
//! with what came back, and, where the run stopped on the tool's own refusal, that it stopped,
//! in the app's words (`renderer/src/terminal.ts`).
//!
//! Beside it, the window's half of asking for logs (`traceReader` in `shared/trace-delta.mts`):
//! the engine is told the stamp of each log already held and answers only with what grew, and
//! this puts each log back together. And the join of a log cut to its tail onto the whole copy
//! read when a conversation's middle is asked for (`renderer/src/every-run.ts`).

use std::collections::HashMap;

use rune::time::LocalTime;
use serde_json::Value;

use super::js::{self, group, re};
use super::notes::{TraceSession, moment_of};

/// What one line of a log is.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Say,
    Tool,
}

/// One line a run typed (`TraceLine`).
#[derive(Clone, Debug, PartialEq)]
pub struct TraceLine {
    /// "9:12:04", on the person's clock.
    pub time: String,
    /// The same moment as a real one: the log's clock carries no date.
    pub at: f64,
    pub kind: Kind,
    pub text: String,
    /// A tool line's call id, and what came back from it when the log kept that.
    pub id: Option<String>,
    pub output: Option<String>,
    pub output_lines: Option<usize>,
    pub failed: bool,
}

re!(STAMPED, r"^(\d\d):(\d\d):(\d\d) {2}(.*)$", "");
re!(EXIT, r"^# exited \((-?\d+)\)", "");
re!(CALL_ID, r"^#id (\S+)$", "");
re!(OUTPUT_HEAD, r"^#out (\S+) (\d+)( error)?$", "");
re!(ERROR_RESULT, r"^\d{2}:\d{2}:\d{2}\s{2}== RESULT \([^)]*\bERROR\b[^)]*\) ==$", "m");
re!(EXITED_LINE, r"^# exited \(", "m");
re!(SPACES, r"\s+", "g");
// `[^\S\n]+$` in the web's: white space but a line break, spelled out as JavaScript's `\s` less `\n`.
re!(TRAILING, r"[\t\v\f\r    -     　﻿]+$", "gm");
re!(GAPS, r"\n{3,}", "g");

/// "9:12:04" (`hers` in `terminal.ts`).
fn hers(started_at: f64, h: u32, m: u32, s: u32) -> String {
    let t = LocalTime::at(moment_of(started_at, h, m, s) as i64);
    let hour = if t.hour.is_multiple_of(12) { 12 } else { t.hour % 12 };
    format!("{hour}:{:02}:{:02}", t.minute, t.second)
}

/// What a tool ran is one line: its words kept, its breaks lost, cut at 300 (`oneLine`).
fn one_line(text: &str) -> String {
    let clean = SPACES.replace_all(text, |_| " ".to_string());
    let clean = js::trim(&clean);
    if js::len(clean) <= 300 { clean.to_string() } else { format!("{}…", js::trim(js::slice(clean, 0, 300))) }
}

/// The body of the run's ERROR result, or nothing when it ended any other way (`errorResultText`).
fn error_result_text(raw: &str) -> String {
    let Some(at) = ERROR_RESULT.exec(raw) else { return String::new() };
    let after = &raw[at.span(0).map_or(raw.len(), |(_, end)| end)..];
    let end = EXITED_LINE.exec(after).and_then(|e| e.span(0)).map_or(after.len(), |(start, _)| start);
    js::trim(&after[..end]).to_string()
}

/// A message as the log keeps it: no spaces at a line's end, no run of blank lines (`said`).
pub fn said(text: &str) -> String {
    let text = TRAILING.strip_all(text);
    let text = GAPS.replace_all(&text, |_| "\n\n".to_string());
    js::trim(&text).to_string()
}

/// A message as `trace_lines` keeps it, which drops every line after the first that opens
/// with `#`, a "## Options" heading (`asTraced`).
pub fn as_traced(text: &str) -> String {
    let all = said(text);
    let mut lines = all.split('\n');
    let first = lines.next().unwrap_or("");
    let kept: Vec<&str> = std::iter::once(first).chain(lines.filter(|l| !l.starts_with('#'))).collect();
    said(&kept.join("\n"))
}

/// Every line one run typed, in the order it typed them (`traceLines`). Left out: the `#`
/// lines the supervisor writes, its stderr noise, and the result block, which is the answer
/// already on the page.
pub fn trace_lines(session: &TraceSession) -> Vec<TraceLine> {
    let mut lines: Vec<TraceLine> = Vec::new();
    let (mut in_result, mut exited, mut in_stderr) = (false, false, false);
    // Blank lines since the last line of prose, so a paragraph break survives.
    let mut blanks = 0;
    // The tool's own refusal, when the run ended on one: its prose copy is said in our words.
    let failure = error_result_text(&session.text);
    let failure_line = if failure.is_empty() { String::new() } else { said(failure.split('\n').next().unwrap_or("")) };
    // The tool line an `#out` block is being read into.
    let mut output_of: Option<usize> = None;
    let mut output_read: Vec<&str> = Vec::new();

    for raw in session.text.split('\n') {
        let Some(stamped) = STAMPED.exec(raw) else {
            if raw.starts_with("stderr:") {
                in_stderr = true;
                continue;
            }
            if in_stderr {
                if !js::trim(raw).is_empty() && !raw.starts_with('#') {
                    continue;
                }
                in_stderr = false;
                if js::trim(raw).is_empty() {
                    continue;
                }
            }
            if let Some(rest) = raw.strip_prefix("#| ") {
                if let Some(at) = output_of {
                    output_read.push(rest);
                    lines[at].output = Some(output_read.join("\n"));
                }
                continue;
            }
            if let Some(head) = OUTPUT_HEAD.exec(raw) {
                output_read.clear();
                output_of = lines.iter().rposition(|l| l.kind == Kind::Tool && l.id.as_deref() == head.get(1));
                if let Some(at) = output_of {
                    let line = &mut lines[at];
                    line.output = Some(String::new());
                    line.output_lines = group(&head, 2).parse().ok();
                    line.failed = head.get(3).is_some();
                }
                continue;
            }
            if let Some(id) = CALL_ID.exec(raw) {
                if let Some(call) = lines.last_mut().filter(|l| l.kind == Kind::Tool && l.id.is_none()) {
                    call.id = id.get(1).map(str::to_string);
                }
                continue;
            }
            if EXIT.test(raw) {
                exited = true;
                in_result = false;
                continue;
            }
            if raw.starts_with('#') || in_result || exited {
                continue;
            }
            // A sentence that ran onto another line continues the line above it, and a blank
            // line between two of them is a paragraph break written on purpose.
            let Some(last) = lines.last_mut().filter(|l| l.kind == Kind::Say) else { continue };
            if js::trim(raw).is_empty() {
                blanks += 1;
                continue;
            }
            last.text = said(&format!("{}{}{raw}", last.text, if blanks > 0 { "\n\n" } else { "\n" }));
            blanks = 0;
            continue;
        };
        blanks = 0;
        output_of = None;
        let body = group(&stamped, 4);
        let n = |i| group(&stamped, i).parse::<u32>().unwrap_or(0);
        let (h, m, s) = (n(1), n(2), n(3));
        let at = moment_of(session.started_at, h, m, s);
        let time = hers(session.started_at, h, m, s);
        in_stderr = body.starts_with("stderr:");
        if body.starts_with("== RESULT") {
            in_result = true;
            continue;
        }
        in_result = false;
        if in_stderr || js::trim(body).is_empty() {
            continue;
        }
        let kind = if body.starts_with('[') { Kind::Tool } else { Kind::Say };
        let text = if kind == Kind::Say && !failure_line.is_empty() && said(body) == failure_line {
            run_stopped_line(&failure)
        } else if kind == Kind::Tool {
            one_line(body)
        } else {
            said(body)
        };
        lines.push(TraceLine { time, at, kind, text, id: None, output: None, output_lines: None, failed: false });
    }
    lines
}

/* ------------------------- where a run stopped, in our words ------------------------- */

// The flags the app runs Claude Code with, which an older one does not know
// (`OUR_FLAGS` in `shared/spawn-trouble.mts`, read off `harnessDefinition('claude').run`).
re!(TOO_OLD_FLAG, r"^error: unknown option '(--output-format|--verbose|--include-partial-messages|--effort|--append-system-prompt|--resume|--fork-session|--input-format|--replay-user-messages|--allowedTools|--disallowedTools|--permission-mode|--mcp-config|--permission-prompt-tool|--settings|--model)'", "m");
re!(TOO_OLD_MODE, r"^error: option '--permission-mode <mode>' argument 'auto' is invalid", "m");
re!(ORG_BLOCKED, r"organi[sz]ation has disabled|disabled for (your|this) organi[sz]ation|ask your admin", "i");
re!(SUBSCRIPTION_ENDED, r"can['’]?t use your claude subscription", "i");
re!(API_KEY, r"signed in with an api key", "i");
re!(CODEX_CONFIG, r"refused config\/read: failed to (read configuration layers|resolve feature override)|error loading config\.toml|config\.toml:\d+:\d+: ", "i");
re!(SIGNED_OUT, r"oauth|session expired|could not be refreshed|failed to authenticate|not logged in|please run\s*\/?login|invalid api key|unauthoriz|subscription access|login expired|session has expired|sign in again to continue|\brun \/login\b", "i");
re!(LOGIN_BUSY, r"another claude code process is refreshing", "i");
re!(AT_LIMIT, r"usage limit|rate limit|quota|out of credit|insufficient credit|hit your [a-z]{0,12} ?limit|session limit|weekly limit", "i");
re!(AT_LIMIT_CODEX, r"\busageLimitExceeded\b", "");
re!(INTERRUPTED, r"went to sleep|connection (closed|lost|dropped)|response stopped arriving|may be incomplete|econnreset|enotfound|etimedout|econnrefused|enetunreach|ehostunreach|eai_again|error sending request for url|failed to send websocket request|aborted due to timeout|\bfetch failed\b|unable to connect|can.{0,3}t reach the api|timed out|overloaded", "i");
re!(INTERRUPTED_CODE, r"\b(429|500|502|503|529)\b", "");
re!(NETWORK, r"self[- ]signed certificate|unable to get local issuer certificate|unable to verify the first certificate|tls_untrusted_ca|invalid peer certificate|certificate verify failed|proxy authentication required", "i");
re!(NETWORK_CODE, r"\b(DEPTH_ZERO_SELF_SIGNED_CERT|SELF_SIGNED_CERT_IN_CHAIN|UNABLE_TO_GET_ISSUER_CERT_LOCALLY|UNABLE_TO_VERIFY_LEAF_SIGNATURE)\b", "");
re!(WORKSPACE, r"untrusted|trust this folder|not a git repos|not inside a trusted directory|skip-git-repo-check", "i");
re!(MODEL_NOT_ON_PLAN, r"model is not supported when using codex with a chatgpt account|not a model the plan of the codex account|not a model the codex on this computer knows|there's an issue with the selected model", "i");
re!(LEVEL_NOT_OFFERED, r"not a level that model offers", "i");
re!(OPEN_ELSEWHERE, r"already has an active writer", "i");

/// What stopped a run, named from the words it stopped with (`troubleCause` in
/// `shared/spawn-trouble.mts`). The order is the web's: each test runs before the ones whose
/// words it shares.
pub fn trouble_cause(text: &str) -> &'static str {
    if js::trim(text).is_empty() {
        return "unknown";
    }
    let checks: [(&dyn Fn() -> bool, &'static str); 15] = [
        (&|| TOO_OLD_FLAG.test(text) || TOO_OLD_MODE.test(text), "too-old"),
        (&|| CODEX_CONFIG.test(text), "codex-config"),
        (&|| API_KEY.test(text), "api-key"),
        (&|| ORG_BLOCKED.test(text), "org-blocked"),
        (&|| SUBSCRIPTION_ENDED.test(text), "subscription-ended"),
        (&|| MODEL_NOT_ON_PLAN.test(text), "model-not-on-plan"),
        (&|| LEVEL_NOT_OFFERED.test(text), "level-not-offered"),
        (&|| OPEN_ELSEWHERE.test(text), "open-elsewhere"),
        (&|| LOGIN_BUSY.test(text), "interrupted"),
        (&|| super::notes::server_busy(text), "interrupted"),
        (&|| SIGNED_OUT.test(text), "signed-out"),
        (&|| AT_LIMIT.test(text) || AT_LIMIT_CODEX.test(text), "at-limit"),
        (&|| NETWORK.test(text) || NETWORK_CODE.test(text), "network"),
        (&|| WORKSPACE.test(text), "workspace"),
        (&|| INTERRUPTED.test(text) || INTERRUPTED_CODE.test(text), "interrupted"),
    ];
    checks.iter().find(|(test, _)| test()).map_or("unknown", |(_, cause)| cause)
}

/// The line in a thread where a run stopped, in the app's words (`runStoppedLine`). It is about
/// the one run, so it says nothing of what was or was not done before it.
pub fn run_stopped_line(raw: &str) -> String {
    match trouble_cause(raw) {
        "signed-out" => "This run stopped because the account was signed out.",
        "org-blocked" => "This run stopped because Claude Code will not run on that account.",
        "subscription-ended" => "This run stopped because that account's Claude subscription has ended.",
        "at-limit" => "This run stopped because the account hit its usage limit.",
        "workspace" => "This run stopped because it would not start in this project's folder.",
        "interrupted" => "This run was cut off in the middle.",
        "model-not-on-plan" => "This run stopped because that account's plan does not include the model.",
        "level-not-offered" => "This run stopped because the model does not offer that thinking level.",
        "network" => "This run stopped because this network would not let it connect securely.",
        "too-old" => "This run stopped because Claude Code on this computer is older than Mercury needs.",
        "api-key" => "This run stopped because Codex is signed in with an API key.",
        "codex-config" => "This run stopped because Codex could not read its config.toml.",
        "open-elsewhere" => "This run stopped because the conversation is open in another Codex window.",
        _ => "This run stopped before it finished.",
    }
    .to_string()
}

/* --------------------------- a cloud run handing a thread back --------------------------- */

/// The line a cloud run writes when it hands its thread back to the Mac (`HAND_BACK` in
/// `shared/cloud-hand-back.mts`). It is the app's to read, never the person's.
const HAND_BACK: &str = "AGENTBOX-NEEDS-THE-COMPUTER:";

/// Whether a reply hands its thread back: the marked line, at a line's start (`handsBack`).
pub fn hands_back(text: &str) -> bool {
    text.split('\n').any(|line| line.trim_start_matches(js::is_space).starts_with(HAND_BACK))
}

/// A reply without its marked line, for drawing (`withoutHandBack`).
pub fn without_hand_back(text: &str) -> String {
    if !hands_back(text) {
        return text.to_string();
    }
    let kept: Vec<&str> = text.split('\n').filter(|line| !line.trim_start_matches(js::is_space).starts_with(HAND_BACK)).collect();
    js::trim(&kept.join("\n")).to_string()
}

/* ------------------------ a cut log joined onto its whole copy ------------------------ */

re!(CUT, r"^… truncated \((\d+) bytes total\)\n\n", "");

/// A log only grows, so a tail cut off it starts at a known place in the whole copy (`joined`).
fn joined(whole: &str, newest: &str) -> String {
    let Some(cut) = CUT.exec(newest) else { return newest.to_string() };
    let tail = &newest[cut.span(0).map_or(0, |(_, end)| end)..];
    let total: f64 = group(&cut, 1).parse().unwrap_or(0.0);
    let from = total - js::len(tail) as f64;
    if js::len(whole) as f64 >= from {
        // `slice` counts a place before the start from the end, as JavaScript's does.
        let end = if from < 0.0 { (js::len(whole) as f64 + from).max(0.0) } else { from };
        format!("{}{tail}", js::slice(whole, 0, end as usize))
    } else {
        newest.to_string()
    }
}

/// A thread's runs once its middle was asked for: every log read whole once, with the regular
/// read laid over it as it keeps arriving (`withEveryRun`).
pub fn with_every_run(every: &[TraceSession], newest: &[TraceSession]) -> Vec<TraceSession> {
    if every.is_empty() {
        return newest.to_vec();
    }
    let mut runs: Vec<TraceSession> = every.to_vec();
    for s in newest {
        match runs.iter_mut().find(|r| r.started_at == s.started_at) {
            Some(whole) => whole.text = joined(&whole.text, &s.text),
            None => runs.push(s.clone()),
        }
    }
    runs.sort_by(|a, b| a.started_at.total_cmp(&b.started_at));
    runs
}

/* ------------------------------ the window's half of the read ------------------------------ */

/// Most of a log a window draws, in characters (`CAP`).
const CAP: usize = 120_000;

/// One log held, as the last answer left it (`HeldLog`).
#[derive(Clone, Debug)]
struct Held {
    stamp: String,
    size: f64,
    raw: String,
    cut: bool,
    session: TraceSession,
}

fn size_of(stamp: &str) -> f64 {
    stamp.split(':').next().and_then(|n| n.parse().ok()).unwrap_or(f64::NAN)
}

fn time_of(stamp: &str) -> f64 {
    stamp.split(':').nth(1).and_then(|n| n.parse().ok()).unwrap_or(f64::NAN)
}

fn drawn(raw: &str, size: f64, cut: bool) -> String {
    if cut { format!("… truncated ({size} bytes total)\n\n{raw}") } else { raw.to_string() }
}

fn entry(started_at: f64, stamp: &str, raw: String, cut: bool) -> Held {
    let size = size_of(stamp);
    Held { stamp: stamp.to_string(), size, session: TraceSession { started_at, text: drawn(&raw, size, cut) }, raw, cut }
}

/// The log to hold after one answer about it, or None when the answer cannot be used (`settle`).
/// Logs only grow, so an answer behind what is held is an older read landing late, unless the
/// file was replaced, which its time says.
fn settle(log: Option<&Held>, s: &Value) -> Option<Held> {
    let started_at = s.get("startedAt").and_then(Value::as_f64).unwrap_or(0.0);
    let stamp = s.get("stamp").and_then(Value::as_str).unwrap_or("");
    let stale = log.is_some_and(|l| size_of(stamp) <= l.size && time_of(stamp) <= time_of(&l.stamp));
    if s.get("same").is_some_and(|v| v.as_bool() == Some(true)) {
        return log.cloned();
    }
    if stale {
        return log.cloned();
    }
    if let Some(more) = s.get("more").and_then(Value::as_str) {
        let log = log?;
        if s.get("from").and_then(Value::as_f64) != Some(log.size) {
            return None;
        }
        let all = format!("{}{more}", log.raw);
        let cut = log.cut || js::len(&all) > CAP;
        let kept = if cut { js::slice(&all, js::len(&all).saturating_sub(CAP), js::len(&all)).to_string() } else { all };
        return Some(entry(started_at, stamp, kept, cut));
    }
    let text = s.get("text").and_then(Value::as_str)?;
    let cut = s.get("cut").is_some_and(|v| v.as_bool() == Some(true));
    let raw = if cut { text.find("\n\n").map_or(text, |at| &text[at + 2..]).to_string() } else { text.to_string() };
    Some(entry(started_at, stamp, raw, cut))
}

/// What the window holds of each open thread's logs, so the engine sends only what grew
/// (`traceReader`). The newest few threads are kept.
#[derive(Default, Debug)]
pub struct Reader {
    /// By thread, oldest first: each log by its start, and the list last handed out.
    held: Vec<(String, HashMap<u64, Held>, Vec<TraceSession>)>,
}

/// How many threads' logs are held (`keep`).
const KEEP: usize = 4;

impl Reader {
    /// The stamps of the logs held for a thread, to send as `have`, or None to send nothing.
    pub fn have(&self, key: &str) -> Option<Value> {
        let (_, logs, _) = self.held.iter().find(|(k, _, _)| k == key)?;
        let mut have = serde_json::Map::new();
        for log in logs.values() {
            have.insert(js_number(log.session.started_at), Value::String(log.stamp.clone()));
        }
        Some(Value::Object(have))
    }

    fn forget(&mut self, key: &str) {
        self.held.retain(|(k, _, _)| k != key);
    }

    /// An answer laid over what is held: the logs whole again, or None when it cannot be put
    /// together and the read has to be asked again with no stamps. An answer without stamps is
    /// used as it came and nothing of it is held.
    pub fn take(&mut self, key: &str, answer: &Value) -> Option<Vec<TraceSession>> {
        let Some(answered) = answer.get("sessions").and_then(Value::as_array) else {
            return Some(Vec::new());
        };
        if !answered.iter().all(|s| s.get("stamp").and_then(Value::as_str).is_some_and(|t| !t.is_empty())) {
            self.forget(key);
            return Some(answered.iter().map(|s| serde_json::from_value(s.clone()).unwrap_or_default()).collect());
        }
        let before = self.held.iter().position(|(k, _, _)| k == key).map(|at| self.held.remove(at));
        let mut logs: HashMap<u64, Held> = HashMap::new();
        let mut order: Vec<u64> = Vec::new();
        for s in answered {
            let started_at = s.get("startedAt").and_then(Value::as_f64).unwrap_or(0.0);
            let had = before.as_ref().and_then(|(_, logs, _)| logs.get(&started_at.to_bits()));
            let log = settle(had, s)?;
            if logs.insert(started_at.to_bits(), log).is_none() {
                order.push(started_at.to_bits());
            }
        }
        let sessions: Vec<TraceSession> = order.iter().map(|at| logs[at].session.clone()).collect();
        // Nothing changed: the same list goes back.
        let list = match before {
            Some((_, _, was)) if was == sessions => was,
            _ => sessions,
        };
        self.held.push((key.to_string(), logs, list.clone()));
        while self.held.len() > KEEP {
            self.held.remove(0);
        }
        Some(list)
    }
}

/// A number as JavaScript writes it as an object's key: a whole number without a point.
fn js_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e21 { format!("{}", n as i64) } else { format!("{n}") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn at(iso: &str) -> f64 {
        super::super::notes::iso_ms(iso).unwrap()
    }

    fn run(started_at: f64, text: &str) -> TraceSession {
        TraceSession { started_at, text: text.to_string() }
    }

    /// `tests/what-the-agent-said-is-not-cut-off.test.mts`.
    #[test]
    fn the_message_she_reads_is_the_message_it_wrote() {
        let started = at("2026-08-24T02:41:00Z");
        let text = [
            "02:49:52  Merged and closed.",
            "",
            "**What landed.** Main is now `fd7df45`. `resolveTheme` takes one argument and returns dark when nothing is stored, a first run writes the look down instead of only painting it, and `prefers-color-scheme` and `systemPrefersDark` appear nowhere in `renderer/`, `main/` or `shared/`.",
            "",
            "**What I actually did rather than merging it as built.** Main had moved since the branch was cut, so I merged main into the branch in a scratch worktree and re-verified there.",
            "",
            "**What I did not do.** No rebuild, no package, no release.",
        ]
        .join("\n");
        let lines = trace_lines(&run(started, &text));
        let message = &lines[0];
        assert_eq!(message.kind, Kind::Say);
        assert!(js::len(&message.text) > 300);
        assert!(message.text.ends_with("No rebuild, no package, no release."));
        assert!(!message.text.contains('…'));
        assert_eq!(message.text.split("\n\n").count(), 4);
        assert!(message.text.starts_with("Merged and closed.\n\n**What landed.**"));
        // The cap stays on what it ran, which is an argument and not prose.
        let tool = trace_lines(&run(started, &format!("02:49:52  [Bash] echo {}", "x".repeat(400))));
        assert_eq!(tool[0].kind, Kind::Tool);
        assert!(tool[0].text.ends_with('…'));
        assert_eq!(js::len(&tool[0].text), 301);
    }

    /// `tests/a-command-a-thread-ran-opens-onto-what-it-printed.test.mts`, with each log written
    /// as `traceStreamLine` writes it.
    #[test]
    fn what_a_command_printed_is_on_its_line() {
        let started = at("2026-11-04T00:00:00Z");
        let tools = |text: &str| trace_lines(&run(started, text)).into_iter().filter(|l| l.kind == Kind::Tool).collect::<Vec<_>>();
        let one = tools("00:00:01  [Bash] npm test\n#id t1\n#out t1 3\n#| ok 1 isEven\n#| # pass 10\n#| # fail 0");
        assert_eq!(one[0].output.as_deref(), Some("ok 1 isEven\n# pass 10\n# fail 0"));
        assert_eq!((one[0].output_lines, one[0].failed), (Some(3), false));
        // Blank lines are kept, the first one too.
        let blank = tools("00:00:01  [Bash] npx vitest run\n#id t1\n#out t1 4\n#| \n#|  RUN  v3\n#| \n#|  ✓ a.test.ts");
        assert_eq!(blank[0].output.as_deref(), Some("\n RUN  v3\n\n ✓ a.test.ts"));
        // A failed command says it failed.
        let failed = tools("00:00:01  [Bash] npm test\n#id t1\n#out t1 2 error\n#| Exit code 1\n#| not ok 2");
        assert!(failed[0].failed);
        // Two calls in one message each get their own.
        let both = tools("00:00:01  [Bash] ls\n#id a\n00:00:01  [Bash] pwd\n#id b\n#out b 1\n#| /repo\n#out a 2\n#| src\n#| test");
        assert_eq!(both.iter().map(|l| l.output.clone().unwrap_or_default()).collect::<Vec<_>>(), ["src\ntest", "/repo"]);
        // What came back is never something the agent said, and never joins a call it does not answer.
        let said: Vec<String> = trace_lines(&run(started, "00:00:01  Running the tests.\n00:00:02  [Bash] npm test\n#id t1\n#out t1 1\n#| # pass 10\n00:00:03  All pass."))
            .into_iter()
            .filter(|l| l.kind == Kind::Say)
            .map(|l| l.text)
            .collect();
        assert_eq!(said, ["Running the tests.", "All pass."]);
        let stray = tools("00:00:01  [Bash] npm test\n#id t1\n#out other 1\n#| stray");
        assert_eq!(stray[0].output, None);
        // An older log with no output still reads.
        let kinds: Vec<Kind> = trace_lines(&run(started, "00:00:01  [Bash] npm test\n00:00:02  Done.")).iter().map(|l| l.kind).collect();
        assert_eq!(kinds, [Kind::Tool, Kind::Say]);
    }

    /// `tests/a-stopped-run-does-not-jump-to-tomorrow.test.mts`.
    #[test]
    fn a_run_stopped_on_a_refusal_says_so_in_our_words() {
        let spawned = at("2026-10-04T18:30:28.097Z");
        let words = "Failed to authenticate: OAuth session expired and could not be refreshed";
        let dead = [
            "# w-test · spawned 2026-10-04T18:30:28.097Z · continuation (the founder answered)",
            "",
            &format!("18:30:28  {words}"),
            "",
            "18:30:28  == RESULT (success ERROR · 1 turns) ==",
            words,
            "",
            "# exited (1) 2026-10-04T18:30:29.848Z",
        ]
        .join("\n");
        let lines = trace_lines(&run(spawned, &dead));
        let text = lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join(" ");
        assert!(!text.to_lowercase().contains("oauth") && !text.to_lowercase().contains("refresh"));
        assert!(text.to_lowercase().contains("signed out"));
        assert_eq!(lines[0].at, at("2026-10-04T18:30:28Z"));
        // The agent's own words that merely mention signing in are left alone.
        let prose = "18:30:28  The login page now says Failed to authenticate when the password is wrong.\n\n18:31:00  == RESULT (success · 3 turns) ==\nDone.\n# exited (0) 2026-10-04T18:31:01.000Z";
        assert_eq!(trace_lines(&run(spawned, prose))[0].text, "The login page now says Failed to authenticate when the password is wrong.");
    }

    /// `tests/the-server-saying-not-your-usage-limit-is-not-a-limit.test.mts`.
    #[test]
    fn the_server_turning_a_request_away_is_not_the_plan_running_out() {
        let busy = "Server is temporarily limiting requests (not your usage limit)";
        assert_eq!(trouble_cause(busy), "interrupted");
        assert_eq!(trouble_cause(&format!("API Error: {busy}")), "interrupted");
        assert_eq!(trouble_cause(&format!("API Error: {busy} · If it persists, check https://status.claude.com.")), "interrupted");
        assert_eq!(trouble_cause("Claude is temporarily limiting requests to the usage page (not your usage limit). Try again in a moment"), "interrupted");
        assert!(!run_stopped_line(&format!("API Error: {busy}")).contains("usage limit"));
        assert_eq!(trouble_cause("You've hit your session limit · resets 6pm (America/Los_Angeles)"), "at-limit");
        assert_eq!(trouble_cause("API Error: Claude AI usage limit reached."), "at-limit");
        assert_eq!(trouble_cause("usage limit reached — check plan"), "at-limit");
    }

    #[test]
    fn a_trace_keeps_the_paragraphs_and_drops_the_noise() {
        let text = "# a task\n\n10:00:01  First line,\nsecond line.\n\n\n\nA new paragraph.\n10:00:02  stderr: warning\nmore noise\n10:00:03  [Read] notes.md\n10:00:04  == RESULT (success · 2 turns) ==\nThe answer.\n# exited (0) 2026-08-12T10:00:05.000Z\n10:00:06  after the exit";
        let lines = trace_lines(&run(at("2026-08-12T10:00:00Z"), text));
        assert_eq!(lines.iter().map(|l| (l.kind, l.text.as_str())).collect::<Vec<_>>(), [(Kind::Say, "First line,\nsecond line.\n\nA new paragraph."), (Kind::Tool, "[Read] notes.md"), (Kind::Say, "after the exit")]);
        assert_eq!(as_traced("**Found it.**\n\n## Options\n1. Ship it"), "**Found it.**\n\n1. Ship it");
        assert_eq!(said("one  \ntwo\n\n\n\nthree "), "one\ntwo\n\nthree");
    }

    #[test]
    fn a_hand_back_line_is_the_apps_to_read() {
        assert!(hands_back("Done.\n  AGENTBOX-NEEDS-THE-COMPUTER: the simulator"));
        assert!(!hands_back("It said AGENTBOX-NEEDS-THE-COMPUTER: inline"));
        assert_eq!(without_hand_back("Done.\nAGENTBOX-NEEDS-THE-COMPUTER: the simulator\n"), "Done.");
    }

    /// `tests/a-long-thread-shows-every-run-when-you-open-the-middle.test.mts`: a cut tail is
    /// joined onto the whole copy at the place it was cut from.
    #[test]
    fn a_cut_tail_joins_its_whole_copy() {
        let whole = "0123456789";
        assert_eq!(joined(whole, "… truncated (12 bytes total)\n\n6789ab"), "0123456789ab");
        assert_eq!(joined("012", "… truncated (12 bytes total)\n\n6789ab"), "… truncated (12 bytes total)\n\n6789ab");
        let every = [run(1.0, "one whole"), run(2.0, "two")];
        let newest = [run(2.0, "two, grown"), run(3.0, "three")];
        assert_eq!(with_every_run(&every, &newest).iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), ["one whole", "two, grown", "three"]);
    }

    /// `tests/an-open-thread-is-sent-only-the-agent-logs-that-changed.test.mts`, the window's half.
    #[test]
    fn an_open_thread_is_sent_only_the_logs_that_changed() {
        let mut reader = Reader::default();
        assert_eq!(reader.have("p/w"), None);
        let first = json!({ "ok": true, "sessions": [{ "startedAt": 1000, "stamp": "5:1", "text": "hello" }, { "startedAt": 2000, "stamp": "3:1", "text": "abc" }] });
        let got = reader.take("p/w", &first).unwrap();
        assert_eq!(got.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), ["hello", "abc"]);
        assert_eq!(reader.have("p/w"), Some(json!({ "1000": "5:1", "2000": "3:1" })));
        // One the same, one grown.
        let next = json!({ "ok": true, "sessions": [{ "startedAt": 1000, "stamp": "5:1", "same": true }, { "startedAt": 2000, "stamp": "6:2", "from": 3, "more": "def" }] });
        let got = reader.take("p/w", &next).unwrap();
        assert_eq!(got.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), ["hello", "abcdef"]);
        // A read landing late, behind what is held, changes nothing.
        let late = json!({ "ok": true, "sessions": [{ "startedAt": 1000, "stamp": "5:1", "same": true }, { "startedAt": 2000, "stamp": "3:1", "text": "abc" }] });
        assert_eq!(reader.take("p/w", &late).unwrap()[1].text, "abcdef");
        // Growth from a place this window is not at cannot be put together: what was held is
        // let go, and the read is asked again with no stamps.
        let lost = json!({ "ok": true, "sessions": [{ "startedAt": 2000, "stamp": "9:3", "from": 4, "more": "zzz" }] });
        assert_eq!(reader.take("p/w", &lost), None);
        assert_eq!(reader.have("p/w"), None);
        // An answer with no stamps is used as it came, and nothing of it is held.
        let plain = json!({ "ok": true, "sessions": [{ "startedAt": 1000, "text": "" }] });
        assert_eq!(reader.take("p/w", &plain).unwrap().len(), 1);
        assert_eq!(reader.have("p/w"), None);
    }

    #[test]
    fn a_log_cut_on_the_engine_is_held_without_its_cut_line_and_handed_out_with_it() {
        let mut reader = Reader::default();
        let cut = json!({ "ok": true, "sessions": [{ "startedAt": 1, "stamp": "500000:1", "cut": true, "text": "… truncated (500000 bytes total)\n\nthe tail" }] });
        assert_eq!(reader.take("k", &cut).unwrap()[0].text, "… truncated (500000 bytes total)\n\nthe tail");
        let grown = json!({ "ok": true, "sessions": [{ "startedAt": 1, "stamp": "500004:2", "from": 500000, "more": " end" }] });
        assert_eq!(reader.take("k", &grown).unwrap()[0].text, "… truncated (500004 bytes total)\n\nthe tail end");
    }
}
