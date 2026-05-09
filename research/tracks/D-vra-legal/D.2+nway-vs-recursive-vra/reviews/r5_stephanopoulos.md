# Review — D.2: N-Way vs. Recursive VRA
**Reviewer**: Nicholas Stephanopoulos (Efficiency gap, partisan symmetry, election law)
**Round**: 1
**Score**: 2/4
**Verdict**: Major Revision

## Summary

D.2 is an empirically careful paper with a clear contribution: the first large-scale comparison of n-way vs. recursive bisection for minority opportunity district formation. The conclusion — that methods are equivalent on average — is well-supported statistically. However, the paper does not engage adequately with what courts and practitioners need to know. The VRA compliance framing is imprecise (see Pildes's concerns), the success metric definition is ambiguous, and the paper does not provide actionable guidance for the contexts where method choice matters (individual states, specific geographic settings). For courts, who need to know whether algorithm choice can determine VRA outcomes, this paper's equivalence conclusion is useful but incomplete.

## Strengths
- 1,760 runs provides genuine statistical power. The p=0.634 and Cohen's d=-0.018 are reliable evidence of national-level equivalence.
- The runtime comparison (n-way 3× faster) is the most immediately actionable finding for computational practitioners.
- The threshold sensitivity analysis is novel: the finding that methods prefer different threshold levels has implications for best-practice parameter selection.

## Concerns
- **Legal operationalization gap**: The paper's "proportional MM district target" is not a legal standard. Courts need to know whether the choice of algorithm can change whether VRA Section 2 is satisfied or violated — specifically, whether one method is more likely to produce the legally required number of majority-minority districts given known minority geography. The paper should reframe its success metric as "geometric opportunity for Gingles prong-1 compliance" and explicitly state what additional evidence courts need (prongs 2 and 3) before concluding VRA compliance.
- **Baseline comparison**: The paper compares n-way and recursive bisection against each other, but not against the enacted plan. The question courts care about is not "n-way vs. recursive" but "can algorithmic redistricting (of either type) produce more majority-minority districts than the enacted plan?" A comparison to enacted plans would dramatically increase the paper's legal utility.
- **Judicial admissibility**: The paper does not address whether testimony based on either method's VRA success rate would be judicially admissible in Section 2 litigation. This question is distinct from statistical equivalence — a court might prefer one method's theoretical properties even if the empirical outputs are equivalent.

## Required Changes (P1/P2)
- **P1**: Replace "VRA compliance" throughout with "geographic VRA feasibility" or "Gingles prong-1 opportunity" and add a disclaimer in the introduction about the need for additional evidence (prongs 2 and 3) for legal VRA compliance.
- **P1**: Add a comparison of both methods' outputs to enacted plans across the 44 states: does algorithmic redistricting (of either type) produce more majority-minority districts than the enacted plan, and if so, by how much?
- **P2**: Add a brief discussion of judicial admissibility considerations: whether courts have accepted either method's outputs as VRA evidence, and what expert-witness standards apply.
- **P2**: Provide a brief practical decision tree for practitioners: "If you care about national aggregate, use n-way (faster, equivalent outcome). If your state has high method sensitivity (check Table X), use the preferred method for your state."
