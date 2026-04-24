# Recursive Bisection

## Short version

Split the state in half by population. Split each half again. Keep going until you have the right number of districts. Each split is a graph partition problem solved by METIS. The shape of every district is determined entirely by geography — no political data enters at any stage.

---

## Why bisection

When you split something in half, neither side gets to choose which half is theirs. The cut is determined by the constraint (equal population) and the geometry, not by who benefits. This is the core fairness property: no one controls the outcome.

Repeating recursively means every district is the product of a sequence of neutral halvings. There is no single moment where a mapmaker decides which community ends up together. The algorithm makes that decision the same way every time, everywhere.

## The binary tree structure

A state with N districts requires ⌈log₂ N⌉ rounds of bisection. Each round doubles the number of regions:

```
Round 0:  1 region  (the whole state)
Round 1:  2 regions
Round 2:  4 regions
Round 3:  8 regions  ← Minnesota (8 districts) stops here
```

For odd-district states, some splits are uneven:

```
Alabama (7 districts):
Round 1:  1 → [4 | 3]
Round 2:  4 → [2|2],  3 → [2|1]
Round 3:  four 2s each → [1|1],  one 1 stays
```

The `rounds_hierarchy.csv` output records this tree for every state.

## How METIS does the splitting

Each state is a graph:
- **Nodes**: census tracts (~1,000–8,000 per state)
- **Node weights**: tract population (ensures balanced splits)
- **Edges**: two tracts are connected if they share a border
- **Edge weights**: length of the shared border in meters (edge-weighted mode)

METIS (`gpmetis`) partitions this graph into two halves with equal total population, minimizing the total weight of cut edges. Because edge weight = shared boundary length, minimizing cut weight = minimizing district perimeter = maximizing compactness.

## Population balance

The Huntington-Hill method determines how many districts each state gets. Within a state, each district must be within ±0.5% of the ideal population (total state population ÷ number of districts).

METIS enforces this via its `ufactor` parameter (imbalance tolerance). We set `ufactor=5` which allows ±0.5% — tight enough for legal compliance, loose enough for the partitioner to find good cuts.

## Determinism

The same input always produces the same output. METIS uses a fixed random seed. This matters for:
- **Reproducibility**: other researchers can verify results
- **Cross-census comparison**: differences between 2010 and 2020 results are due to geography changes, not randomness
- **Debugging**: a run that fails can be re-run identically

## What the algorithm does NOT do

- It does not look at political party registration or voting history
- It does not look at race or ethnicity (except in the VRA variant — see [vra-compliance.md](vra-compliance.md))
- It does not try to make districts competitive, safe, or fair by any political metric
- It does not consider incumbents, county lines, or communities of interest

These are features, not bugs. The algorithm's neutrality is the point.

## Further reading

- [RECURSIVE_BISECTION.md](../RECURSIVE_BISECTION.md) — detailed algorithm walkthrough with pseudocode
- [edge-weighted-bisection.md](edge-weighted-bisection.md) — how edge weights improve compactness
- Paper B.1: *From Apportionment to Boundary Design*
