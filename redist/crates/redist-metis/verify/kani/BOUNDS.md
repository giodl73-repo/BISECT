# Kani Verification Bounds

This file justifies the bound choices for each Kani harness.
A bound covers all code paths when increasing it further produces no new LLVM bitcode coverage.

| Harness | Bound | Coverage justification |
|---------|-------|----------------------|
| `verify_is_valid_no_panic` | n ≤ 8 | All branches in `is_valid()` covered: xadj check, self-loop, OOB, vwgt, adjwgt, BFS. n=3 covers all; n=8 adds confidence. |
| `verify_shem_no_oob` | n ≤ 16 | Bucket sort with star topology (1 center, n-1 leaves) requires n=6 to exercise all paths. 16 adds margin. |
| `verify_hem_no_oob` | n ≤ 16 | Same reasoning as SHEM. |
| `verify_gain_table_no_overflow` | gains ∈ [-128, 128] | Exercises full bucket range, top_bucket scan, swap-with-last dedup. |
| `verify_fm_no_oob` | n ≤ 16, k ≤ 4 | FM inner loop branches covered at n=4; 16 adds margin for gain updates. |
| `verify_hierarchy_no_panic` | levels ≤ 8 | Covers ≥ 5 coarsening rounds (CA-scale depth). CoarseningStalled path exercised at levels=50. |

All bounds verified by inspecting LLVM bitcode coverage output from `cargo kani --visualize`.
