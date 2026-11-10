//! Shared, closed YAML schema for native and browser algorithm projects.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct AlgoYaml {
    /// Must match the label used in `bisect build` commands.
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
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct AlgorithmSection {
    /// Layer 1: bisection structure.
    /// Values: standard-bisect | nway | ratio-optimal | ratio-optimal-area |
    ///         ratio-optimal-vra | prime-factor | compact-polsby | apportion-regions
    pub structure: String,

    /// Layer 2: edge weight mode.
    /// Values: unweighted | geographic | county | vra-aligned | proportional
    pub weights: Option<String>,

    /// County stickiness factor (T.3). Required when weights == "county".
    pub alpha_county: Option<f64>,

    /// Layer 3: seed search strategy.
    /// Values: single | multi | convergence | short-burst
    pub search: Option<String>,

    /// Consecutive non-improving seeds before stopping (B.7/U.1).
    /// Required when search == "convergence".
    pub convergence_threshold: Option<u32>,

    /// Fixed seed count for search == "multi".
    pub seeds: Option<usize>,

    /// METIS random seed. Required by profiles that claim assignment-level
    /// reproducibility across independent executions.
    pub seed: Option<u64>,

    /// ReCom steps per burst for search == "short-burst". Default: 20.
    pub burst_length: Option<usize>,

    /// Number of bursts for search == "short-burst". Default: 50.
    pub n_bursts: Option<usize>,

    /// Percentile for search == "short-burst" (0.0–1.0). Default: 0.0 (minimum EC).
    pub percentile: Option<f64>,

    /// Population balance tolerance (percent). Congressional standard: 0.5.
    pub balance_tolerance: Option<f64>,

    /// Area imbalance multiplier for AreaSection. Default 1.10; each side
    /// may contain at most 0.5 * area_swing of total area at the root cut.
    pub area_swing: Option<f64>,

    /// VRA alignment weight for ratio-optimal-vra (VRASection). Default 0.40.
    pub w_vra: Option<f64>,

    /// METIS backend engine.
    /// Values: c-ffi (default) | metis-core | gpmetis
    /// c-ffi: links libmetis (battle-tested, all k values).
    /// metis-core: pure Rust, no C dependency, portable standalone binary.
    /// gpmetis: external subprocess (reserved, not yet implemented).
    pub engine: Option<String>,

    /// MKA: number of sweep orientations (default: 180).
    /// Only used when structure == "moving-knife".
    pub mka_orientations: Option<usize>,

    /// MKA: compactness metric (default: "reock").
    /// Values: reock | polsby-popper
    /// Only used when structure == "moving-knife".
    pub mka_metric: Option<String>,
}

impl AlgoYaml {
    /// Explicit YAML percentages become runner fractions. Omission delegates to
    /// the state/chamber policy; it must not manufacture a congressional default.
    pub fn run_balance_tolerance(&self) -> Result<Option<f64>, String> {
        self.algorithm.balance_tolerance.map(|percent| {
            if !percent.is_finite() || !(0.01..=50.0).contains(&percent) {
                return Err("[CONFIG] balance_tolerance must be a finite percent in [0.01, 50].".into());
            }
            Ok(percent / 100.0)
        }).transpose()
    }

    pub fn from_text(text: &str) -> Result<Self, String> {
        serde_yaml::from_str(text).map_err(|e| format!("[CONFIG] config: {e}"))
    }
    pub fn from_file(path: &std::path::Path) -> Result<Self, String> {
        let bytes = std::fs::read(path)
            .map_err(|e| format!("[CONFIG] config: cannot read '{}': {e}", path.display()))?;
        let text = std::str::from_utf8(&bytes).map_err(|e| {
            format!(
                "[CONFIG] config: '{}' is not valid UTF-8: {e}",
                path.display()
            )
        })?;
        Self::from_text(text)
    }
    pub fn file_sha256(path: &std::path::Path) -> Result<String, String> {
        use sha2::{Digest, Sha256};
        let bytes = std::fs::read(path).map_err(|e| {
            format!(
                "[CONFIG] config: cannot read '{}' for hashing: {e}",
                path.display()
            )
        })?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }
    /// Return the resolved workers count (config value or default 4).
    pub fn resolved_workers(&self) -> usize {
        self.workers.unwrap_or(4)
    }

    /// Return the resolved years list (config value or default ["2020", "2010", "2000"]).
    pub fn resolved_years(&self) -> Vec<String> {
        self.years
            .clone()
            .unwrap_or_else(|| vec!["2020".to_string(), "2010".to_string(), "2000".to_string()])
    }

    /// Return the resolved analysis types (config value or default).
    pub fn resolved_analysis_types(&self) -> Vec<String> {
        self.analysis_types.clone().unwrap_or_else(|| {
            vec![
                "compactness".to_string(),
                "splits".to_string(),
                "summary".to_string(),
            ]
        })
    }
}

/// Native YAML compositor resolution. Browser inspection does not execute this configuration.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResolvedAlgorithm {
    pub split: ResolvedSplit,
    pub weights: ResolvedWeights,
    pub vertex_constraints: Vec<ResolvedVertex>,
    pub seeds: ResolvedSearch,
    pub metis: ResolvedMetis,
    pub mode_label: Option<String>,
}
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ResolvedSplit {
    Bisect,
    NWay,
    GeoSection,
    AreaSection {
        area_swing: f64,
        area_section_init: ResolvedAreaInit,
    },
    VraSection {
        w_vra: f64,
    },
    ApportionRegions,
    CompactBisect {
        epsilon: f64,
    },
    MovingKnife {
        n_orientations: usize,
        metric: String,
    },
    Spectral {
        max_iters: usize,
    },
    Regionalization,
    FlowConstruction,
}
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResolvedAreaInit {
    RatioOptimal,
}
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResolvedVertex {
    Population,
    Area,
}
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ResolvedSearch {
    Single,
    Multi {
        seeds: usize,
    },
    ConvergenceSweep {
        threshold: u32,
    },
    ShortBurst {
        burst_length: usize,
        n_bursts: usize,
        p: f64,
    },
    ShortBurstForest {
        burst_length: usize,
        n_bursts: usize,
        p: f64,
    },
    ShortBurstMergeSplit {
        burst_length: usize,
        n_bursts: usize,
        p: f64,
    },
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResolvedWeights {
    pub geographic: bool,
    pub minority_weighting: bool,
    pub alpha_county: f64,
}
impl Default for ResolvedWeights {
    fn default() -> Self {
        Self {
            geographic: true,
            minority_weighting: false,
            alpha_county: 0.,
        }
    }
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResolvedMetis {
    pub engine: ResolvedEngine,
    pub seed: Option<u64>,
    pub ufactor: u32,
    pub niter: u32,
}
impl Default for ResolvedMetis {
    fn default() -> Self {
        Self {
            engine: ResolvedEngine::CFfi,
            seed: None,
            ufactor: 5,
            niter: 100,
        }
    }
}
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResolvedEngine {
    CFfi,
    RedistMetis,
    Gpmetis,
}

impl AlgoYaml {
    pub fn resolve(&self) -> Result<ResolvedAlgorithm, String> {
        let sec = &self.algorithm;

        // ── Layer 1: structure ─────────────────────────────────────────────────
        let (split, vertex_constraints) = match sec.structure.as_str() {
            "standard-bisect" => (ResolvedSplit::Bisect, vec![ResolvedVertex::Population]),
            "nway" => (ResolvedSplit::NWay, vec![ResolvedVertex::Population]),
            "ratio-optimal" | "geosection" => {
                (ResolvedSplit::GeoSection, vec![ResolvedVertex::Population])
            }
            "ratio-optimal-area" | "areasection" => {
                let area_swing = sec.area_swing.unwrap_or(1.10);
                if !area_swing.is_finite() || !(1.0..=2.0).contains(&area_swing) {
                    return Err("[CONFIG] area_swing must be a finite multiplier in [1, 2], e.g. 1.10 for a 45%-55% root area split; fractional margins such as 0.10 are not multipliers.".into());
                }
                (
                    ResolvedSplit::AreaSection {
                        area_swing,
                        area_section_init: ResolvedAreaInit::RatioOptimal,
                    },
                    vec![ResolvedVertex::Population, ResolvedVertex::Area],
                )
            }
            "ratio-optimal-vra" | "vra-section" => {
                let w_vra = sec.w_vra.unwrap_or(0.40);
                (
                    ResolvedSplit::VraSection { w_vra },
                    vec![ResolvedVertex::Population],
                )
            }
            "prime-factor" | "apportion-regions" => (
                ResolvedSplit::ApportionRegions,
                vec![ResolvedVertex::Population],
            ),
            "compact-polsby" | "compact-bisect" => {
                let epsilon = 0.01; // default epsilon for CompactBisect
                (
                    ResolvedSplit::CompactBisect { epsilon },
                    vec![ResolvedVertex::Population],
                )
            }
            "moving-knife" => {
                let n_orientations = sec.mka_orientations.unwrap_or(180);
                let metric = sec
                    .mka_metric
                    .clone()
                    .unwrap_or_else(|| "reock".to_string());
                (
                    ResolvedSplit::MovingKnife {
                        n_orientations,
                        metric,
                    },
                    vec![ResolvedVertex::Population],
                )
            }
            "spectral" => (
                ResolvedSplit::Spectral { max_iters: 200 },
                vec![ResolvedVertex::Population],
            ),
            "regionalization" => (
                ResolvedSplit::Regionalization,
                vec![ResolvedVertex::Population],
            ),
            "flow-construction" => (
                ResolvedSplit::FlowConstruction,
                vec![ResolvedVertex::Population],
            ),
            other => {
                return Err(format!(
                    "[CONFIG] config: unknown structure '{}'. \
                     Valid values: standard-bisect | nway | ratio-optimal | \
                     ratio-optimal-area | ratio-optimal-vra | prime-factor | \
                     compact-polsby | apportion-regions | moving-knife | \
                     spectral | regionalization | flow-construction",
                    other
                ));
            }
        };

        // ── Layer 2: weights ───────────────────────────────────────────────────
        let weights = {
            let weight_str = sec.weights.as_deref().unwrap_or("geographic");
            match weight_str {
                "unweighted" => ResolvedWeights {
                    geographic: false,
                    alpha_county: 0.0,
                    ..ResolvedWeights::default()
                },
                "geographic" => ResolvedWeights::default(),
                "county" => {
                    let alpha = sec.alpha_county.ok_or_else(|| {
                        "[CONFIG] config: 'alpha_county' is required when weights: county"
                            .to_string()
                    })?;
                    ResolvedWeights {
                        geographic: true,
                        alpha_county: alpha,
                        ..ResolvedWeights::default()
                    }
                }
                "vra-aligned" => ResolvedWeights {
                    geographic: true,
                    minority_weighting: true,
                    ..ResolvedWeights::default()
                },
                "proportional" => ResolvedWeights {
                    geographic: true,
                    ..ResolvedWeights::default()
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
            "single" => ResolvedSearch::Single,
            "multi" => {
                let n = sec.seeds.unwrap_or(50);
                ResolvedSearch::Multi { seeds: n }
            }
            "convergence" => {
                let threshold = sec.convergence_threshold.ok_or_else(|| {
                    "[CONFIG] config: 'convergence_threshold' is required when search: convergence"
                        .to_string()
                })?;
                ResolvedSearch::ConvergenceSweep { threshold }
            }
            "short-burst" => {
                let burst_length = sec.burst_length.unwrap_or(20);
                let n_bursts = sec.n_bursts.unwrap_or(50);
                let p = sec.percentile.unwrap_or(0.0).clamp(0.0, 1.0);
                ResolvedSearch::ShortBurst {
                    burst_length,
                    n_bursts,
                    p,
                }
            }
            "short-burst-forest" => {
                let burst_length = sec.burst_length.unwrap_or(20);
                let n_bursts = sec.n_bursts.unwrap_or(50);
                let p = sec.percentile.unwrap_or(0.0).clamp(0.0, 1.0);
                ResolvedSearch::ShortBurstForest {
                    burst_length,
                    n_bursts,
                    p,
                }
            }
            "short-burst-merge-split" => {
                let burst_length = sec.burst_length.unwrap_or(20);
                let n_bursts = sec.n_bursts.unwrap_or(50);
                let p = sec.percentile.unwrap_or(0.0).clamp(0.0, 1.0);
                ResolvedSearch::ShortBurstMergeSplit {
                    burst_length,
                    n_bursts,
                    p,
                }
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
            "c-ffi" | "c" | "metis-rs" => ResolvedEngine::CFfi,
            "metis-core" | "rust" => ResolvedEngine::RedistMetis,
            "gpmetis" | "subprocess" => ResolvedEngine::Gpmetis,
            other => {
                return Err(format!(
                    "[CONFIG] config: unknown engine '{}'. \
                     Valid values: c-ffi | metis-core | gpmetis",
                    other
                ));
            }
        };

        Ok(ResolvedAlgorithm {
            split,
            weights,
            vertex_constraints,
            seeds,
            metis: ResolvedMetis {
                engine,
                seed: sec.seed,
                ..ResolvedMetis::default()
            },
            mode_label: None,
        })
    }
}

impl AlgoYaml {
    pub fn validate_court(&self, source_label: &str) -> Result<(), String> {
        let yaml = self;
        let a = &yaml.algorithm;
        let s = a.structure.to_lowercase();
        let w = a.weights.as_deref().unwrap_or("").to_lowercase();
        let sr = a.search.as_deref().unwrap_or("single").to_lowercase();
        if s.contains("proportional") || w == "proportional" || w == "partisan" {
            return Err(format!(
                "profile=court rejects config '{}': partisan/proportional algorithm. \
             Use a neutral config or --profile open.",
                source_label
            ));
        }
        if !matches!(sr.as_str(), "single" | "convergence") {
            return Err(format!(
                "profile=court rejects config '{}': search '{sr}' is not deterministic. \
             Use single or convergence, or --profile open.",
                source_label
            ));
        }
        Ok(())
    }
}

impl ResolvedSplit {
    pub fn mode_name(&self) -> &'static str {
        match self {
            Self::Bisect => "edge-weighted",
            Self::NWay => "metis-vra",
            Self::GeoSection => "geosection",
            Self::AreaSection { .. } => "areasection",
            Self::VraSection { .. } => "vra-section",
            Self::ApportionRegions => "apportion-regions",
            Self::CompactBisect { .. } => "compact-bisect",
            Self::MovingKnife { .. } => "moving-knife",
            Self::Spectral { .. } => "spectral",
            Self::Regionalization => "regionalization",
            Self::FlowConstruction => "flow-construction",
        }
    }
}
impl AlgoYaml {
    pub fn validation_summary(&self, source_label: &str, sha: &str) -> Result<String, String> {
        let algo = self.resolve()?;
        let structure_desc = algo.split.mode_name();
        let weight_desc = if algo.weights.alpha_county > 0. {
            format!("county (alpha={:.1})", algo.weights.alpha_county)
        } else if algo.weights.minority_weighting {
            "vra-aligned".into()
        } else if algo.weights.geographic {
            "geographic".into()
        } else {
            "unweighted".into()
        };
        let search_desc = match algo.seeds {
            ResolvedSearch::Single => "single".to_string(),
            ResolvedSearch::Multi { seeds } => format!("multi (seeds={seeds})"),
            ResolvedSearch::ConvergenceSweep { threshold } => {
                format!("convergence (threshold={threshold})")
            }
            ResolvedSearch::ShortBurst {
                burst_length,
                n_bursts,
                p,
            } => format!("short-burst (p={p:.2}, bursts={n_bursts}, length={burst_length})"),
            ResolvedSearch::ShortBurstForest {
                burst_length,
                n_bursts,
                p,
            } => format!("short-burst-forest (p={p:.2}, bursts={n_bursts}, length={burst_length})"),
            ResolvedSearch::ShortBurstMergeSplit {
                burst_length,
                n_bursts,
                p,
            } => format!(
                "short-burst-merge-split (p={p:.2}, bursts={n_bursts}, length={burst_length})"
            ),
        };
        let years = self.resolved_years().join(", ");
        let workers = self.resolved_workers();

        let mut out = vec![
            format!("[OK] {}", source_label),
            format!("  name:      {}", self.name),
            format!("  structure: {structure_desc}"),
            format!("  weights:   {weight_desc}"),
            format!("  search:    {search_desc}"),
            format!("  years:     {years}"),
            format!("  workers:   {workers}"),
            format!("  sha256:    {sha}"),
        ];
        if let Some(desc) = &self.description {
            let trimmed = desc.trim();
            if !trimmed.is_empty() {
                out.push(format!(
                    "  description: {}",
                    trimmed.lines().next().unwrap_or("")
                ));
            }
        }
        Ok(out.join("\n"))
    }
}

/// Inputs for the native config.New template renderer. Filesystem writes belong to the caller.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateConfig {
    pub name: String,
    pub structure: String,
    pub weights: Option<String>,
    pub alpha_county: Option<f64>,
    pub search: Option<String>,
    pub convergence_threshold: Option<u32>,
    pub seeds: Option<usize>,
    pub balance_tolerance: Option<f64>,
    pub years: Vec<String>,
    pub workers: usize,
    #[serde(default)]
    pub engine: Option<String>,
    #[serde(default)]
    pub seed: Option<String>,
}
pub fn render_template_config(settings: &TemplateConfig) -> Result<String, String> {
    let name = settings.name.as_str();
    let structure = settings.structure.as_str();
    let weights = settings.weights.as_deref();
    let search = settings.search.as_deref();
    let alpha_county = settings.alpha_county;
    let convergence_threshold = settings.convergence_threshold;
    let seeds = settings.seeds;
    let balance_tolerance = settings.balance_tolerance;
    let years = &settings.years;
    let workers = settings.workers;
    let engine = settings.engine.as_deref();
    if name.is_empty()
        || name.len() > 100
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
    {
        return Err("[CONFIG] config new: name must be a plain label of 1-100 ASCII letters, digits, underscores or hyphens".into());
    }
    if years.is_empty()
        || years.len() > 100
        || years
            .iter()
            .any(|y| y.len() != 4 || !y.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err("[CONFIG] config new: supply 1-100 four-digit years".into());
    }
    if [alpha_county, balance_tolerance]
        .into_iter()
        .flatten()
        .any(|n| !n.is_finite())
    {
        return Err("[CONFIG] config new: numeric settings must be finite".into());
    }
    let seed = match &settings.seed {
        None => None,
        Some(s) => {
            if s.is_empty()
                || (s.len() > 1 && s.starts_with('0'))
                || !s.bytes().all(|b| b.is_ascii_digit())
            {
                return Err(
                    "[CONFIG] config new: seed must be a canonical decimal u64 string".into(),
                );
            }
            Some(
                s.parse::<u64>()
                    .map_err(|_| "[CONFIG] config new: seed exceeds u64")?,
            )
        }
    };
    // Validate inputs by doing a dry-run through to_algorithm_config
    let section = AlgorithmSection {
        structure: structure.to_string(),
        weights: weights.map(|s| s.to_string()),
        alpha_county,
        search: search.map(|s| s.to_string()),
        convergence_threshold,
        seeds,
        seed,
        burst_length: None,
        n_bursts: None,
        percentile: None,
        balance_tolerance,
        area_swing: None,
        w_vra: None,
        engine: engine.map(str::to_owned),
        mka_orientations: None,
        mka_metric: None,
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
    yaml_doc.resolve()?;

    // Build YAML text
    let weights_str = weights.unwrap_or("geographic");
    let search_str = search.unwrap_or("single");
    let years_str = years
        .iter()
        .map(|y| format!("\"{}\"", y))
        .collect::<Vec<_>>()
        .join(", ");

    let mut lines = vec![
        format!("# configs/{name}.yml"),
        "# User-editable. Commit this. Do not confuse with .bisect (auto-managed).".to_string(),
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

    if let Some(engine) = engine {
        lines.push(format!("  engine: {engine}"));
    }
    if let Some(seed) = seed {
        lines.push(format!("  seed: {seed}"));
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

    Ok(content)
}

/// Apply explicit Run form seed/tolerance overrides without dropping any native compositor option.
pub fn run_form_yaml(text:&str,seed:Option<u64>,tolerance:Option<f64>)->Result<String,String>{
    let mut yaml=AlgoYaml::from_text(text)?;
    if let Some(seed)=seed{yaml.algorithm.seed=Some(seed);}
    if let Some(tolerance)=tolerance{yaml.algorithm.balance_tolerance=Some(tolerance);}
    yaml.resolve()?;yaml.run_balance_tolerance()?;
    serde_yaml::to_string(&yaml).map_err(|e|e.to_string())
}

#[cfg(test)]
mod run_form_tests {
    use super::*;
    #[test]
    fn optional_form_overrides_preserve_all_other_native_fields() {
        let text = "name: retained\ndescription: original\nworkers: 2\nyears: [2020]\nanalysis_types: [compactness, summary]\nalgorithm:\n  structure: standard-bisect\n  weights: county\n  alpha_county: 7.5\n  engine: metis-core\n  search: short-burst\n  seed: 9\n  balance_tolerance: 3\n  burst_length: 11\n  n_bursts: 13\n  percentile: 0.25\n  area_swing: 1.1\n  w_vra: 0.4\n  mka_orientations: 90\n  mka_metric: reock\n";
        let original: serde_yaml::Value = serde_yaml::from_str(&run_form_yaml(text, None, None).unwrap()).unwrap();
        let mut expected = original.clone();
        expected["algorithm"]["seed"] = serde_yaml::to_value(u64::MAX).unwrap();
        expected["algorithm"]["balance_tolerance"] = serde_yaml::to_value(0.5_f64).unwrap();
        let overridden: serde_yaml::Value = serde_yaml::from_str(&run_form_yaml(text, Some(u64::MAX), Some(0.5)).unwrap()).unwrap();
        assert_eq!(overridden, expected);
        assert_eq!(original["algorithm"]["seed"].as_u64(), Some(9));
        assert!(run_form_yaml(text, None, Some(f64::NAN)).is_err());
        assert!(run_form_yaml(text, None, Some(-1.0)).is_err());
    }
}
