use rhist_core::{package_content_hash, verify_package, RhistPackage, SourceIndexEntry};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RhistIoError {
    #[error("core error: {0}")]
    Core(#[from] rhist_core::RhistCoreError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
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
    #[error("package content hash mismatch: declared {declared}, computed {computed}")]
    PackageHashMismatch { declared: String, computed: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageHashes {
    pub package_content_hash: String,
    pub cycle_count: usize,
    pub context_count: usize,
    pub lineage_event_count: usize,
    pub crosswalk_count: usize,
}

/// Exact package-relative files selected in a browser; never extracted to disk.
pub type PackageFiles = BTreeMap<String, Vec<u8>>;

pub fn read_package_files(files: &PackageFiles) -> Result<RhistPackage, RhistIoError> {
    fn bytes<'a>(files: &'a PackageFiles, path: &str) -> Result<&'a [u8], RhistIoError> {
        files
            .get(path)
            .map(Vec::as_slice)
            .ok_or_else(|| RhistIoError::MissingSourceFile { path: path.into() })
    }
    fn records<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<Vec<T>, RhistIoError> {
        let mut records = Vec::new();
        for line in BufReader::new(bytes).lines() {
            let line = line?;
            if !line.trim().is_empty() {
                records.push(serde_json::from_str(&line)?);
            }
        }
        Ok(records)
    }
    let package = RhistPackage {
        manifest: serde_json::from_slice(bytes(files, "manifest.json")?)?,
        source_index: serde_json::from_slice(bytes(files, "sources/source-index.json")?)?,
        context_index: records(bytes(files, "contexts/context-index.ndjson")?)?,
        cycles: records(bytes(files, "units/cycles.ndjson")?)?,
        lineage_events: records(bytes(files, "units/lineage-events.ndjson")?)?,
        crosswalks: files
            .get("units/crosswalks.ndjson")
            .map(|data| records(data))
            .transpose()?
            .unwrap_or_default(),
        claim_boundary: serde_json::from_slice(bytes(files, "claims/claim-boundary.json")?)?,
    };
    for source in &package.source_index {
        validate_source_path(&source.path)?;
        verify_source_hash(source, sha256_bytes(bytes(files, &source.path)?))?;
    }
    verify_package_hash(&package)?;
    verify_package(&package)?;
    Ok(package)
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifyTranscript {
    pub status: VerificationStatus,
    pub package_id: Option<String>,
    pub package_content_hash: Option<String>,
    pub checks: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum VerificationStatus {
    Pass,
    Fail,
}

/// Same transcript for the disk CLI and the in-memory browser boundary.
pub fn verification_transcript(
    package: Result<RhistPackage, RhistIoError>,
) -> Result<VerifyTranscript, RhistIoError> {
    match package {
        Ok(package) => Ok(VerifyTranscript {
            status: VerificationStatus::Pass,
            package_id: Some(package.manifest.package_id.clone()),
            package_content_hash: Some(package_content_hash(&package)?),
            checks: verify_package(&package)?
                .into_iter()
                .map(|report| report.check_id.to_string())
                .collect(),
            error: None,
        }),
        Err(error) => Ok(VerifyTranscript {
            status: VerificationStatus::Fail,
            package_id: None,
            package_content_hash: None,
            checks: Vec::new(),
            error: Some(error.to_string()),
        }),
    }
}

pub fn read_package_dir(dir: impl AsRef<Path>) -> Result<RhistPackage, RhistIoError> {
    let dir = dir.as_ref();
    let package = read_package_dir_unverified(dir)?;
    verify_source_files(dir, &package.source_index)?;
    verify_package_hash(&package)?;
    verify_package(&package)?;
    Ok(package)
}

pub fn read_package_dir_unverified(dir: impl AsRef<Path>) -> Result<RhistPackage, RhistIoError> {
    let dir = dir.as_ref();
    Ok(RhistPackage {
        manifest: read_json(&dir.join("manifest.json"))?,
        source_index: read_json(&dir.join("sources").join("source-index.json"))?,
        context_index: read_ndjson(&dir.join("contexts").join("context-index.ndjson"))?,
        cycles: read_ndjson(&dir.join("units").join("cycles.ndjson"))?,
        lineage_events: read_ndjson(&dir.join("units").join("lineage-events.ndjson"))?,
        crosswalks: read_optional_ndjson(&dir.join("units").join("crosswalks.ndjson"))?,
        claim_boundary: read_json(&dir.join("claims").join("claim-boundary.json"))?,
    })
}

pub fn refresh_package_hashes(dir: impl AsRef<Path>) -> Result<String, RhistIoError> {
    let dir = dir.as_ref();
    let mut package = read_package_dir_unverified(dir)?;
    verify_source_files(dir, &package.source_index)?;
    let hash = package_content_hash(&package)?;
    package.manifest.package_content_hash = hash.clone();
    write_json_pretty(&dir.join("manifest.json"), &package.manifest)?;
    write_json_pretty(
        &dir.join("proofs").join("package-hashes.json"),
        &PackageHashes {
            package_content_hash: hash.clone(),
            cycle_count: package.cycles.len(),
            context_count: package.context_index.len(),
            lineage_event_count: package.lineage_events.len(),
            crosswalk_count: package.crosswalks.len(),
        },
    )?;
    Ok(hash)
}

pub fn write_package_dir(
    dir: impl AsRef<Path>,
    package: &RhistPackage,
) -> Result<(), RhistIoError> {
    let dir = dir.as_ref();
    fs::create_dir_all(dir.join("sources"))?;
    fs::create_dir_all(dir.join("contexts"))?;
    fs::create_dir_all(dir.join("units"))?;
    fs::create_dir_all(dir.join("proofs"))?;
    fs::create_dir_all(dir.join("claims"))?;
    fs::create_dir_all(dir.join("transcripts"))?;

    let mut package = package.clone();
    package.manifest.package_content_hash = package_content_hash(&package)?;

    write_json_pretty(&dir.join("manifest.json"), &package.manifest)?;
    write_json_pretty(
        &dir.join("sources").join("source-index.json"),
        &package.source_index,
    )?;
    write_ndjson(
        &dir.join("contexts").join("context-index.ndjson"),
        &package.context_index,
    )?;
    write_ndjson(&dir.join("units").join("cycles.ndjson"), &package.cycles)?;
    write_ndjson(
        &dir.join("units").join("lineage-events.ndjson"),
        &package.lineage_events,
    )?;
    write_ndjson(
        &dir.join("units").join("crosswalks.ndjson"),
        &package.crosswalks,
    )?;
    write_json_pretty(
        &dir.join("proofs").join("package-hashes.json"),
        &PackageHashes {
            package_content_hash: package.manifest.package_content_hash.clone(),
            cycle_count: package.cycles.len(),
            context_count: package.context_index.len(),
            lineage_event_count: package.lineage_events.len(),
            crosswalk_count: package.crosswalks.len(),
        },
    )?;
    write_json_pretty(
        &dir.join("claims").join("claim-boundary.json"),
        &package.claim_boundary,
    )?;
    write_json_pretty(
        &dir.join("transcripts").join("verify-transcript.json"),
        &serde_json::json!({
            "status": "generated-fixture",
            "verifier": "rhist-io",
            "checks": [
                "manifest_cycle_refs",
                "source_refs_resolve",
                "context_cycle_refs",
                "context_unit_ids_unique",
                "cycle_context_refs",
                "lineage_unit_refs",
                "lineage_cardinality",
                "crosswalk_unit_refs",
                "crosswalk_weight_sum",
                "claim_boundary_present"
            ]
        }),
    )?;
    Ok(())
}

pub fn verify_package_hash(package: &RhistPackage) -> Result<(), RhistIoError> {
    if is_zero_hash(&package.manifest.package_content_hash) {
        return Ok(());
    }
    let computed = package_content_hash(package)?;
    if package.manifest.package_content_hash != computed {
        return Err(RhistIoError::PackageHashMismatch {
            declared: package.manifest.package_content_hash.clone(),
            computed,
        });
    }
    Ok(())
}

pub fn verify_source_files(
    package_root: impl AsRef<Path>,
    source_index: &[SourceIndexEntry],
) -> Result<(), RhistIoError> {
    let package_root = package_root.as_ref();
    for source in source_index {
        validate_source_path(&source.path)?;
        let path = package_root.join(&source.path);
        if !path.is_file() {
            return Err(RhistIoError::MissingSourceFile {
                path: source.path.clone(),
            });
        }
        let computed = sha256_file(&path)?;
        verify_source_hash(source, computed)?;
    }
    Ok(())
}

pub fn default_fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("fixtures")
        .join("rhist")
        .join(name)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, RhistIoError> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> Result<(), RhistIoError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn read_ndjson<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>, RhistIoError> {
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

fn read_optional_ndjson<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>, RhistIoError> {
    if path.exists() {
        read_ndjson(path)
    } else {
        Ok(Vec::new())
    }
}

fn write_ndjson<T: Serialize>(path: &Path, records: &[T]) -> Result<(), RhistIoError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    for record in records {
        serde_json::to_writer(&mut file, record)?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, RhistIoError> {
    let bytes = fs::read(path)?;
    Ok(sha256_bytes(&bytes))
}

fn validate_source_path(path: &str) -> Result<(), RhistIoError> {
    if !path.starts_with("sources/")
        || path.contains("..")
        || path.contains('\\')
        || path.contains(':')
        || path.split('/').any(|part| part.is_empty() || part == ".")
    {
        return Err(RhistIoError::InvalidSourcePath { path: path.into() });
    }
    Ok(())
}

fn verify_source_hash(source: &SourceIndexEntry, computed: String) -> Result<(), RhistIoError> {
    if computed != source.sha256 {
        return Err(RhistIoError::SourceHashMismatch {
            source_id: source.source_id.clone(),
            declared: source.sha256.clone(),
            computed,
        });
    }
    Ok(())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    format!("sha256:{hex}")
}

fn is_zero_hash(value: &str) -> bool {
    value == "sha256:0000000000000000000000000000000000000000000000000000000000000000"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture_files(root: &Path) -> PackageFiles {
        fn collect(root: &Path, dir: &Path, files: &mut PackageFiles) {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    collect(root, &path, files);
                } else {
                    files.insert(
                        path.strip_prefix(root)
                            .unwrap()
                            .to_string_lossy()
                            .replace('\\', "/"),
                        fs::read(path).unwrap(),
                    );
                }
            }
        }
        let mut files = PackageFiles::new();
        collect(root, root, &mut files);
        files
    }

    #[test]
    fn memory_and_disk_history_verification_match() {
        for name in [
            "l0-rename",
            "l1-split-merge",
            "l2-three-cycle",
            "real-ri-tract-unchanged",
            "l0-missing-unit",
            "l1-bad-weights",
        ] {
            let dir = default_fixture_dir(name);
            let disk = verification_transcript(read_package_dir(&dir)).unwrap();
            let memory = verification_transcript(read_package_files(&fixture_files(&dir))).unwrap();
            assert_eq!(memory, disk, "{name}");
        }
    }

    #[test]
    fn memory_reader_rejects_modified_missing_and_unsafe_sources() {
        let original = fixture_files(&default_fixture_dir("l0-rename"));
        let package = read_package_files(&original).unwrap();
        let source = &package.source_index[0].path;
        let mut modified = original.clone();
        modified.get_mut(source).unwrap().push(0);
        assert!(matches!(
            read_package_files(&modified),
            Err(RhistIoError::SourceHashMismatch { .. })
        ));
        let mut missing = original.clone();
        missing.remove(source);
        assert!(matches!(
            read_package_files(&missing),
            Err(RhistIoError::MissingSourceFile { .. })
        ));
        for path in [
            "sources/../private",
            "sources/..\\private",
            "sources/C:private",
            "sources/./private",
            "sources//private",
            "C:/private",
        ] {
            assert!(matches!(
                validate_source_path(path),
                Err(RhistIoError::InvalidSourcePath { .. })
            ));
        }
    }

    #[test]
    fn reads_l0_rename_fixture() {
        let package = read_package_dir(default_fixture_dir("l0-rename")).unwrap();
        assert_eq!(package.manifest.package_id, "syn-rhist-l0-rename");
        assert_eq!(package.lineage_events.len(), 1);
        assert!(package.crosswalks.is_empty());
    }

    #[test]
    fn reads_l1_split_merge_fixture() {
        let package = read_package_dir(default_fixture_dir("l1-split-merge")).unwrap();
        assert_eq!(package.manifest.package_id, "syn-rhist-l1-split-merge");
        assert_eq!(package.lineage_events.len(), 2);
        assert_eq!(package.crosswalks.len(), 4);
    }

    #[test]
    fn reads_l2_three_cycle_fixture() {
        let package = read_package_dir(default_fixture_dir("l2-three-cycle")).unwrap();
        assert_eq!(package.manifest.package_id, "syn-rhist-l2-three-cycle");
        assert_eq!(package.cycles.len(), 3);
        assert_eq!(package.lineage_events.len(), 7);
        assert_eq!(package.crosswalks.len(), 9);
    }

    #[test]
    fn reads_real_ri_tract_unchanged_fixture() {
        let package = read_package_dir(default_fixture_dir("real-ri-tract-unchanged")).unwrap();
        assert_eq!(package.manifest.package_id, "real-ri-tract-unchanged");
        assert_eq!(package.cycles.len(), 3);
        assert_eq!(package.lineage_events.len(), 2);
        assert_eq!(package.crosswalks.len(), 2);
    }

    #[test]
    fn rejects_l0_missing_unit_fixture() {
        let err = read_package_dir(default_fixture_dir("l0-missing-unit")).unwrap_err();
        assert!(matches!(
            err,
            RhistIoError::Core(rhist_core::RhistCoreError::LineageMissingUnit { .. })
        ));
    }

    #[test]
    fn rejects_l1_bad_weights_fixture() {
        let err = read_package_dir(default_fixture_dir("l1-bad-weights")).unwrap_err();
        assert!(matches!(
            err,
            RhistIoError::Core(rhist_core::RhistCoreError::CrosswalkWeightSum { .. })
        ));
    }

    #[test]
    fn detects_source_hash_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        copy_fixture(default_fixture_dir("l0-rename"), tmp.path());
        fs::write(
            tmp.path().join("sources").join("syn-rename-notice.txt"),
            "tampered\n",
        )
        .unwrap();
        let err = read_package_dir(tmp.path()).unwrap_err();
        assert!(matches!(err, RhistIoError::SourceHashMismatch { .. }));
    }

    #[test]
    fn writes_and_reads_package_dir() {
        let source = default_fixture_dir("l1-split-merge");
        let package = read_package_dir(&source).unwrap();
        let tmp = tempfile::tempdir().unwrap();
        write_package_dir(tmp.path(), &package).unwrap();
        copy_sources(source.join("sources"), tmp.path().join("sources"));
        let decoded = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded.manifest.package_content_hash,
            package_content_hash(&decoded).unwrap()
        );
        assert_eq!(decoded.context_index, package.context_index);
        assert_eq!(decoded.lineage_events, package.lineage_events);
        assert_eq!(decoded.crosswalks, package.crosswalks);
    }

    #[test]
    fn refreshes_package_hashes_for_invalid_fixture_without_core_verifying() {
        let source = default_fixture_dir("l1-bad-weights");
        let tmp = tempfile::tempdir().unwrap();
        copy_fixture(source, tmp.path());
        let hash = refresh_package_hashes(tmp.path()).unwrap();
        let package = read_package_dir_unverified(tmp.path()).unwrap();
        assert_eq!(package.manifest.package_content_hash, hash);
        assert_eq!(package_content_hash(&package).unwrap(), hash);

        let err = read_package_dir(tmp.path()).unwrap_err();
        assert!(matches!(
            err,
            RhistIoError::Core(rhist_core::RhistCoreError::CrosswalkWeightSum { .. })
        ));
    }

    #[test]
    fn detects_package_hash_mismatch_when_not_sentinel() {
        let source = default_fixture_dir("l0-rename");
        let package = read_package_dir(&source).unwrap();
        let tmp = tempfile::tempdir().unwrap();
        write_package_dir(tmp.path(), &package).unwrap();
        copy_sources(source.join("sources"), tmp.path().join("sources"));

        let mut manifest: serde_json::Value = read_json(&tmp.path().join("manifest.json")).unwrap();
        manifest["package_content_hash"] = serde_json::Value::String(
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string(),
        );
        write_json_pretty(&tmp.path().join("manifest.json"), &manifest).unwrap();

        let err = read_package_dir(tmp.path()).unwrap_err();
        assert!(matches!(err, RhistIoError::PackageHashMismatch { .. }));
    }

    fn copy_fixture(source: PathBuf, dest: &Path) {
        for rel in [
            "manifest.json",
            "sources/source-index.json",
            "contexts/context-index.ndjson",
            "units/cycles.ndjson",
            "units/lineage-events.ndjson",
            "claims/claim-boundary.json",
        ] {
            let from = source.join(rel);
            let to = dest.join(rel);
            fs::create_dir_all(to.parent().unwrap()).unwrap();
            fs::copy(from, to).unwrap();
        }
        let crosswalks = source.join("units").join("crosswalks.ndjson");
        if crosswalks.exists() {
            let to = dest.join("units").join("crosswalks.ndjson");
            fs::create_dir_all(to.parent().unwrap()).unwrap();
            fs::copy(crosswalks, to).unwrap();
        }
        copy_sources(source.join("sources"), dest.join("sources"));
    }

    fn copy_sources(source: PathBuf, dest: PathBuf) {
        fs::create_dir_all(&dest).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_file() && entry.file_name() != "source-index.json" {
                fs::copy(entry.path(), dest.join(entry.file_name())).unwrap();
            }
        }
    }
}
