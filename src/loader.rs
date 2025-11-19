// Data loading and cleaning pipeline.
//
// This module is responsible for:
// - reading the raw CSV file using the `csv` crate,
// - deserializing rows into `RawRow`,
// - validating and transforming them into `CleanRecord`, and
// - tracking basic statistics about parsing/imputation.
use crate::types::{CleanRecord, RawRow};
use crate::util::{days_diff, parse_date_safe, parse_f64_safe, parse_i32_safe};
use chrono::NaiveDate;
use csv::ReaderBuilder;
use std::collections::HashMap;
use std::error::Error;

/// Summary of what happened while loading and cleaning the CSV.
///
/// This is used to print user-friendly diagnostics after option `[1]`:
/// how many rows were seen, how many made it through filtering, and how
/// many required coordinate imputation.
#[derive(Debug, Clone)]
pub struct LoadReport {
    pub total_rows: usize,
    pub filtered_rows: usize,
    pub parse_errors: usize,
    pub imputed_coords: usize,
}

/// Load the CSV at `path`, validate and enrich each row, and return a
/// vector of `CleanRecord` plus a `LoadReport`.
///
/// The high-level algorithm is:
/// 1. Stream-deserialize `RawRow` values using `csv::Reader`.
/// 2. For each row, validate funding year, numeric fields, and dates.
/// 3. Compute derived metrics (cost savings, completion delay).
/// 4. Attempt to fill missing coordinates, first from project, then from
///    provincial capital, then later via province-level averages.
/// 5. Drop rows that fail validation and increment `parse_errors`.
pub fn load_and_clean(path: &str) -> Result<(Vec<CleanRecord>, LoadReport), Box<dyn Error>> {
    // `flexible(true)` lets the reader tolerate rows with varying column
    // counts instead of failing hard on minor format issues.
    let mut rdr = ReaderBuilder::new().flexible(true).from_path(path)?;
    let mut total_rows = 0usize;
    let mut parse_errors = 0usize;
    let mut prelim: Vec<CleanRecord> = Vec::new();

    // Stream over the CSV rows; each `result` is a `Result<RawRow, _>`.
    for result in rdr.deserialize::<RawRow>() {
        total_rows += 1;
        let row = match result {
            Ok(r) => r,
            Err(_) => {
                parse_errors += 1;
                continue;
            }
        };

        // Filter FundingYear 2021..=2023
        let funding_year = match parse_i32_safe(row.funding_year.as_deref()) {
            Some(y) if (2021..=2023).contains(&y) => y,
            _ => continue,
        };

        let approved_budget = match parse_f64_safe(row.approved_budget_for_contract.as_deref()) {
            Some(v) if v > 0.0 => v,
            _ => {
                parse_errors += 1;
                continue;
            }
        };
        let contract_cost = match parse_f64_safe(row.contract_cost.as_deref()) {
            Some(v) if v > 0.0 => v,
            _ => {
                parse_errors += 1;
                continue;
            }
        };
        // Both `StartDate` and `ActualCompletionDate` are required to
        // compute a completion delay. Missing start dates are treated as
        // fatal parse errors; missing completion dates are imputed from
        // the start date 
        let start_date: NaiveDate = match parse_date_safe(row.start_date.as_deref()) {
            Some(d) => d,
            None => {
                parse_errors += 1;
                continue;
            }
        };
        let actual_date: NaiveDate = match parse_date_safe(row.actual_completion_date.as_deref()) {
            Some(d) => d,
            None => start_date,
        };

        // Derived metrics:
        // - `completion_delay_days` is the raw day difference.
        // - `cost_savings` is ApprovedBudget - ContractCost.
        let completion_delay_days = days_diff(start_date, actual_date);
        let cost_savings = approved_budget - contract_cost;

        let region = row
            .region
            .unwrap_or_else(|| "Unknown".to_string())
            .trim()
            .to_string();
        let main_island = row
            .main_island
            .unwrap_or_else(|| "Unknown".to_string())
            .trim()
            .to_string();
        let province = row
            .province
            .unwrap_or_else(|| "Unknown".to_string())
            .trim()
            .to_string();
        let type_of_work = row
            .type_of_work
            .unwrap_or_else(|| "Unspecified".to_string())
            .trim()
            .to_string();
        let contractor = row
            .contractor
            .unwrap_or_else(|| "Unknown Contractor".to_string())
            .trim()
            .to_string();

        // Prefer explicit project coordinates, but fall back to the
        // provincial capital coordinates if needed.
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
            completion_delay_days,
            lat,
            lon,
        });
    }

    // Province-level averages imputation if coordinates are still
    // missing: compute (sum_lat, sum_lon, count) per province.
    let mut by_prov: HashMap<String, (f64, f64, usize)> = HashMap::new();
    for r in &prelim {
        if let (Some(lat), Some(lon)) = (r.lat, r.lon) {
            let e = by_prov.entry(r.province.clone()).or_insert((0.0, 0.0, 0));
            e.0 += lat;
            e.1 += lon;
            e.2 += 1;
        }
    }

    let mut imputed_coords = 0usize;
    for r in &mut prelim {
        if r.lat.is_none() || r.lon.is_none() {
            if let Some((s_lat, s_lon, c)) = by_prov.get(&r.province) {
                if *c > 0 {
                    r.lat = r.lat.or(Some(s_lat / *c as f64));
                    r.lon = r.lon.or(Some(s_lon / *c as f64));
                    imputed_coords += 1;
                }
            }
        }
    }

    let filtered_rows = prelim.len();
    let report = LoadReport {
        total_rows,
        filtered_rows,
        parse_errors,
        imputed_coords,
    };
    Ok((prelim, report))
}
