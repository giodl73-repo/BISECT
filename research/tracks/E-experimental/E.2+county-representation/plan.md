# Direct County Representation — Plan

**Artifact Type**: Research Paper (Paper #14 - Experimental)
**Goal**: Test whether existing county boundaries could serve as redistricting units
**Estimated Effort**: 2 weeks
**Status**: Planned
**Source**: Enhancement E23

---

## Objective

Explore an alternative representation system: **allocate representatives directly to counties based on population**, eliminating the need for artificial district boundaries.

**Research Question**: What if we used existing political boundaries (counties) instead of creating new districts? How would representation quality compare to algorithmic redistricting?

**Rationale**: Counties are established political units with administrative functions, cultural identity, and voter familiarity. Using counties could eliminate gerrymandering entirely while respecting existing geographic communities.

---

## Research Questions

### RQ1: Population Match
**How well do county populations align with congressional district targets?**

- Target: ~769,000 residents per representative (2020)
- Most counties too small (need fractional representatives)
- Few counties too large (need multiple representatives)
- **Feasibility**: Can we allocate fractional representatives sensibly?

### RQ2: Representation Quality
**Would county-based representation be more/less compact than algorithmic districts?**

- Counties often irregular shapes (historical boundaries)
- Some counties very compact, others sprawling
- Compare mean county compactness to algorithmic district compactness

### RQ3: Political Consequences
**How would county-based allocation affect partisan representation?**

- Urban counties (dense, Democratic) vs rural counties (sparse, Republican)
- Potential bias toward rural areas if using integer representatives
- Weighted voting in House as solution?

### RQ4: Historical Precedent
**How did county-based representation work historically?**

- Many state legislatures used county-based systems (pre-Reynolds v. Sims 1964)
- Declared unconstitutional (one person, one vote)
- BUT: Could weighted voting satisfy equal protection?

---

## Proposed Structure

### Abstract (150 words)
- Problem: Redistricting creates artificial boundaries prone to gerrymandering
- Alternative: Use existing county boundaries, allocate representatives proportionally
- Method: Compute county populations, allocate fractional representatives, compare outcomes
- Findings: (TBD) County-based system eliminates gerrymandering but creates X% representation inequality
- Contribution: First rigorous analysis of county-based federal representation

### Section 1: Introduction (600 words)
- **Problem**: Redistricting is contentious, expensive, litigation-prone
- **Root cause**: Drawing arbitrary boundaries invites manipulation
- **Alternative**: Use existing boundaries (counties)
- **This paper**: Feasibility analysis of county-based congressional representation
- **Value**: Eliminates redistricting entirely (no boundary drawing)

### Section 2: Historical Context (700 words)

#### 2.1: County-Based State Legislatures
- Pre-1964: Many states allocated senators by county (Tennessee, Vermont)
- Reynolds v. Sims (1964): Struck down as unconstitutional (violates equal protection)
- "One person, one vote" requires population-based representation

#### 2.2: The Weighted Voting Solution
- Counties get representatives proportional to population
- Representatives cast fractional votes in House
- Example: County with 384,500 residents gets 0.5 representatives, casts 0.5 votes
- **Constitutional question**: Does weighted voting satisfy equal protection?

#### 2.3: International Comparisons
- Switzerland: Canton-based representation with weighted voting (Council of States)
- Germany: Länder have unequal representation in Bundesrat
- European Parliament: Degressive proportionality by country

### Section 3: Methodology (800 words)

#### 3.1: County-Level Data
- 3,143 counties in U.S. (2020)
- Population range: Loving County, TX (64 residents) to Los Angeles County, CA (10.0M)
- Mean county population: ~105,000

#### 3.2: Representative Allocation
**Formula**: County *i* gets $r_i = \frac{pop_i}{769,000}$ representatives

**Implementation**:
- Option A: Integer representatives only (round to nearest)
- Option B: Fractional representatives with weighted voting
- Option C: Hybrid (large counties get integers, small counties share)

#### 3.3: Comparison Metrics
- **Representation equality**: Deviation from ideal population per representative
- **Compactness**: Use county geometries (Polsby-Popper)
- **Partisan outcomes**: Aggregate votes by county, compute seat-vote curves
- **Communities of interest**: Counties as natural COI units

### Section 4: Results (1,200 words)

#### 4.1: Representative Allocation by Option

**Table 1**: County-based allocation statistics

| Option | Total Reps | Mean Deviation | Max Deviation | Counties with <1 rep | Counties with >1 rep |
|--------|------------|----------------|---------------|----------------------|----------------------|
| Integer only | 435 | TBD | TBD | ~2,900 | ~240 |
| Fractional | 435.0 | 0% (exact) | 0% | 0 | 0 |
| Hybrid | 435 | TBD | TBD | TBD | TBD |

**Finding**: Fractional system achieves perfect representation equality, but requires House procedural changes

#### 4.2: Compactness Comparison
**Table 2**: County vs Algorithmic Compactness

| System | Mean PP | Median PP | Std Dev |
|--------|---------|-----------|---------|
| Counties | TBD | TBD | TBD |
| Algorithmic | 0.461 | 0.456 | 0.083 |

**Hypothesis**: Counties slightly less compact (historical boundaries, not optimized)

#### 4.3: Partisan Outcomes
**Analysis**: Aggregate 2020 presidential vote by county
- Urban mega-counties (LA, Cook, Harris) heavily Democratic, get many representatives
- Rural counties mostly Republican, get fractional representatives
- **Net effect**: TBD (depends on urban vs rural population distribution)

#### 4.4: Gerrymandering Elimination
**Key advantage**: County boundaries are fixed, cannot be manipulated
- No redistricting litigation
- No partisan map-drawing
- Transparent, permanent system

### Section 5: Discussion (1,000 words)

#### 5.1: Constitutional Feasibility
**Weighted voting question**:
- Does fractional vote-casting satisfy "one person, one vote"?
- Each person's vote still counts equally (county aggregate weight proportional)
- Precedent: Shareholder voting (proportional to shares)

**Legal analysis**:
- Likely requires Supreme Court case (no precedent for federal weighted voting)
- May require constitutional amendment (Article I, Section 2 interpretation)

#### 5.2: Practical Implementation
**House procedural changes**:
- Electronic voting systems (fractional vote counting)
- Committee assignments (fractional members?)
- Quorum calculations (fractional thresholds)

**Feasibility**: Technically straightforward, politically challenging

#### 5.3: Advantages Over Algorithmic Redistricting
1. **Eliminates redistricting**: No decadal boundary disputes
2. **Respects existing communities**: Counties are established political units
3. **Transparent**: No algorithm complexity
4. **Dynamic**: Automatically adjusts to population shifts (re-weight annually?)

#### 5.4: Disadvantages
1. **Weighted voting novelty**: No federal precedent
2. **County inequality**: Some counties poorly designed (historical artifacts)
3. **No VRA compliance**: Counties may dilute minority voting strength
4. **Prisoner populations**: Counties with prisons get inflated populations (unless adjusted)

#### 5.5: Future Work
- Test with alternative units (municipalities, media markets)
- Analyze VRA compliance implications
- Survey public attitudes (weighted voting acceptability)

### Section 6: Conclusion (400 words)
- Summary: County-based representation feasible with weighted voting
- Trade-off: Eliminates gerrymandering but requires procedural innovation
- Recommendation: Consider for future constitutional amendments
- Value: Thought experiment reveals redistricting alternatives

---

## Figures (4 total)

**Figure 1: County Population Distribution**
- Histogram: County populations (log scale)
- Show target 769K threshold
- Annotate mega-counties (LA, Cook, Harris) and micro-counties (Loving, etc.)

**Figure 2: Representative Allocation Map**
- Choropleth: Counties colored by representative count
- Gradient: 0 (white) → 0.5 (light) → 1.0 (medium) → 10+ (dark)
- Shows concentration in urban areas

**Figure 3: Compactness Comparison**
- Box plots: Counties vs Algorithmic Districts
- Shows distribution of Polsby-Popper scores

**Figure 4: Urban-Rural Representation**
- Scatter plot: County population (X) vs Partisan lean (Y)
- Point size = representative count
- Shows concentration of Democratic representatives in large urban counties

---

## Target Venues

### Option 1: State Politics & Policy Quarterly
**Why State Politics?**
- County-level governance focus
- State legislative representation (similar historical questions)
- Federalism and local boundaries
- Format: 7,000-9,000 words

**Fit**: Counties as state-level political units

### Option 2: Political Research Quarterly
**Why PRQ?**
- Representation theory
- Institutional design alternatives
- Thought experiments welcome
- Format: 8,000-10,000 words

**Fit**: Representation system innovation

### Option 3: Electoral Studies
**Why Electoral Studies?**
- Voting systems and representation
- Comparative electoral institutions
- Weighted voting precedents (international)
- Format: 8,000 words

**Fit**: Electoral system design, international comparisons

**Recommendation**: Submit to **State Politics & Policy Quarterly first** (best fit for county-level analysis).

---

## Data Requirements

**Already Available**:
- County populations (2020 Census)
- County boundaries (TIGER/Line shapefiles)
- Presidential vote by county (Dave Leip's Atlas)

**Need to Compute**:
- County compactness (Polsby-Popper)
- Representative allocations (fractional and integer)
- Partisan seat-vote curves
- Comparison to algorithmic districts

**Estimated Data Processing**: 3-4 days

---

## Implementation Timeline

### Phase 1: Data Collection (2-3 days)
- Download county shapefiles and population data
- Compute county compactness scores
- Allocate representatives (fractional, integer, hybrid)

### Phase 2: Analysis (1 week)
- Compare representation equality across options
- Partisan outcome analysis
- VRA compliance check (COI preservation)
- Generate comparison tables

### Phase 3: Writing (1 week)
- Draft all sections
- Constitutional analysis section
- Historical precedent research

### Phase 4: Review & Submission (3-4 days)
- Internal review
- Revise and format
- Submit

**Total: 2 weeks**

---

## Success Criteria

- [ ] County representative allocations computed (3 options)
- [ ] Compactness comparison complete
- [ ] Partisan outcome analysis done
- [ ] Constitutional feasibility assessed
- [ ] All 4 figures generated
- [ ] Draft complete (7,000-9,000 words)

---

## Related Work Integration

**From Paper 01 (recursive-bisection)**:
- Provides algorithmic baseline for comparison

**From Paper 03 (vra-compliance)**:
- Compare VRA outcomes (algorithmic vs county-based)

**Contrast**:
- Algorithmic: Optimizes geometry, requires redistricting every 10 years
- County-based: Uses existing boundaries, requires weighted voting innovation

---

## Next Actions

- [ ] Download 2020 county shapefiles and population data
- [ ] Compute county compactness scores (Polsby-Popper)
- [ ] Allocate fractional representatives to counties
- [ ] Compare to algorithmic districts (Paper 01 baseline)
- [ ] Analyze partisan outcomes
- [ ] Draft introduction and constitutional analysis sections

---

**Created**: 2026-02-08
**Panel Reference**: N/A (experimental)
**Related Enhancement**: E23 (County Representation)
**Risk Level**: Medium (requires constitutional analysis expertise)
**Scientific Value**: Medium-High (novel alternative to redistricting)
