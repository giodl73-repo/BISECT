# Nationwide 2020 Operational Certification

The nationwide operational run is complete for all 50 States and all 435
congressional districts.

## Verified Result

- 8,126,956 canonical Census blocks assigned exactly once;
- zero omissions and zero duplicate assignments;
- 435 connected one-seat leaves;
- 385 nontrivial recursive nodes following the canonical floor/ceiling seat
  schedule; and
- arithmetic population optimality at all 385 nodes.

California was the last operational frontier. At seven-seat node `001`, seed
128 refined the scaled maximum population deviation from 355,479 to the
arithmetic lower bound of 2. The completed California tree contains 52 leaves
and 51 nontrivial nodes.

## Proof Boundary

Population proof coverage is 385/385. Weighted-boundary proof coverage is
0/385, and canonical-tie proof coverage is 0/385. Those latter objectives are
unproved, not failed and not infeasible.

The result is therefore a complete deterministic operational benchmark with
exact population-floor classification. It is not an exact canonical national
plan and is not yet a conformance run of the single manifest-derived seed in
National Redistricting Standard v0.1.

## Replay

```text
cargo run -p bisect-ops -- verify-national-rctx
cargo run -p bisect-ops -- verify-national-trees
```

The national report, proof matrix, verifier source snapshot, and their hashes
are bound by `national-tree-verification-manifest.json`. State operational tree
packages remain under local hash-bound custody in
`data/2020/certified/operational-trees/` pending the publication phase.

Four Rust-native packages contain matching immutable builder-source snapshots.
Forty older packages retain declared builder hashes whose mutable legacy source
path no longer matches. This provenance gap does not alter the independently
recomputed assignment, connectivity, schedule, or population results, but it
must remain visible in any publication bundle.
