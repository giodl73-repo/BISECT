---
pulse: 01
title: Canonical national standard decision
status: done
wave: national-standard-evidence-and-specification
validation_level: L1 specification
---

# Pulse 01 - Canonical National Standard Decision

## Purpose

Freeze one proposed national-standard procedure before changing advocacy,
statutory, experimental, or implementation claims.

## Decisions Required

- Define the mandatory constitutional and data floor.
- Select the canonical structure, weight, search, resolution, seed, and
  tolerance rules.
- Decide whether the baseline is binding, presumptive, or evidentiary.
- Define how VRA, community, state-law, and topology modifications are made.
- Define the public diff, rationale, manifest, and review requirements.
- Record rejected alternatives, including binary versus prime-factor structure
  and geographic versus county-sticky weighting.

## Deliverables

- [x] Versioned national-standard specification under `docs/specs/`.
- [x] Decision matrix covering legal floor, baseline, modifications, and
      evaluation.
- [x] Conformance fixtures or test-vector requirements.
- [x] Explicit claim boundary separating procedural reproducibility from
      substantive fairness.
- [x] Panel review using MERIDIAN, BOUNDARY, DATUM, SCALE, COMMONS, COVENANT,
      and SURVEY.

## Validation

```powershell
git --no-pager diff --check
```

Result: passed 2026-07-09.

## Evidence

- `docs/specs/2026-07-09-national-redistricting-standard-v0.1.md`
- `context/waves/2026-07-09-national-standard-evidence-and-specification/panels/pulse-01-canonical-standard-review.md`

## Closure Rule

Closed as an internal L1 decision record. The specification selects one
canonical procedure and assigns every configurable choice an owner, rationale,
versioning rule, and challenge mechanism. Implementation and external
validation gaps remain explicitly nonconforming carry-forwards.
