# `redist analyze` + `redist map` — Full E2E Native Rust Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Full end-to-end analytics and map rendering in Rust. Zero Python subprocesses. `redist analyze` produces per-district JSON for all metrics. `redist map` renders publication-quality PNG district maps with adaptive labels, coloring, and highlighting. `redist-map` is a new crate with an extension model matching `redist-analysis`.

**Architecture:**
- New `redist-map` crate: geometry dissolve → projection → SVG → PNG pipeline
- Updated `redist-analysis` crate: `Analyzer` trait + demographic / political / urban / summary modules
- Updated `redist-cli`: `analyze` and `map` subcommands, both fully dispatched in Rust

**Tech Stack:** `geo` (dissolve, centroid), `polylabel` (visual center), `geo-svg` (path strings), `resvg` + `tiny-skia` (SVG→PNG), `fontdb` (embedded font), `csv` crate, `serde_json`

---

## Labeling system design (lock this in before coding)

### Bisection round maps

At each bisection round, each region on the map will become some number of final districts.
Label format depends on density:

| Round | Region will become | Label | Font size |
|-------|--------------------|-------|-----------|
| Round 0 | 52 districts | `1 (52)` | largest — proportional to whole state |
| Round 1 | 26 districts | `1 (26)` `2 (26)` | large — proportional to half-state |
| Round 2 | 13 districts | `1 (13)` … `4 (13)` | medium |
| Round N | 1 district (final) | `1` `2` … | small, no parenthetical |

**Font size formula:** `font_px = clamp(inscribed_circle_radius_px * 0.4, 6.0, 28.0)`
where `inscribed_circle_radius_px` comes from `polylabel()` — the radius of the largest circle that fits inside the polygon at the label point.

**Fit check:** If `label_char_count * font_px * 0.6 > inscribed_circle_radius_px * 2`, halve font size. If still doesn't fit, show only the district number (no parenthetical).

**Lineage:** In rounds 2+, show parent district ID as a small superscript next to the label: `"₃2"` = district 2 descended from parent 3. Font size = 40% of main label. Position: upper-right corner of main label. Omit if font_px < 8 (too small to render).

### Final district maps

- Label = district number only: `"1"` `"2"` … `"52"`
- Font size: same formula based on `polylabel` radius
- Omit label entirely if polygon inscribed circle diameter < 12px

### Analytical maps (political, demographic, compactness)

Two-line label — number large, statistic smaller:

| Map type | Label example |
|----------|--------------|
| Political | `"5"` + `"D+12%"` |
| Demographic | `"3"` + `"42% min"` |
| Compactness | `"7"` + `"PP: 0.34"` |

Statistic font size = 65% of main number font size. Centered below the number.

### Text halo (contrast against colored backgrounds)

Two overlapping SVG text elements per label:
1. `stroke="white" stroke-width="3" fill="white" paint-order="stroke"` (halo)
2. `fill="#1a1a1a"` (text on top)

This works in resvg's full SVG spec support.

---

## Color schemes

**Categorical (district maps, round maps):** 12-color qualitative palette (ColorBrewer Set3 variant), cycling for states with >12 districts. Adjacent districts guaranteed different colors via graph coloring (greedy 4-color on adjacency graph, fallback to cycle).

**Sequential (choropleth — political, demographic, compactness):**
- Political: Red–White–Blue diverging (Rep → neutral → Dem)
- Demographic: Yellow–Orange–Brown sequential (minority % low → high)
- Compactness: Green sequential (PP low → high)

---

## File Map

| File | Action |
|------|--------|
| `redist/crates/redist-map/Cargo.toml` | **Create** |
| `redist/crates/redist-map/src/lib.rs` | **Create** |
| `redist/crates/redist-map/src/dissolve.rs` | **Create** — union tract polygons into district polygons |
| `redist/crates/redist-map/src/projection.rs` | **Create** — WGS84 bbox → SVG pixel coordinates |
| `redist/crates/redist-map/src/colorscheme.rs` | **Create** — categorical + choropleth palettes |
| `redist/crates/redist-map/src/labeler.rs` | **Create** — adaptive font size, fit check, halo SVG |
| `redist/crates/redist-map/src/renderer.rs` | **Create** — SVG assembly + resvg PNG output |
| `redist/crates/redist-map/src/map_type.rs` | **Create** — `MapSpec` trait + all map type implementations |
| `redist/crates/redist-map/src/rounds.rs` | **Create** — bisection round progression maps |
| `redist/crates/redist-map/assets/LiberationSans-Regular.ttf` | **Add** — embedded open font |
| `redist/crates/redist-analysis/src/analyzer.rs` | **Create** — `Analyzer` trait + `AnalyzerType` enum |
| `redist/crates/redist-analysis/src/demographic.rs` | **Create** |
| `redist/crates/redist-analysis/src/political.rs` | **Create** |
| `redist/crates/redist-analysis/src/urban.rs` | **Create** |
| `redist/crates/redist-analysis/src/summary.rs` | **Create** |
| `redist/crates/redist-analysis/src/lib.rs` | **Modify** — expose new modules |
| `redist/crates/redist-cli/src/analyze.rs` | **Create** — dispatcher |
| `redist/crates/redist-cli/src/map_cmd.rs` | **Create** — `redist map` dispatcher |
| `redist/crates/redist-cli/src/args.rs` | **Modify** — `AnalyzeArgs`, `MapArgs` |
| `redist/crates/redist-cli/src/main.rs` | **Modify** — wire Commands |
| `redist/Cargo.toml` | **Modify** — add `redist-map` to workspace |
| `tests/unit/test_map_engine.py` | **Create** — L0/L1 PyO3-accessible tests |
| `tests/acceptance/test_analyze_map_acceptance.py` | **Create** — L2 end-to-end |

---

## Task 1: `redist-map` crate scaffold + projection

**Files:** `redist/crates/redist-map/Cargo.toml`, `src/lib.rs`, `src/projection.rs`

- [ ] **Create `Cargo.toml`**

```toml
[package]
name = "redist-map"
version = "0.1.0"
edition = "2021"

[dependencies]
geo = { version = "0.28", features = ["use-serde"] }
geo-types = "0.7"
geo-svg = "0.2"
polylabel = "2.0"
resvg = "0.44"
tiny-skia = "0.11"
fontdb = "0.22"
usvg = "0.44"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1"
```

- [ ] **L0: Write failing projection tests**

```rust
// src/projection.rs tests
#[test]
fn test_unit_bbox_maps_to_full_canvas() {
    // A bbox of exactly [0,0]→[1,1] should map to [0,0]→[500,500]
    let proj = Projection::from_bbox(0.0, 0.0, 1.0, 1.0, 500, 500, 0.05);
    let (x, y) = proj.project(0.0, 0.0);
    assert!((x - 25.0).abs() < 1.0); // 5% padding = 25px
    let (x2, y2) = proj.project(1.0, 1.0);
    assert!((x2 - 475.0).abs() < 1.0);
    assert!((y2 - 25.0).abs() < 1.0); // y flipped: lat max → SVG y=padding
}

#[test]
fn test_lon_lat_monotone() {
    let proj = Projection::from_bbox(-80.0, 35.0, -70.0, 45.0, 800, 600, 0.05);
    // More east (higher lon) = higher x
    let (x1, _) = proj.project(-79.0, 40.0);
    let (x2, _) = proj.project(-71.0, 40.0);
    assert!(x2 > x1);
    // More north (higher lat) = lower y (SVG y flipped)
    let (_, y1) = proj.project(-75.0, 36.0);
    let (_, y2) = proj.project(-75.0, 44.0);
    assert!(y2 < y1);
}

#[test]
fn test_aspect_ratio_preserved() {
    // 2:1 bbox into square canvas → letterboxed
    let proj = Projection::from_bbox(0.0, 0.0, 2.0, 1.0, 500, 500, 0.0);
    // Width of mapped content should be ~500, height ~250 (centered)
    let (x0, _) = proj.project(0.0, 0.5);
    let (x1, _) = proj.project(2.0, 0.5);
    assert!((x1 - x0 - 500.0).abs() < 2.0);
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `Projection`**

```rust
pub struct Projection {
    x_offset: f64, y_offset: f64,
    scale: f64,
    canvas_h: f64,
}

impl Projection {
    /// Fit bbox into canvas with padding. Preserves aspect ratio (letterbox).
    pub fn from_bbox(
        min_lon: f64, min_lat: f64, max_lon: f64, max_lat: f64,
        canvas_w: u32, canvas_h: u32,
        padding_frac: f64,
    ) -> Self { ... }

    /// Project (lon, lat) → (svg_x, svg_y). SVG y-axis is flipped.
    pub fn project(&self, lon: f64, lat: f64) -> (f64, f64) { ... }

    /// Project a geo_types::Coord slice → Vec<(f64,f64)>
    pub fn project_coords(&self, coords: &[geo_types::Coord]) -> Vec<(f64, f64)> { ... }
}
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(map): projection — WGS84 bbox to SVG pixel coords"`

---

## Task 2: Color schemes

**Files:** `src/colorscheme.rs`

- [ ] **L0: Write failing color tests**

```rust
#[test]
fn test_categorical_12_colors_all_distinct() {
    let scheme = CategoricalScheme::default();
    let colors: Vec<_> = (0..12).map(|i| scheme.color(i)).collect();
    // All 12 must be distinct
    let unique: std::collections::HashSet<_> = colors.iter().collect();
    assert_eq!(unique.len(), 12);
}

#[test]
fn test_categorical_cycles_after_12() {
    let scheme = CategoricalScheme::default();
    assert_eq!(scheme.color(0), scheme.color(12));
    assert_eq!(scheme.color(3), scheme.color(15));
}

#[test]
fn test_choropleth_political_dem_end_is_blue() {
    let scheme = PoliticalScheme;
    let (r, g, b) = scheme.color(1.0); // 100% Dem
    assert!(b > r && b > g, "strong Dem should be blue-dominant");
}

#[test]
fn test_choropleth_political_rep_end_is_red() {
    let scheme = PoliticalScheme;
    let (r, g, b) = scheme.color(0.0); // 0% Dem = 100% Rep
    assert!(r > b && r > g, "strong Rep should be red-dominant");
}

#[test]
fn test_choropleth_midpoint_is_near_white() {
    let scheme = PoliticalScheme;
    let (r, g, b) = scheme.color(0.5); // toss-up
    // All channels should be close to 255
    assert!(r > 200 && g > 200 && b > 200);
}

#[test]
fn test_graph_coloring_adjacent_districts_differ() {
    // Triangle graph: 1-2, 2-3, 1-3
    let adjacency = vec![vec![1, 2], vec![0, 2], vec![0, 1]];
    let colors = graph_color(&adjacency, &CategoricalScheme::default());
    assert_ne!(colors[0], colors[1]);
    assert_ne!(colors[1], colors[2]);
    assert_ne!(colors[0], colors[2]);
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement**

```rust
/// 12-color qualitative palette (ColorBrewer Set3, web-safe hex)
pub struct CategoricalScheme {
    colors: [(u8,u8,u8); 12],
}
impl CategoricalScheme {
    pub fn color(&self, district_id: usize) -> (u8,u8,u8) {
        self.colors[district_id % 12]
    }
}

pub struct PoliticalScheme;
impl PoliticalScheme {
    /// t=0.0 → Rep red, t=0.5 → white, t=1.0 → Dem blue
    pub fn color(&self, dem_frac: f64) -> (u8,u8,u8) { ... }
}

pub struct DemographicScheme;  // Yellow → Brown sequential
pub struct CompactnessScheme;  // Green sequential

/// Greedy graph 4-coloring. Returns color_index per district.
pub fn graph_color(adjacency: &[Vec<usize>], scheme: &CategoricalScheme) -> Vec<(u8,u8,u8)> {
    let n = adjacency.len();
    let mut assigned = vec![usize::MAX; n];
    for i in 0..n {
        let used: std::collections::HashSet<usize> = adjacency[i].iter()
            .filter(|&&j| assigned[j] != usize::MAX)
            .map(|&j| assigned[j])
            .collect();
        assigned[i] = (0..).find(|c| !used.contains(c)).unwrap();
    }
    assigned.iter().map(|&c| scheme.color(c)).collect()
}
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(map): color schemes — categorical + choropleth + graph coloring"`

---

## Task 3: Adaptive labeler

**Files:** `src/labeler.rs`

- [ ] **L0: Write failing label tests**

```rust
#[test]
fn test_font_size_large_polygon() {
    // Inscribed circle radius 200px → font ~= 80px, clamped to 28
    let size = adaptive_font_size(200.0);
    assert_eq!(size, 28.0);
}

#[test]
fn test_font_size_small_polygon() {
    // Inscribed circle radius 10px → font ~= 4px, clamped to 6
    let size = adaptive_font_size(10.0);
    assert_eq!(size, 6.0);
}

#[test]
fn test_font_size_medium_polygon() {
    // radius 60px → font = 60*0.4 = 24px
    let size = adaptive_font_size(60.0);
    assert!((size - 24.0).abs() < 0.1);
}

#[test]
fn test_label_fits_check_simple() {
    // label "1 (26)" = 6 chars, font 20px → width ~= 72px
    // inscribed radius 50px → diameter 100px → fits
    assert!(label_fits("1 (26)", 20.0, 50.0));
    // inscribed radius 20px → diameter 40px → doesn't fit (72 > 40)
    assert!(!label_fits("1 (26)", 20.0, 20.0));
}

#[test]
fn test_round_label_early_round() {
    // Round 1, region will become 26 districts
    let lbl = round_label(1, 26, 26);
    assert_eq!(lbl.main, "1");
    assert_eq!(lbl.annotation, Some("26".into()));
}

#[test]
fn test_round_label_final_round() {
    // Final round: 1 district in this region
    let lbl = round_label(3, 1, 52);
    assert_eq!(lbl.main, "3");
    assert_eq!(lbl.annotation, None);
}

#[test]
fn test_round_label_lineage() {
    // District 7, parent was district 3 → superscript "₃"
    let lbl = round_label_with_lineage(7, 1, 52, Some(3));
    assert_eq!(lbl.lineage_superscript, Some(3));
}

#[test]
fn test_political_label_dem() {
    let lbl = political_label(5, 0.67);
    assert_eq!(lbl.main, "5");
    assert_eq!(lbl.stat, "D+34%");
}

#[test]
fn test_political_label_rep() {
    let lbl = political_label(2, 0.38);
    assert_eq!(lbl.main, "2");
    assert_eq!(lbl.stat, "R+24%");
}

#[test]
fn test_political_label_tossup() {
    let lbl = political_label(1, 0.501);
    assert_eq!(lbl.stat, "D+0%");
}

#[test]
fn test_demographic_label() {
    let lbl = demographic_label(3, 0.423);
    assert_eq!(lbl.stat, "42% min");
}

#[test]
fn test_compactness_label() {
    let lbl = compactness_label(7, 0.3456);
    assert_eq!(lbl.stat, "PP: 0.35");
}

#[test]
fn test_halo_svg_has_two_text_elements() {
    let svg = halo_text_svg(250.0, 300.0, "5", "D+12%", 18.0);
    // Must contain exactly 2 <text> elements (halo + foreground)
    assert_eq!(svg.matches("<text").count(), 2);
    // Halo has white stroke
    assert!(svg.contains("stroke=\"white\""));
    // Foreground has dark fill
    assert!(svg.contains("fill=\"#1a1a1a\""));
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement**

```rust
pub struct LabelSpec {
    pub main: String,
    pub annotation: Option<String>,  // "(26)" for round maps
    pub stat: Option<String>,         // "D+12%" for choropleth
    pub lineage_superscript: Option<usize>, // parent district
}

/// font_px = clamp(radius * 0.4, 6.0, 28.0)
pub fn adaptive_font_size(inscribed_radius_px: f64) -> f64 {
    (inscribed_radius_px * 0.4).clamp(6.0, 28.0)
}

/// Estimate label width and check against inscribed circle diameter.
pub fn label_fits(text: &str, font_px: f64, inscribed_radius_px: f64) -> bool {
    let estimated_width = text.chars().count() as f64 * font_px * 0.6;
    estimated_width <= inscribed_radius_px * 2.0
}

pub fn round_label(district_id: usize, districts_in_region: usize, _total: usize) -> LabelSpec {
    LabelSpec {
        main: district_id.to_string(),
        annotation: if districts_in_region > 1 {
            Some(districts_in_region.to_string())
        } else { None },
        stat: None, lineage_superscript: None,
    }
}

pub fn political_label(district: usize, dem_frac: f64) -> LabelSpec {
    let margin = ((dem_frac - 0.5) * 200.0).round() as i32;
    let stat = if margin >= 0 {
        format!("D+{}%", margin)
    } else {
        format!("R+{}%", -margin)
    };
    LabelSpec { main: district.to_string(), annotation: None, stat: Some(stat), lineage_superscript: None }
}

/// Generate two-element SVG halo text (white stroke + dark fill).
pub fn halo_text_svg(cx: f64, cy: f64, main: &str, stat: &str, font_px: f64) -> String { ... }
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(map): adaptive labeler — font sizing, fit check, halo SVG"`

---

## Task 4: Geometry dissolve

**Files:** `src/dissolve.rs`

Dissolve tract polygons into district polygons using `geo::BooleanOps::union`.

- [ ] **L0: Write failing dissolve tests (pure geometry, no I/O)**

```rust
#[test]
fn test_dissolve_two_adjacent_squares() {
    // Two adjacent unit squares → one 2×1 rectangle
    let sq1: Polygon = polygon![(x:0.,y:0.),(x:1.,y:0.),(x:1.,y:1.),(x:0.,y:1.)];
    let sq2: Polygon = polygon![(x:1.,y:0.),(x:2.,y:0.),(x:2.,y:1.),(x:1.,y:1.)];
    let merged = dissolve_polygons(&[sq1, sq2]);
    // Area should be 2.0
    use geo::Area;
    assert!((merged.unsigned_area() - 2.0).abs() < 1e-9);
}

#[test]
fn test_dissolve_group_by_district() {
    // 4 tracts: tracts 0,1 → district 1; tracts 2,3 → district 2
    let tracts = vec![
        polygon![(x:0.,y:0.),(x:1.,y:0.),(x:1.,y:1.),(x:0.,y:1.)],
        polygon![(x:1.,y:0.),(x:2.,y:0.),(x:2.,y:1.),(x:1.,y:1.)],
        polygon![(x:0.,y:1.),(x:1.,y:1.),(x:1.,y:2.),(x:0.,y:2.)],
        polygon![(x:1.,y:1.),(x:2.,y:1.),(x:2.,y:2.),(x:1.,y:2.)],
    ];
    let assignments = vec![1usize, 1, 2, 2]; // tract_idx → district
    let districts = group_dissolve(&tracts, &assignments, 2);
    assert_eq!(districts.len(), 2);
    use geo::Area;
    assert!((districts[&1].unsigned_area() - 2.0).abs() < 1e-9);
    assert!((districts[&2].unsigned_area() - 2.0).abs() < 1e-9);
}

#[test]
fn test_dissolve_single_polygon_unchanged() {
    let p: Polygon = polygon![(x:0.,y:0.),(x:3.,y:0.),(x:3.,y:2.),(x:0.,y:2.)];
    let merged = dissolve_polygons(&[p.clone()]);
    use geo::Area;
    assert!((merged.unsigned_area() - p.unsigned_area()).abs() < 1e-9);
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement**

```rust
use geo::{BooleanOps, MultiPolygon, Polygon};
use std::collections::HashMap;

/// Union all polygons in a slice into one MultiPolygon.
pub fn dissolve_polygons(polys: &[Polygon]) -> MultiPolygon {
    if polys.is_empty() { return MultiPolygon(vec![]); }
    let mut acc: MultiPolygon = MultiPolygon(vec![polys[0].clone()]);
    for p in &polys[1..] {
        acc = acc.union(&MultiPolygon(vec![p.clone()]));
    }
    acc
}

/// Group tracts by district assignment, dissolve each group.
/// Returns HashMap<district_id, MultiPolygon>.
pub fn group_dissolve(
    tract_polys: &[Polygon],
    assignments: &[usize],   // parallel to tract_polys
    num_districts: usize,
) -> HashMap<usize, MultiPolygon> {
    let mut groups: HashMap<usize, Vec<Polygon>> = HashMap::new();
    for (i, &dist) in assignments.iter().enumerate() {
        groups.entry(dist).or_default().push(tract_polys[i].clone());
    }
    groups.into_iter()
        .map(|(dist, polys)| (dist, dissolve_polygons(&polys)))
        .collect()
}
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(map): polygon dissolve — group_dissolve + union"`

---

## Task 5: SVG renderer + PNG output

**Files:** `src/renderer.rs`, `assets/LiberationSans-Regular.ttf`

- [ ] **Download Liberation Sans Regular** (OFL licensed, ~70KB) and save to `redist/crates/redist-map/assets/LiberationSans-Regular.ttf`

- [ ] **L0: Write failing renderer tests**

```rust
#[test]
fn test_svg_polygon_path_is_closed() {
    // A square projected to pixel coords must produce a closed SVG path
    let pts = vec![(10.0,10.0),(90.0,10.0),(90.0,90.0),(10.0,90.0)];
    let path = coords_to_svg_path(&pts);
    assert!(path.starts_with('M'));
    assert!(path.ends_with('Z'));
}

#[test]
fn test_svg_polygon_has_correct_fill() {
    let color = (255u8, 128u8, 0u8);
    let svg_elem = polygon_svg_element(&[(0.,0.),(100.,0.),(100.,100.),(0.,100.)], color, 0.85);
    assert!(svg_elem.contains("fill=\"rgb(255,128,0)\""));
    assert!(svg_elem.contains("opacity=\"0.85\""));
}

#[test]
fn test_svg_stroke_on_boundary() {
    let svg_elem = polygon_svg_element(&[(0.,0.),(100.,0.),(100.,100.),(0.,100.)], (200,200,200), 1.0);
    assert!(svg_elem.contains("stroke="));
}
```

- [ ] **L1: Write integration test (in-memory render)**

```rust
#[test]
fn test_render_two_district_map_produces_nonblank_png() {
    // Two adjacent squares → 2 districts, render to PNG bytes
    use geo_types::polygon;
    let d1: MultiPolygon = MultiPolygon(vec![polygon![(x:0.,y:0.),(x:1.,y:0.),(x:1.,y:1.),(x:0.,y:1.)]]);
    let d2: MultiPolygon = MultiPolygon(vec![polygon![(x:1.,y:0.),(x:2.,y:0.),(x:2.,y:1.),(x:1.,y:1.)]]);
    let districts = vec![(1usize, d1, (141u8,211u8,199u8)), (2, d2, (255,255,179))];

    let png_bytes = render_district_map(&districts, 400, 300, &default_font_db()).unwrap();

    assert!(!png_bytes.is_empty());
    assert!(png_bytes.len() > 5000, "PNG should be substantial (not blank)");
    // First 8 bytes = PNG magic
    assert_eq!(&png_bytes[0..4], b"\x89PNG");
}

#[test]
fn test_render_with_labels_embeds_text() {
    // Same map but with labels — SVG intermediate must contain <text>
    let d1: MultiPolygon = MultiPolygon(vec![polygon![(x:0.,y:0.),(x:10.,y:0.),(x:10.,y:10.),(x:0.,y:10.)]]);
    let spec = LabelSpec { main: "1".into(), annotation: None, stat: None, lineage_superscript: None };
    let svg = build_svg(
        &[(1, d1, (100u8,200u8,100u8), spec)],
        &Projection::from_bbox(0.,0.,10.,10.,400,300,0.05),
        400, 300,
    );
    assert!(svg.contains("<text"), "SVG must contain at least one text element");
}
```

- [ ] **Run L0 + L1** — expect FAIL
- [ ] **Implement `renderer.rs`**

```rust
// Key functions:
pub fn coords_to_svg_path(pts: &[(f64, f64)]) -> String { ... }
pub fn polygon_svg_element(pts: &[(f64,f64)], fill: (u8,u8,u8), opacity: f64) -> String { ... }
pub fn build_svg(districts: &[(usize, MultiPolygon, (u8,u8,u8), LabelSpec)],
                 proj: &Projection, w: u32, h: u32) -> String { ... }
pub fn svg_to_png(svg: &str, font_db: &fontdb::Database) -> anyhow::Result<Vec<u8>> { ... }

// Font loading — called once, cached in CLI runner:
pub fn default_font_db() -> fontdb::Database {
    let mut db = fontdb::Database::new();
    db.load_font_data(include_bytes!("../assets/LiberationSans-Regular.ttf").to_vec());
    db
}
```

- [ ] **Run L0 + L1** — expect PASS
- [ ] **Commit:** `git commit -m "feat(map): SVG renderer + PNG output via resvg"`

---

## Task 6: Bisection round maps

**Files:** `src/rounds.rs`

Round maps show the bisection progression. Each PNG in the sequence shows the state at that round with adaptive labels showing lineage.

- [ ] **L0: Write failing round map tests**

```rust
#[test]
fn test_round_region_counts() {
    // State with 4 districts, 3 rounds: [4] → [2,2] → [1,1,1,1]
    let tree = BisectionTree {
        rounds: vec![
            vec![(0, vec![0,1,2,3])],      // round 0: all 4 tracts in region 0
            vec![(0, vec![0,1]), (1, vec![2,3])],    // round 1: split
            vec![(0,vec![0]),(1,vec![1]),(2,vec![2]),(3,vec![3])], // round 2: final
        ]
    };
    let counts = districts_in_region(&tree, 0, 0); // round 0, region 0
    assert_eq!(counts, 4);
    let counts_r1 = districts_in_region(&tree, 1, 0); // round 1, region 0
    assert_eq!(counts_r1, 2);
}

#[test]
fn test_parent_region_lookup() {
    // In round 1, region 0 spawned from round-0 region 0
    // In round 2, region 1 spawned from round-1 region 0
    let lineage = build_lineage(&sample_tree());
    assert_eq!(lineage.parent_of(1, 1), Some(0)); // round 1 region 1's parent is round 0 region 0
}

#[test]
fn test_round_label_with_lineage_none_at_round0() {
    // Round 0 has no parent
    let lbl = round_label_with_lineage(1, 4, 4, None);
    assert_eq!(lbl.lineage_superscript, None);
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `BisectionTree`, `build_lineage`, `render_round_maps()`**

```rust
/// The full bisection hierarchy: rounds[r][i] = (region_id, tract_indices)
pub struct BisectionTree {
    pub rounds: Vec<Vec<(usize, Vec<usize>)>>,
}

/// Render one PNG per round. Returns Vec<(round, png_bytes)>.
pub fn render_round_maps(
    tree: &BisectionTree,
    tract_polys: &[Polygon<f64>],
    canvas_w: u32,
    canvas_h: u32,
    font_db: &fontdb::Database,
) -> anyhow::Result<Vec<(usize, Vec<u8>)>> { ... }
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(map): bisection round progression maps with lineage labels"`

---

## Task 7: Analytics — `Analyzer` trait + all four modules

**Files:** `redist-analysis/src/analyzer.rs`, `demographic.rs`, `political.rs`, `urban.rs`, `summary.rs`

- [ ] **L0 tests — `analyzer.rs`**

```rust
#[test]
fn test_analyzer_type_all_variants_parse() {
    use clap::ValueEnum;
    let variants = AnalyzerType::value_variants();
    assert!(variants.iter().any(|v| *v == AnalyzerType::Demographic));
    assert!(variants.iter().any(|v| *v == AnalyzerType::Political));
    assert!(variants.iter().any(|v| *v == AnalyzerType::Summary));
    assert!(variants.iter().any(|v| *v == AnalyzerType::All));
}
```

- [ ] **L0 tests — `demographic.rs`**

```rust
#[test]
fn test_demographic_aggregation_two_tracts_one_district() {
    let rows = vec![
        make_demo_row("50001", 1000, 800, 100, 0, 100, 0),
        make_demo_row("50002", 500, 400, 50, 0, 50, 0),
    ];
    let assignments = hashmap!{"50001" => 1, "50002" => 1};
    let result = aggregate_demographic(&rows, &assignments, 1);
    let d = &result.districts[0];
    assert_eq!(d.total_pop, 1500);
    assert!((d.pct_white - 0.8).abs() < 1e-6);
    assert!((d.pct_minority - 0.2).abs() < 1e-6);
}

#[test]
fn test_demographic_unmatched_geoid_ignored() {
    let rows = vec![make_demo_row("99999", 1000, 800, 200, 0, 0, 0)];
    let assignments = hashmap!{"50001" => 1};
    let result = aggregate_demographic(&rows, &assignments, 1);
    assert_eq!(result.districts[0].total_pop, 0);
}

#[test]
fn test_demographic_two_districts() {
    let rows = vec![
        make_demo_row("50001", 600, 600, 0, 0, 0, 0),
        make_demo_row("50002", 400, 0, 400, 0, 0, 0),
    ];
    let assignments = hashmap!{"50001" => 1, "50002" => 2};
    let result = aggregate_demographic(&rows, &assignments, 2);
    assert!((result.districts[0].pct_white - 1.0).abs() < 1e-6);
    assert!((result.districts[1].pct_black - 1.0).abs() < 1e-6);
}
```

- [ ] **L0 tests — `political.rs`**

```rust
#[test]
fn test_political_dem_district() {
    let rows = vec![make_pol_row("50001", 700.0, 300.0)];
    let assignments = hashmap!{"50001" => 1};
    let r = aggregate_political(&rows, &assignments, 1);
    let d = &r.districts[0];
    assert!((d.dem_pct - 0.7).abs() < 1e-6);
    assert!((d.margin - 0.4).abs() < 1e-6);
    assert!(d.lean_dem);
}

#[test]
fn test_political_rep_district() {
    let rows = vec![make_pol_row("50001", 300.0, 700.0)];
    let assignments = hashmap!{"50001" => 1};
    let r = aggregate_political(&rows, &assignments, 1);
    assert!(!r.districts[0].lean_dem);
    assert!((r.districts[0].margin - (-0.4)).abs() < 1e-6);
}

#[test]
fn test_political_missing_year_returns_empty() {
    // If CSV path doesn't exist, return empty result (not an error)
    let ctx = make_ctx_no_files("VT", "2010"); // 2010 election data absent
    let r = PoliticalAnalyzer::run(&ctx);
    assert!(r.is_ok());
    assert!(r.unwrap().districts.is_empty());
}
```

- [ ] **L0 tests — `urban.rs`** (see Task 4 of original plan)
- [ ] **L0 tests — `summary.rs`**

```rust
#[test]
fn test_summary_merges_all_present() {
    let demo = make_demo_district(1, 1000, 0.9, 0.05, 0.02, 0.03);
    let pol  = make_pol_district(1, 0.55, 0.1, true);
    let urb  = make_urb_district(1, Some("Burlington"), 45000);
    let s = merge_district(1, Some(&demo), Some(&pol), None, Some(&urb));
    assert_eq!(s.district, 1);
    assert!((s.dem_pct.unwrap() - 0.55).abs() < 1e-6);
    assert_eq!(s.largest_city.as_deref(), Some("Burlington"));
    assert!(s.polsby_popper.is_none()); // compactness not provided
}

#[test]
fn test_summary_partial_inputs_dont_panic() {
    // Only demographic provided
    let demo = make_demo_district(1, 800, 0.8, 0.1, 0.05, 0.05);
    let s = merge_district(1, Some(&demo), None, None, None);
    assert!(s.dem_pct.is_none());
    assert!(s.largest_city.is_none());
    assert_eq!(s.total_pop, Some(800));
}
```

- [ ] **Implement all four analyzer modules** following the `Analyzer` trait
- [ ] **Run all L0 tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(analysis): Analyzer trait + demographic/political/urban/summary"`

---

## Task 8: `redist analyze` + `redist map` CLI wiring

**Files:** `redist-cli/src/analyze.rs`, `map_cmd.rs`, `args.rs`, `main.rs`

- [ ] **Add `Commands::Analyze` and `Commands::Map` to `args.rs`** (see plan header for struct definitions)

- [ ] **L0 test — CLI args parse correctly**

```rust
#[test]
fn test_analyze_defaults() {
    let args = AnalyzeArgs::parse_from(["analyze", "--state", "VT"]);
    assert_eq!(args.state, "VT");
    assert_eq!(args.year, Year::Y2020);
    assert_eq!(args.version, "v1");
    assert!(!args.force);
    // default types = [All]
    assert!(args.types.contains(&AnalyzerType::All));
}

#[test]
fn test_map_types_parse() {
    let args = MapArgs::parse_from([
        "map", "--state", "VT", "--types", "districts", "rounds"
    ]);
    assert!(args.types.contains(&MapType::Districts));
    assert!(args.types.contains(&MapType::Rounds));
}
```

- [ ] **Implement `analyze.rs` dispatcher** — resolves `All` → all variants, runs each, writes `{analysis_dir}/{type}.json` atomically
- [ ] **Implement `map_cmd.rs` dispatcher** — loads tract geometries from TIGER, loads assignments, dissolves, projects, renders, writes PNGs
- [ ] **Wire into `main.rs`**
- [ ] **Run:** `cargo build --release` — expect success
- [ ] **Commit:** `git commit -m "feat(cli): redist analyze + redist map subcommands wired"`

---

## Task 9: L2 acceptance tests (end-to-end)

**Files:** `tests/acceptance/test_analyze_map_acceptance.py`

- [ ] **Write all acceptance tests**

```python
class TestAnalyzeAcceptance(unittest.TestCase):
    """L2: full binary invocation against real VT data."""

    def test_demographic_json_structure(self):
        run(["redist", "analyze", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "demographic"])
        data = load_json("outputs/V3/2020/vermont/analysis/demographic.json")
        assert "districts" in data
        d = data["districts"][0]
        assert d["district"] == 1
        assert d["total_pop"] > 500_000  # VT pop ~643K
        assert 0.0 < d["pct_white"] < 1.0
        assert abs(d["pct_white"] + d["pct_black"] + d["pct_asian"] +
                   d["pct_hispanic"] + d["pct_other"] - 1.0) < 0.01

    def test_political_json_structure(self):
        run(["redist", "analyze", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "political"])
        data = load_json("outputs/V3/2020/vermont/analysis/political.json")
        d = data["districts"][0]
        assert abs(d["dem_pct"] + d["rep_pct"] - 1.0) < 0.01
        assert isinstance(d["lean_dem"], bool)

    def test_all_types_produce_all_files(self):
        run(["redist", "analyze", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "all"])
        base = Path("outputs/V3/2020/vermont/analysis")
        for name in ["demographic.json", "political.json", "urban.json", "summary.json"]:
            assert (base / name).exists(), f"missing {name}"

    def test_skip_without_force(self):
        run(["redist", "analyze", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "demographic"])
        path = Path("outputs/V3/2020/vermont/analysis/demographic.json")
        mtime1 = path.stat().st_mtime
        time.sleep(0.05)
        run(["redist", "analyze", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "demographic"])
        assert path.stat().st_mtime == mtime1  # unchanged

    def test_force_regenerates(self):
        run(["redist", "analyze", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "demographic"])
        path = Path("outputs/V3/2020/vermont/analysis/demographic.json")
        mtime1 = path.stat().st_mtime
        time.sleep(0.05)
        run(["redist", "analyze", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "demographic", "--force"])
        assert path.stat().st_mtime > mtime1


class TestMapAcceptance(unittest.TestCase):
    """L2: map rendering — checks PNG existence, size, validity."""

    def test_districts_map_vt_produces_valid_png(self):
        run(["redist", "map", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "districts"])
        png = Path("outputs/V3/2020/vermont/maps/districts.png")
        assert png.exists()
        assert png.stat().st_size > 20_000  # meaningful PNG
        assert png.read_bytes()[:4] == b"\x89PNG"

    def test_rounds_maps_vt_one_per_round(self):
        run(["redist", "map", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "rounds"])
        rounds_dir = Path("outputs/V3/2020/vermont/maps/rounds")
        pngs = sorted(rounds_dir.glob("round_*.png"))
        # VT = 1 district, so only round_00 (trivial)
        assert len(pngs) >= 1
        for p in pngs:
            assert p.stat().st_size > 5_000

    def test_political_map_produces_png(self):
        run(["redist", "map", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "political"])
        png = Path("outputs/V3/2020/vermont/maps/political.png")
        assert png.exists() and png.stat().st_size > 10_000

    def test_demographic_map_produces_png(self):
        run(["redist", "map", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "demographic"])
        png = Path("outputs/V3/2020/vermont/maps/demographic.png")
        assert png.exists()

    def test_al_districts_map_52_labeled(self):
        # Alabama has 7 districts — check that map generates for multi-district state
        run(["redist", "map", "--state", "AL", "--year", "2020",
             "--version", "V3", "--types", "districts"])
        png = Path("outputs/V3/2020/alabama/maps/districts.png")
        assert png.exists() and png.stat().st_size > 50_000

    def test_dpi_flag_affects_output_size(self):
        run(["redist", "map", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "districts", "--dpi", "72"])
        run(["redist", "map", "--state", "VT", "--year", "2020",
             "--version", "V3", "--types", "districts", "--dpi", "300",
             "--output-suffix", "_hires"])
        lo = Path("outputs/V3/2020/vermont/maps/districts.png").stat().st_size
        hi = Path("outputs/V3/2020/vermont/maps/districts_hires.png").stat().st_size
        assert hi > lo, "300 DPI must produce larger file than 72 DPI"
```

- [ ] **Run:** `pytest tests/acceptance/test_analyze_map_acceptance.py -v` — expect PASS
- [ ] **Commit:** `git commit -m "test(acceptance): redist analyze + redist map L2 tests"`

---

## Extension model: adding a new analyzer

To add `maup` (MAUP sensitivity) analyzer:

1. Create `redist-analysis/src/maup.rs` implementing `Analyzer` trait
2. Add `Maup` variant to `AnalyzerType` enum in `analyzer.rs`  
3. Add one match arm in `redist-cli/src/analyze.rs` dispatcher
4. Expose in `lib.rs`

**3 files, zero changes to `main.rs` or `args.rs`** (clap picks up new enum variant automatically).

---

## Extension model: adding a new map type

To add a `compactness` choropleth map:

1. Implement `render_compactness_map()` in `redist-map/src/map_type.rs`
2. Add `Compactness` variant to `MapType` enum in `args.rs`
3. Add one match arm in `redist-cli/src/map_cmd.rs`

**3 files** — same pattern as analyzers.

---

## Why native Rust maps (not Python subprocess)

- `tiger.rs` already reads TIGER shapefiles and returns `geometry_wkb: Vec<u8>` per tract — geometry is available
- `geo-svg` converts `geo_types::MultiPolygon` to SVG path strings — polygon rendering is solved
- `resvg` renders full SVG spec to PNG — text, stroke, opacity, labels all supported
- `polylabel` finds the best label placement point (pole of inaccessibility) — maps look right
- `fontdb` embeds Liberation Sans at compile time — consistent fonts, no system dependency
- The entire pipeline has zero external C dependencies (no GDAL, no PROJ, no cairo)
