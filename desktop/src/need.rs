//! What an agent asks the person for, drawn as a card where it asked (`.need`,
//! `renderer/src/components/need-card.css`). Here: the sign-in link an agent handed to its
//! `sign_in` tool and waits on (`SignInCard.tsx`), under the conversation and over the line
//! that says Asking. `rules/sign_in.rs` says which ask is open and what each control writes;
//! `docs/parity-asks.md` is the checklist this was built from.
//!
//! The card keeps a little of its own: whether a write is on its way, what the engine said to
//! the last one, the two boxes' words. It is the card of one ask on one thread: another ask,
//! or leaving the conversation, starts it again, as the web window's does when it is unmounted.

use rune::edit::Key;
use rune::{Cursor, El, KeyEvent, Span, SurfaceFit, TextEdit, TextStyle, Tx, col, el, icon, row, text, text_input};
use serde_json::Value;

use crate::State;
use crate::data::Item;
use crate::rules::sign_in::{self, Ask, Card, DID_NOT_REACH, Said};
use crate::theme::*;
use crate::write::{Landed, Sent};
use crate::{icons, logos};

/// The card is never wider than this, however wide the column (`.need { max-width }`).
const WIDE: f32 = 640.0;
/// `--radius`.
const RADIUS: f32 = 6.0;
/// The line a button's and a link's words stand in: 1.6 of 13 points.
const LINE: f32 = 20.8;
/// A button, a box to type in.
const CONTROL: f32 = 34.0;

/// The boxes' names, for the keyboard.
const PASTED: &str = "need-code";
const LANDED: &str = "need-landed";

/// The card's own state.
pub struct SignIn {
    /// The thread and the ask it is the card of.
    of: Option<(String, String)>,
    /// A write is on its way: the buttons stand at half strength and take no press.
    pub busy: bool,
    /// What the engine said to the last write, when it did not take it.
    pub said: Option<String>,
    /// "Page did not load after signing in?" was pressed: the box for the address is up.
    pub pasting: bool,
    /// The tick that saves a cloud sign-in for every cloud thread, once pressed; before that
    /// it is what the ask says (`ask.keep !== false`).
    pub save: Option<bool>,
    /// The code the service's page showed, and the address the browser ended on.
    pub pasted: TextEdit,
    pub landed: TextEdit,
}

impl Default for SignIn {
    fn default() -> Self {
        SignIn { of: None, busy: false, said: None, pasting: false, save: None, pasted: TextEdit::new(""), landed: TextEdit::new("") }
    }
}

fn thread_of(item: &Item) -> String {
    format!("{}/{}", item.product, item.id)
}

impl State {
    /// The open thread and the sign-in it waits on.
    fn sign_in_asked(&self) -> Option<(Item, Ask)> {
        let item = self.open_item()?;
        let ask = sign_in::open(item)?;
        Some((item.clone(), ask))
    }
    /// The card of that ask, started again when it was another's.
    fn sign_in_card(&mut self, item: &Item, ask: &Ask) -> &mut SignIn {
        let of = (thread_of(item), ask.id.clone());
        if self.sign_in.of.as_ref() != Some(&of) {
            self.sign_in = SignIn { of: Some(of), ..SignIn::default() };
        }
        &mut self.sign_in
    }
    /// Leaving the conversation puts the card away with everything typed into it.
    pub fn leave_the_sign_in(&mut self) {
        self.sign_in = SignIn::default();
    }
    /// Open: the link, in the browser (`openArtifactExternally`).
    fn sign_in_open(&mut self) {
        if let Some((item, ask)) = self.sign_in_asked()
            && !self.sign_in_card(&item, &ask).busy
        {
            crate::files::open_with_system(&ask.url);
        }
    }
    /// Copy, beside the code to type on the service's page.
    fn sign_in_copy(&mut self, tx: &mut Tx) {
        if let Some(code) = self.sign_in_asked().and_then(|(_, ask)| ask.code) {
            tx.copy(code);
            self.tell("Copied", tx);
        }
    }
    /// What the person did with the link, sent to the agent that waits on it
    /// (`api.signInAnswer`). The card holds still until the engine has answered.
    fn sign_in_say(&mut self, said: Said) {
        let Some((item, ask)) = self.sign_in_asked() else { return };
        let card = self.sign_in_card(&item, &ask);
        if card.busy {
            return;
        }
        card.busy = true;
        card.said = None;
        let (channel, payload) = sign_in::answer(&item, &ask.id, &said);
        if !self.send(channel, payload, Sent::SignInSaid { thread: thread_of(&item), ask: ask.id.clone() }) {
            self.sign_in_refused(false, DID_NOT_REACH.to_string());
        }
    }
    /// The code box's Send, and Return in it.
    fn sign_in_send_code(&mut self) {
        let code = self.sign_in.pasted.text.trim().to_string();
        if !code.is_empty() {
            self.sign_in_say(Said::Code(code));
        }
    }
    /// The address box's Send, and Return in it.
    fn sign_in_send_address(&mut self) {
        let url = self.sign_in.landed.text.trim().to_string();
        if !url.is_empty() {
            self.sign_in_say(Said::Landed(url));
        }
    }
    /// The tick: written to the ask at once, so a login that comes back by itself honours it
    /// too (`api.signInKeep`), and put back as it was if the engine does not take it.
    fn sign_in_keep(&mut self) {
        let Some((item, ask)) = self.sign_in_asked() else { return };
        let card = self.sign_in_card(&item, &ask);
        if card.busy {
            return;
        }
        let next = !card.save.unwrap_or(ask.keep);
        card.save = Some(next);
        card.said = None;
        let (channel, payload) = sign_in::keep(&item, &ask.id, next);
        if !self.send(channel, payload, Sent::SignInKeep { thread: thread_of(&item), ask: ask.id.clone() }) {
            self.sign_in_refused(true, DID_NOT_REACH.to_string());
        }
    }
    /// "Page did not load after signing in?": the box for the address, which takes the keys.
    fn sign_in_paste_the_address(&mut self, tx: &mut Tx) {
        let Some((item, ask)) = self.sign_in_asked() else { return };
        let card = self.sign_in_card(&item, &ask);
        card.pasting = true;
        card.landed.focused = true;
        tx.focus_when_shown(LANDED, false);
    }
    /// A write the engine did not take: the card says why under its buttons, and a tick goes
    /// back as it was.
    fn sign_in_refused(&mut self, tick: bool, why: String) {
        // The line about it is the card's own: nothing else says it.
        self.refused = None;
        let card = &mut self.sign_in;
        if tick {
            card.save = card.save.map(|on| !on);
        } else {
            card.busy = false;
        }
        card.said = Some(why);
    }
    /// What the engine said to one of the card's writes (`r.ok ? null : r.error ?? …`).
    pub fn sign_in_landed(&mut self, landed: Landed) {
        let (thread, ask, tick) = match &landed.sent {
            Sent::SignInSaid { thread, ask } => (thread, ask, false),
            Sent::SignInKeep { thread, ask } => (thread, ask, true),
            _ => return,
        };
        // An answer to a card that is no longer up is about nothing on screen.
        if self.sign_in.of.as_ref() != Some(&(thread.clone(), ask.clone())) {
            return;
        }
        let why = match &landed.answer {
            Ok(answer) if crate::data::truthy(answer.get("ok")) => None,
            Ok(answer) => Some(answer.get("error").and_then(Value::as_str).filter(|e| !e.is_empty()).unwrap_or(DID_NOT_REACH).to_string()),
            Err(_) => Some(DID_NOT_REACH.to_string()),
        };
        match why {
            Some(why) => self.sign_in_refused(tick, why),
            None if !tick => self.sign_in.busy = false,
            None => {}
        }
    }
}

/// A picture taken after the card was pressed or typed into says so in its fixture
/// (`signInPressed`, `signInTyped`), and what the engine then did (`signInRefused`,
/// `signInHangs`): the same is done here, with nobody to send to.
pub fn picture(s: &mut State) {
    let pressed: Vec<String> = s.fixture.get("signInPressed").and_then(|p| serde_json::from_value(p.clone()).ok()).unwrap_or_default();
    let typed = s.fixture.get("signInTyped").cloned();
    if pressed.is_empty() && typed.is_none() {
        return;
    }
    let Some((item, ask)) = s.sign_in_asked() else { return };
    let hangs = s.fixture.contains_key("signInHangs");
    let card = s.sign_in_card(&item, &ask);
    for press in &pressed {
        match press.as_str() {
            "also" => {
                card.pasting = true;
                card.landed.focused = true;
            }
            // With nobody to send to, a write is refused at once; a picture of one on its way
            // holds it there.
            "signed-in" | "skipped" if hangs => card.busy = true,
            "signed-in" | "skipped" => card.said = Some(DID_NOT_REACH.to_string()),
            _ => {}
        }
    }
    let words = |field: &str| typed.as_ref().and_then(|t| t.get(field)).and_then(Value::as_str).map(str::to_string);
    if let Some(landed) = words("landed") {
        let focused = card.landed.focused;
        card.landed = TextEdit::new(&landed);
        card.landed.focused = focused;
    }
    if let Some(pasted) = words("pasted") {
        card.pasted = TextEdit::new(&pasted);
    }
}

/// The mark at the card's head (`.need-mark`): the service's own logo when the link is the
/// service's own, a key otherwise, in a square of 40 with the strong line round it.
fn mark(card: &Card, scale: f32) -> El {
    let logo = card.logo.as_deref().and_then(|id| logos::of(id, 20.0, scale));
    let inside = match logo {
        Some(logo) => el().size(20.0, 20.0).pixels_fit(&logo, SurfaceFit::Fit),
        None => icon(&icons::NEED_KEY, 20.0).color(dim()),
    };
    row().size(40.0, 40.0).center().radius(RADIUS).fill(white(0.04)).border(1.0, rule()).child(inside)
}

/// One of the card's buttons (`.need-btn`): 34 high, 15 in on each side, the strong line round
/// it, its words 13 points. The main one (`.need-main`) is the text's ink with the card's
/// ground for its words, and gives up a little of itself under the pointer; the others take
/// the wash. `off` is a button that takes no press: half strength, and nothing under the pointer.
fn button(key: &'static str, words: &str, main: bool, out: bool, off: bool) -> El {
    let on_it = if main { raised() } else { ink() };
    let mut face = row().h(CONTROL).px(15.0).items_center().radius(RADIUS).border(1.0, if main { ink() } else { rule() }).border_layout();
    if main {
        face = face.fill(ink());
    }
    face = face.child(pen(text(words).font(TextStyle::new(13.0, if main { 500.0 } else { 400.0 }).lh(kept(LINE))).color(on_it), 13.0));
    if out {
        // The arrow stands a point under the words' middle line (`vertical-align: -1px`).
        face = face.child(icon(&icons::NEED_OUT, 12.0).color(on_it).ml(6.0).mt(-1.8));
    }
    let face = match (off, main) {
        (true, _) => face.opacity(0.5),
        (false, true) => face.group_opacity(1.0, 0.88),
        (false, false) => face.hover(crate::feel::wash()),
    };
    row().key(key).role("button").label(words).disabled(off).hover_group().child(face)
}

/// A quiet way out of the card (`.need-link`): dim words on a line of their own ink, the
/// text's ink under the pointer.
fn link(key: &'static str, words: &str) -> El {
    let line = text(words).font(TextStyle::new(13.0, 400.0).lh(kept(LINE))).color(dim()).group_color(ink()).decorations(vec![Span::new(0, words.len()).underline(dim())]);
    row().key(key).role("button").label(words).hover_group().child(pen(line, 13.0))
}

/// The keys a box of the card does not keep for typing: Escape lets go of the box and goes no
/// further, so the next one is the page's.
fn in_a_box(_: &mut State, e: &KeyEvent, tx: &mut Tx) -> bool {
    if e.key != Key::Escape {
        return false;
    }
    tx.blur();
    true
}

/// A box to type in (`.need-input`): 34 high, on the page's sunk ground, the strong line round
/// it and the faint ink while it has the keys.
fn boxed(key: &'static str, field: El, focused: bool) -> El {
    row()
        .h(CONTROL)
        .grow(1.0)
        .min_w(0.0)
        .px(11.0)
        .items_center()
        .radius(RADIUS)
        .fill(surface())
        .border(1.0, if focused { muted() } else { rule() })
        .border_layout()
        .cursor(Cursor::Text)
        .on_press_tx(move |_: &mut State, tx| tx.focus(key, false))
        .child(pen(field, 13.0))
}

fn typed(edit: &TextEdit, key: &'static str, placeholder: &str) -> El {
    text_input(edit, TextStyle::new(13.0, 400.0).lh(kept(LINE)))
        .key(key)
        .single_line()
        .grow(1.0)
        .color(ink())
        .placeholder(placeholder, muted())
        .caret(ink(), rgba(110, 165, 255, 0.3))
        .label(placeholder)
        .on_input_key(in_a_box)
}

/// The card of the sign-in the thread `item` waits on, when it waits on one. `cloud` says its
/// run is on another machine, which is when a sign-in can be saved for every cloud thread and
/// a link that comes back to localhost never loads its last page here.
pub fn sign_in_card(s: &State, item: &Item, cloud: bool, column: f32) -> Option<El> {
    let ask = sign_in::open(item)?;
    let card = sign_in::card(&ask, cloud);
    // The card's own state, when it is this ask's; a card nobody has touched has none.
    let fresh = SignIn::default();
    let here = if s.sign_in.of.as_ref() == Some(&(thread_of(item), ask.id.clone())) { &s.sign_in } else { &fresh };
    let busy = here.busy;

    let title = pen(text(card.title.as_str()).font(TextStyle::new(15.0, 500.0).lh(22.0).tracking(-0.09)).color(ink()), 15.0);
    let mut said = col().grow(1.0).min_w(0.0).child(readable(title));
    if let Some(why) = &card.why {
        said = said.child(readable(pen(text(why.as_str()).font(TextStyle::new(13.5, 400.0).lh(20.0)).color(dim()).wrap(), 13.5)).mt(3.0));
    }
    // The code to type on the service's page, in a frame of its own with Copy at its end
    // (`.need-code-box`).
    if let Some(code) = &ask.code {
        let label = pen(text("Enter this code").font(TextStyle::new(13.0, 400.0).lh(kept(LINE))).color(dim()), 13.0);
        let value = readable(text(code.as_str()).font(mono(16.0, kept(25.6)).tracking(2.24)).color(ink()));
        let copy = link("need-copy", "Copy").on_press_tx(|s: &mut State, tx| s.sign_in_copy(tx));
        said = said.child(
            row()
                .mt(12.0)
                .px(14.0)
                .py(10.0)
                .gap(14.0)
                .items_center()
                .radius(RADIUS)
                .border(1.0, edge())
                .border_layout()
                .child(label)
                .child(el().grow(1.0).min_w(0.0).child(value))
                .child(copy),
        );
    }
    // Open, I've signed in and Not now (`.need-acts`). A login that waits for a code has no
    // "I've signed in": the code is the answer.
    let mut open = button("need-open", &card.open, true, true, busy);
    let mut skip = link("need-skip", "Not now").ml(6.0);
    if !busy {
        open = open.on_press(|s: &mut State| s.sign_in_open());
        skip = skip.on_press(|s: &mut State| s.sign_in_say(Said::Skipped));
    }
    let mut acts = row().mt(14.0).gap(8.0).items_center().child(open);
    if !ask.asks_for_code {
        let mut signed = button("need-signed", "I've signed in", false, false, busy);
        if !busy {
            signed = signed.on_press(|s: &mut State| s.sign_in_say(Said::SignedIn));
        }
        acts = acts.child(signed);
    }
    said = said.child(acts.child(skip));
    // The box for the code the service's page shows, and Send (`.need-form`).
    if ask.asks_for_code {
        let field = typed(&here.pasted, PASTED, &card.code_placeholder)
            .on_edit(|s: &mut State| {
                // Whatever was said of the last one is about words that are gone.
                s.sign_in_touched().said = None;
                &mut s.sign_in_touched().pasted
            })
            .on_submit(|s: &mut State, _| s.sign_in_send_code());
        let off = busy || here.pasted.text.trim().is_empty();
        let mut send = button("need-send-code", "Send", true, false, off);
        if !off {
            send = send.on_press(|s: &mut State| s.sign_in_send_code());
        }
        said = said.child(row().mt(14.0).gap(8.0).child(boxed(PASTED, field, here.pasted.focused)).child(send));
    }
    // A sign-in made for a run on another machine is saved for every cloud thread unless the
    // person unticks it (`.need-check`).
    if cloud {
        let on = here.save.unwrap_or(ask.keep);
        let mut tick = row().size(16.0, 16.0).center().radius(4.0).border(1.0, rule());
        if on {
            tick = tick.child(icon(&icons::NEED_TICK, 10.0).color(ink()));
        }
        let words = pen(text("Save this sign-in for every cloud thread").font(TextStyle::new(13.0, 400.0).lh(18.0)).color(dim()), 13.0);
        let mut check = row().key("need-keep").role("checkbox").label("Save this sign-in for every cloud thread").h(18.0).gap(8.0).items_center().child(tick).child(words);
        if !busy {
            check = check.on_press(|s: &mut State| s.sign_in_keep());
        }
        said = said.child(row().mt(12.0).child(check));
    }
    // A link that comes back to localhost never loads its last page here for a run elsewhere:
    // the way to hand over the address the browser ended on (`.need-also`), and its box.
    if card.offers_the_address && !here.pasting {
        // The link is a word in a line of the card's own reading size, which reaches a little
        // under it.
        let also = link("need-also", "Page did not load after signing in?").on_press_tx(|s: &mut State, tx| s.sign_in_paste_the_address(tx));
        said = said.child(row().mt(6.0).mb(0.79).child(also));
    }
    if here.pasting {
        let field = typed(&here.landed, LANDED, "Paste the address from the browser")
            .on_edit(|s: &mut State| {
                s.sign_in_touched().said = None;
                &mut s.sign_in_touched().landed
            })
            .on_submit(|s: &mut State, _| s.sign_in_send_address());
        let off = busy || here.landed.text.trim().is_empty();
        let mut send = button("need-send-address", "Send", true, false, off);
        if !off {
            send = send.on_press(|s: &mut State| s.sign_in_send_address());
        }
        said = said.child(row().mt(14.0).gap(8.0).child(boxed(LANDED, field, here.landed.focused)).child(send));
    }
    if let Some(why) = &here.said {
        said = said.child(readable(pen(text(why.as_str()).font(TextStyle::new(12.5, 400.0).lh(17.0)).color(dim()).wrap(), 12.5)).my(8.0));
    }

    let body = row().items_start().gap(16.0).px(22.0).py(16.0).child(mark(&card, s.pictures.scale())).child(said);
    Some(
        col()
            .key("need-sign-in")
            .w(column.min(WIDE))
            .radius(RADIUS)
            .fill(raised())
            .border(1.0, rule())
            .border_layout()
            .box_shadow(shadow(0.0, 1.0, 2.0, 0.0, rgba(0, 0, 0, 0.3), false))
            .child(body),
    )
}

impl State {
    /// The card being typed into: the open thread's, started again if it was another ask's.
    fn sign_in_touched(&mut self) -> &mut SignIn {
        if let Some((item, ask)) = self.sign_in_asked() {
            return self.sign_in_card(&item, &ask);
        }
        &mut self.sign_in
    }
}

#[cfg(test)]
mod tests {
    use rune::{Key, KeyEvent, Mods, Runtime};
    use serde_json::json;

    use super::*;
    use crate::View;
    use crate::feel::tests::{key, press, run, window, window_dressed};
    use crate::rules::sign_in::{ANSWER, KEEP};

    const THREAD: &str = "storefront/w-feel0";
    const ASK: &str = "s-ask0000001";

    /// The invented inbox's first thread, its agent up and waiting on the sign-in `ask`, on
    /// this Mac or in the cloud, opened; whatever is sent is taken as sent.
    fn asking(name: &str, ask: Value, cloud: bool) -> Runtime<State> {
        let mut rt = window_dressed(name, &["--thread", "w-feel0"], "/fixtures/storefront", "Done.", |fixture| {
            let now = fixture["now"].as_f64().unwrap();
            let mut sign_in = json!({ "id": ASK, "at": now - 180_000.0 });
            for (field, value) in ask.as_object().unwrap() {
                sign_in[field] = value.clone();
            }
            let item = &mut fixture["snapshot"]["items"][0];
            item["status"] = json!("claimed");
            item["signIn"] = sign_in;
            let mut up = json!({ "itemId": "w-feel0", "product": "storefront", "startedAt": now - 300_000.0, "engine": "claude-code", "tail": [], "activity": [] });
            if cloud {
                up["where"] = json!("cloud");
            }
            fixture["snapshot"]["supervisor"]["running"] = json!([up]);
        });
        rt.state.pretend_sent = true;
        rt
    }

    fn cloudflare() -> Value {
        json!({ "service": "Cloudflare", "url": "https://dash.cloudflare.com/oauth2/auth?client_id=1", "why": "to point the shop's domain at the new server" })
    }

    fn vercel() -> Value {
        json!({ "service": "Vercel", "url": "https://vercel.com/oauth/authorize?redirect_uri=http%3A%2F%2Flocalhost%3A8976%2Fcallback", "why": "to deploy the preview" })
    }

    fn texts(rt: &Runtime<State>) -> Vec<String> {
        rt.elements().iter().filter_map(|e| e.text.clone()).collect()
    }

    fn says(rt: &Runtime<State>, words: &str) -> bool {
        texts(rt).iter().any(|t| t == words)
    }

    fn press_on(rt: &mut Runtime<State>, key: &str) {
        let (x, y) = rt.find(key).unwrap_or_else(|| panic!("{key} is not on the page"));
        press(rt, x, y);
    }

    fn type_in(rt: &mut Runtime<State>, words: &str) {
        key(rt, Key::Text(words.into()));
    }

    /// The engine's answer to the card's last write.
    fn lands(rt: &mut Runtime<State>, tick: bool, answer: Result<Value, String>) {
        let (thread, ask) = (THREAD.to_string(), ASK.to_string());
        let sent = if tick { Sent::SignInKeep { thread, ask } } else { Sent::SignInSaid { thread, ask } };
        rt.update(|s, tx| s.landed(Landed { sent, answer, snapshot: None }, tx));
        rt.frame(1.0 / 60.0);
    }

    #[test]
    fn a_thread_whose_agent_waits_on_a_sign_in_draws_its_card_under_the_conversation() {
        let rt = asking("need-card", cloudflare(), false);
        for words in ["Sign in at dash.cloudflare.com", "To point the shop's domain at the new server.", "Open dash.cloudflare.com", "I've signed in", "Not now"] {
            assert!(says(&rt, words), "{words:?} is not on the card: {:?}", texts(&rt));
        }
        // On this Mac there is nothing to save for the cloud and no address to hand over.
        assert!(rt.find("need-keep").is_none() && rt.find("need-also").is_none() && rt.find(PASTED).is_none());
        // Over the line that says the agent is asking.
        let (_, card) = rt.find("need-sign-in").unwrap();
        let asking = rt.elements().iter().find(|e| e.text.as_deref() == Some("Asking")).map(|e| e.rect[1]).expect("the line that says Asking");
        assert!(card < asking);
        // A thread that waits on none has no card.
        let plain = window("need-none", &["--thread"]);
        assert!(plain.find("need-sign-in").is_none());
    }

    #[test]
    fn an_answered_ask_and_a_finished_thread_have_no_card() {
        let mut rt = asking("need-answered", cloudflare(), false);
        rt.update(|s, _| {
            s.snap.items[0].rest.insert("signInAnswer".to_string(), json!({ "id": ASK, "said": "signed-in", "at": 1 }));
        });
        rt.frame(1.0 / 60.0);
        assert!(rt.find("need-sign-in").is_none());
    }

    #[test]
    fn ive_signed_in_writes_the_answer_and_the_card_holds_still_until_the_engine_has_it() {
        let mut rt = asking("need-signed", cloudflare(), false);
        press_on(&mut rt, "need-signed");
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-feel0", "ask": ASK, "said": "signed-in" }))]);
        assert!(rt.state.sign_in.busy);
        // On its way: no button takes another press.
        for held in ["need-skip", "need-signed", "need-open"] {
            press_on(&mut rt, held);
        }
        assert_eq!(rt.state.tried.len(), 1);
        // Taken: the card is free again, and says nothing.
        lands(&mut rt, false, Ok(json!({ "ok": true })));
        assert!(!rt.state.sign_in.busy && rt.state.sign_in.said.is_none());
        // Nothing of it is a toast or a line elsewhere.
        assert!(rt.state.toast.is_none() && rt.state.refused.is_none());
    }

    #[test]
    fn not_now_writes_skipped_and_a_refusal_is_said_under_the_buttons() {
        let mut rt = asking("need-skip", cloudflare(), false);
        press_on(&mut rt, "need-skip");
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-feel0", "ask": ASK, "said": "skipped" }))]);
        // The engine's own reason, when it gives one.
        lands(&mut rt, false, Ok(json!({ "ok": false, "error": "This thread is closed." })));
        assert!(!rt.state.sign_in.busy && says(&rt, "This thread is closed."));
        // And the card's own words when it gives none, or could not be reached.
        press_on(&mut rt, "need-skip");
        lands(&mut rt, false, Ok(json!({ "ok": false })));
        assert!(says(&rt, DID_NOT_REACH) && !says(&rt, "This thread is closed."));
        press_on(&mut rt, "need-skip");
        assert!(!says(&rt, DID_NOT_REACH), "a new press takes back what was said of the last");
        lands(&mut rt, false, Err("connection refused".to_string()));
        assert!(says(&rt, DID_NOT_REACH));
        assert!(rt.state.toast.is_none() && rt.state.refused.is_none());
    }

    #[test]
    fn with_no_engine_behind_the_window_the_card_says_so_itself() {
        let mut rt = asking("need-nobody", cloudflare(), false);
        rt.state.pretend_sent = false;
        press_on(&mut rt, "need-signed");
        assert!(!rt.state.sign_in.busy && says(&rt, DID_NOT_REACH) && rt.state.refused.is_none());
    }

    #[test]
    fn a_login_that_waits_for_a_code_takes_it_from_its_box() {
        let mut rt = asking("need-code", json!({ "service": "Supabase", "url": "https://supabase.com/dashboard/cli/login", "asksForCode": true }), false);
        // The code is the answer: there is no "I've signed in".
        assert!(!says(&rt, "I've signed in") && rt.find("need-signed").is_none());
        // Send takes no press over an empty box, nor does Return.
        press_on(&mut rt, "need-send-code");
        press_on(&mut rt, PASTED);
        key(&mut rt, Key::Enter);
        assert!(rt.state.tried.is_empty());
        // What is typed goes into the box and nowhere else: E is not Done here.
        type_in(&mut rt, " e42-ab ");
        assert_eq!(rt.state.sign_in.pasted.text, " e42-ab ");
        assert!(matches!(rt.state.view, View::Thread { .. }) && rt.state.tried.is_empty());
        key(&mut rt, Key::Enter);
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-feel0", "ask": ASK, "said": "code", "code": "e42-ab" }))]);
        // Refused, the code stays in its box to be sent again, by the button this time.
        lands(&mut rt, false, Ok(json!({ "ok": false })));
        assert!(says(&rt, DID_NOT_REACH));
        press_on(&mut rt, "need-send-code");
        assert_eq!(rt.state.tried.len(), 2);
    }

    #[test]
    fn a_code_to_type_on_the_services_page_is_copied_from_its_box() {
        let mut rt = asking("need-copy", json!({ "service": "GitHub", "url": "https://github.com/login/device", "code": "A1B2-C3D4" }), false);
        assert!(says(&rt, "Enter this code") && says(&rt, "A1B2-C3D4"));
        let clip = rune::clipboard::memory();
        rt.use_clipboard(&clip);
        press_on(&mut rt, "need-copy");
        assert_eq!(clip.text().map(|t| t.to_string()).as_deref(), Some("A1B2-C3D4"));
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Copied"));
        assert!(rt.state.tried.is_empty());
    }

    #[test]
    fn a_cloud_runs_sign_in_is_saved_for_every_cloud_thread_unless_unticked() {
        let mut rt = asking("need-keep", vercel(), true);
        assert!(says(&rt, "Save this sign-in for every cloud thread"));
        assert!(rt.elements().iter().any(|e| e.key.as_deref() == Some("need-keep")));
        press_on(&mut rt, "need-keep");
        assert_eq!(rt.state.tried, vec![(KEEP, json!({ "product": "storefront", "id": "w-feel0", "ask": ASK, "keep": false }))]);
        assert_eq!(rt.state.sign_in.save, Some(false));
        // The engine did not take it: the tick goes back, and the card says why.
        lands(&mut rt, true, Ok(json!({ "ok": false })));
        assert_eq!(rt.state.sign_in.save, Some(true));
        assert!(says(&rt, DID_NOT_REACH));
        // Taken, it stays as pressed, and the buttons were never held for it.
        press_on(&mut rt, "need-keep");
        lands(&mut rt, true, Ok(json!({ "ok": true })));
        assert_eq!(rt.state.sign_in.save, Some(false));
        assert!(!rt.state.sign_in.busy && !says(&rt, DID_NOT_REACH));
        // An ask the person had unticked before comes up unticked.
        let mut unticked = vercel();
        unticked["keep"] = json!(false);
        let mut rt = asking("need-kept-off", unticked, true);
        press_on(&mut rt, "need-keep");
        assert_eq!(rt.state.tried, vec![(KEEP, json!({ "product": "storefront", "id": "w-feel0", "ask": ASK, "keep": true }))]);
    }

    #[test]
    fn a_link_that_comes_back_to_localhost_offers_a_cloud_run_the_address_the_browser_ended_on() {
        // On this Mac the page loads, so there is nothing to hand over.
        let here = asking("need-also-here", vercel(), false);
        assert!(here.find("need-also").is_none());
        let mut rt = asking("need-also", vercel(), true);
        assert!(says(&rt, "Page did not load after signing in?") && rt.find(LANDED).is_none());
        press_on(&mut rt, "need-also");
        run(&mut rt, 0.1);
        // The box takes the place of the question, and the keys.
        assert!(!says(&rt, "Page did not load after signing in?") && rt.find(LANDED).is_some());
        type_in(&mut rt, "http://localhost:8976/callback?code=9f2");
        assert_eq!(rt.state.sign_in.landed.text, "http://localhost:8976/callback?code=9f2");
        press_on(&mut rt, "need-send-address");
        assert_eq!(rt.state.tried, vec![(ANSWER, json!({ "product": "storefront", "id": "w-feel0", "ask": ASK, "said": "landed", "url": "http://localhost:8976/callback?code=9f2" }))]);
    }

    #[test]
    fn escape_lets_go_of_a_box_and_leaving_the_conversation_starts_the_card_again() {
        let mut rt = asking("need-leave", json!({ "service": "Supabase", "url": "https://supabase.com/dashboard/cli/login", "asksForCode": true }), false);
        press_on(&mut rt, PASTED);
        type_in(&mut rt, "12ab");
        // The first Escape is the box's: the conversation stays.
        key(&mut rt, Key::Escape);
        assert!(matches!(rt.state.view, View::Thread { .. }));
        assert_eq!(rt.state.sign_in.pasted.text, "12ab");
        // The next is the page's, and what was typed goes with the card.
        key(&mut rt, Key::Escape);
        assert!(!matches!(rt.state.view, View::Thread { .. }));
        assert!(rt.state.sign_in.pasted.text.is_empty() && rt.state.sign_in.of.is_none());
    }

    #[test]
    fn an_answer_that_lands_after_the_card_went_changes_nothing() {
        let mut rt = asking("need-late", cloudflare(), false);
        press_on(&mut rt, "need-signed");
        key(&mut rt, Key::Escape);
        lands(&mut rt, false, Ok(json!({ "ok": false })));
        assert!(rt.state.sign_in.said.is_none() && !rt.state.sign_in.busy);
        // Nor does a command key reach the card's boxes.
        rt.key(KeyEvent { key: Key::Command('k'), mods: Mods { cmd: true, ..Default::default() } });
        rt.frame(1.0 / 60.0);
        assert!(rt.state.sign_in.of.is_none());
    }

    #[test]
    fn a_picture_taken_after_a_press_is_drawn_as_it_was() {
        let pressed = |name: &str, facts: Value| {
            window_dressed(name, &["--thread", "w-feel0"], "/fixtures/storefront", "Done.", |fixture| {
                let now = fixture["now"].as_f64().unwrap();
                let mut ask = vercel();
                ask["id"] = json!(ASK);
                let item = &mut fixture["snapshot"]["items"][0];
                item["status"] = json!("claimed");
                item["signIn"] = ask;
                fixture["snapshot"]["supervisor"]["running"] = json!([{ "itemId": "w-feel0", "product": "storefront", "startedAt": now - 300_000.0, "where": "cloud", "tail": [], "activity": [] }]);
                for (fact, value) in facts.as_object().unwrap() {
                    fixture[fact] = value.clone();
                }
            })
        };
        let refused = pressed("need-pic-refused", json!({ "signInPressed": ["signed-in"], "signInRefused": true }));
        assert!(says(&refused, DID_NOT_REACH) && !refused.state.sign_in.busy);
        let busy = pressed("need-pic-busy", json!({ "signInPressed": ["skipped"], "signInHangs": true }));
        assert!(busy.state.sign_in.busy && !says(&busy, DID_NOT_REACH));
        let pasting = pressed("need-pic-pasting", json!({ "signInPressed": ["also"], "signInTyped": { "landed": "http://localhost:8976/x" } }));
        assert!(pasting.state.sign_in.pasting && pasting.state.sign_in.landed.focused);
        assert_eq!(pasting.state.sign_in.landed.text, "http://localhost:8976/x");
    }
}
