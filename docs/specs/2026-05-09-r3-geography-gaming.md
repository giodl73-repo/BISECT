---
title: "Geographic Gaming: Can Resolution and Adjacency Choices Produce Partisan Plans?"
series: R.3
status: Planned
date: 2026-05-09
track: R-adversarial-robustness
---

## Claims
1. The resolution choice (tract vs. block-group) is the most consequential algorithmic decision that an adversarial actor could potentially manipulate. F.3 established that the resolution rule (k/n > 0.05) is objective, but an adversary could dispute the threshold.
2. Varying the resolution threshold from 4% to 6% (a plausible dispute range) changes the resolution choice for 3-4 states. The partisan impact is < 0.2 D-seats nationally.
3. The adjacency definition (TIGER/Line vs. manual boundary) could in principle be manipulated by excluding certain shared boundaries (e.g., water-separated tracts that technically share a border). R.3 tests the maximum partisan shift from manipulated adjacency definitions for NC and TX.
4. Audit protection: the manifest records `unit_type` and `adjacency_method`. Any non-standard adjacency definition is immediately visible and must be justified in advance.
