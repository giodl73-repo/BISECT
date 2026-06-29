# Architecture

## AR-00 Scope

Repo or feature: BISECT / apportionment workspace at `C:\src\apportionment`.

Stage: S2 Design Baseline accepted.

This artifact describes the target architecture that satisfies the accepted Mission, CONOPS, Requirements, and Specification Baseline. It is a mixed current/target baseline for an existing repo: current crate families and docs are treated as evidence, while VTRACE controls, package boundaries, and verification hooks are accepted as S2 design controls pending later verification.

## AR-01 Architecture Summary

BISECT is a civic evidence platform centered on deterministic redistricting, plan/package audit, election-count/context package families, and research/public evidence review. The architecture separates:

1. Source and custody surfaces (`data/`, external sources, release caches, generated outputs).
2. Shared deterministic kernels (`rgraph-core`, `rmath-core`, `rstat-core`, `ropt-core`).
3. Package contracts (`rplan-*`, `rcount-*`, `rctx-*`, `rhist-*`).
4. Redistricting construction/search/analysis/reporting (`bisect-*`).
5. Human-facing controls (`docs/`, `research/`, `context/waves/`, `docs/vtrace/`, filed DCRs).

The central architectural rule is that high-stakes claims flow from explicit inputs through controlled interfaces to evidence artifacts with review status. Algorithmic run completion, package integrity, legal readiness, election certification, and public claim readiness are separate gates.

## AR-02 Components

| Component | Boundary ID | Responsibility | Requirement IDs | Interfaces | Evidence | Status |
|---|---|---|---|---|---|---|
| VTRACE control plane | PKG-001 | Mission, CONOPS, requirements, specs, trace, DCRs, verification, validation, review, and stage gates. | REQ-001, REQ-002, REQ-028, REQ-029, REQ-034, REQ-035, REQ-036 | IF-007 | `docs/vtrace/`, `context/waves/PHASES.md` | accepted |
| Source custody and generated-data boundary | PKG-002 | Keep raw data, protected inputs, large outputs, generated artifacts, and publishable evidence under explicit custody disposition. | REQ-004, REQ-020, REQ-027, REQ-033 | IF-002, IF-006, IF-008 | `.gitignore`, `data/{year}/`, `runs/`, `analysis/`, `reports/`, source manifests | accepted |
| Shared deterministic kernels | PKG-003 | Provide graph, math, statistics, and optimization primitives without public-policy claims. | REQ-029, REQ-031, REQ-037 | internal APIs | `crates/rgraph-core`, `crates/rmath-core`, `crates/rstat-core`, `crates/ropt-core` | accepted |
| RPLAN package family | PKG-004 | Plan identity, stable IO, audit certificates, canonicalization, hashes, schema compatibility, and plan package CLI. | REQ-003, REQ-013, REQ-014, REQ-015, REQ-031 | IF-003 | `crates/rplan-core`, `crates/rplan-io`, `crates/rplan-audit`, `crates/rplan-cli` | accepted |
| RCOUNT package family | PKG-005 | Election-count schema, IO, audit, reconciliation, lifecycle status, district aggregation, and certification-boundary statements. | REQ-016, REQ-017, REQ-018, REQ-019, REQ-020, REQ-023 | IF-004 | `crates/rcount-core`, `crates/rcount-io`, `crates/rcount-audit`, `crates/rcount-stats`, `crates/rcount-district`, `crates/rcount-cli`, `crates/rcount-rhist` | accepted |
| RCTX/RHIST package families | PKG-006 | Context packages, geography provenance, crosswalk status, and historical unit lineage. | REQ-021, REQ-022, REQ-023, REQ-033 | IF-005 | `crates/rctx-core`, `crates/rhist-core`, `crates/rhist-io`, `crates/rhist-cli` | accepted |
| BISECT core/data/construction stack | PKG-007 | Load data, build tract graphs, construct district plans, maintain algorithmic invariants, and emit run metadata. | REQ-007, REQ-008, REQ-009, REQ-012 | IF-001, IF-002 | `crates/bisect-core`, `crates/bisect-data`, `crates/bisect-apportion`, `configs/`, `runs/` | accepted |
| BISECT algorithm-family stack | PKG-008 | Provide search, ensemble, Pareto, exact, clustering, flow, and improvement families with explicit evidence status. | REQ-007, REQ-008, REQ-009, REQ-012, REQ-024 | IF-001, IF-003, IF-006 | `crates/bisect-ensemble`, `crates/bisect-smc`, `crates/bisect-pareto`, `crates/bisect-multiscale`, `crates/bisect-local-search`, `crates/bisect-clustering`, `crates/bisect-flow`, `crates/bisect-ilp`, `crates/bisect-column` | accepted |
| BISECT analysis/report/map stack | PKG-009 | Compute metrics, generate maps/reports, preserve SHA chain, and expose inspectable outputs. | REQ-005, REQ-006, REQ-012, REQ-024, REQ-026, REQ-032 | IF-001, IF-006 | `crates/bisect-analysis`, `crates/bisect-map`, `crates/bisect-report`, `analysis/`, `reports/`, dashboards | accepted |
| BISECT CLI and UI shells | PKG-010 | User-facing BISECT commands, TUI/web surfaces, orchestration, and help behavior. Package-family CLIs remain owned by their package boundaries. | REQ-003, REQ-007, REQ-031, REQ-032 | IF-001, IF-006 | `crates/bisect-cli`, `crates/bisect-tui`, `crates/bisect-web` | accepted |
| Python support and legacy research scripts | PKG-011 | Data acquisition, experiments, validation, dashboard generation, and archived reference scripts outside the core Rust production path. | REQ-004, REQ-024, REQ-027, REQ-033 | IF-006, IF-008 | `scripts/`, `python/bisect_py`, `archive/python-pipeline-final/` | accepted |
| Research and public evidence corpus | PKG-012 | Papers, README claims, concept docs, public dashboards, legal docs, and evidence-status reviews. | REQ-005, REQ-006, REQ-024, REQ-025, REQ-026, REQ-027 | IF-006, IF-008 | `README.md`, `docs/`, `docs/papers/`, `research/`, GitHub Pages dashboard | accepted |
| Wave/pulse execution records | PKG-013 | Controlled implementation and review records that connect changes to requirements, validation, risks, and role gates. | REQ-028, REQ-029, REQ-030, REQ-034, REQ-035 | IF-007 | `context/waves/`, pulse plans, review panels | accepted |
| Archived forensic references | PKG-014 | Preserve sealed historical references without silently driving active production behavior. | REQ-031, REQ-033 | none unless explicitly promoted | `archive/python-pipeline-final/` and other archives | accepted |

## AR-03 Package / Language Boundaries

Detailed boundary rules are controlled by `docs/vtrace/PACKAGE_BOUNDARIES.md`. Architecture-level decisions:

| Decision ID | Decision | Rationale | Status |
|---|---|---|---|
| ARCH-001 | Rust crates are the production control plane for algorithmic redistricting and package audit. | The Rust workspace is the maintained, fast, typed implementation surface. | accepted |
| ARCH-002 | Python scripts are support, research, acquisition, validation, or dashboard utilities unless a specific script is promoted by interface control. | Prevents legacy/research scripts from silently becoming production contracts. | accepted |
| ARCH-003 | Docs, papers, dashboards, and reports are evidence surfaces, not self-validating authority. | Public artifacts need claim status, limitations, and source custody. | accepted |
| ARCH-004 | RPLAN, RCOUNT, RCTX, and RHIST packages converge on manifest/schema/audit status vocabulary, but each package family owns its own domain semantics. | Shared integrity mechanics must not flatten plan, count, context, or history concepts. | accepted |
| ARCH-005 | The VTRACE control plane governs accepted IDs and review gates, but implementation work remains in normal crates/scripts/docs with trace pointers. | Keeps process artifacts from replacing engineering ownership. | accepted |

## AR-04 Data Flow

```text
External public/protected sources
  -> source custody / fetch / adapter layer
  -> local data cache or package source manifests
  -> deterministic kernels and package parsers
  -> BISECT construction or package audit workflows
  -> run/package outputs with manifests, hashes, and status
  -> analysis/report/map/research/public artifacts
  -> VTRACE trace, DCR, review, verification, and validation records
```

Redistricting flow:

```text
configs/{label}.yml + data/{year}/ + adjacency
  -> bisect build
  -> runs/{label}/{year}/{state}/final_assignments.json + run metadata
  -> bisect label-analyze
  -> analysis/{label}/{year}/...
  -> bisect label-report
  -> reports/{label}/{year}/...
  -> bisect label-verify / VTRACE evidence review
```

Package/evidence flow:

```text
RPLAN/RCOUNT/RCTX/RHIST source package
  -> package-family IO
  -> audit/reconciliation/context validation
  -> pass / pass-with-risk / partial / blocked / deferred / gap status
  -> public or restricted evidence disposition
```

## AR-05 Dependencies

| Dependency | Purpose | Boundary / Risk | Verification |
|---|---|---|---|
| METIS engines (`metis-core`, `metis`, `bisect-metis`, `gpmetis` where used) | Graph partitioning backend. | External engine behavior affects reproducibility and replay; current workspace dependencies include `metis-core` and `metis`, while `bisect-metis`/`gpmetis` are engine/tool paths described in project docs. | Record engine path, crate/tool source, feature set, version/hash or version-unknown finding, executable provenance where applicable, and deterministic seed/search metadata. |
| Census TIGER/Line, PL 94-171, ACS | Geography, population, and demographics inputs. | Source changes or missing custody break evidence claims. | Source URL/date/version/hash or custody pointer. |
| Election/geography external sources (VEST, OpenElections, state SOS, DRA/PlanScore/GerryChain where applicable) | Research, comparison, count, and interoperability inputs. | External version and schema drift. | Interface records must name version or version-unknown status. |
| Rust workspace dependencies (`serde`, `csv`, `sha2`, `geo`, `clap`, etc.) | Serialization, hashing, geometry, CLI, and support. | Dependency updates may affect outputs or schemas. | Build/test plus provenance for evidence-producing runs. |
| Typst/LaTeX/Python tooling | Reports, papers, figures, dashboards, and experiments. | Toolchain drift changes public artifacts. | Tool versions or gap status in evidence package where claims depend on output. |
| FLETCH source handoff | Source acquisition/cacheline handoff for fetch sources. | A cache hit proves acquisition, not redistricting validation. | Keep handoff claim separate from BISECT build/analyze/report verification. |

## AR-06 Failure Modes

| Failure Mode | Impact | Mitigation | Evidence |
|---|---|---|---|
| CLI docs and binary behavior diverge. | Interface claims become stale. | Interface inspection records current behavior or version-unknown status. | `docs/vtrace/INTERFACES.md`, CLI help snapshots planned. |
| Source data missing, stale, or unverifiable. | Runs or packages cannot support strong provenance claims. | Block, partial, deferred, or gap status rather than inferred success. | Source custody disposition and verification record. |
| Search/run metadata incomplete. | Evidence-producing runs cannot be replayed. | Require seeds, attempt counts, selection rule, executable provenance, and external-tool disclosure going forward. | Run manifest / VTRACE verification. |
| Package canonicalization ambiguous. | Hash/audit results cannot be reproduced. | Version canonicalization rules and treat version unknown as finding. | Package audit fixture. |
| Election-count semantics flattened. | Count/reconciliation claims become invalid or misleading. | Preserve contests, ballots, ballot cards, batches, CVRs, lifecycle status, parser diagnostics, and jurisdiction variation. | RCOUNT audit review. |
| Legal readiness conflated with algorithm success. | Outputs may be overstated in legal contexts. | Separate legal gates and explicit non-claims. | Legal-boundary review. |
| Public artifacts expose protected data or unsupported claims. | Privacy, coercion, copyright, or evidence harms. | VAULT/source-custody review and claim classification. | Public-artifact review. |
| Wave/pulse completion without trace. | New work escapes VTRACE control. | Pulses name requirements, boundaries, validation commands, role gates, risks, and pitfalls. | Pulse review. |
| S4 closure mistaken for release readiness. | Public interoperability, court/legal packaging, or full reproducibility claims may outrun evidence. | DCR-001 through DCR-007 define required follow-on gates. | `docs/vtrace/DCRS.md` and DCR review. |

## AR-07 Open Risks

| ID | Risk | Disposition | Owner | Status |
|---|---|---|---|---|
| AR-RISK-001 | Exact current CLI behavior has not been exhaustively compared to docs. | WP-002 records current surfaces; DCR-005 controls the public compatibility matrix. | BISECT owners | controlled |
| AR-RISK-002 | Exact schemas/canonicalization rules for all package families are not fully inventoried here. | WP-004 records current tests; DCR-001 controls public golden fixture promotion. | Package owners | controlled |
| AR-RISK-003 | Research papers and README contain many public claims not yet classified under VTRACE. | WP-005 bounds headline surfaces; DCR-004 controls the public evidence package contract. | Research owners / DATUM / SCALE | controlled |
| AR-RISK-004 | Generated output commit/publication policy remains incomplete for every artifact class. | WP-006 records custody defaults; DCR-004 and release VAULT review control selected public bundles. | Maintainers / VAULT | controlled |
| AR-RISK-005 | S4 work-package satisfaction could be interpreted as S6 release readiness. | DCR-001 through DCR-007 are filed as release-readiness gates. | Maintainers / role lanes | controlled |

## AR-08 Architecture Gate

Decision: accepted for S2 Design Baseline.

S2 lock disposition:

- [x] Architecture components map to accepted `SPEC-*` and `REQ-*` IDs.
- [x] Package boundaries have owners, public interfaces, dependency directions, and forbidden responsibilities.
- [x] Interface details expand accepted `IF-*` contracts without renumbering locked IDs.
- [x] Detailed design decisions identify invariants, edge cases, and code-rigor hooks.
- [x] Role review findings are fixed or explicitly deferred.
