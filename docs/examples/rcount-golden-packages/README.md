# RCOUNT Golden Packages

These packages are tiny synthetic election-count fixtures. They are not real
election data and make no legal certification claim.

## Packages

| Package | Generator | Purpose |
|---|---|---|
| `summary-basic` | `cargo run -p rcount-io --example summary_basic_package` | One contest, two precincts, one jurisdiction total; verifies `contest_selection_sum` and `jurisdiction_contest_total`. |
| `canvass-correction` | `cargo run -p rcount-io --example canvass_correction_package` | Unofficial and canvassed snapshots where a public correction event explains the changed total. |
| `mail-batch-added` | `cargo run -p rcount-io --example mail_batch_added_package` | Batched precinct summaries where a late mail batch is declared and reconciled. |
| `precinct-split-lineage` | `cargo run -p rcount-io --example precinct_split_lineage_package` | Cross-cycle reporting-unit lineage: one precinct split and two precincts merged. |
| `privacy-inclusion-sketch` | `cargo run -p rcount-io --example privacy_inclusion_sketch_package` | Receipt-safe inclusion proof: an anonymized token is present without choices. |
| `district-aggregation-rplan` | `cargo run -p rcount-district --example district_aggregation_rplan` | Optional RPLAN bridge: verified precinct summaries are assigned into district totals with package and plan hashes. |
| `bad-selection-sum` | `cargo run -p rcount-io --example bad_selection_sum_package` | Negative fixture: manifest and source hashes verify, but local contest arithmetic fails. |
| `missing-batch` | `cargo run -p rcount-io --example missing_batch_package` | Negative fixture: manifest and source hashes verify, but a batch summary references absent batch evidence. |
| `bad-lineage` | `cargo run -p rcount-io --example bad_lineage_package` | Negative fixture: manifest and source hashes verify, but lineage references a missing current unit. |
| `choice-bearing-proof` | `cargo run -p rcount-io --example choice_bearing_proof_package` | Negative fixture: proof reveals a candidate selection and fails the privacy gate. |
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
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/mail-batch-added
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/precinct-split-lineage
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/privacy-inclusion-sketch
cargo run -p rcount-cli -- aggregate-districts docs/examples/rcount-golden-packages/district-aggregation-rplan/package --plan docs/examples/rcount-golden-packages/district-aggregation-rplan/plan.rplan.json
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/bad-selection-sum
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/missing-batch
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/bad-lineage
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/choice-bearing-proof
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/tampered-source
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/missing-source-hash
```
