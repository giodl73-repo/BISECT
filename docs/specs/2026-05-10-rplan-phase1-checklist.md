# RPLAN Phase 1 Implementation Checklist

**Status:** In progress  
**Date:** 2026-05-10  
**Specs:** [`2026-05-10-rplan-v0.2-schema.md`](2026-05-10-rplan-v0.2-schema.md),
[`2026-05-10-plan-audit-certificates.md`](2026-05-10-plan-audit-certificates.md)

## Fixed Point

- [x] RPLAN public name fixed.
- [x] `.rplan` / `.rctx` split approved.
- [x] `rplan-*` crates may not depend on `bisect-*`.
- [x] RPLAN v0.2 schema approved for `rplan-core` / `rplan-io`.
- [x] U.20 approved for `rplan-audit` after core/io land.

## Implementation Order

- [x] Add `rplan-core` crate.
- [x] Add `rplan-io` crate.
- [x] Implement `rplan-core` plan/context/domain types.
- [x] Implement canonical JSON and SHA-256 hash helpers.
- [x] Implement unit-id validation by `UnitKind`.
- [x] Implement `DistrictPlan::plan_hash`.
- [x] Implement `RplanContext::context_hash`.
- [x] Implement `rplan-io` v0.2 read/write.
- [x] Implement `rplan-io` v0.1 compatibility reader.
- [x] Add `.rctx` 5-node path fixture round trip.
- [x] Add `bisect-report` adapter layer.
- [x] Add `rplan-audit`.
- [x] Add `rplan-cli`.

## Phase 1 Stop Line

Do not start U.16-U.19 algorithm-family work until:

- [x] `rplan-core` and `rplan-io` compile.
- [x] v0.1 compatibility conversion is tested.
- [x] v0.2 plan hash fixtures are stable.
- [x] `.rctx` context fixtures are stable.
- [x] `bisect-report` can adapt to the new crates without changing user behavior.
- [x] `rplan-audit` phase-1 plan-shape, population, and contiguity checks compile and pass fixture tests.
- [x] `rplan audit` CLI can emit a phase-1 audit certificate for a `.rplan` plus `.rctx`.
- [x] `.rctx` readers reject stale `context_hash` values.
- [x] `rplan audit` has CLI-level pass/fail/missing-profile regression tests.
