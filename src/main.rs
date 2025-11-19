// Entry point and high-level CLI flow.
//
// The Rust binary mirrors the behavior of the original JavaScript script:
// - Option [1] loads and cleans the CSV, printing diagnostics.
// - Option [2] generates three reports and a JSON summary.
// - After generating reports, the user can choose to go back to the
//   selection menu or exit.
mod loader;
mod output;
mod reports;
mod types;
mod util;

use once_cell::sync::Lazy;
use std::io::{self, Write};
use std::sync::Mutex;
use types::CleanRecord;

// Simple in-memory app state so we only load/clean the CSV once but can
// generate reports multiple times in a single run.
static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState { data: None }));

struct AppState {
    data: Option<Vec<CleanRecord>>,
}

/// Read a single line of input after printing the common "Enter choice:" prompt.
///
/// The prompt is reused for both the main menu and simple numeric inputs.
fn read_choice() -> String {
    print!("Enter choice: ");
    let _ = io::stdout().flush();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
    buf.trim().to_string()
}

/// Ask the user whether to go back to the report selection menu after
/// generating reports.
///
/// Returns `true` if the user chose `Y`, `false` if they chose `N`.
fn prompt_back_to_menu() -> bool {
    loop {
        print!("Back to Report Selection (Y/N): ");
        let _ = io::stdout().flush();
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).ok();
        let resp = buf.trim().to_uppercase();
        match resp.as_str() {
            "Y" => return true,
            "N" => return false,
            _ => println!("Invalid choice. Please enter Y or N."),
        }
    }
}

/// Handle option [1]: load and clean the CSV file.
///
/// On success, we store the `Vec<ClanRecord>` in `APP_STATE` and print
/// a short textual summary of what happened.
fn handle_load() {
    let path = "dpwh_flood_control_projects.csv";
    match loader::load_and_clean(path) {
        Ok((data, load_report)) => {
            println!(
                "Processing dataset... ({} rows loaded, {} filtered for 2021–2023)",
                util::format_int(load_report.total_rows as i64),
                util::format_int(load_report.filtered_rows as i64)
            );
            println!(
                "Note: {} rows skipped due to parse/validation errors.",
                util::format_int(load_report.parse_errors as i64)
            );
            if load_report.imputed_coords > 0 {
                println!(
                    "Info: Imputed coordinates for {} rows.",
                    util::format_int(load_report.imputed_coords as i64)
                );
            }
            println!("");
            let mut state = APP_STATE.lock().unwrap();
            state.data = Some(data);
        }
        Err(e) => {
            eprintln!("Failed to load file: {}\n", e);
        }
    }
}

/// Handle option [2]: generate all reports and the JSON summary.
///
/// This function is intentionally side-effectful:
/// - writes three CSV files,
/// - writes a JSOn summary
/// - and prints Markdown previews of each report to the console.
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
    println!("Outputs saved to individual files...\n");

    let r1 = reports::generate_report1(&data);
    let file1 = "report1_regional_summary.csv";
    if let Err(e) = output::write_csv(file1, &r1) {
        eprintln!("Write error: {}", e);
    }
    println!("Report 1: Regional Flood Mitigation Efficiency Summary\n");
    println!("Regional Flood Mitigation Efficiency Summary");
    println!("(Filtered: 2021–2023 Projects)\n");
    output::preview_table_rows(&r1, 2);
    println!("(Full table exported to {})\n", file1);

    let r2 = reports::generate_report2(&data);
    let file2 = "report2_contractor_ranking.csv";
    if let Err(e) = output::write_csv(file2, &r2) {
        eprintln!("Write error: {}", e);
    }
    println!("Report 2: Top Contractors Performance Ranking\n");
    println!("Top Contractors Performance Ranking");
    println!("(Top 15 by TotalCost, >=5 Projects)\n");
    output::preview_table_rows(&r2, 2);
    println!("(Full table exported to {})\n", file2);

    let r3 = reports::generate_report3(&data);
    let file3 = "report3_annual_trends.csv";
    if let Err(e) = output::write_csv(file3, &r3) {
        eprintln!("Write error: {}", e);
    }
    println!("Report 3: Annual Project Type Cost Overrun Trends");
    println!("Annual Project Type Cost Overrun Trends");
    println!("(Grouped by FundingYear and TypeOfWork)\n");
    output::preview_table_rows(&r3, 3);
    println!("(Full table exported to {})\n", file3);

    let summary = reports::generate_summary(&data, &r2);
    if let Err(e) = output::write_json("summary.json", &summary) {
        eprintln!("Write error: {}", e);
    }
    println!("Summary Stats (summary.json):");
    println!(
        "{{\"global_avg_delay\": {}, \"total_savings\": {}}}\n",
        util::format_number(summary.avg_global_delay, 2),
        util::format_number(summary.total_savings, 2)
    );
}

fn main() {
    loop {
        println!("Select Language Implementation:");
        println!("[1] Load the file");
        println!("[2] Generate Reports\n");
        match read_choice().as_str() {
            "1" => {
                handle_load();
            }
            "2" => {
                println!("");
                handle_generate_reports();
                if !prompt_back_to_menu() {
                    println!("Exiting the program.");
                    break;
                }
            }
            _ => {
                println!("Invalid choice. Please enter 1 or 2.\n");
            }
        }
    }
}
