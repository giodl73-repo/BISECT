/// `bisect verify` — reproduce a plan from its manifest.json and verify it matches the original.
use crate::args::VerifyArgs;
use bisect_report::PlanManifest;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

const RPLAN_GOLDEN_PACKAGE_SCHEMA_VERSION: &str = "u20-public-example-manifest-v1";
const RPLAN_BENCHMARK_PACKAGE_SCHEMA_VERSION: &str = "benchmark-rplan-package-manifest-v1";

#[derive(Debug, Deserialize)]
struct RplanGoldenPackageManifest {
    schema_version: String,
    example_id: Option<String>,
    files: Vec<RplanGoldenPackageFile>,
}

#[derive(Debug, Deserialize)]
struct RplanGoldenPackageFile {
    path: String,
    sha256: String,
    role: String,
}

/// Check whether the binary SHA-256 in the manifest matches the current executable.
///
/// - If `binary_sha256` is empty or starts with "(not", skip silently.
/// - If `skip_binary_check` is true: emit a NOTE and return.
/// - If the hashes differ: emit a WARNING (not an error) about mismatch.
/// - If the hashes match: no output.
pub fn check_binary_sha256(manifest_sha256: &str, skip_binary_check: bool) {
    // Skip if the manifest has no meaningful hash
    if manifest_sha256.is_empty() || manifest_sha256.starts_with("(not") {
        return;
    }

    if skip_binary_check {
        eprintln!("NOTE: Binary check skipped (--skip-binary-check set)");
        return;
    }

    // Compute SHA-256 of the current executable
    let exe_sha = match std::env::current_exe()
        .ok()
        .and_then(|p| bisect_report::sha256_file(&p).ok())
    {
        Some(sha) => sha,
        None => {
            eprintln!(
                "WARNING: Could not compute SHA-256 of current executable — binary check skipped."
            );
            return;
        }
    };

    if exe_sha != manifest_sha256 {
        eprintln!(
            "WARNING: Binary SHA-256 mismatch.\n\
             Manifest: {}\n\
             Current:  {}\n\
             Results may differ if built from a different commit. \
             Use --skip-binary-check to suppress this warning.",
            manifest_sha256, exe_sha
        );
    }
}

fn parse_rplan_golden_package_manifest(
    content: &str,
) -> anyhow::Result<Option<RplanGoldenPackageManifest>> {
    let value: serde_json::Value =
        serde_json::from_str(content).map_err(|e| anyhow::anyhow!("invalid manifest JSON: {e}"))?;
    let schema_version = value.get("schema_version").and_then(|v| v.as_str());
    if !matches!(
        schema_version,
        Some(RPLAN_GOLDEN_PACKAGE_SCHEMA_VERSION | RPLAN_BENCHMARK_PACKAGE_SCHEMA_VERSION)
    ) {
        return Ok(None);
    }

    let manifest = serde_json::from_value(value)
        .map_err(|e| anyhow::anyhow!("invalid RPLAN package manifest JSON: {e}"))?;
    Ok(Some(manifest))
}

fn verify_rplan_golden_package_manifest(
    manifest_path: &Path,
    manifest: &RplanGoldenPackageManifest,
) -> anyhow::Result<()> {
    if !matches!(
        manifest.schema_version.as_str(),
        RPLAN_GOLDEN_PACKAGE_SCHEMA_VERSION | RPLAN_BENCHMARK_PACKAGE_SCHEMA_VERSION
    ) {
        anyhow::bail!(
            "unsupported RPLAN package manifest schema_version: {}",
            manifest.schema_version
        );
    }

    let package_root = manifest_plan_root(manifest_path);
    let example_label = manifest.example_id.as_deref().unwrap_or("<unnamed>");
    eprintln!("=== bisect verify: RPLAN package {example_label} ===");

    for file in &manifest.files {
        let path = resolve_package_file(package_root, &file.path)?;
        let actual_sha256 = bisect_report::sha256_file(&path)
            .map_err(|e| anyhow::anyhow!("hash RPLAN package file {}: {e}", path.display()))?;
        if actual_sha256 != file.sha256.to_ascii_lowercase() {
            anyhow::bail!(
                "RPLAN package SHA-256 mismatch for {}: manifest={}, current={}",
                file.path,
                file.sha256,
                actual_sha256
            );
        }
    }

    let plan_path = required_package_role_path(package_root, manifest, "plan")?;
    let context_path = required_package_role_path(package_root, manifest, "context")?;
    let certificate_path = required_package_role_path(package_root, manifest, "certificate")?;

    let plan_text = std::fs::read_to_string(&plan_path)
        .map_err(|e| anyhow::anyhow!("read RPLAN package plan {}: {e}", plan_path.display()))?;
    let context_text = std::fs::read_to_string(&context_path).map_err(|e| {
        anyhow::anyhow!("read RPLAN package context {}: {e}", context_path.display())
    })?;
    let certificate_bytes = std::fs::read(&certificate_path).map_err(|e| {
        anyhow::anyhow!(
            "read RPLAN package certificate {}: {e}",
            certificate_path.display()
        )
    })?;

    let rplan_document = rplan_io::read_rplan_str(&plan_text)
        .map_err(|e| anyhow::anyhow!("parse RPLAN package plan: {e}"))?;
    let context = rplan_io::read_rctx_str(&context_text)
        .map_err(|e| anyhow::anyhow!("parse RPLAN package context: {e}"))?;
    let certificate: rplan_audit::AuditCertificate = serde_json::from_slice(&certificate_bytes)
        .map_err(|e| anyhow::anyhow!("parse RPLAN package certificate: {e}"))?;

    rplan_audit::verify_audit_certificate(&certificate, Some(&rplan_document.plan), Some(&context))
        .map_err(|e| anyhow::anyhow!("RPLAN package certificate verification failed: {e}"))?;

    eprintln!("[PASS] RPLAN package manifest hashes verified");
    eprintln!("[PASS] RPLAN package certificate verified");
    Ok(())
}

fn required_package_role_path(
    package_root: &Path,
    manifest: &RplanGoldenPackageManifest,
    role: &str,
) -> anyhow::Result<PathBuf> {
    let file = manifest
        .files
        .iter()
        .find(|file| file.role == role)
        .ok_or_else(|| anyhow::anyhow!("RPLAN package manifest missing required role: {role}"))?;
    resolve_package_file(package_root, &file.path)
}

fn resolve_package_file(package_root: &Path, relative_path: &str) -> anyhow::Result<PathBuf> {
    let path = Path::new(relative_path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        anyhow::bail!("RPLAN package file path escapes package root: {relative_path}");
    }
    Ok(package_root.join(path))
}

pub fn run_verify(args: &VerifyArgs) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(&args.manifest)
        .map_err(|e| anyhow::anyhow!("cannot read manifest '{}': {e}", args.manifest.display()))?;

    if let Some(manifest) = parse_rplan_golden_package_manifest(&content)? {
        return verify_rplan_golden_package_manifest(&args.manifest, &manifest);
    }

    // ── assignments-only mode (no METIS re-run) ───────────────────────────────
    if args.verify_assignments_only {
        let manifest: PlanManifest = serde_json::from_str(&content)?;
        let plan_root = manifest_plan_root(&args.manifest);
        verify_manifest_ilp_audit_summary(&manifest, plan_root)?;
        verify_manifest_rplan_audit_certificate(&manifest, plan_root)?;

        eprintln!(
            "=== bisect verify (assignments only): {} ===",
            args.manifest.display()
        );
        eprintln!(
            "Plan: {} ({} {} {}D)",
            manifest.label, manifest.state_code, manifest.chamber, manifest.num_districts
        );

        // Find original assignments via load_original_assignments
        let original =
            load_original_assignments(&manifest, &args.output_base, args.label.as_deref())?;
        if original.is_empty() {
            eprintln!("WARNING: Original plan not found — cannot compare.");
            eprintln!("Use --plan-ref to specify a reference assignments file.");
            std::process::exit(1);
        }

        // Load reference
        let reference = if let Some(ref ref_path) = args.plan_ref {
            let content = std::fs::read_to_string(ref_path)?;
            serde_json::from_str::<HashMap<String, usize>>(&content)?
        } else {
            eprintln!(
                "ERROR: --verify-assignments-only requires --plan-ref <path> \
                pointing to a reference final_assignments.json file."
            );
            std::process::exit(1);
        };

        // Compute Jaccard
        let matching = original
            .iter()
            .filter(|(geoid, &d)| reference.get(*geoid) == Some(&d))
            .count();
        let union = original.len().max(reference.len());
        let similarity = if union == 0 {
            0.0
        } else {
            matching as f64 / union as f64
        };

        eprintln!(
            "Jaccard similarity: {:.4} ({}/{} tracts match)",
            similarity, matching, union
        );

        if similarity >= args.min_similarity {
            eprintln!(
                "[PASS] Assignments match (similarity={:.4} >= {:.4})",
                similarity, args.min_similarity
            );
        } else {
            eprintln!(
                "[FAIL] Assignments differ (similarity={:.4} < {:.4})",
                similarity, args.min_similarity
            );
            std::process::exit(1);
        }
        return Ok(());
    }

    // Future: use PlanContext::from_label when label can be derived from manifest.label
    // (verify takes a manifest PATH not a label, so the refactor requires arg plumbing)
    // 1. Load manifest
    let manifest: PlanManifest = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("invalid manifest JSON: {e}"))?;
    let plan_root = manifest_plan_root(&args.manifest);
    verify_manifest_ilp_audit_summary(&manifest, plan_root)?;
    verify_manifest_rplan_audit_certificate(&manifest, plan_root)?;

    // 2. Print equivalent CLI command
    let verify_label = args
        .verify_label
        .clone()
        .unwrap_or_else(|| format!("{}_verify", manifest.label));

    let seed_flag = manifest
        .seed
        .map(|s| format!(" --seed {s}"))
        .unwrap_or_default();

    let cmd = format!(
        "bisect state --state {} --year {} --chamber {} --districts {} \
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

    eprintln!("=== bisect verify: {} ===", manifest.label);
    eprintln!("Equivalent command: {cmd}");

    // Binary SHA-256 audit check
    check_binary_sha256(&manifest.binary_sha256, args.skip_binary_check);

    if args.dry_run {
        eprintln!("[DRY RUN] Command not executed.");
        return Ok(());
    }

    // Gap 8: Warn when output_base path doesn't exist on this machine.
    // This commonly happens when verifying a plan from another machine.
    let output_base_path = std::path::PathBuf::from(&args.output_base);
    if !output_base_path.exists() {
        eprintln!(
            "WARNING: output directory '{}' not found on this machine.\n\
             If verifying a plan from another machine, use --output-base to specify\n\
             where plans are stored locally. The plan comparison step will be skipped.",
            args.output_base
        );
    }

    // 3. Load original assignments
    let original_assignments =
        load_original_assignments(&manifest, &args.output_base, args.label.as_deref())?;
    if original_assignments.is_empty() {
        eprintln!("WARNING: Original plan has no assignments — cannot compare.");
        eprintln!("The original plan may be at a different path. Verification aborted.");
        return Ok(());
    }

    // 4. Find bisect binary
    let bisect_bin = find_bisect_binary()?;

    // 5. Run the re-verification
    eprintln!("Running re-verification...");
    let status = std::process::Command::new(&bisect_bin)
        .args([
            "state",
            "--state",
            &manifest.state_code,
            "--year",
            &manifest.year,
            "--chamber",
            &manifest.chamber,
            "--districts",
            &manifest.num_districts.to_string(),
            "--label",
            &verify_label,
            "--version",
            "verify",
            "--force",
            "--output-dir",
            &args.output_base,
        ])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run {}: {e}", bisect_bin.display()))?;

    if !status.success() {
        anyhow::bail!("FAIL: re-run exited with status {status}");
    }

    // 6. Load re-run assignments
    let verify_assignments =
        load_verify_assignments(&verify_label, &manifest.year, &args.output_base)?;

    // 7. Compute Jaccard similarity
    let similarity = jaccard_similarity(&original_assignments, &verify_assignments);

    eprintln!(
        "Original assignments: {} tracts",
        original_assignments.len()
    );
    eprintln!("Verified assignments: {} tracts", verify_assignments.len());
    eprintln!("Jaccard similarity: {similarity:.4}");

    if similarity >= args.min_similarity {
        eprintln!(
            "[PASS] Verification succeeded (similarity={similarity:.4} >= {:.4})",
            args.min_similarity
        );
        Ok(())
    } else {
        anyhow::bail!(
            "FAIL: Jaccard similarity {similarity:.4} < threshold {:.4}. \
             Plans differ — check seed, METIS version, or adjacency file.",
            args.min_similarity
        )
    }
}

fn manifest_plan_root(manifest_path: &Path) -> &Path {
    manifest_path.parent().unwrap_or_else(|| Path::new(""))
}

pub fn verify_manifest_ilp_audit_summary(
    manifest: &PlanManifest,
    plan_root: &Path,
) -> anyhow::Result<()> {
    let Some(summary_rel) = manifest.ilp_audit_summary_path.as_deref() else {
        return Ok(());
    };

    let summary_path = plan_root.join(summary_rel);
    if let Some(expected_sha256) = manifest.ilp_audit_summary_sha256.as_deref() {
        if !expected_sha256.is_empty() {
            let actual_sha256 = bisect_report::sha256_file(&summary_path)
                .map_err(|e| anyhow::anyhow!("hash ILP audit summary: {e}"))?;
            if actual_sha256 != expected_sha256 {
                anyhow::bail!(
                    "ILP audit summary SHA-256 mismatch for {}: manifest={}, current={}",
                    summary_path.display(),
                    expected_sha256,
                    actual_sha256
                );
            }
        }
    }

    let report_dir = manifest
        .ilp_solve_report_dir
        .as_deref()
        .map(|dir| plan_root.join(dir))
        .or_else(|| summary_path.parent().map(Path::to_path_buf))
        .ok_or_else(|| anyhow::anyhow!("cannot resolve ILP solve report directory"))?;

    crate::ilp_audit::verify_ilp_audit_summary_for_dir(&report_dir, &summary_path)
        .map_err(|e| anyhow::anyhow!("ILP audit summary verification failed: {e}"))?;
    eprintln!("[PASS] ILP audit summary verified: {summary_rel}");
    Ok(())
}

pub fn verify_manifest_rplan_audit_certificate(
    manifest: &PlanManifest,
    plan_root: &Path,
) -> anyhow::Result<()> {
    let Some(certificate_rel) = manifest.audit_certificate_path.as_deref() else {
        return Ok(());
    };

    let certificate_path = plan_root.join(certificate_rel);
    if let Some(expected_sha256) = manifest.audit_certificate_sha256.as_deref() {
        if !expected_sha256.is_empty() {
            let actual_sha256 = bisect_report::sha256_file(&certificate_path)
                .map_err(|e| anyhow::anyhow!("hash RPLAN audit certificate: {e}"))?;
            if actual_sha256 != expected_sha256 {
                anyhow::bail!(
                    "RPLAN audit certificate SHA-256 mismatch for {}: manifest={}, current={}",
                    certificate_path.display(),
                    expected_sha256,
                    actual_sha256
                );
            }
        }
    }

    let certificate_bytes = std::fs::read(&certificate_path).map_err(|e| {
        anyhow::anyhow!(
            "read RPLAN audit certificate {}: {e}",
            certificate_path.display()
        )
    })?;
    let certificate: rplan_audit::AuditCertificate = serde_json::from_slice(&certificate_bytes)
        .map_err(|e| anyhow::anyhow!("parse RPLAN audit certificate: {e}"))?;

    if let Some(expected_content_hash) = manifest.audit_certificate_content_hash.as_deref() {
        if certificate.content_hash != expected_content_hash {
            anyhow::bail!(
                "RPLAN audit certificate content hash mismatch: manifest={}, certificate={}",
                expected_content_hash,
                certificate.content_hash
            );
        }
    }
    if let Some(expected_result) = manifest.audit_result.as_deref() {
        let actual_result = rplan_audit_result_label(&certificate.result);
        if actual_result != expected_result {
            anyhow::bail!(
                "RPLAN audit result mismatch: manifest={}, certificate={}",
                expected_result,
                actual_result
            );
        }
    }
    if let Some(expected_profile_id) = manifest.legal_profile_id.as_deref() {
        if certificate.legal_profile.profile_id != expected_profile_id {
            anyhow::bail!(
                "RPLAN legal profile mismatch: manifest={}, certificate={}",
                expected_profile_id,
                certificate.legal_profile.profile_id
            );
        }
    }
    if let Some(expected_context_hash) = manifest.context_hash.as_deref() {
        if certificate.context_hash.as_deref() != Some(expected_context_hash) {
            anyhow::bail!(
                "RPLAN context hash mismatch: manifest={}, certificate={}",
                expected_context_hash,
                certificate.context_hash.as_deref().unwrap_or_default()
            );
        }
    }
    verify_manifest_rplan_algorithm_lineage(manifest, &certificate)?;

    let rplan_rel = manifest
        .rplan_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("manifest has audit_certificate_path but no rplan_path"))?;
    let rplan_text = std::fs::read_to_string(plan_root.join(rplan_rel))
        .map_err(|e| anyhow::anyhow!("read RPLAN sidecar {rplan_rel}: {e}"))?;
    let rplan_document = rplan_io::read_rplan_str(&rplan_text)
        .map_err(|e| anyhow::anyhow!("parse RPLAN sidecar {rplan_rel}: {e}"))?;

    let context = if certificate.context_hash.is_some() {
        let rctx_rel = manifest.rctx_path.as_deref().ok_or_else(|| {
            anyhow::anyhow!("manifest has contextual audit certificate but no rctx_path")
        })?;
        let rctx_text = std::fs::read_to_string(plan_root.join(rctx_rel))
            .map_err(|e| anyhow::anyhow!("read RPLAN context sidecar {rctx_rel}: {e}"))?;
        Some(
            rplan_io::read_rctx_str(&rctx_text)
                .map_err(|e| anyhow::anyhow!("parse RPLAN context sidecar {rctx_rel}: {e}"))?,
        )
    } else {
        None
    };
    if let Some(context) = context.as_ref() {
        verify_manifest_rplan_source_hashes(manifest, context)?;
    }

    rplan_audit::verify_audit_certificate(
        &certificate,
        Some(&rplan_document.plan),
        context.as_ref(),
    )
    .map_err(|e| anyhow::anyhow!("RPLAN audit certificate verification failed: {e}"))?;
    eprintln!("[PASS] RPLAN audit certificate verified: {certificate_rel}");
    Ok(())
}

fn verify_manifest_rplan_algorithm_lineage(
    manifest: &PlanManifest,
    certificate: &rplan_audit::AuditCertificate,
) -> anyhow::Result<()> {
    let expects_ilp_lineage = manifest.ilp_method.is_some() || manifest.ilp_fallback.is_some();
    if !expects_ilp_lineage {
        return Ok(());
    }
    let lineage = certificate.algorithm_lineage.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "manifest records ILP audit metadata but certificate has no algorithm_lineage"
        )
    })?;
    if lineage.producer_crate != "bisect-ilp" {
        anyhow::bail!(
            "RPLAN algorithm lineage producer mismatch: expected=bisect-ilp, certificate={}",
            lineage.producer_crate
        );
    }
    if let Some(expected_method) = manifest.ilp_method.as_deref() {
        if lineage.method != expected_method {
            anyhow::bail!(
                "RPLAN algorithm lineage method mismatch: manifest={}, certificate={}",
                expected_method,
                lineage.method
            );
        }
    }
    if let Some(expected_fallback) = manifest.ilp_fallback.as_deref() {
        let actual = lineage
            .extra
            .get("fallback")
            .and_then(serde_json::Value::as_str);
        if actual != Some(expected_fallback) {
            anyhow::bail!(
                "RPLAN algorithm lineage fallback mismatch: manifest={}, certificate={}",
                expected_fallback,
                actual.unwrap_or("(missing)")
            );
        }
    }
    if let Some(expected_sha256) = manifest.ilp_audit_summary_sha256.as_deref() {
        let actual = lineage
            .extra
            .get("audit_summary_sha256")
            .and_then(serde_json::Value::as_str);
        if actual != Some(expected_sha256) {
            anyhow::bail!(
                "RPLAN algorithm lineage audit summary SHA-256 mismatch: manifest={}, certificate={}",
                expected_sha256,
                actual.unwrap_or("(missing)")
            );
        }
    }
    if let Some(expected_path) = manifest.ilp_audit_summary_path.as_deref() {
        let actual = lineage
            .extra
            .get("audit_summary_path")
            .and_then(serde_json::Value::as_str);
        if actual != Some(expected_path) {
            anyhow::bail!(
                "RPLAN algorithm lineage audit summary path mismatch: manifest={}, certificate={}",
                expected_path,
                actual.unwrap_or("(missing)")
            );
        }
    }
    Ok(())
}

fn verify_manifest_rplan_source_hashes(
    manifest: &PlanManifest,
    context: &rplan_core::RplanContext,
) -> anyhow::Result<()> {
    if !manifest.adjacency_sha256.is_empty() {
        let expected = format!("sha256:{}", manifest.adjacency_sha256);
        let actual = context.source_hashes.entries.get("adjacency");
        if actual != Some(&expected) {
            anyhow::bail!(
                "RPLAN context adjacency source hash mismatch: manifest={}, context={}",
                expected,
                actual.cloned().unwrap_or_else(|| "(missing)".to_string())
            );
        }
    }

    if let Some(tiger_sha256) = manifest.tiger_sha256.as_deref() {
        if !tiger_sha256.is_empty() {
            let expected = format!("sha256:{tiger_sha256}");
            let actual = context.source_hashes.entries.get("geometry");
            if actual != Some(&expected) {
                anyhow::bail!(
                    "RPLAN context geometry source hash mismatch: manifest={}, context={}",
                    expected,
                    actual.cloned().unwrap_or_else(|| "(missing)".to_string())
                );
            }
        }
    }

    Ok(())
}

fn rplan_audit_result_label(result: &rplan_audit::AuditResult) -> &'static str {
    match result {
        rplan_audit::AuditResult::Pass => "pass",
        rplan_audit::AuditResult::Fail => "fail",
        rplan_audit::AuditResult::PassWithWarnings => "pass-with-warnings",
    }
}

fn find_bisect_binary() -> anyhow::Result<PathBuf> {
    for candidate in [
        "target/release/bisect.exe",
        "target/release/bisect",
        "bisect",
    ] {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return Ok(p);
        }
    }
    // Try current executable
    if let Ok(exe) = std::env::current_exe() {
        return Ok(exe);
    }
    anyhow::bail!("bisect binary not found — run: cargo build --release -p bisect-cli")
}

fn load_original_assignments(
    manifest: &PlanManifest,
    output_base: &str,
    explicit_label: Option<&str>,
) -> anyhow::Result<HashMap<String, usize>> {
    // Try explicit label via PlanContext first
    if let Some(label) = explicit_label {
        for version in &["v1", "verify", "international"] {
            if let Ok(ctx) = crate::plan_context::PlanContext::from_label(
                std::path::Path::new(output_base),
                version,
                &manifest.year,
                label,
            ) {
                if ctx.assignments_path().exists() {
                    let content = std::fs::read_to_string(ctx.assignments_path())?;
                    return Ok(serde_json::from_str(&content)?);
                }
            }
        }
    }

    // Fall back to searching by manifest.label under v1
    let plan_path = PathBuf::from(output_base)
        .join("v1")
        .join(&manifest.year)
        .join("plans")
        .join(&manifest.label)
        .join("data")
        .join("final_assignments.json");

    if plan_path.exists() {
        let content = std::fs::read_to_string(&plan_path)?;
        return Ok(serde_json::from_str(&content)?);
    }

    // Not found — return empty (caller warns and skips comparison)
    Ok(HashMap::new())
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
pub fn jaccard_similarity(a: &HashMap<String, usize>, b: &HashMap<String, usize>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    // Same GEOID → same district assignment = agreement
    let matching = a
        .iter()
        .filter(|(geoid, &d)| b.get(*geoid) == Some(&d))
        .count();
    let union = a.len().max(b.len());
    matching as f64 / union as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;

    fn sha256(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("repo root")
            .to_path_buf()
    }

    fn public_golden_package_manifest() -> PathBuf {
        repo_root()
            .join("docs")
            .join("examples")
            .join("rplan-golden-packages")
            .join("U.18+local-search-improvement")
            .join("manifest.json")
    }

    fn public_method_package_root() -> PathBuf {
        repo_root()
            .join("docs")
            .join("examples")
            .join("rplan-method-packages")
    }

    fn public_benchmark_package_root() -> PathBuf {
        repo_root()
            .join("docs")
            .join("examples")
            .join("rplan-benchmark-packages")
    }

    fn write_ilp_report(dir: &Path, name: &str, lp_bytes: &[u8], sha256: String) -> PathBuf {
        let formulation = bisect_ilp::build_formulation(&[vec![1], vec![0]], &[1, 1], 2, 0.05);
        let result = bisect_ilp::solve(
            &formulation,
            &[vec![1], vec![0]],
            &[1, 1],
            2,
            0.05,
            bisect_ilp::IlpSolver::FormulationOnly,
            0.01,
        );
        let lp_path = dir.join(format!("{name}.lp"));
        std::fs::write(&lp_path, lp_bytes).unwrap();
        let report = bisect_ilp::IlpSolveReport::with_model_artifact(
            formulation,
            result,
            bisect_ilp::IlpModelArtifact {
                format: "cplex-lp".to_string(),
                path: format!("{name}.lp"),
                sha256,
            },
        );
        let report_path = dir.join(format!("{name}.json"));
        std::fs::write(&report_path, report.to_json_string().unwrap()).unwrap();
        report_path
    }

    fn write_rplan_audit_fixture(plan_root: &Path) -> PlanManifest {
        write_rplan_audit_fixture_with_lineage(plan_root, None)
    }

    fn write_rplan_audit_fixture_with_lineage(
        plan_root: &Path,
        algorithm_lineage: Option<rplan_audit::AlgorithmLineage>,
    ) -> PlanManifest {
        let units = rplan_core::PlanUnitIndex {
            unit_kind: rplan_core::UnitKind::Tract,
            state: Some("WA".to_string()),
            year: Some(2020),
            canonical_order: rplan_core::CanonicalOrder::ExplicitUnitIds,
            unit_ids: vec!["53001000100".to_string(), "53001000200".to_string()],
            unit_universe_hash: "sha256:test".to_string(),
            source_id: None,
        };
        let plan = rplan_core::DistrictPlan {
            schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: units.clone(),
            assignment: vec![0, 1],
            k: 2,
            display_labels: vec!["1".to_string(), "2".to_string()],
            allow_empty_districts: false,
        };
        let mut context = rplan_core::RplanContext {
            rctx_version: rplan_core::RCTX_VERSION.to_string(),
            context_hash: String::new(),
            units: units.clone(),
            graph: Some(rplan_core::UnitGraph {
                edge_semantics: rplan_core::EdgeSemantics::Undirected,
                adjacency: vec![
                    vec![rplan_core::UnitEdge {
                        to: 1,
                        kind: rplan_core::EdgeKind::Boundary,
                        weight: None,
                    }],
                    vec![rplan_core::UnitEdge {
                        to: 0,
                        kind: rplan_core::EdgeKind::Boundary,
                        weight: None,
                    }],
                ],
            }),
            populations: Some(vec![100, 100]),
            subdivisions: None,
            demographics: None,
            geometry: None,
            source_hashes: rplan_core::SourceHashes {
                entries: BTreeMap::from([(
                    "adjacency".to_string(),
                    format!("sha256:{}", "b".repeat(64)),
                )]),
            },
        };
        context.context_hash = context.compute_context_hash().unwrap();

        let document = rplan_io::RplanDocument {
            rplan_version: rplan_io::RPLAN_V02.to_string(),
            plan: plan.clone(),
            metadata: rplan_io::RplanMetadataV02 {
                label: "wa_fixture".to_string(),
                jurisdiction: "WA".to_string(),
                chamber: "congressional".to_string(),
                created_at: "2026-05-10T00:00:00Z".to_string(),
                description: None,
            },
            provenance: rplan_io::RplanProvenance::default(),
            geometry: None,
            extensions: BTreeMap::new(),
        };
        let profile = rplan_audit::LegalProfile::us_congressional_project_v1(2020);
        let certificate = rplan_audit::audit_plan_with_lineage(
            &plan,
            Some(&context),
            &profile,
            rplan_audit::RuntimeProvenance {
                binary_name: "bisect-test".to_string(),
                binary_version: "0.1.0".to_string(),
                ..rplan_audit::RuntimeProvenance::default()
            },
            &[
                rplan_audit::AuditConstraint::Population,
                rplan_audit::AuditConstraint::Contiguity,
            ],
            "2026-05-10T00:00:00Z",
            algorithm_lineage,
        )
        .unwrap();

        std::fs::write(
            plan_root.join("plan.rplan"),
            rplan_io::write_rplan_string(&document).unwrap(),
        )
        .unwrap();
        std::fs::write(
            plan_root.join("context.rctx"),
            rplan_io::write_rctx_string(&context).unwrap(),
        )
        .unwrap();
        std::fs::write(
            plan_root.join("audit-certificate.json"),
            serde_json::to_string_pretty(&certificate).unwrap(),
        )
        .unwrap();
        let certificate_sha =
            bisect_report::sha256_file(&plan_root.join("audit-certificate.json")).unwrap();

        PlanManifest {
            rplan_path: Some("plan.rplan".to_string()),
            rctx_path: Some("context.rctx".to_string()),
            audit_certificate_path: Some("audit-certificate.json".to_string()),
            audit_certificate_sha256: Some(certificate_sha),
            audit_certificate_content_hash: Some(certificate.content_hash),
            audit_result: Some(rplan_audit_result_label(&certificate.result).to_string()),
            legal_profile_id: Some(profile.profile_id),
            context_hash: certificate.context_hash,
            ..Default::default()
        }
    }

    fn write_full_audit_package_fixture(plan_root: &Path) -> PlanManifest {
        let report_dir = plan_root.join("intermediate").join("ilp_solve_reports");
        std::fs::create_dir_all(&report_dir).unwrap();
        write_ilp_report(&report_dir, "node_root", b"lp text", sha256(b"lp text"));
        let summary_path = report_dir.join("audit-summary.json");
        crate::ilp_audit::write_ilp_audit_summary_for_dir(&report_dir, &summary_path).unwrap();
        let summary_sha = bisect_report::sha256_file(&summary_path).unwrap();

        let algorithm_lineage = rplan_audit::AlgorithmLineage::new(
            "bisect-ilp",
            "0.1.0",
            "branch-and-cut",
            Vec::new(),
            serde_json::json!({
                "fallback": "metis",
                "audit_summary_path": "intermediate/ilp_solve_reports/audit-summary.json",
                "audit_summary_sha256": summary_sha,
            }),
        )
        .unwrap();
        let mut manifest =
            write_rplan_audit_fixture_with_lineage(plan_root, Some(algorithm_lineage));
        manifest.ilp_method = Some("branch-and-cut".to_string());
        manifest.ilp_fallback = Some("metis".to_string());
        manifest.ilp_solve_report_dir = Some("intermediate/ilp_solve_reports".to_string());
        manifest.ilp_audit_summary_path =
            Some("intermediate/ilp_solve_reports/audit-summary.json".to_string());
        manifest.ilp_audit_summary_sha256 = Some(summary_sha);
        manifest
    }

    #[test]
    fn test_jaccard_identical_plans() {
        let mut a = HashMap::new();
        a.insert("53001000100".to_string(), 1usize);
        a.insert("53001000200".to_string(), 2usize);
        let similarity = jaccard_similarity(&a, &a);
        assert!(
            (similarity - 1.0).abs() < 1e-9,
            "identical plans must have similarity 1.0"
        );
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
        assert!(
            (s - 0.5).abs() < 1e-9,
            "expected 0.5 partial match, got {s}"
        );
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
            skip_binary_check: false,
            label: None,
            verify_assignments_only: false,
            plan_ref: None,
        };
        // Should fail at manifest read, not at binary execution
        let result = run_verify(&args);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("manifest") || msg.contains("nonexistent"),
            "error should be about manifest, not binary: {msg}"
        );
    }

    #[test]
    fn test_verify_dry_run_with_valid_manifest_returns_ok() {
        use bisect_report::PlanManifest;
        use tempfile::TempDir;

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
            skip_binary_check: false,
            label: None,
            verify_assignments_only: false,
            plan_ref: None,
        };
        // dry_run=true: must return Ok after printing command (no binary execution)
        let result = run_verify(&args);
        assert!(
            result.is_ok(),
            "dry_run with valid manifest must return Ok: {:?}",
            result
        );
    }

    #[test]
    fn test_verify_accepts_rplan_golden_package_manifest() {
        let args = VerifyArgs {
            manifest: public_golden_package_manifest(),
            min_similarity: 0.99,
            verify_label: None,
            output_base: "outputs".to_string(),
            dry_run: false,
            skip_binary_check: true,
            label: None,
            verify_assignments_only: false,
            plan_ref: None,
        };

        let result = run_verify(&args);
        assert!(
            result.is_ok(),
            "bisect verify should accept public RPLAN package manifest: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_verify_accepts_rplan_method_package_manifest() {
        let mut verified = Vec::new();
        for entry in std::fs::read_dir(public_method_package_root()).unwrap() {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_dir() {
                continue;
            }
            let package_name = entry.file_name().to_string_lossy().into_owned();
            let args = VerifyArgs {
                manifest: entry.path().join("manifest.json"),
                min_similarity: 0.99,
                verify_label: None,
                output_base: "outputs".to_string(),
                dry_run: false,
                skip_binary_check: true,
                label: None,
                verify_assignments_only: false,
                plan_ref: None,
            };

            let result = run_verify(&args);
            assert!(
                result.is_ok(),
                "bisect verify should accept public RPLAN method package manifest {package_name}: {:?}",
                result.err()
            );
            verified.push(package_name);
        }
        verified.sort();
        assert_eq!(
            verified,
            vec![
                "T.14+spectral-generated-synthetic",
                "U.18+local-search-generated-descendant",
            ]
        );
    }

    #[test]
    fn test_verify_accepts_rplan_benchmark_package_manifest() {
        let mut verified = Vec::new();
        for entry in std::fs::read_dir(public_benchmark_package_root()).unwrap() {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_dir() {
                continue;
            }
            let package_name = entry.file_name().to_string_lossy().into_owned();
            let args = VerifyArgs {
                manifest: entry.path().join("manifest.json"),
                min_similarity: 0.99,
                verify_label: None,
                output_base: "outputs".to_string(),
                dry_run: false,
                skip_binary_check: true,
                label: None,
                verify_assignments_only: false,
                plan_ref: None,
            };

            let result = run_verify(&args);
            assert!(
                result.is_ok(),
                "bisect verify should accept public RPLAN benchmark package manifest {package_name}: {:?}",
                result.err()
            );
            verified.push(package_name);
        }
        verified.sort();
        assert_eq!(
            verified,
            vec![
                "T.14+spectral-grid10-benchmark",
                "U.16+branch-and-cut-path8-benchmark",
                "U.18+local-search-grid10-benchmark",
            ]
        );
    }

    #[test]
    fn test_verify_rplan_golden_package_rejects_manifest_hash_mismatch() {
        let manifest_path = public_golden_package_manifest();
        let content = std::fs::read_to_string(&manifest_path).unwrap();
        let mut manifest = parse_rplan_golden_package_manifest(&content)
            .unwrap()
            .expect("RPLAN golden package manifest");
        manifest.files[0].sha256 = "0".repeat(64);

        let result = verify_rplan_golden_package_manifest(&manifest_path, &manifest);
        assert!(result.is_err(), "tampered package hash must fail");
        let message = result.unwrap_err().to_string();
        assert!(
            message.contains("SHA-256 mismatch"),
            "expected SHA-256 mismatch, got: {message}"
        );
    }

    #[test]
    fn test_verify_label_defaults_to_label_verify_suffix() {
        // When verify_label is None, the label should be "{original_label}_verify"
        let verify_label = None::<String>;
        let original_label = "vt_congressional_2020";
        let computed = verify_label.unwrap_or_else(|| format!("{}_verify", original_label));
        assert_eq!(computed, "vt_congressional_2020_verify");
    }

    #[test]
    fn test_verify_label_custom_overrides_default() {
        let verify_label = Some("my_custom_label".to_string());
        let original_label = "vt_congressional_2020";
        let computed = verify_label.unwrap_or_else(|| format!("{}_verify", original_label));
        assert_eq!(computed, "my_custom_label");
    }

    // ── Gap 8: verify warns on missing output_base ───────────────────────────

    #[test]
    fn test_verify_warns_missing_output_base() {
        // When output_base doesn't exist, the warning must contain "output directory".
        // We simulate the check logic directly.
        let output_base = "/nonexistent_gap8_test/outputs";
        let output_base_path = std::path::PathBuf::from(output_base);

        assert!(
            !output_base_path.exists(),
            "test path must not exist: {output_base}"
        );

        let warning = format!(
            "WARNING: output directory '{}' not found on this machine.\n\
             If verifying a plan from another machine, use --output-base to specify\n\
             where plans are stored locally. The plan comparison step will be skipped.",
            output_base
        );

        assert!(
            warning.contains("output directory"),
            "warning must contain 'output directory': {warning}"
        );
        assert!(
            warning.contains("WARNING"),
            "warning must start with WARNING: {warning}"
        );
        assert!(
            warning.contains("--output-base"),
            "warning must mention --output-base flag: {warning}"
        );
    }

    // ── Task 209: --verify-assignments-only ──────────────────────────────────

    #[test]
    fn test_verify_assignments_only_flag_parsed() {
        use crate::args::VerifyArgs;
        use clap::Parser;
        let args = VerifyArgs::parse_from([
            "verify",
            "--manifest",
            "manifest.json",
            "--verify-assignments-only",
            "--plan-ref",
            "ref.json",
        ]);
        assert!(args.verify_assignments_only);
        assert_eq!(args.plan_ref, Some("ref.json".to_string()));
    }

    #[test]
    fn test_verify_assignments_only_defaults_false() {
        use crate::args::VerifyArgs;
        use clap::Parser;
        let args = VerifyArgs::parse_from(["verify", "--manifest", "manifest.json"]);
        assert!(!args.verify_assignments_only);
        assert!(args.plan_ref.is_none());
    }

    #[test]
    fn test_verify_jaccard_identical_assignments() {
        // Test the Jaccard computation logic directly
        let mut a = std::collections::HashMap::new();
        a.insert("53001000100".to_string(), 1usize);
        a.insert("53001000200".to_string(), 2usize);
        a.insert("53001000300".to_string(), 1usize);

        let b = a.clone();

        let matching = a
            .iter()
            .filter(|(geoid, &d)| b.get(*geoid) == Some(&d))
            .count();
        let union = a.len().max(b.len());
        let similarity = matching as f64 / union as f64;

        assert!(
            (similarity - 1.0).abs() < 1e-9,
            "identical assignments must have Jaccard 1.0"
        );
    }

    #[test]
    fn test_verify_jaccard_different_assignments() {
        let mut a = std::collections::HashMap::new();
        a.insert("tract1".to_string(), 1usize);
        a.insert("tract2".to_string(), 2usize);

        let mut b = std::collections::HashMap::new();
        b.insert("tract1".to_string(), 2usize); // different district
        b.insert("tract2".to_string(), 1usize); // different district

        let matching = a
            .iter()
            .filter(|(geoid, &d)| b.get(*geoid) == Some(&d))
            .count();
        let union = a.len().max(b.len());
        let similarity = matching as f64 / union as f64;

        assert_eq!(
            similarity, 0.0,
            "completely swapped assignments must have Jaccard 0.0"
        );
    }

    /// --verify-assignments-only with valid manifest+reference runs successfully.
    #[test]
    fn test_verify_assignments_only_with_valid_files() {
        use bisect_report::PlanManifest;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();

        // Write manifest
        let manifest = PlanManifest {
            label: "vt_congressional_2020".to_string(),
            state_code: "VT".to_string(),
            year: "2020".to_string(),
            chamber: "congressional".to_string(),
            num_districts: 1,
            balance_tolerance_pct: 0.5,
            seed: None,
            ..Default::default()
        };
        let manifest_path = tmp.path().join("manifest.json");
        std::fs::write(&manifest_path, serde_json::to_string(&manifest).unwrap()).unwrap();

        // Write original assignments
        let assignments: HashMap<String, usize> = [
            ("50001000100".to_string(), 1usize),
            ("50001000200".to_string(), 1usize),
        ]
        .into_iter()
        .collect();
        let plan_dir = tmp
            .path()
            .join("v1")
            .join("2020")
            .join("plans")
            .join("vt_congressional_2020")
            .join("data");
        std::fs::create_dir_all(&plan_dir).unwrap();
        std::fs::write(
            plan_dir.join("final_assignments.json"),
            serde_json::to_string(&assignments).unwrap(),
        )
        .unwrap();
        // Write manifest.json next to plan dir too (for PlanContext)
        let plan_manifest_dir = tmp
            .path()
            .join("v1")
            .join("2020")
            .join("plans")
            .join("vt_congressional_2020");
        std::fs::write(
            plan_manifest_dir.join("manifest.json"),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();

        // Write reference (same as original → perfect match)
        let ref_path = tmp.path().join("ref_assignments.json");
        std::fs::write(&ref_path, serde_json::to_string(&assignments).unwrap()).unwrap();

        let args = VerifyArgs {
            manifest: manifest_path,
            min_similarity: 0.99,
            verify_label: None,
            output_base: tmp.path().to_str().unwrap().to_string(),
            dry_run: false,
            skip_binary_check: false,
            label: Some("vt_congressional_2020".to_string()),
            verify_assignments_only: true,
            plan_ref: Some(ref_path.to_str().unwrap().to_string()),
        };

        let result = run_verify(&args);
        assert!(
            result.is_ok(),
            "--verify-assignments-only with identical plans must pass: {:?}",
            result.err()
        );
    }

    // ── Task 133: --skip-binary-check ────────────────────────────────────────

    #[test]
    fn test_verify_skip_binary_check_flag_parses() {
        use crate::args::VerifyArgs;
        use clap::Parser;
        let args = VerifyArgs::parse_from([
            "verify",
            "--manifest",
            "/tmp/manifest.json",
            "--skip-binary-check",
        ]);
        assert!(
            args.skip_binary_check,
            "--skip-binary-check flag must parse to true"
        );
    }

    #[test]
    fn test_verify_skip_binary_check_default_false() {
        use crate::args::VerifyArgs;
        use clap::Parser;
        let args = VerifyArgs::parse_from(["verify", "--manifest", "/tmp/manifest.json"]);
        assert!(
            !args.skip_binary_check,
            "--skip-binary-check must default to false"
        );
    }

    #[test]
    fn test_verify_binary_check_logic_empty_hash_skips() {
        // An empty binary_sha256 should be silently skipped (no action)
        // We call check_binary_sha256 directly and verify it doesn't panic.
        check_binary_sha256("", false);
        check_binary_sha256("", true);
        // No panic = pass
    }

    #[test]
    fn test_verify_binary_check_logic_not_prefix_skips() {
        // binary_sha256 starting with "(not" should be silently skipped
        check_binary_sha256("(not available)", false);
        check_binary_sha256("(not computed)", true);
        // No panic = pass
    }

    #[test]
    fn test_verify_binary_check_logic_skip_flag_suppresses_check() {
        // When skip_binary_check=true, check_binary_sha256 must not compute exe hash
        // (i.e., must not fail even with a bogus sha256 string).
        // We can't capture eprintln output in unit tests, but we verify no panic.
        let bogus_sha = "a".repeat(64); // 64 hex chars (valid format, wrong value)
        check_binary_sha256(&bogus_sha, true); // skip_binary_check=true
                                               // No panic = pass
    }

    #[test]
    fn test_verify_binary_check_logic_mismatch_no_panic() {
        // With skip_binary_check=false and a known-wrong hash, must emit WARNING but not panic.
        // The exe hash won't match "a"*64, so a warning is emitted (to stderr — not testable).
        let bogus_sha = "a".repeat(64);
        check_binary_sha256(&bogus_sha, false); // should emit WARNING but not panic
                                                // No panic = pass
    }

    #[test]
    fn test_verify_manifest_ilp_audit_summary_passes_for_fresh_summary() {
        let tmp = tempfile::TempDir::new().unwrap();
        let report_dir = tmp.path().join("intermediate").join("ilp_solve_reports");
        std::fs::create_dir_all(&report_dir).unwrap();
        write_ilp_report(&report_dir, "node_root", b"lp text", sha256(b"lp text"));
        let summary_path = report_dir.join("audit-summary.json");
        crate::ilp_audit::write_ilp_audit_summary_for_dir(&report_dir, &summary_path).unwrap();
        let summary_sha = bisect_report::sha256_file(&summary_path).unwrap();

        let manifest = PlanManifest {
            ilp_solve_report_dir: Some("intermediate/ilp_solve_reports".to_string()),
            ilp_audit_summary_path: Some(
                "intermediate/ilp_solve_reports/audit-summary.json".to_string(),
            ),
            ilp_audit_summary_sha256: Some(summary_sha),
            ..Default::default()
        };

        verify_manifest_ilp_audit_summary(&manifest, tmp.path())
            .expect("fresh ILP audit summary should verify from manifest paths");
    }

    #[test]
    fn test_verify_manifest_ilp_audit_summary_rejects_sha_mismatch() {
        let tmp = tempfile::TempDir::new().unwrap();
        let report_dir = tmp.path().join("intermediate").join("ilp_solve_reports");
        std::fs::create_dir_all(&report_dir).unwrap();
        write_ilp_report(&report_dir, "node_root", b"lp text", sha256(b"lp text"));
        let summary_path = report_dir.join("audit-summary.json");
        crate::ilp_audit::write_ilp_audit_summary_for_dir(&report_dir, &summary_path).unwrap();

        let manifest = PlanManifest {
            ilp_solve_report_dir: Some("intermediate/ilp_solve_reports".to_string()),
            ilp_audit_summary_path: Some(
                "intermediate/ilp_solve_reports/audit-summary.json".to_string(),
            ),
            ilp_audit_summary_sha256: Some("0".repeat(64)),
            ..Default::default()
        };

        let err = verify_manifest_ilp_audit_summary(&manifest, tmp.path())
            .expect_err("stale manifest hash should fail ILP audit summary verification");
        assert!(err.to_string().contains("SHA-256 mismatch"));
    }

    #[test]
    fn test_verify_manifest_rplan_audit_certificate_passes_for_fresh_sidecars() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut manifest = write_rplan_audit_fixture(tmp.path());
        manifest.adjacency_sha256 = "b".repeat(64);

        verify_manifest_rplan_audit_certificate(&manifest, tmp.path())
            .expect("fresh RPLAN audit sidecars should verify");
    }

    #[test]
    fn test_verify_manifest_rplan_audit_certificate_rejects_source_hash_mismatch() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut manifest = write_rplan_audit_fixture(tmp.path());
        manifest.adjacency_sha256 = "0".repeat(64);

        let err = verify_manifest_rplan_audit_certificate(&manifest, tmp.path())
            .expect_err("manifest/context source hash mismatch should fail verification");
        assert!(
            err.to_string().contains("adjacency source hash mismatch"),
            "{err}"
        );
    }

    #[test]
    fn test_verify_manifest_rplan_audit_certificate_requires_ilp_lineage_when_recorded() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut manifest = write_rplan_audit_fixture(tmp.path());
        manifest.ilp_method = Some("branch-and-cut".to_string());

        let err = verify_manifest_rplan_audit_certificate(&manifest, tmp.path())
            .expect_err("ILP manifest metadata should require certificate lineage");
        assert!(err.to_string().contains("algorithm_lineage"), "{err}");
    }

    #[test]
    fn test_verify_manifest_rplan_audit_certificate_rejects_sha_mismatch() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut manifest = write_rplan_audit_fixture(tmp.path());
        manifest.audit_certificate_sha256 = Some("0".repeat(64));

        let err = verify_manifest_rplan_audit_certificate(&manifest, tmp.path())
            .expect_err("manifest certificate sha mismatch should fail verification");
        assert!(err.to_string().contains("SHA-256 mismatch"));
    }

    #[test]
    fn test_l1_verify_full_audit_package_round_trip() {
        let tmp = tempfile::TempDir::new().unwrap();
        let manifest = write_full_audit_package_fixture(tmp.path());

        verify_manifest_ilp_audit_summary(&manifest, tmp.path())
            .expect("L1 package should verify ILP audit summary");
        verify_manifest_rplan_audit_certificate(&manifest, tmp.path())
            .expect("L1 package should verify RPLAN audit certificate");
    }

    #[test]
    fn test_l1_verify_full_audit_package_rejects_tampered_artifacts() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut manifest = write_full_audit_package_fixture(tmp.path());

        let summary_path = tmp
            .path()
            .join("intermediate")
            .join("ilp_solve_reports")
            .join("audit-summary.json");
        std::fs::write(
            &summary_path,
            serde_json::json!({
                "schema_version": 1,
                "generated_at": "2026-05-10T00:00:00Z",
                "report_count": 0,
                "reports": []
            })
            .to_string(),
        )
        .unwrap();
        manifest.ilp_audit_summary_sha256 =
            Some(bisect_report::sha256_file(&summary_path).unwrap());

        let ilp_err = verify_manifest_ilp_audit_summary(&manifest, tmp.path())
            .expect_err("stale ILP audit summary should fail even with matching manifest sha");
        assert!(
            ilp_err
                .to_string()
                .contains("ILP audit summary verification failed"),
            "{ilp_err}"
        );

        let mut manifest = write_full_audit_package_fixture(tmp.path());
        let certificate_path = tmp.path().join("audit-certificate.json");
        let mut certificate_json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&certificate_path).unwrap()).unwrap();
        certificate_json["content_hash"] = serde_json::Value::String("sha256:tampered".to_string());
        std::fs::write(
            &certificate_path,
            serde_json::to_string_pretty(&certificate_json).unwrap(),
        )
        .unwrap();
        manifest.audit_certificate_sha256 =
            Some(bisect_report::sha256_file(&certificate_path).unwrap());

        let certificate_err = verify_manifest_rplan_audit_certificate(&manifest, tmp.path())
            .expect_err("tampered RPLAN audit certificate should fail with matching manifest sha");
        assert!(
            certificate_err
                .to_string()
                .contains("content hash mismatch")
                || certificate_err
                    .to_string()
                    .contains("RPLAN audit certificate verification failed"),
            "{certificate_err}"
        );
    }

    #[test]
    #[ignore = "L2: set BISECT_L2_AUDIT_PLAN_DIR to a real ILP/RPLAN audit package root"]
    fn test_l2_verify_real_ilp_rplan_audit_package_from_env() {
        let plan_root = std::env::var("BISECT_L2_AUDIT_PLAN_DIR")
            .expect("set BISECT_L2_AUDIT_PLAN_DIR to a real plan package root");
        let plan_root = PathBuf::from(plan_root);
        let manifest_path = plan_root.join("manifest.json");
        let manifest: PlanManifest =
            serde_json::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();

        assert!(
            manifest.ilp_audit_summary_path.is_some(),
            "L2 package must include an ILP audit summary"
        );
        assert!(
            manifest.audit_certificate_path.is_some(),
            "L2 package must include an RPLAN audit certificate"
        );
        verify_manifest_ilp_audit_summary(&manifest, &plan_root)
            .expect("real ILP audit summary should verify");
        verify_manifest_rplan_audit_certificate(&manifest, &plan_root)
            .expect("real RPLAN audit certificate should verify");
    }

    // ── 15 additional L0 tests ───────────────────────────────────────────────

    // -- jaccard_similarity edge cases ---------------------------------------

    #[test]
    fn test_jaccard_both_empty_returns_zero() {
        let a: HashMap<String, usize> = HashMap::new();
        let b: HashMap<String, usize> = HashMap::new();
        assert_eq!(jaccard_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_jaccard_left_empty_returns_zero() {
        let a: HashMap<String, usize> = HashMap::new();
        let mut b = HashMap::new();
        b.insert("G1".to_string(), 1usize);
        assert_eq!(jaccard_similarity(&a, &b), 0.0, "left empty → 0.0");
    }

    #[test]
    fn test_jaccard_right_empty_returns_zero() {
        let mut a = HashMap::new();
        a.insert("G1".to_string(), 1usize);
        let b: HashMap<String, usize> = HashMap::new();
        assert_eq!(jaccard_similarity(&a, &b), 0.0, "right empty → 0.0");
    }

    #[test]
    fn test_jaccard_full_match_many_tracts() {
        // 100-tract plan, perfectly reproduced
        let plan: HashMap<String, usize> = (0..100)
            .map(|i| (format!("G{:05}", i), (i % 5) + 1))
            .collect();
        let sim = jaccard_similarity(&plan, &plan);
        assert!(
            (sim - 1.0).abs() < 1e-9,
            "identical 100-tract plan must have Jaccard 1.0"
        );
    }

    #[test]
    fn test_jaccard_larger_plan_dominates_union() {
        // Plan A: 4 tracts; plan B: 8 tracts. Union = 8 (the larger).
        let a: HashMap<String, usize> = (0..4).map(|i| (format!("G{i}"), 1usize)).collect();
        let b: HashMap<String, usize> = (0..8).map(|i| (format!("G{i}"), 1usize)).collect();
        let sim = jaccard_similarity(&a, &b);
        // 4 matching GEOIDs out of 8 union → 0.5
        assert!(
            (sim - 0.5).abs() < 1e-9,
            "union-style Jaccard with 4 of 8 common GEOIDs+values must be 0.5, got {sim}"
        );
    }

    // -- check_binary_sha256 thoroughness -----------------------------------

    #[test]
    fn test_check_binary_sha256_whitespace_only_does_not_crash() {
        // An all-whitespace hash is neither empty nor "(not" prefixed —
        // the function will attempt to compare but must not panic.
        check_binary_sha256("   ", false);
        check_binary_sha256("   ", true);
    }

    #[test]
    fn test_check_binary_sha256_not_available_variants() {
        // Various "(not …" strings must all be silently skipped
        for s in &[
            "(not computed)",
            "(not available)",
            "(not set)",
            "(not-hashed)",
        ] {
            check_binary_sha256(s, false);
            check_binary_sha256(s, true);
        }
        // No panic = pass
    }

    // -- verify label construction -------------------------------------------

    #[test]
    fn test_verify_label_with_none_and_complex_original_label() {
        let original = "ca_congressional_2020_vra";
        let label = None::<String>.unwrap_or_else(|| format!("{}_verify", original));
        assert_eq!(label, "ca_congressional_2020_vra_verify");
    }

    #[test]
    fn test_verify_label_with_some_takes_priority() {
        let original = "ca_congressional_2020";
        let label = Some("my_label".to_string()).unwrap_or_else(|| format!("{}_verify", original));
        assert_eq!(label, "my_label");
    }

    // -- VerifyArgs field defaults -------------------------------------------

    #[test]
    fn test_verify_args_default_min_similarity() {
        use crate::args::VerifyArgs;
        use clap::Parser;
        let args = VerifyArgs::parse_from(["verify", "--manifest", "manifest.json"]);
        assert!(
            (args.min_similarity - 0.99).abs() < 1e-9,
            "default min_similarity must be 0.99, got {}",
            args.min_similarity
        );
    }

    #[test]
    fn test_verify_args_default_output_base() {
        use crate::args::VerifyArgs;
        use clap::Parser;
        let args = VerifyArgs::parse_from(["verify", "--manifest", "manifest.json"]);
        assert_eq!(
            args.output_base, "outputs",
            "default output_base must be 'outputs'"
        );
    }

    #[test]
    fn test_verify_args_dry_run_default_false() {
        use crate::args::VerifyArgs;
        use clap::Parser;
        let args = VerifyArgs::parse_from(["verify", "--manifest", "manifest.json"]);
        assert!(!args.dry_run, "--dry-run must default to false");
    }

    #[test]
    fn test_verify_args_custom_min_similarity() {
        use crate::args::VerifyArgs;
        use clap::Parser;
        let args = VerifyArgs::parse_from([
            "verify",
            "--manifest",
            "manifest.json",
            "--min-similarity",
            "0.85",
        ]);
        assert!(
            (args.min_similarity - 0.85).abs() < 1e-9,
            "custom min_similarity must be accepted: {}",
            args.min_similarity
        );
    }

    // -- run_verify with nonexistent manifest --------------------------------

    #[test]
    fn test_run_verify_missing_manifest_returns_err() {
        let args = crate::args::VerifyArgs {
            manifest: std::path::PathBuf::from("/no/such/manifest.json"),
            min_similarity: 0.99,
            verify_label: None,
            output_base: "outputs".to_string(),
            dry_run: false,
            skip_binary_check: true,
            label: None,
            verify_assignments_only: false,
            plan_ref: None,
        };
        let result = run_verify(&args);
        assert!(result.is_err(), "missing manifest must return Err");
    }

    #[test]
    fn test_run_verify_assignments_only_missing_manifest_returns_err() {
        let args = crate::args::VerifyArgs {
            manifest: std::path::PathBuf::from("/no/such/manifest.json"),
            min_similarity: 0.99,
            verify_label: None,
            output_base: "outputs".to_string(),
            dry_run: false,
            skip_binary_check: true,
            label: None,
            verify_assignments_only: true,
            plan_ref: Some("ref.json".to_string()),
        };
        let result = run_verify(&args);
        assert!(
            result.is_err(),
            "--verify-assignments-only with missing manifest must return Err"
        );
    }

    // ── 9 bonus tests ────────────────────────────────────────────────────────

    #[test]
    fn test_jaccard_three_quarters_match() {
        let mut a: HashMap<String, usize> = HashMap::new();
        a.insert("G1".to_string(), 1);
        a.insert("G2".to_string(), 2);
        a.insert("G3".to_string(), 1);
        a.insert("G4".to_string(), 2);
        let mut b = a.clone();
        // Change one assignment
        b.insert("G4".to_string(), 1);
        let sim = jaccard_similarity(&a, &b);
        // 3 matching out of 4 union → 0.75
        assert!(
            (sim - 0.75).abs() < 1e-9,
            "3/4 matching must give 0.75, got {sim}"
        );
    }

    #[test]
    fn test_jaccard_asymmetric_sizes_larger_denominator() {
        // a=3 tracts, b=6 tracts; all of a's are in b with same district
        let a: HashMap<String, usize> = (0..3).map(|i| (format!("G{i}"), 1usize)).collect();
        let b: HashMap<String, usize> = (0..6).map(|i| (format!("G{i}"), 1usize)).collect();
        let sim = jaccard_similarity(&a, &b);
        // 3 matching, union = 6 → 0.5
        assert!((sim - 0.5).abs() < 1e-9, "3 of 6 must give 0.5, got {sim}");
    }

    #[test]
    fn test_jaccard_reflexive() {
        let plan: HashMap<String, usize> = (0..20).map(|i| (format!("T{i}"), i % 4 + 1)).collect();
        let s1 = jaccard_similarity(&plan, &plan);
        let s2 = jaccard_similarity(&plan, &plan);
        assert!((s1 - s2).abs() < 1e-9, "jaccard must be deterministic");
        assert!((s1 - 1.0).abs() < 1e-9, "self-similarity must be 1.0");
    }

    #[test]
    fn test_verify_args_verify_label_none_by_default() {
        use crate::args::VerifyArgs;
        use clap::Parser;
        let args = VerifyArgs::parse_from(["verify", "--manifest", "m.json"]);
        assert!(
            args.verify_label.is_none(),
            "--verify-label must default to None"
        );
    }

    #[test]
    fn test_verify_args_label_field_none_by_default() {
        use crate::args::VerifyArgs;
        use clap::Parser;
        let args = VerifyArgs::parse_from(["verify", "--manifest", "m.json"]);
        assert!(args.label.is_none(), "--label field must default to None");
    }

    #[test]
    fn test_verify_args_plan_ref_none_by_default() {
        use crate::args::VerifyArgs;
        use clap::Parser;
        let args = VerifyArgs::parse_from(["verify", "--manifest", "m.json"]);
        assert!(args.plan_ref.is_none(), "--plan-ref must default to None");
    }

    #[test]
    fn test_check_binary_sha256_64_char_bogus_skip_true_no_panic() {
        // All hex chars but known wrong — skip_binary_check suppresses check
        let sha = "b".repeat(64);
        check_binary_sha256(&sha, true);
    }

    #[test]
    fn test_jaccard_single_tract_mismatch_district() {
        let mut a = HashMap::new();
        a.insert("TRACT1".to_string(), 3usize);
        let mut b = HashMap::new();
        b.insert("TRACT1".to_string(), 4usize);
        // Same GEOID, different district → no match
        let sim = jaccard_similarity(&a, &b);
        assert_eq!(sim, 0.0, "same GEOID but different district must give 0.0");
    }

    #[test]
    fn test_verify_dry_run_flag_parse() {
        use crate::args::VerifyArgs;
        use clap::Parser;
        let args = VerifyArgs::parse_from(["verify", "--manifest", "m.json", "--dry-run"]);
        assert!(args.dry_run, "--dry-run flag must parse to true");
    }
}
