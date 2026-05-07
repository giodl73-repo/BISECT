# B.9 — AreaSection: Simultaneous Population and Land-Area Balance

**Paper Type**: Algorithm Design + Empirical Comparison
**Status**: Draft complete (2026-05-03)
**Series**: B (Algorithm Design)
**Depends on**: B.8 (GeoSection — shared infrastructure, normalisation)
**Companion**: B.8 — GeoSection (ratio-optimal bisection without area constraint)

---

## Core Idea

GeoSection's ratio-optimal bisection selects the most compact split but does
not constrain land area, producing **urban peeling** (1:k-1 splits at every
level for dense-city states). AreaSection adds a second METIS balance
constraint (ncon=2) requiring each half to receive ~50% of land area (±10%
swing), forcing near-equal geographic splits for most states.

**Key results (50-state sweep, 2020):**
- IL: 1:16 → 8:9, NY: 1:25 → 13:13, FL: 1:27 → 14:14
- GA, MA stay asymmetric (geographic concentration too strong)
- WI + NC: same seat count as GeoSection, different competitiveness profile

**Central claim**: AreaSection is a geographic fairness mechanism, not a
partisan mechanism. Seat counts are stable across algorithms; competitiveness
profile changes (GeoSection creates knife-edge seats from urban peeling;
AreaSection creates cleaner partisan sorting).

---

## Algorithm Summary

1. Compute population-area Lorenz curve → natural ratio p* + feasibility mask
2. For each feasible ratio i:(k-i), run N seeds with ncon=2 METIS
   - tpwgts = [i/k, 0.5, (k-i)/k, 0.5]
   - ubvec = [1.001, 1.10] (tight pop, loose area)
3. Select ratio minimising EC/sqrt(min(i,k-i)) (isoperimetric normalisation)
4. Recurse with GeoSection ncon=1 (area constraint only at first level)

---

## Data

- 50-state sweep: outputs/area_sweep_v1/
- WI comparison: outputs/geo_wi_cmp/ vs outputs/area_wi_cmp/
- NC comparison: from bisection_runner.rs run in session

---

## Status

- [x] Algorithm implemented (bisection_runner.rs, vertex_weights.rs)
- [x] 50-state sweep run (44/50 succeeded)
- [x] WI + NC partisan comparison
- [x] Paper draft complete
- [ ] Figures (Lorenz curve, natural ratio map)
- [ ] Fix 6 large-state balance failures
- [ ] Panel submission
