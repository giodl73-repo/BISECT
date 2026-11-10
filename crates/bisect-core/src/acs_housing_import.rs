//! Pure native ACS API response-to-housing CSV conversion.
pub const HEADER:&str="geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage\n";
#[derive(serde::Serialize)]
pub struct ImportReport {pub csv_text:String,pub written_rows:usize,pub short_rows:usize,pub malformed_fips:Vec<String>,pub empty_response:bool}
pub fn convert(body:&str)->Result<ImportReport,String>{
    // ACS API returns a JSON array-of-arrays: first row is header, rest are data.
    let rows: Vec<Vec<serde_json::Value>> =
        serde_json::from_str(body).map_err(|e| format!("ACS JSON parse: {e}"))?;

    if rows.is_empty() { return Ok(ImportReport {csv_text: HEADER.into(), written_rows:0, short_rows:0, malformed_fips:vec![], empty_response:true}); }

    // Build column index from header row.
    let header_row: Vec<String> = rows[0]
        .iter()
        .map(|v| v.as_str().unwrap_or("").to_string())
        .collect();
    let find_col = |name: &str| -> Result<usize, String> {
        header_row
            .iter()
            .position(|c| c == name)
            .ok_or_else(|| format!("ACS housing: column '{name}' not found in response header"))
    };

    let i_total_units = find_col("B25024_001E")?;
    let i_sf_det = find_col("B25024_002E")?;
    let i_sf_att = find_col("B25024_003E")?;
    let i_mf_1019 = find_col("B25024_007E")?;
    let i_mf_2049 = find_col("B25024_008E")?;
    let i_mf_50p = find_col("B25024_009E")?;
    let i_occ_tot = find_col("B25003_001E")?;
    let i_occ_own = find_col("B25003_002E")?;
    let i_vintage = find_col("B25035_001E")?;
    let i_state = find_col("state")?;
    let i_county = find_col("county")?;
    let i_tract = find_col("tract")?;

    // Helper: parse a JSON value as f64, returning 0.0 on null/error.
    let parse_f64 = |v: &serde_json::Value| -> f64 {
        match v {
            serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
            serde_json::Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
            _ => 0.0,
        }
    };

    let mut out = String::from(HEADER);
    use std::fmt::Write;
    let mut short_rows=0;
    let mut malformed_fips=Vec::new();
    let last_required=[i_total_units,i_sf_det,i_sf_att,i_mf_1019,i_mf_2049,i_mf_50p,i_occ_tot,i_occ_own,i_vintage,i_state,i_county,i_tract].into_iter().max().unwrap();
    let mut row_count = 0usize;
    for row in rows.iter().skip(1) {
        if row.len() <= last_required {
            short_rows+=1;
            continue;
        }

        // Build 11-char GEOID: state(2) + county(3) + tract(6)
        let state_fips = row[i_state].as_str().unwrap_or("");
        let county_fips = row[i_county].as_str().unwrap_or("");
        let tract_code = row[i_tract].as_str().unwrap_or("");
        if state_fips.len() != 2 || county_fips.len() != 3 || tract_code.len() != 6 {
            malformed_fips.push(format!("[WARN] ACS housing: skipping row with malformed FIPS ({state_fips}/{county_fips}/{tract_code})"));
            continue;
        }
        let geoid = format!("{state_fips}{county_fips}{tract_code}");

        let housing = crate::housing_character::derive_housing_character(crate::housing_character::AcsHousingRaw {
            total_units: parse_f64(&row[i_total_units]),
            single_family_detached: parse_f64(&row[i_sf_det]),
            single_family_attached: parse_f64(&row[i_sf_att]),
            multifamily_10_19: parse_f64(&row[i_mf_1019]),
            multifamily_20_49: parse_f64(&row[i_mf_2049]),
            multifamily_50_plus: parse_f64(&row[i_mf_50p]),
            occupied_total: parse_f64(&row[i_occ_tot]),
            owner_occupied: parse_f64(&row[i_occ_own]),
            median_year_built: parse_f64(&row[i_vintage]),
        });

        writeln!(
            out,
            "{geoid},{:.6},{:.6},{:.6},{:.6}",
            housing.pct_single_family,
            housing.pct_multifamily,
            housing.pct_owner,
            housing.housing_vintage
        )
        .map_err(|e| e.to_string())?;
        row_count += 1;
    }

    Ok(ImportReport {csv_text:out,written_rows:row_count,short_rows,malformed_fips,empty_response:false})
}

#[cfg(test)] mod tests {
    use super::*;
    fn header()->Vec<&'static str>{vec!["B25024_001E","B25024_002E","B25024_003E","B25024_007E","B25024_008E","B25024_009E","B25003_001E","B25003_002E","B25035_001E","state","county","tract"]}
    #[test] fn native_csv_formulas_and_order(){
        let h=header();let body=serde_json::json!([h,["100","40","10","10","20","10","80","60","1980","44","001","000101"],["0","0","0","0","0","0","0","0","-666666666","44","001","000102"]]);
        let report=convert(&body.to_string()).unwrap();assert_eq!(report.written_rows,2);assert_eq!(report.csv_text,format!("{HEADER}44001000101,0.500000,0.400000,0.750000,0.500000\n44001000102,0.500000,0.500000,0.500000,0.500000\n"));
    }
    #[test] fn reordered_short_rows_are_skipped_without_indexing_panic(){
        let mut h=header();h.rotate_right(1);let report=convert(&serde_json::json!([h,["000101"]]).to_string()).unwrap();assert_eq!(report.short_rows,1);assert_eq!(report.csv_text,HEADER);
    }
    #[test] fn empty_missing_headers_and_malformed_fips(){
        assert!(convert("[]").unwrap().empty_response);assert!(convert("[[]]").is_err());
        let h=header();let report=convert(&serde_json::json!([h,["1","1","0","0","0","0","1","1","2000","4","001","000101"]]).to_string()).unwrap();assert_eq!(report.malformed_fips.len(),1);assert_eq!(report.written_rows,0);
    }
}
