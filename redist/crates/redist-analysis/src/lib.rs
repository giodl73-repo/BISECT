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
