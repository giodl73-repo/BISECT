# NRS External Replication Record

**Date:** 2026-07-10  
**Reviewer role:** Reproducible-software, non-author operator  
**Reviewer independence:** Did not author the challenge bundle or runtime
overlay; accepted no private correction during the final run  
**Operator:** Automated GitHub Copilot CLI agent  
**Base commit:** `d61a7136d60c27ecdd451067a1c08a063581820f`  
**Bundle manifest SHA-256 at run:** `9751a9bb090413daffe3f512c9dd32ec811cd6bdda9c25885720ee3dd70e7b8e`

## Result

`pass_candidate`

## Tasks

| Task | Result | Evidence |
|---|---|---|
| Claim boundary understood | pass | Agent restated tract-level/non-legal boundary |
| Source reconstructed | pass | Six-file overlay applied to fresh detached worktree |
| Toolchain installed | pass | rustc/cargo 1.95.0 |
| Inputs acquired and checked | pass | Four PL, five TIGER, and adjacency hashes matched |
| Reference assignment reproduced | pass | Raw and canonical hashes matched |
| Label chain verified | pass | Config/build/analysis/report links `MATCH`; verdict `VERIFIED` |
| Ensemble package verified | pass | Python verifier and Rust manifest test passed |
| North-Star design reviewed | pass | Coherent conditional exact-certificate goal |

## Reproduced Hashes

| Artifact | SHA-256 |
|---|---|
| Runtime overlay bytes | `8afdf2482ca541a9efe2722f3f6ecf03cc2647001fe78a85ddf1f9bb1d7e17ca` |
| Raw assignment | `930d3b18024d64ed17f640ac37d16a0204fc318c9df5332f074b5cb0491dac71` |
| Canonical assignment | `6cd96b33ac8fdae2d8e5e4b7bc9674358311eed62becbe624e6913d1507b4822` |

## Friction

1. Windows line endings produce different transport hashes for config and
   Cargo.lock; canonical/LF identities matched.
2. PL files extract under a human-readable State directory rather than the
   manifest's historical path label; bytes matched.
3. The public release currently supplies the adjacency pickle only. The runner
   used the documented legacy pickle shim and reproduced the expected result.

## Prior Blocked Runs

Earlier fresh-agent runs found and preserved:

- stale Census PL URL;
- missing public adjacency release;
- inconsistent reference overlay dependencies;
- malformed line-collapsed patch;
- missing analysis/report stages before `label-verify`.

Each was fixed in the public packet before the final run. The blocked records
were not treated as successful evidence.

## Claim Boundary

This automated non-author result satisfies the wave's independent challenge
gate. It does not satisfy DCR-003's desired human external-user validation and
does not establish legal validity, fairness, VRA compliance, block-level
conformance, peer review, or exact optimality.
