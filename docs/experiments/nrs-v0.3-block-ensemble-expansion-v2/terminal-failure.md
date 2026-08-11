# Expansion v2 Terminal Failure

At `2026-08-11T20:32:18.550231Z`, the first frozen Stage 0 process began for NH
Wilson after host-capacity admission passed with 95,793,344,512 free bytes
against 8,589,934,592 required bytes.

The release executable returned `1` with:

```text
Error: unsupported execution class
```

The v2 wrapper correctly emitted `excluded-expansion-v2-preflight` and base seed
`20260811`. The bound Rust runner accepted only
`excluded-expansion-preflight`/`governed-stage2` and required base seed
`20260810`. Thus the compiled execution contract and the frozen v2 wrapper
contract were incompatible even though both sources and the executable were
individually hash-bound.

The process ran for 1.3137548999511637 seconds, peaked at 4,722,688 RSS bytes,
and produced no trace. The ledger retains zero completed preflights, zero
retained bytes, and zero governed runner wall time. Admission, resource, and
terminal ledger records are preserved in this package.

The retained failure is independently checked with:

```text
python scripts/research/verify_block_ensemble_v2_failure.py docs/experiments/nrs-v0.3-block-ensemble-expansion-v2
```

Under the frozen rule, a nonzero return after process creation closes v2 without
retry. Therefore the other five preflights, all six preflight replays, all
primaries, and all governed replays are prohibited under this protocol.

## Successor requirement

Any successor must use a new protocol identity and fresh package. Before its
execution gate, it must test the compiled runner—not merely the Python command
constructor—against the successor execution classes, seed, State cohort, and
preflight/governed shapes. No v2 completion exists to reuse.

## Claim boundary

This is negative implementation-integration evidence only. It is not an
ensemble draw, feasibility result, convergence diagnostic, or comparison of
Wilson and Kruskal.
