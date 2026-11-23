//! The window's repeating timers, each asked of Rune once.
//!
//! Rune keeps every wake it is asked for and gives each one a turn of the window (`poll`). A
//! timer that repeats is asked for from that turn, and asking for it again on every turn, as
//! `tx.wake_after(period)` reads, plants one more wake a period ahead each time: a second of
//! frames, drawn for any reason, comes back as a second of turns a period later, and each of
//! those asks again. With a clock that turns every twenty seconds, a conversation read again
//! every eight and pictures looked at every two, the window soon had a timer due on every
//! frame. It then never rested, and no frame could be the step Rune draws for a loop alone,
//! which runs no turn at all: a working line cost a whole frame sixty times a second.
//!
//! So a repeating timer is asked for through `every`, which asks only when the one before has
//! come due. A wake for a moment already known (a line that goes at a time, a write that
//! waits three seconds) is asked for as it was: every asking names the same moment, and Rune
//! gives all that are due together one turn.

use std::collections::HashMap;
use std::time::Duration;

use rune::Tx;

/// The moments the window's repeating timers are next due, on the window's clock.
#[derive(Default)]
pub struct Wakes {
    due: HashMap<&'static str, f64>,
}

impl Wakes {
    /// Wakes the window `period` from now for `what`, unless a wake for it is still on its way.
    pub fn every(&mut self, tx: &mut Tx, what: &'static str, period: Duration) {
        let now = tx.clock();
        if self.waits(what, now) {
            return;
        }
        let due = now + period.as_secs_f64();
        self.due.insert(what, due);
        tx.wake_at(due);
    }

    /// Whether a wake asked for `what` has still to come. One due within a thousandth of a
    /// second counts as come: the turn it wakes may read the clock a hair before it.
    fn waits(&self, what: &str, now: f64) -> bool {
        self.due.get(what).is_some_and(|due| *due > now + 0.001)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_repeating_timer_waits_until_the_one_before_has_come_due() {
        let mut wakes = Wakes::default();
        assert!(!wakes.waits("clock", 0.0));
        wakes.due.insert("clock", 20.0);
        // Every turn before it is due finds it on its way, and asks for nothing.
        assert!(wakes.waits("clock", 0.016));
        assert!(wakes.waits("clock", 19.9));
        // The turn it wakes, and any after, asks again.
        assert!(!wakes.waits("clock", 19.9995));
        assert!(!wakes.waits("clock", 20.0));
        assert!(!wakes.waits("clock", 31.0));
        // One timer says nothing of another.
        assert!(!wakes.waits("pictures", 1.0));
    }
}
