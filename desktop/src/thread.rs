//! One conversation, to read: what was asked and what the agent answered.
//!
//! A first draft. It shows the thread's own words in the page's type, with paragraphs kept, but
//! it does not draw markdown (tables, code, lists) yet and has not been matched against the web
//! app's thread view pixel by pixel (`tools/reference.mjs` with the `thread` scene is how).

use rune::{El, ScrollSpec, TextCase, TextStyle, col, el, row, text};

use crate::theme::*;
use crate::State;

fn said(who: &str, words: &str, mine: bool) -> El {
    let mut bubble = col().gap(8.0).p_all(16.0, 14.0).radius(10.0).border(1.0, white(if mine { 0.2 } else { 0.13 })).border_layout();
    bubble = if mine { bubble.fill(white(0.05)) } else { glass(bubble) };
    bubble.child(text(who).font(mono(10.5, 16.0).tracking(0.525)).text_case(TextCase::Uppercase).color(muted())).children(
        words.split("\n\n").filter(|p| !p.trim().is_empty()).map(|p| text(p.trim()).font(TextStyle::new(14.5, 400.0).lh(22.0)).color(ink()).wrap()),
    )
}

trait Pad {
    fn p_all(self, x: f32, y: f32) -> Self;
}
impl Pad for El {
    fn p_all(self, x: f32, y: f32) -> Self {
        self.px(x).py(y)
    }
}

pub fn view(s: &State, id: &str, product: &str) -> El {
    let (page_w, page_h) = s.page_size();
    let column = 760f32.min((page_w - 64.0).max(320.0));
    let Some(item) = s.snap.items.iter().find(|i| i.id == id && i.product == product) else {
        return row().w_full().justify_center().child(el().pt(60.0).child(label("This conversation is not here any more.", 14.0, 400.0, dim())));
    };
    let title = crate::data::first_real_line(item.label.as_deref().or(item.title.as_deref()).unwrap_or(""));
    let mut talk = col().gap(14.0).pb(40.0);
    if let Some(body) = item.body.as_deref().filter(|b| !b.trim().is_empty()) {
        talk = talk.child(said("You", body, true));
    }
    if let Some(result) = item.result.as_deref().filter(|r| !r.trim().is_empty()) {
        talk = talk.child(said("Agent", result, false));
    }
    let head = col().gap(4.0).child(text(title).font(TextStyle::new(22.0, 500.0).lh(28.0)).color(ink()).wrap()).child(
        text(item.product_name.clone().unwrap_or_else(|| item.product.clone())).font(mono(10.5, 16.0).tracking(0.525)).text_case(TextCase::Uppercase).color(muted()),
    );
    row().w_full().justify_center().child(
        col().w(column).h(page_h).pt(24.0).gap(18.0).child(head).child(col().grow(1.0).min_h(0.0).scroll(ScrollSpec { stick_bottom: false }).child(talk)),
    )
}
