# V.11 Plan

## Thesis

RCOUNT verification must be fast enough that public full-election packages can
be checked outside election offices. Parallel verification should preserve the
same transcript semantics as serial verification.

## Landed

- [x] Add first `rcount-core` parallel verifier slice.
- [x] Preserve serial report ordering for parallel selection-sum checks.
- [x] Add serial/parallel equivalence tests for pass and failure cases.

## Next

- [ ] Add package-size and timing benchmarks for larger summary fixtures.
- [ ] Parallelize source-hash checks and batch/accounting checks where ordering
  remains deterministic.
- [ ] Add a benchmark-tier RCOUNT package with public CVR or summary data.
