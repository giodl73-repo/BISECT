---
wave: k5-length-width-uplift
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-k5-length-width-uplift-goal.md
---

# K.5 Length-Width Uplift

## Mission

Resolve the K.5 minor-revision gap by making the implementation boundary
explicit and adding tested support for AABB-vs-MBR diagnostics.

## Claim Boundary

The production compactness output remains `length_width_ratio`, the
rotation-invariant MBR metric. AABB is diagnostic only and is not emitted by
`all_metrics()`.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - AABB diagnostic and paper boundary | DONE | Added helper/test, updated K.5 paper, rebuilt PDF, updated ledgers |
| 02 - Closeout | DONE | Archived wave after validation and commit |

## Validation Gate

```powershell
cargo fmt
cargo test -p bisect-analysis test_lw_mbr_rotation_invariant_aabb_orientation_dependent
git diff --check
```

## Closeout

Completed. K.5 now has a tested AABB diagnostic helper, an explicit production
boundary for MBR-based `length_width_ratio`, a rebuilt PDF, and updated public
ledgers.
