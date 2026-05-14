---
wave: k5-length-width-uplift
pulse: 01
status: in-progress
governing_roles:
  - MERIDIAN
  - LEDGER
---

# Pulse 01 - AABB Diagnostic and Paper Boundary

## Mission

Add a tested AABB diagnostic helper and update K.5 to accurately disclose the
current production compactness surface.

## Deliverables

- [x] Add `axis_aligned_length_width_ratio` as diagnostic helper.
- [x] Add rotation-invariance/AABB-orientation-dependence test.
- [x] Update K.5 paper text.
- [x] Rebuild K.5 PDF.
- [x] Update paper index and scorecard.

## Validation

```powershell
cargo fmt
cargo test -p bisect-analysis test_lw_mbr_rotation_invariant_aabb_orientation_dependent
git diff --check
```
