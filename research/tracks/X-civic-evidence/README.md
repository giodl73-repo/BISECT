# Track X -- Civic Evidence Package Family

Track X coordinates package substrates that sit below or beside RPLAN and
RCOUNT: shared context, map presentation, unit history, custody, logs,
statistics, and public evidence bundles.

The track exists to prevent every domain package from reinventing geography,
lineage, source hashing, and claim boundaries.

Umbrella spec:

- `docs/specs/2026-05-13-civic-evidence-package-family.md`
- `docs/specs/2026-05-13-rctx-boundary.md`
- `docs/specs/2026-05-13-civic-evidence-layer-access-patterns.md`
- `docs/specs/2026-05-13-rhist-boundary.md`
- `docs/specs/2026-05-13-rhist-implementation.md`
- `docs/specs/2026-05-13-downstream-evidence-boundaries.md`

Review records:

- `docs/specs/reviews/civic-evidence-package-family-roles.md`
- `docs/specs/reviews/civic-evidence-package-family-r1_roles.md`
- `docs/specs/reviews/civic-evidence-layer-access-r1_roles.md`
- `docs/specs/reviews/rhist-implementation-r1_roles.md`

## Packages

| Code | Package | Working title | Status |
|------|---------|---------------|--------|
| X.0 | Family | Civic Evidence Package Family | umbrella spec landed |
| X.1 | RHIST | Reproducible Unit History | core/IO/CLI fixtures landed |
| X.2 | RCTX/RMAP | Shared Context And Map Boundary | `rctx-core` crosswalk slice landed |
| X.3 | RCHAIN/RLOG | Custody And Event Logs | deferred; boundary recorded |
| X.4 | RCASE | Public Evidence Bundle | deferred; boundary recorded |
| X.5 | RSTAT/RROLL | Statistics And Eligibility Universes | deferred; boundary recorded |
| X.6 | RAUDIT/RCERT | Audit And Certification Boundaries | deferred; boundary recorded |

## Boundary Plans

- `X.1+rhist-unit-history/plan.md`
- `X.2+rctx-rmap-boundary/plan.md`
- `X.3+custody-log-boundary/plan.md`
- `X.4+case-composition-boundary/plan.md`
- `X.5+statistics-and-roll-boundary/plan.md`
- `X.6+audit-cert-boundary/plan.md`

## Build Order

1. Stabilize RCTX as machine context and keep RMAP as presentation.
2. Build RHIST minimal lineage package.
3. Continue RCOUNT using RHIST-compatible references.
4. Split RAUDIT only if audit transcripts need to stand outside RCOUNT.
5. Add RSTAT/W-series analytics after enough normalized inputs exist.
6. Build RCASE last as a composition layer.
