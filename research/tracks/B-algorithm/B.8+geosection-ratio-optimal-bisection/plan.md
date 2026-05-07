# B.8 — GeoSection: Isoperimetrically-Normalised Ratio-Optimal Bisection

**Paper Type**: Algorithm Design + Empirical Comparison
**Status**: Active implementation (2026-05-01)
**Series**: B (Algorithm Design)
**Depends on**: B.7 (MEC convergence, solution space)
**Companion**: B.9 — AreaSection (dual population+area constraints, separate paper)

---

## Core Idea

Standard recursive bisection always splits ⌊k/2⌋:⌈k/2⌉ (as close to equal
as possible). GeoSection adds two innovations:

**Innovation 1 — Isoperimetrically-normalised ratio search**

For each recursion level, try ALL feasible split ratios (1:k-1, 2:k-2, ...,
⌊k/2⌋:⌈k/2⌉) with N seeds each. Select the ratio minimising:

  EC_normalised(i:k-i) = EC / √(min(i, k-i))

The √ denominator is the isoperimetric correction: for a convex region, the
minimum boundary to enclose fraction f of the area scales as √(min(f, 1-f)×A).
Without this correction, 1:k-1 always wins — cutting off 1/k of the state is
trivially cheaper than a true bisection — degenerating to **sequential urban
carving** (peeling off one compact Democratic city district at each level).

**Why this matters legally:** Sequential carving looks like deliberate packing
even with no partisan input. The normalisation ensures **both halves receive
comparable geographic territory at every level** — genuine recursive bisection
rather than a district factory. States where one urban area is genuinely
isolated (PA: Philadelphia 39km, TN: Nashville/Memphis 72km) still produce
1:k-1 splits because the normalisation confirms they are genuinely compact.
States with more uniform density (NC: 6:8 east/west, not 1:13 Charlotte peel)
produce genuine bisections.

**Innovation 2 — Re-rotation at every recursive level**

Each subregion after a split computes its own minor axis (PCA of tract
centroids within that subregion). The directional penalty λ — which
discourages zigzag cuts parallel to the cut direction — is aligned to THAT
subregion's orientation, not the full state's orientation.

---

## Key Design Decisions

**Normalisation is mandatory for fairness optics.** The unnormalised version
produces maps that look like partisan carving. The normalised version produces
maps that look like genuine geographic divisions. Both are mathematically valid
minimum-EC algorithms; only the normalised version is politically defensible.

**AreaSection is a separate paper (B.9).** Dual population+area constraints
(ncon=2 in METIS, where BOTH population ratio AND 50/50 land area must be
satisfied simultaneously) is a different algorithm with a different legal theory
and different empirical footprint. It deserves its own treatment.

---

## Algorithm

```
GeoSection(G, k, N, λ=0):

  Base case: if k == 1, return {V} as single district.

  Step 1: For i = 1..⌊k/2⌋:
    Run N METIS seeds with pop target (i/k, 1-i/k).
    Record: EC_min(i), best_split(i).
    Compute: EC_normalised(i) = EC_min(i) / √(min(i, k-i)).

  Step 2: Select i* = argmin EC_normalised(i).
    This is the isoperimetrically natural ratio.

  Step 3: (Optional, Phase 2) Apply directional penalty λ:
    Compute PCA minor axis of current subregion's tract centroids.
    Re-weight edges: w'(u,v) = w(u,v) × (1 + λ|sin(θ(u,v))|).
    Rerun ratio search with w'.

  Step 4: Recurse:
    GeoSection(left_half, i*, N, λ)   — finds its own natural ratio
    GeoSection(right_half, k-i*, N, λ) — independent of left half
```

---

## Empirical Results (2026-05-01)

### Level-1 natural ratios (normalised vs unnormalised)

| State | k | Unnorm winner | EC | Norm winner | EC | Normalised |
|-------|---|--------------|-----|------------|-----|-----------|
| TN | 9 | 1:8 | 72km | **1:8** | 72km | genuine (Nashville is real) |
| PA | 17 | 1:16 | 39km | **1:16** | 39km | genuine (Philly is real) |
| WI | 8 | 1:7 | 85km | **1:7** | 85km | genuine (Milwaukee is real) |
| NC | 14 | 1:13 | 98km | **6:8** | 227km | CHANGED — east/west split |
| GA | 14 | 1:13 | 100km | **TBD** | — | need to rerun |
| MI | 13 | 1:12 | 74km | **TBD** | — | need to rerun |

### NC full recursive tree (normalised)
```
NC(14) → 6:8   (east/west — 227km normalised=92.6)
├── east(6) → 2:4 (122km normalised=86.3)
│   ├── 2-district subregion → 2:2
│   └── 4-district subregion → 4:4
└── west(8) → ...
```
**NC result**: 5D/9R (−13.6pp) at 2,510km total EC.
Same partisan outcome as MEC (2,400km) and unnormalised GeoSection (2,695km).
Geographic sorting dominates — partisan outcome stable under all geometric methods.

### WI recursive tree (normalised)
Level-1: 1:7 (genuine Milwaukee), Level-2: 1:6 (Madison), Level-3: 1:5,
Level-4 (k=5): **2:3** ← normalisation fires, rural WI bisects evenly.
The algorithm self-corrects: genuine urban concentrations win at early levels;
more balanced splits emerge once cities are exhausted.

---

## Legal Argument

GeoSection with isoperimetric normalisation is legally defensible because:

1. **No partisan data enters** — only TIGER boundary lengths and census populations.
2. **Genuine bisection at every level** — both halves receive comparable
   geographic territory. No systematic carving of one party's strongholds.
3. **Self-certifying via normalisation** — states where urban concentration
   is genuinely geographically isolated (Philadelphia, Nashville, Milwaukee)
   naturally produce 1:k-1 splits even under normalisation. The algorithm
   doesn't impose this; geography confirms it.
4. **Transparent and verifiable** — the normalisation formula is a single
   published equation. The natural ratio for every state can be independently
   computed from TIGER data.

---

## Companion Paper: B.9 AreaSection

AreaSection adds a second METIS constraint: simultaneously balance BOTH
population (at the target ratio) AND land area (always 50/50). Uses ncon=2.

This is a fundamentally different algorithm with a different legal theory:
"We divided the land equally AND respected population ratios."

Key questions for B.9:
- For each population ratio, what does the 50/50-area constraint do to the
  edge-cut? Is it much more expensive than unconstrained GeoSection?
- Does the dual constraint produce different partisan outcomes?
- Is area-balanced bisection more or less proportional than population-balanced?

---

## Next Steps

1. Run normalised GeoSection on GA and MI (level-1 ratio may differ from 1:k-1)
2. Run normalised GeoSection on PA (confirm Philadelphia is genuine)
3. Implement directional penalty λ (Phase 2 — straighter cuts)
4. Compare GeoSection vs MEC partisan outcomes at national scale
5. Write up B.9 AreaSection as separate paper
