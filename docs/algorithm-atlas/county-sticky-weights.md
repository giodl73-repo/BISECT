# County-Sticky Weights

![County-Sticky weights visual](assets/county-sticky-weights.svg)

## Mental Model

County-Sticky is a weights-layer method, not a structure algorithm. It boosts
intra-county edges so METIS is more reluctant to cut inside a county. The tree
shape can still be standard bisection, GeoSection, ApportionRegions, or another
structure mode.

## How BISECT Uses It

BISECT uses County-Sticky when county integrity should influence cut costs:

```text
same structure + boosted intra-county edges -> fewer county splits
```

Because it changes the meaning of edge cut, it is best understood as a
trade-off knob: fewer county splits may cost some compactness.

## Step-By-Step Mechanics

1. Build the adjacency graph and county labels.
2. Identify edges whose endpoints are in the same county.
3. Multiply those edge weights by the configured county factor.
4. Run the selected structure/search layers with the modified weights.
5. Report county splits and compactness effects separately.

## Claim Boundary

County-Sticky discourages county splits; it does not ban them. Population
balance and contiguity can still force cuts within counties.

## References In This Repo

- Weights value: `county`
- Config knob: `alpha_county`
- Concept guide: `docs/concepts/section-algorithms.md`
