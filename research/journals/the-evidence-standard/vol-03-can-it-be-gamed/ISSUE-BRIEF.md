---
journal: The Evidence Standard
volume: 3
title: "Can It Be Gamed?"
status: seed
arc-position: 10 of 10 — closes the adversarial objection, enables the federal rule
reader-promise: "We tested every way a hostile actor could manipulate the algorithm — parameters, input data, geographic definitions, selective presentation, version selection. The maximum combined effect is less than half a congressional seat nationally. All of it is detectable. The answer is: not meaningfully."
target-reader: Opposing counsel who will argue the algorithm can be gamed, federal legislators who will hear this objection in markup
excluded-claims: No adoption pathway arguments (that's The Districting Review Vol 3). This volume closes the adversarial objection, not the affirmative case.
terminal-connection: This is the last objection before the federal rule vote. "It can be gamed" is the strongest argument against the DIA. This volume answers it quantitatively, formally, and with Rust code that anyone can run.
---

## Reader Promise (expanded)

The most sophisticated objection to algorithmic redistricting is not "it produces bad maps" but "it can be gamed." If a hostile state can manipulate algorithm parameters, input data, or geographic definitions to produce a partisan outcome while claiming algorithmic neutrality, then the DIA is a Trojan horse. This volume answers the objection systematically. Five gaming vectors are identified. Each is tested empirically. The combined maximum effect under DIA pre-registration constraints is ≤ 0.5 seats nationally — within geographic noise. All five vectors are detectable via the audit chain. The Gaming Impossibility Theorem (R.0 Theorem 3.1) provides the formal bound.

## Slots (6-8 candidates, 2-3x pool)

| Slot | Role | Claim class | Candidate papers |
|------|------|-------------|-----------------|
| Editorial | The "Trojan horse" objection — take it seriously before answering it | interpretive | — |
| Taxonomy | Five gaming vectors, their attack surfaces, their DIA protections | operational | R.0 (gaming taxonomy) |
| Parameter gaming | Adversarial B.17 — most-partisan parameters = 0.3 seats, detectable | empirical | R.1 (parameter gaming) |
| Input data | SHA-256 bounds input manipulation — ±1% population = noise floor | empirical | R.2 (input data manipulation) |
| Geography | Resolution choice (0.18 seats) + adjacency manipulation (0.15 seats) | empirical | R.3 (geographic gaming) |
| Combined | Gaming Impossibility Theorem — ≤ 0.5 seats combined, correlation caveat | formal | R.0 §combined, R.0 §correlation-caveat |
| Formal close | Pre-registration is the master defense — lock parameters before Census, SHA-256 binding | operational | R.1 §preregistration, A.4 §sha-256 |

## Arc Note

This is the terminal volume. It ends with pre-registration as the master defense — a design feature of the DIA that makes gaming impossible in practice. The reader who has completed all ten volumes now has:

1. Why the problem is structural (District Studies Vols 1-4)
2. What the algorithm produces (The Districting Review Vols 1-2)
3. How to adopt it (The Districting Review Vol 3)
4. How to prove it in court (The Evidence Standard Vols 1-2)
5. That it cannot be gamed (The Evidence Standard Vol 3)

The conclusion is the federal rule.

## Review lenses

- HERALD: The adversarial framing ("take it seriously") must be genuine — don't dismiss the objection before answering it
- LOKI: The 0.5-seat bound is sub-additive (correlated vectors) — the correlation caveat from R.0 must be stated
- AXIOM: Gaming Impossibility Theorem — formal statement must match R.0's actual theorem, not a paraphrase
