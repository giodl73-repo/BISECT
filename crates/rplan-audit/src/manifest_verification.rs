//! Shared native manifest-to-certificate checks; source adapters supply bytes and digests.
use serde::{Deserialize,Serialize};
macro_rules! fail {($($arg:tt)*)=>{return Err(format!($($arg)*))};}
#[derive(Debug,Default,Deserialize)]
#[serde(default)]
pub struct ManifestAuditBindings {
 pub audit_certificate_path:Option<String>,pub audit_certificate_sha256:Option<String>,
 pub audit_certificate_content_hash:Option<String>,pub audit_result:Option<String>,pub legal_profile_id:Option<String>,pub context_hash:Option<String>,
 pub ilp_method:Option<String>,pub ilp_fallback:Option<String>,pub ilp_audit_summary_sha256:Option<String>,pub ilp_audit_summary_path:Option<String>,
 pub rplan_path:Option<String>,pub rctx_path:Option<String>,pub adjacency_sha256:String,pub tiger_sha256:Option<String>,
}
#[derive(Debug,Serialize)]
pub struct CertificateBindingReceipt {
 pub certificate_path:String,pub certificate_id:String,pub content_hash:String,pub plan_hash:String,pub context_hash:Option<String>,pub recorded_audit_result:crate::AuditResult,
}
pub fn verify_manifest_certificate(
    manifest: &ManifestAuditBindings,
    hash: impl Fn(&str) -> Result<String,String>,
    read: impl Fn(&str) -> Result<Vec<u8>,String>,
) -> Result<Option<CertificateBindingReceipt>,String> {
    let Some(certificate_rel) = manifest.audit_certificate_path.as_deref() else {
        return Ok(None);
    };

    if let Some(expected_sha256) = manifest.audit_certificate_sha256.as_deref() {
        if !expected_sha256.is_empty() {
            let actual_sha256 = hash(certificate_rel)
                .map_err(|e| format!("hash RPLAN audit certificate: {e}"))?;
            if actual_sha256 != expected_sha256 {
                fail!(
                    "RPLAN audit certificate SHA-256 mismatch for {}: manifest={}, current={}",
                    certificate_rel,
                    expected_sha256,
                    actual_sha256
                );
            }
        }
    }

    let certificate_bytes = read(certificate_rel).map_err(|e| {
        format!(
            "read RPLAN audit certificate {}: {e}",
            certificate_rel
        )
    })?;
    let certificate: crate::AuditCertificate = serde_json::from_slice(&certificate_bytes)
        .map_err(|e| format!("parse RPLAN audit certificate: {e}"))?;

    if let Some(expected_content_hash) = manifest.audit_certificate_content_hash.as_deref() {
        if certificate.content_hash != expected_content_hash {
            fail!(
                "RPLAN audit certificate content hash mismatch: manifest={}, certificate={}",
                expected_content_hash,
                certificate.content_hash
            );
        }
    }
    if let Some(expected_result) = manifest.audit_result.as_deref() {
        let actual_result = rplan_audit_result_label(&certificate.result);
        if actual_result != expected_result {
            fail!(
                "RPLAN audit result mismatch: manifest={}, certificate={}",
                expected_result,
                actual_result
            );
        }
    }
    if let Some(expected_profile_id) = manifest.legal_profile_id.as_deref() {
        if certificate.legal_profile.profile_id != expected_profile_id {
            fail!(
                "RPLAN legal profile mismatch: manifest={}, certificate={}",
                expected_profile_id,
                certificate.legal_profile.profile_id
            );
        }
    }
    if let Some(expected_context_hash) = manifest.context_hash.as_deref() {
        if certificate.context_hash.as_deref() != Some(expected_context_hash) {
            fail!(
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
        .ok_or_else(|| format!("manifest has audit_certificate_path but no rplan_path"))?;
    let rplan_text = read(rplan_rel).and_then(|bytes|String::from_utf8(bytes).map_err(|e|e.to_string()))
        .map_err(|e| format!("read RPLAN sidecar {rplan_rel}: {e}"))?;
    let rplan_document = rplan_io::read_rplan_str(&rplan_text)
        .map_err(|e| format!("parse RPLAN sidecar {rplan_rel}: {e}"))?;

    let context = if certificate.context_hash.is_some() {
        let rctx_rel = manifest.rctx_path.as_deref().ok_or_else(|| {
            format!("manifest has contextual audit certificate but no rctx_path")
        })?;
        let rctx_text = read(rctx_rel).and_then(|bytes|String::from_utf8(bytes).map_err(|e|e.to_string()))
            .map_err(|e| format!("read RPLAN context sidecar {rctx_rel}: {e}"))?;
        Some(
            rplan_io::read_rctx_str(&rctx_text)
                .map_err(|e| format!("parse RPLAN context sidecar {rctx_rel}: {e}"))?,
        )
    } else {
        None
    };
    if let Some(context) = context.as_ref() {
        verify_manifest_rplan_source_hashes(manifest, context)?;
    }

    crate::verify_audit_certificate(
        &certificate,
        Some(&rplan_document.plan),
        context.as_ref(),
    )
    .map_err(|e| format!("RPLAN audit certificate verification failed: {e}"))?;
    Ok(Some(CertificateBindingReceipt {certificate_path:certificate_rel.to_string(),certificate_id:certificate.certificate_id,content_hash:certificate.content_hash,plan_hash:certificate.plan_hash,context_hash:certificate.context_hash,recorded_audit_result:certificate.result}))
}

fn verify_manifest_rplan_algorithm_lineage(
    manifest: &ManifestAuditBindings,
    certificate: &crate::AuditCertificate,
) -> Result<(),String> {
    let expects_ilp_lineage = manifest.ilp_method.is_some() || manifest.ilp_fallback.is_some();
    if !expects_ilp_lineage {
        return Ok(());
    }
    let lineage = certificate.algorithm_lineage.as_ref().ok_or_else(|| {
        format!(
            "manifest records ILP audit metadata but certificate has no algorithm_lineage"
        )
    })?;
    if lineage.producer_crate != "bisect-ilp" {
        fail!(
            "RPLAN algorithm lineage producer mismatch: expected=bisect-ilp, certificate={}",
            lineage.producer_crate
        );
    }
    if let Some(expected_method) = manifest.ilp_method.as_deref() {
        if lineage.method != expected_method {
            fail!(
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
            fail!(
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
            fail!(
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
            fail!(
                "RPLAN algorithm lineage audit summary path mismatch: manifest={}, certificate={}",
                expected_path,
                actual.unwrap_or("(missing)")
            );
        }
    }
    Ok(())
}

fn verify_manifest_rplan_source_hashes(
    manifest: &ManifestAuditBindings,
    context: &rplan_core::RplanContext,
) -> Result<(),String> {
    if !manifest.adjacency_sha256.is_empty() {
        let expected = format!("sha256:{}", manifest.adjacency_sha256);
        let actual = context.source_hashes.entries.get("adjacency");
        if actual != Some(&expected) {
            fail!(
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
                fail!(
                    "RPLAN context geometry source hash mismatch: manifest={}, context={}",
                    expected,
                    actual.cloned().unwrap_or_else(|| "(missing)".to_string())
                );
            }
        }
    }

    Ok(())
}

pub fn rplan_audit_result_label(result: &crate::AuditResult) -> &'static str {
    match result {
        crate::AuditResult::Pass => "pass",
        crate::AuditResult::Fail => "fail",
        crate::AuditResult::PassWithWarnings => "pass-with-warnings",
    }
}

