# Edge-Weighted Bisection

## Short version

Weigh each edge in the tract graph by the length of the shared boundary. METIS minimizes the total weight of cut edges. Since weight = boundary length, minimizing cut weight = minimizing total district perimeter. Shorter perimeter at fixed area = higher Polsby-Popper score. Compactness improves automatically, without ever optimizing for it directly.

---

## The insight

Graph partitioning without edge weights minimizes the *number* of cut edges. With a uniform graph, that tends to produce districts shaped like tic-tac-toe boards — lots of small shared borders cut, producing jagged perimeters.

Weighting edges by shared boundary length changes the objective: METIS now prefers to keep long shared boundaries intact and cut short ones. Long shared boundaries only exist between tracts that genuinely border each other in a geographic sense. Cutting them costs more, so the algorithm avoids it.

The result: districts follow natural geographic boundaries (rivers, ridgelines, county lines) rather than arbitrary tract grid lines.

## How boundary length is computed

For each pair of adjacent tracts, we compute the length of their shared border using the TIGER/Line shapefile geometries. The adjacency graph builder (`scripts/data/geography/build_tract_adjacency.py`) does this with:

```python
shared_boundary = tract_a.geometry.intersection(tract_b.geometry)
edge_weight = shared_boundary.length  # in meters
```

Short boundaries (< 10 meters) are filtered out as likely corner touches or rounding artifacts. This is the `--minimum-boundary-length` parameter.

## Results (2020 Census, all 50 states)

| Metric | Unweighted | Edge-Weighted | Improvement |
|--------|-----------|---------------|-------------|
| Mean Polsby-Popper | 0.236 | 0.367 | **+56%** |
| vs enacted 2020 maps | — | 0.306 | **+20%** |
| States beating enacted | — | 37 of 50 | |

Standout state improvements:
- Illinois: +174%
- Louisiana: +104%
- New Hampshire: +102%

## Water boundaries

Coastal states present a problem: tracts separated by water (a bay, river, or sound) may be adjacent on the graph (same county) but not adjacent on the ground. The algorithm would happily cut across water, producing non-contiguous-looking districts.

We handle this with county-based bridge edges: when two tracts are in the same county but their shared boundary is water, we add a synthetic edge with a high weight (discouraging the cut). This preserves the geographic intent without requiring explicit water detection.

## Comparison to unweighted

The unweighted mode (`--partition-mode unweighted`) is available for research comparison. It shows what the algorithm produces without the geometric optimization — useful for quantifying the compactness contribution of the edge weights specifically.

Paper B.3 (*Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization*) proves that the single-objective edge-weighted formulation outperforms multi-constraint approaches that try to optimize compactness and population simultaneously.

## Further reading

- [polsby-popper.md](polsby-popper.md) — what Polsby-Popper measures and why it matters
- [recursive-bisection.md](recursive-bisection.md) — the base algorithm
- Paper B.2: *Edge-Weighted Recursive Bisection for Compact Congressional Districts*
