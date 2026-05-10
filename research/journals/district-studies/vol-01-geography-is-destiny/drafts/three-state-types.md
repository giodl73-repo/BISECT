---
journal: District Studies
volume: 1
title: "Three State Types"
status: draft-note
updated: 2026-05-09
claim-class: empirical/interpretive
source: B.12 ProportionalSection
review-gate: MERIDIAN/SCALE/PRECINCT
---

# Three State Types

## Draft Claim

B.12 is most useful to a public reader when it is not presented as a single
formula with one universal answer. The better frame is state types:

> Geography changes how much room an algorithm has to move. In some states, a
> proportionality constraint can move the result toward proportionality. In
> others, the same kind of constraint can worsen the result or create
> overcorrection. The public lesson is that geographic constraint is real and
> state-specific.

## Candidate Structure

### Type 1: Narrow Improvement Window

Some states may have a narrow setting where a proportionality constraint moves
the map closer to the expected partisan seat split. This is not evidence of a
general knob that can be turned everywhere. It is evidence that geography and
parameter choice interact.

### Type 2: Constraint-Induced Worsening

Some states can become worse under an added partisan constraint. That matters
because it keeps the issue honest: adding partisan information does not
automatically make a map fairer or more proportional.

### Type 3: Granularity And Overcorrection

States with small delegations or coarse partition choices can have too little
room for a smooth adjustment. A constraint can then produce a jump rather than a
small correction.

## Why This Fits Vol. 1

The case-study slot should teach the reader that geography is not a slogan. It
is a set of constraints that can produce different behavior in different
states. A three-type structure supports the issue thesis without pretending the
research has one universal result.

## Public Example

Imagine two states with the same statewide vote split. In one state, voters from
each party are spread fairly evenly across the map. In the other, one party's
voters are packed into a few dense urban areas while the other party's voters
are spread across many suburbs and rural counties. A compact districting rule
will face different choices in those two states. The statewide vote number is
the same, but the geography is not, so the seat outcome may not be the same.

## What Must Wait

Exact state values and table cells should wait for B.12 source-chain closure:

- METIS `niter`, `ncuts`, and `numbering`.
- Table 1 seed treatment.
- Range, SD, or deterministic-seed statement.
- C(G) estimator provenance.

## Review Questions

- PRECINCT: Does the type structure distinguish geographic sorting from
  partisan manipulation?
- SCALE: Does the piece avoid exact numeric claims before source-chain closure?
- MERIDIAN: Are the state examples faithful to B.12's actual algorithmic setup?
- COMMONS: Can the reader understand why a non-universal answer is a strength,
  not a weakness?
