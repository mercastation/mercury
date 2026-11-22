//! What the keys do while no text field has them: the web window's own, from the keydown
//! handler in `renderer/src/App.tsx` and Home's in `renderer/src/home/HomePage.tsx`.
//!
//! J and K (or the arrows) walk a list and Enter opens the row they are on; Tab turns the
//! Inbox's tabs. Inside a conversation Escape goes back, J and K go on to the next thread or
//! back to the one before, the arrows walk the agent's options and Enter or a number sends
//! one, R opens the reply box and E marks the thread done. N starts a thread: in Home's box, or
//! in the New thread card anywhere else, which then has every key (`compose::key`). Z takes
//! back the last thing done.

use rune::edit::Key;
use rune::{KeyEvent, Tx};

use crate::rules::{compose, page};
use crate::{State, View};

/// One step along a list of `len` rows from `at`, stopping at either end.
fn step(at: usize, down: bool, len: usize) -> usize {
    if down { (at + 1).min(len.saturating_sub(1)) } else { at.saturating_sub(1) }
}

/// The tab after `open` among the ones the strip draws, or before it, around the end
/// (`nextTab` in `renderer/src/threads/page-rules.ts`).
fn next_tab(s: &State, back: bool) -> crate::data::Tab {
    let shown = page::tabs(&s.lists, s.inbox_tab);
    let Some(at) = shown.iter().position(|t| *t == s.inbox_tab) else {
        return shown.first().copied().unwrap_or_default();
    };
    shown[(at + if back { shown.len() - 1 } else { 1 }) % shown.len()]
}

/// The keys the reply box does not keep for typing: ⌘Return sends, Escape folds the box.
pub fn in_the_reply(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> bool {
    if e.key == Key::Enter && e.mods.cmd {
        s.send_reply(tx);
        return true;
    }
    if e.key == Key::Escape {
        s.fold_reply();
        return true;
    }
    false
}

/// The keys the box a thread is started in does not keep for typing: ⌘Return sends, and
/// Escape gives the keys back to the page, or closes the New thread card, keeping the words.
pub fn in_the_box(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> bool {
    if e.key == Key::Enter && (e.mods.cmd || e.mods.ctrl) {
        s.send_thread(tx);
        return true;
    }
    if e.key == Key::Escape {
        if s.compose.card { s.close_card(tx) } else { s.leave_the_box(tx) }
        return true;
    }
    false
}

/// The Inbox's row the keys act on, as `(id, product)`: the one the pointer moved onto while it
/// is still there and still in the list, else the picked one (`pointed` in `App.tsx`).
fn picked_row(s: &State) -> Option<(String, String)> {
    let rows = s.lists.under(s.inbox_tab);
    let pointed = rows.iter().find(|r| s.pointed.is_row(&r.product, &r.id));
    pointed.or_else(|| rows.get(s.picked)).map(|r| (r.id.clone(), r.product.clone()))
}

/// E: marks done the thread the keys are on. That is the open conversation, or the Inbox's
/// picked row under Needs you and In progress. Under Later E brings the thread back instead,
/// and under Done there is nothing to close.
pub fn close_under_the_keys(s: &mut State, tx: &mut Tx) {
    match s.view.clone() {
        View::Thread { .. } => s.close_thread(false, tx),
        View::Inbox => {
            let Some((id, product)) = picked_row(s) else {
                return;
            };
            match s.inbox_tab {
                crate::data::Tab::Needs | crate::data::Tab::Progress => s.close(&id, &product, false, tx),
                crate::data::Tab::Later => s.bring_back(&id, &product),
                _ => {}
            }
        }
        _ => {}
    }
}

/// L: opens the Later picker on the thread the keys are on. A running thread can be put off
/// too, because Later is about the person's inbox and not about the session; a closed one
/// cannot, and says so rather than going quiet.
fn later_under_the_keys(s: &mut State, tx: &mut Tx) {
    match s.view.clone() {
        View::Thread { id, product } => s.open_later(&id, &product, tx),
        View::Inbox => {
            let Some((id, product)) = picked_row(s) else {
                return;
            };
            if s.inbox_tab == crate::data::Tab::Done {
                s.tell("Later is for a thread that is still open.", tx);
            } else {
                s.open_later(&id, &product, tx);
            }
        }
        View::Home => {}
    }
}

/// True when the key was this window's. A chord is the system's, never one of these.
pub fn pressed(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> bool {
    // A key hint is for the hand on the pointer, and any key takes it away (`keydown` in
    // `renderer/src/App.tsx`'s hint guards).
    s.pointed.hint = None;
    // ⌘K from anywhere, and every key while the command bar is open, are the bar's. One thing
    // stands over the page at a time, as the web window's one `modal`: the bar opening closes
    // the New thread card, which keeps its words.
    if crate::palette::took(s, e, tx) {
        if s.palette.open && s.compose.card {
            s.close_card(tx);
        }
        return true;
    }
    // Then the box a thread is started in: the card has every key, its open menu has the
    // keyboard, a letter with the keyboard on one of its chips is typed, and ⌘Return sends it.
    if let Some(taken) = crate::compose::key(s, e, tx) {
        return taken;
    }
    // ⌘[ is Back, as Back's title says (`Back to … (⌘[)` in `BackForward.tsx`): a conversation
    // goes back to where it was opened from, any other page to the one before it.
    if e.mods.cmd && e.key == Key::Command('[') {
        match s.view {
            View::Thread { .. } => s.leave_thread(),
            _ => s.back(),
        }
        return true;
    }
    if e.mods.cmd || e.mods.ctrl || e.mods.alt {
        return false;
    }
    // While the Later picker is open the keys are its own, and the page under it has none.
    if s.later.on.is_some() {
        crate::later::pressed(s, e, tx);
        return true;
    }
    // A conversation's open list of files takes Escape and no other key, which goes on to the
    // page under it as in the web window (`ThreadFilesButton`): the arrows still walk the options.
    if e.key == Key::Escape && s.files_open() {
        s.close_files();
        return true;
    }
    let letter = match &e.key {
        Key::Text(t) => t.to_lowercase(),
        _ => String::new(),
    };
    // The hand is back on the keys, so the pointer stops holding a row (`takesTheListBack`).
    if matches!(e.key, Key::Down | Key::Up | Key::Escape | Key::Tab) || letter == "j" || letter == "k" {
        s.pointed.row = None;
    }
    // Z takes back the last thing done, wherever the person is by then.
    if letter == "z" {
        s.undo(tx);
        return true;
    }
    // O goes into the thread the line in the corner is about, while that line is up.
    if letter == "o" && s.open_what_the_line_names() {
        return true;
    }
    // N starts a thread: in the box already on Home's page (below), and anywhere else in the
    // New thread card over the page (`startThread` in `renderer/src/App.tsx`).
    if letter == "n" && s.view != View::Home {
        s.open_card(tx);
        return true;
    }
    if letter == "l" && s.view != View::Home {
        later_under_the_keys(s, tx);
        return true;
    }
    let (j, k) = (letter == "j", letter == "k");
    let (down, up) = (j || e.key == Key::Down, k || e.key == Key::Up);
    match s.view.clone() {
        View::Home => {
            // N goes into the box already on the page, and so does a letter that is none of
            // the list's own: somebody typing, with a box on the page to type in.
            if letter == "n" {
                s.write_in_the_box(tx);
                return true;
            }
            if let Key::Text(typed) = &e.key {
                if !compose::key_is_the_lists(typed) {
                    s.write_in_the_box(tx);
                    s.compose.words.insert(typed);
                    return true;
                }
            }
            // Home lists the first eighty that need the person, and the keys walk those.
            let rows = s.lists.needs.len().min(80);
            if rows == 0 {
                return false;
            }
            if down || up {
                // From no row at all either key lands on the first.
                s.home_pick = Some(s.home_pick.map_or(0, |at| step(at, down, rows)));
                return true;
            }
            if e.key == Key::Enter {
                let Some(r) = s.home_pick.and_then(|at| s.lists.needs.get(at)) else {
                    return false;
                };
                let (id, product) = (r.id.clone(), r.product.clone());
                s.open_thread(&id, &product);
                return true;
            }
            false
        }
        View::Inbox => {
            if e.key == Key::Tab {
                let tab = next_tab(s, e.mods.shift);
                s.show_tab(tab);
                return true;
            }
            let (len, on) = (s.lists.under(s.inbox_tab).len(), picked_row(s));
            if len == 0 {
                return false;
            }
            if down || up {
                s.picked = step(s.picked.min(len - 1), down, len);
                return true;
            }
            if e.key == Key::Enter {
                let Some((id, product)) = on else {
                    return false;
                };
                s.open_thread(&id, &product);
                return true;
            }
            if letter == "e" {
                close_under_the_keys(s, tx);
                return true;
            }
            false
        }
        View::Thread { id, product } => {
            // Escape is one step back: the open reply box first, then the conversation.
            if e.key == Key::Escape {
                if s.replying { s.fold_reply() } else { s.leave_thread() }
                return true;
            }
            // The arrows walk the agent's options, Enter sends the one they are on, and a
            // number sends that option outright.
            let choices = s.open_item().map(crate::thread::live_options).unwrap_or_default();
            if !choices.is_empty() && (e.key == Key::Down || e.key == Key::Up) {
                s.option = match (s.option, e.key == Key::Down) {
                    (None, true) => Some(0),
                    (Some(at), true) => Some((at + 1).min(choices.len() - 1)),
                    // Above the first is no option at all.
                    (Some(at), false) if at > 0 => Some(at - 1),
                    (_, false) => None,
                };
                return true;
            }
            if e.key == Key::Enter {
                let Some(at) = s.option else {
                    return false;
                };
                s.pick_option(at, tx);
                return true;
            }
            if let Some(n) = letter.parse::<f64>().ok().filter(|n| *n >= 1.0) {
                if let Some(at) = choices.iter().position(|c| c.n == n) {
                    s.pick_option(at, tx);
                }
                // Swallowed either way: a pick can open the box, and the key would type itself.
                return true;
            }
            if letter == "r" {
                s.open_reply(tx);
                return true;
            }
            // E closes, and never anything else.
            if letter == "e" {
                close_under_the_keys(s, tx);
                return true;
            }
            // J and K walk the threads that need the person from inside one, as the strip's
            // two arrows do. The arrows themselves are the page's and the options'.
            if j || k {
                let needs = &s.lists.needs;
                let Some(at) = needs.iter().position(|r| r.id == id && r.product == product) else {
                    return true;
                };
                let to = if j { needs.get(at + 1) } else { at.checked_sub(1).and_then(|a| needs.get(a)) };
                if let Some((id, product)) = to.map(|r| (r.id.clone(), r.product.clone())) {
                    s.step_to_thread(&id, &product);
                }
                return true;
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_step_stops_at_either_end_of_the_list() {
        assert_eq!(step(0, true, 3), 1);
        assert_eq!(step(2, true, 3), 2);
        assert_eq!(step(0, false, 3), 0);
        assert_eq!(step(2, false, 3), 1);
        assert_eq!(step(0, true, 1), 0);
    }
}
