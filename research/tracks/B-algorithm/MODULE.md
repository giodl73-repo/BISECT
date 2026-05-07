# Track B — Algorithm

**Module**: track-B
**Theme**: What is the METIS recursive bisection algorithm for congressional redistricting, does it work, and what are its principled variants?
**Papers**: 25
**Author**: Giovanni Della Libera
**Created**: 2026-05-07

---

## Tracks

### Track foundations

**Theme**: Does minimum-edge-cut recursive bisection produce compact, seed-stable, and partisan-neutral congressional districts at national scale?

**Chain**: `B.1+recursive-bisection` → `B.2+edge-weighted-bisection` → `B.3+multi-vs-edge` → `B.4+adaptive-bisection` → `B.5+nway-vs-recursive-general` → `B.6+computational-complexity` → `B.7+solution-space-and-seed-sensitivity`

**Arc**: B.1 establishes the base algorithm — METIS recursive bisection applied to congressional redistricting — and demonstrates a +22% compactness improvement over enacted maps. B.2 introduces edge-weighted bisection (geographic proximity weights), lifting compactness a further +22% to a total of +44%. B.3 isolates why: single-objective edge-cut dominates multi-constraint METIS for this problem. B.4 proves adaptive vs. recursive equivalence under standard conditions. B.5 provides the general nway vs. recursive comparison. B.6 characterizes the O(n log² n) / O(√log n) approximation complexity. B.7 closes the foundations track by proving the solution space is tight: seed CV < 2% for 96% of states, partisan outcome variance near zero.

### Track structure

**Theme**: Given that recursive bisection works, what bisection tree structure best serves each redistricting objective — compactness, county integrity, minority representation, temporal stability, or partisan proportionality?

**Chain**: `B.0+algorithm-design-overview` → `B.8+geosection-ratio-optimal-bisection` → `B.9+areasection-dual-population-area-constraint` → `B.10+subdivision-respecting-redistricting` → `B.11+apportion-regions` → `B.02+one-federal-law` → `B.12+proportional-section` → `B.13+nestsection-nested-multi-chamber` → `B.14+vrasection-minority-opportunity-bisection` → `B.15+stabilitysection-cross-census-stability`

**Arc**: B.0 frames the bakeoff across all structure modes. B.8 introduces GeoSection (ratio-optimal first bisection with isoperimetric normalisation). B.9 adds AreaSection (dual [population, area] constraint). B.10 introduces county-sticky weights, achieving 34% fewer county splits at only 3% compactness cost. B.11 presents ApportionRegions (prime-factor bisection), producing balanced partisan outcomes including NC 7D/7R; B.02 formalizes this as the one-sentence federal law (geographic completion of Huntington-Hill). B.12 reveals the proportionality paradox: sigma approaches zero for competitive states, and the Rodden gap is a Lorenz feasibility constraint not a target. B.13 addresses bicameral compatibility via NestSection. B.14 handles minority geographic alignment post-Callais. B.15 demonstrates cross-decade stability: same bisection tree structure 2000–2020.

### Track search

**Theme**: Given a bisection tree structure, which seed-selection and convergence strategy minimizes partisan variance while meeting the statutory balance threshold?

**Chain**: `B.16+convergence-sweep` → `B.17+parameter-sensitivity`

**Arc**: B.16 establishes the ConvergenceSweep statutory formula (T=600 seeds) that balances computational cost against partisan variance reduction. B.17 confirms parameter insensitivity: partisan outcomes are robust to ufactor, niter, and objtype tuning across 400+ runs.

### Track extensions

**Theme**: What advanced algorithmic techniques extend the bisection framework to address objectives the base algorithm cannot directly optimize?

**Chain**: `B.18+multi-reapportionment-stability` → `B.19+simulated-annealing` → `B.20+parallel-tempering` → `B.21+adaptive-multiscale` → `B.22+centroidal-voronoi` → `B.22b+cvd-geographic` → `B.23+bfs-growth`

**Arc**: B.18 establishes cross-reapportionment stability (what happens when states gain/lose seats). B.19 introduces simulated annealing bisection — cooling the edge-cut objective — accepted at 3.6/4. B.20 adds parallel tempering (multi-chain replica exchange MCMC), accepted at 3.6/4. B.21 presents adaptive multiscale MCMC with self-tuning alpha via Robbins-Monro, accepted at 3.5/4. B.22 introduces Centroidal Voronoi Districts (geometric district construction), accepted at 3.9/4; B.22b extends to geographic CVD. B.23 presents BFS region-growing (greedy geographic district packing), accepted at 3.6/4.

---

## Papers

| Paper | Tracks | Primary Number | Status | Venue |
|-------|--------|----------------|--------|-------|
| B.0+algorithm-design-overview | structure | bakeoff across 8 modes on 6 competitive states | draft | internal |
| B.02+one-federal-law | structure | one-sentence geographic completion of Huntington-Hill | draft | law review |
| B.1+recursive-bisection | foundations | +22% compactness vs. enacted maps (50 states × 3 decades) | ready | APSR |
| B.2+edge-weighted-bisection | foundations | additional +22% compactness lift via geographic weights | ready | APSR |
| B.3+multi-vs-edge | foundations | single-objective edge-cut dominates multi-constraint METIS | ready | APSR |
| B.4+adaptive-bisection | foundations | recursive ≡ adaptive under standard conditions | ready | APSR |
| B.5+nway-vs-recursive-general | foundations | nway vs. recursive general comparison | draft | APSR |
| B.6+computational-complexity | foundations | O(n log² n) / O(√log n) approximation guarantee | draft | JACM |
| B.7+solution-space-and-seed-sensitivity | foundations | CV < 2% for 96% of states; partisan variance ≈ 0 | ready | Pol. Analysis |
| B.8+geosection-ratio-optimal-bisection | structure | ratio-optimal first bisection + isoperimetric normalisation | ready | APSR |
| B.9+areasection-dual-population-area-constraint | structure | 76% seat-stable under ncon=2 [pop, area]; area_swing = 1.10 boundary | ready | APSR |
| B.10+subdivision-respecting-redistricting | structure | 34% fewer county splits at 3% compactness cost | draft | APSR |
| B.11+apportion-regions | structure | NC 7D/7R; 223D/209R national; prime-factor bisection tree | ready | APSR |
| B.12+proportional-section | structure | proportionality paradox: sigma → 0 for competitive states | ready | APSR |
| B.13+nestsection-nested-multi-chamber | structure | senate = 2× house spine compatibility | draft | APSR |
| B.14+vrasection-minority-opportunity-bisection | structure | minority geographic alignment post-Callais | draft | Yale LJ |
| B.15+stabilitysection-cross-census-stability | structure | same bisection tree structure 2000–2020 | draft | APSR |
| B.16+convergence-sweep | search | T=600 statutory seed formula | draft | APSR |
| B.17+parameter-sensitivity | search | partisan outcomes robust to all parameter tuning | draft | Pol. Analysis |
| B.18+multi-reapportionment-stability | extensions | cross-reapportionment stability analysis | draft | APSR |
| B.19+simulated-annealing | extensions | simulated annealing bisection; 3.6/4 accepted | ready | NeurIPS |
| B.20+parallel-tempering | extensions | parallel tempering replica exchange; 3.6/4 accepted | ready | NeurIPS |
| B.21+adaptive-multiscale | extensions | Robbins-Monro self-tuning alpha; 3.5/4 accepted | ready | ICML |
| B.22+centroidal-voronoi | extensions | geometric district construction; 3.9/4 accepted | ready | KDD |
| B.22b+cvd-geographic | extensions | geographic CVD extension | draft | KDD |
| B.23+bfs-growth | extensions | BFS region-growing greedy packing; 3.6/4 accepted | ready | AAAI |

---

## Module Arc

Track B is the methodological spine of the entire program. It progresses in four logical phases: foundations (B.1–B.7) establish that the base algorithm works and characterize its solution space; structure variants (B.0, B.02, B.8–B.15) show that the bisection tree can be shaped to serve any redistricting objective — geographic, partisan, legal, or temporal; search strategies (B.16–B.17) determine how to select among valid plans; and extensions (B.18–B.23) introduce algorithmic techniques from ML and MCMC literature to handle objectives the base algorithm cannot optimize. The central empirical claim unifying all four sub-tracks: minimum-edge-cut bisection with geographic weights produces +22–44% compactness improvement, seed CV < 2%, and near-zero partisan variance — establishing algorithmic redistricting as both reproducible and non-partisan. Track B is a prerequisite for Tracks C (validation), D (VRA compliance), F (state legislative), G (ensemble context), and H (search extensions).
