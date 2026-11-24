//! When the folded sidebar opens and when it folds again (`renderer/src/sidebar-shrinks.ts`).
//! It opens 140 ms after a pointer that came into it from outside last stopped travelling, so a
//! hand that overshoots a tab beside it and swings back never opens it, and neither does a
//! sidebar that slid under a pointer standing still. It folds 300 ms after the pointer leaves.

/// How long the pointer rests on the folded sidebar before it opens, and how long after
/// leaving before it folds, in seconds (`OPENS_AFTER_MS`, `CLOSES_AFTER_MS`).
pub const OPENS_AFTER: f64 = 0.140;
pub const CLOSES_AFTER: f64 = 0.300;
/// Faster than this, in points a millisecond, the pointer is still on its way and not resting
/// (`RESTS_BELOW_SPEED`).
pub const RESTS_BELOW_SPEED: f64 = 0.15;

/// Where the pointer was, and when in milliseconds when that is known.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct At {
    pub x: f32,
    pub y: f32,
    pub t: Option<f64>,
}

/// Where a pointer that left the window is taken to be (`OUT_OF_THE_WINDOW`).
pub const OUT_OF_THE_WINDOW: At = At { x: f32::NEG_INFINITY, y: f32::NEG_INFINITY, t: None };

/// The sidebar's box in the window.
#[derive(Clone, Copy, Debug)]
pub struct Sides {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// `crossedIn`: whether the pointer was outside the sidebar before this move. A sidebar that
/// slid under the pointer gets nudged, not entered, and stays shut. Nothing seen yet counts as
/// not crossed.
pub fn crossed_in(from: Option<At>, sides: &Sides) -> bool {
    from.is_some_and(|from| from.x < sides.left || from.x >= sides.right || from.y < sides.top || from.y >= sides.bottom)
}

/// `stillTravelling`: travelling with no earlier move, or one from outside the window; else by
/// its speed between the two.
pub fn still_travelling(from: Option<At>, to: At) -> bool {
    let (Some(from), Some(to_t)) = (from, to.t) else { return true };
    let Some(from_t) = from.t.filter(|_| from.x.is_finite()) else { return true };
    let dt = to_t - from_t;
    let far = f64::from((to.x - from.x).hypot(to.y - from.y));
    if dt > 0.0 { far / dt > RESTS_BELOW_SPEED } else { far > 0.0 }
}

/// `hoverIntent`: whether to open the folded sidebar.
#[derive(Clone, Copy, Debug, Default)]
pub struct Intent {
    came_in: bool,
    waiting: bool,
}

impl Intent {
    /// A move over the sidebar: answers whether the wait to open starts (over) now.
    pub fn moved(&mut self, before: Option<At>, now: At, sides: &Sides) -> bool {
        self.came_in |= crossed_in(before, sides);
        if !self.came_in {
            return false;
        }
        if self.waiting && !still_travelling(before, now) {
            return false;
        }
        self.waiting = true;
        true
    }
    /// The pointer left the sidebar.
    pub fn left(&mut self) {
        *self = Intent::default();
    }
}

/// `pointerTrail`: where the pointer was one move ago, and where it is.
#[derive(Clone, Copy, Debug, Default)]
pub struct Trail {
    before: Option<At>,
    now: Option<At>,
}

impl Trail {
    pub fn moved(&mut self, x: f32, y: f32, t: f64) {
        self.before = self.now;
        self.now = Some(At { x, y, t: Some(t) });
    }
    /// The pointer went out of the window.
    pub fn left(&mut self) {
        self.now = Some(OUT_OF_THE_WINDOW);
    }
    pub fn before(&self) -> Option<At> {
        self.before
    }
    pub fn now(&self) -> Option<At> {
        self.now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // `tests/the-sidebar-opens-only-once-the-pointer-rests-on-it.test.mts` and
    // `tests/the-sidebar-opens-only-for-a-pointer-that-came-into-it.test.mts`.
    const RAIL: Sides = Sides { left: 14.0, top: 32.0, right: 42.0, bottom: 878.0 };
    const NEEDS_YOU: (f64, f64) = (126.0, 115.0);

    struct Leg {
        to: (f64, f64),
        ms: f64,
        stop: f64,
    }
    fn leg(to: (f64, f64), ms: f64, stop: f64) -> Leg {
        Leg { to, ms, stop }
    }

    /// A hand's move: each leg eases in and out, one sample every 8 ms, with a stop after it.
    fn hand(start: (f64, f64), legs: &[Leg]) -> Vec<(f64, f64, f64)> {
        let smooth = |t: f64| 10.0 * t.powi(3) - 15.0 * t.powi(4) + 6.0 * t.powi(5);
        let (mut out, mut at, mut t) = (Vec::new(), start, 0.0);
        for leg in legs {
            let from = at;
            let n = (leg.ms / 8.0).round() as usize;
            for i in 1..=n {
                t += 8.0;
                let k = smooth(i as f64 / n as f64);
                at = (from.0 + (leg.to.0 - from.0) * k, from.1 + (leg.to.1 - from.1) * k);
                out.push((t, at.0, at.1));
            }
            t += leg.stop;
        }
        out
    }

    /// Runs the moves through the rule the sidebar asks, with its wait and its leave.
    fn opens(samples: &[(f64, f64, f64)]) -> bool {
        let (mut trail, mut intent) = (Trail::default(), Intent::default());
        let (mut due, mut inside): (Option<f64>, bool) = (None, false);
        for &(t, x, y) in samples {
            if due.is_some_and(|due| t >= due) {
                return true;
            }
            let (x, y) = (x as f32, y as f32);
            trail.moved(x, y, t);
            let over = x >= RAIL.left && x < RAIL.right && y >= RAIL.top && y < RAIL.bottom;
            if !over {
                if inside {
                    intent.left();
                    due = None;
                }
                inside = false;
                continue;
            }
            inside = true;
            if intent.moved(trail.before(), At { x, y, t: Some(t) }, &RAIL) {
                due = Some(t + OPENS_AFTER * 1000.0);
            }
        }
        due.is_some()
    }
    fn overshoot(turn_at: f64, stop: f64) -> Vec<(f64, f64, f64)> {
        hand((800.0, 400.0), &[leg((turn_at, NEEDS_YOU.1), 400.0, stop), leg(NEEDS_YOU, 250.0, 0.0)])
    }

    #[test]
    fn a_hand_going_for_needs_you_leaves_it_shut_unless_it_stops_on_the_sidebar() {
        assert!(!opens(&overshoot(30.0, 0.0)), "overshoots and swings straight back");
        assert!(!opens(&overshoot(26.0, 0.0)));
        assert!(!opens(&overshoot(26.0, 40.0)), "a short beat at the turn");
        assert!(opens(&overshoot(22.0, 160.0)), "the hand stops on the sidebar");
        assert!(!opens(&hand((700.0, 300.0), &[leg((2.0, 300.0), 350.0, 0.0)])), "passing over on the way to the window's edge");
    }

    #[test]
    fn a_hand_going_for_the_sidebar_opens_it() {
        assert!(opens(&hand((700.0, 300.0), &[leg((28.0, 235.0), 450.0, 0.0)])), "pointing at an icon and stopping");
        assert!(opens(&hand((700.0, 600.0), &[leg((28.0, 600.0), 450.0, 0.0), leg((28.0, 200.0), 1400.0, 0.0)])), "sliding slowly along the icons");
        // And still never one that slid under the pointer and was nudged.
        let mut intent = Intent::default();
        assert!(!intent.moved(Some(At { x: 34.0, y: 855.0, t: Some(0.0) }), At { x: 35.0, y: 855.0, t: Some(16.0) }, &RAIL));
    }

    #[test]
    fn travelling_or_resting() {
        let at = |x: f64, t: f64| At { x: x as f32, y: 0.0, t: Some(t) };
        assert!(still_travelling(Some(at(0.0, 0.0)), at(RESTS_BELOW_SPEED * 10.0 + 0.1, 10.0)), "just over the speed");
        assert!(!still_travelling(Some(at(0.0, 0.0)), at(RESTS_BELOW_SPEED * 10.0 - 0.1, 10.0)), "just under it");
        // With no earlier move, or one from outside the window.
        assert!(still_travelling(None, At { x: 20.0, y: 300.0, t: Some(5.0) }));
        assert!(still_travelling(Some(OUT_OF_THE_WINDOW), At { x: 20.0, y: 300.0, t: Some(5.0) }));
    }

    #[test]
    fn a_pointer_counts_as_having_come_in_only_from_outside_the_box() {
        let inside = At { x: 20.0, y: 100.0, t: Some(0.0) };
        assert!(!crossed_in(None, &RAIL), "nothing seen yet counts as not crossed");
        assert!(!crossed_in(Some(inside), &RAIL));
        assert!(crossed_in(Some(At { x: 60.0, ..inside }), &RAIL));
        assert!(crossed_in(Some(At { x: 42.0, ..inside }), &RAIL), "the right edge is outside");
        assert!(crossed_in(Some(OUT_OF_THE_WINDOW), &RAIL));
        // Once in, it stays in until it leaves.
        let mut intent = Intent::default();
        assert!(intent.moved(Some(At { x: 60.0, ..inside }), inside, &RAIL));
        intent.left();
        assert!(!intent.moved(Some(inside), At { x: 21.0, ..inside }, &RAIL));
    }

    #[test]
    fn the_trail_keeps_the_move_before_this_one_and_a_pointer_gone_is_outside() {
        let mut trail = Trail::default();
        assert_eq!(trail.before(), None);
        trail.moved(1.0, 2.0, 10.0);
        assert_eq!(trail.before(), None);
        trail.moved(3.0, 4.0, 20.0);
        assert_eq!(trail.before(), Some(At { x: 1.0, y: 2.0, t: Some(10.0) }));
        trail.left();
        trail.moved(5.0, 6.0, 30.0);
        assert_eq!(trail.before(), Some(OUT_OF_THE_WINDOW));
    }
}
