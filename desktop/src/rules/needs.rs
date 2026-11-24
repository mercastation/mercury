//! What a cloud run needs from the person, and how it asks. A port of
//! `shared/cloud-needs.mts`, function by function: a run that cannot go on without something
//! only they can give ends its answer with one marked line per thing, and the thread draws each
//! as a card (`cloud_need.rs`, from `NeedCard.tsx`).
//!
//!   AGENTBOX-NEEDS: sign-in supabase | to apply the storage rules
//!   AGENTBOX-NEEDS: key SENTRY_AUTH_TOKEN | a Sentry auth token
//!   AGENTBOX-NEEDS: file ~/Movies/launch-film-original.mov | the cut is made from the original
//!   AGENTBOX-NEEDS: code npm | the one-time password for npm publish
//!
//! A sign-in or a key is kept in the cloud and given to every later cloud run. A file goes with
//! the thread's next cloud turn, at the same path. A code goes back as the reply.

use super::js::{self, group, re};
use super::key_asks::{self, KeySource};
use crate::data::Item;

// `NEED`, `isNeedLine` and `threadsKeysFree` are read by no screen of this window: the last is
// the engine's own, which frees the threads a saved key lets go on. Kept with their cases.

/// `NEED`.
#[allow(dead_code)]
pub const NEED: &str = "AGENTBOX-NEEDS:";
// A closing backtick is taken off only when the line opened with one: the reason may hold inline code.
re!(NEED_LINE, r"^\s*(`?)AGENTBOX-NEEDS:\s*(sign-in|key|file|code)\s+([^|`]+?)\s*(?:\|\s*(.*?))?\s*\1\s*$", "i");
re!(KEY_NAME, r"^[A-Z][A-Z0-9_]{1,79}$", "");
re!(A_PATH, r"^(~\/|\/)", "");
re!(UNTIDY, r"^[—–:\s-]+", "");

/// A service a sign-in can name, with the variables its tools read and their labels, and the
/// page where a person makes a token (`Service`).
#[derive(Debug, PartialEq)]
pub struct Service {
    pub id: &'static str,
    pub name: &'static str,
    pub keys: &'static [(&'static str, &'static str)],
    pub token_url: &'static str,
}

/// `SERVICES`.
pub static SERVICES: [Service; 12] = [
    Service { id: "supabase", name: "Supabase", keys: &[("SUPABASE_ACCESS_TOKEN", "Supabase access token")], token_url: "https://supabase.com/dashboard/account/tokens" },
    Service { id: "vercel", name: "Vercel", keys: &[("VERCEL_TOKEN", "Vercel token")], token_url: "https://vercel.com/account/tokens" },
    Service { id: "github", name: "GitHub", keys: &[("GH_TOKEN", "GitHub token")], token_url: "https://github.com/settings/tokens" },
    Service { id: "stripe", name: "Stripe", keys: &[("STRIPE_SECRET_KEY", "Stripe secret key")], token_url: "https://dashboard.stripe.com/apikeys" },
    Service { id: "sentry", name: "Sentry", keys: &[("SENTRY_AUTH_TOKEN", "Sentry auth token")], token_url: "https://sentry.io/settings/account/api/auth-tokens/" },
    Service { id: "expo", name: "Expo", keys: &[("EXPO_TOKEN", "Expo access token")], token_url: "https://expo.dev/settings/access-tokens" },
    Service { id: "aws", name: "AWS", keys: &[("AWS_ACCESS_KEY_ID", "Access key ID"), ("AWS_SECRET_ACCESS_KEY", "Secret access key")], token_url: "https://console.aws.amazon.com/iam/home#/security_credentials" },
    Service { id: "npm", name: "npm", keys: &[("NPM_TOKEN", "npm access token")], token_url: "https://www.npmjs.com/settings/~/tokens" },
    Service { id: "netlify", name: "Netlify", keys: &[("NETLIFY_AUTH_TOKEN", "Netlify token")], token_url: "https://app.netlify.com/user/applications#personal-access-tokens" },
    Service { id: "cloudflare", name: "Cloudflare", keys: &[("CLOUDFLARE_API_TOKEN", "Cloudflare API token")], token_url: "https://dash.cloudflare.com/profile/api-tokens" },
    Service { id: "fly", name: "Fly.io", keys: &[("FLY_API_TOKEN", "Fly.io token")], token_url: "https://fly.io/user/personal_access_tokens" },
    Service { id: "linear", name: "Linear", keys: &[("LINEAR_API_KEY", "Linear API key")], token_url: "https://linear.app/settings/account/security" },
];

/// The service of that id, any case, its room trimmed (`serviceById`).
pub fn service_by_id(id: &str) -> Option<&'static Service> {
    let id = js::trim(id).to_lowercase();
    SERVICES.iter().find(|s| s.id == id)
}

/// One thing a message asks for (`Need`).
#[derive(Clone, Debug, PartialEq)]
pub enum Need {
    SignIn { service: &'static Service, why: String },
    Key { name: String, what: String, from: Option<KeySource> },
    File { path: String, why: String },
    Code { about: String, why: String },
}

/// `tidy`: trimmed, a leading dash, colon or space off.
fn tidy(s: &str) -> String {
    UNTIDY.strip(js::trim(s))
}

/// What a message asks for, in order and once each. A sign-in to a service not on the list is
/// left out: the run is told to ask for a key instead (`needsIn`).
pub fn needs_in(text: &str) -> Vec<Need> {
    let mut out: Vec<Need> = Vec::new();
    let mut add = |need: Need| {
        if !out.iter().any(|n| need_id(n) == need_id(&need)) {
            out.push(need);
        }
    };
    for line in text.split('\n') {
        let Some(m) = NEED_LINE.exec(line) else { continue };
        let kind = group(&m, 2).to_lowercase();
        let arg = js::trim(group(&m, 3));
        let why = tidy(group(&m, 4));
        match kind.as_str() {
            "sign-in" => {
                if let Some(service) = service_by_id(arg) {
                    add(Need::SignIn { service, why });
                }
            }
            "key" => {
                let words = js::words(arg);
                let name = words.first().copied().unwrap_or("");
                // A third part may say where on their computer it is.
                let parts: Vec<&str> = why.split('|').collect();
                let from = if parts.len() == 2 { key_asks::key_source_from_words(parts[1]) } else { None };
                let what = if from.is_some() { tidy(parts[0]) } else { why.clone() };
                if KEY_NAME.test(name) {
                    let what = if what.is_empty() { tidy(&words[1..].join(" ")) } else { what };
                    add(Need::Key { name: name.to_string(), what, from });
                }
            }
            "file" => {
                if A_PATH.test(arg) {
                    add(Need::File { path: arg.to_string(), why });
                }
            }
            _ => add(Need::Code { about: arg.to_string(), why }),
        }
    }
    for a in key_asks::key_asks(text) {
        add(Need::Key { name: a.name, what: a.what, from: None });
    }
    out
}

/// Whether `line` is one of those asks, in either form (`isNeedLine`).
#[allow(dead_code)]
pub fn is_need_line(line: &str) -> bool {
    !needs_in(line).is_empty()
}

/// One name for a need, the same in the thread and atop Needs you (`needId`).
pub fn need_id(need: &Need) -> String {
    match need {
        Need::SignIn { service, .. } => format!("sign-in:{}", service.id),
        Need::Key { name, .. } => format!("key:{name}"),
        Need::File { path, .. } => format!("file:{path}"),
        Need::Code { about, .. } => format!("code:{about}"),
    }
}

/// The variables a sign-in or a key asks to be saved; none for a file or a code (`keysOf`).
pub fn keys_of(need: &Need) -> Vec<&str> {
    match need {
        Need::SignIn { service, .. } => service.keys.iter().map(|(name, _)| *name).collect(),
        Need::Key { name, .. } => vec![name.as_str()],
        Need::File { .. } | Need::Code { .. } => Vec::new(),
    }
}

/// The need as the block the thread draws it from: `JSON.stringify(need)`, its fields in the
/// order the web window's object has them.
pub fn need_json(need: &Need) -> String {
    let s = |v: &str| serde_json::to_string(v).unwrap_or_default();
    match need {
        Need::SignIn { service, why } => {
            let keys: Vec<String> = service.keys.iter().map(|(name, label)| format!("{{\"name\":{},\"label\":{}}}", s(name), s(label))).collect();
            format!("{{\"kind\":\"sign-in\",\"service\":{{\"id\":{},\"name\":{},\"keys\":[{}],\"tokenUrl\":{}}},\"why\":{}}}", s(service.id), s(service.name), keys.join(","), s(service.token_url), s(why))
        }
        Need::Key { name, what, from } => {
            let from = from.as_ref().map_or_else(String::new, |f| {
                let entry = f.entry.as_deref().map_or_else(String::new, |e| format!(",\"entry\":{}", s(e)));
                format!(",\"from\":{{\"place\":{},\"from\":{}{entry}}}", s(f.place.word()), s(&f.from))
            });
            format!("{{\"kind\":\"key\",\"name\":{},\"what\":{}{from}}}", s(name), s(what))
        }
        Need::File { path, why } => format!("{{\"kind\":\"file\",\"path\":{},\"why\":{}}}", s(path), s(why)),
        Need::Code { about, why } => format!("{{\"kind\":\"code\",\"about\":{},\"why\":{}}}", s(about), s(why)),
    }
}

/// Said to since it asked: a reply taken back is no answer (`spokenSince`).
pub fn spoken_since(item: &Item) -> bool {
    item.string("answer") != Some("(withdrawn)") && item.wrote_at("answer") > item.wrote_at("result")
}

/// What an open thread still waits on: its latest answer's asks less the sign-ins and keys
/// already saved. Nothing once it has been answered since (`needsLeft`).
pub fn needs_left(item: &Item, saved: &[String]) -> Vec<Need> {
    if item.status != "open" || spoken_since(item) {
        return Vec::new();
    }
    needs_in(item.result.as_deref().unwrap_or(""))
        .into_iter()
        .filter(|n| {
            let keys = keys_of(n);
            keys.is_empty() || !keys.iter().all(|k| saved.iter().any(|s| s == k))
        })
        .collect()
}

/// The few words a thread's row says while it waits on these (`needsWord`).
pub fn needs_word_of(needs: &[Need]) -> Option<String> {
    let first = needs.first()?;
    if needs.len() > 1 {
        return Some(format!("Needs {} things", needs.len()));
    }
    Some(match first {
        Need::SignIn { service, .. } => format!("Needs {}", service.name),
        Need::File { .. } => "Needs a file".into(),
        Need::Code { .. } => "Needs a code".into(),
        Need::Key { .. } => "Needs a key".into(),
    })
}

/// What the Inbox's row says of a thread that still waits on what its latest answer asked for
/// (`needsWord(needsLeft(item, saved))`).
pub fn needs_word(item: &Item, saved: &[String]) -> Option<String> {
    needs_word_of(&needs_left(item, saved))
}

/// What goes back to a thread once its sign-ins and keys are all saved (`connectedReply`).
pub fn connected_reply(needs: &[Need]) -> String {
    let sign_ins: Vec<String> = needs.iter().filter_map(|n| if let Need::SignIn { service, .. } = n { Some(service.name.to_string()) } else { None }).collect();
    let keys: Vec<String> = needs.iter().filter_map(|n| if let Need::Key { name, what, .. } = n { Some(key_asks::key_words(what).map_or_else(|| name.clone(), |w| format!("the {w}"))) } else { None }).collect();
    let said: Vec<String> = [(!sign_ins.is_empty()).then(|| format!("connected {}", key_asks::listed(&sign_ins))), (!keys.is_empty()).then(|| format!("saved {}", key_asks::listed(&keys)))].into_iter().flatten().collect();
    format!("I {}. Carry on.", said.join(" and "))
}

/// The reply for a key given after the person answered its ask in words: the thread may have
/// moved on, so no "Carry on" (`lateReply`).
pub fn late_reply(needs: &[Need]) -> String {
    let said = connected_reply(needs);
    format!("{} after all, so you can use it now.", said.strip_suffix(". Carry on.").unwrap_or(&said))
}

/// A thread a saved key lets carry on, with the reply that sends it on.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub struct Freed {
    pub product: String,
    pub id: String,
    pub answer: String,
}

/// The threads a saved key lets carry on: open, no agent on them, their latest answer asking
/// only for sign-ins and keys that are now all saved, and nothing said to them since
/// (`threadsKeysFree`).
#[allow(dead_code)]
pub fn threads_keys_free(items: &[Item], saved: &[String]) -> Vec<Freed> {
    let mut out = Vec::new();
    for item in items {
        if item.id.is_empty() || item.status != "open" || item.truthy("agent") {
            continue;
        }
        let needs = needs_in(item.result.as_deref().unwrap_or(""));
        if needs.is_empty() || needs.iter().any(|n| matches!(n, Need::File { .. } | Need::Code { .. })) {
            continue;
        }
        if !needs.iter().all(|n| keys_of(n).iter().all(|k| saved.iter().any(|s| s == k))) {
            continue;
        }
        // A withdrawn reply is no answer.
        if spoken_since(item) {
            continue;
        }
        out.push(Freed { product: item.product.clone(), id: item.id.clone(), answer: connected_reply(&needs) });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::key_asks::{KEY_ASK, Place};
    use serde_json::json;

    const SIGN_IN: &str = "AGENTBOX-NEEDS: sign-in supabase | to apply the new storage rules to the Northwind database";
    const FILE: &str = "AGENTBOX-NEEDS: file ~/Movies/launch-film-original.mov | the cut is made from the original";
    const CODE: &str = "AGENTBOX-NEEDS: code npm | the one-time password for npm publish";
    const KEY: &str = "AGENTBOX-NEEDS: key SENTRY_AUTH_TOKEN | a Sentry auth token, from sentry.io/settings/account/api/auth-tokens";

    fn word(result: &str, saved: &[&str]) -> Option<String> {
        let item: Item = serde_json::from_value(json!({ "status": "open", "result": result, "wrote": { "result": { "ts": 10, "source": "agent" } } })).unwrap();
        needs_word(&item, &saved.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    fn key(name: &str, what: &str) -> Need {
        Need::Key { name: name.to_string(), what: what.to_string(), from: None }
    }

    fn sign_in(id: &str, why: &str) -> Need {
        Need::SignIn { service: service_by_id(id).unwrap(), why: why.to_string() }
    }

    fn row(id: &str, result: &str, status: &str, answered: Option<f64>, answer: Option<&str>) -> Item {
        let mut wrote = json!({ "result": { "ts": 10 } });
        if let Some(at) = answered {
            wrote["answer"] = json!({ "ts": at });
        }
        let mut row = json!({ "id": id, "product": "web", "title": "Conferences", "status": status, "result": result, "wrote": wrote });
        if let Some(a) = answer {
            row["answer"] = json!(a);
        }
        serde_json::from_value(row).unwrap()
    }

    fn saved(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_row_says_what_its_cloud_run_waits_on() {
        assert_eq!(word("Deploy is ready.\nAGENTBOX-NEEDS: sign-in vercel | to deploy the preview", &[]).as_deref(), Some("Needs Vercel"));
        assert_eq!(word("`AGENTBOX-NEEDS: key SENTRY_AUTH_TOKEN | a Sentry auth token`", &[]).as_deref(), Some("Needs a key"));
        assert_eq!(word("AGENTBOX-NEEDS: file ~/Movies/cut.mov | the original", &[]).as_deref(), Some("Needs a file"));
        assert_eq!(word("AGENTBOX-NEEDS: code npm | the one-time password", &[]).as_deref(), Some("Needs a code"));
        assert_eq!(word("AGENTBOX-NEEDS: sign-in vercel | a\nAGENTBOX-NEEDS: key SENTRY_AUTH_TOKEN | b", &[]).as_deref(), Some("Needs 2 things"));
        assert_eq!(word("AGENTBOX-NEEDS-A-KEY: STRIPE_SECRET_KEY — your Stripe secret key", &[]).as_deref(), Some("Needs a key"));
        // A service off the list is not a sign-in, and a saved key is no longer waited on.
        assert_eq!(word("AGENTBOX-NEEDS: sign-in mailroom | to send the post", &[]), None);
        assert_eq!(word("AGENTBOX-NEEDS: sign-in vercel | to deploy", &["VERCEL_TOKEN"]), None);
        assert_eq!(word("The export is fixed.", &[]), None);
        assert_eq!(needs_word_of(&needs_in(&format!("{SIGN_IN}\n{FILE}\n{CODE}"))).as_deref(), Some("Needs 3 things"));
        for (line, said) in [(SIGN_IN, Some("Needs Supabase")), (FILE, Some("Needs a file")), (CODE, Some("Needs a code")), (KEY, Some("Needs a key")), ("Done.", None)] {
            assert_eq!(needs_word_of(&needs_in(line)).as_deref(), said);
        }
    }

    #[test]
    fn a_reply_since_or_a_closed_thread_waits_on_nothing() {
        let replied: Item = serde_json::from_value(json!({ "status": "open", "result": "AGENTBOX-NEEDS: code npm | otp", "answer": "123456",
            "wrote": { "result": { "ts": 10, "source": "agent" }, "answer": { "ts": 20, "source": "founder" } } })).unwrap();
        assert_eq!(needs_word(&replied, &[]), None);
        let done: Item = serde_json::from_value(json!({ "status": "done", "result": "AGENTBOX-NEEDS: code npm | otp" })).unwrap();
        assert_eq!(needs_word(&done, &[]), None);
    }

    // a-cloud-run-asks-for-what-only-the-person-can-give.
    #[test]
    fn each_kind_of_line_is_read_with_what_it_is_for() {
        assert_eq!(
            needs_in(&format!("**I need a few things.**\n\n{SIGN_IN}\n{FILE}\n{CODE}\n{KEY}\n\n## Options\n1. Do it from my computer")),
            vec![
                sign_in("supabase", "to apply the new storage rules to the Northwind database"),
                Need::File { path: "~/Movies/launch-film-original.mov".into(), why: "the cut is made from the original".into() },
                Need::Code { about: "npm".into(), why: "the one-time password for npm publish".into() },
                key("SENTRY_AUTH_TOKEN", "a Sentry auth token, from sentry.io/settings/account/api/auth-tokens"),
            ]
        );
        for line in [format!("`{SIGN_IN}`"), "AGENTBOX-NEEDS: sign-in Supabase | to apply the rules".to_string(), format!("   {SIGN_IN}")] {
            assert!(matches!(needs_in(&line).first(), Some(Need::SignIn { service, .. }) if service.id == "supabase"), "{line}");
        }
        assert_eq!(needs_in("AGENTBOX-NEEDS: sign-in vercel\nAGENTBOX-NEEDS: sign-in vercel | again"), vec![sign_in("vercel", "")]);
        assert_eq!(needs_in(&format!("{KEY_ASK} VERCEL_TOKEN a Vercel token")), vec![key("VERCEL_TOKEN", "a Vercel token")]);
        for line in [
            "AGENTBOX-NEEDS: sign-in myspace | to post".to_string(),
            "AGENTBOX-NEEDS: key sentry_token | a token".to_string(),
            "AGENTBOX-NEEDS: file launch.mov | the film".to_string(),
            "AGENTBOX-NEEDS: phone | call me".to_string(),
            format!("I will end with {SIGN_IN}"),
            "AGENTBOX-NEEDS-THE-COMPUTER: Xcode".to_string(),
        ] {
            assert!(needs_in(&line).is_empty() && !is_need_line(&line), "{line}");
        }
    }

    #[test]
    fn the_services_each_have_a_page_to_make_a_token_on() {
        for s in &SERVICES {
            assert!(s.token_url.starts_with("https://"), "{}", s.id);
            for (name, _) in s.keys {
                assert!(KEY_NAME.test(name) && !key_asks::engine_reads(name), "{name}");
            }
        }
    }

    #[test]
    fn a_need_saves_every_variable_of_a_sign_in_and_none_for_a_file_or_a_code() {
        let needs = needs_in(&format!("AGENTBOX-NEEDS: sign-in aws\n{FILE}\n{CODE}\n{KEY}"));
        assert_eq!(keys_of(&needs[0]), ["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"]);
        assert!(keys_of(&needs[1]).is_empty() && keys_of(&needs[2]).is_empty());
        assert_eq!(keys_of(&needs[3]), ["SENTRY_AUTH_TOKEN"]);
    }

    #[test]
    fn a_saved_sign_in_frees_every_open_thread_waiting_on_it() {
        let freed = |items: &[Item], keys: &[&str]| threads_keys_free(items, &saved(keys));
        let supa = ["SUPABASE_ACCESS_TOKEN"];
        assert_eq!(
            freed(&[row("w-1", SIGN_IN, "open", None, None), row("w-2", &format!("**Hm.**\n\n{SIGN_IN}"), "open", None, None), row("w-3", "Done.", "open", None, None)], &supa),
            vec![Freed { product: "web".into(), id: "w-1".into(), answer: "I connected Supabase. Carry on.".into() }, Freed { product: "web".into(), id: "w-2".into(), answer: "I connected Supabase. Carry on.".into() }]
        );
        let aws = "AGENTBOX-NEEDS: sign-in aws | to upload";
        assert!(freed(&[row("w-1", aws, "open", None, None)], &["AWS_ACCESS_KEY_ID"]).is_empty());
        assert!(freed(&[row("w-1", &format!("{aws}\n{KEY}"), "open", None, None)], &["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"]).is_empty());
        assert_eq!(freed(&[row("w-1", &format!("{aws}\n{KEY}"), "open", None, None)], &["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY", "SENTRY_AUTH_TOKEN"])[0].answer, "I connected AWS and saved the Sentry auth token. Carry on.");
        assert!(freed(&[row("w-1", &format!("{SIGN_IN}\n{FILE}"), "open", None, None), row("w-2", &format!("{SIGN_IN}\n{CODE}"), "open", None, None)], &supa).is_empty());
        assert!(freed(&[row("w-1", SIGN_IN, "done", None, None), row("w-2", SIGN_IN, "claimed", None, None), row("w-3", SIGN_IN, "open", Some(20.0), None)], &supa).is_empty());
        assert_eq!(connected_reply(&needs_in("AGENTBOX-NEEDS: sign-in vercel\nAGENTBOX-NEEDS: sign-in github")), "I connected Vercel and GitHub. Carry on.");
        // a-cloud-run-missing-a-key-asks-for-it-on-the-thread: the older line, each with the
        // reply that names its keys.
        let ask = format!("{KEY_ASK} SUPABASE_ACCESS_TOKEN your Supabase access token, from supabase.com/dashboard/account/tokens");
        let paste = format!("**Paste it.**\n\n{ask}");
        assert_eq!(freed(&[row("w-1", &paste, "open", None, None), row("w-2", &paste, "open", None, None), row("w-3", "Done.", "open", None, None)], &supa).iter().map(|f| f.answer.as_str()).collect::<Vec<_>>(), ["I saved the Supabase access token. Carry on."; 2]);
        let two = format!("{KEY_ASK} AWS_ACCESS_KEY_ID the id\n{KEY_ASK} AWS_SECRET_ACCESS_KEY the secret");
        assert!(freed(&[row("w-1", &two, "open", None, None)], &["AWS_ACCESS_KEY_ID"]).is_empty());
        assert_eq!(freed(&[row("w-1", &two, "open", None, None)], &["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"])[0].answer, "I saved the id and the secret. Carry on.");
        assert_eq!(freed(&[row("w-1", &format!("{KEY_ASK} AWS_ACCESS_KEY_ID\n{KEY_ASK} AWS_SECRET_ACCESS_KEY"), "open", None, None)], &["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"])[0].answer, "I saved AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY. Carry on.");
        assert!(freed(&[row("w-1", &paste, "done", None, None), row("w-2", &paste, "claimed", None, None), row("w-3", &paste, "open", Some(20.0), None)], &supa).is_empty());
        assert_eq!(freed(&[row("w-4", &paste, "open", Some(5.0), None)], &supa).len(), 1);
    }

    // a-cloud-ask-is-answered-once-and-checked.
    #[test]
    fn what_a_thread_still_waits_on_leaves_out_what_is_saved() {
        let both = "**I need two sign-ins.**\n\nAGENTBOX-NEEDS: sign-in supabase | to apply the change\nAGENTBOX-NEEDS: sign-in sentry | to read the errors";
        let ids = |needs: Vec<Need>| needs.iter().map(need_id).collect::<Vec<_>>();
        assert_eq!(ids(needs_left(&row("w-1", both, "open", None, None), &[])), ["sign-in:supabase", "sign-in:sentry"]);
        assert_eq!(ids(needs_left(&row("w-1", both, "open", None, None), &saved(&["SUPABASE_ACCESS_TOKEN"]))), ["sign-in:sentry"]);
        assert_eq!(needs_word_of(&needs_left(&row("w-1", both, "open", None, None), &saved(&["SUPABASE_ACCESS_TOKEN"]))).as_deref(), Some("Needs Sentry"));
        assert!(needs_left(&row("w-1", both, "open", None, None), &saved(&["SUPABASE_ACCESS_TOKEN", "SENTRY_AUTH_TOKEN"])).is_empty());
        assert!(needs_left(&row("w-1", both, "open", Some(20.0), Some("Do it from my computer.")), &[]).is_empty());
        assert!(needs_left(&row("w-1", both, "claimed", None, None), &[]).is_empty() && needs_left(&row("w-1", both, "done", None, None), &[]).is_empty());
        assert_eq!(needs_left(&row("w-1", both, "open", Some(20.0), Some("(withdrawn)")), &[]).len(), 2);
        let gh = "AGENTBOX-NEEDS: sign-in github | to open the pull request";
        assert_eq!(threads_keys_free(&[row("w-1", gh, "open", Some(20.0), Some("(withdrawn)"))], &saved(&["GH_TOKEN"])).len(), 1);
        assert!(threads_keys_free(&[row("w-1", gh, "open", Some(20.0), Some("wait"))], &saved(&["GH_TOKEN"])).is_empty());
        let three = needs_in("AGENTBOX-NEEDS: sign-in stripe | to read the MRR\nAGENTBOX-NEEDS: file ~/Movies/launch-film-original.mov | to attach it\nAGENTBOX-NEEDS: code npm | to publish");
        assert_eq!(three.iter().map(need_id).collect::<Vec<_>>(), ["sign-in:stripe", "file:~/Movies/launch-film-original.mov", "code:npm"]);
    }

    // an-ask-with-a-command-in-it-still-draws-its-card.
    #[test]
    fn an_ask_whose_reason_holds_inline_code_is_read_code_and_all() {
        let what = "the treg agent token from `treg org agent-new x-replies`, so cloud runs can search X (about $0.13 a day)";
        let treg = format!("AGENTBOX-NEEDS: key TREG_TOKEN | {what}");
        assert_eq!(needs_in(&treg), vec![key("TREG_TOKEN", what)]);
        assert_eq!(needs_in("AGENTBOX-NEEDS: key TREG_TOKEN | from `treg org agent-new x-replies`"), vec![key("TREG_TOKEN", "from `treg org agent-new x-replies`")]);
        assert_eq!(needs_in("AGENTBOX-NEEDS: sign-in vercel | to run `vercel deploy`"), vec![sign_in("vercel", "to run `vercel deploy`")]);
        assert_eq!(needs_in("AGENTBOX-NEEDS: file ~/Movies/a.mov | for `ffmpeg`"), vec![Need::File { path: "~/Movies/a.mov".into(), why: "for `ffmpeg`".into() }]);
        assert_eq!(needs_in("`AGENTBOX-NEEDS: key TREG_TOKEN | the treg agent token`"), vec![key("TREG_TOKEN", "the treg agent token")]);
        assert_eq!(needs_in("`AGENTBOX-NEEDS: key TREG_TOKEN`"), vec![key("TREG_TOKEN", "")]);
        let drawn = key_asks::needs_as_blocks(&format!("**Next I need a treg token.**\n\n{treg}\n\n## Options\n1. Token's in"));
        assert!(drawn.contains("```agentbox-need") && !drawn.contains("AGENTBOX-NEEDS"));
        assert!(needs_in("A run ends with `AGENTBOX-NEEDS:` lines, one per thing.").is_empty());
        assert!(needs_in("Then AGENTBOX-NEEDS: key lowercase_name | `x`").is_empty());
    }

    // the-key-cards-say-where-on-this-computer-the-key-comes-from, a-key-card-never-reads-a-sign-in-the-cloud-may-not-have.
    #[test]
    fn a_place_after_the_key_is_read_as_a_third_part() {
        let line = "AGENTBOX-NEEDS: key POSTHOG_PERSONAL_KEY | the PostHog personal API key the daily report reads | keychain posthog-read-key";
        assert_eq!(needs_in(line), vec![Need::Key { name: "POSTHOG_PERSONAL_KEY".into(), what: "the PostHog personal API key the daily report reads".into(), from: Some(KeySource { place: Place::Keychain, from: "posthog-read-key".into(), entry: None }) }]);
        let Need::Key { from, .. } = &needs_in("AGENTBOX-NEEDS: key POSTHOG_KEY | the PostHog key | file ~/app/.env POSTHOG_KEY")[0] else { panic!() };
        assert_eq!(from, &Some(KeySource { place: Place::File, from: "~/app/.env".into(), entry: Some("POSTHOG_KEY".into()) }));
        assert_eq!(needs_in("AGENTBOX-NEEDS: key SENTRY_AUTH_TOKEN | a Sentry auth token, from sentry.io/settings"), vec![key("SENTRY_AUTH_TOKEN", "a Sentry auth token, from sentry.io/settings")]);
        assert_eq!(needs_in("AGENTBOX-NEEDS: key SENTRY_AUTH_TOKEN | a Sentry auth token | from sentry.io/settings"), vec![key("SENTRY_AUTH_TOKEN", "a Sentry auth token | from sentry.io/settings")]);
        assert_eq!(needs_in("AGENTBOX-NEEDS: key POSTHOG_READ_KEY | the PostHog read key | file ~/.codex/auth.json"), vec![key("POSTHOG_READ_KEY", "the PostHog read key | file ~/.codex/auth.json")]);
    }

    // an-answered-ask-can-still-be-given.
    #[test]
    fn a_key_that_comes_late_is_said_to_have_been_saved_after_all() {
        let needs = needs_in("AGENTBOX-NEEDS: key ELEVENLABS_API_KEY | ElevenLabs API key, from elevenlabs.io/app/settings/api-keys\nAGENTBOX-NEEDS: sign-in sentry | to read the errors");
        assert_eq!(late_reply(&needs[..1]), "I saved the ElevenLabs API key after all, so you can use it now.");
        assert_eq!(late_reply(&needs[1..]), "I connected Sentry after all, so you can use it now.");
    }

    #[test]
    fn a_need_is_its_block_as_the_web_window_writes_it() {
        assert_eq!(need_json(&sign_in("aws", "to \"upload\"")), r#"{"kind":"sign-in","service":{"id":"aws","name":"AWS","keys":[{"name":"AWS_ACCESS_KEY_ID","label":"Access key ID"},{"name":"AWS_SECRET_ACCESS_KEY","label":"Secret access key"}],"tokenUrl":"https://console.aws.amazon.com/iam/home#/security_credentials"},"why":"to \"upload\""}"#);
        let from = Need::Key { name: "K_KEY".into(), what: "w".into(), from: Some(KeySource { place: Place::File, from: "~/a/.env".into(), entry: Some("K".into()) }) };
        assert_eq!(need_json(&from), r#"{"kind":"key","name":"K_KEY","what":"w","from":{"place":"file","from":"~/a/.env","entry":"K"}}"#);
        assert_eq!(need_json(&Need::Code { about: "npm".into(), why: "".into() }), r#"{"kind":"code","about":"npm","why":""}"#);
        // The thread's own drawing of the lines (`needsAsBlocks`).
        let out = key_asks::needs_as_blocks(&format!("**I need two things.**\n\n{SIGN_IN}\n{FILE}\n\nAfter."));
        assert_eq!(out.matches("```agentbox-need\n").count(), 2);
        assert!(out.starts_with("**I need two things.**") && js::trim_end(&out).ends_with("After."));
        assert_eq!(key_asks::needs_as_blocks("Nothing asked."), "Nothing asked.");
    }
}
