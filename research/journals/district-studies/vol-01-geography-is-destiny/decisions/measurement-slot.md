---
journal: District Studies
volume: 1
title: "Geography Is Destiny"
status: decision
updated: 2026-05-09
decision: defer-result-bearing-measurement-slot
---

# Measurement Slot Decision

## Decision

Defer the result-bearing measurement slot from District Studies Vol. 1 unless
L.1/C.5 outputs are regenerated or traced to a post-`81a57bbb` run.

Vol. 1 may include a short method note explaining why efficiency gap is staged,
but it should not publish C.5/L.1 numeric conclusions in the first public issue
until output provenance is closed.

## Reason

The code-level efficiency-gap sign convention issue is closed. The repository
now computes efficiency gap as:

```text
(Wasted_D - Wasted_R) / Total_votes
```

with positive values indicating Republican-favoring plans.

The publication problem is not the current formula. The publication problem is
provenance: older paper text, figures, tables, or generated outputs may have
been produced before the sign fix. If the issue uses numeric measurement claims,
readers need to know those numbers came from the corrected convention.

## Effect On Lineup

The public-safe Vol. 1 lineup should be:

1. Editorial: The Shape of the Vote.
2. Geography Constrains Proportionality.
3. The Ensemble Median Case.
4. Three State Types or a short bridge note.
5. What This Issue Does Not Show.

The measurement slot becomes:

- a deferred Vol. 2 candidate; or
- a short Vol. 1 method note with no result-bearing numeric claims.

## Reopen Criteria

Reopen the measurement slot for Vol. 1 only if one of these is true:

- C.5/L.1 outputs are regenerated under the current implementation.
- A post-`81a57bbb` artifact is located and linked.
- The article is rewritten as a methods-only note with no directional numeric
  claims.

## Queue Impact

DS1-006 is closed as a decision. Any future work on measurement outputs should
be tracked as a new provenance task, not as a blocker for the first issue frame.
