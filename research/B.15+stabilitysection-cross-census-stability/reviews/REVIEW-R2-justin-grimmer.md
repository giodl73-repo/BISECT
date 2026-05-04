# Review R2: StabilitySection: Cross-Census Stability of GeoSection-Optimal Redistricting Maps

**Reviewer**: Justin Grimmer (Stanford University, Department of Political Science)
**Expertise**: Quantitative political methodology, computational text analysis, redistricting measurement, causal inference, electoral politics
**Round**: 2
**Date**: 2026-05-02

## Overall Assessment

The revision represents meaningful progress on the Iowa case study and the threshold sensitivity analysis. The Iowa section (4.3.3) now provides a concrete mechanistic decomposition — Hypothesis A (Lorenz drift) vs. Hypothesis B (tract redesign) — and correctly identifies the experiment that would distinguish them. The threshold sensitivity table responds to the implicit concern about the 5% threshold's arbitrariness with empirical evidence that the 67% finding is not sensitive to threshold choice over a reasonable range.

The two major methodological concerns from Round 1 are only partially addressed. First, the three uncontrolled confounders in the cross-census comparison (population change, graph topology change, seat count change) are now acknowledged in the limitations section, but the Type I/Type II decomposition experiment that would control for the first two has not been executed. The s_seat clarification addresses the third confounder definitionally, but the empirical consequences have not been explored. Second, the causal inference gap — that cross-census stability does not rule out partisan determination if partisan and physical geography are correlated — has received a more careful discussion in Section 5.1 but still does not directly address the "stable partisan geography" alternative hypothesis I described.

I maintain my Round 1 score of 3.0/4. The paper is making progress but the central causal claim about geographic determinism remains undertested.

## Score: 3.0/4

**My score**: 3.0/4 — Iowa mechanism and threshold sensitivity are genuine improvements; Type I/Type II decomposition still not executed; causal inference gap improved but not closed; null hypothesis (random stability rate) still absent.

## Changes Since Round 1: What Was Addressed

### Iowa Case Study (Issue 3 from R1 — Substantially Addressed)

The addition of Section 4.3.3 is the revision's most valuable contribution. The distinction between Hypothesis A (suburban sprawl reducing isoperimetric advantage of asymmetric peel) and Hypothesis B (tract subdivision changing graph topology) is precisely the mechanistic decomposition the paper needed. The description of Iowa's tract count increase from 1,099 (2000) to approximately 1,286 (2010) driven by suburban tract subdivision is a specific, verifiable empirical observation that directly motivates Hypothesis B.

The authors correctly identify the decisive experiment: run GeoSection on the 2020 graph with 2000 population weights. If $f \approx 0.49$, the topology change is dominant; if $f \approx 0.18$, the population change is dominant. This experiment has not been run, but framing it as a decision procedure is more methodologically rigorous than the Round 1 treatment of Iowa as a data point.

What is still missing: the $p^*_{2000}$ value. The paper projects $p^*_{2000} \approx 0.25$ from the ratio geometry, which is circular (the value is derived from the outcome being explained). Computing $p^*_{2000}$ independently from the Lorenz curve of the 2000 population distribution would provide genuine evidence for or against Hypothesis A.

### Seat Count Confounder (Issue 1 from R1 — Partially Addressed)

The s_seat clarification correctly removes seat-count-changing states (MT, TX, NY named explicitly) from the "unstable" classification. The limitations section acknowledges that "a state like Texas ($k=30$ in 2000, $k=38$ in 2020) has a ratio comparison that conflates algorithm stability with apportionment-change effects."

However, the empirical consequence is not explored. Of the 30 comparable states in the analysis, how many changed seat count? If a substantial fraction of the 10 "unstable" states are also seat-count-changing states, the instability may reflect apportionment confounding rather than genuine geographic reorganisation. The paper does not report this breakdown.

### Causal Inference Gap (Issue 2 from R1 — Partially Addressed)

Section 5.1 now includes the three-part argument structure I recommended: (1) GeoSection uses no partisan data by construction; (2) partisan data is not available for 2000 and 2010 inputs; (3) the stability claim is about algorithm invariance to the specific partisan distribution, not about whether physical and partisan geography are correlated. This framing is a significant improvement.

What is still not addressed: the alternative hypothesis I described, where a partisan-sensitive algorithm tracking stable partisan geography would produce the same cross-census stable outcomes. The paper's three-part argument rebuts the "GeoSection secretly uses partisan data" concern but does not address the "partisan geography mimics physical geography" concern. These are different arguments. The "before living memory" framing addresses this implicitly (if the outcome predates the current partisan alignment, it can't be tracking the current alignment), but the paper does not make this argument explicitly for the 2000--2020 comparison.

## Remaining Issues

### Issue 1: Type I/Type II Decomposition Not Executed
**Severity**: High
**Description**: The Iowa case study correctly identifies the Type I/Type II experiment as decisive, but the experiment has not been run. The paper now frames this as future work with a specific design: run GeoSection on the 2020 graph with 2000 population weights. This is more precise than Round 1, but the decomposition is needed to support the paper's main empirical claim.

The 10 unstable states include Iowa ($\Delta f = 0.31$), Alabama ($\Delta f = 0.06$), South Carolina ($\Delta f = 0.07$), and Utah ($\Delta f = 0.07$). For the paper's geographic determinism argument to hold, at least some of these instabilities should be driven by genuine population change (Hypothesis A) rather than graph topology change (Hypothesis B). The paper cannot distinguish these without running the experiment.

**Recommendation**: Run the Type I/Type II decomposition for at least Iowa (the most dramatic case) and one other unstable state (Alabama or Utah). Report whether the instability survives the population-only perturbation. If Hypothesis B dominates for Iowa, the implications are significant: the paper's headline instability case is a data artifact, not a geographic reorganisation.

### Issue 2: Null Hypothesis (Random Stability Rate) Still Absent
**Severity**: Medium
**Description**: The Round 1 review noted that "what would we expect the stability rate to be if ratios were assigned randomly?" For a typical state with $\approx 5$ candidate ratios, random assignment would give $\approx 20\%$ stability. The 67% finding is substantially above this baseline. The paper still does not compute the null hypothesis or compare the 67% finding against it.

This comparison is important because it quantifies how much the geographic structure constrains the algorithm's output relative to chance. A 67% stability rate that is 3× the random baseline ($\approx 20\%$) is a substantive finding; a 67% rate against a 50% baseline would be less impressive.

**Recommendation**: Compute the expected random stability rate for the dataset. For each state, calculate the number of candidate ratios (positions in the natural ratio scan) and the probability that two independent random draws from that candidate set match. Report the aggregate expected random stability rate and compare it to the 67% observed rate.

### Issue 3: Confounders by Category Not Reported
**Severity**: Medium
**Description**: Of the 10 unstable states, how many also changed seat count? The paper lists 15 seat-count-changing states in Table 2 and says the normalised ratio comparison addresses this, but does not report how many of the 10 unstable states overlap with the seat-count-changing states. If 6 of the 10 unstable states are seat-count-changing states, the instability rate for stable-seat-count states is 4/15 = 27% — lower than the headline 33% but still substantial.

**Recommendation**: Cross-tabulate the stable/unstable classification against seat-count-change status and report the instability rate for each group. This directly addresses the apportionment-change confounder from Issue 1 of Round 1.

## Minor Issues

- The "structurally volatile" description of Sun Belt states (AZ, TX, FL) is now in the limitations section. I suggested this framing could be developed further as a theoretical contribution (states in ongoing geographic transitions reaching a new stable equilibrium). This is still an underexploited connection to the political geography literature on urban-rural sorting dynamics.

- The 2D stability matrix (Table 6) still has Iowa classified as "High seed stability, High census stability" despite Iowa having $\Delta f = 0.31$ — the highest instability in the dataset. Either Iowa should move to the top-left cell, or the matrix should be redesigned to reflect the post-revision understanding that Iowa is census-unstable.

- The "before living memory" argument now correctly cites Rodden 2019 for evidence that the urban-rural divide was less pronounced in 2000 than 2020. This is an improvement over Round 1, but the specific claim — that GeoSection produces the same 5D/9R in Wisconsin in both 2000 and 2020 — requires Wisconsin 2000 data to verify. If Wisconsin is in the 47-state 2000 sweep, this can be stated as confirmed rather than predicted.

- The Iowa discussion mentions 2000 natural ratio as 1:3 (matching $f = 0.18$) and 2010 as 2:2 (matching $f = 0.49$). This is stated in the case study but the Iowa row in Table 3 still shows 2020 as "2:2" with "Same" stability — which is now inconsistent with the new finding that 2000 Iowa was $1:3$.

## Questions for Authors

1. Of the 10 unstable states, how many also changed seat count? What is the instability rate for fixed-seat-count states only?

2. Is Wisconsin's 2000 sweep complete? If so, does it confirm the 1:7 ratio as predicted?

3. For the Iowa row in Table 3: why does it still show "Same" stability prediction if the case study confirms $f_{2000} = 0.18$ (ratio 1:3) vs. $f_{2020} = 0.49$ (ratio 2:2)?

4. For the "before living memory" argument: can the paper confirm that Wisconsin's 5D/9R outcome appears in the 2000 GeoSection run, as asserted in Section 5.1?

## Recommendation

The revision has improved on the Iowa mechanism and the threshold sensitivity. These were the most actionable changes from Round 1. The paper's central causal claim about geographic determinism is getting clearer, but the Type I/Type II decomposition is still needed to fully support the instability findings, and the null hypothesis comparison remains absent. I maintain my score of 3/4. The paper would be in good shape for journal submission after adding the null hypothesis computation, reporting the cross-tabulation of instability by seat-count-change status, and reconciling the Iowa row in Table 3 with the case study findings.
