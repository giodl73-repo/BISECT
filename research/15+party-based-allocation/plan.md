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
- Example (10-seat state with 5% threshold):
  - Democrats: 58% of vote → 6 seats
  - Republicans: 38% of vote → 4 seats
  - Libertarians: 4% of vote → 0 seats (below threshold)
  - Nonpartisan (optional): 1 seat (if guaranteed)

### Step 2: Party-Vote Weighted Redistricting (CRITICAL)
**Each party's districts are balanced by THEIR OWN VOTERS, not total population:**

- **Democrats draw 6 districts** covering the entire state
  - **Vertex weights = Democratic votes per tract** (not total population!)
  - Each district has ~equal number of Democratic voters
  - Naturally concentrates in urban areas where Democrats are strong

- **Republicans draw 4 districts** covering the entire state
  - **Vertex weights = Republican votes per tract**
  - Each district has ~equal number of Republican voters
  - Naturally concentrates in rural/suburban areas where Republicans are strong

- **Districts overlap**: Every geographic point is in multiple districts (one per party)

**Why party-vote weighting?**
- Ensures proportional representation WITHIN each party's caucus
- Without this: urban Democratic district might have 3x more Dem voters than rural Democratic district
- With this: all Democratic districts have equal Democratic representation

### Step 3: Multi-Party Support
**All parties above threshold get representation:**
- Democratic districts (weighted by Dem votes)
- Republican districts (weighted by Rep votes)
- Libertarian districts (if above threshold, weighted by Libertarian votes)
- Green districts (if above threshold, weighted by Green votes)
- Nonpartisan district(s) (optional guaranteed seat, weighted by third-party/unaffiliated votes)

### Step 4: Representation
- Each voter has multiple representatives (one per party above threshold)
- Democratic voter: Gets 1 Democratic rep + 1 Republican rep (+ others if applicable)
- Republican voter: Gets 1 Republican rep + 1 Democratic rep (+ others if applicable)
- Libertarian voter: Gets 1 Libertarian rep (if party > threshold) + major party reps
- **100% of voters have at least one co-partisan representative** (if their party > threshold)

### Special Cases
- **Single-seat parties**: Entire state is their district (no subdivision needed)
- **Below-threshold parties**: Get 0 seats, voters represented by above-threshold parties
- **Non-voters**: Not represented (but could be studied as separate category)

---

## Research Questions

### RQ1: Party-Vote Weighting vs Population Weighting
**Does weighting districts by party votes (not total population) improve representation?**

**Hypothesis:**
- Party-vote weighting ensures equal representation within each party's caucus
- Population weighting creates imbalances (urban districts have more co-partisans)

**Analysis:**
- Compare district voter counts with both methods
- Measure variance in co-partisan representation

### RQ2: Threshold Sensitivity (Ablation Study)
**How does seat allocation change with different thresholds?**

**Thresholds to test:**
- 0% (pure proportionality - all parties get seats)
- 2.5% (low barrier)
- 5.0% (European standard)
- Natural 1/(N+1) (varies by state size)
- 10% (high barrier)

**Questions:**
- How many parties win seats at each threshold?
- Do Libertarians/Greens ever reach threshold?
- Which states are most friendly to third parties?

### RQ3: Nonpartisan/Independent Voters
**Should nonpartisan voters get guaranteed representation?**

**Options to compare:**
- **Threshold-based**: Nonpartisan gets seats only if "Other" vote > threshold
- **Guaranteed 1 seat**: Always allocate 1 nonpartisan district
- **Proportional with no threshold**: Even small parties get fractional representation

**Research questions:**
- Where do nonpartisan districts concentrate geographically? (suburbs?)
- Do nonpartisan voters prefer their own rep or diverse representation?
- What's the partisan composition of nonpartisan districts?

### RQ4: Democratic Legitimacy
**Does overlapping representation enhance or undermine democratic accountability?**

**Pro arguments:**
- Every voter has a representative from their preferred party
- Eliminates "wasted votes" and gerrymandering
- Minority party voters in safe districts finally get representation
- Third parties get representation if they reach threshold

**Con arguments:**
- Voter confusion (which district am I in?)
- Diluted accountability (multiple overlapping reps)
- Breaks geographic community representation
- Violates "one person, one vote" (districts have unequal total population)

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
- State boundaries, census tracts (2020 Census)
- Census tract geometries and adjacency graphs
- Recursive bisection algorithm
- Statewide vote shares (2020 presidential)

**Need to Acquire:**
- **Tract-level presidential vote data (2020)**
  - Democratic votes per tract
  - Republican votes per tract
  - Libertarian/Green/Other votes per tract
  - Non-voter counts per tract
  - Source: Precinct-to-tract disaggregation (VEST, Dave Leip)

**Need to Implement:**
- ✅ Proportional seat allocation (D'Hondt method) - DONE
- ✅ Party-specific wrapper - DONE
- Tract-level vote data loader
- Party-vote weighted recursive bisection
- Threshold ablation framework (0%, 2.5%, 5%, natural, 10%)
- Multi-party support (D, R, L, G, Nonpartisan)
- Overlay visualization (multiple party maps)

**Estimated Processing**:
- Data acquisition: 1-2 days (tract-level votes)
- Implementation: 3-4 days
- 50-state runs: 4-6 hours (multiple parties × multiple thresholds)
- Analysis: 2-3 days

---

## Implementation Details

### Component 1: Proportional Seat Allocation Module

**File**: `src/apportionment/proportional/dhondt.py`

**Function**: `allocate_seats_dhondt(vote_shares: dict, num_seats: int, threshold: float) -> dict`

**Algorithm**:
```python
def allocate_seats_dhondt(vote_shares, num_seats, threshold):
    """
    Allocate seats using D'Hondt method.

    Args:
        vote_shares: {"Democratic": 0.51, "Republican": 0.49}
        num_seats: Total seats for state (e.g., 17 for PA)
        threshold: Minimum vote share to qualify (e.g., 1/(num_seats+1))

    Returns:
        {"Democratic": 9, "Republican": 8}
    """
    # Filter parties above threshold
    eligible_parties = {p: v for p, v in vote_shares.items() if v >= threshold}

    # Compute quotients for each seat
    quotients = []
    for party, votes in eligible_parties.items():
        for divisor in range(1, num_seats + 1):
            quotients.append((votes / divisor, party, divisor))

    # Sort by quotient descending
    quotients.sort(reverse=True, key=lambda x: x[0])

    # Allocate seats
    seat_allocation = {p: 0 for p in eligible_parties}
    for i in range(num_seats):
        _, party, _ = quotients[i]
        seat_allocation[party] += 1

    return seat_allocation
```

**Testing**:
- Unit tests: Known examples (Germany, Netherlands)
- Edge cases: Ties, single-seat states, threshold boundaries

### Component 2: Tract-Level Vote Data Loader

**File**: `scripts/data/load_tract_votes.py`

**Function**: Load presidential vote counts at tract level

**Algorithm**:
```python
def load_tract_votes(state, year=2020):
    """
    Load tract-level presidential vote data.

    Returns:
        DataFrame with columns:
        - tract_id (GEOID)
        - total_votes
        - democratic_votes
        - republican_votes
        - libertarian_votes
        - green_votes
        - other_votes
        - nonvoters (voting_age_pop - total_votes)
    """
    # Load from VEST or Dave Leip tract-level files
    # Disaggregate precinct data to tracts if needed
    # Return vote counts per tract
```

**Data sources:**
- VEST (Voting and Election Science Team) - tract-level estimates
- Dave Leip's Atlas - precinct data → tract disaggregation
- Census voting-age population for non-voter calculation

### Component 3: Party-Vote Weighted Recursive Bisection

**File**: `scripts/pipeline/run_party_specific_redistricting.py` (updated)

**Critical change**: Weight vertices by party votes, not total population

**Algorithm**:
```python
def run_party_specific_redistricting(state, year, version, threshold=None):
    """
    Generate overlapping party-specific districts with party-vote weighting.

    1. Load tract-level vote data (not just statewide!)
    2. Allocate seats to parties using D'Hondt with threshold
    3. For each party with 2+ seats:
       - Load adjacency graph (same for all parties)
       - **Weight vertices by PARTY VOTES** (not total pop!)
         Example Democratic district: vertex_weights = tract['democratic_votes']
       - Run recursive_bisection(adjacency, party_vote_weights, num_seats)
       - Save to outputs/{version}_{year}/{state}/{party}/districts.shp
    4. For parties with 1 seat:
       - Create whole-state district
    5. Generate overlay visualization showing all party maps
    """
    # Step 1: Get vote shares
    vote_shares = load_presidential_results(state, year)
    # {"Democratic": 0.51, "Republican": 0.49, "Other": 0.00}

    # Step 2: Allocate seats
    num_seats = get_state_seat_count(state, year)
    threshold = 1.0 / (num_seats + 1)
    seat_allocation = allocate_seats_dhondt(vote_shares, num_seats, threshold)
    # {"Democratic": 9, "Republican": 8}

    # Step 3: Run redistricting per party
    for party, seats in seat_allocation.items():
        if seats >= 2:
            # Run recursive bisection
            run_recursive_bisection(
                state=state,
                year=year,
                version=version,
                party=party,
                num_districts=seats
            )
        elif seats == 1:
            # Create whole-state district
            create_whole_state_district(state, year, version, party)

    # Step 4: Overlay visualization
    create_overlay_map(state, year, version, seat_allocation)
```

### Component 3: Modified Recursive Bisection

**Modification**: Existing `src/apportionment/partition/recursive_bisection.py` needs NO changes

**Why**: Already takes `num_districts` parameter. Just call it multiple times with different `num_districts` values per party.

**Output Structure**:
```
outputs/party_v1_2020/
├── pennsylvania/
│   ├── democratic/
│   │   ├── districts.shp  (9 districts)
│   │   ├── compactness.csv
│   │   └── metadata.json
│   ├── republican/
│   │   ├── districts.shp  (8 districts)
│   │   ├── compactness.csv
│   │   └── metadata.json
│   └── overlay/
│       ├── combined_map.png
│       └── overlap_analysis.csv
├── texas/
│   ├── democratic/
│   │   └── districts.shp  (18 districts)
│   ├── republican/
│   │   └── districts.shp  (21 districts)
│   └── overlay/
...
```

### Component 4: Overlay Visualization

**File**: `scripts/visualization/create_party_overlay.py`

**Function**: Generate maps showing overlapping districts

**Visualization Types**:
1. **3-panel map**: Democratic districts | Republican districts | Overlay
2. **Animated GIF**: Cycle between party maps
3. **Interactive HTML**: Leaflet map with party toggle

**Key Metrics**:
- Overlap depth: How many districts cover each point? (should be 2 in two-party system)
- Compactness comparison: Democratic PP vs Republican PP vs Neutral PP
- Population balance: Deviation within each party's districts

### Component 5: Threshold Ablation Framework

**File**: `scripts/analysis/threshold_ablation.py`

**Function**: Run full system with different thresholds

**Algorithm**:
```python
THRESHOLDS = {
    'zero': 0.0,           # Pure proportionality
    'low': 0.025,          # 2.5%
    'european': 0.05,      # 5% (standard)
    'natural': None,       # 1/(N+1) per state
    'high': 0.10           # 10%
}

for threshold_name, threshold_value in THRESHOLDS.items():
    for state in all_states:
        # Run party-specific redistricting
        results = run_party_specific_redistricting(
            state, year,
            version=f"party_{threshold_name}",
            threshold=threshold_value
        )

        # Analyze:
        # - How many parties got seats?
        # - Did Libertarians/Greens reach threshold?
        # - Geographic distribution of parties
        # - Voter representation coverage
```

**Output**: Comparison tables across all thresholds

### Component 6: Analysis Scripts

**Proportionality Check** (`scripts/analysis/verify_proportionality.py`):
- Compute efficiency gap (should be 0.0 by design)
- Compare to enacted and neutral baselines
- Per-party voter balance (equal Dem voters per Dem district?)
- Generate Table 1 data

**Compactness Analysis** (`scripts/analysis/compute_party_compactness.py`):
- Polsby-Popper per party
- Compare Democratic vs Republican vs Libertarian vs Neutral
- Do third-party districts have different compactness?
- Generate Table 2 data

**Voter Representation** (`scripts/analysis/analyze_representation.py`):
- Assign voters to party districts
- Compute % with co-partisan rep (should be 100% if party > threshold)
- Coverage by threshold level
- Nonpartisan voter analysis
- Generate Table 3 data

**Threshold Impact** (`scripts/analysis/analyze_thresholds.py`):
- Compare results across all 5 thresholds
- Number of parties represented by state
- Third-party viability
- Generate Table 4 (threshold comparison)

**Nonpartisan Analysis** (`scripts/analysis/analyze_nonpartisan.py`):
- Geographic distribution of nonpartisan districts
- Partisan composition within nonpartisan districts
- Suburban vs urban vs rural concentration

---

## Implementation Timeline

### Phase 1: Algorithm Implementation (3 days)
- Day 1: Implement D'Hondt allocation (dhondt.py + tests)
- Day 2: Create party-specific wrapper (run_party_specific_redistricting.py)
- Day 3: Test on 3 pilot states (PA, TX, CA)

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
