//! Back and forward, as the web window walks them (`renderer/src/back-and-forward.ts`): a
//! trail of the places the person stood on the inbox page, its list and each thread once,
//! where they were in it last, so the same visits never walk round in circles. The arrows are
//! drawn on the inbox page alone, and ⌘[ and ⌘] step only there.
//!
//! The web window's trail also holds a teammate's thread and a chat opened from Needs you;
//! this window draws neither, so a place here is the list or a thread of the person's own.

use crate::data::Tab;

/// A place on the inbox page the arrows can take the person back to (`Stop`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stop {
    /// The list, in the tab it was on.
    List(Tab),
    Thread { product: String, id: String },
}

impl Stop {
    fn is_list(&self) -> bool {
        matches!(self, Stop::List(_))
    }
}

/// The places stood on, oldest first, and which one is on screen, or the last one counted
/// while a new one settles (`Trail`). `at` is past nothing while the trail is empty.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Trail {
    pub stops: Vec<Stop>,
    pub at: Option<usize>,
}

/// How many places it keeps (`TRAIL_LIMIT`).
pub const LIMIT: usize = 50;
/// How long a place has to stay on screen before it counts as a visit, in seconds
/// (`SETTLE_MS`): one that flashed past on the way to the next, the list as E closes threads
/// one after another, is not somewhere the person was.
pub const SETTLE: f64 = 0.3;

/// `samePlace`: the list is one place whichever tab it is on.
pub fn same_place(a: &Stop, b: &Stop) -> bool {
    (a.is_list() && b.is_list()) || a == b
}

impl Trail {
    fn here(&self) -> Option<&Stop> {
        self.at.and_then(|at| self.stops.get(at))
    }
}

/// `arrive`: a place the person went to. A thread is kept once, where they were in it last,
/// so two threads opened in turn are two steps back rather than a back and forth for ever. A
/// tab picked on the list is where they are on it, not a step.
pub fn arrive(trail: &Trail, stop: &Stop) -> Trail {
    let here = trail.here();
    if here == Some(stop) {
        return trail.clone();
    }
    if here.is_some_and(Stop::is_list) && stop.is_list() {
        return land(trail, stop);
    }
    // The place ahead, opened again by hand, keeps the way forward.
    let next = trail.at.map_or(0, |at| at + 1);
    if trail.stops.get(next).is_some_and(|ahead| same_place(ahead, stop)) {
        return land(&Trail { stops: trail.stops.clone(), at: Some(next) }, stop);
    }
    let kept: Vec<&Stop> = trail.stops.iter().take(next).filter(|s| s.is_list() || *s != stop).collect();
    // A thread taken out from between two lists leaves one visit to the list.
    let mut stops: Vec<Stop> = kept.iter().enumerate().filter(|(i, s)| !(s.is_list() && kept.get(i + 1).is_some_and(|after| after.is_list()))).map(|(_, s)| (*s).clone()).collect();
    stops.push(stop.clone());
    let extra = stops.len().saturating_sub(LIMIT);
    let stops = stops.split_off(extra);
    Trail { at: Some(stops.len() - 1), stops }
}

/// `land`: a step showed `stop` rather than the place it was going to, so that is the place
/// it is on.
pub fn land(trail: &Trail, stop: &Stop) -> Trail {
    let Some(at) = trail.at.filter(|at| trail.stops.get(*at).is_some_and(|here| here != stop)) else { return trail.clone() };
    let mut stops = trail.stops.clone();
    stops[at] = stop.clone();
    Trail { stops, at: Some(at) }
}

/// A back or a forward on its way to `to`, and whether its place was shown (`stepping`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub to: Stop,
    pub reached: bool,
}

/// `settle`: the trail once `stop` has stayed on screen.
pub fn settle(trail: &Trail, stop: &Stop, step: Option<&Step>) -> Trail {
    match step {
        None => arrive(trail, stop),
        Some(step) if !step.reached => land(trail, stop),
        Some(step) if &step.to == stop => trail.clone(),
        Some(_) => arrive(trail, stop),
    }
}

fn step(trail: &Trail, forward: bool, here: Option<&Stop>, exists: &dyn Fn(&Stop) -> bool) -> Option<(Trail, Stop)> {
    let standing = here.is_some_and(|here| trail.here().is_some_and(|at| same_place(at, here)));
    // A place not counted yet drops what is ahead once it is.
    if forward && here.is_some() && !standing {
        return None;
    }
    let at = trail.at.map_or(-1, |at| at as isize);
    // From a place not counted yet, back goes first to where the person was.
    let mut i = if forward { at + 1 } else if standing { at - 1 } else { at };
    while i >= 0 && (i as usize) < trail.stops.len() {
        let s = &trail.stops[i as usize];
        if exists(s) && !here.is_some_and(|here| same_place(s, here)) {
            return Some((Trail { stops: trail.stops.clone(), at: Some(i as usize) }, s.clone()));
        }
        i += if forward { 1 } else { -1 };
    }
    None
}

/// `stepBack` and `stepForward`: where the arrow leads from `here`, the place on screen, and
/// the trail once it has been taken; none when there is nowhere to go. `exists` says whether a
/// place can still be opened.
pub fn step_back(trail: &Trail, here: Option<&Stop>, exists: &dyn Fn(&Stop) -> bool) -> Option<(Trail, Stop)> {
    step(trail, false, here, exists)
}
pub fn step_forward(trail: &Trail, here: Option<&Stop>, exists: &dyn Fn(&Stop) -> bool) -> Option<(Trail, Stop)> {
    step(trail, true, here, exists)
}

/// `lastThreadAfter`: the last thread of the person's own they stood on, once they have stood
/// on `stop`. Its row in the list is washed, faintly.
pub fn last_thread_after(last: Option<(String, String)>, stop: &Stop) -> Option<(String, String)> {
    match stop {
        Stop::Thread { product, id } => Some((product.clone(), id.clone())),
        Stop::List(_) => last,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // `tests/back-on-the-inbox-walks-back-through-the-list-and-each-thread-once.test.mts`.
    fn list() -> Stop {
        Stop::List(Tab::Needs)
    }
    fn thread(id: &str) -> Stop {
        Stop::Thread { product: "p".to_string(), id: id.to_string() }
    }
    /// Every place gone to by hand, in order, each staying long enough to count.
    fn walk(stops: &[Stop]) -> Trail {
        stops.iter().fold(Trail::default(), |t, s| arrive(&t, s))
    }
    /// Presses back or forward `n` times from `on`, and says where each press landed.
    fn press(forward: bool, n: usize, mut t: Trail, mut on: Option<Stop>) -> (Vec<Stop>, Trail, Option<Stop>) {
        let mut landed = Vec::new();
        for _ in 0..n {
            let Some((trail, stop)) = step(&t, forward, on.as_ref(), &|_| true) else { break };
            landed.push(stop.clone());
            t = trail;
            on = Some(stop);
        }
        (landed, t, on)
    }
    fn back(n: usize, t: Trail, on: Stop) -> Vec<Stop> {
        press(false, n, t, Some(on)).0
    }

    #[test]
    fn back_from_a_thread_goes_to_the_list_then_the_thread_before_it() {
        let (l, a, b, c) = (list(), thread("a"), thread("b"), thread("c"));
        assert_eq!(back(5, walk(&[l.clone(), a.clone()]), a.clone()), [l.clone()]);
        let all = [l.clone(), a.clone(), l.clone(), b.clone(), l.clone(), c.clone()];
        assert_eq!(back(9, walk(&all), c), [l.clone(), b, l.clone(), a, l]);
    }

    #[test]
    fn back_never_goes_round_in_circles() {
        let (l, a, b, c) = (list(), thread("a"), thread("b"), thread("c"));
        // Two threads each opened from the list twice are one step each.
        let twice = [l.clone(), a.clone(), l.clone(), b.clone(), l.clone(), a.clone(), l.clone(), b.clone()];
        assert_eq!(back(9, walk(&twice), b.clone()), [l.clone(), a.clone(), l.clone()]);
        // A thread checked three times is one step.
        let thrice = [l.clone(), a.clone(), l.clone(), a.clone(), l.clone(), a.clone()];
        assert_eq!(back(9, walk(&thrice), a.clone()), [l.clone()]);
        // Two threads stepped between in the thread view are one step each.
        assert_eq!(back(9, walk(&[l.clone(), a.clone(), b.clone(), a.clone(), b.clone()]), b.clone()), [a.clone(), l.clone()]);
        // A thread opened again is remembered where it was opened last.
        let again = [l.clone(), a.clone(), l.clone(), b.clone(), l.clone(), c.clone(), l.clone(), a.clone()];
        assert_eq!(back(9, walk(&again), a), [l.clone(), c, l.clone(), b, l]);
    }

    #[test]
    fn it_holds_for_any_order_of_visits_backs_and_forwards() {
        // mulberry32, as the web's test seeds it, so a failure replays.
        let mut seed: i32 = 7;
        let mut random = |n: u32| -> u32 {
            seed = seed.wrapping_add(0x6d2b79f5_u32 as i32);
            let mut x = ((seed ^ ((seed as u32) >> 15) as i32) as u32).wrapping_mul((1 | seed) as u32);
            x = (x.wrapping_add((x ^ (x >> 7)).wrapping_mul(61 | x))) ^ x;
            ((f64::from(x ^ (x >> 14)) / 4_294_967_296.0) * f64::from(n)).floor() as u32
        };
        let places = [thread("a"), thread("b"), thread("c"), thread("d"), list(), list(), Stop::List(Tab::Progress), Stop::List(Tab::Done)];
        for _ in 0..400 {
            let mut on = list();
            let mut t = arrive(&Trail::default(), &on);
            for _ in 0..30 {
                let pick = random(12) as usize;
                if pick < places.len() {
                    on = places[pick].clone();
                    t = arrive(&t, &on);
                } else if let Some((trail, stop)) = step(&t, pick % 2 == 0, Some(&on), &|_| true) {
                    t = trail;
                    on = stop;
                }
                let kept: Vec<&Stop> = t.stops.iter().filter(|s| !s.is_list()).collect();
                assert!(kept.iter().enumerate().all(|(i, s)| kept.iter().position(|o| o == s) == Some(i)), "a thread is kept once: {t:?}");
                assert!(t.stops.windows(2).all(|w| !(w[0].is_list() && w[1].is_list())), "never two lists in a row: {t:?}");
                assert!(same_place(t.here().expect("somewhere"), &on));
                let landed = back(99, t.clone(), on.clone());
                let mut was = on.clone();
                for s in &landed {
                    assert!(!same_place(s, &was), "back never stands still: {landed:?}");
                    was = s.clone();
                }
                let threads: Vec<&Stop> = landed.iter().filter(|s| !s.is_list()).collect();
                assert!(threads.iter().enumerate().all(|(i, s)| threads.iter().position(|o| o == s) == Some(i)), "each thread once: {landed:?}");
            }
        }
    }

    #[test]
    fn the_list_is_one_place_in_any_tab() {
        let (l, a) = (list(), thread("a"));
        // Back from the list goes first to the thread just left.
        assert_eq!(step_back(&walk(&[l.clone(), a.clone(), l.clone()]), Some(&l), &|_| true).map(|s| s.1), Some(a.clone()));
        // It keeps the tab picked on it, as one step rather than one per tab.
        let t = walk(&[l.clone(), Stop::List(Tab::Progress), Stop::List(Tab::Done), a.clone()]);
        assert_eq!(t.stops, [Stop::List(Tab::Done), a.clone()]);
        assert_eq!(back(5, t, a.clone()), [Stop::List(Tab::Done)]);
        // The same place in any tab, so back from it is never a step to another tab.
        assert!(same_place(&l, &Stop::List(Tab::Progress)));
        assert_ne!(l, Stop::List(Tab::Progress));
        assert_eq!(step_back(&walk(&[l.clone(), a.clone(), l]), Some(&Stop::List(Tab::Progress)), &|_| true).map(|s| s.1), Some(a));
    }

    #[test]
    fn forward_walks_back_the_way_back_came() {
        let (l, a, b, c, d) = (list(), thread("a"), thread("b"), thread("c"), thread("d"));
        let all = [l.clone(), a.clone(), l.clone(), b.clone(), l.clone(), c.clone()];
        let (landed, trail, on) = press(false, 3, walk(&all), Some(c.clone()));
        assert_eq!(landed, [l.clone(), b.clone(), l.clone()]);
        assert_eq!(press(true, 9, trail, on).0, [b.clone(), l.clone(), c.clone()]);
        // It still goes on after Escape from a thread back took the person to.
        let (_, trail, on) = press(false, 4, walk(&all), Some(c.clone()));
        assert_eq!(on, Some(a.clone()));
        let esc = arrive(&trail, &l);
        assert_eq!(press(true, 9, esc, Some(l.clone())).0, [b.clone(), l.clone(), c.clone()]);
        // It is gone once somewhere new is opened after going back.
        let (_, trail, _) = press(false, 2, walk(&all), Some(c));
        let next = arrive(&trail, &d);
        assert_eq!(next.stops, [l.clone(), a.clone(), l.clone(), b.clone(), d.clone()]);
        assert_eq!(step_forward(&next, Some(&d), &|_| true), None);
        // And there is nothing ahead of a place that has not counted yet.
        assert_eq!(step_forward(&walk(&[l.clone(), a, l, b]), Some(&d), &|_| true), None);
    }

    #[test]
    fn the_trail_has_nowhere_to_go_at_either_end_and_skips_what_is_gone() {
        let (l, a, b, d) = (list(), thread("a"), thread("b"), thread("d"));
        let any = |_: &Stop| true;
        assert_eq!(step_back(&Trail::default(), Some(&l), &any), None);
        assert_eq!(step_forward(&Trail::default(), Some(&l), &any), None);
        assert_eq!(step_back(&walk(&[l.clone()]), Some(&l), &any), None);
        assert_eq!(step_forward(&walk(&[l.clone(), a.clone()]), Some(&a), &any), None);
        // From a thread just opened, before it counts, back goes to where the person was.
        assert_eq!(step_back(&walk(&[l.clone(), a.clone(), l.clone()]), Some(&d), &any).map(|s| s.1), Some(l.clone()));
        // A thread that is gone is skipped, and so is a list it would leave the person on twice.
        let all = [l.clone(), a.clone(), l.clone(), b.clone()];
        assert_eq!(back(9, walk(&all), b.clone()), [l.clone(), a.clone(), l.clone()]);
        let gone = |s: &Stop| *s != a;
        let (mut t, mut on, mut landed) = (walk(&all), b, Vec::new());
        while let Some((trail, stop)) = step_back(&t, Some(&on), &gone) {
            landed.push(stop.clone());
            t = trail;
            on = stop;
        }
        assert_eq!(landed, [l.clone()]);
        // It keeps the newest places when it is long.
        let mut t = Trail::default();
        for i in 0..LIMIT {
            t = arrive(&arrive(&t, &l), &thread(&i.to_string()));
        }
        assert_eq!(t.stops.len(), LIMIT);
        assert_eq!(t.stops.last(), Some(&thread(&(LIMIT - 1).to_string())));
        assert_eq!(t.at, Some(LIMIT - 1));
        // Places are told apart by project.
        assert_ne!(thread("a"), Stop::Thread { product: "q".to_string(), id: "a".to_string() });
        assert!(!same_place(&l, &thread("a")));
    }

    #[test]
    fn a_step_that_has_landed_is_not_a_new_visit() {
        let (l, a, b, d) = (list(), thread("a"), thread("b"), thread("d"));
        let t = press(false, 1, walk(&[l.clone(), a.clone(), l.clone(), b.clone()]), Some(b.clone())).1;
        // Its own place stayed on screen: nothing new.
        assert_eq!(settle(&t, &l, Some(&Step { to: l.clone(), reached: true })), t);
        // It showed somewhere else instead: that place takes its own on the trail, and the way
        // forward is kept.
        let progress = Stop::List(Tab::Progress);
        let landed = settle(&t, &progress, Some(&Step { to: l.clone(), reached: false }));
        assert_eq!(landed, land(&t, &progress));
        assert_eq!(landed.stops, [l.clone(), a, progress.clone(), b.clone()]);
        assert_eq!(step_forward(&landed, Some(&progress), &|_| true).map(|s| s.1), Some(b));
        // Moved on from before it counted: a new visit. And an ordinary one when no step was taken.
        assert_eq!(settle(&t, &d, Some(&Step { to: l, reached: true })), arrive(&t, &d));
        assert_eq!(settle(&t, &d, None), arrive(&t, &d));
    }

    // `tests/the-last-thread-you-were-in-is-washed-in-the-list.test.mts`.
    #[test]
    fn the_last_thread_stood_on_is_the_one_just_left_and_only_the_most_recent() {
        let stood = |stops: &[Stop]| stops.iter().fold(None, last_thread_after);
        let own = |id: &str| Some(("p".to_string(), id.to_string()));
        assert_eq!(stood(&[thread("a")]), own("a"));
        assert_eq!(stood(&[thread("a"), thread("b")]), own("b"));
        assert_eq!(stood(&[]), None);
        // The list is not a stop, and the thread back or forward led into is.
        assert_eq!(stood(&[thread("a"), list()]), own("a"));
        assert_eq!(stood(&[thread("a"), thread("b"), thread("a")]), own("a"));
    }
}
