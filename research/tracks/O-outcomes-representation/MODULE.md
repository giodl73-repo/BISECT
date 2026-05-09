# Track O — Outcomes and Representation Quality

**Theme**: The program proves bisect produces compact, fair-looking *plans*. Track O asks whether compact algorithmic plans actually produce better *democratic outcomes* — more competitive elections, higher turnout, less polarization, better constituent representation. This is the empirical bridge between redistricting methodology and democratic theory.

**Core question per paper**: Compared to enacted gerrymandered plans, do bisect plans produce measurably better democratic outcomes on dimension X?

## Track Chain

O.0 (overview: what does "better redistricting" produce?) 
  → O.1 (electoral competitiveness) 
  → O.2 (voter turnout) 
  → O.3 (legislative polarization) 
  → O.4 (constituent-representative distance) 
  → O.5 (composite representation quality index)

O.1–O.4 are independent outcome dimensions. O.5 synthesizes all four into a single representation quality score.

## Papers

| Paper | Title | Outcome Metric | Data Source |
|-------|-------|---------------|-------------|
| O.0 | Redistricting and Democratic Outcomes: A Framework | Overview | Literature synthesis |
| O.1 | Electoral Competitiveness Under Algorithmic Redistricting | Margin of victory; competitive district fraction | VEST + MIT Election Lab |
| O.2 | Voter Turnout and Redistricting: Does Compactness Help? | Turnout rate by district | VEST + CPS voter supplement |
| O.3 | Legislative Polarization Under Algorithmic vs. Enacted Plans | DW-NOMINATE; ideal point estimation | Voteview |
| O.4 | Constituent-Representative Geographic Distance | Mean centroid distance; travel time | Census + OSM |
| O.5 | A Composite Representation Quality Index | Weighted composite of O.1–O.4 | All above |

## Contracts

Every O.1–O.4 paper must:
- Define the outcome metric precisely (with formula)
- Estimate the metric for bisect plans and enacted plans across 3+ states
- Use a difference-in-differences or comparable causal design where possible
- Acknowledge confounders (incumbency, campaign spending, national tides)
- Report effect sizes and confidence intervals

## Identification Strategy

The core causal challenge: redistricting plans and electoral outcomes are both endogenous to political conditions. Strategy per paper:
- **O.1**: Compare competitive district fraction before/after redistricting cycles; use bisect plan as counterfactual
- **O.2**: Exploit within-district variation in compactness changes; DiD with census tracts as fixed effects
- **O.3**: Use congressional DW-NOMINATE; compare polarization trend under different plan types
- **O.4**: Geographic distance is directly computable from assignment files; no causal identification needed

## Key Empirical Prediction

If bisect plans are more competitive (O.1) and produce lower polarization (O.3), then the "impossibility defense" gains a second dimension: not only does the algorithm not gerrymander, it demonstrably improves democratic functioning. This converts the program from a *process* argument to a *outcomes* argument.
