

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
use types::{
    CleanRecord,
    ContractorRankingRowPreview,
    RegionSummaryRowPreview,
    TypeTrendRowPreview,
};
use util::format_number;

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
    let r1_preview: Vec<RegionSummaryRowPreview> = r1
        .iter()
        .map(|row| RegionSummaryRowPreview {
            region: row.region.clone(),
            main_island: row.main_island.clone(),
            total_budget: parse_and_format(&row.total_budget),
            median_savings: parse_and_format(&row.median_savings),
            avg_delay: parse_and_format(&row.avg_delay),
            high_delay_pct: parse_and_format(&row.high_delay_pct),
            efficiency_score: parse_and_format(&row.efficiency_score),
        })
        .collect();
    output::preview_table_rows(&r1_preview, 2);
    println!("(Full table exported to {})\n", file1);

    let r2 = reports::generate_report2(&data);
    let file2 = "report2_contractor_ranking.csv";
    if let Err(e) = output::write_csv(file2, &r2) {
        eprintln!("Write error: {}", e);
    }
    println!("Report 2: Top Contractors Performance Ranking\n");
    println!("Top Contractors Performance Ranking");
    println!("(Top 15 by TotalCost, >=5 Projects)\n");
    let r2_preview: Vec<ContractorRankingRowPreview> = r2
        .iter()
        .map(|row| ContractorRankingRowPreview {
            rank: row.rank,
            contractor: row.contractor.clone(),
            total_cost: parse_and_format(&row.total_cost),
            num_projects: row.num_projects,
            avg_delay: parse_and_format(&row.avg_delay),
            total_savings: parse_and_format(&row.total_savings),
            reliability_index: parse_and_format(&row.reliability_index),
            risk_flag: row.risk_flag.clone(),
        })
        .collect();
    output::preview_table_rows(&r2_preview, 2);
    println!("(Full table exported to {})\n", file2);

    let r3 = reports::generate_report3(&data);
    let file3 = "report3_annual_trends.csv";
    if let Err(e) = output::write_csv(file3, &r3) {
        eprintln!("Write error: {}", e);
    }
    println!("Report 3: Annual Project Type Cost Overrun Trends");
    println!("Annual Project Type Cost Overrun Trends");
    println!("(Grouped by FundingYear and TypeOfWork)\n");
    let r3_preview: Vec<TypeTrendRowPreview> = r3
        .iter()
        .map(|row| TypeTrendRowPreview {
            funding_year: row.funding_year,
            type_of_work: row.type_of_work.clone(),
            // TotalProjects should not be formatted with decimals.
            total_projects: row.total_projects,
            avg_savings: parse_and_format(&row.avg_savings),
            overrun_rate: parse_and_format(&row.overrun_rate),
            yoy_change: parse_and_format(&row.yoy_change),
        })
        .collect();
    output::preview_table_rows(&r3_preview, 3);
    println!("(Full table exported to {})\n", file3);

    let mut summary = reports::generate_summary(&data, &r2);
    // Fill in report-level counts to match the JS summary.json shape.
    summary.report1_regions = r1.len();
    summary.report2_contractors = r2.len();
    summary.report3_entries = r3.len();
    if let Err(e) = output::write_json("summary.json", &summary) {
        eprintln!("Write error: {}", e);
    }
    println!("Summary Stats (summary.json):");
    println!(
        "{{\"global_avg_delay_days\": \"{}\", \"total_savings\": {}}}\n",
        summary.global_avg_delay_days,
        format_number(summary
            .total_savings
            .replace(",", "")
            .parse::<f64>()
            .unwrap_or(0.0),
            2
        )
    );
}

/// Helper: parse a numeric string and format with commas and two decimals
fn parse_and_format(s: &str) -> String {
    match s.replace(",", "").parse::<f64>() {
        Ok(v) => format_number(v, 2),
        Err(_) => s.to_string(),
    }
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
                    println!(" Exiting DPWH Flood Control Data Pipeline...");
                    break;
                }
            }
            _ => {
                println!("Invalid choice. Please enter 1 or 2.\n");
            }
        }
    }
}
