//! The Inbox: every thread waiting on the person, as a list.
//!
//! A first draft. It lists the right threads in the right order with the right words, in the
//! page's own type, but it has not been matched against the web app's Inbox pixel by pixel yet
//! (`tools/reference.mjs` with the `inbox` scene is how that is done).

use rune::{El, ScrollSpec, TextCase, TextStyle, col, row, text};

use crate::data::{Row, ago_short};
use crate::theme::*;
use crate::{State, View};

fn line(r: &Row, first: bool, now: f64) -> El {
    let (id, product) = (r.id.clone(), r.product.clone());
    let mut e = row().key(format!("inbox-{}", r.id)).items_center().gap(12.0).py(11.0).px(18.0);
    if !first {
        e = e.border_sides([1.0, 0.0, 0.0, 0.0], white(0.067)).border_layout();
    }
    e.on_press(move |s: &mut State| s.go(View::Thread { id: id.clone(), product: product.clone() }))
        .child(row().size(28.0, 28.0).radius(14.0).center().fill(white(0.12)).child(label(r.letter, 12.0, 500.0, ink())))
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .child(text(r.title.as_str()).font(TextStyle::new(14.5, 400.0).lh(20.0)).color(ink()).truncate())
                .child(text(r.line.as_str()).font(sans(13.0, 400.0)).color(dim()).truncate()),
        )
        .child(text(r.project.as_str()).font(mono(10.5, 16.0).tracking(0.525)).text_case(TextCase::Uppercase).color(muted()))
        .child(label(&r.at.map(|at| ago_short(at, now)).unwrap_or_default(), 12.5, 400.0, dim()))
}

pub fn view(s: &State) -> El {
    let needs = s.snap.needs_you();
    let room = s.size.0 - crate::chrome::CARD[0] - crate::chrome::CARD[2] - 64.0;
    let column = 760f32.min(room.max(320.0));
    let page_h = s.size.1 - crate::chrome::PAGE_TOP - crate::chrome::CARD[3];
    let head = row().items_center().gap(10.0).child(label("Inbox", 22.0, 500.0, ink())).child(text(needs.len().to_string()).font(figures(14.0, 400.0)).color(muted()));
    let list = glass(col())
        .radius(10.0)
        .clip_rounded(10.0)
        .border(1.0, white(0.13))
        .border_layout()
        .grow(1.0)
        .min_h(0.0)
        .child(col().grow(1.0).min_h(0.0).scroll(ScrollSpec { stick_bottom: false }).child(col().py(4.0).children(needs.iter().enumerate().map(|(i, r)| line(r, i == 0, s.now)))));
    row().w_full().justify_center().child(col().w(column).h(page_h).pt(24.0).pb(24.0).gap(18.0).child(head).child(list))
}
