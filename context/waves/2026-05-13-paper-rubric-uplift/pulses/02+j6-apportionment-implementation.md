---
wave: paper-rubric-uplift
pulse: 02
status: complete
depends_on: [01]
governing_roles:
  - MERIDIAN
  - DATUM
  - BENCHMARK
  - LEDGER
---

# Pulse 02 - J.6 Apportionment Implementation Pass

## Mission

Lift J.6 out of the "known stale implementation claims" band by aligning the
paper with the current `bisect-apportion` crate. The paper must distinguish
implemented apportionment APIs from the wider prime-factor redistricting
compositor that shares the crate.

## Scope

| Surface | Target | Non-goal |
|---|---|---|
| J.6 paper | Current public APIs, arithmetic/tie boundaries, verification status, and test inventory. | Rewriting apportionment algorithms. |
| J.0-J.5 bridge | Propagate only necessary implementation-bridge caveats. | Full mathematical rewrite of each method paper. |
| Ledgers | Update scorecard/review ledger and wave docs. | Claiming external peer review. |
| Build artifacts | Rebuild changed PDFs and copy to `docs/papers/`. | Repository-wide paper rebuild. |

## Pre-implementation Scout

Run and record:

```powershell
rg -n "huntington_hill|apportionment_divisor|check_alabama_paradox|RoundingRule|Hamilton|SHA-256|priority|verified" crates/bisect-apportion research/tracks/J-apportionment docs/papers docs/specs
Get-ChildItem -Recurse crates/bisect-apportion
git --no-pager status --short
```

## Deliverables

- [x] Materialize this pulse and fork context.
- [x] Decide whether the pulse is paper-only or requires apportionment code.
- [x] Revise J.6 source so API, verification, tests, and limitations match the crate.
- [x] Propagate implementation bridge caveats into J.0-J.5 where needed.
- [x] Update `PAPER-QUALITY-REVIEW.md`, `ALGORITHM-PAPER-SCORECARD.md`, `WAVE.md`, and the active goal.
- [x] Rebuild changed PDFs and copy them to `docs/papers/`.
- [x] Run focused validation and commit.

## Completion Notes

- Decision: paper/docs-only. Scout found no `bisect-apportion` implementation
  bug in scope; the issue was stale paper claims.
- Current boundary recorded: four divisor methods (`HuntingtonHill`, `Webster`,
  `Adams`, `Jefferson`), `f64` priority comparisons, library APIs only, 99 crate
  tests, and missing Census-reference/SHA verifier fixtures.
- Bridge caveats propagated into J.0-J.5 implementation-facing sections.

## Validation

```powershell
cargo test -p bisect-apportion
cd research\tracks\J-apportionment\J.6+bisect-apportion-implementation; pdflatex -interaction=nonstopmode main.tex; bibtex main; pdflatex -interaction=nonstopmode main.tex; pdflatex -interaction=nonstopmode main.tex
git diff --check
```
