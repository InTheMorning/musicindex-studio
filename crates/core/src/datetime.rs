//! Times in RFC 3339 and RFC 2822 text (ADR 0093).

/// A time and the offset of its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Moment {
    /// The time in seconds since the Unix epoch, in UTC.
    pub seconds: i64,
    /// The offset in minutes east of UTC, of the text that the core read or
    /// will write.
    pub offset_minutes: i32,
}

/// Read an ISO 8601 time with a zone, such as `2026-11-01T10:00:00Z` or
/// `2021-09-26T07:30:00.000-0600`. Gives the time in seconds since the
/// Unix epoch.
///
/// The time must have the date, the hours and the minutes. The seconds and
/// a fraction of a second are optional. The zone is `Z`, `+HH:MM`, `+HHMM`
/// or `+HH`, or the same with `-`. A time with no zone gives `None`,
/// because Studio cannot know the zone that the user meant.
#[must_use]
pub fn parse_time(text: &str) -> Option<i64> {
    parse_rfc3339(text).map(|moment| moment.seconds)
}

/// Read an RFC 3339 time with a zone, in the forms that [`parse_time`]
/// reads. Gives the time and the offset of its text. `Z` gives the offset 0.
#[must_use]
pub fn parse_rfc3339(text: &str) -> Option<Moment> {
    let text = text.trim();
    let (date, rest) = text.split_once('T')?;
    let mut date_parts = date.split('-');
    let year: i64 = digits(date_parts.next()?, 4)?;
    let month: i64 = digits(date_parts.next()?, 2)?;
    let day: i64 = digits(date_parts.next()?, 2)?;
    if date_parts.next().is_some() || !(1..=12).contains(&month) {
        return None;
    }
    if day < 1 || day > days_in_month(year, month) {
        return None;
    }
    let zone_at = rest.find(['Z', '+', '-'])?;
    let (clock, zone) = rest.split_at(zone_at);
    let mut clock_parts = clock.split(':');
    let hour = digits(clock_parts.next()?, 2)?;
    let minute = digits(clock_parts.next()?, 2)?;
    let second = match clock_parts.next() {
        None => 0,
        Some(part) => {
            let whole = part.split_once('.').map_or(part, |(whole, fraction)| {
                if fraction.is_empty() || !fraction.bytes().all(|b| b.is_ascii_digit()) {
                    // A bad fraction makes the whole value bad.
                    ""
                } else {
                    whole
                }
            });
            digits(whole, 2)?
        }
    };
    if clock_parts.next().is_some() || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let offset = zone_offset(zone)?;
    let days = days_from_civil(year, month, day);
    Some(Moment {
        seconds: days * 86_400 + hour * 3600 + minute * 60 + second - offset,
        offset_minutes: i32::try_from(offset / 60).ok()?,
    })
}

/// The value of a text of exactly `count` ASCII digits.
fn digits(text: &str, count: usize) -> Option<i64> {
    if text.len() != count || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// The offset of a zone in seconds east of UTC.
fn zone_offset(zone: &str) -> Option<i64> {
    if zone == "Z" {
        return Some(0);
    }
    let sign = match zone.as_bytes().first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let rest = &zone[1..];
    let (hours, minutes) = match rest.len() {
        2 => (digits(rest, 2)?, 0),
        4 => (digits(&rest[..2], 2)?, digits(&rest[2..], 2)?),
        5 if rest.as_bytes()[2] == b':' => (digits(&rest[..2], 2)?, digits(&rest[3..], 2)?),
        _ => return None,
    };
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some(sign * (hours * 3600 + minutes * 60))
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// The number of days from 1970-01-01 to a date of the Gregorian calendar.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_index = (month + 9) % 12;
    let day_of_year = (153 * month_index + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Read an RFC 2822 time, such as `Fri, 6 Feb 2026 09:54:10 +0000`.
///
/// The weekday is optional, and the seconds are optional. A weekday that
/// does not agree with the date is accepted, because some feeds have this
/// error. The zone is `+HHMM`, `-HHMM`, `GMT`, `UT`, `UTC`, `Z` or a US
/// zone of RFC 5322 section 4.3. All other text gives `None`.
#[must_use]
pub fn parse_rfc2822(text: &str) -> Option<Moment> {
    let mut parts = text.split_whitespace().peekable();
    if let Some(first) = parts.peek()
        && let Some(name) = first.strip_suffix(',')
    {
        if !WEEKDAYS.iter().any(|d| d.eq_ignore_ascii_case(name)) {
            return None;
        }
        parts.next();
    }
    let day_text = parts.next()?;
    let month_text = parts.next()?;
    let year_text = parts.next()?;
    let clock = parts.next()?;
    let zone = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let day = if matches!(day_text.len(), 1 | 2) {
        digits_any(day_text)?
    } else {
        return None;
    };
    let month = MONTHS
        .iter()
        .position(|m| m.eq_ignore_ascii_case(month_text))
        .and_then(|i| i64::try_from(i + 1).ok())?;
    let year = digits(year_text, 4)?;
    if day < 1 || day > days_in_month(year, month) {
        return None;
    }
    let mut clock_parts = clock.split(':');
    let hour = digits(clock_parts.next()?, 2)?;
    let minute = digits(clock_parts.next()?, 2)?;
    let second = clock_parts.next().map_or(Some(0), |p| digits(p, 2))?;
    if clock_parts.next().is_some() || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let offset = rfc2822_zone(zone)?;
    let days = days_from_civil(year, month, day);
    Some(Moment {
        seconds: days * 86_400 + hour * 3600 + minute * 60 + second - offset * 60,
        offset_minutes: i32::try_from(offset).ok()?,
    })
}

/// The value of a text of one or more ASCII digits.
fn digits_any(text: &str) -> Option<i64> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// The offset in minutes east of UTC of a zone of RFC 2822.
fn rfc2822_zone(zone: &str) -> Option<i64> {
    let hours = match zone.to_ascii_uppercase().as_str() {
        "GMT" | "UT" | "UTC" | "Z" => 0,
        "EST" | "CDT" => -5,
        "EDT" => -4,
        "CST" | "MDT" => -6,
        "MST" | "PDT" => -7,
        "PST" => -8,
        _ => {
            let sign = match zone.as_bytes().first()? {
                b'+' => 1,
                b'-' => -1,
                _ => return None,
            };
            let rest = &zone[1..];
            if rest.len() != 4 {
                return None;
            }
            let (h, m) = (digits(&rest[..2], 2)?, digits(&rest[2..], 2)?);
            if h > 23 || m > 59 {
                return None;
            }
            return Some(sign * (h * 60 + m));
        }
    };
    Some(hours * 60)
}

/// Write a time in RFC 2822, such as `Fri, 30 Oct 2026 20:00:00 +0100`. The
/// zone is always `+HHMM` or `-HHMM`.
#[must_use]
pub fn format_rfc2822(moment: Moment) -> String {
    let (year, month, day, clock) = local_parts(moment);
    let days = (moment.seconds + i64::from(moment.offset_minutes) * 60).div_euclid(86_400);
    let weekday = WEEKDAYS[usize::try_from((days + 4).rem_euclid(7)).unwrap_or(0)];
    let month_name = MONTHS[usize::try_from(month - 1).unwrap_or(0)];
    let offset = i64::from(moment.offset_minutes);
    let sign = if offset < 0 { '-' } else { '+' };
    format!(
        "{weekday}, {day:02} {month_name} {year:04} {:02}:{:02}:{:02} {sign}{:02}{:02}",
        clock / 3600,
        clock % 3600 / 60,
        clock % 60,
        offset.abs() / 60,
        offset.abs() % 60,
    )
}

/// Write a time in RFC 3339, such as `2026-10-30T20:00:00+01:00`. The
/// offset 0 gives `+00:00`, not `Z`.
#[must_use]
pub fn format_rfc3339(moment: Moment) -> String {
    let (year, month, day, clock) = local_parts(moment);
    let offset = i64::from(moment.offset_minutes);
    let sign = if offset < 0 { '-' } else { '+' };
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}{sign}{:02}:{:02}",
        clock / 3600,
        clock % 3600 / 60,
        clock % 60,
        offset.abs() / 60,
        offset.abs() % 60,
    )
}

/// The year, month and day of the local date, and the seconds of the local
/// day.
fn local_parts(moment: Moment) -> (i64, i64, i64, i64) {
    let local = moment.seconds + i64::from(moment.offset_minutes) * 60;
    let (year, month, day) = civil_from_days(local.div_euclid(86_400));
    (year, month, day, local.rem_euclid(86_400))
}

/// The date of the Gregorian calendar that is a number of days after
/// 1970-01-01. It is the inverse of [`days_from_civil`].
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    const END: i64 = 1_793_534_400; // 2026-11-01T12:00:00Z

    #[test]
    fn a_time_with_a_zone_is_read_and_a_time_with_no_zone_is_not() {
        assert_eq!(parse_time("2026-11-01T12:00:00Z"), Some(END));
        assert_eq!(parse_time("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_time("1970-01-01T00:00Z"), Some(0));
        assert_eq!(parse_time("2026-11-01T12:00:00.000Z"), Some(END));
        assert_eq!(parse_time("2026-11-01T06:00:00-06:00"), Some(END));
        assert_eq!(parse_time("2026-11-01T06:00:00.000-0600"), Some(END));
        assert_eq!(parse_time("2026-11-01T14:00:00+02"), Some(END));
        assert_eq!(parse_time("2024-02-29T00:00:00Z"), Some(1_709_164_800));
        for bad in [
            "2026-11-01T12:00:00",
            "2026-11-01",
            "2026-13-01T12:00:00Z",
            "2026-02-29T12:00:00Z",
            "2026-11-01T24:00:00Z",
            "2026-11-01T12:00:00.Z",
            "2026-11-01T12:00:00+2",
            "soon",
            "",
        ] {
            assert_eq!(parse_time(bad), None, "{bad}");
        }
    }

    fn moment(seconds: i64, offset_minutes: i32) -> Moment {
        Moment {
            seconds,
            offset_minutes,
        }
    }

    // 2026-02-06T09:54:10Z
    const FEB: i64 = 1_770_371_650;

    #[test]
    fn rfc2822_forms_give_the_seconds_and_the_offset() {
        let cases = [
            ("Fri, 6 Feb 2026 09:54:10 +0000", moment(FEB, 0)),
            ("Fri, 06 Feb 2026 09:54:10 GMT", moment(FEB, 0)),
            ("Fri, 6 Feb 2026 04:54:10 -0500", moment(FEB, -300)),
            ("Fri, 6 Feb 2026 04:54:10 EST", moment(FEB, -300)),
            ("6 Feb 2026 09:54:10 +0000", moment(FEB, 0)),
            ("  Fri, 6 Feb 2026 09:54 UTC ", moment(FEB - 10, 0)),
            ("Mon, 6 Feb 2026 15:24:10 +0530", moment(FEB, 330)),
        ];
        for (text, expected) in cases {
            assert_eq!(parse_rfc2822(text), Some(expected), "{text}");
        }
    }

    #[test]
    fn rfc2822_text_that_is_not_a_time_gives_none() {
        for bad in [
            "",
            "2026-10-30",
            "yesterday",
            "Fri, 6 Fév 2026 09:54:10 +0000",
            "Fri, 32 Oct 2026 09:54:10 +0000",
            "Fri, 30 Feb 2026 09:54:10 +0000",
            "Fri, 6 Feb 2026 25:54:10 +0000",
            "Fri, 6 Feb 2026 09:54:10",
            "Fri, 6 Feb 2026 09:54:10 +00",
            "Xyz, 6 Feb 2026 09:54:10 +0000",
        ] {
            assert_eq!(parse_rfc2822(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_formats_give_the_examples() {
        let m = moment(1_793_386_800, 60);
        assert_eq!(format_rfc2822(m), "Fri, 30 Oct 2026 20:00:00 +0100");
        assert_eq!(format_rfc3339(m), "2026-10-30T20:00:00+01:00");
        assert_eq!(format_rfc3339(moment(FEB, 0)), "2026-02-06T09:54:10+00:00");
        assert_eq!(
            format_rfc2822(moment(0, -90)),
            "Wed, 31 Dec 1969 22:30:00 -0130"
        );
    }

    #[test]
    fn rfc3339_text_keeps_the_offset_in_a_round_trip() {
        for offset in [0, 330, -360] {
            let m = moment(FEB, offset);
            assert_eq!(parse_rfc3339(&format_rfc3339(m)), Some(m));
            assert_eq!(parse_rfc2822(&format_rfc2822(m)), Some(m));
        }
    }

    #[test]
    fn each_corpus_pub_date_reads() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
        let mut count = 0;
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "xml") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            let mut rest = text.as_str();
            while let Some(at) = rest.find("<pubDate>") {
                rest = &rest[at + 9..];
                let end = rest.find("</pubDate>").unwrap();
                let value = rest[..end].trim();
                let value = value
                    .strip_prefix("<![CDATA[")
                    .and_then(|v| v.strip_suffix("]]>"))
                    .unwrap_or(value);
                assert!(
                    parse_rfc2822(value).is_some(),
                    "{}: {value}",
                    path.display()
                );
                count += 1;
            }
        }
        assert!(count > 0, "The corpus has no pubDate.");
    }
}
