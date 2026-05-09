---
title: "Sun Belt Growth and Rust Belt Decline: Redistricting Implications"
series: Q.2
status: Planned
date: 2026-05-09
track: Q-forward-2030
---

## Claims
1. The demographic shift from Rust Belt to Sun Belt between 2020 and 2030 creates the largest cross-cycle reapportionment since 1990. The bisect plans for gaining states will have different compactness profiles than their 2020 predecessors.
2. TX k=41 (projected) vs k=38: the prime factorization shifts from 2×19 to prime (41 is prime), requiring a full single 41-way partition as the first bisection step — a significant algorithmic change.
3. In Sun Belt states, population growth is concentrated in suburban ring counties around major metros (Phoenix suburbs, Houston suburbs, Atlanta suburbs). This creates new bisection challenges: the metro-suburb boundary is the natural first cut but the suburb is growing faster than the city.
4. The 2030 bisect plans for TX, FL, and AZ will likely show more compact metropolitan clusters than the 2020 plans, because rapid suburban growth makes previously irregular suburban districts more population-dense and thus more geometrically regular.

## Implications for Algorithm
Q.2 feeds directly into bisect pipeline changes: ApportionRegions needs to handle k=41 (prime, 41-way partition). This may require a switch to GeoSection for TX in 2030 (which handles any k via ratio-optimal bisection) rather than the prime-factor tree.
