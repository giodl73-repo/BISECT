---
name: r-review
description: Review an apportionment wave pulse or implementation using package-boundary roles and write panel findings.
user-invocable: true
---

# r-review

Run a lightweight review panel for wave pulses.

## Usage

```text
/r-review pulse 04
/r-review code crates/rcount-core
```

## Built-In Roles

Use the roles that match the work:

| Role | Lens |
|---|---|
| rcount-core | Package schema and verifier invariants |
| rcount-stats | Deterministic numerical replay and exact rational arithmetic |
| rcount-audit | Transcript status, claim boundary, and replay honesty |
| rcount-io | Package directory, source hashes, and round-trip behavior |
| rcount-cli | User-facing command behavior and exit codes |
| rctx-core | Canonical unit context, crosswalks, and source refs |
| rhist | Cross-cycle lineage ownership |
| docs | Atlas, roadmap, and active goal consistency |

## Output

Write findings to:

```text
context/waves/{active}/panels/{review-name}/R1-{role}.md
context/waves/{active}/panels/{review-name}/R1-consolidated.md
```

## Finding Format

```markdown
### F-01 - BLOCK: title
File: path
Finding: what is wrong
Consequence: what breaks
Fix: concrete recommendation
```

Severity:

- `BLOCK`: must fix before marking pulse done.
- `WARN`: should fix or explicitly defer.
- `NOTE`: useful carry-forward.

## Rules

- Findings lead; summaries follow.
- Cite files and exact local paths where possible.
- Do not perform implementation edits during review.

