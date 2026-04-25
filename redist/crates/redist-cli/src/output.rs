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
    #[error("corrupt state detected: temp file {0} exists without final file — reprocess this state")]
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

/// Write final_assignments (JSON) and vra_analysis.json atomically.
///
/// Both files are written to temp paths first, then renamed together.
/// The rename is not filesystem-atomic on all platforms, but temp-file
/// detection on restart ensures partial writes are caught.
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
    if tmp_assignments.exists() || tmp_vra.exists() {
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

    // Both writes succeeded — now rename atomically
    let final_assignments = data_dir.join("final_assignments.json");
    fs::rename(&tmp_assignments, &final_assignments)
        .map_err(|e| OutputError::Io { path: final_assignments, source: e })?;

    if vra.is_some() {
        let final_vra = data_dir.join("vra_analysis.json");
        fs::rename(&tmp_vra, &final_vra)
            .map_err(|e| OutputError::Io { path: final_vra, source: e })?;
    }

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
}

/// Clean up temp files from a corrupt state (call before reprocessing).
pub fn clean_corrupt_state(data_dir: &Path) -> std::io::Result<()> {
    let tmps = [
        data_dir.join(".final_assignments.tmp.json"),
        data_dir.join(".vra_analysis.tmp.json"),
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
}
