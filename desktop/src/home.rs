//! Home: the greeting, the two doors, the box to start a thread (`compose.rs`), and what needs
//! you beside what is in progress. Built from the web app's own measurements
//! (`renderer/src/home`).

use rune::{El, Icon, ScrollSpec, TextCase, TextStyle, col, el, icon, row, text};

use crate::data::{Priority, Row};
use crate::rules::window;
use crate::theme::*;
use crate::{State, View, icons};

/// The page's column: 760 points, or what the card leaves it.
const COLUMN: f32 = 760.0;

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
/// A door with no count (Team) is its icon and its name.
fn door(glyph: &'static Icon, name: &str, count: Option<usize>, hot: bool) -> El {
    // A Rune that blurs five or more gives the doors theirs back with no change here.
    let glass = if rune::MAX_BACKDROP_BLURS >= 5 { row().backdrop_blur(20.0) } else { row() };
    let door = glass
        .h(32.0)
        .items_center()
        .gap(8.0)
        .pl(11.0)
        .pr(12.0)
        .radius(8.0)
        .fill(white(0.043))
        .border(1.0, white(0.15))
        .border_layout()
        .box_shadow(shadow(0.0, 1.0, 0.0, 0.0, white(0.06), true))
        .opacity(0.9)
        .child(icon(glyph, 14.0).color(ink()).opacity(0.65))
        .child(label(name, 13.5, 400.0, ink()));
    match count {
        Some(n) => door.child(text(n.to_string()).font(figures(12.0, 400.0)).color(if hot { ink() } else { muted() })),
        None => door,
    }
}

/// One of the small framed words under a box to write in (`.tc-chip`): here and in a
/// conversation's reply box.
pub fn chip(children: Vec<El>) -> El {
    row().h(28.0).items_center().gap(8.0).px(10.0).radius(6.0).fill(white(0.03)).border(1.0, edge()).border_layout().children(children)
}

/// `--send-fill`: the text colour at 15%.
fn send_fill() -> rune::Color {
    rgba(239, 235, 231, 0.15)
}

/// Send with its key. Faint until there is something to send.
pub fn send_main(ready: bool) -> El {
    row()
        .key("send")
        .h(32.0)
        .items_center()
        .gap(9.0)
        .px(14.0)
        .corners([6.0, 0.0, 0.0, 6.0])
        .fill(send_fill())
        .opacity(if ready { 1.0 } else { 0.45 })
        .child(label("Send", 13.5, 500.0, ink()))
        .child(text("⌘↵").font(mono(11.0, 14.0)).color(ink()).opacity(0.75))
}

/// The caret beside Send, for sending later.
pub fn send_caret() -> El {
    row()
        .size(32.0, 32.0)
        .items_center()
        .pl(9.0)
        .corners([0.0, 6.0, 6.0, 0.0])
        .fill(send_fill())
        .border_sides([0.0, 0.0, 0.0, 1.0], rgba(26, 24, 23, 0.55))
        .border_layout()
        .child(icon(&icons::CARET, 13.0).color(ink()))
}

/// The two as a conversation's reply box draws them: one pair, pressed as one.
pub fn send_pair(ready: bool) -> El {
    row().child(send_main(ready)).child(send_caret())
}

/// The paper clip beside Send.
pub fn attach() -> El {
    row().size(30.0, 30.0).radius(6.0).center().child(icon(&icons::ATTACH, 16.0).color(muted()))
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

/// `on` is the row the keys are on, which wears what the pointer's does (`.hm-row.on`).
fn task_row(r: &Row, first: bool, on: bool, card: Card) -> El {
    // A thread not yet read has its title a step heavier (`.hm-row.unread .hm-text b`).
    let title = text(r.title.as_str()).font(TextStyle::new(14.5, if r.unread && card == Card::Needs { 500.0 } else { 400.0 }).lh(20.0)).color(ink()).truncate();
    let under = match card {
        Card::Needs => {
            let mut tags = row().mt(3.0).h(16.0).items_center().gap(10.0);
            if let Some(mark) = priority_mark(r.priority) {
                tags = tags.child(mark);
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
    let mut line = row().key(format!("{}-{}", if card == Card::Needs { "need" } else { "progress" }, r.id)).items_center().gap(12.0).py(10.0).px(18.0).hover(white(0.045));
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

/// One of Home's two cards: a head with a link out, and what is in it.
fn home_card(title: &str, count: Option<usize>, link: &str, to: Option<View>, body: El, height: f32) -> El {
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
        .child(label(link, 13.0, 400.0, dim()).on_press(move |s: &mut State| {
            // A link to a page this window does not have yet goes nowhere.
            if let Some(to) = &to {
                s.go(to.clone());
            }
        }));
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
    // tall the window is: the column is never shorter than 660, so nothing in it has to give.
    let (page_w, _) = s.page_size();
    let column = COLUMN.min((page_w - 64.0).max(320.0));
    let above = (0.09 * s.size.1 - 24.0).clamp(0.0, 80.0);
    let cards_h = 300.0;
    // A window 980 points wide or less stacks the cards, each as wide as the column.
    let stacked = s.size.0 <= 980.0;

    // An empty card says so in one line (`HomePage.tsx`), and a head shows its count only above zero.
    let list = |rows: &[Row], empty: &str, card: Card| {
        if rows.is_empty() {
            el().py(14.0).px(18.0).child(label(empty, 13.5, 400.0, dim()))
        } else {
            // Only Needs you is walked by the keys.
            col().children(rows.iter().enumerate().map(|(i, r)| task_row(r, i == 0, card == Card::Needs && s.home_pick == Some(i), card)))
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
        .child(door(&icons::DOOR_INBOX, "Inbox", Some(needs.len()), !needs.is_empty()).on_press(|s: &mut State| s.go(View::Inbox)))
        .child(door(&icons::DOOR_PROGRESS, "In progress", Some(progress.len()), false));
    if team {
        doors = doors.child(door(&icons::DOOR_TEAM, "Team", None, false));
    }
    let second = if team {
        // The team's rows are the Team page's, which this window does not have yet: the card
        // says so when nobody else is on the team, and is bare otherwise.
        let nobody = if window::team_has_others(&s.snap) { el() } else { el().py(14.0).px(18.0).child(label("Nobody else on your team yet.", 13.5, 400.0, dim())) };
        home_card("Team this week", None, "Open Team →", None, nobody, cards_h)
    } else {
        home_card("In progress", count(progress.len()), "Open →", Some(View::Inbox), working, cards_h)
    };

    let page = col()
        .w(column)
        .pt(24.0)
        .pb(40.0)
        .child(el().h(above))
        .child(text(greeting(s.now, window::greeted(&s.snap))).font(TextStyle::new(30.0, 500.0).lh(36.0).tracking(-0.36)).color(ink()))
        .child(doors)
        .child(el().mt(22.0).child(crate::compose::inline(s).w_full()))
        .child(if stacked {
            col().mt(24.0).gap(22.0).child(home_card("Needs you", count(needs.len()), "Open inbox →", Some(View::Inbox), rows, cards_h).w(column)).child(second.w(column))
        } else {
            row()
                .mt(24.0)
                .gap(22.0)
                // `grid-template-columns: 1.35fr 1fr`: the shares are of the row less its gap.
                .child(home_card("Needs you", count(needs.len()), "Open inbox →", Some(View::Inbox), rows, cards_h).w((column - 22.0) * 1.35 / 2.35))
                .child(second.w((column - 22.0) / 2.35))
        });
    row().w_full().justify_center().child(page)
}
