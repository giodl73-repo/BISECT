/// Atomic output writer for redistricting results.
///
/// Writes final_assignments and vra_analysis.json atomically using
/// rename-from-temp-file pattern. If the process dies between writes,
/// the temp files (.tmp.*) are present but the final files are not,
/// making the corrupt state detectable and restartable.
///
/// BOUNDARY invariant: both files are written together or neither is.
/// This eliminates the vra_mode premature-clear class of bugs where
/// vra_analysis.pkl was not written.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OutputError {
    #[error("IO error writing {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("output directory {0} does not exist (create it before writing)")]
    DirectoryMissing(PathBuf),
    #[error("corrupt state detected: temp file {0} exists without final file.\n\
             Recovery: call clean_corrupt_state(data_dir) then retry, or pass --reset to wipe and restart.")]
    CorruptState(PathBuf),
    #[error("JSON serialization error: {0}")]
    Json(String),
}

/// VRA analysis result written alongside final_assignments.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VraAnalysis {
    pub mm_count: usize,
    pub mm_districts: Vec<usize>,
    pub districts: Vec<VraDistrict>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VraDistrict {
    pub district: usize,
    pub pct_minority: f64,
    pub pct_black: f64,
    pub pct_hispanic: f64,
    pub is_mm: bool,
}

/// Write final_assignments (JSON), vra_analysis.json, and provenance.json
/// atomically.
///
/// All three files are written to temp paths first, then renamed together.
/// The rename is not filesystem-atomic on all platforms, but temp-file
/// detection on restart ensures partial writes are caught.
///
/// `provenance.json` is a sidecar capturing the running binary's identity
/// (redist_version, build_commit, build_date, rustc_version) so that any
/// downstream consumer of `final_assignments.json` can attest to its source.
/// This sidecar is always written, regardless of whether `--manifest` is set.
pub fn write_state_outputs(
    data_dir: &Path,
    assignments: &HashMap<usize, usize>,
    vra: Option<&VraAnalysis>,
) -> Result<(), OutputError> {
    if !data_dir.exists() {
        return Err(OutputError::DirectoryMissing(data_dir.to_path_buf()));
    }

    // Detect corrupt state from previous crashed run
    let tmp_assignments = data_dir.join(".final_assignments.tmp.json");
    let tmp_vra = data_dir.join(".vra_analysis.tmp.json");
    let tmp_provenance = data_dir.join(".provenance.tmp.json");
    if tmp_assignments.exists() || tmp_vra.exists() || tmp_provenance.exists() {
        return Err(OutputError::CorruptState(tmp_assignments));
    }

    // Write assignments to temp file
    let assignments_json = serde_json::to_string_pretty(assignments)
        .map_err(|e| OutputError::Json(e.to_string()))?;
    fs::write(&tmp_assignments, &assignments_json)
        .map_err(|e| OutputError::Io { path: tmp_assignments.clone(), source: e })?;

    // Write VRA analysis to temp file (if VRA mode)
    if let Some(vra_data) = vra {
        let vra_json = serde_json::to_string_pretty(vra_data)
            .map_err(|e| OutputError::Json(e.to_string()))?;
        fs::write(&tmp_vra, &vra_json)
            .map_err(|e| OutputError::Io { path: tmp_vra.clone(), source: e })?;
    }

    // Write provenance sidecar (always — captures binary identity for audit trail)
    let provenance = crate::provenance::Provenance::current();
    let provenance_json = serde_json::to_string_pretty(&provenance)
        .map_err(|e| OutputError::Json(e.to_string()))?;
    fs::write(&tmp_provenance, &provenance_json)
        .map_err(|e| OutputError::Io { path: tmp_provenance.clone(), source: e })?;

    // All writes succeeded — now rename atomically
    let final_assignments = data_dir.join("final_assignments.json");
    fs::rename(&tmp_assignments, &final_assignments)
        .map_err(|e| OutputError::Io { path: final_assignments, source: e })?;

    if vra.is_some() {
        let final_vra = data_dir.join("vra_analysis.json");
        fs::rename(&tmp_vra, &final_vra)
            .map_err(|e| OutputError::Io { path: final_vra, source: e })?;
    }

    let final_provenance = data_dir.join("provenance.json");
    fs::rename(&tmp_provenance, &final_provenance)
        .map_err(|e| OutputError::Io { path: final_provenance, source: e })?;

    Ok(())
}

/// Write district_summary.csv from per-district metrics.
pub fn write_district_summary(
    data_dir: &Path,
    rows: &[HashMap<String, String>],
) -> Result<(), OutputError> {
    if rows.is_empty() {
        return Ok(());
    }
    let csv_path = data_dir.join("district_summary.csv");
    let tmp_csv = data_dir.join(".district_summary.tmp.csv");

    // Collect and sort column names for deterministic CSV output.
    // HashMap iteration order is non-deterministic; sorting ensures
    // the same column order across runs and Rust versions.
    let mut headers: Vec<&str> = rows[0].keys().map(|k| k.as_str()).collect();
    headers.sort_unstable();

    let mut content = String::new();
    content.push_str(&headers.join(","));
    content.push('\n');
    for row in rows {
        let values: Vec<&str> = headers.iter()
            .map(|h| row.get(*h).map(|s| s.as_str()).unwrap_or(""))
            .collect();
        content.push_str(&values.join(","));
        content.push('\n');
    }

    fs::write(&tmp_csv, &content)
        .map_err(|e| OutputError::Io { path: tmp_csv.clone(), source: e })?;
    fs::rename(&tmp_csv, &csv_path)
        .map_err(|e| OutputError::Io { path: csv_path, source: e })?;

    Ok(())
}

/// Check if a data directory has temp files indicating a corrupt state.
pub fn has_corrupt_state(data_dir: &Path) -> bool {
    data_dir.join(".final_assignments.tmp.json").exists()
        || data_dir.join(".vra_analysis.tmp.json").exists()
        || data_dir.join(".provenance.tmp.json").exists()
}

/// Clean up temp files from a corrupt state (call before reprocessing).
pub fn clean_corrupt_state(data_dir: &Path) -> std::io::Result<()> {
    let tmps = [
        data_dir.join(".final_assignments.tmp.json"),
        data_dir.join(".vra_analysis.tmp.json"),
        data_dir.join(".provenance.tmp.json"),
        data_dir.join(".district_summary.tmp.csv"),
    ];
    for tmp in &tmps {
        if tmp.exists() {
            fs::remove_file(tmp)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_assignments() -> HashMap<usize, usize> {
        (0..10).map(|i| (i, if i < 5 { 1 } else { 2 })).collect()
    }

    fn sample_vra() -> VraAnalysis {
        VraAnalysis {
            mm_count: 1,
            mm_districts: vec![1],
            districts: vec![
                VraDistrict { district: 1, pct_minority: 0.511, pct_black: 0.42, pct_hispanic: 0.09, is_mm: true },
                VraDistrict { district: 2, pct_minority: 0.25, pct_black: 0.15, pct_hispanic: 0.10, is_mm: false },
            ],
        }
    }

    #[test]
    fn test_write_assignments_creates_file() {
        let tmp = TempDir::new().unwrap();
        write_state_outputs(tmp.path(), &sample_assignments(), None).unwrap();
        assert!(tmp.path().join("final_assignments.json").exists());
    }

    #[test]
    fn test_write_vra_creates_both_files() {
        let tmp = TempDir::new().unwrap();
        let vra = sample_vra();
        write_state_outputs(tmp.path(), &sample_assignments(), Some(&vra)).unwrap();
        assert!(tmp.path().join("final_assignments.json").exists());
        assert!(tmp.path().join("vra_analysis.json").exists());
    }

    #[test]
    fn test_no_temp_files_after_success() {
        let tmp = TempDir::new().unwrap();
        let vra = sample_vra();
        write_state_outputs(tmp.path(), &sample_assignments(), Some(&vra)).unwrap();
        // Temp files cleaned up after successful rename
        assert!(!has_corrupt_state(tmp.path()));
    }

    #[test]
    fn test_vra_json_is_valid_json() {
        let tmp = TempDir::new().unwrap();
        let vra = sample_vra();
        write_state_outputs(tmp.path(), &sample_assignments(), Some(&vra)).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("vra_analysis.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["mm_count"], 1);
        assert!(parsed["districts"].is_array());
    }

    #[test]
    fn test_provenance_sidecar_written_in_non_vra_run() {
        let tmp = TempDir::new().unwrap();
        write_state_outputs(tmp.path(), &sample_assignments(), None).unwrap();
        let prov_path = tmp.path().join("provenance.json");
        assert!(prov_path.exists(), "provenance.json should be written even without VRA");
        let parsed: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&prov_path).unwrap()).unwrap();
        assert!(parsed["redist_version"].is_string());
        assert!(parsed["redist_build_commit"].is_string());
        assert!(parsed["redist_build_date"].is_string());
        assert!(parsed["rustc_version"].is_string());
    }

    #[test]
    fn test_provenance_sidecar_written_in_vra_run() {
        let tmp = TempDir::new().unwrap();
        let vra = sample_vra();
        write_state_outputs(tmp.path(), &sample_assignments(), Some(&vra)).unwrap();
        assert!(tmp.path().join("provenance.json").exists());
        // All three primary outputs side by side
        assert!(tmp.path().join("final_assignments.json").exists());
        assert!(tmp.path().join("vra_analysis.json").exists());
    }

    #[test]
    fn test_provenance_temp_file_blocks_rerun() {
        let tmp = TempDir::new().unwrap();
        // Simulate crash mid-write: provenance temp left behind
        std::fs::write(tmp.path().join(".provenance.tmp.json"), b"partial").unwrap();
        assert!(has_corrupt_state(tmp.path()));
        let result = write_state_outputs(tmp.path(), &sample_assignments(), None);
        assert!(matches!(result, Err(OutputError::CorruptState(_))));
    }

    #[test]
    fn test_clean_corrupt_state_removes_provenance_temp() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".provenance.tmp.json"), b"x").unwrap();
        clean_corrupt_state(tmp.path()).unwrap();
        assert!(!has_corrupt_state(tmp.path()));
    }

    #[test]
    fn test_corrupt_state_detected() {
        let tmp = TempDir::new().unwrap();
        // Simulate a crashed run that left a temp file
        std::fs::write(tmp.path().join(".final_assignments.tmp.json"), b"partial").unwrap();
        assert!(has_corrupt_state(tmp.path()));
        let result = write_state_outputs(tmp.path(), &sample_assignments(), None);
        assert!(matches!(result, Err(OutputError::CorruptState(_))));
    }

    #[test]
    fn test_clean_corrupt_state() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".final_assignments.tmp.json"), b"partial").unwrap();
        clean_corrupt_state(tmp.path()).unwrap();
        assert!(!has_corrupt_state(tmp.path()));
    }

    #[test]
    fn test_missing_directory_error() {
        let result = write_state_outputs(
            Path::new("/nonexistent/path"),
            &sample_assignments(),
            None,
        );
        assert!(matches!(result, Err(OutputError::DirectoryMissing(_))));
    }

    #[test]
    fn test_write_district_summary_creates_csv() {
        let tmp = TempDir::new().unwrap();
        let rows = vec![
            [("district".into(), "1".into()), ("polsby_popper".into(), "0.34".into())]
                .into_iter().collect::<HashMap<String, String>>(),
            [("district".into(), "2".into()), ("polsby_popper".into(), "0.28".into())]
                .into_iter().collect::<HashMap<String, String>>(),
        ];
        write_district_summary(tmp.path(), &rows).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("district_summary.csv")).unwrap();
        assert!(content.contains("district"));
        assert!(content.contains("polsby_popper"));
        assert!(content.contains("0.34"));
    }

    #[test]
    fn test_district_summary_columns_deterministic() {
        // Same data written 3 times should produce identical column order
        let mut make_row = || {
            let mut m = std::collections::HashMap::new();
            m.insert("polsby_popper".to_string(), "0.34".to_string());
            m.insert("district".to_string(), "1".to_string());
            m.insert("reock".to_string(), "0.42".to_string());
            m
        };
        let mut headers_seen = vec![];
        for _ in 0..5 {
            let tmp = TempDir::new().unwrap();
            write_district_summary(tmp.path(), &[make_row()]).unwrap();
            let content = std::fs::read_to_string(tmp.path().join("district_summary.csv")).unwrap();
            let header = content.lines().next().unwrap().to_string();
            headers_seen.push(header);
        }
        for h in &headers_seen[1..] {
            assert_eq!(h, &headers_seen[0], "column order must be deterministic across runs");
        }
    }

    #[test]
    fn test_temp_files_do_not_exist_after_success() {
        let tmp = TempDir::new().unwrap();
        let vra = sample_vra();
        write_state_outputs(tmp.path(), &sample_assignments(), Some(&vra)).unwrap();
        // Direct filesystem check (not just has_corrupt_state)
        assert!(!tmp.path().join(".final_assignments.tmp.json").exists());
        assert!(!tmp.path().join(".vra_analysis.tmp.json").exists());
    }

    #[test]
    fn test_assignments_json_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let orig = sample_assignments();
        write_state_outputs(tmp.path(), &orig, None).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("final_assignments.json")).unwrap();
        let parsed: HashMap<String, usize> = serde_json::from_str(&content).unwrap();
        // Keys are string in JSON; check count matches
        assert_eq!(parsed.len(), orig.len());
    }

    // ── 20 additional L0 tests ───────────────────────────────────────────────

    // -- write_state_outputs error cases -------------------------------------

    #[test]
    fn test_write_outputs_nonexistent_dir_returns_directory_missing() {
        let result = write_state_outputs(
            Path::new("/this/path/absolutely/does/not/exist"),
            &sample_assignments(),
            None,
        );
        match result {
            Err(OutputError::DirectoryMissing(p)) => {
                assert!(p.to_str().unwrap().contains("does/not/exist")
                    || p.to_str().unwrap().contains("does\\not\\exist"),
                    "error path must echo the bad directory: {:?}", p);
            }
            other => panic!("expected DirectoryMissing, got {:?}", other),
        }
    }

    #[test]
    fn test_write_outputs_vra_temp_file_triggers_corrupt_state() {
        let tmp = TempDir::new().unwrap();
        // Simulate a crash that left the VRA temp file
        std::fs::write(tmp.path().join(".vra_analysis.tmp.json"), b"corrupt").unwrap();
        assert!(has_corrupt_state(tmp.path()));
        let result = write_state_outputs(tmp.path(), &sample_assignments(), None);
        assert!(matches!(result, Err(OutputError::CorruptState(_))),
            "vra tmp file must trigger CorruptState");
    }

    #[test]
    fn test_write_outputs_all_three_temp_files_trigger_corrupt_state() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".final_assignments.tmp.json"), b"a").unwrap();
        std::fs::write(tmp.path().join(".vra_analysis.tmp.json"), b"b").unwrap();
        std::fs::write(tmp.path().join(".provenance.tmp.json"), b"c").unwrap();
        assert!(has_corrupt_state(tmp.path()));
        let result = write_state_outputs(tmp.path(), &sample_assignments(), None);
        assert!(matches!(result, Err(OutputError::CorruptState(_))));
    }

    // -- District assignment serialization roundtrip -------------------------

    #[test]
    fn test_empty_assignments_serializes_and_writes() {
        let tmp = TempDir::new().unwrap();
        let empty: HashMap<usize, usize> = HashMap::new();
        // Should succeed with an empty map
        write_state_outputs(tmp.path(), &empty, None).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("final_assignments.json")).unwrap();
        let parsed: HashMap<String, usize> = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_empty(), "empty assignments must round-trip as empty map");
    }

    #[test]
    fn test_single_district_assignment_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let mut single: HashMap<usize, usize> = HashMap::new();
        single.insert(42, 1);
        write_state_outputs(tmp.path(), &single, None).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("final_assignments.json")).unwrap();
        let parsed: HashMap<String, usize> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed.get("42"), Some(&1usize));
    }

    #[test]
    fn test_max_district_count_roundtrip() {
        // 435 districts: one tract per district (congressional scale)
        let tmp = TempDir::new().unwrap();
        let large: HashMap<usize, usize> = (0..435).map(|i| (i, i % 435 + 1)).collect();
        write_state_outputs(tmp.path(), &large, None).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("final_assignments.json")).unwrap();
        let parsed: HashMap<String, usize> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.len(), 435, "435 tracts must all round-trip");
    }

    #[test]
    fn test_assignments_all_same_district() {
        // All tracts in district 1 — edge case for single-district states
        let tmp = TempDir::new().unwrap();
        let one_dist: HashMap<usize, usize> = (0..20).map(|i| (i, 1)).collect();
        write_state_outputs(tmp.path(), &one_dist, None).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("final_assignments.json")).unwrap();
        let parsed: HashMap<String, usize> = serde_json::from_str(&content).unwrap();
        assert!(parsed.values().all(|&d| d == 1), "all tracts must map to district 1");
    }

    // -- VraDistrict / VraAnalysis struct construction and field access -------

    #[test]
    fn test_vra_district_field_access() {
        let d = VraDistrict {
            district: 5,
            pct_minority: 0.611,
            pct_black: 0.50,
            pct_hispanic: 0.111,
            is_mm: true,
        };
        assert_eq!(d.district, 5);
        assert!((d.pct_minority - 0.611).abs() < 1e-9);
        assert!(d.is_mm);
        assert!(!VraDistrict {
            district: 1, pct_minority: 0.1, pct_black: 0.05, pct_hispanic: 0.05, is_mm: false
        }.is_mm);
    }

    #[test]
    fn test_vra_analysis_empty_districts() {
        let vra = VraAnalysis {
            mm_count: 0,
            mm_districts: vec![],
            districts: vec![],
        };
        assert_eq!(vra.mm_count, 0);
        assert!(vra.mm_districts.is_empty());
        assert!(vra.districts.is_empty());
    }

    #[test]
    fn test_vra_analysis_json_roundtrip() {
        let vra = sample_vra();
        let json = serde_json::to_string_pretty(&vra).unwrap();
        let parsed: VraAnalysis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mm_count, vra.mm_count);
        assert_eq!(parsed.mm_districts, vra.mm_districts);
        assert_eq!(parsed.districts.len(), vra.districts.len());
        assert!((parsed.districts[0].pct_minority - vra.districts[0].pct_minority).abs() < 1e-9);
    }

    #[test]
    fn test_vra_analysis_multiple_mm_districts() {
        let vra = VraAnalysis {
            mm_count: 3,
            mm_districts: vec![2, 5, 9],
            districts: vec![
                VraDistrict { district: 2, pct_minority: 0.56, pct_black: 0.30, pct_hispanic: 0.26, is_mm: true },
                VraDistrict { district: 5, pct_minority: 0.52, pct_black: 0.40, pct_hispanic: 0.12, is_mm: true },
                VraDistrict { district: 9, pct_minority: 0.53, pct_black: 0.25, pct_hispanic: 0.28, is_mm: true },
            ],
        };
        assert_eq!(vra.mm_count, 3);
        assert_eq!(vra.mm_districts.len(), 3);
        assert!(vra.districts.iter().all(|d| d.is_mm));
    }

    // -- state_already_complete / done marker tests --------------------------

    #[test]
    fn test_has_corrupt_state_false_when_directory_empty() {
        let tmp = TempDir::new().unwrap();
        assert!(!has_corrupt_state(tmp.path()),
            "fresh empty dir must not be corrupt");
    }

    #[test]
    fn test_clean_corrupt_state_idempotent_on_clean_dir() {
        // clean_corrupt_state on a dir with no tmp files must succeed (no error)
        let tmp = TempDir::new().unwrap();
        clean_corrupt_state(tmp.path()).unwrap();
        assert!(!has_corrupt_state(tmp.path()));
    }

    #[test]
    fn test_clean_corrupt_state_removes_district_summary_tmp() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".district_summary.tmp.csv"), b"x").unwrap();
        // has_corrupt_state only checks the three JSON temps — district_summary tmp is extra
        clean_corrupt_state(tmp.path()).unwrap();
        assert!(!tmp.path().join(".district_summary.tmp.csv").exists(),
            "district_summary tmp must be removed by clean_corrupt_state");
    }

    // -- district_summary helpers --------------------------------------------

    #[test]
    fn test_write_district_summary_empty_rows_is_noop() {
        let tmp = TempDir::new().unwrap();
        write_district_summary(tmp.path(), &[]).unwrap();
        // When rows is empty, no file is written
        assert!(!tmp.path().join("district_summary.csv").exists(),
            "empty rows must not create CSV file");
    }

    #[test]
    fn test_write_district_summary_missing_column_fills_empty_string() {
        let tmp = TempDir::new().unwrap();
        let mut row1: HashMap<String, String> = HashMap::new();
        row1.insert("district".into(), "1".into());
        row1.insert("score".into(), "0.42".into());
        // row2 is missing "score" — should fill with ""
        let mut row2: HashMap<String, String> = HashMap::new();
        row2.insert("district".into(), "2".into());
        write_district_summary(tmp.path(), &[row1, row2]).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("district_summary.csv")).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3, "header + 2 data rows");
        // The second data row must have a trailing comma or empty value for missing column
        assert!(lines[2].contains(','), "row with missing column must still have comma separator");
    }

    #[test]
    fn test_write_district_summary_single_row() {
        let tmp = TempDir::new().unwrap();
        let mut row: HashMap<String, String> = HashMap::new();
        row.insert("district".into(), "1".into());
        row.insert("polsby_popper".into(), "0.35".into());
        write_district_summary(tmp.path(), &[row]).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("district_summary.csv")).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2, "header + 1 data row");
        assert!(content.contains("0.35"));
    }

    // -- output path / version-year tests ------------------------------------

    #[test]
    fn test_provenance_written_for_any_version_year() {
        // Provenance is always written regardless of what assignments or VRA we pass
        let tmp = TempDir::new().unwrap();
        let assignments: HashMap<usize, usize> = [(0, 1), (1, 1)].into_iter().collect();
        write_state_outputs(tmp.path(), &assignments, None).unwrap();
        let prov_path = tmp.path().join("provenance.json");
        assert!(prov_path.exists());
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&prov_path).unwrap()).unwrap();
        // Must have all four required provenance fields
        assert!(val["redist_version"].is_string());
        assert!(val["redist_build_commit"].is_string());
        assert!(val["redist_build_date"].is_string());
        assert!(val["rustc_version"].is_string());
    }

    #[test]
    fn test_final_assignments_is_valid_json() {
        let tmp = TempDir::new().unwrap();
        write_state_outputs(tmp.path(), &sample_assignments(), None).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("final_assignments.json")).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(v.is_object(), "final_assignments.json must be a JSON object");
    }

    #[test]
    fn test_output_error_display_directory_missing() {
        let err = OutputError::DirectoryMissing(PathBuf::from("/bad/path"));
        let msg = err.to_string();
        assert!(msg.contains("bad/path") || msg.contains("bad\\path"),
            "error message must include path: {msg}");
        assert!(msg.contains("does not exist"),
            "error message must say does not exist: {msg}");
    }

    #[test]
    fn test_output_error_display_corrupt_state() {
        let err = OutputError::CorruptState(PathBuf::from("/tmp/.final_assignments.tmp.json"));
        let msg = err.to_string();
        assert!(msg.contains("corrupt") || msg.contains("temp"),
            "CorruptState error must mention corrupt/temp: {msg}");
        assert!(msg.contains("clean_corrupt_state") || msg.contains("--reset"),
            "CorruptState error must mention recovery path: {msg}");
    }

    // ── 8 bonus tests ───────────────────────────────────────────────────────

    #[test]
    fn test_vra_district_not_mm_pct_below_threshold() {
        let d = VraDistrict {
            district: 10,
            pct_minority: 0.489,
            pct_black: 0.20,
            pct_hispanic: 0.289,
            is_mm: false,
        };
        assert!(!d.is_mm, "district below 50% minority must not be MM");
        assert!((d.pct_minority - 0.489).abs() < 1e-9);
    }

    #[test]
    fn test_vra_analysis_clone() {
        let vra = sample_vra();
        let cloned = vra.clone();
        assert_eq!(cloned.mm_count, vra.mm_count);
        assert_eq!(cloned.mm_districts, vra.mm_districts);
        assert_eq!(cloned.districts.len(), vra.districts.len());
    }

    #[test]
    fn test_write_state_outputs_no_vra_creates_no_vra_file() {
        let tmp = TempDir::new().unwrap();
        write_state_outputs(tmp.path(), &sample_assignments(), None).unwrap();
        // Without VRA, vra_analysis.json must NOT be created
        assert!(!tmp.path().join("vra_analysis.json").exists(),
            "vra_analysis.json must not exist when vra=None");
    }

    #[test]
    fn test_has_corrupt_state_only_provenance_tmp() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".provenance.tmp.json"), b"x").unwrap();
        assert!(has_corrupt_state(tmp.path()),
            "only provenance tmp must still register as corrupt");
    }

    #[test]
    fn test_write_district_summary_values_preserved() {
        let tmp = TempDir::new().unwrap();
        let mut row: HashMap<String, String> = HashMap::new();
        row.insert("district".into(), "7".into());
        row.insert("polsby_popper".into(), "0.999".into());
        write_district_summary(tmp.path(), &[row]).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("district_summary.csv")).unwrap();
        assert!(content.contains("7"), "district value must appear in CSV");
        assert!(content.contains("0.999"), "metric value must appear in CSV");
    }

    #[test]
    fn test_write_state_outputs_assignments_json_is_object() {
        let tmp = TempDir::new().unwrap();
        write_state_outputs(tmp.path(), &sample_assignments(), Some(&sample_vra())).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("final_assignments.json")).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(v.is_object(), "final_assignments.json must be a JSON object, not array");
    }

    #[test]
    fn test_clean_corrupt_state_allows_subsequent_write() {
        let tmp = TempDir::new().unwrap();
        // Simulate corrupt state
        std::fs::write(tmp.path().join(".final_assignments.tmp.json"), b"partial").unwrap();
        // Clean it, then retry the write — must succeed
        clean_corrupt_state(tmp.path()).unwrap();
        assert!(!has_corrupt_state(tmp.path()));
        write_state_outputs(tmp.path(), &sample_assignments(), None).unwrap();
        assert!(tmp.path().join("final_assignments.json").exists());
    }

    #[test]
    fn test_output_error_json_message() {
        let err = OutputError::Json("unexpected end of file".to_string());
        let msg = err.to_string();
        assert!(msg.contains("JSON") || msg.contains("serialization"),
            "Json error must mention JSON: {msg}");
        assert!(msg.contains("unexpected end of file"),
            "Json error must include original message: {msg}");
    }
}
