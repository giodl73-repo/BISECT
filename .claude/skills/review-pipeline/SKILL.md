---
name: review-pipeline
description: Audit pipeline scripts for drift, broken invariants, and flag/parameter propagation bugs using MERIDIAN and BOUNDARY lenses. Catches issues like version not propagating, vra_mode conflation, constitutional violations.
user_invocable: true
---

# Pipeline Code Review

Audit one or more pipeline scripts for correctness, flag semantics, and invariant drift. Today's bugs — `vra_mode` conflated with `multi_constraint`, `--version` not propagating, `--reset` not firing — all came from drift in pipeline parameter chains. This skill catches those categories systematically.

## Input

The user specifies which script(s) to review. If not specified, ask. Examples:
- `scripts/pipeline/run_state_redistricting.py`
- `scripts/pipeline/process_single_state.py`
- The full chain: `run_complete_redistricting → run_states_parallel → process_single_state → run_state_redistricting`

## Steps

### 1. Load the scripts

Read every script in the chain being reviewed. For each, identify:
- All `argparse` arguments it registers
- All subprocess calls it makes to other pipeline scripts
- All flags/arguments it passes through vs. silently drops
- All module-level constants or hardcoded values

### 2. MERIDIAN review — algorithm correctness

MERIDIAN checks that the algorithm is doing what it claims.

**Flag propagation audit:**
For each argument registered in the outermost script, trace it through every subprocess call in the chain. Flag any argument that:
- Is registered but never passed to child scripts
- Has a different default at different levels of the chain
- Is passed as a different flag name than it was received

**Hardcoded value audit:**
Search for any string literals that should be variables (e.g., `'v1'`, `'edge-weighted'`, hardcoded paths).

**Invariant audit — VRA-specific:**
- Is `vra_mode` used only as "this is a VRA run" (True throughout)? Never as a METIS mode flag?
- Is `multi_constraint` derived from `vra_target_tree is not None`? Never from `vra_mode`?
- Is `vertex_weights.ndim` used for population indexing? Never `vra_mode`?
- Are population stats always using 1D weights?

**Skip logic audit:**
For any script that checks for existing outputs and skips:
- Does `--reprocess`/`--reset` correctly propagate to force re-run?
- Is the skip check using the right file (e.g., `final_assignments.pkl` not `district_summary.csv`)?

Format each finding:
```
**MERIDIAN [SEVERITY]:** {what is wrong}
File: {script}:{line}
Evidence: `{code snippet}`
Fix: {what to change}
```

### 3. BOUNDARY review — constitutional invariants

BOUNDARY checks that pipeline outputs cannot violate legal requirements.

**Population balance:**
- Does any script that calls METIS enforce `ufactor=5` (0.5% tolerance)?
- Is `ufactor` passed through every level of the call chain?
- Does the VRA mode preserve 1D vertex weights? (2D vertex weights break population balance)

**VRA output integrity:**
- Is `vra_analysis.pkl` written when `vra_mode=True`?
- Is `vra_mode` still `True` at the point where analysis is written, or was it cleared too early?

Format findings in same structure.

### 4. Chain completeness check

For the full pipeline chain, produce a table:

```
| Argument | Registered in | Passed to | Dropped at |
|----------|---------------|-----------|------------|
| --version | run_complete | run_states_parallel | process_single_state ← BUG |
| --partition-mode | run_complete | run_states_parallel ✓ | process_single_state ✓ |
| --reset | run_state_redistricting | — | never propagated from process_single_state ← BUG |
```

### 5. Summary

```
MERIDIAN findings: N (critical: X, warning: Y)
BOUNDARY findings: N (critical: X, warning: Y)

Critical bugs (fix before running pipeline):
1. {description}

Warnings (fix before next paper):
1. {description}
```

## Key Rules

- **Trace the full chain.** A flag dropped at level 2 of 4 looks fine at levels 1 and 3 — only full chain tracing reveals it.
- **Default values are bugs.** If `--version` defaults to `'v1'` in a child script, that default will silently override the parent's intent.
- **The VRA invariants are non-negotiable.** See `.roles/meridian.md` domains section for the complete list.
- **BOUNDARY always wins on population balance.** Any code path that could produce >0.5% district population deviation is a critical bug regardless of how elegant the algorithm is.
