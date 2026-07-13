# Scalable Certified Split Solver - Close Record

**Opened:** 2026-07-10  
**Closed:** 2026-07-10  
**Disposition:** `partial_state_certificate_complete_wave`  
**Posture:** `internal_engineering_baseline_only`

## Achieved

1. Pinned RoundingSat and VeriPB toolchain.
2. Externally generated and independently verified proof smoke artifacts.
3. Polynomial parent/depth connectivity encoding.
4. Compact lexicographic prefix encoding.
5. Deterministic METIS discovery with engine-aware provenance.
6. Articulation-safe population improvement.
7. Rhode Island population-floor incumbent at 548,689 / 548,690.
8. Three State-scale compact OPB models.
9. Verified Rhode Island population optimality proof.
10. Precise boundary/canonical residual blocker.

## Partial Certificate

Proved:

> No connected canonical Rhode Island split improves scaled population
> deviation below 1.

Not proved:

- boundary cut optimality;
- canonical tie-breaking; or
- the complete Rhode Island root assignment.

## Boundary Frontier

Two 300-second searches reached `TIMELIMIT`, with and without proof logging.
The proof-logging run produced a 7.1 GB incomplete proof, which was deleted.

## Carry-Forward

The next wave is **Rhode Island Boundary Certification**:

1. improve the cut while preserving population deviation 1;
2. optimize zero-population boundary placement;
3. add same-population swaps and larger neighborhoods;
4. strengthen the boundary decision model;
5. rerun RoundingSat/VeriPB; and
6. run canonical certification only after boundary optimality closes.

No git commit or public release was created by wave closure. The working tree
remains ready for maintainer review and an explicitly requested commit.
