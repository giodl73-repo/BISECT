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

    /// DPI for output maps (default: 150)
    #[arg(long, default_value_t = 150, value_parser = clap::value_parser!(u32).range(72..=300))]
    pub dpi: u32,

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
    #[arg(short = 's', long, value_delimiter = ' ', num_args = 1..)]
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

    /// DPI for output maps (default: 150)
    #[arg(long, default_value_t = 150)]
    pub dpi: u32,

    /// Progress bar position for parallel mode (default: 2)
    #[arg(long, default_value_t = 2)]
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

    /// DPI for output maps (default: 150)
    #[arg(long, default_value_t = 150)]
    pub dpi: u32,

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
        assert_eq!(args.dpi, 150);
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
    fn test_seed_optional() {
        let no_seed = StateArgs::parse_from(["state", "--state", "VT"]);
        assert!(no_seed.seed.is_none());
        let with_seed = StateArgs::parse_from(["state", "--state", "VT", "--seed", "42"]);
        assert_eq!(with_seed.seed, Some(42));
    }
}
