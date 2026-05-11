# Spec: U.18 Local Search And Improvement

**Status:** Stage 1 active  
**Track:** U.18 - Search and Optimization  
**Depends on:** U.20 RPLAN audit integration fixed point

## Purpose

Add audited local-search improvement methods that start from an existing plan,
preserve validity, improve one or more objectives, and emit reproducible
lineage. U.18 is the shared improvement layer for later LNS/tabu workflows and
for repair-aware evolutionary comparison in U.19.

## Scope

The first vertical slice is deliberately small:

1. Accept an existing assignment in RPLAN unit order.
2. Evaluate deterministic one-vertex boundary moves.
3. Accept the lexicographically first best move that strictly reduces edge cut
   while preserving population tolerance and district contiguity.
4. Emit a versioned local-search summary.
5. Build `algorithm_lineage` through `rplan-audit`.

Tabu and large-neighborhood search are part of the public configuration model,
but remain staged until the one-move contract is wired through CLI/RPLAN
sidecars.

## Crate Boundary

`bisect-local-search` owns:

- local improvement kernels
- local-search summary output
- local-search lineage payloads
- deterministic fixture-level validity checks

It may depend on `rplan-audit` for lineage construction. It must not depend on
`bisect-cli`.

## Input Contract

The stage-one kernel takes:

- `adjacency: &[Vec<usize>]`
- `weights: &[i64]`
- `assignment: &[usize]`, with district ids in `0..k`
- `LocalSearchConfig { k, tolerance, method }`

The caller is responsible for mapping from `.rplan` unit order to this
assignment order. CLI wiring must use the RPLAN unit index/context rather than
an external ordering convention.

## Output Contract

`LocalSearchSummary` includes:

- schema version
- method
- status
- moves evaluated/accepted
- initial/final edge cut
- initial/final population deviation
- tolerance
- parameter hash

The lineage payload uses non-reserved fields only and sets
`producer_crate = "bisect-local-search"`.

## CLI Staging

Preferred surface:

- `bisect improve --plan PATH --context PATH --method one-move`

Acceptable early integration:

- `--search lns`
- `--search tabu`

Any CLI path that emits a final plan must write `.rplan`, `.rctx`, and
`audit-certificate.json`, then pass `bisect verify`.

## Tests

Stage-one tests:

- a valid fixture accepts one move that improves edge cut
- fixed input returns identical assignment and summary
- no-improvement fixture preserves the assignment
- staged tabu/LNS configs return structured errors
- summary builds `rplan-audit::AlgorithmLineage`
