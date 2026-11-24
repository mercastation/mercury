//! E AND Z ALWAYS SAY WHAT THEY DID (`tests/e-and-z-always-say-what-they-did.test.mts`): E on a
//! list where it closes nothing says why (`eOnThisList` in `renderer/src/keys.ts`); the press
//! after the waiting agent's warning goes ahead only on the thread it warned of
//! (`pressAgainGoesAhead` in `renderer/src/stopped-agent.ts`); and two quick Zs are one Z, not
//! nothing (`renderer/src/stray-letters.ts`).

/// What E says on a list where it closes nothing, so the press is never silent: Done's rows are
/// closed already, and In progress's are closed from inside them. `view` is the web window's
/// name for the tab (`progress`, `done`).
pub fn e_on_this_list(view: &str) -> &'static str {
    if view == "done" { "This thread is already done." } else { "Open this thread to close it." }
}

/// "Press again to go ahead" is about the thread it warned of: E on another stopped thread is
/// warned about too. `held_for` is the thread the last warning was about, `product:id`.
pub fn press_again_goes_ahead(held_for: Option<&str>, product: &str, id: &str) -> bool {
    held_for == Some(format!("{product}:{id}").as_str())
}

/// How long the web window holds a Z to see whether it starts a word (`BURST_MS`), during which
/// a second Z is the same press said twice.
pub const BURST_MS: f64 = 250.0;

/// A Z this soon after the last Z that went through is that press again, and does nothing more.
/// Times in milliseconds.
pub fn z_said_twice(last_z: Option<f64>, now: f64) -> bool {
    last_z.is_some_and(|at| now - at < BURST_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e_on_a_list_where_it_closes_nothing_says_why_on_in_progress_and_on_done() {
        assert_eq!(e_on_this_list("progress"), "Open this thread to close it.");
        assert_eq!(e_on_this_list("done"), "This thread is already done.");
    }

    #[test]
    fn the_press_after_a_stuck_agent_warning_goes_ahead_only_on_the_thread_it_warned_about() {
        assert!(press_again_goes_ahead(Some("p:w-a"), "p", "w-a"));
        assert!(!press_again_goes_ahead(Some("p:w-a"), "p", "w-b"));
        assert!(!press_again_goes_ahead(None, "p", "w-a"));
    }

    #[test]
    fn two_quick_zs_reach_the_app_as_one_z() {
        assert!(!z_said_twice(None, 50_000.0));
        assert!(z_said_twice(Some(50_000.0), 50_080.0));
        assert!(!z_said_twice(Some(50_000.0), 50_000.0 + BURST_MS + 10.0));
    }
}
