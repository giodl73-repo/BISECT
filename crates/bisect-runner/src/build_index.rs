//! Shared label-build result/index assembly. Inputs are runner declarations.
use std::collections::{HashMap,BTreeMap};
use crate::config::AlgoYaml;
/// Per-state entry in the build index.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StateIndexEntry {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub districts: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Summary counts in the build index.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BuildSummary {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
}

/// `runs/{label}/{year}/index.json` — build provenance record.
///
/// Written after every successful (or partial) build so that downstream
/// commands (`bisect analyze`, `BISECT report`) can verify provenance.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BuildIndex {
    pub label: String,
    pub year: String,
    pub created: String,
    pub version: String,
    pub config_path: String,
    pub config_sha256: String,
    pub command: String,
    pub output_dir: String,
    pub metis_engine: String,
    /// Snapshot of the `algorithm:` block from the YAML config.
    pub algorithm: serde_json::Value,
    pub states: BTreeMap<String, StateIndexEntry>,
    pub summary: BuildSummary,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigRow { pub state_code:String, pub state_name:String, pub num_districts:usize }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultRow { pub state_code:String, pub success:bool, pub error:Option<String> }
pub fn build_build_index(
    label: &str,
    year: &str,
    config_path: &str,
    config_sha256: &str,
    command: &str,
    output_dir: &str,
    yaml: &AlgoYaml,
    configs: &[ConfigRow],
    results: &[ResultRow],
    created: String,
    version: String,
) -> Result<BuildIndex, String> {
    // Build a lookup from state_code → StateResult.
    let result_map: HashMap<String, &ResultRow> =
        results.iter().map(|r| (r.state_code.clone(), r)).collect();

    // Build per-state entries using state_name (lowercase) as the key.
    let mut state_entries: BTreeMap<String, StateIndexEntry> = BTreeMap::new();
    for cfg in configs {
        let key = cfg.state_name.clone();
        match result_map.get(&cfg.state_code) {
            Some(r) if r.success => {
                state_entries.insert(
                    key,
                    StateIndexEntry {
                        status: "ok".to_string(),
                        districts: Some(cfg.num_districts),
                        error: None,
                    },
                );
            }
            Some(r) => {
                state_entries.insert(
                    key,
                    StateIndexEntry {
                        status: "failed".to_string(),
                        districts: Some(cfg.num_districts),
                        error: r.error.clone(),
                    },
                );
            }
            None => {
                // State was in the config list but produced no result (shouldn't happen).
                state_entries.insert(
                    key,
                    StateIndexEntry {
                        status: "missing".to_string(),
                        districts: Some(cfg.num_districts),
                        error: Some("no result returned from runner".to_string()),
                    },
                );
            }
        }
    }

    let total = results.len();
    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = total - succeeded;

    // Snapshot the algorithm section as a JSON value for the index.
    let algo_section = algorithm_section_to_json(yaml);

    Ok(BuildIndex {
        label: label.to_string(),
        year: year.to_string(),
        created,
        version,
        config_path: config_path.to_string(),
        config_sha256: config_sha256.to_string(),
        command: command.to_string(),
        output_dir: output_dir.to_string(),
        metis_engine: yaml
            .algorithm
            .engine
            .clone()
            .unwrap_or_else(|| "c-ffi".to_string()),
        algorithm: algo_section,
        states: state_entries,
        summary: BuildSummary {
            total,
            succeeded,
            failed,
        },
    })
}

pub fn algorithm_section_to_json(yaml: &AlgoYaml) -> serde_json::Value {
    let sec = &yaml.algorithm;
    let mut map = serde_json::Map::new();
    map.insert(
        "structure".to_string(),
        serde_json::Value::String(sec.structure.clone()),
    );
    if let Some(w) = &sec.weights {
        map.insert("weights".to_string(), serde_json::Value::String(w.clone()));
    }
    if let Some(a) = sec.alpha_county {
        map.insert("alpha_county".to_string(), serde_json::json!(a));
    }
    if let Some(s) = &sec.search {
        map.insert("search".to_string(), serde_json::Value::String(s.clone()));
    }
    if let Some(t) = sec.convergence_threshold {
        map.insert("convergence_threshold".to_string(), serde_json::json!(t));
    }
    if let Some(seeds) = sec.seeds {
        map.insert("seeds".to_string(), serde_json::json!(seeds));
    }
    if let Some(seed) = sec.seed {
        map.insert("seed".to_string(), serde_json::json!(seed));
    }
    if let Some(tol) = sec.balance_tolerance {
        map.insert("balance_tolerance".to_string(), serde_json::json!(tol));
    }
    if let Some(swing) = sec.area_swing {
        map.insert("area_swing".to_string(), serde_json::json!(swing));
    }
    if let Some(w) = sec.w_vra {
        map.insert("w_vra".to_string(), serde_json::json!(w));
    }
    if let Some(bl) = sec.burst_length {
        map.insert("burst_length".to_string(), serde_json::json!(bl));
    }
    if let Some(nb) = sec.n_bursts {
        map.insert("n_bursts".to_string(), serde_json::json!(nb));
    }
    if let Some(pct) = sec.percentile {
        map.insert("percentile".to_string(), serde_json::json!(pct));
    }
    serde_json::Value::Object(map)
}

