# Proof Toolchain Smoke Package

This package contains the first externally generated and independently checked
proof in the Certified BISECT program.

RoundingSat proves that the bounded path-8 population-improvement decision is
UNSAT. VeriPB independently accepts the generated proof.

A second smoke artifact proves the path-8 boundary lower bound using the
polynomial `parent-depth-v3` connectivity encoding. This proof exercises
deterministic roots, parent arcs, acyclic depths, and prefix-root symmetry
reduction rather than static connectivity no-goods.

## Result

```text
RoundingSat: s UNSATISFIABLE
VeriPB: Verification succeeded.
```

The decision is intentionally small and has an incumbent population deviation
of zero, so its lower-bound contradiction is simple. This validates toolchain
compatibility and custody, not scalable proof complexity.

## Files

- `population.opb` — RoundingSat-compatible OPB decision.
- `population.pbp` — pseudo-Boolean proof version 2.0.
- `compact-boundary.opb` / `compact-boundary.pbp` — compact connectivity proof.
- `provenance.json` — pinned sources, binary hash, commands, and results.
- `manifest.json` — package hashes and verifier path.

## Replay

See [`docs/PROOF_TOOLCHAIN.md`](../../PROOF_TOOLCHAIN.md).

## Claim Boundary

This is a bounded proof smoke test. It does not certify a State split or close
the compact-connectivity/discovery frontier.
