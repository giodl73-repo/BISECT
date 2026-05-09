---
title: "Input Data Manipulation: Census Data, GEOID, and Shapefile Integrity"
series: R.2
status: Planned
date: 2026-05-09
track: R-adversarial-robustness
---

## Claims
1. The bisect pipeline inputs — Census P.L. 94-171 file, TIGER/Line shapefiles, ACS tables — are all downloaded from Census Bureau servers with SHA-256 verification. Any modification is immediately detectable.
2. An adversarial agent who controls the state's data download infrastructure (man-in-the-middle) could substitute manipulated data, but this requires either (a) corrupting a federal government server, or (b) intercepting HTTPS traffic — both of which are federal crimes and technically infeasible in the DIA's `bisect fetch` design (certificate pinning).
3. Even if population counts were manipulated by ±1% per tract, the maximum partisan shift is bounded by B.7's CV analysis: with CV < 2%, a ±1% manipulation is within the noise floor and undetectable as manipulation.
4. The audit chain provides provenance at three levels: (a) Census Bureau download timestamp and SHA-256, (b) adjacency graph construction hash, (c) final plan assignment SHA-256. Any manipulation at any level breaks the chain.
