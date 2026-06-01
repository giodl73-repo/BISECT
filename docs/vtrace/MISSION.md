# Mission

## M-01 Scope

Repo: BISECT / apportionment workspace at `C:\src\apportionment`.

VTRACE adoption scope: establish repo-local VTRACE control artifacts for the full civic evidence platform now present in this repo: BISECT redistricting, RPLAN plan packages, RCOUNT count packages, RCTX context/crosswalks, RHIST unit history, shared deterministic kernels, research papers, evidence packages, and wave/pulse execution records.

## M-02 Mission Need

This repo exists to make districting, apportionment, election-count evidence, and related civic analysis reproducible, inspectable, and reviewable from first principles.

The mission is to provide a deterministic public workbench where a user can move from source data and explicit configuration to district plans, count packages, audits, analyses, reports, research claims, and publication evidence with a traceable chain from mission need to requirement, source artifact, command, implementation, verification, validation, and review.

## M-03 Users

| User | Need | Success Signal |
|---|---|---|
| Court-appointed special master | Generate and explain algorithmic baseline district plans with auditable provenance and reported effects | A plan, report, and verification chain can be produced and independently checked |
| Academic researcher | Reproduce algorithmic results, papers, metrics, and sensitivity claims | Papers cite evidence packages, commands, data boundaries, and uncertainty status |
| Election-law expert | Separate legal, empirical, operational, and advocacy claims | Claims are classed, sourced, and reviewable against evidence |
| State staff or commission | Run practical redistricting workflows for congressional or state legislative chambers under jurisdiction-specific rules | State or chamber runs complete with documented inputs, outputs, assumptions, limits, and failure modes |
| Election official or auditor | Verify arithmetic, lineage, reconciliation, and package integrity without displacing legal certification authority | Count packages expose contests, ballots, batches, CVRs where available, reconciliation status, and audit replay evidence |
| Civic advocacy group | Inspect algorithmic baselines and community impacts without relying on black-box tools | Public dashboards, reports, and package artifacts explain what was produced, what was not measured, and who may be affected |
| Future agent or maintainer | Safely extend crates, papers, and workflows without breaking traceability | Work is organized through waves/pulses with requirements, tests, and evidence links |

## M-04 Operating Context

The system is a Rust-first civic evidence workspace with supporting Python and LaTeX assets. It runs locally from the command line, generates gitignored data and run artifacts, commits selected documentation and paper PDFs, and uses wave/pulse records for controlled development.

The repo covers multiple linked domains:

| Domain | Role |
|---|---|
| BISECT | Algorithmic redistricting by recursive bisection, apportionment-region structures, search modes, metrics, maps, reports, dashboards |
| RPLAN | District-plan package model, canonical hashes, schema versions, audit checks, certificates, import/export |
| RCOUNT | Election count package model, contest and batch semantics, reconciliation, audit replay, district aggregation |
| RCTX | Shared context, census/election geography, and crosswalk verifier primitives |
| RHIST | Unit-history, jurisdictional lineage, and versioned geography verification |
| Shared kernels | Deterministic graph, geometry, population-balance, math, statistics, search, and optimization primitives |
| Research corpus | Research papers, journals, specs, evidence packages, and claim-review ledgers |
| Waves and pulses | Execution control for finishing, fixing, reviewing, and documenting work |

## M-05 Constraints

- Outputs must be reproducible, hash-bound where applicable, and inspectable by third parties from source files, commands, manifests, schemas, binaries, and generated artifacts.
- High-stakes claims must distinguish descriptive, empirical, statistical, legal, operational, and advocacy posture.
- Quantitative claims must state evidence status, uncertainty, assumptions, data coverage, and whether the claim is proven, partial, conjectural, or a known gap.
- Redistricting algorithms must preserve stated boundaries between geography, demographics, partisan data, VRA analysis, legal interpretation, community impact, and political effect reporting.
- Congressional, state legislative, VRA, equal-population, and state-law requirements must remain separate gates with jurisdiction-specific assumptions.
- Census and boundary workflows must preserve provenance for PL 94-171, TIGER/Line, ACS, state election geography, precincts, districts, and derived crosswalks.
- Election-count workflows must preserve ballots, contests, batches, CVRs where available, write-ins, overvotes, undervotes, vendor exports, reconciliation records, and canvass status without flattening distinct accounting concepts.
- The software may verify arithmetic, lineage, manifests, and replayability, but election certification remains with authorized officials under applicable law.
- Public verification must not create ballot-choice receipts, reidentification risk, coercion channels, or unnecessary exposure of protected voter data.
- Build and test expectations must remain crate-specific and evidence-specific, and regression tests must be able to fail for the pitfall they are meant to prevent.
- Package schemas, exports, compatibility claims, and external data contracts must be versioned and traceable.
- Generated data, large census files, run outputs, and non-committed artifacts must remain outside version control unless explicitly intended as public evidence.
- Research papers and README claims must not outrun current package evidence, verification status, or review state.
- Known failures, recurrent defects, and unsafe assumptions must become tracked pitfalls with structural prevention and validation coverage.
- VTRACE adoption must integrate with existing waves/pulses rather than replacing the repo's execution model.

## M-06 Non-Goals

- This repo does not claim that any single algorithm proves legal fairness by itself.
- This repo does not replace courts, legislatures, commissions, or expert judgment.
- This repo does not certify elections, adjudicate voter intent, or override official canvass and certification processes.
- This repo does not claim that legal compliance, mathematical compactness, or algorithmic reproducibility alone proves community fairness.
- This repo does not make unsupported final claims from seed papers, smoke fixtures, or incomplete external traces.
- This repo does not require NASA endorsement or a spaceflight lifecycle; VTRACE is adapted as lightweight traceable engineering discipline.
- This repo does not treat dashboards or PDFs alone as sufficient evidence without source, command, package, or review traceability.

## M-07 Success Criteria

| Criterion | Validation Method | Evidence Pointer |
|---|---|---|
| A user can understand what the repo is for | Review README, docs indexes, VTRACE mission | `README.md`, `docs/PAPERS.md`, `docs/vtrace/MISSION.md` |
| A user can run core workflows | Command verification and documented examples | `docs/BISECT_CLI.md`, quickstarts, CLI tests |
| Workflows are usable by non-authors | Documented dependencies, commands, limits, fixtures, expected outputs, and failure modes | README, quickstarts, crate docs, VTRACE verification |
| Plan artifacts can be audited | Hash/package verification, schema version checks, geography provenance checks | RPLAN crates, report/verify commands, package examples |
| Research claims are controlled | Paper index, quality ledgers, uncertainty statements, evidence packages, review panels | `docs/PAPERS.md`, `research/journals/`, `docs/papers/*REVIEW*` |
| Count and district evidence can connect | Package-level verification across RPLAN/RCOUNT/RCTX/RHIST without collapsing canvass, contest, batch, or geography semantics | RCOUNT, RCTX, RHIST crates and VTRACE specs |
| Privacy and public verification coexist | Verification surfaces avoid receipt creation, reidentification risk, and protected-data exposure | RCOUNT/RCTX package rules, VTRACE requirements |
| Jurisdiction-specific rules remain explicit | Congressional, state legislative, VRA, population, and state-law gates are documented independently | legal docs, quickstarts, requirements trace |
| Regressions are catchable | Tests, fixtures, or review gates fail when known pitfalls recur | crate tests, pitfall records, verification matrix |
| Work proceeds in reviewable slices | Wave/pulse execution with gates and validation commands | `context/waves/PHASES.md`, planned VTRACE adoption wave |
| Future changes remain traceable | Requirements, trace matrix, DCRs, verification, review gates | `docs/vtrace/REQUIREMENTS.md`, `TRACE.md`, `DCRS.md`, planned `VERIFICATION.md`, `REVIEW.md` |

## M-08 First Validation Scenarios

| Scenario | Expected Validation |
|---|---|
| Build or inspect a named redistricting plan | Config, output, analysis, report, and verify stages are linked |
| Review a research claim from README or PAPERS | Claim can be traced to a paper, evidence package, uncertainty statement, review state, or marked gap |
| Verify a plan/count package boundary | RPLAN/RCOUNT/RCTX/RHIST responsibilities are explicit and testable, with package schemas and source-data contracts identified |
| Audit a count package | Ballot, contest, batch, CVR, write-in, overvote, undervote, reconciliation, and canvass statuses are preserved where applicable |
| Check a jurisdictional rule | State-specific and chamber-specific assumptions are visible rather than silently inherited from a national default |
| Publish a public verification artifact | The artifact supports independent checking without creating ballot receipts, reidentification risk, or protected-data leakage |
| Reproduce a boundary or census-derived result | PL 94-171, TIGER/Line, ACS, state election geography, precinct, district, and crosswalk provenance is documented |
| Add or close a known pitfall | The pitfall has a prevention mechanism and a validation step that would catch recurrence |
| Execute a future wave pulse | Pulse names scope, affected packages, requirement IDs, validation commands, and review gate |

## M-09 Source Links

- `README.md`
- `Cargo.toml`
- `docs/BISECT_CLI.md`
- `docs/PAPERS.md`
- `docs/vtrace/DCRS.md`
- `research/PAPERS.md`
- `research/README.md`
- `research/journals/README.md`
- `context/waves/PHASES.md`
- `C:\src\tracker\repos\standards-protocols\vtrace\README.md`
- `C:\src\tracker\repos\standards-protocols\vtrace\docs\framework\vtrace-process.md`
- `C:\src\tracker\repos\standards-protocols\vtrace\templates\adoption\MISSION.md`
