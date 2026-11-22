//! The reply box's model and level: what the next reply to a conversation runs on, kept per
//! conversation, and the menu its chip opens. Ported from `DockComposer` in
//! `renderer/src/components/Focus.tsx` and from `ReplyModel.tsx`; the rules are
//! `rules/reply_pick.rs`, and the menu is the New thread box's own (`compose.rs`), read off the
//! reply's pick, as the web window's reply box draws the card's `ModelMenu`.
//!
//! A pick is kept, not written. It waits beside the box's words (`saveDraftPick`), a fold keeps
//! it, and the next reply carries it, the model written before the words so the run that reply
//! starts reads it (`answerItem` in `main/store.mts`). A pick with nothing typed writes nothing,
//! and a picked option of the agent's carries none, as in the web window.
//!
//! The menu opens from the chip in the open box, and from the model's name on the folded box,
//! which opens the box with the menu already open (`openModel`). While it is open the caret is
//! out of the words, as the web window's focus is on the menu's rows: the arrows walk them,
//! Enter presses the one lit and Escape closes the menu; any other key goes on to the page, as
//! the web window's keydown handler lets it through while the reply box is up
//! (`keyWhileABoxIsOpen` in `renderer/src/keys.ts`). However it closes, the caret goes back to
//! the words, where ⌘Return sends from (`onOpenChange`).

use rune::edit::Key;
use rune::{KeyEvent, Tx};

use crate::compose::{Menu, MenuKind};
use crate::data::Item;
use crate::rules::compose::{self as rules, Engines, Entry, Pick};
use crate::rules::reply_pick::{self, ReplyPick};
use crate::{State, write};

/// The reply box's model chip, which the menu rises out of, and the model's name on the folded box.
pub const CHIP: &str = "reply-model";
pub const WORD: &str = "reply-model-word";
/// The reply's words and its Send, which a press that closes the menu goes on to.
pub const FIELD: &str = "reply";
pub const SEND: &str = "reply-send";
/// What a press outside the open menu is held against.
pub const MARKS: [&str; 2] = [FIELD, SEND];

fn key(item: &Item) -> String {
    format!("{}/{}", item.product, item.id)
}

impl State {
    /// The pick of a reply to this thread still waiting its three seconds, which the box and its
    /// folded line show until the row carries it (`pickOnItsWay`).
    pub fn pick_on_its_way(&self, item: &Item) -> Option<ReplyPick> {
        match &self.held.as_ref()?.sent {
            write::Sent::Reply { thread, pick: Some(pick), .. } if *thread == key(item) => Some(pick.clone()),
            _ => None,
        }
    }

    /// What the reply box is on for a thread, and whether it is the person's own pick
    /// (`replyOpensOn`).
    pub fn reply_pick(&self, item: &Item) -> (ReplyPick, bool) {
        reply_pick::reply_opens_on(item, self.reply_picks.get(&key(item)), self.pick_on_its_way(item).as_ref())
    }

    /// The agent the thread's next reply reaches, and the one the box shows: the same until a
    /// pick in the menu names the other (`engineNow`, `engineShown`).
    pub fn reply_engines(&self, item: &Item) -> (String, String) {
        let (pick, _) = self.reply_pick(item);
        reply_pick::engines_of(reply_pick::running_engine(&self.snap, &item.id).as_deref(), &pick)
    }

    /// The model the open conversation's menu is on, as a row of it, and the level picked.
    pub(crate) fn reply_menu_pick(&self) -> (Pick, Option<String>) {
        let Some(item) = self.open_item() else {
            return (Pick::new("claude", rules::DEFAULT_MODEL), None);
        };
        let (pick, _) = self.reply_pick(item);
        let (_, shown) = self.reply_engines(item);
        (reply_pick::menu_pick(&pick, &shown, &self.models), pick.effort)
    }

    /// What the reply carries of the pick, as the thread must record it, when it is the
    /// person's own (`pickToSend`); nothing when she has not touched the menu.
    pub(crate) fn reply_pick_to_send(&self, item: &Item) -> Option<ReplyPick> {
        let kept = self.reply_picks.get(&key(item))?;
        let (now, shown) = self.reply_engines(item);
        Some(reply_pick::pick_to_send(kept, &shown, &now, Engines::of(&self.snap).rows().len()))
    }

    /// The chip's words: the model the box is on, and its level where one will be sent
    /// (`engineModelLabel`, `effortSaid`).
    pub fn reply_chip_words(&self, item: &Item) -> (String, Option<String>) {
        let (pick, _) = self.reply_pick(item);
        let (_, shown) = self.reply_engines(item);
        let running = reply_pick::running_engine(&self.snap, &item.id);
        (reply_pick::model_word(Some(&shown), pick.model.as_deref(), &self.models), reply_pick::effort_said(&pick, running.as_deref(), &self.models))
    }

    /// The model the folded box names: the row's, or that of a reply still on its way to it
    /// (`modelShown`).
    pub fn folded_model_word(&self, item: &Item) -> String {
        let running = reply_pick::running_engine(&self.snap, &item.id);
        let (engine, model) = reply_pick::model_shown(item, running.as_deref(), self.pick_on_its_way(item).as_ref());
        reply_pick::model_word(engine.as_deref(), model.as_deref(), &self.models)
    }

    /// The line under the words while the pick moves the thread to another agent (`freshNote`).
    pub fn reply_fresh_note(&self, item: &Item) -> Option<String> {
        let (now, shown) = self.reply_engines(item);
        reply_pick::fresh_note(&Engines::of(&self.snap), &now, &shown)
    }

    /// A press on the reply box's model chip: its menu opens with the keys on the model the
    /// reply is on, or closes when it is open (`setOpen((o) => !o)`).
    pub fn toggle_reply_menu(&mut self, tx: &mut Tx) {
        if self.reply_menu.is_some() {
            self.close_reply_menu();
        } else {
            self.open_reply_menu(true, tx);
        }
    }

    /// The menu opens. `keys`: with the keys on the model the reply is on, the caret out of the
    /// words; else with no row lit and the caret left in the words, as the web window's box
    /// takes its focus back as it mounts (`focusAtEnd`, after `ReplyModel` gave it to the row).
    fn open_reply_menu(&mut self, keys: bool, tx: &mut Tx) {
        // One thing stands open over a conversation at a time.
        self.close_strip();
        crate::chips::let_go(self);
        let mut menu = Menu { kind: MenuKind::Model, every_model: false, lit: crate::compose::NO_ENTRY, reply: true };
        if keys {
            menu.lit = rules::entry_lit_first(&self.menu_entries(&menu), |entry| self.entry_is_picked(entry, true));
            self.reply.focused = false;
            tx.blur();
        }
        self.reply_menu = Some(menu);
        // The lists may have changed since the window opened: an agent's can land late.
        self.read_the_models();
    }

    /// The model's name on the folded box: the box opens with the menu already open, which is
    /// what the word promised (`setOpenModel(true); onReply()`). The caret is in the words and
    /// no row is lit, so the arrows move the caret until the pointer lights a row.
    pub fn open_reply_on_the_model(&mut self, tx: &mut Tx) {
        self.open_reply(tx);
        if self.replying {
            self.open_reply_menu(false, tx);
        }
    }

    /// The menu closes and the caret is back in the words, which ⌘Return sends from
    /// (`onOpenChange`, the box taking the focus back when the drawer shuts). A menu of the
    /// box's other chips closes with it, as whatever opens over the conversation closes them.
    pub fn close_reply_menu(&mut self) {
        self.chips.menu = None;
        if self.reply_menu.take().is_some() && self.replying {
            self.reply.focused = true;
        }
    }

    /// Keeps a change to the open conversation's pick, beside its words (`pickTo`).
    fn keep_reply_pick(&mut self, change: impl FnOnce(&ReplyPick, &str) -> ReplyPick) {
        let Some(item) = self.open_item().cloned() else {
            return;
        };
        let (was, _) = self.reply_pick(&item);
        let (_, shown) = self.reply_engines(&item);
        let now = change(&was, &shown);
        self.reply_picks.insert(key(&item), now);
    }

    /// Enter on an entry of the reply box's menu, or a press on it. A model closes the menu; a
    /// level stays in it, since she may be there to set both (`onPick`, `onEffort`).
    pub(crate) fn press_reply_entry(&mut self, at: usize) {
        let Some(mut menu) = self.reply_menu.clone() else {
            return;
        };
        let Some(entry) = self.menu_entries(&menu).into_iter().nth(at) else {
            return;
        };
        match entry {
            Entry::Model(to) => {
                self.keep_reply_pick(|was, shown| reply_pick::picking_model(was, shown, &to));
                self.close_reply_menu();
            }
            Entry::AllModels | Entry::Recent => {
                menu.every_model = entry == Entry::AllModels;
                menu.lit = rules::entry_lit_first(&self.menu_entries(&menu), |entry| self.entry_is_picked(entry, true));
                self.reply_menu = Some(menu);
            }
            Entry::Effort(level) => self.keep_reply_pick(|was, _| reply_pick::picking_effort(was, &level)),
            _ => {}
        }
    }

    /// A press outside the open menu, as it goes down, with where the reply's controls stood:
    /// the menu closes and the caret goes back to the words, and a press on Send sends. Anywhere
    /// else it only closes, where the web window's press also lands on what it was aimed at.
    pub(crate) fn reply_pressed_away(&mut self, at: [f32; 2], marks: &[(&'static str, [f32; 4])], tx: &mut Tx) {
        let inside = |key: &str| marks.iter().any(|(mark, [x, y, w, h])| *mark == key && at[0] >= *x && at[0] < x + w && at[1] >= *y && at[1] < y + h);
        if self.reply_menu.is_none() {
            return;
        }
        self.close_reply_menu();
        if inside(SEND) {
            self.send_reply(tx);
        }
    }

    /// What a picture taken with the reply box's model menu open says of it (`actions: model`):
    /// the box opened by the model's name on the folded box, as the scene pressed it.
    pub(crate) fn picture_reply_menu(&mut self, tx: &mut Tx) {
        self.open_reply_on_the_model(tx);
    }
}

/// The keys while the reply box's menu is open: the arrows walk its entries around both ends,
/// Enter presses the one lit, Escape closes it. ⌘Return sends nothing from there, as the web
/// window's rows are buttons outside the words. `None` while it is not open, and `Some(false)`
/// for a key the page has.
pub fn menu_key(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> Option<bool> {
    let menu = s.reply_menu.clone()?;
    if e.mods.cmd || e.mods.ctrl || e.mods.alt {
        return Some(e.key == Key::Enter);
    }
    match e.key {
        Key::Escape => s.close_reply_menu(),
        Key::Down | Key::Up => {
            let len = s.menu_entries(&menu).len();
            // From no row at all, down lands on the first and up on the last.
            let lit = match (menu.lit, e.key == Key::Down) {
                (crate::compose::NO_ENTRY, true) => 0,
                (crate::compose::NO_ENTRY, false) => len.saturating_sub(1),
                (at, down) => rules::entry_step(at, len, down),
            };
            s.light_entry(lit);
            tx.scroll_into_view(format!("entry-{lit}"));
        }
        Key::Enter => s.press_entry(menu.lit, tx),
        _ => return Some(false),
    }
    Some(true)
}
