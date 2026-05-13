# Fork Context - Pulse 01 2020 Census Table 1 Verifier

## Contract

Execute `pulses/01+2020-census-table1-verifier.md` end to end.

## Required Reads

- `context/waves/PHASES.md`
- `context/waves/2026-05-13-j-apportionment-evidence-packages/WAVE.md`
- `docs/specs/2026-05-13-j-apportionment-evidence-packages-goal.md`
- `context/waves/2026-05-13-paper-rubric-uplift/CLOSE.md`
- `crates/bisect-apportion/src/huntington_hill.rs`
- `crates/bisect-apportion/src/divisor_methods.rs`

## Execution Notes

- Source URL:
  `https://www2.census.gov/programs-surveys/decennial/2020/data/apportionment/apportionment-2020-table01.xlsx`
- Source XLSX SHA-256:
  `93e7e77a222f078c0af32457af2ecc7bcae2bcb9db0cedca4ad93ff3f99b55bf`
- The checked-in extracted JSON is the fixture; the source SHA binds it to the
  Census download used for extraction.
