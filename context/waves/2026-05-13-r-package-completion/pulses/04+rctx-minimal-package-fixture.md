---
wave: r-package-completion
pulse: 04
status: done
depends_on: [03]
governing_roles:
  - rctx-core
  - rcount-district
  - rcount-core
  - docs
---

# Pulse 04 - RCTX Minimal Package Fixture

## Mission

Create the first minimal RCTX package fixture so RPLAN and RCOUNT can share
canonical unit context without copying geography semantics into their own
package records.

## Scope

| Surface | Target | Non-goal |
|---|---|---|
| RCTX fixture | Minimal package directory with unit context, source refs, exact crosswalk rows, and package hash. | Full national geography package. |
| Verifier | Reuse `rctx-core` exact rational checks and source-ref rules. | Large IO crate if directory loading is not yet needed. |
| RCOUNT consumer | Confirm district aggregation can preserve declared RCTX reference and crosswalk hash. | RPLAN end-to-end migration. |
| Docs | Update active goal and layer access patterns. | RMAP presentation work. |

## Pre-implementation Scout

Run and record:

```powershell
rg -n "RctxReference|Crosswalk|crosswalk_hash|rctx_reference|verify_crosswalk" crates docs
rg -n "RCTX|shared context|crosswalk|unit context" docs/specs docs/concepts
Get-ChildItem -Recurse crates/rctx-core
```

## Deliverables

- [x] Decide whether pulse 04 needs only fixture/docs or also `rctx-io`.
- [x] Add a tiny positive RCTX fixture or fixture-construction helper.
- [x] Add a negative fixture for missing source ref or bad crosswalk weight.
- [x] Add RCOUNT/RCTX consumer coverage where the existing APIs make sense.
- [x] Update `WAVE.md` pulse status and active goal checklist.
- [x] Run focused validation.

## Completion Notes

- Decision: no `rctx-io` crate for pulse 04. `rctx-core` now owns the minimal
  package fixture/helper and verifier path; directory loading stays deferred
  until there is an independent loader requirement.
- Positive fixture: `docs/fixtures/rctx/l0-shared-context`.
- Negative coverage: missing source ref and bad exhaustive crosswalk weight
  helpers in `rctx-core`.
- Consumer coverage: `rcount-district` preserves the fixture context and
  crosswalk hashes through district aggregation without absorbing RCTX graph or
  source ownership.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rctx-core -p rcount-core -p rcount-district -p rcount-io -p rcount-audit -p rcount-cli
git diff --check
```
