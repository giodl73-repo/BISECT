use crate::adjacency_loader::load_adjacency_pkl;
use crate::bisection_runner::{
    run_all_splits, run_all_splits_compact, run_all_splits_percentile, run_all_splits_with_search,
    run_flip_chain, run_forest_recom, run_geosection, run_merge_split, run_multiscale,
    run_multiscale_adaptive, run_nway_partition, run_parallel_tempering, run_short_burst,
    run_short_burst_forest, run_short_burst_merge_split, run_vra_recom, AdaptiveConfig,
    CompactBisectOpts,
};
use crate::demographics::{
    align_demographics_to_adjacency, align_vap_demographics_to_adjacency, load_demographics,
    load_vap_demographics,
};
use crate::fetch::load_manifest;
use crate::output::{clean_corrupt_state, write_state_outputs, VraAnalysis, VraDistrict};
use crate::partisan_shares::load_partisan_shares;
use crate::status::{ascii_safe, status};
use crate::vertex_weights::{build_vertex_weights, VertexConstraintKind};
use bisect_analysis::analyze_mm_districts;
use bisect_core::{state_code_to_fips, Partition};
use bisect_report;
pub use bisect_runner::runner::{validate_multiscale_levels, AreaSectionInit};

mod support;
#[allow(unused_imports)]
pub use support::*;

/// Multi-state Rayon parallel runner + single-state implementation.
///
/// `run_states_parallel` dispatches states across Rayon threads.
/// `run_single_state` is the core: load adjacency → bisect → balance check → write.
///
/// PROCESS EXPLOSION PREVENTION: state_name and num_districts are pre-resolved
/// in StateConfig BEFORE the Rayon pool starts. This prevents 100+ Python
/// subprocesses from being spawned simultaneously during a 50-state run.
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

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
    /// Enables statutory choice of legal posture (B.7 / U.8).
    Percentile { p: f64, seeds: usize },
    /// At each bisection node, run a local `ensemble_steps`-step 2-way ReCom ensemble
    /// and pick the bisection at percentile `p` of the cut distribution.
    /// Always k=2 at each node — eliminates prime-k bipartition failures. (U.9)
    BisectionEnsemble { p: f64, ensemble_steps: usize },
    /// Flip individual boundary tracts to adjacent districts, collect all visited plans,
    /// return the plan at percentile `p` of the edge-cut distribution.
    /// flip_steps: total flip proposals (default: 10000). p: percentile (default: 0.0).
    Flip { flip_steps: usize, p: f64 },
    /// Short-Burst: run `n_bursts` short ReCom chains of `burst_length` steps on the
    /// full-state k-way assignment. Keep the chain endpoint from each burst (not the
    /// minimum). Chain restarts from the previous burst's endpoint. Sort endpoints by
    /// EC ASC; return the plan at rank floor(p * n_bursts). (G.6)
    ShortBurst {
        burst_length: usize,
        n_bursts: usize,
        p: f64,
    },
    /// Short-Burst using ForestRecomChain (two-tree MH) as the burst chain.
    /// Provides compactness optimization with approximate distributional correctness. (G.12)
    ShortBurstForest {
        burst_length: usize,
        n_bursts: usize,
        p: f64,
    },
    /// Short-Burst using MergeSplitChain as the burst chain. (G.12)
    ShortBurstMergeSplit {
        burst_length: usize,
        n_bursts: usize,
        p: f64,
    },
    /// Run `steps` Forest ReCom MH steps; collect accepted plans; return at percentile p of EC.
    /// Two-tree Metropolis-Hastings — targets uniform distribution (G.9 spec accepted 3.0/4).
    ForestRecom { steps: usize, p: f64 },
    /// Multi-scale MCMC — interleaves fine (tract) and coarse (block-group) ReCom moves.
    /// Requires block-group adjacency (run: bisect fetch --resolution block_group).
    /// CLI: --search multiscale --multiscale-steps 2000 --multiscale-alpha 0.3
    /// (G.11 spec accepted 3.0/4; full implementation pending BISECT-multiscale crate completion)
    MultiScale {
        total_steps: usize,
        p: f64,
        alpha: f64,
    },
    /// Run `steps` Merge-Split MH steps; collect accepted plans; return at percentile p.
    /// Two-tree MH with explicit ratio — O(m log m) per step (G.10 spec accepted 3.0/4).
    MergeSplit { steps: usize, p: f64 },
    /// Adaptive Multi-scale MCMC — Robbins-Monro self-tuning alpha (U.5 spec accepted 3.75/4).
    /// Requires block-group adjacency (run: bisect fetch --resolution block_group).
    /// CLI: --search multiscale-adaptive --multiscale-steps 2000 --ms-target-accept 0.30 --ms-adapt-interval 50
    MultiScaleAdaptive {
        total_steps: usize,
        p: f64,
        target_accept: f64,
        adapt_interval: usize,
    },
    /// Parallel Tempering: N replicas at geometric tolerance ladder with replica exchange.
    /// Cold chain (replica 0) provides the plan distribution. (U.4 spec accepted 4.0/4)
    ParallelTempering {
        n_replicas: usize,    // number of chains (default: 4)
        swap_interval: usize, // steps between swap proposals (default: 10)
        cold_tolerance: f64,  // cold chain balance tolerance (default: 0.005)
        hot_tolerance: f64,   // hot chain balance tolerance (default: 0.05)
        steps: usize,         // total steps per cold chain (default: 1000)
        p: f64,               // percentile of cold chain EC distribution (default: 0.0)
    },
    /// VRA-aware MCMC: ForestRecomChain with hard VRA rejection preserving majority-minority districts.
    /// Requires minority VAP data via --weights-override vra-aligned or explicit minority column.
    /// (G.13 spec accepted 3.75/4)
    VraRecom {
        steps: usize,       // total chain steps (default: 1000)
        p: f64,             // percentile of EC distribution (default: 0.0)
        vap_threshold: f64, // minority VAP fraction threshold (default: 0.50)
    },
    /// SMC weighted ensemble — runs BISECT-smc and selects plan at p-th weighted EC quantile.
    /// The only compositor mode with a calibrated (importance-weighted) stationary distribution.
    /// (SmcPercentile spec accepted 3.88/4)
    SmcPercentile { n_particles: usize, p: f64 },
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
            Self::MultiScaleAdaptive { total_steps, .. } => *total_steps,
            Self::ParallelTempering { steps, .. } => *steps,
            Self::VraRecom { steps, .. } => *steps,
            Self::SmcPercentile { n_particles, .. } => *n_particles,
        }
    }

    pub fn is_single(&self) -> bool {
        matches!(self, Self::Single)
    }
}

impl Default for SeedCompositor {
    fn default() -> Self {
        Self::Multi { seeds: 50 }
    }
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
    /// GeoSection (T.1): ratio-optimal direction-aware bisection.
    /// Seeds controlled by SeedCompositor.
    GeoSection,
    /// CompactBisect (B.7): greedy level-by-level geometric-mean PP selection.
    CompactBisect { epsilon: f64 },
    /// AreaSection (T.2): ratio-optimal bisection with dual population+area constraint.
    /// `area_section_init` controls the warm-start strategy (default: RatioOptimal).
    AreaSection {
        area_swing: f64,
        area_section_init: AreaSectionInit,
    },
    /// ProportionalSection (T.5): ncon=2 [pop, D_votes] with HH-derived tpwgts.
    ProportionalSection { eta: f64 },
    /// ApportionRegions (T.4): prime-factorisation tree — Huntington-Hill geographic completion.
    /// Default seed strategy: Single (content-derived). ConvergenceSweep for federal statute.
    ApportionRegions,
    /// VRASection (T.7): GeoSection modified by geographic minority-VAP alignment score.
    VraSection { w_vra: f64 },
    /// Simulated Annealing bisection: start from METIS, accept/reject boundary flips
    /// via Boltzmann criterion. Structure layer -- replaces METIS at each bisection node.
    /// steps_per_tract: n_steps = steps_per_tract * |subgraph| (default: 10).
    /// t0_factor: T_0 = max(1.0, t0_factor * EC(initial)) (default: 0.01).
    /// t_final: near-zero final temperature (default: 1e-4).
    SimulatedAnnealing {
        steps_per_tract: usize,
        t0_factor: f64,
        t_final: f64,
    },
    /// Centroidal Voronoi Districts — geometric packing via graph-distance Voronoi (T.10 spec).
    /// Seeds placed by k-farthest spread, iteratively moved to medoid of each Voronoi region.
    /// n_iter: max CVD iterations before returning (default: 20).
    /// Centroidal Voronoi Districts -- geometric packing via graph-distance or geographic Voronoi (T.10 spec).
    /// Seeds placed by k-farthest spread, iteratively moved to medoid/centroid of each Voronoi region.
    /// n_iter: max CVD iterations before returning (default: 20).
    /// metric: GraphDistance (Phase 1, default) or Geographic (Phase 2, requires tract_centroids).
    CentroidalVoronoi {
        n_iter: usize,
        metric: crate::bisection_runner::VoronoiMetric,
    },
    /// BFS Region-Growing — greedy geographic packing from k-farthest seeds (T.12 spec 4.0/4).
    /// Seeds placed by maximum BFS spread; tracts assigned to most population-deficient district.
    BfsGrowth,
    /// ILP exact redistricting — provably optimal edge-cut plan via integer programming.
    /// Only practical for n <= max_tracts (default 500). Falls back to METIS for larger nodes.
    /// (U.6 spec accepted 3.38/4)
    Ilp {
        method: crate::args::IlpMethod, // solver/certificate mode (default: formulation-only)
        fallback: crate::args::IlpFallback, // behavior when no ILP plan is available
        time_limit_secs: u64,           // solver time limit (default: 300)
        optimality_gap: f64,            // acceptable gap from optimal (default: 0.01)
        max_tracts: usize,              // size guard (default: 500; fallback to METIS if exceeded)
    },
    /// Moving-Knife Algorithm — maximises Reock compactness via sweep (T.13 spec 3.75/4).
    /// Tests n_orientations candidate sweep directions; picks angle with best min(Reock_L, Reock_R).
    MovingKnife {
        n_orientations: usize, // sweep granularity (default: 180 = every 1°)
        metric: String,        // "reock" (default) | "polsby"
    },
    /// Capacity-constrained clustering (T.15). Crate-level kernel is available;
    /// runner execution is staged after direct k-way plan integration.
    CapacityClustering,
    /// Spectral graph partitioning baseline (T.14).
    Spectral { max_iters: usize },
    /// Hierarchical regionalization baseline (T.16).
    Regionalization,
    /// Flow-style constructive assignment baseline (T.17).
    FlowConstruction,
}

impl SplitStrategy {
    /// Human-readable mode name (for logging and manifest).
    pub fn mode_name(&self) -> &'static str {
        match self {
            Self::Bisect => "edge-weighted",
            Self::NWay => "metis-vra",
            Self::GeoSection => "geosection",
            Self::CompactBisect { .. } => "compact-bisect",
            Self::AreaSection { .. } => "areasection",
            Self::ProportionalSection { .. } => "proportional-section",
            Self::ApportionRegions => "apportion-regions",
            Self::VraSection { .. } => "vra-section",
            Self::SimulatedAnnealing { .. } => "simulated-annealing",
            Self::CentroidalVoronoi { .. } => "centroidal-voronoi",
            Self::BfsGrowth => "bfs-growth",
            Self::Ilp { .. } => "ilp",
            Self::MovingKnife { .. } => "moving-knife",
            Self::CapacityClustering => "capacity-clustering",
            Self::Spectral { .. } => "spectral",
            Self::Regionalization => "regionalization",
            Self::FlowConstruction => "flow-construction",
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
    /// County stickiness alpha (T.3). 0 = disabled.
    pub alpha_county: f64,
    /// MCD stickiness alpha. 0 = disabled.
    pub alpha_mcd: f64,
    /// Place stickiness alpha. 0 = disabled.
    pub alpha_place: f64,
    /// VTD stickiness alpha. 0 = disabled.
    pub alpha_vtd: f64,
    /// Directional lambda for GeoSection (0 = no penalty).
    pub directional_lambda: f64,
    /// Enable economic character similarity weights (M.9/M.1). Requires LODES WAC data.
    pub economic_character: bool,
    /// Enable housing character similarity weights (M.3). Requires ACS housing data.
    pub housing_character: bool,
    /// Blend factor for economic character weighter [0.0, 1.0]. Default 0.5.
    /// alpha=1.0 → no effect; alpha=0.0 → fully similarity-driven.
    pub econ_alpha: f64,
    /// Enable administrative zone co-membership weights (M.6). Requires TIGER school districts
    /// and EIA Form 861 spatial join (Phase 2 — not yet implemented).
    pub zone_membership: bool,
    /// Boost factor for zone co-membership weighter. Default 1.0.
    /// score=1 (all zones shared) → w * (1 + zone_alpha). score=0 → w unchanged.
    pub zone_alpha: f64,
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
            economic_character: false,
            housing_character: false,
            econ_alpha: 0.5,
            zone_membership: false,
            zone_alpha: 1.0,
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
    pub engine: bisect_apportion::split::MetisEngine,
}

impl Default for MetisParams {
    fn default() -> Self {
        Self {
            ufactor: 5,
            niter: 100,
            seed: None,
            engine: bisect_apportion::split::MetisEngine::default(),
        }
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
        let engine = args.metis_engine.map(|e| e.into()).unwrap_or_default();
        let metis = MetisParams {
            ufactor: args.ufactor,
            niter: args.niter,
            seed: args.seed,
            engine,
        };
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
                weights: WeightSpec {
                    geographic: false,
                    alpha_county: args.alpha_county,
                    ..WeightSpec::default()
                },
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
                weights: WeightSpec {
                    minority_weighting: true,
                    alpha_county: args.alpha_county,
                    ..WeightSpec::default()
                },
                vertex_constraints: pop_only,
                metis: MetisParams {
                    seed: None,
                    ..metis
                },
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
                seeds: SeedCompositor::Multi {
                    seeds: args.compact_seeds.max(1),
                },
                weights: base_weights,
                vertex_constraints: pop_only,
                metis: MetisParams {
                    seed: None,
                    ..metis
                },
                mode_label: None,
            },
            PM::GeoSection => Self {
                split: SplitStrategy::GeoSection,
                seeds: SeedCompositor::Multi {
                    seeds: args.geosection_seeds.max(1),
                },
                weights: WeightSpec {
                    directional_lambda: 0.0,
                    alpha_county: args.alpha_county,
                    ..WeightSpec::default()
                },
                vertex_constraints: pop_only,
                metis: MetisParams {
                    seed: None,
                    ..metis
                },
                mode_label: None,
            },
            PM::AreaSection => Self {
                split: SplitStrategy::AreaSection {
                    area_swing: args.area_swing,
                    area_section_init: args.area_section_init.into(),
                },
                seeds: SeedCompositor::Multi {
                    seeds: args.geosection_seeds.max(1),
                },
                weights: base_weights,
                vertex_constraints: pop_and_area, // ncon=2: population + land area
                metis: MetisParams {
                    seed: None,
                    ..metis
                },
                mode_label: None,
            },
            PM::ApportionRegions => Self {
                split: SplitStrategy::ApportionRegions,
                seeds: SeedCompositor::Single, // federal statute: single content-derived seed
                weights: base_weights,
                vertex_constraints: pop_only,
                metis: MetisParams {
                    seed: None,
                    ..metis
                },
                mode_label: None,
            },
            PM::ProportionalSection => Self {
                split: SplitStrategy::ProportionalSection { eta: args.eta },
                seeds: SeedCompositor::Multi {
                    seeds: args.geosection_seeds.max(1),
                },
                weights: base_weights,
                vertex_constraints: vec![VertexConstraintKind::Population],
                metis: MetisParams {
                    seed: None,
                    ..metis
                },
                mode_label: None,
            },
            PM::VraSection => Self {
                split: SplitStrategy::VraSection { w_vra: args.w_vra },
                seeds: SeedCompositor::Multi {
                    seeds: args.geosection_seeds.max(1),
                },
                weights: WeightSpec {
                    alpha_county: args.alpha_county,
                    ..WeightSpec::default()
                },
                vertex_constraints: vec![VertexConstraintKind::Population],
                metis: MetisParams {
                    seed: None,
                    ..metis
                },
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
            PM::CentroidalVoronoi => Self {
                split: SplitStrategy::CentroidalVoronoi {
                    n_iter: args.cvd_iters,
                    metric: args
                        .cvd_metric
                        .parse::<crate::bisection_runner::VoronoiMetric>()
                        .unwrap_or(crate::bisection_runner::VoronoiMetric::GraphDistance),
                },
                seeds: SeedCompositor::Single,
                weights: base_weights,
                vertex_constraints: pop_only,
                metis,
                mode_label: None,
            },
            PM::BfsGrowth => Self {
                split: SplitStrategy::BfsGrowth,
                seeds: SeedCompositor::Single,
                weights: base_weights,
                vertex_constraints: pop_only,
                metis,
                mode_label: None,
            },
            PM::Ilp => Self {
                split: SplitStrategy::Ilp {
                    method: args.ilp_method,
                    fallback: args.ilp_fallback,
                    time_limit_secs: args.ilp_time_limit,
                    optimality_gap: args.ilp_gap,
                    max_tracts: args.ilp_max_tracts,
                },
                seeds: SeedCompositor::Single,
                weights: base_weights,
                vertex_constraints: pop_only,
                metis,
                mode_label: None,
            },
            PM::MovingKnife => Self {
                split: SplitStrategy::MovingKnife {
                    n_orientations: args.mka_orientations,
                    metric: args.mka_metric.clone(),
                },
                seeds: SeedCompositor::Single,
                weights: base_weights,
                vertex_constraints: pop_only,
                metis,
                mode_label: None,
            },
            PM::CapacityClustering => Self {
                split: SplitStrategy::CapacityClustering,
                seeds: SeedCompositor::Single,
                weights: base_weights,
                vertex_constraints: pop_only,
                metis,
                mode_label: None,
            },
            PM::Spectral => Self {
                split: SplitStrategy::Spectral {
                    max_iters: args.spectral_iters,
                },
                seeds: SeedCompositor::Single,
                weights: base_weights,
                vertex_constraints: pop_only,
                metis,
                mode_label: None,
            },
            PM::Regionalization => Self {
                split: SplitStrategy::Regionalization,
                seeds: SeedCompositor::Single,
                weights: base_weights,
                vertex_constraints: pop_only,
                metis,
                mode_label: None,
            },
            PM::FlowConstruction => Self {
                split: SplitStrategy::FlowConstruction,
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
        use crate::args::{SearchMode as SeM, StructureMode as SM, WeightMode as WM};

        // Layer 1: structure override
        if let Some(structure) = args.structure {
            let pop_only = vec![VertexConstraintKind::Population];
            let pop_area = vec![VertexConstraintKind::Population, VertexConstraintKind::Area];
            let (new_split, new_vc) = match structure {
                SM::StandardBisect => (SplitStrategy::Bisect, pop_only),
                SM::NWay => (SplitStrategy::NWay, pop_only),
                SM::RatioOptimal => (SplitStrategy::GeoSection, pop_only),
                SM::RatioOptimalArea => (
                    SplitStrategy::AreaSection {
                        area_swing: args.area_swing,
                        area_section_init: args.area_section_init.into(),
                    },
                    pop_area,
                ),
                SM::RatioOptimalVra => (SplitStrategy::VraSection { w_vra: args.w_vra }, pop_only),
                SM::PrimeFactor => (SplitStrategy::ApportionRegions, pop_only),
                SM::CompactPolsby => (SplitStrategy::CompactBisect { epsilon: 0.05 }, pop_only),
                SM::CentroidalVoronoi => (
                    SplitStrategy::CentroidalVoronoi {
                        n_iter: args.cvd_iters,
                        metric: args
                            .cvd_metric
                            .parse::<crate::bisection_runner::VoronoiMetric>()
                            .unwrap_or(crate::bisection_runner::VoronoiMetric::GraphDistance),
                    },
                    pop_only,
                ),
                SM::BfsGrowth => (SplitStrategy::BfsGrowth, pop_only),
                SM::MovingKnife => (
                    SplitStrategy::MovingKnife {
                        n_orientations: args.mka_orientations,
                        metric: args.mka_metric.clone(),
                    },
                    pop_only,
                ),
                SM::CapacityClustering => (SplitStrategy::CapacityClustering, pop_only),
                SM::Spectral => (
                    SplitStrategy::Spectral {
                        max_iters: args.spectral_iters,
                    },
                    pop_only,
                ),
                SM::Regionalization => (SplitStrategy::Regionalization, pop_only),
                SM::FlowConstruction => (SplitStrategy::FlowConstruction, pop_only),
            };
            algo.split = new_split;
            algo.vertex_constraints = new_vc;
        }

        // Layer 2: weight override
        if let Some(weight) = args.weights_override {
            algo.weights = match weight {
                WM::Unweighted => WeightSpec {
                    geographic: false,
                    alpha_county: 0.0,
                    ..WeightSpec::default()
                },
                WM::Geographic => WeightSpec {
                    geographic: true,
                    alpha_county: 0.0,
                    ..WeightSpec::default()
                },
                WM::County => WeightSpec {
                    geographic: true,
                    alpha_county: args.alpha_county.max(1.0),
                    ..WeightSpec::default()
                },
                WM::VraAligned => WeightSpec {
                    geographic: true,
                    minority_weighting: true,
                    ..WeightSpec::default()
                },
                WM::Proportional => WeightSpec {
                    geographic: true,
                    partisan_shares: args.partisan_shares.as_ref().map(std::path::PathBuf::from),
                    ..WeightSpec::default()
                },
                WM::EconomicCharacter => WeightSpec {
                    geographic: true,
                    economic_character: true,
                    econ_alpha: 0.5,
                    ..WeightSpec::default()
                },
                WM::HousingCharacter => WeightSpec {
                    geographic: true,
                    housing_character: true,
                    econ_alpha: 0.5,
                    ..WeightSpec::default()
                },
                WM::ZoneMembership => WeightSpec {
                    geographic: true,
                    zone_membership: true,
                    zone_alpha: 1.0,
                    ..WeightSpec::default()
                },
            };
        }

        // Layer 3: search strategy override
        if let Some(search) = args.search {
            let n = args
                .seeds
                .unwrap_or(args.geosection_seeds.max(args.compact_seeds).max(50));
            algo.seeds = match search {
                SeM::Single => SeedCompositor::Single,
                SeM::Multi => SeedCompositor::Multi { seeds: n },
                SeM::Convergence => SeedCompositor::ConvergenceSweep {
                    threshold: args.convergence_threshold,
                },
                SeM::Percentile => SeedCompositor::Percentile {
                    p: args.percentile.clamp(0.0, 1.0),
                    seeds: n,
                },
                SeM::BisectionEnsemble => SeedCompositor::BisectionEnsemble {
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
                SeM::MultiScaleAdaptive => SeedCompositor::MultiScaleAdaptive {
                    total_steps: args.multiscale_steps,
                    p: args.percentile.clamp(0.0, 1.0),
                    target_accept: args.ms_target_accept,
                    adapt_interval: args.ms_adapt_interval,
                },
                SeM::ParallelTempering => SeedCompositor::ParallelTempering {
                    n_replicas: args.pt_replicas,
                    swap_interval: args.pt_swap_interval,
                    cold_tolerance: args.pt_cold_tol,
                    hot_tolerance: args.pt_hot_tol,
                    steps: args.seeds.unwrap_or(1000),
                    p: args.percentile.clamp(0.0, 1.0),
                },
                SeM::VraRecom => SeedCompositor::VraRecom {
                    steps: args.seeds.unwrap_or(1000),
                    p: args.percentile.clamp(0.0, 1.0),
                    vap_threshold: args.vra_threshold,
                },
                SeM::SmcPercentile => SeedCompositor::SmcPercentile {
                    n_particles: args.particles.unwrap_or(5000),
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
                weights: WeightSpec {
                    geographic: false,
                    ..WeightSpec::default()
                },
                vertex_constraints: pop,
                metis,
                mode_label: Some("unweighted"),
            },
            PM::EdgeWeighted => Self::default(),
            PM::MetisVra => Self {
                split: SplitStrategy::NWay,
                seeds: SeedCompositor::default(),
                weights: WeightSpec {
                    minority_weighting: true,
                    ..WeightSpec::default()
                },
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
                weights: WeightSpec {
                    directional_lambda: 0.0,
                    ..WeightSpec::default()
                },
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
            PM::AreaSection => Self {
                split: SplitStrategy::AreaSection {
                    area_swing: 1.10,
                    area_section_init: AreaSectionInit::RatioOptimal,
                },
                seeds: SeedCompositor::Multi { seeds: 50 },
                weights: WeightSpec::default(),
                vertex_constraints: pop_area, // ncon=2: population + land area
                metis,
                mode_label: None,
            },
            PM::ApportionRegions => Self {
                split: SplitStrategy::ApportionRegions,
                seeds: SeedCompositor::Single, // federal statute: single content-derived seed
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
            PM::CentroidalVoronoi => Self {
                split: SplitStrategy::CentroidalVoronoi {
                    n_iter: 20,
                    metric: crate::bisection_runner::VoronoiMetric::GraphDistance,
                },
                seeds: SeedCompositor::Single,
                weights: WeightSpec::default(),
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
            PM::BfsGrowth => Self {
                split: SplitStrategy::BfsGrowth,
                seeds: SeedCompositor::Single,
                weights: WeightSpec::default(),
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
            PM::Ilp => Self {
                split: SplitStrategy::Ilp {
                    method: crate::args::IlpMethod::FormulationOnly,
                    fallback: crate::args::IlpFallback::Metis,
                    time_limit_secs: 300,
                    optimality_gap: 0.01,
                    max_tracts: 500,
                },
                seeds: SeedCompositor::Single,
                weights: WeightSpec::default(),
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
            PM::MovingKnife => Self {
                split: SplitStrategy::MovingKnife {
                    n_orientations: 180,
                    metric: "reock".to_string(),
                },
                seeds: SeedCompositor::Single,
                weights: WeightSpec::default(),
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
            PM::CapacityClustering => Self {
                split: SplitStrategy::CapacityClustering,
                seeds: SeedCompositor::Single,
                weights: WeightSpec::default(),
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
            PM::Spectral => Self {
                split: SplitStrategy::Spectral { max_iters: 200 },
                seeds: SeedCompositor::Single,
                weights: WeightSpec::default(),
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
            PM::Regionalization => Self {
                split: SplitStrategy::Regionalization,
                seeds: SeedCompositor::Single,
                weights: WeightSpec::default(),
                vertex_constraints: pop,
                metis,
                mode_label: None,
            },
            PM::FlowConstruction => Self {
                split: SplitStrategy::FlowConstruction,
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
    /// Fine resolution level for --search multiscale (default: "tract").
    /// "tract" = Option B; "bg" = Option A or C.
    pub multiscale_fine: String,
    /// Coarse resolution level for --search multiscale (default: "county").
    /// Valid pairs: (tract,county), (bg,tract), (bg,county).
    pub multiscale_coarse: String,
    /// SMC resample threshold for --search smc-percentile (default: 0.5).
    /// Resample when ESS < threshold × n_particles.
    pub smc_resample_threshold: f64,
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
            multiscale_fine: "tract".to_string(),
            multiscale_coarse: "county".to_string(),
            smc_resample_threshold: 0.5,
        }
    }

    /// Returns the effective balance tolerance based on chamber type.
    ///
    /// Priority order:
    /// 1. Explicit `--balance-tolerance` override (always wins)
    /// 2. Chamber-specific value from state policy database
    /// 3. Fallback: 0.5% congressional / 5% state legislative
    pub fn effective_balance_tolerance(&self) -> f64 {
        self.balance_tolerance
            .unwrap_or_else(|| chamber_balance_tolerance(&self.state_code, &self.chamber))
    }

    /// Returns the effective label for this plan run.
    pub fn effective_label(&self) -> String {
        self.label.clone().unwrap_or_else(|| {
            bisect_report::default_label(&self.state_name, &self.chamber, &self.year)
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
/// This ensures `bisect state --state WA --chamber house` automatically uses 98
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
                    format!(
                        " {} has a unicameral legislature — use --chamber house.",
                        state_code
                    )
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
    let mut states: Vec<(String, String, usize)> = manifest
        .states
        .into_iter()
        .filter_map(|(code, state)| {
            let districts = *state.districts.get(year)?;
            if districts == 0 {
                return None;
            }
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
        configs
            .par_iter()
            .map(|cfg| {
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
                        error: Some(format!(
                            "{}: {}",
                            cfg.state_code,
                            ascii_safe(&e.to_string())
                        )),
                        elapsed_ms,
                    },
                }
            })
            .collect()
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
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if let Some(file_year) = extract_year_from_adj_filename(filename) {
        if file_year != requested_year {
            eprintln!(
                "WARNING: Requested adjacency for year {requested_year} but adjacency file is \
                 for year {file_year}: {filename}\n\
                 Census tract boundaries changed between {file_year} and {requested_year} \
                 -- results will use {file_year} geography.\n\
                 For {requested_year} census tracts: run \
                 bisect fetch --year {requested_year} --type adjacency --states {}",
                state_code.to_uppercase()
            );
        }
    }
}

/// Resolve the adjacency pkl path for a state using the manifest.
///
/// The manifest's `local_outputs_dir` + "V3/data/{year}/adjacency/" is the
/// canonical adjacency store — the same path that `bisect fetch --release` downloads to.
/// Override with BISECT_MANIFEST env var for custom data layouts.
///
/// Returns `(path, effective_resolution)` where `effective_resolution` may differ from
/// the requested resolution if a graceful fallback to tract occurred.
fn resolve_adjacency_path(
    state_code_lower: &str,
    year: &str,
    resolution: &str,
) -> Result<(PathBuf, String), String> {
    let manifest = load_manifest().map_err(|e| format!("cannot load manifest: {e}"))?;
    let outputs_dir = PathBuf::from(&manifest.local_outputs_dir);

    // Choose filename based on requested resolution
    let (adj_filename, is_block_group) = match resolution {
        "block_group" | "block-group" => {
            (format!("{state_code_lower}_bg_adjacency_{year}.pkl"), true)
        }
        _ => (format!("{state_code_lower}_adjacency_{year}.pkl"), false),
    };

    // Try V3 then V4 canonical stores
    let canonical = outputs_dir
        .join("V3")
        .join("data")
        .join(year)
        .join("adjacency")
        .join(&adj_filename);
    if canonical.exists() {
        return Ok((canonical, resolution.to_string()));
    }
    let v4 = outputs_dir
        .join("V4")
        .join("data")
        .join(year)
        .join("adjacency")
        .join(&adj_filename);
    if v4.exists() {
        return Ok((v4, resolution.to_string()));
    }

    // Block group not found — graceful fallback to tract with clear warning
    if is_block_group {
        eprintln!(
            "WARNING: --resolution block_group was requested but block_group adjacency \
             not found for {state_code_lower} {year}.\n\
             To get block_group data: bisect fetch --type adjacency --states {} --year {}\n\
             Falling back to tract resolution.",
            state_code_lower.to_uppercase(),
            year
        );
        let tract_filename = format!("{state_code_lower}_adjacency_{year}.pkl");
        let tract_canonical = outputs_dir
            .join("V3")
            .join("data")
            .join(year)
            .join("adjacency")
            .join(&tract_filename);
        if tract_canonical.exists() {
            return Ok((tract_canonical, "tract".to_string()));
        }
        let tract_v4 = outputs_dir
            .join("V4")
            .join("data")
            .join(year)
            .join("adjacency")
            .join(&tract_filename);
        if tract_v4.exists() {
            return Ok((tract_v4, "tract".to_string()));
        }
        let state_upper = state_code_lower.to_uppercase();
        return Err(format!(
            "Adjacency file not found for {state_code_lower} {year}.\n\
             Run: bisect fetch --type adjacency --states {state_upper} --year {year}\n\
             Then: bisect state --state {state_upper} --year {year} ..."
        ));
    }

    let state_upper = state_code_lower.to_uppercase();
    Err(format!(
        "Adjacency file not found for {state_code_lower} {year}.\n\
         Run: bisect fetch --type adjacency --states {state_upper} --year {year}\n\
         Then: bisect state --state {state_upper} --year {year} ..."
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
    bisect_report::check_incomplete_plan(&plan_root, &label)
        .map_err(|e| ascii_safe(&e.to_string()))?;

    // Label collision check: if manifest.json exists and --force not set, exit
    if cfg.label.is_some() {
        let manifest_path = plan_root.join("manifest.json");
        bisect_report::check_plan_collision(&plan_root, cfg.force)
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
            std::fs::remove_dir_all(&data_dir).map_err(|e| format!("reset failed: {e}"))?;
        }
    }
    // Create plan directory structure if labeled
    if cfg.label.is_some() {
        bisect_report::create_plan_dir(&year_base, &label)
            .map_err(|e| format!("cannot create plan dir: {e}"))?;
    }
    std::fs::create_dir_all(&data_dir).map_err(|e| format!("cannot create data dir: {e}"))?;

    status(
        cfg.position,
        &format!("{}: loading adjacency", cfg.state_code),
    );

    // 1. Load adjacency graph
    // Adjacency path comes from the manifest (same source as `bisect fetch`).
    // The manifest's local_outputs_dir + "V3/data/{year}/adjacency/" is the
    // canonical store. BISECT_MANIFEST can override this for custom setups.
    let state_code_lower = cfg.state_code.to_lowercase();
    let adj_pkl = if let Some(ref override_path) = cfg.adjacency_override {
        override_path.clone()
    } else {
        let (path, _effective_resolution) =
            resolve_adjacency_path(&state_code_lower, &cfg.year, &cfg.resolution)?;
        // Task 135: warn when adjacency file year doesn't match requested year
        check_adjacency_year_mismatch(&path, &cfg.year, &cfg.state_code);
        path
    };

    let graph = load_adjacency_pkl(&adj_pkl).map_err(|e| format!("adjacency load failed: {e}"))?;

    // Check for isolated nodes (no adjacency neighbors) — common with island tracts.
    // Isolated tracts will always form non-contiguous districts.
    let isolated: Vec<usize> = graph
        .adjacency
        .iter()
        .enumerate()
        .filter(|(_, nbrs)| nbrs.is_empty())
        .map(|(i, _)| i)
        .collect();
    if !isolated.is_empty() {
        eprintln!(
            "WARNING: {}: {} isolated tract(s) with no adjacency neighbors. \
             These will form non-contiguous districts. \
             For island states (AK, HI, international), rebuild adjacency with water bridges.",
            cfg.state_code,
            isolated.len()
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
        && cfg.algo.weights.alpha_mcd < 1e-10
        && cfg.algo.weights.alpha_place < 1e-10
        && cfg.algo.weights.alpha_vtd < 1e-10
    {
        status(
            cfg.position,
            &format!("{}: unweighted mode", cfg.state_code),
        );
        HashMap::new()
    } else if num_districts == 1 {
        status(
            cfg.position,
            &format!("{}: single district — skipping weighting", cfg.state_code),
        );
        graph.edge_weights.clone()
    } else {
        build_edge_weights(
            &cfg.algo.weights,
            &graph,
            &cfg.state_code,
            state_name,
            &cfg.year,
            &cfg.output_dir,
            cfg.position,
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
    let needs_area = cfg
        .algo
        .vertex_constraints
        .contains(&VertexConstraintKind::Area);
    let tiger_areas: Vec<f64> = if needs_area {
        let (areas, _) = load_tiger_geometry(
            &cfg.state_code,
            &cfg.year,
            &graph.index_to_geoid,
            &graph.adjacency,
            &graph.edge_weights,
        );
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
            status(
                cfg.position,
                &format!(
                    "{}: balance retry {}/{} (seed {:?})",
                    cfg.state_code, attempt, MAX_BALANCE_RETRIES, seed
                ),
            );
        }

        let partition_t0 = std::time::Instant::now();
        let assignments_attempt = match &cfg.algo.split {
            SplitStrategy::NWay if num_districts > 1 => {
                status(
                    cfg.position,
                    &format!("{}: n-way into {} districts", cfg.state_code, num_districts),
                );
                run_nway_partition(
                    &graph.adjacency,
                    &vwgt,
                    &edge_weights,
                    num_districts,
                    1.0 + ufactor as f64 / 1000.0,
                    niter,
                    seed,
                )
                .map_err(|e| format!("n-way partition failed: {e}"))?
            }
            SplitStrategy::GeoSection => {
                let seeds_per_ratio = cfg.algo.seeds.seed_count();
                let lambda = cfg.algo.weights.directional_lambda;
                let centroids = if lambda > 1e-10 {
                    crate::geosection_orientation::load_centroids_from_tiger(
                        &cfg.state_code,
                        &cfg.year,
                        &graph.index_to_geoid,
                    )
                } else {
                    crate::geosection_orientation::CentroidMap::new()
                };
                if lambda > 1e-10 {
                    status(
                        cfg.position,
                        &format!(
                            "{}: GeoSection λ={:.1} ({} centroids loaded)",
                            cfg.state_code,
                            lambda,
                            centroids.len()
                        ),
                    );
                } else {
                    status(
                        cfg.position,
                        &format!(
                            "{}: GeoSection {} ratios × {} seeds",
                            cfg.state_code,
                            num_districts / 2,
                            seeds_per_ratio
                        ),
                    );
                }
                let (asgn, nat_left, nat_right, nat_ec) = run_geosection(
                    &graph.adjacency,
                    &vwgt,
                    &edge_weights,
                    num_districts,
                    balance_tolerance_frac,
                    niter,
                    seeds_per_ratio,
                    Some(&intermediate_dir),
                    &centroids,
                    lambda,
                    None,
                    1.10,
                    None,
                    0.0,
                    None, // GeoSection does not use MKA override
                )
                .map_err(|e| format!("geosection failed: {e}"))?;
                status(
                    cfg.position,
                    &format!(
                        "{}: natural ratio {}:{} at {:.0}km",
                        cfg.state_code,
                        nat_left,
                        nat_right,
                        nat_ec / 1000.0
                    ),
                );
                asgn
            }
            SplitStrategy::AreaSection {
                area_swing,
                area_section_init,
            } => {
                let seeds_per_ratio = cfg.algo.seeds.seed_count();
                if tiger_areas.is_empty() {
                    return Err(format!(
                        "{}: AreaSection requires TIGER ALAND data — not found",
                        cfg.state_code
                    ));
                }

                // ── MKA warm-start: compute theta* for directional edge pre-bias ──
                //
                // When MKA init is active and centroid data is present, call
                // split_subgraph_mka_direction() to get the Reock-optimal cut angle.
                // That angle is then passed to run_geosection as mka_theta_override,
                // which uses it to pre-bias edge weights via apply_directional_penalty.
                //
                // When centroids are absent, fall back to ratio-optimal with a warning.
                let mka_theta_override: Option<f64> = if *area_section_init
                    == AreaSectionInit::MovingKnife
                {
                    if graph.tract_centroids.is_empty() {
                        eprintln!("WARNING: --area-section-init moving-knife requires tract centroids; falling back to ratio-optimal");
                        None
                    } else {
                        let all_tracts: std::collections::HashSet<usize> =
                            (0..graph.adjacency.len()).collect();
                        let theta = crate::bisection_runner::split_subgraph_mka_direction(
                            &all_tracts,
                            &graph.tract_centroids,
                            180,
                        );
                        eprintln!("[areasection-mka] theta*={:.4} rad ({:.1} deg) — using as directional bias",
                              theta, theta.to_degrees());
                        Some(theta)
                    }
                } else {
                    None
                };

                // When MKA init is active and centroids are available, build a CentroidMap
                // (HashMap<usize, (f64, f64)>) so run_geosection can apply the directional bias
                // via apply_directional_penalty. This converts the Vec<(f64,f64)> to the map form.
                let mka_centroid_map: crate::geosection_orientation::CentroidMap =
                    if mka_theta_override.is_some() && !graph.tract_centroids.is_empty() {
                        graph
                            .tract_centroids
                            .iter()
                            .enumerate()
                            .map(|(i, &pt)| (i, pt))
                            .collect()
                    } else {
                        crate::geosection_orientation::CentroidMap::new()
                    };
                let empty_centroids = crate::geosection_orientation::CentroidMap::new();
                let (centroids_ref, lambda_val): (
                    &crate::geosection_orientation::CentroidMap,
                    f64,
                ) = if mka_theta_override.is_some() && !graph.tract_centroids.is_empty() {
                    (&mka_centroid_map, 1.0) // lambda=1.0: moderate directional bias
                } else {
                    (&empty_centroids, 0.0)
                };
                status(
                    cfg.position,
                    &format!(
                        "{}: AreaSection {} ratios x {} seeds (pop+area dual, ncon=2, init={:?})",
                        cfg.state_code,
                        num_districts / 2,
                        seeds_per_ratio,
                        area_section_init
                    ),
                );
                let (asgn, nat_left, nat_right, nat_ec) = run_geosection(
                    &graph.adjacency,
                    &graph.vertex_weights,
                    &edge_weights,
                    num_districts,
                    balance_tolerance_frac,
                    niter,
                    seeds_per_ratio,
                    Some(&intermediate_dir),
                    centroids_ref,
                    lambda_val,
                    Some(&tiger_areas),
                    *area_swing,
                    None,
                    0.0,
                    mka_theta_override,
                )
                .map_err(|e| format!("areasection failed: {e}"))?;
                status(
                    cfg.position,
                    &format!(
                        "{}: natural ratio {}:{} at {:.0}km",
                        cfg.state_code,
                        nat_left,
                        nat_right,
                        nat_ec / 1000.0
                    ),
                );
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
                    .join(&cfg.year)
                    .join("demographics")
                    .join(format!("{state_name}_demographics_{}.csv", cfg.year));
                let minority_vap_vec: Vec<f64> = if demo_path.exists() {
                    let demo = crate::demographics::load_demographics(&demo_path).map_err(|e| {
                        format!(
                            "{}: VRASection demographics load failed: {e}",
                            cfg.state_code
                        )
                    })?;
                    let fracs = crate::demographics::align_demographics_to_adjacency(
                        &demo,
                        &graph.index_to_geoid,
                        graph.n_vertices,
                    );
                    // Convert fraction × population → approximate minority VAP count
                    fracs
                        .iter()
                        .zip(graph.vertex_weights.iter())
                        .map(|(&frac, &pop)| frac * pop as f64)
                        .collect()
                } else {
                    eprintln!("WARNING: {}: VRASection demographics not found at {} — running as plain GeoSection",
                          cfg.state_code, demo_path.display());
                    vec![]
                };
                let mvap_opt: Option<&[f64]> = if minority_vap_vec.is_empty() {
                    None
                } else {
                    Some(&minority_vap_vec)
                };
                let empty_centroids = crate::geosection_orientation::CentroidMap::new();
                status(
                    cfg.position,
                    &format!(
                        "{}: VRASection {} ratios x {} seeds w_vra={:.2}",
                        cfg.state_code,
                        num_districts / 2,
                        seeds_per_ratio,
                        w_vra
                    ),
                );
                let (asgn, nat_left, nat_right, nat_ec) = run_geosection(
                    &graph.adjacency,
                    &vwgt,
                    &edge_weights,
                    num_districts,
                    balance_tolerance_frac,
                    niter,
                    seeds_per_ratio,
                    Some(&intermediate_dir),
                    &empty_centroids,
                    0.0,
                    None,
                    1.10,
                    mvap_opt,
                    *w_vra,
                    None, // VRASection does not use MKA override
                )
                .map_err(|e| format!("vra-section failed: {e}"))?;
                status(
                    cfg.position,
                    &format!(
                        "{}: natural ratio {}:{} at {:.0}km",
                        cfg.state_code,
                        nat_left,
                        nat_right,
                        nat_ec / 1000.0
                    ),
                );
                asgn
            }
            SplitStrategy::ProportionalSection { eta } => {
                let seeds = cfg.algo.seeds.seed_count();
                // Load D_votes from presidential_by_tract.csv
                let election_path = std::path::PathBuf::from(format!(
                    "data/{}/elections/presidential_by_tract.csv",
                    cfg.year
                ));
                if !election_path.exists() {
                    return Err(format!(
                        "{}: ProportionalSection requires {} — not found",
                        cfg.state_code,
                        election_path.display()
                    ));
                }
                let (d_votes, two_party) = crate::partisan_shares::load_dem_vote_counts(
                    &election_path,
                    &graph.index_to_geoid,
                    graph.n_vertices,
                )
                .map_err(|e| format!("{}: load_dem_vote_counts failed: {e}", cfg.state_code))?;
                status(
                    cfg.position,
                    &format!(
                        "{}: ProportionalSection {} seeds eta={:.2} (pop+D_votes ncon=2)",
                        cfg.state_code, seeds, eta
                    ),
                );
                let (asgn, k_d, k_r, best_ec, d_state) =
                    crate::bisection_runner::run_proportional_section(
                        &graph.adjacency,
                        &graph.vertex_weights,
                        &d_votes,
                        &two_party,
                        &edge_weights,
                        num_districts,
                        balance_tolerance_frac,
                        niter,
                        seeds,
                        *eta,
                        Some(&intermediate_dir),
                    )
                    .map_err(|e| format!("proportional-section failed: {e}"))?;
                status(
                    cfg.position,
                    &format!(
                        "{}: proportional {}/{}D d={:.3} EC={:.0}km",
                        cfg.state_code,
                        k_d,
                        k_r,
                        d_state,
                        best_ec / 1000.0
                    ),
                );
                asgn
            }
            SplitStrategy::CompactBisect { epsilon } => {
                let seeds_per_level = cfg.algo.seeds.seed_count();
                let (vertex_areas, vertex_ext_perimeters) = load_tiger_geometry(
                    &cfg.state_code,
                    &cfg.year,
                    &graph.index_to_geoid,
                    &graph.adjacency,
                    &edge_weights,
                );
                let opts = CompactBisectOpts {
                    seeds_per_level,
                    epsilon: *epsilon,
                };
                run_all_splits_compact(
                    &graph.adjacency,
                    &vwgt,
                    &edge_weights,
                    &vertex_areas,
                    &vertex_ext_perimeters,
                    num_districts,
                    balance_tolerance_frac,
                    niter,
                    None,
                    &opts,
                    Some(&intermediate_dir),
                )
                .map_err(|e| format!("compact-bisect failed: {e}"))?
            }
            SplitStrategy::ApportionRegions => {
                use bisect_apportion::{pfr_tree_depth, MetisPartitioner, PfrCompositor};
                let factor_seq = bisect_apportion::prime_factor_sequence(num_districts as u32);
                let depth = pfr_tree_depth(num_districts as u32).max(1);
                status(
                    cfg.position,
                    &format!(
                        "{}: apportion-regions partition into {} districts \
                (F={:?}, depth={})",
                        cfg.state_code, num_districts, factor_seq, depth
                    ),
                );
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
                let result = compositor
                    .compose(
                        &graph.adjacency,
                        &graph.vertex_weights,
                        &edge_weights,
                        num_districts as u32,
                        seed,
                    )
                    .map_err(|e| format!("apportion-regions failed: {e}"))?;
                // Check balance at relaxed 2% — report actual deviation in manifest.
                let pfr_assignments: std::collections::HashMap<usize, usize> = result
                    .assignment
                    .iter()
                    .enumerate()
                    .map(|(t, &d)| (t, d as usize + 1))
                    .collect();
                let pfr_partition =
                    bisect_core::Partition::from_assignments(pfr_assignments.clone());
                let pfr_balance =
                    pfr_partition.population_balance(&graph.vertex_weights, num_districts);
                if pfr_balance > 0.03 {
                    return Err(format!(
                        "apportion-regions balance {:.1}% exceeds 3% research limit",
                        pfr_balance * 100.0
                    ));
                }
                status(
                    cfg.position,
                    &format!(
                        "{}: balance {:.2}% (cache hits={})",
                        cfg.state_code,
                        pfr_balance * 100.0,
                        result.cache_hits
                    ),
                );
                pfr_assignments
            }
            SplitStrategy::SimulatedAnnealing {
                steps_per_tract,
                t0_factor,
                t_final,
            } => {
                let base_seed_val = seed.unwrap_or(0);
                status(cfg.position, &format!(
                "{}: simulated-annealing {} steps/tract t0_factor={:.4} t_final={:.2e} into {} districts",
                cfg.state_code, steps_per_tract, t0_factor, t_final, num_districts));
                crate::bisection_runner::run_all_splits_sa(
                    &graph.adjacency,
                    &vwgt,
                    &edge_weights,
                    num_districts,
                    balance_tolerance_frac,
                    niter,
                    seed,
                    Some(&intermediate_dir),
                    *steps_per_tract,
                    *t0_factor,
                    *t_final,
                    base_seed_val,
                )
                .map_err(|e| format!("simulated-annealing failed: {e}"))?
            }
            SplitStrategy::BfsGrowth => {
                let base_seed_val = seed.unwrap_or(0);
                status(
                    cfg.position,
                    &format!(
                        "{}: bfs-growth into {} districts",
                        cfg.state_code, num_districts
                    ),
                );
                crate::bisection_runner::run_all_splits_bfs(
                    &graph.adjacency,
                    &vwgt,
                    num_districts,
                    balance_tolerance_frac,
                    Some(&intermediate_dir),
                    base_seed_val,
                )
                .map_err(|e| format!("bfs-growth: {e}"))?
            }
            SplitStrategy::Ilp {
                method,
                fallback,
                time_limit_secs,
                optimality_gap,
                max_tracts,
            } => {
                status(cfg.position, &format!(
                "{}: ilp exact redistricting (method={} fallback={} time_limit={}s gap={:.3} max_tracts={}) into {} districts",
                cfg.state_code, method, fallback, time_limit_secs, optimality_gap, max_tracts, num_districts));
                let ilp_report_dir = intermediate_dir.join("ilp_solve_reports");
                crate::bisection_runner::run_all_splits_ilp(
                    &graph.adjacency,
                    &vwgt,
                    &edge_weights,
                    num_districts,
                    balance_tolerance_frac,
                    *method,
                    *fallback,
                    *time_limit_secs,
                    *optimality_gap,
                    *max_tracts,
                    Some(&ilp_report_dir),
                )
                .map_err(|e| format!("ilp: {e}"))?
            }
            SplitStrategy::MovingKnife {
                n_orientations,
                metric,
            } => {
                let base_seed_val = seed.unwrap_or(0);
                if graph.tract_centroids.is_empty() {
                    return Err(
                        "[CONFIG] --structure moving-knife requires tract centroid data. \
                     Run: bisect fetch --type centroids"
                            .to_string(),
                    );
                }
                let metric_enum = if metric == "polsby" {
                    crate::bisection_runner::MkaMetric::PolsbyPopper
                } else {
                    crate::bisection_runner::MkaMetric::Reock
                };
                status(
                    cfg.position,
                    &format!(
                        "{}: moving-knife ({} orientations, metric={}) into {} districts",
                        cfg.state_code, n_orientations, metric, num_districts
                    ),
                );
                crate::bisection_runner::run_all_splits_mka(
                    &graph.adjacency,
                    &vwgt,
                    num_districts,
                    balance_tolerance_frac,
                    Some(&intermediate_dir),
                    *n_orientations,
                    metric_enum,
                    base_seed_val,
                    &graph.tract_centroids,
                )
                .map_err(|e| format!("moving-knife: {e}"))?
            }
            SplitStrategy::CentroidalVoronoi { n_iter, metric } => {
                use crate::bisection_runner::VoronoiMetric;
                let base_seed_val = seed.unwrap_or(0);
                let metric_name = match metric {
                    VoronoiMetric::GraphDistance => "graph-distance",
                    VoronoiMetric::Geographic => "geographic",
                };
                if *metric == VoronoiMetric::Geographic && graph.tract_centroids.is_empty() {
                    return Err("[CONFIG] --cvd-metric geographic requires centroid data. \
                            Run: bisect fetch --type centroids"
                        .to_string());
                }
                status(
                    cfg.position,
                    &format!(
                        "{}: centroidal-voronoi ({}) {} iters into {} districts",
                        cfg.state_code, metric_name, n_iter, num_districts
                    ),
                );
                crate::bisection_runner::run_all_splits_cvd(
                    &graph.adjacency,
                    &vwgt,
                    num_districts,
                    balance_tolerance_frac,
                    Some(&intermediate_dir),
                    *n_iter,
                    base_seed_val,
                    *metric,
                    &graph.tract_centroids,
                )
                .map_err(|e| format!("centroidal-voronoi: {e}"))?
            }
            SplitStrategy::CapacityClustering => {
                status(
                    cfg.position,
                    &format!(
                        "{}: capacity-clustering into {} districts",
                        cfg.state_code, num_districts
                    ),
                );
                let result = bisect_clustering::capacity_cluster_repaired(
                    &graph.adjacency,
                    &graph.vertex_weights,
                    bisect_clustering::ClusterConfig {
                        k: num_districts,
                        tolerance: balance_tolerance_frac,
                    },
                )
                .map_err(|e| format!("capacity-clustering failed: {e}"))?;
                let summary_path = intermediate_dir.join("capacity_clustering_summary.json");
                std::fs::write(
                    &summary_path,
                    serde_json::to_string_pretty(&result.summary).map_err(|e| {
                        format!("serialize capacity-clustering summary failed: {e}")
                    })?,
                )
                .map_err(|e| format!("write capacity-clustering summary failed: {e}"))?;
                if result.status != bisect_clustering::ClusterStatus::Valid {
                    return Err(format!(
                        "[ALGO] capacity-clustering did not produce a valid plan: {:?}",
                        result.status
                    ));
                }
                result
                    .assignment
                    .into_iter()
                    .enumerate()
                    .map(|(idx, district)| (idx, district + 1))
                    .collect()
            }
            SplitStrategy::Spectral { max_iters } => {
                status(
                    cfg.position,
                    &format!(
                        "{}: spectral recursive bisection into {} districts",
                        cfg.state_code, num_districts
                    ),
                );
                let (spectral_assignment, summary) = run_spectral_recursive(
                    &graph.adjacency,
                    &graph.vertex_weights,
                    num_districts,
                    balance_tolerance_frac,
                    *max_iters,
                )?;
                let summary_path = intermediate_dir.join("spectral_summary.json");
                std::fs::write(
                    &summary_path,
                    serde_json::to_string_pretty(&summary)
                        .map_err(|e| format!("serialize spectral summary failed: {e}"))?,
                )
                .map_err(|e| format!("write spectral summary failed: {e}"))?;
                spectral_assignment
                    .into_iter()
                    .enumerate()
                    .map(|(idx, district)| (idx, district + 1))
                    .collect()
            }
            SplitStrategy::Regionalization => {
                status(
                    cfg.position,
                    &format!(
                        "{}: regionalization into {} districts",
                        cfg.state_code, num_districts
                    ),
                );
                let result = bisect_clustering::regionalize(
                    &graph.adjacency,
                    &graph.vertex_weights,
                    bisect_clustering::ClusterConfig {
                        k: num_districts,
                        tolerance: balance_tolerance_frac,
                    },
                )
                .map_err(|e| format!("regionalization failed: {e}"))?;
                let summary_path = intermediate_dir.join("regionalization_summary.json");
                std::fs::write(
                    &summary_path,
                    serde_json::to_string_pretty(&result.summary)
                        .map_err(|e| format!("serialize regionalization summary failed: {e}"))?,
                )
                .map_err(|e| format!("write regionalization summary failed: {e}"))?;
                let merge_path = intermediate_dir.join("regionalization_merges.json");
                std::fs::write(
                    &merge_path,
                    serde_json::to_string_pretty(&result.merge_log)
                        .map_err(|e| format!("serialize regionalization merges failed: {e}"))?,
                )
                .map_err(|e| format!("write regionalization merges failed: {e}"))?;
                if result.status != bisect_clustering::ClusterStatus::Valid {
                    return Err(format!(
                        "[ALGO] regionalization did not produce a valid plan: {:?}",
                        result.status
                    ));
                }
                result
                    .assignment
                    .into_iter()
                    .enumerate()
                    .map(|(idx, district)| (idx, district + 1))
                    .collect()
            }
            SplitStrategy::FlowConstruction => {
                status(
                    cfg.position,
                    &format!(
                        "{}: flow construction into {} districts",
                        cfg.state_code, num_districts
                    ),
                );
                let result = bisect_flow::construct_flow(
                    &graph.adjacency,
                    &graph.vertex_weights,
                    bisect_flow::FlowConfig::new(num_districts, balance_tolerance_frac),
                )
                .map_err(|e| format!("flow construction failed: {e}"))?;
                let summary_path = intermediate_dir.join("flow_construction_summary.json");
                std::fs::write(
                    &summary_path,
                    serde_json::to_string_pretty(&result.summary)
                        .map_err(|e| format!("serialize flow summary failed: {e}"))?,
                )
                .map_err(|e| format!("write flow summary failed: {e}"))?;
                if result.status != bisect_flow::FlowStatus::Valid {
                    return Err(format!(
                        "[ALGO] flow construction did not produce a valid plan: {:?}",
                        result.status
                    ));
                }
                result
                    .assignment
                    .into_iter()
                    .enumerate()
                    .map(|(idx, district)| (idx, district + 1))
                    .collect()
            }
            _ => {
                match &cfg.algo.seeds {
                    SeedCompositor::Percentile { p, seeds } => {
                        let base = seed.unwrap_or(0);
                        status(
                            cfg.position,
                            &format!(
                                "{}: percentile-sweep p={:.2} {} seeds into {} districts",
                                cfg.state_code, p, seeds, num_districts
                            ),
                        );
                        run_all_splits_percentile(
                            &graph.adjacency,
                            &vwgt,
                            &edge_weights,
                            num_districts,
                            balance_tolerance_frac,
                            niter,
                            base,
                            *seeds,
                            *p,
                            Some(&intermediate_dir),
                        )
                        .map_err(|e| format!("percentile-sweep failed: {e}"))?
                    }
                    SeedCompositor::BisectionEnsemble { p, ensemble_steps } => {
                        status(
                            cfg.position,
                            &format!(
                                "{}: bisection-ensemble p={:.2} {} steps/node into {} districts",
                                cfg.state_code, p, ensemble_steps, num_districts
                            ),
                        );
                        run_all_splits_with_search(
                            &graph.adjacency,
                            &vwgt,
                            &edge_weights,
                            num_districts,
                            balance_tolerance_frac,
                            niter,
                            seed,
                            Some(&intermediate_dir),
                            Some((*p, *ensemble_steps)),
                        )
                        .map_err(|e| format!("bisection-ensemble failed: {e}"))?
                    }
                    SeedCompositor::Flip { flip_steps, p } => {
                        let base = seed.unwrap_or(0);
                        status(
                            cfg.position,
                            &format!(
                                "{}: flip-chain p={:.2} {} steps into {} districts",
                                cfg.state_code, p, flip_steps, num_districts
                            ),
                        );
                        let (result, visited_count, selected_rank) = run_flip_chain(
                            &graph.adjacency,
                            &vwgt,
                            &edge_weights,
                            num_districts,
                            balance_tolerance_frac,
                            *flip_steps,
                            base,
                            *p,
                        )
                        .map_err(|e| format!("flip-chain failed: {e}"))?;
                        flip_visited_count = Some(visited_count);
                        flip_selected_rank = Some(selected_rank);
                        result
                    }
                    SeedCompositor::ShortBurst {
                        burst_length,
                        n_bursts,
                        p,
                    } => {
                        let base = seed.unwrap_or(0);
                        status(
                            cfg.position,
                            &format!(
                                "{}: short-burst p={:.2} {} bursts x {} steps into {} districts",
                                cfg.state_code, p, n_bursts, burst_length, num_districts
                            ),
                        );
                        let (result, b_seeds, b_idx) = run_short_burst(
                            &graph.adjacency,
                            &vwgt,
                            &edge_weights,
                            num_districts,
                            balance_tolerance_frac,
                            niter,
                            base,
                            *burst_length,
                            *n_bursts,
                            *p,
                        )
                        .map_err(|e| format!("short-burst failed: {e}"))?;
                        short_burst_burst_seeds = Some(b_seeds);
                        short_burst_selected_burst_idx = Some(b_idx);
                        result
                    }
                    SeedCompositor::ShortBurstForest {
                        burst_length,
                        n_bursts,
                        p,
                    } => {
                        let base = seed.unwrap_or(0);
                        status(cfg.position, &format!("{}: short-burst-forest p={:.2} {} bursts x {} steps into {} districts",
                           cfg.state_code, p, n_bursts, burst_length, num_districts));
                        run_short_burst_forest(
                            &graph.adjacency,
                            &vwgt,
                            &edge_weights,
                            num_districts,
                            balance_tolerance_frac,
                            niter,
                            base,
                            *burst_length,
                            *n_bursts,
                            *p,
                        )
                        .map_err(|e| format!("short-burst-forest: {e}"))?
                    }
                    SeedCompositor::ShortBurstMergeSplit {
                        burst_length,
                        n_bursts,
                        p,
                    } => {
                        let base = seed.unwrap_or(0);
                        status(cfg.position, &format!("{}: short-burst-merge-split p={:.2} {} bursts x {} steps into {} districts",
                           cfg.state_code, p, n_bursts, burst_length, num_districts));
                        run_short_burst_merge_split(
                            &graph.adjacency,
                            &vwgt,
                            &edge_weights,
                            num_districts,
                            balance_tolerance_frac,
                            niter,
                            base,
                            *burst_length,
                            *n_bursts,
                            *p,
                        )
                        .map_err(|e| format!("short-burst-merge-split: {e}"))?
                    }
                    SeedCompositor::ForestRecom { steps, p } => {
                        let base = seed.unwrap_or(0);
                        status(
                            cfg.position,
                            &format!(
                                "{}: forest-recom p={:.2} {} steps into {} districts",
                                cfg.state_code, p, steps, num_districts
                            ),
                        );
                        run_forest_recom(
                            &graph.adjacency,
                            &vwgt,
                            &edge_weights,
                            num_districts,
                            balance_tolerance_frac,
                            niter,
                            base,
                            *steps,
                            *p,
                        )
                        .map_err(|e| format!("forest-recom failed: {e}"))?
                    }
                    SeedCompositor::MultiScale {
                        total_steps,
                        p,
                        alpha,
                    } => {
                        let base = seed.unwrap_or(0);
                        let fine = cfg.multiscale_fine.as_str();
                        let coarse = cfg.multiscale_coarse.as_str();
                        status(cfg.position, &format!(
                        "{}: multiscale({fine}->{coarse}) p={:.2} {} steps alpha={:.2} into {} districts",
                        cfg.state_code, p, total_steps, alpha, num_districts
                    ));
                        // Load BG adjacency for Options A (bg->tract) and C (bg->county)
                        let bg_graph_opt = if fine == "bg" || fine == "block_group" {
                            match resolve_adjacency_path(&state_code_lower, &cfg.year, "block_group") {
                            Ok((bg_path, _)) => {
                                match crate::adjacency_loader::load_adjacency_pkl(&bg_path) {
                                    Ok(g) => Some(g),
                                    Err(e) => return Err(format!(
                                        "multiscale: failed to load block-group adjacency for {}: {e}", cfg.state_code
                                    )),
                                }
                            }
                            Err(e) => return Err(format!(
                                "multiscale: --multiscale-fine bg requires block-group adjacency for {} {}: {e}",
                                cfg.state_code, cfg.year
                            )),
                        }
                        } else {
                            None
                        };
                        use crate::bisection_runner::MultiscaleFineLevel;
                        let fine_level = MultiscaleFineLevel::from_str(fine)
                            .map_err(|e| format!("multiscale: {e}"))?;
                        run_multiscale(
                            &graph.adjacency,
                            &vwgt,
                            &edge_weights,
                            num_districts,
                            balance_tolerance_frac,
                            niter,
                            base,
                            *total_steps,
                            *alpha,
                            *p,
                            Some(&graph.index_to_geoid),
                            fine_level,
                            coarse,
                            bg_graph_opt.as_ref().map(|g| {
                                (
                                    g.adjacency.as_slice(),
                                    g.vertex_weights.as_slice(),
                                    &g.index_to_geoid,
                                )
                            }),
                        )
                        .map_err(|e| format!("multiscale: {e}"))?
                    }
                    SeedCompositor::MultiScaleAdaptive {
                        total_steps,
                        p,
                        target_accept,
                        adapt_interval,
                    } => {
                        let base = seed.unwrap_or(0);
                        let fine = cfg.multiscale_fine.as_str();
                        let coarse = cfg.multiscale_coarse.as_str();
                        status(cfg.position, &format!(
                        "{}: multiscale-adaptive({fine}->{coarse}) p={:.2} {} steps target_accept={:.2} adapt_interval={} into {} districts",
                        cfg.state_code, p, total_steps, target_accept, adapt_interval, num_districts
                    ));
                        // Load BG adjacency for Options A and C
                        let bg_graph_opt = if fine == "bg" || fine == "block_group" {
                            match resolve_adjacency_path(&state_code_lower, &cfg.year, "block_group") {
                            Ok((bg_path, _)) => {
                                match crate::adjacency_loader::load_adjacency_pkl(&bg_path) {
                                    Ok(g) => Some(g),
                                    Err(e) => return Err(format!(
                                        "multiscale-adaptive: failed to load block-group adjacency for {}: {e}", cfg.state_code
                                    )),
                                }
                            }
                            Err(e) => return Err(format!(
                                "multiscale-adaptive: --multiscale-fine bg requires block-group adjacency for {} {}: {e}",
                                cfg.state_code, cfg.year
                            )),
                        }
                        } else {
                            None
                        };
                        use crate::bisection_runner::MultiscaleFineLevel;
                        let fine_level = MultiscaleFineLevel::from_str(fine)
                            .map_err(|e| format!("multiscale-adaptive: {e}"))?;
                        let acfg = AdaptiveConfig {
                            total_steps: *total_steps,
                            target_accept: *target_accept,
                            initial_alpha: *target_accept, // start at target
                            adapt_interval: *adapt_interval,
                            gamma_0: 0.10,
                            pop_tolerance: balance_tolerance_frac,
                            coarse_tol_factor: 3.0,
                            p: *p,
                        };
                        let (plan, _adaptive_result) = run_multiscale_adaptive(
                            &graph.adjacency,
                            &vwgt,
                            &edge_weights,
                            num_districts,
                            niter,
                            base,
                            acfg,
                            Some(&graph.index_to_geoid),
                            fine_level,
                            coarse,
                            bg_graph_opt.as_ref().map(|g| {
                                (
                                    g.adjacency.as_slice(),
                                    g.vertex_weights.as_slice(),
                                    &g.index_to_geoid,
                                )
                            }),
                        )
                        .map_err(|e| format!("multiscale-adaptive: {e}"))?;
                        plan
                    }
                    SeedCompositor::MergeSplit { steps, p } => {
                        let base = seed.unwrap_or(0);
                        status(
                            cfg.position,
                            &format!(
                                "{}: merge-split p={:.2} {} steps into {} districts",
                                cfg.state_code, p, steps, num_districts
                            ),
                        );
                        run_merge_split(
                            &graph.adjacency,
                            &vwgt,
                            &edge_weights,
                            num_districts,
                            balance_tolerance_frac,
                            niter,
                            base,
                            *steps,
                            *p,
                        )
                        .map_err(|e| format!("merge-split failed: {e}"))?
                    }
                    SeedCompositor::ParallelTempering {
                        n_replicas,
                        swap_interval,
                        cold_tolerance,
                        hot_tolerance,
                        steps,
                        p,
                    } => {
                        let base = seed.unwrap_or(0);
                        status(
                            cfg.position,
                            &format!(
                        "{}: parallel-tempering p={:.2} {} steps {} replicas into {} districts",
                        cfg.state_code, p, steps, n_replicas, num_districts
                    ),
                        );
                        run_parallel_tempering(
                            &graph.adjacency,
                            &vwgt,
                            &edge_weights,
                            num_districts,
                            niter,
                            base,
                            *n_replicas,
                            *swap_interval,
                            *cold_tolerance,
                            *hot_tolerance,
                            *steps,
                            *p,
                        )
                        .map_err(|e| format!("parallel-tempering: {e}"))?
                    }
                    SeedCompositor::VraRecom {
                        steps,
                        p,
                        vap_threshold,
                    } => {
                        let base = seed.unwrap_or(0);
                        // Load minority VAP fractions from demographics CSV (same as VRASection).
                        // Minority fraction = (total_pop - white_non_hispanic) / total_pop.
                        let demo_path = std::path::Path::new("data")
                            .join(&cfg.year)
                            .join("demographics")
                            .join(format!("{state_name}_demographics_{}.csv", cfg.year));
                        let minority_vap: Vec<f64> = if demo_path.exists() {
                            let demo = load_demographics(&demo_path).map_err(|e| {
                                format!(
                                    "{}: vra-recom demographics load failed: {e}",
                                    cfg.state_code
                                )
                            })?;
                            align_demographics_to_adjacency(
                                &demo,
                                &graph.index_to_geoid,
                                graph.n_vertices,
                            )
                        } else {
                            if cfg.algo.weights.minority_weighting {
                                eprintln!("WARNING: {}: vra-recom demographics not found at {} — VRA enforcement is a no-op.",
                                      cfg.state_code, demo_path.display());
                            } else {
                                eprintln!("WARNING: --search vra-recom without --weights-override vra-aligned will have 0 protected districts.");
                            }
                            vec![0.0; graph.adjacency.len()]
                        };
                        status(
                            cfg.position,
                            &format!(
                        "{}: vra-recom p={:.2} {} steps vap_threshold={:.2} into {} districts",
                        cfg.state_code, p, steps, vap_threshold, num_districts
                    ),
                        );
                        crate::bisection_runner::run_vra_recom(
                            &graph.adjacency,
                            &vwgt,
                            &edge_weights,
                            num_districts,
                            niter,
                            base,
                            *steps,
                            *p,
                            *vap_threshold,
                            &minority_vap,
                        )
                        .map_err(|e| format!("vra-recom: {e}"))?
                    }
                    SeedCompositor::SmcPercentile { n_particles, p } => {
                        let base = seed.unwrap_or(0);
                        let resample_threshold = cfg.smc_resample_threshold;
                        status(
                            cfg.position,
                            &format!(
                                "{}: smc-percentile p={:.2} {} particles into {} districts",
                                cfg.state_code, p, n_particles, num_districts
                            ),
                        );
                        crate::bisection_runner::run_smc_percentile(
                            &graph.adjacency,
                            &vwgt,
                            num_districts,
                            base,
                            *n_particles,
                            *p,
                            resample_threshold,
                        )
                        .map_err(|e| format!("smc-percentile: {e}"))?
                    }
                    _ => {
                        status(
                            cfg.position,
                            &format!(
                                "{}: recursive bisection into {} districts",
                                cfg.state_code, num_districts
                            ),
                        );
                        run_all_splits(
                            &graph.adjacency,
                            &vwgt,
                            &edge_weights,
                            num_districts,
                            balance_tolerance_frac,
                            niter,
                            seed,
                            Some(&intermediate_dir),
                        )
                        .map_err(|e| format!("bisection failed: {e}"))?
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
        // Every structure is held to the configured final-plan population
        // contract. Structure-specific research tolerances may guide candidate
        // construction, but must not silently relax final validation.
        let effective_tolerance = balance_tolerance;
        let partition = Partition::from_assignments(assignments_attempt.clone());
        let balance_ok =
            partition.assert_balanced(&graph.vertex_weights, num_districts, effective_tolerance);
        match balance_ok {
            Ok(_) => {
                assignments = assignments_attempt;
                break 'retry;
            }
            Err(e) => {
                last_balance_err = format!(
                    "population balance violation (tolerance {:.1}%): {e}",
                    effective_tolerance * 100.0
                );
                // continue retry loop
            }
        }
    } // end 'retry loop

    if assignments.is_empty() {
        return Err(last_balance_err);
    }

    status(
        cfg.position,
        &format!("{}: balance OK, writing outputs", cfg.state_code),
    );

    // 5. VRA analysis (if VRA mode and multi-district)
    let vra = if matches!(&cfg.algo.split, SplitStrategy::NWay) && num_districts > 1 {
        let demo_path = std::path::Path::new("data")
            .join(&cfg.year)
            .join("demographics")
            .join(format!("{state_name}_demographics_{}.csv", cfg.year));
        let demo = load_demographics(&demo_path)
            .map_err(|e| format!("demographics reload for VRA analysis failed: {e}"))?;
        let minority_fracs =
            align_demographics_to_adjacency(&demo, &graph.index_to_geoid, graph.n_vertices);
        let total_pops = graph.vertex_weights.clone();
        let minority_pops: Vec<f64> = minority_fracs
            .iter()
            .zip(total_pops.iter())
            .map(|(f, &t)| f * t as f64)
            .collect();
        let zeros = vec![0.0f64; graph.n_vertices];
        let analysis = analyze_mm_districts(
            &assignments,
            &total_pops,
            &minority_pops,
            &zeros,
            &zeros,
            0.50,
        );
        status(
            cfg.position,
            &format!("{}: {} MM districts", cfg.state_code, analysis.mm_count),
        );
        Some(VraAnalysis {
            mm_count: analysis.mm_count,
            mm_districts: analysis.mm_districts,
            districts: analysis
                .districts
                .iter()
                .map(|d| VraDistrict {
                    district: d.district,
                    pct_minority: d.pct_minority,
                    pct_black: d.pct_black,
                    pct_hispanic: d.pct_hispanic,
                    is_mm: d.is_mm,
                })
                .collect(),
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
    let edge_cut: f64 = edge_weights
        .iter()
        .map(|(&(u, v), &w)| {
            let du = assignments.get(&u).copied().unwrap_or(0);
            let dv = assignments.get(&v).copied().unwrap_or(0);
            if du != dv {
                w
            } else {
                0.0
            }
        })
        .sum();

    // 7. Write manifest.json atomically (manifest.tmp → manifest.json).
    // Board amendment: atomic write (manifest.tmp + rename) prevents partial manifests.
    if cfg.write_manifest || cfg.label.is_some() {
        let ilp_solve_report_count = count_ilp_solve_reports(&intermediate_dir);
        let ilp_audit_summary_path = intermediate_dir
            .join("ilp_solve_reports")
            .join("audit-summary.json");
        let ilp_audit_summary_sha256 = if ilp_audit_summary_path.exists() {
            Some(
                bisect_report::sha256_file(&ilp_audit_summary_path)
                    .map_err(|e| format!("hash ilp audit summary: {e}"))?,
            )
        } else {
            None
        };
        let adj_filename = format!("{}_adjacency_{}.adj.bin", state_name, cfg.year);
        let state_fips = state_code_to_fips(&cfg.state_code)
            .unwrap_or("00")
            .to_string();
        let tiger_url = bisect_report::tiger_source_url(&state_fips, &cfg.year);
        let gpmetis_version = crate::bisection_runner::detect_gpmetis_version();
        let created_at = bisect_report::now_iso8601();
        let binary_sha256 = std::env::current_exe()
            .map_err(|e| format!("resolve current executable for manifest hash: {e}"))
            .and_then(|path| {
                bisect_report::sha256_file(&path)
                    .map_err(|e| format!("hash current executable {}: {e}", path.display()))
            })?;
        let adjacency_sha256 = bisect_report::sha256_file(&adj_pkl)
            .map_err(|e| format!("hash adjacency source {}: {e}", adj_pkl.display()))?;
        let tiger_sha256 = runner_tiger_sha256(cfg)?;
        let audit_sidecars = write_rplan_audit_sidecars(
            &plan_root,
            cfg,
            &label,
            &graph,
            &assignments,
            &adj_filename,
            &adj_pkl,
            &tiger_url,
            balance_tolerance,
            &created_at,
        )?;

        let manifest = bisect_report::PlanManifest {
            label: label.clone(),
            state_code: cfg.state_code.clone(),
            year: cfg.year.clone(),
            chamber: cfg.chamber.clone(),
            num_districts,
            population_source: cfg.population_source.clone(),
            partition_mode: cfg.algo.mode_name().to_string(),
            seed: seed.map(|s| s as i64),
            binary_version: env!("CARGO_PKG_VERSION").to_string(),
            binary_sha256,
            binary_download_url: format!(
                "https://github.com/owner/BISECT/releases/download/v{}/BISECT",
                env!("CARGO_PKG_VERSION")
            ),
            adjacency_file: adj_filename,
            adjacency_sha256,
            adjacency_build_command: format!(
                "python scripts/data/generate_adj_bin.py --year {} --states {}",
                cfg.year, state_name
            ),
            adjacency_build_version: env!("CARGO_PKG_VERSION").to_string(),
            tiger_source_url: tiger_url,
            tiger_sha256,
            created_at,
            balance_tolerance_pct: balance_tolerance * 100.0,
            population_balance_valid: audit_sidecars.population_balance_valid,
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
            niter: cfg.algo.metis.niter,
            alpha_county: cfg.algo.weights.alpha_county,
            directional_lambda: cfg.algo.weights.directional_lambda,
            split_seeds: match &cfg.algo.split {
                SplitStrategy::GeoSection
                | SplitStrategy::AreaSection { .. }
                | SplitStrategy::VraSection { .. }
                | SplitStrategy::CompactBisect { .. }
                | SplitStrategy::ProportionalSection { .. } => Some(cfg.algo.seeds.seed_count()),
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
            spectral_iters: match &cfg.algo.split {
                SplitStrategy::Spectral { max_iters } => Some(*max_iters),
                _ => None,
            },
            ilp_method: match &cfg.algo.split {
                SplitStrategy::Ilp { method, .. } => Some(method.to_string()),
                _ => None,
            },
            ilp_fallback: match &cfg.algo.split {
                SplitStrategy::Ilp { fallback, .. } => Some(fallback.to_string()),
                _ => None,
            },
            ilp_solve_report_dir: match &cfg.algo.split {
                SplitStrategy::Ilp { .. } if ilp_solve_report_count > 0 => {
                    Some("intermediate/ilp_solve_reports".to_string())
                }
                _ => None,
            },
            ilp_solve_report_count: match &cfg.algo.split {
                SplitStrategy::Ilp { .. } if ilp_solve_report_count > 0 => {
                    Some(ilp_solve_report_count)
                }
                _ => None,
            },
            ilp_audit_summary_path: match &cfg.algo.split {
                SplitStrategy::Ilp { .. } if ilp_audit_summary_sha256.is_some() => {
                    Some("intermediate/ilp_solve_reports/audit-summary.json".to_string())
                }
                _ => None,
            },
            ilp_audit_summary_sha256: match &cfg.algo.split {
                SplitStrategy::Ilp { .. } => ilp_audit_summary_sha256,
                _ => None,
            },
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
                SeedCompositor::ShortBurstMergeSplit { .. } => {
                    Some("short-burst-merge-split".to_string())
                }
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
            multiscale_fine: if matches!(
                &cfg.algo.seeds,
                SeedCompositor::MultiScale { .. } | SeedCompositor::MultiScaleAdaptive { .. }
            ) {
                Some(cfg.multiscale_fine.clone())
            } else {
                None
            },
            multiscale_coarse: if matches!(
                &cfg.algo.seeds,
                SeedCompositor::MultiScale { .. } | SeedCompositor::MultiScaleAdaptive { .. }
            ) {
                Some(cfg.multiscale_coarse.clone())
            } else {
                None
            },
            fine_to_coarse_formula: if matches!(
                &cfg.algo.seeds,
                SeedCompositor::MultiScale { .. } | SeedCompositor::MultiScaleAdaptive { .. }
            ) {
                // BG->tract uses prefix[:11]; all county coarsenings use prefix[:5]
                let formula = if cfg.multiscale_coarse == "tract" {
                    "geoid_prefix[:11]"
                } else {
                    "geoid_prefix[:5]"
                };
                Some(formula.to_string())
            } else {
                None
            },
            fine_to_coarse_comment: if matches!(
                &cfg.algo.seeds,
                SeedCompositor::MultiScale { .. } | SeedCompositor::MultiScaleAdaptive { .. }
            ) {
                let comment = match (cfg.multiscale_fine.as_str(), cfg.multiscale_coarse.as_str()) {
                    ("bg", "tract") => "first 11 chars of 12-char BG GEOID = parent tract GEOID",
                    ("bg", "county") => "first 5 chars of 12-char BG GEOID = parent county FIPS",
                    _ => "first 5 chars of 11-char tract GEOID = parent county FIPS",
                };
                Some(comment.to_string())
            } else {
                None
            },
            index_to_geoid_file: if matches!(
                &cfg.algo.seeds,
                SeedCompositor::MultiScale { .. } | SeedCompositor::MultiScaleAdaptive { .. }
            ) {
                // BG-fine options use the BG geoids file; tract-fine uses tract geoids file
                if cfg.multiscale_fine == "bg" {
                    Some(format!(
                        "{}_bg_adjacency_{}_geoids.json",
                        state_name, cfg.year
                    ))
                } else {
                    Some(format!("{}_adjacency_{}_geoids.json", state_name, cfg.year))
                }
            } else {
                None
            },
            rplan_path: Some(audit_sidecars.rplan_path),
            rctx_path: Some(audit_sidecars.rctx_path),
            audit_certificate_path: Some(audit_sidecars.audit_certificate_path),
            audit_certificate_sha256: Some(audit_sidecars.audit_certificate_sha256),
            audit_certificate_content_hash: Some(audit_sidecars.audit_certificate_content_hash),
            audit_result: Some(audit_sidecars.audit_result),
            legal_profile_id: Some(audit_sidecars.legal_profile_id),
            context_hash: Some(audit_sidecars.context_hash),
        };
        bisect_report::write_manifest_atomic(&plan_root, &manifest)
            .map_err(|e| format!("manifest write failed: {e}"))?;
    }

    status(
        cfg.position,
        &format!("{}: complete ({num_districts}D, {}ms)", cfg.state_code, 0),
    );
    Ok(())
}

#[cfg(test)]
mod tests;
