//! What every screen stands on: the dotted page, the main card with the sky painted on it, the
//! folded rail, and back and forward. Sizes follow the window; the insets are the web app's.

use rune::{El, Icon, col, el, icon, row};

use crate::theme::*;
use crate::{State, View, icons};

/// The main card's insets from the window: left of it the rail, above it the title strip.
pub const CARD: [f32; 4] = [82.0, 40.0, 22.0, 22.0];
/// Where the page begins inside the window: under the card's top strip.
pub const PAGE_TOP: f32 = 104.0;

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
pub fn card(s: &State) -> El {
    let card = el()
        .pin(Some(CARD[0]), Some(CARD[1]), Some(CARD[2]), Some(CARD[3]))
        .radius(6.0)
        .fill(rgba(36, 34, 33, 0.55))
        .box_shadow(shadow(0.0, 40.0, 120.0, 0.0, rgba(0, 0, 0, 0.7), false));
    // The Inbox's sky stands behind its head alone, so under it the card shows its own glass and edge.
    if s.view == View::Inbox {
        return card.box_shadow(shadow(0.0, 0.0, 0.0, 1.0, edge(), true)).child(crate::inbox::sky(s));
    }
    // Home's sky covers the whole card; a conversation has none.
    let mut ground = el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).fill(rgb(0x1b, 0x19, 0x17)).radius(6.0).clip_rounded(6.0);
    if s.view == View::Home {
        ground = ground.child(el().absolute(0.0, 0.0).size(s.sky_size.0, s.sky_size.1).pixels(&s.sky_home));
    }
    card.child(ground).child(el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).radius(6.0).border(1.0, white(0.07)))
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

fn rail_button(glyph: &'static Icon, active: bool) -> El {
    let mut b = row().size(38.0, 36.0).radius(6.0).items_center().pl(10.0);
    if active {
        b = b.fill(rgba(40, 38, 37, 0.7)).box_shadow(shadow(0.0, 0.0, 0.0, 1.0, white(0.07), true));
    }
    b.child(icon(glyph, 18.0).color(if active { ink() } else { muted() }))
}

/// The folded rail: the mark, search, the places, and feedback and settings at the foot.
pub fn rail(s: &State) -> El {
    let on_home = s.view == View::Home;
    let on_inbox = matches!(s.view, View::Inbox | View::Thread { .. });
    let waiting = !s.lists.needs.is_empty();
    let mut inbox = el().child(rail_button(&icons::INBOX, on_inbox).on_press(|s: &mut State| s.go(View::Inbox)));
    if waiting {
        // Something in the Inbox is waiting.
        inbox = inbox.child(el().absolute(25.0, 8.0).size(6.0, 6.0).radius(3.0).fill(accent()));
    }
    el().pin(Some(22.0), Some(50.0), None, Some(22.0))
        .w(38.0)
        .child(el().absolute(13.0, 11.0).child(app_mark()))
        .child(row().absolute(0.0, 52.0).size(38.0, 32.0).items_center().pl(10.0).child(icon(&icons::SEARCH, 18.0).color(muted())))
        .child(
            col().absolute(0.0, 98.0)
                .gap(3.0)
                .child(rail_button(&icons::HOME, on_home).on_press(|s: &mut State| s.go(View::Home)))
                .child(inbox)
                .child(rail_button(&icons::DEMANDS, false))
                .child(rail_button(&icons::CONNECTIONS, false))
                .child(rail_button(&icons::FILES, false)),
        )
        .child(col().pin(Some(0.0), None, None, Some(12.0)).gap(2.0).child(rail_button(&icons::FEEDBACK, false)).child(rail_button(&icons::SETTINGS, false)))
}

/// One quiet line at the card's foot when the engine cannot be read, so an empty inbox is never
/// mistaken for a finished one.
pub fn notice(s: &State) -> Option<El> {
    let said = match (&s.source, &s.error) {
        (crate::source::Source::Nothing, _) => "Mercury is not running, so there is nothing to show yet.",
        (_, Some(_)) => "Mercury is not answering. This will catch up by itself when it does.",
        _ => return None,
    };
    Some(
        row().pin(Some(CARD[0]), None, Some(CARD[2]), Some(CARD[3] + 14.0))
            .justify_center()
            .child(row().h(28.0).px(12.0).items_center().radius(6.0).fill(rgba(17, 16, 15, 0.7)).border(1.0, edge()).child(label(said, 12.5, 400.0, dim()))),
    )
}

/// Back and forward, at the head of the card. Dim while there is nowhere to go.
pub fn back_forward(s: &State) -> El {
    let arrow = |glyph: &'static Icon, live: bool| {
        row().size(26.0, 26.0).radius(6.0).center().opacity(if live { 1.0 } else { 0.45 }).child(icon(glyph, 15.0).color(if live { ink() } else { muted() }))
    };
    let arrows = row().absolute(126.0, 58.5).gap(2.0).child(arrow(&icons::BACK, !s.history.is_empty()).on_press(|s: &mut State| s.back())).child(arrow(&icons::FORWARD, false));
    // The Inbox's title and buttons stand in the same strip as the arrows.
    if s.view == View::Inbox {
        return el().absolute(0.0, 0.0).child(crate::inbox::strip(s)).child(arrows);
    }
    arrows
}
