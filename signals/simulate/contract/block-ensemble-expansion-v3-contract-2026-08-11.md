---
skill: simulate-contract
topic: block-ensemble-expansion-v3
date: 2026-08-11
gate_result: PASS
census-distribution: software-contract/non-census
gate-provenance: §S5.5-Sub-task-A
gate-status: PASS
attestation-by: v3 protocol owner
attestation-result: implementation and compiled readiness match the frozen pre-Stage-0 contract
verification-by: Rust tests, Python unit suite, compiled probe replay, and package verifiers
verification-result: 12 positive and 2 negative compiled probes plus pristine-package verification pass
---

# Block-Ensemble Expansion v3 Contract Simulation

## Scope

Contract: `docs/specs/2026-08-11-nrs-v0.3-block-ensemble-expansion-v3.md`

Implementation: the v3 runner, package verifier, readiness preparer and
verifier, compiled `block_trace` release example, focused tests, and pristine
v3 package. This audit covers readiness only. It does not authorize or attest
to Stage 0, governed chains, convergence, or replay.

## GateTokenSchema Sweep

| Field | Observed value | Result |
|---|---|---|
| `census-distribution` | `software-contract/non-census` | PASS |
| `gate-provenance` | `§S5.5-Sub-task-A` | PASS |
| `gate-status` | `PASS` | PASS |
| `attestation-by` | `v3 protocol owner` | PASS |
| `attestation-result` | frozen readiness contract matched | PASS |
| `verification-by` | compiled probes and independent package checks | PASS |
| `verification-result` | all 14 probes and both verifiers pass | PASS |

`SCHEMA-DIFF-COMPLETE`: all required rows are present, co-required fields are
paired, and attestation and verification roles are non-null.

## Element Diff

| ID | Contract element | Implementation evidence | Result |
|---|---|---|---|
| C01 | Fresh v3 identity | strict protocol constant and official package path | PASS |
| C02 | Fresh ledger schema | `nrs-block-ensemble-expansion-ledger-v3` required | PASS |
| C03 | Closed predecessor isolation | verifier rejects v1 and v2 protocol artifacts | PASS |
| C04 | Fresh seed | wrapper, trace validator, and Rust runner require `20260812` | PASS |
| C05 | Fixed cohort | NH/NM/GA map freezes 2/3/14 districts | PASS |
| C06 | Fixed samplers | wrapper and compiled runner admit Wilson/Kruskal only | PASS |
| C07 | Frozen schedule | prefix ledger validation enforces six-cell order | PASS |
| C08 | Preflight shape | v3 class requires 25 steps and one chain | PASS |
| C09 | Governed shape | v3 class requires 2,000 steps and four chains | PASS |
| C10 | Stage ordering | six preflights and replays precede any primary | PASS |
| C11 | Primary/replay ordering | all primaries precede governed replay | PASS |
| C12 | Compiled positive matrix | 3 States × 2 samplers × 2 shapes return zero | PASS |
| C13 | Wrong-seed control | v3 class with `20260811` returns nonzero | PASS |
| C14 | Predecessor-class control | v2 class with v3 seed returns nonzero | PASS |
| C15 | Probe side-effect boundary | argv omits RCTX, assignments, and output | PASS |
| C16 | Probe replay | readiness verifier reruns argv and matches code/stdout/stderr | PASS |
| C17 | Executable custody | probe and readiness bind identical release SHA-256 | PASS |
| C18 | Source custody | runner, wrapper, verifiers, preparer, and protocol are bound | PASS |
| C19 | Input custody | all three audits and underlying input hashes verify | PASS |
| C20 | Empty package | ledger has zero completions, bytes, wall time, and failures | PASS |
| C21 | No process artifacts | readiness rejects admission/resource/trace artifacts | PASS |
| C22 | Capacity observation | 95,035,809,792 free exceeds 8,589,934,592 required | PASS |
| C23 | Per-launch admission | wrapper delegates process creation to admitted launch | PASS |
| C24 | Resource ceilings | memory, wall, retained, scratch, and reserve constants match | PASS |
| C25 | Terminal failure | a started nonzero process closes ledger without completion | PASS |
| C26 | Replay custody | normalized equality precedes replay scratch deletion | PASS |
| C27 | Deterministic storage | primary gzip uses empty filename and `mtime=0` | PASS |
| C28 | Claim boundary | readiness explicitly reports no ensemble result or authority | PASS |

## Mismatch History

### M01 — compiled identity was not exercised by predecessor readiness

- Severity: blocking.
- Predecessor behavior: v2 bound source and executable hashes but did not invoke
  the binary with the v2 class and seed before Stage 0.
- Remediation: the v3 binary exposes a side-effect-free contract-only path;
  readiness records and replays all frozen tuples and two negative controls.
- Resolution: closed; 14 compiled probes pass and create no package artifact.

### M02 — predecessor protocol IDs were initially checked only for v1

- Severity: blocking.
- Initial v3 copy inherited a v1-only rejection set.
- Remediation: the v3 verifier rejects both closed v1 and v2 IDs.
- Resolution: closed; focused adversarial test passes.

### M03 — successor source evolution invalidated the v2 failure check

- Severity: blocking custody regression.
- Initial behavior: the v2 terminal verifier compared its retained resource
  record to the current shared runner source hash.
- Contract conflict: adding v3 support necessarily changes that source, while
  closed v2 evidence must remain independently verifiable.
- Remediation: the v2 terminal verifier now checks the source, wrapper, and
  protocol hashes frozen in the retained v2 execution record.
- Resolution: closed; the portable v2 failure test and terminal verifier pass
  against the evolved runner.

## Residual Boundaries

- The executable is an author-machine Windows release build; reproducible build
  equivalence on another host remains unclaimed.
- Contract-only probes validate argument acceptance and early return, not input
  loading, chain execution, memory behavior, trace validity, or replay.
- The capacity snapshot is point-in-time evidence and cannot substitute for a
  fresh admission record at process launch.
- The next permissible action after review is Stage 0, beginning with NH Wilson;
  no v3 chain has yet run.

## Gate Result

**PASS — v3 implementation and compiled readiness only.** No schema omission or
contract mismatch remains. Chain execution remains separately gated.
