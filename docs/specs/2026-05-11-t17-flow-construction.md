# T.17 Flow-Based Construction

**Status:** Implementation slice active  
**Track:** T-series construction algorithms  
**Crate:** `bisect-flow`

## Purpose

Add a constructive, flow-style district builder that treats district formation as
balanced assignment from seeds under population capacity bounds. This is not a
global exact optimizer; exact lifecycle and bounds remain in U-series solvers.

## First Slice

The first audited slice implements a deterministic baseline:

- farthest-point seed selection on the adjacency graph
- balanced frontier expansion with an upper population capacity
- structured infeasibility witness when tiny fixtures cannot satisfy capacity
- BFS/exhaustive small repair hook for fixture-sized invalid assignments
- versioned `FlowSummary` with status, seeds, edge cut, population deviation,
  infeasibility witness, and `algorithm_lineage`

The slice is intentionally graph-native. Distance/projection costs and
multi-commodity flow solvers are later extensions behind the same output
contract.

## CLI Surface

- `--partition-mode flow-construction`
- `--structure flow-construction`

Future flags from the roadmap remain staged:

- `--flow-seeds farthest|existing`
- `--flow-cost distance|edge-cut|hybrid`
- `--flow-repair none|bfs`

Until those flags are exposed, defaults are `farthest`, `edge-cut`, and `bfs`.

## Audit Contract

Successful final plans must emit normal RPLAN audit sidecars through the state
runner. The flow summary is included in `algorithm_lineage` with:

- `producer_crate = "bisect-flow"`
- `method = "flow-construction"`
- relative summary path and SHA-256
- structured status and infeasibility witness when present

Invalid outputs must stop before final plan emission with a reproducible status.

## Tests

- L0 path fixture respects capacity and contiguity
- L0 infeasible capacity fixture emits a witness
- L0 deterministic output for fixed inputs
- L1 CLI/RPLAN sidecar fixture records flow lineage
