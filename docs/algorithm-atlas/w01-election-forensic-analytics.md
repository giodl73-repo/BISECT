# W.01 Election Forensic And Anomaly Analytics

## Mental Model

Forensic analytics look for unusual patterns: outlying precinct swings, turnout
residuals, digit distributions, batch-level discontinuities, scanner effects,
or geographic anomalies. They are useful for triage and investigation, but they
do not certify an election outcome by themselves.

This belongs in a W-series because it is adjacent to RCOUNT verification but has
a different claim: detect anomalies, not confirm outcomes.

## How RCOUNT Uses It

```text
certified/public results -> normalized features -> anomaly score -> investigation queue
```

RCOUNT can provide clean inputs: source hashes, reporting-unit lineage,
normalized summaries, batches, CVRs, and audit results. W-series analytics can
then compute reproducible anomaly reports over those records.

## Analytics Families

| Family | What it asks | RCOUNT input |
|---|---|---|
| Turnout residuals | Is turnout unusual after accounting for history/demographics? | summaries, precinct lineage |
| Vote-share residuals | Are candidate shares unusual relative to comparable units? | summaries, CVRs |
| Batch/scanner effects | Do batches or scanners show systematic deviations? | batch manifests, CVRs |
| Digit tests | Do reported numbers have suspicious digit patterns? | summaries |
| Benford-style checks | Are leading-digit distributions unusual where applicable? | large count tables |
| Spatial outliers | Are neighboring units unexpectedly different? | reporting units, geography |
| Change-point tests | Did patterns shift abruptly across time or batches? | status events, batches |
| Audit discrepancy clustering | Do discrepancies cluster by source, machine, or batch? | audit observations |

## Transcript Fields

| Field | Meaning |
|---|---|
| `method_id` | registered forensic analytic |
| `feature_set` | variables used by the model |
| `baseline_population` | comparison units and time window |
| `model_id` | residual, digit test, spatial model, etc. |
| `unit_id` | reporting unit, batch, scanner, or contest |
| `score` | anomaly score or test statistic |
| `p_value_or_rank` | optional statistical calibration |
| `investigation_note` | machine-readable caveat |

## Fixtures

- Synthetic no-anomaly fixture.
- Injected outlier precinct fixture.
- Batch scanner-effect fixture.
- False-positive boundary fixture showing why anomaly is not proof.

## Claim Boundary

Forensic analytics can prioritize investigation. They cannot prove fraud,
certify correctness, or replace risk-limiting audits. Any W-series page must
lead with this boundary.

## Relationship To V-Series

V-series algorithms verify audit/canvass claims with explicit evidence
contracts. W-series algorithms explore patterns that may motivate more review.
RCOUNT should keep their transcripts separate so anomaly scores never masquerade
as certification evidence.
