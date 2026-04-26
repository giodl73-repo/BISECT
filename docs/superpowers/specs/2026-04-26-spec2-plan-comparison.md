# Spec 2: Plan Comparison

**Date**: 2026-04-26
**Status**: Draft
**Dependencies**: Spec 1 (Plan + PlanManifest data model)
**Depended on by**: Spec 6 (Reports)

---

## Problem

A redistricting commission needs to compare:
1. **Their generated plan** vs the **currently enacted districts** (legal baseline)
2. **Plan A vs Plan B** (two runs of the algorithm with different seeds/parameters)
3. **This year's plan vs the prior census plan** (2020 vs 2010 districts)

Today `redist` has no comparison capability — you can generate plans but not evaluate them against alternatives. Practitioners need a `redist compare` subcommand that produces side-by-side metrics.

---

## New subcommand: `redist compare`

```
redist compare --plan-a <LABEL_OR_PATH> --plan-b <LABEL_OR_PATH> [OPTIONS]
```

One of `--plan-b` or `--enacted` is required.

| Flag | Description |
|------|-------------|
| `--plan-a <LABEL>` | First plan (required) — label or path to plan directory |
| `--plan-b <LABEL>` | Second plan — label or path |
| `--enacted` | Use currently enacted districts as plan B (downloads if absent) |
| `--year <YEAR>` | Census year for both plans (default: 2020) |
| `--version <V>` | Version directory to look up plan labels |
| `--metrics <LIST>` | Metrics to compare: `population`, `compactness`, `splits`, `partisan`, `all` (default: all) |
| `--out <PATH>` | Write comparison JSON/CSV (default: stdout summary) |
| `--format` | `table` (default), `json`, `csv` |

### Examples

```bash
# Compare two generated plans
redist compare \
  --plan-a wa_house_draft1 --plan-b wa_house_draft2 \
  --year 2020 --version WA_Plans

# Compare generated plan vs enacted
redist compare \
  --plan-a wa_house_draft1 --enacted \
  --year 2020 --version WA_Plans --metrics compactness splits

# Compare 2020 plan vs 2010 plan (cross-year)
redist compare \
  --plan-a wa_house_2020 --plan-b wa_house_2010 \
  --metrics population compactness
```

---

## Enacted district download: `redist fetch --type enacted`

Extends `redist fetch` to download currently enacted district shapefiles from the Census Bureau's TIGER/Line congressional district files.

```bash
redist fetch --type enacted --year 2020 --states WA
# Downloads: data/2020/enacted/{state}_congressional_2020.shp (or legislative)
# Also: data/2020/enacted/{state}_house_2020.shp
#        data/2020/enacted/{state}_senate_2020.shp (where available)
```

**Source**: Census TIGER/Line Shapefiles — Congressional Districts, State Legislative Districts (Upper/Lower). Public domain, no auth required.

**FIPS-to-URL mapping**: added to `manifest.json` alongside TIGER tract URLs.

### Tract assignment for enacted plans

Enacted shapefiles are polygon boundaries, not tract assignments. Converting them:
1. Load tract centroids (from TIGER tract shapefile)
2. Spatial point-in-polygon: assign each tract centroid to the enacted district it falls in
3. Write as `enacted_assignments.json` in same format as `final_assignments.json`
4. This is a one-time computation cached as `data/{year}/enacted/{state}_tract_assignments.json`

The point-in-polygon step requires `geo` crate's `Contains` algorithm — already a dependency.

---

## Comparison metrics

### 1. Population distribution
For each plan: ideal_pop, min district pop, max district pop, max deviation %, Gini coefficient of district populations.

```
Metric               Plan A (Generated)    Plan B (Enacted)
Ideal population     761,169               761,169
Max deviation        0.23%                 0.87%
Min district pop     759,421               754,832
Max district pop     762,773               767,904
Gini coefficient     0.0012                0.0031
```

### 2. Compactness comparison
Mean PP, mean Reock, min PP district, max PP district for each plan.

### 3. Geographic overlap (Jaccard similarity)
For each district in Plan A, find the best-matching district in Plan B by tract overlap. Report mean Jaccard similarity (1.0 = identical, 0.0 = no shared tracts).

```
Jaccard similarity: 0.73   (plans share 73% of tract assignments)
Significantly changed districts: 3 of 10
```

### 4. County/municipal splits (from Spec 3)
Number of split counties, number of split municipalities per plan.

### 5. Partisan metrics (from Spec 4)
If election data available: efficiency gap, mean-median difference, partisan bias — both plans side by side.

---

## Output format

### Table (default)
```
==================================================
Plan Comparison: wa_house_draft1 vs enacted
State: WA  Year: 2020  Chamber: house
==================================================

POPULATION EQUALITY
  Draft 1     Max deviation: 0.23%   PASS (tolerance: 5.0%)
  Enacted     Max deviation: 0.87%   PASS (tolerance: 5.0%)
  Winner: Draft 1 (lower deviation)

COMPACTNESS (Polsby-Popper)
  Draft 1     Mean PP: 0.387
  Enacted     Mean PP: 0.312
  Winner: Draft 1 (+24% more compact)

GEOGRAPHIC STABILITY
  Jaccard similarity: 0.73
  Districts substantially changed: 3 of 10

COUNTY SPLITS
  Draft 1     Splits: 4
  Enacted     Splits: 7
  Winner: Draft 1 (3 fewer county splits)
==================================================
```

### JSON output
```json
{
  "plan_a": {"label": "wa_house_draft1", "manifest": {...}},
  "plan_b": {"label": "enacted", "source": "census_tiger"},
  "metrics": {
    "population": {"plan_a_max_dev": 0.0023, "plan_b_max_dev": 0.0087},
    "compactness": {"plan_a_mean_pp": 0.387, "plan_b_mean_pp": 0.312},
    "jaccard_similarity": 0.73,
    "county_splits": {"plan_a": 4, "plan_b": 7}
  }
}
```

---

## Implementation

### New crate: `redist-compare` (in `redist-analysis`)

Module `redist-analysis/src/comparison.rs`:

```rust
pub struct PlanComparison {
    pub plan_a: PlanSummary,
    pub plan_b: PlanSummary,
    pub jaccard_similarity: f64,         // mean over best-matched districts
    pub population: PopulationComparison,
    pub compactness: CompactnessComparison,
    pub county_splits: Option<SplitComparison>,
    pub partisan: Option<PartisanComparison>,
}

pub fn compare_plans(
    assignments_a: &HashMap<String, usize>,
    assignments_b: &HashMap<String, usize>,
) -> PlanComparison
```

Jaccard similarity per district:
```rust
fn jaccard(set_a: &HashSet<String>, set_b: &HashSet<String>) -> f64 {
    let intersection = set_a.intersection(set_b).count() as f64;
    let union = set_a.union(set_b).count() as f64;
    if union == 0.0 { 1.0 } else { intersection / union }
}
```

### Enacted assignment conversion

`redist-data/src/enacted.rs`:
```rust
pub fn assign_tracts_to_enacted(
    tract_centroids: &[geo_types::Point<f64>],  // from TIGER
    tract_geoids: &[String],
    enacted_polygons: &[geo_types::MultiPolygon<f64>],
    enacted_ids: &[usize],
) -> HashMap<String, usize>
```

Uses `geo::algorithm::Contains` for point-in-polygon.

---

## Tests

### L0
- `test_jaccard_identical_plans` — same assignments → 1.0
- `test_jaccard_completely_different` — no shared tracts → 0.0
- `test_jaccard_partial_overlap` — known fixture → expected value
- `test_population_comparison_winner` — lower max deviation wins
- `test_tract_centroid_in_polygon` — known point inside polygon → correct district

### L2 acceptance
- `test_compare_two_generated_plans` — two VT runs with different seeds → Jaccard < 1.0 (different assignments), all metrics present
- `test_compare_plan_vs_self` — same plan vs itself → Jaccard 1.0, all metrics equal
- `test_enacted_download_and_assign` — fetch WA enacted, assign tracts, verify 10 districts

---

## Alignment with other specs

- **Spec 1**: reads `PlanManifest` for metadata on both plans
- **Spec 3**: county/split metrics fed into comparison output
- **Spec 4**: partisan metrics fed into comparison output
- **Spec 6**: `PlanComparison` JSON is a primary input to the commission report
