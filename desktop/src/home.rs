//! Home: the greeting, the two doors, the box to start a thread, and what needs you beside
//! what is in progress. Built from the web app's own measurements (`renderer/src/home`).

use rune::{El, Icon, ScrollSpec, TextCase, TextStyle, col, el, icon, row, spacer, text};

use crate::data::{Priority, Row, ago_short};
use crate::theme::*;
use crate::{State, View, icons};

/// The page's column: 760 points, or what the card leaves it.
const COLUMN: f32 = 760.0;

/// The greeting follows the clock, as `renderer/src/home/home-model.ts` has it.
fn greeting(now_ms: f64) -> &'static str {
    match rune::time::LocalTime::at(now_ms as i64).hour {
        4..=11 => "Good morning",
        12..=17 => "Good afternoon",
        _ => "Good evening",
    }
}

/// `.hm-glass`: a faint white wash over the page. The web app also blurs the page behind it;
/// Rune blurs four surfaces a frame (`MAX_BACKDROP_BLURS`) and Home has five, so the three
/// cards take the blur, where the sky's streak runs, and the two small doors go without.
fn door(glyph: &'static Icon, name: &str, count: usize, hot: bool) -> El {
    // A Rune that blurs five or more gives the doors theirs back with no change here.
    let glass = if rune::MAX_BACKDROP_BLURS >= 5 { row().backdrop_blur(20.0) } else { row() };
    glass
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
        .child(label(name, 13.5, 400.0, ink()))
        .child(text(count.to_string()).font(figures(12.0, 400.0)).color(if hot { ink() } else { muted() }))
}

fn chip(children: Vec<El>) -> El {
    row().h(28.0).items_center().gap(8.0).px(10.0).radius(6.0).fill(white(0.03)).border(1.0, edge()).border_layout().children(children)
}

/// The composer: what do you need done, who does it, and Send. Phase 1 only draws it.
fn composer(project: &str) -> El {
    let send_fill = rgba(239, 235, 231, 0.15);
    let bar = row()
        .pt(14.0)
        .items_center()
        .gap(8.0)
        .child(chip(vec![label("Opus 5.5", 13.0, 400.0, dim())]))
        .child(chip(vec![row().size(17.0, 17.0).center().child(el().size(9.0, 9.0).radius(4.5).fill(accent())), label(project, 13.0, 400.0, dim())]))
        .child(chip(vec![bars(&[4.0, 7.5, 11.0], rgb(82, 80, 79)), label("Agent Decides", 13.0, 400.0, dim())]))
        .child(spacer())
        .child(row().size(30.0, 30.0).radius(6.0).center().child(icon(&icons::ATTACH, 16.0).color(muted())))
        .child(
            row()
                .child(
                    row()
                        .h(32.0)
                        .items_center()
                        .gap(9.0)
                        .px(14.0)
                        .corners([6.0, 0.0, 0.0, 6.0])
                        .fill(send_fill)
                        .opacity(0.45)
                        .child(label("Send", 13.5, 500.0, ink()))
                        .child(text("⌘↵").font(mono(11.0, 14.0)).color(ink()).opacity(0.75)),
                )
                .child(
                    row()
                        .size(32.0, 32.0)
                        .items_center()
                        .pl(9.0)
                        .corners([0.0, 6.0, 6.0, 0.0])
                        .fill(send_fill)
                        .border_sides([0.0, 0.0, 0.0, 1.0], rgba(26, 24, 23, 0.55))
                        .border_layout()
                        .child(icon(&icons::CARET, 13.0).color(ink())),
                ),
        );
    glass(col())
        .h(152.0)
        .pt(16.0)
        .px(18.0)
        .pb(14.0)
        .radius(10.0)
        .border(1.0, white(0.2))
        .border_layout()
        .box_shadow(shadow(0.0, 0.0, 0.0, 3.0, white(0.03), false))
        .box_shadow(shadow(0.0, 12.0, 40.0, 0.0, rgba(0, 0, 0, 0.3), false))
        .child(el().h(74.0).child(text("What do you need done?").font(TextStyle::new(22.0, 400.0).lh(30.0)).color(muted())))
        .child(bar)
}

fn priority_mark(p: Priority) -> Option<El> {
    match p {
        Priority::Urgent => Some(bars(&[4.0, 7.0, 10.0, 13.0], ink())),
        Priority::High => Some(bars(&[4.0, 7.5, 11.0], ink())),
        Priority::Low => Some(row().items_end().gap(1.5).child(el().size(3.0, 4.0).radius(1.0).fill(dim())).child(el().size(3.0, 7.5).radius(1.0).fill(rgb(82, 80, 79))).child(el().size(3.0, 11.0).radius(1.0).fill(rgb(82, 80, 79)))),
        Priority::Medium => None,
    }
}

fn task_row(r: &Row, first: bool, now: f64) -> El {
    let mut tags = row().mt(3.0).h(16.0).items_center().gap(10.0);
    if let Some(mark) = priority_mark(r.priority) {
        tags = tags.child(mark);
    }
    tags = tags.child(text(r.project.as_str()).font(mono(10.5, 16.0).tracking(0.525)).text_case(TextCase::Uppercase).color(muted()));
    let mut line = row().key(format!("need-{}", r.id)).items_center().gap(12.0).py(10.0).px(18.0);
    if !first {
        line = line.border_sides([1.0, 0.0, 0.0, 0.0], white(0.067)).border_layout();
    }
    let (id, product) = (r.id.clone(), r.product.clone());
    line.on_press(move |s: &mut State| s.go(View::Thread { id: id.clone(), product: product.clone() }))
        .child(row().size(28.0, 28.0).radius(14.0).center().fill(white(0.12)).child(label(r.letter, 12.0, 500.0, ink())))
        .child(col().grow(1.0).min_w(0.0).child(text(r.title.as_str()).font(TextStyle::new(14.5, 400.0).lh(20.0)).color(ink()).truncate()).child(tags))
        .child(label(&r.at.map(|at| ago_short(at, now)).unwrap_or_default(), 12.5, 400.0, dim()))
}

/// One of Home's two cards: a head with a link out, and what is in it.
fn home_card(title: &str, count: Option<usize>, link: &str, to: View, body: El, height: f32) -> El {
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
        .child(label(link, 13.0, 400.0, dim()).on_press(move |s: &mut State| s.go(to.clone())));
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
    let needs = s.snap.needs_you();
    let progress = s.snap.in_progress();
    // What the card leaves the column, and what the column leaves the two cards.
    let room = s.size.0 - crate::chrome::CARD[0] - crate::chrome::CARD[2] - 64.0;
    let column = COLUMN.min(room.max(320.0));
    let page_h = s.size.1 - crate::chrome::PAGE_TOP - crate::chrome::CARD[3];
    let cards_h = (page_h - 24.0 - 40.0 - 326.3984).max(300.0);

    let rows = col().children(needs.iter().enumerate().map(|(i, r)| task_row(r, i == 0, s.now)));
    let working = if progress.is_empty() {
        el().py(14.0).px(18.0).child(label("No agent is working right now.", 13.5, 400.0, dim()))
    } else {
        col().children(progress.iter().enumerate().map(|(i, r)| task_row(r, i == 0, s.now)))
    };
    let project = s.snap.products.first().map(|p| p.name.clone()).unwrap_or_default();

    let page = col()
        .w(column)
        .pt(24.0)
        .pb(40.0)
        .child(el().h(44.3984))
        .child(text(greeting(s.now)).font(TextStyle::new(30.0, 500.0).lh(36.0).tracking(-0.36)).color(ink()))
        .child(
            row()
                .mt(16.0)
                .gap(8.0)
                .child(door(&icons::DOOR_INBOX, "Inbox", needs.len(), true).on_press(|s: &mut State| s.go(View::Inbox)))
                .child(door(&icons::DOOR_PROGRESS, "In progress", progress.len(), !progress.is_empty())),
        )
        .child(el().mt(22.0).child(composer(&project).w_full()))
        .child(
            row()
                .mt(24.0)
                .gap(22.0)
                // `grid-template-columns: 1.35fr 1fr`: the shares are of the row less its gap.
                .child(home_card("Needs you", Some(needs.len()), "Open inbox →", View::Inbox, rows, cards_h).w((column - 22.0) * 1.35 / 2.35))
                .child(home_card("In progress", None, "Open →", View::Inbox, working, cards_h).w((column - 22.0) / 2.35)),
        );
    row().w_full().justify_center().child(page)
}
