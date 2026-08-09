---
pulse: 16
title: National Tier 2 geometry
status: active
wave: nrs-2010-national-baseline
validation_level: L2 governed empirical evidence
---

# Pulse 16 - National Tier 2 Geometry

## Purpose

Extend the accepted Rhode Island common-block geometry method to all 50
governed 2020 State packages without changing plan assignments or metric
semantics.

## Deliverables

- [x] Freeze the 50-State universe and regional projection rules.
- [x] Freeze one-State-at-a-time execution and complete failure retention.
- [x] Freeze State-weighted and district-weighted estimands.
- [x] Generalize the accepted analyzer and add national runner tests.
- [x] Execute all States sequentially and preserve all failures.
- [ ] Exactly regenerate the complete national package.
- [x] Publish the provisional result and claim boundary.

## Result

The one-worker run passed all 50 States with zero failures. Both plan families
contain 435 districts over the same 7,889,194 retained blocks.

| District-weighted mean | NRS v0.3 | CD118 block projection |
|---|---:|---:|
| Polsby--Popper | 0.025380339 | 0.061087819 |
| Exact Reock | 0.245552262 | 0.377238283 |
| Convex-hull ratio | 0.557042985 | 0.702220801 |
| Schwartzberg | 7.847347095 | 5.656288737 |

State-weighted means have the same direction. Exact regeneration remains the
final package gate. The result describes frozen block-projected geometry and
does not establish superiority or evaluate the original enacted linework.

## Governing Boundary

The package measures block-projected plan geometry, not original enacted
polygon linework. National averages are descriptive and may not be framed as a
winner, fairness score, or legal finding.
