//! The app's own question on a thread whose run ended, left a change and asked nothing
//! (`renderer/src/ship-offer.ts`, `shipAsk` and the `.one-choices` strip in `Focus.tsx`,
//! `offerNow` in `App.tsx`): "Ship this change?", or, off where the folder stands, "This change
//! was never committed.", the question for a Git that knows nobody, or the one about files the
//! thread already shows. It is drawn where an agent's options are and answered the same ways,
//! by the numbers, the arrows and a press; only the three that say something wear their question
//! over the options. The rules are `rules/ship_offer.rs`; what is read is the change's summary
//! `change.rs` already reads for the strip's code button. `docs/parity-ship-and-close.md`, B4.

use rune::{El, TextStyle, col, text};

use crate::data::Item;
use crate::rules::format::Choice;
use crate::rules::roots::referenced_files;
use crate::rules::ship_offer::{self as rules, ChangedFile, Offer, Standing};
use crate::theme::*;
use crate::State;

/// The project's own folder, which the files a thread names are read against (`productDir`).
fn product_dir<'a>(s: &'a State, item: &Item) -> Option<&'a str> {
    s.snap.products.iter().find(|p| p.slug == item.product).and_then(|p| p.rest.get("dir")).and_then(serde_json::Value::as_str)
}

/// The offer the app makes on `item`, when it makes one (`shipAsk`): the run left a change with
/// something in it, nothing is running on the row, and nothing answered the run since.
pub fn offer(s: &State, item: &Item) -> Option<Offer> {
    let summary = crate::change::summary(s, item);
    let change = summary.as_deref().and_then(crate::rules::change::figures_from).is_some();
    // One of the person's own agent sessions is not one of our runs.
    let running = s.run_on(item) || item.truthy("agent");
    if !rules::ship_offer_live(item, change, running) {
        return None;
    }
    let c = summary?;
    let stands = Standing { committed: c.committed, uncommitted: c.uncommitted, knows_you: c.knows_you };
    let changed: Vec<ChangedFile> = c.files.iter().map(|f| ChangedFile { path: f.path.clone(), abs: f.abs.clone() }).collect();
    let dir = product_dir(s, item);
    let words = [item.result.as_deref(), item.text("note"), item.body.as_deref(), item.text("answer")].map(|w| w.unwrap_or(""));
    let named = referenced_files(&words, dir);
    let only = rules::only_named_files(&changed, &named, dir);
    let held = rules::said_not_to_ship(&[item.body.as_deref(), item.text("answer")]) || rules::agent_said_not_ready(item.result.as_deref());
    rules::ship_offer_for(stands, only.as_deref(), held)
}

/// What the open thread offers now, which the keys and a press answer (`offerNow`): the
/// agent's own options while they stand, else the app's.
pub fn offer_now(s: &State, item: &Item) -> Vec<Choice> {
    let live = crate::thread::live_options(item);
    if !live.is_empty() {
        return live;
    }
    offer(s, item).map(|o| o.options).unwrap_or_default()
}

/// The question over the strip, when the app's offer is one that says something
/// (`bare={!shipAsk?.headed}`).
fn headed(s: &State, item: &Item) -> Option<String> {
    if !crate::thread::live_options(item).is_empty() {
        return None;
    }
    offer(s, item).filter(|o| o.headed).map(|o| o.ask)
}

/// The head's words are two lines at most; past that the card over the strip shows them whole
/// (`askIsClipped`).
fn clipped(ask: &str, column: f32) -> bool {
    let room = (column - 2.0 - 32.0).max(1.0);
    rune::text::measure(ask, &[], TextStyle::new(12.5, 600.0).lh(17.5), Some(room)).1 > 2.0 * 17.5 + 1.0
}

/// The strip's head on `item`, when the offer wears one, and the card over the strip that
/// shows it whole while the pointer is on it and it is cut short (`.opt-head-ask`,
/// `onAskEnter`, "The question, in full").
pub fn head(s: &State, item: &Item, column: f32) -> (Option<El>, Option<El>) {
    let Some(ask) = headed(s, item) else { return (None, None) };
    let head = crate::closing::ask_head(&ask).on_hover_change(|s: &mut State, over, _| s.closing.head_peek = over);
    let card = (s.closing.head_peek && clipped(&ask, column)).then(|| {
        let tall = rune::memo::layout_size("options-strip").map_or(0.0, |[_, h]| h);
        let comes_in = rune::Transition::offset(0.0, 4.0).combined(rune::Transition::scale(0.98)).combined(rune::Transition::opacity());
        // Inside the strip's edge, eight points over it (`left: 0; right: 0; bottom: 100%`).
        col()
            .pin(Some(1.0), None, Some(1.0), Some(tall + 7.0))
            .pt(12.0)
            .px(16.0)
            .pb(13.0)
            .radius(10.0)
            .fill(raised())
            .border(1.0, rule())
            .border_layout()
            .box_shadow(shadow(0.0, 24.0, 70.0, 0.0, rgba(0, 0, 0, 0.55), false))
            .animated_changes()
            .transition(rune::Transition::asymmetric(comes_in, rune::Transition::none()).springs(rune::SpringPair::new(rune::Spring::new(0.028, -0.66), rune::Spring::NONE)))
            .child(pen(text("THE QUESTION, IN FULL").font(TextStyle::new(10.5, 600.0).lh(15.75).tracking(0.84)).color(muted()), 10.5).mb(6.0))
            .child(pen(text(ask.as_str()).font(TextStyle::new(14.0, 400.0).lh(21.0)).color(ink()).wrap(), 14.0))
    });
    (Some(head), card)
}

#[cfg(test)]
mod tests {
    use rune::{Key, Runtime};
    use serde_json::{Value, json};

    use super::*;
    use crate::feel::tests::{key, press, window_dressed};
    use crate::rules::writes::ANSWER;

    /// The invented inbox's first thread, finished by its agent with `result`, its run having
    /// left a change of `files` standing as `stands`, opened.
    pub(crate) fn finished(name: &str, result: &str, stands: Value, files: &[&str]) -> Runtime<State> {
        let result = result.to_string();
        let mut summary = json!({ "ok": true, "change": { "files": files.iter().map(|p| json!({ "path": p, "plus": 4, "minus": 1 })).collect::<Vec<_>>(), "plus": 4 * files.len(), "minus": files.len() } });
        for (k, v) in stands.as_object().unwrap() {
            summary["change"][k] = v.clone();
        }
        let said = result.clone();
        let mut rt = window_dressed(name, &["--thread", "w-feel0"], "/fixtures/storefront", &said, move |fixture| {
            let now = fixture["now"].as_f64().unwrap();
            let item = &mut fixture["snapshot"]["items"][0];
            item["status"] = json!("open");
            item["result"] = json!(result);
            item["wrote"]["result"] = json!({ "ts": now - 600_000.0, "source": "agent" });
            let key = format!("{}/{}", item["product"].as_str().unwrap(), item["id"].as_str().unwrap());
            fixture["threads"][key]["zero:code-change"] = summary.clone();
        });
        rt.state.pretend_sent = true;
        crate::feel::tests::run(&mut rt, 1.0);
        rt
    }

    fn says(rt: &Runtime<State>, words: &str) -> bool {
        rt.elements().iter().any(|e| e.text.as_deref() == Some(words))
    }

    #[test]
    fn a_finished_run_that_left_a_commit_is_offered_for_shipping_bare() {
        let mut rt = finished("ship-commit", "Simple columns are implemented and tested. Committed as 799c193 on this task's branch.", json!({ "committed": true, "uncommitted": false }), &["src/receipt.ts"]);
        assert!(says(&rt, "Ship it") && says(&rt, "Discard the change") && says(&rt, "recommended"));
        // The page's own headline is the question: no head over the options.
        assert!(!says(&rt, "Ship this change?"));
        // The number sends it as any pick is sent.
        key(&mut rt, Key::Text("1".into()));
        rt.update(|s, _| s.write_what_waits());
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-feel0", "answer": "Option 1: Ship it" }))]);
    }

    #[test]
    fn work_nobody_committed_says_so_over_its_options() {
        let rt = finished("ship-loose", "Simple columns are implemented and tested.", json!({ "committed": false, "uncommitted": true, "knowsYou": true }), &["src/receipt.ts"]);
        assert!(says(&rt, "This change was never committed.") && says(&rt, "Commit it and ship"));
        let (_, head) = rt.find("ask-head").expect("the head");
        let (_, first) = rt.find("option-0").expect("the first option");
        assert!(head < first);
    }

    #[test]
    fn a_git_that_knows_nobody_opens_the_box_for_a_name_and_an_email() {
        let mut rt = finished("ship-name", "Simple columns are implemented and tested.", json!({ "committed": false, "uncommitted": true, "knowsYou": false }), &["src/receipt.ts"]);
        assert!(says(&rt, "Git does not know who you are, so this change was never committed."));
        key(&mut rt, Key::Text("1".into()));
        assert!(rt.state.tried.is_empty() && rt.state.replying);
        assert_eq!(rt.state.reply.text, "Commit it and ship, as the name and email below\n\n");
    }

    #[test]
    fn throwing_it_away_asks_first_then_sends_the_pick() {
        let mut rt = finished("ship-discard", "Simple columns are implemented and tested.", json!({ "committed": true, "uncommitted": false }), &["src/receipt.ts"]);
        let (x, y) = rt.find("option-1").expect("Discard the change");
        press(&mut rt, x, y);
        assert!(rt.state.discard_asked().is_some() && says(&rt, "Discard the change?"));
        key(&mut rt, Key::Text("1".into()));
        rt.update(|s, _| s.write_what_waits());
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-feel0", "answer": "Option 2: Discard the change" }))]);
    }

    #[test]
    fn files_the_thread_shows_are_asked_about_by_name() {
        let rt = finished("ship-files", "The plan is in launch-plan.md.", json!({ "committed": false, "uncommitted": true, "knowsYou": true }), &["launch-plan.md"]);
        assert!(says(&rt, "Add launch-plan.md to the project?") && says(&rt, "Add it to the project") && says(&rt, "Throw it away"));
    }

    #[test]
    fn a_head_cut_short_shows_whole_over_the_strip_while_the_pointer_is_on_it() {
        let name = format!("{}.md", "the-refunds-for-support-and-the-order-page-and-the-tax-service-".repeat(5));
        let mut rt = finished("ship-peek", &format!("The note is in notes/{name}."), json!({ "committed": false, "uncommitted": true, "knowsYou": true }), &[&format!("notes/{name}")]);
        let ask = format!("Add {name} to the project?");
        assert!(says(&rt, &ask) && !says(&rt, "THE QUESTION, IN FULL"));
        let (x, y) = rt.find("ask-head").expect("the head");
        rest_on(&mut rt, x, y);
        assert!(says(&rt, "THE QUESTION, IN FULL"));
        // A head that fits is never shown twice.
        let mut fits = finished("ship-peek-fits", "Simple columns are implemented and tested.", json!({ "committed": false, "uncommitted": true, "knowsYou": true }), &["src/receipt.ts"]);
        let (x, y) = fits.find("ask-head").expect("the head");
        rest_on(&mut fits, x, y);
        assert!(!says(&fits, "THE QUESTION, IN FULL"));
    }

    fn rest_on(rt: &mut Runtime<State>, x: f32, y: f32) {
        rt.pointer_move(x, y);
        crate::feel::tests::run(rt, 0.2);
    }

    #[test]
    fn nothing_is_offered_over_a_thrown_away_change_or_an_agents_own_question() {
        let gone = finished("ship-gone", "Simple columns are implemented and tested.", json!({ "committed": false, "uncommitted": false }), &["src/receipt.ts"]);
        assert!(!says(&gone, "Ship it") && gone.find("option-0").is_none());
        let asked = finished("ship-asked", "Which layout?\n\n## Options\n\n1. Columns (recommended)\n2. Cards", json!({ "committed": true, "uncommitted": false }), &["src/receipt.ts"]);
        assert!(says(&asked, "Columns") && !says(&asked, "Ship it"));
    }
}
