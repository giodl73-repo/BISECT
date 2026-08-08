---
pulse: 10
title: Initial DFS tie census
status: active
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
- [ ] Rebuild the release executable.
- [ ] Replay all governed multi-district 2020 roots.
- [ ] Verify assignment/objective preservation and publish the census.

## Governing Boundary

The counters cover the initial root-0 DFS candidate only. A count above one is
a seed-sensitive tie opportunity, not evidence that final assignments differ.
