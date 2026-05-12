# Goal: Benchmark-Tier RPLAN Package Frontier

**Status:** Complete
**Date:** 2026-05-11
**Owner:** BISECT / RPLAN publication packaging
**Depends on:** `docs/specs/2026-05-11-method-produced-rplan-package-goal.md`

## Goal

Define and build the next package tier after tiny golden fixtures and
method-produced fixtures: benchmark-tier RPLAN packages.

The purpose is empirical publication support. A benchmark-tier package should
carry a larger or more realistic workflow with data provenance, timing notes,
command transcript, method transcript, RPLAN/RCTX artifacts, audit certificate,
and manifest verification.

This goal should remain claim-disciplined. Benchmark packages can support
runtime, replayability, and empirical smoke-test claims. They do not by
themselves prove legal sufficiency, fairness, optimality, or universal method
superiority.

## Stage 0 - Frontier Setup

- [x] Add a visual dependency map for the T/U paper, crate, package, and
      verifier portfolio.
- [x] Update `docs/NEXT_SESSION.md` so it points past the completed
      method-produced package goal.
- [x] Record this benchmark-tier goal as the next queued packaging frontier.

## Stage 1 - Candidate Selection

- [x] Pick a first benchmark-tier candidate.
- [x] Decide whether it is construction-side, search-side, exact-side, or
      audit-side.
- [x] Prefer a deterministic no-download or tiny pinned-data workflow first.
- [x] Record expected runtime, data requirements, and repository/artifact-store
      footprint before generating outputs.

## Stage 2 - Package Contract

- [x] Decide whether benchmark packages can reuse the public example manifest
      schema or need `benchmark-rplan-package-manifest-v1`.
- [x] Require timing and hardware notes.
- [x] Require source data hashes for all non-synthetic inputs.
- [x] Require a command transcript and method transcript.
- [x] Decide whether large files belong in-repo or in a release artifact bundle.

## Stage 3 - Generate First Benchmark Package

- [x] Generate the chosen package from a real CLI/crate workflow.
- [x] Include `plan.rplan`, `context.rctx`, `audit-certificate.json`,
      `manifest.json`, and transcript files.
- [x] Verify through `rplan verify-certificate`.
- [x] Verify through `bisect verify --manifest` or document why the bridge
      intentionally does not apply.
- [x] Document package claim boundaries.

## Stage 4 - Evidence Integration

- [x] Update affected T/U paper evidence tables only if the benchmark package
      changes the claim level.
- [x] Rebuild affected PDFs only when manuscript text changes.
- [x] Update `docs/PAPERS.md` notes if evidence labels change.
- [x] Update `docs/concepts/t-u-portfolio-dependency-map.md` if the package
      tier taxonomy changes.
- [x] Update `docs/NEXT_SESSION.md`.

## Stage 5 - Review, Commit, Push

- [x] Run focused verifier tests.
- [x] Run package verification commands documented in the package README.
- [x] Run `git diff --check`.
- [x] Commit coherent slices.
- [x] Push to `origin/main`.

## Stage 6 - Add Search-Side Benchmark Package

- [x] Generate a search-side benchmark package from a real CLI workflow.
- [x] Use the T.14 benchmark package as the audited parent input.
- [x] Include parent-plan lineage and the raw `bisect improve` run manifest.
- [x] Verify through `rplan verify-certificate`.
- [x] Verify through `bisect verify --manifest`.
- [x] Update U.18 evidence docs and package inventory.
- [x] Run focused benchmark corpus tests.
- [x] Commit and push the second benchmark slice.

## Stage 7 - Add Exact-Side Benchmark Package

- [x] Generate an exact-side benchmark package from the `bisect-ilp`
      branch-and-cut solver path.
- [x] Include the CPLEX-LP model artifact and `ilp-solve-report.json` hashes.
- [x] Record branch-and-cut certificate metadata in method transcript and
      RPLAN algorithm lineage.
- [x] Verify through `rplan verify-certificate`.
- [x] Verify through `bisect verify --manifest`.
- [x] Update U.16 evidence docs and package inventory.
- [x] Run focused benchmark corpus tests.
- [x] Commit and push the exact benchmark slice.

## Candidate Matrix

| Candidate | Side | Data | Footprint | Why It Is Useful | Risk |
|---|---|---|---|---|---|
| T.14 spectral grid10 benchmark | Construction | no external data | small, in-repo | Exercises benchmark manifest, timing notes, 100-unit context, and verifier bridge | Still synthetic; no wall-clock claim |
| U.16 branch-and-cut path8 benchmark | Exact | no external data | small, in-repo | Exercises solver-grade transcript, LP model artifact hashing, exact certificate metadata, and verifier bridge | Still synthetic and intentionally tiny |
| U.18 local-search grid10 benchmark | Search | no external data; parent is T.14 grid benchmark | small, in-repo | Exercises benchmark search packaging, no-improvement transcript, and parent-plan lineage | Still synthetic; no improvement claim |
| Synthetic multi-family benchmark | Construction/search | no external data | small, in-repo | Exercises multiple package producers with timing notes | Still synthetic |
| T.14 spectral small real state | Construction | TIGER/PL or pinned fixture | medium | Raises spectral evidence beyond synthetic path fixture | External data and larger artifacts |
| U.18 local-search real descendant | Search | existing RPLAN/RCTX input | small to medium | Shows improvement/no-improvement transcript on a realistic parent plan | Needs realistic parent package |
| U.16 formulation-only exact package | Exact | synthetic or tiny pinned | small | Gives solver-grade benchmark transcript without external solver dependency | Superseded by the path8 solved exact benchmark for now |
| U.20 audit stress package | Audit | synthetic generated | small | Exercises verifier scale and negative/positive behavior | Does not advance algorithm quality claims |

## Suggested `/goal`

```text
/goal Build the benchmark-tier RPLAN package frontier described in docs/specs/2026-05-11-benchmark-rplan-package-frontier-goal.md: select the first deterministic candidate, define the benchmark package contract, generate and verify a real workflow package, update evidence docs only where warranted, commit, and push stage by stage.
```
