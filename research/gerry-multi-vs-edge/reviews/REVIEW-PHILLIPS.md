# Review: Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals

**Reviewer**: Cynthia A. Phillips (Sandia National Labs)
**Expertise**: Combinatorial optimization, approximation algorithms, experimental algorithm design
**Date**: 2026-02-08

## Overall Assessment

This paper addresses an important practical question—how to algorithmically create majority-minority districts—through a systematic experimental comparison of two graph partitioning formulations. The core finding (edge-weighted outperforms multi-constraint by 12.9 percentage points) is significant for the redistricting community, and the constraint conflict explanation is intuitive. The experimental scope is impressive: 160 experiments across five states with thorough parameter sweeps.

However, as someone who specializes in rigorous experimental algorithm analysis, I find this paper's methodology concerning. The comparison is statistically informal—no hypothesis tests, confidence intervals, or significance testing. The asymmetric configuration counts (4 vs 140) make "success rate" comparisons misleading. The paper claims to provide "theoretical analysis" but offers only informal reasoning without proofs, approximation bounds, or complexity analysis. The experimental design lacks reproducibility details: single runs or multiple trials? What about variance across random seeds? These gaps prevent me from confidently endorsing the claimed performance differences.

That said, the constraint conflict hypothesis is interesting and the Alabama ubvec sweep provides compelling evidence. With rigorous statistical analysis, controlled experimental design, and honest acknowledgment of limitations, this could be a strong empirical paper.

## Score: 2/4

**My score**: 2/4 - Major revision required. Interesting findings undermined by lack of statistical rigor, unfair experimental comparison, and overstated theoretical claims.

## Major Strengths

1. **Large-Scale Systematic Study**: 160 experiments with comprehensive parameter coverage is impressive. The scope demonstrates commitment to thorough empirical evaluation.

2. **Direct Practical Impact**: The redistricting community needs this guidance. The 12.9 percentage point gap, if validated statistically, would justify changing algorithmic practice.

3. **Hypothesis-Driven Experimentation**: The paper formulates a testable hypothesis (constraint conflict) and designs an experiment (ubvec sweep) specifically to validate it. This is good scientific method.

## Major Issues (Must Address)

### Issue 1: No Statistical Significance Testing
**Severity**: High
**Description**: The paper reports performance differences (47.9% vs 35.0%, 63.7% vs 56.7%) without any statistical testing. Key questions remain unanswered:

- Are differences statistically significant?
- What is the variance across random seeds?
- What confidence intervals apply to success rates?
- What is the power of your experimental design?

For combinatorial optimization papers, we expect:
- Multiple runs per configuration (typically 10-30)
- Mean and standard deviation reported
- Statistical tests (t-tests, Mann-Whitney U, etc.) for claimed differences
- Effect sizes and confidence intervals

The paper reports single-number results suggesting single runs, which is insufficient for stochastic algorithms like METIS.

**Recommendation**:
1. Run each configuration 10+ times with different random seeds
2. Report mean, median, std dev, and quartiles for all metrics
3. Apply appropriate statistical tests (e.g., Welch's t-test for independent samples)
4. Report p-values and effect sizes (Cohen's d)
5. Add error bars to all figures
6. Use boxplots instead of bar charts to show distributions

Without this, I cannot assess whether the 12.9 pp difference is real or within noise.

### Issue 2: Unfair Experimental Comparison (Asymmetric Configuration Counts)
**Severity**: High
**Description**: The paper compares 4 multi-constraint configurations vs 140 edge-weighted configurations and reports "configuration success rate" (47.9% vs 35.0%). This is methodologically invalid for several reasons:

**Statistical issue**: More configurations mean more chances to succeed. If you flip a coin 4 times vs 140 times, the 140-flip sequence will hit "heads" more often. Similarly, testing 35× more edge-weighted configurations inflates its success rate.

**Fair comparison options**:
- **Option A**: Test equal numbers of configurations (4 each or 28 each)
- **Option B**: Compare only "best configuration found per state" (already done in Table 2)
- **Option C**: Use performance profiles showing success rate as function of parameter distance from optimal

The current "configuration success rate" metric is meaningless when configuration counts differ by 35×.

**Recommendation**:
1. Remove "configuration success rate" from abstract and main claims
2. Focus on "state success rate" (4/5 vs 3/5) which is fair
3. Present head-to-head best-configuration comparisons (Table 2) as primary evidence
4. If you want to compare parameter sensitivity, use a normalized metric like "fraction of parameter space achieving success"
5. Alternatively: sample 4 random configurations from edge-weighted space to match multi-constraint's 4, repeat 100 times, report distribution of success rates

### Issue 3: "Theoretical Analysis" Section Lacks Rigor
**Severity**: Medium
**Description**: Section 3 is titled "Theoretical Analysis" but provides only informal reasoning. It contains:

- No theorems, lemmas, or propositions
- No proofs or approximation bounds
- Calculation errors (Section 3.1.2, as other reviewers likely noted)
- No complexity analysis
- No worst-case or expected-case guarantees

The "constraint conflict" explanation is intuitive but not formalized. What we'd expect in a theory section:

**Theorem**: For multi-constraint partitioning with constraints of tolerance $\epsilon_1 < \epsilon_2$, the probability that local search moves violate constraint 1 is at least $f(\epsilon_1, \epsilon_2)$ higher than moves violating constraint 2.

**Proof sketch**: [formalize the local search decision process]

Instead, the section provides informal reasoning like "METIS prefers tight constraints." This may be true empirically, but it's not theory.

**Recommendation**:
1. Rename section to "Constraint Conflict Explanation" or "Informal Analysis"
2. Remove claims of "theoretical proof" or "theoretical prediction"
3. If you want a theory section, collaborate with a theorist to formalize the constraint conflict model and prove bounds
4. Alternatively: frame this as an empirical study with hypothesis testing, which is perfectly valid

### Issue 4: Missing Reproducibility and Implementation Details
**Severity**: Medium
**Description**: The paper lacks details needed to reproduce results:

**METIS details missing**:
- Complete command lines (with all flags)
- Single run or multiple trials per configuration?
- How were random seeds set (-seed=42 mentioned but was this varied)?
- Which METIS version? (5.0.2? 5.1.0? matters for results)
- Was graph pre-processed (e.g., sorted, compressed)?

**Data details missing**:
- Census tract file format and source
- Adjacency construction method (rook? queen? distance threshold?)
- How is VAP computed from census data?
- What counts as "minority" (all non-white? specific groups?)

**Code availability**:
- The paper says "code available at [ANONYMIZED]" but provides no details
- What language? Python? C++?
- Dependencies and versions?

For reproducibility, I should be able to download your code and data and get identical results (within stochastic variance). Currently, this is not possible.

**Recommendation**:
1. Provide complete METIS command lines in supplementary materials
2. Clarify: single runs or multiple trials? If multiple, how many?
3. Specify METIS version, OS, hardware
4. Describe data processing pipeline (census tracts → graph → METIS input)
5. Make code and data publicly available (after de-anonymization)
6. Include a "Reproducibility Checklist" section per ACM/SIAM guidelines

## Minor Issues

- **Page 2**: "12.9 percentage point gap" vs "12.9 pp" vs "+12.9 pp" - be consistent with notation

- **Table 1**: Sample sizes differ dramatically (Alabama n=7 districts vs Georgia n=14). This affects statistical power. Consider stratifying analysis by sample size.

- **Section 4.2.4**: "Random seed: Fixed for reproducibility" - this is good, but you should also report variance across different seeds. Fixing seed tests only one point in algorithm space.

- **Figure 2**: "Edge cut vs maximum minority percentage" - this is a Pareto frontier plot. Explicitly state this and discuss Pareto dominance. Some edge-weighted points Pareto-dominate multi-constraint (higher minority, lower cut).

- **Section 5.3**: "non-monotonic behavior" - this suggests optimization landscape is rugged. Have you considered adaptive parameter selection or meta-heuristics for tuning ubvec?

- **Section 6.1.2**: "Goldilocks zone" - informal language in a technical paper. Say "optimal parameter range" instead.

- **Missing baseline**: What about naive approaches (random partitioning, greedy, etc.)? How much better is METIS than simpler methods?

- **Runtime**: Table in CSV shows runtimes of 4-8 seconds. This is mentioned nowhere in the paper. Include runtime analysis in results.

## Detailed Comments

### Section-by-Section

**Introduction**: Strong motivation and clear contributions. However, Contribution 2 claims "theoretical explanation" which overstates the informal analysis in Section 3.

**Background**: Clear explanation of both methods. Equations 1-6 are correctly specified (though Equation 3 may have implementation issues, per other reviewers).

**Theory**: As discussed in Major Issue 3, this section lacks rigor for a "theory" section. The intuition is good but formalization is absent. The calculations in 3.1.2 are confused and should be removed or corrected.

**Experiments**: The design is comprehensive but lacks statistical planning. How were sample sizes determined? What power analysis was conducted? What effect size would be meaningful? These should be specified a priori.

**Results**: Well-organized with good visualizations. However, all results appear to be single runs without variance quantification. Figures lack error bars. The Alabama constraint conflict test (Figure 3, Table 4) is the strongest evidence, but even here we need to see variance across seeds.

**Discussion**: Section 6.2's algorithm selection guidance is useful but overgeneralized. The guidance is specific to METIS and may not apply to other partitioners (ParMETIS, KaHIP, etc.). Be more careful about scope of claims.

### Figures and Tables

**Table 2**: This is your strongest evidence. Head-to-head best-configuration comparison shows edge-weighted wins 4/5 states. However, "best" based on single run or multiple? Need clarification.

**Figure 3**: The constraint conflict visualization is excellent. This is the most important result. But: are these four points based on single runs each? Show confidence intervals.

**Figure 5**: Parameter sensitivity heatmaps are informative but would benefit from statistical annotations (e.g., significance stars for differences).

**Table 4**: Critical data. But with only one data point per ubvec value, we can't assess whether differences are significant or noise.

## Questions for Authors

1. **Multiple runs**: How many times did you run each configuration? If once, please re-run 10-30 times and report statistics. If multiple times, why are statistics not reported?

2. **Statistical testing**: Have you performed any hypothesis tests on the differences? What are the p-values?

3. **Variance**: What is the variance in success rate across random seeds for each method?

4. **Power analysis**: Given your sample sizes (5 states, 4-140 configurations), what effect size can you reliably detect?

5. **Sensitivity to random seed**: Did you test sensitivity to the -seed parameter? Does fixing seed=42 bias results?

6. **Parameter selection**: For operational use, how would practitioners select edge-weighted parameters? Is there a principled method or just trial-and-error?

7. **Scalability**: Do results generalize to states with more districts (e.g., California k=52, Texas k=38)?

## Recommendation

**Major revision required**. The core findings are potentially valuable, but the experimental methodology is insufficiently rigorous for a computational optimization paper. Required changes:

1. **Add statistical rigor** (Issue 1): Multiple runs, significance tests, confidence intervals
2. **Fix unfair comparison** (Issue 2): Drop "configuration success rate" or equalize configuration counts
3. **Reframe theory section** (Issue 3): Rename to "informal analysis" or add real theorems with proofs
4. **Add reproducibility details** (Issue 4): Complete method specifications, code/data release

After these revisions, the paper could make a solid contribution to the redistricting literature. The constraint conflict hypothesis is interesting and worth publishing, but only after proper statistical validation.

I'm particularly concerned about Issue 1. If variance across random seeds is high, the claimed performance differences may not be significant. This is the most critical fix.

For a conference publication, I would conditionally accept if authors add statistical analysis and fix the comparison fairness issue. For a journal publication, I would require all four major issues to be addressed plus minor issues.

The redistricting application is important, and practitioners need this guidance. But the guidance must be statistically sound to be actionable. Please strengthen the experimental rigor and resubmit.
