# Concept of Operations

## CO-01 Scope

Repo or feature: BISECT / apportionment workspace at `C:\src\apportionment`.

This CONOPS defines how the civic evidence platform is used in real workflows after the mission baseline in `docs/vtrace/MISSION.md`. It covers the major operational paths for redistricting, plan packages, count packages, geography/context verification, research evidence, public review, and wave/pulse development control.

## CO-02 Actors

| Actor | Responsibility | Needs |
|---|---|---|
| Special master or map-drawing analyst | Produce and explain algorithmic baseline plans. | Explicit inputs, reproducible commands, reports, maps, verification status, and legal-boundary caveats. |
| State staff or commission analyst | Run congressional or state legislative workflows under jurisdiction-specific constraints. | State/chamber assumptions, data provenance, output limits, failure modes, and handoff-ready artifacts. |
| Election official or auditor | Inspect count packages, reconciliation, canvass status, and district aggregation without displacing certification authority. | Preserved contest, ballot, batch, CVR, write-in, overvote, undervote, reconciliation, and source-export semantics. |
| Researcher | Reproduce algorithms, metrics, sensitivity claims, papers, and evidence packages. | Versioned commands, data boundaries, uncertainty status, review status, and claim-to-evidence links. |
| Election-law expert | Separate legal, empirical, operational, statistical, and advocacy claims. | Traceable claim classes, jurisdiction-specific gates, and explicit non-claims. |
| Civic advocate or public reviewer | Inspect public artifacts and community impacts without black-box dependency. | Dashboards, reports, package manifests, privacy-safe evidence, and plain-language limitations. |
| Maintainer or future agent | Extend crates, docs, papers, and workflows without breaking traceability. | Wave/pulse scope, requirement IDs, package boundaries, validation commands, review gates, and known pitfalls. |

## CO-03 Operational Modes

| Mode | Purpose | Primary Evidence |
|---|---|---|
| Redistricting run | Build, analyze, report, map, and verify algorithmic district plans. | Configs, run outputs, reports, maps, CLI logs, verification records. |
| Plan package audit | Import, export, hash, inspect, and attest package integrity for RPLAN artifacts. | Manifests, canonical hashes, schema versions, geography provenance, audit findings. |
| Count package audit | Reconcile, replay, and aggregate RCOUNT artifacts. | Source exports, ballots/contests/batches/CVRs where available, reconciliation ledgers, lifecycle status. |
| Context/history verification | Validate crosswalks, boundary lineage, and jurisdictional history. | RCTX/RHIST manifests, source-data registry, geography lineage records, validation fixtures. |
| Research evidence review | Check paper claims against evidence packages and review ledgers. | Papers, evidence bundles, statistical summaries, review panels, claim status records. |
| Public verification | Publish inspectable artifacts without leaking protected data or creating coercion risks. | Privacy-screened reports, dashboards, package summaries, redaction notes. |
| Controlled development | Execute waves/pulses with traceable requirements and verification gates. | Pulse files, work packages, tests, review findings, trace matrix rows. |

## CO-04 Scenario: Build and Verify an Algorithmic District Plan

Trigger: an analyst needs a reproducible baseline plan for a state, year, chamber, or named configuration.

Inputs: configuration label, census year, state or national scope, algorithm structure, weights, search strategy, population tolerance, census/geography files, and relevant jurisdiction assumptions.

Normal path:

1. Confirm required census/geography data and config are present.
2. Run the appropriate BISECT CLI command for the named scope.
3. Generate analysis, maps, reports, and verification output.
4. Record command, config, output paths, hashes where available, and known limitations.
5. Check that legal-readiness gates remain separate for federal law, state law, chamber rules, VRA analysis, population equality, contiguity, subdivision preservation, and any jurisdiction-specific criteria.
6. Hand off reports and package artifacts for legal, statistical, or public review.

Failure or degraded path: missing census data, invalid config, METIS/build failure, disconnected graph, population-balance failure, jurisdictional assumption gap, incomplete report generation, or unresolved legal-readiness gate is recorded as a blocked or partial run with no readiness claim.

Outputs: district assignments, metrics, maps, reports, verification status, logs, and traceable evidence pointers.

Handoffs: RPLAN package audit, public dashboard/report, legal review, research claim review, or wave/pulse pitfall if a recurring defect is discovered.

Validation evidence: plan can be independently rebuilt or inspected from documented inputs and commands; it is not legal-ready until each applicable gate is checked and recorded separately.

## CO-05 Scenario: Audit a District-Plan Package

Trigger: a user receives or generates an RPLAN package and needs to know whether it is complete, canonical, and inspectable.

Inputs: plan package, schema version, district assignments, geography references, manifests, source metadata, optional report artifacts, and expected jurisdiction scope.

Normal path:

1. Validate package structure and schema version.
2. Check canonical bytes, domain-separated hashes, algorithm versions, and manifests.
3. Verify geography references and district assignment completeness.
4. Inspect audit checks, certificates, and compatibility claims.
5. Record findings as pass, pass with risk, blocked, or deferred.

Failure or degraded path: hash mismatch, canonicalization ambiguity, missing source metadata, schema incompatibility, invalid district assignment, stale geography, or unsupported compatibility claim is reported explicitly.

Outputs: package audit result, hash/certificate status, schema status, geography provenance status, and open findings.

Handoffs: legal/statistical review, public verification artifact, RCOUNT aggregation workflow, or package-boundary work package.

Validation evidence: another reviewer can reproduce the package audit against the same artifact, canonicalization rules, hash algorithm versions, and schema.

## CO-06 Scenario: Audit and Reconcile an Election Count Package

Trigger: an election official, auditor, or researcher needs to inspect election-count evidence or aggregate counts to districts.

Inputs: RCOUNT package, source exports, contest definitions, ballot or cast-vote records where available, ballot cards, duplicated ballots, provisional ballots, batches, central-count batches, vote centers, precincts, write-ins, overvotes, undervotes, reconciliation records, lifecycle status, and jurisdiction metadata.

Normal path:

1. Validate package structure, schema version, and source manifests.
2. Preserve distinct accounting concepts for contests, ballots, ballot cards, duplicated ballots, provisional ballots, batches, central-count batches, vote centers, CVRs, write-ins, overvotes, undervotes, vendor exports, and lifecycle status.
3. Distinguish unofficial, canvassed, recounted, amended, court-ordered, and certified totals, including cure and adjudication events where present.
4. Reconcile totals and replay audit paths where supported.
5. Aggregate counts through verified geography/context links when district analysis is requested.
6. Record what the software verifies and what remains official certification authority.

Failure or degraded path: incomplete source exports, ambiguous contest or ballot-card semantics, unsupported vendor format, missing batch/CVR/provisional/duplication data, reconciliation mismatch, protected-data risk, or uncertain lifecycle status prevents stronger claims.

Outputs: reconciliation status, replay evidence, lifecycle status, district aggregation output where applicable, privacy/security notes, and explicit certification boundary statement.

Handoffs: election official review, RCTX/RHIST context verification, statistical analysis, public verification with redaction/privacy controls, or requirements gap.

Validation evidence: arithmetic and lineage checks can be replayed without creating ballot-choice receipts, reidentification risk, or coercion channels.

## CO-07 Scenario: Verify Context, Crosswalks, and Unit History

Trigger: a workflow depends on joining census geography, election geography, precincts, districts, or historical unit lineage.

Inputs: PL 94-171, TIGER/Line, ACS, state election geography, precincts, districts, derived crosswalks, historical unit lineage, schema versions, source manifests, and transformation commands.

Normal path:

1. Identify source data and version.
2. Validate geometry, identifiers, and crosswalk completeness.
3. Check jurisdictional lineage and historical changes.
4. Record assumptions, transformation commands, and evidence status.
5. Provide verified context for BISECT, RPLAN, RCOUNT, or research workflows.

Failure or degraded path: missing source provenance, changed identifiers, ambiguous splits/merges, incomplete crosswalks, stale districts, invalid geometry, or incompatible schema blocks dependent claims.

Outputs: verified context package, crosswalk validation status, lineage notes, source-data contract status, and known gaps.

Handoffs: plan build, plan package audit, count aggregation, research evidence package, or public explanation.

Validation evidence: a downstream workflow can identify the exact geography/context source and whether it is verified, partial, conjectural, or a gap.

## CO-08 Scenario: Review a Research Claim or Paper

Trigger: README, docs, a paper, dashboard, or public statement makes a quantitative, legal, algorithmic, or operational claim.

Inputs: claim text, paper source/PDF, evidence package, command records, data boundaries, statistical summaries, uncertainty statements, review panels, and relevant package artifacts.

Normal path:

1. Classify the claim as descriptive, empirical, statistical, legal, operational, or advocacy.
2. Trace the claim to data, commands, packages, tables, figures, and review state.
3. Check whether uncertainty, assumptions, and scope limits are stated.
4. Mark the claim proven, partial, conjectural, stale, blocked, or a known gap.
5. Update review findings or downstream requirements when evidence is missing.

Failure or degraded path: missing evidence package, stale data, inconsistent numbers, unsupported legal inference, ambiguous uncertainty, or out-of-scope claim prevents publication-ready status.

Outputs: claim review finding, evidence pointer, uncertainty status, paper/repo documentation note, and possible pitfall or requirement.

Handoffs: paper edit, README/doc correction, research evidence package, VTRACE trace row, or review gate.

Validation evidence: a hostile reviewer can follow the chain from claim to current evidence or see the gap plainly.

## CO-09 Scenario: Publish a Public Verification Artifact

Trigger: a plan, count package, dashboard, report, paper, or evidence bundle is prepared for external review.

Inputs: selected artifacts, manifests, redaction/privacy assessment, claim list, known limitations, package schemas, source-data references, and review state.

Normal path:

1. Confirm artifacts are intended for publication.
2. Check that generated data and protected inputs are not accidentally committed or exposed.
3. Verify public evidence supports independent inspection without leaking protected data and that manifests use controlled schemas, canonical bytes, domain-separated hashes, and stated algorithm versions.
4. State limitations, non-claims, uncertainty, and accepted risks.
5. Publish or commit only the intended public artifacts.

Failure or degraded path: protected data exposure, ballot-choice receipt risk, reidentification risk, missing license/provenance, stale claims, or incomplete review blocks publication.

Outputs: public artifact set, evidence index, privacy notes, limitation notes, and review decision.

Handoffs: public reviewer, legal/statistical reviewer, release process, or blocked finding.

Validation evidence: an external reviewer can inspect what was published, reproduce the public artifact manifest, and understand what was deliberately excluded.

## CO-10 Scenario: Execute a Controlled Wave/Pulse

Trigger: maintainers or agents need to finish, fix, or extend the platform without losing traceability.

Inputs: active wave, pulse plan, affected packages, mission/CONOPS/requirement IDs, filed DCRs where applicable, known pitfalls, expected validation commands, review roles, and worktree status.

Normal path:

1. Select a pulse with clear scope and parent IDs.
2. Identify affected crates, docs, schemas, generated artifacts, and package boundaries.
3. Implement only the scoped changes.
4. Run the stated verification commands or record blockers.
5. Update trace rows, evidence pointers, pitfalls, and review findings before claiming completion.

Failure or degraded path: unclear requirement, package-boundary conflict, failing validation, stale docs, unexpected worktree changes, or role-panel blocker stops or narrows the pulse.

Outputs: code/docs/artifact changes, verification evidence, trace updates, review findings, and accepted/deferred risks.

Handoffs: next pulse, review gate, release process, DCR closure, or requirements/design change control.

Validation evidence: a future maintainer can determine why the change was made, what it touched, what evidence was run, and what remains open; each accepted requirement or pitfall has a failing test, validation fixture, or explicit review gate that would catch regression.

## CO-11 Operational Assumptions

- Users run workflows locally from the repo root unless a specific command says otherwise.
- Large raw data and run outputs remain gitignored unless explicitly promoted to public evidence.
- Rust crates are the primary implementation surface; Python scripts, LaTeX sources, docs, configs, and generated reports remain part of the evidence system.
- CLI output, package schemas, file formats, and public docs are controlled interfaces once referenced by requirements.
- Every high-stakes output may be useful evidence, but no output is automatically a legal, statistical, or certification conclusion.
- Missing evidence is recorded as a gap instead of being papered over with success-shaped language.
- Wave/pulse execution remains the operating model for future implementation work.

## CO-12 Open Questions

| ID | Question | Why It Matters | Disposition |
|---|---|---|---|
| OQ-001 | Which VTRACE package specs should become first-class controlled interfaces: RPLAN, RCOUNT, RCTX, RHIST, BISECT configs, or all of them together? | Determines the first requirements and interface-control scope. | Resolved by requirements baseline: BISECT configs/CLI surfaces, RPLAN, RCOUNT, RCTX, RHIST, public verification artifacts, research claim/evidence records, and wave/pulse execution records are controlled interfaces. |
| OQ-002 | Which public verification artifacts are intended to be committed versus generated on demand? | Affects source custody, privacy review, and release evidence. | Decide during verification/validation planning. |
| OQ-003 | Which current papers have complete evidence packages versus partial or historical evidence? | Prevents research claims from outrunning evidence. | Decide during trace and review stages. |
| OQ-004 | What is the first wave/pulse that should be governed by the new VTRACE baseline? | Connects adoption to live execution. | Decide after requirements are accepted. |
| OQ-005 | Which release-readiness DCRs are in scope for the next public transition? | Prevents S4 closure from being mistaken for release readiness. | Filed as DCR-001 through DCR-007 in `docs/vtrace/DCRS.md`; select and close the relevant DCRs before S6 transition claims. |
