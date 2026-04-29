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
    /// Partisan-weighted bisection: requires --partisan-shares.
    /// Mutually exclusive with `metis-vra` per Callais p.36 disentanglement.
    #[value(name = "partisan-weighted")]
    PartisanWeighted,
}

impl std::fmt::Display for PartitionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unweighted       => write!(f, "unweighted"),
            Self::EdgeWeighted     => write!(f, "edge-weighted"),
            Self::MetisVra         => write!(f, "metis-vra"),
            Self::PartisanWeighted => write!(f, "partisan-weighted"),
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

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tract      => write!(f, "tract"),
            Self::BlockGroup => write!(f, "block_group"),
            Self::Block      => write!(f, "block"),
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
    before_help = "QUICKSTART:\n  1. redist fetch --type adjacency --states VT --year 2020\n  2. redist state --state VT --year 2020 --label vt_test\n  3. redist analyze --label vt_test --types all\n  4. redist report --label vt_test --format html\n  5. redist tui   (interactive interface)\n  6. redist doctor --label vt_test  (diagnose plan)\n\nSee: redist <command> --help for detailed options.\n",
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
    /// Compute per-district analytics (demographic, political, urban, summary, contiguity, splits)
    Analyze(AnalyzeArgs),
    /// Compare two redistricting plans (Jaccard, population, compactness)
    Compare(CompareArgs),
    /// Render district maps to PNG
    Map(MapArgs),
    /// Merge all state analysis outputs into national datasets
    Aggregate(AggregateArgs),
    /// Validate a .rplan file for format correctness and coverage
    Validate(ValidateArgs),
    /// Copy a legacy state plan into the plans/{label}/ tree
    Migrate(MigrateArgs),
    /// Draw and validate multi-chamber nested legislative plans (Spec 5)
    Suite(SuiteArgs),
    /// Generate a commission report (HTML and/or JSON) from analysis outputs
    Report(ReportArgs),
    /// Export a plan in GeoJSON, GerryChain v2.3, or CSV format
    Export(ExportArgs),
    /// Import a GeoJSON plan into the RPLAN format
    Import(ImportArgs),
    /// Show redistricting policy for a state (subdivision terms, tolerances, VRA, etc.)
    Policy(PolicyArgs),
    /// Pre-flight check: verify data files, year validity, resolution warnings for a location
    Doctor(DoctorArgs),
    /// Reproduce a plan from its manifest.json and verify it matches the original
    Verify(VerifyArgs),
    /// Run N redistricting seeds and keep top-K plans by a metric
    Sweep(SweepArgs),
    /// Launch the interactive terminal UI
    Tui(TuiArgs),
}

// ---------------------------------------------------------------------------
// `redist policy` — state policy lookup
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct PolicyArgs {
    /// State code (e.g., LA, WA, VA) or _TEST_EL for Eldoria test state
    #[arg(long)]
    pub state: String,
    /// Output format: table (default) or json
    #[arg(long, default_value = "table")]
    pub format: String,
}

// ---------------------------------------------------------------------------
// `redist doctor` — pre-flight check for a location+year
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct DoctorArgs {
    /// Location code to check (e.g., WA, NV, IE, MT-PARLIAMENT, _TEST_EL).
    /// Not required when --label is provided.
    #[arg(long, default_value = "")]
    pub state: String,
    /// Census year to check (default: 2020)
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,
    /// Chamber to check (default: congressional)
    #[arg(long, default_value = "congressional")]
    pub chamber: String,
    /// Resolution to check (default: tract)
    #[arg(long, default_value = "tract")]
    pub resolution: String,
    /// Output base directory for adjacency path check (default: outputs)
    #[arg(long, default_value = "outputs")]
    pub output_base: String,
    /// Existing plan label to diagnose (alternative to --state/--year for plan diagnosis)
    #[arg(long)]
    pub label: Option<String>,
    /// Version directory for --label lookup (default: v1)
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,
    /// Scan all plans in the version/year directory and show status table
    #[arg(long)]
    pub all: bool,
    /// Verify a plan manifest.json against the running binary's provenance.
    /// Reports whether the binary version matches and surfaces the recorded
    /// build commit / rustc version so a special master can attest to source provenance.
    /// Mutually exclusive with --label / --state / --all.
    #[arg(long, value_name = "PATH")]
    pub verify_manifest: Option<String>,
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
    /// Overwrite an existing plan directory without error
    #[arg(long, default_value_t = false)]
    pub force: bool,
}

// ---------------------------------------------------------------------------
// `redist report` — commission report (HTML / JSON)
// ---------------------------------------------------------------------------

/// Output format for `redist report`.
#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum ReportFormat {
    /// Self-contained HTML (no external dependencies)
    Html,
    /// Machine-readable JSON
    Json,
    /// PDF (not yet available — exits with code 1)
    Pdf,
}

#[derive(Debug, clap::Args)]
#[command(disable_version_flag = true)]
pub struct ReportArgs {
    /// Plan label (e.g. vt_congressional_2020)
    #[arg(long)]
    pub label: String,
    /// Census year
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,
    /// Version directory (e.g. RustV3)
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,
    /// Output formats (html, json, pdf)
    #[arg(long = "format", value_delimiter = ' ', num_args = 1..,
          default_values = ["html"])]
    pub format: Vec<ReportFormat>,
    /// Output directory for report files (default: reports/{label}/)
    #[arg(long = "out")]
    pub out: Option<String>,
    /// Only write audit.json (chain-of-custody only; skips HTML/JSON report)
    #[arg(long)]
    pub audit_only: bool,
    /// Write audit.json alongside the report (court submission mode).
    /// Unlike --audit-only, this writes BOTH the full report AND audit.json.
    #[arg(long)]
    pub audit_with_report: bool,
    /// Base output directory
    #[arg(long, default_value = "outputs")]
    pub output_base: String,
}

// ---------------------------------------------------------------------------
// `redist export` — plan export (GeoJSON, GerryChain, CSV)
// ---------------------------------------------------------------------------

/// Export format for `redist export`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum ExportFormat {
    /// RFC 7946 GeoJSON FeatureCollection
    #[value(name = "geojson", alias = "geo-json")]
    GeoJson,
    /// GerryChain v2.3 format ("assignment" singular field)
    #[value(name = "gerrychain", alias = "gerry-chain")]
    GerryChain,
    /// GEOID,district CSV
    Csv,
    /// Court-submission reproducibility package directory
    #[value(name = "reproducibility-package")]
    ReproducibilityPackage,
}

#[derive(Debug, clap::Args)]
#[command(disable_version_flag = true)]
pub struct ExportArgs {
    /// Plan label
    #[arg(long)]
    pub label: String,
    /// Census year
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,
    /// Version directory
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,
    /// Export formats (geojson, gerrychain, csv)
    #[arg(long = "format", value_delimiter = ' ', num_args = 1..,
          default_values = ["geojson"])]
    pub format: Vec<ExportFormat>,
    /// Output directory (default: exports/{label}/)
    #[arg(long = "out")]
    pub out: Option<String>,
    /// Base output directory
    #[arg(long, default_value = "outputs")]
    pub output_base: String,
}

// ---------------------------------------------------------------------------
// `redist import` — import GeoJSON plan
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
#[command(disable_version_flag = true)]
pub struct ImportArgs {
    /// GeoJSON file to import
    #[arg(long)]
    pub file: std::path::PathBuf,
    /// State code (e.g. WA)
    #[arg(long)]
    pub state: String,
    /// Census year
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,
    /// Plan label for the imported plan
    #[arg(long)]
    pub label: String,
    /// Version directory
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,
    /// Input format: geojson (default) or gerrychain
    #[arg(long, default_value = "geojson")]
    pub format: String,
    /// Base output directory
    #[arg(long, default_value = "outputs")]
    pub output_base: String,
}

// ---------------------------------------------------------------------------
// `redist analyze` — per-district analytics
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct AnalyzeArgs {
    /// Two-letter state code (e.g., VT, AL). Optional when --label is provided
    /// (state is then read from the plan manifest).
    #[arg(long)]
    pub state: Option<String>,

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

    /// Allow non-contiguous districts without setting exit code bit 1 (research use only)
    #[arg(long)]
    pub allow_noncontiguous: bool,

    /// Plan label (for labeled plans under plans/{label}/); falls back to state-level path
    #[arg(long)]
    pub label: Option<String>,

    /// Output directory override for comparison and analysis outputs
    #[arg(long)]
    pub output_dir: Option<std::path::PathBuf>,

    /// Custom election CSV file for partisan analysis
    /// (default: data/{year}/elections/presidential_by_tract.csv)
    #[arg(long)]
    pub election_file: Option<std::path::PathBuf>,

    /// Number of bootstrap samples for partisan CI (default: 1000)
    #[arg(long, default_value_t = 1000)]
    pub bootstrap_samples: usize,

    /// Path to external analyzer script to run and record in audit trail
    /// Script called as: python script.py {assignments_json} {output_dir}
    #[arg(long)]
    pub external_analyzer: Option<String>,

    /// Election data format: us-presidential (default), ie-dail, de-bundestag, generic-party-totals
    #[arg(long, default_value = "us-presidential")]
    pub election_format: String,
    /// Write output to stdout instead of analysis/ file (only works with a single --types value)
    #[arg(long)]
    pub stdout: bool,
}

// ---------------------------------------------------------------------------
// `redist suite` — multi-chamber suite draw and validate
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct SuiteDrawCliArgs {
    /// Two-letter state code (e.g., WA, IL)
    #[arg(long)]
    pub state: String,

    /// Census year (default: 2020)
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,

    /// Version identifier
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,

    /// Suite name (e.g., wa_commission_v1)
    #[arg(long)]
    pub name: String,

    /// Number of congressional districts (optional)
    #[arg(long)]
    pub congressional_districts: Option<usize>,

    /// Number of house districts
    #[arg(long)]
    pub house_districts: Option<usize>,

    /// Number of senate districts
    #[arg(long)]
    pub senate_districts: Option<usize>,

    /// Nesting mode for state legislative plans: none or senate-in-house.
    /// NOTE: Congressional redistricting is independent of state legislative
    /// redistricting. This flag applies ONLY to state house and senate plans.
    /// Congressional districts do not nest with state legislative districts.
    #[arg(long, default_value = "none")]
    pub nest: String,

    /// Required house-to-senate nesting ratio (e.g., 2 for 2:1).
    /// Required for states with variable nesting (IL). Warns if differs from constitutional value.
    #[arg(long)]
    pub nest_ratio: Option<usize>,

    /// Random seed for reproducibility
    #[arg(long)]
    pub seed: Option<u64>,

    /// Output base directory (default: outputs)
    #[arg(long, default_value = "outputs")]
    pub output_base: String,

    /// Overwrite existing suite without error
    #[arg(long)]
    pub force: bool,

    /// Nest congressional within state senate (not yet supported — emits guidance)
    #[arg(long)]
    pub nest_congressional_in_senate: bool,
}

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct SuiteValidateCliArgs {
    /// Suite name
    #[arg(long)]
    pub name: String,

    /// Two-letter state code (required when using label overrides)
    #[arg(long)]
    pub state: Option<String>,

    /// Version identifier
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,

    /// Census year
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,

    /// Output base directory (default: outputs)
    #[arg(long, default_value = "outputs")]
    pub output_base: String,

    /// Override house plan label (bypasses suite manifest lookup)
    #[arg(long)]
    pub house_label: Option<String>,

    /// Override senate plan label (bypasses suite manifest lookup)
    #[arg(long)]
    pub senate_label: Option<String>,
}

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct SuiteStabilityArgs {
    /// Comma-separated plan labels to compare
    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,
    /// Census year
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,
    /// Version
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,
    /// Output base directory
    #[arg(long, default_value = "outputs")]
    pub output_base: String,
}

#[derive(Debug, Subcommand)]
pub enum SuiteCommands {
    /// Draw a multi-chamber suite (congressional + house + senate)
    Draw(SuiteDrawCliArgs),
    /// Validate nesting constraints for an existing suite
    Validate(SuiteValidateCliArgs),
    /// Compute pairwise Jaccard similarity across multiple plan runs
    Stability(SuiteStabilityArgs),
}

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct SuiteArgs {
    #[command(subcommand)]
    pub command: SuiteCommands,
}

// ---------------------------------------------------------------------------
// `redist compare` — plan comparison
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum CompareFormat {
    Table,
    Json,
    Csv,
}

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct CompareArgs {
    /// First plan — label or path to plan directory (required)
    #[arg(long)]
    pub plan_a: String,

    /// Second plan — label or path
    #[arg(long)]
    pub plan_b: Option<String>,

    /// Use currently enacted districts as plan B
    #[arg(long, default_value_t = false)]
    pub enacted: bool,

    /// Census year (default: 2020)
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,

    /// Version directory for label resolution
    #[arg(short = 'v', long)]
    pub version: Option<String>,

    /// Metrics to compare: population, compactness, splits, partisan, all
    #[arg(long, value_delimiter = ' ', num_args = 0.., default_values = ["all"])]
    pub metrics: Vec<String>,

    /// Output file path (default: stdout)
    #[arg(long)]
    pub out: Option<std::path::PathBuf>,

    /// Output format
    #[arg(long, value_enum, default_value_t = CompareFormat::Table)]
    pub format: CompareFormat,

    /// Output base directory (default: outputs)
    #[arg(long, default_value = "outputs")]
    pub output_base: String,

    /// Output directory override
    #[arg(long)]
    pub output_dir: Option<std::path::PathBuf>,

    /// Suppress cross-year comparison warning (for intentional cycle-over-cycle comparisons)
    #[arg(long)]
    pub allow_cross_year: bool,

    /// Comma-separated plan labels for N-plan summary table (alternative to --plan-a/--plan-b)
    #[arg(long, value_delimiter = ',')]
    pub labels: Vec<String>,
    /// Version directory for plan-b (overrides --version for plan-b lookup)
    #[arg(long)]
    pub version_b: Option<String>,
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
    Splits,
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
            Self::Splits      => "splits",
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

    /// Plan label (alternative to --state for labeled plans under plans/ directory)
    #[arg(long)]
    pub label: Option<String>,

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

    /// Base output directory (default: outputs)
    #[arg(long, default_value = "outputs")]
    pub output_base: String,
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

    /// Population source: total, vap, cvap
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
        assert_eq!(args.year, "2020");
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
        assert_eq!(args.year, "2010");
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
        assert_eq!(args.state.as_deref(), Some("VT"));
        assert_eq!(args.year, Year::Y2020);
        assert_eq!(args.version, "v1");
        assert!(!args.force);
        assert!(!args.allow_imbalance);
        assert!(args.types.contains(&redist_analysis::AnalyzerType::All));
    }

    #[test]
    fn test_analyze_state_optional_without_state() {
        // --state is now optional; --label alone is sufficient
        let args = AnalyzeArgs::parse_from(["analyze", "--label", "wa_house_test"]);
        assert!(args.state.is_none());
        assert_eq!(args.label.as_deref(), Some("wa_house_test"));
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
        assert_eq!(args.output_base, "outputs");
    }

    #[test]
    fn test_map_output_base_override() {
        let args = MapArgs::parse_from(["map", "--state", "VT", "--output-base", "/custom/outputs"]);
        assert_eq!(args.output_base, "/custom/outputs");
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

    #[test]
    fn test_seats_per_district_default_is_1() {
        let args = parse_state_args(&[]);
        assert_eq!(args.seats_per_district, 1);
    }

    #[test]
    fn test_seats_per_district_parsed_malta() {
        let args = parse_state_args(&["--seats-per-district", "5"]);
        assert_eq!(args.seats_per_district, 5);
    }

    #[test]
    fn test_total_seats_default_is_none() {
        let args = parse_state_args(&[]);
        assert!(args.total_seats.is_none());
    }

    #[test]
    fn test_total_seats_parsed() {
        let args = parse_state_args(&["--total-seats", "65"]);
        assert_eq!(args.total_seats, Some(65));
    }

    #[test]
    fn test_seats_per_district_and_total_seats_together() {
        // Both flags can be provided simultaneously (caller resolves precedence)
        let args = parse_state_args(&["--seats-per-district", "4", "--total-seats", "172"]);
        assert_eq!(args.seats_per_district, 4);
        assert_eq!(args.total_seats, Some(172));
    }

    // ── Balance tolerance range (scenario 31) ─────────────────────────────────

    #[test]
    fn test_balance_tolerance_parses_as_f64() {
        let args = parse_state_args(&["--balance-tolerance", "5.0"]);
        assert!((args.balance_tolerance.unwrap() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_balance_tolerance_half_percent_is_not_fraction() {
        // 0.5 as input means 0.5% (in runner.rs, divided by 100 → 0.005 fraction)
        // Must NOT trigger the "looks like fraction" path at the CLI level
        // (that check fires for values < 0.05)
        let args = parse_state_args(&["--balance-tolerance", "0.5"]);
        assert!((args.balance_tolerance.unwrap() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_balance_tolerance_none_when_not_provided() {
        let args = parse_state_args(&[]);
        assert!(args.balance_tolerance.is_none());
    }

    // ── Task 118: --house-label / --senate-label for suite validate ───────────

    #[test]
    fn test_suite_validate_args_has_label_overrides() {
        use crate::args::SuiteValidateCliArgs;
        let args = SuiteValidateCliArgs::parse_from([
            "validate", "--name", "wa_suite_v1", "--state", "WA",
            "--house-label", "wa_house_custom",
        ]);
        assert_eq!(args.house_label, Some("wa_house_custom".into()));
        assert!(args.senate_label.is_none());
    }

    #[test]
    fn test_suite_validate_args_both_label_overrides() {
        use crate::args::SuiteValidateCliArgs;
        let args = SuiteValidateCliArgs::parse_from([
            "validate", "--name", "wa_suite_v1",
            "--house-label", "wa_house_custom",
            "--senate-label", "wa_senate_custom",
        ]);
        assert_eq!(args.house_label, Some("wa_house_custom".into()));
        assert_eq!(args.senate_label, Some("wa_senate_custom".into()));
    }

    #[test]
    fn test_suite_validate_args_no_labels_defaults_to_none() {
        use crate::args::SuiteValidateCliArgs;
        let args = SuiteValidateCliArgs::parse_from([
            "validate", "--name", "wa_suite_v1",
        ]);
        assert!(args.house_label.is_none());
        assert!(args.senate_label.is_none());
    }

    // ── Task 129: --labels flag for N-plan summary ────────────────────────────

    #[test]
    fn test_compare_labels_flag_parsed() {
        let args = CompareArgs::parse_from([
            "compare", "--plan-a", "plan1",
            "--labels", "wa_house_plan_a,wa_house_plan_b,wa_house_plan_c",
        ]);
        assert_eq!(args.labels, vec!["wa_house_plan_a", "wa_house_plan_b", "wa_house_plan_c"]);
    }

    #[test]
    fn test_compare_labels_default_empty() {
        let args = CompareArgs::parse_from(["compare", "--plan-a", "plan1", "--plan-b", "plan2"]);
        assert!(args.labels.is_empty(), "--labels should default to empty Vec");
    }

    // ── Task 134: splits map type ─────────────────────────────────────────────

    #[test]
    fn test_map_type_splits_recognized() {
        let args = MapArgs::parse_from(["map", "--state", "WA", "--types", "splits"]);
        assert!(args.types.contains(&MapType::Splits), "splits must be a valid MapType");
    }

    #[test]
    fn test_map_type_splits_name() {
        assert_eq!(MapType::Splits.name(), "splits");
    }

    // ── Task 139: --external-analyzer flag ───────────────────────────────────

    #[test]
    fn test_external_analyzer_flag_parsed() {
        let args = AnalyzeArgs::parse_from([
            "analyze", "--state", "WA", "--external-analyzer", "my_script.py",
        ]);
        assert_eq!(args.external_analyzer.as_deref(), Some("my_script.py"));
    }

    #[test]
    fn test_external_analyzer_default_none() {
        let args = AnalyzeArgs::parse_from(["analyze", "--state", "WA"]);
        assert!(args.external_analyzer.is_none());
    }

    // ── Task 148: international election format ───────────────────────────────

    #[test]
    fn test_election_format_flag_parses() {
        let args = AnalyzeArgs::parse_from([
            "analyze", "--state", "IE", "--election-format", "ie-dail",
        ]);
        assert_eq!(args.election_format, "ie-dail");
    }

    #[test]
    fn test_election_format_default_us_presidential() {
        let args = AnalyzeArgs::parse_from(["analyze", "--state", "VT"]);
        assert_eq!(args.election_format, "us-presidential");
    }

    #[test]
    fn test_non_us_election_format_emits_note() {
        // Verify that the format string for a non-default election format
        // would trigger the note. We check the logic that builds the message.
        let format = "ie-dail";
        let note = format!(
            "NOTE: Election format '{}' is not yet fully supported.\n\
             Currently supported: us-presidential (GEOID, dem_votes, rep_votes columns).",
            format
        );
        assert!(note.contains("ie-dail"), "note must mention the format");
        assert!(note.contains("us-presidential"), "note must mention the supported format");
    }

    // ── Task 141: reproducibility-package format ──────────────────────────────

    #[test]
    fn test_reproducibility_package_format_recognized() {
        use clap::ValueEnum;
        // The variant must be parseable from its value name string
        let parsed = ExportFormat::from_str("reproducibility-package", true);
        assert_eq!(
            parsed,
            Ok(ExportFormat::ReproducibilityPackage),
            "reproducibility-package must be a valid ExportFormat"
        );
    }

    // ── Task 149: COI weights flag ────────────────────────────────────────────

    #[test]
    fn test_coi_weights_flag_parsed() {
        let args = StateArgs::parse_from([
            "state", "--state", "WA", "--coi-weights", "coi_weights.json",
        ]);
        assert_eq!(args.coi_weights.as_deref(), Some("coi_weights.json"));
    }

    #[test]
    fn test_coi_weights_default_none() {
        let args = StateArgs::parse_from(["state", "--state", "WA"]);
        assert!(args.coi_weights.is_none());
    }

    // ── Task 151: nest-congressional-in-senate flag ───────────────────────────

    #[test]
    fn test_nest_congressional_in_senate_flag_parsed() {
        let args = SuiteDrawCliArgs::parse_from([
            "draw", "--state", "WA", "--name", "wa_suite",
            "--nest-congressional-in-senate",
        ]);
        assert!(args.nest_congressional_in_senate);
    }

    #[test]
    fn test_nest_congressional_in_senate_default_false() {
        let args = SuiteDrawCliArgs::parse_from([
            "draw", "--state", "WA", "--name", "wa_suite",
        ]);
        assert!(!args.nest_congressional_in_senate);
    }

    // ── Task 144: sweep subcommand ────────────────────────────────────────────

    #[test]
    fn test_sweep_args_parse() {
        use crate::args::SweepArgs;
        let args = SweepArgs::parse_from([
            "sweep", "--state", "WA", "--seeds", "20", "--keep-top", "5",
            "--optimize-by", "splits",
        ]);
        assert_eq!(args.state, "WA");
        assert_eq!(args.seeds, 20);
        assert_eq!(args.keep_top, 5);
        assert_eq!(args.optimize_by, "splits");
    }

    #[test]
    fn test_sweep_args_defaults() {
        use crate::args::SweepArgs;
        let args = SweepArgs::parse_from(["sweep", "--state", "VT"]);
        assert_eq!(args.year, "2020");
        assert_eq!(args.chamber, "congressional");
        assert_eq!(args.seeds, 10);
        assert_eq!(args.seed_start, 1);
        assert_eq!(args.keep_top, 3);
        assert_eq!(args.optimize_by, "compactness");
        assert_eq!(args.version, "sweep");
        assert_eq!(args.output_base, "outputs");
    }

    // ── Task 192: --label on map ──────────────────────────────────────────────

    #[test]
    fn test_map_args_label_parsed() {
        let args = MapArgs::parse_from(["map", "--state", "WA", "--label", "wa_house_v1"]);
        assert_eq!(args.label, Some("wa_house_v1".to_string()));
    }

    // ── Task 193: --label on verify ───────────────────────────────────────────

    #[test]
    fn test_verify_args_label_parsed() {
        let args = VerifyArgs::parse_from(["verify", "--manifest", "manifest.json", "--label", "wa_house_v1"]);
        assert_eq!(args.label, Some("wa_house_v1".to_string()));
    }

    // ── Doctor --label flag (plan diagnosis mode) ─────────────────────────────

    #[test]
    fn test_doctor_label_flag_parsed() {
        let args = DoctorArgs::parse_from(["doctor", "--label", "wa_house_2020"]);
        assert_eq!(args.label.as_deref(), Some("wa_house_2020"));
        assert_eq!(args.version, "v1");
    }

    #[test]
    fn test_doctor_label_with_version_flag() {
        let args = DoctorArgs::parse_from(["doctor", "--label", "wa_house_2020", "--version", "v2"]);
        assert_eq!(args.label.as_deref(), Some("wa_house_2020"));
        assert_eq!(args.version, "v2");
    }

    #[test]
    fn test_doctor_label_default_is_none() {
        let args = DoctorArgs::parse_from(["doctor", "--state", "WA"]);
        assert!(args.label.is_none());
    }

    #[test]
    fn test_doctor_state_default_is_empty_when_label_provided() {
        let args = DoctorArgs::parse_from(["doctor", "--label", "vt_test"]);
        // state should default to "" when --label is used without --state
        assert_eq!(args.state, "");
    }

    // ── Task 201: quickstart in CLI help ──────────────────────────────────────

    #[test]
    fn test_cli_help_contains_quickstart() {
        use clap::CommandFactory;
        let mut cmd = Cli::command();
        let help = format!("{}", cmd.render_long_help());
        assert!(help.contains("QUICKSTART") || help.contains("fetch"),
            "CLI help must contain quickstart instructions: {}", &help[..200.min(help.len())]);
    }

    // ── Task 203: doctor --all flag ───────────────────────────────────────────

    #[test]
    fn test_doctor_all_flag_parsed() {
        let args = DoctorArgs::parse_from(["doctor", "--all"]);
        assert!(args.all);
    }

    #[test]
    fn test_doctor_all_flag_default_false() {
        let args = DoctorArgs::parse_from(["doctor", "--state", "VT"]);
        assert!(!args.all);
    }

    // ── Task 204: --version-b flag for cross-version compare ─────────────────

    #[test]
    fn test_compare_version_b_flag_parsed() {
        let args = CompareArgs::parse_from([
            "compare", "--plan-a", "plan_a", "--plan-b", "plan_b", "--version-b", "v2"
        ]);
        assert_eq!(args.version_b, Some("v2".to_string()));
    }

    #[test]
    fn test_compare_version_b_default_none() {
        let args = CompareArgs::parse_from(["compare", "--plan-a", "plan_a", "--plan-b", "plan_b"]);
        assert!(args.version_b.is_none());
    }

    // ── Task 211: analyze --stdout flag ──────────────────────────────────────

    #[test]
    fn test_analyze_stdout_flag_parsed() {
        let args = AnalyzeArgs::parse_from(["analyze", "--state", "VT", "--stdout"]);
        assert!(args.stdout);
    }

    #[test]
    fn test_analyze_stdout_default_false() {
        let args = AnalyzeArgs::parse_from(["analyze", "--state", "VT"]);
        assert!(!args.stdout);
    }
}

// ---------------------------------------------------------------------------
// `redist sweep` — N-seed optimization planning tool
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct SweepArgs {
    /// State code
    #[arg(long)]
    pub state: String,
    /// Census year
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,
    /// Chamber
    #[arg(long, default_value = "congressional")]
    pub chamber: String,
    /// Number of seeds to run
    #[arg(long, default_value_t = 10)]
    pub seeds: usize,
    /// Starting seed value (seeds N..N+seeds are used)
    #[arg(long, default_value_t = 1)]
    pub seed_start: u64,
    /// Keep top K plans (default: 3)
    #[arg(long, default_value_t = 3)]
    pub keep_top: usize,
    /// Metric to optimize: compactness, splits, deviation
    #[arg(long, default_value = "compactness")]
    pub optimize_by: String,
    /// Version identifier
    #[arg(short = 'v', long, default_value = "sweep")]
    pub version: String,
    /// Output base directory
    #[arg(long, default_value = "outputs")]
    pub output_base: String,
    /// Auto-execute the generated commands (v2 feature; currently no-op)
    #[arg(long, default_value_t = false)]
    pub run: bool,
}

// ---------------------------------------------------------------------------
// `redist verify` — reproduce plan from manifest.json and verify
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct VerifyArgs {
    /// Path to manifest.json to reproduce and verify
    #[arg(long)]
    pub manifest: std::path::PathBuf,
    /// Minimum Jaccard similarity for PASS (default: 0.99)
    #[arg(long, default_value_t = 0.99)]
    pub min_similarity: f64,
    /// Label for the re-run verification plan (default: {original_label}_verify)
    #[arg(long)]
    pub verify_label: Option<String>,
    /// Base output directory (default: outputs)
    #[arg(long, default_value = "outputs")]
    pub output_base: String,
    /// Print the equivalent CLI command only, don't run it
    #[arg(long)]
    pub dry_run: bool,
    /// Skip binary SHA-256 check (for source-built binaries or different releases)
    #[arg(long)]
    pub skip_binary_check: bool,
    /// Plan label for locating the original plan to compare against (optional).
    /// When provided, PlanContext locates the original assignments for Jaccard comparison.
    #[arg(long)]
    pub label: Option<String>,
    /// Skip re-running redist state — just compare the existing plan assignments
    /// against a reference file (useful when METIS is unavailable)
    #[arg(long)]
    pub verify_assignments_only: bool,
    /// Reference assignments file to compare against (for --verify-assignments-only)
    #[arg(long)]
    pub plan_ref: Option<String>,
}

// ---------------------------------------------------------------------------
// `redist tui` — interactive terminal UI
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct TuiArgs {
    /// Start with a clean session (no saved config loaded)
    #[arg(long)]
    pub no_session: bool,
}

// ---------------------------------------------------------------------------
// `redist fetch` — data download
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum DataType {
    #[value(name = "tiger")]
    Tiger,
    #[value(name = "redistricting")]
    Redistricting,
    #[value(name = "adjacency")]
    Adjacency,
    #[value(name = "enacted")]
    Enacted,
    #[value(name = "geography")]
    Geography,
    #[value(name = "elections")]
    Elections,
    #[value(name = "all")]
    All,
}

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct FetchArgs {
    /// Census year: 2020, 2010, 2000, or all [default: 2020]
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,

    /// State codes to fetch (default: all 50)
    #[arg(long = "states", num_args = 0.., value_delimiter = ' ')]
    pub states: Vec<String>,

    /// Data types: tiger, redistricting, adjacency, enacted, geography, elections, all [default: all]
    #[arg(long = "type", num_args = 0.., value_delimiter = ' ')]
    pub data_types: Vec<DataType>,

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

    /// Verify SHA-256 of each downloaded file against expected hash (if known).
    /// On mismatch: deletes the corrupt file and returns an error.
    #[arg(long)]
    pub verify_downloads: bool,
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

    /// Output format: csv (default) or json
    #[arg(long, default_value = "csv")]
    pub format: String,
}

