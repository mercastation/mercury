//! Which thread goes in which list, in what order, and with which words: the web window's own
//! rules, ported from its TypeScript one function at a time, so the native window shows the
//! same rows. `lists` is the chain of lists in `renderer/src/App.tsx` (the inbox, In progress,
//! Later and Done); `row` is what Home and the Inbox table's `ThreadCells`
//! (`renderer/src/threads/Pages.tsx`) say about one thread.
//!
//! One person, with no team: where a team's rule would hook in is said in a comment at the
//! place. A terminal agent the app did not start is not listed either (`agentRows` in
//! `App.tsx`): the web window lists one unless the setting turns it off (`outsideAgentsMode`
//! in `main/settings.mts`, on by default), so with that setting on this window is short of
//! those rows. `tools/rows.mjs` holds these lists against the rows the web window draws from
//! the same snapshot.

pub mod account_menu;
pub mod answering;
pub mod answers;
pub mod attach;
pub mod calendar;
pub mod change;
pub mod close_asks;
pub mod cloud;
pub mod code;
pub mod compose;
pub mod display;
pub mod e_and_z;
pub mod engine;
pub mod filed_in_thread;
pub mod files_home;
pub mod files_shown;
pub mod film;
pub mod folded;
pub mod format;
pub mod found;
pub mod js;
pub mod key_asks;
pub mod kinds;
pub mod later;
pub mod list;
pub mod live;
pub mod mentions;
pub mod motion;
pub mod needs;
pub mod notes;
pub mod nowhere;
pub mod page;
pub mod palette;
pub mod pane;
pub mod paths;
pub mod place;
pub mod plan_setup;
pub mod project_folder;
pub mod rank;
pub mod recent;
pub mod reply_chips;
pub mod reply_pick;
pub mod roots;
pub mod search;
pub mod settings;
pub mod ship_offer;
pub mod shipping;
pub mod sidebar;
pub mod sign_in;
pub mod status;
pub mod table_edit;
pub mod table_file;
pub mod thread_files;
pub mod threads_made;
pub mod title;
pub mod trace;
pub mod trail;
pub mod trouble_row;
pub mod typed;
pub mod update;
pub mod usage;
pub mod viewer;
pub mod waits;
pub mod window;
pub mod work;
pub mod writes;

use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value};

use crate::data::{Item, Lists, Local, Mark, Priority, Row, Snapshot, Tab};

/// Everything the lists read besides one thread: the snapshot, the clock, what the window
/// keeps for itself, and what the supervisor says about every thread at once.
struct World<'a> {
    snap: &'a Snapshot,
    now: f64,
    local: &'a Local,
    /// The newest row of each id (`byId` in `App.tsx`).
    by_id: HashMap<&'a str, &'a Item>,
    /// Threads with a run on them, or with one being started (`agentOnOrComing`).
    live: HashSet<&'a str>,
    /// Threads whose agent is stopped on a permission request or waits on a sign-in.
    asking: HashSet<&'a str>,
    /// Threads the app still has to ship.
    shipping: HashSet<&'a str>,
    /// Threads nothing will start again on its own: the supervisor stopped them, or they wait
    /// on a sign-in (`stoppedIds` in `renderer/src/fleet.ts`).
    stalled: HashSet<&'a str>,
    queued: HashSet<&'a str>,
    /// The sign-ins and keys the cloud already keeps, by variable name (`savedCloudKeys`).
    saved_keys: Vec<String>,
}

fn ids(list: Option<&Value>) -> impl Iterator<Item = &str> {
    list.and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str)
}

impl<'a> World<'a> {
    fn new(snap: &'a Snapshot, now: f64, local: &'a Local) -> World<'a> {
        let supervisor = &snap.supervisor.rest;
        // Each approval in the spool names the thread it came from.
        let approvals = snap.rest.get("approvals").and_then(Value::as_array).into_iter().flatten().filter_map(|a| a.get("item").and_then(Value::as_str)).filter(|id| !id.is_empty());
        let cloud = snap.rest.get("cloud").filter(|c| crate::data::truthy(c.get("setUp")));
        World {
            snap,
            now,
            local,
            by_id: snap.items.iter().map(|i| (i.id.as_str(), i)).collect(),
            live: snap.supervisor.running.iter().map(|r| r.item_id.as_str()).chain(ids(supervisor.get("starting"))).collect(),
            asking: approvals.chain(snap.items.iter().filter(|i| kinds::open_sign_in(i)).map(|i| i.id.as_str())).collect(),
            shipping: ids(supervisor.get("shipping")).collect(),
            stalled: ids(supervisor.get("stalled")).chain(supervisor.get("signInNeeded").and_then(Value::as_object).into_iter().flat_map(Map::keys).map(String::as_str)).collect(),
            queued: ids(supervisor.get("queued")).collect(),
            saved_keys: cloud.and_then(|c| c.get("keys")).and_then(Value::as_array).into_iter().flatten().filter_map(|k| k.get("name").and_then(Value::as_str)).map(str::to_string).collect(),
        }
    }

    /// When the person last opened the row, or when opened times started being kept.
    fn seen(&self, i: &Item) -> f64 {
        self.local.seen_at.get(&i.id).copied().unwrap_or(self.local.unread_since)
    }

    /// The threads this one waits on that can still finish.
    fn waiting_on(&self, i: &Item) -> Vec<String> {
        list::waits_that_can_end(i, &self.by_id, &HashSet::from([i.id.clone()]))
    }

    /// The moment the person put it away until, or its agent will run it again (`hiddenAt`).
    fn hidden_at(&self, i: &Item) -> f64 {
        list::hidden_until(i, self.local.snooze.get(&i.id).copied().unwrap_or(0.0))
    }

    /// `score` in `App.tsx`: what matters most, the same number the supervisor spawns by. On a
    /// team a thread just handed back borrows the top project's place; that would go here.
    fn score(&self, i: &Item) -> f64 {
        rank::score(&self.snap.supervisor.product_order, i)
    }

    /// Every ordered list sorts through this, so the inbox, In progress and the fleet agree on
    /// which row comes next (`byRunningOrder`). Recency only breaks ties.
    fn by_running_order(&self, rows: &mut [&Item]) {
        let larger_first = |a: f64, b: f64| b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal);
        rows.sort_by(|a, b| larger_first(self.score(a), self.score(b)).then_with(|| larger_first(rank::placed_at(a), rank::placed_at(b))));
    }

    /// The inbox's own answer for a row (`inboxCandidates` in `App.tsx`): the view's business
    /// first, then the one membership rule. A team's branches (a thread out with someone asked
    /// for help, file news, a shared project's rows) sit between the two.
    fn in_inbox(&self, i: &Item, owed: &HashSet<&str>, hidden_at: f64) -> bool {
        let id = i.id.as_str();
        if list::words_wait(i, self.seen(i)) {
            return true;
        }
        // An agent is working on it right now, or the app is still to ship it: In progress,
        // whatever it says. A row still asking something is never hidden by either.
        let held = self.live.contains(id) || self.shipping.contains(id);
        if held && !list::still_asks_you(i) && !self.asking.contains(id) {
            return false;
        }
        if owed.contains(id) || self.asking.contains(id) {
            return true;
        }
        let waiting_on = self.waiting_on(i);
        list::belongs_in_inbox(i, &list::Facts {
            delivered_through: self.local.delivered_through,
            hidden_until: hidden_at,
            now: self.now,
            asking: false,
            waiting_on: &waiting_on,
            stopped: self.stalled.contains(id),
            seen: self.seen(i),
            ..list::Facts::default()
        })
    }
}

/// The row the app makes itself when tasks have not been able to run, shaped as a thread so
/// everything downstream treats it as one (`troubleRow` in `renderer/src/trouble-row.ts`).
/// Both its times are when the pile began. Its body is what the opened row says; a list prints
/// its first line.
fn trouble_row(snap: &Snapshot, local: &Local) -> Option<Item> {
    let t = snap.supervisor.rest.get("spawnTrouble")?.as_object()?;
    // It goes quiet when the person closes it, until the pile changes.
    if t.get("since").and_then(Value::as_f64) == Some(local.trouble_closed) {
        return None;
    }
    Some(trouble_row::trouble_row(t, &snap.items))
}

/// A task asked for in a chat answers in that chat: while its answer waits, Needs you lists
/// the chat and not the task (`answersInTheirChat` in `renderer/src/team/answers-in-the-chat.ts`).
/// Returns the tasks taken out and the chats put in, by `product/id`.
fn answers_in_their_chat<'a>(w: &World<'a>, own: &HashSet<usize>) -> (HashSet<String>, HashSet<String>) {
    let key = |product: &str, id: &str| format!("{product}/{id}");
    let here: HashSet<String> = w.snap.items.iter().map(|i| key(&i.product, &i.id)).collect();
    let (mut hide, mut lift) = (HashSet::new(), HashSet::new());
    for (at, i) in w.snap.items.iter().enumerate() {
        let Some((product, id)) = kinds::asked_in(i) else { continue };
        // A task only its own row can answer stays: a permission card, a run that will not start again.
        let own_row = w.asking.contains(i.id.as_str()) || w.stalled.contains(i.id.as_str());
        if !here.contains(&key(product, id)) || own_row || !own.contains(&at) {
            continue;
        }
        hide.insert(key(&i.product, &i.id));
        lift.insert(key(product, id));
    }
    (hide, lift)
}

/// A thread an agent filed that still waits on a press from the person (`approvableFiled` in
/// `renderer/src/threads-made.ts`): sitting in Needs you or Later, open, not a question, not
/// the person's own, begun, and unanswered.
fn approvable_filed(i: &Item, in_needs_you_or_later: bool) -> bool {
    in_needs_you_or_later && i.status == "open" && !kinds::asks_the_person(i) && !answers::filed_by_the_person(i) && !kinds::not_started(i) && i.live_answer().is_none()
}

/// Comments on a thread's shared links, as one line on its row however many there are
/// (`linkCommentLine` in `renderer/src/threads/teammate-thread.ts`).
fn link_comment_line(comments: &[Value]) -> Option<String> {
    let text = |c: &Value, k: &str| c.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let last = comments.last()?;
    let place = last.get("place").filter(|p| p.get("kind").and_then(Value::as_str) == Some("time"));
    let at = place.and_then(|p| p.get("s")).and_then(Value::as_f64).map_or(String::new(), |s| format!(" at {}:{:02}", (s.floor() / 60.0).floor(), s.floor() % 60.0));
    if comments.len() == 1 {
        return Some(format!("{} commented on {}{at}: {}", text(last, "name"), text(last, "file"), text(last, "text")));
    }
    let files: HashSet<String> = comments.iter().map(|c| text(c, "file")).collect();
    let mut people: Vec<String> = comments.iter().map(|c| text(c, "name")).collect::<HashSet<_>>().into_iter().collect();
    people.sort();
    let who = match people.split_last() {
        Some((last, rest)) if people.len() > 2 => format!("{} and {last}", rest.join(", ")),
        _ => people.join(" and "),
    };
    let on = if files.len() == 1 { text(last, "file") } else { format!("{} files", files.len()) };
    Some(format!("{} comments on {on} from {who}: {}", comments.len(), text(last, "text")))
}

/// What one thread's row says under one tab: Home's row (`homeNeeds` and `homeProgress` in
/// `App.tsx`) and the Inbox table's cells (`ThreadCells`), which agree wherever both speak.
///
/// What the web window's row adds that is not here: on a team, who the row is from, a lock on
/// a thread only the person sees, and a message's or a given task's own shape; an AI
/// teammate's face and name on its runs; and the demand a thread serves after its title,
/// which comes on a channel of its own and is not in the snapshot.
fn row(w: &World, i: &Item, tab: &str, to_approve: &HashMap<String, usize>) -> Row {
    let id = i.id.as_str();
    let product = w.snap.products.iter().find(|p| p.slug == i.product);
    let engine = engine::row_engine(i, w.snap);
    let title = title::row_title(i);
    let in_needs_you = tab == "inbox";

    // What is happening on it, for the Status column.
    let facts = status::Facts {
        supervisor: &w.snap.supervisor,
        // The newest run of this id, as a map keyed by id keeps it.
        session: w.snap.supervisor.running.iter().rev().find(|r| r.item_id == i.id),
        asking: w.asking.contains(id),
        in_progress: tab == "progress",
        practice: product.is_some_and(|p| crate::data::truthy(p.rest.get("practice"))),
        later: tab == "snoozed",
        now: w.now,
    };
    let cell = status::status_cell(i, &facts);
    // Comments on its links hold it in Needs you whatever the thread itself says. A teammate's
    // words do the same on a team, and a wait on a person has words of its own; both go here.
    let people_wait = in_needs_you && !cell.live && !i.list("linkComments").is_empty();
    let (word, worded_mark) = match people_wait {
        // The solid dot, where the cell had a mark to replace.
        true => ("Waiting on you", if cell.mark == Mark::None { Mark::None } else { Mark::Need }),
        false => (cell.word.as_str(), cell.mark),
    };
    // A cloud run that asked for a key is waiting on it, not finished.
    let wants = if in_needs_you && !cell.live && !people_wait { needs::needs_word(i, &w.saved_keys) } else { None };
    let shown = wants.or_else(|| page::cell_under_tab(word, tab, people_wait, page::asks_for_your_words(i)));

    // One row per cause: how many threads wait behind it, and from where.
    let behind = (kinds::is_cause(i) && i.status != "done").then(|| kinds::waiting_on_cause(i, &w.snap.items));
    // And on Later a thread behind one says what it waits on, and that it comes back when that
    // is fixed.
    let stuck_on = if behind.is_none() && tab == "snoozed" { kinds::cause_of(i, &w.snap.items) } else { None };
    // Either takes the word's place and keeps the mark the row would have worn; a row whose
    // word the tab dropped keeps the mark of the cell it started from.
    let in_its_place = behind.as_ref().map(|behind| kinds::cause_word(behind.len())).or(stuck_on.map(|_| kinds::CAUSE_WAIT_WORD.to_string()));
    // What the word says under the pointer goes with it wherever it goes, and with no word there
    // is nothing to rest on. Try again follows the word on a thread's own row, and not on a
    // cause's or one behind a cause (`RowCells`' `retry`, passed on the last of them only).
    let (status_title, why) = if in_its_place.is_some() || shown.is_some() { (cell.title.clone(), cell.why.clone()) } else { (None, None) };
    let retry = cell.retry && behind.is_none() && stuck_on.is_none();
    let (status, mark) = match (in_its_place, shown) {
        (Some(word), Some(_)) => (word, worded_mark),
        (Some(word), None) => (word, cell.mark),
        (None, Some(word)) => (word, worded_mark),
        (None, None) => (String::new(), Mark::None),
    };
    let (place, place_more, aside, held, pinned) = match (&behind, stuck_on) {
        (Some(behind), _) => {
            // A project this window does not know is said by its slug, and the cause's own by nothing.
            let name_of = |slug: &str| match w.snap.products.iter().find(|p| p.slug == slug) {
                Some(p) => p.name.clone(),
                None if slug == i.product => String::new(),
                None => slug.to_string(),
            };
            let (name, more) = kinds::cause_place(i, behind, name_of);
            (name, more, None, false, i.truthy("pinned"))
        }
        // The cause's title as it was filed, with no pin and no "not started" beside it.
        (None, Some(cause)) => (product.map_or(String::new(), |p| p.name.clone()), 0, cause.title.clone().filter(|t| !t.is_empty()), false, false),
        (None, None) => {
            // The faint words after the title: a comment on a link, a wait that ended, a
            // hand-over's words, or how many tasks it filed still wait for a yes. A teammate's
            // note and their comment come first on a team.
            let said = |field: &str| i.object(field).and_then(|o| o.get("said")).and_then(Value::as_str).map(str::to_string);
            let commented = link_comment_line(i.list("linkComments")).or_else(|| if list::wait_came_back(i) { said("waitEnded") } else { None }).or_else(|| said("handedOver"));
            let waiting = to_approve.get(&format!("{}/{}", i.product, i.id)).copied().unwrap_or(0);
            let words = commented.or_else(|| (waiting > 0).then(|| threads_made::to_approve_words(waiting)));
            (product.map_or(String::new(), |p| p.name.clone()), 0, list::aside_besides(&title, words.as_deref()), kinds::not_started(i), page::pinned_open(i))
        }
    };

    let at = page::sent_at(i);
    // On Done the time is when the thread was finished, which is its order. On Later it is
    // when the thread comes back, not how long ago it was touched: nothing for one that waits
    // on a person's go and not on a clock, and for one behind a cause, when that is fixed.
    let (shown_at, when) = match tab {
        "done" => (page::finished_at(i), page::updated_words(page::finished_at(i), w.now)),
        "snoozed" if stuck_on.is_some() => (0.0, kinds::CAUSE_BACK_WORDS.to_string()),
        "snoozed" => (page::back_at(i), page::back_words(i, w.now)),
        _ => (at, page::updated_words(at, w.now)),
    };
    let trouble = (id == nowhere::TROUBLE_ID).then(|| i.body.as_deref().unwrap_or(""));
    // Who said it leads the line only for a person or a teammate, whom this window has none of.
    let line = list::line_for(&title, &list::row_summary(i, tab, trouble), None).map(|l| l.text).unwrap_or_default();
    Row {
        id: i.id.clone(),
        product: i.product.clone(),
        letter: engine::engine_letter(engine),
        title,
        project: i.product_name.clone().unwrap_or_default(),
        priority: Priority::of(i.priority),
        at: Some(at),
        line,
        status,
        // Unread while its latest answer is newer than when it was last opened (`isUnread` in
        // `renderer/src/unread.ts`). Only Needs you draws it.
        unread: in_needs_you && i.wrote("result").and_then(|w| w.ts).or(i.created_at).unwrap_or(0.0) > w.seen(i),
        pinned,
        mark,
        status_title,
        why,
        retry,
        aside: aside.unwrap_or_default(),
        held,
        place,
        place_more,
        engine: engine::engine_label(engine),
        harness: engine::harness_mark(engine::harness_on(i, w.snap)),
        when,
        shown_at,
        kept: true,
    }
}

/// Every list at once, as of `now`, in the order the page opens on: by priority.
pub fn lists(snap: &Snapshot, now: f64, local: &Local) -> Lists {
    lists_sorted(snap, now, local, display::Sort::Priority)
}

/// Every list at once, Needs you and Later run by `sort` (the View and filters menu's Sort by;
/// In progress and Done keep their own order whatever it says). Nothing is filtered yet: that
/// is `display::show`, laid on once whatever else thins the lists has.
pub fn lists_sorted(snap: &Snapshot, now: f64, local: &Local, sort: display::Sort) -> Lists {
    let w = World::new(snap, now, local);
    let items = &snap.items;
    let order = &snap.supervisor.product_order;
    let hidden_at: Vec<f64> = items.iter().map(|i| w.hidden_at(i)).collect();
    let trouble = trouble_row(snap, local);

    // A thread whose proposals have been waiting a day comes back.
    let owed = list::threads_owed_an_answer(items, &w.by_id, now);
    let own: HashSet<usize> = (0..items.len()).filter(|at| w.in_inbox(&items[*at], &owed, hidden_at[*at])).collect();
    let (hide, lift) = answers_in_their_chat(&w, &own);
    let mut rows: Vec<&Item> = items
        .iter()
        .enumerate()
        .filter(|(at, i)| {
            let key = format!("{}/{}", i.product, i.id);
            !hide.contains(&key) && (own.contains(at) || (lift.contains(&key) && hidden_at[*at] <= now))
        })
        .map(|(_, i)| i)
        .collect();
    w.by_running_order(&mut rows);
    // What matters most, top down. Above all of it the row that says nothing is running, then
    // a conversation just imported, then everything urgent, lifted out of the scored block
    // because a project's place outweighs any level. On a team what is past its deadline is
    // lifted above the urgent ones.
    let (fresh, rest): (Vec<&Item>, Vec<&Item>) = rows.into_iter().partition(|i| kinds::just_imported(i, &local.seen));
    let (urgent, ordinary): (Vec<&Item>, Vec<&Item>) = rest.into_iter().partition(|i| rank::is_urgent(i));
    let inbox: Vec<&Item> = trouble.iter().chain(fresh).chain(urgent).chain(ordinary).collect();
    let needs_you: HashSet<&str> = inbox.iter().map(|i| i.id.as_str()).collect();

    // In progress is a promise that a worker is on this or about to be. On a team it also
    // holds what is moving for the person elsewhere; those branches sit before the rule.
    let mut progress: Vec<&Item> = items
        .iter()
        .zip(&hidden_at)
        .filter(|(i, hidden_at)| {
            let live = w.live.contains(i.id.as_str());
            if **hidden_at > now && !live {
                return false;
            }
            list::belongs_in_progress(i, &list::Facts { deferred_until: list::deferred_until(i), now, live, shipping: w.shipping.contains(i.id.as_str()), stopped: w.stalled.contains(i.id.as_str()), ..list::Facts::default() })
        })
        .map(|(i, _)| i)
        .collect();
    w.by_running_order(&mut progress);
    // The tab counts each thread once: the list, less what Needs you holds (`inProgressTab`).
    let progress_tab: Vec<&Item> = progress.iter().copied().filter(|i| !needs_you.contains(i.id.as_str())).collect();

    // Not a row that still needs the person, and not one an agent is still on.
    let mut done: Vec<&Item> = items.iter().filter(|i| i.status == "done" && !needs_you.contains(i.id.as_str()) && !w.live.contains(i.id.as_str()) && !w.shipping.contains(i.id.as_str())).collect();
    done.sort_by(|a, b| b.updated_at.unwrap_or(0.0).partial_cmp(&a.updated_at.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));

    // Later is the future inbox (`snoozed` in `App.tsx`): what was written down and not begun,
    // what an agent proposed, and what is put off or waiting with no run on it, less what the
    // person closed. An AI teammate's run queued behind another is its own queue and not in it.
    // Repeating tasks are rules on a channel of their own, and a terminal agent put off is a
    // row of another kind; the web window lists both here and this one lists neither.
    let mut later: Vec<(f64, &Item)> = items
        .iter()
        .zip(&hidden_at)
        .filter_map(|(i, hidden_at)| {
            let id = i.id.as_str();
            let queued_teammate_run = kinds::not_started(i) && title::teammate_id_of(i).is_some();
            let run_on_it = w.live.contains(id) || w.queued.contains(id) || w.shipping.contains(id);
            let waiting_on = w.waiting_on(i);
            let held = kinds::not_started(i) || list::is_proposal(i) || list::put_off_in_later(i, *hidden_at, now, &waiting_on, run_on_it);
            let closed_by_them = i.status == "done" && i.wrote("status").is_some_and(|w| w.source == "founder");
            if queued_teammate_run || !held || closed_by_them {
                return None;
            }
            // Soonest back first. What waits on a person or on another thread has no moment,
            // so it sorts after everything with a clock (`whenShown`, which says so with
            // JavaScript's largest whole number). A proposal has no moment either and is not
            // named there, so it reads as the moment nought and sorts before them all.
            let no_clock = kinds::not_started(i) || list::waits_in_later(i, &waiting_on);
            Some((if no_clock { 9_007_199_254_740_991.0 } else { *hidden_at }, i))
        })
        .collect();
    later.sort_by(|a, b| a.0.total_cmp(&b.0));
    let later: Vec<&Item> = later.into_iter().map(|(_, i)| i).collect();
    // Where each thread sits decides which of the tasks a thread filed still wait for a yes
    // (`tabState` and `waitingCounts`): Needs you wins, then In progress, Later and Done,
    // exactly as the tabs list them.
    let mut needs_you_or_later: HashMap<&str, bool> = done.iter().map(|i| (i.id.as_str(), false)).collect();
    needs_you_or_later.extend(later.iter().map(|i| (i.id.as_str(), true)));
    needs_you_or_later.extend(progress.iter().map(|i| (i.id.as_str(), false)));
    needs_you_or_later.extend(inbox.iter().map(|i| (i.id.as_str(), true)));
    let mut to_approve: HashMap<String, usize> = HashMap::new();
    for i in items {
        let Some(parent) = i.text("parent") else { continue };
        if approvable_filed(i, needs_you_or_later.get(i.id.as_str()).copied().unwrap_or(false)) {
            *to_approve.entry(format!("{}/{parent}", i.product)).or_default() += 1;
        }
    }

    let rows = |list: &[&Item], tab: &str| list.iter().map(|i| row(&w, i, tab, &to_approve)).collect::<Vec<Row>>();
    let lists = Lists {
        // Home's Needs you and the Inbox page both draw the inbox in the page's own order.
        needs: rows(&page::sorted_by(&inbox, "inbox", sort, order), "inbox"),
        // Home draws In progress in the order it will run; the page's tab re-sorts it.
        progress: rows(&progress_tab, "progress"),
        // Later keeps the page's sort, as Needs you does: pinned, the project, the level.
        later: rows(&page::sorted_by(&later, "snoozed", sort, order), "snoozed"),
        done: rows(&page::sorted(&done, "done", order), "done"),
        whole: [0; 4],
        behind: Vec::new(),
        trouble: trouble.clone(),
    };
    Lists { whole: [lists.needs.len(), lists.progress.len(), lists.later.len(), lists.done.len()], ..lists }
}

/// `--rows <out.json>`: the lists a fixture gives, written as JSON, with no window opened.
/// `tools/rows.mjs` holds them against the rows the web window drew from the same fixture.
/// With them goes what the Inbox page draws around its rows with `--tab <name>` open: the
/// strip of tabs, the time column's heading and the cloud button. None when the flag is not there.
/// The lists are under the display the picture was taken on (`display` in the fixture), sorted
/// and filtered as the page drew them.
pub fn rows_from_args(args: &[String]) -> Option<Result<(), String>> {
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|at| args.get(at + 1));
    let out = flag("--rows")?;
    Some((|| {
        let fixture = crate::source::read_fixture(flag("--fixture").ok_or("--rows needs --fixture <file>")?)?;
        let now = fixture.now.ok_or("the fixture has no clock")?;
        let shown = fixture.rest.get("display").cloned().map(display::Display::from).unwrap_or_default();
        let mut lists = lists_sorted(&fixture.snapshot, now, &fixture.local, shown.sort);
        display::show(&mut lists, &shown, now);
        let open = Tab::from_args(args);
        let tabs: Vec<Value> = page::tabs(&lists, open).into_iter().map(|t| serde_json::json!({ "label": t.label(), "count": lists.count(t), "on": t == open })).collect();
        let page = serde_json::json!({ "tabs": tabs, "heading": page::time_heading(open.view()), "cloud": cloud::button(fixture.snapshot.rest.get("cloud"), &fixture.local.cloud_broken_seen) });
        let json = serde_json::json!({ "now": now, "needs": lists.needs, "progress": lists.progress, "progressTab": lists.in_progress_tab(), "later": lists.later, "done": lists.done, "page": page });
        std::fs::write(out, serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?).map_err(|e| format!("{out}: {e}"))
    })())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const NOW: f64 = 1_784_000_000_000.0;
    const MIN: f64 = 60_000.0;

    /// A thread the person wrote `ago` minutes before now.
    fn thread(id: &str, product: &str, ago: f64, extra: Value) -> Value {
        let at = NOW - ago * MIN;
        let mut v = json!({ "id": id, "product": product, "productName": product.to_uppercase(), "status": "open", "kind": "directive", "title": format!("Thread {id}"), "labels": ["founder"],
            "priority": 0, "createdAt": at, "updatedAt": at, "wrote": { "title": { "ts": at, "source": "founder" }, "body": { "ts": at, "source": "founder" } } });
        for (k, value) in extra.as_object().unwrap() {
            v[k] = value.clone();
        }
        v
    }

    /// The same thread, answered and closed by its agent `ago` minutes before now.
    fn finished(id: &str, product: &str, ago: f64, by: &str, extra: Value) -> Value {
        let mut v = thread(id, product, ago + 30.0, extra);
        v["status"] = json!("done");
        v["result"] = json!("Fixed.");
        v["updatedAt"] = json!(NOW - ago * MIN);
        v["wrote"]["result"] = json!({ "ts": NOW - ago * MIN, "source": "agent" });
        v["wrote"]["status"] = json!({ "ts": NOW - ago * MIN, "source": by });
        v
    }

    fn lists_of(items: Vec<Value>, supervisor: Value) -> Lists {
        let snap: Snapshot = serde_json::from_value(json!({
            "products": [{ "slug": "shop", "name": "Storefront" }, { "slug": "desk", "name": "Front desk" }],
            "items": items, "supervisor": supervisor,
        }))
        .unwrap();
        lists(&snap, NOW, &Local::default())
    }

    fn ids(rows: &[Row]) -> Vec<&str> {
        rows.iter().map(|r| r.id.as_str()).collect()
    }

    #[test]
    fn every_thread_lands_in_one_list_in_the_order_its_page_draws() {
        let lists = lists_of(
            vec![
                finished("news-old", "shop", 50.0, "agent", json!({})),
                finished("news-new", "shop", 6.0, "agent", json!({})),
                finished("news-desk", "desk", 5.0, "agent", json!({ "priority": 9 })),
                finished("news-low", "shop", 4.0, "agent", json!({ "priority": 2 })),
                finished("archived-old", "shop", 40.0, "founder", json!({})),
                finished("archived-new", "desk", 3.0, "founder", json!({})),
                thread("queued-low", "shop", 20.0, json!({ "priority": 2 })),
                thread("queued-desk", "desk", 30.0, json!({ "priority": 9 })),
                thread("queued-pinned", "desk", 45.0, json!({ "pinned": true })),
                thread("running", "shop", 10.0, json!({ "status": "claimed", "claim": { "holder": "mcp-1" } })),
                thread("later", "shop", 10.0, json!({ "start": "later" })),
            ],
            json!({ "productOrder": ["shop", "desk"], "queued": ["queued-low"], "running": [{ "itemId": "running", "product": "shop", "startedAt": NOW - MIN }] }),
        );
        // The first project's threads by level and then newest, then the second project's.
        assert_eq!(ids(&lists.needs), ["news-new", "news-old", "news-low", "news-desk"]);
        // Home: in the order the fleet runs them. The tab: pinned first, then newest sent.
        assert_eq!(ids(&lists.progress), ["running", "queued-low", "queued-desk", "queued-pinned"]);
        assert_eq!(lists.in_progress_tab().iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["queued-pinned", "running", "queued-low", "queued-desk"]);
        assert_eq!(ids(&lists.done), ["archived-new", "archived-old"]);

        let news = &lists.needs[0];
        assert_eq!((news.title.as_str(), news.status.as_str(), news.project.as_str(), news.place.as_str()), ("Thread news-new", "Done", "SHOP", "Storefront"));
        // The time is when the person last wrote, not when the agent answered.
        assert_eq!((news.when.as_str(), news.priority, news.letter, news.unread), ("36 min ago", Priority::Medium, "C", true));
        assert_eq!(lists.needs[3].priority, Priority::Urgent);
        let working = &lists.progress[0];
        assert_eq!((working.status.as_str(), working.mark, working.engine, working.when.as_str()), ("Working", Mark::Running, "Claude Code", "10 min ago"));
        assert_eq!((lists.progress[1].status.as_str(), lists.progress[1].mark), ("Queued", Mark::Queued));
        assert!(lists.progress[3].pinned);
        // Done is dated by its close, and nothing in it is unread.
        assert_eq!((lists.done[0].when.as_str(), lists.done[0].unread), ("3 min ago", false));
    }

    #[test]
    fn a_thread_says_how_many_of_its_tasks_wait_for_a_yes() {
        let lists = lists_of(
            vec![
                finished("parent", "shop", 5.0, "agent", json!({})),
                json!({ "id": "task-1", "product": "shop", "status": "open", "kind": "directive", "title": "Add a retry", "labels": [], "parent": "parent", "createdAt": NOW - 4.0 * MIN, "updatedAt": NOW - 4.0 * MIN }),
                json!({ "id": "task-2", "product": "shop", "status": "open", "kind": "directive", "title": "Add a log line", "labels": [], "parent": "parent", "createdAt": NOW - 3.0 * MIN, "updatedAt": NOW - 3.0 * MIN }),
                // One already approved is on its way, and no longer waits.
                json!({ "id": "task-3", "product": "shop", "status": "open", "kind": "directive", "title": "Add a test", "labels": [], "parent": "parent", "answer": "Approved. Run it.", "createdAt": NOW - 2.0 * MIN, "updatedAt": NOW - 2.0 * MIN }),
            ],
            json!({}),
        );
        // The proposals wait in Later; the thread that filed them shows them.
        assert_eq!(ids(&lists.needs), ["parent"]);
        assert_eq!(lists.needs[0].aside, "2 tasks to approve");
        assert_eq!(ids(&lists.progress), ["task-3"]);
    }

    #[test]
    fn later_lists_what_comes_back_in_the_pages_order_and_says_when() {
        const DAY: f64 = 24.0 * 60.0 * MIN;
        // Put off by the person until `back` from now, `ago` minutes ago.
        let put_off = |id: &str, ago: f64, back: f64| {
            let mut v = thread(id, "shop", ago, json!({ "runAt": NOW + back }));
            v["wrote"]["runAt"] = json!({ "ts": NOW - ago * MIN, "source": "founder" });
            v
        };
        let mut stuck = thread("stuck", "shop", 15.0, json!({ "status": "blocked", "blockedBy": ["cause"] }));
        stuck["wrote"]["status"] = json!({ "ts": NOW - 15.0 * MIN, "source": "agent" });
        stuck["wrote"]["blockedBy"] = json!({ "ts": NOW - 15.0 * MIN, "source": "agent" });
        let lists = lists_of(
            vec![
                thread("written-down", "shop", 40.0, json!({ "start": "later" })),
                thread("pinned-written-down", "shop", 50.0, json!({ "start": "later", "pinned": true })),
                put_off("tomorrow", 5.0, DAY),
                // Two put off in the same minute: the one that comes back sooner is first.
                put_off("day-after", 20.0, 2.0 * DAY),
                put_off("tomorrow-evening", 20.0, DAY + 60.0 * MIN),
                finished("parent", "shop", 35.0, "agent", json!({})),
                json!({ "id": "proposal", "product": "shop", "status": "open", "kind": "directive", "title": "Add a retry", "labels": [], "parent": "parent", "priority": 7, "createdAt": NOW - 30.0 * MIN, "updatedAt": NOW - 30.0 * MIN }),
                json!({ "id": "cause", "product": "shop", "status": "open", "kind": "cause", "title": "GitHub is signed out", "labels": ["cause"], "createdAt": NOW - 16.0 * MIN, "updatedAt": NOW - 16.0 * MIN, "wrote": { "kind": { "ts": NOW - 16.0 * MIN, "source": "system" } } }),
                stuck,
                // A run is still queued for it, so it is not Later's yet; and what the person
                // closed is Done's alone, where what its agent finished is in both.
                put_off("queued-put-off", 6.0, DAY),
                finished("closed-written-down", "shop", 46.0, "founder", json!({ "start": "later" })),
                finished("finished-written-down", "shop", 45.0, "agent", json!({ "start": "later" })),
            ],
            json!({ "productOrder": ["shop", "desk"], "queued": ["queued-put-off"] }),
        );
        // Pinned, then the level each row shows, then newest; rows that tie keep soonest back first.
        assert_eq!(ids(&lists.later), ["pinned-written-down", "proposal", "tomorrow", "stuck", "tomorrow-evening", "day-after", "written-down", "finished-written-down"]);
        let row = |id: &str| lists.later.iter().find(|r| r.id == id).unwrap();
        let said = |id: &str| (row(id).status.as_str(), row(id).when.as_str(), row(id).held);
        // Written down with no moment: it says so twice and names no time.
        assert_eq!(said("pinned-written-down"), ("Not started", "", true));
        assert!(row("pinned-written-down").pinned);
        // Put off: one word for everything that comes back, and when it does.
        let tomorrow = page::day_and_time(NOW + DAY, NOW);
        assert_eq!(said("tomorrow"), ("Scheduled", tomorrow.as_str(), false));
        assert_eq!(row("tomorrow").shown_at, NOW + DAY);
        assert_eq!(said("proposal"), ("Scheduled", "", false));
        // Behind a cause: what it waits on, and that it comes back when that is fixed.
        assert_eq!(said("stuck"), ("Waiting", "When fixed", false));
        assert_eq!(row("stuck").aside, "GitHub is signed out");
        assert_eq!(said("finished-written-down"), ("Not started", "", true));
        assert!(lists.later.iter().all(|r| !r.unread && r.mark == Mark::None));
        // The cause itself waits in Needs you, counting what is behind it.
        let cause = lists.needs.iter().find(|r| r.id == "cause").unwrap();
        assert_eq!(cause.status, "1 waiting");
        assert_eq!(ids(&lists.done), ["finished-written-down", "closed-written-down"]);
        assert!(lists.progress.iter().all(|r| r.id != "queued-put-off"));
        assert_eq!((lists.count(Tab::Later), lists.count(Tab::Done)), (8, 0));
        assert_eq!(lists.under(Tab::Later).len(), 8);
    }

    #[test]
    fn the_row_that_says_tasks_cannot_run_is_first_until_it_is_closed() {
        let trouble = json!({ "spawnTrouble": { "since": NOW - 9.0 * MIN, "cause": "signed-out", "message": "", "remedy": "", "count": 2 } });
        let items = || vec![finished("news", "shop", 5.0, "agent", json!({ "priority": 9 }))];
        let lists = lists_of(items(), trouble.clone());
        assert_eq!(ids(&lists.needs), ["trouble", "news"]);
        assert_eq!((lists.needs[0].title.as_str(), lists.needs[0].line.as_str(), lists.needs[0].project.as_str(), lists.needs[0].when.as_str()), ("2 tasks have not been able to run", "2 need a login", "", "9 min ago"));
        let snap: Snapshot = serde_json::from_value(json!({ "items": items(), "supervisor": trouble })).unwrap();
        let closed = Local { trouble_closed: NOW - 9.0 * MIN, ..Local::default() };
        assert_eq!(ids(&super::lists(&snap, NOW, &closed).needs), ["news"]);
    }

    #[test]
    fn a_sparse_or_odd_snapshot_is_listed_without_complaint() {
        // What an old ledger, a half-written line or another engine's version may hand over.
        let odd = vec![
            json!({}),
            json!({ "id": "only-an-id" }),
            json!({ "id": "no-times", "product": "shop", "status": "done", "labels": ["founder"], "wrote": { "status": { "source": "agent" }, "result": {} } }),
            json!({ "id": "odd-waits", "product": "shop", "status": "blocked", "blockedBy": [7, null, "odd-waits", "nowhere", ""], "wrote": { "blockedBy": { "source": "agent" } } }),
            json!({ "id": "odd-text", "product": "shop", "status": "open", "title": "\u{1F600}\u{1F600} \u{e9}_\u{e9}\r\n\u{2028}## x", "body": "[a](b\n![c](d", "result": "1. a\r\n2. b (recommended)\r\n", "labels": ["founder", "codex:", "repeat:", "teammate:", "asked-in:/"],
                "runAt": 1e18, "snoozedUntil": -5, "place": 1e300, "priority": -3, "answer": "", "note": "", "parent": "", "linkComments": [{}], "waitEnded": {}, "signIn": {}, "claim": 3, "routed": [] }),
            json!({ "id": "odd-text", "product": "desk", "status": "claimed", "createdAt": 1e15, "updatedAt": -1.0, "labels": [] }),
        ];
        let supervisor = json!({ "running": [{}], "starting": [null, 3], "queued": "no", "stalled": [["x"]], "signInNeeded": [], "silent": { "odd-text": null }, "spawnTrouble": {}, "productOrder": [] });
        let lists = lists_of(odd, supervisor);
        assert!(lists.needs.len() + lists.progress.len() + lists.done.len() >= 3);
        assert_eq!(lists.needs[0].id, "trouble");
    }

    #[test]
    fn a_thread_opened_since_its_answer_is_read() {
        let snap: Snapshot = serde_json::from_value(json!({ "items": [finished("a", "shop", 5.0, "agent", json!({})), finished("b", "shop", 6.0, "agent", json!({}))] })).unwrap();
        let local = Local { seen_at: HashMap::from([("a".to_string(), NOW - MIN)]), unread_since: NOW - 60.0 * MIN, ..Local::default() };
        let lists = lists(&snap, NOW, &local);
        assert_eq!(lists.needs.iter().map(|r| (r.id.as_str(), r.unread)).collect::<Vec<_>>(), [("a", false), ("b", true)]);
        // Finished before answers started being delivered here: it was never news.
        let late = Local { delivered_through: NOW, ..Local::default() };
        assert!(super::lists(&snap, NOW, &late).needs.is_empty());
    }
}
