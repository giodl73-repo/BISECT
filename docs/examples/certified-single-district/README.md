# Certified Single-District Fixtures

The `grid3x3/` package demonstrates the one-district contract on a connected
nine-unit RCTX fixture.

The `negative/` corpus must be rejected:

- omitted assignment unit;
- nonzero district label; and
- disconnected instance graph.

## Replay

```powershell
python scripts/research/verify_certified_single_fixtures.py

cargo run -p bisect-ilp --example certified_single_district -- `
  verify-package docs/examples/certified-single-district/grid3x3
```

## Claim Boundary

Synthetic contract evidence only; real State packages are produced separately.
