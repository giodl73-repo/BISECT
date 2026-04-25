---
name: review-tests
description: Audit the test suite through the BENCHMARK lens. Finds stale assertions, vacuous mocks, missing coverage for known bugs, CI-incompatible tests, and tests that pass but shouldn't. Run after any bug fix to ensure a test now exists that would have caught it.
user_invocable: true
---

# Test Suite Review (BENCHMARK)

Walk to the ground truth point. BENCHMARK reviews tests the way a surveyor verifies a benchmark marker: is it still where it was placed? Is it at the right elevation? Would it catch an error if the map were wrong?

## Input

Specify scope:
- `unit` — tests/unit/*.py
- `integration` — tests/integration/*.py
- `vra` — VRA-specific tests only
- `pipeline` — pipeline parameter propagation tests
- `all` — full suite (default)
- A specific file: `tests/unit/test_vra_edge_weighting.py`

## Steps

### 1. Load the test files

For each test file in scope, read it completely. Build a list of:
- Every test function and what it claims to test (from the docstring or name)
- Every assertion and what value it expects
- Every mock and what it replaces
- Every `pytest.skip` or `pytest.xfail` and why

### 2. BENCHMARK check — would these tests have caught today's bugs?

Today's known bug categories. For each, find the test that covers it (or flag that none exists):

**Pipeline flag propagation bugs:**
- `--version` defaulting to `v1` in child scripts → test should verify version flows through chain
- `--reset` not firing from `--reprocess` → test should verify reprocess causes redistricting to re-run
- `--run-analysis` not propagating from run_complete → test should verify analysis skips when flag absent
- `--skip-political`/`--skip-demographic` dropped → test should verify political/demographic skips work

**VRA invariant bugs:**
- `vra_mode` cleared prematurely → test: `test_vra_mode_clears_after_setup` (exists in test_vra_edge_weighting.py)
- `multi_constraint=vra_mode` instead of `=(vra_target_tree is not None)` → test: covered
- Population imbalance in high-diversity states (CA, TX) → test: `test_california_vra_population_balance` (exists)
- `vra_analysis.pkl` not saved → test: should verify file exists after VRA run

**Test staleness bugs:**
- `get_cbsa_url(2000)` expected to raise (but 2000 was added) → fixed
- `parse_census_worker_complete` missing `worker_id`/`state_code` fields → fixed
- `test_equal_split_no_targets` expecting 40-60% minority without guidance → fixed

For each bug category, report:
```
Bug: {description}
Test exists: YES / NO
Test name: {if yes}
Would it have caught the bug before the fix: YES / NO / UNCERTAIN
```

### 3. Stale assertion audit

For every hardcoded expected value, check:
- Is the expected value documented with WHY? (e.g., `0.367` should say "Paper B.2 result")
- Has the underlying code changed in a way that might shift the value?
- Is the value from a specific pipeline run that might not be current?

Flag format:
```
**BENCHMARK [STALE RISK]:** {test}:{line}
Assertion: `{code}`
Risk: {why this might drift}
Fix: {add comment explaining why this value, or load from current output}
```

### 4. Vacuous mock audit

For each test that uses `@patch` or `MagicMock`:
- Does the test verify behavior, or just verify the mock was called?
- If the mocked function were replaced with a completely different implementation, would the test still pass?
- Is the mock hiding a real dependency that could fail (e.g., METIS binary, census file)?

Flag format:
```
**BENCHMARK [VACUOUS]:** {test}
Evidence: {test mocks X and only asserts mock_X.called}
Fix: {test actual output/behavior instead, or use real dependency with skipif}
```

### 5. CI compatibility audit

Tests must pass on GitHub Actions (ubuntu-latest) without pipeline outputs. Check:

- Does any test load from `outputs/V3/` or `outputs/V4/` without a `pytest.skip` guard?
- Does any test call gpmetis without checking `find_gpmetis_executable() is not None`?
- Does any integration test run the full pipeline (>5 min timeout risk)?
- Are all integration tests in `tests/integration/` (not `tests/unit/`)?

### 6. Coverage gap analysis

For each critical code path, verify a test exists:

| Code path | Test location | Status |
|-----------|--------------|--------|
| VRA edge weight construction | test_vra_edge_weighting.py::TestEdgeWeightConstruction | ✓ |
| Adaptive boost formula | test_vra_edge_weighting.py::TestAdaptiveBoostScaling | ✓ |
| Population balance in VRA mode | test_vra_pipeline_balance.py | ✓ |
| California/Texas (high diversity) balance | test_vra_pipeline_balance.py | ✓ |
| --version propagation through chain | ??? | MISSING |
| --reprocess fires --reset | ??? | MISSING |
| --run-analysis skip propagation | ??? | MISSING |
| vra_analysis.pkl saved when vra_mode=True | ??? | MISSING |
| 2010 CBSA URL invalid year | test_download_sources.py | ✓ (1990) |
| Temporal stability across decades | test_temporal_stability.py | ✓ |

### 7. Summary

```
BENCHMARK REVIEW — {scope}
Files reviewed: N
Tests reviewed: N

GROUND TRUTH GAPS (tests that should exist but don't):
1. {description — what bug would be uncaught}

STALE ASSERTIONS:
1. {test:line — what might drift and why}

VACUOUS MOCKS:
1. {test — what it isn't actually testing}

CI INCOMPATIBILITIES:
1. {test — why it would fail on ubuntu-latest without outputs}

COVERAGE:
  Known bug categories covered: N/M
  Critical paths covered: N/M

RECOMMENDED ACTIONS:
1. Add test for {specific bug category} in {tests/unit/ or tests/integration/}
2. Document expected value {X} in {test:line} with source
```

## Key Rules

- **The hardest question is "what test would have caught today's bug?"** If the answer is "none," that is the first thing to fix.
- **A test that always passes is worthless.** Temporarily break the code being tested and verify the test fails. If it doesn't, it's not testing what it claims.
- **Magic numbers must be documented.** `assert max_dev <= 0.005` is correct. `assert max_dev <= 0.005  # ±0.5% constitutional standard` is good. `assert value == 0.367` with no comment is dangerous.
- **Integration tests need skip guards.** Every integration test that reads from `outputs/` must have `if not Path('outputs/V3/...').exists(): pytest.skip(...)`.
- **CI runs on ubuntu-latest.** gpmetis is installed via `apt-get install metis`. Tests that require it must use `find_gpmetis_executable()` to detect availability.
- **BENCHMARK is position 4 in the tiebreaker chain.** If the tests don't cover it, we can't trust anything downstream.
