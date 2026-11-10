//! Shared native State command definitions; parsing does not read files or execute a Run.
use clap::{Parser,ValueEnum};
use crate::args::{PartitionMode,SearchMode,StructureMode,AreaSectionInitArg,IlpMethod,IlpFallback};

/// Defensibility profile for a redistricting run.
///
/// `Open` is the research default: every compositor variant, partisan mode, and
/// experimental algorithm is allowed. `Court` (bulletproof) locks the run to the
/// neutral, deterministic, court-defensible subset and refuses any partisan or
/// experimental configuration. See [`validate_court_profile`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RunProfile {
    /// Research mode: all variants allowed (default).
    #[value(name = "open")]
    Open,
    /// Court-defensible mode: neutral, deterministic, partisan-blind only.
    #[value(name = "court", alias = "bulletproof")]
    Court,
}

/// Reject any configuration that is not court-defensible.
///
/// Court mode permits only neutral structures, partisan-blind weights, and
/// deterministic seed selection. Partisan inputs, partisan/proportional modes,
/// and unbounded experimental baselines are refused so a submitted plan cannot
/// be attacked as partisan-tuned. Returns Ok(()) for `Open`.
pub fn validate_court_profile(
    profile: RunProfile,
    partition_mode: PartitionMode,
    structure: Option<StructureMode>,
    weights: Option<WeightMode>,
    search: Option<SearchMode>,
    partisan_shares: &Option<String>,
) -> Result<(), String> {
    let _=structure; // Native filter intentionally does not inspect structure or seed.
    let search=search.map(|s|match s {SearchMode::Single=>"single",SearchMode::Convergence=>"convergence",_=>"non-deterministic search"});
    crate::run_profile::validate(profile==RunProfile::Court,&partition_mode.to_string(),matches!(weights,Some(WeightMode::Proportional)),search,partisan_shares.is_some())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Resolution {
    #[value(name = "tract")]
    Tract,
    #[value(name = "block_group")]
    BlockGroup,
    #[value(name = "block")]
    Block,
}

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tract => write!(f, "tract"),
            Self::BlockGroup => write!(f, "block_group"),
            Self::Block => write!(f, "block"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MetisObjective {
    #[value(name = "cut")]
    Cut,
    #[value(name = "vol")]
    Vol,
}

/// Layer 2 compositor: which edge/vertex weight signal to use.
/// Overrides the weight implied by --partition-mode when set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum WeightMode {
    /// No geographic weights — minimise cut edge count
    #[value(name = "unweighted")]
    Unweighted,
    /// TIGER boundary lengths (default for most modes)
    #[value(name = "geographic")]
    Geographic,
    /// Geographic + county-stickiness (requires --alpha-county > 0)
    #[value(name = "county")]
    County,
    /// Geographic + minority-VAP alignment score (VRASection, requires minority data)
    #[value(name = "vra-aligned")]
    VraAligned,
    /// Geographic + D_votes constraint (ProportionalSection, requires partisan-shares)
    #[value(name = "proportional")]
    Proportional,
    /// Economic character similarity weights (M.9/M.1). Requires LODES WAC data.
    /// Download: bisect fetch --type lodes --year 2020 --states <STATE>
    #[value(name = "economic-character")]
    EconomicCharacter,
    /// Housing character similarity weights (M.3). Requires ACS housing data.
    /// Download: bisect fetch --type acs-housing --year 2020 --states <STATE>
    #[value(name = "housing-character")]
    HousingCharacter,
    /// Administrative zone co-membership weights (M.6). Requires TIGER school districts
    /// and EIA Form 861. Download: bisect fetch --type school-districts --type eia-861
    #[value(name = "zone-membership")]
    ZoneMembership,
}

/// Which METIS backend to use for graph partitioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MetisEngineArg {
    /// C METIS via the `metis` Rust FFI crate (links libmetis). Default.
    /// Battle-tested; handles all k values including prime k without stalling.
    #[value(name = "c-ffi")]
    CFfi,
    /// Pure-Rust multilevel graph partitioning (`metis-core`). No C dependency.
    /// Produces a fully portable standalone binary. May stall on prime k
    /// for large graphs (>1000 tracts with prime k).
    #[value(name = "metis-core")]
    RedistMetis,
    /// External `gpmetis` binary subprocess. Requires gpmetis on PATH.
    /// Not yet implemented — returns an error if selected.
    #[value(name = "gpmetis")]
    Gpmetis,
}

impl From<MetisEngineArg> for bisect_apportion::split::MetisEngine {
    fn from(arg: MetisEngineArg) -> Self {
        match arg {
            MetisEngineArg::CFfi => Self::CFfi,
            MetisEngineArg::RedistMetis => Self::RedistMetis,
            MetisEngineArg::Gpmetis => Self::Gpmetis,
        }
    }
}

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct StateArgs {
    /// Two-letter state code (e.g., CA, TX, VT)
    #[arg(long)]
    pub state: String,

    /// Census year. Accepts any 4-digit year (2020, 2010, 2000 for US; 2022 for
    /// Ireland, 2021 for Malta, etc.). Validated against the location registry.
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,

    /// Geographic resolution (default: tract)
    #[arg(long, default_value = "tract")]
    pub resolution: Resolution,

    /// Version identifier (default: v1)
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,

    /// Output directory (default: auto-generated)
    #[arg(long)]
    pub output_dir: Option<String>,

    /// DPI for output maps: 72, 100, 150, 200, or 300 (default: 150)
    #[arg(long, default_value = "150", value_parser = clap::builder::PossibleValuesParser::new(["72", "100", "150", "200", "300"]))]
    pub dpi: String,

    /// Progress bar position for parallel mode (default: 1, matches Python process_single_state.py)
    #[arg(long, default_value_t = 1)]
    pub position: i32,

    /// Print what would be done without executing
    #[arg(short = 'p', long)]
    pub print_only: bool,

    /// Enable debug mode
    #[arg(short = 'd', long)]
    pub debug: bool,

    /// Delete output directory before starting
    #[arg(short = 'r', long)]
    pub reset: bool,

    /// Partitioning mode (default: edge-weighted)
    #[arg(short = 'm', long = "partition-mode", default_value = "edge-weighted")]
    pub partition_mode: PartitionMode,

    /// Defensibility profile. `open` (default) allows every experimental and
    /// partisan variant; `court` (a.k.a. bulletproof) restricts the run to the
    /// neutral, deterministic, court-defensible compositor and rejects any
    /// partisan or experimental configuration. See `validate_court_profile`.
    #[arg(long, default_value = "open")]
    pub profile: RunProfile,

    /// Target majority-minority districts (VRA mode only)
    #[arg(long)]
    pub target_mm_districts: Option<usize>,

    /// Per-tract Democratic vote share TSV file (partisan-weighted mode only).
    /// Format: header row `geoid<TAB>dem_share`, geoid is 11-char TIGER FIPS,
    /// dem_share is float in [0.0, 1.0]. Required when --partition-mode=partisan-weighted.
    /// MUTUALLY EXCLUSIVE with --partition-mode=metis-vra (Callais p.36 disentanglement).
    #[arg(long)]
    pub partisan_shares: Option<String>,

    /// Threshold above which a unit is considered "strong-D" (default 0.55).
    /// Partisan-weighted mode only.
    #[arg(long, default_value_t = 0.55)]
    pub dem_threshold: f64,

    /// Threshold below which a unit is considered "strong-R" (default 0.45).
    /// Partisan-weighted mode only.
    #[arg(long, default_value_t = 0.45)]
    pub rep_threshold: f64,

    /// METIS imbalance tolerance factor: 5 = ±0.5% (default: 5)
    #[arg(long, default_value_t = 5)]
    pub ufactor: u32,

    /// METIS refinement iterations (default: 100)
    #[arg(long, default_value_t = 100)]
    pub niter: u32,

    /// METIS objective: cut (edge-cut) or vol (volume) (default: cut)
    #[arg(long, default_value = "cut")]
    pub objtype: MetisObjective,

    /// METIS random seed for reproducibility (default: random)
    #[arg(long)]
    pub seed: Option<u64>,

    /// CompactBisect: seeds per bisection level (default: 0 = disabled).
    /// Requires --partition-mode compact-bisect. At each level, runs this many
    /// METIS seeds and selects by geometric-mean Polsby-Popper (or min edge-cut
    /// if TIGER geometry is unavailable). Typical: 20-100.
    #[arg(long, default_value = "0")]
    pub compact_seeds: usize,

    /// GeoSection (T.1): seeds per ratio at the first bisection level.
    /// Requires --partition-mode geosection. Tries all ratios 1:k-1 through
    /// k/2:k/2 with this many seeds each; picks minimum-EC ratio. Typical: 50-200.
    #[arg(long, default_value = "0")]
    pub geosection_seeds: usize,

    /// AreaSection (T.2): area imbalance tolerance multiplier (default: 1.10 = ±10%).
    /// ubvec[1] for the land-area balance constraint. Values <1.05 may cause
    /// convergence failures for concentrated states; >1.25 approaches GeoSection.
    #[arg(long, default_value_t = 1.10)]
    pub area_swing: f64,

    /// AreaSection warm-start strategy (default: ratio-optimal).
    /// ratio-optimal: existing Lorenz-filtered ratio heuristic (no directional pre-bias).
    /// moving-knife: call MKA first to get theta* (Reock-optimal angle), then use
    /// theta* as a directional edge-weight bias before the METIS ratio search.
    /// Requires --partition-mode areasection or --structure ratio-optimal-area.
    /// Falls back to ratio-optimal with a warning if centroid data is absent.
    #[arg(long, value_enum, default_value = "ratio-optimal")]
    pub area_section_init: AreaSectionInitArg,

    /// ProportionalSection (T.5): D_votes constraint softness (ubvec[1]).
    /// 1.05 = tight partisan constraint; 1.20 = loose. Default 1.10.
    #[arg(long, default_value_t = 1.10)]
    pub eta: f64,

    /// Governmental subdivision stickiness — county level (T.3).
    /// Makes intra-county edges more expensive to cut (alpha=0 disables).
    /// alpha=1 doubles the cost of cross-county cuts; alpha=5 = strong preference.
    #[arg(long, default_value_t = 0.0)]
    pub alpha_county: f64,

    // ── Compositor overrides (Layer 1/2/3 explicit flags) ─────────────────────
    // These override the corresponding layer from --partition-mode when set.
    // Allows mixing presets with individual layer customization.
    /// COMPOSITOR Layer 1 — override the split structure independent of --partition-mode.
    /// Possible values: standard-bisect, nway, ratio-optimal, ratio-optimal-area,
    /// ratio-optimal-vra, prime-factor, compact-polsby.
    /// Example: --partition-mode geosection --structure prime-factor
    ///   runs ApportionRegions tree with GeoSection's other defaults.
    #[arg(long)]
    pub structure: Option<StructureMode>,

    /// COMPOSITOR Layer 2 — override the edge/vertex weight signal.
    /// Possible values: unweighted, geographic, county, vra-aligned, proportional.
    /// Example: --partition-mode apportion-regions --weights county --alpha-county 3.0
    #[arg(long)]
    pub weights_override: Option<WeightMode>,

    /// COMPOSITOR Layer 3 — override the seed search strategy.
    /// Possible values: single, multi, convergence.
    /// Use --seeds N with multi; --convergence-threshold T with convergence.
    /// Example: --partition-mode geosection --search convergence --convergence-threshold 500
    #[arg(long)]
    pub search: Option<SearchMode>,

    /// Seeds for budgeted searches such as --search multi.
    /// Convergence uses --convergence-threshold independently of this budget.
    /// Overrides --geosection-seeds and --compact-seeds when --search is set.
    #[arg(long)]
    pub seeds: Option<usize>,

    /// Convergence threshold for --search convergence (default: 500).
    /// Stops when this many consecutive seeds produce no EC improvement.
    #[arg(long, default_value_t = 500)]
    pub convergence_threshold: u32,

    /// Target percentile for --search percentile or bisection-ensemble (0.0–1.0).
    /// 0.0 = minimum EC (like convergence), 0.5 = median, 1.0 = maximum. Default: 0.5.
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

    /// METIS backend engine.
    /// c-ffi (default): links libmetis.so/dll — battle-tested, handles all k.
    /// metis-core: pure Rust, no C dependency — portable standalone binary.
    /// gpmetis: external gpmetis subprocess (reserved, not yet implemented).
    #[arg(long, value_enum)]
    pub metis_engine: Option<MetisEngineArg>,

    // ── Spec 1: custom parameters ─────────────────────────────────────────────
    /// Override district count. For house/senate chambers, count is auto-resolved
    /// from state policy (e.g., WA house=98) when this flag is not provided.
    #[arg(long)]
    pub districts: Option<usize>,

    /// Chamber type: congressional, house, senate, custom.
    /// When house or senate, district count is auto-resolved from state policy.
    #[arg(long, default_value = "congressional")]
    pub chamber: String,

    /// Human label for this plan run (default: {state}_{chamber}_{year})
    #[arg(long)]
    pub label: Option<String>,

    /// Population source for balance metric.
    /// Values: total (default), vap (total voting-age population),
    /// cvap (citizen voting-age population, requires CVAP special tabulation download),
    /// prison_adjusted (total population minus incarcerated-at-prison, requires
    ///   Prison Policy Initiative LSPC adjustment table preprocessing; see N.1).
    /// Note: "cvap" = "citizen_vap" in the N-track research papers.
    /// Falls back to "total" if the required data file is not present.
    #[arg(long, default_value = "total")]
    pub population_source: String,

    /// Max deviation per district in percent (default: 0.5 congressional, 5.0 state)
    #[arg(long)]
    pub balance_tolerance: Option<f64>,

    /// Number of seats per constituency for multi-member systems (default: 1).
    /// Malta: 5, Ireland: 3-5 (use 4 for average), Germany: 1
    /// For uniform seat counts: each district gets exactly this many seats.
    /// Balance formula: population/seats ~ ideal_per_seat (= total_pop / (districts x seats))
    #[arg(long, default_value_t = 1)]
    pub seats_per_district: usize,

    /// Total seat count (alternative to --seats-per-district x --districts).
    /// Specify either --total-seats OR --seats-per-district, not both.
    /// If specified: ideal_per_seat = total_pop / total_seats; districts can have varying seat counts.
    #[arg(long)]
    pub total_seats: Option<usize>,

    /// Write manifest.json alongside outputs
    #[arg(long, default_value_t = true)]
    pub manifest: bool,

    /// Overwrite existing plan without error
    #[arg(long, default_value_t = false)]
    pub force: bool,

    // ── International / research use ─────────────────────────────────────────
    /// Direct path to .adj.bin adjacency file (bypasses manifest lookup).
    /// Required for international states not in the embedded US manifest.
    /// Example: --adjacency outputs/international/mt_adjacency_2021.adj.bin
    /// When set, --districts must also be provided.
    #[arg(long)]
    pub adjacency: Option<String>,

    /// Human-readable name for the state/country (used in file paths and labels).
    /// Defaults to lowercase state code when --adjacency is used.
    /// Example: --state-name "malta" or --state-name "ireland"
    #[arg(long)]
    pub state_name: Option<String>,

    /// Path to JSON file mapping GEOID -> weight (0.0-1.0) for COI preservation
    /// Higher weight = stronger preference to keep tract with its neighbors
    #[arg(long)]
    pub coi_weights: Option<String>,

    /// Print pure partition time (graph construction + METIS) separately from I/O.
    /// Output goes to stderr: "[partition-time] <STATE>: k=<K> n=<N> -> <ms>ms"
    /// Useful for benchmarking partition cost vs. adjacency loading and output writing.
    #[arg(long, default_value_t = false)]
    pub time_partition: bool,

    /// VRASection (T.7): alignment score weight.
    /// Controls the trade-off between edge-cut compactness (60%) and geographic
    /// minority-VAP alignment (40%). Range [0.0, 1.0]. Default 0.40.
    /// Requires --partition-mode vra-section.
    #[arg(long, default_value_t = 0.40)]
    pub w_vra: f64,

    // ── Simulated Annealing (SA) parameters ───────────────────────────────────
    /// SA: steps per tract per bisection node. n_steps = sa_steps_per_tract * |subgraph|.
    /// Default 10. Higher = more refinement, slower.
    /// Requires --partition-mode simulated-annealing.
    #[arg(long, default_value_t = 10)]
    pub sa_steps_per_tract: usize,

    /// SA: T_0 factor. T_0 = max(1.0, sa_t0_factor * EC(initial_metis)).
    /// Default 0.01. Larger = more initial randomness.
    /// Requires --partition-mode simulated-annealing.
    #[arg(long, default_value_t = 0.01)]
    pub sa_t0_factor: f64,

    /// SA: final temperature (near-zero). Default 1e-4.
    /// Requires --partition-mode simulated-annealing.
    #[arg(long, default_value_t = 1e-4)]
    pub sa_t_final: f64,

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
    /// Seeds move to medoid of each district until stable or n_iter reached.
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
    /// branch-and-cut and iterative-separation currently emit U.16 certificates
    /// but still fall back until a concrete solver backend is enabled.
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
    /// Higher = finer angle resolution, slower. 36 (every 5°) is fast but coarser.
    /// Requires --partition-mode moving-knife or --structure moving-knife.
    #[arg(long, default_value_t = 180usize)]
    pub mka_orientations: usize,

    /// MKA: compactness metric to maximise (default: reock).
    /// reock: Area(D)/Area(MEC(D)) — requires centroid positions only.
    /// polsby: 4π·Area(D)/Perimeter(D)² — Phase 1 falls back to reock.
    #[arg(long, default_value = "reock")]
    pub mka_metric: String,

    /// Spectral: smoothing iterations per bisection node (default: 200).
    /// Requires --partition-mode spectral or --structure spectral.
    #[arg(long, default_value_t = 200usize)]
    pub spectral_iters: usize,

    // ── SMC-Percentile parameters ─────────────────────────────────────────────
    /// Number of SMC particles for --search smc-percentile (default: 5000).
    /// Use 1000 for quick checks, 50000+ for publication-quality inference.
    #[arg(long)]
    pub particles: Option<usize>,

    /// SMC resample threshold for --search smc-percentile (default: 0.5).
    /// Lower (0.3) for irregular geometries or VRA-heavy states.
    #[arg(long, default_value_t = 0.5)]
    pub smc_resample_threshold: f64,
}
