//! The sidebar's Recent: the threads the person last had open, newest first
//! (`renderer/src/recent-threads.ts`), and the sections of the sidebar a press on their name
//! folds away (`SectionHead` in `renderer/src/components/WorkspaceNavigation.tsx`).

use std::collections::HashMap;

use crate::data::Item;

/// How many it lists (`RECENT_LIMIT`).
pub const LIMIT: usize = 6;

/// The sections that fold, by the names the web window keeps them under
/// (`agentbox.sidebarFolds`).
pub const WORKSPACE: &str = "workspace";
pub const RECENT: &str = "recent";

/// `recentThreads`: of `items`, the ones opened at some moment, newest opened first, the
/// first `LIMIT`. `seen_at` is when each thread was last opened; 0 is one marked unread, and a
/// thread never opened has no entry. A thread in `seen_at` that is no longer among the items is
/// left out, since only the items are walked.
pub fn threads<'a>(items: impl IntoIterator<Item = &'a Item>, seen_at: &HashMap<String, f64>) -> Vec<&'a Item> {
    let at = |i: &Item| seen_at.get(&i.id).copied().unwrap_or(0.0);
    let mut opened: Vec<&Item> = items.into_iter().filter(|i| at(i) > 0.0).collect();
    // A stable sort, as JavaScript's is: two opened at one moment keep the items' order.
    opened.sort_by(|a, b| at(b).partial_cmp(&at(a)).unwrap_or(std::cmp::Ordering::Equal));
    opened.truncate(LIMIT);
    opened
}

/// `toggleFold`: the folded sections with `id` put in, or taken out if it was there.
pub fn toggle_fold(folded: &[String], id: &str) -> Vec<String> {
    if folded.iter().any(|f| f == id) {
        return folded.iter().filter(|f| *f != id).cloned().collect();
    }
    folded.iter().cloned().chain([id.to_string()]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str) -> Item {
        Item { id: id.to_string(), ..Item::default() }
    }
    fn seen(pairs: &[(&str, f64)]) -> HashMap<String, f64> {
        pairs.iter().map(|(id, at)| (id.to_string(), *at)).collect()
    }
    fn ids(got: Vec<&Item>) -> Vec<&str> {
        got.into_iter().map(|i| i.id.as_str()).collect()
    }

    // `tests/the-sidebar-is-a-plain-list-with-recent-threads-and-you-at-the-foot.test.mts`.
    #[test]
    fn the_threads_last_opened_come_newest_first() {
        let items = [item("a"), item("b"), item("c")];
        assert_eq!(ids(threads(&items, &seen(&[("a", 10.0), ("b", 30.0), ("c", 20.0)]))), ["b", "c", "a"]);
    }

    #[test]
    fn a_thread_never_opened_is_left_out() {
        let items = [item("a"), item("b")];
        assert_eq!(ids(threads(&items, &seen(&[("a", 10.0)]))), ["a"]);
    }

    #[test]
    fn a_thread_marked_unread_which_is_kept_as_0_is_left_out() {
        let items = [item("a"), item("b")];
        assert_eq!(ids(threads(&items, &seen(&[("a", 0.0), ("b", 5.0)]))), ["b"]);
    }

    #[test]
    fn a_thread_no_longer_on_this_computer_is_left_out() {
        let items = [item("a")];
        assert_eq!(ids(threads(&items, &seen(&[("a", 10.0), ("gone", 99.0)]))), ["a"]);
    }

    #[test]
    fn they_stop_at_the_limit_and_not_one_before_it() {
        let many: Vec<Item> = (0..LIMIT + 3).map(|n| item(&format!("t{n}"))).collect();
        let at: HashMap<String, f64> = many.iter().enumerate().map(|(n, i)| (i.id.clone(), n as f64 + 1.0)).collect();
        assert_eq!(threads(&many, &at).len(), LIMIT);
        assert_eq!(threads(&many[..LIMIT], &at).len(), LIMIT);
        assert_eq!(threads(&many[..LIMIT - 1], &at).len(), LIMIT - 1);
    }

    #[test]
    fn they_are_nothing_at_all_for_someone_who_has_opened_nothing() {
        assert!(threads(&[item("a")], &HashMap::new()).is_empty());
    }

    #[test]
    fn a_press_on_a_sections_name_folds_it_and_the_next_opens_it() {
        let folded = toggle_fold(&[], WORKSPACE);
        assert_eq!(folded, [WORKSPACE]);
        let both = toggle_fold(&folded, RECENT);
        assert_eq!(both, [WORKSPACE, RECENT]);
        assert_eq!(toggle_fold(&both, WORKSPACE), [RECENT]);
    }
}
