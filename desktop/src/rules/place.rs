//! Where one thread runs, for the button beside Done over a conversation: this Mac or the
//! cloud. `thread_place` follows `threadPlace` in `renderer/src/cloud-menu.ts`, and `mark`
//! follows `placeView` in `renderer/src/threads/PlaceButton.tsx`.

use serde_json::{Map, Value};

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

/// What the button shows for a place: where the thread runs now, which is this Mac for a reply
/// under way here whatever the setting.
pub fn mark(snap: &Snapshot, item: &Item, place: Place) -> Mark {
    // No reply of the thread can go to the cloud while one of these holds (`STAYS`).
    const STAYS: [&str; 6] = ["folder", "project", "teammate", "codex-reply", "no-account", "version"];
    let running = session(snap, item).is_some_and(|r| !in_the_cloud(r));
    let why = snap.supervisor.rest.get("whyNotCloud").and_then(|w| w.get(&item.id)).and_then(|w| w.get("why")).and_then(Value::as_str);
    let moving = why == Some("moving");
    let set_to_cloud = match place {
        Place::Cloud => true,
        Place::Move => moving,
        Place::Off | Place::Kept | Place::Needs => false,
        Place::Here => !running || moving || !why.is_some_and(|w| STAYS.contains(&w)),
    };
    let now_cloud = if running && place != Place::Cloud { false } else { set_to_cloud };
    Mark { cloud: now_cloud || place == Place::Off, faded: place == Place::Off, dot: place == Place::Needs }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}
