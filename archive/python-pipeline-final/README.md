# Python Pipeline — Final Archive

**Status:** REFERENCE ONLY — do not modify.
**Date archived:** 2026-04-29
**Archived at commit:** see git log for the commit that moved these files

## What this is

This directory contains the last-working state of the Python redistricting pipeline as it existed before the cutover to the Rust `redist` binary.

It is preserved as a forensic reference. If a future investigation needs to compare a current Rust output against what the Python pipeline would have produced, this is the implementation to consult. It is not maintained, not imported by any active code, and should not be modified.

## What's here

### scripts/pipeline/

The orchestration layer:
- `run_complete_redistricting.py` — main multi-year orchestrator (replaced by `redist run`)
- `run_states_parallel.py` — parallel state runner (replaced by `redist states`)
- `process_nation.py` — national post-processing (replaced by `redist run -s nation`)
- `process_single_state.py` — single-state pipeline (replaced by `redist state`)
- `run_state_redistricting.py` — single-state wrapper (replaced by `redist state`)

## Out of scope for this archive

Several utility modules under `scripts/utils/` (pipeline_orchestrator, progress_coordinator, stage_tracker, status_protocol, terminal_utils) have multiple downstream callers in research and analysis scripts. Migrating those is a separate cleanup pass after the active production pipeline is fully off Python. They remain in `scripts/utils/` for now.

The duplicated algorithm library (`src/apportionment/partition/`, `src/apportionment/data/`) is archived separately under `archive/python-pipeline-final/src/apportionment/` per Plan 02 Task 3.

## See also

- `docs/superpowers/specs/2026-04-29-rust-python-final-architecture.md` — target architecture
- `docs/superpowers/plans/2026-04-29-python-deletion.md` — execution plan
- `design/rust-port/migration-log.md` — phases that produced the Rust replacement
- `docs/REDIST_CLI.md` — current CLI reference
