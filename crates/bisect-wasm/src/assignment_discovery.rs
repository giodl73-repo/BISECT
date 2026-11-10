//! Bounded selected-output-tree adapter for native original assignment discovery.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::Path};
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Input {
    pub manifest_json: String,
    pub explicit_label: Option<String>,
    pub files_base64: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine};

    fn input() -> Input {
        let manifest = bisect_report::manifest_schema::ReadOnlyPlanManifest {label:"fallback".into(), year:"2020".into(), ..Default::default()};
        Input {manifest_json:serde_json::to_string(&manifest).unwrap(), explicit_label:Some("chosen".into()), files_base64:BTreeMap::new()}
    }
    fn put(input: &mut Input, version: &str, label: &str, manifest: bool, assignments: &str) {
        let directory = format!("{version}/2020/plans/{label}");
        if manifest { input.files_base64.insert(format!("{directory}/manifest.json"), STANDARD.encode(&input.manifest_json)); }
        input.files_base64.insert(format!("{directory}/data/final_assignments.json"), STANDARD.encode(assignments));
    }
    #[test]
    fn skips_invalid_context_and_preserves_exact_source() {
        let mut input = input();
        put(&mut input, "v1", "chosen", false, "{\"bad\":1}");
        put(&mut input, "verify", "chosen", true, " {\"unit\":2} \n");
        put(&mut input, "international", "chosen", true, "{\"later\":3}");
        let result = discover(input).unwrap();
        assert_eq!(result["status"], "found");
        assert_eq!(result["source_path"], "verify/2020/plans/chosen/data/final_assignments.json");
        assert_eq!(result["source_json"], " {\"unit\":2} \n");
    }
    #[test]
    fn fallback_needs_no_context_and_missing_is_disclosed() {
        let mut selected = input();
        put(&mut selected, "v1", "fallback", false, "{\"unit\":1}");
        assert_eq!(discover(selected).unwrap()["status"], "found");
        assert_eq!(discover(input()).unwrap()["status"], "not-found");
    }
    #[test]
    fn first_malformed_or_empty_source_never_falls_through() {
        for (text, status) in [("not json", "failed"), ("{}", "empty")] {
            let mut selected = input();
            put(&mut selected, "v1", "chosen", true, text);
            put(&mut selected, "verify", "chosen", true, "{\"unit\":1}");
            let result = discover(selected).unwrap();
            assert_eq!(result["status"], status);
            assert_eq!(result["source_path"], "v1/2020/plans/chosen/data/final_assignments.json");
        }
    }
    #[test]
    fn rejects_traversal_and_case_collisions() {
        let mut selected = input(); selected.explicit_label = Some("../chosen".into());
        assert!(discover(selected).is_err());
        let mut selected = input();
        selected.files_base64.insert("a.json".into(), STANDARD.encode("{}"));
        selected.files_base64.insert("A.json".into(), STANDARD.encode("{}"));
        assert!(discover(selected).is_err());
    }
}

pub(crate) fn discover(input: Input) -> Result<Value, String> {
    if input.manifest_json.is_empty() || input.manifest_json.len() > 1024 * 1024 {
        return Err("Select a run manifest up to 1 MiB.".into());
    }
    let manifest: bisect_report::manifest_schema::ReadOnlyPlanManifest = serde_json::from_str(&input.manifest_json)
        .map_err(|e| format!("Invalid run manifest: {e}"))?;
    for component in [&manifest.year, &manifest.label].into_iter().chain(input.explicit_label.iter()) {
        if !crate::package_verification::safe(component) || component.contains('/') {
            return Err("Year and labels must be safe single path components.".into());
        }
    }
    let files = crate::package_verification::selected_bytes(input.files_base64)?;
    let key = |p: &Path| p.to_string_lossy().replace('\\', "/");
    let found = bisect_report::assignment_discovery::original_source(
        Path::new(""), &manifest.year, &manifest.label, input.explicit_label.as_deref(),
        |directory| files.get(&key(&directory.join("manifest.json"))).is_some_and(|bytes|
            bytes.len() <= 1024 * 1024 && serde_json::from_slice::<bisect_report::manifest_schema::ReadOnlyPlanManifest>(bytes).is_ok()),
        |path| files.contains_key(&key(path)),
        |path| Ok::<_, String>(files[&key(path)].clone()),
    )?;
    let (status, source_path, source_json, source_sha256, unit_count, error) = match found {
        None => ("not-found", None, None, None, None, None),
        Some((path, bytes)) => {
            let path = Some(key(&path));
            let hash = Some(format!("{:x}", Sha256::digest(&bytes)));
            let parsed = (|| {
                if bytes.len() > 8 * 1024 * 1024 { return Err("Assignment source exceeds 8 MiB.".into()); }
                let text = String::from_utf8(bytes).map_err(|_| "Assignment source must be UTF-8.".to_string())?;
                let assignments = crate::assignment_verification::assignments(&text)?;
                Ok::<_, String>((text, assignments.len()))
            })();
            match parsed {
                Ok((text, count)) => (if count == 0 {"empty"} else {"found"}, path, Some(text), hash, Some(count), None),
                Err(error) => ("failed", path, None, hash, None, Some(error)),
            }
        }
    };
    Ok(json!({"schema_version":"bisect-original-assignment-discovery-v1",
        "status":status,"source_path":source_path,"source_json":source_json,
        "source_sha256":source_sha256,"unit_count":unit_count,"error":error,
        "manifest_sha256":format!("{:x}",Sha256::digest(input.manifest_json.as_bytes())),
        "scope":"Shared native Verify original-source search order over the selected output-root inventory. Explicit labels require a valid PlanManifest under v1, verify, then international; fallback uses the run manifest label under v1 without a context gate. The first existing assignment is authoritative even if malformed or empty. Missing selected files do not prove absence on disk. No external path is opened, no engine is run, and discovery does not certify the plan."}))
}
