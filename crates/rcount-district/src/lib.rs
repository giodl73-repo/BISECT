use rcount_core::{
    package_content_hash, verify_jurisdiction_total, verify_lineage_conservation, verify_package,
    CountStatus, LineageKind, RcountPackage, ReportingUnit, ReportingUnitKind,
    ReportingUnitLineage, Selection, SelectionKind, SelectionTotal, Summary,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyntheticElectionCycle {
    pub cycle_id: String,
    pub package: RcountPackage,
    pub plan: RplanDocument,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntheticCycleCheck {
    pub cycle_id: String,
    pub package_content_hash: String,
    pub rplan_plan_hash: String,
    pub current_reporting_units: Vec<String>,
    pub lineage_event_count: usize,
    pub district_count: usize,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyntheticMultiElectionHarness {
    pub harness_version: String,
    pub contest_id: String,
    pub status: CountStatus,
    pub cycles: Vec<SyntheticElectionCycle>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntheticMultiElectionTranscript {
    pub harness_version: String,
    pub contest_id: String,
    pub status: CountStatus,
    pub cycle_count: usize,
    pub cycle_checks: Vec<SyntheticCycleCheck>,
    pub district_aggregations: Vec<DistrictAggregationTranscript>,
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

pub fn synthetic_multi_election_harness(
) -> Result<SyntheticMultiElectionHarness, RcountDistrictError> {
    let contest_id = "syn-cycle-mayor";
    let status = CountStatus::Canvassed;
    let cycles = vec![
        SyntheticElectionCycle {
            cycle_id: "SYN-2024-general".to_string(),
            package: synthetic_cycle_2024_package(contest_id, status),
            plan: synthetic_rplan_document_for_units(
                "synthetic-cycle-2024",
                2024,
                &[
                    "syn:precinct:P-001",
                    "syn:precinct:P-002",
                    "syn:precinct:P-003",
                ],
                &[0, 1, 1],
            )?,
        },
        SyntheticElectionCycle {
            cycle_id: "SYN-2026-general".to_string(),
            package: synthetic_cycle_2026_package(contest_id, status),
            plan: synthetic_rplan_document_for_units(
                "synthetic-cycle-2026",
                2026,
                &[
                    "syn:precinct:P-001A",
                    "syn:precinct:P-001B",
                    "syn:precinct:P-002",
                    "syn:precinct:P-003",
                ],
                &[0, 0, 1, 1],
            )?,
        },
        SyntheticElectionCycle {
            cycle_id: "SYN-2028-general".to_string(),
            package: synthetic_cycle_2028_package(contest_id, status),
            plan: synthetic_rplan_document_for_units(
                "synthetic-cycle-2028",
                2028,
                &[
                    "syn:precinct:P-001A",
                    "syn:precinct:P-001B",
                    "syn:precinct:P-023",
                ],
                &[0, 0, 1],
            )?,
        },
    ];

    Ok(SyntheticMultiElectionHarness {
        harness_version: "0.1-draft".to_string(),
        contest_id: contest_id.to_string(),
        status,
        cycles,
    })
}

pub fn synthetic_bad_lineage_multi_election_harness(
) -> Result<SyntheticMultiElectionHarness, RcountDistrictError> {
    let mut harness = synthetic_multi_election_harness()?;
    let cycle = harness
        .cycles
        .iter_mut()
        .find(|cycle| cycle.cycle_id == "SYN-2028-general")
        .expect("synthetic harness includes 2028 cycle");
    cycle.package.lineage[0].current_reporting_unit_ids =
        vec!["syn:precinct:P-023-MISSING".to_string()];
    Ok(harness)
}

pub fn synthetic_stale_plan_multi_election_harness(
) -> Result<SyntheticMultiElectionHarness, RcountDistrictError> {
    let mut harness = synthetic_multi_election_harness()?;
    let cycle = harness
        .cycles
        .iter_mut()
        .find(|cycle| cycle.cycle_id == "SYN-2028-general")
        .expect("synthetic harness includes 2028 cycle");
    cycle.plan = synthetic_rplan_document_for_units(
        "synthetic-cycle-2028-stale-plan",
        2028,
        &[
            "syn:precinct:P-001A",
            "syn:precinct:P-001B",
            "syn:precinct:P-002",
        ],
        &[0, 0, 1],
    )?;
    Ok(harness)
}

pub fn verify_synthetic_multi_election_harness(
    harness: &SyntheticMultiElectionHarness,
) -> Result<SyntheticMultiElectionTranscript, RcountDistrictError> {
    let mut cycle_checks = Vec::new();
    let mut district_aggregations = Vec::new();

    for cycle in &harness.cycles {
        verify_package(&cycle.package)?;
        verify_jurisdiction_total(
            &harness.contest_id,
            "syn:jurisdiction:SYN",
            &cycle.package.summaries,
        )?;
        verify_lineage_conservation(&cycle.package)?;
        let aggregation = aggregate_package_districts(
            &cycle.package,
            &cycle.plan.plan,
            None,
            &harness.contest_id,
            harness.status,
        )?;
        cycle_checks.push(SyntheticCycleCheck {
            cycle_id: cycle.cycle_id.clone(),
            package_content_hash: package_content_hash(&cycle.package)?,
            rplan_plan_hash: cycle.plan.plan.plan_hash()?,
            current_reporting_units: cycle.plan.plan.units.unit_ids.clone(),
            lineage_event_count: cycle.package.lineage.len(),
            district_count: aggregation.district_totals.len(),
            status: "pass".to_string(),
        });
        district_aggregations.push(aggregation);
    }

    Ok(SyntheticMultiElectionTranscript {
        harness_version: harness.harness_version.clone(),
        contest_id: harness.contest_id.clone(),
        status: harness.status,
        cycle_count: harness.cycles.len(),
        cycle_checks,
        district_aggregations,
    })
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

pub fn synthetic_rplan_document_for_units(
    label: &str,
    year: u16,
    unit_ids: &[&str],
    assignment: &[u32],
) -> Result<RplanDocument, RcountDistrictError> {
    let mut units = PlanUnitIndex {
        unit_kind: UnitKind::Precinct,
        state: Some("SYN".to_string()),
        year: Some(year),
        canonical_order: CanonicalOrder::ExplicitUnitIds,
        unit_ids: unit_ids
            .iter()
            .map(|unit_id| (*unit_id).to_string())
            .collect(),
        unit_universe_hash: String::new(),
        source_id: Some(format!("rcount:{label}")),
    };
    units.unit_universe_hash = units.compute_unit_universe_hash()?;
    let plan = DistrictPlan {
        schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
        units,
        assignment: assignment.to_vec(),
        k: 2,
        display_labels: vec!["SYN-D1".to_string(), "SYN-D2".to_string()],
        allow_empty_districts: false,
    };
    plan.validate()?;
    Ok(RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.to_string(),
        plan,
        metadata: RplanMetadataV02 {
            label: label.to_string(),
            jurisdiction: "SYN".to_string(),
            chamber: "county-council".to_string(),
            created_at: "2026-05-12T00:00:00Z".to_string(),
            description: Some("Synthetic multi-election RCOUNT/RPLAN harness cycle.".to_string()),
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

fn synthetic_cycle_2024_package(contest_id: &str, status: CountStatus) -> RcountPackage {
    synthetic_cycle_package(
        contest_id,
        status,
        &[
            (
                "syn:precinct:P-001",
                "2024-11-05",
                None,
                (42, 33, 1, 3, 1, 0),
            ),
            (
                "syn:precinct:P-002",
                "2024-11-05",
                None,
                (27, 31, 0, 4, 0, 1),
            ),
            (
                "syn:precinct:P-003",
                "2024-11-05",
                None,
                (20, 22, 0, 2, 1, 0),
            ),
        ],
        vec![],
    )
}

fn synthetic_cycle_2026_package(contest_id: &str, status: CountStatus) -> RcountPackage {
    synthetic_cycle_package(
        contest_id,
        status,
        &[
            (
                "syn:precinct:P-001",
                "2024-11-05",
                Some("2026-11-03"),
                (0, 0, 0, 0, 0, 0),
            ),
            (
                "syn:precinct:P-001A",
                "2026-11-03",
                None,
                (24, 17, 0, 2, 0, 0),
            ),
            (
                "syn:precinct:P-001B",
                "2026-11-03",
                None,
                (21, 18, 1, 2, 1, 0),
            ),
            (
                "syn:precinct:P-002",
                "2024-11-05",
                None,
                (30, 33, 0, 3, 0, 1),
            ),
            (
                "syn:precinct:P-003",
                "2024-11-05",
                None,
                (22, 25, 0, 2, 1, 0),
            ),
        ],
        vec![ReportingUnitLineage {
            lineage_id: "lineage:2026:P-001-split".to_string(),
            kind: LineageKind::Split,
            prior_cycle: "SYN-2024-general".to_string(),
            current_cycle: "SYN-2026-general".to_string(),
            prior_reporting_unit_ids: vec!["syn:precinct:P-001".to_string()],
            current_reporting_unit_ids: vec![
                "syn:precinct:P-001A".to_string(),
                "syn:precinct:P-001B".to_string(),
            ],
            authority: "SYN County Boundary Board".to_string(),
            explanation: "P-001 split into two precincts before the 2026 general election."
                .to_string(),
        }],
    )
}

fn synthetic_cycle_2028_package(contest_id: &str, status: CountStatus) -> RcountPackage {
    synthetic_cycle_package(
        contest_id,
        status,
        &[
            (
                "syn:precinct:P-001A",
                "2026-11-03",
                None,
                (25, 18, 0, 2, 0, 0),
            ),
            (
                "syn:precinct:P-001B",
                "2026-11-03",
                None,
                (23, 19, 1, 1, 1, 0),
            ),
            (
                "syn:precinct:P-002",
                "2024-11-05",
                Some("2028-11-07"),
                (0, 0, 0, 0, 0, 0),
            ),
            (
                "syn:precinct:P-003",
                "2024-11-05",
                Some("2028-11-07"),
                (0, 0, 0, 0, 0, 0),
            ),
            (
                "syn:precinct:P-023",
                "2028-11-07",
                None,
                (56, 61, 1, 5, 1, 1),
            ),
        ],
        vec![ReportingUnitLineage {
            lineage_id: "lineage:2028:P-002-P-003-merge".to_string(),
            kind: LineageKind::Merge,
            prior_cycle: "SYN-2026-general".to_string(),
            current_cycle: "SYN-2028-general".to_string(),
            prior_reporting_unit_ids: vec![
                "syn:precinct:P-002".to_string(),
                "syn:precinct:P-003".to_string(),
            ],
            current_reporting_unit_ids: vec!["syn:precinct:P-023".to_string()],
            authority: "SYN County Boundary Board".to_string(),
            explanation: "P-002 and P-003 merged into P-023 before the 2028 general election."
                .to_string(),
        }],
    )
}

fn synthetic_cycle_package(
    contest_id: &str,
    status: CountStatus,
    units: &[(&str, &str, Option<&str>, (i64, i64, i64, i64, i64, i64))],
    lineage: Vec<ReportingUnitLineage>,
) -> RcountPackage {
    let contest = rcount_core::Contest {
        contest_id: contest_id.to_string(),
        title: "Synthetic Cycle Mayor".to_string(),
        vote_for: 1,
        selections: vec![
            Selection {
                selection_id: "cand-a".to_string(),
                kind: SelectionKind::Candidate,
                label: "Candidate A".to_string(),
            },
            Selection {
                selection_id: "cand-b".to_string(),
                kind: SelectionKind::Candidate,
                label: "Candidate B".to_string(),
            },
            Selection {
                selection_id: "write-in".to_string(),
                kind: SelectionKind::WriteInBucket,
                label: "Write-in".to_string(),
            },
        ],
    };
    let mut reporting_units: Vec<ReportingUnit> = units
        .iter()
        .map(|(unit_id, valid_from, valid_to, _)| ReportingUnit {
            reporting_unit_id: (*unit_id).to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec![unit_id.replace("syn:precinct:", "")],
            valid_from: Some((*valid_from).to_string()),
            valid_to: valid_to.map(str::to_string),
        })
        .collect();
    reporting_units.push(ReportingUnit {
        reporting_unit_id: "syn:jurisdiction:SYN".to_string(),
        kind: ReportingUnitKind::JurisdictionTotal,
        parent_jurisdiction: "syn".to_string(),
        source_ids: vec!["SYN".to_string()],
        valid_from: Some("2024-11-05".to_string()),
        valid_to: None,
    });

    let mut summaries: Vec<Summary> = units
        .iter()
        .filter(|(_, _, valid_to, _)| valid_to.is_none())
        .map(|(unit_id, _, _, totals)| {
            cycle_summary_with_status(contest_id, unit_id, status, *totals)
        })
        .collect();
    summaries.push(jurisdiction_summary(contest_id, status, &summaries));

    RcountPackage {
        rcount_version: rcount_core::RCOUNT_VERSION.to_string(),
        contests: vec![contest],
        reporting_units,
        batches: vec![],
        lineage,
        inclusion_proofs: vec![],
        cvr: vec![],
        rla_audits: vec![],
        manual_audits: vec![],
        summaries,
        status_events: vec![],
    }
}

fn jurisdiction_summary(contest_id: &str, status: CountStatus, summaries: &[Summary]) -> Summary {
    let mut cand_a = 0;
    let mut cand_b = 0;
    let mut write_in = 0;
    let mut undervotes = 0;
    let mut overvotes = 0;
    let mut blank_contests = 0;
    for summary in summaries {
        cand_a += summary.totals[0].votes;
        cand_b += summary.totals[1].votes;
        write_in += summary.totals[2].votes;
        undervotes += summary.undervotes;
        overvotes += summary.overvotes;
        blank_contests += summary.blank_contests;
    }
    cycle_summary_with_status(
        contest_id,
        "syn:jurisdiction:SYN",
        status,
        (
            cand_a,
            cand_b,
            write_in,
            undervotes,
            overvotes,
            blank_contests,
        ),
    )
}

fn cycle_summary_with_status(
    contest_id: &str,
    reporting_unit_id: &str,
    status: CountStatus,
    totals: (i64, i64, i64, i64, i64, i64),
) -> Summary {
    let (cand_a, cand_b, write_in, undervotes, overvotes, blank_contests) = totals;
    Summary {
        contest_id: contest_id.to_string(),
        reporting_unit_id: reporting_unit_id.to_string(),
        batch_id: None,
        status,
        totals: vec![
            SelectionTotal {
                selection_id: "cand-a".to_string(),
                votes: cand_a,
            },
            SelectionTotal {
                selection_id: "cand-b".to_string(),
                votes: cand_b,
            },
            SelectionTotal {
                selection_id: "write-in".to_string(),
                votes: write_in,
            },
        ],
        undervotes,
        overvotes,
        blank_contests,
        counted_ballots: cand_a + cand_b + write_in + undervotes + overvotes + blank_contests,
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

    #[test]
    fn verifies_multi_election_harness_across_lineage_and_districts() {
        let harness = synthetic_multi_election_harness().unwrap();
        let transcript = verify_synthetic_multi_election_harness(&harness).unwrap();

        assert_eq!(transcript.cycle_count, 3);
        assert_eq!(transcript.district_aggregations.len(), 3);
        assert_eq!(transcript.cycle_checks[0].lineage_event_count, 0);
        assert_eq!(transcript.cycle_checks[1].lineage_event_count, 1);
        assert_eq!(transcript.cycle_checks[2].lineage_event_count, 1);
        assert_eq!(
            transcript.district_aggregations[2].district_totals[1]
                .summary
                .reporting_unit_id,
            "rplan:district:1:SYN-D2"
        );
    }

    #[test]
    fn rejects_multi_election_harness_with_bad_lineage() {
        let harness = synthetic_bad_lineage_multi_election_harness().unwrap();
        let err = verify_synthetic_multi_election_harness(&harness).unwrap_err();
        assert!(matches!(
            err,
            RcountDistrictError::Core(
                rcount_core::RcountCoreError::MissingCurrentLineageUnit { .. }
            )
        ));
    }

    #[test]
    fn rejects_multi_election_harness_with_stale_plan_unit() {
        let harness = synthetic_stale_plan_multi_election_harness().unwrap();
        let err = verify_synthetic_multi_election_harness(&harness).unwrap_err();
        assert!(matches!(
            err,
            RcountDistrictError::MissingPlanUnitSummary {
                reporting_unit_id,
                ..
            } if reporting_unit_id == "syn:precinct:P-002"
        ));
    }
}
