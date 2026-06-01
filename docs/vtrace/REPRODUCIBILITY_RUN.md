# Reproducibility Run Record

## DCR-007 declared scope

Current reproducibility class: `smoke-only`.

Full-scale and release-subset reproducibility remain blocked until a selected
source-data cache, release config, clean checkout/environment, and artifact
storage target are available.

Environment check recorded 2026-06-01: no release config under `configs/*.yml`
and no `data/2020/` source cache were present in this checkout, so no
release-subset or full-scale replay scope can be honestly declared here.

## Smoke-only evidence

| Field | Value |
|---|---|
| Scope | Public label-import fixtures under `docs/fixtures/import-label/`. |
| Command | `cargo test -p bisect-cli public_import_fixture --lib -- --test-threads=1` |
| Build features | Cargo defaults for `bisect-cli`. |
| Source-data hashes | Not applicable; fixture inputs are committed text files. |
| Output comparison | Parser assignments compared against committed expected-assignment JSON. |
| Replay class | smoke-only; not a real-state or full-scale redistricting replay. |

## Required release-subset record

Before DCR-007 can close at release-subset or full-scale level, record:

- Clean checkout commit and working-tree status.
- Rust toolchain, target, OS, CPU class, and build features.
- Binary SHA-256 and METIS engine.
- Config path and SHA-256.
- Source-data custody and hash pointers.
- Exact command lines for build, analyze, report, and verify.
- Seed/search metadata and convergence settings.
- Artifact paths and SHA-256 values.
- Clean replay comparison result or divergence disposition.

## Current disposition

DCR-007 is not closed for full-scale or release-subset reproducibility. Public
claims may cite only smoke-only fixture replay until a selected data-backed run
is executed and reviewed.
