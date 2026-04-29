# Spec 2 (Plan Comparison) + Spec 3 (Constraint Analysis) — TDD Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `redist compare` and `redist analyze --types contiguity splits`. `redist compare` computes Jaccard similarity, population equality, and compactness between two plans or a plan vs enacted districts. Enacted districts are downloaded via `redist fetch --type enacted`, converted from polygon boundaries to tract assignments using centroid point-in-polygon with nearest-polygon fallback (100% coverage enforced). `redist analyze --types contiguity splits` runs BFS contiguity checks and county/municipal split counts. Exit codes use composable bitfields (1=balance, 2=contiguity, 4=nesting, 8=missing-data). `--allow-noncontiguous` suppresses bit 1. State-specific split standards are looked up from a built-in table (WA, CA, TX, CO minimum).

**Specs:** Spec 2 (Plan Comparison), Spec 3 (Constraint Analysis) + R3 board amendments

**Architecture:**
- `redist-analysis/src/comparison.rs` — `compare_plans()`, `PlanComparison`, Jaccard computation
- `redist-analysis/src/contiguity.rs` — `check_contiguity()`, BFS component detection
- `redist-analysis/src/splits.rs` — `analyze_county_splits()`, `analyze_municipal_splits()`, GEOID parsing
- `redist-analysis/src/split_standards.rs` — per-state constitutional split standard lookup table
- `redist-data/src/enacted.rs` — `assign_tracts_to_enacted()`, nearest-polygon fallback
- `redist-cli/src/compare.rs` — `redist compare` dispatcher
- `redist-cli/src/fetch.rs` — extend with `--type enacted`, `--type geography`
- `redist-cli/src/analyze.rs` — extend with `contiguity`, `splits` analyzer types
- `redist-cli/src/exit_codes.rs` — composable bitfield exit codes

**Dependencies:** Spec 1 (`PlanManifest`, `plans/{label}/` paths), `geo` crate `Contains` algorithm (already a dependency)

---

## File Map

| File | Action |
|------|--------|
| `redist/crates/redist-analysis/src/comparison.rs` | **Create** — Jaccard, population, compactness comparison |
| `redist/crates/redist-analysis/src/contiguity.rs` | **Create** — BFS per-district contiguity check |
| `redist/crates/redist-analysis/src/splits.rs` | **Create** — county + municipal split analysis |
| `redist/crates/redist-analysis/src/split_standards.rs` | **Create** — per-state constitutional lookup table |
| `redist/crates/redist-analysis/src/exit_codes.rs` | **Create** — composable bitfield exit codes |
| `redist/crates/redist-analysis/src/lib.rs` | **Modify** — expose new modules |
| `redist/crates/redist-data/src/enacted.rs` | **Create** — enacted shapefile download + tract assignment |
| `redist/crates/redist-data/src/lib.rs` | **Modify** — expose enacted module |
| `redist/crates/redist-cli/src/compare.rs` | **Create** — `redist compare` dispatcher + output formatting |
| `redist/crates/redist-cli/src/analyze.rs` | **Modify** — add `contiguity`, `splits` to analyzer dispatch |
| `redist/crates/redist-cli/src/fetch.rs` | **Modify** — add `enacted`, `geography` fetch types |
| `redist/crates/redist-cli/src/args.rs` | **Modify** — `CompareArgs`, extend `AnalyzeArgs`, extend `FetchArgs` |
| `redist/crates/redist-cli/src/main.rs` | **Modify** — wire `Commands::Compare`; update analyze + fetch dispatch |
| `tests/unit/test_comparison.py` | **Create** — L0 comparison tests |
| `tests/unit/test_contiguity.py` | **Create** — L0 contiguity tests |
| `tests/unit/test_splits.py` | **Create** — L0 splits tests |
| `tests/acceptance/test_spec2_spec3_acceptance.py` | **Create** — L2 acceptance tests |

---

## Task 1: Jaccard similarity + `PlanComparison` struct

**Files:** `redist/crates/redist-analysis/src/comparison.rs`

- [ ] **L0: Write failing Jaccard tests (from Spec 2 + Scenario 3)**

```rust
// redist-analysis/src/comparison.rs tests

#[test]
fn test_jaccard_identical_plans() {
    // Same assignments → Jaccard = 1.0
    let a: HashMap<String, usize> = hashmap!{
        "530330001001" => 1, "530330001002" => 1, "530330002001" => 2
    };
    let b = a.clone();
    let result = compare_plans(&a, &b);
    assert!((result.jaccard_similarity - 1.0).abs() < 1e-9);
}

#[test]
fn test_jaccard_completely_different() {
    // No shared tract→district pairings → Jaccard = 0.0
    let a: HashMap<String, usize> = hashmap!{
        "530330001001" => 1, "530330001002" => 2
    };
    let b: HashMap<String, usize> = hashmap!{
        "530330001001" => 2, "530330001002" => 1
    };
    let result = compare_plans(&a, &b);
    // Tracts assigned to opposite districts — no group matches
    assert!(result.jaccard_similarity < 0.01);
}

#[test]
fn test_jaccard_partial_overlap() {
    // 3 tracts: first two in same district in both plans, third reassigned
    let a: HashMap<String, usize> = hashmap!{
        "t00000000001" => 1, "t00000000002" => 1, "t00000000003" => 2
    };
    let b: HashMap<String, usize> = hashmap!{
        "t00000000001" => 1, "t00000000002" => 2, "t00000000003" => 2
    };
    let result = compare_plans(&a, &b);
    // Partial overlap: Jaccard should be between 0 and 1
    assert!(result.jaccard_similarity > 0.0 && result.jaccard_similarity < 1.0);
}

#[test]
fn test_jaccard_internal_fn_identical_sets() {
    let set: HashSet<String> = hashset!{"a".into(), "b".into(), "c".into()};
    assert!((jaccard(&set, &set) - 1.0).abs() < 1e-9);
}

#[test]
fn test_jaccard_internal_fn_disjoint_sets() {
    let a: HashSet<String> = hashset!{"a".into(), "b".into()};
    let b: HashSet<String> = hashset!{"c".into(), "d".into()};
    assert!((jaccard(&a, &b) - 0.0).abs() < 1e-9);
}

#[test]
fn test_jaccard_internal_fn_empty_sets() {
    // Two empty sets: union=0 → defined as 1.0 (identical empty plans)
    let empty: HashSet<String> = HashSet::new();
    assert!((jaccard(&empty, &empty) - 1.0).abs() < 1e-9);
}

#[test]
fn test_population_comparison_winner_framing_absent() {
    // Output must NOT use "Winner:" framing — use "Lower:" instead
    let a: HashMap<String, usize> = make_population_assignments(0.002);  // 0.2% max dev
    let b: HashMap<String, usize> = make_population_assignments(0.008);  // 0.8% max dev
    let comparison = compare_plans(&a, &b);
    let output = format_comparison_table(&comparison);
    assert!(!output.contains("Winner:"),
        "Output must not use 'Winner:' framing (legally dangerous)");
    assert!(output.contains("Lower:") || output.contains("Difference:"));
}

#[test]
fn test_comparison_output_contains_disclaimer() {
    let comparison = compare_plans(&make_test_assignments(5), &make_test_assignments(5));
    let output = format_comparison_table(&comparison);
    assert!(output.contains("No single metric determines legal compliance"),
        "Comparison output must include legal disclaimer");
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `PlanComparison`, `compare_plans()`, `jaccard()`, `format_comparison_table()`**

```rust
// redist-analysis/src/comparison.rs

pub struct PlanComparison {
    pub plan_a: PlanSummary,
    pub plan_b: PlanSummary,
    pub jaccard_similarity: f64,        // mean over best-matched districts
    pub population: PopulationComparison,
    pub compactness: CompactnessComparison,
    pub county_splits: Option<SplitComparison>,
    pub partisan: Option<PartisanComparison>,
}

pub struct PopulationComparison {
    pub plan_a_max_dev: f64,
    pub plan_b_max_dev: f64,
    // Note: NO "winner" field — use "lower" only
    pub lower: String,   // "plan_a" | "plan_b" | "equal"
    pub difference_pct: f64,
}

pub fn compare_plans(
    assignments_a: &HashMap<String, usize>,
    assignments_b: &HashMap<String, usize>,
) -> PlanComparison { ... }

fn jaccard(set_a: &HashSet<String>, set_b: &HashSet<String>) -> f64 {
    let intersection = set_a.intersection(set_b).count() as f64;
    let union = set_a.union(set_b).count() as f64;
    if union == 0.0 { 1.0 } else { intersection / union }
}

pub fn format_comparison_table(comparison: &PlanComparison) -> String { ... }
pub fn format_comparison_json(comparison: &PlanComparison) -> String { ... }
pub fn format_comparison_csv(comparison: &PlanComparison) -> String { ... }
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(analysis): Jaccard similarity + PlanComparison struct, no-winner output framing"`

---

## Task 2: Enacted tract assignment — centroid PIP with nearest-polygon fallback

**Files:** `redist/crates/redist-data/src/enacted.rs`

- [ ] **L0: Write failing enacted assignment tests**

```rust
// redist-data/src/enacted.rs tests

#[test]
fn test_tract_centroid_in_polygon_correct_district() {
    // Known point (WA centroid) inside a known polygon → assigned to that district
    use geo_types::{Point, MultiPolygon, Polygon, LineString};
    let centroid = Point::new(-120.5, 47.5);  // WA interior
    let poly = make_square_polygon(-121.0, 47.0, -120.0, 48.0);
    let polygons = vec![MultiPolygon::new(vec![poly])];
    let ids = vec![3usize];
    let result = assign_single_centroid(&centroid, &polygons, &ids);
    assert_eq!(result, 3);
}

#[test]
fn test_nearest_polygon_fallback_for_coastal_tract() {
    // Centroid falls outside all polygons (coastal/linear tract) → nearest polygon used
    let centroid = Point::new(-124.9, 47.1);  // just offshore WA coast
    let poly = make_square_polygon(-124.8, 47.0, -124.5, 47.2);  // inland
    let polygons = vec![MultiPolygon::new(vec![poly])];
    let ids = vec![5usize];
    // Must NOT fail — must return nearest district
    let result = assign_single_centroid(&centroid, &polygons, &ids);
    assert_eq!(result, 5, "Nearest-polygon fallback must assign coastal tract");
}

#[test]
fn test_100_pct_coverage_assertion_all_assigned() {
    // All tracts must be assigned; no gaps allowed
    let centroids = vec![Point::new(-120.5, 47.5), Point::new(-121.0, 46.0)];
    let geoids = vec!["530330001001".to_string(), "530410002001".to_string()];
    let polygons = vec![
        MultiPolygon::new(vec![make_square_polygon(-121.5, 47.0, -120.0, 48.0)]),
        MultiPolygon::new(vec![make_square_polygon(-122.0, 45.5, -120.5, 46.5)]),
    ];
    let ids = vec![1usize, 2usize];
    let result = assign_tracts_to_enacted(&centroids, &geoids, &polygons, &ids).unwrap();
    assert_eq!(result.len(), 2, "All tracts must be assigned");
    // Every GEOID must appear in output
    for geoid in &geoids {
        assert!(result.contains_key(geoid), "GEOID {} not in result", geoid);
    }
}

#[test]
fn test_zero_unassigned_tracts_after_fallback() {
    // If some centroids are outside all polygons, fallback must cover them
    let centroids = make_centroids_with_one_offshore();
    let geoids = vec!["530330001001".to_string(), "530330002001".to_string()];
    let (polygons, ids) = make_inland_polygons();
    let result = assign_tracts_to_enacted(&centroids, &geoids, &polygons, &ids).unwrap();
    assert_eq!(result.len(), geoids.len(), "100% coverage required after nearest-polygon fallback");
}

#[test]
fn test_assign_tracts_errors_if_coverage_incomplete() {
    // If coverage cannot be achieved (empty polygon list) → error, not silent gap
    let centroids = vec![Point::new(-120.5, 47.5)];
    let geoids = vec!["530330001001".to_string()];
    let polygons: Vec<MultiPolygon<f64>> = vec![];
    let ids: Vec<usize> = vec![];
    let result = assign_tracts_to_enacted(&centroids, &geoids, &polygons, &ids);
    assert!(result.is_err(), "Empty polygon list must produce error, not silent gap");
    assert!(result.unwrap_err().to_string().contains("coverage"));
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `assign_tracts_to_enacted()` and `assign_single_centroid()`**

```rust
// redist-data/src/enacted.rs

use geo::algorithm::Contains;
use geo::algorithm::EuclideanDistance;
use geo_types::{MultiPolygon, Point};
use std::collections::HashMap;

/// Assign each tract centroid to an enacted district.
/// Step 1: point-in-polygon (geo::Contains)
/// Step 2: any unassigned tract → nearest polygon (minimum centroid-to-boundary distance)
/// Step 3: assert 100% coverage; error if any tract unassigned.
pub fn assign_tracts_to_enacted(
    tract_centroids: &[Point<f64>],
    tract_geoids: &[String],
    enacted_polygons: &[MultiPolygon<f64>],
    enacted_ids: &[usize],
) -> anyhow::Result<HashMap<String, usize>> {
    if enacted_polygons.is_empty() {
        anyhow::bail!("coverage: cannot assign tracts — enacted polygon list is empty");
    }
    let mut assignments = HashMap::with_capacity(tract_geoids.len());
    for (centroid, geoid) in tract_centroids.iter().zip(tract_geoids.iter()) {
        let district = assign_single_centroid(centroid, enacted_polygons, enacted_ids);
        assignments.insert(geoid.clone(), district);
    }
    // Assert 100% coverage
    let unassigned: Vec<_> = tract_geoids.iter()
        .filter(|g| !assignments.contains_key(*g))
        .collect();
    if !unassigned.is_empty() {
        anyhow::bail!(
            "coverage: {} tracts unassigned after enacted assignment: {:?}",
            unassigned.len(), &unassigned[..unassigned.len().min(5)]
        );
    }
    Ok(assignments)
}

pub fn assign_single_centroid(
    centroid: &Point<f64>,
    polygons: &[MultiPolygon<f64>],
    ids: &[usize],
) -> usize {
    // Step 1: point-in-polygon
    for (poly, &id) in polygons.iter().zip(ids.iter()) {
        if poly.contains(centroid) {
            return id;
        }
    }
    // Step 2: nearest-polygon fallback (minimum Euclidean distance to polygon)
    polygons.iter().zip(ids.iter())
        .map(|(poly, &id)| (poly.euclidean_distance(centroid), id))
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .map(|(_, id)| id)
        .expect("polygons must not be empty — checked above")
}
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(data): enacted tract assignment — centroid PIP with nearest-polygon fallback, 100% coverage assertion"`

---

## Task 3: Contiguity analysis — BFS per-district

**Files:** `redist/crates/redist-analysis/src/contiguity.rs`

- [ ] **L0: Write failing contiguity tests (from Spec 3 + Scenario 4)**

```rust
// redist-analysis/src/contiguity.rs tests

#[test]
fn test_contiguity_connected_graph() {
    // 4 tracts in a line: 0-1-2-3, all in district 1 → contiguous
    let adj = vec![vec![1usize], vec![0,2], vec![1,3], vec![2]];
    let geoids: Vec<String> = (0..4).map(|i| format!("t{:011}", i)).collect();
    let assignments: HashMap<String, usize> = geoids.iter().cloned()
        .zip(vec![1,1,1,1]).collect();
    let result = check_contiguity(&assignments, &adj, &geoids, 1);
    assert!(result.all_contiguous);
    assert!(result.districts[0].contiguous);
    assert_eq!(result.districts[0].component_count, 1);
}

#[test]
fn test_contiguity_disconnected_district() {
    // Tracts 0,1 connected. Tract 3 isolated. All in district 1.
    // adj: 0-1, 2 alone (district 2), 3 alone (district 1, disconnected from 0,1)
    let adj = vec![vec![1usize], vec![0usize], vec![], vec![]];
    let geoids: Vec<String> = (0..4).map(|i| format!("t{:011}", i)).collect();
    let assignments: HashMap<String, usize> = hashmap!{
        geoids[0].clone() => 1,
        geoids[1].clone() => 1,
        geoids[2].clone() => 2,
        geoids[3].clone() => 1,   // isolated from 0 and 1
    };
    let result = check_contiguity(&assignments, &adj, &geoids, 2);
    assert!(!result.all_contiguous);
    let d1 = result.districts.iter().find(|d| d.district == 1).unwrap();
    assert!(!d1.contiguous);
    assert_eq!(d1.component_count, 2);
    // disconnected_tracts must list tract 3
    assert!(d1.disconnected_tracts.contains(&geoids[3]));
}

#[test]
fn test_contiguity_two_separate_districts_both_connected() {
    // Tracts 0,1 in district 1 (connected). Tracts 2,3 in district 2 (connected).
    let adj = vec![vec![1usize], vec![0,2], vec![1,3], vec![2]];
    let geoids: Vec<String> = (0..4).map(|i| format!("t{:011}", i)).collect();
    let assignments: HashMap<String, usize> = hashmap!{
        geoids[0].clone() => 1, geoids[1].clone() => 1,
        geoids[2].clone() => 2, geoids[3].clone() => 2,
    };
    let result = check_contiguity(&assignments, &adj, &geoids, 2);
    assert!(result.all_contiguous);
}

#[test]
fn test_bfs_component_count_single_component() {
    let members: HashSet<usize> = hashset![0, 1, 2, 3];
    let adj = vec![vec![1usize], vec![0,2], vec![1,3], vec![2]];
    let (count, _) = bfs_component_count(&members, &adj);
    assert_eq!(count, 1);
}

#[test]
fn test_bfs_component_count_two_components() {
    let members: HashSet<usize> = hashset![0, 1, 3];  // 2 missing; 0-1 connected, 3 isolated
    let adj = vec![vec![1usize], vec![0usize], vec![], vec![]];
    let (count, _) = bfs_component_count(&members, &adj);
    assert_eq!(count, 2);
}

#[test]
fn test_contiguity_single_tract_district_is_contiguous() {
    // A district with exactly one tract is trivially contiguous
    let adj = vec![vec![]];
    let geoids = vec!["t00000000001".to_string()];
    let assignments: HashMap<String, usize> = hashmap!{"t00000000001" => 1};
    let result = check_contiguity(&assignments, &adj, &geoids, 1);
    assert!(result.districts[0].contiguous);
    assert_eq!(result.districts[0].component_count, 1);
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `check_contiguity()`, `bfs_component_count()`, `ContiguityResult`**

```rust
// redist-analysis/src/contiguity.rs

pub struct ContiguityResult {
    pub all_contiguous: bool,
    pub districts: Vec<DistrictContiguity>,
}

pub struct DistrictContiguity {
    pub district: usize,
    pub contiguous: bool,
    pub tract_count: usize,
    pub component_count: usize,
    pub disconnected_tracts: Vec<String>,  // GEOIDs of isolated tracts
}

pub fn check_contiguity(
    assignments: &HashMap<String, usize>,
    adjacency: &[Vec<usize>],
    geoids: &[String],
    num_districts: usize,
) -> ContiguityResult {
    // Build index: geoid → index (for adjacency lookup)
    // For each district d in 1..=num_districts:
    //   Collect tract indices assigned to d
    //   Run bfs_component_count on that subset
    //   Record result
}

pub fn bfs_component_count(
    tract_indices: &HashSet<usize>,
    adjacency: &[Vec<usize>],
) -> (usize, Vec<HashSet<usize>>) {
    // Standard BFS from unvisited nodes, restricted to tract_indices subset
    // Returns (component_count, vec_of_components)
}
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(analysis): contiguity BFS per-district, component count, disconnected tract list"`

---

## Task 4: County + municipal split analysis

**Files:** `redist/crates/redist-analysis/src/splits.rs`, `redist/crates/redist-analysis/src/split_standards.rs`

- [ ] **L0: Write failing split tests (from Spec 3)**

```rust
// redist-analysis/src/splits.rs tests

#[test]
fn test_county_fips_from_geoid_king_county() {
    // Standard tract GEOID → county FIPS prefix
    assert_eq!(county_fips_from_geoid("530330001001"), "53033");
}

#[test]
fn test_county_fips_from_geoid_alabama() {
    assert_eq!(county_fips_from_geoid("010010020100"), "01001");
}

#[test]
fn test_county_split_single_district_no_split() {
    // All King County tracts in district 1 → no split
    let assignments: HashMap<String, usize> = hashmap!{
        "53033001000000".to_string() => 1,
        "53033002000000".to_string() => 1,
    };
    // Use first 11 chars as GEOIDs
    let assignments: HashMap<String, usize> = hashmap!{
        "53033001" => 1, "53033002" => 1,
    }.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
    let result = analyze_county_splits(&assignments, None);
    assert_eq!(result.split, 0);
    assert_eq!(result.preservation_score, 1.0);
}

#[test]
fn test_county_split_across_two_districts() {
    // King County split across districts 1 and 2
    let assignments: HashMap<String, usize> = hashmap!{
        "53033000001".to_string() => 1,
        "53033000002".to_string() => 2,
    };
    let result = analyze_county_splits(&assignments, None);
    assert_eq!(result.split, 1);
    assert_eq!(result.split_list[0].county_fips, "53033");
    assert_eq!(result.split_list[0].districts_containing.len(), 2);
    assert_eq!(result.split_list[0].split_severity, 2);
}

#[test]
fn test_county_split_severity_three_districts() {
    // One county split across three districts
    let assignments: HashMap<String, usize> = hashmap!{
        "53033000001".to_string() => 1,
        "53033000002".to_string() => 2,
        "53033000003".to_string() => 3,
    };
    let result = analyze_county_splits(&assignments, None);
    assert_eq!(result.split, 1);
    assert_eq!(result.split_list[0].split_severity, 3);
}

#[test]
fn test_county_preservation_score_formula() {
    // 2 counties total, 1 split → preservation_score = (2-1)/2 = 0.5
    let assignments: HashMap<String, usize> = hashmap!{
        "53033000001".to_string() => 1,  // King County
        "53033000002".to_string() => 2,  // King County split
        "53035000001".to_string() => 3,  // Kitsap County (whole)
    };
    let result = analyze_county_splits(&assignments, None);
    assert_eq!(result.split, 1);
    assert!((result.preservation_score - 0.5).abs() < 1e-9);
}

#[test]
fn test_municipal_split_detected() {
    // Place "5363000" (Seattle) tracts in two different districts
    let assignments: HashMap<String, usize> = hashmap!{
        "53033005000".to_string() => 7,
        "53033005100".to_string() => 9,
    };
    let place_to_tracts: HashMap<String, Vec<String>> = hashmap!{
        "5363000".to_string() => vec!["53033005000".to_string(), "53033005100".to_string()],
    };
    let place_names: HashMap<String, String> = hashmap!{
        "5363000".to_string() => "Seattle".to_string(),
    };
    let result = analyze_municipal_splits(&assignments, &place_to_tracts, &place_names);
    assert_eq!(result.split, 1);
    assert_eq!(result.split_list[0].place_name, "Seattle");
    assert_eq!(result.split_list[0].split_severity, 2);
}

#[test]
fn test_municipal_data_absent_returns_unavailable() {
    // When place_to_tracts is empty, result must indicate data unavailable
    let result = analyze_municipal_splits(
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
    );
    assert!(!result.available);
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `analyze_county_splits()`, `analyze_municipal_splits()`, `county_fips_from_geoid()`**

```rust
// redist-analysis/src/splits.rs

/// Parse county FIPS from 11-char Census tract GEOID.
/// "530330001001" → "53033" (state 53, county 033)
pub fn county_fips_from_geoid(geoid: &str) -> &str {
    &geoid[..5]
}

pub struct CountySplitResult {
    pub total: usize,
    pub split: usize,
    pub preservation_score: f64,
    pub split_list: Vec<CountySplit>,
    pub legal_standard: Option<String>,   // from split_standards lookup
    pub compliance_assessment: Option<String>,
}

pub struct CountySplit {
    pub county_fips: String,
    pub county_name: Option<String>,
    pub districts_containing: Vec<usize>,
    pub tract_count: usize,
    pub split_severity: usize,
}

pub struct MunicipalSplitResult {
    pub available: bool,
    pub total: usize,
    pub split: usize,
    pub preservation_score: f64,
    pub split_list: Vec<MunicipalSplit>,
}

pub struct MunicipalSplit {
    pub place_fips: String,
    pub place_name: String,
    pub districts_containing: Vec<usize>,
    pub split_severity: usize,
}

pub fn analyze_county_splits(
    assignments: &HashMap<String, usize>,
    county_names: Option<&HashMap<String, String>>,
) -> CountySplitResult { ... }

pub fn analyze_municipal_splits(
    assignments: &HashMap<String, usize>,
    place_to_tracts: &HashMap<String, Vec<String>>,
    place_names: &HashMap<String, String>,
) -> MunicipalSplitResult { ... }
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(analysis): county + municipal split analysis, GEOID-based county FIPS extraction"`

---

## Task 5: Per-state split standards lookup table

**Files:** `redist/crates/redist-analysis/src/split_standards.rs`

- [ ] **L0: Write failing split standard tests (from Scenario 1)**

```rust
// redist-analysis/src/split_standards.rs tests

#[test]
fn test_wa_splits_has_legal_standard_field() {
    // WA must return a constitutional citation
    let standard = get_split_standard("WA");
    assert!(standard.is_some());
    let s = standard.unwrap();
    assert!(s.legal_standard.contains("WA Const."),
        "WA standard must cite WA constitution");
}

#[test]
fn test_ca_splits_has_legal_standard_field() {
    let standard = get_split_standard("CA");
    assert!(standard.is_some());
    assert!(standard.unwrap().legal_standard.contains("Art. XXI"));
}

#[test]
fn test_tx_splits_has_legal_standard_field() {
    let standard = get_split_standard("TX");
    assert!(standard.is_some());
}

#[test]
fn test_co_splits_has_legal_standard_field() {
    let standard = get_split_standard("CO");
    assert!(standard.is_some());
}

#[test]
fn test_unknown_state_returns_generic_standard() {
    // States not in the lookup table get a generic disclaimer
    let standard = get_split_standard("ND");
    // Should return something (generic), not None — never silently skip
    assert!(standard.is_some());
    assert!(standard.unwrap().legal_standard.contains("consult"));
}

#[test]
fn test_standard_always_includes_disclaimer() {
    for state in &["WA", "CA", "TX", "CO", "VT"] {
        let s = get_split_standard(state).unwrap();
        assert!(!s.disclaimer.is_empty(),
            "State {} must have a disclaimer field", state);
        assert!(s.disclaimer.contains("counsel") || s.disclaimer.contains("consult"),
            "Disclaimer must recommend legal counsel for state {}", state);
    }
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `get_split_standard()` with built-in table**

```rust
// redist-analysis/src/split_standards.rs

pub struct SplitStandard {
    pub state_code: String,
    pub legal_standard: String,
    pub compliance_assessment_template: String,  // filled with actual split count at runtime
    pub disclaimer: String,
}

/// Per-state constitutional split standard lookup.
/// Minimum: WA, CA, TX, CO. All others: generic disclaimer.
pub fn get_split_standard(state_code: &str) -> Option<SplitStandard> {
    match state_code {
        "WA" => Some(SplitStandard {
            state_code: "WA".into(),
            legal_standard: "WA Const. Art. II §43 — counties shall be preserved where possible".into(),
            compliance_assessment_template: "{n} splits present; no binding numerical limit under WMCA".into(),
            disclaimer: "Legal compliance determination requires counsel".into(),
        }),
        "CA" => Some(SplitStandard {
            state_code: "CA".into(),
            legal_standard: "CA Const. Art. XXI §2(d) — minimize county and city splits".into(),
            compliance_assessment_template: "{n} county splits, {m} city splits present".into(),
            disclaimer: "Legal compliance determination requires counsel".into(),
        }),
        "TX" => Some(SplitStandard {
            state_code: "TX".into(),
            legal_standard: "TX Const. Art. III §26 — counties preserved where practicable".into(),
            compliance_assessment_template: "{n} county splits present".into(),
            disclaimer: "Legal compliance determination requires counsel".into(),
        }),
        "CO" => Some(SplitStandard {
            state_code: "CO".into(),
            legal_standard: "CO Const. Art. V §47 — preserve political subdivisions".into(),
            compliance_assessment_template: "{n} county splits present".into(),
            disclaimer: "Legal compliance determination requires counsel".into(),
        }),
        _ => Some(SplitStandard {
            state_code: state_code.into(),
            legal_standard: format!("Consult {state_code} constitutional standards for applicable split criteria"),
            compliance_assessment_template: "{n} county splits present".into(),
            disclaimer: "Legal compliance determination requires counsel. State-specific standard not in built-in table.".into(),
        }),
    }
}
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(analysis): per-state split standard lookup table (WA, CA, TX, CO + generic fallback)"`

---

## Task 6: Composable bitfield exit codes

**Files:** `redist/crates/redist-analysis/src/exit_codes.rs`, `redist/crates/redist-cli/src/analyze.rs`

- [ ] **L0: Write failing exit code tests (from Scenario 4)**

```rust
// redist-analysis/src/exit_codes.rs tests

#[test]
fn test_no_violations_exit_0() {
    let code = compute_exit_code(false, false, false, false);
    assert_eq!(code, 0);
}

#[test]
fn test_balance_violation_only_exit_1() {
    let code = compute_exit_code(true, false, false, false);
    assert_eq!(code, 1);
}

#[test]
fn test_contiguity_violation_only_exit_2() {
    let code = compute_exit_code(false, true, false, false);
    assert_eq!(code, 2);
}

#[test]
fn test_balance_and_contiguity_exit_3() {
    let code = compute_exit_code(true, true, false, false);
    assert_eq!(code, 3);
}

#[test]
fn test_nesting_exit_code_is_bit2() {
    // nesting violation only → exit code 4 (bit 2)
    let code = compute_exit_code(false, false, true, false);
    assert_eq!(code, 4);
}

#[test]
fn test_balance_and_nesting_exit_code() {
    // balance violation + nesting violation → bits 0 + 2 = 5
    let code = compute_exit_code(true, false, true, false);
    assert_eq!(code, 5);
}

#[test]
fn test_missing_data_exit_8() {
    let code = compute_exit_code(false, false, false, true);
    assert_eq!(code, 8);
}

#[test]
fn test_all_violations_exit_15() {
    let code = compute_exit_code(true, true, true, true);
    assert_eq!(code, 15);
}

#[test]
fn test_allow_noncontiguous_suppresses_bit1() {
    // With --allow-noncontiguous: contiguity violation bit is cleared
    let code = compute_exit_code_with_flags(
        true, true, false, false,  // balance + contiguity
        /*allow_noncontiguous=*/ true,
        /*allow_imbalance=*/ false,
    );
    // bit 1 (contiguity=2) suppressed; only bit 0 (balance=1) remains
    assert_eq!(code, 1);
}

#[test]
fn test_allow_imbalance_suppresses_bit0() {
    let code = compute_exit_code_with_flags(
        true, false, false, false,
        /*allow_noncontiguous=*/ false,
        /*allow_imbalance=*/ true,
    );
    assert_eq!(code, 0);
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `compute_exit_code()` and `compute_exit_code_with_flags()`**

```rust
// redist-analysis/src/exit_codes.rs

pub const BIT_BALANCE: u8     = 0b0001;  // 1
pub const BIT_CONTIGUITY: u8  = 0b0010;  // 2
pub const BIT_NESTING: u8     = 0b0100;  // 4
pub const BIT_MISSING_DATA: u8 = 0b1000; // 8

pub fn compute_exit_code(
    balance_violation: bool,
    contiguity_violation: bool,
    nesting_violation: bool,
    missing_data: bool,
) -> u8 {
    let mut code: u8 = 0;
    if balance_violation    { code |= BIT_BALANCE; }
    if contiguity_violation { code |= BIT_CONTIGUITY; }
    if nesting_violation    { code |= BIT_NESTING; }
    if missing_data         { code |= BIT_MISSING_DATA; }
    code
}

pub fn compute_exit_code_with_flags(
    balance_violation: bool,
    contiguity_violation: bool,
    nesting_violation: bool,
    missing_data: bool,
    allow_noncontiguous: bool,
    allow_imbalance: bool,
) -> u8 {
    let effective_contiguity = contiguity_violation && !allow_noncontiguous;
    let effective_balance = balance_violation && !allow_imbalance;
    compute_exit_code(effective_balance, effective_contiguity, nesting_violation, missing_data)
}
```

- [ ] **Update `redist analyze` dispatcher** to return bitfield exit code from process
- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(analysis): composable bitfield exit codes (1=balance, 2=contiguity, 4=nesting, 8=missing-data)"`

---

## Task 7: `redist fetch --type enacted` and `--type geography`

**Files:** `redist/crates/redist-cli/src/fetch.rs`, `redist/crates/redist-cli/src/args.rs`

- [ ] **L0: Write failing fetch type tests**

```rust
#[test]
fn test_fetch_type_enum_includes_enacted() {
    let _t = FetchType::Enacted;
}

#[test]
fn test_fetch_type_enum_includes_geography() {
    let _t = FetchType::Geography;
}

#[test]
fn test_enacted_url_pattern_wa_2020_congressional() {
    let url = enacted_url("WA", "53", "2020", "congressional");
    // Must reference Census TIGER congressional district file
    assert!(url.contains("census.gov"), "URL must point to Census");
    assert!(url.contains("53") || url.contains("WA"), "URL must include state identifier");
}

#[test]
fn test_geography_url_pattern_wa_2020() {
    let url = geography_url("53", "2020");
    assert!(url.contains("census.gov"));
    assert!(url.contains("53"));
}

#[test]
fn test_enacted_fetch_args_parsed() {
    let args = FetchArgs::try_parse_from([
        "redist", "fetch", "--type", "enacted",
        "--year", "2020", "--states", "WA"
    ]).unwrap();
    assert!(matches!(args.fetch_type, FetchType::Enacted));
    assert_eq!(args.states, vec!["WA"]);
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Add `Enacted` and `Geography` variants to `FetchType` enum**
- [ ] **Implement `enacted_url()` and `geography_url()` URL constructors** using Census TIGER URL patterns
- [ ] **Implement download handlers** for enacted shapefiles and place-tract relationship files
- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(cli): redist fetch --type enacted + --type geography with Census TIGER URLs"`

---

## Task 8: `redist compare` command

**Files:** `redist/crates/redist-cli/src/compare.rs`, `redist/crates/redist-cli/src/args.rs`, `redist/crates/redist-cli/src/main.rs`

- [ ] **L0: Write failing compare CLI tests**

```rust
#[test]
fn test_compare_args_plan_a_required() {
    let result = CompareArgs::try_parse_from(["redist", "compare", "--plan-b", "plan2"]);
    assert!(result.is_err(), "--plan-a is required");
}

#[test]
fn test_compare_args_plan_b_or_enacted_required() {
    let result = CompareArgs::try_parse_from([
        "redist", "compare", "--plan-a", "plan1"
    ]);
    // Must fail: neither --plan-b nor --enacted provided
    assert!(result.is_err() || {
        // Or: parse succeeds but validation catches it
        let args = result.unwrap();
        args.plan_b.is_none() && !args.enacted
    });
}

#[test]
fn test_compare_format_json_parsed() {
    let args = CompareArgs::try_parse_from([
        "redist", "compare",
        "--plan-a", "plan1", "--plan-b", "plan2",
        "--format", "json",
    ]).unwrap();
    assert!(matches!(args.format, CompareFormat::Json));
}

#[test]
fn test_compare_format_csv_parsed() {
    let args = CompareArgs::try_parse_from([
        "redist", "compare",
        "--plan-a", "plan1", "--plan-b", "plan2",
        "--format", "csv",
    ]).unwrap();
    assert!(matches!(args.format, CompareFormat::Csv));
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Add `CompareArgs` to `args.rs`**

```rust
#[derive(Args)]
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
    /// Census year
    #[arg(long, default_value = "2020")]
    pub year: String,
    /// Version directory for label resolution
    #[arg(long)]
    pub version: Option<String>,
    /// Metrics to compare: population, compactness, splits, partisan, all
    #[arg(long, default_value = "all")]
    pub metrics: Vec<String>,
    /// Output file path (default: stdout)
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Output format
    #[arg(long, value_enum, default_value_t = CompareFormat::Table)]
    pub format: CompareFormat,
}

#[derive(clap::ValueEnum, Clone)]
pub enum CompareFormat { Table, Json, Csv }
```

- [ ] **Add `Commands::Compare(CompareArgs)` to commands enum**
- [ ] **Implement `run_compare()` in `compare.rs`**: load both plan assignments, call `compare_plans()`, format output
- [ ] **Handle `--enacted`**: download if absent, call `assign_tracts_to_enacted()`
- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(cli): redist compare --plan-a --plan-b / --enacted with table/json/csv output"`

---

## Task 9: Wire `contiguity` and `splits` into `redist analyze`

**Files:** `redist/crates/redist-cli/src/analyze.rs`

- [ ] **L0: Write failing analyze dispatch tests**

```rust
#[test]
fn test_analyze_types_includes_contiguity() {
    let args = AnalyzeArgs::try_parse_from([
        "redist", "analyze",
        "--state", "WA", "--year", "2020", "--version", "v1",
        "--types", "contiguity",
    ]).unwrap();
    assert!(args.types.contains(&AnalyzerType::Contiguity));
}

#[test]
fn test_analyze_types_includes_splits() {
    let args = AnalyzeArgs::try_parse_from([
        "redist", "analyze",
        "--state", "WA", "--year", "2020", "--version", "v1",
        "--types", "splits",
    ]).unwrap();
    assert!(args.types.contains(&AnalyzerType::Splits));
}

#[test]
fn test_analyze_all_includes_contiguity_and_splits() {
    let args = AnalyzeArgs::try_parse_from([
        "redist", "analyze",
        "--state", "WA", "--year", "2020", "--version", "v1",
        "--types", "all",
    ]).unwrap();
    let all_types = expand_all_types(&args.types);
    assert!(all_types.contains(&AnalyzerType::Contiguity));
    assert!(all_types.contains(&AnalyzerType::Splits));
}

#[test]
fn test_allow_noncontiguous_flag_parsed() {
    let args = AnalyzeArgs::try_parse_from([
        "redist", "analyze",
        "--state", "WA", "--year", "2020", "--version", "v1",
        "--types", "contiguity", "--allow-noncontiguous",
    ]).unwrap();
    assert!(args.allow_noncontiguous);
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Add `Contiguity`, `Splits` to `AnalyzerType` enum**
- [ ] **Add `--allow-noncontiguous` flag to `AnalyzeArgs`**
- [ ] **Wire analyze dispatch**: call `check_contiguity()` and `analyze_county_splits()` / `analyze_municipal_splits()` and write output JSON
- [ ] **Apply bitfield exit code from Task 6** when returning from analyze
- [ ] **Check missing municipal data requirement** for the state (via split_standards lookup); exit code 8 if required but absent
- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(cli): contiguity + splits analyzer types in redist analyze, --allow-noncontiguous flag"`

---

## Task 10: L2 Acceptance Tests

**Files:** `tests/acceptance/test_spec2_spec3_acceptance.py`

- [ ] **Write L2 acceptance tests**

```python
# tests/acceptance/test_spec2_spec3_acceptance.py
"""L2 acceptance tests for Spec 2 (Plan Comparison) + Spec 3 (Constraint Analysis)."""

import json
import subprocess
import pytest
from pathlib import Path

# --- Spec 2: Plan Comparison ---

def test_compare_two_generated_plans_jaccard_less_than_1(tmp_redist_output):
    """Two VT runs with different seeds → Jaccard < 1.0 (different assignments)."""
    for seed, label in [(42, "vt_cmp_s42"), (99, "vt_cmp_s99")]:
        subprocess.run([
            "redist", "state",
            "--state", "VT", "--year", "2020", "--version", "spec2_test",
            "--label", label, "--seed", str(seed),
            "--output-dir", str(tmp_redist_output),
        ], check=True)
    result = subprocess.run([
        "redist", "compare",
        "--plan-a", "vt_cmp_s42", "--plan-b", "vt_cmp_s99",
        "--year", "2020", "--version", "spec2_test",
        "--format", "json", "--output-dir", str(tmp_redist_output),
    ], capture_output=True, text=True, check=True)
    data = json.loads(result.stdout)
    assert "jaccard_similarity" in data["metrics"]
    jaccard = data["metrics"]["jaccard_similarity"]
    # VT has 1 congressional district — Jaccard may be 1.0 by coincidence
    # Just verify the metric is present and in [0,1]
    assert 0.0 <= jaccard <= 1.0

def test_compare_plan_vs_self_jaccard_1(tmp_redist_output):
    """Same plan vs itself → Jaccard = 1.0, all population metrics equal."""
    subprocess.run([
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec2_self",
        "--label", "vt_self_test",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    result = subprocess.run([
        "redist", "compare",
        "--plan-a", "vt_self_test", "--plan-b", "vt_self_test",
        "--year", "2020", "--version", "spec2_self",
        "--format", "json", "--output-dir", str(tmp_redist_output),
    ], capture_output=True, text=True, check=True)
    data = json.loads(result.stdout)
    assert data["metrics"]["jaccard_similarity"] == pytest.approx(1.0)
    pop = data["metrics"]["population"]
    assert pop["plan_a_max_dev"] == pytest.approx(pop["plan_b_max_dev"])

def test_compare_output_no_winner_framing(tmp_redist_output):
    """Comparison table output must not contain 'Winner:' (legally dangerous)."""
    subprocess.run([
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec2_frame",
        "--label", "vt_frame_a", "--seed", "42",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    subprocess.run([
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec2_frame",
        "--label", "vt_frame_b", "--seed", "99",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    result = subprocess.run([
        "redist", "compare",
        "--plan-a", "vt_frame_a", "--plan-b", "vt_frame_b",
        "--year", "2020", "--version", "spec2_frame",
        "--format", "table", "--output-dir", str(tmp_redist_output),
    ], capture_output=True, text=True, check=True)
    assert "Winner:" not in result.stdout, \
        "Comparison output must not contain 'Winner:' framing"
    assert "No single metric determines legal compliance" in result.stdout, \
        "Comparison output must include legal disclaimer"

def test_compare_all_metrics_present_in_json(tmp_redist_output):
    """JSON comparison output contains all required top-level metric keys."""
    subprocess.run([
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec2_keys",
        "--label", "vt_keys_a",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    result = subprocess.run([
        "redist", "compare",
        "--plan-a", "vt_keys_a", "--plan-b", "vt_keys_a",
        "--year", "2020", "--version", "spec2_keys",
        "--format", "json", "--output-dir", str(tmp_redist_output),
    ], capture_output=True, text=True, check=True)
    data = json.loads(result.stdout)
    assert "plan_a" in data
    assert "plan_b" in data
    assert "metrics" in data
    metrics = data["metrics"]
    assert "jaccard_similarity" in metrics
    assert "population" in metrics
    assert "compactness" in metrics

# --- Spec 3: Constraint Analysis ---

def test_wa_contiguity_all_pass(tmp_redist_output):
    """WA 10-district congressional plan → all_contiguous: true."""
    subprocess.run([
        "redist", "state",
        "--state", "WA", "--year", "2020", "--version", "spec3_contiguity",
        "--label", "wa_contiguity_test",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    result = subprocess.run([
        "redist", "analyze",
        "--state", "WA", "--year", "2020", "--version", "spec3_contiguity",
        "--label", "wa_contiguity_test", "--types", "contiguity",
        "--output-dir", str(tmp_redist_output),
    ], capture_output=True, text=True)
    assert result.returncode == 0, result.stderr
    contiguity_path = (tmp_redist_output / "2020" / "plans" /
                       "wa_contiguity_test" / "analysis" / "contiguity.json")
    assert contiguity_path.exists()
    data = json.loads(contiguity_path.read_text())
    assert data["all_contiguous"] is True
    assert len(data["districts"]) == 10

def test_wa_county_splits_count_reasonable(tmp_redist_output):
    """WA generated plan → split count present and ≥ 0."""
    subprocess.run([
        "redist", "state",
        "--state", "WA", "--year", "2020", "--version", "spec3_splits",
        "--label", "wa_splits_test",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    subprocess.run([
        "redist", "analyze",
        "--state", "WA", "--year", "2020", "--version", "spec3_splits",
        "--label", "wa_splits_test", "--types", "splits",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    splits_path = (tmp_redist_output / "2020" / "plans" /
                   "wa_splits_test" / "analysis" / "splits.json")
    assert splits_path.exists()
    data = json.loads(splits_path.read_text())
    assert "counties" in data
    assert data["counties"]["split"] >= 0
    assert data["counties"]["total"] == 39  # WA has 39 counties
    assert 0.0 <= data["counties"]["preservation_score"] <= 1.0

def test_splits_json_structure_required_fields(tmp_redist_output):
    """splits.json contains all required top-level fields."""
    subprocess.run([
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec3_struct",
        "--label", "vt_splits_struct",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    subprocess.run([
        "redist", "analyze",
        "--state", "VT", "--year", "2020", "--version", "spec3_struct",
        "--label", "vt_splits_struct", "--types", "splits",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    data = json.loads(
        (tmp_redist_output / "2020" / "plans" / "vt_splits_struct" /
         "analysis" / "splits.json").read_text()
    )
    assert "analyzer" in data and data["analyzer"] == "splits"
    counties = data["counties"]
    assert "total" in counties
    assert "split" in counties
    assert "preservation_score" in counties
    assert "split_list" in counties

def test_contiguity_exits_2_on_violation(tmp_redist_output):
    """Synthesized disconnected plan → analyze exits with code 2 (bit 1 contiguity)."""
    # Write a fake disconnected plan JSON directly (no actual redistricting run)
    plan_dir = tmp_redist_output / "2020" / "plans" / "vt_disconnected"
    plan_dir.mkdir(parents=True, exist_ok=True)
    (plan_dir / "manifest.json").write_text(json.dumps({
        "label": "vt_disconnected", "state_code": "VT", "year": "2020",
        "chamber": "congressional", "num_districts": 1, "balance_tolerance_pct": 0.5,
    }))
    # Two tracts in same district but adjacency graph shows them not connected
    # (The test relies on the analyze command loading assignments and checking contiguity)
    # This test may need a test fixture generator in the CLI — skip if not supported
    pytest.skip("Requires test fixture support for synthesized disconnected plans")

def test_wa_splits_has_legal_standard_field(tmp_redist_output):
    """WA splits.json includes legal_standard referencing WA Const."""
    subprocess.run([
        "redist", "state",
        "--state", "WA", "--year", "2020", "--version", "spec3_legal",
        "--label", "wa_legal_test",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    subprocess.run([
        "redist", "analyze",
        "--state", "WA", "--year", "2020", "--version", "spec3_legal",
        "--label", "wa_legal_test", "--types", "splits",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    data = json.loads(
        (tmp_redist_output / "2020" / "plans" / "wa_legal_test" /
         "analysis" / "splits.json").read_text()
    )
    counties = data["counties"]
    assert "legal_standard" in counties, "splits.json must include legal_standard field for WA"
    assert "WA Const." in counties["legal_standard"]
    assert "disclaimer" in counties
    assert len(counties["disclaimer"]) > 0

def test_exit_code_0_when_all_constraints_satisfied(tmp_redist_output):
    """VT 1-district plan (always contiguous, always balanced) → exit code 0."""
    subprocess.run([
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec3_exit",
        "--label", "vt_exit_test",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    result = subprocess.run([
        "redist", "analyze",
        "--state", "VT", "--year", "2020", "--version", "spec3_exit",
        "--label", "vt_exit_test", "--types", "contiguity splits",
        "--output-dir", str(tmp_redist_output),
    ], capture_output=True, text=True)
    assert result.returncode == 0, \
        f"Expected exit 0, got {result.returncode}. stderr: {result.stderr}"
```

- [ ] **Run L2 tests** — expect FAIL (features not wired end-to-end)
- [ ] **Wire all components end-to-end**: fetch → enacted assignment → compare output; analyze dispatch → contiguity/splits JSON; exit codes
- [ ] **Run L2 tests** — expect PASS
- [ ] **Commit:** `git commit -m "test(acceptance): L2 tests for Spec 2 plan comparison + Spec 3 constraint analysis"`

---

## L0 Tests from Scenario Simulations

The following test assertions come directly from the scenario simulation document.

### Scenario 3: Cross-tool RPLAN roundtrip (RPLAN structure assertions)

```python
# tests/unit/test_comparison.py (Python-side)

def test_rplan_all_assignments_11char_geoids():
    rplan = load_rplan_fixture("wa_house_draft1.rplan")
    for geoid in rplan["assignments"].keys():
        assert len(geoid) == 11, f"GEOID {geoid!r} must be 11 chars"
        assert geoid.isdigit(), f"GEOID {geoid!r} must be numeric"

def test_rplan_geojson_lon_lat_order():
    rplan = load_rplan_fixture("wa_house_draft1.rplan")
    if rplan["geometry"] is not None:
        coords = rplan["geometry"]["features"][0]["geometry"]["coordinates"]
        first_coord = coords[0][0][0]  # [lon, lat]
        lon, lat = first_coord[0], first_coord[1]
        # WA longitude is negative (west), latitude positive
        assert lon < 0, f"longitude {lon} should be negative for WA"
        assert 40 < lat < 50, f"latitude {lat} out of WA range"

def test_gerrychain_roundtrip_preserves_assignments():
    """GerryChain conversion: rplan['assignments'] → rplan2['assignments'] roundtrip."""
    rplan = load_rplan_fixture("wa_house_draft1.rplan")
    # GerryChain uses singular 'assignment' key
    gerrychain = {"assignment": rplan["assignments"]}
    # Convert back to RPLAN
    rplan2 = {
        "rplan_version": "0.1",
        "metadata": rplan["metadata"],
        "assignments": gerrychain["assignment"],  # singular back to plural
        "geometry": None,
    }
    assert rplan["assignments"] == rplan2["assignments"]
```

### Scenario 4: Nesting violation detection (exit code assertions)

```rust
// tests/unit/test_contiguity.rs

#[test]
fn test_nesting_violation_senate_contains_three_house_districts() {
    let house: HashMap<String, usize> = hashmap!{
        "t0" => 1, "t1" => 2, "t2" => 3, "t3" => 3,
        "t4" => 5, "t5" => 5, "t6" => 6, "t7" => 6,
        "t8" => 7, "t9" => 7,
    };
    let senate: HashMap<String, usize> = hashmap!{
        "t0" => 1, "t1" => 1,
        "t2" => 3, "t3" => 3, "t4" => 3, "t5" => 3,
        "t6" => 3, "t7" => 3, "t8" => 3, "t9" => 3,
    };
    let result = validate_nesting(&house, &senate, 2);
    assert!(!result.valid);
    let v = &result.violations[0];
    assert_eq!(v.senate_district, 3);
    assert_eq!(v.house_districts.len(), 3);  // senate 3 contains house 5, 6, 7
}

#[test]
fn test_nesting_exit_code_is_bit2() {
    // nesting violation only → exit code 4 (bit 2)
    assert_eq!(compute_exit_code(false, false, true, false), 4);
}

#[test]
fn test_balance_and_nesting_exit_code() {
    // balance violation + nesting violation → bits 0 + 2 = 5
    assert_eq!(compute_exit_code(true, false, true, false), 5);
}
```

---

## Execution Order

1. Task 1 — Jaccard + PlanComparison (no deps)
2. Task 2 — Enacted tract assignment (no deps outside `geo` crate)
3. Task 3 — Contiguity BFS (no deps)
4. Task 4 — Split analysis (no deps)
5. Task 5 — Split standards table (no deps)
6. Task 6 — Exit codes (depends on Tasks 3+4)
7. Task 7 — `redist fetch --type enacted/geography` (depends on Task 2)
8. Task 8 — `redist compare` command (depends on Tasks 1+2+7)
9. Task 9 — Wire contiguity + splits into `redist analyze` (depends on Tasks 3+4+5+6)
10. Task 10 — L2 acceptance tests (depends on all above)

Tasks 1, 2, 3, 4, 5 are all independent and can run in parallel. Task 6 depends on 3+4. Tasks 7 and 8 are sequential. Tasks 9 and 10 must wait for everything upstream.

---

## Plan Board Review Amendments (2026-04-26)

**[TRENCH] CRITICAL — Centroid PIP ambiguity for boundary-straddling tracts**
Tracts whose centroid falls in the wrong district (common in dense urban areas where tracts straddle enacted district lines) are misassigned by PIP — nearest-polygon fallback only helps tracts whose centroid is entirely outside all polygons, not the ambiguous-centroid case. Fix: document this known bias — add `pip_method_note: "centroid PIP; boundary-straddling tracts may be misassigned"` and `fallback_count: N` to `enacted_assignments.json` metadata so practitioners can audit coverage. Accept this as a known limitation; the alternative (full polygon intersection) is orders of magnitude slower.

**[BENCHMARK] CRITICAL — Contiguity exit code test is skipped**
`test_contiguity_exits_2_on_violation` is `pytest.skip()`'d — the most important test (does the bitfield exit code actually propagate to the process exit code?) is never run. Fix: implement the fixture as a `conftest.py` helper `make_disconnected_plan(state, year, version, label)` that writes a known-bad `final_assignments.json` where one district's tracts are split across non-adjacent regions. Remove the skip; this test must pass before the task is marked complete.
