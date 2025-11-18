use crate::types::{CleanRecord, ContractorRankingRow, RegionSummaryRow, SummaryStats, TypeTrendRow};
use crate::util::{average, clamp_0_100, format_number, median};
use std::collections::{BTreeMap, HashMap, HashSet};

pub fn generate_report1(data: &[CleanRecord]) -> Vec<RegionSummaryRow> {
    #[derive(Default)]
    struct Acc { budgets: Vec<f64>, savings: Vec<f64>, delays: Vec<f64>, region: String, island: String }
    let mut map: HashMap<(String, String), Acc> = HashMap::new();
    for r in data {
        let key = (r.region.clone(), r.main_island.clone());
        let e = map.entry(key.clone()).or_insert_with(|| Acc { budgets: vec![], savings: vec![], delays: vec![], region: key.0.clone(), island: key.1.clone() });
        e.budgets.push(r.approved_budget);
        e.savings.push(r.cost_savings);
        e.delays.push(r.completion_delay_days);
    }
    let mut rows: Vec<(f64, RegionSummaryRow)> = map.into_values().map(|acc| {
        let avg_delay = average(&acc.delays);
        let delay_over_30 = if acc.delays.is_empty() { 0.0 } else { (acc.delays.iter().filter(|d| **d > 30.0).count() as f64 / acc.delays.len() as f64) * 100.0 };
        let med_savings = median(acc.savings.clone());
        let mut eff = if avg_delay <= 0.0 { 0.0 } else { (med_savings / avg_delay) * 100.0 };
        eff = clamp_0_100(eff);
        let total_budget: f64 = acc.budgets.iter().sum();
        let row = RegionSummaryRow {
            region: acc.region,
            main_island: acc.island,
            total_approved_budget: format_number(total_budget, 2),
            median_cost_savings: format_number(med_savings, 2),
            avg_completion_delay_days: format_number(avg_delay, 2),
            delay_over_30_percent: format_number(delay_over_30, 2),
            efficiency_score: format_number(eff, 2),
        };
        (eff, row)
    }).collect();
    rows.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    rows.into_iter().map(|(_, r)| r).collect()
}

pub fn generate_report2(data: &[CleanRecord]) -> Vec<ContractorRankingRow> {
    #[derive(Default)]
    struct Acc { projects: usize, delays: Vec<f64>, total_savings: f64, total_cost: f64 }
    let mut map: HashMap<String, Acc> = HashMap::new();
    for r in data {
        let e = map.entry(r.contractor.clone()).or_default();
        e.projects += 1;
        e.delays.push(r.completion_delay_days);
        e.total_savings += r.cost_savings;
        e.total_cost += r.contract_cost;
    }
    let mut tmp: Vec<(f64, String, usize, f64, f64, f64)> = map.into_iter()
        .filter(|(_, v)| v.projects >= 5)
        .map(|(k, v)| {
            let avg_delay = average(&v.delays);
            let mut reliability = (1.0 - (avg_delay / 90.0)) * (v.total_savings / v.total_cost) * 100.0;
            if !reliability.is_finite() { reliability = 0.0; }
            reliability = clamp_0_100(reliability);
            (v.total_cost, k, v.projects, avg_delay, v.total_savings, reliability)
        })
        .collect();
    tmp.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let mut rows: Vec<ContractorRankingRow> = Vec::new();
    for (idx, (total_cost, contractor, projects, avg_delay, total_savings, reliability)) in tmp.into_iter().take(15).enumerate() {
        rows.push(ContractorRankingRow {
            rank: idx + 1,
            contractor,
            total_cost: format_number(total_cost, 2),
            num_projects: projects,
            avg_delay: format_number(avg_delay, 2),
            total_savings: format_number(total_savings, 2),
            reliability_index: format_number(reliability, 2),
            risk_flag: if reliability < 50.0 { "High Risk".to_string() } else { "OK".to_string() },
        });
    }
    rows
}

pub fn generate_report3(data: &[CleanRecord]) -> Vec<TypeTrendRow> {
    #[derive(Default)]
    struct Acc { year: i32, tow: String, savings: Vec<f64> }
    let mut map: HashMap<(i32, String), Acc> = HashMap::new();
    for r in data {
        let key = (r.funding_year, r.type_of_work.clone());
        let e = map.entry(key.clone()).or_insert_with(|| Acc { year: key.0, tow: key.1.clone(), savings: vec![] });
        e.savings.push(r.cost_savings);
    }

    let mut avg_by_year: BTreeMap<i32, Vec<f64>> = BTreeMap::new();
    let mut rows_num: Vec<(i32, f64, TypeTrendRow)> = Vec::new();
    for acc in map.into_values() {
        let avg = average(&acc.savings);
        avg_by_year.entry(acc.year).or_default().push(avg);
        let overrun_rate = if acc.savings.is_empty() { 0.0 } else { (acc.savings.iter().filter(|s| **s < 0.0).count() as f64 / acc.savings.len() as f64) * 100.0 };
        let row = TypeTrendRow {
            funding_year: acc.year,
            type_of_work: acc.tow,
            total_projects: acc.savings.len(),
            avg_cost_savings: format_number(avg, 2),
            overrun_rate: format_number(overrun_rate, 2),
            yoy_change_percent: String::new(), // fill later
        };
        rows_num.push((row.funding_year, avg, row));
    }

    let baseline = average(avg_by_year.get(&2021).map(|v| v.as_slice()).unwrap_or(&[0.0]));
    let mut rows: Vec<TypeTrendRow> = rows_num.into_iter().map(|(year, _avg, mut row)| {
        let year_avg = average(avg_by_year.get(&year).map(|v| v.as_slice()).unwrap_or(&[0.0]));
        let change = if baseline.abs() < 1e-9 || year == 2021 { 0.0 } else { ((year_avg - baseline) / baseline.abs()) * 100.0 };
        row.yoy_change_percent = format!("{:.2}", change);
        row
    }).collect();

    rows.sort_by(|a, b| a.funding_year.cmp(&b.funding_year).then_with(|| b.avg_cost_savings.cmp(&a.avg_cost_savings)));
    rows
}

pub fn generate_summary(data: &[CleanRecord], contractors: &[ContractorRankingRow]) -> SummaryStats {
    let total_projects = data.len();
    let total_contractors = contractors.len();
    let provinces: HashSet<&str> = data.iter().map(|r| r.province.as_str()).collect();
    let avg_global_delay = average(&data.iter().map(|r| r.completion_delay_days).collect::<Vec<_>>());
    let total_savings: f64 = data.iter().map(|r| r.cost_savings).sum();
    SummaryStats { total_projects, total_contractors, total_provinces: provinces.len(), avg_global_delay, total_savings }
}
