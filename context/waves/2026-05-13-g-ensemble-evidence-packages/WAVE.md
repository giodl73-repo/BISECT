---
wave: g-ensemble-evidence-packages
date_open: 2026-05-13
status: active
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
| 03 - Minimal evidence package | TODO | Build one minimal package or explicit missing-evidence package |
| 04 - Paper and ledger update | TODO | Update G.1-G.3, scorecard, paper index, and closeout docs after validation |

## Validation Gate

Run the focused package tests added by each pulse, then:

```powershell
cargo fmt
cargo test -p bisect-ensemble
git diff --check
```

## Next

Start pulse 03: build an explicit missing-evidence package for the absent
G.1-G.3 external traces and election/metric artifacts, unless real local
artifacts are found first.
