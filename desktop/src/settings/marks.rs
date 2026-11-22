//! The marks Settings draws, copied from the web window's markup so each draws with the same
//! path, stroke and caps (`NavIcon` and the controls in `components/Settings.tsx`,
//! `ProjectsPage.tsx`). Kept here rather than in `icons.rs`, which the other screens share.

use rune::Icon;

use crate::rules::settings::PageId;

/// A mark of the page menu: a 24 point box, a 1.4 hairline, round caps and joins.
macro_rules! page_mark {
    ($name:ident, $id:literal, $body:literal) => {
        pub static $name: Icon = Icon {
            name: $id,
            view: 24.0,
            body: concat!(r#"<g fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">"#, $body, "</g>"),
        };
    };
}

page_mark!(GENERAL, "s-general", r#"<path d="M4 7h9M17 7h3M4 12h3M11 12h9M4 17h11M19 17h1"/><circle cx="15" cy="7" r="2"/><circle cx="9" cy="12" r="2"/><circle cx="17" cy="17" r="2"/>"#);
page_mark!(APPEARANCE, "s-appearance", r#"<circle cx="12" cy="12" r="8"/><path d="M12 4a8 8 0 0 1 0 16z" fill="currentColor" stroke="none"/>"#);
page_mark!(SHORTCUTS, "s-shortcuts", r#"<rect x="3.5" y="6.5" width="17" height="11" rx="2"/><path d="M7.5 10.5h.01M11 10.5h.01M14.5 10.5h.01M8.5 14h7"/>"#);
page_mark!(PHONE, "s-phone", r#"<rect x="7" y="3" width="10" height="18" rx="2.2"/><path d="M11 17.5h2"/>"#);
page_mark!(CLAUDE, "s-claude", r#"<rect x="3.5" y="5" width="17" height="14" rx="2"/><path d="m7.5 10 2.5 2-2.5 2M12.5 14.5h4"/>"#);
page_mark!(CODEX, "s-codex", r#"<path d="m9 8-4 4 4 4M15 8l4 4-4 4"/>"#);
page_mark!(ANTIGRAVITY, "s-antigravity", r#"<path d="m7 12.5 5-5 5 5"/><path d="M6.5 18.5h11"/>"#);
page_mark!(RUNNING, "s-running", r#"<circle cx="12" cy="12" r="8"/><path d="M10.5 9.2v5.6l4.5-2.8z"/>"#);
page_mark!(CLOUD, "s-cloud", r#"<path d="M4 18V9M9.5 18V5M15 18v-6M20.5 18v-9"/>"#);
page_mark!(INSTRUCTIONS, "s-instructions", r#"<path d="M6.5 3.5h7.5l4 4V20.5h-11.5zM14 3.5V8h4M9.5 12h5M9.5 16h5"/>"#);
page_mark!(PROJECTS, "s-projects", r#"<path d="M3.5 7.5a2 2 0 0 1 2-2h4l2 2h7a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z"/>"#);
page_mark!(FILES, "s-files", r#"<path d="M6.5 3.5h7l4 4v13h-11zM13.5 3.5v4h4"/>"#);
page_mark!(TEAM, "s-team", r#"<circle cx="9" cy="9" r="3"/><path d="M3.5 19c.8-3 3-4.5 5.5-4.5s4.7 1.5 5.5 4.5"/><circle cx="16.5" cy="9.5" r="2.5"/><path d="M16 14.6c2.3-.2 4 1.2 4.5 4"/>"#);
page_mark!(ALPHA, "s-alpha", r#"<path d="M9.5 3.5h5M10.5 3.5v5.2L5.6 17.6a2 2 0 0 0 1.8 2.9h9.2a2 2 0 0 0 1.8-2.9l-4.9-8.9V3.5"/><path d="M7.6 14.5h8.8"/>"#);
page_mark!(ROOMS, "s-rooms", r#"<path d="M9.5 4.5 7.5 19.5M16.5 4.5l-2 15M5 9.5h14.5M4.5 14.5H19"/>"#);

/// The mark of a page's row in the menu (`NavIcon`).
pub fn page(id: PageId) -> &'static Icon {
    match id {
        PageId::General => &GENERAL,
        PageId::Appearance => &APPEARANCE,
        PageId::Shortcuts => &SHORTCUTS,
        PageId::Phone => &PHONE,
        PageId::Alpha => &ALPHA,
        PageId::Claude => &CLAUDE,
        PageId::Codex => &CODEX,
        PageId::Antigravity => &ANTIGRAVITY,
        PageId::Running => &RUNNING,
        PageId::Instructions => &INSTRUCTIONS,
        PageId::Projects => &PROJECTS,
        PageId::Files => &FILES,
        PageId::Team => &TEAM,
        PageId::Rooms => &ROOMS,
        PageId::Cloud => &CLOUD,
    }
}

/// Back to app, and the crumb back to Projects: a 16 point box at 1.6.
pub static BACK: Icon = Icon {
    name: "s-back",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M10 3.5L5.5 8l4.5 4.5"/></g>"#,
};

/// The menu's search (`.set-search svg`).
pub static SEARCH: Icon = Icon {
    name: "s-search",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><circle cx="10.5" cy="10.5" r="6"/><path d="m15 15 4.5 4.5"/></g>"#,
};

/// The chevron of a picker, drawn at the text's weight (`.set-picker-chev`).
pub static CHEVRON: Icon = Icon {
    name: "s-chev",
    view: 10.0,
    body: r#"<path d="M2.2 4 L5 6.8 L7.8 4" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

/// The Projects page's own search (`.proj-search-icon`).
pub static FIND: Icon = Icon {
    name: "s-find",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><circle cx="7" cy="7" r="4.25"/><path d="m10.2 10.2 3.3 3.3"/></g>"#,
};

/// A project's row: up, down and the door at its end.
macro_rules! row_mark {
    ($name:ident, $id:literal, $d:literal) => {
        pub static $name: Icon = Icon {
            name: $id,
            view: 16.0,
            body: concat!(r#"<path d=""#, $d, r#"" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>"#),
        };
    };
}
row_mark!(UP, "s-up", "M4.5 9.5 8 6l3.5 3.5");
row_mark!(DOWN, "s-down", "M4.5 6.5 8 10l3.5-3.5");
row_mark!(OPEN, "s-open", "M6.5 4 10.5 8l-4 4");

/// The pencil beside a project's name (`.set-title-pencil`).
pub static PENCIL: Icon = Icon {
    name: "s-pencil",
    view: 24.0,
    body: r#"<path d="M4 20h4L19 9l-4-4L4 16z" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

/// A tick in a box (`.ps-tick`, `.pp-check`).
pub static TICK: Icon = Icon {
    name: "s-tick",
    view: 16.0,
    body: r#"<path d="M3.8 8.4 6.6 11.2 12.2 5" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

/// The stepper's two signs, at the text's weight (`Stepper`).
pub static MINUS: Icon = Icon {
    name: "s-minus",
    view: 12.0,
    body: r#"<path d="M2.5 6h7" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>"#,
};
pub static PLUS: Icon = Icon {
    name: "s-plus",
    view: 12.0,
    body: r#"<path d="M2.5 6h7M6 2.5v7" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>"#,
};
