//! The box a thread is started in: open on Home's page, under the greeting. Built from the web
//! app's own measurements of its New thread card (`renderer/src/threads/ThreadComposer.tsx`,
//! `thread-composer.css` and Home's `home.css`); what it sends, to which project and on which
//! model is `rules/compose.rs`.
//!
//! The words are the person's to type and ⌘Return or Send sends them. Nothing waits three
//! seconds here, unlike a reply or a close: a thread held back would be in no list at all
//! while the person looks for it. It is written at once, a line says where it is going and
//! then where it went, and Z takes it back by closing it, which stops its agent.
//!
//! The chips under the words say what the thread will be sent with, and each opens a menu to
//! change it: the model with its agent and how hard it thinks, the project, the level. One
//! menu is open at a time; the pointer and the arrows walk its rows, Enter picks, and Escape
//! closes the menu before anything else.

use rune::edit::Key;
use rune::time::LocalTime;
use rune::{El, KeyEvent, ScrollSpec, TextCase, TextEdit, TextStyle, Tx, col, el, icon, row, spacer, text, text_input};
use serde_json::Value;

use crate::data::{Item, Product};
use crate::rules::compose::{self as rules, Engines, Entry, Hang, Keyboard, Level, Pick};
use crate::rules::{title, window};
use crate::theme::*;
use crate::{State, View, chrome, home, icons, write};

/// The key of the field the words are typed in.
pub const FIELD: &str = "compose";
/// And of the open menu's rows, whose width decides whether it has to slide.
const MENU: &str = "compose-menu";

/// One of the box's menus, named by the chip that opens it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuKind {
    Model,
    Project,
    Level,
}

impl MenuKind {
    /// The key of the chip the menu hangs from.
    fn chip(self) -> &'static str {
        match self {
            MenuKind::Model => "chip-model",
            MenuKind::Project => "chip-project",
            MenuKind::Level => "chip-level",
        }
    }
}

/// The menu that is open.
#[derive(Clone, Debug, PartialEq)]
pub struct Menu {
    pub kind: MenuKind,
    /// The model menu turned to its second page: every model, a column for each agent.
    pub every_model: bool,
    /// The entry the keys are on, which the pointer moves too.
    pub lit: usize,
}

/// What the box holds between one thread and the next.
pub struct Composer {
    pub words: TextEdit,
    /// The level picked on this box; none is Agent Decides.
    pub level: Option<Level>,
    /// The project picked on this box, by its slug; none is the one the rules open it on.
    pub project: Option<String>,
    /// The model picked on this box; none is the one the rules open it on.
    pub pick: Option<Pick>,
    pub menu: Option<Menu>,
    /// The chip the keyboard was left on when Escape closed its menu: an arrow opens the menu
    /// again, a letter goes to the words, and a second Escape lets go of the box.
    pub on_chip: Option<MenuKind>,
    /// When the window says that a thread on its way has not been taken yet, on its clock.
    not_taken_at: Option<f64>,
    /// O pressed while a thread was on its way, and the screen it was pressed on: the thread
    /// is opened when it lands, unless the person has gone somewhere else by then.
    o_waits: Option<View>,
}

impl Default for Composer {
    fn default() -> Composer {
        Composer { words: TextEdit::new(""), level: None, project: None, pick: None, menu: None, on_chip: None, not_taken_at: None, o_waits: None }
    }
}

impl Composer {
    /// What of the box has the keyboard (`Keyboard` in `composer-rules.ts`).
    pub fn keyboard(&self) -> Keyboard {
        if self.words.focused {
            Keyboard::Field
        } else if self.on_chip.is_some() || self.menu.is_some() {
            Keyboard::Card
        } else {
            Keyboard::Nothing
        }
    }
}

/// The box's words, each standing where Chromium puts its glyphs (`theme::pen`).
fn label(s: &str, size: f32, weight: f32, c: rune::Color) -> El {
    pen(crate::theme::label(s, size, weight, c), size)
}

/// The mark beside a level's word (`PriorityIcon` in `renderer/src/components/Priority.tsx`):
/// three bars lit from the left for Low, Medium and High, a fourth for Urgent, and all three
/// unlit while the agent decides.
pub fn level_mark(level: Option<Level>) -> El {
    let off = rule();
    let three = |a, b, c| row().items_end().gap(1.5).child(el().size(3.0, 4.0).radius(1.0).fill(a)).child(el().size(3.0, 7.5).radius(1.0).fill(b)).child(el().size(3.0, 11.0).radius(1.0).fill(c));
    match level {
        None => three(off, off, off),
        Some(Level::Urgent) => bars(&[4.0, 7.0, 10.0, 13.0], ink()),
        Some(Level::High) => three(ink(), ink(), ink()),
        Some(Level::Medium) => three(dim(), dim(), off),
        Some(Level::Low) => three(dim(), off, off),
    }
}

/// A project's own colour, in the square the chip and the menu draw it in (`.tc-swatch`).
pub fn swatch(slug: &str) -> El {
    let (r, g, b) = window::swatch(slug);
    row().size(17.0, 17.0).center().child(el().size(9.0, 9.0).radius(4.5).fill(rgb(r, g, b)))
}

/// One of the chips that opens a menu (`.tc-chip`, and `.tc-chip.open` while its menu is up:
/// its words brighter inside a stronger edge). `inside` is handed the colour of its words.
/// A press never takes the keyboard to the chip itself: the box keeps track of where it is.
fn menu_chip(s: &State, kind: MenuKind, inside: impl FnOnce(rune::Color) -> Vec<El>) -> El {
    let open = s.compose.menu.as_ref().is_some_and(|m| m.kind == kind);
    let chip = home::chip(inside(if open { ink() } else { dim() })).key(kind.chip()).cursor(rune::Cursor::Pointer).on_background_press_tx(move |s: &mut State, tx| s.toggle_menu(kind, tx));
    if open { chip.border(1.0, rule()) } else { chip }
}

/// The project's chip and the level's, which every bar has.
fn filed_chips(s: &State) -> [El; 2] {
    let project = s.compose_project();
    [
        menu_chip(s, MenuKind::Project, |tone| match project {
            Some(p) => vec![swatch(&p.slug), label(&p.name, 13.0, 400.0, tone)],
            None => vec![label("No project", 13.0, 400.0, tone)],
        }),
        menu_chip(s, MenuKind::Level, |tone| vec![level_mark(s.compose.level), label(rules::choice_label(s.compose.level), 13.0, 400.0, tone)]),
    ]
}

/// The paper clip, why Send is off when it is off for want of a project, and Send.
fn bar_end(s: &State) -> Vec<El> {
    let mut end = vec![spacer(), home::attach()];
    if s.compose_project().is_none() {
        // `.tc-why`. Making a project is the web window's to do, so the way out is words here
        // and not a button.
        end.push(label(&format!("{} {}", rules::NO_PROJECT_REASON, rules::NO_PROJECT_ACTION), 13.0, 400.0, dim()).pl(8.0));
    }
    // The two halves of Send are one control with no room between them (`.tc-send`).
    end.push(row().child(home::send_main(s.compose_ready()).cursor(rune::Cursor::Pointer).on_background_press_tx(|s: &mut State, tx| s.send_thread(tx))).child(home::send_caret()));
    end
}

/// The bar under the words on the page: who gets the thread where someone besides the agent
/// could, the model, the project, the level, and Send.
fn bar(s: &State) -> El {
    let mut bar = row().pt(14.0).items_center().gap(8.0);
    if window::to_offered(&s.snap) {
        bar = bar.child(home::chip(vec![label("Agent", 13.0, 400.0, dim())]));
    }
    let model = rules::model_label(&s.compose_pick(), &s.models);
    bar.child(menu_chip(s, MenuKind::Model, |tone| vec![label(&model, 13.0, 400.0, tone)])).children(filed_chips(s)).children(bar_end(s))
}

/// The field the words are typed in, in the type of the place it is drawn in.
fn words_field(s: &State, font: TextStyle, caret: rune::Color) -> El {
    text_input(&s.compose.words, font)
        .key(FIELD)
        .color(ink())
        .placeholder("What do you need done?", muted())
        .caret(caret, rgba(110, 165, 255, 0.3))
        .on_edit(|s: &mut State| &mut s.compose.words)
        // Return is a new line, as in the web window's box; ⌘Return sends.
        .on_submit(|s: &mut State, _| s.compose.words.insert("\n"))
        .on_input_key(crate::keys::in_the_box)
}

/// The box on Home's page (`.hm-start .tc-card.tc-inline`): the words large over the bar, on
/// the page's glass. Its edge brightens while the caret is in it.
pub fn inline(s: &State) -> El {
    // `caret-color: var(--accent)` on the box drawn on a page.
    let field = words_field(s, TextStyle::new(22.0, 400.0).lh(30.0), accent());
    let mut card = glass(col())
        .pt(16.0)
        .px(18.0)
        .pb(14.0)
        .radius(10.0)
        // `:focus-within`: the caret in the words, or the keyboard on a chip or in its menu.
        .border(1.0, white(if s.compose.keyboard() == Keyboard::Nothing { 0.2 } else { 0.3 }))
        .border_layout()
        .box_shadow(shadow(0.0, 0.0, 0.0, 3.0, white(0.03), false))
        .box_shadow(shadow(0.0, 12.0, 40.0, 0.0, rgba(0, 0, 0, 0.3), false))
        // One line tall until the words need more: 74 points with the room under them.
        .child(el().min_h(74.0).pb(14.0).child(field.w_full()));
    if let Some(why) = s.compose_project().and_then(|p| rules::practice_refusal(Some(p))) {
        card = card.child(el().pt(12.0).child(label(why, 13.0, 400.0, dim())));
    }
    card.child(bar(s))
}

/// A row of a menu is 34 points tall: a 20 point line and 7 over and under it (`.tc-row`).
const ROW: f32 = 34.0;

/// What every entry of a menu does under the pointer: it takes the keys when the pointer
/// moves on it, and a press is Enter on it. On a pointer that moves, not one that is merely
/// there: a page that moves under a still pointer must not take a row back from the arrows.
fn entry(e: El, at: usize) -> El {
    e.key(format!("entry-{at}")).cursor(rune::Cursor::Pointer).on_pointer_move(move |s: &mut State, _| s.light_entry(at)).on_background_press_tx(move |s: &mut State, tx| s.press_entry(at, tx))
}

/// One row (`.tc-row`): what leads it, its name, and what stands at its right. `lit` is the
/// row the keys are on, `on` the one already picked, which a menu of the head also sets heavy.
fn menu_row(at: usize, lit: bool, on: (bool, bool), lead: Option<El>, name: &str, aside: Option<El>) -> El {
    let (on, heavy) = on;
    let mut line = entry(row(), at).h(ROW).items_center().gap(10.0).px(10.0).radius(6.0);
    if lit {
        line = line.fill(white(0.08));
    }
    if let Some(lead) = lead {
        line = line.child(lead);
    }
    let words = text(name).font(TextStyle::new(13.5, if on && heavy { 600.0 } else { 400.0 }).lh(20.0)).color(if lit || on { ink() } else { dim() });
    line = line.child(row().grow(1.0).min_w(0.0).child(pen(words, 13.5)));
    match aside {
        Some(aside) => line.child(aside),
        None => line,
    }
}

/// The faint words at a row's right, so far from its name (`.tc-row small`, `.tc-row-note`).
fn aside(words: &str, before: f32) -> El {
    el().pl(before).child(pen(text(words).font(TextStyle::new(12.0, 400.0).lh(20.0)).color(muted()), 12.0))
}

/// A menu's heading (`.tc-menu-head`).
fn menu_head(words: &str) -> El {
    el().pt(7.0).px(10.0).pb(4.0).child(text(words).font(mono(10.5, 13.5).tracking(0.84)).text_case(TextCase::Uppercase).color(muted()))
}

/// The hairline between a menu's parts (`.tc-sep`).
fn menu_sep() -> El {
    el().h(1.0).mx(6.0).my(4.0).fill(edge())
}

/// One level of the strip at the model menu's foot (`.tc-effort button`): `shown` is the level
/// the model thinks at, the person's pick or its own, and `lit` where the keys are.
fn effort_button(at: usize, lit: bool, shown: bool, word: &str) -> El {
    let mut button = entry(row(), at).h(30.0).px(8.0).items_center().radius(6.0);
    if shown {
        button = button.fill(white(0.08));
    } else if lit {
        button = button.fill(white(0.04));
    }
    button.child(pen(text(word).font(TextStyle::new(12.5, if shown { 600.0 } else { 400.0 }).lh(20.0)).color(if shown || lit { ink() } else { dim() }), 12.5))
}

/// The open menu's parts from top to bottom, how tall they add up to with nothing between
/// them, and the least width the stylesheet gives the menu.
fn menu_parts(s: &State, menu: &Menu) -> (Vec<El>, f32, f32) {
    let mut parts: Vec<El> = Vec::new();
    let mut tall = 0.0;
    let mut part = |e: El, height: f32| {
        parts.push(e);
        tall += height;
    };
    let lit = |at: usize| menu.lit == at;
    let least = match menu.kind {
        MenuKind::Project => {
            let offered = s.projects_offered();
            let on = s.compose_project().map(|p| p.slug.as_str());
            // The list is in the person's own order, and says so (`.tc-head-split`). Changing
            // the order is Settings' to do, which this window does not have: the word is drawn.
            let order = text("By priority").font(mono(10.5, 13.5).tracking(0.84)).text_case(TextCase::Uppercase).color(muted());
            let reorder = text("Reorder").font(mono(11.5, 15.0).tracking(0.23)).color(accent());
            part(row().h(26.0).pt(7.0).px(10.0).pb(4.0).justify_between().child(el().pt(1.0).child(order)).child(reorder), 26.0);
            for (at, p) in offered.iter().enumerate() {
                part(menu_row(at, lit(at), (on == Some(p.slug.as_str()), false), Some(swatch(&p.slug)), &p.name, rules::project_note(p).map(|note| aside(note, 16.0))), ROW);
            }
            if !offered.is_empty() {
                // The hairline that keeps the last row from reading as one more project.
                part(el().h(1.0).mx(6.0).my(5.0).fill(rgb(59, 57, 55)), 11.0);
            }
            let at = offered.len();
            part(menu_row(at, lit(at), (false, false), Some(icon(&icons::PLUS, 14.0).color(muted())), "New project", None), ROW);
            230.0
        }
        MenuKind::Level => {
            part(menu_head("Priority"), 24.5);
            for (at, level) in [None].into_iter().chain(rules::LEVELS.into_iter().map(Some)).enumerate() {
                part(menu_row(at, lit(at), (level == s.compose.level, false), Some(level_mark(level)), rules::choice_label(level), None), ROW);
            }
            190.0
        }
        MenuKind::Model => {
            let engines = Engines::of(&s.snap);
            let pick = s.compose_pick();
            let recent = rules::recent_models(&s.snap.items, &s.models, &engines);
            let all = rules::all_models(&s.models, &engines);
            let name = |p: &Pick| rules::model_label(p, &s.models);
            let mut at = 0;
            if menu.every_model {
                part(menu_row(at, lit(at), (false, true), None, "‹ Recent", None), ROW);
                at += 1;
                part(menu_sep(), 9.0);
                // A column for each agent that has models, under the agent's name (`.tc-cols`).
                let mut columns = row().gap(6.0);
                let mut deepest = 0;
                for (engine, picks) in all.columns() {
                    let mut column = col().grow(1.0).min_w(176.0).gap(1.0).child(menu_head(&engines.label(engine)));
                    for p in picks {
                        column = column.child(menu_row(at, lit(at), (*p == pick, true), None, &name(p), None));
                        at += 1;
                    }
                    deepest = deepest.max(picks.len());
                    columns = columns.child(column);
                }
                part(columns, 24.5 + deepest as f32 * (ROW + 1.0));
            } else {
                part(menu_head("Recent"), 24.5);
                for p in &recent {
                    // Where there are two agents, each model says which runs it.
                    part(menu_row(at, lit(at), (*p == pick, true), None, &name(p), engines.tagged().then(|| aside(&engines.label(&p.engine), 18.0))), ROW);
                    at += 1;
                }
                part(menu_sep(), 9.0);
                let more = rules::more_count(&recent, &all);
                part(menu_row(at, lit(at), (false, true), None, "All models", (more > 0).then(|| aside(&format!("{more} more ›"), 18.0))), ROW);
                at += 1;
            }
            let levels = rules::effort_choices(&pick, &s.models);
            if !levels.is_empty() {
                part(menu_sep(), 9.0);
                part(el().pt(6.0).px(10.0).pb(2.0).child(text("Effort").font(TextStyle::new(11.0, 400.0).lh(20.0).tracking(0.22)).color(muted())), 28.0);
                let shown = rules::effort_shown(s.kept_effort(&pick.engine).as_deref(), &levels, rules::default_effort(&pick, &s.models).as_deref());
                let mut strip = row().pt(3.0).px(2.0).pb(2.0).gap(2.0);
                for (id, word) in &levels {
                    strip = strip.child(effort_button(at, lit(at), shown.as_deref() == Some(id.as_str()), word));
                    at += 1;
                }
                part(strip, 35.0);
            }
            230.0
        }
    };
    (parts, tall, least)
}

/// The open menu, over the page: hung from its chip where the stylesheet hangs it, and kept
/// inside the window as the web window keeps it (`rules::fit_menu`). Under it lies a layer the
/// size of the window, and a press there closes the menu.
pub fn menu_layer(s: &State) -> Option<El> {
    let menu = s.compose.menu.as_ref()?;
    // Where its chip is in the window now. A chip not laid out yet has nothing hung from it.
    let anchor = rune::memo::window_layout(menu.kind.chip())?;
    let (parts, sum, least) = menu_parts(s, menu);
    // Its edge and the 5 points inside it, twice, and a point between each two parts.
    let tall = 12.0 + sum + parts.len().saturating_sub(1) as f32;
    let wide = rune::memo::layout_size(MENU).map_or(least, |[wide, _]| wide + 2.0);
    let (w, h) = s.size;
    // On the page every menu drops under its chip, from the chip's left edge
    // (`.tc-inline .tc-menu`), and is seen only inside the page, which clips it (`.hm-page`).
    let hang = Hang { below: true, gap: 8.0, dx: 0.0 };
    let fit = rules::fit_menu(anchor, hang, (wide, tall), h, (chrome::CARD[0], w - chrome::CARD[2]));
    let rows = col().key(MENU).min_w(least - 2.0).items_stretch().gap(1.0).px(5.0).py(5.0).children(parts);
    let shell = col().radius(10.0).clip_rounded(10.0).fill(rgb(54, 52, 49)).border(1.0, rule()).border_layout().box_shadow(shadow(0.0, 18.0, 50.0, 0.0, rgba(0, 0, 0, 0.5), false));
    let shell = match fit.max_height {
        // Too tall for the room it has: it is that tall, and its rows scroll inside it.
        Some(most) => shell.h(most).child(col().grow(1.0).min_h(0.0).scroll(ScrollSpec { stick_bottom: false }).child(rows)),
        None => shell.child(rows),
    };
    let away = el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).on_background_press(|s: &mut State| {
        s.compose.menu = None;
        s.compose.on_chip = None;
    });
    let page = el()
        .pin(Some(chrome::CARD[0]), Some(chrome::PAGE_TOP), Some(chrome::CARD[2]), Some(chrome::CARD[3]))
        .clip_corners([0.0, 0.0, 6.0, 6.0])
        .child(shell.absolute(fit.left - chrome::CARD[0], fit.top - chrome::PAGE_TOP));
    Some(el().absolute(0.0, 0.0).size(w, h).child(away).child(page))
}

/// The keys while the box is on the page, for the ones no text field took. `None` when the
/// key is not the box's, and the page's own keys go on to read it.
pub fn key(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> Option<bool> {
    if s.view != View::Home {
        return None;
    }
    let chord = e.mods.cmd || e.mods.ctrl || e.mods.alt;
    let sends = e.key == Key::Enter && e.mods.cmd;
    // An open menu has the keys: the arrows walk it around both ends, Enter presses the row
    // they are on, Escape closes it and leaves the keyboard on its chip (`menuKeys`).
    if let Some(menu) = s.compose.menu.clone() {
        if sends {
            s.send_thread(tx);
            return Some(true);
        }
        if chord {
            return Some(false);
        }
        match e.key {
            Key::Escape => s.close_menu(false, tx),
            Key::Down | Key::Up => {
                let lit = rules::entry_step(menu.lit, s.menu_entries(&menu).len(), e.key == Key::Down);
                s.light_entry(lit);
                tx.scroll_into_view(format!("entry-{lit}"));
            }
            Key::Enter => s.press_entry(menu.lit, tx),
            _ => {}
        }
        return Some(true);
    }
    let on = s.compose.keyboard();
    if sends {
        // On one of the box's controls the box itself hears ⌘Return. With nothing holding the
        // keyboard the page hears it, and sends only a box that has words in it.
        if on == Keyboard::Card || rules::mod_enter_sends(true, on, !s.compose.words.text.trim().is_empty()) {
            s.send_thread(tx);
        }
        return Some(true);
    }
    if chord {
        return None;
    }
    if let Some(kind) = s.compose.on_chip {
        match e.key {
            // An arrow on a chip opens its menu (`triggerKeys`).
            Key::Down | Key::Up => {
                s.toggle_menu(kind, tx);
                return Some(true);
            }
            // And Escape lets go of the box, keeping the words.
            Key::Escape => {
                s.compose.on_chip = None;
                return Some(true);
            }
            _ => {}
        }
    }
    let typed = match &e.key {
        Key::Text(t) if t.chars().count() == 1 => Some(t.clone()),
        Key::Enter => Some("\n".to_string()),
        _ => None,
    };
    // A letter with the keyboard on a chip is typed into the words: no control under it ever
    // hears it.
    if let Some(typed) = typed.filter(|_| rules::key_is_typed(true, chord, false, on, true)) {
        s.caret_in_the_words(tx);
        s.compose.words.insert(&typed);
        return Some(true);
    }
    None
}

/// What Z takes back once a thread sent from the box stands: a close of it, which stops its
/// worker, and the box's words handed back (`noteNewTask` in `renderer/src/App.tsx`). The line
/// is said once the engine has closed it, and says then where the words went.
fn taking_back(id: &str, product: &str, title: &str, draft: write::Draft) -> write::Undo {
    write::Undo {
        said: rules::said_withdrawn(title, false),
        writes: vec![(crate::rules::writes::ANSWER, rules::withdraw(product, id))],
        sent: write::Sent::Withdraw { thread: format!("{product}/{id}"), title: title.to_string(), draft },
        thread: (id.to_string(), product.to_string()),
        opens: false,
    }
}

/// "6:10pm", and the day after it when that is not today: when an agent out of usage has room
/// again (`backAt` in `renderer/src/compose-says.ts`, with `resetDay` in `shared/usage.mts`).
/// Nothing once the moment has passed.
fn back_at(until: f64, now: f64) -> Option<String> {
    const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    if until <= now {
        return None;
    }
    let t = LocalTime::at(until as i64);
    let day_of = |ms: f64| ((ms / 1000.0) as i64 + i64::from(LocalTime::at(ms as i64).offset)).div_euclid(86_400);
    let hour = if t.hour.is_multiple_of(12) { 12 } else { t.hour % 12 };
    let at = format!("{hour}:{:02}{}", t.minute, if t.hour < 12 { "am" } else { "pm" });
    if day_of(until) == day_of(now) {
        return Some(at);
    }
    let weekday = DAYS[(day_of(until) + 4).rem_euclid(7) as usize];
    // Six days out or more a weekday alone would read as this week's.
    if until - now < 6.0 * 86_400_000.0 { Some(format!("{at} {weekday}")) } else { Some(format!("{at} {weekday} {} {}", MONTHS[(t.month as usize).clamp(1, 12) - 1], t.day)) }
}

impl State {
    /// The projects a thread can be filed into, in the person's own order.
    pub fn projects_offered(&self) -> Vec<&Product> {
        rules::projects_offered(&rules::rank_products(&self.snap.products, &self.snap.supervisor.product_order))
    }

    /// The project the box is on: the one picked on it, else the open conversation's, else
    /// the one last sent to.
    pub fn compose_project(&self) -> Option<&Product> {
        let on_screen = self.open_item().map(|i| i.product.as_str());
        rules::card_project(&self.projects_offered(), self.compose.project.as_deref(), on_screen, self.picks.get().project.as_deref())
    }

    /// The agent and model the box is on: the one picked on it while this Mac still runs it,
    /// else the agent from Settings on the model last picked for it.
    pub fn compose_pick(&self) -> Pick {
        let engines = Engines::of(&self.snap);
        let kept = |engine: &str| self.picks.get().model.get(engine).cloned();
        match &self.compose.pick {
            Some(pick) => rules::pick_this_mac_runs(pick.clone(), &engines, &self.models, kept),
            None => rules::starting_pick(&engines, &self.models, kept),
        }
    }

    /// Whether Send is lit: there is something to send, and somewhere it can run (`canSend`).
    pub fn compose_ready(&self) -> bool {
        rules::thread_message(&self.compose.words.text).is_some() && self.compose_project().is_some_and(|p| rules::practice_refusal(Some(p)).is_none())
    }

    /// The level last picked for an agent, when it is one worth keeping.
    fn kept_effort(&self, engine: &str) -> Option<String> {
        rules::remembered_effort(self.picks.get().effort.get(engine).map(String::as_str), engine)
    }

    /// Puts the caret back in the words where it was: after a pick, and for a letter typed
    /// with the keyboard on a chip.
    fn caret_in_the_words(&mut self, tx: &mut Tx) {
        self.compose.on_chip = None;
        self.compose.words.focused = true;
        tx.focus_when_shown(FIELD, false);
    }

    /// Puts the caret in the box, at the end of what it holds (N on Home, or a letter typed
    /// with nothing to take it).
    pub fn write_in_the_box(&mut self, tx: &mut Tx) {
        let end = self.compose.words.text.len();
        self.compose.words.set_caret(end);
        self.caret_in_the_words(tx);
    }

    /// Escape in the box: the keys go back to the page, and the words stay.
    pub fn leave_the_box(&mut self, tx: &mut Tx) {
        self.compose.words.focused = false;
        self.compose.on_chip = None;
        tx.blur();
    }

    /// What the keys can land on in a menu, in the order it draws them.
    fn menu_entries(&self, menu: &Menu) -> Vec<Entry> {
        match menu.kind {
            MenuKind::Project => rules::project_entries(&self.projects_offered()),
            MenuKind::Level => rules::level_entries(),
            MenuKind::Model => {
                let engines = Engines::of(&self.snap);
                let levels = rules::effort_choices(&self.compose_pick(), &self.models);
                rules::model_entries(menu.every_model, &rules::recent_models(&self.snap.items, &self.models, &engines), &rules::all_models(&self.models, &engines), &levels)
            }
        }
    }

    /// Whether an entry is what the box is already on.
    fn entry_is_picked(&self, entry: &Entry) -> bool {
        match entry {
            Entry::Project(slug) => self.compose_project().is_some_and(|p| &p.slug == slug),
            Entry::Level(level) => *level == self.compose.level,
            Entry::Model(pick) => *pick == self.compose_pick(),
            _ => false,
        }
    }

    /// A press on a chip: its menu opens with the keys on what is already picked, or closes
    /// if it was the one open (`toggle`). The caret leaves the words while a menu is up.
    pub fn toggle_menu(&mut self, kind: MenuKind, tx: &mut Tx) {
        self.compose.on_chip = Some(kind);
        if self.compose.menu.take().is_some_and(|open| open.kind == kind) {
            return;
        }
        self.compose.words.focused = false;
        tx.blur();
        let mut menu = Menu { kind, every_model: false, lit: 0 };
        menu.lit = rules::entry_lit_first(&self.menu_entries(&menu), |entry| self.entry_is_picked(entry));
        self.compose.menu = Some(menu);
        // The lists may have changed since the window opened: an agent's can land late.
        if kind == MenuKind::Model {
            self.read_the_models();
        }
    }

    /// Closes the open menu. A pick puts the caret back in the words, where the person was
    /// going anyway; Escape leaves the keyboard on the chip that opened it (`close`).
    pub fn close_menu(&mut self, to_the_words: bool, tx: &mut Tx) {
        let Some(menu) = self.compose.menu.take() else {
            return;
        };
        if to_the_words {
            self.caret_in_the_words(tx);
        } else {
            self.compose.on_chip = Some(menu.kind);
        }
    }

    /// The pointer moved onto an entry, or the arrows did.
    pub fn light_entry(&mut self, at: usize) {
        if let Some(menu) = &mut self.compose.menu {
            menu.lit = at;
        }
    }

    /// Enter on an entry of the open menu, or a press on it.
    pub fn press_entry(&mut self, at: usize, tx: &mut Tx) {
        let Some(mut menu) = self.compose.menu.clone() else {
            return;
        };
        let Some(entry) = self.menu_entries(&menu).into_iter().nth(at) else {
            return;
        };
        match entry {
            // Remembered the moment it is picked, as the web window does (`zero.lastProduct`).
            Entry::Project(slug) => {
                self.picks.change(|picks| picks.project = Some(slug.clone()));
                self.compose.project = Some(slug);
                self.close_menu(true, tx);
            }
            // The card that makes a project is the web window's, and this one has none yet.
            Entry::NewProject => {
                self.close_menu(true, tx);
                self.refused = Some("A new project is made in Mercury's own window.".to_string());
            }
            Entry::Level(level) => {
                self.compose.level = level;
                self.close_menu(true, tx);
            }
            // An agent's last model is remembered; its default is the absence of one (`writeLastModel`).
            Entry::Model(pick) => {
                let default = rules::default_model_for(&pick.engine, self.models.codex_default.as_deref());
                self.picks.change(|picks| {
                    if pick.model.is_empty() || pick.model == default {
                        picks.model.remove(&pick.engine);
                    } else {
                        picks.model.insert(pick.engine.clone(), pick.model.clone());
                    }
                });
                self.compose.pick = Some(pick);
                self.close_menu(true, tx);
            }
            Entry::AllModels | Entry::Recent => {
                menu.every_model = entry == Entry::AllModels;
                menu.lit = rules::entry_lit_first(&self.menu_entries(&menu), |entry| self.entry_is_picked(entry));
                self.compose.menu = Some(menu);
            }
            // The level stays in the menu: the person may be here to set the model and the
            // level both. Pressed again, their own pick is let go (`pickEffort`).
            Entry::Effort(level) => {
                let engine = self.compose_pick().engine;
                let kept = self.kept_effort(&engine);
                self.picks.change(|picks| {
                    if kept.as_deref() == Some(level.as_str()) {
                        picks.effort.remove(&engine);
                    } else {
                        picks.effort.insert(engine.clone(), level.clone());
                    }
                });
            }
        }
    }

    /// What a picture taken with a menu open says of it (`menu` in its fixture): the same chip
    /// is pressed, and the same page turned.
    pub(crate) fn picture_menu(&mut self, which: &str, tx: &mut Tx) {
        let kind = match which {
            "project" => MenuKind::Project,
            "priority" => MenuKind::Level,
            _ => MenuKind::Model,
        };
        self.toggle_menu(kind, tx);
        if which == "models" {
            let turn = self.compose.menu.as_ref().and_then(|menu| self.menu_entries(menu).iter().position(|entry| *entry == Entry::AllModels));
            if let Some(at) = turn {
                self.press_entry(at, tx);
            }
        }
    }

    /// The project's name, as a line about a thread says where it went.
    fn project_named(&self, slug: &str) -> String {
        self.snap.products.iter().find(|p| p.slug == slug).map_or_else(|| slug.to_string(), |p| p.name.clone())
    }

    /// Sends what is in the box as a new thread (Send, or ⌘Return). The box empties at once
    /// and a line says where the thread is going; a refusal puts every word back.
    pub fn send_thread(&mut self, tx: &mut Tx) {
        let Some(message) = rules::thread_message(&self.compose.words.text) else {
            return;
        };
        let Some(product) = self.compose_project().filter(|p| rules::practice_refusal(Some(p)).is_none()).map(|p| p.slug.clone()) else {
            return;
        };
        let pick = self.compose_pick();
        let levels = rules::effort_choices(&pick, &self.models);
        // A level is sent only when one was picked that this model takes.
        let effort = self.kept_effort(&pick.engine).filter(|level| rules::effort_picked(Some(level), &levels));
        let harness = rules::harness_fields(&pick, effort.as_deref(), Engines::of(&self.snap).rows().len(), self.models.codex_default.as_deref());
        let payload = rules::compose(&product, &message, rules::sent_priority(self.compose.level), harness, false);
        let draft = write::Draft { words: self.compose.words.text.clone(), level: self.compose.level, pick: self.compose.pick.clone() };
        // What was done before this and still waits its three seconds is written first.
        self.write_what_waits();
        let on_its_way = self.send(rules::COMPOSE, payload, write::Sent::Thread { product: product.clone(), draft });
        if !on_its_way && self.refused.is_some() {
            // No door to send through: the words stay where they are, and the line says so.
            return;
        }
        self.picks.change(|picks| picks.project = Some(product.clone()));
        self.compose.words = TextEdit::new("");
        self.compose.menu = None;
        self.compose.on_chip = None;
        self.compose.o_waits = None;
        tx.blur();
        // Alone, on the Inbox, the tab moves to where the thread will land; over Home or a
        // conversation the person stays where they are.
        let covered = self.view != View::Inbox;
        if let Some(tab) = rules::tab_after_sending(window::team_mode(&self.snap), false, Some(self.inbox_tab), covered) {
            self.show_tab(tab);
        }
        self.say(rules::sending_line(&self.project_named(&product), false), false, tx);
        if on_its_way {
            self.compose.not_taken_at = Some(tx.clock() + rules::NOT_ANSWERING_MS / 1000.0);
        }
    }

    /// What the engine said to a new thread. Taken, the thread is in the lists at once and a
    /// line names it, with O to open it and Z to take it back. Refused, every word goes back
    /// in the box and a line says why.
    pub(crate) fn thread_landed(&mut self, landed: write::Landed, tx: &Tx) {
        let write::Sent::Thread { product, draft } = landed.sent.clone() else {
            return;
        };
        self.compose.not_taken_at = None;
        if let Some(why) = landed.refusal() {
            // Unless the box holds something else by now: an older send never takes words away.
            if self.compose.words.text.trim().is_empty() {
                self.compose.words = TextEdit::new(draft.words);
                self.compose.level = draft.level;
                self.compose.pick = draft.pick;
            }
            self.compose.o_waits = None;
            self.say(rules::not_sent(&why), false, tx);
            return;
        }
        let made = landed.answer.ok().and_then(|row| serde_json::from_value::<Item>(row).ok()).filter(|row| !row.id.is_empty());
        // The lists as the engine has them now, with the new thread in them a whole read sooner
        // than the next turn would bring it (`showJustMade`).
        let limit = made.as_ref().and_then(|row| landed.snapshot.as_ref().and_then(|snapshot| rules::limit_of(snapshot, &row.id)));
        if let Some(snapshot) = landed.snapshot.and_then(|snapshot| serde_json::from_value(snapshot).ok()) {
            self.snap = snapshot;
        }
        if let Some(row) = &made {
            if !self.snap.items.iter().any(|i| i.id == row.id && i.product == row.product) {
                self.snap.items.push(row.clone());
            }
        }
        self.refresh();
        let back = limit.as_ref().map(|limit| (limit.tool.as_str(), back_at(limit.until, self.now)));
        let line = rules::sent_line(&self.project_named(&product), false, back.as_ref().map(|(tool, at)| (*tool, at.as_deref())));
        let Some(row) = made else {
            self.say(rules::toast_parts(&line).0, false, tx);
            return;
        };
        self.last = Some(taking_back(&row.id, &row.product, row.title.as_deref().unwrap_or_default(), draft));
        // O pressed while it was on its way opens it now, if the person is still where they were.
        if self.compose.o_waits.take().is_some_and(|screen| screen == self.view) {
            self.toast = None;
            self.open_thread(&row.id, &row.product);
            return;
        }
        self.say_about(&line, (row.id, row.product), tx);
    }

    /// The thread the line in the corner is about, as its row names it, once the lists have it.
    pub fn toast_title(&self) -> Option<String> {
        let (id, product) = self.toast.as_ref()?.goes.as_ref()?;
        let listed = |rows: &[crate::data::Row]| rows.iter().any(|r| &r.id == id && &r.product == product);
        if !(listed(&self.lists.needs) || listed(&self.lists.progress) || listed(&self.lists.later) || listed(&self.lists.done)) {
            return None;
        }
        self.snap.items.iter().find(|i| &i.id == id && &i.product == product).map(title::row_title)
    }

    /// O, or a press on the line: into the thread it is about, while it is up and names one
    /// that is not already open (`opensTheToast`). With a thread still on its way, O waits
    /// for it. False when O is nobody's.
    pub fn open_what_the_line_names(&mut self) -> bool {
        let goes = self.toast.as_ref().and_then(|t| t.goes.clone());
        let Some((id, product)) = goes else {
            if self.compose.not_taken_at.is_some() {
                self.compose.o_waits = Some(self.view.clone());
                return true;
            }
            return false;
        };
        if matches!(&self.view, View::Thread { id: open, product: of } if open == &id && of == &product) {
            return false;
        }
        self.toast = None;
        self.open_thread(&id, &product);
        true
    }

    /// Z on a thread sent from the box: it is closed, which stops its agent, unless the agent
    /// has already answered. Then closing would only hide the answer, so the answer is shown.
    pub(crate) fn withdraw(&mut self, last: write::Undo, tx: &mut Tx) {
        let (id, product) = last.thread.clone();
        let row = self.snap.items.iter().find(|i| i.id == id && i.product == product);
        if rules::answered_already(row) {
            self.say(rules::TOO_LATE_TO_WITHDRAW.to_string(), false, tx);
            self.open_thread(&id, &product);
            return;
        }
        if !self.send_all(last.writes.clone(), last.sent.clone()) {
            // Not sent, so not undone: Z still has it.
            self.last = Some(last);
        }
    }

    /// What the engine said to that close. Done, what the box held goes back in it, unless
    /// it holds something newer; refused, Z keeps the thread and a line says why.
    pub(crate) fn withdraw_landed(&mut self, landed: write::Landed, tx: &Tx) {
        let refusal = landed.refusal();
        let write::Sent::Withdraw { thread, title, draft } = landed.sent else {
            return;
        };
        let Some((product, id)) = thread.split_once('/').map(|(product, id)| (product.to_string(), id.to_string())) else {
            return;
        };
        if let Some(why) = refusal {
            self.say(rules::could_not_undo(&why), false, tx);
            self.last = Some(taking_back(&id, &product, &title, draft));
            return;
        }
        // Out of Done from now on: a send taken back is not finished work.
        self.withdrawn.insert(id, crate::clock());
        self.refresh();
        let back = self.compose.words.text.trim().is_empty() && !draft.words.trim().is_empty();
        if back {
            self.compose.words = TextEdit::new(draft.words);
            self.compose.level = draft.level;
            self.compose.pick = draft.pick;
        }
        self.say(rules::said_withdrawn(&title, back), false, tx);
    }

    /// What a picture taken just after a send hands over (`sent` in its fixture): the row the
    /// page was answered with and when. The same answer lands here, and the line about it is
    /// as old as the page's was at the shutter.
    pub(crate) fn picture_sent(&mut self, sent: &Value, tx: &Tx) {
        let Some(made) = sent.get("made") else {
            return;
        };
        let product = made.get("product").and_then(Value::as_str).unwrap_or_default().to_string();
        let draft = write::Draft { words: self.fixture.get("typed").and_then(Value::as_str).unwrap_or_default().to_string(), ..write::Draft::default() };
        self.thread_landed(write::Landed { sent: write::Sent::Thread { product, draft }, answer: Ok(made.clone()), snapshot: None }, tx);
        let ago = sent.get("at").and_then(Value::as_f64).map_or(0.0, |at| (self.now - at) / 1000.0);
        if let Some(toast) = &mut self.toast {
            toast.since -= ago;
            toast.until -= ago;
        }
    }

    /// The line the window owes a thread it has not heard back about (`SEND_NOT_TAKEN`):
    /// silence after Send reads as the send not having happened. True when it was just said.
    pub(crate) fn say_if_not_taken(&mut self, at: f64, tx: &mut Tx) -> bool {
        match self.compose.not_taken_at {
            Some(due) if at >= due => {
                self.compose.not_taken_at = None;
                self.say(rules::SEND_NOT_TAKEN.to_string(), false, tx);
                true
            }
            Some(due) => {
                tx.wake_after(std::time::Duration::from_secs_f64(due - at));
                false
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_agent_out_of_usage_is_back_at_a_time_and_on_a_day_when_that_is_not_today() {
        let now = 1_784_000_000_000.0;
        assert_eq!(back_at(now - 1.0, now), None);
        assert_eq!(back_at(now, now), None);
        // Whatever the zone, a moment a minute off names no day unless midnight falls between.
        let soon = back_at(now + 60_000.0, now).unwrap();
        assert!(soon.ends_with("am") || soon.ends_with("pm") || soon.split(' ').count() == 2, "{soon}");
        // Two days out names the weekday after the time, and a fortnight out the date too.
        assert_eq!(back_at(now + 2.0 * 86_400_000.0, now).unwrap().split(' ').count(), 2);
        assert_eq!(back_at(now + 14.0 * 86_400_000.0, now).unwrap().split(' ').count(), 4);
    }
}
