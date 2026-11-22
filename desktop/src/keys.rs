//! What the keys do while no text field has them: the web window's own, from the keydown
//! handler in `renderer/src/App.tsx` and Home's in `renderer/src/home/HomePage.tsx`.
//!
//! J and K (or the arrows) walk a list and Enter opens the row they are on; Tab turns the
//! Inbox's tabs; inside a conversation Escape goes back and J and K go on to the next thread
//! or back to the one before, leaving the arrows to the page.

use rune::edit::Key;
use rune::{KeyEvent, Tx};

use crate::rules::page;
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

/// True when the key was this window's. A chord is the system's, never one of these.
pub fn pressed(s: &mut State, e: &KeyEvent, _: &mut Tx) -> bool {
    if e.mods.cmd || e.mods.ctrl || e.mods.alt {
        return false;
    }
    let letter = match &e.key {
        Key::Text(t) => t.to_lowercase(),
        _ => String::new(),
    };
    let (j, k) = (letter == "j", letter == "k");
    let (down, up) = (j || e.key == Key::Down, k || e.key == Key::Up);
    match s.view.clone() {
        View::Home => {
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
            let (len, on) = {
                let rows = s.lists.under(s.inbox_tab);
                (rows.len(), rows.get(s.picked).map(|r| (r.id.clone(), r.product.clone())))
            };
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
            false
        }
        View::Thread { id, product } => {
            if e.key == Key::Escape {
                s.leave_thread();
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
