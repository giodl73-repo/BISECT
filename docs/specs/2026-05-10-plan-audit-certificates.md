# Spec: U.20 Plan Audit Certificates

**Status:** Round 5 approved for `rplan-audit` phase 1 after `rplan-core` / `rplan-io`  
**Date:** 2026-05-10  
**Track:** U.20 — Search and Optimization  
**New crates:** `rplan-core`, `rplan-io`, `rplan-audit`; later `rplan-geo`  
**CLI surface:** `rplan audit`; optional `bisect` wrapper later  
**Depends on:** `rplan-core`, `rplan-io`; optional `rplan-geo`/`geo` feature for geometry checks  
**Blocks:** U.16 branch-and-cut, U.17 branch-and-price, U.18 local search,
T.15 clustering, T.17 flow construction, U.19 evolutionary comparison

**Review records:** [`topology-constraint-certification-r1_roles.md`](reviews/topology-constraint-certification-r1_roles.md),
[`topology-constraint-certification-r2_roles.md`](reviews/topology-constraint-certification-r2_roles.md),
[`topology-constraint-certification-r3_roles.md`](reviews/topology-constraint-certification-r3_roles.md),
[`plan-audit-certificates-r4_rplan_roles.md`](reviews/plan-audit-certificates-r4_rplan_roles.md),
[`plan-audit-certificates-r5_roles.md`](reviews/plan-audit-certificates-r5_roles.md)

**Factoring decision:** [`2026-05-10-rplan-incubation.md`](2026-05-10-rplan-incubation.md)
**Plan/context schema:** [`2026-05-10-rplan-v0.2-schema.md`](2026-05-10-rplan-v0.2-schema.md)

---

## Purpose

Every new algorithm family needs the same answer to a basic question: did this
plan satisfy the constraints it claims to satisfy, and can an outside party
verify that from the emitted artifacts?

`rplan-audit` is the shared plan-audit layer. It must be implemented before the
new algorithm crates so each family emits the same audit-certificate schema,
uses the same plan identity rules, and reports failures in a machine-checkable
way.

This crate does not generate plans. It audits plans produced elsewhere,
including plans that have nothing to do with bisect.

## Non-Goals

- It does not decide which algorithm is best.
- It does not solve redistricting optimization problems.
- It does not replace paper-specific statistical evaluation.
- It does not hardcode state legislative law into source code without a
  versioned legal profile.
- It does not silently convert graph compactness proxies into geometric
  compactness claims.

## Crate Boundary

```text
rplan-audit
  -> rplan-core
  -> rplan-io
  -> rplan-geo / geo (optional feature)

rplan-io
  -> rplan-core

bisect-cli
  -> rplan-audit
```

Allowed dependencies:

- `rplan-core`
- `rplan-io`
- `serde`, `serde_json`
- `thiserror`
- `sha2`
- `rplan-geo`, `geo`, or `geo-types` behind feature `geometry`

Forbidden dependencies:

- any algorithm crate (`bisect-ilp`, `bisect-pareto`, `bisect-smc`,
  `bisect-ensemble`, `bisect-clustering`, `bisect-flow`, etc.)
- `bisect-cli`
- any `bisect-*` crate from any `rplan-*` crate

`bisect-cli` depends on `rplan-audit`, not the reverse.

## Core Data Types

The current `bisect-core::Partition` is a hash-map assignment wrapper. That is
not stable enough for cross-crate audit because hash-map ordering cannot define
plan identity. U.20 introduces a canonical plan representation in `rplan-core`.

### Plan Identity

```rust
pub enum UnitKind {
    Block,
    BlockGroup,
    Tract,
    County,
    Precinct,
    Imported,
}

pub struct PlanUnitIndex {
    pub unit_ids: Vec<String>,
    pub unit_kind: UnitKind,
    pub state: String,
    pub year: u16,
    pub canonical_order: CanonicalOrder,
}

pub enum CanonicalOrder {
    ExplicitUnitIds,
    SortedGeoid,
}

pub struct DistrictPlan {
    pub schema_version: String, // "district-plan-v1"
    pub units: PlanUnitIndex,
    pub assignment: Vec<u32>,
    pub k: usize,
}
```

Rules:

- `assignment[i]` is the district id for `units.unit_ids[i]`.
- `assignment.len() == units.unit_ids.len()` is mandatory.
- District ids are canonicalized to `0..k-1` for internal use.
- RPLAN v0.1 imports may accept `1..k` labels, but `rplan-io` converts them to
  `0..k-1` and stores the original labels in `display_labels`.
- The plan hash is computed from canonical JSON with sorted object keys,
  `schema_version`, `unit_ids`, `assignment`, `k`, `state`, `year`, and
  `unit_kind`.

The normative plan-file schema is defined in
[`2026-05-10-rplan-v0.2-schema.md`](2026-05-10-rplan-v0.2-schema.md). U.20
uses that schema rather than redefining RPLAN.

### Canonical JSON

Plan hashes, legal-profile hashes, and certificate content hashes use the same
canonical JSON rule:

- UTF-8 encoding
- no insignificant whitespace
- object keys sorted lexicographically by Unicode code point
- arrays preserve order
- integers encoded in base 10 with no leading zeros
- finite floats encoded with Rust `serde_json` shortest-roundtrip formatting;
  non-finite floats are forbidden
- absent optional fields and explicit `null` are different values

Implementation must use a local canonical writer or a pinned canonicalization
helper. It must not rely on incidental `serde_json::Map` iteration order.

### Audit Context

```rust
pub struct AuditContext {
    pub context: Option<RplanContext>,
    pub legal_profile: LegalProfile,
    pub runtime: RuntimeProvenance,
}

pub struct RplanContext {
    pub units: PlanUnitIndex,
    pub graph: Option<UnitGraph>,
    pub populations: Option<Vec<i64>>,
    pub source_hashes: SourceHashes,
    pub context_hash: String,
    pub geometry: Option<GeometryContext>,
    pub subdivisions: Option<SubdivisionContext>,
    pub demographics: Option<DemographicContext>,
    pub elections: Option<ElectionContext>,
}
```

The context owns all inputs needed to audit a plan. Auditors return
structured missing-input errors rather than treating absent data as success.

Graph, population, and unit-universe context is carried by the reserved `.rctx`
context artifact defined in the RPLAN v0.2 schema. `.rplan` remains the plan
assignment artifact; `.rctx` supplies the context needed for topology and
population audits.

Audit requires `plan.units.unit_universe_hash == context.units.unit_universe_hash`
when context is supplied. If a loader supplies graph vertices in a different
order, it must reorder either the graph or the plan before audit. U.20 never
guesses a permutation from GEOIDs during audit because that would make plan
identity depend on implicit loader behavior.

## Legal Profile Schema

Legal profiles are data, not comments. The audit result must state which
profile was applied.

```rust
pub struct LegalProfile {
    pub schema_version: String, // "legal-profile-v1"
    pub jurisdiction: String,   // e.g. "US", "NC", "WA"
    pub chamber: Chamber,
    pub year: u16,
    pub population_tolerance: PopulationToleranceRule,
    pub contiguity_required: bool,
    pub county_split_rule: SplitRule,
    pub municipal_split_rule: SplitRule,
    pub nesting_rule: NestingRule,
    pub vra_policy: VraPolicy,
}
```

### Chamber

```rust
pub enum Chamber {
    Congressional,
    StateHouse,
    StateSenate,
    Local,
    Custom(String),
}
```

### Population Tolerance

```rust
pub enum PopulationToleranceRule {
    ExactAbsolute { max_total_deviation: i64 },
    ExactPpm { max_deviation_ppm: i64 },
    Percent { max_deviation_percent: f64 },
    StateSpecific { rule_id: String },
    Unspecified,
}
```

Defaults:

- congressional: the default shipped profile is
  `US_CONGRESSIONAL_PROJECT_V1`, using the existing project convention of
  `ExactPpm { max_deviation_ppm: 5000 }` unless a stricter profile is supplied.
  The certificate must label this as a project profile, not as a guarantee that
  every court would treat ±0.5% as sufficient for congressional plans.
- strict congressional profiles may use `ExactAbsolute { max_total_deviation: 1 }`
  or another externally supplied rule.
- state legislative: no universal default; a state/chamber profile is required
  before auditing compliance

The audit certificate must distinguish "passes supplied profile" from "legally valid
in all respects." U.20 audits against the profile applied; it does not provide legal
advice.

### Split Rules

```rust
pub enum SplitRule {
    NotEvaluated,
    CountOnly,
    MinimizeWherePracticable,
    PreserveUnlessNecessary,
    StateSpecific { rule_id: String },
}
```

`CountOnly` reports split counts but does not assert state-law compliance.

### VRA Policy

```rust
pub enum VraPolicy {
    NotEvaluated,
    ReportOpportunityDistricts {
        minority_group: String,
        vap_threshold: f64,
    },
    StateSpecific { rule_id: String },
}
```

VRA reporting must use VAP when the profile asks for VAP. If only total
population is available, the audit returns `MissingInput`.

## Source Hashes And Provenance

```rust
pub struct SourceHashes {
    pub pl_94171: Option<SourceHash>,
    pub tiger_line: Option<SourceHash>,
    pub adjacency: Option<SourceHash>,
    pub geometry: Option<SourceHash>,
    pub demographics: Option<SourceHash>,
    pub elections: Option<SourceHash>,
    pub subdivisions: Option<SourceHash>,
}

pub struct SourceHash {
    pub source_name: String,
    pub vintage: String,
    pub sha256: String,
    pub path_or_url: Option<String>,
}

pub struct RuntimeProvenance {
    pub binary_name: String,
    pub binary_version: String,
    pub git_commit: Option<String>,
    pub build_profile: Option<String>,
    pub solver: Option<SolverProvenance>,
}

pub struct SolverProvenance {
    pub name: String,
    pub version: Option<String>,
    pub mode: Option<String>, // e.g. lazy-callback, iterative-separation
    pub time_limit_secs: Option<u64>,
    pub optimality_gap: Option<f64>,
}
```

Certificates may be produced for development runs without all source hashes, but
such certificates must include warning code `PROVENANCE_INCOMPLETE`.

## Audit Certificate Schema

Every call to `rplan audit` emits `AuditCertificate` JSON.

```rust
pub struct AuditCertificate {
    pub schema_version: String, // "audit-certificate-v1"
    pub certificate_id: String,
    pub generated_at_utc: String,
    pub content_hash: String,
    pub plan_hash: String,
    pub plan_schema_version: String,
    pub context_hash: Option<String>,
    pub legal_profile_hash: String,
    pub legal_profile: LegalProfileSummary,
    pub source_hashes: SourceHashes,
    pub runtime: RuntimeProvenance,
    pub result: AuditResult,
    pub checks: Vec<AuditCheck>,
    pub warnings: Vec<AuditWarning>,
}

pub struct LegalProfileSummary {
    pub schema_version: String,
    pub profile_id: String,
    pub jurisdiction: String,
    pub chamber: Chamber,
    pub year: u16,
    pub population_tolerance: PopulationToleranceRule,
    pub county_split_rule: SplitRule,
    pub municipal_split_rule: SplitRule,
    pub vra_policy: VraPolicy,
    pub legal_disclaimer: String,
}

pub struct AuditWarning {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub affected_check: Option<String>,
}

pub enum AuditResult {
    Pass,
    Fail,
    PassWithWarnings,
}

pub struct AuditCheck {
    pub name: String,
    pub status: CheckStatus,
    pub severity: Severity,
    pub summary: String,
    pub witnesses: Vec<Witness>,
}
```

`content_hash` is computed from canonical JSON excluding `certificate_id`,
`generated_at_utc`, and `content_hash` itself. This gives auditors a stable hash
for identical inputs while still allowing the certificate document to carry a
unique id and generation time.

### Status And Severity

```rust
pub enum CheckStatus {
    Pass,
    Fail,
    NotEvaluated,
    MissingInput,
}

pub enum Severity {
    Error,
    Warning,
    Info,
}
```

`AuditResult::Pass` requires no `Fail` checks and no warning that
changes legal/profile interpretation. `PassWithWarnings` means the supplied
checks pass, but provenance, optional inputs, or non-blocking checks are
incomplete.

If a check is explicitly requested in `--constraints`, `MissingInput` is an
`Error` unless the selected legal profile marks that check advisory. If a check
is not requested, it may appear as `NotEvaluated` with `Info` severity.

### Witness Types

```rust
pub enum Witness {
    Population(PopulationWitness),
    Contiguity(ContiguityWitness),
    Split(SplitWitness),
    Vra(VraWitness),
    Geometry(GeometryWitness),
    MissingInput(MissingInputWitness),
}
```

Witness structs are part of the public certificate schema. Phase 1 must define
`PopulationWitness`, `ContiguityWitness`, and `MissingInputWitness` concretely.
`SplitWitness`, `VraWitness`, and `GeometryWitness` may be schema stubs until
their rollout phases, but their variant names and JSON tags are reserved.

Required district-level witnesses:

- population: population, ideal, absolute deviation, percent deviation,
  deviation ppm
- contiguity: connected component count and component unit ids for failures
- split counts: split subdivision ids and district memberships
- VRA: minority VAP and total VAP per reported district when evaluated

## Checks

### 1. Plan Shape

Validates:

- assignment length equals unit count
- supplied context has the same `unit_universe_hash` as the plan
- `k > 0`
- all district ids are in range
- all districts are nonempty unless `plan.allow_empty_districts` is explicitly
  true or the legal profile explicitly allows empty districts

Failure codes:

- `PLAN_ASSIGNMENT_LENGTH_MISMATCH`
- `PLAN_CONTEXT_UNIT_UNIVERSE_MISMATCH`
- `PLAN_INVALID_DISTRICT_ID`
- `PLAN_EMPTY_DISTRICT`

### 2. Population

Validates district populations against `LegalProfile.population_tolerance`.

Failure codes:

- `POPULATION_MISSING`
- `POPULATION_OUT_OF_TOLERANCE`
- `POPULATION_PROFILE_UNSPECIFIED`

### 3. Contiguity

For each district, induces the subgraph on assigned units and computes connected
components.

Failure codes:

- `CONTIGUITY_REQUIRED_BUT_GRAPH_MISSING`
- `DISTRICT_DISCONNECTED`

Single-unit districts pass contiguity. Isolated units pass only if the district
contains exactly that isolated unit or the graph/profile marks island handling
explicitly.

### 4. Splits

Reports county/municipal split counts when subdivision membership is available.
Audits against the supplied profile only when that profile's split rule has an
implemented checker.

Failure/warning codes:

- `SUBDIVISION_DATA_MISSING`
- `SPLIT_RULE_NOT_IMPLEMENTED`
- `SUBDIVISION_SPLIT_VIOLATION`

### 5. VRA Opportunity Reporting

Reports opportunity districts under `LegalProfile.vra_policy`.

Failure/warning codes:

- `VRA_DATA_MISSING`
- `VRA_REQUIRES_VAP`
- `VRA_POLICY_NOT_IMPLEMENTED`

U.20 does not decide full Section 2 liability. It audits the demographic inputs
and threshold policy supplied in the profile.

### 6. Geometry / Topology

Optional `geometry` feature:

- geometric compactness witnesses when polygon data are available
- invalid geometry warnings
- projection metadata for Euclidean measures

Warning/failure codes:

- `GEOMETRY_DATA_MISSING`
- `GEOMETRY_INVALID`
- `PROJECTION_UNSPECIFIED`
- `COMPACTNESS_PROXY_USED`

Edge cut is a graph compactness proxy. Polsby-Popper and Reock require geometry.

## CLI

```bash
rplan audit \
  --plan runs/nc_2020/plan.rplan \
  --context data/2020/NC/context.rctx \
  --legal-profile profiles/us-congressional-2020.json \
  --output runs/nc_2020/audit-certificate.json
```

Flags:

- `--plan PATH`: plan assignment JSON/RPLAN file
- `--context PATH`: optional `.rctx` context artifact; required for population,
  contiguity, split, and graph-topology checks
- `--legal-profile PATH`: optional for congressional default, required for
  state legislative audit
- `--constraints LIST`: default `plan-shape,population,contiguity,splits,vra`
- `--output PATH`
- `--format json|pretty-json`
- `--allow-warnings`: exit 0 for `PassWithWarnings`; without this flag,
  `PassWithWarnings` exits 1 in strict automation contexts
- `--fixed-generated-at`: test-only timestamp override for deterministic fixture
  output

Exit codes:

- `0`: pass, or pass with warnings when `--allow-warnings` is set
- `1`: fail, or pass with warnings in strict mode
- `2`: missing required input or malformed plan/profile

`bisect-cli` may provide a convenience wrapper that resolves `--context` from
its run directory or data directory, but `rplan-cli` is context-artifact first.

## Integration Points

## Future Consumers And Lineage

U.20 is a shared contract, not a one-off CLI. Phase 1 must be designed so later
algorithm families can add metadata without rewriting the audit-certificate
schema.

| Consumer | How it uses U.20 | Required audit-certificate lineage fields |
|----------|------------------|-------------------------------------|
| `rplan-cli` | Audits RPLAN files from any producer. | `plan_hash`, `certificate_content_hash`, `legal_profile`, `source_hashes`, final check statuses |
| `bisect-cli::runner` | Audits every final produced state/chamber plan before writing the run manifest. | `plan_hash`, `certificate_content_hash`, `legal_profile`, `source_hashes`, final check statuses |
| `bisect report` | Displays validity, warnings, and district-level witnesses to users and reviewers. | stable check names, severity, warning codes, witness summaries |
| `bisect-ilp` / U.16 | Audits exact or gap-bounded plans and distinguishes true branch-and-cut from iterative separation/fallback. | solver provenance, solver mode, lower bound, incumbent objective, optimality gap, fallback reason |
| `bisect-column` / U.17 | Audits branch-and-price selected plans and preserves column-generation audit trail. | solver provenance, column-pool hash, master status, pricing status, branch status |
| `bisect-clustering` / T.15-T.16 | Audits capacity-clustering and regionalization outputs after repair. | repair witness, repair method, pre-repair plan hash, final plan hash, population/contiguity/split witnesses |
| `bisect-flow` / T.17 | Audits flow assignment outputs and capacity/repair behavior. | flow solver provenance, capacity status, infeasibility witness, repair witness |
| `bisect-local-search` / U.18 | Audits final improved plans produced from an existing valid plan. | starting plan hash, final plan hash, objective name, objective delta, accepted-move count |
| `bisect-pareto` / U.7/U.19 | Audits selected frontier plans and summarizes validity across the frontier. | frontier id, entry id, objective vector, repair status, per-plan validity status |
| `bisect-smc` / G.7/U.14 | Audits selected percentile or representative plans from a weighted sample. | sample id, particle id, particle weight, selected percentile/rank, sampler provenance |
| External audit tools | Independently verify assignment, source provenance, and certificate result. | schema version, canonical JSON rules, plan hash, context hash, content hash, source hashes, legal profile hash |

### Forward-Compatibility Rules

- Phase 1 field names are stable once implementation begins.
- Later phases may add optional fields, warning codes, check names, and witness
  payload fields.
- Later phases must not change the meaning of `Pass`, `Fail`,
  `PassWithWarnings`, `MissingInput`, or existing warning severity.
- Readers for `audit-certificate-v1` must ignore unknown optional fields.
- Readers must fail closed on unknown `CheckStatus`, unknown `Severity`, or
  malformed required fields.
- Reserved witness variants `Split`, `Vra`, and `Geometry` are present in v1 so
  later rollout does not change the top-level witness enum shape.
- Algorithm-specific lineage belongs in optional `algorithm_lineage` payloads,
  not in ad hoc top-level fields.

```rust
pub struct AlgorithmLineage {
    pub producer_crate: String,
    pub producer_version: String,
    pub method: String,
    pub parent_plan_hashes: Vec<String>,
    pub parameters_hash: String,
    pub extra: serde_json::Value,
}
```

`extra` must be canonical JSON and must not duplicate required top-level fields.
External readers may ignore `extra` while still verifying the plan and legal
profile checks.

### `bisect-cli::runner`

Before writing a final run manifest, `bisect-cli` should audit the final plan
and write:

```text
runs/{label}/{year}/audit-certificate.json
```

The run manifest stores:

- `audit_certificate_path`
- `audit_certificate_sha256` for the full certificate document as written
- `audit_certificate_content_hash` for the stable canonical certificate content
- `audit_result`
- `legal_profile_id`
- `context_hash`, when a `.rctx` context was used

### Algorithm Crates

Algorithm crates may call validators during search, but final output still goes
through the same certificate writer.

Required final integration:

- `bisect-ilp`: certificate includes solver status, lower bound, gap, fallback
- `bisect-pareto`: every frontier entry gets plan-level validity status; selected
  exports get full certificates
- `bisect-smc`: sampled plans may carry compact validity metadata; selected
  percentile plans get full certificates
- `bisect-clustering`: repair stage emits repair witness and final certificate
- `bisect-flow`: solver/fallback provenance included
- `bisect-local-search`: every accepted final plan has certificate hash

## Canonical Test Fixtures

These fixtures must be shared by `rplan-audit` tests and reused by later
algorithm crates.

### Fixture A: Path Graph

Five vertices in a line, equal population.

Assertions:

- contiguous assignment `[0,0,0,1,1]` passes contiguity
- assignment `[0,1,0,1,1]` fails district 0 contiguity

### Fixture B: Two-Clique Bridge

Two triangles joined by one bridge edge.

Assertions:

- bridge cut creates two contiguous districts
- mixed assignment across cliques creates split components

### Fixture C: 3x3 Grid

Nine equal-population units with county labels split left/right.

Assertions:

- row/column districts pass contiguity
- checkerboard district fails contiguity
- split checker reports expected county split count

### Fixture D: Impossible Population Capacity

Four units with populations `[90, 1, 1, 1]`, `k=2`, tolerance 5%.

Assertions:

- population audit fails
- witness identifies the overweight district

### Fixture E: Missing VAP

Valid graph and plan with VRA policy requiring VAP but no demographic context.

Assertions:

- VRA check returns `MissingInput`
- certificate result is `PassWithWarnings` or `Fail` depending on whether VRA is
  configured as required in the legal profile

## L0 Tests

Required unit tests in `rplan-audit`:

- canonical plan hash is stable
- assignment length mismatch fails
- invalid district id fails
- missing district fails
- population tolerance pass/fail witnesses are correct
- contiguity pass/fail witnesses are correct
- missing profile for state legislative audit is rejected
- missing optional subdivision data yields warning, not panic
- certificate JSON round-trips through serde
- certificate schema version is exactly `audit-certificate-v1`
- v1 reader ignores unknown optional fields
- v1 reader rejects unknown `CheckStatus`
- v1 reader rejects unknown `Severity`
- `AlgorithmLineage.extra` participates in `content_hash`
- `AlgorithmLineage.extra` cannot override required top-level fields
- a mock future-algorithm certificate remains verifiable without understanding
  the mock algorithm payload

## L1 Tests

Required CLI-level tests:

- `rplan audit` passes a valid 3x3 fixture plan
- `rplan audit` fails a disconnected fixture plan with exit code 1
- `rplan audit` returns `MissingInput` for contiguity when `--context` is absent
- same `.rplan` audited under two `.rctx` contexts produces different
  `context_hash` values
- malformed plan exits 2
- output certificate hash is stable across identical runs

## Rollout Plan

1. Add `rplan-core` types behind no behavior change.
2. Add `rplan-io` extraction from the current `bisect-report::rplan` module.
3. Add `rplan-audit` crate with plan-shape, population, and contiguity checks.
4. Add audit-certificate JSON writer and serde round-trip tests.
5. Add `rplan audit` CLI for fixture plans.
6. Integrate final-plan audit into `bisect-cli::runner`.
7. Add subdivision split reporting.
8. Add VRA opportunity reporting.
9. Add optional geometry/topology checks.

## Acceptance Criteria

U.20 is implementation-ready when:

- `DistrictPlan` identity and hash rules are specified and tested
- `LegalProfile` schema exists with congressional default and state-legislative
  "profile required" behavior
- `AuditCertificate` schema is versioned and round-trippable
- all canonical fixtures are implemented
- `rplan audit` handles pass, fail, and missing-input cases
- audit certificates include `context_hash` when a `.rctx` context is used
- final certificates include source hashes and runtime provenance
- no algorithm crate depends on private certificate internals
