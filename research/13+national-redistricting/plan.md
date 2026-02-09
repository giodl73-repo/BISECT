# National Redistricting Without State Boundaries — Plan

**Artifact Type**: Research Paper (Paper #13 - Experimental)
**Goal**: Establish geometric baseline for redistricting without constitutional state boundary constraints
**Estimated Effort**: 2-3 weeks
**Status**: Planned (Deferred until 3+ core papers published)
**Source**: Enhancement E22

---

## Objective

Create 435 congressional districts nationwide **without respecting state boundaries**, using pure recursive bisection to establish an upper bound for geometric optimization.

**Research Question**: How much more compact could congressional districts be if we removed the constitutional requirement that representatives be apportioned to states?

**Constitutional Note**: This is a **theoretical experiment only**. U.S. Constitution Article I, Section 2 requires representatives be apportioned to states. This paper establishes an academic baseline for comparison, not a policy proposal.

---

## Research Questions

### RQ1: Compactness Upper Bound
**How compact can districts be without state boundaries?**

- Hypothesis: National optimization produces significantly more compact districts (~10-15% improvement)
- Mechanism: State boundaries create geometric constraints that force suboptimal cuts

### RQ2: Natural Regional Emergence
**Do natural geographic regions emerge from pure algorithm?**

- Hypothesis: Algorithm creates districts along natural geographic features (mountains, rivers)
- Alternative: Districts are arbitrary geometric shapes with no coherent identity

### RQ3: State Boundary Crossings
**How many districts would cross state lines?**

- Quantify constitutional incompatibility
- Identify which state boundaries are most constraining (geometric vs political)

### RQ4: Quantifying Federalism's Cost
**What is the geometric cost of respecting state boundaries?**

- Compare national optimization vs 50-state optimization
- Express as % compactness loss
- Academic contribution: First quantification of federalism's geometric cost

---

## Proposed Structure

### Abstract (150 words)
- Problem: Congressional districts must respect state boundaries (constitutional requirement)
- Question: What if they didn't? How much geometric improvement is possible?
- Method: Apply recursive bisection to unified national graph (435 districts, no state constraints)
- Findings: (TBD) National optimization produces X% more compact districts but creates Y cross-state districts
- Contribution: Establishes upper bound for geometric optimization, quantifies cost of federalism

### Section 1: Introduction (700 words)
- **Context**: All redistricting research respects state boundaries
- **Constitutional requirement**: Article I, Section 2 (representatives apportioned to states)
- **Research gap**: No prior work quantifies geometric cost of this constraint
- **This paper**: Theoretical experiment removing state boundaries
- **Value**: Academic baseline for understanding feasibility limits

### Section 2: Constitutional and Historical Context (500 words)
- **Article I, Section 2**: "Representatives...shall be apportioned among the several States"
- **Historical precedent**: 235+ years of state-based apportionment
- **Federalism rationale**: States as political units, not arbitrary geographic regions
- **Why this matters**: Understanding constraints helps evaluate reforms within constitutional bounds

### Section 3: Methodology (800 words)

#### 3.1: National Graph Construction
- Load 85,331 census tracts (2020)
- Build unified adjacency graph (all 50 states)
- Cross-state boundaries treated as normal adjacencies
- Graph size: ~85K nodes, ~240K edges

#### 3.2: National Recursive Bisection
- Target: 435 districts of ~769,000 residents each (2020 population)
- No state boundary constraints
- Population balance: ±0.5% (same as state-based)
- Edge weights: α = 5.0 (for VRA compliance)

#### 3.3: Comparison Baseline
- Compare to 50-state optimization (Paper 01)
- Metrics: Polsby-Popper compactness, perimeter, district sizes

#### 3.4: State Boundary Crossing Analysis
- Count districts that cross state lines
- Quantify "constitutional incompatibility"
- Map showing cross-state districts

### Section 4: Results (1,500 words)

#### 4.1: Compactness Improvement
**Table 1**: National vs State-Based Compactness

| Method | Mean PP | Median PP | Std Dev | Range |
|--------|---------|-----------|---------|-------|
| National | TBD | TBD | TBD | TBD |
| State-based | 0.461 | 0.456 | 0.083 | 0.18-0.82 |
| **Improvement** | **+X%** | **+Y%** | — | — |

**Hypothesis**: ~10-15% improvement (national optimization removes state boundary constraints)

#### 4.2: State Boundary Crossings
**Finding**: Z out of 435 districts cross state lines

**Table 2**: States with most cross-state districts
- Example: New England states (small, dense) likely generate many cross-state districts
- Example: Large states (TX, CA) may have mostly contained districts

#### 4.3: Geographic Patterns
**Figure 1**: National map showing all 435 districts
- Color districts randomly
- Highlight state boundaries in bold
- Annotate cross-state districts

**Figure 2**: Regional zooms (New England, Four Corners, etc.)
- Show how algorithm ignores state lines
- Identify natural geographic features (mountains, rivers) that influence cuts

#### 4.4: District Size Variability
- National optimization: Perfect population equality (±0.5%)
- State-based: Sometimes forced into suboptimal splits (small states, odd numbers)

### Section 5: Discussion (1,200 words)

#### 5.1: Geometric Cost of Federalism
**Key Finding**: State boundaries impose X% compactness cost

**Interpretation**:
- If X < 5%: Federalism imposes minimal geometric cost (state boundaries align with natural regions)
- If X > 15%: Federalism imposes significant cost (arbitrary political boundaries constrain optimization)

#### 5.2: Natural Regions
**Do algorithmic districts follow natural geography?**
- Qualitative analysis of cross-state districts
- Check if districts follow rivers, mountain ranges, cultural regions
- Academic question: Is there a "natural" congressional geography beyond states?

#### 5.3: Constitutional Implications
**What does this mean for reform?**
- National optimization is constitutionally infeasible (requires amendment)
- BUT: Quantifies upper bound for within-state reforms
- Policy relevance: States can achieve ~90% of theoretical maximum without constitutional change

#### 5.4: Limitations
- **Single algorithm**: Only recursive bisection tested (ensemble methods may differ)
- **2020 only**: Single census year (temporal trends unknown)
- **Theoretical only**: Not a policy proposal (constitutional infeasibility)

#### 5.5: Future Work
- Multi-year national optimization (2000/2010/2020)
- Ensemble methods (ReCom, spectral)
- Partial relaxation (allow cross-state districts in specific regions?)

### Section 6: Conclusion (400 words)
- Summary: National optimization establishes X% geometric upper bound
- Academic value: Quantifies federalism's cost for first time
- Policy implication: Within-state reforms can achieve ~90% of theoretical maximum
- Future: Extend to multi-year, alternative algorithms

---

## Figures (4 total)

**Figure 1: National Districts Map**
- 435 districts across all 50 states
- State boundaries in bold black
- Cross-state districts highlighted in red
- Insets for AK/HI

**Figure 2: Regional Zooms (4 panels)**
- Panel A: New England (dense cross-state activity)
- Panel B: Four Corners (NM/AZ/CO/UT intersection)
- Panel C: DMV (DC/MD/VA metro region)
- Panel D: Twin Cities (MN/WI border)

**Figure 3: Compactness Comparison**
- Box plots: National vs State-based
- Shows distribution of Polsby-Popper scores
- Annotate X% improvement

**Figure 4: State Boundary Crossing Analysis**
- Bar chart: Number of cross-state districts per state
- States sorted by count (descending)
- Shows which states are most affected by national optimization

---

## Target Venues

### Option 1: Comparative Political Studies
**Why Comparative Politics?**
- Cross-national institutional comparison
- Federalism vs unitary state analysis
- Political geography focus
- Format: 8,000-10,000 words

**Fit**: Compares federal (US) vs hypothetical unitary system

### Option 2: Political Geography
**Why Political Geography?**
- Spatial analysis of political institutions
- Natural regions vs political boundaries
- Geographic cost of institutions
- Format: 7,000-9,000 words

**Fit**: Geographic focus, natural region emergence

### Option 3: Legislative Studies Quarterly
**Why LSQ?**
- Congressional institutions focus
- Representation and redistricting
- Thought experiments welcome
- Format: 8,000-10,000 words

**Fit**: Congressional representation, institutional analysis

**Recommendation**: Submit to **Comparative Political Studies first** (best fit for federalism cost analysis).

---

## Computational Requirements

### Data Volume
- Census tracts: 85,331 (2020)
- Adjacency edges: ~240,000
- Memory: ~16GB RAM for full graph

### Processing Time
- Graph construction: 1-2 hours (detecting all cross-state adjacencies)
- METIS partitioning: 2-4 hours (435 districts from 85K tracts)
- Visualization: 1-2 hours (national map rendering)

**Total runtime**: ~1 day

### Computational Challenges
1. **Cross-state adjacency detection**: Must handle 50 separate state geometries
2. **Graph size**: Larger than any single-state graph (but smaller than 50-state parallel run)
3. **Visualization density**: 435 districts on one map is cluttered

---

## Implementation Timeline

### Phase 1: Data Preparation (3-4 days)
- Load all 50 state tract data
- Build unified national adjacency graph
- Validate connectivity (single connected component)

### Phase 2: National Redistricting (2-3 days)
- Run METIS on national graph (435 districts)
- Validate population balance
- Export district assignments

### Phase 3: Analysis (1 week)
- Compute compactness metrics
- Identify cross-state districts
- Compare to state-based baseline (Paper 01)
- Generate comparison tables

### Phase 4: Visualization (3-4 days)
- National map with all districts
- Regional zoom panels
- Compactness comparison charts
- State boundary crossing analysis

### Phase 5: Writing (1 week)
- Draft all sections
- Integrate figures and tables
- Constitutional context section

### Phase 6: Internal Review (3-4 days)
- Circulate to advisors
- Revise based on feedback

### Phase 7: Submission (2-3 days)
- Format for target venue
- Cover letter emphasizing theoretical contribution
- Submit

**Total: 2-3 weeks**

---

## Key Challenges

### Challenge 1: Constitutional Sensitivity
**Problem**: Readers may misinterpret as policy proposal
**Mitigation**:
- Prominent constitutional disclaimer (abstract, intro, conclusion)
- Frame as "theoretical upper bound" not "recommended approach"
- Emphasize academic value (quantifying constraints)

### Challenge 2: Cross-State Adjacency
**Problem**: Detecting which tracts border other states
**Mitigation**:
- Use state boundary shapefiles
- Intersect with tract geometries
- Pre-compute cross-state adjacency list

### Challenge 3: Visualization Clutter
**Problem**: 435 districts on one map is hard to read
**Mitigation**:
- Random colors (no other option for 435)
- Regional zoom insets
- Interactive web version (optional)

---

## Success Criteria

- [ ] National adjacency graph constructed (85,331 tracts)
- [ ] METIS produces exactly 435 districts
- [ ] All districts within ±0.5% population target
- [ ] Compactness improvement quantified (X%)
- [ ] Cross-state districts identified and counted
- [ ] All 4 figures generated
- [ ] Constitutional disclaimer prominently featured
- [ ] Draft complete (8,000-10,000 words)

---

## Related Work Integration

**From Paper 01 (recursive-bisection)**:
- Uses same algorithm, removes state boundary constraint

**From Paper 02 (edge-weighted-bisection)**:
- Edge-weighting still applies (VRA compliance maintained)

**Comparison**:
- Establishes upper bound that Papers 01-10 operate below
- Quantifies geometric cost of constitutional federalism

---

## Next Actions

- [ ] Load all 50 state tract geometries
- [ ] Build unified national adjacency graph
- [ ] Detect cross-state adjacencies (validate graph)
- [ ] Run METIS on national graph (435 districts)
- [ ] Compute compactness metrics
- [ ] Compare to state-based baseline
- [ ] Generate national district map
- [ ] Draft paper

---

**Created**: 2026-02-08
**Panel Reference**: N/A (experimental)
**Related Enhancement**: E22 (National Redistricting)
**Deferral**: Wait until 3+ core papers published (Wave F07 guidance)
**Risk Level**: Low (theoretical experiment, clear academic framing)
**Scientific Value**: High (first quantification of federalism's geometric cost)
