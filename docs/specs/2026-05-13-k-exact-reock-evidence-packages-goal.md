# K Exact Reock Evidence Packages Goal

**Opened:** 2026-05-13
**Status:** complete

## Goal

Add auditable exact polygon minimum-bounding-circle (MBC) Reock evidence without
changing the production `reock()` output, which remains the
centroid-plus-max-boundary-radius proxy for Python-pipeline parity.

## First Target

The first slice is a deterministic exact-MBC smoke package containing simple
polygons with known MBC support sets. It validates the exact reference path and
records where exact MBC differs from the production proxy.

## Acceptance

- [x] Add an active wave and pulse context for K exact Reock evidence.
- [x] Add exact polygon-MBC helper coverage in `bisect-analysis`.
- [x] Add a hash-bound exact-MBC Reock fixture package.
- [x] Update K.0/K.2/K.7 ledgers and papers to cite the package.
- [x] Close the wave with validation commands and commit evidence.

## Non-Goals

- Do not replace production `reock()` values in `all_metrics()` in this wave.
- Do not claim full district-scale exact-MBC replay until real district polygons
  are packaged.
