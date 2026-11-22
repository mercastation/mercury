//! The line drawing at the head of each row of the command bar (`ICONS` in
//! `renderer/src/palette-glyphs.ts`): 16 points on a 24 grid, in the thread menu's style, path for
//! path. Which row wears which is `rules::palette::icon_for`; a fixture names the drawing its
//! picture had (`icon` in `paletteRows`).

use rune::Icon;

/// One drawing: its paths in the bar's stroke (`RowIcon` in `renderer/src/components/Palette.tsx`).
macro_rules! bar {
    ($id:literal, $body:literal) => {
        Icon {
            name: $id,
            view: 24.0,
            body: concat!(r#"<g fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">"#, $body, "</g>"),
        }
    };
}

/// Every drawing by the web's own name for it. Arcs are written as the web's `circle` and `box`
/// helpers write them.
static MARKS: [(&str, Icon); 61] = [
    ("rename", bar!("m-bar-rename", r#"<path d="M4 20h4L19 9l-4-4L4 16z"/>"#)),
    ("move", bar!("m-bar-move", r#"<path d="M3 12h13m-4-4 4 4-4 4"/><path d="M20 4v16"/>"#)),
    ("pin", bar!("m-bar-pin", r#"<path d="M8 3h8l-1.5 6 3.5 3.5H6L9.5 9z"/><path d="M12 12.5V21"/>"#)),
    ("later", bar!("m-bar-later", r#"<path d="M3 12a9 9 0 1 0 18 0a9 9 0 1 0 -18 0"/><path d="M12 7v5l3.5 2"/>"#)),
    ("give", bar!("m-bar-give", r#"<path d="M5.5 8a3.5 3.5 0 1 0 7 0a3.5 3.5 0 1 0 -7 0"/><path d="M3 20c0-3.5 2.7-5.5 6-5.5 1.4 0 2.7.4 3.7 1"/><path d="M15 17h6m-2.5-2.5L21 17l-2.5 2.5"/>"#)),
    ("done", bar!("m-bar-done", r#"<path d="m3 13 3.6 3.6L13.4 9"/><path d="m10.6 13 3.6 3.6L21 9"/>"#)),
    ("code", bar!("m-bar-code", r#"<path d="m8.5 7-5 5 5 5"/><path d="m15.5 7 5 5-5 5"/>"#)),
    ("terminal", bar!("m-bar-terminal", r#"<path d="M5 4h14a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-12a2 2 0 0 1 2 -2z"/><path d="m7 9 3 3-3 3m6 0h4"/>"#)),
    ("run", bar!("m-bar-run", r#"<path d="M5 5v14"/><path d="m9 5 10 7-10 7z"/>"#)),
    ("wait", bar!("m-bar-wait", r#"<path d="M6 3h12M6 21h12M7 3v4l5 5 5-5V3M7 21v-4l5-5 5 5v4"/>"#)),
    ("shipped", bar!("m-bar-shipped", r#"<path d="M4 9h16v11H4z"/><path d="M12 15V3m-3.5 3.5L12 3l3.5 3.5"/>"#)),
    ("stop-repeat", bar!("m-bar-stop-repeat", r#"<path d="M20 12a8 8 0 0 1-13.7 5.6M4 12a8 8 0 0 1 13.7-5.6"/><path d="M18 3v4h-4M6 21v-4h4"/><path d="m4 4 16 16"/>"#)),
    ("reopen", bar!("m-bar-reopen", r#"<path d="M4 10h11a5 5 0 0 1 0 10h-4"/><path d="m8 6-4 4 4 4"/>"#)),
    ("add-person", bar!("m-bar-add-person", r#"<path d="M5.5 8a3.5 3.5 0 1 0 7 0a3.5 3.5 0 1 0 -7 0"/><path d="M3 20c0-3.5 2.7-5.5 6-5.5 1.4 0 2.7.4 3.7 1"/><path d="M18 13v7m-3.5-3.5h7"/>"#)),
    ("cloud", bar!("m-bar-cloud", r#"<path d="M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z"/>"#)),
    ("home", bar!("m-bar-home", r#"<path d="M5 10.4 12 5l7 5.4v8.6a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1z"/><path d="M10 20v-4.6a2 2 0 0 1 4 0V20"/>"#)),
    ("team", bar!("m-bar-team", r#"<path d="M9.5 7.5a2.5 2.5 0 1 0 5 0a2.5 2.5 0 1 0 -5 0"/><path d="M4 10a2 2 0 1 0 4 0a2 2 0 1 0 -4 0"/><path d="M16 10a2 2 0 1 0 4 0a2 2 0 1 0 -4 0"/><path d="M8 19c0-2.5 1.8-4.5 4-4.5s4 2 4 4.5"/><path d="M3 18c0-2 1.3-3.5 3-3.5.8 0 1.5.3 2 .8M21 18c0-2-1.3-3.5-3-3.5-.8 0-1.5.3-2 .8"/>"#)),
    ("link", bar!("m-bar-link", r#"<path d="M10 13.5a4 4 0 0 0 5.66 0l2.5-2.5a4 4 0 0 0-5.66-5.66l-1 1"/><path d="M14 10.5a4 4 0 0 0-5.66 0l-2.5 2.5a4 4 0 0 0 5.66 5.66l1-1"/>"#)),
    ("file", bar!("m-bar-file", r#"<path d="M7 4.5h6.5l4 4V19a.5.5 0 0 1-.5.5H7a.5.5 0 0 1-.5-.5V5a.5.5 0 0 1 .5-.5z"/><path d="M13.5 4.5v4h4"/>"#)),
    ("instructions", bar!("m-bar-instructions", r#"<path d="M6 3.5h8l4 4V20H6zM14 3.5V8h4M9 12h6M9 16h6"/>"#)),
    ("search", bar!("m-bar-search", r#"<path d="M4 10.5a6.5 6.5 0 1 0 13 0a6.5 6.5 0 1 0 -13 0"/><path d="M15.5 15.5 20 20"/>"#)),
    ("plus", bar!("m-bar-plus", r#"<path d="M12 5v14M5 12h14"/>"#)),
    ("compose", bar!("m-bar-compose", r#"<path d="M11 4H6a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-5"/><path d="M17.5 3.5l3 3L12 15H9v-3z"/>"#)),
    ("record", bar!("m-bar-record", r#"<path d="M6 12a6 6 0 1 0 12 0a6 6 0 1 0 -12 0"/>"#)),
    ("filter", bar!("m-bar-filter", r#"<path d="M4 6h16M7 12h10M10 18h4"/>"#)),
    ("clear", bar!("m-bar-clear", r#"<path d="M6 6l12 12M18 6 6 18"/>"#)),
    ("folder", bar!("m-bar-folder", r#"<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>"#)),
    ("top", bar!("m-bar-top", r#"<path d="M12 20V9m-5 5 5-5 5 5"/><path d="M5 4h14"/>"#)),
    ("inbox", bar!("m-bar-inbox", r#"<path d="M3 13h5l1.5 2.5h5L16 13h5"/><path d="M5.5 5h13l2.5 8v5a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-5z"/>"#)),
    ("progress", bar!("m-bar-progress", r#"<path d="M3 12a9 9 0 1 0 18 0a9 9 0 1 0 -18 0"/><path d="M10 8.8v6.4l5-3.2z"/>"#)),
    ("hash", bar!("m-bar-hash", r#"<path d="M10 4 8 20M16 4l-2 16M4.5 9h15M4 15h15"/>"#)),
    ("person", bar!("m-bar-person", r#"<path d="M8 8a4 4 0 1 0 8 0a4 4 0 1 0 -8 0"/><path d="M4 21c1.2-4 4.3-6 8-6s6.8 2 8 6"/>"#)),
    ("pause", bar!("m-bar-pause", r#"<path d="M9 5v14M15 5v14"/>"#)),
    ("play", bar!("m-bar-play", r#"<path d="M7 5l12 7-12 7z"/>"#)),
    ("theme", bar!("m-bar-theme", r#"<path d="M4 12a8 8 0 1 0 16 0a8 8 0 1 0 -16 0"/><path d="M12 4v16"/>"#)),
    ("download", bar!("m-bar-download", r#"<path d="M12 4v11m-4.5-4.5L12 15l4.5-4.5"/><path d="M5 20h14"/>"#)),
    ("refresh", bar!("m-bar-refresh", r#"<path d="M20 12a8 8 0 1 1-2.34-5.66"/><path d="M20 4v4.5h-4.5"/>"#)),
    ("gauge", bar!("m-bar-gauge", r#"<path d="M4 17a8 8 0 1 1 16 0"/><path d="m12 17 4-5"/>"#)),
    ("settings", bar!("m-bar-settings", r#"<path d="M4 7h9M17 7h3M4 17h3M11 17h9"/><path d="M13 7a2 2 0 1 0 4 0a2 2 0 1 0 -4 0"/><path d="M7 17a2 2 0 1 0 4 0a2 2 0 1 0 -4 0"/>"#)),
    ("keyboard", bar!("m-bar-keyboard", r#"<path d="M5 6h14a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-8a2 2 0 0 1 2 -2z"/><path d="M7 10h.01M11 10h.01M15 10h.01M8 14h8"/>"#)),
    ("board", bar!("m-bar-board", r#"<path d="M5 4h14a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-12a2 2 0 0 1 2 -2z"/><path d="M9 4v16M15 4v16"/>"#)),
    ("help", bar!("m-bar-help", r#"<path d="M3 12a9 9 0 1 0 18 0a9 9 0 1 0 -18 0"/><path d="M9.5 9.5a2.5 2.5 0 1 1 3.5 2.3c-.6.3-1 .9-1 1.6v.6"/><path d="M12 17h.01"/>"#)),
    ("swap", bar!("m-bar-swap", r#"<path d="M4 8h14m-3.5-3.5L18 8l-3.5 3.5"/><path d="M20 16H6m3.5-3.5L6 16l3.5 3.5"/>"#)),
    ("check", bar!("m-bar-check", r#"<path d="M5 12.5l4.5 4.5L19 7"/>"#)),
    ("closed", bar!("m-bar-closed", r#"<path d="M3 12a9 9 0 1 0 18 0a9 9 0 1 0 -18 0"/><path d="m8 12.3 2.8 2.8L16 9.6"/>"#)),
    ("reply", bar!("m-bar-reply", r#"<path d="M10 7 5 12l5 5"/><path d="M5 12h9a5 5 0 0 1 5 5v1"/>"#)),
    ("read", bar!("m-bar-read", r#"<path d="M3 10l9-6 9 6v9a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1z"/><path d="M3.5 10.5l8.5 5.5 8.5-5.5"/>"#)),
    ("unread", bar!("m-bar-unread", r#"<path d="M5 5h14a2 2 0 0 1 2 2v10a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-10a2 2 0 0 1 2 -2z"/><path d="M3.5 7.5l8.5 6 8.5-6"/>"#)),
    ("stop", bar!("m-bar-stop", r#"<path d="M8 6h8a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-8a2 2 0 0 1 -2 -2v-8a2 2 0 0 1 2 -2z"/>"#)),
    ("open", bar!("m-bar-open", r#"<path d="M14 4h6v6"/><path d="M20 4l-9 9"/><path d="M18 14v4a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4"/>"#)),
    ("lock", bar!("m-bar-lock", r#"<path d="M7 11h10a2 2 0 0 1 2 2v5a2 2 0 0 1 -2 2h-10a2 2 0 0 1 -2 -2v-5a2 2 0 0 1 2 -2z"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/>"#)),
    ("text", bar!("m-bar-text", r#"<path d="M5 7h14M5 12h14M5 17h9"/>"#)),
    ("thread", bar!("m-bar-thread", r#"<path d="M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H9l-5 4z"/>"#)),
    ("comment", bar!("m-bar-comment", r#"<path d="M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H9l-5 4z"/><path d="M8 9h8M8 12.5h5"/>"#)),
    ("flag", bar!("m-bar-flag", r#"<path d="M5 21V4"/><path d="M5 4h11l-2 4 2 4H5"/>"#)),
    ("spark", bar!("m-bar-spark", r#"<path d="M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z"/>"#)),
    ("priority-low", bar!("m-bar-priority-low", r#"<path d="M6 19v-3"/>"#)),
    ("priority-medium", bar!("m-bar-priority-medium", r#"<path d="M6 19v-3M10.5 19v-6.5"/>"#)),
    ("priority-high", bar!("m-bar-priority-high", r#"<path d="M6 19v-3M10.5 19v-6.5M15 19v-10"/>"#)),
    ("priority-urgent", bar!("m-bar-priority-urgent", r#"<path d="M6 19v-3M10.5 19v-6.5M15 19v-10M19.5 19v-14"/>"#)),
    ("chevron", bar!("m-bar-chevron", r#"<path d="m9 6 6 6-6 6"/>"#)),
];

/// The drawing of that name; a name the table does not have wears the plain chevron, as a row
/// nobody has drawn does (`PLAIN_ICON`).
pub fn mark(name: &str) -> &'static Icon {
    MARKS.iter().find(|(n, _)| *n == name).map_or(&MARKS[MARKS.len() - 1].1, |(_, icon)| icon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_name_the_rules_give_a_row_has_its_drawing() {
        for name in crate::rules::palette::icon_names() {
            assert!(MARKS.iter().any(|(n, _)| *n == name), "{name}");
        }
        assert_eq!(mark("compose").name, "m-bar-compose");
        assert_eq!(mark("nothing like it").name, "m-bar-chevron");
    }
}
