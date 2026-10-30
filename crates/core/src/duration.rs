//! The duration of a track in `itunes:duration` (ADR 0094).

/// Read the text of `itunes:duration` as a number of seconds.
///
/// The forms are seconds (`261`), seconds with a fraction (`261.9`),
/// `M:SS`, `MM:SS`, `H:MM:SS` and `HH:MM:SS`. Space at the ends is
/// ignored. A part after the first part is from 0 to 59. A fraction of the
/// seconds rounds down. Other text, an empty text and a negative value
/// give `None`.
#[must_use]
pub fn parse_duration(text: &str) -> Option<u64> {
    let parts: Vec<&str> = text.trim().split(':').collect();
    if parts.len() > 3 {
        return None;
    }
    let last = parts.len() - 1;
    let mut total: u64 = 0;
    for (index, part) in parts.iter().enumerate() {
        // Only the seconds can have a fraction.
        let whole = if index == last {
            part.split_once('.').map_or(*part, |(whole, fraction)| {
                if fraction.bytes().all(|b| b.is_ascii_digit()) {
                    whole
                } else {
                    ""
                }
            })
        } else {
            part
        };
        if whole.is_empty() || !whole.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let value: u64 = whole.parse().ok()?;
        if index > 0 && value > 59 {
            return None;
        }
        total = total.checked_mul(if index == 0 { 1 } else { 60 })?;
        total = total.checked_add(value)?;
    }
    Some(total)
}

/// The duration as `HH:MM:SS`, for example `00:04:21`. A duration of more
/// than 99 hours has more digits in the hours.
#[must_use]
pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_duration_reads_each_form() {
        for text in [
            "261", "4:21", "04:21", "0:04:21", "00:04:21", "261.9", " 4:21 ",
        ] {
            assert_eq!(parse_duration(text), Some(261), "{text}");
        }
        assert_eq!(parse_duration("1:02:03"), Some(3723));
    }

    #[test]
    fn parse_duration_refuses_other_text() {
        for text in [
            "",
            " ",
            "4:61",
            "four minutes",
            "-5",
            "1:2:3:4",
            "4:",
            ":21",
            "4.5:21",
        ] {
            assert_eq!(parse_duration(text), None, "{text}");
        }
    }

    #[test]
    fn format_duration_gives_hh_mm_ss() {
        assert_eq!(format_duration(261), "00:04:21");
        assert_eq!(format_duration(3723), "01:02:03");
        assert_eq!(format_duration(100 * 3600), "100:00:00");
    }
}
