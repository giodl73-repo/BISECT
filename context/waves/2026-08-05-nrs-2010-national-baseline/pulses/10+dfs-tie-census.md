---
pulse: 10
title: Initial DFS tie census
status: complete
wave: nrs-2010-national-baseline
validation_level: L2 governed diagnostic evidence
---

# Pulse 10 - Initial DFS Tie Census

## Purpose

Replace blind sensitivity expansion with direct observability of the only
initial DFS tie at which the seeded METIS assignment can affect candidate
selection.

## Deliverables

- [x] Trace the seed-influence seam and all callers.
- [x] Add behavior-preserving candidate multiplicity instrumentation.
- [x] Add synthetic tie and emitted-method tests.
- [x] Precommit the 44-State root census.
- [x] Rebuild the release executable.
- [x] Replay all governed multi-district 2020 roots.
- [x] Verify assignment/objective preservation and publish the census.

## Result

All 44 governed multi-district State roots passed exact assignment and
objective preservation. The minimum-deviation candidate count ranged from 1
to 10. After the weighted-cut filter, 15 roots had one surviving candidate and
29 roots had two.

The 29 initial tie-opportunity States are AR, CA, CO, FL, GA, HI, IA, ID, KS,
KY, LA, MD, ME, MN, MO, MS, MT, NC, NH, NJ, NV, NY, OR, RI, TX, UT, WA, WI,
and WV.

## Governing Boundary

The counters cover the initial root-0 DFS candidate only. A count above one is
a seed-sensitive tie opportunity, not evidence that final assignments differ.
The already-published RI, NH, and GA sensitivity slices are direct examples:
each has an initial tie opportunity, but all tested seeds still reproduced the
governed assignment. NM has no initial tie opportunity and likewise reproduced
its governed assignment.
