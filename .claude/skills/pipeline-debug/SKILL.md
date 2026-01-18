---
name: pipeline-debug
description: Systematically debug pipeline failures by analyzing error messages, checking common issues, and suggesting fixes. Use when redistricting pipeline fails, encounters errors, or produces unexpected results.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Pipeline Debugging Skill

Systematically diagnose + fix redistricting pipeline failures via knowledge from 18+ enhancements + common error patterns.

## When to Use
User says "Pipeline failed/Why did redistricting crash", pipeline produces errors/unexpected results, scripts hang/run indefinitely, output files missing/corrupted

## Workflow

### Step 1: Identify Failure Stage
Read error messages, determine where failure occurred:

**Redistricting**: Loading census data, loading adjacency graphs, METIS partitioning, saving district assignments
**Analysis**: Compactness calculation, political analysis, demographic analysis, map generation
**Post-Processing**: National aggregation, metro area maps, dashboard generation

### Step 2: Check Common Issues

**Issue 1 - Missing Data Files**:
```
FileNotFoundError: data/tracts/2020/california_tracts_2020.parquet
```
**Diagnosis**: `ls data/tracts/2020/` + `ls data/adjacency/2020/`
**Fix**: `/census-download` for tract data, `/adjacency-build` for graphs, verify year parameter

**Issue 2 - GEOID Type Mismatches**:
```
TypeError: Cannot compare 'str' and 'int' for GEOID
KeyError: GEOID '06001400100' not found (but '6001400100' exists)
```
**Diagnosis**: Check types: `df['GEOID'].dtype` (should be 'object'/string)
**Fix**: Force string on read: `pd.read_csv(file, dtype={'GEOID': str})` or convert: `df['GEOID'] = df['GEOID'].astype(str).str.zfill(11)`

**Issue 3 - Graph Connectivity Failures**:
```
Error: Graph has 2 connected components for hawaii
Error: METIS requires single connected component
```
**Diagnosis**: `python scripts/data/geography/check_graph_connectivity.py --year 2020 --state hawaii`
**Fix**: Rebuild adjacency with water connections, check isolated island tracts, verify all tracts have neighbor

**Issue 4 - Unicode Encoding (Windows)**:
```
UnicodeEncodeError: 'charmap' codec can't encode '\u2713'
```
**Diagnosis**: Check for Unicode in prints: ✓✗→•
**Fix**: Replace with ASCII: `[OK]` not ✓, `[FAIL]` not ✗, `->` not →
⚠️ **Code bug** - scripts must use ASCII for Windows compatibility

**Issue 5 - METIS Errors**:
```
Error: Edge weight overflow
Error: METIS segmentation fault
```
**Diagnosis**: Check max edge weight: `max(graph_data['edge_weights'].values())/1000` km
**Fix**: Weights >100km may overflow, check data errors (incorrect geometries), consider edge weight scaling

**Issue 6 - Memory Errors**:
```
MemoryError: Unable to allocate array
Process killed (out of memory)
```
**Diagnosis**: Check memory: `free -h` (Linux) / `wmic OS get FreePhysicalMemory` (Win)
**Fix**: Close apps, process states individually (`--states`), reduce DPI (`--dpi 100` not 300), use block-level data only when necessary

**Issue 7 - Path Not Found**:
```
FileNotFoundError: outputs/v1/2020/states/new_york/data/districts.csv
```
**Diagnosis**: Check output directory structure:
- Production: `outputs/v1/2020/states/` (default `--run-type production`)
- Test/Dev: `outputs/dev/v1_2020/states/` (use `--run-type test`)
- Experiment: `outputs/experiments/{name}/v1_2020/states/`
**Fix**: Verify correct `--run-type`, lowercase_underscores state names, `--version` matches, redistricting completed for state

### Step 3: Test Fix with Small State
After applying fix, test with small state using `run_test.bat` (outputs to `dev/`):
```bash
# Windows
run_test.bat -y 2020 -v debug_test --states "VT"
runtest -y 2020 -v debug_test --states "VT"           # Short: doskey alias

# Direct Python call
python scripts/pipeline/run_complete_redistricting.py \
  -y 2020 -v debug_test -rt test --states "VT"
```
**Output location**: `outputs/dev/debug_test_2020/` (keeps test runs organized)
If Vermont succeeds → test full pipeline

### Step 4: Check Known Issues
**CLAUDE.md - Common Pitfalls**: Config imports, progress bar protocols, state name formatting, line endings (CRLF on Win)
**CODING_PATTERNS.md - Anti-Patterns**: Hardcoded years, Unicode in console, missing node additions in graphs, relative vs absolute paths
**ENHANCEMENTS_2026.md**: Search for similar problems ("Issue"/"Fix" keywords)

## Debugging Checklist
- [ ] Error message copied for analysis
- [ ] Failure stage identified (redistricting/analysis/post-processing)
- [ ] Data files verified to exist
- [ ] GEOID types checked (should be strings)
- [ ] Graph connectivity validated
- [ ] Unicode characters checked (Windows)
- [ ] Memory usage checked
- [ ] Path formats verified
- [ ] Small state test attempted with `run_test.bat` (outputs to `dev/`)
- [ ] Known issues reviewed in docs

## Error Categories

**Data Errors**: Missing files → Download/create | Type mismatches → Force correct types | Corrupted files → Regenerate
**Code Errors**: Unicode → Replace with ASCII | Paths → Use Path objects | Imports → Check dependencies
**Environment Errors**: Memory → Reduce scope/DPI | Permissions → Check file permissions | Process limits → Kill zombies
**Algorithm Errors**: Graph connectivity → Rebuild with water | METIS failures → Check edge weights | Population imbalance → Verify census data

## What You'll Get
Root cause identified, specific fix recommendations, tested solution with small state, documentation for future reference

## Next Steps
After fixing: Run full pipeline validation, document fix in session notes, consider updating CODING_PATTERNS.md (if new anti-pattern), update error handling in code (if needed)
