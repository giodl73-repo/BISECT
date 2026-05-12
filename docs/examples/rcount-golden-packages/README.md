# RCOUNT Golden Packages

These packages are tiny synthetic election-count fixtures. They are not real
election data and make no legal certification claim.

## Packages

| Package | Generator | Purpose |
|---|---|---|
| `summary-basic` | `cargo run -p rcount-io --example summary_basic_package` | One contest, two precincts, one jurisdiction total; verifies `contest_selection_sum` and `jurisdiction_contest_total`. |

The verifier surface is still crate-level while RCOUNT is incubating. The first
public fixture is generated from `rcount_core::synthetic_summary_basic_package`
and written through `rcount-io`.

The real verifier transcript is generated with:

```text
cargo run -p rcount-audit --example write_summary_basic_transcript
```

The CLI verifier can check the package directly:

```text
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/summary-basic
```
