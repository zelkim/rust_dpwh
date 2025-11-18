use serde::Serialize;
use std::error::Error;
use tabled::{settings::Style, Table, Tabled};

pub fn write_csv<T: Serialize>(path: &str, rows: &[T]) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(path)?;
    for r in rows {
        wtr.serialize(r)?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn write_json<T: Serialize>(path: &str, value: &T) -> Result<(), Box<dyn Error>> {
    let s = serde_json::to_string_pretty(value)?;
    std::fs::write(path, s)?;
    Ok(())
}

pub fn preview_table<T>(report_no: usize, title: &str, note: Option<&str>, rows: &[T], max_rows: usize)
where
    T: Tabled + Clone,
{
    println!("\nReport {}: {}", report_no, title);
    println!("{}", title);
    if let Some(n) = note { println!("({})", n); }
    println!("");
    let slice: Vec<T> = rows.iter().cloned().take(max_rows).collect();
    if slice.is_empty() {
        println!("(no rows)\n");
        return;
    }
    let table_str = Table::new(slice).with(Style::markdown()).to_string();
    println!("{}\n", table_str);
}

pub fn preview_table_rows<T>(rows: &[T], max_rows: usize)
where
    T: Tabled + Clone,
{
    let slice: Vec<T> = rows.iter().cloned().take(max_rows).collect();
    if slice.is_empty() {
        println!("(no rows)\n");
        return;
    }
    let table_str = Table::new(slice).with(Style::markdown()).to_string();
    println!("{}\n", table_str);
}
