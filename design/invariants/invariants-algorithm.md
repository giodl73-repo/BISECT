# Algorithm Correctness Invariants (AC-01..AC-04)

Properties of the bisection algorithm and compactness metrics that ensure mathematical correctness.

---

## AC-01: Bisection is complete — every tract in exactly one leaf

**Invariant:** After run_all_splits completes, the union of all leaf node tract sets equals the full set of all tract indices (0..n-1), and the sets are pairwise disjoint.

**Why it must hold:** If a tract is in two leaf nodes, it gets two district IDs (the second overwrites the first), and one district is systematically over-counted. If a tract is in no leaf node, it's missing from assignments, and the balance check catches this — but the output would be invalid.

**When it can be violated:** A bug in the BFS loop where a node is removed from node_tracts before its children are inserted (the race condition fix: data must be extracted before par_iter, not after).

**Enforcement:** `if assignments.len() != n { return Err("bisection incomplete") }` at end of run_all_splits. The test_run_all_splits_two_districts test checks disjointness explicitly.

**Test:** `redist/crates/redist-cli/src/bisection_runner.rs::test_run_all_splits_two_districts` (checks d1.is_disjoint(&d2) and d1.len()+d2.len()==4)

**Status:** ENFORCED

---

## AC-02: Bisection split target weights match k_left/k ratio

**Invariant:** Every 2-way METIS split passes target partition weights of (k_left/k, k_right/k) to METIS, where k is the total districts for this region and k_left, k_right are the children's district counts. Equal-weight (50/50) is only correct when k_left == k_right.

**Why it must hold:** Without correct target weights, METIS splits 50/50. For k=7 (Alabama), this means the first split tries to put 50% of Alabama's population in each half — but the tree needs 3 districts on one side and 4 on the other (43%/57%). The accumulated imbalance across all levels produces 12-75% district deviation, violating CI-01.

**When it can be violated:** Removing the tpwgts file construction in split_subgraph; passing None always for target_weights; computing target_weights from wrong k values.

**Enforcement:** run_all_splits passes `Some((k_left/k, k_right/k))` when k_left != k_right. split_subgraph writes tpwgts file when target_weights is Some.

**Test:** `tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance::test_al_rust_population_balance` (Alabama k=7 must pass ±0.5% — would fail with 50/50 splits)

**Status:** ENFORCED

---

## AC-03: Leaf sort order is BFS (depth then path), not lexicographic

**Invariant:** Bisection leaf nodes are sorted by (path.len(), path) — depth-first within each level, then lexicographic within each depth. Plain lexicographic sort is wrong for mixed-length binary paths.

**Why it must hold:** The district ID assignment (1..k) follows leaf order. Plain lex order gives "0","00","01","1","10","11" instead of correct BFS order "0","1","00","01","10","11". The wrong order assigns district IDs inconsistently, though it doesn't affect population balance (CI-01). It does affect reproducibility and comparability between runs.

**When it can be violated:** Changing `sort_by_key(|(path,_)| (path.len(), path.clone()))` to `sort_by(|(a,_),(b,_)| a.cmp(b))`.

**Enforcement:** `sort_by_key(|(path, _)| (path.len(), path.clone()))` in run_all_splits.

**Test:** `redist/crates/redist-cli/src/bisection_runner.rs::test_leaf_sort_bfs_order` (verifies ["0","1","00","01"] not ["0","00","01","1"])

**Status:** ENFORCED

---

## AC-05: Target partition weights sum to 1.0

**Invariant:** For every METIS partition call (2-way bisection or n-way), the target partition weights must sum to exactly 1.0. For 2-way: left_frac + right_frac = 1.0. For n-way: w1 + w2 + ... + wn = 1.0.

**Why it must hold:** METIS interprets target weights as fractions of total load. If they don't sum to 1, METIS's balance objective is undefined. In practice METIS may accept non-summing weights but produce arbitrary results. In the 2-way bisection case, k_left/k + k_right/k = (k_left + k_right)/k = k/k = 1.0 by construction from BisectionNode — but this must be verified at the METIS call boundary.

**When it can be violated:** Rounding k_left/k and k_right/k independently (floating-point subtraction gives 1.0 - k_left/k ≠ k_right/k exactly). Implementing n-way partitioning with independently computed weights that don't account for rounding drift.

**Enforcement:** assert at split_subgraph entry: `(left_frac + right_frac - 1.0).abs() < 1e-6`. For n-way (future): assert sum of all partition weights ≈ 1.0 before writing tpwgts file.

**Test:** `redist/crates/redist-cli/src/bisection_runner.rs::test_invariant_target_weights_sum_to_one_2way` — verifies all BisectionTree nodes for k=2,3,4,7,8,14,52

**Status:** ENFORCED for 2-way bisection; OPEN for n-way (not yet implemented)

---

## AC-04: Polsby-Popper formula: 4π×A/P², perimeter = exterior ring only

**Invariant:** Polsby-Popper score = 4π × area / perimeter² where perimeter is the exterior ring length only (not including holes), capped at 1.0. This matches Python Shapely's `geometry.length` which returns exterior perimeter only.

**Why it must hold:** If hole perimeters are included, PP scores for donut-shaped districts are artificially low. More importantly, values would diverge from the Python pipeline, producing different compactness rankings for the same districts.

**When it can be violated:** Adding hole perimeters in polygon_perimeter(); changing to use `euclidean_length()` on the full polygon (which might include holes depending on implementation).

**Enforcement:** `polygon_perimeter` function returns `polygon.exterior().euclidean_length()` only; holes excluded explicitly.

**Test:** `tests/unit/test_rust_compactness.py::TestPolsbyPopper::test_square_pp_is_pi_over_4` (square PP must exactly equal π/4 = 0.7854; wrong perimeter gives different value)
`redist/crates/redist-analysis/src/compactness.rs::test_perimeter_excludes_holes`

**Status:** ENFORCED
