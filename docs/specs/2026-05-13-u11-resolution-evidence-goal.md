# U.11 Resolution Evidence Goal

**Opened:** 2026-05-13
**Status:** complete

## Goal

Promote U.11 resolution-aware manifest and mapping claims to a hash-bound smoke
package that verifies GEOID-prefix fine-to-coarse mapping, population
aggregation, and derived coarse adjacency.

## Acceptance

- [x] Add a package manifest, positive fixture replay, and tamper-rejection tests.
- [x] Add consumer coverage in `bisect-multiscale` for resolution package reads.
- [x] Update U.11 paper, public index, scorecard, and manifest docs.
- [x] Rebuild the U.11 PDF, run focused tests, run formatting, and commit.

## Closeout

The wave delivered
`docs/examples/u-search-evidence-packages/U.11+resolution-smoke/` and
`bisect-multiscale::resolution_evidence`. U.11 now has package-backed mechanics
evidence for manifest fields, GEOID mapping, population aggregation, and coarse
adjacency, while Texas autocorrelation and BG precision claims remain future
archives.
