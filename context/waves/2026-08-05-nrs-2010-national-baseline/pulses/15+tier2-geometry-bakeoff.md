---
pulse: 15
title: Tier 2 geometry bakeoff
status: complete
wave: nrs-2010-national-baseline
validation_level: L2 governed empirical evidence
---

# Pulse 15 - Tier 2 Geometry Bakeoff

## Purpose

Extend the accepted Tier 1 Rhode Island comparison with common-universe
geometry metrics without changing either plan assignment or using original
comparator linework asymmetrically.

## Deliverables

- [x] Audit existing compactness implementations and Tier 1 inputs.
- [x] Freeze block-universe, projection, dissolve, water, and perimeter rules.
- [x] Freeze metrics, failures, claim boundary, and national expansion gate.
- [x] Add the bounded analyzer, verifier, and synthetic tests.
- [x] Execute and exactly regenerate the Rhode Island evidence package.
- [x] Publish the result and stop/advance decision.

## Result

Both plan families dissolved all 24,831 retained blocks into two valid
district geometries without repair. The unweighted district means were:

| Metric | NRS v0.3 | Enacted CD118 block projection |
|---|---:|---:|
| Polsby--Popper | 0.013973094 | 0.019782611 |
| Exact Reock | 0.298088463 | 0.231058299 |
| Convex-hull ratio | 0.530032258 | 0.568561609 |
| Schwartzberg | 8.491570723 | 7.449400105 |

Metric direction is mixed: the comparator has higher mean Polsby--Popper and
convex-hull ratio, while NRS has higher mean exact Reock. This is descriptive
evidence against collapsing compactness into a winner or composite score.

The slice satisfies the accepted-result and exact-regeneration gate. National
Tier 2 may advance only under a separately frozen, sequential execution
protocol with State-specific projections and complete failure retention.

## Governing Boundary

The slice measures block-projected NRS and comparator assignments on identical
retained TIGER block geometry. It does not measure original enacted polygon
linework and cannot support winner, fairness, legal, VRA, or adoption claims.
