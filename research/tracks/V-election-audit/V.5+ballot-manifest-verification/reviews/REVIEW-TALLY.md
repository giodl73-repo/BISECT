# Simulated Review: TALLY

> AI-generated quality-improvement simulation, not real peer review.

## Score

2 / 4

## Summary

The draft has the right accounting instinct, but it needs stronger election
semantics before it can anchor ballot manifest verification.

## Major Issues

1. **Separate accounting objects.** A batch is not a ballot, ballot card, CVR
   row, contest row, selection, voter, or vote. The paper says this once, but
   it should become a central contract.

2. **Model batch kinds more concretely.** Mail, provisional, duplicated,
   central-count, vote-center, and scanner batches can all carry different
   meanings and reconciliation rules.

3. **Clarify contest scope.** `counted_ballots` reconciles a batch container;
   it does not prove every contest on every ballot card has a CVR row.

## Minor Issues

- Mention overvotes, undervotes, blanks, write-ins, and adjudication as V.6
  concerns.
- Clarify whether the same physical batch can produce multiple contest
  summaries.
- Add a failure case for duplicate batch ids in a later fixture.

## Strengths

- Good positive/negative fixture pair.
- Correctly treats manifests as accounting controls.
- Does not overclaim tabulator correctness.
