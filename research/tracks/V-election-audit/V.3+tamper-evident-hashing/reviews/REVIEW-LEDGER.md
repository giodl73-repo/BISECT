# Simulated Review: LEDGER

> AI-generated quality-improvement simulation, not real peer review.

## Score

2 / 4

## Summary

V.3 has the right hash layers, but the format contract is still too implicit.
If this paper is going to anchor future adapters, it needs field-level language
for source-index paths, canonical projections, and domain-separated hash roles.

## Major Issues

1. **Define source-index rows.** A row should include source id, package-relative
   path, media/content type or format label, hash algorithm, declared hash, and
   optional parser/adaptor metadata.

2. **Define canonical package hashing.** Say which records enter the normalized
   package hash and what volatile metadata is excluded.

3. **Version hash roles.** The paper should state that prefixes and algorithms
   are versioned so future RCOUNT versions can migrate.

## Minor Issues

- Distinguish manifest content hash from package content hash if both appear in
  transcripts today.
- Avoid implying every source artifact must be published in every jurisdiction.
- Mention that external standards/adapters are V.9 territory.

## Strengths

- Good layer names.
- Good separation of source and normalized records.
- The negative fixtures make the interchange contract more concrete.
