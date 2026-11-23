//! What an agent did in its project's browser, as the film the browser's host keeps of it
//! (`main/project-browser/film.mts`): the steps read back from `steps.jsonl`, the words each is
//! said in, which step a line of the conversation is, and where the pointer goes on a picture.
//!
//! Nothing here reads a file or draws: `film.rs` does both.

use serde_json::Value;

/// One step an agent took in the browser: its first line (`tool`, `what`, `at`, where it
/// pressed) and, once it is done, its second (`end`, the page after).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Step {
    pub n: u64,
    /// Playwright MCP's own name for the tool (`browser_click`).
    pub tool: String,
    /// What it acted on, in a few words; never what was typed.
    pub what: String,
    pub at: f64,
    pub end: Option<f64>,
    pub url: String,
    /// The page it left the browser on, when that is another.
    pub went_to: String,
    /// Where it pressed, in the page's pixels, and the page's size.
    pub mark: Option<[f32; 4]>,
    pub view: [f32; 2],
    /// The page the moment before it pressed, and once it was done: file names in its folder.
    pub before: Option<String>,
    pub after: Option<String>,
    pub failed: bool,
}

impl Step {
    pub fn running(&self) -> bool {
        self.end.is_none()
    }
    /// A step that only read the page changed nothing of it, and took no picture.
    pub fn reads(&self) -> bool {
        matches!(self.tool.as_str(), "browser_snapshot" | "browser_find" | "browser_take_screenshot")
    }
    /// The point a press landed on, in the page's pixels: the middle of what it pressed.
    pub fn point(&self) -> Option<(f32, f32)> {
        self.mark.map(|[x, y, w, h]| (x + w / 2.0, y + h / 2.0))
    }
    /// The page it ended on: where it went, or where it was.
    pub fn ended_on(&self) -> &str {
        if self.went_to.is_empty() { &self.url } else { &self.went_to }
    }
    /// The site it ended on, as a person names it.
    pub fn site(&self) -> String {
        site_of(self.ended_on())
    }
}

/// An address's host, without `www.`; nothing for a page with none (`about:blank`).
pub fn site_of(url: &str) -> String {
    let Some((_, rest)) = url.split_once("://") else { return String::new() };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = host.rsplit_once('@').map_or(host, |(_, h)| h);
    let host = host.split(':').next().unwrap_or("");
    host.strip_prefix("www.").unwrap_or(host).to_string()
}

fn num(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(Value::as_f64)
}
fn word(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_string)
}
fn four(v: &Value, key: &str) -> Option<[f32; 4]> {
    let a = v.get(key)?.as_array()?;
    if a.len() != 4 {
        return None;
    }
    let n: Vec<f32> = a.iter().filter_map(Value::as_f64).map(|n| n as f32).collect();
    (n.len() == 4).then(|| [n[0], n[1], n[2], n[3]])
}

/// Reads lines the host wrote onto the steps already held: a first line adds a step, a second
/// finishes the one it numbers. A line cut short by a write still under way, or any line it
/// cannot read, is passed over; the caller reads from the last whole line again.
pub fn take_lines(steps: &mut Vec<Step>, text: &str) {
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        let Some(n) = v.get("n").and_then(Value::as_u64) else { continue };
        if let Some(tool) = word(&v, "tool") {
            let view = v.get("view").and_then(Value::as_array).map(|a| [a.first().and_then(Value::as_f64).unwrap_or(1280.0) as f32, a.get(1).and_then(Value::as_f64).unwrap_or(720.0) as f32]).unwrap_or([1280.0, 720.0]);
            let step = Step { n, tool, what: word(&v, "what").unwrap_or_default(), at: num(&v, "at").unwrap_or(0.0), url: word(&v, "url").unwrap_or_default(), mark: four(&v, "box"), view, before: word(&v, "before"), ..Step::default() };
            match steps.iter_mut().find(|s| s.n == n) {
                Some(had) => *had = step,
                None => steps.push(step),
            }
        } else if let Some(end) = num(&v, "end") {
            let Some(step) = steps.iter_mut().find(|s| s.n == n) else { continue };
            step.end = Some(end);
            step.after = word(&v, "after");
            step.failed = v.get("failed").and_then(Value::as_bool).unwrap_or(false);
            step.went_to = word(&v, "url").unwrap_or_default();
        }
    }
}

/// The step's words: what it is doing while it runs (`Clicking`), or what it did (`clicked`),
/// and what on. The verbs are the conversation's own kind of words (`work_verb`'s plain ones).
pub fn words(step: &Step, now_doing: bool) -> (String, String) {
    let (doing, did) = match step.tool.as_str() {
        "browser_navigate" => ("Opening", "opened"),
        "browser_navigate_back" => ("Going back", "went back"),
        "browser_click" => ("Clicking", "clicked"),
        "browser_hover" => ("Pointing at", "pointed at"),
        "browser_type" => ("Typing in", "typed in"),
        "browser_press_key" => ("Pressing", "pressed"),
        "browser_select_option" => ("Choosing in", "chose in"),
        "browser_fill_form" => ("Filling in", "filled in"),
        "browser_handle_dialog" => ("Answering a dialog", "answered a dialog"),
        "browser_wait_for" => ("Waiting for", "waited for"),
        "browser_tabs" => ("Switching to", "switched to"),
        "browser_snapshot" => ("Reading the page", "read the page"),
        "browser_find" => ("Looking for", "looked for"),
        "browser_take_screenshot" => ("Taking a picture of", "took a picture of"),
        _ => ("Working in the browser", "worked in the browser"),
    };
    let what = match step.tool.as_str() {
        // A dialog says how it was answered in its verb already.
        "browser_handle_dialog" | "browser_navigate_back" | "browser_snapshot" => String::new(),
        "browser_take_screenshot" if step.what.is_empty() => "the page".to_string(),
        _ => step.what.clone(),
    };
    (if now_doing { doing } else { did }.to_string(), what)
}

/// The balloon's words while a step runs, or over its picture afterwards, starting as a
/// sentence does.
pub fn balloon(step: &Step, now_doing: bool) -> String {
    let (verb, what) = words(step, now_doing);
    let said = if what.is_empty() { verb } else { format!("{verb} {what}") };
    let mut chars = said.chars();
    chars.next().map_or(String::new(), |c| c.to_uppercase().chain(chars).collect())
}

/// The page's address as a person reads it: host and path, no scheme.
pub fn address(url: &str) -> String {
    let Some((_, rest)) = url.split_once("://") else { return String::new() };
    let rest = rest.strip_prefix("www.").unwrap_or(rest);
    rest.trim_end_matches('/').to_string()
}

/// The conversation's own verb for a browser tool's line (`work_verb` of
/// `mcp__project-browser__browser_click`), so a line can be told to be one.
pub fn line_verb(tool: &str) -> String {
    tool.replace('_', " ")
}

/// Whether a line of the conversation is one of the browser's: its verb is a browser tool's.
pub fn is_browser_line(verb: &str) -> bool {
    verb.starts_with("browser ")
}

/// A run's log stamps each tool to the second, when the agent asked for it; the host stamps the
/// step when it got the call, a moment later. A step is a line's when it is the same tool and
/// started from just before the line's second to a few seconds after it.
const EARLY_MS: f64 = 1_000.0;
const LATE_MS: f64 = 6_000.0;

/// Which step each of a run's lines is, in order: each browser line takes the first step not
/// taken yet that is its tool and in its time. A line the film has no step for stays `None`, as
/// a step from before filming began, or one whose film was let go, does.
pub fn match_lines(lines: &[(f64, &str)], steps: &[Step]) -> Vec<Option<usize>> {
    let mut from = 0;
    lines
        .iter()
        .map(|(at, verb)| {
            if !is_browser_line(verb) {
                return None;
            }
            let found = (from..steps.len()).find(|&i| {
                let s = &steps[i];
                line_verb(&s.tool) == *verb && s.at >= at - EARLY_MS && s.at <= at + LATE_MS
            })?;
            from = found + 1;
            Some(found)
        })
        .collect()
}

/// The picture a step is shown with, and whether its pointer goes on it: a press is shown the
/// moment before, with the pointer where it pressed; anything else as the page was after it. A
/// step with neither (a read, or one whose pictures were let go) shows the page as the last step
/// before it left it.
pub fn picture_of(steps: &[Step], at: usize) -> Option<(String, bool)> {
    let step = steps.get(at)?;
    if let (Some(before), Some(_)) = (&step.before, step.mark) {
        return Some((before.clone(), true));
    }
    if let Some(after) = &step.after {
        return Some((after.clone(), false));
    }
    steps[..at].iter().rev().find_map(|s| s.after.clone().or_else(|| s.before.clone())).map(|f| (f, false))
}

/// How long after a step ends its live view stays up, showing the page it left: an agent thinks
/// between steps, and the card would come and go with each.
pub const LIVE_AFTER_MS: f64 = 45_000.0;

/// The step the live view shows, while a run is up on the thread: the newest one this run
/// took, while it runs or shortly after. None when the run has not used the browser lately.
pub fn live_step(steps: &[Step], run_started: f64, now: f64) -> Option<usize> {
    let at = steps.len().checked_sub(1)?;
    let step = &steps[at];
    if step.at + EARLY_MS < run_started {
        return None;
    }
    match step.end {
        None => Some(at),
        Some(end) if now - end <= LIVE_AFTER_MS => Some(at),
        _ => None,
    }
}

/// Where a point on the page lands on its picture drawn `wide` points across.
pub fn on_picture(point: (f32, f32), view: [f32; 2], wide: f32) -> (f32, f32) {
    let scale = wide / view[0].max(1.0);
    (point.0 * scale, point.1 * scale)
}

/// Where the balloon goes beside the pointer at `tip`, `size` large, inside a picture `room`
/// large: under and right of the tip, else under and left, and never off the picture.
pub fn balloon_at(tip: (f32, f32), size: (f32, f32), room: (f32, f32)) -> (f32, f32) {
    let (gap_x, gap_y) = (14.0, 18.0);
    let x = if tip.0 + gap_x + size.0 <= room.0 - 6.0 { tip.0 + gap_x } else { tip.0 - size.0 - 6.0 };
    let y = if tip.1 + gap_y + size.1 <= room.1 - 6.0 { tip.1 + gap_y } else { tip.1 - size.1 - 10.0 };
    (x.clamp(6.0, (room.0 - size.0 - 6.0).max(6.0)), y.clamp(6.0, (room.1 - size.1 - 6.0).max(6.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<Step> {
        let mut steps = Vec::new();
        take_lines(&mut steps, text);
        steps
    }

    const FILM: &str = r#"{"n":1,"tool":"browser_navigate","what":"gitlab.example.com/group","at":1000,"url":"about:blank","view":[1280,720]}
{"n":1,"end":3000,"after":"1b.jpg","url":"https://gitlab.example.com/group"}
{"n":2,"tool":"browser_snapshot","what":"","at":5000,"url":"https://gitlab.example.com/group","view":[1280,720]}
{"n":2,"end":5200,"url":"https://gitlab.example.com/group"}
{"n":3,"tool":"browser_click","what":"New issue","at":9000,"url":"https://gitlab.example.com/group","box":[600,400,80,32],"view":[1280,720],"before":"3a.jpg"}
{"n":3,"end":9700,"after":"3b.jpg","url":"https://gitlab.example.com/group/issues/new"}
{"n":4,"tool":"browser_type","what":"Title","at":12000,"box":[100,200,300,30],"view":[1280,720],"before":"4a.jpg"}
{"n":4,"end""#;

    #[test]
    fn a_film_is_read_back_into_its_steps_and_a_line_cut_short_waits() {
        let steps = lines(FILM);
        assert_eq!(steps.len(), 4);
        assert_eq!(steps[0].after.as_deref(), Some("1b.jpg"));
        assert_eq!(steps[0].site(), "gitlab.example.com");
        assert_eq!(steps[2].mark, Some([600.0, 400.0, 80.0, 32.0]));
        assert_eq!(steps[2].point(), Some((640.0, 416.0)));
        assert_eq!(steps[2].url, "https://gitlab.example.com/group");
        assert_eq!(steps[2].ended_on(), "https://gitlab.example.com/group/issues/new");
        // The last line was still being written: the step is still running.
        assert!(steps[3].running());
        // Read on from there, it finishes.
        let mut more = steps.clone();
        take_lines(&mut more, r#"{"n":4,"end":12500,"after":"4b.jpg","failed":true}"#);
        assert!(!more[3].running() && more[3].failed);
    }

    #[test]
    fn each_step_is_said_as_it_runs_and_as_it_was_done() {
        let steps = lines(FILM);
        assert_eq!(balloon(&steps[2], true), "Clicking New issue");
        assert_eq!(words(&steps[2], false), ("clicked".to_string(), "New issue".to_string()));
        assert_eq!(balloon(&steps[1], true), "Reading the page");
        assert_eq!(balloon(&steps[0], false), "Opened gitlab.example.com/group");
        assert_eq!(address("https://www.acme.dev/a/b/"), "acme.dev/a/b");
    }

    #[test]
    fn a_runs_browser_lines_find_their_steps_in_order_and_in_time() {
        let steps = lines(FILM);
        // The log's seconds: each a moment before the host heard of the call.
        let run = [(0.0, "browser navigate"), (2000.0, "read"), (5000.0, "browser snapshot"), (8000.0, "browser click"), (11000.0, "browser type"), (30000.0, "browser click")];
        assert_eq!(match_lines(&run, &steps), vec![Some(0), None, Some(1), Some(2), Some(3), None]);
        // A line from before the film began finds nothing, and takes nothing from the next.
        assert_eq!(match_lines(&[(-60000.0, "browser click"), (8000.0, "browser click")], &steps), vec![None, Some(2)]);
    }

    #[test]
    fn a_press_is_shown_the_moment_before_it_with_its_pointer_and_a_read_as_the_page_it_read() {
        let steps = lines(FILM);
        assert_eq!(picture_of(&steps, 2), Some(("3a.jpg".to_string(), true)));
        assert_eq!(picture_of(&steps, 0), Some(("1b.jpg".to_string(), false)));
        assert_eq!(picture_of(&steps, 1), Some(("1b.jpg".to_string(), false)));
        assert_eq!(picture_of(&steps, 9), None);
    }

    #[test]
    fn the_live_view_follows_this_runs_newest_step_and_leaves_once_it_has_been_quiet() {
        let steps = lines(FILM);
        assert_eq!(live_step(&steps, 500.0, 13000.0), Some(3));
        let mut done = steps.clone();
        take_lines(&mut done, r#"{"n":4,"end":12500,"after":"4b.jpg"}"#);
        assert_eq!(live_step(&done, 500.0, 12500.0 + LIVE_AFTER_MS), Some(3));
        assert_eq!(live_step(&done, 500.0, 12501.0 + LIVE_AFTER_MS), None);
        // A run that started after the film's newest step has not used the browser.
        assert_eq!(live_step(&done, 20000.0, 21000.0), None);
        assert_eq!(live_step(&[], 0.0, 0.0), None);
    }

    #[test]
    fn the_pointer_lands_on_the_picture_and_its_balloon_stays_inside_it() {
        assert_eq!(on_picture((640.0, 360.0), [1280.0, 720.0], 640.0), (320.0, 180.0));
        let room = (640.0, 360.0);
        assert_eq!(balloon_at((100.0, 100.0), (120.0, 28.0), room), (114.0, 118.0));
        // Near the right edge it goes to the pointer's left; near the foot, above it.
        let (x, y) = balloon_at((600.0, 340.0), (120.0, 28.0), room);
        assert!(x + 120.0 <= 600.0 && y + 28.0 <= 340.0);
        // Wider than the picture, it starts at its edge.
        assert_eq!(balloon_at((10.0, 10.0), (900.0, 28.0), room).0, 6.0);
    }

    #[test]
    fn a_site_is_its_host() {
        assert_eq!(site_of("https://www.acme.dev:8443/a?b"), "acme.dev");
        assert_eq!(site_of("about:blank"), "");
        assert_eq!(site_of(""), "");
    }
}
