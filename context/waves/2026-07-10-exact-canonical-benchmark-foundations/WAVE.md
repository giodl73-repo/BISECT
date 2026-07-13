---
wave: exact-canonical-benchmark-foundations
date_open: 2026-07-10
status: complete
date_close: 2026-07-10
source_goal: exact canonical benchmark north star
vtrace_posture: internal_engineering_baseline_only
---

# Exact Canonical Benchmark Foundations

## Mission

Implement the first certifiable slice of the Exact Canonical Benchmark:
enacted-model identity, lexicographic objective, canonical tie-breaking,
optimality and infeasibility certificates, and solver-independent verification.

## North-Star Contract

For a declared instance and profile, select:

1. minimum maximum population deviation;
2. minimum total population deviation;
3. minimum weighted boundary cut; and
4. lexicographically smallest canonical assignment.

The output must be either:

- an optimal feasible-assignment certificate; or
- an exact infeasibility certificate.

## Claim Boundary

This wave begins with bounded synthetic `k=2` instances. It must not claim
national block-level exactness, production solver readiness, legal validity, or
two-verifier completion until the applicable pulse closes.

## Success Metrics

| Metric | Baseline | Target |
|---|---|---|
| Exact objective | Edge-cut-only bounded solver | Four-level lexicographic objective |
| Canonical tie handling | Incidental search order | Explicit unique assignment |
| Exact certificate | Branch-and-cut metadata | Versioned optimal/infeasible certificate |
| Verifier | Audit checks feasibility only | Solver-independent exact verifier |
| Negative coverage | Basic infeasible status | Tamper, false optimum, and false infeasibility rejection |
| Real-data readiness | No exact North-Star fixture | At least one small real-State certificate or explicit blocker |

## Pulse Status

| Pulse | Status | Outcome |
|---|---|---|
| 01 - Exact objective and certificate schema | DONE | Bounded exhaustive k=2 solver, optimal/infeasible certificate, reference verifier |
| 02 - Exact CLI package integration | DONE | CLI, optimal/infeasible package manifests, RPLAN/RCTX audit, committed fixtures |
| 03 - Proof artifact and verifier contract | DONE | Hash-bound ordered-search transcript, CLI artifact, tamper-rejecting verifier API |
| 04 - Negative certificate corpus | DONE | Five committed hostile submissions with declared verifier rejection classes |
| 05 - Second verifier implementation | DONE | Independent Python verifier accepts both positives and rejects all five hostile submissions |
| 06 - Small-State exact benchmark | DONE | RI has 25,649 matched blocks; E0 limit 24 and `2^25648-1` search establish explicit model/compute blocker |

## Validation

Every pulse runs:

```powershell
cargo fmt --all -- --check
git --no-pager diff --check
```

Code pulses name targeted crate tests and certificate-verifier commands.

## Closure Rule

Close only when every completed exact claim is backed by a certificate and a
verifier. Heuristic, bounded-gap, timeout, and formulation-only results remain
distinct.
