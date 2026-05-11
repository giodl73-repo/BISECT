# Goal: Complete Remaining Algorithm-Family Roadmap

**Status:** Active  
**Track:** T/U algorithm-family completion  
**Foundation:** [`2026-05-10-algorithm-family-roadmap.md`](2026-05-10-algorithm-family-roadmap.md),
[`2026-05-10-u20-goal.md`](2026-05-10-u20-goal.md),
[`2026-05-11-t15-capacity-clustering.md`](2026-05-11-t15-capacity-clustering.md)

## Goal

Finish the remaining algorithm-family roadmap after the U.20 audit fixed point:
local search, spectral construction, regionalization, branch-and-price,
flow-based construction, and evolutionary comparison. Each family must land
with a spec, crate/module boundary, CLI or subcommand surface, deterministic
fixtures, RPLAN audit sidecars, verification/report integration where relevant,
and focused L0/L1/L2 coverage.

## Current Baseline

- [x] U.20 RPLAN audit integration fixed point is complete.
- [x] U.16 ILP branch-and-cut lineage and audit package coverage are complete
      for the current roadmap milestone.
- [x] T.15 capacity-constrained clustering is implemented through CLI sidecar
      emission and RPLAN audit lineage.

## Global Invariants

- New crates may depend on `bisect-core` and `rplan-*`; `rplan-*` crates must
  not depend on `bisect-*`.
- Every final plan emitted by a new family must either pass `rplan-audit` or
  fail with a structured, reproducible reason.
- Every solver family must expose versioned output metadata and
  `algorithm_lineage` without duplicating reserved audit-certificate fields.
- Every randomized path must record seed derivation and be deterministic for a
  fixed input and seed.
- Every stage must end with focused tests, `cargo +stable fmt --check`,
  relevant `cargo +stable test`/`check`, `git diff --check`, and a commit.

## Stage 0 - Goal Setup

- [x] Create this active completion goal.
- [x] Keep the goal checklist current as stages land.

## Stage 1 - U.18 Local Search

Purpose: add practical plan improvement methods and shared repair hooks for
later construction/evolutionary families.

- [x] Write U.18 implementation spec and role-review notes.
- [x] Add `bisect-local-search` crate or stage-one module boundary.
- [x] Define input contract for improving an existing audited RPLAN plan.
- [x] Implement deterministic one-move improvement kernel.
- [x] Add tabu/LNS scaffolding with structured parameters, even if advanced
      search is staged behind feature-complete tests.
- [x] Emit local-search summaries and `algorithm_lineage`.
- [x] Wire `bisect improve` or `--search lns`/`--search tabu` surface.
- [x] Add L0 fixtures for determinism, validity preservation, and no-op cases.
- [x] Add L1 CLI/RPLAN sidecar test.
- [x] Add L2 verify/report smoke path if the CLI surface emits final plans.

## Stage 2 - T.14 Spectral Partitioning

Purpose: add a cheap deterministic construction baseline.

- [x] Write T.14 implementation spec and role-review notes.
- [x] Add `bisect-apportion::spectral` module.
- [x] Implement deterministic Fiedler-vector approximation with convergence
      metadata.
- [x] Implement population-balanced sweep cut and min-cut tie-breaking.
- [x] Recurse through the existing bisection compositor.
- [x] Add `--structure spectral` and YAML `structure: spectral`.
- [x] Record spectral parameters in manifests and `algorithm_lineage`.
- [x] Add L0 path/two-clique/determinism tests.
- [x] Add L1 CLI/RPLAN sidecar test.
- [x] Phase-review hardening: recursive spectral splits pass proportional
      target fractions for odd/non-power-of-two district counts.

## Stage 3 - T.16 Hierarchical Regionalization

Purpose: extend `bisect-clustering` beyond flat capacity assignment.

- [x] Write T.16 implementation spec and role-review notes.
- [x] Add regionalization module in `bisect-clustering`.
- [x] Implement deterministic agglomerative/regionalization baseline.
- [x] Add population-capacity and contiguity repair witnesses.
- [x] Add `--structure regionalization` and YAML `structure: regionalization`.
- [x] Emit regionalization summaries and `algorithm_lineage`.
- [x] Add L0 hierarchy/capacity/determinism fixtures.
- [x] Add L1 CLI/RPLAN sidecar test.

## Stage 4 - U.17 Branch-And-Price / Column Generation

Purpose: add heavier exact optimization with a distinct solver lifecycle.

- [x] Write U.17 implementation spec and role-review notes.
- [x] Add `bisect-column` crate.
- [x] Define column, pricing, master-problem, and fallback output contracts.
- [x] Implement formulation-only and small exact fixture paths first.
- [x] Add `bisect exact --method branch-and-price`.
- [x] Emit solver provenance, gap/bound metadata, and `algorithm_lineage`.
- [x] Add L0 pricing/master fixtures.
- [x] Add L1 audit package test for a small exact output.
- [x] Phase-review hardening: exact fixture objective reports true partition
      edge cut and solved outputs emit RPLAN/RCTX/audit certificate/manifest.

## Stage 5 - T.17 Flow-Based Construction

Purpose: add constructive flow assignment once audit/repair patterns are stable.

- [x] Write T.17 implementation spec and role-review notes.
- [x] Add `bisect-flow` crate.
- [x] Define flow network, capacities, costs, infeasibility witnesses, and
      repair contract.
- [x] Implement small deterministic flow-construction baseline.
- [x] Add `--structure flow-construction` and YAML
      `structure: flow-construction`.
- [x] Emit flow summaries and `algorithm_lineage`.
- [x] Add L0 capacity/infeasibility/determinism fixtures.
- [x] Add L1 CLI/RPLAN sidecar test.

## Stage 6 - U.19 Evolutionary Comparison

Purpose: extend Pareto/evolutionary comparison using the shared repair and audit
contracts.

- [ ] Write U.19 implementation spec and role-review notes.
- [ ] Extend `bisect-pareto` with repair-aware evolutionary comparison.
- [ ] Define crossover/mutation validity guarantees.
- [ ] Emit frontier entries with per-plan validity status and lineage.
- [ ] Ensure selected/exported plans receive full RPLAN audit packages.
- [ ] Add L0 crossover/mutation/determinism tests.
- [ ] Add L1 selected-frontier audit package test.

## Final Acceptance

- [ ] All roadmap families have specs and implementation boundaries.
- [ ] All promised CLI/subcommand surfaces parse and run or return structured
      staged errors only where explicitly documented.
- [ ] Every final plan path emits RPLAN sidecars and passes `bisect verify`.
- [ ] Reports surface audit identity and lineage for the new final-plan paths.
- [ ] The algorithm-family roadmap and this goal checklist are updated to
      reflect the completed state.
- [ ] Final milestone commit and push are complete.

## Suggested `/goal`

```text
/goal Finish the remaining T/U algorithm-family roadmap in
docs/specs/2026-05-11-algorithm-family-completion-goal.md. Work stage by stage:
U.18 local search, T.14 spectral, T.16 regionalization, U.17 branch-and-price,
T.17 flow construction, and U.19 evolutionary comparison. For each stage, keep
the checklist current, implement the smallest audited vertical slice first,
verify with focused tests plus fmt/diff checks, commit, and continue until the
final acceptance checklist is complete.
```
