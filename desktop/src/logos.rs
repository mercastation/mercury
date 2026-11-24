//! The tools' own logos in colour, one file per catalogue id (`renderer/src/connections/logos`:
//! from gilbarbara/logos, CC0; HubSpot and Stripe are the Simple Icons marks, CC0). The copies in
//! `assets/logos` are the web window's files, byte for byte, and a test holds them to it.
//!
//! A card that asks for a sign-in wears the service's logo in its mark when its link is the
//! service's own (`need.rs`).

use std::cell::RefCell;
use std::collections::HashMap;

use rune::Pixels;

macro_rules! files {
    ($($id:literal),* $(,)?) => {
        &[$(($id, include_bytes!(concat!("../assets/logos/", $id, ".svg")) as &[u8])),*]
    };
}

static FILES: &[(&str, &[u8])] = files![
    "airtable",
    "asana",
    "atlassian",
    "aws",
    "clickup",
    "expo",
    "figma",
    "github",
    "gmail",
    "google-calendar",
    "google-drive",
    "hubspot",
    "intercom",
    "linear",
    "monday",
    "notion",
    "npm",
    "salesforce",
    "sentry",
    "slack",
    "stripe",
    "supabase",
    "vercel",
];

/// Black or near-black marks, drawn light on the dark page so they do not vanish (`DARK_INK`,
/// and `.need-mark-ink img { filter: brightness(0) invert(.94) }`).
const DARK_INK: [&str; 5] = ["github", "intercom", "sentry", "vercel", "expo"];
/// What that filter leaves of every pixel: 94% of white, at the pixel's own alpha.
const LIGHT_INK: u8 = 240;

thread_local! {
    /// Each logo as drawn, by its id and the pixels of the square it was fitted to.
    static DRAWN: RefCell<HashMap<(&'static str, u32), Option<Pixels>>> = RefCell::new(HashMap::new());
}

/// Whether a mark that wears `id` draws its picture light (`DARK_INK.has(id)`).
pub fn light_ink(id: &str) -> bool {
    DARK_INK.contains(&id)
}

/// The logo of the service `id`, fitted inside a square of `points` on a screen of `scale`
/// pixels a point, keeping its shape; nothing for a service the catalogue has no logo for.
pub fn of(id: &str, points: f32, scale: f32) -> Option<Pixels> {
    let (name, bytes) = FILES.iter().find(|(name, _)| *name == id)?;
    let side = (points * scale.max(1.0)).round().max(1.0) as u32;
    DRAWN.with(|drawn| {
        drawn
            .borrow_mut()
            .entry((name, side))
            .or_insert_with(|| {
                let (wide, tall, mut rgba) = rune::image::svg_rgba(bytes, side).ok()?;
                if light_ink(name) {
                    for pixel in rgba.chunks_exact_mut(4) {
                        pixel[..3].fill(LIGHT_INK);
                    }
                }
                let pixels = Pixels::new();
                pixels.set(wide, tall, rgba);
                Some(pixels)
            })
            .clone()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_logo_draws_and_fits_its_square() {
        for (id, _) in FILES {
            let logo = of(id, 20.0, 2.0).unwrap_or_else(|| panic!("{id} does not draw"));
            let (wide, tall) = logo.size();
            assert!(wide.max(tall) == 40 && wide.min(tall) > 0, "{id}: {wide} by {tall}");
        }
        // A service the catalogue has no logo for wears the key (`logoOf` answering null).
        assert!(of("cloudflare", 20.0, 2.0).is_none());
    }

    #[test]
    fn a_dark_mark_is_drawn_light_and_a_coloured_one_keeps_its_colours() {
        let lit = of("github", 20.0, 2.0).unwrap().rgba();
        assert!(lit.chunks_exact(4).all(|p| p[3] == 0 || p[..3] == [LIGHT_INK; 3]));
        assert!(lit.chunks_exact(4).any(|p| p[3] == 255));
        let coloured = of("supabase", 20.0, 2.0).unwrap().rgba();
        assert!(coloured.chunks_exact(4).any(|p| p[3] == 255 && p[1] > p[0] && p[1] > p[2]), "Supabase's green is gone");
    }

    /// The copies are the web window's own files: a logo the web app changes or adds is one
    /// this window is told about by this test, in a checkout that has both.
    #[test]
    fn the_logos_are_the_web_windows_own() {
        let web = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../renderer/src/connections/logos");
        let Ok(theirs) = std::fs::read_dir(&web) else { return };
        let mut names: Vec<String> = theirs.filter_map(|f| f.ok()).filter_map(|f| f.file_name().to_str().and_then(|n| n.strip_suffix(".svg")).map(str::to_string)).collect();
        names.sort();
        assert_eq!(names, FILES.iter().map(|(id, _)| id.to_string()).collect::<Vec<_>>());
        for (id, bytes) in FILES {
            assert_eq!(std::fs::read(web.join(format!("{id}.svg"))).ok().as_deref(), Some(*bytes), "{id}.svg differs from the web window's");
        }
    }
}
