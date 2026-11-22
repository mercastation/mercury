//! Later: the moments its picker offers, and the time a person types into it in words.
//!
//! The grammar is `renderer/src/when-words.ts`, word for word: a phrase is cut into words and
//! every word has to be spent on something this knows (a stretch of time, a day, a date, a
//! clock, a part of the day) or on a filler like "at". One word left over and the phrase is
//! refused, which the picker shows as "not a time". A refusal is honest; a guess is a thread
//! that comes back when nobody asked for it.
//!
//! What the web's box also reads and this one does not: a rhythm ("every monday 9am"), which
//! there makes the thread a repeating task. This window keeps no repeating tasks yet, so the
//! phrase is refused like any other it cannot read.

use super::calendar::Day;
use super::js::{self, group, re};

const MINUTE: f64 = 60_000.0;
const HOUR: f64 = 3_600_000.0;
const DAY: f64 = 86_400_000.0;

/* --------------------------------- the presets --------------------------------- */

/// The moments the picker offers, soonest first (`laterPresets` in `renderer/src/later-times.ts`).
pub fn presets(now: f64) -> Vec<(&'static str, f64)> {
    let today = Day::of(now);
    let mut list = vec![("In 30 minutes", now + 30.0 * MINUTE), ("In 4 hours", now + 4.0 * HOUR)];
    let evening = today.at(18, 0);
    if evening > now {
        list.push(("This evening", evening));
    }
    let tomorrow = Day::of(now + DAY);
    list.push(("Tomorrow", tomorrow.at(8, 0)));
    let mut monday = coming_weekday(now, 1);
    // On a Sunday the coming Monday is tomorrow, already offered.
    if monday == tomorrow {
        monday = monday.plus(7);
    }
    list.push(("Next week", monday.at(8, 0)));
    list.sort_by(|a, b| a.1.total_cmp(&b.1));
    list
}

/* ------------------------------ a time, in words ------------------------------ */

#[derive(Clone, Copy, PartialEq)]
enum Period {
    Morning,
    Afternoon,
    Evening,
    Night,
}

impl Period {
    fn hour(self) -> i64 {
        match self {
            Period::Morning => 8,
            Period::Afternoon => 14,
            Period::Evening => 18,
            Period::Night => 20,
        }
    }
    fn named(word: &str) -> Option<Period> {
        match word {
            "morning" => Some(Period::Morning),
            "afternoon" => Some(Period::Afternoon),
            "evening" => Some(Period::Evening),
            "night" => Some(Period::Night),
            _ => None,
        }
    }
}

fn is_filler(word: &str) -> bool {
    matches!(word, "at" | "on" | "by" | "around" | "about" | "for" | "until" | "till" | "til" | "the" | "of" | "and")
}

fn is_tomorrow(word: &str) -> bool {
    matches!(word, "tomorrow" | "tmrw" | "tmr" | "tom" | "tmw" | "tomorow" | "tommorow" | "tommorrow")
}

// Whole words only: `sat[a-z]*` once read "satisfy the customer" as Saturday. No plurals
// either, because "fridays" is a rhythm.
re!(WEEKDAY, r"^(?:(sun)(?:day)?|(mon)(?:day)?|(tue)(?:s|sday)?|(wed)(?:s|nesday)?|(thu)(?:r|rs|rsday)?|(fri)(?:day)?|(sat)(?:urday)?)$", "");
re!(MONTH, r"^(?:(jan)(?:uary)?|(feb)(?:ruary)?|(mar)(?:ch)?|(apr)(?:il)?|(may)|(jun)e?|(jul)y?|(aug)(?:ust)?|(sep)(?:t|tember)?|(oct)(?:ober)?|(nov)(?:ember)?|(dec)(?:ember)?)$", "");

#[derive(Clone, Copy, PartialEq)]
enum Unit {
    Min,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

fn unit_of(word: &str) -> Option<Unit> {
    Some(match word {
        "m" | "min" | "mins" | "minute" | "minutes" => Unit::Min,
        "h" | "hr" | "hrs" | "hour" | "hours" => Unit::Hour,
        "d" | "day" | "days" => Unit::Day,
        "w" | "wk" | "wks" | "week" | "weeks" => Unit::Week,
        "mo" | "mos" | "month" | "months" => Unit::Month,
        "y" | "yr" | "yrs" | "year" | "years" => Unit::Year,
        _ => return None,
    })
}

const UNITS_WORDS: [&str; 19] = [
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen",
];
const TENS_WORDS: [&str; 8] = ["twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety"];
const ORDINAL_WORDS: [&str; 19] = [
    "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth", "eleventh", "twelfth", "thirteenth", "fourteenth", "fifteenth", "sixteenth", "seventeenth",
    "eighteenth", "nineteenth",
];

re!(CARDINAL, &format!(r"\b(?:({})(?:[\s-]({}))?|({}))\b", TENS_WORDS.join("|"), UNITS_WORDS[..9].join("|"), UNITS_WORDS.join("|")), "");
re!(ORDINAL, &format!(r"\b(?:(twenty|thirty)[\s-])?({})\b|\b(twentieth|thirtieth)\b", ORDINAL_WORDS.join("|")), "");

re!(END_MARKS, r"[.!?]+$", "");
re!(SPACES, r"\s+", "");
re!(HALF_HOUR, r"\b(?:half an? (?:hour|hr)|an? half (?:hour|hr)|half (?:hour|hr))\b", "");
re!(DAY_AFTER, r"\b(?:the )?day after (?:tomorrow|tmrw)\b", "");
re!(END_OF_DAY, r"\b(?:end of (?:the )?(?:day|today)|end of business|close of business|eod|eob|cob)\b", "");
re!(END_OF_WEEK, r"\b(?:end of (?:the )?week|eow)\b", "");
re!(FIRST_THING, r"\bfirst thing(?: in the morning)?\b", "");
re!(IN_THE, r"\bin the (morning|afternoon|evening)\b", "");
re!(AT_NIGHT, r"\bat night\b", "");
re!(OCLOCK, r"\s*o'?clock\b", "");
re!(FROM_NOW, r"\bfrom now\b", "");
re!(COUPLE, r"\b(?:a )?couple(?: of)?\b", "");
re!(FEW, r"\b(?:a )?few\b", "");
re!(NOON_WORDS, r"\b(?:midday|lunch ?time|lunch)\b", "");
re!(UNIT_ALONE, r"^(in )?(min|minute|hr|hour|day|wk|week|month|yr|year)$", "");
re!(AND_A_HALF_OF, r"\b(\d+|an?) ([a-z]+) and a half\b", "");
re!(AND_A_HALF, r"\b(\d+) and a half\b", "");
re!(MERIDIEM, r"(\d)\s*([ap])\.?(?:\s?m\.?)?(?=\s|$)", "");
re!(HALF_PAST, r"\bhalf past (\d{1,2})(am|pm)?\b", "");
re!(QUARTER_PAST, r"\bquarter past (\d{1,2})(am|pm)?\b", "");
re!(QUARTER_TO, r"\bquarter to (\d{1,2})(am|pm)?\b", "");
re!(SPACED_CLOCK, r"\b(\d{1,2}) ([0-5]\d)(am|pm)\b", "");
re!(AT_SPACED_CLOCK, r"\bat (\d{1,2}) ([0-5]\d)\b", "");

/// The place of a word in one of the lists above, as a number.
fn place(list: &[&str], word: &str) -> i64 {
    list.iter().position(|w| *w == word).map_or(-1, |at| at as i64)
}

/// The words of a phrase, spelled one way, ready to be spent (`normalise`).
fn normalise(raw: &str) -> Vec<String> {
    let lower = raw.to_lowercase();
    let mut t = END_MARKS.strip(js::trim(&lower)).replace(',', " ").replace('@', " at ");
    t = SPACES.replace_all(&t, |_| " ".into());

    // Phrases that mean one thing, said as several words.
    t = HALF_HOUR.replace_all(&t, |_| "30 minutes".into());
    t = DAY_AFTER.replace_all(&t, |_| "overmorrow".into());
    t = END_OF_DAY.replace_all(&t, |_| "eod".into());
    t = END_OF_WEEK.replace_all(&t, |_| "eow".into());
    t = FIRST_THING.replace_all(&t, |_| "morning".into());
    t = IN_THE.replace_all(&t, |c| group(c, 1).into());
    t = AT_NIGHT.replace_all(&t, |_| "night".into());
    t = OCLOCK.strip_all(&t);
    t = FROM_NOW.strip_all(&t);
    t = COUPLE.replace_all(&t, |_| "2".into());
    t = FEW.replace_all(&t, |_| "3".into());
    t = NOON_WORDS.replace_all(&t, |_| "noon".into());
    // A unit said alone is one of it: "week" is a week. The whole phrase only, so "this week"
    // stays refused and "next week" stays the coming Monday.
    t = UNIT_ALONE.replace(&t, |c| format!("{}1 {}", group(c, 1), group(c, 2)));

    // Numbers spelled the way they are said. Ordinals first, so "twenty first" is the 21st
    // rather than 20 and a stray "first".
    t = ORDINAL.replace_all(&t, |c| {
        if let Some(round) = c.get(3) {
            return if round == "twentieth" { "20th".into() } else { "30th".into() };
        }
        let tens = match group(c, 1) {
            "twenty" => 20,
            "thirty" => 30,
            _ => 0,
        };
        format!("{}th", tens + place(&ORDINAL_WORDS, group(c, 2)) + 1)
    });
    t = CARDINAL.replace_all(&t, |c| {
        if let Some(small) = c.get(3) {
            return (place(&UNITS_WORDS, small) + 1).to_string();
        }
        let ones = c.get(2).map_or(0, |unit| place(&UNITS_WORDS, unit) + 1);
        ((place(&TENS_WORDS, group(c, 1)) + 2) * 10 + ones).to_string()
    });

    // "an hour and a half", "2 and a half hours".
    t = AND_A_HALF_OF.replace_all(&t, |c| {
        let n = group(c, 1);
        format!("{}.5 {}", if n.starts_with(|d: char| d.is_ascii_digit()) { n } else { "1" }, group(c, 2))
    });
    t = AND_A_HALF.replace_all(&t, |c| format!("{}.5", group(c, 1)));

    // am and pm, glued to their number: "3 p.m.", "3 pm" and "3p" are all "3pm".
    t = MERIDIEM.replace_all(&t, |c| format!("{}{}m", group(c, 1), group(c, 2)));

    // Clocks said in words.
    t = HALF_PAST.replace_all(&t, |c| format!("{}:30{}", group(c, 1), group(c, 2)));
    t = QUARTER_PAST.replace_all(&t, |c| format!("{}:15{}", group(c, 1), group(c, 2)));
    t = QUARTER_TO.replace_all(&t, |c| {
        let before = group(c, 1).parse::<i64>().unwrap_or(0) - 1;
        format!("{}:45{}", if before == 0 { 12 } else { before }, group(c, 2))
    });
    t = SPACED_CLOCK.replace_all(&t, |c| format!("{}:{}{}", group(c, 1), group(c, 2), group(c, 3)));
    t = AT_SPACED_CLOCK.replace_all(&t, |c| format!("at {}:{}", group(c, 1), group(c, 2)));

    t.split(' ').filter(|w| !w.is_empty()).map(str::to_string).collect()
}

#[derive(Clone, Copy)]
struct Clock {
    h: i64,
    m: i64,
    /// Whether it said pm, when it said either.
    pm: Option<bool>,
    sure: bool,
}

struct Said {
    any: bool,
    date: Option<Day>,
    /// Days to add when the moment has already gone. 0 refuses instead.
    roll: i64,
    exact: Option<f64>,
    clock: Option<Clock>,
    period: Option<Period>,
    fallback: Option<i64>,
    midnight: bool,
    need_duration: bool,
    this_day: bool,
}

/// Always the NEXT one: "mon" on a Monday means the Monday coming, not today.
fn coming_weekday(now: f64, day: i64) -> Day {
    let today = Day::of(now);
    let ahead = (day - today.weekday() + 7) % 7;
    today.plus(if ahead == 0 { 7 } else { ahead })
}

/// Which of a regex's groups matched: the day of the week, or the month, counted from zero.
fn group_index(c: &js::Captures, groups: usize) -> i64 {
    (1..=groups).position(|n| c.get(n).is_some()).map_or(-1, |at| at as i64)
}

fn set_date(s: &mut Said, d: Option<Day>, roll: i64) -> bool {
    let (Some(d), None) = (d, s.date) else {
        return false;
    };
    s.date = Some(d);
    s.roll = roll;
    true
}

fn set_clock(s: &mut Said, c: Option<Clock>) -> bool {
    let Some(c) = c else {
        return false;
    };
    if s.clock.is_some() || s.fallback.is_some() {
        return false;
    }
    s.clock = Some(c);
    true
}

re!(CLOCK, r"^(\d{1,2})(?:[:.](\d{2}))?(am|pm)?$", "");

fn clock_of(word: &str) -> Option<Clock> {
    let c = CLOCK.exec(word)?;
    let h = group(&c, 1).parse::<i64>().ok()?;
    let m = c.get(2).map_or(Some(0), |m| m.parse::<i64>().ok())?;
    if m > 59 {
        return None;
    }
    if let Some(mer) = c.get(3) {
        return (1..=12).contains(&h).then_some(Clock { h, m, pm: Some(mer == "pm"), sure: true });
    }
    if h > 23 {
        return None;
    }
    // "09:00", "0:30" and "18:30" say exactly which hour they mean.
    let hours = group(&c, 1);
    let sure = h == 0 || h >= 13 || (hours.len() == 2 && hours.starts_with('0'));
    Some(Clock { h, m, pm: None, sure })
}

re!(GLUED, r"^(\d+)h(\d{1,2})m?$", "");
re!(COUNT_AND_UNIT, r"^(\d+(?:\.\d+)?)([a-z]+)$", "");
re!(NUMBER, r"^\d+(?:\.\d+)?$", "");

/// One count of one unit at `w[j]`: how many, of what, and the words it took.
fn part_at(w: &[String], j: usize) -> Option<(f64, Unit, usize)> {
    let word = w.get(j)?;
    if let Some(c) = GLUED.exec(word) {
        return Some((group(&c, 1).parse::<f64>().ok()? + group(&c, 2).parse::<f64>().ok()? / 60.0, Unit::Hour, 1));
    }
    if let Some(c) = COUNT_AND_UNIT.exec(word) {
        if let Some(unit) = unit_of(group(&c, 2)) {
            return Some((group(&c, 1).parse().ok()?, unit, 1));
        }
    }
    let count = if NUMBER.test(word) {
        word.parse::<f64>().ok()
    } else if word == "a" || word == "an" {
        Some(1.0)
    } else {
        None
    };
    let unit = unit_of(w.get(j + 1)?)?;
    Some((count?, unit, 2))
}

struct Span {
    used: usize,
    ms: f64,
    days: i64,
    months: i64,
    clocked: bool,
}

/// A stretch of time from here: "2 hours 30 minutes", "1h30", "a week".
fn duration_at(w: &[String], i: usize) -> Option<Span> {
    let mut span = Span { used: 0, ms: 0.0, days: 0, months: 0, clocked: false };
    let mut any = false;
    loop {
        let mut j = i + span.used;
        if any && w.get(j).is_some_and(|word| word == "and") {
            j += 1;
        }
        let Some((n, unit, took)) = part_at(w, j) else {
            break;
        };
        if n.fract() != 0.0 && unit != Unit::Min && unit != Unit::Hour {
            return None;
        }
        match unit {
            Unit::Min => span.ms += n * MINUTE,
            Unit::Hour => span.ms += n * HOUR,
            Unit::Day => span.days += n as i64,
            Unit::Week => span.days += n as i64 * 7,
            Unit::Month => span.months += n as i64,
            Unit::Year => span.months += n as i64 * 12,
        }
        span.clocked |= unit == Unit::Min || unit == Unit::Hour;
        span.used = j - i + took;
        any = true;
    }
    any.then_some(span)
}

re!(DAY_NUMBER, r"^(\d{1,2})(?:st|nd|rd|th)?$", "");
re!(YEAR, r"^\d{4}$", "");

/// "oct 10", "10 oct", "the 10th of october", each with an optional year. The words it took,
/// and the day when the calendar has one.
fn month_date_at(w: &[String], i: usize, now: f64) -> Option<(usize, Option<Day>)> {
    let day_num = |word: Option<&String>| word.and_then(|word| DAY_NUMBER.exec(word)).and_then(|c| group(&c, 1).parse::<i64>().ok());
    let (month, day, mut used);
    let first = MONTH.exec(&w[i]);
    if let (Some(first), Some(d)) = (&first, day_num(w.get(i + 1))) {
        (month, day, used) = (group_index(first, 12), d, 2);
    } else if let Some(d) = day_num(w.get(i)) {
        let of = usize::from(w.get(i + 1).is_some_and(|word| word == "of"));
        let after = w.get(i + 1 + of).and_then(|word| MONTH.exec(word))?;
        (month, day, used) = (group_index(&after, 12), d, 2 + of);
    } else {
        return None;
    }
    let year = w.get(i + used).filter(|word| YEAR.test(word)).and_then(|word| word.parse::<i64>().ok());
    if year.is_some() {
        used += 1;
    }
    let today = Day::of(now);
    let mut date = Day::real(year.unwrap_or(today.ymd().0), month, day);
    // No year said, and the day has gone this year: it is next year's.
    if year.is_none() && date.is_some_and(|d| d < today) {
        date = Day::real(today.ymd().0 + 1, month, day);
    }
    Some((used, date))
}

re!(ORDINAL_DAY, r"^(\d{1,2})(?:st|nd|rd|th)$", "");
re!(ISO_DATE, r"^(\d{4})-(\d{1,2})-(\d{1,2})$", "");
re!(SLASH_DATE, r"^(\d{1,2})/(\d{1,2})(?:/(\d{2}|\d{4}))?$", "");

/// One component at `w[i]`. How many words it spent, or 0 if none fits.
fn step(w: &[String], i: usize, s: &mut Said, now: f64) -> usize {
    let word = w[i].as_str();
    let next = w.get(i + 1).map(String::as_str);
    let spent = |ok: bool, words: usize| if ok { words } else { 0 };

    if word == "in" {
        s.need_duration = true;
        return 1;
    }
    if word == "this" || word == "coming" {
        s.this_day = true;
        return 1;
    }
    if is_filler(word) {
        return 1;
    }

    if let Some(span) = duration_at(w, i) {
        s.need_duration = false;
        s.any = true;
        // Hours and minutes are an exact stretch from now; days and longer land on a day, at
        // a time of their own.
        if span.clocked {
            if s.exact.is_some() || span.months != 0 {
                return 0;
            }
            s.exact = Some(span.ms + span.days as f64 * DAY);
            return span.used;
        }
        return spent(set_date(s, Some(Day::of(now).plus(span.days).plus_months(span.months)), 0), span.used);
    }
    if s.need_duration {
        return 0;
    }
    s.any = true;

    let today = Day::of(now);
    if word == "today" {
        return spent(set_date(s, Some(today), 0), 1);
    }
    if is_tomorrow(word) {
        return spent(set_date(s, Some(today.plus(1)), 0), 1);
    }
    if word == "overmorrow" {
        return spent(set_date(s, Some(today.plus(2)), 0), 1);
    }
    if word == "tonight" {
        if s.period.is_some() {
            return 0;
        }
        s.period = Some(Period::Evening);
        return 1;
    }
    if let Some(period) = Period::named(word) {
        if s.period.is_some() {
            return 0;
        }
        s.period = Some(period);
        return 1;
    }

    if word == "next" {
        if let Some(day) = next.and_then(|n| WEEKDAY.exec(n)) {
            return spent(set_date(s, Some(coming_weekday(now, group_index(&day, 7))), 0), 2);
        }
        return match next {
            Some("week") => spent(set_date(s, Some(coming_weekday(now, 1)), 0), 2),
            Some("weekend") => spent(set_date(s, Some(coming_weekday(now, 6)), 0), 2),
            Some("month") => {
                let (y, m, _) = today.ymd();
                spent(set_date(s, Some(Day::first_of(y, m + 1)), 0), 2)
            }
            _ => 0,
        };
    }

    if let Some(day) = WEEKDAY.exec(word) {
        let target = group_index(&day, 7);
        // "this friday" on a Friday is today, while it is still ahead.
        if s.this_day && target == today.weekday() {
            return spent(set_date(s, Some(today), 7), 1);
        }
        return spent(set_date(s, Some(coming_weekday(now, target)), 0), 1);
    }
    if word == "weekend" {
        let saturday = today.weekday() == 6;
        return spent(set_date(s, Some(if saturday { today.plus(1) } else { coming_weekday(now, 6) }), 0), 1);
    }
    if word == "eow" {
        if s.fallback.is_some() || s.clock.is_some() {
            return 0;
        }
        s.fallback = Some(17);
        return spent(set_date(s, Some(today.plus((5 - today.weekday() + 7) % 7)), 7), 1);
    }
    if word == "eod" {
        if s.fallback.is_some() || s.clock.is_some() {
            return 0;
        }
        s.fallback = Some(17);
        return 1;
    }
    if word == "noon" {
        return spent(set_clock(s, Some(Clock { h: 12, m: 0, pm: Some(true), sure: true })), 1);
    }
    if word == "midnight" {
        s.midnight = true;
        return spent(set_clock(s, Some(Clock { h: 0, m: 0, pm: None, sure: true })), 1);
    }

    if let Some((used, date)) = month_date_at(w, i, now) {
        return spent(set_date(s, date, 0), used);
    }

    if let Some(c) = ORDINAL_DAY.exec(word) {
        // "the 15th": this month's, unless it has gone, then next month's. In December the web
        // app asks for a thirteenth month and so refuses; this refuses with it.
        let n = group(&c, 1).parse::<i64>().unwrap_or(0);
        let (y, m, d) = today.ymd();
        return spent(set_date(s, Day::real(y, if n < d { m + 1 } else { m }, n), 0), 1);
    }

    if let Some(c) = ISO_DATE.exec(word) {
        let part = |n: usize| group(&c, n).parse::<i64>().unwrap_or(0);
        return spent(set_date(s, Day::real(part(1), part(2) - 1, part(3)), 0), 1);
    }

    // Month first, the way it is written where this app is used.
    if let Some(c) = SLASH_DATE.exec(word) {
        let part = |n: usize| group(&c, n).parse::<i64>().unwrap_or(0);
        let said = c.get(3);
        let y = match said {
            Some(two) if two.len() == 2 => 2000 + part(3),
            Some(_) => part(3),
            None => today.ymd().0,
        };
        let mut date = Day::real(y, part(1) - 1, part(2));
        if said.is_none() && date.is_some_and(|d| d < today) {
            date = Day::real(y + 1, part(1) - 1, part(2));
        }
        return spent(set_date(s, date, 0), 1);
    }

    spent(set_clock(s, clock_of(word)), 1)
}

/// The hour a clock with no am or pm means, given what else was said.
fn hour_of(c: Clock, s: &Said, now: f64) -> i64 {
    if c.sure {
        return match c.pm {
            Some(true) => c.h % 12 + 12,
            Some(false) => c.h % 12,
            None => c.h,
        };
    }
    if c.h == 12 {
        return 12;
    }
    if let Some(period) = s.period {
        return if period == Period::Morning { c.h } else { c.h + 12 };
    }
    if c.h <= 6 {
        return c.h + 12;
    }
    if s.date.is_some() {
        return c.h;
    }
    if Day::of(now).at(c.h, c.m) > now { c.h } else { c.h + 12 }
}

/// The moment a typed phrase names, or None if it names none still ahead (`readMoment`).
pub fn read_moment(raw: &str, now: f64) -> Option<f64> {
    let w = normalise(raw);
    if w.is_empty() {
        return None;
    }
    let mut s = Said { any: false, date: None, roll: 1, exact: None, clock: None, period: None, fallback: None, midnight: false, need_duration: false, this_day: false };
    let mut i = 0;
    while i < w.len() {
        let used = step(&w, i, &mut s, now);
        if used == 0 {
            return None;
        }
        i += used;
    }
    if !s.any || s.need_duration {
        return None;
    }

    if let Some(exact) = s.exact {
        if s.date.is_some() || s.clock.is_some() || s.period.is_some() || s.fallback.is_some() {
            return None;
        }
        let ts = now + exact;
        return (ts > now).then_some(ts);
    }

    let mut day = s.date.unwrap_or(Day::of(now));
    let roll = if s.date.is_some() { s.roll } else { 1 };
    let (hour, minute) = match (s.clock, s.period) {
        (Some(c), _) => (hour_of(c, &s, now), c.m),
        (None, Some(period)) => (period.hour(), 0),
        (None, None) => (s.fallback.unwrap_or(8), 0),
    };
    // Midnight on a named day is the end of that day.
    if s.midnight && s.date.is_some() {
        day = day.plus(1);
    }
    if day.at(hour, minute) <= now && roll != 0 {
        day = day.plus(roll);
    }
    let ts = day.at(hour, minute);
    (ts > now).then_some(ts)
}

/// A clock as the window writes one: "8:00 AM".
fn clock_words(ts: f64) -> String {
    let t = rune::time::LocalTime::at(ts as i64);
    let hour = if t.hour.is_multiple_of(12) { 12 } else { t.hour % 12 };
    format!("{hour}:{:02} {}", t.minute, if t.hour < 12 { "AM" } else { "PM" })
}

const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// A typed time and the words for it: "today, 3:00 PM", "tomorrow, 8:00 AM", "Fri, Oct 9,
/// 3:00 PM" (`parseWhen` and `stamp2` in `renderer/src/format.ts`). The web app writes the day
/// in the system's language; this writes the English the rest of the window speaks.
pub fn parse_when(raw: &str, now: f64) -> Option<(f64, String)> {
    let ts = read_moment(raw, now)?;
    let day = Day::of(ts);
    let named = match day.0 - Day::of(now).0 {
        0 => "today".to_string(),
        1 => "tomorrow".to_string(),
        _ => {
            let (_, month, date) = day.ymd();
            format!("{}, {} {date}", DAYS[day.weekday() as usize], MONTHS[month as usize])
        }
    };
    Some((ts, format!("{named}, {}", clock_words(ts))))
}

/* ------------------------------- what the box says ------------------------------ */

/// What Enter means in the picker's box (`enterMeans`). Typed words always decide: words that
/// could not be read are a refusal, never a fall through to whichever row is lit.
#[derive(PartialEq, Debug)]
pub enum Enter {
    Typed,
    Refuse,
    Selected,
}

pub fn enter_means(text: &str, read: bool) -> Enter {
    if js::trim(text).is_empty() {
        Enter::Selected
    } else if read {
        Enter::Typed
    } else {
        Enter::Refuse
    }
}

/// The line beside the box: the time it read, "not a time" while the words are unreadable,
/// and after a refused Enter the same words with a hint (`previewLine` in
/// `renderer/src/components/SchedulePicker.tsx`). The second part is whether it was read.
pub fn preview_line(text: &str, read: Option<&str>, refused: bool, hint: &str) -> Option<(String, bool)> {
    if js::trim(text).is_empty() {
        return None;
    }
    Some(match read {
        Some(words) => (words.to_string(), true),
        None if refused => (format!("not a time · {hint}"), false),
        None => ("not a time".to_string(), false),
    })
}

/// What the box asks for while it is empty, and what a refusal suggests. The web's box also
/// names a rhythm on one thread; this one names only what it reads.
pub const PLACEHOLDER: &str = "Try: 30m, 3h, 8am, tomorrow, next week";
pub const HINT_WHEN_REFUSED: &str = "try 3h, 8am, tomorrow";

#[cfg(test)]
mod tests {
    use super::*;

    /// A wall-clock moment on this machine, months counted from one as people say them.
    fn at(y: i64, month: i64, d: i64, h: i64, min: i64) -> f64 {
        Day::real(y, month - 1, d).expect("a real day").at(h, min)
    }

    /// 2026-10-04 is a Sunday; the web's tests read every phrase at 12:05 that day
    /// (`tests/the-schedule-box-reads-what-people-actually-type.test.mts`).
    fn now() -> f64 {
        at(2026, 10, 4, 12, 5)
    }

    fn reads(cases: &[(&str, f64)]) {
        for (text, expected) in cases {
            assert_eq!(read_moment(text, now()), Some(*expected), "\"{text}\"");
        }
    }

    #[test]
    fn a_stretch_of_days_or_weeks_lands_on_a_day_at_eight() {
        reads(&[
            ("one week", at(2026, 10, 11, 8, 0)),
            ("a week", at(2026, 10, 11, 8, 0)),
            ("1 week", at(2026, 10, 11, 8, 0)),
            ("1w", at(2026, 10, 11, 8, 0)),
            ("in a week", at(2026, 10, 11, 8, 0)),
            ("in one week", at(2026, 10, 11, 8, 0)),
            ("a week from now", at(2026, 10, 11, 8, 0)),
            ("week", at(2026, 10, 11, 8, 0)),
            ("2 weeks", at(2026, 10, 18, 8, 0)),
            ("in two weeks", at(2026, 10, 18, 8, 0)),
            ("2 wks", at(2026, 10, 18, 8, 0)),
            ("in a month", at(2026, 11, 4, 8, 0)),
            ("in 3 days at 5pm", at(2026, 10, 7, 17, 0)),
            ("in two days", at(2026, 10, 6, 8, 0)),
        ]);
    }

    #[test]
    fn a_stretch_of_hours_or_minutes_is_exact_from_now() {
        let now = now();
        reads(&[
            ("a couple of hours", now + 2.0 * HOUR),
            ("couple hours", now + 2.0 * HOUR),
            ("a few hours", now + 3.0 * HOUR),
            ("an hour and a half", now + 90.0 * MINUTE),
            ("1.5 hours", now + 90.0 * MINUTE),
            ("90 min", now + 90.0 * MINUTE),
            ("1h30", now + 90.0 * MINUTE),
            ("1h 30m", now + 90.0 * MINUTE),
            ("2 hours 30 minutes", now + 150.0 * MINUTE),
            ("2 and a half hours", now + 150.0 * MINUTE),
            ("a half hour", now + 30.0 * MINUTE),
            ("45 minutes from now", now + 45.0 * MINUTE),
            ("in 20 mins", now + 20.0 * MINUTE),
            ("twenty five minutes", now + 25.0 * MINUTE),
            ("in forty-five mins", now + 45.0 * MINUTE),
            ("in 2 hrs", now + 2.0 * HOUR),
            ("30m", now + 30.0 * MINUTE),
            ("3h", now + 3.0 * HOUR),
        ]);
    }

    #[test]
    fn a_clock_is_read_in_all_its_spellings() {
        reads(&[
            ("at 3pm", at(2026, 10, 4, 15, 0)),
            ("3 pm", at(2026, 10, 4, 15, 0)),
            ("3p", at(2026, 10, 4, 15, 0)),
            ("3 p.m.", at(2026, 10, 4, 15, 0)),
            ("at 3", at(2026, 10, 4, 15, 0)),
            ("3:30", at(2026, 10, 4, 15, 30)),
            ("15:30", at(2026, 10, 4, 15, 30)),
            ("half past 4", at(2026, 10, 4, 16, 30)),
            ("quarter past 4", at(2026, 10, 4, 16, 15)),
            ("quarter to 5", at(2026, 10, 4, 16, 45)),
            ("six thirty pm", at(2026, 10, 4, 18, 30)),
            ("5 o'clock", at(2026, 10, 4, 17, 0)),
            // 12:05 has gone, so noon is tomorrow's.
            ("noon", at(2026, 10, 5, 12, 0)),
            ("midnight", at(2026, 10, 5, 0, 0)),
            ("eod", at(2026, 10, 4, 17, 0)),
            ("end of day", at(2026, 10, 4, 17, 0)),
            ("end of the day", at(2026, 10, 4, 17, 0)),
            ("cob", at(2026, 10, 4, 17, 0)),
            // Eight has gone today, so it is tomorrow's.
            ("8am", at(2026, 10, 5, 8, 0)),
        ]);
    }

    #[test]
    fn a_day_takes_a_time_in_either_order() {
        reads(&[
            ("tomorrow at noon", at(2026, 10, 5, 12, 0)),
            ("noon tomorrow", at(2026, 10, 5, 12, 0)),
            ("3pm tomorrow", at(2026, 10, 5, 15, 0)),
            ("3 pm tomorrow", at(2026, 10, 5, 15, 0)),
            ("tomorrow at 3", at(2026, 10, 5, 15, 0)),
            ("tomorrow at 9", at(2026, 10, 5, 9, 0)),
            ("tomorrow afternoon", at(2026, 10, 5, 14, 0)),
            ("tomorrow evening", at(2026, 10, 5, 18, 0)),
            ("tomorrow night", at(2026, 10, 5, 20, 0)),
            ("tomorrow morning at 9", at(2026, 10, 5, 9, 0)),
            ("tomorrow evening at 7", at(2026, 10, 5, 19, 0)),
            ("tomorrow eod", at(2026, 10, 5, 17, 0)),
            ("this afternoon", at(2026, 10, 4, 14, 0)),
            ("tonight at 9", at(2026, 10, 4, 21, 0)),
            ("this evening at 7:30", at(2026, 10, 4, 19, 30)),
            ("today at 12:06", at(2026, 10, 4, 12, 6)),
            ("day after tomorrow", at(2026, 10, 6, 8, 0)),
            ("the day after tomorrow at 10am", at(2026, 10, 6, 10, 0)),
            ("Tomorrow at 3pm.", at(2026, 10, 5, 15, 0)),
            ("until tomorrow", at(2026, 10, 5, 8, 0)),
            ("tomorrow", at(2026, 10, 5, 8, 0)),
        ]);
    }

    #[test]
    fn a_weekday_is_the_one_coming() {
        reads(&[
            ("friday", at(2026, 10, 9, 8, 0)),
            ("on friday", at(2026, 10, 9, 8, 0)),
            ("this friday", at(2026, 10, 9, 8, 0)),
            ("next friday", at(2026, 10, 9, 8, 0)),
            ("friday at 3pm", at(2026, 10, 9, 15, 0)),
            ("3pm friday", at(2026, 10, 9, 15, 0)),
            ("3pm on friday", at(2026, 10, 9, 15, 0)),
            ("Friday, 3pm", at(2026, 10, 9, 15, 0)),
            ("friday afternoon", at(2026, 10, 9, 14, 0)),
            ("friday morning", at(2026, 10, 9, 8, 0)),
            ("fri @ 9am", at(2026, 10, 9, 9, 0)),
            ("thurs 10am", at(2026, 10, 8, 10, 0)),
            ("tues", at(2026, 10, 6, 8, 0)),
            ("weds at 4", at(2026, 10, 7, 16, 0)),
            ("monday at 9", at(2026, 10, 5, 9, 0)),
            ("next week", at(2026, 10, 5, 8, 0)),
            ("next week at 3pm", at(2026, 10, 5, 15, 0)),
            ("this weekend", at(2026, 10, 10, 8, 0)),
            ("weekend", at(2026, 10, 10, 8, 0)),
            ("end of week", at(2026, 10, 9, 17, 0)),
            ("eow", at(2026, 10, 9, 17, 0)),
            ("next month", at(2026, 11, 1, 8, 0)),
        ]);
    }

    #[test]
    fn a_date_on_the_calendar_is_read() {
        reads(&[
            ("oct 10", at(2026, 10, 10, 8, 0)),
            ("october 10th", at(2026, 10, 10, 8, 0)),
            ("10 oct", at(2026, 10, 10, 8, 0)),
            ("the 10th of october", at(2026, 10, 10, 8, 0)),
            ("Oct 10 at 2pm", at(2026, 10, 10, 14, 0)),
            ("october 10, 2026", at(2026, 10, 10, 8, 0)),
            ("10/10", at(2026, 10, 10, 8, 0)),
            ("10/10/2026", at(2026, 10, 10, 8, 0)),
            ("2026-10-10", at(2026, 10, 10, 8, 0)),
            ("2026-10-10 14:00", at(2026, 10, 10, 14, 0)),
            ("the 15th", at(2026, 10, 15, 8, 0)),
            ("on the 15th at 9am", at(2026, 10, 15, 9, 0)),
            // The 2nd of this month has gone, so it is next month's.
            ("the 2nd", at(2026, 11, 2, 8, 0)),
            // January has gone this year, so it is next year's.
            ("jan 5", at(2027, 1, 5, 8, 0)),
            ("Dec 24 at 6pm", at(2026, 12, 24, 18, 0)),
            ("oct 4 at 5pm", at(2026, 10, 4, 17, 0)),
        ]);
    }

    #[test]
    fn what_it_cannot_read_or_has_gone_is_refused() {
        let refused = [
            "after my meeting",
            "whenever",
            "sometime",
            "soon",
            "later",
            "tomorrow sometime",
            "tomorrowland",
            "todays standup",
            "satisfy the customer",
            "friday or saturday",
            // A bare number after "in" has no unit, and guessing minutes is a guess.
            "in 5",
            "in 30",
            "yesterday",
            "last friday",
            "13pm",
            "25:00",
            "9:75",
            "feb 30",
            "oct 32",
            // Already gone: today's hour, today's date, and this very minute.
            "today at 9am",
            "noon today",
            "oct 4 at 9am",
            "today at 12:05",
            "in 0 minutes",
            // A stretch of hours cannot also carry a clock or a day.
            "3 hours at 5pm",
            "2 hours tomorrow",
            // A rhythm is not a moment.
            "every friday",
            "fridays",
            "every day at 9am",
            "99",
            "",
        ];
        for text in refused {
            assert_eq!(read_moment(text, now()), None, "\"{text}\"");
        }
    }

    #[test]
    fn a_read_time_is_said_as_a_day_and_a_clock() {
        let now = now();
        assert_eq!(parse_when("3pm", now), Some((at(2026, 10, 4, 15, 0), "today, 3:00 PM".to_string())));
        assert_eq!(parse_when("tomorrow", now), Some((at(2026, 10, 5, 8, 0), "tomorrow, 8:00 AM".to_string())));
        assert_eq!(parse_when("friday at 3pm", now), Some((at(2026, 10, 9, 15, 0), "Fri, Oct 9, 3:00 PM".to_string())));
        assert_eq!(parse_when("noon tomorrow", now).map(|m| m.1), Some("tomorrow, 12:00 PM".to_string()));
        assert_eq!(parse_when("whenever", now), None);
    }

    #[test]
    fn the_picker_offers_the_coming_moments_soonest_first() {
        // A Sunday just after noon: this evening is still ahead, and Monday is tomorrow, so
        // next week is the Monday after.
        let now = now();
        assert_eq!(
            presets(now),
            vec![
                ("In 30 minutes", now + 30.0 * MINUTE),
                ("In 4 hours", now + 4.0 * HOUR),
                ("This evening", at(2026, 10, 4, 18, 0)),
                ("Tomorrow", at(2026, 10, 5, 8, 0)),
                ("Next week", at(2026, 10, 12, 8, 0)),
            ]
        );
        // A Wednesday night: the evening has gone, four hours on is past tomorrow's eight.
        let late = at(2026, 10, 7, 23, 0);
        let names: Vec<&str> = presets(late).iter().map(|p| p.0).collect();
        assert_eq!(names, ["In 30 minutes", "In 4 hours", "Tomorrow", "Next week"]);
        assert_eq!(presets(late)[3].1, at(2026, 10, 12, 8, 0));
    }

    #[test]
    fn typed_words_decide_what_enter_means() {
        assert_eq!(enter_means("", false), Enter::Selected);
        assert_eq!(enter_means("   ", false), Enter::Selected);
        assert_eq!(enter_means("tomorrow", true), Enter::Typed);
        assert_eq!(enter_means("whenever", false), Enter::Refuse);
    }

    #[test]
    fn the_line_beside_the_box_says_what_was_read() {
        assert_eq!(preview_line("", None, false, HINT_WHEN_REFUSED), None);
        assert_eq!(preview_line("3h", Some("SUN 3:05 PM"), false, HINT_WHEN_REFUSED), Some(("SUN 3:05 PM".to_string(), true)));
        assert_eq!(preview_line("whenever", None, false, HINT_WHEN_REFUSED), Some(("not a time".to_string(), false)));
        assert_eq!(preview_line("whenever", None, true, HINT_WHEN_REFUSED), Some(("not a time · try 3h, 8am, tomorrow".to_string(), false)));
    }
}
