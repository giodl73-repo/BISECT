# Scenario Simulations — Practitioner Redistricting Toolkit

**Date**: 2026-04-26
**Status**: Draft
**Purpose**: Trace end-to-end user workflows through the spec stack, identify cross-spec data contracts, and write L0 test assertions that validate spec behavior.

---

## How to read this document

Each scenario:
1. States a concrete input (state, year, flags)
2. Traces through the specs the workflow touches, in execution order
3. Identifies the data contracts between specs (files produced/consumed, field requirements)
4. Lists L0 test assertions (unit-level, no pipeline required) that would catch regressions

---

## Scenario 1: WA house redistricting end-to-end

**Input**: State=WA, Year=2020, 98 house districts, balance-tolerance 5.0%, seed 42

```bash
redist state --state WA --year 2020 --version WA_Plans \
  --districts 98 --chamber house --balance-tolerance 5.0 --seed 42
redist analyze --label washington_house_2020 --types contiguity splits
redist report --label washington_house_2020 --format html
```

### Spec trace

**Spec 1 — Custom Parameters**
- `--districts 98 --chamber house` → `StateConfig { num_districts_override: Some(98), chamber: "house" }`
- `--balance-tolerance 5.0` → `balance_tolerance: 0.05` (fraction form internally)
- `--label` omitted → default generated: `washington_house_2020` (state_lower + "_" + chamber + "_" + year)
- Chamber-aware default would also have produced 5.0% (house default), so explicit flag is redundant but must win over default
- PlanManifest written with `balance_tolerance_pct: 5.0`, `chamber: "house"`, `num_districts: 98`
- `tiger_source_url` recorded from Census URL table for FIPS 53 + year 2020
- `adjacency_file` records filename only (not local path), per R3 Finding 4

**Spec 0 — RPLAN format**
- If `--output-format rplan` is added, RPLAN written with:
  - `"rplan_version": "0.1"` at top level ONLY (not inside metadata, per R3 Finding 1)
  - `metadata.chamber: "house"`, `metadata.num_districts: 98`
  - `assignments`: 11-character GEOID keys → 1-based district integers
  - `geometry`: null (assignments-only by default for performance)

**Spec 3 — Constraint Analysis**
- `--types contiguity splits` runs both analyzers
- `contiguity.json`: BFS over each of 98 house districts
- `splits.json`: county FIPS extracted from 11-char GEOIDs (zero-dependency); municipality splits require `data/2020/geography/wa_place_tract_2020.csv`
- WA-specific splits standard applied: `legal_standard: "WA Const. Art. II §43 — counties shall be preserved where possible"`
- Exit code: bitfield OR of all violations (per R3 Finding 5). All pass → exit 0.

**Spec 6 — Commission Reports**
- Report section 1 (executive summary) includes `chamber: house`
- Report section 3 (geographic constraints) reads `contiguity.json` + `splits.json`
- Report section 9 (audit trail) reads PlanManifest verbatim; generates verification command from manifest fields
- All analysis files consumed from `plans/washington_house_2020/analysis/`

### Data contracts

| Producer | File | Consumer | Required fields |
|----------|------|----------|-----------------|
| Spec 1 | `manifest.json` | Spec 3, Spec 6 | `balance_tolerance_pct`, `chamber`, `num_districts`, `tiger_source_url`, `adjacency_file` (filename only) |
| Spec 1 | `final_assignments.json` | Spec 3, Spec 6 | All GEOID keys 11 chars; values 1..=98 |
| Spec 3 | `analysis/contiguity.json` | Spec 6 | `all_contiguous`, `districts[].contiguous` |
| Spec 3 | `analysis/splits.json` | Spec 6 | `counties.split`, `legal_standard`, `compliance_assessment` |

### L0 test assertions

```rust
// Spec 1 — manifest records chamber-aware tolerance
#[test]
fn test_wa_house_manifest_has_chamber_aware_tolerance() {
    let cfg = StateConfig {
        chamber: "house".into(),
        balance_tolerance: None,  // not explicitly set
        ..Default::default()
    };
    let manifest = build_manifest(&cfg);
    // house default is 5.0%
    assert_eq!(manifest.balance_tolerance_pct, 5.0);
}

// Spec 0 — rplan_version must not appear inside metadata
#[test]
fn test_rplan_version_top_level_only() {
    let rplan = generate_rplan(&test_plan());
    // top-level field present
    assert_eq!(rplan["rplan_version"], "0.1");
    // not duplicated inside metadata
    assert!(rplan["metadata"].get("rplan_version").is_none());
}

// Spec 3 — WA splits output includes state-specific legal standard
#[test]
fn test_wa_splits_uses_state_standard() {
    let result = analyze_county_splits_with_standard(&wa_assignments(), "WA");
    assert!(result.legal_standard.contains("WA Const. Art. II"));
    assert!(result.compliance_assessment.is_some());
}

// Spec 1 — label default generation
#[test]
fn test_label_default_washington_house_2020() {
    let label = generate_default_label("WA", "house", "2020");
    assert_eq!(label, "washington_house_2020");
}
```

---

## Scenario 2: Court submission audit trail

**Input**: Same WA house plan as Scenario 1 (label: `washington_house_2020`)

```bash
redist report --label washington_house_2020 --audit-only --out reports/audit.json
```

### Spec trace

**Spec 1 — PlanManifest (source of truth for audit)**
- `manifest.json` must contain all fields required for independent reproduction:
  - `tiger_sha256` — SHA-256 of source TIGER shapefile (not `.adj.bin`)
  - `tiger_source_url` — Census URL from which the shapefile was obtained (R3 Finding 4)
  - `binary_download_url` — GitHub Releases URL for the exact binary used (Spec 6 R2 amendment)
  - `binary_release_sha256` — hash published on the Releases page
  - `adjacency_build_command` + `adjacency_build_version` — how `.adj.bin` was derived
  - `seed`, `binary_version`, `binary_sha256`, `created_at`

**Spec 6 — Audit chain of custody**
- Audit section reads PlanManifest verbatim
- Verification command is machine-generated from manifest fields:
  ```
  1. Install: redist v0.1.0 (SHA-256: {binary_release_sha256})
     Download: {binary_download_url}
  2. Verify input data:
     sha256sum data/2020/tiger/tracts/53/tl_2020_53_tract.shp
     # Expected: {tiger_sha256}
  3. Run:
     redist state --state WA --year 2020 --version WA_Plans \
       --districts 98 --chamber house --label washington_house_2020 \
       --balance-tolerance 5.0 --seed 42
  4. Verify output:
     sha256sum outputs/WA_Plans/2020/plans/washington_house_2020/final_assignments.json
     # Expected: {output_sha256}
  ```
- Verification command is complete: contains seed, state, binary version, all expected hashes

### Data contracts

| Field | Source | Requirement |
|-------|--------|-------------|
| `tiger_sha256` | PlanManifest | SHA-256 of source shapefile, NOT of `.adj.bin` |
| `tiger_source_url` | PlanManifest | Full Census Bureau URL |
| `binary_download_url` | PlanManifest | URL to GitHub release page |
| `binary_release_sha256` | PlanManifest | Hash from GitHub release |
| Verification command | Spec 6 `audit.rs` | Complete: seed + state + version + expected output hash |

### L0 test assertions

```python
def test_manifest_has_tiger_sha256():
    """SHA-256 must be of source shapefile, not derived adj.bin."""
    manifest = load_manifest("plans/washington_house_2020/manifest.json")
    assert "tiger_sha256" in manifest
    # Must not be the same as adjacency_sha256 (different files)
    assert manifest["tiger_sha256"] != manifest["adjacency_sha256"]

def test_manifest_has_binary_download_url():
    """Binary provenance URL must be present and point to GitHub releases."""
    manifest = load_manifest("plans/washington_house_2020/manifest.json")
    assert "binary_download_url" in manifest
    assert "github.com" in manifest["binary_download_url"]
    assert manifest["binary_version"] in manifest["binary_download_url"]

def test_verification_command_is_complete():
    """Machine-generated verification command must include all required fields."""
    audit = load_audit_json("reports/audit.json")
    cmd = audit["verification_command"]
    assert "--seed" in cmd or "seed" in cmd
    assert "--state WA" in cmd
    assert manifest_version in cmd  # binary version
    # expected output hash must appear
    assert audit["expected_output_sha256"] in cmd
```

---

## Scenario 3: Cross-tool RPLAN roundtrip

**Input**: Generate WA plan with redist, export as RPLAN, import into GerryChain, re-export as RPLAN, reimport to redist

```bash
# Step 1: generate
redist state --state WA --year 2020 --version WA_Plans --label wa_roundtrip --output-format rplan
# Step 2: GerryChain roundtrip (external tool)
python gerrychain_import_export.py wa_roundtrip.rplan wa_roundtrip_gc.rplan
# Step 3: reimport to redist
redist import --file wa_roundtrip_gc.rplan --state WA --year 2020 --label wa_roundtrip_check
redist analyze --label wa_roundtrip_check --types contiguity
```

### Spec trace

**Spec 0 — RPLAN format (export)**
- `assignments`: 11-character GEOID keys → district integers (1-based)
- GeoJSON `geometry` section: `[longitude, latitude]` coordinate order (RFC 7946 §3.1.1)
- `rplan_version: "0.1"` at top level only

**GerryChain v2.3 compatibility**
- GerryChain reads `assignments` field from RPLAN (plural key)
- GerryChain internally uses `assignment` (singular); converter:
  ```python
  gc_partition = {"assignment": rplan["assignments"]}
  ```
- Re-export from GerryChain → RPLAN reverses this mapping
- The `assignments` dict must be identical: same GEOIDs, same district integers

**Spec 6 — Import (`redist import`)**
- Point-in-polygon assigns each GEOID to the district polygon containing its centroid
- After import: `len(assignments) == num_tracts_in_WA_2020`
- `manifest.json` records `source: "imported"`, `source_file: "wa_roundtrip_gc.rplan"`
- `100% coverage` — all WA tracts present, no gaps

### Data contracts

| Requirement | Spec | Field |
|-------------|------|-------|
| All assignment keys are 11-char GEOIDs | Spec 0 | `assignments` keys match `^\d{11}$` |
| Coordinate order is [lon, lat] | Spec 0 | RFC 7946 §3.1.1 |
| GerryChain reads `assignments` (plural) | Spec 0 compatibility matrix | Converter documented |
| Import produces 100% coverage | Spec 6 import | `len(assignments) == state_tract_count` |

### L0 test assertions

```python
def test_rplan_assignments_all_11char_geoids():
    """All assignment keys must be 11-character Census tract GEOIDs."""
    import re
    rplan = json.load(open("wa_roundtrip.rplan"))
    pattern = re.compile(r"^\d{11}$")
    for geoid in rplan["assignments"].keys():
        assert pattern.match(geoid), f"Invalid GEOID: {geoid}"

def test_rplan_geojson_coordinate_order():
    """GeoJSON coordinates must be [lon, lat] per RFC 7946 §3.1.1."""
    rplan = json.load(open("wa_roundtrip.rplan"))
    if rplan["geometry"] is None:
        return  # assignments-only mode; skip
    for feature in rplan["geometry"]["features"]:
        coords = feature["geometry"]["coordinates"]
        # WA longitude is roughly -124 to -117; latitude is 45 to 49
        # Flatten to first coordinate pair to check order
        first_coord = _first_coord(coords)
        lon, lat = first_coord[0], first_coord[1]
        assert -130 < lon < -110, f"Longitude out of WA range: {lon} (may be lat/lon swapped)"
        assert 44 < lat < 50, f"Latitude out of WA range: {lat}"

def test_gerrychain_roundtrip_identical():
    """Export -> GerryChain import -> re-export produces identical assignments."""
    original = json.load(open("wa_roundtrip.rplan"))["assignments"]
    roundtripped = json.load(open("wa_roundtrip_gc.rplan"))["assignments"]
    assert original == roundtripped, (
        f"Roundtrip changed {sum(1 for k in original if original[k] != roundtripped.get(k))} assignments"
    )
```

---

## Scenario 4: Nesting violation detection

**Input**: Run WA senate --nest senate-in-house where senate district 3 contains tracts from house districts 5, 6, AND 7 (violation — should contain exactly 2)

```bash
redist suite validate --name wa_violation_test --version WA_Plans --year 2020
```

### Spec trace

**Spec 5 — Nesting validation**
- `validate_nesting` iterates over senate districts
- For senate district 3: collect all tract GEOIDs → look up house assignments → find house districts {5, 6, 7}
- `required_ratio: 2` (WA constitutional: each senate = exactly 2 house districts)
- Violation detected: senate 3 spans 3 house districts, not 2
- `NestingValidation { valid: false, violations: [{ senate: 3, house_districts: [5, 6, 7] }] }`

**Spec 3 + Spec 5 — Exit code (bitfield, R3 Finding 5)**
- Nesting violation = bit 2 → exit code 4
- If balance violation also present: exit code 5 (bits 0+2)
- `redist suite validate` returns exit code 4 for nesting-only violation

**Spec 6 — Report**
- Report section 8 (nesting compliance): reads `NestingValidation`
- `valid: false` → report renders "FAIL" with list of violating senate districts
- Suite-level executive summary: nesting compliance = FAIL

### Data contracts

| Spec | Contract |
|------|----------|
| Spec 5 `validate_nesting` | Returns `valid: bool` + `violations: Vec<{senate: usize, house_districts: Vec<usize>}>` |
| Spec 5 exit code | Nesting violation → exit code 4 (bit 2 in bitfield scheme) |
| Spec 6 report | `nesting.valid == false` → section 8 shows "FAIL" |

### L0 test assertions

```rust
#[test]
fn test_nesting_violation_detected() {
    // Senate district 1 contains tracts from house districts 5, 6, AND 7 (3 districts, violation)
    let house = hashmap!{
        "53033000100" => 5,
        "53033000200" => 6,
        "53033000300" => 7,
    };
    let senate = hashmap!{
        "53033000100" => 3,
        "53033000200" => 3,
        "53033000300" => 3,
    };
    let result = validate_nesting(&house, &senate, 2);  // ratio=2 (WA)
    assert!(!result.valid);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].senate_district, 3);
    let mut hd = result.violations[0].house_districts.clone();
    hd.sort();
    assert_eq!(hd, vec![5, 6, 7]);
}

#[test]
fn test_nesting_exit_code_bitfield() {
    // Nesting violation alone → exit code 4 (bit 2)
    let result = validate_nesting(&house_with_violation(), &senate_causing_violation(), 2);
    let exit_code = compute_exit_code(
        balance_ok: true,
        contiguity_ok: true,
        nesting_ok: result.valid,
        data_ok: true,
    );
    assert_eq!(exit_code, 4);  // bit 2: nesting
}

#[test]
fn test_report_nesting_section_fail() {
    // NestingValidation with valid=false → report section renders "FAIL"
    let nesting = NestingValidation { valid: false, violations: vec![...] };
    let report = build_report_section_nesting(&nesting);
    assert!(report.compliance_status == "FAIL");
    assert!(!report.violations.is_empty());
}
```

---

## Scenario 5: Small-chamber partisan metrics

**Input**: Vermont (1 congressional district), run partisan analysis

```bash
redist state --state VT --year 2020 --version VT_Plans --chamber congressional
redist analyze --label vermont_congressional_2020 --types partisan
```

### Spec trace

**Spec 1 — Custom Parameters**
- VT: 1 congressional district (from `config_2020.py`)
- No override needed; default congressional district count used
- `balance_tolerance: 0.5%` (congressional default)
- Label defaults to `vermont_congressional_2020`

**Spec 4 — Partisan Metrics (R2 amendment: CI suppression)**
- `num_districts = 1 < 10` → bootstrap CI suppressed
- Output:
  ```json
  {
    "ci_available": false,
    "ci_reason": "Bootstrap CI requires >=10 districts (found 1)"
  }
  ```
- Efficiency gap, mean-median, partisan bias: still computed (mathematically valid for N=1, though degenerate)
  - EG for 1 district: all votes in one district; wasted votes computation trivially defined
  - MM for 1 district: mean == median == single value → MM = 0 always
  - PB for 1 district: degenerate (1 district winning = 100% seat share) — note in output
- Output `available: true` (election data present), `ci_available: false`

**Spec 6 — Report**
- Section 4 (partisan fairness): renders metrics without CI range
- "Confidence intervals not available (fewer than 10 districts)" note displayed
- Does not silently omit the partisan section — it renders with available data

### Data contracts

| Spec | Contract |
|------|----------|
| Spec 4 output | `ci_available: false` when `num_districts < 10`; `ci_reason` field present |
| Spec 4 output | EG value present even when CI unavailable |
| Spec 6 report | Partisan section renders gracefully without CI |

### L0 test assertions

```rust
#[test]
fn test_single_district_partisan_no_ci() {
    let districts = vec![DistrictElection { dem_votes: 200000.0, rep_votes: 100000.0 }];
    let result = compute_partisan_metrics(&districts, /*n_districts=*/ 1);
    assert!(!result.ci_available);
    assert!(result.ci_reason.contains("requires >=10 districts"));
    assert!(result.ci_reason.contains("found 1"));
    // CI bounds must not be present
    assert!(result.metrics.efficiency_gap.ci_95_low.is_none());
    assert!(result.metrics.efficiency_gap.ci_95_high.is_none());
}

#[test]
fn test_single_district_eg_computed() {
    // EG is still computed even for N=1
    let districts = vec![DistrictElection { dem_votes: 200000.0, rep_votes: 100000.0 }];
    let result = compute_partisan_metrics(&districts, 1);
    assert!(result.metrics.efficiency_gap.value.is_finite());
    // value should be non-None
    assert!(result.metrics.efficiency_gap.value.abs() < 1.0);  // EG is in [-1, 1]
}

#[test]
fn test_report_partisan_no_ci_section() {
    // Report must render partisan section without crashing when ci_available=false
    let partisan_json = json!({
        "available": true,
        "ci_available": false,
        "ci_reason": "Bootstrap CI requires >=10 districts (found 1)",
        "metrics": {
            "efficiency_gap": {"value": 0.15, "direction": "Democratic"}
        }
    });
    let html = render_partisan_section(&partisan_json);
    assert!(html.contains("efficiency_gap") || html.contains("Efficiency Gap"));
    assert!(html.contains("not available") || html.contains("ci_available"));
    // Must not panic or produce empty section
    assert!(!html.is_empty());
}
```

---

## Scenario 6: Label collision protection

**Input**: Run `redist state --state WA --chamber house --year 2020` twice (same default label `washington_house_2020`)

```bash
# First run — succeeds
redist state --state WA --year 2020 --version WA_Plans \
  --districts 98 --chamber house --seed 42
# → Creates plans/washington_house_2020/manifest.json

# Second run — no --force — should fail
redist state --state WA --year 2020 --version WA_Plans \
  --districts 98 --chamber house --seed 99

# Third run — with --force — should succeed
redist state --state WA --year 2020 --version WA_Plans \
  --districts 98 --chamber house --seed 99 --force
```

### Spec trace

**Spec 1 — Label collision (R3 Finding 10)**
- First run: label defaults to `washington_house_2020`; no `manifest.json` exists; plan created successfully
- Second run: same default label; `manifest.json` found at `plans/washington_house_2020/manifest.json`
  - `--force` not set → exit non-zero immediately (before any computation begins)
  - Error message: `ERROR: Plan 'washington_house_2020' already exists (created 2026-04-26T14:23:00Z). Use --force to overwrite or choose a different --label.`
  - Timestamp from existing manifest's `created_at` field
- Third run: `--force` set → atomically overwrite plan directory contents; proceed normally; exit 0

**Implementation note**: Collision check must happen before any adjacency loading or METIS calls — fail fast, no wasted computation.

### Data contracts

| State | Contract |
|-------|----------|
| First run | Creates `manifest.json` with `created_at` timestamp |
| Second run (no --force) | Exits non-zero; error message contains label name and `created_at` from existing manifest |
| Third run (--force) | Overwrites; exits 0; new `manifest.json` has updated `created_at` |

### L0 test assertions

```rust
#[test]
fn test_label_collision_exits_nonzero() {
    // Set up: manifest.json already exists for label
    let tmp = tempdir();
    let plan_dir = tmp.path().join("plans/washington_house_2020");
    std::fs::create_dir_all(&plan_dir).unwrap();
    let manifest_path = plan_dir.join("manifest.json");
    write_test_manifest(&manifest_path, "2026-04-26T14:23:00Z");

    // Attempt to create same label without --force
    let result = run_redist_state(&[
        "--state", "WA", "--year", "2020",
        "--chamber", "house", "--districts", "98",
        // no --force
    ], tmp.path());

    assert_ne!(result.exit_code, 0, "Expected non-zero exit for label collision");
}

#[test]
fn test_label_collision_error_message() {
    // Error message must contain label name and existing timestamp
    let tmp = tempdir_with_existing_manifest("washington_house_2020", "2026-04-26T14:23:00Z");

    let result = run_redist_state(&[
        "--state", "WA", "--year", "2020",
        "--chamber", "house", "--districts", "98",
    ], tmp.path());

    let stderr = result.stderr;
    assert!(stderr.contains("washington_house_2020"), "Error must name the colliding label");
    assert!(stderr.contains("2026-04-26T14:23:00Z"), "Error must include existing plan timestamp");
    assert!(stderr.contains("--force"), "Error must suggest --force flag");
}

#[test]
fn test_label_force_overwrites() {
    // With --force, second run succeeds (exit 0) and manifest is updated
    let tmp = tempdir_with_existing_manifest("washington_house_2020", "2026-04-26T14:23:00Z");

    let result = run_redist_state(&[
        "--state", "WA", "--year", "2020",
        "--chamber", "house", "--districts", "98",
        "--force",
    ], tmp.path());

    assert_eq!(result.exit_code, 0);
    // Manifest should be overwritten with new timestamp
    let new_manifest = load_manifest(tmp.path().join("plans/washington_house_2020/manifest.json"));
    assert_ne!(new_manifest["created_at"], "2026-04-26T14:23:00Z");
}
```

---

## Cross-scenario data contract summary

The following table summarizes which files and fields are validated across all six scenarios:

| File | Key fields | Scenarios |
|------|-----------|-----------|
| `manifest.json` | `balance_tolerance_pct`, `chamber`, `num_districts`, `tiger_sha256`, `tiger_source_url`, `binary_download_url`, `created_at` | 1, 2, 6 |
| `final_assignments.json` | All keys 11-char GEOIDs; values 1..=num_districts | 1, 3 |
| `*.rplan` | `rplan_version` at top level only; `assignments` keys 11-char; `geometry` [lon, lat] | 1, 3 |
| `analysis/contiguity.json` | `all_contiguous`, `districts[].contiguous`, `districts[].component_count` | 1, 4 |
| `analysis/splits.json` | `counties.split`, `legal_standard`, `compliance_assessment` | 1 |
| `analysis/partisan.json` | `ci_available`, `ci_reason`, `metrics.efficiency_gap.value` | 5 |
| Suite nesting result | `valid`, `violations[].senate_district`, `violations[].house_districts` | 4 |

---

## L0 test file locations

| Scenario | Test file |
|----------|-----------|
| 1 | `redist/crates/redist-cli/tests/test_wa_house_e2e.rs` |
| 2 | `redist/crates/redist-report/tests/test_audit_chain.py` |
| 3 | `redist/crates/redist-report/tests/test_rplan_roundtrip.py` |
| 4 | `redist/crates/redist-analysis/tests/test_nesting.rs` |
| 5 | `redist/crates/redist-analysis/tests/test_partisan.rs` |
| 6 | `redist/crates/redist-cli/tests/test_label_collision.rs` |
