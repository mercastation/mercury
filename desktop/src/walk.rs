//! Back and forward on the inbox page (`useBackAndForward` in
//! `renderer/src/back-and-forward.ts`, as `renderer/src/App.tsx` feeds it): the trail of the
//! list and each thread stood on, which ⌘[ and ⌘] and the two arrows walk. A place counts once
//! it has stayed on screen for 300 ms. The arrows are drawn on the inbox page alone, its list
//! in any tab and a thread; Home, Files and Settings have none, though the keys are held on
//! every page. The rules are `rules/trail.rs`.
//!
//! Escape and a conversation's own way back are not this: they lead to where the thread was
//! opened from (`State::leave_thread`).

use std::time::Duration;

use rune::Tx;

use crate::rules::trail::{self, Step, Stop, Trail};
use crate::{State, View};

#[derive(Default)]
pub struct Walk {
    trail: Trail,
    /// A back or a forward on its way, so arriving there is not recorded as a new visit, which
    /// would drop everything ahead of it (`stepping`).
    stepping: Option<Step>,
    /// The place on screen at the last look, and when it will have stayed long enough to count.
    seen: Option<Stop>,
    due: Option<f64>,
    /// The last thread stood on, whose row the list washes (`lastThread`).
    last_thread: Option<(String, String)>,
}

impl State {
    /// What is on screen, as far as the arrows care (`screenHere` and `stopOnScreen`): a thread,
    /// or the list in its tab while nothing is open over it; nothing on Home, in Files or under
    /// Settings, where the arrows are not drawn and the keys step nowhere.
    pub fn stop_on_screen(&self) -> Option<Stop> {
        if self.settings.open {
            return None;
        }
        match &self.view {
            View::Thread { id, product } => Some(Stop::Thread { product: product.clone(), id: id.clone() }),
            View::Inbox => Some(Stop::List(self.inbox_tab)),
            View::Home | View::Files { .. } => None,
        }
    }

    /// Whether a place can still be opened (`exists`): a thread while it is on this computer.
    fn stop_exists(&self, stop: &Stop) -> bool {
        match stop {
            Stop::Thread { product, id } => self.snap.items.iter().any(|i| &i.id == id && &i.product == product),
            Stop::List(_) => true,
        }
    }

    /// Where Back leads from here, and where Forward does (`backTo`, `forwardTo`); none with
    /// nowhere to go, and none off the inbox page.
    pub fn back_to(&self) -> Option<Stop> {
        let here = self.stop_on_screen()?;
        trail::step_back(&self.walk.trail, Some(&here), &|s| self.stop_exists(s)).map(|(_, stop)| stop)
    }
    pub fn forward_to(&self) -> Option<Stop> {
        let here = self.stop_on_screen()?;
        trail::step_forward(&self.walk.trail, Some(&here), &|s| self.stop_exists(s)).map(|(_, stop)| stop)
    }

    /// A place's name in an arrow's title (`stopName`): the list by its tab, a thread by its name.
    pub fn stop_name(&self, stop: &Stop) -> String {
        match stop {
            Stop::List(tab) => tab.label().to_string(),
            Stop::Thread { product, id } => self.snap.items.iter().find(|i| &i.id == id && &i.product == product).map_or_else(|| "the thread".to_string(), crate::rules::title::row_title),
        }
    }

    /// The last thread stood on, while the arrows walk (`lastThread`).
    pub fn last_thread(&self) -> Option<&(String, String)> {
        self.stop_on_screen().and(self.walk.last_thread.as_ref())
    }

    /// ⌘[ or ⌘], or an arrow pressed (`travel`): one step along the trail, on the inbox page
    /// alone.
    pub fn travel(&mut self, forward: bool) {
        let Some(here) = self.stop_on_screen() else { return };
        let next = if forward { trail::step_forward(&self.walk.trail, Some(&here), &|s| self.stop_exists(s)) } else { trail::step_back(&self.walk.trail, Some(&here), &|s| self.stop_exists(s)) };
        let Some((trail, stop)) = next else { return };
        self.walk.stepping = Some(Step { to: stop.clone(), reached: false });
        self.walk.trail = trail;
        self.go_to_stop(&stop);
    }

    /// Opens a place the arrows reach (`go`). The list comes back as Escape leaves it, the keys
    /// where they were, unless it was on another tab, which opens from its first row. A thread
    /// opens over the inbox, so its own way back leads there.
    fn go_to_stop(&mut self, stop: &Stop) {
        match stop {
            Stop::List(tab) => {
                if *tab != self.inbox_tab {
                    self.go(View::Inbox);
                    self.show_tab(*tab);
                } else if matches!(self.history.last(), Some(View::Inbox)) {
                    self.back();
                } else {
                    self.go(View::Inbox);
                }
            }
            Stop::Thread { product, id } => {
                if !self.stop_exists(stop) {
                    return;
                }
                // `openView(view)`: the list in the tab it is on, from its first row, under the thread.
                self.go(View::Inbox);
                self.picked = 0;
                self.open_thread(id, product);
            }
        }
    }
}

/// A window opened straight onto a conversation, for a picture of it (`--thread`), is taken to
/// have come there from the list in the tab it is on, as the web window's picture of the same
/// thread did: the list counted, then the thread, so Back leads to the list.
pub fn opened_from_the_list(s: &mut State) {
    let Some(thread) = s.stop_on_screen().filter(|stop| matches!(stop, Stop::Thread { .. })) else { return };
    let trail = trail::arrive(&trail::arrive(&Trail::default(), &Stop::List(s.inbox_tab)), &thread);
    s.walk = Walk { trail, seen: Some(thread.clone()), last_thread: trail::last_thread_after(None, &thread), ..Walk::default() };
}

/// The poll's part (`settledPlaces`, and the effect that feeds it): each new place on screen
/// starts its 300 ms, and one that stays them counts. Off the inbox page nothing is counted
/// and a step on its way is forgotten.
pub fn tend(s: &mut State, at: f64, tx: &mut Tx) -> bool {
    let here = s.stop_on_screen();
    if here != s.walk.seen {
        s.walk.seen = here.clone();
        match &here {
            None => {
                s.walk.due = None;
                s.walk.stepping = None;
            }
            Some(here) => {
                if let Some(step) = s.walk.stepping.as_mut().filter(|step| !step.reached && &step.to == here) {
                    step.reached = true;
                }
                s.walk.due = Some(at + trail::SETTLE);
                tx.wake_after(Duration::from_secs_f64(trail::SETTLE));
            }
        }
    }
    let (Some(due), Some(stop)) = (s.walk.due, s.walk.seen.clone()) else { return false };
    if at < due {
        return false;
    }
    s.walk.due = None;
    s.walk.last_thread = trail::last_thread_after(s.walk.last_thread.take(), &stop);
    let step = s.walk.stepping.take();
    s.walk.trail = trail::settle(&s.walk.trail, &stop, step.as_ref());
    // The arrows' looks follow the trail, so the page is drawn again.
    true
}
