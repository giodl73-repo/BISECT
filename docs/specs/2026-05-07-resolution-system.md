# Spec: Resolution System — Geographic Granularity as a First-Class Parameter

**Status**: Accepted (R1 avg 2.8/4 — P1 fixes applied; ready for Phase 1 implementation)  
**Reviewed R1**: MERIDIAN 3/4, BENCHMARK 3/4, SURVEY 3/4, COVENANT 2/4 → avg 2.8/4  
**Date**: 2026-05-07  
**Scope**: Cross-cutting — affects data loading, plan generation, analysis, reporting, CLI  
**Enables**: Multi-scale MCMC (G.11), BG-level redistricting, county-level coarsening  
**Related specs**: 2026-05-07-multiscale-mcmc.md

---

## 1. Motivation

The current pipeline operates exclusively at the census tract level. Resolution is partially exposed (`--resolution block_group`) but only for adjacency loading — it does not flow through plan generation, analysis, or reporting. This creates three problems:

1. **Multi-scale MCMC requires two resolutions simultaneously** — fine (block group) and coarse (tract or county) — but the pipeline has no concept of a resolution hierarchy.
2. **High-precision redistricting** (state legislative chambers with many small districts) would benefit from block-group resolution, but the pipeline cannot output BG-level plans.
3. **County-level coarsening** is useful for large-k states (TX k=38) but requires no extra data — just a GEOID-derived partition — yet has no clean integration path.

### Resolution levels

| Level | Abbrev | Approx size | GEOID length | GEOID structure |
|-------|--------|-------------|--------------|-----------------|
| Block group | `bg` | ~1,000 pop | 12 chars | state(2)+county(3)+tract(6)+bg(1) |
| Tract | `tract` | ~4,000 pop | 11 chars | state(2)+county(3)+tract(6) |
| County | `county` | ~100,000 pop | 5 chars | state(2)+county(3) |

A block group GEOID is a prefix extension of a tract GEOID which is a prefix extension of a county GEOID. The entire fine→coarse mapping hierarchy is derivable from GEOIDs alone — no extra mapping files required.

---

## 2. Resolution threading design

Resolution must flow consistently through five pipeline stages:

```
[data loading] → [plan generation] → [analysis] → [reporting] → [label-verify]
     ↑                   ↑                ↑              ↑              ↑
  --resolution       --resolution     auto-detected  auto-detected  manifest
```

### 2.1 Plan resolution declaration

Every plan manifest (`index.json`) gains a `plan_resolution` field:
```json
"plan_resolution": "tract",    // "bg" | "tract" | "county"
"n_units": 2195,               // number of geographic units at this resolution
"unit_type": "census tract"    // human-readable label for reports
```

This field is written at plan generation time and read by analysis/reporting to set context.

### 2.2 Data loading — three adjacency sources

| Resolution | File pattern | Derived from |
|-----------|-------------|-------------|
| `bg` | `{state}_bg_adjacency_{year}.pkl` | Census block groups (already fetched) |
| `tract` | `{state}_adjacency_{year}.pkl` | Census tracts (current default) |
| `county` | derived in-memory | Tract GEOID prefix (no file needed) |

County-level adjacency is built from the tract graph on demand:
```
county_partition[tract_i] = index of county(geoid_i[:5]) in sorted unique counties
```
No extra fetch step required for county-level coarsening.

### 2.3 CLI — `--resolution` applies everywhere

Currently `--resolution block_group` only affects adjacency loading. After this spec, it applies to the entire run:

```bash
# Tract-level (current default — unchanged)
redist state --state NC --year 2020

# Block-group level (finer spatial resolution)
redist state --state NC --year 2020 --resolution block_group

# Multi-scale: fine=BG, coarse=tract (Option A)
redist state --state NC --year 2020 --search multiscale --multiscale-fine bg --multiscale-coarse tract

# Multi-scale: fine=tract, coarse=county (Option B — no extra data)
redist state --state NC --year 2020 --search multiscale --multiscale-fine tract --multiscale-coarse county
```

### 2.4 Multi-scale resolution options

The `--search multiscale` mode accepts `--multiscale-fine` and `--multiscale-coarse` flags that select any two adjacent levels from the hierarchy:

| Option | `--multiscale-fine` | `--multiscale-coarse` | Extra data needed? | Output plan level |
|--------|--------------------|-----------------------|-------------------|-----------------|
| A | `bg` | `tract` | BG file (already fetched) | BG |
| B | `tract` | `county` | None (derived from GEOIDs) | Tract |
| C | `bg` | `county` | BG file | BG |

All three options produce a plan at the **fine level**. The plan resolution field in the manifest records which level was used.

**Validation**: `--multiscale-fine` must be strictly finer (smaller geographic unit) than `--multiscale-coarse`. Invalid orderings (e.g., `--multiscale-fine county --multiscale-coarse bg`) return an error:
```
ERROR: --multiscale-fine county is not finer than --multiscale-coarse bg.
Valid orderings (fine → coarse): bg→tract, bg→county, tract→county.
```

### 2.5 Output plan format — resolution-aware

The plan output (`assignments.json` or RPLAN) records assignments at the fine resolution level. For BG-level plans, there are more rows (~5,000 for NC vs ~2,200 at tract level) but the format is identical. Downstream analysis tools read `plan_resolution` from the manifest to interpret the assignments correctly.

---

## 3. Implementation plan — three phases

### Phase 1: Resolution threading (this spec)

**Scope**: make `--resolution` flow through plan generation and manifest.

Changes:
- `StateConfig.resolution` already exists → add `plan_resolution` to `PlanManifest`
- `run_single_state()`: pass resolution to adjacency loading (already done); write to manifest (new)
- Analysis: read `plan_resolution` from manifest; handle `bg` and `county` units alongside `tract`
- Reporting: use `unit_type` from manifest for human-readable labels

Crates affected: `redist-cli`, `redist-report`, `redist-analysis`

### Phase 2: County-level coarsening (Option B — no extra data)

**Scope**: build county adjacency in-memory from tract GEOIDs.

New function in `redist-cli/src/adjacency_loader.rs`:
```rust
pub fn build_county_coarsening(
    tract_geoids: &HashMap<usize, String>,
    tract_adj: &[Vec<usize>],
    tract_pop: &[i64],
) -> (Vec<Vec<usize>>, Vec<i64>, Vec<usize>)
// Returns: (county_adj, county_pop, fine_to_coarse)
// county_adj: county-level adjacency. Two counties A and B are adjacent
//   iff any tract in A is adjacent (in the tract graph) to any tract in B.
//   This correctly propagates geographic adjacency from the tract level up.
// county_pop: sum of tract populations per county
// fine_to_coarse[tract_idx]: county index (derived from GEOID prefix[:5])
```

This enables `--search multiscale --multiscale-fine tract --multiscale-coarse county` with no new data requirement.

### Phase 3: Block-group fine level (Option A)

**Scope**: run redistricting at BG resolution; analysis at BG level.

Changes:
- `run_single_state()`: when `plan_resolution == "bg"`, load `_bg_adjacency_` as primary graph
- `build_bg_to_tract_partition()`: derive `fine_to_coarse` from BG vs tract GEOID prefixes
- Analysis: BG-level Polsby-Popper (requires BG geometries — separate fetch), population balance at BG level
- `redist analyze`: detect `plan_resolution == "bg"` and use BG geometries

---

## 4. GEOID partition derivation

The partition from fine to coarse is always derivable from GEOIDs:

```rust
/// Build fine→coarse partition from GEOID prefix matching.
/// fine_geoids: index_to_geoid at the fine level (e.g., BG)
/// coarse_geoids: index_to_geoid at the coarse level (e.g., tract)
/// Returns: fine_to_coarse[fine_idx] = coarse_idx
pub fn derive_partition(
    fine_geoids: &HashMap<usize, String>,
    coarse_geoids: &HashMap<usize, String>,
) -> Result<Vec<usize>, String> {
    // Validate uniform coarse GEOID length (FIPS GEOIDs are uniform within a level,
    // but assert explicitly rather than silently sampling .next() and hoping).
    let prefix_len = coarse_geoids.values()
        .map(|g| g.len())
        .reduce(|a, b| {
            assert_eq!(a, b, "all coarse GEOIDs must have uniform length; got {a} and {b}");
            a
        })
        .unwrap_or(11);
    
    // Build coarse GEOID → coarse index lookup
    let coarse_lookup: HashMap<&str, usize> = coarse_geoids.iter()
        .map(|(&idx, geoid)| (geoid.as_str(), idx))
        .collect();
    
    let n_fine = fine_geoids.len();
    let mut partition = vec![usize::MAX; n_fine];
    for (&fine_idx, fine_geoid) in fine_geoids {
        let prefix = &fine_geoid[..prefix_len.min(fine_geoid.len())];
        let coarse_idx = coarse_lookup.get(prefix)
            .ok_or_else(|| format!("fine GEOID {fine_geoid} (prefix '{prefix}') has no matching coarse unit — ensure both adjacency files are from the same census year"))?;
        partition[fine_idx] = *coarse_idx;
    }
    // Verify no orphans remain
    if let Some(i) = partition.iter().position(|&v| v == usize::MAX) {
        return Err(format!("fine unit {i} has no coarse mapping (usize::MAX)"));
    }
    Ok(partition)
}
```

For BG→tract: `prefix_len = 11` (tract GEOID length)  
For tract→county: `prefix_len = 5` (county GEOID length)  
For BG→county: `prefix_len = 5`

---

## 5. Seeding (resolution-aware)

Multi-scale steps use the same `MSC_STEP_` seeding formula regardless of resolution level. The resolution is recorded in the manifest, not encoded in the seed — changing resolution with the same `base_seed` produces different plans (the adjacency graph differs), which is correct.

---

## 6. Audit chain additions

One manifest example per supported combination — each includes `index_to_geoid_file` so a verifier can reconstruct the full partition independently without running code:

**Option A — BG→tract**:
```json
"plan_resolution": "bg",
"n_units": 5012,
"unit_type": "census block group",
"index_to_geoid_file": "nc_bg_adjacency_2020_geoids.json",
"multiscale_fine": "bg",
"multiscale_coarse": "tract",
"fine_to_coarse_formula": "geoid_prefix[:11]",
"fine_to_coarse_comment": "first 11 chars of 12-char BG GEOID = parent tract GEOID"
```

**Option B — tract→county**:
```json
"plan_resolution": "tract",
"n_units": 2195,
"unit_type": "census tract",
"index_to_geoid_file": "nc_adjacency_2020_geoids.json",
"multiscale_fine": "tract",
"multiscale_coarse": "county",
"fine_to_coarse_formula": "geoid_prefix[:5]",
"fine_to_coarse_comment": "first 5 chars of 11-char tract GEOID = parent county FIPS"
```

**Option C — BG→county**:
```json
"plan_resolution": "bg",
"n_units": 5012,
"unit_type": "census block group",
"index_to_geoid_file": "nc_bg_adjacency_2020_geoids.json",
"multiscale_fine": "bg",
"multiscale_coarse": "county",
"fine_to_coarse_formula": "geoid_prefix[:5]",
"fine_to_coarse_comment": "first 5 chars of 12-char BG GEOID = parent county FIPS"
```

A verifier who holds `index_to_geoid_file` (always co-located with the adjacency file, same public data directory) and `fine_to_coarse_formula` can reconstruct every `fine_to_coarse[i]` entry without code — they apply the prefix rule to each GEOID and look up the coarse index.

For non-multiscale runs: `multiscale_fine`, `multiscale_coarse`, `fine_to_coarse_formula`, and `fine_to_coarse_comment` are absent.

---

## 7. Test invariants

### L0 (unit tests)

- `derive_partition(bg_geoids, tract_geoids)`: every BG maps to a valid tract index (no usize::MAX in output)
- `derive_partition(tract_geoids, county_geoids)`: every tract maps to a valid county index
- County partition covers all tracts (no orphaned tracts)
- `build_county_coarsening()`: county_pop sums equal total tract pop
- County adjacency is symmetric
- BG→tract prefix length is 11; tract→county prefix length is 5

### L1 (integration, synthetic)

- Synthetic 12-node graph with 3 tracts of 4 BGs each: `derive_partition` returns [0,0,0,0,1,1,1,1,2,2,2,2]
- `build_county_coarsening` on VT (1 district, small graph): single county, all tracts map to county 0
- Plan at BG level with `plan_resolution=bg`: manifest correctly records n_units = n_BGs

### L2 (#[ignore])

- NC BG→tract partition: all NC block groups map to one of NC's census tracts (no orphans). NC has 2,195 tracts and ~5,000 BGs per the 2020 Census P.L. 94-171 file; the exact BG count is determined at test time from the loaded file, not hardcoded.
- NC tract→county partition: all NC tracts map to one of NC's 100 counties (first 5 chars of tract GEOID = county FIPS)
- VT BG→tract: VT's small size makes this fast; useful as a non-ignored L1.5 smoke test if BG file is available

---

## 8. Open questions (deferred)

1. **BG geometry for Polsby-Popper**: BG-level compactness requires BG shape files — separate fetch step. Deferred to Phase 3.
2. **Mixed-resolution comparison**: can a BG-level plan be compared to a tract-level plan? (No — different unit count. Must aggregate first.)
3. **County-level redistricting**: `--resolution county` as a standalone mode (k very large states using county-level plans). Deferred.
4. **Which resolution is default for each chamber?**: congressional = tract (current); state house = BG (more precision for smaller districts)?
