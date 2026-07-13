# Exact Canonical Benchmark North Star

**Status:** Research and engineering goal  
**Purpose:** Define the closest redistricting analogue to Huntington--Hill
operational finality

## North-Star Claim

Given enacted axioms, a canonical census instance, and a verified exact
certificate, one benchmark assignment is selected without practitioner choice.

The claim is conditional:

> This is the unique assignment selected by the enacted lexicographic
> objective. The certificate proves feasibility, optimality, and canonical
> tie-breaking.

It does not claim the enacted axioms are politically or constitutionally
inevitable.

## 1. Enacted Feasible Set

Congress fixes:

- block universe and population;
- district count;
- complete assignment;
- contiguity;
- legal population rule;
- adjacency and bridge records;
- prohibited benchmark inputs; and
- treatment of infeasible instances.

VRA, State-law, community, and court modifications remain outside the exact
geographic benchmark and require the governed decision record.

## 2. Lexicographic Objective

Among feasible plans, minimize in order:

1. maximum absolute population deviation;
2. total absolute population deviation;
3. shared-boundary cut cost; and
4. canonical assignment vector.

The canonical assignment vector is formed by:

1. sorting blocks by GEOID;
2. canonicalizing district labels by the lowest GEOID in each district; and
3. selecting the lexicographically smallest assignment sequence.

The fourth objective guarantees a unique selected benchmark when multiple plans
share the first three objective values.

## 3. Exact Certificate

An exact result has one of two mutually exclusive forms:

1. an optimal feasible-assignment certificate; or
2. an exact infeasibility certificate proving that no assignment satisfies the
   enacted feasible set.

The optimal certificate contains:

- instance and legal-profile hashes;
- feasible assignment;
- district population and connectivity witnesses;
- objective tuple;
- solver-independent lower bounds matching each optimized objective;
- branch-and-cut or branch-and-price proof transcript;
- canonical labeling and tie-optimality witness;
- source, compiler, solver, and parameter identity; and
- verifier result.

The infeasibility certificate contains the same instance/profile identity plus
a solver-independent contradiction, exhaustive bound, or proof transcript
accepted by the independent verifiers.

A certificate is invalid if it proves only feasibility, a heuristic bound, or
optimality under a different model.

## 4. Independent Verifier

The verifier:

- does not trust the generating solver;
- recomputes all hashes and objective values;
- checks complete assignment and population;
- verifies district connectivity;
- validates lower-bound and cut-generation witnesses;
- confirms the incumbent matches the bound; and
- verifies no objective-equivalent lexicographically smaller assignment exists.

The verifier should be smaller and simpler than the solver and have at least
one independent implementation.

## 5. Relationship To Current BISECT

Current assets:

- `bisect-ilp` connectivity cuts and solve reports;
- `bisect-column` branch-and-price foundations;
- U.13 exact-versus-heuristic claim boundaries;
- U.20 RPLAN/RCTX audit certificates;
- canonical plan identity and hashing;
- reproducibility manifests; and
- deterministic graph/statistics kernels.

Current gaps:

- national block-level formulation;
- production exact-solver integration;
- proof-producing lower bounds;
- tie-optimality certification;
- scalable all-state benchmarks; and
- independent certificate verifier.

METIS remains the practical benchmark while these gaps remain. It must be
described as heuristic.

## Implemented Foundation

Pulse 01 of the Exact Canonical Benchmark Foundations wave now implements:

- `exact-canonical-instance-v1`;
- the four-level objective for bounded `k=2` instances;
- canonical unit ordering and district-label symmetry removal;
- exhaustive optimal feasible-assignment certificates;
- exhaustive exact infeasibility certificates;
- certificate and instance hashes;
- a submission-independent reference verifier; and
- positive and negative synthetic tests.

The implementation is in `bisect-ilp::canonical` and is limited to at most 24
units. It is the E0 reference oracle and certificate contract, not the
production solver or second independent verifier.

Pulse 02 integrates the bounded method into `bisect exact
--method canonical-exhaustive`. Feasible results emit RPLAN/RCTX, an RPLAN audit
certificate, the exact certificate, and a hash-bound package manifest.
Infeasible results emit the exact instance, infeasibility certificate, and
package manifest without fabricating a plan.

Pulse 03 adds `exact-canonical-proof-v1`, a separate deterministic transcript
that commits to every symmetry-reduced, nonempty bounded-search candidate in
ascending nonzero-mask order. The
certificate binds the proof transcript ID, and the verifier recomputes the
complete transcript and rejects proof tampering. This is a submission-
independent contract over the E0 enumeration; it is not yet the independently
implemented second verifier required by Pulse 05.

Pulse 04 commits an adversarial five-case corpus covering false optimality,
false infeasibility, canonical tie-breaking, disconnected assignments, and
certificate-hash tampering. Every case has a declared rejection class and is
loaded through the Rust verifier integration test. This closes negative
reference coverage but not the second-implementation gate.

Pulse 05 implements the second verifier in Python without importing, executing,
or linking the Rust solver or verifier. It independently reconstructs hashes,
connectivity, objective values, exhaustive search, tie selection, and proof
transcript bytes. Both implementations accept the positive corpus and reject
all five hostile submissions. This satisfies the two-verifier gate only for
the bounded E0 model, not national readiness.

Pulse 06 evaluates Rhode Island 2020 at the normative Census-block unit.
TIGER/Line and PL 94-171 custody match exactly across 25,649 blocks and
1,097,379 people, but the E0 oracle is limited to 24 units. The required
symmetry-reduced enumeration has `2^25648-1` candidates, a 7,721-digit number.
No block adjacency/RCTX artifact is currently in custody. The pulse therefore
publishes a hash-bound blocker report and does not substitute tract-level or
heuristic evidence for an exact State certificate.

The Certified Recursive Bisection wave now freezes the next contract:
`k_left=floor(k/2)`, `k_right=k-k_left`, with population deviation scaled to
that seat ratio. Equal-seat cuts fix canonical unit 0 left to remove label
symmetry; unequal cuts remain seat-oriented and do not impose that restriction.
This preserves the existing BISECT tree rather than replacing it with
unrestricted statewide optimization.

The bounded generalized oracle is now implemented for up to 24 units. It
enumerates the reduced space for equal-seat cuts and both orientations for
unequal-seat cuts, filters connectivity before scoring, and emits
hash/transcript-bound optimality or infeasibility artifacts. Recursive tree
chaining and scalable proof production remain open.

The bounded recursive tree is now implemented. Every non-leaf split is
reverified, child instances are reconstructed from the certified parent
assignment, and one-seat leaves must partition the root universe exactly once.
This certifies the map produced by sequential locally optimal cuts. It does not
yet prove that a locally optimal cut will always permit downstream completion;
an infeasible child remains an explicit procedural failure.

The recursive tree is now exposed through `bisect exact --method
certified-recursive`. A committed path-8/four-district package includes the
tree, final RPLAN/RCTX, audit certificate, and hash manifest. Five hostile tree
fixtures test identity, per-split optimality,
schedule order, leaf completeness, and parent-derived leaf contents. Package
verification binds the final RPLAN assignment back to the verified leaves.

The proof-producing backend contract now separates solver discovery from three
pseudo-Boolean certification decisions: population lower bound, boundary lower
bound, and canonical predecessor. The bounded prototype emits deterministic
OPB requests and distinguishes an optimal UNSAT sequence from a suboptimal SAT
counterexample. A pinned RoundingSat proof has now been independently accepted
by VeriPB on the bounded population smoke instance. State-scale proof
generation and production integration remain open.

The production discovery boundary is now integrated. Rhode Island seed 1 plus
nine articulation-safe population moves produces connected populations 548,689
and 548,690, scaled deviation 1, and weighted cut 102,659,356.

All three Rhode Island compact decisions compile. RoundingSat proves the
population lower bound and VeriPB independently accepts the proof. The boundary
decision reached a time limit and produced no valid proof; canonical
certification remains blocked behind the boundary optimum.

A second 300-second boundary search without proof logging also timed out.
Therefore the current residual blocker is boundary decision search itself, not
only proof-log size or I/O.

The subsequent boundary-certification wave moves two zero-population boundary
blocks without changing population or connectivity, reducing weighted cut from
102,659,356 to 102,622,860. The population proof remains valid; the improved
boundary model has not yet been searched.

Three additional equal-population connectivity-safe swaps reduce the cut to
102,193,710 without changing the proved population objective.

Twenty-five deterministic 1-to-2 equal-population exchanges then reduce the
cut to 98,348,913. All earlier boundary timeouts apply to superseded
incumbents.

The Rhode Island certified-root frontier now includes a local hash-bound block
RCTX with 25,649 units and 66,097 land-boundary edges. The graph has two
components: 25,585 mainland-connected blocks and a 64-block, 1,410-person
Block Island component. Applying BISECT's existing nearest same-county
main-component rule adds 64 median-weight synthetic bridges and produces one
connected graph. At that frontier-report stage, scalable discovery,
connectivity encoding, and the proof toolchain remained blockers.

The proof toolchain and compact connectivity prototype have since advanced:
RoundingSat/VeriPB pass committed smoke proofs, including a substantive
parent/depth boundary proof. The compact compiler no longer invokes exhaustive
classification and can emit models above the 24-unit oracle limit. State-scale
model generation and discovery remain open.

## 6. Research Milestones

### E0 - Axioms and schema

- Freeze the exact feasible set and objective tuple.
- Define certificate and verifier schemas.
- Create positive and negative synthetic fixtures.

### E1 - Small exact States

- Prove exact benchmarks for States with one to four districts.
- Cross-check two solvers.
- Publish certificates and verifier results.

### E2 - Medium States

- Add connectivity separation, decomposition, and branch-and-price.
- Certify selected 5--14 district States.
- Compare exact and METIS benchmarks.

### E3 - Large States

- Certify or publish bounded optimality gaps for CA, TX, FL, and NY.
- Improve decomposition until gaps close.

### E4 - National readiness

- All States produce an optimal feasible-assignment certificate or an exact
  infeasibility certificate.
- Two independent verifier implementations accept every State result,
  including infeasibility.
- Non-author teams complete hostile replication.

## 7. Success Gates

The North Star is achieved only when:

1. the model and objective are enacted or frozen for the candidate standard;
2. every benchmark claim has a valid exact certificate;
3. at least two independent verifiers accept every optimality or infeasibility
   certificate;
4. no hidden solver tie-break affects the selected assignment;
5. all failures and optimality gaps are public; and
6. legal modifications remain separately justified.

## 8. What Mathematics Can And Cannot End

Mathematics can end debate over:

- whether the enacted procedure was followed;
- whether the benchmark is feasible;
- whether a better objective value exists; and
- which tied optimum is canonically selected.

Mathematics cannot end debate over:

- the axioms Congress should enact;
- the relative value of geography, communities, and representation;
- VRA or constitutional application;
- State-law criteria; or
- whether the final legal plan should depart from the benchmark.

That boundary is the basis for a credible Huntington--Hill analogy.

The current statute's heuristic readiness gate is an earlier policy milestone.
It does not satisfy this exact-readiness standard and must not be described as
the North Star's completion.
