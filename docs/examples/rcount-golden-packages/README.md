# RCOUNT Golden Packages

These packages are tiny synthetic election-count fixtures. They are not real
election data and make no legal certification claim.

## Packages

| Package | Generator | Purpose |
|---|---|---|
| `summary-basic` | `cargo run -p rcount-io --example summary_basic_package` | One contest, two precincts, one jurisdiction total; verifies `contest_selection_sum` and `jurisdiction_contest_total`. |
| `canvass-correction` | `cargo run -p rcount-io --example canvass_correction_package` | Unofficial and canvassed snapshots where a public correction event explains the changed total. |
| `tampered-source` | copied from `summary-basic`, then raw source bytes edited | Negative fixture: arithmetic still passes, but `source_hash_match` fails. |
| `missing-source-hash` | copied from `summary-basic`, then source index emptied | Negative fixture: package records omit the raw source hash evidence. |

The verifier surface is still crate-level while RCOUNT is incubating. The first
fixtures are generated from `rcount_core` synthetic packages and written
through `rcount-io`.

The real verifier transcript is generated with:

```text
cargo run -p rcount-audit --example write_summary_basic_transcript
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/canvass-correction --write-transcript
```

The CLI verifier can check the package directly:

```text
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/summary-basic
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/canvass-correction
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/tampered-source
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/missing-source-hash
```
