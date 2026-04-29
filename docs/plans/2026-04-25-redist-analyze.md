# `redist analyze` + `redist map` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `redist analyze` and `redist map` subcommands to the CLI, with an extension model that makes adding new analyzers a one-file addition.

**Architecture:** An `Analyzer` trait in `redist-analysis` defines the contract. Each analyzer is a self-contained module implementing the trait. The CLI dispatcher loads final assignments + data files then runs requested analyzers, writing results to `{state}/analysis/{type}.json`. Maps stay Python — `redist map` is a thin subprocess shim around the existing `visualize_*.py` scripts.

**Tech Stack:** Rust, serde_json, csv crate, `redist-analysis` crate, Python subprocess (maps only)

---

## What gets built

| Analyzer | Input data | Output |
|----------|-----------|--------|
| `compactness` | tract geometries (WKB from units parquet via Python shim), assignments | per-district PP / Reock / CHR scores |
| `demographic` | `data/{year}/demographics/{state}_demographics_{year}.csv`, assignments | per-district race/ethnicity breakdown |
| `political` | `data/{year}/elections/presidential_by_tract.csv`, assignments | per-district Dem/Rep vote shares |
| `urban` | `outputs/{version}/data/{year}/units/{state}_*.parquet` place names, assignments | cities per district |
| `summary` | all of the above | rolled-up `district_summary.json` mirroring Python `district_summary.csv` |

Maps (`redist map`): subprocess calls to `scripts/pipeline/visualize_*.py` — no Rust rendering.

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `redist/crates/redist-analysis/src/analyzer.rs` | **Create** | `Analyzer` trait + `AnalyzerType` enum |
| `redist/crates/redist-analysis/src/demographic.rs` | **Create** | demographic analysis |
| `redist/crates/redist-analysis/src/political.rs` | **Create** | political (partisan) analysis |
| `redist/crates/redist-analysis/src/urban.rs` | **Create** | largest city per district |
| `redist/crates/redist-analysis/src/summary.rs` | **Create** | rolled-up district_summary.json |
| `redist/crates/redist-analysis/src/lib.rs` | **Modify** | expose new modules |
| `redist/crates/redist-cli/src/analyze.rs` | **Create** | `run_analyze()` dispatcher |
| `redist/crates/redist-cli/src/map.rs` | **Create** | `run_map()` Python subprocess shim |
| `redist/crates/redist-cli/src/args.rs` | **Modify** | `AnalyzeArgs`, `MapArgs` added to `Commands` |
| `redist/crates/redist-cli/src/main.rs` | **Modify** | wire `Commands::Analyze` and `Commands::Map` |
| `tests/acceptance/test_analyze_acceptance.py` | **Create** | end-to-end acceptance tests |

---

## Output format

All analysis outputs land under `outputs/{version}/{year}/{state_name}/analysis/`:

```
outputs/V3/2020/vermont/
  ├── final_assignments.json        (existing)
  ├── vra_analysis.json             (existing)
  └── analysis/
      ├── compactness.json
      ├── demographic.json
      ├── political.json
      ├── urban.json
      └── summary.json
```

Each JSON file has a `districts` array + top-level metadata:

```json
{
  "state": "vermont",
  "year": "2020",
  "version": "V3",
  "analyzer": "demographic",
  "districts": [
    { "district": 1, "total_pop": 643503, "pct_white": 0.934, "pct_black": 0.017, ... }
  ]
}
```

---

## Task 1: `Analyzer` trait and `AnalyzerType` enum

**Files:** Create `redist/crates/redist-analysis/src/analyzer.rs`

- [ ] **Write the trait and enum**

```rust
// redist-analysis/src/analyzer.rs
use std::path::Path;

/// Context passed to every analyzer — the common inputs.
pub struct AnalyzerContext<'a> {
    /// tract_index → district_id (1-based), loaded from final_assignments.json
    pub assignments: &'a std::collections::HashMap<String, usize>,
    pub state_name: &'a str,   // e.g. "vermont"
    pub state_code: &'a str,   // e.g. "VT"
    pub year: &'a str,         // e.g. "2020"
    pub version: &'a str,      // e.g. "V3"
    pub num_districts: usize,
    pub data_root: &'a Path,   // "data/" — source inputs
    pub output_root: &'a Path, // "outputs/{version}" — processed data
}

/// Implemented by every analyzer module.
/// Each analyzer reads its own data files given the context,
/// computes metrics, and returns JSON-serializable results.
pub trait Analyzer {
    type Output: serde::Serialize;

    fn name() -> &'static str where Self: Sized;

    fn run(ctx: &AnalyzerContext<'_>) -> anyhow::Result<Self::Output>
    where
        Self: Sized;
}

/// Selectable via --types on the CLI.
#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum AnalyzerType {
    Compactness,
    Demographic,
    Political,
    Urban,
    Summary,
    All,
}
```

- [ ] **Add to `lib.rs`**

```rust
pub mod analyzer;
pub use analyzer::{Analyzer, AnalyzerContext, AnalyzerType};
```

- [ ] **Write unit test** — build a minimal `AnalyzerContext` with 3 tracts and verify the enum variants parse from strings (clap `ValueEnum` derive):

```rust
#[test]
fn test_analyzer_type_all_variants() {
    use clap::ValueEnum;
    let variants = AnalyzerType::value_variants();
    assert!(variants.iter().any(|v| *v == AnalyzerType::Demographic));
    assert!(variants.iter().any(|v| *v == AnalyzerType::Political));
    assert!(variants.iter().any(|v| *v == AnalyzerType::All));
}
```

- [ ] **Run:** `cargo test -p redist-analysis test_analyzer_type_all_variants` — expect PASS

- [ ] **Commit:** `git commit -m "feat(analysis): Analyzer trait + AnalyzerType enum"`

---

## Task 2: Demographic analyzer

**Files:** Create `redist/crates/redist-analysis/src/demographic.rs`

Input: `data/{year}/demographics/{state_name}_demographics_{year}.csv`
Columns: `GEOID,state,county,tract,total_pop,male,female,white_non_hispanic,black_non_hispanic,asian_non_hispanic,hispanic,other`

- [ ] **Write failing test first**

```rust
// redist-analysis/src/demographic.rs (tests module)
#[test]
fn test_demographic_aggregation() {
    // Two tracts, both in district 1
    // Tract A: 1000 total, 800 white, 100 black, 100 hispanic
    // Tract B: 500 total, 400 white, 50 black, 50 hispanic
    // Expected district 1: total=1500, pct_white=0.8, pct_black=0.1, pct_hispanic=0.1
    let rows = vec![
        DemographicRow { geoid: "50001".into(), total_pop: 1000,
            white: 800, black: 100, asian: 0, hispanic: 100, other: 0 },
        DemographicRow { geoid: "50002".into(), total_pop: 500,
            white: 400, black: 50, asian: 0, hispanic: 50, other: 0 },
    ];
    let assignments: HashMap<String, usize> =
        [("50001".into(), 1), ("50002".into(), 1)].into();
    let result = aggregate_demographic(&rows, &assignments, 1);
    assert_eq!(result.districts.len(), 1);
    let d = &result.districts[0];
    assert_eq!(d.district, 1);
    assert_eq!(d.total_pop, 1500);
    assert!((d.pct_white - 0.8).abs() < 1e-6);
    assert!((d.pct_black - 0.1).abs() < 1e-6);
}
```

- [ ] **Run test** — expect FAIL (not yet implemented)

- [ ] **Implement**

```rust
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use anyhow::Context;
use crate::analyzer::{Analyzer, AnalyzerContext};

#[derive(Debug, Deserialize)]
struct DemographicRow {
    #[serde(rename = "GEOID")]
    geoid: String,
    total_pop: u64,
    white_non_hispanic: u64,
    black_non_hispanic: u64,
    asian_non_hispanic: u64,
    hispanic: u64,
    other: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DemographicDistrict {
    pub district: usize,
    pub total_pop: u64,
    pub pct_white: f64,
    pub pct_black: f64,
    pub pct_asian: f64,
    pub pct_hispanic: f64,
    pub pct_other: f64,
    pub pct_minority: f64,  // 1 - pct_white
}

#[derive(Debug, Serialize)]
pub struct DemographicResult {
    pub analyzer: &'static str,
    pub districts: Vec<DemographicDistrict>,
}

fn aggregate_demographic(
    rows: &[DemographicRow],
    assignments: &HashMap<String, usize>,
    num_districts: usize,
) -> DemographicResult {
    let mut totals = vec![(0u64, 0u64, 0u64, 0u64, 0u64, 0u64); num_districts + 1];
    for row in rows {
        if let Some(&dist) = assignments.get(&row.geoid) {
            if dist > 0 && dist <= num_districts {
                let t = &mut totals[dist];
                t.0 += row.total_pop;
                t.1 += row.white_non_hispanic;
                t.2 += row.black_non_hispanic;
                t.3 += row.asian_non_hispanic;
                t.4 += row.hispanic;
                t.5 += row.other;
            }
        }
    }
    let districts = (1..=num_districts).map(|d| {
        let (total, white, black, asian, hisp, other) = totals[d];
        let p = |n: u64| if total > 0 { n as f64 / total as f64 } else { 0.0 };
        DemographicDistrict {
            district: d, total_pop: total,
            pct_white: p(white), pct_black: p(black),
            pct_asian: p(asian), pct_hispanic: p(hisp),
            pct_other: p(other), pct_minority: 1.0 - p(white),
        }
    }).collect();
    DemographicResult { analyzer: "demographic", districts }
}

pub struct DemographicAnalyzer;
impl Analyzer for DemographicAnalyzer {
    type Output = DemographicResult;
    fn name() -> &'static str { "demographic" }
    fn run(ctx: &AnalyzerContext<'_>) -> anyhow::Result<Self::Output> {
        let csv_path = ctx.data_root
            .join(ctx.year).join("demographics")
            .join(format!("{}_demographics_{}.csv", ctx.state_name, ctx.year));
        let mut rdr = csv::Reader::from_path(&csv_path)
            .with_context(|| format!("demographics CSV not found: {}", csv_path.display()))?;
        let rows: Vec<DemographicRow> = rdr.deserialize().collect::<Result<_, _>>()?;
        Ok(aggregate_demographic(&rows, ctx.assignments, ctx.num_districts))
    }
}
```

- [ ] **Run test** — expect PASS
- [ ] **Commit:** `git commit -m "feat(analysis): demographic analyzer"`

---

## Task 3: Political analyzer

**Files:** Create `redist/crates/redist-analysis/src/political.rs`

Input: `data/{year}/elections/presidential_by_tract.csv`
Columns: `geoid,dem_votes,rep_votes,lib_votes,grn_votes,oth_votes`

- [ ] **Write failing test**

```rust
#[test]
fn test_political_aggregation() {
    let rows = vec![
        PoliticalRow { geoid: "50001".into(), dem_votes: 600.0, rep_votes: 400.0,
            lib_votes: 0.0, grn_votes: 0.0, oth_votes: 0.0 },
        PoliticalRow { geoid: "50002".into(), dem_votes: 300.0, rep_votes: 700.0,
            lib_votes: 0.0, grn_votes: 0.0, oth_votes: 0.0 },
    ];
    let assignments: HashMap<String, usize> =
        [("50001".into(), 1), ("50002".into(), 1)].into();
    let result = aggregate_political(&rows, &assignments, 1);
    let d = &result.districts[0];
    assert!((d.dem_pct - 0.45).abs() < 1e-6); // 900/(2000)
    assert!((d.rep_pct - 0.55).abs() < 1e-6);
    assert!(d.lean_dem);  // No: 45% D < 50%, so lean_dem = false
}
```

Wait — 900/2000 = 0.45 D, 0.55 R → `lean_dem = false`. Fix test:

```rust
    assert!(!d.lean_dem);
    assert!((d.margin - (-0.10)).abs() < 1e-6); // D - R = -0.10
```

- [ ] **Run test** — expect FAIL

- [ ] **Implement** (mirrors demographic structure, reads `presidential_by_tract.csv`)

```rust
#[derive(Debug, Deserialize)]
struct PoliticalRow {
    geoid: String,
    dem_votes: f64,
    rep_votes: f64,
    lib_votes: f64,
    grn_votes: f64,
    oth_votes: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PoliticalDistrict {
    pub district: usize,
    pub total_votes: f64,
    pub dem_votes: f64,
    pub rep_votes: f64,
    pub dem_pct: f64,
    pub rep_pct: f64,
    /// Positive = Dem advantage, negative = Rep advantage
    pub margin: f64,
    pub lean_dem: bool,
}

// aggregate_political + PoliticalAnalyzer implementing Analyzer
```

- [ ] **Run test** — expect PASS
- [ ] **Commit:** `git commit -m "feat(analysis): political analyzer"`

---

## Task 4: Urban analyzer (largest city per district)

**Files:** Create `redist/crates/redist-analysis/src/urban.rs`

Input: `outputs/{version}/data/{year}/units/{state_code_lower}_units_{year}.parquet` — has `GEOID` and `place_name` columns.

**Note:** Parquet reading in Rust without Arrow is non-trivial. Use a CSV fallback: if a `{state}_places_{year}.csv` exists in the same directory, read that. Otherwise emit empty results with a warning. Python pipeline can pre-export to CSV.

- [ ] **Write failing test** (CSV path)

```rust
#[test]
fn test_urban_largest_city() {
    let rows = vec![
        PlaceRow { geoid: "50001".into(), place_name: Some("Burlington".into()), place_pop: 45000 },
        PlaceRow { geoid: "50002".into(), place_name: Some("Montpelier".into()), place_pop: 8000 },
        PlaceRow { geoid: "50003".into(), place_name: None, place_pop: 0 },
    ];
    let assignments: HashMap<String, usize> =
        [("50001".into(), 1), ("50002".into(), 1), ("50003".into(), 1)].into();
    let result = aggregate_urban(&rows, &assignments, 1);
    assert_eq!(result.districts[0].largest_city, Some("Burlington".into()));
    assert_eq!(result.districts[0].largest_city_pop, 45000);
}
```

- [ ] **Run test** — expect FAIL
- [ ] **Implement**

```rust
#[derive(Debug, Clone, Serialize)]
pub struct UrbanDistrict {
    pub district: usize,
    pub largest_city: Option<String>,
    pub largest_city_pop: u64,
    pub cities: Vec<CityEntry>,  // all named places in district, sorted by pop desc
}

#[derive(Debug, Clone, Serialize)]
pub struct CityEntry {
    pub name: String,
    pub pop: u64,
}
// aggregate_urban + UrbanAnalyzer implementing Analyzer
// Reads {state}_places_{year}.csv from output_root/data/{year}/places/
```

- [ ] **Run test** — expect PASS
- [ ] **Commit:** `git commit -m "feat(analysis): urban analyzer"`

---

## Task 5: Summary analyzer

**Files:** Create `redist/crates/redist-analysis/src/summary.rs`

Rolls up demographic + political + compactness + urban into one `district_summary.json` that mirrors what the Python pipeline writes to `district_summary.csv`.

- [ ] **Write failing test**

```rust
#[test]
fn test_summary_merge() {
    // Given one district with all four sub-results populated,
    // summary should have all fields on the same record.
    let demo = DemographicDistrict { district: 1, total_pop: 1000, pct_white: 0.9, .. };
    let pol  = PoliticalDistrict  { district: 1, dem_pct: 0.55, margin: 0.10, .. };
    let comp = CompactnessMetrics { district: 1, polsby_popper: 0.35, reock: 0.45, .. };
    let urb  = UrbanDistrict      { district: 1, largest_city: Some("Burlington".into()), .. };
    let s = merge_district(1, Some(&demo), Some(&pol), Some(&comp), Some(&urb));
    assert_eq!(s.district, 1);
    assert!((s.polsby_popper - 0.35).abs() < 1e-9);
    assert!((s.dem_pct - 0.55).abs() < 1e-9);
    assert_eq!(s.largest_city.as_deref(), Some("Burlington"));
}
```

- [ ] **Run test** — expect FAIL

- [ ] **Implement** `SummaryDistrict` struct (all fields optional — absent if analyzer wasn't run), `SummaryAnalyzer` that runs all sub-analyzers internally.

- [ ] **Run test** — expect PASS
- [ ] **Commit:** `git commit -m "feat(analysis): summary analyzer"`

---

## Task 6: `redist analyze` CLI subcommand

**Files:** Create `redist/crates/redist-cli/src/analyze.rs`, modify `args.rs` and `main.rs`

- [ ] **Add `AnalyzeArgs` to `args.rs`**

```rust
#[derive(Debug, Parser)]
pub struct AnalyzeArgs {
    #[arg(long)]
    pub state: String,

    #[arg(short = 'y', long, default_value = "2020")]
    pub year: Year,

    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,

    #[arg(long, default_value = "outputs")]
    pub output_base: String,

    /// Analyzers to run (default: all)
    #[arg(long = "types", value_delimiter = ' ', num_args = 0..,
          default_value = "all")]
    pub types: Vec<AnalyzerType>,

    #[arg(long)]
    pub force: bool,
}
```

- [ ] **Write `analyze.rs` dispatcher**

```rust
// redist-cli/src/analyze.rs
pub fn run_analyze(args: &AnalyzeArgs) -> anyhow::Result<()> {
    // 1. Load final_assignments.json
    let assignments_path = /* outputs/{version}/{year}/{state}/final_assignments.json */;
    let assignments: HashMap<String, usize> = serde_json::from_str(
        &std::fs::read_to_string(&assignments_path)?
    )?;
    
    // 2. Build context
    let ctx = AnalyzerContext { assignments: &assignments, ... };
    
    // 3. Resolve which types to run (expand "all")
    let types = resolve_types(&args.types);  // expand All -> all variants
    
    // 4. Dispatch
    let analysis_dir = /* outputs/{version}/{year}/{state}/analysis/ */;
    std::fs::create_dir_all(&analysis_dir)?;
    
    for typ in &types {
        let out_path = analysis_dir.join(format!("{}.json", typ.name()));
        if out_path.exists() && !args.force {
            eprintln!("[skip] {} (already exists)", typ.name());
            continue;
        }
        match typ {
            AnalyzerType::Demographic => write_json(&out_path, DemographicAnalyzer::run(&ctx)?)?,
            AnalyzerType::Political   => write_json(&out_path, PoliticalAnalyzer::run(&ctx)?)?,
            AnalyzerType::Urban       => write_json(&out_path, UrbanAnalyzer::run(&ctx)?)?,
            AnalyzerType::Compactness => write_json(&out_path, /* compactness - see note */)?,
            AnalyzerType::Summary     => write_json(&out_path, SummaryAnalyzer::run(&ctx)?)?,
            AnalyzerType::All         => unreachable!(), // expanded above
        }
        eprintln!("[OK] {} -> {}", typ.name(), out_path.display());
    }
    Ok(())
}

fn write_json<T: serde::Serialize>(path: &Path, value: T) -> anyhow::Result<()> {
    let tmp = path.with_extension("tmp.json");
    std::fs::write(&tmp, serde_json::to_string_pretty(&value)?)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
```

Note on compactness: the existing `redist-analysis` `all_metrics()` takes WKB geometry per district. The geometry comes from TIGER shapefiles and requires dissolving tract polygons. For now, compactness via `redist analyze` calls a Python shim to dissolve + export WKB, then calls `all_metrics()`. Add `// TODO: native geometry dissolve once redist-data has tract WKB` comment.

- [ ] **Wire into `main.rs`**

```rust
Commands::Analyze(args) => {
    redist_cli::analyze::run_analyze(&args)
        .unwrap_or_else(|e| { eprintln!("ERROR: {e}"); std::process::exit(1); });
}
```

- [ ] **Run:** `redist analyze --state VT --year 2020 --version V3 --types demographic political` against existing VT outputs — expect JSON written to `outputs/V3/2020/vermont/analysis/`

- [ ] **Commit:** `git commit -m "feat(cli): redist analyze subcommand"`

---

## Task 7: `redist map` — Python subprocess shim

**Files:** Create `redist/crates/redist-cli/src/map.rs`, add `MapArgs` to `args.rs`

`redist map` is a thin wrapper that calls the existing Python visualization scripts as subprocesses. It does not render any geometry in Rust.

- [ ] **Add `MapArgs` to `args.rs`**

```rust
#[derive(Debug, Parser)]
pub struct MapArgs {
    #[arg(long)]
    pub state: String,

    #[arg(short = 'y', long, default_value = "2020")]
    pub year: Year,

    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,

    /// Map types to generate
    #[arg(long = "types", value_delimiter = ' ', num_args = 0..,
          default_value = "districts")]
    pub types: Vec<MapType>,

    #[arg(long, default_value = "150",
          value_parser = clap::builder::PossibleValuesParser::new(["72","100","150","200","300"]))]
    pub dpi: String,
}

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum MapType {
    Districts,   // visualize_districts.py
    Rounds,      // visualize_all_rounds.py
    Compactness, // visualize_compactness.py
    Political,   // visualize_partisan_lean.py
    Demographic, // visualize_district_demographics.py
    All,
}
```

- [ ] **Implement `map.rs`**

```rust
// redist-cli/src/map.rs
pub fn run_map(args: &MapArgs) -> anyhow::Result<()> {
    let python = std::env::var("REDIST_PYTHON").unwrap_or_else(|_| "python".to_string());
    let types = resolve_map_types(&args.types);
    
    for map_type in &types {
        let script = map_type.script_path(); // e.g. "scripts/pipeline/visualize_districts.py"
        let status = std::process::Command::new(&python)
            .arg(script)
            .arg("--state").arg(&args.state)
            .arg("--year").arg(args.year.to_string())
            .arg("--version").arg(&args.version)
            .arg("--dpi").arg(&args.dpi)
            .status()?;
        if !status.success() {
            anyhow::bail!("map script failed for {:?}", map_type);
        }
        eprintln!("[OK] {} maps generated", map_type.name());
    }
    Ok(())
}
```

- [ ] **Wire into `main.rs`**

- [ ] **Manual test:** `redist map --state VT --year 2020 --version V3 --types districts` — expect Python script runs and map PNG appears in correct location.

- [ ] **Commit:** `git commit -m "feat(cli): redist map Python subprocess shim"`

---

## Task 8: Acceptance tests

**Files:** Create `tests/acceptance/test_analyze_acceptance.py`

- [ ] **Write failing tests**

```python
class TestAnalyzeAcceptance(unittest.TestCase):
    def setUp(self):
        self.binary = find_redist_binary()
        self.outdir = tmp_state_with_assignments("VT", "2020", "V3")

    def test_demographic_produces_json(self):
        result = subprocess.run(
            [self.binary, "analyze", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "demographic"],
            capture_output=True, text=True
        )
        self.assertEqual(result.returncode, 0)
        out = Path(self.outdir) / "analysis" / "demographic.json"
        self.assertTrue(out.exists())
        data = json.loads(out.read_text())
        self.assertIn("districts", data)
        self.assertEqual(len(data["districts"]), 1)  # VT = 1 district
        d = data["districts"][0]
        self.assertIn("pct_minority", d)
        self.assertIn("total_pop", d)
        self.assertGreater(d["total_pop"], 0)

    def test_political_produces_json(self):
        result = subprocess.run(
            [self.binary, "analyze", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "political"],
            capture_output=True, text=True
        )
        self.assertEqual(result.returncode, 0)
        out = Path(self.outdir) / "analysis" / "political.json"
        data = json.loads(out.read_text())
        d = data["districts"][0]
        self.assertIn("dem_pct", d)
        self.assertAlmostEqual(d["dem_pct"] + d["rep_pct"], 1.0, places=1)

    def test_all_types_run(self):
        result = subprocess.run(
            [self.binary, "analyze", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "all"],
            capture_output=True, text=True
        )
        self.assertEqual(result.returncode, 0)
        analysis_dir = Path(self.outdir) / "analysis"
        for expected in ["demographic.json", "political.json", "urban.json", "summary.json"]:
            self.assertTrue((analysis_dir / expected).exists(), f"missing {expected}")

    def test_skip_existing_without_force(self):
        # Run twice — second run should skip (output unchanged)
        subprocess.run([self.binary, "analyze", "--state", "VT", "--year", "2020",
                        "--version", "V3", "--types", "demographic"], check=True)
        out = Path(self.outdir) / "analysis" / "demographic.json"
        mtime1 = out.stat().st_mtime
        subprocess.run([self.binary, "analyze", "--state", "VT", "--year", "2020",
                        "--version", "V3", "--types", "demographic"], check=True)
        self.assertEqual(out.stat().st_mtime, mtime1)  # not regenerated

    def test_force_regenerates(self):
        subprocess.run([self.binary, "analyze", "--state", "VT", "--year", "2020",
                        "--version", "V3", "--types", "demographic"], check=True)
        out = Path(self.outdir) / "analysis" / "demographic.json"
        mtime1 = out.stat().st_mtime
        import time; time.sleep(0.05)
        subprocess.run([self.binary, "analyze", "--state", "VT", "--year", "2020",
                        "--version", "V3", "--types", "demographic", "--force"], check=True)
        self.assertGreater(out.stat().st_mtime, mtime1)
```

- [ ] **Run:** `pytest tests/acceptance/test_analyze_acceptance.py -v` — expect PASS for demographic + political (urban + compactness can be skipped initially if data not present)

- [ ] **Commit:** `git commit -m "test: redist analyze acceptance tests"`

---

## Adding a new analyzer (the extension model)

To add a new analyzer (e.g., `maup` for MAUP sensitivity):

1. Create `redist/crates/redist-analysis/src/maup.rs` implementing `Analyzer`
2. Add `Maup` variant to `AnalyzerType` in `analyzer.rs`
3. Add one match arm in `analyze.rs` dispatcher
4. Add to `lib.rs` exports

No changes to `main.rs`, no changes to `args.rs` (clap picks up new enum variant automatically). Three files touched.

---

## Notes

**Compactness geometry problem:** Computing PP/Reock requires dissolving tract polygons into district polygons. This needs the tract geometries (WKB), which currently aren't stored separately — they're inside TIGER shapefiles. Short-term: call Python to dissolve and write a `district_geometries.wkb` file, then call `all_metrics()` in Rust. Long-term: `redist-data` reads TIGER natively (Phase 2 complete for adjacency; geometry export is a small addition).

**Political data availability:** `presidential_by_tract.csv` exists for 2020 (`data/2020/elections/presidential_by_tract.csv`). For 2010 and 2000, election data at tract level may not exist — `PoliticalAnalyzer::run()` should return an empty result with a warning rather than failing.

**Urban data:** Places data is in `outputs/V3/data/{year}/places/` as parquet files. Export to CSV with `python scripts/data/export_places_csv.py` or add `polars` as a dependency to read parquet natively.
