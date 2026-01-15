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

## Overview

Systematically diagnose and fix redistricting pipeline failures using knowledge from 18+ enhancements and common error patterns.

## When to Use This Skill

- User says: "The pipeline failed"
- User says: "Why did redistricting crash?"
- Pipeline produces errors or unexpected results
- Scripts hang or run indefinitely
- Output files missing or corrupted

## Workflow

### Step 1: Identify Failure Stage

Read error messages and determine where failure occurred:

**Redistricting Stage**:
- Loading census data
- Loading adjacency graphs
- METIS partitioning
- Saving district assignments

**Analysis Stage**:
- Compactness calculation
- Political analysis
- Demographic analysis
- Map generation

**Post-Processing Stage**:
- National aggregation
- Metro area maps
- Dashboard generation

### Step 2: Check Common Issues

Run diagnostic checks based on failure stage:

#### Issue 1: Missing Data Files

**Symptoms**:
```
FileNotFoundError: data/tracts/2020/california_tracts_2020.parquet not found
```

**Diagnosis**:
```bash
# Check if tract data exists
ls data/tracts/2020/

# Check if adjacency graphs exist
ls data/adjacency/2020/
```

**Solutions**:
- Run `/census-download` to get missing tract data
- Run `/adjacency-build` to create missing graphs
- Verify year parameter matches available data

#### Issue 2: GEOID Type Mismatches

**Symptoms**:
```
TypeError: Cannot compare types 'str' and 'int' for GEOID
KeyError: GEOID '06001400100' not found (but '6001400100' exists)
```

**Diagnosis**:
```python
# Check GEOID types in files
import pandas as pd
df = pd.read_csv('districts.csv')
print(df['GEOID'].dtype)  # Should be 'object' (string)
```

**Solutions**:
```python
# Force GEOID as string when reading
df = pd.read_csv(file, dtype={'GEOID': str})

# Or convert after loading
df['GEOID'] = df['GEOID'].astype(str).str.zfill(11)
```

#### Issue 3: Graph Connectivity Failures

**Symptoms**:
```
Error: Graph has 2 connected components for hawaii
Error: METIS requires single connected component
```

**Diagnosis**:
```bash
# Check graph connectivity
python scripts/data/geography/check_graph_connectivity.py --year 2020 --state hawaii
```

**Solutions**:
- Rebuild adjacency with water connections enabled
- Check for isolated island tracts
- Verify all tracts have at least one neighbor

#### Issue 4: Unicode Encoding Errors (Windows)

**Symptoms**:
```
UnicodeEncodeError: 'charmap' codec can't encode character '\u2713'
```

**Diagnosis**:
Check for Unicode characters in print statements: ✓, ✗, →, •

**Solutions**:
```python
# Replace Unicode with ASCII
# ❌ DON'T: print(f"✓ Success")
# ✅ DO: print(f"[OK] Success")

# ❌ DON'T: print(f"✗ Failed")
# ✅ DO: print(f"[FAIL] Failed")

# ❌ DON'T: print(f"→ Next")
# ✅ DO: print(f"-> Next")
```

This is a **code bug** - scripts must use ASCII for Windows compatibility.

#### Issue 5: METIS Errors

**Symptoms**:
```
Error: Edge weight overflow
Error: METIS segmentation fault
```

**Diagnosis**:
Check for extremely long boundaries:
```python
# Find max edge weight
import pickle
with open('adjacency.pkl', 'rb') as f:
    graph_data = pickle.load(f)
max_weight = max(graph_data['edge_weights'].values())
print(f"Max edge weight: {max_weight/1000:.1f} km")
```

**Solutions**:
- Edge weights >100km may cause overflow
- Check for data errors (incorrect geometries)
- Consider edge weight scaling if needed

#### Issue 6: Memory Errors

**Symptoms**:
```
MemoryError: Unable to allocate array
Process killed (out of memory)
```

**Diagnosis**:
```bash
# Check available memory
free -h  # Linux
wmic OS get FreePhysicalMemory  # Windows
```

**Solutions**:
- Close other applications
- Process states individually (use --states parameter)
- Reduce DPI for maps (--dpi 100 instead of 300)
- Use block-level data only when necessary

#### Issue 7: Path Not Found

**Symptoms**:
```
FileNotFoundError: outputs/us_2020_v1/states/new_york/data/districts.csv
```

**Diagnosis**:
```bash
# Check if output directory exists
ls outputs/us_2020_v1/states/

# Check state name format
# Should be lowercase with underscores: new_york not New_York
```

**Solutions**:
- Verify state names are lowercase_with_underscores
- Check --version parameter matches
- Verify redistricting completed for that state

### Step 3: Test Fix with Small State

After applying fix:
```bash
# Test with Vermont (fastest state)
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version debug_test \
  --states "VT"
```

If Vermont succeeds, test with full pipeline.

### Step 4: Check for Known Issues

Reference project documentation:

**CLAUDE.md - Common Pitfalls**:
- Config imports
- Progress bar protocols
- State name formatting
- Line endings (CRLF on Windows)

**CODING_PATTERNS.md - Anti-Patterns**:
- Hardcoded census years
- Unicode in console output
- Missing node additions in graphs
- Relative vs absolute paths

**ENHANCEMENTS_2026.md - Past Issues**:
- Search for similar problems in enhancement notes
- Look for "Issue" or "Fix" keywords

## Debugging Checklist

- [ ] Error message copied for analysis
- [ ] Failure stage identified (redistricting/analysis/post-processing)
- [ ] Data files verified to exist
- [ ] GEOID types checked (should be strings)
- [ ] Graph connectivity validated
- [ ] Unicode characters checked (Windows)
- [ ] Memory usage checked
- [ ] Path formats verified
- [ ] Small state test attempted
- [ ] Known issues reviewed in docs

## Error Categories

### Data Errors
- Missing files → Download/create data
- Type mismatches → Force correct types
- Corrupted files → Regenerate from source

### Code Errors
- Unicode issues → Replace with ASCII
- Path issues → Use Path objects
- Import errors → Check dependencies

### Environment Errors
- Memory limits → Reduce scope or DPI
- Permission errors → Check file permissions
- Process limits → Kill zombie processes

### Algorithm Errors
- Graph connectivity → Rebuild with water connections
- METIS failures → Check edge weights
- Population imbalance → Verify census data

## What You'll Get

- Identified root cause of failure
- Specific fix recommendations
- Tested solution with small state
- Documentation of issue for future reference

## Next Steps

After fixing:
- Run full pipeline validation
- Document fix in session notes
- Consider updating CODING_PATTERNS.md if new anti-pattern
- Update error handling in code if needed
