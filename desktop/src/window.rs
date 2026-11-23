//! Where the window stood on the desktop and how large, kept from one launch to the next, so it
//! opens where it was left. The web window keeps none of this: it opens at one size, centred,
//! every time (`openingSize` in `main/app-size.mts`). This one is replaced whole at each
//! install, which closes and opens it, so without it every install put a window the person had
//! made large back at its first size.
//!
//! The frame is read from the window itself (`WindowHandle::placement`) when it closes, when it
//! loses the keys and when it rests, never on a frame's turn and never on a timer, and written
//! to one small file beside the note about the app's door when it changed. A window filling the
//! screen keeps the frame it goes back to as well. One left full screen or in the Dock keeps
//! what it had before: it opens as it stood before either.
//!
//! A picture drawn from a fixture, and a window pointed by hand at some engine, keep nothing on
//! disk unless `--window-file <file>` (or `MERCURY_WINDOW`) says where. (`--window` is Rune's own
//! word, for which of an app's windows a picture is of.)

use std::cell::RefCell;
use std::path::PathBuf;

use rune::windows::WindowPlacement;
use rune::{WindowHandle, WindowOptions};
use serde::{Deserialize, Serialize};

use crate::State;

/// The file's name, beside `local-door.json` in the app's own folder.
const FILE: &str = "native-window.json";

/// The least of a window, in its display's pixels, that has to be on the display for the
/// window to be left where it was kept: enough of its top band to take hold of.
const IN_SIGHT: (i64, i64) = (240, 80);

/// A window's frame on the desktop, in its display's own pixels.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    /// The frame's top-left.
    pub position: (i32, i32),
    /// The content's size.
    pub size: (u32, u32),
    /// Pixels to a point on the display it stood on.
    pub scale: f64,
    /// Whether it filled the screen (the green button's zoom). `position` and `size` are then
    /// the frame it goes back to, when one was seen before it filled the screen.
    #[serde(default)]
    pub maximized: bool,
}

/// What a window opened at a kept frame still needs, once it is on a display.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Settle {
    /// It stands where it was kept.
    Nothing,
    /// It is out of sight, on a display no longer there or moved: to the middle of the screen.
    Centre,
    /// Its display draws at another scale than the one it was kept on, so the kept pixels are
    /// another size in points: this many points, in the middle of the screen.
    Size((f32, f32)),
}

impl Frame {
    /// A frame a window can open at. Anything else was not written by this window.
    fn sound(&self) -> bool {
        let side = |v: u32| (200..=20_000).contains(&v);
        side(self.size.0) && side(self.size.1) && self.scale.is_finite() && (0.5..=8.0).contains(&self.scale)
    }

    /// The frame as the window is now.
    fn of(now: &WindowPlacement) -> Frame {
        Frame { position: now.position, size: now.content_size, scale: now.scale, maximized: now.maximized }
    }

    /// What to keep of where the window stands `now`, given what `was` kept.
    pub fn kept_from(now: &WindowPlacement, was: Option<Frame>) -> Option<Frame> {
        // Full screen and the Dock are not places to open at: what it had before stands.
        if now.full_screen || now.minimized || !Frame::of(now).sound() {
            return was;
        }
        if now.maximized {
            // The frame to go back to is the last one seen while it did not fill the screen;
            // a window first seen filling it has only that frame.
            return Some(Frame { maximized: true, ..was.unwrap_or(Frame::of(now)) });
        }
        Some(Frame::of(now))
    }

    /// The window options that open a window at this frame.
    pub fn options(&self) -> WindowOptions {
        WindowOptions { position: Some(self.position), physical_size: Some(self.size), ..WindowOptions::default() }
    }

    /// Its size in points.
    fn points(&self) -> (f32, f32) {
        ((f64::from(self.size.0) / self.scale) as f32, (f64::from(self.size.1) / self.scale) as f32)
    }

    /// What the window still needs after opening at this frame, from where it stands `now`.
    pub fn settle(&self, now: &WindowPlacement) -> Settle {
        // One filling its screen is as large as the screen and wherever it is.
        if now.maximized {
            return Settle::Nothing;
        }
        // No larger than the display it is on, which may be a smaller one than it was kept on.
        let fitted = |points: (f32, f32)| match now.display {
            Some((_, size)) => (points.0.min((f64::from(size.0) / now.scale) as f32), points.1.min((f64::from(size.1) / now.scale) as f32)),
            None => points,
        };
        if (now.scale - self.scale).abs() > 0.01 {
            return Settle::Size(fitted(self.points()));
        }
        let stands = Frame::of(now).points();
        let fits = fitted(stands);
        if fits.0 < stands.0 - 0.5 || fits.1 < stands.1 - 0.5 {
            return Settle::Size(fits);
        }
        if in_sight(now) { Settle::Nothing } else { Settle::Centre }
    }
}

/// Whether enough of the window's top band is on the display it is counted on.
fn in_sight(now: &WindowPlacement) -> bool {
    let Some((at, size)) = now.display else { return false };
    let (left, top) = (i64::from(now.position.0), i64::from(now.position.1));
    let right = left + i64::from(now.content_size.0);
    let (screen_left, screen_top) = (i64::from(at.0), i64::from(at.1));
    let (screen_right, screen_bottom) = (screen_left + i64::from(size.0), screen_top + i64::from(size.1));
    let wide = right.min(screen_right) - left.max(screen_left);
    wide >= IN_SIGHT.0.min(i64::from(now.content_size.0)) && top >= screen_top - 1 && top <= screen_bottom - IN_SIGHT.1
}

/// The frame kept, and the file it is kept in when there is one.
#[derive(Debug, Default)]
pub struct Kept {
    frame: Option<Frame>,
    file: Option<PathBuf>,
}

impl Kept {
    /// The frame kept in `file`. A file that is missing, cannot be read or holds no frame a
    /// window could open at is no frame: the window opens as a first one does.
    pub fn at(file: Option<PathBuf>) -> Kept {
        let frame = file
            .as_ref()
            .and_then(|file| std::fs::read_to_string(file).ok())
            .and_then(|text| serde_json::from_str::<Frame>(&text).ok())
            .filter(Frame::sound);
        Kept { frame, file }
    }

    pub fn get(&self) -> Option<Frame> {
        self.frame
    }

    /// Takes in where the window stands `now`, and writes it down when that changed. A file
    /// that cannot be written loses nothing but the next launch's place.
    pub fn keep(&mut self, now: &WindowPlacement) {
        let next = Frame::kept_from(now, self.frame);
        if next == self.frame {
            return;
        }
        self.frame = next;
        let (Some(file), Some(frame)) = (&self.file, &self.frame) else {
            return;
        };
        // Written beside itself and moved into place, so a window opening meanwhile never
        // reads half a file.
        let draft = file.with_extension(format!("{}.tmp", std::process::id()));
        if let Some(folder) = file.parent() {
            let _ = std::fs::create_dir_all(folder);
        }
        let written = serde_json::to_string(frame).map_err(|e| e.to_string()).and_then(|text| std::fs::write(&draft, text).map_err(|e| e.to_string()));
        if written.is_ok() {
            let _ = std::fs::rename(&draft, file);
        }
    }
}

/// Where the frame is kept: where the command line says; else beside the note the window found
/// the app's door by; else nowhere.
pub fn file(args: &[String], note: Option<&str>) -> Option<PathBuf> {
    let said = args.iter().position(|a| a == "--window-file").and_then(|at| args.get(at + 1)).cloned().or_else(|| std::env::var("MERCURY_WINDOW").ok());
    said.map(PathBuf::from).or_else(|| note.and_then(|note| std::path::Path::new(note).parent().map(|folder| folder.join(FILE))))
}

thread_local! {
    /// The window itself, for the thread that draws it: the only one that may ask where it is.
    static WINDOW: RefCell<Option<WindowHandle>> = const { RefCell::new(None) };
}

/// The native window was made, at `kept` when a frame was kept: holds on to it, and puts right
/// a kept frame that no longer suits the displays there are.
pub fn opened(window: &WindowHandle, kept: Option<Frame>) {
    WINDOW.with(|held| *held.borrow_mut() = Some(window.clone()));
    let (Some(kept), Some(now)) = (kept, window.placement()) else { return };
    match kept.settle(&now) {
        Settle::Nothing => {}
        Settle::Centre => window.center(),
        Settle::Size(points) => {
            window.resize_instant(points);
            window.center();
        }
    }
}

/// Keeps where the window stands now. Asked when it closes, loses the keys or rests.
pub fn keep(s: &mut State) {
    let Some(now) = WINDOW.with(|held| held.borrow().as_ref().and_then(WindowHandle::placement)) else { return };
    s.window.keep(&now);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("mercury-window-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        folder.join(FILE)
    }

    /// A window of 1000 by 700 points at (300, 200) on a 1920 by 1080 screen drawn at 2x.
    fn standing() -> WindowPlacement {
        WindowPlacement { position: (600, 400), content_position: (600, 400), content_size: (2000, 1400), scale: 2.0, display: Some(((0, 0), (3840, 2160))), ..WindowPlacement::default() }
    }

    #[test]
    fn a_window_keeps_the_frame_it_stands_at() {
        let kept = Frame::kept_from(&standing(), None).unwrap();
        assert_eq!((kept.position, kept.size, kept.scale, kept.maximized), ((600, 400), (2000, 1400), 2.0, false));
        // And opens at it: the same pixels, where it was.
        let options = kept.options();
        assert_eq!((options.position, options.physical_size), (Some((600, 400)), Some((2000, 1400))));
    }

    #[test]
    fn one_filling_the_screen_keeps_the_frame_it_goes_back_to() {
        let was = Frame::kept_from(&standing(), None);
        let filling = WindowPlacement { position: (0, 60), content_position: (0, 60), content_size: (3840, 2100), maximized: true, ..standing() };
        let kept = Frame::kept_from(&filling, was).unwrap();
        assert_eq!((kept.position, kept.size, kept.maximized), ((600, 400), (2000, 1400), true));
        // One first seen filling the screen has only that frame.
        let kept = Frame::kept_from(&filling, None).unwrap();
        assert_eq!((kept.position, kept.size, kept.maximized), ((0, 60), (3840, 2100), true));
        // Brought back from filling it, it keeps where it then stands.
        let back = Frame::kept_from(&standing(), Some(kept)).unwrap();
        assert_eq!((back.size, back.maximized), ((2000, 1400), false));
    }

    #[test]
    fn full_screen_and_the_dock_are_not_places_to_open_at() {
        let was = Frame::kept_from(&standing(), None);
        let full = WindowPlacement { position: (0, 0), content_size: (3840, 2160), full_screen: true, ..standing() };
        assert_eq!(Frame::kept_from(&full, was), was);
        let docked = WindowPlacement { minimized: true, ..standing() };
        assert_eq!(Frame::kept_from(&docked, was), was);
        assert_eq!(Frame::kept_from(&docked, None), None);
        // Nor is a window with no size yet.
        let none = WindowPlacement { content_size: (0, 0), ..standing() };
        assert_eq!(Frame::kept_from(&none, was), was);
    }

    #[test]
    fn a_frame_is_written_when_it_changes_and_read_by_the_next_window() {
        let file = scratch("kept");
        let mut kept = Kept::at(Some(file.clone()));
        assert_eq!(kept.get(), None);
        kept.keep(&standing());
        let next = Kept::at(Some(file.clone()));
        assert_eq!(next.get(), kept.get());
        assert_eq!(next.get().map(|frame| frame.size), Some((2000, 1400)));
        // Standing where it stood, nothing is written again.
        std::fs::remove_file(&file).unwrap();
        kept.keep(&standing());
        assert!(!file.exists());
        // Moved, it is.
        kept.keep(&WindowPlacement { position: (700, 400), ..standing() });
        assert_eq!(Kept::at(Some(file.clone())).get().map(|frame| frame.position), Some((700, 400)));
        let _ = std::fs::remove_dir_all(file.parent().unwrap());
    }

    #[test]
    fn a_file_that_holds_no_frame_a_window_could_open_at_is_no_frame() {
        let file = scratch("bad");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        for text in ["not json", r#"{"position":[0,0],"size":[0,0],"scale":2.0}"#, r#"{"position":[0,0],"size":[90000,700],"scale":2.0}"#, r#"{"position":[0,0],"size":[2000,1400],"scale":0.0}"#] {
            std::fs::write(&file, text).unwrap();
            assert_eq!(Kept::at(Some(file.clone())).get(), None, "{text}");
        }
        // One written before a window said whether it filled the screen reads as not filling it.
        std::fs::write(&file, r#"{"position":[600,400],"size":[2000,1400],"scale":2.0}"#).unwrap();
        assert_eq!(Kept::at(Some(file.clone())).get().map(|frame| frame.maximized), Some(false));
        // With no file the frame lasts as long as the window does.
        let mut here = Kept::at(None);
        here.keep(&standing());
        assert_eq!(here.get().map(|frame| frame.size), Some((2000, 1400)));
        let _ = std::fs::remove_dir_all(file.parent().unwrap());
    }

    #[test]
    fn the_frame_sits_beside_the_doors_note_unless_the_command_line_says_where() {
        let none: [String; 0] = [];
        if std::env::var("MERCURY_WINDOW").is_err() {
            assert_eq!(file(&none, None), None);
            assert_eq!(file(&none, Some("/somewhere/Mercury/local-door.json")), Some(PathBuf::from("/somewhere/Mercury/native-window.json")));
        }
        let said = ["--window-file".to_string(), "/tmp/elsewhere.json".to_string()];
        assert_eq!(file(&said, Some("/somewhere/Mercury/local-door.json")), Some(PathBuf::from("/tmp/elsewhere.json")));
    }

    #[test]
    fn a_kept_frame_that_no_longer_suits_the_displays_is_put_right() {
        let kept = Frame::kept_from(&standing(), None).unwrap();
        assert_eq!(kept.settle(&standing()), Settle::Nothing);
        // Its display is gone and it stands on none, or only a sliver of it is on one.
        assert_eq!(kept.settle(&WindowPlacement { display: None, ..standing() }), Settle::Centre);
        assert_eq!(kept.settle(&WindowPlacement { position: (3740, 400), ..standing() }), Settle::Centre);
        assert_eq!(kept.settle(&WindowPlacement { position: (600, 2150), ..standing() }), Settle::Centre);
        assert_eq!(kept.settle(&WindowPlacement { position: (600, -300), ..standing() }), Settle::Centre);
        // Half of it off the side is still a window one can take hold of.
        assert_eq!(kept.settle(&WindowPlacement { position: (2840, 400), ..standing() }), Settle::Nothing);
        // On a display that draws one pixel to the point, the kept pixels are twice the window.
        assert_eq!(kept.settle(&WindowPlacement { scale: 1.0, display: Some(((0, 0), (1920, 1080))), ..standing() }), Settle::Size((1000.0, 700.0)));
        // And the other way a window kept on such a display is no larger than the one it is on.
        let on_one = Frame { position: (0, 30), size: (1920, 1050), scale: 1.0, maximized: false };
        assert_eq!(on_one.settle(&WindowPlacement { position: (0, 30), content_size: (1920, 1050), scale: 2.0, display: Some(((0, 0), (3024, 1964))), ..standing() }), Settle::Size((1512.0, 982.0)));
        // One kept on a larger display of the same scale is brought down to this one.
        let large = Frame { position: (0, 60), size: (3840, 2100), scale: 2.0, maximized: false };
        assert_eq!(large.settle(&WindowPlacement { position: (0, 60), content_size: (3840, 2100), display: Some(((0, 0), (3024, 1964))), ..standing() }), Settle::Size((1512.0, 982.0)));
        // One filling its screen is left to the screen.
        let filling = WindowPlacement { position: (0, 60), content_size: (3840, 2100), maximized: true, ..standing() };
        assert_eq!(Frame::kept_from(&filling, Some(kept)).unwrap().settle(&filling), Settle::Nothing);
    }
}
