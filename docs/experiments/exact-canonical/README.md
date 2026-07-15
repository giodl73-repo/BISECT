# Rhode Island 2020 Block-Level Exact Frontier

This package tests the first real-State target for the Exact Canonical
Benchmark at the normative census-block unit.

## Result

**Blocked, without an exact certificate.**

Rhode Island has:

- 25,649 TIGER/Line tabulation blocks;
- 25,649 matching PL 94-171 block population records;
- 1,097,379 total population;
- 21,382 positive-population blocks; and
- 4,267 zero-population blocks that remain part of the statutory universe.

The bounded E0 oracle accepts at most 24 units. Its symmetry-reduced Rhode
Island search would contain `2^25648-1` candidates, a 7,721-digit number.
Even at one billion candidates per second, enumeration would require roughly
`10^7704` years.

## Historical Input Blocker

At the time this sealed report was generated, local source custody contained
complete block geometry and population files but no block RCTX. That input is
now available and Rust-native, with exact parity against the historical graph.
Closing the input blocker does not resolve the exhaustive-search barrier.

## Reproduce

Verify the sealed package and, optionally, re-read every source using Rust:

```powershell
cargo run -p bisect-ops -- verify-exact-frontier `
  docs\experiments\exact-canonical\manifest.json `
  --check-sources
```

Without the large source files, the committed report's arithmetic and package
hashes remain verifiable by omitting `--check-sources`.

## Claim Boundary

This is a hash-bound real-data blocker report. It is not an exact State
certificate, a tract-level substitute, a heuristic benchmark, or evidence that
an optimality gap has closed. It does not build block adjacency or invoke a
solver.
