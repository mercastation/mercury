//! The person's running order over their projects, and the one score derived from it. A port
//! of `shared/rank.mts`, which the supervisor spawns by and the lists sort by.

use crate::data::Item;

/// A place in the order is worth this much, so it dominates any item's own priority (0 to 9).
const RANK_STEP: f64 = 100.0;

/// The score a product's place in the order contributes. Higher is sooner; one never placed
/// scores nothing (`placeScore`). A conversation with a person borrows the top place on a
/// team; that would be decided here.
pub fn place_score(order: &[String], slug: &str) -> f64 {
    match order.iter().position(|s| s == slug) {
        Some(idx) => (order.len() - idx) as f64 * RANK_STEP,
        None => 0.0,
    }
}

const DEFAULT_PRIORITY: f64 = 5.0;

/// The number an item carries into that score. Nought means untagged, not bottom: it sits in
/// the middle with everything else (`itemPriority`).
pub fn item_priority(item: &Item) -> f64 {
    item.priority.filter(|n| n.is_finite() && *n > 0.0).unwrap_or(DEFAULT_PRIORITY)
}

/// The only level that interrupts (`isUrgent`, and `isUrgentRow` in `renderer/src/urgent-arrival.ts`).
pub fn is_urgent(item: &Item) -> bool {
    item_priority(item) >= 9.0
}

/// A thread the person dragged sits where they put it: its place replaces the score it would
/// otherwise have (`placedScore`).
pub fn placed_score(item: &Item, natural: f64) -> f64 {
    item.number("place").filter(|p| p.is_finite()).unwrap_or(natural)
}

/// What matters most, as the fleet runs it: the project's place, then the item's own number
/// (`score` in `renderer/src/App.tsx`).
pub fn score(order: &[String], item: &Item) -> f64 {
    placed_score(item, place_score(order, &item.product) + item_priority(item))
}

/// Where a row sits by time: a close taken straight back leaves it where it was (`placedAt` in
/// `renderer/src/list-rules.ts`).
pub fn placed_at(item: &Item) -> f64 {
    item.number("placedAt").or(item.updated_at).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn item(v: serde_json::Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn a_projects_place_outweighs_any_level() {
        let order = vec!["shop".to_string(), "desk".to_string()];
        let low_first = item(json!({ "product": "shop", "priority": 2 }));
        let urgent_second = item(json!({ "product": "desk", "priority": 9 }));
        let unranked = item(json!({ "product": "lab", "priority": 9 }));
        assert_eq!(score(&order, &low_first), 202.0);
        assert_eq!(score(&order, &urgent_second), 109.0);
        assert_eq!(score(&order, &unranked), 9.0);
        // Untagged runs in the middle, and a dragged thread sits where it was put.
        assert_eq!(score(&order, &item(json!({ "product": "desk", "priority": 0 }))), 105.0);
        assert_eq!(score(&order, &item(json!({ "product": "desk", "priority": 9, "place": 150.5 }))), 150.5);
        assert!(is_urgent(&urgent_second) && !is_urgent(&low_first));
    }
}
