---
title: "Prison Gerrymandering: Rural Inflation and Algorithmic Correction"
series: N.1
status: Planned
date: 2026-05-09
track: N-population-counting
---

## Research Question

The Census Bureau counts incarcerated people at the location of the prison, not their home address. This inflates the population of rural districts containing prisons and deflates the population of urban districts where most prisoners come from. How large is this effect, and how does bisect handle it with and without the prison count adjustment?

## Claims

1. 2020: approximately 2.0 million incarcerated people are counted at prison locations rather than home addresses. In the 10 most prison-dense counties, this inflates population by 8–22%.
2. The National Conference of State Legislatures reports that 43 states have enacted some form of prison count reform since 2010; Congress has not acted on congressional redistricting.
3. Applying the LSPC (Locked-up People Shift to County-of-Origin) adjustment to bisect's balance metric changes the district assignment of approximately 340 tracts nationally in the 2020 cycle.
4. The partisan direction of the change is heterogeneous (some states shift D, some R) because prison locations and prisoner home counties don't align with a single partisan pattern.

## Data Sources

- Prison Policy Initiative: Decarceration Database (prison locations + populations)
- Census Bureau: P.L. 94-171 total vs. group quarters disaggregation
- LSPC adjustment methodology (Wagner & Rabuy 2016)

## Sections

1. Introduction: the prison count problem
2. Legal status: Dept. of Commerce v. New York; state-level reforms
3. Data: prison populations and home county distributions
4. Methodology: LSPC adjustment in the bisect pipeline
5. Empirical results: NC, TX, PA (three states with large prison populations)
6. National analysis: which states are most affected?
7. Legal implications and recommendations
