# REDIST Pitfalls Collection

Structural vulnerabilities in the redistricting system — not bug reports, but the patterns that make whole classes of bugs possible. Each pitfall describes a general failure mode, a structural solution that makes it impossible (not just unlikely), and a test that proves the solution holds.

Owned by: **TRENCH**. Grows every session. Never shrinks.

## Rule

A pitfall is solved when:
1. A structural solution exists that makes the failure mode impossible (not just mitigated)
2. A test exists that would fail if the structural solution were removed

A bug that led to a pitfall discovery is noted in the pitfall, but the pitfall itself is stated at the pattern level.

## Domains

| Domain | Prefix | File | Count |
|--------|--------|------|-------|
| Algorithm | AP | [pitfalls-algorithm.md](pitfalls-algorithm.md) | 6 |
| Pipeline | PP | [pitfalls-pipeline.md](pitfalls-pipeline.md) | 10 |
| Constitutional | CP | [pitfalls-constitutional.md](pitfalls-constitutional.md) | 2 |
| Data | DP | [pitfalls-data.md](pitfalls-data.md) | 3 |
| Research | RP | [pitfalls-research.md](pitfalls-research.md) | 2 |
| **Total** | | | **23** |

## Status

| ID | Pitfall | Status | Test |
|----|---------|--------|------|
| AP-01 | Multi-objective optimization trades constitutional constraints | **SOLVED** | test_vra_pipeline_balance.py |
| AP-02 | Flag-as-mode conflation creates invariant drift | **SOLVED** | test_vra_edge_weighting.py::TestVRACodePath |
| AP-03 | Weighting saturation — signal becomes noise at high density | **SOLVED** | test_vra_edge_weighting.py::TestAdaptiveBoostScaling |
| AP-04 | Indexing assumptions outlast their context | **SOLVED** | test_vra_edge_weighting.py::TestVRACodePath |
| AP-05 | Degenerate input to algorithm designed for N≥2 | **SOLVED** | test_vra_pipeline_balance.py::TestVRACodePathIntegrity |
| PP-01 | Subprocess flag inheritance gap | **SOLVED** | test_pipeline_flag_propagation.py |
| PP-02 | Force-rerun requires two independent mechanisms to both fire | **SOLVED** | test_pipeline_flag_propagation.py::TestReprocessPropagation |
| PP-03 | Conditional step with inversion gap | **SOLVED** | test_pipeline_flag_propagation.py::TestSkipFlags |
| CP-01 | Policy optimization competes with constitutional constraints | **SOLVED** | test_vra_pipeline_balance.py |
| CP-02 | Algorithm operating outside its valid input domain | **SOLVED** | test_vra_pipeline_balance.py::TestVRACodePathIntegrity |
| DP-01 | Population metric ambiguity across legal and algorithmic contexts | **OPEN** | — |
| DP-02 | Module context loss across subprocess boundary | **SOLVED** | (2010/2000 pipeline runs) |
| DP-03 | Identifier representation ambiguity at system boundaries | **SOLVED** | TestRustCLIAcceptance::test_vt_rust_final_assignments_exists |
| RP-01 | Threshold sensitivity presented as a point result | **OPEN** | — |
| RP-02 | Claim-to-data drift across pipeline evolution | **MITIGATED** | test_vra_compliance.py (partial) |
| PP-04 | Cross-language subprocess environment locality | **SOLVED** | TestRustCLIAcceptance — REDIST_PYTHON=sys.executable |
| PP-05 | Dual-purpose path parameter creates read/write coupling | **SOLVED** | TestRustCLIAcceptance — tmp dir + V3 adjacency |
| PP-06 | Parallel task failure invisibility via silent filter | **SOLVED** | TestRustCLIAcceptance::test_al_rust_mm_count |
| PP-07 | External tool parameter unit mismatch at boundary | **SOLVED** | TestRustCLIAcceptance::test_al_rust_population_balance |
| AP-06 | Implicit partition equality assumption in recursive bisection | **SOLVED** | TestRustCLIAcceptance::test_al_rust_population_balance |
| PP-08 | URL filename extraction without query parameter stripping | **SOLVED** | fetch.rs url.split('?').next() |
| PP-09 | In-memory download buffer OOM for variable-size files | **SOLVED** | fetch.rs::download_and_extract_zip (streaming) |
| PP-10 | Silent state omission on invalid year parameter | **SOLVED** | runner.rs::load_all_states year allowlist |

## Adding a Pitfall

Every session that discovers a new failure mode ends with a new entry. The pattern:

1. State the failure mode generically — not "the --version flag defaulted to v1" but "subprocess flag inheritance gap"
2. Describe which domain it belongs to and why
3. State the structural solution
4. Link to the test that proves it
5. Add to the status table above
