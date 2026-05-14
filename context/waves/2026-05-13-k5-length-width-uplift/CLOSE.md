# K.5 Length-Width Uplift Closeout

**Status:** complete

## Outcome

K.5 is no longer a minor-revision outlier in the K compactness track. The paper
now matches the production implementation boundary: `length_width_ratio` is the
MBR-based compactness metric, while AABB is a diagnostic comparison helper.

## Delivered

- Added `axis_aligned_length_width_ratio()` to `bisect-analysis`.
- Added a focused test showing MBR LW is rotation-invariant while AABB LW is
  orientation-dependent on a rotated 3:1 rectangle.
- Updated K.5 paper sections on implementation, CLI usage, and conclusion.
- Rebuilt `docs/papers/K.5+length-width.pdf`.
- Updated `docs/PAPERS.md` and `docs/papers/ALGORITHM-PAPER-SCORECARD.md`.

## Validation

```powershell
cargo fmt
cargo test -p bisect-analysis test_lw_mbr_rotation_invariant_aabb_orientation_dependent
git diff --check
```

## Carry-Forwards

- Add real district-level LW/AABB replay packages before upgrading the
  single-run discrepancy table into a fully archived empirical package.
