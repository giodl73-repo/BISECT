# Exact Canonical E0 Fixtures

This corpus provides positive and negative package evidence for the bounded
Exact Canonical Benchmark reference implementation.

## Fixtures

### `path4-optimal`

- four canonically ordered units;
- equal population;
- path adjacency;
- unique optimum `[0,0,1,1]`;
- objective `(0, 0, 1)`; and
- full RPLAN/RCTX/audit package with an ordered-search proof transcript.

### `three-islands-infeasible`

- three isolated units;
- two required districts;
- no assignment can make both districts non-empty and connected; and
- exact infeasibility certificate with no fabricated plan; and
- exact infeasibility proof transcript committing to every rejected candidate.

### `negative-corpus`

Five committed adversarial submissions:

- a false optimum;
- false infeasibility;
- a noncanonical primary-objective tie;
- a disconnected assignment; and
- certificate-hash tampering.

Each case contains an exact instance, submitted certificate, submitted proof,
and `expected.json` rejection contract. These are deliberately invalid
submissions, not solver outputs that should verify.

## Replay

```powershell
bisect exact `
  --method canonical-exhaustive `
  --context docs\examples\exact-canonical\path4-optimal\input.rctx `
  --out-dir <temp>\path4 `
  --districts 2 `
  --tolerance 1.0 `
  --exact-fixture-limit 8 `
  --generated-at 2026-07-10T12:00:00Z
```

Use the analogous command for `three-islands-infeasible`.

Verify committed fixture hashes:

```powershell
python scripts\research\verify_exact_canonical_fixtures.py
cargo test -p bisect-ilp --test exact_negative_corpus -- --test-threads=1
```

Verify one valid artifact triplet directly:

```powershell
cargo run -p bisect-ilp --example exact_canonical -- verify `
  docs\examples\exact-canonical\path4-optimal\output\exact-canonical-instance.json `
  docs\examples\exact-canonical\path4-optimal\output\exact-canonical-certificate.json `
  docs\examples\exact-canonical\path4-optimal\output\exact-canonical-proof.json
```

Run the independent Python verifier over the complete corpus:

```powershell
python scripts\research\verify_exact_canonical_independent.py corpus
```

Its deterministic result is committed as
`independent-verifier-report.json`. The Python implementation does not import,
execute, or link the Rust solver or reference verifier.

Rebuild the hostile corpus:

```powershell
python scripts\research\build_exact_negative_corpus.py
```

## Claim Boundary

These fixtures prove the E0 bounded package and certificate contract. They do
not prove production solver scalability, national block-level exactness, legal
validity, or independent second-verifier acceptance. Negative cases begin from
exact JSON artifacts and do not replay RCTX-to-instance conversion or CLI plan
emission. The second verifier establishes independent agreement only for this
bounded E0 model and corpus.
