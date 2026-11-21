//! Mercury's Home, drawn natively. One screen, built from the web app's own measurements
//! (every size, colour, border and shadow below was read off the running renderer), so the two
//! can be laid side by side and compared pixel by pixel.
//!
//!   cargo run -- --snap home.png --size 1280x760 --scale 2     # headless, no window
//!   cargo run                                                 # the window

#![forbid(unsafe_code)]

mod icons;
mod sky;

use rune::color::srgb_decode;
use rune::{App, AppSpec, Color, Cx, El, FontSmoothing, Icon, Pixels, Shadow, TextCase, TextStyle, col, el, icon, row, spacer, text};

/// A colour as CSS writes it: sRGB bytes and an alpha.
fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color { r: srgb_decode(r as f32 / 255.0), g: srgb_decode(g as f32 / 255.0), b: srgb_decode(b as f32 / 255.0), a }
}
fn rgb(r: u8, g: u8, b: u8) -> Color {
    rgba(r, g, b, 1.0)
}
fn white(a: f32) -> Color {
    rgba(255, 255, 255, a)
}

// The dark look's tokens (`--text`, `--text-dim`, `--text-muted`, `--border`, `--accent`).
fn ink() -> Color {
    rgb(239, 235, 231)
}
fn dim() -> Color {
    rgb(196, 189, 184)
}
fn muted() -> Color {
    rgb(160, 153, 148)
}
fn edge() -> Color {
    rgb(70, 68, 67)
}
fn accent() -> Color {
    rgb(238, 96, 24)
}

/// `line-height: normal` as Chromium computes it for Geist: the font's ascent (1.005 em) and
/// descent (0.295 em), each landed on a device pixel of a 2x screen, then added.
fn sans(size: f32, weight: f32) -> TextStyle {
    let snap = |v: f32| (v * 2.0).round() / 2.0;
    TextStyle::new(size, weight).lh(snap(1.005 * size) + snap(0.295 * size))
}

/// The greeting follows the clock, as `renderer/src/home/home-model.ts` has it.
fn greeting() -> &'static str {
    let ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64);
    match rune::time::LocalTime::at(ms).hour {
        4..=11 => "Good morning",
        12..=17 => "Good afternoon",
        _ => "Good evening",
    }
}
fn label(s: &str, size: f32, weight: f32, c: Color) -> El {
    text(s).font(sans(size, weight)).color(c)
}
fn shadow(x: f32, y: f32, blur: f32, spread: f32, color: Color, inset: bool) -> Shadow {
    Shadow { x, y, blur, spread, color, inset }
}

struct State {
    sky: Pixels,
}

const W: f32 = 1280.0;
const H: f32 = 760.0;
// The main card: the window less the rail and its margins.
const CARD: [f32; 4] = [82.0, 40.0, 1176.0, 698.0];

fn init() -> State {
    let sky = Pixels::new();
    let (w, h, bytes) = sky::draw(&sky::ember(), CARD[2] as f64, CARD[3] as f64, 40.0, 300.0, 2);
    sky.set(w, h, bytes);
    State { sky }
}

/// The page under everything: a warm glow from the upper right and a dot every 32 points.
fn backdrop() -> El {
    let mut dots = Vec::new();
    let mut y = 16.0;
    while y < H {
        let mut x = 16.0;
        while x < W {
            dots.push(el().absolute(x - 1.15, y - 1.15).size(2.3, 2.3).radius(1.15).fill(white(0.055)));
            x += 32.0;
        }
        y += 32.0;
    }
    el().absolute(0.0, 0.0)
        .size(W, H)
        .radial_gradient([0.72, 0.28], [0.7 * W, 0.7 * H], [(0.0, rgba(238, 96, 24, 0.075)), (1.0, rgba(238, 96, 24, 0.0))])
        .children(dots)
}

/// The main card with the sky painted on it.
fn card(s: &State) -> El {
    el().absolute(CARD[0], CARD[1])
        .size(CARD[2], CARD[3])
        .radius(6.0)
        .fill(rgba(36, 34, 33, 0.55))
        .box_shadow(shadow(0.0, 40.0, 120.0, 0.0, rgba(0, 0, 0, 0.7), false))
        .child(el().absolute(0.0, 0.0).size(CARD[2], CARD[3]).radius(6.0).pixels(&s.sky))
        .child(el().absolute(0.0, 0.0).size(CARD[2], CARD[3]).radius(6.0).border(1.0, white(0.07)))
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

fn rail_button(glyph: &'static Icon, y: f32, active: bool) -> El {
    let mut b = row().absolute(0.0, y).size(38.0, 36.0).radius(6.0).items_center().pl(10.0);
    if active {
        b = b.fill(rgba(40, 38, 37, 0.7)).box_shadow(shadow(0.0, 0.0, 0.0, 1.0, white(0.07), true));
    }
    b.child(icon(glyph, 18.0).color(if active { ink() } else { muted() }))
}

/// The folded rail: the mark, search, the places, and feedback and settings at the foot.
fn rail() -> El {
    el().absolute(22.0, 50.0)
        .size(38.0, 688.0)
        .child(el().absolute(13.0, 11.0).child(app_mark()))
        .child(row().absolute(0.0, 52.0).size(38.0, 32.0).items_center().pl(10.0).child(icon(&icons::SEARCH, 18.0).color(muted())))
        .child(rail_button(&icons::HOME, 98.0, true))
        .child(rail_button(&icons::INBOX, 137.0, false))
        // Something in the Inbox is waiting.
        .child(el().absolute(25.0, 145.0).size(6.0, 6.0).radius(3.0).fill(accent()))
        .child(rail_button(&icons::DEMANDS, 176.0, false))
        .child(rail_button(&icons::CONNECTIONS, 215.0, false))
        .child(rail_button(&icons::FILES, 254.0, false))
        .child(rail_button(&icons::FEEDBACK, 602.0, false))
        .child(rail_button(&icons::SETTINGS, 640.0, false))
}

fn back_forward() -> El {
    let arrow = |glyph: &'static Icon| row().size(26.0, 26.0).center().opacity(0.45).child(icon(glyph, 15.0).color(muted()));
    row().absolute(126.0, 58.5).gap(2.0).child(arrow(&icons::BACK)).child(arrow(&icons::FORWARD))
}

/// `.hm-glass`: a faint white wash over the page. The web app also blurs the page behind it;
/// Rune blurs four surfaces a frame (`MAX_BACKDROP_BLURS`) and Home has five, so the three
/// cards take the blur, where the sky's streak runs, and the two small doors go without.
fn door(glyph: &'static Icon, name: &str, count: &str, count_color: Color) -> El {
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
        .child(text(count).font(sans(12.0, 400.0).features("'tnum' 1")).color(count_color))
}

/// The three-bar strength mark beside a priority.
fn bars(heights: &[f32], c: Color) -> El {
    row().items_end().gap(1.5).children(heights.iter().map(|h| el().size(3.0, *h).radius(1.0).fill(c)))
}

fn chip(children: Vec<El>) -> El {
    row().h(28.0)
        .items_center()
        .gap(8.0)
        .px(10.0)
        .radius(6.0)
        .fill(white(0.03))
        .border(1.0, edge())
        .border_layout()
        .children(children)
}

/// The composer: what do you need done, who does it, and Send.
fn composer() -> El {
    let send_fill = rgba(239, 235, 231, 0.15);
    let bar = row()
        .pt(14.0)
        .items_center()
        .gap(8.0)
        .child(chip(vec![label("Opus 5.5", 13.0, 400.0, dim())]))
        .child(chip(vec![
            row().size(17.0, 17.0).center().child(el().size(9.0, 9.0).radius(4.5).fill(accent())),
            label("Storefront", 13.0, 400.0, dim()),
        ]))
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
                        .child(text("⌘↵").font(TextStyle::new(11.0, 400.0).mono().lh(14.0)).color(ink()).opacity(0.75)),
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
    col().h(152.0)
        .pt(16.0)
        .px(18.0)
        .pb(14.0)
        .radius(10.0)
        .backdrop_blur(20.0)
        .fill(rgba(17, 16, 15, 0.42))
        .gradient(white(0.075), white(0.024))
        .border(1.0, white(0.2))
        .border_layout()
        .box_shadow(shadow(0.0, 0.0, 0.0, 3.0, white(0.03), false))
        .box_shadow(shadow(0.0, 12.0, 40.0, 0.0, rgba(0, 0, 0, 0.3), false))
        .child(el().h(74.0).child(text("What do you need done?").font(TextStyle::new(22.0, 400.0).lh(30.0)).color(muted())))
        .child(bar)
}

struct Task {
    title: &'static str,
    project: &'static str,
    age: &'static str,
    urgent: bool,
}

const NEEDS_YOU: [Task; 5] = [
    Task { title: "Checkout fails for some customers since this morning", project: "Storefront", age: "1 hr", urgent: true },
    Task { title: "Snooze is lost on restart", project: "Storefront", age: "1 hr", urgent: false },
    Task { title: "Redesign the empty inbox", project: "Storefront", age: "1 hr", urgent: false },
    Task { title: "The weekly usage total is off by one", project: "Storefront", age: "2 hr", urgent: false },
    Task { title: "Command K should search closed tasks", project: "Storefront", age: "2 hr", urgent: false },
];

fn task_row(t: &Task, first: bool) -> El {
    let mut tags = row().mt(3.0).h(16.0).items_center().gap(10.0);
    if t.urgent {
        tags = tags.child(bars(&[4.0, 7.0, 10.0, 13.0], ink()));
    }
    tags = tags.child(
        text(t.project).font(TextStyle::new(10.5, 400.0).mono().lh(16.0).tracking(0.525)).text_case(TextCase::Uppercase).color(muted()),
    );
    let mut r = row().items_center().gap(12.0).py(10.0).px(18.0);
    if !first {
        r = r.border_sides([1.0, 0.0, 0.0, 0.0], white(0.067)).border_layout();
    }
    r.child(row().size(28.0, 28.0).radius(14.0).center().fill(white(0.12)).child(label("C", 12.0, 500.0, ink())))
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .child(text(t.title).font(TextStyle::new(14.5, 400.0).lh(20.0)).color(ink()).truncate())
                .child(tags),
        )
        .child(label(t.age, 12.5, 400.0, dim()))
}

/// One of Home's two cards: a head with a link out, and what is in it.
fn home_card(title: &str, count: Option<&str>, link: &str, body: El) -> El {
    let mut name = row().items_center().child(label(title, 14.0, 500.0, ink()));
    if let Some(n) = count {
        name = name.child(text(n).font(sans(14.0, 400.0).features("'tnum' 1")).color(muted()).ml(8.0));
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
        .child(label(link, 13.0, 400.0, dim()));
    col().h(300.0)
        .radius(10.0)
        .clip_rounded(10.0)
        .backdrop_blur(20.0)
        .fill(rgba(17, 16, 15, 0.42))
        .gradient(white(0.075), white(0.024))
        .border(1.0, white(0.13))
        .border_layout()
        .box_shadow(shadow(0.0, 1.0, 0.0, 0.0, white(0.16), true))
        .box_shadow(shadow(0.0, 12.0, 40.0, 0.0, rgba(0, 0, 0, 0.3), false))
        .child(head)
        .child(col().pt(4.0).pb(28.0).grow(1.0).min_h(0.0).child(body))
        // The rows fade out into the card's foot.
        .child(el().pin(Some(1.0), None, Some(1.0), Some(1.0)).h(56.0).gradient(rgba(24, 21, 26, 0.0), rgba(24, 21, 26, 0.9)))
}

fn home() -> El {
    let rows = col().children(NEEDS_YOU.iter().enumerate().map(|(i, t)| task_row(t, i == 0)));
    let idle = el().py(14.0).px(18.0).child(label("No agent is working right now.", 13.5, 400.0, dim()));
    col().absolute(208.0, 24.0)
        .w(760.0)
        .child(el().h(44.3984))
        .child(text(greeting()).font(TextStyle::new(30.0, 500.0).lh(36.0).tracking(-0.36)).color(ink()))
        .child(
            row()
                .mt(16.0)
                .gap(8.0)
                .child(door(&icons::DOOR_INBOX, "Inbox", "9", ink()))
                .child(door(&icons::DOOR_PROGRESS, "In progress", "0", muted())),
        )
        .child(el().mt(22.0).child(composer().w_full()))
        .child(
            row()
                .mt(24.0)
                .gap(22.0)
                // `grid-template-columns: 1.35fr 1fr`: the shares are of the row less its gap.
                .child(home_card("Needs you", Some("9"), "Open inbox →", rows).w((760.0 - 22.0) * 1.35 / 2.35))
                .child(home_card("In progress", None, "Open →", idle).w((760.0 - 22.0) / 2.35)),
        )
}

fn view(s: &State, _: &mut Cx<State>) -> El {
    el().w_full()
        .h_full()
        .fill(rgb(26, 24, 23))
        .child(backdrop())
        .child(card(s))
        .child(rail())
        .child(back_forward())
        // The page scrolls inside the card, under its top strip.
        .child(el().absolute(82.0, 104.0).size(1176.0, 634.0).clip_corners([0.0, 0.0, 6.0, 6.0]).child(home()))
}

static FONTS: &[&[u8]] = &[include_bytes!("../assets/fonts/Geist-Variable.ttf"), include_bytes!("../assets/fonts/GeistMono-Variable.ttf")];

fn main() {
    // The web app's own lists: Geist Mono has no ⌘, which the system's monospace then draws.
    let spec = AppSpec::<State> { fonts: FONTS, families: ("Geist", "Geist Mono, ui-monospace"), ..AppSpec::new("Mercury", (W, H), init, view) };
    // The web app sets `-webkit-font-smoothing: antialiased`; `Off` is that rendering.
    App::new(spec).font_smoothing(FontSmoothing::Off).run();
}
