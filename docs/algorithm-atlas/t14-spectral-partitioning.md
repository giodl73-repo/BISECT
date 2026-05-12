# T.14 Spectral Partitioning

![T.14 spectral partitioning visual](assets/t14-spectral-partitioning.svg)

## What It Does

Spectral partitioning turns the unit adjacency graph into a Laplacian, computes
a Fiedler-vector ordering, and sweeps possible cuts to find a balanced two-way
partition. In BISECT it is a deterministic construction baseline for creating
a plan from graph structure.

## Algorithm Shape

```text
adjacency graph
  -> graph Laplacian
  -> Fiedler vector
  -> ordered units
  -> balanced sweep cut
  -> RPLAN/RCTX/certificate package
```

## Inputs

- Unit adjacency graph
- Unit populations
- Target district count for the split stage
- Population tolerance

## Outputs

- District assignment
- Spectral summary with edge cut, population deviation, and parameter hash
- RPLAN plan, RCTX context, audit certificate, and manifest in package runs

## When To Use It

Use spectral partitioning when you want a transparent deterministic
construction baseline driven by graph geometry rather than random sampling.

## Claim Boundary

Spectral partitioning proves a deterministic construction path and verifier
lineage. It does not prove legal sufficiency, optimality, or superiority over
other construction methods.

## References In This Repo

- Crate/function: `bisect-apportion`
- Paper: `docs/papers/T.14+spectral-partitioning.pdf`
- Method package: `docs/examples/rplan-method-packages/T.14+spectral-generated-synthetic/`
- Benchmark package: `docs/examples/rplan-benchmark-packages/T.14+spectral-grid10-benchmark/`
