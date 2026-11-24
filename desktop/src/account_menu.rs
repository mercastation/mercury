//! The person's account menu, opening upward from their own row at the sidebar's foot, as the
//! web window's does (`AccountMenu` and `AccountMenuList` in `renderer/src/team/account-menu.tsx`,
//! drawn by `.th-acct-menu` in `renderer/src/team/team.css` as the sidebar's own sheets cascade
//! into it). What it holds, top to bottom: who they are, what they are up to, Go invisible or
//! Come back, Invite people, Settings, and Sign out behind a hairline. A row whose door the
//! app does not hand in is not drawn; which those are is `rules/account_menu.rs`.
//!
//! It hangs from the row as the row is laid out now (`rune::memo::window_layout(ROW)`), six
//! points over it, as wide as the row while the sidebar is open and 240 points wide while it
//! is folded (`.workspace-collapsed .th-acct-menu`). A second press on the row, Escape, or a
//! press anywhere else puts it away; so does a press on any of its rows, before that row does
//! what it does. Inventory and deviations: `docs/parity-sidebar.md`, "The account menu".

use rune::{Cursor, Drag, El, Icon, KeyEvent, Phase, TextStyle, Tx, col, el, icon, row, text};
use serde_json::Value;

use crate::rules::account_menu::{self as rules, Doors};
use crate::rules::settings::PageId;
use crate::theme::*;
use crate::{State, feel, icons, settings};

/// The person's own row in the sidebar, which the menu hangs from (`.th-me-btn`, `chrome::rail`).
pub const ROW: &str = "me-row";
/// The menu itself, and Go invisible's card, by the keys a test finds them by.
pub const MENU: &str = "account-menu";
pub const GOING: &str = "account-invisible";

/// What stands open over the row: the menu, or Go invisible's card in its place.
#[derive(Default)]
pub struct Menu {
    pub open: bool,
    /// Go invisible was pressed: its card takes the menu's place (`going`).
    pub going: bool,
}

/// What the menu wrote, for its answer to be put where it goes.
#[derive(Clone, Debug, PartialEq)]
pub enum Wrote {
    SignOut,
    Invisible,
}

/// A press on the row: the menu opens, or closes when it is open (`open ? close() : setOpen(true)`).
pub fn toggle(s: &mut State, _tx: &mut Tx) {
    if s.account.open {
        close(s);
    } else {
        s.account.open = true;
    }
}

/// Put away, whatever stood open (`close`).
pub fn close(s: &mut State) {
    s.account = Menu::default();
}

/// Escape puts it away (`key`); every other key goes on to the page, as in the web window,
/// whose menu listens for Escape alone.
pub fn key(s: &mut State, e: &KeyEvent, _tx: &mut Tx) -> Option<bool> {
    if !s.account.open || e.key != rune::Key::Escape {
        return None;
    }
    close(s);
    Some(true)
}

/// Who they are, and Invite people: Settings on the team's page (`onAccount`, `onInvite`). That
/// page is the web window's, which comes forward on it; on a Mac with no team Settings has no
/// such page and lands on General (`paneFrom`), which this window draws.
fn team_page(s: &mut State, doors: Doors, tx: &mut Tx) {
    close(s);
    if doors.team_page {
        settings::bridge(s, PageId::Team, tx);
    } else {
        settings::open(s, PageId::General, tx);
    }
}

/// Settings, on General, as the rail's own row opened it before (`onSettings`).
fn open_settings(s: &mut State, tx: &mut Tx) {
    close(s);
    settings::open(s, PageId::General, tx);
}

/// Sign out: the team signs the person out, with no question first, and the lists are read
/// again (`api.teamSignOut().then(() => refresh())`). The web window says nothing of how it
/// went, and neither does this one, unless it could not be asked at all.
fn sign_out(s: &mut State, tx: &mut Tx) {
    close(s);
    send(s, rules::SIGN_OUT, Value::Null, Wrote::SignOut, tx);
}

/// Go invisible for so long, or Come back (`call('goInvisible', minutes)`, `call('comeBack')`).
fn invisible(s: &mut State, payload: Value, tx: &mut Tx) {
    close(s);
    send(s, rules::ROOMS, payload, Wrote::Invisible, tx);
}

fn send(s: &mut State, channel: &'static str, payload: Value, wrote: Wrote, tx: &mut Tx) {
    if !s.send(channel, payload, crate::write::Sent::Account(wrote)) {
        let why = s.refused.take().unwrap_or_else(|| "Mercury is not running, so this was not sent.".to_string());
        s.tell(&why, tx);
    }
}

/// What the app answered. Signing out and going invisible say nothing either way, as the web
/// window's `void` calls do: the lists read again show what changed.
pub fn landed(_s: &mut State, _landed: crate::write::Landed, _tx: &mut Tx) {}

/* ----------------------------------- drawing ----------------------------------- */

/// A row of the menu (`.th-acct-menu button`): 34 points, nine in and seven over and under, its
/// mark 18 points in the faint ink and its words 13 points at 500 in the dim one, cut with dots;
/// under the pointer the strong wash, the words in the text's ink and the mark a step brighter.
/// The words fade with the sidebar's own (`shown`): the folded sidebar's sheet takes every
/// `button > span` inside it to nothing, this menu's words with them (`rooms.css`).
fn menu_row(glyph: &'static Icon, words: &str, shown: f32) -> El {
    row()
        .w_full()
        .px(9.0)
        .py(7.0)
        .gap(10.0)
        .items_center()
        .radius(6.0)
        .cursor(Cursor::Pointer)
        .hover_group()
        .hover(feel::wash_strong())
        .child(icon(glyph, 18.0).color(muted()).group_color(dim()))
        .child(el().grow(1.0).min_w(0.0).opacity(shown).child(pen(text(words).font(TextStyle::new(13.0, 500.0).lh(20.0)).color(dim()).group_color(ink()).truncate(), 13.0)))
        .role("menuitem")
        .label(words)
}

/// The hairline between groups (`.th-acct-rule`): five points in from either side, three over
/// and under.
fn rule_line() -> El {
    el().h(1.0).mx(5.0).my(3.0).fill(edge())
}

/// Who they are (`.th-acct-who`): their face, their name over their email, both cut with dots.
fn who(s: &State, letters: &str, doors: Doors, shown: f32) -> El {
    let (name, email) = crate::rules::window::my_names(&s.snap).unwrap_or_default();
    let words = col()
        .grow(1.0)
        .min_w(0.0)
        .opacity(shown)
        .child(pen(text(name.as_str()).font(TextStyle::new(13.0, 500.0).lh(20.0)).color(ink()).truncate(), 13.0))
        .child(pen(text(email).font(TextStyle::new(12.0, 500.0).lh(20.0)).color(muted()).truncate(), 12.0));
    row()
        .w_full()
        .pt(8.0)
        .px(9.0)
        .pb(9.0)
        .gap(9.0)
        .items_center()
        .radius(6.0)
        .cursor(Cursor::Pointer)
        .hover(feel::wash_strong())
        .child(crate::chrome::my_face(letters))
        .child(words)
        .role("menuitem")
        .label(name)
        .on_press_tx(move |s: &mut State, tx| team_page(s, doors, tx))
}

/// The menu (`AccountMenuList`), on the grey every dropdown floats on, inside the strong line.
fn list(s: &State, letters: &str, doors: Doors) -> El {
    let shown = s.rail.words(s.at);
    let mut menu = col().padding(4.0).gap(1.0).child(who(s, letters, doors, shown)).child(rule_line());
    if let Some(on) = doors.invisible {
        let row = menu_row(&icons::ACCOUNT_GHOST, if on { "Come back" } else { "Go invisible" }, shown);
        menu = menu.child(if on {
            row.on_press_tx(|s: &mut State, tx| invisible(s, rules::come_back(), tx))
        } else {
            row.on_press(|s: &mut State| s.account.going = true)
        });
    }
    if doors.invite {
        menu = menu.child(menu_row(&icons::ACCOUNT_INVITE, "Invite people", shown).on_press_tx(move |s: &mut State, tx| team_page(s, doors, tx)));
    }
    menu.child(menu_row(&icons::ACCOUNT_SETTINGS, "Settings", shown).on_press_tx(open_settings))
        .child(rule_line())
        .child(menu_row(&icons::ACCOUNT_SIGN_OUT, "Sign out", shown).on_press_tx(sign_out))
}

/// A fact on Go invisible's card (`.rm-ipop li`): its mark a point down in the faint ink, its
/// words 13 points on 18 in the dim one, wrapped in what is left of the card's 306 points.
fn fact(glyph: &'static Icon, words: &str) -> El {
    row()
        .gap(10.0)
        .items_start()
        .child(icon(glyph, 18.0).color(muted()).mt(1.0))
        .child(pen(text(words).font(TextStyle::new(13.0, 500.0).lh(18.0)).color(dim()).w(306.0 - 18.0 - 10.0).wrap(), 13.0))
}

/// Go invisible's card, in the menu's place (`GoInvisible`, `.rm-ipop`): what it does said before
/// it does it, and for how long.
fn going(s: &State) -> El {
    let ends = rules::invisible_ends(s.now);
    let title = row()
        .gap(9.0)
        .items_center()
        .child(icon(&icons::ACCOUNT_GHOST, 18.0).color(dim()))
        .child(pen(text("Go invisible").font(TextStyle::new(15.0, 500.0).lh(20.0)).color(ink()), 15.0));
    let facts = col()
        .mt(12.0)
        .mb(10.0)
        .gap(7.0)
        .child(fact(&icons::ACCOUNT_EYE_OFF, "Your card leaves every room"))
        .child(fact(&icons::ACCOUNT_LOCK, "Nobody sees your threads, status or agent hours"))
        .child(fact(&icons::ACCOUNT_CHAT, "You can still read every room"));
    let mut card = col()
        // 340 points with the shell's edge (`width: 340px; box-sizing: border-box`).
        .w(338.0)
        .pt(16.0)
        .px(16.0)
        .pb(8.0)
        .child(title)
        .child(facts)
        .child(el().h(1.0).mt(2.0).mb(6.0).mx(-16.0).fill(edge()))
        .role("menu")
        .label("Go invisible");
    for (words, minutes) in ends {
        let when = row()
            .h(32.0)
            .mx(-8.0)
            .px(8.0)
            .items_center()
            .cursor(Cursor::Pointer)
            .hover_group()
            .hover(feel::wash_strong())
            .child(pen(text(words).font(TextStyle::new(13.0, 500.0).lh(20.0)).color(dim()).group_color(ink()), 13.0))
            .role("menuitem")
            .label(words)
            .on_press_tx(move |s: &mut State, tx| invisible(s, rules::go_invisible(minutes), tx));
        card = card.child(when);
    }
    card
}

/// The menu over the window, hung from the person's row: or nothing while it is shut, while
/// Settings covers the window, or with nobody signed in.
pub fn layer(s: &State) -> Option<El> {
    if !s.account.open || s.settings.open {
        return None;
    }
    let doors = rules::doors(&s.snap, s.now)?;
    let letters = crate::rules::window::my_letters(&s.snap)?;
    let [x, y, w, h] = rune::memo::window_layout(ROW)?;
    let (ww, wh) = s.size;
    // As wide as the row while the sidebar is open; 240 points while it is folded.
    let wide = if s.rail.is_open() { w } else { 240.0 };
    let shell = col()
        .pin(Some(x), None, None, Some(wh - (y - 6.0)))
        .fill(pop())
        .border(1.0, rule())
        .border_layout()
        .radius(10.0)
        .box_shadow(shadow(0.0, 24.0, 70.0, 0.0, rgba(0, 0, 0, 0.55), false));
    let shell = if s.account.going { shell.key(GOING).child(going(s)) } else { shell.key(MENU).w(wide).role("menu").label("Your account").child(list(s, &letters, doors)) };
    // It comes in as the web window's menus do, fading in as it rises its last four points from
    // 98% of its size (`prio-in`, 0.13 s), and goes at once.
    let comes_in = rune::Transition::offset(0.0, 4.0).combined(rune::Transition::scale(0.98)).combined(rune::Transition::opacity());
    let shell = feel::answers(shell)
        .animated_changes()
        .motion(rune::Spring::NONE)
        .transition(rune::Transition::asymmetric(comes_in, rune::Transition::none()).springs(rune::SpringPair::new(rune::Spring::new(0.028, -0.66), rune::Spring::NONE)));
    // A press anywhere but the row closes it on the way down, as the web window's `pointerdown`
    // does; the row's own press closes it on its way up.
    let (bottom, right) = (y + h, x + w);
    let away = [[0.0, 0.0, ww, y], [0.0, bottom, ww, wh - bottom], [0.0, y, x, h], [right, y, ww - right, h]].map(|[ax, ay, aw, ah]| {
        el().absolute(ax, ay).size(aw.max(0.0), ah.max(0.0)).on_drag(|s: &mut State, d: &mut Drag, _| {
            if d.phase == Phase::Down {
                close(s);
            }
        })
    });
    Some(el().absolute(0.0, 0.0).size(ww, wh).children(away).child(shell))
}

#[cfg(test)]
mod tests {
    use rune::{Key, Runtime};
    use serde_json::{Value, json};

    use super::{GOING, MENU, ROW};
    use crate::State;
    use crate::feel::tests::{key, press, rect_of, run, window_dressed};
    use crate::rules::settings::{PageId, Pane};

    /// The invented inbox with someone signed in, the sidebar kept open unless `shrinks`, and
    /// `team` laid over the signed-in team (Multiplayer off unless it says).
    fn signed_in(name: &str, shrinks: bool, more: Value) -> Runtime<State> {
        // The page it opens on, when `more` names one (`"view": "inbox"`); Home otherwise.
        let view = more.get("view").and_then(Value::as_str).map(str::to_string);
        let args: Vec<&str> = view.as_deref().map_or(Vec::new(), |v| vec!["--view", v]);
        window_dressed(name, &args, "/fixtures/storefront", "Done.", |fixture| {
            let mut team = json!({ "configured": false, "signedIn": true, "me": { "id": "p-ada", "email": "ada@example.com", "name": "ada.lovelace", "status": null } });
            let alpha = more.get("alpha").cloned().unwrap_or(json!({ "multiplayer": false }));
            if let Some(extra) = more.get("team").and_then(Value::as_object) {
                for (k, v) in extra {
                    team[k] = v.clone();
                }
            }
            fixture["snapshot"]["team"] = team;
            fixture["snapshot"]["alpha"] = alpha;
            if !shrinks {
                fixture["picks"] = json!({ "rail_shrinks": false });
            }
        })
    }

    fn middle(rt: &Runtime<State>, key: &str) -> (f32, f32) {
        let [x, y, w, h] = rect_of(rt, key).unwrap_or_else(|| panic!("{key} is drawn"));
        (x + w / 2.0, y + h / 2.0)
    }

    /// The middle of the drawn words `words`.
    fn words_at(rt: &Runtime<State>, words: &str) -> (f32, f32) {
        let e = rt.elements().into_iter().find(|e| e.text.as_deref() == Some(words)).unwrap_or_else(|| panic!("{words} is drawn"));
        (e.rect[0] + 10.0, e.rect[1] + e.rect[3] / 2.0)
    }

    fn drawn(rt: &Runtime<State>, words: &str) -> bool {
        rt.elements().iter().any(|e| e.text.as_deref() == Some(words))
    }

    fn open_menu(rt: &mut Runtime<State>) {
        run(rt, 0.1);
        let (x, y) = middle(rt, ROW);
        press(rt, x, y);
        run(rt, 0.2);
        assert!(rt.state.account.open, "the row opens the menu");
    }

    #[test]
    fn the_row_opens_the_menu_over_it_and_a_second_press_escape_or_a_press_elsewhere_closes_it() {
        let mut rt = signed_in("account-open", false, json!({ "view": "inbox" }));
        run(&mut rt, 0.1);
        assert!(rect_of(&rt, MENU).is_none());
        open_menu(&mut rt);
        // Six points over the row, from its left edge, as wide as it.
        let [rx, ry, rw, _] = rect_of(&rt, ROW).unwrap();
        let [mx, my, mw, mh] = rect_of(&rt, MENU).expect("the menu is drawn");
        assert_eq!((mx, mw, my + mh), (rx, rw, ry - 6.0));
        // Who they are, Settings and Sign out; with Multiplayer off no status and no Invite people.
        for words in ["ada.lovelace", "ada@example.com", "Settings", "Sign out"] {
            assert!(drawn(&rt, words), "{words}");
        }
        assert!(!drawn(&rt, "Invite people") && !drawn(&rt, "Set a status") && !drawn(&rt, "Go invisible"));
        // A second press on the row closes it.
        let (x, y) = middle(&rt, ROW);
        press(&mut rt, x, y);
        run(&mut rt, 0.1);
        assert!(!rt.state.account.open && rect_of(&rt, MENU).is_none());
        // Escape closes it, and goes no further: the page under it keeps where it is.
        open_menu(&mut rt);
        let view = rt.state.view.clone();
        key(&mut rt, Key::Escape);
        assert!(!rt.state.account.open);
        assert_eq!(rt.state.view, view);
        // A press anywhere else closes it, and does nothing else.
        open_menu(&mut rt);
        press(&mut rt, 700.0, 400.0);
        run(&mut rt, 0.1);
        assert!(!rt.state.account.open && rect_of(&rt, MENU).is_none());
        assert_eq!(rt.state.view, view);
        // Another key goes on to the page while it is open (`keydown` hears Escape alone).
        open_menu(&mut rt);
        key(&mut rt, Key::Text("n".into()));
        assert!(rt.state.account.open && rt.state.compose.card, "N still opens the New thread card");
    }

    #[test]
    fn the_row_is_lit_while_the_menu_is_open_and_says_your_account_while_the_sidebar_is_folded() {
        let mut rt = signed_in("account-lit", true, json!({}));
        run(&mut rt, 0.1);
        let lit = |rt: &Runtime<State>| rt.elements().into_iter().find(|e| e.key.as_deref() == Some(ROW)).map(|e| e.fill.clone());
        let unlit = lit(&rt);
        let row = rt.elements().into_iter().find(|e| e.key.as_deref() == Some(ROW)).unwrap();
        assert_eq!(row.label.as_deref(), Some("Your account"));
        rt.update(|s, tx| super::toggle(s, tx));
        run(&mut rt, 0.4);
        assert_ne!(lit(&rt), unlit, "open, the row wears the wash (`aria-expanded`)");
        // Folded, the menu takes 240 points of its own, and its words are not shown.
        let [_, _, mw, _] = rect_of(&rt, MENU).unwrap();
        assert_eq!(mw, 240.0);
        let all = rt.elements();
        let at = all.iter().position(|e| e.text.as_deref() == Some("Settings")).unwrap();
        let (mut shown, mut on) = (1.0, Some(at));
        while let Some(i) = on {
            shown *= all[i].opacity;
            on = all[i].parent;
        }
        assert!(shown < 0.01, "the folded sidebar's words, these with them, are faded out");
    }

    #[test]
    fn a_pointer_on_the_menu_is_still_on_the_sidebar_and_off_it_the_sidebar_folds_with_the_menu_left_open() {
        let mut rt = signed_in("account-rail", true, json!({}));
        run(&mut rt, 0.1);
        // Onto the folded sidebar from the page, resting there until it opens.
        rt.pointer_move(120.0, 300.0);
        rt.frame(1.0 / 60.0);
        rt.pointer_move(28.0, 300.0);
        run(&mut rt, 0.5);
        assert!(rt.state.rail.is_open());
        open_menu(&mut rt);
        let (x, y) = words_at(&rt, "Settings");
        rt.pointer_move(x, y);
        run(&mut rt, 0.5);
        assert!(rt.state.rail.is_open(), "the menu is part of the sidebar for the pointer");
        rt.pointer_move(700.0, 400.0);
        run(&mut rt, 0.5);
        assert!(!rt.state.rail.is_open(), "off it, the sidebar folds");
        assert!(rt.state.account.open, "and the menu stays, over the folded sidebar");
        assert_eq!(rect_of(&rt, MENU).map(|r| r[2]), Some(240.0));
    }

    #[test]
    fn settings_opens_this_windows_own_settings_on_general_and_closes_the_menu() {
        let mut rt = signed_in("account-settings", false, json!({}));
        open_menu(&mut rt);
        let (x, y) = words_at(&rt, "Settings");
        press(&mut rt, x, y);
        run(&mut rt, 0.1);
        assert!(!rt.state.account.open);
        assert!(rt.state.settings.open);
        assert_eq!(rt.state.settings.pane, Pane::Page(PageId::General));
    }

    #[test]
    fn who_they_are_opens_general_with_no_team_and_the_web_windows_team_page_with_one() {
        // No team: Settings has no Team page, and the web window's lands on General (`paneFrom`).
        let mut rt = signed_in("account-who", false, json!({}));
        open_menu(&mut rt);
        let (x, y) = words_at(&rt, "ada@example.com");
        press(&mut rt, x, y);
        run(&mut rt, 0.1);
        assert!(!rt.state.account.open);
        assert!(rt.state.settings.open);
        assert_eq!(rt.state.settings.pane, Pane::Page(PageId::General));
        // A team, and Multiplayer on: who they are and Invite people bring the web window
        // forward on its Team page. With no app behind this window, that is said instead.
        for words in ["ada@example.com", "Invite people"] {
            let mut rt = signed_in("account-team", false, json!({ "alpha": { "multiplayer": true }, "team": { "configured": true, "team": { "id": "t-1", "name": "Northwind" } } }));
            open_menu(&mut rt);
            let (x, y) = words_at(&rt, words);
            press(&mut rt, x, y);
            run(&mut rt, 0.1);
            assert!(!rt.state.account.open, "{words}");
            assert!(!rt.state.settings.open, "{words}: this window's Settings stays shut");
            assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Mercury's other window is not open now, so this page cannot be shown."), "{words}");
        }
    }

    #[test]
    fn sign_out_writes_the_teams_sign_out_with_nothing_asked_first() {
        let mut rt = signed_in("account-sign-out", false, json!({}));
        rt.state.pretend_sent = true;
        open_menu(&mut rt);
        let (x, y) = words_at(&rt, "Sign out");
        press(&mut rt, x, y);
        run(&mut rt, 0.1);
        assert!(!rt.state.account.open);
        assert_eq!(rt.state.tried, vec![("zero:team-sign-out", Value::Null)]);
        assert_eq!(rt.state.last_sent, Some(crate::write::Sent::Account(super::Wrote::SignOut)));
        // The app answers, either way: nothing is said, as the web window says nothing.
        rt.update(|s, tx| s.landed(crate::write::Landed { sent: crate::write::Sent::Account(super::Wrote::SignOut), answer: Ok(json!({ "ok": false, "error": "offline" })), snapshot: None }, tx));
        assert!(rt.state.toast.is_none());
    }

    #[test]
    fn with_no_app_behind_the_window_sign_out_says_it_was_not_sent() {
        let mut rt = signed_in("account-sign-out-alone", false, json!({}));
        open_menu(&mut rt);
        let (x, y) = words_at(&rt, "Sign out");
        press(&mut rt, x, y);
        run(&mut rt, 0.1);
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Mercury is not running, so this was not sent."));
    }

    #[test]
    fn go_invisible_opens_its_card_in_the_menus_place_and_each_end_asks_the_rooms_for_so_long() {
        let rooms = |until: Value| json!({ "alpha": { "multiplayer": false, "rooms": true }, "team": { "rooms": { "on": true, "me": { "person_id": "p-ada", "invisible_until": until } } } });
        let mut rt = signed_in("account-invisible", false, rooms(Value::Null));
        rt.state.pretend_sent = true;
        open_menu(&mut rt);
        let (x, y) = words_at(&rt, "Go invisible");
        press(&mut rt, x, y);
        run(&mut rt, 0.2);
        assert!(rt.state.account.going && rect_of(&rt, GOING).is_some() && rect_of(&rt, MENU).is_none());
        assert!(drawn(&rt, "Nobody sees your threads, status or agent hours"));
        let (x, y) = words_at(&rt, "For an hour");
        press(&mut rt, x, y);
        run(&mut rt, 0.1);
        assert!(!rt.state.account.open);
        assert_eq!(rt.state.tried, vec![("zero:rooms", json!({ "op": "goInvisible", "args": [60.0] }))]);
        // Until turned off: no minutes.
        open_menu(&mut rt);
        let (x, y) = words_at(&rt, "Go invisible");
        press(&mut rt, x, y);
        run(&mut rt, 0.2);
        let (x, y) = words_at(&rt, "Until I turn it off");
        press(&mut rt, x, y);
        assert_eq!(rt.state.tried[1], ("zero:rooms", json!({ "op": "goInvisible", "args": [null] })));
        // Invisible already: the row is Come back, which asks for nothing more.
        let mut rt = signed_in("account-come-back", false, rooms(json!("2099-01-01T00:00:00.000Z")));
        rt.state.pretend_sent = true;
        open_menu(&mut rt);
        assert!(!drawn(&rt, "Go invisible"));
        let (x, y) = words_at(&rt, "Come back");
        press(&mut rt, x, y);
        run(&mut rt, 0.1);
        assert!(!rt.state.account.open);
        assert_eq!(rt.state.tried, vec![("zero:rooms", json!({ "op": "comeBack", "args": [] }))]);
    }

    #[test]
    fn escape_on_go_invisibles_card_puts_everything_away() {
        let mut rt = signed_in("account-invisible-escape", false, json!({ "alpha": { "rooms": true }, "team": { "rooms": { "on": true, "me": { "person_id": "p-ada" } } } }));
        open_menu(&mut rt);
        let (x, y) = words_at(&rt, "Go invisible");
        press(&mut rt, x, y);
        run(&mut rt, 0.1);
        key(&mut rt, Key::Escape);
        assert!(!rt.state.account.open && !rt.state.account.going && rect_of(&rt, GOING).is_none());
    }

    #[test]
    fn with_nobody_signed_in_there_is_no_row_and_no_menu() {
        let mut rt = window_dressed("account-nobody", &[], "/fixtures/storefront", "Done.", |fixture| {
            fixture["snapshot"]["team"] = json!({ "configured": false, "signedIn": false, "me": { "id": "p-ada", "email": "ada@example.com", "name": "ada.lovelace" } });
            fixture["picks"] = json!({ "rail_shrinks": false });
        });
        run(&mut rt, 0.1);
        assert!(rect_of(&rt, ROW).is_none(), "Settings is the last row instead");
        rt.state.account.open = true;
        run(&mut rt, 0.1);
        assert!(rect_of(&rt, MENU).is_none(), "nothing to hang a menu from, nor anyone to show in it");
    }
}
