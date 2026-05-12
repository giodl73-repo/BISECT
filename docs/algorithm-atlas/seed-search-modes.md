# Seed Search Modes

![Seed search modes visual](assets/seed-search-modes.svg)

## Mental Model

The search layer controls how BISECT explores seed-dependent local optima. The
same structure and weights can produce different candidate plans when METIS or
local ReCom is seeded differently. Search mode decides which candidates are
considered and which result is returned.

## How BISECT Uses It

BISECT uses seed search to make run selection explicit:

```text
content-derived seed stream -> candidate plans -> selected result
```

The major modes are `single`, `multi`, `convergence`, `percentile`, and
`bisection-ensemble`.

## Picture 1: Convergence Sweep Timeline

![Seed convergence timeline](assets/seed-search-convergence.svg)

Convergence mode keeps walking the seed stream after the current best plan is
found. It stops only after the configured number of consecutive seeds fail to
improve the objective. The stopping threshold is part of the evidence, not an
implementation detail.

## Picture 2: Percentile Selection

![Seed percentile selection](assets/seed-search-percentile.svg)

Percentile mode answers a different question from minimum search. It sorts the
candidate plans by edge cut and returns a rank. The selected plan may be
intentionally non-minimal.

## Step-By-Step Mechanics

1. `single`: run one publicly derived seed.
2. `multi`: run a fixed seed budget and return the minimum cut.
3. `convergence`: walk seeds until a threshold number of consecutive seeds
   produce no improvement.
4. `percentile`: sort candidates and return a requested percentile rather than
   the minimum.
5. `bisection-ensemble`: replace a binary METIS call with local ReCom sampling
   at that bisection node.

## What The Output Needs To Explain

The output should report the seed derivation rule, seed count or stopping
threshold, selected rank, objective values, and whether the result is a minimum,
percentile, convergence-selected, or local-ensemble bisection.

## Claim Boundary

Search mode affects evidence. A single deterministic plan, a minimum over many
seeds, a percentile plan, and a local ensemble bisection answer different
questions. They should not be cited as the same claim.

## Failure Modes

- A percentile result is described as the most compact plan.
- A convergence threshold is omitted, making the stopping rule unreplayable.
- Bisection-ensemble local samples are confused with a full-plan ReCom ensemble.

## References In This Repo

- Concept guide: `docs/concepts/three-layer-compositor.md`
- Taxonomy: `docs/concepts/section-algorithms.md`
- CLI implementation: `crates/bisect-cli/src/runner.rs`
- Seed-search parser tests: `crates/bisect-cli/src/runner.rs`
