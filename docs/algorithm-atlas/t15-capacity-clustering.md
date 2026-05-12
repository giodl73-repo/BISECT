# T.15 Capacity-Constrained Clustering

![T.15 capacity clustering visual](assets/t15-capacity-clustering.svg)

## What It Does

Capacity-constrained clustering chooses deterministic seeds, assigns nearby
units while respecting population capacity, and records whether the resulting
clusters are valid, need repair, or are infeasible under the declared capacity
profile.

## Algorithm Shape

```text
adjacency + populations
  -> farthest-point seeds
  -> nearest-seed assignment with capacity checks
  -> connectedness and population validation
  -> optional repair/status lineage
  -> RPLAN/RCTX/certificate package
```

## Inputs

- Unit adjacency graph
- Unit populations or weights
- Number of districts/clusters
- Population tolerance

## Outputs

- District assignment
- Cluster summary with capacity status, repair status, edge cut, and parameter
  hash
- RPLAN plan, RCTX context, audit certificate, and manifest in package runs

## When To Use It

Use capacity clustering when you want construction behavior that makes
population-capacity status explicit and audit-friendly.

## Claim Boundary

Capacity clustering establishes deterministic capacity-aware assignment and
status reporting. It does not certify compactness, community preservation,
partisan fairness, or legal sufficiency beyond the declared audit profile.

## References In This Repo

- Crate: `bisect-clustering`
- Paper: `docs/papers/T.15+capacity-constrained-clustering.pdf`
- Golden package: `docs/examples/rplan-golden-packages/T.15+capacity-constrained-clustering/`
- Benchmark package: `docs/examples/rplan-benchmark-packages/T.15+capacity-path100-benchmark/`
