// Output utilities for writing CSV/JSON files and previewing tables
// in the terminal.
//
// - `serde` drives serialization of our structs.
// - `csv` writes properly escaped CSV with headers.
// - `tabled` renders Markdown-compatible preview tables
use serde::Serialize;
use std::error::Error;
use tabled::{settings::Style, Table, Tabled};

/// Write a sequence of `rows` to a CSV file at `path`.
///
/// The type `T` only has to implement `Serialize`; column headers come from
/// the `serde(rename = ..)` attributes on the structsin `types.rs`.
pub fn write_csv<T: Serialize>(path: &str, rows: &[T]) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(path)?;
    for r in rows {
        wtr.serialize(r)?;
    }
    // Explicitly flush to ensure all bytes hit disk before we return.
    wtr.flush()?;
    Ok(())
}

/// Serialize `value` as pretty-printed JSON and write it to `path`.
pub fn write_json<T: Serialize>(path: &str, value: &T) -> Result<(), Box<dyn Error>> {
    let s = serde_json::to_string_pretty(value)?;
    std::fs::write(path, s)?;
    Ok(())
}

/// Render up to `max_rows` as a Markdown table preview in the console.
///
/// `tabled` inspects the `Tabledd` implementation (derived from struct
/// fields) and uses the configured`Style::markdown()` to emit a header
/// row, a divider, and aligned coluns.
pub fn preview_table_rows<T>(rows: &[T], max_rows: usize)
where
    T: Tabled + Clone,
{
    // Clone just the first `max_rows` 
    let slice: Vec<T> = rows.iter().cloned().take(max_rows).collect();
    if slice.is_empty() {
        println!("(no rows)\n");
        return;
    }
    // Render the Markdon table. On Windows terminals, `tabled` may include
    // `\r` characters, which can mess up the divider line, so we strip them.
    let table_str = Table::new(slice).with(Style::markdown()).to_string();
    let normalized = table_str.replace('\r', "");
    println!("{}\n", normalized);
}
