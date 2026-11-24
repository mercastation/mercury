//! The rail's usage row and the card of the agents' limits it opens, as the web window's sidebar
//! draws them (`UsagePill` in `renderer/src/components/UsagePill.tsx`, its look `.usage-panel` in
//! `renderer/src/styles.css` and `.workspace-usage` in `workspace-navigation.css`). What it says
//! is `rules/usage.rs`.
//!
//! The row names the agent used last and its headline figure. A pointer resting on it opens the
//! card over it: the card waits 120 ms after the pointer came onto the row, so a pointer crossing
//! the rail's foot does not flash it, and stays 220 ms after it left, so one slipping off the
//! edge does not snap it shut. On the folded rail nothing opens: a pointer that landed there
//! waits for its first move once the rail has opened, so the card never opens into a rail still
//! widening, and the rail folding closes it at once. The card takes no pointer, so it never
//! stands in the way of where the pointer was going. A press on the row opens that agent's page
//! in Settings.

use std::time::Duration;

use rune::{El, TextStyle, Tx, col, el, row, text};

use crate::rules::usage::{self, Row, Usage};
use crate::theme::*;
use crate::{State, settings};

/// How long the pointer rests on the row before the card opens, and how long it lingers after
/// the pointer left (`setTimeout(…, 120)`, `LINGER_MS`).
const OPENS_AFTER: f64 = 0.12;
const LINGERS: f64 = 0.22;

/// The card: open or not, and what is due and when, on the window's clock.
#[derive(Default)]
pub struct Card {
    pub open: bool,
    due: Option<(f64, bool)>,
}

impl Card {
    /// The pointer came onto the row (`enter`): on an open rail the card opens after its wait;
    /// a pending close is let go either way.
    fn enter(&mut self, at: f64, rail_open: bool, tx: &mut Tx) {
        self.due = None;
        if !rail_open {
            return;
        }
        self.due = Some((at + OPENS_AFTER, true));
        tx.wake_after(Duration::from_secs_f64(OPENS_AFTER));
    }
    /// It left (`leave`): the card goes after its linger, and an opening still waiting does not come.
    fn leave(&mut self, at: f64, tx: &mut Tx) {
        self.due = Some((at + LINGERS, false));
        tx.wake_after(Duration::from_secs_f64(LINGERS));
    }
    /// Shut, with nothing due.
    fn shut(&mut self) {
        self.open = false;
        self.due = None;
    }
}

/// The poll's part: what was due lands, and a rail that has folded closes the card at once
/// (`useEffect` on `folded`).
pub fn tend(s: &mut State, at: f64, tx: &mut Tx) -> bool {
    if !s.rail.is_open() {
        let was = s.usage_card.open || s.usage_card.due.is_some();
        s.usage_card.shut();
        return was;
    }
    match s.usage_card.due {
        Some((due, to)) if at >= due => {
            s.usage_card.due = None;
            let changed = s.usage_card.open != to;
            s.usage_card.open = to;
            changed
        }
        Some((due, _)) => {
            tx.wake_after(Duration::from_secs_f64((due - at).max(0.0)));
            false
        }
        None => false,
    }
}

/// What the bar answers: the pointer resting on it opens the card, leaving it closes the card
/// after a linger, and a press opens the agent's page in Settings (`press` in `UsagePill`).
pub fn answering(row: El, engine: &str) -> El {
    let page = usage::settings_page(engine);
    row.on_hover_change(|s: &mut State, inside, tx| {
        let (at, open) = (tx.clock(), s.rail.is_open());
        if inside { s.usage_card.enter(at, open, tx) } else { s.usage_card.leave(at, tx) }
    })
    // A pointer that landed on the folded rail opens the card on its first move once the rail
    // has opened (`move`).
    .on_pointer_move_local(|s: &mut State, _, tx| {
        if s.rail.is_open() && !s.usage_card.open && s.usage_card.due.is_none() {
            s.usage_card.enter(tx.clock(), true, tx);
        } else {
            tx.live_only();
        }
    })
    .on_press_tx(move |s: &mut State, tx| {
        s.usage_card.shut();
        settings::open(s, page, tx);
    })
}

/// The faint ink the card's headings and times are in (`--agent-ink`).
fn faint() -> rune::Color {
    rgb(127, 121, 117)
}

/// A heading (`.usage-panel h3`): the agent's name in small capitals, and beside the one used
/// last, when there are several, "Last used" (`h3 small`).
///
/// The card stands in the sidebar, whose type it inherits: every line of it is 20 points tall
/// and at 500 since the sidebar was redrawn (`.gt-rail-nav.workspace-navigation` in
/// `renderer/src/sidebar.css`: `font-weight: 500; line-height: 20px`).
fn heading(word: &str, last: bool) -> El {
    let name = text(word).font(TextStyle::new(11.0, 600.0).lh(20.0).tracking(0.88)).color(faint()).text_case(rune::TextCase::Uppercase);
    let line = row().h(20.0).items_end().child(pen(name, 11.0));
    if !last {
        return line;
    }
    // The tag sits on the name's own baseline, its smaller line half a point taller than the
    // heading's with it in it.
    let tag = text("Last used").font(TextStyle::new(10.5, 500.0).lh(13.5).tracking(0.88)).color(muted()).text_case(rune::TextCase::Uppercase);
    line.h(20.5).child(pen(tag, 10.5).ml(8.0).mb(3.5))
}

/// One limit (`.usage-lim`): its name and how much is spent, the bar of it, and when it starts
/// over. A window that started over since says Reset, with no bar and no time.
fn limit(r: &Row) -> El {
    let top = row()
        .h(20.0)
        .justify_between()
        .child(pen(text(r.name.as_str()).font(TextStyle::new(12.5, 500.0).lh(20.0)).color(dim()), 12.5))
        .child(pen(text(usage::used_words(r)).font(TextStyle::new(12.5, 550.0).lh(20.0).features("'tnum' 1")).color(ink()), 12.5));
    let mut lim = col().child(top);
    if let Some(used) = r.used {
        // The track is the app's wash and the fill the faint ink, square in the sidebar
        // (`.usage-meter.wide`, 5 points tall).
        let fill = (used.clamp(0.0, 100.0) / 100.0) as f32 * 238.0;
        lim = lim.child(el().mt(5.0).h(5.0).w(238.0).fill(white(0.04)).child(el().size(fill, 5.0).fill(muted())));
        if let Some(when) = &r.when {
            lim = lim.child(pen(text(when.as_str()).font(TextStyle::new(11.5, 500.0).lh(20.0).features("'tnum' 1")).color(faint()), 11.5).mt(4.0));
        }
    }
    lim
}

/// When a reading was taken, under its agent's limits (`.usage-read`, `.usage-panel-foot`).
fn read_line(at: f64, now: f64) -> El {
    pen(text(usage::read_words(at, now)).font(TextStyle::new(11.5, 500.0).lh(20.0)).color(faint()), 11.5)
}

/// The limits under one heading, eleven points apart (`.usage-lim`'s margin).
fn limits(rows: &[Row]) -> El {
    col().gap(11.0).children(rows.iter().map(limit))
}

/// The card over the page, hung six points over the sidebar's last row from the sidebar's left
/// edge, 268 points wide (`.gt-rail-nav .workspace-usage .usage-panel` in `sidebar.css`:
/// `bottom: calc(100% + 6px); left: 0`).
/// One agent: its name, its limits, and under a hairline when it was read. Several: the one used
/// last tagged, its limits and when it was read, then each other under a hairline of its own.
pub fn layer(s: &State) -> Option<El> {
    if !s.usage_card.open || !s.rail.is_open() || s.settings.open {
        return None;
    }
    let usage = Usage::of(&s.snap, s.now)?;
    let now = s.now;
    let joint = usage.joint();
    let mut card = col()
        .pin(Some(crate::chrome::RAIL_LEFT), None, None, Some(22.0 + last_row(s) + 6.0))
        .w(268.0)
        .pt(13.0)
        .px(14.0)
        .pb(12.0)
        .fill(surface())
        .border(1.0, rule())
        .border_layout()
        .box_shadow(shadow(0.0, 24.0, 70.0, 0.0, rgba(0, 0, 0, 0.55), false))
        .child(heading(usage.word(), joint).mb(11.0))
        .child(limits(&usage::rows(&usage.last.limits, now)));
    if joint {
        card = card.child(read_line(usage.last.at, now).mt(11.0));
        for other in usage.agents.iter().skip(1) {
            let section = col()
                .mt(14.0)
                .pt(12.0)
                .border_sides([1.0, 0.0, 0.0, 0.0], edge())
                .border_layout()
                .child(heading(crate::rules::engine::engine_label(&other.engine), false).mb(11.0))
                .child(limits(&usage::rows(&other.limits, now)))
                .child(read_line(other.at, now).mt(11.0));
            card = card.child(section);
        }
    } else {
        card = card.child(el().mt(12.0).pt(10.0).border_sides([1.0, 0.0, 0.0, 0.0], edge()).border_layout().child(read_line(usage.last.at, now)));
    }
    Some(card)
}

/// How tall the sidebar's last row is, the one the bar stands at the end of: the person's own,
/// 32 points, when they are signed in, and else Settings, 30 (`.workspace-usage`, and
/// `.workspace-bottom:has(> .th-me) .workspace-usage`).
fn last_row(s: &State) -> f32 {
    if crate::rules::window::my_letters(&s.snap).is_some() { 32.0 } else { 30.0 }
}

#[cfg(test)]
mod tests {
    use rune::{Cursor, Runtime};
    use serde_json::{Value, json};

    use crate::State;
    use crate::feel::tests::{press, run, window_dressed};
    use crate::rules::settings::{PageId, Pane};

    /// The invented inbox with two agents read this week, Claude Code's just now; the rail
    /// kept open when `kept_open`. Nobody is signed in, so its bar stands at the end of
    /// Settings, the open sidebar's last row: 196 to 228 points across, 713 to 733 down.
    fn read(name: &str, kept_open: bool) -> Runtime<State> {
        window_dressed(name, &[], "/fixtures/storefront", "Done.", |fixture| {
            let now = fixture["now"].as_f64().unwrap();
            let limit = |span: &str, name: &str, percent: f64, ahead: f64| json!({ "span": span, "qualifier": null, "name": name, "percent": percent, "resetsAt": now + ahead, "resetsText": null, "resetsOn": null, "zone": null });
            let claude = json!({ "engine": "claude", "at": now - 30_000.0, "limits": [limit("session", "This session", 4.0, 7_200_000.0), limit("week", "This week", 39.0, 200_000_000.0)] });
            let codex = json!({ "engine": "codex", "at": now - 93_600_000.0, "limits": [limit("week", "This week", 12.0, 500_000_000.0)] });
            fixture["snapshot"]["usage"] = claude.clone();
            fixture["snapshot"]["usageByEngine"] = json!([claude, codex]);
            if kept_open {
                fixture["picks"] = json!({ "rail_shrinks": false });
            }
        })
    }

    fn drawn(rt: &Runtime<State>, words: &str) -> bool {
        rt.elements().iter().any(|e| e.text.as_deref() == Some(words))
    }

    #[test]
    fn the_card_opens_on_the_first_move_once_the_rail_is_open_and_lingers_after_the_pointer_leaves() {
        let mut rt = read("usage-card", false);
        // Folded, the sidebar draws no bar (`.workspace-collapsed … .workspace-usage`): the
        // pointer on its last row meets Settings alone, and the sidebar opens 140 ms later.
        rt.pointer_move(24.0, 719.0);
        rt.frame(1.0 / 60.0);
        rt.pointer_move(28.0, 723.0);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.cursor(), Cursor::Default);
        run(&mut rt, 0.5);
        assert!(rt.state.rail.is_open());
        assert!(!rt.state.usage_card.open, "no card until the pointer is on the bar");
        // Along the row onto the bar at its end: the card comes 120 ms after.
        rt.pointer_move(212.0, 723.0);
        run(&mut rt, 0.1);
        assert!(!rt.state.usage_card.open, "it waits 120 ms");
        run(&mut rt, 0.05);
        assert!(rt.state.usage_card.open, "and comes");
        let said: Vec<String> = rt.elements().iter().filter_map(|e| e.text.clone()).collect();
        assert!(drawn(&rt, "Last used") && drawn(&rt, "Read just now") && drawn(&rt, "Read 1d ago"), "both agents, the one used last tagged: {said:?}");
        assert!(drawn(&rt, "4% used") && drawn(&rt, "12% used"));
        // Up the rail, off the bar: it stays 220 ms, then goes.
        rt.pointer_move(28.0, 200.0);
        run(&mut rt, 0.18);
        assert!(rt.state.usage_card.open, "it lingers");
        run(&mut rt, 0.08);
        assert!(!rt.state.usage_card.open, "then goes");
        // Back on the bar and off the rail altogether: the rail folds, and the card with it.
        rt.pointer_move(212.0, 723.0);
        run(&mut rt, 0.2);
        assert!(rt.state.usage_card.open);
        rt.pointer_move(700.0, 400.0);
        rt.frame(1.0 / 60.0);
        rt.frame(1.0 / 60.0);
        assert!(!rt.state.usage_card.open, "the rail folding closes it at once");
        assert!(!drawn(&rt, "Last used"));
    }

    #[test]
    fn on_a_rail_kept_open_the_card_comes_120_ms_after_the_pointer_and_a_press_opens_the_agents_page() {
        let mut rt = read("usage-kept-open", true);
        run(&mut rt, 0.1);
        assert!(rt.state.rail.is_open(), "the rail stands open");
        rt.pointer_move(212.0, 723.0);
        run(&mut rt, 0.1);
        assert!(!rt.state.usage_card.open);
        run(&mut rt, 0.05);
        assert!(rt.state.usage_card.open);
        // The card names the agent used last; a press on the bar opens its page in Settings.
        assert!(drawn(&rt, "Claude Code"));
        press(&mut rt, 212.0, 723.0);
        assert!(rt.state.settings.open);
        assert_eq!(rt.state.settings.pane, Pane::Page(PageId::Claude));
        assert!(!rt.state.usage_card.open);
    }

    #[test]
    fn a_pointer_only_passing_over_the_row_opens_nothing() {
        let mut rt = read("usage-passing", true);
        run(&mut rt, 0.1);
        rt.pointer_move(212.0, 723.0);
        run(&mut rt, 0.05);
        rt.pointer_move(700.0, 723.0);
        run(&mut rt, 0.4);
        assert!(!rt.state.usage_card.open);
        // And none at all without a reading.
        let mut none = window_dressed("usage-none", &[], "/fixtures/storefront", "Done.", |f: &mut Value| f["picks"] = json!({ "rail_shrinks": false }));
        run(&mut none, 0.1);
        none.pointer_move(212.0, 723.0);
        run(&mut none, 0.3);
        assert!(!none.state.usage_card.open);
    }
}
