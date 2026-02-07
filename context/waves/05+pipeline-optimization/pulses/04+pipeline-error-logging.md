---
wave_uuid: 12043a
slug: pipeline-error-logging
uuid: 588e23
---
# E39: Comprehensive Pipeline Error Logging

**Status**: ✅ COMPLETED
**Priority**: High
**Estimated Complexity**: Medium (3-5 hours)
**Actual Time**: ~3 hours total (MVP: 2h, Phase 3: 1h)
**Created**: January 17, 2026
**Started**: January 17, 2026
**Completed**: January 17, 2026
**Commits**: [2f4cb7f](https://github.com/giodl_microsoft/redistricting/commit/2f4cb7f2c5517a161e27df8747a19c1b5e0bba6a)
**Size**: L - 3,545 lines changed (20 files)

**Phases Completed**:
- ✅ Phase 1: Logging infrastructure (error_logger.py, stage_tracker.py, parse_error_logs.py)
- ✅ Phase 2: Integration into process_nation.py (critical V6 failure point)
- ✅ Phase 3: Integration into analysis scripts (political, demographic, compactness, visualization)
- ✅ Phase 5: Unit tests (16 tests, 100% pass rate)

## Current State

The redistricting pipeline currently has minimal error logging:

1. **No Persistent Logs**: Errors are printed to stdout/stderr but not saved to files
2. **Lost Information**: When parallel workers crash or the terminal closes, error details are lost
3. **Hard to Debug**: The V6 run failure had no error logs, making it impossible to determine what went wrong in national post-processing
4. **Incomplete Exception Handling**: Many try-except blocks catch exceptions but only return status codes without logging details
5. **No Stage Tracking**: Can't determine which specific stage (redistricting, analysis, national post-processing) failed
6. **Progress Bar Interference**: Error messages can be obscured by tqdm progress bars

**Evidence from V6 Failure**:
- All 50 states completed for all 3 years (2000, 2010, 2020)
- National post-processing incomplete (missing round maps: 2000=6, 2010=4, 2020=3, expected=9 each)
- No error.log files to diagnose root cause
- `execution_time_seconds: null` in config.json indicates abnormal termination
- No `.national_complete` markers created

## Goal

Create a comprehensive error logging system that:

1. **Captures All Failures**: Log exceptions with full tracebacks to year-specific `error.log` files
2. **Tracks Pipeline Stages**: Record which stage failed (state redistricting, analysis, national post-processing)
3. **Persistent & Structured**: Error logs survive terminal closures and are easy to parse
4. **Non-Intrusive**: Don't break existing workflows or progress displays
5. **Multi-Year Compatible**: Separate logs per census year in parallel runs
6. **Actionable**: Include enough context to diagnose and fix issues

## Implementation Plan

### Phase 1: Logging Infrastructure
- [ ] Create `scripts/utils/error_logger.py` - Centralized error logging utility
  - `ErrorLogger` class with context managers
  - Methods: `log_exception()`, `log_stage_start()`, `log_stage_complete()`, `log_warning()`
  - Auto-create `outputs/{version}/{year}/error.log`
  - Thread-safe for parallel execution
  - Include timestamps, stage names, exception tracebacks, system info
- [ ] Create `scripts/utils/stage_tracker.py` - Track pipeline stage progress
  - Write `.stage_{name}` marker files as stages complete
  - Helps identify where failures occurred
  - Support states vs national stages

### Phase 2: Integration - Main Pipeline Scripts
- [ ] Modify `scripts/pipeline/run_complete_redistricting.py`
  - Wrap multi-year execution with error logger
  - Log worker failures from parallel state processing
  - Catch and log exceptions from process_nation calls
  - Write summary of failed states/years to master log
- [ ] Modify `scripts/pipeline/process_nation.py`
  - Initialize ErrorLogger for year-specific logs
  - Log start/completion of each of the 9 national tasks
  - Catch exceptions in parallel task execution
  - Mark successful completion in log before creating `.national_complete`
- [ ] Modify `scripts/pipeline/run_state_redistricting.py`
  - Wrap state processing with error logger
  - Log state start/end with timing
  - Catch subprocess failures and log details
- [ ] Modify `scripts/pipeline/process_single_state.py`
  - Log each stage (redistricting, cities, summary, maps, analysis)
  - Catch and log exceptions from library functions
  - Include METIS errors and graph connectivity failures

### Phase 3: Integration - Analysis Scripts
- [ ] Add error logging to analysis scripts:
  - `scripts/pipeline/analyze_districts.py` (political)
  - `scripts/pipeline/analyze_district_demographics.py`
  - `scripts/pipeline/analyze_district_compactness.py`
  - `scripts/pipeline/visualize_*.py` (all visualization scripts)
- [ ] Log missing data as warnings (not errors) with helpful messages
- [ ] Include file paths and data availability checks

### Phase 4: Error Recovery & Reporting
- [ ] Add `--resume-from-errors` flag to skip successfully completed stages
  - Check `.stage_{name}` markers
  - Skip stages that completed successfully
  - Retry only failed stages
- [ ] Create `scripts/utils/parse_error_logs.py` - Summary tool
  - Parse all error.log files
  - Categorize errors (data missing, METIS failures, OOM, etc.)
  - Generate human-readable summary
  - Suggest fixes based on error patterns
- [ ] Update `/pipeline-debug` skill to check error.log files first

### Phase 5: Testing
- [ ] Unit tests (`tests/unit/utils/test_error_logger.py`)
  - Test ErrorLogger creation, writing, thread-safety
  - Test StageTracker marker files
  - Mock file I/O
- [ ] Integration tests (`tests/integration/test_error_logging.py`)
  - Simulate pipeline failures at different stages
  - Verify error.log contents and format
  - Test stage marker creation/checking
- [ ] E2E tests (`tests/e2e/test_pipeline_with_errors.py`)
  - Test Vermont run with injected failures
  - Verify error recovery with `--resume-from-errors`
  - Test multi-year error logging
- [ ] Manual pipeline testing
  - Print-only mode: Verify no errors with dry run
  - Small state (VT): Inject a failure, verify error.log created
  - Multi-year: Test parallel error logging (separate logs per year)
  - Full validation: Run 3 states, verify logs for any warnings

### Phase 6: Documentation
- [ ] Update `CODING_PATTERNS.md` with error logging pattern
- [ ] Update `context/QUICK_REFERENCE.md` with error log locations
- [ ] Update `/pipeline-debug` skill instructions to check error.log first
- [ ] Add error logging section to `ARCHITECTURE.md`
- [ ] Update `CHANGELOG.md` with enhancement details
- [ ] Update `CLAUDE.md` Recent Changes section
- [ ] Mark enhancement complete in INDEX.md

## Files to Modify/Create

### Create
- `scripts/utils/error_logger.py` - Centralized error logging utility (~150 lines)
- `scripts/utils/stage_tracker.py` - Stage marker management (~80 lines)
- `scripts/utils/parse_error_logs.py` - Error log analysis tool (~200 lines)
- `tests/unit/utils/test_error_logger.py` - Unit tests (~150 lines)
- `tests/integration/test_error_logging.py` - Integration tests (~200 lines)
- `tests/e2e/test_pipeline_with_errors.py` - E2E error recovery tests (~150 lines)

### Modify
- `scripts/pipeline/run_complete_redistricting.py` - Add error logging to orchestrator
- `scripts/pipeline/process_nation.py` - Log national post-processing stages
- `scripts/pipeline/run_state_redistricting.py` - Log state wrapper
- `scripts/pipeline/process_single_state.py` - Log per-stage execution
- `scripts/pipeline/analyze_districts.py` - Log political analysis errors
- `scripts/pipeline/analyze_district_demographics.py` - Log demographic errors
- `scripts/pipeline/analyze_district_compactness.py` - Log compactness errors
- `scripts/pipeline/visualize_*.py` (8 files) - Log visualization errors
- `.claude/skills/pipeline-debug/instructions.md` - Add error.log checking

## Testing Plan

### A. Automated Tests (MANDATORY)

**Unit Tests** (`tests/unit/utils/`):
```bash
pytest tests/unit/utils/test_error_logger.py -v
pytest tests/unit/utils/test_stage_tracker.py -v
```
- ErrorLogger: creation, writing, formatting, thread-safety
- StageTracker: marker creation, checking, cleanup
- Coverage: >80%

**Integration Tests** (`tests/integration/`):
```bash
pytest tests/integration/test_error_logging.py -v
```
- Simulate failures at different pipeline stages
- Verify error.log structure and content
- Test stage marker persistence
- Test parallel error logging (multi-year)

**E2E Tests** (`tests/e2e/`):
```bash
pytest tests/e2e/test_pipeline_with_errors.py -v
```
- Run Vermont with injected failures
- Verify error.log creation and contents
- Test `--resume-from-errors` flag
- Test multi-year error isolation

### B. Manual Pipeline Testing

1. **Print-only mode**: Verify no errors with dry run
   ```bash
   python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --print-only
   ```

2. **Small state test** (VT, 30s-2min): Inject a controlled failure
   ```bash
   # Temporarily break METIS or move a data file
   python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test_error --states "VT"
   # Verify outputs/test_error/2020/error.log created with exception details
   ```

3. **Multi-year test**: Verify separate error logs per year
   ```bash
   python scripts/pipeline/run_complete_redistricting.py --year all --version test_multi --states "VT"
   # Verify outputs/test_multi/{2000,2010,2020}/error.log exist and are independent
   ```

4. **Error recovery test**: Test `--resume-from-errors`
   ```bash
   # Run once (may fail partway)
   python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test_resume --states "VT"
   # Resume from where it left off
   python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test_resume --states "VT" --resume-from-errors
   # Verify only failed stages rerun
   ```

5. **Error log parsing**: Test summary tool
   ```bash
   python scripts/utils/parse_error_logs.py --version test_error --year 2020
   # Verify human-readable summary and suggested fixes
   ```

## Success Criteria

- [ ] All automated tests pass (unit, integration, e2e)
- [ ] Error logs created in `outputs/{version}/{year}/error.log` on any failure
- [ ] Error logs include: timestamp, stage name, exception type, full traceback, system info
- [ ] Stage markers (`.stage_{name}`) track pipeline progress
- [ ] Parallel runs create separate logs per year with no interference
- [ ] `--resume-from-errors` successfully skips completed stages
- [ ] `parse_error_logs.py` generates actionable summaries
- [ ] `/pipeline-debug` skill updated to check error.log files first
- [ ] All documentation updated (6 files)
- [ ] No regressions in existing pipeline workflows
- [ ] Error logging overhead <5% of pipeline runtime

## Benefits

1. **Faster Debugging**: Immediately see what went wrong without detective work
2. **No Lost Information**: Error details persist even if terminal closes or parallel workers crash
3. **Better User Experience**: Clear error messages with actionable fixes
4. **Audit Trail**: Track all failures and warnings across runs
5. **Error Recovery**: Resume failed runs without reprocessing completed stages
6. **Proactive Monitoring**: Catch warnings before they become errors
7. **Pattern Detection**: Identify recurring issues across states/years
8. **Skill Improvement**: `/pipeline-debug` becomes more effective

## Dependencies

None - pure addition to existing pipeline.

## Risks & Mitigations

- **Risk 1**: File I/O overhead slows down pipeline
  - *Mitigation*: Buffer writes, async logging, minimal overhead design
  - *Validation*: Benchmark before/after with timing

- **Risk 2**: Thread-safety issues in parallel execution
  - *Mitigation*: Use Python's `logging` module with thread-safe handlers
  - *Testing*: Multi-threaded unit tests

- **Risk 3**: Error logs grow too large over multiple runs
  - *Mitigation*: One log per version/year combination, automatic truncation after N lines
  - *Note*: Users control with `--version` parameter

- **Risk 4**: Stage markers interfere with `--reset` flag
  - *Mitigation*: `--reset` clears all stage markers before starting
  - *Testing*: E2E test of reset behavior

## Implementation Notes

### Key Design Decisions

1. **Error Log Location**: `outputs/{version}/{year}/error.log`
   - Keeps errors with output data
   - One log per year in multi-year runs
   - Version parameter provides isolation

2. **Stage Markers**: `.stage_{name}` hidden files in output directory
   - Simple, filesystem-based
   - No database needed
   - Easy to inspect and clear

3. **Logging Framework**: Python's built-in `logging` module
   - Thread-safe by default
   - Rich formatting options
   - Well-tested and reliable

4. **Non-Intrusive Integration**: Wrap existing code, don't rewrite
   - Context managers for clean syntax
   - Backward compatible (no breaking changes)
   - Optional `--resume-from-errors` flag

### Error Log Format

```
==========================================================================
Pipeline Error Log: outputs/V6/2020/error.log
Started: 2026-01-17 22:10:59
Version: V6 | Year: 2020 | Python: 3.13.9 | METIS: 5.1.0
==========================================================================

[2026-01-17 22:15:23] STAGE_START: national_post_processing
[2026-01-17 22:15:23] TASK_START: visualize_national_rounds
[2026-01-17 22:16:47] ERROR: visualize_national_rounds
Exception: IndexError
Message: list index out of range
Traceback:
  File "scripts/pipeline/visualize_national_rounds.py", line 142, in main
    round_data = rounds_hierarchy[round_num]
IndexError: list index out of range

System Context:
  - Memory: 8.2 GB free / 16.0 GB total
  - Disk: 145 GB free
  - States completed: 50/50
  - Round data files found: 6/9

[2026-01-17 22:16:47] STAGE_FAILED: national_post_processing

==========================================================================
Pipeline Summary: 1 error(s), 0 warning(s)
==========================================================================
```

### Windows Compatibility

- All logging uses ASCII characters only (no Unicode)
- File paths use `Path` objects for Windows compatibility
- Log files use UTF-8 encoding with error handling

## Related Documentation

- Enhancement #37: [Parallel Multi-Year Pipeline](37_parallel_multi_year_pipeline.md) - Added complexity requiring better error tracking
- Architecture: [ARCHITECTURE.md](../../ARCHITECTURE.md) - Pipeline structure
- Coding Patterns: [CODING_PATTERNS.md](../../CODING_PATTERNS.md) - Error handling patterns
- Quick Reference: [QUICK_REFERENCE.md](../../QUICK_REFERENCE.md) - Troubleshooting guide
