---
name: update-invariants
description: Write or update system invariants with tests. An invariant is a property that must always be true. Run after major architectural changes or when a new class of bugs reveals an unguarded property.
user_invocable: true
---

# Update System Invariants

Invariants are properties the system must always satisfy. Unlike pitfalls (which document failure patterns), invariants document what must be true — and prove it with a test.

## Input

Describe the context: what part of the system, what changed, what must remain true.

## Steps

### 1. Read existing invariants

Read `design/invariants/README.md`. Understand what is already covered.

### 2. Identify new invariants

From the context, identify properties that:
- Are non-obvious (a reader wouldn't assume them without being told)
- Must hold for correctness or constitutional compliance
- Have been violated before (or could be violated in the next refactor)
- Are not currently tested

### 3. Write the invariant

Format:
```markdown
## {ID}: {Short name}

**Invariant:** {One sentence: what is always true}

**Why it must hold:** {Constitutional, mathematical, or contractual reason}

**When it can be violated:** {What class of change could break it}

**Enforcement:** {How it is currently enforced — assertion, type system, test}

**Test:** {test_file::test_name}
**Status:** ENFORCED / PARTIAL / OPEN
```

### 4. Add or update the test

Every invariant must have a test that fails if the invariant is violated. The test should be:
- Named `test_invariant_{what}_{property}` or similar
- In `tests/unit/` for pure computation invariants
- In `tests/acceptance/` for end-to-end constitutional invariants

### 5. Update README

Add to `design/invariants/README.md` status table.

## Invariant domains

- **Constitutional**: population balance, contiguity, district count
- **VRA**: MM threshold, adaptive boost formula, vra_mode stays True
- **Data**: GEOID format, edge weight canonical order, adjacency symmetry
- **Algorithm**: bisection completeness, target weight correctness, leaf sort order
- **Interface**: ASCII-only STATUS output, atomic write, error propagation

## Key rules

- **Invariant, not test.** "assert mm_count >= 0" is a test. "MM threshold is exclusive (>), not inclusive (>=)" is an invariant.
- **ENFORCED means the test would catch a violation.** Temporarily break the enforcement mechanism; the test must fail.
- **Invariants outlive code.** When you refactor, re-read the invariants. The invariant doesn't change; only the enforcement mechanism does.
