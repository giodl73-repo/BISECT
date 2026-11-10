//! Pure BISECT cache report projection. Callers validate source manifests and
//! supply the upstream FLETCH gate counts; this module performs no disk access.
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct CacheIndexEntry {
    pub dataset_id: String,
    pub cache_key: String,
    pub sha256: String,
    pub relative_path: String,
    pub bytes: u64,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FletchCacheIndexRow {
    pub fletch_id: String,
    pub dataset_id: String,
    pub cache_key: String,
    pub sha256: String,
    pub relative_path: String,
    pub bytes: u64,
    pub verified: bool,
    pub evidence_status: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FletchCacheIndexReport {
    pub schema_version: String,
    pub generated_by: String,
    pub source_schema: String,
    pub registry_id: String,
    pub fletch_source_count: usize,
    pub indexed_source_count: usize,
    pub missing_source_count: usize,
    pub unexpected_index_count: usize,
    pub unverified_index_count: usize,
    pub byte_count: u64,
    pub rows: Vec<FletchCacheIndexRow>,
}

pub fn project_cache_index(
    registry_id: &str,
    expected_ids: &BTreeSet<String>,
    entries: &[CacheIndexEntry],
    unexpected_index_count: usize,
    unverified_index_count: usize,
) -> FletchCacheIndexReport {
    let indexed_by_dataset = entries
        .iter()
        .map(|entry| (entry.dataset_id.clone(), entry))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::new();
    for fletch_id in expected_ids {
        match indexed_by_dataset.get(fletch_id) {
            Some(entry) => rows.push(FletchCacheIndexRow {
                fletch_id: fletch_id.clone(),
                dataset_id: entry.dataset_id.clone(),
                cache_key: entry.cache_key.clone(),
                sha256: entry.sha256.clone(),
                relative_path: entry.relative_path.clone(),
                bytes: entry.bytes,
                verified: entry.verified,
                evidence_status: if entry.verified {
                    "indexed-verified"
                } else {
                    "indexed-unverified"
                }
                .to_string(),
            }),
            None => rows.push(FletchCacheIndexRow {
                fletch_id: fletch_id.clone(),
                dataset_id: String::new(),
                cache_key: String::new(),
                sha256: String::new(),
                relative_path: String::new(),
                bytes: 0,
                verified: false,
                evidence_status: "missing-index-row".to_string(),
            }),
        }
    }
    for entry in entries {
        if !expected_ids.contains(&entry.dataset_id) {
            rows.push(FletchCacheIndexRow {
                fletch_id: entry.dataset_id.clone(),
                dataset_id: entry.dataset_id.clone(),
                cache_key: entry.cache_key.clone(),
                sha256: entry.sha256.clone(),
                relative_path: entry.relative_path.clone(),
                bytes: entry.bytes,
                verified: entry.verified,
                evidence_status: "unexpected-index-row".to_string(),
            });
        }
    }

    rows.sort_by(|left, right| {
        left.evidence_status
            .cmp(&right.evidence_status)
            .then(left.fletch_id.cmp(&right.fletch_id))
    });
    let indexed_source_count = rows
        .iter()
        .filter(|row| {
            matches!(
                row.evidence_status.as_str(),
                "indexed-verified" | "indexed-unverified"
            )
        })
        .count();
    let missing_source_count = rows
        .iter()
        .filter(|row| row.evidence_status == "missing-index-row")
        .count();
    let byte_count = rows
        .iter()
        .filter(|row| row.evidence_status == "indexed-verified")
        .map(|row| row.bytes)
        .sum();

    FletchCacheIndexReport {
        schema_version: "bisect.fletch-cache-index.v1".to_string(),
        generated_by: "bisect-cli".to_string(),
        source_schema: "fletch.cache-index.v1".to_string(),
        registry_id: registry_id.to_string(),
        fletch_source_count: expected_ids.len(),
        indexed_source_count,
        missing_source_count,
        unexpected_index_count,
        unverified_index_count,
        byte_count,
        rows,
    }
}

// Mirrors the pinned FLETCH manifest wire contract. Unknown fields are accepted
// as in native serde; strings are evidence and never interpreted as paths/URLs.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, Serialize)]
pub struct Manifest {
    pub schema_version: String,
    pub generated_by: String,
    pub cache_root: String,
    pub entries: Vec<ManifestEntry>,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, Serialize)]
pub struct ManifestEntry {
    pub dataset_id: String,
    pub version: Option<String>,
    pub source_url: String,
    pub cache_key: String,
    pub relative_path: String,
    pub sha256: String,
    pub bytes: u64,
    pub fetched_at_ms: u64,
    pub verified: bool,
    #[serde(default = "one_attempt")]
    pub fetch_attempts: u32,
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default)]
    pub last_retryable_error: Option<String>,
}
fn one_attempt() -> u32 {
    1
}


/// Pinned FLETCH upsert semantics: replace the first matching cache key in
/// place, append new keys, retain duplicate base keys and reset generated_by.
/// This is metadata transformation only; it never verifies or opens objects.
pub fn merge_cache_manifest_text(base: &str, updates: &str) -> Result<Manifest, String> {
    let mut manifest: Manifest = serde_json::from_str(base).map_err(|e| e.to_string())?;
    if manifest.schema_version != "fletch.cache-manifest.v1" {
        return Err("Invalid FLETCH cache manifest schema".into());
    }
    let valid = |entry: &ManifestEntry| -> Result<(), String> {
        if !entry.sha256.starts_with("sha256:") || entry.sha256.len() != 71 {
            return Err(format!("Invalid FLETCH SHA256 for {}", entry.dataset_id));
        }
        Ok(())
    };
    for entry in &manifest.entries { valid(entry)?; }
    let entries: Vec<ManifestEntry> = serde_json::from_str(updates).map_err(|e| e.to_string())?;
    for entry in entries {
        valid(&entry)?;
        if let Some(existing) = manifest.entries.iter_mut().find(|e| e.cache_key == entry.cache_key) {
            *existing = entry;
        } else {
            manifest.entries.push(entry);
        }
    }
    manifest.generated_by = "fletch-core/0.1.0".into();
    Ok(manifest)
}

/// Shared pinned native manifest wire validation; no paths are resolved.
pub fn read_cache_manifest_text(text: &str) -> Result<Manifest, String> {
    let manifest: Manifest =
        serde_json::from_str(text).map_err(|e| format!("FLETCH cache manifest: {e}"))?;
    if manifest.schema_version != "fletch.cache-manifest.v1" {
        return Err("Invalid FLETCH cache manifest schema".into());
    }
    let mut total = 0u64;
    for entry in &manifest.entries {
        if !entry.sha256.starts_with("sha256:") || entry.sha256.len() != 71 {
            return Err(format!("Invalid FLETCH SHA256 for {}", entry.dataset_id));
        }
        total = total
            .checked_add(entry.bytes)
            .ok_or("FLETCH index byte total exceeds u64")?;
    }
    Ok(manifest)
}

/// Read selected manifest text without resolving its cache root or entries.
/// The caller controls expected IDs from the native source planner.
pub fn report_from_manifest(
    text: &str,
    expected_ids: &BTreeSet<String>,
) -> Result<FletchCacheIndexReport, String> {
    let manifest = read_cache_manifest_text(text)?;
    // Keep all fields typed even though report projection uses only a subset.
    let _ = (&manifest.generated_by, &manifest.cache_root);
    let mut entries = Vec::with_capacity(manifest.entries.len());
    let mut total = 0u64;
    for entry in manifest.entries {
        if !entry.sha256.starts_with("sha256:") || entry.sha256.len() != 71 {
            return Err(format!("Invalid FLETCH SHA256 for {}", entry.dataset_id));
        }
        total = total
            .checked_add(entry.bytes)
            .ok_or("FLETCH index byte total exceeds u64")?;
        let _ = (
            &entry.version,
            &entry.source_url,
            entry.fetched_at_ms,
            entry.fetch_attempts,
            entry.retry_count,
            &entry.last_retryable_error,
        );
        entries.push(CacheIndexEntry {
            dataset_id: entry.dataset_id,
            cache_key: entry.cache_key,
            sha256: entry.sha256,
            relative_path: entry.relative_path,
            bytes: entry.bytes,
            verified: entry.verified,
        });
    }
    // Native FLETCH treats an empty expected set as unrestricted for its gate.
    let unexpected = entries
        .iter()
        .filter(|e| !expected_ids.is_empty() && !expected_ids.contains(&e.dataset_id))
        .count();
    let unverified = entries.iter().filter(|e| !e.verified).count();
    Ok(project_cache_index(
        "bisect.fetch-sources",
        expected_ids,
        &entries,
        unexpected,
        unverified,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entry(id: &str, bytes: u64, verified: bool) -> CacheIndexEntry {
        CacheIndexEntry {
            dataset_id: id.into(),
            cache_key: "key".into(),
            sha256: "hash".into(),
            relative_path: "objects/data".into(),
            bytes,
            verified,
        }
    }
    #[test]
    fn duplicate_projection_preserves_gate_evidence_and_exact_counts() {
        let expected = BTreeSet::from(["a".into(), "b".into()]);
        let exact = 9_007_199_254_740_993;
        let report = project_cache_index(
            "registry",
            &expected,
            &[
                entry("a", 1, false),
                entry("a", exact, true),
                entry("other", 2, false),
            ],
            1,
            2,
        );
        assert_eq!(report.indexed_source_count, 1);
        assert_eq!(report.missing_source_count, 1);
        assert_eq!(report.unverified_index_count, 2);
        assert_eq!(report.unexpected_index_count, 1);
        assert_eq!(report.byte_count, exact);
        assert_eq!(report.rows[0].bytes, exact);
        assert_eq!(report.rows[1].evidence_status, "missing-index-row");
        assert_eq!(report.rows[2].evidence_status, "unexpected-index-row");
        assert!(serde_json::to_string(&report)
            .unwrap()
            .contains("9007199254740993"));
    }
    #[test]
    fn empty_registry_keeps_upstream_gate_counts_and_unexpected_report_rows() {
        let report = project_cache_index(
            "registry",
            &BTreeSet::new(),
            &[entry("other", 7, true)],
            0,
            0,
        );
        assert_eq!(report.unexpected_index_count, 0);
        assert_eq!(report.rows[0].evidence_status, "unexpected-index-row");
        assert_eq!(report.byte_count, 0);
    }
}
