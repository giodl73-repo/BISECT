use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const RCOUNT_VERSION: &str = "0.1-draft";
pub const SOURCE_HASH_PREFIX: &[u8] = b"RCOUNT_SOURCE_V1\0";
pub const RECORD_HASH_PREFIX: &[u8] = b"RCOUNT_RECORD_V1\0";
pub const FILE_HASH_PREFIX: &[u8] = b"RCOUNT_FILE_V1\0";
pub const PACKAGE_HASH_PREFIX: &[u8] = b"RCOUNT_PACKAGE_V1\0";
pub const EVENT_HASH_PREFIX: &[u8] = b"RCOUNT_EVENT_V1\0";
pub const PROOF_HASH_PREFIX: &[u8] = b"RCOUNT_PROOF_V1\0";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RcountCoreError {
    #[error("canonical JSON error: {0}")]
    CanonicalJson(String),
    #[error("duplicate selection id in contest {contest_id}: {selection_id}")]
    DuplicateSelectionId {
        contest_id: String,
        selection_id: String,
    },
    #[error("summary for contest {contest_id} reporting unit {reporting_unit_id} has duplicate selection id: {selection_id}")]
    DuplicateSummarySelection {
        contest_id: String,
        reporting_unit_id: String,
        selection_id: String,
    },
    #[error("summary for contest {contest_id} reporting unit {reporting_unit_id} references unknown selection id: {selection_id}")]
    UnknownSelection {
        contest_id: String,
        reporting_unit_id: String,
        selection_id: String,
    },
    #[error("votes and residual counts must be non-negative")]
    NegativeCount,
    #[error("contest selection sum mismatch for contest {contest_id} reporting unit {reporting_unit_id}: declared {declared_ballots}, computed {computed_ballots}")]
    ContestSelectionSumMismatch {
        contest_id: String,
        reporting_unit_id: String,
        declared_ballots: i64,
        computed_ballots: i64,
    },
    #[error("missing jurisdiction total summary for contest {contest_id} reporting unit {jurisdiction_reporting_unit_id}")]
    MissingJurisdictionTotal {
        contest_id: String,
        jurisdiction_reporting_unit_id: String,
    },
    #[error("jurisdiction total mismatch for contest {contest_id} selection {selection_id}: declared {declared_votes}, computed {computed_votes}")]
    JurisdictionSelectionMismatch {
        contest_id: String,
        selection_id: String,
        declared_votes: i64,
        computed_votes: i64,
    },
    #[error("jurisdiction residual mismatch for contest {contest_id} field {field}: declared {declared}, computed {computed}")]
    JurisdictionResidualMismatch {
        contest_id: String,
        field: String,
        declared: i64,
        computed: i64,
    },
    #[error("duplicate status event id: {event_id}")]
    DuplicateStatusEventId { event_id: String },
    #[error("status event {event_id} has the same before and after status")]
    NoStatusTransition { event_id: String },
    #[error("status event {event_id} must include authority and explanation")]
    IncompleteStatusEvent { event_id: String },
    #[error("missing canvass correction event from unofficial to canvassed")]
    MissingCanvassCorrectionEvent,
    #[error("missing summaries for status {status:?}")]
    MissingStatusSummaries { status: CountStatus },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SelectionKind {
    Candidate,
    WriteInBucket,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    pub selection_id: String,
    pub kind: SelectionKind,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contest {
    pub contest_id: String,
    pub title: String,
    pub vote_for: u32,
    pub selections: Vec<Selection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReportingUnitKind {
    Precinct,
    SplitPrecinct,
    VoteCenter,
    CentralCountBatch,
    MailBatch,
    ProvisionalBatch,
    JurisdictionTotal,
    DistrictTotal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportingUnit {
    pub reporting_unit_id: String,
    pub kind: ReportingUnitKind,
    pub parent_jurisdiction: String,
    #[serde(default)]
    pub source_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionTotal {
    pub selection_id: String,
    pub votes: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    pub contest_id: String,
    pub reporting_unit_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    pub status: CountStatus,
    pub totals: Vec<SelectionTotal>,
    pub undervotes: i64,
    pub overvotes: i64,
    pub blank_contests: i64,
    pub counted_ballots: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CountStatus {
    Unofficial,
    Canvassed,
    Recounted,
    Amended,
    Certified,
    Withdrawn,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatusEventType {
    InitialUnofficialReport,
    LateMailBatchAdded,
    ProvisionalAdjudication,
    BallotCureUpdate,
    DuplicateBallotResolution,
    WriteInAdjudication,
    RecountUpdate,
    CourtOrder,
    Certification,
    AmendedCertification,
    Correction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusEvent {
    pub event_id: String,
    pub event_type: StatusEventType,
    pub status_before: CountStatus,
    pub status_after: CountStatus,
    pub effective_at: String,
    pub authority: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RcountPackage {
    pub rcount_version: String,
    pub contests: Vec<Contest>,
    pub reporting_units: Vec<ReportingUnit>,
    pub summaries: Vec<Summary>,
    #[serde(default)]
    pub status_events: Vec<StatusEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquationPass {
    pub equation_id: String,
    pub contest_id: String,
    pub reporting_unit_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VerificationReport {
    pub passed: Vec<EquationPass>,
    pub failed: Vec<String>,
}

pub fn canonical_hash(prefix: &[u8], value: &Value) -> Result<String, RcountCoreError> {
    let canonical = canonicalize_value(value);
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|err| RcountCoreError::CanonicalJson(err.to_string()))?;
    let mut h = Sha256::new();
    h.update(prefix);
    h.update(bytes);
    Ok(format!("sha256:{:x}", h.finalize()))
}

pub fn record_hash<T: Serialize>(record: &T) -> Result<String, RcountCoreError> {
    let value = serde_json::to_value(record)
        .map_err(|err| RcountCoreError::CanonicalJson(err.to_string()))?;
    canonical_hash(RECORD_HASH_PREFIX, &value)
}

pub fn package_content_hash(package: &RcountPackage) -> Result<String, RcountCoreError> {
    let value = serde_json::to_value(package)
        .map_err(|err| RcountCoreError::CanonicalJson(err.to_string()))?;
    canonical_hash(PACKAGE_HASH_PREFIX, &value)
}

pub fn verify_package(package: &RcountPackage) -> Result<VerificationReport, RcountCoreError> {
    let contests: BTreeMap<&str, &Contest> = package
        .contests
        .iter()
        .map(|contest| (contest.contest_id.as_str(), contest))
        .collect();
    for contest in package.contests.iter() {
        validate_contest(contest)?;
    }

    let mut report = VerificationReport::default();
    for summary in package.summaries.iter() {
        let contest = contests.get(summary.contest_id.as_str()).ok_or_else(|| {
            RcountCoreError::UnknownSelection {
                contest_id: summary.contest_id.clone(),
                reporting_unit_id: summary.reporting_unit_id.clone(),
                selection_id: "<contest-missing>".to_string(),
            }
        })?;
        verify_contest_selection_sum(contest, summary)?;
        report.passed.push(EquationPass {
            equation_id: "contest_selection_sum".to_string(),
            contest_id: summary.contest_id.clone(),
            reporting_unit_id: summary.reporting_unit_id.clone(),
        });
    }
    report.passed.extend(verify_status_events(package)?);
    Ok(report)
}

pub fn verify_contest_selection_sum(
    contest: &Contest,
    summary: &Summary,
) -> Result<(), RcountCoreError> {
    ensure_non_negative(summary.undervotes)?;
    ensure_non_negative(summary.overvotes)?;
    ensure_non_negative(summary.blank_contests)?;
    ensure_non_negative(summary.counted_ballots)?;

    let valid_selection_ids: BTreeSet<&str> = contest
        .selections
        .iter()
        .map(|selection| selection.selection_id.as_str())
        .collect();
    let mut seen = BTreeSet::new();
    let mut selection_votes = 0i64;
    for total in summary.totals.iter() {
        ensure_non_negative(total.votes)?;
        if !seen.insert(total.selection_id.as_str()) {
            return Err(RcountCoreError::DuplicateSummarySelection {
                contest_id: summary.contest_id.clone(),
                reporting_unit_id: summary.reporting_unit_id.clone(),
                selection_id: total.selection_id.clone(),
            });
        }
        if !valid_selection_ids.contains(total.selection_id.as_str()) {
            return Err(RcountCoreError::UnknownSelection {
                contest_id: summary.contest_id.clone(),
                reporting_unit_id: summary.reporting_unit_id.clone(),
                selection_id: total.selection_id.clone(),
            });
        }
        selection_votes += total.votes;
    }

    let computed =
        selection_votes + summary.undervotes + summary.overvotes + summary.blank_contests;
    if computed != summary.counted_ballots {
        return Err(RcountCoreError::ContestSelectionSumMismatch {
            contest_id: summary.contest_id.clone(),
            reporting_unit_id: summary.reporting_unit_id.clone(),
            declared_ballots: summary.counted_ballots,
            computed_ballots: computed,
        });
    }
    Ok(())
}

pub fn verify_jurisdiction_total(
    contest_id: &str,
    jurisdiction_reporting_unit_id: &str,
    summaries: &[Summary],
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let totals: Vec<&Summary> = summaries
        .iter()
        .filter(|summary| {
            summary.contest_id == contest_id
                && summary.reporting_unit_id == jurisdiction_reporting_unit_id
        })
        .collect();
    if totals.is_empty() {
        return Err(RcountCoreError::MissingJurisdictionTotal {
            contest_id: contest_id.to_string(),
            jurisdiction_reporting_unit_id: jurisdiction_reporting_unit_id.to_string(),
        });
    }

    let mut passes = Vec::new();
    for total in totals {
        verify_jurisdiction_total_for_status(
            contest_id,
            jurisdiction_reporting_unit_id,
            total,
            summaries,
        )?;
        passes.push(EquationPass {
            equation_id: "jurisdiction_contest_total".to_string(),
            contest_id: contest_id.to_string(),
            reporting_unit_id: jurisdiction_reporting_unit_id.to_string(),
        });
    }
    Ok(passes)
}

fn verify_jurisdiction_total_for_status(
    contest_id: &str,
    jurisdiction_reporting_unit_id: &str,
    total: &Summary,
    summaries: &[Summary],
) -> Result<(), RcountCoreError> {
    let mut selection_sums: BTreeMap<&str, i64> = BTreeMap::new();
    let mut undervotes = 0i64;
    let mut overvotes = 0i64;
    let mut blank_contests = 0i64;
    let mut counted_ballots = 0i64;

    for summary in summaries.iter().filter(|summary| {
        summary.contest_id == contest_id
            && summary.reporting_unit_id != jurisdiction_reporting_unit_id
            && summary.status == total.status
    }) {
        for selection in summary.totals.iter() {
            *selection_sums
                .entry(selection.selection_id.as_str())
                .or_default() += selection.votes;
        }
        undervotes += summary.undervotes;
        overvotes += summary.overvotes;
        blank_contests += summary.blank_contests;
        counted_ballots += summary.counted_ballots;
    }

    for total_selection in total.totals.iter() {
        let computed = selection_sums
            .get(total_selection.selection_id.as_str())
            .copied()
            .unwrap_or_default();
        if total_selection.votes != computed {
            return Err(RcountCoreError::JurisdictionSelectionMismatch {
                contest_id: contest_id.to_string(),
                selection_id: total_selection.selection_id.clone(),
                declared_votes: total_selection.votes,
                computed_votes: computed,
            });
        }
    }
    check_residual(contest_id, "undervotes", total.undervotes, undervotes)?;
    check_residual(contest_id, "overvotes", total.overvotes, overvotes)?;
    check_residual(
        contest_id,
        "blank_contests",
        total.blank_contests,
        blank_contests,
    )?;
    check_residual(
        contest_id,
        "counted_ballots",
        total.counted_ballots,
        counted_ballots,
    )?;

    Ok(())
}

pub fn verify_status_events(package: &RcountPackage) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for event in package.status_events.iter() {
        if !seen.insert(event.event_id.as_str()) {
            return Err(RcountCoreError::DuplicateStatusEventId {
                event_id: event.event_id.clone(),
            });
        }
        if event.status_before == event.status_after
            && event.event_type != StatusEventType::InitialUnofficialReport
        {
            return Err(RcountCoreError::NoStatusTransition {
                event_id: event.event_id.clone(),
            });
        }
        if event.authority.trim().is_empty() || event.explanation.trim().is_empty() {
            return Err(RcountCoreError::IncompleteStatusEvent {
                event_id: event.event_id.clone(),
            });
        }
        passes.push(EquationPass {
            equation_id: "status_event_declared".to_string(),
            contest_id: "*".to_string(),
            reporting_unit_id: event.event_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_canvass_correction_event(
    package: &RcountPackage,
) -> Result<EquationPass, RcountCoreError> {
    let has_event = package.status_events.iter().any(|event| {
        event.event_type == StatusEventType::Correction
            && event.status_before == CountStatus::Unofficial
            && event.status_after == CountStatus::Canvassed
    });
    if !has_event {
        return Err(RcountCoreError::MissingCanvassCorrectionEvent);
    }
    for status in [CountStatus::Unofficial, CountStatus::Canvassed] {
        if !package
            .summaries
            .iter()
            .any(|summary| summary.status == status)
        {
            return Err(RcountCoreError::MissingStatusSummaries { status });
        }
    }
    Ok(EquationPass {
        equation_id: "canvass_correction_event".to_string(),
        contest_id: "*".to_string(),
        reporting_unit_id: "*".to_string(),
    })
}

pub fn synthetic_summary_basic_package() -> RcountPackage {
    let contest = Contest {
        contest_id: "syn-2024-mayor".to_string(),
        title: "Synthetic Mayor".to_string(),
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
    let reporting_units = vec![
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-001".to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-001".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: None,
        },
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-002".to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-002".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: None,
        },
        ReportingUnit {
            reporting_unit_id: "syn:jurisdiction:SYN".to_string(),
            kind: ReportingUnitKind::JurisdictionTotal,
            parent_jurisdiction: "syn".to_string(),
            source_ids: vec!["SYN".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: None,
        },
    ];
    let summaries = vec![
        summary("syn:precinct:P-001", 40, 35, 1, 3, 1, 0),
        summary("syn:precinct:P-002", 25, 30, 0, 4, 0, 1),
        summary("syn:jurisdiction:SYN", 65, 65, 1, 7, 1, 1),
    ];
    RcountPackage {
        rcount_version: RCOUNT_VERSION.to_string(),
        contests: vec![contest],
        reporting_units,
        summaries,
        status_events: vec![],
    }
}

pub fn synthetic_canvass_correction_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    let unofficial = vec![
        summary_with_status(
            "syn:precinct:P-001",
            CountStatus::Unofficial,
            40,
            34,
            1,
            3,
            1,
            0,
        ),
        summary_with_status(
            "syn:precinct:P-002",
            CountStatus::Unofficial,
            25,
            30,
            0,
            4,
            0,
            1,
        ),
        summary_with_status(
            "syn:jurisdiction:SYN",
            CountStatus::Unofficial,
            65,
            64,
            1,
            7,
            1,
            1,
        ),
    ];
    let canvassed = vec![
        summary_with_status(
            "syn:precinct:P-001",
            CountStatus::Canvassed,
            40,
            35,
            1,
            3,
            1,
            0,
        ),
        summary_with_status(
            "syn:precinct:P-002",
            CountStatus::Canvassed,
            25,
            30,
            0,
            4,
            0,
            1,
        ),
        summary_with_status(
            "syn:jurisdiction:SYN",
            CountStatus::Canvassed,
            65,
            65,
            1,
            7,
            1,
            1,
        ),
    ];
    package.summaries = unofficial.into_iter().chain(canvassed).collect();
    package.status_events = vec![
        StatusEvent {
            event_id: "event-0001".to_string(),
            event_type: StatusEventType::InitialUnofficialReport,
            status_before: CountStatus::Unofficial,
            status_after: CountStatus::Unofficial,
            effective_at: "2024-11-05T23:00:00Z".to_string(),
            authority: "SYN County Election Office".to_string(),
            source_refs: vec!["source:unofficial-election-night-export".to_string()],
            explanation: "Election-night unofficial report loaded from the first public export.".to_string(),
        },
        StatusEvent {
            event_id: "event-0002".to_string(),
            event_type: StatusEventType::Correction,
            status_before: CountStatus::Unofficial,
            status_after: CountStatus::Canvassed,
            effective_at: "2024-11-12T18:22:00Z".to_string(),
            authority: "SYN County Canvassing Board".to_string(),
            source_refs: vec!["source:canvass-minutes-2024-11-12".to_string()],
            explanation: "Canvass correction added one Candidate B vote in P-001 after write-in adjudication review.".to_string(),
        },
    ];
    package
}

fn summary(
    reporting_unit_id: &str,
    cand_a: i64,
    cand_b: i64,
    write_in: i64,
    undervotes: i64,
    overvotes: i64,
    blank_contests: i64,
) -> Summary {
    summary_with_status(
        reporting_unit_id,
        CountStatus::Canvassed,
        cand_a,
        cand_b,
        write_in,
        undervotes,
        overvotes,
        blank_contests,
    )
}

fn summary_with_status(
    reporting_unit_id: &str,
    status: CountStatus,
    cand_a: i64,
    cand_b: i64,
    write_in: i64,
    undervotes: i64,
    overvotes: i64,
    blank_contests: i64,
) -> Summary {
    Summary {
        contest_id: "syn-2024-mayor".to_string(),
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

fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(canonicalize_value).collect()),
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for key in keys {
                sorted.insert(key.clone(), canonicalize_value(&map[key]));
            }
            Value::Object(sorted)
        }
        other => other.clone(),
    }
}

fn validate_contest(contest: &Contest) -> Result<(), RcountCoreError> {
    let mut seen = BTreeSet::new();
    for selection in contest.selections.iter() {
        if !seen.insert(selection.selection_id.as_str()) {
            return Err(RcountCoreError::DuplicateSelectionId {
                contest_id: contest.contest_id.clone(),
                selection_id: selection.selection_id.clone(),
            });
        }
    }
    Ok(())
}

fn ensure_non_negative(value: i64) -> Result<(), RcountCoreError> {
    if value < 0 {
        return Err(RcountCoreError::NegativeCount);
    }
    Ok(())
}

fn check_residual(
    contest_id: &str,
    field: &str,
    declared: i64,
    computed: i64,
) -> Result<(), RcountCoreError> {
    if declared != computed {
        return Err(RcountCoreError::JurisdictionResidualMismatch {
            contest_id: contest_id.to_string(),
            field: field.to_string(),
            declared,
            computed,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_summary_basic_verifies_selection_sums() {
        let package = synthetic_summary_basic_package();
        let report = verify_package(&package).expect("synthetic summary package must verify");
        assert_eq!(report.passed.len(), 3);
        assert!(report.failed.is_empty());
    }

    #[test]
    fn synthetic_summary_basic_verifies_jurisdiction_total() {
        let package = synthetic_summary_basic_package();
        let passes =
            verify_jurisdiction_total("syn-2024-mayor", "syn:jurisdiction:SYN", &package.summaries)
                .expect("jurisdiction total must verify");
        assert_eq!(passes[0].equation_id, "jurisdiction_contest_total");
    }

    #[test]
    fn synthetic_canvass_correction_verifies_both_status_snapshots() {
        let package = synthetic_canvass_correction_package();
        let report = verify_package(&package).expect("canvass correction package must verify");
        assert_eq!(
            report
                .passed
                .iter()
                .filter(|pass| pass.equation_id == "contest_selection_sum")
                .count(),
            6
        );
        let jurisdiction_passes =
            verify_jurisdiction_total("syn-2024-mayor", "syn:jurisdiction:SYN", &package.summaries)
                .expect("both status snapshots must reconcile");
        assert_eq!(jurisdiction_passes.len(), 2);
    }

    #[test]
    fn canvass_correction_requires_public_event_and_snapshots() {
        let mut package = synthetic_canvass_correction_package();
        let pass = verify_canvass_correction_event(&package).unwrap();
        assert_eq!(pass.equation_id, "canvass_correction_event");

        package.status_events.clear();
        let err = verify_canvass_correction_event(&package)
            .expect_err("missing correction event must fail");
        assert!(matches!(
            err,
            RcountCoreError::MissingCanvassCorrectionEvent
        ));
    }

    #[test]
    fn bad_arithmetic_fails_with_specific_equation_error() {
        let mut package = synthetic_summary_basic_package();
        package.summaries[0].counted_ballots += 1;
        let err = verify_package(&package).expect_err("bad counted ballot total must fail");
        assert!(matches!(
            err,
            RcountCoreError::ContestSelectionSumMismatch { .. }
        ));
    }

    #[test]
    fn tampered_jurisdiction_total_fails() {
        let mut package = synthetic_summary_basic_package();
        let total = package
            .summaries
            .iter_mut()
            .find(|summary| summary.reporting_unit_id == "syn:jurisdiction:SYN")
            .unwrap();
        total.totals[0].votes += 1;
        let err =
            verify_jurisdiction_total("syn-2024-mayor", "syn:jurisdiction:SYN", &package.summaries)
                .expect_err("tampered jurisdiction total must fail");
        assert!(matches!(
            err,
            RcountCoreError::JurisdictionSelectionMismatch { .. }
        ));
    }

    #[test]
    fn record_hash_is_stable_for_equivalent_json_key_order() {
        let a = serde_json::json!({"b": 2, "a": {"d": 4, "c": 3}});
        let b = serde_json::json!({"a": {"c": 3, "d": 4}, "b": 2});
        assert_eq!(
            canonical_hash(RECORD_HASH_PREFIX, &a).unwrap(),
            canonical_hash(RECORD_HASH_PREFIX, &b).unwrap()
        );
    }

    #[test]
    fn package_content_hash_has_rcount_prefix() {
        let package = synthetic_summary_basic_package();
        let hash = package_content_hash(&package).unwrap();
        assert!(hash.starts_with("sha256:"));
        assert_eq!(hash.len(), "sha256:".len() + 64);
    }
}
