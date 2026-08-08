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
- [ ] Rhode Island 2020 accepted evidence slice.
- [ ] Structured failure fixture.
- [ ] Independent analyzer/verifier agreement.
- [ ] National Tier 1 execution plan.

## Governing Boundary

Tier 1 covers common-unit assignment and county/tract splits. Geometry,
elections, demographics, sensitivity, and ensembles remain separate gates.
No output may use winner, fairness, neutrality, or legal-compliance framing.

## Validation

```powershell
pytest tests/unit/test_nrs_bakeoff_slice.py -q
python scripts/research/verify_nrs_bakeoff_slice.py `
  docs/experiments/nrs-v0.3-bakeoff-ri-2020
git --no-pager diff --check
```
