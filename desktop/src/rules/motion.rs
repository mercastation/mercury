//! The web window's looping motion, as Rune's loops draw it: each CSS animation a conversation
//! and its marks run, written as the stops of a `rune::Cycle`, the opacity curve Rune samples off
//! its clock (`live-lattice-orbit` and `live-shimmer` in `renderer/src/styles.css`, `th-breathe`
//! in `renderer/src/threads/pages.css`).
//!
//! A `Cycle` goes from one stop to the next in a straight line, where a CSS animation eases each
//! step between two keyframes on its timing function, so an eased step is laid down here as many
//! short straight ones, nearer together than one frame of a display.
//!
//! Every cycle is an opacity, and the window draws a frame of nothing but loops without building
//! its page again only for a cycle whose opacity never reaches nought. So where the web window's
//! value is nought, a cycle here holds `LEAST`, a five-hundredth, which no screen shows.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use rune::Easing;

/// The least opacity a cycle holds (see the module's note).
pub const LEAST: f32 = 0.002;

/// One CSS keyframe: where in the cycle, from 0 to 1, and the opacity there.
pub type Frame = (f32, f32);

/// The stops of a cycle whose keyframes are `frames` (ascending, from 0 to 1) with `easing`
/// between each two, as CSS's `animation-timing-function` eases each step: a step between two
/// different values laid down as `pieces` straight ones, a step that holds a value as one.
pub fn eased_stops(frames: &[Frame], easing: Easing, pieces: usize) -> Vec<(f32, f32)> {
    let mut out = vec![frames[0]];
    for pair in frames.windows(2) {
        let ((a, from), (b, to)) = (pair[0], pair[1]);
        if from == to {
            out.push((b, to));
            continue;
        }
        for k in 1..=pieces {
            let x = k as f32 / pieces as f32;
            out.push((a + (b - a) * x, from + (to - from) * easing.eval(x)));
        }
    }
    out
}

/// The value of `stops` at `x` (0 to 1), as `rune::Cycle::sample` reads them: in a straight line
/// between the stops either side.
#[cfg(test)]
pub fn sample(stops: &[(f32, f32)], x: f32) -> f32 {
    let Some(k) = stops.iter().position(|&(at, _)| at >= x) else { return stops.last().map_or(1.0, |s| s.1) };
    if k == 0 {
        return stops[0].1;
    }
    let ((x0, v0), (x1, v1)) = (stops[k - 1], stops[k]);
    v0 + (v1 - v0) * (x - x0) / (x1 - x0).max(1e-6)
}

/* --------------------------------- the lattice --------------------------------- */

/// How long one turn of the working mark's orbit takes (`live-lattice-orbit`, 1488 ms).
pub const LATTICE_PERIOD: f32 = 1.488;

/// Each of the mark's nine cells, row by row, by its place in the orbit: eight round the edge, a
/// step apart, and the middle a hole that never lights (`ORBIT` in `LiveMark.tsx`).
pub const LATTICE_ORBIT: [Option<u8>; 9] = [Some(0), Some(1), Some(2), Some(7), None, Some(3), Some(6), Some(5), Some(4)];

/// A cell's opacity under the system's Reduce motion, the hole's at all times, and a cell's in a
/// mark that is not working (`.live-mark.is-still`).
pub const LATTICE_STILL: f32 = 0.65;
pub const LATTICE_HOLE: f32 = 0.1;

/// Where in the orbit a cell is: its `animation-delay` of `place × 186 − 1488` ms puts it
/// `place × 186` ms behind the first, an eighth of the turn, and a `Cycle` stands at
/// `t / period − phase`.
pub fn lattice_phase(place: u8) -> f32 {
    f32::from(place) * 0.186 / LATTICE_PERIOD
}

/// A cell's opacity through one turn: faint, lit by 18%, held lit to 42%, faint again by 62%
/// (`@keyframes live-lattice-orbit`), each change on `cubic-bezier(0.77, 0, 0.175, 1)`.
pub fn lattice_stops() -> &'static [(f32, f32)] {
    static STOPS: OnceLock<Vec<(f32, f32)>> = OnceLock::new();
    STOPS.get_or_init(|| eased_stops(&[(0.0, 0.22), (0.18, 1.0), (0.42, 1.0), (0.62, 0.22), (1.0, 0.22)], Easing::cubic(0.77, 0.0, 0.175, 1.0), 32))
}

/* --------------------------------- the breath --------------------------------- */

/// How long one breath of a running mark takes (`th-breathe`, 1.6 s).
pub const BREATHE_PERIOD: f32 = 1.6;

/// Its opacity through one breath: whole, a little under half at the middle, whole again
/// (`@keyframes th-breathe { 50% { opacity: .45 } }`), each half `ease-in-out`.
pub fn breathe_stops() -> &'static [(f32, f32)] {
    static STOPS: OnceLock<Vec<(f32, f32)>> = OnceLock::new();
    STOPS.get_or_init(|| eased_stops(&[(0.0, 1.0), (0.5, 0.45), (1.0, 1.0)], Easing::EASE_IN_OUT, 24))
}

/// The same breath as a veil of the ground over a mark that stands still: as much of the ground
/// as the mark gives up, so the mark under it shows at the breath's opacity (a veil of the
/// ground at `1 − o` over a mark is the mark at `o`, where the ground is one colour).
pub fn breathe_veil_stops() -> &'static [(f32, f32)] {
    static STOPS: OnceLock<Vec<(f32, f32)>> = OnceLock::new();
    STOPS.get_or_init(|| breathe_stops().iter().map(|&(at, o)| (at, (1.0 - o).max(LEAST))).collect())
}

/* --------------------------------- the shimmer -------------------------------- */

/// How long the band takes to cross a word (`live-shimmer`, 2.6 s, linear).
pub const SHIMMER_PERIOD: f32 = 2.6;

/// How faint the word is at the middle of the band: its mask's 45% black (`color-mix(in srgb,
/// #000 45%, transparent)`), so a veil of the ground over it at 55%.
const DEEPEST: f32 = 0.55;

/// The web window draws a working step's word in full ink under a mask that a soft band of 45%
/// travels through from its left to its right (`.live-word.is-shimmering`): a gradient at 100
/// degrees, whole to 32%, 45% at the middle and whole again from 68%, 260% of the word's width,
/// moved from 160% to −60% of the room left over. Here the word is drawn whole, and over it a veil
/// in the colour of the ground under it, in strips side by side, each a cycle that comes up as the
/// band's middle reaches it: a veil of the ground at `1 − m` over ink is the ink at `m`, exactly,
/// where the ground is one colour. One strip is where it starts and how wide it is, in points from
/// the word's left edge, and where in the cycle the band's middle is on it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strip {
    pub x: f32,
    pub w: f32,
    pub phase: f32,
}

/// The veil over a word `wide` by `tall` points: its strips, and the stops each of them cycles
/// through.
pub struct Veil {
    pub strips: Vec<Strip>,
    pub stops: &'static [(f32, f32)],
}

/// How far the band's middle is from where its veil fades to nothing, across the word at its
/// middle line: 18% of the gradient's length, which a 100 degree line through a box 2.6 words
/// wide and `tall` high measures `2.6 W sin 100° + H |cos 100°|`, measured across.
fn half_band(wide: f32, tall: f32) -> f32 {
    let (sin, cos) = (100f32.to_radians().sin(), 100f32.to_radians().cos().abs());
    0.18 * (2.6 * wide * sin + tall * cos) / sin
}

/// The band's middle travels from 1.26 words left of the word's start to 2.26 words right of it,
/// 3.52 words in one period: the mask's left edge goes from −2.56 W to 0.96 W (`mask-position`
/// of 160% and −60% of `W − 2.6 W`), and the band's middle is 1.3 W into it.
const START: f32 = -1.26;
const TRAVEL: f32 = 3.52;

/// The stops of a strip's cycle when the band is `half` of a period wide either side of its
/// middle: nothing, rising in a straight line to the deepest at the middle of the cycle, and
/// nothing again. Kept by that width to a 2048th, so a window that shows a thousand different
/// steps keeps a few dozen tables.
fn band_stops(half: f32) -> &'static [(f32, f32)] {
    static TABLES: OnceLock<Mutex<HashMap<u32, &'static [(f32, f32)]>>> = OnceLock::new();
    let key = (half * 2048.0).round() as u32;
    let mut tables = TABLES.get_or_init(Default::default).lock().unwrap_or_else(|e| e.into_inner());
    tables.entry(key).or_insert_with(|| {
        let half = key as f32 / 2048.0;
        Box::leak(vec![(0.0, LEAST), (0.5 - half, LEAST), (0.5, DEEPEST), (0.5 + half, LEAST), (1.0, LEAST)].into_boxed_slice())
    })
}

/// The veil over a shimmering word `wide` by `tall` points. Its strips are narrow enough that the
/// band's slope steps by no more than a thirty-second of its depth from one to the next, half a
/// point at the narrowest, so a word of any length has seventy or so.
pub fn shimmer_veil(wide: f32, tall: f32) -> Veil {
    let wide = wide.max(1.0);
    let half = half_band(wide, tall);
    let step = (half / 32.0).max(0.5);
    let count = (wide / step).ceil().max(1.0) as usize;
    let strips = (0..count)
        .map(|k| {
            let x = k as f32 * step;
            let w = step.min(wide - x);
            // The cycle stands at `t / period − phase`, deepest at 0.5: the band's middle is on
            // this strip's middle a fraction `(middle − START W) / (TRAVEL W)` into the period.
            let at = (x + w / 2.0 - START * wide) / (TRAVEL * wide);
            Strip { x, w, phase: (at - 0.5).rem_euclid(1.0) }
        })
        .collect();
    Veil { strips, stops: band_stops(half / (TRAVEL * wide)) }
}

/// How much of the ink a point `x` points into a word `wide` by `tall` shows at `t` seconds into
/// the band's period, as the web window's mask says (1 is the word whole, 0.45 its faintest):
/// what the veil is held to in the tests.
#[cfg(test)]
pub fn shimmer_mask(wide: f32, tall: f32, x: f32, t: f32) -> f32 {
    let middle = (START + TRAVEL * (t / SHIMMER_PERIOD).rem_euclid(1.0)) * wide;
    let off = ((x - middle).abs() / half_band(wide, tall)).min(1.0);
    1.0 - DEEPEST * (1.0 - off)
}

/// The ground a veil is drawn in: what the window shows behind the card at a point, the page
/// under the faint orange glow (`chrome::backdrop`), under the card's wash, as sRGB bytes, the
/// way the window composites them. `at` is where, and `window` how large the window is, in points.
pub fn ground_under_the_card(at: (f32, f32), window: (f32, f32)) -> [f32; 3] {
    let (page, glow, card) = ([26.0, 24.0, 23.0], [238.0, 96.0, 24.0], [36.0, 34.0, 33.0]);
    let (w, h) = (window.0.max(1.0), window.1.max(1.0));
    let d = (((at.0 - 0.72 * w) / (0.7 * w)).powi(2) + ((at.1 - 0.28 * h) / (0.7 * h)).powi(2)).sqrt();
    let lit = 0.075 * (1.0 - d).max(0.0);
    let mut out = [0.0; 3];
    for c in 0..3 {
        let under = page[c] + (glow[c] - page[c]) * lit;
        out[c] = under + (card[c] - under) * 0.55;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_eased_step_is_laid_down_in_straight_pieces_on_its_timing_function() {
        let stops = eased_stops(&[(0.0, 0.0), (0.5, 1.0), (0.75, 1.0), (1.0, 0.0)], Easing::LINEAR, 4);
        assert_eq!(stops, vec![(0.0, 0.0), (0.125, 0.25), (0.25, 0.5), (0.375, 0.75), (0.5, 1.0), (0.75, 1.0), (0.8125, 0.75), (0.875, 0.5), (0.9375, 0.25), (1.0, 0.0)]);
        // A held step is one stop, and a value is read between stops in a straight line.
        assert!((sample(&stops, 0.6) - 1.0).abs() < 1e-6);
        assert!((sample(&stops, 0.0625) - 0.125).abs() < 1e-6);
    }

    /// A CSS `cubic-bezier` at `x`, solved by halving, apart from Rune's own solver.
    fn css_bezier(x1: f64, y1: f64, x2: f64, y2: f64, x: f64) -> f64 {
        let at = |a: f64, b: f64, s: f64| 3.0 * (1.0 - s) * (1.0 - s) * s * a + 3.0 * (1.0 - s) * s * s * b + s * s * s;
        let (mut lo, mut hi) = (0.0, 1.0);
        for _ in 0..60 {
            let mid = (lo + hi) / 2.0;
            if at(x1, x2, mid) < x { lo = mid } else { hi = mid }
        }
        at(y1, y2, (lo + hi) / 2.0)
    }

    /// The lattice through one turn, against `cubic-bezier(0.77, 0, 0.175, 1)` solved apart.
    #[test]
    fn a_lattice_cell_lights_and_fades_on_the_web_windows_keyframes() {
        let stops = lattice_stops();
        let at = |x: f32| sample(stops, x);
        let css = |x: f64| css_bezier(0.77, 0.0, 0.175, 1.0, x) as f32;
        assert!((at(0.0) - 0.22).abs() < 1e-6);
        // Lighting over the first 18%, at every 60th of a second of it.
        for k in 0..=16 {
            let x = 0.18 * k as f32 / 16.0;
            let web = 0.22 + 0.78 * css(f64::from(x / 0.18));
            assert!((at(x) - web).abs() < 0.004, "lighting, {x}: {} against {web}", at(x));
        }
        assert!((at(0.18) - 1.0).abs() < 1e-6 && (at(0.3) - 1.0).abs() < 1e-6 && (at(0.42) - 1.0).abs() < 1e-6);
        // Fading from 42% to 62%.
        for k in 0..=16 {
            let x = 0.42 + 0.2 * k as f32 / 16.0;
            let web = 1.0 - 0.78 * css(f64::from((x - 0.42) / 0.2));
            assert!((at(x) - web).abs() < 0.004, "fading, {x}: {} against {web}", at(x));
        }
        assert!((at(0.62) - 0.22).abs() < 1e-6 && (at(0.9) - 0.22).abs() < 1e-6);
        // Each cell an eighth of the turn behind the one before, 186 ms.
        assert!((lattice_phase(1) - 0.125).abs() < 1e-6 && (lattice_phase(7) - 0.875).abs() < 1e-6);
        assert_eq!(LATTICE_ORBIT.iter().filter(|c| c.is_none()).count(), 1);
        assert_eq!(LATTICE_ORBIT[4], None, "the middle is the hole");
        // Every stop shows: a frame of loops alone draws a cycle only while it does.
        assert!(stops.iter().all(|&(_, a)| a > 0.001 && a <= 1.0));
    }

    #[test]
    fn a_running_mark_breathes_to_045_and_back_in_16_seconds() {
        let stops = breathe_stops();
        assert!((sample(stops, 0.0) - 1.0).abs() < 1e-6);
        assert!((sample(stops, 0.5) - 0.45).abs() < 1e-6);
        assert!((sample(stops, 0.25) - 0.725).abs() < 0.002, "ease-in-out is symmetric: half way at a quarter");
        assert!((sample(stops, 1.0) - 1.0).abs() < 1e-6);
        // As a veil over a mark standing still: nothing of it at the ends, 0.55 at the middle.
        let veil = breathe_veil_stops();
        for x in [0.0, 0.1, 0.25, 0.4, 0.5, 0.75] {
            assert!(((1.0 - sample(veil, x)) - sample(stops, x)).abs() < LEAST + 1e-6, "{x}");
        }
        assert!(veil.iter().all(|&(_, a)| a > 0.001 && a <= 1.0), "every stop shows, for a step to draw it");
    }

    /// The veil, sampled as Rune samples it, against the web window's mask at the same moment,
    /// at every point of a word: the strips are within a few hundredths of it.
    #[test]
    fn the_veil_over_a_word_is_the_web_windows_mask_at_every_moment() {
        for (wide, tall) in [(118.0f32, 20.0f32), (52.0, 20.0), (600.0, 20.0)] {
            let veil = shimmer_veil(wide, tall);
            assert!(veil.strips.len() <= 80, "{} strips over {wide}", veil.strips.len());
            let last = veil.strips.last().unwrap();
            assert!((last.x + last.w - wide).abs() < 1e-3, "the strips end where the word does");
            for frame in 0..156 {
                let t = frame as f32 / 60.0;
                for s in &veil.strips {
                    let x = (t / SHIMMER_PERIOD - s.phase).rem_euclid(1.0);
                    let shown = 1.0 - sample(veil.stops, x);
                    let web = shimmer_mask(wide, tall, s.x + s.w / 2.0, t);
                    assert!((shown - web).abs() < 0.01, "{wide}: at {t:.3} s, {:.1} pt shows {shown:.3} where the web shows {web:.3}", s.x);
                }
            }
        }
    }

    #[test]
    fn the_band_crosses_the_word_from_left_to_right_and_is_off_it_for_the_rest() {
        let (wide, tall) = (118.0, 20.0);
        // At the start of a period the band's middle is 1.26 words to the left: the word is whole.
        assert!((0..118).all(|x| shimmer_mask(wide, tall, x as f32, 0.0) > 0.999));
        // Its middle reaches the word's middle 1.76 / 3.52 of the way through, 1.3 s in.
        assert!((shimmer_mask(wide, tall, 59.0, 1.3) - 0.45).abs() < 1e-4);
        // Further right a moment later.
        assert!(shimmer_mask(wide, tall, 100.0, 1.6) < shimmer_mask(wide, tall, 100.0, 1.3));
        // Gone past its right end before the period is out.
        assert!((0..118).all(|x| shimmer_mask(wide, tall, x as f32, 2.55) > 0.999));
    }

    #[test]
    fn the_ground_is_the_cards_wash_over_the_page_and_its_glow() {
        // Away from the glow: the wash over the page.
        let far = ground_under_the_card((250.0, 640.0), (1280.0, 760.0));
        assert!((far[0] - 31.5).abs() < 0.01 && (far[1] - 29.5).abs() < 0.01 && (far[2] - 28.5).abs() < 0.01, "{far:?}");
        // At the glow's middle, a little warmer.
        let near = ground_under_the_card((0.72 * 1280.0, 0.28 * 760.0), (1280.0, 760.0));
        assert!(near[0] > far[0] + 3.0 && near[2] < far[2] + 0.1, "{near:?}");
    }
}
