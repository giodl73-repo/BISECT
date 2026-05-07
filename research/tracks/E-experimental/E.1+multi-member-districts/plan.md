# Multi-Member Districts and Proportional Representation — Paper Plan

## Research Question

**Primary**: What if we replaced 435 single-member districts with ~100-150 multi-member districts? How would this affect gerrymandering, representation quality, and political competition?

**Sub-questions**:
- RQ1: Can MMDs eliminate gerrymandering through proportional allocation?
- RQ2: How would MMDs affect minority representation compared to VRA single-member districts?
- RQ3: What is the optimal district size (3 vs 5 vs 7 representatives)?
- RQ4: Would MMDs enable minor party representation (Libertarian, Green)?

**Contribution**: First algorithmic MMD design for U.S. Congress using recursive bisection with proportional representation simulation.

---

## Target Venue

- **Journal**: Electoral Studies
- **Deadline**: Rolling submission
- **Page limit**: 8,000-10,000 words
- **Expected contribution type**: Empirical Study + Electoral Systems
- **Why this venue**: Specializes in electoral systems, MMD research, international comparisons (Ireland, Germany, New Zealand)
- **Fit**: Best venue for MMD analysis with algorithmic approach

---

## Sections

### Section 1: Introduction (700 words)
- Problem: Single-member districts + winner-take-all = gerrymandering + two-party lock
- International norm: Most democracies use MMDs with PR (Germany, New Zealand, Ireland)
- U.S. history: Congress used MMDs until 1842, banned by federal law 1967
- This paper: Reintroduce MMDs with modern algorithmic design
- Value: Eliminates gerrymandering structurally, not through oversight
- Contributions: (1) Algorithmic MMD design, (2) Proportionality analysis, (3) Trade-off quantification

### Section 2: Historical and International Context (1,000 words)
- U.S. historical use of MMDs (1789-1842, Apportionment Act, 1967 ban)
- International MMD systems:
  - Ireland: 3-5 member constituencies, STV, high proportionality
  - Germany: MMP (half single-member, half party-list)
  - New Zealand: Switched from FPTP to MMP in 1996 via referendum
  - Netherlands: Single nationwide district (150 members), pure party-list
- Why U.S. resists MMDs: Incumbency protection, federalism, status quo bias
- Literature: Prior MMD research, proportional representation theory

### Section 3: Methodology (1,200 words)
- MMD district design:
  - Option A: ~145 districts of 3 representatives (145 × 3 = 435)
  - Option B: ~109 districts of 4 representatives (109 × 4 = 436)
  - Option C: ~87 districts of 5 representatives (87 × 5 = 435)
- Algorithm: Recursive bisection to create k districts, population target: 769K × m reps
- Electoral system simulation: Party-list PR with D'Hondt method
- Data: Census 2020 tract populations + geometries, presidential vote by tract
- Metrics:
  - Proportionality: Gallagher index
  - Minor party representation: % seats for parties <5% national vote
  - Minority representation: Compare to VRA single-member findings
  - Compactness: Polsby-Popper (expect ~10% loss for larger districts)

### Section 4: Results (1,800 words)
- District configurations: Compare 3/4/5-member options (Table 1)
- Proportionality achievement: Gallagher index comparison (Table 2)
- Minor party representation: Seats by party simulated 2020 election (Table 3)
- Minority representation: Compare to Paper 03 VRA findings (proportional outcomes without geographic packing)
- Compactness trade-off: PP by district size (Table 4)
- Key finding: 5-member MMDs achieve near-perfect proportionality (Gallagher < 2.0) while enabling minor parties

### Section 5: Discussion (1,500 words)
- Advantages: Eliminates gerrymandering, enables minor parties, improves minority representation, reduces polarization
- Disadvantages: Weaker local accountability (large districts ~3.8M), requires federal legislation, party-centric if party-list, ~10% compactness cost
- Optimal district size: 5-member MMDs balance proportionality and local representation
- Path to adoption: Legal feasibility (repeal 1967 Act), political feasibility (low, but possible via state pilots or referendums)
- International precedent: New Zealand switched via referendum in 1993
- Future work: Multi-year stability, STV simulation, mixed-member proportional, public opinion

### Section 6: Conclusion (400 words)
- Summary: MMDs eliminate gerrymandering, enable minor parties, improve minority representation
- Trade-off: Proportionality vs local accountability (~10% compactness cost)
- Recommendation: 5-member MMDs optimal balance
- Path forward: State-level pilots, federal legislation, citizen referendums
- Significance: Structural solution to gerrymandering (vs boundary optimization)

---

## Experiments

### Experiment 1: Generate MMD Configurations
- **Script**: `scripts/experiments/mmd_generate_districts.py --members 3 4 5 --year 2020`
- **Description**: Run recursive bisection to create three MMD configurations (3/4/5 representatives per district)
- **Input**: Census 2020 tract populations and geometries
- **Output**:
  - `outputs/mmd/3-member/districts.csv` (145 districts)
  - `outputs/mmd/4-member/districts.csv` (109 districts)
  - `outputs/mmd/5-member/districts.csv` (87 districts)
- **Duration**: ~30 minutes per configuration
- **Validation**: Population balance ±0.5%, all districts contiguous

### Experiment 2: Electoral Simulation
- **Script**: `scripts/experiments/mmd_simulate_election.py --year 2020 --method dhondt`
- **Description**: Simulate 2020 presidential election using party-list PR with D'Hondt seat allocation
- **Input**:
  - MMD district assignments (from Exp 1)
  - Presidential vote by tract (data/2020/elections/)
- **Output**:
  - `outputs/mmd/3-member/seats_by_party.csv`
  - `outputs/mmd/4-member/seats_by_party.csv`
  - `outputs/mmd/5-member/seats_by_party.csv`
- **Duration**: ~5 minutes per configuration
- **Validation**: Total seats = 435, seat shares match vote shares

### Experiment 3: Proportionality Analysis
- **Script**: `scripts/experiments/mmd_compute_gallagher.py`
- **Description**: Compute Gallagher index for each MMD configuration and compare to single-member baselines
- **Input**: Seat allocation results (from Exp 2)
- **Output**: `outputs/mmd/gallagher_comparison.csv`
- **Duration**: <1 minute
- **Validation**: Gallagher index between 0 (perfect PR) and 15 (high disproportionality)

### Experiment 4: Compactness Analysis
- **Script**: `scripts/experiments/mmd_compare_compactness.py`
- **Description**: Compute Polsby-Popper for MMD configurations and compare to single-member districts
- **Input**:
  - MMD geometries (from Exp 1)
  - Single-member baseline (outputs/v1/2020/districts/)
- **Output**: `outputs/mmd/compactness_comparison.csv`
- **Duration**: ~10 minutes
- **Validation**: MMDs less compact than single-member (expect ~5-10% reduction)

---

## Figures

### Figure 1: MMD Configuration Maps
- **Type**: Multi-panel map (3 panels: single-member / 3-member / 5-member)
- **Script**: `scripts/visualization/mmd_create_maps.py --panels 1 3 5`
- **Data source**: District geometries from Experiment 1
- **Shows**: Visual consolidation from 435 to 145 to 87 districts
- **Output**: `figures/mmd_configurations.png` (300 DPI)

### Figure 2: Proportionality Comparison
- **Type**: Bar chart
- **Script**: `scripts/visualization/mmd_plot_gallagher.py`
- **Data source**: `outputs/mmd/gallagher_comparison.csv`
- **Shows**: Gallagher index for: Enacted / Algorithmic single-member / 3-member / 5-member
- **Output**: `figures/proportionality_comparison.png`

### Figure 3: Minor Party Representation
- **Type**: Grouped bar chart
- **Script**: `scripts/visualization/mmd_plot_minor_parties.py`
- **Data source**: Seat allocation results from Experiment 2
- **Shows**: Seat shares for Democratic, Republican, Libertarian, Green, Other parties across systems
- **Output**: `figures/minor_party_representation.png`

### Figure 4: Minority Representation Comparison
- **Type**: Comparison chart with Paper 03 baseline
- **Script**: `scripts/visualization/mmd_plot_minority_rep.py`
- **Data source**:
  - VRA single-member baseline from Paper 03
  - MMD proportional outcomes from Experiment 2
- **Shows**: Black/Hispanic representation under single-member MM districts vs MMD proportional allocation
- **Output**: `figures/minority_representation.png`

### Figure 5: Compactness-Proportionality Trade-off
- **Type**: Scatter plot with Pareto frontier
- **Script**: `scripts/visualization/mmd_plot_tradeoff.py`
- **Data source**:
  - Gallagher index from Experiment 3
  - Compactness from Experiment 4
- **Shows**: Cannot maximize both proportionality and compactness simultaneously
- **Output**: `figures/compactness_proportionality_tradeoff.png`

---

## Tables

### Table 1: MMD Configuration Comparison
- **Data source**: Experiment 1 outputs
- **Columns**: Option, Districts, Reps per district, Total reps, Mean district population, Mean PP compactness
- **Rows**: 3-member (145), 4-member (109), 5-member (87)

### Table 2: Proportionality Metrics
- **Data source**: Experiment 3 output
- **Columns**: System, Gallagher Index, Interpretation
- **Rows**: Single-member enacted, Single-member algorithmic, 3-member MMD, 5-member MMD

### Table 3: Seat Allocation by Party (2020 Election Simulation)
- **Data source**: Experiment 2 outputs
- **Columns**: Party, Vote %, Single-member seats, 3-member MMD, 5-member MMD
- **Rows**: Democratic, Republican, Libertarian, Green, Other

### Table 4: Compactness by District Size
- **Data source**: Experiment 4 output
- **Columns**: System, Mean PP, Median PP, Change from single-member
- **Rows**: Single-member (baseline), 3-member MMD, 5-member MMD

---

## Analysis Scripts

### Script 1: District Generation
- **Path**: `scripts/experiments/mmd_generate_districts.py`
- **Function**: Modify recursive_bisection.py to target k districts (87/109/145) instead of 435
- **Input**: Census tracts, population targets
- **Output**: District assignments CSV + geometries GeoJSON

### Script 2: Seat Allocation Simulator
- **Path**: `scripts/experiments/mmd_simulate_election.py`
- **Function**: Implement D'Hondt method for proportional seat allocation within each MMD
- **Input**: District assignments, presidential vote by tract
- **Output**: Seats by party per district + national totals

### Script 3: Gallagher Index Calculator
- **Path**: `scripts/experiments/mmd_compute_gallagher.py`
- **Function**: Compute Gallagher index = sqrt(0.5 * sum((vote_share - seat_share)^2))
- **Input**: Vote shares and seat shares by party
- **Output**: Single proportionality metric

### Script 4: Compactness Comparator
- **Path**: `scripts/experiments/mmd_compare_compactness.py`
- **Function**: Compute Polsby-Popper for MMD geometries and compare to single-member baseline
- **Input**: District geometries (MMD and single-member)
- **Output**: Compactness statistics CSV

### Script 5: Visualization Suite
- **Path**: `scripts/visualization/mmd_*.py` (5 scripts for 5 figures)
- **Function**: Generate all figures from experiment outputs
- **Input**: Experiment result CSVs
- **Output**: PNG figures at 300 DPI

---

## Quality Checkpoints

- [ ] **Word count**: 8,000-10,000 words (Electoral Studies target)
- [ ] **References**: 40+ citations (international MMD literature, PR theory, U.S. electoral history)
- [ ] **Figures**: 5 figures (all generated from experiments)
- [ ] **Tables**: 4 tables (all populated with experiment results)
- [ ] **Experiments**: All 4 experiments complete with validated outputs
- [ ] **Reproducibility**: All scripts documented in methodology section
- [ ] **Cross-references**: Integration with Papers 01 (algorithm), 03 (VRA), 15 (proportionality)
- [ ] **International precedent**: Ireland, Germany, New Zealand cases cited with outcomes
- [ ] **Legal analysis**: Uniform Congressional District Act (1967) repeal requirements discussed
- [ ] **Compactness validation**: Polsby-Popper computed for all configurations

---

## Dependencies

### Experiment Dependencies
- Experiment 2 (Electoral Simulation) depends on: Experiment 1 (District Generation)
- Experiment 3 (Proportionality) depends on: Experiment 2 (Seat Allocation)
- Experiment 4 (Compactness) depends on: Experiment 1 (District Generation)

### Figure Dependencies
- Figure 1 (Maps) depends on: Experiment 1
- Figure 2 (Proportionality) depends on: Experiment 3
- Figure 3 (Minor Parties) depends on: Experiment 2
- Figure 4 (Minority Rep) depends on: Experiment 2 + Paper 03 baseline data
- Figure 5 (Trade-off) depends on: Experiments 3 + 4

### Section Dependencies
- Section 4 (Results) depends on: All experiments complete
- Section 5 (Discussion) depends on: Section 4 results
- Figures must be generated before final section writing

### Writing Dependencies
- Write Introduction first (no dependencies)
- Write Historical Context second (no dependencies)
- Write Methodology third (documents experiments)
- Run all experiments (1-4)
- Generate all figures (1-5)
- Write Results (depends on experiments + figures)
- Write Discussion (depends on results)
- Write Conclusion (depends on discussion)

---

## Implementation Timeline

**Total estimate**: 2-3 weeks

### Week 1: Experiments
- Days 1-2: Generate MMD configurations (Experiment 1)
- Day 3: Electoral simulation (Experiment 2)
- Day 4: Proportionality analysis (Experiment 3)
- Day 5: Compactness analysis (Experiment 4)

### Week 2: Writing + Figures
- Days 1-2: Write Introduction + Historical Context (no dependencies)
- Day 3: Write Methodology (document experiments)
- Days 4-5: Generate all figures (Experiments complete)
- Weekend: Write Results section (experiments + figures ready)

### Week 3: Finalization
- Days 1-2: Write Discussion (results complete)
- Day 3: Write Conclusion
- Days 4-5: Review, revisions, compile PDF, check quality gates

---

## Data Requirements

**Already Available**:
- Census 2020 tract populations and geometries
- Presidential vote by tract (for party-list simulation)
- Single-member algorithmic baseline (Paper 01)
- VRA compliance baseline (Paper 03)

**Need to Compute**:
- MMD boundaries for 87/109/145 districts (Experiment 1)
- Seat allocations by D'Hondt method (Experiment 2)
- Gallagher index (Experiment 3)
- Compactness metrics for larger districts (Experiment 4)

**Estimated Data Processing**: 1 week (Experiments 1-4)

---

## Success Criteria

- [ ] 3 MMD configurations generated (3/4/5-member) with validated population balance
- [ ] Electoral simulations complete for all 3 configurations
- [ ] Proportionality quantified: Gallagher index < 2.0 for 5-member MMDs (hypothesis)
- [ ] Minor party representation analyzed: Libertarian + Green seats > 0 for 5-member
- [ ] Compactness trade-off quantified: ~5-10% reduction vs single-member
- [ ] All 5 figures generated at 300 DPI
- [ ] All 4 tables populated with experiment results
- [ ] Draft complete: 8,000-10,000 words
- [ ] 40+ citations (international MMD literature)
- [ ] PDF compiles without errors

---

## Related Work Integration

**From Paper 01 (recursive-bisection)**:
- Same recursive bisection algorithm, different target k (87/109/145 instead of 435)
- Population balance constraints identical

**From Paper 03 (vra-compliance)**:
- Compare VRA outcomes: 137 MM single-member districts vs MMD proportional allocation
- Hypothesis: MMDs achieve similar minority representation without geographic packing

**From Paper 15 (party-based-allocation)**:
- Both seek proportionality, but different mechanisms:
  - Paper 15: Boundary manipulation to achieve proportional outcomes
  - This paper: Structural proportionality through MMDs + PR

**Extension**:
- Structural solution to gerrymandering (vs boundary optimization)
- International precedent validation
- Trade-off quantification (proportionality vs compactness)

---

**Status**: Ready for panel:author orchestration
**Created**: 2026-02-08
**Target Venue**: Electoral Studies (8,000-10,000 words)
**Risk Level**: Medium (requires federal legislation for real-world adoption)
**Scientific Value**: High (proven international system, structural gerrymandering solution)
