# Role Review: U.18 Local Search And Improvement

**Spec:** [`2026-05-11-u18-local-search.md`](../2026-05-11-u18-local-search.md)  
**Status:** Approved for stage-one implementation

## Summary

The U.18 staging is implementation-ready because it starts with a narrow,
auditable kernel: deterministic one-move improvement from an already valid
assignment. It does not claim that local search alone establishes legal
validity; final acceptance remains delegated to U.20 RPLAN audit sidecars.

## Required Guardrails

- Preserve RPLAN unit order at the CLI boundary.
- Reject invalid starting plans rather than repairing silently in the one-move
  kernel.
- Keep tabu/LNS as structured staged methods until the one-move path has
  sidecar and verification coverage.
- Do not duplicate reserved audit-certificate fields in lineage extras.
- Record parent plan hashes once CLI wiring reads an existing RPLAN input.

## Stage-One Approval

Approved to add `bisect-local-search` with L0 deterministic fixtures and L1
lineage coverage before CLI wiring.
