---
wave: r-package-completion
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-r-package-completion-goal.md
---

# R Package Completion

## Mission

Finish the civic evidence R-package family without collapsing ownership
boundaries. RCOUNT is the first mature package; RCTX and RHIST are the base
dimensions that prevent geography and lineage semantics from being rewritten in
every consumer.

## Claim Boundary

This wave may land executable RCOUNT replay and package-reference surfaces. It
must not treat analytic, privacy, or coordinator transcripts as certification
math unless the method-specific replay and validation fixture are present.

## Inputs

| Input | Source |
|---|---|
| Active goal | `docs/specs/2026-05-13-r-package-completion-goal.md` |
| Package family spec | `docs/specs/2026-05-13-civic-evidence-package-family.md` |
| Layer access patterns | `docs/specs/2026-05-13-civic-evidence-layer-access-patterns.md` |
| RCOUNT audit roadmap | `docs/specs/2026-05-13-rcount-audit-algorithm-roadmap.md` |
| Algorithm atlas | `docs/algorithm-atlas/` |

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - RCOUNT audit replay substrate | DONE | `pulses/01+rcount-audit-replay-substrate.md`; V.12/V.13/V.14/V.15/V.17/V.18 docs and tests |
| 02 - Ranked-choice audit boundary | DONE | `pulses/02+ranked-choice-audit-boundary.md`; V.19 RAIRE/AWAIRE fixtures and tests |
| 03 - Analytic and observable boundary methods | DONE | `pulses/03+analytic-observable-boundaries.md`; V.20 Bayesian and V.21 SOBA fixtures and tests |
| 04 - RCTX minimal package fixture | DONE | `pulses/04+rctx-minimal-package-fixture.md`; `docs/fixtures/rctx/l0-shared-context`; `rctx-core` fixture verifier and RCOUNT consumer coverage |
| 05 - RHIST minimal lineage fixture | DONE | `pulses/05+rhist-minimal-lineage-fixture.md`; L2 rename/split/merge fixture lock, manifest hash verifier, and RCOUNT RHIST package-hash reference coverage |
| 06 - RCOUNT/RPLAN reference integration | DONE | `pulses/06+reference-integration.md`; shared RCOUNT fixture references, regenerated RPLAN example extension, and IO/aggregation coverage |

## Validation Gate

Run the focused workspace suite after every implementation pulse:

```powershell
$env:CARGO_INCREMENTAL='0'; cargo test -p rctx-core -p rcount-stats -p rcount-core -p rcount-io -p rcount-audit -p rcount-district -p rcount-cli
```

Also run:

```powershell
cargo fmt
git diff --check
```

`git diff --check` may report existing CRLF normalization warnings. Whitespace
errors are not acceptable.

## Next

All planned pulses are complete. Carry forward RMAP/RAUDIT/RCERT/RSTAT/RLOG/
RCHAIN/RROLL/RCASE activation decisions to the next wave or close review, and
keep `rctx-io` deferred until an independent RCTX package directory loader is
needed.

## Close Summary

This wave completed the RCOUNT audit replay/boundary tranche and stabilized the
first shared RCTX/RHIST base-package references. RCOUNT can now replay or
boundary-report the V.12/V.13/V.14/V.15/V.17/V.18/V.19/V.20/V.21 audit surfaces
named in this wave, while RCTX and RHIST have independent minimal fixtures,
package/hash shapes, verifier coverage, and RCOUNT/RPLAN reference examples.

The wave deliberately did not activate RMAP, RAUDIT, RCERT, RSTAT, RLOG,
RCHAIN, RROLL, or RCASE beyond documented defer/activate boundaries. Those
packages should move only when a second consumer or real-source access pattern
requires them.
