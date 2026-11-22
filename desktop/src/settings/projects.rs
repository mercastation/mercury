//! Settings › Projects (`components/ProjectsPage.tsx`, `projects-page.css`): every project in
//! the order its agents work them, which the arrows and To top change, its search, Select to
//! archive several, and the archived ones folded away; and a project's own page (the end of
//! `Settings.tsx`). And the phone's menu of projects, which lists them too.

use std::rc::Rc;
use std::time::Duration;

use rune::{Cursor, El, TextEdit, TextStyle, Tx, col, el, icon, row, text, text_input};
use serde_json::{Value, json};

use super::kit::{self, Desc, Group, Picker};
use super::{Editing, Edits, marks, write_other, write_project};
use crate::State;
use crate::rules::settings::{self as rules, PageId, Pane, list, num, on, text as t};
use crate::theme::*;

/// What the Projects page holds while it is open (`ProjectsPage`'s state).
#[derive(Default)]
pub struct Ui {
    pub filter: TextEdit,
    pub selecting: bool,
    pub picked: Vec<String>,
    /// The projects just archived from Select, and when the Undo line for them goes.
    pub undone: Option<(Vec<String>, f64)>,
    pub show_archived: bool,
    /// The order shown at once after a move, and the saved order it was made from: the move
    /// shows until the engine's order says otherwise.
    pub pending: Option<(Vec<String>, Vec<String>)>,
    /// The row that just moved, washed until then, so the eye can follow it.
    pub moved: Option<(String, f64)>,
    /// The row under the pointer, whose buttons come up to the dim ink and To top into view
    /// (`.pp-row:hover .pp-acts button`): a second level of hover, which Rune's groups lack.
    pub hovered: Option<String>,
}

/// A project's mark (`ProductMark` with no picture): its own colour, a square inside the box a
/// picture would have filled, rounded to a disc at these sizes.
pub fn mark(slug: &str, size: f32) -> El {
    let (r, g, b) = crate::rules::window::swatch(slug);
    let side = (size * 0.46).round().max(8.0);
    row().size(size, size).center().child(el().size(side, side).radius(6.0_f32.min(side / 2.0)).fill(rgb(r, g, b)))
}

/// Every project as the order has it, then the ones the order never heard of (`rows`).
struct Line {
    slug: String,
    name: String,
    dir: String,
    repo: Option<String>,
}

fn saved_order(s: &State) -> Vec<String> {
    crate::rules::compose::rank_products(&s.snap.products, &s.snap.supervisor.product_order).iter().filter(|p| !p.rest.get("practice").is_some_and(|v| v.as_bool() == Some(true))).map(|p| p.slug.clone()).collect()
}

fn order(s: &State) -> Vec<String> {
    let saved = saved_order(s);
    match &s.settings.projects.pending {
        Some((shown, was)) if *was == saved => shown.clone(),
        _ => saved,
    }
}

fn lines(s: &State) -> Vec<Line> {
    let details = s.settings.projects();
    let archived: Vec<&str> = s.settings.model.as_ref().map(|m| list(m, "archivedProjects").iter().filter_map(|a| t(a, "slug")).collect()).unwrap_or_default();
    let detail = |slug: &str| details.iter().find(|d| t(d, "slug") == Some(slug));
    let mut out = Vec::new();
    for slug in order(s) {
        if archived.contains(&slug.as_str()) {
            continue;
        }
        let product = s.snap.products.iter().find(|p| p.slug == slug);
        let d = detail(&slug);
        let name = d.and_then(|d| t(d, "name")).map(str::to_string).or_else(|| product.map(|p| p.name.clone())).unwrap_or_else(|| slug.clone());
        let dir = d.and_then(|d| t(d, "dir")).or_else(|| product.and_then(|p| p.rest.get("dir")).and_then(Value::as_str)).unwrap_or("").to_string();
        let repo = d.and_then(|d| t(d, "repoPath")).or_else(|| product.and_then(|p| p.rest.get("repoPath")).and_then(Value::as_str)).map(str::to_string);
        out.push(Line { slug, name, dir, repo });
    }
    let seen: Vec<String> = out.iter().map(|l| l.slug.clone()).chain(s.snap.products.iter().map(|p| p.slug.clone())).collect();
    for d in details {
        let slug = t(d, "slug").unwrap_or("").to_string();
        if seen.contains(&slug) || archived.contains(&slug.as_str()) {
            continue;
        }
        out.push(Line { name: t(d, "name").unwrap_or(&slug).to_string(), dir: t(d, "dir").unwrap_or("").to_string(), repo: t(d, "repoPath").map(str::to_string), slug });
    }
    out
}

/// Writes a new order: it shows at once, and the engine's comes back with the snapshot.
fn commit(s: &mut State, slug: &str, next: Option<Vec<String>>, tx: &mut Tx) {
    let Some(next) = next else { return };
    s.settings.projects.undone = None;
    s.settings.projects.pending = Some((next.clone(), saved_order(s)));
    s.settings.projects.moved = Some((slug.to_string(), tx.clock() + 0.9));
    tx.wake_after(Duration::from_millis(900));
    super::quiet(s, "zero:set-product-order", json!({ "order": next }));
}

/// The small capitals of the table (`.pp-head`, `.pp-num`, `.pp-top`).
fn capitals(words: &str, size: f32, line: f32, spacing: f32, c: rune::Color) -> El {
    pen(text(words).font(TextStyle::new(size, 400.0).mono().lh(line).tracking(spacing)).text_case(rune::TextCase::Uppercase).color(c), size)
}

pub fn list_page(s: &State, column: f32) -> Vec<El> {
    let ui = &s.settings.projects;
    let rows = lines(s);
    let full = order(s);
    let all: Vec<String> = rows.iter().map(|l| l.slug.clone()).collect();
    let needle = ui.filter.text.trim().to_lowercase();
    let shown: Vec<&Line> = rows.iter().filter(|p| needle.is_empty() || [p.name.as_str(), p.slug.as_str(), p.dir.as_str(), p.repo.as_deref().unwrap_or("")].iter().any(|f| f.to_lowercase().contains(&needle))).collect();
    let shown_slugs: Vec<String> = shown.iter().map(|l| l.slug.clone()).collect();
    let home = s.settings.workspace().and_then(|w| t(w, "homePath")).map(str::to_string);

    let mut out = vec![
        kit::title("Projects"),
        kit::lede("Agents work from the top down, and a project’s place outranks a task’s own priority. Drag a project or use its arrows to move it, and open it to rename it or change its rules."),
    ];
    // The search, Select and New project (`.proj-index-top`).
    let field = text_input(&ui.filter, sans(13.5, 400.0))
        .key("projects-find")
        .single_line()
        .grow(1.0)
        .color(ink())
        .placeholder("Search projects", muted())
        .caret(ink(), rgba(110, 165, 255, 0.3))
        .on_edit(|s: &mut State| &mut s.settings.projects.filter)
        .on_input_key(|s: &mut State, e: &rune::KeyEvent, _tx: &mut Tx| {
            // Escape empties the search before it does anything else.
            if e.key == rune::edit::Key::Escape && !s.settings.projects.filter.text.is_empty() {
                s.settings.projects.filter = TextEdit::new("");
                s.settings.projects.filter.focused = true;
                return true;
            }
            false
        });
    let find = row()
        .grow(1.0)
        .min_w(0.0)
        .h(32.0)
        .pl(34.0)
        .pr(32.0)
        .items_center()
        .radius(6.0)
        .fill(kit::hover_ground())
        .border(1.0, if ui.filter.focused { muted() } else { edge() })
        .border_layout()
        .cursor(Cursor::Text)
        .child(el().absolute(10.0, 7.5).child(icon(&marks::FIND, 15.0).color(if ui.filter.focused { dim() } else { muted() })))
        // Chromium sits an input's words on the whole pixel above where centring puts them.
        .child(pen(field, 13.5).offset(0.0, -0.5))
        .on_press_tx(|s: &mut State, tx| {
            s.settings.projects.filter.focused = true;
            tx.focus("projects-find", false);
        });
    // Select and Cancel share one width, so the search never twitches; the word keeps to the
    // left, as the web window's centring does nothing on a flex button (`.pp-select`).
    let select = kit::ghost(if ui.selecting { "Cancel" } else { "Select" }, true).min_w(90.0).on_press(|s: &mut State| {
        let ui = &mut s.settings.projects;
        ui.selecting = !ui.selecting;
        ui.picked.clear();
        ui.undone = None;
    });
    // A new project is made on the web window's card.
    let new = kit::ghost("New project", true).on_press_tx(|s: &mut State, tx| super::bridge(s, PageId::Projects, tx));
    let mut top = row().mt(4.0).gap(8.0).items_center().child(find);
    if !rows.is_empty() {
        top = top.child(select);
    }
    out.push(top.child(new));

    if rows.is_empty() {
        out.push(el().mt(14.0).py(8.0).px(10.0).child(pen(text("No projects yet.").font(TextStyle::new(12.5, 400.0).lh(18.75)).color(muted()), 12.5)));
    } else if shown.is_empty() {
        out.push(el().mt(14.0).py(8.0).px(10.0).child(pen(text(format!("Nothing here matches “{}”.", ui.filter.text.trim())).font(TextStyle::new(12.5, 400.0).lh(18.75)).color(muted()), 12.5)));
    } else {
        out.push(table(s, &shown, &shown_slugs, &all, &full, home.as_deref(), column));
    }
    if let Some(m) = s.settings.model.as_ref().filter(|m| !list(m, "archivedProjects").is_empty()) {
        out.push(archived(s, list(m, "archivedProjects"), home.as_deref()));
    }
    out
}

/// The table (`.pp-table`): its head, which is also the line Select acts on, and a row per
/// project.
fn table(s: &State, shown: &[&Line], shown_slugs: &[String], all: &[String], full: &[String], home: Option<&str>, column: f32) -> El {
    let ui = &s.settings.projects;
    let head_font = |words: &str, c: rune::Color| capitals(words, 10.5, 16.0, 0.84, c);
    let head = if ui.selecting || ui.undone.as_ref().is_some_and(|(_, until)| s.at < *until) {
        // The same height in all three states, so the list never moves.
        let (said, button, live): (String, String, bool) = if ui.selecting {
            (rules::selection_line(ui.picked.len()), rules::archive_label(ui.picked.len()), !ui.picked.is_empty())
        } else {
            let n = ui.undone.as_ref().map_or(0, |(gone, _)| gone.len());
            (rules::archived_line(n), "Undo".to_string(), true)
        };
        let press = row().mr(4.0).cursor(if live { Cursor::Pointer } else { Cursor::Default }).child(head_font(&button, if live { ink() } else { muted() }).opacity(if live { 1.0 } else { 0.6 }));
        let press = if !live {
            press
        } else if ui.selecting {
            press.on_press(archive_picked)
        } else {
            press.on_press(undo_archive)
        };
        row().px(4.0).pb(10.0).gap(14.0).items_center().border_sides([0.0, 0.0, 1.0, 0.0], edge()).border_layout().child(el().w(22.0)).child(row().grow(1.0).pl(32.0).child(head_font(&said, dim()))).child(press)
    } else {
        row()
            .px(4.0)
            .pb(10.0)
            .gap(14.0)
            .items_center()
            .border_sides([0.0, 0.0, 1.0, 0.0], edge())
            .border_layout()
            .child(row().w(22.0).justify_center().child(head_font("#", muted())))
            .child(row().grow(1.0).pl(32.0).child(head_font("Project", muted())))
            .child(row().w(132.0).mr(30.0).justify_end().child(head_font("Move", muted())))
    };
    let mut list = col();
    for (i, p) in shown.iter().enumerate() {
        let rank = all.iter().position(|slug| *slug == p.slug).unwrap_or(0);
        list = list.child(project_row(s, p, i, rank, shown_slugs, all, full, home, column));
    }
    col().mt(22.0).child(head).child(list)
}

/// One project (`.pp-row`): its rank, which turns to the grip under the pointer, its mark, its
/// name over where its code is, the arrows and To top, and the door at its end. A press on the
/// row opens the project's page; while selecting, it ticks the row instead.
#[allow(clippy::too_many_arguments)]
fn project_row(s: &State, p: &Line, i: usize, rank: usize, shown: &[String], all: &[String], full: &[String], home: Option<&str>, column: f32) -> El {
    let ui = &s.settings.projects;
    let selecting = ui.selecting;
    let picked = ui.picked.contains(&p.slug);
    let first = rank == 0;
    let rank_cell = if selecting {
        // The tick takes the number's place, in ink and not the accent.
        let tick = row().size(14.0, 14.0).center().radius(4.0).border(1.5, if picked { ink() } else { muted() });
        let tick = if picked { tick.fill(ink()).child(icon(&marks::TICK, 12.0).color(page())) } else { tick };
        row().w(22.0).h(20.0).center().child(tick)
    } else {
        let num = capitals(&format!("{:02}", rank + 1), 11.5, 20.0, 0.575, if first { accent() } else { muted() }).group_opacity(1.0, 0.0);
        let mut dots = Vec::new();
        for y in 0..3 {
            for x in 0..2 {
                dots.push(el().absolute(x as f32 * 4.0, y as f32 * 4.0).size(2.0, 2.0).radius(1.0).fill(dim()));
            }
        }
        let grip = el().absolute(8.0, 5.0).size(6.0, 10.0).children(dots).group_opacity(0.0, 1.0);
        // The handle takes the grab, and a press on it opens the project as one on the row does
        // (`.pp-row.can-move .pp-rank`): the drag it starts in the web window is not done here.
        let slug = p.slug.clone();
        row().w(22.0).h(20.0).center().cursor(Cursor::Grab).child(num).child(grip).on_press_tx(move |s: &mut State, tx| super::go(s, Pane::Project(slug.clone()), tx))
    };
    let where_ = p.repo.clone().filter(|r| !r.is_empty()).unwrap_or_else(|| p.dir.clone());
    // The name, underlined under the pointer (`text-decoration-color` turning from clear to
    // `--line-strong`), the line under its words alone: four points under the baseline, and one
    // point thick, as the web window draws its 1.5.
    let underline = el().pin(Some(0.0), None, Some(0.0), Some(-0.16)).h(1.0).fill(rule()).group_opacity(0.0, 1.0);
    let name = row().self_start().min_w(0.0).child(pen(text(p.name.as_str()).font(TextStyle::new(14.0, 400.0).lh(20.0)).color(ink()).truncate(), 14.0)).child(underline);
    let mut words = col().grow(1.0).min_w(0.0).gap(1.0).child(name);
    if !where_.is_empty() {
        words = words.child(pen(text(rules::short_path(&where_, home)).font(TextStyle::new(11.5, 400.0).mono().lh(16.0)).color(muted()).truncate(), 11.5).opacity(if selecting && !picked { 0.7 } else { 1.0 }).help(where_.clone()));
    }
    let mut line = row()
        .key(format!("project-{}", p.slug))
        .w(column)
        .py(15.0)
        .px(4.0)
        .gap(14.0)
        .items_center()
        .cursor(if selecting { Cursor::Default } else { Cursor::Pointer })
        .hover_group()
        .child(rank_cell)
        .child(mark(&p.slug, 18.0))
        .child(words);
    if ui.moved.as_ref().is_some_and(|(slug, until)| *slug == p.slug && s.at < *until) {
        line = line.fill(white(0.04));
    }
    if !selecting {
        // Quiet at rest, so the page says it can be changed before anyone points at it; up to
        // the dim ink on the row in hand, and the text's own under the pointer.
        // A disabled one keeps the arrow and takes the press, as a disabled button does. Two
        // levels of hover, the row's and the button's, and Rune's groups answer one: the row in
        // hand is the screen's own state (`hovered`), the button's is its group.
        let in_hand = ui.hovered.as_deref() == Some(p.slug.as_str());
        let quiet = if in_hand { dim() } else { muted() };
        let act = |e: El, live: bool| {
            if live { e.cursor(Cursor::Pointer).hover_group().child(crate::feel::edge_lit(6.0, rule(), true)) } else { crate::feel::answers(e.opacity(0.25).cursor(Cursor::Default)) }
        };
        let to_top = |words: El| row().h(26.0).px(8.0).items_center().radius(6.0).border(1.0, rgba(0, 0, 0, 0.0)).border_layout().child(words);
        // The first row's is there and unseen, and takes the pointer with the arrow (`.pp-row
        // .pp-top:disabled`); Rune lets a wholly transparent element take none, so its words are.
        // Another row's comes into view with the row in hand.
        let to_top = if first {
            crate::feel::answers(to_top(capitals("To top", 10.5, 10.5, 0.84, rgba(0, 0, 0, 0.0))).cursor(Cursor::Default))
        } else {
            act(to_top(capitals("To top", 10.5, 10.5, 0.84, if in_hand { quiet } else { rgba(0, 0, 0, 0.0) }).group_color(ink())), true)
        };
        let arrow = |mark: &'static rune::Icon, live: bool| act(row().size(26.0, 26.0).center().radius(6.0).border(1.0, rgba(0, 0, 0, 0.0)).border_layout().child(icon(mark, 15.0).color(if live { quiet } else { muted() }).group_color(ink())), live);
        let (slug_up, slug_down, slug_top) = (p.slug.clone(), p.slug.clone(), p.slug.clone());
        let (full_up, full_down, full_top) = (full.to_vec(), full.to_vec(), full.to_vec());
        let (shown_up, shown_down) = (shown.to_vec(), shown.to_vec());
        let top_slug = all.first().cloned();
        let up = arrow(&marks::UP, i > 0).help("Move up (⌥↑)");
        let down = arrow(&marks::DOWN, i + 1 < shown.len()).help("Move down (⌥↓)");
        let up = if i > 0 { up.on_press_tx(move |s: &mut State, tx| commit(s, &slug_up, rules::nudge(&full_up, &shown_up, &slug_up, true), tx)) } else { up };
        let down = if i + 1 < shown.len() { down.on_press_tx(move |s: &mut State, tx| commit(s, &slug_down, rules::nudge(&full_down, &shown_down, &slug_down, false), tx)) } else { down };
        let to_top = if first { to_top } else { to_top.on_press_tx(move |s: &mut State, tx| commit(s, &slug_top, rules::place_before(&full_top, &slug_top, top_slug.as_deref()), tx)) };
        line = line.child(row().w(132.0).gap(4.0).justify_end().items_center().child(to_top).child(up).child(down));
        line = line.child(icon(&marks::OPEN, 16.0).color(muted()).group_opacity(0.5, 1.0));
    }
    let (hovered, slug) = (p.slug.clone(), p.slug.clone());
    line = line.on_hover_change(move |s: &mut State, inside, tx| {
        let ui = &mut s.settings.projects;
        if inside {
            ui.hovered = Some(hovered.clone());
        } else if ui.hovered.as_deref() == Some(hovered.as_str()) {
            ui.hovered = None;
        } else {
            tx.live_only();
        }
    });
    line = line.on_press_tx(move |s: &mut State, tx| {
        if s.settings.projects.selecting {
            let picked = &mut s.settings.projects.picked;
            if let Some(at) = picked.iter().position(|x| *x == slug) {
                picked.remove(at);
            } else {
                picked.push(slug.clone());
            }
        } else {
            super::go(s, Pane::Project(slug.clone()), tx);
        }
    });
    // The hairline between two rows, four points in from either side.
    if i > 0 {
        return el().child(line).child(el().pin(Some(4.0), Some(0.0), Some(4.0), None).h(1.0).fill(edge()));
    }
    line
}

/// Archive what is ticked (`archivePicked`): Select ends, and the head says what went, with Undo.
fn archive_picked(s: &mut State) {
    let batch = std::mem::take(&mut s.settings.projects.picked);
    s.settings.projects.selecting = false;
    s.settings.projects.undone = Some((batch.clone(), s.at + 12.0));
    for slug in batch {
        write_other(s, "archive", "zero:project-archive", json!({ "product": slug, "archived": true }));
    }
}

fn undo_archive(s: &mut State) {
    if let Some((gone, _)) = s.settings.projects.undone.take() {
        for slug in gone {
            write_other(s, "archive", "zero:project-archive", json!({ "product": slug, "archived": false }));
        }
    }
}

/// The archived projects: one quiet line until opened, then each with Bring back (`.pp-archived`).
fn archived(s: &State, gone: &[Value], home: Option<&str>) -> El {
    let open = s.settings.projects.show_archived;
    let toggle = row().px(4.0).cursor(Cursor::Pointer).hover_group().child(capitals(if open { "Hide archived projects" } else { "Show archived projects" }, 10.5, 16.0, 0.84, muted()).group_color(dim())).on_press(|s: &mut State| s.settings.projects.show_archived = !s.settings.projects.show_archived);
    // The button stands on the line of a block 20 points tall, four points down it.
    let mut part = col().mt(28.0).child(row().h(20.0).pt(4.0).items_start().child(toggle));
    if open {
        let rows = gone.iter().map(|p| {
            let slug = t(p, "slug").unwrap_or("").to_string();
            let mut words = col().grow(1.0).min_w(0.0).gap(1.0).child(pen(text(t(p, "name").unwrap_or("")).font(TextStyle::new(14.0, 400.0).lh(20.0)).color(dim()).truncate(), 14.0));
            if let Some(dir) = t(p, "dir") {
                words = words.child(pen(text(rules::short_path(dir, home)).font(TextStyle::new(11.5, 400.0).mono().lh(16.0)).color(muted()).truncate(), 11.5).help(dir.to_string()));
            }
            let back = kit::ghost("Bring back", true).on_press(move |s: &mut State| write_other(s, "archive", "zero:project-archive", json!({ "product": slug, "archived": false })));
            row().pt(12.0).pr(4.0).pb(12.0).pl(36.0).gap(14.0).items_center().border_sides([0.0, 0.0, 1.0, 0.0], edge()).border_layout().child(words).child(back)
        });
        part = part.child(col().mt(10.0).border_sides([1.0, 0.0, 0.0, 0.0], edge()).border_layout().children(rows));
    }
    part
}

/* ------------------------------- a project's own page ------------------------------ */

pub fn project_page(s: &State, _column: f32) -> Vec<El> {
    let Some(p) = s.settings.current() else {
        // A project gone while its page was open (archived, renamed): the list.
        return list_page(s, _column);
    };
    let slug = t(p, "slug").unwrap_or("").to_string();
    let name = t(p, "name").unwrap_or("").to_string();
    let w = s.settings.workspace().cloned().unwrap_or(Value::Null);
    let two = list(&w, "engineChoices").len() > 1;
    let crumb = row()
        .ml(-6.0)
        .mb(10.0)
        .py(3.0)
        .px(6.0)
        .gap(4.0)
        .items_center()
        .radius(6.0)
        .cursor(Cursor::Pointer)
        .hover_group()
        .hover(kit::hover_ground())
        .child(icon(&marks::BACK, 13.0).color(muted()).group_color(ink()))
        .child(pen(text("Projects").font(TextStyle::new(12.5, 400.0).lh(16.0)).color(muted()).group_color(ink()), 12.5))
        .on_press_tx(|s: &mut State, tx| super::go(s, Pane::Page(PageId::Projects), tx));
    let mut out = vec![row().child(crumb), title_row(s, &slug, &name)];

    // How it runs.
    let mut rows = Vec::new();
    let autonomous = s.settings.shown(&format!("pj:{slug}:autonomous"), p.get("autonomous").unwrap_or(&Value::Bool(false))).as_bool().unwrap_or(false);
    let auto_slug = slug.clone();
    rows.push(kit::line(
        "Let agents start the tasks they file here",
        Some("A task an agent writes on this project gets a session straight away. Without this it waits in your inbox for your yes. Questions and reviews always wait for you either way.".into()),
        vec![kit::switch(autonomous, move |s: &mut State, to, _| write_project(s, &format!("pj:{auto_slug}:autonomous"), &auto_slug, "autonomous", Value::Bool(to)))],
    ));
    let accounts = list(&w, "accounts");
    let account = t(p, "account").unwrap_or("any").to_string();
    if accounts.len() > 1 || account != "any" {
        let mut options = vec![("any".to_string(), "The account in use".to_string())];
        options.extend(accounts.iter().map(|a| {
            let label = t(a, "label").unwrap_or("").to_string();
            (t(a, "profile").unwrap_or("").to_string(), t(a, "email").map_or(label.clone(), |e| format!("{e} ({label})")))
        }));
        let shown = s.settings.shown(&format!("pk:{slug}:account"), &Value::String(account)).as_str().unwrap_or("any").to_string();
        let pick_slug = slug.clone();
        rows.push(kit::line(
            "Claude account",
            Some("Every Claude Code agent on this project runs on this account and on no other. If the account cannot run, the work waits.".into()),
            vec![kit::picker(
                s,
                Picker {
                    key: "project-account".into(),
                    above: None,
                    value: shown,
                    options,
                    long: false,
                    help: Some("The Claude account this project's agents run on.".into()),
                    pick: Rc::new(move |s: &mut State, v: String, _| write_project(s, &format!("pk:{pick_slug}:account"), &pick_slug, "account", Value::String(v))),
                },
            )],
        ));
    }
    let permission = t(p, "permission").unwrap_or("workspace");
    if permission != "workspace" {
        let said = if permission == "custom" { rules::CUSTOM_SENTENCE.to_string() } else { format!("{} {}", rules::mode_what(&rules::CLAUDE_MODES, permission).unwrap_or(""), rules::RULES_ALWAYS_APPLY) };
        let back = slug.clone();
        rows.push(kit::line(
            if two { "This project has its own Claude Code permissions" } else { "This project has its own permissions" },
            Some(said.into()),
            vec![kit::ghost("Use the workspace setting", true).on_press(move |s: &mut State| write_project(s, &format!("pj:{back}:permission"), &back, "permission", json!("workspace")))],
        ));
    }
    let codex = t(p, "codexMode").unwrap_or("workspace");
    if two && codex != "workspace" {
        let back = slug.clone();
        rows.push(kit::line(
            "This project has its own Codex permissions",
            Some(rules::mode_what(&rules::CODEX_MODES, codex).unwrap_or("").into()),
            vec![kit::ghost("Use the workspace setting", true).on_press(move |s: &mut State| write_project(s, &format!("pj:{back}:codexMode"), &back, "codexMode", json!("workspace")))],
        ));
    }
    out.push(Group::new("How it runs", rows).el());

    // Its instructions are the web window's to write.
    let words = t(p, "instructions").map(str::trim).unwrap_or("");
    let open = kit::ghost("Open", true).on_press_tx(|s: &mut State, tx| super::bridge(s, PageId::Instructions, tx));
    out.push(Group::new("Instructions", vec![kit::line("Instructions for this project", Some(if words.is_empty() { "None yet.".into() } else { rules::size_label(t(p, "instructions").unwrap_or("").chars().count()).into() }), vec![open])]).el());

    out.push(Group::new("Browser", browser_rows(s, p, &slug)).el());

    let mut kept = vec![kit::line("Documents", Some(t(p, "dir").unwrap_or("").into()), Vec::new())];
    let repo = t(p, "repoPath");
    kept.push(kit::line("Repository", Some(repo.unwrap_or("No code repo registered for this project.").into()), Vec::new()));
    if repo.is_some() {
        let branches = s.settings.reads.branches.as_ref().filter(|(of, _)| *of == slug).map(|(_, b)| b);
        let main = branches.and_then(|b| t(b, "main"));
        let base = t(p, "baseBranch").unwrap_or("").to_string();
        let mut options = vec![(String::new(), match (branches, main) {
            (Some(_), None) => "Where the checkout is (default)".to_string(),
            (_, main) => format!("{} (default)", main.unwrap_or("main")),
        })];
        let names: Vec<String> = match branches {
            Some(b) => list(b, "branches").iter().filter_map(Value::as_str).map(str::to_string).collect(),
            None => (!base.is_empty()).then(|| base.clone()).into_iter().collect(),
        };
        options.extend(names.into_iter().filter(|b| Some(b.as_str()) != main).map(|b| (b.clone(), b)));
        let shown = s.settings.shown(&format!("pk:{slug}:branch"), &Value::String(base)).as_str().unwrap_or("").to_string();
        let pick_slug = slug.clone();
        kept.push(kit::line(
            "New threads start from",
            Some("The branch each new thread's folder is cut from. The new thread card can pick another one for a single thread.".into()),
            vec![kit::picker(
                s,
                Picker {
                    key: "project-branch".into(),
                    above: None,
                    value: shown,
                    options,
                    long: true,
                    help: Some("The branch new threads start from".into()),
                    pick: Rc::new(move |s: &mut State, v: String, _| write_project(s, &format!("pk:{pick_slug}:branch"), &pick_slug, "baseBranch", Value::String(v))),
                },
            )],
        ));
    }
    out.push(Group::new("Where this project is kept on this computer", kept).el());

    let running = num(p, "running").is_some_and(|n| n > 0.0);
    let gone = slug.clone();
    out.push(
        Group::new(
            "Archive",
            vec![kit::line(
                "Archive this project",
                Some(format!("It leaves your inbox, the sidebar and Projects, and no new agents start on it{}. Its files stay where they are, and you can bring it back from Projects.", if running { "; the ones running now finish" } else { "" }).into()),
                vec![kit::ghost("Archive", true).on_press(move |s: &mut State| write_other(s, "archive", "zero:project-archive", json!({ "product": gone, "archived": true })))],
            )],
        )
        .el(),
    );
    out
}

/// The mark and the name at the head of a project's page (`.set-title-row`): the name is where
/// it is renamed, a press putting it in a box, Enter keeping it and Escape putting it back.
fn title_row(s: &State, slug: &str, name: &str) -> El {
    // Its picture is chosen in the system's file panel, which the web window asks for.
    let icon_pick = row()
        .size(28.0, 28.0)
        .center()
        .child(row().size(34.0, 34.0).center().radius(6.0).border(1.0, rgba(0, 0, 0, 0.0)).cursor(Cursor::Pointer).hover_group().hover(kit::hover_ground()).child(crate::feel::edge_lit(6.0, edge(), false)).child(mark(slug, 26.0).opacity(0.9)).help("Give this project a picture").on_press_tx(|s: &mut State, tx| super::bridge(s, PageId::Projects, tx)));
    let editing = s.settings.editing.as_ref().filter(|e| e.what == Edits::Name(slug.to_string()));
    let name_el = match editing {
        Some(e) => {
            let field = text_input(&e.words, sans(24.0, 400.0).tracking(-0.84))
                .key("project-name")
                .single_line()
                .color(ink())
                .caret(ink(), rgba(110, 165, 255, 0.3))
                .on_edit(|s: &mut State| &mut s.settings.editing.as_mut().expect("the box is open").words)
                .on_submit(rename)
                .on_input_key(|s: &mut State, e: &rune::KeyEvent, tx: &mut Tx| {
                    // Escape puts the name back, and nothing else takes the keyboard.
                    if e.key == rune::edit::Key::Escape {
                        s.settings.editing = None;
                        tx.blur();
                        return true;
                    }
                    false
                });
            row().ml(-7.0).py(2.0).px(7.0).min_w(160.0).radius(6.0).fill(page()).border(1.0, accent()).border_layout().child(pen(field, 24.0))
        }
        None => {
            let (slug, name) = (slug.to_string(), name.to_string());
            let start = {
                let (slug, name) = (slug.clone(), name.clone());
                move |s: &mut State, tx: &mut Tx| {
                    let mut words = TextEdit::new(name.clone());
                    words.focused = true;
                    // Selected, not merely focused: the common rename replaces the whole name.
                    (words.anchor, words.focus) = (0, words.text.len());
                    s.settings.editing = Some(Editing { what: Edits::Name(slug.clone()), words });
                    tx.focus_when_shown("project-name", false);
                }
            };
            // The pencil is no word, so the arrow stands over it; a press on it renames too.
            let pencil = el().ml(8.4).cursor(Cursor::Default).on_press_tx(start.clone()).child(icon(&marks::PENCIL, 13.2).color(muted()).group_color(ink()).group_opacity(0.7, 1.0));
            row()
                .ml(-7.0)
                .py(2.0)
                .px(7.0)
                .items_center()
                .radius(6.0)
                .border(1.0, rgba(0, 0, 0, 0.0))
                .border_layout()
                .cursor(Cursor::Text)
                .hover_group()
                .hover(kit::hover_ground())
                .help("Click to rename this project")
                .child(pen(text(name.as_str()).font(sans(24.0, 400.0).tracking(-0.84)).color(ink()), 24.0))
                .child(pencil)
                .on_press_tx(start)
        }
    };
    row().gap(12.0).items_center().child(icon_pick).child(name_el)
}

/// Enter in the name's box, or a press anywhere else that takes its keyboard away (`onBlur`, as
/// Finder saves a name): the new name is written unless it is empty or the same.
pub fn rename(s: &mut State, tx: &mut Tx) {
    let Some(Editing { what: Edits::Name(slug), words }) = s.settings.editing.take() else { return };
    tx.blur();
    let next = words.text.trim().to_string();
    let was = s.settings.current().and_then(|p| t(p, "name")).unwrap_or("").to_string();
    if !next.is_empty() && next != was {
        write_other(s, "rename", "zero:project-rename", json!({ "product": slug, "name": next }));
    }
}

/// A project's browser (`ProjectBrowserRows`): the sites its agents may open, edited in place,
/// and its owner's sign-in window once one is listed.
fn browser_rows(s: &State, p: &Value, slug: &str) -> Vec<El> {
    let sites: Vec<String> = list(p, "browserSites").iter().filter_map(Value::as_str).map(str::to_string).collect();
    let listed = sites.join(", ");
    let mut rows = Vec::new();
    match s.settings.editing.as_ref().filter(|e| e.what == Edits::Sites(slug.to_string())) {
        Some(e) => {
            let field = text_input(&e.words, sans(13.0, 400.0))
                .key("project-sites")
                .single_line()
                .grow(1.0)
                .color(ink())
                .placeholder("acme.atlassian.net, grafana.acme.dev", muted())
                .caret(ink(), rgba(110, 165, 255, 0.3))
                .on_edit(|s: &mut State| &mut s.settings.editing.as_mut().expect("the box is open").words)
                .on_submit(save_sites)
                .on_input_key(|s: &mut State, e: &rune::KeyEvent, tx: &mut Tx| {
                    if e.key == rune::edit::Key::Escape {
                        s.settings.editing = None;
                        tx.blur();
                        return true;
                    }
                    false
                });
            let changed = e.words.text.trim() != listed;
            let save = if changed { kit::ghost("Save", true).on_press_tx(save_sites) } else { kit::ghost("Save", false) };
            let cancel = kit::ghost("Cancel", true).on_press(|s: &mut State| s.settings.editing = None);
            let said = s.settings.said.get("sites").cloned();
            let mut form = col().py(14.0).gap(9.0).child(pen(text("Sites its agents may open").font(sans(14.0, 400.0)).color(ink()), 14.0)).child(
                row().gap(9.0).items_center().child(row().grow(1.0).min_w(0.0).h(32.0).px(10.0).items_center().radius(6.0).fill(rgb(40, 38, 37)).border(1.0, muted()).border_layout().child(pen(field, 13.0))).child(save).child(cancel.ml(8.0)),
            );
            if let Some(said) = said {
                form = form.child(pen(text(said).font(TextStyle::new(12.5, 400.0).lh(17.0)).color(dim()).wrap(), 12.5));
            }
            rows.push(form);
        }
        None => {
            let (slug, listed) = (slug.to_string(), listed.clone());
            let change = kit::ghost(if sites.is_empty() { "Add a site" } else { "Change" }, true).on_press_tx(move |s: &mut State, tx| {
                let mut words = TextEdit::new(listed.clone());
                words.focused = true;
                s.settings.editing = Some(Editing { what: Edits::Sites(slug.clone()), words });
                s.settings.said.remove("sites");
                tx.focus_when_shown("project-sites", false);
            });
            rows.push(kit::line("Sites its agents may open", Some(rules::sites_line(&sites).into()), vec![change]));
        }
    }
    if !sites.is_empty() {
        let browser = s.settings.reads.browser.as_ref().filter(|(of, _)| of == slug).map(|(_, b)| b);
        let busy = s.settings.busy.contains("browser");
        let (word, done) = rules::sign_in_button(browser);
        let slug = slug.to_string();
        let button = if busy { kit::ghost(word, false) } else { kit::ghost(word, true).on_press(move |s: &mut State| sign_in(s, &slug, if done { "done" } else { "sign-in" })) };
        rows.push(kit::line("Sign in to its browser", Some(Desc::Plain(rules::sign_in_line(browser, busy))), vec![button]));
    }
    rows
}

/// Saves the sites typed (`save`): the engine reads the list out of the words.
fn save_sites(s: &mut State, tx: &mut Tx) {
    let Some(Editing { what: Edits::Sites(slug), words }) = s.settings.editing.take() else { return };
    tx.blur();
    write_other(s, "sites", "zero:settings-set-project", json!({ "product": slug, "key": "browserSites", "value": words.text }));
}

/// The project's browser opened for its owner to sign in, or closed when they are done.
fn sign_in(s: &mut State, slug: &str, action: &'static str) {
    super::browser_press(s, slug, action);
}

/* ------------------------------- the phone's projects ------------------------------- */

/// Which projects the phone has (`PhoneProjectsPicker`): Every project, None, or a tick for each.
/// Its rows say more than a picker's do, and a pick leaves it open.
pub fn phone_picker(s: &State) -> El {
    let w = s.settings.workspace().cloned().unwrap_or(Value::Null);
    let saved_all = w.get("phoneAll") != Some(&Value::Bool(false));
    let projects: Vec<(String, String, bool)> = s.settings.projects().iter().filter(|p| t(p, "slug") != Some("practice")).map(|p| (t(p, "slug").unwrap_or("").to_string(), t(p, "name").unwrap_or("").to_string(), on(p, "onPhone"))).collect();
    // A choice on its way shows at once, as the whole of what it writes.
    let (all, ons): (bool, Vec<(String, bool)>) = match s.settings.wanted.get("pk:phone") {
        Some(want) => {
            let all = want.get("all").and_then(Value::as_bool).unwrap_or(true);
            let except: Vec<&str> = list(want, "except").iter().filter_map(Value::as_str).collect();
            (all, projects.iter().map(|(slug, _, _)| (slug.clone(), all && !except.contains(&slug.as_str()))).collect())
        }
        None => (saved_all, projects.iter().map(|(slug, _, on)| (slug.clone(), *on)).collect()),
    };
    let word = rules::phone_word(all, &ons);
    let open = s.settings.menu.as_deref() == Some("phone-projects");
    let chip_font = sans(13.0, 400.0);
    let chip_w = kit::wide(&word, chip_font) + 8.0 + 9.0 + 20.0 + 2.0;
    s.settings.pickers_drawn("phone-projects");
    let chip = row()
        .key("picker-phone-projects")
        .h(28.0)
        .px(10.0)
        .gap(8.0)
        .items_center()
        .radius(6.0)
        .fill(white(0.04))
        .border(1.0, if open { rule() } else { edge() })
        .border_layout()
        .cursor(Cursor::Pointer)
        .hover_group()
        .child(pen(text(word.as_str()).font(chip_font).color(ink()), 13.0))
        .child(icon(&marks::CHEVRON, 9.0).color(muted()))
        .child(crate::feel::edge_lit(6.0, rule(), true))
        .on_press(|s: &mut State| s.settings.menu = if s.settings.menu.as_deref() == Some("phone-projects") { None } else { Some("phone-projects".into()) });
    let mut wrap = el().child(chip);
    if open {
        let every = all && ons.iter().all(|(_, on)| *on);
        let none = !all || ons.iter().all(|(_, on)| !on);
        let choose = |s: &mut State, all: bool, except: Vec<String>| {
            let value = json!({ "all": all, "except": except });
            super::write_workspace(s, "pk:phone", "phoneProjects", value);
        };
        let two = |name: &str, small: &str, on: bool| {
            let r = row().h(45.0).px(9.0).items_center().radius(6.0).cursor(Cursor::Pointer).hover_group().child(
                col().gap(1.0).child(pen(text(name).font(sans(13.0, if on { 600.0 } else { 400.0 })).color(if on { ink() } else { dim() }).group_color(ink()), 13.0)).child(pen(text(small).font(sans(11.5, 400.0)).color(muted()), 11.5)),
            );
            if on { r.fill(kit::selected()) } else { r.hover(white(0.07)) }
        };
        let mut rows = vec![
            two("Every project", "New ones too", every).on_press(move |s: &mut State| choose(s, true, Vec::new())),
            two("None", "Everything stays on this computer", none).on_press(move |s: &mut State| choose(s, false, Vec::new())),
            el().h(1.0).mx(6.0).my(4.0).fill(edge()),
        ];
        let mut widest = kit::wide("Everything stays on this computer", sans(11.5, 400.0)).max(kit::wide("Every project", sans(13.0, if every { 600.0 } else { 400.0 })));
        for (slug, name, _) in &projects {
            let ticked = ons.iter().any(|(s2, on)| s2 == slug && *on);
            widest = widest.max(14.0 + 9.0 + kit::wide(name, sans(13.0, 400.0)));
            let tick = row().size(14.0, 14.0).center().radius(3.0).border(1.0, if ticked { accent() } else { rule() }).border_layout();
            let tick = if ticked { tick.fill(accent()).child(icon(&marks::TICK, 11.0).color(rgb(255, 255, 255))) } else { tick };
            let (all_now, ons_now, slug) = (all, ons.clone(), slug.clone());
            rows.push(
                row()
                    .h(29.0)
                    .px(9.0)
                    .gap(9.0)
                    .items_center()
                    .radius(6.0)
                    .cursor(Cursor::Pointer)
                    .hover_group()
                    .hover(white(0.07))
                    .child(tick)
                    .child(pen(text(name.as_str()).font(sans(13.0, 400.0)).color(dim()).group_color(ink()), 13.0))
                    .on_press(move |s: &mut State| {
                        let (all, except) = rules::phone_tick(all_now, &ons_now, &slug);
                        choose(s, all, except);
                    }),
            );
        }
        let menu_w = (widest + 18.0 + 8.0 + 2.0).max(chip_w);
        let shell = col()
            .absolute(chip_w - menu_w, 34.0)
            .w(menu_w)
            .radius(10.0)
            .fill(rgb(54, 52, 49))
            .border(1.0, rule())
            .border_layout()
            .box_shadow(shadow(0.0, 24.0, 70.0, 0.0, rgba(0, 0, 0, 0.55), false))
            .raise()
            .child(col().w(menu_w - 2.0).px(4.0).py(4.0).gap(1.0).children(rows));
        wrap = wrap.child(shell);
    }
    wrap
}
