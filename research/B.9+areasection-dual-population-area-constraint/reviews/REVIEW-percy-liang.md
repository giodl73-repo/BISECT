# Review R-2: Percy Liang
**Paper**: AreaSection: Simultaneous Population and Land-Area Balance in Minimum-Edge-Cut Redistricting
**Date**: 2026-05-03
**Score**: 3.0 / 4

---

## Summary

The paper introduces AreaSection with a Lorenz curve pre-filter, a 50-state empirical sweep (44/50 complete), and partisan analysis on WI and NC. The paper claims AreaSection is "a geographic fairness mechanism, not a partisan mechanism" based on identical seat counts in two states and an 8-state Rodden effect analysis showing -6.2pp mean proportionality gap.

The scale of the empirical commitment is commendable. The Lorenz curve pre-filter is practically useful. However, the evaluation design has weaknesses that limit confidence in the central comparative claim.

---

## Strengths

**S1. Scale of empirical commitment.** The 50-state sweep is the right scope for a claim about a national redistricting mechanism. Running all 50 states and reporting failures honestly is scientifically appropriate.

**S2. Lorenz curve pre-filter as a diagnostic tool.** The use of the Lorenz curve to characterize feasible population-area ratio space converts an opaque solver failure mode into an interpretable geometric condition. This is the strongest methodological contribution.

**S3. Honest reporting of the 6 failures.** Reporting FL, IL, MI, NY, PA, and TX as failures rather than silently excluding them is the correct scientific posture.

**S4. Rodden effect validation.** The 8/8 competitive states finding, framed as validation of the geographic efficiency gap literature, is appropriate and situates the paper well in an established empirical debate.

---

## Weaknesses

**W1. The head-to-head comparison is critically underspecified.**
The central comparative claim — that AreaSection produces "identical seat counts" relative to GeoSection — rests on exactly two states: WI and NC. WI and NC are not a random sample; they are among the most-studied states in the redistricting literature, introducing selection bias. The paper needs head-to-head comparison across at minimum all 44 states where both methods succeed.

**W2. The 6-state exclusion creates systematic bias in the partisan analysis.**
IL, NY, and TX — three of the six failures — are among the most politically consequential states in the country. IL and NY lean strongly Democratic; TX is a major Republican-controlled state with large urban-rural disparity. Excluding all three from the partisan analysis while claiming "geographic not partisan" is methodologically problematic.

**W3. Seed sensitivity is uncharacterized for AreaSection.**
The paper uses 50 seeds per ratio but does not report variance across seeds. For ncon=2, the constraint surface is harder and convergence behavior is not analyzed. A finding like "mean -6.2pp" is uninterpretable without knowing whether the distribution across seeds is narrow or wide.

**W4. Single election cycle as political ground truth.**
The partisan analysis relies exclusively on 2020 presidential election returns, which are a known outlier (high polarization, COVID-altered turnout, Trump-specific rural overperformance). The competitive-state classification and the -6.2pp finding could change materially with 2016 or 2018 data. The paper does not test this.

**W5. The "geographic fairness, not partisan mechanism" claim is not falsifiable as presented.**
Showing that AreaSection produces Republican over-representation matching the Rodden prediction does not distinguish between (a) the effect is entirely geographic and (b) the algorithm amplifies geographic sorting in a way that interacts with partisan geography. A mediation analysis or counterfactual construction would be needed to separate them.

---

## P1 Items (Must Fix Before Acceptance)

**P1-I. Expand the head-to-head comparison to all 44 completed states.** Report seat counts and partisan lean for both GeoSection and AreaSection across all states where both methods succeed. Present a scatter plot of GeoSection seats vs. AreaSection seats. If they are truly identical for most states, this is a strong result.

**P1-II. Address the 6-state exclusion explicitly in the partisan analysis section.** At minimum, bound the potential effect: what range of partisan outcomes for IL, NY, PA, TX, FL, MI would change the -6.2pp finding and in which direction?

**P1-III. Report seed variance for AreaSection.** For each state in the partisan analysis, report the distribution of seat outcomes across the 50 seeds. If variance is low, this strengthens the finding.

---

## P2 Items (Should Fix)

**P2-I.** Replicate the partisan analysis using at least one non-presidential election cycle (2018 midterm returns).

**P2-II.** Characterize the WI/NC comparison selection criteria if they remain the primary case studies.

**P2-III.** State why 50 seeds was chosen for AreaSection and provide a convergence analysis.

**P2-IV.** Clarify the ubvec=[1.001, 1.10] choice and provide a small ablation over area tolerance.

**P2-V.** Report the Lorenz pre-filter's precision and recall: of Lorenz-feasible ratio predictions, what fraction actually converge?

---

## Verdict

Accept with Major Revisions. The paper identifies a real problem, proposes a principled algorithmic solution, and reports an honest empirical sweep. The Lorenz curve pre-filter is a genuine contribution. However, the evaluation design in its current form cannot sustain the central comparative and normative claims. A two-state head-to-head is not sufficient for a claim about national redistricting methodology. The exclusion of the six largest failure states, combined with reliance on a single election cycle and uncharacterized seed variance, means the -6.2pp finding and the "geographic not partisan" conclusion are not adequately grounded. If the authors expand the head-to-head comparison to all 44 completed states, bound the 6-state exclusion effect, and report seed variance, the claims will be substantially better supported.

**Score: 3.0 / 4**
