# Review: Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals

**Reviewer**: George Karypis (University of Minnesota)
**Expertise**: Graph partitioning algorithms, METIS development, multi-constraint optimization
**Date**: 2026-02-08

## Overall Assessment

This paper presents an important empirical study comparing multi-constraint and edge-weighted formulations for congressional redistricting under VRA requirements. The authors conduct 160 experiments across five states and demonstrate that edge-weighted single-objective optimization consistently outperforms multi-constraint methods (47.9% vs 35.0% success rate). As the developer of METIS, I find the results both surprising and concerning. The paper correctly identifies that constraint conflict limits multi-constraint effectiveness when tolerances differ by 100×, and provides systematic evidence through ubvec sweeps.

However, the paper has significant technical issues in its multi-constraint implementation and theoretical analysis. The authors claim multi-constraint "fails" for VRA compliance, but I suspect the failure stems from improper parameter selection rather than fundamental algorithmic limitations. The theoretical section contains calculation errors and oversimplifications about how METIS handles constraints. The experimental design is thorough but lacks critical implementation details needed for reproducibility. Despite these concerns, the empirical findings are valuable and the constraint conflict hypothesis warrants further investigation with corrected methodology.

## Score: 2/4

**My score**: 2/4 - Major revision required. The empirical results are interesting but the multi-constraint implementation appears flawed, the theory contains errors, and critical METIS parameter details are missing.

## Major Strengths

1. **Systematic Experimental Design**: The paper conducts 160 experiments with comprehensive parameter sweeps (ubvec from 1.3 to 5.0 for multi-constraint; 7 weight factors × 4 thresholds for edge-weighted). This is thorough empirical work.

2. **Constraint Conflict Hypothesis**: The central idea that tight/loose constraint combinations create conflict is insightful and potentially generalizable beyond redistricting. The ubvec sweep specifically tests this hypothesis.

3. **Practical Impact**: For the redistricting domain, the findings are immediately actionable. The 12.9 percentage point success rate gap is substantial and policy-relevant.

## Major Issues (Must Address)

### Issue 1: Incorrect Multi-Constraint Implementation
**Severity**: High
**Description**: The paper's multi-constraint formulation appears fundamentally flawed. The authors use target weights that specify 60% minority concentration for MM districts, but this creates an impossible optimization problem. They write target weights as:

```
t_i^{min} = 0.60 * M_total/k  (for MM districts)
```

This means they're asking METIS to place 60% of the state's *total* minority population into each MM district. For Alabama with 2 target MM districts, this requires 120% of total minority population (0.60 × 2 = 1.20), which is mathematically impossible.

**Correct formulation**: Target weights should specify the *fraction* of minority population going to each partition, not the *concentration* in each partition. If you want 60% minority concentration in district i, you need to solve backwards from the population balance constraint:

```
Target minority weight for district i = (desired_concentration) × (population_weight_i)
```

This is a fundamental misunderstanding of how METIS's tpwgts parameter works.

**Recommendation**:
1. Review METIS documentation for tpwgts specification
2. Recalculate target weights correctly based on desired concentration
3. Re-run all multi-constraint experiments with corrected targets
4. If this fixes the problem, the entire paper's conclusion changes

### Issue 2: Theoretical Analysis Contains Calculation Errors
**Severity**: High
**Description**: Section 3.1.2 attempts to prove Alabama's VRA targets are "impossible" but contains multiple calculation errors and confused reasoning. The authors repeatedly recalculate the same quantity, arrive at 129% (which they claim is impossible), but this calculation is wrong. They mix total population and VAP, and conflate concentration targets with weight targets.

The corrected calculation is simpler: Alabama has 36.9% minority VAP. If all minority VAP were concentrated into 2 of 7 equal-population districts, maximum concentration would be:
```
(36.9% × 7) / 2 = 129% of average district VAP
```

This is NOT impossible—it means each of those 2 districts would have 1.29× the average minority population, which is perfectly feasible. The confusion arises from treating percentages as absolute constraints rather than as distributions of VAP across districts.

**Recommendation**:
1. Remove or completely rewrite Section 3.1.2
2. Focus theoretical analysis on constraint tolerance mismatch, not on "impossibility"
3. The constraint conflict argument stands without claiming targets are impossible—loose constraints simply provide insufficient guidance

### Issue 3: Missing Critical METIS Parameters
**Severity**: Medium
**Description**: Section 4.2.4 lists METIS parameters but omits several critical settings that dramatically affect multi-constraint performance:

- **-contig**: Is contiguity enforcement enabled? This is crucial for redistricting
- **-minconn**: Minimum connectivity requirement?
- **-ccorder**: Connected component ordering?
- **-ncuts**: Number of different partitionings to compute (default 1)?
- **-no2hop**: Two-hop matching disabled?

Most importantly: multi-constraint partitioning often benefits from repeated runs with different random seeds. Did you run multiple trials per configuration and report the best? Or is each result from a single METIS call?

Additionally, for multi-constraint: are you using -ufactor in addition to ubvec? These interact in complex ways.

**Recommendation**:
1. Provide complete METIS command lines used (both methods)
2. Clarify whether results are single runs or best-of-N
3. Test sensitivity to -ncuts parameter (I recommend -ncuts=10 for multi-constraint)
4. Report full parameter specifications in supplementary materials

### Issue 4: Unfair Comparison - Asymmetric Parameter Tuning
**Severity**: Medium
**Description**: The paper tests 4 multi-constraint configurations vs 140 edge-weighted configurations (7 weight factors × 4 thresholds × 5 states). This is not a fair comparison. Edge-weighted gets 35× more parameter exploration, dramatically increasing its chances of finding good solutions.

The proper comparison would be:
- Option A: Same number of configurations for both (e.g., 28 each)
- Option B: Report results for "best configuration found" rather than "configuration success rate"
- Option C: Use automated parameter tuning (hyperparameter optimization) for both methods

The 47.9% vs 35.0% success rate comparison is misleading because edge-weighted has 35× more chances to succeed.

**Recommendation**:
1. Re-analyze data reporting "state success rate" only (4/5 vs 3/5), which is fair
2. Compare best configuration per state head-to-head (this is done in Table 3, which is good)
3. Either reduce edge-weighted configurations to 4 per state, or increase multi-constraint configurations to ~28 per state
4. Consider Bayesian optimization to find optimal parameters fairly for both methods

## Minor Issues

- **Section 2.3**: The description of edge-weighted approach implies edge weights are binary (α or 1). But did you test non-binary weights? E.g., continuous weights proportional to min(m_u/p_u, m_v/p_v)?

- **Figure labels**: Several figures lack error bars or confidence intervals despite stochastic algorithm. Are results from single runs?

- **Compactness metric**: Edge cut is a weak proxy for compactness. You acknowledge this in Section 6.2 but should include Polsby-Popper scores in results, not defer to future work.

- **Table 1**: "Minority %" column is ambiguous. Is this minority percentage of total population, VAP, or CVAP? Clarify in caption.

- **Georgia anomaly**: Georgia multi-constraint achieves 7 MM districts (above target of 5) with ubvec=1.3, but only 5 with ubvec=1.5. This suggests the constraint is actually helping. This contradicts the "loose constraints are ineffective" theory. Needs explanation.

## Detailed Comments

### Section-by-Section

**Introduction**: Clear motivation and strong empirical preview. However, the claim that multi-constraint "fails" is too strong—it achieves targets in 3/5 states.

**Background**: Good review of METIS and multi-constraint formulation. Equation 2 correctly specifies balance constraints. However, the target weight specification (Equation 3) appears incorrect as discussed in Major Issue 1.

**Theory**: Section 3.1 has the right intuition (tight constraints dominate) but the mathematical analysis in 3.1.2 is confused. Section 3.2 correctly explains why edge-weighting avoids constraint conflict. Section 3.3's prediction is validated, which is valuable.

**Experiments**: Thorough design but missing critical implementation details (Major Issue 3). The asymmetric configuration count (Issue 4) undermines the fairness of comparison.

**Results**: Well-presented with clear tables and figures. The Alabama constraint conflict test (Table 4) is the paper's strongest empirical result. However, the Georgia anomaly (ubvec=1.3 achieves 7 MM, better than 1.5's 5 MM) contradicts the theory.

**Discussion**: Section 6.1's explanation of constraint conflict is good. Section 6.2's algorithm selection guidance is useful but overgeneralized—the guidance assumes METIS-specific behavior may not transfer to other partitioners.

### Figures and Tables

**Figure 1**: Clear success rate comparison. But error bars needed—are these single runs?

**Table 2**: Excellent head-to-head comparison. This is the paper's most convincing evidence (edge-weighted wins 4/5 states).

**Figure 3 & Table 4**: The constraint conflict validation is excellent. This is the most important result in the paper.

**Figure 5**: Parameter sensitivity is thorough but confirms my concern about multi-constraint implementation. The "Goldilocks zone" at ubvec=1.5 suggests the problem is parameter selection, not fundamental algorithmic failure.

## Questions for Authors

1. **Target weight specification**: Can you provide the exact tpwgts arrays passed to METIS for Alabama's multi-constraint experiments? I suspect they are incorrectly specified.

2. **Multiple runs**: Are results from single METIS calls or best-of-N? Multi-constraint performance typically improves significantly with -ncuts=10.

3. **Georgia anomaly**: Why does ubvec=1.3 achieve 7 MM districts while ubvec=1.5 achieves only 5? This contradicts the theory that tighter constraints perform worse.

4. **Implementation availability**: Is your METIS wrapper code publicly available? I would like to verify the implementation details.

5. **Alternative formulations**: Did you consider using continuous target weights (not just 60% for MM districts) that better match the actual minority distribution?

6. **Contiguity**: How is contiguity enforced? METIS's -contig flag or post-processing? This matters for multi-constraint effectiveness.

## Recommendation

**Major revision required**. The empirical findings are potentially important, but I have serious concerns about the multi-constraint implementation (Issue 1) and the fairness of the comparison (Issue 4). The authors should:

1. Verify and correct the multi-constraint target weight specification
2. Re-run experiments with corrected implementation
3. Provide complete METIS command lines and parameter specifications
4. Address the asymmetric configuration count issue
5. Fix or remove the erroneous theoretical calculations in Section 3.1.2

If the multi-constraint implementation is corrected and results still favor edge-weighted, this would be a strong paper. However, if the implementation fix resolves the performance gap, the paper's conclusion would need fundamental revision. I cannot recommend acceptance until these issues are resolved.

The constraint conflict hypothesis is interesting and worth publishing, but only after confirming it's not an artifact of implementation errors.
