---
wave: national-standard-evidence-and-specification
date_open: 2026-07-09
date_closed: 2026-07-10
status: complete
source_goal: fair national redistricting standard
vtrace_posture: internal_engineering_baseline_only
---

# National Standard Evidence And Specification

## Mission

Turn BISECT's strongest technical and research work into one coherent,
auditable proposal for a national congressional redistricting standard.

The wave will define a canonical standard, align the statute and flagship
claims to that standard, close the highest-risk evidence and reproducibility
gaps, and produce an external replication and governance pathway.

## Strategic Thesis

The proposed national standard is a reproducible decision process, not a claim
that one algorithm produces the uniquely fair map:

1. A mandatory constitutional and data floor.
2. A partisan-input-excluded, reproducible baseline plan.
3. Publicly justified legal and community modifications.
4. Multi-metric and ensemble evaluation of the baseline and final plan.

## Claim Boundary

This wave may define, reconcile, test, and document a proposed standard. It
must not claim legislative enactment, judicial acceptance, external peer
review, VRA compliance for a specific plan, clean reproducibility, or national
fairness certification until the applicable evidence and external gates pass.

Internal panel scores and synthetic packages remain internal evidence only.

## Success Metrics

| Metric | Baseline | Target | Actual | Status |
|---|---|---|---|---|
| Canonical national-standard specifications | Competing statute and paper definitions | One versioned specification and conformance matrix | NRS v0.1 plus Technical Schedules A/B and Exact North Star | Met |
| Flagship unsupported or inconsistent claims | Known conflicts in README, A.0, B.02, and C.6 | All classified, corrected, or explicitly blocked | A.0/B.02 bounded; C.6 protocol-only; counts aligned | Met |
| Reproducible toolchain | Moving `stable`; stale build documentation | Exact toolchain pin and one clean reference replay | Rust 1.95, canonical output, RI reference package and replay | Met |
| Real ensemble evidence | G.1-G.3 synthetic/missing-real-evidence posture | Archived real traces, diagnostics, inputs, and cross-tool comparison | 48 chains across RI/IA/NC; Rust/GerryChain; WI failure retained | Met |
| External replication pathway | Internal roles and panels only | Publishable protocol, challenge process, and replication bundle contract | Hash-bound challenge bundle plus automated non-author pass candidate | Met internally; human open |
| Statutory alignment | Binary/geographic statute vs. prime-factor/county proposal | One legal posture with explicit fallback and exception process | v0.2 benchmark/disclosure bill with funded governance and fallbacks | Met internally |

## Governing Inputs

| Input | Source |
|---|---|
| Fairness claim boundary | `docs/legal/FAIRNESS_DOCTRINE.md` |
| Proposed federal text | `docs/legal/MODEL_FEDERAL_STATUTE.md` |
| One-law research argument | `research/tracks/B-foundations/B.02+one-federal-law/` |
| Evidence posture | `docs/vtrace/PAPER_EVIDENCE_INVENTORY.md` |
| Release gates | `docs/vtrace/RELEASE_GATE_REGISTER.md` |
| Reproducibility procedure | `docs/REPRODUCIBLE_BUILD.md` |
| Algorithm configuration | `docs/concepts/three-layer-compositor.md` |
| Wave rules | `context/waves/PHASES.md` |

## Pulse Status

| Pulse | Status | Outcome |
|---|---|---|
| 01 - Canonical standard decision | DONE | `docs/specs/2026-07-09-national-redistricting-standard-v0.1.md`; panel remediation complete |
| 02 - Flagship claim integrity | DONE | Counts aligned; A.0/B.02 bounded; C.6 converted to protocol; PDFs rebuilt |
| 03 - Reproducibility reference baseline | DONE | Rust 1.95 pin, fixed seed metadata, canonical output, and two-run Rhode Island reference package |
| 04 - Real ensemble evidence package | DONE | RI/IA/NC Rust and GerryChain traces, RPLAN/RCTX audits, diagnostics, and revised G.1-G.3/A.0 |
| 05 - Statute, governance, and exception alignment | DONE | v0.2 benchmark-and-disclosure bill, technical/evaluation schedules, funded governance, and revised B.02 |
| 06 - External replication and synthesis | DONE | Challenge bundle, non-author replay, revised A.0/A.5, adoption matrix, and Exact Canonical Benchmark North Star |

## Dependency Graph

```text
01 Canonical decision
 |-- 02 Claim integrity
 |-- 03 Reproducibility baseline -- 04 Real ensemble evidence
 `-- 05 Statute and governance

02 + 03 + 04 + 05 -- 06 External replication and synthesis
```

## Risks And Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Canonicalization becomes a hidden normative choice | High | Publish alternatives, rejected options, and decision rationale |
| Advocacy claims outrun evidence | High | Require evidence-posture labels and DATUM/SCALE review |
| VRA or community review is treated as an algorithmic afterthought | High | Make modifications a first-class, reviewable layer |
| Reproducibility depends on unavailable local data | High | Hash and publish input manifests; separate clean replay from local development data |
| Internal panels are mistaken for peer review | High | Preserve explicit internal-only labels and require outside replication |
| A federal mandate triggers anti-commandeering objections | High | Develop direct-federal, preemption, commission-support, and conditional-funding postures |

## Wave Validation

Every pulse must run:

```powershell
git --no-pager diff --check
```

Documentation pulses must verify cited paths and evidence labels. Code or data
pulses must additionally name targeted package tests, deterministic replay
checks, and artifact hashes in their pulse closure.

## Closure Rule

Close only when all six pulses are done or explicitly deferred with named
evidence blockers. Closure must state which claims remain internal,
publication-ready, externally replicated, legally proposed, or unresolved.
