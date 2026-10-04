use crate::*;

pub fn write_package_dir(dir: &Path, manifest: &RcountManifest, package: &RcountPackage) -> Result<(), RcountIoError> {
    let files = write_package_files(manifest, package)?;
    for folder in ["sources", "normalized", "reconciliation", "status", "proofs", "audits", "transcripts"] { fs::create_dir_all(dir.join(folder))?; }
    for (path, bytes) in files { fs::write(dir.join(path), bytes)?; }
    Ok(())
}

/// Exact package bytes, with the same serialization and hashes as disk output.
pub fn write_package_files(manifest: &RcountManifest, package: &RcountPackage) -> Result<PackageFiles, RcountIoError> {
    let mut files = PackageFiles::new();
    let computed = package_content_hash(package)?;
    let mut manifest = manifest.clone();
    manifest.content_hash = computed.clone();

    insert_json_pretty(&mut files, "manifest.json", &manifest)?;
    let source_entry = write_synthetic_source_export_files(&mut files, package)?;
    insert_json_pretty(
        &mut files, "sources/source-index.json",
        &SourceIndex {
            sources: vec![source_entry],
        },
    )?;
    insert_ndjson(
        &mut files, "normalized/contests.ndjson",
        &package.contests,
    )?;
    insert_ndjson(
        &mut files, "normalized/reporting-units.ndjson",
        &package.reporting_units,
    )?;
    insert_ndjson(
        &mut files, "normalized/batches.ndjson",
        &package.batches,
    )?;
    insert_ndjson(
        &mut files, "normalized/lineage.ndjson",
        &package.lineage,
    )?;
    insert_ndjson(
        &mut files, "normalized/rhist-refs.ndjson",
        &package.rhist_refs,
    )?;
    insert_ndjson(
        &mut files, "normalized/rctx-refs.ndjson",
        &package.rctx_refs,
    )?;
    insert_ndjson(
        &mut files, "proofs/inclusion-proofs.ndjson",
        &package.inclusion_proofs,
    )?;
    insert_ndjson(&mut files, "normalized/cvr.ndjson", &package.cvr)?;
    insert_ndjson(
        &mut files, "audits/algorithm-runs.ndjson",
        &package.audit_algorithm_runs,
    )?;
    insert_ndjson(&mut files, "audits/rla.ndjson", &package.rla_audits)?;
    insert_ndjson(
        &mut files, "audits/manual.ndjson",
        &package.manual_audits,
    )?;
    insert_ndjson(
        &mut files, "audits/batch-comparison.ndjson",
        &package.batch_comparison_audits,
    )?;
    insert_ndjson(
        &mut files, "normalized/summaries.ndjson",
        &package.summaries,
    )?;
    insert_lines(
        &mut files, "reconciliation/equations.ndjson",
        &[
            r#"{"equation_id":"contest_selection_sum","status":"declared"}"#,
            r#"{"equation_id":"jurisdiction_contest_total","status":"declared"}"#,
            r#"{"equation_id":"batch_summary_total","status":"declared"}"#,
            r#"{"equation_id":"lineage_conservation","status":"declared"}"#,
            r#"{"equation_id":"rhist_reference_declared","status":"declared"}"#,
            r#"{"equation_id":"rctx_reference_declared","status":"declared"}"#,
            r#"{"equation_id":"status_event_declared","status":"declared"}"#,
            r#"{"equation_id":"canvass_correction_event","status":"declared"}"#,
            r#"{"equation_id":"cvr_summary_total","status":"declared"}"#,
            r#"{"equation_id":"rla_sampler_replay","status":"declared"}"#,
            r#"{"equation_id":"rla_margin_metadata","status":"declared"}"#,
            r#"{"equation_id":"rla_stopping_rule","status":"declared"}"#,
            r#"{"equation_id":"manual_audit_reconciliation","status":"declared"}"#,
            r#"{"equation_id":"batch_comparison_overstatement","status":"declared"}"#,
        ],
    )?;
    insert_ndjson(
        &mut files, "status/events.ndjson",
        &package.status_events,
    )?;
    insert_json_pretty(
        &mut files, "proofs/package-hashes.json",
        &PackageHashes {
            package_content_hash: computed,
            contest_count: package.contests.len(),
            reporting_unit_count: package.reporting_units.len(),
            batch_count: package.batches.len(),
            lineage_count: package.lineage.len(),
            rhist_ref_count: package.rhist_refs.len(),
            rctx_ref_count: package.rctx_refs.len(),
            inclusion_proof_count: package.inclusion_proofs.len(),
            cvr_count: package.cvr.len(),
            audit_algorithm_run_count: package.audit_algorithm_runs.len(),
            rla_audit_count: package.rla_audits.len(),
            manual_audit_count: package.manual_audits.len(),
            batch_comparison_audit_count: package.batch_comparison_audits.len(),
            summary_count: package.summaries.len(),
        },
    )?;
    insert_json_pretty(
        &mut files, "transcripts/verify-transcript.json",
        &serde_json::json!({
            "status": "generated-fixture",
            "verifier": "rcount-io",
            "checks": ["contest_selection_sum", "jurisdiction_contest_total"]
        }),
    )?;
    Ok(files)
}

pub fn read_package_dir(dir: &Path) -> Result<(RcountManifest, RcountPackage), RcountIoError> {
    read_package_with(&mut |path| package_reader::read_disk_optional(dir, path))
}

pub fn read_source_index(dir: &Path) -> Result<SourceIndex, RcountIoError> {
    read_json(&dir.join("sources").join("source-index.json"))
}

pub fn verify_source_index(dir: &Path) -> Result<Vec<SourceCheck>, RcountIoError> {
    verify_sources_with(&mut |path| package_reader::read_disk_optional(dir, path))
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
