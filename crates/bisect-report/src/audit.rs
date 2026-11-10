/// audit.rs — chain-of-custody, SHA-256 hashing, verification command generation.
///
/// Spec 6 / Board amendments:
/// - Verification command must use relative paths only (no C:\... or /home/...)
/// - Verification command must include --seed for reproducibility
/// - External analyzer scripts must be SHA-256 hashed and recorded
/// - Verification command must include sha256sum step for external analyzer scripts
use std::path::Path;

/// Compute SHA-256 of a file by streaming in 64KB chunks.
/// Returns lowercase 64-character hex string.
pub fn sha256_file(path: &Path) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub use crate::verification::{ExternalAnalyzerRecord, generate_verification_command, generate_verification_command_from_manifest, generate_verification_script};

impl ExternalAnalyzerRecord {
    /// Create a record from a script path and command template.
    /// Hashes the script file and stores the SHA-256.
    pub fn from_script(script_path: &Path, command_template: &str) -> anyhow::Result<Self> {
        let sha256 = sha256_file(script_path)?;
        Ok(Self {
            script: script_path.to_string_lossy().into_owned(),
            sha256,
            command: command_template.to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests — Task 1
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn fixture_manifest_wa_house() -> crate::manifest::PlanManifest {
        crate::manifest::PlanManifest {
            label: "wa_house_draft1".into(),
            state_code: "WA".into(),
            year: "2020".into(),
            chamber: "house".into(),
            num_districts: 98,
            population_source: "total".into(),
            partition_mode: "edge-weighted".into(),
            seed: Some(42),
            binary_version: "0.1.0".into(),
            binary_sha256: "a".repeat(64),
            binary_download_url: "https://github.com/owner/BISECT/releases/download/v0.1.0/BISECT"
                .into(),
            adjacency_file: "wa_adjacency_2020.adj.bin".into(),
            adjacency_sha256: "b".repeat(64),
            adjacency_build_command: "python scripts/data/generate_adj_bin.py".into(),
            adjacency_build_version: "0.1.0".into(),
            tiger_source_url:
                "https://www2.census.gov/geo/tiger/TIGER2020/TRACT/tl_2020_53_tract.zip".into(),
            tiger_sha256: Some("c".repeat(64)),
            created_at: "2026-04-26T00:00:00Z".into(),
            balance_tolerance_pct: 0.5,
            population_balance_valid: true,
            seats_per_district: 1,
            total_seats: 98,
            electoral_system: "single_member".into(),
            gpmetis_version: "METIS 5.1.0".into(),
            ..Default::default()
        }
    }

    #[test]
    fn test_sha256_file_hash_known_content() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"hello world").unwrap();
        let hash = sha256_file(tmp.path()).unwrap();
        assert_eq!(hash.len(), 64, "SHA-256 hex must be 64 chars");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()), "must be hex");
        // Deterministic
        let hash2 = sha256_file(tmp.path()).unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_audit_command_is_reproducible() {
        let manifest = fixture_manifest_wa_house();
        let cmd1 = generate_verification_command_from_manifest(&manifest);
        let cmd2 = generate_verification_command_from_manifest(&manifest);
        assert_eq!(cmd1, cmd2, "verification command must be deterministic");
    }

    #[test]
    fn test_verification_command_contains_seed() {
        let manifest = fixture_manifest_wa_house();
        let cmd = generate_verification_command_from_manifest(&manifest);
        assert!(
            cmd.contains("--seed"),
            "verification command must include --seed for reproducibility"
        );
    }

    #[test]
    fn test_verification_command_no_local_paths() {
        let manifest = fixture_manifest_wa_house();
        let cmd = generate_verification_command_from_manifest(&manifest);
        assert!(
            !cmd.contains("C:\\"),
            "must not contain Windows local paths"
        );
        assert!(!cmd.contains("/home/"), "must not contain Unix home paths");
        assert!(
            !cmd.contains("/Users/"),
            "must not contain macOS home paths"
        );
    }

    #[test]
    fn test_manifest_has_tiger_sha256() {
        let manifest = fixture_manifest_wa_house();
        let tiger_sha = manifest.tiger_sha256.as_deref().unwrap_or("");
        assert_eq!(
            tiger_sha.len(),
            64,
            "tiger_sha256 must be 64-char SHA-256 hex"
        );
        assert!(tiger_sha.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_manifest_has_binary_download_url() {
        let manifest = fixture_manifest_wa_house();
        let url = manifest.binary_download_url.as_str();
        assert!(
            url.contains("github.com"),
            "binary_download_url must reference github.com"
        );
        assert!(url.starts_with("https://"), "must be HTTPS URL");
    }

    #[test]
    fn test_manifest_has_binary_release_sha256() {
        let manifest = fixture_manifest_wa_house();
        // binary_sha256 is the binary release SHA-256
        let sha = &manifest.binary_sha256;
        assert_eq!(sha.len(), 64, "binary_sha256 must be 64-char hex");
    }

    #[test]
    fn test_external_analyzer_script_sha256_recorded() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"print('hello')").unwrap();
        let record =
            ExternalAnalyzerRecord::from_script(tmp.path(), "python {script} {args}").unwrap();
        assert_eq!(record.sha256.len(), 64);
        // Manifest round-trip preserves record
        let json = serde_json::to_string(&record).unwrap();
        let decoded: ExternalAnalyzerRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.sha256, record.sha256);
    }

    #[test]
    fn test_verification_command_includes_external_script_hash() {
        // Board amendment: verification command must include sha256sum step for external scripts
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"# trivial analyzer\nprint('ok')").unwrap();
        let rec = ExternalAnalyzerRecord::from_script(
            tmp.path(),
            "python my_script.py {assignments_json} {output_dir}",
        )
        .unwrap();

        let cmd = generate_verification_command(
            "vt_congressional_2020",
            "VT",
            "2020",
            Some(99),
            "congressional",
            1,
            "vt_adjacency_2020.adj.bin",
            &"d".repeat(64),
            "https://www2.census.gov/geo/tiger/TIGER2020/TRACT/tl_2020_50_tract.zip",
            Some(&"e".repeat(64)),
            &[rec.clone()],
        );

        assert!(
            cmd.contains("sha256sum"),
            "verification command must include sha256sum step for external scripts"
        );
        assert!(
            cmd.contains(&rec.sha256),
            "verification command must contain external script SHA-256 hash"
        );
    }

    #[test]
    fn test_verification_command_tiger_sha256_none_is_descriptive() {
        // When tiger_sha256 is None, the verification command must NOT say "null".
        // It must say something actionable about downloading the file manually.
        let cmd = generate_verification_command(
            "vt_congressional_2020",
            "VT",
            "2020",
            Some(42),
            "congressional",
            1,
            "vt_adjacency_2020.adj.bin",
            &"a".repeat(64),
            "https://www2.census.gov/geo/tiger/TIGER2020/TRACT/tl_2020_50_tract.zip",
            None, // no tiger sha256
            &[],
        );
        assert!(
            !cmd.contains("null"),
            "verification command must not contain raw 'null': {cmd}"
        );
        assert!(
            cmd.contains("Download") || cmd.contains("download"),
            "must tell user to download TIGER file: {cmd}"
        );
    }

    // ── Task 150: generate_verification_script ────────────────────────────────

    #[test]
    fn test_verification_script_has_no_comment_lines() {
        let manifest = fixture_manifest_wa_house();
        let script = generate_verification_script(&manifest);
        for line in script.lines() {
            let trimmed = line.trim();
            assert!(
                !trimmed.starts_with('#'),
                "verification_script must not contain comment lines, but got: {trimmed}"
            );
        }
    }

    #[test]
    fn test_verification_script_contains_bisect_state() {
        let manifest = fixture_manifest_wa_house();
        let script = generate_verification_script(&manifest);
        assert!(
            script.contains("bisect state"),
            "verification_script must contain the bisect state command, got: {script}"
        );
    }

    #[test]
    fn test_verification_script_is_nonempty() {
        let manifest = fixture_manifest_wa_house();
        let script = generate_verification_script(&manifest);
        assert!(
            !script.trim().is_empty(),
            "verification_script must not be empty"
        );
    }

    #[test]
    fn test_audit_report_tiger_sha256_none_shows_descriptive_string() {
        // Scenario 36: In the assembled report audit section, tiger_sha256: None
        // must produce a descriptive placeholder, not a raw JSON null.
        use crate::manifest::PlanManifest;
        use crate::report::{assemble_report, ReportContext, REQUIRED_ANALYSIS_FILES};
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let plan_dir = tmp.path().join("plans").join("vt_test");
        let analysis_dir = plan_dir.join("analysis");
        std::fs::create_dir_all(&analysis_dir).unwrap();
        for name in REQUIRED_ANALYSIS_FILES {
            std::fs::write(
                analysis_dir.join(name),
                serde_json::to_string(&serde_json::json!({"status": "ok"})).unwrap(),
            )
            .unwrap();
        }
        let manifest = PlanManifest {
            label: "vt_test".into(),
            state_code: "VT".into(),
            year: "2020".into(),
            tiger_sha256: None, // <-- the case we're testing
            ..Default::default()
        };
        let ctx = ReportContext::new(plan_dir, manifest);
        let report = assemble_report(&ctx).unwrap();
        let audit_tiger = &report.sections.audit["tiger_sha256"];
        assert!(
            audit_tiger.is_string(),
            "tiger_sha256 in audit must be a String, not null: {:?}",
            audit_tiger
        );
        let s = audit_tiger.as_str().unwrap();
        assert!(
            !s.is_empty() && s != "null",
            "tiger_sha256 audit string must be descriptive, got: {s}"
        );
        assert!(
            s.contains("not recorded") || s.contains("download") || s.contains("compute"),
            "tiger_sha256 audit string must explain what to do: {s}"
        );
    }
}
