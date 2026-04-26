/// CLI argument definitions — exact mirror of the Python argparse surface.
///
/// Every flag here corresponds 1:1 with an argument in one of:
///   run_complete_redistricting.py   (outermost orchestrator)
///   run_states_parallel.py          (parallel runner)
///   run_state_redistricting.py      (per-state runner)
///
/// MERIDIAN invariant: if Python adds or changes a flag, this file must be
/// updated in the same commit. Any discrepancy = silent behavioural drift.
use clap::{Parser, Subcommand, ValueEnum};

// ---------------------------------------------------------------------------
// Shared enums (used by multiple subcommands)
// ---------------------------------------------------------------------------

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
        match self { Self::Y2020 => write!(f, "2020"), Self::Y2010 => write!(f, "2010"), Self::Y2000 => write!(f, "2000") }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum PartitionMode {
    #[value(name = "unweighted")]
    Unweighted,
    #[value(name = "edge-weighted")]
    EdgeWeighted,
    #[value(name = "metis-vra")]
    MetisVra,
}

impl std::fmt::Display for PartitionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unweighted   => write!(f, "unweighted"),
            Self::EdgeWeighted => write!(f, "edge-weighted"),
            Self::MetisVra     => write!(f, "metis-vra"),
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
pub enum Resolution {
    #[value(name = "tract")]
    Tract,
    #[value(name = "block_group")]
    BlockGroup,
    #[value(name = "block")]
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MetisObjective {
    #[value(name = "cut")]
    Cut,
    #[value(name = "vol")]
    Vol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RunType {
    #[value(name = "production")]
    Production,
    #[value(name = "experiment")]
    Experiment,
    #[value(name = "test")]
    Test,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ProcessingMode {
    #[value(name = "streaming")]
    Streaming,
    #[value(name = "batch")]
    Batch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Stage {
    #[value(name = "data")]
    Data,
    #[value(name = "states")]
    States,
    #[value(name = "nation")]
    Nation,
}

// ---------------------------------------------------------------------------
// Top-level CLI
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(
    name = "redist",
    about = "Congressional redistricting pipeline -- compact, population-balanced districts",
    version,
    propagate_version = true,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run the full redistricting pipeline (mirrors run_complete_redistricting.py)
    Run(RunArgs),
    /// Run a single state (mirrors run_state_redistricting.py)
    State(StateArgs),
    /// Run all states in parallel for one year (mirrors run_states_parallel.py)
    States(StatesArgs),
    /// Download census data needed to run redistricting
    Fetch(FetchArgs),
    /// Compute per-district analytics (demographic, political, urban, summary)
    Analyze(AnalyzeArgs),
    /// Render district maps to PNG
    Map(MapArgs),
    /// Merge all state analysis outputs into national datasets
    Aggregate(AggregateArgs),
    /// Validate a .rplan file for format correctness and coverage
    Validate(ValidateArgs),
    /// Copy a legacy state plan into the plans/{label}/ tree
    Migrate(MigrateArgs),
}

// ---------------------------------------------------------------------------
// `redist validate` — validate a .rplan file
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
#[command(disable_version_flag = true)]
pub struct ValidateArgs {
    /// Path to .rplan file to validate
    #[arg(long)]
    pub file: std::path::PathBuf,
    /// Strict mode: fail on warnings
    #[arg(long, default_value_t = false)]
    pub strict: bool,
}

// ---------------------------------------------------------------------------
// `redist migrate` — copy legacy state plan into plans/{label}/ tree
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
#[command(disable_version_flag = true)]
pub struct MigrateArgs {
    /// Source state code (e.g. WA)
    #[arg(long)]
    pub state: String,
    /// Target label for the new plan directory
    #[arg(long)]
    pub label: String,
    /// Version directory (e.g. v1)
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,
    /// Census year
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,
}

// ---------------------------------------------------------------------------
// `redist analyze` — per-district analytics
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct AnalyzeArgs {
    /// Two-letter state code (e.g., VT, AL)
    #[arg(long)]
    pub state: String,

    /// Census year (default: 2020)
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: Year,

    /// Version identifier (default: v1)
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,

    /// Output base directory (default: outputs)
    #[arg(long, default_value = "outputs")]
    pub output_base: String,

    /// Analyzers to run (default: all)
    #[arg(long = "types", value_delimiter = ' ', num_args = 0..,
          default_values = ["all"])]
    pub types: Vec<redist_analysis::AnalyzerType>,

    /// Re-run even if output already exists
    #[arg(long)]
    pub force: bool,

    /// Allow unconstitutional population imbalance without exiting non-zero (research use only)
    #[arg(long)]
    pub allow_imbalance: bool,
}

// ---------------------------------------------------------------------------
// `redist map` — PNG map rendering
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum MapType {
    Districts,
    Rounds,
    Political,
    Demographic,
    Compactness,
    All,
}

impl MapType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Districts   => "districts",
            Self::Rounds      => "rounds",
            Self::Political   => "political",
            Self::Demographic => "demographic",
            Self::Compactness => "compactness",
            Self::All         => "all",
        }
    }
}

/// Map rendering scope.
#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum MapScope {
    /// Render a single state (default)
    State,
    /// Render all present states on one national map
    National,
}

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct MapArgs {
    /// Two-letter state code (ignored when --scope national)
    #[arg(long, default_value = "")]
    pub state: String,

    /// Rendering scope: state (default) or national
    #[arg(long, default_value = "state")]
    pub scope: MapScope,

    /// Census year (default: 2020)
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: Year,

    /// Version identifier (default: v1)
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,

    /// Map types to render (default: districts)
    #[arg(long = "types", value_delimiter = ' ', num_args = 0..,
          default_values = ["districts"])]
    pub types: Vec<MapType>,

    /// DPI for output maps
    #[arg(long, default_value = "150",
          value_parser = clap::builder::PossibleValuesParser::new(["72","100","150","200","300"]))]
    pub dpi: String,

    /// Re-render even if output already exists
    #[arg(long)]
    pub force: bool,
}

// ---------------------------------------------------------------------------
// `redist run` — full pipeline (run_complete_redistricting.py)
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct RunArgs {
    /// Census year: 2020, 2010, 2000, or "all" (default: all)
    #[arg(short = 'y', long, default_value = "all")]
    pub year: String,  // "all" or one of Year enum

    /// Version identifier (default: v1)
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,

    /// Output directory override
    #[arg(long)]
    pub output_dir: Option<String>,

    /// Number of parallel workers (default: 12)
    #[arg(short = 'w', long, default_value_t = 12)]
    pub workers: usize,

    /// DPI for output maps: 72, 100, 150, 200, or 300 (default: 150)
    /// Note: Python argparse accepts only these discrete values; Rust mirrors that.
    #[arg(long, default_value = "150", value_parser = clap::builder::PossibleValuesParser::new(["72", "100", "150", "200", "300"]))]
    pub dpi: String,

    /// Election year for political analysis (default: 2020)
    #[arg(short = 'e', long = "election-year", default_value = "2020")]
    pub election_year: ElectionYear,

    /// Skip per-state analysis (political, demographic, compactness)
    #[arg(long)]
    pub skip_analysis: bool,

    /// Skip political analysis steps
    #[arg(long)]
    pub skip_political: bool,

    /// Skip demographic analysis steps
    #[arg(long)]
    pub skip_demographic: bool,

    /// Pipeline stages to run (default: data states nation)
    #[arg(short = 's', long, value_delimiter = ' ', num_args = 0.., default_value = "data states nation")]
    pub stages: Vec<Stage>,

    /// Reprocess all states (ignore completion markers)
    #[arg(long)]
    pub reprocess: bool,

    /// Delete output directory before starting (fresh run)
    #[arg(short = 'r', long)]
    pub reset: bool,

    /// Print commands without executing (dry run)
    #[arg(short = 'p', long)]
    pub print_only: bool,

    /// Enable debug mode
    #[arg(short = 'd', long)]
    pub debug: bool,

    /// Partitioning mode (default: edge-weighted)
    #[arg(short = 'm', long = "partition-mode", default_value = "edge-weighted")]
    pub partition_mode: PartitionMode,

    /// Processing mode: streaming (default) or batch
    #[arg(long, default_value = "streaming")]
    pub processing_mode: ProcessingMode,

    /// Minimum shared boundary length in metres to include an adjacency edge (default: 10.0)
    #[arg(long, default_value_t = 10.0)]
    pub minimum_boundary_length: f64,

    /// Run type: production, experiment, or test
    #[arg(long, default_value = "production")]
    pub run_type: RunType,

    /// Experiment name (required when --run-type=experiment)
    #[arg(long)]
    pub experiment_name: Option<String>,

    /// Specific state codes to process (default: all)
    #[arg(long = "states", num_args = 0.., value_delimiter = ' ')]
    pub states: Vec<String>,
}

// ---------------------------------------------------------------------------
// `redist state` — single state (run_state_redistricting.py)
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct StateArgs {
    /// Two-letter state code (e.g., CA, TX, VT)
    #[arg(long)]
    pub state: String,

    /// Census year (default: 2020)
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: Year,

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

    /// Target majority-minority districts (VRA mode only)
    #[arg(long)]
    pub target_mm_districts: Option<usize>,

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

    // ── Spec 1: custom parameters ─────────────────────────────────────────────

    /// Override district count (enables non-congressional chambers)
    #[arg(long)]
    pub districts: Option<usize>,

    /// Chamber type: congressional, house, senate, custom
    #[arg(long, default_value = "congressional")]
    pub chamber: String,

    /// Human label for this plan run (default: {state}_{chamber}_{year})
    #[arg(long)]
    pub label: Option<String>,

    /// Population source: total, vap, cvap
    #[arg(long, default_value = "total")]
    pub population_source: String,

    /// Max deviation per district in percent (default: 0.5 congressional, 5.0 state)
    #[arg(long)]
    pub balance_tolerance: Option<f64>,

    /// Write manifest.json alongside outputs
    #[arg(long, default_value_t = true)]
    pub manifest: bool,

    /// Overwrite existing plan without error
    #[arg(long, default_value_t = false)]
    pub force: bool,
}

// ---------------------------------------------------------------------------
// `redist states` — all states in parallel (run_states_parallel.py)
// ---------------------------------------------------------------------------

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_debug_assert() {
        // clap will panic if the CLI definition is internally inconsistent
        Cli::command().debug_assert();
    }

    #[test]
    fn test_run_defaults() {
        let args = RunArgs::parse_from(["run"]);
        assert_eq!(args.year, "all");
        assert_eq!(args.version, "v1");
        assert_eq!(args.workers, 12);
        assert_eq!(args.dpi, "150");
        assert!(!args.skip_analysis);
        assert!(!args.reprocess);
        assert!(!args.reset);
        assert_eq!(args.partition_mode, PartitionMode::EdgeWeighted);
        assert_eq!(args.minimum_boundary_length, 10.0);
    }

    #[test]
    fn test_state_defaults() {
        let args = StateArgs::parse_from(["state", "--state", "VT"]);
        assert_eq!(args.state, "VT");
        assert_eq!(args.year, Year::Y2020);
        assert_eq!(args.version, "v1");
        assert_eq!(args.ufactor, 5);
        assert_eq!(args.niter, 100);
        assert_eq!(args.objtype, MetisObjective::Cut);
        assert!(!args.reset);
    }

    #[test]
    fn test_partition_mode_values() {
        let args = StateArgs::parse_from([
            "state", "--state", "AL", "--partition-mode", "metis-vra"
        ]);
        assert_eq!(args.partition_mode, PartitionMode::MetisVra);
    }

    #[test]
    fn test_states_args_required_fields() {
        let args = StatesArgs::parse_from([
            "states", "--year", "2020", "--version", "V3", "--output-dir", "/tmp/out"
        ]);
        assert_eq!(args.year, Year::Y2020);
        assert_eq!(args.version, "V3");
        assert_eq!(args.workers, 4);
        assert!(!args.run_analysis);
    }

    #[test]
    fn test_year_enum_2010() {
        let args = StateArgs::parse_from([
            "state", "--state", "CA", "--year", "2010"
        ]);
        assert_eq!(args.year, Year::Y2010);
    }

    #[test]
    fn test_states_filter_list() {
        let args = RunArgs::parse_from([
            "run", "--states", "VT", "DE", "AK"
        ]);
        assert_eq!(args.states, vec!["VT", "DE", "AK"]);
    }

    #[test]
    fn test_boolean_flags_default_false() {
        let args = StateArgs::parse_from(["state", "--state", "TX"]);
        assert!(!args.print_only);
        assert!(!args.debug);
        assert!(!args.reset);
    }

    #[test]
    fn test_stages_default_all_three() {
        let args = RunArgs::parse_from(["run"]);
        // Must default to all three stages, not empty — empty Vec silently skips all stages
        assert_eq!(args.stages.len(), 3,
            "stages should default to [data, states, nation], got {:?}", args.stages);
        assert!(args.stages.contains(&Stage::Data));
        assert!(args.stages.contains(&Stage::States));
        assert!(args.stages.contains(&Stage::Nation));
    }

    #[test]
    fn test_stages_explicit_subset() {
        let args = RunArgs::parse_from(["run", "--stages", "states", "nation"]);
        assert_eq!(args.stages.len(), 2);
        assert!(!args.stages.contains(&Stage::Data));
    }

    #[test]
    fn test_workers_defaults_match_python() {
        let run_args = RunArgs::parse_from(["run"]);
        assert_eq!(run_args.workers, 12, "run_complete_redistricting.py defaults to 12");
        let states_args = StatesArgs::parse_from([
            "states", "--year", "2020", "--version", "v1", "--output-dir", "/tmp"
        ]);
        assert_eq!(states_args.workers, 4, "run_states_parallel.py defaults to 4");
    }

    #[test]
    fn test_election_year_long_flag() {
        let args = RunArgs::parse_from(["run", "--election-year", "2016"]);
        assert_eq!(args.election_year, ElectionYear::E2016);
    }

    #[test]
    fn test_seed_optional() {
        let no_seed = StateArgs::parse_from(["state", "--state", "VT"]);
        assert!(no_seed.seed.is_none());
        let with_seed = StateArgs::parse_from(["state", "--state", "VT", "--seed", "42"]);
        assert_eq!(with_seed.seed, Some(42));
    }

    #[test]
    fn test_analyze_defaults() {
        let args = AnalyzeArgs::parse_from(["analyze", "--state", "VT"]);
        assert_eq!(args.state, "VT");
        assert_eq!(args.year, Year::Y2020);
        assert_eq!(args.version, "v1");
        assert!(!args.force);
        assert!(!args.allow_imbalance);
        assert!(args.types.contains(&redist_analysis::AnalyzerType::All));
    }

    #[test]
    fn test_analyze_explicit_types() {
        let args = AnalyzeArgs::parse_from([
            "analyze", "--state", "AL", "--types", "demographic", "political"
        ]);
        assert!(args.types.contains(&redist_analysis::AnalyzerType::Demographic));
        assert!(args.types.contains(&redist_analysis::AnalyzerType::Political));
        assert!(!args.types.contains(&redist_analysis::AnalyzerType::All));
    }

    #[test]
    fn test_analyze_allow_imbalance_flag() {
        let args = AnalyzeArgs::parse_from(["analyze", "--state", "VT", "--allow-imbalance"]);
        assert!(args.allow_imbalance);
    }

    #[test]
    fn test_map_defaults() {
        let args = MapArgs::parse_from(["map", "--state", "VT"]);
        assert_eq!(args.state, "VT");
        assert_eq!(args.year, Year::Y2020);
        assert_eq!(args.dpi, "150");
        assert!(!args.force);
        assert!(args.types.contains(&MapType::Districts));
    }

    #[test]
    fn test_map_types_parse() {
        let args = MapArgs::parse_from([
            "map", "--state", "VT", "--types", "districts", "rounds"
        ]);
        assert!(args.types.contains(&MapType::Districts));
        assert!(args.types.contains(&MapType::Rounds));
    }

    // --- Task 7: Spec 1 flag parsing tests ---

    fn parse_state_args(extra: &[&str]) -> StateArgs {
        let mut base = vec!["redist", "--state", "WA", "--year", "2020", "--version", "v1"];
        base.extend_from_slice(extra);
        StateArgs::try_parse_from(base).expect("failed to parse StateArgs")
    }

    #[test]
    fn test_districts_flag_parsed() {
        let args = parse_state_args(&["--districts", "98"]);
        assert_eq!(args.districts, Some(98));
    }

    #[test]
    fn test_chamber_flag_default_is_congressional() {
        let args = parse_state_args(&[]);
        assert_eq!(args.chamber, "congressional");
    }

    #[test]
    fn test_chamber_flag_parsed() {
        let args = parse_state_args(&["--chamber", "house"]);
        assert_eq!(args.chamber, "house");
    }

    #[test]
    fn test_label_flag_parsed() {
        let args = parse_state_args(&["--label", "wa_house_draft1"]);
        assert_eq!(args.label.as_deref(), Some("wa_house_draft1"));
    }

    #[test]
    fn test_balance_tolerance_flag_parsed() {
        let args = parse_state_args(&["--balance-tolerance", "5.0"]);
        assert!((args.balance_tolerance.unwrap() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_population_source_flag_parsed() {
        let args = parse_state_args(&["--population-source", "vap"]);
        assert_eq!(args.population_source, "vap");
    }

    #[test]
    fn test_force_flag_default_false() {
        let args = parse_state_args(&[]);
        assert!(!args.force);
    }

    #[test]
    fn test_force_flag_set() {
        let args = parse_state_args(&["--force"]);
        assert!(args.force);
    }

    #[test]
    fn test_population_source_default_is_total() {
        let args = parse_state_args(&[]);
        assert_eq!(args.population_source, "total");
    }

    #[test]
    fn test_districts_default_is_none() {
        let args = parse_state_args(&[]);
        assert_eq!(args.districts, None);
    }

    #[test]
    fn test_label_default_is_none() {
        let args = parse_state_args(&[]);
        assert!(args.label.is_none());
    }
}

// ---------------------------------------------------------------------------
// `redist fetch` — data download
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct FetchArgs {
    /// Census year: 2020, 2010, 2000, or all [default: 2020]
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,

    /// State codes to fetch (default: all 50)
    #[arg(long = "states", num_args = 0.., value_delimiter = ' ')]
    pub states: Vec<String>,

    /// Data types: tiger, redistricting, adjacency, all [default: all]
    #[arg(long = "type", num_args = 0.., value_delimiter = ' ')]
    pub data_types: Vec<String>,

    /// Pull adjacency data from GitHub Releases (requires gh auth login)
    #[arg(long)]
    pub release: bool,

    /// Custom manifest file path (overrides REDIST_MANIFEST env var)
    #[arg(long)]
    pub manifest: Option<String>,

    /// Print what would be downloaded without downloading
    #[arg(long)]
    pub check_only: bool,

    /// Re-download even if files already exist
    #[arg(long)]
    pub force: bool,

    /// Parallel download workers [default: 4]
    #[arg(short = 'w', long, default_value_t = 4)]
    pub workers: usize,
}

// ---------------------------------------------------------------------------
// `redist aggregate` — merge state analysis into national datasets
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct AggregateArgs {
    /// Census year (default: 2020)
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: Year,

    /// Version identifier (default: v1)
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,

    /// Analyzer types to aggregate (default: all)
    #[arg(long = "types", value_delimiter = ' ', num_args = 0..,
          default_values = ["all"])]
    pub types: Vec<redist_analysis::AnalyzerType>,

    /// Write CSV alongside JSON
    #[arg(long)]
    pub csv: bool,

    /// Re-aggregate even if output exists
    #[arg(long)]
    pub force: bool,
}

