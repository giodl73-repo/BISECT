---
wave: rgraph-core-expansion
pulse: 04
status: done
depends_on: [03]
governing_roles:
  - BOUNDARY
  - BENCHMARK
  - LEDGER
---

# Pulse 04 - Close Validation

## Boundary Review

`rgraph-core` owns graph-only primitives over the existing directed weighted
adapter:

- deterministic shortest paths and predecessor/path-count trees;
- filtered reachability;
- connected components over all nodes or restricted node subsets;
- weighted Brandes edge betweenness;
- undirected bridge detection over the directed adapter projection;
- undirected articulation point detection over the same projection.

Domain crates retain interpretation:

- `bisect-analysis` keeps district, GEOID, county-context, contiguity reporting,
  legal/report, and civic meaning.
- ROUTE-specific redundancy and corridor scoring remain outside this repository
  until a portable git/local dependency plan is executed.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis contiguity -- --test-threads=1
git diff --check
```
