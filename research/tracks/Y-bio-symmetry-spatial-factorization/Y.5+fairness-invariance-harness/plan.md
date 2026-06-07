# Y.5 Fairness Invariance Harness For Redistricting Modes

Goal: make BISECT fairness boundaries executable by testing whether outputs are
invariant or equivariant under transformations that should not matter.

## Core Properties

```text
invariant:   score(g * plan) = score(plan)
equivariant: build(g * input) = g * build(input)
```

Invariance is appropriate for scores and metrics. Equivariance is appropriate
for plan construction.

## Mode Declarations

Each mode should declare:

- allowed input fields;
- forbidden input fields;
- transformations that must not change canonical output;
- transformations that may change output;
- evidence rows required when a tie-break or seed matters.

## Geographic Mode

Allowed:

- adjacency graph;
- geometry-derived edge attributes;
- population;
- district count and chamber configuration.

Forbidden:

- party fields;
- race and ethnicity fields;
- candidate/incumbent metadata;
- input ordering as a semantic signal.

Required checks:

```text
shuffle rows -> same canonical plan
permute unit IDs -> equivalent canonical plan
renumber districts -> same metrics
scramble party fields -> same canonical plan
scramble demographic fields -> same canonical plan
```

## VRA Mode

Allowed:

- geographic-mode inputs;
- authorized demographic fields;
- declared VRA alignment logic.

Forbidden unless separately authorized:

- party fields;
- candidate/incumbent metadata;
- arbitrary row or unit ordering.

Required checks:

```text
scramble party fields -> same canonical plan
permute unit IDs -> equivalent canonical plan
change authorized demographic field -> output may change and must be logged
```

## County-Sticky Mode

Allowed:

- geographic-mode inputs;
- county or subdivision relation fields.

Required checks:

```text
change county relation -> output may change
scramble party fields -> same canonical plan
scramble demographic fields -> same canonical plan unless mode authorizes them
```

## Tie-Breaking

Tie-breaking should be declared and tested:

```text
tie_break(g * candidates) = g * tie_break(candidates)
```

Preferred tie-breakers:

- canonical cut hash;
- stable sorted unit-set hash;
- minimum canonical split signature;
- recorded seeded randomness;
- enumerated tie set when feasible.

## Harness Shape

Start with:

```text
canonical_plan(assignments) -> CanonicalPlan
canonical_tree(tree) -> CanonicalTree
transform_input(input, transformation) -> Input
assert_mode_equivariant(mode, fixture, transformation)
assert_metric_invariant(metric, plan, transformation)
```

## Compositor Placement

The invariance harness should not be a normal construction algorithm. It should
wrap compositor runs as validation evidence:

```text
selected structure
selected weights
selected search
-> build baseline plan
-> transform input
-> build transformed plan
-> canonicalize both
-> compare expected invariance/equivariance
```

Candidate command shape:

```text
bisect validate-invariance <label> --mode geographic
```

or later:

```text
bisect state --state CA --districts 52 \
  --structure prime-factor \
  --weights-override cohesion \
  --invariance-check geographic
```

The first shape is cleaner because it keeps validation out of ordinary plan
construction and avoids making every run pay for transformed rebuilds.

## Acceptance Criteria

- Each mode has explicit allowed and forbidden sensitivities.
- Transformations produce machine-readable pass/fail rows.
- Canonical output comparison removes district-label and child-order artifacts.
- Failures report whether the cause is input ordering, tie-breaking, forbidden
  field access, or intentional mode sensitivity.
