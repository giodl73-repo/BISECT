# J Apportionment Evidence Packages Closeout

**Status:** complete

## Outcome

J-track apportionment claims now have one active Census/SHA evidence package.
The 2020 Census Table 1 fixture is source-hash-bound and replayed by
`bisect-apportion` tests against the official 435-seat Huntington-Hill result.

## Delivered

- Added `bisect-apportion::evidence_manifest` with
  `j-apportionment-evidence-manifest v1`.
- Added
  `docs/examples/j-apportionment-evidence-packages/2020-census-table01/`.
- Recorded the official Census XLSX URL and SHA-256:
  `93e7e77a222f078c0af32457af2ecc7bcae2bcb9db0cedca4ad93ff3f99b55bf`.
- Added verifier tests for manifest hash binding, Huntington-Hill replay, and
  tampered-seat detection.
- Updated J.0, J.1, J.6 paper text, PDFs, `docs/PAPERS.md`, and the scorecard.

## Validation

```powershell
cargo fmt
cargo test -p bisect-apportion
git diff --check
```

## Carry-Forwards

- Add 2010 and 2000 Census apportionment packages with the same manifest shape.
- Add a standalone `bisect apportion --verify` CLI path if J papers need a
  public command rather than test-only replay.
- Consider packaging the full 1910-2020 Table C2 history before making
  historical apportionment replay claims.
