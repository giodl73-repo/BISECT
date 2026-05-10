# Spec: RPLAN Incubation And Crate Factoring

**Status:** Draft; architecture decision for U.20 implementation  
**Date:** 2026-05-10  
**Scope:** Generic district-plan interchange, audit, and future extraction path  
**Related specs:** [`2026-04-26-spec0-rplan-format.md`](2026-04-26-spec0-rplan-format.md),
[`2026-05-10-rplan-v0.2-schema.md`](2026-05-10-rplan-v0.2-schema.md),
[`2026-05-10-plan-audit-certificates.md`](2026-05-10-plan-audit-certificates.md)
**Review records:** [`rplan-incubation-r2_roles.md`](reviews/rplan-incubation-r2_roles.md),
[`rplan-v0.2-schema-r1_roles.md`](reviews/rplan-v0.2-schema-r1_roles.md),
[`rplan-v0.2-schema-r2_roles.md`](reviews/rplan-v0.2-schema-r2_roles.md),
[`plan-audit-certificates-r4_rplan_roles.md`](reviews/plan-audit-certificates-r4_rplan_roles.md),
[`plan-audit-certificates-r5_roles.md`](reviews/plan-audit-certificates-r5_roles.md)

## Decision

Keep RPLAN in this repository for now, but factor it as a generic crate family
that can be promoted to a standalone repository later.

The `rplan-*` crates are not bisect implementation crates. They define the
district-plan interchange and audit layer used by bisect and by external plan
workflows.

## Fixed Point

Until the RPLAN crate boundary is straightened out, treat this as the fixed
point for downstream work:

- The public name is **RPLAN**.
- The file extension is `.rplan`.
- The generic crate family is `rplan-*`.
- RPLAN incubates in this repo before any standalone repo promotion.
- `rplan-*` crates may not depend on `bisect-*`.
- `bisect-*` crates may depend on `rplan-*`.
- U.20 plan audit work must build on `rplan-core`, `rplan-io`, and
  `rplan-audit`, not a bisect-local validator.

Do not start U.16-U.19 algorithm-family implementation against provisional
plan/audit interfaces. First settle:

- RPLAN v0.2 schema: [`2026-05-10-rplan-v0.2-schema.md`](2026-05-10-rplan-v0.2-schema.md)
- `rplan-core` domain types
- graph/context artifact boundary
- v0.1 compatibility import path
- canonical hashing rules
- `bisect-report` adapter plan

## Names

- **Spec name:** RPLAN
- **File extension:** `.rplan`
- **Crate prefix:** `rplan-*`
- **CLI command:** `rplan`

## Crates

```text
rplan-core   # generic district-plan/domain model
rplan-io     # RPLAN/RCTX read/write, canonical JSON, schema migration
rplan-audit  # audit engine and audit certificates
rplan-geo    # optional topology/geometry helpers later
rplan-cli    # binary package; command name `rplan`
```

Initial dependency direction:

```text
rplan-io -> rplan-core
rplan-audit -> rplan-core
rplan-audit -> rplan-io
rplan-audit -> rplan-geo (optional)
rplan-cli -> rplan-io
rplan-cli -> rplan-audit

bisect-* -> rplan-*
```

Forbidden direction:

```text
rplan-* -> bisect-*
```

This is the extraction boundary. If an `rplan-*` crate needs a type currently
owned by a `bisect-*` crate, move or duplicate the generic concept into
`rplan-core` rather than depending on bisect internals.

## Workspace Policy

RPLAN starts inside the current repo so it can be tested against existing
bisect output, imports, manifests, and research fixtures without cross-repo
release friction.

The root workspace may include both families:

```toml
"crates/rplan-core",
"crates/rplan-io",
"crates/rplan-audit",
"crates/rplan-geo",
"crates/rplan-cli",
"crates/bisect-core",
"crates/bisect-cli",
```

If build time or ownership boundaries become painful, the repo can move to
separate nested workspaces later. The crate dependency boundary above is more
important than the physical workspace boundary.

## Promotion Triggers

Promote RPLAN to a standalone repository only after the interface has been
proven by real consumers.

Minimum triggers:

- RPLAN v0.2 schema exists with migration tests.
- `rplan audit` can audit a non-bisect plan.
- bisect emits RPLAN without private ordering or manifest assumptions.
- format docs define RPLAN without requiring bisect as the generator.
- at least one external importer/exporter path works.

## Relationship To U.20

U.20 should use the RPLAN crate family rather than a bisect-local constraints
crate. Its implementation home is `rplan-audit`, with `rplan-cli` exposing:

```bash
rplan audit plan.rplan --context manifest.json --output audit.json
```

`bisect-cli` may wrap this behavior for bisect users, but final plan validity
and audit-certificate semantics live in the generic RPLAN layer.
