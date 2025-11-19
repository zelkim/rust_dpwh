use serde::{Deserialize, Serialize};
use tabled::Tabled;

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

#[derive(Debug, Serialize)]
pub struct SummaryStats {
    pub total_projects: usize,
    pub total_contractors: usize,
    pub total_provinces: usize,
    pub avg_global_delay: f64,
    pub total_savings: f64,
}
