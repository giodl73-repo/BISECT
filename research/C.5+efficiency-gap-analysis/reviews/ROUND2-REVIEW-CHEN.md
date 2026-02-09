# Round 2 Review: Measuring Partisan Fairness in Algorithmic Redistricting

**Reviewer**: Jowei Chen (University of Michigan)
**Expertise**: Computational redistricting, neutral benchmarks
**Round**: 2 (Revision review)
**Date**: 2026-02-08

---

## Overall Assessment

This revised manuscript sets a new gold standard for computational redistricting research. The authors have addressed every methodological concern I raised, transforming what was already a technically sound analysis into an exemplary demonstration of scientific rigor, transparency, and robustness checking. The additions—algorithmic transparency (Section 3.4), sensitivity analysis, compactness correlation analysis (Section 4.3.1), and multiple metrics comparison (Section 4.8)—collectively establish that the -3.2% algorithmic efficiency gap baseline is not an artifact of METIS parameterization, metric selection, or compactness definition but a robust empirical finding reflecting residential geography.

Most impressive is the compactness correlation analysis, which provides the definitive answer to the central methodological question: "Is enacted plans' partisan bias explained by their lower compactness, or by deliberate manipulation beyond compactness differences?" The Arizona/Nevada identical-compactness cases (both 0.28 and 0.25 Polsby-Popper in algorithmic vs enacted, yet 6-8 pp EG differences) prove manipulation operates independently. This is precisely the kind of rigorous causal reasoning computational social science should aspire to.

The sensitivity analysis—testing alternative algorithms (k-means, shortest splitline, Voronoi), edge weight variations, and ensemble generation (100 maps × 5 states)—demonstrates that findings are not METIS-specific. The narrow range (-2.8% to -3.6% EG across methods, std dev 0.3% within method) confirms that any neutral algorithm produces similar baselines.

**Major methodological achievements:**

1. **Complete algorithmic specification**: METIS parameters fully documented, enabling exact replication
2. **Sensitivity analysis**: Alternative algorithms produce consistent baselines (±0.4 pp)
3. **Ensemble uncertainty quantification**: 100 maps per state show std dev 0.3%, far smaller than 8.3 pp algorithmic-enacted gap
4. **Compactness independence proof**: Identical compactness, different EG demonstrates orthogonal manipulation
5. **Multiple metrics convergence**: Five fairness measures all show consistent direction/magnitude

This paper will be the methodological benchmark for future computational redistricting research.

## Scoring

**Score**: 4.0/4 (Strong Accept)

**Score Justification**: All methodological concerns resolved. Transparency complete. Sensitivity analysis comprehensive. Compactness analysis definitive. Multiple metrics robust. Exemplary computational social science.

---

## Detailed Assessment

### 1. Algorithm Specification (NEW Section 3.4)

This section transforms the paper from "we used METIS" to complete reproducibility documentation.

**METIS parameterization** (lines 46-53):
```
nparts = 2: Binary partition at each recursion level
niter = 100: Number of refinement iterations
ufactor = 10: Population imbalance tolerance (±0.5% of target)
objtype = 'cut': Minimize edge cut (default METIS objective)
seed = deterministic: Fixed random seed for reproducibility
```

Every parameter is specified. The `ufactor = 10` corresponds to ±0.5% population tolerance, meeting constitutional requirements. The `objtype = 'cut'` confirms that METIS optimizes edge-cut minimization (geometric compactness proxy) without partisan objectives.

**Edge weight specification** (lines 39-44): Unweighted edges (equal weight for all adjacent tract pairs) avoids introducing geographic biases. Alternative weighting schemes (inverse distance, shared boundary length) could favor different compactness definitions, so unweighted is the neutral default.

**Partisan data exclusion** (lines 70-88): The explicit list of excluded data (election results, voter registration, demographic variables correlated with partisanship) plus verification via seed variation (different seeds produce different boundaries but stable EG) confirms genuine algorithmic neutrality.

**Population balance enforcement** (lines 58-68): All 435 districts achieve deviations <0.3% of target (mean absolute deviation: 0.12%), well within constitutional ±0.5% threshold. Hierarchical balancing ensures balanced splits at each recursion level before further subdivision.

**Minor suggestion**: Consider adding the specific Python version, NetworkX version, and operating system (you mention Python 3.13, NetworkX 3.1, but OS could affect numerical precision for edge cases). This is extremely minor—current documentation is already exemplary.

**Impact on initial concerns**: Fully resolves P1.2 issue. Algorithmic transparency is complete.

### 2. Sensitivity Analysis (NEW Section 3.4, lines 91-117)

**Alternative algorithms tested**:
- k-means clustering: Districts formed by assigning tracts to nearest centroids (k=435 nationally)
- Shortest splitline: Recursive splitting along shortest geographic lines bisecting population
- Voronoi diagrams: Districts as Voronoi cells around population-weighted seed points

**Results**: EG ranges from -2.8% to -3.6% (mean: -3.1%, std dev: 0.3%). All neutral algorithms cluster within a narrow band (±0.4 pp from -3.2% baseline), far from enacted plan bias (+5.1%).

**Edge weight variations**:
- Unweighted (baseline): EG = -3.2%, Polsby-Popper = 0.33
- Inverse distance: EG = -3.0%, Polsby-Popper = 0.31
- Shared boundary length: EG = -3.4%, Polsby-Popper = 0.35

Edge weight variations produce EG ranging from -3.0% to -3.4% (std dev: 0.2%). Again, tight clustering confirms robustness.

**Interpretation**: The -3.2% baseline is not a METIS-specific artifact but a property of neutral redistricting under compactness constraints. Different algorithms optimize different compactness definitions (k-means: centroid distance; splitline: geographic simplicity; Voronoi: territory compactness), yet all converge on -3% to -3.5% EG. This establishes that the baseline reflects residential geography, not algorithmic choice.

**Minor suggestion**: Consider adding one sentence about *why* different algorithms produce slightly different baselines. My hypothesis: algorithms that allow more elongated districts (splitline) pack urban Democrats less severely than algorithms prioritizing circular shapes (Voronoi), explaining the -2.8% vs -3.6% range. Discussing mechanisms would deepen understanding.

### 3. Ensemble Generation and Uncertainty Quantification (NEW Section 3.4, lines 118-132)

**Method**: Generate 100 algorithmic plans per state by varying random seed while holding all METIS parameters constant.

**Results** (5 states):
- Pennsylvania: Mean EG = -2.8%, std dev = 0.3%, range = [-3.4%, -2.3%]
- Texas: Mean EG = -3.6%, std dev = 0.4%, range = [-4.3%, -2.8%]
- California: Mean EG = -3.1%, std dev = 0.2%, range = [-3.6%, -2.7%]
- Florida: Mean EG = -3.2%, std dev = 0.3%, range = [-3.9%, -2.6%]
- North Carolina: Mean EG = -3.0%, std dev = 0.3%, range = [-3.7%, -2.4%]

**Key finding**: Within-state variation (std dev ≈ 0.3%) is vastly smaller than algorithmic-enacted gap (8.3 pp). This confirms that single algorithmic plan per state provides stable baseline—ensemble generation for all 50 states is unnecessary for establishing benchmarks.

**Statistical interpretation**: The std dev 0.3% represents irreducible uncertainty from tiebreaking in METIS refinement (when multiple partitions achieve similar edge cuts, METIS randomly selects among near-optimal solutions). This uncertainty is negligible compared to policy-relevant differences.

**Minor suggestion**: Consider reporting confidence intervals for the algorithmic-enacted difference. For Pennsylvania: algorithmic -2.8% (95% CI: [-3.4%, -2.3%]) vs enacted +7.5% yields difference of 10.3 pp (95% CI: [9.8 pp, 10.9 pp]). This would formalize statistical significance.

### 4. Compactness Correlation Analysis (NEW Section 4.3.1)

This is the paper's methodological showpiece—the definitive answer to whether enacted plans' partisan bias is explained by compactness differences.

**National comparison** (lines 78-83): Algorithmic plans achieve Polsby-Popper 0.33, enacted plans 0.29 (–12% difference). This modest compactness reduction could potentially explain some partisan bias. The critical question: *does* it?

**Answer: No.** The state-level analysis (Table, lines 91-110) provides falsifying evidence:

**Arizona**: Algorithmic Polsby-Popper = 0.28, Enacted Polsby-Popper = 0.28 (identical). Yet algorithmic EG = -1.9%, enacted EG = +4.7%, difference = 6.6 pp.

**Nevada**: Algorithmic Polsby-Popper = 0.25, Enacted Polsby-Popper = 0.25 (identical). Yet algorithmic EG = -3.1%, enacted EG = +5.3%, difference = 8.4 pp.

**Causal inference**: If compactness reduction *caused* partisan bias, states with identical compactness should show identical partisan bias. Arizona and Nevada falsify this hypothesis. Therefore, partisan bias in enacted plans results from boundary placement choices orthogonal to compactness.

**Scatter plot analysis** (lines 112-114): Within enacted plans, compactness does not predict efficiency gap (r = 0.12, p = 0.68). High-compactness states (Wisconsin: 0.34) show similar EG (+7.2%) to low-compactness states (Texas: 0.24, +6.2%). This lack of correlation demonstrates that mapmakers introduce bias through strategic boundary placement, not by accepting lower geometric compactness.

**Methodological gold standard**: This analysis exemplifies rigorous causal reasoning. By finding cases where the hypothesized cause (compactness reduction) is absent but the effect (partisan bias) remains present, you definitively reject the compactness-explains-bias hypothesis.

**Minor suggestion**: Consider adding a regression analysis predicting EG from compactness, controlling for state fixed effects. The coefficient on compactness should be near zero within enacted plans, formalizing the scatter plot's visual pattern. Including state fixed effects accounts for state-specific geographic baselines.

### 5. Multiple Metrics Comparison (NEW Section 4.8)

The five-metric robustness check addresses my concern that reliance on a single metric risks missing manipulation tactics that escape detection by that measure.

**Convergence finding** (Table 5): All five metrics agree on direction and approximate magnitude:
- Efficiency gap: algorithmic -3.2%, enacted +5.1% (8.3 pp diff)
- Partisan bias @50%: algorithmic +2.0 pp, enacted -6.0 pp (8.0 pp diff)
- Declination: algorithmic -4.2°, enacted +7.8° (12.0° diff)
- Mean-median: algorithmic +0.8 pp, enacted +4.1 pp (3.3 pp diff)
- Elasticity: algorithmic 2.8, enacted 2.1 (-0.7 diff)

**Cross-metric correlations** (lines 167-173): EG × partisan bias (r = 0.94), EG × declination (r = 0.89) demonstrate that metrics capture the same underlying asymmetry through different measurement approaches.

**Implication**: Critics cannot argue "efficiency gap shows bias, but other metrics don't"—all established partisan fairness measures converge on the same substantive conclusion.

**Minor suggestion**: Consider adding a table showing which manipulation tactics each metric detects:
- EG: Detects overall wasted vote asymmetry (packing + cracking)
- Mean-median: Detects packing specifically
- Partisan bias @50%: Detects advantage at vote parity
- Declination: Detects win margin manipulation
- Elasticity: Detects responsiveness dampening

This would clarify that metrics measure different gerrymandering pathologies, so convergence is not redundant but validating.

---

## Methodological Contributions

Beyond addressing my specific concerns, the revision makes three broader methodological contributions:

### 1. Algorithmic Benchmarking Framework

The paper establishes a replicable framework for generating neutral redistricting benchmarks:
1. Specify algorithm + parameters completely (Section 3.4)
2. Generate algorithmic plan(s) for jurisdiction
3. Compute partisan fairness metrics (EG, bias @50%, etc.)
4. Compare enacted plan metrics to algorithmic baseline
5. Deviations > 2-3 pp suggest manipulation

This framework could be operationalized for courts (state constitutional challenges) or redistricting commissions (map evaluation).

**Future application**: Every state could publish algorithmic baselines for its geography, enabling real-time comparison as redistricting commissions propose maps.

### 2. Sensitivity Analysis Protocol

The sensitivity analysis (alternative algorithms, edge weights, ensemble generation) provides a template for computational social science robustness checking. Future papers should report:
- Algorithm-to-algorithm variation (does finding persist across methods?)
- Parameter-to-parameter variation (does finding depend on specific parameterization?)
- Stochastic variation (does finding depend on random seed / initialization?)

Your finding that all three sources of variation produce tight clustering (±0.4 pp) demonstrates robustness.

### 3. Compactness Independence Test

The compactness correlation analysis provides a general method for distinguishing geographic determinism from manipulation:
- If partisan bias correlates with compactness → could be explained by geography
- If partisan bias uncorrelated with compactness (r ≈ 0) → must result from boundary placement

The Arizona/Nevada identical-compactness cases provide the strongest possible evidence: holding compactness exactly constant while observing EG variation directly isolates the causal effect of boundary placement choices.

**Generalization**: This method could be applied to other redistricting criteria. Example: Do enacted plans with similar county-split counts show different partisan bias? If yes → manipulation operates independently of county preservation.

---

## Remaining Methodological Questions

While the revision is methodologically exemplary, a few questions remain for future research:

### 1. Precinct vs Block Level Analysis

You compute efficiency gaps from precinct results allocated to districts. Block-level analysis would be more precise (precincts can split across district boundaries, creating allocation error). However, block-level election results are unavailable in most states.

**Question**: How much measurement error does precinct-level analysis introduce? Could you validate for 2-3 states where block-level results exist?

**Hypothesis**: Measurement error is likely small (<0.5 pp EG) because precincts rarely split across districts and populous precincts (where errors matter most) rarely split. But empirical validation would strengthen confidence.

### 2. Communities of Interest

Your algorithms optimize compactness (edge-cut minimization) but do not explicitly preserve "communities of interest" (cities, counties, neighborhoods). Many state constitutions require minimizing county splits or respecting communities.

**Question**: Do algorithmic plans split counties more than enacted plans? If yes, this could be a legal obstacle to adoption (even if partisan fairness improves).

**Hypothesis**: Algorithmic plans likely split counties at similar rates to enacted plans (both optimize compactness, which correlates with county preservation). But explicit analysis would address potential legal challenges.

### 3. Other Fairness Criteria (Competitiveness, Descriptive Representation)

Beyond partisan fairness, other redistricting values exist:
- **Competitiveness**: Do algorithmic plans create more/fewer competitive districts?
- **Descriptive representation**: Do algorithmic plans preserve/destroy communities of color?

You address descriptive representation (VRA compliance, Section 4.7)—finding algorithmic plans create *more* majority-minority districts (137 vs 68). Competitiveness remains unexplored.

**Future work**: Report count of competitive districts (won by <55%) in algorithmic vs enacted plans. This addresses a separate redistricting value dimension.

---

## Minor Suggestions for Polish

### 1. Computational Reproducibility

Current reproducibility documentation is excellent (METIS parameters, Python versions, seed values). Consider adding:
- DOI link to GitHub repository with code + data
- Docker container or conda environment file ensuring exact replication
- Compute time estimates (enables readers to assess feasibility of replication)

### 2. Figure Enhancements

Figures 1-4 are clear and effective. Consider adding:
- Figure 5: Compactness-EG scatter plot (currently described but not visualized)
- Figure 6: Sensitivity analysis showing EG distributions across alternative algorithms
- Figure 7: Ensemble EG distributions for 5 states (histograms showing std dev 0.3%)

Visual presentation of robustness checks enhances credibility.

### 3. Online Appendix

The paper is comprehensive but dense (34 pages). Consider moving some technical content to online appendix:
- Full METIS parameter justifications
- Alternative algorithm specifications (k-means, splitline, Voronoi)
- State-by-state ensemble statistics (currently aggregated)
- Additional sensitivity analyses

This would tighten the main text while preserving methodological depth for interested readers.

---

## Recommendation

**Strong Accept**. This paper makes foundational methodological contributions:

1. **Algorithmic transparency**: Complete specification enabling exact replication
2. **Sensitivity analysis**: Robustness across algorithms, parameters, stochastic variation
3. **Compactness independence**: Definitive proof that manipulation operates orthogonally to compactness
4. **Multiple metrics**: Convergence across five fairness measures eliminates artifact concerns

The revision transforms the paper from strong empirical study into methodological exemplar for computational redistricting research. It sets the standard for transparency, robustness checking, and causal inference that future work should aspire to match.

I enthusiastically recommend acceptance for publication in the American Political Science Review.

---

## Score Changes from Round 1

**Round 1 Score**: 3.0/4 (Accept - major revisions required)
**Round 2 Score**: 4.0/4 (Strong Accept)

**Reasons for score increase**:
- Algorithmic transparency complete (Section 3.4): METIS fully specified, reproducible
- Sensitivity analysis comprehensive: alternative algorithms, edge weights, ensembles all robust
- Compactness correlation analysis definitive: identical compactness, different EG proves manipulation
- Multiple metrics convergence: five measures all agree, eliminating artifact concerns
- All P1.2 methodological concerns fully addressed
- Paper now exemplifies best practices in computational social science
