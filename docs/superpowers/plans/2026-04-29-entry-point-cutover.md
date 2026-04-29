# Plan 01: Entry-Point Cutover

> **For agentic workers:** Do not execute until the spec at `docs/superpowers/specs/2026-04-29-rust-python-final-architecture.md` v2 has been reviewed and approved. Steps use checkbox (`- [ ]`) syntax for tracking.

**Date:** 2026-04-29
**Updated:** 2026-04-29 (v2 — incorporates SURVEY, BENCHMARK, COVENANT review findings; Task 0 deprecation window removed since no external users)
**Goal:** Switch the documented and default entry points from the Python pipeline to the `redist` Rust binary. After this plan, `run -y 2020 -v v1` produces full 50-state output via the Rust binary with no Python invoked for redistricting.

**Depends on:** spec v2 approval. Nothing else.
**Blocks:** Plan 02 (deletion).

---

## Pre-Conditions

The Rust CLI already runs full 50-state in 18s vs ~55min Python (per `design/rust-port/migration-log.md`). The validation harness already confirms output validity. No new code should be needed for the cutover itself; this plan is configuration changes plus doc updates plus pre-flight artifacts the v2 review surfaced as required. **No external users to migrate** — proceed without a deprecation window.

---

## Task 1: Audit `redist run` flag parity (semantics, not just presence)

**Files:** `redist/crates/redist-cli/src/args.rs`, `scripts/pipeline/run_complete_redistricting.py`

The Rust CLI's flag surface is a self-described mirror of the Python argparse surface (per the `MERIDIAN invariant` comment in `args.rs`). Verify before cutover.

- [ ] **1.1** List every flag accepted by `python run_complete_redistricting.py --help` and every flag in `RunArgs` struct. Diff them.
- [ ] **1.2** For each flag in **both** Python and Rust: document not just that the flag *exists* in both, but that the *semantics match*. For example, `--seed 42` is documented as "reproducible run" in Python — confirm Rust produces the same output given the same seed, OR explicitly publish the difference. Add semantics-comparison column to the diff table.
- [ ] **1.3** For each flag in Python but not in Rust (if any), decide: add to Rust, or document as deprecated.
- [ ] **1.4** For each flag in Rust but not in Python: ensure it has a sensible default so existing scripts don't break.
- [ ] **1.5** Run `redist run --help` and confirm the help text matches user expectation.

**Exit:** A diff table at the bottom of this plan listing every flag with: present-in-python, present-in-rust, semantics-match (yes/no/explained-difference).

---

## Task 2: Compatibility validation run (against parity bounds)

**Files:** N/A (run-time validation)

- [ ] **2.1** Run `python run_complete_redistricting.py -y 2020 -v cutover_python` (current default).
- [ ] **2.2** Run `redist run -y 2020 -v cutover_rust` (target default).
- [ ] **2.3** Run `python scripts/pipeline/compare_rust_vs_python.py --python-version cutover_python --rust-version cutover_rust`. Confirm parity report passes the spec's quantitative gates: population imbalance ≤ 0.5% (both), district counts correct, contiguity, PP within ±3%, Reock within ±5%.
- [ ] **2.4** Spot-check 3 states (VT, AL, CA) by opening district summaries and maps side by side.

**Exit:** Parity report passes all five quantitative gates from the spec. Spot checks visually consistent.

---

## Task 3: Update batch files and doskey aliases

**Files:** `setup_env.bat`, `run_redistricting.bat`, possibly `run_test.bat`

- [ ] **3.1** Edit `run_redistricting.bat`. Replace `py -3.13 scripts/pipeline/run_complete_redistricting.py %*` with `redist run %*`. Add a comment documenting the cutover date and pointing to this plan.
- [ ] **3.2** Edit `setup_env.bat`. Add a pre-flight check at the top: `where redist >NUL 2>&1 || echo WARNING: redist binary not on PATH` (mitigates PP-15). The doskey `run=run_redistricting.bat` line stays the same — it now resolves to Rust transitively.
- [ ] **3.3** If `run_test.bat` is a thin wrapper of `run_redistricting.bat --run-type test`, decide: leave it (one-line wrapper, harmless) or delete it (consistency with `redist run --run-type test`).
- [ ] **3.4** Open a new shell, run `setup_env.bat`, observe the PATH warning if any, then `run -p -y 2020 -v test_cutover -st VT`. Confirm it runs the Rust binary in dry-run mode.

**Exit:** doskey `run` invokes the Rust binary. Old shells need re-sourcing. PATH warning fires if `redist` not installed.

---

## Task 4: Update CLAUDE.md and README.md

**Files:** `CLAUDE.md`, `README.md`, `docs/REDIST_CLI.md`

- [ ] **4.1** In CLAUDE.md, update "Critical Files" → "Pipeline (executable)" section. Replace the Python script paths with the corresponding `redist` subcommands. Note that the Python paths are now archived under `archive/python-pipeline-final/` (Plan 02 will move them) and link there for reference.
- [ ] **4.2** In CLAUDE.md "Common Commands" section, update example invocations. The `run -v v1` examples should explicitly state they run the Rust binary now.
- [ ] **4.3** In CLAUDE.md "Recent Changes", add an entry for cutover date: "Entry-point cutover: doskey `run`/`runtest` now invoke `redist` binary directly. Python pipeline orchestrators archived under `archive/python-pipeline-final/`."
- [ ] **4.4** In CLAUDE.md "Stack" line, change "Python 3.13+, METIS" to "Rust (`redist` CLI, primary) + Python 3.13+ (research/dashboard), METIS".
- [ ] **4.5** README.md: scan for references to the Python pipeline as primary; update to point at `redist run`.
- [ ] **4.6** `docs/REDIST_CLI.md`: replace the "Migration" section with a "Cutover record" section dated this plan's execution.

**Exit:** New contributors reading CLAUDE.md learn that `redist` is primary, not Python.

---

## Task 5: CI / test confirmation (with strict invariants)

**Files:** `.github/workflows/*.yml`, `tests/acceptance/`

- [ ] **5.1** Check that CI runs both `pytest tests/acceptance/` and the Rust test suite. Confirm green.
- [ ] **5.2** Add a test at `tests/acceptance/test_redist_invariants.py` that invokes `redist run -p -y 2020 -v ci_test -st VT` (dry run for CI speed) and a separate slower test that does a real `redist state --state VT --year 2020` and asserts:
   - Exit code 0
   - `final_assignments.json` exists and parses
   - District count exactly equals 1 (VT)
   - Population balance ≤ 0.5%
   - All assigned tracts form a single connected component (contiguity)
   - PP score within published acceptable range for VT
- [ ] **5.3** Add the same kind of test for AL (7 districts, VRA target) — confirms multi-district invariants.

**Exit:** CI green on the new entry-point invocations with strict invariant assertions.

---

## Task 6: Commit and announce

- [ ] **6.1** Commit messages should reference this plan. One commit per task is fine; one commit for the whole plan is also fine if the diff is small.
- [ ] **6.2** Note in the commit body: "Plan 02 (Python archive + deletion) is unblocked after this lands."

---

## Task 7: Reproducible-build documentation (NEW in v2)

**Files:** `docs/REPRODUCIBLE_BUILD.md` (new), `redist/rust-toolchain.toml`

A court-admissibility requirement (per SURVEY, COVENANT review): a third party must be able to reproduce the `redist` binary from source.

- [ ] **7.1** Confirm `redist/rust-toolchain.toml` pins the rustc version. If absent, create it pinning to the version used in current CI.
- [ ] **7.2** Document in `docs/REPRODUCIBLE_BUILD.md`:
   - Pinned toolchain version
   - Build command (`cargo build --release --locked` from clean checkout)
   - How to verify two builds produce byte-identical binaries (or where deterministic-build limits live, e.g., timestamps)
   - Toolchain installation steps for Linux/macOS/Windows
- [ ] **7.3** Add `redist --version` output format that exposes git commit + rustc version (so output provenance blocks can include them). Wire into the provenance block referenced in the spec.

**Exit:** `docs/REPRODUCIBLE_BUILD.md` exists; `redist --version` shows commit + rustc; provenance block populated.

---

## Rollback Plan

If something breaks during cutover:
1. Revert `run_redistricting.bat` to invoke the Python script (one file edit).
2. Revert `setup_env.bat` PATH warning addition (cosmetic).
3. Python pipeline scripts are not yet archived/deleted at this point in the plan, so rollback is purely the batch file edit.

If the deprecation notice (Task 0) needs to extend, simply update the cutover date in `docs/PYTHON_DEPRECATION.md` and announce.

---

## Flag-Surface Diff (Task 1 result, 2026-04-29)

Comparison of `scripts/pipeline/run_complete_redistricting.py` argparse vs `redist/crates/redist-cli/src/args.rs::RunArgs`.

| Flag | Python | Rust | Semantics match | Notes |
|---|---|---|---|---|
| `--output-dir` | ✓ | ✓ (`Option<String>`) | yes | |
| `-y`/`--year` | ✓ (choices: 2020/2010/2000/all) | ✓ (default "all") | yes | Both accept "all" |
| `-v`/`--version` | ✓ (default v1) | ✓ (default v1) | yes | |
| `-w`/`--workers` | ✓ (default 12) | ✓ (default 12) | yes | |
| `--dpi` | ✓ (int, choices 72/100/150/200/300) | ✓ (string PossibleValuesParser) | yes | Type difference is internal; surface identical |
| `--election-year` | `-ey`/`--election-year` | `-e`/`--election-year` | yes | **Short flag mismatch:** `-ey` vs `-e` |
| `--skip-analysis` | ✓ | ✓ | yes | |
| `--run-analysis` | ✓ (default True) | absent | partial | Rust has skip-analysis only; positive form not needed since it's the default |
| `--skip-political` | ✓ | ✓ | yes | |
| `--skip-demographic` | ✓ | ✓ | yes | |
| `-s`/`--stages` | ✓ (default `data states nation`) | ✓ (same default) | yes | |
| `--reprocess` | ✓ | ✓ | yes | |
| `-r`/`--reset` | ✓ | ✓ | yes | |
| `-p`/`--print-only` | ✓ | ✓ | yes | |
| `-d`/`--debug` | ✓ | ✓ | yes | |
| `--partition-mode` | `-pm`/`--partition-mode` | `-m`/`--partition-mode` | yes | **Short flag mismatch:** `-pm` vs `-m` |
| `--processing-mode` | ✓ (choices batch/streaming) | ✓ (same) | yes | |
| `--minimum-boundary-length` | ✓ (default 10.0) | ✓ (same) | yes | |
| `--run-type` | `-rt`/`--run-type` | `--run-type` only | yes | **Short flag absent in Rust:** `-rt` |
| `--experiment-name` | ✓ | ✓ (Option) | yes | |
| `--states` | `-st`/`--states` | `--states` only | yes | **Short flag absent in Rust:** `-st` |
| `--states-only` | ✓ | absent | **DIFFER** | Python-only flag; runs only the states stage |
| `--skip-states` | ✓ | absent | **DIFFER** | Python-only flag; skips the states stage |

**Decision (with no external users):** The Rust flag surface becomes canonical post-cutover. Two functionally distinct Python-only flags (`--states-only`, `--skip-states`) are not migrated — equivalent behaviour can be achieved with `-s data nation` or `-s states` respectively. The four short-flag mismatches (`-ey`, `-pm`, `-rt`, `-st`) are not added; long forms are sufficient.

If parity for those flags becomes a requirement later, add them to `RunArgs` in args.rs without breaking changes.
