---
wave: j-divisor-method-evidence
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-j-divisor-method-evidence-goal.md
---

# J Divisor Method Evidence

## Mission

Promote J.2--J.5 divisor-method and paradox-immunity claims from source/test
references to an auditable, hash-bound smoke evidence package.

## Claim Boundary

This wave adds a synthetic positive fixture and verifier coverage for the
implemented Webster, Adams, Jefferson, and shared divisor Alabama-paradox paths.
It does not claim a full historical Census replay for alternate divisor methods.

## Inputs

| Input | Source |
|---|---|
| Active goal | `docs/specs/2026-05-13-j-divisor-method-evidence-goal.md` |
| Prior J package wave | `context/waves/2026-05-13-j-apportionment-evidence-packages/WAVE.md` |
| Apportionment crate | `crates/bisect-apportion/` |
| J paper sources | `research/tracks/J-apportionment/` |
| Paper index | `docs/PAPERS.md` |
| Scorecard | `docs/papers/ALGORITHM-PAPER-SCORECARD.md` |

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Divisor method smoke package | DONE | Added `j-divisor-evidence-manifest v1`, fixture replay, tamper rejection, J.2--J.5 paper updates, and rebuilt PDFs |

## Validation Gate

```powershell
cargo fmt
cargo test -p bisect-apportion divisor_method_smoke
cargo test -p bisect-apportion tampered_divisor_fixture_rejected
git diff --check
```

## Closeout

Completed. The wave delivered a hash-bound synthetic divisor-method package and
consumer coverage for Webster, Adams, Jefferson, and shared divisor
Alabama-paradox immunity. Historical alternate-method Census packages and a
standalone CLI verifier remain future work.
