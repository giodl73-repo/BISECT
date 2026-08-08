# Pulse 07 National Bakeoff Review

**Date:** 2026-08-07  
**Roles:** MERIDIAN, CONTOUR, DATUM, SCALE, BOUNDARY, BENCHMARK, SURVEY  
**Posture:** protocol review before result generation

## Blocking Findings

- MERIDIAN: do not compare plans on different atomic-unit universes or use
  tract-count population proxies. NRS generation remains untouched.
- CONTOUR: every comparator must use the matching Census geography vintage.
  Cross-cycle assignment rankings are prohibited without a published
  relationship crosswalk or geometry allocation rule.
- DATUM: metrics, comparators, missing-data rules, and failure disposition must
  be frozen before outcomes. Enacted plans are comparators, not truth labels.
- SCALE: national summaries must separate State-weighted and
  district-weighted estimands. Districts and blocks are spatially dependent;
  naive independent-observation p-values are prohibited.
- BOUNDARY: operational checks and descriptive metrics cannot be promoted to
  VRA, fairness, or legal-validity findings.
- BENCHMARK: V1 needs one accepted result, one structured failure, source
  hashes, exact regeneration, and a test that would catch label-matching drift.
- SURVEY: outputs need a compact table, machine-readable JSON, and explicit
  reasons for every unavailable metric.

## Complexity Removed

- No national geometry computation before a perimeter protocol exists.
- No election overlay before the evaluation schedule's crosswalk requirements
  are met.
- No new generic comparison framework; V1 is a narrow research analyzer and
  verifier following the existing NRS package pattern.
- No winner score or composite index.

## Decision

Proceed with the Rhode Island 2020 Tier 1 slice. National expansion remains
gated on accepted/failure proof and exact analyzer/verifier agreement.

## Source-Identity Disposition

The first comparator candidate was rejected because its internal congressional
session was 116 despite a filename containing `cd118`. The accepted slice uses
the official Census Rhode Island CD118 archive and enforces the session field
before projection. This is an input-integrity correction, not an
outcome-selected comparator change.

## Post-Execution Disposition

The first national run preserved failures in Connecticut, Illinois, and New
Hampshire. Their official archives contain water-only `ZZ` polygons identified
as congressional districts not defined. The protocol was amended uniformly to
retain only blocks with `ALAND20 > 0`, exclude non-numbered comparator
polygons, and keep any unassigned retained block as a hard failure.

The amended run passed 50 States and 435 districts. The national verifier then
regenerated every State package exactly. Tier 1 is accepted; geometry,
elections, demographics, sensitivity, ensembles, and non-enacted national
comparators remain gated.
