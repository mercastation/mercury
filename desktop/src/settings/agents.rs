//! One page per coding agent (`enginePage` and `antigravityPage` in `components/Settings.tsx`):
//! what is wrong with it if anything, the accounts signed in with their meters, the cloud, and
//! what its agents may do here and in the cloud.

use std::rc::Rc;

use rune::{Cursor, El, TextStyle, col, el, icon, row, text};
use serde_json::{Value, json};

use super::kit::{self, Desc, Group, Picker};
use super::{press, write_workspace};
use crate::State;
use crate::rules::settings::{self as rules, Connection, NAME, PageId, Place, Tile, TileAct, list, num, on, text as t};
use crate::theme::*;

#[derive(Clone, Copy, PartialEq)]
enum Engine {
    Claude,
    Codex,
    Antigravity,
}

impl Engine {
    fn id(self) -> &'static str {
        match self {
            Engine::Claude => "claude",
            Engine::Codex => "codex",
            Engine::Antigravity => "antigravity",
        }
    }
    fn name(self) -> &'static str {
        match self {
            Engine::Claude => "Claude Code",
            Engine::Codex => "Codex",
            Engine::Antigravity => "Antigravity",
        }
    }
    fn page(self) -> PageId {
        match self {
            Engine::Claude => PageId::Claude,
            Engine::Codex => PageId::Codex,
            Engine::Antigravity => PageId::Antigravity,
        }
    }
}

/// The cloud, when this Mac is set up for it and it has a login of this agent (`cloudHere`).
fn cloud_here(s: &State, engine: Engine) -> Option<&Value> {
    super::pages::part(s, "cloud").filter(|c| on(c, "setUp") && list(c, "accounts").iter().any(|a| t(a, "engine") == Some(engine.id())))
}

/// The reading of this agent the snapshot carries (`agentStory`'s `reading`).
fn snapshot_reading(s: &State, engine: Engine) -> Option<Value> {
    let every = s.snap.rest.get("usageByEngine").and_then(Value::as_array).cloned().unwrap_or_else(|| s.snap.rest.get("usage").filter(|u| u.is_object()).cloned().into_iter().collect());
    every.into_iter().find(|r| t(r, "engine") == Some(engine.id()))
}

/// The newer of two readings (`newerReading`).
fn newer(a: Option<Value>, b: Option<Value>) -> Option<Value> {
    match (a, b) {
        (None, b) => b,
        (a, None) => a,
        (Some(a), Some(b)) => Some(if num(&b, "at").unwrap_or(0.0) > num(&a, "at").unwrap_or(0.0) { b } else { a }),
    }
}

pub fn claude(s: &State, column: f32) -> Vec<El> {
    engine_page(s, Engine::Claude, column)
}

pub fn codex(s: &State, column: f32) -> Vec<El> {
    engine_page(s, Engine::Codex, column)
}

/// Claude Code's page, or Codex's (`enginePage`).
fn engine_page(s: &State, engine: Engine, column: f32) -> Vec<El> {
    let Some(w) = s.settings.workspace() else { return Vec::new() };
    let codex = w.get("codex").cloned().unwrap_or(Value::Null);
    let (found, certain, trouble, bin, chosen, url) = match engine {
        Engine::Codex => (on(&codex, "found"), on(&codex, "certain"), t(&codex, "trouble").map(str::to_string), t(&codex, "bin").unwrap_or("").to_string(), on(&codex, "chosen"), t(&codex, "url").unwrap_or("").to_string()),
        _ => (on(w, "claudeFound"), w.get("claudeCertain") != Some(&Value::Bool(false)), None, t(w, "claudeBin").unwrap_or("").to_string(), on(w, "claudeChosen"), t(w, "claudeInstallUrl").unwrap_or("").to_string()),
    };
    if engine == Engine::Codex && w.get("codex").is_none_or(Value::is_null) {
        return Vec::new();
    }
    let mut out = vec![kit::title(engine.name())];
    if !found || trouble.is_some() || chosen {
        out.push(status(s, engine, found, certain, trouble.as_deref(), &bin, chosen, &url));
    }
    if !found {
        return out;
    }
    let cloud = cloud_here(s, engine);
    let cloud_logins: Vec<Value> = cloud.map(|c| rules::counted(list(c, "accounts")).into_iter().filter(|a| t(a, "engine") == Some(engine.id())).cloned().collect()).unwrap_or_default();
    let here = if engine == Engine::Codex { rules::codex_logins(w) } else { rules::claude_logins(w) };
    let current = snapshot_reading(s, engine);
    let accounts = match engine {
        Engine::Codex => rules::codex_accounts_once(&here, &cloud_logins, &s.settings.reads.codex, newer(current, s.settings.reads.codex_now.clone()).as_ref()),
        _ => rules::accounts_once(&here, &cloud_logins, &s.settings.reads.accounts, current.as_ref(), true),
    };
    let set_up = super::pages::part(s, "cloud").is_some_and(|c| on(c, "setUp"));
    let busy = s.settings.busy.contains("refresh");
    let tiles: Vec<Tile> = accounts
        .iter()
        .map(|a| {
            let claude = engine == Engine::Claude;
            let mut tile = rules::account_tile(a, here.len() > 1, s.now, set_up, claude, claude);
            if a.limits.as_ref().is_some_and(|l| !l.is_empty()) {
                return tile;
            }
            tile.none = match engine {
                Engine::Codex => Some(if tile.in_use { "Codex reports usage as it works" } else { "No reading until this account runs" }.to_string()),
                _ if busy => Some("Reading its usage…".to_string()),
                _ => None,
            };
            tile
        })
        .collect();
    let last = tiles.iter().filter_map(|t| t.checked_at).fold(0.0, f64::max);
    let checked = if last > 0.0 { rules::usage_checked_at(Some(last), s.now) } else { String::new() };
    let mut action = row().gap(12.0).items_center();
    if !busy && !checked.is_empty() {
        action = action.child(pen(text(checked.as_str()).font(sans(12.0, 400.0).tracking(-0.07)).color(muted()), 12.0));
    }
    let refresh = if busy {
        kit::ghost("Refreshing…", false)
    } else {
        kit::ghost("Refresh", true).on_press(move |s: &mut State| if engine == Engine::Codex { super::refresh_codex(s) } else { super::refresh_accounts(s) })
    };
    action = action.child(refresh);
    let warn = if engine == Engine::Claude { t(w, "accountsNote").map(str::to_string) } else { None };
    let line = (tiles.len() > 1).then(|| "Your agents run on one account at a time.".to_string());
    out.push(place(s, "Signed in", Some(action), warn.or(line), &tiles, Some(engine), column));
    if let Some(cloud) = cloud {
        out.push(use_the_cloud(s, cloud, engine, &cloud_logins));
    }
    // What its agents may do, here and in the cloud.
    let mut rows = Vec::new();
    match engine {
        Engine::Codex => {
            let mode = t(w, "codexMode").unwrap_or("auto").to_string();
            rows.push(mode_row(s, if cloud.is_some() { "On this computer" } else { "Permission mode" }, "mode", "codexMode", &mode, &rules::CODEX_MODES, false, Some("The starting point for every Codex agent. A project can be set to its own.")));
            if cloud.is_some() {
                let there = t(w, "cloudCodexMode").unwrap_or("same").to_string();
                rows.push(cloud_mode_row(s, "cloudCodexMode", &there, &rules::CODEX_MODES, |id| if id == "full-access" { Some(rules::CODEX_CLOUD_FULL) } else { rules::mode_what(&rules::CODEX_MODES, id) }));
            }
        }
        _ => {
            let mode = t(w, "permission").unwrap_or("default").to_string();
            if mode == "custom" {
                rows.push(kit::line("Permission mode", Some(rules::CUSTOM_SENTENCE.into()), Vec::new()));
            } else {
                let help = format!("Claude Code's own modes. The starting point for every Claude Code agent, and a single reply can be sent in another. {}", rules::RULES_ALWAYS_APPLY);
                rows.push(mode_row(s, if cloud.is_some() { "On this computer" } else { "Permission mode" }, "mode", "permission", &mode, &rules::CLAUDE_MODES, false, Some(&help)));
            }
            if cloud.is_some() {
                let there = t(w, "cloudPermission").unwrap_or("same").to_string();
                rows.push(cloud_mode_row(s, "cloudPermission", &there, &rules::CLAUDE_MODES, |id| rules::mode_what(&rules::CLAUDE_MODES, id)));
            }
        }
    }
    out.push(Group::new("Permissions", rows).find("permissions").el());
    out
}

/// Antigravity's page (`antigravityPage`): one Google sign-in, so no Add account.
pub fn antigravity(s: &State, column: f32) -> Vec<El> {
    let Some(w) = s.settings.workspace() else { return Vec::new() };
    let Some(a) = w.get("antigravity").filter(|a| a.is_object()) else { return Vec::new() };
    let mut out = vec![kit::title("Antigravity")];
    let found = on(a, "found");
    if !found || on(a, "chosen") {
        out.push(status(s, Engine::Antigravity, found, on(a, "certain"), None, t(a, "bin").unwrap_or(""), on(a, "chosen"), t(a, "url").unwrap_or("")));
    }
    if !found {
        return out;
    }
    let cloud = cloud_here(s, Engine::Antigravity);
    let set_up = super::pages::part(s, "cloud").filter(|c| on(c, "setUp"));
    let tiles = rules::antigravity_tiles(a, set_up, true);
    let busy = s.settings.busy.contains("agy");
    let refresh = if busy {
        kit::ghost("Refreshing…", false)
    } else {
        kit::ghost("Refresh", true).on_press(|s: &mut State| press(s, "agy", vec![("zero:antigravity-refresh", Value::Null), ("zero:settings-read", Value::Null)]))
    };
    out.push(place(s, "Signed in", Some(row().gap(12.0).items_center().child(refresh)), None, &tiles, Some(Engine::Antigravity), column));
    if let Some(cloud) = cloud {
        let logins: Vec<Value> = rules::counted(list(cloud, "accounts")).into_iter().filter(|x| t(x, "engine") == Some("antigravity")).cloned().collect();
        out.push(use_the_cloud(s, cloud, Engine::Antigravity, &logins));
    }
    let mode = t(a, "mode").unwrap_or("auto").to_string();
    let mut rows = vec![mode_row(
        s,
        if cloud.is_some() { "On this computer" } else { "Permission mode" },
        "mode",
        "antigravityMode",
        &mode,
        &rules::AGY_MODES,
        false,
        Some("Antigravity's own modes. The starting point for every Antigravity agent, and a single reply can be sent in another."),
    )];
    if cloud.is_some() {
        let there = t(a, "cloudMode").unwrap_or("same").to_string();
        rows.push(cloud_mode_row(s, "cloudAntigravityMode", &there, &rules::AGY_MODES, |id| rules::mode_what(&rules::AGY_MODES, id)));
    }
    out.push(Group::new("Permissions", rows).find("permissions").el());
    out
}

/// A permission picker's row on this computer.
#[allow(clippy::too_many_arguments)]
fn mode_row(s: &State, label: &str, key: &'static str, setting: &'static str, mode: &str, table: &'static [(&'static str, &'static str, &'static str)], _cloud: bool, help: Option<&str>) -> El {
    let control = format!("pk:{key}");
    let shown = s.settings.shown(&control, &Value::String(mode.to_string())).as_str().unwrap_or(mode).to_string();
    kit::line(
        label,
        rules::mode_what(table, &shown).map(Desc::from),
        vec![kit::picker(
            s,
            Picker {
                key: key.into(),
                above: None,
                value: shown,
                options: table.iter().map(|(id, word, _)| (id.to_string(), word.to_string())).collect(),
                long: false,
                help: help.map(str::to_string),
                pick: Rc::new(move |s: &mut State, v: String, _| write_workspace(s, &format!("pk:{key}"), setting, Value::String(v))),
            },
        )],
    )
}

/// The cloud's own mode (`cloudPermission` and its kin): Same as this computer, or one of the
/// modes, which a cloud run starts in.
fn cloud_mode_row(s: &State, setting: &'static str, there: &str, table: &'static [(&'static str, &'static str, &'static str)], what: fn(&str) -> Option<&'static str>) -> El {
    let shown = s.settings.shown("pk:cloud-mode", &Value::String(there.to_string())).as_str().unwrap_or(there).to_string();
    let desc = if shown == "same" { rules::SAME_AS_HERE.to_string() } else { what(&shown).unwrap_or("").to_string() };
    let mut options = vec![("same".to_string(), "Same as this computer".to_string())];
    options.extend(table.iter().map(|(id, word, _)| (id.to_string(), word.to_string())));
    kit::line(
        "In the cloud",
        Some(desc.into()),
        vec![kit::picker(
            s,
            Picker {
                key: "cloud-mode".into(),
                above: None,
                value: shown,
                options,
                long: false,
                help: Some("The mode every cloud run starts in.".to_string()),
                pick: Rc::new(move |s: &mut State, v: String, _| write_workspace(s, "pk:cloud-mode", setting, Value::String(v))),
            },
        )],
    )
}

/// The Cloud group: whether threads go there, and on which account (`Use the cloud`).
fn use_the_cloud(s: &State, cloud: &Value, engine: Engine, logins: &[Value]) -> El {
    let saved = t(cloud, "mode").is_some_and(|m| m != "mac");
    let shown = s.settings.shown("sw:cloud", &Value::Bool(saved)).as_bool().unwrap_or(saved);
    let refs: Vec<&Value> = logins.iter().collect();
    let in_use = rules::cloud_account_in_use(list(cloud, "accounts"), engine.id()).map(|a| rules::account_label(a, &refs));
    let mode = if shown { t(cloud, "mode").filter(|m| *m != "mac").or(t(cloud, "whenOn")).unwrap_or("full") } else { "mac" };
    let target = rules::cloud_toggle_target(cloud);
    let switch = kit::switch(shown, move |s: &mut State, to, _| {
        s.settings.wanted.insert("sw:cloud".into(), Value::Bool(to));
        let payload = json!({ "key": "cloudRuns", "value": target });
        super::press(s, "cloud-runs", vec![("zero:settings-set-workspace", payload)]);
    });
    Group::new("Cloud", vec![kit::line("Use the cloud", Some(rules::cloud_runs_line(mode, in_use.as_deref()).into()), vec![switch])]).find("cloud-accounts").el()
}

/// One place agents run: its heading with what stands beside it, one line, and a tile per account
/// (`Place`, `PlaceTiles`). `add` names the agent whose Add account the tiles end with.
fn place(s: &State, name: &str, action: Option<El>, line: Option<String>, tiles: &[Tile], engine: Option<Engine>, column: f32) -> El {
    let mut body = Vec::new();
    if let Some(line) = line {
        body.push(col().mt(-4.0).mb(12.0).child(pen(text(line).font(TextStyle::new(12.5, 400.0).lh(18.75)).color(dim()).wrap(), 12.5)));
    }
    let tile_w = (column - 10.0) / 2.0;
    let head = rules::usage_checked_at(Some(tiles.iter().filter_map(|t| t.checked_at).fold(0.0, f64::max)).filter(|a| *a > 0.0), s.now);
    let mut grid = col().gap(10.0);
    for pair in tiles.chunks(2) {
        grid = grid.child(row().gap(10.0).items_stretch().children(pair.iter().map(|tile| account_tile(s, tile, tile_w, &head))));
    }
    // Antigravity holds one sign-in, which signing in again replaces: no Add account.
    if let Some(engine) = engine.filter(|e| *e != Engine::Antigravity) {
        let add = row()
            .w(column)
            .py(11.0)
            .px(16.0)
            .justify_center()
            .radius(10.0)
            .border(1.0, edge())
            .border_dash(3.0, 3.0)
            .border_layout()
            .cursor(Cursor::Pointer)
            .hover_group()
            .child(pen(text("Add account").font(sans(13.0, 400.0)).color(muted()).group_color(ink()), 13.0))
            // Signing in is done in a terminal, which only the web window has.
            .on_press_tx(move |s: &mut State, tx| super::bridge(s, engine.page(), tx));
        grid = grid.child(add);
    }
    body.push(grid);
    let mut group = Group::new(name, body).find("usage").bare();
    if let Some(action) = action {
        group = group.action(action);
    }
    group.el()
}

/// One account's tile (`.acct-tile`): its name with In use or the button that switches to it,
/// its plan and where it works, then its meters, or why it cannot run.
fn account_tile(s: &State, tile: &Tile, width: f32, head: &str) -> El {
    let on_ = tile.in_use;
    // The address gives way to what follows it, and its end goes under an ellipsis.
    let mut top = row().h(20.0).gap(12.0).items_center().child(row().min_w(0.0).shrink().child(pen(text(tile.name.as_str()).font(sans(13.5, 500.0)).color(ink()).truncate(), 13.5)));
    if on_ {
        top = top.child(kit::badge("In use", true).ml(-2.0));
    } else if let Some((word, act)) = &tile.use_ {
        let act = act.clone();
        let button = row()
            .h(24.0)
            .px(10.0)
            .items_center()
            .radius(6.0)
            .fill(edge())
            .hover(rule())
            .cursor(Cursor::Pointer)
            .child(pen(text(word.as_str()).font(TextStyle::new(12.5, 400.0).lh(12.5)).color(ink()), 12.5))
            .on_press_tx(move |s: &mut State, tx| tile_act(s, &act, tx));
        // At the row's end, one gap from the address at the least (`margin-left: auto`).
        top = top.child(row().grow(1.0).justify_end().child(button));
    }
    let mut tile_el = col().w(width).pt(16.0).px(18.0).pb(18.0).radius(10.0).fill(if on_ { white(0.07) } else { white(0.04) }).border(1.0, if on_ { rule() } else { edge() }).border_layout().child(top);
    let plan_line: Vec<String> = tile.plan.iter().cloned().chain(tile.about.iter().cloned()).collect();
    if !plan_line.is_empty() || !tile.places.is_empty() || tile.cloud_act.is_some() {
        let mut about = row().mt(7.0).h(20.0).gap(12.0).items_center();
        if !plan_line.is_empty() {
            about = about.child(row().min_w(0.0).child(pen(text(plan_line.join(" · ")).font(sans(12.0, 400.0)).color(muted()).truncate(), 12.0)));
        }
        for place in &tile.places {
            let (mark, word) = match place {
                Place::Here => (&crate::icons::PLACE_MAC, "Local"),
                Place::Cloud => (&crate::icons::PLACE_CLOUD, "Cloud"),
            };
            about = about.child(row().gap(5.0).items_center().child(icon(mark, 12.0).color(dim())).child(pen(text(word).font(TextStyle::new(10.5, 500.0).mono().lh(13.5).tracking(0.63)).text_case(rune::TextCase::Uppercase).color(dim()), 10.5)));
        }
        if let Some((word, act)) = &tile.cloud_act {
            let act = act.clone();
            about = about.child(pen(text(word.as_str()).font(sans(12.0, 400.0)).color(muted()), 12.0).cursor(Cursor::Pointer).on_press_tx(move |s: &mut State, tx| tile_act(s, &act, tx)));
        }
        tile_el = tile_el.child(about);
    }
    let quiet = |words: &str, gap: f32| col().mt(gap).child(pen(text(words).font(TextStyle::new(12.0, 400.0).lh(18.0)).color(dim()).wrap(), 12.0));
    if let Some(trouble) = &tile.trouble {
        tile_el = tile_el.child(quiet(trouble, 16.0));
    } else if tile.meters {
        let limits = tile.limits.clone().unwrap_or_default();
        if tile.usage_error.is_none() || !limits.is_empty() {
            tile_el = tile_el.child(meters(&limits, s.now, width - 38.0, tile.none.as_deref()));
        }
        if let Some(error) = &tile.usage_error {
            tile_el = tile_el.child(quiet(error, 16.0));
        }
    }
    if rules::own_time(tile, head, s.now) {
        tile_el = tile_el.child(el().mt(12.0).child(pen(text(rules::usage_checked_at(tile.checked_at, s.now)).font(sans(12.0, 400.0)).color(muted()), 12.0)));
    }
    tile_el
}

/// What a press on a tile's button or link does. Signing in is done in a terminal or on the
/// cloud's own screen, both the web window's, which comes forward on this agent's page.
fn tile_act(s: &mut State, act: &TileAct, tx: &mut rune::Tx) {
    let page = match &s.settings.pane {
        rules::Pane::Page(p) => *p,
        _ => PageId::Claude,
    };
    match act {
        TileAct::Use(profile) => {
            let engine = if page == PageId::Codex { "codex" } else { "claude" };
            press(
                s,
                "switch",
                vec![
                    ("zero:settings-set-workspace", json!({ "key": "activeAccount", "value": { "engine": engine, "profile": profile } })),
                    ("zero:agent-update", json!({ "engine": engine, "action": "refresh" })),
                    ("zero:settings-read", Value::Null),
                ],
            );
        }
        TileAct::SignIn | TileAct::CloudSignIn(_) => super::bridge(s, page, tx),
    }
}

/// The meters of an account (`MiniMeters`): session, week and each model's own, each with its
/// bar, its figure and when it starts over, one grid so the columns line up.
fn meters(limits: &[Value], now: f64, width: f32, none: Option<&str>) -> El {
    let meters = rules::meter_pair(limits, now);
    if meters.is_empty() {
        let words = none.unwrap_or("No usage reading yet");
        return el().mt(16.0).child(pen(text(words).font(sans(12.0, 400.0)).color(muted()), 12.0));
    }
    let font = figures(12.0, 400.0);
    let pct = |m: &rules::Meter| m.percent.map_or_else(|| "–".to_string(), |p| format!("{p}%"));
    let pct_w = meters.iter().map(|m| kit::wide(&pct(m), font)).fold(0.0, f32::max);
    let when_w = meters.iter().map(|m| kit::wide(m.resets.as_deref().unwrap_or(""), font)).fold(0.0, f32::max) + 4.0;
    let bar_w = (width - 56.0 - pct_w - when_w - 30.0).max(1.0);
    let rows = meters.iter().map(|m| {
        let full = m.percent.unwrap_or(0.0) >= 100.0;
        let fill = (m.percent.unwrap_or(0.0).clamp(0.0, 100.0) / 100.0) as f32;
        let bar = el().w(bar_w).h(4.0).radius(2.0).fill(white(0.04)).child(el().w(bar_w * fill).h(4.0).radius(2.0).fill(muted()));
        let mut when = row().w(when_w).h(15.5).justify_end().items_center().child(pen(text(m.resets.clone().unwrap_or_default()).font(font).color(if full { accent() } else { muted() }), 12.0));
        if m.percent.is_none() {
            when = when.help("Reset since this was read");
        } else if let Some(at) = &m.resets {
            when = when.help(format!("Resets {at}"));
        }
        row()
            .h(15.5)
            .gap(10.0)
            .items_center()
            .child(row().w(56.0).child(pen(text(m.word.as_str()).font(font).color(muted()), 12.0)))
            .child(bar)
            .child(row().w(pct_w).justify_end().child(pen(text(pct(m)).font(font).color(dim()), 12.0)))
            .child(when)
    });
    col().mt(16.0).gap(10.0).children(rows)
}

/// The Status group, drawn only when something is wrong or the file was chosen by hand
/// (`Connection`): the state, the account it is signed in as, where to get it and where it
/// runs from.
#[allow(clippy::too_many_arguments)]
fn status(s: &State, engine: Engine, found: bool, certain: bool, trouble: Option<&str>, bin: &str, chosen: bool, url: &str) -> El {
    let state = rules::connection_state(found, certain, trouble.is_some());
    let name = engine.name();
    let label = match state {
        Connection::Connected => format!("{name} is connected"),
        Connection::Trouble => format!("{name} is here but not working"),
        Connection::Missing => format!("{name} is not on this computer"),
        Connection::Unsure => format!("{NAME} could not check"),
    };
    let what = if engine == Engine::Antigravity { "agy" } else { name };
    let say = match state {
        Connection::Trouble => trouble.unwrap_or("").to_string(),
        Connection::Connected => match engine {
            Engine::Claude => format!("{NAME} can see it on this computer, so your agents have something to run on. There is nothing for you to set up."),
            Engine::Codex => format!("{NAME} can see it on this computer, so you can put a task on Codex instead of Claude Code. There is nothing for you to set up."),
            Engine::Antigravity => format!("{NAME} can see agy on this computer, so you can put a task on Antigravity. Sign in below once."),
        },
        Connection::Missing => match engine {
            Engine::Claude => format!("{NAME} could not find Claude Code on this computer. Install it, then check again."),
            Engine::Codex => format!("{NAME} could not find Codex on this computer, so every task runs on Claude Code. Install it, then check again."),
            Engine::Antigravity => format!("{NAME} could not find agy, Antigravity's command line, on this computer. Install it, then check again."),
        },
        Connection::Unsure => format!("{NAME} has not been able to look for {what} on this computer. That is not the same as it being missing, only that the answer never came back. Try again in a moment."),
    };
    let channel = match engine {
        Engine::Claude => "zero:claude-recheck",
        Engine::Codex => "zero:codex-recheck",
        Engine::Antigravity => "zero:antigravity-recheck",
    };
    let checking = s.settings.busy.contains("check");
    let check = if checking { kit::ghost("Looking", false) } else { kit::ghost("Check again", true).on_press(move |s: &mut State| press(s, "check", vec![(channel, Value::Null), ("zero:settings-read", Value::Null)])) };
    let dot = el().size(7.0, 7.0).radius(3.5).fill(if state == Connection::Connected { rgb(123, 201, 142) } else { muted() });
    let word = pen(text(rules::connection_word(state)).font(sans(13.0, 400.0)).color(muted()), 13.0);
    let mut rows = vec![kit::line(&label, Some(say.into()), vec![dot, word, check])];
    if !found && certain {
        let link = match engine {
            Engine::Claude => "Get Claude Code",
            Engine::Codex => "Get Codex",
            Engine::Antigravity => "Get Antigravity",
        };
        let get_say = match engine {
            Engine::Claude => "It takes a few minutes. Come back here and press Check again, and your agents can run.",
            Engine::Codex => "It takes a few minutes. Come back here and press Check again, and Codex is a choice on every task.",
            Engine::Antigravity => "It takes a minute. Come back here and press Check again, and Antigravity is a choice on every task.",
        };
        let url = url.to_string();
        rows.push(kit::line("Where to get it", Some(get_say.into()), vec![kit::ghost(link, true).on_press(move |_: &mut State| crate::files::open_with_system(&url))]));
    }
    if found && !bin.is_empty() {
        rows.push(kit::line("Runs from", Some(format!("{NAME} runs it from {bin}.").into()), Vec::new()));
    }
    // Pointing at the file opens the system's file panel from the desktop app, which the web
    // window asks for: it comes forward on this page.
    let page = engine.page();
    let (label, say, button) = if chosen {
        ("You chose this one", if found { format!("You showed {NAME} where it is, so it stopped looking. Let it look again if that was wrong.") } else { format!("The file you chose is not there any more. Choose another, or let {NAME} look.") }, "Look for it instead")
    } else {
        ("Keep it somewhere else?", format!("If {NAME} is looking in the wrong place, show it where {what} is."), "Choose the file")
    };
    rows.push(kit::line(label, Some(say.into()), vec![kit::ghost(button, true).on_press_tx(move |s: &mut State, tx| super::bridge(s, page, tx))]));
    Group::new("Status", rows).find("status").el()
}
