# K.5 Length-Width Uplift Goal

**Opened:** 2026-05-13
**Status:** complete

## Goal

Lift K.5 from minor revision by aligning the paper with current implementation
behavior and adding a tested AABB diagnostic helper for the MBR-vs-AABB claim.

## Acceptance

- [x] Add an active wave and pulse context for K.5.
- [x] Add AABB diagnostic helper coverage.
- [x] Update K.5 paper and ledgers.
- [x] Rebuild K.5 PDF.
- [x] Validate and commit.

## Non-Goals

- Do not change the production `length_width_ratio` value in `all_metrics()`.
- Do not claim district-scale AABB discrepancy replay beyond the existing
  single-run table until real replay packages are archived.
