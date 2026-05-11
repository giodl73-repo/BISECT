# Goal: Write Algorithm-Family Papers

**Status:** Active - Stage 2 Track U spine drafting in progress
**Track:** T/U algorithm-family publication  
**Foundation:**
[`2026-05-11-algorithm-family-completion-goal.md`](2026-05-11-algorithm-family-completion-goal.md),
[`2026-05-10-algorithm-family-roadmap.md`](2026-05-10-algorithm-family-roadmap.md),
[`docs/concepts/algorithm-family-layer-cake.md`](../concepts/algorithm-family-layer-cake.md)

## Goal

Turn the completed algorithm-family implementation slices into native T/U
research papers with source directories, draft manuscripts, evidence plans,
panel-style simulated quality review artifacts, revisions, compiled PDFs, and
`docs/PAPERS.md` index entries.

The implementation milestone is complete. This goal is about paper production:
making the contribution, scope, evidence, limitations, and audit story legible.

## Panel Review Expectations

Use the local panel process in `C:\src\panel` as the publication-quality
discipline:

- Treat panel outputs as AI-generated quality-improvement simulations, not real
  peer review.
- Use generic `article` class by default; use conference templates only when a
  real submission target requires it.
- Each paper starts from a `plan.md` with falsifiable hypotheses, scope
  boundaries, baselines, evaluation criteria, figures, tables, and limitations.
- Each paper should move through the lifecycle:
  `draft -> panel -> synthesis -> revision -> recheck -> ready`.
- Paper-level readiness requires at least 5 simulated review artifacts, a
  P1/P2/P3 synthesis, all P1 items addressed, average score >= 2.5/4, and no
  individual score below 2/4.
- Track/module readiness requires a cross-paper panel pass with PP1 items
  addressed.
- Use language such as "simulated feedback", "quality-improvement
  suggestions", and "AI persona based on ..."; do not describe the artifacts as
  real reviewer feedback or real acceptance.

## Global Paper Invariants

- Every algorithm paper must name the exact crate/module and CLI/YAML surface.
- Every paper with final-plan output must explain the RPLAN/RCTX/audit
  certificate/manifest fixed point.
- Claims must be separated by evidence level:
  fixture correctness, real-data smoke, benchmark result, empirical sweep, or
  legal/policy interpretation.
- Baseline slices may be papered as staged implementations, but limitations
  must be explicit.
- No paper may imply that certificate pass/fail proves political fairness or
  legal compliance beyond the declared audit profile.
- Every quantitative claim needs a reproducible script, command transcript, or
  explicitly marked future-evidence placeholder before publication-ready status.

## Stage 0 - Writing Infrastructure

- [x] Create/update paper directories under `research/tracks/T-plan-construction`
      and `research/tracks/U-search-optimization`.
- [x] Add `plan.md` for every paper in this goal using the panel plan template.
- [x] Add or update per-track `MODULE.md` inventories to move papers out of
      "candidate" status.
- [x] Decide whether these papers share a common architecture figure based on
      `docs/concepts/algorithm-family-layer-cake.md`.
- [x] Add a paper evidence matrix mapping each paper to specs, crates, tests,
      benchmarks, and figures.
- [x] Add first-draft `main.tex`/`Makefile` skeletons for the Stage 1 T papers.

## Stage 1 - Track T Construction Papers

- [x] T.14 Spectral Partitioning: deterministic Laplacian/Fiedler-vector
      construction baseline, proportional recursive split hardening, CLI
      `--structure spectral`.
- [x] T.15 Capacity-Constrained Clustering: capacity assignment, repair-aware
      summaries, audited sidecars, CLI `--structure capacity-clustering`.
- [x] T.16 Hierarchical Regionalization: agglomerative connected-region
      construction, merge witnesses, CLI `--structure regionalization`.
- [x] T.17 Flow-Based Construction: capacity/cost flow baseline,
      infeasibility witnesses, CLI `--structure flow-construction`.

Acceptance for each T paper:

- [x] `main.tex` and section files exist.
- [x] `plan.md` lists hypotheses, scope, baselines, failure modes, figures,
      and limitations.
- [x] The implementation section cites the crate/module boundary and CLI
      surface.
- [x] The audit section explains final-plan RPLAN sidecars where applicable.
- [x] The evaluation section distinguishes synthetic L0/L1 fixtures from
      real-data or future benchmark claims.
- [x] Paper-level simulated review/revision artifacts meet panel readiness
      thresholds.
- [x] PDF is built and indexed in `docs/PAPERS.md`.

## Stage 2 - Track U Spine Papers

- [x] Add first-draft `main.tex`/`Makefile` skeletons for the Stage 2 U spine
      papers.
- [ ] U.0 Search and Optimization Overview: taxonomy and bridge from existing
      U.1-U.11 papers to U.12-U.20.
- [ ] U.12 Algorithm-Selection Matrix: when to use construction, search,
      ensemble, exact, Pareto, and audit paths.
- [ ] U.13 Exact-vs-Heuristic Certification: what certificates, bounds, and
      audit sidecars do and do not prove.
- [ ] U.14 Multi-Objective Selection: selection among objective trade-offs,
      including SMC/Pareto/exported-plan audit framing.
- [ ] U.15 Legal Postures for Search: how search choices support or weaken
      legal/policy claims without overstating neutrality.

Acceptance for each U spine paper:

- [ ] Paper states whether it is taxonomy, methodology, empirical, or legal
      interpretation.
- [ ] Claims are grounded in existing implementation or explicitly marked as
      roadmap/future work.
- [ ] Related-work and limitations sections separate computational claims from
      legal interpretation.
- [ ] Paper-level simulated review/revision artifacts meet panel readiness
      thresholds.
- [ ] PDF is built and indexed in `docs/PAPERS.md`.

## Stage 3 - Track U Implementation Papers

- [ ] U.16 Branch-And-Cut Redistricting: separation routines, connectivity
      cuts, solve reports, and audit lineage.
- [ ] U.17 Branch-And-Price Redistricting: column generation, pricing/master
      contracts, exact fixture solution, and audit package.
- [ ] U.18 Large-Neighborhood Search: one-move baseline, tabu/LNS scaffolding,
      validity preservation, `bisect improve`.
- [ ] U.19 Evolutionary Search Comparison: NSGA-II lineage, validity-preserving
      crossover/mutation, selected-frontier audit packaging.
- [ ] U.20 Plan Audit Certificates: RPLAN/RCTX schema, audit certificates,
      algorithm lineage, verifier contract, and fixed-point role.

Acceptance for each U implementation paper:

- [ ] `main.tex` and section files exist.
- [ ] `plan.md` lists hypotheses, scope, baselines, failure modes, figures,
      and limitations.
- [ ] The implementation section cites the crate/module boundary and CLI
      surface.
- [ ] The audit section explains RPLAN/RCTX/certificate/manifest artifacts.
- [ ] The evaluation section cites the relevant L0/L1/L2 coverage and states
      what remains empirical rather than proven.
- [ ] Paper-level simulated review/revision artifacts meet panel readiness
      thresholds.
- [ ] PDF is built and indexed in `docs/PAPERS.md`.

## Stage 4 - Module-Level Review

- [ ] Run a Track T module-level simulated panel over T.14-T.17.
- [ ] Address all Track T PP1 items.
- [ ] Run a Track U module-level simulated panel over U.0 and U.12-U.20.
- [ ] Address all Track U PP1 items.
- [ ] Update `research/tracks/T-plan-construction/MODULE.md` and
      `research/tracks/U-search-optimization/MODULE.md` with final status.

## Stage 5 - Portfolio Integration

- [ ] Update `docs/PAPERS.md` with final titles, PDF links, and implementation
      notes.
- [ ] Update `docs/concepts/algorithm-family-layer-cake.md` if paper framing
      changes the taxonomy.
- [ ] Update `docs/NEXT_SESSION.md` with the next writing or empirical task.
- [ ] Run paper builds or document any build blockers.
- [ ] Commit and push each completed paper batch.

## Suggested `/goal`

```text
/goal Write the remaining algorithm-family papers listed in docs/specs/2026-05-11-algorithm-family-paper-writing-goal.md. Work stage by stage: create panel-style paper plans, draft Track T papers T.14-T.17, draft Track U spine papers U.0/U.12-U.15, draft implementation papers U.16-U.20, run simulated paper/module review cycles, address P1/PP1 items, build PDFs, update docs/PAPERS.md, commit, and continue until portfolio integration is complete.
```
