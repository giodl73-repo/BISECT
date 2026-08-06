# NRS 2000–2020 Structural Stability

## Outcome

This package compares the verified NRS v0.3 national node snapshots for the
2000, 2010, and 2020 Census cycles. Each input contains 50 States, 435 House
districts, and 385 recursive split nodes. The comparison completed with 120
node signatures common to all three cycles and 18 States with exact recursive
tree topology in all three cycles.

Twelve of the 18 all-cycle exact-topology States have more than one district:
AR, HI, ID, KS, MD, ME, MN, MS, NH, NM, RI, and WI. The remaining six are
single-district States in every compared cycle: AK, DE, ND, SD, VT, and WY.

## Pairwise matrix

| Census cycles | Same-seat States | Exact-topology States | Matched node signatures | Union node signatures | Topology Jaccard | Median absolute relative cut/person change | Median absolute tolerance-usage change |
|---|---:|---:|---:|---:|---:|---:|---:|
| 2000–2010 | 32 | 19 | 190 | 580 | 0.327586 | 0.573645 | 0.050152 |
| 2000–2020 | 26 | 18 | 170 | 600 | 0.283333 | 0.483659 | 0.047197 |
| 2010–2020 | 37 | 25 | 204 | 566 | 0.360424 | 0.427286 | 0.046866 |

The 2010–2020 pair has the highest structural overlap by all three reported
topology counts. The normalized incumbent cut metric is descriptive: a value
of 0.427286 means the median structurally matched node changed by about 42.7%
in weighted cut per parent person. It is not a district-overlap score and does
not establish improvement or optimality because Census graphs and boundary
measurements are cycle-specific.

## Metric definitions

- A node signature binds State, binary path, parent seat count, and child seat
  counts. It deliberately excludes Census block identities.
- Exact State topology means the complete set of node signatures is identical
  for the two cycles. A one-district State has an empty tree and therefore an
  exact empty topology when its seat count is unchanged.
- Topology Jaccard is matched node signatures divided by their union.
- Cut drift is the absolute relative change in weighted boundary cut divided
  by parent population, evaluated only on structurally matched nodes.
- Tolerance drift is the absolute change in the fraction of the frozen
  population tolerance consumed, again only on matched nodes.

## Assignment-overlap boundary

Cross-census assignment Jaccard is **not computed**. Decennial Census block
universes change through splits, merges, boundary corrections, and identifier
changes. A valid assignment comparison requires a published Census
relationship crosswalk or a geometry overlay with an explicit allocation
rule. Inferring overlap from GEOID strings or topology would be invalid.

## Artifacts and hashes

- `stability-matrix.json`: complete national pairwise and per-State matrix
- `manifest.json`: hash-bound input snapshots and matrix output
- 2000 snapshot SHA-256: `ff59e7639ece3d413c6f45e25f9d05e9afe4957e3494b3afcb84e6f5154651d5`
- 2010 snapshot SHA-256: `02314ca1c2a494c20586221c9bf2a6599ce827882def7f84d299de0c94911b42`
- 2020 snapshot SHA-256: `b4d59ae8d0ca3587298282bfc9d35652a75d12976461d3767f35267bfc2a4da7`
- Matrix SHA-256: `d5f6eb6e5a120d63bc00dec8ae4f773fc2bcf296555c73445318c59421801339`

## Claim boundary

This is a structural and descriptive incumbent-objective comparison. It does
not prove boundary or canonical optimality, compare assignments across Census
universes, or establish legal validity, VRA compliance, partisan fairness, or
official adoption.
