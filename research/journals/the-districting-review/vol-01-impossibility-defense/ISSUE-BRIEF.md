---
journal: The Districting Review
volume: 1
title: "The Impossibility Defense"
status: seed
arc-position: 5 of 10 — pivots from "here's the problem" to "here's the solution"
reader-promise: "The algorithm cannot gerrymander — not because we trust it, but because it is architecturally incapable of receiving partisan data. This volume explains why that matters and how to say it in court."
target-reader: Redistricting commissioners, legal staff, practitioners who will use the algorithm or defend its use
excluded-claims: No outcomes claims (that's District Studies Vol 2). No adoption mechanics (that's Vol 3).
terminal-connection: The impossibility defense is the foundation of the DIA's constitutional validity. If the algorithm cannot gerrymander, mandating its use does not constitute partisan favoritism.
---

## Reader Promise (expanded)

The standard objection to algorithmic redistricting is: "algorithms can be tuned to produce any outcome." This volume answers that objection directly and completely. The bisect algorithm does not receive partisan data. It cannot see where Democrats and Republicans live. Its parameter space has been tested adversarially and produces at most 0.3 seats of national variation — within the noise floor of geography itself. Pre-registration of parameters before the Census data is released eliminates even this residual. The impossibility defense is not a trust argument; it is an architectural argument.

## Slots (6-8 candidates, 2-3x pool)

| Slot | Role | Claim class | Candidate papers |
|------|------|-------------|-----------------|
| Editorial | The trust problem — why "trust us" doesn't work and why architecture does | interpretive | — |
| Formula | The impossibility argument — no partisan input → no partisan output | formal | B.17 (parameter sensitivity), R.1 (parameter gaming) |
| Anchor | Seed sensitivity: CV < 2% for 96% of states — results are stable, not cherry-picked | empirical | B.7 (solution space) |
| Parameter gaming | Adversarial test: most-partisan parameter combination = 0.3 seats nationally | empirical | B.17, R.1 |
| Pre-registration | The DIA seed formula — parameters locked before Census data, SHA-256 binding | operational | R.1 §pre-registration, A.4 §software |
| Legal framing | How to use the impossibility argument in court — the Rucho context | interpretive | H.0 §legal posture |
| Measurement note | What the algorithm CAN vary — compactness range, not partisan direction | empirical | B.0 (bakeoff), H.0 §results |

## Review lenses

- HERALD: "Architectural incapability" needs plain-language translation for a commissioner audience
- LOKI: Do not claim the algorithm is provably neutral in all possible configurations — claim it cannot access the inputs required to gerrymander
- AXIOM: B.7's CV < 2% claim should be stated with its exact scope (congressional districts, 2020 Census)
