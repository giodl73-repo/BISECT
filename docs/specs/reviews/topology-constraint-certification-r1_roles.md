---
reviewer: ROLE PANEL
roles: BOUNDARY, WARD, COVENANT, CONTOUR, MERIDIAN, BENCHMARK, SCALE, PRECINCT, DATUM, COMMONS, LEDGER, SURVEY, TRENCH
spec: U.20 Topology Constraint Certification
round: 1
date: 2026-05-10
score: 3
---

# Role Review: U.20 Topology Constraint Certification

## Summary

The U.20 spec is the right first implementation contract for the new algorithm
families. It correctly makes certification a shared crate rather than a helper
buried in one solver, and it blocks later algorithm crates until plan identity,
legal profiles, source hashes, and certificate output are stable.

Round 1 found four concrete issues in the draft:

- certificate structs referenced `LegalProfileSummary` and
  `CertificationWarning` without defining them
- certificate hashing was ambiguous because `generated_at_utc` and
  `certificate_id` are inherently unstable
- graph vertex ordering was not explicitly tied to plan unit ordering
- the congressional population default could be misread as a legal safe harbor

Those items have been patched into the spec. The remaining work is mostly
implementation detail: canonical JSON, fixture files, legal-profile data, and
how `bisect-cli` loads source hashes.

Decision: **approved as implementation-start spec for U.20 phase 1**, limited
to plan shape, population, contiguity, certificate JSON, and canonical fixtures.
Subdivision, VRA, and geometry checks should remain later rollout phases.

## Scores

| Role | Score | Reason |
|------|-------|--------|
| BOUNDARY | 3/4 | Legal-profile framing is sound after clarifying project profile vs legal guarantee. |
| WARD | 3/4 | State/chamber profiles are required for state legislative certification; concrete rule table remains future work. |
| COVENANT | 3/4 | Certificate provenance is strong after adding stable content hash; release signing remains outside this spec. |
| CONTOUR | 3/4 | Unit identity and graph ordering are now explicit; loader source-hash ownership still needs implementation detail. |
| MERIDIAN | 3/4 | Contiguity and graph/unit alignment are correct; geometry/topology checks appropriately deferred. |
| BENCHMARK | 3/4 | Canonical fixtures and L0/L1 checks are sufficient for phase 1; expected JSON goldens still need to be created. |
| SCALE | 3/4 | Spec avoids statistical claims; future algorithm comparison metrics belong in family specs. |
| PRECINCT | 3/4 | VRA/partisan effects are not overclaimed; VRA is framed as reporting, not liability determination. |
| DATUM | 3/4 | Scope is honest and falsifiable; certificate examples should be added before final approval. |
| COMMONS | 2/4 | COI/community cohesion is not yet represented except through subdivisions; acceptable for phase 1 but incomplete. |
| LEDGER | 3/4 | Versioned schemas are present; canonical JSON/RPLAN details need a separate concrete schema file. |
| SURVEY | 3/4 | CLI shape and exit codes are practical after strict warning behavior was added. |
| TRENCH | 3/4 | Major failure modes are named; pitfall records should be added when code starts. |
| **Average** | **3.0/4** | Ready for phase 1 implementation, not complete for all later checks. |

## P1 Items Resolved In Spec

### P1-A: Undefined certificate support types

Raised by: BENCHMARK, LEDGER

The draft referenced `LegalProfileSummary` and `CertificationWarning` without
defining them. The spec now defines both and requires warning codes, severity,
message, and affected check.

### P1-B: Certificate hash instability

Raised by: COVENANT, BENCHMARK

The draft required stable output hashes but also included `generated_at_utc` and
`certificate_id`. The spec now defines `content_hash`, computed from canonical
JSON excluding volatile certificate metadata.

### P1-C: Graph order vs plan order was implicit

Raised by: CONTOUR, MERIDIAN, TRENCH

The draft said `assignment[i]` follows `unit_ids[i]`, but did not require graph
vertex `i` to match that same unit id. The spec now adds `graph_unit_ids` and
requires equality after canonicalization.

### P1-D: Congressional population default could be overstated

Raised by: BOUNDARY, WARD

The draft stated a congressional ±0.5% default. The spec now labels that as the
existing project profile and explicitly says the certificate certifies the
profile applied, not universal legal validity.

### P1-E: Requested missing-input checks needed severity rules

Raised by: BOUNDARY, COVENANT, BENCHMARK

The draft had `MissingInput` status but did not say whether missing input passes,
warns, or fails. The spec now says requested checks missing input are `Error`
unless the legal profile marks the check advisory.

## Remaining P2 Items

### P2-A: Add canonical JSON rules

Raised by: LEDGER, COVENANT

The implementation should define canonical JSON concretely, preferably by a
small local canonicalizer rather than relying on serde map order. This should be
documented in the implementation PR or a schema appendix.

### P2-B: Add example certificates

Raised by: DATUM, BENCHMARK, SURVEY

Before marking U.20 accepted, add two examples:

- valid 3x3 fixture certificate
- disconnected path-graph failure certificate

### P2-C: Add legal-profile seed files

Raised by: BOUNDARY, WARD

Phase 1 needs at least:

- `profiles/us-congressional-project-v1.json`
- one intentionally incomplete state-legislative profile fixture proving the
  CLI refuses state legislative certification without a real profile

### P2-D: Add pitfall records when code starts

Raised by: TRENCH

Create pitfall entries for:

- plan assignment order drift
- certificate hash including volatile fields
- missing-input checks accidentally passing
- project profile mistaken for legal guarantee
- graph vertex order not matching plan unit order

### P2-E: Add COI hooks later

Raised by: COMMONS

Subdivision split checks are not enough for community-interest reporting.
Later phases should allow Track M community-character witnesses, especially for
clustering/regionalization algorithms.

## Role Notes

### BOUNDARY

The spec now properly says certification is against an applied profile, not an
all-purpose legal conclusion. That distinction must survive implementation and
report text.

### WARD

Requiring state/chamber profiles for state legislative certification is the
right gate. Do not let `CountOnly` subdivision reporting be displayed as
state-law compliance.

### COVENANT

`content_hash` fixes the main reproducibility issue. Full admissibility still
requires release/binary provenance outside this crate, but the fields are ready.

### CONTOUR

`graph_unit_ids` closes the GEOID-ordering gap. Loaders must preserve leading
zeros and reject duplicate unit ids before certification.

### MERIDIAN

Contiguity via induced subgraphs is the right first check. Island handling must
remain explicit; do not make isolated units pass by accident in multi-unit
districts.

### BENCHMARK

The fixture set is good enough for phase 1. The first implementation PR should
add JSON golden tests for `content_hash`, warning severity, and disconnected
witnesses.

### SCALE

No statistical finding is made here. Future algorithm-family papers should not
treat certificate pass/fail as evidence that an algorithm is superior.

### PRECINCT

VRA reporting is correctly limited to demographic/opportunity reporting. Do not
extend this into claims about partisan fairness in the certificate.

### DATUM

The spec is now falsifiable: a malformed plan, disconnected plan, missing VAP
input, and unstable certificate hash should all fail specific tests.

### COMMONS

The community lens remains thin. Add COI witnesses later rather than blocking
phase 1.

### LEDGER

Versioned schema names are present. The implementation should pin exact JSON
field names and canonicalization before external export promises are made.

### SURVEY

The CLI is usable. The strict warning behavior is important: courts and CI need
a mode where incomplete provenance is not silently accepted.

### TRENCH

The spec now prevents several likely failures structurally. The pitfall entries
should be created when implementation starts so the protections do not drift.

