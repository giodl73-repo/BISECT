---
wave: g-ensemble-evidence-packages
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-g-ensemble-evidence-packages-goal.md
---

# G Ensemble Evidence Packages

## Mission

Create auditable evidence packages for the G.1-G.3 ensemble comparison papers so
compactness percentiles, partisan outcome positions, and metric-distribution
claims can be traced to concrete artifacts rather than paper-only tables.

## Claim Boundary

This wave may add manifests, fixtures, validators, package docs, tests, and
paper updates. It must not claim that external traces exist unless they are
present, hash-bound, and validated. Missing evidence should be represented as an
explicit manifest gap, not by silently weakening checks.

## Inputs

| Input | Source |
|---|---|
| Active goal | `docs/specs/2026-05-13-g-ensemble-evidence-packages-goal.md` |
| Prior closeout | `context/waves/2026-05-13-paper-rubric-uplift/CLOSE.md` |
| G paper sources | `research/tracks/G-ensemble/` |
| Scorecard | `docs/papers/ALGORITHM-PAPER-SCORECARD.md` |
| Paper index | `docs/PAPERS.md` |
| Ensemble crate | `crates/bisect-ensemble/` |
| RPLAN/RCTX crates | `crates/rplan-*`, `crates/rctx-*` |

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Evidence inventory and manifest contract | DONE | Added `bisect-ensemble::evidence_manifest` contract and documented missing-evidence handling |
| 02 - Validator and fixtures | DONE | Added synthetic positive/negative fixtures and referenced-file SHA-256 validation |
| 03 - Minimal evidence package | DONE | Added hash-bound `G.1-G.3+missing-evidence` package and validator coverage |
| 04 - Paper and ledger update | DONE | G.1-G.3 papers, scorecard, and public index point at the missing-evidence package |

## Validation Gate

Run the focused package tests added by each pulse, then:

```powershell
cargo fmt
cargo test -p bisect-ensemble
git diff --check
```

## Closeout

Completed. The wave produced a manifest contract, synthetic positive/negative
fixtures, referenced-file hash validation, a hash-bound G.1-G.3 missing-evidence
package, and paper/index/scorecard links to that package.
