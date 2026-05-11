# Role Review: T.16 Hierarchical Regionalization

**Spec:** [`2026-05-11-t16-hierarchical-regionalization.md`](../2026-05-11-t16-hierarchical-regionalization.md)  
**Status:** Approved for stage-one implementation

## Summary

T.16 is approved as a deterministic agglomerative baseline. The key audit value
is not optimality; it is a replayable hierarchy of adjacent merges with
population-capacity and contiguity witnesses.

## Required Guardrails

- Merge only adjacent regions in the stage-one hierarchy.
- Preserve deterministic tie-breaking by population score and region id.
- Emit structured infeasible-capacity status instead of panicking.
- Record merge count, hierarchy depth, repair method, population deviation, and
  edge cut in summary lineage.
- Delegate final production-plan validity to U.20 RPLAN sidecars once CLI wiring
  lands.

## Stage-One Approval

Approved to add `bisect-clustering::regionalization` with L0 hierarchy,
capacity, contiguity, and determinism coverage before runner wiring.
