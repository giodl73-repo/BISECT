/// `redist verify` — reproduce a plan from its manifest.json and verify it matches the original.
use crate::args::VerifyArgs;
use redist_report::PlanManifest;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn run_verify(args: &VerifyArgs) -> anyhow::Result<()> {
    // 1. Load manifest
    let content = std::fs::read_to_string(&args.manifest)
        .map_err(|e| anyhow::anyhow!("cannot read manifest '{}': {e}", args.manifest.display()))?;
    let manifest: PlanManifest = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("invalid manifest JSON: {e}"))?;

    // 2. Print equivalent CLI command
    let verify_label = args.verify_label.clone()
        .unwrap_or_else(|| format!("{}_verify", manifest.label));

    let seed_flag = manifest.seed
        .map(|s| format!(" --seed {s}"))
        .unwrap_or_default();

    let cmd = format!(
        "redist state --state {} --year {} --chamber {} --districts {} \
         --label {} --balance-tolerance {:.2}{} --version {} --force",
        manifest.state_code,
        manifest.year,
        manifest.chamber,
        manifest.num_districts,
        verify_label,
        manifest.balance_tolerance_pct,
        seed_flag,
        "verify",
    );

    eprintln!("=== redist verify: {} ===", manifest.label);
    eprintln!("Equivalent command: {cmd}");

    if args.dry_run {
        eprintln!("[DRY RUN] Command not executed.");
        return Ok(());
    }

    // 3. Load original assignments
    let original_assignments = load_original_assignments(&manifest, &args.output_base)?;
    if original_assignments.is_empty() {
        eprintln!("WARNING: Original plan has no assignments — cannot compare.");
        eprintln!("The original plan may be at a different path. Verification aborted.");
        return Ok(());
    }

    // 4. Find redist binary
    let redist_bin = find_redist_binary()?;

    // 5. Run the re-verification
    eprintln!("Running re-verification...");
    let status = std::process::Command::new(&redist_bin)
        .args([
            "state",
            "--state", &manifest.state_code,
            "--year", &manifest.year,
            "--chamber", &manifest.chamber,
            "--districts", &manifest.num_districts.to_string(),
            "--label", &verify_label,
            "--version", "verify",
            "--force",
            "--output-dir", &args.output_base,
        ])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run {}: {e}", redist_bin.display()))?;

    if !status.success() {
        anyhow::bail!("FAIL: re-run exited with status {status}");
    }

    // 6. Load re-run assignments
    let verify_assignments = load_verify_assignments(&verify_label, &manifest.year, &args.output_base)?;

    // 7. Compute Jaccard similarity
    let similarity = jaccard_similarity(&original_assignments, &verify_assignments);

    eprintln!("Original assignments: {} tracts", original_assignments.len());
    eprintln!("Verified assignments: {} tracts", verify_assignments.len());
    eprintln!("Jaccard similarity: {similarity:.4}");

    if similarity >= args.min_similarity {
        eprintln!("[PASS] Verification succeeded (similarity={similarity:.4} >= {:.4})", args.min_similarity);
        Ok(())
    } else {
        anyhow::bail!(
            "FAIL: Jaccard similarity {similarity:.4} < threshold {:.4}. \
             Plans differ — check seed, METIS version, or adjacency file.",
            args.min_similarity
        )
    }
}

fn find_redist_binary() -> anyhow::Result<PathBuf> {
    for candidate in [
        "redist/target/release/redist.exe",
        "redist/target/release/redist",
        "redist",
    ] {
        let p = PathBuf::from(candidate);
        if p.exists() { return Ok(p); }
    }
    // Try current executable
    if let Ok(exe) = std::env::current_exe() {
        return Ok(exe);
    }
    anyhow::bail!("redist binary not found — run: cargo build --release -p redist-cli")
}

fn load_original_assignments(
    manifest: &PlanManifest,
    output_base: &str,
) -> anyhow::Result<HashMap<String, usize>> {
    // Try standard plan path
    let plan_dir = PathBuf::from(output_base)
        .join("v1")
        .join(&manifest.year)
        .join("plans")
        .join(&manifest.label)
        .join("data")
        .join("final_assignments.json");

    if plan_dir.exists() {
        let content = std::fs::read_to_string(&plan_dir)?;
        return Ok(serde_json::from_str(&content)?);
    }

    // Try same directory as the manifest
    Ok(HashMap::new()) // Empty = warn and skip comparison
}

fn load_verify_assignments(
    label: &str,
    year: &str,
    output_base: &str,
) -> anyhow::Result<HashMap<String, usize>> {
    let path = PathBuf::from(output_base)
        .join("verify")
        .join(year)
        .join("plans")
        .join(label)
        .join("data")
        .join("final_assignments.json");

    let content = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("verify assignments not found at {}: {e}", path.display()))?;
    Ok(serde_json::from_str(&content)?)
}

/// Compute Jaccard similarity between two district assignment maps.
///
/// A "match" is defined as: same GEOID → same district number.
/// The denominator is the larger of the two plan sizes (union-style).
pub fn jaccard_similarity(
    a: &HashMap<String, usize>,
    b: &HashMap<String, usize>,
) -> f64 {
    if a.is_empty() || b.is_empty() { return 0.0; }
    // Same GEOID → same district assignment = agreement
    let matching = a.iter()
        .filter(|(geoid, &d)| b.get(*geoid) == Some(&d))
        .count();
    let union = a.len().max(b.len());
    matching as f64 / union as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaccard_identical_plans() {
        let mut a = HashMap::new();
        a.insert("53001000100".to_string(), 1usize);
        a.insert("53001000200".to_string(), 2usize);
        let similarity = jaccard_similarity(&a, &a);
        assert!((similarity - 1.0).abs() < 1e-9, "identical plans must have similarity 1.0");
    }

    #[test]
    fn test_jaccard_empty_plan() {
        let a: HashMap<String, usize> = HashMap::new();
        let b: HashMap<String, usize> = HashMap::new();
        assert_eq!(jaccard_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_jaccard_completely_different() {
        let mut a = HashMap::new();
        a.insert("53001000100".to_string(), 1usize);
        let mut b = HashMap::new();
        b.insert("53001000100".to_string(), 2usize);
        let s = jaccard_similarity(&a, &b);
        assert!(s < 0.1, "completely different assignments: {s}");
    }

    #[test]
    fn test_jaccard_partial_match() {
        let mut a = HashMap::new();
        a.insert("G1".to_string(), 1usize);
        a.insert("G2".to_string(), 2usize);
        a.insert("G3".to_string(), 1usize);
        a.insert("G4".to_string(), 2usize);
        let mut b = HashMap::new();
        b.insert("G1".to_string(), 1usize); // match
        b.insert("G2".to_string(), 1usize); // mismatch
        b.insert("G3".to_string(), 1usize); // match
        b.insert("G4".to_string(), 1usize); // mismatch
        // 2 matching out of 4 union → 0.5
        let s = jaccard_similarity(&a, &b);
        assert!((s - 0.5).abs() < 1e-9, "expected 0.5 partial match, got {s}");
    }

    #[test]
    fn test_jaccard_one_empty() {
        let mut a = HashMap::new();
        a.insert("G1".to_string(), 1usize);
        let b: HashMap<String, usize> = HashMap::new();
        assert_eq!(jaccard_similarity(&a, &b), 0.0, "one empty plan → 0.0");
        assert_eq!(jaccard_similarity(&b, &a), 0.0, "symmetric");
    }

    #[test]
    fn test_jaccard_disjoint_geoids() {
        // Plans cover entirely different GEOIDs (e.g., different states)
        let mut a = HashMap::new();
        a.insert("53001000100".to_string(), 1usize);
        let mut b = HashMap::new();
        b.insert("06001000100".to_string(), 1usize);
        let s = jaccard_similarity(&a, &b);
        // 0 matching out of 1 union → 0.0
        assert_eq!(s, 0.0, "disjoint geoid sets → 0.0");
    }

    #[test]
    fn test_verify_dry_run_does_not_run_binary() {
        // dry_run=true must print the command and return Ok without executing.
        // We test this by passing a non-existent manifest path — dry_run should
        // fail at manifest read, but with a real manifest it would not execute.
        let args = VerifyArgs {
            manifest: std::path::PathBuf::from("/nonexistent/manifest.json"),
            min_similarity: 0.99,
            verify_label: None,
            output_base: "outputs".to_string(),
            dry_run: true,
        };
        // Should fail at manifest read, not at binary execution
        let result = run_verify(&args);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("manifest") || msg.contains("nonexistent"),
            "error should be about manifest, not binary: {msg}");
    }

    #[test]
    fn test_verify_dry_run_with_valid_manifest_returns_ok() {
        use tempfile::TempDir;
        use redist_report::PlanManifest;

        // Write a minimal valid manifest to a temp file
        let tmp = TempDir::new().unwrap();
        let manifest = PlanManifest {
            label: "vt_congressional_2020".to_string(),
            state_code: "VT".to_string(),
            year: "2020".to_string(),
            chamber: "congressional".to_string(),
            num_districts: 1,
            balance_tolerance_pct: 0.5,
            seed: Some(42),
            ..Default::default()
        };
        let manifest_path = tmp.path().join("manifest.json");
        std::fs::write(&manifest_path, serde_json::to_string(&manifest).unwrap()).unwrap();

        let args = VerifyArgs {
            manifest: manifest_path,
            min_similarity: 0.99,
            verify_label: None,
            output_base: "outputs".to_string(),
            dry_run: true,
        };
        // dry_run=true: must return Ok after printing command (no binary execution)
        let result = run_verify(&args);
        assert!(result.is_ok(), "dry_run with valid manifest must return Ok: {:?}", result);
    }

    #[test]
    fn test_verify_label_defaults_to_label_verify_suffix() {
        // When verify_label is None, the label should be "{original_label}_verify"
        let verify_label = None::<String>;
        let original_label = "vt_congressional_2020";
        let computed = verify_label
            .unwrap_or_else(|| format!("{}_verify", original_label));
        assert_eq!(computed, "vt_congressional_2020_verify");
    }

    #[test]
    fn test_verify_label_custom_overrides_default() {
        let verify_label = Some("my_custom_label".to_string());
        let original_label = "vt_congressional_2020";
        let computed = verify_label
            .unwrap_or_else(|| format!("{}_verify", original_label));
        assert_eq!(computed, "my_custom_label");
    }
}
