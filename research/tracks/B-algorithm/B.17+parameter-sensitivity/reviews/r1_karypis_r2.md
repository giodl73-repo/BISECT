# Review 1 — Reviewer: George Karypis (METIS / Graph Partitioning)
**Paper:** B.17 — How Sensitive Are Redistricting Outcomes to Algorithm Parameters? A 50-State Sweep
**Round:** 2
**Score:** 3/4

## Summary

The authors have addressed all of the issues I raised in Round 1, and the paper is now considerably stronger. The addition of the joint `ufactor × acounty` factorial sweep (Table 4 of the revised paper) is the most important improvement: it directly tests the most plausible parameter interaction and finds that the worst-case joint effect (0.4 seats in Wisconsin) does not amplify beyond the single-parameter additive bound. The revision also clarifies seed handling and reports the binomial test results explicitly. My residual concerns are minor.

## Addressed Issues

The joint factorial sweep (Section 4.6) directly answers my Round 1 concern about the `ufactor × acounty` interaction. The 3×4 design (three `ufactor` levels × four `acounty` levels) for Wisconsin and North Carolina is the right scope — these two states are the most contested, and the 12-cell factorial is computationally achievable. The finding that the worst-case cell (ufactor=2.0%, acounty=10.0) produces ΔD = −0.4 (WI) and −0.3 (NC) is consistent with the additive bound and does not reveal a super-additive interaction. This is the result the paper needed.

The `aswing` normalization clarification in Section 2.4 (confirming the B.9 normalization, with the cross-reference) resolves my terminological concern from Round 1.

The binomial test results are now reported in Section 3.3: all five parameters produce non-significant binomial tests (uncorrected p > 0.15 for all; Holm-corrected p > 0.50 for all). This is the right result and should be prominently mentioned in the abstract.

## Remaining Concerns

The ConvergenceSweep within-T seed variance (my Round 1 concern about the Georgia case) is now partially addressed: the revised methodology section explains that each pipeline run uses the ConvergenceSweep best-of-T seed, which is not a single arbitrary seed. However, the paper still does not report the seed variance at T=200 for Georgia specifically. The table shows that T≥200 produces "identical outcomes across all 50 states" in the mean, but the seed variance at T=200 vs. T=600 for Georgia is not quantified. This is a minor point — the methodology clarification is sufficient — but a footnote reporting the Georgia T=200 variance would make the claim airtight.

The B.16 citation issue from Round 1 (B.16 appearing in the text but missing from the reference list) appears to have been partially resolved — B.16 is now in the references, but the specific claim cited to it (the Georgia tail analysis with T=100) still uses a parenthetical reference rather than the numbered citation. This is a formatting issue, not a substantive one.

## Minor Issues

- The joint sweep (Table 4) uses 3 `ufactor` levels rather than the 4 I suggested; the rationale for dropping the 5.0% level should be noted in the text.
- The population neutrality argument (Section 5.3) remains asserted rather than empirically demonstrated. A correlation table (partisan lean vs. tract population across all 50 states) would convert this from an argument to evidence. That said, Rodden's state-level analysis (now included) provides indirect support.

## Recommendation

Accept with minor revisions. The joint sweep and binomial test additions directly address the paper's prior core weakness. The remaining items are formatting and annotation issues that can be resolved in copyediting.
