# Pipeline Pitfalls (PP-01..PP-03)

Structural vulnerabilities in multi-script pipeline chains. The root pattern: a flag or intent expressed at one level of the chain silently fails to reach the level where it matters.

---

## PP-01: Subprocess flag inheritance gap

**Pattern:** A flag registered and honored at level N of a pipeline chain is silently ignored at level N+1 because each subprocess starts with a fresh argument namespace. Child script defaults override parent intent. The system appears to accept the flag (no error) but produces output as if it weren't set.

**Domain:** Any system with a chain of subprocess calls where configuration flows from outer to inner scripts. The longer the chain, the more opportunities for a flag to be dropped. Flags registered in the outermost script must be explicitly passed at every subprocess boundary — they do not propagate automatically.

**Why it's hard to catch:** The bug is invisible at the call site. The parent passes the flag correctly; the child simply never receives it. Only end-to-end testing or chain-completeness auditing reveals the gap.

**Structural solution:** Chain completeness table — for every flag registered in the outermost script, trace its explicit pass-through at every subprocess boundary. Any gap in the table is a bug. `review-pipeline` skill automates this audit.

**Status:** SOLVED for `--version`, `--skip-political`, `--skip-demographic`, `--election-year`
**Proved by:** Flags explicitly traced through `run_complete → run_states_parallel → process_single_state → run_state_redistricting`
**Test:** `tests/unit/test_pipeline_flag_propagation.py` — all 15 tests

---

## PP-02: Force-rerun requires two independent mechanisms to both fire

**Pattern:** A "force re-run" operation requires clearing a completion marker AND passing a reset flag to the execution script. If either mechanism fires without the other, the system appears to re-run (clears the marker, says it's reprocessing) but actually uses stale outputs (execution script skips because output file exists).

**Domain:** Any pipeline with both (a) a completion tracking system (marker files) and (b) an execution-level skip check (output file existence). These are independent safeguards that must both be bypassed for a true force re-run. They were designed separately and their interaction was not specified.

**Why it's hard to catch:** The force-rerun appears to work — states cycle through the pipeline visibly. Only inspecting the output file timestamps reveals they weren't regenerated.

**Structural solution:** The force-rerun flag must propagate all the way to the execution-level skip check. `--reprocess` at the outer level becomes `--reset` at the inner level (deletes output files before checking). The two mechanisms are explicitly linked.

**Status:** SOLVED
**Proved by:** `--reprocess` now appends `--reset` to redistricting flags in process_single_state.py
**Test:** `tests/unit/test_pipeline_flag_propagation.py::TestReprocessPropagation`

---

## PP-03: Conditional step with inversion gap

**Pattern:** An optional analysis step is conditioned on a flag (`run_analysis=True`). The flag has an "enable" path (explicitly set True) and a "disable" path (explicitly set False). But only one path is propagated through the subprocess chain — the disable path is dropped. Result: the step always runs even when the flag says it shouldn't.

**Domain:** Any pipeline where an optional step can be skipped via a flag, and that flag must traverse subprocess boundaries. The inversion (False / skip) is harder to propagate than the assertion (True / run) because conditional logic at each boundary tends to only check for True.

**Structural solution:** Propagate the disable case explicitly: `if not args.run_analysis: cmd.append('--skip-analysis')`. Child scripts must also register the disable flag (`--skip-analysis` sets `run_analysis=False`). The default in child scripts must be False — explicit enable required, not implicit.

**Status:** SOLVED for `--skip-analysis`, `--skip-political`, `--skip-demographic`
**Proved by:** `run_states_parallel.py` defaults `run_analysis=False`; all three skip flags propagated
**Test:** `tests/unit/test_pipeline_flag_propagation.py::TestSkipFlags`
