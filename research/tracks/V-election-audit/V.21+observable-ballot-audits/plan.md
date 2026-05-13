# V.21 Plan

## Thesis

Observable ballot-level audits need privacy-preserving linkage and public
commitment records. SOBA-style evidence is the bridge between ballot-level
comparison math and public observability.

## Atlas

- `docs/algorithm-atlas/v21-soba-observable-ballot-audits.md`

## Implementation Tasks

- [x] Add toy commitment/opening fixture.
- [x] Add missing-opening negative fixture.
- [ ] Add CVR mismatch fixture that can feed V.14 comparison math.
- [x] Add privacy-boundary checks for what RCOUNT must not publish.
- [x] Align with V.4 privacy-safe inclusion proof vocabulary.

## Claim Boundary

RCOUNT can verify commitments and transcript consistency. It must not expose
secret ballot order or infer voter identity.
