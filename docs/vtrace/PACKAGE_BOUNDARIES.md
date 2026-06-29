# Package Boundaries

## PB-00 Scope

Repo or feature: BISECT / apportionment workspace at `C:\src\apportionment`.

Stage: S2 Design Baseline accepted.

This artifact inventories package, language, generated-artifact, and responsibility boundaries for the VTRACE baseline. Boundary IDs are accepted for S2 trace use; later changes require trace-preserving disposition.

## PB-01 Boundary Inventory

| ID | Boundary Unit | Language / Toolchain | Owner | Responsibility | Public Interfaces | Downstream Consumers | Status |
|---|---|---|---|---|---|---|---|
| PKG-001 | `docs/vtrace/`, VTRACE stage artifacts | docs | Maintainers | Requirements/spec/design/trace/verification/validation/review control plane. | IF-007 | All future waves, maintainers, reviewers | accepted |
| PKG-002 | `data/`, `runs/`, `analysis/`, `reports/`, generated/publication custody rules | mixed / generated | Maintainers / VAULT | Source custody, generated-output hygiene, protected-data disposition, public-evidence promotion. | IF-002, IF-006, IF-008 | BISECT, research, public artifacts | accepted |
| PKG-003 | `crates/rgraph-core`, `crates/rmath-core`, `crates/rstat-core`, `crates/ropt-core` | Rust | Shared-kernel owners | Deterministic reusable graph, math, statistics, and optimization primitives. | internal APIs | BISECT, RPLAN/RCOUNT support, research algorithms | accepted |
| PKG-004 | `crates/rplan-core`, `crates/rplan-io`, `crates/rplan-audit`, `crates/rplan-cli` | Rust | RPLAN owners / LEDGER | District-plan package schema, IO, audit, canonicalization, hashes, and CLI. | IF-003 | BISECT reports, package audits, downstream exchange | accepted |
| PKG-005 | `crates/rcount-core`, `crates/rcount-io`, `crates/rcount-audit`, `crates/rcount-stats`, `crates/rcount-district`, `crates/rcount-cli`, `crates/rcount-rhist` | Rust | RCOUNT owners / CANVASS / TALLY | Election-count packages, reconciliation, lifecycle, district aggregation, certification boundary. | IF-004 | Election audits, district aggregation, research/public evidence | accepted |
| PKG-006 | `crates/rctx-core`, `crates/rhist-core`, `crates/rhist-io`, `crates/rhist-cli` | Rust | RCTX/RHIST owners / CONTOUR | Context/crosswalk packages and historical unit lineage. | IF-005 | BISECT, RCOUNT aggregation, RPLAN/RHIST evidence | accepted |
| PKG-007 | `crates/bisect-core`, `crates/bisect-data`, `crates/bisect-apportion` | Rust | BISECT core owners / MERIDIAN | Core graph/population/data loading and redistricting construction primitives. | IF-001, IF-002 | BISECT CLI, analysis, reports, search algorithms | accepted |
| PKG-008 | `crates/bisect-ensemble`, `crates/bisect-smc`, `crates/bisect-pareto`, `crates/bisect-multiscale`, `crates/bisect-local-search`, `crates/bisect-clustering`, `crates/bisect-flow`, `crates/bisect-ilp`, `crates/bisect-column` | Rust | Algorithm-family owners / MERIDIAN / SCALE | Search, ensemble, Pareto, exact, clustering, flow, and improvement families. | IF-001, IF-003, IF-006 | BISECT CLI, research claims, package outputs | accepted |
| PKG-009 | `crates/bisect-analysis`, `crates/bisect-map`, `crates/bisect-report` | Rust | Analysis/report owners / DATUM / COMMONS | Metrics, maps, report manifests, evidence presentation, SHA chain outputs. | IF-001, IF-006 | Reports, dashboards, papers, public reviewers | accepted |
| PKG-010 | `crates/bisect-cli`, `crates/bisect-tui`, `crates/bisect-web` | Rust | CLI/UI owners | User-facing BISECT commands, interactive views, and orchestration. Local cross-repo dependencies must be recorded in `repo-map.toml`. | IF-001, IF-006 | Analysts, researchers, maintainers, reviewers | accepted |
| PKG-011 | `scripts/`, `python/bisect_py` | Python | Data/research support owners | Data acquisition, experiments, validation, dashboards, figures, and legacy/support workflows. | IF-006, IF-008 where published | Research, docs, generated dashboards | accepted |
| PKG-012 | `README.md`, `docs/`, `docs/papers/`, `research/`, `docs/legal/` | docs / LaTeX | Research/docs/legal owners | Public explanation, papers, quickstarts, legal drafts, claim surfaces. | IF-006, IF-008 | Public reviewers, researchers, legal analysts | accepted |
| PKG-013 | `context/waves/`, wave/pulse/review materials | docs | Wave/pulse owners / BENCHMARK / TRENCH | Controlled execution and review records. | IF-007 | Maintainers and future agents | accepted |
| PKG-014 | `archive/python-pipeline-final/` and other archives | archived code/docs | Maintainers | Sealed forensic reference only. | none unless explicitly promoted | Historical comparison | accepted |

## PB-02 Dependency Direction

The directions below are the intended S2 boundary constraints for active and target package families. Until Cargo and script dependency inspection is attached in verification, each row is a design assertion pending verification; any current-code exception must be recorded instead of silently treated as compliant.

| From | To | Allowed? | Rationale | Verification |
|---|---|---|---|---|
| CLI/UI (`PKG-010`) | core/data/algorithm/analysis/report/package crates (`PKG-003`..`PKG-009`) | yes | CLI shells orchestrate lower layers. | Cargo dependency inspection and CLI tests. |
| Shared kernels (`PKG-003`) | BISECT/package/application crates (`PKG-004`..`PKG-010`) | no | Kernels must remain policy-neutral and reusable. | Cargo dependency inspection. |
| RPLAN (`PKG-004`) | BISECT algorithm crates (`PKG-007`, `PKG-008`) | no, except planned adapter decisions | Plan packages define neutral plan identity; construction algorithms may emit RPLAN but RPLAN should not depend on them. | Cargo dependency inspection. |
| RCOUNT (`PKG-005`) | RCTX/RHIST (`PKG-006`) | yes for aggregation/history adapters | Count-to-district and history workflows need verified context links. | Cargo dependency inspection and package tests. |
| RCTX/RHIST (`PKG-006`) | RCOUNT or BISECT construction (`PKG-005`, `PKG-007`, `PKG-008`) | no by default | Context/history should not depend on downstream analytic consumers. | Cargo dependency inspection. |
| Analysis/report (`PKG-009`) | BISECT core/data/package crates (`PKG-004`, `PKG-007`) | yes | Reports summarize algorithm outputs and package metadata. | Cargo dependency inspection and report tests. |
| Python support (`PKG-011`) | Rust CLI outputs and data directories | yes as support/read path | Scripts may consume outputs for experiments/dashboard generation. | Script docs/review. |
| Rust production crates | Python support scripts | no | Production Rust binary must not depend on Python runtime for core work. | Build dependency inspection. |
| BISECT workspace | External local checkouts in `repo-map.toml` | yes, through generated Cargo patches or documented source references | The tracker layout relocates sibling repositories; old hard-coded sibling paths are findings unless mapped. | Repo-map inspection, generated Cargo config check, and Cargo load/build check. |
| Public docs/research (`PKG-012`) | Evidence outputs from all controlled packages | yes by reference | Claims must cite evidence, commands, packages, tables, figures, or review state. | Claim review. |
| Archives (`PKG-014`) | Active code | no | Archived references must not silently drive current behavior. | Inspection. |

## PB-03 Boundary Rules

| Boundary ID | Allowed Changes | Forbidden Changes | Change-Control Trigger |
|---|---|---|---|
| PKG-001 | Add trace/review/verification rows, correct accepted artifacts through controlled revision. | Renumber locked IDs without trace-preserving disposition. | Any accepted `M-*`, `CO-*`, `REQ-*`, `SPEC-*`, `IF-*`, `ARCH-*`, `PKG-*`, or `DES-*` change. |
| PKG-002 | Add custody records, generated-artifact policies, source pointers, redaction dispositions. | Commit protected inputs, unintended generated outputs, or public artifacts without custody review. | New public artifact class, source family, release policy, or protected-data risk. |
| PKG-003 | Add deterministic pure helpers with tests. | Add public-policy/legal/election-certification claims or high-level app dependencies. | API break or nondeterministic behavior change. |
| PKG-004 | Evolve package schemas/canonicalization with versioning and migration. | Treat package integrity as legal plan certification. | Schema, canonicalization, hash, completeness, or CLI contract change. |
| PKG-005 | Add adapters/statistics while preserving election accounting semantics. | Flatten ballots/ballot cards/batches/CVR/lifecycle concepts or replace official certification authority. | Source adapter, lifecycle vocabulary, reconciliation, privacy, or aggregation change. |
| PKG-006 | Add source manifests, crosswalk models, lineage validators. | Treat unknown lineage as verified. | Source-contract, lineage, crosswalk, or status vocabulary change. |
| PKG-007 | Add construction primitives and source loaders with provenance. | Hide seeds/search metadata or source custody needed for replay. | Algorithm input/output schema, data source, or reproducibility change. |
| PKG-008 | Add algorithms/search/improvement methods with explicit evidence status. | Present unreviewed research variants as production/legal defaults. | New algorithm family, objective, search strategy, or selection rule. |
| PKG-009 | Add metrics/maps/reports with evidence pointers and limitations. | Publish unsupported headline claims or omit uncertainty/non-claims. | New metric, report schema, public dashboard, or headline number. |
| PKG-010 | Add commands/flags with help and compatibility notes. | Break command semantics silently. | CLI command/flag/default/output-path/schema change. |
| PKG-011 | Add research/acquisition/support scripts with documented inputs/outputs. | Promote script outputs to public evidence without review. | Script becomes publication input or controlled interface. |
| PKG-012 | Add or revise docs/papers with claim status. | State legal/election/public claims without evidence posture and limitations. | New public claim, paper release, legal claim, or README headline change. |
| PKG-013 | Add pulses/reviews with requirements and validation. | Close work without validation/risk/review status. | New wave, active pulse, or accepted review finding. |
| PKG-014 | Read archived code for forensic comparison. | Modify sealed archive or import it into active production path without decision. | Any planned archive modification or dependency. |

## PB-04 Language Tailoring

| Boundary ID | Code Rigor Profile | L0 | L1 | L2 |
|---|---|---|---|---|
| PKG-001 | docs-only / trace | artifact inspection | cross-artifact trace review | role panel review |
| PKG-002 | custody / generated-data | path/gitignore inspection | source-custody fixture | public-release review |
| PKG-003 | Rust kernel | inline unit tests | crate integration tests | consumer regression |
| PKG-004 | Rust package | schema/canonical unit tests | golden package audit | interoperability package review |
| PKG-005 | Rust package / election semantics | parser/model unit tests | count audit fixture | jurisdiction/source replay |
| PKG-006 | Rust package / context-history | lineage/status unit tests | crosswalk fixture | real-source context review |
| PKG-007 | Rust production | unit tests for invariants | state/synthetic integration | real-data run/replay |
| PKG-008 | Rust research/algorithm | algorithm unit tests | synthetic/fixture comparison | real-state evidence package |
| PKG-009 | Rust report/analysis | metric unit tests | fixture report generation | public artifact review |
| PKG-010 | Rust CLI/UI | CLI parser/unit tests | smoke command tests | workflow demonstration |
| PKG-011 | Python support | unit/script checks where present | fixture run | evidence-review if public |
| PKG-012 | docs/LaTeX | link/claim inspection | paper evidence review | hostile/public review |
| PKG-013 | docs/process | pulse checklist inspection | trace review | role panel review |
| PKG-014 | archive | inspection only | not applicable | not applicable |

## PB-05 Generated Artifacts

Artifact disposition categories:

| Category | Meaning | Publication / Commit Rule |
|---|---|---|
| public | Intended for public review or release after evidence and claim review. | May be published only with source/evidence pointers, limitations, and custody disposition. |
| restricted | Contains protected, licensed, sensitive, or otherwise non-public inputs/intermediates. | Must not be committed or published without explicit VAULT review and redaction/disposition. |
| local-only | Developer or analyst workspace material needed for a local run. | Must remain outside public artifacts unless promoted through custody review. |
| generated-transient | Rebuildable intermediate output, cache, figure, table, map, or report draft. | Default is not committed; promotion requires source-of-truth and verification disposition. |
| archived-reference | Historical sealed reference kept for comparison. | Read-only unless a boundary decision explicitly promotes or revises it. |

| Artifact | Source Of Truth | Regeneration Command | Verification |
|---|---|---|---|
| `data/{year}/` raw/source cache | External source + custody record | `bisect fetch --year <year>` or documented source adapter | Source URL/version/hash/custody inspection. |
| `outputs/data/{year}/adjacency/*.adj.bin` | TIGER/Line geometry and adjacency builder/release cache | `bisect fetch --year <year> --release` or adjacency build workflow | Release/source hash and graph connectivity validation. |
| `configs/{label}.yml` | Human-edited config | not generated | Config review and SHA anchor. |
| `runs/{label}/{year}/...` | Config + data + BISECT build command | `bisect build <label> --year <year>` | Run manifest, assignments hash, replay metadata. |
| `analysis/{label}/{year}/...` | Run outputs + analysis command | `bisect label-analyze <label> --year <year> --types ...` | Analysis manifest and metric-specific checks. |
| `reports/{label}/{year}/...` | Analysis outputs + report command | `bisect label-report <label> --year <year> --format ...` | Report manifest and claim review. |
| Dashboards / docs site | Reports, analysis outputs, scripts | `scripts/web/generate_*.py` or docs-site workflow | Public-artifact custody and claim review. |
| `docs/papers/*.pdf` | LaTeX sources in `research/` | LaTeX compile workflow | Paper evidence review and source/PDF alignment. |
| RPLAN/RCOUNT/RCTX/RHIST packages | Package source files and schemas | package-family CLI/import/export command | Package audit fixture and manifest/hash review. |
| VTRACE artifacts | Accepted stage inputs and role decisions | manual controlled edit | Trace review and `.roles` findings. |

METIS and other external-engine outputs require engine disclosure in the relevant run/package evidence record: engine family (`c-ffi`, `bisect-metis`, `gpmetis`, or version-unknown), crate/tool source (`metis-core`, `metis`, bundled/external executable, or other), version/hash where available, feature flags, and seed/search replay metadata.

WP-006 disposition: `.gitignore` establishes the default custody boundary: `/data/`, `outputs/`, `reports/`, build outputs, local Cargo patches, geospatial exports, and logs are not publishable by default. Tracked exceptions are policy/source-pointer files (`data/README.md`, `data/manifest.json`, `data/location_policy.json`), public documentation artifacts under `docs/`, selected research figures under `research/`, and legacy/policy pointers such as `outputs/README.md`. Local `data/`, `outputs/`, and `reports/` contents remain local-only or generated-transient until a release-specific VAULT review promotes selected artifacts with source pointers, privacy disposition, and claim status. `archive/python-pipeline-final/` remains archived-reference/read-only.

## PB-06 Repository Map

`repo-map.toml` is the authoritative local checkout map for this repository's multi-repo development dependencies. It records the current tracker layout for BISECT, FLETCH, METIS-CORE, SLICE, RCOUNT, RPLAN, and VTRACE source material. Cargo does not consume this file directly; `tools/repo_map.py` generates the local `.cargo/config.toml` `[patch]` entries from it. Any local dependency path that reaches outside this repository must either be generated from `repo-map.toml` or be recorded as a WP-002 boundary finding.

WP-002 disposition: the active Rust workspace exposes no remaining hard-coded cross-repo Cargo path dependency in the BISECT CLI path inspected for this review; FLETCH is a canonical git workspace dependency overridden locally by generated Cargo patches. Python support remains a support/research boundary (`PKG-011`) and must not be treated as a production Rust dependency. RPLAN label-plan import is implemented through `rplan-io` and direct shapefile label-plan import is implemented through the `shapefile` crate in `bisect-cli`; both are tested as bounded adapters rather than universal external-format compatibility claims.

## PB-07 Open Boundary Questions

| ID | Question | Impact | Disposition |
|---|---|---|---|
| PB-Q-001 | Which current CLI commands/flags are stable public contracts versus experimental surfaces? | Interface review may find doc/binary drift. | WP-002 records current legacy and label-pipeline surfaces; command-level stability still requires command-specific evidence. |
| PB-Q-002 | Which Python scripts should be controlled interfaces because they generate public evidence? | Some generated dashboards/papers may depend on scripts without formal status. | Keep Python as support/research until a script output is promoted by WP-005 claim/evidence review or the DCR-004 evidence-package contract. |
| PB-Q-003 | Which package schemas are fully specified and which are version-unknown? | Compatibility and audit claims may be overbroad. | WP-004 verifies current package-family tests and records version/draft status; DCR-001 controls public fixture promotion and integrated package audit evidence; DCR-005 records current matrix status. |
| PB-Q-004 | What is the final publication policy for reports, dashboards, package artifacts, and PDFs? | Generated artifacts may be committed or withheld inconsistently. | WP-006 sets repository defaults and current exceptions; DCR-004 defines `BISECT-EVIDENCE-PACKAGE-v1`, while release-specific VAULT review controls selected bundles before publication. |
| PB-Q-005 | How do future waves know when VTRACE closure evidence is required? | Pulses could inherit old completion habits and close without trace, validation, risk, or custody disposition. | WP-007 updates `context/waves/PHASES.md`; VTRACE-governed pulses now require ID, boundary, validation, role, risk/pitfall, and public/custody closure fields. |

## PB-08 Boundary Gate

Decision: accepted for S2 Design Baseline.

S2 lock disposition:

- [x] Boundary IDs are stable and complete enough for trace matrix use.
- [x] Dependency directions are recorded as intended boundaries and either verified against Cargo/scripts or paired with explicit current-code exceptions.
- [x] Public interfaces are tied to accepted `IF-*` contracts.
- [x] Generated artifacts have source-of-truth and verification disposition.
- [x] Open boundary questions are accepted as risks, deferred, or converted to work.
