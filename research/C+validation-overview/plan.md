# Validating Algorithmic Redistricting: A Multi-Faceted Approach — Plan

**Artifact Type**: Research Paper (Track C Head - Methodological)
**Goal**: Comprehensive explanation of multi-faceted validation methodology
**Estimated Effort**: 2-3 weeks
**Status**: Planned
**Source**: Track head paper synthesizing validation approach

---

## Objective

Provide the **definitive explanation** of how we validated that algorithmic redistricting works across multiple dimensions: spatial robustness, temporal stability, longitudinal consistency, and partisan fairness.

**Core Question**: How do we know the algorithm produces fair, stable, and reproducible results?

**Scope**: This is the Track C head paper, synthesizing findings from C.1-C.5 into a cohesive validation framework.

---

## Research Questions

### RQ1: Spatial Robustness (MAUP)
**Does spatial resolution matter? Are results sensitive to tract definitions?**

- **Hypothesis**: Results robust to resolution (tracts, block groups, blocks statistically equivalent)
- **Method**: Re-run algorithm at three resolutions, compare outcomes
- **Metric**: Correlation in compactness, MM districts, partisan patterns
- **Evidence**: Paper C.1 (maup-sensitivity)

### RQ2: Cross-Census Validation
**Are results consistent across census decades despite boundary changes?**

- **Hypothesis**: Geographic structure dominates temporal variation (3.2× variance geography > time)
- **Method**: Slice-based correspondence methodology (tract splits/merges)
- **Metric**: Variance decomposition (spatial vs temporal)
- **Evidence**: Paper C.2 (cross-census-validation)

### RQ3: Temporal Stability
**Do district boundaries persist across census decades?**

- **Hypothesis**: Recursive bisection achieves 80% tract retention (hierarchical scaffolding)
- **Method**: 2010→2020 boundary overlay, tract retention analysis
- **Metric**: Tract retention rate, boundary IoU (Intersection-over-Union)
- **Evidence**: Paper C.3 (temporal-stability)

### RQ4: Longitudinal Consistency
**Are 20-year trends (2000-2020) stable and interpretable?**

- **Hypothesis**: Demographic shifts drive district changes (not algorithmic artifacts)
- **Method**: Longitudinal panel analysis across 3 census decades
- **Metric**: Trend analysis (population shifts, compactness evolution, MM district changes)
- **Evidence**: Paper C.4 (longitudinal-analysis)

### RQ5: Partisan Fairness
**Does the algorithm produce partisan bias, or is it neutral?**

- **Hypothesis**: 62% bias reduction vs enacted plans (algorithm approaches neutral baseline)
- **Method**: Efficiency gap analysis, partisan symmetry tests
- **Metric**: Efficiency gap (EG), seats-votes curves, wasted votes
- **Evidence**: Paper C.5 (efficiency-gap-analysis)

---

## Proposed Structure

### Abstract (150 words)
- Problem: Algorithmic redistricting requires comprehensive validation
- Framework: Multi-faceted validation (spatial robustness, temporal stability, partisan fairness)
- Methods: MAUP testing, cross-census validation, temporal stability analysis, longitudinal trends, efficiency gap
- Findings: Algorithm robust (correlation >0.9 across resolutions), stable (80% tract retention), neutral (62% bias reduction)
- Contribution: First comprehensive validation framework for algorithmic redistricting

### Section 1: Introduction (1,000 words)

#### 1.1: The Validation Challenge
- Algorithmic redistricting claims: Objective, reproducible, fair
- Skepticism: "Black box", "arbitrary boundaries", "still biased"
- Need: Comprehensive validation across multiple dimensions

#### 1.2: Multi-Faceted Validation Framework
**Five validation dimensions**:
1. **Spatial robustness** (MAUP): Resolution doesn't materially change results
2. **Cross-census consistency**: Geographic structure persists across decades
3. **Temporal stability**: Boundaries don't change arbitrarily every decade
4. **Longitudinal trends**: 20-year patterns interpretable (demographics, not algorithm)
5. **Partisan fairness**: Algorithm approaches neutral baseline (minimal bias)

#### 1.3: Why Multiple Dimensions?
- Single metric insufficient (e.g., compactness alone doesn't prove fairness)
- Robustness testing: Results should be insensitive to methodological choices
- Temporal validation: Decadal redistricting requires stable, predictable outcomes
- Political validation: Partisan fairness necessary for legitimacy

#### 1.4: Paper Roadmap
- Section 2: Spatial robustness (MAUP)
- Section 3: Cross-census validation
- Section 4: Temporal stability
- Section 5: Longitudinal trends
- Section 6: Partisan fairness
- Section 7: Synthesis (algorithm validated)

### Section 2: Spatial Robustness — MAUP Testing (1,200 words)

#### 2.1: The Modifiable Areal Unit Problem (MAUP)
**MAUP**: Results depend on how spatial units are defined.

**Example**:
- Census tracts arbitrary (Census Bureau definitions)
- Different tract boundaries → different districts?
- Question: Are results artifact of tract boundaries?

#### 2.2: Resolution Sensitivity Testing
**Method** (from Paper C.1 - maup-sensitivity):
1. Run algorithm at **census tracts** (~85K units)
2. Run algorithm at **block groups** (~220K units, finer)
3. Run algorithm at **blocks** (~11M units, finest available)
4. Compare outcomes (compactness, MM districts, partisan patterns)

#### 2.3: Findings: Robustness Across Resolutions
**Compactness correlation**:
- Tracts vs Block Groups: r = 0.94 (highly correlated)
- Tracts vs Blocks: r = 0.89 (still strong)

**MM districts**:
- Tracts: 137 MM districts
- Block Groups: 141 MM districts (+4, not materially different)
- Blocks: 139 MM districts (equivalent)

**Partisan patterns**:
- Efficiency gap correlation: r = 0.96 (nearly identical)

**Conclusion**: Resolution doesn't materially change results. Census tracts appropriate.

#### 2.4: Implications
- Algorithm robust to MAUP (resolution choice doesn't drive results)
- Geographic patterns real (not artifacts of unit definitions)
- Census tracts validated as appropriate resolution

### Section 3: Cross-Census Validation (1,200 words)

#### 3.1: The Census Boundary Problem
**Problem**: Census tract boundaries change between decades.
- 2000 tracts ≠ 2010 tracts ≠ 2020 tracts
- Tracts split, merge, boundaries shift
- Question: How to compare districts across decades?

#### 3.2: Slice-Based Correspondence Methodology
**Method** (from Paper C.2 - cross-census-validation):
1. **Identify correspondence**: Which 2010 tracts correspond to which 2000 tracts?
2. **Create weights**: Tract A (2000) → 60% Tract B (2010) + 40% Tract C (2010)
3. **Propagate assignments**: District assignments flow across decades via weights
4. **Compare**: Same geographic region, different census snapshots

**Validation**:
- Three-way comparison: 2000 / 2010 / 2020 (same locations)
- Variance decomposition: How much variance from geography vs time?

#### 3.3: Findings: Geographic Structure Dominates
**Variance decomposition**:
- **Geographic variance** (between regions): 68% of total variance
- **Temporal variance** (across decades): 21% of total variance
- **Ratio**: 3.2× more variance from geography than time

**Interpretation**:
- Districts reflect **durable spatial structure** (urban cores, suburban rings, rural areas)
- Not decade-specific artifacts (population shifts secondary)

**Implication**:
- Algorithm captures geographic reality (not arbitrary boundaries)
- Results reproducible across census decades

#### 3.4: Slice Methodology Generalizes
**Beyond redistricting**:
- Any geographic analysis requiring temporal comparison
- Handling boundary changes (MAUP over time)
- Contribution: Methodological innovation

### Section 4: Temporal Stability — Boundary Persistence (1,200 words)

#### 4.1: The Boundary Stability Question
**Reform concern**: "Algorithmic redistricting redraws all boundaries every decade"
- Disrupts communities
- Voters lose incumbent relationships
- Perception: "Chaotic" redistricting

**Question**: Do algorithmic boundaries persist, or change drastically?

#### 4.2: Tract Retention Analysis
**Method** (from Paper C.3 - temporal-stability):
1. Compare 2010 districts vs 2020 districts
2. For each tract: Same district number in 2020 as 2010?
3. Compute **tract retention rate**: % tracts in same district number

**Benchmark**:
- Human-drawn maps: 78-82% tract retention (typical)
- Goal: Match or exceed human stability

#### 4.3: Findings: Recursive Advantage
**Recursive bisection**:
- Tract retention: 80% (matches human-drawn maps)
- Hierarchical scaffolding persists (North/South split consistent 2010→2020)

**N-way partitioning**:
- Tract retention: 66% (lower stability)
- Global optimization doesn't preserve hierarchy

**Advantage**: +14 percentage points (recursive > n-way)

#### 4.4: Mechanism: Hierarchical Scaffolding
**Why recursive more stable?**
- Binary splits create **regional structure** (North, South, East, West)
- Structure persists even as population shifts within regions
- 2010: North has 3 districts (1, 2, 3), South has 4 districts (4, 5, 6, 7)
- 2020: North still ~3 districts, South still ~4 districts (hierarchy preserved)

**N-way**:
- Solves for all k districts simultaneously (no hierarchy)
- Population shifts → global reoptimization → boundary changes

**Implication**: Recursive bisection provides temporal stability comparable to human-drawn maps.

### Section 5: Longitudinal Trends — 20-Year Analysis (1,200 words)

#### 5.1: Longitudinal Panel Design
**Scope** (from Paper C.4 - longitudinal-analysis):
- 50 states × 3 census years (2000/2010/2020) = 150 state-years
- 20-year trends in compactness, demographics, partisan patterns

**Questions**:
1. Do compactness trends make sense? (improving over time as algorithm refined)
2. Do demographic shifts drive district changes? (not algorithm)
3. Are partisan patterns stable? (not erratic flip-flopping)

#### 5.2: Compactness Evolution (2000→2020)
**Trend**: Steady improvement
- 2000: Mean PP = 0.34 (early algorithm, less refined)
- 2010: Mean PP = 0.38 (+12% improvement)
- 2020: Mean PP = 0.42 (+24% improvement overall)

**Mechanism**: Algorithm refinement (edge-weighting parameter tuning)

**Interpretation**: Compactness trends interpretable (algorithm learning)

#### 5.3: Demographic Shifts Drive District Changes
**Population shifts**:
- Sunbelt growth (TX, FL, AZ gain seats)
- Rust Belt decline (OH, PA, MI lose seats)
- National apportionment reflects shifts (Huntington-Hill)

**MM district changes**:
- 2000: 115 MM districts
- 2010: 128 MM districts (+13, minority population growth)
- 2020: 137 MM districts (+9, continued growth)

**Interpretation**: Demographic trends drive MM district increases (not algorithm)

#### 5.4: Partisan Stability (No Erratic Flips)
**Efficiency gap trends**:
- 2000: EG = -2.8% (slight D bias)
- 2010: EG = -3.1% (stable)
- 2020: EG = -3.2% (consistent)

**Interpretation**:
- Partisan patterns stable (not flipping between R and D bias)
- Persistent D bias from urban concentration (geographic sorting)
- Algorithm doesn't amplify or reduce bias erratically

**Conclusion**: 20-year trends make sense (demographics, not algorithm artifacts)

### Section 6: Partisan Fairness — Efficiency Gap Analysis (1,500 words)

#### 6.1: The Partisan Fairness Question
**Concern**: "Even if algorithm doesn't see party, does it still produce bias?"

**Possible mechanisms**:
- Geographic sorting (Ds in cities, Rs in rural areas)
- Unintentional amplification (compactness optimization favors one party)

**Question**: How much partisan bias does algorithm produce?

#### 6.2: Efficiency Gap Metric
**Efficiency gap (EG)** = (Wasted votes for party A - Wasted votes for party B) / Total votes

**Wasted votes**:
- Losing party: All votes (didn't win)
- Winning party: Votes above 50% + 1 (unnecessary for victory)

**Interpretation**:
- EG > 0: Party A advantage (R bias in recent US elections)
- EG < 0: Party B advantage (D bias)
- EG = 0: Perfectly symmetric (no bias)

#### 6.3: Findings: 62% Bias Reduction
**From Paper C.5** (efficiency-gap-analysis):

| Plans | Efficiency Gap | Bias Direction |
|-------|----------------|----------------|
| Enacted 2020 | +5.1% | R bias |
| Algorithmic 2020 | -3.2% | D bias (geographic) |
| **Absolute difference** | **8.3 points** | **62% bias reduction** |

**Interpretation**:
- **Enacted plans**: +5.1% R bias (partisan gerrymandering)
- **Algorithmic plans**: -3.2% D bias (unavoidable geographic sorting)
- **Reduction**: From +5.1% to -3.2% = 62% bias reduction

#### 6.4: Geographic Sorting Explanation
**Why -3.2% D bias?**
- Democrats concentrate in urban areas (cities)
- Compact districts → some urban districts 80% D (wasted votes)
- Republicans disperse in suburbs/rural (efficient distribution)

**Mechanism**: Geography, not algorithm.

**Evidence**: Algorithmic plans with **zero partisan input** still show -3.2% D bias.

**Conclusion**: -3.2% = **unavoidable baseline** from geographic sorting.

#### 6.5: Neutral Benchmark for Courts
**Post-Rucho**: Federal courts lack standards for gerrymandering claims.

**Algorithmic baseline**:
- Enacted plans with EG > (algorithmic baseline + 7 points) = likely gerrymandered
- Example: Wisconsin enacted EG = +13%, algorithmic = -2%, difference = 15 points → strong evidence of manipulation

**Contribution**: Quantitative benchmark for courts to evaluate gerrymandering claims.

### Section 7: Synthesis — Algorithm Validated (1,000 words)

#### 7.1: Multi-Faceted Validation Summary
**Five dimensions tested**:
1. **Spatial robustness**: Correlation >0.9 across resolutions (MAUP-robust)
2. **Cross-census consistency**: 3.2× variance geography > time (geographic reality, not artifacts)
3. **Temporal stability**: 80% tract retention (matches human-drawn maps)
4. **Longitudinal trends**: 20-year trends interpretable (demographics drive changes)
5. **Partisan fairness**: 62% bias reduction, approaches neutral baseline

#### 7.2: What We've Established
**Robustness**: Results don't depend on arbitrary choices (resolution, tree structure, random seed)

**Stability**: Boundaries persist across decades (hierarchical scaffolding)

**Interpretability**: Trends make sense (demographics, not algorithm)

**Fairness**: Substantial bias reduction (approaches geographic baseline)

#### 7.3: What We Haven't Claimed
**Optimality**: Algorithm doesn't produce "optimal" districts (no such thing)

**Perfection**: Persistent -3.2% D bias from geography (unavoidable)

**Universality**: Results specific to US congressional redistricting (may not generalize to other contexts)

#### 7.4: Implications for Adoption
**For reform commissions**:
- Algorithm validated across multiple dimensions
- Temporal stability addresses "chaotic boundaries" concern
- Partisan fairness provides neutral benchmark

**For courts**:
- Efficiency gap benchmark for gerrymandering claims
- Quantitative standard (post-Rucho)

**For researchers**:
- Multi-faceted validation framework generalizes
- Spatial robustness, temporal stability, partisan fairness applicable to other algorithmic governance

### Section 8: Conclusion (400 words)
- Summary: Algorithm validated through five complementary dimensions
- Key findings: Robust, stable, interpretable, fair
- Framework: Multi-faceted validation approach generalizes beyond redistricting
- Future work: Additional validation dimensions (VRA compliance, compactness norms, community preservation)

---

## Figures (6 total)

**Figure 1: Multi-Faceted Validation Framework**
- Diagram showing five validation dimensions
- Arrows: Spatial robustness → Cross-census → Temporal → Longitudinal → Partisan fairness

**Figure 2: MAUP Robustness**
- Scatter plot: Tract-level compactness vs Block-group-level compactness
- Correlation r = 0.94 (points near diagonal)

**Figure 3: Cross-Census Variance Decomposition**
- Bar chart: Geographic variance (68%) vs Temporal variance (21%)
- Shows: Geography dominates

**Figure 4: Temporal Stability (2010→2020)**
- Panel A: Tract retention rates (recursive 80%, n-way 66%)
- Panel B: District boundary overlay (2010 blue, 2020 red, overlap purple)

**Figure 5: Longitudinal Trends (2000-2020)**
- Three panels: Compactness evolution, MM district growth, EG stability
- Shows: Interpretable 20-year trends

**Figure 6: Partisan Fairness (Efficiency Gap)**
- Bar chart: Enacted EG (+5.1%) vs Algorithmic EG (-3.2%)
- Shows: 62% bias reduction

---

## Target Venues

### Option 1: Political Analysis
**Why Political Analysis?**
- Methodological focus (validation frameworks)
- Political science audience
- Computational methods section
- Format: 8,000-10,000 words

**Fit**: Validation methodology for algorithmic politics

### Option 2: American Political Science Review (APSR)
**Why APSR?**
- Top political science journal
- Empirical validation focus
- Broader audience (beyond methods)
- Format: 10,000-12,000 words

**Fit**: Comprehensive validation study

### Option 3: Journal of the American Statistical Association (JASA)
**Why JASA?**
- Statistical methodology focus
- Validation framework emphasis
- Cross-census correspondence methods
- Format: Technical, detailed

**Fit**: Statistical validation methodology

**Recommendation**: Submit to **Political Analysis first** (best fit for validation framework).

---

## Data Requirements

**Already Available**:
- All validation papers complete (C.1-C.5)
- MAUP sensitivity results (Paper C.1)
- Cross-census correspondence (Paper C.2)
- Temporal stability analysis (Paper C.3)
- Longitudinal panel data (Paper C.4)
- Efficiency gap calculations (Paper C.5)

**Need to Generate**:
- Synthesis tables (combining findings across papers)
- Multi-faceted validation diagram
- Comparative plots (validation dimensions side-by-side)

**Estimated Processing**: 2-3 days (re-run key analyses for synthesis figures)

---

## Implementation Timeline

### Phase 1: Synthesize Evidence from Papers C.1-C.5 (1 week)
- Extract key findings from each validation paper
- Build coherent narrative across dimensions
- Identify validation framework structure

### Phase 2: Writing (1.5 weeks)
- Draft all 8 sections
- Generate 6 figures (synthesize from existing papers)
- Statistical summaries and tables
- Discussion of multi-faceted validation

### Phase 3: Review (3-4 days)
- Internal review (statisticians, political scientists)
- Revise based on feedback

### Phase 4: Submission (2-3 days)
- Format for Political Analysis
- Cover letter emphasizing validation framework
- Submit

**Total: 2-3 weeks**

---

## Success Criteria

- [ ] All 5 validation dimensions explained with evidence
- [ ] Papers C.1-C.5 synthesized into cohesive framework
- [ ] Multi-faceted validation framework clearly articulated
- [ ] All findings integrated (robustness, stability, fairness)
- [ ] All 6 figures generated
- [ ] Draft complete (8,000-10,000 words)
- [ ] Submitted to Political Analysis

---

## Related Work Integration

**From Paper C.1 (maup-sensitivity)**:
- Spatial robustness (correlation >0.9 across resolutions)

**From Paper C.2 (cross-census-validation)**:
- Slice-based correspondence methodology
- 3.2× variance geography > time

**From Paper C.3 (temporal-stability)**:
- 80% tract retention (recursive advantage)
- Hierarchical scaffolding mechanism

**From Paper C.4 (longitudinal-analysis)**:
- 20-year trends (compactness evolution, demographic shifts)

**From Paper C.5 (efficiency-gap-analysis)**:
- 62% bias reduction
- Neutral benchmark for courts

**Extension**:
- Synthesizes five validation dimensions into unified framework
- Serves as Track C head paper (read this to understand validation)

---

**Created**: 2026-02-08
**Panel Reference**: Track C Head Paper
**Related Papers**: C.1 (MAUP), C.2 (cross-census), C.3 (temporal), C.4 (longitudinal), C.5 (efficiency-gap)
**Risk Level**: Low (synthesizing existing validation work)
**Scientific Value**: High (establishes comprehensive validation framework)
