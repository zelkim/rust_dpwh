// Report-generation functions.
//
// These functions take in cleaned project records and aggregate them into
// higher-level summaries for:
// 1. Regions (Report 1)
// 2. Contractors (Report 2)
// 3. Funding year + type of work trends (Report 3)
// 4. Overall summary statistics
use crate::types::{
    CleanRecord, ContractorRankingRow, RegionSummaryRow, SummaryStats, TypeTrendRow,
};
use crate::util::{average, format_number, median};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

/// Generate Report 1: Regional Flood Mitigation Efficiency Summary.
///
/// Algorithm (per (Region, MainIsland) group):
/// - Aggregate budgets, cost savings, and completion delays.
/// - Compute:
///   * TotalBudget (sum of budgets)
///   * MedianSavings (median of savings)
///   * AvgDelay (mean of delays)
///   * HighDelayPct (% of projects with delay > 30 days)
///   * Raw efficiency = MedianSavings / AvgDelay (guarding against /0).
/// - After computing raw efficiency for all regions, perform a min-max
///   normalization so that EfficiencyScore lies in [0, 100] and preserves
pub fn generate_report1(data: &[CleanRecord]) -> Vec<RegionSummaryRow> {
    // Accumulator for each (Region, MainIsland) group.
    #[derive(Default)]
    struct Acc {
        budgets: Vec<f64>,
        savings: Vec<f64>,
        delays: Vec<f64>,
        region: String,
        island: String,
    }
    // Prepared row that holds both formatted strings and the raw
    // efficiency score (used for min-max normalization later).
    #[derive(Clone)]
    struct RowPrep {
        region: String,
        main_island: String,
        total_budget: String,
        median_savings: String,
        avg_delay: String,
        high_delay_pct: String,
        raw_efficiency: f64,
    }

    // First pass: group all rows by (Region, MainIsland).
    let mut map: HashMap<(String, String), Acc> = HashMap::new();
    for r in data {
        let key = (r.region.clone(), r.main_island.clone());
        let e = map.entry(key.clone()).or_insert_with(|| Acc {
            budgets: vec![],
            savings: vec![],
            delays: vec![],
            region: key.0.clone(),
            island: key.1.clone(),
        });
        e.budgets.push(r.approved_budget);
        e.savings.push(r.cost_savings);
        e.delays.push(r.completion_delay_days);
    }
    // Second pass: compute group-level aggregates and raw efficiency.
    let prepared: Vec<RowPrep> = map
        .into_values()
        .map(|acc| {
            let avg_delay = average(&acc.delays);
            let delay_over_30 = if acc.delays.is_empty() {
                0.0
            } else {
                (acc.delays.iter().filter(|d| **d > 30.0).count() as f64 / acc.delays.len() as f64)
                    * 100.0
            };
            let med_savings = median(acc.savings.clone());
            // Raw efficiency is defined as `median_savings / avg_delay`.
            // Values are clamped to non-negative and non-NaN here; the
            // normalization to [0,100] happens in a separate pass below.
            let mut eff = if avg_delay <= 0.0 {
                0.0
            } else {
                med_savings / avg_delay
            };
            if !eff.is_finite() || eff < 0.0 {
                eff = 0.0;
            }
            let total_budget: f64 = acc.budgets.iter().sum();
            RowPrep {
                region: acc.region,
                main_island: acc.island,
                total_budget: format!("{:.2}", total_budget),
                median_savings: format!("{:.2}", med_savings),
                avg_delay: format!("{:.2}", avg_delay),
                high_delay_pct: format!("{:.2}", delay_over_30),
                raw_efficiency: eff,
            }
        })
        .collect();
    if prepared.is_empty() {
        return Vec::new();
    }

    // Compute the min and max raw efficiency across all regions.
    let (mut min_eff, mut max_eff) = (f64::MAX, f64::MIN);
    for row in &prepared {
        min_eff = min_eff.min(row.raw_efficiency);
        max_eff = max_eff.max(row.raw_efficiency);
    }
    if !min_eff.is_finite() {
        min_eff = 0.0;
    }
    if !max_eff.is_finite() {
        max_eff = 0.0;
    }
    let range = max_eff - min_eff;

    // Third pass: transform raw efficiency into a 0–100 score using
    // min-max scaling, then build final `RegionSummaryRow` values.
    let mut scored: Vec<(f64, RegionSummaryRow)> = prepared
        .into_iter()
        .map(|row| {
            let mut scaled = if range.abs() < f64::EPSILON {
                0.0
            } else {
                ((row.raw_efficiency - min_eff) / range) * 100.0
            };
            if !scaled.is_finite() {
                scaled = 0.0;
            }
            scaled = scaled.clamp(0.0, 100.0);
            let rendered = RegionSummaryRow {
                region: row.region,
                main_island: row.main_island,
                total_budget: row.total_budget,
                median_savings: row.median_savings,
                avg_delay: row.avg_delay,
                high_delay_pct: row.high_delay_pct,
                // CSV cells should be "100.00" style, without
                // thousands separators.
                efficiency_score: format!("{:.2}", scaled),
            };
            (scaled, rendered)
        })
        .collect();

    // Sort descending by scaled efficiency so the best-performing regions
    // appear first in both the preview and CSV.
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
    scored.into_iter().map(|(_, row)| row).collect()
}

/// Generate Report 2: Top Contractors Performance Ranking.
///
/// Algorithm:
/// - Group projects by contractor.
/// - Filter out contractors with fewer than 5 projects.
/// - For each contractor, compute:
///   * TotalCost = sum of contract_cost
///   * NumProjects = project count
///   * AvgDelay = mean of completion delays
///   * TotalSavings = sum of cost_savings
///   * ReliabilityIndex = (1 - AvgDelay/90) * (TotalSavings/TotalCost) * 100,
///     clamped only on the upper bound (can be negative).
/// - Sort contractors by TotalCost descending and take the top 15.
pub fn generate_report2(data: &[CleanRecord]) -> Vec<ContractorRankingRow> {
    #[derive(Default)]
    struct Acc {
        projects: usize,
        delays: Vec<f64>,
        total_savings: f64,
        total_cost: f64,
    }
    let mut map: HashMap<String, Acc> = HashMap::new();
    for r in data {
        let e = map.entry(r.contractor.clone()).or_default();
        e.projects += 1;
        e.delays.push(r.completion_delay_days);
        e.total_savings += r.cost_savings;
        e.total_cost += r.contract_cost;
    }
    // Turn the map into a flat list of tuples so we can sort by
    // total_cost while keeping all derived metrics together.
    let mut tmp: Vec<(f64, String, usize, f64, f64, f64)> = map
        .into_iter()
        .filter(|(_, v)| v.projects >= 5)
        .map(|(k, v)| {
            let avg_delay = average(&v.delays);
            let mut reliability =
                (1.0 - (avg_delay / 90.0)) * (v.total_savings / v.total_cost) * 100.0;
            if !reliability.is_finite() {
                reliability = 0.0;
            }
            if reliability > 100.0 {
                reliability = 100.0;
            } // only cap upper bound
            (
                v.total_cost,
                k,
                v.projects,
                avg_delay,
                v.total_savings,
                reliability,
            )
        })
        .collect();
    // Sort descending by total contract cost and keep only the top 15.
    tmp.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let mut rows: Vec<ContractorRankingRow> = Vec::new();
    for (idx, (total_cost, contractor, projects, avg_delay, total_savings, reliability)) in
        tmp.into_iter().take(15).enumerate()
    {
        rows.push(ContractorRankingRow {
            rank: idx + 1,
            contractor,
            total_cost: format!("{:.2}", total_cost),
            num_projects: projects,
            avg_delay: format!("{:.2}", avg_delay),
            total_savings: format!("{:.2}", total_savings),
            reliability_index: format!("{:.2}", reliability),
            risk_flag: if reliability < 50.0 {
                "High Risk".to_string()
            } else {
                "OK".to_string()
            },
        });
    }
    rows
}

/// Generate Report 3: Annual Project Type Cost Overrun Trends.
///
/// Algorithm:
/// - Group projects by (FundingYear, TypeOfWork).
/// - For each group, compute:
///   * TotalProjects
///   * AvgSavings (average of cost_savings)
///   * OverrunRate (% of projects with negative savings).
/// - Separately maintain a per-year weighted average of savings across
///   all types: (sum of savings) / (total project count).
/// - Take 2021's weighted average as the baseline and compute a
///   YoYChange for each year relative to that baseline.
/// - Sort rows by FundingYear ascending, then AvgSavings descending.
pub fn generate_report3(data: &[CleanRecord]) -> Vec<TypeTrendRow> {
    #[derive(Default)]
    struct Acc {
        year: i32,
        tow: String,
        savings: Vec<f64>,
    }
    let mut map: HashMap<(i32, String), Acc> = HashMap::new();
    for r in data {
        let key = (r.funding_year, r.type_of_work.clone());
        let e = map.entry(key.clone()).or_insert_with(|| Acc {
            year: key.0,
            tow: key.1.clone(),
            savings: vec![],
        });
        e.savings.push(r.cost_savings);
    }

    // We track a numeric average per (year, type) plus formatted fields.
    // The numeric average is stored alongside the row for sorting and
    // YoY calculations.
    let mut rows_num: Vec<(i32, f64, TypeTrendRow)> = Vec::new();
    for acc in map.into_values() {
        let avg = average(&acc.savings);
        let total_projects = acc.savings.len();
        let overrun_rate = if acc.savings.is_empty() {
            0.0
        } else {
            (acc.savings.iter().filter(|s| **s < 0.0).count() as f64 / acc.savings.len() as f64)
                * 100.0
        };
        let row = TypeTrendRow {
            funding_year: acc.year,
            type_of_work: acc.tow,
            total_projects,
            avg_savings: format!("{:.2}", avg),
            overrun_rate: format!("{:.2}", overrun_rate),
            yoy_change: String::new(), // fill later
        };
        rows_num.push((row.funding_year, avg, row));
    }

    // Build a per-TypeOfWork baseline from 2021 averages, mirroring the
    // JavaScript implementation's `baselineByType`.
    let mut baseline_by_type: HashMap<String, f64> = HashMap::new();
    for (year, avg_val, row) in &rows_num {
        if *year == 2021 {
            baseline_by_type
                .entry(row.type_of_work.clone())
                .or_insert(*avg_val);
        }
    }

    // Compute YoY change per (year, type) using that type's 2021
    // baseline. If there is no baseline or it is zero, YoYChange is 0.00.
    let mut rows_with_avg: Vec<(i32, f64, TypeTrendRow)> = rows_num
        .into_iter()
        .map(|(year, avg_val, mut row)| {
            let baseline = baseline_by_type
                .get(&row.type_of_work)
                .copied()
                .unwrap_or(0.0);
            let change = if year == 2021 || baseline.abs() < f64::EPSILON {
                0.0
            } else {
                ((avg_val - baseline) / baseline.abs()) * 100.0
            };
            row.yoy_change = format!("{:.2}", change);
            (year, avg_val, row)
        })
        .collect();

    // Sort by FundingYear ascending, then by AvgSavings (numeric) descending.
    // a.0 and b.0 are the funding years; a.1 and b.1 are the numeric
    // average savings used purely for sorting (the formatted string lives
    // inside the `TypeTrendRow`).
    rows_with_avg.sort_by(|a, b| {
        a.0.cmp(&b.0).then_with(|| {
            b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal)
        })
    });

    rows_with_avg.into_iter().map(|(_, _, row)| row).collect()
}

/// Generate high-level summary statistics over all cleaned records.
pub fn generate_summary(
    data: &[CleanRecord],
    contractors: &[ContractorRankingRow],
) -> SummaryStats {
    let total_projects = data.len();
    let total_contractors = contractors.len();
    let provinces: HashSet<&str> = data.iter().map(|r| r.province.as_str()).collect();
    let avg_global_delay = average(
        &data
            .iter()
            .map(|r| r.completion_delay_days)
            .collect::<Vec<_>>(),
    );
    let total_savings: f64 = data.iter().map(|r| r.cost_savings).sum();
    SummaryStats {
        total_projects,
        total_contractors,
        total_provinces: provinces.len(),
        global_avg_delay_days: format_number(avg_global_delay, 2),
        total_savings: format_number(total_savings, 2),
        report1_regions: 0,      // filled by caller if needed
        report2_contractors: 0,  // filled by caller if needed
        report3_entries: 0,      // filled by caller if needed
    }
}
