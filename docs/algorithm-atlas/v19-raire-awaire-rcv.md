# V.19 RAIRE And AWAIRE For RCV / IRV Audits

## Mental Model

Ranked-choice and instant-runoff elections do not reduce cleanly to one
winner-loser plurality margin. RAIRE builds a set of assertions sufficient to
confirm an IRV outcome, often using CVRs. AWAIRE explores adaptive weighted
audits for IRV without requiring the same CVR assumptions.

RCOUNT needs this family before it can seriously support RCV jurisdictions.

## How RCOUNT Uses It

```text
ranked CVRs -> IRV outcome -> assertion set -> sampled ballots/CVRs -> RCV audit decision
```

This should likely live after RCOUNT has ranked-choice contest semantics,
ranked CVR records, and SHANGRLA/ALPHA assertion machinery.

## Step-By-Step Mechanics

1. Parse ranked CVRs and the reported IRV elimination order.
2. Generate RAIRE assertions or AWAIRE audit assertions.
3. For each sampled ballot, compute the assertion-specific evidence.
4. Run the declared risk test for each assertion.
5. Confirm the outcome only when all required assertions pass.
6. Report which assertion remains the bottleneck if the audit continues.

## RCOUNT Transcript Fields

| Field | Meaning |
|---|---|
| `method_id` | `raire-irv-v1` or `awaire-irv-v1` |
| `rcv_contest_id` | ranked-choice contest |
| `elimination_order` | reported IRV order |
| `assertion_id` | RAIRE/AWAIRE assertion |
| `ranked_ballot_value` | assertion value from sampled ballot/CVR |
| `assertion_risk` | risk measure for that assertion |
| `bottleneck_assertion` | assertion controlling sample size |
| `decision` | pass, continue, or full-count-required |

## Fixtures

- RAIRE boundary fixture with reported IRV elimination order and ranked sample
  choices: `synthetic_raire_boundary_package`.
- AWAIRE boundary fixture with the same ranked-choice evidence surface:
  `synthetic_awaire_boundary_package`.
- Malformed ranked-choice negative fixture with a duplicate ranked choice:
  `synthetic_bad_raire_boundary_package`.
- Tiny IRV contest with known elimination order.
- Assertion-generation fixture where one assertion is intentionally wrong.
- CVR comparison fixture for ranked ballots.
- Ballot-polling RCV fixture once AWAIRE support is scoped.

## Current Implementation

RCOUNT can preserve RAIRE/AWAIRE method identity and ranked sample evidence
without claiming ranked-choice audit math:

- `AuditAlgorithmRun.rcv_elimination_order` records the reported IRV
  elimination order;
- `AuditSampleStep.ranked_choices` records sampled ranked choices;
- core verification rejects duplicate ranked choices, unknown ranked choices,
  missing ranked samples, and duplicate elimination-order candidates;
- `rcount replay-audit-algorithms` reports `raire-irv-v1` and `awaire-irv-v1`
  as method-specific boundaries.

## Claim Boundary

RAIRE/AWAIRE audit the reported outcome under ranked-choice rules. They do not
make RCOUNT a general RCV tabulator unless the package also contains a verified
IRV tabulation path and ranked ballot semantics. Current support preserves the
evidence surface; it does not generate RAIRE assertions, run AWAIRE adaptive
logic, or certify the IRV tabulation.

## References

- RAIRE paper: <https://arxiv.org/abs/1903.08804>
- AWAIRE paper: <https://arxiv.org/abs/2307.10972>
