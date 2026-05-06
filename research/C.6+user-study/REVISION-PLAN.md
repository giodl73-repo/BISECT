# REVISION-PLAN — C.6 User Study
**Paper**: Public Perceptions of Algorithmic vs. Human-Drawn Congressional Districts: A Survey Experiment
**Round 1 Review Date**: 2026-05-05
**Round 2 Review Date**: 2026-05-05
**Target**: Avg ≥ 2.5/4 | R1 Achieved: 3.0/4 | R2 Achieved: 4.0/4 (all five reviewers: 4/4 Accept)

---

## Round 1 Score Summary

| Reviewer | Lens | Score | Verdict |
|---|---|---|---|
| Karypis | Algorithmic framing / CS | 3/4 | Accept w/ minor revisions |
| Rodden | Political science / electoral geography | 3/4 | Accept w/ minor revisions |
| Duchin | Mathematics / effect sizes | 3/4 | Accept w/ minor revisions |
| Stephanopoulos | Election law / policy implications | 3/4 | Accept w/ minor revisions |
| Liang | Experimental methods / survey design | 3/4 | Accept w/ minor revisions |

**Panel Average**: 3.0 / 4.0 — Target met. All reviewers recommend acceptance with minor revisions.

---

## Round 2 Score Summary

| Reviewer | Lens | R2 Score | Verdict |
|---|---|---|---|
| Karypis | Algorithmic framing / CS | 4/4 | Accept |
| Rodden | Political science / electoral geography | 4/4 | Accept |
| Duchin | Mathematics / effect sizes | 4/4 | Accept |
| Stephanopoulos | Election law / policy implications | 4/4 | Accept |
| Liang | Experimental methods / survey design | 4/4 | Accept |

**Round 2 Panel Average**: 4.0 / 4.0 — Target (≥ 3.4/4) exceeded. All five reviewers accept unconditionally.

### P1 Issues Resolved (Round 2)

| Issue | Status |
|---|---|
| P1-A: Partisan moderation power adequacy (H3 exploratory reframe) | Resolved — H3 relabeled exploratory; power caveat in §5.3 |
| P1-B: H2 testability (enacted×information asymmetry) | Resolved — H2 rewritten as equivalence test |
| P1-C: Treatment text not reproduced | Resolved — Figure 2 box in §3 |

### Residual Notes (Non-blocking, for final revision)
- Karypis: Technical appendix for algorithm parameters (stimulus generation)
- Karypis: Likert item 2 validity flag in §4 (no "people" in algorithmic condition)
- Rodden: High-awareness sensitivity analysis (or stronger acknowledgment in §6.4)
- Rodden: Cross-platform effect size point estimates
- Duchin: $R^2$ note in §5.1
- Duchin: Introduction "structurally incapable" overclaim vs. discussion framing
- Stephanopoulos: Rucho characterization precision
- Stephanopoulos: Commission condition legal specificity footnote

---

## Cross-Reviewer Issue Synthesis

Issues appearing in 2+ reviews are prioritized. Issues from the lead reviewers (Rodden, Liang) are flagged.

### Priority 1 — Structural Issues (Multiple Reviewers)

**P1-A: Partisan moderation power adequacy** [Karypis, Liang — LEAD]
The 0.18 SD Democrat-Republican moderation effect (p=0.04) was not powered for detection at the reported effect size. With partisan subgroup Ns of ~124-148 per cell, the minimum detectable effect is approximately 0.36-0.40 SD at 80% power. The paper must either (a) reframe moderation findings as exploratory/preliminary with appropriate caveats, or (b) provide post-hoc power calculations showing that the pre-registration's subgroup specifications included adequate power assumptions. Cannot remain as presented.

**P1-B: Enacted-map asymmetry in the design and H2 testability** [Karypis, Liang — LEAD]
The enacted map appears only in the no-information condition. As a result, H2 as written ("process description will increase perceived fairness of algorithmic maps, but not enacted maps") cannot be directly tested — there is no enacted-with-information cell. The β=0.08 coefficient reported in Table 2 for "Process description (vs. none)" estimates the effect across all map types in the no-enacted-information baseline, not an enacted-specific information effect. The paper must either (a) revise H2 to match what can actually be tested, or (b) add a clarifying note specifying that H2 is tested by the non-significance of the enacted × information interaction (inferred, not observed). This is a hypothesis-testing integrity issue.

**P1-C: Treatment text not reproduced** [Karypis, Liang]
The 90-word plain-language process description administered to subjects in the information conditions is not reproduced in the paper. Reviewers cannot evaluate whether the treatment accurately represents the algorithm's properties, whether it is neutral or advocacy-leaning, or whether it would produce similar effects in other research contexts. Include the full treatment text in a clearly labeled appendix or in Section 3.

### Priority 2 — Quantitative Claims (Duchin Lead, Multiple)

**P2-A: "Structurally incapable of partisan strategy" overclaims** [Duchin, Stephanopoulos]
The introduction states the algorithm is "structurally incapable of implementing partisan strategy because it cannot see partisan data." This overclaims. The algorithm cannot receive partisan input, but its outputs are correlated with partisan outcomes due to geographic sorting. The discussion correctly acknowledges this ("the correlation reflects geography, not manipulation"), but the introduction is inconsistent with the discussion. Revise introduction to match the more precise framing: the algorithm cannot receive partisan input, and its partisan-correlated outputs reflect geography, not manipulation.

**P2-B: Effect size contextualization** [Duchin]
The claim that 0.41 SD is "large by the standards of survey experiments on redistricting" requires more systematic support. Provide a brief table or list of effect sizes from the specific cited studies (Bonneau 2021, Elmendorf 2019, Henderson 2018, Nalder 2019) to demonstrate that 0.41 SD exceeds the distribution of effects in the comparison literature. If those studies do not report comparable standardized effect sizes, soften the claim.

**P2-C: Partisan moderation equivalence framing** [Duchin, Rodden]
The 0.18 SD partisan gap is described as "substantively modest" without defining what "modest" means in practical terms. Add either (a) a TOST equivalence test with a defined region of practical equivalence, or (b) a sentence defining the policy threshold: "A gap of 0.18 SD indicates that even the group with the smallest effect (strong Republicans) rates algorithmic maps 0.33 SD fairer than enacted maps — a margin that exceeds the typical effect of most political information treatments in this domain." This makes "modest" mathematically grounded.

**P2-D: R² values not discussed** [Duchin]
The main regression models explain only 7-8% of variance. Add a brief note in Section 5.1 explaining that this is expected in a pre-registered experiment where the primary interest is in the treatment effect point estimate and confidence interval, not in R² as a model fit criterion, and that the low R² does not affect the unbiasedness of the treatment effect estimate.

**P2-E: Post-treatment support measure flagged** [Duchin, Stephanopoulos]
The end-of-survey question about support for algorithmic redistricting ("71% supported") is a post-treatment measure that may reflect treatment contamination. Flag it explicitly as a post-treatment, non-experimental measure in Section 5.4.

### Priority 3 — Political Science Substance (Rodden Lead)

**P3-A: Two-state sensitivity analysis for high-awareness respondents** [Rodden]
North Carolina and Maryland are high-salience gerrymandering cases. Respondents with prior awareness of these states' redistricting may have pre-existing negative opinions about the enacted maps, inflating the algorithmic advantage estimate. Provide a sensitivity analysis using Lucid respondents who report low prior awareness of North Carolina and Maryland redistricting (this information can be approximated from the baseline state-fairness rating or a state-knowledge check if available). If this analysis cannot be run, acknowledge the limitation more quantitatively.

**P3-B: Cross-platform effect size differences should be reported** [Rodden]
The paper reports that MTurk and Lucid did not diverge by more than 0.30 SD (the pre-registered threshold), but does not report the actual cross-platform differences. Report the point estimates and confidence intervals for the primary effects separately by platform, even if in supplementary material.

**P3-C: Commission condition legal specificity** [Stephanopoulos]
The "North Carolina Bipartisan Redistricting Committee (2022)" used as commission-drawn stimulus was a legislative committee, not a citizen commission. This should be accurately described. More importantly, the paper should clarify that the commission condition operationalizes "non-legislative process reform" rather than "independent citizen commission" specifically, and note that results for citizen commissions (California, Arizona) might differ.

**P3-D: Rucho characterization** [Stephanopoulos]
The introduction's statement that Rucho "removed the federal judiciary as a check on partisan gerrymandering" is slightly imprecise — Rucho removed the federal judiciary's remedy for federal constitutional claims but preserved state court remedies and explicitly invited congressional action. Revise to: "The Supreme Court's decision in Rucho v. Common Cause removed federal courts as a check on partisan gerrymandering under the federal Constitution, leaving state court remedies and federal legislation as the primary avenues for reform."

**P3-E: Voice-in-process concern** [Stephanopoulos]
The discussion correctly identifies that algorithms eliminate the "voice" component of procedural fairness. This concern should be more directly addressed: either (a) report whether the process description mentioned that the algorithm incorporated no public input (and if so, whether this was tested as a moderator), or (b) acknowledge that this is a gap in the study and recommend future research.

### Priority 4 — Experimental Methods (Liang Lead)

**P4-A: Stage 4 sequencing clarification** [Liang]
Clarify the sequencing of Stage 4 items. The manipulation check asking respondents to identify map type (options include "computer program / commission of citizens / state legislators") reveals the experimental conditions to respondents in the no-information condition before the debriefing. If this occurs before debriefing, it introduces demand effects. Specify the exact order: manipulation checks should be completed before debriefing, and if they are, acknowledge the potential for condition-revealing effect.

**P4-B: Actual viewing time distributions** [Liang]
Report actual viewing time distributions (mean, SD, and % at or near the 30-second minimum) for each condition. If viewing times cluster at the minimum (30 seconds), the minimum was binding and inattention may exceed what the attention-check exclusions capture.

**P4-C: Standardization procedure specification** [Liang]
Specify exactly how the primary outcome was standardized: (a) using each respondent's mean across their two maps, then standardizing across respondents; or (b) treating each respondent-map observation as an independent data point and standardizing across all 4,800 observations. These produce different standardized values and the choice should be explicit.

**P4-D: Likert item validity in algorithmic condition** [Liang]
Item 2 of the fairness scale ("the people who drew these districts treated different communities equally") refers to "people" in a condition where no people drew the districts. Flag this as a potential within-scale validity issue in the algorithmic condition and report whether item 2 shows systematically different response patterns in the algorithmic vs. non-algorithmic conditions. If it does, report sensitivity analysis excluding item 2.

**P4-E: Pilot sample population** [Liang]
Specify whether the pilot sample (n=200 for scale validation) was drawn from the same platform and population as the main study. If the pilot used MTurk only and the main study pools MTurk and Lucid, note that scale properties may not fully transfer across platforms.

---

## Revision Actions by Section

| Section | Actions Required | Priority |
|---|---|---|
| Abstract | None — accurate as written | — |
| 1. Introduction | Revise "structurally incapable" framing (P2-A); correct Rucho characterization (P3-D) | P2, P3 |
| 2. Related Work | None required | — |
| 3. Design | Add treatment text to appendix (P1-C); clarify commission condition accuracy (P3-C); specify stage 4 sequencing (P4-A) | P1, P3, P4 |
| 4. Measures | Specify standardization procedure (P4-C); flag Likert item validity concern (P4-D); report pilot sample population (P4-E) | P4 |
| 5. Results | Reframe moderation as exploratory (P1-A); add R² note (P2-D); report actual viewing time (P4-B); report cross-platform comparisons (P3-B); flag post-treatment support measure (P2-E) | P1, P2, P3, P4 |
| 6. Discussion | Add equivalence framing for partisan gap (P2-C); address voice-in-process directly (P3-E); revise effect size comparison to literature (P2-B) | P2, P3 |
| 7. Conclusion | None required | — |
| Appendix (new) | Add: treatment text (90-word description; full description + comparison); H2 revision or clarification (P1-B); sensitivity analysis for high-awareness respondents (P3-A) | P1, P3 |

---

## Hypotheses: Revision Needed

**H2** needs revision or clarification. As written: "A plain-language description of the algorithmic process will increase perceived fairness of algorithmic maps, but not enacted maps." The "but not enacted maps" clause cannot be tested because there is no enacted × information condition. Revise to: "A plain-language description of the algorithmic process will increase perceived fairness of algorithmic maps; this effect will be significantly larger for algorithmic and commission maps than for the enacted baseline" — or add a note that the second clause is inferred from the non-significance of a specification that includes enacted-map intercept and the process-description main effect.

---

## Issues Not Requiring Revision

- OSF pre-registration URL redacted — acceptable for blind review; restore before final publication.
- Two-state design limiting generalizability — adequately disclosed in limitations (Section 6.4).
- Experimenter demand effects — adequately disclosed in limitations (Section 6.4).
- Elite cueing external validity concern (Rodden) — acknowledged in conclusion as future research direction; no revision required but could be strengthened.

---

## Estimated Revision Burden

- Priority 1: 2-3 paragraphs (P1-A, P1-B: reframing; P1-C: appendix addition)
- Priority 2: 2-3 paragraphs + minor table revisions (P2-A through P2-E)
- Priority 3: 1-2 paragraphs + sensitivity analysis (P3-A requires data work; P3-B, P3-C, P3-D, P3-E are text only)
- Priority 4: 2-3 paragraphs + data checks (P4-A through P4-E mostly text + minor data)

Total estimated effort: **Medium** — no new data collection or new experiments required. All revisions are text, clarification, sensitivity analysis, or appendix additions.
