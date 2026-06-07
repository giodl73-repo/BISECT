# Y.2 Local Symmetry And Tract Orbits

Goal: detect exact and approximate local symmetries in tract adjacency graphs so
BISECT can canonicalize equivalent choices, audit tie-breaking, and identify
near-interchangeable local tract roles.

## Strict Symmetry

An exact graph automorphism is a vertex permutation that preserves adjacency:

```text
u adjacent to v => pi(u) adjacent to pi(v)
```

Exact automorphisms are useful for synthetic fixtures, grids, and small test
graphs. They are expected to be rare in real California tract geometry because
tract shape, population, and boundaries are irregular.

## Approximate Local Symmetry

For real tracts, the practical target is approximate role equivalence:

```text
tracts are locally similar if they have similar graph role, population mass,
boundary profile, cycle participation, and module position
```

Candidate features:

- degree;
- weighted degree by shared boundary length;
- local clustering coefficient;
- cycle support around incident edges;
- bridge-likeness of incident edges;
- local population mass and density;
- module/community membership;
- distance to separators such as water, mountain, or sparse connectors when
  available and authorized.
- physical-geography role:
  - same river corridor or watershed;
  - same valley floor, basin, or elevation band;
  - same ridge shelf, pass, coastal shelf, delta, lake edge, or island chain.

## Orbit-Like Classes

For an exact group action, an orbit is:

```text
orbit(v) = { g * v for g in G }
```

For BISECT, approximate orbits can be represented as role classes:

```text
role_class(v) = tracts with similar local feature signatures
```

Uses:

- tie-breaking among near-equivalent boundary choices;
- search-state compression;
- fixture design for known symmetries;
- fairness auditing when arbitrary input order selects between similar choices.
- identifying naturally repeating corridor or basin roles that adjacency alone
  cannot explain.

Physical-geography role classes should stay approximate. Two tracts at the same
elevation band or along the same river are not interchangeable in the exact
group-theoretic sense; they are candidates for similar local behavior under a
declared geography-aware weight profile.

## Canonicalization

The first implementation should avoid expensive general graph isomorphism over
large state graphs. Start with stable local signatures:

```text
signature(v) =
  hash(
    degree_bucket,
    weighted_degree_bucket,
    cycle_support_bucket,
    density_bucket,
    physical_geography_bucket,
    module_id_or_bucket
  )
```

District and split canonicalization should remain exact:

```text
canonical_plan = sort districts by stable district signature
canonical_split = sort child unit-set hashes
```

Approximate tract orbits should inform search and diagnostics, not silently
merge non-identical plans.

## Validation Fixtures

1. Square grid with exact rotations/reflections.
2. Hex grid with exact local neighborhoods.
3. Grid with one bridge and two symmetric lobes.
4. Synthetic urban core plus sparse rural connector.
5. Synthetic river/valley corridor with a declared barrier edge.
6. California tract subset with role classes but no exact-symmetry claim.

## Acceptance Criteria

- Exact fixture automorphisms are recognized where intentionally present.
- Approximate role classes are inspectable and do not claim exact equivalence.
- Geography-conditioned role classes record the source layer used.
- Canonical plan hashes remain exact and stable.
- Tie-break diagnostics record when a choice occurred among near-equivalent
  candidates.
