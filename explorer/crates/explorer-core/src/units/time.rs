//! Dates and durations as text.

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Returns the civil date (year, month 1–12, day) of days since 1970-01-01.
fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// Formats Unix seconds plus an offset in minutes, such as `11 Nov 2026, 21:23`.
pub fn date_time(unix: u64, offset_minutes: i64) -> String {
    let t = unix as i64 + offset_minutes * 60;
    let (y, m, d) = civil(t.div_euclid(86_400));
    let s = t.rem_euclid(86_400);
    format!(
        "{d} {} {y}, {:02}:{:02}",
        MONTHS[(m - 1) as usize],
        s / 3600,
        (s % 3600) / 60
    )
}

/// Formats a duration in seconds, such as `2 h 14 min` or `48 s`.
pub fn duration(seconds: u64) -> String {
    match seconds {
        s if s >= 3600 => format!("{} h {} min", s / 3600, (s % 3600) / 60),
        s if s >= 60 => format!("{} min", s / 60),
        s => format!("{s} s"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_dates() {
        assert_eq!(date_time(0, 0), "1 Jan 1970, 00:00");
        assert_eq!(date_time(1_794_428_581, 0), "11 Nov 2026, 20:23");
        assert_eq!(duration(8000), "2 h 13 min");
    }
}
