# Spec: U.17 Branch-And-Price / Column Generation

**Status:** Stage 4 active  
**Track:** U.17 - Exact Optimization  
**Depends on:** U.16 ILP audit reports and U.20 RPLAN audit integration

## Purpose

Add a solver-neutral column-generation family for exact redistricting. U.17 is
staged separately from U.16 branch-and-cut because its audit lifecycle has
distinct artifacts: pricing columns, a master set-partitioning problem, bounds,
and branch-and-price provenance.

## Stage-One Algorithm

The first slice implements `bisect-column`:

1. Generate connected, population-feasible district columns for small graphs.
2. Build a set-partitioning master problem over generated columns.
3. Emit formulation-only reports for large or explicitly dry-run cases.
4. Solve tiny fixture masters exactly by deterministic enumeration.
5. Record status, pricing rounds, generated column count, bounds, gap, and
   optional solution in a stable JSON report.

This is not yet a production branch-and-price solver. It is the audited contract
and tiny exact path that later pricing and branching implementations must
preserve.

## Crate Boundary

`bisect-column` owns:

- column and pricing data contracts
- master-problem data contracts
- formulation-only output reports
- small exact fixture solver
- algorithm lineage metadata

`bisect-cli` owns:

- `bisect exact --method branch-and-price`
- report emission in plan audit packages
- fallback behavior when no exact solution is available

## Tests

- pricing generates connected balanced columns on a path fixture
- formulation-only report has no bound/gap claims
- small exact master solves a path fixture
- report builds `bisect-column` algorithm lineage
