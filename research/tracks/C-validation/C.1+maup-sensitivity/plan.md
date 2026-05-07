# MAUP Sensitivity Analysis — Plan

**Artifact Type**: Research Paper (12th Paper)
**Goal**: Test robustness of portfolio findings across spatial resolutions
**Estimated Effort**: 2-3 months (computational + analysis intensive)
**Status**: In progress - Phase 2 complete (10-state validation)

---

## Objective

Empirically test whether key findings from the 10-paper portfolio are sensitive to the **Modifiable Areal Unit Problem (MAUP)** by rerunning analyses at three spatial resolutions:
- **Census tracts** (~85,000 units, baseline)
- **Block groups** (~240,000 units, finer resolution)
- **Census blocks** (~11 million units, finest resolution)

**Core Question**: Are the portfolio's flagship findings (42% threshold, +69 MM surplus, temporal stability, etc.) artifacts of tract-level aggregation, or do they persist across resolutions?

**Why This Matters**:
- Addresses Duchin's critique: "What if you redid the entire analysis with block groups instead of tracts?"
- MAUP is a fundamental problem in spatial analysis—rarely tested empirically in redistricting
- Establishes whether findings are robust methodological artifacts or resolution-dependent
- Preempts reviewer questions about generalizability

---

## Research Questions

### RQ1: Threshold Robustness
**Does the 42% minority threshold (from threshold-analysis paper) hold across resolutions?**

- Hypothesis: Finer resolutions (blocks) may lower threshold due to more precise minority clustering
- Alternative: Threshold is demographic artifact (state minority %), not spatial resolution effect
- Test: Rerun threshold analysis on block groups and blocks, compare state-by-state

### RQ2: VRA Compliance Persistence
**Does the +69 MM district surplus persist with finer/coarser units?**

- Hypothesis: Block-level resolution produces even more MM districts (better minority clustering)
- Alternative: Tract-level is "sweet spot"—blocks too fine (gerrymandering risk), block groups too coarse
- Test: Count MM districts at all 3 resolutions, compare to enacted baseline

### RQ3: Compactness Sensitivity
**How does compactness change across resolutions?**

- Hypothesis: Finer resolutions improve compactness (more boundary optimization freedom)
- Alternative: Compactness is scale-invariant (Polsby-Popper ratio stable across resolutions)
- Test: Compute Polsby-Popper at tract/block group/block levels, measure variance

### RQ4: Temporal Stability Across Resolutions
**Does hierarchical structure advantage (recursive > n-way) hold at all resolutions?**

- Hypothesis: Hierarchical advantage weakens at finer resolutions (more units = less constraint)
- Alternative: Tree structure provides stability regardless of unit count
- Test: Tract retention rates (2010→2020) at block group and block levels

### RQ5: Computational Scalability
**What are the computational limits of finer resolutions?**

- Blocks: 11M units → 435 districts = massive graph partitioning problem
- METIS scalability: Does multilevel coarsening handle 11M nodes efficiently?
- Runtime analysis: Tract (seconds) vs block groups (minutes) vs blocks (hours?)

---

## Proposed Structure

### Abstract (150 words)
- Problem: Redistricting algorithms depend on choice of atomic units (MAUP)
- Method: Compare recursive bisection at 3 resolutions (tracts, block groups, blocks)
- Findings: (TBD after experiments)
- Contribution: First empirical MAUP test in algorithmic redistricting

### Section 1: Introduction (800 words)
- **The MAUP Problem**: Openshaw's seminal work, why spatial resolution matters
- **Redistricting Context**: Census units as atomic aggregations
- **Portfolio Findings**: Recap 42% threshold, MM surplus, temporal stability
- **Critique**: Duchin's question—"What if you used different units?"
- **This Paper**: Empirical test across 3 resolutions

**Key framing**:
> All prior work (10 papers) used census tracts. Valid methodological choice, but untested assumption. This paper asks: what changes when we don't?

### Section 2: Spatial Resolution in Redistricting (600 words)

#### 2.1: Census Geographic Hierarchy
- **Blocks**: ~11M nationally, ~100 residents, finest census unit
- **Block groups**: ~240K nationally, ~1,200 residents, intermediate aggregation
- **Tracts**: ~85K nationally, ~4,000 residents, standard redistricting unit
- **Why tracts?**: Historical precedent, computational tractability, granularity balance

#### 2.2: MAUP Theory
- **Scale effect**: Aggregate from fine to coarse changes values (minority %, compactness)
- **Zoning effect**: Different boundaries at same scale change values
- **Implications**: Results at tract-level may not generalize to block-level

#### 2.3: Prior Work on MAUP in Redistricting
- Altman & McDonald (1998): MAUP in partisan analysis
- Voss et al. (1995): Ecological inference and aggregation
- **Gap**: No prior work tests MAUP across algorithmic redistricting methods

### Section 3: Methodology (1,000 words)

#### 3.1: Data Preparation
**Census tracts** (baseline):
- 85,000 tracts, existing adjacency graphs, demographics from PL 94-171

**Block groups** (finer resolution):
- 240,000 block groups (~3× more units than tracts)
- Download from Census Bureau TIGER/Line files
- Build adjacency graphs via shared boundaries (Rook contiguity)
- Aggregate demographics from blocks to block groups

**Census blocks** (finest resolution):
- 11 million blocks (~130× more units than tracts)
- Computational challenge: METIS can handle? Memory limits?
- May require state-by-state processing (50 separate runs)
- Adjacency graph: ~35 million edges (estimated)

#### 3.2: Algorithm Configuration
**Consistent parameters across resolutions**:
- Population balance: ±0.5% (same tolerance)
- Edge-weighting: α = 5.0 (minority-minority edges)
- Minority threshold: τ = 0.40 (defines minority tracts/blocks)
- Partitioning method: Recursive bisection (METIS 5.1.0)

**Why consistency matters**:
- Isolate resolution effect—don't confound with parameter changes
- Enables direct comparison (apples-to-apples)

#### 3.3: Experimental Design
**Scope**:
- 50 states × 3 resolutions = 150 total redistricting runs
- Focus on 2020 census (2000/2010 optional if time permits)
- Subset analysis: Test 10 representative states first (validation)

**States for subset**:
- High minority: TX, CA, FL, GA, NY
- Low minority: VT, ME, WV, NH, ID
- Validation: If findings consistent → full 50-state run

**Metrics computed**:
- MM district count (primary VRA metric)
- Polsby-Popper compactness (primary geometric metric)
- Population deviation (verify ±0.5% maintained)
- Runtime (computational scalability analysis)

### Section 4: Results (2,000 words)

#### 4.1: Threshold Analysis Across Resolutions
**Finding 1**: Does 42% threshold hold?

**Expected structure**:
- Table 1: State minority % vs MM success rate (tract / block group / block)
- Figure 1: Scatter plots showing threshold at each resolution
- Statistical test: Logistic regression with resolution as factor

**Hypotheses**:
- H0: Threshold unchanged across resolutions (robust finding)
- H1: Threshold shifts (MAUP-sensitive finding)

#### 4.2: VRA Compliance Comparison
**Finding 2**: MM district counts across resolutions

**Expected structure**:
- Table 2: MM districts by state and resolution
  - Column 1: Enacted (baseline)
  - Column 2: Tract-level (68 → 137 finding from portfolio)
  - Column 3: Block group level
  - Column 4: Block level
- Figure 2: Bar chart showing surplus at each resolution
- Statistical test: Paired t-test (tract vs block group vs block)

**Key question**: Does +69 surplus grow or shrink with finer resolution?

#### 4.3: Compactness Sensitivity
**Finding 3**: Polsby-Popper scores across resolutions

**Expected structure**:
- Table 3: Mean compactness by state and resolution
- Figure 3: Box plots showing compactness distributions
- Analysis: Variance decomposition (between-resolution vs within-resolution)

**Mechanism analysis**:
- Finer resolution → more boundary optimization freedom?
- Or compactness is scale-invariant property?

#### 4.4: Temporal Stability (Optional)
**Finding 4**: Tract retention rates across resolutions (if 2010→2020 data available)

- Hypothesis: Hierarchical advantage (recursive > n-way) weakens with more units
- Test: Compute block group retention (2010 → 2020), compare to tract retention

#### 4.5: Computational Performance
**Finding 5**: Runtime and scalability

**Expected structure**:
- Table 4: Runtime by state and resolution
  - Vermont (smallest): Tract (30s) vs Block group (?) vs Block (?)
  - California (largest): Tract (8min) vs Block group (?) vs Block (?)
- Figure 4: Scatter plot (unit count vs runtime)
- Scalability analysis: Linear? Superlinear? METIS multilevel efficiency

**Computational limits**:
- At what resolution does METIS break? (memory, runtime)
- Block-level feasible for all 50 states?

### Section 5: Discussion (1,200 words)

#### 5.1: Interpretation of Findings
**If findings are robust** (H0 confirmed):
- Strengthens portfolio claims—not artifacts of tract aggregation
- Tract-level is validated as appropriate resolution
- Implications: Researchers can confidently use tracts for redistricting algorithms

**If findings are sensitive** (H1 confirmed):
- MAUP is real concern for algorithmic redistricting
- Findings at tract-level may not generalize to block-level
- Implications: Need resolution-aware analysis in all redistricting work

#### 5.2: Theoretical Implications
**MAUP Mechanism**:
- Why would resolution matter? Geographic clustering at different scales
- Minority communities may be "cohesive" at block-level but "dispersed" at tract-level
- Compactness metrics sensitive to boundary granularity

**Algorithm Behavior**:
- METIS edge-cut minimization interacts with graph size
- Finer graphs → more local optima → different solutions?

#### 5.3: Policy Implications
**For Redistricting Commissions**:
- If robust: Use tracts (computational efficiency advantage)
- If sensitive: Use finest resolution feasible (blocks or block groups)

**For Courts**:
- VRA compliance tests should specify resolution
- "Majority-minority district" definition depends on aggregation level

#### 5.4: Limitations
- **Computational**: Block-level may not be feasible for all states (memory limits)
- **Data**: Blocks have noisier demographics (small sample sizes)
- **Scope**: Only recursive bisection tested (n-way, other partitioners untested)
- **Census years**: Focus on 2020 (2000/2010 optional)

#### 5.5: Future Work
- Test MAUP with other partitioners (SCOTCH, Zoltan, spectral)
- Examine MAUP with different adjacency structures (queen, distance-based)
- Multi-resolution ensemble: Combine tract + block group + block results

### Section 6: Conclusion (400 words)
- Summary: First empirical MAUP test in algorithmic redistricting
- Key findings: (TBD—robust or sensitive?)
- Contribution: Establishes generalizability (or lack thereof) for portfolio
- Recommendation: Resolution choice guidance for practitioners

**Final sentence**:
"The choice of spatial resolution is not merely a technical detail—it is a methodological decision with consequences for democratic representation."

---

## Figures (6 total)

**Figure 1: Threshold Scatter Plots**
- 3 panels (tract / block group / block)
- X-axis: State minority percentage
- Y-axis: MM district success rate (%)
- 42% threshold line overlaid
- Shows whether threshold shifts across resolutions

**Figure 2: MM District Surplus Comparison**
- Bar chart: States on X-axis, MM count on Y-axis
- 4 bars per state: Enacted / Tract / Block Group / Block
- Highlights +69 surplus at tract level, change at other resolutions

**Figure 3: Compactness Distribution by Resolution**
- Box plots: 3 boxes (tract / block group / block)
- Shows whether compactness variance increases/decreases with resolution
- Overlay mean values for comparison

**Figure 4: Computational Scalability**
- Scatter plot: Unit count (X) vs Runtime seconds (Y)
- 3 colors: Tract (blue) / Block group (green) / Block (red)
- Log-log scale to show scaling behavior
- Demonstrates feasibility limits

**Figure 5: State-by-State MAUP Sensitivity**
- Heat map: States (rows) × Resolutions (columns) × Metric (MM count difference)
- Highlights which states are MAUP-sensitive vs robust
- Identifies geographic patterns (urban vs rural, high vs low minority)

**Figure 6: Portfolio Findings Robustness Summary**
- Table/graphic showing 5 key findings from portfolio
- For each finding: Robust (✓) or Sensitive (✗) across resolutions
- Visual summary of paper's main contribution

---

## Supplementary Materials

### Supplementary Tables
- **Table S1**: Full state-by-state results (50 states × 3 resolutions × 4 metrics)
- **Table S2**: Runtime benchmarks (all 150 runs)
- **Table S3**: Compactness by district (435 districts × 3 resolutions)
- **Table S4**: Minority concentration statistics (tract vs block group vs block)

### Supplementary Figures
- **Figure S1-S50**: State maps at all 3 resolutions (3 maps per state)
- **Figure S51**: Adjacency graph size comparison (nodes, edges, density)
- **Figure S52**: Memory usage analysis (METIS memory vs unit count)

### Supplementary Code
- Data download scripts (block groups, blocks)
- Adjacency graph construction (Rook contiguity)
- METIS wrappers for multi-resolution partitioning
- Analysis scripts (threshold regression, compactness variance decomposition)

---

## Target Venues

### Option 1: International Journal of Geographical Information Science (IJGIS)
**Why IJGIS?**
- Premier GIS venue, strong MAUP focus
- Computational geography fits well
- Impact factor ~5, high visibility
- Format: 8,000-10,000 words (sufficient for detailed analysis)

**Fit**:
- MAUP is core GIS problem
- Empirical test with real-world data (not simulation)
- Computational scalability analysis appeals to GIS community

### Option 2: Transactions in GIS
**Why TGIS?**
- Strong spatial analysis focus
- Publishes algorithmic work
- Impact factor ~3, solid venue
- Format: 7,000-9,000 words

**Fit**:
- Resolution sensitivity is fundamental spatial question
- Census geography hierarchy matches journal scope

### Option 3: American Political Science Review (APSR)
**Why APSR?**
- Political science flagship (impact factor ~7)
- Redistricting is political question
- MAUP implications for VRA compliance
- Format: 12,000 words (allows detailed discussion)

**Fit**:
- 42% threshold has legal implications
- MM district findings matter for VRA litigation
- Broader political science audience

**Challenge**:
- APSR may view as "too technical" (GIS methods)
- Requires strong political framing (not just methodological exercise)

### Option 4: Political Analysis
**Why Political Analysis?**
- Methods journal for political science
- Strong computational focus
- Impact factor ~6
- Format: 8,000-10,000 words

**Fit**:
- Methodological contribution (MAUP test)
- Redistricting methods are journal core
- Accepts technical papers

**Recommendation**: Submit to **IJGIS first** (best fit for MAUP), then Political Analysis if rejected.

---

## Computational Requirements

### Data Volume
- **Census tracts**: ~85K units, ~10GB shapefiles
- **Block groups**: ~240K units, ~35GB shapefiles
- **Census blocks**: ~11M units, ~500GB shapefiles (!)

**Storage**: ~600GB total for all resolutions + processed outputs

### Processing Time (estimated)
**Tract-level** (baseline):
- 50 states: ~4 hours (current pipeline)

**Block group level**:
- 3× more units → ~3× longer
- Estimated: 12-15 hours for 50 states

**Block-level**:
- 130× more units → ???
- Estimated: 50-100 hours (2-4 days)
- May require HPC cluster (not laptop)

### Computational Bottlenecks
1. **Adjacency graph construction**: Detecting shared boundaries at block-level is expensive
2. **METIS coarsening**: Multilevel algorithm scales well, but 11M nodes is large
3. **Memory**: Block-level graphs may exceed 64GB RAM for large states (CA, TX)

**Mitigation strategies**:
- State-by-state processing (not national parallel)
- Use METIS's disk-based partitioning (if memory limits hit)
- Start with 10-state subset to validate feasibility

---

## Implementation Timeline

### Phase 1: Data Preparation (2 weeks)
- Download block groups and blocks from Census Bureau
- Build adjacency graphs at all 3 resolutions
- Validate graph connectivity (all states)

### Phase 2: Subset Experiments (1 week)
- Run 10 representative states at all resolutions
- Validate METIS scalability at block-level
- Identify computational bottlenecks

### Phase 3: Full 50-State Runs (3-4 weeks)
- Tract-level: Re-run for consistency (baseline)
- Block group level: 50 states (~12-15 hours)
- Block-level: 50 states (~50-100 hours, may require HPC)

### Phase 4: Analysis (2 weeks)
- Compute metrics (MM districts, compactness, runtimes)
- Statistical tests (paired t-tests, regression, variance decomposition)
- Generate all figures and tables

### Phase 5: Writing (3-4 weeks)
- First draft (8,000 words + 6 figures)
- Internal review and revision
- Supplementary materials preparation

### Phase 6: Submission (1 week)
- Format for IJGIS guidelines
- Cover letter emphasizing MAUP novelty
- Submit with supplementary materials

**Total estimated timeline**: **2-3 months** (assuming no major computational failures)

---

## Key Challenges

### Challenge 1: Computational Feasibility
**Problem**: Block-level may exceed hardware limits (11M nodes, ~35M edges)
**Mitigation**:
- Test on smallest state (Vermont) first
- Use disk-based METIS if memory insufficient
- Consider cloud HPC (AWS, Azure) for large states

### Challenge 2: Data Quality at Block-Level
**Problem**: Blocks have small populations (~100 residents) → noisy minority %
**Mitigation**:
- Use Census Bureau's noise-infused data (differential privacy-aware)
- Report uncertainty intervals for block-level findings
- Acknowledge noisier estimates as limitation

### Challenge 3: Adjacency Graph Complexity
**Problem**: Detecting shared boundaries at block-level is computationally expensive
**Mitigation**:
- Use Census Bureau's adjacency files (if available)
- Or pre-compute adjacency once, cache for reuse
- Parallelize graph construction (50 states independent)

### Challenge 4: MAUP Null Result
**Problem**: What if findings are identical across resolutions? (null result)
**Response**:
- Null result is still valuable—confirms robustness
- Frame as "validation of portfolio findings"
- Emphasizes tract-level is appropriate choice
- Still publishable in IJGIS or Political Analysis

---

## Success Criteria

**Minimum viable paper** (if computational limits hit):
- Compare tract vs block group only (skip blocks)
- Focus on 10-20 representative states (not full 50)
- Core finding: 42% threshold + MM surplus robustness

**Full paper** (ideal scenario):
- All 3 resolutions (tract, block group, block)
- Full 50 states
- Comprehensive MAUP analysis
- Computational scalability insights

**High-impact paper** (stretch goal):
- Multi-census analysis (2000/2010/2020 at all resolutions)
- Theory section explaining MAUP mechanisms
- Policy recommendations for resolution choice

---

## Related Work Integration

This paper connects to all 10 portfolio papers by testing their generalizability:

**From recursive-bisection**:
- Does impossibility defense hold at block-level? (Yes—algorithm still can't see party)
- Core method applies to any resolution

**From edge-weighted-bisection**:
- Does compactness improvement (+56%) persist at finer resolutions?
- Edge-weighting mechanism independent of unit count

**From vra-compliance**:
- **Critical test**: Does +69 MM surplus replicate at block group/block level?
- If yes → robust finding; if no → MAUP artifact

**From threshold-analysis**:
- **Critical test**: Does 42% threshold shift across resolutions?
- If yes → demographic/geographic artifact; if no → universal threshold

**From cross-census-validation**:
- Does tract correspondence methodology work for block group correspondence?
- Hierarchical nesting (blocks → block groups → tracts) simplifies

**From compactness-tradeoff**:
- Does VRA-compactness Pareto frontier look same at all resolutions?
- Tests whether tradeoff is inherent or resolution-dependent

**From temporal-stability**:
- Does hierarchical advantage hold at finer resolutions?
- More units → less structural constraint?

**From multi-vs-edge**:
- Constraint conflict theory applies to any resolution
- Tests whether multi-constraint fails at block-level too

**From adaptive-bisection**:
- Does α≥5 dominance hold at block group/block level?
- Parameter sensitivity may differ with graph size

**From nway-vs-recursive**:
- Statistical equivalence test at multiple resolutions
- Does architectural choice matter more/less with finer units?

---

## Citation Strategy

**This paper cites all 10 portfolio papers**:
- Each paper contributes a finding to test for MAUP sensitivity
- Frame as: "We test whether the following findings hold across resolutions..."

**External citations**:
- Openshaw (1984): Original MAUP formulation
- Tobler (1989): First Law of Geography (scale effects)
- Altman & McDonald (1998): MAUP in redistricting
- Voss et al. (1995): Ecological inference and aggregation
- Chen & Rodden (2013): Automated redistricting methods
- Duchin et al. (MGGG): Ensemble methods

**Positioning**:
- "First empirical MAUP test in algorithmic redistricting"
- "Validation study for 10-paper research program"
- "Addresses Duchin's critique about generalizability"

---

## Progress Update (2026-02-08)

### Phase 1: Data Preparation ✓ COMPLETE
- [x] Downloaded Census block group shapefiles (2020) - All 50 states
- [x] Downloaded Census block shapefiles (2020) - All 50 states
- [x] Built adjacency graphs at block group level - All 50 states (239K units)
- [x] Built adjacency graphs at block level - All 50 states (8.1M units)
- [x] Multi-resolution infrastructure implemented and validated

### Phase 2: Validation Testing ✓ COMPLETE
- [x] Run Alabama + Colorado validation (all 3 resolutions) - SUCCESS
- [x] Run 10-state subset experiment (validate feasibility) - SUCCESS
  - High minority: AL (7D), GA (14D), MS (4D), SC (7D), TX (38D), MD (8D)
  - Low minority: VT (1D), NH (2D), ME (2D), WY (1D)
- [x] All 30 runs passed (10 states × 3 resolutions)
- [x] Infrastructure validated: Algorithm works at all resolutions

### Key Findings from Validation:
- **Scalability confirmed**: METIS handles block-level graphs successfully
- **Runtime feasible**: Even Texas (38D, 2.5M blocks) completes in reasonable time
- **Algorithm robust**: Recursive bisection works across 130× unit count range
- **Data quality verified**: Block groups and blocks have clean adjacency graphs

### Next Actions

#### Phase 2.3: Analyze Subset Results (CURRENT)
- [ ] Compare district boundaries across resolutions (tract vs block_group vs block)
- [ ] Measure computational time/resources for each resolution
- [ ] Preliminary MAUP sensitivity assessment
- [ ] Decide: Full 50-state run vs targeted subset approach

#### Phase 3: Full 50-State Runs (PENDING)
- [ ] If feasible → full 50-state runs at all 3 resolutions
- [ ] If not feasible → pivot to tract vs block group only
- [ ] Runtime estimate: ~50-100 hours for block-level (all 50 states)

#### Phase 4: Analysis & Writing (PENDING)
- [ ] Compute all metrics (MM districts, compactness, runtimes)
- [ ] Statistical tests and variance decomposition
- [ ] Generate figures and tables
- [ ] Draft paper (8,000 words + 6 figures)

---

**Created**: 2026-02-08
**Panel Reference**: REVIEW_PANEL.md Section IV, Theme 4 (Single-Ecosystem Limitation)
**Related Papers**: All 10 papers (tests their generalizability)
**Computational Risk**: High (block-level may not be feasible)
**Scientific Value**: High (addresses fundamental MAUP question)
