//! Native report command chain-of-custody document; recorded metadata, not recomputed checks.
use crate::manifest_schema::PlanManifestWithCounts;

pub fn build_commission_audit_json<C: std::fmt::Display>(manifest: &PlanManifestWithCounts<C>) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "label": manifest.label,
        "audit": {
            "verification_command": crate::verification::generate_verification_command_from_manifest(manifest),
            "verification_script": crate::verification::generate_verification_script(manifest),
            "binary_version": manifest.binary_version,
            "binary_download_url": manifest.binary_download_url,
            "binary_sha256": if manifest.binary_sha256.is_empty() { "(not computed)".to_string() } else { manifest.binary_sha256.clone() },
            "adjacency_file": manifest.adjacency_file,
            "adjacency_sha256": if manifest.adjacency_sha256.is_empty() { "(not computed — run: sha256sum adjacency_file)".to_string() } else { manifest.adjacency_sha256.clone() },
            "tiger_source_url": manifest.tiger_source_url,
            "tiger_sha256": manifest.tiger_sha256.clone().unwrap_or_else(|| format!("(not recorded — download from {} and compute manually)", manifest.tiger_source_url)),
            "created_at": manifest.created_at,
            "seed": manifest.recorded_seed_json(),
        }
    }))?)
}

#[cfg(test)]
mod seed_tests {
 use super::*;
 use crate::PlanManifest;
 #[test]
 fn legacy_signed_seed_and_optional_unsigned_seed_stay_exact(){
  let legacy=PlanManifest{seed:Some(-1),..Default::default()};
  assert_eq!(legacy.recorded_seed_json().as_i64(),Some(-1));
  assert!(serde_json::to_value(&legacy).unwrap().get("engine_seed").is_none());
  let unsigned=PlanManifest{engine_seed:Some(u64::MAX),source_tool:Some("bisect-wasm".into()),..Default::default()};
  let restored:PlanManifest=serde_json::from_str(&serde_json::to_string(&unsigned).unwrap()).unwrap();
  let audit:serde_json::Value=serde_json::from_str(&build_commission_audit_json(&restored).unwrap()).unwrap();
  assert_eq!(audit["audit"]["seed"].as_u64(),Some(u64::MAX));
  assert!(audit["audit"]["verification_script"].as_str().unwrap().starts_with("# Replay requires"));
 }
}
