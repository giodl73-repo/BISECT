# Plan 02: Python Archive and Deletion

> **For agentic workers:** Do not execute until Plan 01 (entry-point cutover) is complete and validated. Each task is a separate commit so any single change can be rolled back individually. **The validation harness is not deleted until a permanent parity record artifact has been committed.**

**Date:** 2026-04-29
**Updated:** 2026-04-29 (v2 — incorporates BENCHMARK BLOCK, MERIDIAN, COVENANT, DATUM, TRENCH review findings)
**Goal:** Move duplicated Python pipeline code to `archive/python-pipeline-final/` (forensic reference) and delete genuinely orphaned scripts. After this plan, the active Python tree contains only: dashboard generation, data download, research scripts, paper figures, and tests.

**Depends on:** Plan 01 complete and validated for ≥ 5 calendar days, ≥ 10 successful 50-state runs, exercising ≥ 3 distinct flag combinations (see Task 4 soak-time signal).
**Blocks:** Plan 03 (partisan work) is cleaner if started after this — fewer stale references to navigate.

**Key v2 change:** "Delete" is mostly replaced by "archive." Forensic value of the Python pipeline outweighs the disk savings of deletion. See spec v2 §Python Algorithm Preservation.

---

## Pre-Conditions

Before any archival/deletion, verify:
- `redist run -y 2020 -v <ver>` is the documented and default entry point (Plan 01 outcome)
- `compare_rust_vs_python.py` and `validate_rust_vs_python.py` continue to pass on a fresh run
- No active development branch references the files we're about to move/delete
- Soak-time signal met (see Task 4)

---

## Task 1: Delete genuinely orphaned research scripts

**Files (delete):**
```
scripts/pipeline/run_party_specific_redistricting.py
scripts/pipeline/run_recursive_bisection.py
scripts/pipeline/complete_paper2_comparison.py
scripts/pipeline/fill_missing_cities.py
```

These are research/legacy artifacts called by no production entry point and have no forensic value (they are *failed* or superseded experiments, not the production pipeline). Safe to delete now; this task can run **before** Plan 01 if you want to start cleanup early.

- [ ] **1.1** Grep the repo to confirm no active code imports these files. Expected: only references in `context/archive/`.
- [ ] **1.2** Delete the four files.
- [ ] **1.3** If any imports `RecursiveBisection` from `src/apportionment/partition/`, leave them deleted (the import target itself goes in Task 3).
- [ ] **1.4** Run `pytest tests/` — must pass.
- [ ] **1.5** Commit: "Delete 4 orphaned research scripts (Plan 02 Task 1)".

**Exit:** Scripts gone. Tests green.

---

## Task 2: Archive pipeline orchestration scripts

**Files (move to `archive/python-pipeline-final/`):**
```
scripts/pipeline/run_complete_redistricting.py
scripts/pipeline/process_nation.py
scripts/pipeline/process_single_state.py
scripts/pipeline/run_state_redistricting.py
scripts/utils/pipeline_orchestrator.py
scripts/utils/progress_coordinator.py
scripts/utils/stage_tracker.py
scripts/utils/status_protocol.py
scripts/utils/terminal_utils.py
```

These are replaced by `redist run` / `redist state` / `redist states` but retain forensic value. **Move via `git mv`, do not delete.**

- [ ] **2.1** Confirm Plan 01 has been live for the full soak-time signal (≥ 5 days, ≥ 10 runs, ≥ 3 flag combinations). Document the soak data in this plan's appendix.
- [ ] **2.2** Create `archive/python-pipeline-final/` directory.
- [ ] **2.3** Write `archive/python-pipeline-final/README.md` per spec v2 template:
   - "REFERENCE ONLY — last working Python pipeline at commit `<sha>`, 2026-04-29"
   - Link to migration-log.md and migration parity record
   - Statement: not imported by any active code; not maintained; do not modify without making it a forensic copy of itself
- [ ] **2.4** Grep for imports of each of these files from elsewhere in `scripts/`, `src/`, and `tests/`. Anywhere they're imported, those callers either also need archival (research scripts) or migration to `subprocess.run(["redist", ...])`. Enumerate every found caller in this plan's appendix.
- [ ] **2.5** For each survivor that imports a soon-to-be-archived module, update the survivor (research scripts to either invoke `redist` via subprocess or be archived themselves).
- [ ] **2.6** `git mv` each of the 9 files into `archive/python-pipeline-final/scripts/...` preserving sub-paths.
- [ ] **2.7** Run `pytest tests/`. Some tests may fail (those exercising the Python pipeline directly). For each failure: either the test is also obsolete (archive it with the same commit) or it should be migrated to invoke `redist` (separate commit).
- [ ] **2.8** Run a 50-state pipeline via `redist run` to confirm the archival didn't break anything.
- [ ] **2.9** Commit: "Archive Python pipeline orchestrators — replaced by redist CLI (Plan 02 Task 2)".

**Exit:** Pipeline scripts moved to archive. `redist run` is the only active orchestrator.

---

## Task 3: Archive duplicated algorithm library (with strict deletion gate)

**Files (move to archive):**
```
src/apportionment/partition/         # whole directory → archive/python-pipeline-final/src/apportionment/partition/
src/apportionment/data/              # whole directory → archive/python-pipeline-final/src/apportionment/data/
src/apportionment/visualization/maps.py → archive/python-pipeline-final/src/apportionment/visualization/maps.py
```

`src/apportionment/partition/recursive_bisection.py` and `metis_wrapper.py` duplicate `redist-core`. `src/apportionment/data/adjacency.py` and siblings duplicate `redist-data`. `visualization/maps.py` is dead matplotlib code. All have forensic value as the historical implementation.

**Strict deletion gate (replaces v1's "pytest tests/ green"):**

- [ ] **3.1** Run an explicit grep over the *active* code paths (excluding `archive/`, `context/archive/`, and `tests/`):
   ```
   grep -r "from apportionment.partition" --include="*.py" .
   grep -r "from apportionment.data" --include="*.py" .
   grep -r "from apportionment.visualization.maps" --include="*.py" .
   grep -r "import apportionment.partition" --include="*.py" .
   ```
   **Required exit:** ZERO hits in active code. Hits in `tests/` go to step 3.2.
- [ ] **3.2** Enumerate every test file that imports the soon-to-be-archived modules. Per BENCHMARK review v1, this includes at minimum:
   - `tests/integration/test_metis_integration.py` (24 calls to `partition_graph` from `metis_wrapper.py`)
   - Any other test surfaced by 3.1
- [ ] **3.3** For each enumerated test: migrate the import to `redist_py` (PyO3 bindings) which call into Rust. The test logic should still work — only the import path changes. If a test cannot be migrated cleanly (e.g., it tested Python-specific behaviour that Rust doesn't replicate exactly), flag it for triage in this plan's appendix and decide: migrate, archive, or rewrite.
- [ ] **3.4** Re-run grep step 3.1. Required exit: ZERO hits anywhere except `archive/`.
- [ ] **3.5** `git mv` the directories/file into `archive/python-pipeline-final/src/apportionment/...`.
- [ ] **3.6** Run `pytest tests/`. Green.
- [ ] **3.7** Commit: "Archive Python algorithm library — replaced by redist-core via PyO3 (Plan 02 Task 3)".

**Exit:** No parallel implementations in the active tree. `redist-core` is the only bisection. Research scripts and tests that need the algorithm call it via `redist_py`.

---

## Task 4: Generate and commit migration parity record (NEW in v2 — required before validation harness deletion)

**Files (new):**
```
design/rust-port/migration-parity-record-2026-04-29.csv
design/rust-port/migration-parity-record-README.md
```

This is the **BENCHMARK BLOCK** unblocker plus the COVENANT/SURVEY/DATUM/MERIDIAN/LEDGER consensus requirement. The validation harness is the only proof Rust matches Python; before deletion, freeze that proof in a permanent artifact.

- [ ] **4.1** Run a final round of `validate_rust_vs_python.py` for all 50 states × 3 census years (2000, 2010, 2020).
- [ ] **4.2** Capture the per-state results in `design/rust-port/migration-parity-record-2026-04-29.csv` with columns:
   ```
   state, year, py_pp_mean, py_reock_mean, py_max_imbalance_pct, py_district_count,
              rust_pp_mean, rust_reock_mean, rust_max_imbalance_pct, rust_district_count,
              pp_diff_pct, reock_diff_pct, balance_gate_pass, contiguity_gate_pass,
              district_count_match, pp_within_3pct, reock_within_5pct, overall_pass
   ```
- [ ] **4.3** Write `migration-parity-record-README.md` documenting: methodology, hardware, dates, validation harness commit hash, summary statistics (states passed / failed, mean PP delta, mean Reock delta, range of population imbalance).
- [ ] **4.4** Compute SHA-256 of the CSV; record it in the README and in the commit message.
- [ ] **4.5** Tag the commit `migration-baseline-2026-04-29` (or similar).
- [ ] **4.6** Commit: "Migration parity record — final validation harness output (Plan 02 Task 4)".

**Exit:** Permanent artifact in repo. SHA-256 published. Tagged release.

---

## Task 5: Delete validation harness (now safe)

**Files (delete):**
```
scripts/pipeline/compare_rust_vs_python.py
scripts/pipeline/validate_rust_vs_python.py
```

After Task 4, the parity record is the permanent proof. The harness scripts have nothing left to compare against (Python pipeline is archived) and the parity record stands as forensic evidence.

- [ ] **5.1** Confirm Task 4 artifact is committed and tagged.
- [ ] **5.2** Confirm `tests/acceptance/test_redist_invariants.py` (from Plan 01 Task 5) is in CI and passing.
- [ ] **5.3** Delete the two harness files.
- [ ] **5.4** Commit: "Delete migration validation harness — superseded by parity record + acceptance tests (Plan 02 Task 5)".

**Exit:** Harness gone. Parity record + acceptance tests are the permanent verification mechanism.

---

## Task 6: Delete one-time bridge scripts

**Files (delete):**
```
scripts/data/generate_adj_bin.py
```

This script converted legacy pkl adjacency files to .adj.bin format. All 50 states have already been converted (per migration-log.md Phase 2 entry, 2026-04-25).

- [ ] **6.1** Verify `outputs/data/{2000,2010,2020}/adjacency/` contains `.adj.bin` files for all 50 states.
- [ ] **6.2** Delete the script.
- [ ] **6.3** Commit: "Delete one-time pkl→adj.bin bridge — conversion complete (Plan 02 Task 6)".

**Exit:** Bridge scripts gone.

---

## Task 7: Keep `redist-web` as documented stub (REVERSED in v2)

Per SURVEY review v1: empty crate stubs signal "this namespace is reserved for future work" and prevent naive re-creation. Replace deletion with documentation.

- [ ] **7.1** Edit `redist/crates/redist-web/src/lib.rs`. Replace `// redist-web — stub` with a short comment block:
   ```rust
   //! redist-web — RESERVED for future Rust dashboard work.
   //!
   //! Currently empty. Dashboard generation is performed by Python
   //! (scripts/web/generate_dashboard.py, Jinja2). If/when a Rust
   //! dashboard becomes desirable, this is where it lives.
   //!
   //! See spec: docs/superpowers/specs/2026-04-29-rust-python-final-architecture.md
   ```
- [ ] **7.2** Commit: "Document redist-web as reserved stub (Plan 02 Task 7)".

**Exit:** Crate kept. Future readers know its status.

---

## Task 8: Update CLAUDE.md after archive + deletion

- [ ] **8.1** Update CLAUDE.md "Critical Files" → remove the Pipeline section's Python file references; replace with `redist` subcommands. Add a line noting Python pipeline scripts archived under `archive/python-pipeline-final/`.
- [ ] **8.2** Update "Structure" tree to reflect the slimmed `src/apportionment/` and `scripts/` plus the new `archive/` entry.
- [ ] **8.3** Update "Recent Changes": "2026-XX-XX: Python pipeline archived — `redist` CLI is sole orchestrator. ~12 Python files moved to `archive/python-pipeline-final/`. Migration parity record committed."

---

## Estimated Outcome

After this plan:
- `src/apportionment/` shrinks to 5-7 files (config and visualization helpers, if any remain)
- `scripts/pipeline/` shrinks from 32 files to ~10 (post-processing utilities, if kept)
- `scripts/utils/` shrinks substantially
- `scripts/web/`, `scripts/data/download_*`, `scripts/figures/`, `scripts/experiments/` unchanged
- `archive/python-pipeline-final/` contains ~12 files (sealed reference, do-not-modify)
- `design/rust-port/migration-parity-record-2026-04-29.csv` permanent

## Soak-Time Signal Definition (NEW in v2 per TRENCH review)

"Plan 01 has been live for ≥ 1 week" was vague. Replace with:
- **≥ 5 calendar days** since Plan 01 Task 6 commit
- **≥ 10 successful 50-state runs** through `redist run` (count from CI + manual runs)
- **≥ 3 distinct flag combinations** exercised across those runs (e.g., default, `--run-type test`, `--states VT DE AK`)
- **Zero unresolved bug reports** filed against `redist run` in that window

Document the actual measured numbers in this plan's appendix at Task 2.1.

## Rollback Plan (rewritten in v2 per TRENCH review)

`git revert <commit>` succeeds for any single commit but does not necessarily produce a *correct* state if intermediate work has accrued. **Correct rollback procedure:**

1. **Within a single task:** Each task is one commit. `git revert <task-commit>` is safe immediately after the task lands.
2. **Across multiple tasks:** Revert in reverse order. To undo the entire plan, revert Task 8, then Task 7, … then Task 1.
3. **After any other work has merged:** Stop. Do not attempt automated rollback. Resolve manually:
   - Identify which downstream commits depend on the to-be-reverted change
   - Coordinate with their authors
   - Either revert the dependent commits first or migrate them to no longer depend
4. **For Tasks 4-5 (parity record + harness deletion):** the parity record is committed at a tag. Reverting Task 5 alone restores the harness; the parity record stays. This is the safe ordering.
5. **For Tasks 2-3 (archival):** archived files are recoverable via `git mv` reverse, but any test migrations done in Task 3 may need separate undo. Track in the appendix.

## Appendix (filled in during execution)

### Soak-time measurements (Task 2.1)
*To be filled in*

### Imports of soon-to-be-archived modules (Task 2.4, 3.2)
*To be filled in*

### Test files migrated to PyO3 bindings (Task 3.3)
*To be filled in*
