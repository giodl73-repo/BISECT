# V.16 SHANGRLA Assorter Audits

## Mental Model

SHANGRLA turns election outcomes into assertions about averages. Each assertion
uses an assorter: a function that maps a ballot, CVR, or comparison record to a
number. If the mean is high enough, the assertion supports the reported outcome.

This is the cleanest abstraction for RCOUNT's audit algorithm layer.

## How RCOUNT Uses It

```text
reported outcome -> assertions -> assorters -> mean tests -> audit transcript
```

RCOUNT should use SHANGRLA as the assertion language beneath BRAVO, ALPHA,
comparison audits, approval/scoring rules, and some ranked-choice audit paths.

## Step-By-Step Mechanics

1. Convert the reported outcome into one or more assertions.
2. Define an assorter for each assertion.
3. Bind each sampled record to an assorter value.
4. Run a valid mean test, such as ALPHA or another risk-limiting test.
5. Combine assertion decisions into an outcome decision.
6. Emit assertion-level transcripts so failures are local and explainable.

## RCOUNT Transcript Fields

| Field | Meaning |
|---|---|
| `method_id` | `shangrla-assorter-v1` |
| `assertion_id` | one outcome assertion |
| `assorter_formula` | stable formula or registered function id |
| `upper_bound` | assorter bound |
| `sample_unit_id` | ballot, CVR, or batch id |
| `assorter_value` | value used by the mean test |
| `test_method` | ALPHA, Kaplan-Markov, or other mean test |
| `assertion_status` | pass, continue, fail |

## Implementation Status

The first RCOUNT substrate slice has landed: `rcount-core` defines audit method
ids, `AuditAlgorithmRun`, `AuditAssertion`, `AuditSampleStep`, rational
assorter values, and `verify_audit_algorithm_runs`. The verifier checks
transcript shape and value bounds, but it does not yet compute SHANGRLA mean
tests or ALPHA martingales.

## Fixtures

- Two-candidate plurality assertion.
- Multi-winner assertion set with one failing assertion.
- Comparison-audit assorter fixture using CVR and hand interpretation.
- Approval/scoring-rule fixture once RCOUNT supports those contest semantics.

## Claim Boundary

SHANGRLA is an assertion framework, not a custody system. It says how to reduce
an outcome claim to statistical tests, but RCOUNT still needs source hashes,
ballot manifests, CVR linkage, and sampler replay to make the transcript
independently meaningful.

## References

- SHANGRLA paper: <https://arxiv.org/abs/1911.10035>
- ALPHA paper: <https://arxiv.org/abs/2201.02707>
