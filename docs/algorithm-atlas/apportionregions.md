# ApportionRegions

![ApportionRegions visual](assets/apportionregions.svg)

## Mental Model

ApportionRegions chooses the bisection tree from the factorization of the seat
count. Composite counts split by their largest prime factor. Small counts up to
three can split directly. Prime counts fall back to a floor/ceil binary split,
after which composite subproblems may use non-binary splits again.

## How BISECT Uses It

BISECT uses ApportionRegions when the plan should be generated from a stable
factorization spine:

```text
seat count k -> factor tree -> regional split sequence -> district leaves
```

The useful property is reuse: related seat counts can share a top-level spine
when their factorization structure aligns.

## Picture 1: Prime Fallback Then Composite Children

![ApportionRegions prime fallback](assets/apportionregions-prime-fallback.svg)

When `k` is prime and larger than three, ApportionRegions cannot split by a
non-trivial factor. It falls back to a floor/ceil binary split. The children are
then handled normally: a child with `k=9` can split as `3 x 3`, while a child
with `k=8` can use binary factors.

## Picture 2: Reusable Regional Spine

![ApportionRegions reusable spine](assets/apportionregions-reuse-spine.svg)

The spine property matters because related seat counts can share an early
regional split. That does not prove the same final districts, but it gives the
pipeline a stable top-level geography to compare across reapportionment
scenarios.

## Step-By-Step Mechanics

1. Read the target district count `k`.
2. If `k <= 3`, create a direct `k`-way split.
3. If `k` is composite, split by its largest prime factor.
4. If `k` is prime and larger than three, split into `floor(k/2)` and
   `ceil(k/2)`.
5. Recurse on each child target count.
6. Record the resulting bisection/factor tree.

## What The Output Needs To Explain

The evidence should expose the factor tree, the split prescribed at each node,
the fallback rule for prime targets, and the final district leaves. For
cross-cycle comparisons, it should identify which top-level spine was reused.

## Claim Boundary

ApportionRegions defines a deterministic tree topology. Claims about national
partisan outcomes, compactness frontier behavior, or legal sufficiency require
separate empirical evidence and uncertainty qualification.

## Failure Modes

- A prime fallback is described as if it were a largest-prime-factor split.
- Direct `k <= 3` splits are mistaken for recursive binary halves.
- Reuse of a top-level spine is overstated as reuse of final district lines.

## References In This Repo

- Structure value: `prime-factor`
- Legacy mode: `apportion-regions`
- Crate: `bisect-apportion`
- Concept guide: `docs/concepts/section-algorithms.md`
- Pipeline tests: `crates/bisect-cli/tests/spec7_pipeline_l2.rs`
