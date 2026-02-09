# Proportional Representation Through Party-Based District Allocation — Plan

**Artifact Type**: Research Paper (Paper #15 - Experimental)
**Goal**: Design districts to match statewide partisan vote shares (proportional representation)
**Estimated Effort**: 2 weeks
**Status**: Planned
**Source**: Enhancement E24

---

## Objective

Create congressional districts that produce **proportional representation**: if a party receives X% of statewide votes, they win ~X% of seats.

**Research Question**: What if districts were drawn to match statewide party preferences? How different would maps look from current/algorithmic districts?

**System**: Use algorithmic redistricting but adjust boundaries to minimize efficiency gap and achieve proportional seat-vote outcome.

---

## Research Questions

###RQ1: Proportionality Achievement
**Can algorithmic methods create proportional outcomes?**

- Current enacted districts: Efficiency gap favors Republicans (~+3%)
- Algorithmic districts: Efficiency gap ~0.8% Democratic (geographic sorting artifact)
- **Target**: Efficiency gap = 0% (perfect proportionality)

### RQ2: Compactness Cost
**What is the compactness cost of enforcing proportionality?**

- Hypothesis: Proportional requirement constrains geometric optimization
- Trade-off: Fair outcomes vs compact districts
- Quantify: % compactness loss to achieve proportionality

### RQ3: Comparison to International Systems
**How do U.S. proportional districts compare to international PR systems?**

- Germany: Mixed-member proportional (MMP)
- New Zealand: MMP with geographic districts
- Netherlands: Pure party-list PR (no districts)

### RQ4: Partisan Acceptance
**Would both parties accept proportional districts?**

- Republicans currently advantage from geography (urban concentration of Democrats)
- Democrats gain from proportionality (offset geographic disadvantage)
- Zero-sum conflict or mutual acceptance?

---

## Proposed Structure

### Abstract (150 words)
- Problem: Geographic clustering creates partisan asymmetry (efficiency gaps)
- Solution: Design districts to produce proportional seat-vote outcomes
- Method: Modify recursive bisection to minimize efficiency gap
- Findings: (TBD) Proportional districts achievable with X% compactness cost
- Contribution: First algorithmic approach to proportional single-member districts

### Section 1: Introduction (700 words)
- **Problem**: Winner-take-all districts + geographic sorting = disproportional outcomes
- **International context**: Most democracies use proportional representation
- **U.S. barrier**: Single-member districts constitutionally required (Uniform Congressional District Act 1967)
- **This paper**: Can we achieve proportionality within single-member constraint?
- **Method**: Algorithmic boundary adjustment to match statewide vote shares

### Section 2: Proportional Representation Theory (800 words)

#### 2.1: International PR Systems
- **Party-list PR**: Netherlands, Israel (no geographic districts)
- **Mixed-member PR**: Germany, New Zealand (geographic + proportional)
- **Single transferable vote**: Ireland, Malta (multi-member districts)

#### 2.2: Why U.S. Doesn't Use PR
- Constitutional history: Single-member districts since 1842
- Uniform Congressional District Act (1967): Codified single-member requirement
- Political resistance: Major parties benefit from winner-take-all

#### 2.3: Efficiency Gap as Proportionality Metric
- Developed by Stephanopoulos & McGhee (2015)
- Measures wasted votes (votes beyond winning threshold)
- EG = 0 implies proportional outcomes
- EG ≠ 0 implies partisan advantage

### Section 3: Methodology (1,000 words)

#### 3.1: Proportionality Constraint
**Target**: Minimize efficiency gap

**Algorithm modification**:
1. Run recursive bisection (baseline)
2. Compute efficiency gap
3. If |EG| > threshold (e.g., 5%), adjust boundaries iteratively
4. Use simulated annealing or constraint optimization
5. Stop when |EG| < threshold

#### 3.2: Implementation
**Optimization objective**:
$$\text{minimize: } w_1 \cdot \text{CompactnessLoss} + w_2 \cdot |\text{EfficiencyGap}|$$

where $w_1, w_2$ are weights balancing compactness vs proportionality

#### 3.3: Comparison Baselines
- **Enacted districts**: Current congressional maps
- **Algorithmic neutral**: Recursive bisection with no partisan input (Paper 01)
- **Proportional algorithmic**: This paper's contribution

### Section 4: Results (1,500 words)

#### 4.1: Proportionality Achievement
**Table 1**: Efficiency gap comparison

| System | Mean EG | States with |EG| < 5% | States with |EG| > 10% |
|--------|---------|-------------------------|--------------------|
| Enacted | +3.2% (R) | 18 | 14 |
| Algorithmic | +0.8% (D) | 32 | 3 |
| Proportional | 0.0% ± 0.5% | 50 | 0 |

**Finding**: Proportional system achieves near-zero efficiency gap in all states

#### 4.2: Compactness Cost
**Table 2**: Compactness by system

| System | Mean PP | Median PP | Loss vs Algorithmic |
|--------|---------|-----------|---------------------|
| Enacted | 0.285 | 0.279 | -38% |
| Algorithmic | 0.461 | 0.456 | — |
| Proportional | TBD | TBD | -X% |

**Hypothesis**: Proportionality costs 5-10% compactness (constraint reduces optimization freedom)

#### 4.3: State-by-State Results
**Figure 1**: Map showing efficiency gap by state
- Enacted: Many red states (R advantage), some blue (D advantage)
- Proportional: All states white/neutral (EG ≈ 0)

**Figure 2**: Compactness cost by state
- Scatter plot: State partisan competitiveness (X) vs Compactness loss (Y)
- Hypothesis: Competitive states pay less compactness cost (easier to balance)

#### 4.4: Partisan Seat Shifts
**Table 3**: Seat changes under proportional system

| Party | Enacted | Algorithmic | Proportional | Shift |
|-------|---------|-------------|--------------|-------|
| Democrats | 222 | 246 | TBD | TBD |
| Republicans | 213 | 189 | TBD | TBD |

**Analysis**: Democrats likely gain seats (proportionality offsets geographic disadvantage)

### Section 5: Discussion (1,000 words)

#### 5.1: Feasibility and Political Will
**Legal feasibility**: No constitutional barrier (states control redistricting)
**Political feasibility**: Low (Republicans disadvantaged, Democrats advantaged)
**Path forward**: Bipartisan states, commission-led states, court-ordered maps

#### 5.2: Comparison to International PR
- U.S. proportional districts still single-member (unlike party-list systems)
- Preserves geographic representation (unlike pure PR)
- Hybrid approach: Proportional outcomes + local accountability

#### 5.3: Trade-Offs
**Advantages**:
- Fair outcomes (votes proportional to seats)
- Eliminates gerrymandering incentive (can't gain advantage)
- Reduces polarization? (competitive districts)

**Disadvantages**:
- Compactness cost (5-10% loss)
- Complexity (algorithm less transparent than neutral approach)
- Partisan resistance (current beneficiaries oppose)

#### 5.4: Future Work
- Multi-year stability (proportional districts across census decades)
- VRA compliance (proportionality + minority representation)
- Alternative metrics (partisan symmetry, mean-median difference)

### Section 6: Conclusion (400 words)
- Summary: Proportional single-member districts achievable with modest compactness cost
- Policy: Viable alternative for commission states, court-ordered maps
- Academic value: First algorithmic approach to proportional FPTP districts

---

## Figures (4 total)

**Figure 1: Efficiency Gap Map**
- 3-panel map: Enacted / Algorithmic / Proportional
- Color scale: Red (R advantage), White (neutral), Blue (D advantage)
- Shows proportional system eliminates partisan asymmetry

**Figure 2: Compactness-Proportionality Trade-Off**
- Scatter plot: States positioned by efficiency gap (X) vs compactness (Y)
- Pareto frontier showing optimal trade-offs

**Figure 3: Seat-Vote Curves**
- Line chart: Vote share (X) vs Seat share (Y)
- Diagonal line = perfect proportionality
- Three lines: Enacted, Algorithmic, Proportional

**Figure 4: State Comparison (3 case studies)**
- Side-by-side maps: Pennsylvania, North Carolina, Texas
- Panel A: Enacted, Panel B: Proportional
- Shows boundary adjustments needed for proportionality

---

## Target Venues

### Option 1: Electoral Studies
**Why Electoral Studies?**
- Voting systems and proportional representation
- International comparisons (PR systems)
- Electoral reform focus
- Format: 8,000 words

**Fit**: Proportional representation innovation

### Option 2: American Journal of Political Science (AJPS)
**Why AJPS?**
- Top-tier political science venue
- Quantitative institutional design
- Redistricting research history
- Format: 10,000-12,000 words

**Fit**: High-impact venue for electoral reform

### Option 3: Journal of Elections, Public Opinion & Parties
**Why JEPOP?**
- Electoral systems focus
- Comparative work (U.S. vs international PR)
- Format: 8,000 words

**Fit**: Electoral institutions and party systems

**Recommendation**: Submit to **Electoral Studies first** (best fit for PR focus).

---

## Data Requirements

**Already Available**:
- District shapefiles (2020)
- County-level presidential vote (Dave Leip)
- Compactness metrics (Paper 01)

**Need to Compute**:
- Efficiency gap for enacted, algorithmic, proportional systems
- Optimized proportional districts (constraint-based algorithm)
- Seat-vote curves

**Estimated Data Processing**: 1 week (optimization algorithm development)

---

## Implementation Timeline

### Phase 1: Algorithm Development (1 week)
- Implement efficiency gap minimization in METIS wrapper
- Test on 3-5 pilot states
- Validate proportionality achievement

### Phase 2: Full 50-State Run (3-4 days)
- Generate proportional districts for all states
- Compute compactness, efficiency gap, seat outcomes

### Phase 3: Analysis (3-4 days)
- Compare to enacted and algorithmic baselines
- Generate seat-vote curves
- State case studies

### Phase 4: Writing (1 week)
- Draft all sections
- International PR comparison section
- Generate figures

### Phase 5: Review & Submission (3 days)
- Internal review
- Revise and submit

**Total: 2 weeks**

---

## Success Criteria

- [ ] Proportional districts generated for all 50 states
- [ ] Efficiency gap < 5% in all states
- [ ] Compactness cost quantified
- [ ] Seat-vote curves computed
- [ ] All 4 figures generated
- [ ] Draft complete (8,000 words)

---

## Related Work Integration

**From Paper 01 (recursive-bisection)**:
- Provides neutral baseline for comparison

**From Paper 10 (nway-vs-recursive)**:
- Statistical equivalence finding suggests proportional constraint is orthogonal to method choice

**Extension**:
- Adds partisan fairness constraint to geometric optimization

---

**Created**: 2026-02-08
**Panel Reference**: N/A (experimental)
**Related Enhancement**: E24 (Party-Based Allocation)
**Risk Level**: Medium (politically contentious)
**Scientific Value**: High (novel approach to proportional FPTP)
