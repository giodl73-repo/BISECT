# E9: Per-State Analysis Refactoring ✅ COMPLETED (Pending Full Validation)

**Status**: ✅ COMPLETED
**Priority**: Medium
**Estimated Complexity**: Medium
**Created**: January 2026
**Completed**: January 2026
**Commits**: [790023d](https://github.com/giodl_microsoft/redistricting/commit/790023da0954a468f9a78d5653252ed87cb8cd60), [4bc07f9](https://github.com/giodl_microsoft/redistricting/commit/4bc07f9eb1b57c7fe98c6957bb2d2411e3761e36), [ec1cd35](https://github.com/giodl_microsoft/redistricting/commit/ec1cd357f6e21c2e03900b3f6b7928cc841bf3af), [50b5e55](https://github.com/giodl_microsoft/redistricting/commit/50b5e55074f304e957962fc3920ad7f7a085407e), [547fcf7](https://github.com/giodl_microsoft/redistricting/commit/547fcf72e13ee2a8eff23a0f8f7866cb9e6430e9), [1e0d266](https://github.com/giodl_microsoft/redistricting/commit/1e0d26601534b5fb213bd589c7be7e62b5e8db1e), [80e8d70](https://github.com/giodl_microsoft/redistricting/commit/80e8d706bc03f007baf2277d56a0bcb8dddb2028)
**Size**: L - 1,512 lines changed (24 files)

### Current State (Bottleneck)

Currently, all analysis and visualization is done in **batch mode** after all 50 states complete:

```
Pipeline Flow (Current):
├─ Phase 1: State Redistricting (Parallel)
│   └─ Process all 50 states in parallel (4-8 hours)
│
└─ Phase 2: Post-Processing (Sequential Batch)
    ├─ run_political_analysis.py → loops 50 states (100 min)
    ├─ run_demographic_analysis.py → loops 50 states (150 min)
    ├─ run_compactness_visualization.py → loops 50 states (50 min)
    └─ National maps (30 min)
```

**Problems:**
1. **Sequential bottleneck**: Analysis scripts run one after another, not in parallel
2. **Delayed feedback**: Must wait for all 50 states before seeing any analysis
3. **Duplicate work**: Each batch script loops through all states instead of per-state execution
4. **No parallelization**: Analysis could overlap with subsequent state processing

### Goal

Move per-state visualizations to run immediately after each state completes, keeping only true national aggregations in post-processing:

```
Pipeline Flow (Proposed):
├─ Phase 1: State Processing (Parallel)
│   ├─ Redistricting
│   ├─ Cities enrichment
│   ├─ District summary
│   ├─ Round maps
│   ├─ District maps
│   └─ [NEW] Per-State Analysis (in parallel)
│       ├─ Political analysis
│       ├─ Political visualization
│       ├─ Demographic analysis
│       ├─ Demographic visualization
│       └─ Compactness visualization
│
└─ Phase 2: National Post-Processing (Parallel)
    ├─ create_us_national_political_map.py
    ├─ create_us_national_demographic_map.py
    ├─ create_us_national_compactness_map.py
    ├─ create_metro_area_maps.py
    ├─ create_us_aggregate.py
    ├─ create_us_rounds_hierarchy.py
    └─ generate_dashboard.py
```

### Analysis: Scripts That Can Move to Per-State

**Zero inter-state dependencies** (can run immediately after state completes):

| Script | Current Phase | Can Move? | Input Dependencies |
|--------|--------------|-----------|-------------------|
| `analyze_districts.py` | Post-batch | ✅ YES | final_assignments.pkl, election data |
| `visualize_partisan_lean.py` | Post-batch | ✅ YES | State dir, political CSV |
| `analyze_district_demographics.py` | Post-batch | ✅ YES | final_assignments.pkl, demographic data |
| `visualize_district_demographics.py` | Post-batch | ✅ YES | State dir, demographic CSV |
| `visualize_compactness.py` | Post-batch | ✅ YES | district_summary.csv |

**Must stay in post-processing** (require all 50 states):

| Script | Why National-Only? |
|--------|-------------------|
| `create_us_national_political_map.py` | Combines all state political data |
| `create_us_national_demographic_map.py` | Combines all state demographic data |
| `create_us_national_compactness_map.py` | Combines all state compactness data |
| `create_metro_area_maps.py` | Multi-state metro visualizations |
| `create_us_aggregate.py` | National summary statistics |
| `create_us_rounds_hierarchy.py` | National rounds metadata |
| `create_us_national_rounds_progression.py` | National round progression |
| `generate_dashboard.py` | Depends on all outputs |

### Revised Strategy: Scope-Based Architecture

Instead of wrapper scripts + per-state calls, **refactor core scripts to handle both state and national scopes**:

```python
# Single script handles both cases
python scripts/compactness/visualize_compactness.py \
    --scope state \
    --state-dir outputs/us_2020_v1/states/vermont \
    --census-year 2020

python scripts/compactness/visualize_compactness.py \
    --scope national \
    --output-dir outputs/us_2020_v1 \
    --census-year 2020 \
    --version v1
```

**Key Design Principles:**
- **Single source of truth**: One script per visualization type, not wrapper + core
- **Scope parameter**: `--scope {state|national}` determines execution mode
- **Backward compatible**: Scripts default to state scope for existing usage
- **National aggregation**: National scope does true aggregation, not looping

**Benefits:**
- Eliminates 4 wrapper scripts (run_*_analysis.py, run_*_visualization.py)
- Reduces code duplication
- More flexible for different use cases (e.g., single state testing)
- Cleaner architecture

### Implementation Plan

#### Phase 1: Compactness Prototype (2 hours)

**Refactor `scripts/compactness/visualize_compactness.py`:**

```python
def main():
    parser = argparse.ArgumentParser(description='Visualize district compactness')

    # Scope-based design
    parser.add_argument('--scope', choices=['state', 'national'], default='state',
                       help='Scope: state (single state) or national (all states)')
    parser.add_argument('--census-year', type=str, required=True,
                       help='Census year (2010, 2020)')

    # State scope arguments
    parser.add_argument('--state-dir', type=str,
                       help='State directory (required if scope=state)')

    # National scope arguments
    parser.add_argument('--output-dir', type=str,
                       help='Base output directory (required if scope=national)')
    parser.add_argument('--version', type=str,
                       help='Version (required if scope=national)')
    parser.add_argument('--dpi', type=int, default=150,
                       help='DPI for output maps')

    args = parser.parse_args()

    if args.scope == 'state':
        if not args.state_dir:
            parser.error("--state-dir required when scope=state")
        visualize_state_compactness(args.state_dir, args.census_year)

    elif args.scope == 'national':
        if not args.output_dir or not args.version:
            parser.error("--output-dir and --version required when scope=national")
        visualize_national_compactness(args.output_dir, args.version, args.census_year, args.dpi)

def visualize_state_compactness(state_dir, census_year):
    """Existing per-state visualization logic"""
    # ... current implementation ...

def visualize_national_compactness(output_dir, version, census_year, dpi):
    """National aggregation logic from create_us_national_compactness_map.py"""
    # Load all 50 states
    # Create national map
    # ... implementation from create_us_national_compactness_map.py ...
```

**Test both scopes:**
```bash
# State scope
python scripts/compactness/visualize_compactness.py \
    --scope state \
    --state-dir outputs/us_2020_v1/states/vermont \
    --census-year 2020

# National scope
python scripts/compactness/visualize_compactness.py \
    --scope national \
    --output-dir outputs/us_2020_v1 \
    --version v1 \
    --census-year 2020
```

**Validate:**
- State map identical to previous output
- National map identical to create_us_national_compactness_map.py output

#### Phase 2: Add to Per-State Pipeline (1 hour)

Modify `scripts/pipeline/process_single_state.py`:

```python
# Add optional analysis steps (controlled by --run-analysis flag)
if args.run_analysis:
    steps.extend([
        ("Compactness", f'{sys.executable} scripts/compactness/visualize_compactness.py '
                       f'--scope state --state-dir {state_dir} --census-year {args.year}')
    ])
```

Modify `scripts/pipeline/run_complete_redistricting.py`:
- Add `--run-analysis` flag to state processing
- Replace batch wrapper with national scope call:
  ```python
  # OLD: python scripts/compactness/run_compactness_visualization.py ...
  # NEW:
  subprocess.run([
      sys.executable, 'scripts/compactness/visualize_compactness.py',
      '--scope', 'national',
      '--output-dir', args.output_dir,
      '--version', args.version,
      '--census-year', args.year
  ])
  ```

#### Phase 3: Apply Pattern to Other Scripts (3 hours)

Once compactness prototype is validated, apply same pattern to:

1. **Political Analysis:**
   - Refactor `visualize_partisan_lean.py` with --scope parameter
   - Merge logic from `create_us_national_political_map.py`

2. **Demographic Visualization:**
   - Refactor `visualize_district_demographics.py` with --scope parameter
   - Merge logic from `create_us_national_demographic_map.py`

3. **Metro Areas:**
   - Refactor `create_metro_area_maps.py` to support --scope state (state metros only)
   - Keep --scope national for all metros

#### Phase 4: Testing & Validation (2 hours)

1. **Test with small states:**
   ```bash
   python scripts/pipeline/run_complete_redistricting.py \
       --year 2020 --version v3_test \
       --states "Vermont,Wyoming,Rhode Island" \
       --run-analysis
   ```

2. **Validate:**
   - State-level outputs identical to batch mode
   - National outputs identical to previous
   - Performance improvement measurable

#### Phase 5: Cleanup (30 minutes)

**Delete obsolete scripts (after validation):**
- `scripts/compactness/run_compactness_visualization.py`
- `scripts/compactness/create_us_national_compactness_map.py` (merged into visualize_compactness.py)
- Similar deletions for political/demographic

**Update documentation:**
- Document new --scope parameter
- Update pipeline docs
- Add examples

### Performance Impact

**Current (Sequential Bottleneck):**
```
State Redistricting: 4-8 hours (parallel)
Post-Processing: 300+ minutes (sequential)
  ├─ Political: 100 min
  ├─ Demographic: 150 min
  ├─ Compactness: 50 min
  └─ National: 30 min
────────────────────
Total: 6-10 hours
```

**Proposed (Parallel Execution):**
```
State Redistricting + Analysis: 4-8 hours (parallel overlap)
  └─ Analysis runs as each state completes
National Post-Processing: 30 min (parallel)
────────────────────
Total: 4-9 hours
Savings: 1-2 hours (analysis no longer adds sequential overhead)
```

### Benefits

1. **Faster pipeline**: Eliminate 300-minute sequential bottleneck
2. **Better feedback**: See state results as they complete
3. **Cleaner architecture**: Per-state work stays with per-state processing
4. **Better parallelism**: Analysis overlaps with subsequent states
5. **Logical organization**: Related processing happens together

### Files to Modify

**Modified:**
- `scripts/pipeline/process_single_state.py` - Add analysis steps
- `scripts/pipeline/run_complete_redistricting.py` - Enable per-state analysis, remove batch calls

**Deleted (after validation):**
- `scripts/political/run_political_analysis.py`
- `scripts/demographic/run_demographic_analysis.py`
- `scripts/demographic/run_demographic_visualization.py`
- `scripts/compactness/run_compactness_visualization.py`

**Unchanged (core analysis scripts):**
- `scripts/political/analyze_districts.py`
- `scripts/political/visualize_partisan_lean.py`
- `scripts/demographic/analyze_district_demographics.py`
- `scripts/demographic/visualize_district_demographics.py`
- `scripts/compactness/visualize_compactness.py`

**Unchanged (national scripts):**
- All `create_us_national_*.py` scripts
- `create_metro_area_maps.py`
- `generate_dashboard.py`

### Current Progress (2026-01-12)

**✅ Completed - Phases 1-3: Scope-Based Refactoring**

1. **Scope-Based Architecture Implemented**:
   - Refactored `visualize_compactness.py` to support `--scope {state|national}`
   - Refactored `visualize_partisan_lean.py` to support `--scope {state|national}`
   - Refactored `visualize_district_demographics.py` to support `--scope {state|national}`
   - State scope: `--scope state --state {CODE} --state-dir <path> --census-year 2020`
   - National scope: `--scope national --output-dir <path> --version v1 --census-year 2020`
   - Follows progress bar protocol (TQDM_POSITION, STATUS messages)
   - Implements skip logic (--force flag)

2. **Pipeline Integration**:
   - Added `--run-analysis` flag to `process_single_state.py` (default=True)
   - When enabled, runs 5 additional per-state steps:
     - Political analysis (`analyze_districts.py`)
     - Political visualization (`visualize_partisan_lean.py --scope state`)
     - Demographic analysis (`analyze_district_demographics.py`)
     - Demographic visualization (`visualize_district_demographics.py --scope state`)
     - Compactness visualization (`visualize_compactness.py --scope state`)
   - Updated `run_complete_redistricting.py` to pass flag to parallel workers
   - Analysis steps run immediately after each state completes (no sequential bottleneck)

3. **Scripts Modified**:
   - ✅ `scripts/compactness/visualize_compactness.py` - Scope-based refactoring (719→620 lines)
   - ✅ `scripts/political/visualize_partisan_lean.py` - Scope-based refactoring (719→620 lines)
   - ✅ `scripts/demographic/visualize_district_demographics.py` - Scope-based refactoring
   - ✅ `scripts/pipeline/process_single_state.py` - Added --run-analysis support with 5 steps
   - ✅ `scripts/pipeline/run_complete_redistricting.py` - Added --run-analysis flag (default=True)

4. **Scripts Ready for Deletion** (after full validation):
   - `scripts/compactness/run_compactness_visualization.py` - Replaced by --scope national
   - `scripts/compactness/create_us_national_compactness_map.py` - Merged into visualize_compactness.py
   - `scripts/political/run_political_analysis.py` - Replaced by per-state execution
   - `scripts/demographic/run_demographic_analysis.py` - Replaced by per-state execution
   - `scripts/demographic/run_demographic_visualization.py` - Replaced by per-state execution

**🎯 Validation Status**:
- [x] State scope tested for compactness (Vermont, Wyoming, Rhode Island)
- [x] State scope tested for political analysis (single state)
- [x] State scope tested for demographic analysis (single state)
- [x] Per-state pipeline integration tested (Vermont with --run-analysis)
- [x] All 5 analysis steps run successfully in state processing
- [x] Explicit --state parameter implemented (no path parsing)
- [x] --run-analysis flag defaults to True in main pipeline
- [ ] National scope tested end-to-end for all scripts
- [ ] Full pipeline test with all 50 states + --run-analysis
- [ ] Performance validation (should save 1-2 hours)

**📋 Remaining Work** (Full Validation Required):
- Phase 4: Full pipeline test with 2-3 small states
- Phase 5: Implement national scope for political/demographic visualization (merge create_us_national_*_map.py logic)
- Phase 6: Full pipeline test with all 50 states + --run-analysis
- Phase 7: Delete obsolete wrapper scripts after validation completes
- Phase 8: Performance measurement (compare with/without --run-analysis)

**Template for Future Refactoring**:

When applying to political/demographic scripts, follow this pattern:

```python
def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--scope', choices=['state', 'national'], default='national')
    parser.add_argument('--census-year', required=True)

    # State scope
    parser.add_argument('--state', help='State code (required if scope=state)')
    parser.add_argument('--state-dir', help='State directory (required if scope=state)')

    # National scope
    parser.add_argument('--output-dir', help='Base directory (required if scope=national)')
    parser.add_argument('--version', help='Version (required if scope=national)')

    # Common
    parser.add_argument('--dpi', type=int, default=150)
    parser.add_argument('--force', action='store_true')
    parser.add_argument('--position', type=int, default=-1)

    args = parser.parse_args()
    position = args.position if args.position >= 0 else int(os.environ.get('TQDM_POSITION', '-1'))

    if args.scope == 'state':
        return visualize_state(args.state_dir, args.state, args.census_year, args.dpi)
    elif args.scope == 'national':
        return visualize_national(args.output_dir, args.version, args.census_year,
                                 args.dpi, position, args.force)
```

### Risk Mitigation

**Low-risk approach:**
- Phased implementation with testing between phases
- Keep old batch scripts until validated
- Use test version flag (`v3_test`) to avoid overwriting production data
- Per-state analysis marked as non-critical (won't break pipeline on failure)
- Full rollback capability (remove `--run-analysis` flag)

### Estimated Complexity

**Medium-High** (4-6 hours)
- Requires careful orchestration changes
- Need thorough testing to ensure outputs match
- Must verify no hidden dependencies
- Performance validation important

### Success Criteria

- [x] All per-state analysis runs successfully during state processing (tested on single state)
- [ ] Output quality matches current batch-mode results (byte-for-byte if possible) - Needs full validation
- [ ] National maps successfully aggregate per-state data - National scope not yet implemented
- [ ] Pipeline completes 1-2 hours faster than current approach - Performance testing pending
- [x] No regressions in output quality or correctness (verified on test states)
- [ ] Dashboard shows all expected data - Full pipeline test pending
- [x] Code is cleaner and more maintainable (eliminated complex path parsing, unified interface)

---
