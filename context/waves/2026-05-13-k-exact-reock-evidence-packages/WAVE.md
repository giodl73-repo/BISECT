---
wave: k-exact-reock-evidence-packages
date_open: 2026-05-13
status: active
source_goal: docs/specs/2026-05-13-k-exact-reock-evidence-packages-goal.md
---

# K Exact Reock Evidence Packages

## Mission

Create exact polygon-MBC Reock evidence for K-track papers while preserving the
current production Reock proxy as a separately documented metric.

## Claim Boundary

The exact-MBC path is an evidence/reference surface. The production
`bisect-analysis::reock()` function remains the centroid-radius proxy used by
`all_metrics()` until a deliberate compatibility-breaking metric migration is
scoped.

## Inputs

| Input | Source |
|---|---|
| Active goal | `docs/specs/2026-05-13-k-exact-reock-evidence-packages-goal.md` |
| Prior carry-forward | `context/waves/2026-05-13-paper-rubric-uplift/CLOSE.md` |
| Compactness crate | `crates/bisect-analysis/src/compactness.rs` |
| K paper sources | `research/tracks/K-compactness/` |
| Paper index | `docs/PAPERS.md` |
| Scorecard | `docs/papers/ALGORITHM-PAPER-SCORECARD.md` |

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Exact-MBC smoke package | DONE | Added exact MBC helper, smoke fixtures, manifest, and verifier tests |
| 02 - K ledger update | TODO | Update K.0/K.2/K.7 papers, index, scorecard, and PDFs |
| 03 - Closeout | TODO | Archive wave with carry-forwards for real district polygon packages |

## Validation Gate

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis
git diff --check
```
