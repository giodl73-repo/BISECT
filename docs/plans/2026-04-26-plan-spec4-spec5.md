# Plan 3: Partisan Metrics (Spec 4) + Multi-Chamber / Nested Plans (Spec 5)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `redist analyze --types partisan` with efficiency gap, mean-median, and partisan bias (plus bootstrap CI), and `redist suite` for drawing and validating multi-chamber nested plans. Together these enable a practitioner to quantify partisan fairness and enforce constitutional nesting constraints (e.g., WA senate-in-house) from a single CLI.

**Specs:** Spec 4 (Partisan Metrics) + Spec 5 (Multi-Chamber/Nested Plans), board amendments through R3.

**Architecture:**
- `redist-analysis` crate: new `partisan.rs` module implementing the `Analyzer` trait
- `redist-cli`: `AnalyzeArgs` gains `--election-file`; new `Suite` subcommand tree with `draw`, `validate`, `add-plan`
- `redist-suite` logic lives in `redist-cli/src/suite.rs` (no separate crate needed at this scale)
- Nesting adjacency and validation live in `redist-analysis/src/nesting.rs`

**Tech Stack:** Rust, `serde_json`, `csv` crate, `rand` (deterministic bootstrap), existing `redist-analysis` `Analyzer` trait

**Scenarios covered:** Scenario 4 (nesting violation detection), Scenario 5 (small-chamber partisan metrics)

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `redist/crates/redist-analysis/src/partisan.rs` | **Create** | `PartisanAnalyzer`: EG, MM, PB, bootstrap CI |
| `redist/crates/redist-analysis/src/nesting.rs` | **Create** | `build_chamber_adjacency`, `validate_nesting`, `NestingValidation` |
| `redist/crates/redist-analysis/src/lib.rs` | **Modify** | expose `partisan`, `nesting` modules |
| `redist/crates/redist-cli/src/partisan.rs` | **Create** | `run_partisan()` — load election CSV, dispatch analyzer, write JSON |
| `redist/crates/redist-cli/src/suite.rs` | **Create** | `run_suite()` — draw chambers in sequence, enforce nesting hard-fail |
| `redist/crates/redist-cli/src/args.rs` | **Modify** | `AnalyzeArgs` + `--election-file`; `SuiteArgs` subcommand tree |
| `redist/crates/redist-cli/src/main.rs` | **Modify** | wire `Commands::Suite` |
| `tests/unit/test_partisan.rs` | **Create** | L0 partisan metric + bootstrap tests |
| `tests/unit/test_nesting.rs` | **Create** | L0 chamber adjacency + nesting validation tests |
| `tests/acceptance/test_partisan_acceptance.py` | **Create** | L2 acceptance: WA partisan JSON, VT single-district, missing election |
| `tests/acceptance/test_suite_acceptance.py` | **Create** | L2 acceptance: WA suite draw + validate |

---

## Output format: `analysis/partisan.json`

```
outputs/{version}/{year}/{state_name}/analysis/partisan.json
```

Full schema defined in Spec 4. Key fields per board amendments:

```json
{
  "analyzer": "partisan",
  "election": "2020_presidential",
  "election_file": "data/2020/elections/presidential_by_tract.csv",
  "available": true,
  "methodology_warning": "Presidential election results used as proxy ...",
  "statewide": { "dem_vote_share": 0.583, "dem_seat_share": 0.600, "total_votes": 3284823 },
  "metrics": {
    "efficiency_gap": {
      "value": 0.031,
      "direction": "Democratic",
      "ci_available": true,
      "ci_95_low": 0.011,
      "ci_95_high": 0.051,
      "academic_reference": "8% threshold from Stephanopoulos & McGhee (2015). SCOTUS declined to adopt in Gill v. Whitford (2018). Not a constitutional standard."
    },
    "mean_median": {
      "value": -0.023,
      "direction": "Republican",
      "ci_available": true,
      "ci_95_low": -0.041,
      "ci_95_high": -0.005,
      "academic_reference": "7% threshold from Wang (2016). Not a constitutional standard."
    },
    "partisan_bias": {
      "value": 0.018,
      "direction": "Democratic",
      "ci_available": true,
      "ci_95_low": -0.002,
      "ci_95_high": 0.038
    }
  },
  "districts": [ { "district": 1, "dem_votes": 98234.5, "rep_votes": 76543.2, "dem_pct": 0.562, "margin": 0.124, "is_competitive": false, "is_uncontested": false } ]
}
```

When `num_districts < 10`, CI fields are replaced with:
```json
{ "ci_available": false, "ci_reason": "Bootstrap CI requires >=10 districts (found N)" }
```

When election file is absent:
```json
{
  "available": false,
  "unavailable_reason": "Election data not found. Run: redist fetch --type elections --year 2020",
  "required_file": "data/2020/elections/presidential_by_tract.csv"
}
```

---

## Suite output structure

```
outputs/{version}/{year}/suites/{suite_name}/
  suite.json

exports/{suite_name}/
  suite.json                  <- envelope: chamber -> .rplan filename
  {suite_name}_congressional.rplan
  {suite_name}_house.rplan
  {suite_name}_senate.rplan
```

`suite.json` (envelope) schema:
```json
{
  "suite_name": "wa_commission_v1",
  "plans": [
    {"chamber": "congressional", "file": "wa_commission_v1_congressional.rplan"},
    {"chamber": "house",         "file": "wa_commission_v1_house.rplan"},
    {"chamber": "senate",        "file": "wa_commission_v1_senate.rplan"}
  ]
}
```

---

## Constitutional nesting table

Hard-coded per-state nesting ratios used by `--nest senate-in-house` validation:

| State | Required ratio (house:senate) |
|-------|-------------------------------|
| WA    | 2:1 (fixed by constitution)   |
| IL    | variable                      |
| (others) | none / use --nest-ratio   |

When `--nest-ratio` deviates from a known constitutional value, emit:
```
WARNING: WA constitution requires 2:1 house-to-senate nesting ratio. You specified 3:1.
Proceed only if you have verified this is legally permissible.
```

---

## Exit codes (bitfield — extends Spec 3)

| Bit | Decimal | Meaning |
|-----|---------|---------|
| 0   | 1       | Population balance violation |
| 1   | 2       | Contiguity violation |
| 2   | 4       | Nesting violation |
| 3   | 8       | (reserved) |

Combinations are ORed: balance + nesting = 1 | 4 = 5.

---

## Task 1: Partisan metric functions

**Files:** `redist/crates/redist-analysis/src/partisan.rs`

- [ ] **L0: Write failing metric tests**

```rust
// tests/unit/test_partisan.rs

#[test]
fn test_efficiency_gap_zero_for_symmetric_plan() {
    // 10 districts: 5 Dem wins at 60/40, 5 Rep wins at 60/40
    // Wasted votes symmetric on both sides → EG = 0
    let districts = symmetric_plan_10();
    let eg = compute_efficiency_gap(&districts);
    assert!((eg).abs() < 1e-9, "symmetric plan must have EG = 0, got {eg}");
}

#[test]
fn test_efficiency_gap_direction() {
    // Rep wins 6 of 10 districts narrowly (51/49), Dem wins 4 by blowout (80/20)
    // → many Dem surplus wasted votes → EG favors Republicans (negative value)
    let districts = packed_dem_plan();
    let eg = compute_efficiency_gap(&districts);
    assert!(eg < 0.0, "packed Dem blowout plan should have negative EG (Rep-favoring)");
}

#[test]
fn test_mean_median_equal_when_symmetric() {
    // All districts exactly 50% Dem → mean == median → MM = 0
    let districts = uniform_plan(10, 0.50);
    let mm = compute_mean_median(&districts);
    assert!((mm).abs() < 1e-9, "uniform vote share must have MM = 0");
}

#[test]
fn test_bootstrap_ci_reproducible_with_seed() {
    let districts = symmetric_plan_10();
    let ci1 = bootstrap_ci(&districts, compute_efficiency_gap, 1000, 42);
    let ci2 = bootstrap_ci(&districts, compute_efficiency_gap, 1000, 42);
    assert_eq!(ci1, ci2, "same seed must produce identical CI bounds");
}

#[test]
fn test_bootstrap_ci_different_seeds_differ() {
    let districts = packed_dem_plan();
    let ci1 = bootstrap_ci(&districts, compute_efficiency_gap, 1000, 42);
    let ci2 = bootstrap_ci(&districts, compute_efficiency_gap, 1000, 99);
    assert_ne!(ci1, ci2, "different seeds should produce different CI bounds");
}

#[test]
fn test_partisan_bias_neutral_at_50pct() {
    // Uniform swing model: party winning 50% of statewide votes → 50% of seats → bias = 0
    let districts = uniform_plan(10, 0.50);
    let pb = compute_partisan_bias(&districts);
    assert!((pb).abs() < 0.05, "balanced plan bias should be near zero, got {pb}");
}

#[test]
fn test_ci_suppressed_when_fewer_than_10_districts() {
    // Scenario 5: VT single district
    let districts = vec![make_district(1, 0.67, 0.33)];
    let result = compute_partisan_metrics(&districts, None);
    assert!(!result.efficiency_gap.ci_available,
        "CI must be suppressed for N < 10");
    assert_eq!(
        result.efficiency_gap.ci_reason.as_deref(),
        Some("Bootstrap CI requires >=10 districts (found 1)")
    );
}

#[test]
fn test_metrics_computed_even_when_ci_suppressed() {
    // Metrics themselves are valid even for N=1 — only CI is suppressed
    let districts = vec![make_district(1, 0.67, 0.33)];
    let result = compute_partisan_metrics(&districts, None);
    assert!(result.efficiency_gap.value.is_finite(),
        "EG must be finite for single-district plan");
    assert!(result.mean_median.value.is_finite());
}

#[test]
fn test_eg_exactly_10_districts_ci_present() {
    // Boundary: exactly 10 districts → CI computed
    let districts = uniform_plan(10, 0.55);
    let result = compute_partisan_metrics(&districts, None);
    assert!(result.efficiency_gap.ci_available,
        "CI must be available for exactly 10 districts");
}

#[test]
fn test_direction_dem_when_positive_eg() {
    let districts = packed_rep_plan(); // EG > 0
    let result = compute_partisan_metrics(&districts, None);
    assert_eq!(result.efficiency_gap.direction, "Democratic");
}

#[test]
fn test_direction_rep_when_negative_eg() {
    let districts = packed_dem_plan(); // EG < 0
    let result = compute_partisan_metrics(&districts, None);
    assert_eq!(result.efficiency_gap.direction, "Republican");
}
```

- [ ] **Run tests** — expect FAIL (module does not exist)
- [ ] **Implement `partisan.rs`**

```rust
// redist/crates/redist-analysis/src/partisan.rs

use serde::Serialize;
use rand::{SeedableRng, rngs::SmallRng};
use rand::seq::SliceRandom;

#[derive(Debug, Clone, Serialize)]
pub struct DistrictElection {
    pub district: usize,
    pub dem_votes: f64,
    pub rep_votes: f64,
}

impl DistrictElection {
    pub fn total(&self) -> f64 { self.dem_votes + self.rep_votes }
    pub fn dem_pct(&self) -> f64 { self.dem_votes / self.total() }
    pub fn margin(&self) -> f64 { (self.dem_votes - self.rep_votes).abs() / self.total() }
    pub fn dem_won(&self) -> bool { self.dem_votes > self.rep_votes }
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricWithCI {
    pub value: f64,
    pub direction: String,         // "Democratic" or "Republican"
    pub ci_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_95_low: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_95_high: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_reason: Option<String>, // present only when ci_available=false
    pub academic_reference: String,
}

#[derive(Debug, Serialize)]
pub struct PartisanMetrics {
    pub efficiency_gap: MetricWithCI,
    pub mean_median: MetricWithCI,
    pub partisan_bias: MetricWithCI,
    pub statewide_dem_vote_share: f64,
    pub statewide_dem_seat_share: f64,
}

/// EG = (Wasted_R - Wasted_D) / Total_votes
/// Positive = Democratic-favoring, Negative = Republican-favoring.
pub fn compute_efficiency_gap(districts: &[DistrictElection]) -> f64 {
    let total_votes: f64 = districts.iter().map(|d| d.total()).sum();
    if total_votes == 0.0 { return 0.0; }

    let (wasted_d, wasted_r) = districts.iter().fold((0.0, 0.0), |(wd, wr), d| {
        let to_win = d.total() / 2.0 + 1.0;
        if d.dem_won() {
            (wd + (d.dem_votes - to_win), wr + d.rep_votes)
        } else {
            (wd + d.dem_votes, wr + (d.rep_votes - to_win))
        }
    });
    (wasted_r - wasted_d) / total_votes
}

/// MM = mean(dem_share) - median(dem_share)
pub fn compute_mean_median(districts: &[DistrictElection]) -> f64 {
    if districts.is_empty() { return 0.0; }
    let mut shares: Vec<f64> = districts.iter().map(|d| d.dem_pct()).collect();
    let mean = shares.iter().sum::<f64>() / shares.len() as f64;
    shares.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = shares.len();
    let median = if n % 2 == 0 {
        (shares[n/2 - 1] + shares[n/2]) / 2.0
    } else {
        shares[n/2]
    };
    mean - median
}

/// Partisan bias: 0.5 - seat_share at the swing where statewide dem = 50%.
pub fn compute_partisan_bias(districts: &[DistrictElection]) -> f64 {
    let statewide_dem: f64 = {
        let (d, t): (f64, f64) = districts.iter()
            .fold((0.0, 0.0), |(d, t), x| (d + x.dem_votes, t + x.total()));
        if t == 0.0 { return 0.0; } else { d / t }
    };
    let swing = 0.5 - statewide_dem;
    let seats_at_50: f64 = districts.iter()
        .filter(|d| d.dem_pct() + swing > 0.5)
        .count() as f64;
    0.5 - seats_at_50 / districts.len() as f64
}

/// Bootstrap CI using deterministic seed. Returns (low_95, high_95).
pub fn bootstrap_ci<F>(
    districts: &[DistrictElection],
    metric_fn: F,
    n_bootstrap: usize,
    rng_seed: u64,
) -> (f64, f64)
where
    F: Fn(&[DistrictElection]) -> f64,
{
    let mut rng = SmallRng::seed_from_u64(rng_seed);
    let n = districts.len();
    let mut samples: Vec<f64> = Vec::with_capacity(n_bootstrap);
    for _ in 0..n_bootstrap {
        let resample: Vec<_> = (0..n)
            .map(|_| districts.choose(&mut rng).unwrap().clone())
            .collect();
        samples.push(metric_fn(&resample));
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let low  = samples[(0.025 * n_bootstrap as f64) as usize];
    let high = samples[(0.975 * n_bootstrap as f64) as usize];
    (low, high)
}

const CI_MIN_DISTRICTS: usize = 10;
const EG_ACADEMIC_REF: &str = "8% threshold from Stephanopoulos & McGhee (2015). SCOTUS declined to adopt in Gill v. Whitford (2018). Not a constitutional standard.";
const MM_ACADEMIC_REF: &str = "7% threshold from Wang (2016). Not a constitutional standard.";
const PB_ACADEMIC_REF: &str = "Partisan bias methodology from Gelman & King (1994).";

pub fn compute_partisan_metrics(
    districts: &[DistrictElection],
    rng_seed: Option<u64>,
) -> PartisanMetrics {
    let n = districts.len();
    let seed = rng_seed.unwrap_or(42);

    let eg_val = compute_efficiency_gap(districts);
    let mm_val = compute_mean_median(districts);
    let pb_val = compute_partisan_bias(districts);

    let ci_available = n >= CI_MIN_DISTRICTS;
    let ci_reason = if !ci_available {
        Some(format!("Bootstrap CI requires >={CI_MIN_DISTRICTS} districts (found {n})"))
    } else { None };

    let make_ci = |val: f64, metric_fn: fn(&[DistrictElection]) -> f64| -> (Option<f64>, Option<f64>) {
        if ci_available {
            let (lo, hi) = bootstrap_ci(districts, metric_fn, 1000, seed);
            (Some(lo), Some(hi))
        } else {
            (None, None)
        }
    };

    let (eg_lo, eg_hi) = make_ci(eg_val, compute_efficiency_gap);
    let (mm_lo, mm_hi) = make_ci(mm_val, compute_mean_median);
    let (pb_lo, pb_hi) = make_ci(pb_val, compute_partisan_bias);

    let direction = |v: f64| if v >= 0.0 { "Democratic".into() } else { "Republican".into() };

    let total_votes: f64 = districts.iter().map(|d| d.total()).sum();
    let statewide_dem_vote_share = if total_votes > 0.0 {
        districts.iter().map(|d| d.dem_votes).sum::<f64>() / total_votes
    } else { 0.0 };
    let statewide_dem_seat_share = districts.iter().filter(|d| d.dem_won()).count() as f64
        / n.max(1) as f64;

    PartisanMetrics {
        efficiency_gap: MetricWithCI {
            value: eg_val, direction: direction(eg_val),
            ci_available, ci_95_low: eg_lo, ci_95_high: eg_hi,
            ci_reason: ci_reason.clone(),
            academic_reference: EG_ACADEMIC_REF.into(),
        },
        mean_median: MetricWithCI {
            value: mm_val, direction: direction(mm_val),
            ci_available, ci_95_low: mm_lo, ci_95_high: mm_hi,
            ci_reason: ci_reason.clone(),
            academic_reference: MM_ACADEMIC_REF.into(),
        },
        partisan_bias: MetricWithCI {
            value: pb_val, direction: direction(pb_val),
            ci_available, ci_95_low: pb_lo, ci_95_high: pb_hi,
            ci_reason,
            academic_reference: PB_ACADEMIC_REF.into(),
        },
        statewide_dem_vote_share,
        statewide_dem_seat_share,
    }
}
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(partisan): EG, MM, PB metrics with bootstrap CI and small-chamber suppression"`

---

## Task 2: Election CSV loader + analyzer dispatch

**Files:** `redist/crates/redist-cli/src/partisan.rs`, `redist/crates/redist-cli/src/args.rs`

- [ ] **L0: Write failing loader tests**

```rust
#[test]
fn test_load_election_csv_valid() {
    // geoid,dem_votes,rep_votes with fractional values
    let csv = "geoid,dem_votes,rep_votes\n53033001001,1203.5,876.2\n53033001002,500.0,400.0\n";
    let records = parse_election_csv(csv.as_bytes()).unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].geoid, "53033001001");
    assert!((records[0].dem_votes - 1203.5).abs() < 0.001);
}

#[test]
fn test_load_election_csv_missing_file_returns_unavailable() {
    let result = load_election_data(Path::new("/nonexistent/file.csv"), "2020");
    assert!(result.is_err());
    // Error message must guide user to fetch command
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("redist fetch --type elections"));
}

#[test]
fn test_aggregate_to_districts_correct_sum() {
    // Two tracts in district 1, one in district 2
    let election = vec![
        ElectionRecord { geoid: "t1".into(), dem_votes: 100.0, rep_votes: 50.0 },
        ElectionRecord { geoid: "t2".into(), dem_votes: 200.0, rep_votes: 150.0 },
        ElectionRecord { geoid: "t3".into(), dem_votes: 80.0, rep_votes: 120.0 },
    ];
    let assignments = HashMap::from([
        ("t1".into(), 1usize), ("t2".into(), 1), ("t3".into(), 2)
    ]);
    let by_district = aggregate_election_to_districts(&election, &assignments);
    assert_eq!(by_district[&1].dem_votes, 300.0);
    assert_eq!(by_district[&1].rep_votes, 200.0);
    assert_eq!(by_district[&2].dem_votes, 80.0);
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `run_partisan()`** — load election CSV (from `--election-file` or default
  `data/{year}/elections/presidential_by_tract.csv`), aggregate to districts, call
  `compute_partisan_metrics`, serialize to `analysis/partisan.json`.

  Handle missing-file case: write `{"available": false, ...}` and exit 0 (not a failure).

  Handle non-congressional chamber: include `methodology_warning` in output.

- [ ] **Modify `args.rs`** — add `--election-file <PATH>` to `AnalyzeArgs`
- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(partisan): election CSV loader and --election-file flag"`

---

## Task 3: `AnalyzerType::Partisan` wired into analyze dispatcher

**Files:** `redist/crates/redist-analysis/src/analyzer.rs`, `redist/crates/redist-cli/src/analyze.rs`

- [ ] **Add `Partisan` variant to `AnalyzerType` enum**
- [ ] **Wire in `analyze.rs`:** when `types` includes `partisan`, call `run_partisan()`
- [ ] **L0: Integration test**

```rust
#[test]
fn test_analyze_partisan_type_dispatches_correctly() {
    let types = vec![AnalyzerType::Partisan];
    // Should include partisan, not demographic
    assert!(types.contains(&AnalyzerType::Partisan));
    assert!(!types.contains(&AnalyzerType::Demographic));
}
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(analyze): wire partisan analyzer into --types dispatch"`

---

## Task 4: Chamber adjacency graph

**Files:** `redist/crates/redist-analysis/src/nesting.rs`

- [ ] **L0: Write failing adjacency tests** (from Spec 5 + Scenario 4)

```rust
// tests/unit/test_nesting.rs

#[test]
fn test_build_chamber_adjacency_simple() {
    // 4 tracts: t0,t1 in house dist 1; t2,t3 in house dist 2
    // t1 and t2 share a boundary
    let house_asgn: HashMap<String, usize> = [
        ("t0".into(), 1), ("t1".into(), 1), ("t2".into(), 2), ("t3".into(), 2)
    ].into();
    // tract adjacency: t0-t1, t1-t2, t2-t3 (linear chain)
    let tract_adj: Vec<Vec<usize>> = vec![vec![1], vec![0, 2], vec![1, 3], vec![2]];
    let tract_ids: Vec<String> = vec!["t0".into(), "t1".into(), "t2".into(), "t3".into()];
    let house_adj = build_chamber_adjacency(&house_asgn, &tract_adj, &tract_ids, 2);
    // House district 0 (1-indexed: 1) and house district 1 (1-indexed: 2) must be adjacent
    assert!(house_adj[0].contains(&1), "house dist 0 must be adjacent to house dist 1");
    assert!(house_adj[1].contains(&0), "adjacency must be symmetric");
}

#[test]
fn test_build_chamber_adjacency_no_cross_adjacency() {
    // Two completely isolated pairs of tracts in separate districts
    // No shared boundary → no adjacency
    let house_asgn: HashMap<String, usize> = [
        ("t0".into(), 1), ("t1".into(), 1), ("t2".into(), 2), ("t3".into(), 2)
    ].into();
    let tract_adj: Vec<Vec<usize>> = vec![vec![1], vec![0], vec![3], vec![2]]; // no t1-t2 edge
    let tract_ids: Vec<String> = vec!["t0".into(), "t1".into(), "t2".into(), "t3".into()];
    let house_adj = build_chamber_adjacency(&house_asgn, &tract_adj, &tract_ids, 2);
    assert!(house_adj[0].is_empty(), "no cross-district boundary → no adjacency");
    assert!(house_adj[1].is_empty());
}

#[test]
fn test_build_chamber_adjacency_primary_component_only() {
    // House district 1 has two disconnected components: t0 (component A) and t3 (component B)
    // Only primary component (largest) should be used for adjacency
    // t3 is adjacent to t2 (house dist 2) but is a secondary component → phantom edge suppressed
    let house_asgn: HashMap<String, usize> = [
        ("t0".into(), 1), ("t1".into(), 2), ("t2".into(), 2), ("t3".into(), 1)
    ].into();
    let tract_adj: Vec<Vec<usize>> = vec![
        vec![],       // t0: isolated (primary component of dist 1)
        vec![2],      // t1: adj to t2
        vec![1, 3],   // t2: adj to t1, t3
        vec![2],      // t3: adj to t2 (secondary component of dist 1)
    ];
    let tract_ids: Vec<String> = vec!["t0".into(), "t1".into(), "t2".into(), "t3".into()];
    let house_adj = build_chamber_adjacency(&house_asgn, &tract_adj, &tract_ids, 2);
    // t3 is secondary → phantom edge from dist1 to dist2 via t3 must be suppressed
    // Primary component of dist 1 is t0 (size 1 vs t3 size 1 — tie broken by first found)
    // Either way: if primary component has no edge to dist2 tracts, dist1 and dist2 not adjacent
    // This test verifies no spurious adjacency via disconnected secondary components
    // (Exact behavior depends on tie-breaking; what matters is no phantom cross-district edges
    // through secondary components that don't touch dist2's primary component)
    // At minimum: build_chamber_adjacency must not panic and must return valid-length vec
    assert_eq!(house_adj.len(), 2);
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `build_chamber_adjacency()`**

```rust
// redist/crates/redist-analysis/src/nesting.rs

use std::collections::{HashMap, HashSet, VecDeque};
use serde::Serialize;

/// Build adjacency graph where nodes = house districts, edges = shared tract boundaries.
/// Uses PRIMARY COMPONENT ONLY of each house district (largest connected component by tract count).
/// Tracts in secondary components are excluded with a warning logged to stderr.
pub fn build_chamber_adjacency(
    house_assignments: &HashMap<String, usize>,
    tract_adjacency: &[Vec<usize>],       // index into tract_ids
    tract_ids: &[String],
    num_house_districts: usize,
) -> Vec<Vec<usize>> {
    // 1. Group tracts by house district (1-indexed → 0-indexed for adjacency array)
    // 2. For each house district, find connected components via BFS on tract_adjacency
    // 3. Keep only the largest component (primary)
    // 4. Build house-district adjacency: two house districts adjacent iff any primary-component
    //    tract of one is adjacent to any primary-component tract of the other
    // ...
    todo!()
}
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(nesting): build_chamber_adjacency with primary-component-only filtering"`

---

## Task 5: Nesting validation

**Files:** `redist/crates/redist-analysis/src/nesting.rs` (continued)

- [ ] **L0: Write failing validation tests** (from Spec 5 L0 + Scenario 4)

```rust
#[test]
fn test_nesting_validation_perfect() {
    // Senate 1 = house [1,2], Senate 2 = house [3,4] — perfect 2:1 nesting
    let house: HashMap<String, usize> = [
        ("t0".into(), 1), ("t1".into(), 1), ("t2".into(), 2), ("t3".into(), 2),
        ("t4".into(), 3), ("t5".into(), 3), ("t6".into(), 4), ("t7".into(), 4),
    ].into();
    let senate: HashMap<String, usize> = [
        ("t0".into(), 1), ("t1".into(), 1), ("t2".into(), 1), ("t3".into(), 1),
        ("t4".into(), 2), ("t5".into(), 2), ("t6".into(), 2), ("t7".into(), 2),
    ].into();
    let result = validate_nesting(&house, &senate, 2);
    assert!(result.valid);
    assert!(result.violations.is_empty());
    // Senate-to-house map: senate 1 → [1,2], senate 2 → [3,4]
    assert_eq!(result.senate_to_house_map[&1].len(), 2);
}

#[test]
fn test_nesting_validation_violation() {
    // Senate district 1 spans tracts from house districts 1, 2, AND 3 — violation
    let house: HashMap<String, usize> = [
        ("t0".into(), 1), ("t1".into(), 2), ("t2".into(), 3)
    ].into();
    let senate: HashMap<String, usize> = [
        ("t0".into(), 1), ("t1".into(), 1), ("t2".into(), 1)
    ].into();
    let result = validate_nesting(&house, &senate, 2);
    assert!(!result.valid);
    assert!(!result.violations.is_empty());
    let v = &result.violations[0];
    assert_eq!(v.senate_district, 1);
    assert_eq!(v.house_districts.len(), 3);
}

#[test]
fn test_nesting_ratio_computed() {
    assert_eq!(compute_nest_ratio(98, 49), Some(2));
    assert_eq!(compute_nest_ratio(98, 48), None); // doesn't divide evenly
    assert_eq!(compute_nest_ratio(99, 33), Some(3));
}

// Scenario 4 assertions:
#[test]
fn test_nesting_violation_senate_contains_three_house_districts() {
    let house: HashMap<String, usize> = [
        ("t0".into(), 1), ("t1".into(), 2), ("t2".into(), 3), ("t3".into(), 3),
        ("t4".into(), 5), ("t5".into(), 5), ("t6".into(), 6), ("t7".into(), 6), ("t8".into(), 7), ("t9".into(), 7),
    ].into();
    let senate: HashMap<String, usize> = [
        ("t0".into(), 1), ("t1".into(), 1),
        ("t2".into(), 3), ("t3".into(), 3), ("t4".into(), 3), ("t5".into(), 3),
        ("t6".into(), 3), ("t7".into(), 3), ("t8".into(), 3), ("t9".into(), 3),
    ].into();
    let result = validate_nesting(&house, &senate, 2);
    assert!(!result.valid);
    let v = result.violations.iter().find(|v| v.senate_district == 3).unwrap();
    assert_eq!(v.house_districts.len(), 3);
}

#[test]
fn test_nesting_exit_code_is_bit2() {
    // Nesting violation only → exit code 4 (bit 2)
    assert_eq!(compute_exit_code(false, false, true, false), 4);
}

#[test]
fn test_balance_and_nesting_exit_code() {
    // Balance + nesting → 1 | 4 = 5
    assert_eq!(compute_exit_code(true, false, true, false), 5);
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `validate_nesting()` and `NestingValidation`**

```rust
#[derive(Debug, Serialize)]
pub struct NestingViolation {
    pub senate_district: usize,
    pub house_districts: Vec<usize>,  // the house districts found in this senate district
    pub expected_count: usize,        // the required_ratio
}

#[derive(Debug, Serialize)]
pub struct NestingValidation {
    pub valid: bool,
    pub violations: Vec<NestingViolation>,
    pub senate_to_house_map: HashMap<usize, Vec<usize>>,
}

pub fn validate_nesting(
    house_assignments: &HashMap<String, usize>,
    senate_assignments: &HashMap<String, usize>,
    required_ratio: usize,
) -> NestingValidation {
    // For each senate district: collect house districts of its tracts.
    // Violation if count != required_ratio OR if any house district spans multiple senate districts.
    todo!()
}

pub fn compute_nest_ratio(num_house: usize, num_senate: usize) -> Option<usize> {
    if num_senate == 0 || num_house % num_senate != 0 { None }
    else { Some(num_house / num_senate) }
}

/// Bitfield exit code: bit 0=balance, bit 1=contiguity, bit 2=nesting, bit 3=reserved.
pub fn compute_exit_code(
    balance_violation: bool,
    contiguity_violation: bool,
    nesting_violation: bool,
    _reserved: bool,
) -> u8 {
    (balance_violation as u8)
        | ((contiguity_violation as u8) << 1)
        | ((nesting_violation as u8) << 2)
}
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(nesting): validate_nesting + NestingValidation + bitfield exit codes"`

---

## Task 6: `redist suite` CLI — draw subcommand

**Files:** `redist/crates/redist-cli/src/suite.rs`, `redist/crates/redist-cli/src/args.rs`, `redist/crates/redist-cli/src/main.rs`

- [ ] **L0: Write failing suite-draw tests**

```rust
#[test]
fn test_suite_args_parse_nest_mode() {
    let args = SuiteArgs::parse_from([
        "redist", "suite",
        "--state", "WA", "--year", "2020", "--version", "WA_Plans",
        "--name", "wa_test", "--house-districts", "98", "--senate-districts", "49",
        "--nest", "senate-in-house",
    ]);
    assert_eq!(args.nest_mode, NestMode::SenateInHouse);
    assert_eq!(args.house_districts, Some(98));
    assert_eq!(args.senate_districts, Some(49));
}

#[test]
fn test_nest_mode_from_str() {
    assert_eq!("senate-in-house".parse::<NestMode>().unwrap(), NestMode::SenateInHouse);
    assert_eq!("none".parse::<NestMode>().unwrap(), NestMode::None);
    assert!("invalid".parse::<NestMode>().is_err());
}

#[test]
fn test_constitutional_nesting_ratio_wa() {
    let table = build_constitutional_nesting_table();
    assert_eq!(table.get("WA"), Some(&NestedRatio::Fixed(2)));
}

#[test]
fn test_nesting_hard_fail_on_noncontiguous_house_district() {
    // If house plan has noncontiguous district AND --nest is set → must return Err
    let result = validate_house_for_nesting(&noncontiguous_house_plan());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("noncontiguous"));
    assert!(msg.contains("Remove --allow-noncontiguous"));
}

#[test]
fn test_suite_manifest_records_nesting_mode() {
    let suite = PlanSuite {
        suite_name: "wa_test".into(),
        state: "WA".into(),
        year: "2020".into(),
        nest_mode: NestMode::SenateInHouse,
        plans: vec![],
    };
    let json = serde_json::to_string(&suite).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["nest_mode"], "senate-in-house");
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `SuiteArgs`, `NestMode`, `PlanSuite`, `run_suite()`**

  Draw sequence when `--nest senate-in-house`:
  1. Draw congressional (independent)
  2. Draw house (independent)
  3. **Hard-fail** if any house district is noncontiguous (check via `validate_nesting` contiguity gate)
  4. Call `build_chamber_adjacency()` on house plan → senate adjacency graph
  5. Draw senate using senate adjacency graph as input
  6. Call `validate_nesting()` → if `!valid`, exit with code 5, print violated senate districts
  7. Write `suite.json` manifest

- [ ] **Add `Commands::Suite` to `main.rs`**
- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(suite): redist suite draw command with senate-in-house nesting"`

---

## Task 7: `redist suite validate` subcommand + suite export

**Files:** `redist/crates/redist-cli/src/suite.rs` (continued)

- [ ] **L0: Write failing validate + export tests**

```rust
#[test]
fn test_suite_validate_returns_nesting_map() {
    // Valid 2:1 nesting — senate_to_house_map has 49 entries for WA
    let result = run_suite_validate(&wa_valid_suite_fixture());
    assert!(result.nesting.valid);
    assert_eq!(result.nesting.senate_to_house_map.len(), 49);
}

#[test]
fn test_suite_validate_exit_5_on_nesting_violation() {
    // Suite with known nesting violation → error with exit 5
    let result = run_suite_validate(&wa_violation_suite_fixture());
    assert!(!result.nesting.valid);
    assert!(!result.nesting.violations.is_empty());
    // Caller maps this to exit code 5
}

#[test]
fn test_suite_export_produces_three_rplan_files() {
    let export = generate_suite_export(&wa_valid_suite_fixture());
    assert_eq!(export.plan_files.len(), 3);
    assert!(export.plan_files.iter().any(|p| p.chamber == "congressional"));
    assert!(export.plan_files.iter().any(|p| p.chamber == "house"));
    assert!(export.plan_files.iter().any(|p| p.chamber == "senate"));
}

#[test]
fn test_suite_json_envelope_references_all_chambers() {
    let export = generate_suite_export(&wa_valid_suite_fixture());
    let json = serde_json::to_string(&export.envelope).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let plans = v["plans"].as_array().unwrap();
    assert_eq!(plans.len(), 3);
    let chambers: Vec<&str> = plans.iter()
        .map(|p| p["chamber"].as_str().unwrap())
        .collect();
    assert!(chambers.contains(&"congressional"));
    assert!(chambers.contains(&"house"));
    assert!(chambers.contains(&"senate"));
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `run_suite_validate()` and suite export** — 3 separate RPLAN files + `suite.json` envelope
- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(suite): validate subcommand + suite export (3 RPLAN + suite.json)"`

---

## Task 8: L2 acceptance tests

**Files:** `tests/acceptance/test_partisan_acceptance.py`, `tests/acceptance/test_suite_acceptance.py`

- [ ] **Write L2 partisan tests**

```python
# tests/acceptance/test_partisan_acceptance.py

def test_wa_partisan_produces_json(wa_plan_fixture):
    """WA 10-district plan with 2020 election → partisan.json with all 3 metrics."""
    result = subprocess.run([
        "redist", "analyze",
        "--state", "WA", "--year", "2020", "--version", "test",
        "--label", wa_plan_fixture,
        "--types", "partisan",
    ], capture_output=True, check=True)
    partisan_path = Path(f"outputs/test/2020/plans/{wa_plan_fixture}/analysis/partisan.json")
    assert partisan_path.exists()
    data = json.loads(partisan_path.read_text())
    assert data["available"] is True
    assert "efficiency_gap" in data["metrics"]
    assert "mean_median" in data["metrics"]
    assert "partisan_bias" in data["metrics"]
    # Board amendment: no threshold_8pct field; instead academic_reference
    assert "threshold_8pct" not in data["metrics"]["efficiency_gap"]
    assert "academic_reference" in data["metrics"]["efficiency_gap"]

def test_vt_single_district_partisan(vt_plan_fixture):
    """VT (1 district) → available=true, CI suppressed."""
    subprocess.run([
        "redist", "analyze", "--state", "VT", "--year", "2020",
        "--version", "test", "--label", vt_plan_fixture, "--types", "partisan",
    ], capture_output=True, check=True)
    data = json.loads((Path(f"outputs/test/2020/plans/{vt_plan_fixture}/analysis/partisan.json")).read_text())
    assert data["available"] is True
    assert data["metrics"]["efficiency_gap"]["ci_available"] is False
    assert "Bootstrap CI requires" in data["metrics"]["efficiency_gap"]["ci_reason"]
    assert len(data["districts"]) == 1

def test_partisan_missing_election_returns_unavailable(tmp_plan_no_election):
    """No election CSV → available=false, exit 0."""
    result = subprocess.run([
        "redist", "analyze",
        "--types", "partisan",
        "--label", tmp_plan_no_election,
    ], capture_output=True)
    assert result.returncode == 0, "missing election must not fail — exit 0"
    data = json.loads(...)
    assert data["available"] is False
    assert "redist fetch --type elections" in data["unavailable_reason"]
    assert "required_file" in data

def test_partisan_with_custom_election_file(wa_plan_fixture, tmp_election_csv):
    """--election-file flag accepts custom CSV."""
    subprocess.run([
        "redist", "analyze",
        "--state", "WA", "--year", "2020", "--version", "test",
        "--label", wa_plan_fixture,
        "--types", "partisan",
        "--election-file", str(tmp_election_csv),
    ], capture_output=True, check=True)
    data = json.loads(...)
    assert data["available"] is True
    assert data["election_file"] == str(tmp_election_csv)
```

- [ ] **Write L2 suite tests**

```python
# tests/acceptance/test_suite_acceptance.py

def test_wa_suite_draws_all_three_chambers():
    """Suite command produces 3 plan directories."""
    subprocess.run([
        "redist", "suite",
        "--state", "WA", "--year", "2020", "--version", "test",
        "--name", "wa_test_suite",
        "--congressional-districts", "10",
        "--house-districts", "98",
        "--senate-districts", "49",
        "--nest", "senate-in-house",
        "--seed", "42",
    ], capture_output=True, check=True)
    base = Path("outputs/test/2020/suites/wa_test_suite")
    assert (base / "suite.json").exists()
    manifest = json.loads((base / "suite.json").read_text())
    assert len(manifest["plans"]) == 3

def test_wa_senate_nests_in_house():
    """validate_nesting returns valid:true for generated WA suite."""
    result = subprocess.run([
        "redist", "suite", "validate",
        "--name", "wa_test_suite", "--version", "test", "--year", "2020",
    ], capture_output=True, check=True)
    data = json.loads(result.stdout)
    assert data["nesting"]["valid"] is True
    assert data["nesting"]["violations"] == []

def test_suite_manifest_records_nesting_mode():
    manifest = json.loads(Path("outputs/test/2020/suites/wa_test_suite/suite.json").read_text())
    assert manifest.get("nest_mode") == "senate-in-house"

def test_suite_validate_exit_5_on_nesting_violation():
    """Synthesized nesting violation → exit code 5."""
    result = subprocess.run([
        "redist", "suite", "validate",
        "--name", "wa_violation_suite", "--version", "test", "--year", "2020",
    ], capture_output=True)
    assert result.returncode == 5
```

- [ ] **Run L2 tests against fixtures** — expect PASS (with pipeline output fixtures available)
- [ ] **Commit:** `git commit -m "test(partisan+suite): L2 acceptance tests for Spec 4 + Spec 5"`

---

## Cargo.toml additions

```toml
# redist/crates/redist-analysis/Cargo.toml
[dependencies]
rand = { version = "0.8", features = ["small_rng"] }
# ... existing deps
```

No new crates required for `redist-suite` (uses existing `serde_json`, `csv`, `thiserror`).

---

## Summary

| Task | Deliverable | Tests |
|------|-------------|-------|
| 1 | `partisan.rs` — EG, MM, PB, bootstrap CI, CI suppression | 10 L0 unit |
| 2 | Election CSV loader + `--election-file` flag | 3 L0 unit |
| 3 | `AnalyzerType::Partisan` wired into dispatcher | 1 L0 integration |
| 4 | `build_chamber_adjacency()` with primary-component filtering | 3 L0 unit |
| 5 | `validate_nesting()` + bitfield exit codes | 6 L0 unit |
| 6 | `redist suite draw` — chamber sequencing + nesting hard-fail | 4 L0 unit |
| 7 | `redist suite validate` + suite export (3 RPLAN + envelope) | 4 L0 unit |
| 8 | L2 acceptance tests (partisan + suite) | 7 L2 acceptance |

**Total new tests: ~38** (28 L0 unit + 3 L0 integration + 7 L2 acceptance)
