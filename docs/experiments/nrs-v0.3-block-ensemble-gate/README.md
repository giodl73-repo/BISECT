# NRS v0.3 Block-Level Ensemble Gate

Status: governed Rhode Island Stage 1 completed, converged under the frozen
scalar rules, and regenerated exactly.

The Rhode Island 2020 block RCTX and governed NRS v0.3 starting assignment
join exactly across 25,649 blocks. The adapter verified symmetric adjacency,
finite nonnegative weights, two nonempty contiguous districts, and maximum
population deviation `0.0011873746444938348` against the frozen `0.005` bound.

The excluded engineering preflight ran 25 steps once for each frozen kernel.
Both accepted 25/25 steps, stayed inside the population bound, and reproduced
their normalized traces exactly. Wilson averaged 177.758 ms/step and Kruskal
61.259 ms/step on the author machine. These runtime and feasibility diagnostics
are not ensemble findings.

The governed run completed all four 2,000-step chains for each kernel. After
the frozen 500-step burn-in, both registered scalar metrics passed the
split-R-hat below 1.05 and pooled-ESS at least 100 rules. Wilson's mean cut
fraction was `0.0103331`; Kruskal's was `0.00778270`, with a descriptive
two-sample KS statistic of `0.405833`. Weighted boundary cut also differed
materially (means `193,294,034` and `152,599,928`; KS `0.31`). This is evidence
of Rhode Island kernel sensitivity, not sampler equivalence.

The governed Wilson and Kruskal traces contain 8,000 metrics and 800 canonical
snapshots each. Acceptance rates were `0.94575` and `0.953`; both observed
maximum population deviations were `0.004998273216187954`. Passing the
registered diagnostics does not prove mixing or independence. No preflight
sample entered the analysis.

Fresh sequential replay regenerated both kernels' normalized metrics and
canonical snapshots exactly. The package verifier also re-hashed every bound
artifact and source, replayed Stage 0, and recomputed the governed analysis and
flat summary. The expansion gate remains closed because this package does not
contain an instrumented peak-memory measurement or an explicit multi-State
compute and storage budget. See the frozen protocol at
`docs/specs/2026-08-10-nrs-v0.3-block-ensemble-gate.md`.

## Reproduce The Input Audit

```powershell
cargo test -p bisect-ensemble block_input
cargo run -p bisect-ensemble --example validate_block_input -- --rctx data/2020/certified/ri_blocks_2020.rctx --assignments runs/nrs-v0.3/neutral-analysis/national-2020/states/ri/package/baseline_assignments.json --state RI --year 2020 --districts 2 --tolerance 0.005
python scripts/research/verify_block_ensemble_gate.py docs/experiments/nrs-v0.3-block-ensemble-gate
```

The exact excluded preflight commands and hashes are retained in
`preflight-summary.json`; raw traces are `preflight-wilson.json` and
`preflight-kruskal.json`.

## Verify Governed Stage 1

The `--stage1` verifier first re-hashes the package and recomputes the analysis,
then launches a fresh full Wilson run followed by a fresh full Kruskal run and
compares normalized metrics and snapshots exactly:

```powershell
python scripts/research/verify_block_ensemble_gate.py docs/experiments/nrs-v0.3-block-ensemble-gate --stage1
```

Runtime diagnostics are excluded from normalized comparison. Commands and
known deviations are retained in `commands.md` and `deviations.md`.
