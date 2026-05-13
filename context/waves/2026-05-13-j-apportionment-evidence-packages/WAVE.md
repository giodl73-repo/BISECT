---
wave: j-apportionment-evidence-packages
date_open: 2026-05-13
status: active
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
| 02 - J ledger update | TODO | Update J.0-J.6 paper/index/scorecard references to package evidence |
| 03 - Historical carry-forward | TODO | Decide 2010/2000/1910-2020 package scope or close with explicit gaps |

## Validation Gate

```powershell
cargo fmt
cargo test -p bisect-apportion
git diff --check
```

## Next

Start pulse 02: update J.0-J.6 paper/index/scorecard references to the 2020
Census Table 1 evidence package.
