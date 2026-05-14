---
wave: g-active-ensemble-evidence-packages
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-g-active-ensemble-evidence-packages-goal.md
---

# G Active Ensemble Evidence Packages

## Mission

Add an active synthetic G.1-G.3 evidence package that covers all artifact roles
needed by external-ensemble percentile claims while keeping real empirical
claims gated on real trace/election/metric packages.

## Claim Boundary

The package is active but synthetic. It validates package shape, hash binding,
and role completeness; it does not prove real external ensemble percentiles.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Active synthetic package | DONE | Added trace/election/metric/diagnostic/RPLAN/RCTX package and verifier test |
| 02 - G ledger update | DONE | Updated G.1/G.2/G.3 papers, index, scorecard, and PDFs |
| 03 - Closeout | DONE | Archived wave with carry-forwards for real external packages |

## Validation Gate

```powershell
cargo fmt
cargo test -p bisect-ensemble active_g1_g3_package_fixture_validates
git diff --check
```

## Closeout

Completed. The wave delivered a docs-level active synthetic G.1-G.3 package,
hash verifier coverage, G.1/G.2/G.3 paper ledger updates, rebuilt PDFs, and
explicit carry-forwards for real external trace, election, metric, diagnostic,
RPLAN, and RCTX packages.
