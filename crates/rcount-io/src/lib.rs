use rcount_core::{
    package_content_hash, verify_jurisdiction_total, verify_package, RcountPackage, RCOUNT_VERSION,
    SOURCE_HASH_PREFIX,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RcountIoError {
    #[error("core error: {0}")]
    Core(#[from] rcount_core::RcountCoreError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unsupported RCOUNT version: {0}")]
    UnsupportedVersion(String),
    #[error("manifest content_hash mismatch: declared {declared}, computed {computed}")]
    ContentHashMismatch { declared: String, computed: String },
    #[error("source index is empty")]
    EmptySourceIndex,
    #[error("source path is not package-relative under sources/: {path}")]
    InvalidSourcePath { path: String },
    #[error("source file is missing: {path}")]
    MissingSourceFile { path: String },
    #[error("source hash mismatch for {source_id}: declared {declared}, computed {computed}")]
    SourceHashMismatch {
        source_id: String,
        declared: String,
        computed: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RcountManifest {
    pub rcount_version: String,
    pub jurisdiction: Jurisdiction,
    pub election: Election,
    pub status: String,
    pub hash_algorithm: String,
    pub content_hash: String,
    pub created_by: CreatedBy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Jurisdiction {
    pub country: String,
    pub state: String,
    pub county: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Election {
    pub date: String,
    #[serde(rename = "type")]
    pub election_type: String,
    pub scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedBy {
    pub tool: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceIndex {
    pub sources: Vec<SourceEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceEntry {
    pub source_id: String,
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageHashes {
    pub package_content_hash: String,
    pub contest_count: usize,
    pub reporting_unit_count: usize,
    pub batch_count: usize,
    pub lineage_count: usize,
    pub inclusion_proof_count: usize,
    pub cvr_count: usize,
    pub rla_audit_count: usize,
    pub summary_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceCheck {
    pub source_id: String,
    pub path: String,
    pub sha256: String,
}

pub fn synthetic_summary_basic_manifest(
    package: &RcountPackage,
) -> Result<RcountManifest, RcountIoError> {
    synthetic_manifest(package, "canvassed")
}

pub fn synthetic_canvass_correction_manifest(
    package: &RcountPackage,
) -> Result<RcountManifest, RcountIoError> {
    synthetic_manifest(package, "canvassed")
}

fn synthetic_manifest(
    package: &RcountPackage,
    status: &str,
) -> Result<RcountManifest, RcountIoError> {
    Ok(RcountManifest {
        rcount_version: RCOUNT_VERSION.to_string(),
        jurisdiction: Jurisdiction {
            country: "US".to_string(),
            state: "SYN".to_string(),
            county: "SYN-COUNTY-1".to_string(),
        },
        election: Election {
            date: "2024-11-05".to_string(),
            election_type: "general".to_string(),
            scope: "synthetic-county".to_string(),
        },
        status: status.to_string(),
        hash_algorithm: "sha256".to_string(),
        content_hash: package_content_hash(package)?,
        created_by: CreatedBy {
            tool: "rcount-io-example".to_string(),
            version: RCOUNT_VERSION.to_string(),
        },
    })
}

pub fn write_package_dir(
    dir: &Path,
    manifest: &RcountManifest,
    package: &RcountPackage,
) -> Result<(), RcountIoError> {
    fs::create_dir_all(dir.join("sources"))?;
    fs::create_dir_all(dir.join("normalized"))?;
    fs::create_dir_all(dir.join("reconciliation"))?;
    fs::create_dir_all(dir.join("status"))?;
    fs::create_dir_all(dir.join("proofs"))?;
    fs::create_dir_all(dir.join("audits"))?;
    fs::create_dir_all(dir.join("transcripts"))?;

    let computed = package_content_hash(package)?;
    let mut manifest = manifest.clone();
    manifest.content_hash = computed.clone();

    write_json_pretty(&dir.join("manifest.json"), &manifest)?;
    let source_entry = write_synthetic_source_export(dir, package)?;
    write_json_pretty(
        &dir.join("sources").join("source-index.json"),
        &SourceIndex {
            sources: vec![source_entry],
        },
    )?;
    write_ndjson(
        &dir.join("normalized").join("contests.ndjson"),
        &package.contests,
    )?;
    write_ndjson(
        &dir.join("normalized").join("reporting-units.ndjson"),
        &package.reporting_units,
    )?;
    write_ndjson(
        &dir.join("normalized").join("batches.ndjson"),
        &package.batches,
    )?;
    write_ndjson(
        &dir.join("normalized").join("lineage.ndjson"),
        &package.lineage,
    )?;
    write_ndjson(
        &dir.join("proofs").join("inclusion-proofs.ndjson"),
        &package.inclusion_proofs,
    )?;
    write_ndjson(&dir.join("normalized").join("cvr.ndjson"), &package.cvr)?;
    write_ndjson(&dir.join("audits").join("rla.ndjson"), &package.rla_audits)?;
    write_ndjson(
        &dir.join("normalized").join("summaries.ndjson"),
        &package.summaries,
    )?;
    write_lines(
        &dir.join("reconciliation").join("equations.ndjson"),
        &[
            r#"{"equation_id":"contest_selection_sum","status":"declared"}"#,
            r#"{"equation_id":"jurisdiction_contest_total","status":"declared"}"#,
            r#"{"equation_id":"batch_summary_total","status":"declared"}"#,
            r#"{"equation_id":"lineage_conservation","status":"declared"}"#,
            r#"{"equation_id":"status_event_declared","status":"declared"}"#,
            r#"{"equation_id":"canvass_correction_event","status":"declared"}"#,
            r#"{"equation_id":"cvr_summary_total","status":"declared"}"#,
            r#"{"equation_id":"rla_sampler_replay","status":"declared"}"#,
            r#"{"equation_id":"rla_margin_metadata","status":"declared"}"#,
            r#"{"equation_id":"rla_stopping_rule","status":"declared"}"#,
        ],
    )?;
    write_ndjson(
        &dir.join("status").join("events.ndjson"),
        &package.status_events,
    )?;
    write_json_pretty(
        &dir.join("proofs").join("package-hashes.json"),
        &PackageHashes {
            package_content_hash: computed,
            contest_count: package.contests.len(),
            reporting_unit_count: package.reporting_units.len(),
            batch_count: package.batches.len(),
            lineage_count: package.lineage.len(),
            inclusion_proof_count: package.inclusion_proofs.len(),
            cvr_count: package.cvr.len(),
            rla_audit_count: package.rla_audits.len(),
            summary_count: package.summaries.len(),
        },
    )?;
    write_json_pretty(
        &dir.join("transcripts").join("verify-transcript.json"),
        &serde_json::json!({
            "status": "generated-fixture",
            "verifier": "rcount-io",
            "checks": ["contest_selection_sum", "jurisdiction_contest_total"]
        }),
    )?;
    Ok(())
}

pub fn read_package_dir(dir: &Path) -> Result<(RcountManifest, RcountPackage), RcountIoError> {
    let manifest: RcountManifest = read_json(&dir.join("manifest.json"))?;
    if manifest.rcount_version != RCOUNT_VERSION {
        return Err(RcountIoError::UnsupportedVersion(manifest.rcount_version));
    }
    let package = RcountPackage {
        rcount_version: manifest.rcount_version.clone(),
        contests: read_ndjson(&dir.join("normalized").join("contests.ndjson"))?,
        reporting_units: read_ndjson(&dir.join("normalized").join("reporting-units.ndjson"))?,
        batches: read_optional_ndjson(&dir.join("normalized").join("batches.ndjson"))?,
        lineage: read_optional_ndjson(&dir.join("normalized").join("lineage.ndjson"))?,
        inclusion_proofs: read_optional_ndjson(
            &dir.join("proofs").join("inclusion-proofs.ndjson"),
        )?,
        cvr: read_optional_ndjson(&dir.join("normalized").join("cvr.ndjson"))?,
        rla_audits: read_optional_ndjson(&dir.join("audits").join("rla.ndjson"))?,
        summaries: read_ndjson(&dir.join("normalized").join("summaries.ndjson"))?,
        status_events: read_ndjson(&dir.join("status").join("events.ndjson"))?,
    };
    let computed = package_content_hash(&package)?;
    if manifest.content_hash != computed {
        return Err(RcountIoError::ContentHashMismatch {
            declared: manifest.content_hash,
            computed,
        });
    }
    Ok((manifest, package))
}

pub fn read_source_index(dir: &Path) -> Result<SourceIndex, RcountIoError> {
    read_json(&dir.join("sources").join("source-index.json"))
}

pub fn verify_source_index(dir: &Path) -> Result<Vec<SourceCheck>, RcountIoError> {
    let index = read_source_index(dir)?;
    if index.sources.is_empty() {
        return Err(RcountIoError::EmptySourceIndex);
    }

    let mut checks = Vec::new();
    for source in index.sources {
        let path = package_relative_source_path(&source.path)?;
        let full_path = dir.join(&path);
        if !full_path.exists() {
            return Err(RcountIoError::MissingSourceFile {
                path: source.path.clone(),
            });
        }
        let computed = source_file_hash(&full_path)?;
        if computed != source.sha256 {
            return Err(RcountIoError::SourceHashMismatch {
                source_id: source.source_id,
                declared: source.sha256,
                computed,
            });
        }
        checks.push(SourceCheck {
            source_id: source.source_id,
            path: source.path,
            sha256: computed,
        });
    }
    Ok(checks)
}

pub fn source_file_hash(path: &Path) -> Result<String, RcountIoError> {
    Ok(source_bytes_hash(&fs::read(path)?))
}

pub fn verify_summary_basic_dir(dir: &Path) -> Result<(), RcountIoError> {
    let (_, package) = read_package_dir(dir)?;
    verify_package(&package)?;
    verify_jurisdiction_total("syn-2024-mayor", "syn:jurisdiction:SYN", &package.summaries)?;
    Ok(())
}

pub fn default_summary_basic_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("summary-basic")
}

pub fn default_canvass_correction_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("canvass-correction")
}

pub fn default_bad_selection_sum_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-selection-sum")
}

pub fn default_mail_batch_added_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("mail-batch-added")
}

pub fn default_missing_batch_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("missing-batch")
}

pub fn default_precinct_split_lineage_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("precinct-split-lineage")
}

pub fn default_bad_lineage_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-lineage")
}

pub fn default_privacy_inclusion_sketch_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("privacy-inclusion-sketch")
}

pub fn default_choice_bearing_proof_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("choice-bearing-proof")
}

pub fn default_cvr_summary_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("cvr-summary")
}

pub fn default_bad_cvr_summary_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-cvr-summary")
}

pub fn default_rla_replay_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("rla-replay")
}

pub fn default_bad_rla_replay_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-rla-replay")
}

pub fn default_rla_stopping_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("rla-stopping")
}

pub fn default_bad_rla_stopping_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-rla-stopping")
}

pub fn default_rla_discrepancy_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("rla-discrepancy")
}

pub fn default_bad_rla_discrepancy_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-rla-discrepancy")
}

pub fn default_rla_margin_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("rla-margin")
}

pub fn default_bad_rla_margin_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-rla-margin")
}

pub fn default_rla_statistical_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("rla-statistical")
}

pub fn default_bad_rla_statistical_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-rla-statistical")
}

pub fn default_colorado_rla_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("colorado-rla")
}

pub fn default_bad_colorado_rla_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-colorado-rla")
}

fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> Result<(), RcountIoError> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes)?;
    Ok(())
}

fn write_synthetic_source_export(
    dir: &Path,
    package: &RcountPackage,
) -> Result<SourceEntry, RcountIoError> {
    let path = PathBuf::from("sources").join("synthetic-summary-export.json");
    let full_path = dir.join(&path);
    let value = serde_json::json!({
        "source_format": "synthetic-summary-export-v1",
        "contest_count": package.contests.len(),
        "reporting_unit_count": package.reporting_units.len(),
        "batch_count": package.batches.len(),
        "lineage_count": package.lineage.len(),
        "inclusion_proof_count": package.inclusion_proofs.len(),
        "cvr_count": package.cvr.len(),
        "rla_audit_count": package.rla_audits.len(),
        "summary_count": package.summaries.len(),
        "status_event_count": package.status_events.len(),
    });
    let bytes = serde_json::to_vec_pretty(&value)?;
    fs::write(&full_path, &bytes)?;
    Ok(SourceEntry {
        source_id: "source:synthetic-summary-export".to_string(),
        path: path.to_string_lossy().replace('\\', "/"),
        sha256: source_bytes_hash(&bytes),
    })
}

fn source_bytes_hash(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(SOURCE_HASH_PREFIX);
    h.update(bytes);
    format!("sha256:{:x}", h.finalize())
}

fn package_relative_source_path(path: &str) -> Result<PathBuf, RcountIoError> {
    let candidate = Path::new(path);
    if candidate.is_absolute()
        || candidate
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(RcountIoError::InvalidSourcePath {
            path: path.to_string(),
        });
    }
    let mut components = candidate.components();
    match components.next() {
        Some(std::path::Component::Normal(first)) if first == "sources" => {}
        _ => {
            return Err(RcountIoError::InvalidSourcePath {
                path: path.to_string(),
            });
        }
    }
    Ok(candidate.to_path_buf())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, RcountIoError> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn write_ndjson<T: Serialize>(path: &Path, records: &[T]) -> Result<(), RcountIoError> {
    let mut file = File::create(path)?;
    for record in records {
        serde_json::to_writer(&mut file, record)?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

fn read_ndjson<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>, RcountIoError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        records.push(serde_json::from_str(&line)?);
    }
    Ok(records)
}

fn read_optional_ndjson<T: for<'de> Deserialize<'de>>(
    path: &Path,
) -> Result<Vec<T>, RcountIoError> {
    if path.exists() {
        read_ndjson(path)
    } else {
        Ok(Vec::new())
    }
}

fn write_lines(path: &Path, lines: &[&str]) -> Result<(), RcountIoError> {
    let mut file = File::create(path)?;
    for line in lines {
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcount_core::{
        synthetic_bad_colorado_rla_package, synthetic_bad_cvr_summary_package,
        synthetic_bad_lineage_package, synthetic_bad_rla_discrepancy_package,
        synthetic_bad_rla_margin_package, synthetic_bad_rla_replay_package,
        synthetic_bad_rla_statistical_package, synthetic_bad_rla_stopping_package,
        synthetic_bad_selection_sum_package, synthetic_canvass_correction_package,
        synthetic_choice_bearing_proof_package, synthetic_colorado_rla_package,
        synthetic_cvr_summary_package, synthetic_mail_batch_added_package,
        synthetic_missing_batch_package, synthetic_precinct_split_lineage_package,
        synthetic_privacy_inclusion_package, synthetic_rla_discrepancy_package,
        synthetic_rla_margin_package, synthetic_rla_replay_package,
        synthetic_rla_statistical_package, synthetic_rla_stopping_package,
        synthetic_summary_basic_package,
    };

    #[test]
    fn round_trips_synthetic_summary_basic_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (decoded_manifest, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_manifest.content_hash, manifest.content_hash);
        assert_eq!(decoded_package.summaries.len(), 3);
        assert_eq!(verify_source_index(tmp.path()).unwrap().len(), 1);
        verify_summary_basic_dir(tmp.path()).unwrap();
    }

    #[test]
    fn round_trips_synthetic_canvass_correction_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_canvass_correction_package();
        let manifest = synthetic_canvass_correction_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.summaries.len(), 6);
        assert_eq!(decoded_package.status_events.len(), 2);
        assert_eq!(verify_source_index(tmp.path()).unwrap().len(), 1);
    }

    #[test]
    fn round_trips_synthetic_mail_batch_added_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_mail_batch_added_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.batches.len(), 3);
        assert_eq!(
            decoded_package
                .summaries
                .iter()
                .filter(|summary| summary.batch_id.is_some())
                .count(),
            3
        );
    }

    #[test]
    fn round_trips_synthetic_missing_batch_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_missing_batch_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.batches.len(), 2);
        assert_eq!(
            decoded_package
                .summaries
                .iter()
                .filter(|summary| summary.batch_id.as_deref() == Some("batch:P-001:late-mail"))
                .count(),
            1
        );
    }

    #[test]
    fn round_trips_synthetic_precinct_split_lineage_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_precinct_split_lineage_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.lineage.len(), 2);
        assert!(decoded_package
            .lineage
            .iter()
            .any(|event| event.lineage_id == "lineage:P-004-split"));
    }

    #[test]
    fn round_trips_synthetic_bad_lineage_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_lineage_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.lineage.len(), 2);
        assert!(decoded_package.lineage[0]
            .current_reporting_unit_ids
            .contains(&"syn:precinct:P-004C".to_string()));
    }

    #[test]
    fn round_trips_synthetic_privacy_inclusion_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_privacy_inclusion_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.inclusion_proofs.len(), 1);
        assert!(decoded_package.inclusion_proofs[0]
            .candidate_selections
            .is_empty());
    }

    #[test]
    fn round_trips_synthetic_choice_bearing_proof_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_choice_bearing_proof_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.inclusion_proofs[0].candidate_selections,
            vec!["cand-a".to_string()]
        );
    }

    #[test]
    fn round_trips_synthetic_cvr_summary_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_cvr_summary_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.cvr.len(), 140);
        assert!(decoded_package
            .cvr
            .iter()
            .any(|row| row.cvr_id == "cvr:P-001:001"));
    }

    #[test]
    fn round_trips_synthetic_bad_cvr_summary_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_cvr_summary_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.cvr.len(), 140);
        assert!(verify_source_index(tmp.path()).is_ok());
    }

    #[test]
    fn round_trips_synthetic_rla_replay_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_replay_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.rla_audits.len(), 1);
        assert_eq!(decoded_package.rla_audits[0].sample_draws.len(), 12);
    }

    #[test]
    fn round_trips_synthetic_bad_rla_replay_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_replay_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rla_audits[0].sample_draws[0].cvr_id,
            "cvr:P-999:999"
        );
    }

    #[test]
    fn round_trips_synthetic_rla_stopping_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_stopping_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.rla_audits[0].observations.len(), 12);
        assert_eq!(
            decoded_package.rla_audits[0].stopping_rule_id.as_deref(),
            Some("zero-discrepancy-threshold-v1")
        );
    }

    #[test]
    fn round_trips_synthetic_bad_rla_stopping_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_stopping_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rla_audits[0].observations[0].observed_selection_ids,
            vec!["cand-b".to_string()]
        );
    }

    #[test]
    fn round_trips_synthetic_rla_discrepancy_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_discrepancy_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.rla_audits[0].discrepancies.len(), 1);
    }

    #[test]
    fn round_trips_synthetic_bad_rla_discrepancy_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_discrepancy_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.rla_audits[0].discrepancies.len(), 1);
    }

    #[test]
    fn round_trips_synthetic_rla_margin_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_margin_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rla_audits[0]
                .margin
                .as_ref()
                .unwrap()
                .reported_margin,
            64
        );
    }

    #[test]
    fn round_trips_synthetic_bad_rla_margin_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_margin_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rla_audits[0]
                .margin
                .as_ref()
                .unwrap()
                .reported_margin,
            65
        );
    }

    #[test]
    fn round_trips_synthetic_rla_statistical_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_statistical_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.rla_audits[0].declared_risk_ppm, Some(1303));
    }

    #[test]
    fn round_trips_synthetic_bad_rla_statistical_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_statistical_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.rla_audits[0].declared_risk_ppm, Some(1304));
    }

    #[test]
    fn round_trips_synthetic_colorado_rla_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_colorado_rla_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rla_audits[0]
                .jurisdiction_method_id
                .as_deref(),
            Some("colorado-rule-25-comparison-v1")
        );
    }

    #[test]
    fn round_trips_synthetic_bad_colorado_rla_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_colorado_rla_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rla_audits[0].public_seed,
            "3141592653589793238X"
        );
    }

    #[test]
    fn rejects_manifest_content_hash_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let manifest_path = tmp.path().join("manifest.json");
        let mut raw: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
        raw["content_hash"] = serde_json::Value::String("sha256:bad".to_string());
        std::fs::write(&manifest_path, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
        assert!(matches!(
            read_package_dir(tmp.path()),
            Err(RcountIoError::ContentHashMismatch { .. })
        ));
    }

    #[test]
    fn round_trips_synthetic_bad_selection_sum_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_selection_sum_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.summaries[0].counted_ballots,
            synthetic_summary_basic_package().summaries[0].counted_ballots + 1
        );
        assert!(verify_source_index(tmp.path()).is_ok());
    }

    #[test]
    fn rejects_tampered_source_file() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        std::fs::write(
            tmp.path()
                .join("sources")
                .join("synthetic-summary-export.json"),
            br#"{"tampered":true}"#,
        )
        .unwrap();

        assert!(matches!(
            verify_source_index(tmp.path()),
            Err(RcountIoError::SourceHashMismatch { .. })
        ));
    }

    #[test]
    fn rejects_empty_source_index() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        std::fs::write(
            tmp.path().join("sources").join("source-index.json"),
            br#"{"sources":[]}"#,
        )
        .unwrap();

        assert!(matches!(
            verify_source_index(tmp.path()),
            Err(RcountIoError::EmptySourceIndex)
        ));
    }

    #[test]
    fn docs_summary_basic_fixture_verifies_when_present() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("examples")
            .join("rcount-golden-packages")
            .join("summary-basic");
        if dir.exists() {
            verify_summary_basic_dir(&dir).unwrap();
        }
    }
}
