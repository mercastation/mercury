//! The Inbox: its sky behind the head, the title and buttons in the card's top strip, the tabs,
//! and the threads of the open tab as a table. Built from the web app's own measurements
//! (`renderer/src/threads/pages.css`, `Pages.tsx`), and matched against it pixel by pixel.

use rune::{Color, Cursor, El, TextCase, TextStyle, col, el, icon, row, text};

use crate::data::{Mark, Priority, Row, Tab};
use crate::rules::{cloud, page};
use crate::theme::*;
use crate::{State, chrome, head, icons};

/// How far down the card the sky shows at all, and how far it shows whole (`.inbox-sky-head`'s
/// mask: black to 46 points, clear by 90).
const SKY_REACH: f32 = 90.0;
const SKY_WHOLE: f32 = 46.0;

/// The sky's own dark (`--sky-under`, the ember art's ground): what the web app fills the strip's
/// buttons with and shades its words in, so the streak never runs through either.
pub(crate) fn under(a: f32) -> Color {
    surface_at(a)
}

/// The web app paints the streak in a band at the card's foot and turns the picture over with
/// CSS (`transform: scaleY(-1)`), so the band stands behind the head. Rune turns nothing over, so
/// the rows the mask lets through are copied out bottom row first. That is done where the sky is
/// painted, off the main thread, and the rest of the painting is let go there: only the band is
/// ever drawn, and a whole second sky kept for it took as much memory as Home's.
pub fn band((w, h, all): (u32, u32, Vec<u8>)) -> (u32, u32, Vec<u8>) {
    let rows = ((SKY_REACH * crate::sky::PER_POINT as f32).ceil() as u32).min(h);
    let line = w as usize * 4;
    let mut out = Vec::with_capacity(line * rows as usize);
    for y in (h - rows..h).rev() {
        out.extend_from_slice(&all[y as usize * line..(y as usize + 1) * line]);
    }
    (w, rows, out)
}

/// How far a button of the strip blurs the sky behind it, in points (`backdrop-filter: blur(6px)`).
pub const STRIP_BLUR: f32 = 6.0;

/// The band as the strip's buttons have it behind them, before they blur it: whole to
/// `SKY_WHOLE` points down and gone by `SKY_REACH`, as the card's mask fades it, into the
/// card's own ground (`--skin-pane`, the deep black `--surface`).
pub fn band_under_the_strip((w, h, band): &(u32, u32, Vec<u8>)) -> (u32, u32, Vec<u8>) {
    const GROUND: [f32; 3] = [19.0, 19.0, 21.0];
    let line = *w as usize * 4;
    let mut out = band.clone();
    for (row, pixels) in out.chunks_exact_mut(line.max(4)).enumerate().take(*h as usize) {
        let down = row as f32 / crate::sky::PER_POINT as f32;
        let keep = ((SKY_REACH - down) / (SKY_REACH - SKY_WHOLE)).clamp(0.0, 1.0);
        if keep >= 1.0 {
            continue;
        }
        for px in pixels.chunks_exact_mut(4) {
            for c in 0..3 {
                px[c] = (f32::from(px[c]) * keep + GROUND[c] * (1.0 - keep)).round() as u8;
            }
        }
    }
    (*w, *h, out)
}

/// Where the band stands in the window, stretched with the card while a window is resized.
pub(crate) fn band_in_the_window(s: &State) -> [f32; 4] {
    let [x, y, sky_w, sky_h] = chrome::sky_in_the_window(s);
    let band_h = s.sky_inbox.size().1 as f32 / crate::sky::PER_POINT as f32;
    [x, y, sky_w, if s.sky_size.1 >= 1.0 { band_h * sky_h / s.sky_size.1 } else { band_h }]
}

/// The Inbox's sky, as the card draws it under this page: at full strength behind the head and
/// gone before the tabs, with the card's own glass showing below it. The ring is the sky's own
/// edge (`.home-sky::after`), which fades with it. It stands on the card's right edge, so the
/// rail opening covers its left edge, as Home's does.
pub fn sky(s: &State) -> El {
    let (card_w, card_h) = s.card_size();
    // The band's height in points, as it was painted.
    let band_h = s.sky_inbox.size().1 as f32 / crate::sky::PER_POINT as f32;
    // A sky painted for another size is stretched to the card as it is now, until the card
    // has held still and the new one lands (`chrome::sky_box`).
    let (sky_w, sky_h) = chrome::sky_box(s);
    let band_h = if s.sky_size.1 >= 1.0 { band_h * sky_h / s.sky_size.1 } else { band_h };
    el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).radius(PANE_CORNER).clip_rounded(PANE_CORNER).child(
        el().pin(Some(0.0), Some(0.0), Some(0.0), None)
            .h(SKY_REACH)
            .clip_fade_linear(0.0, 0.0, 0.0, SKY_REACH - SKY_WHOLE)
            .child(el().absolute(card_w - sky_w, 0.0).size(sky_w, band_h).pixels(&s.sky_inbox))
            .child(el().absolute(0.0, 0.0).size(card_w, card_h).radius(PANE_CORNER).border(1.0, white(0.07))),
    )
}

/// The five shadows the web app gives words over the sky, each a blur of this many points
/// (`text-shadow: 0 0 1px, 0 0 3px, 0 0 6px, 0 0 10px, 0 0 16px`, a shadow's blur being twice
/// its Gaussian's deviation).
const SHADOWS: [f32; 5] = [8.0, 5.0, 3.0, 1.5, 0.5];

/// Words over the sky sit in a soft patch of the sky's own dark. `draw(true)` is the words
/// alone, in that dark; `draw(false)` is the thing itself, which lands on top. Every button of
/// the strip wears it, those of the menus it opens included (`.topbar :is(h1, button)`).
pub(crate) fn shaded(draw: impl Fn(bool) -> El) -> El {
    let shadows = SHADOWS.iter().map(|sigma| el().absolute(0.0, 0.0).blur(*sigma).child(draw(true)));
    // The shadows are gathered in one layer before they reach the screen, which an opacity just
    // short of whole asks Rune for: each is too faint by itself to move a pixel a whole level,
    // and drawn one at a time the wide ones round away to nothing. The patch is a picture of the
    // words, so the pointer passes through it and a screen reader is told them once.
    let patch = el().pin(Some(0.0), Some(0.0), None, None).cache().layer_opacity(0.999).inert().a11y_hidden().children(shadows);
    el().child(patch).child(draw(false))
}

/// A square button of the strip: the sky's dark at 82% over the sky blurred behind it. Under the
/// pointer it gives up the dark and the blur for `--wash-strong` over the sky itself
/// (`.app.sky-shown:not(.home-shown) :is(.th-new, .th-disp, …):not(:hover)` in
/// `renderer/src/home/home.css`, and `.th-new:hover, .th-disp:hover` in `threads/pages.css`):
/// a fill and a blur given up, which Rune's own hover cannot draw, so the window keeps which
/// button the pointer is on (`feel::pointed`) and draws that one so. `key` names the button. A
/// button whose menu is open keeps the dark and the blur (`:not([aria-expanded="true"])` does
/// not take it off a button that says nothing of being open).
pub(crate) fn strip_button(s: &State, key: &'static str) -> El {
    let button = row().key(key).h(32.0).items_center().radius(6.0);
    let edge = rule();
    let whole = || el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0));
    let button = if s.pointed.is(key) {
        button.fill(crate::feel::wash_strong()).border(1.0, edge)
    } else {
        // The sky blurred behind it is the soft band, painted with the sky and shown where the
        // button stands (`chrome::sky_through`): under the button's dark and its edge, which
        // are drawn over it here because a button's own would be drawn under what it holds.
        // Until the button has a place it has its dark alone, in a build that is never drawn.
        match chrome::sky_through(key, &s.sky_inbox_soft, band_in_the_window(s), s.sky_inbox.size(), chrome::SOFT_STRIP, 6.0, 1.0) {
            Some(behind) => button.border(1.0, rgba(0, 0, 0, 0.0)).child(behind.child(whole().fill(under(0.82))).child(whole().radius(6.0).border(1.0, edge))),
            None => button.fill(under(0.82)).border(1.0, edge),
        }
    };
    crate::feel::pointed(button.border_layout().cursor(Cursor::Pointer), key)
}

/// Where threads run (`CloudButton`): a square like the display button's with a cloud in it,
/// orange while the cloud is on. It widens to carry how many threads run there, or a word:
/// "Cloud" until an account is set up, "Sign in again" when one has stopped working. The dot
/// in its corner is the display button's, for something about the setup that wants a look.
/// Under the pointer, or with its menu open, its word is the text's own colour
/// (`.th-cloud.named:hover .th-cloud-word`, `.open`), and so is an unlit cloud while the menu is
/// open (`.th-disp.open`). A press opens its menu (`head.rs`).
fn cloud_button(s: &State, b: cloud::Button) -> El {
    let open = s.head_open() == Some(head::Menu::Cloud);
    let cloud = icon(&icons::CLOUD, 16.0).color(if b.lit { accent() } else if open { ink() } else { dim() });
    let said = |words: String, font: TextStyle, c: Color| shaded(move |shadow| pen(text(words.as_str()).font(font).color(if shadow { under(1.0) } else { c }), 13.0));
    let lit = s.pointed.is(CLOUD) || open;
    let e = match (b.word, b.count) {
        // A word reads as brightly as New thread when it asks for something. With the dot it
        // leaves the dot a corner of its own rather than touching the word's last letter
        // (`.th-cloud.named:has(> i)`).
        (Some(word), _) => strip_button(s, CLOUD).gap(7.0).pl(9.0).pr(if b.dot { 20.0 } else { 14.0 }).child(cloud).child(said(word.said().to_string(), sans(13.0, 400.0), if word == cloud::Word::Again || lit { ink() } else { dim() })),
        (None, Some(running)) => strip_button(s, CLOUD).gap(7.0).pl(9.0).pr(10.0).child(cloud).child(said(running.to_string(), figures(13.0, 400.0), if b.lit { accent() } else { ink() })),
        (None, None) => strip_button(s, CLOUD).w(32.0).justify_center().child(cloud),
    };
    // Its title says what it is, or what it asks (`CloudButton`'s `title`).
    let title = match b.word {
        Some(cloud::Word::Again) => cloud::BROKEN,
        Some(cloud::Word::Cloud) => "Set up the cloud",
        None if b.lit => "Cloud on (⌘C)",
        None => "Cloud off (⌘C)",
    };
    // On or off with its key is what its plate says too ("⌘C", "Cloud on or off"), and which of
    // the two is the cloud's own colour. What it asks for, a setup or a sign-in, only the title says.
    let e = match b.word {
        None => crate::feel::titled_unless_hinted(s, e, title),
        Some(_) => e.help(title),
    };
    let e = e.expanded(open).on_press(|s: &mut State| {
        crate::feel::pressed(s);
        s.toggle_head(head::Menu::Cloud)
    });
    if b.dot { e.child(el().pin(None, Some(5.0), Some(5.0), None).size(6.0, 6.0).radius(3.0).fill(accent())) } else { e }
}

/// The strip's buttons, by the names the pointer keeps them under.
const NEW_THREAD: &str = head::NEW_THREAD;
const CLOUD: &str = head::CLOUD_BUTTON;
const DISPLAY: &str = head::DISPLAY_BUTTON;

/// The card's top strip beside the arrows: the page's title, and at the right New thread, the
/// cloud on a Mac that has one, and the display button. New thread opens the New thread card
/// (`onCompose`); the other two open their menus (`head.rs`).
pub fn strip(s: &State) -> El {
    let (card_w, _) = s.card_size();
    let title = shaded(|shadow| pen(text("Inbox").font(sans(17.0, 400.0).tracking(-0.51)).color(if shadow { under(1.0) } else { ink() }), 17.0));
    let new_thread = strip_button(s, NEW_THREAD)
        .gap(8.0)
        .px(13.0)
        .child(icon(&icons::PEN, 15.0).color(dim()))
        .child(shaded(|shadow| pen(label("New thread", 13.5, 400.0, if shadow { under(1.0) } else { ink() }), 13.5)))
        .on_press_tx(|s: &mut State, tx| {
            crate::feel::pressed(s);
            s.close_head();
            s.open_card(tx);
        });
    // Its plate says N and "New thread": the title would say them again over it.
    let new_thread = crate::feel::titled_unless_hinted(s, new_thread, "New thread (N)");
    // View and filters: its menu opens under it, the sliders brighter while it is open
    // (`.th-disp.open`), and the accent dot while a filter is on (`isFiltered`).
    let open = s.head_open() == Some(head::Menu::Display);
    let display = strip_button(s, DISPLAY)
        .key(DISPLAY)
        .w(32.0)
        .justify_center()
        .child(icon(&icons::SLIDERS, 15.0).color(if open { ink() } else { dim() }))
        .help("View and filters")
        .label("View and filters")
        .expanded(open)
        .on_press(|s: &mut State| s.toggle_head(head::Menu::Display));
    let display = if s.display().is_filtered() { display.child(el().pin(None, Some(5.0), Some(5.0), None).size(6.0, 6.0).radius(3.0).fill(accent())) } else { display };
    // New thread and the cloud carry their keys' hints (`data-hint="new-task"`, `"cloud"`).
    let mut buttons = row().key(head::BUTTONS).gap(10.0).child(crate::feel::hinted(new_thread, "new-task", NEW_THREAD, crate::feel::Align::Left, None));
    if let Some(button) = cloud::button(s.snap.rest.get("cloud"), &s.broken_seen()) {
        buttons = buttons.child(crate::feel::hinted(cloud_button(s, button), "cloud", CLOUD, crate::feel::Align::Center, None));
    }
    // `.topbar`: 64 points with a clear hairline under it, the arrows' 54 and their 12 before the
    // title. It moves the window as the title bar would, its buttons keeping their presses
    // (`-webkit-app-region: drag`, and `no-drag` on `.topbar button`).
    row().absolute(chrome::card_left(s), chrome::CARD[1])
        .size(card_w, 64.0)
        .pl(110.0)
        .pr(28.0)
        .pb(1.0)
        .items_center()
        .justify_between()
        .window_drag()
        .child(title)
        .child(buttons.child(display))
}

/// The table's own type: Geist Mono capitals, as the tabs, the headings and the small cells wear.
fn capitals(s: &str, size: f32, line: f32, spacing: f32, c: Color) -> El {
    pen(text(s).font(mono(size, line).tracking(spacing)).text_case(TextCase::Uppercase).color(c), size)
}

/// One tab: its name, its count while it has one, and the orange line under the open one.
fn tab(at: Tab, count: usize, open: bool, shadow: bool) -> El {
    let (words, number) = match (shadow, open) {
        (true, _) => (under(1.0), under(1.0)),
        (false, true) => (ink(), ink()),
        (false, false) => (muted(), dim()),
    };
    let mut name = capitals(at.label(), 11.0, 30.0, 0.88, words);
    if !shadow {
        name = name.group_color(ink());
    }
    let mut e = row().h(30.0).items_start().gap(8.0).child(name);
    if count > 0 {
        // The count sits on the name's baseline in a 16 point line of its own.
        e = e.child(el().mt(7.0).px(5.0).child(pen(text(count.to_string()).font(mono(11.0, 16.0).features("'tnum' 1")).color(number), 11.0)));
    }
    if shadow {
        return e;
    }
    // Every tab carries Tab's hint, since Tab walks them (`data-hint="state-tab"`), but not the
    // tab just pressed while the pointer stays on it (`quietOnceClicked`).
    let e = e.border_sides([0.0, 0.0, 1.0, 0.0], if open { accent() } else { rgba(0, 0, 0, 0.0) }).hover_group().cursor(Cursor::Pointer).on_press(move |s: &mut State| {
        crate::feel::pressed(s);
        s.show_tab(at)
    });
    crate::feel::hinted(e, "state-tab", format!("tab-{}", at.view()), crate::feel::Align::Left, None)
}

/// `.th-bar`: Needs you, In progress, Later while something is in it, and Done. A number only
/// where it changes what you do.
fn tabs(s: &State) -> El {
    let open = s.inbox_tab;
    let shown = page::tabs(&s.lists, open);
    let all = |shadow: bool| row().gap(22.0).children(shown.iter().map(|at| tab(*at, s.lists.count(*at), *at == open, shadow)));
    row().pt(14.0).pr(32.0).pb(8.0).pl(44.0).child(shaded(all))
}

/// Which of the table's middle columns are drawn, and how wide (`.th-grid`). A narrow table
/// gives up Priority, then Project, and a column no row fills is not drawn at all.
struct Columns {
    title: f32,
    state: Option<f32>,
    project: Option<f32>,
    priority: Option<f32>,
}

/// Between two columns, and the time's own column at the row's end.
const GAP: f32 = 16.0;
const WHEN: f32 = 96.0;

impl Columns {
    fn of(rows: &[&Row], table: f32) -> Columns {
        // The steps are the table's own width (`@container th`): Priority goes at 915 points,
        // Project at 785, where Status also gives the title 16 points back.
        let state = rows.iter().any(|r| !r.status.is_empty()).then_some(if table <= 785.0 { 160.0 } else { 176.0 });
        let project = (table > 785.0 && rows.iter().any(|r| !r.place.is_empty())).then_some(150.0);
        // Every row now says its level, Medium included, so the column is drawn wherever the
        // table has room for it.
        let priority = (table > 915.0 && !rows.is_empty()).then_some(112.0);
        // The rows sit 76 points inside the table, and the title takes what the others leave.
        let taken: f32 = [state, project, priority].into_iter().flatten().map(|w| w + GAP).sum();
        Columns { title: (table - 76.0 - WHEN - GAP - taken).max(0.0), state, project, priority }
    }

    /// One line of the grid: the title's cell, the middle cells that are drawn, the time's. A
    /// row with a second line (`two`) has its title's column hold both lines, and its other
    /// cells centred on them (`.th-grid.has-line`: `grid-row: 1 / span 2; align-self: center`):
    /// the two lines are 20 + 3 + 18 points, so a 20 point cell starts 10.5 down. Status,
    /// Project and Priority are lifted 0.6 from there and the time is let down 0.5
    /// (`transform: translateY(-0.6px)`, `top: 0.5px`), as the web window puts their ink on
    /// the lines' middle.
    fn line(&self, title: El, state: El, project: El, priority: El, when: El, two: bool) -> El {
        let (middle, end) = if two { (TWO_LINES_MIDDLE - 0.6, TWO_LINES_MIDDLE + 0.5) } else { (0.0, 0.0) };
        let mut e = row().items_start().gap(GAP).child(title.w(self.title));
        if !two {
            e = e.h(20.0);
        }
        for (width, cell) in [(self.state, state), (self.project, project), (self.priority, priority)] {
            if let Some(width) = width {
                e = e.child(cell.w(width).mt(middle));
            }
        }
        // The time's words end at the column's end while they fit in it. Words wider than the
        // column start at its start and run out past its end, as a line of text set to the
        // right does in a browser (a dated "Mon, Nov 30, 5:24 PM" under Back): the space
        // before them takes what room there is, and there is none to take.
        e.child(row().w(WHEN).mt(end).child(el().grow(1.0)).child(when))
    }
}

/// A row's two lines (`.th-grid.has-line`): the title's 20 points, 3 between (`row-gap`), and
/// the second line's 18, over which a one-line cell is centred.
const LINE_GAP: f32 = 3.0;
const SECOND_LINE: f32 = 18.0;
const TWO_LINES_MIDDLE: f32 = (20.0 + LINE_GAP + SECOND_LINE - 20.0) / 2.0;
/// A row's padding above and below its lines (`.th-table .row:not(.th-head)`): a row is 17 +
/// 20 + 17 = 54 points tall with one line and 17 + 41 + 17 = 75 with two.
const ROW_PAD: f32 = 17.0;
/// The title and the second line start 35 points into their column, after the face, and stop
/// 32 short of its end (`.th-cell-title`, `.th-line`: `padding-left: 35px; padding-right: 32px`).
const TEXT_IN: f32 = 35.0;
const TEXT_END: f32 = 32.0;

/// A column's name, at the head of the table.
fn heading(s: &str) -> El {
    row().child(capitals(s, 10.5, 20.0, 0.84, muted()))
}

/// The marks beside a priority's word: one to four bars, the strength filled in. Every level
/// has them, Medium included: a blank there read as a gap in the column (`RowCells`).
fn priority_mark(p: Priority) -> El {
    let off = rule();
    let three = |filled: usize| row().items_end().gap(1.5).children([4.0, 7.5, 11.0].into_iter().enumerate().map(move |(n, h)| el().size(3.0, h).radius(1.0).fill(if n < filled { dim() } else { off })));
    match p {
        Priority::Urgent => bars(&[4.0, 7.0, 10.0, 13.0], ink()),
        Priority::High => bars(&[4.0, 7.5, 11.0], ink()),
        Priority::Medium => three(2),
        Priority::Low => three(1),
    }
}

fn priority_word(p: Priority) -> &'static str {
    match p {
        Priority::Urgent => "Urgent",
        Priority::High => "High",
        Priority::Medium => "Medium",
        Priority::Low => "Low",
    }
}

/// The mark before a Status word (`StatusMark`), drawn only for a word about an agent: a ring
/// for one waiting its turn, the orange ring for one nothing will start until something
/// changes, the solid dot where someone's words wait on the person, and the half-dot or the
/// cloud while an agent is on it. The last two breathe, every one in step, as the web window's
/// do (`th-breathe`, which `inStep` starts on one clock), and stand still under Reduce motion.
/// `ground` is the colour of the card under the mark.
fn status_mark(mark: Mark, ground: rune::Color) -> Option<El> {
    use crate::rules::motion::{BREATHE_PERIOD, breathe_stops};
    let dot = || el().size(12.0, 12.0).radius(6.0).mr(8.0);
    let breathing = |mark: El, fps: f32| mark.a11y_hidden().cycle(rune::Cycle { period: BREATHE_PERIOD, phase: 0.0, stops: breathe_stops(), still: 1.0, fps });
    match mark {
        Mark::None => None,
        Mark::Queued => Some(dot().border(1.5, muted())),
        Mark::Held => Some(dot().border(1.5, accent())),
        Mark::Need => Some(dot().fill(accent())),
        // The ring, and its right half filled, drawn as plain fills, which the window can step
        // alone on a breath without drawing the page: the disc breathing, its middle in the
        // card's own colour standing still over it, and the middle's right half breathing over
        // that. It shows what a breathing ring shows, at every moment.
        // Where the screen's pixels are larger than a point, the middle stands between two of
        // them, as a border's inside edge does, rather than on one.
        Mark::Running => Some(
            dot()
                .pixel_snap(false)
                .child(breathing(el().absolute(0.0, 0.0).size(12.0, 12.0).radius(6.0).fill(accent()), 60.0))
                .child(el().absolute(1.5, 1.5).size(9.0, 9.0).radius(4.5).fill(ground))
                .child(breathing(el().absolute(6.0, 1.5).size(4.5, 9.0).corners([0.0, 4.5, 4.5, 0.0]).fill(accent()), 60.0)),
        ),
        // A mark of strokes is not one the window can step alone, so the cloud stands still and
        // a veil of the card's colour over it breathes the other way: where the web window's
        // cloud is at 45%, 55% of the card's colour covers it.
        Mark::Cloud => Some(
            el().size(14.0, 11.0)
                .mr(6.0)
                .child(el().absolute(0.0, 0.0).child(icon(&icons::CLOUD_MARK, 14.0).color(accent())))
                .child(el().absolute(0.0, -2.0).size(14.0, 15.0).fill(ground).a11y_hidden().cycle(rune::Cycle {
                    period: BREATHE_PERIOD,
                    phase: 0.0,
                    stops: crate::rules::motion::breathe_veil_stops(),
                    still: crate::rules::motion::LEAST,
                    fps: 60.0,
                })),
        ),
    }
}

/// The mark a thread of the person's own leads with (`HarnessMark`), at its own size: its
/// harness's mark, 11 points across in a tile of 18, on a faint wash of the text's colour inside
/// a stronger one, Claude Code's in its own orange; and for a harness with no mark, the agent's
/// two eyes in their ring (`AgentFace`). The sidebar's Recent rows draw it so; the Inbox's rows
/// draw their own, larger (`row_face`). `harness` is `rules::engine::harness_mark`'s word.
pub(crate) fn harness_mark(harness: &str) -> El {
    let glyph = match harness {
        "claude" => &icons::HARNESS_CLAUDE,
        "codex" => &icons::HARNESS_CODEX,
        "antigravity" => &icons::HARNESS_ANTIGRAVITY,
        _ => return row().size(18.0, 18.0).radius(9.0).center().border(1.0, rgba(239, 235, 231, 0.12)).child(icon(&icons::AGENT_EYES, 16.0).color(dim())),
    };
    let tone = if harness == "claude" { rgb(0xd9, 0x77, 0x57) } else { ink() };
    row()
        .size(18.0, 18.0)
        .radius(5.0)
        .center()
        .fill(rgba(239, 235, 231, 0.07))
        .border(1.0, rgba(239, 235, 231, 0.16))
        .border_layout()
        .child(icon(glyph, 16.0).color(tone))
}

/// The face an Inbox row leads with, 24 points (`.th-table .row:not(.th-head) .th-slot` in
/// `renderer/src/threads/pages.css`): a harness's mark on a tile of the card's own ground with
/// the agents' 7 point corner and a soft 8% edge, the mark drawn 19.68 points across, which is
/// 11/16 of its square (`IN_TILE`); and for a harness with no mark the agent's two eyes in
/// their faint 12% ring, round, its eyes at their own 16 points (`.agent-face`, which neither
/// the corner nor the softer edge reaches).
fn row_face(harness: &str) -> El {
    let glyph = match harness {
        "claude" => &icons::HARNESS_CLAUDE,
        "codex" => &icons::HARNESS_CODEX,
        "antigravity" => &icons::HARNESS_ANTIGRAVITY,
        _ => return row().size(24.0, 24.0).radius(12.0).center().border(1.0, ink_at(0.12)).border_layout().child(icon(&icons::AGENT_EYES, 16.0).color(dim())),
    };
    let tone = if harness == "claude" { rgb(0xd9, 0x77, 0x57) } else { ink() };
    row().size(24.0, 24.0).radius(7.0).center().fill(surface()).border(1.0, ink_at(0.08)).border_layout().child(icon(glyph, 19.68).color(tone))
}

/// A thread's title cell, `room` points wide: the title, and what follows it, 35 points in,
/// past the face, which the row draws over the cell's start (`row_face`), and ending 32 short.
/// An aside is what the row adds to its title (a comment on a link, how many tasks wait for a
/// yes, the cause a thread in Later is stuck behind), unless the second line says the same
/// (`aside_moved`, `.aside-moved`); "not started" is on a thread written down in Later and never
/// begun; the pin is on one kept at the top. The title is underlined on the selected row, and
/// faintly on the row the pointer holds (`.row.pointed .subject`), which is not the row under it
/// once the keys have taken the list back.
fn title_cell(r: &Row, selected: bool, pointed: bool, room: f32, aside_moved: bool) -> El {
    // An unread thread's title is heavier, and a dot in the gutter says so too.
    let type_ = TextStyle::new(TITLE, if r.unread { 530.0 } else { 400.0 }).lh(20.0).tracking(-0.01 * TITLE);
    let words = pen(text(r.title.as_str()).font(type_).color(ink()).truncate(), TITLE);
    // The line is the title's own, so it ends where the words do.
    let line = el().pin(Some(0.0), None, Some(0.0), Some(0.0)).h(1.0);
    let line = if selected {
        line.fill(rgba(238, 96, 24, 0.6))
    } else if pointed {
        line.fill(rule())
    } else {
        line
    };
    // Every title starts after the face, so the titles line up.
    let cell = row().h(20.0).pl(TEXT_IN).pr(TEXT_END).items_center().child(el().shrink().child(words).child(line));
    // Faint words after the title (`.th-aside`), 12 points on, in the title's own type.
    let faint = |s: &str| pen(text(s).font(type_).color(muted()), TITLE);
    let held = || el().ml(12.0).child(faint("not started"));
    // The pin sits 3 points under the words' line.
    let pin = || el().ml(9.0).child(icon(&icons::PIN, 13.0).color(muted()).mt(3.0));
    let wide = |s: &str| rune::text::measure(s, &[], type_, None).0;

    if !r.aside.is_empty() {
        // With an aside the cell is a row of boxes (`.th-cell-title.with-aside`). The aside
        // takes what the title leaves and no more than its own words need, and is cut short
        // before the title is; "not started" and the pin keep their room, so in a narrow cell
        // it is the title that gives way to them. The aside's 12 points are inside its box,
        // which may close to nothing. Said again by the second line, it is not drawn, and the
        // cell stays a row of boxes without it.
        let cell = if aside_moved {
            cell
        } else {
            cell.child(row().grow(1.0).min_w(0.0).max_w(12.0 + wide(&r.aside)).clip_rounded(0.0).child(el().w(12.0).min_w(12.0)).child(faint(&r.aside).truncate()))
        };
        let cell = if r.held { cell.child(held()) } else { cell };
        return if r.pinned { cell.child(pin()) } else { cell };
    }
    if !r.held && !r.pinned {
        return cell;
    }
    // Without one the cell is a line of text (`.th-cell-title`), and a line too long for it
    // is cut from its end: the pin goes first, then "not started", and only then the title's
    // own words, with the three dots after whatever is left.
    let (title, after, pinned) = (wide(&r.title), if r.held { 12.0 + wide("not started") } else { 0.0 }, if r.pinned { 22.0 } else { 0.0 });
    let room = room - TEXT_IN - TEXT_END;
    if title + after + pinned <= room {
        let cell = if r.held { cell.child(held()) } else { cell };
        return if r.pinned { cell.child(pin()) } else { cell };
    }
    let dots = wide("…");
    if title > room - dots {
        return cell;
    }
    let cell = if r.held && title + after <= room - dots { cell.child(held()) } else { cell };
    cell.child(pen(text("…").font(type_).color(ink()), TITLE))
}

/// The title's size on a row (`.th-table .row:not(.th-head) .th-cell-title`), its letters
/// drawn 1% of it closer.
const TITLE: f32 = 14.5;

/// A row's second line (`.th-line`): what came back, in 13 points of the faint ink on an 18
/// point line, 35 points in under the title, cut with three dots where it runs out of room.
fn second_line(words: &str, room: f32) -> El {
    let said = pen(text(words).font(TextStyle::new(13.0, 400.0).lh(SECOND_LINE)).color(muted()).truncate(), 13.0);
    row().w(room).h(SECOND_LINE).mt(LINE_GAP).pl(TEXT_IN).pr(TEXT_END).child(el().shrink().child(said))
}

/// One thread (`RowCells`, under `.th-table .row:not(.th-head)` in the web window's two-line
/// look): the face on the top of the title's capitals, the title, and under it what came back,
/// the other cells centred on the two lines; 75 points with its second line, 54 without. A
/// hairline above every row but the first, and the title's underline, which is how a row
/// answers: orange on the selected one, faint under the pointer.
fn thread(s: &State, r: &Row, (at, count): (usize, usize), picked: usize, cols: &Columns) -> El {
    // The title's cell is what the row's hint lines its keys up with (`data-hint-text`).
    let subject = format!("subject-{}-{}", r.product, r.id);
    let line = (!r.line.is_empty()).then(|| crate::rules::list::RowLine { who: None, text: r.line.clone() });
    let (_, aside_moved) = crate::rules::list::moved_to_the_line(line.as_ref(), &r.aside, None);
    let title = title_cell(r, at == picked, s.pointed.is_row(&r.product, &r.id), cols.title, aside_moved).w(cols.title).key(subject.clone());
    // The title's column: its cell, the second line under it, and the face over the cell's
    // start, its top on the top of the title's capitals, 5 points into the title's 20 point
    // line and 0.36 higher (`.th-slot`: `top: 5px; transform: translateY(-0.36px)`).
    let title = col()
        .child(title)
        .children(line.as_ref().map(|l| second_line(&l.text, cols.title)))
        // The web window moves it by a transform so that it is not snapped to a whole pixel, and
        // neither is it here.
        .child(row_face(r.harness).absolute(0.0, 5.0 - 0.36).pixel_snap(false).a11y_hidden());
    let small = |s: &str, c: Color| capitals(s, 11.5, 20.0, 0.575, c).truncate();
    let mut state = row().items_center();
    // The card's colour where the row stands, as last laid out: what shows inside a ring.
    let at_row = rune::memo::window_layout(&format!("thread-{}-{}", r.product, r.id)).map_or((s.size.0 / 2.0, s.size.1 / 2.0), |[x, y, w, h]| (x + w / 2.0, y + h / 2.0));
    let [red, green, blue] = crate::rules::motion::ground_under_the_card(at_row, s.size);
    if let Some(mark) = status_mark(r.mark, rgb(red.round() as u8, green.round() as u8, blue.round() as u8)) {
        state = state.child(mark);
    }
    let state = state.child(small(&r.status, dim()));
    // Try again after the word, on a thread nothing will start without the person.
    let state = state.children(r.retry.then(|| crate::foot::row_retry(&r.id, &r.product)));
    // Under the pointer the word says its whole sentence, or the card says why the cloud has not
    // got the thread, in its place (`RowCells`' `title`, `.th-cell-state.has-card`).
    // The rows after it paint over the card, so it shows down to this row's foot unless it is the last.
    let cut_at = (at + 1 < count).then(|| format!("thread-{}-{}", r.product, r.id));
    let state = match (&r.why, &r.status_title) {
        (Some(why), _) => crate::foot::with_card(state, format!("status-{}-{}", r.product, r.id), why.clone(), false, cut_at),
        (None, Some(title)) => state.help(title.as_str()),
        (None, None) => state,
    };
    // A cause's row names where most of the threads behind it are, and counts the other projects.
    let mut project = row().items_center().child(small(&r.place, muted()));
    if r.place_more > 0 {
        project = project.child(el().ml(6.0).child(capitals(&format!("+{}", r.place_more), 11.5, 20.0, 0.575, muted())));
    }
    let c = if r.priority == Priority::Urgent { accent() } else { muted() };
    let priority = row().items_center().child(priority_mark(r.priority).mr(8.0)).child(capitals(priority_word(r.priority), 11.5, 20.0, 0.575, c));
    // When it was sent, when it was finished on Done, when it comes back on Later.
    let when = pen(text(r.when.as_str()).font(TextStyle::new(13.0, 400.0).lh(20.0)).color(dim()), 13.0);
    let (id, product) = (r.id.clone(), r.product.clone());
    let e = row()
        .key(format!("thread-{}-{}", r.product, r.id))
        .items_start()
        .py(ROW_PAD)
        .pr(32.0)
        .pl(44.0)
        .cursor(Cursor::Pointer)
        // The row pressed is the row the keys are on when the person comes back.
        .on_press(move |s: &mut State| {
            crate::feel::pressed(s);
            s.picked = at;
            s.open_thread(&id, &product);
        })
        .child(cols.line(title, state, project, priority, when, line.is_some()));
    let mut e = crate::feel::pointed_row(e, &r.product, &r.id);
    if at > 0 {
        e = e.child(el().pin(Some(20.0), Some(0.0), Some(32.0), None).h(1.0).fill(edge_at(0.375)));
    }
    // The row the app makes itself wears a faint rule at its left edge where a thread's select
    // box would be (`.trouble-rule`: from 19 points down to 15 short of its foot).
    if r.id == crate::rules::trouble_row::TROUBLE_ID && r.product.is_empty() {
        e = e.child(el().pin(Some(20.0), Some(19.0), None, Some(15.0)).w(2.0).radius(1.0).fill(muted()));
    }
    // The unread dot stands in the gutter, level with the middle of the face beside it:
    // 17 + 5 + 12 - 0.36, less half the dot (`.gt-inset .th-table .row.unread::before`).
    if r.unread {
        e = e.child(el().absolute(UNREAD_LEFT, ROW_PAD + 5.0 + 12.0 - 0.36 - 3.0).size(6.0, 6.0).fill(accent()));
    }
    // Its keys' hint, lined up with its title (`data-hint="row"`, `data-hint-text=".subject"`),
    // only under Needs you, where its keys mean what the plate says (`rowKeys`).
    if s.inbox_tab != Tab::Needs {
        return e;
    }
    crate::feel::hinted(e, "row", format!("thread-{}-{}", r.product, r.id), crate::feel::Align::Left, Some(subject))
}

/// The unread dot's left, in the row (`.gt-inset .th-table .row.unread::before { left: 25px }`).
const UNREAD_LEFT: f32 = 25.0;

/// The one quiet line an empty tab carries (`.th-empty`), in the middle of what the tabs leave
/// of the page, less 12% of the window's height at its foot.
fn empty(words: &str, s: &State) -> El {
    let (_, page_h) = s.page_size();
    row().min_h((page_h - 52.0).max(0.0))
        .pt(32.0)
        .px(24.0)
        .pb(0.12 * s.size.1)
        .center()
        .child(pen(text(words).font(TextStyle::new(14.0, 400.0).lh(21.0)).color(muted()).align(rune::Align::Center).wrap().max_w(420.0), 14.0))
}

pub fn view(s: &State) -> El {
    let (page_w, _) = s.page_size();
    let tab = s.inbox_tab;
    // The open tab's rows in its own order, and its words when it has none. The web app's
    // empty Inbox is a page of its own, a sky over the whole card and a box to write in, which
    // this window does not draw while it only reads. A tab only the filters emptied says so
    // first, so it is never read as clear (`FilteredEmpty`).
    let rows = s.lists.under(tab);
    let page = col().w(page_w).child(tabs(s));
    if rows.is_empty() {
        let hidden = s.lists.whole(tab);
        if hidden > 0 {
            return page.child(head::filtered_empty(s, tab, hidden));
        }
        return page.child(empty(&page::empty_tab_words(tab.view(), s.lists.needs.len()), s));
    }
    let cols = Columns::of(&rows, page_w);
    // The time column is named for what it tells under this tab: Sent, Back on Later, Done.
    let head = row().h(38.0).pt(10.0).pr(32.0).pl(44.0).child(cols.line(heading("Thread"), heading("Status"), heading("Project"), heading("Priority"), heading(page::time_heading(tab.view())), false));
    let picked = s.picked.min(rows.len() - 1);
    page.child(head).children(rows.iter().enumerate().map(|(i, r)| thread(s, r, (i, rows.len()), picked, &cols)))
}

#[cfg(test)]
mod tests {
    use rune::{Key, Runtime};
    use serde_json::json;

    use super::*;
    use crate::feel::tests::{key, run, window_dressed};

    /// The invented inbox of four, its last thread's answer only its title again, so its row has
    /// nothing to say under the title; the second still open, the person's own work waiting its
    /// turn, which puts it under In progress; and the third closed by the person, under Done.
    fn rows(name: &str) -> Runtime<State> {
        window_dressed(name, &["--view", "inbox"], "/fixtures/storefront", "Done, and tested. Merge it?", |f| {
            let items = f["snapshot"]["items"].as_array_mut().unwrap();
            items[3]["result"] = json!("Invented thread number 3");
            items[1]["status"] = json!("open");
            items[1].as_object_mut().unwrap().remove("result");
            items[1]["wrote"].as_object_mut().unwrap().remove("result");
            items[1]["wrote"].as_object_mut().unwrap().remove("status");
            items[2]["wrote"]["status"]["source"] = json!("founder");
        })
    }

    fn rect(rt: &Runtime<State>, key: &str) -> [f32; 4] {
        rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key)).map(|e| e.rect).unwrap_or_else(|| panic!("{key} is drawn"))
    }

    /// Within a hundredth of a point.
    fn at(rt: &Runtime<State>, want: [f32; 4]) -> bool {
        rt.elements().iter().any(|e| e.rect.iter().zip(want).all(|(a, b)| (a - b).abs() < 0.01))
    }

    fn said(rt: &Runtime<State>) -> Option<String> {
        rt.state.toast.as_ref().map(|t| t.line.clone())
    }

    // `tests/an-inbox-row-has-two-lines-and-its-face-sits-on-the-title.test.mts`, on the desktop.
    #[test]
    fn a_row_is_its_title_and_what_came_back_and_one_with_nothing_more_to_say_stays_one_line() {
        let rt = rows("two-lines");
        let [x, y, _, h] = rect(&rt, "thread-storefront-w-feel0");
        // 17 + 20 + 3 + 18 + 17.
        assert_eq!(h, 75.0);
        assert_eq!(rt.state.lists.needs.iter().find(|r| r.id == "w-feel0").map(|r| r.line.as_str()), Some("Done, and tested. Merge it?"));
        assert!(rt.elements().iter().any(|e| e.text.as_deref() == Some("Done, and tested. Merge it?") && (e.rect[1] - (y + 17.0 + 23.0)).abs() < 0.01 && (e.rect[0] - (x + 44.0 + 35.125)).abs() < 0.01), "the second line, 35 points in under the title");
        // The title, 35 points into its cell, at 14.5 points.
        let [tx, ty, ..] = rect(&rt, "subject-storefront-w-feel0");
        assert_eq!((tx, ty), (x + 44.0, y + 17.0));
        // The face: 24 points on the top of the title's capitals.
        assert!(at(&rt, [x + 44.0, y + 17.0 + 5.0 - 0.36, 24.0, 24.0]), "the face");
        // Status, Project, Priority and the time centred on the two lines.
        let done = rt.elements().into_iter().find(|e| e.text.as_deref() == Some("Done") && e.rect[1] > y && e.rect[1] < y + h).expect("its status");
        assert!((done.rect[1] - (y + 17.0 + 10.5 - 0.6)).abs() < 0.01, "the status at {:?}", done.rect);
        // Nothing to say but the title: no second line, and 54 points.
        assert_eq!(rt.state.lists.needs.iter().find(|r| r.id == "w-feel3").map(|r| r.line.as_str()), Some(""));
        let [_, y3, _, h3] = rect(&rt, "thread-storefront-w-feel3");
        assert_eq!(h3, 54.0);
        let done = rt.elements().into_iter().find(|e| e.text.as_deref() == Some("Done") && e.rect[1] > y3 && e.rect[1] < y3 + h3).expect("its status");
        assert_eq!(done.rect[1], y3 + 17.0, "a one-line row's cells on its one line");
    }

    #[test]
    fn words_after_the_title_that_the_second_line_says_leave_the_first_and_the_app_s_own_row_wears_its_rule() {
        let rt = window_dressed("moved", &["--view", "inbox"], "/fixtures/storefront", "Done, and tested. Merge it?", |f| {
            f["snapshot"]["items"][0]["handedOver"] = json!({ "said": "Done, and tested. Merge it?" });
            f["snapshot"]["supervisor"]["spawnTrouble"] = json!({ "since": f["now"].as_f64().unwrap() - 600_000.0, "cause": "signed-out", "message": "", "remedy": "", "count": 2 });
        });
        let row = rt.state.lists.needs.iter().find(|r| r.id == "w-feel0").unwrap();
        assert_eq!((row.aside.as_str(), row.line.as_str()), ("Done, and tested. Merge it?", "Done, and tested. Merge it?"));
        let [_, y, ..] = rect(&rt, "thread-storefront-w-feel0");
        let on = |line: f32| rt.elements().iter().filter(|e| e.text.as_deref() == Some("Done, and tested. Merge it?") && (e.rect[1] - line).abs() < 0.01).count();
        assert_eq!((on(y + 17.0), on(y + 40.0)), (0, 1), "said once, on the second line");
        // The row of tasks that could not run: a faint rule at its left edge, 19 points down to 15 short of its foot.
        let [x, y, _, h] = rect(&rt, "thread--trouble");
        assert!(at(&rt, [x + 20.0, y + 19.0, 2.0, h - 34.0]), "its rule");
    }

    #[test]
    fn an_unread_rows_dot_is_level_with_the_middle_of_its_face() {
        // Nothing in the invented inbox has been opened, so every row is unread.
        let rt = rows("unread-dot");
        assert!(rt.state.lists.needs.iter().all(|r| r.unread));
        let [x, y, ..] = rect(&rt, "thread-storefront-w-feel0");
        assert!(at(&rt, [x + 25.0, y + 30.64, 6.0, 6.0]), "17 + 5 + 12 - 0.36, less half the dot");
    }

    #[test]
    fn the_row_hint_is_only_where_its_keys_mean_what_it_says() {
        let mut rt = rows("row-hint");
        let [_, y, ..] = rect(&rt, "thread-storefront-w-feel0");
        rt.pointer_move(400.0, y + 30.0);
        run(&mut rt, 0.8);
        assert_eq!(rt.state.pointed.hint.as_ref().map(|h| h.id), Some("row"), "under Needs you");
        rt.update(|s, _| s.show_tab(Tab::Done));
        rt.pointer_move(640.0, 740.0);
        rt.frame(1.0 / 60.0);
        let [_, y, ..] = rect(&rt, &format!("thread-storefront-{}", rt.state.lists.done[0].id));
        rt.pointer_move(400.0, y + 20.0);
        run(&mut rt, 0.8);
        assert!(rt.state.pointed.hint.is_none(), "not under Done, where E closes nothing");
    }

    // `tests/e-and-z-always-say-what-they-did.test.mts`: E on a list where it closes nothing.
    #[test]
    fn e_closes_in_needs_you_and_says_why_it_does_not_in_progress_and_done() {
        let mut rt = rows("e-tabs");
        rt.state.pretend_sent = true;
        run(&mut rt, 1.0);
        rt.update(|s, _| s.show_tab(Tab::Progress));
        assert_eq!(rt.state.lists.progress.first().map(|r| r.id.as_str()), Some("w-feel1"));
        key(&mut rt, Key::Text("e".into()));
        assert_eq!(said(&rt).as_deref(), Some("Open this thread to close it."));
        assert!(rt.state.held.is_none() && rt.state.tried.is_empty(), "nothing closed");
        assert_eq!(rt.state.lists.progress.len(), 1);
        rt.update(|s, _| s.show_tab(Tab::Done));
        assert!(!rt.state.lists.done.is_empty());
        key(&mut rt, Key::Text("e".into()));
        assert_eq!(said(&rt).as_deref(), Some("This thread is already done."));
        assert!(rt.state.held.is_none() && rt.state.tried.is_empty(), "nothing closed");
        // Under Needs you it closes the row the keys are on.
        rt.update(|s, _| s.show_tab(Tab::Needs));
        let first = rt.state.lists.needs[0].id.clone();
        key(&mut rt, Key::Text("e".into()));
        assert!(rt.state.held.as_ref().is_some_and(|h| h.closes.is_some()), "held its three seconds");
        assert!(rt.state.lists.needs.iter().all(|r| r.id != first));
    }

    #[test]
    fn of_the_inbox_sky_only_the_rows_behind_the_head_are_kept_the_last_row_first() {
        // A painting two pixels wide and 400 rows tall, each row filled with its own number.
        let all: Vec<u8> = (0..400u32).flat_map(|y| [(y % 256) as u8; 8]).collect();
        let (w, rows, kept) = band((2, 400, all));
        // Ninety points of it show, at two pixels a point.
        assert_eq!((w, rows), (2, 180));
        assert_eq!(kept.len(), 2 * 180 * 4);
        assert_eq!(kept[0], (399 % 256) as u8);
        assert_eq!(kept[kept.len() - 1], 220);
        // A painting shorter than the band is kept whole, and an empty one stays empty.
        assert_eq!(band((2, 10, vec![7; 80])), (2, 10, vec![7; 80]));
        assert_eq!(band((0, 0, Vec::new())), (0, 0, Vec::new()));
    }
}
