---
pulse: 05
title: National proof coverage matrix
status: done
depends_on: 04
wave: nationwide-2020-operational-certification
validation_level: L2 nationwide proof classification
---

# Pulse 05 - National Proof Coverage Matrix

Every nontrivial recursive node is classified by objective layer.

| Objective layer | Proved | Unproved | Coverage |
|---|---:|---:|---:|
| Population arithmetic floor | 385 | 0 | 100% |
| Weighted boundary optimum | 0 | 385 | 0% |
| Canonical tie optimum | 0 | 385 | 0% |

The six single-district States have no recursive cut objectives and are marked
not applicable. Canonical proof remains blocked by boundary proof because the
objectives are lexicographic.

The machine-readable result is
`docs/experiments/nationwide-2020/national-proof-coverage.json`.

The completed population layer is exact ratio arithmetic. It is not a claim
that the heuristic weighted-boundary incumbents are globally optimal.
