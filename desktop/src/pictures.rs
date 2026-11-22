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
//! - Leaving a conversation lets go of every copy (`keep`). Only the sizes are remembered, of
//!   the last few thousand pictures, so a conversation opened again holds each one's exact
//!   place at once.
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

/// How many pictures' sizes are remembered after their pixels are let go. Each is the places
/// the picture was looked for and two numbers, about a kilobyte.
const SIZES_KEPT: usize = 4096;

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
    /// What it is expected to take, in bytes, while it is asked for and has not come.
    claim: usize,
    /// On screen, as far as the page has said.
    near: bool,
    /// When it last left the screen, on the store's own count of looks.
    left: u64,
}

/// What a folder holds: its pictures, newest first, as far as the page shows them, and how
/// many there are in all (`main/folder-pictures.mts`).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Listing {
    /// Where the folder was found.
    pub folder: String,
    pub pictures: Vec<String>,
    pub total: usize,
}

/// What the page is told about one folder.
pub enum Listed {
    /// Being read.
    Asking,
    Here(Listing),
    /// In none of the places it was looked for.
    Nowhere,
}

struct Folder {
    places: Vec<String>,
    /// None until it has been read once.
    listing: Option<Option<Listing>>,
    asked: bool,
}

struct Job {
    key: String,
    places: Vec<String>,
    /// None for a folder, whose pictures are listed and not read.
    want: Option<Want>,
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
    /// A folder's pictures, or None when it is nowhere.
    Listed(Option<Listing>),
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
    folders: HashMap<String, Folder>,
    bytes: usize,
    /// The bytes expected of what has been asked for and has not come.
    coming: usize,
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
    /// The most bytes of pixels held at once: `BUDGET`, or less in a test.
    budget: usize,
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

/// The most pictures of a folder the page is told about (`limit` in `folderPictures`).
const LISTED: usize = 60;

/// The pictures in the first of `places` that is a folder: one level only, the newest first,
/// and of two written at the same moment the one whose name sorts first. The web app also
/// lists `.avif`, which this window cannot decode, so here it is left out.
fn list(places: &[String]) -> Option<Listing> {
    let folder = places.iter().find(|p| Path::new(p).is_dir())?;
    let mut pictures: Vec<(String, u128)> = std::fs::read_dir(folder)
        .ok()?
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let kind = path.extension()?.to_str()?.to_ascii_lowercase();
            if !matches!(kind.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp") {
                return None;
            }
            let (_, written) = stamp(&path)?;
            Some((path.to_str()?.to_string(), written))
        })
        .collect();
    pictures.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    Some(Listing { folder: folder.clone(), total: pictures.len(), pictures: pictures.into_iter().take(LISTED).map(|(path, _)| path).collect() })
}

fn work(job: &Job) -> Answer {
    let Some(want) = job.want else { return Answer::Listed(list(&job.places)) };
    if let Some((path, held)) = &job.held {
        // Still the first place that has it, and still the same file.
        let first = job.places.iter().find(|p| stamp(Path::new(p)).is_some());
        if first == Some(path) && stamp(Path::new(path)) == Some(*held) {
            return Answer::Same;
        }
    }
    read(&job.places, want)
}

impl Pictures {
    /// `scale`: device pixels per point, until the window says its own. `still`: a still
    /// picture is being taken. `never`: no picture ever comes.
    pub fn new(scale: f32, still: bool, never: bool) -> Pictures {
        Pictures { shelf: Mutex::default(), desk: Arc::default(), scale: Mutex::new(scale), still, never, looked: Mutex::new(0.0), budget: BUDGET }
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
        if let Answer::Listed(listing) = answer {
            let Some(folder) = shelf.folders.get_mut(&key) else { return false };
            folder.asked = false;
            let changed = folder.listing.as_ref() != Some(&listing);
            folder.listing = Some(listing);
            return changed;
        }
        let Some(entry) = shelf.entries.get_mut(&key) else { return false };
        entry.asked = false;
        // What it was expected to cost gives way to what it does.
        shelf.coming -= std::mem::take(&mut entry.claim);
        match answer {
            Answer::Same | Answer::Listed(_) => false,
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
                // Sizes outlive their conversation, so they are counted too: past a few
                // thousand the list starts again, and a picture seen long ago is measured anew.
                if shelf.sizes.len() >= SIZES_KEPT {
                    shelf.sizes.clear();
                }
                shelf.sizes.insert(key, read.natural);
                entry.state = State::Here { held: Held { pixels, natural: read.natural, path: read.path }, stamp: read.stamp, bytes, copy: read.size };
                true
            }
        }
    }

    /// Lets go of the copies furthest from the screen until what is held fits the budget. A
    /// still picture is not trimmed: it reads everything its one frame asks for, before the
    /// page has said what is on screen.
    fn trim(shelf: &mut Shelf, budget: usize) {
        while shelf.bytes > budget {
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
        // What is held, and what has been asked for and not come yet: both count against the
        // budget, or a page of thirty pictures would have all thirty read before the first lands.
        let (spoken_for, known) = (shelf.bytes + shelf.coming, shelf.sizes.get(&key).copied());
        let entry = shelf.entries.entry(key.clone()).or_insert_with(|| Entry { places: places.clone(), want, state: State::Waiting, asked: false, near: false, left: 0, claim: 0 });
        entry.want = want;
        let mut claim = 0;
        // The copy to go on drawing while another is made, when there is one.
        let meanwhile = match &entry.state {
            State::Missing => return Look::Missing,
            // A copy made for a smaller window is made again once the picture is drawn a tenth
            // larger than it; until then the one held is drawn. A copy larger than it now needs
            // to be is kept: two places that draw one picture at two sizes then settle on the
            // larger, and neither asks again.
            State::Here { held, copy, .. } => {
                let now = want.copy(held.natural);
                let short = now.0 as f32 > 1.1 * copy.0 as f32 || now.1 as f32 > 1.1 * copy.1 as f32;
                if !short || entry.asked {
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
                claim = cost.0 as usize * cost.1 as usize * 4;
                // A still has one frame and no word yet of what is on screen: it reads all.
                let fits = self.still || spoken_for + claim <= self.budget;
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
            entry.claim = claim;
        }
        shelf.coming += claim;
        drop(shelf);
        self.send(Job { key, places, want: Some(want), held: None });
        match meanwhile {
            Some(held) => Look::Here(held),
            None => Look::Arriving(known),
        }
    }

    /// The pictures of the folder that is the first of `places` there is. Asking is what has
    /// it read, off the main thread like a picture.
    pub fn folder(&self, places: Vec<String>) -> Listed {
        let key = format!("folder\n{}", places.join("\n"));
        let mut shelf = self.shelf.lock().unwrap();
        let folder = shelf.folders.entry(key.clone()).or_insert_with(|| Folder { places: places.clone(), listing: None, asked: false });
        if self.still && folder.listing.is_none() {
            folder.listing = Some(list(&places));
        }
        let said = match &folder.listing {
            Some(Some(listing)) => Listed::Here(listing.clone()),
            Some(None) => Listed::Nowhere,
            None => Listed::Asking,
        };
        if folder.listing.is_none() && !folder.asked {
            folder.asked = true;
            drop(shelf);
            self.send(Job { key, places, want: None, held: None });
        }
        said
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
        Self::trim(&mut shelf, self.budget);
        // `MERCURY_PICTURES=1` says what is held each time it changes, for a measurement.
        if changed && std::env::var_os("MERCURY_PICTURES").is_some() {
            let here = shelf.entries.values().filter(|e| matches!(e.state, State::Here { .. })).count();
            let near = shelf.entries.values().filter(|e| e.near).count();
            eprintln!("pictures: {here} of {} held, {:.1} MB of pixels, {near} on screen", shelf.entries.len(), shelf.bytes as f64 / (1 << 20) as f64);
        }
        changed
    }

    /// The conversation on screen, or none. Another one's pictures are let go. The page calls
    /// this before it asks for a conversation's first picture, and the window's turn calls it
    /// with the same name, or with none once no conversation is open.
    pub fn keep(&self, owner: Option<&str>) {
        let mut shelf = self.shelf.lock().unwrap();
        if shelf.owner.as_deref() == owner {
            return;
        }
        shelf.owner = owner.map(str::to_string);
        shelf.entries.clear();
        shelf.folders.clear();
        shelf.bytes = 0;
        shelf.coming = 0;
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
                jobs.push(Job { key: key.clone(), places: entry.places.clone(), want: Some(entry.want), held });
            }
            // And a folder is listed again: a round drawn since shows up beside the others.
            for (key, folder) in shelf.folders.iter_mut().filter(|(_, f)| !f.asked && f.listing.is_some()) {
                folder.asked = true;
                jobs.push(Job { key: key.clone(), places: folder.places.clone(), want: None, held: None });
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
        let nothing = {
            let shelf = self.shelf.lock().unwrap();
            shelf.entries.is_empty() && shelf.folders.is_empty()
        };
        if self.still || self.never || nothing {
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

    /// Waits for the worker, as the window does on its waker, and says what the page is told.
    fn settled(store: &Pictures, places: &[String], want: Want, owner: &str, clock: f64) -> Look {
        for _ in 0..500 {
            store.tend(Some(owner), clock);
            match store.look(places.to_vec(), want) {
                Look::Arriving(_) => std::thread::sleep(std::time::Duration::from_millis(10)),
                told => return told,
            }
        }
        panic!("the worker never answered");
    }

    #[test]
    fn a_picture_is_read_off_the_main_thread_again_when_it_changes_and_let_go_on_leaving() {
        let folder = std::env::temp_dir().join(format!("mercury-desktop-pictures-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(folder.join("designs")).unwrap();
        let file = folder.join("designs/round.png");
        let paint = |w: u32, h: u32| image::RgbImage::from_pixel(w, h, image::Rgb([40, 120, 200])).save(&file).unwrap();
        paint(400, 200);
        // Looked for in the project's folder first, where it is not, then in its designs.
        let places = vec![folder.join("round.png").to_string_lossy().to_string(), file.to_string_lossy().to_string()];
        let want = Want::Within(200, 200);
        let store = Pictures::new(2.0, false, false);
        // A conversation just opened is drawn before the window's turn hears of it: the page
        // names it, asks, and the turn that follows keeps what was asked.
        store.tend(None, 0.0);
        store.keep(Some("storefront/w-1"));
        // Asking does not read: the page is told it is on its way, with no size yet.
        assert!(matches!(store.look(places.clone(), want), Look::Arriving(None)));
        assert!(matches!(store.folder(vec![folder.join("designs").to_string_lossy().to_string()]), Listed::Asking));
        let listed = (0..500).find_map(|_| {
            store.tend(Some("storefront/w-1"), 0.0);
            std::thread::sleep(std::time::Duration::from_millis(10));
            if let Listed::Here(listing) = store.folder(vec![folder.join("designs").to_string_lossy().to_string()]) { Some(listing) } else { None }
        });
        assert_eq!(listed.map(|l| (l.total, l.pictures)), Some((1, vec![places[1].clone()])));
        let Look::Here(held) = settled(&store, &places, want, "storefront/w-1", 0.0) else { panic!("the picture was not found") };
        assert_eq!((held.natural, held.pixels.size(), held.path.as_str()), ((400, 200), (200, 100), places[1].as_str()));
        assert_eq!(store.held(), (200 * 100 * 4, 1));
        // Drawn again under the same name at another size: the next look at the disk reads it.
        paint(300, 300);
        let mut clock = 0.0;
        let redrawn = loop {
            clock += AGAIN_MS;
            assert!(clock < 500.0 * AGAIN_MS, "the changed file was never read again");
            store.tend(Some("storefront/w-1"), clock);
            std::thread::sleep(std::time::Duration::from_millis(10));
            match store.look(places.clone(), want) {
                Look::Here(held) if held.natural == (300, 300) => break held,
                _ => {}
            }
        };
        assert_eq!(redrawn.pixels.size(), (200, 200));
        // Another conversation: every copy is let go, and its size is still known on return.
        store.tend(Some("storefront/w-2"), clock);
        assert_eq!(store.held(), (0, 0));
        store.tend(Some("storefront/w-1"), clock);
        assert!(matches!(store.look(places.clone(), want), Look::Arriving(Some((300, 300)))));
        // And a picture that is taken away says so.
        settled(&store, &places, want, "storefront/w-1", clock);
        std::fs::remove_file(&file).unwrap();
        let gone = loop {
            clock += AGAIN_MS;
            assert!(clock < 1000.0 * AGAIN_MS, "the missing file was never noticed");
            store.tend(Some("storefront/w-1"), clock);
            std::thread::sleep(std::time::Duration::from_millis(10));
            if let told @ Look::Missing = store.look(places.clone(), want) {
                break told;
            }
        };
        assert!(matches!(gone, Look::Missing));
        assert_eq!(store.held(), (0, 0));
        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// Not a check but a stopwatch: how long the worker takes over one screenshot.
    /// `cargo test --release -- --ignored --nocapture a_screenshot_is_read_in` prints it.
    #[test]
    #[ignore]
    fn a_screenshot_is_read_in() {
        let folder = std::env::temp_dir().join("mercury-desktop-stopwatch");
        std::fs::create_dir_all(&folder).unwrap();
        let file = folder.join("screen.png");
        // Bands, a gradient and a grain, so the file is not one flat colour to inflate.
        image::RgbImage::from_fn(2880, 1800, |x, y| image::Rgb([(x / 12) as u8, (y / 8) as u8, ((x * 31 + y * 17) % 251) as u8])).save(&file).unwrap();
        let places = vec![file.to_string_lossy().to_string()];
        let began = std::time::Instant::now();
        let Answer::Read(read) = read(&places, Want::Within(1516, 876)) else { panic!("the screenshot was not read") };
        eprintln!("a 2880 by 1800 screenshot read and shrunk to {} by {} in {:?}, {:.1} MB held", read.size.0, read.size.1, began.elapsed(), read.rgba.len() as f64 / (1 << 20) as f64);
        assert_eq!(read.natural, (2880, 1800));
        std::fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn what_is_far_from_the_screen_goes_first_and_what_is_on_it_stays() {
        let mut shelf = Shelf::default();
        let here = |near: bool, left: u64, bytes: usize| Entry {
            places: Vec::new(),
            want: Want::Tall(1),
            state: State::Here { held: Held { pixels: Pixels::new(), natural: (1, 1), path: String::new() }, stamp: (0, 0), bytes, copy: (1, 1) },
            asked: false,
            claim: 0,
            near,
            left,
        };
        shelf.entries.insert("on screen".into(), here(true, 0, BUDGET / 2));
        shelf.entries.insert("left long ago".into(), here(false, 3, BUDGET / 2));
        shelf.entries.insert("left just now".into(), here(false, 9, BUDGET / 2));
        shelf.bytes = BUDGET / 2 * 3;
        Pictures::trim(&mut shelf, BUDGET);
        assert!(matches!(shelf.entries["left long ago"].state, State::Waiting));
        assert!(matches!(shelf.entries["left just now"].state, State::Here { .. }));
        assert!(matches!(shelf.entries["on screen"].state, State::Here { .. }));
        assert_eq!(shelf.bytes, BUDGET);
        // Nothing on screen is let go, even past the budget.
        shelf.entries.get_mut("left just now").unwrap().near = true;
        shelf.bytes = BUDGET * 2;
        Pictures::trim(&mut shelf, BUDGET);
        assert!(matches!(shelf.entries["left just now"].state, State::Here { .. }));
    }

    #[test]
    fn past_the_budget_only_what_is_on_screen_is_read_and_nothing_is_read_twice_over() {
        let folder = std::env::temp_dir().join(format!("mercury-desktop-budget-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        // Four pictures of 100 by 100, 40,000 bytes each as pixels, and room for two and a bit.
        let places: Vec<Vec<String>> = (1..=4)
            .map(|n| {
                let file = folder.join(format!("round-{n}.png"));
                image::RgbImage::from_pixel(100, 100, image::Rgb([n * 40, 90, 160])).save(&file).unwrap();
                vec![file.to_string_lossy().to_string()]
            })
            .collect();
        let want = Want::Within(100, 100);
        let store = Pictures { budget: 100_000, ..Pictures::new(1.0, false, false) };
        store.keep(Some("storefront/w-1"));
        // The page draws all four, over and over, as a page does. Two are read; the two the
        // budget has no room for hold their place and are not asked for.
        let draw = |store: &Pictures| places.iter().map(|p| matches!(store.look(p.clone(), want), Look::Here(_))).collect::<Vec<bool>>();
        let settle = |store: &Pictures, until: &dyn Fn(&[bool]) -> bool| {
            for _ in 0..500 {
                store.tend(Some("storefront/w-1"), 0.0);
                let drawn = draw(store);
                if until(&drawn) {
                    return drawn;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            panic!("the pictures never settled: {:?}", draw(store));
        };
        assert_eq!(settle(&store, &|d| d[0] && d[1]), [true, true, false, false]);
        for _ in 0..20 {
            store.tend(Some("storefront/w-1"), 0.0);
            assert_eq!(draw(&store), [true, true, false, false]);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(store.held(), (80_000, 2));
        // The person scrolls: the first leaves the screen and the fourth comes onto it. The
        // fourth is read whatever the budget says, and the first, the one gone, makes room.
        store.near(&places[1], want, true);
        store.near(&places[0], want, false);
        store.near(&places[3], want, true);
        assert_eq!(settle(&store, &|d| d[3]), [false, true, false, true]);
        for _ in 0..20 {
            store.tend(Some("storefront/w-1"), 0.0);
            assert_eq!(draw(&store), [false, true, false, true]);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(store.held(), (80_000, 2));
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
