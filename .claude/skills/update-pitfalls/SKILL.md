---
name: update-pitfalls
description: Extract generic pitfall patterns from bugs or issues found this session and add them to design/pitfalls/. TRENCH role. Run at end of every significant development session.
user_invocable: true
---

# Update Pitfalls Collection (TRENCH)

Extract structural vulnerability patterns from this session's bugs and findings. Add them to the pitfalls collection. A pitfall is not a bug report — it is the pattern that made the bug possible.

## Input

Describe what happened this session: bugs found, fixes made, code reviewed. The skill will extract patterns.

## Steps

### 1. List this session's bugs and findings

From the session history, enumerate:
- Every bug fixed
- Every incorrect assumption discovered
- Every place the system produced wrong output

### 2. Extract the pattern (TRENCH lens)

For each bug, ask: "What general class of failure enabled this specific bug?" The pitfall is one level of abstraction above the bug.

Examples of the transform:
- Bug: "--version defaulted to v1" → Pitfall: "Subprocess flag inheritance gap"
- Bug: "vra_mode used for multi_constraint" → Pitfall: "Flag-as-mode conflation"
- Bug: "CA/TX 68% deviation" → Pitfall: "Weighting saturation"

The pitfall must be:
- Stated without reference to specific variable names or file lines
- Applicable to other systems with the same structural property
- Solvable by a structural change (not just "be more careful")

### 3. Check for duplicates

Read `design/pitfalls/README.md`. If the pattern already exists (possibly in different wording), update the existing entry rather than creating a duplicate.

### 4. Write the pitfall entry

Format:
```markdown
## {ID}: {Generic pattern name}

**Pattern:** {1-2 sentences describing the structural vulnerability — no specific variable names}

**Domain:** {What kind of system is vulnerable to this — why does it arise here?}

**Why it's hard to catch:** {Why normal code review or testing wouldn't surface it}

**Structural solution:** {What change makes the failure impossible — not just unlikely}

**Status:** OPEN / SOLVED / MITIGATED
**Proved by:** {Evidence if SOLVED}
**Test:** {Test file::test_name if SOLVED}
```

### 5. Update README.md status table

Add the new pitfall to the coverage table in `design/pitfalls/README.md`.

### 6. Update the domain file

Add the new pitfall to the appropriate domain file (`pitfalls-algorithm.md`, `pitfalls-pipeline.md`, etc.).

## Key Rules

- **Pattern, not bug.** "The --version flag defaulted to v1" is a bug. "Subprocess flag inheritance gap" is a pitfall.
- **Generic enough to catch the next instance.** If the pitfall is stated specifically enough that only the exact bug we fixed would match, it's still a bug report.
- **SOLVED means proven, not fixed.** A pitfall is SOLVED only when a test exists that would fail if the structural solution were removed.
- **Every session adds at least one.** If a session produced no bugs, it produced at least one observation about system fragility that belongs here.
