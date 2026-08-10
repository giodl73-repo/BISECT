---
pulse: 07
title: NRS v0.3 national bakeoff
status: active
wave: nrs-2010-national-baseline
validation_level: L2 governed empirical evidence
---

# Pulse 07 - NRS v0.3 National Bakeoff

## Purpose

Place the governed NRS v0.3 assignments relative to enacted and alternative
plans without changing the generator or promoting incomplete metric families.

## Deliverables

- [x] Precommitted tiered comparison protocol.
- [x] Internal and external comparison matrix.
- [x] Role-review findings.
- [x] Rhode Island 2020 accepted evidence slice.
- [x] Structured failure fixture.
- [x] Independent analyzer/verifier agreement.
- [x] National Tier 1 execution plan.
- [x] National Tier 1 execution across 50 States.
- [x] Exact regeneration of all 50 State packages.
- [x] Rhode Island Tier 2 geometry proof slice.
- [x] National Tier 2 geometry execution across 50 States.
- [x] Exact regeneration of the national Tier 2 geometry package.
- [x] Paper-facing matrix and U.21/A.0/A.5 integration.
- [x] Legacy B.0 and K.0 evidence-boundary audit.

## Slice Result

The amended Rhode Island slice retained 24,831 land-containing blocks from
25,649 source blocks, excluding 818 water-only blocks uniformly. After
maximum-overlap label matching, 17,365 blocks matched and 7,466 differed. NRS
split 3 counties and 50 tracts; the enacted comparator split 1 county and 11
tracts. These are descriptive counts, not a superiority or legal finding.

The accepted package is
`docs/experiments/nrs-v0.3-bakeoff-ri-2020/`. The structured negative fixture
rejects a district-count mismatch before metric computation. A separately
preserved deviation records rejection of a filename-labeled CD118 parquet whose
internal session identifier was CD116.

## National Tier 1 Result

The national package passed all 50 States and all 435 districts. From
8,126,956 source blocks, the frozen `ALAND20 > 0` rule excluded 237,762
water-only blocks and retained 7,889,194 land-containing blocks. State-level
maximum-overlap matching found 4,194,107 matching assignments and 3,695,087
different assignments, a 53.162680% block-weighted agreement rate.

NRS split 1,808 county units and 19,789 tract units; the official CD118
comparators split 404 county units and 4,720 tract units. These counts describe
the two plan families under the frozen atomic universe; they do not establish
superiority, compactness, population balance, partisanship, or legal validity.
The national verifier re-hashed inputs and regenerated every State package.

## National Tier 2 Geometry Result

The sequential geometry package passed 50 States, 435 districts per plan
family, and 7,889,194 retained blocks with zero failures. Under the common
block-projected geometry contract, both State-weighted and district-weighted
means place the comparator above NRS on Polsby--Popper, exact Reock, and
convex-hull ratio and below NRS on Schwartzberg.

These are aligned descriptive compactness summaries, not measurements of
original enacted linework and not a superiority, fairness, or legal finding.

## Governing Boundary

Tier 1 covers common-unit assignment and county/tract splits. Tier 2 covers
common-block geometry. Elections, demographics, ensembles, and non-enacted
national comparators remain separate gates. No output may use winner,
fairness, neutrality, or legal-compliance framing.

## Validation

```powershell
pytest tests/unit/test_nrs_bakeoff_slice.py tests/unit/test_nrs_bakeoff_national.py -q
python scripts/research/verify_nrs_bakeoff_slice.py `
  docs/experiments/nrs-v0.3-bakeoff-ri-2020
python scripts/research/verify_nrs_bakeoff_national.py `
  docs/experiments/nrs-v0.3-national-bakeoff-2020
git --no-pager diff --check
```
