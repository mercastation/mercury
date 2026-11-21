//! The cloud button in the Inbox's head strip: whether it is there at all, and what it wears.
//! A port of the button's own part of `CloudButton` and of the line in `HeaderActions` that
//! decides it is drawn (`renderer/src/threads/Pages.tsx`), with `cloudButtonCount` and
//! `cloudIsOn` (`renderer/src/cloud-menu.ts`) and `menuSetupLine`, `brokenAccounts` and
//! `buttonAsksSignIn` (`renderer/src/cloud-setup.ts`). Its menu is not ported: this window
//! only reads.

use serde::Serialize;
use serde_json::Value;

use crate::data::truthy;

/// The word a button that is not a bare square carries beside its cloud.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Word {
    /// "Cloud", until one account can run there: nothing else says what the icon is.
    Cloud,
    /// "Sign in again", while an account has stopped working and nobody has looked.
    Again,
}

impl Word {
    pub fn said(self) -> &'static str {
        match self {
            Word::Cloud => "Cloud",
            Word::Again => "Sign in again",
        }
    }
}

/// What the button wears (the classes `lit`, `count` and `named` and what is inside it).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Button {
    /// The cloud is on and an account can run there: the cloud and its count are orange.
    pub lit: bool,
    /// How many threads run in the cloud now, while any do and nothing is being asked.
    pub count: Option<u64>,
    pub word: Option<Word>,
    /// The small orange dot in its corner: something about the setup wants a look.
    pub dot: bool,
}

fn list<'a>(cloud: &'a Value, field: &str) -> &'a [Value] {
    cloud.get(field).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

fn text<'a>(v: &'a Value, field: &str) -> &'a str {
    v.get(field).and_then(Value::as_str).unwrap_or("")
}

/// The accounts that stopped working, as `engine:key` (`brokenAccounts`).
fn broken_accounts(cloud: &Value) -> Vec<String> {
    list(cloud, "accounts").iter().filter(|a| text(a, "state") == "broken").map(|a| format!("{}:{}", text(a, "engine"), text(a, "key"))).collect()
}

/// The cloud menu has a line about the setup: an account that did not get added, one being
/// added, or one that stopped working (`menuSetupLine`, which of them it is being the menu's).
fn menu_has_a_setup_line(cloud: &Value) -> bool {
    let sign_ins = list(cloud, "signIns");
    sign_ins.iter().any(|s| matches!(text(s, "state"), "failed" | "checking")) || !broken_accounts(cloud).is_empty()
}

/// The button for a snapshot's `cloud`, or none on a Mac that is not set up for the cloud and
/// is not offered it. `seen` is what the web window keeps of the stopped accounts already
/// looked at (`zero.cloudBrokenSeen`).
pub fn button(cloud: Option<&Value>, seen: &[String]) -> Option<Button> {
    let cloud = cloud.filter(|c| truthy(c.get("setUp")) || truthy(c.get("offered")))?;
    // How many run there, while any do (`cloudButtonCount`).
    let count = cloud.get("running").and_then(Value::as_f64).filter(|n| *n > 0.0).map(|n| n as u64);
    // On is anything but "this Mac only" (`cloudIsOn`).
    let on = cloud.get("mode").and_then(Value::as_str) != Some("mac");
    // The invitation is for a cloud with no account at all, and none on its way.
    let nobody_ready = list(cloud, "accounts").is_empty() && list(cloud, "signIns").is_empty();
    // A stopped account is said in words until the menu has been opened on it.
    let asks = !nobody_ready && broken_accounts(cloud).iter().any(|k| !seen.contains(k));
    Some(Button {
        lit: on && !nobody_ready,
        count: count.filter(|_| !asks),
        word: if asks {
            Some(Word::Again)
        } else if count.is_none() && nobody_ready {
            Some(Word::Cloud)
        } else {
            None
        },
        dot: asks || (count.is_none() && (nobody_ready || menu_has_a_setup_line(cloud))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn worn(cloud: Value) -> Option<Button> {
        button(Some(&cloud), &[])
    }

    const PLAIN: Button = Button { lit: false, count: None, word: None, dot: false };

    #[test]
    fn there_is_no_button_on_a_mac_the_cloud_was_never_offered_to() {
        assert_eq!(button(None, &[]), None);
        assert_eq!(worn(json!({ "setUp": false, "mode": "full", "running": 0 })), None);
        assert_eq!(worn(json!({})), None);
        assert!(worn(json!({ "setUp": false, "offered": true, "mode": "mac", "running": 0 })).is_some());
    }

    #[test]
    fn it_is_orange_while_the_cloud_is_on_and_counts_what_runs_there() {
        let account = json!([{ "engine": "claude", "key": "a-1", "ready": true, "state": "ready" }]);
        let cloud = |mode: &str, running: u64| json!({ "setUp": true, "mode": mode, "running": running, "accounts": account, "signIns": [] });
        assert_eq!(worn(cloud("full", 0)), Some(Button { lit: true, ..PLAIN }));
        assert_eq!(worn(cloud("all", 3)), Some(Button { lit: true, count: Some(3), ..PLAIN }));
        // Off: a plain square, and what was already running there is still counted.
        assert_eq!(worn(cloud("mac", 0)), Some(PLAIN));
        assert_eq!(worn(cloud("mac", 2)), Some(Button { count: Some(2), ..PLAIN }));
    }

    #[test]
    fn it_says_cloud_until_an_account_is_set_up() {
        let invited = Some(Button { word: Some(Word::Cloud), dot: true, ..PLAIN });
        assert_eq!(worn(json!({ "setUp": false, "offered": true, "mode": "full", "running": 0 })), invited);
        // On with nobody to run it is not on.
        assert_eq!(worn(json!({ "setUp": true, "mode": "all", "running": 0, "accounts": [], "signIns": [] })), invited);
        // One on its way is no longer nobody, and the dot says the menu has a line about it.
        let adding = json!({ "setUp": true, "mode": "full", "running": 0, "accounts": [], "signIns": [{ "engine": "codex", "key": "a-2", "state": "checking" }] });
        assert_eq!(worn(adding), Some(Button { lit: true, dot: true, ..PLAIN }));
        let waiting = json!({ "setUp": true, "mode": "full", "running": 0, "accounts": [], "signIns": [{ "engine": "codex", "key": "a-2", "state": "needs-code" }] });
        assert_eq!(worn(waiting), Some(Button { lit: true, ..PLAIN }));
    }

    #[test]
    fn it_asks_for_a_sign_in_until_the_stopped_account_has_been_looked_at() {
        let cloud = json!({ "setUp": true, "mode": "full", "running": 2, "signIns": [],
            "accounts": [{ "engine": "claude", "key": "a-1", "ready": true, "state": "ready" }, { "engine": "codex", "key": "a-2", "ready": false, "state": "broken" }] });
        // The ask takes the count's place.
        assert_eq!(button(Some(&cloud), &[]), Some(Button { lit: true, word: Some(Word::Again), dot: true, ..PLAIN }));
        // Looked at: the count is back, and no dot while there is a number to read.
        let seen = ["codex:a-2".to_string()];
        assert_eq!(button(Some(&cloud), &seen), Some(Button { lit: true, count: Some(2), ..PLAIN }));
        let mut idle = cloud.clone();
        idle["running"] = json!(0);
        assert_eq!(button(Some(&idle), &seen), Some(Button { lit: true, dot: true, ..PLAIN }));
        assert_eq!(Word::Again.said(), "Sign in again");
    }
}
