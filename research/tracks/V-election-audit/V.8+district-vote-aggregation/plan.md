# V.8 Plan

## Thesis

District vote aggregation should be an optional bridge from verified RCOUNT
summaries to RPLAN district assignments. The bridge must bind the count package,
plan hash, RCTX context hash, and any crosswalk hash used to project reporting
units into districts.

## Implementation Status

- [x] Add `rcount-district` optional bridge crate.
- [x] Aggregate verified RCOUNT summaries over a synthetic RPLAN assignment.
- [x] Emit district aggregation transcript with RCOUNT package hash and RPLAN
  plan hash.
- [x] Accept optional `.rctx` context and verify it matches the RPLAN unit
  universe.
- [x] Emit optional RCTX context hash in the transcript.
- [x] Add RCOUNT `rctx_refs` consumer references.
- [x] Emit `rctx_reference_id` and `rctx_crosswalk_hash` when aggregation uses
  a context matching a declared RCOUNT RCTX reference.
- [x] Add CLI coverage for the declared RCTX reference path.
- [x] Add L2 multi-election harness with split/merge lineage and per-cycle
  district aggregation.
- [x] Add negative coverage for stale plan units and broken lineage.

## Current Boundary

RCOUNT owns count summaries and reconciliation. RPLAN owns assignments. RCTX
owns shared context/crosswalk identity and verification. The district bridge
composes those hashes; it does not make RCOUNT a plan-validity verifier.

## Next Work

- [x] Add an explicit RCTX crosswalk NDJSON CLI path using `rctx-core` records.
- [x] Reject district aggregation when a declared RCOUNT crosswalk hash drifts
  from the explicit RCTX crosswalk set.
- [x] Use explicit crosswalk rows to project RCOUNT summaries before district
  aggregation.
- [x] Reject non-integral weighted count allocations instead of rounding.
- [ ] Add a real or realistic precinct-to-district crosswalk fixture using
  public source records.
- [ ] Record non-exhaustive crosswalk caveats in district aggregation
  transcripts.
- [ ] Add a rational-output transcript mode for genuinely fractional
  population/area crosswalks.
- [x] Add a CLI path for an explicit crosswalk file.
- [ ] Consider renaming the implementation crate from `rcount-district` to
  `rcount-rplan` only if a broader RPLAN bridge emerges.
