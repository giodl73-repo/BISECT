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

## Picture 1: Weight Multiplication

![County-Sticky numeric weights](assets/county-sticky-numeric.svg)

The method does not add a hard county constraint. It changes the graph weights.
If two units are in the same county, their shared edge receives a multiplier.
That makes cutting that edge more expensive for METIS, while still allowing the
cut when population balance or contiguity pressure requires it.

## Picture 2: Trade-Off Surface

![County-Sticky trade-off](assets/county-sticky-tradeoff.svg)

County preservation is a trade-off, not a magic switch. Increasing the county
multiplier can reduce county splits, but it can also increase perimeter or edge
cut. A mature report should show both sides of the trade.

## Step-By-Step Mechanics

1. Build the adjacency graph and county labels.
2. Identify edges whose endpoints are in the same county.
3. Multiply those edge weights by the configured county factor.
4. Run the selected structure/search layers with the modified weights.
5. Report county splits and compactness effects separately.

## What The Output Needs To Explain

The output should report the county multiplier, how county labels were joined,
which metric counts county splits, and the compactness effect relative to the
same structure/search run without the county multiplier.

## Claim Boundary

County-Sticky discourages county splits; it does not ban them. Population
balance and contiguity can still force cuts within counties.
It also does not automatically satisfy state subdivision-preservation law,
which varies by jurisdiction.

## Failure Modes

- County labels are missing or joined with malformed GEOIDs.
- The page claims counties are preserved as a hard legal constraint.
- Compactness cost is omitted when split reduction is reported.

## References In This Repo

- Weights value: `county`
- Config knob: `alpha_county`
- Concept guide: `docs/concepts/section-algorithms.md`
- State-law caution: `.roles/ward.md`
