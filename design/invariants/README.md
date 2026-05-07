# REDIST System Invariants

Properties the system must always satisfy. Unlike pitfalls (failure patterns), invariants state what is always true — and prove it with a test.

An invariant is **ENFORCED** when a test would fail if the invariant were violated. PARTIAL means partial coverage. OPEN means the invariant is stated but no test guards it.

---

## Domains

| Domain | Prefix | Count |
|--------|--------|-------|
| Constitutional | CI | 3 |
| VRA Algorithm | VA | 5 |
| Data Format | DF | 4 |
| Algorithm Correctness | AC | 5 |
| Interface/Protocol | IP | 4 |
| **Total** | | **21** |

---

## Status Table

| ID | Invariant | Status | Test |
|----|-----------|--------|------|
| CI-01 | Population balance ≤ 0.5% in every final partition | ENFORCED | test_vra_pipeline_balance.py, test_rust_graph.py::test_assert_balanced_fails |
| CI-02 | Every tract assigned to exactly one district | ENFORCED | TestRustCLIAcceptance::test_vt_rust_193_tracts |
| CI-03 | District count matches state constitutional target | ENFORCED | TestRustCLIAcceptance::test_vt_rust_one_district, test_al_rust_mm_count |
| VA-01 | VRA MM threshold is exclusive (> 0.50, not ≥) | ENFORCED | test_rust_compactness.py::test_threshold_exclusive_at_50pct |
| VA-02 | Adaptive boost formula: max(3.0, 10.0*(1-0.7*f)) | ENFORCED | test_vra_edge_weighting.py::TestRustFormulaParity |
| VA-03 | vra_mode stays True throughout a VRA run | ENFORCED | test_vra_edge_weighting.py::test_vra_mode_stays_true_throughout_run |
| VA-04 | vra_analysis written atomically with final_assignments | ENFORCED | test_vra_pipeline_balance.py::test_vra_analysis_pkl_saved_when_vra_mode_true |
| VA-05 | VRA formula is single authoritative implementation (Rust) | ENFORCED | test_vra_edge_weighting.py::TestRustFormulaParity (6 state profiles) |
| DF-01 | GEOID is exactly 11 characters for census tracts | ENFORCED | test_rust_tiger.py::test_geoid_length |
| DF-02 | Edge weight keys are canonical (u < v, never u > v) | ENFORCED | test_rust_graph.py::test_edge_weights_canonical_order |
| DF-03 | Adjacency is symmetric (j in adj[i] ⟺ i in adj[j]) | ENFORCED | test_rust_adjacency.py::test_adjacency_is_symmetric |
| DF-04 | Vertex weights are positive integers (population ≥ 1) | ENFORCED | bisection_runner.rs::test_invariant_vertex_weights_positive |
| AC-01 | Bisection is complete: every tract in exactly one leaf | ENFORCED | test_rust_cli.py::test_al_rust_population_balance (implies all assigned) |
| AC-02 | Bisection split target weights match k_left/k ratio | ENFORCED | TestRustCLIAcceptance::test_al_rust_population_balance |
| AC-03 | Leaf sort order is BFS (depth then path), not lexicographic | ENFORCED | bisection_runner.rs::test_leaf_sort_bfs_order |
| AC-04 | Polsby-Popper formula: 4π×A/P², perimeter = exterior only | ENFORCED | test_rust_compactness.py::test_square_pp_is_pi_over_4 |
| AC-05 | Target partition weights sum to 1.0 (2-way and n-way) | PARTIAL | bisection_runner.rs::test_invariant_target_weights_sum_to_one_2way; n-way OPEN |
| IP-01 | STATUS output is ASCII-only (no Unicode) | ENFORCED | test_rust_cli.py::test_no_unicode_in_help_output |
| IP-02 | Error propagation from parallel tasks: no silent filter | ENFORCED | TestRustCLIAcceptance (Alabama run would show error not silent empty) |
| IP-03 | REDIST_PYTHON used for all Python subprocess calls | ENFORCED | TestRustCLIAcceptance fixtures set REDIST_PYTHON=sys.executable |
| IP-04 | CWD must equal project root when using relative manifest paths | PARTIAL | test_fetch.py::TestCheckOnly (uses cwd=str(tmp_path)) |

---

## Full Invariant Entries

See domain files:
- [invariants-constitutional.md](invariants-constitutional.md)
- [invariants-vra.md](invariants-vra.md)
- [invariants-data.md](invariants-data.md)
- [invariants-algorithm.md](invariants-algorithm.md)
- [invariants-interface.md](invariants-interface.md)

---

## Adding an Invariant

Use `/update-invariants` skill. Required fields:

1. **Invariant**: one sentence, no qualifications
2. **Why it must hold**: constitutional, mathematical, or contractual reason
3. **When it can be violated**: what refactor could break it
4. **Enforcement**: assertion, type system, or test
5. **Test**: must exist; if not, create it before marking ENFORCED
