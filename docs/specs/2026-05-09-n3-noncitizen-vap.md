---
title: "Noncitizen Populations and Citizen VAP Redistricting"
series: N.3
status: Planned
date: 2026-05-09
track: N-population-counting
---

## Research Question

Evenwel v. Abbott (2016) held that Texas could use total population for state legislative redistricting. Justice Ginsburg's majority opinion explicitly left open whether states *may* use citizen VAP. If a state switched to citizen VAP, which districts would change and what would the partisan effect be?

## Claims

1. Citizen VAP and total population diverge most sharply in high-immigration states (CA, TX, NY, FL). The difference exceeds 5 percentage points of district population in approximately 80 of 435 congressional districts.
2. Switching from total population to citizen VAP for *congressional* redistricting (which no state currently does) would shift representation from urban (immigrant-dense) to suburban districts.
3. The partisan direction is consistently Republican-advantaging: immigrant-dense urban districts are predominantly Democratic; the suburbs that would gain population are more competitive.
4. The bisect pipeline supports `--balance-metric citizen_vap` using ACS 5-year citizen VAP by tract. This is the first systematic 50-state analysis using citizen VAP as the congressional balance metric.

## Key Legal Note

Congressional redistricting is governed by Article I, which specifies "actual Enumeration" — this has been interpreted to mean total population. Citizen VAP redistricting for Congress would require either a constitutional amendment or a Supreme Court ruling overturning *Burns v. Richardson* (1966). State legislative redistricting is different: *Evenwel* directly applies.

## Data Sources

- ACS 5-year: B05001 (Nativity and Citizenship Status), B01001 (Age by Sex)
- Census Bureau: CVAP special tabulation (Citizen Voting-Age Population)
