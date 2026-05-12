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

## Step-By-Step Mechanics

1. Read the target district count `k`.
2. If `k <= 3`, create a direct `k`-way split.
3. If `k` is composite, split by its largest prime factor.
4. If `k` is prime and larger than three, split into `floor(k/2)` and
   `ceil(k/2)`.
5. Recurse on each child target count.
6. Record the resulting bisection/factor tree.

## Claim Boundary

ApportionRegions defines a deterministic tree topology. Claims about national
partisan outcomes, compactness frontier behavior, or legal sufficiency require
separate empirical evidence and uncertainty qualification.

## References In This Repo

- Structure value: `prime-factor`
- Legacy mode: `apportion-regions`
- Crate: `bisect-apportion`
- Concept guide: `docs/concepts/section-algorithms.md`
