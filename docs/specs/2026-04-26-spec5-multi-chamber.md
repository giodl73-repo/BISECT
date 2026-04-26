# Spec 5: Multi-Chamber and Nested Plans

**Date**: 2026-04-26
**Status**: Draft
**Dependencies**: Spec 1 (Custom Parameters), Spec 3 (Constraint Analysis — contiguity)
**Depended on by**: Spec 6 (Reports)

---

## Problem

State legislatures often require **nesting**: senate districts must be composed of whole house districts. In Washington:
- 49 senate districts, each containing exactly 2 house districts
- No house district may be split across senate districts

This "whole number nesting" constraint is a formal constitutional requirement in Washington and ~20 other states. The current CLI treats each chamber run independently — there's no mechanism to enforce or validate that one plan nests inside another.

Additionally, practitioners need to manage multiple chambers as a **suite** — drawing all three chambers for a state and tracking them as a unit.

---

## New concept: `PlanSuite`

A suite is a named collection of plans for different chambers of the same state/year:

```
outputs/{version}/{year}/suites/{suite_name}/
  suite.json          ← suite manifest
  congressional/      ← symlink or copy of plan labeled {suite}_congressional
  house/              ← plan labeled {suite}_house
  senate/             ← plan labeled {suite}_senate
```

### `redist suite` subcommand

```bash
# Create a new suite (draws all three chambers)
redist suite --state WA --year 2020 --version WA_Plans \
  --name wa_commission_v1 \
  --congressional-districts 10 \
  --house-districts 98 \
  --senate-districts 49 \
  --nest senate-in-house \
  --seed 42

# Validate an existing suite
redist suite validate --name wa_commission_v1 --version WA_Plans --year 2020

# Add a single chamber to an existing suite
redist suite add-plan --name wa_commission_v1 \
  --label wa_house_alt1 --chamber house
```

---

## Nesting modes

| Mode | Description |
|------|-------------|
| `senate-in-house` | Each senate district = exactly 2 (or N) whole house districts |
| `house-in-senate` | Each house district ⊆ one senate district (weaker constraint) |
| `none` | No nesting constraint (default) |
| `custom` | User-specified nesting via `--nest-ratio N:M` |

### Washington example: `senate-in-house`

WA has 49 senate districts, each composed of exactly 2 house districts (98/49 = 2). The nesting algorithm:

1. Draw house plan first (98 districts)
2. Draw senate plan using house districts as atomic units (not census tracts)
   - Build a new adjacency graph where nodes = house districts, edges = shared boundaries
   - Run METIS on this house-district graph with k=49
   - Each "super-node" in the senate plan = 2 house districts

This guarantees perfect nesting by construction.

### Generic nesting: house-district adjacency graph

```rust
// Build adjacency between house districts (for senate nesting)
pub fn build_chamber_adjacency(
    house_assignments: &HashMap<String, usize>,   // tract → house_district
    tract_adjacency: &[Vec<usize>],               // original tract adjacency
    num_house_districts: usize,
) -> Vec<Vec<usize>>  // house_district adjacency (node = house district)
```

Two house districts are adjacent if any of their constituent tracts share a boundary.

---

## Suite validation

`redist suite validate` runs all constraint checks and produces a suite-level report:

```json
{
  "suite_name": "wa_commission_v1",
  "state": "WA",
  "year": "2020",
  "chambers": {
    "congressional": {"label": "wa_congressional", "districts": 10, "balance_ok": true},
    "house": {"label": "wa_house_draft1", "districts": 98, "balance_ok": true},
    "senate": {"label": "wa_senate_draft1", "districts": 49, "balance_ok": true}
  },
  "nesting": {
    "mode": "senate-in-house",
    "valid": true,
    "violations": [],
    "senate_to_house_map": {
      "1": [1, 2],
      "2": [3, 4],
      ...
    }
  },
  "constraint_summary": {
    "all_contiguous": true,
    "total_county_splits": 7,
    "total_municipal_splits": 14
  }
}
```

### Nesting validation algorithm

For `senate-in-house`:
1. For each senate district S, collect all tracts in S
2. Look up which house districts those tracts belong to
3. Assert: all tracts in senate district S belong to exactly N house districts
4. Assert: no house district's tracts appear in more than one senate district

```rust
pub fn validate_nesting(
    house_assignments: &HashMap<String, usize>,   // tract → house_district
    senate_assignments: &HashMap<String, usize>,  // tract → senate_district
    required_ratio: usize,  // 2 for WA (each senate = 2 house)
) -> NestingValidation {
    // Returns: valid bool + list of violations
}
```

---

## `redist suite draw` workflow

When `--nest senate-in-house` is specified, the suite command runs chambers in sequence:

1. Draw congressional (independent, any order)
2. Draw house (independent)
3. **Build house-district adjacency graph** from house plan
4. Draw senate using house-district graph as input (not tract graph)
5. Validate nesting
6. Write suite manifest

If nesting validation fails, exit with code 5 and print violated senate districts.

---

## Tests

### L0

```rust
#[test]
fn test_build_chamber_adjacency_simple() {
    // 4 tracts: 0,1 in house district 1; 2,3 in house district 2
    // Tracts 1 and 2 are adjacent
    // → house districts 1 and 2 should be adjacent
    let house_asgn = hashmap!{"t0"=>1,"t1"=>1,"t2"=>2,"t3"=>2};
    let tract_adj = vec![vec![1],vec![0,2],vec![1,3],vec![2]];
    let house_adj = build_chamber_adjacency(&house_asgn, &tract_adj, 2);
    assert!(house_adj[0].contains(&1)); // house dist 0 adj to house dist 1
}

#[test]
fn test_nesting_validation_perfect() {
    // Senate 1 = house [1,2], Senate 2 = house [3,4] → valid
    let house = hashmap!{"t0"=>1,"t1"=>1,"t2"=>2,"t3"=>2,"t4"=>3,"t5"=>3,"t6"=>4,"t7"=>4};
    let senate = hashmap!{"t0"=>1,"t1"=>1,"t2"=>1,"t3"=>1,"t4"=>2,"t5"=>2,"t6"=>2,"t7"=>2};
    let result = validate_nesting(&house, &senate, 2);
    assert!(result.valid);
    assert!(result.violations.is_empty());
}

#[test]
fn test_nesting_validation_violation() {
    // Senate 1 contains tracts from house 1, 2, AND 3 → violation
    let house = hashmap!{"t0"=>1,"t1"=>2,"t2"=>3};
    let senate = hashmap!{"t0"=>1,"t1"=>1,"t2"=>1};
    let result = validate_nesting(&house, &senate, 2);
    assert!(!result.valid);
    assert!(!result.violations.is_empty());
}

#[test]
fn test_nesting_ratio_computed() {
    // 98 house / 49 senate = ratio 2
    assert_eq!(compute_nest_ratio(98, 49), Some(2));
    assert_eq!(compute_nest_ratio(98, 48), None);  // doesn't divide evenly
}
```

### L2 acceptance

- `test_wa_suite_draws_all_three_chambers` — suite command produces 3 plan directories
- `test_wa_senate_nests_in_house` — `validate_nesting` returns `valid: true` for generated WA suite
- `test_suite_manifest_records_nesting_mode`
- `test_suite_validate_exit_5_on_nesting_violation` — synthesized violation → exit code 5

---

## Exit codes (adds to Spec 3)

| Code | Meaning |
|------|---------|
| 5 | Nesting violation |

---

## Alignment with other specs

- **Spec 1**: suite uses `--label` and `PlanManifest` for each chamber plan
- **Spec 3**: contiguity and splits run on each chamber plan; nesting adds a fourth constraint type
- **Spec 6**: report covers all chambers in a suite; nesting compliance section included

---

## Board Review Amendments (2026-04-26)

**[MERIDIAN] CRITICAL — Noncontiguous house plan silently breaks senate nesting**
If `--allow-noncontiguous` is used to proceed with a noncontiguous house plan, the house-district adjacency graph used for senate nesting will be disconnected. METIS will produce an invalid senate plan with no error.
Fix: `redist suite --nest` hard-fails if any house district fails contiguity check, regardless of `--allow-noncontiguous`. Error message: `ERROR: Senate nesting requires all house districts to be contiguous. District 7 has 2 components. Remove --allow-noncontiguous or fix the house plan before nesting.`

**[WARD] CONCERN — Variable nesting ratios need constitutional validation**
Several states have variable or non-integer nesting schemes. The `--nest-ratio N:M` flag exists but there is no validation against known constitutional requirements.
Fix: Add a per-state nesting table. When `--nest-ratio` deviates from the constitutional value, print a warning: `WARNING: WA constitution requires 2:1 house-to-senate nesting ratio. You specified 3:1. Proceed only if you have verified this is legally permissible.`

**[LEDGER] CONCERN — Finding 7: Suite export produces separate RPLAN files, not a multi-plan RPLAN**
The spec was ambiguous about whether a suite export produces one multi-plan RPLAN file or multiple single-plan files. Multi-plan RPLAN is deferred to RPLAN v0.2.
Fix: Suite export produces three separate RPLAN files plus a `suite.json` envelope. Structure:
```
exports/wa_commission_v1/
  suite.json               <- suite manifest referencing each plan
  wa_congressional.rplan
  wa_house.rplan
  wa_senate.rplan
```
`suite.json` schema:
```json
{
  "suite_name": "wa_commission_v1",
  "plans": [
    {"chamber": "congressional", "file": "wa_congressional.rplan"},
    {"chamber": "house", "file": "wa_house.rplan"},
    {"chamber": "senate", "file": "wa_senate.rplan"}
  ]
}
```
Each `.rplan` file is a standalone valid RPLAN v0.1 document. Multi-plan RPLAN envelope waits for RPLAN v0.2.

**[MERIDIAN] CONCERN — Finding 9: build_chamber_adjacency must handle disconnected house districts**
If a house district has disconnected components (e.g., due to `--allow-noncontiguous`), `build_chamber_adjacency` currently includes all tracts from all components, which can create phantom adjacencies between geographically separated regions.
Fix: `build_chamber_adjacency` uses only the PRIMARY component of each house district (largest by tract count). Tracts in disconnected secondary components are excluded from adjacency computation with a warning:
```
WARNING: House district N has disconnected components; only primary component used for senate adjacency.
```
This ensures that senate nesting is computed from the geographically coherent portion of each house district. The full fix is: `redist suite --nest` hard-fails on noncontiguous house districts (per the MERIDIAN CRITICAL amendment above), making this defensive path a fallback for research use only.
