# Final Rust + Python Architecture
**Date:** 2026-04-29
**Updated:** 2026-04-29 (v2 — incorporates 7-role review findings)
**Status:** Authoritative target — supersedes design/rust-port/README.md (Phase-0 planning)
**Author:** Gio Della-Libera

## Principle

The `redist` Rust CLI is the production interface. Python is retained only for research, ad-hoc analysis, and HTML dashboard templating — work that does not need to ship and does not need to be fast.

Phases 0-5 of the migration are complete (per `design/rust-port/migration-log.md`). Remaining work is **cutover and cleanup**, not new development. This spec defines the steady state, the deletion list, and the artifacts that must be preserved before deletion.

## Implementation Status

| Component | Status |
|---|---|
| Rust algorithm kernel (redist-core) | ✅ Complete |
| Rust data layer (redist-data) | ✅ Complete |
| Rust analysis (redist-analysis) | ✅ Complete |
| Rust CLI subcommands | ✅ Complete (`state`, `states`, `run`, `fetch`, `analyze`, `map`, `doctor`, `tui`) |
| Native map rendering (redist-map) | ✅ Complete (3 stubs: #62-#64) |
| HTML reports (redist-report) | ✅ Complete |
| Interactive TUI (redist-tui) | ✅ Complete (v1) |
| Performance benchmark (see §Performance) | ✅ Validated (1 run; reproduction recipe pending) |
| Migration parity record artifact | ⏭️ Skipped — no users to migrate |
| Permanent acceptance tests for `redist` invariants | ✅ `tests/acceptance/test_redist_invariants.py` (Plan 01) |
| Binary provenance embedding in outputs | ⚠️ Specced; doctor verify-manifest subcommand TODO |
| Entry-point cutover (doskey/batch → `redist`) | ✅ Plan 01 complete |
| Python deprecation notice | ⏭️ Skipped — no users to migrate |
| Python deletion / archival | ✅ Plan 02 complete |
| Partisan edge-weighting (Plan 03) | ✅ Module + CLI wiring + format spec; producer + e2e deferred |
| File-format specs (`adj-bin`, `final-assignments`, `partisan-shares`) | ✅ Written under `docs/file-formats/` |
| Reproducible-build pin (`redist/rust-toolchain.toml`) | ✅ Pinned to 1.95.0 |
| `redist-web` crate | ⚠️ Stub — kept as documented placeholder |

## Steady-State Surface

### Rust owns everything in the production path

| Crate | Responsibility |
|---|---|
| `redist-core` | Graph, Partition, BisectionTree, edge-weighting (VRA, partisan), METIS file I/O, FIPS, population balance |
| `redist-data` | TIGER reading, adjacency build, island bridging, .adj.bin serialization |
| `redist-analysis` | Compactness, VRA, demographic, political, partisan metrics, urban analysis, comparison |
| `redist-cli` | All production subcommands |
| `redist-map` | SVG → PNG rendering with insets, projections, color schemes, embedded fonts |
| `redist-report` | HTML reports, plan manifests, audit, export, rplan roundtrip |
| `redist-tui` | Interactive ratatui frontend |
| `redist-web` | Documented stub crate; future Rust dashboard work would land here |
| `redist_py` | PyO3 bridge — retained for research scripts and validation harness |

### Python retained, scoped narrowly

| Path | Purpose | Justification |
|---|---|---|
| `scripts/web/generate_dashboard.py` and siblings | Static HTML dashboard (Jinja2) | No Rust equivalent worth building; HTML templating fine in Python |
| `scripts/data/download_orchestrator.py` and siblings | Census data acquisition | Network-bound; Python shorter than Rust here |
| `scripts/figures/` | LaTeX paper figures | Research artifact; runs occasionally |
| `scripts/experiments/`, `baseline/`, `experimental/` | Research scripts | Disposable, prototype-grade |
| `tests/` | pytest suite (covers Rust via PyO3 and Python remnants) | Test framework |
| `archive/python-pipeline-final/` (new) | Sealed reference of the last-working Python pipeline | Forensic ground truth (see §Python Algorithm Preservation) |

### Maps: zero Python in the production path

This is definitive. `redist-map` is feature-complete. All matplotlib/cartopy code paths are dead — `src/apportionment/visualization/maps.py` is not imported by any active pipeline orchestrator. The only Python visualization that survives is `scripts/figures/` for LaTeX paper figures, which is **not** part of the production redistricting pipeline. After cutover, `import matplotlib` does not appear anywhere a `redist` invocation touches.

The three remaining map stubs (#62 choropleth real geometry, #63 rounds rendering, #64 splits) are tracked as Rust issues, not Python fallbacks.

### Algorithm: zero parallel implementations in active code

After cutover, `redist-core` is the only implementation of the bisection algorithm in the active code paths. The Python `RecursiveBisection` class is **archived, not deleted** (see §Python Algorithm Preservation). PyO3 bindings (`redist_py.build_vra_edge_weights`, etc.) remain so research scripts can call the Rust algorithm from Python without re-implementing it.

## Provenance and Reproducibility

### Binary provenance (required before Plan 01)

Every output JSON file produced by `redist` must include an embedded provenance block:

```json
{
  "redist_version": "0.1.2",
  "redist_build_commit": "<git sha>",
  "redist_build_date": "2026-04-29T00:00:00Z",
  "rustc_version": "1.84.0",
  "manifest_sha256": "<hash of input manifest>"
}
```

A new `redist doctor --verify-manifest <output.json>` subcommand validates: (a) binary version recorded, (b) build commit reachable in git history, (c) input manifest hash matches recorded hash. This enables a special master to attest "this output was produced by the published `redist` source at commit X."

### Reproducible-build recipe

`docs/REPRODUCIBLE_BUILD.md` (new) documents:
- Pinned `rustc` toolchain (rust-toolchain.toml)
- Build command: `cargo build --release --locked` from clean checkout
- Verification: two builds from the same source produce byte-identical binaries (or the recipe explains where deterministic-build limitations live)

### Python algorithm preservation

`src/apportionment/partition/recursive_bisection.py` is the forensic ground truth for the migration. Two roles in the v1 review (MERIDIAN, COVENANT) flagged that deleting it eliminates the only sealed reference for future "Rust output looks weird; is this a regression vs. the historical Python output?" investigations.

**Decision:** before Plan 02 deletion, the Python pipeline scripts and library are **moved** to `archive/python-pipeline-final/` with a top-level README:
- Marking the directory "REFERENCE ONLY — last working Python pipeline at commit `<sha>`, 2026-04-29"
- Linking to migration-log.md and the migration parity record
- Stating: not imported by any active code; not maintained; do not modify without making it a forensic copy of itself

Disk cost: ~5 MB. Audit-trail value: substantial.

### Migration parity record (required before Plan 02 Task 4)

Before the validation harness is deleted, a single one-shot run produces and commits:

`design/rust-port/migration-parity-record-2026-04-29.csv` — all 50 states × 3 census years, each row containing:
- State, year
- Python output: PP, Reock, max population imbalance, district count
- Rust output: PP, Reock, max population imbalance, district count
- Pass/fail under validation-harness criteria

This is the in-repo proof that the migration produced equivalent outputs. Hashable for court discovery. Tagged at commit `migration-baseline-2026-04-29`.

## Performance

### Headline benchmark

50-state V3 2020 (8 workers, Windows 11, 2026-04-25):
- Python: ~55 minutes
- Rust CLI (.adj.bin native): 15.5 seconds
- Speedup: ~213×

### Reproduction recipe (TODO before publishing the speedup claim)

A separate document `design/rust-port/benchmark-protocol.md` (currently `benchmarks.md`, may need expansion) must specify:
- Hardware (CPU, RAM, storage class)
- Toolchain versions (rustc, Python, METIS)
- Input set (which states, which year, which adjacency files)
- Repetition count + wall-clock measurement methodology
- Warmup / cold-cache convention

The "~213×" number is currently one run. Before the spec or any paper cites it as a published claim, the protocol must be reproducible by a third party.

## File Formats

### Canonical output formats

| File | Format | Status | Notes |
|---|---|---|---|
| `final_assignments.json` | JSON | **Canonical** | Object: `{geoid: district_id}`. GEOIDs are 11-character TIGER strings with leading zeros. |
| `final_assignments.pkl` | Python pickle | **Deprecated legacy** | Written by the old Python pipeline. No active reader after Plan 02. Existing files remain readable from `archive/python-pipeline-final/` for forensic purposes only. |
| `*.adj.bin` | RADJ binary | **Canonical** | Adjacency graph with vertex weights. Spec at `docs/file-formats/adj-bin.md` (REQUIRED, not yet written). |
| `vra_analysis.json` | JSON | **Canonical** | Atomic write with assignments. Schema in redist-cli output module. |
| `district_summary.csv` | CSV | **Canonical** | Per-district metrics. Header documented in REDIST_CLI.md. |

### Required new docs

- `docs/file-formats/adj-bin.md` — formal spec of the RADJ format: magic bytes, version, endianness, header layout, vertex/edge encoding, validation tests, reference Python reader (audit-only)
- `docs/file-formats/final-assignments.md` — JSON schema with examples
- `docs/file-formats/manifest.md` — provenance block schema (see §Binary Provenance)

### External-tool compatibility

Validate exports parse cleanly in:
- GerryChain (Python; expects GeoJSON FeatureCollections or assignment CSV)
- Districtr (web; consumes assignment CSV with `geoid, district_id` columns)
- PlanScore (web; consumes shapefile + assignment CSV)

A compatibility matrix at `docs/file-formats/external-tool-matrix.md` documents what `redist export` produces for each, with smoke tests.

## Entry Points

| Entry | Today | After cutover |
|---|---|---|
| doskey `run` | `python run_complete_redistricting.py` | `redist run` |
| doskey `runtest` | `python run_complete_redistricting.py --run-type test` | `redist run --run-type test` |
| `run_redistricting.bat` | Python pipeline | `redist run` |
| `deploy_web.bat` | `scripts/web/generate_dashboard.py` | unchanged (Python kept) |
| `redist tui` | New | unchanged |

## Deletion / Archive List (final state)

### Move to archive at cutover

```
src/apportionment/partition/                   → archive/python-pipeline-final/partition/
src/apportionment/data/                        → archive/python-pipeline-final/data/
src/apportionment/visualization/maps.py        → archive/python-pipeline-final/visualization/
scripts/pipeline/run_complete_redistricting.py → archive/python-pipeline-final/scripts/pipeline/
scripts/pipeline/process_nation.py             → archive/python-pipeline-final/scripts/pipeline/
scripts/pipeline/process_single_state.py       → archive/python-pipeline-final/scripts/pipeline/
scripts/pipeline/run_state_redistricting.py    → archive/python-pipeline-final/scripts/pipeline/
scripts/utils/pipeline_orchestrator.py         → archive/python-pipeline-final/scripts/utils/
scripts/utils/progress_coordinator.py          → archive/python-pipeline-final/scripts/utils/
scripts/utils/stage_tracker.py                 → archive/python-pipeline-final/scripts/utils/
scripts/utils/status_protocol.py               → archive/python-pipeline-final/scripts/utils/
scripts/utils/terminal_utils.py                → archive/python-pipeline-final/scripts/utils/
```

### Delete now (orphaned, no forensic value)

```
scripts/pipeline/run_party_specific_redistricting.py
scripts/pipeline/run_recursive_bisection.py
scripts/pipeline/complete_paper2_comparison.py
scripts/pipeline/fill_missing_cities.py
```

### Delete after migration parity record is committed (Plan 02 final step)

```
scripts/pipeline/compare_rust_vs_python.py     # once parity record is committed
scripts/pipeline/validate_rust_vs_python.py    # once parity record is committed
scripts/data/generate_adj_bin.py               # one-time bridge, conversion complete
```

### Keep

```
redist/crates/redist-web/                      # documented stub; reserve namespace for future
```

## Verification Gates

### In place during migration (do not delete until Plan 02 final step)

- `scripts/pipeline/compare_rust_vs_python.py` — assignment-file parity check
- `scripts/pipeline/validate_rust_vs_python.py` — head-to-head pipeline validation

These deliberately do **not** require bit-identical output (METIS has multiple valid partitions). They require both pipelines to produce *valid* outputs. Quantitative bounds:
- Population imbalance ≤ 0.5% per district (hard gate)
- District count exactly equals state's congressional seats (hard gate)
- Contiguity: each district connected component count = 1 (hard gate)
- Polsby-Popper agreement within ±3% per state mean (parity gate)
- Reock agreement within ±5% per state mean (parity gate)

### Permanent (required before validation-harness deletion)

`tests/acceptance/test_redist_invariants.py` (new) — runs `redist run` against `--scope` VT and AL fixtures and asserts:
- Exit code 0
- All districts within population balance
- All districts contiguous
- District counts correct
- PP and Reock within published bounds

Run in CI on every PR.

## Out of Scope

- Phase 6 (native METIS): explicitly skipped per migration-log.md (subprocess overhead is ms-scale, not a bottleneck)
- New algorithmic features (e.g., partisan edge-weighting): see [Plan 03](../plans/2026-04-29-partisan-bisection-weighting.md). Not started until cutover and deletion are done.

## Related Docs

- `design/rust-port/migration-log.md` — historical record of Phases 0-5
- `docs/REDIST_CLI.md` — CLI reference (canonical)
- `docs/REPRODUCIBLE_BUILD.md` (new, required) — toolchain pin + build recipe
- `docs/file-formats/` (new, required) — `.adj.bin`, `final_assignments.json`, manifest schemas
- `docs/superpowers/specs/2026-04-28-redist-cli-architecture.md` — CLI internal architecture
- `docs/superpowers/plans/2026-04-29-entry-point-cutover.md` — Plan 01
- `docs/superpowers/plans/2026-04-29-python-deletion.md` — Plan 02
- `docs/superpowers/plans/2026-04-29-partisan-bisection-weighting.md` — Plan 03

## v2 Changelog

Incorporates findings from 7-role review (2026-04-29):
- **BENCHMARK (BLOCK):** required permanent acceptance tests before validation harness deletion; spec'd quantitative invariant bounds
- **MERIDIAN, COVENANT:** Python algorithm preserved via `archive/` rather than deleted
- **DATUM:** "~213× faster" claim flagged as needing reproducible benchmark protocol
- **SURVEY, COVENANT:** binary provenance embedding required in outputs; `redist doctor --verify-manifest` subcommand specified
- **SURVEY:** redist-web stub kept rather than deleted (documented placeholder)
- **LEDGER:** canonical assignment file is JSON; `.pkl` deprecated; `.adj.bin` formal spec required; external-tool compatibility matrix required
- **TRENCH:** new pitfalls PP-15, PP-16, PP-17 added to design/pitfalls/pitfalls-pipeline.md
- **MERIDIAN:** parity gates now have numerical bounds (PP ±3%, Reock ±5%); contiguity required as hard gate
