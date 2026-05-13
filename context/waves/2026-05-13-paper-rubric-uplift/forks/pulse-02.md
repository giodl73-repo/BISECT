# Pulse 02 Fork Context - J.6 Apportionment Implementation Pass

## Execution Contract

Execute `context/waves/2026-05-13-paper-rubric-uplift/pulses/02+j6-apportionment-implementation.md`
end to end.

## Role Notes

- MERIDIAN: verify algorithm descriptions against the actual
  `bisect-apportion` modules.
- DATUM/BENCHMARK: keep Census verification and test-count claims tied to
  executable tests, not aspirational CI language.
- LEDGER: separate apportionment outputs from RPLAN/RCTX package evidence; this
  pulse is a paper-quality pass unless implementation tests expose a real bug.

## Required Reads

- `crates/bisect-apportion/src/lib.rs`
- `crates/bisect-apportion/src/huntington_hill.rs`
- `crates/bisect-apportion/src/divisor_methods.rs`
- `crates/bisect-apportion/src/paradoxes.rs`
- `research/tracks/J-apportionment/J.6+bisect-apportion-implementation/`
- `research/tracks/J-apportionment/J.0+apportionment-overview/`
- `docs/papers/ALGORITHM-PAPER-SCORECARD.md`
- `docs/papers/PAPER-QUALITY-REVIEW.md`

## Completion Standard

J.6 no longer contains stale API/test/verification claims, affected PDFs rebuild,
and `cargo test -p bisect-apportion` plus `git diff --check` pass.
