//! Actual native bulk States argument surface, shared without host I/O.
use clap::{Parser,ValueEnum};
use crate::args::{PartitionMode,SearchMode,AreaSectionInitArg,IlpMethod,IlpFallback};
use crate::state_args::RunProfile;
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Year {
    #[value(name = "2020")]
    Y2020,
    #[value(name = "2010")]
    Y2010,
    #[value(name = "2000")]
    Y2000,
}

impl std::fmt::Display for Year {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Y2020 => write!(f, "2020"),
            Self::Y2010 => write!(f, "2010"),
            Self::Y2000 => write!(f, "2000"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ElectionYear {
    #[value(name = "2020")]
    E2020,
    #[value(name = "2016")]
    E2016,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ProcessingMode {
    #[value(name = "streaming")]
    Streaming,
    #[value(name = "batch")]
    Batch,
}

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct StatesArgs {
    /// Census year (required)
    #[arg(short = 'y', long)]
    pub year: Year,

    /// Version identifier (required)
    #[arg(short = 'v', long)]
    pub version: String,

    /// Output directory (required)
    #[arg(long)]
    pub output_dir: String,

    /// Number of parallel workers (default: 4)
    #[arg(short = 'w', long, default_value_t = 4)]
    pub workers: usize,

    /// DPI for output maps: 72, 100, 150, 200, or 300 (default: 150)
    #[arg(long, default_value = "150", value_parser = clap::builder::PossibleValuesParser::new(["72", "100", "150", "200", "300"]))]
    pub dpi: String,

    /// Partitioning mode (default: edge-weighted)
    #[arg(short = 'm', long = "partition-mode", default_value = "edge-weighted")]
    pub partition_mode: PartitionMode,

    /// Defensibility profile (open|court). court/bulletproof rejects partisan and
    /// experimental configs for every state. See `validate_court_profile`.
    #[arg(long, default_value = "open")]
    pub profile: RunProfile,

    /// Specific states to process (default: all)
    #[arg(long = "states", num_args = 0.., value_delimiter = ' ')]
    pub states: Vec<String>,

    /// Reprocess all states (ignore completion markers)
    #[arg(long)]
    pub reprocess: bool,

    /// Print commands without executing
    #[arg(short = 'p', long)]
    pub print_only: bool,

    /// Enable debug mode
    #[arg(short = 'd', long)]
    pub debug: bool,

    /// Run per-state analysis
    #[arg(long)]
    pub run_analysis: bool,

    /// Skip political analysis
    #[arg(long)]
    pub skip_political: bool,

    /// Skip demographic analysis
    #[arg(long)]
    pub skip_demographic: bool,

    /// Election year for political analysis (default: 2020)
    #[arg(short = 'e', long = "election-year", default_value = "2020")]
    pub election_year: ElectionYear,

    /// Processing mode: streaming (default) or batch
    #[arg(long, default_value = "streaming")]
    pub processing_mode: ProcessingMode,

    /// Seeds per ratio for GeoSection/AreaSection (default: 50)
    #[arg(long, default_value_t = 50)]
    pub geosection_seeds: usize,

    /// AreaSection area imbalance tolerance (default: 1.10 = ±10%)
    #[arg(long, default_value_t = 1.10)]
    pub area_swing: f64,

    /// AreaSection warm-start strategy (default: ratio-optimal).
    /// moving-knife: use MKA theta* as directional edge bias before METIS ratio search.
    #[arg(long, value_enum, default_value = "ratio-optimal")]
    pub area_section_init: AreaSectionInitArg,

    /// Max deviation per district in percent (default: 0.5 congressional, 5.0 state)
    #[arg(long)]
    pub balance_tolerance: Option<f64>,

    /// County stickiness alpha for subdivision-respecting redistricting (T.3).
    /// Default 0 = disabled. 2.0 = statutory default for the official proposal.
    #[arg(long, default_value_t = 0.0)]
    pub alpha_county: f64,

    /// COMPOSITOR Layer 3 — override seed search strategy for all states.
    #[arg(long)]
    pub search: Option<SearchMode>,

    /// Seeds for --search multi or threshold for --search convergence.
    #[arg(long)]
    pub seeds: Option<usize>,

    /// Convergence threshold for --search convergence (default: 600).
    #[arg(long, default_value_t = 600)]
    pub convergence_threshold: u32,

    /// Target percentile for --search percentile or bisection-ensemble (0.0–1.0). Default: 0.5.
    #[arg(long, default_value_t = 0.5)]
    pub percentile: f64,

    /// Local ReCom steps per bisection node for --search bisection-ensemble. Default: 200.
    #[arg(long, default_value_t = 200)]
    pub ensemble_steps: usize,

    /// ReCom steps per burst for --search short-burst. Default: 20.
    #[arg(long, default_value_t = 20)]
    pub burst_length: usize,

    /// Number of bursts for --search short-burst. Default: 50.
    #[arg(long, default_value_t = 50)]
    pub n_bursts: usize,

    /// Total flip proposals for --search flip. Default: 10000.
    #[arg(long, default_value_t = 10000)]
    pub flip_steps: usize,

    /// Total Forest ReCom MH steps for --search forest-recom. Default: 1000.
    #[arg(long, default_value_t = 1000)]
    pub forest_steps: usize,

    /// Total steps for --search multiscale. Default: 2000.
    #[arg(long, default_value_t = 2000)]
    pub multiscale_steps: usize,

    /// P(coarse move) for --search multiscale. Default: 0.3.
    #[arg(long, default_value_t = 0.3)]
    pub multiscale_alpha: f64,

    /// Target coarse acceptance rate for --search multiscale-adaptive (Robbins-Monro). Default: 0.30.
    #[arg(long, default_value_t = 0.30)]
    pub ms_target_accept: f64,

    /// Adaptation interval (steps between alpha updates) for --search multiscale-adaptive. Default: 50.
    #[arg(long, default_value_t = 50usize)]
    pub ms_adapt_interval: usize,

    /// Fine resolution level for --search multiscale (default: tract).
    /// "tract" = Option B (tract->county); "bg" = Option A (BG->tract) or C (BG->county).
    #[arg(long, default_value = "tract")]
    pub multiscale_fine: String,

    /// Coarse resolution level for --search multiscale (default: county).
    /// "county" with fine=tract = Option B; "tract" with fine=bg = Option A; "county" with fine=bg = Option C.
    #[arg(long, default_value = "county")]
    pub multiscale_coarse: String,

    /// Total Merge-Split MH steps for --search merge-split. Default: 1000.
    #[arg(long, default_value_t = 1000)]
    pub merge_split_steps: usize,

    // ── Parallel Tempering (PT) parameters ───────────────────────────────────
    /// Number of replicas for --search parallel-tempering. Default: 4.
    #[arg(long, default_value_t = 4)]
    pub pt_replicas: usize,

    /// Swap interval for --search parallel-tempering. Default: 10.
    #[arg(long, default_value_t = 10)]
    pub pt_swap_interval: usize,

    /// Cold chain balance tolerance for --search parallel-tempering. Default: 0.005.
    #[arg(long, default_value_t = 0.005)]
    pub pt_cold_tol: f64,

    /// Hot chain balance tolerance for --search parallel-tempering. Default: 0.05.
    #[arg(long, default_value_t = 0.05)]
    pub pt_hot_tol: f64,

    // ── VRA-ReCom parameters ─────────────────────────────────────────────────
    /// Minority VAP fraction threshold for --search vra-recom. Default: 0.50.
    #[arg(long, default_value_t = 0.50)]
    pub vra_threshold: f64,

    // ── Centroidal Voronoi Districts (CVD) parameters ─────────────────────────
    /// CVD: maximum Voronoi iterations per bisection node. Default: 20.
    /// Requires --partition-mode centroidal-voronoi or --structure centroidal-voronoi.
    #[arg(long, default_value_t = 20)]
    pub cvd_iters: usize,

    /// Distance metric for --structure centroidal-voronoi (default: graph-distance).
    /// graph-distance: BFS hop-count (Phase 1, no extra data required).
    /// geographic: Euclidean on Albers-projected coordinates (Phase 2, requires centroids).
    #[arg(long, default_value = "graph-distance")]
    pub cvd_metric: String,

    // ── ILP parameters ────────────────────────────────────────────────────────
    /// ILP method (default: formulation-only).
    #[arg(long, value_enum, default_value_t = IlpMethod::FormulationOnly)]
    pub ilp_method: IlpMethod,

    /// ILP fallback when no solver plan is available (default: metis).
    #[arg(long, value_enum, default_value_t = IlpFallback::Metis)]
    pub ilp_fallback: IlpFallback,

    /// ILP time limit in seconds (default: 300).
    #[arg(long, default_value_t = 300u64)]
    pub ilp_time_limit: u64,

    /// ILP optimality gap (default: 0.01 = 1% from optimal).
    #[arg(long, default_value_t = 0.01f64)]
    pub ilp_gap: f64,

    /// Max tracts before ILP falls back to METIS (default: 500).
    #[arg(long, default_value_t = 500usize)]
    pub ilp_max_tracts: usize,

    // ── Moving-Knife Algorithm (MKA) parameters ───────────────────────────────
    /// MKA: number of sweep orientations (default: 180 = every 1°).
    #[arg(long, default_value_t = 180usize)]
    pub mka_orientations: usize,

    /// MKA: compactness metric (default: reock). polsby falls back to reock in Phase 1.
    #[arg(long, default_value = "reock")]
    pub mka_metric: String,

    // ── SMC-Percentile parameters ─────────────────────────────────────────────
    /// Number of SMC particles for --search smc-percentile (default: 5000).
    #[arg(long)]
    pub particles: Option<usize>,

    /// SMC resample threshold for --search smc-percentile (default: 0.5).
    /// Lower (0.3) for irregular geometries or VRA-heavy states.
    #[arg(long, default_value_t = 0.5)]
    pub smc_resample_threshold: f64,
}

#[derive(Debug,Parser)]
#[command(name="bisect states")]
pub struct StatesCommand {#[command(flatten)] pub args:StatesArgs}
