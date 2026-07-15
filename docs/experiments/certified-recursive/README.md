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
cargo run -p bisect-ops -- build-state-rctx `
  --state-code RI --state-fips 44 --state-name rhode_island `
  --shapefile data\2020\tiger\blocks\tl_2020_44_tabblock20\tl_2020_44_tabblock20.shp `
  --pl-geo data\2020\redistricting\ri2020.pl\rigeo2020.pl `
  --pl-population data\2020\redistricting\ri2020.pl\ri000012020.pl `
  --rctx target\native-rctx-ri\ri_blocks_2020.rctx `
  --report target\native-rctx-ri\ri.json `
  --manifest target\native-rctx-ri\ri-manifest.json

cargo run -p bisect-ops -- compare-rctx `
  data\2020\certified\ri_blocks_2020.rctx `
  target\native-rctx-ri\ri_blocks_2020.rctx

cargo run -p bisect-ops -- verify-ri-frontier `
  docs\experiments\certified-recursive\manifest.json `
  --check-rctx
```

Without the ignored RCTX, the committed hashes and report arithmetic remain
verifiable by omitting `--check-rctx`. The native Rust rebuild has exact graph
parity with the historical context across every unit, population, edge kind,
and edge weight. Its context hash intentionally differs because source hashes
now bind the Rust implementation. `--check-rctx` verifies custody of the
historical generated file.

## Claim Boundary

This package closes block-input and island-link custody using the repository's
existing rule. It does not produce a candidate plan, generate an optimality
proof, or certify Rhode Island's first cut.
