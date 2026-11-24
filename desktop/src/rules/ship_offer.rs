//! The decision a finished run left behind, when the run offered none: a port of
//! `renderer/src/ship-offer.ts`, function by function. The app asks "Ship this change?" on a
//! thread whose run left a change and asked nothing, and which words it asks in come off where
//! the folder stands (committed, loose, thrown away, a Git that knows nobody), as the saved
//! change records it (`main/git-change.mts`). It is a floor, not a policy: an agent that offers
//! its own pick keeps it. `docs/parity-ship-and-close.md`, B4, is what was ported and how it
//! was checked.

use serde_json::Value;

use super::format::{self, Choice};
use super::js::{self, re};
use super::{answers, nowhere};
use crate::data::Item;

/// The label the ship queue takes a thread by (`main/ship-queue.mts`).
pub const SHIP_LABEL: &str = "ship";

pub const SHIP_ASK: &str = "Ship this change?";
pub const NOT_COMMITTED_ASK: &str = "This change was never committed.";
pub const NAME_YOURSELF_ASK: &str = "Git does not know who you are, so this change was never committed.";

fn choice(n: f64, text: &str, recommended: bool) -> Choice {
    Choice { n, text: text.to_string(), recommended }
}

/// Answered the way the agents' own ship offers are, in their words.
pub fn ship_offer() -> Vec<Choice> {
    vec![choice(1.0, "Ship it", true), choice(2.0, "Discard the change", false)]
}

/// Shipping pushes a branch, so over loose edits "Ship it" would have shipped nothing.
pub fn commit_first_offer() -> Vec<Choice> {
    vec![choice(1.0, "Commit it and ship", true), choice(2.0, "Discard the change", false)]
}

/// A commit needs a name and an email: the recommended press opens the reply box with its own
/// words in it, and the person types them under (`needsYourWords`).
pub fn name_yourself_offer() -> Vec<Choice> {
    vec![choice(1.0, "Commit it and ship, as the name and email below", true), choice(2.0, "Discard the change", false)]
}

/// A document, picture or page the thread already shows is not code to ship.
fn add_files_ask(names: &[String]) -> String {
    if names.len() == 1 { format!("Add {} to the project?", names[0]) } else { format!("Add these {} files to the project?", names.len()) }
}

fn add_files_offer(many: bool) -> Vec<Choice> {
    vec![
        choice(1.0, if many { "Add them to the project" } else { "Add it to the project" }, true),
        choice(2.0, if many { "Throw them away" } else { "Throw it away" }, false),
    ]
}

/// One file a saved change touched, and where it was written when the change says.
#[derive(Clone, Debug, PartialEq)]
pub struct ChangedFile {
    pub path: String,
    pub abs: Option<String>,
}

/// The files a saved change touched (`changedFiles`). The window reads them, and the standing
/// below, off the summary `change.rs` has already parsed (`Change`); these are kept with the
/// web's cases for what reads the answer as it comes.
#[allow(dead_code)]
pub fn changed_files(change: &Value) -> Vec<ChangedFile> {
    let Some(files) = change.get("files").and_then(Value::as_array) else { return Vec::new() };
    files
        .iter()
        .filter_map(|f| {
            let path = f.get("path")?.as_str()?.to_string();
            Some(ChangedFile { path, abs: f.get("abs").and_then(Value::as_str).map(str::to_string) })
        })
        .collect()
}

/// The names of the changed files when every one is a file the thread's words name, else
/// nothing (`onlyNamedFiles`). `named` is relative to `dir`, the project folder. A path read
/// off the disk is matched whole; one without is matched as the same path, or under a parent
/// folder only when the name carries a folder of its own, since a bare "plan.md" could be any.
pub fn only_named_files(changed: &[ChangedFile], named: &[String], dir: Option<&str>) -> Option<Vec<String>> {
    if changed.is_empty() || named.is_empty() {
        return None;
    }
    let root = dir.unwrap_or("").trim_end_matches('/');
    let shown = |c: &ChangedFile| {
        named.iter().any(|n| {
            c.abs.as_deref().is_some_and(|abs| !abs.is_empty() && !root.is_empty() && abs == format!("{root}/{n}")) || c.path == *n || (n.contains('/') && c.path.ends_with(&format!("/{n}")))
        })
    };
    if !changed.iter().all(shown) {
        return None;
    }
    Some(changed.iter().map(|c| c.path.rsplit('/').next().filter(|last| !last.is_empty()).unwrap_or(&c.path).to_string()).collect())
}

/// What the folder a run worked in holds, as the saved change records it. Nothing is "could
/// not be read", which is every product with no repository.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Standing {
    pub committed: Option<bool>,
    pub uncommitted: Option<bool>,
    pub knows_you: Option<bool>,
}

#[allow(dead_code)]
pub const UNREAD: Standing = Standing { committed: None, uncommitted: None, knows_you: None };

/// The standing off a saved change. Anything that is not a yes or a no is unread, never a
/// guess (`standingFrom`).
#[allow(dead_code)]
pub fn standing_from(change: &Value) -> Standing {
    if !change.is_object() {
        return UNREAD;
    }
    let flag = |k: &str| change.get(k).and_then(Value::as_bool);
    Standing { committed: flag("committed"), uncommitted: flag("uncommitted"), knows_you: flag("knowsYou") }
}

/// The offer: its question, its options, and whether the question is drawn over them on the
/// one-at-a-time page, where the agent's own question is already the page's headline. Only the
/// offer that says something is.
#[derive(Clone, Debug, PartialEq)]
pub struct Offer {
    pub ask: String,
    pub options: Vec<Choice>,
    pub headed: bool,
}

// The person said not to: both choices stay, neither recommended over their own words.
re!(HOLD_IT, r"\b(?:do not|don['’]t|dont|never|without)\s+(?:merg(?:e|ing)|ship(?:ping)?|deploy(?:ing)?|push(?:ing)?)\b", "i");

/// Whether any of these words say not to merge, ship, deploy or push (`saidNotToShip`).
pub fn said_not_to_ship(words: &[Option<&str>]) -> bool {
    words.iter().any(|w| HOLD_IT.test(w.unwrap_or("")))
}

// And the agent saying the change is not ready. Its usual "committed without pushing" is not that.
re!(
    NOT_READY,
    &[
        r"\bnot (?:yet )?(?:production[- ]ready|ready to (?:ship|merge|deploy|go out))\b",
        r"\brequired before (?:production|it ships|shipping)\b",
        r"\bbefore (?:shipping|it ships|merging|deploying|(?:it )?goes to production|production)\b,? (?:I|we) (?:recommend|suggest|need|would)",
        r"\bbefore (?:it )?goes to production\b",
        r"\bsuite (?:has|had) (?:a |one |\d+ )?failures?\b",
        r"\b(?:[1-9]\d*|one|two|three|four|five|several|some) (?:tests? )?(?:failed|fail|failing)\b",
        r"\b(?:[1-9]\d*|a|an|one|two|three) (?:existing |remaining |pre-existing )?failures?\b",
        r"\bsuite (?:still )?fails\b",
        r"\b(?:can|could|may|will) (?:still )?(?:produce|return|give) (?:incorrect|wrong) results?\b",
        r"\b(?:lost|lose|is lost|are lost) on (?:restart|deploy)\b",
    ]
    .join("|"),
    "i"
);

/// Whether the agent's result says its change is not ready (`agentSaidNotReady`).
pub fn agent_said_not_ready(result: Option<&str>) -> bool {
    NOT_READY.test(result.unwrap_or(""))
}

/// The offer, or nothing when the folder holds nothing to ask about (`shipOfferFor`). Nothing
/// committed and nothing loose is a change that was thrown away: there is no decision left.
/// `only_files` is `only_named_files`: set, everything loose is a file the thread shows.
/// `held` takes the recommendation off every option.
pub fn ship_offer_for(stands: Standing, only_files: Option<&[String]>, held: bool) -> Option<Offer> {
    let offer = ship_offer_for_standing(stands, only_files)?;
    if !held {
        return Some(offer);
    }
    Some(Offer { options: offer.options.into_iter().map(|o| Choice { recommended: false, ..o }).collect(), ..offer })
}

fn ship_offer_for_standing(stands: Standing, only_files: Option<&[String]>) -> Option<Offer> {
    if stands.committed == Some(false) && stands.uncommitted == Some(false) {
        return None;
    }
    if stands.committed == Some(false) && stands.uncommitted == Some(true) {
        if stands.knows_you == Some(false) {
            return Some(Offer { ask: NAME_YOURSELF_ASK.to_string(), options: name_yourself_offer(), headed: true });
        }
        if let Some(files) = only_files.filter(|f| !f.is_empty()) {
            return Some(Offer { ask: add_files_ask(files), options: add_files_offer(files.len() > 1), headed: true });
        }
        return Some(Offer { ask: NOT_COMMITTED_ASK.to_string(), options: commit_first_offer(), headed: true });
    }
    Some(Offer { ask: SHIP_ASK.to_string(), options: ship_offer(), headed: false })
}

/// How long after a run's result the folder it worked in is still read again. The window's read
/// of the summary keeps to it through `change.rs`, where the code button's figures are read.
#[allow(dead_code)]
pub const FOLDER_SETTLES_MS: f64 = super::change::FOLDER_SETTLES_MS;

/// Whether the folder's standing may still change under a run that has ended: the result lands
/// on the row before the folder is read back (`folderMayStillMove`).
#[allow(dead_code)]
pub fn folder_may_still_move(item: &Item, now: f64) -> bool {
    super::change::folder_may_still_move(item.wrote_at("result"), now)
}

// The agent's last word that the work has landed or that no decision is left.
re!(
    NOTHING_LEFT,
    r"^\s*(?:\*\*)?\s*(?:merged|discarded|already on (?:main|master)\b)\b|\bno net (?:file )?changes\b|\bnothing (?:is )?left (?:for you )?to (?:decide|do)\b|\bnothing (?:more )?for you to (?:decide|do)\b|\bno decision (?:is )?(?:needed|left)\b",
    "i"
);

/// Whether the app asks on this thread's behalf (`shipOfferLive`). `change` is whether the run
/// left one with something in it, `running` whether anything is still going on the row.
pub fn ship_offer_live(item: &Item, change: bool, running: bool) -> bool {
    if !change || running {
        return false;
    }
    // A thread finished with is the person's, and one waiting on the ship queue asks nobody.
    if item.status == "done" || item.has_label(SHIP_LABEL) {
        return false;
    }
    // Its agent set a run still to come, so the work is not finished.
    if answers::wakes_itself_at(item) > 0.0 {
        return false;
    }
    // A result is written when a worker exits, so it is how the window knows the run ended.
    let result = item.result.as_deref().unwrap_or("");
    if js::trim(result).is_empty() {
        return false;
    }
    // And it is the newest word on the row: shipped, its ship failed, or answered already. A
    // pick taken back with Z leaves '(withdrawn)', which answers nothing.
    let answered = item.string("answer") != Some("(withdrawn)") && item.wrote_at("result") < item.wrote_at("answer");
    if item.wrote_at("result") < item.wrote_at("note") || answered {
        return false;
    }
    if NOTHING_LEFT.test(result) {
        return false;
    }
    // The app's own word that the run stopped: the work is half done.
    if nowhere::says_a_run_got_nowhere(Some(result)) {
        return false;
    }
    !format::offer_is_live(item)
}

#[cfg(test)]
mod tests {
    //! The web's own cases, from `tests/a-change-nobody-committed-is-not-offered-for-shipping`,
    //! `a-file-the-thread-shows-is-not-offered-as-code-to-ship`,
    //! `ship-it-is-not-recommended-when-you-said-not-to-ship`,
    //! `the-first-commit-is-not-offered-before-git-knows-who-you-are`,
    //! `a-discarded-change-is-not-offered-to-ship-again`,
    //! `a-finished-run-that-left-a-change-offers-to-ship-it`,
    //! `a-run-that-stopped-partway-is-not-offered-for-shipping`,
    //! `a-ship-it-you-took-back-is-offered-again`, `a-thread-the-app-merged-stops-offering-ship-it`,
    //! `a-thread-you-finished-leaves-needs-you-and-stops-asking-to-ship`,
    //! `a-thread-offers-the-agents-own-choice-before-shipping` and
    //! `the-ship-card-catches-up-with-a-commit-the-last-run-made` (`.test.mts`).
    use super::*;
    use serde_json::json;

    fn st(committed: Option<bool>, uncommitted: Option<bool>, knows_you: Option<bool>) -> Standing {
        Standing { committed, uncommitted, knows_you }
    }
    fn texts(o: &[Choice]) -> Vec<&str> {
        o.iter().map(|c| c.text.as_str()).collect()
    }
    fn recommended(o: &[Choice]) -> Vec<f64> {
        o.iter().filter(|c| c.recommended).map(|c| c.n).collect()
    }
    fn item(v: Value) -> Item {
        serde_json::from_value(v).unwrap()
    }
    fn names(n: &[&str]) -> Vec<String> {
        n.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_offers_words_are_the_webs() {
        assert_eq!(texts(&ship_offer()), ["Ship it", "Discard the change"]);
        assert_eq!(recommended(&ship_offer()), [1.0]);
        assert_eq!(ship_offer().iter().map(|o| o.n).collect::<Vec<_>>(), [1.0, 2.0]);
        assert_eq!(SHIP_ASK, "Ship this change?");
        assert_eq!(SHIP_LABEL, "ship");
        assert_eq!(NOT_COMMITTED_ASK, "This change was never committed.");
        assert_eq!(commit_first_offer().iter().map(|o| o.n).collect::<Vec<_>>(), [1.0, 2.0]);
        assert_eq!(NAME_YOURSELF_ASK, "Git does not know who you are, so this change was never committed.");
    }

    #[test]
    fn the_standing_is_read_off_a_saved_change_and_nothing_else_is_either() {
        assert_eq!(standing_from(&json!({ "committed": true, "uncommitted": false })), st(Some(true), Some(false), None));
        assert_eq!(standing_from(&json!({})), UNREAD);
        assert_eq!(standing_from(&Value::Null), UNREAD);
        assert_eq!(standing_from(&json!({ "committed": "yes" })), UNREAD);
        assert_eq!(standing_from(&json!({ "committed": false, "uncommitted": true, "knowsYou": false })), st(Some(false), Some(true), Some(false)));
        assert_eq!(standing_from(&json!({ "committed": false, "uncommitted": true })).knows_you, None);
        assert_eq!(standing_from(&json!({ "knowsYou": "yes" })), UNREAD);
    }

    #[test]
    fn the_offer_comes_off_where_the_folder_stands() {
        let ship = ship_offer_for(st(Some(true), Some(false), None), None, false).unwrap();
        assert_eq!((ship.ask.as_str(), ship.options.clone(), ship.headed), (SHIP_ASK, ship_offer(), false));
        let loose = ship_offer_for(st(Some(false), Some(true), None), None, false).unwrap();
        assert_eq!((loose.ask.as_str(), texts(&loose.options), recommended(&loose.options), loose.headed), (NOT_COMMITTED_ASK, vec!["Commit it and ship", "Discard the change"], vec![1.0], true));
        assert_eq!(ship_offer_for(st(Some(false), Some(false), None), None, false), None);
        assert_eq!(ship_offer_for(UNREAD, None, false).unwrap().options, ship_offer());
        assert_eq!(ship_offer_for(st(None, Some(true), None), None, false).unwrap().options, ship_offer());
    }

    #[test]
    fn a_file_the_thread_shows_is_asked_about_in_plain_words() {
        let loose = st(Some(false), Some(true), Some(true));
        let one = ship_offer_for(loose, Some(&names(&["launch-plan.md"])), false).unwrap();
        assert_eq!((one.ask.as_str(), texts(&one.options), one.options[0].recommended, one.headed), ("Add launch-plan.md to the project?", vec!["Add it to the project", "Throw it away"], true, true));
        let two = ship_offer_for(loose, Some(&names(&["seat-picker.html", "seat-picker.png"])), false).unwrap();
        assert_eq!((two.ask.as_str(), texts(&two.options)), ("Add these 2 files to the project?", vec!["Add them to the project", "Throw them away"]));
        for n in [names(&["a.md"]), names(&["a.md", "b.png"])] {
            let offer = ship_offer_for(loose, Some(&n), false).unwrap();
            let (add, bin) = (&offer.options[0].text, &offer.options[1].text);
            assert!(format::discards_the_change(bin));
            assert!(!(format::needs_your_words(add) || format::leaves_it_on_the_branch(add) || format::discards_the_change(add)));
        }
        let commit = Offer { ask: NOT_COMMITTED_ASK.to_string(), options: commit_first_offer(), headed: true };
        assert_eq!(ship_offer_for(loose, None, false).as_ref(), Some(&commit));
        assert_eq!(ship_offer_for(loose, Some(&[]), false).as_ref(), Some(&commit));
        assert_eq!(ship_offer_for(st(Some(false), Some(true), Some(false)), Some(&names(&["launch-plan.md"])), false).unwrap().options, name_yourself_offer());
        assert_eq!(ship_offer_for(st(Some(true), Some(false), Some(true)), Some(&names(&["launch-plan.md"])), false).unwrap().options, ship_offer());
        assert_eq!(ship_offer_for(st(Some(false), Some(false), Some(true)), Some(&names(&["launch-plan.md"])), false), None);
    }

    #[test]
    fn a_git_that_knows_nobody_asks_for_a_name_and_an_email() {
        let named = ship_offer_for(st(Some(false), Some(true), Some(false)), None, false).unwrap();
        assert_eq!((named.ask.as_str(), texts(&named.options), recommended(&named.options), named.headed), (NAME_YOURSELF_ASK, vec!["Commit it and ship, as the name and email below", "Discard the change"], vec![1.0], true));
        assert!(format::needs_your_words("Commit it and ship, as the name and email below"));
        assert!(format::needs_your_words("Commit it and ship, as the name and email below (recommended)"));
        assert!(format::discards_the_change("Discard the change"));
        assert!(!format::needs_your_words("Commit it and ship") && !format::needs_your_words("Ship it"));
        assert_eq!(ship_offer_for(st(Some(false), Some(true), Some(true)), None, false).unwrap().ask, NOT_COMMITTED_ASK);
        assert_eq!(ship_offer_for(st(Some(false), Some(true), None), None, false).unwrap().options, commit_first_offer());
        assert_eq!(ship_offer_for(st(Some(true), Some(false), Some(false)), None, false).unwrap().options, ship_offer());
        assert_eq!(ship_offer_for(st(Some(false), Some(false), Some(false)), None, false), None);
    }

    #[test]
    fn ship_it_is_not_recommended_over_the_persons_own_words() {
        let committed = st(Some(true), Some(false), None);
        assert_eq!(ship_offer_for(committed, None, false).unwrap().options.iter().map(|o| o.recommended).collect::<Vec<_>>(), [true, false]);
        let held = ship_offer_for(committed, None, true).unwrap();
        assert_eq!(held.options.iter().map(|o| (o.text.as_str(), o.recommended)).collect::<Vec<_>>(), [("Ship it", false), ("Discard the change", false)]);
        assert_eq!(ship_offer_for(st(Some(false), Some(false), None), None, true), None);
        for said in [&[Some("Add request IDs. Do not merge or deploy.")][..], &[Some("Review it, but don't ship yet")], &[Some("Fix it without pushing anything")], &[None, Some("Never deploy on Fridays")]] {
            assert!(said_not_to_ship(said), "{said:?}");
        }
        for said in [&[Some("Ship it when the tests pass")][..], &[Some("Write the release note for the merge")], &[None]] {
            assert!(!said_not_to_ship(said), "{said:?}");
        }
    }

    #[test]
    fn the_agent_saying_it_is_not_ready_holds_the_recommendation() {
        for said in [
            "The payment route is not production-ready. Persistence must come first.",
            "It is not ready to ship until retries are covered.",
            "It is not yet ready to merge.",
            "Shared durable storage is required before production, so nothing shipped.",
            "Before shipping, I recommend rejecting invalid timestamps on new bookings.",
            "The full suite has a failure in the export test, and the retry store is in memory.",
            "Two tests failed: the export test and the refund test.",
            "Coordinate durable storage with Sam before it goes to production.",
            "The full suite had four passing tests and one failure.",
            "The full suite has one existing failure in the overlap test.",
            "The full suite still fails the existing overlap test.",
            "Mixed time formats or offsets can produce incorrect results.",
            "Keys are lost on restart, so a retry after a deploy can charge twice.",
        ] {
            assert!(agent_said_not_ready(Some(said)), "{said}");
        }
        for said in [
            "0 tests failed and 41 passed.",
            "The test that failed before now passes.",
            "I ran the suite before committing. Everything passes.",
            "All 13 tests pass with no failures.",
            "It no longer produces incorrect totals.",
            "Committed on the branch without pushing. Ready to ship.",
            "All tests pass.",
        ] {
            assert!(!agent_said_not_ready(Some(said)), "{said}");
        }
        assert!(!agent_said_not_ready(None));
    }

    #[test]
    fn the_changed_files_and_the_ones_the_thread_names() {
        let f = |path: &str, abs: Option<&str>| ChangedFile { path: path.to_string(), abs: abs.map(str::to_string) };
        let dir = Some("/work/lumen");
        assert_eq!(only_named_files(&[f("launch-plan.md", None)], &names(&["launch-plan.md"]), dir), Some(names(&["launch-plan.md"])));
        assert_eq!(only_named_files(&[f("pricing-table.md", Some("/work/lumen/.claude/worktrees/w-1/pricing-table.md"))], &names(&["pricing-table.md"]), dir), Some(names(&["pricing-table.md"])));
        assert_eq!(only_named_files(&[f("pricing-table.md", Some("/work/lumen/.claude/worktrees/w-1/pricing-table.md")), f("src/app.ts", Some("/work/lumen/.claude/worktrees/w-1/src/app.ts"))], &names(&["pricing-table.md"]), dir), None);
        assert_eq!(only_named_files(&[f("x/seat-picker.png", Some("/work/lumen/designs/seat-picker.png"))], &names(&["designs/seat-picker.png"]), dir), Some(names(&["seat-picker.png"])));
        assert_eq!(only_named_files(&[f("app/designs/seat-picker.html", None), f("app/designs/seat-picker.png", None)], &names(&["designs/seat-picker.html", "designs/seat-picker.png"]), dir), Some(names(&["seat-picker.html", "seat-picker.png"])));
        assert_eq!(only_named_files(&[f("launch-plan.md", None), f("src/booking.ts", None)], &names(&["launch-plan.md"]), dir), None);
        assert_eq!(only_named_files(&[f("src/plan.md", None)], &names(&["plan.md.bak"]), dir), None);
        assert_eq!(only_named_files(&[f("other/launch-plan.md", None)], &names(&["launch-plan.md"]), dir), None);
        assert_eq!(only_named_files(&[], &names(&["launch-plan.md"]), dir), None);
        assert_eq!(only_named_files(&[f("launch-plan.md", None)], &[], dir), None);
        assert_eq!(changed_files(&Value::Null), []);
        assert_eq!(changed_files(&json!({ "files": [{ "path": "a.md" }, { "path": "b.ts", "abs": "/w/b.ts" }, { "nope": 1 }], "plus": 10, "minus": 0 })), [f("a.md", None), f("b.ts", Some("/w/b.ts"))]);
    }

    /// The finished thread of `a-finished-run-that-left-a-change-offers-to-ship-it`.
    fn finished() -> Value {
        json!({
            "id": "w-1", "product": "p", "status": "open", "labels": [],
            "body": "Make the receipt output nicer. Give me 3 layouts first.",
            "note": "Presented three layouts; waiting on your pick.",
            "answer": "Option 1: Simple columns (recommended)",
            "result": "Simple columns are implemented and tested. Committed as 799c193 on this task's branch.",
            "wrote": { "body": { "ts": 100 }, "note": { "ts": 200 }, "answer": { "ts": 300 }, "result": { "ts": 400 } },
        })
    }
    fn with(mut v: Value, f: impl FnOnce(&mut Value)) -> Item {
        f(&mut v);
        item(v)
    }

    #[test]
    fn a_finished_run_that_left_a_change_is_asked_about_until_something_answers_it() {
        assert!(ship_offer_live(&item(finished()), true, false));
        assert!(!ship_offer_live(&item(finished()), true, true));
        assert!(!ship_offer_live(&item(finished()), false, false));
        assert!(!ship_offer_live(&with(finished(), |v| v["result"] = json!("Ship the currency option?\n\n## Options\n1. Ship it (recommended)\n2. Discard the change")), true, false));
        assert!(!ship_offer_live(&with(finished(), |v| { v["note"] = json!("Shipped to main as 3e4d813."); v["wrote"]["note"] = json!({ "ts": 500 }); }), true, false));
        assert!(!ship_offer_live(&with(finished(), |v| v["labels"] = json!(["ship"])), true, false));
        assert!(!ship_offer_live(&with(finished(), |v| { v["answer"] = json!("Ship it"); v["wrote"]["answer"] = json!({ "ts": 500 }); }), true, false));
        assert!(!ship_offer_live(&with(finished(), |v| v["status"] = json!("done")), true, false));
        assert!(!ship_offer_live(&with(finished(), |v| v["result"] = json!("")), true, false));
        assert!(!ship_offer_live(&with(finished(), |v| v["result"] = json!("   ")), true, false));
        assert!(!ship_offer_live(&with(finished(), |v| { v["note"] = json!("It did not ship, and it went back to its agent: push refused."); v["wrote"]["note"] = json!({ "ts": 500 }); }), true, false));
    }

    #[test]
    fn a_pick_taken_back_is_offered_again() {
        let base = || json!({ "id": "w-1", "product": "p", "status": "open", "labels": [], "body": "Add a /health endpoint.", "result": "Added GET /health with a test. Committed as c41c3f4 on this task's branch.", "wrote": { "body": { "ts": 100 }, "result": { "ts": 400 } } });
        assert!(ship_offer_live(&with(base(), |v| { v["answer"] = json!("(withdrawn)"); v["wrote"]["answer"] = json!({ "ts": 500 }); }), true, false));
        assert!(!ship_offer_live(&with(base(), |v| { v["answer"] = json!("Option 1: Ship it"); v["wrote"]["answer"] = json!({ "ts": 500 }); }), true, false));
        assert!(!ship_offer_live(&with(base(), |v| { v["answer"] = json!("(withdrawn)"); v["note"] = json!("Shipped to main."); v["wrote"]["answer"] = json!({ "ts": 500 }); v["wrote"]["note"] = json!({ "ts": 600 }); }), true, false));
    }

    #[test]
    fn an_answer_that_says_nothing_is_left_asks_nothing() {
        let said = |result: &str, wrote: Value| item(json!({ "id": "w-1", "product": "p", "status": "open", "labels": [], "result": result, "wrote": wrote }));
        let at10 = || json!({ "result": { "ts": 10 } });
        for r in [
            "**Discarded the runbook change.**\n\nCommitted its removal as cba3751. The working tree is clean, with no net changes from the original state.",
            "Discarded. The release-check doc is removed in 4f523eb.",
            "**Removed the scratch doc.** Removal committed in 4f523eb, so there are no net file changes and nothing was merged.",
            "Merged the cancel dialog into the branch, and nothing is left for you to decide.",
            "Merged into main.",
            "It is done. No decision needed from you.",
        ] {
            assert!(!ship_offer_live(&said(r, at10()), true, false), "{r}");
        }
        for r in [
            "**The retry fix is committed.** I discarded the first approach and kept the backoff in client.ts.",
            "**Added the runbook.** Committed as cba3751.",
            "Committed as 799c193 on this task's branch; the working tree is clean.",
            "The two branches can be merged once you say so.",
            "Nothing is left to test. Ready to merge.",
        ] {
            assert!(ship_offer_live(&said(r, at10()), true, false), "{r}");
        }
        let merged = || json!({ "result": { "ts": 2000 }, "note": { "ts": 1000 } });
        for r in [
            "**Merged into main.** The 2 commits this thread made are on main now, merged on this computer.",
            "**Merged into main and pushed.** The commit this thread made is on main here and on origin now.",
            "**Already on main.** Everything this thread made was on main already, so nothing needed merging.",
            "Merged the fix into main.",
            "**Merged.** The branch is on main.",
        ] {
            assert!(!ship_offer_live(&said(r, merged()), true, false), "{r}");
        }
        for r in ["**Fixed the date bug.** The test is green.", "**Ready to merge.** The change is committed."] {
            assert!(ship_offer_live(&said(r, merged()), true, false), "{r}");
        }
    }

    #[test]
    fn a_run_the_app_says_got_nowhere_is_not_offered_for_shipping() {
        let ended = |r: &str| item(json!({ "id": "w-1", "product": "p", "status": "open", "labels": [], "body": "Make the outreach films.", "answer": "verify there are no issues", "result": r, "wrote": { "body": { "ts": 100 }, "answer": { "ts": 200 }, "result": { "ts": 300 } } }));
        for r in [
            "Codex was signed out partway through, so this task is not finished. Sign that account back in. Then reply here to start it again.",
            "Codex was signed out partway through, so this task is not finished. Sign that account back in and it picks straight up.",
            "Codex could not sign in, so nothing has been done on this task. Sign that account back in and it picks straight up.",
            "Codex was cut off partway through, so this task is not finished. Mercury runs it again on its own.",
            "Codex was cut off in the middle, so nothing has been done on this task. Mercury runs it again on its own.",
            "Codex hit its usage limit partway through, so this task is not finished. It starts again on its own when your ChatGPT plan's limit resets.",
            "Codex is at its usage limit, so nothing has been done on this task. It starts again on its own when your ChatGPT plan's limit resets.",
            "Codex stopped partway through and did not finish this task. Mercury tries it again on its own.",
            "Codex stopped before it did anything, so this task has not been started. Check that your ChatGPT plan is active at chatgpt.com, or run codex in a terminal to see what it says. Mercury tries again on its own.",
            "Mercury stopped starting this thread. It was started 6 times in 10 minutes, and each run ended before its agent began. The last one said \"Error running remote compact task: workspace routing discovery unauthorized (401)\". It tries again on its own at 10:13pm. Reply here to try again now.",
        ] {
            assert!(!ship_offer_live(&ended(r), true, false), "{r}");
        }
        for r in ["The films are rebuilt from the approved cut and pass the frame scan.", "I stopped partway through the list on purpose: the rest need your pick."] {
            assert!(ship_offer_live(&ended(r), true, false), "{r}");
        }
    }

    #[test]
    fn the_agents_own_question_wins_over_the_apps() {
        let thread = |r: &str| item(json!({ "id": "w-1", "product": "p", "status": "open", "labels": [], "body": "A retried POST /payments can charge a clinic twice. Add an idempotency key on the route.", "result": r, "wrote": { "body": { "ts": 100 }, "result": { "ts": 200 } } }));
        let reported = "I added an `Idempotency-Key` check to `POST /payments` in `src/routes/payments.js`, and a `payment_requests` table that keeps each key with the response it got.\n\nOne decision before I go on. The old unique index on `(appointment_id, amount)` now overlaps the new key. **Should I drop it in this migration?** Dropping it is simpler, but on a busy table it locks writes while it runs.\n\n1. Drop it in this migration\n2. Keep it for now and drop it in a later release";
        assert!(!ship_offer_live(&thread(reported), true, false));
        assert!(!ship_offer_live(&thread(reported), false, false));
        for r in [
            "I added an `Idempotency-Key` check to `POST /payments` and a `payment_requests` table. Tests pass.",
            "Done. What I changed:\n\n1. Added the key check\n2. Added the table",
            "Should I worry about the old index? No, it still holds.\n\nSteps I took:\n1. Added the check\n2. Added the table",
            "Should I drop it?\n\n1. Drop it\n2. Keep it\n\nI dropped it.\nTests pass.\nThe table is live.",
        ] {
            assert!(ship_offer_live(&thread(r), true, false), "{r}");
        }
    }

    #[test]
    fn the_folder_is_read_again_for_two_minutes_after_the_result() {
        let now = 9_000_000.0;
        let ended = |ago: f64| item(json!({ "id": "w", "product": "p", "result": "Committed.", "wrote": { "result": { "ts": now - ago } } }));
        for ago in [0.0, 30_000.0, FOLDER_SETTLES_MS - 1.0] {
            assert!(folder_may_still_move(&ended(ago), now));
        }
        for ago in [FOLDER_SETTLES_MS, 60.0 * 60_000.0] {
            assert!(!folder_may_still_move(&ended(ago), now));
        }
        assert!(!folder_may_still_move(&item(json!({ "id": "w", "product": "p" })), now));
        assert!(!folder_may_still_move(&item(json!({ "id": "w", "product": "p", "result": "x" })), now));
    }
}
