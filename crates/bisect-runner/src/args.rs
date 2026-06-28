//! Three-layer compositor enums and ILP-audit CLI args, extracted from
//! `bisect-cli::args` so the bisection engine no longer depends on the CLI crate.
//!
//! These types derive clap traits (`ValueEnum`/`Parser`) because they are the
//! parsed representation of the compositor flags. The `bisect-cli` `args` module
//! re-exports them so existing `crate::args::*` references continue to resolve.

use clap::{Parser, ValueEnum};

/// Layer-collapsed partition mode (legacy `--partition-mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum PartitionMode {
    #[value(name = "unweighted")]
    Unweighted,
    #[value(name = "edge-weighted")]
    EdgeWeighted,
    #[value(name = "metis-vra")]
    MetisVra,
    /// Partisan-weighted bisection: requires --partisan-shares.
    /// Mutually exclusive with `metis-vra` per Callais p.36 disentanglement.
    #[value(name = "partisan-weighted")]
    PartisanWeighted,
    /// Proportional bisection (B.7): at each recursion level, split the subgraph
    /// so that both halves contain vote totals proportional to the subregion's
    /// Dem/Rep vote shares, then minimise edge-cut within that constraint.
    /// Requires --partisan-shares. Mutually exclusive with metis-vra.
    /// Stat 104(e) prohibits this for federal congressional districts;
    /// valid for state legislative redistricting.
    #[value(name = "proportional")]
    Proportional,
    /// CompactBisect (B.7): greedy level-by-level selection by geometric-mean
    /// Polsby-Popper. Requires --compact-seeds N.
    #[value(name = "compact-bisect")]
    CompactBisect,
    /// GeoSection (T.1): ratio-optimal first-level bisection.
    /// Tries all split ratios (1:k-1 through k/2:k/2) at the first level,
    /// each with --geosection-seeds seeds. Selects the ratio with the minimum
    /// edge-cut. Subsequent levels use standard bisection.
    #[value(name = "geosection")]
    GeoSection,
    /// AreaSection (T.2): dual population+area constraint bisection.
    /// ncon=2: tight population balance AND 50/50 area (±10% swing allowed).
    #[value(name = "areasection")]
    AreaSection,
    /// ProportionalSection (T.5): ncon=2 [pop, D_votes] bisection for partisan proportionality.
    /// Uses HH seat allocation to set tpwgts: right (R-bloc) gets minimum D votes for 50% D.
    /// Requires presidential_by_tract.csv in data/{year}/elections/.
    /// Use --eta to control D_votes constraint softness (1.05=tight, 1.20=loose).
    #[value(name = "proportional-section")]
    ProportionalSection,
    /// ApportionRegions (T.4): hierarchical k-way partition driven by prime
    /// factorization of the seat count. Geographic completion of Huntington-Hill.
    /// Requires --compact-seeds N (seeds per level, default 1).
    #[value(name = "apportion-regions")]
    ApportionRegions,
    /// VRASection (T.7): GeoSection + geographic alignment score.
    /// At the first bisection level, ratio selection is modified by how well
    /// minority VAP concentrates on one side (Gingles Prong 1 alignment).
    /// Uses only spatial minority-VAP distribution — no partisan data.
    /// Requires demographics CSV in data/{year}/demographics/.
    /// Use --w-vra to control alignment weight (default 0.40).
    #[value(name = "vra-section")]
    VraSection,
    /// SimulatedAnnealing: METIS initial partition + SA boundary-flip refinement.
    /// At each bisection node, starts from METIS and accepts/rejects random
    /// boundary flips via the Boltzmann criterion (geometric cooling schedule).
    /// Tracks best-ever EC plan and returns it.
    /// Use --sa-steps-per-tract, --sa-t0-factor, --sa-t-final to tune.
    #[value(name = "simulated-annealing")]
    SimulatedAnnealing,
    /// BFS Region-Growing — greedy geographic district packing. No hyperparameters.
    /// Seeds placed by maximum BFS spread; tracts assigned to most population-deficient district.
    /// Inherits balance_tolerance from standard flags. (T.12)
    #[value(name = "bfs-growth")]
    BfsGrowth,
    /// Centroidal Voronoi Districts — geometric packing via graph-distance Voronoi (T.10).
    /// Seeds placed by k-farthest spread, iteratively moved to medoid of each Voronoi region.
    /// Use --cvd-iters to control max iterations (default: 20). No METIS call.
    #[value(name = "centroidal-voronoi")]
    CentroidalVoronoi,
    /// ILP exact redistricting — provably optimal for small instances (n <= 500).
    #[value(name = "ilp")]
    Ilp,
    /// Moving-Knife Algorithm — maximises Reock compactness via orientation sweep (T.13).
    /// Tests n_orientations candidate sweep directions; picks angle that maximises
    /// min(Reock_left, Reock_right). Requires centroid data (bisect fetch --type centroids).
    /// Use --mka-orientations N (default 180) and --mka-metric [reock|polsby] (default reock).
    #[value(name = "moving-knife")]
    MovingKnife,
    /// Capacity-constrained clustering (T.15). Crate-level kernel is available;
    /// full runner execution is staged behind repair/RPLAN integration.
    #[value(name = "capacity-clustering")]
    CapacityClustering,
    /// Spectral graph partitioning baseline (T.14).
    #[value(name = "spectral")]
    Spectral,
    /// Hierarchical regionalization baseline (T.16).
    #[value(name = "regionalization")]
    Regionalization,
    /// Flow-style constructive assignment baseline (T.17).
    #[value(name = "flow-construction")]
    FlowConstruction,
}

impl std::fmt::Display for PartitionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unweighted => write!(f, "unweighted"),
            Self::EdgeWeighted => write!(f, "edge-weighted"),
            Self::MetisVra => write!(f, "metis-vra"),
            Self::PartisanWeighted => write!(f, "partisan-weighted"),
            Self::Proportional => write!(f, "proportional"),
            Self::CompactBisect => write!(f, "compact-bisect"),
            Self::GeoSection => write!(f, "geosection"),
            Self::AreaSection => write!(f, "areasection"),
            Self::ProportionalSection => write!(f, "proportional-section"),
            Self::ApportionRegions => write!(f, "apportion-regions"),
            Self::VraSection => write!(f, "vra-section"),
            Self::SimulatedAnnealing => write!(f, "simulated-annealing"),
            Self::BfsGrowth => write!(f, "bfs-growth"),
            Self::CentroidalVoronoi => write!(f, "centroidal-voronoi"),
            Self::Ilp => write!(f, "ilp"),
            Self::MovingKnife => write!(f, "moving-knife"),
            Self::CapacityClustering => write!(f, "capacity-clustering"),
            Self::Spectral => write!(f, "spectral"),
            Self::Regionalization => write!(f, "regionalization"),
            Self::FlowConstruction => write!(f, "flow-construction"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum IlpMethod {
    /// Build and validate the ILP formulation, then fall back to METIS.
    #[value(name = "formulation-only")]
    FormulationOnly,
    /// Branch-and-cut with lazy connectivity callbacks.
    #[value(name = "branch-and-cut")]
    BranchAndCut,
    /// Branch-and-cut simulated by iterative separation rounds.
    #[value(name = "iterative-separation")]
    IterativeSeparation,
}

impl std::fmt::Display for IlpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FormulationOnly => write!(f, "formulation-only"),
            Self::BranchAndCut => write!(f, "branch-and-cut"),
            Self::IterativeSeparation => write!(f, "iterative-separation"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum IlpFallback {
    /// Fall back to METIS when ILP cannot produce a plan.
    #[value(name = "metis")]
    Metis,
    /// Return an error instead of falling back when ILP cannot produce a plan.
    #[value(name = "error")]
    Error,
}

impl std::fmt::Display for IlpFallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Metis => write!(f, "metis"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// CLI argument for AreaSection warm-start strategy.
/// Converts to `crate::runner::AreaSectionInit` via `Into`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AreaSectionInitArg {
    /// Use the existing Lorenz-filtered ratio heuristic (no directional pre-bias).
    #[value(name = "ratio-optimal")]
    RatioOptimal,
    /// Call MKA first to get theta* (Reock-optimal angle), then use theta* as a
    /// directional edge-weight bias before the METIS ratio search.
    #[value(name = "moving-knife")]
    MovingKnife,
}

impl From<AreaSectionInitArg> for crate::runner::AreaSectionInit {
    fn from(a: AreaSectionInitArg) -> Self {
        match a {
            AreaSectionInitArg::RatioOptimal => crate::runner::AreaSectionInit::RatioOptimal,
            AreaSectionInitArg::MovingKnife => crate::runner::AreaSectionInit::MovingKnife,
        }
    }
}

/// Layer 1 compositor: which tree structure to use for bisection.
/// Overrides the structure implied by --partition-mode when set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StructureMode {
    /// Always ⌊k/2⌋:⌈k/2⌉ bisection (default for most modes)
    #[value(name = "standard-bisect")]
    StandardBisect,
    /// N-way direct partition (METIS kway)
    #[value(name = "nway")]
    NWay,
    /// Ratio-optimal scan: try all 1:k-1..k/2:k/2 ratios, pick min normalised EC (GeoSection)
    #[value(name = "ratio-optimal")]
    RatioOptimal,
    /// Ratio-optimal + area balance constraint (AreaSection)
    #[value(name = "ratio-optimal-area")]
    RatioOptimalArea,
    /// Ratio-optimal + VRA minority alignment score (VRASection)
    #[value(name = "ratio-optimal-vra")]
    RatioOptimalVra,
    /// Prime-factorisation tree — ApportionRegions/Huntington-Hill extension (T.4)
    #[value(name = "prime-factor")]
    PrimeFactor,
    /// Compact-by-Polsby-Popper greedy level selection (CompactBisect B.7)
    #[value(name = "compact-polsby")]
    CompactPolsby,
    /// BFS Region-Growing — greedy geographic packing from k-farthest seeds (T.12)
    #[value(name = "bfs-growth")]
    BfsGrowth,
    /// Centroidal Voronoi Districts — graph-distance Voronoi iteration (T.10)
    /// --structure centroidal-voronoi --cvd-iters 20
    #[value(name = "centroidal-voronoi")]
    CentroidalVoronoi,
    /// Moving-Knife Algorithm — Reock-maximising orientation sweep (T.13).
    /// --structure moving-knife --mka-orientations 180 --mka-metric reock
    #[value(name = "moving-knife")]
    MovingKnife,
    /// Capacity-constrained clustering (T.15).
    /// --structure capacity-clustering
    #[value(name = "capacity-clustering")]
    CapacityClustering,
    /// Spectral graph partitioning baseline (T.14).
    /// --structure spectral --spectral-iters 200
    #[value(name = "spectral")]
    Spectral,
    /// Hierarchical regionalization baseline (T.16).
    /// --structure regionalization
    #[value(name = "regionalization")]
    Regionalization,
    /// Flow-style constructive assignment baseline (T.17).
    /// --structure flow-construction
    #[value(name = "flow-construction")]
    FlowConstruction,
}

/// Layer 3 compositor: how to search the seed space.
/// Overrides the search strategy implied by --partition-mode when set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SearchMode {
    /// One content-derived seed (SHA-256). Deterministic. Use with ApportionRegions.
    #[value(name = "single")]
    Single,
    /// Try --seeds N seeds, keep the minimum-EC result (default for most modes)
    #[value(name = "multi")]
    Multi,
    /// Run until --convergence-threshold T consecutive non-improving seeds.
    /// Certifies convergence per B.7. The seed-buster for the federal statute.
    #[value(name = "convergence")]
    Convergence,
    /// Run --seeds T plans, sort by edge cut, return plan at rank floor(--percentile * T).
    /// p=0.0 → minimum (like convergence), p=0.5 → median, p=1.0 → maximum.
    /// Enables statutory choice of legal posture (U.8).
    #[value(name = "percentile")]
    Percentile,
    /// At each bisection node run a local --ensemble-steps-step 2-way ReCom ensemble,
    /// pick the bisection at --percentile of the cut distribution.
    /// Compatible with all structure modes; eliminates prime-k bipartition failures (U.9).
    #[value(name = "bisection-ensemble")]
    BisectionEnsemble,
    /// Short-Burst: run --n-bursts short ReCom chains of --burst-length steps.
    /// Keep the chain endpoint from each burst; chain restarts from the previous endpoint.
    /// Return plan at --percentile of endpoints sorted by EC.
    /// burst_length=20, n_bursts=50 are good defaults for full-state k.
    #[value(name = "short-burst")]
    ShortBurst,
    /// Short-Burst with Forest ReCom chain (approximate uniform target).
    /// Uses two RNG streams per step for reversibility. Reuses --burst-length and --n-bursts. (G.12)
    #[value(name = "short-burst-forest")]
    ShortBurstForest,
    /// Short-Burst with Merge-Split chain. Two-tree MH per burst step.
    /// Reuses --burst-length and --n-bursts. (G.12)
    #[value(name = "short-burst-merge-split")]
    ShortBurstMergeSplit,
    /// Flip boundary tracts; return plan at --percentile of all visited EC distribution.
    /// Use --flip-steps for total proposals (default 10000).
    #[value(name = "flip")]
    Flip,
    /// Forest ReCom MH chain — reversible, targets uniform distribution.
    /// Use --forest-steps N for total steps (default 1000).
    #[value(name = "forest-recom")]
    ForestRecom,
    /// Multi-scale MCMC -- requires block-group adjacency file.
    /// Use --multiscale-steps N --multiscale-alpha A (defaults: 2000, 0.3).
    #[value(name = "multiscale")]
    MultiScale,
    /// Merge-Split MH chain — reversible, two-tree acceptance ratio.
    /// Use --merge-split-steps N (default 1000).
    #[value(name = "merge-split")]
    MergeSplit,
    /// Adaptive Multi-scale MCMC — Robbins-Monro self-tuning coarse-move probability.
    /// Requires block-group adjacency. Use --multiscale-steps N, --ms-target-accept F,
    /// --ms-adapt-interval N (defaults: 2000, 0.30, 50). (U.5 spec accepted 3.75/4)
    #[value(name = "multiscale-adaptive")]
    MultiScaleAdaptive,
    /// Parallel Tempering: N replicas at geometric tolerance ladder. --pt-replicas, --pt-swap-interval.
    #[value(name = "parallel-tempering")]
    ParallelTempering,
    /// VRA-aware Forest ReCom — hard rejection preserving majority-minority districts.
    /// Requires minority VAP data (--weights-override vra-aligned).
    #[value(name = "vra-recom")]
    VraRecom,
    /// SMC weighted ensemble — selects plan at p-th weighted EC quantile.
    /// Requires --particles N (default 5000). Only calibrated compositor mode.
    #[value(name = "smc-percentile")]
    SmcPercentile,
}

/// `bisect ilp-audit` — verify ilp-solve-report-v1 artifacts and emit a summary.
#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct IlpAuditArgs {
    /// One or more ilp-solve-report-v1 JSON files to verify
    #[arg(value_name = "REPORT")]
    pub reports: Vec<std::path::PathBuf>,
    /// Recursively verify JSON reports under this directory
    #[arg(long, value_name = "DIR")]
    pub dir: Option<std::path::PathBuf>,
    /// Emit a machine-readable JSON summary
    #[arg(long)]
    pub json: bool,
    /// Write the machine-readable audit summary to this path
    #[arg(long, value_name = "PATH")]
    pub out: Option<std::path::PathBuf>,
    /// Verify an existing audit summary matches the current per-node reports
    #[arg(long, value_name = "PATH")]
    pub verify_summary: Option<std::path::PathBuf>,
}
