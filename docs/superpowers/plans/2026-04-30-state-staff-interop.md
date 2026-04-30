# Plan: State Staff Interop — Districtr / DRA / Shapefile Round-Trip

> **For agentic workers:** Do not execute until the spec at `docs/superpowers/specs/2026-04-30-state-staff-interop.md` v2 (with v2.1 patches) has been reviewed and approved. Steps use checkbox (`- [ ]`) syntax for tracking.

**Date:** 2026-04-30
**Spec:** `docs/superpowers/specs/2026-04-30-state-staff-interop.md`
**v2.1 tracking ref:** `docs/superpowers/specs/2026-04-30-v21-tracking.md`
**Goal:** Make the redist CLI a first-class backend for Districtr, Dave's Redistricting App (DRA), and QGIS, so a state legislative staffer's loop is "draw in Districtr → `redist import` → `redist analyze` → `redist report` → iterate," with a parallel civic-bypass entry point for community-submitted counter-proposals.

**Depends on:** `redist-report` import infrastructure (`import_geojson_plan`, `import_gerrychain_to_assignments`), the existing partial `redist import --format districtr` shell, `write_manifest_atomic` already in `redist-report::manifest`, and the Callais p.36 mutex centralised in `runner.rs::validate_partisan_config`.
**Blocks:** quickstart-state-staff.md (Onboarding plan Task 5.4 cross-references the artifacts delivered here); Plan Comparison and Civic Bidirectional pipelines need civic-bypass-tagged plans to flow through.

**v2.1 items addressed by this plan:**
- B-05 (hermetic Districtr fixture under `tests/fixtures/districtr/` instead of CDN-fetch)
- PP-33 (schema-version handshake fingerprint collision — multi-attribute fingerprint)
- C-05 (import manifest captures `import_compat_sha256` of the active version-pin lookup)
- M-01 (apply `--label` → `--plan-label` rename in import/export examples; keep `--label` as a deprecated alias for one release)

---

## Pre-Conditions

- `redist-cli` builds clean from `cargo build --release --locked` on Windows + Linux.
- `redist-report::manifest` exposes `write_manifest_atomic` and `check_incomplete_plan` (already in tree).
- `redist-core::validate_partisan_config` (in `runner.rs`) enforces the Callais p.36 mutex at the `redist state` gate; this plan extends the same guard to `redist import` and `redist analyze`.
- A small synthetic Vermont-shape adjacency + tract-centroid fixture exists for L0/L1 tests (already used by Onboarding plan Task 1; reuse, do not duplicate).

---

## Task 1: Atomic-import infrastructure (PP-22, spec §7)

**Files:** `redist/crates/redist-cli/src/import_cmd.rs`, `redist/crates/redist-report/src/manifest.rs` (extend), `redist/crates/redist-cli/src/io_utils.rs`

The current `run_import` writes `final_assignments.json` directly into `plan_dir`, then writes `manifest.json` via `write_manifest_atomic`. A failure between those two writes leaves a half-imported plan visible to subsequent commands. Tasks 2–4 add new formats; doing them on the current foundation would multiply that race surface. Atomicity must come first.

- [ ] **1.1** Add `redist_report::manifest::atomic_plan_dir(plan_root: &Path) -> Result<PlanDirGuard>`. The guard owns a sibling `{label}.tmp/` directory; on `commit()` it renames to `{label}/`; on drop without commit, it deletes the tmp directory. Implement on Windows with `std::fs::rename` (works for empty *destination*, fails if destination exists — handle by writing into `{label}.tmp/` then renaming once after destination check).
- [ ] **1.2** Refactor `run_import` to: (a) build the plan into `plan_root/{label}.tmp/`; (b) run every validation (GEOID format, population sum, contiguity warning, schema-version handshake, Callais preflight); (c) only call `guard.commit()` after all validations pass. On any error, the guard's `Drop` removes the tmp directory and no `{label}/` exists.
- [ ] **1.3** Failure path: on validation failure, the error message must name the validation that failed (spec §7) and list the tmp directory that was cleaned up. Exit non-zero.
- [ ] **1.4** L0 test `test_atomic_import_failure_leaves_no_label`: inject a forced contiguity validation failure mid-import; assert `plan_root/{label}/` does not exist, `plan_root/{label}.tmp/` does not exist, and exit code is non-zero.
- [ ] **1.5** L0 test `test_atomic_import_label_collision`: pre-create `plan_root/{label}/`; assert import refuses to overwrite without `--force` and the existing directory is untouched.

**Exit:** A failing import never produces a visible plan label; the existing happy-path tests in `import_cmd.rs::tests` still pass against the refactored guard.

---

## Task 2: Districtr import + export (spec §1, §2)

**Files:** `redist/crates/redist-report/src/districtr.rs` (new), `redist/crates/redist-cli/src/args.rs`, `redist/crates/redist-cli/src/import_cmd.rs`, `redist/crates/redist-cli/src/export_cmd.rs`

- [ ] **2.1** Add `ExportFormat::Districtr` and accept `"districtr"` in `ImportArgs::format`. Document the new variant in `args.rs` doc strings.
- [ ] **2.2** Implement `parse_districtr_plan(json: &str) -> Result<DistrictrPlan>` covering Districtr's three assignment shapes: tract-level (`assignment` keyed by `GEOID`), block-level (`assignment` keyed by `BLOCKID`), precinct-level (`assignment` keyed by precinct ID). Auto-detect by inspecting the first key length (tract=11, block=15, otherwise precinct).
- [ ] **2.3** Implement GEOID translation:
    - Tract-level → 1:1 (validate length, leading-zero preservation).
    - Block-level → aggregate via Census Block Assignment File (BAF) crosswalk; majority-vote per tract; record split blocks in `translation_log.txt`.
    - Precinct-level → out of scope for v1; emit a `[INPUT]` error referencing the spec's "user provides precinct shapefiles" prerequisite. Document the wire-up surface for a follow-up.
- [ ] **2.4** Implement `export_districtr_plan(rplan: &RplanFile) -> Result<String>` producing a Districtr-loadable JSON. Schema target: Districtr's current public plan-export shape (top-level `id`, `placeId`, `problem`, `assignment` map, `units`).
- [ ] **2.5** Wire dispatch in `run_import` (`format == "districtr"`) and `run_export` (`ExportFormat::Districtr`). Reuse `write_rplan` and the atomic-import guard from Task 1.
- [ ] **2.6** Cache strategy for block→tract crosswalks (spec risk row): store under `~/.cache/redist/baf/{year}/{state_fips}.csv` on Linux/macOS, `%LOCALAPPDATA%\redist\baf\{year}\{state_fips}.csv` on Windows. Fetch on demand from Census-published BAF; SHA-256-pinned in `import_compat.json` (Task 5). Document in `docs/file-formats/districtr.md` (new file written by Task 9).
- [ ] **2.7** L0 test `test_districtr_tract_import_round_trips`: a 5-tract synthetic Districtr JSON imports to assignments matching a hand-written expected map.
- [ ] **2.8** L0 test `test_districtr_block_import_majority_vote`: a synthetic plan with 3 blocks per tract (2:1 split) assigns each tract to the majority district and logs the split in `translation_log.txt`.

**Exit:** `redist import --format districtr <PATH>` produces a plan label that `redist analyze` can consume; `redist export --format districtr` produces JSON whose top-level keys match Districtr's expected shape.

---

## Task 3: DRA import + export (spec §3)

**Files:** `redist/crates/redist-report/src/dra.rs` (new), `redist/crates/redist-cli/src/import_cmd.rs`, `redist/crates/redist-cli/src/export_cmd.rs`

The existing `import_assignments_from_dra_csv` in `import_cmd.rs` already handles the GEOID-first, DISTRICT-first, and headerless variants. This task formalises it as a named format, adds export, and wires schema-version fingerprinting (Task 5).

- [ ] **3.1** Promote `import_assignments_from_dra_csv` from `import_cmd.rs` into `redist_report::dra::parse_dra_csv`, returning `(assignments, fingerprint)`. The fingerprint is the column-set string (e.g., `"GEOID,DISTRICT"` or `"DISTRICT,GEOID"` or `"<no-header>"`) — consumed by the schema-version handshake.
- [ ] **3.2** Add `ExportFormat::Dra`. Implementation: emit `GEOID,DISTRICT\n` header followed by sorted `(GEOID, district)` rows. Sort order is GEOID-ascending (deterministic for round-trip equality).
- [ ] **3.3** Wire `--format dra` (in addition to the existing `--format dra-csv` alias) in both `ImportArgs` and `ExportFormat`. Keep `dra-csv` as a `clap` alias for one release per the M-01 deprecation policy.
- [ ] **3.4** L0 test `test_dra_round_trip_preserves_assignments`: build a 7-tract assignment map; export to DRA CSV; reimport; assert canonical-form equality (Task 8 defines the equality function — this test calls it).
- [ ] **3.5** L0 test `test_dra_import_records_fingerprint`: import each of the three DRA variants; assert the manifest's `source_format_fingerprint` field matches the expected variant string.

**Exit:** DRA round-trip on the synthetic Vermont fixture passes; the column-set fingerprint reaches the import manifest.

---

## Task 4: Shapefile + GeoJSON import + export (spec §4)

**Files:** `redist/crates/redist-cli/src/import_cmd.rs`, `redist/crates/redist-cli/src/export_cmd.rs`, `redist/crates/redist-report/src/shapefile.rs` (new)

Today, `.shp`/`.shx`/`.dbf`/`.zip` extensions hit the `is_shapefile_extension` guard with an `ogr2ogr` guidance error. The spec requires native shapefile-with-`district`-column import + export so QGIS works as an external test client.

- [ ] **4.1** Add the `shapefile` crate dependency to `redist-report/Cargo.toml` (read-only — DBF + SHP). No GDAL link.
- [ ] **4.2** Implement `parse_shapefile_assignments(shp_path: &Path) -> Result<HashMap<GEOID, district>>`: open the `.shp`+`.dbf` pair, read each record's `GEOID` (or `GEOID20`/`GEOID10` fallback) and `district` attribute, return the assignment map. Reject if either column is missing with a `[INPUT]` error naming the missing column.
- [ ] **4.3** Implement `export_shapefile_assignments` as **out of scope for v1** — emit the existing ogr2ogr guidance for the export side, but make it specific: "redist does not write shapefiles natively; use `redist export --format geojson` then `ogr2ogr -f \"ESRI Shapefile\" out.shp out.geojson`." Rationale: writing valid shapefiles requires geometry, which `redist export` doesn't always have (see `warn_if_null_geometry`); GeoJSON-then-ogr2ogr is the supported path. Document this trade-off in `docs/file-formats/state-staff-interop.md` (Task 9).
- [ ] **4.4** Remove the `is_shapefile_extension` early-bail for `.shp` and `.shx` only when `--format shapefile` is explicit; preserve the guidance error when the extension is encountered without an explicit format flag (avoids surprising users who pass a `.zip` they expected us to extract). `.zip` keeps the existing guidance unconditionally.
- [ ] **4.5** Promote the existing `import_assignments_from_geojson_properties` into a named format (`--format geojson`); this is already wired as the default but is currently auto-detected — make it explicit and add `ExportFormat::GeoJson` as the symmetric export path (already exists in `export_cmd.rs`; just verify the round-trip).
- [ ] **4.6** L0 test `test_shapefile_import_with_district_column`: a tiny synthetic 3-feature shapefile (committed under `tests/fixtures/shapefile/`) imports to the expected assignment map.
- [ ] **4.7** L0 test `test_shapefile_missing_district_column_errors_actionably`: a shapefile without a `district` column produces an `[INPUT]` error naming the missing column.

**Exit:** QGIS users can save a layer with a `district` column and import it with `redist import --format shapefile`; the export path goes through GeoJSON+ogr2ogr with documented guidance.

---

## Task 5: Schema-version handshake + `import_compat.json` (spec §8, PP-33, C-05)

**Files:** `redist/crates/redist-cli/src/import_compat.json` (new), `redist/crates/redist-report/src/import_compat.rs` (new), `redist/crates/redist-cli/src/import_cmd.rs`

The handshake is what keeps state staff from silently feeding us a Districtr build whose JSON shape we've never tested against. PP-33 surfaces the trap: a single schema-version string can collide across forks; we need a multi-attribute fingerprint. C-05 wants the manifest to record the SHA-256 of the version-pin file itself so a court can verify which compat ranges were active at import time.

- [ ] **5.1** Author `redist/crates/redist-cli/src/import_compat.json` with this schema:
    ```json
    {
      "schema_version": "import-compat v1",
      "districtr": {
        "tested_versions": ["2024.10", "2025.02", "2025.06"],
        "fingerprint_keys": ["schema_version", "problem.type", "units.name"],
        "known_fingerprints": {
          "districtr-2025.06-tract": {
            "schema_version": "1.2",
            "problem.type": "districts",
            "units.name": "tracts20"
          }
        }
      },
      "dra": {
        "tested_column_sets": ["GEOID,DISTRICT", "DISTRICT,GEOID", "<no-header>"]
      },
      "baf_crosswalk": {
        "url_template": "https://www2.census.gov/geo/docs/maps-data/data/baf{year}/BlockAssign_ST{fips}_*.zip",
        "sha256_by_state_year": { "50_2020": "<pinned-sha>" }
      }
    }
    ```
- [ ] **5.2** Compile-time-embed via `include_str!`. Add `redist_report::import_compat::compat_sha256()` returning the SHA-256 of the embedded JSON bytes — written to every import manifest as `import_compat_sha256` (C-05).
- [ ] **5.3** Build the **multi-attribute fingerprint** for Districtr: extract `(schema_version, problem.type, units.name)` from the imported JSON; compare to `known_fingerprints` first (exact match → no warning), fall back to `tested_versions` range check (PP-33 — single-string collision is impossible because we match three attributes jointly). On out-of-range, emit a warning naming the supported version range and the fingerprint actually seen.
- [ ] **5.4** Build the **column-set fingerprint** for DRA: the variant string from Task 3.1; warn if outside `tested_column_sets`.
- [ ] **5.5** Extend `PlanManifest` with three optional fields (default-Some empty; serde-renamed for stability): `source_tool`, `source_tool_version`, `source_format_fingerprint`, `import_compat_sha256`. Existing manifests deserialise without breakage because each field is `#[serde(default)]`.
- [ ] **5.6** L0 test `test_districtr_handshake_warns_out_of_range`: imported plan with `schema_version: "9.9"` outside the pinned range produces a warning naming the supported range. Exit code remains 0 (warning, not error).
- [ ] **5.7** L0 test `test_import_compat_sha256_recorded_in_manifest`: the manifest's `import_compat_sha256` matches `sha256_bytes(include_str!("import_compat.json").as_bytes())`.
- [ ] **5.8** L0 test `test_pp33_fingerprint_collision_resistance`: two synthetic Districtr JSONs with identical `schema_version` but different `problem.type` produce different fingerprints; the manifest distinguishes them.

**Exit:** Every import manifest pins `import_compat_sha256`; PP-33 collision is impossible because the fingerprint is multi-attribute.

---

## Task 6: Callais p.36 mutex preflight at import + analyze (spec §9)

**Files:** `redist/crates/redist-cli/src/import_cmd.rs`, `redist/crates/redist-cli/src/analyze.rs`, `redist/crates/redist-cli/src/runner.rs` (extract preflight)

The mutex is enforced at `redist state` via `validate_partisan_config(cfg)`. State staff who import a plan from Districtr never touch `redist state` — so they can skip the guard. This task fires the same check at every entry point that consumes a plan.

- [ ] **6.1** Extract `redist_report::manifest::callais_preflight(manifest: &PlanManifest) -> Result<()>` from `runner.rs::validate_partisan_config`. The function inspects: (a) `partition_mode`; (b) any `vra_aware` marker (e.g., `population_source == "cvap"` combined with `metis-vra`); (c) any `partisan_weighted` marker (`partition_mode == "partisan-weighted"` or non-empty partisan-shares fingerprint). If both classes appear, return the same error string the state-runner uses ("partisan-weighted and metis-vra are mutually exclusive per Callais p.36 disentanglement").
- [ ] **6.2** Call `callais_preflight(&manifest)` in `run_import` after the manifest is built but BEFORE the atomic-guard `commit()`. Failure deletes the tmp directory and exits non-zero with the named error.
- [ ] **6.3** Call `callais_preflight(&manifest)` at the top of `run_analyze`, after the manifest is loaded but before any analyzer runs. Failure exits non-zero before any output is touched.
- [ ] **6.4** L0 test `test_callais_preflight_blocks_import`: an imported manifest with both VRA + partisan markers exits non-zero with the named error; no plan label is created (relies on Task 1's atomic guard).
- [ ] **6.5** L0 test `test_callais_preflight_blocks_analyze`: with a pre-existing tampered manifest (both markers), `redist analyze --label X` exits non-zero before any `analysis/*.json` is written.
- [ ] **6.6** L0 test `test_callais_preflight_passes_clean_manifest`: a manifest with only `partition_mode == "partisan-weighted"` (no VRA marker) passes; a manifest with only `metis-vra` passes.

**Exit:** Callais disentanglement holds at three gates (`redist state`, `redist import`, `redist analyze`) and is centralised in one function.

---

## Task 7: Civic-bypass `--as-civic-counter-proposal` flag (spec §10, COMMONS)

**Files:** `redist/crates/redist-cli/src/args.rs`, `redist/crates/redist-cli/src/import_cmd.rs`, `redist/crates/redist-report/src/manifest.rs`

State staff is the authoritative-mapping path; civic groups need a parallel entry that doesn't pretend to be authoritative but also isn't second-class for downstream analysis.

- [ ] **7.1** Add three flags to `ImportArgs`: `--as-civic-counter-proposal` (bool), `--submitted-by` (String), `--submitted-at` (Option<String>; default = current ISO-8601 UTC).
- [ ] **7.2** Extend `PlanManifest` with `submission_type: String` (default `"authoritative"`), `submitted_by: Option<String>`, `submitted_at: Option<String>`. When `--as-civic-counter-proposal` is set, write `submission_type = "civic_counter_proposal"`.
- [ ] **7.3** Suppress the "is this from an authoritative state mapping tool?" warning when `submission_type == "civic_counter_proposal"`. (The warning itself is added in this task — it fires when we cannot identify the source tool from the fingerprint and `--as-civic-counter-proposal` is not set.)
- [ ] **7.4** Validate that `--as-civic-counter-proposal` requires `--submitted-by` to be non-empty; otherwise `[INPUT]` error.
- [ ] **7.5** Plan-Comparison and Civic-Bidirectional cross-wiring is OUT OF SCOPE for this plan but the manifest field is what those plans key on. Document the contract in `docs/file-formats/state-staff-interop.md` (Task 9) so the downstream plans can rely on it.
- [ ] **7.6** L0 test `test_civic_bypass_records_submitter`: import with `--as-civic-counter-proposal --submitted-by "League of Women Voters"`; assert manifest has `submission_type == "civic_counter_proposal"`, `submitted_by == "League of Women Voters"`, `submitted_at` is a valid ISO-8601 string.
- [ ] **7.7** L0 test `test_civic_bypass_requires_submitter`: `--as-civic-counter-proposal` without `--submitted-by` exits non-zero with `[INPUT]` category.

**Exit:** Civic counter-proposals are tagged at import; the manifest contract is fixed for the downstream Plan Comparison + Civic Bidirectional plans.

---

## Task 8: Round-trip property tests with canonical-form equality (spec §6)

**Files:** `redist/crates/redist-report/src/canonical.rs` (new), `redist/crates/redist-cli/src/import_cmd.rs::tests`, `redist/crates/redist-report/src/districtr.rs::tests`, `redist/crates/redist-report/src/dra.rs::tests`, `tests/fixtures/districtr/` (new — B-05)

Spec §6 fixes the equality definition: `T(redist(T(P))) == P` after district-label canonicalisation (re-number districts in increasing order of their lowest-GEOID member). This task delivers (a) the canonicalisation function, (b) a property-test harness, (c) the hermetic Districtr fixture (B-05).

- [ ] **8.1** Implement `redist_report::canonical::canonicalize_assignments(a: &HashMap<GEOID, usize>) -> BTreeMap<GEOID, usize>`: re-number districts by ascending min-GEOID; tie-break is impossible because GEOIDs are unique. Return BTreeMap so equality is order-independent.
- [ ] **8.2** Implement `assert_canonical_equal(a, b)` returning a structured diff (which GEOIDs disagree, which districts permuted) for use in test failure messages.
- [ ] **8.3** **B-05**: commit a hermetic Districtr fixture under `tests/fixtures/districtr/`:
    - `vermont-tract-5-districts.json` (~5 KB) — synthetic plan over the Vermont 193-tract topology, hand-built. NOT fetched from a CDN; lives in-repo. Document the regeneration procedure in `tests/fixtures/districtr/README.md` (one Python script, deterministic).
    - `vermont-block-3districts.json` — synthetic block-level plan that exercises the BAF crosswalk with a known split block.
    - `dra-vermont-7-tract.csv` (lives under `tests/fixtures/dra/` for symmetry).
- [ ] **8.4** L0 test `test_districtr_round_trip_canonical_equal`: load fixture; import; export; reimport from the export; assert `canonicalize_assignments` of original and final are equal. Run with intentionally-permuted district labels in the original to prove canonicalisation works.
- [ ] **8.5** L0 test `test_dra_round_trip_canonical_equal`: same shape, against the DRA fixture.
- [ ] **8.6** L0 test `test_canonicalize_handles_label_permutation`: build two assignments differing only in a `1↔2` district-label swap; assert canonicalisation makes them equal.
- [ ] **8.7** L1 test `tests/integration/test_round_trip_vermont.rs` (or pytest equivalent under `redist/tests/`): drives the full CLI via `Command::new("redist")` against the in-repo fixtures; asserts canonical equality and exit code 0.

**Exit:** Both round-trips pass against committed fixtures; no test depends on a CDN fetch (B-05).

---

## Task 9: Docs + `quickstart-state-staff.md` content delivery

**Files:** `docs/quickstart/quickstart-state-staff.md` (new — claimed by Onboarding plan Task 5.4 but content lives here), `docs/file-formats/state-staff-interop.md` (new), `docs/file-formats/districtr.md` (new), `docs/REDIST_CLI.md` (extend), `docs/CHANGELOG.md`

The Onboarding plan creates the file at Task 5.4; this plan supplies the actual Districtr → redist → PDF walkthrough that lives inside it.

- [ ] **9.1** `quickstart-state-staff.md` follows the spec's persona-template (Who you are / What you'll have / Time / Steps / Expected output / Where to go next). Steps:
    1. Draw a 5-district map in Districtr against Vermont 2020 tracts; "Save Plan" → JSON.
    2. `redist import --format districtr senate_v3.json --plan-label vt_senate_v3 --state VT`.
    3. `redist analyze --plan-label vt_senate_v3 --types all`.
    4. `redist report --plan-label vt_senate_v3 --format pdf`.
    5. Iterate in Districtr; bump the label suffix.
- [ ] **9.2** Annotate each step with expected wall-clock time (step 2 ≤ 3s, step 3 ≤ 30s on Vermont, step 4 ≤ 10s).
- [ ] **9.3** Document the civic-bypass variant: append a short section "If you're a civic group, not state staff: add `--as-civic-counter-proposal --submitted-by '<group>'` to step 2." Cross-reference Plan Comparison + Civic Bidirectional plans by spec path.
- [ ] **9.4** `docs/file-formats/state-staff-interop.md`: documents the four formats (Districtr, DRA, GeoJSON, shapefile) and the expected schema fields for each. Cross-references `import_compat.json`. Includes the manifest fingerprint contract relied on by Tasks 7.5.
- [ ] **9.5** `docs/file-formats/districtr.md`: documents the Districtr JSON shapes we accept, the fingerprint keys, the BAF crosswalk path, and how to bump `tested_versions` when a new Districtr release ships (the one-line bump promised in spec §8).
- [ ] **9.6** Extend `docs/REDIST_CLI.md`: add `--format districtr|dra|shapefile` to the import + export sections; add `--as-civic-counter-proposal`, `--submitted-by`, `--submitted-at` to the import section.
- [ ] **9.7** `docs/CHANGELOG.md` entry naming the new formats, the civic-bypass flag, and the schema-version handshake.
- [ ] **9.8** Manually walk the quickstart end-to-end on a clean Windows VM and a clean Linux container. Record actual wall-clock at the top of the file. The spec's DoD line "A real state staffer (not us) gets through the quickstart in under 15 minutes" is a separate UAT step gated outside this plan; this task delivers the artifact under that bar.

**Exit:** `quickstart-state-staff.md` walks end-to-end; the two file-format docs are referenced from REDIST_CLI.md and the spec; CHANGELOG.md notes the new surface.

---

## Definition of Done

- `redist import --format districtr`, `--format dra`, `--format shapefile`, `--format geojson` all produce a plan label that `redist analyze` consumes
- `redist export --format districtr` produces JSON loadable in Districtr's web app (manual UAT against a real Districtr instance)
- DRA round-trip + Districtr round-trip both pass canonical-form equality on committed in-repo fixtures (B-05 — no CDN fetch)
- Atomic-import guard: a forced validation failure leaves no `{label}/` directory and no `{label}.tmp/` directory; tested
- Callais p.36 preflight fires at `redist import` AND `redist analyze`; tested at both gates
- Schema-version handshake uses a multi-attribute fingerprint (PP-33); manifest pins `import_compat_sha256` (C-05); both tested
- `--as-civic-counter-proposal --submitted-by <X>` produces a manifest with `submission_type=civic_counter_proposal` and the submitter recorded; tested
- `quickstart-state-staff.md` walks end-to-end; `docs/file-formats/state-staff-interop.md` and `docs/file-formats/districtr.md` exist
- All five quickstart artifacts reachable from `docs/REDIST_CLI.md` and `README.md`

---

## Risks

| Risk | Mitigation |
|---|---|
| Districtr's JSON shape changes between pinning and shipping | Multi-attribute fingerprint (PP-33) catches divergence; `import_compat.json` is a one-line bump (spec §8); `import_compat_sha256` in the manifest tells future readers exactly which compat table was active |
| BAF crosswalk fetch hits a Census URL change | SHA-256 pin in `import_compat.json`; failure surfaces a clear `[NETWORK]` error naming the pin, not a silent stale hit |
| `shapefile` crate read path doesn't tolerate every state's TIGER variant | Test against committed Vermont fixture first; document QGIS-export-then-import as the supported producer (not raw TIGER) in Task 9.4 |
| Atomic-rename semantics differ on Windows vs POSIX | Use `std::fs::rename` over an empty destination; explicit pre-check for destination existence before rename; tested on both platforms in CI |
| Civic-bypass becomes a back-door past authoritative-import warnings | Manifest tag is loud; downstream Plan Comparison + Civic Bidirectional plans surface `submission_type` in every comparison summary card per spec §10 |
| Round-trip equality test passes by accident (vacuous mock) | Test 8.6 explicitly proves canonicalisation eats label permutations; Test 8.4 uses an *intentionally permuted* original to defeat false positives |
| Block-level Districtr import drops a split block silently | `translation_log.txt` records every split; Task 2.8 asserts the log line is present |

---

## Out of Scope

- Native shapefile *write* (export goes via GeoJSON + ogr2ogr per Task 4.3)
- Precinct-level Districtr import (Task 2.3 documents the wire-up surface for follow-up)
- Real-time collaborative editing (Districtr's domain)
- A Districtr CDN-fetch L2 test (B-05 explicitly replaces this with a hermetic fixture)
- Bidirectional civic-counter-proposal flow into Plan Comparison + Civic Bidirectional pipelines (this plan delivers the manifest contract; consumption lives in those plans)
- DRA schema-version *bumps* — we record the column-set fingerprint; if DRA ships a new column set, that's a `tested_column_sets` bump in `import_compat.json` (one-line)
