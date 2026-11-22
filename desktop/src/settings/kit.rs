//! What every page of Settings is made of, read off the web window's computed styles
//! (`settings.css`, and `styles.css` under it): the title, a group's heading over its card of
//! rows, a row's label and sentence with its control at the right edge, and the controls
//! themselves, which are the New thread card's (a chip, a switch in the Send orange, a stepper
//! of two chips).

use std::rc::Rc;

use rune::{Cursor, Drag, El, Phase, ScrollSpec, Span, TextStyle, Tx, col, el, icon, row, text};

use super::marks;
use crate::State;
use crate::feel;
use crate::theme::*;

/// `--bg-hover`, `--bg-selected` and the menus' ground (`--pop` under its edge).
pub fn hover_ground() -> rune::Color {
    rgb(46, 44, 43)
}
pub fn selected() -> rune::Color {
    rgb(55, 53, 52)
}

/// How wide a run of words is in a face.
pub fn wide(words: &str, font: TextStyle) -> f32 {
    rune::text::measure(words, &[], font, None).0
}

/// Words a hand can select (`.set-inner :is(.set-title, .set-lede, .set-row-label, …)`), in a
/// box no wider than they are, so the I-beam stands over them and the arrow past their end as
/// it does over Chromium's line boxes; a sentence longer than its column still wraps in it.
pub fn selectable(words: El) -> El {
    row().min_w(0.0).child(readable(words).shrink())
}

/* --------------------------------- the page --------------------------------- */

/// A page's title (`.set-title`): 24 points, its words the person can select.
pub fn title(words: &str) -> El {
    selectable(pen(text(words).font(sans(24.0, 400.0).tracking(-0.84)).color(ink()), 24.0))
}

/// The sentence under a title (`.set-lede`).
pub fn lede(words: &str) -> El {
    col().mt(6.0).child(selectable(pen(text(words).font(TextStyle::new(13.5, 400.0).lh(kept(20.925))).color(muted()).wrap(), 13.5)))
}

/// A page with nothing on it yet but its one line (`Reading settings…`).
pub fn lede_alone(words: &str) -> El {
    selectable(pen(text(words).font(TextStyle::new(13.5, 400.0).lh(kept(20.925))).color(muted()), 13.5))
}

/// A refused read or write, over the page (`.set-error`).
pub fn error(words: &str) -> El {
    col().mb(12.0)
        .py(9.0)
        .px(14.0)
        .radius(6.0)
        .fill(rgb(36, 34, 33))
        .border(1.0, rule())
        .border_layout()
        .child(pen(text(words).font(TextStyle::new(12.5, 400.0).lh(kept(18.75))).color(dim()).wrap(), 12.5))
}

/// A heading over one card of rows (`Group`): `find` is what the search lands on, `action`
/// stands at the heading's right (`.set-group-head`), `warn` is the card's first line, and a
/// `bare` group draws its body without a card.
pub struct Group {
    pub find: Option<&'static str>,
    pub label: Option<El>,
    pub action: Option<El>,
    pub warn: Option<String>,
    pub bare: bool,
    pub body: Vec<El>,
}

impl Group {
    pub fn new(label: &str, body: Vec<El>) -> Group {
        Group { find: None, label: Some(heading(label)), action: None, warn: None, bare: false, body }
    }
    pub fn unnamed(body: Vec<El>) -> Group {
        Group { find: None, label: None, action: None, warn: None, bare: false, body }
    }
    pub fn find(mut self, id: &'static str) -> Group {
        self.find = Some(id);
        self
    }
    pub fn action(mut self, action: El) -> Group {
        self.action = Some(action);
        self
    }
    pub fn bare(mut self) -> Group {
        self.bare = true;
        self
    }
    /// Forty points under what stands above it (`.set-group`).
    pub fn el(self) -> El {
        let mut group = col().mt(40.0);
        if let Some(id) = self.find {
            group = group.key(format!("find-{id}"));
        }
        match (self.label, self.action) {
            (Some(label), Some(action)) => {
                group = group.child(row().pb(12.0).gap(16.0).items_center().justify_between().child(label).child(action));
            }
            (Some(label), None) => group = group.child(el().pb(12.0).child(label)),
            _ => {}
        }
        if self.bare {
            return group.children(self.body);
        }
        group.child(plate(self.warn, self.body))
    }
}

/// A group's heading (`.set-group-label`).
pub fn heading(words: &str) -> El {
    pen(text(words).font(sans(14.0, 500.0).tracking(-0.07)).color(ink()), 14.0)
}

/// The card of a group (`.set-plate`): a hairline round it, a wash inside, its rows divided by
/// the same hairline, and a warning about the whole group as its first line.
pub fn plate(warn: Option<String>, rows: Vec<El>) -> El {
    let mut lines: Vec<El> = Vec::new();
    if let Some(warn) = warn {
        lines.push(col().py(13.0).child(pen(text(warn).font(TextStyle::new(12.5, 400.0).lh(18.75)).color(dim()).wrap(), 12.5)));
    }
    lines.extend(rows);
    let lines = lines.into_iter().enumerate().map(|(at, line)| if at > 0 { line.border_sides([1.0, 0.0, 0.0, 0.0], edge()).border_layout() } else { line });
    col().px(18.0).radius(10.0).fill(white(0.04)).border(1.0, edge()).border_layout().children(lines)
}

/// What a row says under its label: a sentence, or words with some of them set heavier.
pub enum Desc {
    Plain(String),
    /// The words, and the byte ranges set at 700 (`<b>`).
    Bold(String, Vec<(usize, usize)>),
}

impl From<&str> for Desc {
    fn from(s: &str) -> Desc {
        Desc::Plain(s.to_string())
    }
}
impl From<String> for Desc {
    fn from(s: String) -> Desc {
        Desc::Plain(s)
    }
}

/// A row's label (`.set-row-label`).
pub fn label_words(words: &str) -> El {
    selectable(pen(text(words).font(sans(13.5, 500.0).tracking(-0.0405)).color(ink()).wrap(), 13.5))
}

/// A row's sentence (`.set-row-desc`).
pub fn desc_words(desc: Desc) -> El {
    let font = TextStyle::new(12.5, 400.0).lh(18.75);
    let words = match desc {
        Desc::Plain(words) => text(words).font(font),
        Desc::Bold(words, bold) => text(words).font(font).spans(bold.into_iter().map(|(a, b)| Span::new(a, b).weight(700.0)).collect::<Vec<Span>>()),
    };
    selectable(pen(words.color(muted()).wrap(), 12.5))
}

/// One row (`.set-row`): its label and sentence, and its controls at the right edge.
pub fn line(label: &str, desc: Option<Desc>, controls: Vec<El>) -> El {
    let mut words = col().grow(1.0).min_w(0.0).child(label_words(label));
    if let Some(desc) = desc {
        words = words.child(col().mt(3.0).child(desc_words(desc)));
    }
    let mut r = row().py(14.0).gap(24.0).items_center().child(words);
    if !controls.is_empty() {
        r = r.child(row().items_center().gap(9.0).children(controls));
    }
    r
}

/* -------------------------------- the controls -------------------------------- */

/// A button of the page (`.set-ghost`): the composer's chip, which under the pointer takes the
/// text's colour and a stronger edge. A disabled one keeps its faint words and the arrow.
pub fn ghost(word: &str, enabled: bool) -> El {
    let tone = if enabled { dim() } else { muted() };
    let b = row().h(28.0).px(11.0).items_center().radius(6.0).fill(white(0.04)).border(1.0, edge()).border_layout().child(pen(text(word).font(TextStyle::new(13.0, 400.0).lh(13.0)).color(tone).group_color(ink()), 13.0));
    if enabled { b.hover_group().cursor(Cursor::Pointer).child(feel::edge_lit(6.0, rule(), true)) } else { feel::answers(b) }
}

/// A switch (`Switch`, `.set-sw`): a hairline track with a dark knob off; on, the text's ink with
/// a knob of the ground, as every other switch in the app (the accent is only ever a small dot).
/// It flips on the press and shows what was saved once the answer lands.
pub fn switch(on: bool, flip: impl Fn(&mut State, bool, &mut Tx) + 'static) -> El {
    let track = el().size(30.0, 18.0).radius(9.0).cursor(Cursor::Pointer);
    let track = if on { track.fill(ink()).border(1.0, ink()) } else { track.fill(white(0.08)).border(1.0, rule()) };
    let knob = el().absolute(if on { 15.0 } else { 3.0 }, 3.0).size(12.0, 12.0).radius(6.0).fill(if on { page() } else { rgb(40, 38, 37) });
    track.child(knob).on_press_tx(move |s: &mut State, tx| flip(s, !on, tx))
}

/// A picker's menu: each option as `(value, label)`.
pub type Options = Vec<(String, String)>;

/// A one-line picker (`Picker`): the value it is on and the list behind it. `above` is the word
/// over a picker whose row does not already name it; a `long` one scrolls after five rows.
pub struct Picker {
    pub key: String,
    pub above: Option<String>,
    pub value: String,
    pub options: Options,
    pub long: bool,
    pub help: Option<String>,
    pub pick: Rc<dyn Fn(&mut State, String, &mut Tx)>,
}

/// The chip of a picker (`.set-picker-now`), with its menu hung under it while it is open.
pub fn picker(s: &State, p: Picker) -> El {
    let open = s.settings.menu.as_deref() == Some(p.key.as_str());
    let now = p.options.iter().find(|(v, _)| *v == p.value).map_or(p.value.clone(), |(_, l)| l.clone());
    let chip_font = sans(13.0, 400.0);
    let chip_w = wide(&now, chip_font) + 8.0 + 9.0 + 20.0 + 2.0;
    let key = p.key.clone();
    let mut chip = row()
        .key(format!("picker-{}", p.key))
        .h(28.0)
        .px(10.0)
        .gap(8.0)
        .items_center()
        .radius(6.0)
        .fill(white(0.04))
        .border(1.0, if open { rule() } else { edge() })
        .border_layout()
        .cursor(Cursor::Pointer)
        .hover_group()
        .child(pen(text(now).font(chip_font).color(ink()), 13.0))
        .child(icon(&marks::CHEVRON, 9.0).color(muted()))
        .child(feel::edge_lit(6.0, rule(), true))
        .on_press(move |s: &mut State| {
            s.settings.menu = if s.settings.menu.as_deref() == Some(key.as_str()) { None } else { Some(key.clone()) };
        });
    if let Some(help) = &p.help {
        chip = chip.help(help.clone());
    }
    s.settings.pickers_drawn(&p.key);
    let mut wrap = el().child(chip);
    if open {
        wrap = wrap.child(menu(&p, chip_w));
    }
    match p.above {
        Some(above) => col().gap(7.0).child(pen(text(above).font(sans(11.5, 400.0)).color(muted()), 11.5)).child(row().child(wrap)),
        None => wrap,
    }
}

/// A picker's menu (`.set-picker-menu`): under its chip and six points clear of it, its right
/// edge on the chip's, at least as wide as the chip, painted over the rows after it and seen
/// only inside the page, which it scrolls with.
fn menu(p: &Picker, chip_w: f32) -> El {
    let row_font = |on: bool| sans(13.0, if on { 600.0 } else { 400.0 });
    let widest = p.options.iter().map(|(v, l)| wide(l, row_font(*v == p.value))).fold(0.0, f32::max);
    let menu_w = if p.long { 240.0 } else { (widest + 18.0 + 8.0 + 2.0).max(chip_w) };
    let rows = p.options.iter().map(|(value, label)| {
        let on = *value == p.value;
        let pick = p.pick.clone();
        let value = value.clone();
        let words = pen(text(label.as_str()).font(row_font(on)).color(if on { ink() } else { dim() }).group_color(ink()), 13.0);
        let words = if p.long { words.truncate() } else { words };
        let r = row().h(29.0).px(9.0).items_center().radius(6.0).cursor(Cursor::Pointer).hover_group().child(words);
        let r = if on { r.fill(selected()) } else { r.hover(white(0.07)) };
        let r = if p.long { r.help(label.clone()) } else { r };
        r.on_press_tx(move |s: &mut State, tx| {
            s.settings.menu = None;
            pick(s, value.clone(), tx);
        })
    });
    let list = col().w(menu_w - 2.0).px(4.0).py(4.0).gap(1.0).children(rows);
    let shell = col()
        .absolute(chip_w - menu_w, 34.0)
        .w(menu_w)
        .radius(10.0)
        .fill(rgb(54, 52, 49))
        .border(1.0, rule())
        .border_layout()
        .box_shadow(shadow(0.0, 24.0, 70.0, 0.0, rgba(0, 0, 0, 0.55), false))
        .raise();
    if p.long {
        // Five rows show and the rest scroll (`.set-picker-long`).
        return shell.h(30.0 * 5.0 + 8.0 + 2.0).child(col().grow(1.0).min_h(0.0).scroll(ScrollSpec { stick_bottom: false }).child(list));
    }
    shell.child(list)
}

/// The layer under an open picker: a press anywhere else closes it on the way down, and one on
/// another picker's chip opens that one, as the web window's `mousedown` and `click` would.
pub fn away(s: &State) -> El {
    let chips: Vec<(String, [f32; 4])> = s.settings.drawn_pickers().into_iter().filter_map(|key| rune::memo::window_layout(&format!("picker-{key}")).map(|rect| (key, rect))).collect();
    el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).on_drag(move |s: &mut State, d: &mut Drag, _tx: &mut Tx| {
        if d.phase != Phase::Down {
            return;
        }
        let [x, y] = d.window_point();
        let was = s.settings.menu.take();
        if let Some((key, _)) = chips.iter().find(|(key, [cx, cy, cw, ch])| Some(key) != was.as_ref() && x >= *cx && x <= cx + cw && y >= *cy && y <= cy + ch) {
            s.settings.menu = Some(key.clone());
        }
    })
}

/// The stepper (`Stepper`, `.set-step`): two chips with the number between them.
pub fn stepper(value: f64, min: f64, max: f64, set: impl Fn(&mut State, f64) + 'static + Clone) -> El {
    let sign = |mark: &'static rune::Icon, live: bool, to: f64, set: Box<dyn Fn(&mut State, f64)>| {
        let b = row().size(28.0, 28.0).center().radius(6.0).fill(white(0.04)).border(1.0, edge()).border_layout().child(icon(mark, 12.0).color(if live { dim() } else { rule() }).group_color(ink()));
        if live { b.hover_group().cursor(Cursor::Pointer).child(feel::edge_lit(6.0, rule(), true)).on_press(move |s: &mut State| set(s, to)) } else { feel::answers(b) }
    };
    let (less, more) = (set.clone(), set);
    row()
        .gap(2.0)
        .items_center()
        .child(sign(&marks::MINUS, value > min, value - 1.0, Box::new(less)))
        .child(row().size(32.0, 28.0).center().child(pen(text(format!("{value}")).font(TextStyle::new(13.5, 500.0).lh(28.0)).color(ink()), 13.5)))
        .child(sign(&marks::PLUS, value < max, value + 1.0, Box::new(more)))
}

/// The small capitals of a label, in the accent or faint (`.set-badge`).
pub fn badge(words: &str, lit: bool) -> El {
    pen(text(words).font(TextStyle::new(10.5, 500.0).mono().lh(13.5).tracking(0.63)).text_case(rune::TextCase::Uppercase).color(if lit { accent() } else { muted() }), 10.5)
}

/// A line of the page's own words, quiet (`.cloud-plans-say`).
pub fn say(words: &str) -> El {
    col().mt(14.0).child(selectable(pen(text(words).font(TextStyle::new(12.5, 400.0).lh(20.0)).color(muted()).wrap(), 12.5)))
}

impl super::Screen {
    /// The pickers drawn this frame, so a press on one of them, while another's menu is open,
    /// opens it (`away`).
    pub fn pickers_drawn(&self, key: &str) {
        self.drawn.borrow_mut().push(key.to_string());
    }
    pub fn drawn_pickers(&self) -> Vec<String> {
        std::mem::take(&mut *self.drawn.borrow_mut())
    }
}
