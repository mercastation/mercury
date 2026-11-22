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

// The rail. Search is drawn at 80% inside its box, as the web app does. The web window's rail
// has Demands, Chats, Connections, Files, Record meeting and Feedback too; this one has none of
// them (`chrome::rail`), and their marks went with them.
pub static SEARCH: Icon = Icon {
    name: "m-search",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-linecap="round"><g transform="translate(12 12) scale(.8) translate(-12 -12)" stroke-width="1.5625"><circle cx="10.8" cy="10.8" r="6.6"/><path d="M15.6 15.6 20 20"/></g></g>"#,
};
stroked!(HOME, "m-home", "1.25", r#"<path d="M5 10.4 12 5l7 5.4v8.6a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1z"/><path d="M10 20v-4.6a2 2 0 0 1 4 0V20"/>"#);
stroked!(INBOX, "m-inbox", "1.25", r#"<path d="M5 7h14M5 12h14M5 17h9"/>"#);
stroked!(
    SETTINGS,
    "m-settings",
    "1.25",
    r#"<path d="M4.5 8h8M16.5 8h3M4.5 16h3M11.5 16h8"/><circle cx="14.5" cy="8" r="2"/><circle cx="9.5" cy="16" r="2"/>"#
);
// What the rail has only sometimes: Restart while a new version waits, what is left of the limit.
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

// Home's doors, drawn at 16 points with a 1.4 stroke (`ICONS` in `renderer/src/home/HomePage.tsx`).
// Team is there only for someone with teammates.
stroked!(
    DOOR_TEAM,
    "m-door-team",
    "1.4",
    r#"<circle cx="12" cy="7.5" r="2.5"/><circle cx="6" cy="10" r="2"/><circle cx="18" cy="10" r="2"/><path d="M8 19c0-2.5 1.8-4.5 4-4.5s4 2 4 4.5"/><path d="M3 18c0-2 1.3-3.5 3-3.5M21 18c0-2-1.3-3.5-3-3.5"/>"#
);
pub static DOOR_INBOX: Icon = Icon {
    name: "m-door-inbox",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"><path d="M5 7h14M5 12h14M5 17h9"/></g>"#,
};
pub static DOOR_PROGRESS: Icon = Icon {
    name: "m-door-progress",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"><circle cx="12" cy="12" r="7.5"/><path d="M12 4.5a7.5 7.5 0 0 1 0 15z" fill="currentColor" stroke="none"/></g>"#,
};

// Each agent's maker's mark, before the model on the box a thread is started in (`EngineLogo` in
// `renderer/src/components/EngineLogos.tsx`): Claude's in its own orange, Codex's and
// Antigravity's in the colour they are drawn in.
pub static ENGINE_CLAUDE: Icon = Icon {
    name: "m-engine-claude",
    view: 24.0,
    body: r##"<path fill="#D97757" d="M4.709 15.955l4.72-2.647.08-.23-.08-.128H9.2l-.79-.048-2.698-.073-2.339-.097-2.266-.122-.571-.121L0 11.784l.055-.352.48-.321.686.06 1.52.103 2.278.158 1.652.097 2.449.255h.389l.055-.157-.134-.098-.103-.097-2.358-1.596-2.552-1.688-1.336-.972-.724-.491-.364-.462-.158-1.008.656-.722.881.06.225.061.893.686 1.908 1.476 2.491 1.833.365.304.145-.103.019-.073-.164-.274-1.355-2.446-1.446-2.49-.644-1.032-.17-.619a2.97 2.97 0 01-.104-.729L6.283.134 6.696 0l.996.134.42.364.62 1.414 1.002 2.229 1.555 3.03.456.898.243.832.091.255h.158V9.01l.128-1.706.237-2.095.23-2.695.08-.76.376-.91.747-.492.584.28.48.685-.067.444-.286 1.851-.559 2.903-.364 1.942h.212l.243-.242.985-1.306 1.652-2.064.73-.82.85-.904.547-.431h1.033l.76 1.129-.34 1.166-1.064 1.347-.881 1.142-1.264 1.7-.79 1.36.073.11.188-.02 2.856-.606 1.543-.28 1.841-.315.833.388.091.395-.328.807-1.969.486-2.309.462-3.439.813-.042.03.049.061 1.549.146.662.036h1.622l3.02.225.79.522.474.638-.079.485-1.215.62-1.64-.389-3.829-.91-1.312-.329h-.182v.11l1.093 1.068 2.006 1.81 2.509 2.33.127.578-.322.455-.34-.049-2.205-1.657-.851-.747-1.926-1.62h-.128v.17l.444.649 2.345 3.521.122 1.08-.17.353-.608.213-.668-.122-1.374-1.925-1.415-2.167-1.143-1.943-.14.08-.674 7.254-.316.37-.729.28-.607-.461-.322-.747.322-1.476.389-1.924.315-1.53.286-1.9.17-.632-.012-.042-.14.018-1.434 1.967-2.18 2.945-1.726 1.845-.414.164-.717-.37.067-.662.401-.589 2.388-3.036 1.44-1.882.93-1.086-.006-.158h-.055L4.132 18.56l-1.13.146-.487-.456.061-.746.231-.243 1.908-1.312-.006.006z"/>"##,
};
pub static ENGINE_CODEX: Icon = Icon {
    name: "m-engine-codex",
    view: 24.0,
    body: r#"<path fill="currentColor" fill-rule="evenodd" d="M8.086.457a6.105 6.105 0 013.046-.415c1.333.153 2.521.72 3.564 1.7a.117.117 0 00.107.029c1.408-.346 2.762-.224 4.061.366l.063.03.154.076c1.357.703 2.33 1.77 2.918 3.198.278.679.418 1.388.421 2.126a5.655 5.655 0 01-.18 1.631.167.167 0 00.04.155 5.982 5.982 0 011.578 2.891c.385 1.901-.01 3.615-1.183 5.14l-.182.22a6.063 6.063 0 01-2.934 1.851.162.162 0 00-.108.102c-.255.736-.511 1.364-.987 1.992-1.199 1.582-2.962 2.462-4.948 2.451-1.583-.008-2.986-.587-4.21-1.736a.145.145 0 00-.14-.032c-.518.167-1.04.191-1.604.185a5.924 5.924 0 01-2.595-.622 6.058 6.058 0 01-2.146-1.781c-.203-.269-.404-.522-.551-.821a7.74 7.74 0 01-.495-1.283 6.11 6.11 0 01-.017-3.064.166.166 0 00.008-.074.115.115 0 00-.037-.064 5.958 5.958 0 01-1.38-2.202 5.196 5.196 0 01-.333-1.589 6.915 6.915 0 01.188-2.132c.45-1.484 1.309-2.648 2.577-3.493.282-.188.55-.334.802-.438.286-.12.573-.22.861-.304a.129.129 0 00.087-.087A6.016 6.016 0 015.635 2.31C6.315 1.464 7.132.846 8.086.457zm-.804 7.85a.848.848 0 00-1.473.842l1.694 2.965-1.688 2.848a.849.849 0 001.46.864l1.94-3.272a.849.849 0 00.007-.854l-1.94-3.393zm5.446 6.24a.849.849 0 000 1.695h4.848a.849.849 0 000-1.696h-4.848z"/>"#,
};
pub static ENGINE_ANTIGRAVITY: Icon = Icon {
    name: "m-engine-antigravity",
    view: 24.0,
    body: r#"<path fill="currentColor" fill-rule="evenodd" d="M21.751 22.607c1.34 1.005 3.35.335 1.508-1.508C17.73 15.74 18.904 1 12.037 1 5.17 1 6.342 15.74.815 21.1c-2.01 2.009.167 2.511 1.507 1.506 5.192-3.517 4.857-9.714 9.715-9.714 4.857 0 4.522 6.197 9.714 9.715z"/>"#,
};

/// The mark of the agent a model runs on.
pub fn engine_logo(engine: &str) -> &'static Icon {
    match engine {
        "codex" => &ENGINE_CODEX,
        "antigravity" => &ENGINE_ANTIGRAVITY,
        _ => &ENGINE_CLAUDE,
    }
}

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
// The View line of the View and filters menu (`ListIcon`, `BoardIcon` in `threads/Pages.tsx`),
// with the square ends they are drawn with.
pub static LIST: Icon = Icon {
    name: "m-list",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.6"><path d="M4 6h16M4 12h16M4 18h16"/></g>"#,
};
pub static BOARD: Icon = Icon {
    name: "m-board",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.6"><rect x="3.5" y="4" width="5" height="16" rx="1"/><rect x="10.5" y="4" width="5" height="11" rx="1"/><rect x="17.5" y="4" width="3" height="7" rx="1"/></g>"#,
};
// The small tick over a tab the filters emptied (`ClearMark`).
pub static CLEAR_MARK: Icon = Icon {
    name: "m-clear-mark",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="square"><path d="m4 12.5 5 5L20 6.5"/></g>"#,
};
// The mark of a wait on the cloud menu's line about an account being added (`.cs-busy`): a ring
// with its top quarter in the accent. The web window turns it; it stands still here.
pub static BUSY: Icon = Icon {
    name: "m-busy",
    view: 12.0,
    body: r##"<g fill="none" stroke-width="1.5"><circle cx="6" cy="6" r="5.25" stroke="#52504f"/><path d="M2.29 2.29A5.25 5.25 0 0 1 9.71 2.29" stroke="#ee6018"/></g>"##,
};
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
// And before a file it names, one for each kind of thing a file is (`PATHS` in `Mention.tsx`).
stroked!(MENTION_FOLDER, "m-mention-folder", "1.6", r#"<path d="M3 7.5A2.5 2.5 0 0 1 5.5 5H9l2 2.2h7.5A2.5 2.5 0 0 1 21 9.7v7.8a2.5 2.5 0 0 1-2.5 2.5h-13A2.5 2.5 0 0 1 3 17.5z"/>"#);
stroked!(MENTION_PAGE, "m-mention-page", "1.6", r#"<rect x="3" y="4" width="18" height="16" rx="2.5"/><path d="M3 9h18M6.5 6.6h.01M9.3 6.6h.01"/>"#);
stroked!(MENTION_DOC, "m-mention-doc", "1.6", r#"<path d="M14 3H7.5A2.5 2.5 0 0 0 5 5.5v13A2.5 2.5 0 0 0 7.5 21h9a2.5 2.5 0 0 0 2.5-2.5V8z"/><path d="M14 3v5h5M9 13h6M9 16.5h4"/>"#);
stroked!(MENTION_IMAGE, "m-mention-image", "1.6", r#"<rect x="3" y="4" width="18" height="16" rx="2.5"/><circle cx="9" cy="10" r="1.7"/><path d="m21 15.5-4.5-4.5L7 20.5"/>"#);
stroked!(MENTION_CODE, "m-mention-code", "1.6", r#"<path d="m8 8-4 4 4 4M16 8l4 4-4 4M13.5 5l-3 14"/>"#);
stroked!(MENTION_MEDIA, "m-mention-media", "1.6", r#"<rect x="3" y="5" width="18" height="14" rx="2.5"/><path d="m10 9.5 4.5 2.5-4.5 2.5z"/>"#);
stroked!(MENTION_FILE, "m-mention-file", "1.6", r#"<path d="M14 3H7.5A2.5 2.5 0 0 0 5 5.5v13A2.5 2.5 0 0 0 7.5 21h9a2.5 2.5 0 0 0 2.5-2.5V8z"/><path d="M14 3v5h5"/>"#);

// The command bar's own magnifier, before its box (`PaletteBox` in `components/Palette.tsx`):
// square ends and no round joins, unlike the rail's.
pub static BAR_SEARCH: Icon = Icon {
    name: "m-bar-search",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="7" cy="7" r="4.6"/><path d="M10.4 10.4L14 14"/></g>"#,
};

// The mark on a line of what a run did that opens (`chevron` in `Thread.tsx`), and the same
// mark turned a quarter for open, which the web window does with `rotate(90deg)`.
pub static WORK_MARK: Icon = Icon {
    name: "m-work-mark",
    view: 8.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><path d="M2.6 1.2 L5.6 4 L2.6 6.8"/></g>"#,
};
// The working line's chevron, which shows under the pointer and turns down while its steps are
// open (`.live-chevron`), and the mark of each step (`ActivityRow`).
pub static LIVE_CHEVRON: Icon = Icon {
    name: "m-live-chevron",
    view: 12.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.25" stroke-linecap="round" stroke-linejoin="round"><path d="m4 2 4 4-4 4"/></g>"#,
};
pub static LIVE_CHEVRON_OPEN: Icon = Icon {
    name: "m-live-chevron-open",
    view: 12.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.25" stroke-linecap="round" stroke-linejoin="round"><path d="m10 4-4 4-4-4"/></g>"#,
};
pub static LIVE_COMMAND: Icon = Icon {
    name: "m-live-command",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.3"><rect x="1.5" y="1.5" width="13" height="13" rx="3"/><path d="m4 5 3 3-3 3m5 0h3"/></g>"#,
};
pub static WORK_MARK_OPEN: Icon = Icon {
    name: "m-work-mark-open",
    view: 8.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><path d="M6.8 2.6 L4 5.6 L1.2 2.6"/></g>"#,
};

// The last row of the project menu, where a project's swatch would be: New project.
stroked!(PLUS, "m-plus", "1.8", r#"<path d="M12 5v14M5 12h14"/>"#);

// The rows of the caret's menu beside Send: a time to send at, and taking it back (`ClockIcon`
// and `ClearIcon` in `ThreadComposer.tsx`, square ends as drawn there).
pub static CLOCK: Icon = Icon {
    name: "m-clock",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="8"/><path d="M12 8v4.5l3 2"/></g>"#,
};
pub static CLEAR: Icon = Icon {
    name: "m-clear",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.5"><path d="M7 7l10 10M17 7 7 17"/></g>"#,
};

// The Files button over a conversation (`FILES` in `renderer/src/threads/ThreadFiles.tsx`): a
// page with its corner turned, and the edge of another behind it.
stroked!(
    THREAD_FILES,
    "m-thread-files",
    "1.4",
    r#"<path d="M8 3.5h6l4 4V17a.5.5 0 0 1-.5.5H8a.5.5 0 0 1-.5-.5V4a.5.5 0 0 1 .5-.5z"/><path d="M14 3.5v4h4"/><path d="M5 7v12.5a1 1 0 0 0 1 1h9"/>"#
);

// The marks of the thread's own menu under the dots, square as every mark there is (`MARKS` in
// `renderer/src/threads/ThreadMenu.tsx`), and the tick beside the level a thread is at.
stroked!(MENU_TERMINAL, "m-menu-terminal", "1.4", r#"<rect x="3" y="4" width="18" height="16"/><path d="m7 9 3 3-3 3m6 0h4"/>"#);
stroked!(MENU_LATER, "m-menu-later", "1.4", r#"<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3.5 2"/>"#);
stroked!(MENU_RENAME, "m-menu-rename", "1.4", r#"<path d="M4 20h4L19 9l-4-4L4 16z"/>"#);
stroked!(MENU_MOVE, "m-menu-move", "1.4", r#"<path d="M3 12h13m-4-4 4 4-4 4"/><path d="M20 4v16"/>"#);
stroked!(MENU_TICK, "m-menu-tick", "1.4", r#"<path d="m5 12.5 4.5 4.5L19 7.5"/>"#);

// The Files place, on the rail with the places (`data-tab="files"` in `WorkspaceNavigation.tsx`).
stroked!(FILES, "m-files", "1.25", r#"<path d="M7 4.5h6.5l4 4V19a.5.5 0 0 1-.5.5H7a.5.5 0 0 1-.5-.5V5a.5.5 0 0 1 .5-.5z"/><path d="M13.5 4.5v4h4"/>"#);

/// A file opened big: the corner marks of its bar, on one optical scale (`ArtifactControlGlyph`):
/// back, open outside and close, each 16 points in a box of 20.
macro_rules! control {
    ($name:ident, $id:literal, $body:literal) => {
        pub static $name: Icon = Icon {
            name: $id,
            view: 20.0,
            body: concat!(r#"<g fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">"#, $body, "</g>"),
        };
    };
}
control!(FILE_BACK, "m-file-back", r#"<path d="M16 10H4m5-5-5 5 5 5"/>"#);
control!(FILE_OUTSIDE, "m-file-outside", r#"<path d="M11 3h6v6m0-6-8 8M7 4H4a1 1 0 0 0-1 1v11a1 1 0 0 0 1 1h11a1 1 0 0 0 1-1v-3"/>"#);
control!(FILE_CLOSE, "m-file-close", r#"<path d="m5 5 10 10M15 5 5 15"/>"#);

/// The zoom's pill under a picture opened big (`ZoomGlyph` in `ArtifactStage.tsx`).
pub static ZOOM_OUT: Icon = Icon {
    name: "m-zoom-out",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M3.5 8h9"/></g>"#,
};
pub static ZOOM_IN: Icon = Icon {
    name: "m-zoom-in",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M3.5 8h9M8 3.5v9"/></g>"#,
};

/// Where a file was sent, under it opened from Files: a thread, or a chat (`PlaceGlyph`).
pub static SENT_THREAD: Icon = Icon {
    name: "m-sent-thread",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><path d="M3 4h10M3 8h10M3 12h6"/></g>"#,
};
pub static SENT_CHAT: Icon = Icon {
    name: "m-sent-chat",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"><path d="M2.5 3.5h11v7h-6l-3 2.5v-2.5h-2z"/></g>"#,
};

/// The Files place's own marks (`FilesPlace.tsx`): a folder, the disk the total is on, the grid
/// and the list, the search's glass and a file's comments.
pub static FOLDER: Icon = Icon {
    name: "m-folder",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"><path d="M2 4.6c0-.6.5-1.1 1.1-1.1h2.7c.3 0 .6.1.8.3l1 1h5.3c.6 0 1.1.5 1.1 1.1v6c0 .6-.5 1.1-1.1 1.1H3.1c-.6 0-1.1-.5-1.1-1.1z"/></g>"#,
};
pub static DISK: Icon = Icon {
    name: "m-disk",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.2"><rect x="2" y="4" width="12" height="8" rx="1.5"/><path d="M4.5 9.5h3" stroke-linecap="round"/><circle cx="11.2" cy="9.5" r=".6" fill="currentColor" stroke="none"/></g>"#,
};
pub static GRID: Icon = Icon {
    name: "m-grid",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.2"><rect x="2.5" y="2.5" width="4.4" height="4.4" rx="1.1"/><rect x="9.1" y="2.5" width="4.4" height="4.4" rx="1.1"/><rect x="2.5" y="9.1" width="4.4" height="4.4" rx="1.1"/><rect x="9.1" y="9.1" width="4.4" height="4.4" rx="1.1"/></g>"#,
};
// The Files place's List, a 16-point mark of its own: not the View and filters menu's `LIST`.
pub static FILES_LIST: Icon = Icon {
    name: "m-files-list",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><path d="M3 4h10M3 8h10M3 12h10"/></g>"#,
};
pub static COMMENT: Icon = Icon {
    name: "m-comment",
    view: 16.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"><path d="M2.5 3.5h11v7h-6l-3 2.5v-2.5h-2z"/></g>"#,
};
pub static PLAY: Icon = Icon {
    name: "m-play",
    view: 16.0,
    body: r#"<path fill="currentColor" d="M5 3.2v9.6a.6.6 0 0 0 .9.5l7.6-4.8a.6.6 0 0 0 0-1L5.9 2.7a.6.6 0 0 0-.9.5Z"/>"#,
};
