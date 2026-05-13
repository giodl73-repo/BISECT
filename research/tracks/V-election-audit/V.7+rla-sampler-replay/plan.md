# V.7 Plan

## Landed

- [x] Add RCOUNT schema records for replayable RLA sampler rounds.
- [x] Recompute contest audit population from CVR rows.
- [x] Replay deterministic SHA-256 sample draws from public seed material.
- [x] Add positive and negative golden fixtures.
- [x] Expose sampler replay through `rcount verify`.
- [x] Add observation rows for sampled ballot interpretations.
- [x] Add first stopping-rule verifier and escalation transcript behavior.
- [x] Add discrepancy taxonomy for ballot-level comparison audits.
- [x] Bind reported RLA margin metadata to public summaries.

## Next

- [ ] Add Kaplan-Markov or state-specific statistical stopping math.
- [ ] Add state-specific sampler/stopping adapters once legal targets are selected.
