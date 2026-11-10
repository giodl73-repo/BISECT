//! Historical native tract CSV semantics, shared with browser inspection.
use std::collections::HashMap;
use crate::{economic_character::LodesWacRaw,housing_character::HousingChar};

pub fn parse_lodes_wac_raw(content:&str,source_name:&str)->Result<HashMap<String,LodesWacRaw>,String>{
    let mut map = HashMap::new();
    let mut lines = content.lines();

    // Parse header
    let header_line = match lines.next() {
        Some(h) => h,
        None => return Ok(HashMap::new()),
    };
    let headers: Vec<&str> = header_line.split(',').map(str::trim).collect();

    // Locate required column indices
    let col = |name: &str| -> Result<usize, String> {
        headers
            .iter()
            .position(|&h| h.eq_ignore_ascii_case(name))
            .ok_or_else(|| format!("LODES WAC: column '{name}' not found in {}", source_name))
    };

    let idx_geoid = col("geoid")?;
    let idx_c000 = col("c000")?;
    let idx_cns07 = col("cns07")?;
    let idx_cns09 = col("cns09")?;
    let idx_cns10 = col("cns10")?;
    let idx_cns11 = col("cns11")?;
    let idx_cns01 = col("cns01")?;
    let idx_cns02 = col("cns02")?;
    let idx_cns05 = col("cns05")?;
    let idx_cns08 = col("cns08")?;

    for (lineno, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();

        let get = |idx: usize| -> Result<f64, String> {
            fields
                .get(idx)
                .ok_or_else(|| {
                    format!(
                        "LODES WAC {}: row {} has too few columns",
                        source_name,
                        lineno + 2
                    )
                })?
                .parse::<f64>()
                .map_err(|e| {
                    format!(
                        "LODES WAC {}: row {} column {idx} parse error: {e}",
                        source_name,
                        lineno + 2
                    )
                })
        };

        let geoid = fields
            .get(idx_geoid)
            .ok_or_else(|| {
                format!(
                    "LODES WAC {}: row {} missing geoid",
                    source_name,
                    lineno + 2
                )
            })?
            .to_string();

        map.insert(
            geoid,
            LodesWacRaw {
                c000: get(idx_c000)?,
                cns01: get(idx_cns01)?,
                cns02: get(idx_cns02)?,
                cns05: get(idx_cns05)?,
                cns07: get(idx_cns07)?,
                cns08: get(idx_cns08)?,
                cns09: get(idx_cns09)?,
                cns10: get(idx_cns10)?,
                cns11: get(idx_cns11)?,
            },
        );
    }

    Ok(map)
}

pub fn parse_housing_tract(content:&str,source_name:&str)->Result<HashMap<String,HousingChar>,String>{
    let mut map = HashMap::new();
    let mut lines = content.lines();
    let header_line = match lines.next() {
        Some(h) => h,
        None => return Ok(HashMap::new()),
    };
    let headers: Vec<&str> = header_line.split(',').map(str::trim).collect();
    let col = |name: &str| -> Result<usize, String> {
        headers
            .iter()
            .position(|&h| h.eq_ignore_ascii_case(name))
            .ok_or_else(|| {
                format!(
                    "ACS housing: column '{name}' not found in {}",
                    source_name
                )
            })
    };

    let idx_geoid = col("geoid")?;
    let idx_sf = col("pct_single_family").or_else(|_| col("pct_sf"))?;
    let idx_mf = col("pct_multifamily").or_else(|_| col("pct_mf"))?;
    let idx_owner = col("pct_owner")?;
    let idx_vintage = col("housing_vintage")?;

    for (lineno, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        let get = |idx: usize| -> Result<f64, String> {
            fields
                .get(idx)
                .ok_or_else(|| {
                    format!(
                        "ACS housing {}: row {} has too few columns",
                        source_name,
                        lineno + 2
                    )
                })?
                .parse::<f64>()
                .map_err(|e| {
                    format!(
                        "ACS housing {}: row {} column {idx} parse error: {e}",
                        source_name,
                        lineno + 2
                    )
                })
        };
        let geoid = fields
            .get(idx_geoid)
            .ok_or_else(|| {
                format!(
                    "ACS housing {}: row {} missing geoid",
                    source_name,
                    lineno + 2
                )
            })?
            .to_string();
        map.insert(
            geoid,
            HousingChar {
                pct_single_family: get(idx_sf)?.clamp(0.0, 1.0),
                pct_multifamily: get(idx_mf)?.clamp(0.0, 1.0),
                pct_owner: get(idx_owner)?.clamp(0.0, 1.0),
                housing_vintage: get(idx_vintage)?.clamp(0.0, 1.0),
            },
        );
    }

    Ok(map)
}
