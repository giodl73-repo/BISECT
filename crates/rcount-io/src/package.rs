use crate::*;

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
        &dir.join("normalized").join("rhist-refs.ndjson"),
        &package.rhist_refs,
    )?;
    write_ndjson(
        &dir.join("normalized").join("rctx-refs.ndjson"),
        &package.rctx_refs,
    )?;
    write_ndjson(
        &dir.join("proofs").join("inclusion-proofs.ndjson"),
        &package.inclusion_proofs,
    )?;
    write_ndjson(&dir.join("normalized").join("cvr.ndjson"), &package.cvr)?;
    write_ndjson(
        &dir.join("audits").join("algorithm-runs.ndjson"),
        &package.audit_algorithm_runs,
    )?;
    write_ndjson(&dir.join("audits").join("rla.ndjson"), &package.rla_audits)?;
    write_ndjson(
        &dir.join("audits").join("manual.ndjson"),
        &package.manual_audits,
    )?;
    write_ndjson(
        &dir.join("audits").join("batch-comparison.ndjson"),
        &package.batch_comparison_audits,
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
        rhist_refs: read_optional_ndjson(&dir.join("normalized").join("rhist-refs.ndjson"))?,
        rctx_refs: read_optional_ndjson(&dir.join("normalized").join("rctx-refs.ndjson"))?,
        inclusion_proofs: read_optional_ndjson(
            &dir.join("proofs").join("inclusion-proofs.ndjson"),
        )?,
        cvr: read_optional_ndjson(&dir.join("normalized").join("cvr.ndjson"))?,
        audit_algorithm_runs: read_optional_ndjson(
            &dir.join("audits").join("algorithm-runs.ndjson"),
        )?,
        rla_audits: read_optional_ndjson(&dir.join("audits").join("rla.ndjson"))?,
        manual_audits: read_optional_ndjson(&dir.join("audits").join("manual.ndjson"))?,
        batch_comparison_audits: read_optional_ndjson(
            &dir.join("audits").join("batch-comparison.ndjson"),
        )?,
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
