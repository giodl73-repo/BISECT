# Length-Width Ratio in Algorithmic Redistricting

**Series**: K.5
**Status**: Accepted 3.5/4
**Target**: Political Analysis

## Algorithm / Subject

Empirical study of the Length-Width Ratio (LW = maximum diameter / minimum diameter of the minimum bounding rectangle) as a compactness metric in redistricting. Covers the formula and the distinction between the minimum bounding rectangle (MBR, orientation-optimised) and the axis-aligned bounding box (AABB, computationally simpler but direction-dependent). Documents the LW $> 3$ threshold that courts have applied to "elongated" district challenges, the computational procedure for the MBR using rotating calipers, and LW values for bisect structure algorithms across NC, WI, and TX.

## Key Claims

1. LW $> 3$ is a documented court threshold for flagging "elongated" districts in redistricting challenges: districts with LW $> 3$ have been successfully challenged in litigation in Illinois (7th Circuit) and New York as visually non-compact under state constitutional standards.
2. All four bisect structure algorithms produce LW $\leq 2.8$ on NC 2020 congressional districts, confirming that bisect's recursive bisection does not generate elongated districts and that bisect plans satisfy the LW $< 3$ threshold for all NC districts.
3. LW is direction-dependent and must be computed using the minimum bounding rectangle (rotating calipers, O($n \log n$)) rather than the axis-aligned bounding box (AABB): AABB systematically overestimates LW by up to 41% for diagonally oriented districts, making AABB-based LW legally unreliable.

## Layer

Standalone

## Empirical Targets

- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), 2020 census
- Compare across: standard-bisect, prime-factor, ratio-optimal, moving-knife structure algorithms
- Metrics: district-level LW distribution (min, median, mean, max), state-level max LW (the worst-case district), AABB vs MBR discrepancy quantification, proportion of districts with LW $> 3$

## Test Invariants

- L0: LW $\geq 1$ for any polygon (equality only for a square); LW of a square is 1.0 (exact); LW of a $1 \times 3$ rectangle is 3.0 (exact, for MBR); MBR area $\leq$ AABB area for any polygon (MBR is always at least as tight as AABB); LW is rotation-invariant when computed via MBR
- L1: on a $1 \times 4$ rectangular district (horizontal), MBR gives LW = 4.0 and AABB gives LW = 4.0 (aligned case — same); on the same rectangle rotated 45°, MBR still gives LW = 4.0 but AABB gives LW = $4\cos(45°) + 4\sin(45°)$ ... width = $4\sqrt{2}/2 + 4\sqrt{2}/2 = 4\sqrt{2}$, height = same — the discrepancy test; verify MBR LW $\neq$ AABB LW for the rotated case
- L2: NC 2020 bisect (all structure algorithms) produces maximum district LW $\leq 2.8$; at least one NC enacted map district has LW $> 2.8$ (comparison showing bisect outperforms the enacted map on this metric)

## Legal / Practitioner Value

Length-Width Ratio is used in redistricting litigation when courts apply a visual test for elongation. The LW $> 3$ threshold — which has appeared in expert reports in Illinois redistricting cases and New York state court filings — provides an objective operationalisation of a district that is "three times as long as it is wide." LW is more intuitive than PP for elongation detection because courts can visualise a 3:1 aspect ratio easily. The rotating-calipers MBR computation documented in this paper is required to make LW legally credible: axis-aligned bounding boxes depend on map orientation and are rejected by courts as arbitrary. This paper provides certified MBR-based LW values for bisect plans and documents the methodology that makes them defensible in expert testimony.

## Section Structure

§1 Introduction, §2 Mathematical Definition (LW Formula: MBR vs AABB), §3 Rotating Calipers Algorithm, §4 The LW $> 3$ Court Threshold — Case Survey, §5 Empirical Comparison Across Structure Algorithms, §6 AABB vs MBR Discrepancy Quantification, §7 Practitioner Usage Guide, §8 Conclusion
