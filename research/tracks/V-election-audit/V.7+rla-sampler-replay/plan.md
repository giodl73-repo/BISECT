# V.7 Plan

## Landed

- [x] Add RCOUNT schema records for replayable RLA sampler rounds.
- [x] Recompute contest audit population from CVR rows.
- [x] Replay deterministic SHA-256 sample draws from public seed material.
- [x] Add positive and negative golden fixtures.
- [x] Expose sampler replay through `rcount verify`.

## Next

- [ ] Add observation rows for sampled ballot interpretations.
- [ ] Add discrepancy taxonomy for ballot-level comparison audits.
- [ ] Add stopping-rule verifier and escalation transcript.
- [ ] Add state-specific sampler/stopping adapters once legal targets are selected.
