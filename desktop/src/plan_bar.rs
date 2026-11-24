//! The bar across the top of the page that says agents cannot start until they have a plan to
//! run on, or until a plan is signed in (`PlanBar` in `renderer/src/components/PlanSetup.tsx`,
//! and THE FALLBACK BAR in `App.tsx`). The rules are `rules/plan_setup.rs`.
//!
//! It asks the engine `zero:engine-setup` with `ready`, for Claude and for ChatGPT's Codex, as
//! the window opens and again each time it comes to the front while the bar is up (`checkPlan`).
//! It stands over Home, the Inbox and Files, under the card's strip, never over an open
//! conversation and never while anything runs, which proves the sentence false. While it is up a
//! signed-out row's line points at it (`signInBar`, `rules::live`), whether or not it is drawn.
//!
//! Its two buttons open a setup card that is the web window's first-run family, which this
//! window does not draw: a press brings the web window forward on that card for the plan
//! pressed (`zero:open-plan-setup-here`, a channel of the desktop app's door alone). When the
//! person comes back to this window the bar reads the computer again.

use rune::{Cursor, El, Latest, Tx, el, row, text};
use serde_json::Value;

use crate::rules::plan_setup::{self as rules, BarWords, Reading};
use crate::source::{self, Source};
use crate::theme::*;
use crate::{State, View, feel, front};

/// Its height: twelve points over and under a button of 31, and its line under that
/// (`.ps-bar`: `padding: 12px 18px; border-bottom: 1px solid`).
pub const HEIGHT: f32 = 56.0;

/// What a press says when the app is not there to open its setup card.
const SETUP_NOT_HERE: &str = "A plan is set up in Mercury's own window.";

/// The bar's state: what the last reading of the computer said, and the reading under way.
#[derive(Default)]
pub struct Bar {
    words: Option<BarWords>,
    asking: Latest<Vec<Result<Value, String>>>,
}

impl Bar {
    pub fn set_waker(&self, wake: rune::Waker) {
        self.asking.set_waker(wake);
    }
    /// Whether the bar has words, which is what `planWords` says, drawn or not.
    pub fn up(&self) -> bool {
        self.words.is_some()
    }
}

/// The two answers read as the web window reads them, and the bar decided from them.
pub fn read(s: &mut State, answers: &[Result<Value, String>]) {
    let readings: Vec<Reading> = rules::ENGINES.iter().zip(answers).map(|(engine, answer)| rules::reading(engine, answer)).collect();
    s.plan.words = rules::plan_bar(&readings);
    crate::rules::live::sign_in_bar_is_up(s.plan.up());
    s.refresh();
}

/// Asks the engine again, off this window's thread. A window with no engine behind it draws no
/// bar: a page that cannot reach the app reads as ready (`assumeReady` in `api.ts`).
pub fn check(s: &mut State) {
    // A test reads what would have been asked, from a window with nobody to ask.
    #[cfg(test)]
    s.tried.extend(rules::asks());
    let Source::Door(door) = &s.source else {
        return;
    };
    let Some(door) = door.now() else {
        return;
    };
    s.plan.asking.request(door, |(url, token)| rules::asks().into_iter().map(|(channel, payload)| source::call(&url, &token, channel, &payload)).collect());
}

/// Each frame: a reading that came back.
pub fn poll(s: &mut State) -> bool {
    let Some(answers) = s.plan.asking.take() else {
        return false;
    };
    read(s, &answers);
    true
}

/// A picture's fixture carries what the page was answered (`zero:engine-setup`, by agent).
pub fn picture(s: &mut State) {
    let Some(answered) = s.fixture.get(rules::ENGINE_SETUP).cloned() else {
        return;
    };
    let answers: Vec<Result<Value, String>> = rules::ENGINES.iter().map(|engine| answered.get(engine).cloned().ok_or_else(|| "no answer".to_string())).collect();
    read(s, &answers);
}

/// The window came to the front: while the bar is up the computer is read again, since that is
/// the move somebody signing in somewhere else makes (`window.addEventListener('focus', …)`).
pub fn came_forward(s: &mut State, focused: bool) {
    if focused && s.plan.up() {
        check(s);
    }
}

/// Whether the bar is drawn now: it has words, nothing runs, and the page is not a conversation
/// (`planWords && !snap.supervisor.running?.length && !focused`).
fn shown(s: &State) -> Option<&BarWords> {
    let words = s.plan.words.as_ref()?;
    (s.snap.supervisor.running.is_empty() && !matches!(s.view, View::Thread { .. }) && !s.settings.open).then_some(words)
}

/// How much of the page it takes, which the page is laid out under.
pub fn height(s: &State) -> f32 {
    if shown(s).is_some() { HEIGHT } else { 0.0 }
}

/// A press on one of its buttons: the web window forward on its setup card for that plan.
fn set_up(s: &mut State, plan: &str, tx: &Tx) {
    feel::pressed(s);
    let (channel, payload) = rules::open_setup(plan);
    #[cfg(test)]
    s.tried.push((channel, payload.clone()));
    match &s.source {
        Source::Door(door) => front::ask_the_app_with(door, channel, payload, SETUP_NOT_HERE),
        _ => s.tell(SETUP_NOT_HERE, tx),
    }
}

/// One of its two buttons (`.ps-bar button`, `.quiet`): 13 points, six over and fourteen
/// beside inside the strong line round it; the first on the selected ground in the text's ink, the
/// second on none in the dim ink. Under the pointer both take the hover ground and the ink.
fn button(words: &str, plan: String, loud: bool, key: &'static str) -> El {
    let ground = if loud { selected() } else { rune::Color::TRANSPARENT };
    let b = row()
        .key(key)
        .h(31.0)
        .px(14.0)
        .items_center()
        .radius(6.0)
        .fill(ground)
        .border(1.0, rule())
        .border_layout()
        .hover_group()
        .hover(hover_ground())
        .cursor(Cursor::Pointer)
        .child(text(words).font(sans(13.0, 400.0)).color(if loud { ink() } else { dim() }).group_color(ink()));
    b.on_press_tx(move |s: &mut State, tx| set_up(s, &plan, tx))
}

/// `--bg-selected` and `--bg-hover`.
fn selected() -> rune::Color {
    crate::theme::selected()
}
fn hover_ground() -> rune::Color {
    crate::theme::hovered()
}

/// The bar, across the card under its strip at `top`, when it is drawn.
pub fn view(s: &State, top: f32) -> Option<El> {
    let words = shown(s)?;
    let say = row().grow(1.0).min_w(0.0).child(
        text(&words.say).font(sans(13.5, 400.0)).color(ink()),
    );
    let say = say.child(text(&format!(" {}", rules::BAR_SUB)).font(sans(13.5, 400.0)).color(dim()));
    let bar = row()
        .key("plan-bar")
        .w_full()
        .h(HEIGHT)
        .px(18.0)
        .gap(14.0)
        .items_center()
        .fill(feel::film())
        .border_sides([0.0, 0.0, 1.0, 0.0], edge())
        .border_layout()
        .child(say)
        .child(button(&words.first.1, words.first.0.clone(), true, "plan-bar-first"))
        .child(button(&words.second.1, words.second.0.clone(), false, "plan-bar-second"));
    Some(el().pin(Some(crate::chrome::card_left(s)), Some(top), Some(crate::chrome::CARD[2]), None).child(bar))
}

#[cfg(test)]
mod tests {
    use rune::Runtime;
    use serde_json::{Value, json};

    use super::*;
    use crate::feel::tests::{press, run, window_dressed};

    const NOTHING: &str = r#"{ "ok": true, "phase": "idle", "found": false, "signedIn": false, "certain": true }"#;
    const SIGNED_OUT: &str = r#"{ "ok": true, "phase": "idle", "found": true, "signedIn": false, "certain": true }"#;
    const READY: &str = r#"{ "ok": true, "phase": "ready", "found": true, "signedIn": true, "certain": true }"#;

    /// The invented inbox, the page told `claude` and `codex` of this computer's plans, its first
    /// thread signed out (`signedOut`) or an agent at work on another (`running`).
    fn told(name: &str, args: &[&str], claude: &str, codex: &str, signed_out: bool, running: bool) -> Runtime<State> {
        let (claude, codex): (Value, Value) = (serde_json::from_str(claude).unwrap(), serde_json::from_str(codex).unwrap());
        window_dressed(name, args, "/fixtures/storefront", "Done, and tested. Merge it?", move |fixture| {
            let now = fixture["now"].as_f64().unwrap();
            fixture["zero:engine-setup"] = json!({ "claude": claude, "codex": codex });
            if signed_out {
                fixture["snapshot"]["items"][0]["status"] = json!("open");
                fixture["snapshot"]["supervisor"]["signInNeeded"] = json!({ "w-feel0": "Claude Code" });
            }
            if running {
                fixture["snapshot"]["items"][3]["status"] = json!("claimed");
                fixture["snapshot"]["supervisor"]["running"] = json!([{ "itemId": "w-feel3", "product": "storefront", "startedAt": now - 60_000.0, "tail": [] }]);
            }
        })
    }

    fn said(rt: &Runtime<State>) -> Vec<String> {
        rt.elements().iter().filter_map(|e| e.text.clone()).collect()
    }

    #[test]
    fn the_bar_stands_over_the_inbox_says_why_and_lays_the_page_out_under_it() {
        let mut rt = told("plan-bar", &["--view", "inbox"], NOTHING, NOTHING, false, false);
        run(&mut rt, 0.1);
        let bar = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("plan-bar")).expect("the bar");
        assert_eq!((bar.rect[1], bar.rect[3]), (104.0, HEIGHT));
        let words = said(&rt);
        for line in ["Agents cannot start until they have a plan to run on.", " Takes a couple of clicks.", "Set up with Claude", "Set up with ChatGPT"] {
            assert!(words.iter().any(|t| t == line), "{line} in {words:?}");
        }
        // The page is laid out under it: the same Inbox, 56 points lower.
        let tab = |rt: &Runtime<State>| rt.elements().into_iter().find(|e| e.text.as_deref() == Some("Needs you")).map(|e| e.rect[1]).unwrap();
        let with = tab(&rt);
        let mut bare = told("plan-bar-none", &["--view", "inbox"], NOTHING, READY, false, false);
        run(&mut bare, 0.1);
        assert!(bare.elements().iter().all(|e| e.key.as_deref() != Some("plan-bar")), "a plan signed in: no bar");
        assert_eq!(with - tab(&bare), HEIGHT);
    }

    #[test]
    fn its_buttons_lead_with_the_agent_on_the_computer_and_hand_over_to_the_web_windows_setup_card() {
        let mut rt = told("plan-bar-press", &["--view", "inbox"], NOTHING, SIGNED_OUT, false, false);
        run(&mut rt, 0.1);
        let words = said(&rt);
        assert!(words.iter().any(|t| t == "Agents cannot start until you sign in to your ChatGPT plan."), "{words:?}");
        rt.state.tried.clear();
        let (x, y) = rt.find("plan-bar-first").expect("the first button");
        press(&mut rt, x, y);
        assert_eq!(rt.state.tried, vec![(rules::OPEN_PLAN_SETUP, json!({ "plan": "codex" }))]);
        // With no app behind this window, a line says where a plan is set up.
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some(SETUP_NOT_HERE));
        let (x, y) = rt.find("plan-bar-second").expect("the second button");
        press(&mut rt, x, y);
        assert_eq!(rt.state.tried.last(), Some(&(rules::OPEN_PLAN_SETUP, json!({ "plan": "claude" }))));
    }

    #[test]
    fn it_stands_over_no_conversation_and_not_while_anything_runs() {
        let mut rt = told("plan-bar-thread", &["--thread", "w-feel1"], NOTHING, NOTHING, false, false);
        run(&mut rt, 0.1);
        assert!(rt.find("plan-bar").is_none());
        assert!(rt.state.plan.up(), "its words stand all the same");
        let mut rt = told("plan-bar-running", &["--view", "inbox"], NOTHING, NOTHING, false, true);
        run(&mut rt, 0.1);
        assert!(rt.find("plan-bar").is_none());
    }

    #[test]
    fn while_it_is_up_a_signed_out_thread_points_at_it() {
        let mut rt = told("plan-bar-line", &["--thread", "w-feel0"], SIGNED_OUT, NOTHING, true, false);
        run(&mut rt, 0.1);
        assert!(said(&rt).iter().any(|t| t == "Claude Code is signed out. Sign in to Claude at the top of your inbox, and this starts on its own."), "{:?}", said(&rt));
        let mut rt = told("plan-bar-line-none", &["--thread", "w-feel0"], READY, NOTHING, true, false);
        run(&mut rt, 0.1);
        assert!(said(&rt).iter().any(|t| t == "Claude Code is signed out. Open a terminal, run claude, and type /login, and this starts on its own."));
    }

    #[test]
    fn coming_back_to_the_window_reads_the_computer_again_only_while_it_is_up() {
        let mut rt = told("plan-bar-forward", &["--view", "inbox"], NOTHING, NOTHING, false, false);
        run(&mut rt, 0.1);
        rt.state.tried.clear();
        rt.update(|s, tx| crate::front::focus_changed(s, false, tx));
        assert!(rt.state.tried.is_empty());
        rt.update(|s, tx| crate::front::focus_changed(s, true, tx));
        assert_eq!(rt.state.tried, rules::asks());
        // Signed in somewhere else: the bar goes, and the window's coming back asks nothing more.
        rt.update(|s, _| read(s, &[Ok(serde_json::from_str(READY).unwrap()), Ok(serde_json::from_str(NOTHING).unwrap())]));
        run(&mut rt, 0.1);
        assert!(rt.find("plan-bar").is_none());
        rt.state.tried.clear();
        rt.update(|s, tx| crate::front::focus_changed(s, true, tx));
        assert!(rt.state.tried.is_empty());
    }
}
