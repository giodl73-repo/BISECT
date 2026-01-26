# E49: Pipeline Download Integration (Opt-In)

**Status**: 📋 PLANNED
**Priority**: Low
**Estimated Effort**: Small (< 2h)
**Created**: 2026-01-18
**Depends On**: Enhancement #48 (Unified Download Orchestrator)

## Priority Levels

- **Low**: Quality-of-life improvement for fresh installations

## Current State

**Downloads are completely separate** from pipeline (by design):
- User manually downloads data or restores from backup
- Pipeline uses existing data in `data/Census {year}/`
- Download orchestrator is standalone tool
- Cache checking in download orchestrator shows what's available vs missing

**This works well for:**
- Explicit control over data acquisition
- Easier debugging (download vs processing are separate)
- Clear separation of concerns
- Flexibility to download once, process many times

## Goal

**Optional pipeline integration with explicit opt-in flag** for convenience:
- Pipeline checks local cache first (always)
- If data missing, pipeline shows what's needed
- User can opt-in to auto-download with `--download` flag
- Without flag, pipeline just informs and uses what's available

**Not a goal:**
- Force automatic downloads (maintain manual control as default)
- Make downloads mandatory (pipeline should work with existing data)

## Implementation Plan

### Phase 1: Add Cache Checking to Pipeline
- [ ] Import download stage checking into `run_complete_redistricting.py`
- [ ] In "data" stage, check what's available vs missing
- [ ] Print informative message about cache status
- [ ] Continue with whatever data exists (graceful degradation)

### Phase 2: Add Optional Download Flag
- [ ] Add `--download` flag to `run_complete_redistricting.py`
- [ ] If flag set AND data missing, call download orchestrator
- [ ] Pass through appropriate stages, year, workers
- [ ] Download orchestrator runs, then pipeline continues

### Phase 3: Interactive Mode (Optional)
- [ ] If missing data and not `--download`, prompt user
- [ ] "Download missing data now? [y/N]:"
- [ ] Respect user choice, continue either way

## Files to Modify

### Modified
- `scripts/pipeline/run_complete_redistricting.py` - Add cache checking and optional download call
- `scripts/data/download_orchestrator.py` - Ensure it can be called from Python (not just CLI)

### Tests
- `tests/integration/test_pipeline_download_integration.py` - Test cache checking
- `tests/e2e/test_pipeline_with_download.py` - Test full workflow

## Usage Examples

### Without Flag (Default - Current Behavior)
```bash
run -y 2020 -v v1

# Output:
# [INFO] Checking data cache...
# [OK] 2020 redistricting data: Available
# [OK] 2020 demographics: Available
# [WARN] 2020 elections: Not found (optional - continuing)
#
# Proceeding with available data...
```

### With Download Flag (New - Opt-In)
```bash
run -y 2020 -v v1 --download

# Output:
# [INFO] Checking data cache...
# [DOWNLOAD] 2020 elections: Not found
# [INFO] Downloading missing data...
#
# [2020] ████████████████████ 50/50 states complete
#   ├─ Worker 1: Idle
#   └─ Worker 2: Idle
#
# [OK] Downloads complete. Proceeding with pipeline...
```

### Check-Only Mode
```bash
run -y 2020 -v v1 -s data --check

# Output:
# [INFO] Data cache status for 2020:
#   [OK] Redistricting data: Available
#   [OK] Demographics: Available
#   [MISSING] Elections: Not found
#
# To download missing data:
#   run -y 2020 -v v1 --download
# Or manually:
#   python scripts/data/download_orchestrator.py --stages elections --year 2020
```

## Success Criteria

- [ ] Pipeline checks cache before downloading (always)
- [ ] Without `--download` flag, pipeline informs but continues
- [ ] With `--download` flag, pipeline downloads missing data automatically
- [ ] User retains full control (opt-in, not forced)
- [ ] All existing workflows continue to work (backward compatible)
- [ ] Clear messaging about what's available vs missing

## Benefits

- **Fresh installs**: Easier setup for new users (opt-in to auto-download)
- **Partial data**: Pipeline clearly shows what's missing
- **Explicit control**: User decides when to download
- **Backward compatible**: Doesn't change existing workflows

## Dependencies

- **Enhancement #48**: Unified Download Orchestrator must be complete
- Download orchestrator must support being called from Python (not just CLI)
- Stage checking utilities must be importable

## Risks & Mitigations

### Risk 1: Breaking Existing Workflows
**Mitigation**: Make all download behavior opt-in with explicit flag

### Risk 2: Confusing User Experience
**Mitigation**: Clear messaging about cache status and download options

### Risk 3: Network Issues During Pipeline Run
**Mitigation**: Pipeline should gracefully handle download failures, continue with available data

## Implementation Notes

**Key Principle**: Pipeline should **never force downloads**. It should:
1. Check cache (inform user)
2. Offer to download (if user opts in)
3. Continue with available data (graceful degradation)

**Example Integration**:
```python
# In run_complete_redistricting.py

if 'data' in args.stages:
    from scripts.data.download_stages import get_download_plan

    for year in year_queue:
        plan = get_download_plan(int(year), ['redistricting', 'demographics'], year_output_dirs[year])

        if plan['needed_downloads']:
            print(f"[INFO] {year}: Missing {', '.join(plan['needed_downloads'].keys())}")

            if args.download:
                # User opted in - download automatically
                print(f"[DOWNLOAD] Downloading missing data for {year}...")
                # Call download orchestrator
            else:
                # Just inform
                print(f"[INFO] To download: run with --download or manually")
```

## Related Enhancements

- **#48** (Unified Download Orchestrator) - Provides the download infrastructure
- **#47** (Census Data Processing) - Provides the processing pipeline
- **#37** (Parallel Multi-Year Pipeline) - Established CLI patterns for pipeline

## Estimated Complexity

**Effort**: < 2 hours
- Phase 1 (cache checking): 30 min
- Phase 2 (optional flag): 30 min
- Phase 3 (testing): 1 hour

**Risk**: Low
- No changes to core pipeline logic
- Purely additive (opt-in feature)
- Easy to test and validate

## Next Steps

After Enhancement #48 is complete:
1. Get user feedback on desired behavior
2. Implement cache checking (Phase 1) - standalone value
3. Add optional download flag (Phase 2) - opt-in convenience
4. Test with fresh install scenario
