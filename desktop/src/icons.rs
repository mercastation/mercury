//! Mercury's own icons, copied from the web app's markup so each draws with the same path,
//! stroke width, caps and joins. `view` is the SVG's viewBox side; the colour comes from the element.

use rune::Icon;

macro_rules! stroked {
    ($name:ident, $id:literal, $width:literal, $body:literal) => {
        pub static $name: Icon = Icon {
            name: $id,
            view: 24.0,
            body: concat!(
                r#"<g fill="none" stroke="currentColor" stroke-width=""#,
                $width,
                r#"" stroke-linecap="round" stroke-linejoin="round">"#,
                $body,
                "</g>"
            ),
        };
    };
}

// The rail. Search and Feedback are drawn at 80% inside their box, as the web app does.
pub static SEARCH: Icon = Icon {
    name: "m-search",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-linecap="round"><g transform="translate(12 12) scale(.8) translate(-12 -12)" stroke-width="1.5625"><circle cx="10.8" cy="10.8" r="6.6"/><path d="M15.6 15.6 20 20"/></g></g>"#,
};
stroked!(HOME, "m-home", "1.25", r#"<path d="M5 10.4 12 5l7 5.4v8.6a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1z"/><path d="M10 20v-4.6a2 2 0 0 1 4 0V20"/>"#);
stroked!(INBOX, "m-inbox", "1.25", r#"<path d="M5 7h14M5 12h14M5 17h9"/>"#);
stroked!(DEMANDS, "m-demands", "1.25", r#"<path d="M6.5 20V4.5"/><path d="M6.5 5.5h10.5l-2.2 3.5 2.2 3.5H6.5"/>"#);
stroked!(
    CONNECTIONS,
    "m-connections",
    "1.25",
    r#"<path d="M10 13.5a4 4 0 0 0 5.66 0l2.5-2.5a4 4 0 0 0-5.66-5.66l-1 1"/><path d="M14 10.5a4 4 0 0 0-5.66 0l-2.5 2.5a4 4 0 0 0 5.66 5.66l1-1"/>"#
);
stroked!(FILES, "m-files", "1.25", r#"<path d="M7 4.5h6.5l4 4V19a.5.5 0 0 1-.5.5H7a.5.5 0 0 1-.5-.5V5a.5.5 0 0 1 .5-.5z"/><path d="M13.5 4.5v4h4"/>"#);
pub static FEEDBACK: Icon = Icon {
    name: "m-feedback",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><g transform="translate(12 12) scale(.8) translate(-12 -12)" stroke-width="1.5625"><path d="M2.5 11.5 21.5 3l-5 18-5-6.5z"/><path d="M11.5 14.5 21.5 3"/><path d="m11.5 14.5-1.2 6 3-3.6"/></g></g>"#,
};
stroked!(
    SETTINGS,
    "m-settings",
    "1.25",
    r#"<path d="M4.5 8h8M16.5 8h3M4.5 16h3M11.5 16h8"/><circle cx="14.5" cy="8" r="2"/><circle cx="9.5" cy="16" r="2"/>"#
);
// What the rail has only for some, or only sometimes: Chats with teammates, Record meeting,
// Restart while a new version waits, what is left of the limit.
stroked!(
    CHATS,
    "m-chats",
    "1.25",
    r#"<circle cx="9" cy="8.5" r="3"/><path d="M3.5 19c.5-3.2 2.7-5 5.5-5s5 1.8 5.5 5"/><path d="M15 5.8a3 3 0 0 1 0 5.4M16.8 14.2c2 .6 3.3 2.2 3.7 4.8"/>"#
);
pub static RECORD: Icon = Icon {
    name: "m-record",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><g transform="translate(12 12) scale(.8) translate(-12 -12)" stroke-width="1.5625"><rect x="9" y="3" width="6" height="11" rx="3"/><path d="M5 11a7 7 0 0 0 14 0"/><path d="M12 18v3"/></g></g>"#,
};
pub static RESTART: Icon = Icon {
    name: "m-restart",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.5"><path d="M19.5 12a7.5 7.5 0 1 1-2.2-5.3"/><path d="M19.5 4.5v4h-4"/></g>"#,
};
pub static USAGE: Icon = Icon {
    name: "m-usage",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"><path d="M4.5 16.5a7.5 7.5 0 1 1 15 0"/><path d="m12 16.5 3.6-4.2"/></g>"#,
};

// Where a conversation runs, beside Done: the cloud, or this Mac (`CloudMark`, `MacMark`).
stroked!(PLACE_CLOUD, "m-place-cloud", "1.4", r#"<path d="M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z"/>"#);
stroked!(PLACE_MAC, "m-place-mac", "1.4", r#"<rect x="3" y="5" width="18" height="12"/><path d="M8 21h8M12 17v4"/>"#);

// Home's doors. Team is there only for someone with teammates.
stroked!(
    DOOR_TEAM,
    "m-door-team",
    "1.8",
    r#"<circle cx="12" cy="7.5" r="2.5"/><circle cx="6" cy="10" r="2"/><circle cx="18" cy="10" r="2"/><path d="M8 19c0-2.5 1.8-4.5 4-4.5s4 2 4 4.5"/><path d="M3 18c0-2 1.3-3.5 3-3.5M21 18c0-2-1.3-3.5-3-3.5"/>"#
);
pub static DOOR_INBOX: Icon = Icon {
    name: "m-door-inbox",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><path d="M5 7h14M5 12h14M5 17h9"/></g>"#,
};
pub static DOOR_PROGRESS: Icon = Icon {
    name: "m-door-progress",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><circle cx="12" cy="12" r="7.5"/><path d="M12 4.5a7.5 7.5 0 0 1 0 15z" fill="currentColor" stroke="none"/></g>"#,
};

// The composer.
pub static ATTACH: Icon = Icon {
    name: "m-attach",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.5"><path d="m20 11.5-8.2 8.2a5 5 0 0 1-7.1-7.1l8.5-8.5a3.3 3.3 0 0 1 4.7 4.7l-8.4 8.4a1.7 1.7 0 0 1-2.4-2.4l7.7-7.7"/></g>"#,
};
pub static CARET: Icon = Icon {
    name: "m-caret",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></g>"#,
};

// Back and forward, above the page.
pub static BACK: Icon = Icon {
    name: "m-back",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M10 3.5L5.5 8l4.5 4.5"/></g>"#,
};
pub static FORWARD: Icon = Icon {
    name: "m-forward",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M6 3.5L10.5 8 6 12.5"/></g>"#,
};

// The Inbox: New thread, the display button, an agent's two eyes, and the pin on a row kept at the top.
pub static PEN: Icon = Icon {
    name: "m-pen",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 20h8"/><path d="m4 20 1-4.5L15.5 5a2.1 2.1 0 0 1 3 3L8 18.5 4 20Z"/></g>"#,
};
pub static SLIDERS: Icon = Icon {
    name: "m-sliders",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 7h10M18 7h2M4 17h4M12 17h8"/><circle cx="16" cy="7" r="2"/><circle cx="10" cy="17" r="2"/></g>"#,
};
// Where threads run: the cloud on the strip's button (`CloudIcon` in `threads/Pages.tsx`).
stroked!(CLOUD, "m-cloud", "1.5", r#"<path d="M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z"/>"#);
// The same cloud before a Status word, while an agent is on the thread in the cloud
// (`RunningMark`). The web app draws it 14 by 11 from the box "2 5 21 15", which fits it to the
// width and leaves half a point over and under; here that is the top 11 of a 14 point square.
pub static CLOUD_MARK: Icon = Icon {
    name: "m-cloud-mark",
    view: 14.0,
    body: r#"<g transform="translate(0 .5) scale(.66667) translate(-2 -5)" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round"><path d="M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z"/></g>"#,
};
stroked!(PIN, "m-pin", "1.4", r#"<path d="M8 3h8l-1.5 6 3.5 3.5H6L9.5 9z"/><path d="M12 12.5V21"/>"#);
pub static AGENT_EYES: Icon = Icon {
    name: "m-agent-eyes",
    view: 16.0,
    body: r#"<g fill="currentColor"><rect x="4.5" y="5.5" width="2" height="5" rx="1"/><rect x="9.5" y="5.5" width="2" height="5" rx="1"/></g>"#,
};

// A conversation's top strip: the threads before and after it, what finishing it is, and more.
pub static UP: Icon = Icon {
    name: "m-up",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M4 10l4-4 4 4"/></g>"#,
};
pub static DOWN: Icon = Icon {
    name: "m-down",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6l4 4 4-4"/></g>"#,
};
pub static MORE: Icon = Icon {
    name: "m-more",
    view: 16.0,
    body: r#"<g fill="currentColor"><rect x="2.5" y="7" width="2" height="2"/><rect x="7" y="7" width="2" height="2"/><rect x="11.5" y="7" width="2" height="2"/></g>"#,
};
stroked!(DONE, "m-done", "1.4", r#"<path d="m3 13 3.6 3.6L13.4 9"/><path d="m10.6 13 3.6 3.6L21 9"/>"#);

// Before a link in an agent's words.
stroked!(LINK, "m-link", "1.6", r#"<path d="M10 14a4 4 0 0 0 5.66 0l3-3a4 4 0 0 0-5.66-5.66l-1 1"/><path d="M14 10a4 4 0 0 0-5.66 0l-3 3a4 4 0 0 0 5.66 5.66l1-1"/>"#);

// The command bar's own magnifier, before its box (`PaletteBox` in `components/Palette.tsx`):
// square ends and no round joins, unlike the rail's.
pub static BAR_SEARCH: Icon = Icon {
    name: "m-bar-search",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="7" cy="7" r="4.6"/><path d="M10.4 10.4L14 14"/></g>"#,
};
