# Certified Proof Backend Prototype

This package separates fast plan discovery from mathematical certification for
the root split of the committed path-8/four-district recursive fixture.

## Decision Sequence

Given a discovered connected split, compile three pseudo-Boolean decisions:

1. Does any feasible split have lower population deviation?
2. At the proven population bound, does any split have lower boundary cut?
3. At both proven bounds, does any lexicographically smaller assignment exist?

An UNSAT proof for each decision certifies the discovered split. A SAT result
returns a counterexample and rejects the discovery.

## Contents

- `optimal/`: three requests classified `unsat-proof-required`;
- `compact-optimal/`: parent/depth requests emitted for external classification;
- `suboptimal/`: a deliberate worse discovery with a SAT counterexample;
- `compact-suboptimal/`: the same counterexample under parent-depth v3;
- deterministic OPB files with static connectivity no-goods;
- discovery and request identities; and
- a hash-bound package manifest.

## Proof Status

The pinned RoundingSat/VeriPB toolchain now passes a committed smoke proof under
`docs/examples/proof-toolchain-smoke/`. This request package still does not
bundle proofs for all six decision files.

The original bounded requests enumerate disconnected assignments to create
static connectivity no-goods. `compact-optimal/` instead uses one root per
child, parent arcs, acyclic binary depths, zero depth outside each child, and a
deterministic minimum-index root selected by prefix variables. The separate
smoke package preserves the earlier parent-depth-v1 proof accepted by VeriPB.

## Rebuild And Verify

```powershell
cargo run -p bisect-ilp --example certified_proof_path8 -- `
  docs\examples\certified-recursive\path8-k4\output\certified-bisection-tree.json `
  docs\examples\certified-proof-backend\path8-root

python scripts\research\verify_certified_proof_backend.py
cargo test -p bisect-ilp proof_backend -- --test-threads=1
```

## Claim Boundary

This is a deterministic OPB compiler and bounded SAT/UNSAT classification
prototype. It is not a generated proof, production solver integration, or
evidence of block-scale tractability.
