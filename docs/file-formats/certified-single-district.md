# Certified Single-District Package

`certified-single-district-instance-v1` and
`certified-single-district-certificate-v1` cover States with one congressional
district.

## Certificate Claims

- every canonical unit appears exactly once;
- every assignment label is district `0`;
- the population total matches the instance;
- the full RCTX graph is connected;
- the weighted inter-district boundary cut is zero; and
- the certificate and instance identities are SHA-256 bound.

No optimization solver is required. With one district there is no nontrivial
partition, boundary objective, or canonical tie.

## Package Files

- `single-district-instance.json`
- `single-district-certificate.json`
- `manifest.json`

## Verification

```powershell
cargo run -p bisect-ilp --example certified_single_district -- `
  verify-package <package-dir>
```

The committed fixture package also has an independent Python verifier and
hostile omitted-assignment, nonzero-assignment, and disconnected-graph cases.

## Claim Boundary

This certificate proves wall-to-wall assignment and connectivity for a
one-district context. It does not establish source-data accuracy beyond the
hash-bound RCTX inputs.
