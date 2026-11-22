//! General, Running, Phone, Plan and Billing, Files and Alpha features, as the web window draws
//! them (`components/Settings.tsx`), each from the settings the engine answered and the
//! snapshot.

use std::rc::Rc;

use rune::{El, TextStyle, col, el, row, text};
use serde_json::{Value, json};

use super::kit::{self, Desc, Group, Picker};
use super::{projects, write_workspace};
use crate::State;
use crate::rules::settings::{self as rules, NAME, PageId, Pane, list, num, on, text as t};
use crate::theme::*;

/// The page's title, its sentence, and its groups.
fn page(title: &str, lede: Option<&str>, groups: Vec<El>) -> Vec<El> {
    let mut out = vec![kit::title(title)];
    if let Some(lede) = lede {
        out.push(kit::lede(lede));
    }
    out.extend(groups);
    out
}

/// A switch on a workspace setting: what it shows while its write is out, and the write.
fn workspace_switch(s: &State, key: &'static str, saved: bool) -> El {
    let control = format!("ws:{key}");
    let shown = s.settings.shown(&control, &Value::Bool(saved)).as_bool().unwrap_or(saved);
    kit::switch(shown, move |s: &mut State, to, _| write_workspace(s, &format!("ws:{key}"), key, Value::Bool(to)))
}

/// The snapshot's part by name, when it is an object.
pub fn part<'a>(s: &'a State, key: &str) -> Option<&'a Value> {
    s.snap.rest.get(key).filter(|v| v.is_object())
}

/// Who is signed in, for the Account group: their address and their team's name.
fn account(s: &State) -> Option<(String, Option<String>)> {
    let team = part(s, "team").filter(|t| on(t, "signedIn"))?;
    let me = team.get("me").filter(|m| m.is_object())?;
    Some((t(me, "email").unwrap_or("").to_string(), team.get("team").and_then(|t| rules::text(t, "name")).map(str::to_string)))
}

/* ---------------------------------- General ---------------------------------- */

pub fn general(s: &State) -> Vec<El> {
    let Some(w) = s.settings.workspace() else { return Vec::new() };
    let kept = s.picks.get();
    let hints = kept.key_hints.unwrap_or(true);
    let shrinks = kept.rail_shrinks.unwrap_or(true);
    let mut groups = vec![
        Group::new("Keyboard", vec![kit::line("Show keyboard shortcut hints", Some("Point at something and its keys appear beside it. Off, every key still works.".into()), vec![kit::switch(hints, |s: &mut State, to, _| s.picks.change(|p| p.key_hints = Some(to)))])]).find("keys").el(),
        Group::new("Sidebar", vec![kit::line("Shrink the sidebar to icons", None, vec![kit::switch(shrinks, |s: &mut State, to, _| s.picks.change(|p| p.rail_shrinks = Some(to)))])]).find("sidebar").el(),
        Group::new(
            "Agent Tray",
            vec![kit::line(
                "Show the Agent Tray",
                Some(format!("When an agent has something for you, it grows out of the notch at the top of the screen you are on, or hangs from the top of a screen without one. Point at it for the list; click it to open {NAME}. Make it bigger or smaller with Size, at the foot of that list. Right-click it to hide it a while or turn it off.").into()),
                vec![workspace_switch(s, "cornerTag", w.get("cornerTag") != Some(&Value::Bool(false)))],
            )],
        )
        .find("corner-tag")
        .el(),
        Group::new(
            &format!("What {NAME} sends"),
            vec![kit::line(
                "Counts and crash reports",
                Some(format!("That {NAME} was opened, that a task was opened, that a reply was sent, and a report when something breaks. Never your code, your prompts, your keys or your paths, and never a title.").into()),
                vec![workspace_switch(s, "diagnostics", on(w, "diagnostics"))],
            )],
        )
        .find("privacy")
        .el(),
        Group::new(&format!("Where {NAME} keeps things on this computer"), vec![kit::line("Your projects, tasks and documents", Some(t(w, "storePath").unwrap_or("").into()), Vec::new())]).find("storage").el(),
        updates(s),
    ];
    if let Some((email, team)) = account(s) {
        let out = kit::ghost("Sign out", true).on_press(|s: &mut State| super::press(s, "sign-out", vec![("zero:team-sign-out", Value::Null)]));
        groups.push(Group::new("Account", vec![kit::line(&format!("Signed in as {email}"), team.map(|t| Desc::Plain(format!("On the {t} team."))), vec![out])]).find("account").el());
    }
    page("General", None, groups)
}

/// The version and the way to ask for a newer one (`Updates`): what the check answered is held
/// only until the snapshot's update moves.
fn updates(s: &State) -> El {
    let state = s.snap.rest.get("update").filter(|u| u.is_object());
    let key = state.map(Value::to_string).unwrap_or_default();
    let found = s.settings.reads.update.as_ref().filter(|(was, _)| *was == key).map(|(_, got)| got);
    let shown = found.or(state);
    let look = rules::update_look(shown, s.settings.busy.contains("update"));
    let label = match shown.and_then(|u| t(u, "currentVersion")) {
        Some(v) => format!("{NAME} {v}"),
        None => NAME.to_string(),
    };
    let button = kit::ghost(look.button, !look.busy);
    let button = if look.busy {
        button
    } else if look.restart {
        button.on_press(|s: &mut State| super::quiet(s, "zero:update-install", Value::Null))
    } else if look.move_ {
        button.on_press(|s: &mut State| super::quiet(s, "zero:update-move", Value::Null))
    } else {
        button.on_press(super::check_for_updates)
    };
    Group::new("Updates", vec![kit::line(&label, Some(look.sentence.into()), vec![button])]).find("updates").el()
}

/* ---------------------------------- Running ---------------------------------- */

pub fn running(s: &State, column: f32) -> Vec<El> {
    let Some(w) = s.settings.workspace() else { return Vec::new() };
    let two = list(w, "engineChoices").len() > 1;
    let mut groups = Vec::new();
    if let Some(f) = rules::running_figures(num(w, "running"), num(w, "agentsTotal").or_else(|| num(w, "capacity")), w.get("memoryGate")) {
        groups.push(reading(&f, column));
    }
    let auto = on(w, "agentsAuto");
    let mut rows = Vec::new();
    let at_once_value = if auto { "auto".to_string() } else { format!("{}", num(w, "sessionsAtOnce").unwrap_or(1.0)) };
    let picked = s.settings.shown("pk:at-once", &Value::String(at_once_value)).as_str().unwrap_or("auto").to_string();
    rows.push(kit::line(
        "How many run at once",
        Some(if auto {
            "From this computer’s memory, kept there as you add accounts or turn the memory check on.".into()
        } else {
            format!("You set this. Automatic would run {} on this computer.", num(w, "agentsAutoTotal").or_else(|| num(w, "sessionsAtOnce")).unwrap_or(1.0)).into()
        }),
        vec![kit::picker(
            s,
            Picker {
                key: "at-once".into(),
                above: Some("How many run at once".into()),
                value: picked,
                options: rules::agent_count_options(w),
                long: false,
                help: None,
                pick: Rc::new(|s: &mut State, v: String, _| {
                    if v == "auto" {
                        super::write_workspace(s, "pk:at-once", "agentsAuto", Value::Bool(true));
                    } else {
                        let n: f64 = v.parse().unwrap_or(1.0);
                        super::write_workspace(s, "pk:at-once", "sessionsAtOnce", json!(n));
                        // The chip shows the option's own value until the answer lands.
                        s.settings.wanted.insert("pk:at-once".into(), Value::String(v));
                    }
                }),
            },
        )],
    ));
    if let Some(gate) = w.get("memoryGate").filter(|g| g.is_object()) {
        rows.push(kit::line(
            "Hold heavy work when memory is short",
            Some(format!("Tests and builds take turns, urgent first, and new agents wait to start until memory frees up.{}", if two { " Claude Code and Codex both." } else { "" }).into()),
            vec![workspace_switch(s, "memoryGate", on(gate, "on"))],
        ));
        if on(gate, "on") && !auto {
            let slots_auto = num(gate, "slotsAuto").unwrap_or(1.0);
            let value = num(gate, "slots").unwrap_or(slots_auto);
            let most = num(gate, "slotsMax").unwrap_or(8.0);
            rows.push(kit::line(
                "Heavy commands at once",
                Some(match num(gate, "slots") {
                    None => format!("Auto: {slots_auto} on this computer, one for every 8 GB of memory."),
                    Some(n) => format!("You picked {n}. Auto is {slots_auto} on this computer."),
                }.into()),
                vec![kit::stepper(value, 1.0, most, move |s: &mut State, v| {
                    let to = if v == slots_auto { Value::Null } else { json!(v) };
                    super::write_workspace(s, "ws:memoryGateSlots", "memoryGateSlots", to);
                })],
            ));
        }
    }
    if let Some(left) = w.get("leftovers").filter(|l| l.is_object()) {
        let now = t(left, "now").map(|n| format!(" {n}")).unwrap_or_default();
        rows.push(kit::line(
            "Stop programs agents leave behind",
            Some(format!("Dev servers, previews and test runs an agent started keep running until you close its task, or until memory runs short. Apps installed on this computer are never stopped.{}{now}", if two { " Codex agents’ programs are not found yet." } else { "" }).into()),
            vec![workspace_switch(s, "cleanupLeftovers", on(left, "on"))],
        ));
    }
    if auto {
        let nudge = num(w, "agentsNudge").filter(|n| *n != 0.0);
        let mut buttons = Vec::new();
        if nudge.is_some() {
            buttons.push(kit::ghost("Undo", true).on_press(|s: &mut State| super::write_workspace(s, "ws:agentsFeltSlow", "agentsFeltSlow", json!("reset"))));
        }
        buttons.push(kit::ghost("Tell it", true).on_press(|s: &mut State| super::write_workspace(s, "ws:agentsFeltSlow", "agentsFeltSlow", Value::Bool(true))));
        rows.push(kit::line(
            "Something felt slow",
            Some(match nudge {
                Some(n) => format!("{NAME} is running {n} fewer than this computer suggests."),
                None => format!("{NAME} runs one fewer from now on, and remembers that it did."),
            }.into()),
            buttons.into_iter().enumerate().map(|(at, b)| if at > 0 { b.ml(8.0) } else { b }).collect(),
        ));
    }
    groups.push(Group::new("Agents", rows).find("at-once").el());
    let outside = t(w, "outsideAgents").unwrap_or("all").to_string();
    let outside = s.settings.shown("pk:outside", &Value::String(outside)).as_str().unwrap_or("all").to_string();
    let sentence = rules::OUTSIDE.iter().find(|(id, ..)| *id == outside).map_or("", |(_, _, what)| *what);
    groups.push(
        Group::new(
            "Agents you started yourself",
            vec![
                kit::line(
                    "Show while they run",
                    Some(format!("Claude Code started in a terminal, not from here. {sentence}").into()),
                    vec![kit::picker(
                        s,
                        Picker {
                            key: "outside".into(),
                            above: None,
                            value: outside,
                            options: rules::OUTSIDE.iter().map(|(id, label, _)| (id.to_string(), label.to_string())).collect(),
                            long: false,
                            help: None,
                            pick: Rc::new(|s: &mut State, v: String, _| super::write_workspace(s, "pk:outside", "outsideAgents", Value::String(v))),
                        },
                    )],
                ),
                kit::line(
                    "Bring in new chats",
                    Some("A new Claude Code or Codex chat in one of your projects comes in after ten quiet minutes. Waiting ones go to Needs you, finished ones to Done.".into()),
                    vec![workspace_switch(s, "importNew", w.get("importNew") == Some(&Value::Bool(true)))],
                ),
            ],
        )
        .find("outside")
        .el(),
    );
    page("Running", None, groups)
}

/// What the Running page leads with (`Reading`): the measured figures in the label voice, the
/// memory's bar, and what holds work back.
fn reading(f: &rules::Figures, column: f32) -> El {
    let mut rows = Vec::new();
    if !f.cells.is_empty() {
        let cells = f.cells.iter().map(|(n, of)| {
            col()
                .child(pen(text(n.as_str()).font(sans(19.0, 600.0).tracking(-0.19).features("'tnum' 1")).color(ink()), 19.0))
                .child(el().mt(3.0).child(pen(text(of.as_str()).font(TextStyle::new(10.5, 500.0).mono().lh(13.5).tracking(0.63)).text_case(rune::TextCase::Uppercase).color(muted()), 10.5)))
        });
        rows.push(row().py(14.0).gap(30.0).children(cells));
    }
    if let Some(pct) = f.used_pct {
        let inner = column - 36.0;
        rows.push(el().py(13.0).child(el().w(inner).h(5.0).radius(2.0).fill(white(0.08)).clip_rounded(2.0).child(el().w(inner * (pct as f32 / 100.0).clamp(0.0, 1.0)).h(5.0).fill(if f.hot { accent() } else { muted() }))));
    }
    for line in [&f.holding, &f.tally].into_iter().flatten() {
        rows.push(col().py(13.0).child(pen(text(line.as_str()).font(TextStyle::new(12.5, 400.0).lh(18.75)).color(dim()).wrap(), 12.5)));
    }
    col().mt(40.0).child(kit::plate(None, rows))
}

/* ----------------------------------- Phone ----------------------------------- */

/// The App Store's code (`IPHONE_APP_URL` in `shared/phone-app.mts`), as the `qrcode` package
/// makes it at level M: 29 modules a side, drawn with one module of white round it.
const APP_STORE_QR: [&str; 29] = [
    "#######.#.###..#####..#######",
    "#.....#.###...##..#...#.....#",
    "#.###.#..#.#.###..###.#.###.#",
    "#.###.#.#.#......#.#..#.###.#",
    "#.###.#......#...#.#..#.###.#",
    "#.....#..#.#..#.#..##.#.....#",
    "#######.#.#.#.#.#.#.#.#######",
    "........#.##.##..###.........",
    "#.##.###.#..#######...#..#.##",
    "###.##..#.##.......####.#...#",
    ".###.##.#.##..#.##...##.#.##.",
    "###.##.....#.##.#...........#",
    "#..##.#.#...#..###.....#.##..",
    "###..#..#.####...###..#...###",
    "......#####.#.#.#..#..#...###",
    ".##.##.#..#..#.#..###...#..#.",
    "#.###.##.##.#......###..##.#.",
    ".##.....#..###.###...#.#.###.",
    "#.#...####.##....#..##..#.#..",
    "...##....##....#.##....#..#..",
    ".##..###.#..##.############..",
    "........#.#####..##.#...#####",
    "#######.##.###.#.#.##.#.##.#.",
    "#.....#.##.....##.#.#...##.##",
    "#.###.#...###.#..#..#####.#.#",
    "#.###.#.##..####.#..##..##.#.",
    "#.###.#.###.....#.#.##.#..#.#",
    "#.....#.....#.##..#.##.###.#.",
    "#######.#..##.#.##.##..#...#.",
];
pub const IPHONE_APP_URL: &str = "https://apps.apple.com/app/id6826402748";

/// The code on white, 136 points with 8 of padding (`.set-phone-qr img`). Each module's edges
/// land on whole device pixels of a 2x screen, as the picture's `crispEdges` draws them.
fn app_store_code() -> El {
    let n = APP_STORE_QR.len() as f32 + 2.0;
    let unit = 120.0 / n;
    let snap = |v: f32| (v * 2.0).round() / 2.0;
    let mut modules = Vec::new();
    for (y, line) in APP_STORE_QR.iter().enumerate() {
        for (x, c) in line.chars().enumerate() {
            if c == '#' {
                let (x0, y0) = (snap((x as f32 + 1.0) * unit), snap((y as f32 + 1.0) * unit));
                let (x1, y1) = (snap((x as f32 + 2.0) * unit), snap((y as f32 + 2.0) * unit));
                modules.push(el().absolute(x0, y0).size(x1 - x0, y1 - y0).fill(rgb(0, 0, 0)));
            }
        }
    }
    el().size(136.0, 136.0).px(8.0).py(8.0).radius(10.0).fill(rgb(255, 255, 255)).child(el().size(120.0, 120.0).children(modules))
}

pub fn phone(s: &State) -> Vec<El> {
    let team = part(s, "team");
    let signed_in = team.is_some_and(|t| on(t, "signedIn"));
    let email = team.and_then(|t| t.get("me")).and_then(|m| t(m, "email")).map(str::to_string);
    // Mercury for iOS: the code, a sentence and the App Store.
    let said = match &email {
        Some(email) => {
            let head = "Scan with your phone's camera, then sign in as ";
            Desc::Bold(format!("{head}{email}."), vec![(head.len(), head.len() + email.len())])
        }
        None => Desc::Plain(format!("Scan with your phone's camera, then sign in with your {NAME} account.")),
    };
    let store = kit::ghost("Open the App Store", true).on_press(|_: &mut State| crate::files::open_with_system(IPHONE_APP_URL));
    // The page's rows keep their 24 points apart here too: the card's row rule outranks the
    // code's own 18 (`.set-phone-qr`).
    let app = row()
        .py(14.0)
        .gap(24.0)
        .items_center()
        .child(app_store_code())
        .child(col().grow(1.0).min_w(0.0).child(kit::label_words(&format!("{NAME} for iOS"))).child(col().mt(3.0).child(kit::desc_words(said))).child(row().mt(12.0).child(store)));
    let android = kit::line("On Android", None, vec![kit::ghost("Request the Android app", true).on_press_tx(|s: &mut State, tx| super::bridge(s, PageId::Phone, tx))]);
    let mut groups = vec![Group::new("Get the app", vec![app, android]).find("phone-app").el()];
    if signed_in {
        let as_who = email.clone().unwrap_or_else(|| "your account".to_string());
        let mut rows = Vec::new();
        match team.and_then(|part| t(part, "phoneError")) {
            Some(error) => {
                let why = format!("{}.", error.trim_end_matches('.'));
                let why = format!("{}{}", why[..1].to_uppercase(), &why[1..]);
                rows.push(kit::line("Not reaching the web app", Some(format!("{why} Signed in as {as_who}. It tries again every few seconds.").into()), Vec::new()));
            }
            None => {
                if team.and_then(|t| num(t, "phoneSentAt")).is_some_and(|at| at != 0.0) {
                    rows.push(kit::line(&format!("Connected as {as_who}"), Some("Your phone and the web app have this computer's threads.".into()), Vec::new()));
                }
            }
        }
        rows.push(kit::line("Projects on your phone", Some("A copy of their threads is kept in your account, so your phone can read and reply with this computer closed.".into()), vec![projects::phone_picker(s)]));
        groups.push(Group::new("On your phone", rows).find("phone-mac-shut").el());
    } else {
        rows_sign_in(&mut groups);
    }
    page("Phone", Some("Read and reply from your phone, even with this computer closed."), groups)
}

/// The phone's group for someone not signed in: signing in is the web window's to do.
fn rows_sign_in(groups: &mut Vec<El>) {
    let sign_in = kit::ghost("Sign in", true).on_press_tx(|s: &mut State, tx| super::bridge(s, PageId::Phone, tx));
    groups.push(Group::new("On your phone", vec![kit::line("Sign in to use your phone", Some("Then your phone can read and reply with this computer closed.".into()), vec![sign_in])]).find("phone-mac-shut").el());
}

/* ------------------------------- Plan and Billing ------------------------------- */

pub fn cloud(s: &State, column: f32) -> Vec<El> {
    let plan = s.settings.reads.plan.as_ref();
    let offer = plan.and_then(|p| p.get("trialOffer")).filter(|o| o.is_object());
    let lede = match offer {
        Some(o) => format!("Every plan starts with {} days free and {} cloud hours.", num(o, "days").unwrap_or(7.0), num(o, "hours").unwrap_or(50.0)),
        None => "Your computer runs a few threads at once. The cloud runs the rest.".to_string(),
    };
    // A paid plan has a summary of its own, which this window does not draw: its page is the web
    // window's, where Stripe's billing is reached.
    if rules::paid_plan(plan) {
        let open = kit::ghost("Open in Mercury", true).on_press_tx(|s: &mut State, tx| super::bridge(s, PageId::Cloud, tx));
        return page("Plan and Billing", None, vec![Group::new("Your plan", vec![kit::line("Your plan and this month", Some("Shown in Mercury's own window, with Stripe's billing.".into()), vec![open])]).el()]);
    }
    let gap = 12.0;
    let card_w = (column - 2.0 * gap) / 3.0;
    let cards = rules::plan_cards(plan).into_iter().map(|c| {
        // The price and " a month" after it on its line, the small words fifteen points down
        // the price's line, where their baseline meets it (`.cloud-plan-price span`).
        let mut card = col()
            .w(card_w)
            .px(18.0)
            .py(18.0)
            .radius(6.0)
            .fill(rgba(239, 235, 231, 0.03))
            .border(1.0, if c.current { accent() } else { rule() })
            .border_layout()
            .child(row().justify_between().child(pen(text(c.name).font(TextStyle::new(11.0, 400.0).mono().lh(14.0).tracking(1.1)).text_case(rune::TextCase::Uppercase).color(muted()), 11.0)))
            .child(row().mt(12.0).items_start().child(pen(text(c.price.as_str()).font(sans(28.0, 400.0).tracking(-0.56)).color(ink()), 28.0)).child(el().mt(15.0).child(pen(text(" a month").font(sans(13.0, 400.0)).color(muted()), 13.0))))
            .child(el().mt(6.0).mb(22.0).child(pen(text(c.hours.as_str()).font(sans(13.0, 400.0)).color(dim()), 13.0)));
        if let Some(button) = &c.button {
            // Choosing a plan opens Stripe from the web window, which follows the payment.
            card = card.child(row().child(kit::ghost(button, true).on_press_tx(|s: &mut State, tx| super::bridge(s, PageId::Cloud, tx))));
        }
        card
    });
    let counts = [
        "Agents in the cloud run on top of the ones your computer runs, as many at once as you like.",
        "Time counts only while an agent is working. Waiting for your reply is free.",
        "Past a plan's hours, threads wait for your computer until the month turns.",
    ];
    // The list's own discs, in the 18 points before each line (`.cloud-counts`).
    let bullets = col().pt(4.0).gap(8.0).children(counts.iter().map(|line| {
        row()
            .child(el().w(18.0).h(kept(20.925)).child(el().absolute(5.5, 7.83).size(4.5, 4.5).radius(2.25).fill(dim())))
            .child(col().grow(1.0).min_w(0.0).child(kit::selectable(pen(text(*line).font(TextStyle::new(13.5, 400.0).lh(kept(20.925))).color(dim()).wrap(), 13.5))))
    }));
    let mut groups = vec![row().mt(40.0).gap(gap).items_stretch().children(cards)];
    if let Some(says) = plan.filter(|p| t(p, "billing") == Some("given") && rules::paid_plan(Some(p))).map(|_| "Choosing a plan starts paying for it.") {
        groups.push(kit::say(says));
    }
    groups.push(Group::new("How the cloud counts", vec![bullets]).find("cloud-counts").bare().el());
    page("Plan and Billing", Some(&lede), groups)
}

/* ------------------------------------ Files ------------------------------------ */

pub fn files(s: &State) -> Vec<El> {
    let Some(w) = s.settings.workspace() else { return Vec::new() };
    let kept = w.get("files").cloned().unwrap_or_else(|| json!({ "on": true, "offAfterDays": 30 }));
    let cap = s.settings.reads.files.as_ref().and_then(|f| num(f, "cap")).unwrap_or(0.0);
    let mut rows = Vec::new();
    if cap > 0.0 {
        rows.push(kit::line(
            "Keep files in the cloud",
            Some(format!("What your agents make is saved to {NAME} Cloud as they make it.").into()),
            vec![workspace_switch(s, "filesInCloud", on(&kept, "on"))],
        ));
        if on(&kept, "on") {
            let days = format!("{}", num(&kept, "offAfterDays").unwrap_or(30.0));
            let days = s.settings.shown("pk:files-off-after", &Value::String(days)).as_str().unwrap_or("30").to_string();
            rows.push(kit::line(
                &format!("Take big files off this {}", rules::MACHINE),
                Some("Films and pictures only. Documents always stay.".into()),
                vec![kit::picker(
                    s,
                    Picker {
                        key: "files-off-after".into(),
                        above: None,
                        value: days,
                        options: rules::OFF_AFTER.iter().map(|(v, l)| (v.to_string(), l.to_string())).collect(),
                        long: false,
                        help: None,
                        pick: Rc::new(|s: &mut State, v: String, _| {
                            write_workspace(s, "pk:files-off-after", "filesOffAfterDays", json!(v.parse::<f64>().unwrap_or(30.0)));
                            // The chip shows the option's own value until the answer lands.
                            s.settings.wanted.insert("pk:files-off-after".into(), Value::String(v));
                        }),
                    },
                )],
            ));
        }
        let bytes = s.settings.reads.files.as_ref().and_then(|f| num(f, "bytes")).unwrap_or(0.0);
        let fill = (bytes / cap * 100.0).max(if bytes > 0.0 { 2.0 } else { 0.0 }).min(100.0) as f32;
        rows.push(kit::line(
            "Space used",
            None,
            vec![
                pen(text(format!("{} of {}", rules::size_words(bytes), rules::size_words(cap))).font(figures(13.0, 400.0)).color(dim()), 13.0),
                el().ml(5.0).w(164.0).h(2.0).fill(edge()).child(el().w(164.0 * fill / 100.0).h(2.0).fill(dim())),
            ],
        ));
    } else {
        let plans = kit::ghost("See plans", true).on_press_tx(|s: &mut State, tx| super::go(s, Pane::Page(PageId::Cloud), tx));
        rows.push(kit::line("Keep files in the cloud", Some(format!("Comes with Pro, Plus and Max. Until then your files stay on this {}.", rules::MACHINE).into()), vec![plans]));
    }
    page("Files", None, vec![Group::new("In the cloud", rows).find("files-cloud").el()])
}

/* -------------------------------- Alpha features -------------------------------- */

pub fn alpha(s: &State) -> Vec<El> {
    let Some(w) = s.settings.workspace() else { return Vec::new() };
    let alpha = w.get("alpha").cloned().unwrap_or(Value::Null);
    let switch = |key: &'static str, field: &str| workspace_switch(s, key, on(&alpha, field));
    let rows = vec![
        kit::line("Rooms", Some("See the people you work with and their agents, with a chat beside them. Needs a team sign-in.".into()), vec![switch("alphaRooms", "rooms")]),
        kit::line("Teammates", Some("Agents with a standing job that work on their own, talk in one team chat and reach you only when they need you.".into()), vec![switch("alphaTeammates", "teammates")]),
        kit::line("Multiplayer", Some("Share projects with your team and see their threads beside yours. Needs a sign-in.".into()), vec![switch("alphaMultiplayer", "multiplayer")]),
        kit::line("Glass", Some("The space around the page becomes frosted glass over your desktop.".into()), vec![switch("alphaGlass", "glass")]),
    ];
    page("Alpha features", Some("Early features you can try before they are finished. They may change or go away."), vec![Group::unnamed(rows).find("alpha").el()])
}
