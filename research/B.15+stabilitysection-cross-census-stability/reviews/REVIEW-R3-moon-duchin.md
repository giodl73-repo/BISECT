> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review R3: StabilitySection: Cross-Census Stability of GeoSection-Optimal Redistricting Maps

**Reviewer**: Moon Duchin (Tufts University, MGGG Redistricting Lab)
**Expertise**: Redistricting mathematics, GerryChain ensemble methods, geometric probability, redistricting law, metric geometry
**Round**: 3
**Date**: 2026-05-02

## Overall Assessment

The revision adds the Wilson CI [48%, 82%], the null hypothesis calculation (67% is
3.4× the binomial null of 20%), and the Iowa Lorenz p* proxy values. The Iowa CSS
table correction (moved to Low CSS) is the most important internal fix. These changes
address two of the three issues I listed as blockers in Round 2. My remaining concern
about the abstract overstating CSS computability is partially but not fully resolved.
I am upgrading to 3.5/4.

## Score: 3.5/4

**My score**: 3.5/4 — Null hypothesis and Wilson CI address my principal concerns;
Iowa reclassification corrects the most glaring internal inconsistency; Lorenz proxy
values are a genuine improvement. Abstract framing is improved but still not fully
precise; proxy Jaccard validation remains absent. Both are fixable editorially before
submission.

## Changes Since Round 2: What Was Addressed

### Abstract Framing — Substantially Addressed

The abstract has been revised to lead with "ratio stability" and positions CSS as the
framework the ratio-stability finding contributes to, rather than presenting CSS as the
computed metric. The revised framing — "we report that 67% of states exhibit ratio
stability at the first bisection level, the geometric foundation of the CSS framework" —
is close to what I recommended in Round 2. The s_seat component (weighted 0.5 in the
CSS formula) is still marked [TBD] in the body text, which is consistent with the
revised abstract. The remaining tension: the paper's title still mentions "cross-census
stability" which implies a CSS-level result; the actual finding is a ratio-stability
result. This is a minor point for a theory paper, but the introduction should
acknowledge the gap between the ratio-stability measurement and the full CSS claim in
its first mention of the headline finding.

### Null Hypothesis — Resolved

The binomial null calculation is now present and well-executed. The 20% expected random
stability rate (derived from the mean of 1/(number of candidate ratios) across 30
states) provides the baseline that was missing through two rounds of review. The paper
correctly reports the observed 67% as 3.4× the null with binomial p < 0.001. For a
political science or law review audience, this quantification of the finding relative
to chance is the most important statistical addition in the revision.

The comparison between 67% (observed) and 20% (random null) also partially addresses
my Round 2 suggestion about the GerryChain null hypothesis: while GerryChain-based
comparison would be the gold standard, the binomial null is a credible first benchmark.
The paper could note that the GerryChain null — what fraction of compact-plan random
draws from the GerryChain ensemble agree with the census-year-optimal ratio — would
provide a stronger comparison. This is appropriate as a future work suggestion rather
than a current requirement.

### Iowa Reclassification — Resolved

Moving Iowa to Low CSS is the correct fix. The 2D stability matrix now reflects that
Iowa is the paper's primary census-instability case study, not a high-stability state.
The internal consistency between the case study (Delta-f = 0.31), the new p* values
(p*_2000 ≈ 0.65 → p*_2010 ≈ 0.55), and the Low CSS classification is now achieved.
This was the most glaring internal inconsistency in Round 2 and its resolution is
a substantial improvement.

### Iowa Lorenz p* Proxy — Addressed

The p*_2000 ≈ 0.65 and p*_2010 ≈ 0.55 values ground the Lorenz drift narrative with
actual numbers. The direction is consistent with Hypothesis A (suburban sprawl
reducing the isoperimetric advantage of the asymmetric peel). The values are described
as derived from the Lorenz curve of population density by geographic area — which is
the correct conceptual definition. My question from Round 2 about the formal
relationship between p* and the actual Jaccard is not answered by these values, but
their inclusion is a meaningful improvement over the projected/circular treatment in
Round 2.

### Wilson CI — Resolved

The Wilson CI [48%, 82%] is correctly computed and now appears in the appropriate
location alongside the 67% headline finding. The paper correctly notes that even the
lower bound (48%) substantially exceeds the 20% null. This is sufficient statistical
characterisation for the 30-state sample.

## Remaining Issues

### Issue 1: Proxy Jaccard Relationship to Actual Jaccard Still Not Validated
**Severity**: Low-Medium (unchanged from Round 2)
The paper still does not establish a formal or empirical relationship between Delta-f
and the actual district-level Jaccard score. The Hungarian algorithm pseudocode in
Section 4.4 shows that the actual Jaccard computation is feasible from pipeline output.
Even three states with computed Jaccard values would validate the proxy. This was
a Round 2 request that remains unaddressed. For the current submission target (a
political science or law review venue), the proxy's intuitive justification is
probably sufficient; for a methodology venue, validation would be required.

### Issue 2: Abstract Title–Measurement Gap
**Severity**: Low
The title "Cross-Census Stability" implicitly claims a CSS-level result; the actual
measurement is ratio stability. A subtitle or parenthetical acknowledgment in the
introduction ("we measure ratio stability as the primary observable component of
cross-census stability") would resolve this precisely. This is editorial rather than
substantive.

### Issue 3: p* Computation Specificity
**Severity**: Low
The p* values are reported without specifying whether they are computed from census
tract population data or from the pipeline's adjacency graph edge weights. If computed
from tract data, the paper should state "computed from the 2000 Iowa census tract
population density Lorenz curve." This is a reproducibility concern, not a correctness
concern.

## Assessment

The null hypothesis, Wilson CI, Iowa reclassification, and Lorenz p* proxy values
represent genuine improvements that address the principal concerns from Round 2. The
paper's most important statistical deficiencies are now resolved. The proxy Jaccard
validation remains the principal unresolved methodological issue, but the bar for a
theory paper at a political science venue is met by the current treatment. The paper
is ready for submission with minor editorial revisions to the abstract framing and the
p* computation description.

**Score: 3.5/4 — Accept with minor revisions.**
