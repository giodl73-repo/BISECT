# Canonical Compactness Baseline Reference Table
**Cross-track document** — cite this table when reporting any compactness improvement figure.
**Last updated**: 2026-05-09
**Purpose**: Resolve the B2.1 board inconsistency (B.2: +56%, B.1: +22%, F.5: +20%)

---

## The Three Valid Baselines

All three figures (+56%, +22%, +20%) are correct — they measure different things.
Any paper reporting a compactness improvement **must** label which baseline it uses.

| Figure | Baseline | Scope | Paper |
|--------|----------|-------|-------|
| **+56%** | Unweighted METIS bisection (same algorithm, no edge weights) | All 50 states, 2020 congressional | B.2 |
| **+22%** | Enacted 2020 congressional maps | All 50 states, 2020 congressional | B.1, A.0, A.5 |
| **+20%** | Enacted state legislative maps | 50 states, 2020 state house | F.5 |

---

## Canonical Polsby-Popper Point Estimates

| Level | Mean PP | Temporal scope | Source |
|-------|---------|----------------|--------|
| Congressional algorithmic (2020) | 0.361 | 2020 census | C.0, B.2 |
| Congressional algorithmic (2010) | 0.358 | 2010 census | C.2 |
| Congressional algorithmic (2000) | 0.352 | 2000 census | C.2 |
| Congressional enacted (2020) | 0.295 | 2020 enacted | C.5 |
| State house algorithmic (tract) | 0.381 | 2020 census | F.1 |
| State house algorithmic (block-group) | 0.401 | 2020 census | F.1 |
| State house enacted | 0.305 | 2020 enacted | F.5 |
| Congressional unweighted METIS | 0.231 | 2020 census | B.2 |

---

## Required Citation Language

When reporting a compactness improvement, use one of these exact phrases:

- `+56\% more compact than unweighted METIS bisection \citep{dellaLibera2026edgeWeighted}` — for the contribution of edge weighting
- `+22\% more compact than enacted 2020 congressional maps \citep{dellaLibera2026edgeWeighted, dellaLibera2026recursive}` — for the total algorithmic advantage
- `+20\% more compact than enacted state legislative maps \citep{dellaLibera2026compactScale}` — for state house maps

**Do not** use "more compact" without specifying the baseline.

---

## Cross-Citing Papers

The following papers report compactness figures and should cite this table:
- B.1, B.2: define the two congressional baselines
- C.5, C.7: provide CI around the +22% figure
- F.5: defines the state house baseline
- A.0, A.1, A.2, A.5: synthesis documents — use the +22% enacted baseline for broad audiences
