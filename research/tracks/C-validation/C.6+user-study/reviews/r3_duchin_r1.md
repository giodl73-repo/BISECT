# Review 3 — Moon Duchin (Mathematics / Redistricting Metrics Lens)
**Paper**: C.6 — Public Perceptions of Algorithmic vs. Human-Drawn Congressional Districts: A Survey Experiment
**Round**: R1
**Score**: 3/4 (Accept with Minor Revisions)

## Summary

This paper reports a well-powered survey experiment on public perceptions of algorithmic redistricting. My review focuses on the quantitative claims, effect size interpretation, and the mathematical characterizations of the algorithm. Overall, the study is carefully designed and the statistical analysis appears sound, but several quantitative claims require clarification or more careful presentation. I am recommending acceptance with minor revisions contingent on addressing the mathematical and statistical issues below.

## Strengths

The effect sizes reported (0.41 SD main effect, 0.22 SD transparency treatment, 0.18 SD partisan gap) are plausible given the redistricting opinion literature, and the paper appropriately contextualizes them against prior work. The citation of Bonneau et al. (2021) and Elmendorf (2019) for the claim that "information treatments typically move opinion by 0.05–0.15 SD" provides appropriate benchmarking, and the paper's transparency treatment effect (0.22 SD) exceeds this benchmark — the paper is right to flag this as noteworthy.

The use of respondent-map as the unit of observation (N=4,800) with standard errors clustered on respondent is methodologically correct and appropriately documented. The distinction between the full analytic sample N and the respondent-map N is handled transparently in the regression tables.

The convergent validity of the primary outcome scale (Cronbach's α=0.87, CFI=0.96, RMSEA=0.06) is strong, and reporting these validation statistics from the pilot sample is good practice.

## Concerns

**Effect size plausibility requires more context.** The paper claims that 0.41 SD is "large by the standards of survey experiments on redistricting." This claim is supported by two citations, but the comparison class is not well-defined. What is the distribution of effect sizes across survey experiments on redistricting specifically? If the effect size literature on redistricting contains only a handful of studies, the claim of being "large by the standards of" this literature may be misleading. The paper should either provide a more systematic review of effect sizes in the comparison literature or soften this claim.

**The 0.18 SD partisan gap claim warrants equivalence framing.** The paper presents the 0.18 SD Democrat-Republican difference as "statistically significant but substantively modest." This is the right qualitative framing, but "substantively modest" is not mathematically defined. In many practical contexts, 0.18 SD is not trivially small. The paper should define a threshold for "substantively modest" in terms of policy relevance — for example, if the paper claims that partisan moderation is small enough not to be a barrier to reform, it should argue explicitly for a specific magnitude claim. An equivalence test (TOST) or a Bayesian analysis with a region of practical equivalence (ROPE) would make this claim rigorous.

**The claim that the algorithm is "structurally incapable of implementing partisan strategy because it cannot see partisan data" requires more precise language.** The paper uses this framing in the introduction. While directionally correct, it elides an important mathematical point: even an algorithm that does not access partisan data can produce partisan-correlated outputs, because geography and partisanship are correlated. The paper should distinguish between (a) the algorithm is incapable of partisan input and (b) the algorithm's outputs are uncorrelated with partisan outcomes. The latter is false, and the paper's own discussion section correctly acknowledges this when it notes that "the correlation reflects geography, not manipulation." The introduction should be aligned with this more precise framing from the start to avoid overclaiming.

**The R² values in Table 2 are low but not discussed.** The main regression models have R² of 0.071 (no controls) and 0.083 (with controls), indicating that map type and information condition together explain approximately 7-8% of variance in perceived fairness. This is consistent with a pre-registered experiment in which the outcome variable is heterogeneous across individuals. But it also means that most of the variance in perceived fairness is unaccounted for — raising the question of whether there are unmeasured individual differences that would interact with treatment effects. The paper should briefly note this and explain what the low R² implies (or does not imply) for the treatment effect estimates.

**The full-description-plus-comparison interaction (0.31 SD) is larger than the process-description interaction (0.22 SD), but the comparison condition structure is asymmetric.** In the full-description-plus-comparison condition, respondents see both the assigned map type and "a second-panel image of the alternative map type." This means subjects in the algorithmic-full-description condition see the algorithmic map alongside the enacted map. This direct comparison may activate contrast effects in which the enacted map's appearance makes the algorithmic map look better by juxtaposition, independent of any process information. The paper should discuss this confound and whether the 0.31 SD estimate is attributable to process learning or visual contrast.

**The five-item fairness scale conflates process and outcome fairness.** Items 1, 3, and 5 ("these district boundaries seem fair," "district shapes seem reasonable and not manipulated," "seem to reflect where people actually live") assess outcomes. Items 2 and 4 ("people who drew these districts treated communities equally," "I would trust election results from these districts") assess process legitimacy and outcome trust. The pre-validated CFA confirms unidimensionality, but the theoretical distinction matters: the paper's discussion draws heavily on procedural fairness theory, yet the scale conflates procedural and distributive fairness. If the paper wants to make strong claims about procedural fairness mechanisms, a subscale analysis (process items vs. outcome items) would be informative.

## Minor Issues

The pilot sample (n=200) CFA and reliability estimates are useful, but it's not stated whether the pilot used the same population or recruitment platform as the main study. If the pilot used MTurk and the main study pools MTurk and Lucid, the scale properties from the pilot may not fully transfer.

The paper reports that 71% of respondents "supported their state using algorithmic redistricting" at end of survey. This is a non-experimental measurement administered after treatment, which may reflect treatment contamination. This result should be flagged as post-treatment and interpreted cautiously.

## Recommendation

Accept with minor revisions. The paper makes a solid empirical contribution. The mathematical claims need tightening in the introduction, and the effect size interpretation requires more formal treatment. The subscale analysis and discussion of the comparison condition confound are desirable additions.
