# Pulse 01 Fork Context - K.2 Reock Correctness Pass

## Execution Contract

Execute `context/waves/2026-05-13-paper-rubric-uplift/pulses/01+k2-reock-correctness.md`
end to end.

## Role Notes

- MERIDIAN: separate canonical Reock, polygon approximation, and point-cloud MKA
  proxy mechanics.
- DATUM/BENCHMARK: cite concrete implementation files and tests; do not claim
  empirical strength beyond local evidence.
- BOUNDARY/SURVEY: keep court/practitioner language usable without implying that
  BISECT currently emits exact MBC-based Reock.

## Required Reads

- `docs/papers/ALGORITHM-PAPER-SCORECARD.md`
- `docs/papers/PAPER-QUALITY-REVIEW.md`
- `research/tracks/K-compactness/K.2+reock/`
- `research/tracks/K-compactness/K.0+compactness-overview/`
- `research/tracks/K-compactness/K.7+composite-court-guide/`
- `crates/bisect-analysis/src/compactness.rs`
- `crates/bisect-cli/src/bisection_runner.rs`

## Completion Standard

Changed papers rebuild, PDFs are copied into `docs/papers/`, ledgers reflect the
new quality status, and whitespace validation is clean.
