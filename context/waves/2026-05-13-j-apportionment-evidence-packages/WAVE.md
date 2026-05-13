---
wave: j-apportionment-evidence-packages
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-j-apportionment-evidence-packages-goal.md
---

# J Apportionment Evidence Packages

## Mission

Create auditable Census/SHA packages for J-track apportionment claims, beginning
with 2020 Census Table 1 and the Method of Equal Proportions verifier.

## Claim Boundary

This wave may add manifests, extracted official-data fixtures, verifier tests,
and paper ledger updates. It must not imply full historical replay until every
historical source table is hash-bound and validated.

## Inputs

| Input | Source |
|---|---|
| Active goal | `docs/specs/2026-05-13-j-apportionment-evidence-packages-goal.md` |
| Prior carry-forward | `context/waves/2026-05-13-paper-rubric-uplift/CLOSE.md` |
| Apportionment crate | `crates/bisect-apportion/` |
| J paper sources | `research/tracks/J-apportionment/` |
| Paper index | `docs/PAPERS.md` |
| Scorecard | `docs/papers/ALGORITHM-PAPER-SCORECARD.md` |

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - 2020 Census Table 1 verifier | DONE | Added source-SHA manifest, extracted rows, and Huntington-Hill replay coverage |
| 02 - J ledger update | DONE | Updated J.0/J.1/J.6 papers, public index, and scorecard to cite the 2020 package |
| 03 - Historical carry-forward | DONE | Closed wave with 2000/2010 packages and CLI verifier as carry-forwards |

## Validation Gate

```powershell
cargo fmt
cargo test -p bisect-apportion
git diff --check
```

## Closeout

Completed. The wave delivered a 2020 Census Table 1 source-SHA package,
Huntington-Hill verifier coverage, J.0/J.1/J.6 ledger updates, rebuilt PDFs, and
explicit carry-forwards for 2000/2010 packages and a standalone CLI verifier.
