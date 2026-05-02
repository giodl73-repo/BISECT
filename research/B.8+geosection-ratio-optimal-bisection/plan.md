# B.8 — GeoSection: Ratio-Optimal, Direction-Aware Recursive Bisection

**Paper Type**: Algorithm Design + Empirical Comparison
**Status**: Planned (stub 2026-05-01)
**Target Venue**: Journal of Computational Geometry / Political Analysis
**Series**: B (Algorithm Design)
**Depends on**: B.7 (solution space, MEC convergence, CompactBisect)

---

## Core Idea

Standard recursive bisection always splits ⌊k/2⌋ : ⌈k/2⌉ (as close to equal
as possible) and applies a fixed edge-cut objective. GeoSection adds two
orthogonal innovations:

**Innovation 1 — Ratio search**: For each recursion level, try ALL feasible
split ratios (1:k-1, 2:k-2, ..., ⌊k/2⌋:⌈k/2⌉) with N seeds each. The
ratio with the globally minimum MEC across all seeds is the "geographically
natural" division of the subregion. A state like Tennessee (4× longer E-W
than N-S) may have a natural split ratio of 3:6 (not 5:4) where the N-S
boundary is much shorter — geography chooses the ratio, not the algorithm.

**Innovation 2 — Directional edge weights (optional, λ controls strength)**:
Compute the minor axis of the current subregion's geographic footprint (via
PCA on tract centroids). Penalise edges that run PARALLEL to the desired cut
direction (these edges, if crossed, create zigzags perpendicular to the cut).

  Modified weight: w'(u,v) = w(u,v) × (1 + λ × |sin(θ(u,v))|)

  where θ(u,v) = angle between edge (u,v) direction and the minor-axis cut
  direction. λ=0 recovers standard MEC; λ→∞ forces a straight cut.

**Key structural property — re-rotation at every level**: After bisecting a
subregion, each resulting half is a NEW geographic shape with its OWN minor
axis. The minor axis must be recomputed from the actual tract centroids of
THAT half before the next level's bisection. A horizontal cut of Tennessee
produces a northern half (roughly rectangular) and a southern half (with the
Memphis corner) — each needs its own PCA before the next cut. The algorithm
is self-adapting: no global coordinate system is imposed.

---

## Algorithm

```
GeoSection(subgraph G, k districts, N seeds, λ penalty):

  Base case: if k == 1, return G as single district.

  Step 1: Compute minor axis direction d for G.
    - Collect centroids of all tracts in G.
    - Run 2D PCA on centroids.
    - Minor axis = eigenvector of SMALLER eigenvalue.
    - (Minor axis = direction of narrowest geographic extent of G.)

  Step 2: For each ratio r ∈ {(i, k-i) : i = 1..⌊k/2⌋}:
    a. Compute directionally-adjusted edge weights w'(u,v) using d and λ.
    b. Run N seeds of METIS on G with:
         - vertex weights = [population, land_area]  (dual constraint)
         - target partition = [i/k, (k-i)/k]  for BOTH pop AND area
         - edge weights = w' (penalised if λ > 0)
    c. Record minimum MEC across N seeds for this ratio.

  Step 3: Select ratio r* = argmin over ratios of minimum MEC.
    (The geographically natural split: the ratio where the
     minimum-length cut is shortest.)

  Step 4: Recurse:
    GeoSection(left_half,  r*[0] districts, N, λ)
    GeoSection(right_half, r*[1] districts, N, λ)
```

---

## Dual Vertex Weights: Population + Area

METIS supports ncon=2 (two balance constraints simultaneously).
- Constraint 1: population — each part gets pop_i = i × (total_pop/k)
- Constraint 2: land area — each part gets area_i = i × (total_area/k)

Balancing BOTH ensures neither half is a tiny geographic sliver with dense
population (which would make a compact but geographically absurd district).
The population constraint satisfies the legal requirement; the area constraint
enforces geographic fairness.

Vertex weights for tract v: [population(v), ALAND(v)]

---

## The λ=0 vs λ>0 Comparison

A central empirical question: does the directional penalty (λ>0) actually
produce straighter, more aesthetically neutral cuts — and does it change the
partisan outcome?

Run the full algorithm at:
- λ=0: standard MEC with ratio search (no directional bias)
- λ=1: moderate directional penalty
- λ=5: strong directional penalty
- λ=10: near-straight-line enforcement

For each λ, record:
- Minimum MEC at the optimal ratio
- Number of edges crossed at level-1 (fewer = straighter)
- "Straightness ratio": (level-1 EC) / (state minor axis km)
  — closer to 1.0 = straighter
- Partisan outcome (D seats, proportionality gap)
- Map appearance (qualitative, rendered PNG)

**Hypothesis H1**: Higher λ → fewer edges crossed → straighter boundary →
  higher MEC (you pay a cost for straightness).
**Hypothesis H2**: The partisan outcome is INSENSITIVE to λ — the geographic
  structure of the state determines the outcome, not the precise path of the
  line.
**Hypothesis H3**: The optimal ratio r* is INSENSITIVE to λ — the natural
  ratio is geometric, not a product of the cut's wiggliness.

If H2 and H3 hold, λ is purely an aesthetic parameter: any λ gives the same
partisan result, but higher λ gives a cleaner-looking map. This closes the
"you drew a wiggly line to favor your party" attack permanently — the partisan
outcome is the same regardless of how straight the line is.

---

## Why This Matters Legally

The directional penalty makes the redistricting algorithm's choices
transparently geometric:

1. "We computed the minor axis of each subregion from public TIGER centroids."
2. "We penalised zigzag cuts — cuts that cross parallel-direction boundaries."
3. "The straightness parameter λ is a statutory value published before any
   runs (in the parameter table, §107(c))."
4. "We ran with λ=0 AND λ=5 and got the same partisan outcome — proving the
   line's wiggliness doesn't affect who wins seats."

Point 4 is the killer argument: if a challenger claims "you drew a wiggly
line to gerrymander," the response is "here's the same run with a straight
line — same result."

---

## Research Questions

**RQ1**: Does GeoSection's ratio search identify a "natural" split ratio
  that is consistently smaller (lower MEC) than the standard 50/50 split?
  For which states is the natural ratio far from 50/50?

**RQ2**: What is the rate of convergence for ratio-optimal MEC vs standard
  MEC? Does trying all ratios require more or fewer seeds per ratio to
  achieve partisan stability?

**RQ3**: Does the dual vertex weight (pop + area) produce observably more
  compact or proportional districts than pop-only?

**RQ4**: How sensitive is the partisan outcome to λ? (If insensitive:
  big legal win. If sensitive: λ must be justifiable.)

**RQ5**: Do the "natural" split ratios vary systematically with state
  elongation (major/minor axis ratio)? Prediction: elongated states
  (TN, NC) have natural ratios far from 50/50; compact states (GA, MI)
  have natural ratios near 50/50.

---

## Computational Cost

For state with k districts:
- Ratios to try: ⌊k/2⌋ (e.g., 4 for k=8, 19 for k=38)
- Seeds per ratio: N = 100-1000
- λ values: 4 (0, 1, 5, 10)
- Total METIS calls per state per recursion level: ⌊k/2⌋ × N × 4

For WI (k=8): 4 × 100 × 4 = 1,600 calls ≈ 15 min
For PA (k=17): 8 × 100 × 4 = 3,200 calls ≈ 30 min
For TX (k=38): 19 × 100 × 4 = 7,600 calls ≈ 75 min

Full 50-state sweep at N=100: ~2,000 CPU hours total. Run in parallel.
Initial paper: 8 focal states × full analysis ≈ 200 CPU hours.

---

## Connection to B.7

B.7 established:
- MEC convergence requires depth (false floors at N=200 for GA, TN, WI)
- Fiedler bound is loose for geographic graphs
- Empirical convergence tail (300+ seeds) is the practical criterion

B.8 extends this:
- GeoSection's ratio search also needs convergence testing per ratio
- False floors may appear for some ratios and not others
- The natural ratio r* may itself converge (stabilise) as N increases
- The λ=0 case of GeoSection is exactly B.7's CompactBisect with ratio search added

---

## Implementation Notes

Minor axis computation (Step 1):
- Load tract centroids from TIGER geoid → (lat, lon) centroid mapping
- Run 2D PCA: compute covariance matrix of centroid coordinates
- Eigenvector of smaller eigenvalue = minor axis direction
- This is O(n) once centroids are loaded

Edge direction computation:
- For edge (u,v): direction = angle of (centroid_v - centroid_u)
- θ(u,v) = |angle(u,v) - minor_axis_angle|
- Penalty: |sin(θ(u,v))| ∈ [0,1]

METIS dual weight:
- In bisection_runner.rs: pass ncon=2, vwgt=[pop, area], tpwgts=[pop_frac, area_frac]
- METIS 5.x supports this via the ubvec parameter

---

## See Also

- `B.7` — MEC convergence, false floors, CompactBisect
- `redist/crates/redist-cli/src/bisection_runner.rs` — METIS interface
- `redist/crates/redist-data/src/tiger.rs` — ALAND field (tract area)
- `docs/legal/MODEL_FEDERAL_STATUTE.md` — §107(c) parameter table
