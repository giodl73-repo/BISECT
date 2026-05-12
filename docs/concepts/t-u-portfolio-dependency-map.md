# T/U Portfolio Dependency Map

This map shows how the T.14-T.17 construction papers, U.16-U.20
search/optimization papers, crates, and public artifacts depend on each other.

The center of the portfolio is the RPLAN fixed point: every family can use
different internal machinery, but publication-grade outputs converge to
RPLAN/RCTX/certificate/manifest packages that are independently verifiable.

## Layer Cake

```text
Publication-facing artifacts
============================

docs/PAPERS.md
docs/papers/*.pdf
research/tracks/T-plan-construction/*
research/tracks/U-search-optimization/*
docs/examples/rplan-golden-packages/*
docs/examples/rplan-method-packages/*
docs/examples/rplan-benchmark-packages/*
docs/examples/u20-plan-audit-certificates/*
          ^
          |
          | evidence labels, PDFs, examples, transcripts
          |
Verification fixed point
========================

rplan verify-certificate
bisect verify --manifest
          ^
          |
          | reads committed packages
          |
RPLAN artifact contract
=======================

plan.rplan
context.rctx
audit-certificate.json
manifest.json
method-transcript.json
optional method reports / solver reports / command transcripts
          ^
          |
          | produced by algorithm-family crates and CLI commands
          |
Algorithm families
==================

T.14 spectral construction
  bisect-apportion::spectral
  bisect-cli example generator
  package: T.14+spectral-generated-synthetic
  benchmark package: T.14+spectral-grid10-benchmark

T.15 capacity clustering
  bisect-clustering
  golden package: T.15+capacity-constrained-clustering

T.16 hierarchical regionalization
  bisect-clustering
  golden package: T.16+hierarchical-regionalization

T.17 flow construction
  bisect-flow
  golden package: T.17+flow-construction

U.16 branch-and-cut
  bisect-ilp
  golden package: U.16+branch-and-cut
  benchmark package: U.16+branch-and-cut-path8-benchmark

U.17 branch-and-price
  bisect-column
  golden package: U.17+branch-and-price

U.18 local search / LNS
  bisect-local-search
  bisect improve
  golden package: U.18+local-search-improvement
  method package: U.18+local-search-generated-descendant
  benchmark package: U.18+local-search-grid10-benchmark

U.19 evolutionary / selected frontier
  bisect-pareto
  golden package: U.19+selected-frontier

U.20 audit certificates
  rplan-core
  rplan-io
  rplan-audit
  reference package: docs/examples/u20-plan-audit-certificates/grid3x3-valid
```

## Dependency Spine

```text
bisect-core + bisect-data
        |
        +--> construction crates
        |      bisect-apportion, bisect-clustering, bisect-flow
        |
        +--> search / exact crates
        |      bisect-ilp, bisect-column, bisect-local-search, bisect-pareto
        |
        +--> orchestration
               bisect-cli
                    |
                    v
             RPLAN/RCTX/certificate/manifest
                    |
                    v
        rplan-core + rplan-io + rplan-audit
                    |
                    v
        rplan verify-certificate / bisect verify --manifest
                    |
                    v
          papers, package corpus, replication notes
```

## Evidence Tiers

| Tier | Directory | What It Proves | What It Does Not Prove |
|---|---|---|---|
| Tiny golden package | `docs/examples/rplan-golden-packages/` | Verifier-facing package shape, lineage, hashes, and certificate acceptance across implemented families | Method runtime, real-data quality, benchmark performance |
| U.20 reference package | `docs/examples/u20-plan-audit-certificates/` | The audit-certificate fixed point and negative/positive verifier behavior | Algorithm production quality |
| Method-produced package | `docs/examples/rplan-method-packages/` | A real CLI or crate workflow can generate package artifacts with transcripts and stable hashes | Large-instance performance, legal sufficiency, empirical superiority |
| Benchmark package | `docs/examples/rplan-benchmark-packages/` or future release bundle | Larger verifier-scale or real-data runs with timing protocol, data provenance, and package-footprint notes | Final legal judgment or universal method ranking |

## Current Completed Public Packages

```text
docs/examples/rplan-golden-packages/
  T.14+spectral-partitioning/
  T.15+capacity-constrained-clustering/
  T.16+hierarchical-regionalization/
  T.17+flow-construction/
  U.16+branch-and-cut/
  U.17+branch-and-price/
  U.18+local-search-improvement/
  U.19+selected-frontier/

docs/examples/rplan-method-packages/
  T.14+spectral-generated-synthetic/
  U.18+local-search-generated-descendant/

docs/examples/rplan-benchmark-packages/
  T.14+spectral-grid10-benchmark/
  U.16+branch-and-cut-path8-benchmark/
  U.18+local-search-grid10-benchmark/

docs/examples/u20-plan-audit-certificates/
  grid3x3-valid/
  negative verifier fixtures
```

## Next Publication-Packaging Work

The next natural step is not another tiny verifier fixture. The useful frontier
is benchmark-tier packaging:

- choose one no-download or pinned-data real workflow small enough to commit or
  release as an artifact bundle
- record full data provenance and command transcript
- include timing and hardware notes
- verify through the same RPLAN and BISECT surfaces
- update the affected paper evidence tables only where the new package changes
  the claim level
