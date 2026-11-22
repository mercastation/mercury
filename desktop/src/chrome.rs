//! What every screen stands on: the dotted page, the main card with the sky painted on it, the
//! rail, which opens while the pointer rests on it, and back and forward. Sizes follow the
//! window; the insets are the web app's.

use std::time::Duration;

use rune::{Cursor, Easing, El, Icon, TextCase, TextStyle, Tx, col, el, icon, row, spacer, text};

use crate::rules::window;
use crate::theme::*;
use crate::{State, View, feel, front, icons};

/// The main card's insets from the window with the rail folded: left of it the rail, above it
/// the title strip. While the rail is open the card starts further right (`card_left`).
pub const CARD: [f32; 4] = [82.0, 40.0, 22.0, 22.0];
/// Where the page begins inside the window: under the card's top strip.
pub const PAGE_TOP: f32 = 104.0;

/// The rail's width folded and open (`--workspace-width`); the card starts 44 points past it
/// (`--workspace-left` in `renderer/src/workspace-navigation.css`).
const FOLDED: f32 = 38.0;
const OPEN: f32 = 202.0;

/// Where the card starts now, beside the rail as wide as it is at this frame.
pub fn card_left(s: &State) -> f32 {
    22.0 + s.rail.width(s.at) + 22.0
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
    /// The line under Search.
    line: Fade,
    /// The dot on Inbox while something waits, which the count stands for when open.
    dot: Fade,
}

impl Default for Rail {
    fn default() -> Rail {
        Rail { open: false, due: None, width: Fade::still(FOLDED), words: Fade::still(0.0), line: Fade::still(0.0), dot: Fade::still(1.0) }
    }
}

impl Rail {
    /// The pointer's 140 ms on the rail are up, or it left the rail (`on`). The rules
    /// are `renderer/src/team/rooms/rooms.css`'s: the width eases open over 0.24 s and shut
    /// over 0.2 s; the words come in 0.08 s later over 0.14 s and go over 0.06 s; Search's line
    /// crosses in 0.14 s and Inbox's dot in 0.1 s, its `ease` both ways. Under the system's
    /// Reduce motion (`still`) none of it moves: it opens and folds whole, as the same rules'
    /// `prefers-reduced-motion` block takes every transition away.
    fn point(&mut self, on: bool, now: f64, still: bool) {
        if on == self.open {
            return;
        }
        self.open = on;
        let (ease, k) = (Easing::EASE, if still { 0.0 } else { 1.0 });
        if on {
            self.width = self.width.toward(OPEN, now, 0.0, 0.24 * k, Easing::cubic(0.22, 0.61, 0.36, 1.0));
            self.words = self.words.toward(1.0, now, 0.08 * k, 0.14 * k, ease);
            self.line = self.line.toward(1.0, now, 0.0, 0.14 * k, ease);
            self.dot = self.dot.toward(0.0, now, 0.0, 0.1 * k, ease);
        } else {
            self.width = self.width.toward(FOLDED, now, 0.0, 0.2 * k, Easing::cubic(0.4, 0.0, 0.2, 1.0));
            self.words = self.words.toward(0.0, now, 0.0, 0.06 * k, ease);
            self.line = self.line.toward(0.0, now, 0.0, 0.14 * k, ease);
            self.dot = self.dot.toward(1.0, now, 0.0, 0.1 * k, ease);
        }
    }
    /// How wide it is at `now`.
    pub fn width(&self, now: f64) -> f32 {
        self.width.at(now)
    }
    /// Whether it is opening or folding at `now`, when every frame has to be drawn.
    pub fn moving(&self, now: f64) -> bool {
        [self.width, self.words, self.line, self.dot].iter().any(|f| f.moving(now))
    }
}

/// The poll's part: the rail opens once its 140 ms are up, and while it opens or folds each
/// frame is drawn and asks for the next.
pub fn rail_moves(s: &mut State, at: f64, tx: &mut Tx) -> bool {
    if let Some((due, still)) = s.rail.due.filter(|&(due, _)| at >= due) {
        s.rail.due = None;
        s.rail.point(true, due, still);
    }
    let moving = s.rail.moving(at);
    if moving {
        tx.wake_after(Duration::ZERO);
    }
    moving
}

/// The same for the screen on show: the strip over a conversation is 56 points tall, not 64.
pub fn page_top(view: &View) -> f32 {
    match view {
        View::Thread { .. } => 96.0,
        _ => PAGE_TOP,
    }
}

/// The page under everything: a warm glow from the upper right and a dot every 32 points.
pub fn backdrop(w: f32, h: f32) -> El {
    let mut dots = Vec::new();
    let mut y = 16.0;
    while y < h {
        let mut x = 16.0;
        while x < w {
            dots.push(el().absolute(x - 1.15, y - 1.15).size(2.3, 2.3).radius(1.15).fill(white(0.055)));
            x += 32.0;
        }
        y += 32.0;
    }
    el().absolute(0.0, 0.0)
        .size(w, h)
        .radial_gradient([0.72, 0.28], [0.7 * w, 0.7 * h], [(0.0, rgba(238, 96, 24, 0.075)), (1.0, rgba(238, 96, 24, 0.0))])
        .children(dots)
}

/// The main card. The sky is drawn at the size it was painted for and cut to the card, so a
/// window being resized shows the old sky still, never a stretched one, until the new one lands.
/// It stands on the card's right edge, so the rail opening covers its left edge.
pub fn card(s: &State) -> El {
    let card = el()
        .pin(Some(card_left(s)), Some(CARD[1]), Some(CARD[2]), Some(CARD[3]))
        .radius(6.0)
        .fill(rgba(36, 34, 33, 0.55))
        .box_shadow(shadow(0.0, 40.0, 120.0, 0.0, rgba(0, 0, 0, 0.7), false));
    let whole = || el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).radius(6.0);
    let (card_w, _) = s.card_size();
    match s.view {
        // Home's sky covers the whole card, on a ground of its own, under a faint edge.
        View::Home => card
            .child(whole().fill(rgb(0x1b, 0x19, 0x17)).clip_rounded(6.0).child(el().absolute(card_w - s.sky_size.0, 0.0).size(s.sky_size.0, s.sky_size.1).pixels(&s.sky_home)))
            .child(whole().border(1.0, white(0.07))),
        // The Inbox's sky stands behind its head alone, so under it the card shows its own glass
        // and edge. The Files place wears it too (`skyOnPlace` in `renderer/src/App.tsx`).
        View::Inbox | View::Files { .. } => card.box_shadow(shadow(0.0, 0.0, 0.0, 1.0, edge(), true)).child(crate::inbox::sky(s)),
        // A conversation has no sky: the card is only its wash over the dotted page, inside a plain edge.
        View::Thread { .. } => card.child(whole().border(1.0, edge())),
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

/// One of the rail's buttons (`.workspace-navigation button`): its mark, and the word beside it
/// that shows while the rail is open, cut off by the button's edge as it widens. The page on
/// screen is lit; any other washes under the pointer, its mark and word in the text's colour
/// (`button:hover`), and every one takes the pointing hand.
fn rail_button(glyph: &'static Icon, word: &str, active: bool, wide: f32, words: f32) -> El {
    let tone = if active { ink() } else { muted() };
    let b = row().size(wide, 36.0).radius(6.0).items_center().pl(10.0).pr(10.0).gap(11.0).clip_rounded(6.0).cursor(Cursor::Pointer);
    let b = if active { b.fill(rgba(40, 38, 37, 0.7)).box_shadow(shadow(0.0, 0.0, 0.0, 1.0, white(0.07), true)) } else { b.hover_group().hover(feel::wash()) };
    b.child(icon(glyph, 18.0).color(tone).group_color(ink())).child(pen(label(word, 13.0, 400.0, tone).group_color(ink()), 13.0).opacity(words))
}

/// The person's own face at the rail's end: two letters on a light disc (`.tm-av.you`).
fn my_face(letters: &str) -> El {
    row()
        .size(22.0, 22.0)
        .radius(11.0)
        .center()
        .fill(dim())
        .child(text(letters).font(TextStyle::new(9.0, 500.0).mono().lh(20.0).tracking(0.18)).text_case(TextCase::Uppercase).color(page()))
}

/// The rail: the mark, search, Home, the Inbox and Files, and a foot with Restart to update while
/// a new version waits, Settings, what is left of the limit and the person's own face. Folded it
/// is the marks alone; while the pointer rests on it, it opens to the whole sidebar, each mark
/// with its word and Inbox with its count (`Rail`). `still`: the system asks for reduced motion.
///
/// ONLY WHAT WORKS HERE. The web window's rail also has Demands, Chats, Connections, Record
/// meeting and Feedback. Drawn here they were buttons that went nowhere, which is worse than no
/// button, so they are left out until this window has their pages. Settings, which it has no
/// page for either, brings the app's own window forward on them.
pub fn rail(s: &State, still: bool) -> El {
    let now = s.at;
    let (wide, words, line, dot) = (s.rail.width(now), s.rail.words.at(now), s.rail.line.at(now), s.rail.dot.at(now));
    // Folded, a button's title names it; open, its word does (`title={collapsed ? … : undefined}`).
    let folded = !s.rail.open;
    let button = |glyph: &'static Icon, word: &str, active: bool| {
        let b = rail_button(glyph, word, active, wide, words);
        if folded { b.help(word) } else { b }
    };
    let on_home = s.view == View::Home;
    let on_inbox = matches!(s.view, View::Inbox | View::Thread { .. });
    let waiting = s.lists.needs.len();
    // Inbox: the number that waits, open (`.workspace-running`, in the accent's figures), and
    // folded a dot in its corner standing for it (`.has-count::after`).
    let mut inbox = rail_button(&icons::INBOX, "Inbox", on_inbox, wide, words);
    if waiting > 0 {
        let count = text(waiting.to_string()).font(mono(13.0, 18.0).features("'tnum' 1")).color(accent());
        inbox = inbox.child(spacer()).child(row().min_w(20.0).justify_end().opacity(words).child(count)).help(format!("Inbox · {waiting} {} waiting", if waiting == 1 { "thread" } else { "threads" }));
    } else if folded {
        inbox = inbox.help("Inbox");
    }
    let mut inbox = el().child(inbox.on_press(|s: &mut State| s.go(View::Inbox)));
    if waiting > 0 {
        inbox = inbox.child(el().absolute(25.0, 8.0).size(6.0, 6.0).radius(3.0).fill(accent()).opacity(dot));
    }
    // Files, the place of every file the threads made, opens on All files (`openFiles(null)`).
    let on_files = matches!(s.view, View::Files { .. });
    let places = col()
        .absolute(0.0, 98.0)
        .gap(3.0)
        .child(button(&icons::HOME, "Home", on_home).on_press(|s: &mut State| s.go(View::Home)))
        .child(inbox)
        .child(button(&icons::FILES, "Files", on_files).on_press(|s: &mut State| s.open_files_place(None)));

    // The foot stands on the rail's end and grows upward: the person's face under a hairline,
    // over it the usage row, over that the buttons (`.workspace-bottom`). Their titles stand
    // whether the rail is open or not.
    let me = window::my_letters(&s.snap);
    let usage = window::usage_shown(&s.snap, s.now);
    let face_room = if me.is_some() { 54.0 } else { 0.0 };
    let foot = |glyph: &'static Icon, word: &str| rail_button(glyph, word, false, wide, words).help(word);
    let mut buttons = col().pin(Some(0.0), None, None, Some(face_room + if usage { 50.0 } else { 12.0 })).gap(2.0);
    if window::update_waits(&s.snap) {
        // Restart to update stands first, with the room its card rises from (`.sb-update-slot`).
        // The app quits onto the new version and comes back, and this window finds its new door.
        let restart = foot(&icons::RESTART, "Restart to update").on_press_tx(|s: &mut State, tx| s.ask_the_app(front::INSTALL_UPDATE, "Mercury could not restart onto the new version. Try again in a moment.", tx));
        buttons = buttons.child(el().pb(9.0).child(restart));
    }
    buttons = buttons.child(foot(&icons::SETTINGS, "Settings").on_press_tx(open_settings));

    // The team's mark, and its name beside it while the rail is open (`.th-team`), which the
    // rail's edge cuts as it widens.
    let team = row()
        .absolute(0.0, 0.0)
        .size(wide, 34.0)
        .clip_rounded(0.0)
        .child(el().absolute(13.0, 11.0).child(app_mark()))
        .child(el().absolute(39.0, 8.0).opacity(words).child(pen(text(window::team_name(&s.snap)).font(sans(14.0, 400.0)).color(ink()), 14.0)));
    let team = if folded { team } else { team.help(window::team_name(&s.snap)) };
    // Search, over the places (`.workspace-find`): the glass, its word and its keys, and a line
    // under them that shows only open, fainter than any divider until the pointer is on it. It
    // opens the command bar, as ⌘K does.
    let under = el().pin(Some(10.0), None, Some(10.0), Some(0.0)).h(1.0).fill(edge()).opacity(line).group_opacity(0.55, 1.0);
    let search = row()
        .absolute(0.0, 52.0)
        .size(wide, 32.0)
        .items_center()
        .pl(10.0)
        .pr(10.0)
        .gap(11.0)
        .clip_rounded(0.0)
        .hover_group()
        .cursor(Cursor::Pointer)
        .child(icon(&icons::SEARCH, 18.0).color(muted()).group_color(ink()))
        .child(pen(label("Search", 13.0, 400.0, muted()).group_color(ink()), 13.0).opacity(words))
        .child(spacer())
        .child(pen(text("⌘K").font(sans(12.0, 400.0).tracking(0.24)).color(muted()), 12.0).opacity(0.7 * words))
        .child(under)
        .help("Search · ⌘K")
        .on_press_tx(|s: &mut State, tx| s.open_palette(tx));

    let mut rail = el()
        .pin(Some(22.0), Some(50.0), None, Some(22.0))
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
            if inside {
                tx.live_only();
                return;
            }
            s.rail.due = None;
            s.rail.point(false, tx.clock(), still);
            tx.wake_after(Duration::ZERO);
        })
        .child(team)
        .child(search)
        .child(places)
        .child(buttons);
    if usage {
        // Open, it names the agent read last and, where the Inbox has its count, the figure of
        // its headline window (`.usage-row-label`, `.usage-row-num`). The web window opens the
        // limits under it; here it only reads, so it does not answer the pointer as if it would.
        let (word, used) = window::usage_row(&s.snap, s.now).unwrap_or(("Usage", None));
        let mut row = rail_button(&icons::USAGE, word, false, wide, words);
        if let Some(used) = used {
            row = row.child(spacer()).child(pen(label(&format!("{used}%"), 13.0, 400.0, muted()).group_color(ink()), 13.0).opacity(words));
        }
        rail = rail.child(el().pin(Some(0.0), None, None, Some(face_room + 12.0)).child(row));
    }
    if let Some(letters) = me {
        // Open, the person's own row has their name over their email beside their face
        // (`.th-me-btn`). The web window opens their account menu on it, whose first row, their
        // name again, opens Settings (`AccountMenuList`); here, with no menu, it goes straight there.
        let (name, email) = window::my_names(&s.snap).unwrap_or_default();
        let names = col()
            .absolute(39.0, 2.25)
            .opacity(words)
            .child(pen(label(&name, 13.5, 400.0, ink()), 13.5))
            .child(pen(text(email).font(TextStyle::new(11.5, 400.0).lh(16.0)).color(muted()), 11.5));
        let row = el().pin(Some(0.0), None, None, Some(0.0)).size(wide, 38.0).radius(6.0).clip_rounded(6.0).hover(feel::wash()).cursor(Cursor::Pointer).child(names);
        rail = rail
            .child(el().pin(Some(0.0), None, Some(0.0), Some(45.0)).h(1.0).fill(edge()))
            .child(row.help("Settings").on_press_tx(open_settings))
            .child(el().pin(Some(8.0), None, None, Some(7.5)).child(my_face(&letters)));
    }
    rail
}

/// What the rail's Settings say when the app's own window is not there to show them.
pub const SETTINGS_REFUSED: &str = "Settings live in Mercury's other window, and it is not open now.";

/// The app's Settings, from the rail: its own window comes forward on them, since this one has
/// no Settings page yet.
fn open_settings(s: &mut State, tx: &mut Tx) {
    s.ask_the_app(front::OPEN_SETTINGS, SETTINGS_REFUSED, tx);
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
