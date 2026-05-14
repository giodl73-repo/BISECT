# Fork Context: Pulse 01 - RGRAPH Core Skeleton

Read `pulses/01+rgraph-core-skeleton.md` as the execution contract.

## Governing Roles

### MERIDIAN

Checks graph correctness, deterministic shortest-path behavior, reachability,
and explicit handling of invalid weights.

### BENCHMARK

Requires synthetic tests that would catch tie-handling, filtered-edge, invalid
node, and invalid-weight regressions.

### LEDGER

Checks that the shared crate remains a kernel crate, not a package format or
domain-model crate.

### ROUTE-NUMERACY-CHECKER

Requires weight semantics to be explicit. Weights are abstract non-negative
costs; domain crates own units such as miles, travel time, or population
penalties.

## Execution Contract

- Work in repository root `C:\src\apportionment`.
- Use `apply_patch` for manual edits.
- Do not change ROUTE in pulse 01.
- Keep `rgraph-core` dependency-light.
- Run the pulse validation before marking done.

