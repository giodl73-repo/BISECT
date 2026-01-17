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

## Overview

Systematically debug test failures using pattern recognition and guided troubleshooting. This skill analyzes pytest output to identify failure types, checks for common issues automatically, provides step-by-step debugging guidance, and suggests specific fixes based on error patterns.

## Prerequisites

**Required**:
- Test suite with failing tests
- Recent pytest output (or ability to run tests)
- Access to test files and fixtures

**Recommended**:
- `/run-tests` skill output for context
- Knowledge of recent code changes
- Understanding of test structure

## When to Use This Skill

- User says: "Debug tests"
- User says: "Why are my tests failing?"
- User says: "Fix test failures"
- User says: "Tests aren't passing"
- After `/run-tests` reports failures
- When tests fail in CI/CD
- When test errors are unclear
- When multiple tests fail with similar errors

## Workflow

### Step 1: Gather Test Failure Information

**Check for recent pytest output**:
- Look for pytest output in recent messages
- If not found, ask user to run tests: "Run `pytest tests/ -v` and share output"
- Or offer to run tests using Bash: `pytest tests/ -v --tb=short`

**What to capture**:
- Failed test names
- Error messages and tracebacks
- Number of failures
- Test categories affected (unit/integration/E2E)
- Any patterns in failure locations

### Step 2: Categorize Failures

Parse output to identify failure types using `Grep` for pattern matching:

#### Pattern 1: Import Errors
**Symptoms**:
```
ModuleNotFoundError: No module named 'apportionment'
ImportError: cannot import name 'X' from 'Y'
```

**Detection**:
```bash
grep -i "ModuleNotFoundError\|ImportError" pytest_output
```

**Common causes**:
- PYTHONPATH not set correctly
- Missing `__init__.py` files
- Circular imports
- Package not installed

#### Pattern 2: Mock Data Errors
**Symptoms**:
```
FileNotFoundError: [Errno 2] No such file or directory: 'outputs/us_2020_test/...'
AttributeError: 'NoneType' object has no attribute 'get'
```

**Detection**:
```bash
grep -i "FileNotFoundError.*outputs\|FileNotFoundError.*tests/fixtures" pytest_output
```

**Common causes**:
- Mock fixtures not generated
- conftest.py fixture not executed
- Mock data path incorrect
- Session fixture scope issue

#### Pattern 3: Assertion Failures
**Symptoms**:
```
AssertionError: assert 42 == 43
AssertionError: assert 'expected' == 'actual'
```

**Detection**:
```bash
grep "AssertionError: assert" pytest_output
```

**Common causes**:
- Expected values outdated
- Implementation logic changed
- Mock data values incorrect
- Test expectations too strict

#### Pattern 4: Playwright/Browser Errors
**Symptoms**:
```
TimeoutError: Timeout 30000ms exceeded
Error: locator.click: Target closed
playwright._impl._api_types.Error: Executable doesn't exist
```

**Detection**:
```bash
grep -i "TimeoutError\|playwright.*Error\|Target closed" pytest_output
```

**Common causes**:
- Browser not installed (`playwright install chromium`)
- Element selectors incorrect/outdated
- Page load too slow
- Mock data not loading properly

#### Pattern 5: File Not Found (Data)
**Symptoms**:
```
FileNotFoundError: data/tracts/2020/california_tracts_2020.parquet
```

**Detection**:
```bash
grep "FileNotFoundError.*data/" pytest_output
```

**Common causes**:
- Real data files referenced (should use mocks)
- Data path incorrect for test environment
- Year-specific path issues
- Data not downloaded

#### Pattern 6: AttributeError (API Mismatch)
**Symptoms**:
```
AttributeError: 'DataFrame' object has no attribute 'foo'
AttributeError: module 'X' has no attribute 'Y'
```

**Detection**:
```bash
grep "AttributeError:" pytest_output
```

**Common causes**:
- Mock objects incomplete
- API changed but tests not updated
- Pandas/GeoPandas version mismatch
- Method name typos

**Report categories found**:
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

### Step 3: Provide Guided Debugging Steps

For each failure category, provide specific troubleshooting:

#### For Import Errors

**Step-by-step debugging**:
1. **Check PYTHONPATH**:
   ```bash
   echo $PYTHONPATH  # Unix
   echo %PYTHONPATH%  # Windows
   ```
   Should include project root: `C:\src\apportionment`

2. **Set PYTHONPATH** (if not set):
   ```bash
   export PYTHONPATH="${PYTHONPATH}:$(pwd)"  # Unix
   set PYTHONPATH=%PYTHONPATH%;%CD%  # Windows
   ```

3. **Check package structure**:
   ```bash
   # Use Glob to find __init__.py files
   ```
   Verify `apportionment/__init__.py` exists

4. **Check for circular imports**:
   - Review recent changes to import statements
   - Look for A imports B, B imports A patterns

5. **Re-run tests**:
   ```bash
   pytest tests/ -v
   ```

#### For Mock Data Errors

**Step-by-step debugging**:
1. **Check fixture execution**:
   - Verify `tests/e2e/conftest.py` has `mock_run` fixture
   - Verify fixture is session-scoped

2. **Manually generate mock data**:
   ```bash
   python tests/fixtures/generate_mock_run.py --states "vermont,alabama" --year 2020 --version test
   ```

3. **Check mock data exists**:
   ```bash
   # Use Glob to find mock data files
   ls outputs/us_2020_test/states/vermont/
   ```

4. **Verify fixture usage**:
   - Check test functions have `mock_run` parameter
   - Verify fixture returns correct path

5. **Re-run tests with fixture debugging**:
   ```bash
   pytest tests/e2e/ -v -s  # Show print statements
   ```

#### For Assertion Failures

**Step-by-step debugging**:
1. **Show expected vs actual**:
   - Parse assertion error for values
   - Display side-by-side comparison

2. **Check if mock data changed**:
   - Review mock generator changes
   - Verify test expectations match mock values

3. **Run single test with verbose output**:
   ```bash
   pytest tests/path/to/test.py::test_name -vv
   ```

4. **Update test expectations** (if implementation correct):
   - Use Read to view test file
   - Suggest new expected values based on actual

5. **Verify implementation** (if test correct):
   - Check code logic
   - Review recent changes
   - Suggest fix if logic error found

#### For Playwright Errors

**Step-by-step debugging**:
1. **Check browser installation**:
   ```bash
   playwright install chromium
   ```

2. **Run with visible browser**:
   ```bash
   pytest tests/e2e/ --headed -v
   ```

3. **Run with slow motion**:
   ```bash
   pytest tests/e2e/ --headed --slowmo=500 -v
   ```

4. **Check element selectors**:
   - Use Grep to find selectors in test file
   - Verify selectors match dashboard HTML

5. **Check mock data loading**:
   - Ensure dashboard HTML generated correctly
   - Verify paths in dashboard point to mock data

6. **Increase timeout**:
   - Add `page.set_default_timeout(60000)` to test
   - Or use `page.wait_for_timeout(2000)` before action

#### For File Not Found (Data)

**Step-by-step debugging**:
1. **Check if test uses real data**:
   - Tests should use mocks, not real data
   - Use Grep to find data path references

2. **Fix test to use mocks**:
   - Replace data paths with mock fixtures
   - Use `mock_run` fixture for E2E tests

3. **If data needed**:
   - Download with `/census-download` skill
   - Build adjacency with `/adjacency-build` skill

4. **Check year-specific paths**:
   - Verify path format: `data/tracts/{year}/{state}_tracts_{year}.parquet`
   - Not: `data/raw/{state}_tracts_2020.parquet`

#### For AttributeError

**Step-by-step debugging**:
1. **Check mock object structure**:
   - Use Grep to find mock generator
   - Verify mock includes required attributes

2. **Check API compatibility**:
   - Verify pandas/geopandas versions
   - Check if method names changed

3. **Update mocks** (if needed):
   - Add missing attributes to mock generators
   - Match real data structure

4. **Update tests** (if API changed):
   - Use new method names
   - Update test expectations

### Step 4: Check Common Issues Automatically

Run automatic checks using Bash and Grep:

**Check 1: PYTHONPATH**
```bash
python -c "import sys; print('\\n'.join(sys.path))"
```
Verify project root in path

**Check 2: Package imports**
```bash
python -c "import apportionment; print(apportionment.__file__)"
```
Should succeed without error

**Check 3: Mock data existence**
```bash
ls -R outputs/us_2020_test/ 2>/dev/null | wc -l
```
Should show files if mock run generated

**Check 4: Pytest plugins**
```bash
pytest --version
pip list | grep pytest
```
Verify pytest-playwright, pytest-cov installed

**Check 5: Browser installation**
```bash
playwright install --dry-run chromium
```
Check if browser needs installation

**Report findings**:
```
Automatic Checks:
[OK] PYTHONPATH includes project root
[OK] apportionment package importable
[FAIL] Mock data not found in outputs/
[OK] pytest-playwright installed
[WARN] Chromium browser not installed
```

### Step 5: Suggest Specific Fixes

Based on analysis, provide actionable fix suggestions:

**Example output**:
```
Recommended Fixes:

1. Import Errors (5 tests):
   Issue: PYTHONPATH not set
   Fix: export PYTHONPATH="${PYTHONPATH}:$(pwd)"
   Then: Re-run tests

2. Assertion Failures (2 tests):
   Issue: Expected value 42, got 43
   Fix: Update test expectation in tests/unit/test_political_analysis.py:45
   Change: assert result == 42
   To: assert result == 43
   Verify: Implementation logic is correct

3. Playwright Browser (E2E tests):
   Issue: Chromium not installed
   Fix: playwright install chromium
   Then: Re-run E2E tests

Priority: Fix PYTHONPATH first (affects 5 tests), then browser (affects E2E)
```

### Step 6: Offer Targeted Re-test

After suggesting fixes:

**Offer to re-run failed tests**:
```bash
# Re-run only previously failed tests
pytest --lf -v

# Re-run specific test with verbose output
pytest tests/unit/test_political_analysis.py::test_seat_calculation -vv

# Re-run with debugger (stops on failure)
pytest --lf --pdb -v
```

**Monitor for improvement**:
- If failures reduce: Report progress, continue debugging remaining
- If failures persist: Suggest deeper investigation or manual debugging
- If new failures appear: Analyze new error patterns

## Common Failure Patterns Reference

### Pattern Detection Quick Reference

| Pattern | Grep Command | Common Fix |
|---------|--------------|------------|
| Import Error | `grep -i "ModuleNotFoundError\|ImportError"` | Set PYTHONPATH |
| Mock Data | `grep "FileNotFoundError.*outputs"` | Generate mock data |
| Assertion | `grep "AssertionError: assert"` | Update expectations |
| Playwright | `grep -i "TimeoutError\|playwright.*Error"` | Install browser |
| File Not Found | `grep "FileNotFoundError.*data/"` | Use mocks not real data |
| AttributeError | `grep "AttributeError:"` | Update mocks/API |

### Quick Fix Commands

```bash
# Fix PYTHONPATH (Unix)
export PYTHONPATH="${PYTHONPATH}:$(pwd)"

# Fix PYTHONPATH (Windows)
set PYTHONPATH=%PYTHONPATH%;%CD%

# Generate mock data
python tests/fixtures/generate_mock_run.py --states "vermont,alabama" --year 2020 --version test

# Install Playwright browser
playwright install chromium

# Re-run failed tests only
pytest --lf -v

# Run with debugger
pytest --lf --pdb -v

# Run with full traceback
pytest --lf -v --tb=long

# Show print statements
pytest --lf -v -s
```

## Troubleshooting the Debugger

**No pytest output available**:
```
Solution: Run tests first with /run-tests or manually:
          pytest tests/ -v > pytest_output.txt 2>&1
```

**Can't identify failure pattern**:
```
Solution: Show raw error output
          Use --tb=long for full traceback
          Manual investigation may be needed
```

**Fixes don't resolve failures**:
```
Solution: Check if multiple issues present
          Fix one category at a time
          Consider using pytest --pdb for interactive debugging
```

**E2E tests still failing after browser install**:
```
Solution: Check element selectors with --headed mode
          Verify mock data generated correctly
          Check dashboard HTML exists
```

## Advanced Debugging

### Interactive Debugging with pdb

```bash
# Stop at first failure
pytest --pdb -v

# Common pdb commands:
# l (list) - Show code context
# p variable - Print variable value
# c (continue) - Continue execution
# q (quit) - Exit debugger
```

### Debugging Specific Tests

```bash
# Single test with verbose
pytest tests/unit/test_file.py::test_name -vv

# Single test file
pytest tests/unit/test_file.py -v

# Test class
pytest tests/unit/test_file.py::TestClassName -v

# Test by marker
pytest -m redistricting -v
```

### Debugging Mock Fixtures

```bash
# Show fixture setup/teardown
pytest tests/e2e/ -v --setup-show

# List available fixtures
pytest --fixtures

# Use specific fixture
pytest tests/e2e/ -v --fixtures | grep mock
```

## What You'll Get

After successful debugging:
- **Failure categorization** by pattern type
- **Guided troubleshooting steps** for each category
- **Automatic checks** for common issues
- **Specific fix suggestions** with commands
- **Re-test recommendations** to verify fixes
- **Progress tracking** as failures are resolved

## Next Steps

After debugging:

**If failures resolved**:
- Run full test suite with `/run-tests` to verify
- Commit fixes with clear message
- Document any test changes made

**If some failures remain**:
- Focus on one category at a time
- Use interactive debugging: `pytest --pdb`
- Check for multiple simultaneous issues
- Review recent code changes

**If stuck**:
- Review test documentation in `tests/README.md`
- Check for similar issues in git history
- Run single test in isolation
- Consider reverting recent changes

**To prevent future failures**:
- Run tests before committing
- Use `/run-tests` as part of development workflow
- Keep test expectations up to date
- Review CI/CD failures promptly

## Related Skills

- `/run-tests` - Execute test suite (typically run before debugging)
- `/enhancement-implement` - Includes testing phase
- `/pipeline-debug` - Debug pipeline failures (different from tests)

## Examples

### Example 1: Debug Import Errors

**User**: "Why are my tests failing?"

**Skill actions**:
1. Read pytest output (5 import errors)
2. Categorize: All ModuleNotFoundError
3. Check: PYTHONPATH not set
4. Suggest: `export PYTHONPATH="${PYTHONPATH}:$(pwd)"`
5. Offer: Re-run with `pytest --lf -v`
6. Result: All tests now pass

### Example 2: Debug Mock Data Issues

**User**: "E2E tests can't find mock data"

**Skill actions**:
1. Read pytest output (FileNotFoundError in outputs/)
2. Categorize: Mock data errors
3. Check: Mock fixtures directory empty
4. Suggest: `python tests/fixtures/generate_mock_run.py --states "vermont,alabama" --year 2020 --version test`
5. Verify: Mock data now exists
6. Re-run: `pytest tests/e2e/ -v`
7. Result: E2E tests pass

### Example 3: Debug Playwright Failures

**User**: "Dashboard tests timing out"

**Skill actions**:
1. Read pytest output (TimeoutError)
2. Categorize: Playwright browser errors
3. Check: Browser not installed
4. Suggest: `playwright install chromium`
5. Additional: Run with `--headed` to see browser
6. Re-run: `pytest tests/e2e/ -v`
7. Result: Tests pass

### Example 4: Debug Assertion Failures

**User**: "Tests expect wrong values"

**Skill actions**:
1. Read pytest output (AssertionError: assert 42 == 43)
2. Categorize: Assertion failures
3. Analyze: Mock data generates 43, test expects 42
4. Suggest: Update test expectation or verify implementation
5. Check: Implementation correct, test outdated
6. Fix: Update test from 42 to 43
7. Result: Test passes

### Example 5: Multiple Failure Types

**User**: "Lots of tests failing"

**Skill actions**:
1. Read pytest output (18 failures)
2. Categorize:
   - 10 import errors (PYTHONPATH)
   - 5 mock data errors
   - 3 assertion failures
3. Priority: Fix PYTHONPATH first
4. Fix: Set PYTHONPATH
5. Re-run: 8 failures remain (mock + assertions)
6. Fix: Generate mock data
7. Re-run: 3 failures remain (assertions)
8. Analyze: Tests need updating
9. Result: All resolved
