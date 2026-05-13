# V.18 Batch Comparison Audits

## Mental Model

Batch comparison audits sample batches rather than individual ballots. A batch
may be a precinct, scanner batch, mail batch, or other tabulation unit. The
audit board hand-tallies the selected batch, then RCOUNT compares that hand
tally to the reported batch total and converts discrepancies into risk evidence.

## How RCOUNT Uses It

```text
batch manifest -> sampled batches -> hand tally -> reported tally comparison -> risk decision
```

RCOUNT already has batch manifests and batch accounting. This page defines the
statistical layer above those records.

## Step-By-Step Mechanics

1. Read the batch manifest and reported batch-level contest totals.
2. Replay or preserve the random batch sample.
3. Parse hand-tallied batch results.
4. Compute batch-level discrepancies and margin overstatements.
5. Feed discrepancies into a comparison-audit risk calculation.
6. Emit both accounting checks and risk checks.

## RCOUNT Transcript Fields

| Field | Meaning |
|---|---|
| `method_id` | `batch-comparison-v1` |
| `batch_id` | sampled batch |
| `batch_ballots` | batch size |
| `reported_totals` | machine/tabulated batch totals |
| `hand_totals` | audit-board batch totals |
| `overstatement` | margin overstatement for assertion |
| `sampling_weight` | probability or weight for selected batch |
| `p_value` | running risk measure |

## Fixtures

- Package-level boundary fixture for `batch-comparison-v1`.
- One-batch no-discrepancy positive fixture.
- Discrepancy fixture where a batch overstates margin.
- Missing batch hand tally negative fixture.
- Batch-size drift fixture that fails before risk math.

## Current Implementation

V.18 currently reuses the V.14 comparison-audit substrate:

- plurality winner/loser overstatement values;
- exact taint normalization by reported margin;
- exact batch plurality margin overstatement from reported totals versus
  hand-tallied totals;
- package-level `BatchComparisonAudit` records in `audits/batch-comparison.ndjson`;
- `batch_comparison_overstatement` verification against package batch summaries;
- negative package checks for missing hand tallies and batch-size drift;
- `rcount replay-audit-algorithms` taint-product replay for
  `batch-comparison-v1`, using the same initial comparison-risk transcript as
  V.14;
- `batch_comparison_algorithm_linkage` verification, which checks that
  `batch-comparison-v1` sample-step taints match the corresponding verified
  `BatchComparisonAudit` overstatement divided by reported margin;
- `derive_batch_comparison_algorithm_run`, a core helper that turns verified
  `BatchComparisonAudit` records plus sampled-batch order into a replayable
  `batch-comparison-v1` `AuditAlgorithmRun`;
- IO round-trip and CLI replay coverage for a package that contains both the
  verified batch-comparison evidence and the derived algorithm run;
- boundary output for incomplete batch comparison algorithm runs, such as
  missing `risk_limit_ppm`.

The next implementation slice should call the derivation helper from source
adapters once public audit reports provide sampled-batch order. The verifier
already rejects package drift between hand-tally evidence and declared
algorithm taints.

## Claim Boundary

Batch comparison is less granular than ballot-level comparison. It can be
practical where ballot-level CVR linkage is unavailable, but large batches can
make risk evidence weaker and discrepancy localization less precise.

## References

- Arlo audit types: <https://docs.voting.works/arlo/audit-types>
- Arlo audit report guide: <https://docs.voting.works/arlo/resources/audit-report-guide>
