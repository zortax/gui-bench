//! Number formatting and the display unit of each report unit.

/// How a report unit shows: the factor from report values to shown values, and the short label.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Display {
    pub factor: f64,
    /// The unit after a value, such as `ms` or `MiB`; empty for counts and fractions.
    pub label: &'static str,
    /// The unit as words for an axis title, such as `milliseconds`.
    pub words: &'static str,
}

const MIB: f64 = 1024.0 * 1024.0;

/// Returns how a report unit shows.
pub fn display(unit: &str) -> Display {
    let (factor, label, words) = match unit {
        "ms" => (1.0, "ms", "milliseconds"),
        "bytes" => (1.0 / MIB, "MiB", "MiB"),
        "kib" => (1.0, "KiB", "KiB"),
        "ms_per_s" => (1.0, "ms/s", "ms per second"),
        "ms_per_op" => (1.0, "ms/op", "ms per operation"),
        "per_s" => (1.0, "/s", "per second"),
        "px_per_s" => (1.0, "px/s", "pixels per second"),
        "px" => (1.0, "px", "pixels"),
        "fps" => (1.0, "fps", "frames per second"),
        "percent" => (1.0, "%", "percent"),
        "watts" => (1.0, "W", "watts"),
        "score" => (1.0, "", "score"),
        "ratio" => (1.0, "×", "ratio"),
        "fraction" => (1.0, "", "fraction"),
        _ => (1.0, "", "count"),
    };
    Display {
        factor,
        label,
        words,
    }
}

/// Formats a value with thousands separators and no trailing zeros, as tick labels print.
pub fn thousands(value: f64) -> String {
    if !value.is_finite() {
        return "–".to_owned();
    }
    if value.abs() >= 1000.0 {
        return group(value.round());
    }
    trim(&format!("{:.*}", decimals(value, 4), value))
}

/// Formats a value with three significant digits, grouped from a thousand on.
pub fn short(value: f64) -> String {
    if !value.is_finite() {
        return "–".to_owned();
    }
    if value.abs() >= 1000.0 {
        return group(value.round());
    }
    format!("{:.*}", decimals(value, 3), value)
}

/// Formats a report value in its display unit, such as `13.2 ms` or `228 MiB`.
pub fn format_value(value: f64, unit: &str) -> String {
    let d = display(unit);
    let text = short(value * d.factor);
    match d.label {
        "" => text,
        "%" | "×" => format!("{text}{}", d.label),
        label => format!("{text} {label}"),
    }
}

/// Returns the decimals that give `digits` significant digits, from 0 to 6.
fn decimals(value: f64, digits: i32) -> usize {
    let a = value.abs();
    if a == 0.0 {
        return 0;
    }
    let magnitude = a.log10().floor() as i32;
    (digits - 1 - magnitude).clamp(0, 6) as usize
}

fn trim(text: &str) -> String {
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        text.to_owned()
    }
}

fn group(value: f64) -> String {
    let digits = format!("{}", value.abs() as u64);
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if value < 0.0 { format!("-{out}") } else { out }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        assert_eq!(thousands(10000.0), "10,000");
        assert_eq!(thousands(2.5), "2.5");
        assert_eq!(thousands(0.0), "0");
        assert_eq!(short(13.2714), "13.3");
        assert_eq!(short(0.00123), "0.00123");
        assert_eq!(format_value(234_881_024.0, "bytes"), "224 MiB");
        assert_eq!(format_value(1.5, "ratio"), "1.50×");
    }
}
