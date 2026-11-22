//! One conversation, to read: its name, what was asked, what the agent answered and what was
//! said since, with the reply box at the foot. Built from the web app's own measurements of
//! its thread page (`renderer/src/components/ItemThread.tsx`, `Thread.tsx`,
//! `OneThingAtATime.tsx` and their stylesheets).
//!
//! The page opens the way the web app's does: on the newest answer alone, with the rest of the
//! conversation folded behind one line under the title. Pressing that line unfolds it. What
//! each run did is in the conversation (`Thread.tsx`): what it said, at reading size, and what
//! it ran folded to one quiet line under that, which a press opens to each thing it ran. While
//! an agent is on the thread, a line at the foot says what it is doing now (`Live.tsx`).
//!
//! A thread that made or named files carries a Files button in its head strip, with how many,
//! which opens the list of them (`ThreadFiles.tsx`).

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use rune::time::LocalTime;
use rune::{Drag, El, Icon, Phase, ScrollSpec, TextCase, TextStyle, col, el, icon, row, spacer, text, text_input};
use serde_json::{Map, Value, json};

use crate::attach::Holder;
use crate::data::{Item, Tab};
use crate::files::Files;
use crate::home;
use crate::markdown::{self, Block};
use crate::rules::format::{self, Choice};
use crate::rules::notes::TraceSession;
use crate::rules::thread_files::{self, ThreadFile};
use crate::rules::{live, place, work};
use crate::talk::{self, Answers, Field, Line, Message, Page, Said, Saying, Step, Talk, Work};
use crate::theme::*;
use crate::{State, View, chrome, icons, reply, strip};

/// What the person has unfolded while reading: the threads whose earlier messages are shown,
/// those whose whole middle was asked for, the folds of what a run did that are open, and the
/// conversation whose list of files is open.
#[derive(Default)]
pub struct Reading {
    unfolded: HashSet<String>,
    whole: HashSet<String>,
    runs_open: HashSet<String>,
    /// By `product/id`, one at most, and only while that conversation is on screen: the web
    /// window's list is the button's own state, which leaving the thread lets go of.
    files: Option<String>,
    /// The file of the open list under the pointer.
    file_lit: Option<usize>,
    /// The conversation, by `product/id`, whose working line is on screen: its loops run only
    /// while it is, since a loop nobody can see would have the window draw the whole page
    /// every frame (`on_visible`).
    live_seen: Option<String>,
    /// The run, by `product/id|started`, whose line was first seen, and when on the window's
    /// clock: the web window's mark and band start their loops as the line mounts, and so do
    /// these, from then on, the line scrolled away and back or not.
    live_from: Option<(String, f64)>,
    /// The run whose line is pressed open onto its steps, and the steps opened onto their
    /// commands, by the run and their place and words. A new run starts folded (`expandedAt`).
    live_open: Option<String>,
    live_rows_open: HashSet<String>,
}

impl Reading {
    /// Whether a conversation's whole middle has been asked for: from the first time its
    /// earlier messages were unfolded, every run's whole log is read, as in the web window.
    pub fn whole(&self, thread: &str) -> bool {
        self.whole.contains(thread)
    }
}

fn key(product: &str, id: &str) -> String {
    format!("{product}/{id}")
}

/// The page's measures, which follow the card's width (`one-thing-at-a-time.css`): the type
/// grows a little with the card, the column is a fixed measure of that type, and it starts a
/// third of the way into the room left over.
struct Measure {
    /// The reading size: 16 points on a 1280 window, at most 18.
    size: f32,
    column: f32,
    indent: f32,
    /// The room right of the column and under the reply box.
    right: f32,
    foot: f32,
    /// The title's size and line, the room under it, and the newest answer's headline's size and
    /// the room under it.
    title: (f32, f32),
    title_gap: f32,
    headline: (f32, f32),
}

impl Measure {
    fn of(card: f32) -> Measure {
        let size = (0.0025 * card + 13.0).clamp(16.0, 18.0);
        let column = (size * 47.5).min(card - 96.0);
        Measure { size, column, indent: ((card - column) * 0.35).max(32.0), right: 32.0, foot: 24.0, title: (1.7 * size, 2.125 * size), title_gap: 8.0, headline: (kept(1.05 * size), 10.0) }
    }
    /// Beside a file (`[data-artifact-layout="beside"] .focus`): 44 points in, lined up with the
    /// back arrow, 28 from the file, and the page's own sizes rather than the measure that grows
    /// with the window (`.app:not(.doc-open)` in `one-thing-at-a-time.css`): a smaller title
    /// where the conversation is 560 points or less (`@container talk`).
    fn beside(wide: f32) -> Measure {
        let title = if wide <= 560.0 { (21.0, 27.3) } else { (27.0, 33.75) };
        Measure { size: 16.0, column: (wide - 72.0).max(1.0), indent: 44.0, right: 28.0, foot: 24.0, title, title_gap: 6.0, headline: (17.0, 8.0) }
    }
    /// Under a file that takes the whole card: the reply box twelve points in all round.
    fn under_file(wide: f32) -> Measure {
        Measure { size: 16.0, column: (wide - 24.0).max(1.0), indent: 12.0, right: 12.0, foot: 12.0, title: (27.2, 34.0), title_gap: 6.0, headline: (17.0, 8.0) }
    }
}

fn item<'a>(s: &'a State, id: &str, product: &str) -> Option<&'a Item> {
    s.snap.items.iter().find(|i| i.id == id && i.product == product)
}

fn name(item: &Item) -> String {
    crate::rules::title::row_title(item)
}

/// The name as the top strip shows it: whole up to 59 letters, else cut at a word with no
/// small word left dangling (`crumbName`).
fn crumb(title: &str) -> String {
    const SMALL: [&str; 22] = ["a", "an", "the", "for", "to", "of", "on", "in", "at", "and", "or", "but", "so", "by", "with", "from", "before", "after", "until", "it", "as", "is"];
    let name = title.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.chars().count() <= 59 {
        return name;
    }
    let mut kept: Vec<&str> = Vec::new();
    let mut length = 0;
    for word in name.split(' ') {
        let with = length + word.chars().count() + usize::from(!kept.is_empty());
        if with > 59 {
            break;
        }
        kept.push(word);
        length = with;
    }
    if kept.is_empty() {
        return format!("{}…", name.chars().take(59).collect::<String>());
    }
    while kept.len() > 1 && SMALL.contains(&kept[kept.len() - 1].to_lowercase().trim_matches(|c: char| !c.is_ascii_lowercase())) {
        kept.pop();
    }
    format!("{}…", kept.join(" ").trim_end_matches([',', ';', ':', '.', '-', '–']))
}

/// The tab that holds the thread now, not the one it was opened from, so a thread that finishes
/// while it is read stops saying In progress; none for a thread in no tab (`crumbTab` in
/// `renderer/src/threads/page-rules.ts`). A tab holds it whether or not the Inbox's filters show
/// it there.
fn holding_tab(s: &State, id: &str, product: &str) -> Option<Tab> {
    let holds = |tab: Tab| s.lists.holds(tab, id, product);
    if holds(Tab::Progress) {
        Some(Tab::Progress)
    } else if holds(Tab::Later) {
        Some(Tab::Later)
    } else if holds(Tab::Done) && !holds(Tab::Needs) {
        Some(Tab::Done)
    } else if holds(Tab::Needs) {
        Some(Tab::Needs)
    } else {
        None
    }
}

/// The tab the strip names. Needs you is called the Inbox there, as is a thread in no tab.
fn crumb_from(s: &State, id: &str, product: &str) -> &'static str {
    match holding_tab(s, id, product) {
        Some(Tab::Needs) | None => "Inbox",
        Some(tab) => tab.label(),
    }
}

/// How tall the strip over a conversation is, and the side of its square buttons and the
/// height of its worded ones (`.ts-more`, `.ts-done`, `.ts-more.one-step`).
pub const STRIP: f32 = 64.0;
const BUTTON: f32 = 32.0;
/// What the caret beside Send says: its menu, a reply held for a time or repeated, is the web
/// window's, and a press says so rather than going nowhere.
pub const SEND_LATER_NOT_HERE: &str = "A reply is sent later, or repeated, from Mercury's own window.";

/// The card's top strip over a conversation: the Inbox, back and forward, where the thread
/// is, the threads before and after it in Needs you, and what can be done to it.
pub fn head(s: &State, id: &str, product: &str) -> El {
    let (card_w, _) = s.card_size();
    let square = |glyph: &'static Icon, size: f32, side: f32, color| row().size(side, side).radius(6.0).center().child(icon(glyph, size).color(color).group_color(ink()));
    let at = s.lists.needs.iter().position(|r| r.id == id && r.product == product);
    // An arrow with a thread to go to brightens on a ground of its own under the pointer
    // (`.ts-more.one-step:hover`) and carries its key's hint; one with none is faded and keeps
    // the arrow pointer.
    let step = |glyph: &'static Icon, hint: &'static str, to: Option<&crate::data::Row>| {
        let b = square(glyph, 14.0, BUTTON, dim());
        match to {
            Some(r) => {
                let (id, product) = (r.id.clone(), r.product.clone());
                let b = b.hover_group().hover(crate::feel::wash_strong()).cursor(rune::Cursor::Pointer).on_press(move |s: &mut State| {
                    crate::feel::pressed(s);
                    s.step_to_thread(&id, &product)
                });
                crate::feel::hinted(b, hint, format!("strip-{hint}"), crate::feel::Align::Right, None)
            }
            // Kept from the strip under it, which moves the window (`.topbar button { no-drag }`).
            None => b.opacity(0.35).no_window_drag(),
        }
    };
    let crumbs = mono(11.0, 14.0).tracking(0.99);
    let title = item(s, id, product).map(name).unwrap_or_default();
    let finished = item(s, id, product).is_some_and(|i| i.status == "done");
    let close = crate::feel::strip_button(row().h(BUTTON).items_center().gap(7.0).pl(9.0).pr(12.0).radius(6.0).border(1.0, rule()).border_layout(), true);
    let close = if finished {
        close.child(label("Reopen", 13.0, 400.0, dim()).group_color(ink())).on_press(|s: &mut State| s.reopen_thread())
    } else {
        let done = close.child(icon(&icons::DONE, 16.0).color(dim()).group_color(ink())).child(label("Done", 13.0, 400.0, dim()).group_color(ink())).on_press_tx(|s: &mut State, tx| {
            crate::feel::pressed(s);
            s.close_thread(false, tx)
        });
        crate::feel::hinted(done, "done", "strip-done", crate::feel::Align::Right, None)
    };
    let open = s.strip_open().cloned();
    // A button of the strip whose menu or panel is up is brighter, on the wash
    // (`.ts-more[aria-expanded="true"]`, `.ts-place.open`); every one wears the strong line, as
    // the Inbox strip's buttons do (`.ts-more`, `.ts-done`).
    let lit = |b: El, on: bool| if on { b.border(1.0, rule()).fill(white(0.08)) } else { b.border(1.0, rule()) };
    // Where the thread runs, for one the cloud could take (`PlaceButton.tsx`): a cloud or a Mac,
    // faded while the whole cloud is off, dotted when it was handed back for something. A press
    // opens the panel that says why and offers the other place; under the pointer it takes the
    // strip's wash and its hint, as its neighbours do.
    let placing = open == Some(strip::Over::Place);
    let place = item(s, id, product).and_then(|i| strip::place_view(s, i)).map(|v| {
        let m = place::mark_of(&v);
        let glyph = icon(if m.cloud { &icons::PLACE_CLOUD } else { &icons::PLACE_MAC }, 16.0).color(if placing { ink() } else { dim() }).group_color(ink()).opacity(if m.faded { 0.7 } else { 1.0 });
        let button = lit(crate::feel::strip_button(row().size(BUTTON, BUTTON).radius(6.0).center(), false).child(glyph), placing).label(if m.faded { "Runs on: This Mac, the cloud is off" } else if m.cloud { "Runs on: Cloud" } else { "Runs on: This Mac" }).expanded(placing).on_background_press(|s: &mut State| {
            crate::feel::pressed(s);
            s.toggle_place()
        });
        // The dot four points in from the top right corner (`.ts-place-dot`).
        let button = if m.dot { button.child(el().absolute(BUTTON - 10.0, 4.0).size(6.0, 6.0).radius(3.0).fill(accent())) } else { button };
        // Laid out under the key the panel hangs from.
        crate::feel::hinted(button, "runs-on", strip::PLACE_BUTTON, crate::feel::Align::Right, None)
    });
    // The dots, which the web window draws over every thread of work: its menu always holds the
    // terminal and renaming there, which this window does not do (`threadMenuRows`).
    let more_open = matches!(open, Some(strip::Over::More { .. }));
    let more = lit(crate::feel::strip_button(square(&icons::MORE, 16.0, BUTTON, if more_open { ink() } else { dim() }), false), more_open)
        .label("More actions")
        .expanded(more_open)
        .on_background_press_tx(|s: &mut State, tx| {
            crate::feel::pressed(s);
            s.toggle_more(tx)
        });
    // Laid out under the key the menu hangs from.
    let more = crate::feel::hinted(more, "more", strip::MORE_BUTTON, crate::feel::Align::Right, None);
    let mut actions = row().ml(16.0).min_w(0.0).items_center();
    // The line about what was just done stands first in this group (`.toast-in-bar`), and the
    // thread's name gives way to it.
    // Or, over a thread a send moved the person to, the line saying where it went (`SentLine`).
    // Six points after the line, sixteen after the sent line, and the strip's ten after either
    // where the arrows follow (`.toast-in-bar`, `.ts-top > .sent-line`, `.ts-top { gap }`).
    let arrows = if at.is_some() { 10.0 } else { 0.0 };
    if let Some(line) = crate::answer::sent_line(s) {
        actions = actions.child(line.mr(16.0 + arrows));
    } else if let Some(line) = chrome::toast_in_bar(s) {
        actions = actions.child(line.mr(6.0 + arrows));
    }
    // The threads before and after it are those of Needs you, so a thread elsewhere has none
    // (`QueueNav` stands only on a thread with a place in that queue).
    if at.is_some() {
        actions = actions
            .child(step(&icons::UP, "previous-thread", at.and_then(|a| a.checked_sub(1)).and_then(|a| s.lists.needs.get(a))))
            .child(step(&icons::DOWN, "next-thread", at.and_then(|a| s.lists.needs.get(a + 1))));
    }
    // What can be done to the thread, ten points apart (`.ts-more-wrap`): the files it made, for
    // one that made any, where it runs, Done or Reopen, and the dots. A press on any of them
    // is theirs while the strip has something open (`strip::layer`).
    let mut group = row().key(strip::GROUP).items_center().gap(10.0);
    let files = item(s, id, product).map(|i| files_made_here(s, i)).unwrap_or_default();
    if !files.is_empty() {
        group = group.child(crate::feel::hinted(files_button(files.len(), s.files_open()), "thread-files", FILES_BUTTON, crate::feel::Align::Right, None));
    }
    let group = group.children(place).child(close).child(more);
    let actions = actions.child(group.ml(10.0));
    // The arrows at the strip's left end take `--film-strong` under the pointer, and only while
    // there is somewhere to go (`.back-forward button:not(:disabled):hover`).
    let arrow = |e: El| e.hover(crate::feel::film()).cursor(rune::Cursor::Pointer);
    // The strip moves the window as the title bar would (`.topbar`, `-webkit-app-region: drag`),
    // and every button keeps its press (`no-drag`), as does all of the middle, where the crumb
    // and the name stand (`.workspace-task-header { no-drag }`). It starts where the card does,
    // further right while the rail is open. It is the Inbox strip's 64 points, so the arrows do
    // not move between the two (`.workspace-task:has(.thread-crumb) > .topbar`).
    // With a file beside the conversation its arrows stand sixteen points further in
    // (`[data-artifact-layout="beside"] > .topbar` in `team/rooms/rooms.css`).
    let beside = s.pane_placement() == Some(crate::rules::pane::Placement::Beside);
    row().absolute(chrome::card_left(s), chrome::CARD[1])
        .size(card_w, STRIP)
        .items_center()
        .pl(if beside { 36.0 } else { 20.0 })
        .pr(28.0)
        .border_sides([0.0, 0.0, 1.0, 0.0], edge())
        .border_layout()
        .window_drag()
        // The Inbox's first tab, from its first row (`openView('inbox')`).
        .child(
            crate::feel::hinted(
                arrow(square(&icons::INBOX, 15.0, 26.0, ink())).on_press(|s: &mut State| {
                    crate::feel::pressed(s);
                    s.go(View::Inbox);
                    s.show_tab(Tab::Needs);
                }),
                "inbox",
                "strip-inbox",
                crate::feel::Align::Left,
                None,
            )
            .mr(10.0),
        )
        // Back is where the person came from, and a thread opened from nowhere came from the Inbox.
        .child(arrow(square(&icons::BACK, 15.0, 26.0, ink())).ml(2.0).help(chrome::back_title(s)).on_press(|s: &mut State| s.leave_thread()))
        .child(square(&icons::FORWARD, 15.0, 26.0, muted()).ml(2.0).opacity(0.45).no_window_drag())
        .child(
            row().grow(1.0)
                .min_w(0.0)
                .h(28.0)
                .ml(12.0)
                .items_center()
                .no_window_drag()
                // The tab's word is the way back, as Escape is, onto the tab that holds the
                // thread now (`ThreadCrumb`'s back half, `closeThreadOntoItsTab`): a solid ground
                // under the pointer, from its left edge to eight points past its words
                // (`.thread-crumb-back`).
                .child(crate::feel::hinted(
                    row().h(28.0).pr(8.0).items_center().radius(6.0).hover(rgb(46, 44, 43)).cursor(rune::Cursor::Pointer).child(text(crumb_from(s, id, product)).font(crumbs).text_case(TextCase::Uppercase).color(muted())).on_press(|s: &mut State| {
                        crate::feel::pressed(s);
                        s.close_onto_its_tab()
                    }),
                    "back",
                    "strip-crumb",
                    crate::feel::Align::Left,
                    None,
                ))
                .child(text("/").font(crumbs).color(muted()).ml(5.0).mr(7.0))
                // The name gives way to what can be done to the thread, cut short with dots
                // (`.thread-crumb-name`): Rune's flex never shrinks a child below what it holds
                // unless told to, and a strip with a Files button in it can leave the name less.
                // It is the box to rename the thread in, so the pointer is the I-beam over it and
                // its words brighten under it (`button.thread-crumb-name:hover`). Renaming is the
                // web window's, so a press on it does nothing here.
                // Its title is the whole name, which the strip may cut.
                .child(crate::feel::answers(row().min_w(0.0).shrink().px(6.0).hover_group().cursor(rune::Cursor::Text).help(title.as_str()).child(text(crumb(&title)).font(crumbs).text_case(TextCase::Uppercase).color(dim()).group_color(ink()).truncate()))),
        )
        .child(actions)
}

/* ------------------------------ the files it made ------------------------------ */

/// The key the Files button is laid out under: its list hangs from where it is.
const FILES_BUTTON: &str = "thread-files";

thread_local! {
    /// The files last read off a thread's ledger and what they were read from. The strip is drawn
    /// every frame, and they are read again only when the ledger or the notes change, as the web
    /// window's `useMemo` reads them.
    static FILES: std::cell::RefCell<Option<(String, Rc<Vec<ThreadFile>>)>> = const { std::cell::RefCell::new(None) };
}

/// The files a thread made or named, each once at its newest version, newest first
/// (`filesMadeHere` in `Focus.tsx`). Read off its ledger as the engine answered it, the lines
/// `zero:item-history` hands over, and nothing until it has: the web window's list starts empty
/// and fills when that answer comes.
pub fn files_made_here(s: &State, item: &Item) -> Rc<Vec<ThreadFile>> {
    let thread = key(&item.product, &item.id);
    let Some(answered) = s.threads.get(&thread).or_else(|| s.fixture.get("threads").and_then(|t| t.get(&thread))) else {
        return Rc::default();
    };
    // The notes on the thread's files, by the version each is on (`item.artifacts.notes`).
    let notes: Vec<&str> = item.rest.get("artifacts").and_then(|a| a.get("notes")).and_then(Value::as_array).map(|all| all.iter().filter_map(|n| n.get("file").and_then(Value::as_str)).collect()).unwrap_or_default();
    let print = format!("{thread}|{}|{notes:?}", fingerprint(answered));
    if let Some(files) = FILES.with(|last| last.borrow().as_ref().filter(|(p, _)| *p == print).map(|(_, f)| f.clone())) {
        return files;
    }
    let lines: Vec<Line> = answered.get("zero:item-history").and_then(|h| h.get("lines")).and_then(|l| serde_json::from_value(l.clone()).ok()).unwrap_or_default();
    let files = Rc::new(thread_files::files_made_here(&lines, &notes, s.now));
    FILES.with(|last| *last.borrow_mut() = Some((print, files.clone())));
    files
}

/// The button (`.ts-done.ts-files`): Done's frame around the files mark and how many there are.
fn files_button(count: usize, open: bool) -> El {
    let button = row().key(FILES_BUTTON).h(BUTTON).min_w(BUTTON).px(9.0).gap(6.0).justify_center().items_center().radius(6.0).border(1.0, rule()).border_layout();
    // Done's look under the pointer too (`.ts-done:hover`).
    crate::feel::strip_button(button, true)
        .child(icon(&icons::THREAD_FILES, 16.0).color(dim()).group_color(ink()))
        .child(text(count.to_string()).font(figures(12.5, 400.0)).color(dim()).group_color(ink()))
        .label(format!("Files in this thread ({count})"))
        .expanded(open)
        .on_press(|s: &mut State| {
            crate::feel::pressed(s);
            s.toggle_files()
        })
}

/// One file of the list: its kind, four letters at most in a small tile, its name, cut short
/// with dots, and the notes on it. A press closes the list and opens the file: big for what the
/// big view looks at, beside the conversation for a document, a table or a script
/// (`pane::open_listed`). `lit`: the pointer is on it.
fn file_row(at: usize, file: &ThreadFile, lit: bool) -> El {
    let kind = row()
        .w(34.0)
        .py(5.0)
        .radius(5.0)
        .fill(white(0.04))
        .justify_center()
        .child(text(thread_files::kind_tag(&file.name)).font(mono(9.5, 12.5).tracking(0.38)).text_case(TextCase::Uppercase).color(dim()));
    // Washed under the pointer (`--wash`), and the wash comes and goes at once: the web's
    // `:hover` has no transition, where Rune's own hover overlay fades out.
    let mut line = row()
        .min_h(36.0)
        .px(10.0)
        .py(6.0)
        .gap(10.0)
        .items_center()
        .radius(6.0)
        .fill(if lit { white(0.04) } else { rgba(0, 0, 0, 0.0) })
        .on_hover_change(move |s: &mut State, inside, _| {
            if inside {
                s.reading.file_lit = Some(at);
            } else if s.reading.file_lit == Some(at) {
                s.reading.file_lit = None;
            }
        })
        .child(kind)
        .child(row().grow(1.0).min_w(0.0).child(text(file.name.as_str()).font(sans(13.5, 400.0)).color(ink()).truncate()));
    if file.notes > 0 {
        line = line.child(text(thread_files::notes_words(file.notes)).font(sans(12.0, 400.0)).color(muted()));
    }
    let src = file.src.clone();
    line.cursor(rune::Cursor::Pointer).help(format!("Open {}", file.name)).on_press(move |s: &mut State| {
        s.close_files();
        crate::pane::open_listed(s, &src);
    })
}

/// The list the Files button opens, over the page (`.th-pop.ts-files-pop`): 360 points wide,
/// its right edge on the button's and its top 40 points under the button's, every file at
/// once up to 440 points tall or seven tenths of the window, past which they scroll. Under it
/// lies a layer over the rest of the window, and a press there closes it on the way down, as
/// the web window's `pointerdown` does; Escape closes it too (`keys.rs`). The layer takes the
/// press, which in the web window goes on to what it was aimed at.
pub fn files_list(s: &State) -> Option<El> {
    let View::Thread { id, product } = &s.view else {
        return None;
    };
    if !s.files_open() {
        return None;
    }
    let files = files_made_here(s, item(s, id, product)?);
    if files.is_empty() {
        return None;
    }
    // Where the button is in the window now. One not laid out yet has nothing hung from it.
    let [x, y, w, button_tall] = rune::memo::window_layout(FILES_BUTTON)?;
    const WIDE: f32 = 360.0;
    // Its edge and the six points inside it, twice, each file 36 points and two between them.
    let tall = 14.0 + files.len() as f32 * 38.0 - 2.0;
    let most = 440.0_f32.min(0.7 * s.size.1);
    // A scroller lays its content out at the content's own width, so the rows are told theirs.
    let rows = col().w(WIDE - 2.0).padding(6.0).gap(2.0).children(files.iter().enumerate().map(|(at, file)| file_row(at, file, s.reading.file_lit == Some(at))));
    // No rounded clip on the list itself: Rune's takes the element's own shadow with it, and
    // nothing inside reaches its corners.
    let list = col()
        .absolute(x + w - WIDE, y + 40.0)
        .w(WIDE)
        .radius(10.0)
        .fill(rgb(54, 52, 49))
        .border(1.0, rule())
        .border_layout()
        .box_shadow(shadow(0.0, 18.0, 50.0, 0.0, rgba(0, 0, 0, 0.5), false))
        .role("dialog")
        .label("Files in this thread");
    let list = if tall > most { list.h(most).child(col().grow(1.0).min_h(0.0).scroll(ScrollSpec { stick_bottom: false }).child(rows)) } else { list.child(rows) };
    // Everywhere but the button, which the web window's `pointerdown` leaves to the button's own
    // click: a press there closes the list on its way up, as the press that opened it did.
    let (ww, wh) = s.size;
    let (bottom, right) = (y + button_tall, x + w);
    let away = [[0.0, 0.0, ww, y], [0.0, bottom, ww, wh - bottom], [0.0, y, x, button_tall], [right, y, ww - right, button_tall]].map(|[ax, ay, aw, ah]| {
        el().absolute(ax, ay).size(aw.max(0.0), ah.max(0.0)).on_drag(|s: &mut State, d: &mut Drag, _| {
            if d.phase == Phase::Down {
                s.close_files();
            }
        })
    });
    Some(el().absolute(0.0, 0.0).size(ww, wh).children(away).child(list))
}

impl State {
    /// The open conversation, as `product/id`.
    fn thread_on_screen(&self) -> Option<String> {
        match &self.view {
            View::Thread { id, product } => Some(key(product, id)),
            _ => None,
        }
    }
    /// Whether the open conversation's list of files is open.
    pub fn files_open(&self) -> bool {
        self.reading.files.is_some() && self.reading.files == self.thread_on_screen()
    }
    /// The Files button: the list opens, or closes when it is open. It opens with no row
    /// washed, wherever the pointer last left one.
    pub fn toggle_files(&mut self) {
        self.reading.files = if self.files_open() { None } else { self.thread_on_screen() };
        self.reading.file_lit = None;
    }
    /// The list opens on the open conversation, for a picture taken with it open.
    pub fn open_files(&mut self) {
        self.reading.files = self.thread_on_screen();
        self.reading.file_lit = None;
    }
    /// Escape, a press outside it, a file pressed, the conversation left: the list closes.
    pub fn close_files(&mut self) {
        self.reading.files = None;
        self.reading.file_lit = None;
    }
    /// A press on the working line: it opens onto the steps the run took since it last said
    /// anything, and closes again.
    fn toggle_live_steps(&mut self, run: &str) {
        self.reading.live_open = if self.reading.live_open.as_deref() == Some(run) { None } else { Some(run.to_string()) };
    }
    /// Escape, while the open conversation's working line is open onto its steps: it folds them,
    /// and that is all the key does (`useEscapePutsAway`). True when there were steps to fold.
    pub fn fold_live_steps(&mut self) -> bool {
        let here = self.thread_on_screen().is_some_and(|thread| self.reading.live_open.as_deref().is_some_and(|run| run.starts_with(&format!("{thread}|"))));
        if here {
            self.reading.live_open = None;
            self.reading.live_rows_open.clear();
        }
        here
    }
    /// What a picture taken with the working line open says of it (`liveOpen`), and of which of
    /// its steps were open onto their commands, by their place (`liveRowsOpen`).
    pub(crate) fn picture_live(&mut self) {
        if self.fixture.remove("liveOpen").is_none() {
            return;
        }
        let View::Thread { id, product } = &self.view else { return };
        let Some(session) = running(self, id, product) else { return };
        let run = format!("{}|{}", key(product, id), session.started_at);
        let opened: Vec<usize> = self.fixture.remove("liveRowsOpen").and_then(|rows| serde_json::from_value(rows).ok()).unwrap_or_default();
        // The rows are named by their words, which the page draws from the conversation read.
        let item = self.snap.items.iter().find(|i| &i.id == id && &i.product == product).cloned();
        if let Some(item) = item {
            let talk = read(self, &item, false, true, None);
            let took: Vec<live::Took> = talk.trailing.iter().filter(|w| w.at >= session.started_at).map(|w| live::Took { at: w.at, verb: w.verb.clone(), subject: w.subject.clone(), full: w.full.clone() }).collect();
            let steps = live::steps(&took, &session.activity);
            for at in opened {
                if let Some(step) = steps.rows.get(at) {
                    self.reading.live_rows_open.insert(format!("{run}|{at}|{}", step.label));
                }
            }
        }
        self.reading.live_open = Some(run);
    }
    /// Leaving a conversation closes what was opened in it: its list of files, and its working
    /// line's steps, which the web window folds for another thread (`setExpandedAt(null)`).
    pub fn leave_the_reading(&mut self) {
        self.close_files();
        self.reading.live_open = None;
        self.reading.live_rows_open.clear();
    }
    /// Out of the conversation, as Escape and the strip's tab word take the person: back where
    /// they came from, and when that is the Inbox and the thread has moved tabs while it was
    /// read, onto the tab the strip named, with the keys on the thread's row
    /// (`closeThreadOntoItsTab` in `renderer/src/App.tsx`).
    pub fn close_onto_its_tab(&mut self) {
        let View::Thread { id, product } = self.view.clone() else {
            return;
        };
        let tab = holding_tab(self, &id, &product);
        let onto_the_inbox = matches!(self.history.last(), None | Some(View::Inbox));
        self.leave_thread();
        if let Some(tab) = tab.filter(|tab| onto_the_inbox && *tab != self.inbox_tab) {
            self.show_tab(tab);
            if let Some(at) = self.lists.under(tab).iter().position(|r| r.id == id && r.product == product) {
                self.picked = at;
            }
        }
    }
}

/// "Today 12:16pm": the day, then the clock (`when` in `Thread.tsx`).
fn when(at: f64, now: f64) -> String {
    const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let t = LocalTime::at(at as i64);
    // Whole days between the two dates, each counted on its own clock.
    let day_of = |ms: f64| {
        let l = LocalTime::at(ms as i64);
        ((ms / 1000.0) as i64 + i64::from(l.offset)).div_euclid(86_400)
    };
    let day = match day_of(now) - day_of(at) {
        0 => "Today".to_string(),
        1 => "Yesterday".to_string(),
        _ => format!("{}, {} {}", DAYS[(day_of(at) + 4).rem_euclid(7) as usize], MONTHS[(t.month as usize).clamp(1, 12) - 1], t.day),
    };
    let hour = if t.hour.is_multiple_of(12) { 12 } else { t.hour % 12 };
    format!("{day} {hour}:{:02}{}", t.minute, if t.hour < 12 { "am" } else { "pm" })
}

/// A message's head: who, when, and for a checkpoint the tag that says so. Its words can be
/// selected with the message's, as everything in the web window's conversation can
/// (`.focus-body { user-select: text }`).
fn said_by(m: &Message, now: f64) -> El {
    let line = kept(22.4);
    let who = if m.mine { text("You").font(TextStyle::new(14.0, 600.0).lh(line)).color(ink()) } else { text("The agent").font(TextStyle::new(14.0, 400.0).lh(line)).color(dim()) };
    // The three sit on one baseline: the smaller the type, the lower its box starts. What the
    // person just sent says where it is in the time's place, in the dim ink, with Send now after
    // it while it waits on the step its agent is in (`.msg-sending`, `.msg-now`).
    let when = match m.sending {
        Some(sending) => {
            let (said, now_button) = crate::answer::head_words(sending);
            let words = readable(text(said).font(TextStyle::new(12.5, 400.0).lh(20.0)).color(dim())).mt(1.5);
            if now_button { row().gap(10.0).child(words).child(crate::answer::send_now().mt(1.5)) } else { words }
        }
        None => readable(text(when(m.at, now)).font(TextStyle::new(12.5, 400.0).lh(20.0).features("'tnum' 1")).color(muted())).mt(1.5),
    };
    let mut head = row().h(line).items_start().gap(10.0).child(readable(who)).child(when);
    if !m.mine && m.field == Field::Note {
        let tag = mono(11.0, 14.0);
        head = head.child(
            row().mt(3.0)
                .gap(6.0)
                .px(7.0)
                .py(1.0)
                .radius(6.0)
                .border(1.0, edge())
                .border_layout()
                .child(readable(text("Checkpoint").font(tag.tracking(0.22)).color(dim())))
                .child(row().gap(6.0).child(readable(text("·").font(tag).color(muted()))).child(readable(text("progress so far").font(tag).color(muted())))),
        );
    }
    head
}

/// What a person typed is drawn as typed, line breaks and all, never as markdown, and never
/// wider than 760 points however wide the column grows.
fn typed(words: &str, m: &Measure) -> El {
    let line = kept(1.55 * m.size);
    let font = TextStyle::new(m.size, 500.0).lh(line);
    col().max_w(760.0).children(words.split('\n').map(|words| if words.is_empty() { el().h(line) } else { readable(text(words).font(font).color(ink()).wrap_anywhere()) }))
}

/// A person's message: their words as typed, and each picture or file the app attached to it
/// drawn as what it is, as an agent's words would draw it (`rules/typed.rs`), ten points from
/// what is around it (`.msg-body.typed .typed-embed`).
fn typed_with_files(words: &str, m: &Measure, files: &Files) -> El {
    let parts = crate::rules::typed::typed_parts(words);
    if let [crate::rules::typed::Part::Text(words)] = parts.as_slice() {
        return typed(words, m);
    }
    col().children(parts.iter().enumerate().map(|(at, part)| {
        let drawn = match part {
            crate::rules::typed::Part::Text(words) => typed(words, m),
            crate::rules::typed::Part::Embed(line) => markdown::stack(markdown::blocks_typed(line, m.size, m.column, files), true).0,
        };
        drawn.mt(if at == 0 { 0.0 } else { 10.0 })
    }))
}

/// One message, and how much of what it holds reaches under it.
fn message(msg: &Message, turn: (bool, bool), now: f64, m: &Measure, files: &Files) -> El {
    let (body, leak) = if msg.mine {
        // Still on its way, its words a little fainter (`.msg.sending .msg-body`).
        let body = typed_with_files(&msg.text, m, files);
        (if msg.sending.is_some() { body.opacity(0.82) } else { body }, 0.0)
    } else {
        let (body, _, bottom) = markdown::stack(markdown::blocks(&msg.text, m.size, m.column, files), true);
        (body, bottom)
    };
    // A run still talking after its own line above wears no second name and time; across a
    // long conversation's cut it gets its name back, since what it continued is not on screen.
    let mut e = if msg.same && !msg.resumed { col().child(body) } else { col().child(said_by(msg, now)).child(body.mt(3.0)) };
    // The person's turn is framed: a rule over it where the agent spoke before, and one under
    // it where the agent speaks or works next.
    if turn.0 {
        e = e.mt(16.0).pt(22.0);
    }
    if turn.1 {
        e = e.pb(22.0);
    }
    if turn.0 || turn.1 {
        e = e.border_sides([if turn.0 { 1.0 } else { 0.0 }, 0.0, if turn.1 { 1.0 } else { 0.0 }, 0.0], rule()).border_layout();
    }
    col().child(e).pb(leak + if turn.1 { 16.0 } else { 0.0 })
}

/// Something that happened, on one quiet line.
fn step(s: &Step) -> El {
    let font = TextStyle::new(13.0, 400.0).lh(19.5);
    let mut line = row().gap(8.0).child(text(s.verb.as_str()).font(font).color(dim()));
    if !s.words.is_empty() {
        line = line.child(row().min_w(0.0).child(text(s.words.as_str()).font(font).color(muted()).truncate()));
    }
    line
}

/// `--state-stopped`: the one colour on a run's lines, and only where a tool said no.
fn stopped() -> rune::Color {
    rgb(224, 138, 108)
}

/// One thing a run ran, inside its opened fold (`WorkLine`, shut): its mark when there is
/// something behind it, its verb, what it acted on in the code face, and how many lines came
/// back. A tool that said no wears the stopped colour on its verb.
fn work_line(w: &Work) -> El {
    let came_back = w.output.split('\n').count() - usize::from(w.output.ends_with('\n') || w.output.is_empty());
    let opens = came_back > 0 || !w.full.is_empty();
    let small = TextStyle::new(12.0, 400.0).lh(18.0);
    let mut line = row().min_w(0.0).items_start().gap(8.0);
    if opens {
        line = line.child(icon(&icons::WORK_MARK, 8.0).color(muted()).mt(5.75));
    }
    line = line.child(text(w.verb.as_str()).font(TextStyle::new(13.0, 400.0).lh(19.5)).color(if w.failed { stopped() } else { dim() }));
    if !w.subject.is_empty() {
        line = line.child(row().min_w(0.0).mt(1.0).child(text(w.subject.as_str()).font(mono(12.0, 18.0)).color(muted()).truncate()));
    }
    if opens && w.lines > 1 {
        line = line.child(text(format!("· {} lines", w.lines)).font(small).color(muted()).mt(1.0));
    }
    if w.more > 0 {
        return col().child(line).child(text(work::run_overflow(w.more)).font(small).color(muted()).pt(5.0));
    }
    line
}

/// Consecutive things a run ran, folded to one line (`RunLine`): the kinds of thing it did,
/// how many of them failed, and, open, each one on its own line under it. A press opens and
/// shuts it at once. Under the pointer its words are the text's own colour and its mark, which
/// folded shows only then, a step brighter (`.did-head:hover .did-verb`, `.did-run-head:hover >
/// .did-run-mark`).
fn run_line(items: &[&Work], open: bool, key: String, m: &Measure) -> El {
    let summary = work::activity_summary(items.iter().map(|w| (w.verb.as_str(), w.subject.as_str())));
    let failed = work::run_failures(items.iter().filter(|w| w.failed).count());
    let mut head = row().max_w(m.column).items_start().gap(8.0).child(row().min_w(0.0).child(text(summary).font(TextStyle::new(13.0, 400.0).lh(19.5)).color(dim()).group_color(ink()).truncate()));
    if !failed.is_empty() {
        head = head.child(text(failed).font(TextStyle::new(12.0, 400.0).lh(18.0)).color(stopped()).mt(1.0));
    }
    // Its mark after the words (`order: 2`): open it shows turned down; folded it shows only
    // under the pointer (`opacity: 0`), pointing at the words.
    head = if open {
        head.child(icon(&icons::WORK_MARK_OPEN, 8.0).color(muted()).group_color(dim()).mt(5.75))
    } else {
        head.child(icon(&icons::WORK_MARK, 8.0).color(dim()).mt(5.75).group_opacity(0.0, 1.0))
    };
    let head = row().key(key.clone()).hover_group().cursor(rune::Cursor::Pointer).child(head).on_press(move |s: &mut State| {
        if !s.reading.runs_open.remove(&key) {
            s.reading.runs_open.insert(key.clone());
        }
    });
    let run = col().child(head);
    if !open {
        return run;
    }
    // The lines inside step in by the mark and its gap, so the nested marks land on the fold's words.
    run.child(col().pt(5.0).pb(2.0).pl(16.0).gap(5.0).children(items.iter().map(|w| work_line(w))))
}

/// What a block of the conversation holds, which decides the room above it (`.thread-block`
/// and the rules on its neighbours in `styles.css`).
#[derive(Clone, Copy, PartialEq)]
enum Holds {
    Msg,
    /// A message of a run still talking, pulled up 14 points inside its block (`.msg.same`).
    Same,
    /// A line of the app's own about the thread (`is-work`).
    Step,
    /// Something the person did (`is-act`).
    Act,
    /// What a run ran, folded (`is-run`).
    Run,
}

/// The room between two blocks: the column's 22 points, less what the rules on their
/// neighbours pull back. A line of what happened sits 8 under a message and a run of them 5
/// apart, as an opened fold's own lines are.
fn room(above: Holds, here: Holds) -> f32 {
    use Holds::*;
    match (above, here) {
        (_, Msg) => 22.0,
        (_, Same) => 8.0,
        (Msg | Same, Run | Step | Act) => 8.0,
        (Run | Step, Run) | (Run, Step) => 5.0,
        (Act, Run) | (Run, Act) => 14.0,
        (Step | Act, Step | Act) => 5.0,
    }
}

/// One name per thing said or run, from what it is and when it happened, numbered only among
/// exact twins (`lineIds` in `renderer/src/thread-keys.ts`). A fold is named after the block
/// above it, which stays put while the agent adds to the run under it (`blockKeys`).
fn line_ids(said: &[Said]) -> Vec<String> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    said.iter()
        .map(|e| {
            let base = match e {
                Said::Message(m) => format!("m{}", m.at),
                Said::Step(s) => format!("w{}:{}:{}", s.at, s.verb, s.words),
                Said::Work(w) => format!("w{}:{}:{}", w.at, w.verb, w.subject),
            };
            let twins = seen.entry(base.clone()).or_insert(0);
            *twins += 1;
            if *twins > 1 { format!("{base}#{}", *twins - 1) } else { base }
        })
        .collect()
}

/// The line that stands for a long conversation's missing middle, which asks for it whole.
fn gap_line(omitted: usize, thread: String) -> El {
    let line = el().h(kept(25.6)).items_center().child(text(work::conversation_gap(omitted)).font(TextStyle::new(13.0, 400.0).lh(kept(25.6))).color(muted()));
    line.mt(18.0).cursor(rune::Cursor::Pointer).on_press(move |s: &mut State| {
        s.reading.whole.insert(thread.clone());
    })
}

/// The conversation: each thing said or done under the last, 22 points apart, with what
/// happened drawn closer to what it follows, and each stretch of what a run ran folded to one
/// line (`.thread`, `.thread-block` and `groupWork`).
fn conversation(s: &State, thread: &str, page: &Page, now: f64, m: &Measure, files: &Files) -> El {
    let said = page.said;
    let cut = page.gap.map(|(at, _)| at);
    let turns = talk::turns(said, cut);
    let ids = line_ids(said);
    let pictured = pictured_runs(s);
    let mut out = col();
    let (mut above, mut gap_above): (Option<Holds>, bool) = (None, false);
    let mut name_above = "start".to_string();
    let (mut i, mut runs) = (0, 0);
    while i < said.len() {
        let (block, holds) = match &said[i] {
            Said::Work(_) => {
                let from = i;
                while i < said.len() && matches!(said[i], Said::Work(_)) {
                    i += 1;
                }
                let items: Vec<&Work> = said[from..i].iter().filter_map(|e| if let Said::Work(w) = e { Some(w) } else { None }).collect();
                let key = format!("{thread}|run>{name_above}");
                let open = s.reading.runs_open.contains(&key) != pictured.contains(&runs);
                runs += 1;
                (run_line(&items, open, key, m), Holds::Run)
            }
            Said::Message(msg) => {
                name_above = ids[i].clone();
                i += 1;
                (message(msg, turns[i - 1], now, m, files), if msg.same && !msg.resumed { Holds::Same } else { Holds::Msg })
            }
            Said::Step(st) => {
                name_above = ids[i].clone();
                i += 1;
                (step(st), if st.mine { Holds::Act } else { Holds::Step })
            }
        };
        // Under the line for the missing middle, nothing is pulled closer.
        let gap = match above {
            None => 0.0,
            Some(_) if gap_above => 22.0,
            Some(a) => room(a, holds),
        };
        gap_above = cut == Some(i - 1);
        let block = match page.gap.filter(|_| gap_above) {
            Some((_, omitted)) => col().child(block).child(gap_line(omitted, thread.to_string())),
            None => block,
        };
        above = Some(holds);
        out = out.child(block.mt(gap));
    }
    // The page's own foot mark is one more thing in the stack, so the gap comes after the last too.
    out.pb(22.0)
}

/// Frames a second the line's loops ask for while nothing else moves: the web window's run at
/// the display's rate, and at 60 the lattice's quickest change is nine frames long and the band
/// moves a point or two a frame.
const LOOP_FPS: f32 = 60.0;

/// A loop's place in its period at `from` seconds on the window's clock, for a loop that starts
/// then: a `Cycle` stands at `t / period − phase`, so one that starts at `from` adds this.
fn started(from: f64, period: f32) -> f32 {
    (from / f64::from(period)).rem_euclid(1.0) as f32
}

/// The working mark (`LiveMark`): nine round cells in a square of 14 points, the eight round the
/// edge lighting in turn as an orbit (`live-lattice-orbit`), the middle a hole. Each cell is a
/// loop of its own on Rune's clock (`rules::motion`), which the window draws without building the
/// page again, and which holds the web's still opacity under Reduce motion. The orbit starts
/// where the web's does as the line mounts, at `from` on the window's clock. The cells are where
/// the grid puts them, between device pixels, as the browser draws them.
fn lattice(moving: bool, from: f64) -> El {
    use crate::rules::motion::{LATTICE_HOLE, LATTICE_ORBIT, LATTICE_PERIOD, LATTICE_STILL, lattice_phase, lattice_stops};
    let side = 14.0 / 3.0 - 1.0;
    let start = started(from, LATTICE_PERIOD);
    let cell = move |at: usize| {
        let dot = el().size(side, side).radius(side / 2.0).fill(ink()).a11y_hidden();
        match LATTICE_ORBIT[at] {
            Some(place) if moving => dot.cycle(rune::Cycle { period: LATTICE_PERIOD, phase: lattice_phase(place) + start, stops: lattice_stops(), still: LATTICE_STILL, fps: LOOP_FPS }),
            Some(_) => dot.opacity(LATTICE_STILL),
            None => dot.opacity(LATTICE_HOLE),
        }
    };
    // A still mark is other elements than a turning one, so the loops leave with the cells they
    // ran on rather than staying on them, cooling.
    col().key(if moving { "live-mark" } else { "live-mark-still" }).size(14.0, 14.0).gap(1.5).pixel_snap(false).children((0..3).map(move |r| row().gap(1.5).children((0..3).map(move |c| cell(r * 3 + c)))))
}

/// The key of the step's word, whose place says what ground its veil is drawn in.
const LIVE_WORD: &str = "live-word";

/// Words that shimmer (`.live-word.is-shimmering`): in full ink, with the soft band of the web
/// window's mask travelling through them from left to right every 2.6 seconds, drawn as a veil
/// of the ground over them in strips, each a loop (`rules::motion::shimmer_veil`). `wide` and
/// `tall` are how large they stand, and `key` names them, for where they are. Under Reduce
/// motion (`still`) the web window takes the band away and draws the words in dim ink, and so
/// does this one; off screen (`moving` false) they have no band. The band starts off their left
/// end at `from`, as the web's does when they mount.
#[allow(clippy::too_many_arguments)]
fn shimmering(s: &State, key: &'static str, words: &str, font: TextStyle, (wide, tall): (f32, f32), still: bool, moving: bool, from: f64) -> El {
    use crate::rules::motion::{LEAST, SHIMMER_PERIOD, ground_under_the_card, shimmer_veil};
    // What they say is said aloud each time it changes (`live-announcement`, `role="status"`).
    let word = text(words).font(font).color(if still { dim() } else { ink() }).truncate().live_region(rune::LiveRegion::Polite);
    let boxed = el().key(key).shrink().min_w(0.0).child(word);
    if still || !moving {
        return boxed;
    }
    // The ground under them: where they were last laid out, else where a working line stands.
    let at = rune::memo::window_layout(key).map_or((s.size.0 * 0.3, s.size.1 * 0.8), |[x, y, w, h]| (x + w / 2.0, y + h / 2.0));
    let [r, g, b] = ground_under_the_card(at, s.size);
    let ground = rgb(r.round() as u8, g.round() as u8, b.round() as u8);
    let veil = shimmer_veil(wide, tall);
    let start = started(from, SHIMMER_PERIOD);
    let strips = veil.strips.into_iter().map(move |strip| {
        el().absolute(strip.x, 0.0).size(strip.w, tall).fill(ground).a11y_hidden().cycle(rune::Cycle { period: SHIMMER_PERIOD, phase: strip.phase + start, stops: veil.stops, still: LEAST, fps: LOOP_FPS })
    });
    boxed.clip_rounded(0.0).children(strips)
}

/// Has `line` say when it is on screen and when it leaves (`on_visible`), as `seen`, whose loops
/// run only while it is; and the first time, when, as the start of `run`'s loops.
fn watched(line: El, seen: String, run: String) -> El {
    line.on_visible(move |s: &mut State, on, tx| {
        if on {
            s.reading.live_seen = Some(seen.clone());
            if s.reading.live_from.as_ref().is_none_or(|(seen_run, _)| *seen_run != run) {
                s.reading.live_from = Some((run.clone(), tx.clock()));
            }
        } else if s.reading.live_seen.as_deref() == Some(seen.as_str()) {
            s.reading.live_seen = None;
        }
    })
}

/// The line a chat just brought in shows until the run that writes it up has (`catchUpDue`,
/// `.one-catching-up`): one sentence at 15 points, shimmering as the working line's word does.
fn getting_ready(s: &State, thread: &str, m: &Measure, still: bool) -> El {
    const WORDS: &str = "Getting this chat ready. Its summary and options will appear here.";
    let seen = format!("{thread}#ready");
    let moving = !still && s.reading.live_seen.as_deref() == Some(seen.as_str());
    let from = s.reading.live_from.as_ref().filter(|(run, _)| *run == seen).map_or(0.0, |(_, at)| *at);
    let font = TextStyle::new(15.0, 500.0).lh(24.0);
    let wide = rune::text::measure(WORDS, &[], font, None).0.min(m.column);
    watched(row().w(m.column).child(shimmering(s, "getting-ready", WORDS, font, (wide, 24.0), still, moving, from)), seen.clone(), seen)
}

/// How wide the step's word stands on the line: as wide as its words, unless the column leaves
/// it less after the mark, the counts and times beside it and the chevron, when it is cut short.
fn word_room(line: &live::Working, word: TextStyle, beside: TextStyle, column: f32) -> f32 {
    let natural = rune::text::measure(&line.word, &[], word, None).0;
    let spans: f32 = [line.more.as_deref(), Some(line.span.as_str()), line.place.as_deref()].into_iter().flatten().map(|s| 6.0 + rune::text::measure(s, &[], beside, None).0).sum();
    natural.min((column - 14.0 - 7.0 - spans - 6.0 - 10.0).max(0.0))
}

/// The line at the foot of a conversation an agent is on (`Live.tsx`, working): the mark, the
/// step it is on in its own words, how many more run beside it, how long it has been at it,
/// and where it runs when that is news. The mark turns and the word shimmers, as the web
/// window's do, and both stand still under Reduce motion (`still`), as the web's then do. They
/// move only while the line is on screen: the window reports it (`on_visible`), and until it
/// has, for the frame a conversation opens on, the mark stands as it does under Reduce motion.
fn live_line(s: &State, thread: &str, run: String, line: &live::Working, steps: &live::Steps, m: &Measure, still: bool) -> El {
    let moving = !still && s.reading.live_seen.as_deref() == Some(thread);
    let from = s.reading.live_from.as_ref().filter(|(seen, _)| *seen == run).map_or(0.0, |(_, at)| *at);
    let open = s.reading.live_open.as_deref() == Some(run.as_str());
    let font = TextStyle::new(13.5, 400.0).lh(20.0);
    let word_font = TextStyle::new(13.5, 500.0).lh(20.0);
    let wide = word_room(line, word_font, font.features("'tnum' 1"), m.column);
    // Rune's flex never shrinks a child below what it holds unless told to (`shrink`), and
    // the step is the one part that gives way.
    let mut words = row().items_center().gap(6.0).shrink().child(shimmering(s, LIVE_WORD, &line.word, word_font, (wide, 20.0), still, moving, from));
    for span in [line.more.as_deref(), Some(line.span.as_str()), line.place.as_deref()].into_iter().flatten() {
        words = words.child(text(span).font(font.features("'tnum' 1")).color(muted()));
    }
    // The words are a press that opens the line onto its steps (`.live-disclosure`), with a
    // chevron after them that shows only under the pointer and turns down while it is open.
    let chevron = icon(if open { &icons::LIVE_CHEVRON_OPEN } else { &icons::LIVE_CHEVRON }, 10.0).color(dim());
    let chevron = if open { chevron } else { chevron.group_opacity(0.0, 1.0) };
    let pressed = run.clone();
    let words = words.child(chevron).hover_group().cursor(rune::Cursor::Pointer).role("button").expanded(open).on_press(move |s: &mut State| s.toggle_live_steps(&pressed));
    // No wider than the column: a step is often a whole command line, and it is cut short with
    // dots, as `.live-disclosure .live-word` cuts it, rather than run past the page. The stack
    // it stands in takes its width from what is in it, so the column is said here.
    let brief = watched(row().max_w(m.column).items_center().gap(7.0).child(lattice(moving, from)).child(words), thread.to_string(), run.clone());
    if !open {
        return brief;
    }
    col().w(m.column).child(brief).child(live_steps(s, &run, steps))
}

/// What the working line opens onto (`.live-details`), eight points under it in dim ink: a
/// line counting the finished steps it does not list, then a row for each step it does and
/// for each call running now, its mark and its words, which a press opens onto the command
/// behind it, set in a block as code is (`ActivityRow`, `details.live-command`).
fn live_steps(s: &State, run: &str, steps: &live::Steps) -> El {
    let words = TextStyle::new(13.0, 400.0).lh(20.8);
    let mut out = col().pt(8.0);
    if steps.earlier > 0 {
        out = out.child(row().h(28.0).pl(21.0).items_center().child(text(live::earlier_steps(steps.earlier)).font(words).color(muted())));
    }
    if steps.rows.is_empty() {
        return out.child(row().h(28.0).items_center().child(text("No commands yet.").font(words).color(dim())));
    }
    for (at, step) in steps.rows.iter().enumerate() {
        let key = format!("{run}|{at}|{}", step.label);
        let open = s.reading.live_rows_open.contains(&key);
        let head = row()
            .h(28.0)
            .items_center()
            .gap(7.0)
            .cursor(rune::Cursor::Pointer)
            .role("button")
            .expanded(open)
            .child(icon(&icons::LIVE_COMMAND, 14.0).color(dim()))
            .child(row().min_w(0.0).shrink().child(text(step.label.as_str()).font(words).color(dim()).truncate()))
            .on_press(move |s: &mut State| {
                if !s.reading.live_rows_open.remove(&key) {
                    s.reading.live_rows_open.insert(key.clone());
                }
            });
        let mut item = col().child(head);
        if open {
            let command = readable(text(step.detail.as_str()).font(mono(12.0, 19.2)).color(dim()).wrap_anywhere());
            item = item.child(el().ml(21.0).mt(4.0).mb(10.0).px(14.0).py(12.0).radius(6.0).fill(rgba(40, 38, 37, 0.9)).border(1.0, edge()).border_layout().child(command));
        }
        out = out.child(item);
    }
    out
}

/// The line under the title (`.one-cols`): the summary a thread carries, or while an agent
/// works what was asked, a rule down its left side and three lines at most.
fn context_line(words: &str, m: &Measure) -> El {
    let size = (0.76 * m.size).max(13.5);
    row().w((size * 50.37).min(m.column)).border_sides([0.0, 0.0, 0.0, 2.0], rule()).border_layout().pl(14.0).child(
        text(words).font(TextStyle::new(size, 400.0).lh(kept(1.55 * size))).color(muted()).wrap_anywhere().max_lines(3),
    )
}

/// The folds a fixture's picture was taken with open, by their place among the page's folds.
fn pictured_runs(s: &State) -> Vec<usize> {
    s.fixture.get("runsOpen").and_then(Value::as_array).map(|list| list.iter().filter_map(Value::as_u64).map(|n| n as usize).collect()).unwrap_or_default()
}

/// The run on a thread now, off the snapshot's running list, when there is one.
fn running(s: &State, id: &str, product: &str) -> Option<live::Session> {
    let run = s.snap.supervisor.running.iter().find(|r| r.item_id == id && r.product == product)?;
    let mut facts: Map<String, Value> = run.rest.clone();
    facts.insert("itemId".to_string(), json!(id));
    facts.insert("product".to_string(), json!(product));
    serde_json::from_value(Value::Object(facts)).ok()
}

/// The line under the title that folds and unfolds the rest: small, quiet and underlined, the
/// text's own colour under the pointer (`.one-earlier:hover`), its line staying as it is.
fn toggle(words: &str, thread: String, m: &Measure) -> El {
    let size = (0.7 * m.size).max(12.5);
    let line = kept(1.6 * size);
    // The underline sits 4 points under the baseline, which Geist puts 1.005 of the size down,
    // on the nearest device pixel of a 2x screen.
    let under = (((line - 1.3 * size) / 2.0 + 1.005 * size + 4.0) * 2.0).round() / 2.0;
    row().child(
        el().key("thread-earlier")
            .hover_group()
            .cursor(rune::Cursor::Pointer)
            .child(text(words).font(TextStyle::new(size, 400.0).lh(line)).color(muted()).group_color(ink()))
            .child(el().pin(Some(0.0), Some(under), Some(0.0), None).h(1.0).fill(rule()))
            .on_press(move |s: &mut State| {
                if !s.reading.unfolded.remove(&thread) {
                    s.reading.unfolded.insert(thread.clone());
                }
                // The middle is asked for whole from the first unfolding on (`setWhole(true)`).
                s.reading.whole.insert(thread.clone());
            }),
    )
}

/// The agent's last word, whole, at the foot of the conversation.
fn outcome(answer: &Message, plain: bool, m: &Measure, files: &Files) -> Vec<Block> {
    let mut blocks = markdown::blocks(&answer.text, m.size, m.column, files);
    if !plain {
        // A thread still in flight says which of the two this is.
        let words = if answer.field == Field::Result { "Result" } else { "Latest checkpoint" };
        blocks.insert(0, markdown::block(0.0, 7.0, readable(text(words).font(TextStyle::new(12.5, 400.0).lh(20.0)).color(muted()))));
    }
    blocks
}

/// The newest answer alone, as a page that leads with it: a first line that is bold and
/// nothing else is its headline, unless it only repeats the thread's name.
fn answer_alone(answer: &Message, title: &str, m: &Measure, files: &Files) -> Vec<Block> {
    let same = |a: &str, b: &str| a.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase() == b.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    let (headline, rest) = talk::headline(&answer.text);
    let mut blocks = markdown::blocks(rest, m.size, m.column, files);
    if let Some(words) = headline.filter(|h| !same(h, title)) {
        let (size, under) = m.headline;
        blocks.insert(0, markdown::block(0.0, under, readable(text(words).font(TextStyle::new(size, 600.0).lh(kept(1.45 * size))).color(ink()).wrap())));
    }
    blocks
}

/// What the engine answered about this thread, its lines and its runs' logs: asked through the
/// door when the window is live, carried by a fixture for a picture. With nothing to read yet,
/// the row itself still says what was asked and what came back.
fn ledger(s: &State, item: &Item) -> (Vec<Line>, Vec<TraceSession>) {
    let thread = key(&item.product, &item.id);
    let answered = s.threads.get(&thread).cloned().or_else(|| s.fixture.get("threads").and_then(|t| t.get(&thread)).cloned());
    if let Some(answers) = answered.and_then(|v| serde_json::from_value::<Answers>(v).ok()) {
        let sessions = answers.sessions();
        return (answers.history.lines, sessions);
    }
    let wrote = |field: &str| item.rest.get("wrote").and_then(|w| w.get(field)).cloned().unwrap_or(Value::Null);
    let said = |field: &str, words: Option<&str>, fallback: Option<f64>| {
        let words = words.filter(|w| !w.trim().is_empty())?;
        let by = wrote(field);
        let line = json!({
            "ts": by.get("ts").and_then(Value::as_f64).or(fallback).unwrap_or(0.0),
            "source": by.get("source").and_then(Value::as_str).unwrap_or(if field == "body" || field == "answer" { "founder" } else { "agent" }),
            "patch": { field: words },
        });
        serde_json::from_value::<Line>(line).ok()
    };
    let rest = |field: &str| item.rest.get(field).and_then(Value::as_str);
    let mut lines: Vec<Line> = [
        said("body", item.body.as_deref(), item.created_at),
        said("note", rest("note"), item.updated_at),
        said("answer", rest("answer"), item.updated_at),
        said("result", item.result.as_deref(), item.updated_at),
    ]
    .into_iter()
    .flatten()
    .collect();
    lines.sort_by(|a, b| a.ts.total_cmp(&b.ts));
    (lines, Vec::new())
}

thread_local! {
    /// The conversation last read and what it was read from. Its runs' logs can run to megabytes
    /// and the page is drawn every frame, so it is read again only when what it is read from changes.
    static READ: std::cell::RefCell<Option<(String, std::rc::Rc<Talk>)>> = const { std::cell::RefCell::new(None) };
}

/// What an answer about a thread is, in a few numbers: how many lines its ledger has and the
/// newest, and each log's start and length. Logs only grow, so the same numbers are the same logs.
fn fingerprint(answers: &Value) -> String {
    let lines = answers.get("zero:item-history").and_then(|h| h.get("lines")).and_then(Value::as_array);
    let logs = |channel: &str| {
        let sessions = answers.get(channel).and_then(|t| t.get("sessions")).and_then(Value::as_array);
        sessions.map(|all| all.iter().map(|s| format!("{}:{}", s.get("startedAt").unwrap_or(&Value::Null), s.get("text").and_then(Value::as_str).map_or(0, str::len))).collect::<Vec<_>>().join(","))
    };
    let newest = lines.and_then(|l| l.last()).and_then(|l| l.get("ts")).map(Value::to_string);
    format!("{}|{newest:?}|{:?}|{:?}", lines.map_or(0, Vec::len), logs("zero:session-trace"), logs("zero:session-trace/whole"))
}

/// The conversation, read from the thread's answers once per change of them: while an agent
/// works, without the work its live line stands for.
fn read(s: &State, item: &Item, whole: bool, working: bool, saying: Option<&Saying>) -> std::rc::Rc<Talk> {
    let thread = key(&item.product, &item.id);
    let answered = s.threads.get(&thread).or_else(|| s.fixture.get("threads").and_then(|t| t.get(&thread)));
    let from = answered.map_or_else(|| format!("the row at {:?}", item.updated_at), fingerprint);
    // What the person has just sent and the Zs pressed here are placed among the lines too
    // (`answer.rs`).
    let sent = s.answering.sent_on(&thread, s.waits_on_the_step(item));
    let undone = s.answering.undone_on(&thread);
    let print = format!("{thread}|{from}|{whole}|{working}|{saying:?}|{sent:?}|{undone:?}");
    if let Some(talk) = READ.with(|last| last.borrow().as_ref().filter(|(p, _)| *p == print).map(|(_, t)| t.clone())) {
        return talk;
    }
    let (lines, sessions) = ledger(s, item);
    let mut talk = Talk::read_with(&lines, &sessions, saying, whole, &sent, &undone);
    if working {
        talk.without_trailing_work();
    }
    // While the offer stands, the message it came from loses its list, which the strip over
    // the reply box draws (`cleanMessage` in `Focus`). A message only holding the same words as
    // an older offer keeps them: on a finished thread the list is the record.
    if let Some(offered) = format::offer_is_live(item).then(|| format::offered(item)).flatten() {
        let strip = |m: &mut Message| {
            if !m.mine && js_trim(&m.text) == offered {
                m.text = format::strip_options_section(&m.text);
            }
        };
        talk.said.iter_mut().for_each(|e| {
            if let Said::Message(m) = e {
                strip(m);
            }
        });
        if let Some(m) = talk.outcome.as_mut() {
            strip(m);
        }
    }
    let talk = std::rc::Rc::new(talk);
    READ.with(|last| *last.borrow_mut() = Some((print, talk.clone())));
    talk
}

fn js_trim(s: &str) -> &str {
    crate::rules::js::trim(s)
}

/// Whether a fixture's picture was taken with this thread's earlier messages unfolded.
fn pictured_unfolded(s: &State, thread: &str) -> bool {
    s.fixture.get("unfolded").and_then(Value::as_array).is_some_and(|list| list.iter().any(|k| k.as_str() == Some(thread)))
}

/// The reply box folded to its one line, as a conversation first shows it. R, or a press,
/// opens it (`composer`). It says what is in it: the words written in it and not sent, on one
/// line and dimmer, and the files in it counted, or only the invitation when it holds nothing
/// (`foldedReply`, `.dock-pill-kept`, `.dock-pill-files`). At its right end the model that
/// picks up the next reply, whose press opens the box with the model menu already open
/// (`.ts-model`, `openModel`).
fn reply_box(s: &State, item: &Item, m: &Measure) -> El {
    let size = (0.78 * m.size).max(14.5);
    let files: Vec<(&str, bool)> = s.reply_files.open.iter().map(|a| (a.file.name.as_str(), a.file.image)).collect();
    let folded = crate::rules::folded::folded_reply(&s.reply.text, &files);
    // With the pointing hand and the text's own colour under the pointer (`.ts-model`), and its
    // press is not a press on the box.
    let model = el()
        .key(reply::WORD)
        .hover_group()
        .help("Which model this conversation runs on")
        .child(label(&s.folded_model_word(item), 13.0, 400.0, dim()).group_color(ink()))
        // `underline dotted`, three points under the baseline: a dot every three points, as
        // far as the words reach.
        .child(row().pin(Some(-0.25), Some(16.0), Some(0.0), None).h(1.5).gap(1.5).clip_rounded(0.0).children((0..60).map(|_| el().size(1.5, 1.5).fill(rule()))))
        .cursor(rune::Cursor::Pointer)
        .on_background_press_tx(|s: &mut State, tx| s.open_reply_on_the_model(tx));
    let key = row().px(5.5).py(1.5).radius(6.0).fill(white(0.04)).border(1.0, rule()).border_layout().child(text("R").font(mono(10.5, 13.5)).color(dim()));
    let said = text(folded.text.as_str()).font(sans(size, 400.0)).color(if folded.draft { dim() } else { muted() }).truncate();
    let counted = (!folded.files.is_empty()).then(|| el().pr(12.0).child(label(&folded.files, 12.0, 400.0, muted())));
    // With the agent's choices folded away, the sign that they are there (`ChoicesShow`).
    let choices = live_options(item);
    let sign = (!choices.is_empty() && !s.answer_waits(item) && crate::chips::choices_folded(s, item)).then(|| crate::chips::choices_sign(choices.len()));
    row().w(m.column)
        .radius(10.0)
        .fill(white(0.04))
        .border(1.0, edge())
        .border_layout()
        .box_shadow(shadow(0.0, 10.0, 32.0, 0.0, rgba(0, 0, 0, 0.38), false))
        .items_center()
        .child(row().grow(1.0).min_w(0.0).px(16.0).py(11.0).child(said))
        .children(counted)
        .child(model.mr(12.0))
        .child(key.mr(14.0))
        .children(sign)
}

/// The mark of the agent a reply goes to, before the model's name on the reply box's chip
/// (`EngineLogo`, 14 points): Claude Code's in its own orange, Codex's and Antigravity's in the
/// chip's faint ink (`.tc-chip svg`). An agent with no mark of its own wears Claude Code's, as
/// in the web window.
fn engine_logo(engine: &str) -> El {
    match engine {
        "codex" => icon(&icons::ENGINE_CODEX, 14.0).color(muted()),
        "antigravity" => icon(&icons::ENGINE_ANTIGRAVITY, 14.0).color(muted()),
        _ => icon(&icons::ENGINE_CLAUDE, 14.0).color(rgb(0xd9, 0x77, 0x57)),
    }
}

/// The reply box opened (`.dock-card`): the words, then who takes them and Send. The words
/// are the person's to type. Of the three chips the model's opens its menu (`reply.rs`); the
/// level and what the message may do are drawn, not chosen here.
fn composer(s: &State, item: &Item, m: &Measure) -> El {
    let field = text_input(&s.reply, TextStyle::new(15.5, 400.0).lh(kept(24.025)))
        .key("reply")
        .color(ink())
        .placeholder("What should the agent do next?", muted())
        .caret(ink(), rgba(110, 165, 255, 0.3))
        .on_edit(|s: &mut State| &mut s.reply)
        // Return is a new line here, as in the web window's box; ⌘Return sends.
        .on_submit(|s: &mut State, _| s.reply.insert("\n"))
        .on_input_key(crate::keys::in_the_reply)
        // A picture or a file on the clipboard is attached rather than typed (`onPaste`).
        .on_input_paste(|s: &mut State, content, _| s.paste_into_the_box(Holder::Reply, content, crate::clock()));
    // Send is lit for words or files, and off while the files of a reply are being saved, when
    // it says so (`disabled={!hasDraft || saving}`).
    let thread = key(&item.product, &item.id);
    let saving = s.reply_files.saving.as_deref() == Some(thread.as_str());
    let ready = (!s.reply.text.trim().is_empty() || !s.reply_files.open.is_empty()) && !saving;
    // The model, with its level where one will be sent (`.tc-chip-level`), brighter inside a
    // stronger edge while its menu is up (`.tc-chip.open`).
    let open = s.reply_menu.is_some();
    // A box narrow enough drops the chips' marks and the level (`@container reply`).
    let compact = crate::chips::compact(m.column);
    let (model, level_said) = s.reply_chip_words(item);
    let said = level_said.filter(|_| !compact).map_or_else(|| model.clone(), |level| format!("{model} · {level}"));
    // Led by the mark of the agent the reply goes to (`EngineLogo`, the box's `engineShown`).
    let (_, shown) = s.reply_engines(item);
    let said = pen(label(&said, 13.0, 400.0, if open { ink() } else { dim() }).group_color(ink()), 13.0);
    let model_chip = home::chip(if compact { vec![said] } else { vec![engine_logo(&shown), said] })
        .key(reply::CHIP)
        .cursor(rune::Cursor::Pointer)
        .label("Which model this conversation runs on")
        .help("Which model this conversation runs on")
        .on_background_press_tx(|s: &mut State, tx| s.toggle_reply_menu(tx));
    let model_chip = if open { model_chip.border(1.0, rule()) } else { model_chip };
    // The priority the next reply sets and what it may do, each a chip that opens its menu
    // (`chips.rs`; no mode on a terminal agent's row). The paper clip opens the system's file
    // picker (`.dock-clip`). Send and its caret are two controls, as there (`.dock-send`,
    // `.dock-send-later`); the caret answers the pointer and opens nothing yet.
    let send = if compact { crate::chips::send_arrow(ready) } else if saving { home::send_sending() } else { home::send_main(ready) };
    // While the thread is under way and the box holds nothing, Stop stands in Send's place,
    // its caret with it; the first thing typed makes it Send again (`stop && !hasDraft`).
    let send_pair = if s.stop_in_the_box(item) {
        row().key(crate::chips::SEND_PAIR).child(crate::answer::stop_button(s, item, compact))
    } else {
        row().key(crate::chips::SEND_PAIR).child(send.key(reply::SEND).help("Send · ⌘↵").on_press_tx(|s: &mut State, tx| s.send_reply(tx))).child(home::send_caret().help("Send later").on_press_tx(|s: &mut State, tx| s.tell(SEND_LATER_NOT_HERE, tx)))
    };
    let mode = crate::chips::offers_a_mode(item).then(|| crate::chips::mode_chip(s, item, compact, crate::chips::mode_folded()));
    let bar = row()
        .key(crate::chips::BAR)
        .h(57.0)
        .px(16.0)
        .gap(8.0)
        .items_center()
        .border_sides([1.0, 0.0, 0.0, 0.0], edge())
        .border_layout()
        .child(crate::chips::priority_chip(s, item, compact))
        .children(mode)
        .child(model_chip)
        .child(spacer())
        // The paper clip's square is 28 points, the chips' height (`.tc-icon`).
        .child(home::attach().size(28.0, 28.0).help("Attach files").on_background_press_tx(|s: &mut State, tx| s.pick_files(Holder::Reply, tx)))
        // Send carries the key a press away from the model menu lands on (`reply.rs`).
        .child(send_pair);
    // The thread moves to another agent with the next reply, and its session cannot follow it:
    // a note says so while that is true (`freshNote`, `.dock-card .compose-note`).
    let note = s.reply_fresh_note(item).map(|note| el().px(16.0).pb(10.0).child(label(&note, 12.0, 400.0, muted())));
    // Pasted pictures past what a draft keeps still go, and the box says a close loses them
    // (`tooBig`), before the note about a move.
    let too_big = crate::drafts::too_big_note(s.kept_drafts.too_big).map(|note| el().px(16.0).pb(10.0).child(label(&note, 12.0, 400.0, muted())));
    // The files it carries, between the words and the notes (`.attach-row` in `.dock-card`):
    // eight points under the words, flush on the bar, and the line saying why one cannot go.
    let files = crate::attach::rows(s, Holder::Reply, |r| r.pt(8.0).px(18.0), |n| n.px(16.0).pb(10.0));
    // The words fill the box above the bar, as the web window's text box does: the I-beam over
    // all of it, and a press anywhere in it puts the caret in the words.
    let area = el().min_h(92.0).pt(12.0).px(16.0).pb(6.0).cursor(rune::Cursor::Text).on_press_tx(|s: &mut State, tx| {
        s.reply.focused = true;
        tx.focus("reply", false);
    });
    col().w(m.column)
        .radius(10.0)
        .fill(white(0.04))
        .border(1.0, rule())
        .border_layout()
        .box_shadow(shadow(0.0, 10.0, 32.0, 0.0, rgba(0, 0, 0, 0.38), false))
        // Files dropped anywhere on the box attach. The web window takes them on its words,
        // which are most of it, as the box a thread is started in does here.
        .on_drop_files(|s: &mut State, files, _| s.attach_paths(Holder::Reply, files))
        .child(area.child(field.w_full()))
        .children(files)
        .children(too_big)
        .children(note)
        .child(bar)
}

/// The agent's choices over the reply box (`.opt-strip`): each with its number, the one the
/// arrows are on lit, and what Enter would do said beside it.
fn options_strip(choices: &[Choice], on: Option<usize>, m: &Measure) -> El {
    let count = choices.len();
    let rows = choices.iter().enumerate().map(|(at, c)| {
        let lit = on == Some(at);
        let tone = if lit { ink() } else { dim() };
        let number = row().size(20.0, 20.0).radius(6.0).center().border(1.0, if lit { muted() } else { rule() }).child(text(format!("{}", c.n)).font(mono(11.0, 11.0)).color(tone));
        let words = format::without_recommended(&c.text);
        let mut line = row()
            .key(format!("option-{at}"))
            .pt(if at == 0 { 7.0 } else { 3.0 })
            .pb(if at + 1 == count { 7.0 } else { 3.0 })
            .pl(14.0)
            .pr(44.0)
            .gap(10.0)
            .cursor(rune::Cursor::Pointer)
            // A press is aimed at the place it lands: not taken when the options moved under the
            // pointer, or the app moved the person here, a moment before (`answer.rs`).
            .on_press_tx(move |s: &mut State, tx| s.click_option(at, tx))
            .child(number)
            .child(pen(text(words).font(TextStyle::new(14.5, 400.0).lh(20.0)).color(tone).group_color(ink()).wrap().max_lines(2), 14.5));
        // The lit one keeps its stronger wash under the pointer; any other is washed, its words
        // the text's own colour (`.opt-row:hover`, which `.opt-row.selected` comes after).
        line = if lit { line.fill(white(0.08)) } else { line.hover_group().hover(crate::feel::wash()) };
        // Beside the lit one, what Enter does; beside the agent's own pick, that it is.
        let aside = if lit { Some("↵ send") } else if c.recommended { Some("recommended") } else { None };
        match aside {
            Some(said) => line.child(el().pl(8.0).mt(1.0).child(pen(text(said).font(TextStyle::new(11.5, 400.0).lh(20.0)).color(muted()), 11.5))),
            None => line,
        }
    });
    el().w(m.column)
        .key("options-strip")
        // Where the strip stands while the pointer is on it, and where the pointer is, so a
        // press can be told from one aimed at what stood there before (`stripUnderThePointer`).
        .on_pointer_move_local(|s: &mut State, p, _| {
            s.answering.strip = Some(p.window_y - p.y);
            s.answering.pointer = Some((p.window_x, p.window_y));
        })
        .on_hover_change(|s: &mut State, over, _| {
            if !over {
                s.answering.strip = None;
            }
        })
        .child(col().w(m.column).radius(10.0).clip_rounded(10.0).border(1.0, edge()).border_layout().children(rows))
        // The little arrow that folds the choices away, washed under the pointer, its mark the
        // text's own colour (`ChoicesFold`, `.one-choices-fold:hover`).
        .child(row().pin(None, Some(6.0), Some(8.0), None).size(24.0, 24.0).radius(6.0).center().hover_group().hover(crate::feel::wash()).cursor(rune::Cursor::Pointer).key("choices-fold").label("Fold the choices").child(icon(&icons::DOWN, 14.0).color(muted()).group_color(ink())).on_press(|s: &mut State| crate::chips::toggle_choices(s)))
}

/// The dock at the conversation's foot (`.focus-dock`): the choices the agent's newest answer
/// still offers, over the reply box, folded to one line or open.
fn dock(s: &State, item: &Item, m: &Measure) -> El {
    let mut inner = col().w(m.column);
    let choices = live_options(item);
    // An answer on its way has spent the offer, though the engine has not heard it yet. Folded
    // away, the choices show only while one is picked out with the keys
    // (`.one-choices-folded:not(:has(.opt-row.selected))`).
    // While the question before a discard is up, it stands where the options stood and they go.
    if !choices.is_empty() && !s.answer_waits(item) && s.discard_asked().is_none() && (s.option.is_some() || !crate::chips::choices_folded(s, item)) {
        // The strip moving four points or more under a pointer resting on it is noticed here,
        // as the page is drawn, and a press in the next moment is not taken.
        if let (Some(rested), Some([_, top, _, _])) = (s.answering.strip, rune::memo::window_layout("options-strip")) {
            let seen = s.answering.strip_moved.get().map_or(rested, |(at, _)| at);
            if (top - seen).abs() >= crate::rules::answering::MOVED_PX {
                s.answering.strip_moved.set(Some((top, s.at)));
            }
        }
        inner = inner.child(options_strip(&choices, s.option, m).mb(10.0));
    }
    inner = inner.child(if s.replying { composer(s, item, m) } else { reply_box(s, item, m).cursor(rune::Cursor::Text).on_press_tx(|s: &mut State, tx| s.open_reply(tx)) });
    el().pl(m.indent).pr(m.right).pb(m.foot).child(inner)
}

/// The reply box alone, `wide` across, under a file that takes the whole card (`pane.rs`).
pub fn dock_alone(s: &State, id: &str, product: &str, wide: f32) -> El {
    match item(s, id, product) {
        Some(item) => dock(s, item, &Measure::under_file(wide)),
        None => el(),
    }
}

/// The choices a thread offers now, while the offer still stands (`liveOption` in `renderer/src/format.ts`).
pub fn live_options(item: &Item) -> Vec<Choice> {
    if format::offer_is_live(item) { format::item_options(item) } else { Vec::new() }
}

/// The conversation `product/id`. `still`: the system asks for reduced motion, and what loops
/// in the web window stands still here as it does there.
pub fn view(s: &State, id: &str, product: &str, still: bool) -> El {
    // Beside a file, the conversation is what the file leaves of the page (`pane.rs`).
    let (page_w, page_h) = s.conversation_size();
    let m = if s.pane_placement() == Some(crate::rules::pane::Placement::Beside) { Measure::beside(page_w) } else { Measure::of(page_w) };
    let Some(item) = item(s, id, product) else {
        return row().w(page_w).pt(36.0).pl(m.indent).child(label("This conversation is not here any more.", 14.0, 400.0, muted()));
    };
    let thread = key(product, id);
    let unfolded = s.reading.unfolded.contains(&thread) != pictured_unfolded(s, &thread);
    // The run on the thread right now, and the sentence it is typing this second.
    let session = running(s, id, product);
    let working = session.is_some();
    let saying = session.as_ref().and_then(|r| Some(Saying { text: r.saying.clone().filter(|t| !t.is_empty())?, at: r.saying_at?, run: r.started_at }));
    let imported = item.labels.iter().any(|l| l == "thread-import" || l.starts_with("codex:"));
    let title = name(item);
    let every_run = s.reading.whole(&thread) || pictured_unfolded(s, &thread);
    let talk = read(s, item, every_run, working, saying.as_ref());
    let page = talk.page(working, unfolded, imported);
    // What the files this conversation names are read against: its project's own folders.
    let files = Files::of(s, item);

    let mut summary = col().child(readable(text(title.as_str()).font(TextStyle::new(m.title.0, 500.0).lh(m.title.1).tracking(-0.015 * m.title.0)).color(ink()).wrap()));
    if let Some(words) = &page.toggle {
        summary = summary.child(toggle(words, thread.clone(), &m).mt(m.title_gap));
    }
    // Under it, the summary the thread carries, or while an agent works what was asked.
    let problem = item.rest.get("problem").and_then(Value::as_str).filter(|p| !p.trim().is_empty());
    let context = match problem {
        Some(problem) => Some(problem.to_string()),
        None if working => live::working_context(item.body.as_deref().or(item.title.as_deref()).unwrap_or(""), &title),
        None => None,
    };
    if let Some(words) = context {
        summary = summary.child(context_line(&words, &m).mt(if page.toggle.is_some() { 10.0 } else { 8.0 }));
    }
    // Under the summary: the newest answer alone, or the conversation with its outcome. A chat
    // just brought in says first that it is being got ready, and then shows its newest answer,
    // 28 points under that line (`catchUpDue` in `ItemThread.tsx`).
    let mut blocks = vec![markdown::block(0.0, 18.0, summary)];
    let getting = crate::rules::kinds::catch_up_due(item);
    if getting {
        blocks.push(markdown::block(8.0, 8.0, getting_ready(s, &thread, &m, still)));
    }
    match page.alone.or(if getting { talk.newest() } else { None }) {
        Some(answer) if getting => {
            let (body, top, bottom) = markdown::stack(answer_alone(answer, &title, &m, &files), false);
            blocks.push(markdown::block(top.max(28.0), bottom, body));
        }
        Some(answer) => blocks.extend(answer_alone(answer, &title, &m, &files)),
        None if getting => {}
        None => {
            if page.said.is_empty() && talk.outcome.is_none() {
                blocks.push(markdown::block(0.0, 0.0, el().py(8.0).child(label("Nothing has been said here yet.", 14.0, 400.0, muted()))));
            } else {
                blocks.push(markdown::block(0.0, 0.0, conversation(s, &thread, &page, s.now, &m, &files)));
            }
            if let Some(answer) = &talk.outcome {
                // The agent's own close leads with its result as it is; anything else is labelled.
                let closed_by_agent = item.status == "done" && item.rest.get("wrote").and_then(|w| w.get("status")).and_then(|st| st.get("source")).and_then(Value::as_str) != Some("founder");
                let (body, top, bottom) = markdown::stack(outcome(answer, answer.field == Field::Result && closed_by_agent, &m, &files), false);
                blocks.push(markdown::block(top.max(28.0), bottom.max(8.0), body));
            }
        }
    }
    // What the person did after the answer, under it and in order.
    if !talk.after.is_empty() {
        blocks.push(markdown::block(22.0, 0.0, col().gap(9.0).children(talk.after.iter().map(step))));
    }
    // And whether anything is happening, under the last message and over the reply box.
    let asking = s.snap.rest.get("approvals").and_then(Value::as_array).is_some_and(|a| a.iter().any(|a| a.get("item").and_then(Value::as_str) == Some(id)));
    let why_not_cloud = s.snap.supervisor.rest.get("whyNotCloud").and_then(|w| w.get(id)).and_then(|w| w.get("at")).and_then(Value::as_str);
    if let Some(line) = live::working(session.as_ref(), &item.status, &item.labels, asking, why_not_cloud, s.now) {
        let run = format!("{thread}|{}", session.as_ref().map_or(0.0, |r| r.started_at));
        // What it opens onto: the steps the run that is up took since it last said anything,
        // and the calls running now (`stepsSinceLastSaid` of the newest run, `previousActivity`).
        let started = session.as_ref().map_or(0.0, |r| r.started_at);
        let took: Vec<live::Took> = talk.trailing.iter().filter(|w| w.at >= started).map(|w| live::Took { at: w.at, verb: w.verb.clone(), subject: w.subject.clone(), full: w.full.clone() }).collect();
        let steps = live::steps(&took, session.as_ref().map_or(&[][..], |r| r.activity.as_slice()));
        // The line's two points under it are its own (`.live-brief`), and its steps, open, end
        // the block with none.
        let under = if s.reading.live_open.as_deref() == Some(run.as_str()) { 0.0 } else { 2.0 };
        blocks.push(markdown::block(14.0, under, live_line(s, &thread, run, &line, &steps, &m, still)));
    }
    // The question before throwing the change away, at the conversation's foot (`closeAskStrip`).
    if let Some(strip) = crate::answer::discard_strip(s, m.column) {
        blocks.push(markdown::block(22.0, 6.0, strip));
    }
    let (body, _, under) = markdown::stack(blocks, false);
    // A browser scrolls a page only as far as a whole device pixel, whatever its lines add up
    // to, so the foot of a long conversation rests on one (1211.61 points scroll to 1211.5 on a
    // 2x screen). Once the words have been laid out, the room under them takes up the difference.
    let scale = s.pictures.scale().max(1.0);
    let whole = rune::memo::layout_size("thread-said").map_or(0.0, |[_, tall]| {
        let page = 28.0 + tall + 40.0 + under;
        (page * scale).floor() / scale - page
    });
    // Unfolding lands on the newest thing said, at the foot, as does a conversation drawn from
    // the answer an agent is working from; a page that leads with the answer opens at its top.
    let reading = col()
        .key(if unfolded { "thread-whole" } else { "thread-newest" })
        .grow(1.0)
        .min_h(0.0)
        // So does one the person has just sent something to: their words are where they look.
        .scroll(ScrollSpec { stick_bottom: unfolded || (working && page.alone.is_none()) || s.answering.sending.iter().any(|d| d.thread == thread) })
        .child(el().pt(28.0).pl(m.indent).pr(m.right).pb(40.0 + under + whole).child(body.key("thread-said").w(m.column).selection_group(true)));
    // A press on nothing in the page folds an open box that holds nothing, on Escape's own rule:
    // a draft holds it open (`DockComposer`'s `mousedown` away).
    col().w(page_w).h(page_h).on_background_press(|s: &mut State| {
        if s.replying && s.reply.text.trim().is_empty() && s.reply_files.open.is_empty() {
            s.fold_reply();
        }
    })
    .child(reading)
    .child(dock(s, item, &m))
}

#[cfg(test)]
mod tests {
    use rune::{Key, Runtime};
    use serde_json::json;

    use crate::State;
    use crate::feel::tests::{key, press, run, window_dressed};

    /// The invented inbox's first thread with an agent on it, five minutes into a run that is
    /// running the tests, opened.
    fn working(name: &str) -> Runtime<State> {
        window_dressed(name, &["--thread", "w-feel0"], "/fixtures/storefront", "Done, and tested. Merge it?", |fixture| {
            let now = fixture["now"].as_f64().unwrap();
            fixture["snapshot"]["items"][0]["status"] = json!("claimed");
            fixture["snapshot"]["supervisor"]["running"] = json!([{
                "itemId": "w-feel0", "product": "storefront", "startedAt": now - 300_000.0, "tail": [],
                "activity": [{ "id": "toolu_1", "label": "Running the tests", "detail": "npm test", "startedAt": now - 20_000.0 }],
            }]);
        })
    }

    /// How many elements on screen run a loop.
    fn looping(rt: &Runtime<State>) -> usize {
        rt.elements().iter().filter(|e| e.loop_phase.is_some()).count()
    }

    #[test]
    fn the_working_lines_mark_turns_and_its_word_shimmers_once_it_is_on_screen_and_not_under_reduce_motion() {
        let mut rt = working("live-loops");
        run(&mut rt, 0.1);
        assert_eq!(rt.state.reading.live_seen.as_deref(), Some("storefront/w-feel0"), "the line reported itself on screen");
        // Eight cells of the lattice, and the veil's strips over the word.
        let loops = looping(&rt);
        assert!(loops > 8 + 20, "{loops} loops");
        // Under Reduce motion everything of it stands still, as the web window's does.
        rt.set_system_reduced_motion(true);
        run(&mut rt, 0.1);
        assert_eq!(looping(&rt), 0);
    }

    #[test]
    fn a_press_opens_the_working_line_onto_its_steps_and_escape_folds_them_before_leaving() {
        let mut rt = working("live-steps");
        run(&mut rt, 0.1);
        let (x, y) = rt.find("live-word").expect("the line's word");
        press(&mut rt, x, y);
        assert!(rt.state.reading.live_open.is_some(), "open onto its steps");
        assert!(rt.find("live-word").is_some());
        // The running call is a row of it.
        assert!(rt.elements().iter().any(|e| e.text.as_deref() == Some("Running the tests") && e.rect[1] > y + 10.0), "the call running now is listed under the line");
        // Escape folds the steps and stays in the conversation; the next one leaves it.
        key(&mut rt, Key::Escape);
        assert!(rt.state.reading.live_open.is_none());
        assert!(matches!(rt.state.view, crate::View::Thread { .. }));
        key(&mut rt, Key::Escape);
        assert!(!matches!(rt.state.view, crate::View::Thread { .. }));
    }
}
