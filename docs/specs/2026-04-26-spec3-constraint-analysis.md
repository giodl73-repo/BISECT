# Spec 3: Constraint Analysis

**Date**: 2026-04-26
**Status**: Draft
**Dependencies**: Spec 1 (Plan + PlanManifest)
**Depended on by**: Spec 2 (Plan Comparison), Spec 5 (Multi-chamber), Spec 6 (Reports)

---

## Problem

Redistricting plans must satisfy legal constraints beyond population balance:

1. **Contiguity** — every district must be a connected set of tracts (no islands)
2. **County preservation** — minimize county splits (WA constitution; most state constitutions)
3. **Municipal preservation** — minimize city/town splits
4. **Community of interest preservation** — tracks of demographic/geographic cohesion (harder to automate; addressed partially)

Today `redist analyze` checks none of these. A practitioner submitting a plan to a commission needs a machine-verifiable constraint report as part of the plan record.

---

## New analyzer types

Extends `redist analyze --types` with:
- `contiguity` — per-district connectivity check
- `splits` — county and municipal split counts and enumeration
- (future) `communities` — community of interest scoring

```bash
redist analyze --state WA --year 2020 --version WA_Plans \
  --label wa_house_draft1 --types contiguity splits compactness

# Or with all:
redist analyze --state WA --year 2020 --version WA_Plans \
  --label wa_house_draft1 --types all
```

---

## Contiguity Analysis

### Algorithm

For each district:
1. Build adjacency subgraph: tracts assigned to this district + edges between them
2. Run BFS/DFS from any node
3. If all tracts reached: contiguous ✓
4. If not all reached: identify disconnected components

"Contiguous" for coastal/island states: water-adjacent tracts in the same county are considered connected via a synthetic bridge edge (already built during adjacency construction in Phase 2).

### Output: `analysis/contiguity.json`

```json
{
  "analyzer": "contiguity",
  "all_contiguous": true,
  "districts": [
    {
      "district": 1,
      "contiguous": true,
      "tract_count": 14,
      "component_count": 1
    },
    {
      "district": 7,
      "contiguous": false,
      "tract_count": 19,
      "component_count": 2,
      "disconnected_tracts": ["53033005300", "53033005400"]
    }
  ]
}
```

`all_contiguous: false` causes `redist analyze` to exit with code 3 (separate from balance failure code 2) unless `--allow-noncontiguous` is passed.

### Implementation

```rust
// redist-analysis/src/contiguity.rs
pub fn check_contiguity(
    assignments: &HashMap<String, usize>,   // GEOID → district
    adjacency: &[Vec<usize>],              // tract adjacency graph
    geoids: &[String],                     // index → GEOID
    num_districts: usize,
) -> ContiguityResult

fn bfs_component_count(
    tract_indices: &HashSet<usize>,
    adjacency: &[Vec<usize>],
) -> (usize, Vec<HashSet<usize>>)  // (component_count, components)
```

---

## Split Analysis

### Data requirements

County and municipal boundaries need to be matched to census tracts. Two sources:

**County → tract**: Census tract GEOIDs encode county FIPS directly. Tract GEOID `530330001001` → state 53, county 033, tract 0001001. Parse directly from GEOID — no extra data file needed.

**Municipality → tract**: Requires Census Place-to-Tract relationship file (`data/{year}/geography/place_tract_{year}.csv`). Download via `redist fetch --type geography`. If absent, municipal splits are skipped with a warning.

### Metrics

**County splits**: A county is "split" if its tracts appear in more than one district.
- Count: how many counties are split
- Which: list of split counties with their district assignments
- Severity: for each split county, how many districts contain its tracts

**Municipal splits**: Same logic for Census-designated places.

**Preservation score**: `(total_counties - split_counties) / total_counties` — higher is better.

### Output: `analysis/splits.json`

```json
{
  "analyzer": "splits",
  "counties": {
    "total": 39,
    "split": 4,
    "preservation_score": 0.897,
    "split_list": [
      {
        "county_fips": "53033",
        "county_name": "King",
        "districts_containing": [1, 7, 9],
        "tract_count": 398,
        "split_severity": 3
      }
    ]
  },
  "municipalities": {
    "available": true,
    "total": 281,
    "split": 12,
    "preservation_score": 0.957,
    "split_list": [
      {
        "place_fips": "5363000",
        "place_name": "Seattle",
        "districts_containing": [7, 9],
        "split_severity": 2
      }
    ]
  }
}
```

### Implementation

```rust
// redist-analysis/src/splits.rs

/// Parse county FIPS from 11-char Census tract GEOID.
/// "530330001001" → "53033" (state 53, county 033)
fn county_fips_from_geoid(geoid: &str) -> &str { &geoid[..5] }

pub fn analyze_county_splits(
    assignments: &HashMap<String, usize>,  // GEOID → district
    county_names: Option<&HashMap<String, String>>, // fips → name (optional)
) -> CountySplitResult

pub fn analyze_municipal_splits(
    assignments: &HashMap<String, usize>,
    place_to_tracts: &HashMap<String, Vec<String>>,  // place_fips → GEOIDs
    place_names: &HashMap<String, String>,
) -> MunicipalSplitResult
```

County FIPS extraction from GEOID requires no data file — it's embedded in the GEOID format. This is zero-dependency.

---

## `redist fetch --type geography`

New fetch type for geographic relationship files:

```bash
redist fetch --type geography --year 2020 --states WA
# Downloads: data/2020/geography/wa_place_tract_2020.csv
```

Source: Census Bureau's Geographic Relationship Files (public, no auth).
URL pattern: `https://www2.census.gov/geo/docs/maps-data/data/rel2020/tract-place/tab20_tract20_{fips}_natl.txt`

---

## Tests

### L0

```rust
#[test]
fn test_county_fips_from_geoid() {
    assert_eq!(county_fips_from_geoid("530330001001"), "53033");
    assert_eq!(county_fips_from_geoid("010010020100"), "01001");
}

#[test]
fn test_contiguity_connected_graph() {
    // 4 tracts in a line: 0-1-2-3, all in district 1
    // BFS from 0 reaches all → contiguous
    let adj = vec![vec![1], vec![0,2], vec![1,3], vec![2]];
    let assignments = hashmap!{"t0"=>1,"t1"=>1,"t2"=>1,"t3"=>1};
    let result = check_contiguity(&assignments, &adj, &geoids, 1);
    assert!(result.districts[0].contiguous);
}

#[test]
fn test_contiguity_disconnected_district() {
    // Tracts 0,1 connected. Tract 3 isolated. All in district 1.
    // 0-1, 3 (no edge to 0 or 1) → not contiguous
    let adj = vec![vec![1], vec![0], vec![], vec![]];
    let assignments = hashmap!{"t0"=>1,"t1"=>1,"t2"=>2,"t3"=>1};
    let result = check_contiguity(&assignments, &adj, &geoids, 2);
    assert!(!result.districts[0].contiguous);
    assert_eq!(result.districts[0].component_count, 2);
}

#[test]
fn test_county_split_single_district() {
    // All King County tracts in district 1 → no split
    let assignments = hashmap!{"53033001"=>1, "53033002"=>1};
    let result = analyze_county_splits(&assignments, None);
    assert_eq!(result.split, 0);
}

#[test]
fn test_county_split_across_two_districts() {
    // King County split across districts 1 and 2
    let assignments = hashmap!{"53033001"=>1, "53033002"=>2};
    let result = analyze_county_splits(&assignments, None);
    assert_eq!(result.split, 1);
    assert_eq!(result.split_list[0].districts_containing.len(), 2);
}
```

### L2 acceptance

- `test_wa_contiguity_all_pass` — WA 10-district plan → `all_contiguous: true`
- `test_wa_county_splits_count` — WA generated plan → split count ≤ enacted split count
- `test_splits_json_structure` — all required fields present
- `test_contiguity_exits_3_on_violation` — synthesize disconnected plan → exit code 3

---

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | All constraints satisfied |
| 2 | Population balance violation (Spec 1) |
| 3 | Contiguity violation |
| 4 | Both violations |

`--allow-noncontiguous` suppresses exit code 3 (research/draft use).

---

## Alignment with other specs

- **Spec 1**: reads `manifest.json` for `balance_tolerance`, `chamber` context
- **Spec 2**: `splits.json` → `county_splits` and `municipal_splits` in comparison output
- **Spec 5**: contiguity check mandatory for nested plan validation
- **Spec 6**: `contiguity.json` + `splits.json` → constraint compliance section of report

---

## Board Review Amendments (2026-04-26)

**[WARD] CRITICAL — Split standard must be state-specific**
"Minimize splits" and a generic preservation score are not legally sufficient. WA's constitutional language differs from CA's Article XXI, TX, CO, etc. The same split count may be compliant in one state and a constitutional violation in another.
Fix: Add `--state-splits-standard <STATE>` lookup (or auto-detect from `--state` flag). Per-state table records the applicable constitutional language and threshold. Output includes:
```json
{
  "legal_standard": "WA Const. Art. II §43 — counties shall be preserved where possible",
  "compliance_assessment": "4 splits present; no binding numerical limit under WMCA",
  "disclaimer": "Legal compliance determination requires counsel"
}
```

**[LEDGER] CONCERN — Missing municipal data must be an error for required jurisdictions**
Silently skipping municipal splits when the geography file is absent could constitute a legal deficiency if municipal preservation is constitutionally required in that state.
Fix: `redist analyze --types splits` checks if the state has a municipal preservation requirement (lookup table). If yes and the geography file is absent, exit with error code 6 and message: `ERROR: Municipal preservation is constitutionally required for WA but place-tract data is absent. Run: redist fetch --type geography --states WA`. Add `--require-municipal` flag to force this behavior in any state.

**[BENCHMARK] CONCERN — Finding 5: Adopt bitfield exit codes**
The current exit code table (0, 2, 3, 4) has gaps and does not compose when multiple violations occur simultaneously. Exit code 4 means "both violations" but this cannot scale to additional constraint types.
Fix: Adopt bitfield exit codes. Each constraint type owns one bit:
- 0 = success (no violations)
- 1 = population balance violation (bit 0)
- 2 = contiguity violation (bit 1)
- 4 = nesting violation (bit 2)
- 8 = missing required data (bit 3)

Combinations OR the bits: balance+contiguity=3, balance+nesting=5, contiguity+nesting=6, all three=7, etc. Remove the old codes 2, 3, 4, 5, 6 and replace with this table. `redist analyze` returns the OR of all active violation bits across all constraint types checked in a single invocation.

Updated exit code table:
| Code | Meaning |
|------|---------|
| 0 | All constraints satisfied |
| 1 | Population balance violation |
| 2 | Contiguity violation |
| 3 | Balance + contiguity violation |
| 4 | Nesting violation |
| 5 | Balance + nesting violation |
| 6 | Contiguity + nesting violation |
| 7 | Balance + contiguity + nesting violation |
| 8 | Missing required data (geography file absent for required jurisdiction) |

`--allow-noncontiguous` suppresses bit 1 (contiguity) from the exit code.
