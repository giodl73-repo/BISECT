use std::path::PathBuf;
use crate::args::AggregateArgs;
use redist_analysis::AnalyzerType;

pub fn run_aggregate(args: &AggregateArgs) -> anyhow::Result<()> {
    let output_root = PathBuf::from("outputs").join(&args.version);
    let national_dir = output_root.join("national");
    std::fs::create_dir_all(&national_dir)?;

    let types = resolve_types(&args.types);

    // Discover states with analysis outputs
    let states_dir = output_root.join("states");
    if !states_dir.exists() {
        anyhow::bail!("No states directory at {}. Run: redist states first.", states_dir.display());
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
        let merged = merge_analyzer_outputs(&refs);

        // Atomic write
        let tmp = out_path.with_extension("tmp.json");
        std::fs::write(&tmp, serde_json::to_string_pretty(&merged)?)?;
        std::fs::rename(&tmp, &out_path)?;

        let n_states = merged["state_count"].as_u64().unwrap_or(0);
        let n_districts = merged["district_count"].as_u64().unwrap_or(0);
        eprintln!("[OK] {type_name} -> {} ({n_states} states, {n_districts} districts)",
            out_path.display());

        // CSV export
        if args.csv {
            let csv_path = national_dir.join(format!("us_{type_name}.csv"));
            let csv = districts_to_csv(&merged);
            std::fs::write(&csv_path, csv)?;
            eprintln!("[OK] {type_name} CSV -> {}", csv_path.display());
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
