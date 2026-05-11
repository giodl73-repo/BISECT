# Spec: T.16 Hierarchical Regionalization

**Status:** Stage 3 active  
**Track:** T.16 - Plan Construction  
**Depends on:** U.20 RPLAN audit integration fixed point

## Purpose

Extend `bisect-clustering` with a deterministic regionalization constructor.
Unlike flat capacity clustering, this slice records a hierarchy of adjacent
region merges so audits can inspect how contiguous candidate regions were
formed.

## Stage-One Algorithm

The first slice implements `bisect_clustering::regionalization`:

1. Start with each unit as a singleton region.
2. Enumerate adjacent region pairs from graph edges.
3. Merge the adjacent pair whose combined population is closest to ideal
   district population, preferring pairs that do not exceed the capacity upper
   bound and then breaking ties by region id.
4. Continue until exactly `k` regions remain.
5. Convert final regions to district assignments in stable region-id order.
6. Emit merge witnesses, population deviation, edge cut, repair status, and
   summary lineage metadata.

The stage-one repair path reuses the existing exhaustive small repair witness
for tiny fixtures. Larger invalid outputs remain structured `needs-repair`
results until a scalable repair module is introduced.

## Crate Boundary

`bisect-clustering` owns:

- deterministic agglomerative regionalization
- merge witnesses and summary hashing
- L0 hierarchy, capacity, contiguity, and determinism fixtures

`bisect-cli` owns:

- `--structure regionalization`
- YAML `structure: regionalization`
- manifest fields and RPLAN audit sidecars

## Tests

- path fixture produces adjacent pair hierarchy
- impossible-capacity fixture emits structured infeasible status
- fixed grid input produces identical assignment, merge log, and summary
- two-clique fixture remains contiguous and capacity-valid
- CLI/RPLAN sidecar test after runner wiring
