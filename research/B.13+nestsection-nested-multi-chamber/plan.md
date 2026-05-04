# B.13 — NestSection: Nested Multi-Chamber Redistricting via Shared Bisection Trees

**Paper Type**: Algorithm Design + Empirical Analysis  
**Status**: Planning (2026-05-02)  
**Series**: B (Algorithm Design)  
**Depends on**: B.8 (GeoSection — ratio-optimal bisection), B.11 (ApportionRegions — factorization tree infrastructure)  
**Companion**: B.8, B.9 (ratio selection mechanics), B.10 (subdivision hierarchy as a nesting anchor)

---

## Core Idea

Congressional, state senate, and state house districts for the same state are
currently drawn by three independent redistricting processes — separate
algorithms, separate boundary consultants, separate litigation. NestSection
forces them to share the same hierarchical bisection tree, so that:

- Every **state senate district** is a union of complete congressional
  sub-districts.
- Every **congressional district** is a union of complete state house
  sub-districts.

The ApportionRegions factorization tree for the congressional delegation (e.g.
NC: 14 = 2×7) defines a bisection spine. The state senate (50 seats) and state
house (120 seats) trees are constructed as *refinements* of each other at
different depths: the same geographic bisection points, applied iteratively.

**Why this matters legally:** A legislature that gerrymanders congressional
districts cannot then draw state senate districts that "undo" the geographic
sorting imposed by the congressional tree — the nesting constraint forces all
three chambers to reflect the same geographic structure. The shared hierarchy
is a *mutual constraint*: it limits gerrymandering at every level because each
chamber's boundaries are determined by the same sequence of geographic cuts.

---

## The Compatibility Problem

Three independently-apportioned seat counts rarely share compatible prime
factorizations. For NC:

```
Congressional:  14 = 2 × 7
State Senate:   50 = 2 × 5 × 5
State House:   120 = 2 × 3 × 4 × 5
```

These factorizations are not refinements of each other: there is no way to
recurse the 14-seat tree to get exactly 50 children then 120 grandchildren.

NestSection's central algorithmic contribution is defining and computing the
"closest compatible nesting structure" when factorizations are incompatible.

### Compatibility Modes

**Mode 1 — Strict nesting (C ⊂ S ⊂ H)**
Congressional ⊂ Senate ⊂ House: each congressional district is a union of
senate districts, each senate district is a union of house districts.
Requires: S divisible by C, and H divisible by S.

**Mode 2 — Partial nesting (senate anchored)**
The senate tree is the spine. Congressional districts are coarsened senate
groups; house districts are senate sub-splits. Useful when `lcm(C, H)` is
compatible with S.

**Mode 3 — Best-effort nesting with mismatch tolerance τ**
Allow up to τ tracts (by population) to be assigned to a "boundary zone"
that straddles a congressional/senate boundary. Districts on either side of
a boundary zone are marked as partially overlapping. τ is published and
verifiable.

---

## Algorithm Design

### Step 1: Build the Compatibility Matrix

For a state with C congressional seats, S senate seats, H house seats:

```
CompatCheck(C, S, H):
  For each divisor d of gcd(S, H):
    candidate_spine = d
    senate_per_spine = S / d
    house_per_senate = H / S
    check: can C decompose into d groups of roughly (C/d) each?
    score: sum of |C_i - C/d| for each of the d groups
  Return spine with minimum score.
```

For NC (14, 50, 120): `gcd(50, 120) = 10`. Test spine = 10:
- Senate per spine: 50/10 = 5 (each spine region gets 5 senate seats)
- House per senate: 120/50 = 2.4 — not integer. Mode 3 required.
- Mismatch tolerance τ computed from the fractional remainder.

### Step 2: GeoSection on the Spine

Run GeoSection (B.8) to bisect the state into `spine` regions, selecting the
isoperimetrically-optimal ratio at each level. This produces the shared
geographic skeleton.

### Step 3: Assign Congressional Seats to Spine Regions

With `spine = 10` and `C = 14`, assign approximately 14/10 = 1.4 congressional
seats per spine region. Use ApportionRegions (B.11) to find the integer
allocation that minimizes population imbalance:

```
ApportionToSpine(populations[], C):
  target = total_pop / C
  Assign c_i ∈ {1, 2} to each spine region i
  s.t. Σ c_i = C and |pop_i / c_i - target| ≤ 0.5% for all i
```

### Step 4: Recurse Within Spine Regions

For each spine region i with population P_i and seat count c_i:
- Run GeoSection(G_i, c_i) for congressional districts
- Run GeoSection(G_i, s_i) for senate districts (s_i = S/spine)
- Guarantee: all house sub-splits are refinements of the senate cuts

### Key Invariant

```
NestSection invariant: for every pair of districts (d_C, d_S) where d_C is
congressional and d_S is senatorial, either:
  d_C ∩ d_S = ∅ (disjoint), or
  d_C ⊇ d_S (congressional contains senate), or
  |d_C ∩ d_S| ≤ τ_pop (boundary tolerance — Mode 3 only)
```

---

## Key Equations

**Natural spine count** (compatible factorization heuristic):

```
spine*(C, S, H) = argmin_{d | gcd(S,H)} Σ_i |c_i(d) - C/d|
```

where `c_i(d)` is the optimal ApportionRegions allocation for spine region i.

**Nesting violation score** for a proposed assignment:

```
V(assignment) = Σ_{(d_C, d_S): overlap} pop(d_C ∩ d_S) / total_pop
```

A strictly nested plan has V = 0. Mode 3 allows V ≤ τ.

---

## Pseudocode Sketch

```
NestSection(G, C, S, H, τ=0.01):

  spine = spine*(C, S, H)           # compatibility matrix
  senate_per = S / spine
  house_per_senate = H / S          # may be non-integer → Mode 3

  # Step 1: Geographic spine (shared across all chambers)
  spine_regions = GeoSection(G, spine, N=50)

  # Step 2: Apportion congressional seats to spine regions
  c_allocation = ApportionRegions(spine_regions.populations, C)

  # Step 3: Build each chamber within each spine region
  for region i in 1..spine:
    congressional[i] = GeoSection(G_i, c_allocation[i])
    senate[i]        = GeoSection(G_i, senate_per)
    house[i]         = NestRefine(senate[i], house_per_senate, τ)

  return (congressional, senate, house)

NestRefine(senate_districts, h_per_s, τ):
  # Split each senate district into h_per_s house sub-districts.
  # If h_per_s is not integer, allow ±1 and record boundary zones.
  for d in senate_districts:
    h_count = round(h_per_s × pop(d) / (total_pop / H))
    house_sub[d] = GeoSection(G_d, h_count)
  validate_nesting_violation(house_sub, τ)
  return house_sub
```

---

## Empirical Plan

### States of Interest

| State | C | S | H | gcd(S,H) | Compatible? |
|-------|---|---|---|----------|-------------|
| NC    | 14 | 50 | 120 | 10 | Partial (Mode 3, τ~0.04) |
| TX    | 38 | 31 | 150 | 1  | Minimal spine (Mode 3, τ large) |
| CA    | 52 | 40 | 80  | 40 | Strong (S=40 exact spine; H=80=2×40) |
| WI    |  8 | 33 | 99  | 33 | H = 3×S, C not aligned |
| FL    | 28 | 40 | 120 | 40 | H = 3×S, C partial |
| PA    | 17 | 50 | 203 | 1  | Minimal — Mode 3 |
| VA    | 11 | 40 | 100 | 20 | Moderate |
| OH    | 15 | 33 | 99  | 33 | H = 3×S, C partial |

### Experiments

1. **Compatibility census**: For all 50 states, compute `spine*(C, S, H)` and
   the mismatch tolerance τ required for Mode 3. Produce a national map: which
   states are "naturally nestable" (τ < 1%) vs. "structurally incompatible"?

2. **NC full run**: Construct the nested congressional/senate/house tree for NC
   (14/50/120). Compare against three independent GeoSection runs. Measure:
   - Nesting violation V
   - Edge-cut cost increase (nesting premium)
   - Partisan outcome stability

3. **CA clean nest**: CA (52/40/80) has gcd(40,80) = 40 and 80 = 2×40, giving
   a potentially exact nesting. Verify whether the geographic bisection tree
   actually produces V = 0 or whether population irregularities force τ > 0.

4. **Gerrymander resistance test**: Generate 100 GeoSection maps for NC
   (standard, unnested) and 100 NestSection maps. Measure variance in
   congressional/senate seat counts. Hypothesis: NestSection reduces variance
   because the spine constraint limits the set of feasible solutions.

### Metrics

| Metric | Definition |
|--------|-----------|
| `spine_count` | Number of first-level geographic regions |
| `nesting_V` | Population fraction in boundary zones (ideal: 0) |
| `ec_premium_pct` | Extra edge-cut cost vs. independent GeoSection |
| `c_seats_d`, `s_seats_d`, `h_seats_d` | Democratic seats each chamber |
| `cross_chamber_variance` | D/R seat count std dev across 100 seeds |

---

## Expected Findings

- **CA, FL, OH**: Near-perfect nesting achievable (τ < 0.5%) because
  factorizations are compatible. Small EC premium (~5%).
- **NC, WI, PA**: Moderate τ (1-5%) required. EC premium ~10-15%. The nesting
  constraint visibly reshapes some district boundaries.
- **TX**: Severe incompatibility (gcd=1). Nesting requires τ > 10% or
  congressional/house alignment is effectively abandoned.
- **Gerrymander resistance**: NestSection reduces across-seed partisan variance
  by ~30-50% in moderate-compatibility states. The spine anchors the solution
  space.
- **Legal finding**: A statute requiring NestSection for states with gcd(S,H) ≥ 5
  would cover ~25 states and effectively constrain partisan line-drawing at all
  three legislative levels.

---

## Legal / Policy Argument

**The shared-hierarchy constraint is anti-gerrymandering by construction.** A
partisan mapmaker who controls the congressional process cannot then draw senate
districts that "undo" the geographic sorting — the senate tree is locked to the
congressional spine. Conversely, a partisan senate map cannot strategically
fragment congressional districts because the congressional splits are
geometrically prior.

**Statutory hook**: Many state constitutions require that districts at one level
be composed of complete districts at another. NestSection operationalizes this
as a computational constraint rather than a post-hoc check.

**Comparison to independent chambers**: Three independent optimal maps can
produce "coherent gerrymandering" — where the same geographic area is packed
into a safe district at every level. NestSection makes this structurally
impossible: if a region is split at the congressional level, that same split
propagates to senate and house levels.

---

## Dependencies

- **B.8 (GeoSection)**: Isoperimetric ratio-optimal bisection is the core
  geographic primitive used at every recursion level.
- **B.11 (ApportionRegions)**: Integer seat allocation to spine regions.
  NestSection requires that ApportionRegions' output be the input to each
  sub-region's GeoSection call.
- **B.9 (AreaSection)**: Optional — AreaSection can replace GeoSection at the
  spine level to force land-area balance across chambers.
- **B.10 (SubdivisionSection)**: County boundaries can serve as spine anchors,
  replacing the computed spine with a governmental hierarchy spine.

---

## Target Venue

*Harvard Journal on Legislation* (statutory design focus) or  
*Election Law Journal* (empirical redistricting with legal argument)  
Backup: *Computers, Environment and Urban Systems* (algorithmic geography)

---

## Open Questions

1. Is Mode 3 (boundary tolerance τ) legally defensible? Does a small τ satisfy
   "composed of complete districts" language in state constitutions?
2. In TX (gcd=1), is NestSection meaningless, or is there a coarser notion of
   "soft nesting" (maximize district overlap rather than require containment)?
3. Does the EC premium scale with τ inversely? (More tolerance → less cost?)
4. For states with non-partisan redistricting commissions, does NestSection
   reduce workload (one spine, three allocations) or increase it (complex
   compatibility analysis)?
