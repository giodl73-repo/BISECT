---
wave: certified-recursive-bisection
date_open: 2026-07-10
status: complete
date_close: 2026-07-10
source_goal: certify every enacted standard-bisect cut
vtrace_posture: internal_engineering_baseline_only
---

# Certified Recursive Bisection

## Mission

Turn the existing `standard-bisect` construction into a certifiable procedure:
at every tree node, prove the unique best connected cut for the enacted
`k_left:k_right` seat ratio, then bind the child certificates into one
canonical recursive plan.

## Algorithm Contract

The split schedule remains `bisect-core::BisectionTree`:

```text
k_left  = floor(k / 2)
k_right = k - k_left
```

Thus California 2020 begins:

```text
52 -> 26/26
26 -> 13/13
13 -> 6/7
```

For each parent node, select lexicographically:

1. minimum maximum seat-ratio-scaled population deviation;
2. minimum total seat-ratio-scaled population deviation;
3. minimum weighted boundary cut; and
4. minimum canonical left/right assignment.

Both children must be nonempty and connected. The selected cut becomes the
fixed parent of all downstream cuts.

## Claim Boundary

This wave certifies the unique map produced by the enacted recursive BISECT
procedure. It does not claim global optimality among all unrestricted
districtings or permit a solver to alter the bisection tree.

## Success Metrics

| Metric | Baseline | Target |
|---|---|---|
| Exact split target | Equal-population `k=2` only | Arbitrary canonical `k_left:k_right` |
| Recursive identity | Final assignment only | Hash-bound split-certificate tree |
| Tree fidelity | Heuristic runtime schedule | Certified `BisectionTree` schedule |
| Tie handling | Per-fixture assignment | Canonical orientation at every node |
| Proof backend | Exhaustive E0 only | Solver/proof separation contract and prototype |
| Real-data frontier | RI exhaustive blocker | Block RCTX plus scalable certification attempt |

## Pulse Status

| Pulse | Status | Outcome |
|---|---|---|
| 01 - Certified split contract | DONE | Ratio objective, canonical schedule/orientation, parent and unit identity schemas |
| 02 - Generalized bounded split oracle | DONE | Exact equal/unequal-ratio oracle, transcript, optimal/infeasible verifier |
| 03 - Recursive certificate tree | DONE | Canonical BFS schedule, exact child derivation, parent links, complete leaf coverage |
| 04 - CLI recursive package | DONE | CLI tree/RPLAN/audit package plus five-case hostile recursive corpus |
| 05 - Proof-producing backend contract | DONE | Discovery contract, three-stage OPB compiler, SAT counterexample fixture; proof tools unavailable |
| 06 - Rhode Island certified split frontier | DONE | Hash-bound connected 25,649-block RCTX with 64 established island bridges; scalable proof-tool blockers |
| 07 - Public narrative and adoption docs | DONE | README, plain-language explainer, readiness table, and adoption language |
| 08 - U.21 paper and portfolio alignment | DONE | U.21, ten aligned PDFs/sources, claim review, and path-8 METIS comparison |

## Validation

Every pulse runs:

```powershell
cargo fmt --all -- --check
git --no-pager diff --check
```

## Closure Rule

Close only when recursive certificates prove the exact enacted split schedule,
child universes cover their parent exactly, and every completed claim has two
verifier paths or an explicit scoped blocker.
