# Spec 4: Partisan Metrics

**Date**: 2026-04-26
**Status**: Draft
**Dependencies**: Spec 1 (Plan + PlanManifest)
**Depended on by**: Spec 2 (Plan Comparison), Spec 6 (Reports)

---

## Problem

Redistricting commissions and courts increasingly require quantitative partisan fairness analysis. The three legally recognized metrics are:

1. **Efficiency Gap** — measures "wasted votes" asymmetry between parties (Stephanopoulos & McGhee, 2015)
2. **Mean-Median Difference** — difference between a party's mean district vote share and its median (Wang, 2016)
3. **Partisan Bias** — the seat share a party would win if it got exactly 50% of the statewide vote

Additionally:
4. **Declination** — geometric measure of vote/seat relationship asymmetry (Warrington, 2018)
5. **Responsiveness** — how much seat share changes per unit of vote share near 50%

These are already surfaced in the dashboard. This spec makes them first-class `redist analyze` outputs with proper statistical confidence intervals.

---

## New analyzer: `partisan`

Extends Spec 1's analyzer extension model:

```bash
redist analyze --state WA --year 2020 --version WA_Plans \
  --label wa_house_draft1 --types partisan
```

Requires: `data/{year}/elections/presidential_by_tract.csv` (already exists for 2020). For state legislative elections, see Election Data below.

---

## Metrics definitions

### Efficiency Gap (EG)

```
EG = (Wasted_R - Wasted_D) / Total_votes

Where wasted votes = losing votes + (winning votes - votes_to_win)
votes_to_win = total_district_votes / 2 + 1
```

Range: [-1, 1]. Values outside ±8% have been treated as presumptively unconstitutional in some courts (Gill v. Whitford threshold).

### Mean-Median Difference (MM)

```
MM = mean(district_dem_vote_share) - median(district_dem_vote_share)
```

Positive = Democratic advantage, Negative = Republican advantage.
Threshold: |MM| > 7% considered potentially problematic.

### Partisan Bias (PB)

Estimated by: fit a uniform swing to the district vote shares, find the swing S where Dem gets exactly 50% of votes statewide, then compute `0.5 - seat_share_at_S`.

### Bootstrap confidence intervals

All three metrics reported with 95% bootstrap CI (1000 resamples, resampling districts with replacement). This is standard practice in expert witness reports.

---

## Election data sources

### Presidential (available now)
`data/{year}/elections/presidential_by_tract.csv` — already present for 2020. For 2010/2000, may require separate download.

### State legislative elections (new)
State legislative election results by precinct → needs to be disaggregated to census tracts. This is a hard data problem — precincts don't align with tracts, and the data varies by state.

**Pragmatic approach for v1**: support presidential results only (available for all states). State legislative election support deferred to v2 via `--election-file <PATH>` flag that accepts any CSV in the standard format.

### `--election-file` flag

Allows practitioners to supply their own election data:

```bash
redist analyze --state WA --types partisan \
  --election-file data/custom/wa_2022_governor_by_tract.csv
```

Expected CSV format:
```
geoid,dem_votes,rep_votes
53033001001,1203.5,876.2
```

Fractional votes are allowed (precinct-to-tract interpolation produces them).

---

## Output: `analysis/partisan.json`

```json
{
  "analyzer": "partisan",
  "election": "2020_presidential",
  "election_file": "data/2020/elections/presidential_by_tract.csv",
  "available": true,
  "statewide": {
    "dem_vote_share": 0.583,
    "dem_seat_share": 0.600,
    "total_votes": 3284823
  },
  "metrics": {
    "efficiency_gap": {
      "value": 0.031,
      "direction": "Democratic",
      "ci_95_low": 0.011,
      "ci_95_high": 0.051,
      "threshold_8pct": "within_normal"
    },
    "mean_median": {
      "value": -0.023,
      "direction": "Republican",
      "ci_95_low": -0.041,
      "ci_95_high": -0.005,
      "threshold_7pct": "within_normal"
    },
    "partisan_bias": {
      "value": 0.018,
      "direction": "Democratic",
      "ci_95_low": -0.002,
      "ci_95_high": 0.038
    }
  },
  "districts": [
    {
      "district": 1,
      "dem_votes": 98234.5,
      "rep_votes": 76543.2,
      "dem_pct": 0.562,
      "margin": 0.124,
      "is_competitive": false,
      "is_uncontested": false
    }
  ],
  "note": "WA 2020 presidential election used as proxy. State legislative election not available."
}
```

---

## External tool interoperability: PlanScore

[PlanScore](https://planscore.org) is the leading public redistricting fairness tool. It accepts GeoJSON district plans via API and returns efficiency gap + other metrics.

**Integration**: `redist analyze --types partisan --also-submit-to planscore` option (future). For v1, we produce a GeoJSON export compatible with PlanScore's input format:

```bash
redist export --label wa_house_draft1 --format geojson \
  --out wa_house_draft1_planscore.geojson
# → Submit this file to planscore.org for independent verification
```

This is the same GeoJSON export from Spec 2's interoperability section.

---

## Implementation

### `redist-analysis/src/partisan.rs`

```rust
pub struct PartisanMetrics {
    pub efficiency_gap: MetricWithCI,
    pub mean_median: MetricWithCI,
    pub partisan_bias: MetricWithCI,
    pub statewide_dem_vote_share: f64,
    pub statewide_dem_seat_share: f64,
}

#[derive(Serialize)]
pub struct MetricWithCI {
    pub value: f64,
    pub direction: String,     // "Democratic" or "Republican"
    pub ci_95_low: f64,
    pub ci_95_high: f64,
    pub threshold: Option<ThresholdAssessment>,
}

pub fn compute_efficiency_gap(districts: &[DistrictElection]) -> f64

pub fn bootstrap_ci(
    districts: &[DistrictElection],
    metric_fn: impl Fn(&[DistrictElection]) -> f64,
    n_bootstrap: usize,
    rng_seed: u64,
) -> (f64, f64)
```

Bootstrap uses a deterministic seed derived from the plan seed for reproducibility.

---

## Tests

### L0

```rust
#[test]
fn test_efficiency_gap_zero_for_symmetric_plan() {
    // Perfect symmetry: each party wins 50% of districts by 60/40
    // → wasted votes equal → EG = 0
}

#[test]
fn test_efficiency_gap_direction() {
    // Rep wins 6 of 10 districts narrowly, Dem wins 4 blowouts
    // → EG should be Republican-favoring (negative value)
}

#[test]
fn test_mean_median_equal_when_symmetric() {
    // Uniform vote share across districts → mean == median → MM = 0
}

#[test]
fn test_bootstrap_ci_reproducible_with_seed() {
    // Same seed → same CI bounds
    let ci1 = bootstrap_ci(&districts, efficiency_gap, 1000, 42);
    let ci2 = bootstrap_ci(&districts, efficiency_gap, 1000, 42);
    assert_eq!(ci1, ci2);
}

#[test]
fn test_partisan_bias_neutral_at_50pct() {
    // Plan where party winning 50% of votes gets 50% of seats → bias = 0
}
```

### L2 acceptance

- `test_wa_partisan_produces_json` — WA plan with 2020 election → `partisan.json` with all 3 metrics
- `test_vt_single_district_partisan` — VT (1 district) → available=true, single district record
- `test_partisan_missing_election_returns_unavailable` — no election CSV → available=false, exit 0

---

## Alignment with other specs

- **Spec 1**: `--election-file` flag added to `AnalyzeArgs`
- **Spec 2**: `partisan` metrics in side-by-side plan comparison
- **Spec 3**: no direct dependency, but share the `AnalyzerContext`
- **Spec 6**: partisan fairness section of commission report includes EG + MM + PB with thresholds

---

## Board Review Amendments (2026-04-26)

**[SCALE] CRITICAL — Bootstrap CI invalid for small chambers**
Bootstrapping over districts with N < 10 produces meaningless variance estimates. Single-district states (VT, WY) and small chambers would silently report CIs that are statistically invalid.
Fix: Suppress CI computation when `num_districts < 10`. Report:
```json
{"ci_available": false, "ci_reason": "Bootstrap CI requires ≥10 districts (found N)"}
```
Do not report CI bounds at all for small chambers.

**[BOUNDARY] CONCERN — Gill threshold is not a constitutional standard**
The ±8% efficiency gap threshold (Stephanopoulos & McGhee) was explicitly not adopted by SCOTUS in Gill v. Whitford (2018). Labeling plans `within_normal` against this threshold misleads commissioners.
Fix: Replace `threshold_8pct: "within_normal"` with:
```json
{"academic_reference": "8% threshold from Stephanopoulos & McGhee (2015). SCOTUS declined to adopt in Gill v. Whitford (2018). Not a constitutional standard."}
```

**[PRECINCT] CONCERN — Presidential proxy limitation must be visible**
Using presidential results for state legislative chambers systematically inflates urban Democratic margins. This is a methodological limitation requiring disclosure in every report.
Fix: When `--election-file` is not specified and presidential data is used for a non-congressional chamber, include in `partisan.json`:
```json
{"methodology_warning": "Presidential election results used as proxy for state legislative district partisanship. Presidential coattail effects may systematically inflate urban Democratic margins. Provide state legislative election data via --election-file for more accurate analysis."}
```

**[SURVEY] CONCERN — Finding 6: Election data dependency must be explicit**
Partisan analysis has a hard dependency on a local data file that is NOT part of the RPLAN format. This dependency is invisible to tools that receive an RPLAN for analysis.
Fix: Add explicit note in this spec and in the partisan analyzer output: "Partisan analysis requires `data/{year}/elections/presidential_by_tract.csv`. Obtain via `redist fetch --type elections --year {year}`. RPLAN format does not carry election data; this dependency is local to the analysis machine."
If the election file is absent, `partisan.json` must include:
```json
{
  "available": false,
  "unavailable_reason": "Election data not found. Run: redist fetch --type elections --year 2020",
  "required_file": "data/2020/elections/presidential_by_tract.csv"
}
```
Do not silently produce metrics with missing data.

---

## R3 Board Review Amendments (2026-04-26)

**[SURVEY] CONCERN — Election data dependency not explicit**
Partisan analysis requires `data/{year}/elections/presidential_by_tract.csv` but this is never stated as a prerequisite. Fix: Add to the "Requires" section: "Requires `data/{year}/elections/presidential_by_tract.csv`. Obtain via `redist fetch --type elections --year {year}`. RPLAN format does not carry election data; this dependency is local to the analysis machine. When unavailable, `partisan.json` has `available: false` and `unavailable_reason: 'election data not found'`."
