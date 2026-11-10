//! Shared rendering of the native comparison artifact set. Callers supply
//! selected input bytes and their own producer identity.
use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::comparison::ComparisonReport;
use crate::html_comparison::{render_comparison_html, HtmlComparisonContext};
use crate::narrative::{render_narrative, NarrativeConfig};
use crate::narrative_manifest::{
    build_narrative_manifest_with_clock, serialize_manifest, CivicCounterProposalAttribution,
    NarrativeManifestInputs,
};

pub struct ComparisonProducer {
    pub version: String,
    pub build_commit: String,
    pub rustc_version: String,
}

pub struct ComparisonArtifacts {
    pub markdown: String,
    pub manifest_json: String,
    pub html: Option<String>,
}

pub fn render_native_comparison(
    report: &ComparisonReport,
    cfg: &NarrativeConfig,
    producer: ComparisonProducer,
    source_date_epoch: Option<i64>,
    include_html: bool,
) -> Result<ComparisonArtifacts, serde_json::Error> {
    let a = &report.plan_a;
    let b = &report.plan_b;
    let markdown = render_narrative(report, cfg);
    let (submission_type, civic_counter_proposal_attribution) = pick_civic_attribution(
        a.submission_type.as_deref(), &a.label, a.submitted_by.as_deref(), a.submitted_at.as_deref(),
        b.submission_type.as_deref(), &b.label, b.submitted_by.as_deref(), b.submitted_at.as_deref(),
    );
    let inputs = NarrativeManifestInputs {
        template_path_rel: "crates/bisect-report/src/narrative.rs".to_string(),
        template_sha256: sha256_hex(include_bytes!("narrative.rs")),
        leaning_threshold: cfg.leaning_threshold,
        close_call_band: cfg.close_call_band,
        moe_inputs: BTreeMap::new(),
        plan_a_label: a.label.clone(),
        plan_a_manifest_sha256: a.manifest_sha256.clone(),
        plan_b_label: b.label.clone(),
        plan_b_manifest_sha256: b.manifest_sha256.clone(),
        baseline_label: None,
        baseline_manifest_sha256: None,
        analysis_sha256: BTreeMap::from([
            ("plan_a".to_string(), a.analysis_sha256.clone()),
            ("plan_b".to_string(), b.analysis_sha256.clone()),
        ]),
        approved_by: cfg.approved_by.clone(),
        version: producer.version,
        build_commit: producer.build_commit,
        rustc_version: producer.rustc_version,
        submission_type,
        civic_counter_proposal_attribution,
        comments_label: None,
        comments_overlay_sha256: None,
    };
    let manifest = build_narrative_manifest_with_clock(inputs, source_date_epoch);
    let manifest_json = serialize_manifest(&manifest)?;
    let html = include_html.then(|| {
        render_comparison_html(report, cfg, &HtmlComparisonContext {
            narrative_markdown: markdown.clone(),
            template_sha256: manifest.template_sha256,
            build_commit_short: manifest.build_commit_short,
            version: manifest.version,
            approved_at: manifest.approved_at,
        })
    });
    Ok(ComparisonArtifacts { markdown, manifest_json, html })
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[allow(clippy::too_many_arguments)]
pub fn pick_civic_attribution(
    a_type: Option<&str>, a_label: &str, a_by: Option<&str>, a_at: Option<&str>,
    b_type: Option<&str>, b_label: &str, b_by: Option<&str>, b_at: Option<&str>,
) -> (Option<String>, Option<CivicCounterProposalAttribution>) {
    let civic = "civic_counter_proposal";
    let make = |label: &str, by: Option<&str>, at: Option<&str>| Some(CivicCounterProposalAttribution {
        plan_label: label.to_string(),
        submitted_by: by.unwrap_or("").to_string(),
        submitted_at: at.unwrap_or("").to_string(),
    });
    if b_type == Some(civic) {
        (Some(civic.to_string()), make(b_label, b_by, b_at))
    } else if a_type == Some(civic) {
        (Some(civic.to_string()), make(a_label, a_by, a_at))
    } else {
        (None, None)
    }
}
