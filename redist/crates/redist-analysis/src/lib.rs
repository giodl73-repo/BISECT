pub mod county_names;
pub mod dhondt;
pub mod compactness;
pub mod vra_analysis;
pub mod analyzer;
pub mod demographic;
pub mod political;
pub mod urban;
pub mod summary;
pub mod partisan;
pub mod nesting;
pub mod comparison;
pub mod contiguity;
pub mod splits;
pub mod split_standards;
pub mod exit_codes;
pub mod bloc_voting;
pub mod ensemble_diagnostics;
pub mod race_of_candidate;
pub mod bloc_voting_writer;

pub use compactness::{polsby_popper, reock, convex_hull_ratio, all_metrics, CompactnessMetrics, CompactnessError};
pub use vra_analysis::{analyze_mm_districts, VraAnalysis, VraDistrict};
pub use analyzer::{Analyzer, AnalyzerContext, AnalyzerType};
pub use demographic::{DemographicAnalyzer, DemographicResult, DemographicDistrict};
pub use political::{PoliticalAnalyzer, PoliticalResult, PoliticalDistrict};
pub use urban::{UrbanAnalyzer, UrbanResult, UrbanDistrict};
pub use summary::{SummaryAnalyzer, SummaryResult, SummaryDistrict};
pub use partisan::{
    DistrictElection, MetricWithCI, PartisanMetrics,
    compute_efficiency_gap, compute_mean_median, compute_partisan_bias,
    bootstrap_ci, compute_partisan_metrics,
};
pub use nesting::{
    NestingViolation, NestingValidation,
    build_chamber_adjacency, validate_nesting, compute_nest_ratio,
};
pub use comparison::{compare_plans, jaccard, format_comparison_table, format_comparison_json, format_comparison_csv, PlanComparison};
pub use contiguity::{check_contiguity, bfs_component_count, ContiguityResult, DistrictContiguity};
pub use splits::{analyze_county_splits, analyze_county_splits_with_state, analyze_municipal_splits, county_fips_from_geoid, CountySplitResult, MunicipalSplitResult};
pub use split_standards::{get_split_standard, SplitStandard};
pub use exit_codes::{compute_exit_code, compute_exit_code_with_flags, BIT_BALANCE, BIT_CONTIGUITY, BIT_NESTING, BIT_MISSING_DATA};
pub use dhondt::{dhondt_allocate, gallagher_index};
pub use county_names::county_name;
pub use bloc_voting::{
    fit_wls, hc3_stderr, compute_vif, holm_bonferroni, cluster_bootstrap,
    run_bloc_voting_family,
    Precinct, Coef, RegressionFit, BlocVotingError, ClusterCi,
    BlocVotingTest, BlocVotingConfig, BlocVotingTestResult, BlocVotingFamilyResult,
    RobustnessCheck,
};
pub use race_of_candidate::{
    parse_race_of_candidate_csv, AnnotationSet, AttestationDocFormat, AttestationDocRecord,
    CandidateAnnotation, CandidateRace, CuratorRecord, RaceOfCandidateProvenance, RaceParseError,
};
pub use bloc_voting_writer::{
    build_bloc_voting_json, render_summary_md, regression_specification_string,
    write_bloc_voting_outputs, BlocVotingJson, CandidateBlock, EcologyBlock, FamilyDetail,
    ProvenanceBlock, RegressionBlock, WriteContext, ECOLOGY_CAVEAT,
};
