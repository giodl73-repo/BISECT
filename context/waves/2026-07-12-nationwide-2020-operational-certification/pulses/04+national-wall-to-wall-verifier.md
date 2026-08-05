---
pulse: 04
title: National wall-to-wall verifier
status: done
depends_on: 03
wave: nationwide-2020-operational-certification
validation_level: L2 nationwide assignment verification
---

# Pulse 04 - National Wall-to-Wall Verifier

The Rust-native verifier independently reopened every State context and tree,
checked package and context identity, replayed the recursive seat schedule,
recomputed leaf unit and population accounting, and traversed every assigned
district in the original adjacency graph.

## Result

- 50 States;
- 435 connected one-seat leaves;
- 8,126,956 canonical 2020 Census blocks;
- zero omitted or duplicate units; and
- zero disconnected leaves.

The replay command is:

```text
cargo run -p bisect-ops -- verify-national-trees
```

The hash-bound result is
`docs/experiments/nationwide-2020/national-tree-verification-manifest.json`.

This verifies operational coverage and contiguity. It does not prove boundary
or canonical optimality. The report also classifies source custody: four
Rust-native packages contain matching immutable builder snapshots, while 40
older packages declare builder hashes that no longer match the mutable legacy
source path. Their independently checked tree hashes and assignments remain
valid, but the historical construction-source gap must be remediated or
published as a limitation.
