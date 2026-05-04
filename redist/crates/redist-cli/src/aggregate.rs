use std::path::PathBuf;
use crate::args::AggregateArgs;
use redist_analysis::AnalyzerType;

/// Analyzers that produce one record per STATE (not per district).
/// These need a different merge + CSV path than district-level analyzers.
fn is_state_level_analyzer(t: &AnalyzerType) -> bool {
    matches!(t, AnalyzerType::Proportionality)
}

pub fn run_aggregate(args: &AggregateArgs) -> anyhow::Result<()> {
    let year = args.year.to_string();
    let output_root = PathBuf::from("outputs").join(&args.version);
    let national_dir = output_root.join(&year).join("national");
    std::fs::create_dir_all(&national_dir)?;

    let types = resolve_types(&args.types);

    // Discover states with analysis outputs — year-specific path
    let states_dir = output_root.join(&year).join("states");
    if !states_dir.exists() {
        anyhow::bail!("No states directory at {}. Run: redist states --year {year} first.", states_dir.display());
    }
    let mut state_dirs: Vec<(String, PathBuf)> = std::fs::read_dir(&states_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
        .collect();
    state_dirs.sort_by_key(|(name, _)| name.clone());

    eprintln!("[aggregate] {} states in {}", state_dirs.len(), states_dir.display());

    for typ in &types {
        let type_name = typ.name();
        let out_path = national_dir.join(format!("us_{type_name}.json"));

        if out_path.exists() && !args.force {
            eprintln!("[skip] {type_name} (use --force to regenerate)");
            continue;
        }

        // Load per-state JSONs
        let mut state_data: Vec<(String, serde_json::Value)> = vec![];
        for (state_name, state_dir) in &state_dirs {
            let json_path = state_dir.join("analysis").join(format!("{type_name}.json"));
            if json_path.exists() {
                match std::fs::read_to_string(&json_path)
                    .and_then(|s| serde_json::from_str(&s).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
                {
                    Ok(v) => state_data.push((state_name.clone(), v)),
                    Err(e) => eprintln!("[warn] {state_name}/{type_name}.json: {e}"),
                }
            }
        }

        if state_data.is_empty() {
            eprintln!("[skip] {type_name} -- no state analysis files found. Run: redist analyze --types {type_name}");
            continue;
        }

        let refs: Vec<(&str, &serde_json::Value)> = state_data.iter()
            .map(|(n, v)| (n.as_str(), v))
            .collect();

        let merged = if is_state_level_analyzer(typ) {
            merge_state_level_outputs(&year, &refs)
        } else {
            merge_analyzer_outputs(&refs)
        };

        // Atomic write
        let tmp = out_path.with_extension("tmp.json");
        std::fs::write(&tmp, serde_json::to_string_pretty(&merged)?)?;
        std::fs::rename(&tmp, &out_path)?;

        let n_states = merged["state_count"].as_u64().unwrap_or(0);
        if is_state_level_analyzer(typ) {
            eprintln!("[OK] {type_name} -> {} ({n_states} states)", out_path.display());
        } else {
            let n_districts = merged["district_count"].as_u64().unwrap_or(0);
            eprintln!("[OK] {type_name} -> {} ({n_states} states, {n_districts} districts)",
                out_path.display());
        }

        // CSV export
        if args.csv || args.format == "csv" {
            let csv_path = national_dir.join(format!("us_{type_name}.csv"));
            let csv = if is_state_level_analyzer(typ) {
                state_records_to_csv(&merged)
            } else {
                districts_to_csv(&merged)
            };
            std::fs::write(&csv_path, csv)?;
            eprintln!("[OK] {type_name} CSV -> {}", csv_path.display());
        }

        // JSON format export (explicit --format json)
        if args.format == "json" {
            let json_path = if is_state_level_analyzer(typ) {
                national_dir.join(format!("us_{type_name}_states.json"))
            } else {
                national_dir.join(format!("us_{type_name}_districts.json"))
            };
            let rows = if is_state_level_analyzer(typ) {
                state_records_to_json_array(&merged)
            } else {
                districts_to_json_array(&merged)
            };
            let tmp = json_path.with_extension("tmp.json");
            std::fs::write(&tmp, serde_json::to_string_pretty(&rows)?)?;
            std::fs::rename(&tmp, &json_path)?;
            eprintln!("[OK] {type_name} JSON -> {}", json_path.display());
        }

        // Parquet format: helpful guidance
        if args.format == "parquet" {
            eprintln!(
                "Parquet format not yet supported. \
                 Use --format csv and load with pandas.read_csv() or polars."
            );
        }
    }
    Ok(())
}

/// Merge per-state analyzer JSONs into one national JSON.
/// Adds "state" field to each district record.
pub fn merge_analyzer_outputs(states: &[(&str, &serde_json::Value)]) -> serde_json::Value {
    let analyzer = states.first()
        .and_then(|(_, v)| v.get("analyzer"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let mut all_districts: Vec<serde_json::Value> = vec![];
    for (state_name, state_json) in states {
        if let Some(districts) = state_json.get("districts").and_then(|d| d.as_array()) {
            for d in districts {
                let mut record = d.clone();
                record["state"] = serde_json::Value::String(state_name.to_string());
                all_districts.push(record);
            }
        }
    }

    serde_json::json!({
        "analyzer": analyzer,
        "scope": "national",
        "state_count": states.len(),
        "district_count": all_districts.len(),
        "districts": all_districts,
    })
}

/// Merge per-state STATE-LEVEL analyzer JSONs (e.g. proportionality) into one
/// national JSON. Each state's whole JSON becomes one record in the "states"
/// array, with "state" and "year" fields injected.
pub fn merge_state_level_outputs(year: &str, states: &[(&str, &serde_json::Value)]) -> serde_json::Value {
    let analyzer = states.first()
        .and_then(|(_, v)| v.get("analyzer"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let mut records: Vec<serde_json::Value> = vec![];
    for (state_name, state_json) in states {
        let mut record = match state_json.as_object() {
            Some(obj) => serde_json::Value::Object(obj.clone()),
            None => continue,
        };
        record["state"] = serde_json::Value::String(state_name.to_string());
        record["year"] = serde_json::Value::String(year.to_string());
        records.push(record);
    }

    serde_json::json!({
        "analyzer": analyzer,
        "scope": "national",
        "year": year,
        "state_count": records.len(),
        "states": records,
    })
}

/// Convert state-level national JSON to CSV. Each state record becomes one row.
/// Column order: state, year, then all other fields from the first record
/// (excluding "analyzer" and "state"/"year" which are placed first).
pub fn state_records_to_csv(merged: &serde_json::Value) -> String {
    let records = match merged.get("states").and_then(|d| d.as_array()) {
        Some(d) => d,
        None => return String::new(),
    };
    if records.is_empty() { return String::new(); }

    // Build header: state + year first, then remaining fields in stable order
    let first = match records[0].as_object() {
        Some(o) => o,
        None => return String::new(),
    };
    let mut extra_headers: Vec<String> = first.keys()
        .filter(|k| *k != "state" && *k != "year" && *k != "analyzer"
                    && *k != "per_district_dem_share_sorted")
        .cloned()
        .collect();
    extra_headers.sort(); // stable ordering
    let mut headers = vec!["state".to_string(), "year".to_string()];
    headers.extend(extra_headers);

    let mut out = headers.join(",") + "\n";
    for record in records {
        let row: Vec<String> = headers.iter().map(|h| {
            match record.get(h) {
                None => String::new(),
                Some(serde_json::Value::String(s)) => {
                    if s.contains(',') { format!("\"{}\"", s) } else { s.clone() }
                }
                Some(serde_json::Value::Bool(b)) => b.to_string(),
                Some(serde_json::Value::Number(n)) => n.to_string(),
                Some(serde_json::Value::Null) => String::new(),
                Some(v) => v.to_string(),
            }
        }).collect();
        out += &row.join(",");
        out += "\n";
    }
    out
}

/// Convert state-level national JSON to a JSON array.
pub fn state_records_to_json_array(merged: &serde_json::Value) -> serde_json::Value {
    let records = merged.get("states")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    serde_json::Value::Array(records)
}

/// Convert national JSON to CSV string. Uses first district's keys as headers.
pub fn districts_to_csv(merged: &serde_json::Value) -> String {
    let districts = match merged.get("districts").and_then(|d| d.as_array()) {
        Some(d) => d,
        None => return String::new(),
    };
    if districts.is_empty() { return String::new(); }

    // Collect all keys from first record
    let headers: Vec<String> = districts[0].as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();

    let mut out = headers.join(",") + "\n";
    for d in districts {
        let row: Vec<String> = headers.iter().map(|h| {
            match d.get(h) {
                None => String::new(),
                Some(serde_json::Value::String(s)) => {
                    // Quote strings containing commas
                    if s.contains(',') { format!("\"{}\"", s) } else { s.clone() }
                }
                Some(serde_json::Value::Bool(b)) => b.to_string(),
                Some(serde_json::Value::Number(n)) => n.to_string(),
                Some(serde_json::Value::Null) => String::new(),
                Some(v) => v.to_string(),
            }
        }).collect();
        out += &row.join(",");
        out += "\n";
    }
    out
}

/// Convert national JSON districts array to a JSON array of objects.
/// Each district record becomes a JSON object preserving all fields.
pub fn districts_to_json_array(merged: &serde_json::Value) -> serde_json::Value {
    let districts = merged.get("districts")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();
    serde_json::Value::Array(districts)
}

fn resolve_types(types: &[AnalyzerType]) -> Vec<AnalyzerType> {
    if types.iter().any(|t| *t == AnalyzerType::All) {
        AnalyzerType::all_concrete()
    } else {
        types.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_two_states_correct_district_count() {
        let vt = serde_json::json!({
            "analyzer": "demographic",
            "districts": [{"district": 1, "total_pop": 643077, "pct_minority": 0.11}]
        });
        let al = serde_json::json!({
            "analyzer": "demographic",
            "districts": [
                {"district": 1, "total_pop": 715760, "pct_minority": 0.40},
                {"district": 2, "total_pop": 715760, "pct_minority": 0.35},
            ]
        });
        let merged = merge_analyzer_outputs(&[("vermont", &vt), ("alabama", &al)]);
        assert_eq!(merged["district_count"].as_u64().unwrap(), 3);
        assert_eq!(merged["state_count"].as_u64().unwrap(), 2);
    }

    #[test]
    fn test_merge_adds_state_field() {
        let vt = serde_json::json!({
            "analyzer": "demographic",
            "districts": [{"district": 1, "total_pop": 643077}]
        });
        let merged = merge_analyzer_outputs(&[("vermont", &vt)]);
        let d = &merged["districts"][0];
        assert_eq!(d["state"].as_str().unwrap(), "vermont");
    }

    #[test]
    fn test_merge_empty_states_returns_zero() {
        let merged = merge_analyzer_outputs(&[]);
        assert_eq!(merged["district_count"].as_u64().unwrap(), 0);
    }

    #[test]
    fn test_merge_preserves_all_fields() {
        let vt = serde_json::json!({
            "analyzer": "compactness",
            "districts": [{"district": 1, "polsby_popper": 0.364, "reock": 0.385}]
        });
        let merged = merge_analyzer_outputs(&[("vermont", &vt)]);
        let d = &merged["districts"][0];
        assert!((d["polsby_popper"].as_f64().unwrap() - 0.364).abs() < 1e-9);
        assert!((d["reock"].as_f64().unwrap() - 0.385).abs() < 1e-9);
    }

    // ── Task 143: JSON format output ──────────────────────────────────────────

    #[test]
    fn test_aggregate_json_format_parsed() {
        use clap::Parser;
        // Verify --format json is accepted as an arg
        let args = crate::args::AggregateArgs::parse_from([
            "aggregate", "--year", "2020", "--version", "v1", "--format", "json",
        ]);
        assert_eq!(args.format, "json", "--format json should be accepted");
    }

    #[test]
    fn test_aggregate_parquet_format_parsed() {
        use clap::Parser;
        // Verify --format parquet parses (guidance emitted at runtime, not a parse error)
        let args = crate::args::AggregateArgs::parse_from([
            "aggregate", "--year", "2020", "--version", "v1", "--format", "parquet",
        ]);
        assert_eq!(args.format, "parquet");
    }

    #[test]
    fn test_districts_to_json_array_is_array() {
        let merged = serde_json::json!({
            "analyzer": "demographic",
            "districts": [
                {"district": 1, "state": "vermont", "total_pop": 643077},
                {"district": 2, "state": "vermont", "total_pop": 600000}
            ]
        });
        let arr = districts_to_json_array(&merged);
        assert!(arr.is_array(), "output should be a JSON array");
        assert_eq!(arr.as_array().unwrap().len(), 2, "should have 2 district objects");
        assert_eq!(arr[0]["district"].as_u64().unwrap(), 1);
        assert_eq!(arr[1]["total_pop"].as_u64().unwrap(), 600000);
    }

    #[test]
    fn test_aggregate_parquet_gives_guidance() {
        // Test that parquet is accepted at the args level (guidance emitted at runtime).
        // We verify the format string parses without error.
        use clap::Parser;
        let args = crate::args::AggregateArgs::parse_from([
            "aggregate", "--format", "parquet",
        ]);
        // Parquet guidance fires at runtime inside run_aggregate when format=="parquet".
        // We verify the field is correctly stored.
        assert_eq!(args.format, "parquet",
            "parquet format should be stored for runtime guidance");
    }

    // ── State-level aggregation (proportionality) ─────────────────────────────

    #[test]
    fn test_merge_state_level_injects_state_and_year() {
        let vt = serde_json::json!({
            "analyzer": "proportionality",
            "available": true,
            "dem_vote_share_statewide": 0.665,
            "proportionality_gap_pp": 33.5,
            "n_districts": 1
        });
        let merged = merge_state_level_outputs("2020", &[("vermont", &vt)]);
        let state = &merged["states"][0];
        assert_eq!(state["state"].as_str().unwrap(), "vermont");
        assert_eq!(state["year"].as_str().unwrap(), "2020");
        assert_eq!(merged["year"].as_str().unwrap(), "2020");
        assert_eq!(merged["state_count"].as_u64().unwrap(), 1);
    }

    #[test]
    fn test_merge_state_level_two_states() {
        let vt = serde_json::json!({"analyzer": "proportionality", "proportionality_gap_pp": 33.5, "n_districts": 1});
        let tx = serde_json::json!({"analyzer": "proportionality", "proportionality_gap_pp": -8.2, "n_districts": 38});
        let merged = merge_state_level_outputs("2020", &[("vermont", &vt), ("texas", &tx)]);
        assert_eq!(merged["state_count"].as_u64().unwrap(), 2);
        assert_eq!(merged["states"][0]["state"].as_str().unwrap(), "vermont");
        assert_eq!(merged["states"][1]["state"].as_str().unwrap(), "texas");
    }

    #[test]
    fn test_state_records_to_csv_has_state_year_first() {
        let vt = serde_json::json!({"analyzer": "proportionality", "proportionality_gap_pp": 33.5, "n_districts": 1, "available": true});
        let merged = merge_state_level_outputs("2020", &[("vermont", &vt)]);
        let csv = state_records_to_csv(&merged);
        let lines: Vec<&str> = csv.lines().collect();
        assert!(lines.len() >= 2);
        let header = lines[0];
        assert!(header.starts_with("state,year"), "header must start with state,year; got: {header}");
        assert!(lines[1].contains("vermont"));
        assert!(lines[1].contains("2020"));
        assert!(lines[1].contains("33.5"));
    }

    #[test]
    fn test_state_records_to_csv_excludes_per_district_array() {
        let vt = serde_json::json!({
            "analyzer": "proportionality",
            "proportionality_gap_pp": 33.5,
            "per_district_dem_share_sorted": [0.665]
        });
        let merged = merge_state_level_outputs("2020", &[("vermont", &vt)]);
        let csv = state_records_to_csv(&merged);
        assert!(!csv.contains("per_district_dem_share_sorted"),
            "per_district array must be excluded from CSV");
    }

    #[test]
    fn test_is_state_level_proportionality() {
        assert!(is_state_level_analyzer(&AnalyzerType::Proportionality));
        assert!(!is_state_level_analyzer(&AnalyzerType::Demographic));
        assert!(!is_state_level_analyzer(&AnalyzerType::Political));
        assert!(!is_state_level_analyzer(&AnalyzerType::Summary));
    }

    #[test]
    fn test_state_records_to_json_array_is_array() {
        let vt = serde_json::json!({"analyzer": "proportionality", "proportionality_gap_pp": 33.5});
        let merged = merge_state_level_outputs("2020", &[("vermont", &vt)]);
        let arr = state_records_to_json_array(&merged);
        assert!(arr.is_array());
        assert_eq!(arr.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_csv_export_has_header_and_rows() {
        let merged = serde_json::json!({
            "analyzer": "demographic",
            "districts": [
                {"state": "vermont", "district": 1, "total_pop": 643077, "pct_minority": 0.11}
            ]
        });
        let csv = districts_to_csv(&merged);
        let lines: Vec<&str> = csv.lines().collect();
        assert!(lines.len() >= 2, "need header + at least 1 row");
        assert!(lines[0].contains("state"), "header must contain 'state'");
        assert!(lines[0].contains("district"), "header must contain 'district'");
        assert!(lines[1].contains("vermont"), "data row must contain state name");
        assert!(lines[1].contains("643077"), "data row must contain population");
    }
}
