# Nationwide 2020 Operational Certification

The inventory confirms all 50 States have local TIGER block geometry and PL
94-171 population sources.

- States: 50
- Congressional districts: 435
- Census blocks: 8,126,956
- Existing connected RCTX packages: 11
- Estimated complete RCTX storage: 2.02 GiB

See `inventory.json` for the size-ranked batch order and per-State readiness.

## Connected Contexts

All 50 State contexts now verify:

- 8,126,956 canonical blocks;
- zero disconnected State graphs; and
- 9,657 deterministic bridge edges.

See `rctx-verification.json`.

## Operational Trees

All 50 State assignments and all 435 one-seat leaves now verify. The national
replay independently checks the original State adjacency graphs and reports
zero omitted blocks, duplicate assignments, or disconnected leaves. All 385
nontrivial recursive nodes reach their arithmetic population floors.

See `OPERATIONAL_CERTIFICATION.md`, `national-tree-verification.json`, and
`national-proof-coverage.json`. Weighted-boundary and canonical optimality
remain unproved.
