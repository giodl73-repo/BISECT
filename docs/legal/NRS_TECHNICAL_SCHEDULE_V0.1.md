# Statutory Technical Schedule A: Geographic Benchmark v0.1

**Status:** Candidate schedule incorporated by the model statute
**Readiness status:** Not certified for enactment
**Current implementation evidence:** The block-level reference path, seed
wiring, conformance fixtures, and hostile-verifier corpus are implemented. A
later operational research profile, NRS v0.3, has generated and independently
verified complete 50-State, 435-district assignments for each of the 2000,
2010, and 2020 Census cycles. That evidence does not silently amend this v0.1
candidate schedule or establish enactment readiness: v0.3 remains a proposed
technical successor, exact weighted-boundary and canonical proof coverage is
0 of 1,155 national recursive nodes, a new external v0.3 replication record is
pending, and the full evaluation schedule is incomplete.

## 1. Source identity

- Census unit: decennial census blocks, 15-digit GEOIDs.
- Population: official resident population from the controlling PL 94-171
  release and numbered corrections.
- Geometry: controlling TIGER/Line tabulation-block release.
- Every source URL, release identifier, archive, component file, and correction
  carries a SHA-256 transport hash and canonical manifest content hash.

## 2. Adjacency

Two blocks are adjacent when their polygons share a positive-length land
boundary after geometry validation.

- Point-only contacts are excluded.
- Water-only contacts are excluded.
- Islands and enclaves require numbered bridge records identifying endpoints,
  reason, evidence, and approving authority.
- Sliver corrections require numbered errata.
- Zero-population blocks remain in the graph and assignment universe.
- Every undirected edge appears symmetrically.

Shared boundary is measured in an equal-area/equidistant State-appropriate CRS
named by the profile. Length is converted to millimeters and rounded to the
nearest integer using ties-to-even. The integer is the edge weight.

## 3. Recursive structure

At a node allocated `k` districts:

1. stop when `k = 1`;
2. assign `floor(k/2)` districts to one child and `ceil(k/2)` to the other;
3. partition the connected induced subgraph using the pinned engine;
4. assign the lower district-label range to the child containing the
   lexicographically smallest GEOID; and
5. recurse in ascending district-label order.

No prime-factor, ratio-search, county, racial, partisan, or community signal is
used.

METIS recursive bisection does not guarantee contiguous parts. When its raw
candidate is fragmented, the reference engine constructs a depth-first
spanning tree rooted at the lexicographically smallest GEOID, visiting adjacent
blocks in ascending GEOID order. Every tree-edge cut and both orientations are
evaluated. The candidate is selected in ascending order of maximum absolute
population deviation, weighted boundary cut, moved population relative to the
raw METIS labels, and minimum GEOID. A tree-edge cut makes both children
connected by construction. This normalization is skipped when both raw METIS
parts are already connected.

## 4. Population target

The child population targets are proportional to child district counts.
Optimizer imbalance is capped at 0.5% during candidate generation.

After recursive partitioning, a deterministic repair pass evaluates
boundary-block moves in ascending order of:

1. reduction in maximum absolute population deviation;
2. increase in weighted cut;
3. moved population; and
4. GEOID.

A move is permitted only if both affected districts remain contiguous. Repair
stops when no move reduces maximum deviation. The record reports whether a
lower-deviation assignment is known under the same profile.

If the candidate or repair cannot satisfy the legal floor, the benchmark fails
with a machine-readable infeasibility witness. It shall not silently change
units, tolerance, engine, seed, or adjacency.

## 5. Engine

Candidate engine:

- METIS 5.1 family through Rust crate `metis` 0.2.2;
- `metis-sys` 0.3.2, registry checksum
  `769ee6be814b21c52afcc631f07051f578487d642f3e89a45f2a086fd94fffa9`;
- recursive partition type;
- edge-cut objective;
- `ufactor = 5`;
- `niter = 100`;
- all other options explicitly serialized in `standard_profile.json`.

The final enacted schedule shall replace this package identity with the source
archive hash and certified reference-engine commit. The Act does not become
effective until that identity and the block conformance corpus are enacted or
certified under the readiness provision.

## 6. Seed

Compute:

```text
digest = SHA-256(
  ASCII("NRS_BASELINE_V0_1") ||
  canonical-json-v1(input_manifest)
)
seed = little_endian_u64(digest[0..8])
```

If the engine requires a signed 32-bit seed:

```text
seed32 = seed mod 2147483647
```

No timestamp, local path, comment, report option, or presentation field enters
the seed manifest.

## 7. Canonical output

The assignment object maps 15-digit GEOID strings to 1-based district labels.
It is serialized under `canonical-json-v1`. Display copies may be pretty
printed, but canonical content and transport hashes are recorded separately.

## 8. Conformance corpus

Positive and negative fixtures include:

- one-district;
- even and odd district counts;
- disconnected graph rejection;
- islands, water, enclave, and bridge records;
- zero-population blocks;
- population boundary and repair;
- seed vector;
- engine option vector;
- tie-breaking;
- canonicalization;
- tampered input;
- legal-profile mismatch; and
- comparative-engine non-equivalence.

The corpus shall include at least one manually inspectable fixture and one real
State block fixture before readiness certification.
