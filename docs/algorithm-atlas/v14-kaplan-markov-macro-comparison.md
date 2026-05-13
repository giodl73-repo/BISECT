# V.14 Kaplan-Markov / MACRO Comparison Audits

## Mental Model

Comparison audits compare a voting system's cast vote record to a human
interpretation of the same paper ballot or batch. The key statistic is not just
who appears on sampled ballots; it is the overstatement error: how much the
machine record overstated the reported margin.

Kaplan-Markov and MACRO-style methods turn those overstatement errors into a
risk-limiting P-value.

## How RCOUNT Uses It

```text
CVR -> sampled paper ballot -> human interpretation -> overstatement error -> P-value
```

RCOUNT should use this family for ballot-level comparison audits and batch
comparison audits when sampled units have reported totals and hand-audited
totals.

## Step-By-Step Mechanics

1. Bind each sampled ballot or batch to the matching CVR/reporting-unit record.
2. Compute reported margin for each audited assertion.
3. Compute overstatement error from CVR/reported interpretation versus hand
   interpretation.
4. Convert overstatement errors to taints or bounded error terms.
5. Update the Kaplan-Markov P-value.
6. Stop when the P-value is below the risk limit, otherwise continue.

## RCOUNT Transcript Fields

| Field | Meaning |
|---|---|
| `method_id` | `kaplan-markov-comparison-v1` |
| `sample_unit_id` | ballot or batch id |
| `cvr_selection` | voting-system interpretation |
| `human_selection` | audit-board interpretation |
| `margin` | reported assertion margin |
| `overstatement` | margin overstatement from discrepancy |
| `taint` | normalized overstatement term |
| `p_value` | running risk measure |
| `decision` | pass, continue, or full-count-required |

## Fixtures

- Package-level boundary fixture for `kaplan-markov-comparison-v1`.
- No-error ballot-comparison fixture that stops.
- One-vote overstatement fixture that still stops later.
- Two-vote overstatement fixture that fails or escalates.
- Batch-comparison fixture with hand tally versus reported batch tally.

## Current Implementation

`rcount-stats` now owns the first exact comparison primitive:

- `PluralityComparisonSelection::{Winner, Loser, Other}`;
- `PluralityComparisonObservation`;
- `plurality_winner_loser_overstatement`;
- exact margin contributions where winner is `+1`, loser is `-1`, and other is
  `0`;
- overstatement values from `-2` through `2`.
- exact taint normalization with `overstatement_taint(overstatement,
  reported_margin)`.
- an initial `replay_kaplan_markov_taint_product` transcript that accumulates
  exact rational taints into a running P-value-like product, records
  per-sample `p_value_ppm`, and marks pass/continue against `risk_limit_ppm`.
- `replay_kaplan_markov_macro_bound`, an exact rational implementation of the
  published MACRO Kaplan-Markov product shape using ballot count `N`, reported
  margin `V`, gamma, and one-/two-vote overstatement categories.

`rcount replay-audit-algorithms` now replays package-level
`kaplan-markov-comparison-v1` runs in two forms. If the run declares
`macro_ballot_count`, `macro_reported_margin`, and `macro_gamma`, RCOUNT uses
the published MACRO product primitive and treats `assorter_value` as the
integer overstatement category for each sampled unit. Otherwise, it preserves
the earlier taint-product replay, where each sample step carries a taint in
`assorter_value`. Incomplete runs still produce a method-specific boundary.
Core package verification rejects partial MACRO design declarations and gamma
values that are not greater than one.
`batch-comparison-v1` reuses this same taint-product replay surface after its
package-level overstatement arithmetic is verified.
`synthetic_kaplan_markov_macro_package` is the reusable in-repo fixture for the
MACRO path; it verifies in core, round-trips through IO, and replays through the
CLI.

The taint-product fallback is intentionally conservative and narrow: negative
taints do not reduce the running product. The MACRO path is the preferred
published-formula replay when the package has the design inputs needed to bind
`N`, `V`, and gamma.

## Claim Boundary

Comparison audits require reliable ballot/CVR or batch/reported-total linkage.
RCOUNT can verify the arithmetic once the link exists, but it cannot prove the
physical ballot was retrieved correctly unless source artifacts support that
chain of custody.

## References

- Arlo audit types: <https://docs.voting.works/arlo/audit-types>
- Super-Simple simultaneous audits: <https://www.stat.berkeley.edu/~stark/Preprints/superSimple10.pdf>
- SOBA context for ballot-level linkage: <https://arxiv.org/abs/1105.5803>
