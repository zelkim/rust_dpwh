use chrono::NaiveDate;
use num_format::{Locale, ToFormattedString};

pub fn parse_f64_safe(s: Option<&str>) -> Option<f64> {
    let s = s?.trim();
    if s.is_empty() { return None; }
    if s.chars().any(|c| c.is_ascii_alphabetic()) { return None; }
    let s = s.replace(",", "");
    s.parse::<f64>().ok()
}

pub fn parse_i32_safe(s: Option<&str>) -> Option<i32> {
    let s = s?.trim();
    if s.is_empty() { return None; }
    s.parse::<i32>().ok()
}

pub fn parse_date_safe(s: Option<&str>) -> Option<NaiveDate> {
    let s = s?.trim();
    if s.is_empty() { return None; }
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

pub fn days_diff(start: NaiveDate, end: NaiveDate) -> f64 {
    (end - start).num_days() as f64
}

pub fn average(v: &[f64]) -> f64 {
    if v.is_empty() { return 0.0; }
    let sum: f64 = v.iter().copied().sum();
    sum / v.len() as f64
}

pub fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() { return 0.0; }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = v.len() / 2;
    if v.len() % 2 == 1 {
        v[mid]
    } else {
        (v[mid - 1] + v[mid]) / 2.0
    }
}

pub fn clamp_0_100(x: f64) -> f64 {
    if !x.is_finite() { return 0.0; }
    if x < 0.0 { 0.0 } else if x > 100.0 { 100.0 } else { x }
}

pub fn format_number(n: f64, decimals: usize) -> String {
    // Fixed decimals, with thousands separators.
    let neg = n.is_sign_negative();
    let abs_n = n.abs();
    let s = format!("{:.*}", decimals, abs_n);
    let mut parts = s.split('.');
    let int_part = parts.next().unwrap_or("0");
    let frac_part = parts.next();
    let int_val: i64 = int_part.parse().unwrap_or(0);
    let mut res = int_val.to_formatted_string(&Locale::en);
    if let Some(frac) = frac_part {
        if decimals > 0 { res.push('.'); res.push_str(frac); }
    } else if decimals > 0 {
        res.push('.');
        res.push_str(&"0".repeat(decimals));
    }
    if neg { format!("-{}", res) } else { res }
}
