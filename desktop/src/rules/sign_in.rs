//! A sign-in an agent puts in front of the person while it waits (`shared/sign-in-ask.mts`).
//! An agent hands a service's own sign-in link to its `sign_in` tool; the thread carries it in
//! `signIn`, the card under the conversation draws it (`SignInCard.tsx`, `need.rs` here), and
//! what the person did goes back in `signInAnswer`, to an agent that is still running.
//!
//! This is the reading half: which ask is open, what the card says of it, and what each of its
//! controls writes.

use serde_json::{Value, json};

use super::js;
use super::writes::Write;
use crate::data::{Item, truthy};

/// What the person did with the link (`zero:sign-in-answer`), and whether a cloud sign-in is
/// kept for every cloud thread (`zero:sign-in-keep`).
pub const ANSWER: &str = "zero:sign-in-answer";
pub const KEEP: &str = "zero:sign-in-keep";
/// What the card says when the engine did not take a write and gave no reason of its own.
pub const DID_NOT_REACH: &str = "That did not reach the agent. Try again.";

/// Each service's own sign-in addresses. The agent names the service, so the card trusts the
/// name, and draws its logo, only for one of these (`SERVICE_HOSTS`).
const SERVICE_HOSTS: [(&str, &[&str]); 27] = [
    ("github", &["github.com"]),
    ("vercel", &["vercel.com"]),
    ("supabase", &["supabase.com"]),
    ("stripe", &["stripe.com"]),
    ("sentry", &["sentry.io"]),
    ("expo", &["expo.dev"]),
    ("aws", &["amazon.com", "amazonaws.com", "awsapps.com"]),
    ("npm", &["npmjs.com"]),
    ("netlify", &["netlify.com"]),
    ("cloudflare", &["cloudflare.com"]),
    ("fly", &["fly.io"]),
    ("linear", &["linear.app"]),
    ("notion", &["notion.so", "notion.com"]),
    ("slack", &["slack.com"]),
    ("figma", &["figma.com"]),
    ("airtable", &["airtable.com"]),
    ("asana", &["asana.com"]),
    ("atlassian", &["atlassian.com", "atlassian.net"]),
    ("clickup", &["clickup.com"]),
    ("hubspot", &["hubspot.com"]),
    ("intercom", &["intercom.com"]),
    ("monday", &["monday.com"]),
    ("salesforce", &["salesforce.com"]),
    ("google", &["google.com"]),
    ("gmail", &["google.com"]),
    ("google-drive", &["google.com"]),
    ("google-calendar", &["google.com"]),
];

/// The ask a thread carries, as the card reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct Ask {
    pub id: String,
    pub service: String,
    pub url: String,
    /// A code the service's page asks the person to type in (`gh auth login`'s).
    pub code: Option<String>,
    /// Why the agent needs it, in its own words.
    pub why: Option<String>,
    /// The login waits for a code the service's page shows the person (`supabase login`).
    pub asks_for_code: bool,
    /// Whether a cloud sign-in is saved for every cloud thread: on unless the person unticked it.
    pub keep: bool,
}

/// The sign-in this thread still waits on (`openSignIn`): its ask, unless the ask is done, the
/// thread is, or the answer on the thread is to this very ask.
pub fn open(item: &Item) -> Option<Ask> {
    let ask = item.object("signIn")?;
    let id = ask.get("id").and_then(Value::as_str).filter(|id| !id.is_empty())?;
    if truthy(ask.get("done")) || item.status == "done" {
        return None;
    }
    if item.object("signInAnswer").and_then(|a| a.get("id")).and_then(Value::as_str) == Some(id) {
        return None;
    }
    let words = |field: &str| ask.get(field).and_then(Value::as_str).filter(|w| !w.is_empty()).map(str::to_string);
    Some(Ask {
        id: id.to_string(),
        service: words("service").unwrap_or_default(),
        url: words("url").unwrap_or_default(),
        code: words("code"),
        why: words("why"),
        asks_for_code: ask.get("asksForCode") == Some(&Value::Bool(true)),
        keep: ask.get("keep") != Some(&Value::Bool(false)),
    })
}

/// A service's name as its id: lower case, every run of anything else a dash (`serviceId`).
pub fn service_id(service: &str) -> String {
    let mut id = String::new();
    for c in service.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            id.push(c);
        } else if !id.ends_with('-') {
            id.push('-');
        }
    }
    id.trim_matches('-').to_string()
}

/// The host a browser opens for `url`, or nothing (`linkHost`).
pub fn link_host(url: &str) -> String {
    url::Url::parse(url).ok().and_then(|u| u.host_str().map(str::to_lowercase)).unwrap_or_default()
}

/// Whether `url` is one of `service`'s own addresses (`linkIsTheService`).
pub fn link_is_the_service(service: &str, url: &str) -> bool {
    let host = link_host(url);
    let id = service_id(service);
    !host.is_empty() && SERVICE_HOSTS.iter().filter(|(name, _)| *name == id).flat_map(|(_, hosts)| hosts.iter()).any(|h| host == *h || host.ends_with(&format!(".{h}")))
}

/// Whether the sign-in sends the browser back to this machine (`http://localhost:<port>/…`),
/// read off the link's `redirect_uri` (`comesBackTo`). A run on another machine never sees
/// that page load, so the card then offers to take the address the browser ended on.
pub fn comes_back_here(url: &str) -> bool {
    let Ok(link) = url::Url::parse(url) else { return false };
    let Some(back) = link.query_pairs().find(|(name, _)| name == "redirect_uri").and_then(|(_, to)| url::Url::parse(&to).ok()) else {
        return false;
    };
    back.scheme() == "http" && matches!(back.host_str(), Some("localhost" | "127.0.0.1")) && back.port().is_some_and(|port| port > 0)
}

/// The agent's reason as a sentence: its first letter raised, and a full stop unless it ends
/// one already (`sentence` in `SignInCard.tsx`).
pub fn sentence(why: &str) -> String {
    let stop = if why.ends_with(['.', '!', '?']) { "" } else { "." };
    format!("{}{stop}", js::upper_first(why))
}

/// What the card says of an ask: the address it opens, whose mark it wears, and what its box
/// for a code is called.
#[derive(Clone, Debug, PartialEq)]
pub struct Card {
    /// "Sign in at dash.cloudflare.com".
    pub title: String,
    /// "Open dash.cloudflare.com".
    pub open: String,
    /// The service's id when the link is the service's own, for its logo; the key otherwise.
    pub logo: Option<String>,
    pub why: Option<String>,
    /// "Paste the code Supabase shows", for a login that waits for one.
    pub code_placeholder: String,
    /// The run is on another machine and the link comes back to localhost.
    pub offers_the_address: bool,
}

pub fn card(ask: &Ask, cloud: bool) -> Card {
    let host = link_host(&ask.url);
    let theirs = link_is_the_service(&ask.service, &ask.url);
    Card {
        title: format!("Sign in at {host}"),
        open: format!("Open {host}"),
        logo: theirs.then(|| service_id(&ask.service)),
        why: ask.why.as_deref().map(sentence),
        code_placeholder: format!("Paste the code {} shows", if theirs { ask.service.as_str() } else { host.as_str() }),
        offers_the_address: cloud && comes_back_here(&ask.url),
    }
}

/// What the person said of the link (`SignInSaid`).
#[derive(Clone, Debug, PartialEq)]
pub enum Said {
    SignedIn,
    Skipped,
    /// The address the browser ended on, for a login that waits on another machine.
    Landed(String),
    /// The code the service's page showed.
    Code(String),
}

/// `api.signInAnswer({ product, id, ask, ...said })`.
pub fn answer(item: &Item, ask: &str, said: &Said) -> Write {
    let mut payload = json!({ "product": item.product, "id": item.id, "ask": ask });
    match said {
        Said::SignedIn => payload["said"] = json!("signed-in"),
        Said::Skipped => payload["said"] = json!("skipped"),
        Said::Landed(url) => {
            payload["said"] = json!("landed");
            payload["url"] = json!(url);
        }
        Said::Code(code) => {
            payload["said"] = json!("code");
            payload["code"] = json!(code);
        }
    }
    (ANSWER, payload)
}

/// `api.signInKeep({ product, id, ask, keep })`.
pub fn keep(item: &Item, ask: &str, keep: bool) -> Write {
    (KEEP, json!({ "product": item.product, "id": item.id, "ask": ask, "keep": keep }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thread(fields: Value) -> Item {
        let mut item = json!({ "id": "w-1", "product": "shop", "status": "claimed" });
        for (k, v) in fields.as_object().unwrap() {
            item[k] = v.clone();
        }
        serde_json::from_value(item).unwrap()
    }

    fn asking(ask: Value) -> Item {
        thread(json!({ "signIn": ask }))
    }

    #[test]
    fn an_ask_is_open_until_it_is_answered_done_or_its_thread_is() {
        let ask = json!({ "id": "s-abc123", "service": "Cloudflare", "url": "https://dash.cloudflare.com/x", "at": 1 });
        assert_eq!(open(&asking(ask.clone())).map(|a| a.id).as_deref(), Some("s-abc123"));
        // Answered: the answer names this ask.
        assert!(open(&thread(json!({ "signIn": ask, "signInAnswer": { "id": "s-abc123", "said": "signed-in" } }))).is_none());
        // An answer to an older ask leaves the new one open.
        assert!(open(&thread(json!({ "signIn": ask, "signInAnswer": { "id": "s-older1", "said": "skipped" } }))).is_some());
        let mut done = ask.clone();
        done["done"] = json!(true);
        assert!(open(&asking(done)).is_none());
        assert!(open(&thread(json!({ "signIn": ask, "status": "done" }))).is_none());
        assert!(open(&thread(json!({}))).is_none());
        assert!(open(&asking(json!({ "service": "x", "url": "https://x.dev" }))).is_none());
    }

    #[test]
    fn an_ask_carries_its_code_its_reason_and_whether_it_is_kept() {
        let plain = open(&asking(json!({ "id": "s-abc123", "service": "GitHub", "url": "https://github.com/login/device" }))).unwrap();
        assert_eq!((plain.code, plain.why, plain.asks_for_code, plain.keep), (None, None, false, true));
        let full = open(&asking(json!({ "id": "s-abc123", "service": "GitHub", "url": "https://github.com/login/device", "code": "A1B2-C3D4", "why": "to push", "asksForCode": true, "keep": false }))).unwrap();
        assert_eq!((full.code.as_deref(), full.why.as_deref(), full.asks_for_code, full.keep), (Some("A1B2-C3D4"), Some("to push"), true, false));
    }

    #[test]
    fn a_services_name_is_its_id() {
        assert_eq!(service_id("GitHub"), "github");
        assert_eq!(service_id("Google Drive"), "google-drive");
        assert_eq!(service_id("  Fly.io!"), "fly-io");
        assert_eq!(service_id(""), "");
    }

    #[test]
    fn the_card_trusts_a_name_only_at_the_services_own_address() {
        assert_eq!(link_host("https://Dash.Cloudflare.com/oauth2/auth?x=1"), "dash.cloudflare.com");
        assert_eq!(link_host("not a link"), "");
        assert!(link_is_the_service("Cloudflare", "https://dash.cloudflare.com/oauth2/auth"));
        assert!(link_is_the_service("GitHub", "https://github.com/login/device"));
        assert!(link_is_the_service("AWS", "https://eu-west-1.signin.amazonaws.com/x"));
        // Named GitHub, pointing somewhere else: no logo, and the address is what is named.
        assert!(!link_is_the_service("GitHub", "https://github.com.evil.dev/login"));
        assert!(!link_is_the_service("GitHub", "https://notgithub.com/login"));
        assert!(!link_is_the_service("Some tool", "https://example.com"));
    }

    #[test]
    fn a_link_that_comes_back_to_localhost_is_known() {
        assert!(comes_back_here("https://vercel.com/oauth?redirect_uri=http%3A%2F%2Flocalhost%3A8976%2Fcallback"));
        assert!(comes_back_here("https://vercel.com/oauth?a=1&redirect_uri=http://127.0.0.1:3000/"));
        // No port is port 80 to a browser and nothing to the tool that listens.
        assert!(!comes_back_here("https://vercel.com/oauth?redirect_uri=http%3A%2F%2Flocalhost%2Fcallback"));
        assert!(!comes_back_here("https://vercel.com/oauth?redirect_uri=https%3A%2F%2Flocalhost%3A8976%2Fcallback"));
        assert!(!comes_back_here("https://vercel.com/oauth?redirect_uri=http%3A%2F%2Fexample.com%3A8976%2F"));
        assert!(!comes_back_here("https://vercel.com/oauth"));
        assert!(!comes_back_here("nothing"));
    }

    #[test]
    fn the_reason_reads_as_a_sentence() {
        assert_eq!(sentence("to point the shop's domain at the new server"), "To point the shop's domain at the new server.");
        assert_eq!(sentence("Already one."), "Already one.");
        assert_eq!(sentence("really?"), "Really?");
    }

    #[test]
    fn the_card_names_the_address_and_wears_the_services_mark_only_when_it_is_theirs() {
        let ask = |service: &str, url: &str| open(&asking(json!({ "id": "s-abc123", "service": service, "url": url, "why": "to move the domain" }))).unwrap();
        let cf = card(&ask("Cloudflare", "https://dash.cloudflare.com/oauth2/auth"), false);
        assert_eq!((cf.title.as_str(), cf.open.as_str(), cf.logo.as_deref(), cf.why.as_deref()), ("Sign in at dash.cloudflare.com", "Open dash.cloudflare.com", Some("cloudflare"), Some("To move the domain.")));
        assert_eq!(cf.code_placeholder, "Paste the code Cloudflare shows");
        let odd = card(&ask("GitHub", "https://login.example.dev/start"), false);
        assert_eq!((odd.title.as_str(), odd.logo, odd.code_placeholder.as_str()), ("Sign in at login.example.dev", None, "Paste the code login.example.dev shows"));
        // The address is offered only to a run on another machine.
        let back = "https://vercel.com/oauth?redirect_uri=http%3A%2F%2Flocalhost%3A8976%2Fcallback";
        assert!(card(&ask("Vercel", back), true).offers_the_address);
        assert!(!card(&ask("Vercel", back), false).offers_the_address);
        assert!(!card(&ask("Vercel", "https://vercel.com/oauth"), true).offers_the_address);
    }

    #[test]
    fn each_control_writes_what_the_web_window_writes() {
        let item = asking(json!({ "id": "s-abc123", "service": "Supabase", "url": "https://supabase.com/dashboard/cli/login" }));
        assert_eq!(answer(&item, "s-abc123", &Said::SignedIn), (ANSWER, json!({ "product": "shop", "id": "w-1", "ask": "s-abc123", "said": "signed-in" })));
        assert_eq!(answer(&item, "s-abc123", &Said::Skipped), (ANSWER, json!({ "product": "shop", "id": "w-1", "ask": "s-abc123", "said": "skipped" })));
        assert_eq!(answer(&item, "s-abc123", &Said::Code("12ab".into())), (ANSWER, json!({ "product": "shop", "id": "w-1", "ask": "s-abc123", "said": "code", "code": "12ab" })));
        assert_eq!(
            answer(&item, "s-abc123", &Said::Landed("http://localhost:8976/callback?code=1".into())),
            (ANSWER, json!({ "product": "shop", "id": "w-1", "ask": "s-abc123", "said": "landed", "url": "http://localhost:8976/callback?code=1" }))
        );
        assert_eq!(keep(&item, "s-abc123", false), (KEEP, json!({ "product": "shop", "id": "w-1", "ask": "s-abc123", "keep": false })));
    }
}
