use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Observation {
    sha256: String,
    bytes: u64,
}
pub(crate) fn relative_path(path: &str) -> Result<String, String> {
    if path.is_empty() || path.len() > 1024 {
        return Err("Invalid FLETCH cache relative path length".into());
    }
    let parts = path.split(['/', '\\']).collect::<Vec<_>>();
    if parts
        .iter()
        .any(|p| p.is_empty() || *p == "." || *p == ".." || p.contains(':') || p.contains('\0'))
    {
        return Err("Unsafe FLETCH cache relative path".into());
    }
    Ok(parts.join("/"))
}
fn manifest(text: &str) -> Result<bisect_core::fletch_cache::Manifest, String> {
    if text.len() > 8 * 1024 * 1024 {
        return Err("FLETCH cache manifest exceeds 8 MiB".into());
    }
    let m = bisect_core::fletch_cache::read_cache_manifest_text(text)?;
    if m.entries.len() > 2000 {
        return Err("FLETCH cache verification supports up to 2000 manifest entries".into());
    }
    for entry in &m.entries {
        relative_path(&entry.relative_path)?;
    }
    Ok(m)
}
pub(crate) fn plan(text: String) -> Result<Value, String> {
    let m = manifest(&text)?;
    let paths = m
        .entries
        .iter()
        .map(|e| relative_path(&e.relative_path))
        .collect::<Result<BTreeSet<_>, _>>()?;
    Ok(
        json!({"schema_version":"bisect-fletch-cache-file-plan-v1","paths":paths,"entry_count":m.entries.len(),"manifest_sha256":format!("{:x}",Sha256::digest(text.as_bytes()))}),
    )
}
pub(crate) fn verify(text: String, files: BTreeMap<String, Observation>) -> Result<Value, String> {
    let m = manifest(&text)?;
    if files.len() > 2000 {
        return Err("Too many selected FLETCH object observations".into());
    }
    for (path, obs) in &files {
        if relative_path(path)? != *path
            || obs.sha256.len() != 71
            || !obs.sha256.starts_with("sha256:")
            || !obs.sha256[7..]
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err("Invalid selected FLETCH object evidence".into());
        }
    }
    let mut statuses = vec![];
    let mut view = vec![];
    let mut verified = 0usize;
    let mut missing = 0usize;
    let mut mismatched = 0usize;
    let mut expected_bytes = 0u64;
    let mut actual_bytes = 0u64;
    for e in &m.entries {
        let path = relative_path(&e.relative_path)?;
        let observation = files.get(&path);
        let (status, freshness) = match observation {
            None => {
                missing += 1;
                ("missing", "missing")
            }
            Some(obs) if obs.sha256 == e.sha256 && obs.bytes == e.bytes => {
                verified += 1;
                ("verified", "fresh")
            }
            Some(_) => {
                mismatched += 1;
                ("hash-mismatch", "stale")
            }
        };
        expected_bytes = expected_bytes
            .checked_add(e.bytes)
            .ok_or("FLETCH expected byte total exceeds u64")?;
        if let Some(obs) = observation {
            actual_bytes = actual_bytes
                .checked_add(obs.bytes)
                .ok_or("FLETCH actual byte total exceeds u64")?;
        }
        let root = m.cache_root.replace('\\', "/");
        let absolute = if root.is_empty() {
            path.clone()
        } else {
            format!(
                "{}{}{}",
                root,
                if root.ends_with('/') { "" } else { "/" },
                path
            )
        };
        statuses.push(json!({"dataset_id":e.dataset_id,"cache_key":e.cache_key,"relative_path":path,"absolute_path":absolute,"expected_sha256":e.sha256,"actual_sha256":observation.map(|o|o.sha256.clone()),"expected_bytes":e.bytes,"actual_bytes":observation.map(|o|o.bytes),"object_status":status,"freshness_status":freshness}));
        view.push(json!({"dataset_id":e.dataset_id,"relative_path":path,"object_status":status,"expected_bytes_text":e.bytes.to_string(),"actual_bytes_text":observation.map(|o|o.bytes.to_string()),"actual_sha256":observation.map(|o|o.sha256.clone())}));
    }
    let summary = json!({"cache_root":m.cache_root,"entry_count":m.entries.len(),"verified_count":verified,"missing_count":missing,"hash_mismatch_count":mismatched,"fresh_count":verified,"stale_count":mismatched,"missing_freshness_count":missing,"expected_bytes":expected_bytes,"actual_bytes":actual_bytes});
    let report = json!({"schema_version":"fletch.cache-verify.v1","generated_by":"fletch-core/0.1.0","cache_root":m.cache_root,"summary":summary,"statuses":statuses});
    Ok(
        json!({"schema_version":"bisect-fletch-cache-verification-v1","report_json":serde_json::to_string_pretty(&report).map_err(|e|e.to_string())?,"manifest_sha256":format!("{:x}",Sha256::digest(text.as_bytes())),"entry_count":m.entries.len(),"verified_count":verified,"missing_count":missing,"hash_mismatch_count":mismatched,"expected_bytes_text":expected_bytes.to_string(),"actual_bytes_text":actual_bytes.to_string(),"rows":view,"all_selected_entries_verified":missing==0&&mismatched==0,"scope":"Immutable-policy native FLETCH comparison of observed hashes and byte counts. Missing means absent from this file selection. Saved verified flags do not determine this result. Cache-root and absolute-path fields are informational portable spellings; no manifest path is opened. Report text preserves exact u64 values. Source authentication, acquisition, installation, freshness against providers and plan admissibility were not checked."}),
    )
}
