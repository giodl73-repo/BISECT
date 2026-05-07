# Review 5 — Leah Liang (Experimental Methods / Survey Design Lens)
**Paper**: C.6 — Public Perceptions of Algorithmic vs. Human-Drawn Congressional Districts: A Survey Experiment
**Round**: R1
**Score**: 3/4 (Accept with Minor Revisions)

## Summary

This is a carefully pre-registered survey experiment with a well-powered design, appropriate sampling strategy, and rigorous pre-treatment balance verification. The methodological contribution is solid, and the paper follows best practices in survey experiment design and reporting. My comments focus primarily on three methodological concerns: the adequacy of the design for detecting interaction effects (moderation analysis power), the design asymmetry between enacted and non-enacted map conditions, and several measurement concerns. None of these requires major redesign, but several require explicit acknowledgment and, where feasible, sensitivity analyses.

## Strengths

The pre-registration framework is exemplary. The paper specifies primary outcome, analysis plan, exclusion criteria, stopping rules, and subgroup analyses in advance, and reports pre-registered deviations inline. This transparency substantially increases the credibility of the findings. The pre-registration of the cross-platform divergence criterion (pooling if platform effect < 0.30 SD) is especially thoughtful.

The double-platform recruitment strategy (MTurk + Lucid) is appropriate for a paper making broad claims about public opinion. The Lucid quota sampling matches age, gender, education, and region distributions of the American adult population; this substantially increases the generalizability of results compared to MTurk-only designs that are common in this literature.

The pre-treatment balance table (Table 2) is properly executed: seven covariates across six conditions, Bonferroni-corrected, no significant imbalances. This confirms that randomization succeeded and that the experimental controls in the regression specifications are truly variance-reduction rather than bias-correction tools.

The manipulation check results are strong: 89.4% correct identification in information conditions. The follow-up analysis showing robustness to excluding failed-manipulation-check respondents is appropriate.

The exclusion rate analysis (14.6% overall, no significant cross-condition differences) is properly documented and confirms that attrition did not introduce systematic bias.

## Concerns

**The 2×3 design is not adequately powered for moderation effects, and this must be stated explicitly.** The a priori power analysis specified N=394 per cell (400 implemented) for a minimum detectable effect of d=0.20 on the primary outcome at 80% power. This provides adequate power for the main effects and for the map-type × information-condition interaction. However, the paper reports multiple moderation analyses by partisanship (five partisan subgroups), in-party/out-party status, education, and political knowledge. These moderation tests are under-powered by design. A cell of N=400 contains approximately 31-37% Democrats or Republicans, yielding partisan subgroup Ns of approximately 124-148 per cell. With these Ns, the minimum detectable effect for a partisan moderation comparison within a cell is approximately 0.36-0.40 SD at 80% power — substantially larger than the 0.18 SD effect the paper reports (p=0.04). The paper should explicitly acknowledge that the moderation analyses were not adequately powered for detection of effects of the magnitude observed, and should report these as exploratory or preliminary. Pre-registration as a moderator analysis does not substitute for pre-registration of appropriate power calculations for each analysis tier.

**The design asymmetry for enacted maps is a structural limitation that requires more transparent acknowledgment.** The enacted map condition appears only in the no-information cell, serving as the reference category for the main effects (Table 2). The paper explains this choice — enacted maps with process information would direct attention to provenance in confounding ways — but does not fully acknowledge the consequence: the transparency treatment effect is estimated only relative to algorithmic and commission maps, not relative to enacted maps. This means H2 as stated ("process description will increase perceived fairness of algorithmic maps, but not enacted maps") cannot be directly tested. The paper's text at the beginning of the Results section (Section 5.2) implicitly acknowledges this when it reports that the main effect of the process description treatment on enacted maps is β=0.08 (p=0.17) — but this estimate is based on the enacted-no-information cell only, not on an enacted-with-information condition. The paper should clarify what this coefficient actually estimates.

**The survey procedure contains a potential demand artifact in Stage 4.** The paper describes Stage 4 as involving "a brief debriefing explaining the study's purpose and revealing which maps were algorithmically drawn, commission-drawn, or enacted." This debriefing occurs after outcome measurement, which is correct. However, Stage 4 also includes "attention and manipulation checks" and an "open-ended question" administered alongside the debriefing. If the manipulation check asking "were the maps you saw drawn by a computer program, a commission, or state legislators?" is administered before the debriefing, respondents in the no-information condition may infer from the question that algorithmic and commission map types exist (as the question options reveal the experimental conditions). This could affect post-survey attitude measures. The paper should clarify the sequencing of Stage 4 items.

**The minimum display time of 30 seconds is low.** The paper imposes a 30-second minimum display time per map to "discourage inattentive rushing." With two maps per respondent, this provides at most 60 seconds of map exposure, which may be insufficient for careful evaluation of congressional district maps that require geographic understanding. The paper does not report actual viewing time distributions, which would allow assessment of whether this minimum was binding and whether viewing time correlates with outcome ratings. If viewing times cluster at the minimum, inattention may be higher than the attention-check exclusions capture.

**The five-point primary scale and standardization procedure are not fully documented.** The paper reports that outcomes are standardized "within the full analytic sample (mean = 0, SD = 1)." The full analytic sample includes N=2,400 respondents who each rate two maps (N=4,800 respondent-map observations). It is not clear whether standardization was performed at the respondent level (using each respondent's two-map mean) or at the respondent-map level (pooling all 4,800 observations). These procedures yield different results when respondents vary systematically in their rating levels. The paper should specify the standardization procedure exactly.

## Minor Issues

The Likert items include "The people who drew these districts treated different communities equally." In conditions where respondents know the map was drawn algorithmically, there are no "people" who drew the districts. This item may produce strange response patterns in the algorithmic condition (who are the "people" being evaluated?). The item should be flagged as potentially problematic for within-scale validity in the algorithmic condition.

The paper does not report whether the outcome scale was administered once per map (immediately after each map) or once for both maps at the end of stimulus exposure. The design description ("Stage 3: outcome questions for each map") suggests per-map administration, but this should be stated explicitly.

## Recommendation

Accept with minor revisions. The primary revisions required are: (1) explicit statement that moderation analyses were not powered for detection of 0.18 SD effects, with reframing as exploratory; (2) clarification of the enacted-map information condition asymmetry and its implication for H2 testing; (3) clarification of Stage 4 sequencing; (4) reporting of actual viewing time distributions; (5) exact specification of the standardization procedure.
