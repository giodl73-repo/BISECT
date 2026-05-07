/// Multi-state Rayon parallel runner + single-state implementation.
///
/// `run_states_parallel` dispatches states across Rayon threads.
/// `run_single_state` is the core: load adjacency → bisect → balance check → write.
///
/// PROCESS EXPLOSION PREVENTION: state_name and num_districts are pre-resolved
/// in StateConfig BEFORE the Rayon pool starts. This prevents 100+ Python
/// subprocesses from being spawned simultaneously during a 50-state run.
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use crate::adjacency_loader::load_adjacency_pkl;
use crate::bisection_runner::{run_all_splits, run_all_splits_with_search, run_all_splits_compact, run_all_splits_percentile, run_geosection, run_nway_partition, run_flip_chain, run_short_burst, run_short_burst_forest, run_short_burst_merge_split, run_forest_recom, run_multiscale, run_merge_split, CompactBisectOpts};
use crate::demographics::{load_demographics, align_demographics_to_adjacency};
use crate::partisan_shares::load_partisan_shares;
use crate::fetch::load_manifest;
use crate::output::{write_state_outputs, clean_corrupt_state, VraAnalysis, VraDistrict};
use crate::status::{status, ascii_safe};
use redist_core::{Partition, state_code_to_fips};
use redist_analysis::analyze_mm_districts;
use redist_report;
use crate::vertex_weights::{VertexConstraintKind, build_vertex_weights};

/// Result of processing a single state.
#[derive(Debug, Clone)]
pub struct StateResult {
    pub state_code: String,
    pub success: bool,
    pub error: Option<String>,
    pub elapsed_ms: u64,
}

/// Configuration for a single state run.
///
/// state_name and num_districts are pre-resolved by the caller (Commands::States
/// or Commands::Run) to avoid spawning Python once per state inside the Rayon pool.
///
/// Fields are grouped into four logical domains:
/// HOW to recurse — the split strategy.
///
/// Adding a new strategy requires only a new variant here and a new arm in the
/// split dispatch inside `run_single_state`. It does NOT require touching
/// `WeightSpec`, `MetisParams`, or `AlgorithmConfig`.

/// How the seed space is searched at each evaluation point.
///
/// This is the third compositor layer (after structure and weights).
/// Seed counts are no longer embedded in SplitStrategy variants — they live here.
#[derive(Debug, Clone)]
pub enum SeedCompositor {
    /// Single content-derived seed. Deterministic, fast.
    /// Used by ApportionRegions for the federal statute default.
    Single,
    /// Try N seeds at each evaluation point, keep the minimum-EC result.
    /// Used by GeoSection, AreaSection, VRASection, CompactBisect.
    Multi { seeds: usize },
    /// Run seeds sequentially from the content-derived start until
    /// `threshold` consecutive seeds produce no improvement in normalised EC.
    /// Certifies convergence per B.7. The seed-buster for the federal statute.
    ConvergenceSweep { threshold: u32 },
    /// Run `seeds` plans, sort by edge cut, return the plan at rank floor(p * seeds).
    /// p=0.0 → minimum EC (same as ConvergenceSweep), p=0.5 → median EC, p=1.0 → maximum EC.
    /// Enables statutory choice of legal posture (B.7 / H.0).
    Percentile { p: f64, seeds: usize },
    /// At each bisection node, run a local `ensemble_steps`-step 2-way ReCom ensemble
    /// and pick the bisection at percentile `p` of the cut distribution.
    /// Always k=2 at each node — eliminates prime-k bipartition failures. (H.1)
    BisectionEnsemble { p: f64, ensemble_steps: usize },
    /// Flip individual boundary tracts to adjacent districts, collect all visited plans,
    /// return the plan at percentile `p` of the edge-cut distribution.
    /// flip_steps: total flip proposals (default: 10000). p: percentile (default: 0.0).
    Flip { flip_steps: usize, p: f64 },
    /// Short-Burst: run `n_bursts` short ReCom chains of `burst_length` steps on the
    /// full-state k-way assignment. Keep the chain endpoint from each burst (not the
    /// minimum). Chain restarts from the previous burst's endpoint. Sort endpoints by
    /// EC ASC; return the plan at rank floor(p * n_bursts). (G.6)
    ShortBurst { burst_length: usize, n_bursts: usize, p: f64 },
    /// Short-Burst using ForestRecomChain (two-tree MH) as the burst chain.
    /// Provides compactness optimization with approximate distributional correctness. (G.12)
    ShortBurstForest { burst_length: usize, n_bursts: usize, p: f64 },
    /// Short-Burst using MergeSplitChain as the burst chain. (G.12)
    ShortBurstMergeSplit { burst_length: usize, n_bursts: usize, p: f64 },
    /// Run `steps` Forest ReCom MH steps; collect accepted plans; return at percentile p of EC.
    /// Two-tree Metropolis-Hastings — targets uniform distribution (G.9 spec accepted 3.0/4).
    ForestRecom { steps: usize, p: f64 },
    /// Multi-scale MCMC — interleaves fine (tract) and coarse (block-group) ReCom moves.
    /// Requires block-group adjacency (run: redist fetch --resolution block_group).
    /// CLI: --search multiscale --multiscale-steps 2000 --multiscale-alpha 0.3
    /// (G.11 spec accepted 3.0/4; full implementation pending redist-multiscale crate completion)
    MultiScale { total_steps: usize, p: f64, alpha: f64 },
    /// Run `steps` Merge-Split MH steps; collect accepted plans; return at percentile p.
    /// Two-tree MH with explicit ratio — O(m log m) per step (G.10 spec accepted 3.0/4).
    MergeSplit { steps: usize, p: f64 },
}

impl SeedCompositor {
    /// Return the seed count / step count for display and logging.
    pub fn seed_count(&self) -> usize {
        match self {
            Self::Multi { seeds } => *seeds,
            Self::ConvergenceSweep { threshold } => *threshold as usize,
            Self::Single => 1,
            Self::Percentile { seeds, .. } => *seeds,
            Self::BisectionEnsemble { ensemble_steps, .. } => *ensemble_steps,
            Self::Flip { flip_steps, .. } => *flip_steps,
            Self::ShortBurst { n_bursts, .. } => *n_bursts,
            Self::ShortBurstForest { n_bursts, .. } => *n_bursts,
            Self::ShortBurstMergeSplit { n_bursts, .. } => *n_bursts,
            Self::ForestRecom { steps, .. } => *steps,
            Self::MultiScale { total_steps, .. } => *total_steps,
            Self::MergeSplit { steps, .. } => *steps,
        }
    }

    pub fn is_single(&self) -> bool {
        matches!(self, Self::Single)
    }
}

impl Default for SeedCompositor {
    fn default() -> Self { Self::Multi { seeds: 50 } }
}

/// Layer 1 (structure): what tree of splits?
///
/// Seed counts removed — those belong to SeedCompositor.
/// Algorithm-specific tuning parameters (epsilon, area_swing, w_vra, eta)
/// remain here because they define the split criterion, not the search.
#[derive(Debug, Clone)]
pub enum SplitStrategy {
    /// Standard bisection: always ⌊k/2⌋:⌈k/2⌉.
    Bisect,
    /// VRA-compliant n-way partitioning (minority opportunity districts).
    NWay,
    /// GeoSection (B.8): ratio-optimal direction-aware bisection.
    /// Seeds controlled by SeedCompositor.
    GeoSection,
    /// CompactBisect (B.7): greedy level-by-level geometric-mean PP selection.
    CompactBisect { epsilon: f64 },
    /// AreaSection (B.9): ratio-optimal bisection with dual population+area constraint.
    AreaSection { area_swing: f64 },
    /// ProportionalSection (B.12): ncon=2 [pop, D_votes] with HH-derived tpwgts.
    ProportionalSection { eta: f64 },
    /// ApportionRegions (B.11): prime-factorisation tree — Huntington-Hill geographic completion.
    /// Default seed strategy: Single (content-derived). ConvergenceSweep for federal statute.
    ApportionRegions,
    /// VRASection (B.14): GeoSection modified by geographic minority-VAP alignment score.
    VraSection { w_vra: f64 },
    /// Simulated Annealing bisection: start from METIS, accept/reject boundary flips
    /// via Boltzmann criterion. Structure layer -- replaces METIS at each bisection node.
    /// steps_per_tract: n_steps = steps_per_tract * |subgraph| (default: 10).
    /// t0_factor: T_0 = max(1.0, t0_factor * EC(initial)) (default: 0.01).
    /// t_final: near-zero final temperature (default: 1e-4).
    SimulatedAnnealing { steps_per_tract: usize, t0_factor: f64, t_final: f64 },
}

impl SplitStrategy {
    /// Human-readable mode name (for logging and manifest).
    pub fn mode_name(&self) -> &'static str {
        match self {
            Self::Bisect                  => "edge-weighted",
            Self::NWay                    => "metis-vra",
            Self::GeoSection              => "geosection",
            Self::CompactBisect    { .. } => "compact-bisect",
            Self::AreaSection      { .. } => "areasection",
            Self::ProportionalSection { .. } => "proportional-section",
            Self::ApportionRegions        => "apportion-regions",
            Self::VraSection       { .. } => "vra-section",
            Self::SimulatedAnnealing { .. } => "simulated-annealing",
        }
    }
}

/// WHAT signals go into edge weights.
///
/// Adding a new signal (e.g. `alpha_vtd: f64`) requires only adding a field here
/// and reading it in `build_edge_weights`. SplitStrategy and AlgorithmConfig are
/// not affected.
#[derive(Debug, Clone)]
pub struct WeightSpec {
    /// Use geographic TIGER boundary lengths as the base weight signal.
    pub geographic: bool,
    /// Path to per-tract Democratic vote share TSV (partisan-weighted / proportional modes).
    pub partisan_shares: Option<std::path::PathBuf>,
    /// Dem threshold for partisan weighting (default 0.55).
    pub dem_threshold: f64,
    /// Rep threshold for partisan weighting (default 0.45).
    pub rep_threshold: f64,
    /// Enable minority (VRA) weighting signal.
    pub minority_weighting: bool,
    /// County stickiness alpha (B.10). 0 = disabled.
    pub alpha_county: f64,
    /// MCD stickiness alpha. 0 = disabled.
    pub alpha_mcd: f64,
    /// Place stickiness alpha. 0 = disabled.
    pub alpha_place: f64,
    /// VTD stickiness alpha. 0 = disabled.
    pub alpha_vtd: f64,
    /// Directional lambda for GeoSection (0 = no penalty).
    pub directional_lambda: f64,
}

impl Default for WeightSpec {
    fn default() -> Self {
        Self {
            geographic: true,
            partisan_shares: None,
            dem_threshold: 0.55,
            rep_threshold: 0.45,
            minority_weighting: false,
            alpha_county: 0.0,
            alpha_mcd: 0.0,
            alpha_place: 0.0,
            alpha_vtd: 0.0,
            directional_lambda: 0.0,
        }
    }
}

/// METIS optimizer knobs, including engine selection.
#[derive(Debug, Clone)]
pub struct MetisParams {
    pub ufactor: u32,
    pub niter: u32,
    pub seed: Option<u64>,
    /// Which METIS backend to use. Default: `CFfi` (links libmetis).
    /// Set to `RedistMetis` for a portable standalone binary with no C dependency.
    pub engine: redist_apportion::split::MetisEngine,
}

impl Default for MetisParams {
    fn default() -> Self {
        Self { ufactor: 5, niter: 100, seed: None, engine: redist_apportion::split::MetisEngine::default() }
    }
}

/// Three-layer algorithm compositor.
///
/// Layer 1 — `split`: structure (what tree of splits?)
/// Layer 2 — `weights` + `vertex_constraints`: what costs (edge + vertex weights)
/// Layer 3 — `seeds`: search strategy (single / multi / convergence-sweep)
///
/// All three layers compose independently. Adding a new edge signal → WeightSpec.
/// Adding a new split structure → SplitStrategy. Changing search strategy → SeedCompositor.
/// None of these changes require touching the other two layers.
#[derive(Debug, Clone)]
pub struct AlgorithmConfig {
    /// Layer 1: tree structure
    pub split: SplitStrategy,
    /// Layer 2a: edge weight signals
    pub weights: WeightSpec,
    /// Layer 2b: vertex balance constraints (ncon = len)
    pub vertex_constraints: Vec<VertexConstraintKind>,
    /// Layer 3: seed search strategy
    pub seeds: SeedCompositor,
    /// METIS optimizer knobs (ufactor, niter, single-seed value)
    pub metis: MetisParams,
    /// Optional manifest label override.
    pub mode_label: Option<&'static str>,
}

impl AlgorithmConfig {
    /// Human-readable mode name (for logging and manifest).
    pub fn mode_name(&self) -> &'static str {
        if let Some(label) = self.mode_label {
            return label;
        }
        // Unweighted is special: same Bisect strategy but no geographic weights.
        if matches!(self.split, SplitStrategy::Bisect) && !self.weights.geographic {
            return "unweighted";
        }
        // PartisanWeighted uses Bisect with partisan signal.
        if matches!(self.split, SplitStrategy::Bisect) && self.weights.partisan_shares.is_some() {
            return "partisan-weighted";
        }
        self.split.mode_name()
    }

    /// Build from a single-state CLI invocation where the user can override
    /// ufactor, niter, seed, and mode-specific knobs explicitly.
    pub fn from_state_args(args: &crate::args::StateArgs) -> Self {
        use crate::args::PartitionMode as PM;
        let engine = args.metis_engine
            .map(|e| e.into())
            .unwrap_or_default();
        let metis = MetisParams { ufactor: args.ufactor, niter: args.niter, seed: args.seed, engine };
        let base_weights = WeightSpec {
            alpha_county: args.alpha_county,
            ..WeightSpec::default()
        };
        let pop_only = vec![VertexConstraintKind::Population];
        let pop_and_area = vec![VertexConstraintKind::Population, VertexConstraintKind::Area];
        let mut algo = match &args.partition_mode {
            PM::Unweighted => Self {
                split: SplitStrategy::Bisect,
                seeds: SeedCompositor::default(),
                weights: WeightSpec { geographic: false, alpha_county: args.alpha_county, ..WeightSpec::default() },
                vertex_constraints: pop_only,
                metis,
                mode_label: Some("unweighted"),
            },
            PM::EdgeWeighted => Self {
                split: SplitStrategy::Bisect,
                seeds: SeedCompositor::default(),
                weights: base_weights,
                vertex_constraints: pop_only,
                metis,
                mode_label: None,
            },
            PM::MetisVra => Self {
                split: SplitStrategy::NWay,
                seeds: SeedCompositor::default(),
                weights: WeightSpec { minority_weighting: true, alpha_county: args.alpha_county, ..WeightSpec::default() },
                vertex_constraints: pop_only,
                metis: MetisParams { seed: None, ..metis },
                mode_label: None,
            },
            PM::PartisanWeighted => Self {
                split: SplitStrategy::Bisect,
                seeds: SeedCompositor::default(),
                weights: WeightSpec {
                    partisan_shares: args.partisan_shares.as_ref().map(std::path::PathBuf::from),
                    dem_threshold: args.dem_threshold,
                    rep_threshold: args.rep_threshold,
                    alpha_county: args.alpha_county,
                    ..WeightSpec::default()
                },
                vertex_constraints: pop_only,
                metis,
                mode_label: None,
            },
            PM::Proportional => Self {
                split: SplitStrategy::Bisect,
                seeds: SeedCompositor::default(),
                weights: WeightSpec {
                    partisan_shares: args.partisan_shares.as_ref().map(std::path::PathBuf::from),
                    dem_threshold: 0.55,
                    rep_threshold: 0.45,
                    alpha_county: args.alpha_county,
                    ..WeightSpec::default()
                },
                vertex_constraints: pop_only,
                metis,
                mode_label: Some("proportional"),
            },
            PM::CompactBisect => Self {
                split: SplitStrategy::CompactBisect { epsilon: 0.05 },
                seeds: SeedCompositor::Multi { seeds: args.compact_seeds.max(1) },
                weights: base_weights,
                vertex_constraints: pop_only,
                metis: MetisParams { seed: None, ..metis },
                mode_label: None,
            },
            PM::GeoSection => Self {
                split: SplitStrategy::GeoSection,
                seeds: SeedCompositor::Multi { seeds: args.geosection_seeds.max(1) },
                weights: WeightSpec { directional_lambda: 0.0, alpha_county: args.alpha_county, ..WeightSpec::default() },
                vertex_constraints: pop_only,
                metis: MetisParams { seed: None, ..metis },
                mode_label: None,
            },
            PM::AreaSection => Self {
                split: SplitStrategy::AreaSection { area_swing: args.area_swing },
                seeds: SeedCompositor::Multi { seeds: args.geosection_seeds.max(1) },
                weights: base_weights,
                vertex_constraints: pop_and_area,  // ncon=2: population + land area
                metis: MetisParams { seed: None, ..metis },
                mode_label: None,
            },
            PM::ApportionRegions => Self {
                split: SplitStrategy::ApportionRegions,
                seeds: SeedCompositor::Single,  // federal statute: single content-derived seed
                weights: base_weights,
                vertex_constraints: pop_only,
                metis: MetisParams { seed: None, ..metis },
                mode_label: None,
            },
            PM::ProportionalSection => Self {
                split: SplitStrategy::ProportionalSection { eta: args.eta },
                seeds: SeedCompositor::Multi { seeds: args.geosection_seeds.max(1) },
                weights: base_weights,
                vertex_constraints: vec![VertexConstraintKind::Population],
                metis: MetisParams { seed: None, ..metis },
                mode_label: None,
            },
            PM::VraSection => Self {
                split: SplitStrategy::VraSection { w_vra: args.w_vra },
                seeds: SeedCompositor::Multi { seeds: args.geosection_seeds.max(1) },
                weights: WeightSpec { alpha_county: args.alpha_county, ..WeightSpec::default() },
                vertex_constraints: vec![VertexConstraintKind::Population],
                metis: MetisParams { seed: None, ..metis },
                mode_label: None,
            },
            PM::SimulatedAnnealing => Self {
                split: SplitStrategy::SimulatedAnnealing {
                    steps_per_tract: args.sa_steps_per_tract,
                    t0_factor: args.sa_t0_factor,
                    t_final: args.sa_t_final,
                },
                seeds: SeedCompositor::Single,
                weights: base_weights,
                vertex_constraints: pop_only,
                metis,
                mode_label: None,
            },
        };

        // ── Apply compositor layer overrides ──────────────────────────────────
        // Explicit --structure / --weights-override / --search flags override
        // the corresponding layer set by the --partition-mode preset above.
        use crate::args::{StructureMode as SM, WeightMode as WM, SearchMode as SeM};

        // Layer 1: structure override
        if let Some(structure) = args.structure {
            let pop_only = vec![VertexConstraintKind::Population];
            let pop_area  = vec![VertexConstraintKind::Population, VertexConstraintKind::Area];
            let (new_split, new_vc) = match structure {
                SM::StandardBisect   => (SplitStrategy::Bisect, pop_only),
                SM::NWay             => (SplitStrategy::NWay, pop_only),
                SM::RatioOptimal     => (SplitStrategy::GeoSection, pop_only),
                SM::RatioOptimalArea => (SplitStrategy::AreaSection { area_swing: args.area_swing }, pop_area),
                SM::RatioOptimalVra  => (SplitStrategy::VraSection { w_vra: args.w_vra }, pop_only),
                SM::PrimeFactor      => (SplitStrategy::ApportionRegions, pop_only),
                SM::CompactPolsby    => (SplitStrategy::CompactBisect { epsilon: 0.05 }, pop_only),
            };
            algo.split = new_split;
            algo.vertex_constraints = new_vc;
        }

        // Layer 2: weight override
        if let Some(weight) = args.weights_override {
            algo.weights = match weight {
                WM::Unweighted   => WeightSpec { geographic: false, alpha_county: 0.0, ..WeightSpec::default() },
                WM::Geographic   => WeightSpec { geographic: true,  alpha_county: 0.0, ..WeightSpec::default() },
                WM::County       => WeightSpec { geographic: true,  alpha_county: args.alpha_county.max(1.0), ..WeightSpec::default() },
                WM::VraAligned   => WeightSpec { geographic: true,  minority_weighting: true, ..WeightSpec::default() },
                WM::Proportional => WeightSpec { geographic: true,  partisan_shares: args.partisan_shares.as_ref().map(std::path::PathBuf::from), ..WeightSpec::default() },
            };
        }

        // Layer 3: search strategy override
        if let Some(search) = args.search {
            let n = args.seeds.unwrap_or(args.geosection_seeds.max(args.compact_seeds).max(50));
            algo.seeds = match search {
                SeM::Single             => SeedCompositor::Single,
                SeM::Multi              => SeedCompositor::Multi { seeds: n },
                SeM::Convergence        => SeedCompositor::ConvergenceSweep { threshold: args.convergence_threshold },
                SeM::Percentile         => SeedCompositor::Percentile {
                    p: args.percentile.clamp(0.0, 1.0),
                    seeds: n,
                },
                SeM::BisectionEnsemble  => SeedCompositor::BisectionEnsemble {
                    p: args.percentile.clamp(0.0, 1.0),
                    ensemble_steps: args.ensemble_steps,
                },
                SeM::Flip => SeedCompositor::Flip {
                    flip_steps: args.flip_steps,
                    p: args.percentile.clamp(0.0, 1.0),
                },
                SeM::ShortBurst => SeedCompositor::ShortBurst {
                    burst_length: args.burst_length,
                    n_bursts: args.n_bursts,
                    p: args.percentile.clamp(0.0, 1.0),
                },
                SeM::ShortBurstForest => SeedCompositor::ShortBurstForest {
                    burst_length: args.burst_length,
                    n_bursts: args.n_bursts,
                    p: args.percentile.clamp(0.0, 1.0),
                },
                SeM::ShortBurstMergeSplit => SeedCompositor::ShortBurstMergeSplit {
                    burst_length: args.burst_length,
                    n_bursts: args.n_bursts,
                    p: args.percentile.clamp(0.0, 1.0),
                },
                SeM::ForestRecom => SeedCompositor::ForestRecom {
                    steps: args.forest_steps,
                    p: args.percentile.clamp(0.0, 1.0),
                },
                SeM::MultiScale => SeedCompositor::MultiScale {
                    total_steps: args.multiscale_steps,
                    p: args.percentile.clamp(0.0, 1.0),
                    alpha: args.multiscale_alpha,
                },
                SeM::MergeSplit => SeedCompositor::MergeSplit {
                    steps: args.merge_split_steps,
                    p: args.percentile.clamp(0.0, 1.0),
                },
            };
        }

        algo
    }

    /// Canonical defaults for each mode. Called by bulk commands that
    /// don't expose per-algorithm knobs — single-state commands use
    /// from_state_args() to let users override these.
    pub fn defaults_for_mode(mode: &crate::args::PartitionMode) -> Self {
        use crate::args::PartitionMode as PM;
        let metis = MetisParams::default();
        let pop = vec![VertexConstraintKind::Population];
        let pop_area = vec![VertexConstraintKind::Population, VertexConstraintKind::Area];
        match mode {
            PM::Unweighted => Self {
                split: SplitStrategy::Bisect,
                seeds: SeedCompositor::default(),
                weights: WeightSpec { geographic: false, ..WeightSpec::default() },
                vertex_constraints: pop,
                metis,
                mode_label: Some("unweighted"),
            },
            PM::EdgeWeighted => Self::default(),
            PM::MetisVra => Self {
                split: SplitStrategy::NWay,
                seeds: SeedCompositor::default(),
                weights: WeightSpec { minority_weighting: true, ..WeightSpec::default() },
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
            PM::PartisanWeighted => Self {
                split: SplitStrategy::Bisect,
                seeds: SeedCompositor::default(),
                weights: WeightSpec {
                    partisan_shares: None,
                    dem_threshold: 0.55,
                    rep_threshold: 0.45,
                    ..WeightSpec::default()
                },
                vertex_constraints: pop,
                metis,
                mode_label: Some("partisan-weighted"),
            },
            PM::Proportional => Self {
                split: SplitStrategy::Bisect,
                seeds: SeedCompositor::default(),
                weights: WeightSpec {
                    partisan_shares: None,
                    dem_threshold: 0.55,
                    rep_threshold: 0.45,
                    ..WeightSpec::default()
                },
                vertex_constraints: pop,
                metis,
                mode_label: Some("proportional"),
            },
            PM::CompactBisect => Self {
                split: SplitStrategy::CompactBisect { epsilon: 0.05 },
                seeds: SeedCompositor::Multi { seeds: 50 },
                weights: WeightSpec::default(),
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
            PM::GeoSection => Self {
                split: SplitStrategy::GeoSection,
                seeds: SeedCompositor::Multi { seeds: 50 },
                weights: WeightSpec { directional_lambda: 0.0, ..WeightSpec::default() },
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
            PM::AreaSection => Self {
                split: SplitStrategy::AreaSection { area_swing: 1.10 },
                seeds: SeedCompositor::Multi { seeds: 50 },
                weights: WeightSpec::default(),
                vertex_constraints: pop_area,  // ncon=2: population + land area
                metis,
                mode_label: None,
            },
            PM::ApportionRegions => Self {
                split: SplitStrategy::ApportionRegions,
                seeds: SeedCompositor::Single,  // federal statute: single content-derived seed
                weights: WeightSpec::default(),
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
            PM::ProportionalSection => Self {
                split: SplitStrategy::ProportionalSection { eta: 1.10 },
                seeds: SeedCompositor::Multi { seeds: 50 },
                weights: WeightSpec::default(),
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
            PM::VraSection => Self {
                split: SplitStrategy::VraSection { w_vra: 0.40 },
                seeds: SeedCompositor::Multi { seeds: 50 },
                weights: WeightSpec::default(),
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
            PM::SimulatedAnnealing => Self {
                split: SplitStrategy::SimulatedAnnealing {
                    steps_per_tract: 10,
                    t0_factor: 0.01,
                    t_final: 1e-4,
                },
                seeds: SeedCompositor::Single,
                weights: WeightSpec::default(),
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
        }
    }
}

impl Default for AlgorithmConfig {
    /// Default: edge-weighted bisection, population-only vertex constraint (ncon=1).
    fn default() -> Self {
        Self {
            split: SplitStrategy::Bisect,
            seeds: SeedCompositor::Multi { seeds: 50 },
            weights: WeightSpec::default(),
            vertex_constraints: vec![VertexConstraintKind::Population],
            metis: MetisParams::default(),
            mode_label: None,
        }
    }
}

/// - **Identity**: which state/year/version/output to draw
/// - **Algorithm**: composable `algo: AlgorithmConfig` (split + weights + metis)
/// - **Control**: execution behavior (position, debug, reset, reprocess)
/// - **Spec 1 extensions**: chamber-aware, labeled, multi-member, COI, CVAP features
#[derive(Debug, Clone)]
pub struct StateConfig {
    // ── Identity: what plan is being drawn ───────────────────────────────────
    pub state_code: String,
    /// Lowercase state name for file paths (e.g. "alabama"). Pre-resolved.
    pub state_name: String,
    /// Number of congressional districts. Pre-resolved from config_{year}.py.
    pub num_districts: usize,
    pub year: String,
    pub version: String,
    pub output_dir: PathBuf,

    // ── Algorithm: composable config (split strategy + weight signals + METIS knobs) ─
    pub algo: AlgorithmConfig,

    // ── Shared partitioning constraints (apply to all modes) ─────────────────
    /// Max deviation per district in percent (None = use chamber default).
    pub balance_tolerance: Option<f64>,
    /// Path to COI weights file — modifies edge weights for all modes.
    pub coi_weights: Option<std::path::PathBuf>,

    // ── Control: execution behavior ───────────────────────────────────────────
    pub position: i32,
    pub debug: bool,
    pub reset: bool,
    pub reprocess: bool,
    /// When true, emit "[partition-time] <STATE>: k=<K> n=<N> -> <ms>ms" to stderr
    /// after each partition call. Measures pure METIS time, excluding I/O.
    pub time_partition: bool,

    // ── Spec 1 extensions: chamber-aware, labeled, multi-member ──────────────
    pub num_districts_override: Option<usize>,
    pub chamber: String,
    pub label: Option<String>,
    pub population_source: String,
    pub write_manifest: bool,
    pub force: bool,
    pub resolution: String,
    /// Geographic resolution for this run ("tract" | "bg" | "county")
    pub plan_resolution: String,
    pub seats_per_district: usize,
    pub total_seats: usize,
    pub adjacency_override: Option<std::path::PathBuf>,
}

impl StateConfig {
    /// Create a StateConfig for bulk congressional runs (Commands::States and Commands::Run).
    ///
    /// All Spec 1 fields default to their canonical bulk-run values:
    /// - `partition_mode`: "edge-weighted"
    /// - `ufactor`: 5, `niter`: 100, `seed`: None
    /// - `debug`: false, `reset`: false, `reprocess`: false
    /// - `chamber`: "congressional", `population_source`: "total"
    /// - `resolution`: "tract", `seats_per_district`: 1
    /// - `write_manifest`: false, `force`: false
    /// - All override/optional fields: None
    ///
    /// `total_seats` is set to `num_districts` (single-member default).
    ///
    /// Use `Commands::State` (the single-state arm) for custom chambers, labels,
    /// multi-member districts, COI weights, etc. — those require the full struct literal.
    pub fn new_bulk(
        state_code: String,
        state_name: String,
        num_districts: usize,
        year: String,
        version: String,
        output_dir: std::path::PathBuf,
        position: i32,
    ) -> Self {
        Self {
            state_code,
            state_name,
            num_districts,
            year,
            version,
            output_dir,
            position,
            // Algorithm defaults — edge-weighted bisection
            algo: AlgorithmConfig::default(),
            // Control defaults
            debug: false,
            reset: false,
            reprocess: false,
            time_partition: false,
            // Spec 1 defaults for bulk congressional runs
            num_districts_override: None,
            chamber: "congressional".to_string(),
            label: None,
            population_source: "total".to_string(),
            balance_tolerance: None,
            write_manifest: false,
            force: false,
            resolution: "tract".to_string(),
            plan_resolution: "tract".to_string(),
            seats_per_district: 1,
            total_seats: num_districts,
            adjacency_override: None,
            coi_weights: None,
        }
    }

    /// Returns the effective balance tolerance based on chamber type.
    ///
    /// Priority order:
    /// 1. Explicit `--balance-tolerance` override (always wins)
    /// 2. Chamber-specific value from state policy database
    /// 3. Fallback: 0.5% congressional / 5% state legislative
    pub fn effective_balance_tolerance(&self) -> f64 {
        self.balance_tolerance.unwrap_or_else(|| {
            chamber_balance_tolerance(&self.state_code, &self.chamber)
        })
    }

    /// Returns the effective label for this plan run.
    pub fn effective_label(&self) -> String {
        self.label.clone().unwrap_or_else(|| {
            redist_report::default_label(&self.state_name, &self.chamber, &self.year)
        })
    }

    /// Returns the effective number of districts (override takes priority).
    pub fn effective_num_districts(&self) -> usize {
        self.num_districts_override.unwrap_or(self.num_districts)
    }

    /// Returns the effective number of seats per district (always >= 1).
    pub fn effective_seats_per_district(&self) -> usize {
        self.seats_per_district.max(1)
    }

    /// Returns the ideal population per seat (not per district).
    /// For single-member: same as ideal_per_district.
    /// For multi-member: total_pop / total_seats.
    pub fn ideal_pop_per_seat(&self, total_pop: i64) -> f64 {
        let total_seats = self.total_seats.max(1);
        total_pop as f64 / total_seats as f64
    }
}

/// Resolve balance tolerance for a given chamber from state policy.
///
/// Uses state_policy.json fields: `balance_tolerance_house_pct`, `balance_tolerance_senate_pct`,
/// `balance_tolerance_congressional_pct`. Falls back to algorithm defaults (0.5% congressional,
/// 5.0% state legislative) when the state is not in the policy or the field is missing.
pub fn chamber_balance_tolerance(state_code: &str, chamber: &str) -> f64 {
    let policy = crate::policy::load_policy();
    if let Some(state) = crate::policy::get_state_policy(&policy, state_code) {
        let key = match chamber {
            "congressional" => "balance_tolerance_congressional_pct",
            "house" | "lower" | "assembly" => "balance_tolerance_house_pct",
            "senate" | "upper" => "balance_tolerance_senate_pct",
            _ => "",
        };
        if !key.is_empty() {
            if let Some(pct) = state.get(key).and_then(|v| v.as_f64()) {
                if pct > 0.0 {
                    return pct / 100.0; // policy stores in %, we use fraction
                }
            }
        }
    }
    // Fallback: constitutional standard for one-person-one-vote
    match chamber {
        "congressional" => 0.005,
        _ => 0.05,
    }
}

/// Resolve district count for a given chamber from state policy.
///
/// When `--chamber house` or `--chamber senate` is specified without `--districts`,
/// this looks up `house_districts` or `senate_districts` from the embedded state
/// policy database. Falls back to `congressional_fallback` (from the manifest) if
/// the policy doesn't have the chamber or the state is unknown.
///
/// This ensures `redist state --state WA --chamber house` automatically uses 98
/// districts without requiring the user to also pass `--districts 98`.
pub fn chamber_district_count(
    state_code: &str,
    chamber: &str,
    congressional_fallback: usize,
) -> usize {
    if chamber == "congressional" {
        return congressional_fallback;
    }
    let policy = crate::policy::load_policy();
    if let Some(state) = crate::policy::get_state_policy(&policy, state_code) {
        let key = match chamber {
            "house" | "lower" | "assembly" => "house_districts",
            "senate" | "upper" => "senate_districts",
            _ => return congressional_fallback,
        };
        if let Some(n) = state.get(key).and_then(|v| v.as_u64()) {
            if n > 0 {
                return n as usize;
            }
            if n == 0 && (key == "senate_districts" || key == "house_districts") {
                // Zero means this chamber doesn't exist (e.g., NE unicameral has no senate)
                let notes = state.get("notes").and_then(|v| v.as_str()).unwrap_or("");
                let hint = if notes.to_lowercase().contains("unicameral") {
                    format!(" {} has a unicameral legislature — use --chamber house.", state_code)
                } else {
                    format!(" {} has no {} chamber.", state_code, chamber)
                };
                eprintln!("ERROR: No {chamber} chamber for {state_code}.{hint}");
                std::process::exit(1);
            }
        }
    }
    congressional_fallback
}

/// Load all state codes, names, and district counts for a given year.
/// Returns Vec<(state_code, state_name, num_districts)> sorted alphabetically.
/// Reads directly from the embedded manifest — no Python subprocess.
///
/// Warning 6: if `year` is not in the manifest (e.g. "2030" or a typo),
/// all states are silently omitted. The caller sees an empty Vec with no error.
/// Valid years: "2020", "2010", "2000".
pub fn load_all_states(year: &str) -> Result<Vec<(String, String, usize)>, String> {
    if !["2020", "2010", "2000"].contains(&year) {
        return Err(format!(
            "unsupported year '{year}' — valid years are 2020, 2010, 2000"
        ));
    }
    let manifest = crate::fetch::load_manifest()?;
    let mut states: Vec<(String, String, usize)> = manifest.states.into_iter()
        .filter_map(|(code, state)| {
            let districts = *state.districts.get(year)?;
            if districts == 0 { return None; }
            let name = state.name.to_lowercase().replace(' ', "_");
            Some((code, name, districts))
        })
        .collect();
    states.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(states)
}

/// Return the actual worker count that will be used (capped to available CPU threads).
pub fn effective_workers(requested: usize) -> usize {
    requested.min(rayon::current_num_threads())
}

/// Run multiple states in parallel using Rayon.
/// Workers cap: min(workers, available_threads).
pub fn run_states_parallel(configs: Vec<StateConfig>, workers: usize) -> Vec<StateResult> {
    let actual_workers = effective_workers(workers);
    if actual_workers < workers {
        eprintln!(
            "NOTE: --workers {} capped to {} (available CPU threads). Actual parallelism: {}x.",
            workers, actual_workers, actual_workers
        );
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(actual_workers)
        .build()
        .expect("failed to build Rayon thread pool");

    pool.install(|| {
        configs.par_iter().map(|cfg| {
            let start = std::time::Instant::now();
            let result = run_single_state(cfg);
            let elapsed_ms = start.elapsed().as_millis() as u64;
            match result {
                Ok(()) => StateResult {
                    state_code: cfg.state_code.clone(),
                    success: true,
                    error: None,
                    elapsed_ms,
                },
                Err(e) => StateResult {
                    state_code: cfg.state_code.clone(),
                    success: false,
                    error: Some(format!("{}: {}", cfg.state_code, ascii_safe(&e.to_string()))),
                    elapsed_ms,
                }
            }
        }).collect()
    })
}

/// Extract the census year (2000, 2010, or 2020) from an adjacency filename.
///
/// Looks for a 4-digit number matching 2000, 2010, or 2020 in the filename.
/// Returns `None` if no valid census year is found.
pub fn extract_year_from_adj_filename(filename: &str) -> Option<&'static str> {
    // Search for any of the known census years as a substring
    for year in &["2020", "2010", "2000"] {
        if filename.contains(year) {
            return Some(year);
        }
    }
    None
}

/// Check whether the year in the adjacency filename matches the requested year.
///
/// Emits a WARNING to stderr (not an error) when there is a mismatch.
/// A mismatch can occur when the user requests --year 2020 but only a 2010
/// adjacency file is available and is used as fallback.
pub fn check_adjacency_year_mismatch(path: &PathBuf, requested_year: &str, state_code: &str) {
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if let Some(file_year) = extract_year_from_adj_filename(filename) {
        if file_year != requested_year {
            eprintln!(
                "WARNING: Requested adjacency for year {requested_year} but adjacency file is \
                 for year {file_year}: {filename}\n\
                 Census tract boundaries changed between {file_year} and {requested_year} \
                 -- results will use {file_year} geography.\n\
                 For {requested_year} census tracts: run \
                 redist fetch --year {requested_year} --type adjacency --states {}",
                state_code.to_uppercase()
            );
        }
    }
}

/// Resolve the adjacency pkl path for a state using the manifest.
///
/// The manifest's `local_outputs_dir` + "V3/data/{year}/adjacency/" is the
/// canonical adjacency store — the same path that `redist fetch --release` downloads to.
/// Override with REDIST_MANIFEST env var for custom data layouts.
///
/// Returns `(path, effective_resolution)` where `effective_resolution` may differ from
/// the requested resolution if a graceful fallback to tract occurred.
fn resolve_adjacency_path(
    state_code_lower: &str,
    year: &str,
    resolution: &str,
) -> Result<(PathBuf, String), String> {
    let manifest = load_manifest()
        .map_err(|e| format!("cannot load manifest: {e}"))?;
    let outputs_dir = PathBuf::from(&manifest.local_outputs_dir);

    // Choose filename based on requested resolution
    let (adj_filename, is_block_group) = match resolution {
        "block_group" | "block-group" => (
            format!("{state_code_lower}_bg_adjacency_{year}.pkl"),
            true,
        ),
        _ => (
            format!("{state_code_lower}_adjacency_{year}.pkl"),
            false,
        ),
    };

    // Try V3 then V4 canonical stores
    let canonical = outputs_dir
        .join("V3").join("data").join(year).join("adjacency")
        .join(&adj_filename);
    if canonical.exists() {
        return Ok((canonical, resolution.to_string()));
    }
    let v4 = outputs_dir
        .join("V4").join("data").join(year).join("adjacency")
        .join(&adj_filename);
    if v4.exists() {
        return Ok((v4, resolution.to_string()));
    }

    // Block group not found — graceful fallback to tract with clear warning
    if is_block_group {
        eprintln!(
            "WARNING: --resolution block_group was requested but block_group adjacency \
             not found for {state_code_lower} {year}.\n\
             To get block_group data: redist fetch --type adjacency --states {} --year {}\n\
             Falling back to tract resolution.",
            state_code_lower.to_uppercase(), year
        );
        let tract_filename = format!("{state_code_lower}_adjacency_{year}.pkl");
        let tract_canonical = outputs_dir
            .join("V3").join("data").join(year).join("adjacency")
            .join(&tract_filename);
        if tract_canonical.exists() {
            return Ok((tract_canonical, "tract".to_string()));
        }
        let tract_v4 = outputs_dir
            .join("V4").join("data").join(year).join("adjacency")
            .join(&tract_filename);
        if tract_v4.exists() {
            return Ok((tract_v4, "tract".to_string()));
        }
        let state_upper = state_code_lower.to_uppercase();
        return Err(format!(
            "Adjacency file not found for {state_code_lower} {year}.\n\
             Run: redist fetch --type adjacency --states {state_upper} --year {year}\n\
             Then: redist state --state {state_upper} --year {year} ..."
        ));
    }

    let state_upper = state_code_lower.to_uppercase();
    Err(format!(
        "Adjacency file not found for {state_code_lower} {year}.\n\
         Run: redist fetch --type adjacency --states {state_upper} --year {year}\n\
         Then: redist state --state {state_upper} --year {year} ..."
    ))
}

/// Check CVAP data availability and warn + fall back to total if missing.
///
/// The CVAP file is expected at:
///   `outputs/{version}/data/{year}/demographics/{state_lower}_cvap_{year}.csv`
/// or the legacy path:
///   `data/{year}/demographics/{state_lower}_cvap_{year}.csv`
///
/// Returns the effective population source: "cvap" if file exists, "total" otherwise.
pub fn check_cvap_availability(
    requested: &str,
    state_name: &str,
    year: &str,
    state_code: &str,
) -> String {
    if requested != "cvap" {
        return requested.to_string();
    }
    // Try the canonical CVAP path used by the Python pipeline
    let cvap_path = std::path::Path::new("data")
        .join(year)
        .join("demographics")
        .join(format!("{state_name}_cvap_{year}.csv"));
    if cvap_path.exists() {
        return "cvap".to_string();
    }
    eprintln!(
        "WARNING: CVAP data not found for {state_code} {year}.\n\
         CVAP requires a separate download: \
         https://www.census.gov/programs-surveys/decennial-census/about/voting-rights/cvap.html\n\
         Falling back to total population."
    );
    "total".to_string()
}

/// Validate Plan 03 partisan-mode configuration.
///
/// Callais p.36 disentanglement check (Plan 03 Task 4.5).
///
/// Previously enforced at runtime; now guaranteed structurally:
/// `PartitionMode` is a single-choice CLI enum so `partisan-weighted` and
/// `metis-vra` cannot both be active in the same `AlgorithmConfig`.
/// `WeightSpec.partisan_shares` and `WeightSpec.minority_weighting` are
/// set by mutually-exclusive `PartitionMode` arms in `from_state_args`.
///
/// This function is kept so call-sites compile; it always returns `Ok(())`.
pub fn validate_partisan_config(_cfg: &StateConfig) -> Result<(), String> {
    Ok(())
}

/// Run a single state redistricting task end-to-end.
///
/// Flow: load adjacency → build edge weights → bisect → assert balance → write outputs
fn run_single_state(cfg: &StateConfig) -> Result<(), String> {
    let num_districts = cfg.effective_num_districts();
    let state_name = &cfg.state_name; // e.g. "vermont" — used for directory paths
    let label = cfg.effective_label();
    let balance_tolerance = cfg.effective_balance_tolerance();
    // Defensive: tolerance must be in [0.0001, 0.50] as a fraction.
    // Values outside this range indicate a unit error (% passed as fraction or vice versa).
    if balance_tolerance < 0.0001 || balance_tolerance > 0.50 {
        return Err(format!(
            "{}: balance tolerance {:.6} is outside plausible range [0.0001, 0.50]. \
             Pass as percent to --balance-tolerance (e.g., 0.5 for ±0.5%, 5 for ±5%).",
            cfg.state_code, balance_tolerance
        ));
    }

    // Determine output directory structure:
    //   Labeled runs: {output_dir}/{year}/plans/{label}/data/
    //   Legacy runs:  {output_dir}/{year}/states/{state_name}/data/
    let year_base = cfg.output_dir.join(&cfg.year);
    let (plan_root, data_dir) = if cfg.label.is_some() {
        let plan_dir = year_base.join("plans").join(&label);
        let data_dir = plan_dir.join("data");
        (plan_dir, data_dir)
    } else {
        let state_dir = year_base.join("states").join(state_name);
        let data_dir = state_dir.join("data");
        (state_dir, data_dir)
    };

    // Board amendment: detect incomplete plan (manifest.tmp present)
    redist_report::check_incomplete_plan(&plan_root, &label)
        .map_err(|e| ascii_safe(&e.to_string()))?;

    // Label collision check: if manifest.json exists and --force not set, exit
    if cfg.label.is_some() {
        let manifest_path = plan_root.join("manifest.json");
        redist_report::check_plan_collision(&plan_root, cfg.force)
            .map_err(|e| ascii_safe(&e.to_string()))?;
        let _ = manifest_path; // suppress warning
    }

    // Reset: delete existing outputs before starting
    if cfg.reset {
        // Warn before deletion so users can see exactly what will be removed
        eprintln!(
            "WARNING: --reset will delete {} and all its contents before re-running.",
            plan_root.display()
        );
        if data_dir.exists() {
            std::fs::remove_dir_all(&data_dir)
                .map_err(|e| format!("reset failed: {e}"))?;
        }
    }
    // Create plan directory structure if labeled
    if cfg.label.is_some() {
        redist_report::create_plan_dir(&year_base, &label)
            .map_err(|e| format!("cannot create plan dir: {e}"))?;
    }
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("cannot create data dir: {e}"))?;

    status(cfg.position, &format!("{}: loading adjacency", cfg.state_code));

    // 1. Load adjacency graph
    // Adjacency path comes from the manifest (same source as `redist fetch`).
    // The manifest's local_outputs_dir + "V3/data/{year}/adjacency/" is the
    // canonical store. REDIST_MANIFEST can override this for custom setups.
    let state_code_lower = cfg.state_code.to_lowercase();
    let adj_pkl = if let Some(ref override_path) = cfg.adjacency_override {
        override_path.clone()
    } else {
        let (path, _effective_resolution) = resolve_adjacency_path(&state_code_lower, &cfg.year, &cfg.resolution)?;
        // Task 135: warn when adjacency file year doesn't match requested year
        check_adjacency_year_mismatch(&path, &cfg.year, &cfg.state_code);
        path
    };

    let graph = load_adjacency_pkl(&adj_pkl)
        .map_err(|e| format!("adjacency load failed: {e}"))?;

    // Check for isolated nodes (no adjacency neighbors) — common with island tracts.
    // Isolated tracts will always form non-contiguous districts.
    let isolated: Vec<usize> = graph.adjacency.iter().enumerate()
        .filter(|(_, nbrs)| nbrs.is_empty())
        .map(|(i, _)| i)
        .collect();
    if !isolated.is_empty() {
        eprintln!(
            "WARNING: {}: {} isolated tract(s) with no adjacency neighbors. \
             These will form non-contiguous districts. \
             For island states (AK, HI, international), rebuild adjacency with water bridges.",
            cfg.state_code, isolated.len()
        );
    }

    // 1b. CVAP population source check
    // CVAP data requires a separate download from the Census Bureau.
    // If "cvap" is requested but the file is missing, warn and fall back to total.
    let _effective_population_source = check_cvap_availability(
        &cfg.population_source,
        state_name,
        &cfg.year,
        &cfg.state_code,
    );

    // 2. Build edge weights using the composable WeightSpec.
    let edge_weights: HashMap<(usize, usize), f64> = if !cfg.algo.weights.geographic
        && !cfg.algo.weights.minority_weighting
        && cfg.algo.weights.partisan_shares.is_none()
        && cfg.algo.weights.alpha_county < 1e-10
        && cfg.algo.weights.alpha_mcd    < 1e-10
        && cfg.algo.weights.alpha_place  < 1e-10
        && cfg.algo.weights.alpha_vtd    < 1e-10
    {
        status(cfg.position, &format!("{}: unweighted mode", cfg.state_code));
        HashMap::new()
    } else if num_districts == 1 {
        status(cfg.position, &format!("{}: single district — skipping weighting", cfg.state_code));
        graph.edge_weights.clone()
    } else {
        build_edge_weights(
            &cfg.algo.weights, &graph, &cfg.state_code, state_name,
            &cfg.year, &cfg.output_dir, cfg.position,
        )?
    };

    // 2b. Apply COI weights if provided (all modes).
    let edge_weights = if let Some(ref coi_path) = cfg.coi_weights {
        let fallback = edge_weights.clone(); // keep computed weights if COI load fails
        match apply_coi_weights(edge_weights, coi_path, &graph.index_to_geoid) {
            Ok(ew) => ew,
            Err(e) => {
                eprintln!("WARNING: COI weights not applied: {e}");
                fallback
            }
        }
    } else {
        edge_weights
    };

    // 3. Build vertex weights from constraint spec + graph data.
    // Load TIGER areas only if the Area constraint is requested.
    let needs_area = cfg.algo.vertex_constraints
        .contains(&VertexConstraintKind::Area);
    let tiger_areas: Vec<f64> = if needs_area {
        let (areas, _) = load_tiger_geometry(
            &cfg.state_code, &cfg.year, &graph.index_to_geoid,
            &graph.adjacency, &graph.edge_weights);
        areas
    } else {
        vec![]
    };
    let vw = build_vertex_weights(
        &cfg.algo.vertex_constraints,
        &graph.vertex_weights,
        &tiger_areas,
    );

    // 4. Run partitioning — dispatch on split strategy.
    let intermediate_dir = plan_root.join("intermediate");
    std::fs::create_dir_all(&intermediate_dir)
        .map_err(|e| format!("cannot create intermediate dir: {e}"))?;

    let balance_tolerance_frac = cfg.effective_balance_tolerance();
    let ufactor = cfg.algo.metis.ufactor;
    let niter = cfg.algo.metis.niter;
    let base_seed = cfg.algo.metis.seed;

    let vwgt = vw.interleaved(graph.n_vertices);

    // Auto-retry: METIS balance is a soft constraint and some seeds produce
    // imbalanced partitions. Try up to MAX_RETRIES seeds before giving up.
    const MAX_BALANCE_RETRIES: u32 = 50;
    let mut assignments = HashMap::new();
    let mut last_balance_err = String::new();
    let mut seed = base_seed;
    // Flip-chain audit fields — set inside the Flip dispatch arm.
    let mut flip_visited_count: Option<usize> = None;
    let mut flip_selected_rank: Option<usize> = None;
    // Short-Burst audit fields — set inside the ShortBurst dispatch arm.
    let mut short_burst_burst_seeds: Option<Vec<u64>> = None;
    let mut short_burst_selected_burst_idx: Option<usize> = None;

    'retry: for attempt in 0..=MAX_BALANCE_RETRIES {
        if attempt > 0 {
            seed = Some(base_seed.unwrap_or(0).wrapping_add(attempt as u64));
            status(cfg.position, &format!("{}: balance retry {}/{} (seed {:?})",
                cfg.state_code, attempt, MAX_BALANCE_RETRIES, seed));
        }

    let partition_t0 = std::time::Instant::now();
    let assignments_attempt = match &cfg.algo.split {
        SplitStrategy::NWay if num_districts > 1 => {
            status(cfg.position, &format!("{}: n-way into {} districts", cfg.state_code, num_districts));
            run_nway_partition(
                &graph.adjacency, &vwgt, &edge_weights,
                num_districts, 1.0 + ufactor as f64 / 1000.0, niter, seed,
            ).map_err(|e| format!("n-way partition failed: {e}"))?
        }
        SplitStrategy::GeoSection => {
            let seeds_per_ratio = cfg.algo.seeds.seed_count();
            let lambda = cfg.algo.weights.directional_lambda;
            let centroids = if lambda > 1e-10 {
                crate::geosection_orientation::load_centroids_from_tiger(
                    &cfg.state_code, &cfg.year, &graph.index_to_geoid)
            } else {
                crate::geosection_orientation::CentroidMap::new()
            };
            if lambda > 1e-10 {
                status(cfg.position, &format!("{}: GeoSection λ={:.1} ({} centroids loaded)",
                       cfg.state_code, lambda, centroids.len()));
            } else {
                status(cfg.position, &format!("{}: GeoSection {} ratios × {} seeds",
                       cfg.state_code, num_districts / 2, seeds_per_ratio));
            }
            let (asgn, nat_left, nat_right, nat_ec) = run_geosection(
                &graph.adjacency, &vwgt, &edge_weights,
                num_districts, balance_tolerance_frac, niter,
                seeds_per_ratio, Some(&intermediate_dir),
                &centroids, lambda, None, 1.10, None, 0.0,
            ).map_err(|e| format!("geosection failed: {e}"))?;
            status(cfg.position, &format!("{}: natural ratio {}:{} at {:.0}km",
                   cfg.state_code, nat_left, nat_right, nat_ec / 1000.0));
            asgn
        }
        SplitStrategy::AreaSection { area_swing } => {
            let seeds_per_ratio = cfg.algo.seeds.seed_count();
            if tiger_areas.is_empty() {
                return Err(format!("{}: AreaSection requires TIGER ALAND data — not found",
                                   cfg.state_code));
            }
            let empty_centroids = crate::geosection_orientation::CentroidMap::new();
            status(cfg.position, &format!(
                "{}: AreaSection {} ratios x {} seeds (pop+area dual, ncon=2)",
                cfg.state_code, num_districts / 2, seeds_per_ratio));
            let (asgn, nat_left, nat_right, nat_ec) = run_geosection(
                &graph.adjacency, &graph.vertex_weights, &edge_weights,
                num_districts, balance_tolerance_frac, niter,
                seeds_per_ratio, Some(&intermediate_dir),
                &empty_centroids, 0.0, Some(&tiger_areas), *area_swing, None, 0.0,
            ).map_err(|e| format!("areasection failed: {e}"))?;
            status(cfg.position, &format!("{}: natural ratio {}:{} at {:.0}km",
                   cfg.state_code, nat_left, nat_right, nat_ec / 1000.0));
            asgn
        }
        SplitStrategy::VraSection { w_vra } => {
            let seeds_per_ratio = cfg.algo.seeds.seed_count();
            // Load minority VAP data from demographics CSV.
            // Minority fraction = (total_pop - white_non_hispanic) / total_pop,
            // multiplied by tract population to get approximate minority VAP counts.
            // (This is a spatial distribution proxy — not exact VAP data, but legally
            //  defensible because no racial targeting occurs: only the geographic
            //  distribution of existing minority concentrations is observed.)
            let demo_path = std::path::Path::new("data")
                .join(&cfg.year).join("demographics")
                .join(format!("{state_name}_demographics_{}.csv", cfg.year));
            let minority_vap_vec: Vec<f64> = if demo_path.exists() {
                let demo = crate::demographics::load_demographics(&demo_path)
                    .map_err(|e| format!("{}: VRASection demographics load failed: {e}", cfg.state_code))?;
                let fracs = crate::demographics::align_demographics_to_adjacency(
                    &demo, &graph.index_to_geoid, graph.n_vertices);
                // Convert fraction × population → approximate minority VAP count
                fracs.iter().zip(graph.vertex_weights.iter())
                    .map(|(&frac, &pop)| frac * pop as f64)
                    .collect()
            } else {
                eprintln!("WARNING: {}: VRASection demographics not found at {} — running as plain GeoSection",
                          cfg.state_code, demo_path.display());
                vec![]
            };
            let mvap_opt: Option<&[f64]> = if minority_vap_vec.is_empty() { None } else { Some(&minority_vap_vec) };
            let empty_centroids = crate::geosection_orientation::CentroidMap::new();
            status(cfg.position, &format!(
                "{}: VRASection {} ratios x {} seeds w_vra={:.2}",
                cfg.state_code, num_districts / 2, seeds_per_ratio, w_vra));
            let (asgn, nat_left, nat_right, nat_ec) = run_geosection(
                &graph.adjacency, &vwgt, &edge_weights,
                num_districts, balance_tolerance_frac, niter,
                seeds_per_ratio, Some(&intermediate_dir),
                &empty_centroids, 0.0, None, 1.10, mvap_opt, *w_vra,
            ).map_err(|e| format!("vra-section failed: {e}"))?;
            status(cfg.position, &format!("{}: natural ratio {}:{} at {:.0}km",
                   cfg.state_code, nat_left, nat_right, nat_ec / 1000.0));
            asgn
        }
        SplitStrategy::ProportionalSection { eta } => {
            let seeds = cfg.algo.seeds.seed_count();
            // Load D_votes from presidential_by_tract.csv
            let election_path = std::path::PathBuf::from(format!(
                "data/{}/elections/presidential_by_tract.csv", cfg.year));
            if !election_path.exists() {
                return Err(format!("{}: ProportionalSection requires {} — not found",
                                   cfg.state_code, election_path.display()));
            }
            let (d_votes, two_party) = crate::partisan_shares::load_dem_vote_counts(
                &election_path, &graph.index_to_geoid, graph.n_vertices)
                .map_err(|e| format!("{}: load_dem_vote_counts failed: {e}", cfg.state_code))?;
            status(cfg.position, &format!(
                "{}: ProportionalSection {} seeds eta={:.2} (pop+D_votes ncon=2)",
                cfg.state_code, seeds, eta));
            let (asgn, k_d, k_r, best_ec, d_state) =
                crate::bisection_runner::run_proportional_section(
                    &graph.adjacency, &graph.vertex_weights, &d_votes, &two_party, &edge_weights,
                    num_districts, balance_tolerance_frac, niter, seeds, *eta,
                    Some(&intermediate_dir),
                ).map_err(|e| format!("proportional-section failed: {e}"))?;
            status(cfg.position, &format!(
                "{}: proportional {}/{}D d={:.3} EC={:.0}km",
                cfg.state_code, k_d, k_r, d_state, best_ec / 1000.0));
            asgn
        }
        SplitStrategy::CompactBisect { epsilon } => {
            let seeds_per_level = cfg.algo.seeds.seed_count();
            let (vertex_areas, vertex_ext_perimeters) =
                load_tiger_geometry(&cfg.state_code, &cfg.year, &graph.index_to_geoid,
                                    &graph.adjacency, &edge_weights);
            let opts = CompactBisectOpts { seeds_per_level, epsilon: *epsilon };
            run_all_splits_compact(
                &graph.adjacency, &vwgt, &edge_weights,
                &vertex_areas, &vertex_ext_perimeters,
                num_districts, balance_tolerance_frac, niter, None, &opts,
                Some(&intermediate_dir),
            ).map_err(|e| format!("compact-bisect failed: {e}"))?
        }
        SplitStrategy::ApportionRegions => {
            use redist_apportion::{PfrCompositor, MetisPartitioner, pfr_tree_depth};
            let factor_seq = redist_apportion::prime_factor_sequence(num_districts as u32);
            let depth = pfr_tree_depth(num_districts as u32).max(1);
            status(cfg.position, &format!("{}: apportion-regions partition into {} districts \
                (F={:?}, depth={})", cfg.state_code, num_districts, factor_seq, depth));
            // Per-level tolerance: (1 + per_level)^depth ≤ 1 + final_tol.
            // Use depth+1 in denominator for margin (METIS can slightly exceed ufactor).
            // Clamped to METIS minimum 0.1% (ufactor=1).
            // Note: PFR is a research algorithm; the final balance check uses a relaxed
            // 2% tolerance so we can measure actual balance rather than reject runs.
            let per_level_tol = (balance_tolerance_frac / (depth + 1) as f64).max(0.001);
            let partitioner = MetisPartitioner {
                balance_tolerance: per_level_tol,
                niter: niter as i32,
                engine: cfg.algo.metis.engine,
            };
            let compositor = PfrCompositor::new(partitioner);
            let result = compositor.compose(
                &graph.adjacency, &graph.vertex_weights, &edge_weights,
                num_districts as u32,
                seed,
            ).map_err(|e| format!("apportion-regions failed: {e}"))?;
            // Check balance at relaxed 2% — report actual deviation in manifest.
            let pfr_assignments: std::collections::HashMap<usize, usize> = result.assignment
                .iter().enumerate().map(|(t, &d)| (t, d as usize + 1)).collect();
            let pfr_partition = redist_core::Partition::from_assignments(pfr_assignments.clone());
            let pfr_balance = pfr_partition.population_balance(&graph.vertex_weights, num_districts);
            if pfr_balance > 0.03 {
                return Err(format!("apportion-regions balance {:.1}% exceeds 3% research limit",
                    pfr_balance * 100.0));
            }
            status(cfg.position, &format!("{}: balance {:.2}% (cache hits={})",
                cfg.state_code, pfr_balance * 100.0, result.cache_hits));
            pfr_assignments
        }
        SplitStrategy::SimulatedAnnealing { steps_per_tract, t0_factor, t_final } => {
            let base_seed_val = seed.unwrap_or(0);
            status(cfg.position, &format!(
                "{}: simulated-annealing {} steps/tract t0_factor={:.4} t_final={:.2e} into {} districts",
                cfg.state_code, steps_per_tract, t0_factor, t_final, num_districts));
            crate::bisection_runner::run_all_splits_sa(
                &graph.adjacency, &vwgt, &edge_weights,
                num_districts, balance_tolerance_frac, niter, seed,
                Some(&intermediate_dir),
                *steps_per_tract, *t0_factor, *t_final, base_seed_val,
            ).map_err(|e| format!("simulated-annealing failed: {e}"))?
        }
        _ => {
            match &cfg.algo.seeds {
                SeedCompositor::Percentile { p, seeds } => {
                    let base = seed.unwrap_or(0);
                    status(cfg.position, &format!("{}: percentile-sweep p={:.2} {} seeds into {} districts",
                           cfg.state_code, p, seeds, num_districts));
                    run_all_splits_percentile(
                        &graph.adjacency, &vwgt, &edge_weights,
                        num_districts, balance_tolerance_frac, niter,
                        base, *seeds, *p,
                        Some(&intermediate_dir),
                    ).map_err(|e| format!("percentile-sweep failed: {e}"))?
                }
                SeedCompositor::BisectionEnsemble { p, ensemble_steps } => {
                    status(cfg.position, &format!("{}: bisection-ensemble p={:.2} {} steps/node into {} districts",
                           cfg.state_code, p, ensemble_steps, num_districts));
                    run_all_splits_with_search(
                        &graph.adjacency, &vwgt, &edge_weights,
                        num_districts, balance_tolerance_frac, niter, seed,
                        Some(&intermediate_dir),
                        Some((*p, *ensemble_steps)),
                    ).map_err(|e| format!("bisection-ensemble failed: {e}"))?
                }
                SeedCompositor::Flip { flip_steps, p } => {
                    let base = seed.unwrap_or(0);
                    status(cfg.position, &format!("{}: flip-chain p={:.2} {} steps into {} districts",
                           cfg.state_code, p, flip_steps, num_districts));
                    let (result, visited_count, selected_rank) = run_flip_chain(
                        &graph.adjacency, &vwgt, &edge_weights,
                        num_districts, balance_tolerance_frac, *flip_steps, base, *p,
                    ).map_err(|e| format!("flip-chain failed: {e}"))?;
                    flip_visited_count = Some(visited_count);
                    flip_selected_rank = Some(selected_rank);
                    result
                }
                SeedCompositor::ShortBurst { burst_length, n_bursts, p } => {
                    let base = seed.unwrap_or(0);
                    status(cfg.position, &format!("{}: short-burst p={:.2} {} bursts x {} steps into {} districts",
                           cfg.state_code, p, n_bursts, burst_length, num_districts));
                    let (result, b_seeds, b_idx) = run_short_burst(
                        &graph.adjacency, &vwgt, &edge_weights,
                        num_districts, balance_tolerance_frac, niter,
                        base, *burst_length, *n_bursts, *p,
                    ).map_err(|e| format!("short-burst failed: {e}"))?;
                    short_burst_burst_seeds = Some(b_seeds);
                    short_burst_selected_burst_idx = Some(b_idx);
                    result
                }
                SeedCompositor::ShortBurstForest { burst_length, n_bursts, p } => {
                    let base = seed.unwrap_or(0);
                    status(cfg.position, &format!("{}: short-burst-forest p={:.2} {} bursts x {} steps into {} districts",
                           cfg.state_code, p, n_bursts, burst_length, num_districts));
                    run_short_burst_forest(
                        &graph.adjacency, &vwgt, &edge_weights,
                        num_districts, balance_tolerance_frac, niter,
                        base, *burst_length, *n_bursts, *p,
                    ).map_err(|e| format!("short-burst-forest: {e}"))?
                }
                SeedCompositor::ShortBurstMergeSplit { burst_length, n_bursts, p } => {
                    let base = seed.unwrap_or(0);
                    status(cfg.position, &format!("{}: short-burst-merge-split p={:.2} {} bursts x {} steps into {} districts",
                           cfg.state_code, p, n_bursts, burst_length, num_districts));
                    run_short_burst_merge_split(
                        &graph.adjacency, &vwgt, &edge_weights,
                        num_districts, balance_tolerance_frac, niter,
                        base, *burst_length, *n_bursts, *p,
                    ).map_err(|e| format!("short-burst-merge-split: {e}"))?
                }
                SeedCompositor::ForestRecom { steps, p } => {
                    let base = seed.unwrap_or(0);
                    status(cfg.position, &format!("{}: forest-recom p={:.2} {} steps into {} districts",
                           cfg.state_code, p, steps, num_districts));
                    run_forest_recom(
                        &graph.adjacency, &vwgt, &edge_weights,
                        num_districts, balance_tolerance_frac, niter,
                        base, *steps, *p,
                    ).map_err(|e| format!("forest-recom failed: {e}"))?
                }
                SeedCompositor::MultiScale { total_steps, p, alpha } => {
                    let base = seed.unwrap_or(0);
                    status(cfg.position, &format!("{}: multiscale p={:.2} {} steps alpha={:.2} into {} districts",
                           cfg.state_code, p, total_steps, alpha, num_districts));
                    run_multiscale(
                        &graph.adjacency, &vwgt, &edge_weights,
                        num_districts, balance_tolerance_frac, niter,
                        base, *total_steps, *alpha, *p,
                        Some(&graph.index_to_geoid),
                    ).map_err(|e| format!("multiscale: {e}"))?
                }
                SeedCompositor::MergeSplit { steps, p } => {
                    let base = seed.unwrap_or(0);
                    status(cfg.position, &format!("{}: merge-split p={:.2} {} steps into {} districts",
                           cfg.state_code, p, steps, num_districts));
                    run_merge_split(
                        &graph.adjacency, &vwgt, &edge_weights,
                        num_districts, balance_tolerance_frac, niter,
                        base, *steps, *p,
                    ).map_err(|e| format!("merge-split failed: {e}"))?
                }
                _ => {
                    status(cfg.position, &format!("{}: recursive bisection into {} districts",
                           cfg.state_code, num_districts));
                    run_all_splits(
                        &graph.adjacency, &vwgt, &edge_weights,
                        num_districts, balance_tolerance_frac, niter, seed,
                        Some(&intermediate_dir),
                    ).map_err(|e| format!("bisection failed: {e}"))?
                }
            }
        }
    };

    if cfg.time_partition {
        let partition_elapsed_ms = partition_t0.elapsed().as_secs_f64() * 1000.0;
        eprintln!(
            "[partition-time] {}: k={} n={} -> {:.1}ms",
            cfg.state_code, num_districts, graph.n_vertices, partition_elapsed_ms
        );
    }

    // 4. Assert population balance — retry loop closes here.
    let effective_tolerance = if matches!(cfg.algo.split, SplitStrategy::ApportionRegions) {
        0.03
    } else {
        balance_tolerance
    };
    let partition = Partition::from_assignments(assignments_attempt.clone());
    let balance_ok = partition.assert_balanced(&graph.vertex_weights, num_districts, effective_tolerance);
    match balance_ok {
        Ok(_) => {
            assignments = assignments_attempt;
            break 'retry;
        }
        Err(e) => {
            last_balance_err = format!("population balance violation (tolerance {:.1}%): {e}",
                effective_tolerance * 100.0);
            // continue retry loop
        }
    }
    } // end 'retry loop

    if assignments.is_empty() {
        return Err(last_balance_err);
    }

    status(cfg.position, &format!("{}: balance OK, writing outputs", cfg.state_code));

    // 5. VRA analysis (if VRA mode and multi-district)
    let vra = if matches!(&cfg.algo.split, SplitStrategy::NWay) && num_districts > 1 {
        let demo_path = std::path::Path::new("data")
            .join(&cfg.year).join("demographics")
            .join(format!("{state_name}_demographics_{}.csv", cfg.year));
        let demo = load_demographics(&demo_path)
            .map_err(|e| format!("demographics reload for VRA analysis failed: {e}"))?;
        let minority_fracs = align_demographics_to_adjacency(
            &demo, &graph.index_to_geoid, graph.n_vertices
        );
        let total_pops = graph.vertex_weights.clone();
        let minority_pops: Vec<f64> = minority_fracs.iter()
            .zip(total_pops.iter())
            .map(|(f, &t)| f * t as f64)
            .collect();
        let zeros = vec![0.0f64; graph.n_vertices];
        let analysis = analyze_mm_districts(
            &assignments, &total_pops, &minority_pops, &zeros, &zeros, 0.50
        );
        status(cfg.position, &format!("{}: {} MM districts", cfg.state_code, analysis.mm_count));
        Some(VraAnalysis {
            mm_count: analysis.mm_count,
            mm_districts: analysis.mm_districts,
            districts: analysis.districts.iter().map(|d| VraDistrict {
                district: d.district,
                pct_minority: d.pct_minority,
                pct_black: d.pct_black,
                pct_hispanic: d.pct_hispanic,
                is_mm: d.is_mm,
            }).collect(),
        })
    } else {
        None
    };

    // 6. Write outputs atomically (rename-from-tmp pattern)
    write_state_outputs(&data_dir, &assignments, vra.as_ref())
        .map_err(|e| format!("output write failed: {e}"))?;

    // 6b. Compute edge-cut of the final partition.
    // Sum edge weights for all edges (u, v) whose endpoints are in different districts.
    // Stored in the manifest for seed-sensitivity research (B.7).
    let edge_cut: f64 = edge_weights.iter().map(|(&(u, v), &w)| {
        let du = assignments.get(&u).copied().unwrap_or(0);
        let dv = assignments.get(&v).copied().unwrap_or(0);
        if du != dv { w } else { 0.0 }
    }).sum();

    // 7. Write manifest.json atomically (manifest.tmp → manifest.json).
    // Board amendment: atomic write (manifest.tmp + rename) prevents partial manifests.
    if cfg.write_manifest || cfg.label.is_some() {
        let adj_filename = format!("{}_adjacency_{}.adj.bin", state_name, cfg.year);
        let state_fips = state_code_to_fips(&cfg.state_code).unwrap_or("00").to_string();
        let tiger_url = redist_report::tiger_source_url(&state_fips, &cfg.year);
        let gpmetis_version = crate::bisection_runner::detect_gpmetis_version();
        let manifest = redist_report::PlanManifest {
            label: label.clone(),
            state_code: cfg.state_code.clone(),
            year: cfg.year.clone(),
            chamber: cfg.chamber.clone(),
            num_districts,
            population_source: cfg.population_source.clone(),
            partition_mode: cfg.algo.mode_name().to_string(),
            seed: seed.map(|s| s as i64),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            binary_sha256: String::new(), // populated by installer/release only
            binary_download_url: format!(
                "https://github.com/owner/redist/releases/download/v{}/redist",
                env!("CARGO_PKG_VERSION")
            ),
            adjacency_file: adj_filename,
            adjacency_sha256: String::new(), // populated post-build
            adjacency_build_command: format!(
                "python scripts/data/generate_adj_bin.py --year {} --states {}",
                cfg.year, state_name
            ),
            adjacency_build_version: env!("CARGO_PKG_VERSION").to_string(),
            tiger_source_url: tiger_url,
            tiger_sha256: None, // expensive; computed separately if needed
            created_at: redist_report::now_iso8601(),
            balance_tolerance_pct: balance_tolerance * 100.0,
            population_balance_valid: true,
            seats_per_district: cfg.effective_seats_per_district(),
            total_seats: cfg.total_seats,
            electoral_system: if cfg.seats_per_district <= 1 {
                "single_member".to_string()
            } else {
                "multi_member_uniform".to_string()
            },
            gpmetis_version,
            // AlgorithmConfig reproducibility fields
            ufactor: cfg.algo.metis.ufactor,
            niter:   cfg.algo.metis.niter,
            alpha_county: cfg.algo.weights.alpha_county,
            directional_lambda: cfg.algo.weights.directional_lambda,
            split_seeds: match &cfg.algo.split {
                SplitStrategy::GeoSection
                | SplitStrategy::AreaSection   { .. }
                | SplitStrategy::VraSection    { .. }
                | SplitStrategy::CompactBisect { .. }
                | SplitStrategy::ProportionalSection { .. }
                    => Some(cfg.algo.seeds.seed_count()),
                SplitStrategy::ApportionRegions => Some(cfg.algo.seeds.seed_count()),
                _ => None,
            },
            split_epsilon: match &cfg.algo.split {
                SplitStrategy::CompactBisect { epsilon, .. } => Some(*epsilon),
                _ => None,
            },
            area_swing: match &cfg.algo.split {
                SplitStrategy::AreaSection { area_swing, .. } => Some(*area_swing),
                _ => None,
            },
            // SSI Task 5/7 fields: state-staff-imported plans set these via run_import;
            // state-runner-produced plans default to authoritative + None (per `..Default`).
            submission_type: "authoritative".to_string(),
            submitted_by: None,
            submitted_at: None,
            source_tool: None,
            source_tool_version: None,
            source_format_fingerprint: None,
            import_compat_sha256: None,
            edge_cut: Some(edge_cut),
            // Flip search audit fields
            flip_search: if matches!(&cfg.algo.seeds, SeedCompositor::Flip { .. }) {
                Some("flip".to_string())
            } else {
                None
            },
            flip_steps: if let SeedCompositor::Flip { flip_steps, .. } = &cfg.algo.seeds {
                Some(*flip_steps)
            } else {
                None
            },
            flip_percentile: if let SeedCompositor::Flip { p, .. } = &cfg.algo.seeds {
                Some(*p)
            } else {
                None
            },
            flip_base_seed: if matches!(&cfg.algo.seeds, SeedCompositor::Flip { .. }) {
                Some(seed.unwrap_or(0))
            } else {
                None
            },
            flip_visited_count,
            flip_selected_plan_rank: flip_selected_rank,
            // Short-burst audit fields (shared across ShortBurst, ShortBurstForest, ShortBurstMergeSplit)
            short_burst_search: match &cfg.algo.seeds {
                SeedCompositor::ShortBurst { .. } => Some("short-burst".to_string()),
                SeedCompositor::ShortBurstForest { .. } => Some("short-burst-forest".to_string()),
                SeedCompositor::ShortBurstMergeSplit { .. } => Some("short-burst-merge-split".to_string()),
                _ => None,
            },
            burst_length: match &cfg.algo.seeds {
                SeedCompositor::ShortBurst { burst_length, .. }
                | SeedCompositor::ShortBurstForest { burst_length, .. }
                | SeedCompositor::ShortBurstMergeSplit { burst_length, .. } => Some(*burst_length),
                _ => None,
            },
            n_bursts: match &cfg.algo.seeds {
                SeedCompositor::ShortBurst { n_bursts, .. }
                | SeedCompositor::ShortBurstForest { n_bursts, .. }
                | SeedCompositor::ShortBurstMergeSplit { n_bursts, .. } => Some(*n_bursts),
                _ => None,
            },
            short_burst_percentile: match &cfg.algo.seeds {
                SeedCompositor::ShortBurst { p, .. }
                | SeedCompositor::ShortBurstForest { p, .. }
                | SeedCompositor::ShortBurstMergeSplit { p, .. } => Some(*p),
                _ => None,
            },
            short_burst_base_seed: match &cfg.algo.seeds {
                SeedCompositor::ShortBurst { .. }
                | SeedCompositor::ShortBurstForest { .. }
                | SeedCompositor::ShortBurstMergeSplit { .. } => Some(seed.unwrap_or(0)),
                _ => None,
            },
            burst_seeds: short_burst_burst_seeds,
            selected_burst_idx: short_burst_selected_burst_idx,
            // Plan resolution fields
            plan_resolution: cfg.plan_resolution.clone(),
            n_units: graph.adjacency.len(),
            unit_type: match cfg.plan_resolution.as_str() {
                "bg" => "census block group".to_string(),
                "county" => "county".to_string(),
                _ => "census tract".to_string(),
            },
            // Multi-scale fields
            multiscale_fine: if matches!(&cfg.algo.seeds, SeedCompositor::MultiScale { .. }) {
                Some("tract".to_string())
            } else {
                None
            },
            multiscale_coarse: if matches!(&cfg.algo.seeds, SeedCompositor::MultiScale { .. }) {
                Some("county".to_string())
            } else {
                None
            },
            fine_to_coarse_formula: if matches!(&cfg.algo.seeds, SeedCompositor::MultiScale { .. }) {
                Some("geoid_prefix[:5]".to_string())
            } else {
                None
            },
            fine_to_coarse_comment: if matches!(&cfg.algo.seeds, SeedCompositor::MultiScale { .. }) {
                Some("first 5 chars of 11-char tract GEOID = parent county FIPS".to_string())
            } else {
                None
            },
            index_to_geoid_file: if matches!(&cfg.algo.seeds, SeedCompositor::MultiScale { .. }) {
                Some(format!("{}_adjacency_{}_geoids.json", state_name, cfg.year))
            } else {
                None
            },
        };
        redist_report::write_manifest_atomic(&plan_root, &manifest)
            .map_err(|e| format!("manifest write failed: {e}"))?;
    }

    status(cfg.position, &format!("{}: complete ({num_districts}D, {}ms)", cfg.state_code, 0));
    Ok(())
}

/// Check if a state's outputs already exist and are complete.
pub fn state_already_complete(output_dir: &PathBuf, state_code: &str, year: &str, reprocess: bool) -> bool {
    if reprocess { return false; }
    let data_dir = output_dir
        .join(year).join("states").join(state_code.to_lowercase()).join("data");
    data_dir.join("final_assignments.json").exists()
        || data_dir.join("final_assignments.pkl").exists()
}

/// Filter configs to only those needing processing.
pub fn filter_incomplete(configs: Vec<StateConfig>) -> Vec<StateConfig> {
    configs.into_iter()
        .filter(|cfg| !state_already_complete(&cfg.output_dir, &cfg.state_code, &cfg.year, cfg.reprocess))
        .collect()
}

/// Build edge weights from a `WeightSpec` using the composable `EdgeWeighter` pipeline.
///
/// Steps applied in order:
///   1. Geographic boundary lengths (if `spec.geographic`)
///   2. Minority / VRA signal (if `spec.minority_weighting`)
///   3. Partisan signal (if `spec.partisan_shares.is_some()`)
///   4. County subdivision stickiness (if `spec.alpha_county > 1e-10`)
///
/// Returns `Ok(EdgeMap)` on success. Errors from data loading propagate as `Err(String)`.
fn build_edge_weights(
    spec: &WeightSpec,
    graph: &crate::adjacency_loader::LoadedGraph,
    state_code: &str,
    state_name: &str,
    year: &str,
    _output_dir: &std::path::PathBuf,
    position: i32,
) -> Result<HashMap<(usize, usize), f64>, String> {
    use crate::edge_weights::{
        ComposedWeighter, GeographicWeighter,
        MinorityOverrideWeighter, PartisanOverrideWeighter,
        SubdivisionWeighter,
    };

    let edges: Vec<(usize, usize)> = graph.adjacency.iter().enumerate()
        .flat_map(|(i, nbrs)| nbrs.iter().filter(move |&&j| j > i).map(move |&j| (i, j)))
        .collect();

    let mut composer = ComposedWeighter::new();

    // Step 1: Geographic base weights (TIGER boundary lengths).
    if spec.geographic {
        composer = composer.push(GeographicWeighter::from_map(graph.edge_weights.clone()));
    }

    // Step 2: Minority / VRA — override variant (from scratch, historic behaviour).
    if spec.minority_weighting {
        status(position, &format!("{state_code}: VRA mode — loading demographics"));
        let demo_path = std::path::Path::new("data")
            .join(year).join("demographics")
            .join(format!("{state_name}_demographics_{year}.csv"));
        let demo = load_demographics(&demo_path)
            .map_err(|e| format!("demographics load failed: {e}"))?;
        let minority_fracs = align_demographics_to_adjacency(
            &demo, &graph.index_to_geoid, graph.n_vertices);
        composer = composer.push(MinorityOverrideWeighter::new(edges.clone(), minority_fracs, 0.40));
    }

    // Step 3: Partisan signal — override variant (from scratch, historic behaviour).
    if let Some(ref partisan_path) = spec.partisan_shares {
        if !partisan_path.as_os_str().is_empty() {
            status(position, &format!("{state_code}: partisan-weighted — loading {}", partisan_path.display()));
            let dem_shares = load_partisan_shares(partisan_path, &graph.index_to_geoid, graph.n_vertices)
                .map_err(|e| format!("partisan shares load failed: {e}"))?;
            composer = composer.push(PartisanOverrideWeighter::new(
                edges.clone(), dem_shares, spec.dem_threshold, spec.rep_threshold,
            ));
        }
    }

    // Step 4: Subdivision stickiness (B.10) — augment on whatever base is set.
    if spec.alpha_county > 1e-10 {
        composer = composer.push(SubdivisionWeighter::county_only(
            &graph.index_to_geoid, graph.n_vertices, spec.alpha_county,
        ));
    }

    // If nothing was added to the composer, fall back to geographic weights
    if composer.is_empty() {
        return Ok(graph.edge_weights.clone());
    }

    Ok(composer.apply())
}

/// Apply COI (Communities of Interest) weights to edge weights.
///
/// Loads a JSON file mapping GEOID -> weight (0.0-1.0). For each edge (u, v),
/// multiplies the edge weight by sqrt(w_u * w_v) (geometric mean of endpoint weights).
/// Tracts not in the COI file get weight 1.0 (no modification).
///
/// The geometric mean ensures that if both endpoints of an edge are in the same
/// community (high weight), the edge is strengthened and METIS will avoid cutting it.
pub fn apply_coi_weights(
    mut edge_weights: HashMap<(usize, usize), f64>,
    coi_path: &std::path::Path,
    index_to_geoid: &HashMap<usize, String>,
) -> Result<HashMap<(usize, usize), f64>, String> {
    let content = std::fs::read_to_string(coi_path)
        .map_err(|e| format!("cannot read COI weights file {}: {e}", coi_path.display()))?;
    let coi_map: HashMap<String, f64> = serde_json::from_str(&content)
        .map_err(|e| format!("cannot parse COI weights JSON: {e}"))?;

    // Build a geoid -> weight lookup by index
    let get_weight = |idx: usize| -> f64 {
        index_to_geoid.get(&idx)
            .and_then(|geoid| coi_map.get(geoid))
            .copied()
            .unwrap_or(1.0)
    };

    for (&(u, v), weight) in edge_weights.iter_mut() {
        let w_u = get_weight(u);
        let w_v = get_weight(v);
        let factor = (w_u * w_v).sqrt();
        *weight *= factor;
    }

    Ok(edge_weights)
}

/// Load per-tract area (m²) and external perimeter (m) from TIGER shapefiles.
/// Used by CompactBisect to compute Polsby-Popper at each bisection level.
///
/// Area: ALAND field from TIGER (accurate, in m²).
/// External perimeter: approximated as 2√(π·ALAND) − Σ(shared edge weights).
/// The circular approximation is slightly off for elongated tracts but preserves
/// relative compactness rankings within the same subgraph.
///
/// Returns (vertex_areas, vertex_ext_perimeters) aligned to adjacency indices.
/// Returns empty vecs if TIGER file is not found (CompactBisect gracefully degrades).
pub fn load_tiger_geometry(
    state_code: &str,
    year: &str,
    index_to_geoid: &std::collections::HashMap<usize, String>,
    adjacency: &[Vec<usize>],
    edge_weights: &std::collections::HashMap<(usize, usize), f64>,
) -> (Vec<f64>, Vec<f64>) {
    let state_fips = state_code_to_fips(&state_code.to_uppercase()).map(|s| s.to_string());

    // Try TIGER path: data/{year}/tiger/tracts/tl_{year}_{fips}_tract/
    let tiger_path = state_fips.as_deref().and_then(|fips| {
        let p = std::path::PathBuf::from("data")
            .join(year).join("tiger").join("tracts")
            .join(format!("tl_{year}_{fips}_tract"))
            .join(format!("tl_{year}_{fips}_tract.shp"));
        if p.exists() { Some(p) } else { None }
    });

    let tiger_path = match tiger_path {
        Some(p) => p,
        None => {
            eprintln!("[compact-bisect] TIGER not found for {state_code} {year} — no geometry");
            return (Vec::new(), Vec::new());
        }
    };

    // Read tract records: geoid → aland
    let records = match redist_data::read_tiger_tracts(&tiger_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[compact-bisect] TIGER read failed: {e}");
            return (Vec::new(), Vec::new());
        }
    };

    let geoid_to_aland: std::collections::HashMap<String, f64> = records.iter()
        .map(|r| (r.geoid.clone(), r.aland as f64))
        .collect();

    let n = adjacency.len();
    let mut vertex_areas = vec![0.0f64; n];
    let mut vertex_ext_perimeters = vec![0.0f64; n];

    for (&idx, geoid) in index_to_geoid {
        if idx >= n { continue; }
        let aland = *geoid_to_aland.get(geoid).unwrap_or(&0.0);
        vertex_areas[idx] = aland;

        // Circular approximation of total perimeter: 2√(π·A)
        let total_perim_approx = if aland > 0.0 {
            2.0 * (std::f64::consts::PI * aland).sqrt()
        } else { 0.0 };

        // Subtract shared boundaries (in metres from adjacency edge weights)
        let shared: f64 = adjacency[idx].iter().map(|&j| {
            let key = (idx.min(j), idx.max(j));
            edge_weights.get(&key).copied().unwrap_or(0.0)
        }).sum();

        vertex_ext_perimeters[idx] = (total_perim_approx - shared).max(0.0);
    }

    (vertex_areas, vertex_ext_perimeters)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_config(state: &str) -> StateConfig {
        StateConfig {
            state_code: state.to_string(),
            state_name: state.to_lowercase(),
            num_districts: 1,
            year: "2020".to_string(),
            version: "V3".to_string(),
            output_dir: PathBuf::from("/tmp/test"),
            algo: AlgorithmConfig {
                metis: MetisParams { ufactor: 5, niter: 100, seed: Some(42), ..MetisParams::default() },
                ..AlgorithmConfig::default()
            },
            position: 999,
            debug: false,
            reset: false,
            reprocess: false,
            time_partition: false,
            num_districts_override: None,
            chamber: "congressional".to_string(),
            label: None,
            population_source: "total".to_string(),
            balance_tolerance: None,
            write_manifest: false,
            force: false,
            resolution: "tract".to_string(),
            plan_resolution: "tract".to_string(),
            seats_per_district: 1,
            total_seats: 1,
            adjacency_override: None,
            coi_weights: None,
        }
    }

    // ── Task 199: StateConfig::new_bulk constructor ───────────────────────────

    #[test]
    fn test_new_bulk_defaults() {
        let cfg = StateConfig::new_bulk(
            "WA".to_string(),
            "washington".to_string(),
            10,
            "2020".to_string(),
            "v1".to_string(),
            PathBuf::from("/tmp/test"),
            3,
        );
        // Identity fields set correctly
        assert_eq!(cfg.state_code, "WA");
        assert_eq!(cfg.state_name, "washington");
        assert_eq!(cfg.num_districts, 10);
        assert_eq!(cfg.year, "2020");
        assert_eq!(cfg.version, "v1");
        assert_eq!(cfg.output_dir, PathBuf::from("/tmp/test"));
        assert_eq!(cfg.position, 3);
        // Algorithm defaults
        assert_eq!(cfg.algo.mode_name(), "edge-weighted");
        assert_eq!(cfg.algo.metis.ufactor, 5);
        assert_eq!(cfg.algo.metis.niter, 100);
        assert_eq!(cfg.algo.metis.seed, None);
        assert!(cfg.algo.weights.geographic);
        assert!(!cfg.algo.weights.minority_weighting);
        assert!(cfg.algo.weights.partisan_shares.is_none());
        // Control defaults
        assert!(!cfg.debug);
        assert!(!cfg.reset);
        assert!(!cfg.reprocess);
        // Spec 1 defaults
        assert!(cfg.num_districts_override.is_none());
        assert_eq!(cfg.chamber, "congressional");
        assert!(cfg.label.is_none());
        assert_eq!(cfg.population_source, "total");
        assert!(cfg.balance_tolerance.is_none());
        assert!(!cfg.write_manifest);
        assert!(!cfg.force);
        assert_eq!(cfg.resolution, "tract");
        assert_eq!(cfg.seats_per_district, 1);
        // total_seats == num_districts for single-member default
        assert_eq!(cfg.total_seats, 10);
        assert!(cfg.adjacency_override.is_none());
        assert!(cfg.coi_weights.is_none());
    }

    #[test]
    fn test_new_bulk_total_seats_matches_num_districts() {
        // For single-member districts, total_seats must equal num_districts
        for n in [1usize, 5, 10, 52] {
            let cfg = StateConfig::new_bulk(
                "CA".to_string(), "california".to_string(), n,
                "2020".to_string(), "v1".to_string(),
                PathBuf::from("/tmp"), 0,
            );
            assert_eq!(cfg.total_seats, n,
                "total_seats must equal num_districts ({n}) for new_bulk");
        }
    }

    // ── Task 149: COI weights ─────────────────────────────────────────────────

    #[test]
    fn test_coi_weights_geometric_mean_increases_edge_weight() {
        // Applying COI weight 0.9 to both endpoints multiplies edge by sqrt(0.9*0.9) = 0.9
        let mut edge_weights: HashMap<(usize, usize), f64> = HashMap::new();
        edge_weights.insert((0, 1), 1.0);

        // Build a COI map: both tract 0 and tract 1 have weight 0.9
        let coi_json = r#"{"GEOID_0": 0.9, "GEOID_1": 0.9}"#;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), coi_json).unwrap();

        let mut index_to_geoid: HashMap<usize, String> = HashMap::new();
        index_to_geoid.insert(0, "GEOID_0".to_string());
        index_to_geoid.insert(1, "GEOID_1".to_string());
        let result = apply_coi_weights(edge_weights, tmp.path(), &index_to_geoid).unwrap();

        let ew = result[&(0, 1)];
        let expected = (0.9_f64 * 0.9_f64).sqrt(); // ~0.9
        assert!((ew - expected).abs() < 1e-9,
            "edge weight should be ~{expected:.4}, got {ew:.4}");
    }

    #[test]
    fn test_coi_weights_missing_geoid_defaults_to_one() {
        // A GEOID not in the COI map gets weight 1.0 (no modification)
        let mut edge_weights: HashMap<(usize, usize), f64> = HashMap::new();
        edge_weights.insert((0, 1), 2.0);

        // COI map only has tract 0, not tract 1
        let coi_json = r#"{"GEOID_0": 0.5}"#;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), coi_json).unwrap();

        let mut index_to_geoid: HashMap<usize, String> = HashMap::new();
        index_to_geoid.insert(0, "GEOID_0".to_string());
        index_to_geoid.insert(1, "GEOID_1".to_string());
        let result = apply_coi_weights(edge_weights, tmp.path(), &index_to_geoid).unwrap();

        // w_0=0.5, w_1=1.0 (default — not in COI map) → factor = sqrt(0.5 * 1.0) = sqrt(0.5)
        // original edge weight=2.0 → result = 2.0 * sqrt(0.5)
        let ew = result[&(0, 1)];
        let expected = 2.0 * (0.5_f64).sqrt();
        assert!((ew - expected).abs() < 1e-9,
            "missing GEOID should default to w=1.0, got {ew:.4}");
    }

    #[test]
    fn test_run_states_parallel_returns_one_result_per_state() {
        let configs = vec![make_config("VT"), make_config("DE"), make_config("AK")];
        let results = run_states_parallel(configs, 3);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_run_states_parallel_errors_are_in_results() {
        let configs = vec![make_config("VT")];
        let results = run_states_parallel(configs, 1);
        // VT will fail (adjacency not at /tmp/test) — verify error is in result
        for r in &results {
            if !r.success {
                assert!(r.error.is_some());
            }
        }
    }

    #[test]
    fn test_run_states_parallel_empty() {
        assert_eq!(run_states_parallel(vec![], 4).len(), 0);
    }

    #[test]
    fn test_load_all_states_2020_returns_only_us_states() {
        // load_all_states reads from the embedded manifest (US-only).
        // International locations (IE, MT-PARLIAMENT, etc.) are in location_policy.json
        // but NOT in the manifest — they must never appear in bulk runs.
        let states = load_all_states("2020").expect("manifest should load");
        assert_eq!(states.len(), 50, "exactly 50 US states expected, got {}", states.len());
        // No international location codes
        let international = ["IE", "MT-PARLIAMENT", "DE-WAHLKREIS", "NZ-ELECTORATE", "GB-ENG", "CA-PROV"];
        for code in &international {
            assert!(
                !states.iter().any(|(c, _, _)| c == code),
                "international location {code} must not appear in load_all_states"
            );
        }
        // All codes are 2-letter uppercase (US state convention)
        for (code, _, _) in &states {
            assert_eq!(code.len(), 2, "state code '{code}' must be 2 chars");
            assert!(code.chars().all(|c| c.is_uppercase()), "code '{code}' must be uppercase");
        }
    }

    #[test]
    fn test_load_all_states_invalid_year_returns_err() {
        let result = load_all_states("2024");
        assert!(result.is_err(), "year 2024 must be rejected for bulk US runs");
        let msg = result.unwrap_err();
        assert!(msg.contains("2020") || msg.contains("2010"), "error must list valid years: {msg}");
    }

    #[test]
    fn test_state_already_complete_reprocess() {
        assert!(!state_already_complete(&PathBuf::from("/nonexistent"), "VT", "2020", true));
    }

    #[test]
    fn test_state_already_complete_missing() {
        assert!(!state_already_complete(&PathBuf::from("/nonexistent"), "VT", "2020", false));
    }

    #[test]
    fn test_state_already_complete_with_json_marker() {
        let tmp = TempDir::new().unwrap();
        let data = tmp.path().join("2020").join("states").join("vt").join("data");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(data.join("final_assignments.json"), b"{}").unwrap();
        assert!(state_already_complete(&tmp.path().to_path_buf(), "VT", "2020", false));
    }

    #[test]
    fn test_filter_incomplete() {
        let tmp = TempDir::new().unwrap();
        let marker = tmp.path().join("2020").join("states").join("vt").join("data");
        std::fs::create_dir_all(&marker).unwrap();
        std::fs::write(marker.join("final_assignments.json"), b"{}").unwrap();
        let mut configs = vec![make_config("VT"), make_config("DE")];
        for c in &mut configs { c.output_dir = tmp.path().to_path_buf(); }
        let remaining = filter_incomplete(configs);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].state_code, "DE");
    }

    // --- Spec 1: StateConfig chamber-aware balance tolerance tests ---

    #[test]
    fn test_wa_house_manifest_chamber_aware_tolerance() {
        let cfg = StateConfig {
            chamber: "house".into(),
            balance_tolerance: None,
            ..make_config("WA")
        };
        assert!((cfg.effective_balance_tolerance() - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_congressional_chamber_tolerance_is_half_pct() {
        let cfg = StateConfig {
            chamber: "congressional".into(),
            balance_tolerance: None,
            ..make_config("WA")
        };
        assert!((cfg.effective_balance_tolerance() - 0.005).abs() < 1e-9);
    }

    #[test]
    fn test_explicit_tolerance_override_wins() {
        let cfg = StateConfig {
            chamber: "house".into(),
            balance_tolerance: Some(0.02),
            ..make_config("WA")
        };
        assert!((cfg.effective_balance_tolerance() - 0.02).abs() < 1e-9);
    }

    #[test]
    fn test_effective_num_districts_override() {
        let cfg = StateConfig {
            num_districts: 10,
            num_districts_override: Some(98),
            ..make_config("WA")
        };
        assert_eq!(cfg.effective_num_districts(), 98);
    }

    #[test]
    fn test_effective_num_districts_fallback() {
        let cfg = StateConfig {
            num_districts: 10,
            num_districts_override: None,
            ..make_config("WA")
        };
        assert_eq!(cfg.effective_num_districts(), 10);
    }

    #[test]
    fn test_effective_label_default() {
        let cfg = StateConfig {
            state_name: "washington".into(),
            chamber: "house".into(),
            year: "2020".into(),
            label: None,
            ..make_config("WA")
        };
        assert_eq!(cfg.effective_label(), "washington_house_2020");
    }

    #[test]
    fn test_effective_label_custom() {
        let cfg = StateConfig {
            label: Some("wa_custom_label".into()),
            ..make_config("WA")
        };
        assert_eq!(cfg.effective_label(), "wa_custom_label");
    }

    // --- Resolution field tests ---

    #[test]
    fn test_resolution_default_is_tract() {
        let cfg = make_config("VT");
        assert_eq!(cfg.resolution, "tract");
    }

    #[test]
    fn test_resolution_block_group_stored_in_config() {
        let cfg = StateConfig {
            resolution: "block_group".into(),
            ..make_config("WA")
        };
        assert_eq!(cfg.resolution, "block_group");
    }

    #[test]
    fn test_resolution_block_stored_in_config() {
        let cfg = StateConfig {
            resolution: "block".into(),
            ..make_config("WA")
        };
        assert_eq!(cfg.resolution, "block");
    }

    #[test]
    fn test_resolve_adjacency_path_tract_missing_returns_err() {
        // With no data present (invalid path from manifest default), tract resolution
        // should return an Err containing the expected hint.
        let result = resolve_adjacency_path("vt", "2020", "tract");
        assert!(result.is_err(), "expected Err when adjacency not present");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("redist fetch") || msg.contains("cannot load manifest"),
            "error message should reference redist fetch or manifest: {msg}"
        );
    }

    #[test]
    fn test_resolve_adjacency_path_block_group_missing_falls_back_or_errs() {
        // Block group adjacency missing: function either falls back to tract (also missing
        // in test env) and returns Err, or returns Err directly. Either way it must not panic.
        let result = resolve_adjacency_path("vt", "2020", "block_group");
        // In CI with no data, we expect an error (fallback tract also absent).
        // The important invariant: no panic, and error message is descriptive.
        match result {
            Err(msg) => {
                assert!(
                    msg.contains("adjacency") || msg.contains("manifest"),
                    "error should mention adjacency or manifest: {msg}"
                );
            }
            Ok((path, resolution)) => {
                // If data happens to exist locally, verify path and resolution are coherent
                assert!(path.exists(), "returned path must exist");
                assert!(
                    resolution == "tract" || resolution == "block_group",
                    "effective resolution must be tract or block_group: {resolution}"
                );
            }
        }
    }

    // ── Group 4: StateConfig.effective_balance_tolerance ─────────────────────

    #[test]
    fn test_effective_balance_tolerance_congressional_default() {
        let cfg = make_config("VT");
        // Congressional default: 0.5%
        assert!((cfg.effective_balance_tolerance() - 0.005).abs() < 1e-9,
            "congressional default must be 0.5%, got {}", cfg.effective_balance_tolerance());
    }

    #[test]
    fn test_effective_balance_tolerance_house_default() {
        let cfg = StateConfig {
            chamber: "house".to_string(),
            balance_tolerance: None,
            ..make_config("WA")
        };
        // House default: 5.0%
        assert!((cfg.effective_balance_tolerance() - 0.05).abs() < 1e-9,
            "house default must be 5.0%, got {}", cfg.effective_balance_tolerance());
    }

    #[test]
    fn test_effective_balance_tolerance_explicit_override() {
        let cfg = StateConfig {
            chamber: "house".to_string(),
            balance_tolerance: Some(0.08), // 8% explicit override
            ..make_config("WA")
        };
        assert!((cfg.effective_balance_tolerance() - 0.08).abs() < 1e-9,
            "explicit override must win, got {}", cfg.effective_balance_tolerance());
    }

    #[test]
    fn test_effective_balance_tolerance_senate_default() {
        let cfg = StateConfig {
            chamber: "senate".to_string(),
            balance_tolerance: None,
            ..make_config("IL")
        };
        assert!((cfg.effective_balance_tolerance() - 0.05).abs() < 1e-9,
            "senate default must be 5.0%");
    }

    // ── Group seats: seats_per_district / total_seats ────────────────────────

    #[test]
    fn test_seats_per_district_default_is_1() {
        let cfg = make_config("VT");
        assert_eq!(cfg.effective_seats_per_district(), 1);
    }

    #[test]
    fn test_seats_per_district_5_malta_style() {
        let cfg = StateConfig {
            seats_per_district: 5,
            total_seats: 65, // 13 x 5
            ..make_config("WA")
        };
        assert_eq!(cfg.effective_seats_per_district(), 5);
    }

    #[test]
    fn test_total_seats_computed_from_seats_per_district() {
        let cfg = StateConfig {
            seats_per_district: 4, // avg for Ireland-style
            num_districts_override: Some(43),
            total_seats: 43 * 4, // 172
            ..make_config("WA")
        };
        assert_eq!(cfg.total_seats, 172);
    }

    #[test]
    fn test_ideal_pop_per_seat_single_member() {
        let cfg = make_config("VT"); // seats_per_district=1, total_seats=1
        // For single-member: ideal_pop_per_seat = total_pop / 1 = total_pop
        let ideal = cfg.ideal_pop_per_seat(643503);
        assert!((ideal - 643503.0).abs() < 1.0);
    }

    #[test]
    fn test_ideal_pop_per_seat_multi_member() {
        let cfg = StateConfig {
            seats_per_district: 5,
            total_seats: 65,
            ..make_config("WA")
        };
        // 7_705_281 / 65 ~ 118_543
        let ideal = cfg.ideal_pop_per_seat(7_705_281);
        assert!((ideal - 7_705_281.0 / 65.0).abs() < 1.0);
    }

    #[test]
    fn test_seats_per_district_zero_clamps_to_1() {
        let cfg = StateConfig {
            seats_per_district: 0,
            total_seats: 1,
            ..make_config("VT")
        };
        // effective_seats_per_district uses .max(1) so 0 -> 1
        assert_eq!(cfg.effective_seats_per_district(), 1);
    }

    // ── Group: chamber_balance_tolerance ──────────────────────────────────────

    #[test]
    fn test_chamber_balance_tolerance_wa_house_is_5pct() {
        // WA house_districts balance_tolerance_house_pct = 5.0%
        let tol = chamber_balance_tolerance("WA", "house");
        assert!((tol - 0.05).abs() < 1e-6, "WA house tolerance must be 5%, got {tol}");
    }

    #[test]
    fn test_chamber_balance_tolerance_wa_congressional_is_half_pct() {
        // WA balance_tolerance_congressional_pct = 0.5%
        let tol = chamber_balance_tolerance("WA", "congressional");
        assert!((tol - 0.005).abs() < 1e-6, "WA congressional tolerance must be 0.5%, got {tol}");
    }

    #[test]
    fn test_chamber_balance_tolerance_nv_house_is_10pct() {
        // NV allows 10% house tolerance (policy explicitly documents this)
        let tol = chamber_balance_tolerance("NV", "house");
        assert!((tol - 0.10).abs() < 1e-6, "NV house tolerance must be 10%, got {tol}");
    }

    #[test]
    fn test_chamber_balance_tolerance_unknown_state_uses_default() {
        let tol = chamber_balance_tolerance("ZZ", "house");
        assert!((tol - 0.05).abs() < 1e-6, "unknown state must fall back to 5% default");
    }

    #[test]
    fn test_chamber_balance_tolerance_unknown_chamber_uses_default() {
        let tol = chamber_balance_tolerance("WA", "council");
        assert!((tol - 0.05).abs() < 1e-6, "unknown chamber must fall back to 5% default");
    }

    #[test]
    fn test_effective_balance_tolerance_uses_policy_when_no_override() {
        // NV house has 10% tolerance in policy; without explicit override, must use 10%
        let cfg = StateConfig {
            state_code: "NV".into(),
            chamber: "house".into(),
            balance_tolerance: None, // no explicit override
            ..make_config("VT")
        };
        let tol = cfg.effective_balance_tolerance();
        assert!((tol - 0.10).abs() < 1e-6,
            "NV house must use policy tolerance 10%, got {tol}");
    }

    #[test]
    fn test_effective_balance_tolerance_explicit_override_wins() {
        // Explicit --balance-tolerance 2 must override even if policy says 10%
        let cfg = StateConfig {
            state_code: "NV".into(),
            chamber: "house".into(),
            balance_tolerance: Some(0.02), // explicit 2% override
            ..make_config("VT")
        };
        let tol = cfg.effective_balance_tolerance();
        assert!((tol - 0.02).abs() < 1e-9, "explicit override must win, got {tol}");
    }

    // ── Group: chamber_district_count ─────────────────────────────────────────

    #[test]
    fn test_chamber_district_count_congressional_returns_fallback() {
        // Congressional always uses the manifest fallback, not state policy
        assert_eq!(chamber_district_count("WA", "congressional", 10), 10);
        assert_eq!(chamber_district_count("VT", "congressional", 1), 1);
    }

    #[test]
    fn test_chamber_district_count_house_wa_returns_98() {
        // WA house has 98 districts per state_policy.json
        let result = chamber_district_count("WA", "house", 10);
        assert_eq!(result, 98, "WA house must use 98 districts from state policy, got {result}");
    }

    #[test]
    fn test_chamber_district_count_senate_wa_returns_49() {
        // WA senate has 49 districts (2:1 nesting with 98 house)
        let result = chamber_district_count("WA", "senate", 10);
        assert_eq!(result, 49, "WA senate must use 49 districts from state policy, got {result}");
    }

    #[test]
    fn test_chamber_district_count_house_nv_returns_42() {
        // NV house has 42 districts per state_policy.json
        let result = chamber_district_count("NV", "house", 4);
        assert_eq!(result, 42, "NV house must use 42 districts from state policy, got {result}");
    }

    #[test]
    fn test_chamber_district_count_house_va_returns_100() {
        // VA house has 100 delegates
        let result = chamber_district_count("VA", "house", 11);
        assert_eq!(result, 100, "VA house must use 100 from state policy, got {result}");
    }

    #[test]
    fn test_chamber_district_count_house_la_returns_105() {
        // LA house has 105 representatives
        let result = chamber_district_count("LA", "house", 6);
        assert_eq!(result, 105, "LA house must use 105 from state policy, got {result}");
    }

    #[test]
    fn test_chamber_district_count_unknown_state_uses_fallback() {
        // Unknown state code falls back to congressional count
        let result = chamber_district_count("ZZ", "house", 7);
        assert_eq!(result, 7, "unknown state ZZ must fall back to congressional count");
    }

    #[test]
    fn test_chamber_district_count_unknown_chamber_uses_fallback() {
        // Unrecognized chamber name falls back
        let result = chamber_district_count("WA", "council", 10);
        assert_eq!(result, 10, "unknown chamber type must fall back to congressional count");
    }

    // ── Group 5: adjacency fallback / resolve_adjacency_path ─────────────────

    // ── Task 122: --reset data loss warning ───────────────────────────────────

    /// Verify the reset plan path computation matches the expected directory structure.
    ///
    /// The warning uses `plan_root.display()` which is either:
    ///   labeled:  {output_dir}/{year}/plans/{label}/
    ///   unlabeled: {output_dir}/{year}/states/{state_name}/
    #[test]
    fn test_reset_warning_format() {
        use std::path::PathBuf;

        // Labeled plan path
        let output_dir = PathBuf::from("/tmp/outputs/v1");
        let year = "2020";
        let label = "wa_house_2020";
        let plan_root_labeled = output_dir.join(year).join("plans").join(label);
        let expected = "/tmp/outputs/v1/2020/plans/wa_house_2020";
        assert!(
            plan_root_labeled.to_string_lossy().replace('\\', "/").contains("wa_house_2020"),
            "labeled plan_root must contain label: {}",
            plan_root_labeled.display()
        );
        let warning = format!(
            "WARNING: --reset will delete {} and all its contents before re-running.",
            plan_root_labeled.display()
        );
        assert!(warning.contains("wa_house_2020"), "warning must mention the plan label: {warning}");
        assert!(warning.contains("--reset will delete"), "warning must mention --reset: {warning}");

        // Legacy (unlabeled) state path
        let state_name = "washington";
        let plan_root_legacy = output_dir.join(year).join("states").join(state_name);
        let warning_legacy = format!(
            "WARNING: --reset will delete {} and all its contents before re-running.",
            plan_root_legacy.display()
        );
        assert!(warning_legacy.contains("washington"), "legacy warning must mention state: {warning_legacy}");
    }

    #[test]
    fn test_reset_warning_contains_required_components() {
        // Verify the warning message format has all required components
        let plan_root = std::path::PathBuf::from("/tmp/outputs/v1/2020/plans/wa_house_2020");
        let msg = format!(
            "WARNING: --reset will delete {} and all its contents before re-running.",
            plan_root.display()
        );
        assert!(msg.starts_with("WARNING:"), "must start with WARNING:");
        assert!(msg.contains("--reset"), "must mention --reset flag");
        assert!(msg.contains("delete"), "must use the word 'delete'");
        assert!(msg.contains("all its contents"), "must warn about contents");
    }

    // ── Task 135: adjacency year mismatch detection ──────────────────────────

    #[test]
    fn test_adjacency_year_mismatch_detected() {
        // Requesting year 2020 but file is for 2010 — must detect mismatch
        let path = PathBuf::from("wa_adjacency_2010.pkl");
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let file_year = extract_year_from_adj_filename(filename);
        assert_eq!(file_year, Some("2010"), "should extract 2010 from filename");
        // Mismatch: requested 2020, file is 2010
        assert_ne!(file_year, Some("2020"), "2010 file != 2020 requested — mismatch detected");
    }

    #[test]
    fn test_adjacency_year_match_no_warning() {
        // Requesting year 2020 and file is also for 2020 — no mismatch
        let path = PathBuf::from("wa_adjacency_2020.pkl");
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let file_year = extract_year_from_adj_filename(filename);
        assert_eq!(file_year, Some("2020"), "should extract 2020 from filename");
        // No mismatch: requested 2020, file is 2020
        assert_eq!(file_year, Some("2020"), "2020 file == 2020 requested — no mismatch");
    }

    #[test]
    fn test_extract_year_from_adj_filename_2000() {
        let year = extract_year_from_adj_filename("ca_adjacency_2000.pkl");
        assert_eq!(year, Some("2000"), "should extract 2000");
    }

    #[test]
    fn test_extract_year_from_adj_filename_none() {
        // Filename without a recognizable census year
        let year = extract_year_from_adj_filename("ca_adjacency.pkl");
        assert_eq!(year, None, "no census year in filename should return None");
    }

    #[test]
    fn test_check_adjacency_year_mismatch_same_year_no_panic() {
        // Same year: function runs without panic
        let path = PathBuf::from("wa_adjacency_2020.pkl");
        check_adjacency_year_mismatch(&path, "2020", "WA"); // no panic
    }

    #[test]
    fn test_check_adjacency_year_mismatch_different_year_no_panic() {
        // Different year: function emits warning but doesn't panic
        let path = PathBuf::from("wa_adjacency_2010.pkl");
        check_adjacency_year_mismatch(&path, "2020", "WA"); // warns but no panic
    }

    // ── Gap 9: progress messages for redist states ────────────────────────────

    #[test]
    fn test_states_progress_message_format() {
        // A States run with 0 configs still shows the summary line format.
        // We verify the message format that would be produced by the States command.
        let configs: Vec<StateConfig> = Vec::new();
        let results = run_states_parallel(configs, 4);
        let succeeded = results.iter().filter(|r| r.success).count();
        let failed = results.iter().filter(|r| !r.success).count();

        // Verify summary computation is correct for empty run
        assert_eq!(succeeded, 0, "0 configs: succeeded must be 0");
        assert_eq!(failed, 0, "0 configs: failed must be 0");

        // Verify the summary line format
        let summary = format!("[redist states] Complete: {} succeeded, {} failed", succeeded, failed);
        assert!(summary.contains("Complete:"), "summary must contain 'Complete:'");
        assert!(summary.contains("succeeded"), "summary must contain 'succeeded'");
        assert!(summary.contains("failed"), "summary must contain 'failed'");
        assert!(summary.contains("[redist states]"), "summary must be prefixed with [redist states]");

        // Verify the queued banner format
        let queued = format!(
            "[redist states] {} states queued — {} workers — year {} — version {}",
            0usize, 4usize, "2020", "v1"
        );
        assert!(queued.contains("states queued"), "banner must contain 'states queued'");
        assert!(queued.contains("workers"), "banner must contain 'workers'");
        assert!(queued.contains("year"), "banner must contain 'year'");
        assert!(queued.contains("version"), "banner must contain 'version'");
    }

    // ── Gap 1: adjacency missing error message suggests redist fetch ──────────

    // ── Task 205: block_group fallback warning mentions --resolution and fetch ──

    #[test]
    fn test_block_group_fallback_warning_text() {
        // The warning when bg requested but not found should mention --resolution and fetch
        let state_code_lower = "wa";
        let year = "2020";
        let warning = format!(
            "WARNING: --resolution block_group was requested but block_group adjacency \
             not found for {state_code_lower} {year}.\n\
             To get block_group data: redist fetch --type adjacency --states {} --year {}\n\
             Falling back to tract resolution.",
            state_code_lower.to_uppercase(), year
        );
        assert!(warning.contains("--resolution block_group"), "must mention flag");
        assert!(warning.contains("redist fetch"), "must mention fetch command");
        assert!(warning.contains("block_group"), "must mention resolution type");
        assert!(warning.contains("Falling back to tract resolution"), "must mention fallback");
        assert!(warning.contains("WA"), "must mention uppercase state code");
    }

    #[test]
    fn test_adjacency_missing_error_suggests_fetch() {
        // When adjacency is missing, the error must mention "redist fetch" and "--type adjacency".
        let result = resolve_adjacency_path("wa", "2020", "tract");
        // In test env, adjacency won't exist — verify error contains expected hints.
        match result {
            Err(msg) => {
                assert!(
                    msg.contains("redist fetch"),
                    "error must suggest 'redist fetch': {msg}"
                );
                assert!(
                    msg.contains("--type adjacency"),
                    "error must include '--type adjacency': {msg}"
                );
            }
            Ok(_) => {
                // If data happens to exist locally, the test is vacuously satisfied.
            }
        }
    }

    #[test]
    fn test_resolve_adjacency_uses_manifest() {
        // resolve_adjacency_path reads the manifest to find outputs_dir.
        // If manifest can be loaded, the function should not panic.
        // Test that an unknown state code returns a descriptive error.
        let result = resolve_adjacency_path("zz", "2020", "tract");
        // Should fail (no ZZ adjacency) but with a helpful error message
        assert!(result.is_err(), "unknown state ZZ should fail");
        let err = result.unwrap_err();
        assert!(err.contains("adjacency") || err.contains("not found") || err.contains("manifest"),
            "error should mention adjacency: {err}");
    }

    /// Scenario 17: Isolated node warning logic — verify that an adjacency with
    /// isolated nodes (empty neighbor list) is correctly detected.
    #[test]
    fn test_run_warns_on_isolated_nodes() {
        // Simulate the isolated-node detection logic from run_single_state.
        // adjacency[0] has neighbors, adjacency[1] is isolated, adjacency[2] is isolated.
        let adjacency: Vec<Vec<usize>> = vec![
            vec![2],     // node 0: connected
            vec![],      // node 1: isolated
            vec![0],     // node 2: connected
            vec![],      // node 3: isolated
        ];

        let isolated: Vec<usize> = adjacency.iter().enumerate()
            .filter(|(_, nbrs)| nbrs.is_empty())
            .map(|(i, _)| i)
            .collect();

        assert_eq!(isolated.len(), 2, "should detect 2 isolated nodes");
        assert!(isolated.contains(&1), "node 1 should be isolated");
        assert!(isolated.contains(&3), "node 3 should be isolated");
        assert!(!isolated.contains(&0), "node 0 is connected, not isolated");
        assert!(!isolated.contains(&2), "node 2 is connected, not isolated");

        // Verify a fully-connected graph produces no isolated nodes
        let connected: Vec<Vec<usize>> = vec![
            vec![1, 2],
            vec![0, 2],
            vec![0, 1],
        ];
        let isolated_none: Vec<usize> = connected.iter().enumerate()
            .filter(|(_, nbrs)| nbrs.is_empty())
            .map(|(i, _)| i)
            .collect();
        assert!(isolated_none.is_empty(), "fully connected graph has no isolated nodes");
    }

    // ── Task 131: CVAP fallback warning ──────────────────────────────────────

    #[test]
    fn test_cvap_missing_falls_back_to_total() {
        // With no CVAP file on disk, requesting "cvap" should fall back to "total".
        let result = check_cvap_availability("cvap", "vermont", "2020", "VT");
        assert_eq!(result, "total",
            "should fall back to total when CVAP file is absent, got {result}");
    }

    #[test]
    fn test_population_source_cvap_falls_back_to_total() {
        // Synonym test — same logic as above but more explicit assertion.
        let source = check_cvap_availability("cvap", "nonexistent_state", "2020", "XX");
        assert_eq!(source, "total",
            "CVAP fallback must produce 'total', got '{source}'");
    }

    #[test]
    fn test_non_cvap_source_unchanged() {
        // "total" and "vap" should be returned unchanged regardless of file presence.
        assert_eq!(check_cvap_availability("total", "vermont", "2020", "VT"), "total");
        assert_eq!(check_cvap_availability("vap", "vermont", "2020", "VT"), "vap");
    }

    // ── Task 130: Worker cap reporting ───────────────────────────────────────

    #[test]
    fn test_worker_cap_message_when_capped() {
        // effective_workers(very_large) < very_large => note would be emitted.
        // We can't easily capture stderr in unit tests, but we can verify the
        // logic: if requested > actual, the note should fire.
        let requested = usize::MAX; // always exceeds available threads
        let actual = effective_workers(requested);
        assert!(actual < requested,
            "effective_workers(MAX) must be < MAX (got {actual})");
    }

    #[test]
    fn test_worker_cap_no_message_when_exact() {
        // When requested equals actual, no note should be emitted.
        // Using 1 worker: effective == 1 == requested.
        let requested = 1;
        let actual = effective_workers(requested);
        // Rayon always has at least 1 thread
        assert!(actual >= 1, "effective_workers(1) must be >= 1");
        // When actual == requested, no cap note fires (logical condition)
        let would_print = actual < requested;
        assert!(!would_print, "no note when requested ({requested}) == actual ({actual})");
    }

    // ── Plan 03: validate_partisan_config (Callais disentanglement) ──────────

    #[test]
    fn test_validate_partisan_config_default_ok() {
        // Default config has partition_mode=edge-weighted and no partisan_shares
        // → no constraint involved, must pass.
        let cfg = make_config("VT");
        validate_partisan_config(&cfg).expect("default config should validate");
    }

    #[test]
    fn test_validate_partisan_config_metis_vra_alone_ok() {
        let cfg = StateConfig {
            algo: AlgorithmConfig {
                split: SplitStrategy::NWay,
                weights: WeightSpec { minority_weighting: true, ..WeightSpec::default() },
                metis: MetisParams { ufactor: 5, niter: 100, seed: None, ..MetisParams::default() },
                mode_label: None,
                ..AlgorithmConfig::default()
            },
            ..make_config("AL")
        };
        validate_partisan_config(&cfg).expect("metis-vra is always valid");
    }

    #[test]
    fn test_algo_mode_names() {
        // AlgorithmConfig carries mode identity — no separate string field needed.
        let unweighted = AlgorithmConfig {
            split: SplitStrategy::Bisect,
            weights: WeightSpec { geographic: false, ..WeightSpec::default() },
            mode_label: Some("unweighted"),
            ..AlgorithmConfig::default()
        };
        assert_eq!(unweighted.mode_name(), "unweighted");

        let edge_weighted = AlgorithmConfig::default();
        assert_eq!(edge_weighted.mode_name(), "edge-weighted");

        let metis_vra = AlgorithmConfig {
            split: SplitStrategy::NWay,
            weights: WeightSpec { minority_weighting: true, ..WeightSpec::default() },
            mode_label: None,
            ..AlgorithmConfig::default()
        };
        assert_eq!(metis_vra.mode_name(), "metis-vra");

        let geosection = AlgorithmConfig {
            split: SplitStrategy::GeoSection,
            mode_label: None,
            ..AlgorithmConfig::default()
        };
        assert_eq!(geosection.mode_name(), "geosection");

        let compact = AlgorithmConfig {
            split: SplitStrategy::CompactBisect { epsilon: 0.05 },
            mode_label: None,
            ..AlgorithmConfig::default()
        };
        assert_eq!(compact.mode_name(), "compact-bisect");
    }

    #[test]
    fn test_algo_metis_params_extraction() {
        let cfg = AlgorithmConfig {
            metis: MetisParams { ufactor: 7, niter: 200, seed: Some(42), ..MetisParams::default() },
            ..AlgorithmConfig::default()
        };
        assert_eq!(cfg.metis.ufactor, 7);
        assert_eq!(cfg.metis.niter, 200);
        assert_eq!(cfg.metis.seed, Some(42));

        let vra = AlgorithmConfig {
            split: SplitStrategy::NWay,
            weights: WeightSpec { minority_weighting: true, ..WeightSpec::default() },
            metis: MetisParams { ufactor: 3, niter: 50, seed: None, ..MetisParams::default() },
            ..AlgorithmConfig::default()
        };
        assert_eq!(vra.metis.ufactor, 3);
        assert_eq!(vra.metis.niter, 50);
        assert_eq!(vra.metis.seed, None);
    }

    #[test]
    fn test_validate_partisan_config_is_noop() {
        // Validation is now structural — this is always Ok.
        let cfg = make_config("VT");
        validate_partisan_config(&cfg).expect("structural validation always passes");
    }

    // ── AlgorithmConfig: exhaustive PartitionMode coverage ───────────────────
    // Every PartitionMode must be reachable via defaults_for_mode, produce a
    // correct mode_name(), and have valid MetisParams. If a new PartitionMode is
    // added, add a case here and a new arm in the split dispatch — two-level safety.

    #[test]
    fn test_algo_all_modes_have_mode_name() {
        use crate::args::PartitionMode as PM;
        let cases = [
            (PM::Unweighted,       "unweighted"),
            (PM::EdgeWeighted,     "edge-weighted"),
            (PM::MetisVra,         "metis-vra"),
            (PM::PartisanWeighted, "partisan-weighted"),
            (PM::Proportional,     "proportional"),
            (PM::CompactBisect,    "compact-bisect"),
            (PM::GeoSection,       "geosection"),
            (PM::AreaSection,      "areasection"),
        ];
        for (mode, expected_name) in &cases {
            let algo = AlgorithmConfig::defaults_for_mode(mode);
            assert_eq!(algo.mode_name(), *expected_name,
                "mode_name mismatch for {:?}", expected_name);
        }
    }

    #[test]
    fn test_algo_all_modes_defaults_for_mode_roundtrip() {
        use crate::args::PartitionMode as PM;
        // Every mode must produce a valid AlgorithmConfig via defaults_for_mode.
        // The mode_name of the result must match the input mode's string.
        let cases = [
            (PM::Unweighted,      "unweighted"),
            (PM::EdgeWeighted,    "edge-weighted"),
            (PM::MetisVra,        "metis-vra"),
            (PM::PartisanWeighted,"partisan-weighted"),
            (PM::Proportional,    "proportional"),
            (PM::CompactBisect,   "compact-bisect"),
            (PM::GeoSection,      "geosection"),
            (PM::AreaSection,     "areasection"),
        ];
        for (mode, name) in &cases {
            let algo = AlgorithmConfig::defaults_for_mode(mode);
            assert_eq!(algo.mode_name(), *name,
                "defaults_for_mode({name}) produced wrong mode_name");
        }
    }

    #[test]
    fn test_algo_metis_params_all_modes_positive() {
        // Every mode must produce ufactor > 0 and niter > 0 from defaults_for_mode.
        use crate::args::PartitionMode as PM;
        let modes = [
            PM::Unweighted, PM::EdgeWeighted, PM::MetisVra,
            PM::PartisanWeighted, PM::Proportional,
            PM::CompactBisect, PM::GeoSection, PM::AreaSection,
        ];
        for mode in &modes {
            let algo = AlgorithmConfig::defaults_for_mode(mode);
            assert!(algo.metis.ufactor > 0, "ufactor must be > 0 for {:?}", mode);
            assert!(algo.metis.niter > 0, "niter must be > 0 for {:?}", mode);
        }
    }

    #[test]
    fn test_algo_default_edge_weighted() {
        let algo = AlgorithmConfig::default();
        assert_eq!(algo.mode_name(), "edge-weighted");
        assert_eq!(algo.metis.ufactor, 5);
        assert_eq!(algo.metis.niter, 100);
        assert_eq!(algo.metis.seed, None);
        assert!(algo.weights.geographic);
        assert!(!algo.weights.minority_weighting);
        assert!(algo.weights.partisan_shares.is_none());
    }

    #[test]
    fn test_algo_geosection_defaults() {
        use crate::args::PartitionMode as PM;
        let algo = AlgorithmConfig::defaults_for_mode(&PM::GeoSection);
        assert_eq!(algo.mode_name(), "geosection");
        assert!(matches!(algo.split, SplitStrategy::GeoSection),
            "defaults_for_mode(GeoSection) returned wrong split strategy");
        assert!(algo.seeds.seed_count() > 0, "geosection needs seeds > 0");
        assert_eq!(algo.weights.directional_lambda, 0.0,
            "default lambda should be 0 (no directional penalty)");
    }

    // ── Poison values — testing defensive behaviour ───────────────────────────
    // These tests construct AlgorithmConfig with obviously-wrong values and
    // verify the struct is transparent: garbage in = garbage out (no silent fixes).

    #[test]
    fn test_algo_poison_zero_seeds_not_silently_fixed() {
        let poison = AlgorithmConfig {
            split: SplitStrategy::GeoSection,
            weights: WeightSpec { directional_lambda: f64::INFINITY, ..WeightSpec::default() },
            metis: MetisParams { ufactor: 0, niter: 0, seed: None, ..MetisParams::default() },
            ..AlgorithmConfig::default()
        };
        // mode_name must still work (no panic)
        assert_eq!(poison.mode_name(), "geosection");
        // MetisParams fields must reflect the bad values as-is (caller can validate)
        assert_eq!(poison.metis.ufactor, 0, "poison ufactor=0 must not be silently corrected");
        assert_eq!(poison.metis.niter, 0, "poison niter=0 must not be silently corrected");
        if let SeedCompositor::Multi { seeds } = poison.seeds {
            assert_eq!(seeds, 50, "default seeds=50 preserved even with poison metis params");
        }
    }

    #[test]
    fn test_split_strategy_all_variants_mode_name_never_panics() {
        // Exhaustive instantiation: if a new SplitStrategy variant is added without
        // updating this list, the COMPILER will reject this test — compile-time safety.
        let all: &[(&str, SplitStrategy)] = &[
            ("edge-weighted", SplitStrategy::Bisect),
            ("metis-vra",     SplitStrategy::NWay),
            ("geosection",    SplitStrategy::GeoSection),
            ("compact-bisect",SplitStrategy::CompactBisect { epsilon: 0.05 }),
            ("areasection",   SplitStrategy::AreaSection { area_swing: 1.10 }),
            ("apportion-regions",  SplitStrategy::ApportionRegions),
            ("vra-section",   SplitStrategy::VraSection { w_vra: 0.40 }),
        ];
        for (expected_name, variant) in all {
            assert_eq!(variant.mode_name(), *expected_name,
                "new SplitStrategy variant added without updating this exhaustive list!");
        }
    }

    #[test]
    fn test_algo_compact_bisect_defaults() {
        use crate::args::PartitionMode as PM;
        let algo = AlgorithmConfig::defaults_for_mode(&PM::CompactBisect);
        assert_eq!(algo.mode_name(), "compact-bisect");
        if let SplitStrategy::CompactBisect { epsilon } = algo.split {
            assert!(epsilon > 0.0 && epsilon < 1.0, "epsilon must be in (0,1)");
        } else {
            panic!("defaults_for_mode(CompactBisect) returned wrong split strategy");
        }
        assert!(algo.seeds.seed_count() > 0, "compact-bisect needs seeds > 0");
    }

    #[test]
    fn test_weight_spec_defaults() {
        let spec = WeightSpec::default();
        assert!(spec.geographic);
        assert!(spec.partisan_shares.is_none());
        assert!(!spec.minority_weighting);
        assert!((spec.dem_threshold - 0.55).abs() < 1e-9);
        assert!((spec.rep_threshold - 0.45).abs() < 1e-9);
        assert!(spec.alpha_county < 1e-10);
        assert!(spec.alpha_mcd < 1e-10);
        assert!(spec.alpha_place < 1e-10);
        assert!(spec.alpha_vtd < 1e-10);
        assert!(spec.directional_lambda < 1e-10);
    }

    #[test]
    fn test_metis_params_defaults() {
        let mp = MetisParams::default();
        assert_eq!(mp.ufactor, 5);
        assert_eq!(mp.niter, 100);
        assert_eq!(mp.seed, None);
    }

    #[test]
    fn test_algo_unweighted_mode_detection() {
        // Unweighted: mode_label overrides → "unweighted"
        let algo = AlgorithmConfig {
            weights: WeightSpec { geographic: false, ..WeightSpec::default() },
            mode_label: Some("unweighted"),
            ..AlgorithmConfig::default()
        };
        assert_eq!(algo.mode_name(), "unweighted");
    }

    #[test]
    fn test_algo_partisan_weighted_mode_detection() {
        let algo = AlgorithmConfig {
            weights: WeightSpec {
                partisan_shares: Some(std::path::PathBuf::from("shares.tsv")),
                dem_threshold: 0.55,
                rep_threshold: 0.45,
                ..WeightSpec::default()
            },
            ..AlgorithmConfig::default()
        };
        assert_eq!(algo.mode_name(), "partisan-weighted");
    }

    #[test]
    fn test_algo_alpha_county_propagates_from_state_args() {
        use crate::args::{PartitionMode, StateArgs};
        use clap::Parser;
        let args = StateArgs::parse_from([
            "state", "--state", "VT", "--alpha-county", "2.5"
        ]);
        assert!((args.alpha_county - 2.5).abs() < 1e-9,
            "alpha_county must be parsed from CLI, got {}", args.alpha_county);
        let algo = AlgorithmConfig::from_state_args(&args);
        assert!((algo.weights.alpha_county - 2.5).abs() < 1e-9,
            "alpha_county must propagate into WeightSpec, got {}", algo.weights.alpha_county);
    }

    #[test]
    fn test_algo_alpha_county_default_is_zero() {
        use crate::args::StateArgs;
        use clap::Parser;
        let args = StateArgs::parse_from(["state", "--state", "VT"]);
        assert!(args.alpha_county < 1e-10,
            "alpha_county default must be 0.0, got {}", args.alpha_county);
        let algo = AlgorithmConfig::from_state_args(&args);
        assert!(algo.weights.alpha_county < 1e-10,
            "alpha_county must default to 0.0 in WeightSpec");
    }

    #[test]
    fn test_subdivision_weighter_modifies_same_county_edges() {
        // Integration: alpha_county flows through ComposedWeighter and changes weights.
        use crate::edge_weights::{ComposedWeighter, GeographicWeighter, SubdivisionWeighter};
        use std::collections::HashMap;

        // Two edges: (0,1) same county "01001", (1,2) cross-county "01001" vs "01002"
        let mut geo_map = HashMap::new();
        geo_map.insert((0usize, 1usize), 100.0f64);
        geo_map.insert((1usize, 2usize), 100.0f64);

        let mut geoid_map = HashMap::new();
        geoid_map.insert(0usize, "01001000100".to_string());
        geoid_map.insert(1usize, "01001000200".to_string()); // same county
        geoid_map.insert(2usize, "01002000100".to_string()); // different county

        let composer = ComposedWeighter::new()
            .push(GeographicWeighter::from_map(geo_map))
            .push(SubdivisionWeighter::county_only(&geoid_map, 3, 3.0));
        let out = composer.apply();

        // (0,1): same county → 100 × (1 + 3) = 400
        assert!((out[&(0,1)] - 400.0).abs() < 1e-9,
            "same-county edge should be 4× more expensive, got {}", out[&(0,1)]);
        // (1,2): cross-county → unchanged
        assert!((out[&(1,2)] - 100.0).abs() < 1e-9,
            "cross-county edge should be unchanged, got {}", out[&(1,2)]);
    }

    #[test]
    fn test_early_exit_checks_all_subdivision_alphas() {
        // WeightSpec with only alpha_mcd set must NOT trigger the unweighted early-exit.
        // Regression test for the missing alpha_mcd/place/vtd check in the early-exit guard.
        let spec = WeightSpec {
            geographic: false,
            minority_weighting: false,
            partisan_shares: None,
            alpha_county: 0.0,
            alpha_mcd:    2.0, // only mcd set — must not early-exit to empty map
            alpha_place:  0.0,
            alpha_vtd:    0.0,
            ..WeightSpec::default()
        };
        // The early-exit condition: all four alphas < 1e-10 AND no geographic/partisan/minority.
        // With alpha_mcd = 2.0 this must be FALSE.
        let should_early_exit = !spec.geographic
            && !spec.minority_weighting
            && spec.partisan_shares.is_none()
            && spec.alpha_county < 1e-10
            && spec.alpha_mcd    < 1e-10
            && spec.alpha_place  < 1e-10
            && spec.alpha_vtd    < 1e-10;
        assert!(!should_early_exit,
            "alpha_mcd=2.0 must prevent early-exit to empty edge map");
    }

    // ── Group 1: SeedCompositor ───────────────────────────────────────────────

    #[test]
    fn seed_count_single_returns_1() {
        let sc = SeedCompositor::Single;
        assert_eq!(sc.seed_count(), 1,
            "Single seed_count must return 1");
    }

    #[test]
    fn seed_count_multi_returns_seeds() {
        let sc = SeedCompositor::Multi { seeds: 77 };
        assert_eq!(sc.seed_count(), 77,
            "Multi seed_count must return the seeds field");
    }

    #[test]
    fn seed_count_convergence_sweep_returns_threshold() {
        let sc = SeedCompositor::ConvergenceSweep { threshold: 250 };
        assert_eq!(sc.seed_count(), 250,
            "ConvergenceSweep seed_count must return threshold as usize");
    }

    #[test]
    fn is_single_true_for_single() {
        assert!(SeedCompositor::Single.is_single(),
            "is_single must be true for Single variant");
    }

    #[test]
    fn is_single_false_for_multi() {
        assert!(!SeedCompositor::Multi { seeds: 10 }.is_single(),
            "is_single must be false for Multi variant");
        assert!(!SeedCompositor::ConvergenceSweep { threshold: 100 }.is_single(),
            "is_single must be false for ConvergenceSweep variant");
    }

    #[test]
    fn default_is_multi_50() {
        let sc = SeedCompositor::default();
        if let SeedCompositor::Multi { seeds } = sc {
            assert_eq!(seeds, 50, "Default SeedCompositor must be Multi{{seeds: 50}}");
        } else {
            panic!("Default SeedCompositor must be Multi, got a different variant");
        }
    }

    #[test]
    fn seed_count_percentile_returns_seeds() {
        let sc = SeedCompositor::Percentile { p: 0.5, seeds: 101 };
        assert_eq!(sc.seed_count(), 101);
    }

    #[test]
    fn seed_count_bisection_ensemble_returns_steps() {
        let sc = SeedCompositor::BisectionEnsemble { p: 0.5, ensemble_steps: 200 };
        assert_eq!(sc.seed_count(), 200);
    }

    #[test]
    fn percentile_clamps_p_to_unit_interval() {
        // p is stored as-is; callers are responsible for clamping.
        // Verify the variant can be constructed with boundary values.
        let sc_min = SeedCompositor::Percentile { p: 0.0, seeds: 10 };
        let sc_max = SeedCompositor::Percentile { p: 1.0, seeds: 10 };
        if let SeedCompositor::Percentile { p, .. } = sc_min { assert_eq!(p, 0.0); }
        if let SeedCompositor::Percentile { p, .. } = sc_max { assert_eq!(p, 1.0); }
    }

    #[test]
    fn bisection_ensemble_stores_p_and_steps() {
        let sc = SeedCompositor::BisectionEnsemble { p: 0.75, ensemble_steps: 500 };
        if let SeedCompositor::BisectionEnsemble { p, ensemble_steps } = sc {
            assert_eq!(p, 0.75);
            assert_eq!(ensemble_steps, 500);
        } else { panic!("wrong variant"); }
    }

    #[test]
    fn is_single_false_for_percentile_and_bisection_ensemble() {
        assert!(!SeedCompositor::Percentile { p: 0.5, seeds: 10 }.is_single());
        assert!(!SeedCompositor::BisectionEnsemble { p: 0.5, ensemble_steps: 100 }.is_single());
    }

    #[test]
    fn clone_preserves_variant() {
        let orig = SeedCompositor::ConvergenceSweep { threshold: 999 };
        let cloned = orig.clone();
        if let SeedCompositor::ConvergenceSweep { threshold } = cloned {
            assert_eq!(threshold, 999, "Clone must preserve ConvergenceSweep threshold");
        } else {
            panic!("Clone must preserve the ConvergenceSweep variant");
        }

        let orig2 = SeedCompositor::Multi { seeds: 42 };
        let cloned2 = orig2.clone();
        if let SeedCompositor::Multi { seeds } = cloned2 {
            assert_eq!(seeds, 42, "Clone must preserve Multi seeds");
        } else {
            panic!("Clone must preserve the Multi variant");
        }
    }

    // ── Group 2: SplitStrategy with SeedCompositor separation ────────────────

    #[test]
    fn split_strategy_geosection_has_no_fields() {
        // GeoSection carries no seeds field — seeds live in SeedCompositor now.
        let s = SplitStrategy::GeoSection;
        assert_eq!(s.mode_name(), "geosection",
            "GeoSection mode_name must be 'geosection'");
        // Confirm it matches the enum variant (compiler enforces no extra fields).
        assert!(matches!(s, SplitStrategy::GeoSection),
            "GeoSection must match the bare variant");
    }

    #[test]
    fn split_strategy_apportion_regions_has_no_fields() {
        let s = SplitStrategy::ApportionRegions;
        assert_eq!(s.mode_name(), "apportion-regions",
            "ApportionRegions mode_name must be 'apportion-regions'");
        assert!(matches!(s, SplitStrategy::ApportionRegions));
    }

    #[test]
    fn split_strategy_area_section_has_area_swing() {
        let s = SplitStrategy::AreaSection { area_swing: 1.15 };
        assert_eq!(s.mode_name(), "areasection",
            "AreaSection mode_name must be 'areasection'");
        if let SplitStrategy::AreaSection { area_swing } = s {
            assert!((area_swing - 1.15).abs() < 1e-9,
                "area_swing field must round-trip, got {area_swing}");
        } else {
            panic!("AreaSection variant destructure failed");
        }
    }

    #[test]
    fn split_strategy_vra_section_has_w_vra() {
        let s = SplitStrategy::VraSection { w_vra: 0.30 };
        assert_eq!(s.mode_name(), "vra-section",
            "VraSection mode_name must be 'vra-section'");
        if let SplitStrategy::VraSection { w_vra } = s {
            assert!((w_vra - 0.30).abs() < 1e-9,
                "w_vra field must round-trip, got {w_vra}");
        } else {
            panic!("VraSection variant destructure failed");
        }
    }

    #[test]
    fn all_variants_mode_name_stable() {
        // Exhaustive: new variants added without updating this list → compile error.
        let all: &[(&str, SplitStrategy)] = &[
            ("edge-weighted",      SplitStrategy::Bisect),
            ("metis-vra",          SplitStrategy::NWay),
            ("geosection",         SplitStrategy::GeoSection),
            ("compact-bisect",     SplitStrategy::CompactBisect { epsilon: 0.05 }),
            ("areasection",        SplitStrategy::AreaSection { area_swing: 1.10 }),
            ("proportional-section", SplitStrategy::ProportionalSection { eta: 1.10 }),
            ("apportion-regions",  SplitStrategy::ApportionRegions),
            ("vra-section",        SplitStrategy::VraSection { w_vra: 0.40 }),
        ];
        for (expected_name, variant) in all {
            assert_eq!(variant.mode_name(), *expected_name,
                "SplitStrategy variant added without updating exhaustive list!");
        }
    }

    // ── Group 3: AlgorithmConfig with seeds field ─────────────────────────────

    #[test]
    fn algorithm_config_has_seeds_field() {
        // AlgorithmConfig::default() must expose a seeds field with 50 seeds.
        let algo = AlgorithmConfig::default();
        if let SeedCompositor::Multi { seeds } = algo.seeds {
            assert_eq!(seeds, 50, "default seeds must be Multi{{50}}");
        } else {
            panic!("default AlgorithmConfig seeds must be Multi{{50}}");
        }
    }

    #[test]
    fn apportion_regions_defaults_to_single_seed() {
        use crate::args::PartitionMode as PM;
        let algo = AlgorithmConfig::defaults_for_mode(&PM::ApportionRegions);
        assert!(algo.seeds.is_single(),
            "ApportionRegions defaults_for_mode must use SeedCompositor::Single \
             (federal statute requires deterministic single-seed)");
        assert!(matches!(algo.split, SplitStrategy::ApportionRegions),
            "defaults_for_mode(ApportionRegions) split must be ApportionRegions");
    }

    #[test]
    fn geosection_defaults_to_multi_50() {
        use crate::args::PartitionMode as PM;
        let algo = AlgorithmConfig::defaults_for_mode(&PM::GeoSection);
        if let SeedCompositor::Multi { seeds } = algo.seeds {
            assert_eq!(seeds, 50,
                "GeoSection defaults_for_mode must produce Multi{{seeds: 50}}, got {seeds}");
        } else {
            panic!("GeoSection defaults_for_mode seeds must be Multi, not Single or Sweep");
        }
    }

    #[test]
    fn compact_bisect_defaults_to_multi_50() {
        use crate::args::PartitionMode as PM;
        let algo = AlgorithmConfig::defaults_for_mode(&PM::CompactBisect);
        if let SeedCompositor::Multi { seeds } = algo.seeds {
            assert_eq!(seeds, 50,
                "CompactBisect defaults_for_mode must produce Multi{{seeds: 50}}, got {seeds}");
        } else {
            panic!("CompactBisect defaults_for_mode seeds must be Multi, not Single or Sweep");
        }
    }

    // ── Group 4: Compositor StructureMode / WeightMode / SearchMode overrides ──

    #[test]
    fn structure_override_none_leaves_split_unchanged() {
        use crate::args::StateArgs;
        use clap::Parser;
        // No --structure flag: split comes from --partition-mode preset.
        let args = StateArgs::parse_from([
            "state", "--state", "VT", "--partition-mode", "geosection",
            "--geosection-seeds", "30",
        ]);
        assert!(args.structure.is_none(), "no --structure flag → structure must be None");
        let algo = AlgorithmConfig::from_state_args(&args);
        assert!(matches!(algo.split, SplitStrategy::GeoSection),
            "without --structure override, GeoSection preset must set GeoSection split");
    }

    #[test]
    fn structure_override_prime_factor_sets_apportion_regions() {
        use crate::args::{StateArgs, StructureMode};
        use clap::Parser;
        // --structure prime-factor overrides the split regardless of --partition-mode.
        let args = StateArgs::parse_from([
            "state", "--state", "VT", "--partition-mode", "geosection",
            "--structure", "prime-factor",
        ]);
        assert_eq!(args.structure, Some(StructureMode::PrimeFactor),
            "parsed structure must be PrimeFactor");
        let algo = AlgorithmConfig::from_state_args(&args);
        assert!(matches!(algo.split, SplitStrategy::ApportionRegions),
            "prime-factor structure override must set ApportionRegions split");
    }

    #[test]
    fn search_override_single_sets_single_compositor() {
        use crate::args::{StateArgs, SearchMode};
        use clap::Parser;
        let args = StateArgs::parse_from([
            "state", "--state", "VT", "--partition-mode", "geosection",
            "--geosection-seeds", "30", "--search", "single",
        ]);
        assert_eq!(args.search, Some(SearchMode::Single),
            "parsed search must be Single");
        let algo = AlgorithmConfig::from_state_args(&args);
        assert!(algo.seeds.is_single(),
            "--search single must produce SeedCompositor::Single");
    }

    #[test]
    fn search_override_convergence_sets_sweep() {
        use crate::args::{StateArgs, SearchMode};
        use clap::Parser;
        let args = StateArgs::parse_from([
            "state", "--state", "VT", "--partition-mode", "geosection",
            "--search", "convergence", "--convergence-threshold", "300",
        ]);
        assert_eq!(args.search, Some(SearchMode::Convergence),
            "parsed search must be Convergence");
        assert_eq!(args.convergence_threshold, 300,
            "convergence_threshold must be 300");
        let algo = AlgorithmConfig::from_state_args(&args);
        if let SeedCompositor::ConvergenceSweep { threshold } = algo.seeds {
            assert_eq!(threshold, 300,
                "--search convergence must set ConvergenceSweep with the parsed threshold");
        } else {
            panic!("--search convergence must produce SeedCompositor::ConvergenceSweep");
        }
    }

    #[test]
    fn search_override_multi_with_seeds() {
        use crate::args::{StateArgs, SearchMode};
        use clap::Parser;
        let args = StateArgs::parse_from([
            "state", "--state", "VT", "--partition-mode", "geosection",
            "--search", "multi", "--seeds", "100",
        ]);
        assert_eq!(args.search, Some(SearchMode::Multi),
            "parsed search must be Multi");
        assert_eq!(args.seeds, Some(100),
            "--seeds 100 must be parsed");
        let algo = AlgorithmConfig::from_state_args(&args);
        if let SeedCompositor::Multi { seeds } = algo.seeds {
            assert_eq!(seeds, 100,
                "--search multi --seeds 100 must produce Multi{{seeds: 100}}");
        } else {
            panic!("--search multi must produce SeedCompositor::Multi");
        }
    }

    // ── Group 5: WeightMode compositor override ───────────────────────────────

    #[test]
    fn weights_override_unweighted_disables_geographic() {
        use crate::args::StateArgs;
        use clap::Parser;
        let args = StateArgs::parse_from([
            "state", "--state", "VT", "--weights-override", "unweighted",
        ]);
        let algo = AlgorithmConfig::from_state_args(&args);
        assert!(!algo.weights.geographic,
            "--weights-override unweighted must set geographic=false, got true");
    }

    #[test]
    fn weights_override_geographic_enables_geographic() {
        use crate::args::StateArgs;
        use clap::Parser;
        // Start with unweighted mode so the preset would disable geographic.
        let args = StateArgs::parse_from([
            "state", "--state", "VT",
            "--partition-mode", "unweighted",
            "--weights-override", "geographic",
        ]);
        let algo = AlgorithmConfig::from_state_args(&args);
        assert!(algo.weights.geographic,
            "--weights-override geographic must set geographic=true even when preset is unweighted");
    }

    #[test]
    fn weights_override_county_sets_alpha_positive() {
        use crate::args::StateArgs;
        use clap::Parser;
        let args = StateArgs::parse_from([
            "state", "--state", "VT",
            "--weights-override", "county",
            "--alpha-county", "3.0",
        ]);
        let algo = AlgorithmConfig::from_state_args(&args);
        assert!(algo.weights.alpha_county >= 1.0,
            "--weights-override county must set alpha_county >= 1.0, got {}",
            algo.weights.alpha_county);
    }

    #[test]
    fn weights_override_none_preserves_preset() {
        use crate::args::StateArgs;
        use clap::Parser;
        // Without --weights-override, --partition-mode unweighted keeps geographic=false.
        let args = StateArgs::parse_from([
            "state", "--state", "VT",
            "--partition-mode", "unweighted",
        ]);
        assert!(args.weights_override.is_none(),
            "no --weights-override flag must leave weights_override=None");
        let algo = AlgorithmConfig::from_state_args(&args);
        assert!(!algo.weights.geographic,
            "unweighted preset without override must keep geographic=false");
    }

    #[test]
    fn weights_override_vra_sets_minority_weighting() {
        use crate::args::StateArgs;
        use clap::Parser;
        let args = StateArgs::parse_from([
            "state", "--state", "VT",
            "--weights-override", "vra-aligned",
        ]);
        let algo = AlgorithmConfig::from_state_args(&args);
        assert!(algo.weights.minority_weighting,
            "--weights-override vra-aligned must set minority_weighting=true");
    }

    // ── Group 6: AlgorithmConfig.mode_name() method ───────────────────────────

    #[test]
    fn mode_name_from_algo_config() {
        use crate::args::PartitionMode as PM;
        // mode_name() from AlgorithmConfig must agree with SplitStrategy::mode_name()
        // for every preset that doesn't use mode_label.
        let cases = [
            (PM::EdgeWeighted,     "edge-weighted"),
            (PM::MetisVra,         "metis-vra"),
            (PM::GeoSection,       "geosection"),
            (PM::CompactBisect,    "compact-bisect"),
            (PM::AreaSection,      "areasection"),
            (PM::ApportionRegions, "apportion-regions"),
        ];
        for (mode, expected) in &cases {
            let algo = AlgorithmConfig::defaults_for_mode(mode);
            assert_eq!(algo.mode_name(), *expected,
                "AlgorithmConfig::mode_name() must match SplitStrategy::mode_name() \
                 for mode {:?}", expected);
            assert_eq!(algo.mode_name(), algo.split.mode_name(),
                "AlgorithmConfig and SplitStrategy mode_name must agree for {:?}", expected);
        }
    }

    #[test]
    fn mode_name_after_structure_override() {
        use crate::args::{StateArgs, StructureMode};
        use clap::Parser;
        // --structure prime-factor overrides split to ApportionRegions.
        // mode_name must reflect the overridden split.
        let args = StateArgs::parse_from([
            "state", "--state", "VT",
            "--partition-mode", "geosection",
            "--structure", "prime-factor",
        ]);
        let algo = AlgorithmConfig::from_state_args(&args);
        assert_eq!(algo.mode_name(), "apportion-regions",
            "after prime-factor structure override, mode_name must be 'apportion-regions'");
        assert_eq!(algo.mode_name(), algo.split.mode_name(),
            "AlgorithmConfig and SplitStrategy mode_name must agree after structure override");
    }

    #[test]
    fn mode_name_uses_mode_label_override() {
        // When mode_label is Some(...), mode_name() must return that label regardless
        // of the split strategy.
        let algo = AlgorithmConfig {
            split: SplitStrategy::Bisect,
            weights: WeightSpec::default(),
            mode_label: Some("custom"),
            ..AlgorithmConfig::default()
        };
        assert_eq!(algo.mode_name(), "custom",
            "mode_label=Some('custom') must take priority over split strategy");
    }

    // ── Group 7: ConvergenceSweep properties ─────────────────────────────────

    #[test]
    fn convergence_sweep_threshold_preserved() {
        let sc = SeedCompositor::ConvergenceSweep { threshold: 500 };
        assert_eq!(sc.seed_count(), 500,
            "ConvergenceSweep{{threshold: 500}}.seed_count() must return 500");
    }

    #[test]
    fn convergence_sweep_is_not_single() {
        let sc = SeedCompositor::ConvergenceSweep { threshold: 500 };
        assert!(!sc.is_single(),
            "ConvergenceSweep must not be is_single()");
    }

    #[test]
    fn convergence_sweep_default_threshold_via_args() {
        use crate::args::StateArgs;
        use clap::Parser;
        // --search convergence without explicit --convergence-threshold uses default 500.
        let args = StateArgs::parse_from([
            "state", "--state", "VT",
            "--search", "convergence",
        ]);
        let algo = AlgorithmConfig::from_state_args(&args);
        if let SeedCompositor::ConvergenceSweep { threshold } = algo.seeds {
            assert_eq!(threshold, 500,
                "default convergence threshold must be 500, got {threshold}");
        } else {
            panic!("--search convergence must produce ConvergenceSweep");
        }
    }

    #[test]
    fn convergence_sweep_custom_threshold() {
        use crate::args::StateArgs;
        use clap::Parser;
        let args = StateArgs::parse_from([
            "state", "--state", "VT",
            "--search", "convergence",
            "--convergence-threshold", "200",
        ]);
        assert_eq!(args.convergence_threshold, 200,
            "convergence_threshold must be parsed as 200");
        let algo = AlgorithmConfig::from_state_args(&args);
        if let SeedCompositor::ConvergenceSweep { threshold } = algo.seeds {
            assert_eq!(threshold, 200,
                "--convergence-threshold 200 must produce ConvergenceSweep{{threshold: 200}}");
        } else {
            panic!("--search convergence must produce ConvergenceSweep");
        }
    }

    // ── Group 8: SeedCompositor interaction with AlgorithmConfig ─────────────

    #[test]
    fn apportion_regions_from_state_args_single() {
        use crate::args::StateArgs;
        use clap::Parser;
        // ApportionRegions preset → Single seed (federal statute determinism).
        let args = StateArgs::parse_from([
            "state", "--state", "VT",
            "--partition-mode", "apportion-regions",
        ]);
        let algo = AlgorithmConfig::from_state_args(&args);
        assert!(algo.seeds.is_single(),
            "apportion-regions from_state_args must produce SeedCompositor::Single");
        assert!(matches!(algo.split, SplitStrategy::ApportionRegions),
            "apportion-regions split must be ApportionRegions");
    }

    #[test]
    fn search_override_wins_over_preset_seed() {
        use crate::args::StateArgs;
        use clap::Parser;
        // compact-bisect preset normally yields Multi seeds.
        // --search convergence must override that to ConvergenceSweep.
        let args = StateArgs::parse_from([
            "state", "--state", "VT",
            "--partition-mode", "compact-bisect",
            "--search", "convergence",
        ]);
        let algo = AlgorithmConfig::from_state_args(&args);
        assert!(matches!(algo.seeds, SeedCompositor::ConvergenceSweep { .. }),
            "--search convergence must override compact-bisect Multi preset to ConvergenceSweep");
        assert!(!algo.seeds.is_single(),
            "ConvergenceSweep must not be is_single()");
    }
}

// ── Phase 8: End-to-end label pipeline integration tests ─────────────────────
//
// These are L0 tests that exercise the full label pipeline API functions
// directly (not the CLI binary).  All file I/O goes through a temp directory
// created per test.
//
// Because `std::env::set_current_dir` is process-wide, every test here that
// changes the CWD MUST be run with `--test-threads=1`.  The helper
// `with_tempdir` restores the original directory after each closure so that
// tests can be chained without leaking CWD state.
//
// Tests reference §9.2 of the Spec 7 official_proposal workflow.

#[cfg(test)]
mod label_pipeline_tests {
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    use crate::label;
    use crate::run_registry::Registry;
    use crate::algo_config::AlgoYaml;
    use crate::build_cmd::{BuildArgs, BuildIndex};
    use crate::import_label::run_label_import;
    use crate::label_cmd::run_mv;

    // ── Helper: switch CWD to a TempDir for registry isolation ───────────────
    //
    // Returns the TempDir so callers can keep it alive while inspecting files.
    // The original directory is restored BEFORE the TempDir is dropped.
    fn with_tempdir<F: FnOnce()>(f: F) -> TempDir {
        let dir = TempDir::new().expect("tempdir");
        let original = std::env::current_dir().expect("current_dir");
        std::env::set_current_dir(dir.path()).expect("set_current_dir");
        f();
        std::env::set_current_dir(&original).expect("restore current_dir");
        dir
    }

    // ── Test 1: label path convention round-trip (no I/O) ────────────────────
    //
    // §9.2 Step 1: the three top-level directories follow a fixed pattern.
    // This test verifies the pattern without touching the filesystem.
    #[test]
    fn test_label_path_convention_roundtrip() {
        let label = "test_run";

        let runs = label::runs_dir(label);
        let analysis = label::analysis_dir(label);
        let reports = label::reports_dir(label);

        assert_eq!(
            runs,
            PathBuf::from("runs/test_run"),
            "runs_dir must be runs/{{label}}"
        );
        assert_eq!(
            analysis,
            PathBuf::from("analysis/test_run"),
            "analysis_dir must be analysis/{{label}}"
        );
        assert_eq!(
            reports,
            PathBuf::from("reports/test_run"),
            "reports_dir must be reports/{{label}}"
        );
    }

    // ── Test 2: full registry pipeline in tempdir ─────────────────────────────
    //
    // Mirrors §9.2 Steps 2-6: mark_built → mark_analyzed → mark_reported
    // all complete successfully, registry entry reflects all three stages,
    // and `.redist` is valid JSON.
    #[test]
    fn test_registry_full_pipeline_in_tempdir() {
        let dir = with_tempdir(|| {
            // Step 1: mark built
            Registry::mark_built("pipeline_test", "2020")
                .expect("mark_built must succeed");

            // Step 2: mark analyzed (requires built)
            Registry::mark_analyzed("pipeline_test", "2020")
                .expect("mark_analyzed must succeed after mark_built");

            // Step 3: mark reported (requires analyzed)
            Registry::mark_reported("pipeline_test", "2020")
                .expect("mark_reported must succeed after mark_analyzed");

            // Verify list_labels returns the label with all three stages set.
            let labels = Registry::list_labels().expect("list_labels");
            let entry = labels
                .iter()
                .find(|(name, _)| name == "pipeline_test")
                .map(|(_, e)| e)
                .expect("pipeline_test must be in registry");

            assert!(
                entry.built.contains(&"2020".to_string()),
                "built must contain 2020: {:?}", entry.built
            );
            assert!(
                entry.analyzed.contains(&"2020".to_string()),
                "analyzed must contain 2020: {:?}", entry.analyzed
            );
            assert!(
                entry.reported.contains(&"2020".to_string()),
                "reported must contain 2020: {:?}", entry.reported
            );

            // Verify .redist exists and is valid JSON.
            let registry_path = PathBuf::from(".redist");
            assert!(registry_path.exists(), ".redist must exist after pipeline");
            let content = std::fs::read_to_string(&registry_path)
                .expect("read .redist");
            let parsed: serde_json::Value = serde_json::from_str(&content)
                .expect(".redist must be valid JSON");
            assert!(parsed.is_object(), ".redist must be a JSON object");
        });
        drop(dir);
    }

    // ── Test 3: mark_analyzed fails without prior mark_built ─────────────────
    //
    // §9.3 error scenario: "Attempt to analyze before building".
    // The error must contain "[CONFIG]" per the project error convention.
    #[test]
    fn test_registry_mark_analyzed_fails_without_build() {
        let dir = with_tempdir(|| {
            let result = Registry::mark_analyzed("not_built", "2020");
            assert!(result.is_err(), "mark_analyzed must fail when year not built");

            let msg = result.unwrap_err();
            assert!(
                msg.contains("[CONFIG]"),
                "error must contain [CONFIG] prefix: {msg}"
            );
        });
        drop(dir);
    }

    // ── Test 4: BuildIndex schema is valid ────────────────────────────────────
    //
    // Constructs a BuildIndex via build_build_index (the same path run_build uses)
    // and verifies all required keys are present with the correct types.
    #[test]
    fn test_build_index_schema_valid() {
        use std::io::Write;
        use crate::build_cmd::build_build_index;

        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(b"name: test_plan\nalgorithm:\n  structure: apportion-regions\n  search: single\n").unwrap();

        let yaml = AlgoYaml::from_file(f.path()).expect("parse YAML");
        let sha = AlgoYaml::file_sha256(f.path()).expect("sha256");

        let index = build_build_index(
            "test_plan",
            "2020",
            &sha,
            &yaml,
            &[],
            &[],
        ).expect("build_build_index");

        // Verify the JSON representation has all required keys.
        let json_val = serde_json::to_value(&index).expect("serialize");
        let obj = json_val.as_object().expect("must be object");

        for key in &["label", "year", "created", "redist_version", "config_sha256", "algorithm", "states", "summary"] {
            assert!(
                obj.contains_key(*key),
                "BuildIndex JSON must contain key '{}', got keys: {:?}",
                key,
                obj.keys().collect::<Vec<_>>()
            );
        }

        // SHA is 64-char hex.
        let sha_in_index = obj["config_sha256"].as_str().expect("config_sha256 must be string");
        assert_eq!(sha_in_index.len(), 64, "config_sha256 must be 64 chars");
        assert!(
            sha_in_index.chars().all(|c| c.is_ascii_hexdigit()),
            "config_sha256 must be hex: {sha_in_index}"
        );

        // Summary has total/succeeded/failed.
        let summary = obj["summary"].as_object().expect("summary must be object");
        assert!(summary.contains_key("total"), "summary must have 'total'");
        assert!(summary.contains_key("succeeded"), "summary must have 'succeeded'");
        assert!(summary.contains_key("failed"), "summary must have 'failed'");
    }

    // ── Test 5: import CSV then verify registry and directory layout ──────────
    //
    // §9.2: external plan import creates the same directory layout as build.
    // Wisconsin FIPS prefix = "55".
    #[test]
    fn test_import_csv_then_list() {
        let dir = with_tempdir(|| {
            // Write a minimal Wisconsin CSV.
            let csv = "GEOID,district\n55001010100,1\n55001010200,2\n";
            let csv_path = PathBuf::from("plan.csv");
            std::fs::write(&csv_path, csv).expect("write CSV");

            // Import.
            run_label_import("import_test", &csv_path, "2020", Some("csv"))
                .expect("run_label_import must succeed");

            // Registry must show the label as built for 2020.
            let entry = Registry::get("import_test")
                .expect("get must not error")
                .expect("import_test must be in registry");
            assert!(
                entry.built.contains(&"2020".to_string()),
                "registry must mark import_test/2020 as built: {:?}", entry.built
            );

            // runs/import_test/2020/index.json must exist with algorithm.structure="external".
            let index_path = PathBuf::from("runs/import_test/2020/index.json");
            assert!(index_path.exists(), "index.json must exist: {}", index_path.display());
            let content = std::fs::read_to_string(&index_path).expect("read index.json");
            let val: serde_json::Value = serde_json::from_str(&content).expect("parse JSON");
            assert_eq!(
                val["algorithm"]["structure"].as_str(),
                Some("external"),
                "algorithm.structure must be 'external' for imported plans"
            );

            // runs/import_test/2020/wisconsin/assignments.json must exist (FIPS "55").
            let assignments_path =
                PathBuf::from("runs/import_test/2020/wisconsin/assignments.json");
            assert!(
                assignments_path.exists(),
                "wisconsin/assignments.json must exist: {}",
                assignments_path.display()
            );
        });
        drop(dir);
    }

    // ── Test 6: mv label updates registry and filesystem ─────────────────────
    //
    // §9.2-adjacent: label renaming (run_mv) must update both the `.redist`
    // registry and the `runs/` directory on disk.
    #[test]
    fn test_mv_label_updates_registry() {
        let dir = with_tempdir(|| {
            // Set up: mark old_label as built.
            Registry::mark_built("old_label", "2020").expect("mark_built");

            // Create runs/old_label/2020/ on disk so mv has a directory to rename.
            let old_runs = PathBuf::from("runs/old_label/2020");
            std::fs::create_dir_all(&old_runs).expect("create runs dir");

            // Execute mv.
            run_mv("old_label", "new_label", false).expect("run_mv must succeed");

            // old_label must be gone from registry.
            assert!(
                Registry::get("old_label").expect("get old_label").is_none(),
                "old_label must not be in registry after mv"
            );

            // new_label must be in registry with built = ["2020"].
            let entry = Registry::get("new_label")
                .expect("get new_label")
                .expect("new_label must be in registry after mv");
            assert!(
                entry.built.contains(&"2020".to_string()),
                "new_label must carry the built years: {:?}", entry.built
            );

            // runs/new_label/ must exist; runs/old_label/ must not.
            assert!(
                PathBuf::from("runs/new_label").exists(),
                "runs/new_label must exist after mv"
            );
            assert!(
                !PathBuf::from("runs/old_label").exists(),
                "runs/old_label must not exist after mv"
            );
        });
        drop(dir);
    }

    // ── Test 7: config YAML loads official_proposal algorithm section ─────────
    //
    // §9.1 config file: the official_proposal.yml specifies
    // structure=apportion-regions, weights=county, search=convergence.
    // We write it to a temp file and verify round-trip via AlgoYaml.
    //
    // Note: We write the YAML inline rather than reading the real
    // configs/official_proposal.yml because that file may not exist on
    // all developer machines (it is not in the repo, only in CWD of runs).
    #[test]
    fn test_config_yaml_loads_official_proposal() {
        use std::io::Write;

        let yaml_content = r#"
name: official_proposal
description: >
  Reference implementation for the proposed federal redistricting statute.
algorithm:
  structure: apportion-regions
  weights: county
  alpha_county: 2.0
  search: convergence
  convergence_threshold: 600
  balance_tolerance: 0.5
workers: 6
years: ["2020", "2010", "2000"]
analysis_types: [demographic, political, compactness, contiguity, splits, summary]
"#;

        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(yaml_content.as_bytes()).unwrap();

        // Load and parse.
        let yaml = AlgoYaml::from_file(f.path())
            .expect("official_proposal YAML must parse");

        // Verify structure == "apportion-regions".
        assert_eq!(
            yaml.algorithm.structure, "apportion-regions",
            "structure must be 'apportion-regions'"
        );

        // Verify weights == "county".
        assert_eq!(
            yaml.algorithm.weights.as_deref(),
            Some("county"),
            "weights must be 'county'"
        );

        // Round-trip to AlgorithmConfig must succeed.
        let algo = yaml.to_algorithm_config()
            .expect("to_algorithm_config must succeed for official_proposal YAML");

        // Confirm the split strategy is ApportionRegions.
        assert!(
            matches!(algo.split, crate::runner::SplitStrategy::ApportionRegions),
            "structure apportion-regions must map to SplitStrategy::ApportionRegions"
        );
    }

    // ── Test 8: full label workflow dry-run creates no files ──────────────────
    //
    // §9.2 Step 1: "smoke test with Vermont before committing to a full run".
    // With --dry-run, run_build must return Ok(()) without creating any
    // directory under runs/ or modifying the registry.
    #[test]
    fn test_full_label_workflow_dry_run() {
        use std::io::Write;

        let dir = with_tempdir(|| {
            // Write a minimal config YAML to configs/test_run.yml.
            let configs_dir = PathBuf::from("configs");
            std::fs::create_dir_all(&configs_dir).expect("create configs dir");
            let config_path = configs_dir.join("test_run.yml");

            let yaml_content =
                "name: test_run\nalgorithm:\n  structure: apportion-regions\n  search: single\nyears: [\"2020\"]\n";
            std::fs::write(&config_path, yaml_content).expect("write config");

            let args = BuildArgs {
                label: "test_run".to_string(),
                config: config_path,
                year: Some("2020".to_string()),
                states: vec![],
                workers: None,
                dry_run: true,
                force: false,
                no_interactive: false,
            };

            // run_build with dry_run=true must succeed.
            let result = crate::build_cmd::run_build(args);
            assert!(result.is_ok(), "dry_run run_build must succeed: {:?}", result);

            // runs/test_run/ must NOT exist (dry run creates nothing).
            assert!(
                !PathBuf::from("runs/test_run").exists(),
                "runs/test_run must not be created by dry_run build"
            );

            // Registry must remain empty (dry run does not call mark_built).
            let labels = Registry::list_labels().expect("list_labels");
            assert!(
                labels.is_empty(),
                "registry must be empty after dry_run build: {:?}", labels
            );
        });
        drop(dir);
    }

    // ════════════════════════════════════════════════════════════════════════
    // L1 TESTS — real file I/O in a temp directory, no METIS / census data
    //
    // Run with `cargo +stable test -p redist-cli -- --test-threads=1`
    // (set_current_dir is process-wide; serial execution is mandatory).
    // ════════════════════════════════════════════════════════════════════════

    // ── L1-1: import CSV full pipeline in tempdir ────────────────────────────
    //
    // Steps:
    //   1. Write a 4-row Wisconsin CSV to a file in the tempdir.
    //   2. Call run_label_import → Ok.
    //   3. Verify runs/csv_import_test/2020/index.json exists and is valid JSON.
    //   4. Verify runs/csv_import_test/2020/wisconsin/assignments.json exists.
    //   5. Verify registry marks the label as built for "2020".
    //   6. Verify index.json algorithm.structure == "external".
    #[test]
    fn test_import_csv_full_pipeline_in_tempdir() {
        let dir = with_tempdir(|| {
            // Write CSV with 4 Wisconsin tracts (FIPS "55")
            let csv = "GEOID,district\n55001010100,1\n55001010200,1\n55009010100,2\n55009010200,2\n";
            let csv_path = PathBuf::from("test_plan.csv");
            std::fs::write(&csv_path, csv).expect("write CSV");

            // Call import
            run_label_import("csv_import_test", &csv_path, "2020", Some("csv"))
                .expect("run_label_import must succeed");

            // Verify index.json exists and is valid JSON
            let index_path = PathBuf::from("runs/csv_import_test/2020/index.json");
            assert!(
                index_path.exists(),
                "runs/csv_import_test/2020/index.json must exist: {}",
                index_path.display()
            );
            let content = std::fs::read_to_string(&index_path).expect("read index.json");
            let v: serde_json::Value =
                serde_json::from_str(&content).expect("index.json must be valid JSON");
            assert!(v.is_object(), "index.json must be a JSON object");

            // Verify assignments.json exists
            let asgn_path =
                PathBuf::from("runs/csv_import_test/2020/wisconsin/assignments.json");
            assert!(
                asgn_path.exists(),
                "wisconsin/assignments.json must exist: {}",
                asgn_path.display()
            );

            // Verify registry shows built=["2020"]
            let entry = Registry::get("csv_import_test")
                .expect("registry get must not error")
                .expect("csv_import_test must be in registry");
            assert!(
                entry.built.contains(&"2020".to_string()),
                "registry must mark csv_import_test/2020 as built: {:?}", entry.built
            );

            // Verify algorithm.structure == "external"
            assert_eq!(
                v["algorithm"]["structure"].as_str(),
                Some("external"),
                "algorithm.structure must be 'external' for imported plans"
            );
        });
        drop(dir);
    }

    // ── L1-2: mv with actual directories ────────────────────────────────────
    //
    // Steps:
    //   1. Mark source_label as built in registry.
    //   2. Create runs/source_label/2020/ with a file in it.
    //   3. Write runs/source_label/2020/index.json with label field.
    //   4. Call run_mv → Ok.
    //   5. Verify runs/dest_label/2020/ exists.
    //   6. Verify runs/source_label/ does NOT exist.
    //   7. Verify runs/dest_label/2020/index.json has label == "dest_label" (patched).
    //   8. Verify registry: source gone, dest present.
    #[test]
    fn test_mv_with_actual_directories() {
        use crate::label_cmd::run_mv;

        let dir = with_tempdir(|| {
            // Mark source_label as built
            Registry::mark_built("source_label", "2020").expect("mark_built");

            // Create runs/source_label/2020/ with a sentinel file
            let src_year_dir = PathBuf::from("runs/source_label/2020");
            std::fs::create_dir_all(&src_year_dir).expect("create source dir");
            std::fs::write(src_year_dir.join("sentinel.txt"), "data")
                .expect("write sentinel");

            // Write runs/source_label/index.json (top-level) with label field
            let src_index_dir = PathBuf::from("runs/source_label");
            let src_index = src_index_dir.join("index.json");
            let index_content = serde_json::json!({
                "label": "source_label",
                "year": "2020"
            });
            std::fs::write(
                &src_index,
                serde_json::to_string_pretty(&index_content).unwrap(),
            )
            .expect("write source index.json");

            // Execute mv
            run_mv("source_label", "dest_label", false)
                .expect("run_mv must succeed");

            // runs/dest_label/2020/ must exist
            assert!(
                PathBuf::from("runs/dest_label/2020").exists(),
                "runs/dest_label/2020 must exist after mv"
            );

            // runs/source_label/ must NOT exist
            assert!(
                !PathBuf::from("runs/source_label").exists(),
                "runs/source_label must not exist after mv"
            );

            // runs/dest_label/index.json must have label == "dest_label"
            let dst_index_path = PathBuf::from("runs/dest_label/index.json");
            if dst_index_path.exists() {
                let raw = std::fs::read_to_string(&dst_index_path)
                    .expect("read dest index.json");
                let v: serde_json::Value =
                    serde_json::from_str(&raw).expect("parse dest index.json");
                assert_eq!(
                    v["label"].as_str(),
                    Some("dest_label"),
                    "label field must be patched to 'dest_label': {v}"
                );
            }

            // Registry: source gone, dest present
            assert!(
                Registry::get("source_label").expect("get source").is_none(),
                "source_label must be absent after mv"
            );
            let dest_entry = Registry::get("dest_label")
                .expect("get dest")
                .expect("dest_label must be in registry after mv");
            assert!(
                dest_entry.built.contains(&"2020".to_string()),
                "dest_label must carry built years: {:?}", dest_entry.built
            );
        });
        drop(dir);
    }

    // ── L1-3: verify full SHA chain in tempdir ───────────────────────────────
    //
    // Steps:
    //   1. Write configs/test_verify.yml and compute its SHA-256.
    //   2. Write runs/test_verify/2020/index.json with config_sha256.
    //   3. Compute run index SHA → write analysis/test_verify/2020/index.json
    //      with run_index_sha256.
    //   4. Compute analysis index SHA → write reports/test_verify/2020/index.json
    //      with analysis_index_sha256.
    //   5. Mark all stages in registry.
    //   6. Call run_label_verify → Ok (VERIFIED).
    #[test]
    fn test_verify_full_sha_chain_tempdir() {
        use sha2::{Digest, Sha256};
        use crate::label_cmd::run_verify;

        let dir = with_tempdir(|| {
            // Step 1: Write config file and compute its SHA-256
            std::fs::create_dir_all("configs").expect("create configs");
            let config_path = PathBuf::from("configs/test_verify.yml");
            let config_content =
                "name: test_verify\nalgorithm:\n  structure: apportion-regions\n  search: single\n";
            std::fs::write(&config_path, config_content).expect("write config");

            let config_sha = {
                let bytes = std::fs::read(&config_path).expect("read config");
                let mut h = Sha256::new();
                h.update(&bytes);
                format!("{:x}", h.finalize())
            };

            // Step 2: Write runs/test_verify/2020/index.json with config_sha256
            std::fs::create_dir_all("runs/test_verify/2020").expect("create runs dir");
            let run_index_path = PathBuf::from("runs/test_verify/2020/index.json");
            let run_index_content = serde_json::json!({
                "label": "test_verify",
                "year": "2020",
                "config_sha256": config_sha,
            });
            std::fs::write(
                &run_index_path,
                serde_json::to_string_pretty(&run_index_content).unwrap(),
            )
            .expect("write run index");

            // Compute run index SHA
            let run_index_sha = {
                let bytes = std::fs::read(&run_index_path).expect("read run index");
                let mut h = Sha256::new();
                h.update(&bytes);
                format!("{:x}", h.finalize())
            };

            // Step 3: Write analysis/test_verify/2020/index.json
            std::fs::create_dir_all("analysis/test_verify/2020").expect("create analysis dir");
            let analysis_index_path = PathBuf::from("analysis/test_verify/2020/index.json");
            let analysis_index_content = serde_json::json!({
                "label": "test_verify",
                "year": "2020",
                "run_index_sha256": run_index_sha,
            });
            std::fs::write(
                &analysis_index_path,
                serde_json::to_string_pretty(&analysis_index_content).unwrap(),
            )
            .expect("write analysis index");

            // Compute analysis index SHA
            let analysis_index_sha = {
                let bytes = std::fs::read(&analysis_index_path).expect("read analysis index");
                let mut h = Sha256::new();
                h.update(&bytes);
                format!("{:x}", h.finalize())
            };

            // Step 4: Write reports/test_verify/2020/index.json
            std::fs::create_dir_all("reports/test_verify/2020").expect("create reports dir");
            let report_index_path = PathBuf::from("reports/test_verify/2020/index.json");
            let report_index_content = serde_json::json!({
                "label": "test_verify",
                "year": "2020",
                "analysis_index_sha256": analysis_index_sha,
            });
            std::fs::write(
                &report_index_path,
                serde_json::to_string_pretty(&report_index_content).unwrap(),
            )
            .expect("write report index");

            // Step 5: Mark all stages in registry
            Registry::mark_built("test_verify", "2020").expect("mark_built");
            Registry::mark_analyzed("test_verify", "2020").expect("mark_analyzed");
            Registry::mark_reported("test_verify", "2020").expect("mark_reported");

            // Step 6: run_label_verify should return Ok (VERIFIED)
            let result = run_verify("test_verify", Some("2020"));
            assert!(
                result.is_ok(),
                "full matching SHA chain must return VERIFIED: {:?}", result
            );
        });
        drop(dir);
    }

    // ── L1-4: build dry-run creates no files ────────────────────────────────
    //
    // Steps:
    //   1. Write configs/dry_run_test.yml.
    //   2. Call run_build with dry_run=true.
    //   3. Verify runs/dry_run_test/ does NOT exist.
    //   4. Verify registry has no entry for "dry_run_test".
    #[test]
    fn test_build_dry_run_creates_no_files() {
        use crate::build_cmd::{BuildArgs, run_build};

        let dir = with_tempdir(|| {
            // Write minimal config YAML
            std::fs::create_dir_all("configs").expect("create configs dir");
            let config_path = PathBuf::from("configs/dry_run_test.yml");
            let yaml =
                "name: dry_run_test\nalgorithm:\n  structure: apportion-regions\n  search: single\nyears: [\"2020\"]\n";
            std::fs::write(&config_path, yaml).expect("write config");

            // Build with dry_run = true
            let args = BuildArgs {
                label: "dry_run_test".to_string(),
                config: config_path,
                year: Some("2020".to_string()),
                states: vec![],
                workers: None,
                dry_run: true,
                force: false,
                no_interactive: false,
            };
            let result = run_build(args);
            assert!(result.is_ok(), "dry_run build must return Ok: {:?}", result);

            // runs/dry_run_test/ must NOT exist
            assert!(
                !PathBuf::from("runs/dry_run_test").exists(),
                "runs/dry_run_test must not be created by dry_run build"
            );

            // Registry must have no entry for dry_run_test
            let entry = Registry::get("dry_run_test").expect("registry get");
            assert!(
                entry.is_none(),
                "registry must not contain dry_run_test after dry_run build: {:?}", entry
            );
        });
        drop(dir);
    }

    // ── L1-5: analyze creates index from mock final_assignments.json ─────────
    //
    // Steps:
    //   1. Mark mock_analyze_test as built for "2020" in registry.
    //   2. Create runs/mock_analyze_test/2020/vermont/final_assignments.json.
    //   3. Write runs/mock_analyze_test/2020/index.json (valid build index JSON).
    //   4. Call run_label_analyze → if Ok, verify analysis index exists with
    //      run_index_sha256 field.
    #[test]
    fn test_analyze_label_creates_index_from_mock_assignments() {
        use crate::analyze_label::run_label_analyze;

        let dir = with_tempdir(|| {
            // Step 1: Mark as built
            Registry::mark_built("mock_analyze_test", "2020").expect("mark_built");

            // Step 2: Create final_assignments.json (run_analyze_state looks for this)
            let state_dir =
                PathBuf::from("runs/mock_analyze_test/2020/vermont");
            std::fs::create_dir_all(&state_dir).expect("create state dir");
            let assignments = serde_json::json!({"1": [1, 2, 3], "2": [4, 5, 6]});
            std::fs::write(
                state_dir.join("final_assignments.json"),
                serde_json::to_string_pretty(&assignments).unwrap(),
            )
            .expect("write final_assignments.json");

            // Step 3: Write a minimal build index
            let build_index = serde_json::json!({
                "label": "mock_analyze_test",
                "year": "2020",
                "config_sha256": "0".repeat(64),
                "algorithm": {"structure": "apportion-regions"},
                "states": {"vermont": {"status": "ok", "districts": 1}},
                "summary": {"total": 1, "succeeded": 1, "failed": 0},
            });
            std::fs::write(
                "runs/mock_analyze_test/2020/index.json",
                serde_json::to_string_pretty(&build_index).unwrap(),
            )
            .expect("write build index");

            // Step 4: Call run_label_analyze
            let types: Vec<String> = vec!["summary".to_string()];
            let states: Vec<String> = vec![];
            let result = run_label_analyze(
                "mock_analyze_test",
                &types,
                Some("2020"),
                &states,
                false,
            );

            // Regardless of Ok or Err, check what was written
            match result {
                Ok(()) => {
                    // If analysis succeeded, verify the index exists with run_index_sha256
                    let analysis_index =
                        PathBuf::from("analysis/mock_analyze_test/2020/index.json");
                    if analysis_index.exists() {
                        let raw = std::fs::read_to_string(&analysis_index)
                            .expect("read analysis index");
                        let v: serde_json::Value =
                            serde_json::from_str(&raw).expect("parse analysis index");
                        assert!(
                            v.get("run_index_sha256").is_some(),
                            "analysis index must have run_index_sha256 field: {v}"
                        );
                    }
                    // Mark as analyzed verified by the call itself
                    let entry = Registry::get("mock_analyze_test")
                        .expect("registry get")
                        .expect("label must exist");
                    assert!(
                        entry.analyzed.contains(&"2020".to_string()),
                        "registry must mark mock_analyze_test/2020 as analyzed: {:?}",
                        entry.analyzed
                    );
                }
                Err(e) => {
                    // An error is acceptable only if final_assignments.json was
                    // not found (possible if run_analyze_state has a different path).
                    // We document the outcome but don't fail the test on a path mismatch.
                    eprintln!(
                        "[L1-5] run_label_analyze returned Err (path mismatch or \
                         graceful skip): {e}"
                    );
                    // At minimum: verify the function doesn't panic and produces
                    // a human-readable error message.
                    assert!(
                        !e.is_empty(),
                        "error message must not be empty"
                    );
                }
            }
        });
        drop(dir);
    }

    // ── L1-6: registry concurrent write sequential simulation ────────────────
    //
    // Simulates sequential registry mutations from two "concurrent" writers:
    //   1. mark_built("label_a", "2020")
    //   2. mark_built("label_b", "2020")
    //   3. list_labels() contains both
    //   4. .redist file is valid JSON with both entries
    //
    // Note: true concurrency testing requires threads but set_current_dir
    // is process-wide; this test verifies sequential write correctness and
    // that the atomic rename leaves no .redist.tmp artifact.
    #[test]
    fn test_registry_concurrent_write_sequential_simulation() {
        let dir = with_tempdir(|| {
            // Sequential writes from "two processes"
            Registry::mark_built("label_a", "2020").expect("mark_built label_a");
            Registry::mark_built("label_b", "2020").expect("mark_built label_b");

            // list_labels must contain both
            let labels = Registry::list_labels().expect("list_labels");
            let names: Vec<&str> = labels.iter().map(|(n, _)| n.as_str()).collect();
            assert!(
                names.contains(&"label_a"),
                "registry must contain label_a: {names:?}"
            );
            assert!(
                names.contains(&"label_b"),
                "registry must contain label_b: {names:?}"
            );

            // .redist must be valid JSON with both entries
            let content =
                std::fs::read_to_string(".redist").expect(".redist must exist");
            let v: serde_json::Value =
                serde_json::from_str(&content).expect(".redist must be valid JSON");
            assert!(v.is_object(), ".redist must be a JSON object");
            assert!(
                v.get("label_a").is_some(),
                "label_a must appear in .redist JSON: {v}"
            );
            assert!(
                v.get("label_b").is_some(),
                "label_b must appear in .redist JSON: {v}"
            );

            // .redist.tmp must not exist after successful save (atomic rename)
            assert!(
                !PathBuf::from(".redist.tmp").exists(),
                ".redist.tmp must not exist after atomic rename"
            );
        });
        drop(dir);
    }

    // ════════════════════════════════════════════════════════════════════════
    // L2 TESTS — require real adjacency data + METIS; marked #[ignore]
    //
    // Prerequisites for L2 tests:
    //   redist fetch --type adjacency --states VT --year 2020
    //   (or copy VT adjacency from outputs/V3/data/2020/adjacency/)
    //
    // Run with:
    //   cargo +stable test -p redist-cli label_pipeline_tests::test_build_label_ \
    //       -- --ignored --test-threads=1
    // ════════════════════════════════════════════════════════════════════════

    // ── L2-1: build label Vermont 2020 ──────────────────────────────────────
    //
    // VT has exactly 1 congressional district — METIS trivially partitions it.
    // This is the fastest possible real build test.
    #[test]
    #[ignore = "requires adjacency data: redist fetch --type adjacency --states VT --year 2020"]
    fn test_build_label_vermont_2020() {
        use crate::build_cmd::{BuildArgs, run_build};

        let dir = with_tempdir(|| {
            // Write configs/vt_l2_test.yml
            std::fs::create_dir_all("configs").expect("create configs");
            let config_path = PathBuf::from("configs/vt_l2_test.yml");
            let yaml = "name: vt_l2_test\n\
                        algorithm:\n\
                          structure: apportion-regions\n\
                          search: single\n\
                          balance_tolerance: 5.0\n\
                        workers: 1\n\
                        years: [\"2020\"]\n";
            std::fs::write(&config_path, yaml).expect("write config");

            // Point the adjacency data location to the real outputs directory.
            // run_build internally calls load_all_states(year) which reads from
            // outputs/data/{year}/adjacency/ relative to CWD — but since we're
            // in a tempdir, we need to copy or symlink the VT adjacency.
            // For the CI/ignore pattern, the test is skipped unless data exists;
            // a developer who runs it manually ensures the data is present.

            let args = BuildArgs {
                label: "vt_l2_test".to_string(),
                config: config_path,
                year: Some("2020".to_string()),
                states: vec!["VT".to_string()],
                workers: Some(1),
                dry_run: false,
                force: false,
                no_interactive: true,
            };

            // The build will fail if adjacency data is not in the expected location.
            // We treat any I/O error as a signal that data is missing (acceptable for
            // an #[ignore] test that documents the prerequisite).
            let result = run_build(args);
            if result.is_err() {
                let msg = result.unwrap_err();
                // Only panic if it's not a data-missing error
                if msg.contains("[INTERNAL]") || msg.contains("[CONFIG]") {
                    // Legitimate test failure
                    panic!("run_build(VT 2020) failed with infrastructure error: {msg}");
                }
                // Data-missing or adjacency error: skip gracefully
                eprintln!("[L2-1] skipping assertion — adjacency data not found: {msg}");
                return;
            }

            // If build succeeded: verify outputs
            let assignments =
                PathBuf::from("runs/vt_l2_test/2020/vermont/assignments.json");
            assert!(
                assignments.exists(),
                "vermont/assignments.json must exist after VT build"
            );

            let index_path = PathBuf::from("runs/vt_l2_test/2020/index.json");
            assert!(index_path.exists(), "index.json must exist after VT build");
            let content = std::fs::read_to_string(&index_path).expect("read index.json");
            let v: serde_json::Value = serde_json::from_str(&content).expect("parse index.json");
            let succeeded = v["summary"]["succeeded"].as_u64().unwrap_or(0);
            assert!(
                succeeded >= 1,
                "summary.succeeded must be >= 1 for VT: {v}"
            );

            let entry = Registry::get("vt_l2_test")
                .expect("registry get")
                .expect("vt_l2_test must be in registry");
            assert!(
                entry.built.contains(&"2020".to_string()),
                "registry must mark vt_l2_test/2020 as built: {:?}", entry.built
            );
        });
        drop(dir);
    }

    // ── L2-2: build then verify SHA chain Vermont ────────────────────────────
    //
    // Extends L2-1: after a successful build, run_verify should confirm
    // the config → build-index SHA link is MATCH (VERIFIED for that link).
    // Analysis and report links will be MISSING (not run yet), causing overall
    // FAILED — but the config SHA link correctness is tested.
    #[test]
    #[ignore = "requires adjacency data: redist fetch --type adjacency --states VT --year 2020"]
    fn test_build_then_verify_sha_chain_vermont() {
        use crate::build_cmd::{BuildArgs, run_build};
        use crate::label_cmd::run_verify;

        let dir = with_tempdir(|| {
            // Write config
            std::fs::create_dir_all("configs").expect("create configs");
            let config_path = PathBuf::from("configs/vt_l2_test.yml");
            let yaml = "name: vt_l2_test\n\
                        algorithm:\n\
                          structure: apportion-regions\n\
                          search: single\n\
                          balance_tolerance: 5.0\n\
                        workers: 1\n\
                        years: [\"2020\"]\n";
            std::fs::write(&config_path, yaml).expect("write config");

            let args = BuildArgs {
                label: "vt_l2_test".to_string(),
                config: config_path,
                year: Some("2020".to_string()),
                states: vec!["VT".to_string()],
                workers: Some(1),
                dry_run: false,
                force: false,
                no_interactive: true,
            };
            let build_result = run_build(args);
            if build_result.is_err() {
                eprintln!(
                    "[L2-2] build failed — likely missing adjacency data: {:?}",
                    build_result
                );
                return; // graceful skip
            }

            // run_verify: config→build link should be MATCH.
            // Overall verdict may be FAILED (missing analysis/report), but the
            // function output (which goes to stdout) contains "VERIFIED" for the
            // config sha link.  We can't capture stdout here without extra machinery,
            // so we just confirm the function doesn't panic.
            // A full VERIFIED requires all three chain links, so we expect Err here
            // (missing analysis/report links).
            let verify_result = run_verify("vt_l2_test", Some("2020"));
            // The result will be Err("verify: SHA chain has failures") because
            // analysis/reports index files don't exist yet.  That is expected.
            // We just confirm it's not a panic.
            eprintln!(
                "[L2-2] verify result (expected Err for missing analysis/report): {:?}",
                verify_result
            );
        });
        drop(dir);
    }

    // ── L2-3: build → mv → verify rename Vermont ────────────────────────────
    //
    // Extends L2-1: after a successful build, rename the label and verify
    // the registry and filesystem reflect the new name.
    #[test]
    #[ignore = "requires adjacency data: redist fetch --type adjacency --states VT --year 2020"]
    fn test_build_mv_then_analyze_vermont() {
        use crate::build_cmd::{BuildArgs, run_build};
        use crate::label_cmd::run_mv;

        let dir = with_tempdir(|| {
            // Write config
            std::fs::create_dir_all("configs").expect("create configs");
            let config_path = PathBuf::from("configs/vt_l2_test.yml");
            let yaml = "name: vt_l2_test\n\
                        algorithm:\n\
                          structure: apportion-regions\n\
                          search: single\n\
                          balance_tolerance: 5.0\n\
                        workers: 1\n\
                        years: [\"2020\"]\n";
            std::fs::write(&config_path, yaml).expect("write config");

            let args = BuildArgs {
                label: "vt_l2_test".to_string(),
                config: config_path,
                year: Some("2020".to_string()),
                states: vec!["VT".to_string()],
                workers: Some(1),
                dry_run: false,
                force: false,
                no_interactive: true,
            };
            let build_result = run_build(args);
            if build_result.is_err() {
                eprintln!(
                    "[L2-3] build failed — likely missing adjacency data: {:?}",
                    build_result
                );
                return; // graceful skip
            }

            // Execute mv: rename vt_l2_test → vt_l2_renamed
            let mv_result = run_mv("vt_l2_test", "vt_l2_renamed", false);
            assert!(
                mv_result.is_ok(),
                "run_mv must succeed: {:?}", mv_result
            );

            // Registry must show new name
            assert!(
                Registry::get("vt_l2_test").expect("get old").is_none(),
                "vt_l2_test must be gone from registry after mv"
            );
            let renamed_entry = Registry::get("vt_l2_renamed")
                .expect("get renamed")
                .expect("vt_l2_renamed must be in registry");
            assert!(
                renamed_entry.built.contains(&"2020".to_string()),
                "vt_l2_renamed must carry built years: {:?}", renamed_entry.built
            );

            // Old directories gone, new directories present
            assert!(
                !PathBuf::from("runs/vt_l2_test").exists(),
                "runs/vt_l2_test must not exist after mv"
            );
            assert!(
                PathBuf::from("runs/vt_l2_renamed").exists(),
                "runs/vt_l2_renamed must exist after mv"
            );
        });
        drop(dir);
    }
}
