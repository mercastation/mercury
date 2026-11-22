//! Home: the greeting, the two doors, the box to start a thread (`compose.rs`), and what needs
//! you beside what is in progress. Built from the web app's own measurements
//! (`renderer/src/home`).

use rune::{Cursor, El, Icon, ScrollSpec, TextCase, TextStyle, col, el, icon, row, text};

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
fn give_way(room: f32, above: f32, box_h: f32) -> (f32, f32, f32) {
    // Each part that can give: its height, how readily it gives, and the least it can be.
    let parts = [(above, 1000.0, 0.0), (box_h, 1.0, 0.0), (CARDS_TALL, 100.0, CARDS_LEAST)];
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

/// `.hm-glass`: a faint white wash over the page. The web app also blurs the page behind it;
/// Rune blurs four surfaces a frame (`MAX_BACKDROP_BLURS`) and Home has five, so the three
/// cards take the blur, where the sky's streak runs, and the two small doors go without.
/// A door with no count (Team) is its icon and its name. Under the pointer the door is whole
/// where it stood at .9, and its edge brighter (`.hm-door:hover`): its look is a layer the
/// door's hover lifts, and its translucent edge two, the one at rest giving way to the other.
fn door(glyph: &'static Icon, name: &str, count: Option<usize>, hot: bool) -> El {
    // A Rune that blurs five or more gives the doors theirs back with no change here.
    let glass = if rune::MAX_BACKDROP_BLURS >= 5 { row().backdrop_blur(20.0) } else { row() };
    let face = glass
        .h(32.0)
        .items_center()
        .gap(8.0)
        .pl(11.0)
        .pr(12.0)
        .radius(8.0)
        .fill(white(0.043))
        .border(1.0, rgba(0, 0, 0, 0.0))
        .border_layout()
        .box_shadow(shadow(0.0, 1.0, 0.0, 0.0, white(0.06), true))
        .group_opacity(0.9, 1.0)
        .children(feel::edges(8.0, white(0.15), white(0.26), true))
        .child(icon(glyph, 14.0).color(ink()).opacity(0.65))
        .child(label(name, 13.5, 400.0, ink()));
    let face = match count {
        Some(n) => face.child(text(n.to_string()).font(figures(12.0, 400.0)).color(if hot { ink() } else { muted() })),
        None => face,
    };
    row().hover_group().cursor(Cursor::Pointer).child(face)
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
/// pointing hand nor a look under the pointer (`:disabled`).
pub fn send_main(ready: bool) -> El {
    let send = row()
        .key("send")
        .h(32.0)
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

/// The caret beside Send, for sending later.
pub fn send_caret() -> El {
    let caret = row()
        .size(32.0, 32.0)
        .items_center()
        .pl(9.0)
        .corners([0.0, 6.0, 6.0, 0.0])
        .fill(send_fill())
        .border_sides([0.0, 0.0, 0.0, 1.0], rgba(26, 24, 23, 0.55))
        .border_layout()
        .child(icon(&icons::CARET, 13.0).color(ink()).group_color(white(1.0)));
    brightened(caret).cursor(Cursor::Pointer)
}

/// The paper clip beside Send, the text's own colour under the pointer (`.tc-icon:hover`).
pub fn attach() -> El {
    row().size(30.0, 30.0).radius(6.0).center().hover_group().cursor(Cursor::Pointer).child(icon(&icons::ATTACH, 16.0).color(muted()).group_color(ink()))
}

fn priority_mark(p: Priority) -> Option<El> {
    match p {
        Priority::Urgent => Some(bars(&[4.0, 7.0, 10.0, 13.0], ink())),
        Priority::High => Some(bars(&[4.0, 7.5, 11.0], ink())),
        Priority::Low => Some(row().items_end().gap(1.5).child(el().size(3.0, 4.0).radius(1.0).fill(dim())).child(el().size(3.0, 7.5).radius(1.0).fill(rgb(82, 80, 79))).child(el().size(3.0, 11.0).radius(1.0).fill(rgb(82, 80, 79)))),
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
    let title = text(r.title.as_str()).font(TextStyle::new(14.5, if r.unread && card == Card::Needs { 500.0 } else { 400.0 }).lh(20.0)).color(ink()).truncate();
    let under = match card {
        Card::Needs => {
            let mut tags = row().mt(3.0).h(16.0).items_center().gap(10.0);
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
            if !r.project.is_empty() {
                tags = tags.child(text(r.project.as_str()).font(mono(10.5, 16.0).tracking(0.525)).text_case(TextCase::Uppercase).color(muted()));
            }
            tags
        }
        // Who is working on it and where, in one faint line (`.hm-text small`).
        Card::Progress => row().child(pen(text(format!("{} · {}", r.engine, r.project)).font(TextStyle::new(12.5, 400.0).lh(17.0)).color(dim()).truncate(), 12.5)),
    };
    let mut face = row().size(28.0, 28.0).radius(14.0).center().fill(white(0.12)).child(label(r.letter, 12.0, 500.0, ink()));
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

/// One of Home's two cards: a head with a link out, and what is in it. The link is the text's
/// own colour under the pointer (`.hm-link:hover`).
fn home_card(title: &str, count: Option<usize>, link: &str, to: Option<Tab>, body: El, height: f32) -> El {
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
    glass(col())
        .h(height)
        .radius(10.0)
        .clip_rounded(10.0)
        .border(1.0, white(0.13))
        .border_layout()
        .box_shadow(shadow(0.0, 1.0, 0.0, 0.0, white(0.16), true))
        .box_shadow(shadow(0.0, 12.0, 40.0, 0.0, rgba(0, 0, 0, 0.3), false))
        .child(head)
        .child(col().grow(1.0).min_h(0.0).scroll(ScrollSpec { stick_bottom: false }).child(col().pt(4.0).pb(28.0).child(body)))
        // The rows fade out into the card's foot.
        .child(el().pin(Some(1.0), None, Some(1.0), Some(1.0)).h(56.0).gradient(rgba(24, 21, 26, 0.0), rgba(24, 21, 26, 0.9)))
}

pub fn view(s: &State) -> El {
    let (needs, progress) = (&s.lists.needs, &s.lists.progress);
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
        rune::memo::layout_size(crate::compose::BOX).map_or(152.0 + files, |[_, tall]| tall)
    };
    let (above, slot, cards_h) = if stacked { (above, box_h, CARDS_TALL) } else { give_way(COLUMN_TALL.max(page_h - 64.0), above, box_h) };

    // An empty card says so in one line (`HomePage.tsx`), and a head shows its count only above zero.
    let list = |rows: &[Row], empty: &str, card: Card| {
        if rows.is_empty() {
            el().py(14.0).px(18.0).child(label(empty, 13.5, 400.0, dim()))
        } else {
            // Only Needs you is walked by the keys.
            col().children(rows.iter().enumerate().map(|(i, r)| task_row(r, i, card == Card::Needs && s.home_pick == Some(i), card)))
        }
    };
    // Home shows the first eighty that need the person and the first sixty in progress (`App.tsx`).
    let rows = list(&needs[..needs.len().min(80)], "Nothing needs you.", Card::Needs);
    let working = list(&progress[..progress.len().min(60)], "No agent is working right now.", Card::Progress);
    let count = |n: usize| (n > 0).then_some(n);
    // With teammates the second card is the team's week, and Team is a third door (`HomePage.tsx`).
    let team = window::team_mode(&s.snap);
    let mut doors = row()
        .mt(16.0)
        .gap(8.0)
        // Only the Inbox's count is lit, and only above zero (`.hm-door.hot`).
        .child(door(&icons::DOOR_INBOX, "Inbox", Some(needs.len()), !needs.is_empty()).on_press(door_to(Some(Tab::Needs))))
        .child(door(&icons::DOOR_PROGRESS, "In progress", Some(progress.len()), false).on_press(door_to(Some(Tab::Progress))));
    if team {
        doors = doors.child(feel::answers(door(&icons::DOOR_TEAM, "Team", None, false)));
    }
    let second = if team {
        // The team's rows are the Team page's, which this window does not have yet: the card
        // says so when nobody else is on the team, and is bare otherwise.
        let nobody = if window::team_has_others(&s.snap) { el() } else { el().py(14.0).px(18.0).child(label("Nobody else on your team yet.", 13.5, 400.0, dim())) };
        home_card("Team this week", None, "Open Team →", None, nobody, cards_h)
    } else {
        home_card("In progress", count(progress.len()), "Open →", Some(Tab::Progress), working, cards_h)
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
            col().mt(24.0).gap(22.0).child(home_card("Needs you", count(needs.len()), "Open inbox →", Some(Tab::Needs), rows, cards_h).w(column)).child(second.w(column))
        } else {
            row()
                .mt(24.0)
                .gap(22.0)
                // `grid-template-columns: 1.35fr 1fr`: the shares are of the row less its gap.
                .child(home_card("Needs you", count(needs.len()), "Open inbox →", Some(Tab::Needs), rows, cards_h).w((column - 22.0) * 1.35 / 2.35))
                .child(second.w((column - 22.0) / 2.35))
        });
    row().w_full().justify_center().child(page)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_taller_box_takes_its_room_from_above_the_greeting_first_and_then_from_the_cards() {
        let near = |a: (f32, f32, f32), b: (f32, f32, f32)| (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01 && (a.2 - b.2).abs() < 0.01;
        // A window 760 points tall: one line of words leaves everything where it was.
        assert_eq!(give_way(660.0, 44.4, 152.0), (44.4, 152.0, 300.0));
        // A file's chip (the web window draws the greeting at 171.27, the box 187.49 tall and the
        // cards 299.24) and a picture (135.29, 247.79 and 274.92).
        assert!(near(give_way(660.0, 44.4, 187.5), (43.27, 187.49, 299.24)), "{:?}", give_way(660.0, 44.4, 187.5));
        assert!(near(give_way(660.0, 44.4, 248.0), (7.29, 247.79, 274.92)), "{:?}", give_way(660.0, 44.4, 248.0));
        // Far too tall: the room above is gone, the cards stop at 240, and the rest is the box's.
        let squeezed = give_way(660.0, 44.4, 500.0);
        assert_eq!((squeezed.0, squeezed.2), (0.0, 240.0));
        // A window with room to spare gives nothing up.
        assert_eq!(give_way(810.0, 66.0, 248.0), (66.0, 248.0, 300.0));
    }
}
