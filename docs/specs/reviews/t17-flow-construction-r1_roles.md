# T.17 Flow Construction Review Notes

**Spec:** [`2026-05-11-t17-flow-construction.md`](../2026-05-11-t17-flow-construction.md)  
**Decision:** Approved for a conservative first slice.

## Notes

- Keep the first implementation deterministic and graph-native.
- Treat the flow path as constructive assignment, not an exact optimizer.
- Require structured infeasibility for capacity failures.
- Emit lineage from the versioned summary; do not duplicate reserved audit
  certificate fields.
- Stage distance/projection and external solver work behind the same summary
  contract.
