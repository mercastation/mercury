//! The box a thread is started in: open on Home's page, under the greeting, and the same box as
//! the New thread card N opens over any other page. Built from the web app's own measurements
//! of its card (`renderer/src/threads/ThreadComposer.tsx`, `thread-composer.css` and Home's
//! `home.css`); what it sends, to which project and on which model is `rules/compose.rs`.
//!
//! The words are the person's to type and ⌘Return or Send sends them. Nothing waits three
//! seconds here, unlike a reply or a close: a thread held back would be in no list at all
//! while the person looks for it. It is written at once, a line says where it is going and
//! then where it went, and Z takes it back by closing it, which stops its agent.
//!
//! The chips under the words say what the thread will be sent with, and each opens a menu to
//! change it: the model with its agent and how hard it thinks, the project, the level, and
//! the caret beside Send, Later. One menu is open at a time; the pointer and the arrows walk
//! its rows, Enter picks, and Escape closes the menu before anything else. Files dropped on the
//! box, picked with the paper clip or pasted wait in a row under the words and go first when
//! it is sent (`attach.rs`).
//!
//! There is one box and two places it is drawn, as in the web window, where one composer is
//! drawn either in Home's slot or in the dialog and both read the same draft: open on Home's
//! page, or as the card over any page, with a head over the words (who it is to, the model).
//! While the card is up, Home's slot is empty.

use rune::edit::Key;
use rune::time::LocalTime;
use rune::{Drag, El, KeyEvent, Phase, ScrollSpec, TextCase, TextEdit, TextStyle, Tx, col, el, icon, row, spacer, text, text_input};
use serde_json::Value;

use crate::data::{Item, Product};
use crate::rules::compose::{self as rules, Engines, Entry, Hang, Keyboard, Level, Pick};
use crate::rules::{title, window};
use crate::theme::*;
use crate::{State, View, attach, chrome, home, icons, write};

/// The key of the field the words are typed in.
pub const FIELD: &str = "compose";
/// The box on Home's page, whose height Home's column makes room for.
pub const BOX: &str = "compose-box";
/// And of the open menu's rows, whose width decides whether it has to slide.
const MENU: &str = "compose-menu";
/// The card, which a press outside closes.
const CARD: &str = "compose-card";
/// Send, and the caret beside it that opens Later. The reply box has a Send of its own.
const SEND: &str = "compose-send";
const CARET: &str = "compose-caret";
/// The paper clip, which opens the system's file picker.
const CLIP: &str = "compose-clip";

/// One of the box's menus, named by the control that opens it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuKind {
    Model,
    Project,
    Level,
    /// The caret beside Send (`laterMenu`).
    Later,
}

impl MenuKind {
    const ALL: [MenuKind; 4] = [MenuKind::Model, MenuKind::Project, MenuKind::Level, MenuKind::Later];

    /// The key of the control a press opens the menu with.
    fn trigger(self) -> &'static str {
        match self {
            MenuKind::Model => "chip-model",
            MenuKind::Project => "chip-project",
            MenuKind::Level => "chip-level",
            MenuKind::Later => CARET,
        }
    }

    /// The key of what the menu hangs from: the control, except Later's, which lines up with
    /// the whole of Send (`ref={anchor('later')}` on `.tc-send`).
    fn anchor(self) -> &'static str {
        match self {
            MenuKind::Later => "compose-send-pair",
            kind => kind.trigger(),
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
    /// The model menu of a conversation's reply box (`reply.rs`), which is this box's menu as
    /// the web window's is the card's (`ReplyModel` in `renderer/src/components/ReplyModel.tsx`
    /// draws `ModelMenu`): the same rows, read off the reply's pick, rising out of its chip.
    pub reply: bool,
}

/// What the model menu is drawn from for the box it hangs from: the model it is on, the level
/// its strip lights, and the levels the strip offers.
pub struct ModelFacts {
    pub pick: Pick,
    pub shown: Option<String>,
    pub levels: Vec<(String, String)>,
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
    /// "Add it to Later" was picked under the caret: Send writes the thread down under Later,
    /// where nothing starts it until the person does.
    pub later: bool,
    /// The files dropped on it, picked or pasted, in the order they came (`attach.rs`).
    pub files: Vec<attach::Attached>,
    /// The engine's answers about the files it is turning into pictures.
    pub staging: attach::Staging,
    pub menu: Option<Menu>,
    /// The chip the keyboard was left on when Escape closed its menu: an arrow opens the menu
    /// again, a letter goes to the words, and a second Escape lets go of the box.
    pub on_chip: Option<MenuKind>,
    /// The box is the New thread card over the page, not the one on Home's page.
    pub card: bool,
    /// When the window says that a thread on its way has not been taken yet, on its clock.
    not_taken_at: Option<f64>,
    /// O pressed while a thread was on its way, and the screen it was pressed on: the thread
    /// is opened when it lands, unless the person has gone somewhere else by then.
    o_waits: Option<View>,
}

impl Default for Composer {
    fn default() -> Composer {
        Composer {
            words: TextEdit::new(""),
            level: None,
            project: None,
            pick: None,
            later: false,
            files: Vec::new(),
            staging: attach::Staging::default(),
            menu: None,
            on_chip: None,
            card: false,
            not_taken_at: None,
            o_waits: None,
        }
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

    /// What Send takes off the box, for a refusal or Z to put back.
    fn draft(&self) -> write::Draft {
        write::Draft { words: self.words.text.clone(), level: self.level, pick: self.pick.clone(), later: self.later, files: self.files.clone() }
    }

    /// Whether there is anything in the box to keep: words, or files (`composeDraftIsEmpty`).
    fn holds_something(&self) -> bool {
        !self.words.text.trim().is_empty() || !self.files.is_empty()
    }

    /// What a refusal or Z puts back: every word, every file and every pick, in a box that holds
    /// nothing newer (`restoreComposeDraft`). True when it did.
    fn put_back(&mut self, draft: write::Draft) -> bool {
        if self.holds_something() || (draft.words.trim().is_empty() && draft.files.is_empty()) {
            return false;
        }
        self.words = TextEdit::new(draft.words);
        self.level = draft.level;
        self.pick = draft.pick;
        self.later = draft.later;
        self.files = draft.files;
        true
    }
}

/// The box's words, each standing where Chromium puts its glyphs (`theme::pen`).
fn label(s: &str, size: f32, weight: f32, c: rune::Color) -> El {
    pen(crate::theme::label(s, size, weight, c), size)
}

/// The card's hairlines, one step lighter than its edge (`--tc-hair`).
fn hair() -> rune::Color {
    rgb(59, 57, 55)
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
    let chip = home::chip(inside(if open { ink() } else { dim() })).key(kind.trigger()).cursor(rune::Cursor::Pointer).on_background_press_tx(move |s: &mut State, tx| s.toggle_menu(kind, tx));
    if open { chip.border(1.0, rule()) } else { chip }
}

/// The project's chip and the level's, which every bar has.
fn filed_chips(s: &State) -> [El; 2] {
    let project = s.compose_project();
    // Their titles: the project's name, and what the level is (`title` on each `.tc-chip`).
    let named = project.map_or_else(|| "No project".to_string(), |p| p.name.clone());
    [
        menu_chip(s, MenuKind::Project, |tone| match project {
            Some(p) => vec![swatch(&p.slug), label(&p.name, 13.0, 400.0, tone).group_color(ink())],
            None => vec![label("No project", 13.0, 400.0, tone).group_color(ink())],
        })
        .help(named),
        menu_chip(s, MenuKind::Level, |tone| vec![level_mark(s.compose.level), label(rules::choice_label(s.compose.level), 13.0, 400.0, tone).group_color(ink())]).help("Priority"),
    ]
}

/// The caret half of Send, which opens the menu to send later; with Later picked it says so
/// before the caret (`.tc-send-caret`, and `.set` with `.tc-send-when`).
fn later_caret(s: &State) -> El {
    let caret = if s.compose.later {
        let set = row()
            .h(32.0)
            .items_center()
            .gap(6.0)
            .pl(11.0)
            .pr(9.0)
            .corners([0.0, 6.0, 6.0, 0.0])
            .fill(rgba(239, 235, 231, 0.15))
            .border_sides([0.0, 0.0, 0.0, 1.0], rgba(26, 24, 23, 0.55))
            .border_layout()
            .child(pen(text(rules::LATER_WORD).font(figures(12.5, 500.0)).color(ink()).group_color(white(1.0)), 12.5))
            .child(icon(&icons::CARET, 13.0).color(ink()).group_color(white(1.0)));
        home::brightened(set).cursor(rune::Cursor::Pointer)
    } else {
        home::send_caret()
    };
    caret.key(CARET).help("Send later").on_background_press_tx(|s: &mut State, tx| s.toggle_menu(MenuKind::Later, tx))
}

/// The paper clip, why Send is off when it is off for want of a project, and Send.
fn bar_end(s: &State) -> Vec<El> {
    // The two halves of Send are one control with no room between them (`.tc-send`).
    let send = row().key(MenuKind::Later.anchor()).child(home::send_main(s.compose_ready()).key(SEND).help("Send · ⌘↵").on_background_press_tx(|s: &mut State, tx| s.send_thread(tx))).child(later_caret(s));
    let clip = home::attach().key(CLIP).help("Attach files").on_background_press_tx(|s: &mut State, tx| s.pick_files(tx));
    if s.compose_project().is_none() {
        // `.tc-why` takes the room the clip would otherwise push Send across, so the clip
        // stays by the chips. Making a project is the web window's to do, so the way out is
        // words here and not a button.
        let why = label(&format!("{} {}", rules::NO_PROJECT_REASON, rules::NO_PROJECT_ACTION), 13.0, 400.0, dim());
        return vec![clip, row().grow(1.0).min_w(0.0).pl(8.0).child(why), send];
    }
    vec![spacer(), clip, send]
}

/// The bar under the words on the page: who gets the thread where someone besides the agent
/// could, the model, the project, the level, and Send.
fn bar(s: &State) -> El {
    let mut bar = row().pt(14.0).items_center().gap(8.0);
    if window::to_offered(&s.snap) {
        bar = bar.child(crate::feel::answers(home::chip(vec![label("Agent", 13.0, 400.0, dim()).group_color(ink())])));
    }
    let model = rules::model_label(&s.compose_pick(), &s.models);
    bar.child(menu_chip(s, MenuKind::Model, |tone| vec![label(&model, 13.0, 400.0, tone).group_color(ink())])).children(filed_chips(s)).children(bar_end(s))
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
        // A picture or a file on the clipboard is attached rather than typed (`onPaste`).
        .on_input_paste(|s: &mut State, content, _| s.paste_into_the_box(content, crate::clock()))
}

/// The lines between the words and the bar (`.tc-note`): why Practice takes nothing, and where
/// words about code are going. `pad` is the room the place it is drawn in gives a note.
fn notes(s: &State, pad: impl Fn(El) -> El) -> Vec<El> {
    let mut lines = Vec::new();
    let note = |words: &str| pen(text(words).font(TextStyle::new(13.0, 400.0).lh(19.0)).color(dim()).wrap(), 13.0);
    if let Some(why) = s.compose_project().and_then(|p| rules::practice_refusal(Some(p))) {
        lines.push(pad(el().child(note(why))));
    }
    if let (Some(code), Some(pointed)) = (s.compose_code(), s.compose_pointed()) {
        // The way back is a word of the sentence, underlined (`.tc-note button`).
        let slug = pointed.slug.clone();
        let keep = el()
            .child(pen(text(rules::keep(&pointed.name)).font(TextStyle::new(13.0, 400.0).lh(19.0)).color(ink()), 13.0))
            .child(el().pin(Some(0.0), Some(16.5), Some(0.0), None).h(1.0).fill(rule()))
            .cursor(rune::Cursor::Pointer)
            .on_background_press(move |s: &mut State| s.compose.project = Some(slug.clone()));
        lines.push(pad(row().gap(4.0).child(note(&rules::going_to_code(&code.name))).child(keep)));
    }
    lines
}

/// The box on Home's page (`.hm-start .tc-card.tc-inline`): the words large over the bar, on
/// the page's glass. Its edge brightens while the caret is in it.
pub fn inline(s: &State) -> El {
    // `caret-color: var(--accent)` on the box drawn on a page.
    let field = words_field(s, TextStyle::new(22.0, 400.0).lh(30.0), accent());
    glass(col())
        .key(BOX)
        .pt(16.0)
        .px(18.0)
        .pb(14.0)
        .radius(10.0)
        // `:focus-within`: the caret in the words, or the keyboard on a chip or in its menu.
        .border(1.0, white(if s.compose.keyboard() == Keyboard::Nothing { 0.2 } else { 0.3 }))
        .border_layout()
        .box_shadow(shadow(0.0, 0.0, 0.0, 3.0, white(0.03), false))
        .box_shadow(shadow(0.0, 12.0, 40.0, 0.0, rgba(0, 0, 0, 0.3), false))
        // Files dropped anywhere on the box attach. The web window takes them on its words,
        // which are most of it; a drop on the rest of it does nothing there.
        .on_drop_files(|s: &mut State, files, _| s.attach_paths(files))
        // One line tall until the words need more: 74 points with the room under them, all of
        // it the words' box, as the web window's text box is: the I-beam over it, and a press
        // anywhere in it puts the caret after the words.
        .child(el().min_h(74.0).pb(14.0).cursor(rune::Cursor::Text).on_press_tx(|s: &mut State, tx| s.write_in_the_box(tx)).child(field.w_full()))
        // The files, under the words and over the notes (`.tc-inline .attach-row`).
        .children(attach::rows(s, |r| r.pt(12.0)))
        .children(notes(s, |n| n.pt(12.0)))
        .child(bar(s))
}

/* -------------------------------- the card -------------------------------- */

/// A value of the card's head (`.tc-word`): the word with a dotted line under it, a point
/// wide every other point, three under its foot; the line brightens while the word's menu is
/// open (`.tc-word.open`) and while the pointer is on it (`.tc-word:hover`), when the bright
/// line is laid over the faint one. The line is as long as the word and stops on the last whole
/// dot.
fn head_word(words: &str, open: bool) -> El {
    let dots = |tone| row().gap(1.0).children((0..160).map(move |_| el().size(1.0, 1.0).fill(tone)));
    let line = |tone| el().pin(Some(0.0), Some(18.5), Some(0.0), None).h(1.0).clip_corners([0.0; 4]).child(dots(tone));
    el().hover_group()
        .cursor(rune::Cursor::Pointer)
        .child(pen(text(words).font(TextStyle::new(14.0, 400.0).lh(21.0)).color(ink()), 14.0))
        .child(line(if open { muted() } else { rule() }))
        .child(line(muted()).group_opacity(0.0, 1.0))
}

/// A field of the card's head (`.tc-field`): its name in small capitals, then its value. A
/// field under another has a hairline over it.
fn head_field(name: &str, value: El, first: bool) -> El {
    let field = row().h(42.0).px(18.0).items_center().child(el().w(84.0).child(text(name).font(mono(11.0, 11.0).tracking(0.88)).text_case(TextCase::Uppercase).color(muted()))).child(value);
    if first { field } else { field.border_sides([1.0, 0.0, 0.0, 0.0], hair()).border_layout() }
}

/// An agent's face, as To draws it: two eyes in a faint ring (`AgentFace`).
fn agent_face() -> El {
    row().size(18.0, 18.0).radius(9.0).center().border(1.0, rgba(239, 235, 231, 0.12)).child(icon(&icons::AGENT_EYES, 16.0).color(dim()))
}

/// The card's head (`.tc-head`): To where someone besides the agent could get the thread, then
/// the model, whose word opens its menu.
fn card_head(s: &State) -> El {
    let mut head = col().border_sides([0.0, 0.0, 1.0, 0.0], hair()).border_layout();
    let to = window::to_offered(&s.snap);
    if to {
        // The face hangs to the left of the word, so the two values start on one line
        // (`.tc-word > .agent-face`, a point higher than the word's middle). Who else could get
        // the thread is the web window's to pick: To says Agent here, and opens nothing.
        let face = agent_face().ml(-27.0).mr(9.0).mt(-1.0).mb(1.0);
        head = head.child(head_field("To", row().items_center().child(face).child(crate::feel::answers(head_word("Agent", false))), true));
    }
    let open = s.compose.menu.as_ref().is_some_and(|m| m.kind == MenuKind::Model);
    let model = head_word(&rules::model_label(&s.compose_pick(), &s.models), open)
        .key(MenuKind::Model.trigger())
        .cursor(rune::Cursor::Pointer)
        .on_background_press_tx(|s: &mut State, tx| s.toggle_menu(MenuKind::Model, tx));
    head.child(head_field("Model", model, !to))
}

/// The New thread card over the page (`.modal-backdrop.tc-veil` around `.tc-card`): the page
/// veiled and blurred a little behind it, and the card centred a little above the middle of
/// the window. The same box as Home's, with its head over the words and its bar under a
/// hairline. A press outside it closes it, keeping every word.
fn card(s: &State) -> El {
    let (w, h) = s.size;
    let width = 680.0_f32.min(0.92 * w);
    // In the dialog the caret is the text's own colour, and the words scroll inside a box
    // 156 points tall (`.tc-text`).
    let field = words_field(s, TextStyle::new(15.0, 400.0).lh(23.25), ink());
    // All of it the words' box, as the web window's text box is: the I-beam over it, and a
    // press anywhere in it puts the caret after the words.
    let words = col()
        .h(156.0)
        .pt(12.0)
        .px(18.0)
        .pb(6.0)
        .cursor(rune::Cursor::Text)
        .on_press_tx(|s: &mut State, tx| s.write_in_the_box(tx))
        .child(col().grow(1.0).min_h(0.0).scroll(ScrollSpec { stick_bottom: false }).child(field.w_full()));
    // The files, between the words and the notes (`.tc-card .attach-row`).
    let files = attach::rows(s, |r| r.px(18.0).pb(10.0));
    let lines = notes(s, |n| n.px(18.0).pb(10.0));
    // How tall the card is, so it can stand where the grid puts it: as it was last laid out,
    // else its edge, the head's fields and hairline, the words, the files on one line, one
    // line a note and the bar.
    let fields = if window::to_offered(&s.snap) { 2.0 } else { 1.0 };
    let filed = if files.is_empty() { 0.0 } else { attach::rows_tall(&s.compose.files) + 10.0 };
    let tall = rune::memo::layout_size(CARD).map_or(2.0 + fields * 42.0 + 1.0 + 156.0 + filed + lines.len() as f32 * 29.0 + 57.0, |[_, tall]| tall);
    // `place-items: center` in the window less a tenth of its height at the foot.
    let top = ((h - 0.1 * h - tall) / 2.0).max(0.0).round();
    let bar = row().pt(12.0).pr(14.0).pb(12.0).pl(18.0).gap(8.0).items_center().border_sides([1.0, 0.0, 0.0, 0.0], hair()).border_layout().children(filed_chips(s)).children(bar_end(s));
    col()
        .key(CARD)
        .absolute(((w - width) / 2.0).round(), top)
        .w(width)
        .radius(10.0)
        .fill(rgb(45, 43, 41))
        .border(1.0, edge())
        .border_layout()
        .box_shadow(shadow(0.0, 32.0, 90.0, 0.0, rgba(0, 0, 0, 0.55), false))
        .box_shadow(shadow(0.0, 1.0, 0.0, 0.0, white(0.05), true))
        // A press on the card's own padding leaves the keyboard on nothing (`onClick` stops it
        // reaching the veil), where a letter still goes to the words.
        .on_background_press_tx(|s: &mut State, tx| s.let_go(tx))
        // Escape that nothing else took, and a press outside: the card goes, the words stay.
        .modal_tx(true, |s: &mut State, tx| s.close_card(tx))
        // Files dropped anywhere on the card attach, as on the box on Home's page.
        .on_drop_files(|s: &mut State, files, _| s.attach_paths(files))
        .child(card_head(s))
        .child(words)
        .children(files)
        .children(lines)
        .child(bar)
}

/// The box's layers over the page: the card with the page veiled under it, when it is up, and
/// the open menu over either.
pub fn layers(s: &State) -> Vec<El> {
    let mut layers = Vec::new();
    if s.compose.card {
        let (w, h) = s.size;
        layers.push(el().absolute(0.0, 0.0).size(w, h).backdrop_blur(3.0).fill(rgba(16, 14, 13, 0.52)).child(card(s)));
    }
    layers.extend(menu_layer(s));
    // And the same model menu, raised out of a conversation's reply box.
    layers.extend(reply_menu_layer(s));
    layers
}

/* -------------------------------- the menus ------------------------------- */

/// A row of a menu is 34 points tall: a 20 point line and 7 over and under it (`.tc-row`).
const ROW: f32 = 34.0;

/// What a menu's `lit` is while no entry is lit: the reply box's menu opened from the model's
/// name on the folded box, with the caret still in the words.
pub const NO_ENTRY: usize = usize::MAX;

/// What every entry of a menu does under the pointer: it takes the keys when the pointer
/// moves on it, and a press is Enter on it. On a pointer that moves, not one that is merely
/// there: a page that moves under a still pointer must not take a row back from the arrows.
fn entry(e: El, at: usize) -> El {
    e.key(format!("entry-{at}")).cursor(rune::Cursor::Pointer).on_pointer_move_local(move |s: &mut State, _, tx: &mut Tx| s.light_entry_by_pointer(at, tx)).on_background_press_tx(move |s: &mut State, tx| s.press_entry(at, tx))
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
        MenuKind::Later => {
            part(menu_head(rules::SEND_LATER), 24.5);
            for (at, entry) in rules::later_entries(s.compose.later).iter().enumerate() {
                let (glyph, words, on) = match entry {
                    Entry::RemoveTheTime => (&icons::CLEAR, rules::REMOVE_THE_TIME, false),
                    _ => (&icons::CLOCK, rules::ADD_TO_LATER, s.compose.later),
                };
                part(menu_row(at, lit(at), (on, false), Some(icon(glyph, 15.0).color(muted())), words, None), ROW);
            }
            // `.tc-wide`.
            300.0
        }
        MenuKind::Model => {
            let engines = Engines::of(&s.snap);
            let facts = s.model_facts(menu.reply);
            let pick = facts.pick.clone();
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
            let levels = facts.levels;
            if !levels.is_empty() {
                part(menu_sep(), 9.0);
                part(el().pt(6.0).px(10.0).pb(2.0).child(text("Effort").font(TextStyle::new(11.0, 400.0).lh(20.0).tracking(0.22)).color(muted())), 28.0);
                let shown = facts.shown;
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

/// A menu built and placed: its parts in their shell, hung from its control (`anchor`) where
/// `hang` says for a menu so wide, and kept inside the window and inside `sides`
/// (`rules::fit_menu`). The shell, and where it landed.
fn placed_menu(s: &State, menu: &Menu, anchor: [f32; 4], hang: impl Fn(f32) -> Hang, sides: (f32, f32)) -> (El, rules::Fit) {
    let (parts, sum, least) = menu_parts(s, menu);
    // Its edge and the 5 points inside it, twice, and a point between each two parts.
    let tall = 12.0 + sum + parts.len().saturating_sub(1) as f32;
    let wide = rune::memo::layout_size(MENU).map_or(least, |[wide, _]| wide + 2.0);
    let fit = rules::fit_menu(anchor, hang(wide), (wide, tall), s.size.1, sides);
    let rows = col().key(MENU).min_w(least - 2.0).items_stretch().gap(1.0).px(5.0).py(5.0).children(parts);
    // No clip on the shell itself: Rune's rounded clip takes the element's own shadow with it,
    // and the shadow is what lifts the menu off the card. The rows keep inside its corners.
    let shell = col().radius(10.0).fill(rgb(54, 52, 49)).border(1.0, rule()).border_layout().box_shadow(shadow(0.0, 18.0, 50.0, 0.0, rgba(0, 0, 0, 0.5), false));
    let shell = match fit.max_height {
        // Too tall for the room it has: it is that tall, and its rows scroll inside it.
        Some(most) => shell.h(most).child(col().grow(1.0).min_h(0.0).scroll(ScrollSpec { stick_bottom: false }).child(rows)),
        None => shell.child(rows),
    };
    (shell, fit)
}

/// The model menu of a conversation's reply box, over the page: it rises out of the reply's
/// chip eight points over it, its left edge on the chip's (`.dock-chips .model-pick .tc-drop`,
/// `.tc-drop-up`), and is seen only inside the page. Under it lies a layer the size of the
/// window, and a press there closes it on the way down, as the web window's `pointerdown` does
/// (`ReplyModel`), going on to the reply's own Send (`reply.rs`).
pub fn reply_menu_layer(s: &State) -> Option<El> {
    let menu = s.reply_menu.as_ref()?;
    let anchor = rune::memo::window_layout(crate::reply::CHIP)?;
    let (w, h) = s.size;
    let hang = |_| Hang { below: false, gap: 8.0, dx: 0.0 };
    // The page starts where the card does, further right while the rail is open.
    let left = chrome::card_left(s);
    let (shell, fit) = placed_menu(s, menu, anchor, hang, (left, w - chrome::CARD[2]));
    let marks: Vec<(&'static str, [f32; 4])> = crate::reply::MARKS.into_iter().filter_map(|key| rune::memo::window_layout(key).map(|rect| (key, rect))).collect();
    let away = el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).on_drag(move |s: &mut State, d: &mut Drag, tx: &mut Tx| {
        if d.phase == Phase::Down {
            s.reply_pressed_away(d.window_point(), &marks, tx);
        }
    });
    let page_top = chrome::page_top(&s.view);
    let page = el().pin(Some(left), Some(page_top), Some(chrome::CARD[2]), Some(chrome::CARD[3])).clip_corners([0.0, 0.0, 6.0, 6.0]).child(shell.absolute(fit.left - left, fit.top - page_top));
    Some(el().absolute(0.0, 0.0).size(w, h).child(away).child(page))
}

/// The open menu, over the page or the card: hung from its control where the stylesheet hangs
/// it, and kept inside the window as the web window keeps it (`rules::fit_menu`). Under it lies
/// a layer the size of the window, and a press there closes the menu and goes on to what it
/// was aimed at (`pressed_away`).
fn menu_layer(s: &State) -> Option<El> {
    let menu = s.compose.menu.as_ref()?;
    // Where its control is in the window now. One not laid out yet has nothing hung from it.
    let anchor = rune::memo::window_layout(menu.kind.anchor())?;
    let (w, h) = s.size;
    let card = s.compose.card;
    // On the page every menu drops under its control from its left edge (`.tc-inline
    // .tc-menu`). On the card the head's menu drops from its word, sixteen points to the left
    // so its names stand under the value (`.tc-drop`), and the bar's rise twelve points over
    // their chips (`.tc-rise`). The caret's lines up with Send's right edge, eight points off
    // (`.tc-right`), under it on the page and over it on the card.
    let hang = |wide: f32| match (card, menu.kind) {
        (_, MenuKind::Later) => Hang { below: !card, gap: 8.0, dx: anchor[2] - wide },
        (false, _) => Hang { below: true, gap: 8.0, dx: 0.0 },
        (true, MenuKind::Model) => Hang { below: true, gap: 8.0, dx: -16.0 },
        (true, _) => Hang { below: false, gap: 12.0, dx: 0.0 },
    };
    // A menu on the page is seen only inside the page, which clips it (`.hm-page`); over the
    // card, anywhere in the window.
    let left = chrome::card_left(s);
    let sides = if card { (0.0, w) } else { (left, w - chrome::CARD[2]) };
    let (shell, fit) = placed_menu(s, menu, anchor, hang, sides);
    // On the way down, as the web window's `pointerdown` listener does, so the same press can
    // still land the caret in the words. Where the box's controls are is read here, while the
    // window is drawn: a handler has no layout to read.
    let marks: Vec<(&'static str, [f32; 4])> = [SEND, CLIP, FIELD, CARD].into_iter().chain(MenuKind::ALL.map(MenuKind::trigger)).filter_map(|key| rune::memo::window_layout(key).map(|rect| (key, rect))).collect();
    let away = el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).on_drag(move |s: &mut State, d: &mut Drag, tx: &mut Tx| {
        if d.phase == Phase::Down {
            s.pressed_away(d.window_point(), &marks, tx);
        }
    });
    if card {
        // Over the card the open menu is a dialog of its own, holding the keyboard itself: the
        // card's would otherwise hand it straight back to the words, the one field it has, and
        // the arrows would move the caret instead of the menu's rows.
        return Some(
            el().absolute(0.0, 0.0)
                .size(w, h)
                .on_key(|s: &mut State, e: &KeyEvent, tx: &mut Tx| key(s, e, tx).unwrap_or(false))
                .modal_tx(true, |s: &mut State, tx| s.close_menu(false, tx))
                .child(away)
                .child(shell.absolute(fit.left, fit.top)),
        );
    }
    let page = el()
        .pin(Some(left), Some(chrome::PAGE_TOP), Some(chrome::CARD[2]), Some(chrome::CARD[3]))
        .clip_corners([0.0, 0.0, 6.0, 6.0])
        .child(shell.absolute(fit.left - left, fit.top - chrome::PAGE_TOP));
    Some(el().absolute(0.0, 0.0).size(w, h).child(away).child(page))
}

/* ------------------------------- the keyboard ----------------------------- */

/// The keys while the box is on Home's page or the card is up, for the ones no text field
/// took. `None` when the key is not the box's, and the page's own keys go on to read it. The
/// card owns the screen as the web window's dialog does: every key that is not a chord is its
/// own (`keyWhileABoxIsOpen` in `renderer/src/keys.ts`).
pub fn key(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> Option<bool> {
    let card = s.compose.card;
    if !card && (s.view != View::Home || s.later.on.is_some()) {
        return None;
    }
    let chord = e.mods.cmd || e.mods.ctrl || e.mods.alt;
    let sends = e.key == Key::Enter && (e.mods.cmd || e.mods.ctrl);
    // An open menu has the keys: the arrows walk it around both ends, Enter presses the row
    // they are on, Escape closes it and leaves the keyboard on its control (`menuKeys`).
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
        // The card hears ⌘Return wherever the keyboard is; on the page, the box's own controls
        // hear it, and with nothing holding the keyboard the page sends only a box with words.
        if card || on == Keyboard::Card || rules::mod_enter_sends(true, on, !s.compose.words.text.trim().is_empty()) {
            s.send_thread(tx);
        }
        return Some(true);
    }
    if chord {
        return if card { Some(false) } else { None };
    }
    if e.key == Key::Escape {
        // On the card Escape closes it, keeping the words; on the page it lets go of the box.
        if card {
            s.close_card(tx);
            return Some(true);
        }
        if s.compose.on_chip.take().is_some() {
            return Some(true);
        }
        return None;
    }
    // An arrow on a chip opens its menu (`triggerKeys`).
    if let (Some(kind), Key::Down | Key::Up) = (s.compose.on_chip, &e.key) {
        s.toggle_menu(kind, tx);
        return Some(true);
    }
    let typed = match &e.key {
        Key::Text(t) if t.chars().count() == 1 => Some(t.clone()),
        Key::Enter => Some("\n".to_string()),
        _ => None,
    };
    // A letter is typed into the words wherever the keyboard sits on the card, and on the
    // page with the keyboard on one of its chips: no control under it ever hears it.
    if let Some(typed) = typed.filter(|_| rules::key_is_typed(true, chord, false, on, !card)) {
        s.caret_in_the_words(tx);
        s.compose.words.insert(&typed);
        return Some(true);
    }
    if card { Some(true) } else { None }
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

impl State {
    /// The projects a thread can be filed into, in the person's own order.
    pub fn projects_offered(&self) -> Vec<&Product> {
        rules::projects_offered(&rules::rank_products(&self.snap.products, &self.snap.supervisor.product_order))
    }

    /// The project the box points at: the one picked on it, else the open conversation's,
    /// else the one last sent to (`cardProject`).
    fn compose_pointed(&self) -> Option<&Product> {
        let on_screen = self.open_item().map(|i| i.product.as_str());
        rules::card_project(&self.projects_offered(), self.compose.project.as_deref(), on_screen, self.picks.get().project.as_deref())
    }

    /// Where words about code go instead, while nobody has picked a project on the box.
    fn compose_code(&self) -> Option<&Product> {
        let pointed = self.compose_pointed().filter(|_| self.compose.project.is_none())?;
        rules::code_project_for(&self.compose.words.text, &self.projects_offered(), &pointed.slug)
    }

    /// The project the box sends to.
    pub fn compose_project(&self) -> Option<&Product> {
        self.compose_code().or_else(|| self.compose_pointed())
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

    /// What the box sends as the thread's title and words: its words, or with files and no words
    /// a title naming the first file.
    fn compose_message(&self) -> Option<rules::Message> {
        rules::thread_message(&self.compose.words.text, self.compose.files.first().map(|a| a.file.name.as_str()))
    }

    /// Whether Send is lit: there is something to send, every file in the box can go, and there
    /// is somewhere it can run (`canSend`, with `filesReady`).
    pub fn compose_ready(&self) -> bool {
        self.compose_message().is_some() && attach::ready(&self.compose.files) && self.compose_project().is_some_and(|p| rules::practice_refusal(Some(p)).is_none())
    }

    /// The level last picked for an agent, when it is one worth keeping.
    fn kept_effort(&self, engine: &str) -> Option<String> {
        rules::remembered_effort(self.picks.get().effort.get(engine).map(String::as_str), engine)
    }

    /// Puts the caret back in the words where it was: after a pick, and for a letter typed
    /// with the keyboard on a chip. Through the words' own `focused`, which Rune honours the
    /// moment the field is drawn: an ask that waits for the field (`focus_when_shown`) outlived
    /// the send after it and put the caret back in the emptied box, where the next Z was typed.
    fn caret_in_the_words(&mut self, _: &mut Tx) {
        self.compose.on_chip = None;
        self.compose.words.focused = true;
    }

    /// Puts the caret in the box, at the end of what it holds (N on Home, or a letter typed
    /// with nothing to take it).
    pub fn write_in_the_box(&mut self, tx: &mut Tx) {
        let end = self.compose.words.text.len();
        self.compose.words.set_caret(end);
        self.caret_in_the_words(tx);
    }

    /// Escape in the box on the page: the keys go back to the page, and the words stay.
    pub fn leave_the_box(&mut self, tx: &mut Tx) {
        self.compose.words.focused = false;
        self.compose.on_chip = None;
        tx.blur();
    }

    /// A press on the card's own padding: no menu, no chip, no caret, and the keyboard on
    /// nothing.
    fn let_go(&mut self, tx: &mut Tx) {
        self.compose.menu = None;
        self.leave_the_box(tx);
    }

    /// N away from Home, or New thread: the card over the page, holding what the box holds,
    /// with the caret at the end of the words (`setModal('compose')` in `App.tsx`).
    pub fn open_card(&mut self, tx: &mut Tx) {
        self.compose.card = true;
        self.compose.menu = None;
        // One menu stands at a time: the card closes whatever the conversation under it had open.
        self.close_thread_menus();
        self.write_in_the_box(tx);
    }

    /// Escape on the card, or a press outside it: it goes, and what it held stays in the box
    /// for the next time (`onClose`, with the draft keeping every word).
    pub fn close_card(&mut self, tx: &mut Tx) {
        self.compose.card = false;
        self.let_go(tx);
    }

    /// What the model menu reads for the box it is open on: this box, or the reply box of the
    /// conversation on screen (`reply.rs`), each with its own pick and its own level.
    pub(crate) fn model_facts(&self, reply: bool) -> ModelFacts {
        let (pick, chosen) = if reply {
            let (pick, effort) = self.reply_menu_pick();
            (pick, effort)
        } else {
            let pick = self.compose_pick();
            let kept = self.kept_effort(&pick.engine);
            (pick, kept)
        };
        let levels = rules::effort_choices(&pick, &self.models);
        let shown = rules::effort_shown(chosen.as_deref(), &levels, rules::default_effort(&pick, &self.models).as_deref());
        ModelFacts { pick, shown, levels }
    }

    /// What the keys can land on in a menu, in the order it draws them.
    pub(crate) fn menu_entries(&self, menu: &Menu) -> Vec<Entry> {
        match menu.kind {
            MenuKind::Project => rules::project_entries(&self.projects_offered()),
            MenuKind::Level => rules::level_entries(),
            MenuKind::Later => rules::later_entries(self.compose.later),
            MenuKind::Model => {
                let engines = Engines::of(&self.snap);
                let levels = self.model_facts(menu.reply).levels;
                rules::model_entries(menu.every_model, &rules::recent_models(&self.snap.items, &self.models, &engines), &rules::all_models(&self.models, &engines), &levels)
            }
        }
    }

    /// Whether an entry is what the box is already on; `reply`, the reply box's.
    pub(crate) fn entry_is_picked(&self, entry: &Entry, reply: bool) -> bool {
        match entry {
            Entry::Project(slug) => self.compose_project().is_some_and(|p| &p.slug == slug),
            Entry::Level(level) => *level == self.compose.level,
            Entry::Model(pick) => *pick == self.model_facts(reply).pick,
            Entry::AddToLater => self.compose.later,
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
        let mut menu = Menu { kind, every_model: false, lit: 0, reply: false };
        menu.lit = rules::entry_lit_first(&self.menu_entries(&menu), |entry| self.entry_is_picked(entry, false));
        self.compose.menu = Some(menu);
        // The lists may have changed since the window opened: an agent's can land late.
        if kind == MenuKind::Model {
            self.read_the_models();
        }
    }

    /// Closes the open menu. A pick puts the caret back in the words, where the person was
    /// going anyway; Escape leaves the keyboard on the control that opened it (`close`).
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

    /// A press outside the open menu, as it goes down, with where the box's controls stood
    /// (`marks`). The menu closes and the press goes on to what it was aimed at in the box:
    /// another chip opens its menu, its own chip keeps the keyboard, the words take the caret,
    /// Send sends, the paper clip opens the picker. On the card a press outside the card closes
    /// the card too, as the veil's own press does. Anywhere else on the page the menu only
    /// closes.
    pub fn pressed_away(&mut self, at: [f32; 2], marks: &[(&'static str, [f32; 4])], tx: &mut Tx) {
        let inside = |key: &str| marks.iter().any(|(mark, [x, y, w, h])| *mark == key && at[0] >= *x && at[0] < x + w && at[1] >= *y && at[1] < y + h);
        let Some(menu) = self.compose.menu.take() else {
            return;
        };
        self.compose.on_chip = None;
        if inside(SEND) {
            return self.send_thread(tx);
        }
        if inside(CLIP) {
            return self.pick_files(tx);
        }
        if let Some(kind) = MenuKind::ALL.into_iter().find(|kind| inside(kind.trigger())) {
            if kind == menu.kind {
                self.compose.on_chip = Some(kind);
            } else {
                self.toggle_menu(kind, tx);
            }
            return;
        }
        if inside(FIELD) {
            return self.caret_in_the_words(tx);
        }
        if self.compose.card && !inside(CARD) {
            self.close_card(tx);
        }
    }

    /// The pointer moved onto an entry, or the arrows did: of the reply box's menu when that is
    /// the one open, else of this box's.
    pub fn light_entry(&mut self, at: usize) {
        if let Some(menu) = self.reply_menu.as_mut().or(self.compose.menu.as_mut()) {
            menu.lit = at;
        }
    }

    /// The pointer moved onto an entry. In the reply box's menu it also takes the keys from the
    /// words, as the web window's row takes the focus under the pointer (`onHover` in
    /// `ReplyModel.tsx`); this box's menus have them already.
    fn light_entry_by_pointer(&mut self, at: usize, tx: &mut Tx) {
        if self.reply_menu.is_some() && self.reply.focused {
            self.reply.focused = false;
            tx.blur();
        }
        self.light_entry(at);
    }

    /// Enter on an entry of the open menu, or a press on it.
    pub fn press_entry(&mut self, at: usize, tx: &mut Tx) {
        if self.reply_menu.is_some() {
            return self.press_reply_entry(at);
        }
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
                menu.lit = rules::entry_lit_first(&self.menu_entries(&menu), |entry| self.entry_is_picked(entry, false));
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
            // A row of Send later sets when it goes, and Send sends it, so there is still room
            // to add a line after picking (`pickWhen`).
            Entry::AddToLater | Entry::RemoveTheTime => {
                self.compose.later = entry == Entry::AddToLater;
                self.close_menu(true, tx);
            }
        }
    }

    /// What a picture taken with a menu open says of it (`menu` in its fixture): the same
    /// control is pressed, and the same page turned.
    pub(crate) fn picture_menu(&mut self, which: &str, tx: &mut Tx) {
        let kind = match which {
            "project" => MenuKind::Project,
            "priority" => MenuKind::Level,
            "later" => MenuKind::Later,
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

    /// Sends what is in the box as a new thread (Send, or ⌘Return). The box empties at once,
    /// the card closes, and a line says where the thread is going; a refusal puts every word
    /// back.
    pub fn send_thread(&mut self, tx: &mut Tx) {
        let Some(message) = self.compose_message() else {
            return;
        };
        // A file that cannot go holds the whole send, and the line under the files says why.
        if !attach::ready(&self.compose.files) {
            return;
        }
        let Some(product) = self.compose_project().filter(|p| rules::practice_refusal(Some(p)).is_none()).map(|p| p.slug.clone()) else {
            return;
        };
        let pick = self.compose_pick();
        let levels = rules::effort_choices(&pick, &self.models);
        // A level is sent only when one was picked that this model takes.
        let effort = self.kept_effort(&pick.engine).filter(|level| rules::effort_picked(Some(level), &levels));
        let harness = rules::harness_fields(&pick, effort.as_deref(), Engines::of(&self.snap).rows().len(), self.models.codex_default.as_deref());
        let later = self.compose.later;
        let payload = rules::compose(&product, &message, rules::sent_priority(self.compose.level), harness, later);
        // With files, they are saved into the project first and the thread's body links them
        // where they landed (`persistAttachments`); a file on this Mac goes as its path, a
        // pasted picture as its bytes. Either write refused puts the words and the files back.
        let files = attach::pending(&self.compose.files);
        let steps = if files.is_empty() {
            vec![write::Step::Write((rules::COMPOSE, payload))]
        } else {
            vec![write::Step::Save { product: product.clone(), files }, write::Step::Linked((rules::COMPOSE, payload))]
        };
        let draft = self.compose.draft();
        // What was done before this and still waits its three seconds is written first.
        self.write_what_waits();
        let on_its_way = self.send_steps(steps, write::Sent::Thread { product: product.clone(), draft });
        if !on_its_way && self.refused.is_some() {
            // No door to send through: the words stay where they are, and the line says so.
            return;
        }
        self.picks.change(|picks| picks.project = Some(product.clone()));
        self.compose.words = TextEdit::new("");
        self.compose.files.clear();
        self.compose.later = false;
        self.compose.menu = None;
        self.compose.on_chip = None;
        self.compose.o_waits = None;
        // The card goes on Send, and the next one opens fresh: the level back on Agent Decides
        // and the model on the agent from Settings (`onSending`, and a new card's draft). The
        // box on the page keeps both for the next thread, as the web window's does.
        if self.compose.card {
            self.compose.card = false;
            self.compose.level = None;
            self.compose.pick = None;
        }
        tx.blur();
        // Alone, on the Inbox, the tab moves to where the thread will land; over Home or a
        // conversation the person stays where they are.
        let covered = self.view != View::Inbox;
        if let Some(tab) = rules::tab_after_sending(window::team_mode(&self.snap), later, Some(self.inbox_tab), covered) {
            self.show_tab(tab);
        }
        self.say(rules::sending_line(&self.project_named(&product), later), false, tx);
        if on_its_way {
            self.compose.not_taken_at = Some(tx.clock() + rules::NOT_ANSWERING_MS / 1000.0);
        }
    }

    /// What the engine said to a new thread. Taken, the thread is in the lists at once and a
    /// line names it, with O to open it and Z to take it back. Refused, every word goes back
    /// and the card opens on them, unless something else is over the page by then: then a line
    /// says where the words are (`onFailed` in `App.tsx`).
    pub(crate) fn thread_landed(&mut self, landed: write::Landed, tx: &mut Tx) {
        let write::Sent::Thread { product, draft } = landed.sent.clone() else {
            return;
        };
        self.compose.not_taken_at = None;
        if let Some(why) = landed.refusal() {
            self.compose.o_waits = None;
            self.compose.put_back(draft);
            let said = rules::not_sent(&why);
            if self.compose.card || self.later.on.is_some() {
                self.say(rules::not_sent_words_kept(&said), false, tx);
            } else {
                self.open_card(tx);
                self.say(said, false, tx);
            }
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
        let line = rules::sent_line(&self.project_named(&product), draft.later, back.as_ref().map(|(tool, at)| (*tool, at.as_deref())));
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

    /// What the engine said to that close. Done, what the box held goes back in it, unless it
    /// holds something newer, and the card opens on it over the list (`shownAfterUndo`);
    /// refused, Z keeps the thread and a line says why.
    pub(crate) fn withdraw_landed(&mut self, landed: write::Landed, tx: &mut Tx) {
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
        let back = self.compose.put_back(draft);
        self.say(rules::said_withdrawn(&title, back), false, tx);
        // The whole point of the press was to add a sentence to what was written: the card
        // opens on it, over the list rather than a conversation.
        if back && !self.compose.card && self.later.on.is_none() {
            if matches!(self.view, View::Thread { .. }) {
                self.leave_thread();
            }
            self.open_card(tx);
        }
    }

    /// What a picture taken just after a send hands over (`sent` in its fixture): the row the
    /// page was answered with and when. The same answer lands here, and the line about it is
    /// as old as the page's was at the shutter.
    pub(crate) fn picture_sent(&mut self, sent: &Value, tx: &mut Tx) {
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

    fn pasted(name: &str) -> attach::Attached {
        attach::Attached::pasted(b"png", (1440, 900), name.to_string())
    }

    #[test]
    fn a_refusal_or_z_puts_every_pick_back_but_never_over_newer_words() {
        let mut box_ = Composer::default();
        let draft = write::Draft { words: "Ship the fix".to_string(), level: Some(Level::High), pick: Some(Pick::new("claude", "sonnet")), later: true, files: vec![pasted("pasted-1.png")] };
        assert!(box_.put_back(draft.clone()));
        assert_eq!(box_.draft(), draft);
        // Something typed since stays, and so do its picks.
        let mut busy = Composer { words: TextEdit::new("A newer thought"), ..Composer::default() };
        assert!(!busy.put_back(draft.clone()));
        assert_eq!((busy.words.text.as_str(), busy.level, busy.later, busy.files.len()), ("A newer thought", None, false, 0));
        // Nothing to give back is not a reason to empty the picks.
        let mut picked = Composer { level: Some(Level::Low), ..Composer::default() };
        assert!(!picked.put_back(write::Draft::default()));
        assert_eq!(picked.level, Some(Level::Low));
    }

    #[test]
    fn files_alone_are_something_to_put_back_and_something_newer_to_keep() {
        // A thread sent with only a picture comes back as that picture.
        let mut box_ = Composer::default();
        let draft = write::Draft { files: vec![pasted("pasted-1.png")], ..write::Draft::default() };
        assert!(box_.put_back(draft.clone()));
        assert_eq!(box_.files, draft.files);
        // A file attached since is newer than what was sent, and stays alone in the box.
        let mut busy = Composer { files: vec![pasted("pasted-2.png")], ..Composer::default() };
        assert!(!busy.put_back(write::Draft { words: "Ship the fix".to_string(), ..draft }));
        assert_eq!((busy.words.text.as_str(), busy.files[0].file.name.as_str()), ("", "pasted-2.png"));
    }

    #[test]
    fn the_caret_hangs_its_menu_from_the_whole_of_send_and_the_rest_from_their_own_control() {
        assert_eq!(MenuKind::Later.anchor(), "compose-send-pair");
        assert_eq!(MenuKind::Later.trigger(), CARET);
        for kind in [MenuKind::Model, MenuKind::Project, MenuKind::Level] {
            assert_eq!(kind.anchor(), kind.trigger());
        }
    }
}
