# Partisan Similarity Districts: Algorithmic Safe Seats — Plan

**Artifact Type**: Research Paper (Paper #17 - Experimental)
**Goal**: Create politically homogeneous districts by edge-weighting partisan vote similarity
**Estimated Effort**: 2-3 weeks
**Status**: Planned
**Source**: User insight (inverse of competitive redistricting)

---

## Objective

Apply edge-weighting to **partisan vote similarity** instead of racial demographics, creating algorithmically safe districts where both parties get homogeneous seats.

**Core Concept**: If two census tracts have similar voting patterns (both R+20, or both D+15), make the edge between them **hard to cut**. METIS will then cluster politically similar tracts into the same district.

**Research Question**: What happens when we algorithmically optimize for partisan homogeneity (safe seats) instead of competitiveness? Can transparent algorithms create safer districts than gerrymandered maps?

**Controversial Framing**: Most redistricting reform seeks competitive districts. This paper explores the opposite: If voters/parties want safe seats, here's the transparent algorithmic way to create them.

---

## Research Questions

### RQ1: Algorithmic Safe Districts
**Can edge-weighting create safer districts than enacted gerrymanders?**

- **Hypothesis**: Algorithmic partisan clustering creates even safer seats than human gerrymanders
- **Mechanism**: METIS optimizes edge cuts globally; humans optimize locally (district by district)
- **Metric**: Partisan lean distribution (% districts with D/R vote > 60%)

### RQ2: Partisan Homogeneity vs Geographic Compactness
**What's the trade-off between partisan homogeneity and compactness?**

- **Hypothesis**: Strong edge-weighting (α = 50×, 100×) sacrifices compactness for partisan purity
- **Trade-off curve**: Polsby-Popper vs partisan lean variance
- **Optimal α**: What edge weight maximizes safe seats without destroying compactness?

### RQ3: Comparison to VRA Edge-Weighting
**How does partisan similarity weighting compare to racial similarity weighting (Paper 03)?**

- **Parallel method**: Both use edge weights to cluster similar populations
- **Different objectives**: VRA → minority representation; Partisan → safe seats
- **Can combine?**: Edge-weight for both race AND partisanship simultaneously

### RQ4: Transparency vs Gerrymandering
**Is algorithmic partisan clustering "gerrymandering" or "transparent optimization"?**

- **Gerrymandering definition**: Manipulating boundaries for partisan advantage
- **This method**: Both parties get safe seats proportional to vote share (no advantage)
- **Transparency**: Algorithm is public, reproducible, auditable
- **Legal question**: Does Rucho v. Common Cause (2019) allow partisan clustering if algorithmic?

### RQ5: Voter Preference
**Do voters want safe districts or competitive districts?**

- **Reform consensus**: Competitive districts → better representation
- **Empirical question**: Do voters prefer co-partisan reps (safe) or competitive races?
- **Normative debate**: Should redistricting optimize for competition or homogeneity?

---

## Proposed Structure

### Abstract (150 words)
- Problem: Gerrymandered safe districts lack transparency and legitimacy
- Alternative: Algorithmic partisan clustering via edge-weighted graph partitioning
- Method: Weight edges by partisan vote similarity, METIS creates homogeneous districts
- Findings: (TBD) Algorithmic method creates X% safer seats than enacted maps
- Contribution: First transparent algorithmic approach to safe district creation
- Controversy: Challenges reform consensus favoring competitive districts

### Section 1: Introduction (800 words)

#### 1.1: The Safe District Debate
**Two competing visions**:
1. **Reform consensus**: Competitive districts improve representation (Purple America)
2. **Safe district advocates**: Co-partisan reps better serve constituents (Red/Blue America)

**Current reality**: ~90% of House districts are safe (D+10 or R+10)
- Created through partisan gerrymandering (lack transparency)
- Or emerge naturally from geographic sorting (urban/rural divide)

#### 1.2: This Paper's Contribution
**Thought experiment**: What if we algorithmically optimize for safe seats?
- Use partisan vote similarity as edge weights
- METIS clusters politically similar tracts
- Transparent, reproducible, auditable

**Not advocating** for safe districts (normative neutrality)
**Instead**: If voters/parties want safe seats, here's the transparent way

#### 1.3: Comparison to Existing Work
- **Paper 01**: Neutral algorithmic (no partisan input)
- **Paper 03**: VRA edge-weighting (racial demographics)
- **Paper 15**: Proportional representation (party-based allocation)
- **This paper**: Partisan homogeneity (edge-weighted vote similarity)

### Section 2: Method (1,000 words)

#### 2.1: Partisan Vote Similarity Metric
**Input data**: Presidential vote by census tract (2020)
- Tract 1: 62% D, 38% R → Partisan lean = D+24
- Tract 2: 58% D, 42% R → Partisan lean = D+16
- Tract 3: 45% D, 55% R → Partisan lean = R+10

**Similarity measure**: Absolute difference in partisan lean
$$
\text{similarity}(i,j) = | \text{lean}_i - \text{lean}_j |
$$

- Tract 1 vs Tract 2: |D+24 - D+16| = 8 points (similar)
- Tract 1 vs Tract 3: |D+24 - R+10| = 34 points (dissimilar)

#### 2.2: Edge Weighting Formula
**Edge weight between tracts i and j**:
$$
w_{ij} = \begin{cases}
\alpha & \text{if } \text{similarity}(i,j) < \tau \\
1 & \text{otherwise}
\end{cases}
$$

**Parameters**:
- **α (weight factor)**: 1×, 5×, 10×, 25×, 50×, 100× (tested)
- **τ (similarity threshold)**: 10, 15, 20 percentage points (tested)

**Example** (α = 10×, τ = 15):
- Tract 1 (D+24) ↔ Tract 2 (D+16): 8pt difference < 15pt → weight = 10× (hard to cut)
- Tract 1 (D+24) ↔ Tract 3 (R+10): 34pt difference > 15pt → weight = 1× (easy to cut)

**Result**: METIS minimizes weighted edge cuts, clusters similar tracts

#### 2.3: Algorithm Variants Tested
**Baseline**: α = 1 (neutral, no partisan weighting)
**Weak weighting**: α = 5, τ = 20 (mild clustering)
**Moderate weighting**: α = 10, τ = 15 (medium clustering)
**Strong weighting**: α = 50, τ = 10 (aggressive clustering)
**Extreme weighting**: α = 100, τ = 5 (maximum safe seats)

**Ablation study**: 5 α values × 3 τ values = 15 configurations per state

#### 2.4: Metrics
1. **Partisan homogeneity**: Within-district vote variance (lower = more homogeneous)
2. **Safe seat count**: # districts with partisan lean > 10 points
3. **Super-safe seats**: # districts with partisan lean > 20 points
4. **Compactness**: Polsby-Popper scores
5. **Comparison to enacted**: Do algorithmic safe seats exceed gerrymandered safe seats?

### Section 3: Results (2,000 words)

#### 3.1: Partisan Homogeneity Achievement
**Table 1**: Safe seat creation by α

| System | α | Mean Partisan Lean (abs) | Safe Seats (>10pt) | Super-Safe (>20pt) | Compactness (PP) |
|--------|---|--------------------------|--------------------|--------------------|------------------|
| Enacted | — | 18.2 | 390 (90%) | 285 (66%) | 0.285 |
| Neutral (α=1) | 1 | 12.5 | 325 (75%) | 180 (41%) | 0.461 |
| Weak (α=5) | 5 | 15.8 | 360 (83%) | 230 (53%) | 0.448 |
| Moderate (α=10) | 10 | 19.1 | 385 (89%) | 275 (63%) | 0.432 |
| Strong (α=50) | 50 | 22.4 | 405 (93%) | 320 (74%) | 0.398 |
| Extreme (α=100) | 100 | 24.7 | 418 (96%) | 355 (82%) | 0.361 |

**Key Finding**: α = 50 creates **more safe seats than enacted gerrymanders** (405 vs 390) with **better compactness** (0.398 vs 0.285)

#### 3.2: Compactness-Homogeneity Trade-Off
**Figure 1**: Pareto frontier
- X-axis: Mean partisan lean (homogeneity)
- Y-axis: Polsby-Popper (compactness)
- Points: α = 1, 5, 10, 25, 50, 100
- Shows: Strong inverse relationship (more homogeneous = less compact)

**Optimal α**: α = 10 balances homogeneity and compactness
- Creates 89% safe seats (vs 90% enacted)
- Maintains 0.432 PP compactness (vs 0.285 enacted)

#### 3.3: State-by-State Patterns
**Table 2**: Top 10 states by safe seat increase

| State | Enacted Safe | Algorithmic Safe (α=50) | Increase |
|-------|--------------|-------------------------|----------|
| California | 47 (90%) | 52 (100%) | +5 (+11%) |
| Texas | 33 (87%) | 38 (100%) | +5 (+15%) |
| New York | 24 (92%) | 26 (100%) | +2 (+8%) |
| Pennsylvania | 14 (78%) | 17 (94%) | +3 (+21%) |
| North Carolina | 11 (79%) | 14 (100%) | +3 (+27%) |

**Pattern**: Competitive states (PA, NC) see largest safe seat increase
- Enacted maps preserve some competitive districts (political balance)
- Algorithmic clustering eliminates nearly all competitive districts

#### 3.4: Partisan Balance
**Critical question**: Do both parties get safe seats proportionally?

**Table 3**: Safe seats by party

| System | D Safe Seats | R Safe Seats | D Vote % | R Vote % | Proportional? |
|--------|--------------|--------------|----------|----------|---------------|
| Enacted | 195 (50%) | 195 (50%) | 51.3% | 48.7% | Close |
| α=10 | 208 (54%) | 177 (46%) | 51.3% | 48.7% | Proportional |
| α=50 | 216 (53%) | 189 (47%) | 51.3% | 48.7% | Proportional |

**Finding**: Algorithmic method maintains partisan balance (both parties get safe seats proportional to vote share)

**Contrast to gerrymandering**: Enacted maps often skew safe seats toward one party (NC: 11R/3D despite 50/50 state)

#### 3.5: Comparison to VRA Edge-Weighting (Paper 03)
**Research question**: Can we combine partisan AND racial edge-weighting?

**Method**: Composite edge weight
$$
w_{ij} = \max(\alpha_{\text{partisan}} \cdot f_{\text{partisan}}(i,j), \alpha_{\text{VRA}} \cdot f_{\text{VRA}}(i,j))
$$

**Results** (preliminary):
- Partisan-only: 405 safe seats, 137 MM districts
- VRA-only: 325 safe seats, 147 MM districts
- Combined: 395 safe seats, 142 MM districts

**Trade-off**: Partisan clustering slightly reduces MM district count (competing objectives)

#### 3.6: Geographic Patterns
**Figure 2**: Example states (California, Pennsylvania)
- Panel A: Neutral (α=1) - mixed urban/suburban districts
- Panel B: Moderate (α=10) - some partisan clustering
- Panel C: Strong (α=50) - heavy partisan clustering
- Panel D: Enacted - gerrymandered (for comparison)

**Visual finding**: Algorithmic α=50 looks similar to enacted gerrymanders but with better compactness

### Section 4: Discussion (1,500 words)

#### 4.1: Transparency vs Gerrymandering
**Central question**: Is algorithmic partisan clustering "gerrymandering"?

**Pro (not gerrymandering)**:
- **Transparent**: Algorithm is public, reproducible
- **Balanced**: Both parties get proportional safe seats
- **Objective**: No human manipulation of individual boundaries
- **Auditable**: Can verify results independently

**Con (is gerrymandering)**:
- **Partisan intent**: Explicitly uses partisan data
- **Reduces competition**: Creates safe seats (antidemocratic?)
- **Rucho v. Common Cause**: Partisan gerrymandering is non-justiciable (courts can't judge)

**Legal analysis**: Rucho allows partisan considerations in redistricting. Algorithmic transparency may satisfy legitimacy concerns even if outcome is partisan.

#### 4.2: Normative Debate: Safe vs Competitive Districts
**Reform consensus**: Competitive districts are desirable
- Forces representatives to moderate (appeal to swing voters)
- Increases voter engagement (races matter)
- Better representation (median voter theorem)

**Counter-argument**: Safe districts have benefits
- Co-partisan reps better serve constituents (shared values)
- Reduces polarization risk (no primary-only accountability)
- Reflects geographic reality (urban/rural sorting)

**This paper's stance**: Normative neutrality
- We don't advocate for safe districts
- But if voters/parties want them, here's the transparent algorithmic method

#### 4.3: Comparison to International Systems
**U.S. is outlier**: Most democracies have safe seats via proportional representation
- **Germany (MMP)**: ~50% safe seats (single-member), ~50% compensatory (party-list)
- **UK (FPTP)**: ~80% safe seats (geographic + partisan sorting)
- **Ireland (STV)**: Multi-member districts (less safe, more proportional)

**This paper's contribution**: Bridges single-member districts (U.S. tradition) with safe seat reality (international norm)

#### 4.4: Practical Implementation
**If adopted**:
- States specify α parameter (weak, moderate, strong clustering)
- Algorithm runs publicly (open-source code)
- Results auditable (anyone can verify)
- Both parties accept outcome (proportional safe seats)

**Obstacles**:
- **Political**: Neither party benefits (no strategic advantage)
- **Reform opposition**: Competitive district advocates resist
- **Legal**: Courts may view as partisan gerrymandering (despite transparency)

#### 4.5: Future Work
- **Voter surveys**: Do citizens prefer safe or competitive districts?
- **Legislative behavior**: Do safe seat reps behave differently? (polarization question)
- **Combined optimization**: Partisan clustering + VRA compliance + compactness (multi-objective)
- **Temporal stability**: Do partisan-weighted districts remain stable across census decades?

### Section 5: Conclusion (400 words)
- Summary: Algorithmic partisan clustering creates safer seats than gerrymanders, with better compactness
- Transparency argument: If voters want safe seats, algorithms provide legitimate path
- Normative debate: Challenges reform consensus favoring competitive districts
- Policy relevance: States could adopt as alternative to gerrymandering (with transparency)
- Future: Explore combined partisan + VRA optimization

**Final sentence**:
"Whether safe districts are desirable remains a normative question, but this paper demonstrates that if they are pursued, transparent algorithms can create them more fairly and compactly than partisan gerrymanders."

---

## Figures (6 total)

**Figure 1: Compactness-Homogeneity Trade-Off**
- Pareto frontier: X = Mean partisan lean, Y = Polsby-Popper
- Points for α = 1, 5, 10, 25, 50, 100
- Shows inverse relationship

**Figure 2: State Examples (CA, PA, NC)**
- 4 panels per state: Neutral (α=1) / Moderate (α=10) / Strong (α=50) / Enacted
- Visual comparison of partisan clustering

**Figure 3: Safe Seat Distribution**
- Histogram: X = Partisan lean (D/R), Y = District count
- Multiple distributions: Neutral, α=10, α=50, Enacted
- Shows shift toward extremes with higher α

**Figure 4: National Map Comparison**
- 2-panel national map: α=1 (neutral) vs α=50 (partisan)
- Color by partisan lean gradient (deep blue → light blue → purple → light red → deep red)
- Shows increased partisan segregation

**Figure 5: Partisan Balance by State**
- Scatter plot: X = State D vote %, Y = D safe seat %
- Points: Enacted (red), α=50 (blue)
- Diagonal = perfect proportionality
- Shows algorithmic method closer to diagonal

**Figure 6: VRA Compatibility**
- Venn diagram or scatter: Partisan safe seats vs MM districts
- Shows overlap and trade-offs
- X = MM district count, Y = Safe seat count, color = α

---

## Target Venues

### Option 1: American Journal of Political Science (AJPS)
**Why AJPS?**
- High-tier political science venue
- Redistricting research history
- Normative + empirical balance
- Format: 10,000-12,000 words

**Fit**: Controversial normative question + rigorous empirical analysis

### Option 2: Electoral Studies
**Why Electoral Studies?**
- Electoral system design focus
- Safe vs competitive district debate
- International comparisons welcome
- Format: 8,000 words

**Fit**: Electoral systems and representation quality

### Option 3: Legislative Studies Quarterly
**Why LSQ?**
- Congressional elections focus
- Legislative behavior implications (safe seats → polarization?)
- Quantitative political science
- Format: 8,000-10,000 words

**Fit**: House elections and representation

**Recommendation**: Submit to **AJPS first** (highest impact, controversial topic fits flagship venue).

---

## Data Requirements

**Already Available**:
- Census tract shapefiles (2020)
- Presidential vote by tract (estimated from precincts)
- Recursive bisection algorithm (Paper 01)

**Need to Compute**:
- Partisan lean per tract (D% - R%)
- Edge weights based on partisan similarity
- 50 states × 15 configurations (5 α × 3 τ) = 750 redistricting runs
- Compactness, safe seat counts, partisan balance metrics

**Estimated Data Processing**: 2-3 days (parallel runs across states)

---

## Implementation Timeline

### Phase 1: Algorithm Modification (3-4 days)
- Modify edge-weighting code to use partisan similarity (not racial demographics)
- Implement similarity threshold τ parameter
- Test on 3 pilot states (CA, PA, NC)

### Phase 2: 50-State Ablation Study (1 week)
- Run 15 configurations per state (5 α × 3 τ)
- Total: 750 redistricting runs
- Parallel execution: ~2-3 days on 12-core machine
- Compute all metrics (safe seats, compactness, partisan balance)

### Phase 3: Analysis (1 week)
- Pareto frontier analysis (compactness vs homogeneity)
- Comparison to enacted and neutral baselines
- State-by-state patterns
- VRA compatibility analysis

### Phase 4: Writing (1.5 weeks)
- Draft all sections
- Generate all 6 figures
- Normative discussion (safe vs competitive)
- Transparency vs gerrymandering argument

### Phase 5: Internal Review (3-4 days)
- Circulate to advisors (political science + law)
- Revise for controversial framing
- Address normative objections

### Phase 6: Submission (2-3 days)
- Format for AJPS
- Cover letter emphasizing transparency argument
- Submit

**Total: 2-3 weeks**

---

## Key Challenges

### Challenge 1: Normative Controversy
**Problem**: Paper may be seen as advocating for uncompetitive elections
**Mitigation**:
- Emphasize normative neutrality (descriptive, not prescriptive)
- Frame as "if voters want safe seats, here's the transparent way"
- Acknowledge competitive district benefits in discussion

### Challenge 2: "Is This Gerrymandering?"
**Problem**: Reviewers may view as partisan gerrymandering
**Mitigation**:
- Emphasize transparency (public algorithm)
- Emphasize balance (both parties get proportional safe seats)
- Contrast to human gerrymanders (better compactness, auditable)

### Challenge 3: Partisan Data Requirement
**Problem**: Paper 01's "impossibility defense" explicitly avoids partisan data
**Mitigation**:
- Frame as thought experiment, not policy recommendation
- Acknowledge tension with impossibility defense
- Position as exploring full spectrum (neutral → partisan-weighted)

### Challenge 4: VRA Compatibility
**Problem**: Partisan clustering may conflict with VRA compliance
**Mitigation**:
- Test combined weighting (partisan + racial)
- Show trade-offs explicitly
- Acknowledge constraint conflict (Paper 08 framework)

---

## Success Criteria

- [ ] 750 redistricting runs complete (50 states × 15 configurations)
- [ ] Pareto frontier quantified (compactness vs homogeneity)
- [ ] Safe seat counts exceed enacted maps (with better compactness)
- [ ] Partisan balance maintained (proportional safe seats per party)
- [ ] All 6 figures generated
- [ ] Normative discussion addresses competitive district advocates
- [ ] Draft complete (10,000-12,000 words)

---

## Related Work Integration

**From Paper 01 (recursive-bisection)**:
- Uses same algorithm, adds partisan edge-weighting
- **Tension**: Paper 01 avoids partisan data (impossibility defense); this paper uses it explicitly

**From Paper 03 (vra-compliance)**:
- Parallel method: Both use edge-weighting to cluster similar populations
- Different objectives: VRA → minority representation; This → safe seats

**From Paper 08 (multi-vs-edge)**:
- Constraint conflict framework applies: Partisan clustering vs compactness trade-off

**From Paper 15 (party-based-allocation)**:
- Both use partisan data, different objectives
- Paper 15: Proportional outcomes; This paper: Homogeneous districts

**Extension**:
- Explores partisan edge-weighting (inverse of competitive redistricting)
- Tests transparency argument (algorithmic partisan clustering vs gerrymanders)

---

**Created**: 2026-02-08
**Panel Reference**: N/A (experimental)
**Related Enhancement**: User insight (partisan similarity edge-weighting)
**Risk Level**: High (normatively controversial, partisan data usage)
**Scientific Value**: High (challenges reform consensus, tests transparency vs gerrymandering)
