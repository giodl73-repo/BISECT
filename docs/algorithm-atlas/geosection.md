# GeoSection

![GeoSection visual](assets/geosection.svg)

## Mental Model

GeoSection solves the caterpillar problem in recursive bisection. A tiny
1:(k-1) split can look artificially cheap because enclosing a small region has a
short boundary. GeoSection scans first-level split ratios and normalizes edge
cut by `sqrt(min(i, k-i))` so small-ratio splits do not win only because they
are small.

## How BISECT Uses It

BISECT uses GeoSection when the top-level split ratio should be chosen from
geometry rather than assumed:

```text
try ratios -> normalize cut cost -> choose first-level split -> recurse
```

After the first level, subsequent regions use ordinary recursive bisection.

## Step-By-Step Mechanics

1. For target `k`, enumerate ratios from `1:(k-1)` through the balanced split.
2. Run the configured METIS/search budget for each candidate ratio.
3. Record the best edge cut for each ratio.
4. Score each ratio as `EC_min / sqrt(min(i, k-i))`.
5. Select the best normalized ratio.
6. Recurse inside the chosen regions.

## Claim Boundary

GeoSection explains a ratio-selection rule. It does not claim that one
statewide partisan result is legally required; empirical outcome claims need
their own seed and data provenance.

## References In This Repo

- Structure value: `ratio-optimal`
- Legacy mode: `geosection`
- Concept guide: `docs/concepts/section-algorithms.md`
