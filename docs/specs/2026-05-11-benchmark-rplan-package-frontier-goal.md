# Goal: Benchmark-Tier RPLAN Package Frontier

**Status:** Queued
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

- [ ] Pick a first benchmark-tier candidate.
- [ ] Decide whether it is construction-side, search-side, exact-side, or
      audit-side.
- [ ] Prefer a deterministic no-download or tiny pinned-data workflow first.
- [ ] Record expected runtime, data requirements, and repository/artifact-store
      footprint before generating outputs.

## Stage 2 - Package Contract

- [ ] Decide whether benchmark packages can reuse the public example manifest
      schema or need `benchmark-rplan-package-manifest-v1`.
- [ ] Require timing and hardware notes.
- [ ] Require source data hashes for all non-synthetic inputs.
- [ ] Require a command transcript and method transcript.
- [ ] Decide whether large files belong in-repo or in a release artifact bundle.

## Stage 3 - Generate First Benchmark Package

- [ ] Generate the chosen package from a real CLI/crate workflow.
- [ ] Include `plan.rplan`, `context.rctx`, `audit-certificate.json`,
      `manifest.json`, and transcript files.
- [ ] Verify through `rplan verify-certificate`.
- [ ] Verify through `bisect verify --manifest` or document why the bridge
      intentionally does not apply.
- [ ] Document package claim boundaries.

## Stage 4 - Evidence Integration

- [ ] Update affected T/U paper evidence tables only if the benchmark package
      changes the claim level.
- [ ] Rebuild affected PDFs only when manuscript text changes.
- [ ] Update `docs/PAPERS.md` notes if evidence labels change.
- [ ] Update `docs/concepts/t-u-portfolio-dependency-map.md` if the package
      tier taxonomy changes.
- [ ] Update `docs/NEXT_SESSION.md`.

## Stage 5 - Review, Commit, Push

- [ ] Run focused verifier tests.
- [ ] Run package verification commands documented in the package README.
- [ ] Run `git diff --check`.
- [ ] Commit coherent slices.
- [ ] Push to `origin/main`.

## Candidate Matrix

| Candidate | Side | Data | Footprint | Why It Is Useful | Risk |
|---|---|---|---|---|---|
| Synthetic multi-family benchmark | Construction/search | no external data | small, in-repo | Exercises multiple package producers with timing notes | Still synthetic |
| T.14 spectral small real state | Construction | TIGER/PL or pinned fixture | medium | Raises spectral evidence beyond synthetic path fixture | External data and larger artifacts |
| U.18 local-search real descendant | Search | existing RPLAN/RCTX input | small to medium | Shows improvement/no-improvement transcript on a realistic parent plan | Needs realistic parent package |
| U.16 formulation-only exact package | Exact | synthetic or tiny pinned | small | Gives solver-grade benchmark transcript without external solver dependency | May be too close to golden fixtures |
| U.20 audit stress package | Audit | synthetic generated | small | Exercises verifier scale and negative/positive behavior | Does not advance algorithm quality claims |

## Suggested `/goal`

```text
/goal Build the benchmark-tier RPLAN package frontier described in docs/specs/2026-05-11-benchmark-rplan-package-frontier-goal.md: select the first deterministic candidate, define the benchmark package contract, generate and verify a real workflow package, update evidence docs only where warranted, commit, and push stage by stage.
```
