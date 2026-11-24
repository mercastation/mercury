//! What a cloud run that cannot go on asks the person for, drawn as a card where its marked
//! line stood (`renderer/src/components/NeedCard.tsx`), and what the reply box of such a thread
//! does with a key or a code typed into it (`sendReply` in `Focus.tsx`). `rules/needs.rs` reads
//! the lines and `rules/key_asks.rs` checks what is typed; `docs/parity-cloud-needs.md` is the
//! checklist this was built from.
//!
//! The cards are built from `need.rs`'s kit. Each keeps what the web window's component keeps,
//! by the thread, the message and its place among the message's cards; leaving the
//! conversation lets go of all of it, as the web window's are unmounted. What outlives a card is
//! the window's own, as the web window's `Focus` keeps it: the keys saved from here
//! (`keysJustSaved`) and the files held for a sign-in still open (`HELD`), by name and words
//! only. A key's value is in its box and in the one write that saves it, nowhere else.

use std::collections::HashMap;

use rune::file_dialog::FileDialogOptions;
use rune::{Cursor, El, Span, TextEdit, TextStyle, Tx, col, icon, row, text, text_input};
use serde_json::{Map, Value, json};

use crate::State;
use crate::data::{Item, truthy};
use crate::files::Files;
use crate::icons;
use crate::markdown::{self, Block};
use crate::need::{self, LINE, RADIUS};
use crate::rules::js::{self, re};
use crate::rules::key_asks::{self, KeySource, Place};
use crate::rules::needs::{self, Need};
use crate::theme::*;
use crate::write::{Landed, Sent};

/// The engine's channels for what a cloud run asks for (`main/ipc.mts`).
pub const CLOUD_KEY: &str = "zero:cloud-key";
pub const GITHUB_CLOUD_SIGN_IN: &str = "zero:github-cloud-sign-in";
pub const CLOUD_FILE_INFO: &str = "zero:cloud-file-info";
pub const CLOUD_SEND_FILE: &str = "zero:cloud-send-file";
pub const CLOUD_SEND_CODE: &str = "zero:cloud-send-code";

const KEY_NOT_TAKEN: &str = "The cloud did not take the key. Try again.";
const FILE_NOT_SENT: &str = "The file could not be sent. Try again.";
const CODE_NOT_SENT: &str = "The code could not be sent. Try again.";
const GITHUB_DID_NOT_FINISH: &str = "GitHub's sign-in did not finish.";
/// The way out, as the reply it sends (`onComputer`).
pub const FROM_MY_COMPUTER: &str = "Do it from my computer.";
const WAY_OUT: &str = "Do it on my computer instead";
const KEPT: &str = "Kept encrypted in your Mercury Cloud";
const EVERY_THREAD: &str = "Every thread can use it from now on";
const FILE_FACT: &str = "Goes only to this thread's cloud machine, and is deleted when it ends";
/// The title of this window's own file panel: the words the app's panel opens with
/// (`dialog.showOpenDialog` in `zero:cloud-send-file`).
const PANEL_TITLE: &str = "Pick the file to send";

/* ------------------------------------------ what is kept ------------------------------------------ */

/// What a file on this computer is, as the engine says (`zero:cloud-file-info`).
#[derive(Clone, Debug, PartialEq)]
pub struct Info {
    pub ok: bool,
    pub size: Option<f64>,
    pub error: Option<String>,
}

impl Info {
    fn of(v: &Value) -> Info {
        Info { ok: truthy(v.get("ok")), size: v.get("size").and_then(Value::as_f64), error: v.get("error").and_then(Value::as_str).map(str::to_string) }
    }
}

/// The boxes for a sign-in's or a key's variables (`KeyFields`), each hiding what is typed.
#[derive(Default)]
pub struct Fields {
    pub values: Vec<TextEdit>,
    pub said: Option<String>,
    /// The values a warning was said over: Save, pressed again over them, saves.
    warned: Option<Vec<String>>,
    pub busy: bool,
}

impl Fields {
    fn now(&self) -> Vec<String> {
        self.values.iter().map(|v| v.text.clone()).collect()
    }
    fn anyway(&self) -> bool {
        self.warned.as_ref().is_some_and(|w| *w == self.now())
    }
}

/// GitHub's own sign-in on a card (`GitHubOwnSignIn`).
#[derive(Default)]
pub struct Gh {
    pub device: Option<(String, String)>,
    pub busy: bool,
    pub said: Option<String>,
}

/// One card's own state, as the web window's `NeedCard` and the parts inside it keep it.
pub struct Card {
    pub busy: bool,
    pub said: Option<String>,
    pub code: TextEdit,
    pub replacing: bool,
    pub late: bool,
    pub sent_late: bool,
    pub pasting: bool,
    pub fields: Fields,
    pub here_busy: bool,
    pub here_said: Option<String>,
    pub gh: Gh,
}

impl Default for Card {
    fn default() -> Self {
        Card { busy: false, said: None, code: TextEdit::new(""), replacing: false, late: false, sent_late: false, pasting: false, fields: Fields::default(), here_busy: false, here_said: None, gh: Gh::default() }
    }
}

/// A file sent while another ask on the same message was open, with the words that go when the
/// last is given (`HELD`).
#[derive(Clone, Debug, PartialEq)]
pub struct Held {
    pub need: String,
    pub words: String,
}

/// What the window keeps of the cards and round them.
#[derive(Default)]
pub struct Needs {
    cards: HashMap<String, Card>,
    /// The sign-ins and keys saved from this window, by name, which count as saved before the
    /// engine's lists say so (`keysJustSaved`).
    pub just_saved: Vec<String>,
    /// By thread id, for the window's life (`HELD`, `sessionStorage`).
    pub held: HashMap<String, Vec<Held>>,
    /// What the files asked for are, by `product/id|path`, asked once a visit.
    infos: HashMap<String, Option<Info>>,
    /// A save or a send went: the next card still open takes the keys (`focusNextAsk`).
    focus_next: bool,
    /// The words a send left in the reply box of `product/id`, which are not kept as its draft
    /// any more (`clearDraft` before `onSend`): kept again once they change.
    pub unkept: Option<(String, String)>,
}

thread_local! {
    /// The cards still open on the page last drawn, in order, each with its first box when
    /// that is the first thing in it to take the keys.
    static OPEN: std::cell::RefCell<Vec<(String, Option<String>)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// What a write of a card's is, for what is said and done once the engine answers.
#[derive(Clone, Debug, PartialEq)]
pub enum Wrote {
    Keys(Saving),
    GitHubStart { card: String, ask: Ask },
    GitHubToken { card: String, ask: Ask },
    Info { at: String },
    File { card: String, item: String, need: String, late: bool, waits_on: Vec<String>, clears: bool },
    Code { card: Option<String>, item: String, clears: bool },
}

/// Where a save's refusal is said: under the boxes, under the press that read the key where it
/// is, under GitHub's own sign-in, or as a line, for a key pasted into the reply box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Part {
    Fields,
    Here,
    GitHub,
    Reply,
}

/// A save on its way, and what is said once it lands.
#[derive(Clone, Debug, PartialEq)]
pub struct Saving {
    card: Option<String>,
    part: Part,
    item: String,
    names: Vec<String>,
    said: String,
    connected: bool,
    late: bool,
    asks: Vec<String>,
    left: Vec<String>,
    clears: bool,
}

/// The card a press is on: its key, its thread, its need, and whether it is the live one.
#[derive(Clone, Debug, PartialEq)]
pub struct Ask {
    card: String,
    id: String,
    product: String,
    need: Need,
    live: bool,
}

/* ------------------------------------------- the rules round them ------------------------------------------- */

/// The sign-ins and keys the cloud keeps, with those just saved here; none on a computer with
/// no cloud set up (`savedKeys` of `cloudKeys`).
pub fn saved_keys(s: &State) -> Option<Vec<String>> {
    let cloud = s.snap.rest.get("cloud").filter(|c| truthy(c.get("setUp")))?;
    let mut keys: Vec<String> = cloud.get("keys").and_then(Value::as_array).into_iter().flatten().filter_map(|k| k.get("name").and_then(Value::as_str)).map(str::to_string).collect();
    for k in &s.cloud_needs.just_saved {
        if !keys.contains(k) {
            keys.push(k.clone());
        }
    }
    Some(keys)
}

/// What the thread's newest answer asks for, on a computer with a cloud (`latestNeeds`).
fn latest_needs(s: &State, item: &Item) -> Vec<Need> {
    if saved_keys(s).is_some() { needs::needs_in(item.result.as_deref().unwrap_or("")) } else { Vec::new() }
}

fn keys_saved(need: &Need, saved: &[String]) -> bool {
    needs::keys_of(need).iter().all(|k| saved.iter().any(|s| s == k))
}

/// What the latest ask still waits on, given these keys saved and these files held (`openAfter`).
fn open_after(latest: &[Need], saved: &[String], given: &[String]) -> Vec<Need> {
    latest.iter().filter(|n| if needs::keys_of(n).is_empty() { !given.contains(&needs::need_id(n)) } else { !keys_saved(n, saved) }).cloned().collect()
}

/// A need by the name the lines say it by (`needName`).
pub fn need_name(n: &Need) -> String {
    match n {
        Need::SignIn { service, .. } => service.name.to_string(),
        Need::Key { name, what, .. } => key_asks::key_words(what).map_or_else(|| name.clone(), |w| format!("the {w}")),
        Need::File { path, .. } => path.split('/').next_back().unwrap_or(path).to_string(),
        Need::Code { about, .. } => format!("the {about} code"),
    }
}

/// A file's name (`fileName`), and the folder it is in (`folderOf`).
fn file_name(p: &str) -> String {
    p.split('/').rfind(|s| !s.is_empty()).unwrap_or(p).to_string()
}
fn folder_of(p: &str) -> Option<String> {
    let parts: Vec<&str> = p.split('/').filter(|s| !s.is_empty()).collect();
    (parts.len() > 1).then(|| parts[parts.len() - 2].to_string())
}

/// `sizeWords`.
fn size_words(n: f64) -> String {
    let (kb, mb, gb) = (1024.0, 1024.0 * 1024.0, 1024.0 * 1024.0 * 1024.0);
    if n >= gb {
        format!("{:.1} GB", n / gb)
    } else if n >= mb {
        format!("{} MB", (n / mb).round())
    } else if n >= kb {
        format!("{} KB", (n / kb).round())
    } else {
        format!("{n} bytes")
    }
}

re!(ENDS_A_SENTENCE, r"[.!?]$", "");
re!(STARTS_A_WORD, r"^[A-Z][a-z]", "");

/// `sentence`: the first letter raised and a full stop unless it ends one.
fn sentence(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    format!("{}{}", js::upper_first(s), if ENDS_A_SENTENCE.test(s) { "" } else { "." })
}

/// `lower`: a word's first letter lowered, unless it is an initialism.
fn lower(s: &str) -> String {
    if !STARTS_A_WORD.test(s) {
        return s.to_string();
    }
    let mut chars = s.chars();
    chars.next().map(|c| c.to_lowercase().chain(chars).collect()).unwrap_or_default()
}

/// What a file card says under its title: where the file is and how big, or that it is not
/// there, and why it is asked for (`fileWhy`).
pub fn file_why(path: &str, why: &str, info: Option<&Info>) -> String {
    let first = match (info, folder_of(path)) {
        (Some(info), _) if !info.ok => info.error.clone(),
        (_, Some(at)) => Some(format!("In {at} on this computer{}.", info.and_then(|i| i.size).map_or_else(String::new, |n| format!(", {}", size_words(n))))),
        _ => None,
    };
    [first.unwrap_or_default(), sentence(why)].into_iter().filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" ")
}

re!(TRAILING_STOPS, r"[.\s]+$", "");
re!(ARTICLE, r"^\s*(your|an?|the)\s+", "i");

/// The title is `keyWords(what)`; a `what` that is no more than that is not said again under it
/// (`saysOnlyTheTitle`).
fn says_only_the_title(what: &str) -> bool {
    key_asks::key_words(what).unwrap_or_default().to_lowercase() == TRAILING_STOPS.strip(&ARTICLE.strip(what)).to_lowercase()
}

/// Where a key is, as the press names it (`placeName`), and exactly (`placeItem`).
fn place_name(src: &KeySource) -> String {
    match src.place {
        Place::Keychain => "your Keychain".to_string(),
        Place::Shell => "your shell".to_string(),
        Place::File => file_name(&src.from),
    }
}
fn place_item(src: &KeySource) -> String {
    match (src.place, &src.entry) {
        (Place::Shell, _) => format!("${}", src.from),
        (_, Some(entry)) => format!("{entry} in {}", src.from),
        _ => src.from.clone(),
    }
}

/// The variables a sign-in or a key is saved under, with their labels.
fn fields_of(need: &Need) -> Vec<(String, String)> {
    match need {
        Need::SignIn { service, .. } => service.keys.iter().map(|(n, l)| (n.to_string(), l.to_string())).collect(),
        Need::Key { name, what, .. } => vec![(name.clone(), key_asks::key_label(what).unwrap_or_else(|| name.clone()))],
        _ => Vec::new(),
    }
}

/// Which Stripe key is kept, said where it is listed.
fn stripe_mode(name: &str, value: &str) -> &'static str {
    if !name.starts_with("STRIPE_") {
        return "";
    }
    let v = js::trim(value);
    if v.starts_with("sk_live_") || v.starts_with("rk_live_") {
        "Live "
    } else if v.starts_with("sk_test_") || v.starts_with("rk_test_") {
        "Test-mode "
    } else {
        ""
    }
}

fn source_json(src: &KeySource) -> Value {
    let mut o = Map::new();
    o.insert("place".into(), json!(src.place.word()));
    o.insert("from".into(), json!(src.from));
    if let Some(entry) = &src.entry {
        o.insert("entry".into(), json!(entry));
    }
    Value::Object(o)
}

fn thread_of(item: &Item) -> String {
    format!("{}/{}", item.product, item.id)
}

/// What an answer's field says, as the web window's `r.field ?? fallback` reads it.
fn said_in(answer: &Result<Value, String>, field: &str) -> Option<String> {
    answer.as_ref().ok().and_then(|a| a.get(field)).and_then(Value::as_str).map(str::to_string)
}

fn took(answer: &Result<Value, String>) -> bool {
    answer.as_ref().is_ok_and(|a| truthy(a.get("ok")))
}

/* ------------------------------------------------ the writes ------------------------------------------------ */

impl State {
    fn need_card(&mut self, card: &str) -> &mut Card {
        self.cloud_needs.cards.entry(card.to_string()).or_default()
    }

    /// Leaving the conversation lets go of every card and what it held.
    pub fn leave_the_cloud_cards(&mut self) {
        self.cloud_needs.cards.clear();
        self.cloud_needs.infos.clear();
        self.cloud_needs.focus_next = false;
    }

    /// What went before the last thing given: the sign-ins saved for this ask, which no reply
    /// has named yet, then what was held (`heldWords`).
    fn held_words(&self, item: &Item, latest: &[Need]) -> String {
        let keyed: Vec<Need> = latest.iter().filter(|n| !needs::keys_of(n).is_empty()).cloned().collect();
        let connected = if keyed.is_empty() { String::new() } else { without_carry_on(&needs::connected_reply(&keyed)) };
        let held = self.cloud_needs.held.get(&item.id).map(|h| h.iter().map(|h| h.words.clone()).collect::<Vec<_>>().join(" ")).unwrap_or_default();
        [connected, held].into_iter().filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" ")
    }

    fn held_here(&self, item: &Item) -> Vec<String> {
        self.cloud_needs.held.get(&item.id).map(|h| h.iter().map(|h| h.need.clone()).collect()).unwrap_or_default()
    }

    /// The keys of `need` saved, each as its own write in order, the first refusal stopping the
    /// rest (`saveKeys`, and `saveLate` for a card answered in words). `words` is the person's
    /// own, which go as the thread's reply instead of the app's.
    #[allow(clippy::too_many_arguments)]
    fn save_keys(&mut self, item: &Item, need: &Need, fields: Vec<(String, String, String, Option<KeySource>)>, words: Option<String>, late: bool, card: Option<String>, part: Part) {
        let names: Vec<String> = fields.iter().map(|f| f.0.clone()).collect();
        let said = match need {
            Need::SignIn { service, .. } => service.name.to_string(),
            _ => fields.first().map(|f| f.2.clone()).unwrap_or_default(),
        };
        let connected = matches!(need, Need::SignIn { .. });
        let reply_of = |answer: String| json!({ "product": item.product, "id": item.id, "answer": answer });
        let count = fields.len();
        let (reply, asks, left, clears) = if late {
            (Some(needs::late_reply(std::slice::from_ref(need))), names.clone(), Vec::new(), false)
        } else {
            let latest = latest_needs(self, item);
            let held = self.held_here(item);
            let mut after = saved_keys(self).unwrap_or_default();
            after.extend(names.iter().cloned());
            let left = open_after(&latest, &after, &held);
            let with_held = if !held.is_empty() && left.is_empty() {
                let keyed: Vec<Need> = latest.iter().filter(|n| !needs::keys_of(n).is_empty()).cloned().collect();
                Some(format!("{} {}{} Carry on.", without_carry_on(&needs::connected_reply(&keyed)), self.held_words(item, &latest), words.as_ref().map_or_else(String::new, |w| format!(" {w}"))))
            } else {
                words.clone()
            };
            let asks = latest.iter().flat_map(|n| needs::keys_of(n).into_iter().map(str::to_string)).collect();
            let clears = with_held != words;
            (with_held, asks, left.iter().map(need_name).collect(), clears)
        };
        let writes: Vec<crate::rules::writes::Write> = fields
            .into_iter()
            .enumerate()
            .map(|(i, (name, value, label, from))| {
                let mut o = Map::new();
                o.insert("action".into(), json!("save"));
                let mode = if late { "" } else { stripe_mode(&name, &value) };
                o.insert("name".into(), json!(name));
                o.insert("value".into(), json!(value));
                o.insert("label".into(), json!(format!("{mode}{label}")));
                if let Some(from) = &from {
                    o.insert("from".into(), source_json(from));
                }
                if let Some(answer) = reply.clone().filter(|_| i + 1 == count) {
                    o.insert("reply".into(), reply_of(answer));
                }
                (CLOUD_KEY, Value::Object(o))
            })
            .collect();
        let saving = Saving { card, part, item: item.id.clone(), names, said, connected, late, asks, left, clears };
        if !self.send_all(writes, Sent::CloudNeed(Wrote::Keys(saving.clone()))) {
            self.refused = None;
            self.keys_refused(&saving, KEY_NOT_TAKEN.to_string(), None);
        }
    }

    /// A save the engine did not take: said where it was asked from.
    fn keys_refused(&mut self, saving: &Saving, why: String, tx: Option<&mut Tx>) {
        let Some(card) = saving.card.clone() else {
            if let Some(tx) = tx {
                self.tell(&why, tx);
            }
            return;
        };
        let Some(card) = self.cloud_needs.cards.get_mut(&card) else { return };
        match saving.part {
            Part::Fields => {
                card.fields.busy = false;
                card.fields.said = Some(why);
            }
            Part::Here => {
                card.here_busy = false;
                card.here_said = Some(why);
            }
            Part::GitHub => {
                card.gh.device = None;
                card.gh.said = Some(why);
            }
            Part::Reply => {}
        }
    }

    /// The boxes' Save, and Return in a box (`KeyFields`' `go`).
    fn fields_save(&mut self, ask: &Ask) {
        let Some(item) = self.item_of(ask) else { return };
        let fields = fields_of(&ask.need);
        let card = self.need_card(&ask.card);
        if card.fields.busy {
            return;
        }
        let values = card.fields.now();
        let value = |i: usize| values.get(i).cloned().unwrap_or_default();
        if let Some((_, label)) = fields.iter().enumerate().find(|(i, _)| js::trim(&value(*i)).is_empty()).map(|(_, f)| f) {
            card.fields.said = Some(format!("Paste the {} too.", lower(label)));
            return;
        }
        let wrong = fields.iter().enumerate().find_map(|(i, (name, label))| key_asks::key_looks_wrong(name, &value(i), Some(label)));
        if let Some(wrong) = wrong
            && card.fields.warned.as_ref() != Some(&values)
        {
            card.fields.warned = Some(values);
            card.fields.said = Some(wrong);
            return;
        }
        card.fields.busy = true;
        card.fields.said = None;
        let sent = fields.into_iter().enumerate().map(|(i, (name, label))| (name, value(i), label, None)).collect();
        self.save_keys(&item, &ask.need, sent, None, !ask.live, Some(ask.card.clone()), Part::Fields);
    }

    /// "Use the one in …": the key read where the run said it is, and saved (`FromHere`).
    fn here_save(&mut self, ask: &Ask) {
        let Some(item) = self.item_of(ask) else { return };
        let Need::Key { name, from: Some(src), .. } = &ask.need else { return };
        let label = fields_of(&ask.need).first().map(|f| f.1.clone()).unwrap_or_default();
        let card = self.need_card(&ask.card);
        if card.here_busy {
            return;
        }
        card.here_busy = true;
        card.here_said = None;
        self.save_keys(&item, &ask.need, vec![(name.clone(), String::new(), label, Some(src.clone()))], None, !ask.live, Some(ask.card.clone()), Part::Here);
    }

    /// "Sign in with GitHub": GitHub's own sign-in through gh on this computer (`begin`).
    fn github_cloud_begin(&mut self, ask: &Ask) {
        let card = self.need_card(&ask.card);
        if card.gh.busy {
            return;
        }
        card.gh.busy = true;
        card.gh.said = None;
        if !self.send(GITHUB_CLOUD_SIGN_IN, json!({ "action": "start" }), Sent::CloudNeed(Wrote::GitHubStart { card: ask.card.clone(), ask: ask.clone() })) {
            self.refused = None;
            let card = self.need_card(&ask.card);
            card.gh.busy = false;
            card.gh.said = Some(need::GITHUB_DID_NOT_START.to_string());
        }
    }

    fn github_cloud_open(&mut self, card: &str) {
        if let Some((_, url)) = self.cloud_needs.cards.get(card).and_then(|c| c.gh.device.clone()) {
            crate::files::open_with_system(&url);
        }
    }

    /// Copy beside GitHub's code: onto the clipboard, and nothing said (`writeText`).
    fn github_cloud_copy(&mut self, card: &str, tx: &mut Tx) {
        if let Some((code, _)) = self.cloud_needs.cards.get(card).and_then(|c| c.gh.device.clone()) {
            tx.copy(code);
        }
    }

    /// A file sent from this computer: the path the run named, or one picked in this window's own
    /// file panel (`send` in `NeedCard`, `sendFile` in `Focus`). Closing the panel sends nothing.
    fn file_send(&mut self, ask: &Ask, choose: bool, tx: &mut Tx) {
        let Need::File { path, .. } = &ask.need else { return };
        let card = self.need_card(&ask.card);
        if card.busy {
            return;
        }
        card.busy = true;
        card.said = None;
        if !choose {
            let path = path.clone();
            return self.file_send_path(ask, path);
        }
        let ask = ask.clone();
        tx.open_file(FileDialogOptions { title: PANEL_TITLE.to_string(), ..FileDialogOptions::default() }, move |s: &mut State, picked, _| match picked {
            Ok(Some(path)) => s.file_send_path(&ask, path.to_string_lossy().into_owned()),
            Ok(None) => {
                let card = s.need_card(&ask.card);
                card.busy = false;
                if ask.live {
                    s.cloud_needs.focus_next = true;
                }
            }
            Err(e) => {
                let card = s.need_card(&ask.card);
                card.busy = false;
                card.said = Some(format!("The file picker did not open: {e}"));
            }
        });
    }

    fn file_send_path(&mut self, ask: &Ask, path: String) {
        let Some(item) = self.item_of(ask) else { return };
        let id = needs::need_id(&ask.need);
        let late = !ask.live;
        let (payload, waits_on, clears) = if late {
            (json!({ "product": item.product, "id": item.id, "path": path, "hold": false }), Vec::new(), false)
        } else {
            let latest = latest_needs(self, &item);
            let mut given = self.held_here(&item);
            given.push(id.clone());
            let wait_for = open_after(&latest, &saved_keys(self).unwrap_or_default(), &given);
            let also = self.held_words(&item, &latest);
            let mut o = Map::new();
            o.insert("product".into(), json!(item.product));
            o.insert("id".into(), json!(item.id));
            o.insert("path".into(), json!(path));
            o.insert("hold".into(), json!(!wait_for.is_empty()));
            if !also.is_empty() && wait_for.is_empty() {
                o.insert("also".into(), json!(also));
            }
            (Value::Object(o), wait_for.iter().map(need_name).collect(), !also.is_empty())
        };
        let wrote = Wrote::File { card: ask.card.clone(), item: item.id.clone(), need: id, late, waits_on, clears };
        if !self.send(CLOUD_SEND_FILE, payload, Sent::CloudNeed(wrote)) {
            self.refused = None;
            let card = self.need_card(&ask.card);
            card.busy = false;
            card.said = Some(FILE_NOT_SENT.to_string());
        }
    }

    /// A code's Send, and Return in its box (`sendCode` in `NeedCard`).
    fn code_send(&mut self, ask: &Ask, keyed_left: &[String]) {
        let Some(item) = self.item_of(ask) else { return };
        let Need::Code { about, .. } = &ask.need else { return };
        let card = self.need_card(&ask.card);
        if card.busy || js::trim(&card.code.text).is_empty() {
            return;
        }
        if !keyed_left.is_empty() {
            card.said = Some(format!("Connect {} first, then the code: it runs out in a minute.", keyed_left.join(" and ")));
            return;
        }
        let code = card.code.text.clone();
        if let Some(wrong) = key_asks::code_looks_wrong(&code)
            && card.said.as_deref() != Some(wrong.as_str())
        {
            card.said = Some(wrong);
            return;
        }
        card.busy = true;
        card.said = None;
        self.send_code(&item, about, &code, Some(ask.card.clone()));
    }

    /// The code sent as the reply that carries the thread on, with what was held
    /// (`sendCode` in `Focus`).
    fn send_code(&mut self, item: &Item, about: &str, code: &str, card: Option<String>) {
        let latest = latest_needs(self, item);
        let also = self.held_words(item, &latest);
        let mut o = Map::new();
        o.insert("product".into(), json!(item.product));
        o.insert("id".into(), json!(item.id));
        o.insert("about".into(), json!(about));
        o.insert("code".into(), json!(code));
        if !also.is_empty() {
            o.insert("also".into(), json!(also));
        }
        let wrote = Wrote::Code { card: card.clone(), item: item.id.clone(), clears: !also.is_empty() };
        if !self.send(CLOUD_SEND_CODE, Value::Object(o), Sent::CloudNeed(wrote)) {
            self.refused = None;
            if let Some(card) = card.and_then(|c| self.cloud_needs.cards.get_mut(&c)) {
                card.busy = false;
                card.said = Some(CODE_NOT_SENT.to_string());
            }
        }
    }

    /// "Do it on my computer instead": the reply "Do it from my computer.", sent as any reply is
    /// (`onReplySend`), which leaves whatever the reply box holds where it is.
    fn way_out(&mut self, ask: &Ask, tx: &mut Tx) {
        let Some(item) = self.item_of(ask) else { return };
        let thread = thread_of(&item);
        let here = self.open_item().is_some_and(|open| open.id == item.id && open.product == item.product);
        let kept = here.then(|| (self.reply.clone(), self.replying, self.drafts.get(&thread).cloned(), self.reply_picks.get(&thread).cloned(), self.reply_files.open.clone()));
        let said = crate::rules::reply_chips::Said::untouched(&item);
        self.hold_reply(&item, FROM_MY_COMPUTER.to_string(), "", None, said, Vec::new(), tx);
        if let Some((reply, replying, draft, pick, files)) = kept {
            self.reply = reply;
            self.replying = replying;
            if let Some(draft) = draft {
                self.drafts.insert(thread.clone(), draft);
            }
            if let Some(pick) = pick {
                self.reply_picks.insert(thread, pick);
            }
            self.reply_files.open = files;
        }
    }

    /// "Connect it now", "Add it now", "Send it now": the card answered in words, opened as it
    /// would be live, its first box taking the keys (`reopen`).
    fn reopen_card(&mut self, ask: &Ask, first_box: Option<String>, tx: &mut Tx) {
        let card = self.need_card(&ask.card);
        card.late = true;
        card.fields = Fields::default();
        if let Some(key) = first_box {
            tx.focus_when_shown(key, false);
        }
        if let Need::File { path, .. } = &ask.need {
            let at = format!("{}/{}|{path}", ask.product, ask.id);
            self.ask_file_info(at, path.clone());
        }
    }

    fn item_of(&self, ask: &Ask) -> Option<Item> {
        self.snap.items.iter().find(|i| i.id == ask.id && i.product == ask.product).cloned()
    }

    fn ask_file_info(&mut self, at: String, path: String) {
        if self.cloud_needs.infos.contains_key(&at) {
            return;
        }
        self.cloud_needs.infos.insert(at.clone(), None);
        if !self.send(CLOUD_FILE_INFO, json!({ "path": path }), Sent::CloudNeed(Wrote::Info { at })) {
            // Nobody to ask: the card says where the file is without its size.
            self.refused = None;
        }
    }

    /// What the engine said to one of the cards' writes.
    pub fn cloud_need_landed(&mut self, landed: Landed, tx: &mut Tx) {
        let Sent::CloudNeed(wrote) = landed.sent else { return };
        let answer = landed.answer;
        match wrote {
            Wrote::Info { at } => {
                if let Ok(v) = &answer
                    && self.cloud_needs.infos.contains_key(&at)
                {
                    self.cloud_needs.infos.insert(at, Some(Info::of(v)));
                }
            }
            Wrote::Keys(saving) => {
                if !took(&answer) {
                    let why = said_in(&answer, "error").filter(|e| !e.is_empty()).unwrap_or_else(|| KEY_NOT_TAKEN.to_string());
                    return self.keys_refused(&saving, why, Some(tx));
                }
                let out = answer.as_ref().ok().cloned().unwrap_or(Value::Null);
                if saving.clears {
                    self.cloud_needs.held.remove(&saving.item);
                }
                let names_of = |v: Option<&Value>, field: Option<&str>| -> Vec<String> {
                    v.and_then(Value::as_array).into_iter().flatten().filter_map(|k| match field { Some(f) => k.get(f).and_then(Value::as_str), None => k.as_str() }).map(str::to_string).collect()
                };
                let mut saved = names_of(out.get("keys"), Some("name"));
                saved.extend(self.cloud_needs.just_saved.iter().cloned());
                saved.extend(saving.names.iter().cloned());
                for name in &saving.names {
                    if !self.cloud_needs.just_saved.contains(name) {
                        self.cloud_needs.just_saved.push(name.clone());
                    }
                }
                let freed = names_of(out.get("freed"), None);
                let line = if saving.late {
                    key_asks::key_saved_words(&saving.said, &key_asks::Saving { asks: &saving.names, saved: &saving.names, freed: &freed, thread: &saving.item, cloud_off: truthy(out.get("cloudOff")), connected: saving.connected, left: None })
                } else {
                    key_asks::key_saved_words(&saving.said, &key_asks::Saving { asks: &saving.asks, saved: &saved, freed: &freed, thread: &saving.item, cloud_off: truthy(out.get("cloudOff")), connected: saving.connected, left: Some(&saving.left) })
                };
                self.tell(&line, tx);
                if let Some(card) = saving.card.as_ref().and_then(|c| self.cloud_needs.cards.get_mut(c)) {
                    card.fields.busy = false;
                    card.here_busy = false;
                }
                if saving.part != Part::Reply {
                    self.cloud_needs.focus_next = true;
                    tx.wake_after(std::time::Duration::from_millis(150));
                }
            }
            Wrote::GitHubStart { card, ask } => {
                let (code, url) = (said_in(&answer, "code").filter(|c| !c.is_empty()), said_in(&answer, "url").filter(|u| !u.is_empty()));
                let started = took(&answer).then_some(()).and(code.zip(url));
                let Some((code, url)) = started else {
                    let why = said_in(&answer, "error").unwrap_or_else(|| need::GITHUB_DID_NOT_START.to_string());
                    let card = self.need_card(&card);
                    card.gh.busy = false;
                    card.gh.said = Some(why);
                    return;
                };
                let c = self.need_card(&card);
                c.gh.busy = false;
                c.gh.device = Some((code, url.clone()));
                // In the person's own browser, where they are already signed in to GitHub.
                crate::files::open_with_system(&url);
                if !self.send(GITHUB_CLOUD_SIGN_IN, json!({ "action": "token" }), Sent::CloudNeed(Wrote::GitHubToken { card: card.clone(), ask })) {
                    self.refused = None;
                    let c = self.need_card(&card);
                    c.gh.device = None;
                    c.gh.said = Some(GITHUB_DID_NOT_FINISH.to_string());
                }
            }
            Wrote::GitHubToken { card, ask } => {
                let token = took(&answer).then(|| said_in(&answer, "token")).flatten().filter(|t| !t.is_empty());
                let Some(token) = token else {
                    let why = said_in(&answer, "error").unwrap_or_else(|| GITHUB_DID_NOT_FINISH.to_string());
                    let c = self.need_card(&card);
                    c.gh.device = None;
                    c.gh.said = Some(why);
                    return;
                };
                let Some(item) = self.item_of(&ask) else { return };
                // The token goes straight into its save, and is kept nowhere else.
                self.save_keys(&item, &ask.need, vec![("GH_TOKEN".to_string(), token, "GitHub token".to_string(), None)], None, !ask.live, Some(card), Part::GitHub);
            }
            Wrote::File { card, item, need, late, waits_on, clears } => {
                if !took(&answer) {
                    let why = said_in(&answer, "error").filter(|e| !e.is_empty()).unwrap_or_else(|| FILE_NOT_SENT.to_string());
                    let c = self.need_card(&card);
                    c.busy = false;
                    c.said = Some(why);
                    return;
                }
                let out = answer.as_ref().ok().cloned().unwrap_or(Value::Null);
                let name = out.get("path").and_then(Value::as_str).map_or_else(|| "undefined".to_string(), |p| p.split('/').next_back().unwrap_or(p).to_string());
                let c = self.need_card(&card);
                c.busy = false;
                if late {
                    c.late = false;
                    c.sent_late = true;
                    return self.tell(&format!("Sent {name}. This thread has it now."), tx);
                }
                let words = out.get("words").and_then(Value::as_str).filter(|w| !w.is_empty());
                if truthy(out.get("held"))
                    && let Some(words) = words
                {
                    let list = self.cloud_needs.held.entry(item).or_default();
                    list.retain(|h| h.need != need);
                    list.push(Held { need, words: words.to_string() });
                    self.tell(&format!("{name} is ready. It goes with {}.", waits_on.join(" and ")), tx);
                } else {
                    if clears {
                        self.cloud_needs.held.remove(&item);
                    }
                    self.tell(&format!("Sent {name}. This thread carries on in the cloud."), tx);
                }
                self.cloud_needs.focus_next = true;
                tx.wake_after(std::time::Duration::from_millis(150));
            }
            Wrote::Code { card, item, clears } => {
                if !took(&answer) {
                    let why = said_in(&answer, "error").filter(|e| !e.is_empty()).unwrap_or_else(|| CODE_NOT_SENT.to_string());
                    match card.and_then(|c| self.cloud_needs.cards.get_mut(&c)) {
                        Some(c) => {
                            c.busy = false;
                            c.said = Some(why);
                        }
                        None => self.tell(&why, tx),
                    }
                    return;
                }
                if let Some(c) = card.and_then(|c| self.cloud_needs.cards.get_mut(&c)) {
                    c.busy = false;
                }
                if clears {
                    self.cloud_needs.held.remove(&item);
                }
                self.tell("Code sent. This thread carries on.", tx);
            }
        }
    }

    /// A key or a code typed into the reply box of a thread that asked for one, caught before it
    /// is drawn, written, kept as a reply on its way or held (`sendReply` in `Focus`): a code goes
    /// as the code, a key is saved, and neither goes out as words. True when it was caught.
    /// `text` is what the box sends: its words, and the links to its files once they are saved.
    pub fn catch_the_reply(&mut self, item: &Item, text: &str, tx: &mut Tx) -> bool {
        let latest = latest_needs(self, item);
        if latest.is_empty() {
            return false;
        }
        let thread = thread_of(item);
        let saved = saved_keys(self).unwrap_or_default();
        // A code typed in the box goes as the code it asked for, never as words.
        if let Some(Need::Code { about, .. }) = latest.iter().find(|n| matches!(n, Need::Code { .. }))
            && CODE_IN_A_REPLY.test(text)
        {
            let first: Vec<String> = latest.iter().filter(|n| !needs::keys_of(n).is_empty() && !keys_saved(n, &saved)).map(need_name).collect();
            if !first.is_empty() {
                self.cloud_needs.unkept = Some((thread, self.reply.text.clone()));
                self.tell(&format!("Connect {} first, then the code: it runs out in a minute.", first.join(" and ")), tx);
                return true;
            }
            self.empty_the_reply(item);
            self.send_code(item, &about.clone(), js::trim(text), None);
            return true;
        }
        let asks: Vec<String> = latest.iter().flat_map(|n| needs::keys_of(n).into_iter().map(str::to_string)).collect();
        let Some(pasted) = key_asks::key_from_reply(text, &asks, &saved) else { return false };
        let Some(need) = latest.iter().find(|n| needs::keys_of(n).contains(&pasted.name.as_str())).cloned() else { return false };
        let label = match &need {
            Need::SignIn { service, .. } => service.keys.iter().find(|(n, _)| *n == pasted.name).map_or_else(|| pasted.name.clone(), |(_, l)| l.to_string()),
            Need::Key { what, .. } => key_asks::key_label(what).unwrap_or_else(|| pasted.name.clone()),
            _ => pasted.name.clone(),
        };
        if let Some(wrong) = key_asks::key_looks_wrong(&pasted.name, &pasted.value, Some(&label)) {
            let warning = wrong.split(" Press Save again").next().unwrap_or(&wrong).to_string();
            self.cloud_needs.unkept = Some((thread, self.reply.text.clone()));
            self.tell(&format!("{warning} Nothing was sent or saved; paste it in the card on the thread to keep it anyway."), tx);
            return true;
        }
        // A sign-in of several variables cannot tell which this is: it goes as words, as the web
        // window sends it.
        if matches!(&need, Need::SignIn { service, .. } if service.keys.len() > 1) {
            return false;
        }
        self.empty_the_reply(item);
        let words = pasted.words.then(|| text.replacen(&pasted.value, &format!("[{label}, saved]"), 1));
        self.save_keys(item, &need, vec![(pasted.name, pasted.value, label, None)], words, false, None, Part::Reply);
        true
    }

    /// The reply box folded and let go of, as `onReplyClose` closes it once its draft was
    /// cleared: its words, its files and its pick.
    fn empty_the_reply(&mut self, item: &Item) {
        let thread = thread_of(item);
        let here = self.open_item().is_some_and(|open| open.id == item.id && open.product == item.product);
        self.drafts.remove(&thread);
        self.reply_picks.remove(&thread);
        self.reply_files.clear(&thread, here);
        if here {
            self.reply_menu = None;
            self.reply = TextEdit::new("");
            self.replying = false;
        }
        self.cloud_needs.unkept = None;
    }
}

re!(CODE_IN_A_REPLY, r"^\s*\d{4,8}\s*$", "");

fn without_carry_on(said: &str) -> String {
    said.strip_suffix(" Carry on.").unwrap_or(said).to_string()
}

/// Every turn: the size of each file the open thread's newest answer asks for, once a visit
/// (`fileInfo` in the card's effect); and the keys handed to the next card still open after a
/// save or a send (`focusNextAsk`).
pub fn tend(s: &mut State, tx: &mut Tx) -> bool {
    let Some(item) = s.open_item().cloned() else { return false };
    if saved_keys(s).is_some() && item.status == "open" && !needs::spoken_since(&item) {
        for need in needs::needs_in(item.result.as_deref().unwrap_or("")) {
            if let Need::File { path, .. } = need {
                s.ask_file_info(format!("{}|{path}", thread_of(&item)), path);
            }
        }
    }
    if s.cloud_needs.focus_next {
        s.cloud_needs.focus_next = false;
        if let Some(key) = OPEN.with(|open| open.borrow().first().cloned()).and_then(|(_, first_box)| first_box) {
            tx.focus_when_shown(key, false);
        }
        return true;
    }
    // A box's words that a caught send left there are kept as its draft again once they change.
    if let Some((thread, words)) = &s.cloud_needs.unkept
        && (thread_of(&item) != *thread || s.reply.text != *words)
    {
        s.cloud_needs.unkept = None;
    }
    false
}

/// Whether the open box's words are the ones a caught send left there, which are not kept.
pub fn unkept(s: &State, thread: &str, words: &str) -> bool {
    s.cloud_needs.unkept.as_ref().is_some_and(|(t, w)| t == thread && w == words)
}

/* ------------------------------------------------ the drawing ------------------------------------------------ */

/// What a card counts over: the needs of its own message, the keys saved, the files held, and
/// whether its message takes anything (`NeedPlace`).
struct Asked<'a> {
    item: &'a Item,
    saved: &'a [String],
    given: Vec<String>,
    live: bool,
    needs: Vec<Need>,
}

impl Asked<'_> {
    fn done(&self, n: &Need) -> bool {
        if needs::keys_of(n).is_empty() { self.given.contains(&needs::need_id(n)) } else { keys_saved(n, self.saved) }
    }
    /// The way a card names what is still open (`keyedLeft`, `othersLeft`).
    fn short_name(n: &Need) -> String {
        match n {
            Need::SignIn { service, .. } => service.name.to_string(),
            Need::Key { name, what, .. } => key_asks::key_words(what).unwrap_or_else(|| name.clone()),
            Need::File { path, .. } => file_name(path),
            Need::Code { about, .. } => format!("the {about} code"),
        }
    }
}

/// An agent's message `text` as blocks: its words as markdown, and each need line a card where
/// it stood (`md` with `needsAsBlocks`). `at` names the message among the thread's, and `older`
/// says it is not the agent's last word in a conversation (`NeedOlder`). On a computer with no
/// cloud the line is the block the web window falls back on, the need as JSON in a code block.
#[allow(clippy::too_many_arguments)]
pub fn blocks(s: &State, item: &Item, text: &str, at: f64, older: bool, size: f32, column: f32, files: &Files) -> Vec<Block> {
    let found = needs::needs_in(text);
    if found.is_empty() {
        return markdown::blocks(text, size, column, files);
    }
    let Some(saved) = saved_keys(s) else {
        return markdown::blocks(&key_asks::needs_as_blocks(text), size, column, files);
    };
    // Said to since, or not open, it has been answered.
    let live = key_asks::is_newest_answer(text, item.result.as_deref()) && item.status == "open" && !needs::spoken_since(item) && !older;
    let place = Asked { item, saved: &saved, given: s.held_here(item), live, needs: found };
    let mut out: Vec<Block> = Vec::new();
    let mut words: Vec<&str> = Vec::new();
    let mut ordinal = 0;
    let mut card_last = false;
    let flush = |words: &mut Vec<&str>, out: &mut Vec<Block>, card_last: &mut bool| {
        if words.iter().any(|w| !js::trim(w).is_empty()) {
            out.extend(markdown::blocks(&words.join("\n"), size, column, files));
            *card_last = false;
        }
        words.clear();
    };
    for line in text.split('\n') {
        let Some(need) = needs::needs_in(line).into_iter().next() else {
            words.push(line);
            continue;
        };
        flush(&mut words, &mut out, &mut card_last);
        let slug = format!("{}-{ordinal}", at as i64);
        ordinal += 1;
        let card = card(s, &place, &need, &slug, column);
        // `.need`: 16 over and 6 under, 10 between two (`.need + .need`).
        out.push(markdown::block(if card_last { 10.0 } else { 16.0 }, 6.0, card));
        card_last = true;
    }
    flush(&mut words, &mut out, &mut card_last);
    out
}

/// A key with the card's own name after it, so each card's controls are its own.
fn k(base: &str, slug: &str) -> String {
    format!("{base}~{slug}")
}

/// The card's title (`.need-title`).
fn title(words: &str) -> El {
    readable(pen(text(words).font(TextStyle::new(15.0, 500.0).lh(22.0).tracking(-0.09)).color(ink()), 15.0))
}

re!(BACKTICKED, r"`([^`]+)`", "g");
re!(LINK, r"(https?:\/\/[^\s)]+|\b[a-z0-9-]+(?:\.[a-z0-9-]+)*\.(?:com|io|dev|app|net|org|co|ai)\/[^\s),]*)", "gi");
re!(HTTP, r"^https?:", "i");

/// A line under the title, its code and its links as the web window draws them
/// (`.need-why`, `withLinks`): code in the mono face, a link in the text's ink, underlined,
/// opened in the browser.
fn why_line(words: &str, size: f32) -> El {
    let mut flat = String::new();
    let mut spans: Vec<Span> = Vec::new();
    let mut links: Vec<rune::Link> = Vec::new();
    let mut last = 0;
    let mut runs: Vec<(bool, &str)> = Vec::new();
    for m in BACKTICKED.each(words) {
        let (a, b) = m.span(0).unwrap_or((0, 0));
        runs.push((false, &words[last..a]));
        runs.push((true, m.get(1).unwrap_or("")));
        last = b;
    }
    runs.push((false, &words[last..]));
    for (code, run) in runs {
        if code {
            // The box's room is a space before and after, sized to its padding, on the chip a
            // conversation's code wears (`.agent-thread.markdown :not(pre) > code`), as the
            // markdown draws it.
            let pad = Span::new(0, 0).mono().size(markdown::CODE_PAD / markdown::MONO_ADVANCE).bg(white(0.07), 0.0);
            let a = flat.len();
            flat.push('\u{a0}');
            let b = flat.len();
            flat.push_str(run);
            let c = flat.len();
            flat.push('\u{a0}');
            spans.push(Span { start: a as u32, end: b as u32, ..pad.clone() });
            spans.push(Span::new(b, c).mono().size(markdown::CODE).bg(white(0.07), 0.0));
            spans.push(Span { start: c as u32, end: flat.len() as u32, ..pad });
            continue;
        }
        let mut from = 0;
        for m in LINK.each(run) {
            let (a, b) = m.span(0).unwrap_or((0, 0));
            flat.push_str(&run[from..a]);
            let part = &run[a..b];
            let start = flat.len();
            flat.push_str(part);
            spans.push(Span::new(start, flat.len()).color(ink()).underline(ink()));
            let href = if HTTP.test(part) { part.to_string() } else { format!("https://{part}") };
            links.push(rune::Link::new(start..flat.len(), href));
            from = b;
        }
        flat.push_str(&run[from..]);
    }
    let mut line = text(flat.as_str()).font(TextStyle::new(size, 400.0).lh(20.0)).color(dim()).wrap().spans(spans);
    let linked = !links.is_empty();
    for link in links {
        line = line.link(link);
    }
    if linked {
        line = line.cursor(Cursor::Default).on_link(|_: &mut State, href: &str, _: &mut Tx| crate::files::open_with_system(href));
    }
    readable(pen(line, size))
}

/// A card's frame and body: the mark, then what it says (`.need-body`); `done` cards stand their
/// parts on their middles (`.need-done`).
fn body(mark: El, said: El, end: Option<El>, done: bool) -> El {
    let mut b = row().gap(16.0).px(22.0).py(16.0).child(mark).child(said);
    b = if done { b.items_center() } else { b.items_start() };
    if let Some(end) = end {
        b = b.child(end);
    }
    b
}

/// A done card's line under its title (`.need-done .need-why`).
fn done_why(words: &str) -> El {
    readable(pen(text(words).font(TextStyle::new(13.0, 400.0).lh(20.0)).color(dim()).wrap(), 13.0)).mt(1.0)
}

/// The way out, under the last card still open (`Footer`, `.need-foot`): the laptop, the
/// words, the arrow; the words take the text's ink under the pointer.
fn foot(ask: &Ask, slug: &str) -> El {
    let ask = ask.clone();
    let words = pen(text(WAY_OUT).font(TextStyle::new(13.0, 400.0).lh(kept(LINE))).color(dim()).group_color(ink()), 13.0);
    row()
        .key(k("need-foot", slug))
        .role("button")
        .label(WAY_OUT)
        .items_center()
        .gap(8.0)
        .pt(10.0)
        .pb(10.0)
        .pl(78.0)
        .pr(22.0)
        .border_sides([1.0, 0.0, 0.0, 0.0], edge())
        .border_layout()
        .hover_group()
        .child(icon(&icons::NEED_LAPTOP, 14.0).color(muted()))
        .child(words)
        .child(icon(&icons::NEED_OUT, 12.0).color(muted()).ml(2.0))
        .on_press_tx(move |s: &mut State, tx| s.way_out(&ask, tx))
}

/// The facts under a sign-in or a key (`.need-facts`).
fn facts(rows: Vec<El>) -> El {
    col().mt(12.0).px(14.0).py(10.0).gap(7.0).radius(RADIUS).border(1.0, edge()).border_layout().children(rows)
}

/// "Get {a} {plain} from {Service}'s own page", the page a link.
fn get_it(plain: &str, name: &str, url: &str, size: f32, lh: f32, end: &str) -> El {
    let article = if plain.chars().next().is_some_and(|c| "aeiouAEIOU".contains(c)) { "an" } else { "a" };
    let lead = format!("Get {article} {plain} from ");
    let page = format!("{name}'s own page");
    let whole = format!("{lead}{page}{end}");
    let (a, b) = (lead.len(), lead.len() + page.len());
    pen(
        text(whole.as_str())
            .font(TextStyle::new(size, 400.0).lh(lh))
            .color(dim())
            .wrap()
            .spans(vec![Span::new(a, b).color(ink()).underline(ink())])
            .link(rune::Link::new(a..b, url.to_string()))
            .cursor(Cursor::Default)
            .on_link(|_: &mut State, href: &str, _: &mut Tx| crate::files::open_with_system(href)),
        size,
    )
}

/// The card of one need of a message (`NeedCard`).
fn card(s: &State, place: &Asked, need: &Need, slug: &str, column: f32) -> El {
    let key = format!("{}|{slug}", thread_of(place.item));
    let fresh = Card::default();
    let here = s.cloud_needs.cards.get(&key).unwrap_or(&fresh);
    let ask = Ask { card: key.clone(), id: place.item.id.clone(), product: place.item.product.clone(), need: need.clone(), live: place.live };
    let taking = place.live || here.late;
    let id = needs::need_id(need);
    let at = place.needs.iter().position(|n| needs::need_id(n) == id);
    let first_keyed = place.needs.iter().position(|n| matches!(n, Need::SignIn { .. } | Need::Key { .. }) && !place.done(n));
    let show_facts = at.is_none() || at == first_keyed;
    let last_open = place.needs.iter().rposition(|n| !place.done(n));
    let show_foot = at.is_none() || at == last_open;
    let keyed_left: Vec<String> = place.needs.iter().filter(|n| !needs::keys_of(n).is_empty() && !place.done(n)).map(|n| match n {
        Need::SignIn { service, .. } => service.name.to_string(),
        Need::Key { name, what, .. } => key_asks::key_words(what).unwrap_or_else(|| name.clone()),
        _ => String::new(),
    }).collect();
    let others_left: Vec<String> = place.needs.iter().filter(|n| !place.done(n) && needs::need_id(n) != id).map(Asked::short_name).collect();
    let scale = s.pictures.scale();
    let frame = |inner: El, done: bool| need::framed(k(if done { "need-done" } else { "need-cloud" }, slug), column, inner);
    let held = |t: String, mark: El| {
        let why = if others_left.is_empty() { "Sent.".to_string() } else { format!("Ready. It goes with {}.", others_left.join(" and ")) };
        frame(body(mark, col().grow(1.0).min_w(0.0).child(title(&t)).child(done_why(&why)), None, true), true)
    };
    let mut open_first: Option<Option<String>> = None;

    let drawn = match need {
        Need::SignIn { .. } | Need::Key { .. } => {
            let (svc, key_need) = match need {
                Need::SignIn { service, .. } => (Some(*service), None),
                Need::Key { name, what, from } => (None, Some((name, what, from))),
                _ => unreachable!(),
            };
            let fields = fields_of(need);
            let saved = fields.iter().all(|(n, _)| place.saved.contains(n));
            let name = match (svc, key_need) {
                (Some(svc), _) => svc.name.to_string(),
                (_, Some((n, what, _))) => key_asks::key_label(what).unwrap_or_else(|| n.clone()),
                _ => String::new(),
            };
            let heading = match (svc, key_need) {
                (Some(svc), _) => format!("Connect {}", svc.name),
                (_, Some((n, what, _))) => format!("Add the {}", key_asks::key_words(what).unwrap_or_else(|| n.clone())),
                _ => String::new(),
            };
            let mark = need::mark_of(svc.map(|s| s.id), &icons::NEED_KEY, scale);
            if (saved && !here.replacing) || !taking {
                let (t, why) = if saved { (if svc.is_some() { format!("{name} connected") } else { format!("{name} saved") }, "Every cloud thread can use it.") } else { (heading.clone(), "Answered below.") };
                let end = if saved && place.live {
                    let card = key.clone();
                    Some(need::link(k("need-replace", slug), "Replace").on_press(move |s: &mut State| {
                        let c = s.need_card(&card);
                        c.replacing = true;
                        c.fields = Fields::default();
                    }))
                } else if !saved {
                    let ask = ask.clone();
                    let first = fields_first_box(svc, key_need.and_then(|(_, _, f)| f.as_ref()), slug);
                    Some(need::link(k("need-now", slug), if svc.is_some() { "Connect it now" } else { "Add it now" }).on_press_tx(move |s: &mut State, tx| s.reopen_card(&ask, first.clone(), tx)))
                } else {
                    None
                };
                let mut c = col().child(body(mark, col().grow(1.0).min_w(0.0).child(title(&t)).child(done_why(why)), end, true));
                if saved && place.live && show_foot {
                    c = c.child(foot(&ask, slug));
                }
                frame(c, true)
            } else {
                let token_url = svc.map(|s| s.token_url);
                // What the person fetches, in the service's own word.
                let plain = match svc {
                    Some(svc) if svc.keys.len() > 1 => "access key".to_string(),
                    Some(svc) => {
                        let rest = svc.keys[0].1.replacen(svc.name, "", 1);
                        let rest = js::trim(&rest);
                        lower(if rest.is_empty() { "token" } else { rest })
                    }
                    None => "key".to_string(),
                };
                let own = svc.is_some_and(|s| s.id == "github") && !here.pasting;
                let mut said = col().grow(1.0).min_w(0.0).child(title(&heading));
                match need {
                    Need::SignIn { why, .. } => {
                        let why = sentence(why);
                        if !why.is_empty() {
                            said = said.child(why_line(&why, 13.5).mt(3.0));
                        }
                    }
                    Need::Key { what, .. } if !what.is_empty() && !says_only_the_title(what) => said = said.child(why_line(&sentence(what), 13.5).mt(3.0)),
                    _ => {}
                }
                let get = |size: f32, lh: f32, end: &str| match (svc, token_url) {
                    (Some(svc), Some(url)) if !own => Some(get_it(&plain, svc.name, url, size, lh, end)),
                    _ => None,
                };
                if show_facts {
                    let mut rows = Vec::new();
                    if let Some(get) = get(13.0, 18.0, "") {
                        rows.push(need::fact(&icons::NEED_OUT, 12.0, get));
                    }
                    rows.push(need::fact(&icons::NEED_LOCK, 13.0, need::fact_words(KEPT)));
                    rows.push(need::fact(&icons::NEED_CLOUD, 13.0, need::fact_words(EVERY_THREAD)));
                    said = said.child(facts(rows));
                } else if let Some(get) = get(13.5, 20.0, ". Kept the same way.") {
                    said = said.child(readable(get).mt(6.0));
                }
                let from = key_need.and_then(|(_, _, f)| f.clone());
                if let Some(src) = &from {
                    // The key where the run said it is, read and saved in one press (`FromHere`).
                    let a = ask.clone();
                    let mut use_it = need::button(k("need-here", slug), &if here.here_busy { "Reading".to_string() } else { format!("Use the one in {}", place_name(src)) }, true, false, here.here_busy);
                    if !here.here_busy {
                        use_it = use_it.on_press(move |s: &mut State| s.here_save(&a));
                    }
                    let item_words = place_item(src);
                    let spot = row().min_w(0.0).help(item_words.as_str()).child(pen(text(item_words.as_str()).font(mono(12.5, 20.0)).color(muted()).truncate(), 12.5));
                    said = said.child(row().mt(14.0).gap(12.0).items_center().child(use_it).child(spot));
                    if let Some(why) = &here.here_said {
                        said = said.child(need::told(why));
                    }
                    open_first.get_or_insert(None);
                }
                if own {
                    said = said.child(github_own(here, &ask, slug));
                    open_first.get_or_insert(None);
                } else {
                    let (form, first) = key_fields(here, &ask, &fields, &plain, from.is_some(), slug);
                    said = said.child(form.mt(if from.is_some() && here.here_said.is_none() { 10.0 } else { 14.0 }));
                    open_first.get_or_insert(Some(first));
                }
                let mut c = col().child(body(mark, said, None, false));
                if show_foot && place.live {
                    c = c.child(foot(&ask, slug));
                }
                frame(c, false)
            }
        }
        Need::File { path, why } => {
            let name = file_name(path);
            let mark = need::mark_of(None, &icons::NEED_FILE, scale);
            if place.live && place.given.contains(&id) {
                held(format!("{name} is ready"), mark)
            } else if !taking {
                let (t, w) = if here.sent_late { (format!("{name} sent"), "This thread has it now.") } else { (format!("Send {name}"), "Answered below.") };
                let end = (!here.sent_late).then(|| {
                    let ask = ask.clone();
                    need::link(k("need-now", slug), "Send it now").on_press_tx(move |s: &mut State, tx| s.reopen_card(&ask, None, tx))
                });
                frame(body(mark, col().grow(1.0).min_w(0.0).child(title(&t)).child(done_why(w)), end, true), true)
            } else {
                let info = s.cloud_needs.infos.get(&format!("{}|{path}", thread_of(place.item))).cloned().flatten();
                let mut said = col().grow(1.0).min_w(0.0).child(title(&format!("Send {name}")));
                said = said.child(readable(pen(text(file_why(path, why, info.as_ref()).as_str()).font(TextStyle::new(13.5, 400.0).lh(20.0)).color(dim()).wrap(), 13.5)).mt(3.0));
                said = said.child(facts(vec![need::fact(&icons::NEED_LOCK, 13.0, need::fact_words(FILE_FACT))]));
                let busy = here.busy;
                let mut acts = row().mt(14.0).gap(8.0);
                let (a, b) = (ask.clone(), ask.clone());
                if info.as_ref().is_some_and(|i| !i.ok) {
                    let mut choose = need::button(k("need-choose", slug), if busy { "Sending" } else { "Choose the file" }, true, false, busy);
                    if !busy {
                        choose = choose.on_press_tx(move |s: &mut State, tx| s.file_send(&a, true, tx));
                    }
                    acts = acts.child(choose);
                } else {
                    let mut send = need::button(k("need-send-file", slug), if busy { "Sending" } else { "Send from this computer" }, true, false, busy);
                    let mut other = need::button(k("need-choose", slug), "Choose another file", false, false, busy);
                    if !busy {
                        send = send.on_press_tx(move |s: &mut State, tx| s.file_send(&a, false, tx));
                        other = other.on_press_tx(move |s: &mut State, tx| s.file_send(&b, true, tx));
                    }
                    acts = acts.child(send).child(other);
                }
                said = said.child(acts);
                if let Some(why) = &here.said {
                    said = said.child(need::told(why));
                }
                open_first = Some(None);
                let mut c = col().child(body(mark, said, None, false));
                if show_foot && place.live {
                    c = c.child(foot(&ask, slug));
                }
                frame(c, false)
            }
        }
        Need::Code { about, why } => {
            let mark = need::mark_of(Some(about.as_str()), &icons::NEED_KEY, scale);
            if !place.live {
                frame(body(mark, col().grow(1.0).min_w(0.0).child(title(&format!("Enter the {about} code"))).child(done_why("Answered below.")), None, true), true)
            } else {
                let mut said = col().grow(1.0).min_w(0.0).child(title(&format!("Enter the {about} code")));
                if !why.is_empty() {
                    said = said.child(readable(pen(text(format!("{} Goes to this thread only.", sentence(why)).as_str()).font(TextStyle::new(13.5, 400.0).lh(20.0)).color(dim()).wrap(), 13.5)).mt(3.0));
                }
                if !keyed_left.is_empty() {
                    let words = format!("Enter it once {} {} in: these codes run out in a minute.", keyed_left.join(" and "), if keyed_left.len() > 1 { "are" } else { "is" });
                    said = said.child(readable(pen(text(words.as_str()).font(TextStyle::new(13.5, 400.0).lh(20.0)).color(dim()).wrap(), 13.5)).mt(6.0));
                }
                let box_key = k("need-otp", slug);
                let (a, b) = (ask.clone(), ask.clone());
                let (left_a, left_b) = (keyed_left.clone(), keyed_left.clone());
                let card_key = key.clone();
                let field = text_input(&here.code, TextStyle::new(16.0, 400.0).lh(kept(25.6)).tracking(2.88))
                    .key(box_key.clone())
                    .single_line()
                    .grow(1.0)
                    .min_w(0.0)
                    .color(ink())
                    .placeholder("123456", muted())
                    .caret(ink(), rgba(110, 165, 255, 0.3))
                    .label(format!("The {about} code").as_str())
                    .on_input_key(need::in_a_box)
                    .on_edit(move |s: &mut State| &mut s.need_card(&card_key).code)
                    .on_input_change({
                        let card = key.clone();
                        move |s: &mut State, _| s.code_typed(&card)
                    })
                    .on_submit(move |s: &mut State, _| s.code_send(&a, &left_a));
                let off = here.busy || js::trim(&here.code.text).is_empty();
                let mut send = need::button(k("need-send-code", slug), if here.busy { "Sending" } else { "Send" }, true, false, off);
                if !off {
                    send = send.on_press(move |s: &mut State| s.code_send(&b, &left_b));
                }
                // `.need-code`: 150 wide and no more (`flex-grow: 0`).
                let boxed = row().w(150.0).child(need::boxed(box_key.clone(), field, here.code.focused));
                said = said.child(row().mt(14.0).gap(8.0).child(boxed).child(send));
                if let Some(why) = here.said.as_deref().filter(|w| !(w.starts_with("Connect ") && keyed_left.is_empty())) {
                    said = said.child(need::told(why));
                }
                open_first = Some(Some(box_key));
                let mut c = col().child(body(mark, said, None, false));
                if show_foot {
                    c = c.child(foot(&ask, slug));
                }
                frame(c, false)
            }
        }
    };
    if let Some(first) = open_first {
        OPEN.with(|open| open.borrow_mut().push((key, first)));
    }
    drawn
}

/// The first box of a key card that is opened anew, when a box is the first thing in it to take
/// the keys: not where a press reads the key, nor where GitHub's own sign-in leads.
fn fields_first_box(svc: Option<&needs::Service>, from: Option<&KeySource>, slug: &str) -> Option<String> {
    (from.is_none() && svc.is_none_or(|s| s.id != "github")).then(|| k("need-field0", slug))
}

/// GitHub's own sign-in on the card, and the way to paste a token instead (`GitHubOwnSignIn`).
fn github_own(here: &Card, ask: &Ask, slug: &str) -> El {
    let mut out = col();
    if let Some((code, _)) = &here.gh.device {
        let card = ask.card.clone();
        let copy = need::link(k("need-copy", slug), "Copy").on_press_tx(move |s: &mut State, tx| s.github_cloud_copy(&card, tx));
        out = out.child(need::code_box(code, copy));
        let card = ask.card.clone();
        out = out.child(row().mt(14.0).gap(8.0).child(need::button(k("need-open", slug), "Open GitHub", true, true, false).on_press(move |s: &mut State| s.github_cloud_open(&card))));
    } else {
        let busy = here.gh.busy;
        let mut begin = need::button(k("need-github", slug), if busy { "Starting" } else { "Sign in with GitHub" }, true, true, busy);
        if !busy {
            let a = ask.clone();
            begin = begin.on_press(move |s: &mut State| s.github_cloud_begin(&a));
        }
        let card = ask.card.clone();
        let instead = need::link(k("need-paste", slug), "Paste a token instead").ml(6.0).on_press(move |s: &mut State| {
            let c = s.need_card(&card);
            c.pasting = true;
            c.fields = Fields::default();
        });
        out = out.child(row().mt(14.0).gap(8.0).items_center().child(begin).child(instead));
    }
    if let Some(why) = here.gh.said.as_deref().filter(|w| !w.is_empty()) {
        out = out.child(need::told(why));
    }
    out
}

/// The boxes for a sign-in's or a key's variables, and Save (`KeyFields`), with the key of its
/// first box. `second`: a press above already offers the key.
fn key_fields(here: &Card, ask: &Ask, fields: &[(String, String)], plain: &str, second: bool, slug: &str) -> (El, String) {
    let f = &here.fields;
    let some = f.values.iter().any(|v| !js::trim(&v.text).is_empty());
    let busy = f.busy;
    let mut form = col().gap(8.0);
    let empty = TextEdit::new("");
    for (i, (_, label)) in fields.iter().enumerate() {
        let edit = f.values.get(i).unwrap_or(&empty);
        let placeholder = if second { "Or paste it".to_string() } else { format!("Paste the {}", if fields.len() > 1 { lower(label) } else { plain.to_string() }) };
        let box_key = k(&format!("need-field{i}"), slug);
        let (card, typed) = (ask.card.clone(), ask.card.clone());
        let count = fields.len();
        let submit = ask.clone();
        let field = need::typed(edit, box_key.clone(), &placeholder)
            .secure()
            .label(label.as_str())
            .on_edit(move |s: &mut State| s.field_of(&card, i, count))
            .on_input_change(move |s: &mut State, _| s.need_card(&typed).fields.said = None)
            .on_submit(move |s: &mut State, _| {
                let c = s.need_card(&submit.card);
                if c.fields.values.iter().any(|v| !js::trim(&v.text).is_empty()) && !c.fields.busy {
                    s.fields_save(&submit);
                }
            });
        let mut line = row().gap(8.0).child(need::boxed(box_key, field, edit.focused));
        if i + 1 == fields.len() {
            let words = if busy { "Saving" } else if f.anyway() { "Save anyway" } else { "Save" };
            let off = busy || !some;
            let mut save = need::button(k("need-save", slug), words, !second, false, off);
            if !off {
                let a = ask.clone();
                save = save.on_press(move |s: &mut State| s.fields_save(&a));
            }
            line = line.child(save);
        }
        form = form.child(line);
    }
    if let Some(why) = &f.said {
        form = form.child(need::told(why));
    }
    (form, k("need-field0", slug))
}

impl State {
    /// The box of variable `i` of a card's fields, made the first time it is typed into, each
    /// one hiding what it holds.
    fn field_of(&mut self, card: &str, i: usize, count: usize) -> &mut TextEdit {
        let fields = &mut self.need_card(card).fields;
        while fields.values.len() < count.max(i + 1) {
            fields.values.push(TextEdit::new("").secure());
        }
        &mut fields.values[i]
    }

    /// The code's box keeps no spaces and no more than twelve, and a letter typed takes back
    /// what was said (`setCode(value.replace(/\s+/g, ''))`, `maxLength={12}`).
    fn code_typed(&mut self, card: &str) {
        let c = self.need_card(card);
        let bare: String = c.code.text.chars().filter(|ch| !js::is_space(*ch)).collect();
        let bare = js::slice(&bare, 0, 12).to_string();
        if bare != c.code.text {
            c.code.replace_text(bare.clone());
            c.code.set_caret(bare.len());
        }
        c.said = None;
    }
}

/// Each frame starts the list of the cards still open afresh.
pub fn start_frame() {
    OPEN.with(|open| open.borrow_mut().clear());
}

/// A picture taken after a card was pressed or typed into says so in its fixture
/// (`cloudNeeds`): the keys just saved and the files held, each card's state by its place
/// (`cards`: `{ "<n>": { … } }`, the `n`th card of the newest answer), and what the engine said
/// of each file (`fileInfo`, by its path). The
/// same is done here, with nobody to send to.
pub fn picture(s: &mut State) {
    let Some(facts) = s.fixture.get("cloudNeeds").cloned() else { return };
    let words = |v: Option<&Value>| v.and_then(Value::as_str).map(str::to_string);
    for name in facts.get("justSaved").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
        s.cloud_needs.just_saved.push(name.to_string());
    }
    if let Some(held) = facts.get("held").and_then(Value::as_object) {
        for (item, list) in held {
            let list = list.as_array().into_iter().flatten().map(|h| Held { need: words(h.get("need")).unwrap_or_default(), words: words(h.get("words")).unwrap_or_default() }).collect();
            s.cloud_needs.held.insert(item.clone(), list);
        }
    }
    let Some(item) = s.open_item().cloned() else { return };
    let thread = thread_of(&item);
    if let Some(info) = facts.get("fileInfo").and_then(Value::as_object) {
        for (path, v) in info {
            s.cloud_needs.infos.insert(format!("{thread}|{path}"), Some(Info::of(v)));
        }
    }
    let Some(cards) = facts.get("cards").and_then(Value::as_object) else { return };
    for (place, v) in cards {
        // A card by its place among the newest answer's cards.
        let key = format!("{thread}|{}-{place}", item.wrote_at("result") as i64);
        let flag = |f: &str| truthy(v.get(f));
        let c = s.need_card(&key);
        c.busy = flag("busy");
        c.said = words(v.get("said"));
        c.replacing = flag("replacing");
        c.late = flag("late");
        c.sent_late = flag("sentLate");
        c.pasting = flag("pasting");
        if let Some(code) = words(v.get("code")) {
            c.code = TextEdit::new(code);
        }
        if let Some(values) = v.get("values").and_then(Value::as_array) {
            c.fields.values = values.iter().map(|t| TextEdit::new(t.as_str().unwrap_or("")).secure()).collect();
        }
        c.fields.said = words(v.get("fieldsSaid"));
        c.fields.busy = flag("saving");
        if flag("warned") {
            c.fields.warned = Some(c.fields.now());
        }
        c.here_busy = flag("reading");
        c.here_said = words(v.get("hereSaid"));
        c.gh.busy = flag("starting");
        c.gh.said = words(v.get("gitHubSaid"));
        if let Some(d) = v.get("device") {
            c.gh.device = Some((words(d.get("code")).unwrap_or_default(), words(d.get("url")).unwrap_or_default()));
        }
        // The box that has the keys: a code card's own, or a key card's `n`th.
        match v.get("focused").and_then(Value::as_u64) {
            Some(_) if !c.code.text.is_empty() => c.code.focused = true,
            Some(n) => {
                let n = n as usize;
                s.field_of(&key, n, n + 1).focused = true;
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use rune::{Key, Runtime};
    use serde_json::json;

    use super::*;
    use crate::View;
    use crate::feel::tests::{key, press, run, window_dressed};
    use crate::rules::writes::ANSWER;

    const SUPABASE: &str = "AGENTBOX-NEEDS: sign-in supabase | to apply the storage rules to the shop's database";
    /// An invented token, of the shape Supabase gives one.
    const TOKEN: &str = "sbp_0123456789abcdef0123";

    /// The invented inbox's first thread, open again with its newest answer `result`, on a Mac
    /// whose cloud keeps `keys`; `dress` changes the row. Whatever is sent is taken as sent.
    fn asking(name: &str, result: &str, keys: &[&str], dress: impl FnOnce(&mut Value)) -> Runtime<State> {
        // Each window's row at a moment of its own: the page keeps its last reading by the row's
        // moment, and two windows of one test would otherwise share it.
        static WINDOWS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let nth = f64::from(WINDOWS.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        let keys: Vec<Value> = keys.iter().map(|k| json!({ "name": k, "label": k })).collect();
        let mut rt = window_dressed(name, &["--thread", "w-feel0"], "/fixtures/storefront", result, |fixture| {
            fixture["snapshot"]["cloud"] = json!({ "setUp": true, "mode": "full", "running": 0, "accounts": [], "signIns": [], "keys": keys });
            let item = &mut fixture["snapshot"]["items"][0];
            item["updatedAt"] = json!(item["updatedAt"].as_f64().unwrap() + nth);
            item["status"] = json!("open");
            dress(item);
        });
        rt.state.pretend_sent = true;
        rt
    }

    fn texts(rt: &Runtime<State>) -> Vec<String> {
        rt.elements().iter().filter_map(|e| e.text.clone()).collect()
    }

    fn says(rt: &Runtime<State>, words: &str) -> bool {
        texts(rt).iter().any(|t| t == words)
    }

    /// The key of the first control whose key starts `base~`.
    fn control(rt: &Runtime<State>, base: &str) -> Option<String> {
        rt.elements().iter().filter_map(|e| e.key.clone()).find(|k| k.starts_with(&format!("{base}~")))
    }

    fn press_on(rt: &mut Runtime<State>, base: &str) {
        let key = control(rt, base).unwrap_or_else(|| panic!("{base} is not on the page: {:?}", texts(rt)));
        let (x, y) = rt.find(&key).unwrap();
        press(rt, x, y);
    }

    fn type_in(rt: &mut Runtime<State>, base: &str, words: &str) {
        press_on(rt, base);
        key(rt, Key::Text(words.into()));
    }

    fn lands(rt: &mut Runtime<State>, answer: Result<Value, String>) {
        let sent = rt.state.last_sent.clone().expect("something was sent");
        rt.update(|s, tx| s.landed(Landed { sent, answer, snapshot: None }, tx));
        rt.frame(1.0 / 60.0);
    }

    fn toast(rt: &Runtime<State>) -> Option<String> {
        rt.state.toast.as_ref().map(|t| t.line.clone())
    }

    #[test]
    fn a_line_a_cloud_run_ends_on_is_a_card_where_it_stood() {
        let rt = asking("cn-card", &format!("**I need one thing.**\n\nThe rules are written.\n\n{SUPABASE}\n\nThen I ship."), &[], |_| {});
        for words in ["Connect Supabase", "To apply the storage rules to the shop's database.", "Get an access token from Supabase's own page", KEPT, EVERY_THREAD, WAY_OUT, "Save"] {
            assert!(says(&rt, words), "{words:?} is not on the card: {:?}", texts(&rt));
        }
        assert!(!texts(&rt).iter().any(|t| t.contains("AGENTBOX-NEEDS")), "the line itself is not drawn");
        // Where the line stood: under the words before it, over the words after it.
        let y = |words: &str| rt.elements().iter().find(|e| e.text.as_deref() == Some(words)).map(|e| e.rect[1]).unwrap();
        let card = rt.find(&control(&rt, "need-cloud").unwrap()).unwrap().1;
        assert!(y("The rules are written.") < card && card < y("Then I ship."));
        // Save takes no press over an empty box.
        let mut rt = rt;
        press_on(&mut rt, "need-save");
        assert!(rt.state.tried.is_empty());
    }

    #[test]
    fn on_a_mac_with_no_cloud_the_line_is_the_block_the_web_window_falls_back_on() {
        let rt = window_dressed("cn-none", &["--thread", "w-feel0"], "/fixtures/storefront", SUPABASE, |f| f["snapshot"]["items"][0]["status"] = json!("open"));
        assert!(control(&rt, "need-cloud").is_none());
        assert!(texts(&rt).iter().any(|t| t.contains(r#"{"kind":"sign-in","service":{"id":"supabase""#)), "{:?}", texts(&rt));
    }

    #[test]
    fn a_token_pasted_is_saved_for_every_cloud_thread_and_the_card_says_connected() {
        let mut rt = asking("cn-save", SUPABASE, &[], |_| {});
        type_in(&mut rt, "need-field0", TOKEN);
        // Hidden as it is typed: no element shows it.
        assert!(!rt.elements().iter().any(|e| e.text.as_deref().is_some_and(|t| t.contains(TOKEN)) || e.label.as_deref().is_some_and(|t| t.contains(TOKEN))));
        key(&mut rt, Key::Enter);
        assert_eq!(rt.state.tried, vec![(CLOUD_KEY, json!({ "action": "save", "name": "SUPABASE_ACCESS_TOKEN", "value": TOKEN, "label": "Supabase access token" }))]);
        assert!(says(&rt, "Saving"));
        lands(&mut rt, Ok(json!({ "ok": true, "keys": [{ "name": "SUPABASE_ACCESS_TOKEN" }], "freed": ["w-feel0"], "cloudOff": false })));
        assert_eq!(toast(&rt).as_deref(), Some("Supabase connected. This thread carries on."));
        // Saved here, before the engine's lists say so. The way out goes with the last ask still
        // open, which this was (`showFoot`: a done card is never the last open).
        for words in ["Supabase connected", "Every cloud thread can use it.", "Replace"] {
            assert!(says(&rt, words), "{words:?}: {:?}", texts(&rt));
        }
        assert!(!says(&rt, WAY_OUT));
        // Replace opens the boxes again, empty.
        press_on(&mut rt, "need-replace");
        assert!(says(&rt, "Connect Supabase") && control(&rt, "need-field0").is_some());
        assert!(rt.state.cloud_needs.cards.values().all(|c| c.fields.values.iter().all(|v| v.text.is_empty())));
    }

    #[test]
    fn a_refused_save_is_said_under_the_boxes_and_a_letter_typed_takes_it_back() {
        let mut rt = asking("cn-refused", SUPABASE, &[], |_| {});
        type_in(&mut rt, "need-field0", TOKEN);
        press_on(&mut rt, "need-save");
        lands(&mut rt, Ok(json!({ "ok": false, "error": "The cloud is out of room." })));
        assert!(says(&rt, "The cloud is out of room.") && says(&rt, "Save"));
        press_on(&mut rt, "need-save");
        lands(&mut rt, Ok(json!({ "ok": false })));
        assert!(says(&rt, KEY_NOT_TAKEN));
        type_in(&mut rt, "need-field0", "4");
        assert!(!says(&rt, KEY_NOT_TAKEN));
        assert!(rt.state.toast.is_none() && rt.state.refused.is_none());
    }

    #[test]
    fn a_value_that_looks_wrong_is_warned_about_and_the_second_press_saves_it_anyway() {
        let mut rt = asking("cn-warn", "AGENTBOX-NEEDS: sign-in vercel | to deploy the preview", &[], |_| {});
        assert!(control(&rt, "need-field0").is_some() && says(&rt, "Get a token from Vercel's own page"));
        type_in(&mut rt, "need-field0", "hunter2!");
        press_on(&mut rt, "need-save");
        assert!(says(&rt, "That does not look like the Vercel token. Press Save again to keep it anyway.") && says(&rt, "Save anyway"));
        assert!(rt.state.tried.is_empty());
        press_on(&mut rt, "need-save");
        assert_eq!(rt.state.tried, vec![(CLOUD_KEY, json!({ "action": "save", "name": "VERCEL_TOKEN", "value": "hunter2!", "label": "Vercel token" }))]);
        // A Stripe key is listed with its mode.
        let mut rt = asking("cn-stripe", "AGENTBOX-NEEDS: sign-in stripe | to read the MRR", &[], |_| {});
        assert!(says(&rt, "Get a secret key from Stripe's own page"), "{:?}", texts(&rt));
        type_in(&mut rt, "need-field0", "sk_test_51Habcdef0123456789");
        press_on(&mut rt, "need-save");
        assert_eq!(rt.state.tried[0].1["label"], json!("Test-mode Stripe secret key"));
        let mut rt = asking("cn-stripe-live", "AGENTBOX-NEEDS: sign-in stripe | to read the MRR", &[], |_| {});
        type_in(&mut rt, "need-field0", "sk_live_51Habcdef0123456789");
        press_on(&mut rt, "need-save");
        assert!(says(&rt, "That is a live Stripe key: every cloud thread could make real charges with it. Press Save again to keep it anyway."));
        press_on(&mut rt, "need-save");
        assert_eq!(rt.state.tried[0].1["label"], json!("Live Stripe secret key"));
    }

    #[test]
    fn a_sign_in_of_two_variables_asks_for_both_and_saves_each_in_order() {
        let mut rt = asking("cn-aws", "AGENTBOX-NEEDS: sign-in aws | to upload the build", &[], |_| {});
        for words in ["Get an access key from AWS's own page"] {
            assert!(says(&rt, words), "{words:?}: {:?}", texts(&rt));
        }
        type_in(&mut rt, "need-field0", "AKIAINVENTED0000");
        press_on(&mut rt, "need-save");
        assert!(says(&rt, "Paste the secret access key too.") && rt.state.tried.is_empty());
        type_in(&mut rt, "need-field1", "invented/secret/0000000000000000");
        press_on(&mut rt, "need-save");
        assert_eq!(
            rt.state.tried,
            vec![
                (CLOUD_KEY, json!({ "action": "save", "name": "AWS_ACCESS_KEY_ID", "value": "AKIAINVENTED0000", "label": "Access key ID" })),
                (CLOUD_KEY, json!({ "action": "save", "name": "AWS_SECRET_ACCESS_KEY", "value": "invented/secret/0000000000000000", "label": "Secret access key" })),
            ]
        );
    }

    #[test]
    fn a_key_where_the_run_said_it_is_is_read_and_saved_in_one_press() {
        let line = "AGENTBOX-NEEDS: key POSTHOG_PERSONAL_KEY | the PostHog personal API key the daily report reads | keychain posthog-read-key";
        let mut rt = asking("cn-here", line, &[], |_| {});
        for words in ["Add the PostHog personal API key the daily report reads", "Use the one in your Keychain", "posthog-read-key", "Save"] {
            assert!(says(&rt, words), "{words:?}: {:?}", texts(&rt));
        }
        // The title is not said again under it, and the box under the press is a second way.
        assert_eq!(texts(&rt).iter().filter(|t| t.contains("daily report reads")).count(), 1);
        // Its Save is a quiet one: the press above is the main way.
        press_on(&mut rt, "need-here");
        assert_eq!(rt.state.tried, vec![(CLOUD_KEY, json!({ "action": "save", "name": "POSTHOG_PERSONAL_KEY", "value": "", "label": "PostHog personal API key the daily report reads", "from": { "place": "keychain", "from": "posthog-read-key" } }))]);
        assert!(says(&rt, "Reading"));
        lands(&mut rt, Ok(json!({ "ok": false, "error": "There is nothing at posthog-read-key in your Keychain." })));
        assert!(says(&rt, "There is nothing at posthog-read-key in your Keychain.") && says(&rt, "Use the one in your Keychain"));
        let file = asking("cn-here-file", "AGENTBOX-NEEDS: key POSTHOG_KEY | the PostHog key | file ~/app/.env POSTHOG_KEY", &[], |_| {});
        assert!(says(&file, "Use the one in .env") && says(&file, "POSTHOG_KEY in ~/app/.env"), "{:?}", texts(&file));
        let shell = asking("cn-here-shell", "AGENTBOX-NEEDS: key POSTHOG_KEY | the PostHog key | shell POSTHOG_KEY", &[], |_| {});
        assert!(says(&shell, "Use the one in your shell") && says(&shell, "$POSTHOG_KEY"));
        let plain = asking("cn-here-none", "AGENTBOX-NEEDS: key SENTRY_AUTH_TOKEN | a Sentry auth token", &[], |_| {});
        assert!(control(&plain, "need-here").is_none() && says(&plain, "Add the Sentry auth token"));
    }

    #[test]
    fn githubs_own_sign_in_saves_the_token_gh_made() {
        let mut rt = asking("cn-github", "AGENTBOX-NEEDS: sign-in github | to read the failure log", &[], |_| {});
        assert!(says(&rt, "Sign in with GitHub") && says(&rt, "Paste a token instead") && control(&rt, "need-field0").is_none());
        // GitHub's own page is not offered beside its own sign-in.
        assert!(!texts(&rt).iter().any(|t| t.starts_with("Get a token")));
        press_on(&mut rt, "need-github");
        assert_eq!(rt.state.tried, vec![(GITHUB_CLOUD_SIGN_IN, json!({ "action": "start" }))]);
        assert!(says(&rt, "Starting"));
        lands(&mut rt, Ok(json!({ "ok": true, "code": "A1B2-C3D4", "url": "https://github.com/login/device" })));
        for words in ["Enter this code", "A1B2-C3D4", "Open GitHub", "Copy"] {
            assert!(says(&rt, words), "{words:?}: {:?}", texts(&rt));
        }
        assert_eq!(rt.state.tried[1], (GITHUB_CLOUD_SIGN_IN, json!({ "action": "token" })));
        let clip = rune::clipboard::memory();
        rt.use_clipboard(&clip);
        press_on(&mut rt, "need-copy");
        assert_eq!(clip.text().map(|t| t.to_string()).as_deref(), Some("A1B2-C3D4"));
        assert!(rt.state.toast.is_none(), "Copy says nothing (`writeText`)");
        lands(&mut rt, Ok(json!({ "ok": true, "token": "gho_invented0000" })));
        assert_eq!(rt.state.tried[2], (CLOUD_KEY, json!({ "action": "save", "name": "GH_TOKEN", "value": "gho_invented0000", "label": "GitHub token" })));
        lands(&mut rt, Ok(json!({ "ok": false })));
        assert!(says(&rt, KEY_NOT_TAKEN) && says(&rt, "Sign in with GitHub"), "the code goes and the reason is said");
        // Did not finish, or did not start.
        press_on(&mut rt, "need-github");
        lands(&mut rt, Ok(json!({ "ok": true, "code": "A1B2-C3D4", "url": "https://github.com/login/device" })));
        lands(&mut rt, Ok(json!({ "ok": false })));
        assert!(says(&rt, GITHUB_DID_NOT_FINISH));
        press_on(&mut rt, "need-github");
        lands(&mut rt, Ok(json!({ "ok": false, "error": "gh is not installed." })));
        assert!(says(&rt, "gh is not installed."));
        // A token instead: the facts with GitHub's page, and the box.
        press_on(&mut rt, "need-paste");
        assert!(says(&rt, "Get a token from GitHub's own page") && control(&rt, "need-field0").is_some() && !says(&rt, "Sign in with GitHub"));
    }

    #[test]
    fn a_file_is_sent_from_this_computer_or_one_picked_in_this_windows_own_panel() {
        let line = "AGENTBOX-NEEDS: file ~/Downloads/signed-lease.pdf | to attach it to the filing";
        let mut rt = asking("cn-file", line, &[], |_| {});
        // Its size is asked as the card comes up.
        assert_eq!(rt.state.tried, vec![(CLOUD_FILE_INFO, json!({ "path": "~/Downloads/signed-lease.pdf" }))]);
        assert!(says(&rt, "In Downloads on this computer. To attach it to the filing."));
        lands(&mut rt, Ok(json!({ "ok": true, "size": 3.0 * 1024.0 * 1024.0 })));
        for words in ["Send signed-lease.pdf", "In Downloads on this computer, 3 MB. To attach it to the filing.", FILE_FACT, "Send from this computer", "Choose another file", WAY_OUT] {
            assert!(says(&rt, words), "{words:?}: {:?}", texts(&rt));
        }
        press_on(&mut rt, "need-send-file");
        assert_eq!(rt.state.tried[1], (CLOUD_SEND_FILE, json!({ "product": "storefront", "id": "w-feel0", "path": "~/Downloads/signed-lease.pdf", "hold": false })));
        assert!(says(&rt, "Sending"));
        lands(&mut rt, Ok(json!({ "ok": true, "path": "/Users/you/Downloads/signed-lease.pdf" })));
        assert_eq!(toast(&rt).as_deref(), Some("Sent signed-lease.pdf. This thread carries on in the cloud."));
        // Another one, in this window's own panel; closing it sends nothing.
        press_on(&mut rt, "need-choose");
        let (kind, options) = rt.file_dialog().expect("the panel is up");
        assert_eq!((kind, options.title.as_str()), (rune::file_dialog::FileDialogKind::Open, PANEL_TITLE));
        rt.resolve_file_dialog(None).unwrap();
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.tried.len(), 2);
        assert!(says(&rt, "Send from this computer"));
        press_on(&mut rt, "need-choose");
        rt.resolve_file_dialog(Some("/Users/you/Desktop/lease-v2.pdf".into())).unwrap();
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.tried[2], (CLOUD_SEND_FILE, json!({ "product": "storefront", "id": "w-feel0", "path": "/Users/you/Desktop/lease-v2.pdf", "hold": false })));
        lands(&mut rt, Ok(json!({ "ok": false })));
        assert!(says(&rt, FILE_NOT_SENT));
        // A file that is not there can only be chosen.
        let mut missing = asking("cn-file-missing", line, &[], |_| {});
        lands(&mut missing, Ok(json!({ "ok": false, "error": "~/Downloads/signed-lease.pdf is not on this computer." })));
        assert!(says(&missing, "~/Downloads/signed-lease.pdf is not on this computer. To attach it to the filing.") && says(&missing, "Choose the file") && !says(&missing, "Send from this computer"));
    }

    #[test]
    fn a_file_sent_while_a_sign_in_is_open_waits_for_it_and_goes_with_its_save() {
        let result = "AGENTBOX-NEEDS: sign-in stripe | to read the MRR\nAGENTBOX-NEEDS: file ~/Movies/launch-film.mov | to cut it";
        let mut rt = asking("cn-held", result, &[], |_| {});
        // The way out is under the last ask still open, the facts on the first.
        assert_eq!(texts(&rt).iter().filter(|t| *t == WAY_OUT).count(), 1);
        rt.state.tried.clear();
        press_on(&mut rt, "need-send-file");
        assert_eq!(rt.state.tried, vec![(CLOUD_SEND_FILE, json!({ "product": "storefront", "id": "w-feel0", "path": "~/Movies/launch-film.mov", "hold": true }))]);
        lands(&mut rt, Ok(json!({ "ok": true, "path": "/Users/you/Movies/launch-film.mov", "held": true, "words": "I sent ~/Movies/launch-film.mov. It is at that same path on your machine now." })));
        assert_eq!(toast(&rt).as_deref(), Some("launch-film.mov is ready. It goes with Stripe."));
        assert!(says(&rt, "launch-film.mov is ready") && says(&rt, "Ready. It goes with Stripe."));
        assert_eq!(rt.state.cloud_needs.held["w-feel0"].len(), 1);
        // The keys go on to the next ask still open (`focusNextAsk`): Stripe's box.
        run(&mut rt, 0.3);
        let field = control(&rt, "need-field0").unwrap();
        assert!(rt.elements().iter().any(|e| e.key.as_ref() == Some(&field) && e.focused), "Stripe's box has the keys");
        // The save of the last one carries what was held, in the web window's words.
        type_in(&mut rt, "need-field0", "sk_test_51Habcdef0123456789");
        press_on(&mut rt, "need-save");
        let reply = json!({ "product": "storefront", "id": "w-feel0", "answer": "I connected Stripe. I connected Stripe. I sent ~/Movies/launch-film.mov. It is at that same path on your machine now. Carry on." });
        assert_eq!(rt.state.tried[1], (CLOUD_KEY, json!({ "action": "save", "name": "STRIPE_SECRET_KEY", "value": "sk_test_51Habcdef0123456789", "label": "Test-mode Stripe secret key", "reply": reply })));
        lands(&mut rt, Ok(json!({ "ok": true, "keys": [{ "name": "STRIPE_SECRET_KEY" }], "freed": [] })));
        assert!(!rt.state.cloud_needs.held.contains_key("w-feel0"));
        assert_eq!(toast(&rt).as_deref(), Some("Stripe connected."));
    }

    #[test]
    fn a_code_waits_for_the_sign_ins_and_is_checked_before_it_goes() {
        let result = "AGENTBOX-NEEDS: sign-in stripe | to read the MRR\nAGENTBOX-NEEDS: code npm | the one-time password for npm publish";
        let mut rt = asking("cn-code", result, &[], |_| {});
        for words in ["Enter the npm code", "The one-time password for npm publish. Goes to this thread only.", "Enter it once Stripe is in: these codes run out in a minute."] {
            assert!(says(&rt, words), "{words:?}: {:?}", texts(&rt));
        }
        type_in(&mut rt, "need-otp", "48 21 39");
        assert_eq!(rt.state.cloud_needs.cards.values().find(|c| !c.code.text.is_empty()).map(|c| c.code.text.clone()).as_deref(), Some("482139"), "no spaces");
        key(&mut rt, Key::Enter);
        assert!(says(&rt, "Connect Stripe first, then the code: it runs out in a minute.") && rt.state.tried.is_empty());
        // Once Stripe is in: a code that is not six digits is asked about once.
        let mut rt = asking("cn-code-alone", "AGENTBOX-NEEDS: code npm | the one-time password for npm publish", &[], |_| {});
        assert!(!says(&rt, "Enter it once Stripe is in: these codes run out in a minute."));
        type_in(&mut rt, "need-otp", "48213");
        press_on(&mut rt, "need-send-code");
        assert!(says(&rt, "That is 5 digits; these codes are 6. Press Send again to send it anyway.") && rt.state.tried.is_empty());
        press_on(&mut rt, "need-send-code");
        assert_eq!(rt.state.tried, vec![(CLOUD_SEND_CODE, json!({ "product": "storefront", "id": "w-feel0", "about": "npm", "code": "48213" }))]);
        lands(&mut rt, Ok(json!({ "ok": false, "error": "That does not look like a code." })));
        assert!(says(&rt, "That does not look like a code."));
        press_on(&mut rt, "need-send-code");
        lands(&mut rt, Ok(json!({ "ok": true })));
        assert_eq!(toast(&rt).as_deref(), Some("Code sent. This thread carries on."));
        // Twelve at most.
        type_in(&mut rt, "need-otp", "1234567890123456");
        let codes: Vec<String> = rt.state.cloud_needs.cards.values().map(|c| c.code.text.clone()).collect();
        assert!(codes.iter().any(|c| c == "482131234567"), "{codes:?}");
    }

    /// A thread whose newest answer was answered in words since it asked.
    fn answered(name: &str, result: &str, keys: &[&str]) -> Runtime<State> {
        asking(name, result, keys, |item| {
            let at = item["updatedAt"].as_f64().unwrap();
            item["answer"] = json!("Do it from my computer.");
            item["wrote"]["answer"] = json!({ "ts": at + 60_000.0, "source": "founder" });
        })
    }

    #[test]
    fn an_ask_answered_in_words_says_so_and_can_still_be_given() {
        let result = "AGENTBOX-NEEDS: key ELEVENLABS_API_KEY | ElevenLabs API key, from elevenlabs.io/app/settings/api-keys\nAGENTBOX-NEEDS: sign-in sentry | to read the errors\nAGENTBOX-NEEDS: file ~/Movies/launch.mov | to cut it\nAGENTBOX-NEEDS: code npm | to publish";
        let mut rt = answered("cn-answered", result, &[]);
        for words in ["Add the ElevenLabs API key", "Connect Sentry", "Send launch.mov", "Enter the npm code", "Add it now", "Connect it now", "Send it now"] {
            assert!(says(&rt, words), "{words:?}: {:?}", texts(&rt));
        }
        assert_eq!(texts(&rt).iter().filter(|t| *t == "Answered below.").count(), 4);
        assert!(control(&rt, "need-field0").is_none() && !says(&rt, WAY_OUT));
        // Added late: no way out, and the thread is told it came after all, its label as it is.
        press_on(&mut rt, "need-now");
        let field = control(&rt, "need-field0").expect("its box");
        assert!(rt.elements().iter().any(|e| e.key.as_ref() == Some(&field) && e.label.as_deref() == Some("ElevenLabs API key")) && !says(&rt, WAY_OUT));
        type_in(&mut rt, "need-field0", "invented_elevenlabs_0000");
        press_on(&mut rt, "need-save");
        let reply = json!({ "product": "storefront", "id": "w-feel0", "answer": "I saved the ElevenLabs API key after all, so you can use it now." });
        assert_eq!(rt.state.tried.last().unwrap(), &(CLOUD_KEY, json!({ "action": "save", "name": "ELEVENLABS_API_KEY", "value": "invented_elevenlabs_0000", "label": "ElevenLabs API key", "reply": reply })));
        lands(&mut rt, Ok(json!({ "ok": true, "keys": [], "freed": [] })));
        assert_eq!(toast(&rt).as_deref(), Some("ElevenLabs API key saved."));
        assert!(says(&rt, "ElevenLabs API key saved") && !says(&rt, "Replace"), "saved, and nothing to replace off the live card");
        // A file sent late: no hold, and the card says it was sent.
        rt.state.tried.clear();
        let files: Vec<String> = rt.elements().iter().filter_map(|e| e.key.clone()).filter(|k| k.starts_with("need-now~")).collect();
        let (x, y) = rt.find(files.iter().find(|k| rt.elements().iter().any(|e| e.key.as_ref() == Some(k) && e.label.as_deref() == Some("Send it now"))).unwrap()).unwrap();
        press(&mut rt, x, y);
        press_on(&mut rt, "need-send-file");
        assert_eq!(rt.state.tried.last().unwrap(), &(CLOUD_SEND_FILE, json!({ "product": "storefront", "id": "w-feel0", "path": "~/Movies/launch.mov", "hold": false })));
        lands(&mut rt, Ok(json!({ "ok": true, "path": "/Users/you/Movies/launch.mov" })));
        assert_eq!(toast(&rt).as_deref(), Some("Sent launch.mov. This thread has it now."));
        assert!(says(&rt, "launch.mov sent") && says(&rt, "This thread has it now."));
        // A code has run out: nothing to press.
        assert!(control(&rt, "need-otp").is_none());
    }

    #[test]
    fn the_way_out_sends_do_it_from_my_computer_as_any_reply_and_keeps_the_box() {
        let mut rt = asking("cn-way-out", SUPABASE, &[], |_| {});
        rt.update(|s, _| s.reply = TextEdit::new("half a thought"));
        press_on(&mut rt, "need-foot");
        let held = rt.state.held.as_ref().expect("held its three seconds");
        assert_eq!((held.channel, held.payload["answer"].clone()), (ANSWER, json!(FROM_MY_COMPUTER)));
        assert_eq!(rt.state.reply.text, "half a thought");
    }

    #[test]
    fn a_key_pasted_into_the_reply_box_is_saved_and_never_sent_as_words() {
        let mut rt = asking("cn-reply-key", SUPABASE, &[], |_| {});
        key(&mut rt, Key::Text("r".into()));
        key(&mut rt, Key::Text(format!("here you go {TOKEN}")));
        rt.update(|s, tx| s.send_reply(tx));
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.tried, vec![(CLOUD_KEY, json!({ "action": "save", "name": "SUPABASE_ACCESS_TOKEN", "value": TOKEN, "label": "Supabase access token" }))]);
        // Nothing of it is held, drawn, kept on its way, or left in the box or its draft.
        assert!(rt.state.held.is_none() && rt.state.answering.in_flight.is_empty() && rt.state.answering.sending.is_empty());
        assert!(rt.state.reply.text.is_empty() && !rt.state.replying && !rt.state.drafts.contains_key("storefront/w-feel0"));
        assert!(!texts(&rt).iter().any(|t| t.contains(TOKEN)));
        lands(&mut rt, Ok(json!({ "ok": true, "keys": [{ "name": "SUPABASE_ACCESS_TOKEN" }], "freed": ["w-feel0"] })));
        assert_eq!(toast(&rt).as_deref(), Some("Supabase connected. This thread carries on."));
        // Inside a longer sentence, the words go as the reply with the key named instead.
        let mut rt = asking("cn-reply-sentence", SUPABASE, &[], |_| {});
        key(&mut rt, Key::Text("r".into()));
        key(&mut rt, Key::Text(format!("no time today, skip the deploy step and just write the github action yaml for {TOKEN}")));
        rt.update(|s, tx| s.send_reply(tx));
        let reply = json!({ "product": "storefront", "id": "w-feel0", "answer": "no time today, skip the deploy step and just write the github action yaml for [Supabase access token, saved]" });
        assert_eq!(rt.state.tried, vec![(CLOUD_KEY, json!({ "action": "save", "name": "SUPABASE_ACCESS_TOKEN", "value": TOKEN, "label": "Supabase access token", "reply": reply }))]);
        // A refusal is said as a line.
        lands(&mut rt, Ok(json!({ "ok": false })));
        assert_eq!(toast(&rt).as_deref(), Some(KEY_NOT_TAKEN));
    }

    #[test]
    fn a_key_with_files_in_the_box_is_caught_once_the_files_are_saved() {
        let mut rt = asking("cn-reply-files", SUPABASE, &[], |_| {});
        // The words with the link to the file the engine saved, as the web window's text is.
        let saved = json!([{ "rel": "attachments/mx1-ab-notes.pdf", "name": "notes.pdf", "image": false }]);
        let said = crate::rules::reply_chips::Said { level: None, mode: None };
        let landed = Landed { sent: Sent::Carry { thread: "storefront/w-feel0".to_string(), words: TOKEN.to_string(), pick: None, said, files: Vec::new() }, answer: Ok(saved), snapshot: None };
        rt.update(|s, tx| s.carried(landed, tx));
        assert!(rt.state.held.is_none(), "never held as a reply");
        assert_eq!(rt.state.tried, vec![(CLOUD_KEY, json!({ "action": "save", "name": "SUPABASE_ACCESS_TOKEN", "value": TOKEN, "label": "Supabase access token" }))]);
    }

    #[test]
    fn a_key_that_looks_wrong_in_the_reply_box_is_neither_sent_nor_saved_nor_kept() {
        let mut rt = asking("cn-reply-wrong", SUPABASE, &[], |_| {});
        key(&mut rt, Key::Text("r".into()));
        key(&mut rt, Key::Text("ghp_abcdefghijklmnop123".into()));
        rt.update(|s, tx| s.send_reply(tx));
        rt.frame(1.0 / 60.0);
        assert!(rt.state.tried.is_empty() && rt.state.held.is_none());
        assert_eq!(toast(&rt).as_deref(), Some("That looks like a GitHub token, not the Supabase access token. Nothing was sent or saved; paste it in the card on the thread to keep it anyway."));
        // The words stay in the box, and are not its draft any more: folded, they go.
        assert_eq!(rt.state.reply.text, "ghp_abcdefghijklmnop123");
        assert!(unkept(&rt.state, "storefront/w-feel0", "ghp_abcdefghijklmnop123"));
        rt.update(|s, _| s.fold_reply());
        assert!(rt.state.reply.text.is_empty());
    }

    #[test]
    fn a_code_typed_into_the_reply_box_goes_as_the_code() {
        let mut rt = asking("cn-reply-code", "AGENTBOX-NEEDS: code npm | to publish", &[], |_| {});
        rt.state.tried.clear();
        key(&mut rt, Key::Text("r".into()));
        key(&mut rt, Key::Text("482139".into()));
        rt.update(|s, tx| s.send_reply(tx));
        assert_eq!(rt.state.tried, vec![(CLOUD_SEND_CODE, json!({ "product": "storefront", "id": "w-feel0", "about": "npm", "code": "482139" }))]);
        assert!(rt.state.held.is_none() && rt.state.reply.text.is_empty() && !rt.state.replying);
        // While a sign-in is still open it is held back, never sent as words.
        let mut rt = asking("cn-reply-code-early", "AGENTBOX-NEEDS: sign-in stripe | x\nAGENTBOX-NEEDS: code npm | to publish", &[], |_| {});
        key(&mut rt, Key::Text("r".into()));
        key(&mut rt, Key::Text("482139".into()));
        rt.update(|s, tx| s.send_reply(tx));
        assert!(rt.state.tried.is_empty() && rt.state.held.is_none());
        assert_eq!(toast(&rt).as_deref(), Some("Connect Stripe first, then the code: it runs out in a minute."));
        // Words are words.
        let mut rt = asking("cn-reply-words", "AGENTBOX-NEEDS: code npm | to publish", &[], |_| {});
        key(&mut rt, Key::Text("r".into()));
        key(&mut rt, Key::Text("do it from my computer".into()));
        rt.update(|s, tx| s.send_reply(tx));
        assert!(rt.state.held.is_some());
        // A key for a sign-in of two variables goes as words, as the web window sends it.
        let mut rt = asking("cn-reply-aws", "AGENTBOX-NEEDS: sign-in aws | x", &["AWS_ACCESS_KEY_ID"], |_| {});
        key(&mut rt, Key::Text("r".into()));
        key(&mut rt, Key::Text("invented-secret-0000000000".into()));
        rt.update(|s, tx| s.send_reply(tx));
        assert!(rt.state.held.is_some() && !rt.state.tried.iter().any(|(c, _)| *c == CLOUD_KEY));
    }

    #[test]
    fn escape_lets_go_of_a_box_and_leaving_the_conversation_lets_go_of_the_cards() {
        let mut rt = asking("cn-leave", SUPABASE, &[], |_| {});
        type_in(&mut rt, "need-field0", "sbp_");
        key(&mut rt, Key::Escape);
        assert!(matches!(rt.state.view, View::Thread { .. }));
        key(&mut rt, Key::Escape);
        assert!(!matches!(rt.state.view, View::Thread { .. }));
        assert!(rt.state.cloud_needs.cards.is_empty());
        run(&mut rt, 0.1);
    }
}
