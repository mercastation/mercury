//! The row that says tasks have not been able to run, and what it says when it is opened
//! (`renderer/src/trouble-row.ts`, function by function). It is a thread in every way the
//! lists, the keys and the page can see, built here, with an id that is in no ledger. None of
//! its words are chosen here: every sentence comes off the supervisor's `spawnTrouble`, which
//! words them in `shared/spawn-trouble.mts`.

use serde_json::{Map, Value};

use super::js::{group, re};
use super::nowhere;
use crate::data::Item;

/// The one id this row ever has (`TROUBLE_ID`).
pub use super::nowhere::TROUBLE_ID;

/// The app's name, as `shared/product-name.mjs` spells it at the start of a sentence.
const NAME: &str = "Mercury";

/// What the opened row says of a stopped task whose thread cannot be found.
const NO_LONGER_HERE: &str = "A task that is no longer in your inbox";

/// Whether a row is the one the app made for the stopped tasks (`isTroubleRow`).
pub fn is_trouble_row(item: &Item) -> bool {
    item.id == TROUBLE_ID
}

re!(BACK_AT, r"at (\d[^.]*)\.", "");

/// What happens next to one cause's tasks, in as few words as one line of a row holds
/// (`shortNext`). The hour is read out of the cause's own sentence.
pub fn short_next(cause: &str, next: &str) -> String {
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

/// One cause of the pile: how many tasks, what it is and what happens next.
#[derive(Clone, Debug, PartialEq)]
pub struct Cause {
    pub cause: String,
    pub count: f64,
    pub what: String,
    pub next: String,
}

fn text(v: &Value, k: &str) -> String {
    v.get(k).and_then(Value::as_str).unwrap_or("").to_string()
}

/// The pile, cause by cause (`causes`). A supervisor from before the breakdown existed says one
/// cause for all of them, with its own two sentences.
pub fn causes(t: &Map<String, Value>) -> Vec<Cause> {
    match t.get("byCause").and_then(Value::as_array).filter(|list| !list.is_empty()) {
        Some(list) => list.iter().map(|c| Cause { cause: text(c, "cause"), count: c.get("count").and_then(Value::as_f64).unwrap_or(0.0), what: text(c, "what"), next: text(c, "next") }).collect(),
        None => {
            let t = Value::Object(t.clone());
            let cause = text(&t, "cause");
            vec![Cause {
                cause: if cause == "mixed" { "unknown".into() } else { cause },
                count: t.get("count").and_then(Value::as_f64).unwrap_or(1.0),
                what: text(&t, "message"),
                next: text(&t, "remedy"),
            }]
        }
    }
}

/// How many tasks the row counts (`troubleCount`).
pub fn trouble_count(t: &Map<String, Value>) -> f64 {
    t.get("count").and_then(Value::as_f64).unwrap_or_else(|| causes(t).iter().map(|c| c.count).sum())
}

/// The row's title, which is the opened row's heading too (`troubleTitle`).
pub fn trouble_title(t: &Map<String, Value>) -> String {
    nowhere::stranded_title(trouble_count(t))
}

/// The row's second line: one short clause per cause, in the supervisor's order, which puts
/// what needs the person first (`troubleSummary`).
pub fn trouble_summary(t: &Map<String, Value>) -> String {
    causes(t).iter().map(|c| format!("{} {}", c.count, short_next(&c.cause, &c.next))).collect::<Vec<_>>().join(" · ")
}

/// What opening it says (`troubleBody`): the summary line first, then each cause in full and
/// the tasks stopped on it, by their titles, with their project. A task whose thread cannot be
/// found is still listed, so the list adds up to the number in the title.
pub fn trouble_body(t: &Map<String, Value>, items: &[Item]) -> String {
    let by_id = |id: &str| items.iter().find(|i| i.id == id);
    let stopped: Vec<(String, String)> = match t.get("stopped").filter(|s| !s.is_null()) {
        Some(list) => list.as_array().into_iter().flatten().map(|s| (text(s, "id"), text(s, "cause"))).collect(),
        None => t.get("ids").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).map(|id| (id.to_string(), "unknown".to_string())).collect(),
    };
    let mut parts = vec![trouble_summary(t)];
    let all = causes(t);
    for c in &all {
        parts.push(format!("**{}.** {}", c.what, c.next));
        let mine: Vec<String> = stopped
            .iter()
            .filter(|(_, cause)| *cause == c.cause)
            .map(|(id, _)| {
                let item = by_id(id);
                let title = item.map_or(NO_LONGER_HERE.to_string(), |i| i.title.clone().unwrap_or_default());
                let project = item.and_then(|i| i.product_name.as_deref()).filter(|p| !p.is_empty()).map_or(String::new(), |p| format!(" ({p})"));
                format!("- {title}{project}")
            })
            .collect();
        if !mine.is_empty() {
            parts.push(mine.join("\n"));
        }
    }
    // The rows the breakdown does not account for: drawn rather than dropped.
    let rest: Vec<&(String, String)> = stopped.iter().filter(|(_, cause)| !all.iter().any(|c| c.cause == *cause)).collect();
    if !rest.is_empty() {
        parts.push(format!("**{} more.** {NAME} keeps trying on its own.", rest.len()));
        parts.push(rest.iter().map(|(id, _)| format!("- {}", by_id(id).and_then(|i| i.title.clone()).unwrap_or_else(|| NO_LONGER_HERE.to_string()))).collect::<Vec<_>>().join("\n"));
    }
    parts.join("\n\n")
}

/// The row itself, in the shape everything downstream reads (`troubleRow`): open, at the middle
/// level, in no project, and both its times when the pile began.
pub fn trouble_row(t: &Map<String, Value>, items: &[Item]) -> Item {
    let since = t.get("since").and_then(Value::as_f64);
    Item {
        id: TROUBLE_ID.into(),
        product: String::new(),
        product_name: Some(String::new()),
        status: "open".into(),
        kind: Some("trouble".into()),
        title: Some(trouble_title(t)),
        body: Some(trouble_body(t, items)),
        labels: Vec::new(),
        priority: Some(5.0),
        created_at: since,
        updated_at: since,
        ..Item::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // `tests/the-row-that-says-nothing-is-running.test.mts`: her night, four tasks on a
    // signed-out login and seven at the limit that resets at 6pm, as `causeBreakdown` sorts them.
    const SINCE: f64 = 1_787_965_260_000.0;

    fn trouble(rows: &[(&str, &str)], resets: Option<&str>) -> Map<String, Value> {
        let mut by: Vec<(String, f64)> = Vec::new();
        for (_, cause) in rows {
            match by.iter_mut().find(|(c, _)| c == cause) {
                Some(entry) => entry.1 += 1.0,
                None => by.push((cause.to_string(), 1.0)),
            }
        }
        let what = |cause: &str, n: f64| match cause {
            "signed-out" => format!("{n} stopped by a signed-out Claude login"),
            "at-limit" => format!("{n} waiting on your Claude limit"),
            "workspace" => format!("{n} in a folder Claude will not run in"),
            _ => format!("{n} stopped for a reason we cannot name"),
        };
        let next = |cause: &str| match (cause, resets) {
            ("signed-out", _) => "Type /login in a terminal and they pick straight up.".to_string(),
            ("at-limit", Some(at)) => format!("They start again on their own at {at}."),
            ("at-limit", None) => "They start again on their own when it resets.".to_string(),
            ("workspace", _) => "Open the folder in Claude once and say yes to trusting it.".to_string(),
            _ => "Mercury keeps trying on its own.".to_string(),
        };
        // Hers first, then the bigger pile (`causeBreakdown`).
        let hers = |cause: &str| cause != "at-limit" && cause != "interrupted";
        by.sort_by(|a, b| hers(&b.0).cmp(&hers(&a.0)).then(b.1.partial_cmp(&a.1).unwrap()));
        let t = json!({
            "since": SINCE, "cause": "mixed", "message": "", "remedy": "", "count": rows.len(), "resetsAt": resets,
            "byCause": by.iter().map(|(cause, n)| json!({ "cause": cause, "count": n, "what": what(cause, *n), "next": next(cause), "act": null, "hers": hers(cause) })).collect::<Vec<_>>(),
            "ids": rows.iter().map(|r| r.0).collect::<Vec<_>>(),
            "stopped": rows.iter().map(|(id, cause)| json!({ "id": id, "cause": cause })).collect::<Vec<_>>(),
            "retryAt": 0,
        });
        t.as_object().unwrap().clone()
    }

    fn her_night() -> Map<String, Value> {
        let ids: Vec<(String, &str)> = (0..4).map(|i| (format!("s{i}"), "signed-out")).chain((0..7).map(|i| (format!("l{i}"), "at-limit"))).collect();
        let rows: Vec<(&str, &str)> = ids.iter().map(|(id, cause)| (id.as_str(), *cause)).collect();
        trouble(&rows, Some("6pm"))
    }

    fn her_items() -> Vec<Item> {
        vec![
            serde_json::from_value(json!({ "id": "s0", "product": "shop", "title": "Buy:", "productName": "Mercury" })).unwrap(),
            serde_json::from_value(json!({ "id": "l0", "product": "shop", "title": "Urgent tasks are being prioritized below high priority tasks", "productName": "Mercury" })).unwrap(),
        ]
    }

    #[test]
    fn the_row_says_how_many_in_the_words_the_app_uses_everywhere_else() {
        assert_eq!(trouble_title(&her_night()), "11 tasks have not been able to run");
        assert_eq!(trouble_title(&trouble(&[("a", "signed-out")], None)), "1 task has not been able to run");
    }

    #[test]
    fn its_second_line_leads_with_what_happens_next_and_what_needs_her_hands_first() {
        assert_eq!(trouble_summary(&her_night()), "4 need a login · 7 back at 6pm");
        assert_eq!(trouble_summary(&trouble(&[("l", "at-limit")], None)), "1 back when the limit resets");
        let three: Vec<(String, &str)> = (0..4).map(|i| (format!("s{i}"), "signed-out")).chain((0..7).map(|i| (format!("l{i}"), "at-limit"))).chain((0..2).map(|i| (format!("w{i}"), "workspace"))).collect();
        let rows: Vec<(&str, &str)> = three.iter().map(|(id, cause)| (id.as_str(), *cause)).collect();
        let summary = trouble_summary(&trouble(&rows, Some("6pm")));
        // `SUMMARY_BUDGET` in `renderer/src/list-rules.ts`.
        assert!(summary.chars().count() < 112, "{summary}");
        assert!(summary.contains("back at 6pm"));
    }

    #[test]
    fn it_is_stamped_with_when_this_started_and_opens_with_the_line_the_row_prints() {
        let row = trouble_row(&her_night(), &her_items());
        assert_eq!((row.created_at, row.updated_at), (Some(SINCE), Some(SINCE)));
        assert_eq!(row.body.as_deref().unwrap().split('\n').next(), Some("4 need a login · 7 back at 6pm"));
        assert_eq!(super::super::list::row_summary(&row, row.body.as_deref()), "4 need a login · 7 back at 6pm");
    }

    #[test]
    fn opening_it_names_every_cause_what_happens_next_and_the_tasks() {
        let body = trouble_body(&her_night(), &her_items());
        for said in ["4 stopped by a signed-out Claude login.", "Type /login in a terminal and they pick straight up.", "7 waiting on your Claude limit.", "They start again on their own at 6pm.", "Buy: (Mercury)", "Urgent tasks are being prioritized below high priority tasks (Mercury)"] {
            assert!(body.contains(said), "{said} in {body}");
        }
        // A task whose row cannot be found is still listed: the list adds up to the number.
        let lines = |body: &str| body.lines().filter(|l| l.starts_with("- ")).count();
        assert_eq!(lines(&trouble_body(&her_night(), &[])), 11);
        // A supervisor that sent only ids: nothing is put under a cause it cannot prove.
        let mut old = her_night();
        old.remove("stopped");
        let body = trouble_body(&old, &her_items());
        assert!(body.contains("4 stopped by a signed-out Claude login."));
        assert!(body.contains("**11 more.** Mercury keeps trying on its own."));
        assert_eq!(lines(&body), 11);
    }

    #[test]
    fn it_is_one_row_with_one_id_no_urgent_mark_and_no_project() {
        let row = trouble_row(&her_night(), &her_items());
        assert_eq!(row.id, TROUBLE_ID);
        assert!(is_trouble_row(&row));
        assert_eq!((row.status.as_str(), row.priority, row.product.as_str(), row.product_name.as_deref()), ("open", Some(5.0), "", Some("")));
        assert!(row.labels.is_empty());
    }

    #[test]
    fn a_supervisor_from_before_the_breakdown_still_draws_its_own_two_sentences() {
        let old = json!({ "since": 5, "cause": "mixed", "message": "", "remedy": "" });
        assert_eq!(trouble_summary(old.as_object().unwrap()), "1 stopped, still trying");
        let t = json!({ "since": 5, "cause": "mixed", "byCause": [
            { "cause": "signed-out", "count": 2, "what": "", "next": "" },
            { "cause": "at-limit", "count": 1, "what": "", "next": "It resets at 6:40pm. Nothing to do." },
        ] });
        assert_eq!(trouble_count(t.as_object().unwrap()), 3.0);
        assert_eq!(trouble_summary(t.as_object().unwrap()), "2 need a login · 1 back at 6:40pm");
    }
}
