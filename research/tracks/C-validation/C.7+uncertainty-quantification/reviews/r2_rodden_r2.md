---
reviewer: Jonathan Rodden
round: 2
score: 3
date: 2026-05-05
---

## Summary

The revisions address the abstract CI inconsistency (my P1.3 via Karypis and Liang) and add a useful legal-relevance mapping in Section 7. The new section distinguishing how courts should treat each uncertainty source in the context of specific legal challenges is the most practically valuable addition in this revision for my purposes. However, my three Round 1 P1 items — the underestimation of electoral uncertainty, the missing enacted plan CIs in Table 5.5, and the unacknowledged implications of the MMD-zero boundary — remain unaddressed. The legal-relevance mapping is a partial response to the "which CI for which challenge" framing I raised, but it does not substitute for the sensitivity analysis on electoral uncertainty or the enacted-plan CIs that would make the partisan metrics section fully robust under adversarial examination.

## Evaluation of Revisions

**P1.1 (Electoral uncertainty underestimated)**: Not addressed. Section 5.2 still uses the cross-election variance from three elections (2016, 2018, 2020) to calibrate electoral uncertainty. The request was for a sensitivity table showing how joint EG CIs change if electoral uncertainty is doubled or tripled. This has not been added. For a paper that will be cited in redistricting litigation — where opposing counsel will note that three elections from a single redistricting cycle do not characterise full decadal electoral uncertainty — this is a significant omission.

**P1.2 (Missing enacted EG CIs)**: Not addressed. Table 5.5 still reports algorithmic EG with bootstrap CIs and enacted EG as point estimates. Without CIs for enacted EG, the comparison between algorithmic and enacted plans is asymmetric in its uncertainty reporting. An opponent can argue that the enacted plans' efficiency gaps may vary substantially across elections (which is true) while the paper treats them as fixed.

**P1.3 (MMD-zero boundary implications)**: Not addressed. The MMD CI [+0.0, +1.6] pp is still reported without discussing its implication that the packing effect measured by MMD is not statistically significant under electoral uncertainty.

**Abstract CI fix (Karypis/Liang P1)**: Resolved. The abstract now correctly states [+15%, +29%] for the joint all-three-sources CI.

**Bootstrap exchangeability argument (Karypis P1)**: Resolved. The new paragraph is clear and appropriately argued.

**Section 4.4 arithmetic error (Duchin P1)**: Resolved. "The lower bound (0.438) is below the tract-level point estimate (0.441) but above the enacted plan mean PP (0.295)" is factually correct and clearly states what the finding means.

**Legal-relevance mapping (Stephanopoulos P1 and my framing)**: Added in Section 7. The three-item mapping (seed → determinism challenges; census → Wesberry compliance; resolution → compactness measurement) is well-organized and legally accurate. This is a genuine improvement.

## Remaining Concerns

My three P1 items from Round 1 remain unresolved. Of these, the most important for litigation purposes is the electoral uncertainty sensitivity analysis: without it, a sophisticated opponent can argue that the EG CI widths are underestimates by a factor of 2-3. The enacted EG CIs are also important for symmetric uncertainty reporting. I cannot raise my score without at least one of these two items being addressed.

## Score: 3 — Minor Revision

Good progress on the cross-paper CI consistency and the legal-relevance framing. The core partisan metrics section issues (electoral uncertainty quantification and enacted plan CIs) remain. Either a sensitivity table for doubled/tripled electoral uncertainty or enacted EG cross-election variance estimates would bring this to acceptance.
