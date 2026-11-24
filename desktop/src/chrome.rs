//! What every screen stands on: the dotted page, the main card with the sky painted on it, the
//! rail, which opens while the pointer rests on it, and back and forward. Sizes follow the
//! window; the insets are the web app's.

use std::time::Duration;

use rune::{Cursor, Easing, El, Icon, LineCap, Shape, TextCase, TextStyle, Tx, col, el, icon, row, spacer, text};

use crate::rules::{recent, window};
use crate::theme::*;
use crate::{State, View, feel, front, icons};

/// The main card's insets from the window with the rail folded: left of it the rail, above it
/// the title strip. While the rail is open the card starts further right (`card_left`).
pub const CARD: [f32; 4] = [58.0, 40.0, 22.0, 22.0];
/// Where the page begins inside the window: under the card's top strip.
pub const PAGE_TOP: f32 = 104.0;

/// The rail's left edge, and its width folded and open (`--sidebar-w` in
/// `renderer/src/sidebar.css`: folded, as wide as the column each mark keeps when open).
pub const RAIL_LEFT: f32 = 14.0;
pub const FOLDED: f32 = 28.0;
pub const OPEN: f32 = 220.0;
/// Where the card starts beside the open rail (`--workspace-left`: 44 points past the older
/// `--workspace-width` of 202, which the sidebar's sheet leaves standing; folded it is `CARD[0]`).
const CARD_OPEN: f32 = 246.0;

/// Where the card starts now, beside the rail as wide as it is at this frame: the window's
/// padding and the rail's width ease together, on one curve (`rooms.css`).
pub fn card_left(s: &State) -> f32 {
    CARD[0] + (CARD_OPEN - CARD[0]) * (s.rail.width(s.at) - FOLDED) / (OPEN - FOLDED)
}

/// The card's size with the rail folded, which the skies are painted for: the rail opening
/// covers their left edge rather than painting them again (`sky-follows-the-card.ts`).
pub fn folded_card_size(s: &State) -> (f32, f32) {
    ((s.size.0 - CARD[0] - CARD[2]).max(1.0), (s.size.1 - CARD[1] - CARD[3]).max(1.0))
}

/// A value carried from where it was to where it goes as a CSS transition carries it: it holds
/// for `delay` seconds, then moves for `dur` along `curve`. A change before it lands starts the
/// next from wherever it had got to.
#[derive(Clone, Copy)]
struct Fade {
    from: f32,
    to: f32,
    since: f64,
    delay: f64,
    dur: f64,
    curve: Easing,
}

impl Fade {
    const fn still(at: f32) -> Fade {
        Fade { from: at, to: at, since: 0.0, delay: 0.0, dur: 0.0, curve: Easing::LINEAR }
    }
    fn at(&self, now: f64) -> f32 {
        let t = now - self.since - self.delay;
        if t < 0.0 {
            return self.from;
        }
        if self.dur <= 0.0 || t >= self.dur {
            return self.to;
        }
        self.from + (self.to - self.from) * self.curve.eval((t / self.dur) as f32)
    }
    fn toward(&self, to: f32, now: f64, delay: f64, dur: f64, curve: Easing) -> Fade {
        Fade { from: self.at(now), to, since: now, delay, dur, curve }
    }
    /// Still moving at `now`, or landed less than a few frames ago, so the frame that draws it
    /// where it lands is drawn.
    fn moving(&self, now: f64) -> bool {
        now < self.since + self.delay + self.dur + 0.05
    }
}

/// The rail, folded to its marks, or open to the whole sidebar with its words while the pointer
/// rests on it (`railOpen` in `renderer/src/App.tsx`, `onHover` in `WorkspaceNavigation.tsx`).
/// Open, it moves the card over rather than covering it.
pub struct Rail {
    open: bool,
    /// When it opens, 140 ms after the pointer moved onto it, and whether it then opens whole.
    due: Option<(f64, bool)>,
    width: Fade,
    /// The words, the team's name, Inbox's count and the keys beside Search.
    words: Fade,
    /// The dot on Inbox while something waits, which the count stands for when open.
    dot: Fade,
    /// The pointer is on it.
    pointed: bool,
}

impl Default for Rail {
    fn default() -> Rail {
        Rail { open: false, due: None, width: Fade::still(FOLDED), words: Fade::still(0.0), dot: Fade::still(1.0), pointed: false }
    }
}

impl Rail {
    /// The pointer's 140 ms on the rail are up, or it left the rail (`on`). The rules
    /// are `renderer/src/team/rooms/rooms.css`'s: the width eases open over 0.24 s and shut
    /// over 0.2 s; the words come in 0.08 s later over 0.14 s and go over 0.06 s; Inbox's dot
    /// crosses in 0.1 s, its `ease` both ways. Search has no line under it to cross any more
    /// (`.workspace-find`). Under the system's Reduce motion (`still`) none of it moves: it opens
    /// and folds whole, as the same rules' `prefers-reduced-motion` block takes every transition
    /// away.
    fn point(&mut self, on: bool, now: f64, still: bool) {
        if on == self.open {
            return;
        }
        self.open = on;
        let (ease, k) = (Easing::EASE, if still { 0.0 } else { 1.0 });
        if on {
            self.width = self.width.toward(OPEN, now, 0.0, 0.24 * k, Easing::cubic(0.22, 0.61, 0.36, 1.0));
            self.words = self.words.toward(1.0, now, 0.08 * k, 0.14 * k, ease);
            self.dot = self.dot.toward(0.0, now, 0.0, 0.1 * k, ease);
        } else {
            self.width = self.width.toward(FOLDED, now, 0.0, 0.2 * k, Easing::cubic(0.4, 0.0, 0.2, 1.0));
            self.words = self.words.toward(0.0, now, 0.0, 0.06 * k, ease);
            self.dot = self.dot.toward(1.0, now, 0.0, 0.1 * k, ease);
        }
    }
    /// Whether it is open, or opening: the web window's rail stops being folded the moment it
    /// starts to widen (`railOpen`).
    pub fn is_open(&self) -> bool {
        self.open
    }
    /// How wide it is at `now`.
    pub fn width(&self, now: f64) -> f32 {
        self.width.at(now)
    }
    /// Whether it is opening or folding at `now`, when every frame has to be drawn.
    pub fn moving(&self, now: f64) -> bool {
        [self.width, self.words, self.dot].iter().any(|f| f.moving(now))
    }
}

/// The poll's part: the rail opens once its 140 ms are up, and while it opens or folds each
/// frame is drawn and asks for the next.
pub fn rail_moves(s: &mut State, at: f64, tx: &mut Tx) -> bool {
    if let Some((due, still)) = s.rail.due.filter(|&(due, _)| at >= due) {
        s.rail.due = None;
        s.rail.point(true, due, still);
    }
    // With the sidebar's shrinking turned off in Settings the rail stands open, and folds again
    // once it is turned back on (`useSidebarShrinks`).
    let shrinks = s.picks.get().rail_shrinks.unwrap_or(true);
    if !shrinks && !s.rail.open {
        s.rail.point(true, at, true);
    } else if shrinks && s.rail.open && s.rail.due.is_none() && !s.rail.pointed {
        s.rail.point(false, at, true);
    }
    let moving = s.rail.moving(at);
    if moving {
        tx.wake_after(Duration::ZERO);
    }
    moving
}

/// The same for the screen on show. The strip over a conversation is the Inbox's 64 points
/// too (`crate::thread::STRIP`), so every page starts under the same line.
pub fn page_top(view: &View) -> f32 {
    match view {
        View::Thread { .. } => CARD[1] + crate::thread::STRIP,
        _ => PAGE_TOP,
    }
}

/// The page under everything: the deep black `--bg` and a dot every 32 points (`--skin-image`
/// under a dark sky, in `renderer/src/home/home.css`: a disc of 1.15 points at a twentieth of
/// white in the middle of each 32-point square, and no warm glow any more).
///
/// The dots are one drawing on this one element, each a round stroke too short to have a length.
/// They were an element apiece, 1,900 of them under a maximised window, and Rune walks every
/// element on every frame it draws: while a line was working they were most of what it walked.
pub fn backdrop(w: f32, h: f32) -> El {
    let mut dots = Shape::new();
    let mut y = 16.0;
    while y < h {
        let mut x = 16.0;
        while x < w {
            // A stroke needs two points apart to have a direction: a hundredth of a point.
            dots = dots.line([x - 0.005, y], [x + 0.005, y], 2.3, LineCap::Round, white(0.05));
            x += 32.0;
        }
        y += 32.0;
    }
    el().absolute(0.0, 0.0).size(w, h).fill(page()).shape(dots)
}

/// How long the card holds still before a sky stretched by a new size is painted again
/// (`STILL_MS` in `renderer/src/home/sky-follows-the-card.ts`).
pub const STILL: Duration = Duration::from_millis(150);

/// The box a painted sky is drawn in: the card with the rail folded, as wide and tall as it is
/// now. A sky painted for another size is stretched to it until the card has held still and the
/// new one lands, as the web window stretches its canvas (`followTheCard`).
pub fn sky_box(s: &State) -> (f32, f32) {
    folded_card_size(s)
}

/// How many times smaller each way a soft sky is kept than its sky (`sky::soft`): Home's glass
/// blurs by 20 points, which leaves nothing finer than an eighth of the painting holds, and
/// the strip's buttons by 6, a quarter.
pub const SOFT_HOME: u32 = 8;
pub const SOFT_STRIP: u32 = 4;

/// Where a painted sky stands in the window: on the card's right edge, from the card's top.
pub fn sky_in_the_window(s: &State) -> [f32; 4] {
    let (w, h) = sky_box(s);
    [s.size.0 - CARD[2] - w, CARD[1], w, h]
}

/// What a pane of glass keyed `key` has behind it, drawn instead of blurred: `soft`, a sky
/// blurred when it was painted, standing where that sky stands in the window (`at`) and cut
/// to the pane's corners. The pane lays its wash over this and asks Rune for no blur. `painted`
/// is the sky's own size in pixels, `shrink` how much smaller the soft one was kept, and `edge`
/// the width of the pane's own edge, which takes room in its layout.
///
/// Rune blurs what is behind an element from copies of the whole window, some 180 MB of
/// textures at 1900 by 1040 points for as long as one such element is on screen. What Home's
/// glass and the strip's buttons have behind them is a sky that only changes when it is
/// painted, so it is blurred then, once, off the main thread, into a picture of half a megabyte.
///
/// None until the pane has been laid out, since its place is what says which part of the sky
/// shows through it: that one build has the pane's wash alone, and is never drawn, because the
/// page is built again with the place before its first frame.
pub fn sky_through(key: &str, soft: &rune::Pixels, at: [f32; 4], painted: (u32, u32), shrink: u32, radius: f32, edge: f32) -> Option<El> {
    let [x, y, _, _] = rune::memo::window_layout(key)?;
    let sky = soft_sky_from((x, y), soft, at, painted, shrink)?;
    // The pane's children are laid out inside its edge, and the glass runs under the edge too.
    Some(el().pin(Some(-edge), Some(-edge), Some(-edge), Some(-edge)).radius(radius).clip_rounded(radius).inert().a11y_hidden().child(sky))
}

/// A soft sky as it lies for something whose corner is at `corner` in the window: the picture
/// alone, where the sky stands (`at`), for whatever cuts it to its own shape. None while the
/// sky has not been painted.
pub fn soft_sky_from(corner: (f32, f32), soft: &rune::Pixels, at: [f32; 4], painted: (u32, u32), shrink: u32) -> Option<El> {
    let (soft_w, soft_h) = soft.size();
    if soft_w == 0 || painted.0 == 0 || painted.1 == 0 {
        return None;
    }
    // The soft sky's last block may stand for less than a whole one: it is drawn as far past
    // the sky's edge as that block reaches, so every block lands where it was taken from.
    let wide = at[2] * (soft_w * shrink) as f32 / painted.0 as f32;
    let tall = at[3] * (soft_h * shrink) as f32 / painted.1 as f32;
    Some(el().absolute(at[0] - corner.0, at[1] - corner.1).size(wide, tall).pixels(soft))
}

/// The main card. The sky fills the card as it is with the rail folded, stretched while a
/// window is being resized and painted again once it holds still. It stands on the card's right
/// edge, so the rail opening covers its left edge.
pub fn card(s: &State) -> El {
    // `.workspace-layout::before` under a dark sky: the deep black `--skin-pane`, whole, in a
    // hairline of `--skin-pane-line`, on `--shadow-card`.
    let card = el()
        .pin(Some(card_left(s)), Some(CARD[1]), Some(CARD[2]), Some(CARD[3]))
        .radius(PANE_CORNER)
        .fill(surface())
        .box_shadow(shadow(0.0, 40.0, 120.0, 0.0, rgba(0, 0, 0, 0.7), false))
        .box_shadow(shadow(0.0, 0.0, 0.0, 1.0, edge(), true));
    let whole = || el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).radius(PANE_CORNER);
    let (card_w, _) = s.card_size();
    let (sky_w, sky_h) = sky_box(s);
    match s.view {
        // Home's sky covers the whole card, on the sky's own ground, under its faint ring
        // (`.home-sky::after`).
        View::Home => card
            .child(whole().fill(surface()).clip_rounded(PANE_CORNER).child(el().absolute(card_w - sky_w, 0.0).size(sky_w, sky_h).pixels(&s.sky_home)))
            .child(whole().border(1.0, white(0.07))),
        // The Inbox's sky stands behind its head alone, so under it the card shows its own
        // ground and edge. The Files place wears it too (`skyOnPlace` in `renderer/src/App.tsx`).
        View::Inbox | View::Files { .. } => card.child(crate::inbox::sky(s)),
        // A conversation has no sky: the card is only its ground, inside its edge.
        View::Thread { .. } => card,
    }
}

/// The nine-dot mark: eight in the text colour at 38%, the top right one orange.
fn app_mark() -> El {
    let mut dots = Vec::new();
    for (j, cy) in [21.0f32, 50.0, 79.0].into_iter().enumerate() {
        for (i, cx) in [21.0f32, 50.0, 79.0].into_iter().enumerate() {
            let k = 12.0 / 80.0;
            let r = 10.5 * k;
            let c = if i == 2 && j == 0 { accent() } else { rgba(239, 235, 231, 0.38) };
            dots.push(el().absolute((cx - 10.0) * k - r, (cy - 10.0) * k - r).size(2.0 * r, 2.0 * r).radius(r).fill(c));
        }
    }
    el().size(12.0, 12.0).children(dots)
}

/// What a row of the sidebar takes under the pointer, and wears while its page is the one on
/// screen (`--sidebar-fill`: the text's colour at six hundredths).
fn rail_fill() -> rune::Color {
    ink_at(0.06)
}

/// A row of the sidebar (`.workspace-tab`, `.workspace-find button`, `.workspace-utilities >
/// button` and `.ws-recent-row` in `renderer/src/sidebar.css`): thirty points tall, its mark in
/// a column of sixteen six points in, its word eight past the mark, cut off by the row's edge
/// as the sidebar widens. None takes the pointing hand (`cursor: default`).
fn rail_row(wide: f32) -> El {
    row().size(wide, 30.0).radius(6.0).items_center().px(6.0).gap(8.0).clip_rounded(6.0)
}

/// A row's mark: drawn 18 points across in its 16-point column (`margin: -1px`).
fn rail_mark(glyph: &'static Icon, tone: rune::Color) -> El {
    icon(glyph, 18.0).color(tone).ml(-1.0).mr(-1.0)
}

/// A row's word, in the sidebar's own type (14 points at 500 on a line of 20).
fn rail_word(word: &str, tone: rune::Color) -> El {
    pen(text(word).font(TextStyle::new(14.0, 500.0).lh(20.0)).color(tone), 14.0)
}

/// One of the sidebar's places or the foot's buttons: its mark, and the word beside it that
/// shows while the sidebar is open. The page on screen is lit, its word in the text's colour
/// and its mark a step brighter; any other takes the same under the pointer
/// (`.workspace-tab.active`, `:hover`).
fn rail_button(glyph: &'static Icon, word: &str, active: bool, wide: f32, words: f32) -> El {
    let b = rail_row(wide);
    let b = if active { b.fill(rail_fill()) } else { b.hover_group().hover(rail_fill()) };
    let (mark, tone) = if active { (dim(), ink()) } else { (muted(), dim()) };
    b.child(rail_mark(glyph, mark).group_color(dim())).child(rail_word(word, tone).group_color(ink()).opacity(words))
}

/// Restart to update, which keeps the older row's shape (`.sb-update-row`): 36 points tall,
/// its mark ten points in and its word eleven past it, in the faint ink.
fn update_row(wide: f32, words: f32) -> El {
    row().size(wide, 36.0)
        .radius(6.0)
        .items_center()
        .pl(10.0)
        .pr(10.0)
        .gap(11.0)
        .clip_rounded(6.0)
        .cursor(Cursor::Pointer)
        .hover_group()
        .hover(feel::wash())
        .child(icon(&icons::RESTART, 18.0).color(muted()).group_color(ink()))
        .child(pen(text("Restart to update").font(TextStyle::new(13.0, 500.0).lh(20.0)).color(muted()).group_color(ink()), 13.0).opacity(words))
}

/// The person's own face on their row: two letters on a light disc (`.th-me .tm-av`).
fn my_face(letters: &str) -> El {
    row()
        .size(20.0, 20.0)
        .radius(10.0)
        .center()
        .fill(dim())
        .child(text(letters).font(TextStyle::new(9.0, 500.0).mono().lh(18.0).tracking(0.18)).text_case(TextCase::Uppercase).color(page()))
}

/// A section's name (`SectionHead`, `.ws-sec`): a press folds its rows away and the next
/// brings them back, kept for the next time the window opens. It shows only while the sidebar
/// is open, and folded it keeps its height, so no mark under it moves when the sidebar opens.
fn section_head(s: &State, id: &'static str, word: &str, words: f32) -> El {
    let folded = s.picks.get().rail_folds.iter().any(|f| f == id);
    let arrow = icon(&icons::RAIL_FOLD, 7.0).color(muted()).group_color(ink()).opacity(0.75);
    let arrow = if folded { arrow.rotate(-std::f32::consts::FRAC_PI_2) } else { arrow };
    let head = row()
        .h(24.0)
        .px(6.0)
        .gap(4.0)
        .items_center()
        .radius(6.0)
        .opacity(words)
        .hover_group()
        .hover(rail_fill())
        .child(pen(text(word).font(TextStyle::new(12.0, 500.0).lh(14.0)).color(muted()).group_color(ink()), 12.0))
        .child(arrow);
    // Folded to its marks the sidebar's sections take no press (`pointer-events: none`).
    let head = if s.rail.open { head.on_press(move |s: &mut State| s.picks.change(|p| p.rail_folds = recent::toggle_fold(&p.rail_folds, id))) } else { head };
    // As wide as its words, not the sidebar (`align-self: flex-start`), sixteen under what is over it.
    row().mt(16.0).child(head)
}

/// A thread in Recent (`.ws-recent-row`): whose it is, its name, and a dot while it waits on
/// the person and is not the one open. A press opens it, which reads it (`onOpen`).
fn recent_row(s: &State, item: &crate::data::Item, wide: f32) -> El {
    let title = crate::rules::title::row_title(item);
    let harness = crate::rules::engine::harness_mark(crate::rules::engine::row_engine(item, &s.snap));
    let open_here = matches!(&s.view, View::Thread { id, product } if *id == item.id && *product == item.product);
    let waits = !open_here && s.lists.needs.iter().any(|r| r.id == item.id && r.product == item.product);
    let (id, product) = (item.id.clone(), item.product.clone());
    // An 18-point face in the marks' 16-point column, so every name starts on one line.
    let mark = el().size(16.0, 16.0).child(crate::inbox::harness_mark(harness).absolute(-1.0, -1.0));
    let name = el().grow(1.0).min_w(0.0).child(pen(text(title.as_str()).font(TextStyle::new(14.0, 500.0).lh(20.0)).color(dim()).group_color(ink()).truncate(), 14.0));
    let mut r = rail_row(wide).hover_group().hover(rail_fill()).child(mark).child(name);
    if waits {
        r = r.child(el().size(6.0, 6.0).radius(3.0).fill(accent()).label("Waiting on you"));
    }
    r.help(title).on_press(move |s: &mut State| s.open_thread(&id, &product))
}

/// The sidebar (`WorkspaceNavigation.tsx`, drawn as `renderer/src/sidebar.css` draws it): the
/// team's mark and name with the pencil that starts a thread, Search as a field that shows its
/// keys, Home and the Inbox, Workspace with Files under it, Recent with the threads last
/// opened, and at its foot Restart to update while a new version waits and the person's own
/// row, with what is left of their limit as a small bar on it. Folded it is the marks alone;
/// while the pointer rests on it, it opens to the whole sidebar (`Rail`). `still`: the system
/// asks for reduced motion.
///
/// ONLY WHAT WORKS HERE. The web window's sidebar also has Demands, Chats, Connections, Record
/// meeting and Feedback. Drawn here they were buttons that went nowhere, which is worse than no
/// button, so they are left out until this window has their pages, and what is under each
/// stands that much higher. The person's own row opens Settings, where the web window's opens
/// their account menu (which has Settings in it); with nobody signed in Settings is a row of
/// its own, as it is there.
pub fn rail(s: &State, still: bool) -> El {
    let now = s.at;
    let (wide, words, dot) = (s.rail.width(now), s.rail.words.at(now), s.rail.dot.at(now));
    // Folded, a button's title names it; open, its word does (`title={collapsed ? … : undefined}`).
    let folded = !s.rail.open;
    let button = |glyph: &'static Icon, word: &str, active: bool| {
        let b = rail_button(glyph, word, active, wide, words);
        if folded { b.help(word) } else { b }
    };
    let on_home = s.view == View::Home;
    let on_inbox = matches!(s.view, View::Inbox | View::Thread { .. });
    let waiting = s.lists.needs.len();
    // Inbox: the number that waits, open (`.workspace-running`: small figures in the dim ink,
    // at the row's end), and folded a dot on its mark standing for it (`.has-count::after`).
    let mut inbox = rail_button(&icons::RAIL_INBOX, "Inbox", on_inbox, wide, words);
    if waiting > 0 {
        let count = text(waiting.to_string()).font(mono(11.5, 16.0).tracking(0.23).features("'tnum' 1")).color(dim());
        inbox = inbox.child(spacer()).child(row().min_w(20.0).justify_end().opacity(words).child(count)).help(format!("Inbox · {waiting} {} waiting", if waiting == 1 { "thread" } else { "threads" }));
    } else if folded {
        inbox = inbox.help("Inbox");
    }
    let mut inbox = el().child(inbox.on_press(|s: &mut State| s.go(View::Inbox)));
    if waiting > 0 {
        inbox = inbox.child(el().absolute(18.0, 4.0).size(5.0, 5.0).radius(2.5).fill(accent()).opacity(dot));
    }
    let tabs = col().mt(12.0).gap(2.0).child(button(&icons::RAIL_HOME, "Home", on_home).on_press(|s: &mut State| s.go(View::Home))).child(inbox);

    // The team's mark, its name beside it while the sidebar is open (`.th-team`), and at the
    // row's end the pencil that starts a thread (`.ws-new`), which the sidebar's edge cuts off
    // while it is folded.
    let name = window::team_name(&s.snap);
    let mut team = row()
        .size(wide, 28.0)
        .mt(8.0)
        .pl(4.0)
        .gap(6.0)
        .items_center()
        .clip_rounded(0.0)
        .child(row().size(20.0, 20.0).center().child(app_mark()))
        .child(el().grow(1.0).min_w(0.0).opacity(words).child(pen(text(name).font(TextStyle::new(14.0, 600.0).lh(20.0).tracking(-0.18)).color(ink()).truncate(), 14.0)));
    if words > 0.0 {
        let pencil = row()
            .size(28.0, 28.0)
            .radius(6.0)
            .center()
            .opacity(words)
            .hover_group()
            .hover(rail_fill())
            .child(icon(&icons::RAIL_NEW, 16.0).color(muted()).group_color(ink()))
            .label("New thread")
            .help("New thread (N)");
        team = team.child(if s.rail.open { pencil.on_press_tx(|s: &mut State, tx| s.open_card(tx)) } else { pencil });
    }
    let team = if folded { team } else { team.help(name) };
    // Search, over the places (`.workspace-find`): a field while the sidebar is open, on the
    // card's own black inside a rule, with its glass, its word and its keys, all in the faint
    // ink; folded, the glass alone on no ground. Under the pointer its word and keys brighten a
    // step and the glass does not (`.workspace-find button:hover` names the button's colour, and
    // the glass keeps `--text-faint` from the rule for every mark). It opens the command bar,
    // as ⌘K does.
    let search = rail_row(wide)
        .mt(12.0)
        .fill(surface_at(words))
        .box_shadow(shadow(0.0, 0.0, 0.0, 1.0, rule_at(words), true))
        .hover_group()
        .child(icon(&icons::RAIL_SEARCH, 20.0).color(muted()).ml(-2.0).mr(-2.0))
        .child(pen(text("Search").font(TextStyle::new(14.0, 400.0).lh(20.0)).color(muted()).group_color(dim()), 14.0).opacity(words))
        .child(spacer())
        .child(pen(text("⌘K").font(TextStyle::new(12.0, 400.0).lh(20.0)).color(muted()).group_color(dim()), 12.0).opacity(words))
        .help("Search · ⌘K")
        .on_press_tx(|s: &mut State, tx| s.open_palette(tx));

    // Workspace: Files, the place of every file the threads made, which opens on All files
    // (`openFiles(null)`). Its name folds it.
    let on_files = matches!(s.view, View::Files { .. });
    let folds = &s.picks.get().rail_folds;
    let workspace_open = !folds.iter().any(|f| f == recent::WORKSPACE);
    let places = workspace_open.then(|| col().gap(2.0).child(button(&icons::RAIL_FILES, "Files", on_files).on_press(|s: &mut State| s.open_files_place(None))));

    // Recent: the threads last opened, newest first, under their own name. Not drawn at all
    // while the sidebar is folded, and the first to give way in a short window (`.ws-recent`).
    let direct = |slug: &str| s.snap.products.iter().any(|p| p.slug == slug && crate::rules::compose::is_direct(p));
    let opened = recent::threads(s.snap.items.iter().filter(|i| !direct(&i.product)), &s.local.seen_at);
    let recents = (words > 0.0 && !opened.is_empty()).then(|| {
        let mut block = col().shrink().min_h(0.0).clip_rounded(0.0).opacity(words).child(section_head(s, recent::RECENT, "Recent", words));
        if !folds.iter().any(|f| f == recent::RECENT) {
            block = block.child(col().gap(2.0).children(opened.iter().map(|i| recent_row(s, i, wide))));
        }
        block
    });

    // The foot (`.workspace-bottom`): Restart to update while a new version waits, then the
    // person's own row; with nobody signed in, Settings is the last row instead. Their titles
    // stand whether the sidebar is open or not.
    let me = window::my_letters(&s.snap);
    let reading = crate::rules::usage::Usage::of(&s.snap, s.now);
    let mut utilities = col().gap(2.0);
    if window::update_waits(&s.snap) {
        // Restart to update stands first, with the room its card rises from (`.sb-update-slot`).
        // The app quits onto the new version and comes back, and this window finds its new door.
        let restart = update_row(wide, words).help("Restart to update").on_press_tx(|s: &mut State, tx| s.ask_the_app(front::INSTALL_UPDATE, "Mercury could not restart onto the new version. Try again in a moment.", tx));
        utilities = utilities.child(el().pb(9.0).child(restart));
    }
    if me.is_none() {
        utilities = utilities.child(rail_button(&icons::RAIL_SETTINGS, "Settings", false, wide, words).help("Settings").on_press_tx(open_settings));
    }
    let mut foot = col().pt(16.0).child(utilities);
    if let Some(letters) = &me {
        // Their face and their name (`.th-me-btn`; the email under it is no longer drawn). The
        // web window opens their account menu on it, whose rows are what they are up to, Invite
        // people, Settings and Sign out; here, with no menu, it goes straight to Settings.
        let (name, _) = window::my_names(&s.snap).unwrap_or_default();
        let mine = row()
            .size(wide, 32.0)
            .mt(2.0)
            .pl(4.0)
            .pr(if reading.is_some() { 48.0 } else { 6.0 })
            .gap(6.0)
            .items_center()
            .radius(6.0)
            .clip_rounded(6.0)
            .hover(rail_fill())
            .child(my_face(letters))
            .child(el().grow(1.0).min_w(0.0).opacity(words).child(pen(text(name).font(TextStyle::new(14.0, 500.0).lh(20.0)).color(ink()).truncate(), 14.0)));
        foot = foot.child(mine.help("Settings").on_press_tx(open_settings));
    }
    if let Some(reading) = reading.filter(|_| words > 0.0) {
        // What is left of the limit: a small bar at the end of the last row, the person's own
        // when they are signed in (`.workspace-usage`, `.usage-pill`), as full as the headline
        // window is spent and in the text's colour once that is close. It says the whole of it
        // to whatever reads the screen aloud, since the corner has no words. Resting on it opens
        // the card of every agent's limits over it, and a press opens that agent's page in
        // Settings (`usage.rs`). Not drawn while the sidebar is folded.
        let used = reading.used(s.now);
        let close = used.is_some_and(|u| u >= crate::rules::usage::CLOSE);
        let spent = 32.0 * (used.unwrap_or(0.0).clamp(0.0, 100.0) / 100.0) as f32;
        let meter = el().size(32.0, 4.0).radius(2.0).clip_rounded(2.0).fill(ink_at(0.13)).child(el().size(spent, 4.0).radius(2.0).fill(if close { ink() } else { dim() }));
        let pill = crate::usage::answering(row().size(32.0, 20.0).items_center().child(meter).label(reading.sentence(s.now)), &reading.last.engine);
        foot = foot.child(row().pin(None, None, Some(6.0), Some(0.0)).h(crate::usage::last_row(s)).items_center().opacity(words).child(pill));
    }

    let mut rail = col()
        .pin(Some(RAIL_LEFT), Some(50.0), None, Some(22.0))
        .w(wide)
        // It opens 140 ms after the pointer moves onto it and folds the moment it leaves
        // (`hoverAt` in `WorkspaceNavigation.tsx`): a pointer passing over on its way somewhere
        // else is gone before then, and one that did not move opens nothing, so a rail uncovered
        // under a pointer standing still stays folded (`pointerMoved` in `sidebar-shrinks.ts`).
        .on_pointer_move_local(move |s: &mut State, _, tx| {
            if s.rail.open || s.rail.due.is_some() {
                tx.live_only();
                return;
            }
            s.rail.due = Some((tx.clock() + 0.14, still));
            tx.wake_after(Duration::from_millis(140));
        })
        .on_hover_change(move |s: &mut State, inside, tx| {
            s.rail.pointed = inside;
            if inside || !s.picks.get().rail_shrinks.unwrap_or(true) {
                tx.live_only();
                return;
            }
            s.rail.due = None;
            s.rail.point(false, tx.clock(), still);
            tx.wake_after(Duration::ZERO);
        })
        .child(team)
        .child(search)
        .child(tabs)
        .child(section_head(s, recent::WORKSPACE, "Workspace", words));
    if let Some(places) = places {
        rail = rail.child(places);
    }
    if let Some(recents) = recents {
        rail = rail.child(recents);
    }
    // The foot stands on the sidebar's end, whatever is over it (`.workspace-bottom`, `flex: none`).
    rail.child(spacer()).child(foot)
}

/// Settings, from the rail: the screen over everything, on General, as the web window's cog
/// opens it (`settings.rs`).
fn open_settings(s: &mut State, tx: &mut Tx) {
    crate::settings::open(s, crate::rules::settings::PageId::General, tx);
}

/// The band along the window's top that moves the window, as the title bar it stands in for
/// does (`.workspace-layout::after`, 40 points of `-webkit-app-region: drag`), and a double
/// press there zooms it as the system says. It stands over everything, the New thread card's
/// and the Later picker's veils included, since the web window's band is not cut by them; it
/// is not there while the command bar is open (`:has(> .palette-backdrop)::after { no-drag }`).
/// Above it the system's own title bar band moves the window whatever is open, as the band
/// `hiddenInset` keeps does in the web window (`.find-bar` in `renderer/src/styles.css`). The
/// rail is never part of it (`.workspace-navigation { no-drag }`), nor is anything else in the
/// window but the card's top strip on the Inbox and over a conversation: Home's strip is
/// hidden (`.app.home-shown > .topbar { visibility: hidden }`), which takes it out.
pub fn drag_band(s: &State) -> Option<El> {
    // A file open big has its own bar to move the window by (`.av-top`).
    (!s.palette.open && s.viewer.open.is_none()).then(|| el().absolute(0.0, 0.0).size(s.size.0, CARD[1]).window_drag())
}

/// One quiet line at the card's foot when the engine cannot be read, so an empty inbox is never
/// mistaken for a finished one; and when an agent asks for a yes this window cannot yet give.
pub fn notice(s: &State) -> Option<El> {
    // A file open big covers the window's notices, as the web window's view does.
    if s.viewer.open.is_some() {
        return None;
    }
    // Something just sent that was not taken comes first: it is about what the person did.
    let said = match (&s.refused, &s.source, &s.error) {
        (Some(why), _, _) => why.as_str(),
        (_, crate::source::Source::Nothing, _) => "Mercury is not running, so there is nothing to show yet.",
        (_, _, Some(_)) => "Mercury is not answering. This will catch up by itself when it does.",
        // The card that asks is the web window's; here it is only said that one is up.
        _ => match window::approvals_waiting(&s.snap) {
            0 => return None,
            1 => "An agent is waiting for your yes. Answer it in Mercury's own window.",
            _ => "Agents are waiting for your yes. Answer them in Mercury's own window.",
        },
    };
    Some(
        row().pin(Some(card_left(s)), None, Some(CARD[2]), Some(CARD[3] + 14.0))
            .justify_center()
            .child(row().h(28.0).px(12.0).items_center().radius(6.0).fill(rgba(17, 16, 15, 0.7)).border(1.0, edge()).child(label(said, 12.5, 400.0, dim()))),
    )
}

/// A key and its word beside a line (`.toast-key`).
fn key_cap(key: &str, word: &str) -> El {
    row().items_center()
        .gap(5.0)
        .child(row().h(17.0).px(4.0).items_center().border(1.0, rule()).border_layout().child(text(key).font(TextStyle::new(10.5, 600.0).lh(15.0)).color(muted())))
        .child(pen(text(word).font(TextStyle::new(12.0, 400.0).lh(15.6)).color(muted()), 12.0))
}

/// Z and its word, while Z can still take the thing back (`.toast-undo`). A press on it is Z.
fn undo_key() -> El {
    key_cap("Z", "Undo").cursor(rune::Cursor::Pointer).on_press_tx(|s: &mut State, tx| {
        s.toast = None;
        s.undo(tx);
    })
}

/// The keys a line about one thread offers: O while that thread is not the one open, and Z
/// while it can be taken back (`.toast-keys`).
fn toast_keys(s: &State, toast: &crate::write::Toast) -> Option<El> {
    let opens = toast.goes.as_ref().is_some_and(|(id, product)| !matches!(&s.view, View::Thread { id: open, product: of } if open == id && of == product));
    if !opens && !toast.undo {
        return None;
    }
    let mut keys = row().items_center().gap(12.0);
    if opens {
        keys = keys.child(key_cap("O", "Open"));
    }
    if toast.undo {
        keys = keys.child(undo_key());
    }
    Some(keys)
}

/// The bar along the foot of a line that names a thread (`.toast-clock`): the time left to
/// open it, running out to the left. It is cut to the card's rounded inside (`overflow:
/// hidden`) by a layer of its own: a rounded clip on the card would take the card's shadow too.
fn toast_clock(toast: &crate::write::Toast, at: f64, width: f32) -> El {
    let left = ((toast.until - at) / (toast.until - toast.since).max(0.001)).clamp(0.0, 1.0) as f32;
    let bar = el().pin(Some(0.0), None, None, Some(0.0)).size(width * left, 2.0).fill(accent()).opacity(0.7);
    el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).clip_rounded(9.0).child(bar)
}

/// The line about what was just done, as a conversation's head strip carries it
/// (`.toast.toast-in-bar`): framed, at most 520 points, cut short with dots.
pub fn toast_in_bar(s: &State) -> Option<El> {
    let toast = s.toast.as_ref()?;
    // A send that moved the person to another thread is said over that thread's reply box.
    if crate::answer::sent_line(s).is_some() {
        return None;
    }
    // A line about one thread names it, then says what went where, its keys after, and the time
    // left to open it running out along its foot (`.toast-goes.toast-in-bar`).
    if let (Some(_), Some(title)) = (&toast.goes, s.toast_title()) {
        let wide = rune::memo::layout_size("toast-in-bar").map_or(0.0, |[w, _]| w);
        // Three columns, twelve apart, in at most 520 points less the frame and its padding:
        // the keys at their width, and the title and the line sharing what is left equally, each
        // no wider than its words (`grid-template-columns: minmax(0, auto) minmax(0, auto) auto`).
        let font = TextStyle::new(12.5, 400.0);
        let measure = |s: &str, style: TextStyle| rune::text::measure(s, &[], style, None).0;
        let opens = toast.goes.as_ref().is_some_and(|(id, product)| !matches!(&s.view, View::Thread { id: open, product: of } if open == id && of == product));
        let cap = |k: &str| 10.0 + measure(k, TextStyle::new(10.5, 600.0));
        let key = |k: &str, w: &str| cap(k) + 5.0 + measure(w, TextStyle::new(12.0, 400.0));
        let keys = [opens.then(|| key("O", "Open")), toast.undo.then(|| key("Z", "Undo"))].into_iter().flatten().collect::<Vec<f32>>();
        let keys_w = keys.iter().sum::<f32>() + 12.0 * keys.len().saturating_sub(1) as f32;
        let room = (520.0 - 22.0 - 24.0 - keys_w).max(0.0);
        let (tw, lw) = (measure(&title, font), measure(&toast.line, font));
        let (tw, lw) = if tw + lw <= room {
            (tw, lw)
        } else if tw < room / 2.0 {
            (tw, room - tw)
        } else if lw < room / 2.0 {
            (room - lw, lw)
        } else {
            (room / 2.0, room / 2.0)
        };
        let mut line = row()
            .key("toast-in-bar")
            .max_w(520.0)
            .min_w(0.0)
            .items_center()
            .gap(12.0)
            .pt(4.0)
            .pr(8.0)
            .pb(5.0)
            .pl(12.0)
            .radius(10.0)
            .border(1.0, edge())
            .border_layout()
            .cursor(rune::Cursor::Pointer)
            .on_press(|s: &mut State| {
                s.open_what_the_line_names();
            })
            .child(row().w(tw).child(pen(text(title).font(TextStyle::new(12.5, 400.0).lh(17.5)).color(ink()).truncate(), 12.5)))
            .child(row().w(lw).child(pen(text(toast.line.as_str()).font(TextStyle::new(12.5, 400.0).lh(16.25)).color(muted()).truncate(), 12.5)))
            // The keys' column stands, and its gap with it, when it holds nothing.
            .child(toast_keys(s, toast).unwrap_or_else(el));
        if wide > 0.0 {
            line = line.child(toast_clock(toast, s.at, wide - 2.0));
        }
        return Some(line);
    }
    let mut line = row()
        .max_w(520.0)
        .min_w(0.0)
        .items_center()
        .gap(12.0)
        .pt(4.0)
        .pr(8.0)
        .pb(5.0)
        .pl(12.0)
        .radius(10.0)
        .border(1.0, edge())
        .border_layout()
        .child(row().min_w(0.0).child(pen(text(toast.line.as_str()).font(TextStyle::new(13.5, 400.0).lh(17.55)).color(ink()).truncate(), 13.5)));
    if toast.undo {
        line = line.child(undo_key());
    }
    Some(line)
}

/// The same line over a list: a card in the window's corner (`.toast`), its words on two
/// lines if they need them.
pub fn toast_card(s: &State) -> Option<El> {
    if matches!(s.view, View::Thread { .. }) {
        return None;
    }
    toast_corner(s)
}

/// The same card in the window's corner, whatever page is under it: over Settings too.
pub fn toast_corner(s: &State) -> Option<El> {
    let toast = s.toast.as_ref()?;
    if toast.goes.is_some() {
        return Some(toast_about(s, toast));
    }
    let room = if toast.undo { 263.95 } else { 326.0 };
    // As wide as its words, up to 360 points (`.toast-plain`).
    let mut card = row()
        .pin(None, None, Some(30.0), Some(30.0))
        .max_w(360.0)
        .items_center()
        .gap(12.0)
        .py(11.0)
        .px(16.0)
        .radius(10.0)
        .fill(rgb(54, 52, 49))
        .border(1.0, edge())
        .border_layout()
        .box_shadow(shadow(0.0, 24.0, 70.0, 0.0, rgba(0, 0, 0, 0.55), false))
        // The words take at most what the key beside them leaves of the card's 326 points inside.
        .child(el().max_w(room).child(pen(text(toast.line.as_str()).font(TextStyle::new(13.5, 400.0).lh(17.55)).color(ink()).wrap().max_w(room), 13.5)));
    if toast.undo {
        card = card.child(undo_key());
    }
    Some(card)
}

/// The line about one thread (`.toast.toast-goes`): the thread's name over where it went, the
/// keys at the right, and the time left to open it along the foot. A press on it opens the
/// thread. Before the lists have the thread there is no name to draw, and the card is the
/// plain one with its keys.
fn toast_about(s: &State, toast: &crate::write::Toast) -> El {
    let keys = toast_keys(s, toast);
    let card = |e: El| {
        e.pin(None, None, Some(30.0), Some(30.0))
            .radius(10.0)
            .fill(rgb(54, 52, 49))
            .border(1.0, edge())
            .border_layout()
            .box_shadow(shadow(0.0, 24.0, 70.0, 0.0, rgba(0, 0, 0, 0.55), false))
            .cursor(rune::Cursor::Pointer)
            .on_press(|s: &mut State| {
                s.open_what_the_line_names();
            })
    };
    let at = s.at;
    let Some(title) = s.toast_title() else {
        let mut plain = card(row()).max_w(360.0).items_center().gap(12.0).py(11.0).px(16.0).child(pen(text(toast.line.as_str()).font(TextStyle::new(13.5, 400.0).lh(17.55)).color(ink()), 13.5));
        if let Some(keys) = keys {
            plain = plain.child(keys);
        }
        return plain.child(toast_clock(toast, at, 358.0));
    };
    let mut second = row().items_center().gap(16.0).child(row().grow(1.0).min_w(0.0).child(pen(text(toast.line.as_str()).font(TextStyle::new(12.5, 400.0).lh(16.25)).color(muted()).truncate(), 12.5)));
    if let Some(keys) = keys {
        second = second.child(keys);
    }
    card(col())
        .w(360.0)
        .pt(14.0)
        .px(16.0)
        .pb(15.0)
        .gap(8.0)
        // Two lines of the name and no more (`-webkit-line-clamp: 2`).
        .child(pen(text(title).font(TextStyle::new(14.0, 500.0).lh(kept(19.6))).color(ink()).wrap().max_lines(2), 14.0))
        .child(second.h(17.0))
        .child(toast_clock(toast, at, 358.0))
}

/// What Back's title says it goes back to (`Back to … (⌘[)`, `stopName` in `renderer/src/App.tsx`):
/// Home, the Inbox's tab, or a thread's name. A conversation opened from nowhere goes back to the
/// Inbox.
pub fn back_title(s: &State) -> String {
    let name = match s.history.last() {
        Some(View::Home) => "Home".to_string(),
        Some(View::Thread { id, product }) => s.snap.items.iter().find(|i| &i.id == id && &i.product == product).map_or_else(|| "the thread".to_string(), crate::rules::title::row_title),
        Some(View::Files { .. }) => "Files".to_string(),
        Some(View::Inbox) | None => s.inbox_tab.label().to_string(),
    };
    format!("Back to {name} (⌘[)")
}

/// Back and forward, at the head of the card. Dim while there is nowhere to go. A conversation
/// has a strip of its own there, with these two in it.
pub fn back_forward(s: &State) -> El {
    if let View::Thread { id, product } = &s.view {
        return crate::thread::head(s, id, product);
    }
    // An arrow with somewhere to go takes `--film-strong` under the pointer; one with nowhere
    // keeps the arrow pointer and no look (`.back-forward button:disabled`). Either keeps its
    // press from the strip under it, which moves the window (`.topbar button { no-drag }`).
    let arrow = |glyph: &'static Icon, live: bool| {
        let e = row().size(26.0, 26.0).radius(6.0).center().opacity(if live { 1.0 } else { 0.45 }).child(icon(glyph, 15.0).color(if live { ink() } else { muted() }));
        if live { e.hover(crate::feel::film()).cursor(rune::Cursor::Pointer) } else { e.no_window_drag() }
    };
    // 44 points into the card (`.gt-inset .topbar`), wherever the rail has put the card. Back
    // with somewhere to go says where in its title.
    let back = arrow(&icons::BACK, !s.history.is_empty()).on_press(|s: &mut State| s.back());
    let back = if s.history.is_empty() { back } else { back.help(back_title(s)) };
    let arrows = row().absolute(card_left(s) + 44.0, 58.5).gap(2.0).child(back).child(arrow(&icons::FORWARD, false));
    // The Inbox's title and buttons stand in the same strip as the arrows, and the Files place's name.
    match s.view {
        View::Inbox => el().absolute(0.0, 0.0).child(crate::inbox::strip(s)).child(arrows),
        View::Files { .. } => el().absolute(0.0, 0.0).child(crate::files_page::strip(s)).child(arrows),
        _ => arrows,
    }
}
