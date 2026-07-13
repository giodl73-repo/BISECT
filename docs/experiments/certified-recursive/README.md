# Rhode Island Certified Root Frontier

## Result

The full Rhode Island 2020 block-level county-bridged RCTX now exists locally
and is hash-bound by `ri-2020-root-frontier.json`.

Measured instance:

- 25,649 Census blocks;
- 66,097 positive shared-boundary edges;
- 64 synthetic county-based island bridge edges;
- 91,810 assignment-plus-cut variables before connectivity encoding;
- 1,097,379 total population; and
- theoretical best `1:1` scaled population deviation of 1.

The 6.96 MB RCTX is stored under ignored `data/` custody and is not committed.
Its SHA-256, context hash, unit-universe hash, source hashes, and graph summary
are committed.

## Island Connectivity

The land-only graph has two components:

| Component | Blocks | Population |
|---|---:|---:|
| Main graph | 25,585 | 1,095,969 |
| Block Island component | 64 | 1,410 |

The established BISECT island rule is then applied:

1. choose the largest component as the main graph;
2. for every unit in each remaining component, find the nearest main-component
   unit in the same county;
3. fall back to the nearest main-component unit statewide if needed; and
4. assign every synthetic bridge the median land-boundary weight.

For Rhode Island this adds 64 deterministic bridge edges and produces one
connected graph.

## Certification Blockers

1. The bounded 24-unit oracle cannot process 25,649 blocks.
2. Static connectivity no-goods can require up to `2^25648` canonical
   assignments.
3. No production discovery solver is installed.
4. The pinned RoundingSat/VeriPB smoke toolchain is not yet integrated with the
   production State-scale runner.

Rust successfully parses the RCTX and reaches the intended explicit error:

```text
certified recursive instance has 25649 units, above --exact-fixture-limit 24
```

## Rebuild And Verify

With the local Census sources:

```powershell
python scripts\research\build_ri_block_rctx.py build `
  --rctx data\2020\certified\ri_blocks_2020.rctx `
  --report docs\experiments\certified-recursive\ri-2020-root-frontier.json `
  --manifest docs\experiments\certified-recursive\manifest.json

python scripts\research\build_ri_block_rctx.py verify `
  docs\experiments\certified-recursive\manifest.json `
  --check-rctx
```

Without the ignored RCTX, the committed hashes and report arithmetic remain
verifiable by omitting `--check-rctx`. Byte-identical RCTX reconstruction also
depends on the GEOS/PROJ/Shapely/GeoPandas versions recorded in the report;
`--check-rctx` verifies custody of that exact generated file rather than
claiming toolchain-independent geometry output.

## Claim Boundary

This package closes block-input and island-link custody using the repository's
existing rule. It does not produce a candidate plan, generate an optimality
proof, or certify Rhode Island's first cut.
