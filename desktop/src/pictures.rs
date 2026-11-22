//! The pictures of the open conversation, read off the disk by this window itself.
//!
//! A picture is never read on the frame. The page asks for one by the places it may be in and
//! the size it will be drawn at (`look`), draws a place for it, and a worker thread finds the
//! file, decodes it and shrinks it to that size. What comes back lands in `land`, from the
//! window's `poll`, and the page is drawn again with the picture in its place. A still picture
//! for a comparison (`--snap`) cannot wait, so there the asking does the reading.
//!
//! What is kept is small on purpose, because memory is why this window exists:
//! - A copy is no larger than the box it is drawn in, in device pixels (`Want`). A screenshot
//!   2880 pixels wide costs 20 MB whole and 6 MB at the reading column's width on a 2x screen.
//! - All the copies together stay under `BUDGET`. Past it, the ones furthest from what is on
//!   screen are let go and read again when they come back into view.
//! - Leaving a conversation lets go of every copy (`keep`). Only the sizes are remembered, a
//!   few bytes each, so a conversation opened again holds each picture's exact place at once.
//! - A file that changes on disk is read again (`look_again`).

use std::collections::{HashMap, VecDeque};
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::time::UNIX_EPOCH;

use image::{DynamicImage, ImageDecoder, ImageReader, imageops::FilterType};
use rune::{Pixels, Waker};

/// The most decoded pixels held at once: 64 MiB. A screenshot at the reading column's width on
/// a 2x screen is about 6 MB (1516 by 948 pixels, four bytes each), so this is the ten or so
/// pictures of a long answer, all at once, with room to spare, and a small share of the 180 MB
/// the whole window takes. Pictures on screen are always held, whatever this says.
pub const BUDGET: usize = 64 << 20;

/// How large a copy the page needs, in device pixels.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Want {
    /// The whole picture inside a box this wide and tall: one drawn on a line of its own.
    Within(u32, u32),
    /// This tall, whatever that makes its width: one of a row.
    Tall(u32),
    /// Filling a box this wide and tall, the overflow cropped: a folder's cover.
    Cover(u32, u32),
}

impl Want {
    /// The size of the copy for a picture of `natural` pixels. One less than a tenth larger
    /// than needed is kept whole, as the web window keeps it (`fit < 0.9` in `fit-picture.ts`).
    fn copy(self, natural: (u32, u32)) -> (u32, u32) {
        let (w, h) = (natural.0.max(1) as f32, natural.1.max(1) as f32);
        let fit = match self {
            Want::Within(bw, bh) => (bw as f32 / w).min(bh as f32 / h),
            Want::Tall(bh) => bh as f32 / h,
            Want::Cover(bw, bh) => (bw as f32 / w).max(bh as f32 / h),
        };
        if fit < 0.9 { (((w * fit).round() as u32).max(1), ((h * fit).round() as u32).max(1)) } else { natural }
    }
    fn tag(self) -> &'static str {
        match self {
            Want::Within(..) => "within",
            Want::Tall(_) => "tall",
            Want::Cover(..) => "cover",
        }
    }
}

/// A picture that has been read.
#[derive(Clone)]
pub struct Held {
    pub pixels: Pixels,
    /// The file's own size in pixels, which is its size in points.
    pub natural: (u32, u32),
    /// Where it was found.
    pub path: String,
}

/// What the page is told about one picture.
pub enum Look {
    /// On its way. With its size when this window has read it before.
    Arriving(Option<(u32, u32)>),
    Here(Held),
    /// In none of the places it was looked for, or not a picture this window can read.
    Missing,
}

/// A file's length and when it was last written: what says it changed.
type Stamp = (u64, u128);

enum State {
    Waiting,
    Here { held: Held, stamp: Stamp, bytes: usize, copy: (u32, u32) },
    Missing,
}

struct Entry {
    places: Vec<String>,
    want: Want,
    state: State,
    /// Asked of the worker and not answered yet.
    asked: bool,
    /// On screen, as far as the page has said.
    near: bool,
    /// When it last left the screen, on the store's own count of looks.
    left: u64,
}

struct Job {
    key: String,
    places: Vec<String>,
    want: Want,
    /// The file already held and its stamp: read again only if that changed.
    held: Option<(String, Stamp)>,
}

struct Read {
    path: String,
    stamp: Stamp,
    natural: (u32, u32),
    size: (u32, u32),
    rgba: Vec<u8>,
}

enum Answer {
    Read(Read),
    Missing,
    /// The file held is still the file on disk.
    Same,
}

#[derive(Default)]
struct Desk {
    jobs: VecDeque<Job>,
    done: Vec<(String, Answer)>,
    waker: Option<Waker>,
    working: bool,
}

#[derive(Default)]
struct Shelf {
    entries: HashMap<String, Entry>,
    /// Each picture's own size once read, kept when its pixels are let go.
    sizes: HashMap<String, (u32, u32)>,
    bytes: usize,
    looks: u64,
    /// The conversation these pictures belong to.
    owner: Option<String>,
}

/// The store. One per window; the page reaches it through the window's state.
pub struct Pictures {
    shelf: Mutex<Shelf>,
    desk: Arc<(Mutex<Desk>, Condvar)>,
    /// Device pixels per point.
    scale: Mutex<f32>,
    /// A still picture is being taken: read on the spot.
    still: bool,
    /// A fixture whose pictures had not come when it was photographed: none ever does.
    never: bool,
    /// When the disk was last looked at again, in milliseconds.
    looked: Mutex<f64>,
}

/// How often the files behind the pictures on the page are looked at again: every two
/// seconds, which is how soon the web window looks again for a picture it did not find. Each
/// look is one `stat` a file, on the worker.
pub const AGAIN_MS: f64 = 2000.0;

fn key(places: &[String], want: Want) -> String {
    format!("{}\n{}", want.tag(), places.join("\n"))
}

fn stamp(path: &Path) -> Option<Stamp> {
    let meta = std::fs::metadata(path).ok().filter(|m| m.is_file())?;
    let written = meta.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map_or(0, |d| d.as_nanos());
    Some((meta.len(), written))
}

/// The first of `places` that is a file, decoded and shrunk to what `want` needs.
fn read(places: &[String], want: Want) -> Answer {
    let Some((path, stamp)) = places.iter().find_map(|p| stamp(Path::new(p)).map(|s| (p, s))) else { return Answer::Missing };
    let decoded = (|| {
        let bytes = std::fs::read(path).ok()?;
        let mut reader = ImageReader::new(Cursor::new(&bytes)).with_guessed_format().ok()?;
        // A file that claims a size no screen shows is not decoded at all.
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(20_000);
        limits.max_image_height = Some(20_000);
        limits.max_alloc = Some(768 << 20);
        reader.limits(limits);
        let mut decoder = reader.into_decoder().ok()?;
        // A photograph says which way up it is, and a browser turns it so.
        let turned = decoder.orientation().ok();
        let mut picture = DynamicImage::from_decoder(decoder).ok()?;
        if let Some(turned) = turned {
            picture.apply_orientation(turned);
        }
        Some(picture)
    })();
    let Some(picture) = decoded else { return Answer::Missing };
    let natural = (picture.width(), picture.height());
    let size = want.copy(natural);
    // The whole picture is let go here, in the worker, the moment its copy is made.
    let rgba = if size == natural { picture.into_rgba8() } else { picture.resize_exact(size.0, size.1, FilterType::CatmullRom).into_rgba8() };
    Answer::Read(Read { path: path.clone(), stamp, natural, size, rgba: rgba.into_raw() })
}

fn work(job: &Job) -> Answer {
    if let Some((path, held)) = &job.held {
        // Still the first place that has it, and still the same file.
        let first = job.places.iter().find(|p| stamp(Path::new(p)).is_some());
        if first == Some(path) && stamp(Path::new(path)) == Some(*held) {
            return Answer::Same;
        }
    }
    read(&job.places, job.want)
}

impl Pictures {
    /// `scale`: device pixels per point, until the window says its own. `still`: a still
    /// picture is being taken. `never`: no picture ever comes.
    pub fn new(scale: f32, still: bool, never: bool) -> Pictures {
        Pictures { shelf: Mutex::default(), desk: Arc::default(), scale: Mutex::new(scale), still, never, looked: Mutex::new(0.0) }
    }

    /// Who to wake when a picture has been read.
    pub fn set_waker(&self, wake: Waker) {
        self.desk.0.lock().unwrap().waker = Some(wake);
    }

    pub fn set_scale(&self, scale: f32) {
        *self.scale.lock().unwrap() = scale;
    }

    /// Points as device pixels.
    pub fn pixels(&self, points: f32) -> u32 {
        (points * *self.scale.lock().unwrap()).round().max(1.0) as u32
    }

    fn send(&self, job: Job) {
        let (desk, bell) = &*self.desk;
        let mut desk = desk.lock().unwrap();
        // A newer asking for the same picture stands in for an older one still waiting.
        desk.jobs.retain(|j| j.key != job.key);
        desk.jobs.push_back(job);
        if !desk.working {
            desk.working = true;
            let shared = self.desk.clone();
            std::thread::spawn(move || {
                let (desk, bell) = &*shared;
                loop {
                    let job = {
                        let mut desk = desk.lock().unwrap();
                        loop {
                            if let Some(job) = desk.jobs.pop_front() {
                                break job;
                            }
                            desk = bell.wait(desk).unwrap();
                        }
                    };
                    let answer = work(&job);
                    // A file found unchanged is nothing to draw: it waits for the window's
                    // next turn, where anything else wakes it.
                    let news = !matches!(answer, Answer::Same);
                    let mut desk = desk.lock().unwrap();
                    desk.done.push((job.key, answer));
                    let wake = desk.waker.clone().filter(|_| news);
                    drop(desk);
                    if let Some(wake) = wake {
                        wake();
                    }
                }
            });
        }
        bell.notify_one();
    }

    /// Puts an answer on the shelf. Says whether the page now draws something else.
    fn place(shelf: &mut Shelf, key: String, answer: Answer) -> bool {
        let Some(entry) = shelf.entries.get_mut(&key) else { return false };
        entry.asked = false;
        match answer {
            Answer::Same => false,
            Answer::Missing => {
                if let State::Here { bytes, .. } = entry.state {
                    shelf.bytes -= bytes;
                }
                let was_missing = matches!(entry.state, State::Missing);
                entry.state = State::Missing;
                !was_missing
            }
            Answer::Read(read) => {
                // The same handle goes on drawing: what shows it need not be drawn again.
                let pixels = match &entry.state {
                    State::Here { held, bytes, .. } => {
                        shelf.bytes -= bytes;
                        held.pixels.clone()
                    }
                    _ => Pixels::new(),
                };
                let bytes = read.rgba.len();
                pixels.set(read.size.0, read.size.1, read.rgba);
                shelf.bytes += bytes;
                shelf.sizes.insert(key, read.natural);
                entry.state = State::Here { held: Held { pixels, natural: read.natural, path: read.path }, stamp: read.stamp, bytes, copy: read.size };
                true
            }
        }
    }

    /// Lets go of the copies furthest from the screen until what is held fits the budget.
    fn trim(shelf: &mut Shelf) {
        while shelf.bytes > BUDGET {
            let far = shelf.entries.iter().filter(|(_, e)| !e.near && matches!(e.state, State::Here { .. })).min_by_key(|(_, e)| e.left).map(|(k, _)| k.clone());
            let Some(far) = far else { break };
            let entry = shelf.entries.get_mut(&far).expect("just found");
            if let State::Here { bytes, .. } = entry.state {
                shelf.bytes -= bytes;
            }
            entry.state = State::Waiting;
        }
    }

    /// What the page draws for the picture that is the first file among `places`, at the size
    /// `want` says. Asking is what has it read.
    pub fn look(&self, places: Vec<String>, want: Want) -> Look {
        if self.never {
            return Look::Arriving(None);
        }
        let key = key(&places, want);
        let mut shelf = self.shelf.lock().unwrap();
        shelf.looks += 1;
        let (held_bytes, known) = (shelf.bytes, shelf.sizes.get(&key).copied());
        let entry = shelf.entries.entry(key.clone()).or_insert_with(|| Entry { places: places.clone(), want, state: State::Waiting, asked: false, near: false, left: 0 });
        entry.want = want;
        // The copy to go on drawing while another is made, when there is one.
        let meanwhile = match &entry.state {
            State::Missing => return Look::Missing,
            // A copy made for another size of window is made again once the size has settled
            // a tenth away from it; until then the one held is drawn.
            State::Here { held, copy, .. } => {
                let now = want.copy(held.natural);
                let off = (now.0 as f32 - copy.0 as f32).abs() > 0.1 * copy.0 as f32 || (now.1 as f32 - copy.1 as f32).abs() > 0.1 * copy.1 as f32;
                if !off || entry.asked {
                    return Look::Here(held.clone());
                }
                Some(held.clone())
            }
            State::Waiting => {
                // Past the budget only what is on screen is read; the rest holds its place.
                let cost = match (known, want) {
                    (Some(natural), _) => want.copy(natural),
                    (None, Want::Within(w, h) | Want::Cover(w, h)) => (w, h),
                    (None, Want::Tall(h)) => (h * 2, h),
                };
                let fits = held_bytes + cost.0 as usize * cost.1 as usize * 4 <= BUDGET;
                if entry.asked || !(fits || entry.near) {
                    return Look::Arriving(known);
                }
                None
            }
        };
        if self.still {
            let answer = read(&places, want);
            Self::place(&mut shelf, key.clone(), answer);
            return match &shelf.entries[&key].state {
                State::Here { held, .. } => Look::Here(held.clone()),
                _ => Look::Missing,
            };
        }
        if let Some(entry) = shelf.entries.get_mut(&key) {
            entry.asked = true;
        }
        drop(shelf);
        self.send(Job { key, places, want, held: None });
        match meanwhile {
            Some(held) => Look::Here(held),
            None => Look::Arriving(known),
        }
    }

    /// The page says a picture came on screen or left it.
    pub fn near(&self, places: &[String], want: Want, on: bool) {
        let key = key(places, want);
        let mut shelf = self.shelf.lock().unwrap();
        let looks = shelf.looks;
        if let Some(entry) = shelf.entries.get_mut(&key) {
            entry.near = on;
            if !on {
                entry.left = looks;
            }
        }
    }

    /// What the worker has read since the last call, put where the page finds it. Says
    /// whether the page should be drawn again.
    pub fn land(&self) -> bool {
        let done = std::mem::take(&mut self.desk.0.lock().unwrap().done);
        if done.is_empty() {
            return false;
        }
        let mut shelf = self.shelf.lock().unwrap();
        let mut changed = false;
        for (key, answer) in done {
            changed |= Self::place(&mut shelf, key, answer);
        }
        Self::trim(&mut shelf);
        changed
    }

    /// The conversation on screen, or none. Another one's pictures are let go.
    pub fn keep(&self, owner: Option<&str>) {
        let mut shelf = self.shelf.lock().unwrap();
        if shelf.owner.as_deref() == owner {
            return;
        }
        shelf.owner = owner.map(str::to_string);
        shelf.entries.clear();
        shelf.bytes = 0;
        self.desk.0.lock().unwrap().jobs.clear();
    }

    /// Looks at the disk again: a picture held is read again if its file changed, and one
    /// that was missing is looked for again, as the web window looks again for a picture that
    /// can land after its message (`picture-looks-again.ts`).
    fn look_again(&self) {
        let mut jobs = Vec::new();
        {
            let mut shelf = self.shelf.lock().unwrap();
            for (key, entry) in shelf.entries.iter_mut().filter(|(_, e)| !e.asked) {
                let held = match &entry.state {
                    State::Here { held, stamp, .. } => Some((held.path.clone(), *stamp)),
                    State::Missing => None,
                    State::Waiting => continue,
                };
                entry.asked = true;
                jobs.push(Job { key: key.clone(), places: entry.places.clone(), want: entry.want, held });
            }
        }
        for job in jobs {
            self.send(job);
        }
    }

    /// The window's turn, from its `poll`: `open` is the conversation on screen and `now` the
    /// clock in milliseconds. Lets go of another conversation's pictures, puts what was read
    /// where the page finds it, and every `AGAIN_MS` looks at the disk again. Says whether
    /// the page should be drawn again, and whether the window should come back for another
    /// look when nothing else brings it.
    pub fn tend(&self, open: Option<&str>, now: f64) -> (bool, bool) {
        self.keep(open);
        let changed = self.land();
        if self.still || self.never || self.shelf.lock().unwrap().entries.is_empty() {
            return (changed, false);
        }
        let mut looked = self.looked.lock().unwrap();
        if now - *looked >= AGAIN_MS {
            *looked = now;
            drop(looked);
            self.look_again();
        }
        (changed, true)
    }

    /// How many bytes of pixels are held now, and in how many pictures: for a measurement.
    pub fn held(&self) -> (usize, usize) {
        let shelf = self.shelf.lock().unwrap();
        (shelf.bytes, shelf.entries.values().filter(|e| matches!(e.state, State::Here { .. })).count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_copy_is_the_size_it_is_drawn_at() {
        // A screenshot wider than the column is shrunk to it; a strip smaller than it is kept.
        assert_eq!(Want::Within(1516, 876).copy((2400, 1350)), (1516, 853));
        assert_eq!(Want::Within(1516, 876).copy((360, 64)), (360, 64));
        // Taller than the window lets it be: the height decides.
        assert_eq!(Want::Within(1516, 876).copy((900, 1950)), (404, 876));
        // Less than a tenth too large is not worth a second copy.
        assert_eq!(Want::Within(1516, 876).copy((1600, 800)), (1600, 800));
        // One of a row is as tall as the row, and a small one is not blown up.
        assert_eq!(Want::Tall(476).copy((1200, 800)), (714, 476));
        assert_eq!(Want::Tall(476).copy((240, 64)), (240, 64));
        // A cover fills its box, so the shorter side decides.
        assert_eq!(Want::Cover(494, 308).copy((1600, 1000)), (494, 309));
        assert_eq!(Want::Cover(494, 308).copy((1000, 2000)), (494, 988));
    }

    #[test]
    fn a_file_is_asked_for_once_and_missing_when_nowhere() {
        let store = Pictures::new(2.0, true, false);
        let nowhere = vec!["/nonexistent/mercury-desktop/a.png".to_string(), "/nonexistent/mercury-desktop/designs/a.png".to_string()];
        assert!(matches!(store.look(nowhere.clone(), Want::Within(100, 100)), Look::Missing));
        assert_eq!(store.held(), (0, 0));
        // And nothing ever comes to a page photographed before its pictures did.
        let never = Pictures::new(2.0, true, true);
        assert!(matches!(never.look(nowhere, Want::Within(100, 100)), Look::Arriving(None)));
    }

    #[test]
    fn what_is_far_from_the_screen_goes_first_and_what_is_on_it_stays() {
        let mut shelf = Shelf::default();
        let here = |near: bool, left: u64, bytes: usize| Entry {
            places: Vec::new(),
            want: Want::Tall(1),
            state: State::Here { held: Held { pixels: Pixels::new(), natural: (1, 1), path: String::new() }, stamp: (0, 0), bytes, copy: (1, 1) },
            asked: false,
            near,
            left,
        };
        shelf.entries.insert("on screen".into(), here(true, 0, BUDGET / 2));
        shelf.entries.insert("left long ago".into(), here(false, 3, BUDGET / 2));
        shelf.entries.insert("left just now".into(), here(false, 9, BUDGET / 2));
        shelf.bytes = BUDGET / 2 * 3;
        Pictures::trim(&mut shelf);
        assert!(matches!(shelf.entries["left long ago"].state, State::Waiting));
        assert!(matches!(shelf.entries["left just now"].state, State::Here { .. }));
        assert!(matches!(shelf.entries["on screen"].state, State::Here { .. }));
        assert_eq!(shelf.bytes, BUDGET);
        // Nothing on screen is let go, even past the budget.
        shelf.entries.get_mut("left just now").unwrap().near = true;
        shelf.bytes = BUDGET * 2;
        Pictures::trim(&mut shelf);
        assert!(matches!(shelf.entries["left just now"].state, State::Here { .. }));
    }
}
