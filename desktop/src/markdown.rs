//! An agent's markdown, drawn as the web app's `.markdown` draws it: paragraphs, bold, italic,
//! struck text, inline code, links, bullet and numbered lists, headings, quotes, rules, code
//! blocks and tables (`renderer/src/styles.css` and `components/table-view.css`).
//!
//! A web page stacks blocks by their margins, and where two margins meet only the larger one
//! counts, also through a box with no edge of its own (a list around its items). Rune stacks
//! with no margins at all, so every block here carries its own and the stacking is done by hand.
//!
//! A file the words name is not left as words. The web window reads a message in three steps
//! and so does this: markdown's own pieces (`inlines`); then every path typed into a sentence
//! becomes a link or a picture (`name_files`, after `remarkArtifactPaths`); then what draws
//! moves under the sentence that names it, which keeps its name (`sentence`, after
//! `remarkMediaUnderSentences`). `files.rs` draws the pictures; a link is drawn here, as the
//! quiet mention the web window's `Mention.tsx` draws: an icon for its kind, then its name.

use std::iter::Peekable;

use pulldown_cmark::{Event, Options, Parser, Tag};
use regex::Regex;
use rune::{El, InlineIcon, Link, Span, TextCase, TextStyle, col, el, rich, row, text};

use crate::files::{self, Files, Shown};
use crate::icons;
use crate::rules::mentions::{self, Kind};
use crate::rules::paths::{self, Piece};
use crate::theme::*;

/// Inline code is 13 points whatever the text around it, and its box reaches 5 points past it.
const CODE: f32 = 13.0;
const CODE_PAD: f32 = 5.0;
/// Geist Mono's advance, in ems: what one padding space is sized by.
const MONO_ADVANCE: f32 = 0.6;

/// One piece of a line of words, as markdown hands it over.
#[derive(Clone, Debug, PartialEq)]
enum Inline {
    /// A line end the writer did not mean stays in the words as `\n`, which is how the rule
    /// for a folder on a line of its own tells where a line starts.
    Text(String),
    Code(String),
    Break,
    Strong(Vec<Inline>),
    Emphasis(Vec<Inline>),
    Strike(Vec<Inline>),
    /// `name_only`: what it points at is drawn under the sentence, which keeps its name.
    Link { url: String, kids: Vec<Inline>, name_only: bool },
    Image { url: String, alt: String },
}

/// A run of words with its styles: what one paragraph, heading or list line holds.
#[derive(Default)]
struct Run {
    text: String,
    spans: Vec<Span>,
    /// Where each mention begins, and its kind: its icon sits there.
    marks: Vec<(usize, Kind)>,
    /// What can be pressed, and what pressing it opens.
    presses: Vec<(usize, usize, String)>,
    strikes: Vec<(usize, usize)>,
    /// The spaces that stand for inline code's padding: drawn, never copied.
    pads: Vec<(usize, usize)>,
}

enum Node {
    /// Words with no paragraph around them: a list item written on one line.
    Line(Vec<Inline>),
    Paragraph(Vec<Inline>),
    Heading(u8, Vec<Inline>),
    Quote(Vec<Node>),
    Code(String),
    List(Option<u64>, Vec<Vec<Node>>),
    Table(Vec<String>, Vec<Vec<String>>),
    Rule,
    /// What a sentence names that is drawn under it: one picture large, or, `true`, several
    /// in a row. The last says whether a paragraph's own margins stand around it.
    Media(Vec<Shown>, bool, bool),
}

type Events<'a> = Peekable<Parser<'a>>;

fn is_block(tag: &Tag) -> bool {
    !matches!(tag, Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link { .. } | Tag::Image { .. })
}

/// The pieces up to the end of what holds them, or to a block that starts inside it. Words
/// markdown hands over in several parts are joined, as the web window's reader has them.
fn inlines(it: &mut Events) -> Vec<Inline> {
    fn say(out: &mut Vec<Inline>, words: &str) {
        match out.last_mut() {
            Some(Inline::Text(said)) => said.push_str(words),
            _ => out.push(Inline::Text(words.to_string())),
        }
    }
    let mut out = Vec::new();
    loop {
        match it.peek() {
            None | Some(Event::End(_)) => break,
            Some(Event::Start(tag)) if is_block(tag) => break,
            _ => {}
        }
        match it.next() {
            Some(Event::Start(tag)) => {
                let kids = inlines(it);
                // What the style holds has been read, so what comes next is its own end.
                if matches!(it.peek(), Some(Event::End(_))) {
                    it.next();
                }
                match tag {
                    Tag::Strong => out.push(Inline::Strong(kids)),
                    Tag::Emphasis => out.push(Inline::Emphasis(kids)),
                    Tag::Strikethrough => out.push(Inline::Strike(kids)),
                    Tag::Link { dest_url, .. } => out.push(Inline::Link { url: dest_url.to_string(), kids, name_only: false }),
                    Tag::Image { dest_url, .. } => out.push(Inline::Image { url: dest_url.to_string(), alt: plain(&kids) }),
                    _ => out.extend(kids),
                }
            }
            Some(Event::Code(code)) => out.push(Inline::Code(code.to_string())),
            Some(Event::Text(t) | Event::Html(t) | Event::InlineHtml(t) | Event::InlineMath(t) | Event::DisplayMath(t)) => say(&mut out, &t),
            Some(Event::SoftBreak) => say(&mut out, "\n"),
            Some(Event::HardBreak) => out.push(Inline::Break),
            Some(Event::TaskListMarker(done)) => say(&mut out, if done { "[x] " } else { "[ ] " }),
            Some(Event::End(_) | Event::FootnoteReference(_) | Event::Rule) | None => {}
        }
    }
    out
}

/// The words alone, with nothing of how they are drawn.
fn plain(kids: &[Inline]) -> String {
    let mut out = String::new();
    for kid in kids {
        match kid {
            Inline::Text(words) => out.push_str(&words.replace('\n', " ")),
            Inline::Code(code) => out.push_str(code),
            Inline::Break => out.push('\n'),
            Inline::Strong(kids) | Inline::Emphasis(kids) | Inline::Strike(kids) | Inline::Link { kids, .. } => out.push_str(&plain(kids)),
            Inline::Image { alt, .. } => out.push_str(alt),
        }
    }
    out
}

/// A path typed into a sentence becomes the file: a link to a document, a picture for a
/// picture (`linkArtifactPaths`). A link somebody wrote keeps its words unless they only spell
/// out its own path, and nothing inside a link or a code span is looked at.
fn name_files(kids: Vec<Inline>, f: &Files) -> Vec<Inline> {
    let mut out = Vec::new();
    for kid in kids {
        match kid {
            Inline::Text(words) => {
                let found = paths::pieces(&words, &f.dir, false, f.folder.as_deref());
                if found.is_empty() {
                    out.push(Inline::Text(words));
                    continue;
                }
                out.extend(found.into_iter().map(|piece| match piece {
                    Piece::Text(words) => Inline::Text(words),
                    Piece::Picture { url, alt } => Inline::Image { url, alt },
                    Piece::Link { url, words } => Inline::Link { url, kids: vec![Inline::Text(words)], name_only: false },
                }));
            }
            Inline::Link { url, mut kids, name_only } => {
                if let [Inline::Text(said)] = kids.as_mut_slice() {
                    *said = paths::link_words(&url, said).to_string();
                }
                out.push(Inline::Link { url, kids, name_only });
            }
            Inline::Strong(kids) => out.push(Inline::Strong(name_files(kids, f))),
            Inline::Emphasis(kids) => out.push(Inline::Emphasis(name_files(kids, f))),
            Inline::Strike(kids) => out.push(Inline::Strike(name_files(kids, f))),
            other => out.push(other),
        }
    }
    out
}

/// Where a piece points, when it is something that draws or plays (`drawnAt`).
fn drawn_at(kid: &Inline) -> Option<&str> {
    match kid {
        Inline::Image { url, .. } | Inline::Link { url, .. } => paths::is_drawn_file(url).then_some(url.as_str()),
        Inline::Code(code) => (paths::is_drawn_file(code) && !code.contains(char::is_whitespace)).then_some(code.as_str()),
        _ => None,
    }
}

/// What a piece that draws is in the row under its sentence: the picture each of the web
/// window's three components (`img`, `a`, `code`) would draw there. A film is played there
/// by the web window and only named by this one, so it is nothing here.
fn shown(kid: &Inline, f: &Files) -> Option<Shown> {
    match kid {
        Inline::Image { url, .. } | Inline::Link { url, .. } => paths::is_picture(url.split(['?', '#']).next().unwrap_or(url)).then(|| Shown::Picture(paths::normalize_uri(url))),
        Inline::Code(code) => {
            let written = code.trim();
            let path = paths::short_path(written, Some(&f.dir), f.folder.as_deref()).unwrap_or_else(|| written.to_string());
            (paths::is_code_path(&path) && crate::rules::roots::is_image_path(&path)).then(|| Shown::Picture(paths::normalize_uri(&path)))
        }
        _ => None,
    }
}

/// A sentence that names pictures stays whole: each stays in it as its name, and what draws
/// sits under it, one picture large or several in a row (`mediaUnderSentences`). `tight`: the
/// words are a list item's own line, with no paragraph around them.
fn sentence(kids: Vec<Inline>, f: &Files, tight: bool, out: &mut Vec<Node>) {
    let wrap = |kids| if tight { Node::Line(kids) } else { Node::Paragraph(kids) };
    let media: Vec<&Inline> = kids.iter().filter(|k| drawn_at(k).is_some()).collect();
    if media.is_empty() {
        return out.push(wrap(kids));
    }
    let pictures: Vec<Shown> = media.iter().filter_map(|k| shown(k, f)).collect();
    let films = media.iter().any(|k| drawn_at(k).is_some_and(paths::is_film));
    let says = |k: &Inline| match k {
        Inline::Text(words) => paths::says_something(words),
        Inline::Break => false,
        _ => true,
    };
    let has_words = kids.iter().any(|k| drawn_at(k).is_none() && says(k));
    let row = media.len() > 1 || films;
    if !has_words && !films && !pictures.is_empty() {
        // Several on a line of their own are a row with no sentence left over, and one alone
        // is left to draw large.
        return out.push(Node::Media(pictures, media.len() > 1, !tight));
    }
    if !has_words && pictures.is_empty() {
        // Nothing here draws in this window (a film, which the web window plays): its name.
        return out.push(wrap(kids));
    }
    let named = kids
        .into_iter()
        .map(|kid| {
            let Some(at) = drawn_at(&kid).map(str::to_string) else { return kid };
            let words = match kid {
                Inline::Link { kids, .. } => kids,
                Inline::Image { alt, .. } if !alt.is_empty() => vec![Inline::Text(alt)],
                _ => vec![Inline::Text(paths::base_name(&at).to_string())],
            };
            Inline::Link { url: at, kids: words, name_only: true }
        })
        .collect();
    out.push(wrap(named));
    if !pictures.is_empty() {
        out.push(Node::Media(pictures, row, !tight));
    }
}

/// The three steps after markdown's own, down through quotes and lists.
fn settle(nodes: Vec<Node>, f: &Files) -> Vec<Node> {
    let mut out = Vec::new();
    for node in nodes {
        match node {
            Node::Paragraph(kids) => sentence(name_files(kids, f), f, false, &mut out),
            Node::Line(kids) => sentence(name_files(kids, f), f, true, &mut out),
            Node::Heading(level, kids) => out.push(Node::Heading(level, name_files(kids, f))),
            Node::Quote(inside) => out.push(Node::Quote(settle(inside, f))),
            Node::List(start, items) => out.push(Node::List(start, items.into_iter().map(|item| settle(item, f)).collect())),
            other => out.push(other),
        }
    }
    out
}

/// One mention: the icon of its kind, then its words, softly underlined and pressable.
fn mention(r: &mut Run, href: &str, words: impl FnOnce(&mut Run)) {
    r.marks.push((r.text.len(), mentions::mention_kind(href)));
    let from = r.text.len();
    words(r);
    r.spans.push(Span::new(from, r.text.len()).underline(rule()));
    r.presses.push((from, r.text.len(), href.to_string()));
}

/// A line's pieces as one run of words with its styles.
fn flow(kids: &[Inline], f: &Files, r: &mut Run) {
    for kid in kids {
        let from = r.text.len();
        match kid {
            // A line end the writer did not mean is a space to the reader.
            Inline::Text(words) => r.text.push_str(&words.replace('\n', " ")),
            Inline::Break => r.text.push('\n'),
            Inline::Strong(kids) => {
                flow(kids, f, r);
                r.spans.push(Span::new(from, r.text.len()).weight(650.0));
            }
            Inline::Emphasis(kids) => {
                flow(kids, f, r);
                r.spans.push(Span::new(from, r.text.len()).italic());
            }
            Inline::Strike(kids) => {
                flow(kids, f, r);
                r.strikes.push((from, r.text.len()));
            }
            Inline::Code(code) => code_span(code, f, r),
            // What a link's own words become (`linkLabel`): a path or an address written out
            // is shortened to the name it goes by, and words the worker chose stay theirs.
            Inline::Link { url, kids, .. } => {
                let href = paths::normalize_uri(url);
                let label = match kids.as_slice() {
                    [] => Some(mentions::mention_name(&href)),
                    kids if kids.iter().all(|k| matches!(k, Inline::Text(_))) => mentions::mention_label(&href, Some(&plain(kids))),
                    _ => None,
                };
                mention(r, &href, |r| match label {
                    Some(name) => r.text.push_str(&name),
                    None => flow(kids, f, r),
                });
            }
            // A picture that stayed in its line (in a heading, inside bold) is its name there.
            Inline::Image { url, alt } => {
                let href = paths::normalize_uri(url);
                let name = if alt.is_empty() { mentions::mention_name(&href) } else { mentions::mention_label(&href, Some(alt)).unwrap_or_else(|| alt.clone()) };
                mention(r, &href, |r| r.text.push_str(&name));
            }
        }
    }
}

/// What a code span draws (`code` in `Focus.tsx`): a source file or a path to something the
/// window opens is a mention of that file, and anything else is code in its box.
fn code_span(code: &str, f: &Files, r: &mut Run) {
    if let Some(source) = mentions::source_reference(code) {
        r.marks.push((r.text.len(), Kind::Code));
        let from = r.text.len();
        r.text.push_str(&mentions::mention_name(code));
        r.spans.push(Span::new(from, r.text.len()).underline(rule()));
        r.presses.push((from, r.text.len(), source));
        return;
    }
    // Written out in full into the project's folder, or the thread's own, counts as the
    // short path it names, as a full path in a sentence does.
    let written = code.trim();
    let path = paths::short_path(written, Some(&f.dir), f.folder.as_deref()).unwrap_or_else(|| written.to_string());
    if paths::is_code_path(&path) {
        return mention(r, &path, |r| r.text.push_str(&mentions::mention_name(&path)));
    }
    // The box's padding is part of the line in a browser. Here it is a space before and
    // after, sized to that padding and never a place to break the line.
    let pad = Span::new(0, 0).mono().size(CODE_PAD / MONO_ADVANCE).bg(white(0.07), 0.0);
    let a = r.text.len();
    r.text.push('\u{a0}');
    let b = r.text.len();
    r.text.push_str(code);
    let c = r.text.len();
    r.text.push('\u{a0}');
    r.spans.push(Span { start: a as u32, end: b as u32, ..pad.clone() });
    r.spans.push(Span::new(b, c).mono().size(CODE).bg(white(0.07), 0.0));
    r.spans.push(Span { start: c as u32, end: r.text.len() as u32, ..pad });
    r.pads.extend([(a, b), (c, r.text.len())]);
}

fn nodes(it: &mut Events) -> Vec<Node> {
    let mut out = Vec::new();
    loop {
        let block = match it.peek() {
            None | Some(Event::End(_)) => return out,
            Some(Event::Rule) => {
                it.next();
                out.push(Node::Rule);
                continue;
            }
            Some(Event::Start(tag)) => is_block(tag),
            Some(_) => false,
        };
        if !block {
            out.push(Node::Line(inlines(it)));
            continue;
        }
        let Some(Event::Start(tag)) = it.next() else { return out };
        match tag {
            Tag::Paragraph => out.push(Node::Paragraph(inlines(it))),
            Tag::Heading { level, .. } => out.push(Node::Heading(level as u8, inlines(it))),
            Tag::BlockQuote(_) => out.push(Node::Quote(nodes(it))),
            Tag::CodeBlock(_) => {
                let mut code = String::new();
                while let Some(Event::Text(t)) = it.peek() {
                    code.push_str(t);
                    it.next();
                }
                out.push(Node::Code(code.trim_end_matches('\n').to_string()));
            }
            Tag::List(start) => {
                let mut items = Vec::new();
                while let Some(Event::Start(Tag::Item)) = it.peek() {
                    it.next();
                    items.push(nodes(it));
                    it.next();
                }
                out.push(Node::List(start, items));
            }
            Tag::Table(_) => {
                let cells = |it: &mut Events| {
                    let mut row = Vec::new();
                    while let Some(Event::Start(Tag::TableCell)) = it.peek() {
                        it.next();
                        row.push(plain(&inlines(it)).trim().to_string());
                        it.next();
                    }
                    row
                };
                let mut head = Vec::new();
                let mut rows = Vec::new();
                while let Some(Event::Start(part)) = it.peek() {
                    let heading = matches!(part, Tag::TableHead);
                    it.next();
                    let row = cells(it);
                    it.next();
                    if heading { head = row } else { rows.push(row) }
                }
                out.push(Node::Table(head, rows));
            }
            // Anything else (a block of HTML, a footnote, a definition list) is read as what it holds.
            _ => out.extend(nodes(it)),
        }
        // What a block holds has been read, so what comes next is its own end.
        if matches!(it.peek(), Some(Event::End(_))) {
            it.next();
        }
    }
}

/// One laid-out block and the margins it stacks by: its own, and those of what is inside it
/// that reach through its top and bottom.
pub struct Block {
    own: [f32; 2],
    through: [f32; 2],
    el: El,
}

impl Block {
    fn leaf(top: f32, bottom: f32, el: El) -> Block {
        Block { own: [top, bottom], through: [0.0, 0.0], el }
    }
    fn top(&self) -> f32 {
        self.own[0].max(self.through[0])
    }
    fn bottom(&self) -> f32 {
        self.own[1].max(self.through[1])
    }
}

/// Blocks stacked as a page stacks them. `trim` drops the first block's own top margin and the
/// last one's own bottom margin, as `.msg-body > :first-child` and `:last-child` do; what
/// reaches through from inside them still counts. Gives back the stack and the margins left
/// over its top and under its bottom, which belong to whatever it is stacked into.
pub fn stack(mut blocks: Vec<Block>, trim: bool) -> (El, f32, f32) {
    if trim {
        if let Some(first) = blocks.first_mut() {
            first.own[0] = 0.0;
        }
        if let Some(last) = blocks.last_mut() {
            last.own[1] = 0.0;
        }
    }
    let top = blocks.first().map_or(0.0, Block::top);
    let bottom = blocks.last().map_or(0.0, Block::bottom);
    let mut out = col();
    let mut above: Option<f32> = None;
    for b in blocks {
        let gap = above.map_or(0.0, |a| a.max(b.top()));
        above = Some(b.bottom());
        out = out.child(b.el.mt(gap));
    }
    (out, top, bottom)
}

/// The sizes one piece of markdown is drawn at: the page's reading size, the width it has,
/// and the colour of its words. And what the files it names are read against.
#[derive(Clone, Copy)]
struct Page<'a> {
    size: f32,
    width: f32,
    ink: rune::Color,
    /// How heavy its words are, and how tall a line is for their size: an agent's, or a
    /// person's (`.msg.yours .msg-body`).
    weight: f32,
    leading: f32,
    /// How deep in lists, for the bullet's shape.
    depth: usize,
    files: &'a Files<'a>,
}

impl Page<'_> {
    fn body(&self) -> TextStyle {
        TextStyle::new(self.size, self.weight).lh(self.line())
    }
    /// A line of reading text: 1.6 of its size, or 1.55 of a person's.
    fn line(&self) -> f32 {
        kept(self.leading * self.size)
    }
}

/// The icon a mention of each kind wears (`PATHS` in `Mention.tsx`).
fn icon_of(kind: Kind) -> &'static rune::Icon {
    match kind {
        Kind::Folder => &icons::MENTION_FOLDER,
        Kind::Page => &icons::MENTION_PAGE,
        Kind::Doc => &icons::MENTION_DOC,
        Kind::Image => &icons::MENTION_IMAGE,
        Kind::Code => &icons::MENTION_CODE,
        Kind::Media => &icons::MENTION_MEDIA,
        Kind::Link => &icons::LINK,
        Kind::File => &icons::MENTION_FILE,
    }
}

fn words(kids: &[Inline], font: TextStyle, p: Page) -> El {
    let mut r = Run::default();
    flow(kids, p.files, &mut r);
    let Run { text, spans, marks, presses, strikes, pads } = r;
    let mut e = readable(rich(text, spans).font(font).color(p.ink).wrap()).span_radius(6.0).strikes(strikes.into_iter().map(|(a, b)| (a, b, None)));
    for (a, b) in pads {
        e = e.uncopied(a..b);
    }
    for (at, kind) in marks {
        e = e.inline_icon(at, InlineIcon::glyph(icon_of(kind)).size(13.0).gap(4.0).color(muted()));
    }
    if presses.is_empty() {
        return e;
    }
    // Each mention is pressed like a link, and says where it points when the pointer rests on it.
    for (a, b, href) in presses {
        e = e.link(Link::new(a..b, href.clone()).help(paths::decode_uri(&href)));
    }
    e.on_link(|s: &mut crate::State, href: &str, _: &mut rune::Tx| files::open(s, href))
}

/// The folder a link points at, when it is one: an address that keeps its slash, which is how
/// a folder on a line of its own is told from a file (`a` in `Focus.tsx`).
fn folder_of(kid: &Inline) -> Option<&str> {
    let Inline::Link { url, .. } = kid else { return None };
    let elsewhere = ["http:", "https:", "data:", "file:", "mailto:"].iter().any(|scheme| url.to_ascii_lowercase().starts_with(scheme));
    (url.ends_with('/') && !elsewhere).then_some(url.as_str())
}

/// A paragraph's words. A folder named in it is a block of its own between the words before
/// it and the words after: its name over its newest pictures (`.folder-preview`, eight points
/// under the line above it and four over the line below).
fn lines(kids: &[Inline], font: TextStyle, p: Page) -> El {
    if !kids.iter().any(|k| folder_of(k).is_some()) {
        return words(kids, font, p);
    }
    let mut out = col();
    let mut any = false;
    let mut after_folder = false;
    let mut said: Vec<Inline> = Vec::new();
    let flush = |said: &mut Vec<Inline>, out: El, any: &mut bool, after_folder: bool| {
        // The line end that set the folder apart is not a word of the line before or after.
        if let Some(Inline::Text(first)) = said.first_mut() {
            *first = first.trim_start().to_string();
        }
        if let Some(Inline::Text(last)) = said.last_mut() {
            *last = last.trim_end().to_string();
        }
        let kept: Vec<Inline> = std::mem::take(said).into_iter().filter(|k| !matches!(k, Inline::Text(t) if t.is_empty())).collect();
        if kept.is_empty() {
            return out;
        }
        let gap = if after_folder { 4.0 } else { 0.0 };
        *any = true;
        out.child(words(&kept, font, p).mt(gap))
    };
    for kid in kids {
        match folder_of(kid) {
            Some(url) => {
                out = flush(&mut said, out, &mut any, after_folder);
                out = out.child(files::folder(p.files, &paths::normalize_uri(url), p.width, p.size).mt(if any { 8.0 } else { 0.0 }));
                (any, after_folder) = (true, true);
            }
            None => said.push(kid.clone()),
        }
    }
    flush(&mut said, out, &mut any, after_folder)
}

/// A list's marker hangs in the 22 points left of its item: a bullet, or the item's number.
fn marker(number: Option<u64>, p: Page) -> El {
    let em = p.size / 16.0;
    match number {
        // A browser sets a list's numbers in figures of one width.
        Some(n) => row().absolute(-60.0, 0.0).w(60.0).justify_end().child(text(format!("{n}.\u{a0}")).font(p.body().features("'tnum' 1")).color(p.ink)),
        // A disc, a ring one level in (a point wider, as the browser draws it), then a square.
        None => {
            let dot = el().absolute(-14.09 * em, 10.5 * em).size(5.5 * em, 5.5 * em);
            match p.depth {
                0 => dot.radius(2.75 * em).fill(p.ink),
                1 => el().absolute(-14.59 * em, 10.0 * em).size(6.5 * em, 6.5 * em).radius(3.25 * em).border(1.0, p.ink),
                _ => dot.fill(p.ink),
            }
        }
    }
}

fn list(start: Option<u64>, items: Vec<Vec<Node>>, p: Page) -> Block {
    let inner = Page { width: p.width - 22.0, depth: p.depth + 1, ..p };
    let laid: Vec<Block> = items
        .into_iter()
        .enumerate()
        .map(|(i, item)| {
            let (content, top, bottom) = stack(lay(item, inner), false);
            Block { own: [3.0, 3.0], through: [top, bottom], el: el().child(marker(start.map(|s| s + i as u64), p)).child(content.w_full()) }
        })
        .collect();
    let (items, top, bottom) = stack(laid, false);
    Block { own: [8.0, 8.0], through: [top, bottom], el: items.ml(22.0) }
}

fn code_block(code: &str, p: Page) -> Block {
    // The lines are the block's own, and the block's font is not the code's: it is the
    // system's monospace (Menlo, 0.928 em up and 0.236 down, each on a device pixel of a 2x
    // screen) at the reading size and line height. Geist Mono at 13 points sits on that line's
    // baseline, about a point lower than a line of its own would put it.
    let line = p.line();
    let (up, down) = ((0.928 * 2.0 * p.size).round() / 2.0, (0.236 * 2.0 * p.size).round() / 2.0);
    let drop = ((line - up - down) / 2.0 + up) - ((line - 1.3 * CODE) / 2.0 + 1.005 * CODE);
    let lines = readable(text(code).font(mono(CODE, line)).color(ink()));
    let block = el()
        .fill(rgba(40, 38, 37, 0.9))
        .border(1.0, edge())
        .border_layout()
        .radius(6.0)
        .clip_rounded(6.0)
        .child(row().scroll_x().child(el().pt(12.0 + drop).pb(12.0 - drop).px(14.0).child(lines)));
    // The margins are the block's inside a wrapper that has none, so they reach through it
    // even where a message drops its last block's own.
    Block { own: [0.0, 0.0], through: [10.0, 10.0], el: block }
}

fn number_like() -> Regex {
    Regex::new(r"(?i)^[-+]?[$€£]?\s?\d[\d,]*(\.\d+)?\s?(%|k|m|bn|b|t|tn)?$").expect("a fixed pattern")
}
fn nothing_like() -> Regex {
    Regex::new(r"(?i)^(n/?a|none|null|nil|tbd|unknown|-|\x{2013}|\x{2014}|\?)$").expect("a fixed pattern")
}

/// A table in a message (`TableView` with `inline`): as wide as the message, every cell
/// wrapping, the columns sharing the width by how much each holds (`inlineTracks`), and a
/// column of numbers set to the right.
fn table(head: &[String], rows: &[Vec<String>], p: Page) -> Block {
    let cell_font = TextStyle::new(14.0, 400.0).lh(22.0);
    let wide = |s: &str| rune::text::measure(s, &[], TextStyle::new(14.0, 400.0), None).0;
    let (number, nothing) = (number_like(), nothing_like());
    let columns = head.len().max(rows.iter().map(Vec::len).max().unwrap_or(0));
    fn cell(r: &[String], c: usize) -> &str {
        r.get(c).map(String::as_str).unwrap_or("")
    }
    // Each column's share of the width, and the least it may be squeezed to.
    let mut share = Vec::new();
    let mut least = Vec::new();
    let mut numeric = Vec::new();
    for c in 0..columns {
        let name = head.get(c).map(String::as_str).unwrap_or("");
        let longest = rows.iter().map(|r| cell(r, c).chars().count()).max().unwrap_or(0);
        let widest = rows.iter().map(|r| cell(r, c)).filter(|v| v.chars().count() as f32 >= longest as f32 * 0.7).map(|v| wide(v).min(1200.0)).fold(0.0, f32::max);
        let word = name.split_whitespace().map(|w| w.chars().count()).max().unwrap_or(0);
        let heading = name.chars().count().min(word.max(18)) as f32 * 7.6 + 14.0;
        share.push((heading.max(widest) + 34.0).ceil().round());
        let widest_cell = rows.iter().map(|r| wide(cell(r, c)) + 34.0).fold(0.0, f32::max);
        let widest_word = rows.iter().flat_map(|r| cell(r, c).split_whitespace()).map(|w| wide(w) + 34.0).fold(0.0, f32::max);
        least.push(widest_cell.min(140.0).max(widest_word).max(word as f32 * 8.0 + 34.0).ceil());
        let filled: Vec<&str> = rows.iter().map(|r| cell(r, c)).filter(|v| !v.is_empty() && !nothing.is_match(v)).collect();
        numeric.push(c > 0 && !filled.is_empty() && filled.iter().filter(|v| number.is_match(v)).count() as f32 >= 0.9 * filled.len() as f32);
    }
    // `minmax(least, share fr)`: a column that would fall under its least takes it, and the
    // rest share what is left.
    let mut widths = vec![0.0f32; columns];
    let mut fixed = vec![false; columns];
    loop {
        let free = p.width - (0..columns).filter(|c| fixed[*c]).map(|c| least[c]).sum::<f32>();
        let parts: f32 = (0..columns).filter(|c| !fixed[*c]).map(|c| share[c]).sum();
        let mut again = false;
        for c in 0..columns {
            widths[c] = if fixed[c] { least[c] } else { free.max(0.0) * share[c] / parts.max(1.0) };
            if !fixed[c] && widths[c] < least[c] {
                fixed[c] = true;
                again = true;
            }
        }
        if !again {
            break;
        }
    }
    let line = white(0.09);
    // The first column stays put while the rest scrolls sideways, so it paints the ground under it.
    let grounded = |c: usize, e: El| if c == 0 { e.fill(rgb(26, 24, 23)).gradient(rgba(36, 34, 33, 0.55), rgba(36, 34, 33, 0.55)) } else { e };
    let sides = |c: usize, top: f32| [top, 1.0, 1.0, if c == 0 { 1.0 } else { 0.0 }];
    let heads = row().children((0..columns).map(|c| {
        let name = text(head.get(c).cloned().unwrap_or_default()).font(mono(10.5, 15.0).tracking(0.945)).text_case(TextCase::Uppercase).color(muted()).truncate();
        let e = grounded(c, row().w(widths[c]).items_end().px(16.0).py(11.0).border_sides(sides(c, 1.0), line).border_layout());
        (if numeric[c] { e.justify_end() } else { e }).child(name)
    }));
    let body = rows.iter().map(|r| {
        row().items_stretch().children((0..columns).map(|c| {
            let font = if numeric[c] { cell_font.features("'tnum' 1") } else { cell_font };
            let e = grounded(c, row().w(widths[c]).min_h(47.0).items_start().px(16.0).py(12.0).border_sides(sides(c, 0.0), line).border_layout());
            (if numeric[c] { e.justify_end() } else { e }).child(readable(text(cell(r, c)).font(font).color(p.ink).wrap()))
        }))
    });
    Block::leaf(18.0, 18.0, row().scroll_x().child(col().child(heads).children(body)))
}

fn lay(nodes: Vec<Node>, p: Page) -> Vec<Block> {
    nodes
        .into_iter()
        .map(|n| match n {
            Node::Line(r) => Block::leaf(0.0, 0.0, lines(&r, p.body(), p)),
            Node::Paragraph(r) => Block::leaf(8.0, 8.0, lines(&r, p.body(), p)),
            // One picture under its sentence is a paragraph of its own (`.inline-image`), and
            // several are a row with its own room around it (`.media-row`).
            Node::Media(shown, false, margins) => {
                let Some(Shown::Picture(src)) = shown.first() else { return Block::leaf(0.0, 0.0, el()) };
                let around = if margins { 8.0 } else { 0.0 };
                Block::leaf(around, around, files::picture(p.files, src, p.width, p.size, p.line()))
            }
            Node::Media(shown, true, _) => Block::leaf(2.0, 18.0, files::media_row(p.files, &shown, p.width)),
            // A first-level heading is the browser's own: twice the size and bold. The second
            // and third are the app's small labels. Deeper ones are bold at the reading size.
            Node::Heading(1, r) => Block::leaf(0.0, 0.0, words(&r, TextStyle::new(2.0 * p.size, 700.0).lh(kept(3.2 * p.size)), p)),
            Node::Heading(2 | 3, r) => Block::leaf(24.0, 7.0, words(&r, TextStyle::new(12.0, 600.0).lh(kept(19.2)).tracking(0.84), Page { ink: muted(), ..p }).text_case(TextCase::Uppercase)),
            Node::Heading(_, r) => Block::leaf(0.0, 0.0, words(&r, TextStyle::new(p.size, 700.0).lh(p.line()), p)),
            Node::Quote(inside) => {
                let (content, top, bottom) = stack(lay(inside, Page { width: p.width - 15.0, ink: dim(), ..p }), false);
                Block { own: [8.0, 8.0], through: [top, bottom], el: content.pl(12.0).border_sides([0.0, 0.0, 0.0, 3.0], edge()).border_layout() }
            }
            Node::Code(code) => code_block(&code, p),
            Node::List(start, items) => list(start, items, p),
            Node::Table(head, rows) => table(&head, &rows, p),
            // The browser's own rule, which the app leaves as it is: grey above and near white below.
            Node::Rule => Block::leaf(0.0, 0.0, col().child(el().h(1.0).fill(rgb(154, 154, 154))).child(el().h(1.0).fill(rgb(238, 238, 238)))),
        })
        .collect()
}

/// A piece of markdown as blocks, `size` being the page's reading size and `width` the room
/// it has, and `files` what the files it names are read against. Stack them with [`stack`].
pub fn blocks(source: &str, size: f32, width: f32, files: &Files) -> Vec<Block> {
    let mut events = Parser::new_ext(source, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH).peekable();
    lay(settle(nodes(&mut events), files), Page { size, width, ink: ink(), weight: 400.0, leading: 1.6, depth: 0, files })
}

/// The same for a line the app wrote into a person's own message (`rules/typed.rs`), in the
/// type of the words around it, heavier and on shorter lines than an agent's
/// (`.msg.yours .msg-body`, which the web window's `.typed-embed` inherits).
pub fn blocks_typed(source: &str, size: f32, width: f32, files: &Files) -> Vec<Block> {
    let mut events = Parser::new_ext(source, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH).peekable();
    lay(settle(nodes(&mut events), files), Page { size, width, ink: ink(), weight: 500.0, leading: 1.55, depth: 0, files })
}

/// One block that is not markdown, to stack among those that are.
pub fn block(top: f32, bottom: f32, el: El) -> Block {
    Block::leaf(top, bottom, el)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Snapshot;
    use crate::pictures::Pictures;

    const DIR: &str = "/Users/you/Work/accounts/acct/storefront";

    /// A line's pieces as words: a name that stands for what is drawn under the sentence in
    /// square brackets, any other link as its words and where it points, code in backticks.
    fn said(kids: &[Inline]) -> String {
        kids.iter()
            .map(|kid| match kid {
                Inline::Text(words) => words.clone(),
                Inline::Code(code) => format!("`{code}`"),
                Inline::Break => "\n".to_string(),
                Inline::Strong(kids) | Inline::Emphasis(kids) | Inline::Strike(kids) => said(kids),
                Inline::Link { url, kids, name_only: true } => format!("[{} -> {url}]", plain(kids)),
                Inline::Link { url, kids, .. } => format!("<{} -> {url}>", plain(kids)),
                Inline::Image { url, .. } => format!("<picture {url}>"),
            })
            .collect()
    }

    fn shape(nodes: &[Node]) -> Vec<String> {
        nodes
            .iter()
            .map(|node| match node {
                Node::Paragraph(kids) | Node::Line(kids) => said(kids),
                Node::Media(shown, row, _) => format!("{}: {}", if *row { "row" } else { "large" }, shown.iter().map(|Shown::Picture(src)| src.as_str()).collect::<Vec<_>>().join(" ")),
                Node::List(_, items) => format!("list of {}", items.iter().map(|item| shape(item).join(" / ")).collect::<Vec<_>>().join(" | ")),
                Node::Code(_) => "code".to_string(),
                _ => "other".to_string(),
            })
            .collect()
    }

    /// What a message's words become before they are drawn.
    fn read(source: &str) -> Vec<String> {
        let (snap, pictures) = (Snapshot::default(), Pictures::new(2.0, true, true));
        let files = Files::invented(DIR, &snap, &pictures);
        let mut events = Parser::new_ext(source, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH).peekable();
        shape(&settle(nodes(&mut events), &files))
    }

    // tests/media-sit-in-a-row-under-the-whole-sentence.test.mts. A film is played in the row by
    // the web window and only named by this one, so here a row holds the pictures alone.
    #[test]
    fn a_sentence_naming_several_files_stays_whole_over_what_it_draws() {
        // A PDF does not draw, so it stays the code span it was, which is drawn as its name.
        assert_eq!(
            read("I made `chart.png`, `report.pdf` and `clip.mp4` in your working folder."),
            ["I made [chart.png -> chart.png], `report.pdf` and [clip.mp4 -> clip.mp4] in your working folder.", "row: chart.png"]
        );
        // A markdown link and a picture in a sentence move the same way, and keep their words.
        assert_eq!(read("See [the shot](a.png) and ![it](b.webp) for detail."), ["See [the shot -> a.png] and [it -> b.webp] for detail.", "row: a.png b.webp"]);
        // Two pictures on a line of their own become a row with no sentence left over.
        assert_eq!(read("![a](a.png) ![b](b.png)"), ["row: a.png b.png"]);
        // A sentence in a list keeps what it draws in the list.
        assert_eq!(read("- The cart is `a.png` now."), ["list of The cart is [a.png -> a.png] now. / large: a.png"]);
        assert_eq!(read("- The clip `a.mp4` is short."), ["list of The clip [a.mp4 -> a.mp4] is short."]);
    }

    #[test]
    fn what_a_row_must_not_change() {
        // One picture alone on its line is left to draw large.
        assert_eq!(read("![a](a.png)"), ["large: a.png"]);
        // A sentence with no media is left alone, and so is a web address that ends like a picture.
        assert_eq!(read("Read `src/app.ts` and [the docs](https://x.test/a.png)."), ["Read `src/app.ts` and <the docs -> https://x.test/a.png>."]);
        assert_eq!(read("See [this](https://x.test/a.png) now.").len(), 1);
        // Code that only looks like a file, in a fence, is not touched.
        assert_eq!(read("```\nclip.mp4\n```"), ["code"]);
        // A voice note stays where it is, since it is a message of its own.
        assert_eq!(read("Listen to [note](attachments/voice-note-1200.webm) now.").len(), 1);
    }

    // tests/one-picture-named-in-a-sentence-is-drawn-large-under-it.test.mts
    #[test]
    fn one_picture_named_in_a_sentence_is_drawn_large_under_it() {
        assert_eq!(
            read("Needs you: what waits on you. ![1-needs-you.png](designs/a/1-needs-you.png)"),
            ["Needs you: what waits on you. [1-needs-you.png -> designs/a/1-needs-you.png]", "large: designs/a/1-needs-you.png"]
        );
        // The same for a path in code and a markdown link.
        assert_eq!(read("See `shot.png` for it.")[1], "large: shot.png");
        assert_eq!(read("See [the shot](shot.png) for it.")[1], "large: shot.png");
        // Two in one sentence stay a row, and so does a picture beside a film.
        assert_eq!(read("Compare ![a](a.png) with ![b](b.png) here.")[1], "row: a.png b.png");
        assert_eq!(read("See `a.png` and `b.mp4` now.")[1], "row: a.png");
    }

    #[test]
    fn a_path_typed_into_a_sentence_is_the_file() {
        // Bare in prose: a picture is drawn under its sentence and a page is a link by its name.
        assert_eq!(read("The total is in designs/w-1/total-line.png now."), ["The total is in [designs/w-1/total-line.png -> designs/w-1/total-line.png] now.", "large: designs/w-1/total-line.png"]);
        assert_eq!(read("The page is pages/checkout.html today."), ["The page is <checkout.html -> pages/checkout.html> today."]);
        // Written out in full into the project, it is the short path; outside it, it stays words.
        assert_eq!(read(&format!("It is {DIR}/designs/w-1/tax-line.png here."))[1], "large: designs/w-1/tax-line.png");
        assert_eq!(read("It wrote /tmp/scratch/out.png and left."), ["It wrote /tmp/scratch/out.png and left."]);
        // A name with a space in it reaches the disk as written and the page as an address.
        assert_eq!(read("The old one, \"receipt old.png\", is gone.")[1], "large: receipt%20old.png");
        // A folder on a line of its own, under the words that lead to it, is a door.
        assert_eq!(read(&format!("They are in:\n{DIR}/designs/rounds/")), [format!("They are in:\n<{DIR}/designs/rounds/ -> designs/rounds/>")]);
        // A link that spells out its own path reads as the file's name.
        assert_eq!(read("See [notes/rounding.md](notes/rounding.md)."), ["See <rounding.md -> notes/rounding.md>."]);
    }
}
