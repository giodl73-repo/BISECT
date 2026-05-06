# Review 3 — Reviewer: Moon Duchin (Metric Geometry / GerryChain)
**Paper:** B.17 — How Sensitive Are Redistricting Outcomes to Algorithm Parameters? A 50-State Sweep
**Round:** 2
**Score:** 3/4

## Summary

My Round 1 score of 2/4 was driven primarily by the absence of the adversarial joint run — the paper claimed an empirical bound but had not actually run the adversarial combination. The revised paper addresses this directly: Section 4.6 reports the joint `ufactor × acounty` factorial sweep, and Section 5.1 now references the empirical combined sweeps that yield ΔD ≤ 0.3. I am upgrading to 3/4. The paper still does not fully answer every concern I raised, but the critical gap is closed.

## Addressed Issues

The joint factorial sweep (Table 4) is the revision I required. Running the 12-cell factorial for the two most adversarially relevant states (Wisconsin and North Carolina) and finding that the worst-case cell produces ΔD = −0.4 (WI) directly tests the additive bound. The result is that the joint effect does not amplify beyond the additive calculation: the worst-case joint cell is approximately consistent with the single-parameter sum. This is the empirical test the paper needed, and it is now present.

The B.16 citation issue is resolved in the references. The missing multiple-testing correction for the binomial tests is addressed: the paper now reports Holm-corrected p-values for all 20 non-baseline comparisons, and all are non-significant.

The acknowledgment that the studied parameter ranges should themselves be justified (not just treated as the ranges studied) is partially addressed in Section 6.2 with the note that ranges exceeding these bounds would require DIA amendment — which is a procedural argument for why the ranges are the appropriate scope.

## Remaining Concerns

The joint sweep is the most important fix, but it is not the full adversarial experiment I requested. I specifically asked for the 5-parameter adversarial setting (all five parameters at their most Republican-favorable values simultaneously: ufactor=5%, acounty=10, T=1000, aswing=1.20, wvra=0.6). The paper has tested the ufactor × acounty interaction but has not run the full 5-parameter adversarial combination. Section 5.1 still says the joint effect is ≤0.3 "empirically" — but the empirical basis is the 2-parameter factorial, not a 5-parameter adversarial run.

This is a residual concern but not a blocking one: the additive bound with the 2-parameter empirical confirmation is substantially more rigorous than the original additive argument alone. Running the full 5-parameter adversarial combination remains the most complete test, and I would recommend the authors do so for the final published version, but I no longer consider it mandatory for acceptance.

The multiple-testing correction (20 non-baseline comparisons) is now addressed, but the paper uses Holm correction applied to 20 tests treating each parameter × grid-point as an independent test. This is slightly conservative (the tests within a parameter are correlated), but conservatism is appropriate here. I accept this approach.

## Minor Issues

- The range-endpoint justification in Section 6.2 is brief. A more careful argument would distinguish between "these are the ranges studied" and "these are the appropriate statutory boundaries." The paper makes a procedural argument but not an empirical one.
- The "60x smaller" calculation in Section 5.2 now provides the range (40x to 80x) that I requested, which addresses my Round 1 minor concern.

## Recommendation

Accept. The adversarial joint run remains incomplete (only 2 of 5 parameters tested jointly), but the critical gap from Round 1 — the absence of any empirical joint test — is closed. The binomial test reporting and multiple-testing correction are now adequate. I raise my score from 2/4 to 3/4.
