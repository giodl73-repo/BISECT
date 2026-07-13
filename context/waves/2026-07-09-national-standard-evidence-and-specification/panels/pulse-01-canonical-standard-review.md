# Pulse 01 Canonical Standard Review

**Date:** 2026-07-09  
**Artifact:** `docs/specs/2026-07-09-national-redistricting-standard-v0.1.md`  
**Roles:** MERIDIAN, BOUNDARY, DATUM, SCALE, COMMONS, COVENANT, SURVEY  
**Posture:** Internal panel review; not external peer review

## Blocking Findings And Disposition

| Finding | Disposition |
|---|---|
| `standard-bisect` does not execute convergence search | Canonical search changed to one precommitted seed; seed sensitivity moved to Layer D |
| Seed formula was under-specified and unimplemented | Canonical JSON, digest-to-u64 mapping, assignment-affecting fields, implementation status, and test vector added |
| Geographic baseline was described too neutrally | Renamed and described as a partisan-input-excluded geographic benchmark that encodes compactness values |
| Profiles and modifications could be selected after observing results | Added precommitment rules for assignment-affecting profiles, COI criteria, evaluation data, metrics, and stopping rules |
| VRA changes appeared permissive | VRA-required changes made mandatory; baseline safe-harbor and defense language prohibited |
| COI authority conflicted with the current model statute | Candidate standard now treats a precommitted COI rule as an authorized Layer C criterion and explicitly leaves statute reconciliation to Pulse 05 |
| Standards board had excessive assignment-changing authority | Assignment-affecting profile changes reserved to the enacting authority; technical custodian limited to ministerial patches |
| Federal service appeared to draw the operative map | Jurisdiction executes the reference implementation; federal role is technical custody and verification |
| Block normativity was mixed with tract computation | Blocks selected for normative computation and assignment; tract runs labeled exploratory/nonconforming |
| New artifact list duplicated existing manifests and audit certificates | Added logical-to-physical mapping and required reuse of RPLAN, audit-certificate, report, and manifest registries |
| Binary byte identity was operationally unrealistic | Conformance changed to canonical assignment and record hash equivalence |
| Input-exclusion claim lacked a verifier | Added required benchmark input-purity verification and an explicit current implementation gap |

## Accepted Carry-Forwards

- The current model statute conflicts with the candidate standard on blocks,
  COI modifications, fixed tolerance, seed governance, and baseline posture.
  Pulse 05 owns reconciliation.
- The manifest-derived seed, block-level execution path, planned schemas, and
  package conformance verifier are not implemented. Pulses 03 and later
  implementation waves own those gaps.
- External legitimacy requires a multi-stakeholder adversarial process and
  non-author replication. Pulse 06 owns that gate.
- Real ensemble evidence remains required before neutrality-percentile claims
  can be strengthened. Pulse 04 owns that gate.

## Review Decision

The candidate specification may close Pulse 01 as an internal L1 decision
record. It may not be represented as executable national conformance,
release-ready evidence, enacted law, or externally validated policy.

