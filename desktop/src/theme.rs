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

// `--text`, `--text-dim`, `--text-muted`, `--border`, `--accent` and the page under everything.
pub fn ink() -> Color {
    rgb(239, 235, 231)
}
pub fn dim() -> Color {
    rgb(196, 189, 184)
}
pub fn muted() -> Color {
    rgb(160, 153, 148)
}
pub fn edge() -> Color {
    rgb(70, 68, 67)
}
/// `--line-strong`: the rule around the person's turn, and under a quiet link.
pub fn rule() -> Color {
    rgb(82, 80, 79)
}
pub fn accent() -> Color {
    rgb(238, 96, 24)
}
pub fn page() -> Color {
    rgb(26, 24, 23)
}

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

/// `.hm-glass` and the composer: the page behind, blurred, under a dark wash and a faint
/// white gradient. The caller adds its own border and shadows.
pub fn glass(e: El) -> El {
    e.backdrop_blur(20.0).fill(rgba(17, 16, 15, 0.42)).gradient(white(0.075), white(0.024))
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
