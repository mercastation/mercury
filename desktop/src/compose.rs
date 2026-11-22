//! The box a thread is started in: open on Home's page, under the greeting. Built from the web
//! app's own measurements of its New thread card (`renderer/src/threads/ThreadComposer.tsx`,
//! `thread-composer.css` and Home's `home.css`); what it sends, to which project and on which
//! model is `rules/compose.rs`.
//!
//! The words are the person's to type and ⌘Return or Send sends them. Nothing waits three
//! seconds here, unlike a reply or a close: a thread held back would be in no list at all
//! while the person looks for it. It is written at once, a line says where it is going and
//! then where it went, and Z takes it back by closing it, which stops its agent.

use rune::time::LocalTime;
use rune::{El, TextEdit, TextStyle, Tx, col, el, row, spacer, text_input};
use serde_json::Value;

use crate::data::{Item, Product};
use crate::rules::compose::{self as rules, Engines, Level, Pick};
use crate::rules::{title, window};
use crate::theme::*;
use crate::{State, View, home, write};

/// The key of the field the words are typed in.
pub const FIELD: &str = "compose";

/// What the box holds between one thread and the next.
pub struct Composer {
    pub words: TextEdit,
    /// The level picked on this box; none is Agent Decides.
    pub level: Option<Level>,
    /// The project picked on this box, by its slug; none is the one the rules open it on.
    pub project: Option<String>,
    /// The model picked on this box; none is the one the rules open it on.
    pub pick: Option<Pick>,
    /// When the window says that a thread on its way has not been taken yet, on its clock.
    not_taken_at: Option<f64>,
    /// O pressed while a thread was on its way, and the screen it was pressed on: the thread
    /// is opened when it lands, unless the person has gone somewhere else by then.
    o_waits: Option<View>,
}

impl Default for Composer {
    fn default() -> Composer {
        Composer { words: TextEdit::new(""), level: None, project: None, pick: None, not_taken_at: None, o_waits: None }
    }
}

/// The box's words, each standing where Chromium puts its glyphs (`theme::pen`).
fn label(s: &str, size: f32, weight: f32, c: rune::Color) -> El {
    pen(crate::theme::label(s, size, weight, c), size)
}

/// The mark beside a level's word (`PriorityIcon` in `renderer/src/components/Priority.tsx`):
/// three bars lit from the left for Low, Medium and High, a fourth for Urgent, and all three
/// unlit while the agent decides.
pub fn level_mark(level: Option<Level>) -> El {
    let off = rule();
    let three = |a, b, c| row().items_end().gap(1.5).child(el().size(3.0, 4.0).radius(1.0).fill(a)).child(el().size(3.0, 7.5).radius(1.0).fill(b)).child(el().size(3.0, 11.0).radius(1.0).fill(c));
    match level {
        None => three(off, off, off),
        Some(Level::Urgent) => bars(&[4.0, 7.0, 10.0, 13.0], ink()),
        Some(Level::High) => three(ink(), ink(), ink()),
        Some(Level::Medium) => three(dim(), dim(), off),
        Some(Level::Low) => three(dim(), off, off),
    }
}

/// A project's own colour, in the square the chip and the menu draw it in (`.tc-swatch`).
pub fn swatch(slug: &str) -> El {
    let (r, g, b) = window::swatch(slug);
    row().size(17.0, 17.0).center().child(el().size(9.0, 9.0).radius(4.5).fill(rgb(r, g, b)))
}

/// The bar under the words: who gets the thread where someone besides the agent could, the
/// model, the project, the level, and Send.
fn bar(s: &State) -> El {
    let project = s.compose_project();
    let mut bar = row().pt(14.0).items_center().gap(8.0);
    if window::to_offered(&s.snap) {
        bar = bar.child(home::chip(vec![label("Agent", 13.0, 400.0, dim())]));
    }
    let model = rules::model_label(&s.compose_pick(), &s.models);
    let filed = match project {
        Some(p) => home::chip(vec![swatch(&p.slug), label(&p.name, 13.0, 400.0, dim())]),
        None => home::chip(vec![label("No project", 13.0, 400.0, dim())]),
    };
    let mut bar = bar
        .child(home::chip(vec![label(&model, 13.0, 400.0, dim())]))
        .child(filed)
        .child(home::chip(vec![level_mark(s.compose.level), label(rules::choice_label(s.compose.level), 13.0, 400.0, dim())]))
        .child(spacer())
        .child(home::attach());
    if project.is_none() {
        // Why Send is off when it is off for want of a project (`.tc-why`). Making one is
        // the web window's to do, so the way out is words here and not a button.
        bar = bar.child(label(&format!("{} {}", rules::NO_PROJECT_REASON, rules::NO_PROJECT_ACTION), 13.0, 400.0, dim()).pl(8.0));
    }
    // The two halves of Send are one control with no room between them (`.tc-send`).
    bar.child(row().child(home::send_main(s.compose_ready()).cursor(rune::Cursor::Pointer).on_press_tx(|s: &mut State, tx| s.send_thread(tx))).child(home::send_caret()))
}

/// The box on Home's page (`.hm-start .tc-card.tc-inline`): the words large over the bar, on
/// the page's glass. Its edge brightens while the caret is in it.
pub fn inline(s: &State) -> El {
    let field = text_input(&s.compose.words, TextStyle::new(22.0, 400.0).lh(30.0))
        .key(FIELD)
        .color(ink())
        .placeholder("What do you need done?", muted())
        // `caret-color: var(--accent)` on the box drawn on a page.
        .caret(accent(), rgba(110, 165, 255, 0.3))
        .on_edit(|s: &mut State| &mut s.compose.words)
        // Return is a new line, as in the web window's box; ⌘Return sends.
        .on_submit(|s: &mut State, _| s.compose.words.insert("\n"))
        .on_input_key(crate::keys::in_the_box);
    let mut card = glass(col())
        .pt(16.0)
        .px(18.0)
        .pb(14.0)
        .radius(10.0)
        .border(1.0, white(if s.compose.words.focused { 0.3 } else { 0.2 }))
        .border_layout()
        .box_shadow(shadow(0.0, 0.0, 0.0, 3.0, white(0.03), false))
        .box_shadow(shadow(0.0, 12.0, 40.0, 0.0, rgba(0, 0, 0, 0.3), false))
        // One line tall until the words need more: 74 points with the room under them.
        .child(el().min_h(74.0).pb(14.0).child(field.w_full()));
    if let Some(why) = s.compose_project().and_then(|p| rules::practice_refusal(Some(p))) {
        card = card.child(el().pt(12.0).child(label(why, 13.0, 400.0, dim())));
    }
    card.child(bar(s))
}

/// "6:10pm", and the day after it when that is not today: when an agent out of usage has room
/// again (`backAt` in `renderer/src/compose-says.ts`, with `resetDay` in `shared/usage.mts`).
/// Nothing once the moment has passed.
fn back_at(until: f64, now: f64) -> Option<String> {
    const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    if until <= now {
        return None;
    }
    let t = LocalTime::at(until as i64);
    let day_of = |ms: f64| ((ms / 1000.0) as i64 + i64::from(LocalTime::at(ms as i64).offset)).div_euclid(86_400);
    let hour = if t.hour.is_multiple_of(12) { 12 } else { t.hour % 12 };
    let at = format!("{hour}:{:02}{}", t.minute, if t.hour < 12 { "am" } else { "pm" });
    if day_of(until) == day_of(now) {
        return Some(at);
    }
    let weekday = DAYS[(day_of(until) + 4).rem_euclid(7) as usize];
    // Six days out or more a weekday alone would read as this week's.
    if until - now < 6.0 * 86_400_000.0 { Some(format!("{at} {weekday}")) } else { Some(format!("{at} {weekday} {} {}", MONTHS[(t.month as usize).clamp(1, 12) - 1], t.day)) }
}

impl State {
    /// The projects a thread can be filed into, in the person's own order.
    pub fn projects_offered(&self) -> Vec<&Product> {
        rules::projects_offered(&rules::rank_products(&self.snap.products, &self.snap.supervisor.product_order))
    }

    /// The project the box is on: the one picked on it, else the open conversation's, else
    /// the one last sent to.
    pub fn compose_project(&self) -> Option<&Product> {
        let on_screen = self.open_item().map(|i| i.product.as_str());
        rules::card_project(&self.projects_offered(), self.compose.project.as_deref(), on_screen, self.picks.get().project.as_deref())
    }

    /// The agent and model the box is on: the one picked on it while this Mac still runs it,
    /// else the agent from Settings on the model last picked for it.
    pub fn compose_pick(&self) -> Pick {
        let engines = Engines::of(&self.snap);
        let kept = |engine: &str| self.picks.get().model.get(engine).cloned();
        match &self.compose.pick {
            Some(pick) => rules::pick_this_mac_runs(pick.clone(), &engines, &self.models, kept),
            None => rules::starting_pick(&engines, &self.models, kept),
        }
    }

    /// Whether Send is lit: there is something to send, and somewhere it can run (`canSend`).
    pub fn compose_ready(&self) -> bool {
        rules::thread_message(&self.compose.words.text).is_some() && self.compose_project().is_some_and(|p| rules::practice_refusal(Some(p)).is_none())
    }

    /// Puts the caret in the box, at the end of what it holds (N on Home, or a letter typed
    /// with nothing to take it).
    pub fn write_in_the_box(&mut self, tx: &mut Tx) {
        let end = self.compose.words.text.len();
        self.compose.words.set_caret(end);
        self.compose.words.focused = true;
        tx.focus_when_shown(FIELD, false);
    }

    /// Escape in the box: the keys go back to the page, and the words stay.
    pub fn leave_the_box(&mut self, tx: &mut Tx) {
        self.compose.words.focused = false;
        tx.blur();
    }

    /// The project's name, as a line about a thread says where it went.
    fn project_named(&self, slug: &str) -> String {
        self.snap.products.iter().find(|p| p.slug == slug).map_or_else(|| slug.to_string(), |p| p.name.clone())
    }

    /// Sends what is in the box as a new thread (Send, or ⌘Return). The box empties at once
    /// and a line says where the thread is going; a refusal puts every word back.
    pub fn send_thread(&mut self, tx: &mut Tx) {
        let Some(message) = rules::thread_message(&self.compose.words.text) else {
            return;
        };
        let Some(product) = self.compose_project().filter(|p| rules::practice_refusal(Some(p)).is_none()).map(|p| p.slug.clone()) else {
            return;
        };
        let pick = self.compose_pick();
        let levels = rules::effort_choices(&pick, &self.models);
        // A level is sent only when one was picked that this model takes.
        let effort = self.picks.get().effort.get(&pick.engine).filter(|level| rules::effort_picked(Some(level), &levels)).cloned();
        let harness = rules::harness_fields(&pick, effort.as_deref(), Engines::of(&self.snap).rows().len(), self.models.codex_default.as_deref());
        let payload = rules::compose(&product, &message, rules::sent_priority(self.compose.level), harness, false);
        let draft = write::Draft { words: self.compose.words.text.clone(), level: self.compose.level, pick: self.compose.pick.clone() };
        // What was done before this and still waits its three seconds is written first.
        self.write_what_waits();
        let on_its_way = self.send(rules::COMPOSE, payload, write::Sent::Thread { product: product.clone(), draft });
        if !on_its_way && self.refused.is_some() {
            // No door to send through: the words stay where they are, and the line says so.
            return;
        }
        self.picks.change(|picks| picks.project = Some(product.clone()));
        self.compose.words = TextEdit::new("");
        self.compose.o_waits = None;
        tx.blur();
        // Alone, on the Inbox, the tab moves to where the thread will land; over Home or a
        // conversation the person stays where they are.
        let covered = self.view != View::Inbox;
        if let Some(tab) = rules::tab_after_sending(window::team_mode(&self.snap), false, Some(self.inbox_tab), covered) {
            self.show_tab(tab);
        }
        self.say(rules::sending_line(&self.project_named(&product), false), false, tx);
        if on_its_way {
            self.compose.not_taken_at = Some(tx.clock() + rules::NOT_ANSWERING_MS / 1000.0);
        }
    }

    /// What the engine said to a new thread. Taken, the thread is in the lists at once and a
    /// line names it, with O to open it and Z to take it back. Refused, every word goes back
    /// in the box and a line says why.
    pub(crate) fn thread_landed(&mut self, landed: write::Landed, tx: &Tx) {
        let write::Sent::Thread { product, draft } = landed.sent.clone() else {
            return;
        };
        self.compose.not_taken_at = None;
        if let Some(why) = landed.refusal() {
            // Unless the box holds something else by now: an older send never takes words away.
            if self.compose.words.text.trim().is_empty() {
                self.compose.words = TextEdit::new(draft.words);
                self.compose.level = draft.level;
                self.compose.pick = draft.pick;
            }
            self.compose.o_waits = None;
            self.say(rules::not_sent(&why), false, tx);
            return;
        }
        let made = landed.answer.ok().and_then(|row| serde_json::from_value::<Item>(row).ok()).filter(|row| !row.id.is_empty());
        // The lists as the engine has them now, with the new thread in them a whole read sooner
        // than the next turn would bring it (`showJustMade`).
        let limit = made.as_ref().and_then(|row| landed.snapshot.as_ref().and_then(|snapshot| rules::limit_of(snapshot, &row.id)));
        if let Some(snapshot) = landed.snapshot.and_then(|snapshot| serde_json::from_value(snapshot).ok()) {
            self.snap = snapshot;
        }
        if let Some(row) = &made {
            if !self.snap.items.iter().any(|i| i.id == row.id && i.product == row.product) {
                self.snap.items.push(row.clone());
            }
        }
        self.refresh();
        let back = limit.as_ref().map(|limit| (limit.tool.as_str(), back_at(limit.until, self.now)));
        let line = rules::sent_line(&self.project_named(&product), false, back.as_ref().map(|(tool, at)| (*tool, at.as_deref())));
        let Some(row) = made else {
            self.say(rules::toast_parts(&line).0, false, tx);
            return;
        };
        self.last = Some(write::Undo::Withdraw { id: row.id.clone(), product: row.product.clone(), title: row.title.clone().unwrap_or_default(), draft });
        // O pressed while it was on its way opens it now, if the person is still where they were.
        if self.compose.o_waits.take().is_some_and(|screen| screen == self.view) {
            self.toast = None;
            self.open_thread(&row.id, &row.product);
            return;
        }
        self.say_about(&line, (row.id, row.product), tx);
    }

    /// The thread the line in the corner is about, as its row names it, once the lists have it.
    pub fn toast_title(&self) -> Option<String> {
        let (id, product) = self.toast.as_ref()?.goes.as_ref()?;
        let listed = |rows: &[crate::data::Row]| rows.iter().any(|r| &r.id == id && &r.product == product);
        if !(listed(&self.lists.needs) || listed(&self.lists.progress) || listed(&self.lists.later) || listed(&self.lists.done)) {
            return None;
        }
        self.snap.items.iter().find(|i| &i.id == id && &i.product == product).map(title::row_title)
    }

    /// O, or a press on the line: into the thread it is about, while it is up and names one
    /// that is not already open (`opensTheToast`). With a thread still on its way, O waits
    /// for it. False when O is nobody's.
    pub fn open_what_the_line_names(&mut self) -> bool {
        let goes = self.toast.as_ref().and_then(|t| t.goes.clone());
        let Some((id, product)) = goes else {
            if self.compose.not_taken_at.is_some() {
                self.compose.o_waits = Some(self.view.clone());
                return true;
            }
            return false;
        };
        if matches!(&self.view, View::Thread { id: open, product: of } if open == &id && of == &product) {
            return false;
        }
        self.toast = None;
        self.open_thread(&id, &product);
        true
    }

    /// Z on a thread sent from the box: it is closed, which stops its agent, unless the agent
    /// has already answered. Then closing would only hide the answer, so the answer is shown.
    pub(crate) fn withdraw(&mut self, id: String, product: String, title: String, draft: write::Draft, tx: &mut Tx) {
        let row = self.snap.items.iter().find(|i| i.id == id && i.product == product);
        if rules::answered_already(row) {
            self.say(rules::TOO_LATE_TO_WITHDRAW.to_string(), false, tx);
            self.open_thread(&id, &product);
            return;
        }
        if !self.send(crate::rules::writes::ANSWER, rules::withdraw(&product, &id), write::Sent::Withdraw { id: id.clone(), product: product.clone(), title: title.clone(), draft: draft.clone() }) {
            // Not sent, so not undone: Z still has it.
            self.last = Some(write::Undo::Withdraw { id, product, title, draft });
        }
    }

    /// What the engine said to that close. Done, what the box held goes back in it, unless
    /// it holds something newer; refused, Z keeps the thread and a line says why.
    pub(crate) fn withdraw_landed(&mut self, landed: write::Landed, tx: &Tx) {
        let refusal = landed.refusal();
        let write::Sent::Withdraw { id, product, title, draft } = landed.sent else {
            return;
        };
        if let Some(why) = refusal {
            self.say(rules::could_not_undo(&why), false, tx);
            self.last = Some(write::Undo::Withdraw { id, product, title, draft });
            return;
        }
        // Out of Done from now on: a send taken back is not finished work.
        self.withdrawn.insert(id, crate::clock());
        self.refresh();
        let back = self.compose.words.text.trim().is_empty() && !draft.words.trim().is_empty();
        if back {
            self.compose.words = TextEdit::new(draft.words);
            self.compose.level = draft.level;
            self.compose.pick = draft.pick;
        }
        self.say(rules::said_withdrawn(&title, back), false, tx);
    }

    /// What a picture taken just after a send hands over (`sent` in its fixture): the row the
    /// page was answered with and when. The same answer lands here, and the line about it is
    /// as old as the page's was at the shutter.
    pub(crate) fn picture_sent(&mut self, sent: &Value, tx: &Tx) {
        let Some(made) = sent.get("made") else {
            return;
        };
        let product = made.get("product").and_then(Value::as_str).unwrap_or_default().to_string();
        let draft = write::Draft { words: self.fixture.get("typed").and_then(Value::as_str).unwrap_or_default().to_string(), ..write::Draft::default() };
        self.thread_landed(write::Landed { sent: write::Sent::Thread { product, draft }, answer: Ok(made.clone()), snapshot: None }, tx);
        let ago = sent.get("at").and_then(Value::as_f64).map_or(0.0, |at| (self.now - at) / 1000.0);
        if let Some(toast) = &mut self.toast {
            toast.since -= ago;
            toast.until -= ago;
        }
    }

    /// The line the window owes a thread it has not heard back about (`SEND_NOT_TAKEN`):
    /// silence after Send reads as the send not having happened. True when it was just said.
    pub(crate) fn say_if_not_taken(&mut self, at: f64, tx: &mut Tx) -> bool {
        match self.compose.not_taken_at {
            Some(due) if at >= due => {
                self.compose.not_taken_at = None;
                self.say(rules::SEND_NOT_TAKEN.to_string(), false, tx);
                true
            }
            Some(due) => {
                tx.wake_after(std::time::Duration::from_secs_f64(due - at));
                false
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_agent_out_of_usage_is_back_at_a_time_and_on_a_day_when_that_is_not_today() {
        let now = 1_784_000_000_000.0;
        assert_eq!(back_at(now - 1.0, now), None);
        assert_eq!(back_at(now, now), None);
        // Whatever the zone, a moment a minute off names no day unless midnight falls between.
        let soon = back_at(now + 60_000.0, now).unwrap();
        assert!(soon.ends_with("am") || soon.ends_with("pm") || soon.split(' ').count() == 2, "{soon}");
        // Two days out names the weekday after the time, and a fortnight out the date too.
        assert_eq!(back_at(now + 2.0 * 86_400_000.0, now).unwrap().split(' ').count(), 2);
        assert_eq!(back_at(now + 14.0 * 86_400_000.0, now).unwrap().split(' ').count(), 4);
    }
}
