mod types;
mod util;
mod loader;
mod reports;
mod output;

use once_cell::sync::Lazy;
use std::io::{self, Write};
use std::sync::Mutex;
use types::CleanRecord;

static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState { data: None }));

struct AppState {
    data: Option<Vec<CleanRecord>>,
}

fn prompt(line: &str) -> String {
    print!("{}", line);
    let _ = io::stdout().flush();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
    buf.trim().to_string()
}

fn handle_load() {
    let path = "dpwh_flood_control_projects.csv";
    match loader::load_and_clean(path) {
        Ok((data, load_report)) => {
            println!(
                "Processing dataset... ({} rows loaded, {} filtered for 2021-2023)",
                load_report.total_rows, load_report.filtered_rows
            );
            if load_report.parse_errors > 0 {
                println!("Note: {} rows skipped due to parse/validation errors.", load_report.parse_errors);
            }
            if load_report.imputed_coords > 0 {
                println!("Info: Imputed coordinates for {} rows.", load_report.imputed_coords);
            }
            let mut state = APP_STATE.lock().unwrap();
            state.data = Some(data);
        }
        Err(e) => {
            eprintln!("Failed to load file: {}", e);
        }
    }
}

fn handle_generate_reports() {
    let data = {
        let state = APP_STATE.lock().unwrap();
        state.data.clone()
    };
    let Some(data) = data else {
        println!("Error: No data loaded. Please load the CSV file first (option 1).\n");
        return;
    };

    println!("Generating reports...");

    let r1 = reports::generate_report1(&data);
    let file1 = "report1_regional_summary.csv";
    if let Err(e) = output::write_csv(file1, &r1) { eprintln!("Write error: {}", e); }
    output::preview_table(
        1,
        "Regional Flood Mitigation Efficiency Summary",
        Some("Filtered: Projects from 2021–2023 only"),
        &r1,
        3,
    );
    println!("(Full table exported to {})", file1);

    let r2 = reports::generate_report2(&data);
    let file2 = "report2_contractor_ranking.csv";
    if let Err(e) = output::write_csv(file2, &r2) { eprintln!("Write error: {}", e); }
    output::preview_table(
        2,
        "Top Contractors Performance Ranking",
        Some("Top 15 by TotalCost, >= 5 Projects"),
        &r2,
        3,
    );
    println!("(Full table exported to {})", file2);

    let r3 = reports::generate_report3(&data);
    let file3 = "report3_cost_trends.csv";
    if let Err(e) = output::write_csv(file3, &r3) { eprintln!("Write error: {}", e); }
    output::preview_table(
        3,
        "Annual Project Type Cost Overrun Trends",
        Some("Grouped by FundingYear & TypeOfWork"),
        &r3,
        4,
    );
    println!("(Full table exported to {})", file3);

    let summary = reports::generate_summary(&data, &r2);
    if let Err(e) = output::write_json("summary.json", &summary) { eprintln!("Write error: {}", e); }
    println!("Summary Stats (summary.json)");
    println!(
        "\nSummary Preview:\n{{\"global_avg_delay\": {}, \"total_savings\": {}}}",
        util::format_number(summary.avg_global_delay, 2),
        util::format_number(summary.total_savings, 2)
    );
}

fn main() {
    loop {
        println!("Select Language Implementation:");
        println!("1. Load the File");
        println!("2. Generate Reports");
        let choice = prompt("\nEnter choice: ");
        match choice.parse::<u32>() {
            Ok(1) => handle_load(),
            Ok(2) => {
                handle_generate_reports();
                let back = prompt("\nBack to Report Selection (Y/N): ");
                match back.to_uppercase().as_str() {
                    "Y" => continue,
                    "N" => break,
                    _ => continue,
                }
            }
            _ => {
                println!("Please enter 1 or 2.\n");
            }
        }
    }
}
