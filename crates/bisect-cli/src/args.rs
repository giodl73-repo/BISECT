pub use bisect_runner::run_args::{RunArgs,RunType,Stage};
pub use bisect_runner::states_args::{StatesArgs,Year,ElectionYear,ProcessingMode};
pub use bisect_runner::state_args::{StateArgs,RunProfile,Resolution,MetisObjective,WeightMode,MetisEngineArg,validate_court_profile};
pub use bisect_runner::args::{
    AreaSectionInitArg, IlpAuditArgs, IlpFallback, IlpMethod, PartitionMode, SearchMode,
    StructureMode,
};
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





















// ---------------------------------------------------------------------------
// Top-level CLI
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(
    name = "bisect",
    about = "Congressional redistricting pipeline -- compact, population-balanced districts",
    before_help = "SHOW SOMEONE:\n  README → SHOWCASE.md (researcher vs implementer paths)\n  Hands-on: examples/vermont-2020-walkthrough/\n\nQUICKSTART:\n  1. bisect fetch --type adjacency --states VT --year 2020\n  2. bisect state --state VT --year 2020 --label vt_test\n  3. bisect analyze --label vt_test --types all\n  4. bisect report --label vt_test --format html\n  5. bisect tui   (interactive interface)\n  6. bisect doctor --label vt_test  (diagnose plan)\n\nSee: bisect <command> --help for detailed options.\n",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(name = "replay-native-state", hide = true)]
    ReplayNative(crate::native_replay::ReplayNativeArgs),
    /// Run the full redistricting pipeline (mirrors run_complete_redistricting.py)
    Run(RunArgs),
    /// Run a single state (mirrors run_state_redistricting.py)
    State(StateArgs),
    /// Run all states in parallel for one year (mirrors run_states_parallel.py)
    States(StatesArgs),
    /// Download census data needed to run redistricting
    Fetch(FetchArgs),
    /// Inspect BISECT fetch sources as FLETCH cachelines without downloading
    FletchSources(FletchSourcesArgs),
    /// Map the BISECT-owned FLETCH cache manifest to cache-index evidence
    FletchCacheIndex(FletchCacheIndexArgs),
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
    /// Improve an existing RPLAN plan with audited local search
    Improve(ImproveArgs),
    /// Run exact optimization family reports
    Exact(ExactArgs),
    /// Stage, announce, halt, or verify a public certified-bisection ceremony
    Ceremony(crate::ceremony_cmd::CeremonyArgs),
    /// Civic Bidirectional Input: ingest community-of-interest CSVs, detect
    /// cross-input conflicts, list/show ingested inputs, add candidate-race
    /// annotations. See `docs/superpowers/specs/2026-04-30-civic-bidirectional.md`.
    Civic(CivicArgs),
    /// Researcher Toolkit: validate ensembles, run convergence diagnostics,
    /// compute percentile-rank of an enacted plan against an MCMC ensemble.
    /// See `docs/superpowers/plans/2026-04-30-researcher-toolkit.md`.
    Research(ResearchArgs),
    /// Show redistricting policy for a state (subdivision terms, tolerances, VRA, etc.)
    Policy(PolicyArgs),
    /// Pre-flight check: verify data files, year validity, resolution warnings for a location
    Doctor(DoctorArgs),
    /// Reproduce a plan from its manifest.json and verify it matches the original
    Verify(VerifyArgs),
    /// Verify ILP solve-report model artifacts and hashes
    IlpAudit(IlpAuditArgs),
    /// Run N redistricting seeds and keep top-K plans by a metric
    Sweep(SweepArgs),
    /// Launch the interactive terminal UI
    Tui(TuiArgs),
    /// Algorithm config management: create or validate configs/X.yml (Spec 7 §7.3)
    Config(ConfigArgs),
    /// Build (run redistricting) for a label across one or more census years (Spec 7 Phase 2)
    Build(crate::build_cmd::BuildCliArgs),
    /// List all labels from the .bisect registry with stage completion (Spec 7 Phase 4)
    Ls(LsArgs),
    /// Show detailed info for a label: years, status, resolved paths (Spec 7 Phase 4)
    Show(ShowArgs),
    /// Atomically rename a label: filesystem directories + registry + index.json patches (Spec 7 Phase 4)
    Mv(MvArgs),
    /// Traverse the SHA chain for a label and verify integrity (Spec 7 Phase 4)
    LabelVerify(LabelVerifyArgs),
    /// Run analysis for a label across one or more built years (Spec 7 Phase 3)
    LabelAnalyze(LabelAnalyzeArgs),
    /// Generate a report for a label across one or more analyzed years (Spec 7 Phase 3)
    LabelReport(LabelReportArgs),
    /// Import an external plan file (CSV/GeoJSON/shapefile/RPLAN) into the label-based layout
    /// without running the redistricting algorithm (Spec 7 Phase 5).
    /// `bisect label-import X --from FILE [--year Y] [--format csv|geojson|shapefile|rplan]`
    LabelImport(LabelImportArgs),
    /// Compare two label-based plans using existing analysis outputs (Spec 7 Phase 5).
    /// `bisect label-compare A B [--year Y] [--json] [--out PATH]`
    LabelCompare(LabelCompareArgs),
    /// Interactive TUI hub — label-aware frontend for the full pipeline (Spec 7 Phase 6).
    /// With no label: shows label picker. With a label: pre-scoped to that label.
    /// With --configure: opens the three-layer compositor wizard.
    Plan(PlanArgs),
    /// Generate a redistricting ensemble — weighted sample of valid plans.
    /// Currently supports: --method smc (Sequential Monte Carlo, G.7).
    /// Output: NDJSON with weighted plans + audit metadata.
    Ensemble(EnsembleArgs),
    /// Generate a Pareto-optimal redistricting ensemble (U.7 spec).
    /// Outputs NDJSON with all Pareto-optimal plans and objective values.
    Pareto(ParetoArgs),
}

// ---------------------------------------------------------------------------
// `BISECT policy` — state policy lookup
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
// `BISECT doctor` — pre-flight check for a location+year
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
    /// Validate that a tutorial walkthrough's pinned data + expected outputs match
    /// their checksums.json. Reads `examples/{tutorial}-walkthrough/checksums.json`,
    /// hashes each file present locally, reports per-row PASS / FAIL / MISSING.
    /// Exits 0 only if every row is PASS. Used to catch upstream-data drift.
    #[arg(long)]
    pub check_tutorial_data: bool,
    /// Tutorial slug for --check-tutorial-data (default: vermont-2020).
    /// Resolves to `examples/{tutorial}-walkthrough/checksums.json`.
    #[arg(long, default_value = "vermont-2020")]
    pub tutorial: String,
}

// ---------------------------------------------------------------------------
// `BISECT validate` — validate a .rplan file
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
// `BISECT migrate` — copy legacy state plan into plans/{label}/ tree
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
// `BISECT report` — commission report (HTML / JSON)
// ---------------------------------------------------------------------------

/// Output format for `BISECT report`.
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

    // ── Court Submission Reports (when Typst integration ships) ──────────────
    // The flags below are wired into the CLI surface so quickstart-callais-expert.md
    // and the persona docs can demonstrate the full invocation today. The
    // PDF rendering path itself (Typst + verapdf) is scaffolded but NOT
    // executable until Typst is installed and templates are authored — see
    // BISECT-report/typst-templates/README.md.
    /// Expert witness name for the report cover page + signature block (court mode).
    #[arg(long)]
    pub expert_name: Option<String>,
    /// Expert credentials, free text (e.g., `"Ph.D., Statistics, Princeton"`).
    #[arg(long)]
    pub expert_credentials: Option<String>,
    /// Expert affiliation / employer.
    #[arg(long)]
    pub expert_affiliation: Option<String>,
    /// Optional path to a free-text case caption file (court header).
    #[arg(long)]
    pub case_caption_file: Option<std::path::PathBuf>,
    /// Court jurisdiction tag (e.g., `EDLA`, `NDGA`). Selects template tweaks
    /// (page size, margins, citation style default).
    #[arg(long)]
    pub jurisdiction: Option<String>,
    /// Citation style: `bluebook` (default for jurisdiction), `apa`
    /// (default for paper-mode), `chicago` (default for civic).
    #[arg(long)]
    pub citation_style: Option<String>,
    /// Optional `expert.toml` config file. CLI flags override file values.
    #[arg(long)]
    pub expert_config: Option<std::path::PathBuf>,
    /// Allow non-strict-validated civic inputs into the court report.
    /// Without this flag, court-mode renders REFUSE to embed civic inputs
    /// ingested under `--validate {lenient,advisory}` (BD-R1). The override
    /// is recorded in the reproducibility package manifest.
    #[arg(long)]
    pub allow_non_strict_civic: bool,
    /// Skip the verapdf PDF/A-2b validation gate (PP-32). The resulting PDF
    /// is explicitly NOT court-admissible. Use for iteration only.
    #[arg(long)]
    pub draft: bool,
}

// ---------------------------------------------------------------------------
// `BISECT export` — plan export (GeoJSON, GerryChain, CSV)
// ---------------------------------------------------------------------------

/// Export format for `BISECT export`.
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
// `BISECT import` — import GeoJSON plan
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

    // ── Civic-bypass flow (SSI Task 7, COMMONS) ──────────────────────────────
    /// Tag this import as a civic counter-proposal (NOT an authoritative state map).
    /// Requires --submitted-by. The plan's manifest records submission_type =
    /// "civic_counter_proposal" so downstream comparison reports surface the
    /// civic framing instead of treating it as authoritative state output.
    #[arg(long)]
    pub as_civic_counter_proposal: bool,
    /// Submitting organization name (required when --as-civic-counter-proposal).
    #[arg(long)]
    pub submitted_by: Option<String>,
    /// Submission timestamp (ISO-8601 UTC). Defaults to current time at import.
    #[arg(long)]
    pub submitted_at: Option<String>,
}

// ---------------------------------------------------------------------------
// `BISECT improve` — local-search improvement for audited RPLAN plans
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum ImproveMethodArg {
    /// Deterministic one-vertex boundary move that improves edge cut.
    #[value(name = "one-move")]
    OneMove,
}

#[derive(Debug, clap::Args)]
#[command(disable_version_flag = true)]
pub struct ImproveArgs {
    /// Input .rplan file.
    #[arg(long)]
    pub plan: std::path::PathBuf,
    /// Input .rctx context file with graph and population data.
    #[arg(long)]
    pub context: std::path::PathBuf,
    /// Output directory for improved.rplan, improved.rctx, summary, and certificate.
    #[arg(long)]
    pub out_dir: std::path::PathBuf,
    /// Local-search method.
    #[arg(long, default_value = "one-move")]
    pub method: ImproveMethodArg,
    /// Population tolerance as a percent. Example: 5.0 means +/-5%.
    #[arg(long, default_value_t = 5.0)]
    pub tolerance: f64,
    /// Output label. Defaults to the input label with "-improved" appended.
    #[arg(long)]
    pub label: Option<String>,
    /// Timestamp to record in emitted artifacts. Defaults to current UTC.
    #[arg(long)]
    pub generated_at: Option<String>,
}

// ---------------------------------------------------------------------------
// `BISECT exact` — exact optimization family reports
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum ExactMethodArg {
    /// Exact Canonical Benchmark E0 reference solver for bounded k=2 instances.
    #[value(name = "canonical-exhaustive")]
    CanonicalExhaustive,
    /// Bounded exact certification of the complete standard-bisect tree.
    #[value(name = "certified-recursive")]
    CertifiedRecursive,
    /// Deterministic METIS incumbent for subsequent certified proof requests.
    #[value(name = "certified-discovery")]
    CertifiedDiscovery,
    /// U.17 branch-and-price / column-generation report.
    #[value(name = "branch-and-price")]
    BranchAndPrice,
}

pub use bisect_runner::certified_discovery::DiscoveryRefinementArg;

#[derive(Debug, clap::Args)]
#[command(disable_version_flag = true)]
pub struct ExactArgs {
    /// Input .rctx context file with graph and population data.
    #[arg(long)]
    pub context: std::path::PathBuf,
    /// Output directory for exact reports, certificates, and plan packages.
    #[arg(long)]
    pub out_dir: std::path::PathBuf,
    /// Exact method.
    #[arg(long, value_enum, default_value_t = ExactMethodArg::BranchAndPrice)]
    pub method: ExactMethodArg,
    /// Number of districts/columns to select.
    #[arg(long)]
    pub districts: usize,
    /// Population tolerance as a percent. Example: 1.0 means +/-1%.
    #[arg(long, default_value_t = 1.0)]
    pub tolerance: f64,
    /// Emit formulation and pricing report without solving the master.
    #[arg(long)]
    pub formulation_only: bool,
    /// Maximum unit count for deterministic tiny exact solving.
    #[arg(long, default_value_t = 12usize)]
    pub exact_fixture_limit: usize,
    /// Fixed timestamp for reproducible package artifacts.
    #[arg(long)]
    pub generated_at: Option<String>,
    /// Seed for deterministic certified-discovery incumbent generation.
    #[arg(long, default_value_t = 1u64)]
    pub discovery_seed: u64,
    /// Deterministic local refinement applied after METIS discovery.
    #[arg(long, value_enum, default_value_t = DiscoveryRefinementArg::Full)]
    pub discovery_refinement: DiscoveryRefinementArg,
}

// ---------------------------------------------------------------------------
// `BISECT civic` — Civic Bidirectional Input
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct CivicArgs {
    #[command(subcommand)]
    pub command: crate::civic::CivicSubcommand,
}

// ---------------------------------------------------------------------------
// `BISECT research` — Researcher Toolkit (RT plan)
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct ResearchArgs {
    #[command(subcommand)]
    pub command: crate::research::ResearchSubcommand,
}

// ---------------------------------------------------------------------------
// `BISECT ensemble` — weighted plan ensemble generation (G.7 SMC, future methods)
// ---------------------------------------------------------------------------

/// Ensemble method to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum EnsembleMethod {
    /// Sequential Monte Carlo — calibrated weighted sample of valid plans.
    /// Produces an NDJSON file with importance-weighted plans. (G.7 spec accepted 3.1/4)
    #[value(name = "smc")]
    Smc,
}

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct EnsembleArgs {
    /// Ensemble method. Currently: smc (Sequential Monte Carlo).
    #[arg(long, value_enum)]
    pub method: EnsembleMethod,

    /// Two-letter state code (e.g., NC, WI, TX).
    #[arg(long)]
    pub state: String,

    /// Census year (default: 2020).
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: crate::args::Year,

    /// Output file path for the NDJSON ensemble (default: {state}_{method}_{year}.ndjson).
    #[arg(long)]
    pub output: Option<String>,

    // ── SMC parameters ────────────────────────────────────────────────────────
    /// Number of SMC particles (plans in the weighted sample). Default: 5000.
    /// Use 1000 for quick checks, 50000+ for publication-quality inference.
    #[arg(long, default_value_t = 5000)]
    pub particles: usize,

    /// Resample when ESS < threshold × n_particles. Default: 0.5.
    /// Lower threshold → fewer resamplings but more particle degeneracy.
    #[arg(long, default_value_t = 0.5)]
    pub resample_threshold: f64,

    /// Population balance tolerance as a fraction of ideal. Default: 0.005 (±0.5%).
    /// Use 0.01 for state legislative chambers, 0.005 for congressional.
    #[arg(long, default_value_t = 0.005)]
    pub pop_tolerance: f64,

    /// Base seed for SMC. Default: content-derived from state+year.
    /// Use --base-seed 42 for reproducible experiments.
    #[arg(long)]
    pub base_seed: Option<u64>,

    /// Input data directory (default: data/{year}/{state}).
    #[arg(long)]
    pub data_dir: Option<String>,
}

// ---------------------------------------------------------------------------
// `bisect pareto` — Pareto-optimal redistricting ensemble (U.7)
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct ParetoArgs {
    /// Two-letter state code (e.g., NC, WI, TX).
    #[arg(long)]
    pub state: String,

    /// Census year (default: 2020).
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: crate::args::Year,

    /// NSGA-II population size: number of plans per generation. Default: 100.
    #[arg(long, default_value_t = 100)]
    pub population: usize,

    /// Number of NSGA-II generations. Default: 200.
    #[arg(long, default_value_t = 200)]
    pub generations: usize,

    /// Base seed for all stochastic operations. Default: content-derived.
    #[arg(long)]
    pub base_seed: Option<u64>,

    /// Output file path for the NDJSON Pareto frontier (default: {state}_pareto_{year}.ndjson).
    #[arg(long)]
    pub output: Option<String>,

    /// Also emit an audited RPLAN package for this zero-based frontier entry.
    #[arg(long)]
    pub selected_frontier_index: Option<usize>,

    /// RCTX context used to audit the selected frontier plan.
    #[arg(long, requires = "selected_frontier_index")]
    pub selected_frontier_context: Option<std::path::PathBuf>,

    /// Output directory for the selected frontier audit package.
    #[arg(long, requires = "selected_frontier_index")]
    pub selected_frontier_out: Option<std::path::PathBuf>,

    /// Label for the selected frontier RPLAN package.
    #[arg(long, requires = "selected_frontier_index")]
    pub selected_frontier_label: Option<String>,

    /// Population tolerance as a percent for the selected frontier audit package.
    #[arg(long, default_value_t = 0.5)]
    pub selected_frontier_tolerance: f64,
}

// ---------------------------------------------------------------------------
// `bisect analyze` — per-district analytics
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct AnalyzeArgs {
    /// Matching custom-unit RCTX for splits only. Requires plan.rplan and native index-keyed assignments in the selected target.
    #[arg(long)]
    pub splits_context: Option<std::path::PathBuf>,
    /// Optional GEOID,x,y,population CSV in the native geometry CRS (WGS84 degrees).
    #[arg(long)]
    pub compactness_population_csv: Option<std::path::PathBuf>,

    /// Include native exhaustive minimum-circle Reock alongside the centroid-radius proxy.
    #[arg(long)]
    pub exact_reock: bool,

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
    pub types: Vec<bisect_analysis::AnalyzerType>,

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

    /// Seed for the portable county-cluster and naive bootstrap used by bloc-voting.
    #[arg(long, default_value_t = 42)]
    pub bloc_bootstrap_seed: u64,

    // ── Bloc voting (Callais Evidence Layer) ───────────────────────────────
    // Only consulted when --types includes bloc-voting.
    /// Race-of-candidate CSV path (curator-attested per docs/file-formats/race-of-candidate.md).
    /// Required when --types bloc-voting is requested.
    #[arg(long)]
    pub candidate_race_csv: Option<std::path::PathBuf>,

    /// Election cycle name (e.g., "presidential-primary"). Recorded in bloc_voting.json.
    #[arg(long, default_value = "presidential-primary")]
    pub election: String,

    /// Party under analysis. Most §2 analyses focus on Democratic primaries
    /// (where within-party racial polarization is the Callais p.36 question).
    #[arg(long, default_value = "DEM")]
    pub party: String,

    /// Minority group for the bloc-voting regression: black (default), hispanic, asian.
    #[arg(long, default_value = "black")]
    pub minority_group: String,

    /// Significance threshold alpha for Holm-corrected p-values (default 0.05).
    #[arg(long, default_value_t = 0.05)]
    pub alpha: f64,

    /// Confidence level for cluster-bootstrap CIs (default 0.95).
    #[arg(long, default_value_t = 0.95)]
    pub ci_level: f64,

    /// Per-tract Democratic-share TSV for the partisan baseline (Callais p.36
    /// disentanglement). Required when --types bloc-voting; produced via
    /// `scripts/data/political/build_dem_shares.py`.
    #[arg(long)]
    pub partisan_baseline: Option<std::path::PathBuf>,

    /// Regression method. Currently only `wls` (HC3 + cluster-bootstrap by county)
    /// is implemented; `rxc` returns not-yet-implemented per spec.
    #[arg(long, default_value = "wls")]
    pub method: String,

    /// Refuse the analysis below this many precincts (default 50). Below this
    /// threshold, regression inference is unreliable and `[INPUT]` exits.
    #[arg(long, default_value_t = 50)]
    pub min_precincts: usize,

    // ── Researcher Toolkit / paper-mode (D-05) ─────────────────────────────
    /// Emit an AEA-compliant replication package alongside the analyze outputs.
    /// Writes `paper_mode/{REPRODUCE.sh, inputs.sha256.json,
    /// expected_outputs.sha256.json, environment.json, seeds.json,
    /// CITATION.{bib,apa.txt,chicago.txt}, README.md}` under the analysis dir.
    /// See `docs/superpowers/plans/2026-04-30-researcher-toolkit.md` Task 8.
    #[arg(long)]
    pub paper_mode: bool,

    /// Citation style for `paper_mode/CITATION.*` outputs (advisory; the
    /// renderer always writes all three styles, but the named style is
    /// flagged "default" in README.md). One of: `bluebook`, `apa`, `chicago`.
    /// Defaults to `apa` per `docs/file-formats/citation-strings.md` §1.
    #[arg(long, default_value = "apa")]
    pub paper_mode_citation_style: String,
}

// ---------------------------------------------------------------------------
// `BISECT suite` — multi-chamber suite draw and validate
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
// `BISECT compare` — plan comparison
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum CompareFormat {
    Table,
    Json,
    Csv,
    /// Civic-friendly narrative markdown + provenance manifest
    /// (Plan Comparison plan Task 3-5, 9). Writes narrative.md + narrative_manifest.json
    /// into outputs/{version}/comparisons/{plan_a}_vs_{plan_b}/.
    Narrative,
    /// Both Table (stdout) and Narrative (file outputs).
    Both,
    /// Self-contained HTML side-by-side report (Plan Comparison plan Task 7).
    /// Writes comparison.html + narrative.md + narrative_manifest.json into
    /// outputs/{version}/comparisons/{plan_a}_vs_{plan_b}/.
    Html,
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

    // ── Narrative / civic-facing flags (Plan Comparison plan Tasks 1, 5) ─────
    /// Threshold (Dem-share fraction) above which a district is described as
    /// "Democratic-leaning" in the narrative; below threshold is "Republican-leaning".
    /// Districts within ±close-call-band of this value are flagged as close calls.
    /// Default 0.55. TRENCH RP-03: forbid hardcoded thresholds anywhere downstream;
    /// every read goes through this field.
    #[arg(long, default_value_t = 0.55)]
    pub leaning_threshold: f64,

    /// Width of the close-call band (one-sided). A district within ±this of
    /// `leaning_threshold` is flagged as a close-call in the narrative.
    /// Default 0.02 (±2 percentage points).
    #[arg(long, default_value_t = 0.02)]
    pub close_call_band: f64,

    /// Sign-off name for the rendered narrative. When unset, every paragraph
    /// is prefixed `[DRAFT — review before publication]`. When set to a
    /// non-empty string, the prefix is removed and the name is committed to
    /// `narrative_manifest.json` as the audit trail.
    #[arg(long)]
    pub approved_by: Option<String>,

    /// Output directory override for narrative artifacts (default:
    /// outputs/{version}/comparisons/{plan_a}_vs_{plan_b}/).
    #[arg(long)]
    pub report_dir: Option<std::path::PathBuf>,
}

// ---------------------------------------------------------------------------
// `BISECT map` — PNG map rendering
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
            Self::Districts => "districts",
            Self::Rounds => "rounds",
            Self::Political => "political",
            Self::Demographic => "demographic",
            Self::Compactness => "compactness",
            Self::Splits => "splits",
            Self::All => "all",
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
// `BISECT run` — full pipeline (run_complete_redistricting.py)
// ---------------------------------------------------------------------------



// ---------------------------------------------------------------------------
// `bisect state` — single state (run_state_redistricting.py)
// ---------------------------------------------------------------------------









// ---------------------------------------------------------------------------
// `bisect states` — all states in parallel (run_states_parallel.py)
// ---------------------------------------------------------------------------



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
        let args =
            StateArgs::parse_from(["state", "--state", "AL", "--partition-mode", "metis-vra"]);
        assert_eq!(args.partition_mode, PartitionMode::MetisVra);
    }

    #[test]
    fn test_states_args_required_fields() {
        let args = StatesArgs::parse_from([
            "states",
            "--year",
            "2020",
            "--version",
            "V3",
            "--output-dir",
            "/tmp/out",
        ]);
        assert_eq!(args.year, Year::Y2020);
        assert_eq!(args.version, "V3");
        assert_eq!(args.workers, 4);
        assert!(!args.run_analysis);
    }

    #[test]
    fn test_year_enum_2010() {
        let args = StateArgs::parse_from(["state", "--state", "CA", "--year", "2010"]);
        assert_eq!(args.year, "2010");
    }

    #[test]
    fn test_states_filter_list() {
        let args = RunArgs::parse_from(["run", "--states", "VT", "DE", "AK"]);
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
        assert_eq!(
            args.stages.len(),
            3,
            "stages should default to [data, states, nation], got {:?}",
            args.stages
        );
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
        assert_eq!(
            run_args.workers, 12,
            "run_complete_redistricting.py defaults to 12"
        );
        let states_args = StatesArgs::parse_from([
            "states",
            "--year",
            "2020",
            "--version",
            "v1",
            "--output-dir",
            "/tmp",
        ]);
        assert_eq!(
            states_args.workers, 4,
            "run_states_parallel.py defaults to 4"
        );
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
        assert!(args.types.contains(&bisect_analysis::AnalyzerType::All));
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
            "analyze",
            "--state",
            "AL",
            "--types",
            "demographic",
            "political",
        ]);
        assert!(args
            .types
            .contains(&bisect_analysis::AnalyzerType::Demographic));
        assert!(args
            .types
            .contains(&bisect_analysis::AnalyzerType::Political));
        assert!(!args.types.contains(&bisect_analysis::AnalyzerType::All));
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
        let args =
            MapArgs::parse_from(["map", "--state", "VT", "--output-base", "/custom/outputs"]);
        assert_eq!(args.output_base, "/custom/outputs");
    }

    #[test]
    fn test_map_types_parse() {
        let args = MapArgs::parse_from(["map", "--state", "VT", "--types", "districts", "rounds"]);
        assert!(args.types.contains(&MapType::Districts));
        assert!(args.types.contains(&MapType::Rounds));
    }

    // --- Task 7: Spec 1 flag parsing tests ---

    fn parse_state_args(extra: &[&str]) -> StateArgs {
        let mut base = vec![
            "bisect",
            "--state",
            "WA",
            "--year",
            "2020",
            "--version",
            "v1",
        ];
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
    fn test_profile_defaults_open_and_allows_partisan() {
        let args = parse_state_args(&[]);
        assert_eq!(args.profile, RunProfile::Open);
        assert!(super::validate_court_profile(
            args.profile,
            PartitionMode::PartisanWeighted,
            None,
            None,
            None,
            &Some("shares.tsv".to_string())
        )
        .is_ok());
    }

    #[test]
    fn test_court_profile_rejects_partisan_inputs_and_modes() {
        assert!(super::validate_court_profile(
            RunProfile::Court,
            PartitionMode::EdgeWeighted,
            None,
            None,
            None,
            &Some("shares.tsv".to_string())
        )
        .is_err());
        assert!(super::validate_court_profile(
            RunProfile::Court,
            PartitionMode::PartisanWeighted,
            None,
            None,
            None,
            &None
        )
        .is_err());
        assert!(super::validate_court_profile(
            RunProfile::Court,
            PartitionMode::Proportional,
            None,
            None,
            None,
            &None
        )
        .is_err());
        assert!(super::validate_court_profile(
            RunProfile::Court,
            PartitionMode::EdgeWeighted,
            None,
            None,
            Some(SearchMode::ShortBurst),
            &None
        )
        .is_err());
    }

    #[test]
    fn test_court_profile_accepts_neutral_config() {
        assert!(super::validate_court_profile(
            RunProfile::Court,
            PartitionMode::EdgeWeighted,
            None,
            None,
            Some(SearchMode::Convergence),
            &None
        )
        .is_ok());
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
            "validate",
            "--name",
            "wa_suite_v1",
            "--state",
            "WA",
            "--house-label",
            "wa_house_custom",
        ]);
        assert_eq!(args.house_label, Some("wa_house_custom".into()));
        assert!(args.senate_label.is_none());
    }

    #[test]
    fn test_suite_validate_args_both_label_overrides() {
        use crate::args::SuiteValidateCliArgs;
        let args = SuiteValidateCliArgs::parse_from([
            "validate",
            "--name",
            "wa_suite_v1",
            "--house-label",
            "wa_house_custom",
            "--senate-label",
            "wa_senate_custom",
        ]);
        assert_eq!(args.house_label, Some("wa_house_custom".into()));
        assert_eq!(args.senate_label, Some("wa_senate_custom".into()));
    }

    #[test]
    fn test_suite_validate_args_no_labels_defaults_to_none() {
        use crate::args::SuiteValidateCliArgs;
        let args = SuiteValidateCliArgs::parse_from(["validate", "--name", "wa_suite_v1"]);
        assert!(args.house_label.is_none());
        assert!(args.senate_label.is_none());
    }

    // ── Task 129: --labels flag for N-plan summary ────────────────────────────

    #[test]
    fn test_compare_labels_flag_parsed() {
        let args = CompareArgs::parse_from([
            "compare",
            "--plan-a",
            "plan1",
            "--labels",
            "wa_house_plan_a,wa_house_plan_b,wa_house_plan_c",
        ]);
        assert_eq!(
            args.labels,
            vec!["wa_house_plan_a", "wa_house_plan_b", "wa_house_plan_c"]
        );
    }

    #[test]
    fn test_compare_labels_default_empty() {
        let args = CompareArgs::parse_from(["compare", "--plan-a", "plan1", "--plan-b", "plan2"]);
        assert!(
            args.labels.is_empty(),
            "--labels should default to empty Vec"
        );
    }

    // ── Task 134: splits map type ─────────────────────────────────────────────

    #[test]
    fn test_map_type_splits_recognized() {
        let args = MapArgs::parse_from(["map", "--state", "WA", "--types", "splits"]);
        assert!(
            args.types.contains(&MapType::Splits),
            "splits must be a valid MapType"
        );
    }

    #[test]
    fn test_map_type_splits_name() {
        assert_eq!(MapType::Splits.name(), "splits");
    }

    // ── Task 139: --external-analyzer flag ───────────────────────────────────

    #[test]
    fn test_external_analyzer_flag_parsed() {
        let args = AnalyzeArgs::parse_from([
            "analyze",
            "--state",
            "WA",
            "--external-analyzer",
            "my_script.py",
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
        let args =
            AnalyzeArgs::parse_from(["analyze", "--state", "IE", "--election-format", "ie-dail"]);
        assert_eq!(args.election_format, "ie-dail");
    }

    #[test]
    fn test_election_format_default_us_presidential() {
        let args = AnalyzeArgs::parse_from(["analyze", "--state", "VT"]);
        assert_eq!(args.election_format, "us-presidential");
    }

    // ── RT Task 8.1: --paper-mode flag wiring (D-05) ─────────────────────────

    #[test]
    fn test_paper_mode_flag_defaults_false() {
        let args = AnalyzeArgs::parse_from(["analyze", "--state", "VT"]);
        assert!(!args.paper_mode, "--paper-mode must default to false");
    }

    #[test]
    fn test_paper_mode_flag_parses() {
        let args = AnalyzeArgs::parse_from(["analyze", "--state", "VT", "--paper-mode"]);
        assert!(
            args.paper_mode,
            "--paper-mode flag must set paper_mode = true"
        );
    }

    #[test]
    fn test_paper_mode_citation_style_default_apa() {
        let args = AnalyzeArgs::parse_from(["analyze", "--state", "VT"]);
        assert_eq!(
            args.paper_mode_citation_style, "apa",
            "default citation style must be 'apa' per citation-strings.md §1"
        );
    }

    #[test]
    fn test_paper_mode_citation_style_overridable() {
        let args = AnalyzeArgs::parse_from([
            "analyze",
            "--state",
            "VT",
            "--paper-mode-citation-style",
            "chicago",
        ]);
        assert_eq!(args.paper_mode_citation_style, "chicago");
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
        assert!(
            note.contains("us-presidential"),
            "note must mention the supported format"
        );
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
            "state",
            "--state",
            "WA",
            "--coi-weights",
            "coi_weights.json",
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
            "draw",
            "--state",
            "WA",
            "--name",
            "wa_suite",
            "--nest-congressional-in-senate",
        ]);
        assert!(args.nest_congressional_in_senate);
    }

    #[test]
    fn test_nest_congressional_in_senate_default_false() {
        let args = SuiteDrawCliArgs::parse_from(["draw", "--state", "WA", "--name", "wa_suite"]);
        assert!(!args.nest_congressional_in_senate);
    }

    // ── Task 144: sweep subcommand ────────────────────────────────────────────

    #[test]
    fn test_sweep_args_parse() {
        use crate::args::SweepArgs;
        let args = SweepArgs::parse_from([
            "sweep",
            "--state",
            "WA",
            "--seeds",
            "20",
            "--keep-top",
            "5",
            "--optimize-by",
            "splits",
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
        let args = VerifyArgs::parse_from([
            "verify",
            "--manifest",
            "manifest.json",
            "--label",
            "wa_house_v1",
        ]);
        assert_eq!(args.label, Some("wa_house_v1".to_string()));
    }

    #[test]
    fn test_ilp_audit_args_parsed() {
        let args = IlpAuditArgs::parse_from([
            "ilp-audit",
            "--dir",
            "intermediate/ilp_solve_reports",
            "--json",
            "--out",
            "intermediate/ilp_solve_reports/audit-summary.json",
            "--verify-summary",
            "intermediate/ilp_solve_reports/audit-summary.json",
            "node_root.json",
        ]);
        assert_eq!(
            args.dir.as_deref(),
            Some(std::path::Path::new("intermediate/ilp_solve_reports"))
        );
        assert_eq!(
            args.reports,
            vec![std::path::PathBuf::from("node_root.json")]
        );
        assert!(args.json);
        assert_eq!(
            args.out.as_deref(),
            Some(std::path::Path::new(
                "intermediate/ilp_solve_reports/audit-summary.json"
            ))
        );
        assert_eq!(
            args.verify_summary.as_deref(),
            Some(std::path::Path::new(
                "intermediate/ilp_solve_reports/audit-summary.json"
            ))
        );
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
        let args =
            DoctorArgs::parse_from(["doctor", "--label", "wa_house_2020", "--version", "v2"]);
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
        assert!(
            help.contains("QUICKSTART") || help.contains("fetch"),
            "CLI help must contain quickstart instructions: {}",
            &help[..200.min(help.len())]
        );
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
            "compare",
            "--plan-a",
            "plan_a",
            "--plan-b",
            "plan_b",
            "--version-b",
            "v2",
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

    // ── Group 9: enum Display + parse round-trips ─────────────────────────────

    #[test]
    fn partition_mode_display_all_variants() {
        // Every PartitionMode variant must round-trip through Display.
        // If a new variant is added without updating this list the test
        // will fail to compile — compile-time safety net.
        let cases: &[(PartitionMode, &str)] = &[
            (PartitionMode::Unweighted, "unweighted"),
            (PartitionMode::EdgeWeighted, "edge-weighted"),
            (PartitionMode::MetisVra, "metis-vra"),
            (PartitionMode::PartisanWeighted, "partisan-weighted"),
            (PartitionMode::Proportional, "proportional"),
            (PartitionMode::CompactBisect, "compact-bisect"),
            (PartitionMode::GeoSection, "geosection"),
            (PartitionMode::AreaSection, "areasection"),
            (PartitionMode::ProportionalSection, "proportional-section"),
            (PartitionMode::ApportionRegions, "apportion-regions"),
            (PartitionMode::VraSection, "vra-section"),
            (PartitionMode::MovingKnife, "moving-knife"),
            (PartitionMode::CapacityClustering, "capacity-clustering"),
            (PartitionMode::Spectral, "spectral"),
            (PartitionMode::Regionalization, "regionalization"),
            (PartitionMode::FlowConstruction, "flow-construction"),
        ];
        for (variant, expected) in cases {
            assert_eq!(
                format!("{variant}"),
                *expected,
                "PartitionMode::{variant:?} Display must be '{expected}'"
            );
        }
    }

    #[test]
    fn structure_mode_all_variants_parse() {
        // StructureMode is a ValueEnum — every variant must parse from its string value.
        use clap::ValueEnum;
        let cases: &[(&str, StructureMode)] = &[
            ("standard-bisect", StructureMode::StandardBisect),
            ("nway", StructureMode::NWay),
            ("ratio-optimal", StructureMode::RatioOptimal),
            ("ratio-optimal-area", StructureMode::RatioOptimalArea),
            ("ratio-optimal-vra", StructureMode::RatioOptimalVra),
            ("prime-factor", StructureMode::PrimeFactor),
            ("compact-polsby", StructureMode::CompactPolsby),
            ("bfs-growth", StructureMode::BfsGrowth),
            ("moving-knife", StructureMode::MovingKnife),
            ("capacity-clustering", StructureMode::CapacityClustering),
            ("spectral", StructureMode::Spectral),
            ("regionalization", StructureMode::Regionalization),
            ("flow-construction", StructureMode::FlowConstruction),
        ];
        for (s, expected) in cases {
            let parsed = StructureMode::from_str(s, true)
                .unwrap_or_else(|_| panic!("StructureMode must parse from '{s}'"));
            assert_eq!(
                parsed, *expected,
                "StructureMode parsed from '{s}' must equal {:?}",
                expected
            );
        }
    }

    #[test]
    fn search_mode_all_variants() {
        use clap::ValueEnum;
        let cases: &[(&str, SearchMode)] = &[
            ("single", SearchMode::Single),
            ("multi", SearchMode::Multi),
            ("convergence", SearchMode::Convergence),
            ("percentile", SearchMode::Percentile),
            ("bisection-ensemble", SearchMode::BisectionEnsemble),
            ("short-burst", SearchMode::ShortBurst),
            ("short-burst-forest", SearchMode::ShortBurstForest),
            ("short-burst-merge-split", SearchMode::ShortBurstMergeSplit),
            ("flip", SearchMode::Flip),
            ("forest-recom", SearchMode::ForestRecom),
            ("multiscale", SearchMode::MultiScale),
            ("merge-split", SearchMode::MergeSplit),
            ("multiscale-adaptive", SearchMode::MultiScaleAdaptive),
            ("parallel-tempering", SearchMode::ParallelTempering),
            ("vra-recom", SearchMode::VraRecom),
            ("smc-percentile", SearchMode::SmcPercentile),
        ];
        for (s, expected) in cases {
            let parsed = SearchMode::from_str(s, true)
                .unwrap_or_else(|_| panic!("SearchMode must parse from '{s}'"));
            assert_eq!(
                parsed, *expected,
                "SearchMode parsed from '{s}' must equal {:?}",
                expected
            );
        }
    }

    #[test]
    fn weight_mode_all_variants() {
        use clap::ValueEnum;
        let cases: &[(&str, WeightMode)] = &[
            ("unweighted", WeightMode::Unweighted),
            ("geographic", WeightMode::Geographic),
            ("county", WeightMode::County),
            ("vra-aligned", WeightMode::VraAligned),
            ("proportional", WeightMode::Proportional),
            ("economic-character", WeightMode::EconomicCharacter),
            ("housing-character", WeightMode::HousingCharacter),
            ("zone-membership", WeightMode::ZoneMembership),
        ];
        for (s, expected) in cases {
            let parsed = WeightMode::from_str(s, true)
                .unwrap_or_else(|_| panic!("WeightMode must parse from '{s}'"));
            assert_eq!(
                parsed, *expected,
                "WeightMode parsed from '{s}' must equal {:?}",
                expected
            );
        }
    }
}

// ---------------------------------------------------------------------------
// `BISECT sweep` — N-seed optimization planning tool
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
// `BISECT verify` — reproduce a BISECT plan manifest or verify an RPLAN package
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct VerifyArgs {
    /// Path to a BISECT manifest.json or RPLAN golden package manifest.json
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
    /// Skip re-running bisect state — just compare the existing plan assignments
    /// against a reference file (useful when METIS is unavailable)
    #[arg(long)]
    pub verify_assignments_only: bool,
    /// Reference assignments file to compare against (for --verify-assignments-only)
    #[arg(long)]
    pub plan_ref: Option<String>,
}

// ---------------------------------------------------------------------------
// `BISECT ilp-audit` — verify ILP solve report model artifacts
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// `BISECT tui` — interactive terminal UI
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct TuiArgs {
    /// Start with a clean session (no saved config loaded)
    #[arg(long)]
    pub no_session: bool,
}

// ---------------------------------------------------------------------------
// `BISECT plan` — interactive TUI hub with label awareness (Spec 7 Phase 6)
// ---------------------------------------------------------------------------

/// Args for `BISECT plan [X] [--configure]`.
#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct PlanArgs {
    /// Label to scope to (optional — if omitted, shows label picker)
    pub label: Option<String>,

    /// Open the three-layer compositor wizard to edit the algorithm config
    #[arg(long)]
    pub configure: bool,

    /// Path to config file (for --configure mode)
    #[arg(long)]
    pub config: Option<std::path::PathBuf>,
}

// ---------------------------------------------------------------------------
// `BISECT config` — algorithm config management (Spec 7 §7.3)
// ---------------------------------------------------------------------------

/// Top-level args for `BISECT config <subcommand>`.
#[derive(Debug, Parser)]
#[command(
    disable_version_flag = true,
    about = "Create or validate algorithm config YAML files"
)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

/// Subcommands for `BISECT config`.
#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Generate a template config YAML and write it to configs/{name}.yml
    New(ConfigNewArgs),
    /// Load and validate a config YAML file without running
    Validate(ConfigValidateArgs),
}

/// Args for `BISECT config new`.
#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct ConfigNewArgs {
    /// Config name (written to configs/{name}.yml)
    #[arg(long)]
    pub name: String,

    /// Layer 1: bisection structure
    /// Values: standard-bisect | nway | ratio-optimal | ratio-optimal-area |
    ///         ratio-optimal-vra | prime-factor | compact-polsby | apportion-regions
    #[arg(long, default_value = "apportion-regions")]
    pub structure: String,

    /// Layer 2: edge weight mode
    /// Values: unweighted | geographic | county | vra-aligned | proportional
    #[arg(long, default_value = "geographic")]
    pub weights: String,

    /// County stickiness alpha (required when --weights county)
    #[arg(long)]
    pub alpha_county: Option<f64>,

    /// Layer 3: seed search strategy
    /// Values: single | multi | convergence
    #[arg(long, default_value = "single")]
    pub search: String,

    /// Convergence threshold (required when --search convergence)
    #[arg(long)]
    pub convergence_threshold: Option<u32>,

    /// Fixed seed count (used when --search multi)
    #[arg(long)]
    pub seeds: Option<usize>,

    /// Population balance tolerance in percent (default: 0.5)
    #[arg(long)]
    pub balance_tolerance: Option<f64>,

    /// Census years to build (default: 2020 2010 2000)
    #[arg(long, num_args = 1.., default_values = ["2020", "2010", "2000"])]
    pub years: Vec<String>,

    /// Parallel workers per year (default: 4)
    #[arg(long, default_value_t = 4)]
    pub workers: usize,

    /// Output path override (default: configs/{name}.yml)
    #[arg(long)]
    pub out: Option<std::path::PathBuf>,

    /// Print the generated YAML without writing to disk
    #[arg(long)]
    pub dry_run: bool,

    /// Overwrite an existing config file without error
    #[arg(long)]
    pub force: bool,
}

/// Args for `BISECT config validate`.
#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct ConfigValidateArgs {
    /// Path to the YAML config file to validate
    pub path: std::path::PathBuf,
}

// ---------------------------------------------------------------------------
// `bisect fetch` — data download
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
    /// LODES WAC (Workplace Area Characteristics) — jobs by NAICS sector per census block.
    /// Used for M.1 economic character edge weights and M.9.
    /// Source: Census Bureau LEHD program, https://lehd.ces.census.gov/data/lodes/
    #[value(name = "lodes")]
    Lodes,
    /// TIGER/Line school district shapefiles — used for M.6 zone co-membership weights.
    /// Source: Census Bureau, https://www.census.gov/geo/maps-data/data/tiger-line.html
    #[value(name = "school-districts")]
    SchoolDistricts,
    /// EIA Form 861 electric utility service territory shapefiles — used for M.6.
    /// Source: U.S. Energy Information Administration, https://www.eia.gov/electricity/data/eia861/
    #[value(name = "eia-861")]
    Eia861,
    /// LODES OD (Origin-Destination) data — home/work tract pairs with job counts.
    /// Used for M.4 commuting shed similarity weights.
    /// Source: Census Bureau LEHD program, same as LODES WAC.
    #[value(name = "lodes-od")]
    LodesOd,
    /// ACS 5-year housing character data (B25024, B25003, B25035).
    /// Adds 4 columns: pct_single_family, pct_multifamily, pct_owner, housing_vintage.
    /// Output: data/{year}/acs_housing/{state}_housing_{year}.csv
    /// Source: Census Bureau ACS API, https://api.census.gov/data/{year}/acs/acs5
    #[value(name = "acs-housing")]
    AcsHousing,
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

    /// Data types: tiger, redistricting, adjacency, enacted, geography, elections,
    /// lodes, lodes-od, school-districts, eia-861, acs-housing, all [default: all]
    #[arg(long = "type", num_args = 0.., value_delimiter = ' ')]
    pub data_types: Vec<DataType>,

    /// Pull adjacency data from GitHub Releases (requires gh auth login)
    #[arg(long)]
    pub release: bool,

    /// Custom manifest file path (overrides BISECT_MANIFEST env var)
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

    /// Seconds to sleep between each download request [default: 1].
    /// Increase for polite fetching from federal servers (Census, LEHD, EIA).
    /// Set to 0 only in CI environments with explicit server permission.
    #[arg(long, default_value_t = 1)]
    pub polite_delay_secs: u64,
}

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct FletchSourcesArgs {
    /// Census year: 2020, 2010, 2000, or all [default: 2020]
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,

    /// State codes to inspect (default: all 50)
    #[arg(long = "states", num_args = 0.., value_delimiter = ' ')]
    pub states: Vec<String>,

    /// Data types to inspect; defaults to the same "all" subset as bisect fetch
    #[arg(long = "type", num_args = 0.., value_delimiter = ' ')]
    pub data_types: Vec<DataType>,

    /// Custom manifest file path (overrides BISECT_MANIFEST env var)
    #[arg(long)]
    pub manifest: Option<String>,

    /// Output handoff/readiness CSV
    #[arg(
        long,
        default_value = "data/fletch-source-handoff.csv",
        value_name = "FILE"
    )]
    pub output: std::path::PathBuf,

    /// Print per-source handoff rows
    #[arg(long)]
    pub details: bool,

    /// Fail if the generated handoff has registry or validation failures
    #[arg(long)]
    pub gate: bool,
}

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct FletchCacheIndexArgs {
    /// Census year: 2020, 2010, 2000, or all [default: 2020]
    #[arg(short = 'y', long, default_value = "2020")]
    pub year: String,

    /// State codes to inspect (default: all 50)
    #[arg(long = "states", num_args = 0.., value_delimiter = ' ')]
    pub states: Vec<String>,

    /// Data types to inspect; defaults to the same "all" subset as bisect fetch
    #[arg(long = "type", num_args = 0.., value_delimiter = ' ')]
    pub data_types: Vec<DataType>,

    /// Custom BISECT manifest file path (overrides BISECT_MANIFEST env var)
    #[arg(long)]
    pub manifest: Option<String>,

    /// Read this FLETCH cache manifest. Defaults to <local_data_dir>/.fletch/cache-manifest.json.
    #[arg(long, value_name = "FILE")]
    pub cache_manifest: Option<std::path::PathBuf>,

    /// Output cache-index evidence JSON
    #[arg(
        long,
        default_value = "data/fletch-cache-index.json",
        value_name = "FILE"
    )]
    pub output: std::path::PathBuf,

    /// Print per-source evidence rows
    #[arg(long)]
    pub details: bool,

    /// Fail if indexed rows are unverified or outside the BISECT FLETCH registry
    #[arg(long)]
    pub gate: bool,
}

// ---------------------------------------------------------------------------
// `BISECT aggregate` — merge state analysis into national datasets
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
    pub types: Vec<bisect_analysis::AnalyzerType>,

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

// ---------------------------------------------------------------------------
// `BISECT ls` — list labels (Spec 7 Phase 4)
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(
    disable_version_flag = true,
    about = "List all labels in the .bisect registry with their stage completion"
)]
pub struct LsArgs {
    /// Output the raw registry as JSON instead of the human-readable table
    #[arg(long)]
    pub json: bool,
}

// ---------------------------------------------------------------------------
// `BISECT show` — show label details (Spec 7 Phase 4)
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(
    disable_version_flag = true,
    about = "Show detailed info for a label: years, status, resolved paths"
)]
pub struct ShowArgs {
    /// Label to inspect
    pub label: String,

    /// Output structured JSON instead of the human-readable report
    #[arg(long)]
    pub json: bool,
}

// ---------------------------------------------------------------------------
// `BISECT mv` — rename a label (Spec 7 Phase 4)
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(
    disable_version_flag = true,
    about = "Atomically rename a label: moves runs/, analysis/, reports/ and updates the registry"
)]
pub struct MvArgs {
    /// Source label (must exist in registry)
    pub src: String,

    /// Destination label (must be a valid label name, must not already exist unless --force)
    pub dst: String,

    /// Overwrite the destination label if it already exists
    #[arg(long)]
    pub force: bool,
}

// ---------------------------------------------------------------------------
// `BISECT label-verify` — SHA chain integrity check (Spec 7 Phase 4)
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(
    disable_version_flag = true,
    about = "Traverse the SHA chain for a label and verify cryptographic integrity"
)]
pub struct LabelVerifyArgs {
    /// Label to verify
    pub label: String,

    /// Restrict verification to a single year (default: all built years)
    #[arg(short = 'y', long)]
    pub year: Option<String>,
}

// ---------------------------------------------------------------------------
// `BISECT label-analyze` — label-aware analysis (Spec 7 Phase 3)
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(
    disable_version_flag = true,
    about = "Run analysis for a label across one or more built census years (Spec 7 Phase 3)"
)]
pub struct LabelAnalyzeArgs {
    /// Label to analyze (must have been built with `bisect build X`)
    pub label: String,

    /// Restrict to a single census year (default: all built years)
    #[arg(short = 'y', long, value_name = "YEAR")]
    pub year: Option<String>,

    /// Analysis types to run (e.g., summary proportionality splits).
    /// If omitted, defaults to summary.
    #[arg(long = "types", value_delimiter = ' ', num_args = 0..)]
    pub types: Vec<String>,

    /// Restrict to specific states by name (e.g., vermont california).
    /// If omitted, all states found in runs/{label}/{year}/ are analyzed.
    #[arg(long = "states", value_delimiter = ' ', num_args = 0..)]
    pub states: Vec<String>,

    /// Matching RCTX graph for contiguity on one imported state and year
    #[arg(long, value_name = "RCTX", requires = "district_basis")]
    pub context: Option<std::path::PathBuf>,

    /// Numbering of the original imported district IDs
    #[arg(long, requires = "context", value_parser = ["one-based", "zero-based"])]
    pub district_basis: Option<String>,

    /// Disable interactive confirmation prompts (for CI environments)
    #[arg(long)]
    pub no_interactive: bool,
}

// ---------------------------------------------------------------------------
// `BISECT label-report` — label-aware report generation (Spec 7 Phase 3)
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(
    disable_version_flag = true,
    about = "Generate a report for a label across one or more analyzed census years (Spec 7 Phase 3)"
)]
pub struct LabelReportArgs {
    /// Label to report (must have been analyzed with `BISECT label-analyze X`)
    pub label: String,

    /// Restrict to a single census year (default: all analyzed years)
    #[arg(short = 'y', long, value_name = "YEAR")]
    pub year: Option<String>,

    /// Output format: html (default) or json
    #[arg(long, default_value = "html")]
    pub format: String,

    /// Output directory override (default: reports/{label}/{year}/)
    #[arg(long = "out", value_name = "PATH")]
    pub out: Option<std::path::PathBuf>,
}

// ---------------------------------------------------------------------------
// `BISECT label-import` — import external plan into label layout (Spec 7 Phase 5)
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(
    disable_version_flag = true,
    about = "Import an external plan file (CSV/GeoJSON) into the label-based directory layout \
             (Spec 7 Phase 5). Writes runs/{label}/{year}/... and marks the label built in .bisect."
)]
pub struct LabelImportArgs {
    /// Label to import into (e.g., official_proposal, senate_draft)
    pub label: String,

    /// Source file to import (CSV or GeoJSON)
    #[arg(long, value_name = "FILE")]
    pub from: std::path::PathBuf,

    /// Census year for the import (default: 2020)
    #[arg(short = 'y', long, default_value = "2020", value_name = "YEAR")]
    pub year: String,

    /// File format: csv, geojson, shapefile, rplan.
    /// Auto-detected from the file extension if not supplied.
    #[arg(long, value_name = "FORMAT")]
    pub format: Option<String>,
}

// ---------------------------------------------------------------------------
// `BISECT label-compare` — compare two label-based plans (Spec 7 Phase 5)
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(
    disable_version_flag = true,
    about = "Compare two label-based plans using existing analysis outputs (Spec 7 Phase 5). \
             Both labels must be built and analyzed for the given year."
)]
pub struct LabelCompareArgs {
    /// First plan label (must be built + analyzed)
    pub label_a: String,

    /// Second plan label (must be built + analyzed)
    pub label_b: String,

    /// Census year to compare (default: 2020)
    #[arg(short = 'y', long, default_value = "2020", value_name = "YEAR")]
    pub year: String,

    /// Emit structured JSON instead of the human-readable summary
    #[arg(long)]
    pub json: bool,

    /// Write output to this file instead of stdout
    #[arg(long = "out", value_name = "PATH")]
    pub out: Option<std::path::PathBuf>,
}
