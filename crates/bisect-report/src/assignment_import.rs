/// Import assignments from GeoJSON properties (tract-level assignments embedded in features).
/// Each feature must have a "geoid" and "district" property.
/// District polygons alone require a separate centroid-based import workflow.
pub fn import_assignments_from_geojson_properties(
    geojson_str: &str,
) -> anyhow::Result<std::collections::HashMap<String, usize>> {
    let v: serde_json::Value = serde_json::from_str(geojson_str)?;
    if v["type"]!="FeatureCollection" { anyhow::bail!("Expected GeoJSON FeatureCollection"); }

    // Check if this is a tract-level assignment GeoJSON (features have geoid property)
    let features = v["features"].as_array();
    if let Some(arr) = features {
        // Try to extract geoid → district from feature properties
        let mut assignments = std::collections::HashMap::new();
        for feature in arr {
            let props = &feature["properties"];
            let geoid = props["geoid"].as_str().or_else(|| props["GEOID"].as_str());
            if let Some(geoid) = geoid {
                let district = props["district"].as_u64().ok_or_else(|| {
                    anyhow::anyhow!(
                        "[INPUT] GeoJSON assignment for GEOID {geoid} is missing an integer district"
                    )
                })?;
                if assignments.insert(
                    geoid.to_string(),
                    validate_import_district(geoid, district)?,
                ).is_some() { anyhow::bail!("[INPUT] Duplicate GeoJSON unit {geoid}"); }
            } else { anyhow::bail!("[INPUT] GeoJSON feature lacks a geoid/GEOID assignment identifier; district polygons require tract centroids"); }
        }
        if !assignments.is_empty() {
            return Ok(assignments);
        }

        // If no tract assignments found: this is a district-polygon GeoJSON.
        // We can't do PIP without real tract centroids; return error with guidance.
        anyhow::bail!(
            "GeoJSON does not contain tract-level assignments (geoid + district properties). \
             For district-polygon GeoJSON, provide tract centroids via a supplementary file \
             or use the Python pipeline's import command."
        );
    }

    anyhow::bail!("Invalid GeoJSON: missing 'features' array")
}

/// Import assignments from Dave's Redistricting App CSV format.
///
/// DRA exports a two-column CSV:
///   Variant 1: GEOID,DISTRICT header  (GEOID = 11-char numeric, DISTRICT = small int)
///   Variant 2: DISTRICT,GEOID header  (reversed columns)
///   Variant 3: no header              (first column is detected as GEOID by length)
///
/// Returns HashMap<GEOID, district_id>.
pub fn import_assignments_from_dra_csv(
    csv_str: &str,
) -> anyhow::Result<std::collections::HashMap<String, usize>> {
    let mut lines = csv_str.lines().peekable();
    if lines.peek().is_none() {
        anyhow::bail!("DRA CSV is empty");
    }

    // Detect header row by checking whether the first token is numeric.
    let first_line = *lines.peek().unwrap();
    let first_fields: Vec<&str> = first_line.splitn(2, ',').collect();
    let has_header = !first_fields
        .get(0)
        .map(|f| f.trim())
        .unwrap_or("")
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false);

    // Determine column order from header (or default to GEOID=0, DISTRICT=1).
    let (geoid_col, district_col) = if has_header {
        let header_line = lines.next().unwrap(); // consume the header
        let headers: Vec<&str> = header_line.split(',').collect();
        let h0 = headers
            .get(0)
            .map(|h| h.trim().to_uppercase())
            .unwrap_or_default();
        let h1 = headers
            .get(1)
            .map(|h| h.trim().to_uppercase())
            .unwrap_or_default();
        // Detect reversed order: first column is DISTRICT (small-int-like name)
        let geoid_first = h0.contains("GEOID") || h0.contains("GEO_ID");
        let district_first = h0.contains("DISTRICT") || h0.contains("DIST") || h0 == "ID";
        if district_first && (h1.contains("GEOID") || h1.contains("GEO_ID")) {
            (1usize, 0usize) // reversed: DISTRICT, GEOID
        } else if geoid_first {
            (0, 1) // normal: GEOID, DISTRICT
        } else {
            // Unknown header order — try to auto-detect from first data row
            (0, 1)
        }
    } else {
        (0usize, 1usize) // no header: assume GEOID first
    };

    let mut assignments = std::collections::HashMap::new();
    for (lineno, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        let geoid_raw = fields.get(geoid_col).map(|f| f.trim()).unwrap_or("");
        let district_raw = fields.get(district_col).map(|f| f.trim()).unwrap_or("");

        // If auto-detect failed and geoid looks numeric with small value, swap
        let (geoid, district_str) = if geoid_raw.len() < 5
            && district_raw.len() == 11
            && district_raw.chars().all(|c| c.is_ascii_digit())
        {
            (district_raw, geoid_raw)
        } else {
            (geoid_raw, district_raw)
        };

        let district: usize = district_str.parse().map_err(|_| {
            anyhow::anyhow!(
                "DRA CSV line {}: cannot parse district '{}' as integer",
                lineno + 1,
                district_str
            )
        })?;
        if district == 0 {
            anyhow::bail!(
                "[INPUT] DRA CSV line {}: assignment for GEOID {} uses invalid district 0",
                lineno + 1,
                geoid
            );
        }
        if assignments.insert(geoid.to_string(), district).is_some() {
            anyhow::bail!("[INPUT] Duplicate CSV unit {geoid}");
        }
    }

    if assignments.is_empty() {
        anyhow::bail!("DRA CSV produced no assignments — check file format");
    }
    Ok(assignments)
}

fn validate_import_district(geoid: &str, district: u64) -> anyhow::Result<usize> {
    if district == 0 {
        anyhow::bail!("[INPUT] GeoJSON assignment for GEOID {geoid} uses invalid district 0");
    }
    usize::try_from(district)
        .map_err(|_| anyhow::anyhow!("[INPUT] district value {district} is too large"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csv_orders_and_headerless_match(){
        let expected=import_assignments_from_dra_csv("GEOID,DISTRICT\n53001000100,1\n53001000200,2\n").unwrap();
        for text in ["DISTRICT,GEOID\n1,53001000100\n2,53001000200\n","53001000100,1\n53001000200,2\n"]{
            assert_eq!(import_assignments_from_dra_csv(text).unwrap(),expected);
        }
    }
    #[test]
    fn duplicate_csv_unit_is_not_overwritten(){assert!(import_assignments_from_dra_csv("GEOID,DISTRICT\n53001000100,1\n53001000100,2\n").is_err());}
    #[test]
    fn duplicate_geojson_unit_is_not_overwritten(){
        assert!(import_assignments_from_geojson_properties(r#"{"type":"FeatureCollection","features":[{"properties":{"geoid":"53001000100","district":1}},{"properties":{"geoid":"53001000100","district":2}}]}"#).is_err());
    }
    #[test]
    fn missing_geojson_assignment_is_not_dropped(){
        assert!(import_assignments_from_geojson_properties(r#"{"type":"FeatureCollection","features":[{"properties":{"geoid":"53001000100","district":1}},{"properties":{"district_id":2}}]}"#).is_err());
    }
    #[test]
    fn uppercase_geojson_id_and_positive_district(){
        let a=import_assignments_from_geojson_properties(r#"{"type":"FeatureCollection","features":[{"properties":{"GEOID":"53001000100","district":1}}]}"#).unwrap();assert_eq!(a["53001000100"],1);
        assert!(import_assignments_from_geojson_properties(r#"{"type":"FeatureCollection","features":[{"properties":{"GEOID":"53001000100","district":0}}]}"#).is_err());
    }
}

