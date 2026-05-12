use rcount_core::{
    package_content_hash, verify_jurisdiction_total, verify_package, RcountPackage, RCOUNT_VERSION,
};
use serde::{Deserialize, Serialize};
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
    pub summary_count: usize,
}

pub fn synthetic_summary_basic_manifest(
    package: &RcountPackage,
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
        status: "canvassed".to_string(),
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
    fs::create_dir_all(dir.join("transcripts"))?;

    let computed = package_content_hash(package)?;
    let mut manifest = manifest.clone();
    manifest.content_hash = computed.clone();

    write_json_pretty(&dir.join("manifest.json"), &manifest)?;
    write_json_pretty(
        &dir.join("sources").join("source-index.json"),
        &SourceIndex { sources: vec![] },
    )?;
    write_ndjson(&dir.join("normalized").join("contests.ndjson"), &package.contests)?;
    write_ndjson(
        &dir.join("normalized").join("reporting-units.ndjson"),
        &package.reporting_units,
    )?;
    write_ndjson(
        &dir.join("normalized").join("summaries.ndjson"),
        &package.summaries,
    )?;
    write_lines(
        &dir.join("reconciliation").join("equations.ndjson"),
        &[
            r#"{"equation_id":"contest_selection_sum","status":"declared"}"#,
            r#"{"equation_id":"jurisdiction_contest_total","status":"declared"}"#,
        ],
    )?;
    write_lines(&dir.join("status").join("events.ndjson"), &[])?;
    write_json_pretty(
        &dir.join("proofs").join("package-hashes.json"),
        &PackageHashes {
            package_content_hash: computed,
            contest_count: package.contests.len(),
            reporting_unit_count: package.reporting_units.len(),
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
        summaries: read_ndjson(&dir.join("normalized").join("summaries.ndjson"))?,
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

fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> Result<(), RcountIoError> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes)?;
    Ok(())
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
    use rcount_core::synthetic_summary_basic_package;

    #[test]
    fn round_trips_synthetic_summary_basic_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (decoded_manifest, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_manifest.content_hash, manifest.content_hash);
        assert_eq!(decoded_package.summaries.len(), 3);
        verify_summary_basic_dir(tmp.path()).unwrap();
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
