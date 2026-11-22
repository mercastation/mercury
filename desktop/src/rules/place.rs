//! Where one thread runs, for the button beside Done over a conversation: this Mac or the
//! cloud. `thread_place` follows `threadPlace` in `renderer/src/cloud-menu.ts`; `view`, `mark`
//! and `after_pick` follow `placeView` and `afterPick` in `renderer/src/threads/PlaceButton.tsx`,
//! with the reasons in words of `renderer/src/where-why.ts`; `writes` is what a pick writes
//! (`placeThread` in `renderer/src/App.tsx`).

use serde_json::{Map, Value, json};

use crate::data::{Item, Snapshot, truthy};

/// The thread's setting (`ThreadPlace`).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Place {
    /// In the cloud now.
    Cloud,
    /// Here, and free to go to the cloud.
    Here,
    /// The person kept it on this Mac.
    Kept,
    /// Working here, and it can carry on in the cloud.
    Move,
    /// Here because it needs something only this Mac has.
    Needs,
    /// Here because the whole cloud is off.
    Off,
}

/// What the closed button draws.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Mark {
    /// A cloud, else a Mac.
    pub cloud: bool,
    /// The whole cloud is off: the cloud is drawn faded.
    pub faded: bool,
    /// It was handed back for something the cloud does not have.
    pub dot: bool,
}

fn has(list: Option<&Value>, word: &str) -> bool {
    list.and_then(Value::as_array).is_some_and(|l| l.iter().any(|v| v.as_str() == Some(word)))
}

/// Set up for the cloud, and not on the managed cloud without a plan (`onCloudPlan`).
fn on_cloud_plan(cloud: Option<&Map<String, Value>>) -> Option<&Map<String, Value>> {
    let cloud = cloud.filter(|c| truthy(c.get("setUp")))?;
    let no_plan = cloud.get("plan").and_then(Value::as_str) == Some("none")
        || cloud.get("noRoom").and_then(|n| n.get("error")).and_then(Value::as_str) == Some("plan");
    (!(truthy(cloud.get("managed")) && no_plan)).then_some(cloud)
}

/// The run on this thread now, as the supervisor tells it.
fn session<'a>(snap: &'a Snapshot, item: &Item) -> Option<&'a crate::data::Run> {
    snap.supervisor.running.iter().find(|r| r.item_id == item.id)
}

fn in_the_cloud(run: &crate::data::Run) -> bool {
    run.rest.get("where").and_then(Value::as_str) == Some("cloud")
}

/// Where the thread runs, or nothing where the cloud could never run it or it runs nowhere:
/// a finished thread, an outside agent's, no cloud set up, a project or agent the cloud does
/// not take.
pub fn thread_place(snap: &Snapshot, item: &Item) -> Option<Place> {
    // A finished thread runs nowhere (`Focus.tsx`), and an outside agent's is not ours to place (`App.tsx`).
    if item.status == "done" || item.truthy("agent") {
        return None;
    }
    let run = session(snap, item);
    if run.is_some_and(|r| truthy(r.rest.get("movable")) && !in_the_cloud(r)) {
        return Some(Place::Move);
    }
    let cloud = on_cloud_plan(snap.rest.get("cloud").and_then(Value::as_object))?;
    if item.truthy("needsMac") {
        return Some(Place::Needs);
    }
    if item.truthy("onMac") {
        return Some(Place::Kept);
    }
    if run.is_some_and(in_the_cloud) {
        return Some(Place::Cloud);
    }
    // The agent its next reply is on: this thread's own, else the workspace's (`runningEngine`).
    let engines = snap.rest.get("engines").and_then(Value::as_object);
    let engine = engines.and_then(|e| e.get("byItem")).and_then(|b| b.get(&item.id)).and_then(Value::as_str).or_else(|| engines.and_then(|e| e.get("workspace")).and_then(Value::as_str));
    if engine.is_some_and(|e| !e.is_empty() && has(cloud.get("noCloudSignIn"), e)) {
        return None;
    }
    let taken = has(cloud.get("projects"), &item.product);
    // `cloudIsOn`: every mode but "this Mac only".
    if cloud.get("mode").and_then(Value::as_str) == Some("mac") {
        return taken.then_some(Place::Off);
    }
    if has(cloud.get("movable"), &item.id) {
        return Some(Place::Move);
    }
    taken.then_some(Place::Here)
}

/// Why a run here is not in the cloud, as the supervisor says it (`WhyNotCloud`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Why {
    pub at: String,
    pub why: String,
    pub detail: Option<String>,
}

/// What the button and its panel are drawn from (`PlaceFacts`).
#[derive(Clone, Debug, PartialEq)]
pub struct Facts {
    pub place: Place,
    /// An agent is working on it on this Mac now.
    pub running: bool,
    pub why: Option<Why>,
    /// What the cloud handed it back for ("your Stripe key").
    pub needs: Option<String>,
}

/// The facts of a thread the cloud could take, off the snapshot.
pub fn facts(snap: &Snapshot, item: &Item, place: Place) -> Facts {
    let why = snap.supervisor.rest.get("whyNotCloud").and_then(|w| w.get(&item.id)).map(|w| {
        let text = |field: &str| w.get(field).and_then(|v| v.as_str().map(str::to_string).or_else(|| v.as_f64().map(|n| n.to_string())));
        Why { at: text("at").unwrap_or_default(), why: text("why").unwrap_or_default(), detail: text("detail").filter(|d| !d.is_empty()) }
    });
    Facts { place, running: session(snap, item).is_some_and(|r| !in_the_cloud(r)), why, needs: item.text("needsMac").map(str::to_string) }
}

/// What picking a place does (`PlaceAct`): keep it on this Mac (`Keep(true)`), let the cloud
/// have it (`Keep(false)`), move its run there now, or turn the whole cloud on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Act {
    Keep(bool),
    Move,
    CloudOn,
}

/// The panel, and the closed button with it (`PlaceView`).
#[derive(Clone, Debug, PartialEq)]
pub struct View {
    /// The thread's setting, which the panel picks: the cloud, else this Mac.
    pub on_cloud: bool,
    /// Where it runs now, which the closed button names: a reply running here is on this Mac
    /// whatever the setting.
    pub now_cloud: bool,
    /// The whole cloud is off: the button is a faded cloud.
    pub off: bool,
    pub dot: bool,
    pub note: String,
    pub needs: Option<String>,
    /// What picking each place does, or nothing where it is the place already or cannot be picked.
    pub pick_cloud: Option<Act>,
    pub pick_mac: Option<Act>,
}

/// No reply of the thread can go to the cloud while one of these holds, so Cloud cannot be picked
/// (`STAYS`); while one of the other two does the cloud is out only for now (`FOR_NOW`).
const STAYS: [&str; 6] = ["folder", "project", "teammate", "codex-reply", "no-account", "version"];
const FOR_NOW: [&str; 2] = ["cloud-failing", "cloud-account-limited"];
const MOVING: &str = "Working on this Mac now. It moves to the cloud as soon as the cloud has room.";

/// The cloud brief's own words for a run that has to merge (`/^merging into main\b/i`).
fn merging_into_main(detail: &str) -> bool {
    const WORDS: &str = "merging into main";
    detail.get(..WORDS.len()).is_some_and(|head| head.eq_ignore_ascii_case(WORDS)) && !detail[WORDS.len()..].starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
}

fn engine_name(detail: Option<&str>) -> &'static str {
    match detail {
        Some("claude") => "Claude",
        Some("codex") => "Codex",
        Some("antigravity") => "Antigravity",
        _ => "this agent",
    }
}

/// Why a thread runs here, in words (`local` in `renderer/src/where-why.ts`).
fn local(why: &str, detail: Option<&str>) -> String {
    match why {
        "kept" => "You chose to keep this thread on this computer.".to_string(),
        "handed-back" => match detail {
            Some(d) if merging_into_main(d) => "Merging into main happens on this computer.".to_string(),
            Some(d) => format!("Its cloud run needed {d}, which only this computer has."),
            None => "Its cloud run needed something only this computer has.".to_string(),
        },
        "cloud-refused" => "The cloud couldn’t start it, so it runs here instead.".to_string(),
        "codex-reply" => "Codex on this computer has this thread open, so its reply runs here.".to_string(),
        "project" => "The cloud doesn’t run threads from this project.".to_string(),
        "folder" => "It works in a folder that stays on this computer.".to_string(),
        "no-account" => format!("No {} account is signed in to the cloud.", engine_name(detail)),
        "cloud-failing" => "The cloud’s last runs failed before they started, so new threads run here for now.".to_string(),
        "cloud-no-room" => match detail {
            Some("hours") => "This month’s cloud hours are used up.",
            Some("agents") => "Every cloud agent on your plan was busy when it started.",
            Some("paused") => "The cloud wasn’t starting new runs when it started.",
            Some("terms") => "The cloud’s new terms need accepting first.",
            _ => "Your cloud plan has no agents to run it.",
        }
        .to_string(),
        "version" => format!("The cloud has a different version of {} from this computer.", engine_name(detail)),
        "cloud-account-limited" => "The cloud’s account is at its usage limit, so this runs on the account this computer uses.".to_string(),
        "started-before" => "It started before the cloud was set to Everything.".to_string(),
        _ => "The cloud couldn’t take it when it started.".to_string(),
    }
}

/// The body of `whyWords`: why it waits or runs where it does.
fn why_words(w: &Why) -> String {
    let detail = w.detail.as_deref();
    match w.at.as_str() {
        "wait-cloud" => match w.why.as_str() {
            "starting-in-turn" => "The cloud starts threads a few at a time, and this one is in line.".to_string(),
            "cloud-full" => format!("The cloud is running all {}it can, so this starts when one finishes.", detail.map(|d| format!("{d} threads ")).unwrap_or_default()),
            "accounts-limited" => "Your cloud accounts are at their usage limit, so this starts when one resets.".to_string(),
            _ => "It starts as soon as the cloud has room.".to_string(),
        },
        "wait-mac" => format!("{} It starts when this computer has room.", local(&w.why, detail)),
        _ if w.why == "moving" => "It keeps working here and moves as soon as the cloud has room, picking up where it left off.".to_string(),
        _ => local(&w.why, detail),
    }
}

/// A reply under way here while the thread may use the cloud: where it finishes, and where the
/// next starts (`finishesHere`).
fn finishes_here(why: Option<&Why>) -> String {
    let Some(why) = why else {
        return "This reply finishes on this Mac.".to_string();
    };
    if why.why == "kept" {
        return "This reply finishes on this Mac, and the next one starts in the cloud.".to_string();
    }
    if why.why == "handed-back" {
        if let Some(detail) = why.detail.as_deref().filter(|d| !merging_into_main(d)) {
            return format!("This reply needs {detail}, so it finishes on this Mac. The next one starts in the cloud.");
        }
    }
    let body = why_words(why);
    if FOR_NOW.contains(&why.why.as_str()) { format!("{body} This reply finishes on this Mac.") } else { format!("{body} This reply finishes on this Mac, and the next one starts in the cloud.") }
}

/// The panel's two places, its words, and what the closed button names (`placeView`).
pub fn view(facts: &Facts) -> View {
    let mac = |note: &str, pick_cloud: Option<Act>| View { on_cloud: false, now_cloud: false, off: false, dot: false, note: note.to_string(), needs: None, pick_cloud, pick_mac: None };
    let cloud = |note: &str| View { on_cloud: true, now_cloud: true, off: false, dot: false, note: note.to_string(), needs: None, pick_cloud: None, pick_mac: Some(Act::Keep(true)) };
    let moving = facts.why.as_ref().is_some_and(|w| w.why == "moving");
    let mut v = match facts.place {
        Place::Off => View { off: true, ..mac("The cloud is off. Pick Cloud to turn it on.", Some(Act::CloudOn)) },
        Place::Cloud => cloud("Running in the cloud now. Pick This Mac and its work comes back here and carries on."),
        Place::Move if moving => cloud(MOVING),
        Place::Move => mac("Working on this Mac now. Pick Cloud and it carries on there from where it is.", Some(Act::Move)),
        Place::Kept => mac("You kept it here. Its replies run on this Mac until you pick Cloud.", Some(Act::Keep(false))),
        Place::Needs => {
            let needs = facts.needs.as_deref().map_or_else(|| "Something only this Mac has".to_string(), crate::rules::js::upper_first);
            View { dot: true, needs: Some(needs), ..mac("Its replies run here until you pick Cloud.", Some(Act::Keep(false))) }
        }
        Place::Here if !facts.running => cloud("Its next reply starts in the cloud, so it keeps going with your Mac shut."),
        Place::Here if moving => cloud(MOVING),
        Place::Here => match facts.why.as_ref().filter(|w| STAYS.contains(&w.why.as_str())) {
            Some(why) => mac(&why_words(why), None),
            None => cloud(&finishes_here(facts.why.as_ref())),
        },
    };
    v.now_cloud = if facts.running && facts.place != Place::Cloud { false } else { v.on_cloud };
    v
}

/// The facts a pick leads to, drawn until the engine's own arrive (`afterPick`).
pub fn after_pick(facts: &Facts, act: Act) -> Facts {
    match act {
        Act::Move => Facts { place: Place::Move, why: Some(Why { at: "mac".to_string(), why: "moving".to_string(), detail: None }), ..facts.clone() },
        Act::CloudOn => Facts { place: Place::Here, running: false, why: None, ..facts.clone() },
        Act::Keep(keep) => Facts { place: if keep { Place::Kept } else { Place::Here }, needs: None, ..facts.clone() },
    }
}

/// What the engine says about a thread's place, in one line: a pick is drawn until this changes
/// (`said` in `PlaceButton`).
pub fn said(facts: &Facts) -> String {
    let why = facts.why.as_ref().map(|w| format!("{}:{}", w.at, w.why)).unwrap_or_default();
    format!("{:?}|{}|{why}|{:?}", facts.place, facts.running, facts.needs)
}

/// How long a pick is drawn while the engine says nothing new (`PICK_SHOWS_MS`).
pub const PICK_SHOWS_MS: f64 = 4000.0;

/// The closed button for a view: where the thread runs now, which is this Mac for a reply under
/// way here whatever the setting. A cloud where it runs in the cloud, or is set to and the cloud
/// is off, faded then; a Mac otherwise, with a dot where the cloud handed it back.
pub fn mark_of(v: &View) -> Mark {
    Mark { cloud: v.now_cloud || v.off, faded: v.off, dot: v.dot }
}

/// Where the cloud goes when Cloud is picked with the cloud off: back to what it was set to
/// when on (`cloudToggleTarget` in `renderer/src/cloud-menu.ts`).
fn cloud_on_target(snap: &Snapshot) -> &'static str {
    let cloud = snap.rest.get("cloud");
    if cloud.and_then(|c| c.get("whenOn")).and_then(Value::as_str) == Some("all") { "all" } else { "full" }
}

/// What a pick writes, in order (`placeThread` and `moveToCloud` in `renderer/src/App.tsx`):
/// keeping it here or letting the cloud have it is the thread's own setting; moving a run asks
/// the engine to carry it there as the cloud has room, naming its project as the phone's
/// command needs; turning the cloud on sets the workspace back to what it ran when on, and
/// lets this thread go there.
pub fn writes(snap: &Snapshot, item: &Item, act: Act) -> Vec<(&'static str, Value)> {
    let on_mac = |keep: bool| ("zero:set-thread-on-mac", json!({ "onMac": { item.id.clone(): keep } }));
    match act {
        Act::Keep(keep) => vec![on_mac(keep)],
        Act::Move => vec![("zero:move-to-cloud", json!({ "ids": [item.id], "product": item.product, "id": item.id }))],
        Act::CloudOn => vec![("zero:settings-set-workspace", json!({ "key": "cloudRuns", "value": cloud_on_target(snap) })), on_mac(false)],
    }
}

/// How many runs the engine says will move (`moveToCloud` in `renderer/src/api.ts`): its count,
/// else every one asked for when it only says yes.
pub fn moving(answer: &Value, asked: usize) -> usize {
    answer.get("moving").and_then(Value::as_u64).map_or_else(|| if answer.get("ok") == Some(&Value::Bool(true)) { asked } else { 0 }, |n| n as usize)
}

/// What the window says when the cloud takes none of it (`moveToCloud`, a pick in the panel
/// otherwise saying nothing: the panel shows it).
pub const CLOUD_CANNOT_TAKE: &str = "The cloud can’t take this thread right now";

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// What the button shows for a place, as the strip reads it with no pick standing.
    fn mark(snap: &Snapshot, item: &Item, place: Place) -> Mark {
        mark_of(&view(&facts(snap, item, place)))
    }

    fn thread(extra: Value) -> Item {
        let mut v = json!({ "id": "w-1", "product": "shop", "status": "open", "labels": ["founder"] });
        for (k, value) in extra.as_object().unwrap() {
            v[k] = value.clone();
        }
        serde_json::from_value(v).unwrap()
    }

    fn snap(cloud: Value, supervisor: Value) -> Snapshot {
        serde_json::from_value(json!({ "products": [{ "slug": "shop", "name": "Storefront" }], "items": [], "supervisor": supervisor, "cloud": cloud })).unwrap()
    }

    fn cloud_on() -> Value {
        json!({ "setUp": true, "managed": false, "mode": "full", "projects": ["shop"], "movable": [], "noCloudSignIn": [] })
    }

    #[test]
    fn a_thread_has_no_place_where_the_cloud_could_never_run_it() {
        let quiet = json!({ "running": [] });
        assert_eq!(thread_place(&snap(Value::Null, quiet.clone()), &thread(json!({}))), None);
        assert_eq!(thread_place(&snap(json!({ "setUp": false }), quiet.clone()), &thread(json!({}))), None);
        // The managed cloud with no plan is no cloud.
        assert_eq!(thread_place(&snap(json!({ "setUp": true, "managed": true, "plan": "none", "mode": "full", "projects": ["shop"] }), quiet.clone()), &thread(json!({}))), None);
        // A project the cloud does not take.
        assert_eq!(thread_place(&snap(cloud_on(), quiet.clone()), &thread(json!({ "product": "desk" }))), None);
        // A finished thread runs nowhere.
        assert_eq!(thread_place(&snap(cloud_on(), quiet.clone()), &thread(json!({ "status": "done" }))), None);
        // An agent the cloud has no sign-in for.
        let mut no_codex = snap(json!({ "setUp": true, "mode": "full", "projects": ["shop"], "noCloudSignIn": ["codex"] }), quiet);
        no_codex.rest.insert("engines".into(), json!({ "workspace": "codex", "byItem": {} }));
        assert_eq!(thread_place(&no_codex, &thread(json!({}))), None);
    }

    #[test]
    fn a_thread_the_cloud_takes_is_here_kept_moving_needing_or_there() {
        let quiet = json!({ "running": [] });
        assert_eq!(thread_place(&snap(cloud_on(), quiet.clone()), &thread(json!({}))), Some(Place::Here));
        assert_eq!(thread_place(&snap(cloud_on(), quiet.clone()), &thread(json!({ "onMac": true }))), Some(Place::Kept));
        assert_eq!(thread_place(&snap(cloud_on(), quiet.clone()), &thread(json!({ "needsMac": "your Stripe key" }))), Some(Place::Needs));
        let there = json!({ "running": [{ "itemId": "w-1", "product": "shop", "where": "cloud" }] });
        assert_eq!(thread_place(&snap(cloud_on(), there), &thread(json!({}))), Some(Place::Cloud));
        let movable = json!({ "running": [{ "itemId": "w-1", "product": "shop", "movable": true }] });
        assert_eq!(thread_place(&snap(Value::Null, movable), &thread(json!({}))), Some(Place::Move));
        let off = json!({ "setUp": true, "mode": "mac", "projects": ["shop"] });
        assert_eq!(thread_place(&snap(off, quiet), &thread(json!({}))), Some(Place::Off));
    }

    #[test]
    fn the_button_names_where_the_thread_runs_now() {
        let quiet = snap(cloud_on(), json!({ "running": [] }));
        let item = thread(json!({}));
        // Its next reply starts in the cloud.
        assert_eq!(mark(&quiet, &item, Place::Here), Mark { cloud: true, faded: false, dot: false });
        assert_eq!(mark(&quiet, &item, Place::Kept), Mark { cloud: false, faded: false, dot: false });
        assert_eq!(mark(&quiet, &item, Place::Needs), Mark { cloud: false, faded: false, dot: true });
        // The whole cloud is off: a faded cloud.
        assert_eq!(mark(&quiet, &item, Place::Off), Mark { cloud: true, faded: true, dot: false });
        // A reply under way here is on this Mac whatever the setting.
        let here = snap(cloud_on(), json!({ "running": [{ "itemId": "w-1", "product": "shop" }] }));
        assert_eq!(mark(&here, &item, Place::Here), Mark { cloud: false, faded: false, dot: false });
        let there = snap(cloud_on(), json!({ "running": [{ "itemId": "w-1", "product": "shop", "where": "cloud" }] }));
        assert_eq!(mark(&there, &item, Place::Cloud), Mark { cloud: true, faded: false, dot: false });
    }

    fn facts_of(place: Place, running: bool, why: Option<(&str, &str, Option<&str>)>) -> Facts {
        Facts { place, running, why: why.map(|(at, why, detail)| Why { at: at.to_string(), why: why.to_string(), detail: detail.map(str::to_string) }), needs: None }
    }

    // The cases of `tests/the-runs-on-button-names-this-mac-while-a-reply-runs-here.test.mts`.
    #[test]
    fn the_panel_keeps_the_setting_and_the_button_names_this_mac_while_a_reply_runs_here() {
        let refused = view(&facts_of(Place::Here, true, Some(("mac", "cloud-refused", None))));
        assert!(refused.on_cloud && !refused.now_cloud && refused.pick_mac == Some(Act::Keep(true)));
        assert!(refused.note.contains("This reply finishes on this Mac"), "{}", refused.note);
        assert!(!view(&facts_of(Place::Here, true, Some(("mac", "handed-back", Some("merging into main"))))).now_cloud);
        assert!(!view(&facts_of(Place::Here, true, None)).now_cloud);
        assert!(!view(&facts_of(Place::Move, true, Some(("mac", "moving", None)))).now_cloud);
        assert!(!view(&after_pick(&facts_of(Place::Move, true, None), Act::Move)).now_cloud);
        assert!(view(&facts_of(Place::Cloud, false, None)).now_cloud);
        assert!(view(&facts_of(Place::Here, false, None)).now_cloud);
        let kept = view(&facts_of(Place::Kept, false, None));
        assert!(!kept.now_cloud && kept.pick_cloud == Some(Act::Keep(false)) && kept.pick_mac.is_none());
        let off = view(&facts_of(Place::Off, false, None));
        assert!(off.off && off.pick_cloud == Some(Act::CloudOn));
    }

    #[test]
    fn a_reason_the_cloud_cannot_take_it_leaves_cloud_unpicked_and_says_why() {
        let folder = view(&facts_of(Place::Here, true, Some(("mac", "folder", None))));
        assert_eq!((folder.on_cloud, folder.pick_cloud, folder.note.as_str()), (false, None, "It works in a folder that stays on this computer."));
        let handed = view(&facts_of(Place::Here, true, Some(("mac", "handed-back", Some("your Stripe key")))));
        assert_eq!(handed.note, "This reply needs your Stripe key, so it finishes on this Mac. The next one starts in the cloud.");
        let failing = view(&facts_of(Place::Here, true, Some(("mac", "cloud-failing", None))));
        assert_eq!(failing.note, "The cloud’s last runs failed before they started, so new threads run here for now. This reply finishes on this Mac.");
        let needs = view(&Facts { needs: Some("your Stripe key".to_string()), ..facts_of(Place::Needs, false, None) });
        assert_eq!((needs.dot, needs.needs.as_deref()), (true, Some("Your Stripe key")));
    }

    #[test]
    fn a_pick_writes_the_threads_setting_moves_its_run_or_turns_the_cloud_on() {
        let item = thread(json!({}));
        let quiet = snap(cloud_on(), json!({ "running": [] }));
        assert_eq!(writes(&quiet, &item, Act::Keep(true)), vec![("zero:set-thread-on-mac", json!({ "onMac": { "w-1": true } }))]);
        assert_eq!(writes(&quiet, &item, Act::Keep(false)), vec![("zero:set-thread-on-mac", json!({ "onMac": { "w-1": false } }))]);
        assert_eq!(writes(&quiet, &item, Act::Move), vec![("zero:move-to-cloud", json!({ "ids": ["w-1"], "product": "shop", "id": "w-1" }))]);
        let off = snap(json!({ "setUp": true, "mode": "mac", "whenOn": "all", "projects": ["shop"] }), json!({ "running": [] }));
        assert_eq!(writes(&off, &item, Act::CloudOn)[0], ("zero:settings-set-workspace", json!({ "key": "cloudRuns", "value": "all" })));
        assert_eq!(moving(&json!({ "moving": 0 }), 1), 0);
        assert_eq!(moving(&json!({ "ok": true }), 1), 1);
        // A pick is drawn until the engine says anything new about the thread.
        let here = facts_of(Place::Here, false, None);
        assert_ne!(said(&here), said(&after_pick(&here, Act::Keep(true))));
    }
}
