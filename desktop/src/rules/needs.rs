//! What a cloud run needs from the person, and the few words a thread's row says while it
//! waits on it. A port of the reading half of `shared/cloud-needs.mts`: a run that cannot go
//! on ends its answer with one marked line per thing.
//!
//!   AGENTBOX-NEEDS: sign-in supabase | to apply the storage rules
//!   AGENTBOX-NEEDS: key SENTRY_AUTH_TOKEN | a Sentry auth token
//!   AGENTBOX-NEEDS: file ~/Movies/launch-film-original.mov | the cut is made from the original
//!   AGENTBOX-NEEDS: code npm | the one-time password for npm publish

use super::js::{self, group, re};
use crate::data::Item;

// A closing backtick is taken off only when the line opened with one: the reason may hold inline code.
re!(NEED_LINE, r"^\s*(`?)AGENTBOX-NEEDS:\s*(sign-in|key|file|code)\s+([^|`]+?)\s*(?:\|\s*(.*?))?\s*\1\s*$", "i");
re!(KEY_NAME, r"^[A-Z][A-Z0-9_]{1,79}$", "");
re!(A_PATH, r"^(~\/|\/)", "");
// The older form of a key ask (`shared/cloud-keys.mts`).
re!(KEY_ASK_LINE, r"^\s*AGENTBOX-NEEDS-A-KEY:\s*([A-Z][A-Z0-9_]{1,79})\b[\s:\u2014\u2013-]*(.*)$", "");
re!(OUTER_BACKTICKS, r"^`+|`+$", "g");

/// The services a sign-in can name, each with its name and the variables its tools read.
const SERVICES: [(&str, &str, &[&str]); 12] = [
    ("supabase", "Supabase", &["SUPABASE_ACCESS_TOKEN"]),
    ("vercel", "Vercel", &["VERCEL_TOKEN"]),
    ("github", "GitHub", &["GH_TOKEN"]),
    ("stripe", "Stripe", &["STRIPE_SECRET_KEY"]),
    ("sentry", "Sentry", &["SENTRY_AUTH_TOKEN"]),
    ("expo", "Expo", &["EXPO_TOKEN"]),
    ("aws", "AWS", &["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"]),
    ("npm", "npm", &["NPM_TOKEN"]),
    ("netlify", "Netlify", &["NETLIFY_AUTH_TOKEN"]),
    ("cloudflare", "Cloudflare", &["CLOUDFLARE_API_TOKEN"]),
    ("fly", "Fly.io", &["FLY_API_TOKEN"]),
    ("linear", "Linear", &["LINEAR_API_KEY"]),
];

/// One thing a message asks for.
#[derive(Clone, Debug, PartialEq)]
enum Need {
    SignIn { name: &'static str, keys: &'static [&'static str] },
    Key(String),
    File,
    Code,
}

/// What a message asks for, in order and once each (`needsIn`). A sign-in to a service not on
/// the list is left out: the run is told to ask for a key instead.
fn needs_in(text: &str) -> Vec<Need> {
    let mut out: Vec<(String, Need)> = Vec::new();
    let mut add = |id: String, need: Need| {
        if !out.iter().any(|(seen, _)| *seen == id) {
            out.push((id, need));
        }
    };
    for line in text.split('\n') {
        let Some(m) = NEED_LINE.exec(line) else { continue };
        let kind = group(&m, 2).to_lowercase();
        let arg = js::trim(group(&m, 3));
        match kind.as_str() {
            "sign-in" => {
                let id = js::trim(arg).to_lowercase();
                if let Some((id, name, keys)) = SERVICES.iter().find(|s| s.0 == id) {
                    add(format!("sign-in:{id}"), Need::SignIn { name, keys });
                }
            }
            "key" => {
                let name = js::words(arg).first().copied().unwrap_or("");
                if KEY_NAME.test(name) {
                    add(format!("key:{name}"), Need::Key(name.to_string()));
                }
            }
            "file" => {
                if A_PATH.test(arg) {
                    add(format!("file:{arg}"), Need::File);
                }
            }
            _ => add(format!("code:{arg}"), Need::Code),
        }
    }
    for line in text.split('\n') {
        if let Some(m) = KEY_ASK_LINE.exec(&OUTER_BACKTICKS.strip_all(line)) {
            let name = group(&m, 1);
            add(format!("key:{name}"), Need::Key(name.to_string()));
        }
    }
    out.into_iter().map(|(_, need)| need).collect()
}

/// The variables a sign-in or a key asks to be saved; none for a file or a code (`keysOf`).
fn keys_of(need: &Need) -> Vec<&str> {
    match need {
        Need::SignIn { keys, .. } => keys.to_vec(),
        Need::Key(name) => vec![name.as_str()],
        Need::File | Need::Code => Vec::new(),
    }
}

/// The few words a thread's row says while it still waits on what its latest answer asks for,
/// less the sign-ins and keys already saved. Nothing once it has been answered since, and
/// nothing on a thread that is not open (`needsWord` of `needsLeft`).
pub fn needs_word(item: &Item, saved: &[String]) -> Option<String> {
    let spoken_since = item.string("answer") != Some("(withdrawn)") && item.wrote_at("answer") > item.wrote_at("result");
    if item.status != "open" || spoken_since {
        return None;
    }
    let left: Vec<Need> = needs_in(item.result.as_deref().unwrap_or(""))
        .into_iter()
        .filter(|n| {
            let keys = keys_of(n);
            keys.is_empty() || !keys.iter().all(|k| saved.iter().any(|s| s == k))
        })
        .collect();
    let first = left.first()?;
    if left.len() > 1 {
        return Some(format!("Needs {} things", left.len()));
    }
    Some(match first {
        Need::SignIn { name, .. } => format!("Needs {name}"),
        Need::File => "Needs a file".into(),
        Need::Code => "Needs a code".into(),
        Need::Key(_) => "Needs a key".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn word(result: &str, saved: &[&str]) -> Option<String> {
        let item: Item = serde_json::from_value(json!({ "status": "open", "result": result, "wrote": { "result": { "ts": 10, "source": "agent" } } })).unwrap();
        needs_word(&item, &saved.iter().map(|s| s.to_string()).collect::<Vec<_>>())
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
    }

    #[test]
    fn a_reply_since_or_a_closed_thread_waits_on_nothing() {
        let replied: Item = serde_json::from_value(json!({ "status": "open", "result": "AGENTBOX-NEEDS: code npm | otp", "answer": "123456",
            "wrote": { "result": { "ts": 10, "source": "agent" }, "answer": { "ts": 20, "source": "founder" } } })).unwrap();
        assert_eq!(needs_word(&replied, &[]), None);
        let done: Item = serde_json::from_value(json!({ "status": "done", "result": "AGENTBOX-NEEDS: code npm | otp" })).unwrap();
        assert_eq!(needs_word(&done, &[]), None);
    }
}
