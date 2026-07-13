# Pulse 03 Reproducibility Review

**Date:** 2026-07-09  
**Roles:** COVENANT, MERIDIAN, BENCHMARK  
**Posture:** Internal replay review; not external replication

## Delivered Controls

| Control | Result |
|---|---|
| Exact compiler | `rustc 1.95.0` pinned in `rust-toolchain.toml` |
| Locked dependencies | Missing FLETCH and METIS-CORE source identities restored in `Cargo.lock`; clean `--locked` release build passed |
| Fixed config seed | YAML `seed` added to `AlgorithmSection`, propagated to METIS, tested, and recorded in build index |
| Canonical assignments | `final_assignments.json` keys serialized in lexicographic order with positive coverage |
| Reference scope | Rhode Island 2020, 250 tracts, 2 districts |
| Input custody | PL 94-171, TIGER, adjacency, GEOID, config, lockfile, and binary hashes recorded |
| Source reconstruction | Base commit plus hash-bound runtime overlay and config reproduced the validation snapshot sources |
| Independent executions | Two executions, each beginning from a clean source snapshot |
| Assignment result | Zero mapping differences; identical raw and canonical assignment hashes |
| Chain verification | Both executions reported config/build/analysis links `MATCH` and verdict `VERIFIED` |

## Reference Hashes

| Artifact | SHA-256 |
|---|---|
| Config, canonical LF | `6a2e6577f2d675a0d3716bd74206dbd1e17ba70ab1146351ca9bdb978f4d72bf` |
| Config, Windows replay bytes | `27457b80be6e4b7c01b63d1d43c00519808e7087431d35e7852bbcec8db532da` |
| Canonical assignment | `6cd96b33ac8fdae2d8e5e4b7bc9674358311eed62becbe624e6913d1507b4822` |
| Raw assignment | `930d3b18024d64ed17f640ac37d16a0204fc318c9df5332f074b5cb0491dac71` |
| Runtime overlay, canonical LF | `556bbc4eb3c5237ca4103fd0a5837187997cf8a5c061ec69386a62e8d92e82fb` |

## Review Findings And Disposition

- The first replay exposed noncanonical `HashMap` JSON ordering. The writer was
  changed to a lexicographically ordered map and both raw hashes then matched.
- The first clean build exposed missing git source identities in `Cargo.lock`.
  The lockfile was repaired and `cargo build --release --locked` passed.
- The first fixture overlay accidentally embedded the output patch as hunk
  context. The generated overlay replaced it, apply/reconstruction checks
  passed, and the output test was added to the documented verifier commands.
- `label-report --format html json` is not valid CLI syntax. The reference
  procedure uses one explicit format per invocation.

## Carry-Forwards

- This is tract-level functional evidence, not the block-level benchmark
  selected by NRS v0.1.
- Data custody is hash-bound local evidence; no public raw-data bundle is
  promoted by this pulse.
- A non-author replay is still required before external or release-grade
  promotion.
- Executable bytes are environment-specific; the pulse does not claim
  byte-identical binary reproduction.
- The NRS manifest-derived seed remains future work; this fixture uses the
  explicit fixed seed `424242`.

## Decision

Pulse 03 may close as an internal clean-source functional reference replay.
Broader DCR-007 and NRS conformance claims remain blocked by the carry-forwards.
