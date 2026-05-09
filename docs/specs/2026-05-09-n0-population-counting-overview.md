---
title: "The Balance Metric: Who Counts in Congressional Redistricting?"
series: N.0
status: Planned
date: 2026-05-09
track: N-population-counting
---

## Research Question

Reynolds v. Sims (1964) requires equal population across congressional districts. But "population" is not defined by the Constitution — and different definitions produce measurably different maps. This paper surveys the legal landscape and empirical magnitude of population definition choices.

## Claims

1. Five distinct population definitions are used or proposed for redistricting: total population, total voting-age population (VAP), citizen VAP, registered voters, and actual voters. Each produces different district boundaries for the same state.
2. The difference between total population and citizen VAP exceeds 5% in 12 states, sufficient to change district assignments for thousands of tracts.
3. Evenwel v. Abbott (2016) left total population permissible but citizen VAP unresolved. This ambiguity creates strategic litigation risk.
4. The bisect pipeline's `--balance-metric` flag supports all five definitions; default is total population.

## Data Sources

- Census P.L. 94-171 redistricting file (total population, VAP)
- ACS 5-year: citizen VAP by tract
- Evenwel v. Abbott briefing; amicus briefs

## Sections

1. Introduction: the population ambiguity
2. Legal landscape: Reynolds through Evenwel
3. Five population definitions and their statutory basis
4. Empirical magnitude: where do the definitions diverge?
5. Algorithmic implications: how bisect handles each
6. Conclusion: a statutory recommendation

## Output

50-state table showing the difference between total population and citizen VAP by state. Legal analysis of which definition is required, permitted, or prohibited.
