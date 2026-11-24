//! Home: the greeting, the two doors, the box to start a thread (`compose.rs`), and what needs
//! you beside what is in progress. Built from the web app's own measurements
//! (`renderer/src/home`).

use rune::{Cursor, El, Icon, ScrollSpec, TextStyle, col, el, icon, row, text};

use crate::data::{Priority, Row, Tab};
use crate::rules::window;
use crate::theme::*;
use crate::{State, View, feel, icons};

/// The page's column: 760 points, or what the card leaves it.
const COLUMN: f32 = 760.0;
/// How tall the column is at least (`.hm-col { min-height: 660px }`), and what Home's two cards
/// are, at most and at least (`.hm-two`).
const COLUMN_TALL: f32 = 660.0;
const CARDS_TALL: f32 = 300.0;
const CARDS_LEAST: f32 = 240.0;
/// The greeting's line, the doors under it with their margin, and the margins over the box and
/// over the cards: what in the column never gives.
const FIXED: f32 = 36.0 + 16.0 + 32.0 + 22.0 + 24.0;

/// What the column gives up when the box in it grows past the room it has, as `home.css` lays
/// it out: the room above the greeting first, a thousand times more readily than the cards,
/// which give up to 240 points a hundred times more readily than the box (CSS flexbox's
/// shrinking, with those weights times each one's own height). `room` is how tall the column
/// is, `above` the room over the greeting, `box_h` the box. Gives the room above, the box's
/// slot and the cards' height. The box gives a sliver of a point until the other two are at
/// their least, and its slot is that much shorter with the box itself whole; past that, where
/// the web window's box would scroll its words inside itself, this page scrolls instead.
/// `fitted`: the cards hold three rows or fewer and are as tall as those (`.hm-two.fits`,
/// `flex: none`), which they never give up.
fn give_way(room: f32, above: f32, box_h: f32, fitted: Option<f32>) -> (f32, f32, f32) {
    // Each part that can give: its height, how readily it gives, and the least it can be.
    let cards = fitted.map_or((CARDS_TALL, 100.0, CARDS_LEAST), |tall| (tall, 0.0, tall));
    let parts = [(above, 1000.0, 0.0), (box_h, 1.0, 0.0), cards];
    let mut size = parts.map(|(basis, _, _)| basis);
    let mut held = [false; 3];
    loop {
        // What is over, with each part already at its least counted at that.
        let over = FIXED - room + (0..3).map(|i| if held[i] { size[i] } else { parts[i].0 }).sum::<f32>();
        let weights: f32 = (0..3).filter(|&i| !held[i]).map(|i| parts[i].1 * parts[i].0).sum();
        if over <= 0.0 || weights <= 0.0 {
            break;
        }
        let mut more = false;
        for i in 0..3 {
            if held[i] {
                continue;
            }
            let wanted = parts[i].0 - over * parts[i].1 * parts[i].0 / weights;
            size[i] = wanted.max(parts[i].2);
            // One that would go under its least stays there, and the rest is spread again.
            if size[i] > wanted {
                held[i] = true;
                more = true;
            }
        }
        if !more {
            break;
        }
    }
    (size[0], size[1], size[2])
}

/// Home's words, each standing where Chromium puts its glyphs to a quarter pixel (`theme::pen`).
fn label(s: &str, size: f32, weight: f32, c: rune::Color) -> El {
    pen(crate::theme::label(s, size, weight, c), size)
}

/// The greeting follows the clock and names whoever is signed in, as
/// `renderer/src/home/home-model.ts` has it.
fn greeting(now_ms: f64, name: Option<&str>) -> String {
    let part = match rune::time::LocalTime::at(now_ms as i64).hour {
        4..=11 => "Good morning",
        12..=17 => "Good afternoon",
        _ => "Good evening",
    };
    name.map_or(part.to_string(), |first| format!("{part}, {first}"))
}

/// `.hm-glass`: a faint white wash over the page, which the web app also blurs behind it. The
/// blur is the soft sky shown where the door stands, as the cards' is (`glass`): it is under
/// the door's face, and fades with it to the nine tenths the door rests at. Rune's own blur
/// could not give the doors theirs, at four surfaces a frame with Home holding five.
/// A door with nothing behind it shows no number (`doors` in `home-model.ts`), nor does Team.
/// Under the pointer the door is whole where it stood at .9, and its edge brighter
/// (`.hm-door:hover`): its look is a layer the door's hover lifts, and its translucent edge two,
/// the one at rest giving way to the other. Its corner is the small one things pressed wear
/// (`--artifact-corner-small`).
fn door(s: &State, key: &'static str, glyph: &'static Icon, name: &str, count: Option<usize>, hot: bool) -> El {
    let behind = crate::chrome::sky_through(key, &s.sky_home_soft, crate::chrome::sky_in_the_window(s), s.sky_home.size(), crate::chrome::SOFT_HOME, 6.0, 0.0);
    let face = row()
        .h(32.0)
        .items_center()
        .gap(8.0)
        .pl(11.0)
        .pr(12.0)
        .radius(6.0)
        .fill(white(0.043))
        .border(1.0, rgba(0, 0, 0, 0.0))
        .border_layout()
        .box_shadow(shadow(0.0, 1.0, 0.0, 0.0, white(0.06), true))
        .group_opacity(0.9, 1.0)
        .children(feel::edges(6.0, white(0.15), white(0.26), true))
        .child(icon(glyph, 16.0).color(ink()).opacity(0.65))
        .child(label(name, 13.5, 400.0, ink()));
    let face = match count {
        Some(n) => face.child(text(n.to_string()).font(figures(12.0, 400.0)).color(if hot { ink() } else { muted() })),
        None => face,
    };
    row().key(key).hover_group().cursor(Cursor::Pointer).children(behind.map(|sky| sky.group_opacity(0.9, 1.0))).child(face)
}

/// One of the small framed words under a box to write in (`.tc-chip`): here and in a
/// conversation's reply box. Under the pointer its edge is `--line-strong` and its words the
/// text's own colour (`.tc-chip:hover`, `.dock-chips .compose-word:hover`), which they take with
/// `group_color(ink())`.
pub fn chip(children: Vec<El>) -> El {
    row()
        .h(28.0)
        .items_center()
        .gap(8.0)
        .px(10.0)
        .radius(6.0)
        .fill(white(0.03))
        .border(1.0, edge())
        .border_layout()
        .hover_group()
        .cursor(Cursor::Pointer)
        .children(children)
        .child(feel::edge_lit(6.0, rule(), true))
}

/// `--send-fill`: the text colour at 15%.
fn send_fill() -> rune::Color {
    rgba(239, 235, 231, 0.15)
}

/// Send, and the caret beside it, under the pointer: `filter: brightness(1.15)`
/// (`.tc-send > button:hover:not(:disabled)`, `.dock-send-pair > .dock-send:hover`), which takes
/// their words to white and their fill, the text's colour at 15%, two levels lighter over the
/// box's dark glass, which this much white laid over it gives. Rune has no brightness filter.
pub fn brightened(e: El) -> El {
    e.hover_group().hover(white(0.013))
}

/// Send with its key. Faint until there is something to send, when it takes neither the
/// pointing hand nor a look under the pointer (`:disabled`). A conversation's reply box has it 32
/// points tall (`.dock-send-pair`).
pub fn send_main(ready: bool) -> El {
    send_main_at(ready, 32.0)
}

/// Send, `tall` points tall: the box a thread is started in has it 28 (`.tc-send`).
pub fn send_main_at(ready: bool, tall: f32) -> El {
    let send = row()
        .key("send")
        .h(tall)
        .items_center()
        .gap(9.0)
        .px(14.0)
        .corners([6.0, 0.0, 0.0, 6.0])
        .fill(send_fill())
        .opacity(if ready { 1.0 } else { 0.45 })
        .child(label("Send", 13.5, 500.0, ink()).group_color(white(1.0)))
        .child(text("⌘↵").font(mono(11.0, 14.0)).color(ink()).group_color(white(1.0)).opacity(0.75));
    if ready { brightened(send).cursor(Cursor::Pointer) } else { send.cursor(Cursor::Default) }
}

/// Send while the files a reply carries are being saved: off, and saying so in place of its
/// word and keys (`saving ? 'Sending…'` on `.dock-send`).
pub fn send_sending() -> El {
    row()
        .key("send")
        .h(32.0)
        .items_center()
        .px(14.0)
        .corners([6.0, 0.0, 0.0, 6.0])
        .fill(send_fill())
        .opacity(0.45)
        .child(label("Sending…", 13.5, 500.0, ink()))
        .cursor(Cursor::Default)
}

/// The caret beside Send, for sending later.
pub fn send_caret() -> El {
    send_caret_at(32.0, false)
}

/// The caret, `tall` points tall. On the box a thread is started in it dims with Send while
/// there is nothing to send, so the two read as one button (`.tc-send:has(> .tc-send-main:disabled)
/// > .tc-send-caret`), and still opens its menu.
pub fn send_caret_at(tall: f32, dimmed: bool) -> El {
    let caret = row()
        .size(32.0, tall)
        .items_center()
        .pl(9.0)
        .corners([0.0, 6.0, 6.0, 0.0])
        .fill(send_fill())
        .border_sides([0.0, 0.0, 0.0, 1.0], page_at(0.55))
        .border_layout()
        .opacity(if dimmed { 0.45 } else { 1.0 })
        .child(icon(&icons::CARET, 13.0).color(ink()).group_color(white(1.0)));
    brightened(caret).cursor(Cursor::Pointer)
}

/// The paper clip beside Send, the text's own colour under the pointer (`.tc-icon:hover`).
pub fn attach() -> El {
    attach_at(30.0)
}

/// The paper clip in a square `side` points wide: 28 on the box a thread is started in.
pub fn attach_at(side: f32) -> El {
    row().size(side, side).radius(6.0).center().hover_group().cursor(Cursor::Pointer).child(icon(&icons::ATTACH, 16.0).color(muted()).group_color(ink()))
}

fn priority_mark(p: Priority) -> Option<El> {
    match p {
        Priority::Urgent => Some(bars(&[4.0, 7.0, 10.0, 13.0], ink())),
        Priority::High => Some(bars(&[4.0, 7.5, 11.0], ink())),
        Priority::Low => Some(row().items_end().gap(1.5).child(el().size(3.0, 4.0).radius(1.0).fill(dim())).child(el().size(3.0, 7.5).radius(1.0).fill(rule())).child(el().size(3.0, 11.0).radius(1.0).fill(rule()))),
        Priority::Medium => None,
    }
}

/// Which of Home's two cards a row is on: they share the frame and differ under the title.
#[derive(Clone, Copy, PartialEq)]
enum Card {
    Needs,
    Progress,
}

/// `on` is the row the keys are on, which wears what the pointer's does (`.hm-row.on`). The
/// pointer coming onto a row of Needs you puts the keys there (`onMouseEnter={() => setPick(i)}`
/// in `HomePage.tsx`), so that row is lit by being the keys', and stays the keys' when the
/// pointer leaves; a row In progress is only washed while the pointer is on it.
fn task_row(r: &Row, at: usize, on: bool, card: Card) -> El {
    let first = at == 0;
    // A thread not yet read has its title a step heavier (`.hm-row.unread .hm-text b`).
    let title = text(r.title.as_str()).font(TextStyle::new(14.0, if r.unread && card == Card::Needs { 500.0 } else { 400.0 }).lh(20.0)).color(ink()).truncate();
    let under = match card {
        Card::Needs => {
            // The facts under the title, on the second line's own 17 points (`.hm-tags`).
            let mut tags = row().h(17.0).items_center().gap(10.0);
            // The mark's title is its level's word (`.hm-prio`'s `title`).
            if let Some(mark) = priority_mark(r.priority) {
                let word = match r.priority {
                    Priority::Urgent => "Urgent",
                    Priority::High => "High",
                    Priority::Low => "Low",
                    Priority::Medium => "Medium",
                };
                tags = tags.child(mark.help(word));
            }
            // The project as the line's own faint words, no longer the Inbox's small capitals
            // (`.hm-tag`), cut short before the mark is.
            if !r.project.is_empty() {
                tags = tags.child(row().min_w(0.0).child(pen(text(r.project.as_str()).font(TextStyle::new(12.5, 400.0).lh(17.0)).color(muted()).truncate(), 12.5)));
            }
            tags
        }
        // Who is working on it and where, in one faint line (`.hm-text small`).
        Card::Progress => row().child(pen(text(format!("{} · {}", r.engine, r.project)).font(TextStyle::new(12.5, 400.0).lh(17.0)).color(dim()).truncate(), 12.5)),
    };
    // An agent's thread wears the app's agent face, two eyes in a faint ring, where a letter in
    // a circle read as a close button (`.hm-face.hm-agent`): 28 points, its eyes 22.
    let mut face = row().size(28.0, 28.0).radius(14.0).center().border(1.0, rgba(239, 235, 231, 0.12)).child(icon(&icons::AGENT_EYES, 22.0).color(dim()));
    if card == Card::Progress {
        // The green dot of a thread an agent is on (`.hm-face.live::after`), ringed in the card's dark.
        face = face.child(el().absolute(21.0, 21.0).size(8.0, 8.0).radius(4.0).fill(rgb(0x8f, 0xd1, 0x8f)).box_shadow(shadow(0.0, 0.0, 0.0, 2.0, rgba(18, 16, 18, 0.95), false)));
    }
    let mut line = row().key(format!("{}-{}", if card == Card::Needs { "need" } else { "progress" }, r.id)).items_center().gap(12.0).py(10.0).px(18.0).cursor(Cursor::Pointer);
    line = match card {
        Card::Needs => line.on_hover(move |s: &mut State| s.home_pick = Some(at)),
        Card::Progress => line.hover(white(0.045)),
    };
    if !first {
        // The CSS says .065; the browser keeps alpha in 255ths, which lands it on 17.
        line = line.border_sides([1.0, 0.0, 0.0, 0.0], white(17.0 / 255.0)).border_layout();
    }
    if on {
        line = line.fill(white(0.045));
    }
    let (id, product) = (r.id.clone(), r.product.clone());
    line.on_press(move |s: &mut State| s.open_thread(&id, &product))
        .child(face)
        .child(col().grow(1.0).min_w(0.0).child(title).child(under))
        // The age in the Inbox's own words, "6 min ago" (`updatedWords`).
        .child(label(&r.when, 12.5, 400.0, dim()))
}

/// Where a door or a card's link goes: the Inbox on one of its tabs (`onDoor` in `App.tsx`), or a
/// page this window does not have yet, which a press goes nowhere from.
fn door_to(tab: Option<Tab>) -> impl Fn(&mut State) + 'static {
    move |s: &mut State| {
        if let Some(tab) = tab {
            s.go(View::Inbox);
            s.show_tab(tab);
        }
    }
}

/// How tall a card is that fits its rows (`.hm-two.fits .hm-card`, its height its own): its
/// edge, its head, and its rows four points clear of the head and of its foot, the first row 57
/// points and each after it 58 with the hairline over it; with no rows, the line that says so.
fn fitted(rows: usize) -> f32 {
    let body = if rows == 0 { 45.5 } else { 57.0 + 58.0 * (rows - 1) as f32 };
    2.0 + 44.0 + 4.0 + body + 4.0
}

/// The two cards, by the names their glass is laid out under.
const NEEDS_CARD: &str = "home-needs";
const SECOND_CARD: &str = "home-second";

/// A pane of Home's glass, with corners of `radius`: the sky behind it blurred, under the
/// glass's dark wash (`.hm-glass`, the composer). `pane` makes the pane of the glass it is
/// handed, and what comes back is the pane on the place it stands, keyed `key`.
///
/// The blur is the soft sky, painted with the sky and shown where the pane stands
/// (`chrome::sky_through`), so nothing on Home asks Rune to blur what is behind it. It is a
/// thing of its own under the pane, which is why the place is one element and the pane
/// another: the pane's wash, its edge and the light along its top are then drawn over the
/// blurred sky, in the order the web window draws them. The one build before the place has
/// been laid out has the wash alone, and is never drawn.
pub fn glass(s: &State, key: &'static str, radius: f32, pane: impl FnOnce(El) -> El) -> El {
    let behind = crate::chrome::sky_through(key, &s.sky_home_soft, crate::chrome::sky_in_the_window(s), s.sky_home.size(), crate::chrome::SOFT_HOME, radius, 0.0);
    col().key(key).children(behind).child(pane(glass_wash(col())))
}

/// One of Home's two cards: a head with a link out, and what is in it. The link is the text's
/// own colour under the pointer (`.hm-link:hover`). Holding more rows than three, the rows fade
/// out into its foot; with three or fewer it is as tall as they are and has no fade
/// (`.hm-two.fits`).
fn home_card(s: &State, key: &'static str, title: &str, count: Option<usize>, link: &str, to: Option<Tab>, body: El, height: f32, fits: bool) -> El {
    let mut name = row().items_center().child(label(title, 14.0, 500.0, ink()));
    if let Some(n) = count {
        name = name.child(text(n.to_string()).font(figures(14.0, 400.0)).color(muted()).ml(8.0));
    }
    let head = row()
        .pt(13.0)
        .px(18.0)
        .pb(12.0)
        .items_center()
        .justify_between()
        .border_sides([0.0, 0.0, 1.0, 0.0], white(0.07))
        .border_layout()
        .child(name)
        .child(row().hover_group().cursor(Cursor::Pointer).child(label(link, 13.0, 400.0, dim()).group_color(ink())).on_press(door_to(to)));
    glass(s, key, 10.0, |glass| {
        let card = glass
            .h(height)
            .radius(10.0)
            .clip_rounded(10.0)
            .border(1.0, white(0.13))
            .border_layout()
            .box_shadow(shadow(0.0, 1.0, 0.0, 0.0, white(0.16), true))
            .box_shadow(shadow(0.0, 12.0, 40.0, 0.0, rgba(0, 0, 0, 0.3), false))
            .child(head)
            .child(col().grow(1.0).min_h(0.0).scroll(ScrollSpec { stick_bottom: false }).child(col().pt(4.0).pb(if fits { 4.0 } else { 28.0 }).child(body)));
        if fits {
            return card;
        }
        // The rows fade out into the card's foot.
        card.child(el().pin(Some(1.0), None, Some(1.0), Some(1.0)).h(56.0).gradient(rgba(24, 21, 26, 0.0), rgba(24, 21, 26, 0.9)))
    })
}

pub fn view(s: &State) -> El {
    let (needs, progress) = (&s.lists.needs, &s.lists.progress);
    // In progress lists the whole tab under a number that counts what the Inbox's filters keep
    // (`homeProgress`, `progressCount`); Needs you lists only what they keep.
    let working_count = s.lists.count(Tab::Progress);
    // What the card leaves the column (`home.css`). The room above the greeting grows with the
    // window, `clamp(0px, 9vh - 24px, 80px)`, and the two cards are 300 points tall however
    // tall the window is: the column is never shorter than 660, so with the box one line tall
    // nothing in it has to give. Files in the box, or words on many lines, make it taller.
    let (page_w, page_h) = s.page_size();
    let column = COLUMN.min((page_w - 64.0).max(320.0));
    let above = (0.09 * s.size.1 - 24.0).clamp(0.0, 80.0);
    // A window 980 points wide or less stacks the cards, each as wide as the column, and then
    // nothing gives: the page scrolls.
    let stacked = s.size.0 <= 980.0;
    // The box as it is laid out, its slot empty while the card is up; before it ever was, one
    // line of words and its files on one line.
    let box_h = if s.compose.card {
        0.0
    } else {
        let files = if s.compose.files.is_empty() { 0.0 } else { 12.0 + crate::attach::rows_tall(&s.compose.files) };
        rune::memo::layout_size(crate::compose::BOX).map_or(148.0 + files, |[_, tall]| tall)
    };
    // Home shows the first eighty that need the person and the first sixty in progress (`App.tsx`).
    let (needs_shown, progress_shown) = (&needs[..needs.len().min(80)], &progress[..progress.len().min(60)]);
    // With teammates the second card is the team's week, and Team is a third door (`HomePage.tsx`).
    let team = window::team_mode(&s.snap);
    // Three rows or fewer in the fuller card and both cards fit their rows (`tallest` in
    // `HomePage.tsx`): side by side they are one height, the fuller card's, and stacked each its
    // own. The team's card counts the team's rows, which this window does not draw.
    let second_rows = if team { window::team_rows(&s.snap) } else { progress_shown.len() };
    let fits = needs_shown.len().max(second_rows).max(1) <= 3;
    let fit_h = fits.then(|| fitted(needs_shown.len()).max(fitted(second_rows)).min(CARDS_TALL));
    let (above, slot, cards_h) = if stacked { (above, box_h, CARDS_TALL) } else { give_way(COLUMN_TALL.max(page_h - 64.0), above, box_h, fit_h) };
    // Stacked, a card that fits is its own height.
    let (needs_h, second_h) = if stacked && fits { (fitted(needs_shown.len()), fitted(second_rows)) } else { (cards_h, cards_h) };

    // An empty card says so in one line (`HomePage.tsx`), and a head shows its count only above zero.
    let list = |rows: &[Row], empty: &str, card: Card| {
        if rows.is_empty() {
            el().py(14.0).px(18.0).child(label(empty, 13.5, 400.0, dim()))
        } else {
            // Only Needs you is walked by the keys.
            col().children(rows.iter().enumerate().map(|(i, r)| task_row(r, i, card == Card::Needs && s.home_pick == Some(i), card)))
        }
    };
    let rows = list(needs_shown, "Nothing needs you.", Card::Needs);
    let working = list(progress_shown, "No agent is working right now.", Card::Progress);
    let count = |n: usize| (n > 0).then_some(n);
    let mut doors = row()
        .mt(16.0)
        .gap(8.0)
        // Only the Inbox's count is lit (`.hm-door.hot`), and neither shows a zero.
        .child(door(s, "home-door-inbox", &icons::DOOR_INBOX, "Inbox", count(needs.len()), !needs.is_empty()).on_press(door_to(Some(Tab::Needs))))
        .child(door(s, "home-door-progress", &icons::DOOR_PROGRESS, "In progress", count(working_count), false).on_press(door_to(Some(Tab::Progress))));
    if team {
        doors = doors.child(feel::answers(door(s, "home-door-team", &icons::DOOR_TEAM, "Team", None, false)));
    }
    let second = if team {
        // The team's rows are the Team page's, which this window does not have yet: the card
        // says so when nobody else is on the team, and is bare otherwise.
        let nobody = if window::team_has_others(&s.snap) { el() } else { el().py(14.0).px(18.0).child(label("Nobody else on your team yet.", 13.5, 400.0, dim())) };
        home_card(s, SECOND_CARD, "Team this week", None, "Open Team →", None, nobody, second_h, fits)
    } else {
        home_card(s, SECOND_CARD, "In progress", count(working_count), "Open →", Some(Tab::Progress), working, second_h, fits)
    };

    let page = col()
        .w(column)
        .pt(24.0)
        .pb(40.0)
        .child(el().h(above))
        .child(text(greeting(s.now, window::greeted(&s.snap))).font(TextStyle::new(30.0, 500.0).lh(36.0).tracking(-0.36)).color(ink()))
        .child(doors)
        // The box is drawn in this slot, or in the New thread card while that is up, and then
        // the slot is empty and the cards under it come up (`drawnIn` in `App.tsx`).
        .child(if s.compose.card {
            el().mt(22.0)
        } else if slot < box_h - 0.001 {
            // The box gives its sliver too: what is under it moves up by that, as in the web
            // window, while the box keeps all of itself. A column, which leaves the box its own
            // height: stretched to the slot it could never be measured growing.
            col().mt(22.0).h(slot).child(crate::compose::inline(s).w_full())
        } else {
            el().mt(22.0).child(crate::compose::inline(s).w_full())
        })
        .child(if stacked {
            col().mt(24.0).gap(22.0).child(home_card(s, NEEDS_CARD, "Needs you", count(needs.len()), "Open inbox →", Some(Tab::Needs), rows, needs_h, fits).w(column)).child(second.w(column))
        } else {
            row()
                .mt(24.0)
                .gap(22.0)
                // `grid-template-columns: 1.35fr 1fr`: the shares are of the row less its gap.
                .child(home_card(s, NEEDS_CARD, "Needs you", count(needs.len()), "Open inbox →", Some(Tab::Needs), rows, needs_h, fits).w((column - 22.0) * 1.35 / 2.35))
                .child(second.w((column - 22.0) / 2.35))
        });
    row().w_full().justify_center().child(page)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Home's sky is the one large picture the window paints for itself, and a window spends
    /// its day on conversations: it is held while Home is the page and ten seconds after, let
    /// go then, and painted the same again before Home is next drawn.
    #[test]
    fn homes_sky_is_held_only_while_home_is_the_page_or_was_a_moment_ago() {
        use crate::feel::tests::{run, window};
        let mut rt = window("home-sky", &[]);
        let (sky, soft) = (rt.state.sky_home.rgba(), rt.state.sky_home_soft.rgba());
        let size = rt.state.sky_home.size();
        assert!(size.0 > 1000 && !sky.is_empty() && !soft.is_empty(), "Home opens with its sky: {size:?}");
        // On the Inbox, it stays for ten seconds, in case the person comes straight back.
        rt.update(|s, _| s.go(crate::View::Inbox));
        run(&mut rt, 9.0);
        assert_eq!(rt.state.sky_home.size(), size, "still held nine seconds on");
        // Back on Home and away again: the ten seconds start over.
        rt.update(|s, _| s.go(crate::View::Home));
        run(&mut rt, 2.0);
        rt.update(|s, _| s.go(crate::View::Inbox));
        run(&mut rt, 9.0);
        assert_eq!(rt.state.sky_home.size(), size, "nine seconds after leaving again");
        run(&mut rt, 2.0);
        assert_eq!((rt.state.sky_home.size(), rt.state.sky_home_soft.size()), ((0, 0), (0, 0)), "let go past ten");
        // The Inbox keeps its own band, which it draws.
        assert!(rt.state.sky_inbox.size().0 > 1000 && rt.state.sky_inbox_soft.size().0 > 0);
        // Home again: its sky is there before the page is drawn, the same sky to the byte.
        rt.update(|s, _| {
            s.go(crate::View::Home);
            assert_eq!(s.sky_home.size(), size, "painted as Home becomes the page");
        });
        run(&mut rt, 0.1);
        assert!(rt.state.sky_home.rgba() == sky && rt.state.sky_home_soft.rgba() == soft, "the same sky and the same glass over it");
        // And by Back as by a press on Home.
        rt.update(|s, _| s.go(crate::View::Inbox));
        run(&mut rt, 11.0);
        assert_eq!(rt.state.sky_home.size(), (0, 0));
        rt.update(|s, _| {
            s.back();
            assert_eq!((s.view == crate::View::Home, s.sky_home.size()), (true, size));
        });
    }

    #[test]
    fn a_taller_box_takes_its_room_from_above_the_greeting_first_and_then_from_the_cards() {
        let near = |a: (f32, f32, f32), b: (f32, f32, f32)| (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01 && (a.2 - b.2).abs() < 0.01;
        // A window 760 points tall: one line of words leaves everything where it was, and so does
        // a file's chip, the box 183.5 tall (the web window draws the greeting at 172.4).
        assert_eq!(give_way(660.0, 44.4, 148.0, None), (44.4, 148.0, 300.0));
        assert_eq!(give_way(660.0, 44.4, 183.5, None), (44.4, 183.5, 300.0));
        // A film too big to send, with the line saying why (the greeting at 159.66, the box 206.94
        // tall and the cards 291.4), and a picture (137.66, 243.81 and 276.53).
        assert!(near(give_way(660.0, 44.4, 207.0, None), (31.66, 206.94, 291.4)), "{:?}", give_way(660.0, 44.4, 207.0, None));
        assert!(near(give_way(660.0, 44.4, 244.0, None), (9.66, 243.81, 276.53)), "{:?}", give_way(660.0, 44.4, 244.0, None));
        // Far too tall: the room above is gone, the cards stop at 240, and the rest is the box's.
        let squeezed = give_way(660.0, 44.4, 500.0, None);
        assert_eq!((squeezed.0, squeezed.2), (0.0, 240.0));
        // A window with room to spare gives nothing up.
        assert_eq!(give_way(810.0, 66.0, 248.0, None), (66.0, 248.0, 300.0));
    }

    #[test]
    fn cards_of_three_rows_or_fewer_are_as_tall_as_their_rows_and_never_give_them_up() {
        // The head and edge, then the rows four points clear above and below: one row, three, and
        // the one line of an empty card.
        assert_eq!((fitted(1), fitted(3), fitted(0)), (111.0, 227.0, 99.5));
        // A box grown far past the room takes it from above the greeting alone: the cards stand.
        let squeezed = give_way(660.0, 44.4, 500.0, Some(fitted(2)));
        assert_eq!((squeezed.0, squeezed.2), (0.0, fitted(2)));
        assert_eq!(give_way(660.0, 44.4, 148.0, Some(fitted(3))), (44.4, 148.0, 227.0));
    }
}
