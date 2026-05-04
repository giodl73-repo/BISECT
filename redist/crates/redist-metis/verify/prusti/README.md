# Prusti Verification

Deductive verification of redist-metis postconditions using Prusti 0.2.x.

## Three legal postconditions

All verified on the public `Partitioner::split()` output:

1. **Coverage**: `result.assignment.len() == graph.n()`
   - Every vertex is assigned to exactly one partition

2. **Validity**: `forall |i: usize| i < result.assignment.len() ==> result.assignment[i] < result.k`
   - All partition IDs are in the valid range [0, k)

3. **Balance**: `population_balance(&result, graph) <= params.epsilon`
   - Population balance ≤ ε (machine-verified, not tested)

## Proof artifacts

Prusti generates Viper intermediate verification language (`.vpr`) files committed to `artifacts/`.
These are legally archivable proof objects suitable for expert testimony (not just CI pass/fail logs).

## Directory structure

- `GAPS.md` — Functions that cannot be verified (currently zero)
- `artifacts/` — Committed Viper proof files (.vpr)
- Verified functions: flagged with `#[verify]` or postconditions in trait implementations

## CI gate

- **On PR**: Advisory run (`continue-on-error`). Failures do not block merge.
- **On release tag**: Blocking gate. Zero GAPS required.

## Implementation status

- Task 19: Prusti postconditions (pending)

See redist-metis-verify.md for full specification.
