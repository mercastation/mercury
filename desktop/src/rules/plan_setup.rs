//! The bar that says agents cannot start until a plan is signed in: whether it is up, what it
//! says and what its two buttons offer (`planBar`, `sureNoPlan`, `needsPlan` and the bar's words
//! in `renderer/src/plan-setup.ts`), and how one reading of this computer is read off the
//! engine's answer (`engineSetup('ready', …)` in `renderer/src/api.ts`).
//!
//! The rest of that file is the walk and the setup card, which this window does not draw: a
//! press on the bar hands over to the web window, which opens its setup card on the plan
//! pressed (`zero:open-plan-setup-here`).

use serde_json::{Value, json};

/// The two agents a plan runs, in the order the web window asks them (`checkPlan`).
pub const ENGINES: [&str; 2] = ["claude", "codex"];

/// The channel each reading is asked on.
pub const ENGINE_SETUP: &str = "zero:engine-setup";

/// The desktop app's door channel that brings its window forward on the setup card for a plan.
pub const OPEN_PLAN_SETUP: &str = "zero:open-plan-setup-here";

/// The words beside the sentence (`COPY.barSub`).
pub const BAR_SUB: &str = "Takes a couple of clicks.";
const BAR_SAY: &str = "Agents cannot start until they have a plan to run on.";
const BAR_SIGNED_OUT_BOTH: &str = "Agents cannot start until you sign in to your Claude or ChatGPT plan.";

/// What one look at the computer found for one agent (`EngineSetupState`, the part the bar reads).
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    pub engine: &'static str,
    pub found: bool,
    pub signed_in: bool,
    /// Whether the reading established anything: false when the search or the sign-in check
    /// did not answer, which is not a no.
    pub certain: bool,
}

/// What the bar says and offers, first button loud and second quiet (`BarWords`). A button
/// carries the plan it sets up: `claude`, `codex` or `both`.
#[derive(Clone, Debug, PartialEq)]
pub struct BarWords {
    pub say: String,
    pub first: (String, String),
    pub second: (String, String),
}

/// The plan's name, the word on screen (`planName`): never the tool's.
pub fn plan_name(engine: &str) -> &'static str {
    if engine == "claude" { "Claude" } else { "ChatGPT" }
}

/// The questions the bar is decided by: `ready` for each agent (`checkPlan`).
pub fn asks() -> Vec<(&'static str, Value)> {
    ENGINES.iter().map(|engine| (ENGINE_SETUP, json!({ "action": "ready", "engine": engine }))).collect()
}

/// One answer read as the web window reads it (`engineSetup`): no answer, or one that is not
/// `ok`, is a check that did not run, sure of nothing; otherwise what it found, sure unless it
/// says it is not.
pub fn reading(engine: &'static str, answer: &Result<Value, String>) -> Reading {
    match answer {
        Ok(r) if crate::data::truthy(r.get("ok")) => Reading {
            engine,
            found: crate::data::truthy(r.get("found")),
            signed_in: crate::data::truthy(r.get("signedIn")),
            certain: r.get("certain") != Some(&Value::Bool(false)),
        },
        _ => Reading { engine, found: false, signed_in: false, certain: false },
    }
}

/// Whether nothing on this computer can run an agent (`needsPlan`): one agent found and signed
/// in is enough for every one.
pub fn needs_plan(readings: &[Reading]) -> bool {
    !readings.iter().any(|r| r.found && r.signed_in)
}

/// And whether the bar may say so (`sureNoPlan`): only a certain no raises it.
pub fn sure_no_plan(readings: &[Reading]) -> bool {
    needs_plan(readings) && readings.iter().all(|r| r.certain)
}

/// What the bar says, or None for no bar (`planBar`). An agent already on the disk leads, and
/// what it is short of is a sign-in rather than a plan.
pub fn plan_bar(readings: &[Reading]) -> Option<BarWords> {
    if !sure_no_plan(readings) {
        return None;
    }
    let here: Vec<&str> = readings.iter().filter(|r| r.found).map(|r| r.engine).collect();
    let word = |engine: &str| {
        let name = plan_name(engine);
        if here.contains(&engine) { format!("Sign in to {name}") } else { format!("Set up with {name}") }
    };
    let button = |engine: &str| (engine.to_string(), word(engine));
    let lead = if here.len() == 1 { here[0] } else { "claude" };
    let other = if lead == "claude" { "codex" } else { "claude" };
    let say = match here.len() {
        1 => format!("Agents cannot start until you sign in to your {} plan.", plan_name(lead)),
        0 => BAR_SAY.to_string(),
        _ => BAR_SIGNED_OUT_BOTH.to_string(),
    };
    Some(BarWords { say, first: button(lead), second: button(other) })
}

/// What a press on one of its buttons asks the desktop app for: its window forward, on the
/// setup card for that plan.
pub fn open_setup(plan: &str) -> (&'static str, Value) {
    (OPEN_PLAN_SETUP, json!({ "plan": plan }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(engine: &'static str, found: bool, signed_in: bool, certain: bool) -> Reading {
        Reading { engine, found, signed_in, certain }
    }
    fn nothing(engine: &'static str) -> Reading {
        read(engine, false, false, true)
    }
    fn words(say: &str, first: (&str, &str), second: (&str, &str)) -> Option<BarWords> {
        let s = |(a, b): (&str, &str)| (a.to_string(), b.to_string());
        Some(BarWords { say: say.to_string(), first: s(first), second: s(second) })
    }

    /// `tests/the-plan-bar-only-says-agents-cannot-start-when-that-is-true.test.mts`.
    #[test]
    fn the_bar_says_agents_cannot_start_only_on_a_certain_no() {
        assert!(sure_no_plan(&[nothing("claude"), nothing("codex")]));
        assert!(sure_no_plan(&[read("claude", true, false, true), nothing("codex")]));
        assert!(!sure_no_plan(&[nothing("claude"), read("codex", true, true, true)]));
        assert!(!sure_no_plan(&[read("claude", true, false, false), nothing("codex")]));
        assert!(!sure_no_plan(&[read("claude", false, false, false), read("codex", false, false, false)]));
        // The walk keeps asking when it cannot be sure; only the bar needs certainty.
        assert!(needs_plan(&[read("claude", true, false, false), read("codex", false, false, false)]));
    }

    /// `tests/the-plan-bar-offers-the-agent-that-is-on-the-computer.test.mts`.
    #[test]
    fn the_bar_leads_with_the_agent_that_is_on_the_computer() {
        assert_eq!(plan_bar(&[nothing("claude"), read("codex", true, false, true)]), words("Agents cannot start until you sign in to your ChatGPT plan.", ("codex", "Sign in to ChatGPT"), ("claude", "Set up with Claude")));
        assert_eq!(plan_bar(&[read("claude", true, false, true), nothing("codex")]), words("Agents cannot start until you sign in to your Claude plan.", ("claude", "Sign in to Claude"), ("codex", "Set up with ChatGPT")));
        assert_eq!(plan_bar(&[nothing("claude"), nothing("codex")]), words("Agents cannot start until they have a plan to run on.", ("claude", "Set up with Claude"), ("codex", "Set up with ChatGPT")));
        assert_eq!(plan_bar(&[read("claude", true, false, true), read("codex", true, false, true)]), words("Agents cannot start until you sign in to your Claude or ChatGPT plan.", ("claude", "Sign in to Claude"), ("codex", "Sign in to ChatGPT")));
        assert_eq!(plan_bar(&[nothing("claude"), read("codex", true, true, true)]), None);
        assert_eq!(plan_bar(&[nothing("claude"), read("codex", true, false, false)]), None);
    }

    #[test]
    fn a_reading_is_the_engines_answer_and_a_check_that_did_not_run_is_sure_of_nothing() {
        assert_eq!(asks(), vec![(ENGINE_SETUP, json!({ "action": "ready", "engine": "claude" })), (ENGINE_SETUP, json!({ "action": "ready", "engine": "codex" }))]);
        assert_eq!(reading("codex", &Ok(json!({ "ok": true, "engine": "codex", "found": true, "signedIn": false, "certain": true }))), read("codex", true, false, true));
        // An engine from before `certain` was said is sure of what it says.
        assert_eq!(reading("claude", &Ok(json!({ "ok": true, "found": false, "signedIn": false }))), nothing("claude"));
        assert_eq!(reading("claude", &Ok(json!({ "ok": true, "found": false, "certain": false }))), read("claude", false, false, false));
        assert_eq!(reading("claude", &Ok(json!({ "ok": false, "error": "Unknown coding agent." }))), read("claude", false, false, false));
        assert_eq!(reading("claude", &Err("no door".into())), read("claude", false, false, false));
        assert_eq!(open_setup("codex"), (OPEN_PLAN_SETUP, json!({ "plan": "codex" })));
    }
}
