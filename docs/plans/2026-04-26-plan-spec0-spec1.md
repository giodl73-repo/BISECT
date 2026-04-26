# Spec 0 (RPLAN Format) + Spec 1 (Custom Parameters) — TDD Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the RPLAN v0.1 interchange format and custom redistricting parameters. `redist state`/`redist states`/`redist run` gain `--districts`, `--chamber`, `--label`, `--population-source`, `--balance-tolerance` flags. A new `redist-report` crate handles RPLAN read/write/validate. `redist validate --file plan.rplan` verifies format and coverage. `PlanManifest` provides a full audit chain. `plans/{label}/` output trees coexist with legacy `states/{state}/` paths. Label collision exits non-zero without `--force`. `redist migrate --state X --label Y` copies legacy plans into the new tree.

**Specs:** Spec 0 (RPLAN Format), Spec 1 (Custom Parameters) + R3 board amendments

**Architecture:**
- New `redist-report` crate: `rplan.rs` (read/write/validate), `manifest.rs` (PlanManifest)
- Updated `redist-core` crate: `StateConfig` additions, `assert_balanced()` with tolerance param
- Updated `redist-cli`: `--districts`, `--chamber`, `--label`, `--population-source`, `--balance-tolerance`, `--force` flags; `Commands::Validate`, `Commands::Migrate`

---

## File Map

| File | Action |
|------|--------|
| `redist/crates/redist-report/Cargo.toml` | **Create** — new crate |
| `redist/crates/redist-report/src/lib.rs` | **Create** — public API |
| `redist/crates/redist-report/src/rplan.rs` | **Create** — RPLAN reader/writer/validator |
| `redist/crates/redist-report/src/manifest.rs` | **Create** — PlanManifest struct + SHA-256 hashing |
| `redist/crates/redist-core/src/config.rs` | **Modify** — add `StateConfig` fields: `num_districts_override`, `chamber`, `label`, `population_source`, `balance_tolerance`, `write_manifest`, `force` |
| `redist/crates/redist-core/src/balance.rs` | **Modify** — `assert_balanced(tolerance: f64)` accepts param |
| `redist/crates/redist-core/src/population.rs` | **Create** — `PopulationSource` enum + VAP/CVAP CSV loading |
| `redist/crates/redist-cli/src/args.rs` | **Modify** — add new flags to `StateArgs`; add `ValidateArgs`, `MigrateArgs` |
| `redist/crates/redist-cli/src/main.rs` | **Modify** — wire `Commands::Validate`, `Commands::Migrate`; pass new flags through |
| `redist/crates/redist-cli/src/validate.rs` | **Create** — `redist validate` dispatcher |
| `redist/crates/redist-cli/src/migrate.rs` | **Create** — `redist migrate` dispatcher |
| `redist/crates/redist-cli/src/paths.rs` | **Modify** — `plan_dir(label)` alongside `state_dir(state)` |
| `redist/Cargo.toml` | **Modify** — add `redist-report` to workspace |
| `tests/unit/test_rplan.py` | **Create** — L0 Python-accessible RPLAN tests |
| `tests/unit/test_manifest.py` | **Create** — L0 manifest / SHA-256 tests |
| `tests/acceptance/test_spec0_spec1_acceptance.py` | **Create** — L2 acceptance tests |

---

## Task 1: `redist-report` crate scaffold + RPLAN writer

**Files:** `redist/crates/redist-report/Cargo.toml`, `src/lib.rs`, `src/rplan.rs`

- [ ] **Create `Cargo.toml`**

```toml
[package]
name = "redist-report"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1"
sha2 = "0.10"
hex = "0.4"
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **L0: Write failing RPLAN structure tests**

```rust
// src/rplan.rs tests

#[test]
fn test_rplan_version_top_level_only() {
    // Serialized RPLAN must have rplan_version only at root, not in metadata
    let plan = RplanFile {
        rplan_version: "0.1".into(),
        metadata: RplanMetadata {
            label: "wa_house_draft1".into(),
            state_fips: "53".into(),
            ..Default::default()
        },
        assignments: HashMap::new(),
        geometry: None,
    };
    let json = serde_json::to_value(&plan).unwrap();
    assert!(json["rplan_version"].is_string());
    assert!(json["metadata"].get("rplan_version").is_none(),
        "rplan_version must NOT appear inside metadata");
}

#[test]
fn test_rplan_geoid_format_valid() {
    let result = validate_geoid_format("530330001001");
    assert!(result.is_ok());
}

#[test]
fn test_rplan_geoid_format_too_short() {
    let result = validate_geoid_format("5303300010");  // 10 chars
    assert!(result.is_err());
}

#[test]
fn test_rplan_geoid_format_non_numeric() {
    let result = validate_geoid_format("5303300010A");
    assert!(result.is_err());
}

#[test]
fn test_rplan_district_range_valid() {
    // district value 1 <= d <= num_districts is valid
    let result = validate_district_range(1, 5, 10);
    assert!(result.is_ok());
}

#[test]
fn test_rplan_district_range_zero_invalid() {
    // district 0 is invalid (must be 1-based)
    let result = validate_district_range(0, 5, 10);
    assert!(result.is_err());
}

#[test]
fn test_rplan_district_range_exceeds_num_districts() {
    let result = validate_district_range(11, 5, 10);
    assert!(result.is_err());
}

#[test]
fn test_rplan_roundtrip_preserves_assignments() {
    let mut assignments = HashMap::new();
    assignments.insert("530330001001".to_string(), 7usize);
    assignments.insert("530330001002".to_string(), 7usize);
    assignments.insert("530330002001".to_string(), 12usize);
    let plan = make_test_rplan(assignments.clone());
    let json = serde_json::to_string(&plan).unwrap();
    let decoded: RplanFile = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.assignments, assignments);
}

#[test]
fn test_rplan_geometry_null_allowed() {
    let plan = make_test_rplan(HashMap::new());
    let json = serde_json::to_value(&plan).unwrap();
    assert!(json["geometry"].is_null());
}
```

- [ ] **Run tests** — expect FAIL (struct not defined)
- [ ] **Implement `RplanFile`, `RplanMetadata`, `validate_geoid_format`, `validate_district_range`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RplanFile {
    pub rplan_version: String,
    pub metadata: RplanMetadata,
    pub assignments: HashMap<String, usize>,
    pub geometry: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RplanMetadata {
    pub label: String,
    pub state_fips: String,
    pub state_code: String,
    pub year: String,
    pub chamber: String,
    pub num_districts: usize,
    pub population_source: String,
    pub balance_tolerance_pct: f64,
    pub created_at: String,
    pub created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_manifest: Option<serde_json::Value>,
    // NOTE: rplan_version is intentionally absent from metadata
}

pub fn validate_geoid_format(geoid: &str) -> Result<(), RplanError> { ... }
pub fn validate_district_range(value: usize, district: usize, num_districts: usize) -> Result<(), RplanError> { ... }
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(report): RPLAN v0.1 struct definitions and GEOID/district validation"`

---

## Task 2: RPLAN validator — schema + coverage

**Files:** `redist/crates/redist-report/src/rplan.rs` (continued)

- [ ] **L0: Write failing validator tests**

```rust
#[test]
fn test_validate_missing_required_field_fails() {
    // metadata.label missing → schema error
    let json = r#"{"rplan_version":"0.1","metadata":{"state_fips":"53"},"assignments":{},"geometry":null}"#;
    let result = validate_rplan_str(json);
    assert!(result.is_err());
}

#[test]
fn test_validate_invalid_geoid_key_fails() {
    let json = r#"{
        "rplan_version": "0.1",
        "metadata": {"label":"x","state_fips":"53","state_code":"WA","year":"2020","chamber":"congressional","num_districts":1,"population_source":"total","balance_tolerance_pct":0.5,"created_at":"2026-04-26T00:00:00Z","created_by":"test"},
        "assignments": {"530330": 1},
        "geometry": null
    }"#;
    let result = validate_rplan_str(json);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("GEOID"));
}

#[test]
fn test_validate_district_out_of_range_fails() {
    // num_districts=3 but assignment value=5
    let result = validate_rplan(&make_rplan_with_bad_district(3, 5));
    assert!(result.is_err());
}

#[test]
fn test_validate_valid_rplan_passes() {
    let rplan = make_minimal_valid_rplan();
    assert!(validate_rplan(&rplan).is_ok());
}

#[test]
fn test_validate_unknown_major_version_fails() {
    let mut rplan = make_minimal_valid_rplan();
    rplan.rplan_version = "2.0".into();
    let result = validate_rplan(&rplan);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("major version"));
}

#[test]
fn test_validate_unknown_minor_version_warns_not_fails() {
    // Minor version bump: 0.2 with 0.1 reader → warn only
    let mut rplan = make_minimal_valid_rplan();
    rplan.rplan_version = "0.2".into();
    let result = validate_rplan(&rplan);
    // Should succeed with a warning, not fail
    assert!(result.is_ok());
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `validate_rplan()` and `validate_rplan_str()`**

```rust
pub struct ValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub fn validate_rplan(rplan: &RplanFile) -> Result<ValidationResult, RplanError> {
    // 1. Schema: all required metadata fields present
    // 2. Version: parse major.minor; fail on major mismatch, warn on minor
    // 3. GEOID format: all keys match ^\d{11}$
    // 4. District range: all values in [1, num_districts]
    // 5. GeoJSON (if geometry present): type="FeatureCollection"
}

pub fn validate_rplan_str(json: &str) -> Result<ValidationResult, RplanError> {
    let rplan: RplanFile = serde_json::from_str(json)?;
    validate_rplan(&rplan)
}
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(report): RPLAN validator — schema, GEOID format, district range, version handling"`

---

## Task 3: `PlanManifest` with SHA-256 audit chain

**Files:** `redist/crates/redist-report/src/manifest.rs`

- [ ] **L0: Write failing manifest tests**

```rust
#[test]
fn test_manifest_sha256_is_deterministic() {
    // Hashing same file content twice gives identical result
    let content = b"test shapefile data";
    let hash1 = sha256_bytes(content);
    let hash2 = sha256_bytes(content);
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64);  // hex-encoded SHA-256
}

#[test]
fn test_manifest_has_tiger_sha256_field() {
    let manifest = PlanManifest {
        tiger_sha256: Some("abc123".repeat(8)[..64].to_string()),
        ..Default::default()
    };
    let json = serde_json::to_value(&manifest).unwrap();
    assert!(json["tiger_sha256"].is_string());
}

#[test]
fn test_manifest_has_tiger_source_url() {
    let manifest = make_test_manifest("53");
    assert!(manifest.tiger_source_url.contains("census.gov"));
    assert!(manifest.tiger_source_url.contains("53"));
}

#[test]
fn test_manifest_adjacency_file_is_filename_not_path() {
    // adjacency_file must be filename only, not a full local path
    let manifest = make_test_manifest("53");
    assert!(!manifest.adjacency_file.contains('/'));
    assert!(!manifest.adjacency_file.contains('\\'));
}

#[test]
fn test_manifest_binary_download_url_contains_github() {
    let manifest = make_test_manifest("53");
    assert!(manifest.binary_download_url.contains("github.com"));
}

#[test]
fn test_label_default_generation_wa_house_2020() {
    let label = default_label("washington", "house", "2020");
    assert_eq!(label, "washington_house_2020");
}

#[test]
fn test_label_default_generation_normalizes_spaces() {
    // "New York" → "new_york_congressional_2020"
    let label = default_label("new york", "congressional", "2020");
    assert_eq!(label, "new_york_congressional_2020");
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `PlanManifest`, `sha256_bytes()`, `default_label()`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlanManifest {
    pub label: String,
    pub state_code: String,
    pub year: String,
    pub chamber: String,
    pub num_districts: usize,
    pub population_source: String,
    pub partition_mode: String,
    pub seed: Option<i64>,
    pub binary_version: String,
    pub binary_sha256: String,
    pub binary_download_url: String,      // GitHub release URL — no local paths
    pub adjacency_file: String,           // filename only (not full path)
    pub adjacency_sha256: String,
    pub adjacency_build_command: String,
    pub adjacency_build_version: String,
    pub tiger_source_url: String,         // Census.gov URL for independent verification
    pub tiger_sha256: Option<String>,     // SHA-256 of upstream TIGER shapefile
    pub created_at: String,
    pub balance_tolerance_pct: f64,
    pub population_balance_valid: bool,
}

pub fn sha256_bytes(data: &[u8]) -> String { ... }  // hex-encoded
pub fn sha256_file(path: &std::path::Path) -> anyhow::Result<String> { ... }
pub fn default_label(state_name: &str, chamber: &str, year: &str) -> String { ... }
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(report): PlanManifest with SHA-256 audit chain, tiger_source_url, filename-only adjacency_file"`

---

## Task 4: Chamber-aware balance tolerance + `StateConfig` additions

**Files:** `redist/crates/redist-core/src/config.rs`, `redist/crates/redist-core/src/balance.rs`, `redist/crates/redist-core/src/population.rs`

- [ ] **L0: Write failing StateConfig and balance tests**

```rust
#[test]
fn test_wa_house_manifest_chamber_aware_tolerance() {
    // StateConfig with chamber="house" → effective_balance_tolerance = 0.05 (5%)
    let cfg = StateConfig {
        chamber: "house".into(),
        balance_tolerance: None,  // no explicit override
        ..Default::default()
    };
    assert!((cfg.effective_balance_tolerance() - 0.05).abs() < 1e-9);
}

#[test]
fn test_congressional_chamber_tolerance_is_half_pct() {
    let cfg = StateConfig {
        chamber: "congressional".into(),
        balance_tolerance: None,
        ..Default::default()
    };
    assert!((cfg.effective_balance_tolerance() - 0.005).abs() < 1e-9);
}

#[test]
fn test_explicit_tolerance_override_wins() {
    // --balance-tolerance 2.0 overrides chamber default
    let cfg = StateConfig {
        chamber: "house".into(),
        balance_tolerance: Some(0.02),
        ..Default::default()
    };
    assert!((cfg.effective_balance_tolerance() - 0.02).abs() < 1e-9);
}

#[test]
fn test_balance_tolerance_5pct_passes_where_halfpct_fails() {
    // District with 3% deviation: passes at 5.0%, fails at 0.5%
    let ideal = 1000usize;
    let district_pop = 1030usize;  // 3% above ideal
    assert!(check_balance(district_pop, ideal, 0.05).is_ok());
    assert!(check_balance(district_pop, ideal, 0.005).is_err());
}

#[test]
fn test_population_source_vap_loads_from_demographics_csv() {
    // Mock CSV with vap column; verify weights are overridden
    let csv_content = "geoid,total,vap,cvap\n530330001001,1000,750,600\n";
    let weights = load_population_weights(
        PopulationSource::Vap,
        csv_content.as_bytes(),
        &["530330001001".to_string()],
    ).unwrap();
    assert_eq!(weights["530330001001"], 750);
}

#[test]
fn test_population_source_cvap_loads_from_demographics_csv() {
    let csv_content = "geoid,total,vap,cvap\n530330001001,1000,750,600\n";
    let weights = load_population_weights(
        PopulationSource::Cvap,
        csv_content.as_bytes(),
        &["530330001001".to_string()],
    ).unwrap();
    assert_eq!(weights["530330001001"], 600);
}

#[test]
fn test_population_source_vap_missing_column_errors() {
    // CSV with no vap column → clear error message
    let csv_content = "geoid,total\n530330001001,1000\n";
    let result = load_population_weights(
        PopulationSource::Vap,
        csv_content.as_bytes(),
        &["530330001001".to_string()],
    );
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("vap"), "Error must name the missing column");
    assert!(msg.contains("--population-source"), "Error must reference the flag");
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `StateConfig` additions, `effective_balance_tolerance()`, `PopulationSource`, `load_population_weights()`**

```rust
pub struct StateConfig {
    // ... existing fields ...
    pub num_districts_override: Option<usize>,
    pub chamber: String,                      // "congressional" | "house" | "senate" | "custom"
    pub label: Option<String>,
    pub population_source: PopulationSource,
    pub balance_tolerance: Option<f64>,       // None = use chamber default
    pub write_manifest: bool,
    pub force: bool,                          // --force: overwrite existing plan
}

impl StateConfig {
    pub fn effective_balance_tolerance(&self) -> f64 {
        self.balance_tolerance.unwrap_or_else(|| match self.chamber.as_str() {
            "congressional" => 0.005,
            _ => 0.05,  // house, senate, custom — state legislative default
        })
    }

    pub fn effective_label(&self, state_name: &str, year: &str) -> String {
        self.label.clone().unwrap_or_else(|| default_label(state_name, &self.chamber, year))
    }
}

#[derive(Debug, Clone, Default)]
pub enum PopulationSource { #[default] Total, Vap, Cvap }
```

- [ ] **Modify `assert_balanced()`** to accept `tolerance: f64` param (replaces hardcoded 0.005)
- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(core): chamber-aware balance tolerance, PopulationSource enum, StateConfig extensions"`

---

## Task 5: Label collision check + `plans/{label}/` path resolution

**Files:** `redist/crates/redist-cli/src/paths.rs`, `redist/crates/redist-report/src/manifest.rs`

- [ ] **L0: Write failing collision and path tests (from Scenario 6)**

```rust
#[test]
fn test_label_collision_exits_nonzero_without_force() {
    let tmp = TempDir::new().unwrap();
    // Create existing manifest
    let plan_dir = tmp.path().join("plans").join("washington_house_2020");
    std::fs::create_dir_all(&plan_dir).unwrap();
    std::fs::write(plan_dir.join("manifest.json"),
        r#"{"created_at":"2026-04-26T14:23:00Z","label":"washington_house_2020"}"#).unwrap();
    // Attempt to create without --force
    let result = check_plan_collision(&plan_dir, false);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("already exists"));
    assert!(msg.contains("2026-04-26"), "Error must include existing plan timestamp");
    assert!(msg.contains("--force"), "Error must mention --force option");
}

#[test]
fn test_label_force_flag_allows_overwrite() {
    let tmp = TempDir::new().unwrap();
    let plan_dir = tmp.path().join("plans").join("wa_house");
    std::fs::create_dir_all(&plan_dir).unwrap();
    std::fs::write(plan_dir.join("manifest.json"),
        r#"{"created_at":"2026-04-26T00:00:00Z"}"#).unwrap();
    // With --force, collision check passes
    assert!(check_plan_collision(&plan_dir, true).is_ok());
}

#[test]
fn test_plan_dir_structure_created() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();
    create_plan_dir(&base, "wa_house_draft1").unwrap();
    assert!(base.join("plans").join("wa_house_draft1").exists());
    assert!(base.join("plans").join("wa_house_draft1").join("analysis").exists());
    assert!(base.join("plans").join("wa_house_draft1").join("maps").exists());
    assert!(base.join("plans").join("wa_house_draft1").join("intermediate").exists());
}

#[test]
fn test_legacy_state_dir_unaffected_by_labeled_run() {
    // Unlabeled runs continue writing to states/{state_name}/
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_path_buf();
    let state_dir = state_output_dir(&base, "washington");
    assert!(state_dir.ends_with("states/washington"));
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `check_plan_collision()`, `create_plan_dir()`, `plan_output_dir()`, `state_output_dir()`**

```rust
/// Check whether a plan directory already exists before beginning computation.
/// Returns Err if manifest.json exists and force=false.
pub fn check_plan_collision(plan_dir: &Path, force: bool) -> anyhow::Result<()> {
    let manifest_path = plan_dir.join("manifest.json");
    if !force && manifest_path.exists() {
        let manifest: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&manifest_path)?
        )?;
        let created_at = manifest["created_at"].as_str().unwrap_or("unknown");
        anyhow::bail!(
            "ERROR: Plan '{}' already exists (created {}). Use --force to overwrite or choose a different --label.",
            plan_dir.file_name().unwrap().to_string_lossy(),
            created_at,
        );
    }
    Ok(())
}

pub fn plan_output_dir(base: &Path, label: &str) -> PathBuf {
    base.join("plans").join(label)
}

pub fn state_output_dir(base: &Path, state_name: &str) -> PathBuf {
    base.join("states").join(state_name)
}
```

- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(cli): plans/{label}/ output tree, label collision check with --force"`

---

## Task 6: `redist validate` command

**Files:** `redist/crates/redist-cli/src/validate.rs`, `redist/crates/redist-cli/src/args.rs`, `redist/crates/redist-cli/src/main.rs`

- [ ] **L0: Write failing validate CLI tests**

```rust
#[test]
fn test_validate_command_exists_in_commands_enum() {
    // Commands::Validate must be a valid variant
    let _cmd = Commands::Validate(ValidateArgs {
        file: PathBuf::from("test.rplan"),
        strict: false,
    });
}

#[test]
fn test_validate_valid_file_exits_zero() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("plan.rplan");
    std::fs::write(&path, make_valid_rplan_json()).unwrap();
    let result = run_validate(&ValidateArgs { file: path, strict: false });
    assert!(result.is_ok());
}

#[test]
fn test_validate_invalid_geoid_exits_nonzero() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("plan.rplan");
    std::fs::write(&path, make_rplan_with_bad_geoid_json()).unwrap();
    let result = run_validate(&ValidateArgs { file: path, strict: false });
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("GEOID"));
}

#[test]
fn test_validate_output_shows_tract_count() {
    // "PASS: valid RPLAN v0.1 (530 tracts, 10 districts, WA 2020 congressional)"
    let output = capture_validate_output("wa_test.rplan");
    assert!(output.contains("tracts"));
    assert!(output.contains("districts"));
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Add `ValidateArgs` to `args.rs`**

```rust
#[derive(Args)]
pub struct ValidateArgs {
    /// Path to .rplan file to validate
    #[arg(long)]
    pub file: PathBuf,
    /// Strict mode: fail on warnings
    #[arg(long, default_value_t = false)]
    pub strict: bool,
}
```

- [ ] **Add `Commands::Validate(ValidateArgs)` to commands enum in `main.rs`**
- [ ] **Implement `run_validate()` in `validate.rs`** — dispatches to `redist_report::validate_rplan()`
- [ ] **Wire `Commands::Validate => run_validate(args)` in `main.rs` match arm**
- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(cli): redist validate --file plan.rplan dispatches to redist_report::validate_rplan()"`

---

## Task 7: New CLI flags on `redist state`/`redist states`/`redist run`

**Files:** `redist/crates/redist-cli/src/args.rs`, `redist/crates/redist-cli/src/main.rs`

- [ ] **L0: Write failing flag-parsing tests**

```rust
#[test]
fn test_districts_flag_parsed() {
    let args = StateArgs::try_parse_from(["redist", "state", "--state", "WA",
        "--year", "2020", "--version", "v1", "--districts", "98"]).unwrap();
    assert_eq!(args.districts, Some(98));
}

#[test]
fn test_chamber_flag_default_is_congressional() {
    let args = StateArgs::try_parse_from(["redist", "state", "--state", "WA",
        "--year", "2020", "--version", "v1"]).unwrap();
    assert_eq!(args.chamber, "congressional");
}

#[test]
fn test_label_flag_parsed() {
    let args = StateArgs::try_parse_from(["redist", "state", "--state", "WA",
        "--year", "2020", "--version", "v1", "--label", "wa_house_draft1"]).unwrap();
    assert_eq!(args.label.as_deref(), Some("wa_house_draft1"));
}

#[test]
fn test_balance_tolerance_flag_parsed() {
    let args = StateArgs::try_parse_from(["redist", "state", "--state", "WA",
        "--year", "2020", "--version", "v1", "--balance-tolerance", "5.0"]).unwrap();
    assert!((args.balance_tolerance.unwrap() - 5.0).abs() < 1e-9);
}

#[test]
fn test_population_source_flag_parsed() {
    let args = StateArgs::try_parse_from(["redist", "state", "--state", "WA",
        "--year", "2020", "--version", "v1", "--population-source", "vap"]).unwrap();
    assert_eq!(args.population_source, "vap");
}

#[test]
fn test_force_flag_default_false() {
    let args = StateArgs::try_parse_from(["redist", "state", "--state", "WA",
        "--year", "2020", "--version", "v1"]).unwrap();
    assert!(!args.force);
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Add new fields to `StateArgs`**

```rust
#[derive(Args)]
pub struct StateArgs {
    // ... existing fields ...
    /// Override district count (enables non-congressional chambers)
    #[arg(long)]
    pub districts: Option<usize>,
    /// Chamber type: congressional, house, senate, custom
    #[arg(long, default_value = "congressional")]
    pub chamber: String,
    /// Human label for this plan run (default: {state}_{chamber}_{year})
    #[arg(long)]
    pub label: Option<String>,
    /// Population source: total, vap, cvap
    #[arg(long, default_value = "total")]
    pub population_source: String,
    /// Max deviation per district in percent (default: 0.5 congressional, 5.0 state)
    #[arg(long)]
    pub balance_tolerance: Option<f64>,
    /// Write manifest.json alongside outputs
    #[arg(long, default_value_t = true)]
    pub manifest: bool,
    /// Overwrite existing plan without error
    #[arg(long, default_value_t = false)]
    pub force: bool,
}
```

- [ ] **Wire new args into `StateConfig` in `main.rs`**
- [ ] **Emit chamber-aware notice for state chambers**: `"State legislative balance tolerance set to 5.0%. Verify your state's constitutional standard."`
- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(cli): --districts, --chamber, --label, --population-source, --balance-tolerance, --force flags"`

---

## Task 8: RPLAN writer integration + manifest writing after plan output

**Files:** `redist/crates/redist-cli/src/main.rs` (output path), `redist/crates/redist-report/src/rplan.rs`

- [ ] **L0: Write failing integration tests**

```rust
#[test]
fn test_write_rplan_produces_valid_file() {
    let tmp = TempDir::new().unwrap();
    let assignments: HashMap<String, usize> = make_test_assignments(10);
    let metadata = RplanMetadata {
        label: "wa_test".into(),
        state_fips: "53".into(),
        state_code: "WA".into(),
        year: "2020".into(),
        chamber: "congressional".into(),
        num_districts: 10,
        population_source: "total".into(),
        balance_tolerance_pct: 0.5,
        created_at: "2026-04-26T00:00:00Z".into(),
        created_by: "redist test".into(),
        ..Default::default()
    };
    let path = tmp.path().join("plan.rplan");
    write_rplan(&path, &metadata, &assignments, None).unwrap();
    // Re-read and validate
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(validate_rplan_str(&content).is_ok());
}

#[test]
fn test_manifest_written_after_plan_output() {
    // After write_state_outputs succeeds, manifest.json must exist in plan dir
    let tmp = TempDir::new().unwrap();
    run_write_with_manifest(&tmp.path(), make_test_state_config()).unwrap();
    assert!(tmp.path().join("plans").join("wa_test").join("manifest.json").exists());
}

#[test]
fn test_manifest_sha256_is_deterministic() {
    let data = b"stable test content";
    let h1 = sha256_bytes(data);
    let h2 = sha256_bytes(data);
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 64);
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Implement `write_rplan()`** — serializes `RplanFile` to `.rplan` file
- [ ] **Implement manifest-writing step**: after `write_state_outputs()` succeeds, compute SHA-256 of adj and TIGER files, write `manifest.json` atomically
- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(report): RPLAN file writer + manifest written atomically after plan output"`

---

## Task 9: `redist migrate` command

**Files:** `redist/crates/redist-cli/src/migrate.rs`, `redist/crates/redist-cli/src/args.rs`, `redist/crates/redist-cli/src/main.rs`

- [ ] **L0: Write failing migrate tests**

```rust
#[test]
fn test_migrate_copies_final_assignments() {
    let tmp = TempDir::new().unwrap();
    // Set up legacy state dir
    let state_dir = tmp.path().join("states").join("washington");
    std::fs::create_dir_all(&state_dir).unwrap();
    std::fs::write(state_dir.join("final_assignments.json"),
        r#"{"530330001001":1}"#).unwrap();
    // Run migrate
    run_migrate(tmp.path(), "WA", "wa_congressional_2020").unwrap();
    // Verify plan dir exists with assignments
    let plan_dir = tmp.path().join("plans").join("wa_congressional_2020");
    assert!(plan_dir.join("final_assignments.json").exists());
}

#[test]
fn test_migrate_creates_minimal_manifest() {
    let tmp = TempDir::new().unwrap();
    setup_legacy_state_dir(tmp.path(), "washington");
    run_migrate(tmp.path(), "WA", "wa_congressional_2020").unwrap();
    let manifest_path = tmp.path()
        .join("plans").join("wa_congressional_2020").join("manifest.json");
    assert!(manifest_path.exists());
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(manifest_path).unwrap()
    ).unwrap();
    assert_eq!(manifest["label"].as_str().unwrap(), "wa_congressional_2020");
}

#[test]
fn test_migrate_nonexistent_state_errors() {
    let tmp = TempDir::new().unwrap();
    let result = run_migrate(tmp.path(), "ZZ", "zz_plan");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}
```

- [ ] **Run tests** — expect FAIL
- [ ] **Add `MigrateArgs` to `args.rs`**

```rust
#[derive(Args)]
pub struct MigrateArgs {
    /// Source state (e.g. WA)
    #[arg(long)]
    pub state: String,
    /// Target label for the new plan directory
    #[arg(long)]
    pub label: String,
    /// Version directory (e.g. v1)
    #[arg(long)]
    pub version: String,
    /// Census year
    #[arg(long, default_value = "2020")]
    pub year: String,
}
```

- [ ] **Add `Commands::Migrate(MigrateArgs)` to commands enum**
- [ ] **Implement `run_migrate()` in `migrate.rs`**: copy `states/{state}/` → `plans/{label}/`, write minimal manifest
- [ ] **Run tests** — expect PASS
- [ ] **Commit:** `git commit -m "feat(cli): redist migrate --state X --label Y copies legacy plan into plans/{label}/ tree"`

---

## Task 10: L2 Acceptance Tests

**Files:** `tests/acceptance/test_spec0_spec1_acceptance.py`

These require a real VT or DE run (small states, fast). Run with `pytest tests/acceptance/ -v`.

- [ ] **Write L2 acceptance tests**

```python
# tests/acceptance/test_spec0_spec1_acceptance.py
"""L2 acceptance tests for Spec 0 (RPLAN) + Spec 1 (Custom Parameters)."""

import json
import subprocess
import pytest
from pathlib import Path

# --- Spec 1 acceptance ---

def test_wa_house_98_districts_produces_manifest(tmp_redist_output):
    """redist state --state VT --districts 1 --chamber house produces manifest.json with correct fields."""
    # Use VT (1 district) with house chamber for fast acceptance run
    result = subprocess.run([
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec1_test",
        "--districts", "1", "--chamber", "house", "--label", "vt_house_test",
        "--output-dir", str(tmp_redist_output),
    ], capture_output=True, text=True)
    assert result.returncode == 0, result.stderr
    manifest_path = tmp_redist_output / "2020" / "plans" / "vt_house_test" / "manifest.json"
    assert manifest_path.exists(), f"manifest.json not found at {manifest_path}"
    manifest = json.loads(manifest_path.read_text())
    assert manifest["chamber"] == "house"
    assert manifest["num_districts"] == 1
    assert manifest["balance_tolerance_pct"] == pytest.approx(5.0)

def test_seed_produces_reproducible_assignments(tmp_redist_output):
    """Same seed run twice → identical final_assignments.json."""
    for run_i in range(2):
        subprocess.run([
            "redist", "state",
            "--state", "VT", "--year", "2020", "--version", "spec1_repro",
            "--label", f"vt_repro_run{run_i}", "--seed", "42",
            "--output-dir", str(tmp_redist_output), "--force",
        ], check=True)
    run0 = json.loads((tmp_redist_output / "2020" / "plans" / "vt_repro_run0" / "final_assignments.json").read_text())
    run1 = json.loads((tmp_redist_output / "2020" / "plans" / "vt_repro_run1" / "final_assignments.json").read_text())
    assert run0 == run1, "Reproducible seed must produce identical assignments"

def test_balance_tolerance_in_manifest(tmp_redist_output):
    """--balance-tolerance 2.0 is recorded in manifest."""
    subprocess.run([
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec1_tol",
        "--label", "vt_tol_test", "--balance-tolerance", "2.0",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    manifest = json.loads(
        (tmp_redist_output / "2020" / "plans" / "vt_tol_test" / "manifest.json").read_text()
    )
    assert manifest["balance_tolerance_pct"] == pytest.approx(2.0)

def test_population_source_in_manifest(tmp_redist_output):
    """--population-source total is recorded in manifest."""
    subprocess.run([
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec1_pop",
        "--label", "vt_pop_test", "--population-source", "total",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    manifest = json.loads(
        (tmp_redist_output / "2020" / "plans" / "vt_pop_test" / "manifest.json").read_text()
    )
    assert manifest["population_source"] == "total"

def test_label_collision_exits_nonzero_without_force(tmp_redist_output):
    """Running twice with same default label fails on second run without --force."""
    common_args = [
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec1_collision",
        "--label", "vt_collision_test",
        "--output-dir", str(tmp_redist_output),
    ]
    subprocess.run(common_args, check=True)
    result = subprocess.run(common_args, capture_output=True, text=True)
    assert result.returncode != 0, "Second run without --force must fail"
    assert "already exists" in result.stderr or "already exists" in result.stdout

def test_force_flag_allows_second_run(tmp_redist_output):
    """--force on second run succeeds and overwrites."""
    common_args = [
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec1_force",
        "--label", "vt_force_test",
        "--output-dir", str(tmp_redist_output),
    ]
    subprocess.run(common_args, check=True)
    result = subprocess.run(common_args + ["--force"], capture_output=True, text=True)
    assert result.returncode == 0, result.stderr

# --- Spec 0 acceptance ---

def test_rplan_all_assignments_11char_geoids(tmp_redist_output):
    """Every GEOID key in exported RPLAN is exactly 11 numeric characters."""
    subprocess.run([
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec0_test",
        "--label", "vt_rplan_test", "--output-format", "rplan",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    rplan_path = tmp_redist_output / "2020" / "plans" / "vt_rplan_test" / "vt_rplan_test.rplan"
    rplan = json.loads(rplan_path.read_text())
    for geoid in rplan["assignments"]:
        assert len(geoid) == 11, f"GEOID {geoid!r} must be 11 chars"
        assert geoid.isdigit(), f"GEOID {geoid!r} must be numeric"

def test_rplan_version_top_level_only(tmp_redist_output):
    """rplan_version must appear at root level only, not inside metadata."""
    subprocess.run([
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec0_ver",
        "--label", "vt_ver_test", "--output-format", "rplan",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    rplan_path = tmp_redist_output / "2020" / "plans" / "vt_ver_test" / "vt_ver_test.rplan"
    rplan = json.loads(rplan_path.read_text())
    assert "rplan_version" in rplan
    assert "rplan_version" not in rplan.get("metadata", {}), \
        "rplan_version must NOT appear inside metadata"

def test_redist_validate_passes_for_generated_rplan(tmp_redist_output):
    """redist validate --file plan.rplan exits 0 for a freshly generated plan."""
    subprocess.run([
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec0_val",
        "--label", "vt_val_test", "--output-format", "rplan",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    rplan_path = tmp_redist_output / "2020" / "plans" / "vt_val_test" / "vt_val_test.rplan"
    result = subprocess.run(
        ["redist", "validate", "--file", str(rplan_path)],
        capture_output=True, text=True
    )
    assert result.returncode == 0, result.stderr
    assert "PASS" in result.stdout

def test_manifest_tiger_sha256_is_64_hex_chars(tmp_redist_output):
    """manifest.json tiger_sha256 is a 64-character hex string."""
    subprocess.run([
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec0_sha",
        "--label", "vt_sha_test",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    manifest = json.loads(
        (tmp_redist_output / "2020" / "plans" / "vt_sha_test" / "manifest.json").read_text()
    )
    assert "tiger_sha256" in manifest
    assert len(manifest["tiger_sha256"]) == 64
    assert all(c in "0123456789abcdef" for c in manifest["tiger_sha256"])

def test_manifest_adjacency_file_is_filename_not_path(tmp_redist_output):
    """manifest.json adjacency_file is a bare filename, not a local path."""
    subprocess.run([
        "redist", "state",
        "--state", "VT", "--year", "2020", "--version", "spec0_adj",
        "--label", "vt_adj_test",
        "--output-dir", str(tmp_redist_output),
    ], check=True)
    manifest = json.loads(
        (tmp_redist_output / "2020" / "plans" / "vt_adj_test" / "manifest.json").read_text()
    )
    adj = manifest["adjacency_file"]
    assert "/" not in adj and "\\" not in adj, \
        f"adjacency_file must be filename only, got: {adj!r}"
    assert "tiger_source_url" in manifest
    assert "census.gov" in manifest["tiger_source_url"]
```

- [ ] **Run L2 tests** — expect FAIL (features not yet wired end-to-end)
- [ ] **Wire all components end-to-end in CLI**: plans dir, manifest, RPLAN export, validate command
- [ ] **Run L2 tests** — expect PASS
- [ ] **Commit:** `git commit -m "test(acceptance): L2 tests for Spec 0 RPLAN + Spec 1 custom parameters"`

---

## L0 Tests from Scenario Simulations

The following test assertions come directly from the scenario simulation document.

### Scenario 1: WA house redistricting (chamber-aware tolerance + RPLAN structure)

```rust
// tests/unit/test_spec0_spec1.rs

#[test]
fn test_wa_house_manifest_chamber_aware_tolerance() {
    // StateConfig with chamber="house" → effective_balance_tolerance = 0.05 (5%)
    let cfg = StateConfig { chamber: "house".into(), balance_tolerance: None, ..Default::default() };
    assert_eq!(cfg.effective_balance_tolerance(), 0.05);
}

#[test]
fn test_rplan_version_top_level_only() {
    // Serialized RPLAN must have rplan_version only at root, not in metadata
    let plan = make_test_rplan_file();
    let parsed: serde_json::Value = serde_json::to_value(&plan).unwrap();
    assert!(parsed["rplan_version"].is_string());
    assert!(parsed["metadata"].get("rplan_version").is_none(),
        "rplan_version must NOT appear inside metadata");
}
```

### Scenario 2: Court submission audit trail

```python
# tests/unit/test_manifest.py

def test_manifest_has_tiger_sha256():
    manifest = load_manifest_fixture("wa_house_draft1")
    assert "tiger_sha256" in manifest
    assert len(manifest["tiger_sha256"]) == 64  # SHA-256 hex

def test_manifest_has_binary_download_url():
    manifest = load_manifest_fixture("wa_house_draft1")
    assert "binary_download_url" in manifest
    assert "github.com" in manifest["binary_download_url"]

def test_verification_command_no_local_paths():
    report = generate_audit_report_fixture("wa_house_draft1")
    cmd = report["audit"]["verification_command"]
    assert "C:\\" not in cmd   # no Windows local paths
    assert "/home/" not in cmd  # no Unix local paths
    assert "--seed" in cmd      # seed present for reproducibility
```

### Scenario 6: Label collision protection

```rust
// (already included in Task 5 above — reproduced here for traceability)

#[test]
fn test_label_collision_exits_nonzero_without_force() {
    let tmp = TempDir::new().unwrap();
    let plan_dir = tmp.path().join("plans").join("washington_house_2020");
    std::fs::create_dir_all(&plan_dir).unwrap();
    std::fs::write(plan_dir.join("manifest.json"),
        r#"{"created_at":"2026-04-26T14:23:00Z"}"#).unwrap();
    let result = check_plan_collision(&plan_dir, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
    assert!(result.unwrap_err().to_string().contains("2026-04-26"));
}

#[test]
fn test_label_force_flag_allows_overwrite() {
    let tmp = TempDir::new().unwrap();
    let plan_dir = tmp.path().join("plans").join("wa_house");
    std::fs::create_dir_all(&plan_dir).unwrap();
    std::fs::write(plan_dir.join("manifest.json"), b"{}").unwrap();
    assert!(check_plan_collision(&plan_dir, true).is_ok());
}
```

---

## Execution Order

1. Task 1 — `redist-report` crate + RPLAN writer (no deps)
2. Task 2 — RPLAN validator (depends on Task 1)
3. Task 3 — `PlanManifest` + SHA-256 (no deps outside report crate)
4. Task 4 — `StateConfig` additions + balance/population (redist-core)
5. Task 5 — Label collision + paths (depends on Tasks 3+4)
6. Task 6 — `redist validate` command (depends on Tasks 1+2)
7. Task 7 — New CLI flags (depends on Task 4)
8. Task 8 — RPLAN writer integration + manifest writing (depends on Tasks 1+3+5+7)
9. Task 9 — `redist migrate` command (depends on Task 5)
10. Task 10 — L2 acceptance tests (depends on all above)

Tasks 1+3+4 can run in parallel. Tasks 2 and 5 and 6 each depend on one prior task only, so the critical path is 1→2→6 and 3+4→5→7→8→9→10.
