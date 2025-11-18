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

fn read_choice() -> String {
    print!("Enter choice: ");
    let _ = io::stdout().flush();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
    let choice = buf.trim().to_string();
    print!("\rEnter choice: {}\n", choice); // replace the same line
    choice
}

fn handle_load() {
    let path = "dpwh_flood_control_projects.csv";
    match loader::load_and_clean(path) {
        Ok((data, load_report)) => {
            println!(
                "Processing dataset... ({} rows loaded, {} filtered for 2021–2023)\n",
                load_report.total_rows, load_report.filtered_rows
            );
            let mut state = APP_STATE.lock().unwrap();
            state.data = Some(data);
        }
        Err(e) => {
            eprintln!("Failed to load file: {}\n", e);
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
    println!("Outputs saved to individual files...\n");

    let r1 = reports::generate_report1(&data);
    let file1 = "report1_regional_summary.csv";
    if let Err(e) = output::write_csv(file1, &r1) { eprintln!("Write error: {}", e); }
    println!("Report 1: Regional Flood Mitigation Efficiency Summary\n");
    println!("Regional Flood Mitigation Efficiency Summary");
    println!("(Filtered: 2021–2023 Projects)\n");
    output::preview_table_rows(&r1, 2);
    println!("(Full table exported to {})\n", file1);

    let r2 = reports::generate_report2(&data);
    let file2 = "report2_contractor_ranking.csv";
    if let Err(e) = output::write_csv(file2, &r2) { eprintln!("Write error: {}", e); }
    println!("Report 2: Top Contractors Performance Ranking\n");
    println!("Top Contractors Performance Ranking");
    println!("(Top 15 by TotalCost, >=5 Projects)\n");
    output::preview_table_rows(&r2, 2);
    println!("(Full table exported to {})\n", file2);

    let r3 = reports::generate_report3(&data);
    let file3 = "report3_annual_trends.csv";
    if let Err(e) = output::write_csv(file3, &r3) { eprintln!("Write error: {}", e); }
    println!("Report 3: Annual Project Type Cost Overrun Trends");
    println!("Annual Project Type Cost Overrun Trends");
    println!("(Grouped by FundingYear and TypeOfWork)\n");
    output::preview_table_rows(&r3, 3);
    println!("(Full table exported to {})\n", file3);

    let summary = reports::generate_summary(&data, &r2);
    if let Err(e) = output::write_json("summary.json", &summary) { eprintln!("Write error: {}", e); }
    println!("Summary Stats (summary.json):");
    println!("{{\"global_avg_delay\": {}, \"total_savings\": {}}}\n", util::format_number(summary.avg_global_delay, 2), util::format_number(summary.total_savings, 2));
}

fn main() {
    println!("Select Language Implementation:");
    println!("[1] Load the file");
    println!("[2] Generate Reports\n");
    let first = read_choice();
    if first.trim() == "1" { handle_load(); }
    let second = read_choice();
    if second.trim() == "2" { println!(""); handle_generate_reports(); }
}
