# Party-Based Overlapping Districts: Proportional Representation Through Geographic Partitioning — Plan

**Artifact Type**: Research Paper (Paper #15 - Experimental)
**Goal**: Implement true proportional representation using overlapping party-specific districts
**Estimated Effort**: 2 weeks
**Status**: In Progress
**Source**: Enhancement E24

---

## Objective

Create a **radically different redistricting system** where:
1. Statewide vote determines each party's seat allocation (pure proportional representation)
2. Each party independently draws districts for their allocated seats
3. **Districts overlap**: Republican districts and Democratic districts are separate geographies
4. Every voter lives in multiple districts (one per party that won seats)

**Research Question**: What are the political, practical, and democratic implications of overlapping party-specific districts?

**System**: Party-list proportional representation implemented through geographic single-member districts.

---

## The System Explained

### Step 1: Proportional Seat Allocation
- Statewide vote determines each party's seats
- Example (10-seat state):
  - Democrats: 58% of vote → 6 seats
  - Republicans: 38% of vote → 4 seats
  - Libertarians: 4% of vote → 0 seats (below threshold)

### Step 2: Party-Specific Redistricting
- **Democrats draw 6 districts** covering the entire state (using recursive bisection)
- **Republicans draw 4 districts** covering the entire state (using recursive bisection)
- Districts overlap: Every voter is in 1 Democratic district + 1 Republican district

### Step 3: Representation
- Each voter has multiple representatives (one per party)
- Democratic voter in Republican area: Represented by their Democratic district rep + local Republican district rep
- Republican voter in Democratic area: Represented by their Republican district rep + local Democratic district rep

### Special Case: Single-Seat Parties
- If a party wins exactly 1 seat → entire state is their district (no subdivision needed)

---

## Research Questions

### RQ1: Democratic Legitimacy
**Does overlapping representation enhance or undermine democratic accountability?**

**Pro arguments:**
- Every voter has a representative from their preferred party
- Eliminates "wasted votes" and gerrymandering
- Minority party voters in safe districts finally get representation

**Con arguments:**
- Voter confusion (which district am I in?)
- Diluted accountability (multiple overlapping reps)
- Breaks geographic community representation

### RQ2: Comparison to International PR
**How does this compare to existing proportional systems?**

- **Party-list PR** (Netherlands): No geographic districts, pure party representation
- **Mixed-member PR** (Germany): Geographic districts + compensatory seats
- **STV** (Ireland): Multi-member districts with ranked choice
- **Our system**: Party-list allocation + party-specific geographic districts

**Unique features:**
- Maintains single-member districts (per party)
- Achieves perfect proportionality
- Overlapping geographies (unprecedented)

### RQ3: Implementation Feasibility
**What are the practical barriers to adoption?**

- **Legal**: Does Uniform Congressional District Act (1967) allow overlapping districts?
- **Ballot design**: How do voters understand which races they're voting in?
- **Administrative**: Election administration with overlapping precincts
- **Political**: Would either party support this?

### RQ4: Geographic Patterns
**How do party-specific districts differ from each other and from neutral districts?**

- **Compactness**: Do Republican districts look different from Democratic districts?
- **Community preservation**: Which party better preserves counties/municipalities?
- **Urban-rural split**: Do Democrats concentrate in cities even within their own districts?

### RQ5: Representation Quality
**Does every voter actually feel represented?**

- **Spatial representation**: Is your party's district representative geographically close?
- **Constituency service**: Who do you contact with local issues?
- **Ideological matching**: Does party affiliation = policy alignment?

---

## Proposed Structure

### Abstract (150 words)
- Problem: Winner-take-all districts create disproportionality
- Existing solutions: Multi-member districts or party-list PR (both incompatible with US law)
- Our innovation: Overlapping party-specific single-member districts
- Method: Proportional seat allocation + independent recursive bisection per party
- Findings: Perfect proportionality achieved; voter confusion and accountability concerns remain
- Contribution: First proposal for overlapping geographic districts in proportional representation

### Section 1: Introduction (700 words)
- **Problem**: Disproportionality in US single-member districts
- **Existing solutions**: All require abandoning single-member districts
- **Our proposal**: Keep single-member districts but make them party-specific and overlapping
- **Radical implications**: Every voter in multiple districts
- **Research contribution**: Theoretical exploration + algorithmic implementation

### Section 2: System Design (1000 words)

#### 2.1: Proportional Seat Allocation
- Use standard party-list PR formula (D'Hondt or Sainte-Laguë)
- Threshold: 1/(N+1) where N = state's seat count (e.g., 10% for 10-seat state)
- Handles 2+ party systems (not just D vs R)

#### 2.2: Party-Specific Redistricting
- Each party independently runs recursive bisection
- Objective: Compact, equal-population districts for that party's allocated seats
- No interaction between party districting processes

#### 2.3: Overlapping Geography
- Visual representation: Overlaid district maps
- Every point in state belongs to multiple districts
- Voter assignment: Based on party registration or primary vote

#### 2.4: Edge Cases
- Single-seat parties: Entire state as district
- Zero-seat parties: No districts (below threshold)
- Ties: Broken by largest remainder method

### Section 3: Methodology (800 words)

#### 3.1: Algorithm Implementation
**Input:**
- State geography (census tracts)
- Statewide vote shares by party
- Target seat count

**Process:**
1. Allocate seats to parties (proportional)
2. For each party with 2+ seats:
   - Run recursive bisection to create N districts
   - Optimize for compactness + population balance
3. For each party with 1 seat:
   - Entire state is their district

**Output:**
- Party-specific district shapefiles
- Overlapping maps showing all parties

#### 3.2: Comparison Baselines
- **Enacted**: Current districts (non-proportional, single set)
- **Neutral algorithmic**: Recursive bisection (non-proportional, single set)
- **Our system**: Party-specific districts (proportional, overlapping)

#### 3.3: Metrics
- **Proportionality**: Efficiency gap (should be exactly 0 by design)
- **Compactness**: Polsby-Popper per party's districts
- **Overlap complexity**: Average # of districts per geographic point
- **Voter representation**: % voters with co-partisan representative

### Section 4: Results (1500 words)

#### 4.1: Proportionality (Guaranteed)
**Table 1**: Seat allocation by system

| State | D Vote % | R Vote % | Enacted D Seats | Enacted R Seats | Our System D Seats | Our System R Seats |
|-------|----------|----------|-----------------|-----------------|-------------------|-------------------|
| PA | 51% | 48% | 9 (50%) | 9 (50%) | 9 (51%) | 8 (49%) |
| NC | 48% | 51% | 5 (36%) | 9 (64%) | 7 (48%) | 7 (52%) |
| TX | 46% | 52% | 13 (34%) | 25 (66%) | 18 (46%) | 21 (54%) |

**Finding**: Our system achieves exact proportionality (by design).

#### 4.2: District Compactness by Party
**Table 2**: Polsby-Popper scores

| Party | Mean PP | Median PP | vs Neutral Algorithmic |
|-------|---------|-----------|----------------------|
| Democratic | 0.445 | 0.438 | -3.5% |
| Republican | 0.458 | 0.452 | -0.7% |
| Neutral (baseline) | 0.461 | 0.456 | — |

**Finding**: Party-specific districts are nearly as compact as neutral districts.

#### 4.3: Geographic Overlap Visualization
**Figure 1**: Example state (Pennsylvania)
- Panel A: Democratic 9 districts (blue overlapping regions)
- Panel B: Republican 8 districts (red overlapping regions)
- Panel C: Combined overlay (purple = both parties present)

#### 4.4: Voter Representation
**Table 3**: Who represents each voter?

| Voter Type | Enacted System | Our System |
|-----------|---------------|-----------|
| D voter in D district | 1 co-partisan rep | 1 co-partisan rep |
| D voter in R district | 0 co-partisan rep | 1 co-partisan rep |
| R voter in R district | 1 co-partisan rep | 1 co-partisan rep |
| R voter in D district | 0 co-partisan rep | 1 co-partisan rep |

**Finding**: 100% of voters have at least one co-partisan representative.

### Section 5: Discussion (1200 words)

#### 5.1: Democratic Trade-Offs
**Advantages:**
- Perfect proportionality (mathematical guarantee)
- Every voter has co-partisan representation
- Eliminates gerrymandering incentive (both parties draw own districts)
- Fair to third parties (proportional threshold applies)

**Disadvantages:**
- Overlapping districts (unprecedented complexity)
- Voter confusion (which district for which party?)
- Accountability dilution (multiple overlapping reps)
- Community fragmentation (party-based, not geographic)

#### 5.2: Comparison to International PR
- **More geographic** than party-list PR (Netherlands)
- **More proportional** than mixed-member PR (Germany still has overhang seats)
- **More complex** than STV (Ireland has simpler ballot)

**Unique position**: Achieves proportionality while maintaining single-member districts, but at cost of geographic overlap.

#### 5.3: Implementation Challenges
**Legal:**
- Does "single-member district" mean one district per geographic area, or one representative per district?
- Uniform Congressional District Act (1967) interpretation

**Administrative:**
- Ballot design: List all party-specific races?
- Precinct mapping: Which districts overlap each precinct?
- Redistricting commissions: One per party or unified?

**Political:**
- Neither party has incentive to adopt (both prefer current system in states they control)
- Possible in commission-led states (CA, CO, MI, etc.)

#### 5.4: Voter Experience
**Scenario**: You live in Philadelphia, PA
- You're a registered Democrat
- Your **Democratic district**: PA-4 (Philadelphia metro)
- Your **Republican district**: PA-2 (SE Pennsylvania including Philly suburbs)
- You vote in: Democratic primary for PA-4, Republican primary for PA-2 (if open primary)
- You're represented by: 2 House members (one from each party)

**Questions:**
- Which rep do you contact for constituent services?
- Do you pay attention to both races?
- Does this reduce polarization (cross-party representation) or increase confusion?

#### 5.5: Future Research
- Voter studies: How do citizens respond to overlapping representation?
- Comparative analysis: Similar systems in other countries?
- Legislative behavior: Do representatives with overlapping constituencies cooperate more?
- Alternative allocation: Should seats be allocated by district-level vote or statewide?

### Section 6: Conclusion (400 words)
- Summary: Overlapping party-districts achieve perfect proportionality
- Radical departure from US tradition (geographic communities)
- Implementation barriers (legal, administrative, political)
- Theoretical contribution: Expands space of possible PR systems
- Policy relevance: States could experiment with this system

---

## Figures (5 total)

**Figure 1: System Diagram**
- Flowchart: Statewide vote → Seat allocation → Party-specific redistricting → Overlapping maps

**Figure 2: Pennsylvania Example**
- 3-panel map: D districts / R districts / Overlay
- Show how every point is in 2 districts

**Figure 3: Seat Allocation Comparison**
- Bar chart: Enacted vs Our System seat shares by party across 10 states
- Show proportionality gap closing

**Figure 4: Compactness by Party**
- Box plots: Democratic districts / Republican districts / Neutral baseline
- Compare compactness distributions

**Figure 5: Voter Representation**
- Map showing % voters with co-partisan rep: Enacted (purple gradient) vs Our System (solid 100%)

---

## Tables (4 total)

**Table 1**: Seat allocation (10 example states)
**Table 2**: Compactness metrics by party
**Table 3**: Voter representation by type
**Table 4**: Implementation challenges matrix (Legal/Admin/Political)

---

## Target Venue

### Electoral Studies (Primary Target)
**Why:**
- Proportional representation focus
- International comparisons welcome
- Electoral system innovation
- 8,000 word limit (good fit)

**Alternative**: Representation (journal) - focus on representation theory

---

## Data Requirements

**Already Available:**
- State boundaries, census tracts
- Statewide vote shares (presidential 2020)
- Recursive bisection algorithm

**Need to Implement:**
- Proportional seat allocation (D'Hondt method)
- Party-specific redistricting runs (separate per party)
- Overlay visualization

**Estimated Processing**: 3-4 days (straightforward - no optimization needed)

---

## Implementation Timeline

### Phase 1: Algorithm Implementation (3 days)
- Implement proportional seat allocation
- Modify pipeline to run per-party recursive bisection
- Create overlay visualization

### Phase 2: 50-State Run (2 days)
- Generate party-specific districts for all states
- Compute compactness per party
- Analyze voter representation

### Phase 3: Analysis (2 days)
- Compare to enacted and neutral baselines
- Create all tables and figures
- Case studies (PA, NC, TX)

### Phase 4: Writing (1 week)
- Draft all sections
- International PR comparison
- Implementation feasibility analysis

### Phase 5: Review (2 days)
- Internal review and revision
- Submit to Electoral Studies

**Total: 2 weeks**

---

## Success Criteria

- [ ] Party-specific districts generated for all 50 states
- [ ] Perfect proportionality verified (efficiency gap = 0)
- [ ] Compactness measured per party
- [ ] Overlay maps created
- [ ] All 5 figures generated
- [ ] All 4 tables completed
- [ ] Draft complete (8,000 words)

---

## Related Work Integration

**From Paper 01 (recursive-bisection)**:
- Provides the core algorithm for party-specific districting

**From Paper 17 (multi-member-districts)**:
- Comparative case: Another path to proportional representation

**Extension**:
- Combines proportional seat allocation with single-member geographic districts
- Unique contribution: Overlapping party-specific districts

---

**Created**: 2026-02-08
**Updated**: 2026-02-08 (System redesign: overlapping party districts)
**Panel Reference**: N/A (experimental)
**Related Enhancement**: E24 (Party-Based Allocation)
**Risk Level**: High (radical departure from US redistricting norms)
**Scientific Value**: High (novel PR system design)
