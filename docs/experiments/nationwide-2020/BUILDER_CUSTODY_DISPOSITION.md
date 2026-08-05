# Operational Builder Custody Disposition

**Date:** 2026-08-04

**Scope:** Nationwide 2020 operational recursive-tree packages

**Disposition:** Accepted historical limitation for the operational benchmark;
not acceptable precedent for new packages

## Finding

All 44 multi-district package manifests retain an expected builder SHA-256.
The four Rust-native AZ, CA, FL, and NY packages also retain immutable source
snapshots whose bytes match those hashes. The other 40 packages reference
mutable legacy Python paths. The source presently at those paths does not match
the historical hashes recorded when the packages were built.

The missing bytes were sought in:

- the active worktree;
- the archived legacy-Python tree;
- all reachable revisions of both historical builder paths;
- unreachable Git commits, trees, and blobs reported by full `git fsck`; and
- all retained State package directories.

No matching byte sequence was found for the outstanding historical versions.
The old source must therefore be treated as unavailable rather than recreated
or silently replaced.

## Evidence That Remains Independently Verifiable

The custody gap does not require trusting the historical builders for the
published operational claims. The Rust national verifier independently:

1. checks every retained tree against its package hash;
2. binds each tree to the original State context hash;
3. reopens all 8,126,956 block records;
4. verifies the floor/ceiling recursive schedule;
5. recomputes every leaf's unit and population totals;
6. traverses every district in the original adjacency graph; and
7. verifies all 385 arithmetic population lower bounds.

Those checks establish assignment coverage, contiguity, schedule adherence,
and population-floor status from retained outputs and normative inputs. They do
not reconstruct the historical execution or prove boundary/canonical
optimality.

## Decision

- Preserve all 40 expected builder hashes and report them as
  `declared-hash-not-currently-matching`.
- Do not rewrite historical manifests or substitute later source bytes.
- Permit the independently verified assignments to enter an internal release
  candidate with this limitation adjacent to every claim.
- Require all new or regenerated packages to embed immutable source snapshots.
- A public release still requires concrete VAULT and public-claim review; this
  internal disposition does not close that human gate.
