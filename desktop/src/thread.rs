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

use std::collections::{HashMap, HashSet};

use rune::time::LocalTime;
use rune::{El, Icon, ScrollSpec, TextCase, TextStyle, col, el, icon, row, spacer, text, text_input};
use serde_json::{Map, Value, json};

use crate::data::{Item, Priority};
use crate::files::Files;
use crate::home;
use crate::markdown::{self, Block};
use crate::rules::format::{self, Choice};
use crate::rules::notes::TraceSession;
use crate::rules::{live, place, work};
use crate::talk::{self, Answers, Field, Line, Message, Page, Said, Saying, Step, Talk, Work};
use crate::theme::*;
use crate::{State, View, chrome, icons};

/// What the person has unfolded while reading: the threads whose earlier messages are shown,
/// those whose whole middle was asked for, and the folds of what a run did that are open.
#[derive(Default)]
pub struct Reading {
    unfolded: HashSet<String>,
    whole: HashSet<String>,
    runs_open: HashSet<String>,
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
}

impl Measure {
    fn of(card: f32) -> Measure {
        let size = (0.0025 * card + 13.0).clamp(16.0, 18.0);
        let column = (size * 47.5).min(card - 96.0);
        Measure { size, column, indent: ((card - column) * 0.35).max(32.0) }
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

/// The tab the strip names: the one that holds the thread now, not the one it was opened from,
/// so a thread that finishes while it is read stops saying In progress (`crumbTab` in
/// `renderer/src/threads/page-rules.ts`). Needs you is called the Inbox there.
fn crumb_from(s: &State, id: &str, product: &str) -> &'static str {
    let holds = |list: &[crate::data::Row]| list.iter().any(|r| r.id == id && r.product == product);
    if holds(&s.lists.progress) {
        "In progress"
    } else if holds(&s.lists.later) {
        "Later"
    } else if holds(&s.lists.done) && !holds(&s.lists.needs) {
        "Done"
    } else {
        "Inbox"
    }
}

/// The card's top strip over a conversation: the Inbox, back and forward, where the thread
/// is, the threads before and after it in Needs you, and what can be done to it.
pub fn head(s: &State, id: &str, product: &str) -> El {
    let (card_w, _) = s.card_size();
    let square = |glyph: &'static Icon, size: f32, side: f32, color| row().size(side, side).radius(6.0).center().child(icon(glyph, size).color(color));
    let at = s.lists.needs.iter().position(|r| r.id == id && r.product == product);
    let step = |glyph: &'static Icon, to: Option<&crate::data::Row>| {
        let b = square(glyph, 14.0, 30.0, dim());
        match to {
            Some(r) => {
                let (id, product) = (r.id.clone(), r.product.clone());
                b.on_press(move |s: &mut State| s.step_to_thread(&id, &product))
            }
            None => b.opacity(0.35),
        }
    };
    let crumbs = mono(11.0, 14.0).tracking(0.99);
    let title = item(s, id, product).map(name).unwrap_or_default();
    let finished = item(s, id, product).is_some_and(|i| i.status == "done");
    let close = row().h(30.0).items_center().gap(7.0).pl(9.0).pr(12.0).radius(6.0).border(1.0, edge()).border_layout();
    let close = if finished {
        close.child(label("Reopen", 13.0, 400.0, dim())).on_press(|s: &mut State| s.reopen_thread())
    } else {
        close.child(icon(&icons::DONE, 16.0).color(dim())).child(label("Done", 13.0, 400.0, dim())).on_press_tx(|s: &mut State, tx| s.close_thread(false, tx))
    };
    // Where the thread runs, for one the cloud could take (`PlaceButton.tsx`): a cloud or a Mac,
    // faded while the whole cloud is off, dotted when it was handed back for something.
    let place = item(s, id, product).and_then(|i| place::thread_place(&s.snap, i).map(|p| place::mark(&s.snap, i, p))).map(|m| {
        let glyph = icon(if m.cloud { &icons::PLACE_CLOUD } else { &icons::PLACE_MAC }, 16.0).color(dim()).opacity(if m.faded { 0.7 } else { 1.0 });
        let button = row().size(30.0, 30.0).radius(6.0).center().border(1.0, edge()).child(glyph);
        if m.dot { button.child(el().absolute(20.0, 4.0).size(6.0, 6.0).radius(3.0).fill(accent())) } else { button }
    });
    let mut actions = row().ml(16.0).min_w(0.0).items_center();
    // The line about what was just done stands first in this group (`.toast-in-bar`), and the
    // thread's name gives way to it.
    if let Some(line) = chrome::toast_in_bar(s) {
        actions = actions.child(line.mr(16.0));
    }
    // The threads before and after it are those of Needs you, so a thread elsewhere has none
    // (`QueueNav` stands only on a thread with a place in that queue).
    if at.is_some() {
        actions = actions
            .child(step(&icons::UP, at.and_then(|a| a.checked_sub(1)).and_then(|a| s.lists.needs.get(a))))
            .child(step(&icons::DOWN, at.and_then(|a| s.lists.needs.get(a + 1))));
    }
    let mut actions = actions;
    if let Some(place) = place {
        actions = actions.child(place.ml(10.0));
    }
    let actions = actions.child(close.ml(10.0)).child(square(&icons::MORE, 16.0, 30.0, dim()).ml(10.0).border(1.0, edge()));
    row().absolute(chrome::CARD[0], chrome::CARD[1])
        .size(card_w, 56.0)
        .items_center()
        .pl(20.0)
        .pr(28.0)
        .border_sides([0.0, 0.0, 1.0, 0.0], edge())
        .border_layout()
        .child(square(&icons::INBOX, 15.0, 26.0, ink()).mr(10.0).on_press(|s: &mut State| s.go(View::Inbox)))
        // Back is where the person came from, and a thread opened from nowhere came from the Inbox.
        .child(square(&icons::BACK, 15.0, 26.0, ink()).ml(2.0).on_press(|s: &mut State| s.leave_thread()))
        .child(square(&icons::FORWARD, 15.0, 26.0, muted()).ml(2.0).opacity(0.45))
        .child(
            row().grow(1.0)
                .min_w(0.0)
                .h(28.0)
                .ml(12.0)
                .items_center()
                .child(text(crumb_from(s, id, product)).font(crumbs).text_case(TextCase::Uppercase).color(muted()).mr(8.0).on_press(|s: &mut State| s.go(View::Inbox)))
                .child(text("/").font(crumbs).color(muted()).ml(5.0).mr(7.0))
                .child(row().min_w(0.0).px(6.0).child(text(crumb(&title)).font(crumbs).text_case(TextCase::Uppercase).color(dim()).truncate())),
        )
        .child(actions)
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

/// A message's head: who, when, and for a checkpoint the tag that says so.
fn said_by(m: &Message, now: f64) -> El {
    let line = kept(22.4);
    let who = if m.mine { text("You").font(TextStyle::new(14.0, 600.0).lh(line)).color(ink()) } else { text("The agent").font(TextStyle::new(14.0, 400.0).lh(line)).color(dim()) };
    // The three sit on one baseline: the smaller the type, the lower its box starts.
    let mut head = row().h(line).items_start().gap(10.0).child(who).child(text(when(m.at, now)).font(TextStyle::new(12.5, 400.0).lh(20.0).features("'tnum' 1")).color(muted()).mt(1.5));
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
                .child(text("Checkpoint").font(tag.tracking(0.22)).color(dim()))
                .child(row().gap(6.0).child(text("·").font(tag).color(muted())).child(text("progress so far").font(tag).color(muted()))),
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
            crate::rules::typed::Part::Embed(line) => markdown::stack(markdown::blocks(line, m.size, m.column, files), true).0,
        };
        drawn.mt(if at == 0 { 0.0 } else { 10.0 })
    }))
}

/// One message, and how much of what it holds reaches under it.
fn message(msg: &Message, turn: (bool, bool), now: f64, m: &Measure, files: &Files) -> El {
    let (body, leak) = if msg.mine {
        (typed_with_files(&msg.text, m, files), 0.0)
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
/// shuts it at once.
fn run_line(items: &[&Work], open: bool, key: String, m: &Measure) -> El {
    let summary = work::activity_summary(items.iter().map(|w| (w.verb.as_str(), w.subject.as_str())));
    let failed = work::run_failures(items.iter().filter(|w| w.failed).count());
    let mut head = row().max_w(m.column).items_start().gap(8.0).child(row().min_w(0.0).child(text(summary).font(TextStyle::new(13.0, 400.0).lh(19.5)).color(dim()).truncate()));
    if !failed.is_empty() {
        head = head.child(text(failed).font(TextStyle::new(12.0, 400.0).lh(18.0)).color(stopped()).mt(1.0));
    }
    // Its mark shows only open, turned down, after the words (`order: 2; opacity: 0`).
    if open {
        head = head.child(icon(&icons::WORK_MARK_OPEN, 8.0).color(muted()).mt(5.75));
    }
    let head = row().key(key.clone()).cursor(rune::Cursor::Pointer).child(head).on_press(move |s: &mut State| {
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

/// The line at the foot of a conversation an agent is on (`Live.tsx`, working): the mark, the
/// step it is on in its own words, how many more run beside it, how long it has been at it,
/// and where it runs when that is news. The web window's mark turns and its word shimmers;
/// here both stand still, the mark at the web's own opacity for no motion.
fn live_line(line: &live::Working, m: &Measure) -> El {
    let font = TextStyle::new(13.5, 400.0).lh(20.0);
    let cell = |lit: bool| el().size(14.0 / 3.0 - 1.0, 14.0 / 3.0 - 1.0).radius(1.85).fill(ink()).opacity(if lit { 0.65 } else { 0.1 });
    let mark = col().size(14.0, 14.0).gap(1.5).children((0..3).map(|r| row().gap(1.5).children((0..3).map(move |c| cell(!(r == 1 && c == 1))))));
    // The step is said aloud each time it changes (`live-announcement`, `role="status"`).
    let word = text(line.word.as_str()).font(TextStyle::new(13.5, 500.0).lh(20.0)).color(ink()).truncate().live_region(rune::LiveRegion::Polite);
    // Rune's flex never shrinks a child below what it holds unless told to (`shrink`), and
    // the step is the one part that gives way.
    let mut words = row().items_center().gap(6.0).shrink().child(row().shrink().child(word));
    for span in [line.more.as_deref(), Some(line.span.as_str()), line.place.as_deref()].into_iter().flatten() {
        words = words.child(text(span).font(font.features("'tnum' 1")).color(muted()));
    }
    // No wider than the column: a step is often a whole command line, and it is cut short with
    // dots, as `.live-disclosure .live-word` cuts it, rather than run past the page. The stack
    // it stands in takes its width from what is in it, so the column is said here.
    row().max_w(m.column).items_center().gap(7.0).child(mark).child(words)
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

/// The line under the title that folds and unfolds the rest: small, quiet and underlined.
fn toggle(words: &str, thread: String, m: &Measure) -> El {
    let size = (0.7 * m.size).max(12.5);
    let line = kept(1.6 * size);
    // The underline sits 4 points under the baseline, which Geist puts 1.005 of the size down,
    // on the nearest device pixel of a 2x screen.
    let under = (((line - 1.3 * size) / 2.0 + 1.005 * size + 4.0) * 2.0).round() / 2.0;
    row().child(
        el().key("thread-earlier")
            .child(text(words).font(TextStyle::new(size, 400.0).lh(line)).color(muted()))
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
        blocks.insert(0, markdown::block(0.0, 7.0, text(words).font(TextStyle::new(12.5, 400.0).lh(20.0)).color(muted())));
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
        let size = kept(1.05 * m.size);
        blocks.insert(0, markdown::block(0.0, 10.0, text(words).font(TextStyle::new(size, 600.0).lh(kept(1.45 * size))).color(ink()).wrap()));
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
    let print = format!("{thread}|{from}|{whole}|{working}|{saying:?}");
    if let Some(talk) = READ.with(|last| last.borrow().as_ref().filter(|(p, _)| *p == print).map(|(_, t)| t.clone())) {
        return talk;
    }
    let (lines, sessions) = ledger(s, item);
    let mut talk = Talk::read(&lines, &sessions, saying, whole);
    if working {
        talk.without_trailing_work();
    }
    let talk = std::rc::Rc::new(talk);
    READ.with(|last| *last.borrow_mut() = Some((print, talk.clone())));
    talk
}

/// Whether a fixture's picture was taken with this thread's earlier messages unfolded.
fn pictured_unfolded(s: &State, thread: &str) -> bool {
    s.fixture.get("unfolded").and_then(Value::as_array).is_some_and(|list| list.iter().any(|k| k.as_str() == Some(thread)))
}

/// The reply box folded to its one line, as a conversation first shows it. R, or a press,
/// opens it (`composer`).
fn reply_box(m: &Measure) -> El {
    let size = (0.78 * m.size).max(14.5);
    let model = el()
        .child(label("Opus 5.5", 13.0, 400.0, dim()))
        // `underline dotted`, three points under the baseline: a dot every three points, as
        // far as the words reach.
        .child(row().pin(Some(-0.25), Some(16.0), Some(0.0), None).h(1.5).gap(1.5).clip_rounded(0.0).children((0..18).map(|_| el().size(1.5, 1.5).fill(rule()))));
    let key = row().px(5.5).py(1.5).radius(6.0).fill(white(0.04)).border(1.0, rule()).border_layout().child(text("R").font(mono(10.5, 13.5)).color(dim()));
    row().w(m.column)
        .radius(10.0)
        .fill(white(0.04))
        .border(1.0, edge())
        .border_layout()
        .box_shadow(shadow(0.0, 10.0, 32.0, 0.0, rgba(0, 0, 0, 0.38), false))
        .items_center()
        .child(el().px(16.0).py(11.0).child(label("Reply…", size, 400.0, muted())))
        .child(spacer())
        .child(model.mr(12.0))
        .child(key.mr(14.0))
}

/// The reply box opened (`.dock-card`): the words, then who takes them and Send. The words
/// are the person's to type; the three chips are drawn, not yet chosen.
fn composer(s: &State, item: &Item, m: &Measure) -> El {
    let field = text_input(&s.reply, TextStyle::new(15.5, 400.0).lh(kept(24.025)))
        .key("reply")
        .color(ink())
        .placeholder("What should the agent do next?", muted())
        .caret(ink(), rgba(110, 165, 255, 0.3))
        .on_edit(|s: &mut State| &mut s.reply)
        // Return is a new line here, as in the web window's box; ⌘Return sends.
        .on_submit(|s: &mut State, _| s.reply.insert("\n"))
        .on_input_key(crate::keys::in_the_reply);
    // The thread's own level, as the Inbox marks it, with its word.
    let off = rule();
    let level = match Priority::of(item.priority) {
        Priority::Urgent => (bars(&[4.0, 7.0, 10.0, 13.0], ink()), "Urgent"),
        Priority::High => (bars(&[4.0, 7.5, 11.0], ink()), "High"),
        Priority::Medium => (row().items_end().gap(1.5).child(el().size(3.0, 4.0).radius(1.0).fill(dim())).child(el().size(3.0, 7.5).radius(1.0).fill(dim())).child(el().size(3.0, 11.0).radius(1.0).fill(off)), "Medium"),
        Priority::Low => (row().items_end().gap(1.5).child(el().size(3.0, 4.0).radius(1.0).fill(dim())).child(el().size(3.0, 7.5).radius(1.0).fill(off)).child(el().size(3.0, 11.0).radius(1.0).fill(off)), "Low"),
    };
    let ready = !s.reply.text.trim().is_empty();
    let bar = row()
        .h(57.0)
        .px(16.0)
        .gap(8.0)
        .items_center()
        .border_sides([1.0, 0.0, 0.0, 0.0], edge())
        .border_layout()
        .child(home::chip(vec![level.0, label(level.1, 13.0, 400.0, dim())]))
        .child(home::chip(vec![label("Auto", 13.0, 400.0, dim())]))
        .child(home::chip(vec![label("Opus 5.5", 13.0, 400.0, dim())]))
        .child(spacer())
        .child(home::attach())
        .child(home::send_pair(ready).on_press_tx(|s: &mut State, tx| s.send_reply(tx)));
    col().w(m.column)
        .radius(10.0)
        .fill(white(0.04))
        .border(1.0, rule())
        .border_layout()
        .box_shadow(shadow(0.0, 10.0, 32.0, 0.0, rgba(0, 0, 0, 0.38), false))
        .child(el().min_h(92.0).pt(12.0).px(16.0).pb(6.0).child(field.w_full()))
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
            .on_press_tx(move |s: &mut State, tx| s.pick_option(at, tx))
            .child(number)
            .child(pen(text(words).font(TextStyle::new(14.5, 400.0).lh(20.0)).color(tone).wrap().max_lines(2), 14.5));
        if lit {
            line = line.fill(white(0.08));
        }
        // Beside the lit one, what Enter does; beside the agent's own pick, that it is.
        let aside = if lit { Some("↵ send") } else if c.recommended { Some("recommended") } else { None };
        match aside {
            Some(said) => line.child(el().pl(8.0).mt(1.0).child(pen(text(said).font(TextStyle::new(11.5, 400.0).lh(20.0)).color(muted()), 11.5))),
            None => line,
        }
    });
    el().w(m.column)
        .child(col().w(m.column).radius(10.0).clip_rounded(10.0).border(1.0, edge()).border_layout().children(rows))
        // The caret that folds the choices away, drawn and not yet pressed.
        .child(row().pin(None, Some(6.0), Some(8.0), None).size(24.0, 24.0).radius(6.0).center().child(icon(&icons::DOWN, 14.0).color(muted())))
}

/// The dock at the conversation's foot (`.focus-dock`): the choices the agent's newest answer
/// still offers, over the reply box, folded to one line or open.
fn dock(s: &State, item: &Item, m: &Measure) -> El {
    let mut inner = col().w(m.column);
    let choices = live_options(item);
    // An answer on its way has spent the offer, though the engine has not heard it yet.
    if !choices.is_empty() && !s.answer_waits(item) {
        inner = inner.child(options_strip(&choices, s.option, m).mb(10.0));
    }
    inner = inner.child(if s.replying { composer(s, item, m) } else { reply_box(m).cursor(rune::Cursor::Text).on_press_tx(|s: &mut State, tx| s.open_reply(tx)) });
    el().pl(m.indent).pr(32.0).pb(24.0).child(inner)
}

/// The choices a thread offers now, while the offer still stands (`liveOption` in `renderer/src/format.ts`).
pub fn live_options(item: &Item) -> Vec<Choice> {
    if format::offer_is_live(item) { format::item_options(item) } else { Vec::new() }
}

pub fn view(s: &State, id: &str, product: &str) -> El {
    let (page_w, page_h) = s.page_size();
    let m = Measure::of(page_w);
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

    let mut summary = col().child(text(title.as_str()).font(TextStyle::new(1.7 * m.size, 500.0).lh(2.125 * m.size).tracking(-0.0255 * m.size)).color(ink()).wrap());
    if let Some(words) = &page.toggle {
        summary = summary.child(toggle(words, thread.clone(), &m).mt(8.0));
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
    // Under the summary: the newest answer alone, or the conversation with its outcome.
    let mut blocks = vec![markdown::block(0.0, 18.0, summary)];
    match page.alone {
        Some(answer) => blocks.extend(answer_alone(answer, &title, &m, &files)),
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
        blocks.push(markdown::block(14.0, 2.0, live_line(&line, &m)));
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
        .scroll(ScrollSpec { stick_bottom: unfolded || (working && page.alone.is_none()) })
        .child(el().pt(28.0).pl(m.indent).pr(32.0).pb(40.0 + under + whole).child(body.key("thread-said").w(m.column).selection_group(true)));
    col().w(page_w).h(page_h).child(reading).child(dock(s, item, &m))
}
