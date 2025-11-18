use crate::types::{CleanRecord, RawRow};
use crate::util::{days_diff, parse_date_safe, parse_f64_safe, parse_i32_safe};
use chrono::NaiveDate;
use csv::ReaderBuilder;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct LoadReport {
    pub total_rows: usize,
    pub filtered_rows: usize,
    pub parse_errors: usize,
    pub imputed_coords: usize,
}

pub fn load_and_clean(path: &str) -> Result<(Vec<CleanRecord>, LoadReport), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().flexible(true).from_path(path)?;
    let mut total_rows = 0usize;
    let mut parse_errors = 0usize;
    let mut prelim: Vec<CleanRecord> = Vec::new();

    for result in rdr.deserialize::<RawRow>() {
        total_rows += 1;
        let row = match result {
            Ok(r) => r,
            Err(_) => { parse_errors += 1; continue; }
        };

        // Filter FundingYear 2021..=2023
        let funding_year = match parse_i32_safe(row.funding_year.as_deref()) {
            Some(y) if (2021..=2023).contains(&y) => y,
            _ => continue,
        };

        let approved_budget = match parse_f64_safe(row.approved_budget_for_contract.as_deref()) { Some(v) if v > 0.0 => v, _ => { parse_errors += 1; continue; } };
        let contract_cost = match parse_f64_safe(row.contract_cost.as_deref()) { Some(v) if v > 0.0 => v, _ => { parse_errors += 1; continue; } };
        let start_date: NaiveDate = match parse_date_safe(row.start_date.as_deref()) { Some(d) => d, None => { parse_errors += 1; continue; } };
        let actual_date: NaiveDate = match parse_date_safe(row.actual_completion_date.as_deref()) { Some(d) => d, None => start_date };

        let completion_delay_days = days_diff(start_date, actual_date);
        let cost_savings = approved_budget - contract_cost;

        let region = row.region.unwrap_or_else(|| "Unknown".to_string()).trim().to_string();
        let main_island = row.main_island.unwrap_or_else(|| "Unknown".to_string()).trim().to_string();
        let province = row.province.unwrap_or_else(|| "Unknown".to_string()).trim().to_string();
        let type_of_work = row.type_of_work.unwrap_or_else(|| "Unspecified".to_string()).trim().to_string();
        let contractor = row.contractor.unwrap_or_else(|| "Unknown Contractor".to_string()).trim().to_string();

        let mut lat = parse_f64_safe(row.project_latitude.as_deref());
        let mut lon = parse_f64_safe(row.project_longitude.as_deref());
        if lat.is_none() || lon.is_none() {
            // Try provincial capital
            if let (Some(clat), Some(clon)) = (
                parse_f64_safe(row.provincial_capital_latitude.as_deref()),
                parse_f64_safe(row.provincial_capital_longitude.as_deref()),
            ) {
                lat = lat.or(Some(clat));
                lon = lon.or(Some(clon));
            }
        }

        prelim.push(CleanRecord {
            funding_year,
            region,
            main_island,
            province,
            type_of_work,
            contractor,
            approved_budget,
            contract_cost,
            cost_savings,
            start_date,
            actual_completion_date: actual_date,
            completion_delay_days,
            lat,
            lon,
        });
    }

    // Province averages for imputation if still missing
    let mut by_prov: HashMap<String, (f64, f64, usize)> = HashMap::new();
    for r in &prelim {
        if let (Some(lat), Some(lon)) = (r.lat, r.lon) {
            let e = by_prov.entry(r.province.clone()).or_insert((0.0, 0.0, 0));
            e.0 += lat; e.1 += lon; e.2 += 1;
        }
    }

    let mut imputed_coords = 0usize;
    for r in &mut prelim {
        if r.lat.is_none() || r.lon.is_none() {
            if let Some((s_lat, s_lon, c)) = by_prov.get(&r.province) {
                if *c > 0 { r.lat = r.lat.or(Some(s_lat / *c as f64)); r.lon = r.lon.or(Some(s_lon / *c as f64)); imputed_coords += 1; }
            }
        }
    }

    let filtered_rows = prelim.len();
    let report = LoadReport { total_rows, filtered_rows, parse_errors, imputed_coords };
    Ok((prelim, report))
}
