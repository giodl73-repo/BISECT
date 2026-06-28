use crate::*;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BatchKind {
    ElectionDay,
    Mail,
    Provisional,
    CentralCount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchManifest {
    pub batch_id: String,
    pub reporting_unit_id: String,
    pub kind: BatchKind,
    pub status: CountStatus,
    pub accepted_ballots: i64,
    pub counted_ballots: i64,
    pub rejected_ballots: i64,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LineageKind {
    Unchanged,
    Split,
    Merge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportingUnitLineage {
    pub lineage_id: String,
    pub kind: LineageKind,
    pub prior_cycle: String,
    pub current_cycle: String,
    pub prior_reporting_unit_ids: Vec<String>,
    pub current_reporting_unit_ids: Vec<String>,
    pub authority: String,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RhistReference {
    pub reference_id: String,
    pub package_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_path: Option<String>,
    #[serde(default)]
    pub cycle_ids: Vec<String>,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RctxReference {
    pub reference_id: String,
    pub context_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crosswalk_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crosswalk_path: Option<String>,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InclusionProofKind {
    AnonymizedAcceptedBallotToken,
    AnonymizedCountedBallotToken,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InclusionProof {
    pub proof_id: String,
    pub kind: InclusionProofKind,
    pub token_hash: String,
    pub reporting_unit_id: String,
    #[serde(default)]
    pub candidate_selections: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voter_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ballot_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issued_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CvrContestRecord {
    pub cvr_id: String,
    pub contest_id: String,
    pub reporting_unit_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    pub status: CountStatus,
    #[serde(default)]
    pub selection_ids: Vec<String>,
    #[serde(default)]
    pub undervote: bool,
    #[serde(default)]
    pub overvote: bool,
    #[serde(default)]
    pub blank_contest: bool,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskLimitAudit {
    pub audit_id: String,
    pub contest_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction_method_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ballot_manifest_format_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_software_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_software_source_url: Option<String>,
    pub risk_limit_ppm: u32,
    pub public_seed: String,
    pub sampling_algorithm_id: String,
    pub manifest_hash: String,
    pub sample_size: u32,
    pub sample_draws: Vec<RlaSampleDraw>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observations: Vec<RlaSampleObservation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub discrepancies: Vec<RlaDiscrepancy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin: Option<RlaMarginMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stopping_rule_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_discrepancies: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_status: Option<RlaStoppingStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_risk_ppm: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RlaSampleDraw {
    pub draw_index: u32,
    pub cvr_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RlaSampleObservation {
    pub draw_index: u32,
    pub cvr_id: String,
    #[serde(default)]
    pub observed_selection_ids: Vec<String>,
    #[serde(default)]
    pub undervote: bool,
    #[serde(default)]
    pub overvote: bool,
    #[serde(default)]
    pub blank_contest: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RlaDiscrepancyKind {
    SelectionMismatch,
    ResidualMismatch,
    SelectionAndResidualMismatch,
    WrongCvrObserved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RlaDiscrepancy {
    pub draw_index: u32,
    pub cvr_id: String,
    pub kind: RlaDiscrepancyKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RlaMarginMetadata {
    pub winner_selection_id: String,
    pub loser_selection_id: String,
    pub reported_winner_votes: i64,
    pub reported_loser_votes: i64,
    pub reported_margin: i64,
    pub diluted_margin_denominator: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RationalValue {
    pub numerator: i64,
    pub denominator: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditAssertionKind {
    PluralityWinnerLoser,
    AssorterMean,
    ComparisonOverstatement,
    RankedChoiceAssertion,
    BayesianOutcome,
    ObservableBallotLinkage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditAssertion {
    pub assertion_id: String,
    pub kind: AuditAssertionKind,
    pub assorter_id: String,
    pub assorter_upper_bound: RationalValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub winner_selection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loser_selection_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditSamplingMode {
    WithReplacement,
    WithoutReplacement,
    Bernoulli,
    Weighted,
    Batch,
    BoundaryOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditAlgorithmDecision {
    Pass,
    Continue,
    Escalate,
    Boundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditSampleStep {
    pub step_index: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub round_index: Option<u32>,
    pub assertion_id: String,
    pub sample_unit_id: String,
    pub assorter_value: RationalValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bet: Option<RationalValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statistic: Option<RationalValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p_value_ppm: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ranked_choices: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditStratum {
    pub stratum_id: String,
    pub method_id: String,
    pub component_run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ballot_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allocation_ppm: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditAlgorithmRun {
    pub run_id: String,
    pub contest_id: String,
    pub method_id: String,
    pub sampling_mode: AuditSamplingMode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rcv_elimination_order: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_limit_ppm: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reported_winner_votes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reported_loser_votes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub macro_ballot_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub macro_reported_margin: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub macro_gamma: Option<RationalValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combining_rule_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nuisance_parameter: Option<RationalValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bayesian_prior_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bayesian_likelihood_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub posterior_winner_probability_ppm: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub posterior_risk_ppm: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub simulation_seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub posterior_draws: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibrated_risk_limit_ppm: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub strata: Vec<AuditStratum>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assertions: Vec<AuditAssertion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sample_steps: Vec<AuditSampleStep>,
    pub decision: AuditAlgorithmDecision,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RlaStoppingStatus {
    Pass,
    Escalate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ManualAuditStatus {
    Pass,
    Escalate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualAudit {
    pub audit_id: String,
    pub contest_id: String,
    pub reporting_unit_id: String,
    pub authority: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audited_batch_ids: Vec<String>,
    pub tolerance_votes: i64,
    pub machine_totals: Vec<SelectionTotal>,
    pub hand_totals: Vec<SelectionTotal>,
    pub declared_status: ManualAuditStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchComparisonAudit {
    pub audit_id: String,
    pub contest_id: String,
    pub batch_id: String,
    pub declared_batch_ballots: i64,
    pub winner_selection_id: String,
    pub loser_selection_id: String,
    pub reported_totals: Vec<SelectionTotal>,
    pub hand_totals: Vec<SelectionTotal>,
    pub declared_reported_margin: i64,
    pub declared_hand_margin: i64,
    pub declared_overstatement: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    #[serde(default)]
    pub batches: Vec<BatchManifest>,
    #[serde(default)]
    pub lineage: Vec<ReportingUnitLineage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rhist_refs: Vec<RhistReference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rctx_refs: Vec<RctxReference>,
    #[serde(default)]
    pub inclusion_proofs: Vec<InclusionProof>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cvr: Vec<CvrContestRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audit_algorithm_runs: Vec<AuditAlgorithmRun>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rla_audits: Vec<RiskLimitAudit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub manual_audits: Vec<ManualAudit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub batch_comparison_audits: Vec<BatchComparisonAudit>,
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
