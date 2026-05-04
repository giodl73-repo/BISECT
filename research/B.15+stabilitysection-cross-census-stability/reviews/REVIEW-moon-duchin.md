# Review: StabilitySection: Cross-Census Stability of GeoSection-Optimal Redistricting Maps

**Reviewer**: Moon Duchin (Rutgers University, MGGG Redistricting Lab)
**Expertise**: Redistricting mathematics, GerryChain ensemble methods, geometric probability, redistricting law, metric geometry
**Date**: 2026-05-02

## Overall Assessment

StabilitySection introduces a question that the redistricting mathematics literature has not addressed directly: is an algorithmic map's output invariant to the choice of census year? The Census Stability Score (CSS) is a well-motivated composite metric, and the connection to post-*Rucho* geographic determinism arguments is the paper's most important contribution. The finding that 67% of states show ratio-stability across the 2000--2010 decade is empirically interesting and appropriately cautious in how it is presented.

My central concern is structural: the CSS formula as defined cannot be computed from the data the paper currently has. The $s_{\text{seat}}$ component (weighted 50%) requires knowing partisan outcomes for 2000 and 2010 GeoSection maps, which in turn requires precinct-level election data for those years. The paper's abstract and introduction present CSS as a defined, computable quantity, but the evaluation section reports only ratio-stability findings ($s_{\text{ratio}}$). This gap needs to be resolved before the paper is submitted to a journal.

My second concern is that "cross-census stability" as defined in this paper is not a legally recognized concept. The legal section argues that CSS $\geq 0.90$ provides the "strongest possible geographic determinism claim," but there is no legal precedent for this standard. Courts have not accepted algorithmic stability across census cycles as a defense against gerrymandering claims, and the paper does not cite any case in which such an argument was advanced, let alone accepted. This does not mean the argument is wrong --- it is a novel legal theory --- but the paper should present it as such.

## Score: 3/4

**My score**: 3/4 --- Correct technical framework and compelling theoretical contribution; CSS is underspecified given current data availability, and the legal section overstates doctrinal grounding.

## Major Strengths

1. **Complement to ensemble methods**: The MCMC/GerryChain approach (which my lab develops) samples the space of valid plans within a single census year and asks whether an enacted plan is an outlier in that ensemble. StabilitySection asks a different question: whether the *optimal* plan is invariant across the three decennial ensembles. These are genuinely complementary analyses. A plan that is both (a) a typical member of the 2020 GeoSection ensemble and (b) the same plan that would be typical under 2000 and 2010 data has a stronger geographic determinism claim than either property alone.

2. **The Type I / Type II decomposition is methodologically important**: Separating population-shift effects (Type I) from tract-boundary-redesign effects (Type II) is the right way to think about cross-census comparability. The Census Bureau's tract redesign decisions are not neutral; they can propagate to downstream algorithmic outputs. The paper's planned 10-state decomposition addresses this correctly, though it has not yet been executed.

3. **Lorenz drift as a leading indicator**: The use of $\Delta p^*$ as a pre-census early warning is genuinely useful. If ACS 5-year estimates can predict which states will see ratio shifts before the decennial redistricting trigger, that has practical value for redistricting commissions. This is the paper's most actionable methodological contribution.

4. **Honest about what is pending**: The paper uses [TBD] markers appropriately and acknowledges that the three-census CSS requires 2010 data that is not yet complete. This transparency is appreciated.

## Major Issues (Must Address)

### Issue 1: CSS Formula Cannot Be Evaluated from Current Data
**Severity**: High
**Description**: The CSS formula is:
$$\mathrm{CSS}(s) = 0.5 \cdot s_{\text{seat}} + 0.3 \cdot s_{\text{ratio}} + 0.2 \cdot s_{\text{gap}}$$

The $s_{\text{seat}}$ component requires knowing whether GeoSection produces the same Democratic seat count in 2000 and 2010 as in 2020. Computing seat counts for 2000 and 2010 GeoSection maps requires precinct-level presidential election returns for those years matched to census tracts. The paper does not describe any such data source for 2000 and 2010, and the 2000/2010 evaluation section relies entirely on the tract split ratio $f$ (a purely geometric quantity) without any partisan data.

This means the paper is reporting two-year ratio-stability findings and calling the result a preliminary CSS, but the majority of the CSS weight (50%) is in a component that cannot be computed. The $s_{\text{gap}}$ component (20%) similarly requires election data.

This is not an insurmountable problem --- the paper could define a geometry-only stability metric for the current analysis and reserve the full CSS for when election data becomes available. But presenting CSS as the paper's primary contribution while being unable to compute its primary component is a structural problem.

**Recommendation**: Define a ratio-stability-only intermediate metric, call it CSS-geo, and present the 67% finding as a CSS-geo result. Reserve the full CSS definition for the complete three-census analysis. Alternatively, identify a 2000/2010 precinct election data source (Harvard Dataverse VEST has some coverage; MIT Election Lab has presidential returns by county) and explain why it cannot be matched to census tracts.

### Issue 2: "Cross-Census Stability" Is Not a Legally Recognized Concept
**Severity**: High
**Description**: Section 5.1 ("Legal Theory: CSS as a Litigation Tool") argues that CSS $\geq 0.90$ provides the "strongest possible geographic determinism claim" in post-*Rucho* state-court litigation. I find the theoretical argument compelling, but the paper presents it as if CSS stability is already a recognized legal standard.

No court has accepted cross-census algorithmic stability as a defense or as evidence in a redistricting case. The paper does not cite any precedent for the specific argument that a map being the "same across twenty years of demographic change" shifts the burden of proof to challengers. The cases cited --- *Rucho*, *LWV v. PA*, *NC Harper*, *Allen v. Milligan* --- do not discuss algorithmic stability at all.

This is a novel legal theory, and should be presented as such. It may well be persuasive to a court, but the paper needs to argue for *why* it should be persuasive, not assert that it provides a burden shift as a matter of existing doctrine. The proposed Districting Integrity Act framework is interesting but is entirely prospective; it does not describe current law.

**Recommendation**: Reframe Section 5.1 as a proposed legal theory with the following structure: (1) what the current post-*Rucho* doctrine requires in state courts; (2) why CSS stability is a novel argument that has not yet been litigated; (3) the theoretical case for why it should be persuasive, drawing on the analogy to established "natural geography" arguments. The "before living memory" framing is rhetorically powerful but should be labeled as an argument, not a legal standard.

### Issue 3: The Jaccard Similarity Computation Is Announced but Not Executed
**Severity**: Medium
**Description**: Section 3.2 defines the district Jaccard similarity and Section 6 lists it as a future work item ("Jaccard district-assignment stability"). But the introduction and conclusion both list Jaccard similarity as a contribution of this paper. The 2D stability picture (seed, census) refers to "district assignment stability" as if it is measured. It is not.

For a paper claiming to present StabilitySection as a framework, this is a significant gap. The ratio-stability result (f comparison) answers RQ1. It does not answer RQ3 (district assignment stability). A state could have identical ratio $f$ but substantially different tract assignments if population shifts reorganize which tracts cluster together.

**Recommendation**: Remove district Jaccard similarity from the contributions list and clearly place it in the future work section. The current paper's empirical contribution is ratio-stability (67% of states) and the CSS framework definition. That is sufficient for a journal paper; claiming additional contributions that are not executed weakens the paper.

## Minor Issues

- **The 5% threshold deserves a brief theoretical derivation**: The use of $|f_{2000} - f_{2010}| \leq 0.05$ as the stability threshold is stated without justification. For a methods paper, even a brief argument (e.g., that 5% of tracts corresponds to approximately one district's worth of tract-boundary uncertainty) would improve credibility. Alternatively, present results at 3% and 10% thresholds as robustness checks.

- **GerryChain cross-year comparison**: My lab's GerryChain/ReCom work could be used to generate a cross-year comparison baseline: sample 10,000 plans from the 2000 and 2010 MCMC ensembles and measure how often a randomly drawn 2000 plan matches a randomly drawn 2010 plan. This would give the paper a non-trivial null hypothesis for what "67% stable" means: is GeoSection more stable across years than a random compact plan?

- **The two-census CSS formula is ad hoc**: Equation for $\mathrm{CSS}_{2\text{yr}}$ uses the same weights as $\mathrm{CSS}_{3\text{yr}}$, but with fewer comparison points. The justification for identical weights with 2-year versus 3-year data should be stated.

- **Proposition 1 (Lorenz Drift Predicts CSS) needs a proof or proof sketch**: The proposition is stated with a formal bound involving $C_{\text{pop}}$ but this constant is not bounded or estimated empirically. Without an empirical estimate of $C_{\text{pop}}$, the proposition cannot be used as a stability predictor.

- **Tract-redesign contribution to instability is the most scientifically interesting question**: The Type I / Type II decomposition is the experiment I most want to see executed. For states where the ratio changes between census years, understanding whether it is population dynamics or Census Bureau tract redesign that drives the change would be a genuinely new contribution to the literature.

## Questions for Authors

1. For the 30 states with 2000 and 2010 sweep data, what fraction of the unstable states are in the Sun Belt growth corridor versus other regions? Does instability correlate with population growth rate as the metropolitan-fixity hypothesis predicts?

2. Is there a 2000 or 2010 precinct-level presidential election dataset that can be matched to census tracts? If so, can seat stability be computed for even a subset of states?

3. For the Lorenz drift proposition (Proposition 1), what is a typical empirical estimate of $C_{\text{pop}}$ in the dataset? Is the bound tight enough to make it predictive?

4. For states where the 2000 ratio is confirmed (47 complete), does the ratio-stability rate differ between states that changed seat count and states with fixed seat count? This would test whether the normalised ratio comparison introduces additional noise.

## Recommendation

The paper is a genuine contribution: it introduces an important new question (cross-census stability), provides a framework for answering it, and reports a first empirical finding (67% ratio-stability). The CSS formula needs to be reconciled with the available data, the legal theory needs to be presented as novel argument rather than established doctrine, and the Jaccard similarity contribution needs to be moved to future work. These are fixable issues. Revise and resubmit.
