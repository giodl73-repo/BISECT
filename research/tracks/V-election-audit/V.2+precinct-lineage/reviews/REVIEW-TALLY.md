# Simulated Review: TALLY

> AI-generated quality-improvement simulation, not real peer review.

## Score

3 / 4

## Summary

The paper keeps tally semantics mostly intact by separating lineage records from
contest summaries. That is the right move: a reporting-unit crosswalk should
not silently rewrite votes. The district aggregation hash triple is also a
strong control.

## Major Issues

1. **Unit universe hash needs a definition.** The paper uses the term but does
   not say which records are covered. It should state that the hash binds the
   reporting-unit ids used by aggregation for a cycle, and should be versioned
   by the package/plan transcript contract.

2. **Lineage is structural, not conservation of votes.** The term
   `lineage_conservation` may sound like vote conservation across elections.
   Explain that it checks referenced units and split/merge cardinality, not that
   vote totals should be equal across cycles.

3. **Vote-center and central-count units need a caveat.** Some units are not
   precinct-native. V.2 should say lineage can connect reporting units, not only
   geographic precinct polygons.

## Minor Issues

- Add one example of a stale plan failing before totals are aggregated.
- Say that V.8 owns district aggregation in depth.
- Mention that CVR/batch lineage is future work.

## Strengths

- Correctly avoids label matching.
- Good cycle-level hash binding.
- Negative cases are well chosen.
