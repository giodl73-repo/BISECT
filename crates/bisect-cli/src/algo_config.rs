/// Algorithm configuration YAML loading — Phase 7 of Spec 7 (Label-Based Run Management).
///
/// Loads `configs/X.yml` into an `AlgoYaml` struct and converts it to the
/// three-layer `AlgorithmConfig` compositor used by the runner.
///
/// YAML schema (§7.1):
/// ```yaml
/// name: official_proposal
/// description: "..."
/// algorithm:
///   structure: apportion-regions   # maps to SplitStrategy
///   weights: county                # maps to WeightSpec
///   alpha_county: 2.0
///   search: convergence            # maps to SeedCompositor
///   convergence_threshold: 600
///   seed: 42                        # fixed METIS seed for reproducibility
///   balance_tolerance: 0.5
/// workers: 6
/// years: ["2020", "2010", "2000"]
/// analysis_types: [compactness, splits, summary]
/// ```
use std::path::Path;

// ---------------------------------------------------------------------------
// Public structs
// ---------------------------------------------------------------------------

/// Top-level YAML document for a label's algorithm configuration.
///
/// This is the committed, human-editable file in `configs/X.yml`.
/// Separate from `.bisect` (auto-managed registry) and `runs/X/index.json`
/// (auto-managed build index).
pub use bisect_runner::config::{AlgoYaml, AlgorithmSection};
// ---------------------------------------------------------------------------
// impl AlgoYaml
// ---------------------------------------------------------------------------

pub trait NativeAlgorithmConfig {
    fn to_algorithm_config(&self) -> Result<crate::runner::AlgorithmConfig, String>;
}
impl NativeAlgorithmConfig for AlgoYaml {
    /// Convert to `AlgorithmConfig` using the three-layer compositor logic.
    ///
    /// Maps structure/weights/search strings to SplitStrategy/WeightSpec/SeedCompositor.
    /// Returns `[CONFIG]` error for unknown values or missing conditional fields.
    fn to_algorithm_config(&self) -> Result<crate::runner::AlgorithmConfig, String> {
        use bisect_runner::config::{ResolvedSplit as S,ResolvedSearch as Q,ResolvedEngine as E,ResolvedVertex as V};
        use crate::runner::{AlgorithmConfig,SplitStrategy,SeedCompositor,WeightSpec,MetisParams,AreaSectionInit};
        use crate::vertex_weights::VertexConstraintKind;
        let r=self.resolve()?;
        let split=match r.split {
            S::Bisect=>SplitStrategy::Bisect,S::NWay=>SplitStrategy::NWay,S::GeoSection=>SplitStrategy::GeoSection,
            S::AreaSection{area_swing,..}=>SplitStrategy::AreaSection{area_swing,area_section_init:AreaSectionInit::RatioOptimal},
            S::VraSection{w_vra}=>SplitStrategy::VraSection{w_vra},S::ApportionRegions=>SplitStrategy::ApportionRegions,
            S::CompactBisect{epsilon}=>SplitStrategy::CompactBisect{epsilon},
            S::MovingKnife{n_orientations,metric}=>SplitStrategy::MovingKnife{n_orientations,metric},
            S::Spectral{max_iters}=>SplitStrategy::Spectral{max_iters},S::Regionalization=>SplitStrategy::Regionalization,
            S::FlowConstruction=>SplitStrategy::FlowConstruction,
        };
        let seeds=match r.seeds {
            Q::Single=>SeedCompositor::Single,Q::Multi{seeds}=>SeedCompositor::Multi{seeds},
            Q::ConvergenceSweep{threshold}=>SeedCompositor::ConvergenceSweep{threshold},
            Q::ShortBurst{burst_length,n_bursts,p}=>SeedCompositor::ShortBurst{burst_length,n_bursts,p},
            Q::ShortBurstForest{burst_length,n_bursts,p}=>SeedCompositor::ShortBurstForest{burst_length,n_bursts,p},
            Q::ShortBurstMergeSplit{burst_length,n_bursts,p}=>SeedCompositor::ShortBurstMergeSplit{burst_length,n_bursts,p},
        };
        let engine=match r.metis.engine{E::CFfi=>bisect_apportion::split::MetisEngine::CFfi,E::RedistMetis=>bisect_apportion::split::MetisEngine::RedistMetis,E::Gpmetis=>bisect_apportion::split::MetisEngine::Gpmetis};
        Ok(AlgorithmConfig {split,seeds,
            weights:WeightSpec{geographic:r.weights.geographic,minority_weighting:r.weights.minority_weighting,alpha_county:r.weights.alpha_county,..WeightSpec::default()},
            vertex_constraints:r.vertex_constraints.into_iter().map(|v|match v{V::Population=>VertexConstraintKind::Population,V::Area=>VertexConstraintKind::Area}).collect(),
            metis:MetisParams{engine,seed:r.metis.seed,ufactor:r.metis.ufactor,niter:r.metis.niter},mode_label:None})
    }


}

// ---------------------------------------------------------------------------
// `BISECT config new` — write a template YAML to configs/X.yml
// ---------------------------------------------------------------------------

/// Write a template config YAML file to `configs/{name}.yml`.
///
/// Called by `BISECT config new --name X ...`.
/// Returns `[CONFIG]` error if the directory cannot be created or the file
/// already exists (unless `force` is true).
pub fn write_template_config(
    name: &str,
    structure: &str,
    weights: Option<&str>,
    alpha_county: Option<f64>,
    search: Option<&str>,
    convergence_threshold: Option<u32>,
    seeds: Option<usize>,
    balance_tolerance: Option<f64>,
    years: &[String],
    workers: usize,
    output_path: Option<&Path>,
    force: bool,
    dry_run: bool,
) -> Result<String, String> {
    let content=bisect_runner::config::render_template_config(&bisect_runner::config::TemplateConfig {
        name:name.to_owned(),structure:structure.to_owned(),weights:weights.map(str::to_owned),alpha_county,
        search:search.map(str::to_owned),convergence_threshold,seeds,balance_tolerance,years:years.to_vec(),workers,
        engine:None,seed:None,
    })?;

    if dry_run {
        return Ok(content);
    }

    let out_path = output_path.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        let configs_dir = std::path::PathBuf::from("configs");
        configs_dir.join(format!("{name}.yml"))
    });

    // Create the configs/ directory if needed
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "[CONFIG] config new: cannot create directory '{}': {e}",
                    parent.display()
                )
            })?;
        }
    }

    if out_path.exists() && !force {
        return Err(format!(
            "[CONFIG] config new: '{}' already exists. Use --force to overwrite.",
            out_path.display()
        ));
    }

    std::fs::write(&out_path, content.as_bytes()).map_err(|e| {
        format!(
            "[CONFIG] config new: cannot write '{}': {e}",
            out_path.display()
        )
    })?;

    Ok(format!("Written: {}", out_path.display()))
}

// ---------------------------------------------------------------------------
// `BISECT config validate` — load and validate without running
// ---------------------------------------------------------------------------

/// Load and validate a config YAML file.
///
/// Prints a summary of the resolved algorithm configuration if valid.
/// Returns `[CONFIG]` error for parse or validation failures.
/// Reject a build config that is not court-defensible. ASCII-only messages.
pub fn validate_config_court(path: &Path) -> Result<(), String> {
    let yaml = AlgoYaml::from_file(path)?;
    yaml.validate_court(&path.display().to_string())
}

pub fn validate_config(path: &Path) -> Result<String, String> {
    let yaml=AlgoYaml::from_file(path)?;
    yaml.validation_summary(&path.display().to_string(), &AlgoYaml::file_sha256(path)?)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_yaml(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    // ── Test 1: valid YAML parses correctly ───────────────────────────────────

    #[test]
    fn test_valid_yaml_parses_correctly() {
        let yaml = r#"
name: official_proposal
description: "Test run"
algorithm:
  structure: apportion-regions
  weights: county
  alpha_county: 2.0
  search: convergence
  convergence_threshold: 600
  balance_tolerance: 0.5
workers: 6
years: ["2020", "2010", "2000"]
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).expect("valid YAML must parse");
        assert_eq!(doc.name, "official_proposal");
        assert_eq!(doc.workers, Some(6));
        assert_eq!(doc.years.as_ref().unwrap().len(), 3);
        assert_eq!(doc.algorithm.structure, "apportion-regions");
        assert_eq!(doc.algorithm.alpha_county, Some(2.0));
        assert_eq!(doc.algorithm.convergence_threshold, Some(600));
    }

    // ── Test 2: structure "apportion-regions" → SplitStrategy::ApportionRegions ─

    #[test]
    fn test_structure_apportion_regions_maps_correctly() {
        use crate::runner::SplitStrategy;
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        assert!(
            matches!(algo.split, SplitStrategy::ApportionRegions),
            "apportion-regions must map to SplitStrategy::ApportionRegions, got: {:?}",
            algo.split
        );
    }

    #[test]
    fn test_fixed_seed_maps_to_metis_params() {
        let yaml = r#"
name: reference
algorithm:
  structure: standard-bisect
  weights: geographic
  search: single
  seed: 424242
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        assert_eq!(doc.algorithm.seed, Some(424242));
        assert_eq!(algo.metis.seed, Some(424242));
    }

    #[test]
    fn test_structure_spectral_maps_correctly() {
        use crate::runner::SplitStrategy;
        let yaml = r#"
name: test
algorithm:
  structure: spectral
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        assert!(
            matches!(algo.split, SplitStrategy::Spectral { max_iters: 200 }),
            "spectral must map to SplitStrategy::Spectral, got: {:?}",
            algo.split
        );
    }

    #[test]
    fn test_structure_regionalization_maps_correctly() {
        use crate::runner::SplitStrategy;
        let yaml = r#"
name: test
algorithm:
  structure: regionalization
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        assert!(
            matches!(algo.split, SplitStrategy::Regionalization),
            "regionalization must map to SplitStrategy::Regionalization, got: {:?}",
            algo.split
        );
    }

    #[test]
    fn test_structure_flow_construction_maps_correctly() {
        use crate::runner::SplitStrategy;
        let yaml = r#"
name: test
algorithm:
  structure: flow-construction
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        assert!(
            matches!(algo.split, SplitStrategy::FlowConstruction),
            "flow-construction must map to SplitStrategy::FlowConstruction, got: {:?}",
            algo.split
        );
    }

    // ── Test 3: weights "county" with alpha_county → WeightSpec.alpha_county set ─

    #[test]
    fn test_weights_county_sets_alpha_county() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  weights: county
  alpha_county: 3.5
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        assert!(
            (algo.weights.alpha_county - 3.5).abs() < 1e-9,
            "alpha_county must be 3.5, got: {}",
            algo.weights.alpha_county
        );
        assert!(
            algo.weights.geographic,
            "county weights must enable geographic"
        );
    }

    // ── Test 4: search "convergence" with threshold → SeedCompositor::ConvergenceSweep{600} ─

    #[test]
    fn test_search_convergence_maps_to_convergence_sweep() {
        use crate::runner::SeedCompositor;
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  search: convergence
  convergence_threshold: 600
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        match &algo.seeds {
            SeedCompositor::ConvergenceSweep { threshold } => {
                assert_eq!(*threshold, 600, "threshold must be 600");
            }
            other => panic!("expected ConvergenceSweep, got: {:?}", other),
        }
    }

    // ── Test 5: unknown structure → [CONFIG] error ────────────────────────────

    #[test]
    fn test_unknown_structure_returns_config_error() {
        let yaml = r#"
name: test
algorithm:
  structure: flying-saucer
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let err = doc.to_algorithm_config().unwrap_err();
        assert!(
            err.contains("[CONFIG]"),
            "error must start with [CONFIG]: {err}"
        );
        assert!(
            err.contains("flying-saucer"),
            "error must mention the unknown value: {err}"
        );
    }

    // ── Test 6: missing required fields → error ───────────────────────────────

    #[test]
    fn test_missing_structure_field_returns_error() {
        // `structure` is required — omitting it should cause a parse error
        let yaml = r#"
name: test
algorithm:
  search: single
"#;
        let f = write_yaml(yaml);
        let result = AlgoYaml::from_file(f.path());
        assert!(
            result.is_err(),
            "missing 'structure' field must produce an error"
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("[CONFIG]"),
            "error must start with [CONFIG]: {err}"
        );
    }

    // ── Test 7: unknown top-level key → [CONFIG] error ────────────────────────

    #[test]
    fn test_unknown_top_level_field_returns_error() {
        let yaml = r#"
name: test
weigths: county
algorithm:
  structure: apportion-regions
  search: single
"#;
        let f = write_yaml(yaml);
        let result = AlgoYaml::from_file(f.path());
        assert!(
            result.is_err(),
            "unknown field 'weigths' must produce an error"
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("[CONFIG]"),
            "error must start with [CONFIG]: {err}"
        );
    }

    // ── Test 8: SHA-256 of file content is 64-char hex ────────────────────────

    #[test]
    fn test_file_sha256_is_64_char_hex() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  search: single
"#;
        let f = write_yaml(yaml);
        let sha = AlgoYaml::file_sha256(f.path()).unwrap();
        assert_eq!(sha.len(), 64, "SHA-256 must be 64 hex chars, got: {sha}");
        assert!(
            sha.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA-256 must be lowercase hex: {sha}"
        );
    }

    // ── Test 9: SHA-256 is deterministic ─────────────────────────────────────

    #[test]
    fn test_file_sha256_is_deterministic() {
        let yaml = "name: test\nalgorithm:\n  structure: apportion-regions\n  search: single\n";
        let f1 = write_yaml(yaml);
        let f2 = write_yaml(yaml);
        let sha1 = AlgoYaml::file_sha256(f1.path()).unwrap();
        let sha2 = AlgoYaml::file_sha256(f2.path()).unwrap();
        assert_eq!(sha1, sha2, "same content must produce same SHA-256");
    }

    // ── Test 10: search "multi" with seeds → SeedCompositor::Multi ───────────

    #[test]
    fn test_search_multi_with_seeds_maps_correctly() {
        use crate::runner::SeedCompositor;
        let yaml = r#"
name: test
algorithm:
  structure: geosection
  search: multi
  seeds: 100
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        match &algo.seeds {
            SeedCompositor::Multi { seeds } => {
                assert_eq!(*seeds, 100, "seeds must be 100");
            }
            other => panic!("expected Multi, got: {:?}", other),
        }
    }

    // ── Test 11: search "single" → SeedCompositor::Single ────────────────────

    #[test]
    fn test_search_single_maps_to_single() {
        use crate::runner::SeedCompositor;
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        assert!(
            matches!(algo.seeds, SeedCompositor::Single),
            "search: single must map to SeedCompositor::Single"
        );
    }

    // ── Test 12: convergence without threshold → [CONFIG] error ──────────────

    #[test]
    fn test_convergence_without_threshold_returns_config_error() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  search: convergence
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let err = doc.to_algorithm_config().unwrap_err();
        assert!(
            err.contains("[CONFIG]"),
            "missing convergence_threshold must produce [CONFIG] error: {err}"
        );
        assert!(
            err.contains("convergence_threshold"),
            "error must mention 'convergence_threshold': {err}"
        );
    }

    // ── Test 13: county weights without alpha_county → [CONFIG] error ─────────

    #[test]
    fn test_county_weights_without_alpha_returns_config_error() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  weights: county
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let err = doc.to_algorithm_config().unwrap_err();
        assert!(
            err.contains("[CONFIG]"),
            "missing alpha_county must produce [CONFIG] error: {err}"
        );
        assert!(
            err.contains("alpha_county"),
            "error must mention 'alpha_county': {err}"
        );
    }

    // ── Test 14: unknown weights → [CONFIG] error ─────────────────────────────

    #[test]
    fn test_unknown_weights_returns_config_error() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  weights: turbo-charged
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let err = doc.to_algorithm_config().unwrap_err();
        assert!(
            err.contains("[CONFIG]"),
            "unknown weights must produce [CONFIG] error: {err}"
        );
        assert!(
            err.contains("turbo-charged"),
            "error must mention the unknown value: {err}"
        );
    }

    // ── Test 15: resolved_workers defaults to 4 ───────────────────────────────

    #[test]
    fn test_resolved_workers_defaults_to_4() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        assert_eq!(doc.resolved_workers(), 4, "default workers must be 4");
    }

    // ── Test 16: resolved_years defaults to all three census years ─────────────

    #[test]
    fn test_resolved_years_defaults_to_three_census_years() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let years = doc.resolved_years();
        assert_eq!(
            years,
            vec!["2020", "2010", "2000"],
            "default years must be 2020/2010/2000"
        );
    }

    // ── Test 17: ratio-optimal maps to GeoSection ─────────────────────────────

    #[test]
    fn test_ratio_optimal_maps_to_geosection() {
        use crate::runner::SplitStrategy;
        let yaml = r#"
name: test
algorithm:
  structure: ratio-optimal
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        assert!(
            matches!(algo.split, SplitStrategy::GeoSection),
            "ratio-optimal must map to SplitStrategy::GeoSection"
        );
    }

    // ── Test 18: geographic weights (default) enables geographic flag ──────────

    #[test]
    fn test_geographic_weights_sets_geographic_flag() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  weights: geographic
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        assert!(
            algo.weights.geographic,
            "geographic weights must set geographic flag"
        );
        assert_eq!(
            algo.weights.alpha_county, 0.0,
            "geographic weights must leave alpha_county at 0"
        );
    }

    // ── Test 19: write_template_config dry_run returns YAML text ──────────────

    #[test]
    fn test_write_template_config_dry_run_returns_yaml() {
        let years = vec!["2020".to_string(), "2010".to_string()];
        let result = write_template_config(
            "my_plan",
            "apportion-regions",
            Some("county"),
            Some(2.0),
            Some("convergence"),
            Some(600),
            None,
            Some(0.5),
            &years,
            6,
            None,
            false,
            true, // dry_run
        )
        .unwrap();
        assert!(
            result.contains("name: my_plan"),
            "YAML must contain name: {result}"
        );
        assert!(
            result.contains("apportion-regions"),
            "YAML must contain structure: {result}"
        );
        assert!(
            result.contains("county"),
            "YAML must contain weights: {result}"
        );
        assert!(
            result.contains("alpha_county: 2"),
            "YAML must contain alpha_county: {result}"
        );
        assert!(
            result.contains("convergence_threshold: 600"),
            "YAML must contain threshold: {result}"
        );
    }

    // ── Test 20: validate_config returns [OK] for valid file ──────────────────

    #[test]
    fn test_validate_config_returns_ok_for_valid_yaml() {
        let yaml = r#"
name: official_proposal
algorithm:
  structure: apportion-regions
  weights: county
  alpha_county: 2.0
  search: convergence
  convergence_threshold: 600
workers: 6
years: ["2020", "2010", "2000"]
"#;
        let f = write_yaml(yaml);
        let result = validate_config(f.path()).unwrap();
        assert!(
            result.starts_with("[OK]"),
            "validate_config must return [OK] for valid YAML: {result}"
        );
        assert!(
            result.contains("apportion-regions"),
            "output must mention structure: {result}"
        );
        assert!(
            result.contains("convergence"),
            "output must mention search: {result}"
        );
    }

    // ── Test 21: validate_config returns error for invalid YAML ───────────────

    #[test]
    fn test_validate_config_returns_error_for_invalid_yaml() {
        let yaml = r#"
name: test
algorithm:
  structure: flying-saucer
  search: single
"#;
        let f = write_yaml(yaml);
        let result = validate_config(f.path());
        assert!(
            result.is_err(),
            "validate_config must fail for unknown structure"
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("[CONFIG]"),
            "error must contain [CONFIG]: {err}"
        );
    }

    // ── Test 22: structure "standard-bisect" maps to SplitStrategy::Bisect ──

    #[test]
    fn test_structure_standard_bisect_maps_correctly() {
        use crate::runner::SplitStrategy;
        let yaml = r#"
name: test
algorithm:
  structure: standard-bisect
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        assert!(
            matches!(algo.split, SplitStrategy::Bisect),
            "standard-bisect must map to SplitStrategy::Bisect, got: {:?}",
            algo.split
        );
    }

    // ── Test 23: structure "nway" maps to SplitStrategy::NWay ───────────────

    #[test]
    fn test_structure_nway_maps_correctly() {
        use crate::runner::SplitStrategy;
        let yaml = r#"
name: test
algorithm:
  structure: nway
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        assert!(
            matches!(algo.split, SplitStrategy::NWay),
            "nway must map to SplitStrategy::NWay, got: {:?}",
            algo.split
        );
    }

    // ── Test 24: structure "ratio-optimal-area" uses default area_swing 1.10 ─

    #[test]
    fn test_structure_ratio_optimal_area_default_area_swing() {
        use crate::runner::SplitStrategy;
        let yaml = r#"
name: test
algorithm:
  structure: ratio-optimal-area
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        match algo.split {
            SplitStrategy::AreaSection { area_swing, .. } => {
                assert!(
                    (area_swing - 1.10).abs() < 1e-9,
                    "default area_swing must be 1.10, got: {area_swing}"
                );
            }
            other => panic!("expected AreaSection, got: {:?}", other),
        }
    }

    // ── Test 25: structure "ratio-optimal-vra" uses default w_vra 0.40 ───────

    #[test]
    fn test_structure_ratio_optimal_vra_default_w_vra() {
        use crate::runner::SplitStrategy;
        let yaml = r#"
name: test
algorithm:
  structure: ratio-optimal-vra
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        match algo.split {
            SplitStrategy::VraSection { w_vra } => {
                assert!(
                    (w_vra - 0.40).abs() < 1e-9,
                    "default w_vra must be 0.40, got: {w_vra}"
                );
            }
            other => panic!("expected VraSection, got: {:?}", other),
        }
    }

    // ── Test 26: structure "compact-polsby" maps to CompactBisect ────────────

    #[test]
    fn test_structure_compact_polsby_maps_correctly() {
        use crate::runner::SplitStrategy;
        let yaml = r#"
name: test
algorithm:
  structure: compact-polsby
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        assert!(
            matches!(algo.split, SplitStrategy::CompactBisect { .. }),
            "compact-polsby must map to SplitStrategy::CompactBisect, got: {:?}",
            algo.split
        );
    }

    // ── Test 27: weights "unweighted" → geographic=false, alpha=0.0 ──────────

    #[test]
    fn test_weights_unweighted_disables_geographic() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  weights: unweighted
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        assert!(
            !algo.weights.geographic,
            "unweighted must set geographic=false"
        );
        assert_eq!(
            algo.weights.alpha_county, 0.0,
            "unweighted must have alpha_county=0.0"
        );
    }

    // ── Test 28: weights "vra-aligned" sets minority_weighting=true ──────────

    #[test]
    fn test_weights_vra_aligned_sets_minority_weighting() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  weights: vra-aligned
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        assert!(
            algo.weights.minority_weighting,
            "vra-aligned must set minority_weighting=true"
        );
        assert!(
            algo.weights.geographic,
            "vra-aligned must also set geographic=true"
        );
    }

    // ── Test 29: search "multi" with no seeds → defaults to 50 ───────────────

    #[test]
    fn test_search_multi_without_seeds_defaults_to_50() {
        use crate::runner::SeedCompositor;
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  search: multi
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        match &algo.seeds {
            SeedCompositor::Multi { seeds } => {
                assert_eq!(*seeds, 50, "default seeds for multi must be 50");
            }
            other => panic!("expected Multi, got: {:?}", other),
        }
    }

    // ── Test 30: unknown search → [CONFIG] error names the bad value ──────────

    #[test]
    fn test_unknown_search_error_names_bad_value() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  search: turbo-search
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let err = doc.to_algorithm_config().unwrap_err();
        assert!(err.contains("[CONFIG]"), "[CONFIG] prefix required: {err}");
        assert!(
            err.contains("turbo-search"),
            "error must name the bad value: {err}"
        );
    }

    // ── Test 31: resolved_workers respects explicit value ─────────────────────

    #[test]
    fn test_resolved_workers_respects_explicit_value() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  search: single
workers: 12
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        assert_eq!(
            doc.resolved_workers(),
            12,
            "explicit workers value must be respected"
        );
    }

    // ── Test 32: resolved_years respects single-year config ───────────────────

    #[test]
    fn test_resolved_years_single_year() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  search: single
years: ["2020"]
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let years = doc.resolved_years();
        assert_eq!(
            years,
            vec!["2020"],
            "single-year config must resolve to [2020]"
        );
    }

    // ── Test 33: resolved_analysis_types defaults to compactness/splits/summary

    #[test]
    fn test_resolved_analysis_types_defaults() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let types = doc.resolved_analysis_types();
        assert!(
            types.contains(&"compactness".to_string()),
            "default types must include compactness: {types:?}"
        );
        assert!(
            types.contains(&"splits".to_string()),
            "default types must include splits: {types:?}"
        );
        assert!(
            types.contains(&"summary".to_string()),
            "default types must include summary: {types:?}"
        );
    }

    // ── Test 34: resolved_analysis_types respects config value ────────────────

    #[test]
    fn test_resolved_analysis_types_respects_config() {
        let yaml = r#"
name: test
algorithm:
  structure: apportion-regions
  search: single
analysis_types: [demographic, summary]
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let types = doc.resolved_analysis_types();
        assert_eq!(
            types,
            vec!["demographic", "summary"],
            "configured analysis_types must be returned: {types:?}"
        );
    }

    // ── Test 35: file_sha256 on missing file → [CONFIG] error ─────────────────

    #[test]
    fn test_file_sha256_missing_file_returns_config_error() {
        let result = AlgoYaml::file_sha256(std::path::Path::new("/nonexistent/config_xyz.yml"));
        assert!(result.is_err(), "sha256 of missing file must fail");
        let err = result.unwrap_err();
        assert!(err.contains("[CONFIG]"), "[CONFIG] prefix required: {err}");
    }

    // ── Test 36: from_file on missing file → [CONFIG] error ──────────────────

    #[test]
    fn test_from_file_missing_file_returns_config_error() {
        let result = AlgoYaml::from_file(std::path::Path::new("/nonexistent/config_xyz.yml"));
        assert!(result.is_err(), "loading missing file must fail");
        let err = result.unwrap_err();
        assert!(err.contains("[CONFIG]"), "[CONFIG] prefix required: {err}");
    }

    // ── Test 37: description field is optional and preserved ─────────────────

    #[test]
    fn test_description_field_is_optional_and_preserved() {
        let yaml = r#"
name: test
description: "A detailed test plan for redistricting."
algorithm:
  structure: apportion-regions
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        assert_eq!(
            doc.description.as_deref(),
            Some("A detailed test plan for redistricting."),
            "description must be preserved when present"
        );
    }

    // ── Test 38: area_swing explicit value is used ────────────────────────────

    #[test]
    fn test_structure_area_section_custom_area_swing() {
        use crate::runner::SplitStrategy;
        let yaml = r#"
name: test
algorithm:
  structure: ratio-optimal-area
  area_swing: 1.25
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        match algo.split {
            SplitStrategy::AreaSection { area_swing, .. } => {
                assert!(
                    (area_swing - 1.25).abs() < 1e-9,
                    "explicit area_swing 1.25 must be used, got: {area_swing}"
                );
            }
            other => panic!("expected AreaSection, got: {:?}", other),
        }
    }

    // ── Test 39: w_vra explicit value is used ────────────────────────────────

    #[test]
    fn test_structure_vra_section_custom_w_vra() {
        use crate::runner::SplitStrategy;
        let yaml = r#"
name: test
algorithm:
  structure: ratio-optimal-vra
  w_vra: 0.60
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        match algo.split {
            SplitStrategy::VraSection { w_vra } => {
                assert!(
                    (w_vra - 0.60).abs() < 1e-9,
                    "explicit w_vra 0.60 must be used, got: {w_vra}"
                );
            }
            other => panic!("expected VraSection, got: {:?}", other),
        }
    }

    // ── Test 40: write_template_config: force overwrites existing file ────────

    #[test]
    fn test_write_template_config_force_overwrites() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let out_path = tmp.path().join("my_plan.yml");

        // Write a first time
        let years = vec!["2020".to_string()];
        write_template_config(
            "my_plan",
            "apportion-regions",
            None,
            None,
            Some("single"),
            None,
            None,
            None,
            &years,
            4,
            Some(&out_path),
            false,
            false,
        )
        .unwrap();
        assert!(out_path.exists(), "file must be written on first call");

        // Write a second time without force — must fail
        let result = write_template_config(
            "my_plan",
            "apportion-regions",
            None,
            None,
            Some("single"),
            None,
            None,
            None,
            &years,
            4,
            Some(&out_path),
            false,
            false,
        );
        assert!(result.is_err(), "second write without force must fail");
        let msg = result.unwrap_err();
        assert!(msg.contains("[CONFIG]"), "[CONFIG] prefix required: {msg}");
        assert!(msg.contains("--force"), "error must mention --force: {msg}");

        // Write a third time WITH force — must succeed
        let result = write_template_config(
            "my_plan",
            "apportion-regions",
            None,
            None,
            Some("single"),
            None,
            None,
            None,
            &years,
            4,
            Some(&out_path),
            true,
            false,
        );
        assert!(
            result.is_ok(),
            "write with force must succeed: {:?}",
            result
        );
    }

    // ── Test 41: write_template_config with invalid structure → error ─────────

    #[test]
    fn test_write_template_config_invalid_structure_returns_error() {
        let years = vec!["2020".to_string()];
        let result = write_template_config(
            "my_plan",
            "flying-saucer",
            None,
            None,
            Some("single"),
            None,
            None,
            None,
            &years,
            4,
            None,
            false,
            true, // dry_run
        );
        assert!(
            result.is_err(),
            "invalid structure must fail config generation"
        );
        let msg = result.unwrap_err();
        assert!(msg.contains("[CONFIG]"), "[CONFIG] prefix required: {msg}");
    }

    // ── Test 42: balance_tolerance default 0.5 appears in template ───────────

    #[test]
    fn test_write_template_config_default_balance_tolerance() {
        let years = vec!["2020".to_string()];
        let content = write_template_config(
            "my_plan",
            "apportion-regions",
            None,
            None,
            Some("single"),
            None,
            None,
            None, // balance_tolerance=None → default 0.5
            &years,
            4,
            None,
            false,
            true, // dry_run
        )
        .unwrap();
        assert!(
            content.contains("balance_tolerance: 0.5"),
            "default balance_tolerance 0.5 must appear in template: {content}"
        );
    }
}
