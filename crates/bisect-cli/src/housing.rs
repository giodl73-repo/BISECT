use std::collections::HashMap;

pub use bisect_core::housing_character::{cosine_similarity, derive_housing_character, AcsHousingRaw, HousingChar};

pub fn load_acs_housing_tract(
    state_name: &str,
    year: &str,
) -> Result<HashMap<String, HousingChar>, String> {
    let path = std::path::Path::new("data")
        .join(year)
        .join("acs_housing")
        .join(format!("{state_name}_housing_{year}.csv"));

    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("ACS housing read error {}: {e}", path.display()))?;

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
                    path.display()
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
                        path.display(),
                        lineno + 2
                    )
                })?
                .parse::<f64>()
                .map_err(|e| {
                    format!(
                        "ACS housing {}: row {} column {idx} parse error: {e}",
                        path.display(),
                        lineno + 2
                    )
                })
        };
        let geoid = fields
            .get(idx_geoid)
            .ok_or_else(|| {
                format!(
                    "ACS housing {}: row {} missing geoid",
                    path.display(),
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

pub fn align_housing_to_adjacency(
    chars: &HashMap<String, HousingChar>,
    index_to_geoid: &HashMap<usize, String>,
    n: usize,
) -> Vec<HousingChar> {
    let mut result = vec![HousingChar::neutral(); n];
    for (&idx, geoid) in index_to_geoid {
        if idx < n {
            if let Some(hc) = chars.get(geoid) {
                result[idx] = *hc;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn housing_character_formula_uses_total_units_and_10plus_multifamily() {
        let hc = derive_housing_character(AcsHousingRaw {
            total_units: 100.0,
            single_family_detached: 60.0,
            single_family_attached: 10.0,
            multifamily_10_19: 10.0,
            multifamily_20_49: 5.0,
            multifamily_50_plus: 5.0,
            occupied_total: 80.0,
            owner_occupied: 60.0,
            median_year_built: 1980.0,
        });
        assert!((hc.pct_single_family - 0.70).abs() < 1e-9);
        assert!((hc.pct_multifamily - 0.20).abs() < 1e-9);
        assert!((hc.pct_owner - 0.75).abs() < 1e-9);
        assert!((hc.housing_vintage - 0.50).abs() < 1e-9);
    }

    #[test]
    fn housing_character_defaults_missing_denominators_to_neutral() {
        let hc = derive_housing_character(AcsHousingRaw {
            total_units: 0.0,
            single_family_detached: 0.0,
            single_family_attached: 0.0,
            multifamily_10_19: 0.0,
            multifamily_20_49: 0.0,
            multifamily_50_plus: 0.0,
            occupied_total: 0.0,
            owner_occupied: 0.0,
            median_year_built: -666_666_666.0,
        });
        assert_eq!(hc, HousingChar::neutral());
    }

    #[test]
    fn housing_character_similarity_invariants() {
        let suburb = HousingChar {
            pct_single_family: 0.88,
            pct_multifamily: 0.02,
            pct_owner: 0.75,
            housing_vintage: 0.30,
        };
        let apartment = HousingChar {
            pct_single_family: 0.08,
            pct_multifamily: 0.72,
            pct_owner: 0.18,
            housing_vintage: 0.20,
        };
        let same = cosine_similarity(&suburb, &suburb);
        let cross = cosine_similarity(&suburb, &apartment);
        assert!((same - 1.0).abs() < 1e-9);
        assert!(cross < 0.35, "expected low housing similarity; got {cross}");
        assert!((cross - cosine_similarity(&apartment, &suburb)).abs() < 1e-12);
    }
}
