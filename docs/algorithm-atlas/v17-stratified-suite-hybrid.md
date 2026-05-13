# V.17 Stratified And Hybrid RLAs

## Mental Model

Hybrid audits appear when different parts of an election have different
available evidence. One jurisdiction may have ballot-level CVRs and support
ballot comparison; another may only support ballot polling. Stratified methods
combine evidence across those strata while preserving a single risk limit.

Arlo names this surface as hybrid RLA support. The older SUITE framing combines
stratum-level P-values; newer SHANGRLA/ALPHA approaches can often express the
same goal more directly.

## How RCOUNT Uses It

```text
stratum A comparison + stratum B polling -> combined risk decision
```

RCOUNT should use this page as the package-level coordinator when a single
contest spans multiple audit methods.

## Step-By-Step Mechanics

1. Partition ballots or jurisdictions into strata.
2. Record each stratum's audit method and sampling frame.
3. Compute stratum-level evidence or assertion-level martingales.
4. Combine evidence using the declared combining rule.
5. Preserve nuisance-parameter choices or maximization records when required.
6. Report the contest-level decision and stratum-level boundaries.

## RCOUNT Transcript Fields

| Field | Meaning |
|---|---|
| `method_id` | `stratified-hybrid-rla-v1` |
| `stratum_id` | jurisdiction, ballot group, or evidence family |
| `stratum_method` | ballot-polling, ballot-comparison, batch-comparison |
| `stratum_ballots` | sampling-frame size |
| `allocation_ppm` | stratum risk/allocation share in ppm |
| `stratum_risk` | stratum-level risk evidence |
| `combining_rule` | SUITE, SHANGRLA/ALPHA, Fisher, or registered rule |
| `nuisance_parameter` | value or optimization transcript if applicable |
| `combined_risk` | contest-level risk measure |

## Fixtures

- Two-stratum toy audit: one batch comparison component run and one Minerva
  ballot-polling component run, coordinated by
  `synthetic_stratified_hybrid_package`.
- Nuisance/allocation boundary fixture: the synthetic coordinator records a
  nuisance parameter and two 500,000 ppm stratum allocations.
- Flattened-stratum negative fixture: a malformed package collapses the hybrid
  coordinator to one 1,000,000 ppm stratum and is rejected.
- Hybrid Arlo-style fixture if public artifacts expose both strata.

## Current Implementation

RCOUNT can preserve the structure of a stratified/hybrid audit without claiming
combined-risk math:

- `AuditAlgorithmRun` can carry `combining_rule_id`;
- `AuditAlgorithmRun` can carry a rational `nuisance_parameter`;
- `AuditAlgorithmRun.strata` records `stratum_id`, stratum method, component
  run id, optional ballot count, allocation ppm, and source refs;
- core verification rejects missing component runs, duplicate/empty strata,
  unsupported stratum methods, invalid allocations, flattened one-stratum
  coordinators, and stratified runs with no combining rule or nuisance
  parameter;
- `rcount replay-audit-algorithms` reports `stratified-hybrid-rla-v1` as a
  method-specific boundary.

## Claim Boundary

The combined result is only as good as the stratum definitions and sampling
frames. RCOUNT must not flatten strata and pretend one homogeneous sampler was
used. Current packages may claim stratum preservation and component-link
verification, but not combined-risk replay.

## References

- Arlo audit types: <https://docs.voting.works/arlo/audit-types>
- ALPHA paper: <https://arxiv.org/abs/2201.02707>
- SHANGRLA paper: <https://arxiv.org/abs/1911.10035>
