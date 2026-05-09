# Review — D.2: N-Way vs. Recursive VRA
**Reviewer**: Richard Pildes (Election law, VRA doctrine)
**Round**: 1
**Score**: 2/4
**Verdict**: Major Revision

## Summary

D.2's empirical contribution — a large-scale ablation study showing n-way and recursive bisection achieve equivalent VRA compliance success rates — is valuable. However, the paper systematically conflates its own "proportional MM district target" metric with VRA compliance. This conflation is a fundamental legal framing error. VRA Section 2 compliance requires satisfying the three Gingles preconditions (sufficiently large and compact minority group; political cohesion; racially polarized voting). The paper's success metric addresses only prong 1 (geographic compactness enabling majority-minority district formation). A 47.5% or 48.3% "success rate" tells us how often the algorithm can form districts with minority VAP ≥ threshold — but not whether those districts satisfy Gingles prong 2 or 3, and therefore not whether they provide VRA compliance in the legal sense.

## Strengths
- The scale of the study (1,760 runs) is impressive and provides reliable national-scale evidence.
- The equivalence finding is useful for courts and practitioners who need to know whether algorithm choice affects VRA outcomes.
- The speed difference (n-way 3× faster) is a practical operational finding.
- State-level variation analysis (VA, CT, GA, NC highlighted) is appropriately granular.

## Concerns
- **Legal framing error (major)**: Throughout the paper, "VRA compliance" is used to describe what is actually "successful formation of districts with minority VAP ≥ threshold." These are not the same. Gingles prong 1 requires compact minority communities capable of forming a majority-minority district — but Gingles prongs 2 and 3 require political cohesion and racially polarized voting evidence that no algorithm can assess from population data alone. The paper should use "geographic VRA feasibility" or "Gingles-prong-1 feasibility" throughout, not "VRA compliance."
- **50% threshold vs. legal standard**: The paper varies thresholds from 40%–55%, but VRA majority-minority districts typically require minority VAP ≥ 50%. Using 40% configurations in the success calculation inflates the apparent success rate and may mislead practitioners about actual VRA opportunity district feasibility.
- **No legal guidance on method selection**: The conclusion that "practitioners can confidently choose either method based on computational constraints" is correct for national averages but does not help practitioners in specific states (e.g., Virginia, Connecticut) where method choice matters substantially. The paper should provide state-specific guidance.

## Required Changes (P1/P2)
- **P1**: Replace "VRA compliance" with "Gingles prong-1 geographic feasibility" (or equivalent) throughout. Add a disclaimer in the abstract and introduction that the paper's success metric addresses only the geographic component of VRA Section 2 analysis.
- **P1**: Report results separately for the 50% threshold (the legally standard majority-minority threshold) vs. aggregate across all thresholds including sub-50%. The legal standard is 50%; the aggregate is a methodological convenience.
- **P2**: Add a subsection in the Discussion on limitations: this study cannot evaluate Gingles prongs 2 and 3, and practitioner VRA analysis must incorporate racially polarized voting evidence from election results.
- **P2**: Provide state-specific guidance for the 8–10 states where method choice produces materially different outcomes (>10pp success rate difference).
