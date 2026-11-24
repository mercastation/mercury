//! Which turn carries the threads a run filed: a port of `renderer/src/filed-in-thread.ts`.
//! Each filed thread hangs off the end of the agent's turn that was being spoken when it was
//! filed, which is where the agent came back carrying it; the newest turn's last word is drawn
//! under the conversation, so what it filed stays at the foot until anything else is said.

use std::collections::BTreeMap;

/// As much of a conversation's event as this rule reads: when, whether it is a line of what
/// happened rather than something said, and whether the person said it.
#[derive(Clone, Copy, Debug)]
pub struct Spoken {
    pub at: f64,
    pub work: bool,
    pub mine: bool,
}

/// The threads drawn at the end of one turn, by that message's index, and those whose turn is
/// the newest, drawn under the conversation (`FiledInThread`).
#[derive(Clone, Debug, PartialEq)]
pub struct FiledInThread<T> {
    pub on_turn: BTreeMap<usize, Vec<T>>,
    pub at_foot: Vec<T>,
}

/// Where each message is the agent's last word before the person spoke again. The last message
/// of the whole conversation is left out: its turn is still the newest (`turnEnds`).
fn turn_ends(events: &[Spoken]) -> Vec<usize> {
    let said: Vec<(usize, &Spoken)> = events.iter().enumerate().filter(|(_, e)| !e.work).collect();
    said.windows(2).filter(|w| !w[0].1.mine && w[1].1.mine).map(|w| w[0].0).collect()
}

/// Each filed thread on its turn (`filedOnTurn`); `at` is when each was filed.
pub fn filed_on_turn<T: Clone>(events: &[Spoken], rows: &[T], at: impl Fn(&T) -> f64) -> FiledInThread<T> {
    let ends = turn_ends(events);
    let mut sorted: Vec<&T> = rows.iter().collect();
    sorted.sort_by(|a, b| at(a).total_cmp(&at(b)));
    let mut made = FiledInThread { on_turn: BTreeMap::new(), at_foot: Vec::new() };
    for row in sorted {
        match ends.iter().find(|i| events[**i].at >= at(row)) {
            Some(end) => made.on_turn.entry(*end).or_default().push(row.clone()),
            None => made.at_foot.push(row.clone()),
        }
    }
    made
}

/// The turn whose list the close question stands under: the newest one with a task waiting on
/// the person, or the foot of the thread (`askTurnOf`, whose -1 is `None`).
pub fn ask_turn_of<T>(made: &FiledInThread<T>, waits: impl Fn(&T) -> bool) -> Option<usize> {
    if made.at_foot.iter().any(&waits) {
        return None;
    }
    made.on_turn.iter().filter(|(_, rows)| rows.iter().any(&waits)).map(|(at, _)| *at).max()
}

#[cfg(test)]
mod tests {
    //! The web's cases, from `tests/the-threads-a-thread-filed-sit-on-the-turn-that-filed-them`
    //! and `the-close-question-stands-under-the-task-it-is-about` (`.test.mts`).
    use super::*;

    const T: f64 = 1_791_190_800_000.0;
    fn min(n: f64) -> f64 {
        T + n * 60_000.0
    }
    fn you(n: f64) -> Spoken {
        Spoken { at: min(n), work: false, mine: true }
    }
    fn it(n: f64) -> Spoken {
        Spoken { at: min(n), work: false, mine: false }
    }
    fn work(n: f64) -> Spoken {
        Spoken { at: min(n), work: true, mine: false }
    }
    /// Split the redesign up; reading; three threads filed; the chat layout asked for; built.
    fn conversation() -> Vec<Spoken> {
        vec![you(0.0), work(1.0), it(2.0), it(5.0), you(30.0), work(31.0), it(40.0)]
    }
    fn filed(id: &'static str, at: f64) -> (&'static str, f64) {
        (id, at)
    }
    fn on(events: &[Spoken], rows: &[(&'static str, f64)]) -> FiledInThread<(&'static str, f64)> {
        filed_on_turn(events, rows, |r| r.1)
    }
    fn names(rows: &[(&'static str, f64)]) -> Vec<&'static str> {
        rows.iter().map(|r| r.0).collect()
    }

    #[test]
    fn they_hang_off_the_last_message_of_the_turn_that_filed_them() {
        let made = on(&conversation(), &[filed("w-a", min(4.0)), filed("w-b", min(4.0))]);
        assert_eq!(made.on_turn.keys().copied().collect::<Vec<_>>(), [3]);
        assert_eq!(names(&made.on_turn[&3]), ["w-a", "w-b"]);
        assert!(made.at_foot.is_empty());
        let one = on(&conversation(), &[filed("w-a", min(4.0))]);
        assert!(!one.on_turn.contains_key(&2) && one.at_foot.is_empty());
    }

    #[test]
    fn the_newest_turns_stay_at_the_foot() {
        for at in [35.0, 99.0] {
            let made = on(&conversation(), &[filed("w-c", min(at))]);
            assert!(made.on_turn.is_empty());
            assert_eq!(names(&made.at_foot), ["w-c"]);
        }
        let both = on(&conversation(), &[filed("w-a", min(4.0)), filed("w-c", min(35.0))]);
        assert_eq!((names(&both.on_turn[&3]), names(&both.at_foot)), (vec!["w-a"], vec!["w-c"]));
        for never in [0, 1, 4, 5] {
            assert!(!both.on_turn.contains_key(&never));
        }
    }

    #[test]
    fn a_turns_rows_are_oldest_first_and_nothing_filed_is_nothing() {
        let made = on(&conversation(), &[filed("w-late", min(5.0)), filed("w-early", min(3.0))]);
        assert_eq!(names(&made.on_turn[&3]), ["w-early", "w-late"]);
        let none = on(&conversation(), &[]);
        assert!(none.on_turn.is_empty() && none.at_foot.is_empty());
        let alone = on(&[you(0.0)], &[filed("w-a", min(4.0))]);
        assert!(alone.on_turn.is_empty());
        assert_eq!(names(&alone.at_foot), ["w-a"]);
    }

    #[test]
    fn the_close_question_stands_under_the_newest_turn_with_a_task_waiting() {
        let made = |on_turn: Vec<(usize, Vec<(&'static str, bool)>)>, at_foot: Vec<(&'static str, bool)>| FiledInThread { on_turn: on_turn.into_iter().collect(), at_foot };
        let waits = |r: &(&str, bool)| r.1;
        assert_eq!(ask_turn_of(&made(vec![(3, vec![("a", true)])], vec![]), waits), Some(3));
        assert_eq!(ask_turn_of(&made(vec![(3, vec![("a", true)]), (9, vec![("b", true)]), (6, vec![("c", false)])], vec![]), waits), Some(9));
        assert_eq!(ask_turn_of(&made(vec![(3, vec![("a", true)]), (9, vec![("b", false)])], vec![]), waits), Some(3));
        assert_eq!(ask_turn_of(&made(vec![(3, vec![("a", true)])], vec![("b", true)]), waits), None);
        assert_eq!(ask_turn_of(&made(vec![(3, vec![("a", false)])], vec![]), waits), None);
        assert_eq!(ask_turn_of(&made(vec![], vec![]), waits), None);
    }
}
