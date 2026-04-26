use std::path::Path;
use std::collections::HashMap;

pub struct AnalyzerContext<'a> {
    pub assignments: &'a HashMap<String, usize>,  // GEOID -> district_id (1-based)
    pub state_name: &'a str,
    pub state_code: &'a str,
    pub year: &'a str,
    pub version: &'a str,
    pub num_districts: usize,  // BOUNDARY-R2-01: required for ideal_pop
    pub data_root: &'a Path,
    pub output_root: &'a Path,
    /// Maximum allowed per-district population deviation as a fraction (not percent).
    /// Congressional default: 0.005 (±0.5%). State legislative default: 0.05 (±5%).
    /// Read from plan manifest when available so non-congressional plans pass correctly.
    pub balance_tolerance: f64,
}

pub trait Analyzer {
    type Output: serde::Serialize;
    fn name() -> &'static str where Self: Sized;
    fn run(ctx: &AnalyzerContext<'_>) -> anyhow::Result<Self::Output> where Self: Sized;
}

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum AnalyzerType {
    Compactness,
    Demographic,
    Political,
    Partisan,
    Urban,
    Summary,
    Contiguity,
    Splits,
    All,
}

impl AnalyzerType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Compactness => "compactness",
            Self::Demographic => "demographic",
            Self::Political => "political",
            Self::Partisan => "partisan",
            Self::Urban => "urban",
            Self::Summary => "summary",
            Self::Contiguity => "contiguity",
            Self::Splits => "splits",
            Self::All => "all",
        }
    }

    /// Returns all concrete analyzer types for use with `--types all`.
    ///
    /// **Excluded from this list:**
    /// - `Compactness`: requires TIGER geometry loading; handled separately in analyze.rs
    /// - `Partisan`: requires election data file; handled separately in analyze.rs
    ///
    /// These analyzers are invoked via explicit `--types compactness` or `--types partisan`.
    pub fn all_concrete() -> Vec<Self> {
        vec![
            Self::Demographic,
            Self::Political,
            Self::Urban,
            Self::Summary,
            Self::Contiguity,
            Self::Splits,
        ]
    }
}

/// Expand `All` variant to all concrete types, keeping non-All as-is.
pub fn expand_all_types(types: &[AnalyzerType]) -> Vec<AnalyzerType> {
    if types.iter().any(|t| *t == AnalyzerType::All) {
        AnalyzerType::all_concrete()
    } else {
        types.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_type_all_variants_parse() {
        use clap::ValueEnum;
        let variants = AnalyzerType::value_variants();
        assert!(variants.iter().any(|v| *v == AnalyzerType::Demographic));
        assert!(variants.iter().any(|v| *v == AnalyzerType::Political));
        assert!(variants.iter().any(|v| *v == AnalyzerType::Summary));
        assert!(variants.iter().any(|v| *v == AnalyzerType::All));
    }

    #[test]
    fn test_all_concrete_excludes_all_and_compactness() {
        let concrete = AnalyzerType::all_concrete();
        assert!(!concrete.contains(&AnalyzerType::All));
        assert!(!concrete.contains(&AnalyzerType::Compactness));
        assert!(concrete.contains(&AnalyzerType::Demographic));
        assert!(concrete.contains(&AnalyzerType::Political));
        assert!(concrete.contains(&AnalyzerType::Urban));
        assert!(concrete.contains(&AnalyzerType::Summary));
    }
}
