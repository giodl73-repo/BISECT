> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review R2: StabilitySection: Cross-Census Stability of GeoSection-Optimal Redistricting Maps

**Reviewer**: Moon Duchin (Rutgers University, MGGG Redistricting Lab)
**Expertise**: Redistricting mathematics, GerryChain ensemble methods, geometric probability, redistricting law, metric geometry
**Round**: 2
**Date**: 2026-05-02

## Overall Assessment

The revision addresses the two main structural concerns from Round 1 in sensible ways. The s_seat problem has been partially resolved by the distinction between "structurally changed" states (MT, TX, NY — seat-count-changing states that are outside the CSS comparison scope) and the stable/unstable classification, which now rests on the computable ratio proxy $f$. The legal section no longer presents cross-census stability as established doctrine; it now reads as an explicit theoretical argument. The Jaccard proxy table is a reasonable response to the "Jaccard announced but not executed" criticism: the paper has added a concrete proxy computation with clear acknowledgment that the full district-level Jaccard is deferred.

My remaining concern is the framing of the Jaccard section: the paper still lists "Jaccard district-assignment stability" as a contribution in the introduction, and the new Section 4.4 provides a pseudocode implementation but no actual computed Jaccard values. The proxy ($\Delta f$ as a Jaccard substitute) is useful, but the gap between the proxy and the actual metric should be stated more precisely. I also have a methodological concern about the threshold sensitivity framing that was not present in Round 1.

## Score: 3.0/4

**My score**: 3.0/4 — Structural issues from R1 substantially addressed; Jaccard proxy is underspecified as a proxy; legal section improved but legal theory still needs one more round of precision; threshold sensitivity raises a new question about the metric's continuity structure.

## Changes Since Round 1: What Was Addressed

### CSS Formula vs. Available Data (Issue 1 from R1 — Addressed)

The paper now consistently describes the 2000--2010 findings as ratio-stability findings using the proxy $f$, not as full-CSS findings. The three-census CSS table remains [TBD] pending the 2010 sweep. The term "CSS-geo" was not adopted (the paper uses "proxy Jaccard" and "ratio stability" separately), but the distinction is now made in the text. The s_seat clarification — named MT, TX, NY as structurally changed rather than unstable — is exactly the right definitional fix.

The s_seat computation for 2000 and 2010 is still marked [TBD pending election data]. This is honest but means the CSS formula's seat-stability component (weighted 0.5) remains uncomputable. The paper should say this more explicitly in the abstract, which still presents CSS as if it were the paper's primary computed metric.

### Legal Theory (Issue 2 from R1 — Substantially Addressed)

The legal section now distinguishes the theoretical argument (what CSS $\geq 0.90$ would mean if we could compute it) from the current empirical finding (ratio stability only). Proposition 2 is now explicitly labeled as a proposed legal theory, not a claim about existing doctrine. The cases cited (Rucho, LWV v. PA, NC Harper, Allen v. Milligan) are correctly framed as establishing the post-Rucho state-court landscape rather than as precedents for CSS-based claims. This is a meaningful improvement.

### Jaccard Proxy (Issue 3 from R1 — Partially Addressed)

The Table 4 proxy Jaccard table provides the $\Delta f$ categorisation for all 30 comparable states. The Hungarian algorithm pseudocode in Section 4.4 shows how actual Jaccard values would be computed. But the introduction still lists Jaccard similarity as a paper contribution, and no actual Jaccard values appear in the paper. The proxy and the actual metric are different things: a state could have $\Delta f = 0$ (same ratio) but substantially different district boundaries if the population distribution within each half shifted.

This distinction is not made in the text. The proxy is presented as if it tracks the actual Jaccard, but the correlation between $\Delta f$ and district-level Jaccard is an empirical claim that the paper asserts without demonstrating.

## Remaining Issues

### Issue 1: Abstract Still Overstates CSS Computability
**Severity**: Medium
**Description**: The abstract mentions CSS as a primary contribution and the 67% finding as a CSS result. But the paper's actual 67% finding is a ratio-stability finding using the $f$ proxy — not a CSS result. The CSS formula's dominant component ($s_{\text{seat}}$, weighted 0.5) is marked [TBD] throughout the paper. A reader who reads only the abstract will believe CSS has been computed; a reader who reaches Section 4.4 will discover it has not.

**Recommendation**: Revise the abstract to lead with "ratio stability" and position CSS as the framework toward which the ratio-stability finding is a first step. Something like: "We report that 67% of states exhibit ratio stability at the first bisection level — the geometric foundation of the CSS framework — across the 2000--2010 decade."

### Issue 2: The Proxy Jaccard Does Not Bound the Actual Jaccard
**Severity**: Medium
**Description**: The paper categorises proxy Jaccard stability by $\Delta f$: High ($\Delta f < 0.05$), Medium ($0.05 \leq \Delta f < 0.15$), Low ($\Delta f \geq 0.15$). This is a reasonable heuristic, but the paper does not establish any formal or empirical relationship between $\Delta f$ and the actual Jaccard score. The proxy could be arbitrarily loose: a state with $\Delta f = 0$ (identical first-level split) could have completely different district boundaries if the population within each half reorganised substantially.

For the proxy to be defensible as a proxy, the paper needs either: (a) a theoretical argument that $\Delta f \approx 0 \Rightarrow$ high Jaccard (perhaps via some stability result for the GeoSection bisection), or (b) an empirical validation on at least a few states where actual Jaccard values can be computed. The pseudocode in Section 4.4 implies that the actual computation is feasible from the pipeline output files. Even three or four states with computed Jaccard values would validate the proxy.

**Recommendation**: Add even a minimal empirical check: compute actual Jaccard for 3--5 states with identical $f$ values (e.g., ME, WV, KS) and verify that High proxy Jaccard corresponds to high actual Jaccard. If the pipeline output files are available for these states, this computation should be feasible.

### Issue 3: Threshold Sensitivity Table Reveals a Metric Structure Issue
**Severity**: Low-Medium
**Description**: Table 5 shows that the stable count jumps from 15 to 20 between $\tau = 0.02$ and $\tau = 0.05$ — adding five states — and then from 20 to 23 between $\tau = 0.05$ and $\tau = 0.10$. This asymmetry (5 states in the 0.02--0.05 band vs. 3 states in the 0.05--0.10 band) is interesting: it suggests there is a cluster of states with $\Delta f$ between 0.02 and 0.05 that the paper is implicitly treating as stable at the primary threshold.

The paper argues that the 67% finding is not a threshold artifact because Iowa ($\Delta f = 0.31$) and the other large-magnitude unstable states remain unstable across all thresholds. This is correct for Iowa but misses the point that five states change classification in the 0.02--0.05 range — they are the most threshold-sensitive cases. Which five states are they? Are they the same AL, SC, UT, MS that appear near the boundary in Table 4?

**Recommendation**: Identify the states in each $\Delta f$ band: 0.02--0.05 (5 states that are classified stable at $\tau = 0.05$ but not at $\tau = 0.02$), 0.05--0.10 (3 states), and $\geq 0.10$ (2 states). This would make the threshold sensitivity table substantively interpretable rather than just a count.

## Minor Issues

- The GerryChain null hypothesis suggestion from my R1 review was not addressed. I understand this is a significant undertaking, but even a qualitative argument comparing the 67% finding to what random compact-plan sampling would produce would strengthen the paper's claims.

- The Type I/Type II decomposition is now a well-framed future work item (Section 4.3.3 on Iowa makes the decomposition design explicit), but it is described in three different places in the paper without a unified reference. Consider consolidating all Type I/Type II discussion into a single subsection.

- Proposition 1 ($C_{\text{pop}}$ bound) remains unproved and the constant is still not estimated. This was a Medium issue in R1 and is still unresolved.

## Questions for Authors

1. In Table 4, NC appears in the Medium Jaccard section but with $\Delta f = 0.00$ and label "High" — there seems to be a table placement error. NC should be in the High section if $\Delta f = 0.00$.

2. For the 5 states that change classification between $\tau = 0.02$ and $\tau = 0.05$: are these the same states identified as "borderline" by the Lorenz proxy (near $p^* \approx 0.25$--$0.33$)?

3. Has the paper computed the actual district-level Jaccard for any state? The pipeline output files described in Section 4.4 should contain the assignment data needed.

## Recommendation

The revision is a genuine improvement over Round 1. The Iowa case study, s_seat clarification, and threshold sensitivity table address the major structural concerns. The paper's framing is more honest about what is computed versus projected. The Jaccard proxy needs one more pass to establish its relationship to the actual metric, and the abstract overstates CSS computability. These are fixable. I maintain my Round 1 score of 3/4, with the expectation that a third revision resolving the abstract framing and the proxy validation would merit acceptance.
