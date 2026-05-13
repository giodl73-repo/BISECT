# V.18 Plan

## Thesis

Batch comparison audits extend RCOUNT's current batch-manifest accounting into
hand-tally versus reported-total statistical replay.

## Atlas

- `docs/algorithm-atlas/v18-batch-comparison.md`

## Implementation Tasks

- [ ] Add reported batch contest totals to fixtures.
- [ ] Add hand-tallied batch totals and source references.
- [x] Compute shared plurality overstatements through V.14 primitive.
- [x] Reuse V.14 overstatement/taint arithmetic where possible.
- [x] Add boundary replay surface for `batch-comparison-v1`.
- [x] Reuse V.14 taint-product replay for `batch-comparison-v1`.
- [x] Compute batch-level plurality margin overstatements from reported and
  hand-tallied totals.
- [x] Add package records for reported and hand-tallied batch contest totals.
- [x] Verify package batch comparison records against batch summaries.
- [x] Add missing-hand-tally and batch-size-drift negative fixtures.
- [x] Cross-check `AuditAlgorithmRun.sample_steps` against verified
  `BatchComparisonAudit` overstatement taints.
- [x] Add a core derivation helper that builds `batch-comparison-v1` algorithm
  runs from verified `BatchComparisonAudit` records plus sampled-batch order.
- [x] Round-trip a derived batch-comparison algorithm package through IO and
  replay it through the CLI.
- [ ] Call the derivation helper from source adapters when public audit reports
  provide sampled-batch order.

## Claim Boundary

Batch comparison is useful when ballot-level linkage is unavailable, but large
batches weaken localization and can increase audit workload.
