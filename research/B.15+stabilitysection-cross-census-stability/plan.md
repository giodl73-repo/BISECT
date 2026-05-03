# B.15 — StabilitySection: Cross-Census Stability of Optimal Redistricting Maps

**Paper Type**: Algorithm Theory + Empirical Study  
**Status**: Planning (2026-05-02)  
**Series**: B (Algorithm Design)  
**Depends on**: B.7 (seed stability, solution space), B.8 (GeoSection — the algorithm whose stability is tested)  
**Companion**: C.2 (cross-census validation), C.3 (temporal stability at the data pipeline level)

---

## Core Idea

B.7 established that minimum-edge-cut maps are stable across seeds *within* a
single census year: for most states, the partisan outcome variance across 1000
seeds is small (≤ 5pp proportionality gap). StabilitySection asks the deeper
question: are they stable across *census years*?

If the GeoSection-optimal map for a state using 2000 census data is also the
GeoSection-optimal map for 2010 and 2020 data, that is the strongest possible
form of **geographic determinism** — the natural partition is determined by
the state's geographic structure, not by which decennial population snapshot
happens to be in force. Three independent censuses, twenty years apart, all
converge to the same answer.

**The compelling version of this claim**: If a state has a census-stable map,
a court can be told not merely "this is the compact map" but "this is the
compact map — and it was the compact map in 2000, and in 2010, and again in
2020. The geography of this state has always implied these boundaries. No
decennial politics could have changed this outcome."

**The interesting counter-case**: States that *change* their natural ratio
between census years reveal population dynamics creating new geographic
structures. A state where GeoSection returned 1:7 (Milwaukee peel) in 2000
but shifts to 4:4 by 2020 has undergone a structural change in its
population-geography relationship. This is not an algorithmic failure — it
is a signal that the state's human geography has reorganized.

---

## Relationship to Existing Papers

- **B.7** studies stability across random seeds at fixed census year. This
  paper studies stability across census years at fixed algorithm. Together
  they provide a 2D stability picture: (seed-stable, census-stable) states
  are the most defensible; (seed-sensitive, census-sensitive) states require
  the most careful procedural justification.
- **C.2 (cross-census validation)** checks that the *pipeline* produces
  consistent outputs across years. StabilitySection is about the *mathematical
  property* of the algorithm — whether the optimal answer changes when the
  inputs (population data) change.
- **C.3 (temporal stability)** examines district assignment drift over time.
  StabilitySection provides the theoretical underpinning: algorithmic stability
  is the *cause* of the temporal stability observed in C.3.

---

## Research Questions

**RQ1 — Natural ratio stability**: Does GeoSection select the same first-level
split ratio i*:(k-i*) for a given state across 2000, 2010, and 2020 data?

**RQ2 — Full tree stability**: Beyond the first level, does the complete
bisection tree structure remain the same? (A state might have the same level-1
ratio but different level-2 sub-splits as urban areas shift.)

**RQ3 — District assignment stability**: What fraction of census tracts are
assigned to the same district across census years? Measured by Jaccard
similarity between the 2010-optimal and 2020-optimal assignment.

**RQ4 — Partisan stability**: Does the D/R seat count remain the same across
census years, even as exact tract assignments shift?

**RQ5 — Lorenz curve stability**: How much does the optimal ratio p* (from the
population-geography Lorenz curve used in GeoSection's ratio scan) change
between census years? A stable p* means the state's population-geography
relationship is durable; a shifting p* reveals demographic reorganization.

**RQ6 — Predictive power of stability**: Can we identify in advance (from
geographic features: urbanization index, population growth rate, spatial
Gini coefficient) which states will be stable vs. unstable? This would allow
the algorithm to flag "high-risk" states where census-year-specific population
data matters most.

---

## Theoretical Frame: Two Sources of Change

A map can change between census years for two reasons:

**Type I — Population shift within a fixed geography**: People move between
tracts, changing the population balance targets. The tract graph itself is
unchanged (same boundaries, same adjacency). The algorithm must redraw to
achieve the new population balance. This is analogous to a smooth perturbation
of the METIS input weights.

**Type II — Tract boundary redesign**: Between 2000 and 2010, or 2010 and
2020, the Census Bureau redraws some tract boundaries. A tract split (one
2000 tract becomes two 2010 tracts) or merge changes the graph topology.
This is a structural change that can propagate to any downstream districts.

StabilitySection controls for these separately:
- **Population-only perturbation stability**: Fix the 2010 tract graph; run
  GeoSection with 2000, 2010, and 2020 population weights. Measure stability.
- **Full cross-year stability**: Use the actual 2000/2010/2020 tract graphs.
  Measure stability including topology changes.

The difference between these two measures is the contribution of Census
tract redesign to map instability — a finding of independent interest.

---

## Algorithm

StabilitySection is not a new redistricting algorithm. It is an **analysis
framework** that runs GeoSection three times per state and compares outputs.

```
StabilitySection(state, years=[2000, 2010, 2020]):

  for year in years:
    G_year = load_adjacency_graph(state, year)
    map_year = GeoSection(G_year, k=seats(state), N=100)
    ratio_year[year] = map_year.level1_ratio
    tree_year[year] = map_year.bisection_tree
    assignment_year[year] = map_year.tract_assignments
    lorenz_year[year] = map_year.lorenz_curve_pstar

  # Compute stability metrics
  ratio_stable = all(ratio_year[y] == ratio_year[2020] for y in years)
  tree_stable = tree_edit_distance(tree_year[2000], tree_year[2020])
  jaccard_2010_2020 = district_jaccard(assignment_year[2010],
                                       assignment_year[2020])
  lorenz_drift = |lorenz_year[2000].pstar - lorenz_year[2020].pstar|

  return StabilityReport(ratio_stable, tree_stable,
                         jaccard_2010_2020, lorenz_drift)
```

**District Jaccard similarity**: Two tract assignments are compared by
mapping each 2010 district to the most-overlapping 2020 district (by
tract-population overlap), then computing the fraction of the population
covered by the matched pairs.

```
Jaccard(A_2010, A_2020):
  for each district d in A_2010:
    d' = argmax_{d'' in A_2020} pop(d ∩ d'')
    match(d) = d'
  score = Σ_{d} pop(d ∩ match(d)) / Σ_{d} pop(d ∪ match(d))
```

**Tree edit distance**: Count the number of (node addition/deletion/relabeling)
operations to transform one bisection tree into another. A stable tree has
edit distance 0; a fully reshuffled tree has edit distance proportional to k.

---

## Census-Stability Score

The headline metric for each state:

```
CSS(state) = (jaccard_2010_2020 + jaccard_2000_2020 + ratio_stable) / 3
```

where each component is normalized to [0, 1]. A CSS of 1.0 means the
algorithm produces identical district assignments across all three censuses.
CSS ≥ 0.90 is "highly stable" (the target for constitutional strength
arguments). CSS < 0.70 is "structurally volatile."

---

## Empirical Plan

### States of Interest

**Expected high stability** (low growth, stable urban structure):
- IA, KS, NE, MO, IN (Midwest row states — stable rural geography)
- ME, VT, NH (New England — low growth, old urbanization)
- WV, MS, AL (declining population, minimal new development)

**Expected low stability** (rapid growth, population redistribution):
- TX (38 seats — large growth in Houston/DFW/Austin metro; rural decline)
- AZ (growing from 8→9→9 seats — Phoenix suburban expansion)
- GA (11→13→14 seats — Atlanta suburban expansion)
- FL (25→25→28 seats — south Florida + I-4 corridor growth)
- CO (7→7→8 seats — Denver metro growth)
- NV (2→3→4 seats — Las Vegas growth)

**Expected ratio-change states** (cities growing to change peel geometry):
- WI: Milwaukee may shift from a clean 1:7 peel (2000) to a contested
  2:6 as the Fox Valley grows
- PA: Philadelphia suburbs shifting; may change the natural PA ratio
- MI: Detroit decline may change whether the city anchors a clean peel

### Experiments

1. **50-state cross-census sweep**: Run GeoSection for all 50 states × 3
   years × 50 seeds. Compute CSS for each state. Produce a national map of
   CSS values. This is the core empirical result.

2. **Natural ratio change tracking**: For the 10 "ratio watch" states
   (WI, PA, MI, TX, AZ, GA, FL, CO, NV, NC), report the level-1 natural
   ratio for each census year. Plot the ratio trajectory as a population
   geography time series.

3. **Lorenz curve drift analysis**: For all 50 states, plot the 2000/2010/2020
   Lorenz curves on the same axes. States with tightly overlapping curves are
   population-geography stable. States with diverging curves reveal demographic
   reorganization. Report the Lorenz drift metric (area between 2000 and 2020
   curves) as a predictor of CSS.

4. **Population-only vs. full stability decomposition**: For a 10-state subset,
   run the "population-only perturbation" experiment: fix the 2010 graph,
   substitute 2000 and 2020 population weights. Compare stability to the
   full cross-year stability (different graphs). Attribute instability to
   population drift vs. tract redesign.

5. **Predictive model for CSS**: Fit a regression:
   ```
   CSS(state) ~ f(population_growth_rate, urbanization_index,
                  spatial_gini, seat_count_change, tract_redesign_rate)
   ```
   which states have low CSS is predictable from pre-algorithm features?

### Metrics

| Metric | Definition |
|--------|-----------|
| `css` | Census-stability score (composite, 0-1) |
| `ratio_stable` | Same level-1 ratio across all 3 years (bool) |
| `tree_edit_dist` | Tree edit distance 2000→2020 |
| `jaccard_2010_2020` | District assignment Jaccard (2010 vs. 2020) |
| `lorenz_drift` | Area between 2000 and 2020 Lorenz curves |
| `ratio_2000`, `ratio_2010`, `ratio_2020` | Level-1 natural ratio per year |
| `pstar_drift` | |p*(2000) - p*(2020)| |
| `seat_change` | Change in congressional delegation size (0, +1, +2) |

---

## Expected Findings

**National stability pattern**: ~30 states will have CSS ≥ 0.90. These are
states with stable population geography — mostly Midwest, Great Plains, and
New England. Their natural ratios are identical across all three censuses,
and district boundaries shift by at most 2-3 tracts at the margins (as
population balance targets are updated).

**Unstable states cluster in the Sun Belt**: TX, AZ, GA, FL, NV will have
CSS ≤ 0.75. The dominant instability source is not tract redesign but
population redistribution: suburban growth creates new compact geographic
regions that attract different level-2 sub-splits.

**Ratio stability is near-universal at level 1**: Even in unstable states,
the first-level ratio tends to be stable (e.g. Texas level-1 is 19:19
regardless of census year — the state is too large for a single dominant
city to anchor a 1:k-1 peel). Instability accumulates at levels 2-4 of the
bisection tree, where local geography matters.

**Lorenz drift predicts CSS strongly** (expected r ≥ 0.75): States where the
population-Lorenz curve shifts significantly between censuses will have low CSS.
This finding supports using Lorenz drift as a pre-redistricting "census
stability alert" — flagging states where the algorithm should be rerun even
for off-year apportionment purposes.

---

## Legal / Policy Argument

**Census-stable maps deserve the strongest constitutional protection.** A map
that is optimal under 2000, 2010, and 2020 data is not a product of any
particular census cycle's partisan distribution. It is the *geographic
structure of the state*, expressed consistently over twenty years of population
change. No redistricting commission, court, or legislature should be able to
override a CSS ≥ 0.90 map without demonstrating that the deviation serves a
compelling interest — because the deviation would, by definition, deviate from
the state's stable geographic structure.

**For litigation**: CSS provides a quantitative answer to the question "was this
the right map?" If the challenged congressional map differs from the GeoSection-
optimal map, and GeoSection was census-stable for this state, then the challenger
can argue: "The natural map has existed for two decades and three censuses. The
legislature chose a different map. That choice cannot be explained by geography
and therefore must be explained by politics."

**For statute design**: A proposed Districting Integrity Act could require that
maps be published with their CSS score. A map with CSS ≥ 0.90 may be adopted
with administrative certification. A map with CSS < 0.70 triggers a heightened
review process — because the algorithm itself says the current census data is
producing a structurally different result from prior censuses.

**Anti-gerrymandering argument**: A gerrymander that would be CSS-unstable is
self-defeating over time. The next census will naturally shift populations in
ways that undermine the packed/cracked design. Census-stable maps are more
durable and require less decennial litigation.

---

## Dependencies

- **B.7 (seed stability)**: StabilitySection is the census-year generalization
  of B.7's seed-sensitivity analysis. Both papers should be read together:
  B.7 measures within-year variance; B.15 measures across-year variance. The
  joint claim is that GeoSection-optimal maps are stable in both dimensions.
- **B.8 (GeoSection)**: The algorithm being tested. All stability measurements
  are GeoSection-specific; the CSS metric could be applied to other algorithms
  (AreaSection, SubdivisionSection) in follow-on work.
- **C.2 (cross-census validation)**: C.2 checks pipeline infrastructure;
  B.15 checks algorithmic invariance. C.2 is a prerequisite for running B.15
  experiments (all three census years must be in the pipeline).
- **C.3 (temporal stability)**: C.3's empirical findings motivate B.15's
  theoretical framing. If C.3 found high temporal stability, B.15 explains
  why (algorithmic census-invariance). If C.3 found instability, B.15 diagnoses
  which states are algorithmically volatile.

---

## Target Venue

*Political Analysis* (quantitative methods for electoral research) or  
*Annals of the American Association of Geographers* (spatial analysis)  
Backup: *Journal of the Royal Statistical Society Series A* (statistics in public policy)

---

## Open Questions

1. **Tract alignment across years**: 2000, 2010, and 2020 tract boundaries are
   not identical. How do we compare district assignments when the underlying
   units differ? (Proposed: use 2020 tracts as the reference; map 2000/2010
   tract assignments to 2020 by spatial intersection, weighted by population.)

2. **Seat count changes**: States that gained or lost seats between censuses
   (e.g. TX: 30→32→36→38) cannot have the same districts — k itself changed.
   For these states, compare the *bisection tree structure* (normalized to a
   common depth) rather than raw assignments.

3. **Is CSS the right stability metric?** Jaccard measures district-by-district
   overlap, but districts can shift substantially while maintaining the same
   partisan outcome. Should CSS weight by political significance (battleground
   districts count more) or be purely geographic?

4. **2000 data quality**: 2000 Census tracts are the coarsest (fewer blocks per
   tract in rural areas). Does the coarser resolution mechanically increase
   instability, or does GeoSection naturally smooth over this?

5. **Inter-census stability**: The current plan uses three census years. Could
   the framework also test stability using *ACS 5-year estimates* between
   censuses? If yes, stability claims could be updated annually rather than
   decennially.
