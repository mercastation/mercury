//! How the window answers the pointer and the hands, where the web window's stylesheet says so:
//! what a control does while the pointer rests on it. Every look here was read off the web
//! window with the pointer resting on the control (`tools/scenes/*-hover-*.mjs`), and every
//! pointer shape off its computed `cursor`.
//!
//! The web window's `:hover` rules have no transition unless one says so, so the window's root
//! lands every hover at once both ways (`hover_motion_within` in `main.rs`): Rune's own hover
//! overlay would otherwise fade out over a tenth of a second.

use rune::{Color, El};

use crate::State;
use crate::theme::*;

/// A control the web window does something with that this window draws and does not do yet:
/// it still answers the pointer as the web window's does, its look and its pointer, by taking
/// the pointer itself, which in Rune an element with nothing to press does not. A press on it
/// does nothing and goes nowhere else, as one on a button goes to the button.
pub fn answers(e: El) -> El {
    e.on_hover(|_: &mut State| {})
}

/// `--wash`: what a quiet control's ground turns under the pointer.
pub fn wash() -> Color {
    white(0.04)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use rune::Cursor;
    use serde_json::Value;

    /// The web window's word for a pointer (`cursor`), as Rune names it.
    fn shape(css: &str) -> Option<Cursor> {
        Some(match css {
            "pointer" => Cursor::Pointer,
            "text" => Cursor::Text,
            "default" => Cursor::Default,
            "zoom-in" => Cursor::ZoomIn,
            "zoom-out" => Cursor::ZoomOut,
            "not-allowed" => Cursor::NotAllowed,
            "grab" => Cursor::Grab,
            "grabbing" => Cursor::Grabbing,
            "move" => Cursor::Move,
            "crosshair" => Cursor::Crosshair,
            "col-resize" => Cursor::ResizeColumn,
            "row-resize" => Cursor::ResizeRow,
            _ => return None,
        })
    }

    /// The command line a scene's picture is drawn with, by its name, as the notes draw them.
    fn view_of(scene: &str) -> Vec<String> {
        if let Ok(own) = std::env::var("MERCURY_FEEL_ARGS") {
            return own.split_whitespace().map(str::to_string).collect();
        }
        let name = scene.rsplit('/').next().unwrap_or(scene);
        let words: &[&str] = match name {
            n if n.starts_with("thread") => &["--thread"],
            n if n.starts_with("inbox-later") => &["--view", "inbox", "--tab", "later"],
            n if n.starts_with("inbox-progress") => &["--view", "inbox", "--tab", "progress"],
            n if n.starts_with("inbox-done") => &["--view", "inbox", "--tab", "done"],
            n if n.starts_with("inbox") || n.starts_with("compose-card") || n.starts_with("palette") => &["--view", "inbox"],
            _ => &[],
        };
        words.iter().map(|w| w.to_string()).collect()
    }

    /// Walks the pointer over a scene the reference tool photographed (`tools/reference.mjs`, whose
    /// dump says which shape the web window's pointer takes at the middle of each thing on
    /// screen) and holds each point's shape here against it. A tool, not a test of the build: it
    /// reads the pictures' folder, which is never committed.
    ///
    ///   MERCURY_FEEL=shots/thread cargo test pointer -- --ignored --nocapture
    #[test]
    #[ignore]
    fn the_pointer_takes_the_web_windows_shape_everywhere() {
        let Ok(scene) = std::env::var("MERCURY_FEEL") else { return };
        let dump: Value = serde_json::from_str(&std::fs::read_to_string(format!("{scene}-dump.json")).expect("the scene's dump")).expect("a dump");
        let (w, h) = (dump["viewport"][0].as_f64().unwrap_or(1280.0) as f32, dump["viewport"][1].as_f64().unwrap_or(760.0) as f32);
        let mut args = vec!["--fixture".to_string(), format!("{scene}-fixture.json")];
        args.extend(view_of(&scene));
        let mut s = crate::init_from(&args);
        s.size = (w, h);
        let text = rune::text::TextSystem::new(crate::FONTS, ("Geist", "Geist Mono, ui-monospace"));
        let mut rt = rune::Runtime::with_text(s, crate::view, (w, h), rune::Env::new(true, 2.0), text);
        rt.frame(0.0);
        rt.update(|s, tx| crate::press_what_the_picture_pressed(s, tx));
        rt.frame(1.0 / 60.0);
        let mut wrong = Vec::new();
        let mut kinds: BTreeMap<String, (usize, usize, Vec<String>)> = BTreeMap::new();
        for p in dump["pointers"].as_array().expect("the web window's pointers") {
            let (x, y) = (p["x"].as_f64().unwrap_or(0.0) as f32, p["y"].as_f64().unwrap_or(0.0) as f32);
            let css = p["cursor"].as_str().unwrap_or("default");
            let what = format!("{}.{} {:?}", p["tag"].as_str().unwrap_or(""), p["cls"].as_str().unwrap_or(""), p["label"].as_str().unwrap_or(""));
            let Some(want) = shape(css) else {
                println!("{x},{y}: the web window's {css} has no shape here ({what})");
                continue;
            };
            // Each point is a pointer that just arrived, so nothing waiting on a resting pointer
            // (the rail opening, a hint) comes due between two of them.
            rt.pointer_move(x, y);
            rt.frame(1.0 / 60.0);
            let got = rt.cursor();
            rt.pointer_leave();
            rt.frame(1.0 / 60.0);
            if p["resting"].as_bool() == Some(true) {
                println!("where the scene's pointer rests, {x},{y}: the web window's {css} over {what}, here {got:?}");
            }
            let kind = kinds.entry(css.to_string()).or_default();
            kind.1 += 1;
            if got == want {
                kind.0 += 1;
                if !kind.2.contains(&what) {
                    kind.2.push(what);
                }
            } else {
                wrong.push(format!("{x},{y}: the web window's {css} over {what}, here {got:?}"));
            }
        }
        for (css, (same, all, over)) in &kinds {
            println!("{css}: {same} of {all}");
            if css != "default" {
                for what in over {
                    println!("    {what}");
                }
            }
        }
        for line in &wrong {
            println!("DIFFERS {line}");
        }
        assert!(wrong.is_empty(), "{} points take another shape here", wrong.len());
    }
}
