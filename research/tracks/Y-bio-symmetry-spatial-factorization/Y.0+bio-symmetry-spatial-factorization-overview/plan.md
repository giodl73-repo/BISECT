# Y.0 Bio-Symmetry Spatial Factorization Overview

Goal: define Track Y as a research family for cohesion-weighted,
symmetry-aware spatial factorization of redistricting graphs.

## Core Claim

BISECT should continue to rely on transparent graph partitioning, but it can
build a richer spatial field before cutting:

```text
topology finds the mesh
geometry measures physical connection
population supplies flow/mass
district count supplies factorization pressure
symmetry supplies canonical comparison and fairness tests
```

The purpose is to make bisection less raw. Instead of treating every adjacency
edge as a similar cut candidate after ordinary weighting, Track Y asks whether
local mesh structure, population mass, repeated neighborhood roles, and weak
bridges can reveal where the map naturally wants to factor.

## Spatial Factorization

The final plan is a constrained factorization:

```text
State = D1 * D2 * ... * Dk
```

The product means disjoint connected spatial composition, not numeric
multiplication.

Each district factor must satisfy:

```text
connected(Di) = true
population(Di) ~= ideal
union(D1..Dk) = State
intersections(Di, Dj) = empty for i != j
mode constraints hold
```

The factorization tree records how larger factors split into smaller factors:

```text
Region(k) -> Region(a) * Region(b)
where a + b = k
```

## Biological Network Analogy

Biological spatial systems do not fill space uniformly. They reinforce useful
connections and let weaker connections decay.

Track Y uses these analogies as graph-design inspiration:

- slime mold: efficient, redundant transport under cost pressure;
- spider webs: coverage, redundancy, and material economy;
- roots and veins: flow-weighted branching and pruning;
- rivers: drainage, hierarchy, and low-resistance paths.

These analogies do not define legal fairness. They suggest measurable graph
signals that BISECT can audit.

## Symmetry And Groups

Strict groups appear where transformations preserve structure exactly:

- district-label permutations;
- bisection child swaps;
- graph automorphisms in synthetic fixtures;
- geometric rotations/reflections in regular fixtures.

Real tract geography usually has approximate rather than exact symmetry.
Track Y therefore separates:

- exact canonicalization for labels, trees, and fixtures;
- approximate local equivalence for tract roles and neighborhoods;
- fairness invariance tests for mode boundaries.

## Research Sequence

1. Define the cohesion field.
2. Prove cycle and bridge signals on synthetic fixtures.
3. Add population mass as flow without using forbidden policy fields.
4. Detect local tract-role equivalence and approximate orbits.
5. Run mode-specific invariance tests.
6. Compare against existing B/T/U construction and search baselines.

## Non-Claim Boundaries

Track Y does not claim:

- census tracts form exact groups;
- biological systems prove civic fairness;
- dense areas should be protected from all cuts;
- population mass can replace equal-population constraints;
- algorithmic output is legal certification.

Track Y may claim only fixture-backed changes in graph weighting, canonical
comparison, and invariance validation.
