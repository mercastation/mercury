//! The Inbox: its sky behind the head, the title and buttons in the card's top strip, the tabs,
//! and the threads of the open tab as a table. Built from the web app's own measurements
//! (`renderer/src/threads/pages.css`, `Pages.tsx`), and matched against it pixel by pixel.

use std::cell::RefCell;

use rune::time::LocalTime;
use rune::{Color, Cursor, El, Pixels, TextCase, TextStyle, col, el, icon, row, text};

use crate::data::{Priority, Row};
use crate::theme::*;
use crate::{State, View, chrome, icons};

/// How far down the card the sky shows at all, and how far it shows whole (`.inbox-sky-head`'s
/// mask: black to 46 points, clear by 90).
const SKY_REACH: f32 = 90.0;
const SKY_WHOLE: f32 = 46.0;

/// The sky's own dark (`--sky-under`, the ember art's ground): what the web app fills the strip's
/// buttons with and shades its words in, so the streak never runs through either.
fn under(a: f32) -> Color {
    rgba(0x1b, 0x19, 0x17, a)
}

thread_local! {
    /// The sky's band turned over, and the painting it was cut from.
    static TURNED: RefCell<Option<((u64, u64), Pixels)>> = const { RefCell::new(None) };
}

/// The web app paints the streak in a band at the card's foot and turns the picture over with
/// CSS (`transform: scaleY(-1)`), so the band stands behind the head. Rune turns nothing over, so
/// the rows the mask lets through are copied out bottom row first, once for each painting.
fn turned(s: &State) -> (Pixels, f32) {
    let painted = s.sky_inbox.signature();
    let (w, h) = s.sky_inbox.size();
    // The painting's pixels a point, from the card it was painted for.
    let per_point = if s.sky_size.1 >= 1.0 { h as f32 / s.sky_size.1.floor() } else { 1.0 };
    let rows = ((SKY_REACH * per_point).ceil() as u32).min(h);
    let band = TURNED.with(|kept| {
        let mut kept = kept.borrow_mut();
        if let Some((from, band)) = kept.as_ref()
            && *from == painted
        {
            return band.clone();
        }
        let all = s.sky_inbox.rgba();
        let line = w as usize * 4;
        let mut out = Vec::with_capacity(line * rows as usize);
        for y in (h - rows..h).rev() {
            out.extend_from_slice(&all[y as usize * line..(y as usize + 1) * line]);
        }
        let band = Pixels::new();
        band.set(w, rows, out);
        *kept = Some((painted, band.clone()));
        band
    });
    (band, rows as f32 / per_point)
}

/// The Inbox's sky, as the card draws it under this page: at full strength behind the head and
/// gone before the tabs, with the card's own glass showing below it. The ring is the sky's own
/// edge (`.home-sky::after`), which fades with it.
pub fn sky(s: &State) -> El {
    let (card_w, card_h) = s.card_size();
    let (band, band_h) = turned(s);
    el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).radius(6.0).clip_rounded(6.0).child(
        el().pin(Some(0.0), Some(0.0), Some(0.0), None)
            .h(SKY_REACH)
            .clip_fade_linear(0.0, 0.0, 0.0, SKY_REACH - SKY_WHOLE)
            .child(el().absolute(0.0, 0.0).size(s.sky_size.0, band_h).pixels(&band))
            .child(el().absolute(0.0, 0.0).size(card_w, card_h).radius(6.0).border(1.0, white(0.07))),
    )
}

/// The five shadows the web app gives words over the sky, each a blur of this many points
/// (`text-shadow: 0 0 1px, 0 0 3px, 0 0 6px, 0 0 10px, 0 0 16px`, a shadow's blur being twice
/// its Gaussian's deviation).
const SHADOWS: [f32; 5] = [8.0, 5.0, 3.0, 1.5, 0.5];

/// Words over the sky sit in a soft patch of the sky's own dark. `draw(true)` is the words
/// alone, in that dark; `draw(false)` is the thing itself, which lands on top.
fn shaded(draw: impl Fn(bool) -> El) -> El {
    let shadows = SHADOWS.iter().map(|sigma| el().absolute(0.0, 0.0).blur(*sigma).child(draw(true)));
    // The shadows are gathered in one layer before they reach the screen, which an opacity just
    // short of whole asks Rune for: each is too faint by itself to move a pixel a whole level,
    // and drawn one at a time the wide ones round away to nothing. The patch is a picture of the
    // words, so the pointer passes through it and a screen reader is told them once.
    let patch = el().pin(Some(0.0), Some(0.0), None, None).cache().layer_opacity(0.999).inert().a11y_hidden().children(shadows);
    el().child(patch).child(draw(false))
}

/// A square button of the strip: the sky's dark at 82% over the sky blurred behind it.
fn strip_button() -> El {
    row().h(32.0).items_center().radius(6.0).backdrop_blur(6.0).fill(under(0.82)).border(1.0, rgb(82, 80, 79)).border_layout()
}

/// The card's top strip beside the arrows: the page's title, and at the right New thread and
/// the display button. Phase 1 only draws the buttons.
pub fn strip(s: &State) -> El {
    let (card_w, _) = s.card_size();
    let title = shaded(|shadow| pen(text("Inbox").font(sans(17.0, 400.0).tracking(-0.51)).color(if shadow { under(1.0) } else { ink() }), 17.0));
    let new_thread = strip_button()
        .gap(8.0)
        .px(13.0)
        .child(icon(&icons::PEN, 15.0).color(dim()))
        .child(shaded(|shadow| pen(label("New thread", 13.5, 400.0, if shadow { under(1.0) } else { ink() }), 13.5)));
    let display = strip_button().w(32.0).justify_center().child(icon(&icons::SLIDERS, 15.0).color(dim()));
    // `.topbar`: 64 points with a clear hairline under it, the arrows' 54 and their 12 before the title.
    row().absolute(chrome::CARD[0], chrome::CARD[1])
        .size(card_w, 64.0)
        .pl(110.0)
        .pr(28.0)
        .pb(1.0)
        .items_center()
        .justify_between()
        .child(title)
        .child(row().gap(10.0).child(new_thread).child(display))
}

/// The table's own type: Geist Mono capitals, as the tabs, the headings and the small cells wear.
fn capitals(s: &str, size: f32, line: f32, spacing: f32, c: Color) -> El {
    pen(text(s).font(mono(size, line).tracking(spacing)).text_case(TextCase::Uppercase).color(c), size)
}

/// One tab: its name, its count while it has one, and the orange line under the open one.
fn tab(name: &str, count: usize, at: u8, open: bool, shadow: bool) -> El {
    let (words, number) = match (shadow, open) {
        (true, _) => (under(1.0), under(1.0)),
        (false, true) => (ink(), ink()),
        (false, false) => (muted(), dim()),
    };
    let mut name = capitals(name, 11.0, 30.0, 0.88, words);
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
    e.key(format!("tab-{at}"))
        .border_sides([0.0, 0.0, 1.0, 0.0], if open { accent() } else { rgba(0, 0, 0, 0.0) })
        .hover_group()
        .cursor(Cursor::Pointer)
        .on_press(move |s: &mut State| s.inbox_tab = at)
}

/// `.th-bar`: Needs you, In progress and Done. A number only where it changes what you do.
fn tabs(s: &State) -> El {
    let open = s.inbox_tab;
    let all = |shadow: bool| {
        row().gap(22.0)
            .child(tab("Needs you", s.lists.needs.len(), 0, open == 0, shadow))
            .child(tab("In progress", s.lists.progress.len(), 1, open == 1, shadow))
            .child(tab("Done", 0, 2, open == 2, shadow))
    };
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
    fn of(rows: &[Row], table: f32) -> Columns {
        // The steps are the table's own width (`@container th`): Priority goes at 915 points,
        // Project at 785, where Status also gives the title 16 points back.
        let state = rows.iter().any(|r| !r.status.is_empty()).then_some(if table <= 785.0 { 160.0 } else { 176.0 });
        let project = (table > 785.0 && rows.iter().any(|r| !r.project.is_empty())).then_some(150.0);
        let priority = (table > 915.0 && rows.iter().any(|r| r.priority != Priority::Medium)).then_some(112.0);
        // The rows sit 76 points inside the table, and the title takes what the others leave.
        let taken: f32 = [state, project, priority].into_iter().flatten().map(|w| w + GAP).sum();
        Columns { title: (table - 76.0 - WHEN - GAP - taken).max(0.0), state, project, priority }
    }

    /// One line of the grid: the title's cell, the middle cells that are drawn, the time's.
    fn line(&self, title: El, state: El, project: El, priority: El, when: El) -> El {
        let mut e = row().h(20.0).gap(GAP).child(title.w(self.title));
        for (width, cell) in [(self.state, state), (self.project, project), (self.priority, priority)] {
            if let Some(width) = width {
                e = e.child(cell.w(width));
            }
        }
        e.child(when.w(WHEN).justify_end())
    }
}

/// A column's name, at the head of the table.
fn heading(s: &str) -> El {
    row().child(capitals(s, 10.5, 20.0, 0.84, muted()))
}

/// The marks beside a priority's word: one to four bars, the strength filled in.
fn priority_mark(p: Priority) -> Option<El> {
    let off = rgb(82, 80, 79);
    match p {
        Priority::Urgent => Some(bars(&[4.0, 7.0, 10.0, 13.0], ink())),
        Priority::High => Some(bars(&[4.0, 7.5, 11.0], ink())),
        Priority::Low => Some(row().items_end().gap(1.5).child(el().size(3.0, 4.0).radius(1.0).fill(dim())).child(el().size(3.0, 7.5).radius(1.0).fill(off)).child(el().size(3.0, 11.0).radius(1.0).fill(off))),
        Priority::Medium => None,
    }
}

fn priority_word(p: Priority) -> &'static str {
    match p {
        Priority::Urgent => "Urgent",
        Priority::High => "High",
        Priority::Low => "Low",
        Priority::Medium => "",
    }
}

/// Midnight today on the person's own clock, in milliseconds.
fn start_of_day(now: f64) -> f64 {
    let t = LocalTime::at(now as i64);
    let midnight = now - f64::from(t.hour * 3600 + t.minute * 60 + t.second) * 1000.0 - now.rem_euclid(1000.0);
    // A day the clocks changed on is an hour longer or shorter than its face says.
    midnight + f64::from(t.offset - LocalTime::at(midnight as i64).offset) * 1000.0
}

/// When a thread was sent, in plain words: "just now", "6 min ago", "4 hours ago", "Yesterday",
/// "Sep 28" (`updatedWords` in `renderer/src/threads/page-rules.ts`).
fn sent_words(at: f64, now: f64) -> String {
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let min = ((now - at) / 60_000.0).floor().max(0.0) as i64;
    if min < 1 {
        return "just now".into();
    }
    if min < 60 {
        return format!("{min} min ago");
    }
    let today = start_of_day(now);
    if at >= today {
        let hours = min / 60;
        return format!("{hours} hour{} ago", if hours == 1 { "" } else { "s" });
    }
    if at >= today - 86_400_000.0 {
        return "Yesterday".into();
    }
    let day = LocalTime::at(at as i64);
    format!("{} {}", MONTHS[(day.month as usize).clamp(1, 12) - 1], day.day)
}

/// One thread: 54 points, a hairline above every row but the first, and the title's underline,
/// which is how a row answers: orange on the selected one, faint under the pointer.
fn thread(r: &Row, at: usize, cols: &Columns, now: f64) -> El {
    let selected = at == 0;
    // An unread thread's title is heavier, and a square in the gutter says so too.
    let words = pen(text(r.title.as_str()).font(TextStyle::new(14.0, if r.unread { 530.0 } else { 400.0 }).lh(20.0).tracking(-0.14)).color(ink()).truncate(), 14.0);
    // The line is the title's own, so it ends where the words do.
    let line = el().pin(Some(0.0), None, Some(0.0), Some(0.0)).h(1.0);
    let line = if selected { line.fill(rgba(238, 96, 24, 0.6)) } else { line.fill(rgb(82, 80, 79)).group_opacity(0.0, 1.0) };
    // Every title starts after a face in a slot of its own, so the titles line up.
    let face = row().size(18.0, 18.0).radius(9.0).center().border(1.0, rgba(239, 235, 231, 0.12)).child(icon(&icons::AGENT_EYES, 16.0).color(dim()));
    let mut title = row().items_center().child(face.mr(10.0)).child(el().shrink().child(words).child(line));
    if r.pinned {
        // A thread kept at the top wears a small pin after its words, 3 points under their line.
        title = title.child(el().ml(9.0).child(icon(&icons::PIN, 13.0).color(muted()).mt(3.0)));
    }
    let small = |s: &str, c: Color| row().items_center().child(capitals(s, 11.5, 20.0, 0.575, c).truncate());
    let mut priority = row().items_center();
    if let Some(mark) = priority_mark(r.priority) {
        let c = if r.priority == Priority::Urgent { accent() } else { muted() };
        priority = priority.child(mark.mr(8.0)).child(capitals(priority_word(r.priority), 11.5, 20.0, 0.575, c));
    }
    let when = row().child(pen(text(r.at.map(|at| sent_words(at, now)).unwrap_or_default()).font(TextStyle::new(13.0, 400.0).lh(20.0)).color(dim()), 13.0));
    let (id, product) = (r.id.clone(), r.product.clone());
    let mut e = row()
        .key(format!("thread-{}-{}", r.product, r.id))
        .h(54.0)
        .pt(19.0)
        .pr(32.0)
        .pl(44.0)
        .hover_group()
        .cursor(Cursor::Pointer)
        .on_press(move |s: &mut State| s.go(View::Thread { id: id.clone(), product: product.clone() }))
        .child(cols.line(title, small(&r.status, dim()), small(&r.project, muted()), priority, when));
    if at > 0 {
        e = e.child(el().pin(Some(20.0), Some(0.0), Some(32.0), None).h(1.0).fill(rgba(70, 68, 67, 0.375)));
    }
    if r.unread {
        e = e.child(el().absolute(13.0, 27.0).size(6.0, 6.0).fill(accent()));
    }
    e
}

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
    // Each tab's rows, what its time column is called, and its words when it has none
    // (`emptyTabWords`). The web app's empty Inbox is a page of its own, a sky over the whole card
    // and a box to write in, which this window does not draw while it only reads.
    let (rows, sent, nothing) = match s.inbox_tab {
        1 => (&s.lists.progress, "Sent", match s.lists.needs.len() {
            0 => "Nothing is running.".to_string(),
            1 => "Nothing is running. One thread waits in Needs you.".to_string(),
            n => format!("Nothing is running. {n} threads wait in Needs you."),
        }),
        2 => (&s.lists.done, "Done", "Nothing closed yet. A finished thread waits in Needs you until you close it.".to_string()),
        _ => (&s.lists.needs, "Sent", "Nothing needs you.".to_string()),
    };
    let page = col().w(page_w).child(tabs(s));
    if rows.is_empty() {
        return page.child(empty(&nothing, s));
    }
    let cols = Columns::of(rows, page_w);
    let head = row().h(38.0).pt(10.0).pr(32.0).pl(44.0).child(cols.line(heading("Thread"), heading("Status"), heading("Project"), heading("Priority"), heading(sent)));
    page.child(head).children(rows.iter().enumerate().map(|(i, r)| thread(r, i, &cols, s.now)))
}
