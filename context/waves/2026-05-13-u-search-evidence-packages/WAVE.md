---
wave: u-search-evidence-packages
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-u-search-evidence-packages-goal.md
---

# U Search Evidence Packages

## Mission

Create auditable package evidence for U.2 parameter sensitivity and U.4 parallel
tempering while keeping the claim boundary below full 50-state sweep or
production CLI support until those artifacts exist.

## Claim Boundary

The first package is a synthetic evidence surface. It validates manifest,
parameter-sweep, and parallel-tempering audit shapes, but it does not replace a
real national sweep archive or a production `bisect` CLI run.

## Inputs

| Input | Source |
|---|---|
| Active goal | `docs/specs/2026-05-13-u-search-evidence-packages-goal.md` |
| Scorecard gap | `docs/papers/ALGORITHM-PAPER-SCORECARD.md` |
| Ensemble crate | `crates/bisect-ensemble/` |
| U paper sources | `research/tracks/U-search-optimization/` |
| Paper index | `docs/PAPERS.md` |

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Synthetic U.2/U.4 package | DONE | Added manifest, fixtures, verifier tests, and manifest docs |
| 02 - U ledger update | DONE | Updated U.2/U.4 papers, index, scorecard, and PDFs |
| 03 - Closeout | DONE | Archived first slice with carry-forwards for real sweep and CLI packages |

## Validation Gate

```powershell
cargo fmt
cargo test -p bisect-ensemble search_evidence
git diff --check
```

## Closeout

Completed. The wave delivered a synthetic U.2/U.4 package, verifier coverage,
manifest documentation, paper ledger updates, rebuilt PDFs, and explicit
carry-forwards for real 50-state sweep packages and a production
parallel-tempering CLI package.
