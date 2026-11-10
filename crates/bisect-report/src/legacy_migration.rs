//! Native legacy migration metadata, independent of filesystem and clock.
use crate::manifest_schema::PlanManifest;
use std::collections::{HashMap, HashSet};

/// Preserve direct-folder precedence and lazy manifest lookup from the CLI.
pub fn select_source(identifier:&str,exists:impl Fn(&str)->bool,resolve:impl FnOnce()->Option<String>)->Option<String>{
    let direct=format!("states/{}",identifier.to_lowercase());
    if exists(&direct){return Some(direct);}
    resolve().map(|name|format!("states/{name}")).filter(|path|exists(path))
}

/// Native 64-bit district identifiers remain lossless on wasm32.
/// Invalid, empty or missing assignments retain the native one-district fallback.
pub fn district_count(bytes: Option<&[u8]>) -> (usize, &'static str) {
    match bytes {
        None => (1, "missing"),
        Some(bytes) => match serde_json::from_slice::<HashMap<String, u64>>(bytes) {
            Ok(a) => (a.values().copied().collect::<HashSet<_>>().len().max(1), if a.is_empty() {"empty"} else {"parsed"}),
            Err(_) => (1, "invalid"),
        },
    }
}

pub fn build_manifest(label: &str, state_identifier: &str, source_dir_name: &str, assignments: Option<&[u8]>, created_at: String) -> PlanManifest {
    let (num_districts, _) = district_count(assignments);
    PlanManifest {
        label: label.into(), state_code: state_identifier.to_uppercase(),
        chamber: "congressional".into(), num_districts,
        population_source: "total".into(), partition_mode: "edge-weighted".into(),
        binary_download_url: "https://github.com/owner/BISECT/releases".into(),
        adjacency_file: format!("{source_dir_name}_adjacency.adj.bin"),
        tiger_source_url: "https://www2.census.gov/geo/tiger/TIGER2020/TRACT/".into(),
        created_at, seats_per_district: 1, total_seats: num_districts,
        electoral_system: "single_member".into(), ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_fallback_and_lossless_ids() {
        assert_eq!(district_count(None), (1,"missing"));
        assert_eq!(district_count(Some(b"{}")), (1,"empty"));
        assert_eq!(district_count(Some(b"broken")), (1,"invalid"));
        assert_eq!(district_count(Some(br#"{"a":18446744073709551615,"b":4294967296,"c":18446744073709551615}"#)), (2,"parsed"));
        assert_eq!(district_count(Some(br#"{"a":-1}"#)), (1,"invalid"));
    }
}
