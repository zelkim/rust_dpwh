// Utility helpers for parsing and basic statistics.
//
// This module centralizes all the "dirty" CSV/number/date handling so the
// rest of the code can assume clean, typed values.
use chrono::NaiveDate;
use num_format::{Locale, ToFormattedString};

/// Parse a string-like value into `f64` while being forgiving about
/// formatting issues that are common in CSV exports (commas, spaces, text).
///
/// - Accepts `Option<&str>` so callers can pass through optional fields.
/// - Trims whitespace.
/// - Rejects values that contain alphabetic characters.
/// - Strips thousands separators like `","` before parsing.
/// - Returns `None` for anything that cannot be safely parsed.
pub fn parse_f64_safe(s: Option<&str>) -> Option<f64> {
    let s = s?.trim();
    if s.is_empty() {
        return None;
    }
    if s.chars().any(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let s = s.replace(",", "");
    s.parse::<f64>().ok()
}

pub fn parse_i32_safe(s: Option<&str>) -> Option<i32> {
    // `?` propagates `None` early if the option is missing.
    let s = s?.trim();
    if s.is_empty() {
        return None;
    }
    s.parse::<i32>().ok()
}

pub fn parse_date_safe(s: Option<&str>) -> Option<NaiveDate> {
    // CSV dates are expected in `YYYY-MM-DD` format.
    let s = s?.trim();
    if s.is_empty() {
        return None;
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

pub fn days_diff(start: NaiveDate, end: NaiveDate) -> f64 {
    // `NaiveDate` supports subtraction; the result is a `Duration` in days.
    (end - start).num_days() as f64
}

pub fn average(v: &[f64]) -> f64 {
    // Standard arithmetic mean; returns 0 for an empty slice to avoid NaNs.
    if v.is_empty() {
        return 0.0;
    }
    let sum: f64 = v.iter().copied().sum();
    sum / v.len() as f64
}

pub fn median(mut v: Vec<f64>) -> f64 {
    // Median of a list of numbers. We accept `Vec<f64>` by value so the
    // function can sort in-place without cloning at the call site.
    if v.is_empty() {
        return 0.0;
    }
    // Use `partial_cmp` to handle floating-point comparisons and fall back to
    // equality if either side is NaN.
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = v.len() / 2;
    if v.len() % 2 == 1 {
        v[mid]
    } else {
        (v[mid - 1] + v[mid]) / 2.0
    }
}

pub fn format_number(n: f64, decimals: usize) -> String {
    // Format a floating-point value with:
    // - a fixed number of decimal places, and
    // - locale-aware thousands separators (e.g., `1,234,567.89`).
    let neg = n.is_sign_negative();
    let abs_n = n.abs();
    // First, format to a plain fixed-decimal string like `1234567.89`.
    let s = format!("{:.*}", decimals, abs_n);
    let mut parts = s.split('.');
    let int_part = parts.next().unwrap_or("0");
    let frac_part = parts.next();
    // Use `num-format` to insert commas into the integer portion.
    let int_val: i64 = int_part.parse().unwrap_or(0);
    let mut res = int_val.to_formatted_string(&Locale::en);
    if let Some(frac) = frac_part {
        if decimals > 0 {
            res.push('.');
            res.push_str(frac);
        }
    } else if decimals > 0 {
        res.push('.');
        res.push_str(&"0".repeat(decimals));
    }
    if neg {
        format!("-{}", res)
    } else {
        res
    }
}

pub fn format_int<T>(n: T) -> String
where
    T: ToFormattedString,
{
    // Thin wrapper around `num-format` for integer-like values. This is used
    // for counts in console messages (e.g., `9,855 rows loaded`).
    n.to_formatted_string(&Locale::en)
}
