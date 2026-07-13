---
pulse: 04
title: Boundary proof search
status: in_progress
depends_on: 03
wave: ri-boundary-certification
validation_level: external proof
---

# Pulse 04 - Boundary Proof Search

Run proof-producing boundary decisions until SAT supplies a better incumbent or
VeriPB accepts the optimality proof.

## Current Result

- exact 548,689 and 548,690 right-population branches cover the population
  floor;
- parent-depth v1 reached `TIMELIMIT` at 300 seconds on both branches;
- parent-depth v2 removed unassigned-depth symmetry and fixed the canonical
  left root, but also reached `TIMELIMIT` at 300 seconds on both branches;
- default LP relaxation spent 241 of 300 seconds in LP without a conclusion;
- parent-depth v3 removes arbitrary root choice for both children, but its
  low-population branch also reached `TIMELIMIT` at 300 seconds;
- no SAT counterexample or UNSAT proof has been produced; and
- boundary and canonical optimality remain unresolved.

## Discovery And Cutset Update

- `cutset-v1` reduces the initial Rhode Island branch to 194,406 variables and
  521,137 emitted constraints, with graph-checked component cuts;
- the zero-cut relaxation still reached `TIMELIMIT` at 120 and 600 seconds;
- a deterministic 32-seed METIS ensemble accepted 13 connected
  population-floor candidates and rejected 19 during exact validation;
- seed 4 plus full deterministic refinement reduced the incumbent from
  97,994,953 to 64,132,468; and
- all earlier boundary probes are superseded by the new incumbent; and
- both current exact-population branches reached `TIMELIMIT` at 600 seconds.
- the top-three METIS candidates define two connected stable cores containing
  21,213 blocks and a 4,436-block disagreement region;
- fixing the stable cores with equality constraints still timed out because the
  full variable set remained in the model; and
- the next backend must eliminate fixed variables and compile only the
  disagreement band, while treating that branch as heuristic rather than
  globally exhaustive.
- true variable elimination reduced the elite-three branch to 16,081 variables
  and the elite-two branch to 5,388 variables;
- RoundingSat still timed out on those reduced branches; and
- SciPy HiGHS optimized the elite-two fixed-core branch and produced a
  connected assignment with weighted cut 49,081,395, independently accepted
  by the Rust certified split validator.
- RoundingSat proved no lower cut exists inside that elite-two fixed-core
  branch, and VeriPB 3.0.2 accepted the streamed proof;
- the proof is retained as a 5.62 GB gzip artifact under ignored local custody;
  and
- the result remains branch-only because the heuristic fixed cores do not cover
  every Rhode Island assignment.
- pairwise seed-4/28 and seed-4/11 bands subsequently produced connected cuts
  43,806,724 and 43,885,450; and
- additional pairwise search was frozen at 43,806,724;
- a nested shell ladder subsequently reduced the cut to 43,628,645 at one hop,
  43,156,153 at two hops, and 43,047,238 at four hops; and
- 43,047,238 is the final frozen heuristic incumbent.
- the unrestricted root-free relaxation removes 102,596 connectivity-witness
  variables but still reaches `TIMELIMIT` with and without LP;
- an exact two-way decomposition now separates the four-hop fixed-core branch
  from its complement, where at least one core block must change;
- HiGHS found one disconnected 42,989,485 witness inside the fixed-core branch
  and generated its exact single-component connectivity cut; and
- both the strengthened fixed-core branch and the exact outside-core complement
  still reach `TIMELIMIT`.
- redundant cut-edge cardinality bounds were tested and removed after reducing
  solver throughput without improving classification.
- Exact, a proof-producing RoundingSat fork, returned `UNKNOWN` on both the
  91,810-variable root-free model and the 23,192-variable strengthened branch;
- VeritasPBLib translated the root-free OPB into a certified adder CNF with
  1,427,694 variables and 13,623,141 clauses;
- Kissat 4.0.4 and CaDiCaL 3.0.0 both returned `UNKNOWN` after 900 seconds; and
- solver substitution without additional mathematical decomposition is closed.
- five disjoint county-population branches now exhaust the population-optimal
  feasible set;
- the central county branch eliminates 10,270 forced positive-population blocks
  and compiles to 56,932 variables;
- Providence County has 134 population-pure tracts and only 11 split tracts;
- fixing only pure-tract positive labels yields an exact central/complement
  decomposition with a 23,669-variable central branch; and
- both reduced central branches still reach `TIMELIMIT`.
