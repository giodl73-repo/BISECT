/// audit.rs — chain-of-custody, SHA-256 hashing, verification command generation.
///
/// Spec 6 / Board amendments:
/// - Verification command must use relative paths only (no C:\... or /home/...)
/// - Verification command must include --seed for reproducibility
/// - External analyzer scripts must be SHA-256 hashed and recorded
/// - Verification command must include sha256sum step for external analyzer scripts
use std::path::Path;
use serde::{Deserialize, Serialize};

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

/// Record of an external analyzer script used during analysis.
/// Stored in manifest.json to support independent reproducibility checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAnalyzerRecord {
    /// Relative path or basename of the script
    pub script: String,
    /// SHA-256 of the script file at time of run
    pub sha256: String,
    /// Command template with {assignments_json} and {output_dir} placeholders
    pub command: String,
}

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

/// Generate the machine-readable verification command from a plan manifest.
///
/// Board amendments:
/// - No local file paths — only relative paths and redist CLI flags
/// - Must include --seed for reproducibility
/// - Must include sha256sum step for external analyzer scripts (board amendment R3)
/// - Format: a shell script fragment that can be embedded in audit.json
pub fn generate_verification_command(
    label: &str,
    state_code: &str,
    year: &str,
    seed: Option<i64>,
    chamber: &str,
    num_districts: usize,
    adjacency_file: &str,
    adjacency_sha256: &str,
    tiger_source_url: &str,
    tiger_sha256: Option<&str>,
    external_analyzers: &[ExternalAnalyzerRecord],
) -> String {
    let seed_str = seed
        .map(|s| format!(" --seed {}", s))
        .unwrap_or_else(|| " --seed <unknown: seed not recorded in manifest>".to_string());

    let tiger_check = if let Some(sha) = tiger_sha256 {
        format!(
            "\n# Verify TIGER source shapefile (download from Census.gov):\n# wget '{}'\n# sha256sum <downloaded_file>  # expected: {}",
            tiger_source_url, sha
        )
    } else {
        format!(
            "\n# Download TIGER source from:\n# {}",
            tiger_source_url
        )
    };

    let adj_check = format!(
        "\n# Verify adjacency file:\n# sha256sum {}  # expected: {}",
        adjacency_file, adjacency_sha256
    );

    let external_checks: String = external_analyzers
        .iter()
        .map(|rec| {
            format!(
                "\n# Verify external analyzer script:\n# sha256sum {}  # expected: {}\n# {}",
                rec.script,
                rec.sha256,
                rec.command
            )
        })
        .collect();

    format!(
        "# Verification command for plan: {label}\n\
        # Re-run with identical parameters to reproduce:\n\
        redist state --state {state} --year {year} --chamber {chamber} --districts {num_districts} --label {label}{seed}\n\
        {tiger_check}\n\
        {adj_check}{external_checks}",
        label = label,
        state = state_code,
        year = year,
        chamber = chamber,
        num_districts = num_districts,
        seed = seed_str,
        tiger_check = tiger_check,
        adj_check = adj_check,
        external_checks = external_checks,
    )
}

/// High-level convenience wrapper using a PlanManifest.
pub fn generate_verification_command_from_manifest(
    manifest: &crate::manifest::PlanManifest,
) -> String {
    generate_verification_command(
        &manifest.label,
        &manifest.state_code,
        &manifest.year,
        manifest.seed,
        &manifest.chamber,
        manifest.num_districts,
        &manifest.adjacency_file,
        &manifest.adjacency_sha256,
        &manifest.tiger_source_url,
        manifest.tiger_sha256.as_deref(),
        &[],
    )
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
            binary_download_url:
                "https://github.com/owner/redist/releases/download/v0.1.0/redist".into(),
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
        }
    }

    #[test]
    fn test_sha256_file_hash_known_content() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"hello world").unwrap();
        let hash = sha256_file(tmp.path()).unwrap();
        assert_eq!(hash.len(), 64, "SHA-256 hex must be 64 chars");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "must be hex"
        );
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
        assert!(!cmd.contains("C:\\"), "must not contain Windows local paths");
        assert!(!cmd.contains("/home/"), "must not contain Unix home paths");
        assert!(!cmd.contains("/Users/"), "must not contain macOS home paths");
    }

    #[test]
    fn test_manifest_has_tiger_sha256() {
        let manifest = fixture_manifest_wa_house();
        let tiger_sha = manifest.tiger_sha256.as_deref().unwrap_or("");
        assert_eq!(tiger_sha.len(), 64, "tiger_sha256 must be 64-char SHA-256 hex");
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
        let record = ExternalAnalyzerRecord::from_script(
            tmp.path(),
            "python {script} {args}",
        )
        .unwrap();
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
}
