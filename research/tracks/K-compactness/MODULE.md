# K — Compactness Measures

**Theme**: Mathematical and legal analysis of compactness metrics used in redistricting.
Each paper defines one metric, proves its properties, implements it in `bisect-analysis`,
and evaluates it across B-series structure algorithms on NC/WI/TX.

## Track Chain

K.0 (taxonomy) → K.1 (Polsby-Popper) → K.2 (Reock) → K.3 (Convex Hull)
              → K.4 (Schwartzberg) → K.5 (Length-Width) → K.6 (PWC) → K.7 (composite)

K.1–K.6 are independent; K.7 synthesises all six.

## Papers

| Paper | Title | Stage | Score |
|-------|-------|-------|-------|
| K.0 | Compactness Taxonomy and Overview | planned | — |
| K.1 | Polsby-Popper Score | planned | — |
| K.2 | Reock Score | planned | — |
| K.3 | Convex Hull Ratio | planned | — |
| K.4 | Schwartzberg Score | planned | — |
| K.5 | Length-Width Ratio | planned | — |
| K.6 | Population-Weighted Compactness | planned | — |
| K.7 | Multi-Metric Composite and Court Guide | planned | — |

## Contracts

Every K.1–K.6 paper must:
- Define the metric with a formal mathematical expression
- Prove at least one non-trivial property (monotonicity, range, projection sensitivity)
- Report mean metric value for NC/WI/TX across ≥3 B-series structure algorithms
- Include a §5 Legal Usage section citing real case law
- Implement the metric in `crates/bisect-analysis/`

## Quantification

- Primary number: mean metric value for ratio-optimal on NC 2020
- Comparison: vs standard-bisect, prime-factor, moving-knife
- Legal threshold: documented court usage (where exists)
