# Review 1 — George Karypis
**Paper**: F.2: NestSection at Scale — Spine-Compatible Bicameral Redistricting for All 50 States
**Round**: R1
**Score**: 3/4

## Summary

F.2 applies NestSection to all 49 bicameral state legislatures, classifying each by gcd(H,S) compatibility and reporting compactness penalties and partisan effects for the 42 compatible states. The paper is well-structured and the algorithmic content is clear. My focus is on the gcd calculations, the spine construction correctness, and the within-super-region partitioning approach.

## Strengths

The spine construction is clearly described and algorithmically sound. The key insight — that METIS g-way partitioning of the full state graph into g balanced super-regions, followed by independent (H/g)-way and (S/g)-way partitions within each super-region, guarantees nesting — is correct and the implementation via the --chamber nest flag is a natural extension of the existing CLI.

The gcd table (Table 1) is the most important empirical contribution. The values for the 2:1 states (WA 98:49, MN 134:67, AK 40:20, etc.) are all correctly computed: gcd(98,49)=49, gcd(134,67)=67, gcd(40,20)=20. The 3:1 states (WI 99:33, OH 99:33, TN 99:33) correctly give gcd=33. The Vermont 5:1 ratio (150:30, gcd=30) is correct. New Hampshire's unusual 400:24 ratio gives gcd(400,24)=8 — independently verified. Colorado's 65:35 gives gcd(65,35)=5 — correct (since 65=5×13, 35=5×7).

## Concerns

**C1 — Count inconsistency: 7 vs. 9 incompatible states.** Table 1's incompatible column (gcd=1) lists: Missouri, Oklahoma, Texas, Hawaii, Pennsylvania, Connecticut, Rhode Island, Maine, Delaware — that is 9 states with gcd=1. The classification section (Section 3.2) says "7 incompatible states." The abstract says "8 states have gcd(H,S) = 1." The footnote in Table 1 says "we recount compatible states at 40 after careful tally." This is a fundamental inconsistency in the paper's primary claim. The count must be reconciled. By my count from Table 1: 49 bicameral states minus 9 gcd=1 states = 40 compatible states, not 42.

**C2 — New Hampshire 50:3 spine characterisation.** Table 1 correctly shows NH as 400:24 with gcd=8, giving a spine of 8 super-regions each containing 50 house districts and 3 senate districts. The NestSection results table (Table 2) lists NH as "50:3 spine=8." This is accurate. However, the paper does not discuss the algorithmic challenge of creating a 50-district partition within each super-region of a state with only 698 block groups and k/n_bg = 0.57. Each spine super-region contains approximately 698/8 ≈ 87 block groups and must support 50 house districts — meaning approximately 1.74 block groups per district within each super-region. This is the same resolution problem as the full-state NH redistricting, compounded by the spine constraint. The paper should explicitly acknowledge that NH requires the sub-unit splitting procedure described in F.1 even within the spine architecture.

**C3 — Senate PP lower than house PP.** The paper correctly notes that senate PP < house PP in all 42 compatible states (senate PP = house PP - 0.028 on average). The explanation (interior boundaries inflate senate perimeter) is correct. However, the paper does not report what the independent (non-NestSection) senate PP would be. If independently drawn senate districts have PP = house PP - 0.005 (say), then the nesting constraint adds approximately -0.023 PP units, which is the actual compactness cost of nesting. Without this baseline, the reader cannot determine whether the observed senate-house PP gap is due to nesting or simply due to larger district size.

**C4 — Simultaneous balance constraint at small g.** Section 4.1 notes that states with small spine counts (g ≤ 4) and large per-spine district counts can have difficulty achieving simultaneous population balance for both chambers. The paper claims "in practice, all 42 compatible states succeed." This is an important positive result, but the mechanism is described as "METIS's balance tolerance is applied at the sub-region level, not the full-state level." This means the 0.5% tolerance is applied independently to the h and s partitions within each spine region. Whether the two constraints are simultaneously achievable depends on the population distribution within each spine region, which is not analysed.

## Recommendation

Revise and resubmit. C1 (state count inconsistency) is a fundamental error that must be corrected. The count of compatible and incompatible states should be reconciled and stated consistently across abstract, body, and table.
