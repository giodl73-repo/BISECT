# Shared Kernel External Consumer Contract

BISECT exposes a small set of reusable Rust crates for sibling products such as
CROP. These crates are generic infrastructure, not redistricting-only code.

## Canonical Git dependencies

External repos should depend on the BISECT repository directly until release
tags are introduced:

```toml
rctx-core = { git = "https://github.com/giodl73-repo/BISECT.git", package = "rctx-core", branch = "main" }
rgraph-core = { git = "https://github.com/giodl73-repo/BISECT.git", package = "rgraph-core", branch = "main" }
rstat-core = { git = "https://github.com/giodl73-repo/BISECT.git", package = "rstat-core", branch = "main" }
ropt-core = { git = "https://github.com/giodl73-repo/BISECT.git", package = "ropt-core", branch = "main" }
```

Local peer development should use a Cargo patch in the consumer repository,
not path dependencies in committed manifests:

```toml
[patch."https://github.com/giodl73-repo/BISECT.git"]
rctx-core = { path = "../apportionment/crates/rctx-core" }
rgraph-core = { path = "../apportionment/crates/rgraph-core" }
rstat-core = { path = "../apportionment/crates/rstat-core" }
ropt-core = { path = "../apportionment/crates/ropt-core" }
```

## Consumer-facing APIs

The following APIs are intended for external consumers and should retain
deterministic behavior, explicit errors, and non-redistricting names:

| Crate | Consumer-facing surface |
|-------|-------------------------|
| `rctx-core` | `RctxPackage`, `RctxManifest`, `RctxSourceIndexEntry`, `ContextUnitIndex`, `GraphRecord`, `ClaimBoundary`, `package_content_hash`, `verify_package`, `synthetic_ai_context_package_fixture` |
| `rgraph-core` | `NodeIndex`, `BoundaryMetrics`, `ConnectorPath`, `ClusterSummary`, `undirected_edge_cut`, `undirected_boundary_metrics`, `shortest_connector_path`, `undirected_cluster_summaries`, `connected_components*` |
| `rstat-core` | `summary::*`, `scoring::*`, deterministic validation errors for empty/non-finite samples |
| `ropt-core` | `ObjectiveVector`, Pareto helpers, `SeedPart`, `derive_seed`, `BudgetItem`, `BudgetSelection`, `exact_budgeted_selection`, `greedy_budgeted_selection` |

## Current branch discipline

- `main` must keep these four crates buildable as Git dependencies.
- Breaking changes should land with a consumer migration note in this file.
- CROP may pin a specific BISECT commit in `Cargo.lock`; updating the lock is
  the consumer-side compatibility check.
- Release tags can replace branch dependencies once CROP moves beyond fixtures.

## Validation

Focused validation for this contract:

```powershell
cargo test -p rctx-core -p rgraph-core -p rstat-core -p ropt-core -- --test-threads=1
```

The `shared-kernels` GitHub Actions workflow runs the same focused gate on
pushes and pull requests touching shared kernel crates or this contract.
