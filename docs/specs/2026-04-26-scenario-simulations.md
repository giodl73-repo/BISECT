# Practitioner Toolkit — Scenario Simulations

**Date**: 2026-04-26
**Purpose**: Validate spec data contracts via concrete end-to-end scenarios before implementation. Each scenario traces through the specs it touches and specifies L0 test assertions.

---

## Scenario 1: WA house redistricting end-to-end

**Input**: `redist state --state WA --year 2020 --districts 98 --chamber house --label wa_house_draft1 --seed 42`

**Spec trace**:
- Spec 1: `--districts 98` overrides config (config has 10 congressional) → num_districts=98 in StateConfig
- Spec 1: `--chamber house` → balance_tolerance defaults to 5.0% (chamber-aware, not 0.5%)
- Spec 1: manifest records `chamber: "house"`, `num_districts: 98`, `balance_tolerance_pct: 5.0`
- Spec 0: RPLAN written — `rplan_version: "0.1"` at top level ONLY (not in metadata)
- Spec 3: `redist analyze --types splits` uses WA constitutional standard, not generic preservation score
- Spec 6: report section 1 shows `chamber: house`, section 3 shows WA-specific split standard

**L0 test assertions**:
```rust
test_wa_house_manifest_chamber_aware_tolerance() {
    // StateConfig with chamber="house" → balance_tolerance=0.05 (5%)
    let cfg = StateConfig { chamber: "house".into(), ..default() };
    assert_eq!(cfg.effective_balance_tolerance(), 0.05);
}

test_rplan_version_top_level_only() {
    // Serialized RPLAN must have rplan_version only at root, not in metadata
    let rplan = serialize_plan(&plan);
    let parsed: Value = serde_json::from_str(&rplan).unwrap();
    assert!(parsed["rplan_version"].is_string());
    assert!(parsed["metadata"]["rplan_version"].is_null());
}

test_wa_splits_has_legal_standard_field() {
    // splits.json must contain legal_standard field for WA
    let splits = run_splits_analysis("WA", &assignments);
    assert!(splits.counties.legal_standard.contains("WA Const."));
}
```

---

## Scenario 2: Court submission audit trail

**Input**: WA house plan → submit to court as evidence

**Spec trace**:
- Spec 1: manifest has `tiger_sha256` (source shapefile, not derived adj.bin)
- Spec 1: manifest has `binary_download_url` + `binary_release_sha256` (from GitHub release)
- Spec 6: audit section generates verification command with all hashes
- Spec 6: verification command is machine-generated, contains no local paths

**L0 test assertions**:
```python
def test_manifest_has_tiger_sha256():
    manifest = load_manifest("wa_house_draft1")
    assert "tiger_sha256" in manifest
    assert len(manifest["tiger_sha256"]) == 64  # SHA-256 hex

def test_manifest_has_binary_download_url():
    manifest = load_manifest("wa_house_draft1")
    assert "binary_download_url" in manifest
    assert "github.com" in manifest["binary_download_url"]

def test_verification_command_no_local_paths():
    report = generate_audit_report("wa_house_draft1")
    cmd = report["audit"]["verification_command"]
    assert "C:\\" not in cmd  # no Windows local paths
    assert "/home/" not in cmd  # no Unix local paths
    assert "--seed" in cmd  # seed present for reproducibility
```

---

## Scenario 3: Cross-tool RPLAN roundtrip

**Input**: Generate WA plan → export as RPLAN → convert to GerryChain → convert back to RPLAN

**Spec trace**:
- Spec 0: export uses 11-char GEOIDs in `assignments`
- Spec 0: GeoJSON coordinates in [lon, lat] order (RFC 7946)
- GerryChain v2.3: reads `assignments` field (its own field is `assignment` singular)
- Spec 0: GerryChain→RPLAN conversion maps `assignment` → `assignments`
- Spec 6: `redist import` achieves 100% tract coverage

**L0 test assertions**:
```python
def test_rplan_all_assignments_11char_geoids():
    rplan = load_rplan("wa_house_draft1.rplan")
    for geoid in rplan["assignments"].keys():
        assert len(geoid) == 11, f"GEOID {geoid!r} must be 11 chars"
        assert geoid.isdigit(), f"GEOID {geoid!r} must be numeric"

def test_rplan_geojson_lon_lat_order():
    rplan = load_rplan("wa_house_draft1.rplan")
    if rplan["geometry"] is not None:
        coords = rplan["geometry"]["features"][0]["geometry"]["coordinates"]
        first_coord = coords[0][0][0]  # [lon, lat]
        lon, lat = first_coord[0], first_coord[1]
        # WA longitude is negative (west), latitude is positive
        assert lon < 0, f"longitude {lon} should be negative for WA"
        assert 40 < lat < 50, f"latitude {lat} out of WA range"

def test_gerrychain_roundtrip_preserves_assignments():
    rplan = load_rplan("wa_house_draft1.rplan")
    gerrychain = rplan_to_gerrychain(rplan)
    rplan2 = gerrychain_to_rplan(gerrychain, metadata=rplan["metadata"])
    assert rplan["assignments"] == rplan2["assignments"]
```

---

## Scenario 4: Nesting violation detection

**Input**: WA senate where district 3 contains tracts from house districts 5, 6, AND 7

**Spec trace**:
- Spec 5: `validate_nesting` returns `valid=false, violations=[{senate: 3, house_districts: [5,6,7]}]`
- Spec 3 R3: exit code = 4 (bit 2, nesting violation)
- Spec 6: report section 8 shows `nesting: FAIL`

**L0 test assertions**:
```rust
test_nesting_violation_senate_contains_three_house_districts() {
    let house = hashmap!{"t0"=>1,"t1"=>2,"t2"=>3,"t3"=>3,"t4"=>5,"t5"=>5,"t6"=>6,"t7"=>6,"t8"=>7,"t9"=>7};
    let senate = hashmap!{"t0"=>1,"t1"=>1,"t2"=>3,"t3"=>3,"t4"=>3,"t5"=>3,"t6"=>3,"t7"=>3,"t8"=>3,"t9"=>3};
    let result = validate_nesting(&house, &senate, 2);
    assert!(!result.valid);
    let v = &result.violations[0];
    assert_eq!(v.senate_district, 3);
    assert_eq!(v.house_districts.len(), 3); // contains 3 house districts
}

test_nesting_exit_code_is_bit2() {
    // nesting violation only → exit code 4 (bit 2)
    assert_eq!(compute_exit_code(false, false, true, false), 4);
}

test_balance_and_nesting_exit_code() {
    // balance violation + nesting violation → bits 0 + 2 = 5
    assert_eq!(compute_exit_code(true, false, true, false), 5);
}
```

---

## Scenario 5: Small-chamber partisan metrics

**Input**: Vermont (1 congressional district), `redist analyze --types partisan`

**Spec trace**:
- Spec 4: `num_districts = 1 < 10` → CI suppressed, `ci_available: false`
- Spec 4: EG, MM, PB are still computed (valid for N=1, just no CI)
- Spec 6: report section 4 renders metrics without CI table

**L0 test assertions**:
```rust
test_single_district_ci_suppressed() {
    let districts = vec![make_district(1, 0.67, 0.33)];
    let result = compute_partisan_metrics(&districts);
    assert!(!result.efficiency_gap.ci_available);
    assert_eq!(result.efficiency_gap.ci_reason.as_deref(),
        Some("Bootstrap CI requires >=10 districts (found 1)"));
}

test_single_district_metrics_still_computed() {
    let districts = vec![make_district(1, 0.67, 0.33)];
    let result = compute_partisan_metrics(&districts);
    // EG is defined even for 1 district
    assert!(result.efficiency_gap.value.is_finite());
}
```

---

## Scenario 6: Label collision protection

**Input**: `redist state --state WA --chamber house --year 2020` run twice (same default label)

**Spec trace**:
- First run: creates `plans/washington_house_2020/manifest.json` → exit 0
- Second run without `--force`: detects existing manifest → exit 1 with timestamp error
- Second run with `--force`: overwrites → exit 0

**L0 test assertions**:
```rust
test_label_collision_exits_nonzero_without_force() {
    let tmp = TempDir::new().unwrap();
    // Create existing manifest
    let plan_dir = tmp.path().join("plans").join("washington_house_2020");
    std::fs::create_dir_all(&plan_dir).unwrap();
    std::fs::write(plan_dir.join("manifest.json"),
        r#"{"created_at":"2026-04-26T14:23:00Z"}"#).unwrap();
    // Attempt to create without --force
    let result = check_plan_collision(&plan_dir, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
    assert!(result.unwrap_err().to_string().contains("2026-04-26"));
}

test_label_force_flag_allows_overwrite() {
    let tmp = TempDir::new().unwrap();
    let plan_dir = tmp.path().join("plans").join("wa_house");
    std::fs::create_dir_all(&plan_dir).unwrap();
    std::fs::write(plan_dir.join("manifest.json"), b"{}").unwrap();
    // With --force, no error
    assert!(check_plan_collision(&plan_dir, true).is_ok());
}
```
