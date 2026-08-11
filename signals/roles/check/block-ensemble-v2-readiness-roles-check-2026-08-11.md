---
skill: roles-check
topic: block-ensemble-v2-readiness
date: 2026-08-11
roles_used:
  - covenant
  - contour
  - benchmark
  - trench
  - datum
  - survey
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

# Role Check: Block-Ensemble v2 Readiness

## Artifact and scope

Review of the Pulse 31 input audits, release-executable custody, capacity
snapshot, pristine v2 package, readiness verifier, and operator documentation.
The approval applies only to the local pre-Stage-0 readiness gate. It does not
approve a process launch or any ensemble claim.

## Findings

### Covenant — evidence and audit custody

1. **Resolved P1:** three Pulse 24 manifest hashes described CRLF bytes while
   repository checkout policy materialized canonical LF bytes. Recomputed CRLF
   hashes exactly matched the former manifest entries; only the three manifest
   hashes were repaired, and the full retained-package verifier passes.
2. **Pass:** readiness binds every fresh audit, both local executables, their
   source files, the v2 wrapper/verifier, capacity checker, frozen protocol, and
   repaired resource manifest by SHA-256.
3. **Pass:** the launcher now rejects alternate executable paths or hash drift
   before process creation, and completed-resource verification requires the
   same readiness-bound digest.
4. **Condition:** executable hashes establish custody on this author machine;
   without an independent rebuild or signature they are not a cross-host
   reproducible-build attestation. The package states this boundary.

### Contour — data provenance

1. **Pass:** NH, NM, and GA were freshly audited against the exact RCTX and
   baseline-assignment paths consumed by the runner.
2. **Pass:** the readiness verifier recomputes both live-file hashes for every
   State and freezes units, edges, population, districts, year, and starting
   population deviation.
3. **Condition:** these audits bind derived inputs; upstream Census acquisition
   and derivation remain covered by their existing custody packages and were not
   independently reconstructed in Pulse 31.

### Benchmark — tests and verification

1. **Pass:** the official readiness package passes alongside the resource and
   empty-v2 verifiers.
2. **Pass:** a tampered input-audit value is rejected before execution.
3. **Pass:** executable hash drift and the appearance of any process artifact
   are independently rejected; the focused admission/v2 suite totals 25 passing
   tests.

### Trench — failure modes

1. **Pass:** the package must remain at the exact zero-completion ledger state;
   stale admissions, resources, preflights, primaries, or replays invalidate the
   readiness snapshot.
2. **Pass:** the 99.14 GiB free-space observation exceeds the frozen 8 GiB
   requirement and its arithmetic is reverified.
3. **Condition:** disk availability is time-varying. The snapshot is explicitly
   non-reusable and the runner still requires fresh fail-closed admission before
   every process.

### Datum — methodology and claim scope

1. **Pass:** the readiness record is attached to the fresh v2 seed/protocol and
   cannot admit v1 identity or completions.
2. **Pass:** no preflight, primary, or replay ran, so readiness is not presented
   as sampling, convergence, mixing, or inferential evidence.
3. **Condition:** Stage 0 remains excluded diagnostic work. Even a later six-pair
   pass can establish executable/input consistency only, not ensemble validity.

### Survey — operational use

1. **Pass:** the package README gives the two non-executing verifier commands.
2. **Pass:** the operator is told that every launch needs a fresh immutable
   admission attempt and that rejected attempts cannot be overwritten.
3. **Pass:** limits are operationally legible: 21 cumulative governed hours,
   2.25 GiB per process, 3 GiB retained, 3 GiB scratch, and 2 GiB safety reserve.

## Cross-role synthesis

The artifact is ready for a later, explicit Stage 0 decision. Custody is
machine-checkable, the prior line-ending inconsistency is repaired without
changing evidence semantics, live derived inputs match their audits, and the
package is demonstrably pristine. The remaining conditions are boundaries,
not open implementation defects: local binaries are not independently
reproduced, derived inputs are not freshly rebuilt from upstream sources, and
capacity must be remeasured for every process.

## Amendments incorporated

1. Added a dedicated readiness verifier with exact live-file and executable
   bindings plus adversarial tamper tests.
2. Enforced the readiness-bound executable at both launch and retained-resource
   verification boundaries.
3. Recast the host-capacity result as a point-in-time observation that can never
   substitute for per-process admission.
4. Added explicit author-machine, derived-input, and no-statistical-result claim
   boundaries to the package and Pulse 31 record.

## Verdict

**APPROVED-WITH-CONDITIONS** for local pre-Stage-0 readiness only. Open P1 count:
zero. A process launch remains a separate action and must pass fresh admission.
