---
title: "2030 Reapportionment Projections and Algorithmic Stability"
series: Q.4
status: Planned
date: 2026-05-09
track: Q-forward-2030
---

## Claims
1. Running Huntington-Hill apportionment on Census Bureau medium-series population projections gives a projected 2030 seat allocation. The projection is stable across low/medium/high series for 44 states; 6 states are on the boundary (gain or hold a seat depending on the series).
2. Cross-census algorithmic stability: comparing the 2020 bisect plan to the projected 2030 bisect plan (using projected populations), the mean county disruption is approximately 58% — lower than the historical 2010-2020 average (66%) because Sun Belt growth is more concentrated and predictable.
3. States with the highest expected boundary stability 2020-2030: VA (slow growth, stable geography), MD (stable suburbs), NE (flat plains, stable). States with lowest stability: TX (fast suburb growth), FL (coastal development), AZ (desert suburb expansion).
4. The DIA's algorithmic consistency provision — that the same parameters must be used in 2030 as in 2020 — is satisfied by the bisect version-pinning mechanism (`.bisect.lock` file). Cross-census parameter consistency is demonstrable from the audit chain.
