//! An agent's markdown, drawn as the web app's `.markdown` draws it: paragraphs, bold, italic,
//! struck text, inline code, links, bullet and numbered lists, headings, quotes, rules, code
//! blocks and tables (`renderer/src/styles.css` and `components/table-view.css`).
//!
//! A web page stacks blocks by their margins, and where two margins meet only the larger one
//! counts, also through a box with no edge of its own (a list around its items). Rune stacks
//! with no margins at all, so every block here carries its own and the stacking is done by hand.

use std::iter::Peekable;

use pulldown_cmark::{Event, Options, Parser, Tag};
use regex::Regex;
use rune::{El, InlineIcon, Span, TextCase, TextStyle, col, el, rich, row, text};

use crate::icons;
use crate::theme::*;

/// Inline code is 13 points whatever the text around it, and its box reaches 5 points past it.
const CODE: f32 = 13.0;
const CODE_PAD: f32 = 5.0;
/// Geist Mono's advance, in ems: what one padding space is sized by.
const MONO_ADVANCE: f32 = 0.6;

/// A run of words with its styles: what one paragraph, heading or list line holds.
#[derive(Default)]
struct Run {
    text: String,
    spans: Vec<Span>,
    /// Where each link begins: its icon sits there.
    links: Vec<usize>,
    strikes: Vec<(usize, usize)>,
    /// The spaces that stand for inline code's padding: drawn, never copied.
    pads: Vec<(usize, usize)>,
}

enum Node {
    /// Words with no paragraph around them: a list item written on one line.
    Line(Run),
    Paragraph(Run),
    Heading(u8, Run),
    Quote(Vec<Node>),
    Code(String),
    List(Option<u64>, Vec<Vec<Node>>),
    Table(Vec<String>, Vec<Vec<String>>),
    Rule,
}

type Events<'a> = Peekable<Parser<'a>>;

fn is_block(tag: &Tag) -> bool {
    !matches!(tag, Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link { .. } | Tag::Image { .. })
}

/// The words up to the end of the block they are in, or to a block that starts inside it.
fn run(it: &mut Events) -> Run {
    let mut r = Run::default();
    // Each open style, and where in the text it began.
    let mut open: Vec<(Tag, usize)> = Vec::new();
    loop {
        match it.peek() {
            None => break,
            Some(Event::End(_)) if open.is_empty() => break,
            Some(Event::Start(tag)) if is_block(tag) => break,
            _ => {}
        }
        match it.next() {
            Some(Event::Start(tag)) => {
                if matches!(tag, Tag::Link { .. }) {
                    r.links.push(r.text.len());
                }
                open.push((tag, r.text.len()));
            }
            Some(Event::End(_)) => {
                let Some((tag, from)) = open.pop() else { break };
                let to = r.text.len();
                match tag {
                    Tag::Strong => r.spans.push(Span::new(from, to).weight(650.0)),
                    Tag::Emphasis => r.spans.push(Span::new(from, to).italic()),
                    Tag::Strikethrough => r.strikes.push((from, to)),
                    Tag::Link { .. } => r.spans.push(Span::new(from, to).underline(rule())),
                    _ => {}
                }
            }
            Some(Event::Code(code)) => {
                // The box's padding is part of the line in a browser. Here it is a space before
                // and after, sized to that padding and never a place to break the line.
                let pad = Span::new(0, 0).mono().size(CODE_PAD / MONO_ADVANCE).bg(white(0.07), 0.0);
                let a = r.text.len();
                r.text.push('\u{a0}');
                let b = r.text.len();
                r.text.push_str(&code);
                let c = r.text.len();
                r.text.push('\u{a0}');
                r.spans.push(Span { start: a as u32, end: b as u32, ..pad.clone() });
                r.spans.push(Span::new(b, c).mono().size(CODE).bg(white(0.07), 0.0));
                r.spans.push(Span { start: c as u32, end: r.text.len() as u32, ..pad });
                r.pads.extend([(a, b), (c, r.text.len())]);
            }
            Some(Event::Text(t) | Event::Html(t) | Event::InlineHtml(t) | Event::InlineMath(t) | Event::DisplayMath(t)) => r.text.push_str(&t),
            Some(Event::SoftBreak) => r.text.push(' '),
            Some(Event::HardBreak) => r.text.push('\n'),
            Some(Event::TaskListMarker(done)) => r.text.push_str(if done { "[x] " } else { "[ ] " }),
            Some(Event::FootnoteReference(_) | Event::Rule) | None => {}
        }
    }
    r
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
            out.push(Node::Line(run(it)));
            continue;
        }
        let Some(Event::Start(tag)) = it.next() else { return out };
        match tag {
            Tag::Paragraph => out.push(Node::Paragraph(run(it))),
            Tag::Heading { level, .. } => out.push(Node::Heading(level as u8, run(it))),
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
                        row.push(run(it).text.replace('\u{a0}', "").trim().to_string());
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
/// and the colour of its words.
#[derive(Clone, Copy)]
struct Page {
    size: f32,
    width: f32,
    ink: rune::Color,
    /// How deep in lists, for the bullet's shape.
    depth: usize,
}

impl Page {
    fn body(&self) -> TextStyle {
        TextStyle::new(self.size, 400.0).lh(self.line())
    }
    /// A line of reading text: 1.6 of its size.
    fn line(&self) -> f32 {
        kept(1.6 * self.size)
    }
}

fn words(r: Run, font: TextStyle, ink: rune::Color) -> El {
    let Run { text, spans, links, strikes, pads } = r;
    let mut e = readable(rich(text, spans).font(font).color(ink).wrap()).span_radius(6.0).strikes(strikes.into_iter().map(|(a, b)| (a, b, None)));
    for (a, b) in pads {
        e = e.uncopied(a..b);
    }
    for at in links {
        e = e.inline_icon(at, InlineIcon::glyph(&icons::LINK).size(13.0).gap(4.0).color(muted()));
    }
    e
}

/// A list's marker hangs in the 22 points left of its item: a bullet, or the item's number.
fn marker(number: Option<u64>, p: Page) -> El {
    let em = p.size / 16.0;
    match number {
        // A browser sets a list's numbers in figures of one width.
        Some(n) => row().absolute(-60.0, 0.0).w(60.0).justify_end().child(text(format!("{n}.\u{a0}")).font(p.body().features("'tnum' 1")).color(p.ink)),
        None => {
            let dot = el().absolute(-14.09 * em, 10.5 * em).size(5.5 * em, 5.5 * em);
            match p.depth {
                0 => dot.radius(2.75 * em).fill(p.ink),
                1 => dot.radius(2.75 * em).border(1.0, p.ink),
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
    Block::leaf(10.0, 10.0, block)
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
            Node::Line(r) => Block::leaf(0.0, 0.0, words(r, p.body(), p.ink)),
            Node::Paragraph(r) => Block::leaf(8.0, 8.0, words(r, p.body(), p.ink)),
            // A first-level heading is the browser's own: twice the size and bold. The second
            // and third are the app's small labels. Deeper ones are bold at the reading size.
            Node::Heading(1, r) => Block::leaf(0.0, 0.0, words(r, TextStyle::new(2.0 * p.size, 700.0).lh(kept(3.2 * p.size)), p.ink)),
            Node::Heading(2 | 3, r) => Block::leaf(24.0, 7.0, words(r, TextStyle::new(12.0, 600.0).lh(kept(19.2)).tracking(0.84), muted()).text_case(TextCase::Uppercase)),
            Node::Heading(_, r) => Block::leaf(0.0, 0.0, words(r, TextStyle::new(p.size, 700.0).lh(p.line()), p.ink)),
            Node::Quote(inside) => {
                let (content, top, bottom) = stack(lay(inside, Page { width: p.width - 15.0, ink: dim(), ..p }), false);
                Block { own: [8.0, 8.0], through: [top, bottom], el: content.pl(12.0).border_sides([0.0, 0.0, 0.0, 3.0], edge()).border_layout() }
            }
            Node::Code(code) => code_block(&code, p),
            Node::List(start, items) => list(start, items, p),
            Node::Table(head, rows) => table(&head, &rows, p),
            Node::Rule => Block::leaf(0.0, 0.0, el().h(1.0).fill(edge())),
        })
        .collect()
}

/// A piece of markdown as blocks, `size` being the page's reading size and `width` the room
/// it has. Stack them with [`stack`].
pub fn blocks(source: &str, size: f32, width: f32) -> Vec<Block> {
    let mut events = Parser::new_ext(source, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH).peekable();
    lay(nodes(&mut events), Page { size, width, ink: ink(), depth: 0 })
}

/// One block that is not markdown, to stack among those that are.
pub fn block(top: f32, bottom: f32, el: El) -> Block {
    Block::leaf(top, bottom, el)
}
