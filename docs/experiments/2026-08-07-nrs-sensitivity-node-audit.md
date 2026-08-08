# NRS v0.3 Sensitivity Node Audit

**Date:** 2026-08-07
**Scope:** governed 2020 State roots before any new diagnostic-seed execution

## Selection Source

The audit read each multi-district State's committed
`baseline-tree.json` and root `certified-discovery.json`. It did not generate
new diagnostic seeds.

Of 44 multi-district State roots, seven recorded one or more
`connected-subtree-population-operations` under the NRS v0.3 refinement path:

| State | Seats | Root split | Blocks | Population operations | Max scaled deviation |
|---|---:|---:|---:|---:|---:|
| CA | 52 | 26:26 | 519,723 | 1 | 1,794 |
| CO | 8 | 4:4 | 140,345 | 1 | 1,480 |
| GA | 14 | 7:7 | 232,717 | 1 | 364 |
| MT | 2 | 1:1 | 88,417 | 3 | 163 |
| NH | 2 | 1:1 | 31,948 | 1 | 11 |
| NM | 3 | 1:2 | 107,215 | 1 | 4 |
| NY | 26 | 13:13 | 288,819 | 1 | 12,233 |

These operations do not prove that a seed-sensitive tie exists. They identify
roots that exercised more of the NRS refinement path than Rhode Island's
zero-operation root.

## Frozen Slice Selection

- **New Hampshire:** smallest equal-seat root in the candidate set.
- **New Mexico:** only unequal-seat root in the candidate set.
- **Georgia:** bounded higher-seat equal root, selected between the smaller
  Colorado and substantially larger New York/California roots.

The three roots cover 371,880 blocks and root seat schedules `1:1`, `1:2`, and
`7:7`.

## Benchmark Replay

The committed release executable replayed each published root seed exactly:

| State | Published engine seed | Discovery SHA-256 | Replay |
|---|---:|---|---|
| NH | 828041789 | `8b92f34f11b0ad08b17e8f25a062f84b9ec8bdf48f674a1890804a0f6cffad83` | exact |
| NM | 1922790591 | `563dee5a5584ca50d914f09d38f18361d7952887c6512feaaa53b3abc1920224` | exact |
| GA | 1570084764 | `7e908aa442ca61ae4633db06d3156080d41afcf09ab59dcc2c4c2834ceb61b77` | exact |

Replay establishes executable/context readiness only. It is not a diagnostic
seed result.
