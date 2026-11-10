//! Native reproducibility-package construction without filesystem access.
use std::collections::BTreeMap;
use crate::{manifest_schema::ReadOnlyPlanManifest, rplan::RplanFileWithIds};
use serde::Serialize;
use crate::verification::{generate_verification_command_from_manifest, generate_verification_script};

/// Build native package files. Source analysis and manifest bytes remain unmodified.
/// Recorded audit metadata is not a verified audit certificate.
pub fn build_reproducibility_package<D: Clone + Serialize>(
    label: &str, rplan: &RplanFileWithIds<D>, manifest_text: Option<&str>,
    analysis: &BTreeMap<String, String>,
) -> anyhow::Result<BTreeMap<String, String>> {
    anyhow::ensure!(!label.is_empty() && label.len() <= 120 && label.bytes().all(|b|b.is_ascii_alphanumeric() || b == b'_' || b == b'-'),
        "Package label must use 1–120 ASCII letters, digits, underscores or hyphens");
    let mut files = BTreeMap::new();
    let manifest: Option<ReadOnlyPlanManifest> = manifest_text.map(serde_json::from_str).transpose()?;
    if let Some(text) = manifest_text { files.insert("manifest.json".into(), text.into()); }
    let legacy = RplanFileWithIds { rplan_version: "0.1".into(), metadata: rplan.metadata.clone(),
        assignments: rplan.assignments.clone(), geometry: rplan.geometry.clone() };
    files.insert(format!("{label}.rplan"), serde_json::to_string_pretty(&legacy)?);
    for (name, text) in analysis {
        anyhow::ensure!(name.ends_with(".json") && name.len() <= 120 && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.') && !name.starts_with('.'),
            "Analysis file must be a plain .json basename");
        files.insert(format!("analysis/{name}"), text.clone());
    }
    let audit_json = if let Some(ref m) = manifest {
        serde_json::to_string_pretty(&serde_json::json!({
            "label": m.label,
            "audit": {
                "verification_command": generate_verification_command_from_manifest(m),
                "verification_script": generate_verification_script(m),
                "binary_version": m.binary_version,
                "binary_download_url": m.binary_download_url,
                "binary_sha256": if m.binary_sha256.is_empty() { "(not computed)".to_string() } else { m.binary_sha256.clone() },
                "adjacency_file": m.adjacency_file,
                "adjacency_sha256": if m.adjacency_sha256.is_empty() { "(not computed — run: sha256sum adjacency_file)".to_string() } else { m.adjacency_sha256.clone() },
                "tiger_source_url": m.tiger_source_url,
                "tiger_sha256": m.tiger_sha256.clone().unwrap_or_else(|| format!("(not recorded — download from {} and compute manually)", m.tiger_source_url)),
                "created_at": m.created_at,
                "seed": m.seed,
            }
        }))?
    } else {
        serde_json::to_string_pretty(&serde_json::json!({
            "label": label,
            "audit": { "note": "manifest.json not found — audit fields unavailable" }
        }))?
    };

    files.insert("audit.json".into(), audit_json);
    let verify_content = if let Some(ref m) = manifest {
        generate_verification_script(m)
    } else {
        format!("# Verification script unavailable — manifest.json not found\n# Re-run: bisect state --label {}", label)
    };
    files.insert("verify.sh".into(), verify_content);
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PlanManifest,RplanFile};
    fn fixture() -> RplanFile { RplanFile { rplan_version:"0.1".into(), metadata:Default::default(), assignments:std::collections::HashMap::from([("53001000100".into(),1)]), geometry:None } }
    #[test]
    fn retains_exact_supplied_bytes_and_native_audit() {
        let text=serde_json::to_string_pretty(&PlanManifest { label:"fixture".into(),seed:Some(42),state_code:"WA".into(),..Default::default() }).unwrap();
        let analysis=BTreeMap::from([("population.json".into(), "{ \"supplied\": true }\n".into())]);
        let out=build_reproducibility_package("fixture",&fixture(),Some(&text),&analysis).unwrap();
        assert_eq!(out["manifest.json"],text);assert_eq!(out["analysis/population.json"],analysis["population.json"]);
        let audit:serde_json::Value=serde_json::from_str(&out["audit.json"]).unwrap();assert_eq!(audit["audit"]["seed"],42);
        assert!(out["verify.sh"].contains("--seed 42"));
    }
    #[test]
    fn missing_manifest_is_explicit() {
        let out=build_reproducibility_package("fixture",&fixture(),None,&BTreeMap::new()).unwrap();
        assert_eq!(out.len(),3);assert!(!out.contains_key("manifest.json"));assert!(out["verify.sh"].starts_with('#'));
    }
    #[test]
    fn rejects_output_path_injection() {
        for label in ["../bad","bad/a","bad\\a", "bad;touch x", ""] { assert!(build_reproducibility_package(label,&fixture(),None,&BTreeMap::new()).is_err()); }
        for name in ["../bad.json","bad/a.json", ".hidden.json", "bad.txt"] {
            assert!(build_reproducibility_package("fixture",&fixture(),None,&BTreeMap::from([(name.into(),"{}".into())])).is_err());
        }
    }
    #[test]
    fn rejects_malformed_supplied_manifest() { assert!(build_reproducibility_package("fixture",&fixture(),Some("bad"),&BTreeMap::new()).is_err()); }
}
