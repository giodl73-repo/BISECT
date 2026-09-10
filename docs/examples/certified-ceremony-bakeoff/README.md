# Certified Ceremony Bakeoff Fixture

This bounded synthetic fixture exercises the prospective two-scoreboard
bakeoff extension. It compares procedural evidence classes, not map quality:

- `certified-bisection` has a completed two-round ceremony for the path-8,
  four-district certified tree;
- `metis-replay` is retained as `replay-only` and receives no invented proof or
  ceremony credit.

The receipt strings and Unix times are deterministic test values. No external
transparency service validated them, so `external_witnesses_validated` is false.

Regenerate the procedural analysis from the repository root:

```powershell
py scripts/research/analyze_certified_ceremony_bakeoff.py `
  docs/examples/certified-ceremony-bakeoff/input.json `
  --bisect target/debug/bisect.exe `
  --out docs/examples/certified-ceremony-bakeoff/analysis.json
```

`verification_seconds` is an environment-specific diagnostic. All categorical,
count, coverage, binding, and byte-size fields are the reproducible fixture
contract.

Verify that contract and a timestamp-tamper rejection:

```powershell
py scripts/research/verify_certified_ceremony_bakeoff.py `
  docs/examples/certified-ceremony-bakeoff `
  --bisect target/debug/bisect.exe
```

The fixture reports `tamper_suite: not-run`: the verifier above exercises one
representative attack, not the full frozen production negative corpus.
