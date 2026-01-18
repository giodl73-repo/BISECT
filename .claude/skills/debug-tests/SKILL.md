---
name: debug-tests
description: Systematically debug test failures with guided troubleshooting. Analyze pytest output, detect common failure patterns (imports, mocks, assertions, Playwright, etc.), provide step-by-step debugging guidance, and suggest specific fixes.
allowed-tools:
  - Read
  - Bash
  - Grep
  - Glob
user-invocable: true
---

# Debug Tests

Systematically debug test failures via pattern recognition + guided troubleshooting. Analyzes pytest output → identifies failure types → checks common issues → suggests specific fixes.

## Prerequisites
**Required**: Test suite with failing tests, recent pytest output (or ability to run), access to test files/fixtures
**Recommended**: `/run-tests` output for context, knowledge of recent changes, understanding of test structure

## When to Use
User says "Debug tests/Why are tests failing/Fix test failures/Tests aren't passing", after `/run-tests` reports failures, CI/CD failures, unclear errors, multiple similar failures

## Workflow

### Step 1: Gather Failure Info
**Check pytest output**: Recent messages, or ask user to run `pytest tests/ -v`, or run via Bash: `pytest tests/ -v --tb=short`
**Capture**: Failed test names, error messages/tracebacks, failure count, categories (unit/integration/E2E), patterns

### Step 2: Categorize Failures
Parse output via `Grep` for patterns:

**Pattern 1 - Import Errors**:
```
ModuleNotFoundError: No module named 'apportionment'
ImportError: cannot import name 'X'
```
**Detect**: `grep -i "ModuleNotFoundError\|ImportError" pytest_output`
**Causes**: PYTHONPATH not set, missing `__init__.py`, circular imports, package not installed

**Pattern 2 - Mock Data Errors**:
```
FileNotFoundError: outputs/us_2020_test/...
AttributeError: 'NoneType' object has no attribute 'get'
```
**Detect**: `grep -i "FileNotFoundError.*outputs\|FileNotFoundError.*tests/fixtures"`
**Causes**: Fixtures not generated, conftest.py not executed, path incorrect, scope issue

**Pattern 3 - Assertion Failures**:
```
AssertionError: assert 42 == 43
```
**Detect**: `grep "AssertionError: assert"`
**Causes**: Expected values outdated, implementation changed, mock values incorrect, test too strict

**Pattern 4 - Playwright/Browser**:
```
TimeoutError: Timeout 30000ms exceeded
Error: locator.click: Target closed
playwright._impl._api_types.Error: Executable doesn't exist
```
**Detect**: `grep -i "TimeoutError\|playwright.*Error\|Target closed"`
**Causes**: Browser not installed (`playwright install chromium`), selectors incorrect, page slow, mock data not loading

**Pattern 5 - File Not Found (Data)**:
```
FileNotFoundError: data/tracts/2020/california_tracts_2020.parquet
```
**Detect**: `grep "FileNotFoundError.*data/"`
**Causes**: Real data referenced (should use mocks), path incorrect, year-specific issues, data not downloaded

**Pattern 6 - AttributeError (API Mismatch)**:
```
AttributeError: 'DataFrame' object has no attribute 'foo'
AttributeError: module 'X' has no attribute 'Y'
```
**Detect**: `grep "AttributeError:"`
**Causes**: Mocks incomplete, API changed but tests not updated, pandas/GeoPandas version mismatch, typos

**Report**:
```
Failure Analysis:
- Import Errors: 5 tests
- Mock Data Errors: 0 tests
- Assertion Failures: 2 tests
- Playwright Errors: 0 tests
- File Not Found: 0 tests
- AttributeError: 0 tests
- Other: 1 test
```

### Step 3: Guided Debugging

**Import Errors**:
1. Check PYTHONPATH: `echo $PYTHONPATH` / `echo %PYTHONPATH%` (should include `C:\src\apportionment`)
2. Set if not: `export PYTHONPATH="${PYTHONPATH}:$(pwd)"` (Unix) / `set PYTHONPATH=%PYTHONPATH%;%CD%` (Win)
3. Check structure: Glob for `__init__.py`, verify `apportionment/__init__.py` exists
4. Check circular: Review recent import changes (A imports B, B imports A)
5. Re-run: `pytest tests/ -v`

**Mock Data Errors**:
1. Check fixture: Verify `tests/e2e/conftest.py` has `mock_run` (session-scoped)
2. Generate manually: `python tests/fixtures/generate_mock_run.py --states "vermont,alabama" --year 2020 --version test`
3. Check exists: Glob for mock files, `ls outputs/us_2020_test/states/vermont/`
4. Verify usage: Test functions have `mock_run` parameter, fixture returns correct path
5. Re-run debug: `pytest tests/e2e/ -v -s` (show prints)

**Assertion Failures**:
1. Show diff: Parse assertion for expected vs actual, display comparison
2. Check mock changes: Review generator changes, verify expectations match mock values
3. Run single: `pytest tests/path/to/test.py::test_name -vv`
4. Update expectations (if impl correct): Read test file, suggest new values
5. Verify impl (if test correct): Check logic, review changes, suggest fix

**Playwright Errors**:
1. Install browser: `playwright install chromium`
2. Run visible: `pytest tests/e2e/ --headed -v`
3. Slow motion: `pytest tests/e2e/ --headed --slowmo=500 -v`
4. Check selectors: Grep for selectors in test, verify match dashboard HTML
5. Check mock loading: Dashboard HTML generated correctly, paths point to mock data
6. Increase timeout: Add `page.set_default_timeout(60000)` or `page.wait_for_timeout(2000)`

**File Not Found (Data)**:
1. Check real data usage: Tests should use mocks, Grep for data path references
2. Fix to mocks: Replace paths with fixtures, use `mock_run` for E2E
3. If data needed: `/census-download` + `/adjacency-build`
4. Check year paths: Format `data/tracts/{year}/{state}_tracts_{year}.parquet`

**AttributeError**:
1. Check mock structure: Grep for mock generator, verify includes required attributes
2. Check API compat: Verify pandas/geopandas versions, check method name changes
3. Update mocks: Add missing attributes, match real data structure
4. Update tests: Use new method names, update expectations

### Step 4: Automatic Checks
Run via Bash + Grep:

**Check 1 - PYTHONPATH**: `python -c "import sys; print('\\n'.join(sys.path))"` → Verify project root
**Check 2 - Package imports**: `python -c "import apportionment; print(apportionment.__file__)"` → Should succeed
**Check 3 - Mock data**: `ls -R outputs/us_2020_test/ 2>/dev/null | wc -l` → Should show files
**Check 4 - Pytest plugins**: `pytest --version && pip list | grep pytest` → Verify pytest-playwright, pytest-cov
**Check 5 - Browser**: `playwright install --dry-run chromium` → Check if needs install

**Report**:
```
Automatic Checks:
[OK] PYTHONPATH includes project root
[OK] apportionment package importable
[FAIL] Mock data not found in outputs/
[OK] pytest-playwright installed
[WARN] Chromium browser not installed
```

### Step 5: Suggest Fixes
**Example**:
```
Recommended Fixes:
1. Import Errors (5 tests):
   Issue: PYTHONPATH not set
   Fix: export PYTHONPATH="${PYTHONPATH}:$(pwd)"
   Then: Re-run tests

2. Assertion Failures (2 tests):
   Issue: Expected 42, got 43
   Fix: Update tests/unit/test_political_analysis.py:45
   Change: assert result == 42 → assert result == 43
   Verify: Implementation correct

3. Playwright Browser:
   Issue: Chromium not installed
   Fix: playwright install chromium
   Then: Re-run E2E tests

Priority: Fix PYTHONPATH first (5 tests), then browser (E2E)
```

### Step 6: Offer Targeted Re-test
```bash
pytest --lf -v                                           # Re-run failed only
pytest tests/unit/test_political_analysis.py::test_seat_calculation -vv  # Specific test
pytest --lf --pdb -v                                     # With debugger
```

**Monitor**: Failures reduce → report progress, continue debugging | persist → deeper investigation | new failures → analyze new patterns

## Pattern Detection Reference

| Pattern | Grep | Fix |
|---------|------|-----|
| Import Error | `grep -i "ModuleNotFoundError\|ImportError"` | Set PYTHONPATH |
| Mock Data | `grep "FileNotFoundError.*outputs"` | Generate mock |
| Assertion | `grep "AssertionError: assert"` | Update expectations |
| Playwright | `grep -i "TimeoutError\|playwright.*Error"` | Install browser |
| File Not Found | `grep "FileNotFoundError.*data/"` | Use mocks |
| AttributeError | `grep "AttributeError:"` | Update mocks/API |

## Quick Fix Commands
```bash
export PYTHONPATH="${PYTHONPATH}:$(pwd)"                  # PYTHONPATH (Unix)
set PYTHONPATH=%PYTHONPATH%;%CD%                          # PYTHONPATH (Win)
python tests/fixtures/generate_mock_run.py --states "vermont,alabama" --year 2020 --version test  # Mock data
playwright install chromium                               # Browser
pytest --lf -v                                           # Re-run failed
pytest --lf --pdb -v                                     # With debugger
pytest --lf -v --tb=long                                 # Full traceback
pytest --lf -v -s                                        # Show prints
```

## Troubleshooting Debugger

**No pytest output**: Run `/run-tests` or `pytest tests/ -v > pytest_output.txt 2>&1`
**Can't identify pattern**: Show raw error, use `--tb=long`, manual investigation
**Fixes don't work**: Check multiple issues, fix one category at a time, use `pytest --pdb`
**E2E still failing**: Check selectors with `--headed`, verify mock data, check dashboard HTML

## Advanced Debugging

**Interactive (pdb)**:
```bash
pytest --pdb -v  # Stop at first failure
# pdb: l (list code), p variable (print), c (continue), q (quit)
```

**Specific tests**:
```bash
pytest tests/unit/test_file.py::test_name -vv        # Single test
pytest tests/unit/test_file.py -v                    # File
pytest tests/unit/test_file.py::TestClassName -v     # Class
pytest -m redistricting -v                           # Marker
```

**Mock fixtures**:
```bash
pytest tests/e2e/ -v --setup-show    # Show fixture setup/teardown
pytest --fixtures                    # List fixtures
pytest --fixtures | grep mock        # Mock fixtures
```

## What You'll Get
Failure categorization by pattern, guided troubleshooting steps per category, automatic checks for common issues, specific fix suggestions with commands, re-test recommendations, progress tracking

## Next Steps

**Resolved**: Run full suite (`/run-tests`) to verify, commit fixes with clear message, document test changes
**Some remain**: Focus one category at a time, use `pytest --pdb`, check for multiple issues, review recent changes
**Stuck**: Review `tests/README.md`, check git history, run single test in isolation, consider reverting changes
**Prevent future**: Run tests before commit, use `/run-tests` in dev workflow, keep expectations updated, review CI/CD failures promptly

## Related Skills
`/run-tests` (execute suite), `/enhancement-implement` (testing phase), `/pipeline-debug` (pipeline failures)

## Examples

**Ex 1 - Import**: User "Why failing?" → Read output (5 import errors) → Categorize: ModuleNotFoundError → Check: PYTHONPATH not set → Fix: `export PYTHONPATH` → Re-run: All pass

**Ex 2 - Mock Data**: User "E2E can't find mock" → Read: FileNotFoundError in outputs/ → Categorize: Mock data → Check: Fixtures empty → Fix: `python tests/fixtures/generate_mock_run.py` → Verify: Data exists → Re-run: Pass

**Ex 3 - Playwright**: User "Dashboard timeout" → Read: TimeoutError → Categorize: Browser errors → Check: Not installed → Fix: `playwright install chromium` → Additional: `--headed` → Re-run: Pass

**Ex 4 - Assertion**: User "Wrong values" → Read: assert 42 == 43 → Categorize: Assertion → Analyze: Mock generates 43, test expects 42 → Check: Impl correct, test outdated → Fix: Update 42→43 → Pass

**Ex 5 - Multiple**: User "Lots failing" → Read: 18 failures → Categorize: 10 import (PYTHONPATH), 5 mock, 3 assertion → Priority: PYTHONPATH first → Fix: Set PYTHONPATH → Re-run: 8 remain → Fix: Generate mock → Re-run: 3 remain → Analyze: Update tests → Resolved
