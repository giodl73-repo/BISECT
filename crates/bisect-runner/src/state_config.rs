//! Native State compositor conversion, shared unchanged with CLI and browser execution adapters.
use crate::runner::AreaSectionInit;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VertexConstraintKind {
    /// Census population (always present, usually first).
    Population,
    /// TIGER ALAND land area in hectares. Requires TIGER geometry.
    Area,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SeedCompositor<C = usize> {
    /// Single content-derived seed. Deterministic, fast.
    /// Used by ApportionRegions for the federal statute default.
    Single,
    /// Try N seeds at each evaluation point, keep the minimum-EC result.
    /// Used by GeoSection, AreaSection, VRASection, CompactBisect.
    Multi { seeds: C },
    /// Run seeds sequentially from the content-derived start until
    /// `threshold` consecutive seeds produce no improvement in normalised EC.
    /// Stops after a measured non-improvement tail; this is not an optimality certificate.
    ConvergenceSweep { threshold: u32 },
    /// Run `seeds` plans, sort by edge cut, return the plan at rank floor(p * seeds).
    /// p=0.0 → minimum EC (same as ConvergenceSweep), p=0.5 → median EC, p=1.0 → maximum EC.
    /// Enables statutory choice of legal posture (B.7 / U.8).
    Percentile { p: f64, seeds: C },
    /// At each bisection node, run a local `ensemble_steps`-step 2-way ReCom ensemble
    /// and pick the bisection at percentile `p` of the cut distribution.
    /// Always k=2 at each node — eliminates prime-k bipartition failures. (U.9)
    BisectionEnsemble { p: f64, ensemble_steps: C },
    /// Flip individual boundary tracts to adjacent districts, collect all visited plans,
    /// return the plan at percentile `p` of the edge-cut distribution.
    /// flip_steps: total flip proposals (default: 10000). p: percentile (default: 0.0).
    Flip { flip_steps: C, p: f64 },
    /// Short-Burst: run `n_bursts` short ReCom chains of `burst_length` steps on the
    /// full-state k-way assignment. Keep the chain endpoint from each burst (not the
    /// minimum). Chain restarts from the previous burst's endpoint. Sort endpoints by
    /// EC ASC; return the plan at rank floor(p * n_bursts). (G.6)
    ShortBurst {
        burst_length: C,
        n_bursts: C,
        p: f64,
    },
    /// Short-Burst using ForestRecomChain (two-tree MH) as the burst chain.
    /// Provides compactness optimization with approximate distributional correctness. (G.12)
    ShortBurstForest {
        burst_length: C,
        n_bursts: C,
        p: f64,
    },
    /// Short-Burst using MergeSplitChain as the burst chain. (G.12)
    ShortBurstMergeSplit {
        burst_length: C,
        n_bursts: C,
        p: f64,
    },
    /// Run `steps` Forest ReCom MH steps; collect accepted plans; return at percentile p of EC.
    /// Two-tree Metropolis-Hastings — targets uniform distribution (G.9 spec accepted 3.0/4).
    ForestRecom { steps: C, p: f64 },
    /// Multi-scale MCMC — interleaves fine (tract) and coarse (block-group) ReCom moves.
    /// Requires block-group adjacency (run: bisect fetch --resolution block_group).
    /// CLI: --search multiscale --multiscale-steps 2000 --multiscale-alpha 0.3
    /// (G.11 spec accepted 3.0/4; full implementation pending BISECT-multiscale crate completion)
    MultiScale {
        total_steps: C,
        p: f64,
        alpha: f64,
    },
    /// Run `steps` Merge-Split MH steps; collect accepted plans; return at percentile p.
    /// Two-tree MH with explicit ratio — O(m log m) per step (G.10 spec accepted 3.0/4).
    MergeSplit { steps: C, p: f64 },
    /// Adaptive Multi-scale MCMC — Robbins-Monro self-tuning alpha (U.5 spec accepted 3.75/4).
    /// Requires block-group adjacency (run: bisect fetch --resolution block_group).
    /// CLI: --search multiscale-adaptive --multiscale-steps 2000 --ms-target-accept 0.30 --ms-adapt-interval 50
    MultiScaleAdaptive {
        total_steps: C,
        p: f64,
        target_accept: f64,
        adapt_interval: C,
    },
    /// Parallel Tempering: N replicas at geometric tolerance ladder with replica exchange.
    /// Cold chain (replica 0) provides the plan distribution. (U.4 spec accepted 4.0/4)
    ParallelTempering {
        n_replicas: C,    // number of chains (default: 4)
        swap_interval: C, // steps between swap proposals (default: 10)
        cold_tolerance: f64,  // cold chain balance tolerance (default: 0.005)
        hot_tolerance: f64,   // hot chain balance tolerance (default: 0.05)
        steps: C,         // total steps per cold chain (default: 1000)
        p: f64,               // percentile of cold chain EC distribution (default: 0.0)
    },
    /// VRA-aware MCMC: ForestRecomChain with hard VRA rejection preserving majority-minority districts.
    /// Requires minority VAP data via --weights-override vra-aligned or explicit minority column.
    /// (G.13 spec accepted 3.75/4)
    VraRecom {
        steps: C,       // total chain steps (default: 1000)
        p: f64,             // percentile of EC distribution (default: 0.0)
        vap_threshold: f64, // minority VAP fraction threshold (default: 0.50)
    },
    /// SMC weighted ensemble — runs BISECT-smc and selects plan at p-th weighted EC quantile.
    /// The only compositor mode with a calibrated (importance-weighted) stationary distribution.
    /// (SmcPercentile spec accepted 3.88/4)
    SmcPercentile { n_particles: C, p: f64 },
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SplitStrategy<C = usize> {
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
        steps_per_tract: C,
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
        n_iter: C,
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
        max_tracts: C,              // size guard (default: 500; fallback to METIS if exceeded)
    },
    /// Moving-Knife Algorithm — maximises Reock compactness via sweep (T.13 spec 3.75/4).
    /// Tests n_orientations candidate sweep directions; picks angle with best min(Reock_L, Reock_R).
    MovingKnife {
        n_orientations: C, // sweep granularity (default: 180 = every 1°)
        metric: String,        // "reock" (default) | "polsby"
    },
    /// Capacity-constrained clustering (T.15). Crate-level kernel is available;
    /// runner execution is staged after direct k-way plan integration.
    CapacityClustering,
    /// Spectral graph partitioning baseline (T.14).
    Spectral { max_iters: C },
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetisParams {
    pub ufactor: u32,
    pub niter: u32,
    #[serde(with = "crate::native_replay_recipe::optional_seed")]
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
#[derive(Debug, Clone, serde::Serialize)]
pub struct AlgorithmConfig<C = usize> {
    /// Layer 1: tree structure
    pub split: SplitStrategy<C>,
    /// Layer 2a: edge weight signals
    pub weights: WeightSpec,
    /// Layer 2b: vertex balance constraints (ncon = len)
    pub vertex_constraints: Vec<VertexConstraintKind>,
    /// Layer 3: seed search strategy
    pub seeds: SeedCompositor<C>,
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
    pub fn from_state_args(args: &crate::state_args::StateArgs) -> Self {
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
        use crate::args::{SearchMode as SeM, StructureMode as SM};
        use crate::state_args::WeightMode as WM;

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
