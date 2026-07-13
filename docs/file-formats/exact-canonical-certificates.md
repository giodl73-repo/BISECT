# Exact Canonical Certificates

## Schemas

- Instance: `exact-canonical-instance-v1`
- Certificate: `exact-canonical-certificate-v1`
- Proof transcript: `exact-canonical-proof-v1`
- Package manifest: `exact-canonical-package-manifest-v1`
- Model: `exact-canonical-k2-exhaustive-v1`

## Exact Instance

The bounded E0 instance contains:

- canonically ordered unique unit IDs;
- non-negative integer populations;
- positive integer weighted edges with `left < right`;
- `k = 2`; and
- at most 24 units.

The instance hash is canonical JSON SHA-256.

## Objective

The solver selects lexicographically:

1. minimum maximum scaled population deviation;
2. minimum total scaled population deviation;
3. minimum weighted boundary cut; and
4. minimum canonical assignment vector.

Scaled deviation for district `d` is:

```text
abs(k * district_population[d] - total_population)
```

## Result Types

### Optimal

Contains the canonical assignment, full objective tuple, exhaustive-search
statistics, and lower bound equal to the optimum.

### Infeasible

Contains an exhaustive proof record with zero feasible assignments. No RPLAN or
audit certificate is emitted because no plan exists.

## Proof Transcript

`exact-canonical-proof.json` is separate from the result certificate. It
records:

- instance and model identity;
- the fixed-label and ascending nonzero-mask enumeration rule;
- candidate, feasible, and primary-objective tie counts;
- the exact lower bound and canonical assignment, when one exists; and
- a SHA-256 search commitment over every candidate in enumeration order.

Masks run from `1` through `2^(n-1)-1`; mask `0` is excluded because it leaves
district 1 empty. Each candidate contributes its mask and feasibility status. Feasible
candidates additionally contribute the complete objective tuple and assignment
bytes. The transcript ID is a canonical JSON hash of all proof fields. The
certificate binds that ID, so neither artifact can be substituted alone.

## Package

```powershell
bisect exact `
  --method canonical-exhaustive `
  --context fixture.rctx `
  --out-dir exact-output `
  --districts 2 `
  --tolerance 1.0 `
  --exact-fixture-limit 24
```

Always emitted:

- `exact-canonical-instance.json`
- `exact-canonical-certificate.json`
- `exact-canonical-proof.json`
- `exact-package-manifest.json`

Emitted for an optimal feasible result:

- `exact.rplan`
- `exact.rctx`
- `audit-certificate.json`

The package manifest hashes every present artifact and records the declared
population audit tolerance.

## Verification

`bisect-ilp::verify_exact_canonical_artifacts` checks:

- schemas, model, instance hash, and certificate ID;
- canonical unit ordering and numeric bounds;
- submitted result and proof statistics;
- proof ID and certificate-to-proof binding;
- the ordered search commitment; and
- the exact result through fresh bounded enumeration.

This verifier does not trust submitted solver output, but it uses the same
crate-level enumeration implementation as the generator.

`scripts/research/verify_exact_canonical_independent.py` is the second
implementation. It independently performs schema and numeric validation,
canonical hashing, connectivity checks, objective evaluation, exhaustive
enumeration, canonical tie-breaking, and transcript reconstruction in Python.
It does not import, execute, or link `bisect-ilp`.

### Adversarial Corpus

`docs/examples/exact-canonical/negative-corpus/` commits five invalid
certificate/proof submissions with machine-readable expected rejection
classes. `crates/bisect-ilp/tests/exact_negative_corpus.rs` loads those files
through the public verifier API and requires rejection of false optimality,
false infeasibility, noncanonical tie selection, disconnected assignment, and
certificate-ID tampering.

Both implementations accept the two positive fixtures and reject all five
adversarial submissions. The deterministic Python verifier result is committed
as `docs/examples/exact-canonical/independent-verifier-report.json`.

## Real-State Frontier

`docs/experiments/exact-canonical/ri-2020-block-frontier.json` records the
first statutory-unit real-State gate. Rhode Island has 25,649 matched 2020
Census blocks, while the E0 oracle supports 24 units. Its exhaustive search
would contain `2^25648-1` candidates. The artifact is a verified blocker
report, not an exact certificate.

## Claim Boundary

This format proves exactness only for the identified E0 bounded model and
instance. It does not prove national scalability, legal validity, block-level
readiness, or exactness under a different feasible set or objective.
