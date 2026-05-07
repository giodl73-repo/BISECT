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
///   balance_tolerance: 0.5
/// workers: 6
/// years: ["2020", "2010", "2000"]
/// analysis_types: [compactness, splits, summary]
/// ```
use std::path::Path;
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Public structs
// ---------------------------------------------------------------------------

/// Top-level YAML document for a label's algorithm configuration.
///
/// This is the committed, human-editable file in `configs/X.yml`.
/// Separate from `.redist` (auto-managed registry) and `runs/X/index.json`
/// (auto-managed build index).
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlgoYaml {
    /// Must match the label used in `redist build` commands.
    pub name: String,
    /// Free-text description (optional, for human readers and audit reports).
    pub description: Option<String>,
    /// Three-layer algorithm compositor parameters.
    pub algorithm: AlgorithmSection,
    /// Parallel workers per year (default: 4).
    pub workers: Option<usize>,
    /// Census years to build (default: ["2020", "2010", "2000"]).
    pub years: Option<Vec<String>>,
    /// Analysis types to run after each build year.
    /// Values: demographic | political | compactness | contiguity | splits | summary
    pub analysis_types: Option<Vec<String>>,
}

/// The `algorithm:` block in the YAML.
///
/// All fields are `Option<_>` except `structure` which is required.
/// Conditional requirements (e.g. `alpha_county` when `weights: county`) are
/// enforced in `to_algorithm_config`.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlgorithmSection {
    /// Layer 1: bisection structure.
    /// Values: standard-bisect | nway | ratio-optimal | ratio-optimal-area |
    ///         ratio-optimal-vra | prime-factor | compact-polsby | apportion-regions
    pub structure: String,

    /// Layer 2: edge weight mode.
    /// Values: unweighted | geographic | county | vra-aligned | proportional
    pub weights: Option<String>,

    /// County stickiness factor (B.10). Required when weights == "county".
    pub alpha_county: Option<f64>,

    /// Layer 3: seed search strategy.
    /// Values: single | multi | convergence | short-burst
    pub search: Option<String>,

    /// Consecutive non-improving seeds before stopping (B.7/B.16).
    /// Required when search == "convergence".
    pub convergence_threshold: Option<u32>,

    /// Fixed seed count for search == "multi".
    pub seeds: Option<usize>,

    /// ReCom steps per burst for search == "short-burst". Default: 20.
    pub burst_length: Option<usize>,

    /// Number of bursts for search == "short-burst". Default: 50.
    pub n_bursts: Option<usize>,

    /// Percentile for search == "short-burst" (0.0–1.0). Default: 0.0 (minimum EC).
    pub percentile: Option<f64>,

    /// Population balance tolerance (percent). Congressional standard: 0.5.
    pub balance_tolerance: Option<f64>,

    /// Area swing for ratio-optimal-area (AreaSection). Default 0.10.
    pub area_swing: Option<f64>,

    /// VRA alignment weight for ratio-optimal-vra (VRASection). Default 0.40.
    pub w_vra: Option<f64>,

    /// METIS backend engine.
    /// Values: c-ffi (default) | metis-core | gpmetis
    /// c-ffi: links libmetis (battle-tested, all k values).
    /// metis-core: pure Rust, no C dependency, portable standalone binary.
    /// gpmetis: external subprocess (reserved, not yet implemented).
    pub engine: Option<String>,
}

// ---------------------------------------------------------------------------
// impl AlgoYaml
// ---------------------------------------------------------------------------

impl AlgoYaml {
    /// Load from a YAML file path. Returns `[CONFIG]` error on parse failure.
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| {
            format!("[CONFIG] config: cannot read '{}': {e}", path.display())
        })?;
        let text = std::str::from_utf8(&bytes).map_err(|e| {
            format!("[CONFIG] config: '{}' is not valid UTF-8: {e}", path.display())
        })?;
        serde_yaml::from_str::<AlgoYaml>(text).map_err(|e| {
            // serde_yaml error messages already mention the field name for unknown-field
            // errors; wrap with [CONFIG] prefix to match the project error convention.
            format!("[CONFIG] config: {}", e)
        })
    }

    /// Convert to `AlgorithmConfig` using the three-layer compositor logic.
    ///
    /// Maps structure/weights/search strings to SplitStrategy/WeightSpec/SeedCompositor.
    /// Returns `[CONFIG]` error for unknown values or missing conditional fields.
    pub fn to_algorithm_config(&self) -> Result<crate::runner::AlgorithmConfig, String> {
        use crate::runner::{AlgorithmConfig, SeedCompositor, SplitStrategy, WeightSpec, MetisParams};
        use crate::vertex_weights::VertexConstraintKind;

        let sec = &self.algorithm;

        // ── Layer 1: structure ─────────────────────────────────────────────────
        let (split, vertex_constraints) = match sec.structure.as_str() {
            "standard-bisect" => (
                SplitStrategy::Bisect,
                vec![VertexConstraintKind::Population],
            ),
            "nway" => (
                SplitStrategy::NWay,
                vec![VertexConstraintKind::Population],
            ),
            "ratio-optimal" | "geosection" => (
                SplitStrategy::GeoSection,
                vec![VertexConstraintKind::Population],
            ),
            "ratio-optimal-area" | "areasection" => {
                let area_swing = sec.area_swing.unwrap_or(0.10);
                (
                    SplitStrategy::AreaSection { area_swing },
                    vec![VertexConstraintKind::Population, VertexConstraintKind::Area],
                )
            }
            "ratio-optimal-vra" | "vra-section" => {
                let w_vra = sec.w_vra.unwrap_or(0.40);
                (
                    SplitStrategy::VraSection { w_vra },
                    vec![VertexConstraintKind::Population],
                )
            }
            "prime-factor" | "apportion-regions" => (
                SplitStrategy::ApportionRegions,
                vec![VertexConstraintKind::Population],
            ),
            "compact-polsby" | "compact-bisect" => {
                let epsilon = 0.01; // default epsilon for CompactBisect
                (
                    SplitStrategy::CompactBisect { epsilon },
                    vec![VertexConstraintKind::Population],
                )
            }
            other => {
                return Err(format!(
                    "[CONFIG] config: unknown structure '{}'. \
                     Valid values: standard-bisect | nway | ratio-optimal | \
                     ratio-optimal-area | ratio-optimal-vra | prime-factor | \
                     compact-polsby | apportion-regions",
                    other
                ));
            }
        };

        // ── Layer 2: weights ───────────────────────────────────────────────────
        let weights = {
            let weight_str = sec.weights.as_deref().unwrap_or("geographic");
            match weight_str {
                "unweighted" => WeightSpec {
                    geographic: false,
                    alpha_county: 0.0,
                    ..WeightSpec::default()
                },
                "geographic" => WeightSpec::default(),
                "county" => {
                    let alpha = sec.alpha_county.ok_or_else(|| {
                        "[CONFIG] config: 'alpha_county' is required when weights: county".to_string()
                    })?;
                    WeightSpec {
                        geographic: true,
                        alpha_county: alpha,
                        ..WeightSpec::default()
                    }
                }
                "vra-aligned" => WeightSpec {
                    geographic: true,
                    minority_weighting: true,
                    ..WeightSpec::default()
                },
                "proportional" => WeightSpec {
                    geographic: true,
                    ..WeightSpec::default()
                },
                other => {
                    return Err(format!(
                        "[CONFIG] config: unknown weights '{}'. \
                         Valid values: unweighted | geographic | county | vra-aligned | proportional",
                        other
                    ));
                }
            }
        };

        // ── Layer 3: seeds ─────────────────────────────────────────────────────
        let seeds = match sec.search.as_deref().unwrap_or("single") {
            "single" => SeedCompositor::Single,
            "multi" => {
                let n = sec.seeds.unwrap_or(50);
                SeedCompositor::Multi { seeds: n }
            }
            "convergence" => {
                let threshold = sec.convergence_threshold.ok_or_else(|| {
                    "[CONFIG] config: 'convergence_threshold' is required when search: convergence"
                        .to_string()
                })?;
                SeedCompositor::ConvergenceSweep { threshold }
            }
            "short-burst" => {
                let burst_length = sec.burst_length.unwrap_or(20);
                let n_bursts = sec.n_bursts.unwrap_or(50);
                let p = sec.percentile.unwrap_or(0.0).clamp(0.0, 1.0);
                SeedCompositor::ShortBurst { burst_length, n_bursts, p }
            }
            "short-burst-forest" => {
                let burst_length = sec.burst_length.unwrap_or(20);
                let n_bursts = sec.n_bursts.unwrap_or(50);
                let p = sec.percentile.unwrap_or(0.0).clamp(0.0, 1.0);
                SeedCompositor::ShortBurstForest { burst_length, n_bursts, p }
            }
            "short-burst-merge-split" => {
                let burst_length = sec.burst_length.unwrap_or(20);
                let n_bursts = sec.n_bursts.unwrap_or(50);
                let p = sec.percentile.unwrap_or(0.0).clamp(0.0, 1.0);
                SeedCompositor::ShortBurstMergeSplit { burst_length, n_bursts, p }
            }
            other => {
                return Err(format!(
                    "[CONFIG] config: unknown search '{}'. \
                     Valid values: single | multi | convergence | short-burst | \
                     short-burst-forest | short-burst-merge-split",
                    other
                ));
            }
        };

        // ── Engine selection ───────────────────────────────────────────────────
        let engine = match sec.engine.as_deref().unwrap_or("c-ffi") {
            "c-ffi" | "c" | "metis-rs" => redist_apportion::split::MetisEngine::CFfi,
            "metis-core" | "rust"      => redist_apportion::split::MetisEngine::RedistMetis,
            "gpmetis" | "subprocess"   => redist_apportion::split::MetisEngine::Gpmetis,
            other => {
                return Err(format!(
                    "[CONFIG] config: unknown engine '{}'. \
                     Valid values: c-ffi | metis-core | gpmetis",
                    other
                ));
            }
        };

        Ok(AlgorithmConfig {
            split,
            weights,
            vertex_constraints,
            seeds,
            metis: MetisParams { engine, ..MetisParams::default() },
            mode_label: None,
        })
    }

    /// Compute SHA-256 of the raw file bytes. Used for the provenance chain.
    ///
    /// Returns a 64-character lowercase hex string.
    pub fn file_sha256(path: &Path) -> Result<String, String> {
        let bytes = std::fs::read(path).map_err(|e| {
            format!("[CONFIG] config: cannot read '{}' for hashing: {e}", path.display())
        })?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// Return the resolved workers count (config value or default 4).
    pub fn resolved_workers(&self) -> usize {
        self.workers.unwrap_or(4)
    }

    /// Return the resolved years list (config value or default ["2020", "2010", "2000"]).
    pub fn resolved_years(&self) -> Vec<String> {
        self.years.clone().unwrap_or_else(|| {
            vec!["2020".to_string(), "2010".to_string(), "2000".to_string()]
        })
    }

    /// Return the resolved analysis types (config value or default).
    pub fn resolved_analysis_types(&self) -> Vec<String> {
        self.analysis_types.clone().unwrap_or_else(|| {
            vec!["compactness".to_string(), "splits".to_string(), "summary".to_string()]
        })
    }
}

// ---------------------------------------------------------------------------
// `redist config new` — write a template YAML to configs/X.yml
// ---------------------------------------------------------------------------

/// Write a template config YAML file to `configs/{name}.yml`.
///
/// Called by `redist config new --name X ...`.
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
    // Validate inputs by doing a dry-run through to_algorithm_config
    let section = AlgorithmSection {
        structure: structure.to_string(),
        weights: weights.map(|s| s.to_string()),
        alpha_county,
        search: search.map(|s| s.to_string()),
        convergence_threshold,
        seeds,
        burst_length: None,
        n_bursts: None,
        percentile: None,
        balance_tolerance,
        area_swing: None,
        w_vra: None,
        engine: None,
    };
    let yaml_doc = AlgoYaml {
        name: name.to_string(),
        description: None,
        algorithm: section,
        workers: Some(workers),
        years: Some(years.to_vec()),
        analysis_types: None,
    };
    // Validate that the config would produce a valid AlgorithmConfig.
    yaml_doc.to_algorithm_config()?;

    // Build YAML text
    let weights_str = weights.unwrap_or("geographic");
    let search_str = search.unwrap_or("single");
    let years_str = years.iter()
        .map(|y| format!("\"{}\"", y))
        .collect::<Vec<_>>()
        .join(", ");

    let mut lines = vec![
        format!("# configs/{name}.yml"),
        "# User-editable. Commit this. Do not confuse with .redist (auto-managed).".to_string(),
        String::new(),
        format!("name: {name}"),
        "description: >".to_string(),
        format!("  Run configuration for '{name}'."),
        String::new(),
        "algorithm:".to_string(),
        format!("  structure: {structure}"),
        format!("  weights: {weights_str}"),
    ];

    if let Some(alpha) = alpha_county {
        lines.push(format!("  alpha_county: {alpha}"));
    }

    lines.push(format!("  search: {search_str}"));

    if let Some(t) = convergence_threshold {
        lines.push(format!("  convergence_threshold: {t}"));
    }
    if let Some(n) = seeds {
        lines.push(format!("  seeds: {n}"));
    }

    let tol = balance_tolerance.unwrap_or(0.5);
    lines.push(format!("  balance_tolerance: {tol}"));

    lines.push(String::new());
    lines.push(format!("workers: {workers}"));
    lines.push(format!("years: [{years_str}]"));
    lines.push("analysis_types: [compactness, splits, summary]".to_string());
    lines.push(String::new());

    let content = lines.join("\n");

    if dry_run {
        return Ok(content);
    }

    let out_path = output_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            let configs_dir = std::path::PathBuf::from("configs");
            configs_dir.join(format!("{name}.yml"))
        });

    // Create the configs/ directory if needed
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("[CONFIG] config new: cannot create directory '{}': {e}", parent.display())
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
        format!("[CONFIG] config new: cannot write '{}': {e}", out_path.display())
    })?;

    Ok(format!("Written: {}", out_path.display()))
}

// ---------------------------------------------------------------------------
// `redist config validate` — load and validate without running
// ---------------------------------------------------------------------------

/// Load and validate a config YAML file.
///
/// Prints a summary of the resolved algorithm configuration if valid.
/// Returns `[CONFIG]` error for parse or validation failures.
pub fn validate_config(path: &Path) -> Result<String, String> {
    let yaml = AlgoYaml::from_file(path)?;
    let algo = yaml.to_algorithm_config()?;

    let sha = AlgoYaml::file_sha256(path)?;

    let structure_desc = algo.split.mode_name();
    let weight_desc = if algo.weights.alpha_county > 0.0 {
        format!("county (alpha={:.1})", algo.weights.alpha_county)
    } else if algo.weights.minority_weighting {
        "vra-aligned".to_string()
    } else if algo.weights.geographic {
        "geographic".to_string()
    } else {
        "unweighted".to_string()
    };
    let search_desc = match &algo.seeds {
        crate::runner::SeedCompositor::Single => "single".to_string(),
        crate::runner::SeedCompositor::Multi { seeds } => format!("multi (seeds={seeds})"),
        crate::runner::SeedCompositor::ConvergenceSweep { threshold } => {
            format!("convergence (threshold={threshold})")
        }
        crate::runner::SeedCompositor::Percentile { p, seeds } => {
            format!("percentile (p={p:.2}, seeds={seeds})")
        }
        crate::runner::SeedCompositor::BisectionEnsemble { p, ensemble_steps } => {
            format!("bisection-ensemble (p={p:.2}, steps={ensemble_steps})")
        }
        crate::runner::SeedCompositor::Flip { flip_steps, p } => {
            format!("flip (p={p:.2}, steps={flip_steps})")
        }
        crate::runner::SeedCompositor::ShortBurst { burst_length, n_bursts, p } => {
            format!("short-burst (p={p:.2}, bursts={n_bursts}, length={burst_length})")
        }
        crate::runner::SeedCompositor::ShortBurstForest { burst_length, n_bursts, p } => {
            format!("short-burst-forest (p={p:.2}, bursts={n_bursts}, length={burst_length})")
        }
        crate::runner::SeedCompositor::ShortBurstMergeSplit { burst_length, n_bursts, p } => {
            format!("short-burst-merge-split (p={p:.2}, bursts={n_bursts}, length={burst_length})")
        }
        crate::runner::SeedCompositor::ForestRecom { steps, p } => {
            format!("forest-recom (p={p:.2}, steps={steps})")
        }
        crate::runner::SeedCompositor::MultiScale { total_steps, p, alpha } => {
            format!("multiscale (p={p:.2}, steps={total_steps}, alpha={alpha:.2})")
        }
        crate::runner::SeedCompositor::MergeSplit { steps, p } => {
            format!("merge-split (p={p:.2}, steps={steps})")
        }
        crate::runner::SeedCompositor::MultiScaleAdaptive { total_steps, p, target_accept, adapt_interval } => {
            format!("multiscale-adaptive (p={p:.2}, steps={total_steps}, target_accept={target_accept:.2}, adapt_interval={adapt_interval})")
        }
        crate::runner::SeedCompositor::ParallelTempering { n_replicas, swap_interval, cold_tolerance, hot_tolerance, steps, p } => {
            format!("parallel-tempering (p={p:.2}, steps={steps}, replicas={n_replicas}, swap_interval={swap_interval}, cold_tol={cold_tolerance:.4}, hot_tol={hot_tolerance:.4})")
        }
        crate::runner::SeedCompositor::VraRecom { steps, p, vap_threshold } => {
            format!("vra-recom (p={p:.2}, steps={steps}, vap_threshold={vap_threshold:.2})")
        }
    };

    let years = yaml.resolved_years().join(", ");
    let workers = yaml.resolved_workers();

    let mut out = vec![
        format!("[OK] {}", path.display()),
        format!("  name:      {}", yaml.name),
        format!("  structure: {structure_desc}"),
        format!("  weights:   {weight_desc}"),
        format!("  search:    {search_desc}"),
        format!("  years:     {years}"),
        format!("  workers:   {workers}"),
        format!("  sha256:    {sha}"),
    ];
    if let Some(desc) = &yaml.description {
        let trimmed = desc.trim();
        if !trimmed.is_empty() {
            out.push(format!("  description: {}", trimmed.lines().next().unwrap_or("")));
        }
    }
    Ok(out.join("\n"))
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
        assert_eq!(
            sha.len(),
            64,
            "SHA-256 must be 64 hex chars, got: {sha}"
        );
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
        assert_eq!(years, vec!["2020", "2010", "2000"], "default years must be 2020/2010/2000");
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
        assert!(algo.weights.geographic, "geographic weights must set geographic flag");
        assert_eq!(algo.weights.alpha_county, 0.0, "geographic weights must leave alpha_county at 0");
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
        ).unwrap();
        assert!(result.contains("name: my_plan"), "YAML must contain name: {result}");
        assert!(result.contains("apportion-regions"), "YAML must contain structure: {result}");
        assert!(result.contains("county"), "YAML must contain weights: {result}");
        assert!(result.contains("alpha_county: 2"), "YAML must contain alpha_county: {result}");
        assert!(result.contains("convergence_threshold: 600"), "YAML must contain threshold: {result}");
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
        assert!(result.starts_with("[OK]"), "validate_config must return [OK] for valid YAML: {result}");
        assert!(result.contains("apportion-regions"), "output must mention structure: {result}");
        assert!(result.contains("convergence"), "output must mention search: {result}");
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
        assert!(result.is_err(), "validate_config must fail for unknown structure");
        let err = result.unwrap_err();
        assert!(err.contains("[CONFIG]"), "error must contain [CONFIG]: {err}");
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
            "standard-bisect must map to SplitStrategy::Bisect, got: {:?}", algo.split
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
            "nway must map to SplitStrategy::NWay, got: {:?}", algo.split
        );
    }

    // ── Test 24: structure "ratio-optimal-area" uses default area_swing 0.10 ─

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
            SplitStrategy::AreaSection { area_swing } => {
                assert!(
                    (area_swing - 0.10).abs() < 1e-9,
                    "default area_swing must be 0.10, got: {area_swing}"
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
            "compact-polsby must map to SplitStrategy::CompactBisect, got: {:?}", algo.split
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
        assert!(!algo.weights.geographic, "unweighted must set geographic=false");
        assert_eq!(algo.weights.alpha_county, 0.0, "unweighted must have alpha_county=0.0");
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
        assert!(algo.weights.minority_weighting, "vra-aligned must set minority_weighting=true");
        assert!(algo.weights.geographic, "vra-aligned must also set geographic=true");
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
        assert!(err.contains("turbo-search"), "error must name the bad value: {err}");
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
        assert_eq!(doc.resolved_workers(), 12, "explicit workers value must be respected");
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
        assert_eq!(years, vec!["2020"], "single-year config must resolve to [2020]");
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
        assert!(types.contains(&"compactness".to_string()), "default types must include compactness: {types:?}");
        assert!(types.contains(&"splits".to_string()),      "default types must include splits: {types:?}");
        assert!(types.contains(&"summary".to_string()),     "default types must include summary: {types:?}");
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
        assert_eq!(types, vec!["demographic", "summary"],
            "configured analysis_types must be returned: {types:?}");
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
  area_swing: 0.25
  search: single
"#;
        let f = write_yaml(yaml);
        let doc = AlgoYaml::from_file(f.path()).unwrap();
        let algo = doc.to_algorithm_config().unwrap();
        match algo.split {
            SplitStrategy::AreaSection { area_swing } => {
                assert!(
                    (area_swing - 0.25).abs() < 1e-9,
                    "explicit area_swing 0.25 must be used, got: {area_swing}"
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
            "my_plan", "apportion-regions", None, None, Some("single"),
            None, None, None, &years, 4, Some(&out_path), false, false
        ).unwrap();
        assert!(out_path.exists(), "file must be written on first call");

        // Write a second time without force — must fail
        let result = write_template_config(
            "my_plan", "apportion-regions", None, None, Some("single"),
            None, None, None, &years, 4, Some(&out_path), false, false
        );
        assert!(result.is_err(), "second write without force must fail");
        let msg = result.unwrap_err();
        assert!(msg.contains("[CONFIG]"), "[CONFIG] prefix required: {msg}");
        assert!(msg.contains("--force"), "error must mention --force: {msg}");

        // Write a third time WITH force — must succeed
        let result = write_template_config(
            "my_plan", "apportion-regions", None, None, Some("single"),
            None, None, None, &years, 4, Some(&out_path), true, false
        );
        assert!(result.is_ok(), "write with force must succeed: {:?}", result);
    }

    // ── Test 41: write_template_config with invalid structure → error ─────────

    #[test]
    fn test_write_template_config_invalid_structure_returns_error() {
        let years = vec!["2020".to_string()];
        let result = write_template_config(
            "my_plan", "flying-saucer", None, None, Some("single"),
            None, None, None, &years, 4, None, false, true // dry_run
        );
        assert!(result.is_err(), "invalid structure must fail config generation");
        let msg = result.unwrap_err();
        assert!(msg.contains("[CONFIG]"), "[CONFIG] prefix required: {msg}");
    }

    // ── Test 42: balance_tolerance default 0.5 appears in template ───────────

    #[test]
    fn test_write_template_config_default_balance_tolerance() {
        let years = vec!["2020".to_string()];
        let content = write_template_config(
            "my_plan", "apportion-regions", None, None, Some("single"),
            None, None, None, // balance_tolerance=None → default 0.5
            &years, 4, None, false, true // dry_run
        ).unwrap();
        assert!(
            content.contains("balance_tolerance: 0.5"),
            "default balance_tolerance 0.5 must appear in template: {content}"
        );
    }
}
