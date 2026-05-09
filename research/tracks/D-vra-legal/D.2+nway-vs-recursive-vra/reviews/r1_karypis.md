# Review — D.2: N-Way vs. Recursive VRA
**Reviewer**: George Karypis (Graph partitioning, METIS)
**Round**: 1
**Score**: 3/4
**Verdict**: Minor Revision

## Summary

D.2 presents a rigorous 1,760-run empirical comparison of n-way vs. recursive bisection for VRA compliance. The paired t-test (p=0.634, Cohen's d=-0.018) strongly supports the equivalence finding, and the 880-run scale is large enough to detect meaningful differences if they existed. The methodology is sound. The main gap is the definition of "success" — the paper uses its own "proportional MM district target" without fully grounding this in the Gingles preconditions or the legal VRA standard.

## Strengths
- 1,760 total runs across 44 states and 20 parameter configurations per method — this is the largest VRA ablation study in the redistricting literature.
- Statistical methodology is correct: paired t-test exploits within-state correlation; Cohen's d correctly calculated.
- Practical finding is useful: n-way is 3× faster at equivalent VRA performance, providing a real operational recommendation.
- State-level variation table (n-way +35% in VA, recursive +45% in CT) is appropriately nuanced.

## Concerns
- **Success metric definition**: The abstract and abstract say "fraction achieving proportional MM district target." What is the proportional target? Is it that minority share ≥ 50% in the required number of districts (as determined by minority population share)? Or is it an absolute count from some baseline? Section 4 should define this unambiguously in a Definition box. Without it, the 47.5%/48.3% figures are uninterpretable.
- **Multiple-threshold comparison**: The paper varies minority thresholds from 40%–55%, but the VRA legal standard is 50% (majority-minority). Configurations with threshold <50% may not constitute VRA-compliant districts. The paper should clarify whether the 47.5%/48.3% success rates are for the 50% threshold only or aggregated across all four thresholds including sub-50%.
- **State-specific dominance**: Virginia (+35% n-way advantage) and Connecticut (+45% recursive advantage) suggest that for individual states, method choice matters substantially. The equivalence finding holds on average but practitioners choosing for specific states should not rely on the national aggregate.

## Required Changes (P1/P2)
- **P1**: Define "proportional MM district target" precisely in §3 (Methodology): state the exact rule used to determine target MM district count per state per configuration.
- **P1**: Report success rates separately for the 50% legal threshold vs. aggregate across all thresholds. The aggregate may dilute the legally relevant signal.
- **P2**: Add a state-specific recommendation table: for which states does method choice matter substantially (>10pp difference), and what parameter settings are recommended in those states?
