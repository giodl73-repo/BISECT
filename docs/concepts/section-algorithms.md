# Section Algorithms: The B-Series Algorithm Family

## Short version

All algorithms in the B series are "section algorithms" — they define how the
bisection tree is structured, how ratios are selected, and what constraints
METIS optimises. The foundational result (B.1-B.7) establishes that
edge-weighted recursive bisection is the correct baseline. The section
algorithms (B.8-B.15) extend the baseline by varying structure or constraints.
The search algorithms (B.16, H.0-H.1) vary how the seed space is explored.

---

## Taxonomy

```
B-series
├── Foundations (B.1-B.7)           — standard bisection, correctness, complexity
├── Structure algorithms (B.8-B.15) — varied tree topology or METIS constraints
│   └── (excludes County-Sticky B.10 — see Weights layer below)
├── Weights layer (B.10)            — edge weight modifications to standard bisection
└── Search algorithms (B.16, H.0-H.1) — seed exploration strategies
```

All section algorithms are composed through the three-layer compositor. The
structure layer selects the algorithm; the weights and search layers apply
independently. See [three-layer-compositor.md](three-layer-compositor.md) for
the full composition model.

---

## Foundations (B.1-B.7)

These papers establish the baseline and prove it is correct.

**B.1 (Recursive Bisection)**: Defines the binary bisection tree. For a state
with k districts, the algorithm recurses until each leaf is a single district.
See [recursive-bisection.md](recursive-bisection.md).

**B.2 (Edge-Weighted Bisection)**: Proves that using TIGER boundary lengths as
METIS edge weights minimises perimeter automatically. Mean Polsby-Popper
improves by 56% over unweighted bisection across all 50 states.
See [edge-weighted-bisection.md](edge-weighted-bisection.md).

**B.3 (Single-Objective vs. Multi-Constraint)**: Proves that the
single-objective formulation (minimise EC, enforce population balance as a hard
constraint) outperforms multi-constraint approaches that attempt to optimise
compactness and population simultaneously.

**B.4 (Adaptive Bisection)**: Level-by-level parameter tuning.

**B.5 (N-Way vs. Recursive)**: Characterises when direct k-way partition
outperforms iterated bisection.

**B.6 (Computational Complexity)**: METIS runs in O(m log n). Full state sweep
runs in O(k * m log n) for k districts and a graph with n tracts and m edges.

**B.7 (Seed Sensitivity)**: The last improving seed occurs before index 1,023
for all 50 states across all three census years. This establishes the empirical
basis for the T=600 stopping criterion in B.16.

---

## Structure algorithms (B.8-B.15)

### GeoSection (B.8)

**CLI**: `--partition-mode geosection` / `structure: ratio-optimal`

GeoSection solves the "caterpillar problem" in standard bisection. Without
ratio optimisation, METIS at the top level often produces a 1:(k-1) split,
carving off a single compact urban district and leaving k-1 districts in a
long strip. This happens because 1:(k-1) splits have intrinsically short
boundaries — a small region can be enclosed cheaply.

The key innovation is the **isoperimetric normalisation**. At the first
bisection level, GeoSection tries all split ratios from 1:(k-1) through
floor(k/2):ceil(k/2). For each ratio i:(k-i), it runs N METIS calls and
records the minimum edge-cut EC_min(i:(k-i)). It then selects the ratio that
minimises:

```
EC_min(i:(k-i)) / sqrt(min(i, k-i))
```

The denominator sqrt(min(i, k-i)) corrects for the geometric fact that
smaller-fraction splits have intrinsically shorter boundaries. Without it,
ratio 1:(k-1) would win by default for any state with a compact urban core.
With it, the competition is fair: a 1:(k-1) split must be genuinely more
compact than an 8:6 split, not just shorter because it encloses less area.

After selecting the ratio at level 1, subsequent levels use standard bisection.
Each subregion independently selects its own natural ratio.

**NC result**: North Carolina (k=14=7x2) produces a 5D/9R outcome that is
stable across seeds (CV < 2% across T=10,000 seeds (B.7)). The first-level
ratio scan selects the 7:7 split. The 5D/9R outcome reflects North Carolina's
geographic sorting of Democratic voters along the I-85 corridor and Republican
voters in the rural Piedmont and mountains.

---

### AreaSection (B.9)

**CLI**: `--partition-mode areasection` / `structure: ratio-optimal-area`

AreaSection extends GeoSection with a dual population+area constraint (METIS
ncon=2). Each vertex carries two weights: population and land area (ALAND from
TIGER, in hectares). METIS must simultaneously satisfy:

- Population balance: tight tolerance, ubvec[0] = 1.001 (0.1%)
- Area balance: loose tolerance, ubvec[1] = 1 + area_swing (default 10%)

When constraints conflict — common in states with dense urban cores — METIS
prioritises the tighter population constraint. The area constraint acts as a
soft signal: the algorithm tries to split area roughly equally but will violate
this if needed to maintain population balance.

**Lorenz feasibility filter**: Before running any METIS calls, AreaSection
computes the population-area Lorenz curve (tracts sorted by density
rho_v = population/area). This identifies ratios that are geometrically
infeasible for a given state: if the minimum area fraction achievable for a
target population fraction does not overlap the allowed area window, the ratio
is skipped. This identifies states where geographic concentration makes
equal-area splitting impossible.

**Results**: 76% seat stability versus standard bisection at the
area_swing=1.10 regime boundary. The Lorenz filter identifies approximately
8 states per census year where geographic population concentration makes
area balance infeasible at strict tolerances.

---

### ApportionRegions (B.11)

**CLI**: `--partition-mode apportion-regions` / `structure: prime-factor`

ApportionRegions is the geographic completion of the Huntington-Hill
apportionment. Just as Huntington-Hill determines seat counts by priority
ordering, ApportionRegions determines the bisection tree by prime factorisation.

**Split prescription**: At each tree node with k target districts:

1. If k <= 3: split into k equal parts directly.
2. If k is composite: let p = largest prime factor of k. Split into p equal
   parts, each with target k/p districts.
3. If k is prime and k > 3: fall back to a 2-way binary split with targets
   floor(k/2) and ceil(k/2).

Examples:
- k=8=2^3: three binary levels (standard bisection)
- k=9=3^2: two ternary levels
- k=14=7x2: 7-way primary split, then 7 bisections
- k=17 (prime): binary fallback 9+8

**The reuse property**: Two seat counts sharing the same largest prime factor
(e.g., k=34=2x17 and k=51=3x17) produce the same top-level 17-way partition
of the state. This means ApportionRegions can cache top-level partitions across
decennial reapportionments when the prime factorisation is stable.

**NC result**: k=14=7x2 produces 7D/7R. The 7:7 first-level split equally
divides North Carolina between its Democratic-leaning eastern districts and
Republican-leaning western districts.

**National 2020 result**: 223D/209R across all 50 states. Among all plans
tested in the GerryChain ensemble for WI, GA, PA (1,000 steps each), the
bisection plan falls in the 0.1–0.2th percentile for edge cut — more compact
than 99.8%+ of sampled valid plans. NC is at the 50th percentile due to
geographic convergence.

---

### ProportionalSection (B.12)

**CLI**: `--partition-mode proportional-section` / `structure: standard-bisect`
with `weights: proportional`

ProportionalSection uses METIS ncon=2 vertex weights [population, D_votes].
The bisection targets are set to allocate Democratic seats proportionally:
if Democrats have d% of the statewide vote and k seats are at stake, the
target Democratic seat fraction is HH(d, 1-d, k)/k.

**The proportionality paradox**: For competitive states (d approximately 50%),
the proportionality constraint produces sigma approximately 0 — essentially a
single deterministic outcome. The partisan outcome is determined by geography,
not by the algorithm. The algorithm cannot "produce proportionality" for
competitive states because the Lorenz geometry of vote distributions in such
states does not support geographically feasible equal-vote splits.

**The Rodden gap**: The systematic Republican advantage in seat share for
equal vote shares is a Lorenz feasibility constraint, not a target. Urban
geographic concentration of Democratic voters makes equal-vote, equal-seat
splitting geometrically impossible in many states. ProportionalSection
identifies this impossibility formally rather than treating it as a political
outcome.

**Legal note**: Stat 104(e) prohibits proportional-section for federal
congressional districts. It is valid for state legislative redistricting
where proportionality standards apply.

---

### NestSection (B.13)

**CLI**: `structure: nest-section` (planned)

NestSection produces compatible house+senate district spines from a single
factorisation tree. Given a house seat count H and senate seat count S, the
algorithm finds gcd(H,S) to determine the shared top-level partition. House
and senate districts are nested: each senate district is the union of exactly
H/S house districts.

The spine structure enables multi-chamber redistricting from a single METIS
run at the top level, with independent optimisation at the house and senate
levels.

---

### VRASection (B.14)

**CLI**: `--partition-mode vra-section` / `structure: ratio-optimal-vra`

VRASection extends GeoSection with a minority geographic alignment score. At
the first bisection level, the ratio selection objective becomes:

```
EC_min(i:(k-i)) / sqrt(min(i, k-i)) + w_vra * alignment_penalty(i:(k-i))
```

The alignment_penalty measures how well minority VAP concentrates on one side
of the proposed split (Gingles Prong 1: geographic compactness of minority
population). Higher w_vra increases the influence of minority alignment; w_vra
= 0 reduces to standard GeoSection.

**Alignment score definition**: The alignment score quantifies geographic
concentration of minority population in the split region, computed as the
fraction of minority VAP in the more-concentrated half divided by the minority
VAP fraction statewide. A score above 1.0 means the minority population is
more concentrated in one half of the split than it is statewide.

The algorithm uses spatial minority-VAP distribution from demographics CSVs.
No partisan data enters the algorithm. This maintains the Callais
disentanglement requirement: partisan and racial considerations cannot be
combined in the same weight formula.

---

### StabilitySection (B.15)

**CLI**: analysis mode, not a partition mode

StabilitySection is a cross-census analysis tool, not a bisection algorithm.
Given maps for 2000, 2010, and 2020, it measures how many census tracts change
district assignment across decennial reapportionments. The factorisation-tree
reuse property of ApportionRegions predicts stable district assignments in
states where the seat count's largest prime factor does not change.

---

## Weights layer (B.10)

### County-Sticky (B.10)

**CLI**: `structure: standard-bisect` + `weights: county` + `alpha_county: 3.0`

County-Sticky is not a structure algorithm — it is a weights-layer algorithm
composed with standard bisection. Intra-county edges (both endpoints in the
same county) receive a 3.0x weight multiplier. METIS becomes reluctant to cut
within counties.

**Results**: 34% fewer county splits compared to geographic-weight baseline,
at a 3% mean Polsby-Popper cost. For states with strong county-based political
geography (Georgia, Texas, North Carolina), county integrity preservation is
a meaningful legal claim.

---

## Search algorithms (B.16, H.0-H.1)

### ConvergenceSweep (B.16)

**CLI**: `search: convergence` + `convergence_threshold: 600`

ConvergenceSweep walks seeds starting from a SHA-256 content-derived seed
s_0 = SHA-256(census_release_id || "DIA_SEED_V1") mod 2^31. It stops when T=600
consecutive seeds produce no improvement in normalised edge cut.

The content-derived seed is publicly verifiable: any party can compute s_0 from
the Census Bureau's release identifier. No expert discretion or manual seed
selection enters the algorithm.

**Statutory rationale**: T=600 satisfies the Districting Integrity Act (DIA)
statutory stopping criterion. The empirical B.7 finding that all 50 states
plateau before seed index 1,023 means T=600 is conservative: the algorithm
always terminates within the statutory window and always finds the true
compactness optimum.

### PercentileSweep (H.0)

**CLI**: `search: percentile` + `seeds: T` + `percentile: p`

PercentileSweep runs T seeds, sorts plans by normalised edge cut, and returns
the plan at rank floor(p * T). Rather than always returning the minimum-cut
plan, it allows callers to sample from the distribution of near-optimal plans.
This is useful for legal analysis where the court may want to see the range of
outcomes at different compactness percentiles.

### BisectionEnsemble (H.1)

**CLI**: `search: bisection-ensemble` + `seeds: T` + `percentile: p`

BisectionEnsemble replaces the METIS call at each binary tree node with a local
ReCom chain. For T steps, it samples spanning trees via Wilson's algorithm,
enumerates all balanced cuts, and accepts valid bisections. After T steps, it
selects the accepted bisection at percentile p of the edge-cut distribution.

For prime-factor nodes (p-way splits with p > 2), standard METIS is used;
only binary nodes use the ReCom primitive. A fully binary tree (k = 2^d)
applies BisectionEnsemble at every level.

BisectionEnsemble guarantees: contiguity (Wilson's spanning tree always
produces connected components), tractability (the population balance constraint
is always satisfiable for census tracts), and level-parallelism (all nodes at
the same tree depth are independent and can run concurrently).

---

## Further reading

- [three-layer-compositor.md](three-layer-compositor.md) — how the three layers compose
- [ensemble-methods.md](ensemble-methods.md) — GerryChain evaluation of section algorithm output
- [recursive-bisection.md](recursive-bisection.md) — the foundation algorithm
- [edge-weighted-bisection.md](edge-weighted-bisection.md) — the B.2 result
- Papers B.8-B.15 in `research/`
