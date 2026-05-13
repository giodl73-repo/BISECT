# J — Apportionment Methods

**Theme**: Mathematical analysis of congressional seat apportionment methods —
how seats are allocated to states before districts are drawn. Documents the
`bisect-apportion` crate and compares all major divisor and quota methods.

## Track Chain

J.0 (overview) → J.1 (Huntington-Hill) → J.2 (Webster) → J.3 (Adams)
              → J.4 (Jefferson/D'Hondt) → J.5 (paradoxes) → J.6 (implementation)

## Papers

| Paper | Title | Stage | Score |
|-------|-------|-------|-------|
| J.0 | Apportionment Methods: Overview | planned | — |
| J.1 | Huntington-Hill: The Federal Method | planned | — |
| J.2 | Webster Method and the Sainte-Laguë Connection | planned | — |
| J.3 | Adams Method and Smallest-State Bias | planned | — |
| J.4 | Jefferson/D'Hondt and Largest-State Bias | planned | — |
| J.5 | Apportionment Paradoxes | planned | — |
| J.6 | bisect-apportion: Implementation and Verification | planned | — |

## Contracts

J.1: Prove Huntington-Hill minimises relative deviation from quota.
J.5: Show which of {Alabama, population, new-states} paradoxes each method
  is susceptible to; prove HH is paradox-free for fixed house size.
J.6: Document the current bisect-apportion library boundary, including four
  divisor methods, paradox helpers, and missing Census-verification fixtures.

## Quantification

- Primary: 2020 apportionment seat allocation for all 50 states under each method
- Deviation: max relative deviation from Hamiltonian quota per method
- Bias: small-state vs. large-state seat-share deviation from population share
