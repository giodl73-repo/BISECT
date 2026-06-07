# Y.3 Cycle-Supported Cut Weighting

Goal: use local graph cycles to identify cohesive mesh tissue and weak bridge
connectors before running BISECT partitioning.

## Intuition

An edge inside a dense local mesh often has an alternate short path around it.
An edge that connects two regions often has no short alternate path.

```text
short alternate path exists -> edge belongs to local mesh
no alternate path -> edge is bridge-like
```

BISECT can make mesh edges more expensive to cut and bridge-like edges easier
to cut, while preserving population balance and contiguity requirements.

## First Algorithm

For each edge `(u, v)`:

```text
remove edge (u, v)
path = bounded_bfs(u, v, max_depth = L)
restore edge (u, v)

if path exists:
    cycle_len = path_len + 1
    cycle_support(u, v) = 1 / cycle_len
else:
    cycle_support(u, v) = 0
```

This detects whether cutting the edge destroys the only local connection or
whether nearby adjacency can route around it.

## Weighted Variant

A more geographic version uses edge costs:

```text
cost(e) = 1 / shared_boundary_length(e)
```

or a mixed cost:

```text
cost(e) = geographic_distance(e) / normalized_boundary_length(e)
```

Then shortest alternate paths favor tight local geography rather than long
wandering loops.

## Cohesion Weight

Cycle support can feed the edge weight:

```text
edge_weight(e)
  = boundary_strength(e)
    * mass_factor(e)
    * (1 + alpha * normalized_cycle_support(e))
    * (1 - gamma * bridge_likeness(e))
```

Where:

```text
bridge_likeness(e) = 1 - normalized_cycle_support(e)
```

Exact bridges should remain visible as evidence rows even if later scoring
adds softer bridge metrics such as edge betweenness.

## Population Interaction

Cycle support is topology. Population mass thickens or thins that topology.

```text
dense cycle-rich mesh -> strongly protected local tissue
sparse bridge-like connector -> likely cut candidate
dense region with many ideal districts -> factor internally along weakest
local cuts
```

This prevents the model from treating an empty mesh and a dense urban mesh as
equally important.

## Metrics

- edge cycle support distribution;
- bridge-like edge count;
- share of selected cut edges with low cycle support;
- retained internal cycle support inside districts;
- compactness and split count changes;
- population deviation;
- mode-specific invariance pass/fail.

## Acceptance Criteria

- Synthetic mesh fixtures retain high-cycle-support edges.
- Synthetic bridge fixtures prefer bridge-like cuts when population feasible.
- Edge scores are inspectable per edge.
- The method does not use party or demographic fields in geographic mode.
