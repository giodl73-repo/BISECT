# Round 2 Review: Measuring Partisan Fairness in Algorithmic Redistricting

**Reviewer**: Nicholas O. Stephanopoulos (Harvard Law School)
**Expertise**: Efficiency gap metric, partisan gerrymandering law
**Round**: 2 (Revision review)
**Date**: 2026-02-08

---

## Overall Assessment

This revised manuscript represents a dramatic improvement over the initial submission. The authors have systematically addressed all blocking issues I identified, transforming what was a methodologically sound but legally and theoretically incomplete analysis into a comprehensive, nuanced treatment of algorithmic redistricting and partisan fairness. The additions—particularly the efficiency gap limitations discussion (Section 2.3), the proportionality-EG connection (Section 4.6), and the multiple metrics comparison (Section 4.8)—demonstrate deep engagement with both the legal skepticism surrounding efficiency gap and the broader partisan fairness literature.

**Most impressive improvements:**

1. **Legal framing correction (Section 2.3)**: The new subsection on efficiency gap utility and limitations directly confronts *Gill* and *Rucho* skepticism while defending EG's value for comparative analysis. The reframing from "constitutional threshold" to "empirical baseline for geographic neutrality" is exactly right. This section will be invaluable for state courts evaluating gerrymandering claims under state constitutions.

2. **Proportionality integration (Section 4.6)**: The mathematical connection between efficiency gap and proportionality (EG ≈ 2×[seat share - vote share]) was missing from the initial draft. The revised treatment shows that the -3.2% algorithmic EG predicts ~54% Democratic seats with 51.3% votes, matching empirical findings within 1 percentage point. This validates EG as a robust partisan fairness measure.

3. **Multiple metrics robustness check (Section 4.8)**: Addressing my concern about over-reliance on a single metric, the authors now report five partisan fairness measures (EG, mean-median, partisan bias @50%, declination, elasticity). All converge on consistent conclusions: algorithmic plans show 2-3 pp Democratic advantage, enacted plans show 5-6 pp Republican advantage. The high cross-metric correlations (r=0.89-0.94) eliminate concerns about metric-specific artifacts.

The paper now provides the strongest empirical case yet for algorithmic redistricting as a tool to reduce partisan gerrymandering while acknowledging—rather than dismissing—the limitations of both algorithms and fairness metrics.

## Scoring

**Score**: 4.0/4 (Strong Accept)

**Score Justification**: All P1 blocking issues resolved. Legal framing accurate and nuanced. Multiple metrics demonstrate robustness. Proportionality analysis integrated. Methodologically transparent. This will be a landmark APSR paper.

---

## Detailed Assessment

### 1. Efficiency Gap Utility and Limitations (NEW Section 2.3)

**Strength**: This section is exemplary in its treatment of Supreme Court skepticism while defending EG's comparative value. The distinction between EG as "constitutional threshold" (rejected by *Rucho*) versus "empirical baseline for geographic neutrality" (appropriate use) is precisely the reframing courts and scholars need.

Key passages that work well:

- Roberts's concerns from *Rucho* (paraphrased accurately): no clear threshold, temporal sensitivity, incomplete measure, uniform swing assumption
- Methodological safeguards: multiple metrics, temporal stability testing, state-specific baselines (not universal thresholds)
- Post-*Rucho* state constitutional litigation context: PA, NC, FL striking down gerrymanders under state law, with EG as relevant (though not dispositive) evidence

**Minor suggestion**: Consider adding a sentence explicitly stating that your state-specific algorithmic baselines (e.g., Pennsylvania: -2.8% algorithmic vs +7.5% enacted) provide courts with quantitative tools for detecting manipulation *beyond* what geography alone would produce. This bridges the gap between descriptive findings and legal application.

**Impact on initial concerns**: Fully resolves my P1.3 issue. The 7% threshold is now correctly described as a "durability threshold" (EG above 7% tends to persist across elections) rather than a constitutional bright line. The paper no longer risks misleading courts or reformers about legal standards.

### 2. Proportionality-EG Mathematical Connection (NEW Section 4.6)

**Strength**: The derivation of EG ≈ 2×(seat share - vote share) provides the theoretical foundation that was missing. The validation is compelling: -3.2% EG predicts Democrats winning 54% seats with 51.3% votes, versus observed 54.1%—within 1 pp.

The conceptual distinction between partisan symmetry (efficiency gap) and majoritarian representation (proportionality) is helpful. The example showing they can diverge (52% votes → 48% seats with perfect symmetry but imperfect proportionality) clarifies that EG measures fairness, not proportionality per se.

**Connection to mean-median difference**: The explanation of how packing (high MMD) interacts with efficiency gap is valuable. Enacted plans show +4.1 pp mean-median (severe Democratic packing) yet +5.1% EG (Republican advantage), revealing that enacted maps use *both* packing *and* cracking—a sophisticated gerrymandering signature.

**Minor suggestion**: The approximation breakdown conditions (vote shares near 50%, uniform swing, similar turnout) are clearly stated. Consider adding a brief discussion of when these assumptions fail most severely (e.g., landslide states like Wyoming or Vermont where EG becomes less meaningful). This would strengthen the argument that your 15-state competitive sample is appropriate for EG analysis.

**Impact on initial concerns**: Fully resolves my P1.4 issue. Proportionality analysis is no longer a disconnected appendix but an integrated component demonstrating that EG, proportionality, and mean-median all measure related but distinct fairness dimensions.

### 3. Multiple Metrics Comparison (NEW Section 4.8)

**Strength**: This is exactly what the paper needed. Reporting five metrics with convergent conclusions (Table 5) demonstrates that findings are not EG-specific artifacts. The cross-metric correlations (EG × partisan bias: r=0.94; EG × declination: r=0.89) show these measures capture the same underlying partisan asymmetry through different approaches.

The discussion of responsiveness (elasticity) as a separate dimension is valuable. Algorithmic plans maintain high responsiveness (2.8) while enacted plans dampen it (2.1), indicating enacted boundaries insulate incumbents from electoral swings. This finding addresses a distinct gerrymandering pathology beyond bias.

**Minor gap**: The five metrics are EG, mean-median, partisan bias @50%, declination, and elasticity. Consider briefly noting why you *don't* include other established metrics like lopsided wins test or seats-votes curve symmetry deviation. (Answer: you do analyze seats-votes curves in Section 4.6.2, so this is partially addressed, but explicit acknowledgment would be helpful.)

**Impact on initial concerns**: Partially resolves my original concern about over-reliance on EG (which was a P2-level issue, not P1). The robustness demonstration is convincing. However, I note that all five metrics still rely on district-level election results—none directly measure geographic compactness or communities of interest. This is appropriate given your research question (partisan fairness), but worth acknowledging as a scope limitation.

### 4. Algorithmic Transparency (NEW Section 3.4)

**Strength**: Complete METIS parameterization (nparts=2, niter=100, ufactor=10, objtype='cut') enables replication. The sensitivity analysis—testing alternative algorithms (k-means, shortest splitline, Voronoi) and edge weight variations—shows that -3.2% baseline is robust (±0.4 pp) across neutral approaches.

Ensemble generation (100 maps × 5 states, std dev ~0.3%) quantifies uncertainty, demonstrating that within-state variation is tiny compared to the algorithmic-enacted gap (8.3 pp).

**Impact on Chen's concerns**: Fully resolves methodological transparency issues.

### 5. VRA Compliance Analysis (NEW Section 4.7)

**Strength**: The finding that algorithmic plans create 137 majority-minority districts versus 68 enacted (+101% increase) is striking and politically significant. It refutes the common concern that partisan fairness improvements necessarily dilute minority representation.

The distinction between opportunity districts (first *Gingles* prong) and performing districts (requires prongs 2-3) is legally precise. The VRA-constrained analysis (locking enacted majority-minority districts, observing <0.3 pp EG impact) demonstrates that VRA compliance does not explain enacted plans' Republican advantage.

**Impact on Grofman's concerns**: Fully resolves his blocking VRA issue.

### 6. Geographic Sorting Mechanism Deep Dive (NEW Section 5.1)

**Strength**: The quantification of urban concentration (47 high-density Democratic districts producing 3.8× wasted vote asymmetry) provides empirical foundation for the -3.2% algorithmic baseline. The compactness-partisan tradeoff demonstration (27% compactness reduction → 1.5 pp EG improvement) shows courts face a fundamental choice: maximize compactness (producing -3.2% geographic bias) or tolerate non-compact districts to approach proportionality.

**Minor suggestion**: Consider adding a sentence clarifying that the compactness-partisan tradeoff is not *linear*—small compactness reductions yield modest EG improvements, but eliminating EG entirely would require severely non-compact districts (which courts reject as gerrymandered on compactness grounds). This reinforces that -3.2% represents a practical lower bound.

### 7. Compactness Correlation Analysis (NEW Section 4.3.1)

**Strength**: The critical finding—Arizona and Nevada enacted plans match algorithmic compactness (Polsby-Popper 0.28 and 0.25) yet show 6-8 pp higher efficiency gaps—definitively proves that partisan bias in enacted plans cannot be explained by compactness differences. The scatter plot showing no compactness-EG correlation within enacted plans (r=0.12) is powerful visual evidence.

**Impact**: This analysis strengthens the manipulation argument. Enacted plans achieve similar geometric compactness while introducing substantial partisan advantage, indicating mapmakers optimized for partisanship orthogonally to compactness.

### 8. Seats-Votes Full Treatment (NEW Section 4.6.2)

**Strength**: The expanded seats-votes analysis with uniform swing simulation, bootstrap confidence intervals, and historical comparison (2000: +1.2 pp bias → 2010: +3.8 pp → 2020: +6.0 pp) shows escalating partisan manipulation over time. The bias vs responsiveness decomposition clarifies that enacted plans fail on *both* dimensions: 6 pp Republican bias *plus* 25% responsiveness reduction.

**Minor suggestion**: The historical comparison (2000/2010/2020) is valuable but limited to five states (PA, WI, MI, NC, OH). Consider noting whether these five states are representative of national trends or whether Rust Belt states show uniquely escalating bias.

---

## Remaining Limitations and Future Work

While the revisions have addressed all blocking issues, a few limitations remain worth acknowledging:

### 1. Precinct-Level vs Block-Level Analysis

The efficiency gap calculations rely on precinct results allocated to districts. Block-level analysis would be more precise but computationally intensive. This is a reasonable tradeoff for national-scale analysis, but worth noting explicitly (you do mention this in Section 5.5, so this is addressed).

### 2. Election Selection

You analyze presidential and midterm elections (2016, 2018, 2020). Off-year or local elections might show different patterns. Again, you acknowledge this in limitations (Section 5.5).

### 3. Single Algorithmic Approach (Partially Addressed)

While you now report sensitivity analysis (alternative algorithms produce -2.8% to -3.6% EG), the primary findings use recursive bisection. Consider noting that other approaches like ReCom (Markov chain Monte Carlo) might produce different baselines. Your sensitivity analysis suggests the range would be narrow, but explicit acknowledgment strengthens methodological transparency.

### 4. Communities of Interest

The paper does not address whether algorithmic plans respect communities of interest (counties, cities, neighborhoods). Many state constitutions require minimizing county splits or respecting "communities of interest." Future work could analyze county-split counts or community coherence metrics to assess whether algorithmic plans meet these legal requirements.

---

## Minor Suggestions for Polish

### 1. Abstract

The abstract accurately summarizes findings but could benefit from one sentence previewing the robustness checks (multiple metrics) and compactness analysis (similar compactness, different EG). This would signal to readers that the paper goes beyond simple algorithmic-enacted EG comparison.

### 2. Figure Quality

Figures 1-4 (EG distribution, regional comparison, temporal stability, seats-votes curves) are clear and effective. Consider adding Figure 5 for the compactness-EG scatter plot (currently described in text but not visualized). Scatter plots are highly effective for demonstrating lack of correlation.

### 3. Table Formatting

Tables 1-5 are well-formatted. Consider adding a summary row at the bottom of Table 5 (multiple metrics) showing "Algorithmic advantage" or "Enacted advantage" to make the consistent directionality immediately obvious to readers skimming tables.

### 4. Citation Updates

Some references appear to be from pre-2024 literature. If any major partisan fairness papers have been published in 2024-2025, consider updating the literature review. (This is a minor point—your current citations are appropriate and comprehensive.)

---

## Recommendation

**Strong Accept**. This paper makes three major contributions:

1. **Empirical**: First national-scale efficiency gap analysis of algorithmic redistricting (50 states, 3 election years)
2. **Methodological**: Robustness demonstration across five partisan fairness metrics; compactness-EG independence proof
3. **Legal/policy**: State-specific algorithmic baselines for courts evaluating partisan gerrymandering claims under state constitutions

The revisions have transformed an already-strong empirical study into a comprehensive, legally-informed, methodologically-rigorous analysis that will shape both academic understanding and legal/policy debates around algorithmic redistricting.

I enthusiastically recommend acceptance for publication in the American Political Science Review.

---

## Score Changes from Round 1

**Round 1 Score**: 3.0/4 (Weak Accept - major revisions required)
**Round 2 Score**: 4.0/4 (Strong Accept)

**Reasons for score increase**:
- Legal framing corrected (Section 2.3): EG reframed as comparative tool, not constitutional threshold
- Proportionality-EG connection established (Section 4.6): mathematical derivation + empirical validation
- Multiple metrics robustness demonstrated (Section 4.8): convergence across five measures eliminates artifact concerns
- All P1 blocking issues resolved
- Paper now provides strongest empirical case for algorithmic redistricting in the literature
