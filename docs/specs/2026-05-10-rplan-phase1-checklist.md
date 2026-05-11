# RPLAN Phase 1 Implementation Checklist

**Status:** Complete  
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
- [x] Emit `.rplan`, `.rctx`, and `audit-certificate.json` from `bisect-cli::runner`.
- [x] Record RPLAN sidecar paths, certificate hashes, audit result, legal profile, and context hash in `PlanManifest`.
- [x] Record source hashes in both `PlanManifest` and RPLAN provenance/context.
- [x] Surface RPLAN audit provenance in JSON and HTML reports.
- [x] Verify manifest source hashes against RPLAN context source hashes in `bisect verify`.

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
- [x] `bisect-cli::runner` writes audited RPLAN sidecars before manifest write.
- [x] `bisect verify` checks RPLAN certificate freshness, certificate content hash,
      audit result, legal profile, context hash, and manifest/context source-hash
      consistency.
- [x] `bisect report` displays RPLAN sidecar paths, certificate identity, audit
      result, context hash, and `.rctx` source hashes.

## Phase 1 Completion Notes

Phase 1 now reaches the U.20 fixed point:

- `rplan-core`, `rplan-io`, `rplan-audit`, and `rplan-cli` provide the generic
  plan/context/audit contract.
- `bisect-cli::runner` emits audited `.rplan` and `.rctx` sidecars and records
  their certificate metadata in the legacy `PlanManifest`.
- Source provenance is tied together across the manifest, `.rplan` provenance,
  `.rctx` source hashes, audit certificates, reports, and `bisect verify`.

Remaining work belongs to later algorithm-family stages, not Phase 1:

- richer solver-specific `algorithm_lineage` payloads for U.16+;
- frontier-wide certificates for Pareto/Sampling consumers;
- standalone RPLAN repository promotion after interface stabilization.
