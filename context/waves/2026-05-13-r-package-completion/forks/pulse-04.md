# Fork Context: Pulse 04 - RCTX Minimal Package Fixture

> Generated as the first apportionment wave fork. Read this file completely
> before acting. The pulse plan is the execution contract.

## Pulse Plan

```markdown
---
wave: r-package-completion
pulse: 04
status: todo
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

- [ ] Decide whether pulse 04 needs only fixture/docs or also `rctx-io`.
- [ ] Add a tiny positive RCTX fixture or fixture-construction helper.
- [ ] Add a negative fixture for missing source ref or bad crosswalk weight.
- [ ] Add RCOUNT/RCTX consumer coverage where the existing APIs make sense.
- [ ] Update `WAVE.md` pulse status and active goal checklist.
- [ ] Run focused validation.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rctx-core -p rcount-core -p rcount-district -p rcount-io -p rcount-audit -p rcount-cli
git diff --check
```
```

## Governing Roles

### rctx-core

Owns canonical unit context, source references, exact rational crosswalk rows,
and crosswalk hash semantics. This role should reject any RCOUNT or RPLAN code
that invents canonical geography ids outside the shared context layer.

### rcount-district

Owns RCOUNT aggregation over plan/district/context references. This role checks
that RCOUNT consumes RCTX references without absorbing RCTX ownership.

### rcount-core

Owns package schema, reference structs, fixture helpers, and verifier equation
passes. This role checks positive and negative fixture coverage.

### docs

Owns claim boundaries and roadmap consistency. This role checks that the active
goal, layer access patterns, and package-family docs describe exactly what the
code verifies.

## Execution Contract

- Work in repository root `c:\src\apportionment`.
- Use `rg` for search.
- Use `apply_patch` for manual edits.
- Do not revert unrelated user changes.
- Prefer a minimal fixture/helper if `rctx-io` is not yet justified.
- Check off pulse gates only after they are complete.
- Run the validation commands before marking the pulse done.

