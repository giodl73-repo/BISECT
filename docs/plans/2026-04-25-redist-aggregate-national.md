# `redist aggregate` + `redist map --scope national` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the last two Python-only gaps in the pipeline. `redist aggregate` merges all 50 state analysis JSONs into national datasets (replacing `create_us_aggregate.py`). `redist map --scope national` renders the full 435-district national map with Alaska and Hawaii insets (replacing `visualize_national_districts.py` and the national choropleth scripts).

**Architecture:**
- New `Commands::Aggregate` in `redist-cli` — walks `outputs/{version}/states/*/analysis/` and merges
- `InsetProjection` in `redist-map/src/projection.rs` — continental + AK inset + HI inset in one canvas
- Extended `MapArgs` with `--scope state|national` flag
- National map renderer in `redist-cli/src/map_cmd.rs` that loads all state geometries and dispatches to `InsetProjection`

**Tech Stack:** Rust, existing `redist-map` + `redist-analysis` + `redist-cli` crates; `serde_json`, `csv` crate

---

## What `process_nation.py` does that we're replacing

| Python task | Rust equivalent | Plan task |
|-------------|-----------------|-----------|
| `create_us_aggregate.py` — combine 50 state CSVs | `redist aggregate` — merge state analysis JSONs + write CSV | Task 1–2 |
| `create_us_rounds_hierarchy.py` — national rounds CSV | `redist aggregate --types rounds` | Task 2 |
| `visualize_national_districts.py` — 435-district map | `redist map --scope national --types districts` | Task 3–4 |
| `visualize_partisan_lean.py --scope national` | `redist map --scope national --types political` | Task 4 |
| `visualize_district_demographics.py --scope national` | `redist map --scope national --types demographic` | Task 4 |
| `visualize_compactness.py --scope national` | `redist map --scope national --types compactness` | Task 4 |

Dashboard generation (`process_nation.py` task 7) stays Python — `redist-web` is future work.

---

## National map projection design

The continental US fits a standard equirectangular bbox. Alaska and Hawaii are placed as insets:

```
┌─────────────────────────────────────┐
│                                     │
│      Continental US                 │
│      lon: -125 to -65               │
│      lat:   24 to  50               │
│                                     │
│  ┌──────┐ ┌───┐                     │
│  │  AK  │ │HI │                     │
│  │ 35%  │ │90%│                     │
│  └──────┘ └───┘                     │
└─────────────────────────────────────┘
```

**Inset parameters:**
- **Continental**: bbox (-125, 24) → (-65, 50), occupies 100% of canvas width, ~75% of height (top)
- **Alaska inset**: occupies bottom-left ~28% width × ~25% height. Alaska natural bbox: (-170, 51) → (-130, 72). Scale factor: 0.35× continental scale (Alaska is huge — scaled down to fit)
- **Hawaii inset**: occupies bottom-left next to Alaska, ~12% width × ~10% height. Hawaii bbox: (-161, 18) → (-154, 23)

`InsetProjection` struct holds three independent `Projection` instances + pixel offsets for each region. When projecting a coordinate, it first classifies it by longitude/latitude range, then applies the appropriate sub-projection + offset.

---

## File Map

| File | Action |
|------|--------|
| `redist/crates/redist-map/src/projection.rs` | **Modify** — add `InsetProjection` struct |
| `redist/crates/redist-cli/src/aggregate.rs` | **Create** — `run_aggregate()` dispatcher |
| `redist/crates/redist-cli/src/args.rs` | **Modify** — `AggregateArgs`, `--scope` flag on `MapArgs` |
| `redist/crates/redist-cli/src/lib.rs` | **Modify** — expose `aggregate` module |
| `redist/crates/redist-cli/src/main.rs` | **Modify** — wire `Commands::Aggregate` |
| `redist/crates/redist-cli/src/map_cmd.rs` | **Modify** — national map rendering path |
| `redist/tests/acceptance/test_aggregate_national.py` | **Create** — L2 acceptance tests |

---

## Task 1: `InsetProjection` for national maps

**Files:** `redist/crates/redist-map/src/projection.rs`

- [ ] **Write failing L0 tests**

```rust
#[test]
fn test_inset_projection_continental_coord_projects_in_main_area() {
    let proj = InsetProjection::us_national(1200, 800);
    // Kansas City, MO: lon=-94.6, lat=39.1 → should be roughly center-right of canvas
    let (x, y) = proj.project(-94.6, 39.1);
    // Canvas is 1200×800; continental occupies top 75% (height 600px)
    assert!(x > 400.0 && x < 900.0, "x={x}");
    assert!(y > 50.0 && y < 600.0, "y={y}");
}

#[test]
fn test_inset_projection_alaska_coord_in_inset_area() {
    let proj = InsetProjection::us_national(1200, 800);
    // Anchorage, AK: lon=-149.9, lat=61.2 → should be in lower-left inset
    let (x, y) = proj.project(-149.9, 61.2);
    // AK inset occupies bottom-left: x < 340px, y > 600px
    assert!(x < 340.0, "AK x={x} should be in left inset");
    assert!(y > 580.0, "AK y={y} should be in bottom inset");
}

#[test]
fn test_inset_projection_hawaii_coord_in_inset_area() {
    let proj = InsetProjection::us_national(1200, 800);
    // Honolulu, HI: lon=-157.8, lat=21.3 → should be in HI inset (right of AK)
    let (x, y) = proj.project(-157.8, 21.3);
    // HI inset is to the right of AK inset
    assert!(x > 340.0 && x < 500.0, "HI x={x}");
    assert!(y > 680.0, "HI y={y} should be near bottom");
}

#[test]
fn test_inset_projection_consistent_scaling() {
    let proj = InsetProjection::us_national(1200, 800);
    // Two points at same continental latitude should differ only in x
    let (x1, y1) = proj.project(-120.0, 37.0);
    let (x2, y2) = proj.project(-80.0, 37.0);
    assert!((y2 - y1).abs() < 2.0, "same lat should give same y: y1={y1} y2={y2}");
    assert!(x2 > x1, "more east = higher x");
}
```

- [ ] **Run tests** — expect FAIL

- [ ] **Implement `InsetProjection`**

```rust
/// National US map projection with Alaska and Hawaii insets.
///
/// Three independent sub-projections are placed on one canvas:
/// - Continental: top 75% of canvas, standard equirectangular
/// - Alaska inset: bottom-left 28%×25% of canvas, scaled to 0.35×
/// - Hawaii inset: bottom-left, right of AK, scaled to 0.9×
pub struct InsetProjection {
    continental: (Projection, f64, f64),  // (proj, x_offset_px, y_offset_px)
    alaska: (Projection, f64, f64),
    hawaii: (Projection, f64, f64),
    canvas_w: u32,
    canvas_h: u32,
}

impl InsetProjection {
    /// Standard US national layout for a given canvas size.
    pub fn us_national(canvas_w: u32, canvas_h: u32) -> Self {
        let w = canvas_w as f64;
        let h = canvas_h as f64;

        // Continental: full width, top 75%
        let cont_h = (h * 0.75) as u32;
        let continental = (
            Projection::from_bbox(-125.0, 24.0, -65.0, 50.0, canvas_w, cont_h, 0.02),
            0.0, 0.0
        );

        // Alaska inset: bottom-left, 28% width × 25% height
        let ak_w = (w * 0.28) as u32;
        let ak_h = (h * 0.25) as u32;
        let alaska = (
            Projection::from_bbox(-180.0, 51.0, -130.0, 72.0, ak_w, ak_h, 0.02),
            0.0, cont_h as f64
        );

        // Hawaii inset: right of AK, 12% width × 10% height, vertically centred in inset strip
        let hi_w = (w * 0.12) as u32;
        let hi_h = (h * 0.10) as u32;
        let hi_y_offset = cont_h as f64 + (ak_h as f64 - hi_h as f64) * 0.5;
        let hawaii = (
            Projection::from_bbox(-161.0, 18.0, -154.0, 23.0, hi_w, hi_h, 0.04),
            ak_w as f64 + 4.0, hi_y_offset
        );

        Self { continental, alaska, hawaii, canvas_w, canvas_h }
    }

    /// Project (lon, lat) to (svg_x, svg_y) using appropriate sub-projection.
    pub fn project(&self, lon: f64, lat: f64) -> (f64, f64) {
        // Alaska: lon < -130 and lat > 51
        if lon < -130.0 && lat > 51.0 {
            let (p, ox, oy) = &self.alaska;
            let (x, y) = p.project(lon, lat);
            return (x + ox, y + oy);
        }
        // Hawaii: lon < -154 and lat < 25
        if lon < -154.0 && lat < 25.0 {
            let (p, ox, oy) = &self.hawaii;
            let (x, y) = p.project(lon, lat);
            return (x + ox, y + oy);
        }
        // Continental (everything else)
        let (p, ox, oy) = &self.continental;
        let (x, y) = p.project(lon, lat);
        (x + ox, y + oy)
    }

    pub fn canvas_w(&self) -> u32 { self.canvas_w }
    pub fn canvas_h(&self) -> u32 { self.canvas_h }
}
```

- [ ] **Expose from `redist-map/src/lib.rs`**:
```rust
pub use projection::{Projection, InsetProjection};
```

- [ ] **Run tests** — expect PASS

- [ ] **Commit:** `git commit -m "feat(map): InsetProjection for US national map (continental + AK + HI insets)"`

---

## Task 2: `redist aggregate` subcommand

**Files:** `redist/crates/redist-cli/src/aggregate.rs`, `args.rs`, `lib.rs`, `main.rs`

Walks `outputs/{version}/states/*/analysis/` for all present states, merges all per-type JSONs into national datasets.

- [ ] **Add `AggregateArgs` to `args.rs`**

```rust
#[derive(Debug, Parser)]
pub struct AggregateArgs {
    #[arg(short = 'v', long, default_value = "v1")]
    pub version: String,

    /// Types to aggregate (default: all)
    #[arg(long = "types", value_delimiter = ' ', num_args = 0..,
          default_values = ["all"])]
    pub types: Vec<AnalyzerType>,

    /// Write CSV exports alongside JSON
    #[arg(long)]
    pub csv: bool,

    /// Re-aggregate even if output exists
    #[arg(long)]
    pub force: bool,
}
```

- [ ] **Write L0 tests for merge logic**

```rust
// aggregate.rs tests
#[test]
fn test_merge_demographic_two_states() {
    let vt = serde_json::json!({
        "analyzer": "demographic",
        "districts": [{"district": 1, "state": "VT", "total_pop": 643077, "pct_minority": 0.11}]
    });
    let al = serde_json::json!({
        "analyzer": "demographic",
        "districts": [
            {"district": 1, "state": "AL", "total_pop": 715760, "pct_minority": 0.40},
            {"district": 2, "state": "AL", "total_pop": 715760, "pct_minority": 0.35},
        ]
    });
    let merged = merge_analyzer_outputs(&[("vermont", &vt), ("alabama", &al)]);
    assert_eq!(merged["districts"].as_array().unwrap().len(), 3);
    assert_eq!(merged["state_count"].as_u64().unwrap(), 2);
    assert_eq!(merged["district_count"].as_u64().unwrap(), 3);
}

#[test]
fn test_merge_adds_state_field() {
    let vt = serde_json::json!({"analyzer": "demographic",
        "districts": [{"district": 1, "total_pop": 643077}]
    });
    let merged = merge_analyzer_outputs(&[("vermont", &vt)]);
    let d = &merged["districts"][0];
    assert_eq!(d["state"].as_str().unwrap(), "vermont");
}
```

- [ ] **Run tests** — expect FAIL

- [ ] **Implement `aggregate.rs`**

```rust
pub fn run_aggregate(args: &AggregateArgs) -> anyhow::Result<()> {
    let output_root = PathBuf::from("outputs").join(&args.version);
    let national_dir = output_root.join("national");
    std::fs::create_dir_all(&national_dir)?;

    let types = resolve_types(&args.types);

    // Discover all states with analysis outputs
    let states_dir = output_root.join("states");
    let mut state_dirs: Vec<(String, PathBuf)> = std::fs::read_dir(&states_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
        .collect();
    state_dirs.sort_by_key(|(name, _)| name.clone());

    eprintln!("[aggregate] Found {} states in {}",
        state_dirs.len(), states_dir.display());

    for analyzer_type in &types {
        let type_name = analyzer_type.name();
        let out_path = national_dir.join(format!("us_{type_name}.json"));
        if out_path.exists() && !args.force {
            eprintln!("[skip] {type_name} (exists)");
            continue;
        }

        // Load per-state JSONs
        let mut state_data: Vec<(String, serde_json::Value)> = vec![];
        for (state_name, state_dir) in &state_dirs {
            let json_path = state_dir.join("analysis").join(format!("{type_name}.json"));
            if json_path.exists() {
                let v: serde_json::Value = serde_json::from_str(
                    &std::fs::read_to_string(&json_path)?
                )?;
                state_data.push((state_name.clone(), v));
            }
        }

        let refs: Vec<(&str, &serde_json::Value)> = state_data.iter()
            .map(|(n, v)| (n.as_str(), v))
            .collect();
        let merged = merge_analyzer_outputs(&refs);
        write_json_atomic(&out_path, &merged)?;
        eprintln!("[OK] {type_name} -> {} ({} states, {} districts)",
            out_path.display(),
            merged["state_count"].as_u64().unwrap_or(0),
            merged["district_count"].as_u64().unwrap_or(0));

        // CSV export
        if args.csv {
            write_csv_export(&national_dir, type_name, &merged)?;
        }
    }
    Ok(())
}

/// Merge per-state analyzer outputs into one national JSON.
/// Adds "state" field to each district record. Top-level has state_count and district_count.
pub fn merge_analyzer_outputs(states: &[(&str, &serde_json::Value)]) -> serde_json::Value {
    let analyzer = states.first()
        .and_then(|(_, v)| v.get("analyzer"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let mut all_districts: Vec<serde_json::Value> = vec![];
    for (state_name, state_json) in states {
        if let Some(districts) = state_json.get("districts").and_then(|d| d.as_array()) {
            for d in districts {
                let mut record = d.clone();
                record["state"] = serde_json::Value::String(state_name.to_string());
                all_districts.push(record);
            }
        }
    }

    serde_json::json!({
        "analyzer": analyzer,
        "scope": "national",
        "state_count": states.len(),
        "district_count": all_districts.len(),
        "districts": all_districts,
    })
}
```

- [ ] **Run tests** — expect PASS
- [ ] **Wire into `main.rs` and `lib.rs`**
- [ ] **Commit:** `git commit -m "feat(cli): redist aggregate — merge state analysis JSONs into national datasets"`

---

## Task 3: CSV export from aggregate

**Files:** `aggregate.rs` (extend)

- [ ] **Write L0 tests for CSV export**

```rust
#[test]
fn test_csv_export_demographic_headers() {
    let merged = serde_json::json!({
        "analyzer": "demographic",
        "districts": [
            {"state": "vermont", "district": 1, "total_pop": 643077,
             "pct_white": 0.89, "pct_minority": 0.11, "is_majority_minority": false}
        ]
    });
    let csv = districts_to_csv(&merged);
    assert!(csv.contains("state,district,"));
    assert!(csv.contains("vermont,1,"));
    assert!(csv.contains("643077"));
}
```

- [ ] **Run test** — expect FAIL
- [ ] **Implement `districts_to_csv(merged: &Value) -> String`**
  - Extract all unique keys from first district record as headers
  - Write header row + one row per district
  - Write to `national/us_{type}.csv`

- [ ] **Run test** — expect PASS
- [ ] **Commit:** `git commit -m "feat(aggregate): CSV export for national district datasets"`

---

## Task 4: `redist map --scope national`

**Files:** `redist/crates/redist-cli/src/args.rs`, `map_cmd.rs`

- [ ] **Add `--scope` flag to `MapArgs`**

```rust
#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum MapScope { State, National }

// In MapArgs:
#[arg(long, default_value = "state")]
pub scope: MapScope,
```

- [ ] **Write L0 tests**

```rust
#[test]
fn test_map_scope_default_is_state() {
    let args = MapArgs::parse_from(["map", "--state", "VT"]);
    assert_eq!(args.scope, MapScope::State);
}
#[test]
fn test_map_scope_national_parses() {
    let args = MapArgs::parse_from(["map", "--scope", "national"]);
    assert_eq!(args.scope, MapScope::National);
}
```

- [ ] **Run tests** — expect FAIL, then PASS

- [ ] **Implement national map rendering in `map_cmd.rs`**

When `scope == National`, `run_map` ignores `--state` and renders all present states:

```rust
MapScope::National => run_national_map(args, &font_db)?,
```

```rust
fn run_national_map(args: &MapArgs, font_db: &FontDb) -> anyhow::Result<()> {
    use redist_map::InsetProjection;

    let output_root = PathBuf::from("outputs").join(&args.version);
    let national_maps_dir = output_root.join("national").join("maps");
    std::fs::create_dir_all(&national_maps_dir)?;
    let dpi: u32 = args.dpi.parse().unwrap_or(150);
    let (w, h) = canvas_size_from_dpi(dpi, 16.0, 1.6); // 16-inch wide, 16:10 ratio
    let proj = InsetProjection::us_national(w, h);

    let types = resolve_map_types(&args.types);

    for map_type in &types {
        let out = national_maps_dir.join(format!("{}.png", map_type.name()));
        if out.exists() && !args.force {
            eprintln!("[skip] national {}", map_type.name());
            continue;
        }

        // Discover all state assignments
        let states_dir = output_root.join("states");
        let state_dirs: Vec<_> = std::fs::read_dir(&states_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();

        let mut all_districts = vec![];
        let scheme = CategoricalScheme::default();

        for state_entry in &state_dirs {
            let state_name = state_entry.file_name().to_string_lossy().into_owned();
            let state_data_dir = state_entry.path().join("data");
            let assignments_path = state_data_dir.join("final_assignments.json");
            if !assignments_path.exists() { continue; }

            let raw: HashMap<String, usize> = serde_json::from_str(
                &std::fs::read_to_string(&assignments_path)?
            )?;

            // Resolve state code from state name (reverse lookup)
            let state_code = state_name_to_code(&state_name).unwrap_or("XX");

            let geoms = match load_district_geometries(
                &state_name, state_code, &args.year.to_string(),
                &args.version, &raw, Path::new("data")
            ) {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("[warn] {state_name}: {e}");
                    continue;
                }
            };

            // Load analysis stat for choropleth types
            let analysis = load_analysis_json(&state_entry.path(), map_type.name());

            for (district_id, mp) in geoms {
                let (color, label) = district_color_label(
                    district_id, &mp, map_type, &analysis, &scheme
                );
                all_districts.push((district_id, mp, color, label));
            }
        }

        eprintln!("[national {}] {} districts across {} states",
            map_type.name(), all_districts.len(), state_dirs.len());

        // Build SVG using InsetProjection
        let svg = build_national_svg(&all_districts, &proj, w, h);
        let png = svg_to_png(&svg, font_db)?;
        std::fs::write(&out, &png)?;
        eprintln!("[OK] national {} -> {} ({} bytes)", map_type.name(), out.display(), png.len());
    }
    Ok(())
}
```

- [ ] **Implement `build_national_svg`** — same as `build_svg` but uses `InsetProjection` instead of `Projection`. Extract shared path-building logic.

- [ ] **Implement `state_name_to_code`** — reverse lookup of STATE_NAMES map.

- [ ] **Commit:** `git commit -m "feat(cli): redist map --scope national with InsetProjection (AK+HI insets)"`

---

## Task 5: L0 tests for aggregate merge + L2 acceptance tests

**Files:** `tests/acceptance/test_aggregate_national.py`

- [ ] **Write L2 acceptance tests**

```python
class TestAggregateAcceptance(unittest.TestCase):
    """Requires VT + AL outputs (at minimum) from redist state + redist analyze."""

    @classmethod
    def setUpClass(cls):
        # Need at least 2 states with analysis
        states_dir = Path("outputs/V3/states")
        states_with_analysis = [
            s for s in states_dir.iterdir()
            if (s / "analysis" / "demographic.json").exists()
        ]
        if len(states_with_analysis) < 2:
            raise unittest.SkipTest(
                "Need at least 2 states with analysis. "
                "Run: redist analyze --state VT --types all && redist analyze --state AL --types all"
            )
        cls.binary = find_redist_binary()
        cls.n_states = len(states_with_analysis)

    def test_aggregate_demographic_produces_json(self):
        r = run([binary, "aggregate", "--version", "V3", "--types", "demographic", "--force"])
        assert r.returncode == 0
        out = Path("outputs/V3/national/us_demographic.json")
        assert out.exists()
        data = json.loads(out.read_text())
        assert data["state_count"] == self.n_states
        assert data["district_count"] >= self.n_states  # at least 1 per state
        assert all("state" in d for d in data["districts"])

    def test_aggregate_all_types_produce_files(self):
        r = run([binary, "aggregate", "--version", "V3", "--types", "all", "--force"])
        assert r.returncode == 0
        for name in ["us_demographic.json", "us_political.json",
                     "us_urban.json", "us_compactness.json", "us_summary.json"]:
            assert (Path("outputs/V3/national") / name).exists()

    def test_aggregate_csv_export(self):
        r = run([binary, "aggregate", "--version", "V3", "--types", "demographic",
                 "--force", "--csv"])
        assert r.returncode == 0
        csv_out = Path("outputs/V3/national/us_demographic.csv")
        assert csv_out.exists()
        rows = csv_out.read_text().splitlines()
        assert len(rows) > 1  # header + at least one district
        assert "state" in rows[0]
        assert "district" in rows[0]

    def test_aggregate_district_count_matches_state_sum(self):
        run([binary, "aggregate", "--version", "V3", "--types", "demographic",
             "--force"])
        data = json.loads(Path("outputs/V3/national/us_demographic.json").read_text())
        assert len(data["districts"]) == data["district_count"]

    def test_aggregate_skip_without_force(self):
        run([binary, "aggregate", "--version", "V3", "--types", "demographic", "--force"])
        out = Path("outputs/V3/national/us_demographic.json")
        mtime1 = out.stat().st_mtime
        time.sleep(0.05)
        r = run([binary, "aggregate", "--version", "V3", "--types", "demographic"])
        assert "skip" in r.stderr.lower()
        assert out.stat().st_mtime == mtime1


class TestNationalMapAcceptance(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        states_dir = Path("outputs/V3/states")
        states_with_data = [
            s for s in states_dir.iterdir()
            if (s / "data" / "final_assignments.json").exists()
        ]
        if len(states_with_data) < 2:
            raise unittest.SkipTest("Need at least 2 states with assignments.")
        cls.binary = find_redist_binary()

    def test_national_districts_map_valid_png(self):
        r = run([binary, "map", "--scope", "national", "--version", "V3",
                 "--types", "districts", "--force"])
        assert r.returncode == 0
        png = Path("outputs/V3/national/maps/districts.png")
        assert png.exists()
        data = png.read_bytes()
        assert data[:4] == b"\x89PNG"
        # National map should be large (16-inch wide at 150 DPI = 2400px wide)
        assert len(data) > 50_000

    def test_national_map_skip_without_force(self):
        run([binary, "map", "--scope", "national", "--version", "V3",
             "--types", "districts", "--force"])
        png = Path("outputs/V3/national/maps/districts.png")
        mtime1 = png.stat().st_mtime
        time.sleep(0.05)
        r = run([binary, "map", "--scope", "national", "--version", "V3",
                 "--types", "districts"])
        assert "skip" in r.stderr.lower()
```

- [ ] **Run:** `pytest tests/acceptance/test_aggregate_national.py -v` — expect PASS
- [ ] **Commit:** `git commit -m "test(acceptance): redist aggregate + national map L2 tests"`

---

## Board review notes (pre-implementation)

**MERIDIAN:** `InsetProjection` uses equirectangular sub-projections. Alaska at WGS84 coordinates will have significant east-west compression (Anchorage at 61°N, a degree of longitude is ~55km vs ~111km at equator). This is display-only — no metric computation uses these coordinates. Document in `InsetProjection` struct comment.

**CONTOUR:** `state_name_to_code()` must handle multi-word state names with underscores (e.g., `rhode_island` → `RI`). Add a complete 50-state map. Test: `test_state_name_to_code_all_50`.

**BENCHMARK:** The national map load-all-states loop will silently skip states without TIGER shapefiles. Add a counter: emit stderr "X states loaded, Y skipped (no TIGER data)" so the user knows what's in the map.

**TRENCH (PP-new-01): Silent state skip in national map.** If `load_district_geometries` fails for a state (missing TIGER, wrong FIPS, etc.), the national map silently omits that state. Unlike per-state errors which produce a clear failure, the national map may silently show 48 states when the user expects 50. Structural fix: collect all errors, emit a summary warning at the end: "WARNING: 2 states omitted from national map — see above for details."

---

## Running the full national pipeline (after implementation)

```bash
# Step 1: Run all 50 states
redist states --year 2020 --version V3 --output-dir outputs/V3 --workers 8

# Step 2: Run all analytics for all states
for state in AL AK AZ ...; do
  redist analyze --state $state --year 2020 --version V3 --types all
done
# Or: run all at once (future: redist analyze --states all)

# Step 3: Aggregate into national datasets
redist aggregate --version V3 --types all --csv

# Step 4: Render national maps
redist map --scope national --version V3 --types districts political demographic compactness

# Outputs:
# outputs/V3/national/us_demographic.json + .csv
# outputs/V3/national/us_political.json + .csv
# outputs/V3/national/us_compactness.json + .csv
# outputs/V3/national/us_summary.json + .csv
# outputs/V3/national/maps/districts.png
# outputs/V3/national/maps/political.png
# outputs/V3/national/maps/demographic.png
# outputs/V3/national/maps/compactness.png
```
