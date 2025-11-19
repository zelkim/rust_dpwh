use crate::types::{
    CleanRecord, ContractorRankingRow, RegionSummaryRow, SummaryStats, TypeTrendRow,
};
use crate::util::{average, format_number, median};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

pub fn generate_report1(data: &[CleanRecord]) -> Vec<RegionSummaryRow> {
    #[derive(Default)]
    struct Acc {
        budgets: Vec<f64>,
        savings: Vec<f64>,
        delays: Vec<f64>,
        region: String,
        island: String,
    }
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
                total_budget: format_number(total_budget, 2),
                median_savings: format_number(med_savings, 2),
                avg_delay: format_number(avg_delay, 2),
                high_delay_pct: format_number(delay_over_30, 2),
                raw_efficiency: eff,
            }
        })
        .collect();
    if prepared.is_empty() {
        return Vec::new();
    }

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
                efficiency_score: format_number(scaled, 2),
            };
            (scaled, rendered)
        })
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
    scored.into_iter().map(|(_, row)| row).collect()
}

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
    tmp.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let mut rows: Vec<ContractorRankingRow> = Vec::new();
    for (idx, (total_cost, contractor, projects, avg_delay, total_savings, reliability)) in
        tmp.into_iter().take(15).enumerate()
    {
        rows.push(ContractorRankingRow {
            rank: idx + 1,
            contractor,
            total_cost: format_number(total_cost, 2),
            num_projects: projects,
            avg_delay: format_number(avg_delay, 2),
            total_savings: format_number(total_savings, 2),
            reliability_index: format_number(reliability, 2),
            risk_flag: if reliability < 50.0 {
                "High Risk".to_string()
            } else {
                "OK".to_string()
            },
        });
    }
    rows
}

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

    let mut yearly_totals: HashMap<i32, (f64, usize)> = HashMap::new();
    let mut rows_num: Vec<(i32, f64, TypeTrendRow)> = Vec::new();
    for acc in map.into_values() {
        let avg = average(&acc.savings);
        let total_projects = acc.savings.len();
        let sum_savings: f64 = acc.savings.iter().sum();
        let entry = yearly_totals.entry(acc.year).or_insert((0.0, 0));
        entry.0 += sum_savings;
        entry.1 += total_projects;
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
            avg_savings: format_number(avg, 2),
            overrun_rate: format_number(overrun_rate, 2),
            yoy_change: String::new(), // fill later
        };
        rows_num.push((row.funding_year, avg, row));
    }

    let baseline = yearly_totals
        .get(&2021)
        .map(|(total, count)| {
            if *count > 0 {
                *total / *count as f64
            } else {
                0.0
            }
        })
        .unwrap_or(0.0);
    let denom = if baseline.abs() < f64::EPSILON {
        1.0
    } else {
        baseline.abs()
    };

    let mut rows_with_avg: Vec<(i32, f64, TypeTrendRow)> = rows_num
        .into_iter()
        .map(|(year, avg_val, mut row)| {
            let year_avg = yearly_totals
                .get(&year)
                .map(|(total, count)| {
                    if *count > 0 {
                        *total / *count as f64
                    } else {
                        0.0
                    }
                })
                .unwrap_or(0.0);
            let change = if year == 2021 {
                0.0
            } else {
                ((year_avg - baseline) / denom) * 100.0
            };
            row.yoy_change = format!("{:.2}", change);
            (year, avg_val, row)
        })
        .collect();

    rows_with_avg.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal))
    });

    rows_with_avg.into_iter().map(|(_, _, row)| row).collect()
}

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
        avg_global_delay,
        total_savings,
    }
}
