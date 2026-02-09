# Multi-Member Districts and Proportional Representation — Plan

**Artifact Type**: Research Paper (Paper #17 - Experimental)
**Goal**: Design larger districts with multiple representatives elected proportionally
**Estimated Effort**: 2-3 weeks
**Status**: Planned
**Source**: Enhancement E28

---

## Objective

Explore **multi-member districts (MMDs)**: Fewer, larger districts with 3-5 representatives each, elected through proportional representation.

**Research Question**: What if we replaced 435 single-member districts with ~100-150 multi-member districts? How would this affect gerrymandering, representation quality, and political competition?

**System**: Use algorithmic redistricting to create multi-member districts, simulate proportional elections (ranked-choice or party-list).

---

## Research Questions

### RQ1: Gerrymandering Elimination
**Can MMDs eliminate gerrymandering?**

- Hypothesis: Proportional allocation within MMDs prevents packing/cracking
- Mechanism: Even if district boundaries are biased, seats allocated proportionally
- **Key advantage**: Removes gerrymandering incentive

### RQ2: Minority Representation
**How would MMDs affect VRA compliance?**

- Current: Majority-minority single-member districts (Paper 03 findings)
- MMDs: Minorities elect representatives proportionally without geographic concentration
- Example: 40% minority population → 2 out of 5 representatives (if voting cohesively)

### RQ3: District Size and Representativeness
**What is optimal district size?**

- Trade-offs:
  - **Small MMDs** (3 reps): More local, easier to gerrymander proportional outcomes
  - **Large MMDs** (7+ reps): More proportional, less local connection
- International precedent: Ireland (3-5), Malta (5-7), Netherlands (150)

### RQ4: Political Competition
**Would MMDs increase political competition?**

- Hypothesis: Minor parties become viable (Libertarian, Green) under PR
- Current: Two-party duopoly enforced by winner-take-all
- MMDs: Multi-party representation possible

---

## Proposed Structure

### Abstract (150 words)
- Problem: Single-member districts prone to gerrymandering, exclude minor parties
- Alternative: Multi-member districts with proportional representation
- Method: Create ~100-150 MMDs using recursive bisection, simulate PR elections
- Findings: (TBD) MMDs eliminate gerrymandering, improve minority representation, enable multi-party system
- Contribution: First algorithmic MMD design for U.S. Congress

### Section 1: Introduction (700 words)
- **Problem**: Single-member districts + winner-take-all = gerrymandering + two-party lock
- **International norm**: Most democracies use MMDs with PR (Germany, New Zealand, Ireland, etc.)
- **U.S. history**: Congress used MMDs until 1842, banned by federal law 1967
- **This paper**: Reintroduce MMDs with modern algorithmic design
- **Value**: Eliminates gerrymandering structurally, not through oversight

### Section 2: Historical and International Context (1,000 words)

#### 2.1: U.S. Historical Use of MMDs
- **1789-1842**: Many states used at-large or multi-member districts
- **Apportionment Act of 1842**: Required single-member districts
- **Uniform Congressional District Act (1967)**: Codified single-member requirement
- **Constitutional question**: Could federal law be changed to allow MMDs?

#### 2.2: International MMD Systems
**Ireland**:
- 3-5 member constituencies
- Single transferable vote (STV)
- High proportionality, local representation preserved

**Germany**:
- Mixed-member proportional (MMP)
- Half single-member, half party-list
- Balances local accountability with proportionality

**New Zealand**:
- Switched from FPTP to MMP in 1996
- Referendum-approved after citizen dissatisfaction with disproportional outcomes

**Netherlands**:
- Single nationwide district (150 members)
- Pure party-list PR
- Extreme proportionality, no local representation

#### 2.3: Why U.S. Resists MMDs
- **Incumbency protection**: Two-party system entrenched
- **Federalism**: Single-member districts seen as preserving local accountability
- **Status quo bias**: Difficult to change electoral system (requires federal legislation)

### Section 3: Methodology (1,200 words)

#### 3.1: MMD District Design
**District size options**:
- **Option A**: ~145 districts of 3 representatives each (145 × 3 = 435)
- **Option B**: ~109 districts of 4 representatives each (109 × 4 = 436)
- **Option C**: ~87 districts of 5 representatives each (87 × 5 = 435)

**Recommendation**: Test all three options, compare outcomes

**Algorithm**:
1. Apply recursive bisection to create k districts (k ∈ {87, 109, 145})
2. Population target: 769K × m representatives (m ∈ {3, 4, 5})
3. Optimize for compactness (same as single-member)
4. No need for VRA-specific edge-weighting (proportional allocation handles minority representation)

#### 3.2: Electoral System Simulation
**Option 1: Party-list PR**
- Voters select party, not individual candidate
- Seats allocated by D'Hondt method (most common)
- Example: District with 40% D, 35% R, 15% L, 10% G → 2D, 2R, 1L (if m=5)

**Option 2: Single Transferable Vote (STV)**
- Voters rank candidates (1st choice, 2nd choice, etc.)
- Candidates exceeding quota elected, surplus votes transferred
- More complex but preserves voter choice over individual candidates

**Recommendation**: Use party-list for simplicity (international norm)

#### 3.3: Metrics
- **Proportionality**: Gallagher index (measures deviation from perfect PR)
- **Minor party representation**: % seats for parties with <5% national vote
- **Minority representation**: Compare to single-member VRA findings (Paper 03)
- **Compactness**: Polsby-Popper (larger districts likely less compact)

### Section 4: Results (1,800 words)

#### 4.1: District Configurations
**Table 1**: MMD options comparison

| Option | Districts | Reps per district | Total reps | Mean district population | Mean PP compactness |
|--------|-----------|-------------------|------------|--------------------------|---------------------|
| 3-member | 145 | 3 | 435 | 2.3M | TBD |
| 4-member | 109 | 4 | 436 | 3.1M | TBD |
| 5-member | 87 | 5 | 435 | 3.8M | TBD |

**Finding**: Larger districts (5-member) likely less compact but more proportional

#### 4.2: Proportionality Achievement
**Table 2**: Gallagher index comparison

| System | Gallagher Index | Perfect PR = 0 |
|--------|-----------------|----------------|
| Single-member (enacted) | 8.5 | High disproportionality |
| Single-member (algorithmic) | 3.2 | Moderate disproportionality |
| 3-member MMD | TBD | Low disproportionality |
| 5-member MMD | TBD | Very low disproportionality |

**Hypothesis**: 5-member MMDs achieve near-perfect proportionality (Gallagher < 2.0)

#### 4.3: Minor Party Representation
**Table 3**: Seats by party (simulated 2020 election)

| Party | Vote % | Single-member seats | 3-member MMD | 5-member MMD |
|-------|--------|---------------------|--------------|--------------|
| Democratic | 51.3% | 246 (56.6%) | ~223 (51.3%) | ~223 (51.3%) |
| Republican | 46.8% | 189 (43.4%) | ~204 (46.8%) | ~204 (46.8%) |
| Libertarian | 1.2% | 0 (0%) | 0 (0%) | ~5 (1.2%) |
| Green | 0.4% | 0 (0%) | 0 (0%) | ~2 (0.4%) |
| Other | 0.3% | 0 (0%) | 0 (0%) | ~1 (0.2%) |

**Finding**: Only 5-member MMDs enable minor party representation (threshold ~3-5% per district)

#### 4.4: Minority Representation
**Analysis**: Compare to Paper 03 findings (137 MM single-member districts)

**Hypothesis**: MMDs improve minority representation through proportional allocation
- Example: District with 35% Black population → 2 out of 5 representatives likely Black (if cohesive voting)
- No need for majority-minority geographic districts (VRA packing)

**Constitutional question**: Do MMDs satisfy Voting Rights Act Section 2?
- Likely yes: Proportional outcomes achieve VRA goals without geographic segregation
- Legal analysis needed (no Supreme Court precedent)

#### 4.5: Compactness Trade-Off
**Table 4**: Compactness by district size

| System | Mean PP | Median PP | Compared to single-member |
|--------|---------|-----------|---------------------------|
| Single-member (algorithmic) | 0.461 | 0.456 | — |
| 3-member MMD | TBD | TBD | -5%? |
| 5-member MMD | TBD | TBD | -10%? |

**Trade-off**: Proportionality vs compactness (larger districts less compact)

### Section 5: Discussion (1,500 words)

#### 5.1: Advantages of MMDs
**1. Eliminates gerrymandering**:
- Proportional allocation removes incentive to manipulate boundaries
- Even if district boundaries biased, seats allocated fairly within districts

**2. Enables minor parties**:
- Libertarians, Greens, others become viable
- Breaks two-party duopoly

**3. Improves minority representation**:
- No need for geographic packing
- Proportional outcomes more robust than majority-minority districts

**4. Reduces polarization**:
- Representatives must appeal to broader coalitions (proportional threshold)
- Competitive within districts (not winner-take-all)

#### 5.2: Disadvantages of MMDs
**1. Weaker local accountability**:
- 5-member districts are large (~3.8M residents)
- Representatives less connected to local communities

**2. Requires federal legislation**:
- Uniform Congressional District Act (1967) must be repealed
- Politically difficult (incumbents resist)

**3. Party-centric (if party-list)**:
- Voters choose parties, not individuals
- Reduces candidate accountability (unless STV used)

**4. Compactness cost**:
- Larger districts less compact (~10% loss)

#### 5.3: Optimal District Size
**Trade-off analysis**:
- **3-member**: Better local representation, but still disproportional (minor parties excluded)
- **5-member**: High proportionality, minor parties viable, but large districts
- **7-member**: Extreme proportionality, but very large districts (~5M residents)

**Recommendation**: **5-member MMDs** balance proportionality and local representation

#### 5.4: Path to Adoption
**Legal feasibility**:
- Requires repealing Uniform Congressional District Act (simple majority in Congress)
- No constitutional barrier (Constitution silent on district size)

**Political feasibility**:
- Low (two-party duopoly resists)
- Possible scenarios:
  - Bipartisan commission states (CA, AZ, CO) test MMDs
  - Federal legislation after major gerrymandering scandal
  - Citizen ballot initiatives in states (then federal follow-up)

**International precedent**:
- New Zealand switched via referendum (1993)
- Can U.S. follow similar path?

#### 5.5: Future Work
- Multi-year stability (MMDs across census decades)
- STV simulation (vs party-list)
- Mixed-member proportional (half single-member, half MMD)
- Public opinion polling (voter acceptance of MMDs)

### Section 6: Conclusion (400 words)
- Summary: MMDs eliminate gerrymandering, enable minor parties, improve minority representation
- Trade-off: Proportionality vs local accountability (~10% compactness cost)
- Recommendation: 5-member MMDs optimal balance
- Path forward: State-level pilots, federal legislation, citizen referendums

---

## Figures (5 total)

**Figure 1: MMD Configuration Maps**
- 3 panels: Single-member (435) / 3-member (145) / 5-member (87)
- Shows consolidation into larger districts

**Figure 2: Proportionality Comparison**
- Gallagher index bar chart: Enacted / Algorithmic single-member / 3-member / 5-member
- Shows MMDs achieve near-perfect proportionality

**Figure 3: Minor Party Representation**
- Bar chart: Seat shares for Libertarian, Green, Other parties
- Single-member: 0%, MMD: 1-2%
- Shows MMDs enable minor parties

**Figure 4: Minority Representation**
- Comparison to Paper 03 findings (137 MM districts)
- MMD simulation: Proportional representation without geographic packing
- Shows comparable or better outcomes

**Figure 5: Compactness Trade-Off**
- Scatter plot: Proportionality (X) vs Compactness (Y)
- Shows Pareto frontier: Can't maximize both simultaneously

---

## Target Venues

### Option 1: Electoral Studies
**Why Electoral Studies?**
- Electoral systems and PR
- International comparisons (Ireland, Germany, New Zealand)
- MMD research history
- Format: 8,000-10,000 words

**Fit**: Best venue for MMD analysis

### Option 2: Representation
**Why Representation?**
- Representation theory journal
- Proportional vs majoritarian systems
- Institutional design
- Format: 8,000 words

**Fit**: Theoretical focus on representation quality

### Option 3: American Journal of Political Science (AJPS)
**Why AJPS?**
- High-tier venue
- Electoral institutions research
- Quantitative analysis
- Format: 10,000-12,000 words

**Fit**: High-impact venue for electoral reform

**Recommendation**: Submit to **Electoral Studies first** (best fit for MMD focus).

---

## Data Requirements

**Already Available**:
- Census tract populations (2020)
- Census tract geometries
- Presidential vote by tract (for party-list simulation)

**Need to Compute**:
- MMD boundaries (87/109/145 districts)
- Compactness metrics (larger districts)
- Seat allocation by D'Hondt method
- Gallagher index (proportionality measure)

**Estimated Data Processing**: 1 week

---

## Implementation Timeline

### Phase 1: MMD Design (1 week)
- Generate 3 configurations (3/4/5-member)
- Validate population balance
- Compute compactness

### Phase 2: Electoral Simulation (3-4 days)
- Simulate 2020 election with party-list PR
- Allocate seats by D'Hondt method
- Compute Gallagher index

### Phase 3: Analysis (1 week)
- Compare to single-member baselines
- Minority representation analysis
- Trade-off quantification

### Phase 4: Writing (1 week)
- Draft all sections
- International comparisons
- Generate figures

### Phase 5: Review & Submission (3 days)
- Internal review
- Revise and submit

**Total: 2-3 weeks**

---

## Success Criteria

- [ ] 3 MMD configurations generated (3/4/5-member)
- [ ] Electoral simulations complete (party-list PR)
- [ ] Proportionality quantified (Gallagher index)
- [ ] Minor party representation analyzed
- [ ] Compactness trade-off quantified
- [ ] All 5 figures generated
- [ ] Draft complete (8,000-10,000 words)

---

## Related Work Integration

**From Paper 01 (recursive-bisection)**:
- Same algorithm, different k (87/109/145 instead of 435)

**From Paper 03 (vra-compliance)**:
- Compare VRA outcomes (MM districts vs MMD proportional)

**From Paper 15 (party-based-allocation)**:
- Both seek proportionality; MMDs achieve it structurally

**Extension**:
- Structural solution to gerrymandering (vs boundary optimization)

---

**Created**: 2026-02-08
**Panel Reference**: N/A (experimental)
**Related Enhancement**: E28 (Multi-Member Districts)
**Risk Level**: Medium (requires federal legislation)
**Scientific Value**: High (international precedent, proven PR system)
