---
skill: simulate-contract
topic: block-ensemble-expansion-v2
date: 2026-08-11
gate_result: PASS
census-distribution: software-contract/non-census
gate-provenance: §S5.5-Sub-task-A
gate-status: PASS
attestation-by: Pulse 29 protocol owner
attestation-result: implementation matches the frozen pre-Stage-0 contract
verification-by: focused v2 verifier and adversarial unit suite
verification-result: 24 focused tests and empty-package verification pass
---

# Block-Ensemble Expansion v2 Contract Simulation

## Scope

Contract:
`docs/specs/2026-08-11-nrs-v0.3-block-ensemble-expansion-v2.md`

Implementation:

- `scripts/research/run_block_ensemble_expansion_v2.py`
- `scripts/research/verify_block_ensemble_expansion_v2.py`
- `tests/unit/test_block_ensemble_expansion_v2.py`
- `docs/experiments/nrs-v0.3-block-ensemble-expansion-v2/ledger.json`

This is an implementation-gate audit. It does not attest to input custody,
release-executable custody, Stage 0, governed execution, convergence, or replay.

## GateTokenSchema Sweep

| Field | Observed value | Result |
|---|---|---|
| `census-distribution` | `software-contract/non-census` | PASS |
| `gate-provenance` | `§S5.5-Sub-task-A` | PASS |
| `gate-status` | `PASS` | PASS |
| `attestation-by` | `Pulse 29 protocol owner` | PASS |
| `attestation-result` | frozen pre-Stage-0 contract matched | PASS |
| `verification-by` | v2 verifier and adversarial unit suite | PASS |
| `verification-result` | 24 focused tests plus empty-package verifier | PASS |

`SCHEMA-DIFF-COMPLETE`: all required rows are present, co-required fields are
paired, and attestation and verification roles are non-null.

## Element Diff

| ID | Contract element | Implementation evidence | Result |
|---|---|---|---|
| C01 | Dedicated v2 protocol identity | strict `PROTOCOL_ID` constant | PASS |
| C02 | Dedicated v2 ledger schema | strict `LEDGER_SCHEMA`; v1 rejected | PASS |
| C03 | Fresh empty custody | committed ledger has zero completions/bytes/wall | PASS |
| C04 | No v1 completion reuse | verifier rejects v1 `protocol_id` artifacts | PASS |
| C05 | Fresh seed | command and trace validator require `20260811` | PASS |
| C06 | Fixed cohort | CLI choices and State district map freeze NH/NM/GA | PASS |
| C07 | Fixed kernel set | CLI accepts only Wilson and Kruskal | PASS |
| C08 | Frozen order | prefix validation and `expected_next` gate every phase | PASS |
| C09 | Four governed chains | command and trace validator require four | PASS |
| C10 | 2,000 governed steps | command and trace validator require 2,000 | PASS |
| C11 | 25-step excluded preflight | separate execution class and one-chain shape | PASS |
| C12 | Stage 0 before primaries | ledger rejects primary before all preflight replays | PASS |
| C13 | All primaries before replay | ledger rejects early governed replay | PASS |
| C14 | Official fresh package only | CLI resolves and requires the v2 package path | PASS |
| C15 | Admission before process | process creation delegates to `launch_if_admitted` | PASS |
| C16 | Recheck after rejection | monotonic admission-attempt custody preserves rejects | PASS |
| C17 | Non-overwriting admission custody | attempt paths are new and adapter uses exclusive create | PASS |
| C18 | Memory ceiling | monitor enforces 2,415,919,104 bytes | PASS |
| C19 | Cumulative wall ceiling | monitor enforces 21 hours on primary/replay | PASS |
| C20 | Storage ceilings | retained and replay scratch checks enforce 3 GiB | PASS |
| C21 | Terminal failure semantics | failure helper closes ledger without completion | PASS |
| C22 | Exact replay | normalized equality precedes scratch deletion | PASS |
| C23 | Deterministic primary custody | gzip uses empty filename and `mtime=0` | PASS |
| C24 | Resource/admission binding | resource record hashes its successful admission record | PASS |
| C25 | Wrapper/source/protocol binding | resource record hashes all three plus executable | PASS |
| C26 | Empty active package verification | dedicated verifier reports 0/6, 0/6, 0/6 PASS | PASS |

## Mismatch History

### M01 — rejected admission blocked permitted recheck

- Severity: blocking
- Initial behavior: one fixed admission filename existed per phase and key.
- Contract conflict: a prelaunch rejection may be remediated and checked again,
  while prior custody must not be overwritten.
- Remediation: monotonic `attempt-01`, `attempt-02`, ... admission records;
  verifier checks every attempt and binds each completed process to one passing
  record.
- Resolution: closed; adversarial retry-custody test passes.

## Residual Boundaries

- No runner process was created during this pulse.
- Input hashes and release-executable custody remain a later Stage 0 gate.
- The verifier has exercised only the empty active package plus synthetic
  negative fixtures; populated trace verification remains unclaimed.
- A PASS here authorizes review of implementation, not preflight or governed
  execution.

## Gate Result

**PASS — implementation contract only.** No schema omission remains and the
blocking admission-recheck mismatch is remediated. Stage 0 remains closed.
