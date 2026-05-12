use rcount_core::{
    package_content_hash, verify_package, CountStatus, RcountPackage, SelectionTotal, Summary,
};
use rplan_core::{CanonicalOrder, DistrictPlan, PlanUnitIndex, RplanContext, UnitKind};
use rplan_io::{read_rctx_str, read_rplan_str, RplanDocument, RplanMetadataV02, RplanProvenance};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use thiserror::Error;

pub const RCOUNT_DISTRICT_AGGREGATION_VERSION: &str = "0.1-draft";

#[derive(Debug, Error)]
pub enum RcountDistrictError {
    #[error("core error: {0}")]
    Core(#[from] rcount_core::RcountCoreError),
    #[error("io error: {0}")]
    Io(#[from] rcount_io::RcountIoError),
    #[error("rplan core error: {0}")]
    RplanCore(#[from] rplan_core::RplanCoreError),
    #[error("rplan io error: {0}")]
    RplanIo(#[from] rplan_io::RplanIoError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("filesystem error: {0}")]
    Fs(#[from] std::io::Error),
    #[error("missing contest: {contest_id}")]
    MissingContest { contest_id: String },
    #[error("missing plan unit summary for contest {contest_id}, unit {reporting_unit_id}, status {status:?}")]
    MissingPlanUnitSummary {
        contest_id: String,
        reporting_unit_id: String,
        status: CountStatus,
    },
    #[error("duplicate plan unit summary for contest {contest_id}, unit {reporting_unit_id}, status {status:?}")]
    DuplicatePlanUnitSummary {
        contest_id: String,
        reporting_unit_id: String,
        status: CountStatus,
    },
    #[error("context unit universe does not match plan unit universe")]
    ContextUnitUniverseMismatch,
    #[error("context hash mismatch: declared {declared}, computed {computed}")]
    ContextHashMismatch { declared: String, computed: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistrictAggregationCheck {
    pub equation_id: String,
    pub district_id: u32,
    pub district_label: String,
    pub source_reporting_unit_count: usize,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistrictTotal {
    pub district_id: u32,
    pub district_label: String,
    pub source_reporting_unit_ids: Vec<String>,
    pub summary: Summary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistrictAggregationTranscript {
    pub aggregation_version: String,
    pub rcount_package_content_hash: String,
    pub rplan_plan_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rctx_context_hash: Option<String>,
    pub contest_id: String,
    pub status: CountStatus,
    pub unit_universe_hash: String,
    pub district_totals: Vec<DistrictTotal>,
    pub checks: Vec<DistrictAggregationCheck>,
}

pub fn aggregate_package_dir_with_plan_path(
    package_dir: &Path,
    plan_path: &Path,
    context_path: Option<&Path>,
    contest_id: &str,
    status: CountStatus,
) -> Result<DistrictAggregationTranscript, RcountDistrictError> {
    let (_, package) = rcount_io::read_package_dir(package_dir)?;
    let plan_doc = read_rplan_str(&std::fs::read_to_string(plan_path)?)?;
    let context = match context_path {
        Some(path) => Some(read_rctx_str(&std::fs::read_to_string(path)?)?),
        None => None,
    };
    aggregate_package_districts(
        &package,
        &plan_doc.plan,
        context.as_ref(),
        contest_id,
        status,
    )
}

pub fn aggregate_package_districts(
    package: &RcountPackage,
    plan: &DistrictPlan,
    context: Option<&RplanContext>,
    contest_id: &str,
    status: CountStatus,
) -> Result<DistrictAggregationTranscript, RcountDistrictError> {
    verify_package(package)?;
    plan.validate()?;
    validate_context_matches_plan(plan, context)?;
    let contest = package
        .contests
        .iter()
        .find(|contest| contest.contest_id == contest_id)
        .ok_or_else(|| RcountDistrictError::MissingContest {
            contest_id: contest_id.to_string(),
        })?;

    let package_hash = package_content_hash(package)?;
    let plan_hash = plan.plan_hash()?;
    let context_hash = context.map(|context| context.context_hash.clone());
    let summary_index = index_plan_unit_summaries(package, plan, contest_id, status)?;
    let mut district_sources: Vec<Vec<&Summary>> = vec![Vec::new(); plan.k];
    let mut district_units: Vec<Vec<String>> = vec![Vec::new(); plan.k];
    for (unit_idx, unit_id) in plan.units.unit_ids.iter().enumerate() {
        let district_id = plan.assignment[unit_idx] as usize;
        let summary = summary_index.get(unit_id.as_str()).ok_or_else(|| {
            RcountDistrictError::MissingPlanUnitSummary {
                contest_id: contest_id.to_string(),
                reporting_unit_id: unit_id.clone(),
                status,
            }
        })?;
        district_sources[district_id].push(*summary);
        district_units[district_id].push(unit_id.clone());
    }

    let mut district_totals = Vec::with_capacity(plan.k);
    let mut checks = Vec::with_capacity(plan.k);
    for district_id in 0..plan.k {
        let label = district_label(plan, district_id);
        let sources = &district_sources[district_id];
        let summary =
            sum_sources_for_district(contest, contest_id, status, district_id, &label, sources);
        checks.push(DistrictAggregationCheck {
            equation_id: "district_aggregation_total".to_string(),
            district_id: district_id as u32,
            district_label: label.clone(),
            source_reporting_unit_count: sources.len(),
            status: "pass".to_string(),
        });
        district_totals.push(DistrictTotal {
            district_id: district_id as u32,
            district_label: label,
            source_reporting_unit_ids: district_units[district_id].clone(),
            summary,
        });
    }

    Ok(DistrictAggregationTranscript {
        aggregation_version: RCOUNT_DISTRICT_AGGREGATION_VERSION.to_string(),
        rcount_package_content_hash: package_hash,
        rplan_plan_hash: plan_hash,
        rctx_context_hash: context_hash,
        contest_id: contest_id.to_string(),
        status,
        unit_universe_hash: plan.units.unit_universe_hash.clone(),
        district_totals,
        checks,
    })
}

pub fn synthetic_summary_basic_rplan_document() -> Result<RplanDocument, RcountDistrictError> {
    let mut units = PlanUnitIndex {
        unit_kind: UnitKind::Precinct,
        state: Some("SYN".to_string()),
        year: Some(2024),
        canonical_order: CanonicalOrder::ExplicitUnitIds,
        unit_ids: vec![
            "syn:precinct:P-001".to_string(),
            "syn:precinct:P-002".to_string(),
        ],
        unit_universe_hash: String::new(),
        source_id: Some("rcount:summary-basic".to_string()),
    };
    units.unit_universe_hash = units.compute_unit_universe_hash()?;
    let plan = DistrictPlan {
        schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
        units,
        assignment: vec![0, 1],
        k: 2,
        display_labels: vec!["SYN-D1".to_string(), "SYN-D2".to_string()],
        allow_empty_districts: false,
    };
    plan.validate()?;
    Ok(RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.to_string(),
        plan,
        metadata: RplanMetadataV02 {
            label: "synthetic-count-districts".to_string(),
            jurisdiction: "SYN".to_string(),
            chamber: "county-council".to_string(),
            created_at: "2026-05-12T00:00:00Z".to_string(),
            description: Some(
                "Two-precinct synthetic district assignment for RCOUNT aggregation.".to_string(),
            ),
        },
        provenance: RplanProvenance::default(),
        geometry: None,
        extensions: BTreeMap::new(),
    })
}

fn validate_context_matches_plan(
    plan: &DistrictPlan,
    context: Option<&RplanContext>,
) -> Result<(), RcountDistrictError> {
    let Some(context) = context else {
        return Ok(());
    };
    context.validate()?;
    let computed = context.compute_context_hash()?;
    if context.context_hash != computed {
        return Err(RcountDistrictError::ContextHashMismatch {
            declared: context.context_hash.clone(),
            computed,
        });
    }
    if context.units.unit_universe_hash != plan.units.unit_universe_hash
        || context.units.unit_ids != plan.units.unit_ids
    {
        return Err(RcountDistrictError::ContextUnitUniverseMismatch);
    }
    Ok(())
}

fn index_plan_unit_summaries<'a>(
    package: &'a RcountPackage,
    plan: &DistrictPlan,
    contest_id: &str,
    status: CountStatus,
) -> Result<BTreeMap<&'a str, &'a Summary>, RcountDistrictError> {
    let plan_units: BTreeSet<&str> = plan.units.unit_ids.iter().map(String::as_str).collect();
    let mut index = BTreeMap::new();
    for summary in package.summaries.iter().filter(|summary| {
        summary.contest_id == contest_id
            && summary.status == status
            && summary.batch_id.is_none()
            && plan_units.contains(summary.reporting_unit_id.as_str())
    }) {
        if index
            .insert(summary.reporting_unit_id.as_str(), summary)
            .is_some()
        {
            return Err(RcountDistrictError::DuplicatePlanUnitSummary {
                contest_id: contest_id.to_string(),
                reporting_unit_id: summary.reporting_unit_id.clone(),
                status,
            });
        }
    }
    Ok(index)
}

fn district_label(plan: &DistrictPlan, district_id: usize) -> String {
    plan.display_labels
        .get(district_id)
        .cloned()
        .unwrap_or_else(|| (district_id + 1).to_string())
}

fn sum_sources_for_district(
    contest: &rcount_core::Contest,
    contest_id: &str,
    status: CountStatus,
    district_id: usize,
    district_label: &str,
    sources: &[&Summary],
) -> Summary {
    let mut selection_sums: BTreeMap<&str, i64> = contest
        .selections
        .iter()
        .map(|selection| (selection.selection_id.as_str(), 0))
        .collect();
    let mut undervotes = 0;
    let mut overvotes = 0;
    let mut blank_contests = 0;
    let mut counted_ballots = 0;
    for source in sources {
        for total in &source.totals {
            *selection_sums
                .entry(total.selection_id.as_str())
                .or_default() += total.votes;
        }
        undervotes += source.undervotes;
        overvotes += source.overvotes;
        blank_contests += source.blank_contests;
        counted_ballots += source.counted_ballots;
    }
    Summary {
        contest_id: contest_id.to_string(),
        reporting_unit_id: format!("rplan:district:{district_id}:{district_label}"),
        batch_id: None,
        status,
        totals: contest
            .selections
            .iter()
            .map(|selection| SelectionTotal {
                selection_id: selection.selection_id.clone(),
                votes: selection_sums
                    .get(selection.selection_id.as_str())
                    .copied()
                    .unwrap_or_default(),
            })
            .collect(),
        undervotes,
        overvotes,
        blank_contests,
        counted_ballots,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcount_core::{synthetic_summary_basic_package, CountStatus};

    #[test]
    fn aggregates_summary_basic_into_rplan_districts() {
        let package = synthetic_summary_basic_package();
        let plan_doc = synthetic_summary_basic_rplan_document().unwrap();
        let transcript = aggregate_package_districts(
            &package,
            &plan_doc.plan,
            None,
            "syn-2024-mayor",
            CountStatus::Canvassed,
        )
        .unwrap();

        assert_eq!(transcript.district_totals.len(), 2);
        assert_eq!(transcript.checks.len(), 2);
        assert_eq!(transcript.district_totals[0].district_label, "SYN-D1");
        assert_eq!(transcript.district_totals[0].summary.counted_ballots, 80);
        assert_eq!(transcript.district_totals[1].summary.counted_ballots, 60);
        assert_eq!(transcript.district_totals[0].summary.totals[0].votes, 40);
        assert_eq!(transcript.district_totals[1].summary.totals[1].votes, 30);
    }
}
