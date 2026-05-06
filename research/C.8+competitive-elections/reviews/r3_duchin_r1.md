# Review 3 — Reviewer: Moon Duchin (Metric Geometry / Electoral Analysis)
**Paper:** C.8 — Do Algorithmic Districts Produce Competitive Elections? Evidence from All 50 States
**Round:** 1
**Score:** 3/4

## Summary

This is a solid empirical paper on a well-defined question. The methodology is appropriate, the findings are plausible, and the "byproduct not design target" framing is correct and important. My concerns are about the absence of ensemble uncertainty quantification and about whether the 85 vs. 65 comparison is a fair comparison given that enacted maps are drawn by different processes in different states.

## Strengths

The competitive district definition is well-justified. Using |D_i - 0.5| < 0.10 on the two-party presidential vote share is standard, and the footnote about the 5-point swing threshold providing a more conservative measure is useful. The acknowledgment of the presidential vote proxy's limitations (incumbency effects, single-election snapshot) shows appropriate methodological humility.

The durability analysis (Section 6) is one of the paper's strongest features. The finding that 61 of 85 competitive algorithmic districts (72%) remain competitive under every shift from -4 to +4 percentage points is more informative than a static count. It shows that competitive algorithmic districts are not marginal cases clustered just inside the 10-point threshold.

The engagement with the Brunell argument in Section 5.2 is well-done. I particularly appreciate the third argument (competitive districts are the mechanism for translating national preferences into seat changes), which is the most persuasive counter to Brunell.

## Weaknesses and Concerns

The paper's central claim — 85 competitive districts under algorithmic maps vs. 65 under enacted maps — is based on comparing a single algorithmic map (one seed) to a single enacted map (the 117th Congress map). The 85 figure has variance: different random seeds would produce different algorithmic maps with different competitive district counts. The paper cites "dellaLibera2026uncertainty" for an ensemble analysis but does not include its results here.

The minimum requirement before publication is to characterize the range of competitive district counts across the ensemble. If the ensemble produces competitive district counts ranging from 75 to 95, the comparison to 65 under enacted maps is still meaningful. If the range is 60 to 110, the comparison is much weaker — the specific seed chosen happens to produce an unusually high competitive district count.

The comparison to enacted maps is also a comparison across different redistricting processes. The 50 enacted maps represent 50 different redistricting processes (some drawn by commissions, some by courts, some by partisan legislatures, some by bipartisan legislatures). The California CRC map, the Arizona AIRC map, and the Ohio Republican-drawn map are all lumped together as "enacted." A more precise comparison would separate enacted maps by process type: partisan legislature, bipartisan legislature, commission, court-ordered. The paper's finding that algorithmic maps produce more competitive districts than "enacted" maps is less meaningful if many of the "enacted" maps are already drawn by independent commissions.

The claim that competitive algorithmic districts are not "artifacts of the vote-share definition" (Section 4.2) is supported by showing that 72 of 85 have margins under 8 points and 57 have margins under 5 points. This is correct but could be presented more rigorously: report the full distribution of margins within competitive algorithmic districts and compare to the distribution within competitive enacted districts.

## Minor Issues

- The partisan bias test in Section 5.3 (algorithmic maps allocate 50.6% of competitive seats to Democrats vs. 51.3% national Biden share) is a reasonable test, but comparing to the national presidential vote share is not quite right. The relevant comparison is the expected competitive-seat allocation under a symmetric map, which is not necessarily equal to the national presidential vote share.
- Section 7.3 ("States with Few or No Gains") notes that California under the CRC map shows "little difference from algorithmic output." This is an important validation — if California's commission-drawn map already looks like an algorithmic map, that is evidence that both are achieving similar geometric objectives. This point deserves more development.

## Recommendation

Accept with minor revisions. Report ensemble-based uncertainty bounds on the competitive district count. Stratify the enacted-map comparison by redistricting process type (partisan legislature vs. commission vs. court).
