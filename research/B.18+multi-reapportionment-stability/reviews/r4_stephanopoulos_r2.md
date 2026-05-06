# Review 4 — Reviewer: Nicholas Stephanopoulos (Election Law / Partisan Gerrymandering)
**Paper:** B.18 — Redistricting Stability Across Reapportionment Cycles
**Round:** 2
**Score:** 3/4

## Summary

The revision addresses my two primary legal concerns: the seat projection sensitivity analysis is now present, and *Karcher v. Daggett* is now cited in the constitutional analysis. I maintain my score at 3/4. The paper is now legally sound for its core claims, and the remaining concerns are about depth and completeness rather than accuracy.

## Addressed Issues

The seat projection sensitivity analysis is now present for Florida (the most important case). Section 3.3 explicitly addresses 28→29 (prime, flat tree) vs. 28→30 (composite, 2×3×5, three-level tree) and notes that the structural characterization changes dramatically between these two projections. The paper correctly observes that 30=2×3×5 would be preferable from a compactness standpoint because the hierarchical tree can preserve county boundaries better than the 29-prime flat tree. This is exactly the analysis I requested, and it is well-framed.

*Karcher v. Daggett* (1983) is now cited in Section 6.2 with the appropriate framing: the fresh-recomputation design satisfies *Karcher*'s good-faith equal-population requirement by treating the decennial census as the trigger for a full recomputation, not as the trigger for minimal adjustment of an existing map.

The note on "stability as byproduct not target" (Section 6.3) is now more explicitly developed: the paper argues that by not targeting stability, the DIA cannot be accused of prioritizing stability over population equality. This is a legally important distinction and it is now clearly stated.

## Remaining Concerns

The ±1 sensitivity analysis for California (52→50 vs. 52→51) is present but brief. The paper notes that both 50=2×5² and 51=3×17 produce depth-3 trees, so the structural characterization does not change dramatically between these projections. This is a correct observation, but the Hamming distance sensitivity for California at ±1 seat (would d_Ham change significantly between 50 and 51 seats?) is not analyzed. Given that California is the largest state, this is a relevant question even if the structural characterization is similar.

The incumbent protection analysis remains too brief for legal purposes. The claim that "at most 23% of incumbents could be moved to a different district" is an upper bound, but redistricting litigation frequently involves specific incumbent protection claims. The paper should either (a) give a lower bound based on how often incumbents reside in tracts that would be in the same district under both tree structures, or (b) acknowledge that the incumbent protection analysis is insufficient for litigation purposes and that the full analysis would require identifying the incumbents' home tracts.

The path dependency concern about transition-minimizing objectives (Section 7.2) now includes the note I requested about gaming the transition objective. The paper correctly characterizes the tension between transition stability and political neutrality.

## Minor Issues

- The Texas apportionment lobbying note (Section 5 or 6) — the observation that knowing the factorization in advance could motivate lobbying for k=40 — is an interesting political observation but should be caveated: the DIA presumably specifies that the seat count is determined by Huntington-Hill and cannot be lobbied, making factorization-motivated lobbying available only for at-the-margin seat count disputes.

## Recommendation

Accept. The seat projection sensitivity (Florida case) and the *Karcher* citation are genuine legal improvements. The paper is now at an appropriate level of legal rigor for a technical redistricting paper.
