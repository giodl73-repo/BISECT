---
journal: The Districting Review
volume: 1
title: "The Input-Exclusion Defense"
status: provisional-public-preview
arc-position: 5 of 10 -- pivots from "here's the problem" to "here's the solution"
reader-promise: "Bisect's core claim is not trust. It is input exclusion: the baseline run does not ingest partisan data. This volume tests what that does and does not prove for practitioners."
target-reader: Redistricting commissioners, legal staff, practitioners who will use the algorithm or defend its use
excluded-claims: No outcomes claims (that's District Studies Vol 2). No adoption mechanics (that's Vol 3). No universal impossibility claim.
terminal-connection: The input-exclusion defense is one foundation for evaluating candidate adoption language. It is not, by itself, a constitutional conclusion.
---

> Audition status: court-facing neutrality language requires BOUNDARY, WARD, COVENANT, and SCALE review before publication.

## Reader Promise (expanded)

The standard objection to algorithmic redistricting is: "algorithms can be tuned to produce any outcome." This volume takes that objection seriously. The baseline Bisect workflow does not receive partisan data, so it cannot directly optimize for partisan outcome. The remaining questions are empirical and legal: how much variation exists across allowed parameters, whether pre-registration controls that variation, and how a court or commission should describe the defense without overselling it.

## Slots (6-8 candidates, 2-3x pool)

| Slot | Role | Claim class | Candidate papers |
|------|------|-------------|-----------------|
| Editorial | The trust problem -- why "trust us" doesn't work and why architecture does | interpretive | -- |
| Formula | The input-exclusion argument — no partisan input in the baseline workflow | formal | B.17 (parameter sensitivity), R.1 (parameter gaming) |
| Anchor | Seed sensitivity: CV < 2% for 96% of states — results are stable, not cherry-picked | empirical | B.7 (solution space) |
| Parameter gaming | Adversarial test: most-partisan parameter combination = 0.3 seats nationally | empirical | B.17, R.1 |
| Pre-registration | The DIA seed formula -- parameters locked before Census data, SHA-256 binding | operational | R.1 §pre-registration, A.4 §software |
| Legal framing | How to use the input-exclusion defense in court -- the Rucho context | interpretive | H.0 §legal posture |
| Measurement note | What the algorithm CAN vary — compactness range, not partisan direction | empirical | B.0 (bakeoff), H.0 §results |

## Review lenses

- COMMONS: "Architectural incapability" needs plain-language translation for a commissioner audience
- BOUNDARY: Do not claim the algorithm is provably neutral in all possible configurations -- claim the baseline workflow excludes partisan inputs and then audit residual variation.
- SCALE: B.7's CV < 2% claim should be stated with its exact scope (congressional districts, 2020 Census)
