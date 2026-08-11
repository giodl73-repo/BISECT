# NRS v0.3 NH/NM/GA Block-Ensemble Expansion

Status: terminal failure; expansion closed without retry under the frozen
resource contract. Stage 0 and five governed primaries passed. The sixth
primary, GA Kruskal, terminated when the host filesystem returned disk-full
(`os error 112`); no governed replay ran.

Frozen order: NH Wilson, NH Kruskal, NM Wilson, NM Kruskal, GA Wilson, GA
Kruskal. All six excluded 25-step preflights and their fresh-process exact
normalized replays passed under the enforcing ledger. The canonical Stage 0
package verifies with `scripts/research/verify_block_ensemble_expansion.py`.

The evidence history deliberately retains two engineering records:

- `preflight-pre-compression/` preserves the initial sweep before deterministic
  gzip custody was added.
- `preflight-determinism-failure/` preserves the NM Wilson replay mismatch that
  exposed process-dependent `HashSet` iteration for three-district adjacency.

The defect was remediated by sorting adjacent district pairs before the seeded
shuffle. A new same-seed, three-district regression test and a completely fresh
six-run preflight/replay sweep passed before Stage 2 authorization. See
`preflight-summary.json` for the canonical measurements and claim boundary.

`terminal-failure.md` records the final infrastructure failure and the frozen
stop rule. The five completed primary traces remain valid retained evidence,
but the expansion gate did not pass and they cannot support the protocol's
completed-expansion claim.

Budgets inherited from the verified Pulse 24 package are 21 cumulative runner
hours across primary and replay, 2.25 GiB per process, 3 GiB retained evidence,
and 3 GiB scratch evidence. See
`docs/specs/2026-08-10-nrs-v0.3-block-ensemble-expansion.md`.
