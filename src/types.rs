// Core data structures used across the pipeline: raw CSV rows, cleaned
// records, and the various report output shapes.
//
// `serde` is used for both CSV deserialization (input) and JSON/CSV
// serialization (output). `tabled` is used to pretty-print Markdown tables
// in the terminal previews.
use serde::{Deserialize, Serialize};
use tabled::Tabled;

/// Direct mapping of the input CSV schema.
///
/// All fields are optional `String` because CSV parsing happens in two
/// stages:
/// 1. Deserialize raw strings from disk into this struct.
/// 2. Apply validation, trimming, and type conversion inside `loader.rs`.
#[derive(Debug, Deserialize)]
pub struct RawRow {
    #[serde(rename = "MainIsland")]
    pub main_island: Option<String>,
    #[serde(rename = "Region")]
    pub region: Option<String>,
    #[serde(rename = "Province")]
    pub province: Option<String>,
    #[serde(rename = "TypeOfWork")]
    pub type_of_work: Option<String>,
    #[serde(rename = "FundingYear")]
    pub funding_year: Option<String>,
    #[serde(rename = "ApprovedBudgetForContract")]
    pub approved_budget_for_contract: Option<String>,
    #[serde(rename = "ContractCost")]
    pub contract_cost: Option<String>,
    #[serde(rename = "ActualCompletionDate")]
    pub actual_completion_date: Option<String>,
    #[serde(rename = "Contractor")]
    pub contractor: Option<String>,
    #[serde(rename = "StartDate")]
    pub start_date: Option<String>,
    #[serde(rename = "ProjectLatitude")]
    pub project_latitude: Option<String>,
    #[serde(rename = "ProjectLongitude")]
    pub project_longitude: Option<String>,
    #[serde(rename = "ProvincialCapitalLatitude")]
    pub provincial_capital_latitude: Option<String>,
    #[serde(rename = "ProvincialCapitalLongitude")]
    pub provincial_capital_longitude: Option<String>,
}

/// Fully validated and normalized project record.
///
/// This is the internal representation used by all reporting code. By the
/// time we construct `CleanRecord`, we have:
/// - ensured the funding year is within 2021–2023,
/// - parsed all numeric/date fields,
/// - filled in defaults for missing text fields, and
/// - possibly imputed latitude/longitude values.
#[derive(Debug, Clone)]
pub struct CleanRecord {
    pub funding_year: i32,
    pub region: String,
    pub main_island: String,
    pub province: String,
    pub type_of_work: String,
    pub contractor: String,
    pub approved_budget: f64,
    pub contract_cost: f64,
    pub cost_savings: f64,
    pub completion_delay_days: f64,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

/// Row for Report 1: Regional Flood Mitigation Efficiency Summary.
#[derive(Debug, Serialize, Tabled, Clone)]
pub struct RegionSummaryRow {
    #[serde(rename = "Region")]
    #[tabled(rename = "Region")]
    pub region: String,
    #[serde(rename = "MainIsland")]
    #[tabled(rename = "MainIsland")]
    pub main_island: String,
    #[serde(rename = "TotalBudget")]
    #[tabled(rename = "TotalBudget")]
    pub total_budget: String,
    #[serde(rename = "MedianSavings")]
    #[tabled(rename = "MedianSavings")]
    pub median_savings: String,
    #[serde(rename = "AvgDelay")]
    #[tabled(rename = "AvgDelay")]
    pub avg_delay: String,
    #[serde(rename = "HighDelayPct")]
    #[tabled(rename = "HighDelayPct")]
    pub high_delay_pct: String,
    #[serde(rename = "EfficiencyScore")]
    #[tabled(rename = "EfficiencyScore")]
    pub efficiency_score: String,
}

/// Preview-only variant of `RegionSummaryRow` with prettier number formatting
/// (commas + two decimal places) for console tables.
#[derive(Debug, Tabled, Clone)]
pub struct RegionSummaryRowPreview {
    #[tabled(rename = "Region")]
    pub region: String,
    #[tabled(rename = "MainIsland")]
    pub main_island: String,
    #[tabled(rename = "TotalBudget")]
    pub total_budget: String,
    #[tabled(rename = "MedianSavings")]
    pub median_savings: String,
    #[tabled(rename = "AvgDelay")]
    pub avg_delay: String,
    #[tabled(rename = "HighDelayPct")]
    pub high_delay_pct: String,
    #[tabled(rename = "EfficiencyScore")]
    pub efficiency_score: String,
}

/// Row for Report 2: Top Contractors Performance Ranking.
///
/// The `serde` and `tabled` renames ensure that both the CSV files and the
/// Markdown previews share the same headers (e.g., `TotalCost`, `AvgDelay`).
#[derive(Debug, Serialize, Tabled, Clone)]
pub struct ContractorRankingRow {
    #[serde(rename = "Rank")]
    #[tabled(rename = "Rank")]
    pub rank: usize,
    #[serde(rename = "Contractor")]
    #[tabled(rename = "Contractor")]
    pub contractor: String,
    #[serde(rename = "TotalCost")]
    #[tabled(rename = "TotalCost")]
    pub total_cost: String,
    #[serde(rename = "NumProjects")]
    #[tabled(rename = "NumProjects")]
    pub num_projects: usize,
    #[serde(rename = "AvgDelay")]
    #[tabled(rename = "AvgDelay")]
    pub avg_delay: String,
    #[serde(rename = "TotalSavings")]
    #[tabled(rename = "TotalSavings")]
    pub total_savings: String,
    #[serde(rename = "ReliabilityIndex")]
    #[tabled(rename = "ReliabilityIndex")]
    pub reliability_index: String,
    #[serde(rename = "RiskFlag")]
    #[tabled(rename = "RiskFlag")]
    pub risk_flag: String,
}

/// Preview-only variant of `ContractorRankingRow` with comma formatting for
/// numeric columns in console tables.
#[derive(Debug, Tabled, Clone)]
pub struct ContractorRankingRowPreview {
    #[tabled(rename = "Rank")]
    pub rank: usize,
    #[tabled(rename = "Contractor")]
    pub contractor: String,
    #[tabled(rename = "TotalCost")]
    pub total_cost: String,
    #[tabled(rename = "NumProjects")]
    pub num_projects: usize,
    #[tabled(rename = "AvgDelay")]
    pub avg_delay: String,
    #[tabled(rename = "TotalSavings")]
    pub total_savings: String,
    #[tabled(rename = "ReliabilityIndex")]
    pub reliability_index: String,
    #[tabled(rename = "RiskFlag")]
    pub risk_flag: String,
}

/// Row for Report 3: Annual Project Type Cost Overrun Trends.
///
/// Each row represents a (FundingYear, TypeOfWork) pair together with
/// summary metrics and year-over-year change.
#[derive(Debug, Serialize, Tabled, Clone)]
pub struct TypeTrendRow {
    #[serde(rename = "FundingYear")]
    #[tabled(rename = "FundingYear")]
    pub funding_year: i32,
    #[serde(rename = "TypeOfWork")]
    #[tabled(rename = "TypeOfWork")]
    pub type_of_work: String,
    #[serde(rename = "TotalProjects")]
    #[tabled(rename = "TotalProjects")]
    pub total_projects: usize,
    #[serde(rename = "AvgSavings")]
    #[tabled(rename = "AvgSavings")]
    pub avg_savings: String,
    #[serde(rename = "OverrunRate")]
    #[tabled(rename = "OverrunRate")]
    pub overrun_rate: String,
    #[serde(rename = "YoYChange")]
    #[tabled(rename = "YoYChange")]
    pub yoy_change: String,
}

/// Preview-only variant of `TypeTrendRow` where all numeric columns except
/// `TotalProjects` are rendered with commas and two decimals.
#[derive(Debug, Tabled, Clone)]
pub struct TypeTrendRowPreview {
    #[tabled(rename = "FundingYear")]
    pub funding_year: i32,
    #[tabled(rename = "TypeOfWork")]
    pub type_of_work: String,
    #[tabled(rename = "TotalProjects")]
    pub total_projects: usize,
    #[tabled(rename = "AvgSavings")]
    pub avg_savings: String,
    #[tabled(rename = "OverrunRate")]
    pub overrun_rate: String,
    #[tabled(rename = "YoYChange")]
    pub yoy_change: String,
}

/// High-level summary statistics exported as `summary.json`.
#[derive(Debug, Serialize)]
pub struct SummaryStats {
    pub total_projects: usize,
    pub total_contractors: usize,
    pub total_provinces: usize,
    pub avg_global_delay: f64,
    pub total_savings: String,
}
