# Algorithm Atlas

The Algorithm Atlas is the visual field guide for BISECT's algorithm families.
It complements `docs/concepts/` and the research papers with small diagrams,
plain-language summaries, and links to the crates, CLI surfaces, papers, and
RPLAN packages that make each method concrete.

## Overview

![BISECT algorithm atlas overview](assets/overview.svg)

Every family can use different internal machinery, but publication-grade plan
outputs converge on the same fixed point:

```text
algorithm output -> RPLAN -> RCTX -> audit certificate -> manifest -> verifier
```

## Construction Family

| Algorithm | Visual Guide | What To Look For |
|---|---|---|
| T.14 Spectral Partitioning | [T.14 Spectral Partitioning](t14-spectral-partitioning.md) | Fiedler ordering, sweep cuts, deterministic construction |
| T.15 Capacity-Constrained Clustering | [T.15 Capacity Clustering](t15-capacity-clustering.md) | Seeds, capacity-aware assignment, repair/status lineage |
| T.16 Hierarchical Regionalization | [T.16 Hierarchical Regionalization](t16-hierarchical-regionalization.md) | Adjacent merges, merge log, hierarchy depth |
| T.17 Flow-Based Construction | [T.17 Flow Construction](t17-flow-construction.md) | Seeds, capacities, flow-style assignment, infeasibility witness |

## Coming Next

The next atlas pages should cover:

- U.16 branch-and-cut
- U.17 branch-and-price
- U.18 local search
- U.19 selected/evolutionary frontier
- U.20 RPLAN audit certificates
- ReCom, SMC, multiscale, and ensemble search families

## Relationship To Other Docs

- `docs/concepts/algorithm-family-layer-cake.md` is the crate and evidence
  taxonomy.
- `docs/concepts/t-u-portfolio-dependency-map.md` maps papers, packages, and
  verifier paths.
- `docs/PAPERS.md` is the publication index.
- `docs/examples/rplan-benchmark-packages/` contains committed benchmark-tier
  packages referenced by these pages.
