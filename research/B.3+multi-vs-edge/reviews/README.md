# Peer Reviews: Multi-Constraint vs Edge-Weighted Optimization

**Paper**: Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals

**Date**: 2026-02-08

**Number of Reviews**: 5

## Review Summary

This directory contains five comprehensive peer reviews from expert perspectives across graph algorithms, optimization, and redistricting domains.

### Reviewers

1. **George Karypis** (University of Minnesota) - METIS creator, graph partitioning algorithms
2. **Cynthia A. Phillips** (Sandia National Labs) - Combinatorial optimization, experimental design
3. **Bruce Hendrickson** (Sandia National Labs) - Theoretical foundations, multilevel methods
4. **William J. Cook** (University of Waterloo) - Exact algorithms, algorithm selection
5. **Moon Duchin** (Rutgers University) - Redistricting, VRA compliance, geometric methods

### Score Distribution

| Reviewer | Score | Recommendation |
|----------|-------|----------------|
| Karypis | 2/4 | Major revision required |
| Phillips | 2/4 | Major revision required |
| Hendrickson | 3/4 | Minor revision required |
| Cook | 3/4 | Minor revision required |
| Duchin | 3/4 | Minor revision required |

**Average**: 2.6/4 - **Major to minor revision required**

## Common Themes Across Reviews

### Strengths (All Reviewers Agree)

1. **Systematic experimental design**: 160 experiments with comprehensive parameter sweeps
2. **Important practical problem**: VRA compliance is policy-relevant
3. **Constraint conflict hypothesis**: Intuitive and testable explanation
4. **Alabama ubvec sweep**: Strongest empirical evidence (Table 4)
5. **Actionable guidance**: Practical recommendations for practitioners

### Major Issues (Consensus Concerns)

#### 1. Multi-Constraint Implementation Concerns (Karypis, Phillips)
- **Karypis**: Target weight specification likely incorrect (Equation 3)
- **Phillips**: Missing METIS parameter details, unclear if single or multiple runs
- **Both**: Re-running experiments with corrected implementation may change conclusions

#### 2. Theoretical Section Problems (Hendrickson, Cook, Karypis)
- **All**: Section 3.1.2 calculation errors (129% "impossibility" claim is confused)
- **Hendrickson**: Lacks rigor—no theorems, proofs, or formal analysis
- **Cook**: "Theoretical Analysis" title overpromises; should be "Intuitive Explanation"
- **Recommendation**: Either add rigorous theory or reframe as informal discussion

#### 3. Statistical Rigor (Phillips, Cook)
- **Phillips**: No significance testing, confidence intervals, or variance analysis
- **Cook**: Unclear if results are single runs or best-of-N
- **Both**: Need multiple runs per configuration with statistical validation
- **Critical**: Is 12.9 pp difference significant or within noise?

#### 4. Unfair Experimental Comparison (Phillips, Karypis)
- **Issue**: 4 multi-constraint configs vs 140 edge-weighted configs
- **Problem**: "Configuration success rate" (47.9% vs 35.0%) is misleading when counts differ 35×
- **Fix**: Focus on state-level success (4/5 vs 3/5) or equalize configuration counts

#### 5. Overgeneralized Claims (Hendrickson, Cook, Duchin)
- **Evidence**: Only METIS, only redistricting, only 2020, only southern states
- **Claims**: "Graph partitioning problems," "parallel computing," "database partitioning"
- **Gap**: No validation beyond redistricting
- **Fix**: Scope claims to match evidence or add experiments in other domains

#### 6. Legal and Geographic Realism (Duchin)
- **VRA standards**: Oversimplified (missing *Gingles* test, Section 2 analysis)
- **Minority definition**: "All non-white" is legally questionable—should analyze specific groups
- **Compactness**: Edge cut is weak proxy; need Polsby-Popper, Reock scores
- **Missing**: No maps or geographic visualizations

### Minor Issues (Multiple Reviewers)

- Missing implementation details and reproducibility specifications (all)
- No bounds analysis or optimality assessment (Cook)
- Narrow experimental scope—only southern states, single year (Duchin, Hendrickson)
- "Configuration success rate" metric is unfair comparison (Phillips, Karypis)
- Missing variance quantification and error bars (Phillips, Cook)
- No comparison to enacted plans or MCMC ensembles (Duchin)

## Reviewer-Specific Concerns

### Karypis (Implementation Expert)
- **Critical**: Verify target weight specification (tpwgts parameter)
- **Missing**: Complete METIS command lines, all parameter flags
- **Question**: Is -ncuts=1 or -ncuts=10? Multiple trials significantly affect multi-constraint
- **Georgia anomaly**: ubvec=1.3 achieves 7 MM (better than 1.5's 5 MM)—contradicts theory

### Phillips (Experimental Rigor)
- **Critical**: Add statistical hypothesis testing (t-tests, p-values, effect sizes)
- **Need**: 10-30 runs per configuration with mean, std dev, confidence intervals
- **Issue**: Without variance data, cannot assess if differences are significant
- **Method**: Use boxplots, error bars, and statistical annotations

### Hendrickson (Theory)
- **Critical**: Section 3 needs complete rewrite or rename
- **Missing**: Theorems, proofs, complexity analysis, approximation bounds
- **Question**: Is constraint conflict METIS-specific or fundamental to multi-constraint?
- **Suggest**: Test k-way mode vs recursive bisection to isolate mechanism

### Cook (Optimality & Bounds)
- **Critical**: Compute upper bounds on minority concentration (what's theoretically achievable?)
- **Missing**: Optimal solutions for small instances (VT, DE) to calibrate heuristic quality
- **Issue**: Algorithm selection framework lacks objective criteria
- **Question**: How to predict which method works without running experiments?

### Duchin (Redistricting Expert)
- **Critical**: Use race-specific populations (Black VAP), not aggregate "minority"
- **Missing**: Polsby-Popper and Reock compactness scores
- **Legal**: Need *Gingles* test discussion, case law citations, functional analysis
- **Essential**: Add maps showing district geography (at least Alabama)
- **Question**: Are 50% districts sufficient for functional electoral opportunity?

## Recommended Revision Strategy

### Essential (Must Address for Acceptance)

1. **Verify multi-constraint implementation** (Karypis concern)
   - Check target weight specification (Equation 3)
   - Provide complete METIS command lines
   - Test with -ncuts=10 for multi-constraint
   - If implementation was wrong, re-run all experiments

2. **Fix or remove theoretical section** (Hendrickson, Cook)
   - Remove Section 3.1.2 calculation errors
   - Rename to "Informal Analysis" or add rigorous proofs
   - Acknowledge theory is METIS-specific, not proven general

3. **Fix unfair comparison** (Phillips, Karypis)
   - Remove "configuration success rate" from abstract/main claims
   - Focus on state-level success (4/5 vs 3/5)
   - Either equalize configuration counts or use different metric

4. **Scope claims carefully** (all reviewers)
   - Change "graph partitioning problems" → "congressional redistricting using METIS"
   - Clearly separate validated findings from conjectures
   - Add limitations section acknowledging narrow scope

### High Priority (Strongly Recommended)

5. **Add statistical rigor** (Phillips, Cook)
   - Run 10+ trials per configuration with different seeds
   - Report mean, std dev, confidence intervals
   - Add hypothesis tests and p-values
   - Include error bars on all figures

6. **Add geographic analysis** (Duchin)
   - Compute Polsby-Popper and Reock scores
   - Include at least one map (Alabama comparison)
   - Compare to enacted plans for context
   - Use race-specific populations (Black VAP)

7. **Add reproducibility details** (all)
   - Complete METIS parameters and command lines
   - Clarify single runs vs multiple trials
   - Provide code and data availability
   - Document computational environment

### Medium Priority (Would Strengthen)

8. **Bounds analysis** (Cook)
   - Solve small instances optimally (VT, DE)
   - Compute upper bounds on minority concentration
   - Report gap-to-bound for both methods

9. **Expand experimental scope** (Hendrickson, Duchin)
   - Test additional census year (2010)
   - Test geographically diverse states (AZ, CA, NM)
   - Test k-way mode to see if RB-specific

10. **Engage with VRA legal standards** (Duchin)
    - Discuss *Gingles* three-prong test
    - Cite relevant case law (*Allen v. Milligan*, etc.)
    - Clarify functional electoral opportunity requirements

## Paths Forward

### Path A: Major Revision → Resubmit to Same Venue
**If**: Implementation is correct and statistical analysis confirms differences are significant
**Timeline**: 3-6 months for revisions
**Focus**: Add statistical rigor, fix theory section, scope claims carefully
**Outcome**: Strong empirical paper for redistricting community

### Path B: Major Revision → Different Venue
**If**: Implementation needs correction and results change substantially
**Timeline**: 6-12 months to re-run experiments and rewrite
**Focus**: Correct implementation, validate findings, reframe conclusions
**Outcome**: Methodological paper on proper multi-constraint usage

### Path C: Partial Acceptance of Reviews
**If**: Authors believe implementation is correct but reviewers are skeptical
**Strategy**: Provide detailed response addressing implementation concerns
**Include**: Complete METIS command lines, validation experiments, ablation studies
**Outcome**: Convince reviewers through transparency and additional evidence

## Suitable Publication Venues

Based on reviewer feedback and paper content:

### Tier 1 (With Major Revisions)
- **SIAM Journal on Scientific Computing** (if bounds analysis added)
- **ACM Transactions on Mathematical Software** (if reproducibility strengthened)
- **Operations Research** (if optimality analysis added)

### Tier 2 (With Minor Revisions)
- **SIAM Conference on Applied and Computational Discrete Algorithms (ACDA)**
- **Algorithm Engineering and Experiments (ALENEX)**
- **SIAM Conference on Data Mining (SDM)** - applications track

### Tier 3 (Interdisciplinary)
- **Political Analysis** (if legal/geographic analysis added)
- **Election Law Journal** (if VRA compliance emphasized)
- **PLOS ONE** (broader audience, less stringent technical requirements)

### Not Recommended
- **SODA, FOCS, STOC**: Insufficient theoretical rigor
- **IPCO**: Lacks exact algorithms and bounds
- **INFORMS**: Needs more OR methodology

## Overall Assessment

The paper presents valuable empirical findings that challenge conventional assumptions about multi-constraint optimization. The constraint conflict hypothesis is interesting and the experimental scope is impressive. However, significant concerns about implementation correctness, statistical rigor, and generalizability must be addressed.

**Key Question**: Is the multi-constraint implementation correct? If yes, this is an important finding. If no, the entire conclusion changes.

**Recommendation**: Authors should carefully review Karypis's implementation concerns first, then decide on revision strategy. If implementation is correct, this is a strong empirical contribution with minor revisions needed. If implementation has issues, major revision is required.

**Bottom Line**: Interesting and potentially important work, but needs significant strengthening before acceptance. All five reviewers see value in the contribution but raise substantive concerns that must be addressed.
