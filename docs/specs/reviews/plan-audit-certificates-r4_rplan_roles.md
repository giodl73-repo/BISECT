---
reviewer: ROLE PANEL
roles: BOUNDARY, WARD, COVENANT, CONTOUR, MERIDIAN, BENCHMARK, SCALE, PRECINCT, DATUM, COMMONS, LEDGER, SURVEY, TRENCH
spec: U.20 Plan Audit Certificates
round: 4
date: 2026-05-10
score: 3.1
---

# Round 4 Role Review: U.20 Plan Audit Certificates

## Summary

The U.20 refactor from `bisect-constraints` / `bisect certify` to
`rplan-audit` / `rplan audit` is the right architectural move. It makes the
audit layer usable for non-bisect plans and gives later algorithm crates a
shared validity contract.

The spec is no longer blocked on naming or scope. It is blocked on alignment
with the new RPLAN v0.2 / `.rctx` model. The audit context still has duplicate
or inconsistent graph/unit fields, the CLI still looks partly data-directory
based rather than context-artifact based, and population context is not optional
even though missing-input behavior is part of the audit model.

Decision: **revise before implementation; no further naming debate needed**.

## Scores

| Role | Score | Reason |
|------|-------|--------|
| BOUNDARY | 3.5/4 | Correct generic crate boundary and no bisect dependency in `rplan-*`. |
| WARD | 3/4 | Legal-profile disclaimers are still appropriate. |
| COVENANT | 3/4 | Certificate model is strong, but context hashes need to be explicit. |
| CONTOUR | 2.5/4 | `AuditContext` must align with `.rctx` and avoid duplicate graph-unit state. |
| MERIDIAN | 3/4 | Contiguity check remains clear once graph context is defined. |
| BENCHMARK | 3/4 | Fixtures are useful; add `.rplan`/`.rctx` fixture pairs. |
| SCALE | 3.5/4 | Optional context keeps the format scalable. |
| PRECINCT | 3/4 | Multi-unit support is now possible via RPLAN v0.2. |
| DATUM | 3/4 | Source hashes exist; context hash should be added to certificates. |
| COMMONS | 3.5/4 | External audit tools are now first-class consumers. |
| LEDGER | 3/4 | Audit certificate hash rules are good; context identity is missing. |
| SURVEY | 3/4 | `rplan audit` is clear but flags need context-first cleanup. |
| TRENCH | 3/4 | Rollout order is practical. |
| **Average** | **3.1/4** | Revise, then implement phase 1. |

## Required Fixes

### R4-A: Align `AuditContext` with `.rctx`

Current `AuditContext` has both:

```rust
pub graph: Option<UnitGraph>
pub plan_units: PlanUnitIndex
pub graph_unit_ids: Vec<String>
```

This is redundant and can disagree. Prefer:

```rust
pub struct AuditContext {
    pub context: Option<RplanContext>,
    pub legal_profile: LegalProfile,
    pub runtime: RuntimeProvenance,
}
```

where `RplanContext` owns:

```rust
pub units: PlanUnitIndex
pub graph: Option<UnitGraph>
pub populations: Option<Vec<i64>>
pub source_hashes: SourceHashes
pub context_hash: String
```

Then the audit rule is:

```text
plan.units must match context.units by canonical hash
```

Do not maintain a separate `graph_unit_ids` vector.

### R4-B: Make population context optional

The current struct has:

```rust
pub populations: Vec<i64>
```

But the check model has `MissingInput`. Use `Option<Vec<i64>>` in the context
model so population checks can return `POPULATION_MISSING` rather than requiring
callers to invent empty vectors.

### R4-C: Add context hash to audit certificates

`AuditCertificate` should include:

```rust
pub context_hash: Option<String>
```

This lets an auditor distinguish:

- same plan against same context
- same plan against different adjacency/population context
- format-only audit with no context

The `content_hash` should include `context_hash`.

### R4-D: Make CLI context-first

The CLI example still uses `--data-dir`. The generic tool should take explicit
context:

```bash
rplan audit \
  --plan runs/nc_2020/plan.rplan \
  --context data/2020/NC/context.rctx \
  --legal-profile profiles/us-congressional-2020.json \
  --output runs/nc_2020/audit-certificate.json
```

`--data-dir` may exist as a bisect wrapper convenience, but `rplan-cli` should
prefer `--context`.

### R4-E: Replace remaining certification wording

The spec still uses "certifies compliance" language in a few places. For this
tool, use:

- "audits against the supplied profile"
- "reports pass/fail for the supplied profile"
- "does not certify legal validity"

This keeps the public claim precise.

### R4-F: Add paired fixtures

Phase 1 tests should include paired `.rplan` and `.rctx` fixtures:

- 5-node path graph with contiguous and disconnected assignments
- 3x3 grid with populations and county labels
- missing-context plan where contiguity returns `MissingInput`
- same `.rplan` under two `.rctx` contexts to prove `context_hash` changes

## Non-Blocking Notes

- `rplan-audit` should expose library checks separately from CLI command
  handling.
- `bisect-cli` can wrap `rplan-audit`, but the final audit certificate writer
  should live in `rplan-audit`.
- The old R1-R3 topology review records should remain as history, but future
  approvals should use the Plan Audit / RPLAN framing.
