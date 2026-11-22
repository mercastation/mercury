//! A thread an agent is on right now: the line at the conversation's foot that says what it is
//! doing and for how long (`renderer/src/live-line.ts`, drawn by `components/Live.tsx`), and the
//! line under the title while it works (`workingContext` in `renderer/src/one-thing-at-a-time.ts`).
//!
//! Every fact comes off the snapshot's running session (`supervisor.running`): when it started,
//! the tool calls still running (`activity`), the sentence being typed (`saying`), where it runs.
//! The web's line has eleven other states (queued, signed out, at its limit, paused, nothing
//! came back…), each read off the supervisor's own lists. Only the working one is drawn here.

use serde::Deserialize;

use super::js::{self, re};

/// One tool call still running (`activity` on `RunningSession`).
#[derive(Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Activity {
    pub id: String,
    /// "Running the tests".
    pub label: String,
    /// The whole command.
    pub detail: String,
    pub started_at: Option<f64>,
}

/// What the snapshot says of a run that is up (`RunningSession`).
#[derive(Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Session {
    pub item_id: String,
    pub product: String,
    pub started_at: f64,
    /// When the reply a cloud run is working on went in.
    pub turn_started_at: Option<f64>,
    pub activity: Vec<Activity>,
    /// The prose being typed this second, never on disk.
    pub saying: Option<String>,
    pub saying_at: Option<f64>,
    pub helpers: Option<u32>,
    /// `cloud` while the run is in a cloud sandbox.
    #[serde(rename = "where")]
    pub place: Option<String>,
}

/// How long a run has been up, never a seconds counter (`shortSpan`): under a minute it is
/// "just now", and it is floored, so it never says more time has passed than has.
pub fn short_span(ms: f64) -> String {
    let seconds = (ms / 1000.0).floor().max(0.0) as u64;
    if seconds < 60 {
        return "just now".to_string();
    }
    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("{minutes}m");
    }
    format!("{}h {}m", minutes / 60, minutes % 60)
}

/// When the work on screen began: the reply a cloud run took between turns, else the run's
/// start (`runningSince`).
pub fn running_since(session: &Session) -> f64 {
    session.turn_started_at.filter(|t| *t != 0.0).unwrap_or(session.started_at)
}

/// The line at the foot of a working thread, as `Live.tsx` draws it.
#[derive(Clone, Debug, PartialEq)]
pub struct Working {
    /// What is running now, by its own label, or "Thinking".
    pub word: String,
    /// "+2": the other calls running beside it.
    pub more: Option<String>,
    /// "4m".
    pub span: String,
    /// "in the cloud", said only where it is news.
    pub place: Option<String>,
}

/// The working line, or None where the web window draws something else or nothing: a finished
/// thread, one whose agent waits on a yes (`asking`), a practice thread (`liveLine`).
pub fn working(session: Option<&Session>, status: &str, labels: &[String], asking: bool, why_not_cloud_at: Option<&str>, now: f64) -> Option<Working> {
    let session = session?;
    // The walkthrough's practice rows are answered by the app, and drawn so until it answers.
    if labels.iter().any(|l| l == "first-run") || status == "done" || asking {
        return None;
    }
    let current = session.activity.last();
    let word = current.map(|a| a.label.clone()).filter(|l| !l.is_empty()).unwrap_or_else(|| "Thinking".to_string());
    let more = (session.activity.len() > 1).then(|| format!("+{}", session.activity.len() - 1));
    // Where it runs is said only where it is news: in the cloud, or on this Mac while the
    // cloud takes everything, the one time the supervisor gives a reason (`livePlace`).
    let place = if session.place.as_deref() == Some("cloud") {
        Some("in the cloud".to_string())
    } else if why_not_cloud_at == Some("mac") {
        Some("on this Mac".to_string())
    } else {
        None
    };
    Some(Working { word, more, span: short_span(now - running_since(session)), place })
}

/// How many of the steps a run finished since it last said anything the unfolded line lists;
/// the rest are counted (`LIVE_STEPS_SHOWN` in `renderer/src/activity-summary.ts`).
pub const STEPS_SHOWN: usize = 5;

/// One step a run took since it last said anything, as its log has it: when, its verb, what it
/// acted on, and the whole command or path behind that when the words shortened it.
#[derive(Clone, Debug, PartialEq)]
pub struct Took {
    pub at: f64,
    pub verb: String,
    pub subject: String,
    pub full: String,
}

/// A row of what the line unfolds onto (`ActivityRow`): its words, and the command it opens onto.
#[derive(Clone, Debug, PartialEq)]
pub struct StepRow {
    pub label: String,
    pub detail: String,
}

/// What the working line unfolds onto (`Live.tsx`, expanded): how many finished steps came
/// before the newest it lists, and the rows, the finished ones oldest first and then each call
/// running now.
#[derive(Clone, Debug, PartialEq)]
pub struct Steps {
    pub earlier: usize,
    pub rows: Vec<StepRow>,
}

/// The steps of `history` not still running: each call in `running` takes the newest step
/// with its command out of the list, unless that step is older than the call by more than a
/// second (`previousActivity`).
pub fn previous_activity(history: &[Took], running: &[Activity]) -> Vec<Took> {
    let normal = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut previous = history.to_vec();
    for action in running {
        let found = (0..previous.len()).rev().find(|&at| {
            let row = &previous[at];
            let too_old = action.started_at.is_some_and(|started| row.at < started - 1000.0);
            let said = if row.full.is_empty() { &row.subject } else { &row.full };
            !too_old && normal(said) == normal(&action.detail)
        });
        if let Some(at) = found {
            previous.remove(at);
        }
    }
    previous
}

/// What the line unfolds onto, from the steps the run took since it last said anything and the
/// calls running now: the newest five finished, oldest at the top, then what runs now.
pub fn steps(history: &[Took], running: &[Activity]) -> Steps {
    let finished = previous_activity(history, running);
    let recent = &finished[finished.len().saturating_sub(STEPS_SHOWN)..];
    let mut rows: Vec<StepRow> = recent
        .iter()
        .map(|took| {
            let mut verb = took.verb.chars();
            let capital = verb.next().map(|c| c.to_uppercase().collect::<String>() + verb.as_str()).unwrap_or_default();
            StepRow { label: format!("{capital} {}", took.subject).trim().to_string(), detail: if took.full.is_empty() { took.subject.clone() } else { took.full.clone() } }
        })
        .collect();
    rows.extend(running.iter().map(|a| StepRow { label: a.label.clone(), detail: a.detail.clone() }));
    Steps { earlier: finished.len() - recent.len(), rows }
}

/// The line over the rows for the finished steps not listed.
pub fn earlier_steps(earlier: usize) -> String {
    format!("{earlier} earlier {} since it last said something", if earlier == 1 { "step" } else { "steps" })
}

/// The words every run of a repeating task is briefed with, as the person wrote them
/// (`instructionOf` in `shared/repeats.mts`).
fn instruction_of(body: &str) -> &str {
    const OPENS: &str = "THIS IS ONE RUN OF A REPEATING TASK.";
    const MARK: &str = "HER INSTRUCTION FOLLOWS.";
    match body.find(&format!("\n{MARK}\n")) {
        Some(at) if body.starts_with(OPENS) => js::trim(&body[at + MARK.len() + 2..]),
        _ => body,
    }
}

re!(PARAGRAPHS, r"\n\s*\n", "");
re!(SPACES, r"\s+", "g");
re!(ENDS_A_SENTENCE, r"[.!?]$", "");

/// The line under the title while an agent works: the first paragraph of what was asked,
/// unless the title already says it (`askLine`).
pub fn ask_line(ask: &str, title: &str) -> Option<String> {
    let flat = |s: &str| js::trim(&SPACES.replace_all(s, |_| " ".to_string())).to_string();
    let first = flat(PARAGRAPHS.split(js::trim(instruction_of(ask)))[0]);
    let named = flat(title).to_lowercase();
    let lower = first.to_lowercase();
    // A repeating run's title is its ask with the day after it.
    if first.is_empty() || named == lower || named.starts_with(&format!("{lower} · ")) {
        return None;
    }
    Some(format!("You asked: {}", if ENDS_A_SENTENCE.test(&first) { first } else { format!("{first}.") }))
}

/// The line under a working thread's title: the ask, and its schedule if it repeats
/// (`workingContext`). This window keeps no repeating tasks, so there is no schedule to say.
pub fn working_context(ask: &str, title: &str) -> Option<String> {
    ask_line(ask, title)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: f64 = 1_800_000_000_000.0;

    fn session(started: f64) -> Session {
        Session { item_id: "w-1".into(), product: "p".into(), started_at: started, ..Session::default() }
    }

    /// `tests/is-anything-happening.test.mts`.
    #[test]
    fn a_span_is_never_a_seconds_counter() {
        assert_eq!(short_span(0.0), "just now");
        assert_eq!(short_span(41_000.0), "just now");
        assert_eq!(short_span(59_999.0), "just now");
        assert_eq!(short_span(60_000.0), "1m");
        assert_eq!(short_span(4.0 * 60_000.0), "4m");
        assert_eq!(short_span(59.0 * 60_000.0), "59m");
        assert_eq!(short_span(2.0 * 3_600_000.0 + 7.0 * 60_000.0), "2h 7m");
        assert_eq!(short_span(-5_000.0), "just now");
    }

    #[test]
    fn a_working_thread_says_what_runs_now() {
        let mut s = session(NOW - 4.0 * 60_000.0);
        let line = working(Some(&s), "claimed", &[], false, None, NOW).unwrap();
        assert_eq!(line, Working { word: "Thinking".into(), more: None, span: "4m".into(), place: None });
        s.activity = vec![
            Activity { id: "a".into(), label: "Reading notes.md".into(), detail: "notes.md".into(), started_at: None },
            Activity { id: "b".into(), label: "Running the tests".into(), detail: "npm test".into(), started_at: None },
        ];
        s.place = Some("cloud".into());
        let line = working(Some(&s), "claimed", &[], false, None, NOW).unwrap();
        assert_eq!((line.word.as_str(), line.more.as_deref(), line.place.as_deref()), ("Running the tests", Some("+1"), Some("in the cloud")));
        // A cloud run's reply is timed from the reply.
        s.turn_started_at = Some(NOW - 30_000.0);
        assert_eq!(working(Some(&s), "claimed", &[], false, None, NOW).unwrap().span, "just now");
        // Finished is finished, and a yes waited on outranks the run behind it.
        assert_eq!(working(Some(&s), "done", &[], false, None, NOW), None);
        assert_eq!(working(Some(&s), "claimed", &[], true, None, NOW), None);
        assert_eq!(working(None, "claimed", &[], false, None, NOW), None);
    }

    #[test]
    fn a_local_run_says_where_only_when_the_cloud_takes_everything() {
        let s = session(NOW);
        assert_eq!(working(Some(&s), "claimed", &[], false, Some("mac"), NOW).unwrap().place.as_deref(), Some("on this Mac"));
        assert_eq!(working(Some(&s), "claimed", &[], false, Some("cloud"), NOW).unwrap().place, None);
    }

    /// `tests/a-repeating-run-reads-as-your-words-on-the-day-it-ran.test.mts`.
    #[test]
    fn the_line_under_a_working_title_is_what_was_asked() {
        assert_eq!(ask_line("Fix the login button on the settings page", "Fix the login").as_deref(), Some("You asked: Fix the login button on the settings page."));
        assert_eq!(ask_line("Checkout fails.\n\nFind out why.", "Checkout").as_deref(), Some("You asked: Checkout fails."));
        assert_eq!(ask_line("Weekly news", "Weekly news"), None);
        assert_eq!(ask_line("Weekly news", "Weekly news · Thu, Oct 29"), None);
        let repeating = "THIS IS ONE RUN OF A REPEATING TASK.\nbrief\nHER INSTRUCTION FOLLOWS.\nSend the weekly news";
        assert_eq!(ask_line(repeating, "Weekly news").as_deref(), Some("You asked: Send the weekly news."));
        assert_eq!(ask_line("", "x"), None);
    }

    fn took(at: f64, verb: &str, subject: &str, full: &str) -> Took {
        Took { at, verb: verb.into(), subject: subject.into(), full: full.into() }
    }

    fn call(label: &str, detail: &str, started: Option<f64>) -> Activity {
        Activity { id: label.into(), label: label.into(), detail: detail.into(), started_at: started }
    }

    /// `previousActivity`, with `tests/thinking-expands-to-clean-gray-commands.test.mts`'s case.
    #[test]
    fn a_call_still_running_takes_its_own_step_out_of_the_finished_ones() {
        let rows = [took(0.0, "", "npm test", ""), took(0.0, "", "git status", ""), took(0.0, "", "npm test", "")];
        assert_eq!(previous_activity(&rows, &[call("", "npm test", None)]), rows[..2].to_vec(), "the current command is not called completed, nor an earlier repeat erased");
        assert_eq!(previous_activity(&rows, &[]), rows.to_vec());
        assert_eq!(previous_activity(&rows, &[call("", "npm build", None)]), rows.to_vec());
        let history = [took(1_000.0, "edited", "src/tax.ts", ""), took(2_000.0, "ran", "npm test", "npm  test\n")];
        // The running call has the same command, whitespace aside: it is that step, still going.
        let left = previous_activity(&history, &[call("Running the tests", "npm test", Some(2_500.0))]);
        assert_eq!(left, vec![history[0].clone()]);
        // A step more than a second older than the call is another run of it, and stays.
        let left = previous_activity(&history, &[call("Running the tests", "npm test", Some(9_000.0))]);
        assert_eq!(left.len(), 2);
        // A call with no start still takes its newest match.
        assert_eq!(previous_activity(&history, &[call("Running the tests", "npm test", None)]).len(), 1);
    }

    #[test]
    fn the_unfolded_line_lists_the_newest_five_finished_and_then_what_runs_now() {
        let history: Vec<Took> = (0..8).map(|k| took(f64::from(k) * 1000.0, "read", &format!("file-{k}.md"), &format!("/repo/file-{k}.md"))).collect();
        let shown = steps(&history, &[call("Running the tests", "npm test -- checkout", Some(10_000.0))]);
        assert_eq!(shown.earlier, 3);
        assert_eq!(shown.rows.len(), 6);
        assert_eq!(shown.rows[0], StepRow { label: "Read file-3.md".into(), detail: "/repo/file-3.md".into() });
        assert_eq!(shown.rows[5], StepRow { label: "Running the tests".into(), detail: "npm test -- checkout".into() });
        assert_eq!(earlier_steps(3), "3 earlier steps since it last said something");
        assert_eq!(earlier_steps(1), "1 earlier step since it last said something");
        // A step whose words are its whole command opens onto them.
        assert_eq!(steps(&[took(0.0, "ran", "git status", "")], &[]).rows[0].detail, "git status");
        assert_eq!(steps(&[], &[]), Steps { earlier: 0, rows: vec![] });
    }

    #[test]
    fn a_running_session_is_read_off_the_snapshot() {
        let run = serde_json::json!({ "itemId": "w-1", "product": "p", "startedAt": 5.0, "where": "cloud", "activity": [{ "id": "a", "label": "Reading", "detail": "x", "startedAt": 4.0 }], "tail": [] });
        let s: Session = serde_json::from_value(run).unwrap();
        assert_eq!((s.started_at, s.place.as_deref(), s.activity.len()), (5.0, Some("cloud"), 1));
    }
}
