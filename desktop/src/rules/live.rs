//! A thread an agent is on right now: the line at the conversation's foot that says what it is
//! doing and for how long (`renderer/src/live-line.ts`, drawn by `components/Live.tsx`), and the
//! line under the title while it works (`workingContext` in `renderer/src/one-thing-at-a-time.ts`).
//!
//! Every fact comes off the snapshot's running session (`supervisor.running`): when it started,
//! the tool calls still running (`activity`), the sentence being typed (`saying`), where it runs.
//! The web's line has eleven other states (queued, signed out, at its limit, paused, nothing
//! came back…), each read off the supervisor's own lists. Only the working one is drawn here.

use serde::Deserialize;
use serde_json::Value;

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

impl Session {
    /// The run on a thread, off the snapshot's running list.
    pub fn of(running: &[Value], id: &str, product: &str) -> Option<Session> {
        running.iter().filter_map(|r| serde_json::from_value::<Session>(r.clone()).ok()).find(|s| s.item_id == id && s.product == product)
    }
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

    #[test]
    fn a_running_session_is_read_off_the_snapshot() {
        let running = vec![serde_json::json!({ "itemId": "w-1", "product": "p", "startedAt": 5.0, "where": "cloud", "activity": [{ "id": "a", "label": "Reading", "detail": "x", "startedAt": 4.0 }], "tail": [] })];
        let s = Session::of(&running, "w-1", "p").unwrap();
        assert_eq!((s.started_at, s.place.as_deref(), s.activity.len()), (5.0, Some("cloud"), 1));
        assert_eq!(Session::of(&running, "w-2", "p"), None);
    }
}
