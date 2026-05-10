---
journal: The Evidence Standard
volume: 3
title: "Can It Be Gamed?"
status: seed
arc-position: 10 of 10 — tests the adversarial objection before any candidate rule
reader-promise: "The strongest objection is that an algorithmic baseline can be gamed. This volume catalogs the attack surface and tests which defenses survive adversarial review."
target-reader: Opposing counsel who will argue the algorithm can be gamed, federal legislators who will hear this objection in markup
excluded-claims: No adoption pathway arguments (that's The Districting Review Vol 3). This volume tests the adversarial objection, not the affirmative case.
terminal-connection: This is the terminal objection before any candidate rule can be responsibly proposed. It must be answered quantitatively, formally, and reproducibly.
---

> Seed status: the gaming bound is a candidate claim. It cannot be public locked copy until TRENCH, SCALE, MERIDIAN, and BENCHMARK review agree on scope.

## Reader Promise (expanded)

The most sophisticated objection to algorithmic redistricting is not "it produces bad maps" but "it can be gamed." If a hostile actor can manipulate algorithm parameters, input data, geographic definitions, selective presentation, or version choice while claiming neutrality, then the baseline fails its public purpose. This volume identifies five gaming vectors and auditions empirical, operational, and formal defenses for each.

## Slots (6-8 candidates, 2-3x pool)

| Slot | Role | Claim class | Candidate papers |
|------|------|-------------|-----------------|
| Editorial | The "Trojan horse" objection — take it seriously before answering it | interpretive | — |
| Taxonomy | Five gaming vectors, their attack surfaces, their DIA protections | operational | R.0 (gaming taxonomy) |
| Parameter gaming | Adversarial U.2 — most-partisan parameters = 0.3 seats, detectable | empirical | R.1 (parameter gaming) |
| Input data | SHA-256 bounds input manipulation — ±1% population = noise floor | empirical | R.2 (input data manipulation) |
| Geography | Resolution choice (0.18 seats) + adjacency manipulation (0.15 seats) | empirical | R.3 (geographic gaming) |
| Combined | Candidate gaming bound — <= 0.5 seats combined, correlation caveat | formal | R.0 §combined, R.0 §correlation-caveat |
| Formal close | Pre-registration is the master defense — lock parameters before Census, SHA-256 binding | operational | R.1 §preregistration, A.4 §sha-256 |

## Arc Note

This is the terminal volume. It ends with pre-registration as the master defense candidate. The reader who has completed all ten volumes now has:

1. Why the problem is structural (District Studies Vols 1-4)
2. What the algorithm produces (The Districting Review Vols 1-2)
3. How to adopt it (The Districting Review Vol 3)
4. How to prove it in court (The Evidence Standard Vols 1-2)
5. An adversarial test of whether and how it can be gamed (The Evidence Standard Vol 3)

The conclusion is a reviewed recommendation, not a premise.

## Review lenses

- COMMONS: The adversarial framing ("take it seriously") must be genuine — don't dismiss the objection before answering it
- SCALE: The 0.5-seat bound is sub-additive (correlated vectors) — the correlation caveat from R.0 must be stated
- MERIDIAN: The candidate gaming-bound statement must match R.0's actual theorem, not a paraphrase
