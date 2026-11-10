//! Portable reader for the native ten-section report. Supplied evidence is not authenticated.
use crate::{
    report_assembly::{self, Report, OPTIONAL_ANALYSIS_FILES, REQUIRED_ANALYSIS_FILES},
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn source_hashes_from_context(text: Option<&str>) -> anyhow::Result<Value> {
    let Some(text) = text else {
        return Ok(json!({"status":"unavailable","reason":"rctx_path not recorded in manifest"}));
    };
    let context = rplan_io::read_rctx_str(text).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(if context.source_hashes.entries.is_empty() {
        json!({"status":"missing","reason":"context source_hashes is empty"})
    } else {
        json!({"status":"ok","entries":context.source_hashes.entries})
    })
}
pub fn lineage_from_certificate(text: Option<&str>) -> anyhow::Result<Value> {
    let Some(text) = text else {
        return Ok(
            json!({"status":"unavailable","reason":"audit_certificate_path not recorded in manifest"}),
        );
    };
    let certificate: Value = serde_json::from_str(text)?;
    Ok(match certificate.get("algorithm_lineage") {
        Some(Value::Null) | None => {
            json!({"status":"unavailable","reason":"certificate has no algorithm_lineage"})
        }
        Some(lineage) => json!({"status":"ok","lineage":lineage}),
    })
}
fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 240
        && !path.contains('\\')
        && !path.contains(':')
        && !path.starts_with('/')
        && path.split('/').all(|p| {
            !p.is_empty() && !matches!(p, "." | ".." | "__proto__" | "constructor" | "prototype")
        })
}
fn source_text<'a>(files: &'a BTreeMap<String, Vec<u8>>, path: &str) -> anyhow::Result<&'a str> {
    let raw = files
        .get(path)
        .ok_or_else(|| anyhow::anyhow!("Required report source missing: {path}"))?;
    std::str::from_utf8(raw).map_err(|e| anyhow::anyhow!("Invalid UTF-8 report source {path}: {e}"))
}
pub fn assemble_report_package(
    files: &BTreeMap<String, Vec<u8>>,
    generated_at: String,
) -> anyhow::Result<Report> {
    assemble_report_package_with_fallback(files, generated_at, None)
}
pub fn assemble_report_package_with_fallback(
    files: &BTreeMap<String, Vec<u8>>,
    generated_at: String,
    fallback: Option<&crate::commission_manifest::ReportFallback>,
) -> anyhow::Result<Report> {
    if files.len() > 16
        || files
            .iter()
            .any(|(path, raw)| !safe_path(path) || raw.len() > 16 * 1024 * 1024)
        || files.values().map(Vec::len).sum::<usize>() > 25 * 1024 * 1024
    {
        anyhow::bail!("Report package exceeds its safe path, count or byte limits.");
    }
    if generated_at.is_empty() || generated_at.len() > 100 {
        anyhow::bail!("Record a bounded report timestamp.");
    }
    let resolved = crate::commission_manifest::resolve_report_manifest(files.get("manifest.json").map(Vec::as_slice), fallback)?;
    let m = resolved.manifest;
    for reference in [m.rplan_path.as_deref(), m.rctx_path.as_deref(), m.audit_certificate_path.as_deref()]
        .into_iter()
        .flatten()
    {
        if !safe_path(reference) {
            anyhow::bail!("Report references must be safe relative package paths.");
        }
    }
    let mut allowed: Vec<String> = vec!["manifest.json".into()];
    allowed.extend(
        REQUIRED_ANALYSIS_FILES
            .iter()
            .chain(OPTIONAL_ANALYSIS_FILES)
            .map(|n| format!("analysis/{n}")),
    );
    allowed.extend(
        [
            "districts.png",
            "district_map.png",
            "political.png",
            "compactness.png",
        ]
        .map(|n| format!("maps/{n}")),
    );
    allowed.extend(
        [m.rplan_path.clone(), m.rctx_path.clone(), m.audit_certificate_path.clone()]
            .into_iter()
            .flatten(),
    );
    // Native fallback ignores obsolete references in the rejected manifest.
    // Permit their bounded supplied bytes without interpreting or following them.
    if resolved.fallback_used {
        if let Some(original) = files.get("manifest.json").and_then(|raw| serde_json::from_slice::<Value>(raw).ok()) {
            for key in ["rplan_path", "rctx_path", "audit_certificate_path"] {
                if let Some(reference) = original.get(key).and_then(Value::as_str).filter(|p| safe_path(p)) {
                    allowed.push(reference.to_string());
                }
            }
        }
    }
    if files.keys().any(|p| !allowed.contains(p)) {
        anyhow::bail!("Unexpected report source file.");
    }
    let mut analysis = BTreeMap::new();
    for name in REQUIRED_ANALYSIS_FILES
        .iter()
        .chain(OPTIONAL_ANALYSIS_FILES)
    {
        let path = format!("analysis/{name}");
        if files.contains_key(&path) {
            analysis.insert(
                (*name).into(),
                serde_json::from_str(source_text(files, &path)?)?,
            );
        }
    }
    let context = m
        .rctx_path
        .as_deref()
        .map(|p| source_text(files, p))
        .transpose()?;
    let certificate = m
        .audit_certificate_path
        .as_deref()
        .map(|p| source_text(files, p))
        .transpose()?;
    let audit = report_assembly::build_audit_from_values(
        &m,
        source_hashes_from_context(context)?,
        lineage_from_certificate(certificate)?,
    )?;
    let mut maps = serde_json::Map::new();
    for name in [
        "districts.png",
        "district_map.png",
        "political.png",
        "compactness.png",
    ] {
        if let Some(raw) = files.get(&format!("maps/{name}")) {
            use base64::Engine;
            maps.insert(
                name.trim_end_matches(".png").replace('.', "_"),
                json!(base64::engine::general_purpose::STANDARD.encode(raw)),
            );
        }
    }
    let maps = if maps.is_empty() {
        json!({"status":"unavailable"})
    } else {
        Value::Object(maps)
    };
    report_assembly::assemble_report_from_values(&m, &analysis, audit, maps, generated_at)
}
