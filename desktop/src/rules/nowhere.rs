//! The app's own word that a run got nowhere, and the row that says tasks have not been able
//! to run. The sentences are `shared/spawn-trouble.mts`'s, read back here the way its
//! `saysARunGotNowhere` reads them; the row is `renderer/src/trouble-row.ts`.

use std::sync::LazyLock;

use serde_json::{Map, Value};

use super::js::{self, group, re};

// The app's name, as `shared/product-name.mjs` spells it at the start of a sentence.
const NAME: &str = "Mercury";

/// One opener per cause, for a run that stopped before it did anything (`DEAD_RUN_OPENERS`).
static DEAD_RUN_OPENERS: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        "would not start in this project's folder, so nothing has been done here.".into(),
        "could not sign in, so nothing has been done on this task.".into(),
        "is switched off for that account by its organization, so nothing has been done.".into(),
        "cannot run because that account's Claude subscription has ended, so nothing has been done.".into(),
        "is at its usage limit, so nothing has been done on this task.".into(),
        "was cut off in the middle, so nothing has been done on this task.".into(),
        "could not use this task's model on that account's plan, so nothing has been done.".into(),
        "could not run this task's model at the thinking level it asks for, so nothing has been done.".into(),
        "could not connect securely through this network, so nothing has been done on this task.".into(),
        format!("on this computer is older than {NAME} needs, so nothing has been done on this task."),
        format!("is signed in with an API key, which {NAME} never bills, so nothing has been done."),
        "could not read its config.toml, so nothing has been done on this task.".into(),
        "has this conversation open in another window, so nothing has been done here.".into(),
        "No AI plan is set up on this computer, so nothing has been done on this task.".into(),
        "stopped before it did anything, so this task has not been started.".into(),
    ]
});

// A named login sits inside one opener ("is at its usage limit on alex@acme.com, so nothing
// has been done"), so that one is read back by its two halves (`DEAD_RUN_HALVES`).
const AT_LIMIT_HALVES: [&str; 2] = ["is at its usage limit", "so nothing has been done on this task."];

// The same, for a run that used a tool before it died (`WORKED_OPENERS`).
const WORKED_OPENERS: [&str; 7] = [
    "was signed out partway through, so this task is not finished.",
    "was switched off for that account partway through, so this task is not finished.",
    "stopped partway through because that account's subscription ended.",
    "hit its usage limit partway through, so this task is not finished.",
    "was cut off partway through, so this task is not finished.",
    "lost its secure connection partway through, so this task is not finished.",
    "stopped partway through and did not finish this task.",
];

const UNDELIVERED: &str = "Your reply did not reach this task.";

re!(MODEL_FOR_ANOTHER_ENGINE, r#"^This task chose .+ model ".*", but this computer would run it with .+\. Nothing has been done\."#, "");

/// Whether a result is the app saying a run got nowhere: a dead-run sentence, from a run that
/// used a tool or one that did not, the reply that never landed, a thread it stopped starting,
/// or a model that is for the other agent.
pub fn says_a_run_got_nowhere(text: Option<&str>) -> bool {
    let s = text.unwrap_or("");
    if js::trim(s).is_empty() {
        return false;
    }
    if DEAD_RUN_OPENERS.iter().any(|opener| s.contains(opener.as_str())) || AT_LIMIT_HALVES.iter().all(|half| s.contains(half)) {
        return true;
    }
    if MODEL_FOR_ANOTHER_ENGINE.test(s) || WORKED_OPENERS.iter().any(|opener| s.contains(opener)) {
        return true;
    }
    s.starts_with(UNDELIVERED) || s.starts_with(&format!("{NAME} stopped starting this thread"))
}

/// The id of the row the app makes itself when tasks cannot run. It belongs to no project.
pub const TROUBLE_ID: &str = "trouble";

/// The row's title (`strandedTitle`).
pub fn stranded_title(count: f64) -> String {
    let n = count.trunc().max(0.0);
    let many = n != 1.0;
    format!("{n} task{} {} not been able to run", if many { "s" } else { "" }, if many { "have" } else { "has" })
}

re!(BACK_AT, r"at (\d[^.]*)\.", "");

/// What one cause's tasks are waiting on, in a few words (`shortNext`).
fn short_next(cause: &str, next: &str) -> String {
    match cause {
        "at-limit" => BACK_AT.exec(next).map_or("back when the limit resets".to_string(), |m| format!("back at {}", group(&m, 1))),
        "signed-out" => "need a login".into(),
        "no-plan" => "need a plan to run on".into(),
        "org-blocked" => "need an admin".into(),
        "subscription-ended" => "need a renewed subscription".into(),
        "workspace" => "need the folder trusted".into(),
        "interrupted" => "retrying on their own".into(),
        "model-not-on-plan" => "need another model".into(),
        "level-not-offered" => "need another thinking level".into(),
        "too-old" => "need Claude Code updated".into(),
        "network" => "need the network set up".into(),
        "api-key" => "need a ChatGPT login".into(),
        "codex-config" => "need config.toml fixed".into(),
        "open-elsewhere" => "need the other Codex window closed".into(),
        _ => "stopped, still trying".into(),
    }
}

/// The pile, cause by cause: how many tasks and what each is waiting on. A supervisor from
/// before the breakdown existed says one cause for all of them (`causes`).
fn causes(t: &Map<String, Value>) -> Vec<(String, f64, String)> {
    let text = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    match t.get("byCause").and_then(Value::as_array).filter(|list| !list.is_empty()) {
        Some(list) => list.iter().map(|c| (text(c, "cause"), c.get("count").and_then(Value::as_f64).unwrap_or(0.0), text(c, "next"))).collect(),
        None => {
            let cause = t.get("cause").and_then(Value::as_str).unwrap_or("");
            vec![(if cause == "mixed" { "unknown".into() } else { cause.to_string() }, t.get("count").and_then(Value::as_f64).unwrap_or(1.0), t.get("remedy").and_then(Value::as_str).unwrap_or("").to_string())]
        }
    }
}

/// How many tasks the row counts (`troubleCount`).
pub fn trouble_count(t: &Map<String, Value>) -> f64 {
    t.get("count").and_then(Value::as_f64).unwrap_or_else(|| causes(t).iter().map(|c| c.1).sum())
}

/// The row's first line, which is all a list prints of it (`troubleSummary`).
pub fn trouble_summary(t: &Map<String, Value>) -> String {
    causes(t).iter().map(|(cause, count, next)| format!("{count} {}", short_next(cause, next))).collect::<Vec<_>>().join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_apps_own_sentences_are_read_back() {
        assert!(says_a_run_got_nowhere(Some("Claude Code could not sign in, so nothing has been done on this task. Sign in and reply.")));
        assert!(says_a_run_got_nowhere(Some("Codex is at its usage limit on someone@example.com, so nothing has been done on this task.")));
        assert!(says_a_run_got_nowhere(Some("Claude Code was cut off partway through, so this task is not finished.")));
        assert!(says_a_run_got_nowhere(Some("Your reply did not reach this task. The agent ran once and finished without writing anything back on the row.")));
        assert!(says_a_run_got_nowhere(Some("Mercury stopped starting this thread. It was started 6 times in 10 minutes.")));
        assert!(says_a_run_got_nowhere(Some("This task chose Codex model \"gpt-9\", but this computer would run it with Claude Code. Nothing has been done. Pick a Claude Code model to continue.")));
        assert!(!says_a_run_got_nowhere(Some("The export is fixed and every test passes.")));
        assert!(!says_a_run_got_nowhere(Some("  ")));
        assert!(!says_a_run_got_nowhere(None));
    }

    #[test]
    fn the_trouble_row_counts_and_says_what_each_pile_waits_on() {
        assert_eq!(stranded_title(1.0), "1 task has not been able to run");
        assert_eq!(stranded_title(3.0), "3 tasks have not been able to run");
        let t = json!({ "since": 5, "cause": "mixed", "message": "", "remedy": "", "byCause": [
            { "cause": "signed-out", "count": 2, "what": "", "next": "" },
            { "cause": "at-limit", "count": 1, "what": "", "next": "It resets at 6:40pm. Nothing to do." },
        ] });
        let t = t.as_object().unwrap();
        assert_eq!(trouble_count(t), 3.0);
        assert_eq!(trouble_summary(t), "2 need a login · 1 back at 6:40pm");
        let old = json!({ "since": 5, "cause": "mixed", "message": "", "remedy": "" });
        assert_eq!(trouble_summary(old.as_object().unwrap()), "1 stopped, still trying");
    }
}
