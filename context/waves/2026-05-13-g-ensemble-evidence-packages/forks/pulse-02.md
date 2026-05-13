# Fork Context - Pulse 02 Fixtures and Consumer Validation

## Contract

Execute `pulses/02+fixtures-consumer-validation.md` end to end.

## Required Reads

- `context/waves/PHASES.md`
- `context/waves/2026-05-13-g-ensemble-evidence-packages/WAVE.md`
- `context/waves/2026-05-13-g-ensemble-evidence-packages/pulses/02+fixtures-consumer-validation.md`
- `docs/specs/2026-05-13-g-ensemble-evidence-packages-goal.md`
- `crates/bisect-ensemble/src/evidence_manifest.rs`

## Execution Notes

- Positive fixtures must be synthetic unless real external traces are present.
- Negative fixtures should fail for a specific reason and keep the parser valid.
- Do not cite fixture values as G.1-G.3 empirical evidence.
