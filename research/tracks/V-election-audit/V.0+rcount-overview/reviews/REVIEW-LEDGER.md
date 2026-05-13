# Simulated Review: LEDGER

> AI-generated quality-improvement simulation, not real peer review.

## Score

2 / 4

## Summary

The format story is promising, but V.0 reads more like an implementation note
than a standardization note. If RCOUNT may become an interchange artifact, the
paper needs stronger versioning, schema, and compatibility language.

## Major Issues

1. **Schema/version contract is too implicit.** The paper mentions package
   layers but should name `rcount_version`, hash algorithm/versioning, optional
   files, and backward-compatibility expectations.

2. **External standards are deferred without a map.** It is fine not to build a
   NIST CDF or vendor adapter yet, but V.0 should include a compatibility
   roadmap: what would map to NIST CDF, vendor CSV/JSON/XML, statements of
   votes, and RPLAN/RCTX.

3. **Canonical JSON needs a stability note.** A public format must say what is
   canonicalized, what byte projections are hashed, and how domain separation
   prevents cross-layer substitution.

## Minor Issues

- The package layer table should mark required vs optional directories.
- The source index should be described as package-relative.
- RPLAN plan hash and optional RCTX context hash deserve one concrete sentence.

## Strengths

- Directory layout is easy to inspect.
- Optional RPLAN boundary is a good standards decision.
- The fixture suite makes future schema changes measurable.

