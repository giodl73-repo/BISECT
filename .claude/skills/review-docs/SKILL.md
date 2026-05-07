---
name: review-docs
description: Audit documentation against current code state. Catches stale references, deleted features, wrong paths, and drift between docs and implementation. Uses DATUM (evidence accuracy) and MERIDIAN (algorithm description accuracy).
user_invocable: true
---

# Documentation Drift Review

Documentation drifts when code changes and docs don't. Today we found ARCHITECTURE.md describing 400 lines of deleted FastAPI/React infrastructure. This skill systematically checks docs against the current codebase to find that category of drift.

## Input

Specify scope:
- `architecture` — context/ARCHITECTURE.md
- `claude` — CLAUDE.md
- `concepts` — docs/concepts/*.md
- `all` — all documentation

## Steps

### 1. Build the code inventory

Before reading docs, build the ground truth:

**Directories that exist:**
```bash
ls scripts/pipeline/ scripts/data/ src/apportionment/ web/ .roles/ .claude/skills/
```

**Deleted things to watch for:**
- `api/`, `backend/`, `frontend/` — deleted April 2026
- `tools/pipeline_manager/` — deleted
- Wave 9 FastAPI/React app — gone
- `run_manager.bat` — gone
- Enhancement Manager web app — gone

**Current pipeline scripts:** List `scripts/pipeline/*.py` (not `.bak`)

**Current partition modes:** `unweighted`, `edge-weighted`, `metis-vra`

### 2. DATUM review — evidence accuracy

For each documentation file, check every claim that references:
- A file path — does it exist?
- A script name — does the script exist?
- A command — does it work?
- A directory structure — does it match `ls` output?
- A feature — was it deleted?

Flag format:
```
**DATUM [STALE]:** {doc file}:{line} claims "{text}" but {actual state}
Fix: {what to update}
```

### 3. MERIDIAN review — algorithm accuracy

For algorithm descriptions in any doc:

**VRA algorithm (docs/concepts/vra-compliance.md, CLAUDE.md):**
- Does it describe edge-weighting (not multi-constraint vertex weights)?
- Is the adaptive boost formula correct: `max(3.0, 10.0 * (1 - 0.7 * minority_frac))`?
- Is the threshold correctly stated as 40%?
- Is `vra_mode` described as "VRA run flag" not "multi-constraint flag"?

**Edge-weighted bisection (docs/concepts/edge-weighted-bisection.md):**
- Is the edge weight source correctly described (TIGER boundary lengths)?
- Is the minimum_boundary_length default (10m) correct?

**Pipeline stages (docs/concepts/pipeline-stages.md):**
- Are all 5 stages current? Is the streaming mode described correctly?
- Are the skip-detection marker files (`.states_complete`, `.tract_adjacency_complete`) current?

**CLAUDE.md:**
- Are all command examples current? (e.g., `run -v v1` syntax)
- Is the test count current?
- Are "Recent Changes" entries accurate?

### 4. Cross-reference: docs vs. skills

Check that skills reference current file paths:
- Does `review-pipeline` skill reference scripts that exist?
- Does `review-vra` skill reference correct invariants?
- Do concept guides cross-reference the right paper track (D.0 not D.1 for edge-weighting)?

### 5. README check

For README.md specifically:
- Do all dashboard links resolve? (`dashboard_2020.html`, `dashboard_vra.html`)
- Do all paper PDF links point to committed files?
- Is the data distribution section current (GitHub Releases, not LFS)?
- Is the `setup_data.py` command correct?

### 6. Summary

```
DOCUMENTATION AUDIT
Files checked: N
Total drift findings: N

CRITICAL (broken links/missing files):
1. {file:line — description}

STALE (outdated descriptions):
1. {file:line — description}

ALGORITHM DRIFT (wrong description of current code):
1. {file:line — description}

CLEAN files: {list}
```

## Key Rules

- **Every path claim is testable.** If a doc says a file exists, check it with `ls`.
- **Deleted code leaves traces.** Search for `Wave 9`, `FastAPI`, `backend/`, `frontend/`, `pipeline_manager` — any mention is stale.
- **Algorithm docs must match code.** If `vra-compliance.md` still describes multi-constraint vertex weights, that's MERIDIAN's critical failure category.
- **CLAUDE.md is the source of truth for commands.** Any command example that doesn't work is a bug, not just a doc issue.
- **Paper cross-references matter.** If a concept guide says "see Paper D.1" for edge-weighting but the methodology is in D.0, that's a DATUM finding.
