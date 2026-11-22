//! A day on this machine's calendar and the instant a clock on it names: what JavaScript's
//! `Date` does for the web app's rules when they say `setHours`, `setDate` and `getDay`.
//!
//! The day is counted, not spelled: adding to it is adding days, and a month or a year is
//! worked out only when a rule asks for one.

use rune::time::LocalTime;

/// A calendar day on this machine's clock, counted from 1 January 1970.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Day(pub i64);

/// Seconds east of Greenwich at an instant.
fn offset(ms: f64) -> i64 {
    i64::from(LocalTime::at(ms as i64).offset)
}

/// How many days a month has. `month` counts from zero, as JavaScript's does.
pub fn days_in(year: i64, month: i64) -> i64 {
    Day::first_of(year, month + 1).0 - Day::first_of(year, month).0
}

impl Day {
    /// The day an instant falls on (`new Date(ms)` with its clock set to midnight).
    pub fn of(ms: f64) -> Day {
        Day(((ms / 1000.0).floor() as i64 + offset(ms)).div_euclid(86_400))
    }

    /// The first of a month. A month past December or before January runs into the years
    /// beside it, as `new Date(y, m, 1)` lets it.
    pub fn first_of(year: i64, month: i64) -> Day {
        let (y, m) = (year + month.div_euclid(12), month.rem_euclid(12) + 1);
        // Days from the civil calendar, counting years from March so the leap day is last.
        let y = if m <= 2 { y - 1 } else { y };
        let era = y.div_euclid(400);
        let of_era = y.rem_euclid(400);
        let of_year = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5;
        let day_of_era = of_era * 365 + of_era / 4 - of_era / 100 + of_year;
        Day(era * 146_097 + day_of_era - 719_468)
    }

    /// A day that is on the calendar, or None: there is no 30 February and no thirteenth
    /// month (`calendarDay` in `renderer/src/when-words.ts`). `month` counts from zero.
    pub fn real(year: i64, month: i64, day: i64) -> Option<Day> {
        ((0..12).contains(&month) && day >= 1 && day <= days_in(year, month)).then(|| Day(Day::first_of(year, month).0 + day - 1))
    }

    /// The year, the month counted from zero, and the day of the month.
    pub fn ymd(self) -> (i64, i64, i64) {
        let z = self.0 + 719_468;
        let era = z.div_euclid(146_097);
        let of_era = z.rem_euclid(146_097);
        let year_of_era = (of_era - of_era / 1460 + of_era / 36_524 - of_era / 146_096) / 365;
        let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let mp = (5 * of_year + 2) / 153;
        let day = of_year - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = year_of_era + era * 400 + i64::from(month <= 2);
        (year, month - 1, day)
    }

    /// The day of the week, Sunday being zero (`getDay`).
    pub fn weekday(self) -> i64 {
        (self.0 + 4).rem_euclid(7)
    }

    pub fn plus(self, days: i64) -> Day {
        Day(self.0 + days)
    }

    /// The same day of the month so many months on, or that month's last day when it is
    /// shorter (`addMonths` in `renderer/src/when-words.ts`).
    pub fn plus_months(self, months: i64) -> Day {
        let (y, m, d) = self.ymd();
        let first = Day::first_of(y, m + months);
        let (y, m, _) = first.ymd();
        Day(first.0 + d.min(days_in(y, m)) - 1)
    }

    /// The instant this day's clock reads `hour:minute` (`setHours(hour, minute, 0, 0)`).
    pub fn at(self, hour: i64, minute: i64) -> f64 {
        let wall = self.0 * 86_400 + hour * 3600 + minute * 60;
        // The offset is asked at the instant itself, which is only known once it is. Twice
        // round settles it on any day but the hour a clock change skips.
        let mut ms = (wall - offset(wall as f64 * 1000.0)) as f64 * 1000.0;
        ms = (wall - offset(ms)) as f64 * 1000.0;
        ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_counts_from_the_first_of_1970_and_back_again() {
        assert_eq!(Day::first_of(1970, 0), Day(0));
        assert_eq!(Day(0).ymd(), (1970, 0, 1));
        assert_eq!(Day(0).weekday(), 4);
        let day = Day::real(2026, 10, 21).unwrap();
        assert_eq!(day.ymd(), (2026, 10, 21));
        assert_eq!(day.weekday(), 6);
        assert_eq!(day.plus(10).ymd(), (2026, 11, 1));
    }

    #[test]
    fn a_day_the_calendar_does_not_have_is_none() {
        assert!(Day::real(2026, 1, 30).is_none());
        assert!(Day::real(2026, 12, 3).is_none());
        assert!(Day::real(2024, 1, 29).is_some());
        assert!(Day::real(2026, 1, 29).is_none());
        assert_eq!(days_in(2026, 11), 31);
    }

    #[test]
    fn a_month_on_keeps_the_day_or_stops_at_the_months_end() {
        let end = Day::real(2026, 0, 31).unwrap();
        assert_eq!(end.plus_months(1).ymd(), (2026, 1, 28));
        assert_eq!(end.plus_months(12).ymd(), (2027, 0, 31));
        assert_eq!(Day::real(2026, 10, 15).unwrap().plus_months(2).ymd(), (2027, 0, 15));
    }

    #[test]
    fn a_clock_on_a_day_reads_back_as_that_clock() {
        let day = Day::real(2026, 10, 21).unwrap();
        let t = LocalTime::at(day.at(18, 30) as i64);
        assert_eq!((t.year, t.month, t.day, t.hour, t.minute), (2026, 11, 21, 18, 30));
        assert_eq!(Day::of(day.at(0, 0)), day);
        assert_eq!(Day::of(day.at(23, 59)), day);
    }
}
