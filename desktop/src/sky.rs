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
        bg: [0x1b, 0x19, 0x17],
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
