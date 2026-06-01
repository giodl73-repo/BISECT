# Interfaces

## IF-00 Scope

Repo or feature: BISECT / apportionment workspace at `C:\src\apportionment`.

Stage: S2 Design Baseline accepted.

This artifact expands the accepted public-contract IDs from `docs/vtrace/SPECIFICATION_BASELINE.md`. `IF-001` through `IF-008` are already established baseline contract IDs and are not renumbered here. Details are accepted as S2 interface controls; later verification must still compare docs to current binary/package behavior.

## IF-01 Interface Inventory

| ID | Interface | Type | Owner | Consumers | Compatibility Rule | Verification | Status |
|---|---|---|---|---|---|---|---|
| IF-001 | BISECT CLI commands, flags, config labels, and output paths | CLI / config / file | BISECT CLI owners | Analysts, researchers, reports, wave/pulse workflows | Compatibility is not claimed until command/help/output inspection records current behavior; after S2 lock, breaking CLI changes require explicit break disposition and migration guidance. | CLI help/doc inspection, smoke run, report verification. | accepted detail |
| IF-002 | BISECT YAML/TOML/JSON configs and generated run manifests | config / schema / file | BISECT core/report owners | Build/analyze/report/verify, trace, research evidence | Version/schema identity must be recorded before compatibility claims; version unknown is not compatibility. | Schema/config inspection and replay demonstration. | accepted detail |
| IF-003 | RPLAN package manifests, canonical bytes, hashes, geography references, and audit results | package / schema / CLI | RPLAN owners / LEDGER | Package auditors, BISECT reports, external exchange | Canonicalization/hash algorithm changes require explicit versioning; version unknown is an audit finding. | Package audit test/demo. | accepted detail |
| IF-004 | RCOUNT package manifests, accounting records, reconciliation ledgers, lifecycle status, and aggregation outputs | package / schema / CLI | RCOUNT owners / CANVASS / TALLY | Election auditors, researchers, district aggregation workflows | Election accounting semantics must not be flattened; source adapter versions and parser warnings are part of the contract. | Count audit test/review. | accepted detail |
| IF-005 | RCTX/RHIST source manifests, crosswalks, unit lineage, and validation status | package / schema / CLI | RCTX/RHIST owners / CONTOUR | BISECT, RCOUNT, RPLAN, research workflows | Source lineage and confidence/status vocabulary remain explicit; version unknown blocks strong compatibility claims. | Context validation fixture or inspection. | accepted detail |
| IF-006 | Public reports, dashboards, papers, evidence indexes, and claim reviews | docs / report / dashboard / evidence | Research/report owners / DATUM / SCALE / COMMONS / VAULT | Public reviewers, legal analysts, researchers | Published claims carry evidence status and limitations; stale claims require correction or gap status. | Claim review finding and evidence pointer. | accepted detail |
| IF-007 | VTRACE docs, trace rows, review findings, and wave/pulse records | docs / process | Maintainers / BENCHMARK / TRENCH | Maintainers, agents, role panels | Accepted IDs are stable; retired/superseded IDs require trace-preserving disposition. | Trace-matrix review. | accepted detail |
| IF-008 | External standard mappings and interchange claims, including GeoJSON, shapefile, Census/TIGER, NIST/CDF where applicable, GerryChain, DRA, PlanScore, and package-family adapters | external format / adapter / schema | Package owners / LEDGER | Interop consumers, public evidence, research comparisons | Claims name external format/schema/tool version, required field/geometry semantics, and known gaps. | Interoperability inspection or fixture. | accepted detail |

## IF-02 Interface Details

### IF-001: BISECT CLI commands, flags, config labels, and output paths

Purpose: Provide the user-facing command surface for fetching data, building plans, analyzing outputs, generating reports, verifying chains, and invoking algorithm/package workflows.

Inputs: command name, flags, `configs/{label}.yml`, data/cache paths, package paths, optional source manifests, and environment/toolchain assumptions.

Outputs: data cache entries, run directories, analysis outputs, reports, maps, package/audit outputs, logs, and verification status.

Errors: missing data/config, invalid state/year/label, incompatible algorithm flags, METIS/tool failure, disconnected graph, population-balance failure, output-path conflict, incomplete provenance, or legal-readiness gap must surface as explicit failure/partial/blocked status.

Versioning or compatibility: Current docs indicate both legacy commands (`state`, `states`, `run`) and label pipeline commands (`build`, `label-analyze`, `label-report`, `label-verify`). Interface verification must distinguish stable, legacy, experimental, and version-unknown commands before compatibility claims. After a command is accepted as stable, a breaking change must carry explicit break disposition and migration notes.

Evidence: `README.md`, `docs/BISECT_CLI.md`, `docs/concepts/label-pipeline.md`, `docs/concepts/pipeline-stages.md`, CLI help snapshots planned.

WP-002 note: CLI build/help verification depends on local cross-repo dependencies resolving through `repo-map.toml` and the generated `.cargo/config.toml` Cargo patches; unmapped local checkout paths are interface findings, not missing-tool passes.

WP-002 current snapshot: `cargo run -q -p bisect-cli -- --help` loads and exposes both legacy command surfaces (`run`, `state`, `states`, `analyze`, `report`, `export`, `import`, `verify`) and label-pipeline command surfaces (`config`, `build`, `ls`, `show`, `mv`, `label-verify`, `label-analyze`, `label-report`, `label-import`, `label-compare`, `plan`). Compatibility status is therefore mixed by command family: legacy commands remain available, label-pipeline commands are the current reproducible-run path, and command-level stability must be asserted per command rather than inferred from the binary existing.

### IF-002: Configs and run manifests

Purpose: Define reproducible BISECT runs and carry provenance through the SHA-256 audit chain.

Inputs: `configs/{label}.yml`, year/state/chamber/scope, algorithm structure, weights, search strategy, population tolerance, workers, METIS engine, seed/search parameters, and source-data references.

Outputs: run manifests, build indexes, assignment hashes, analysis/report hashes, replay metadata, output paths, and verification results. WP-003 current snapshot: new label build indexes include label, year, `config_path`, `config_sha256`, command invocation, output directory, git commit, METIS engine, algorithm, state list, state result pointers, and created-at timestamp; per-state manifests include data source hashes, adjacency hash, seed, partition mode, package version, and executable SHA-256.

Errors: missing required field, unknown algorithm value, incompatible flags, version-unknown schema, missing source custody, or nondeterministic search metadata gap.

Versioning or compatibility: Schema identity is required for compatibility claims. If the current schema is informal, S2 records it as version unknown until verification.

Evidence: `docs/concepts/three-layer-compositor.md`, `docs/concepts/label-pipeline.md`, `docs/vtrace/SPECIFICATION_BASELINE.md`.

WP-006 current snapshot: `data/manifest.json` is an embedded public source manifest consumed by the fetch path and now points to the active `giodl73-repo/BISECT` repository. Raw/source data under `data/`, generated runs under `runs/`/`outputs/`, analysis/report outputs, local `.cargo/config.toml` patches, and geospatial exports remain non-public by default unless promoted through custody review.

### IF-003: RPLAN packages

Purpose: Exchange district plans with manifests, geography references, canonical bytes, hashes, schema versions, and audit findings.

Inputs: district assignments, geography/context references, source metadata, package manifest, schema/canonicalization version, optional report artifacts.

Outputs: audit result, hash/certificate status, schema status, geography provenance status, and explicit findings.

Errors: hash mismatch, canonicalization ambiguity, missing source metadata, invalid district assignment, schema incompatibility, stale geography, unsupported external compatibility claim.

Versioning or compatibility: Canonical bytes, hash algorithm, domain separation, completeness rules, and schema fields are controlled. Version unknown is an audit finding.

Evidence: `crates/rplan-*`, RPLAN package tests/fixtures planned, SPEC-007.

WP-002 current snapshot: `rplan-core` declares `DISTRICT_PLAN_SCHEMA_VERSION = "district-plan-v1"` and `RCTX_VERSION = "0.1"`; `rplan-io` declares `RPLAN_V02 = "0.2"` and supports `0.2` plus `0.1` compatibility input. The accepted schema source is `docs/specs/2026-05-10-rplan-v0.2-schema.md`; fixture-level audit closure was deferred from WP-002 to WP-004 rather than treated as a WP-002 compatibility pass.

WP-004 current snapshot: RPLAN core/IO/audit/CLI tests pass, and public U.20 plan-audit certificate examples plus negative fixtures exercise contextual certificates, plan/context hash binding, unit-order mismatch, stale context, profile mismatch, and missing-input statuses. This is evidence for package audit semantics, not legal certification.

### IF-004: RCOUNT packages

Purpose: Preserve and audit election-count evidence without displacing official certification authority.

Inputs: source exports, contest definitions, ballot/CVR records where available, ballot cards, duplicated/provisional ballots, batches, central-count batches, vote centers, precincts, write-ins, overvotes, undervotes, reconciliation records, lifecycle status, jurisdiction metadata, parser diagnostics.

Outputs: package audit result, reconciliation status, lifecycle status, district aggregation outputs where applicable, parser warnings, privacy/security disposition, and certification-boundary statement.

Errors: incomplete source exports, ambiguous contest/ballot-card semantics, unsupported vendor format, missing batch/CVR/provisional/duplication data, reconciliation mismatch, protected-data risk, uncertain lifecycle status.

Versioning or compatibility: Source adapter versions, parser diagnostics, lifecycle vocabulary, and semantics are controlled. Changes that flatten or reinterpret accounting concepts are breaking.

Evidence: `crates/rcount-*`, CO-06, SPEC-008.

WP-002 current snapshot: `rcount-core` declares `RCOUNT_VERSION = "0.1-draft"` and domain-separated hash prefixes for source, record, file, package, status event, proof, RLA manifest, and RLA sample records. `rcount-io` exposes package-directory IO and synthetic package helpers; NIST CDF and Rhode Island RLA import errors/guards are present, but source-adapter completeness and jurisdiction replay remain WP-004 evidence.

WP-004 current snapshot: RCOUNT core/IO/audit/district/stats/RHIST bridge and CLI tests pass. The model preserves lifecycle status transitions, batch/reconciliation checks, CVR/RLA/proof privacy guards, RCTX/RHIST references, and district aggregation semantics. Evidence is primarily inline synthetic tests plus CLI verification tests; jurisdiction/vendor replay and promoted public golden fixtures remain deferred.

### IF-005: RCTX/RHIST context and history packages

Purpose: Provide verified or explicitly partial context for joins among census geography, election geography, precincts, districts, and historical units.

Inputs: PL 94-171, TIGER/Line, ACS, state election geography, precincts, districts, derived crosswalks, historical lineage, source manifests, transformation commands.

Outputs: source provenance, crosswalk validation status, unit-history lineage, verified/partial/conjectural/blocked/gap status, known assumptions and limitations.

Errors: missing provenance, changed identifiers, ambiguous splits/merges, invalid geometry, incomplete crosswalks, stale districts, incompatible schema.

Versioning or compatibility: Status vocabulary and source lineage are controlled. Unknown lineage cannot be treated as verified.

Evidence: `crates/rctx-core`, `crates/rhist-*`, CO-07, SPEC-009.

WP-002 current snapshot: `rctx-core` declares `RCTX_VERSION = "0.1"` with package, source, graph, crosswalk, and claim-boundary records; `rhist-core` declares `RHIST_VERSION = "0.1"` with cycle, context, lineage, crosswalk, source, and claim-boundary records. `rhist-io` provides package-directory IO and hash verification; fixture-level lineage/context closure was deferred from WP-002 to WP-004.

WP-004 current snapshot: RCTX and RHIST core/IO/CLI tests pass. The models preserve source indexes, context/unit hashes, crosswalk rational weights and exhaustiveness checks, lineage cardinality, cycle/context consistency, and claim-boundary records. Evidence is sufficient for L1 package semantics, but public interoperability still requires promoted external fixtures or an integrated downstream package scenario.

### IF-006: Public reports, dashboards, papers, evidence indexes, and claim reviews

Purpose: Present inspectable evidence to public reviewers without overstating claims or exposing protected data.

Inputs: reports, maps, analysis tables, dashboards, paper sources/PDFs, evidence bundles, source/custody records, claim classifications, limitations, non-claims.

Outputs: public artifacts with evidence status, assumptions, uncertainty, comparison baseline, political/community-effect scope where relevant, privacy/custody disposition, and review state.

Errors: unsupported or stale headline claim, missing evidence pointer, missing uncertainty/assumptions, protected-data exposure, ballot-choice receipt/coercion channel risk, source-custody gap.

Versioning or compatibility: Public claims are controlled by claim status and evidence pointer, not only artifact filenames. Changed numbers or claims trigger review.

Evidence: `README.md`, `docs/PAPERS.md`, `research/`, dashboards, SPEC-004, SPEC-010.

WP-005 current snapshot: L0 review inspected README headline claims, paper index/review surfaces, algorithm scorecard, legal docs, and high-risk claim patterns. README claims are now bounded for replay completeness, compactness mechanism, default-geographic versus VRA inputs, procedural fairness, runtime dependence, headline metric posture, and legal-thesis status. `docs/PAPERS.md` now states that review/accepted/golden/score labels are internal project status markers, not external peer review, legal certification, or official election certification.

### IF-007: VTRACE docs, trace rows, review findings, and wave/pulse records

Purpose: Keep mission, CONOPS, requirements, specs, interfaces, design, trace, verification, validation, role review, and execution records connected.

Inputs: accepted IDs, stage artifacts, role findings, pulse metadata, validation commands, risk dispositions, pitfall status.

Outputs: trace matrix rows, review findings, accepted/deferred risks, verification/validation status, wave/pulse completion evidence.

Errors: orphan accepted requirement, undocumented ID change, missing validation command, untriaged review finding, completed pulse without risk/verification disposition.

Versioning or compatibility: Accepted IDs are stable. Superseded, retired, split, or merged IDs must keep trace-preserving disposition.

Evidence: `docs/vtrace/`, `context/waves/PHASES.md`, `repo-map.toml`, and `TRACE.md`.

WP-007 current snapshot: `context/waves/PHASES.md` is the live wave/pulse convention file. It now requires VTRACE-governed pulses to name parent IDs, validation level, affected package boundaries, verification state, role gates, risk/pitfall disposition, and public/custody effects. Existing archived pulses were inspected as historical examples of role and validation-command records, not retroactively upgraded to full VTRACE-governed closure evidence.

### IF-008: External standard mappings and interchange claims

Purpose: Make import/export and interoperability claims explicit, versioned, and bounded.

Inputs: external data formats/tools/schemas such as GeoJSON, shapefile, Census/TIGER, NIST/CDF where applicable, GerryChain, DRA, PlanScore, state source exports, and package-family adapters.

Outputs: named standard/tool/schema version, required fields, coordinate/reference-system assumptions, geometry semantics, mapping gaps, compatibility status.

Errors: schema mismatch, missing required fields, coordinate/reference-system ambiguity, geometry invalidity, external version unknown, unsupported adapter path.

Versioning or compatibility: A partial adapter cannot imply universal compatibility. Version unknown blocks strong compatibility claims.

Evidence: `docs/vtrace/IMPORT_COMPATIBILITY.md`, `docs/fixtures/import-label/`, external adapter docs/fixtures, SPEC-002, SPEC-007, SPEC-008, SPEC-009.

WP-002 current snapshot: implemented BISECT external plan adapters include GeoJSON, GerryChain v2.3 JSON, DRA-style CSV detection/import paths, CSV assignment import, RPLAN `0.2`/`0.1` label import through `rplan-io`, direct shapefile label import through DBF `GEOID`/district fields, and GeoJSON/GerryChain/CSV/reproducibility-package export paths. DCR-005 centralizes status, field assumptions, failure modes, and fixture evidence in `docs/vtrace/IMPORT_COMPATIBILITY.md`. DRA/PlanScore/NIST/CDF compatibility claims remain bounded unless a named command, fixture, or package adapter proves the specific format/version; shapefile claims are limited to tract/district assignment attributes, not arbitrary geometry transformation.

## IF-03 Cross-Interface Compatibility Rules

| Rule ID | Rule | Applies To | Status |
|---|---|---|---|
| IF-RULE-001 | Version unknown is a finding, not compatibility. | IF-001..IF-008 | accepted |
| IF-RULE-002 | A passing hash or package audit does not by itself establish legal readiness, election certification, or public claim readiness. | IF-003, IF-004, IF-006 | accepted |
| IF-RULE-003 | Generated/public artifacts require custody/privacy disposition before publication or commit. | IF-002, IF-006, IF-008 | accepted |
| IF-RULE-004 | Command, schema, canonicalization, default, and semantic changes require migration notes or explicit break disposition. | IF-001..IF-005, IF-008 | accepted |
| IF-RULE-005 | Public claims must identify evidence posture/status and limitations even when the underlying computation succeeded. | IF-006 | accepted |

## IF-04 Open Questions

| ID | Question | Impact | Disposition |
|---|---|---|---|
| IF-Q-001 | Which CLI command namespace is canonical for current production docs: legacy `state/states/run` or label pipeline `build/label-*`? | Could confuse non-author workflows and compatibility claims. | WP-002 records both as current binary surfaces; label pipeline is the reproducible-run path, while legacy command compatibility remains command-specific. |
| IF-Q-002 | Are all RPLAN/RCOUNT/RCTX/RHIST schemas sufficiently versioned today? | Package compatibility claims may be premature. | WP-004 verifies current schema/audit tests; DCR-001 controls public golden fixture promotion. |
| IF-Q-003 | Which external adapters are implemented versus claimed in docs/papers? | Interoperability claims could overreach. | WP-002 records implemented/stubbed import/export surfaces; DCR-005 is closed at L1 through `docs/vtrace/IMPORT_COMPATIBILITY.md`; DCR-001 remains the fixture-promotion gate for stronger public claims. |
| IF-Q-004 | What artifacts need durable evidence indexes versus transient development outputs? | Publication policy and trace coverage may be incomplete. | DCR-004 now has the L1 `BISECT-EVIDENCE-PACKAGE-v1` baseline in `docs/vtrace/EVIDENCE_PACKAGE_CONTRACT.md`; L2 closure requires review against a concrete public bundle. |

## IF-05 Interface Gate

Decision: accepted for S2 Design Baseline.

S2 lock disposition:

- [x] `IF-001` through `IF-008` retain accepted IDs and have owner, consumer, compatibility, error, and evidence detail.
- [x] Version-unknown items are recorded instead of implied compatible.
- [x] CLI/doc drift is either resolved or explicitly listed as a verification target.
- [x] External compatibility claims are bounded by named standards/tools/schemas or marked unknown.
- [x] Role review findings are fixed, deferred, or accepted as risk.
