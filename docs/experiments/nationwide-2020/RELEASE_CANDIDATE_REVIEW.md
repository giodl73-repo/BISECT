# Nationwide 2020 Operational Release-Candidate Review

**Review level:** internal L1 concrete-bundle review  
**Decision:** `accepted_as_local_release_candidate`  
**Public L2 gate:** open  
**External publication:** not performed

## Candidate identity

- Bundle root: `release_staging/nationwide-2020-operational-v1`
- Contract: `BISECT-EVIDENCE-PACKAGE-v1`
- Source commit: `2da2f658726437676bbf12a71ad2802e3cc5436e`
- Created: `2026-08-05T04:00:30.699Z`
- Manifest SHA-256:
  `4cd027421e33e636b954ffc6d05bfab912c3241aa334028b852e7be959249830`
- Package size: 115 files, 188,812,420 bytes
- Manifest-bound artifacts: 113
- Whole-package hash entries: 114 (all files except `HASHES.sha256`)

The generated bundle remains ignored and local-only under the artifact
publication policy. This tracked record identifies it without promoting the
generated assignments, maps, reports, or raw source data into Git.

## Verification performed

The builder completed all 50 States and then passed its integrated verifier.
An independent second invocation also passed:

```text
cargo run -p bisect-ops -- verify-national-release release_staging/nationwide-2020-operational-v1
```

The review additionally counted 50 assignment CSV files containing exactly
8,126,956 data rows and 50 nontrivial PNG maps. California, Alaska, and New
York maps were visually inspected. They are suitable for the declared
projected-block-centroid diagnostic role; they are not dissolved district
boundary maps.

## Retained runtime and proof-size evidence

The package records 12,748 completed seed screens and 354 timeout screens.
Full screening history is retained for 24 multi-district States; 20 legacy
multi-district States retain only partial screening history. Successful-screen
and State wall-clock durations were not retained and were not reconstructed.

The proof-size table records 385 embedded population-proof records in
67,592,652 bytes of containing tree files and 28,788 bytes of package
manifests. Population proof records are embedded, so a smaller standalone byte
count would be artificial. Boundary and canonical certificate sizes are zero
because neither proof stage was run.

## Claim disposition

The candidate supports the operational claim already established by the
independent national verifier: all 50 States, 435 connected leaves, 8,126,956
blocks exactly once, and the arithmetic population floor at all 385 nontrivial
nodes.

It does not establish exact boundary optimality, canonical optimality, NRS
v0.1 single-seed conformance, clean historical execution replay for the 40
legacy packages, VRA compliance, partisan fairness, legal validity, official
adoption, or public-release approval.

Public promotion still requires the applicable DATUM, SCALE, COMMONS, and
VAULT review lanes and the DCR-004 L2 concrete-bundle decision. This review
does not substitute for those human/external gates.
