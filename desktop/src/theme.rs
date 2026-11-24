//! The dark look's colours and type, as the web app's CSS resolves them, and the few shapes
//! every screen shares. Every number here was read off the running renderer.

use rune::color::srgb_decode;
use rune::{Color, El, Shadow, TextStyle, el, row, text};

/// A colour as CSS writes it: sRGB bytes and an alpha.
pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color { r: srgb_decode(r as f32 / 255.0), g: srgb_decode(g as f32 / 255.0), b: srgb_decode(b as f32 / 255.0), a }
}
pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    rgba(r, g, b, 1.0)
}
pub fn white(a: f32) -> Color {
    rgba(255, 255, 255, a)
}

// The grounds and lines are the dark skies' deep black (`html:root[data-skin="ember-grid"]
// [data-sky-dark]` in `renderer/src/home/home.css`), which is the look this window wears: it
// draws the dark orange sky and no other. The warm Ash they replaced is still the web window's
// plain Dark, which this window does not have. Every ground and line is named here, by the
// variable the web window's sheets call it, so a look that moves again moves in this one place.

// `--text`, `--text-dim`, `--text-faint`, `--line`, `--accent` and the page under everything.
pub fn ink() -> Color {
    rgb(239, 235, 231)
}
pub fn dim() -> Color {
    rgb(196, 189, 184)
}
pub fn muted() -> Color {
    rgb(160, 153, 148)
}
/// `--line`, `--card-line`, `--skin-pane-line`: a hairline.
pub fn edge() -> Color {
    rgb(54, 54, 57)
}
/// `--line` thinned to `a`, as `color-mix(in srgb, var(--line) …, transparent)` thins it.
pub fn edge_at(a: f32) -> Color {
    rgba(54, 54, 57, a)
}
/// `--line-strong`, `--card-line-hover`, `--control-face-line`: the rule around the person's
/// turn, and under a quiet link.
pub fn rule() -> Color {
    rgb(66, 66, 69)
}
/// `--line-strong` thinned to `a`.
pub fn rule_at(a: f32) -> Color {
    rgba(66, 66, 69, a)
}
/// `--text` thinned to `a`, as `color-mix(in srgb, var(--text) …, transparent)` thins it.
pub fn ink_at(a: f32) -> Color {
    rgba(239, 235, 231, a)
}
pub fn accent() -> Color {
    rgb(238, 96, 24)
}
/// `--bg-raised`: the ground of a card that stands on the page, and of a menu.
pub fn raised() -> Color {
    rgb(26, 26, 28)
}
/// `--bg-hover`: a row or a button under the pointer, where it has a ground of its own.
pub fn hovered() -> Color {
    rgb(25, 25, 27)
}
/// `--bg-selected`: the row or the choice that is picked.
pub fn selected() -> Color {
    rgb(33, 33, 35)
}
/// `--surface`, and with it `--skin-pane`, `--skin-solid`, `--doc-page`, `--work-fill` and
/// `--control-face`: the main card's ground, a document's page, and a box to type in.
pub fn surface() -> Color {
    rgb(19, 19, 21)
}
/// `--surface` thinned to `a`.
pub fn surface_at(a: f32) -> Color {
    rgba(19, 19, 21, a)
}
/// `--pop`: the ground of what stands over the page (a menu, a toast, a card of choices).
pub fn pop() -> Color {
    rgb(31, 31, 33)
}
/// `--pop` thinned to `a`.
pub fn pop_at(a: f32) -> Color {
    rgba(31, 31, 33, a)
}
/// `--code-ground`: under a block of code.
pub fn code_ground() -> Color {
    rgb(15, 15, 17)
}
/// `--bg`, `--on-solid`: the window under the main card, and the ink on a light button.
pub fn page() -> Color {
    rgb(11, 11, 12)
}
/// `--bg` thinned to `a`.
pub fn page_at(a: f32) -> Color {
    rgba(11, 11, 12, a)
}
/// `--pane-corner`: the main card's corner, and the sky's that fills it.
pub const PANE_CORNER: f32 = 10.0;

/// `line-height: normal` as Chromium computes it for Geist: the font's ascent (1.005 em) and
/// descent (0.295 em), each landed on a device pixel of a 2x screen, then added.
pub fn sans(size: f32, weight: f32) -> TextStyle {
    let snap = |v: f32| (v * 2.0).round() / 2.0;
    TextStyle::new(size, weight).lh(snap(1.005 * size) + snap(0.295 * size))
}
/// A length as Chromium keeps it on a 2x screen: in whole 64ths of a device pixel, rounded
/// down. A line height that is not a whole number of them (25.6 points, 22.4) loses a sliver on
/// every line, and down a page of lines the slivers decide which pixel a line lands on.
pub fn kept(points: f32) -> f32 {
    (points * 128.0).floor() / 128.0
}
/// Geist Mono at an explicit line height.
pub fn mono(size: f32, line: f32) -> TextStyle {
    TextStyle::new(size, 400.0).mono().lh(line)
}
/// Tabular figures, for counts that must not jiggle.
pub fn figures(size: f32, weight: f32) -> TextStyle {
    sans(size, weight).features("'tnum' 1")
}
pub fn label(s: &str, size: f32, weight: f32, c: Color) -> El {
    text(s).font(sans(size, weight)).color(c)
}
pub fn shadow(x: f32, y: f32, blur: f32, spread: f32, color: Color, inset: bool) -> Shadow {
    Shadow { x, y, blur, spread, color, inset }
}

/// Words the person can select and copy, washed with the web app's `--select` while they are.
pub fn readable(words: El) -> El {
    words.selectable(true).selection_color(rgba(110, 165, 255, 0.3))
}

/// The strength mark beside a priority: bars three points wide, standing on one line.
pub fn bars(heights: &[f32], c: Color) -> El {
    row().items_end().gap(1.5).children(heights.iter().map(|h| el().size(3.0, *h).radius(1.0).fill(c)))
}

/// How far the glass blurs what is behind it, in points, and how much stronger it makes its
/// colours (`.hm-glass` and the composer: `backdrop-filter: blur(20px) saturate(1.5)`). The
/// sky behind it is blurred so when it is painted (`sky::soft`): nothing in this window asks
/// Rune to blur what is behind an element, which holds copies of the whole window while it does.
pub const GLASS_BLUR: f32 = 20.0;
pub const GLASS_SATURATES: f32 = 1.5;

/// The glass's own dark wash and faint white gradient, over the sky blurred behind it. The
/// caller adds its own border and shadows.
pub fn glass_wash(e: El) -> El {
    e.fill(rgba(17, 16, 15, 0.42)).gradient(white(0.075), white(0.024))
}

/// Lands a text's glyphs where Chromium lands them, to a fraction of a pixel. Rune in this font
/// mode puts each glyph's pen on CoreGraphics' step, rounded down: half a pixel for type up to
/// 16 points on a 2x screen, a whole pixel above. Chromium rounds its pens to the nearest quarter
/// pixel. Starting the text half a step later turns rounding down into rounding to the nearest,
/// which is Chromium's place to within a quarter pixel. On a 1x screen the steps are finer and
/// this is never more than half of one. The element keeps its box: what the left takes, the
/// right gives back.
pub fn pen(e: El, size: f32) -> El {
    let half_step = if size > 16.65 { 0.25 } else { 0.125 };
    e.ml(half_step).mr(-half_step)
}
