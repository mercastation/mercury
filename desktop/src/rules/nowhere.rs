//! The app's own word that a run got nowhere, and how the tasks that have not been able to run
//! are counted. The sentences are `shared/spawn-trouble.mts`'s, read back here the way its
//! `saysARunGotNowhere` reads them; the row that counts them is `trouble_row.rs`.

use std::sync::LazyLock;

use super::js::{self, re};

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

/// The app's note that the thread's model is for the other agent (`saysTheModelIsForAnotherEngine`):
/// another try is refused the same way.
pub fn says_the_model_is_for_another_engine(text: Option<&str>) -> bool {
    MODEL_FOR_ANOTHER_ENGINE.test(text.unwrap_or(""))
}

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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn the_stopped_tasks_are_counted_as_one_task_has_or_many_have() {
        assert_eq!(stranded_title(1.0), "1 task has not been able to run");
        assert_eq!(stranded_title(3.0), "3 tasks have not been able to run");
    }
}
