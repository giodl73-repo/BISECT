# T.16 Hierarchical Regionalization

![T.16 hierarchical regionalization visual](assets/t16-hierarchical-regionalization.svg)

## What It Does

Hierarchical regionalization starts with each unit as its own region and
repeatedly merges adjacent regions using a deterministic balanced-agglomerative
policy until the target district count is reached. The merge log is a first
class artifact.

## Algorithm Shape

```text
unit regions
  -> choose eligible adjacent pair
  -> merge regions
  -> record merge witness
  -> repeat until k regions remain
  -> RPLAN/RCTX/certificate package
```

## Inputs

- Unit adjacency graph
- Unit populations or weights
- Number of target regions/districts
- Population tolerance

## Outputs

- District assignment
- Regionalization summary with merge count, hierarchy depth, edge cut, and
  population deviation
- Merge log
- RPLAN plan, RCTX context, audit certificate, and manifest in package runs

## When To Use It

Use regionalization when you want a construction trace that explains how small
connected units became larger connected regions through recorded merges.

## Claim Boundary

Regionalization explains a deterministic merge history and verifier lineage. It
does not prove that the merge tree is globally optimal, legally sufficient, or
better than clustering, flow construction, or METIS on real data.

## References In This Repo

- Crate/module: `bisect-clustering::regionalization`
- Paper: `docs/papers/T.16+hierarchical-regionalization.pdf`
- Golden package: `docs/examples/rplan-golden-packages/T.16+hierarchical-regionalization/`
- Benchmark package: `docs/examples/rplan-benchmark-packages/T.16+regionalization-path100-benchmark/`
