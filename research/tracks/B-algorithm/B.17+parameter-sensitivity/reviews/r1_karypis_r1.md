# Review 1 — Reviewer: George Karypis (METIS / Graph Partitioning)
**Paper:** B.17 — How Sensitive Are Redistricting Outcomes to Algorithm Parameters? A 50-State Sweep
**Round:** 1
**Score:** 3/4

## Summary

This paper addresses a practically important question for the Districting Integrity Act framework: can parameter tuning within a fixed algorithm produce meaningful partisan effects? The central finding — that partisan outcomes vary by at most 0.3 seats nationally across the full parameter range — is well-motivated and well-presented. The paper makes effective use of the B.0 bakeoff as a comparative scale reference, and the separation of compactness sensitivity from partisan sensitivity is a genuine insight.

## Strengths

The OAT sweep design is appropriate and clearly documented. The parameter grid (5 levels per parameter, 25 non-baseline runs) is sufficient to establish monotonicity or lack thereof for each parameter independently. The decision to report both the absolute metric range and the standardized sensitivity index (normalized by B.7 seed variance) is methodologically sound — it contextualizes the effect size against known algorithmic noise rather than against an arbitrary absolute scale.

The compactness finding for `ufactor` is the most interesting result in the paper. The Polsby-Popper variation of ±0.010 across the ufactor range (0.1% to 5%) is a meaningful 3.2% relative change and has clear policy implications for DIA drafting. The recommendation to fix `ufactor = 1%` as the statutory default is well-supported.

The explanation for why partisanship is parameter-insensitive is technically sound. Population neutrality and geographic continuity arguments in Section 5.3 correctly identify that METIS parameters affect cut quality within a fixed tree structure, not the tree structure itself.

## Weaknesses and Concerns

The OAT design is acknowledged to miss interaction effects, but the paper's treatment of this limitation is too casual. The additive bound calculation in Section 5.1 produces a maximum combined effect of 0.5 seats assuming additivity, but parameter interactions in METIS graph partitioning are non-trivial. Specifically, high `ufactor` combined with high `acounty` could have compounding effects on how county boundaries interact with population balance tolerance. A 2x2 factorial for just `ufactor` x `acounty` (4 additional runs) would address the most plausible interaction and should be added before publication.

The ConvergenceSweep threshold results deserve more scrutiny. The claim that "outcomes are identical across all 50 states" for T >= 200 is stronger than the evidence warrants. Table 3 shows identical mean outcomes, but the seed-level variance within each T setting is not reported. A T=200 run might produce the same *mean* D_nat as T=600 while having higher variance across seeds. The paper should report within-T seed variance for at least the Georgia case (known to have a late-converging seed).

The `aswing` parameter description conflates two different things. In GeoSection/AreaSection, the area swing parameter controls the ratio of geographic area to population in the bisection objective — but the paper's Table 1 lists it as ranging from 1.05 to 1.20, which matches the B.9 normalization convention. The paper should clarify whether it is using the same normalization as B.9 or a different one, and the effect on compactness (PP varies from 0.311 to 0.307 across the range) should be explained in terms of the area-weight mechanism, not just reported.

## Minor Issues

- The T=100 result ("slightly higher D_nat of 0.1 seats because it occasionally misses the Georgia global minimum") is interesting and should be supported with a reference to the Georgia-specific B.16 tail analysis rather than just cited parenthetically.
- The binomial test mentioned in Section 3.3 is never reported in the results section. Were the tests run? What were the results? If they were all non-significant, say so explicitly.
- Section 6 recommends `wvra = 0.4` as the DIA default with the justification "consistent with B.14." B.14 should be cited with a more specific statement of what it found.

## Recommendation

Accept with minor revisions. The core finding is robust and the policy implications for DIA statutory drafting are valuable. Add the 2x2 `ufactor` x `acounty` interaction check, report within-T seed variance for the Georgia case, and report the binomial test results explicitly.
