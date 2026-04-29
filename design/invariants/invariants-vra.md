# VRA Algorithm Invariants (VA-01..VA-05)

Properties of the Voting Rights Act compliance implementation that must be maintained across all refactors. These invariants encode the algorithm's correctness conditions — violating any one produces wrong VRA compliance results.

---

## VA-01: VRA MM threshold is exclusive (> 0.50, not ≥)

**Invariant:** A district is majority-minority if and only if its minority fraction strictly exceeds the threshold. At exactly 50.00%, the district is NOT majority-minority.

**Why it must hold:** Python vra_utils.py line 236: `is_mm = pct_minority > mm_threshold`. The Rust implementation must match Python exactly. Using ≥ instead of > would misclassify borderline districts (exactly 50%) as MM, producing incorrect mm_count values that diverge from the Python pipeline.

**When it can be violated:** Changing `>` to `>=` in vra_analysis.rs:83. This was done by mistake in Phase 4a and caught by the role review.

**Enforcement:** `is_mm = pct_minority > mm_threshold` in vra_analysis.rs. Test verifies that exactly 50% is NOT MM.

**Test:** `tests/unit/test_rust_compactness.py::TestVraAnalysis::test_threshold_exclusive_at_50pct`
`redist/crates/redist-analysis/src/vra_analysis.rs::test_threshold_boundary_exclusive`

**Status:** ENFORCED

---

## VA-02: Adaptive boost formula: max(3.0, 10.0*(1-0.7*f_minority))

**Invariant:** The VRA edge weight factor α must equal `max(3.0, 10.0 * (1.0 - 0.7 * f_minority))` where f_minority is the fraction of tracts at or above the 40% threshold. This formula may not be changed without updating paper D.0.

**Why it must hold:** This is the single authoritative implementation (redist-core/src/vra.rs). Both the Python pipeline and the Rust CLI call this function. Any divergence produces different redistricting results across pipeline modes.

**When it can be violated:** Changing the formula in vra.rs without updating the Python pipeline call; changing the formula in the Python pipeline without deleting it (Phase 1a deleted the Python copy).

**Enforcement:** The Python formula was deleted in Phase 1a. Only vra.rs contains it. Parity tests verify output matches for 6 state profiles.

**Test:** `tests/unit/test_vra_edge_weighting.py::TestRustFormulaParity` (6 state profiles: AL, GA, CA, SC, all-minority, no-minority)

**Status:** ENFORCED

---

## VA-03: vra_mode stays True throughout a VRA run

**Invariant:** In metis-vra partition mode, the vra_mode flag is set to True at the start of the run and never cleared before the vra_analysis block completes.

**Why it must hold:** If vra_mode is cleared prematurely, the vra_analysis block is skipped, and vra_analysis.json/pkl is not written. The dashboard and downstream tools have no VRA compliance data. This was the original vra_mode premature-clear bug.

**When it can be violated:** Any code that sets `vra_mode = False` before the analysis block. This happened because developers thought clearing it would prevent an IndexError (it did the opposite).

**Enforcement:** vra_analysis.json is always written in run_single_state when partition_mode == 'metis-vra' AND num_districts > 1. The acceptance test verifies the file exists.

**Test:** `tests/unit/test_vra_edge_weighting.py::TestVRACodePath::test_vra_mode_stays_true_throughout_run`
`tests/integration/test_vra_pipeline_balance.py::TestVRACodePathIntegrity::test_vra_analysis_pkl_saved_when_vra_mode_true`
`tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance::test_al_rust_vra_analysis_written`

**Status:** ENFORCED

---

## VA-04: vra_analysis written atomically with final_assignments

**Invariant:** In VRA mode, vra_analysis.json and final_assignments.json are written together or not at all. If the process dies after writing one but before the other, both must be absent (recoverable as corrupt state).

**Why it must hold:** A dashboard that reads vra_analysis.json but finds no final_assignments.json (or vice versa) will produce inconsistent data. The atomic write ensures consistency. Corrupt states (partial writes) are detectable and restartable.

**When it can be violated:** Changing write_state_outputs to write files sequentially without rename-from-tmp pattern; removing the temp file detection; adding a write between the two renames.

**Enforcement:** write_state_outputs uses rename-from-tmp: both .tmp files written, then both renamed. Corrupt state detected by .tmp file presence.

**Test:** `tests/unit/test_pipeline_balance_check.py::TestRustBalanceCheckInPipeline`
`tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance::test_al_rust_vra_analysis_written`

**Status:** ENFORCED

---

## VA-05: VRA formula is single authoritative implementation

**Invariant:** The adaptive boost formula exists in exactly one place: `redist-core/src/vra.rs::build_vra_edge_weights`. No other file contains an implementation of this formula.

**Why it must hold:** Two implementations drift. The Phase 1a deletion of lines 226-260 from run_state_redistricting.py enforces this. Any future duplication creates a drift risk where one implementation gets updated and the other doesn't.

**When it can be violated:** Adding a Python fallback that reimplements the formula; reverting the Phase 1a deletion.

**Enforcement:** The Python call site (run_state_redistricting.py) now calls `redist_py.build_vra_edge_weights()` — no formula present. If someone adds a fallback, the parity tests will catch drift when they disagree.

**Test:** `tests/unit/test_vra_edge_weighting.py::TestRustFormulaParity` (would fail if Python reimplementation differs from Rust)

**Status:** ENFORCED
