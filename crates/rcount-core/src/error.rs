use crate::*;

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
    #[error("duplicate batch id: {batch_id}")]
    DuplicateBatchId { batch_id: String },
    #[error("summary for contest {contest_id} reporting unit {reporting_unit_id} references missing batch id: {batch_id}")]
    MissingBatch {
        contest_id: String,
        reporting_unit_id: String,
        batch_id: String,
    },
    #[error("batch total mismatch for batch {batch_id}: declared {declared_ballots}, summary {summary_ballots}")]
    BatchSummaryTotalMismatch {
        batch_id: String,
        declared_ballots: i64,
        summary_ballots: i64,
    },
    #[error("accepted ballots mismatch for batch {batch_id}: declared {declared_ballots}, counted plus rejected {computed_ballots}")]
    AcceptedBallotsMismatch {
        batch_id: String,
        declared_ballots: i64,
        computed_ballots: i64,
    },
    #[error("duplicate lineage event id: {lineage_id}")]
    DuplicateLineageId { lineage_id: String },
    #[error(
        "lineage event {lineage_id} references missing prior reporting unit: {reporting_unit_id}"
    )]
    MissingPriorLineageUnit {
        lineage_id: String,
        reporting_unit_id: String,
    },
    #[error(
        "lineage event {lineage_id} references missing current reporting unit: {reporting_unit_id}"
    )]
    MissingCurrentLineageUnit {
        lineage_id: String,
        reporting_unit_id: String,
    },
    #[error("lineage event {lineage_id} has invalid split cardinality")]
    InvalidSplitLineage { lineage_id: String },
    #[error("lineage event {lineage_id} has invalid merge cardinality")]
    InvalidMergeLineage { lineage_id: String },
    #[error("duplicate RHIST reference id: {reference_id}")]
    DuplicateRhistReference { reference_id: String },
    #[error("RHIST reference {reference_id} has invalid package hash: {package_hash}")]
    InvalidRhistPackageHash {
        reference_id: String,
        package_hash: String,
    },
    #[error("RHIST reference {reference_id} must include at least one cycle id")]
    EmptyRhistCycleRefs { reference_id: String },
    #[error("RHIST reference {reference_id} has unsupported role: {role}")]
    UnsupportedRhistReferenceRole { reference_id: String, role: String },
    #[error("duplicate RCTX reference id: {reference_id}")]
    DuplicateRctxReference { reference_id: String },
    #[error("RCTX reference {reference_id} has invalid context hash: {context_hash}")]
    InvalidRctxContextHash {
        reference_id: String,
        context_hash: String,
    },
    #[error("RCTX reference {reference_id} has invalid crosswalk hash: {crosswalk_hash}")]
    InvalidRctxCrosswalkHash {
        reference_id: String,
        crosswalk_hash: String,
    },
    #[error("RCTX reference {reference_id} has unsupported role: {role}")]
    UnsupportedRctxReferenceRole { reference_id: String, role: String },
    #[error("duplicate proof id: {proof_id}")]
    DuplicateProofId { proof_id: String },
    #[error("proof {proof_id} exposes candidate selections")]
    ChoiceBearingProof { proof_id: String },
    #[error("proof {proof_id} combines voter identity with ballot style and timestamp")]
    LinkableVoterProof { proof_id: String },
    #[error("proof {proof_id} has invalid token hash: {token_hash}")]
    InvalidProofTokenHash {
        proof_id: String,
        token_hash: String,
    },
    #[error("duplicate CVR contest row for cvr {cvr_id} contest {contest_id}")]
    DuplicateCvrContest { cvr_id: String, contest_id: String },
    #[error("CVR contest row {cvr_id}/{contest_id} has invalid mark cardinality")]
    InvalidCvrContestCardinality { cvr_id: String, contest_id: String },
    #[error(
        "CVR contest row {cvr_id}/{contest_id} references unknown selection id: {selection_id}"
    )]
    UnknownCvrSelection {
        cvr_id: String,
        contest_id: String,
        selection_id: String,
    },
    #[error(
        "missing summary for CVR aggregate contest {contest_id} reporting unit {reporting_unit_id}"
    )]
    MissingCvrSummary {
        contest_id: String,
        reporting_unit_id: String,
    },
    #[error("CVR summary mismatch for contest {contest_id} reporting unit {reporting_unit_id} field {field}: summary {summary}, cvr {cvr}")]
    CvrSummaryMismatch {
        contest_id: String,
        reporting_unit_id: String,
        field: String,
        summary: i64,
        cvr: i64,
    },
    #[error("duplicate RLA audit id: {audit_id}")]
    DuplicateRlaAuditId { audit_id: String },
    #[error("RLA audit {audit_id} has invalid risk limit ppm: {risk_limit_ppm}")]
    InvalidRlaRiskLimit {
        audit_id: String,
        risk_limit_ppm: u32,
    },
    #[error("RLA audit {audit_id} has invalid sample size: {sample_size}")]
    InvalidRlaSampleSize { audit_id: String, sample_size: u32 },
    #[error("RLA audit {audit_id} has unsupported sampling algorithm: {sampling_algorithm_id}")]
    UnsupportedRlaSamplingAlgorithm {
        audit_id: String,
        sampling_algorithm_id: String,
    },
    #[error("duplicate audit algorithm run id: {run_id}")]
    DuplicateAuditAlgorithmRunId { run_id: String },
    #[error("audit algorithm run {run_id} has invalid risk limit ppm: {risk_limit_ppm}")]
    InvalidAuditAlgorithmRiskLimit { run_id: String, risk_limit_ppm: u32 },
    #[error("audit algorithm run {run_id} has invalid MACRO design fields")]
    InvalidAuditMacroDesign { run_id: String },
    #[error("audit algorithm run {run_id} has invalid stratified/hybrid design")]
    InvalidStratifiedHybridDesign { run_id: String },
    #[error("audit algorithm run {run_id} references missing stratified/hybrid component run: {component_run_id}")]
    MissingStratifiedHybridComponent {
        run_id: String,
        component_run_id: String,
    },
    #[error("audit algorithm run {run_id} has invalid ranked-choice audit design")]
    InvalidRankedChoiceAuditDesign { run_id: String },
    #[error("audit algorithm run {run_id} step {step_index} has invalid ranked choices")]
    InvalidRankedChoiceSample { run_id: String, step_index: u32 },
    #[error("audit algorithm run {run_id} has invalid Bayesian audit design")]
    InvalidBayesianAuditDesign { run_id: String },
    #[error("audit algorithm run {run_id} has invalid observable-ballot audit design")]
    InvalidObservableBallotAuditDesign { run_id: String },
    #[error("audit algorithm run {run_id} step {step_index} references missing observable-ballot opening: {proof_id}")]
    MissingObservableBallotOpening {
        run_id: String,
        step_index: u32,
        proof_id: String,
    },
    #[error("audit algorithm run {run_id} has unsupported method id: {method_id}")]
    UnsupportedAuditAlgorithmMethod { run_id: String, method_id: String },
    #[error("audit algorithm run {run_id} has duplicate assertion id: {assertion_id}")]
    DuplicateAuditAssertion {
        run_id: String,
        assertion_id: String,
    },
    #[error("audit algorithm run {run_id} assertion {assertion_id} has invalid assorter bound")]
    InvalidAuditAssorterBound {
        run_id: String,
        assertion_id: String,
    },
    #[error("audit algorithm run {run_id} step {step_index} references missing assertion {assertion_id}")]
    MissingAuditAssertion {
        run_id: String,
        step_index: u32,
        assertion_id: String,
    },
    #[error("audit algorithm run {run_id} has duplicate sample step {step_index} for assertion {assertion_id}")]
    DuplicateAuditSampleStep {
        run_id: String,
        assertion_id: String,
        step_index: u32,
    },
    #[error("audit algorithm run {run_id} step {step_index} has invalid assorter value")]
    InvalidAuditAssorterValue { run_id: String, step_index: u32 },
    #[error(
        "audit algorithm run {run_id} step {step_index} has invalid p-value ppm: {p_value_ppm}"
    )]
    InvalidAuditPValue {
        run_id: String,
        step_index: u32,
        p_value_ppm: u32,
    },
    #[error("audit algorithm run {run_id} step {step_index} references missing batch comparison audit for batch {batch_id}")]
    MissingBatchComparisonAlgorithmEvidence {
        run_id: String,
        step_index: u32,
        batch_id: String,
    },
    #[error("audit algorithm run {run_id} step {step_index} batch comparison taint mismatch: declared {declared:?}, computed {computed:?}")]
    BatchComparisonAlgorithmTaintMismatch {
        run_id: String,
        step_index: u32,
        declared: RationalValue,
        computed: RationalValue,
    },
    #[error("audit algorithm run {run_id} batch comparison sample order is empty")]
    EmptyBatchComparisonAlgorithmSample { run_id: String },
    #[error("audit algorithm run {run_id} batch comparison audit {audit_id} has nonpositive reported margin: {reported_margin}")]
    InvalidBatchComparisonAlgorithmMargin {
        run_id: String,
        audit_id: String,
        reported_margin: i64,
    },
    #[error("audit algorithm run {run_id} batch comparison audit {audit_id} assertion mismatch")]
    BatchComparisonAlgorithmAssertionMismatch { run_id: String, audit_id: String },
    #[error("RLA audit {audit_id} has no CVR population for contest {contest_id}")]
    MissingRlaPopulation {
        audit_id: String,
        contest_id: String,
    },
    #[error(
        "RLA audit {audit_id} manifest hash mismatch: declared {declared}, computed {computed}"
    )]
    RlaManifestHashMismatch {
        audit_id: String,
        declared: String,
        computed: String,
    },
    #[error("RLA audit {audit_id} sample mismatch at draw {draw_index}: declared {declared_cvr_id}, computed {computed_cvr_id}")]
    RlaSampleMismatch {
        audit_id: String,
        draw_index: u32,
        declared_cvr_id: String,
        computed_cvr_id: String,
    },
    #[error("RLA audit {audit_id} has incomplete stopping-rule fields")]
    MissingRlaStoppingRule { audit_id: String },
    #[error("RLA audit {audit_id} has duplicate observation for draw {draw_index}")]
    DuplicateRlaObservation { audit_id: String, draw_index: u32 },
    #[error("RLA audit {audit_id} is missing observation for draw {draw_index}")]
    MissingRlaObservation { audit_id: String, draw_index: u32 },
    #[error("RLA audit {audit_id} observation draw {draw_index} references cvr {observed_cvr_id}, expected {expected_cvr_id}")]
    RlaObservationCvrMismatch {
        audit_id: String,
        draw_index: u32,
        expected_cvr_id: String,
        observed_cvr_id: String,
    },
    #[error("RLA audit {audit_id} declares status {declared:?}, computed {computed:?}")]
    RlaStoppingStatusMismatch {
        audit_id: String,
        declared: RlaStoppingStatus,
        computed: RlaStoppingStatus,
    },
    #[error("RLA audit {audit_id} declared discrepancy count {declared}, computed {computed}")]
    RlaDiscrepancyCountMismatch {
        audit_id: String,
        declared: usize,
        computed: usize,
    },
    #[error("RLA audit {audit_id} discrepancy mismatch at draw {draw_index}: declared {declared:?}, computed {computed:?}")]
    RlaDiscrepancyMismatch {
        audit_id: String,
        draw_index: u32,
        declared: RlaDiscrepancyKind,
        computed: RlaDiscrepancyKind,
    },
    #[error("RLA audit {audit_id} is missing margin metadata")]
    MissingRlaMarginMetadata { audit_id: String },
    #[error(
        "RLA audit {audit_id} margin metadata references missing selection id: {selection_id}"
    )]
    MissingRlaMarginSelection {
        audit_id: String,
        selection_id: String,
    },
    #[error("RLA audit {audit_id} reported margin is not positive: {margin}")]
    InvalidRlaReportedMargin { audit_id: String, margin: i64 },
    #[error("RLA audit {audit_id} reported winner votes mismatch for {selection_id}: declared {declared}, summary {summary}")]
    RlaWinnerVotesMismatch {
        audit_id: String,
        selection_id: String,
        declared: i64,
        summary: i64,
    },
    #[error("RLA audit {audit_id} reported loser votes mismatch for {selection_id}: declared {declared}, summary {summary}")]
    RlaLoserVotesMismatch {
        audit_id: String,
        selection_id: String,
        declared: i64,
        summary: i64,
    },
    #[error(
        "RLA audit {audit_id} reported margin mismatch: declared {declared}, summary {summary}"
    )]
    RlaReportedMarginMismatch {
        audit_id: String,
        declared: i64,
        summary: i64,
    },
    #[error("RLA audit {audit_id} diluted margin denominator mismatch: declared {declared}, summary {summary}")]
    RlaDilutedMarginDenominatorMismatch {
        audit_id: String,
        declared: i64,
        summary: i64,
    },
    #[error("RLA audit {audit_id} is missing statistical risk estimate")]
    MissingRlaRiskEstimate { audit_id: String },
    #[error("RLA audit {audit_id} risk estimate mismatch: declared {declared_ppm} ppm, computed {computed_ppm} ppm")]
    RlaRiskEstimateMismatch {
        audit_id: String,
        declared_ppm: u32,
        computed_ppm: u32,
    },
    #[error("RLA audit {audit_id} has unsupported jurisdiction method: {jurisdiction_method_id}")]
    UnsupportedRlaJurisdictionMethod {
        audit_id: String,
        jurisdiction_method_id: String,
    },
    #[error("RLA audit {audit_id} has invalid Colorado-style public seed: {public_seed}")]
    InvalidColoradoRlaSeed {
        audit_id: String,
        public_seed: String,
    },
    #[error("RLA audit {audit_id} is missing Colorado-style comparison audit fields")]
    MissingColoradoRlaComparisonFields { audit_id: String },
    #[error("RLA audit {audit_id} is missing California-style public audit tool fields")]
    MissingCaliforniaRlaPublicToolFields { audit_id: String },
    #[error("RLA audit {audit_id} has invalid California-style ballot manifest format: {ballot_manifest_format_id}")]
    InvalidCaliforniaRlaManifestFormat {
        audit_id: String,
        ballot_manifest_format_id: String,
    },
    #[error("RLA audit {audit_id} has invalid public audit software source URL: {source_url}")]
    InvalidRlaSoftwareSourceUrl {
        audit_id: String,
        source_url: String,
    },
    #[error("duplicate manual audit id: {audit_id}")]
    DuplicateManualAuditId { audit_id: String },
    #[error("manual audit {audit_id} is missing canvassed summary for contest {contest_id} reporting unit {reporting_unit_id}")]
    MissingManualAuditSummary {
        audit_id: String,
        contest_id: String,
        reporting_unit_id: String,
    },
    #[error("manual audit {audit_id} machine total mismatch for {selection_id}: declared {declared}, summary {summary}")]
    ManualAuditMachineTotalMismatch {
        audit_id: String,
        selection_id: String,
        declared: i64,
        summary: i64,
    },
    #[error("manual audit {audit_id} declares status {declared:?}, computed {computed:?}")]
    ManualAuditStatusMismatch {
        audit_id: String,
        declared: ManualAuditStatus,
        computed: ManualAuditStatus,
    },
    #[error("duplicate batch comparison audit id: {audit_id}")]
    DuplicateBatchComparisonAuditId { audit_id: String },
    #[error("batch comparison audit {audit_id} references missing batch: {batch_id}")]
    MissingBatchComparisonBatch { audit_id: String, batch_id: String },
    #[error("batch comparison audit {audit_id} batch size mismatch for {batch_id}: declared {declared}, batch manifest {manifest}")]
    BatchComparisonBatchSizeMismatch {
        audit_id: String,
        batch_id: String,
        declared: i64,
        manifest: i64,
    },
    #[error("batch comparison audit {audit_id} is missing batch summary for contest {contest_id} batch {batch_id}")]
    MissingBatchComparisonSummary {
        audit_id: String,
        contest_id: String,
        batch_id: String,
    },
    #[error("batch comparison audit {audit_id} reported total mismatch for {selection_id}: declared {declared}, summary {summary}")]
    BatchComparisonReportedTotalMismatch {
        audit_id: String,
        selection_id: String,
        declared: i64,
        summary: i64,
    },
    #[error("batch comparison audit {audit_id} is missing hand tally for {selection_id}")]
    MissingBatchComparisonHandTally {
        audit_id: String,
        selection_id: String,
    },
    #[error("batch comparison audit {audit_id} declared reported margin {declared}, computed {computed}")]
    BatchComparisonReportedMarginMismatch {
        audit_id: String,
        declared: i64,
        computed: i64,
    },
    #[error(
        "batch comparison audit {audit_id} declared hand margin {declared}, computed {computed}"
    )]
    BatchComparisonHandMarginMismatch {
        audit_id: String,
        declared: i64,
        computed: i64,
    },
    #[error(
        "batch comparison audit {audit_id} declared overstatement {declared}, computed {computed}"
    )]
    BatchComparisonOverstatementMismatch {
        audit_id: String,
        declared: i64,
        computed: i64,
    },
}
