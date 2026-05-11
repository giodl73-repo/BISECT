# U.19 Evolutionary Comparison Review Notes

**Spec:** [`2026-05-11-u19-evolutionary-comparison.md`](../2026-05-11-u19-evolutionary-comparison.md)  
**Decision:** Approved to extend `bisect-pareto` rather than create a new crate.

## Notes

- Keep U.19 focused on comparison and selected-output auditability.
- Reuse existing NSGA-II/crossover/mutation tests as the validity foundation.
- Do not force every frontier entry to carry a full certificate in the NDJSON
  stream; package selected/exported plans explicitly.
- Algorithm lineage must identify the selected frontier index and reproducible
  run parameters.
