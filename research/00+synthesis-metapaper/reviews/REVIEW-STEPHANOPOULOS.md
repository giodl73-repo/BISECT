# Quality Assessment: Algorithmic Objectivity for Congressional Redistricting

**AI Persona**: Nicholas Stephanopoulos (based on work at University of Pennsylvania Law School)
**Expertise Area**: Gerrymandering, efficiency gap, election law
**Round**: 1
**Date**: 2026-02-08

> **Simulation Notice**: This is AI-generated feedback for quality improvement, not a real peer review. Use these insights to strengthen your work.

---

**Content Mode**: full

---

## Overall Assessment

This paper makes an important contribution to post-*Rucho* redistricting reform by demonstrating that algorithmic methods can satisfy constitutional requirements at national scale. The "impossibility defense" concept is legally innovative and the VRA surplus finding (+69 MM districts) is empirically significant. As a co-creator of the efficiency gap metric, I'm pleased to see partisan symmetry analysis included in the evaluation framework.

However, the paper's treatment of partisan fairness is incomplete and potentially misleading. Reporting that algorithmic maps produce 56.5% Democratic-leaning districts without calculating the efficiency gap or comparing to proportionality standards leaves readers unable to assess whether this represents fair, biased, or neutral outcomes. The paper correctly notes that geography drives partisan patterns, but doesn't grapple with whether **geographic bias** is legally or normatively acceptable.

More fundamentally, the paper underestimates the *Rucho* justiciability problem. The Court rejected partisan gerrymandering claims not because better methods don't exist, but because courts lack **manageable standards** to evaluate them. Saying "use this algorithm" replaces one non-justiciable question (is this map too partisan?) with another equally difficult question (are these parameters correct?). The paper needs to address how courts would review algorithmic districting disputes.

For Science, this paper makes valuable contributions but needs stronger partisan analysis and more realistic assessment of legal obstacles to adoption. The technical work is impressive; the normative framing needs refinement.

## Score

**Score**: 3/4 — Accept (with partisan fairness revisions)

## Major Issues (Blocking)

### M1: Efficiency Gap Not Calculated

Section 4, Finding 4 reports "56.5% Democratic-leaning districts" but doesn't calculate the efficiency gap—the metric I developed with Eric McGhee specifically to quantify gerrymandering. The efficiency gap measures wasted votes: votes for losing candidates and surplus votes beyond what's needed to win. A map with 0% efficiency gap produces proportional representation; positive gaps favor Republicans, negative gaps favor Democrats.

Without efficiency gap calculation, we can't assess partisan fairness. 56.5% Democratic districts could represent:
- **Fair outcome** if Democrats win 56% of votes
- **Republican bias** if Democrats win 60% of votes
- **Democratic bias** if Democrats win 52% of votes

**Required**: Calculate efficiency gaps for:
- All 50 states (algorithmic vs. enacted plans)
- 2016, 2018, 2020 election results (to test temporal stability)
- Report mean, median, and outliers (states with >7% gaps)

This transforms descriptive partisan data into normative fairness assessment.

### M2: Proportionality Standard Absent

The paper treats partisan asymmetry as inevitable geographic artifact but doesn't assess whether outcomes are **proportional** to vote shares. This is the core normative question: should districts reflect statewide vote proportions, or is geographic bias acceptable?

My work argues that systematic deviations from proportionality (>2 seats in partisan composition vs. vote share) constitute actionable gerrymandering. The paper should report:
- Statewide Democratic vote share (avg 2016-2020)
- Predicted Democratic seat share from algorithmic districts
- Difference between vote share and seat share
- Comparison to enacted plans

**Required**: Add proportionality analysis showing whether algorithmic maps produce outcomes closer to or farther from proportional representation than enacted plans. If algorithmic maps show 56.5% Democratic districts but Democrats win 52% of votes, this 4.5pp gap should be evaluated against enacted plan gaps.

### M3: Justiciability Problem Unresolved

Section 5 suggests algorithmic methods provide "manageable standards" that address *Rucho*'s concerns. But *Rucho* rejected partisan gerrymandering claims because courts can't determine what level of partisan asymmetry is "too much." The majority opinion (Roberts) explicitly stated that even if partisan effects can be measured, courts lack authority to decide acceptable thresholds.

Algorithmic redistricting faces the same justiciability problem: **who decides the parameters?** If plaintiffs argue edge-weights should be higher to create more MM districts, what standard do courts apply? Parameter choices are inherently political—delegating them to algorithms doesn't eliminate politics, it obscures them.

**Required**: Add discussion addressing:
- How would courts review parameter disputes? (arbitrary and capricious? rational basis?)
- If state chooses parameters producing 3 MM districts but alternative parameters yield 4, can plaintiffs challenge?
- Does algorithmic "objectivity" provide judicially manageable standards or just relocate discretion?
- Distinguish between **procedural improvements** (transparency, reproducibility) and **substantive justiciability** (courts can determine right answer)

This is the central barrier to legal adoption—the paper needs to confront it directly.

## Minor Issues

### m1: Seats-Votes Curve Not Reported

My work uses seats-votes curves to assess responsiveness and bias. A fair map should show symmetric seats-votes relationship: if Party A wins 55% of votes, they should win approximately 55% of seats, and vice versa for Party B. The paper reports partisan lean but not the seats-votes elasticity.

Report: If Democrats win X% of votes, they win Y% of districts. Do this for vote shares from 45-55% to show whether algorithmic maps exhibit partisan bias or symmetric responsiveness.

### m2: Competitive Districts Analysis Missing

Competitive districts (45-55% vote margin) are normatively valuable for accountability and representation. My work shows that bipartisan gerrymanders often eliminate competitive districts to protect incumbents. Does your algorithm create more or fewer competitive districts than enacted plans? This is an independent dimension of fairness beyond proportionality.

### m3: Mean-Median Difference Not Calculated

The mean-median difference is another metric for detecting partisan bias: if the median district leans more Republican than the mean district, this suggests packing of Democrats. Report mean-median differences for algorithmic vs. enacted plans.

### m4: Durability Analysis Incomplete

One advantage of algorithmic redistricting is that it's reproducible after each census. But if geographic sorting intensifies over time (as urbanization continues), will algorithmic maps produce increasing Republican bias? Report whether efficiency gaps/proportionality deviations grow over your 2000-2020 window.

### m5: Alternative Metrics Needed

While efficiency gap is useful, it has limitations (debated in literature). Report additional partisan metrics:
- Declination: angle between seats-votes curve and 45-degree line
- Partisan bias: seats won at 50% vote share
- Lopsided margins: difference in average victory margins

If multiple metrics agree, this strengthens fairness claims.

## Strengths

1. **VRA surplus finding is important**: Demonstrating neutral methods can exceed enacted plans on VRA compliance is valuable for Section 2 litigation.

2. **Impossibility defense is novel**: Even if not legally dispositive, this framing advances the discourse and provides courts with new conceptual tools.

3. **Huntington-Hill analogy is effective**: Historical precedent for mathematical governance makes the argument accessible to non-specialists.

4. **National scale validation**: 50-state analysis provides external validity rare in redistricting research.

5. **Acknowledges partisan effects**: The paper honestly reports partisan outcomes rather than claiming algorithmic neutrality eliminates partisan consequences.

## Questions for Authors

1. Have you calculated efficiency gaps for algorithmic maps? If so, how do they compare to enacted plans?

2. The 56.5% Democratic figure—is this consistent across states or driven by a few large states? State-level variation matters for legal adoption.

3. If algorithmic maps produce systematic partisan bias due to geography, should algorithms try to correct for this? What are the constitutional implications?

4. Have you tested whether your algorithm could detect known gerrymanders? (e.g., compare to North Carolina 2011, Maryland 2011—would the algorithm produce substantially different maps?)

5. How sensitive are partisan outcomes to parameter choices? If edge-weighting changes produce different partisan effects, this undermines the "objectivity" claim.

## Recommendations

- Calculate efficiency gaps for all states, compare to enacted plans
- Add proportionality analysis (vote share vs. predicted seat share)
- Report seats-votes curves to assess symmetric responsiveness
- Analyze competitive district rates compared to enacted plans
- Calculate mean-median differences and alternative partisan metrics
- Address justiciability problem directly (parameter disputes, judicial review standards)
- Test parameter sensitivity on partisan outcomes
- Report temporal trends in partisan metrics (2000-2020)
- Consider historical gerrymander detection as external validation
- Discuss whether geographic bias is normatively acceptable
- Add section on "Limitations of Algorithmic Objectivity"

---

**Verdict**: This paper makes valuable technical and legal contributions to redistricting reform. The algorithmic work is sound and the VRA findings are important. However, the partisan analysis is incomplete—reporting Democratic district percentages without efficiency gaps, proportionality assessment, or fairness metrics leaves readers unable to evaluate partisan outcomes. The justiciability problem also needs direct confrontation: algorithmic methods improve transparency but don't resolve the fundamental question of who decides what's fair. With revisions addressing partisan fairness metrics and legal adoption barriers, this will be a strong Science paper that bridges computer science, law, and political science. The work deserves publication in a high-impact venue after addressing these analytical gaps.
