pub mod analyzer;
pub mod bloc_voting;
pub mod bloc_voting_writer;
pub mod compactness;
pub mod compactness_evidence;
pub mod comparison;
pub mod contiguity;
pub mod county_names;
pub mod demographic;
pub mod dhondt;
pub mod ensemble_diagnostics;
pub mod exit_codes;
pub mod nesting;
pub mod partisan;
pub mod permutation;
pub mod political;
pub mod proportionality;
pub mod race_of_candidate;
pub mod split_standards;
pub mod splits;
pub mod summary;
pub mod urban;
pub mod vra_analysis;

pub use analyzer::{Analyzer, AnalyzerContext, AnalyzerType};
pub use bloc_voting::{
    cluster_bootstrap, compute_vif, fit_wls, hc3_stderr, holm_bonferroni, run_bloc_voting_family,
    BlocVotingConfig, BlocVotingError, BlocVotingFamilyResult, BlocVotingTest,
    BlocVotingTestResult, ClusterCi, Coef, Precinct, RegressionFit, RobustnessCheck,
};
pub use bloc_voting_writer::{
    build_bloc_voting_json, regression_specification_string, render_summary_md,
    write_bloc_voting_outputs, BlocVotingJson, CandidateBlock, EcologyBlock, FamilyDetail,
    ProvenanceBlock, RegressionBlock, WriteContext, ECOLOGY_CAVEAT,
};
pub use compactness::{
    all_metrics, axis_aligned_length_width_ratio, convex_hull_ratio, exact_reock,
    length_width_ratio, minimum_bounding_circle, polsby_popper, population_weighted_compactness,
    reock, schwartzberg, BoundingCircle, CompactnessError, CompactnessMetrics,
};
pub use comparison::{
    compare_plans, format_comparison_csv, format_comparison_json, format_comparison_table, jaccard,
    try_compare_plans, ComparisonError, PlanComparison,
};
pub use contiguity::{bfs_component_count, check_contiguity, ContiguityResult, DistrictContiguity};
pub use county_names::county_name;
pub use demographic::{
    DemographicAnalyzer, DemographicDistrict, DemographicError, DemographicResult,
};
pub use dhondt::{
    dhondt_allocate, gallagher_index, try_dhondt_allocate, try_gallagher_index, DhondtError,
};
pub use exit_codes::{
    compute_exit_code, compute_exit_code_with_flags, BIT_BALANCE, BIT_CONTIGUITY, BIT_MISSING_DATA,
    BIT_NESTING,
};
pub use nesting::{
    build_chamber_adjacency, compute_nest_ratio, validate_nesting, NestingValidation,
    NestingViolation,
};
pub use partisan::{
    bootstrap_ci, compute_declination, compute_efficiency_gap, compute_mean_median,
    compute_partisan_bias, compute_partisan_metrics, compute_seats_votes_curve, try_bootstrap_ci,
    try_compute_partisan_metrics, DistrictElection, MetricWithCI, PartisanError, PartisanMetrics,
    SeatsVotesCurve,
};
pub use political::{PoliticalAnalyzer, PoliticalDistrict, PoliticalError, PoliticalResult};
pub use proportionality::{
    aggregate_proportionality, ProportionalityAnalyzer, ProportionalityError, ProportionalityResult,
};
pub use race_of_candidate::{
    parse_race_of_candidate_csv, AnnotationSet, AttestationDocFormat, AttestationDocRecord,
    CandidateAnnotation, CandidateRace, CuratorRecord, RaceOfCandidateProvenance, RaceParseError,
};
pub use split_standards::{get_split_standard, SplitStandard};
pub use splits::{
    analyze_county_splits, analyze_county_splits_with_state, analyze_municipal_splits,
    county_fips_from_geoid, CountySplitResult, MunicipalSplitResult,
};
pub use summary::{
    try_merge_district, SummaryAnalyzer, SummaryDistrict, SummaryError, SummaryResult,
};
pub use urban::{UrbanAnalyzer, UrbanDistrict, UrbanResult};
pub use vra_analysis::{
    analyze_mm_districts, try_analyze_mm_districts, VraAnalysis, VraDistrict, VraError,
};
