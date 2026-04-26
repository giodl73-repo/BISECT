# Spec 1: Custom Redistricting Parameters

**Date**: 2026-04-26
**Status**: Draft
**Dependencies**: None (foundation spec)
**Depended on by**: Specs 2, 3, 4, 5, 6

---

## Problem

The `redist` CLI currently draws congressional districts using hardcoded counts from `config_{year}.py`. A Washington state redistricting commission needs to draw:
- 98 state house districts
- 49 state senate districts
- 10 congressional districts (for comparison/audit)

All from the same 2020 census tract data. They also need reproducible runs with a signed audit trail, and control over population base (total vs VAP vs CVAP).

---

## Shared Data Model (used by all 6 specs)

### `Plan`

A plan is the atomic unit of redistricting work. Every subcommand produces or consumes plans.

```
outputs/{version}/{year}/plans/{label}/
  manifest.json          ← PlanManifest (provenance, signed)
  final_assignments.json ← tract → district map
  analysis/              ← outputs of Specs 3, 4
  maps/                  ← outputs of redist map
  intermediate/          ← bisection rounds
```

`--label` is the human name for the plan (e.g., `wa_house_draft1`, `wa_senate_v3`). If omitted, defaults to `{state_code}_{chamber}_{year}`.

### `PlanManifest`

Written alongside every plan. Provides full audit chain of custody.

```json
{
  "label": "wa_house_draft1",
  "state_code": "WA",
  "year": "2020",
  "chamber": "house",
  "num_districts": 98,
  "population_source": "total",
  "partition_mode": "edge-weighted",
  "seed": 12345,
  "binary_version": "0.1.0",
  "binary_sha256": "abc123...",
  "adjacency_file": "outputs/V3/data/2020/adjacency/wa_adjacency_2020.adj.bin",
  "adjacency_sha256": "def456...",
  "tiger_file": "data/2020/tiger/tracts/53/tl_2020_53_tract.shp",
  "tiger_sha256": "ghi789...",
  "created_at": "2026-04-26T14:23:00Z",
  "balance_tolerance_pct": 0.5,
  "population_balance_valid": true
}
```

---

## New CLI Flags

### `redist state` / `redist states` / `redist run`

| Flag | Default | Description |
|------|---------|-------------|
| `--districts N` | *(from config)* | Override district count (enables non-congressional chambers) |
| `--chamber <NAME>` | `congressional` | Label for this chamber: `congressional`, `house`, `senate`, `custom` |
| `--label <NAME>` | auto-generated | Human name for this plan run |
| `--population-source` | `total` | `total` (default), `vap` (voting-age), `cvap` (citizen VAP) |
| `--balance-tolerance` | `0.5` | Max % deviation per district (0.5 = ±0.5%, 5.0 = ±5%) |
| `--manifest` | true | Write `manifest.json` alongside outputs (default: true) |

### Population source behavior

| Source | Description | Data file |
|--------|-------------|-----------|
| `total` | Total census population (default, congressional standard) | `vertex_weights` from adjacency graph |
| `vap` | Voting-age population (18+) | `data/{year}/demographics/{state}_demographics_{year}.csv` col `vap` |
| `cvap` | Citizen voting-age population (state legislature standard in some states) | `data/{year}/demographics/{state}_demographics_{year}.csv` col `cvap` |

If `vap` or `cvap` is requested but the data column is absent, `redist` exits with a clear error: `ERROR: --population-source vap requires 'vap' column in demographics CSV. Column not found.`

### Examples

```bash
# Washington house — 98 districts
redist state --state WA --year 2020 --version WA_Plans \
  --districts 98 --chamber house --label wa_house_draft1 \
  --balance-tolerance 5.0 --seed 42

# Washington senate — 49 districts
redist state --state WA --year 2020 --version WA_Plans \
  --districts 49 --chamber senate --label wa_senate_draft1 \
  --balance-tolerance 5.0 --seed 42

# Congressional (for audit comparison, using official config count)
redist state --state WA --year 2020 --version WA_Plans \
  --chamber congressional --label wa_congress_audit

# All 50 states, custom balance tolerance
redist states --year 2020 --version WA_Plans \
  --output-dir outputs/WA_Plans --balance-tolerance 2.0

# Using citizen VAP instead of total population
redist state --state WA --year 2020 --version WA_CVAP \
  --districts 98 --chamber house --population-source cvap
```

---

## Implementation

### `StateConfig` additions

```rust
pub struct StateConfig {
    // ... existing fields ...
    pub num_districts_override: Option<usize>,  // None = use config
    pub chamber: String,                         // "congressional", "house", "senate", "custom"
    pub label: Option<String>,                   // plan label
    pub population_source: PopulationSource,
    pub balance_tolerance: f64,                  // fraction (0.005 = ±0.5%)
    pub write_manifest: bool,
}

#[derive(Debug, Clone)]
pub enum PopulationSource {
    Total,   // default — from adjacency vertex_weights
    Vap,     // from demographics CSV
    Cvap,    // from demographics CSV
}
```

### Path resolution with label

```
outputs/{version}/{year}/plans/{label}/data/final_assignments.json
outputs/{version}/{year}/plans/{label}/manifest.json
outputs/{version}/{year}/plans/{label}/analysis/
outputs/{version}/{year}/plans/{label}/maps/
```

If no `--label`, default: `{state_lower}_{chamber}_{year}`.

### Manifest writing

After `write_state_outputs()` succeeds, compute SHA-256 of adjacency and TIGER files (cached — don't re-read large files), write `manifest.json` atomically alongside outputs.

### Balance tolerance

`assert_balanced()` in `redist-core` currently uses hardcoded 0.005. Change to accept `tolerance: f64` parameter. CLI passes `cfg.balance_tolerance`.

---

## Tests

### L0
- `test_population_source_vap_loads_from_demographics_csv` — mock CSV with vap column, verify vertex weights overridden
- `test_balance_tolerance_5pct_passes_where_halfpct_fails` — district with 3% deviation: passes at 5.0, fails at 0.5
- `test_manifest_sha256_is_deterministic` — same file produces same hash
- `test_label_default_generation` — no label + WA house 2020 → `washington_house_2020`

### L2 acceptance
- `test_wa_house_98_districts_produces_manifest` — `redist state --state WA --districts 98 --chamber house` → manifest.json with `num_districts: 98`, `chamber: "house"`
- `test_seed_produces_reproducible_assignments` — same seed twice → identical `final_assignments.json`
- `test_balance_tolerance_in_manifest` — manifest records the tolerance used
- `test_population_source_in_manifest` — manifest records population_source

---

## Alignment with other specs

- **Spec 2 (Plan Comparison)**: compares two plans by label — needs `label` from this spec
- **Spec 3 (Constraint Analysis)**: reads `manifest.json` to know `balance_tolerance` and `chamber`
- **Spec 4 (Partisan Metrics)**: reads `final_assignments.json` from plan directory
- **Spec 5 (Multi-chamber)**: references parent plan label for nesting validation
- **Spec 6 (Reports)**: reads `manifest.json` as audit chain of custody anchor

---

## Board Review Amendments (2026-04-26)

**[WARD] CRITICAL — Chamber-aware balance tolerance defaults**
The `--balance-tolerance` default of 0.5% incorrectly applies a federal congressional standard to state legislative chambers. WA state legislative districts have unlimited deviation under WMCA case law; most states allow ±5-10% for state chambers.
Fix: Defaults are now chamber-aware:
- `congressional`: 0.5% (Baker v. Carr strict)
- `house`, `senate`: 5.0% with a printed notice: "State legislative balance tolerance set to 5.0%. Verify your state's constitutional standard."
- `--balance-tolerance` override always wins.

**[COVENANT] CRITICAL — Manifest must hash Census source, not derived files**
Hashing `adjacency.adj.bin` (a derived product) does not allow a special master to verify correctness back to Census-published data. The manifest must separately record the SHA-256 of the upstream TIGER tract shapefile and the adjacency-build command/version.
Fix: `PlanManifest` gains two additional fields:
```json
{
  "tiger_sha256": "hash of data/2020/tiger/tracts/{fips}/tl_2020_{fips}_tract.shp",
  "adjacency_build_command": "python scripts/data/geography/build_tract_adjacency.py ...",
  "adjacency_build_version": "redist_py 0.1.0 (abi3)"
}
```
