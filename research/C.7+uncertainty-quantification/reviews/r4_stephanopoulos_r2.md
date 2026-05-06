---
reviewer: Nicholas Stephanopoulos
round: 2
score: 4
date: 2026-05-05
---

## Summary

The authors have directly addressed my primary Round 1 P1 item — the missing legal-relevance mapping — with a new subsection in Section 7 that distinguishes how courts should treat each of the three uncertainty sources. The mapping is accurate, actionable, and legally precise: seed variance → determinism claims, census uncertainty → Wesberry compliance, shapefile resolution → compactness measurement challenges. The Wesberry framing for census uncertainty is especially apt: the paper now states that the ±0.002 propagated uncertainty means the expert "can certify Wesberry compliance with high confidence," which is the kind of affirmative legal framing that makes this paper useful beyond its academic audience. My other two P1 items — the CI interpretation language and the single-source vs. joint-source citation guidance — are not directly revised but the new legal-relevance mapping partially addresses the "which CI for which challenge" question I raised.

## Evaluation of Revisions

**P1.1 (Legal-relevance mapping)**: Fully resolved. The new "Legal Relevance of Uncertainty Sources" subsection in Section 7 is well-constructed. The three-item structure maps neatly onto the three-source taxonomy and answers the question "which uncertainty source does a court care about?" for each category of legal challenge. The explicit recommendation that experts should "report the DIA-seed PP value and its 95% seed-variance CI" when addressing determinism challenges is exactly the kind of actionable guidance that expert witnesses need. The Wesberry compliance framing is the strongest item: stating that the ±0.002 CI "means the expert can certify Wesberry compliance with high confidence" converts an abstract statistical finding into a litigation-ready affirmative claim.

**P1.2 (CI interpretation language)**: Not revised. The frequentist CI interpretation issue — the risk that judges or opposing counsel misunderstand "95% CI" as "95% probability the true value is in this interval" — is not addressed. However, the legal-relevance mapping's framing ("the data are consistent with values in the range [X, Y] at 95% confidence levels" is implied by the discussion even if not stated explicitly) partially mitigates the concern. The CI interpretation problem is real but is more of a presentation issue for expert witnesses than a problem with the paper itself.

**P1.3 (Single-source vs. joint-source citation guidance)**: Partially addressed. The legal-relevance mapping implicitly provides guidance: if only shapefile resolution is challenged, cite the resolution-only CI from Table 7.4; if all three sources are challenged, cite [+15%, +29%]. But this guidance is not stated explicitly as a decision rule. The paper would benefit from a one-sentence explicit statement: "When only one source is under challenge, cite the appropriate single-source CI from Table 7.4; when all sources are under challenge, cite the joint [+15%, +29%] CI."

**Abstract CI fix**: Resolved. The [+15%, +29%] three-source CI is now correctly stated.

**Bootstrap exchangeability argument**: Resolved. Clear and legally well-framed.

**Section 4.4 arithmetic error**: Resolved. The correction is both factually right and substantively improved.

## Remaining Concerns

The explicit citation-guidance decision rule (one sentence) would make Section 7 more actionable. The CI interpretation language could be clarified in a footnote. Neither prevents acceptance.

## Score: 4 — Accept

The legal-relevance mapping is the most important addition in this revision and directly addresses my primary P1 concern. The abstract CI fix and Section 4.4 correction are also important improvements. The paper is now fit for purpose as a litigation resource.
