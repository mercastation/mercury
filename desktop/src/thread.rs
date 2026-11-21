//! One conversation, to read: its name, what was asked, what the agent answered and what was
//! said since, with the reply box at the foot. Built from the web app's own measurements of
//! its thread page (`renderer/src/components/ItemThread.tsx`, `Thread.tsx`,
//! `OneThingAtATime.tsx` and their stylesheets).
//!
//! The page opens the way the web app's does: on the newest answer alone, with the rest of the
//! conversation folded behind one line under the title. Pressing that line unfolds it.

use std::collections::HashSet;

use rune::time::LocalTime;
use rune::{El, Icon, ScrollSpec, TextCase, TextStyle, col, el, icon, row, spacer, text};
use serde_json::{Value, json};

use crate::data::Item;
use crate::markdown::{self, Block};
use crate::talk::{self, Answers, Field, Line, Message, Said, Step, Talk};
use crate::theme::*;
use crate::{State, View, chrome, icons};

/// What the person has unfolded while reading: the threads whose earlier messages are shown.
#[derive(Default)]
pub struct Reading {
    unfolded: HashSet<String>,
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
                let to = View::Thread { id: r.id.clone(), product: r.product.clone() };
                b.on_press(move |s: &mut State| s.go(to.clone()))
            }
            None => b.opacity(0.35),
        }
    };
    let crumbs = mono(11.0, 14.0).tracking(0.99);
    let title = item(s, id, product).map(name).unwrap_or_default();
    let finished = item(s, id, product).is_some_and(|i| i.status == "done");
    let close = row().h(30.0).items_center().gap(7.0).pl(9.0).pr(12.0).radius(6.0).border(1.0, edge()).border_layout();
    let close = if finished { close.child(label("Reopen", 13.0, 400.0, dim())) } else { close.child(icon(&icons::DONE, 16.0).color(dim())).child(label("Done", 13.0, 400.0, dim())) };
    row().absolute(chrome::CARD[0], chrome::CARD[1])
        .size(card_w, 56.0)
        .items_center()
        .pl(20.0)
        .pr(28.0)
        .border_sides([0.0, 0.0, 1.0, 0.0], edge())
        .border_layout()
        .child(square(&icons::INBOX, 15.0, 26.0, ink()).mr(10.0).on_press(|s: &mut State| s.go(View::Inbox)))
        // Back is where the person came from, and a thread opened from nowhere came from the Inbox.
        .child(square(&icons::BACK, 15.0, 26.0, ink()).ml(2.0).on_press(|s: &mut State| if s.history.is_empty() { s.go(View::Inbox) } else { s.back() }))
        .child(square(&icons::FORWARD, 15.0, 26.0, muted()).ml(2.0).opacity(0.45))
        .child(
            row().grow(1.0)
                .min_w(0.0)
                .h(28.0)
                .ml(12.0)
                .items_center()
                .child(text("Inbox").font(crumbs).text_case(TextCase::Uppercase).color(muted()).mr(8.0).on_press(|s: &mut State| s.go(View::Inbox)))
                .child(text("/").font(crumbs).color(muted()).ml(5.0).mr(7.0))
                .child(row().min_w(0.0).px(6.0).child(text(crumb(&title)).font(crumbs).text_case(TextCase::Uppercase).color(dim()).truncate())),
        )
        .child(
            row().ml(16.0)
                .items_center()
                .child(step(&icons::UP, at.and_then(|a| a.checked_sub(1)).and_then(|a| s.lists.needs.get(a))))
                .child(step(&icons::DOWN, at.and_then(|a| s.lists.needs.get(a + 1))))
                .child(close.ml(10.0))
                .child(square(&icons::MORE, 16.0, 30.0, dim()).ml(10.0).border(1.0, edge())),
        )
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

/// One message, and how much of what it holds reaches under it.
fn message(msg: &Message, turn: (bool, bool), now: f64, m: &Measure) -> El {
    let (body, leak) = if msg.mine {
        (typed(&msg.text, m), 0.0)
    } else {
        let (body, _, bottom) = markdown::stack(markdown::blocks(&msg.text, m.size, m.column), true);
        (body, bottom)
    };
    let mut e = col().child(said_by(msg, now)).child(body.mt(3.0));
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

/// The conversation: each thing said or done under the last, 22 points apart, with lines of
/// what happened drawn closer to what they follow (`.thread` and `.thread-block`).
fn conversation(said: &[Said], now: f64, m: &Measure) -> El {
    let turns = talk::turns(said);
    let mut out = col();
    let mut above: Option<bool> = None;
    for (e, turn) in said.iter().zip(turns) {
        let (block, is_step) = match e {
            Said::Message(msg) => (message(msg, turn, now, m), false),
            Said::Step(s) => (step(s), true),
        };
        let gap = match (above, is_step) {
            (None, _) => 0.0,
            (Some(true), true) => 5.0,
            (Some(false), true) => 8.0,
            (Some(_), false) => 22.0,
        };
        above = Some(is_step);
        out = out.child(block.mt(gap));
    }
    // The page's own foot mark is one more thing in the stack, so the gap comes after the last too.
    out.pb(22.0)
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
            }),
    )
}

/// The agent's last word, whole, at the foot of the conversation.
fn outcome(answer: &Message, plain: bool, m: &Measure) -> Vec<Block> {
    let mut blocks = markdown::blocks(&answer.text, m.size, m.column);
    if !plain {
        // A thread still in flight says which of the two this is.
        let words = if answer.field == Field::Result { "Result" } else { "Latest checkpoint" };
        blocks.insert(0, markdown::block(0.0, 7.0, text(words).font(TextStyle::new(12.5, 400.0).lh(20.0)).color(muted())));
    }
    blocks
}

/// The newest answer alone, as a page that leads with it: a first line that is bold and
/// nothing else is its headline, unless it only repeats the thread's name.
fn answer_alone(answer: &Message, title: &str, m: &Measure) -> Vec<Block> {
    let same = |a: &str, b: &str| a.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase() == b.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    let (headline, rest) = talk::headline(&answer.text);
    let mut blocks = markdown::blocks(rest, m.size, m.column);
    if let Some(words) = headline.filter(|h| !same(h, title)) {
        let size = kept(1.05 * m.size);
        blocks.insert(0, markdown::block(0.0, 10.0, text(words).font(TextStyle::new(size, 600.0).lh(kept(1.45 * size))).color(ink()).wrap()));
    }
    blocks
}

/// What the engine answered about this thread: asked through the door when the window is live,
/// carried by a fixture for a picture. With nothing to read yet, the row itself still says
/// what was asked and what came back.
fn ledger(s: &State, item: &Item) -> Vec<Line> {
    let thread = key(&item.product, &item.id);
    let answered = s.threads.get(&thread).cloned().or_else(|| s.fixture.get("threads").and_then(|t| t.get(&thread)).cloned());
    if let Some(answers) = answered.and_then(|v| serde_json::from_value::<Answers>(v).ok()) {
        return answers.history.lines;
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
    lines
}

/// Whether a fixture's picture was taken with this thread's earlier messages unfolded.
fn pictured_unfolded(s: &State, thread: &str) -> bool {
    s.fixture.get("unfolded").and_then(Value::as_array).is_some_and(|list| list.iter().any(|k| k.as_str() == Some(thread)))
}

/// The reply box at the foot. Phase 1 only draws it.
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

pub fn view(s: &State, id: &str, product: &str) -> El {
    let (page_w, page_h) = s.page_size();
    let m = Measure::of(page_w);
    let Some(item) = item(s, id, product) else {
        return row().w(page_w).pt(36.0).pl(m.indent).child(label("This conversation is not here any more.", 14.0, 400.0, muted()));
    };
    let thread = key(product, id);
    let unfolded = s.reading.unfolded.contains(&thread) != pictured_unfolded(s, &thread);
    let working = s.snap.supervisor.running.iter().any(|r| r.item_id == id && r.product == product);
    let imported = item.labels.iter().any(|l| l == "thread-import" || l.starts_with("codex:"));
    let title = name(item);
    let talk = Talk::read(&ledger(s, item));
    let page = talk.page(working, unfolded, imported);

    let mut summary = col().child(text(title.as_str()).font(TextStyle::new(1.7 * m.size, 500.0).lh(2.125 * m.size).tracking(-0.0255 * m.size)).color(ink()).wrap());
    if let Some(words) = &page.toggle {
        summary = summary.child(toggle(words, thread, &m).mt(8.0));
    }
    // Under the summary: the newest answer alone, or the conversation with its outcome.
    let mut blocks = vec![markdown::block(0.0, 18.0, summary)];
    match page.alone {
        Some(answer) => blocks.extend(answer_alone(answer, &title, &m)),
        None => {
            if page.said.is_empty() && talk.outcome.is_none() {
                blocks.push(markdown::block(0.0, 0.0, el().py(8.0).child(label("Nothing has been said here yet.", 14.0, 400.0, muted()))));
            } else {
                blocks.push(markdown::block(0.0, 0.0, conversation(page.said, s.now, &m)));
            }
            if let Some(answer) = &talk.outcome {
                // The agent's own close leads with its result as it is; anything else is labelled.
                let closed_by_agent = item.status == "done" && item.rest.get("wrote").and_then(|w| w.get("status")).and_then(|st| st.get("source")).and_then(Value::as_str) != Some("founder");
                let (body, top, bottom) = markdown::stack(outcome(answer, answer.field == Field::Result && closed_by_agent, &m), false);
                blocks.push(markdown::block(top.max(28.0), bottom.max(8.0), body));
            }
        }
    }
    // What the person did after the answer, under it and in order.
    if !talk.after.is_empty() {
        blocks.push(markdown::block(22.0, 0.0, col().gap(9.0).children(talk.after.iter().map(step))));
    }
    let (body, _, under) = markdown::stack(blocks, false);
    // A browser scrolls a page that is a whole number of points tall, whatever its lines add
    // up to, so the foot of a long conversation rests on a whole point. Once the words have
    // been laid out, the room under them takes up the difference.
    let whole = rune::memo::layout_size("thread-said").map_or(0.0, |[_, tall]| {
        let page = 28.0 + tall + 40.0 + under;
        page.round() - page
    });
    // Unfolding lands on the newest thing said, at the foot; a folded page opens at its top.
    let reading = col()
        .key(if unfolded { "thread-whole" } else { "thread-newest" })
        .grow(1.0)
        .min_h(0.0)
        .scroll(ScrollSpec { stick_bottom: unfolded })
        .child(el().pt(28.0).pl(m.indent).pr(32.0).pb(40.0 + under + whole).child(body.key("thread-said").w(m.column).selection_group(true)));
    col().w(page_w).h(page_h).child(reading).child(el().pl(m.indent).pr(32.0).pb(24.0).child(reply_box(&m)))
}
