> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied.

---

# Round 3 Review: Recursive Bisection for Congressional Redistricting

**Reviewer**: Nicholas Stephanopoulos (Harvard Law School)
**Expertise**: Election law, redistricting doctrine, efficiency gap, partisan gerrymandering
**Round**: 3
**Date**: 2026-05-05

---

## Summary

This is a first read for me on this paper; I was not part of the prior review rounds. I approach the paper from an election law perspective, with particular interest in the paper's legal claims, the efficiency gap and partisan fairness analysis, and the relationship between algorithmic redistricting and post-*Rucho* doctrine.

## Strengths

**1. The impossibility defense framework is legally significant.**

The paper's core legal claim — that an algorithm which cannot access partisan data cannot be used to intentionally gerrymander — is sound and aligns with how courts think about intent-based redistricting claims. Under *Rucho v. Common Cause* (2019), federal courts cannot adjudicate partisan gerrymandering claims. The paper correctly identifies state constitutional litigation as the viable pathway. The argument that "algorithmic selection eliminates the intent element" is persuasive for state constitutional claims that require discriminatory intent (e.g., *League of Women Voters v. Commonwealth of Pennsylvania*).

However, the paper does not fully engage with the efficiency gap as an analytical tool, which is surprising given its prominence in post-*Rucho* litigation. The efficiency gap (EG) is the most widely accepted quantitative metric for partisan gerrymandering, having been cited in *Gill v. Whitford* and multiple state court cases. If the paper is making claims about "partisan neutrality" or "gerrymandering resistance," it should compute EG values for algorithmic versus enacted plans and show whether algorithmic plans fall within the EG thresholds courts have found acceptable (typically |EG| < 8%).

**2. The VRA analysis is sophisticated and legally accurate.**

Section 5.6's treatment of *Allen v. Milligan* (2023), the Gingles three-part test, and the distinction between compactness as a defense and compactness as a limit to VRA liability is accurate and well-sourced. The finding that algorithmic plans produce 137 majority-minority districts versus 68 in enacted plans (+69 surplus) is striking and legally relevant: it directly challenges the common claim that geometric optimization is hostile to minority representation.

The discussion of coalition districts (total minority approach) is legally sophisticated and reflects the current doctrinal debate following *Bartlett v. Strickland* (2009). The paper correctly notes that coalition districts are legally more contested than single-group majority districts. This nuance is important for practitioners.

**3. The state constitutional litigation pathway is the right frame.**

After *Rucho*, the paper correctly identifies state courts and state constitutional provisions as the operative arena for redistricting reform. The discussion of how algorithmic maps can provide a "manageable standard" under state constitutional anti-gerrymandering clauses (e.g., Pennsylvania, Michigan, Ohio) is a valuable legal contribution.

## Weaknesses

**1. Efficiency gap analysis is absent.**

The paper does not compute efficiency gap values for algorithmic versus enacted plans. Given my work developing the efficiency gap metric and its role in post-*Rucho* litigation, this omission is surprising. The paper discusses partisan bias qualitatively but does not use the standard quantitative metric that courts and litigants actually rely on. An EG table for all 50 states (algorithmic vs. enacted) would be the single most legally actionable addition to the paper.

Expected pattern based on geographic sorting literature: algorithmic plans will show small but systematic pro-Republican EG values in states with high urban-rural sorting (e.g., Wisconsin, Ohio, Pennsylvania), because compact districts in those states will pack Democrats more efficiently than Republicans. This pattern, if documented, should be discussed as an "unavoidable geographic efficiency gap" rather than evidence of partisan manipulation — a legally important distinction.

**2. The relationship between compactness and the efficiency gap is underexplored.**

There is a well-documented theoretical tension: compact districts in states with geographic sorting will tend to pack urban Democrats, producing a structural Democratic efficiency disadvantage. The paper's claim that edge-weighted optimization produces "neutral" plans does not fully engage with this tension. The paper would benefit from a section explicitly analyzing whether compactness and partisan fairness are compatible objectives in states with strong geographic sorting, and what tradeoffs practitioners face.

**3. *Rucho*'s non-justiciability holding needs more careful treatment.**

The paper discusses *Rucho* but does not fully engage with the holding's implications for the impossibility defense. *Rucho* did not say that partisan gerrymandering is permissible — it said federal courts cannot adjudicate it. State courts remain free to hear such claims. The paper should clearly distinguish: (a) the federal non-justiciability holding (no federal remedy), (b) state constitutional claims (viable pathway), and (c) the algorithmic approach as a policy solution (not a legal remedy). These three tracks are sometimes conflated in the paper.

## P1 Items (New)

None blocking for political science publication.

**High priority addition** (not blocking but legally important):
- Efficiency gap values for all 50 states (algorithmic vs. enacted plans)
- Discussion of unavoidable geographic efficiency gap versus gerrymandering premium
- Estimated effort: 1-2 days if partisan outcome data already exists (paper suggests it does)

## P2 Items

- **Efficiency gap analysis**: The most legally relevant addition. Courts use EG; the paper should speak directly to this metric.
- **Compactness-EG tradeoff discussion**: Theoretical treatment of the tension between compact districts and partisan fairness in states with geographic sorting.
- **Clearer *Rucho* track distinction**: Separate federal non-justiciability, state constitutional claims, and algorithmic policy solutions as three distinct tracks.

## Score

**Score: 3.5/4 — Accept with Minor Revisions**

The paper makes a genuine legal and algorithmic contribution. The impossibility defense framework is sound, the VRA analysis is legally accurate, and the state constitutional litigation pathway is correctly identified. The absence of efficiency gap analysis is the primary weakness from an election law perspective: the paper's neutrality claims need to be tested against the standard quantitative metric that courts actually use. This addition would be straightforward given the paper's existing partisan outcome data and would make the legal contributions substantially more actionable for practitioners and litigants.

**Recommendation**: Accept with minor revisions. Efficiency gap analysis is the priority addition. With EG values and a discussion of the geographic efficiency gap versus gerrymandering premium, I would recommend Strong Accept.
