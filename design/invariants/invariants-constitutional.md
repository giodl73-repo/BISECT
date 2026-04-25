# Constitutional Invariants (CI-01..CI-03)

Properties required by the Equal Protection Clause (one-person-one-vote) and the district count mandated by state law.

---

## CI-01: Population balance ≤ 0.5% in every final partition

**Invariant:** The maximum fractional deviation of any district population from the ideal district population is at most 0.005 (±0.5%).

**Why it must hold:** The Equal Protection Clause requires congressional districts to be as equal in population as practicable. The Supreme Court has interpreted ±0.5% (Karcher v. Daggett) as the outer bound for justification without explanation. Any partition exceeding this is constitutionally suspect.

**When it can be violated:** Changing the METIS ufactor to a value > 1.005, changing the ufactor conversion formula (int→float), or applying ufactor at wrong bisection depth.

**Enforcement:** `Partition::assert_balanced(&vw, num_districts, 0.005)` called after every final partition in run_single_state. Raises ValueError immediately before any output is written.

**Test:** `tests/unit/test_rust_graph.py::TestPartitionRoundTrip::test_assert_balanced_fails`
`tests/acceptance/test_pipeline_acceptance.py::TestAlabamaVRAAcceptance::test_population_balance_within_half_percent`
`tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance::test_al_rust_population_balance`

**Status:** ENFORCED

---

## CI-02: Every tract assigned to exactly one district

**Invariant:** After bisection, the assignments dict contains every tract index exactly once, with no tract appearing in multiple districts.

**Why it must hold:** A tract in zero districts means residents are unrepresented. A tract in two districts means residents are double-counted. Both are unconstitutional.

**When it can be violated:** A bug in run_all_splits that drops a node from node_tracts before inserting its children (the race-condition fix in Phase 3d: removing before par_iter was the fix, but must not remove before getting the data). The "0/1437 tracts assigned" bug was this invariant being violated.

**Enforcement:** `if assignments.len() != n { return Err("bisection incomplete") }` at the end of run_all_splits. Acceptance test verifies exact count.

**Test:** `tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance::test_vt_rust_193_tracts`
`tests/acceptance/test_pipeline_acceptance.py::TestAlabamaVRAAcceptance::test_all_1437_tracts_assigned`

**Status:** ENFORCED

---

## CI-03: District count matches constitutional target

**Invariant:** The number of districts produced equals num_districts from the state configuration (which matches state law and census apportionment).

**Why it must hold:** Each state's congressional delegation size is set by law (apportionment after each decennial census). Producing more or fewer districts is unconstitutional.

**When it can be violated:** A bug in the bisection tree (wrong k_left+k_right sum) or leaf assignment that produces fewer leaves than k.

**Enforcement:** `len(set(assignments.values())) == num_districts` is implicitly verified by the acceptance tests that check known district counts (VT=1, AL=7).

**Test:** `tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance::test_vt_rust_one_district`
`tests/acceptance/test_pipeline_acceptance.py::TestRustCLIAcceptance::test_al_rust_mm_count` (implies 7 districts)

**Status:** ENFORCED
