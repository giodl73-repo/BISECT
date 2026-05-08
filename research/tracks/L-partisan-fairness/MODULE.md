# L — Partisan Fairness

**Theme**: Mathematical definition, empirical behavior, and legal landscape of partisan
fairness metrics — how algorithmically-drawn maps compare to enacted maps and to each
other across all six major metrics.

## Track Chain

L.0 (overview) → L.1 (EG) → L.2 (MM) → L.3 (Bias) → L.4 (Declination)
              → L.5 (Seats-Votes) → L.6 (Proportionality)

L.1–L.5 are independent comparisons; L.5 is the unifying exhibit (S(v) curve
recovers EG and Bias analytically); L.6 is normative synthesis.

## Papers

| Paper | Title | Stage | Score |
|-------|-------|-------|-------|
| L.0 | Partisan Fairness Metrics: Overview and Framework | planned | — |
| L.1 | Efficiency Gap: Dedicated Treatment | planned | — |
| L.2 | Mean-Median Difference | planned | — |
| L.3 | Partisan Bias and Swing Ratio | planned | — |
| L.4 | Declination | planned | — |
| L.5 | Seats-Votes Curve and Responsiveness | planned | — |
| L.6 | Proportionality vs. Majoritarianism | planned | — |

## Data Requirements

All L papers require VEST 2020 presidential precinct returns mapped to 2020
census tracts for NC, WI, TX, FL. Uniform swing model documented in L.3.

## Contracts

Every L.1–L.5 paper must:
- Define the metric mathematically with range and neutral-map expected value
- Report metric for bisect algorithmic maps vs. enacted 2022 congressional maps
- Document the legal/court status of the metric explicitly (accepted/rejected/academic)
- Include multi-election robustness analysis (SD across swing scenarios)

## Quantification

- Primary: |metric| for bisect NC 2020 vs. enacted NC 2022
- Neutral baseline: bisect algorithmic map centroid value
- Legal context: post-Rucho federal landscape + state constitutional cases
