//! What the Inbox's head strip opens: the View and filters menu under the display button
//! (`DisplayMenu` in `renderer/src/threads/Pages.tsx`), the cloud's menu under the cloud button
//! (`CloudButton`), and the question ⌘C asks before it turns the cloud off (`CloudOffAsk`).
//! Drawn from the web window's measurements of each, photographed open
//! (`tools/scenes/inbox-display*.mjs`, `inbox-cloud-menu*.mjs`, `inbox-cloud-invite.mjs`,
//! `inbox-cloud-off-ask.mjs`). The rules behind them are `rules/display.rs` and `rules/cloud.rs`.
//!
//! One menu is open at a time, and only over the Inbox: leaving the page closes it, as the web
//! window's header is made anew. A press on its button closes it and one on the other button
//! opens that one instead; a press anywhere else outside the strip's buttons and the menu
//! closes it on its way down; Escape closes it. Every other key goes on to the page, as the
//! web window's menus take only Escape (`useOutside`): J and K still walk the list, and Tab
//! turns its tabs under the menu, whose foot then counts the new tab.
//!
//! A pick in View and filters changes the list at once and leaves the menu open, and is kept
//! for the next window (`picks.rs`). A pick in the cloud's menu is written at once, on the web
//! window's channels, and the menu says the new state once the engine does.

use rune::edit::Key;
use rune::{Align, Color, Cursor, Drag, El, KeyEvent, Latest, Phase, TextCase, Tx, Waker, col, el, icon, row, text};
use serde_json::Value;

use crate::compose::level_mark;
use crate::data::Tab;
use crate::rules::cloud::{self, Go, Mode};
use crate::rules::display::{self, Display, Sort, View as Shape, Window};
use crate::theme::*;
use crate::{State, View, feel, front, icons, write};

/// The strip's buttons the menus hang from, by the keys they are laid out under, and the row of
/// them, a press on which is theirs and not one outside the open menu.
pub const NEW_THREAD: &str = "inbox-new-thread";
pub const CLOUD_BUTTON: &str = "inbox-cloud";
pub const DISPLAY_BUTTON: &str = "inbox-display";
pub const BUTTONS: &str = "inbox-buttons";

/// The menus' widths (`.th-pop`, `.th-pop.th-cloud-pop`, `.th-pop.th-cloud-invite`).
const DISPLAY_W: f32 = 470.0;
const CLOUD_W: f32 = 540.0;
const INVITE_W: f32 = 360.0;
/// What a line leaves for what it says: its edge, sixteen points each side, the label's column
/// of 92 and the ten between (`grid-template-columns: 92px 1fr`).
const fn says(width: f32) -> f32 {
    width - 2.0 - 32.0 - 92.0 - 10.0
}

/// Which of the two menus is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Menu {
    Display,
    Cloud,
}

#[derive(Default)]
pub struct Head {
    open: Option<Menu>,
    /// The Project line shows every project, not only the eight used last ("Show all").
    all_projects: bool,
    /// The stopped cloud accounts the cloud's menu has shown since it opened (`looked`).
    looked: Vec<String>,
    /// ⌘C's question before turning the cloud off is up.
    pub asking_off: bool,
    /// Agentbox Cloud's plan as its server last said it, for the month's hours, and its reading.
    plan: Option<Value>,
    reading: Latest<Result<Value, String>>,
    plan_asked: bool,
}

impl Head {
    pub fn set_waker(&self, wake: Waker) {
        self.reading.set_waker(wake);
    }
}

/// What the cloud's buttons that the web window answers with its own screen say here, when the
/// app cannot bring that screen forward.
const SETUP_NOT_HERE: &str = "Cloud accounts are set up in Mercury's own window.";
/// The app's door channel that brings its window forward on the cloud's setup.
pub const OPEN_CLOUD_SETUP: &str = "zero:open-cloud-setup-here";

impl State {
    /// The display the Inbox is drawn with: the one kept from the last window, or a picture's.
    pub fn display(&self) -> &Display {
        &self.picks.get().display
    }

    /// A pick in View and filters: the list follows at once and the pick is kept. The row the
    /// keys are on keeps its place in the list, as far as the list now goes.
    pub fn set_display(&mut self, d: Display) {
        if d.view == Shape::Board {
            return;
        }
        self.picks.change(|p| p.display = d);
        self.refresh();
        self.picked = self.picked.min(self.lists.under(self.inbox_tab).len().saturating_sub(1));
    }

    /// The menu open under the strip, while the Inbox is on screen.
    pub fn head_open(&self) -> Option<Menu> {
        if self.view == View::Inbox { self.head.open } else { None }
    }

    /// A press on the display button or the cloud's: its menu opens, or closes when it is the
    /// one open, and the other closes either way (`setOpen((o) => !o)`, and the other's
    /// `useOutside`).
    pub fn toggle_head(&mut self, menu: Menu) {
        let was = self.head_open();
        self.close_head();
        if was == Some(menu) {
            return;
        }
        self.close_thread_menus();
        self.head.open = Some(menu);
        self.head.all_projects = false;
        if menu == Menu::Cloud {
            if let Some(cloud) = self.snap.rest.get("cloud") {
                self.head.looked = cloud::looked_at(cloud);
            }
            self.ask_for_the_plan(true);
        }
    }

    /// Closes the open menu. The cloud's counts the stopped accounts it showed as seen, so the
    /// button stops asking for their sign-in (`seenAfter`), once it is closed and not under it.
    pub fn close_head(&mut self) {
        if self.head.open.take() == Some(Menu::Cloud) {
            let looked = std::mem::take(&mut self.head.looked);
            self.see_cloud_accounts(&looked);
        }
    }

    /// The stopped cloud accounts already looked at: kept by this window, and a picture's own.
    pub fn broken_seen(&self) -> Vec<String> {
        let mut seen = self.picks.get().cloud_broken_seen.clone();
        for k in &self.local.cloud_broken_seen {
            if !seen.contains(k) {
                seen.push(k.clone());
            }
        }
        seen
    }

    fn see_cloud_accounts(&mut self, looked: &[String]) {
        let Some(cloud) = self.snap.rest.get("cloud") else {
            return;
        };
        let next = cloud::seen_after(cloud, &self.picks.get().cloud_broken_seen, looked);
        self.picks.change(|p| p.cloud_broken_seen = next);
    }

    /// The engine said something new: what the open cloud menu shows is looked at, and an
    /// account that works again stops counting as seen (the effect on `cloud` in `CloudButton`).
    pub(crate) fn tend_head(&mut self) {
        let Some(cloud) = self.snap.rest.get("cloud") else {
            return;
        };
        if self.head_open() == Some(Menu::Cloud) {
            for k in cloud::looked_at(cloud) {
                if !self.head.looked.contains(&k) {
                    self.head.looked.push(k);
                }
            }
        } else {
            self.see_cloud_accounts(&[]);
        }
        // A cloud on Agentbox Cloud's server has its month counted there: read once it is seen.
        self.ask_for_the_plan(false);
    }

    /// Asks Agentbox Cloud's server for the plan and the month's hours, off the main thread, as
    /// the web window does while its cloud is the server's (`cloudAccountPlan`): once, and again
    /// each time the menu opens. A picture's plan is its fixture's.
    fn ask_for_the_plan(&mut self, again: bool) {
        let managed = self.snap.rest.get("cloud").is_some_and(|c| crate::data::truthy(c.get("managed")));
        if !managed || (self.head.plan_asked && !again) || self.head.reading.is_pending() {
            return;
        }
        let crate::source::Source::Door(door) = &self.source else {
            return;
        };
        if let Some((url, token)) = door.now() {
            self.head.plan_asked = true;
            self.head.reading.request((url, token), |(url, token)| crate::source::call(&url, &token, "zero:cloud-account-plan", &Value::Null));
        }
    }

    /// The poll's part: the plan the server said lands.
    pub(crate) fn plan_landed(&mut self) -> bool {
        match self.head.reading.take() {
            Some(Ok(plan)) => {
                self.head.plan = Some(plan);
                true
            }
            _ => false,
        }
    }

    /// The plan the month's line reads: the server's, or a picture's (`cloudPlan`).
    fn cloud_plan(&self) -> Option<&Value> {
        self.head.plan.as_ref().or_else(|| self.fixture.get("cloudPlan"))
    }

    /// One of the cloud's own settings written (`onCloudMode`, `onCloudWhenOn`). The menu says
    /// the new state once the engine does.
    pub fn pick_cloud(&mut self, write: (&'static str, Value)) {
        self.write_what_waits();
        self.send(write.0, write.1, write::Sent::Cloud);
    }

    /// Move them: the menu closes and the runs go to the cloud as it has room (`onCloudMove`).
    pub fn move_to_the_cloud(&mut self) {
        let Some(ids) = self.snap.rest.get("cloud").map(cloud::movable) else {
            return;
        };
        self.close_head();
        self.write_what_waits();
        let (channel, payload) = cloud::move_runs(&ids);
        self.send(channel, payload, write::Sent::Moved { asked: ids.len() });
    }

    /// Set up the cloud, Manage and Fix: the web window's own screen for the accounts, which the
    /// app brings forward. The menu closes first (`goSetUp`).
    pub fn set_up_the_cloud(&mut self, tx: &Tx) {
        self.close_head();
        match &self.source {
            crate::source::Source::Door(door) => front::ask_the_app(door, OPEN_CLOUD_SETUP, SETUP_NOT_HERE),
            _ => self.tell(SETUP_NOT_HERE, tx),
        }
    }

    /// Try again on an account that was not added: its sign-in is let go and started again, and
    /// once the engine has answered both the setup opens on it (`retry`, which awaits each
    /// before `goSetUp`).
    pub fn sign_in_again(&mut self, tx: &Tx) {
        let writes = self.snap.rest.get("cloud").map(cloud::sign_in_again).unwrap_or_default();
        self.write_what_waits();
        if writes.is_empty() || !self.send_all(writes, write::Sent::SignIn) {
            self.set_up_the_cloud(tx);
        }
    }

    /// What the engine said to a write of the cloud's: a move says how many go, and a refusal
    /// is said in one line. A sign-in started again opens the setup on it, whatever the engine
    /// said, as the web window's does: the setup says how it went.
    pub(crate) fn cloud_landed(&mut self, landed: write::Landed, tx: &mut Tx) {
        if landed.sent == write::Sent::SignIn {
            self.set_up_the_cloud(tx);
            return;
        }
        if let Some(why) = landed.refusal() {
            self.refused = Some(match landed.sent {
                write::Sent::Moved { .. } => format!("Could not move them to the cloud: {why}"),
                _ => format!("Could not change where threads run: {why}"),
            });
            return;
        }
        if let (write::Sent::Moved { asked }, Ok(answer)) = (&landed.sent, &landed.answer) {
            let line = cloud::moved_line(crate::rules::place::moving(answer, *asked));
            self.say(line, false, tx);
        }
    }

    /// ⌘C: the cloud on or off, from anywhere a key reaches it, with a line saying which; off
    /// asks first, as it was pressed by accident (`toggleCloudRef`). Never while words are
    /// selected, a box has the keys or something stands over the page: then it is Copy
    /// (`cloudChord`). Nothing without a cloud set up.
    pub fn command_c(&mut self, e: &KeyEvent, tx: &mut Tx) -> bool {
        let c = match &e.key {
            Key::Command(c) => c.eq_ignore_ascii_case(&'c'),
            Key::Text(t) => t.eq_ignore_ascii_case("c"),
            _ => false,
        };
        if !c || !(e.mods.cmd || e.mods.ctrl) || e.mods.shift || e.mods.alt {
            return false;
        }
        let boxed = self.reply.focused || self.compose.words.focused || self.later.words.focused || self.palette.words.focused;
        let over = self.palette.open || self.compose.card || self.later.on.is_some() || self.head.asking_off;
        if boxed || over || tx.text_selection().is_some() {
            return false;
        }
        let Some(cloud) = self.snap.rest.get("cloud").filter(|c| crate::data::truthy(c.get("setUp"))) else {
            return true;
        };
        match cloud::toggle_target(cloud) {
            Mode::Mac => self.head.asking_off = true,
            mode => {
                self.pick_cloud(cloud::set_mode(mode));
                self.tell(cloud::toggle_note(mode), tx);
            }
        }
        true
    }

    /// The question's Turn off, or Enter (`turnCloudOff`).
    pub fn turn_the_cloud_off(&mut self, tx: &Tx) {
        self.head.asking_off = false;
        self.pick_cloud(cloud::set_mode(Mode::Mac));
        self.tell(cloud::toggle_note(Mode::Mac), tx);
    }

    /// Keep it on, Escape, or a press outside the question.
    pub fn keep_the_cloud_on(&mut self) {
        self.head.asking_off = false;
    }

    /// What a picture of the web window had open over the Inbox (`inboxMenu`, `cloudOffAsk` in
    /// its fixture).
    pub(crate) fn picture_head(&mut self) {
        match self.fixture.remove("inboxMenu").as_ref().and_then(Value::as_str) {
            Some("display") => self.toggle_head(Menu::Display),
            Some("cloud") => self.toggle_head(Menu::Cloud),
            _ => {}
        }
        if self.fixture.remove("cloudOffAsk").is_some() {
            self.head.asking_off = true;
        }
    }
}

/// The keys while a menu of the strip, or ⌘C's question, stands open. The question takes
/// Enter and Escape and keeps every other key from the page under it; a menu takes Escape and
/// lets the rest go on. `None` with nothing open.
pub fn key(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> Option<bool> {
    if s.head.asking_off {
        if !(e.mods.cmd || e.mods.ctrl || e.mods.alt) {
            match e.key {
                Key::Enter => s.turn_the_cloud_off(tx),
                Key::Escape => s.keep_the_cloud_on(),
                _ => {}
            }
        }
        return Some(true);
    }
    s.head_open()?;
    if e.key == Key::Escape && !(e.mods.cmd || e.mods.ctrl || e.mods.alt) {
        s.close_head();
        return Some(true);
    }
    Some(false)
}

/* -------------------------------- drawing -------------------------------- */

/// The accent at `alpha` (`color-mix(in srgb, var(--accent) N%, transparent)`).
fn accent_at(alpha: f32) -> Color {
    rgba(238, 96, 24, alpha)
}

/// What a fill must be laid over `base` with, for the two together to be `lit` (Rune's hover
/// is drawn over the fill, where the web window's replaces it).
fn over(base: f32, lit: f32) -> f32 {
    1.0 - (1.0 - lit) / (1.0 - base)
}

/// A line's label (`.th-pop .lab`): small capitals, faint, in the line's column of 92. Beside a
/// row of choices it stands at their top, a line of their height; beside anything else, in
/// the middle of the line.
fn lab(words: &str, beside_choices: bool) -> El {
    let label = pen(text(words).font(mono(10.5, if beside_choices { 26.0 } else { 13.5 }).tracking(0.84)).text_case(TextCase::Uppercase).color(muted()), 10.5);
    el().w(92.0).child(label)
}

/// One line of a menu (`.th-pop .line`): its label, then what it says, ten points on.
fn line(label: El, what: El, beside_choices: bool) -> El {
    let e = row().px(16.0).py(7.0).gap(10.0).child(label).child(what);
    if beside_choices { e.items_start() } else { e.items_center() }
}

/// A button's words, in the soft patch of the sky's dark every button of the strip wears, those
/// of its menus included (`text-shadow` on `.topbar :is(h1, button)`); under the pointer they
/// turn `lit`, where they do.
fn button_words(words: &str, size: f32, tone: Color, lit: Option<Color>) -> El {
    let words = words.to_string();
    crate::inbox::shaded(move |shadow| {
        let said = label(&words, size, 400.0, if shadow { crate::inbox::under(1.0) } else { tone });
        let said = match (shadow, lit) {
            (false, Some(lit)) => said.group_color(lit),
            _ => said,
        };
        pen(said, size)
    })
}

/// A rule between a menu's parts (`.th-pop .rule`).
fn rule_line() -> El {
    el().h(1.0).my(6.0).fill(edge())
}

/// How a choice looks: plain; on (`.on`), brighter in a faint edge; or picked in the cloud's
/// menu (`.on.picked`), on a wash in a strong edge that the pointer does not change.
#[derive(Clone, Copy, PartialEq)]
enum Lit {
    Off,
    On,
    Picked,
}

/// One choice of a line (`.th-pop .opts > button`): what leads it, its words, framed. Under the
/// pointer its words brighten and its edge strengthens, picked or not, but a picked one in the
/// cloud's menu stays as it is.
fn choice(lead: Option<El>, words: &str, lit: Lit) -> El {
    let tone = if lit == Lit::Off { dim() } else { ink() };
    let mut b = row().h(26.0).px(9.0).gap(7.0).items_center().radius(6.0).cursor(Cursor::Pointer);
    b = match lit {
        Lit::Off => b.border(1.0, edge()),
        Lit::On => b.border(1.0, muted()).fill(white(0.09)),
        Lit::Picked => b.border(1.0, rule()).fill(rgba(239, 235, 231, 0.1)),
    };
    let b = b.border_layout();
    let b = if lit == Lit::Picked { b } else { b.hover_group().child(feel::edge_lit(6.0, rule(), true)) };
    let b = match lead {
        Some(lead) => b.child(lead),
        None => b,
    };
    b.child(button_words(words, 12.5, tone, (lit != Lit::Picked).then(ink)))
}

/// A row of choices, wrapping onto a second line when it runs out of room (`.th-pop .opts`).
fn choices(width: f32, items: impl IntoIterator<Item = El>) -> El {
    row().w(width).flex_wrap().gap(5.0).items_center().children(items)
}

/// A menu's shell, under its button: its right edge on the button's, forty points under the
/// button's top (`.th-pop { top: 40px; right: 0 }`).
fn hung(key: &str, width: f32) -> Option<El> {
    let [x, y, w, _] = rune::memo::window_layout(key)?;
    // A press on the menu's own ground is the menu's: it does not fall through to the layer
    // that closes it.
    Some(crate::strip::shell(col()).absolute(x + w - width, y + 40.0).w(width).on_press(|_: &mut State| {}))
}

/// The View and filters menu (`DisplayMenu`), for one person: View, Sort by, then the filters
/// (Priority, Project where there is more than one, Sent) and the foot.
fn display_menu(s: &State) -> Option<El> {
    let d = s.display().clone();
    let w = says(DISPLAY_W);
    let pick = |next: Display| move |s: &mut State| s.set_display(next.clone());
    let view = {
        let list = choice(Some(icon(&icons::LIST, 13.0).color(if d.view == Shape::List { ink() } else { dim() }).group_color(ink())), "List", if d.view == Shape::List { Lit::On } else { Lit::Off });
        let board = choice(Some(icon(&icons::BOARD, 13.0).color(if d.view == Shape::Board { ink() } else { dim() }).group_color(ink())), "Board", if d.view == Shape::Board { Lit::On } else { Lit::Off });
        choices(w, [
            list.key("pop-view-list").help("List (B)").on_press(pick(Display { view: Shape::List, ..d.clone() })),
            // The board is the web window's: a press says so, and the list stays.
            board.key("pop-view-board").help("Board (B)").on_press_tx(|s: &mut State, tx| s.tell(display::BOARD_NOT_HERE, tx)),
        ])
    };
    let lit = |on: bool| if on { Lit::On } else { Lit::Off };
    let mut sort = vec![
        choice(None, "Priority", lit(d.sort == Sort::Priority)).key("pop-sort-priority").on_press(pick(Display { sort: Sort::Priority, ..d.clone() })),
        choice(None, "Sent", lit(d.sort == Sort::Sent)).key("pop-sort-sent").on_press(pick(Display { sort: Sort::Sent, ..d.clone() })),
    ];
    // Done and In progress keep their own order, and say so where the choice is made.
    if let Some(note) = display::sort_note(s.inbox_tab) {
        sort.push(el().pl(4.0).child(pen(label(note, 12.0, 400.0, muted()), 12.0)));
    }
    let levels = display::LEVELS.map(|level| {
        let words = display::level_word(level);
        choice(Some(level_mark(Some(level))), words, lit(d.priorities.contains(&level))).key(format!("pop-level-{}", words.to_lowercase())).on_press(pick(d.toggled_level(level)))
    });
    let mut menu = hung(DISPLAY_BUTTON, DISPLAY_W)?
        .py(8.0)
        .role("dialog")
        .label("Inbox view and filters")
        .child(line(lab("View", true), view, true))
        .child(line(lab("Sort by", true), choices(w, sort), true))
        .child(rule_line())
        .child(line(lab("Priority", true), choices(w, levels), true));
    // The eight projects used last, the rest behind one button, and All projects first, so
    // "nothing picked means everything" is on the screen (`projectChoices`).
    let (shown, rest) = display::project_choices(&s.snap.products, &s.snap.items, &d.projects, display::PROJECTS_SHOWN);
    if shown.len() + rest.len() > 1 {
        let all = s.head.all_projects;
        let mut projects = vec![choice(None, "All projects", lit(d.projects.is_empty())).key("pop-projects-all").on_press(pick(Display { projects: Vec::new(), ..d.clone() }))];
        for p in shown.iter().chain(rest.iter().take(if all { rest.len() } else { 0 })) {
            projects.push(choice(None, &p.name, lit(d.projects.contains(&p.slug))).key(format!("pop-project-{}", p.slug)).on_press(pick(d.toggled_project(&p.slug))));
        }
        if !rest.is_empty() {
            let words = if all { "Show fewer".to_string() } else { format!("Show all {}", shown.len() + rest.len()) };
            // `.th-pop-more`: no edge, faint, four points in on its left.
            projects.push(
                row().key("pop-projects-more").h(26.0).pl(4.0).pr(9.0).items_center().hover_group().cursor(Cursor::Pointer).child(button_words(&words, 12.5, muted(), Some(ink()))).on_press(|s: &mut State| s.head.all_projects = !s.head.all_projects),
            );
        }
        menu = menu.child(line(lab("Project", true), choices(w, projects), true));
    }
    let windows = Window::ALL.map(|window| choice(None, window.label(), lit(d.updated == window)).key(format!("pop-sent-{}", window.id())).on_press(pick(Display { updated: window, ..d.clone() })));
    menu = menu.child(line(lab("Sent", true), choices(w, windows), true)).child(rule_line());
    // How many of the tab's threads show, and the way back to all of them.
    let mut foot = row().px(16.0).pt(6.0).pb(2.0).justify_between().items_center().child(pen(label(&display::showing(&s.lists, s.inbox_tab), 12.5, 400.0, muted()), 12.5));
    if d.is_filtered() {
        foot = foot.child(row().key("pop-clear").hover_group().cursor(Cursor::Pointer).child(button_words("Clear filters", 12.5, dim(), Some(ink()))).on_press(pick(d.cleared())));
    }
    Some(menu.child(foot))
}

/// A framed button at the end of a line of the cloud's menu (`.th-cloud-manage`): under the
/// pointer its words brighten and its edge goes faint.
fn manage(words: &str) -> El {
    row().h(24.0)
        .px(10.0)
        .items_center()
        .radius(6.0)
        .border(1.0, rule())
        .border_layout()
        .hover_group()
        .cursor(Cursor::Pointer)
        .child(feel::edge_lit(6.0, muted(), true))
        .child(button_words(words, 12.5, dim(), Some(ink())))
}

/// A line of the cloud's menu that says something with a button at its end (`.th-cloud-setup`).
fn setup_line(name: &str, say: &str, button: Option<El>) -> El {
    let mut what = row().w(says(CLOUD_W)).gap(12.0).items_center().child(row().grow(1.0).min_w(0.0).child(pen(label(say, 13.0, 400.0, ink()), 13.0)));
    if let Some(button) = button {
        what = what.child(button);
    }
    line(lab(name, false), what, false)
}

/// The line about the setup at the menu's head (`.th-cloud-banner`): on the accent's wash, in
/// its edge, with the button that answers it, or the mark of a wait while it needs nothing.
fn banner(setup: &cloud::SetupLine) -> El {
    let words = pen(text(setup.say.as_str()).font(sans(13.0, 500.0).lh(18.85)).color(ink()).wrap(), 13.0);
    let say = match setup.go {
        // The mark keeps the ten points it has wherever it stands (`.cs-busy`), and the gap is
        // the line's own.
        None => row().grow(1.0).min_w(0.0).gap(9.0).items_center().child(icon(&icons::BUSY, 12.0).mr(10.0)).child(words),
        Some(_) => row().grow(1.0).min_w(0.0).child(words),
    };
    let mut b = row()
        .mx(12.0)
        .mt(4.0)
        .mb(8.0)
        .px(14.0)
        .py(12.0)
        .gap(14.0)
        .items_center()
        .radius(6.0)
        .fill(accent_at(0.08))
        .border(1.0, accent_at(0.45))
        .border_layout()
        .child(say);
    if let Some(go) = setup.go {
        let button = row()
            .h(26.0)
            .px(12.0)
            .items_center()
            .radius(6.0)
            .fill(accent_at(0.12))
            .hover(accent_at(over(0.12, 0.20)))
            .border(1.0, accent())
            .border_layout()
            .cursor(Cursor::Pointer)
            .child(button_words(go.said(), 12.5, ink(), None));
        let button = button.key("cloud-go");
        b = b.child(match go {
            Go::TryAgain => button.on_press_tx(|s: &mut State, tx| s.sign_in_again(tx)),
            Go::Fix => button.on_press_tx(|s: &mut State, tx| s.set_up_the_cloud(tx)),
        });
    }
    b
}

/// The cloud's menu (`CloudButton`): the invitation while no account can run there, else on
/// or off, what on means, the runs the cloud could take, the accounts, the month and the foot.
fn cloud_menu(s: &State) -> Option<El> {
    let cloud = s.snap.rest.get("cloud")?;
    if cloud::nobody_ready(cloud) {
        return invitation();
    }
    let w = says(CLOUD_W);
    let on = cloud::is_on(cloud);
    let when_on = cloud::when_on(cloud);
    let mut menu = hung(CLOUD_BUTTON, CLOUD_W)?.py(8.0).role("dialog").label("Where threads run");
    if let Some(setup) = cloud::setup_line(cloud) {
        menu = menu.child(banner(&setup));
    }
    // On and Off, the one picked brighter and On with its dot; the key that does the same.
    let on_button = choice(on.then(|| el().size(6.0, 6.0).radius(3.0).fill(accent()).mr(7.0)), "On", if on { Lit::Picked } else { Lit::Off }).key("cloud-on").on_press(move |s: &mut State| s.pick_cloud(cloud::set_mode(when_on)));
    let off_button = choice(None, "Off", if on { Lit::Off } else { Lit::Picked }).key("cloud-off").on_press(|s: &mut State| s.pick_cloud(cloud::set_mode(Mode::Mac)));
    let key = row()
        .h(20.0)
        .px(5.0)
        .items_center()
        .radius(6.0)
        .border(1.0, edge())
        .border_layout()
        .child(text("⌘C").font(mono(11.0, 18.0)).color(dim()));
    let keysay = row().gap(8.0).items_center().child(pen(label("turn on or off with", 12.0, 400.0, muted()), 12.0)).child(key);
    let onoff = row().w(w).justify_between().items_center().gap(12.0).child(row().gap(5.0).child(on_button).child(off_button)).child(keysay);
    menu = menu.child(line(lab("Cloud", false), onoff, false));
    let meanings = cloud::WHEN_ON.map(|(mode, words, _)| choice(None, words, if mode == when_on { Lit::Picked } else { Lit::Off }).key(format!("cloud-when-{}", mode.id())).on_press(move |s: &mut State| s.pick_cloud(cloud::set_when_on(mode))));
    menu = menu.child(line(lab("When on", true), choices(w, meanings), true));
    let note = if on { cloud::WHEN_ON.iter().find(|(mode, ..)| *mode == when_on).map_or("", |(_, _, note)| note) } else { "Off: every thread runs on this Mac." };
    menu = menu.child(row().px(16.0).pb(7.0).gap(10.0).child(el().w(92.0)).child(pen(text(note).font(sans(12.5, 400.0).lh(18.75)).color(muted()).wrap().w(w), 12.5)));
    // The runs on this Mac the cloud could take now.
    if let Some(moving) = cloud::move_line(cloud::mode(cloud), cloud::movable(cloud).len()) {
        let button = moving.go.map(|go| manage(go).key("cloud-move").on_press(|s: &mut State| s.move_to_the_cloud()));
        menu = menu.child(setup_line("This Mac", &moving.say, button));
    }
    menu = menu
        .child(rule_line())
        .child(setup_line("Accounts", &cloud::accounts_line(cloud), Some(manage("Manage").key("cloud-manage").on_press_tx(|s: &mut State, tx| s.set_up_the_cloud(tx)))))
        .child(line(lab("This month", false), pen(label(&cloud::month_line(cloud, s.cloud_plan()), 13.0, 400.0, ink()), 13.0), false))
        .child(rule_line())
        .child(row().px(16.0).pt(6.0).pb(2.0).child(pen(label(&cloud::foot_line(cloud, s.now), 12.5, 400.0, muted()), 12.5)));
    Some(menu)
}

/// The menu while no account can run in the cloud: one invitation, with one button
/// (`.th-cloud-invite`).
fn invitation() -> Option<El> {
    let [head, line, go] = cloud::INVITE;
    let mark = row().size(44.0, 44.0).center().radius(12.0).fill(accent_at(0.12)).child(icon(&icons::CLOUD, 22.0).color(accent()));
    let button = row()
        .mt(18.0)
        .w_full()
        .h(34.0)
        .center()
        .radius(6.0)
        .fill(accent_at(0.16))
        .hover(accent_at(over(0.16, 0.24)))
        .border(1.0, accent())
        .border_layout()
        .cursor(Cursor::Pointer)
        .child(button_words(go, 13.5, ink(), None))
        .key("cloud-invite-go")
        .on_press_tx(|s: &mut State, tx| s.set_up_the_cloud(tx));
    Some(
        hung(CLOUD_BUTTON, INVITE_W)?
            .pt(26.0)
            .px(26.0)
            .pb(22.0)
            .items_center()
            .role("dialog")
            .label(head)
            .child(mark)
            .child(el().mt(14.0).child(pen(text(head).font(sans(16.0, 400.0).tracking(-0.16)).color(ink()), 16.0)))
            // Thirty of the type's figures wide (`max-width: 30ch`), which the line fills.
            .child(el().mt(6.0).child(pen(text(line).font(sans(13.0, 400.0).lh(19.5)).color(dim()).align(Align::Center).wrap().w(258.56), 13.0)))
            .child(button),
    )
}

/// What stands open over the Inbox's strip, with the layer around the strip's buttons that
/// closes it on a press's way down: a press on the buttons goes on to them.
pub fn layer(s: &State) -> Option<El> {
    let open = match s.head_open()? {
        Menu::Display => display_menu(s)?,
        Menu::Cloud => cloud_menu(s)?,
    };
    let (ww, wh) = s.size;
    let [gx, gy, gw, gh] = rune::memo::window_layout(BUTTONS).unwrap_or([0.0, 0.0, 0.0, 0.0]);
    let (bottom, right) = (gy + gh, gx + gw);
    let away = [[0.0, 0.0, ww, gy], [0.0, bottom, ww, wh - bottom], [0.0, gy, gx, gh], [right, gy, ww - right, gh]].map(|[ax, ay, aw, ah]| {
        el().absolute(ax, ay).size(aw.max(0.0), ah.max(0.0)).on_drag(|s: &mut State, d: &mut Drag, _| {
            if d.phase == Phase::Down {
                s.close_head();
            }
        })
    });
    Some(el().absolute(0.0, 0.0).size(ww, wh).children(away).child(open))
}

/// ⌘C's question before turning the cloud off (`CloudOffAsk`): over the whole window, where the
/// command bar stands, with its two answers and their keys.
pub fn off_ask(s: &State) -> Option<El> {
    if !s.head.asking_off {
        return None;
    }
    let [head, said, keep, off] = cloud::OFF_ASK;
    let (w, h) = s.size;
    // `.ts-done` as the question sizes it: twenty-eight points tall, twelve in on the left and
    // eight on the right, its key after its words; the strip's look under the pointer.
    let answer = |words: &str, cap: &str, strong: bool| {
        let tone = if strong { ink() } else { dim() };
        feel::strip_button(row().h(28.0).pl(12.0).pr(8.0).gap(8.0).items_center().radius(6.0).border(1.0, if strong { rule() } else { edge() }).border_layout(), true)
            .child(pen(label(words, 13.0, 400.0, tone).group_color(ink()), 13.0))
            .child(text(cap).font(mono(13.0, 15.0)).color(tone).group_color(ink()))
    };
    let tall = 127.75;
    let card = col()
        .absolute((w - 400.0) / 2.0, (0.75 * h - tall) / 2.0)
        .w(400.0)
        .pt(18.0)
        .px(18.0)
        .pb(16.0)
        .radius(10.0)
        .fill(rgb(38, 36, 34))
        .border(1.0, white(0.08))
        .border_layout()
        .box_shadow(shadow(0.0, 24.0, 70.0, 0.0, rgba(0, 0, 0, 0.55), false))
        .modal_tx(true, |s: &mut State, _| s.keep_the_cloud_on())
        .role("alertdialog")
        .label(head)
        .child(pen(label(head, 15.0, 400.0, ink()), 15.0))
        .child(el().mt(6.0).child(pen(text(said).font(sans(13.5, 400.0).lh(20.25)).color(dim()), 13.5)))
        .child(
            row().mt(18.0)
                .justify_end()
                .gap(8.0)
                .child(answer(keep, "esc", false).key("off-keep").on_press(|s: &mut State| s.keep_the_cloud_on()))
                .child(answer(off, "↵", true).key("off-go").on_press_tx(|s: &mut State, tx| s.turn_the_cloud_off(tx))),
        );
    Some(el().absolute(0.0, 0.0).size(w, h).fill(rgba(0, 0, 0, 0.72)).child(card))
}

/// The tab the filters emptied (`FilteredEmpty`): a small tick, what is true either way, how
/// many threads the filters hold back, and the button that clears them. In the middle of what
/// the tabs leave of the page, a little above it.
pub fn filtered_empty(s: &State, tab: Tab, hidden: usize) -> El {
    let (_, page_h) = s.page_size();
    let (head, said) = display::filtered_empty_words(tab, hidden);
    let clear = s.display().cleared();
    let button = crate::inbox::strip_button(s, "inbox-clear-filters")
        .key("inbox-clear-filters")
        .px(13.0)
        .child(pen(label("Clear filters", 13.5, 400.0, ink()), 13.5))
        .on_press(move |s: &mut State| s.set_display(clear.clone()));
    col().min_h((page_h - 52.0).max(0.0))
        .pt(40.0)
        .px(24.0)
        .pb(0.12 * s.size.1)
        .items_center()
        .justify_center()
        .child(icon(&icons::CLEAR_MARK, 22.0).color(accent()).opacity(0.55).mb(14.0))
        .child(pen(text(head).font(sans(18.0, 500.0).tracking(-0.18)).color(ink()), 18.0).mb(8.0))
        .child(pen(text(said).font(sans(14.0, 400.0).lh(21.0)).color(muted()).align(Align::Center), 14.0).mb(22.0))
        .child(button)
}

#[cfg(test)]
mod tests {
    use rune::{Cursor, Key, KeyEvent, Mods, Runtime};
    use serde_json::{Value, json};

    use super::Menu;
    use crate::State;
    use crate::data::{Priority, Tab};
    use crate::rules::display::{self, Display};

    const NOW: f64 = 1_790_000_000_000.0;
    const MIN: f64 = 60_000.0;

    /// The Inbox on an invented inbox of six finished threads, three a project and each at its
    /// own level, with a cloud set up and on, and `display` as the page's display.
    fn inbox(name: &str, cloud: Value, display: Option<Value>) -> Runtime<State> {
        let threads = [("storefront", 9), ("storefront", 5), ("storefront", 2), ("mobile", 9), ("mobile", 7), ("mobile", 5)];
        let items: Vec<Value> = threads
            .iter()
            .enumerate()
            .map(|(i, (product, priority))| {
                // The later the thread, the more lately the person wrote it, so the order by when
                // each was sent runs the other way from the order by priority.
                let (asked, answered) = (NOW - (60.0 + 30.0 * (5 - i) as f64) * MIN, NOW - (4.0 + i as f64) * MIN);
                json!({
                    "id": format!("w-head{i}"), "product": product, "productName": product.to_uppercase(),
                    "status": "done", "kind": "directive", "title": format!("Invented thread number {i}"), "body": "Look into it.",
                    "result": "Done, and tested.", "labels": ["founder"], "priority": priority, "createdAt": asked, "updatedAt": answered,
                    "wrote": { "title": { "ts": asked, "source": "founder" }, "body": { "ts": asked, "source": "founder" },
                        "result": { "ts": answered, "source": "agent" }, "status": { "ts": answered, "source": "agent" } },
                })
            })
            .collect();
        let mut fixture = json!({
            "now": NOW,
            "snapshot": {
                "products": [{ "slug": "storefront", "name": "Storefront" }, { "slug": "mobile", "name": "Mobile" }],
                "items": items, "approvals": [],
                "supervisor": { "running": [], "stalled": [], "queued": [], "productOrder": ["storefront", "mobile"], "capacity": 0 },
                "cloud": cloud,
            },
            "local": {},
        });
        if let Some(display) = display {
            fixture["display"] = display;
        }
        let path = std::env::temp_dir().join(format!("mercury-head-{name}-{}.json", std::process::id()));
        std::fs::write(&path, fixture.to_string()).expect("the invented inbox");
        let s = crate::init_from(&["--fixture".to_string(), path.to_string_lossy().into_owned(), "--view".to_string(), "inbox".to_string()]);
        let _ = std::fs::remove_file(&path);
        let text = rune::text::TextSystem::new(crate::FONTS, ("Geist", "Geist Mono, ui-monospace"));
        let mut rt = Runtime::with_text(s, crate::view, (1280.0, 760.0), rune::Env::new(true, 2.0), text);
        rt.poll = Some(crate::poll);
        rt.on_key = Some(crate::keys::pressed);
        rt.frame(0.0);
        rt.frame(1.0 / 60.0);
        rt
    }

    fn cloud_on() -> Value {
        json!({ "setUp": true, "mode": "full", "whenOn": "full", "running": 0, "month": { "runs": 3, "agentHours": 1.24 },
            "accounts": [{ "engine": "claude", "key": "a-1", "ready": true, "state": "ready", "inUse": true }], "signIns": [] })
    }

    /// Presses the element laid out under `key`, as a hand does, and lets a frame go by.
    fn press(rt: &mut Runtime<State>, key: &str) {
        let (x, y) = rt.find(key).unwrap_or_else(|| panic!("nothing is laid out under {key}"));
        press_at(rt, x, y);
    }

    fn press_at(rt: &mut Runtime<State>, x: f32, y: f32) {
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
        rt.frame(1.0 / 60.0);
    }

    fn key(rt: &mut Runtime<State>, key: Key, mods: Mods) {
        rt.key(KeyEvent { key, mods });
        rt.frame(1.0 / 60.0);
    }

    fn ids(rt: &Runtime<State>) -> Vec<String> {
        rt.state.lists.needs.iter().map(|r| r.id.clone()).collect()
    }

    fn command() -> Mods {
        Mods { cmd: true, ..Mods::default() }
    }

    #[test]
    fn view_and_filters_opens_under_its_button_and_a_pick_thins_the_list_at_once() {
        let mut rt = inbox("filters", cloud_on(), None);
        assert_eq!(ids(&rt).len(), 6);
        press(&mut rt, super::DISPLAY_BUTTON);
        assert_eq!(rt.state.head_open(), Some(Menu::Display));
        // The menu hangs from the button: its right edge on the button's (1230), 40 points under its top.
        let (x, _) = rt.find("pop-sort-priority").expect("the menu is open");
        assert!((x - (760.0 + 1.0 + 16.0 + 92.0 + 10.0 + 30.4)).abs() < 1.0, "Priority stands at {x}");
        // Urgent: the list keeps the two urgent threads, the tab counts them, the menu stays open.
        press(&mut rt, "pop-level-urgent");
        assert_eq!(ids(&rt), ["w-head0", "w-head3"]);
        assert_eq!((rt.state.lists.count(Tab::Needs), rt.state.lists.whole(Tab::Needs)), (2, 6));
        assert_eq!(rt.state.head_open(), Some(Menu::Display));
        assert_eq!(display::showing(&rt.state.lists, Tab::Needs), "Showing 2 of 6");
        // And one project of the two: one thread left.
        press(&mut rt, "pop-project-mobile");
        assert_eq!(ids(&rt), ["w-head3"]);
        // Sort by Sent puts the newest sent first; Priority puts the project order first, then
        // the level each thread shows.
        press(&mut rt, "pop-level-urgent");
        press(&mut rt, "pop-projects-all");
        press(&mut rt, "pop-sort-sent");
        assert_eq!(ids(&rt), ["w-head5", "w-head4", "w-head3", "w-head2", "w-head1", "w-head0"]);
        press(&mut rt, "pop-sort-priority");
        assert_eq!(ids(&rt), ["w-head0", "w-head1", "w-head2", "w-head3", "w-head4", "w-head5"]);
        // This week keeps all six, which were written in the last hours; the filter's dot is on.
        press(&mut rt, "pop-sent-week");
        assert_eq!((ids(&rt).len(), rt.state.display().is_filtered()), (6, true));
        // Clear filters takes every filter off and keeps the sort.
        press(&mut rt, "pop-clear");
        assert_eq!(rt.state.display(), &Display::default());
        // The board is the web window's: a press says so, and the list stays a list.
        press(&mut rt, "pop-view-board");
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some(display::BOARD_NOT_HERE));
        assert_eq!(rt.state.display().view, display::View::List);
    }

    #[test]
    fn escape_closes_the_menu_and_every_other_key_goes_on_to_the_page_under_it() {
        let mut rt = inbox("keys", cloud_on(), None);
        press(&mut rt, super::DISPLAY_BUTTON);
        // J walks the list and Tab turns the tab, with the menu still open over them.
        key(&mut rt, Key::Text("j".into()), Mods::default());
        assert_eq!((rt.state.picked, rt.state.head_open()), (1, Some(Menu::Display)));
        key(&mut rt, Key::Tab, Mods::default());
        assert_eq!((rt.state.inbox_tab, rt.state.head_open()), (Tab::Progress, Some(Menu::Display)));
        assert_eq!(display::showing(&rt.state.lists, Tab::Progress), "Showing 0 of 0");
        key(&mut rt, Key::Escape, Mods::default());
        assert_eq!(rt.state.head_open(), None);
        assert_eq!(rt.state.view, crate::View::Inbox, "Escape closed the menu and went nowhere");
        // B is the board's key, which is the web window's alone.
        key(&mut rt, Key::Text("b".into()), Mods::default());
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some(display::BOARD_NOT_HERE));
    }

    #[test]
    fn a_press_outside_closes_the_menu_and_the_other_button_opens_its_own() {
        let mut rt = inbox("outside", cloud_on(), None);
        press(&mut rt, super::DISPLAY_BUTTON);
        // The cloud's button, beside it: its menu opens in place of this one.
        press(&mut rt, super::CLOUD_BUTTON);
        assert_eq!(rt.state.head_open(), Some(Menu::Cloud));
        // A press on the button of the menu that is open closes it.
        press(&mut rt, super::CLOUD_BUTTON);
        assert_eq!(rt.state.head_open(), None);
        // A press on a row while a menu is open closes the menu and opens nothing.
        press(&mut rt, super::DISPLAY_BUTTON);
        press_at(&mut rt, 300.0, 221.0);
        assert_eq!((rt.state.head_open(), &rt.state.view), (None, &crate::View::Inbox));
        // A press on the menu's own ground leaves it open, and the pointer is the arrow there.
        press(&mut rt, super::DISPLAY_BUTTON);
        rt.pointer_move(1225.0, 300.0);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.cursor(), Cursor::Default);
        press_at(&mut rt, 1225.0, 300.0);
        assert_eq!(rt.state.head_open(), Some(Menu::Display));
        // Over a choice, the pointing hand.
        let (x, y) = rt.find("pop-level-high").unwrap();
        rt.pointer_move(x, y);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.cursor(), Cursor::Pointer);
        // Leaving the Inbox closes it, and coming back finds it closed.
        key(&mut rt, Key::Enter, Mods::default());
        assert!(matches!(rt.state.view, crate::View::Thread { .. }));
        key(&mut rt, Key::Escape, Mods::default());
        assert_eq!((rt.state.view.clone(), rt.state.head_open()), (crate::View::Inbox, None));
    }

    #[test]
    fn a_tab_the_filters_emptied_says_so_and_its_button_clears_them() {
        let mut rt = inbox("emptied", cloud_on(), Some(json!({ "view": "list", "sort": "priority", "priorities": ["low"], "projects": ["mobile"], "updated": "any" })));
        assert!(rt.state.lists.needs.is_empty());
        assert_eq!(rt.state.lists.whole(Tab::Needs), 6);
        press(&mut rt, "inbox-clear-filters");
        assert_eq!(ids(&rt).len(), 6);
        assert!(!rt.state.display().is_filtered());
    }

    #[test]
    fn command_c_turns_the_cloud_on_with_a_line_and_asks_before_it_goes_off() {
        let mut rt = inbox("cloud-key", cloud_on(), None);
        key(&mut rt, Key::Command('c'), command());
        assert!(rt.state.head.asking_off, "off asks first");
        // Escape keeps it on; nothing else happened.
        key(&mut rt, Key::Escape, Mods::default());
        assert!(!rt.state.head.asking_off);
        assert!(rt.state.toast.is_none());
        // Asked again, and answered with Enter: the line says where new threads run.
        key(&mut rt, Key::Command('c'), command());
        key(&mut rt, Key::Text("j".into()), Mods::default());
        assert_eq!(rt.state.picked, 0, "the question keeps every key from the page");
        key(&mut rt, Key::Enter, Mods::default());
        assert!(!rt.state.head.asking_off);
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Cloud off: new threads run on this Mac"));
        // With the cloud off it turns on at once, to what on means.
        let mut off = cloud_on();
        off["mode"] = json!("mac");
        off["whenOn"] = json!("all");
        let mut rt = inbox("cloud-key-off", off, None);
        key(&mut rt, Key::Command('c'), command());
        assert!(!rt.state.head.asking_off);
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Cloud on: everything starts in the cloud"));
        // Not while the command bar has the keys, nor on a Mac with no cloud set up.
        let mut rt = inbox("cloud-key-bar", cloud_on(), None);
        key(&mut rt, Key::Command('k'), command());
        key(&mut rt, Key::Command('c'), command());
        assert!(!rt.state.head.asking_off);
        let mut rt = inbox("cloud-key-none", json!({ "setUp": false, "offered": true, "mode": "mac", "running": 0, "accounts": [], "signIns": [] }), None);
        key(&mut rt, Key::Command('c'), command());
        assert!(!rt.state.head.asking_off && rt.state.toast.is_none());
    }

    #[test]
    fn the_questions_buttons_answer_as_its_keys_do() {
        let mut rt = inbox("cloud-ask", cloud_on(), None);
        key(&mut rt, Key::Command('c'), command());
        press(&mut rt, "off-keep");
        assert!(!rt.state.head.asking_off && rt.state.toast.is_none());
        key(&mut rt, Key::Command('c'), command());
        press(&mut rt, "off-go");
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Cloud off: new threads run on this Mac"));
    }

    #[test]
    fn a_stopped_account_stops_asking_once_the_menu_was_opened_on_it_and_closed() {
        let mut cloud = cloud_on();
        cloud["accounts"].as_array_mut().unwrap().push(json!({ "engine": "codex", "key": "a-2", "ready": false, "state": "broken" }));
        let mut rt = inbox("broken", cloud, None);
        let asks = |rt: &Runtime<State>| crate::rules::cloud::button(rt.state.snap.rest.get("cloud"), &rt.state.broken_seen()).and_then(|b| b.word);
        assert_eq!(asks(&rt), Some(crate::rules::cloud::Word::Again));
        press(&mut rt, super::CLOUD_BUTTON);
        assert!(rt.find("cloud-go").is_some(), "the menu says so, with Fix");
        assert_eq!(asks(&rt), Some(crate::rules::cloud::Word::Again), "not under the open menu");
        key(&mut rt, Key::Escape, Mods::default());
        assert_eq!(asks(&rt), None);
        assert_eq!(rt.state.broken_seen(), ["codex:a-2"]);
    }

    #[test]
    fn the_cloud_menu_says_what_runs_where_and_its_buttons_do_what_they_say() {
        let mut cloud = cloud_on();
        cloud["movable"] = json!(["w-head1", "w-head2"]);
        let mut rt = inbox("cloud-menu", cloud, None);
        press(&mut rt, super::CLOUD_BUTTON);
        for there in ["cloud-on", "cloud-off", "cloud-when-full", "cloud-when-all", "cloud-move", "cloud-manage"] {
            assert!(rt.find(there).is_some(), "{there} is drawn");
        }
        // Move them closes the menu. With no app behind the window nothing is sent.
        press(&mut rt, "cloud-move");
        assert_eq!(rt.state.head_open(), None);
        // Manage is the web window's own screen; with no app to bring it forward, a line says so.
        press(&mut rt, super::CLOUD_BUTTON);
        press(&mut rt, "cloud-manage");
        assert_eq!(rt.state.head_open(), None);
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some(super::SETUP_NOT_HERE));
        // A cloud never set up opens on the invitation.
        let mut rt = inbox("cloud-invite", json!({ "setUp": false, "offered": true, "mode": "mac", "running": 0, "accounts": [], "signIns": [] }), None);
        press(&mut rt, super::CLOUD_BUTTON);
        assert!(rt.find("cloud-invite-go").is_some() && rt.find("cloud-on").is_none());
    }

    #[test]
    fn home_counts_what_the_filters_keep_and_lists_in_progress_whole() {
        let mut rt = inbox("home", cloud_on(), Some(json!({ "priorities": ["high"] })));
        assert_eq!(ids(&rt), ["w-head4"]);
        assert_eq!(rt.state.lists.needs.iter().map(|r| r.priority).collect::<Vec<_>>(), [Priority::High]);
        // The rail's count is the Inbox's, of what the filters keep.
        rt.state.go(crate::View::Home);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.lists.needs.len(), 1);
    }
}
