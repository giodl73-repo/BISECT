# V.12 BRAVO Ballot-Polling RLA

## Mental Model

BRAVO is the classic ballot-polling risk-limiting audit. It samples paper
ballots, observes votes for the audited contest, and updates a sequential
likelihood ratio until the reported winner has enough evidence or the audit
continues to a larger sample or full hand count.

In RCOUNT, BRAVO is the simplest statistically meaningful bridge from sampled
ballot observations to a risk-limit transcript.

## How RCOUNT Uses It

```text
reported contest totals -> random sample -> ballot observations -> likelihood ratio -> stop/continue
```

RCOUNT should use BRAVO when the public artifact contains ballot-polling
observations but no ballot-level CVR comparison data. The RI Rep. 28 adapter is
near this surface, but its published report declares Minerva rather than BRAVO.

## Step-By-Step Mechanics

1. Read the contest, reported winner, loser, total ballots, and risk limit.
2. Compute reported winner and loser shares for the two-candidate assertion.
3. Replay the public-seed sample order when enough sampler metadata exists.
4. For each audited ballot, record whether the observation supports the winner,
   supports the loser, or is non-vote/other for that assertion.
5. Update the BRAVO likelihood ratio after each draw.
6. Stop when the likelihood ratio crosses the risk-limit threshold.
7. Emit a round transcript even when the audit does not stop.

## RCOUNT Transcript Fields

| Field | Meaning |
|---|---|
| `method_id` | `bravo-ballot-polling-v1` |
| `assertion_id` | winner-loser assertion being tested |
| `risk_limit_ppm` | declared risk limit |
| `reported_winner_share` | reported share used by the likelihood ratio |
| `draw_index` | sequential sample index |
| `observation` | winner, loser, invalid, undervote, other |
| `likelihood_ratio` | current test statistic |
| `stop_threshold` | risk-limit-derived threshold |
| `decision` | continue, stop, or full-count-required |

## Fixtures

- Positive tiny two-candidate crate fixture that stops after six winner
  observations with reported votes 3-1 and risk limit 10%.
- Negative/drift fixture target: declared statistic or P-value changes after
  replay.
- Continue fixture: observations exist and replay cleanly, but the likelihood
  ratio does not cross the stop threshold.
- Boundary fixture: observations exist but required reported vote or risk-limit
  fields are missing, so the statistical claim is preserved but not replayed.
- Real-data comparison fixture against an Arlo or state BRAVO report if one is
  published with enough per-draw evidence.

## Current Implementation

`rcount-stats` owns the exact BRAVO likelihood-ratio replay:

- `replay_bravo_ballot_polling`;
- `BravoObservation::{Winner, Loser, Other}`;
- exact rational likelihood ratios;
- P-value ppm computed from `1 / likelihood_ratio`;
- stop threshold computed from the declared risk limit.

`rcount-audit` adds `replay_audit_algorithm_statistics` for
`bravo-ballot-polling-v1`. The first replay surface maps `assorter_value = 1`
to winner, `0` to loser, and other nonnegative rational values to neutral
observations. It requires `reported_winner_votes`, `reported_loser_votes`, and
`risk_limit_ppm` on the run.

## Claim Boundary

BRAVO can confirm a reported plurality assertion under its sampling assumptions.
It does not prove ballot custody, voter eligibility, complete source-file
publication, or machine security. It also does not handle comparison-audit
overstatement errors; that belongs to Kaplan-Markov/MACRO-style methods.

## References

- Arlo audit types: <https://docs.voting.works/arlo/audit-types>
- Stark ballot-polling tools: <https://www.stat.berkeley.edu/~stark/Vote/ballotPollTools.htm>
- Super-Simple RLAs: <https://www.stat.berkeley.edu/~stark/Preprints/superSimple10.pdf>
