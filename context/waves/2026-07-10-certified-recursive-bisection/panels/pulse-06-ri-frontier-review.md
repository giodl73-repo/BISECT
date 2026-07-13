# Pulse 06 Rhode Island Frontier Review

**Date:** 2026-07-10  
**Roles:** DATUM, SCALE, MERIDIAN, COVENANT  
**Posture:** Real block input custody; no certified split

## Evidence

- 25,649 canonical RI 2020 blocks.
- 66,097 positive shared-boundary edges plus 64 deterministic bridges.
- 6.96 MB local hash-bound RCTX accepted by the Rust parser.
- Two initial land components: 25,585 blocks / 1,095,969 people and 64 blocks /
  1,410 people; one final connected graph.
- 91,810 assignment-plus-cut variables before connectivity encoding.
- Bounded oracle limit: 24 units.
- RoundingSat, VeriPB, SCIP, and HiGHS unavailable.

## Review Disposition

The missing block-input custody gap is closed. The active Rust island rule is
applied exactly: every non-main-component block connects to its nearest
same-county main-component block, with statewide fallback and median
land-boundary weight.

No exact result is claimed. Tract, inhabited-only, heuristic, and
optimality-gap substitutions remain prohibited.

## Carry-Forward

The next wave must install a production discovery solver and proof toolchain,
and replace exponential static connectivity
no-goods with a compact proof-loggable formulation.
