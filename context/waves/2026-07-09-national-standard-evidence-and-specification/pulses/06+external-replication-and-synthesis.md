---
pulse: 06
title: External replication and synthesis
status: done
depends_on: 02, 03, 04, 05
wave: national-standard-evidence-and-specification
validation_level: external handoff
---

# Pulse 06 - External Replication And Synthesis

## Purpose

Package the canonical standard, evidence, and legal posture for independent,
hostile replication and bounded public communication.

## Deliverables

- [x] External replication protocol with no private project knowledge needed.
- [x] Challenge bundle containing source tag, build recipe, input manifests,
      reference outputs, ensemble evidence, and known limitations.
- [x] Reviewer recruitment criteria spanning redistricting, civil rights,
      federalism, election administration, statistics, and reproducible
      software.
- [x] Public issue and response process for replication failures.
- [x] Revised synthesis paper and policy brief.
- [x] Adoption matrix for commissions, courts, Congress, state staff, and
      civic groups.

## Validation

- A fresh external-style operator can execute the documented reference replay.
- All public claims map to a source artifact and evidence posture.
- `git --no-pager diff --check`.

Results:

- Fresh automated non-author replay reproduced RI raw and canonical assignment
  hashes and obtained `label-verify: VERIFIED`.
- Blocked precursor runs exposed stale Census URLs, missing release custody,
  inconsistent/malformed overlays, and missing pipeline stages; each was fixed
  and preserved in the challenge history.
- The final literal script acquired the full public adjacency triplet and
  verified the chain.
- Challenge and ensemble bundle verifiers passed.
- A.0 and A.5 were rebuilt with the Exact Canonical Benchmark North Star.

## Evidence

- `docs/external/nrs-v0.1-challenge-bundle/`
- `docs/specs/2026-07-10-exact-canonical-benchmark-north-star.md`
- `docs/papers/A.5+policy-brief.pdf`

## Closure Rule

Closed after an automated non-author `pass_candidate` replication. DCR-003
human external-user validation remains open, and the posture remains internal
external-review candidate rather than public release, legal approval, peer
review, or exact certification.
