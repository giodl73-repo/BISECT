//! Native report fallback metadata; never recovers or authenticates missing provenance.
use crate::{PlanManifest, manifest_schema::ReadOnlyPlanManifest};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReportFallback {
    pub label: String,
    pub year: String,
}
pub fn fallback_manifest(label: String, year: String) -> PlanManifest {
    PlanManifest {
        label,
        year,
        ..Default::default()
    }
}
pub struct ResolvedReportManifest {
    pub manifest: ReadOnlyPlanManifest,
    pub fallback_used: bool,
    pub source_status: &'static str,
}
pub fn resolve_report_manifest(
    raw: Option<&[u8]>,
    fallback: Option<&ReportFallback>,
) -> anyhow::Result<ResolvedReportManifest> {
    if raw.is_some_and(|b| b.len() > 16 * 1024 * 1024) {
        anyhow::bail!("Commission manifest exceeds 16 MiB.");
    }
    if fallback.is_some_and(|f| f.label.len() > 200 || f.year.len() > 200) {
        anyhow::bail!("Fallback report label/year exceeds 200 bytes.");
    }
    let parsed = raw.map(serde_json::from_slice::<ReadOnlyPlanManifest>);
    match parsed {
        Some(Ok(manifest)) => Ok(ResolvedReportManifest {
            manifest,
            fallback_used: false,
            source_status: "supplied",
        }),
        other => {
            let source_status = if other.is_none() {
                "missing"
            } else {
                "invalid"
            };
            let Some(fallback) = fallback else {
                return match other {
                    Some(Err(e)) => Err(e.into()),
                    _ => Err(anyhow::anyhow!(
                        "Required report source missing: manifest.json"
                    )),
                };
            };
            Ok(ResolvedReportManifest {
                manifest: ReadOnlyPlanManifest { label: fallback.label.clone(), year: fallback.year.clone(), ..Default::default() },
                fallback_used: true,
                source_status,
            })
        }
    }
}
