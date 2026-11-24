//! The dithered sky behind Home, ported line for line from `renderer/src/home/sky.ts`
//! (`drawSky`, the dark branch). Square dots on a grid, four levels a channel, ordered by an
//! 8x8 Bayer matrix. The numbers are the web app's own: same seed, same order of random draws,
//! so the picture is the same picture.

pub struct Art {
    pub core: [f64; 3],
    pub hot: [f64; 3],
    pub halo: [f64; 3],
    pub ember: [f64; 3],
    pub bg: [u8; 3],
    pub y0: f64,
    pub y1: f64,
    pub bend: f64,
    pub peak: f64,
    pub fall: f64,
    pub dim: f64,
    pub core_w: f64,
    pub hot_w: f64,
    pub halo_w: f64,
    pub grain: f64,
}

/// `DARK_SKIES`' "ember": `DARK_BASE` with the EMBER palette.
pub fn ember() -> Art {
    Art {
        core: [250.0, 92.0, 28.0],
        hot: [140.0, 150.0, 90.0],
        halo: [30.0, 92.0, 175.0],
        ember: [170.0, 55.0, 20.0],
        bg: [0x13, 0x13, 0x15],
        y0: 0.25,
        y1: 1.0,
        bend: -0.1,
        peak: 0.3,
        fall: 0.9,
        dim: 0.95,
        core_w: 0.024,
        hot_w: 0.006,
        halo_w: 0.3,
        grain: 0.1,
    }
}

const BAYER: [f64; 64] = [
    0.0, 32.0, 8.0, 40.0, 2.0, 34.0, 10.0, 42.0, 48.0, 16.0, 56.0, 24.0, 50.0, 18.0, 58.0, 26.0, 12.0, 44.0, 4.0, 36.0, 14.0, 46.0,
    6.0, 38.0, 60.0, 28.0, 52.0, 20.0, 62.0, 30.0, 54.0, 22.0, 3.0, 35.0, 11.0, 43.0, 1.0, 33.0, 9.0, 41.0, 51.0, 19.0, 59.0, 27.0,
    49.0, 17.0, 57.0, 25.0, 15.0, 47.0, 7.0, 39.0, 13.0, 45.0, 5.0, 37.0, 63.0, 31.0, 55.0, 23.0, 61.0, 29.0, 53.0, 21.0,
];

struct Rnd(u64);
impl Rnd {
    fn next(&mut self) -> f64 {
        self.0 = (self.0 * 16807) % 2147483647;
        self.0 as f64 / 2147483647.0
    }
}

/// The pixels a point both skies are painted at.
pub const PER_POINT: u32 = 2;

/// Paints `art` for a card `w` by `h` points, at `dpr` pixels a point. Dots start at `from`
/// points and are fully in by `from + span`. `curve_top` lays the streak out in the card below
/// that line instead of the whole card (the Inbox's head). Returns the pixel size and
/// straight-alpha sRGB bytes.
pub fn draw(art: &Art, w: f64, h: f64, from: f64, span: f64, curve_top: f64, dpr: u32) -> (u32, u32, Vec<u8>) {
    let pw = (w * dpr as f64).round() as u32;
    let ph = (h * dpr as f64).round() as u32;
    let mut out = vec![0u8; (pw * ph * 4) as usize];
    for px in out.chunks_exact_mut(4) {
        px.copy_from_slice(&[art.bg[0], art.bg[1], art.bg[2], 255]);
    }
    let (pitch, dot) = (3.0f64, 2u32);
    let cols = (w / pitch).ceil() as i64;
    let rows = (h / pitch).ceil() as i64;
    let mut rnd = Rnd(7);
    const N: i64 = 64;
    let grid: Vec<f64> = (0..N * N).map(|_| rnd.next()).collect();
    let at = |x: i64, y: i64| grid[(y.rem_euclid(N) * N + x.rem_euclid(N)) as usize];
    let vn = |x: f64, y: f64| {
        let (xi, yi) = (x.floor(), y.floor());
        let (xf, yf) = (x - xi, y - yi);
        let (xi, yi) = (xi as i64, yi as i64);
        let (a, b, c, d) = (at(xi, yi), at(xi + 1, yi), at(xi, yi + 1), at(xi + 1, yi + 1));
        let s = xf * xf * (3.0 - 2.0 * xf);
        let t = yf * yf * (3.0 - 2.0 * yf);
        a + (b - a) * s + (c - a) * t + (a - b - c + d) * s * t
    };
    let Art { y0, y1, bend, peak, fall, dim, core_w, hot_w, halo_w, grain, .. } = *art;
    let ch = h - curve_top;
    let s_max = w.max(h);
    let levels = 4.0f64;
    let q = |v: f64, th: f64| {
        let s = (v / 255.0).clamp(0.0, 1.0) * (levels - 1.0);
        let f = s.floor();
        ((f + if s - f > th { 1.0 } else { 0.0 }) / (levels - 1.0)) * 255.0
    };
    for j in 0..rows {
        let y = j as f64 * pitch;
        let keep = ((y - from) / span).clamp(0.0, 1.0);
        if keep <= 0.0 {
            continue;
        }
        for i in 0..cols {
            let x = i as f64 * pitch;
            let t = x / w;
            let cyv = curve_top + (y0 + (y1 - y0) * t + bend * t * (1.0 - t)) * ch;
            let slope = ((y1 - y0) + bend * (1.0 - 2.0 * t)) * ch / w;
            let d = ((cyv - y) / (1.0 + slope * slope).sqrt()) / s_max;
            let n = vn(i as f64 / 22.0, j as f64 / 22.0) * 0.6 + vn(i as f64 / 6.0, j as f64 / 6.0) * 0.4;
            let along = (1.0 - (t - peak).abs() * fall).max(0.0);
            let dc = d + (vn(i as f64 / 40.0, 3.0) - 0.5) * core_w * 1.2;
            let core = (-((dc / core_w).powi(2))).exp() * (0.5 + 0.5 * n) * (0.35 + 0.65 * along);
            let hot = (-(((dc + core_w * 0.25) / hot_w).powi(2))).exp() * along * (0.6 + 0.4 * n);
            let halo = if dc > 0.0 { (-dc / halo_w).exp() * (0.3 + 0.7 * n) * (0.25 + 0.75 * along) } else { 0.0 };
            let ember = if dc < 0.0 { (dc / (core_w * 3.0)).exp() * 0.45 * n * along } else { 0.0 };
            let mix = |k: usize| (core * art.core[k] + hot * art.hot[k] + halo * art.halo[k] + ember * art.ember[k]) * dim;
            let (r, g, b) = (mix(0), mix(1), mix(2));
            let th = (BAYER[((j % 8) * 8 + (i % 8)) as usize] + 0.5) / 64.0;
            let mut fade = 1.0;
            if keep < 1.0 {
                let th2 = 0.5 * (BAYER[(((j + 3) % 8) * 8 + ((i + 5) % 8)) as usize] + 0.5) / 64.0 + 0.5 * rnd.next();
                if keep.powf(1.6) < th2 {
                    continue;
                }
                fade = keep.powf(1.2);
            }
            let (mut rr, mut gg, mut bb) = (q(r * fade, th), q(g * fade, th), q(b * fade, th));
            if rr + gg + bb < 30.0 {
                if rnd.next() >= grain * fade {
                    continue;
                }
                let m = 20.0 + rnd.next() * 24.0;
                rr = m * (0.7 + rnd.next() * 0.6);
                gg = m * (0.7 + rnd.next() * 0.5);
                bb = m * (0.7 + rnd.next() * 0.6);
            }
            let rgba = [rr as u8, gg as u8, bb as u8, 255];
            let (x0, y0p) = ((x as u32) * dpr, (y as u32) * dpr);
            for yy in y0p..(y0p + dot * dpr).min(ph) {
                for xx in x0..(x0 + dot * dpr).min(pw) {
                    let o = ((yy * pw + xx) * 4) as usize;
                    out[o..o + 4].copy_from_slice(&rgba);
                }
            }
        }
    }
    (pw, ph, out)
}

/// A painting as a pane of glass over it shows it: each pixel a Gaussian's worth of its
/// neighbours, `sigma` pixels wide (the length CSS's `blur()` takes is that deviation), and
/// the whole kept `shrink` times smaller each way. A blur that wide holds nothing finer than
/// the smaller picture can, so drawn back at the painting's size it is the blur, for a
/// sixty-fourth of the pixels at a `shrink` of 8. The bytes are blurred as they are, sRGB,
/// which is how a browser blurs what is behind an element. Past the painting's edges it reads
/// the edge again.
///
/// This is what lets glass over a sky ask Rune for no blur of its own, which costs a window
/// some 180 MB of textures at 1900 by 1040 points for as long as one blurred thing is on screen.
pub fn soft((w, h, rgba): &(u32, u32, Vec<u8>), sigma: f32, shrink: u32) -> (u32, u32, Vec<u8>) {
    let (w, h, k) = (*w as usize, *h as usize, shrink.max(1) as usize);
    if w == 0 || h == 0 {
        return (0, 0, Vec::new());
    }
    let (sw, sh) = (w.div_ceil(k), h.div_ceil(k));
    // Each smaller pixel is the mean of the block it stands for.
    let mut small = vec![[0f32; 3]; sw * sh];
    for sy in 0..sh {
        let rows = (sy * k)..((sy + 1) * k).min(h);
        for sx in 0..sw {
            let cols = (sx * k)..((sx + 1) * k).min(w);
            let mut sum = [0u32; 3];
            for y in rows.clone() {
                let line = &rgba[(y * w + cols.start) * 4..(y * w + cols.end) * 4];
                for px in line.chunks_exact(4) {
                    sum[0] += u32::from(px[0]);
                    sum[1] += u32::from(px[1]);
                    sum[2] += u32::from(px[2]);
                }
            }
            let n = (rows.len() * cols.len()) as f32;
            small[sy * sw + sx] = [sum[0] as f32 / n, sum[1] as f32 / n, sum[2] as f32 / n];
        }
    }
    // The Gaussian, in the smaller picture's pixels, across and then down.
    let s = (sigma / k as f32).max(0.01);
    let reach = (3.0 * s).ceil() as isize;
    let weights: Vec<f32> = (-reach..=reach).map(|d| (-(d * d) as f32 / (2.0 * s * s)).exp()).collect();
    let whole: f32 = weights.iter().sum();
    let pass = |from: &[[f32; 3]], across: bool| {
        let mut to = vec![[0f32; 3]; sw * sh];
        for y in 0..sh {
            for x in 0..sw {
                let mut sum = [0f32; 3];
                for (i, weight) in weights.iter().enumerate() {
                    let d = i as isize - reach;
                    let at = if across { y * sw + (x as isize + d).clamp(0, sw as isize - 1) as usize } else { (y as isize + d).clamp(0, sh as isize - 1) as usize * sw + x };
                    let px = from[at];
                    sum[0] += px[0] * weight;
                    sum[1] += px[1] * weight;
                    sum[2] += px[2] * weight;
                }
                to[y * sw + x] = [sum[0] / whole, sum[1] / whole, sum[2] / whole];
            }
        }
        to
    };
    let blurred = pass(&pass(&small, true), false);
    let mut out = Vec::with_capacity(sw * sh * 4);
    for px in blurred {
        out.extend_from_slice(&[px[0].round().clamp(0.0, 255.0) as u8, px[1].round().clamp(0.0, 255.0) as u8, px[2].round().clamp(0.0, 255.0) as u8, 255]);
    }
    (sw as u32, sh as u32, out)
}

/// A painting with its colours made `by` times as strong, as CSS's `saturate()` makes them
/// (the filter's own matrix, on the bytes as they are). The web window's glass is
/// `blur(20px) saturate(1.5)`, and Rune's blur has no such step: a sky blurred here can take it.
pub fn saturated((w, h, mut rgba): (u32, u32, Vec<u8>), by: f32) -> (u32, u32, Vec<u8>) {
    let (r, g, b) = (0.213, 0.715, 0.072);
    for px in rgba.chunks_exact_mut(4) {
        let (x, y, z) = (f32::from(px[0]), f32::from(px[1]), f32::from(px[2]));
        let grey = r * x + g * y + b * z;
        px[0] = (grey + (x - grey) * by).round().clamp(0.0, 255.0) as u8;
        px[1] = (grey + (y - grey) * by).round().clamp(0.0, 255.0) as u8;
        px[2] = (grey + (z - grey) * by).round().clamp(0.0, 255.0) as u8;
    }
    (w, h, rgba)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saturating_leaves_grey_alone_and_pulls_a_colour_from_its_grey() {
        let (_, _, px) = saturated((2, 1, vec![90, 90, 90, 255, 200, 100, 50, 255]), 1.5);
        assert_eq!(&px[..4], &[90, 90, 90, 255]);
        // 200, 100, 50 has a grey of 117.7: each channel moves half as far again from it.
        assert_eq!(&px[4..], &[241, 91, 16, 255]);
    }

    #[test]
    fn a_soft_painting_is_smaller_and_keeps_a_flat_colour_and_spreads_a_bright_spot() {
        // A flat painting stays the colour it was, at an eighth of its size each way.
        let flat: Vec<u8> = [40u8, 30, 20, 255].repeat(64 * 32);
        let (w, h, px) = soft(&(64, 32, flat), 8.0, 8);
        assert_eq!((w, h), (8, 4));
        assert!(px.chunks_exact(4).all(|p| p == [40, 30, 20, 255]));
        // One bright square in the dark spreads: fainter where it was, and seen beside it.
        let mut spot = vec![0u8; 64 * 64 * 4];
        for y in 24..40 {
            for x in 24..40 {
                spot[(y * 64 + x) * 4..(y * 64 + x) * 4 + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
        let (w, _, px) = soft(&(64, 64, spot), 8.0, 4);
        let at = |x: usize, y: usize| px[(y * w as usize + x) * 4];
        assert!(at(7, 7) < 255 && at(7, 7) > at(4, 7) && at(4, 7) > at(1, 7));
        // A size that is no whole number of blocks keeps its last, narrower block.
        assert_eq!(soft(&(10, 3, vec![9; 120]), 2.0, 4).0, 3);
        assert_eq!(soft(&(0, 0, Vec::new()), 2.0, 4), (0, 0, Vec::new()));
    }
}
