//! What every screen stands on: the dotted page, the main card with the sky painted on it, the
//! folded rail, and back and forward. Sizes follow the window; the insets are the web app's.

use rune::{El, Icon, TextCase, TextStyle, col, el, icon, row, text};

use crate::rules::window;
use crate::theme::*;
use crate::{State, View, icons};

/// The main card's insets from the window: left of it the rail, above it the title strip.
pub const CARD: [f32; 4] = [82.0, 40.0, 22.0, 22.0];
/// Where the page begins inside the window: under the card's top strip.
pub const PAGE_TOP: f32 = 104.0;

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
pub fn card(s: &State) -> El {
    let card = el()
        .pin(Some(CARD[0]), Some(CARD[1]), Some(CARD[2]), Some(CARD[3]))
        .radius(6.0)
        .fill(rgba(36, 34, 33, 0.55))
        .box_shadow(shadow(0.0, 40.0, 120.0, 0.0, rgba(0, 0, 0, 0.7), false));
    let whole = || el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).radius(6.0);
    match s.view {
        // Home's sky covers the whole card, on a ground of its own, under a faint edge.
        View::Home => card
            .child(whole().fill(rgb(0x1b, 0x19, 0x17)).clip_rounded(6.0).child(el().absolute(0.0, 0.0).size(s.sky_size.0, s.sky_size.1).pixels(&s.sky_home)))
            .child(whole().border(1.0, white(0.07))),
        // The Inbox's sky stands behind its head alone, so under it the card shows its own glass and edge.
        View::Inbox => card.box_shadow(shadow(0.0, 0.0, 0.0, 1.0, edge(), true)).child(crate::inbox::sky(s)),
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

fn rail_button(glyph: &'static Icon, active: bool) -> El {
    let mut b = row().size(38.0, 36.0).radius(6.0).items_center().pl(10.0);
    if active {
        b = b.fill(rgba(40, 38, 37, 0.7)).box_shadow(shadow(0.0, 0.0, 0.0, 1.0, white(0.07), true));
    }
    b.child(icon(glyph, 18.0).color(if active { ink() } else { muted() }))
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

/// The folded rail: the mark, search, the places, and a foot that holds what this person's
/// Mercury has: Record meeting, feedback, settings, what is left of the limit, their own face.
pub fn rail(s: &State) -> El {
    let on_home = s.view == View::Home;
    let on_inbox = matches!(s.view, View::Inbox | View::Thread { .. });
    let waiting = !s.lists.needs.is_empty();
    let mut inbox = el().child(rail_button(&icons::INBOX, on_inbox).on_press(|s: &mut State| s.go(View::Inbox)));
    if waiting {
        // Something in the Inbox is waiting.
        inbox = inbox.child(el().absolute(25.0, 8.0).size(6.0, 6.0).radius(3.0).fill(accent()));
    }
    let mut places = col()
        .absolute(0.0, 98.0)
        .gap(3.0)
        .child(rail_button(&icons::HOME, on_home).on_press(|s: &mut State| s.go(View::Home)))
        .child(inbox)
        .child(rail_button(&icons::DEMANDS, false));
    // With teammates the one row under the threads is Chats (`teammatesUi.rail`).
    if window::teammates_live(&s.snap) {
        places = places.child(rail_button(&icons::CHATS, false));
    }
    places = places.child(rail_button(&icons::CONNECTIONS, false)).child(rail_button(&icons::FILES, false));

    // The foot stands on the rail's end and grows upward: the person's face under a hairline,
    // over it the usage button, over that the buttons everyone has (`.workspace-bottom`).
    let me = window::my_letters(&s.snap);
    let usage = window::usage_shown(&s.snap, s.now);
    let face_room = if me.is_some() { 54.0 } else { 0.0 };
    let mut buttons = col().pin(Some(0.0), None, None, Some(face_room + if usage { 50.0 } else { 12.0 })).gap(2.0);
    if window::update_waits(&s.snap) {
        // Restart to update stands first, with the room its card rises from (`.sb-update-slot`).
        buttons = buttons.child(el().pb(9.0).child(rail_button(&icons::RESTART, false)));
    }
    if window::can_record(&s.snap) {
        buttons = buttons.child(rail_button(&icons::RECORD, false));
    }
    if window::takes_feedback(&s.snap) {
        buttons = buttons.child(rail_button(&icons::FEEDBACK, false));
    }
    buttons = buttons.child(rail_button(&icons::SETTINGS, false));

    let mut rail = el()
        .pin(Some(22.0), Some(50.0), None, Some(22.0))
        .w(38.0)
        .child(el().absolute(13.0, 11.0).child(app_mark()))
        .child(row().absolute(0.0, 52.0).size(38.0, 32.0).items_center().pl(10.0).child(icon(&icons::SEARCH, 18.0).color(muted())))
        .child(places)
        .child(buttons);
    if usage {
        rail = rail.child(el().pin(Some(0.0), None, None, Some(face_room + 12.0)).child(rail_button(&icons::USAGE, false)));
    }
    if let Some(letters) = me {
        rail = rail
            .child(el().pin(Some(0.0), None, Some(0.0), Some(45.0)).h(1.0).fill(edge()))
            .child(el().pin(Some(8.0), None, None, Some(7.5)).child(my_face(&letters)));
    }
    rail
}

/// One quiet line at the card's foot when the engine cannot be read, so an empty inbox is never
/// mistaken for a finished one; and when an agent asks for a yes this window cannot yet give.
pub fn notice(s: &State) -> Option<El> {
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
        row().pin(Some(CARD[0]), None, Some(CARD[2]), Some(CARD[3] + 14.0))
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
/// open it, running out to the left.
fn toast_clock(toast: &crate::write::Toast, at: f64, width: f32) -> El {
    let left = ((toast.until - at) / (toast.until - toast.since).max(0.001)).clamp(0.0, 1.0) as f32;
    el().pin(Some(0.0), None, None, Some(0.0)).size(width * left, 2.0).fill(accent()).opacity(0.7)
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
            .clip_rounded(10.0)
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

/// Back and forward, at the head of the card. Dim while there is nowhere to go. A conversation
/// has a strip of its own there, with these two in it.
pub fn back_forward(s: &State) -> El {
    if let View::Thread { id, product } = &s.view {
        return crate::thread::head(s, id, product);
    }
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
