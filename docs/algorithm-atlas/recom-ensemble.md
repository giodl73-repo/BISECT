# ReCom Ensemble

![ReCom ensemble visual](assets/recom-ensemble.svg)

## Mental Model

ReCom is a Markov chain over valid redistricting plans. Each step chooses two
adjacent districts, merges them into one connected region, samples a random
spanning tree of that region, and cuts a balanced edge to create two replacement
districts.

It does not optimize for the best plan. It samples plan space so BISECT can ask
where a constructed or enacted plan sits relative to a neutral distribution.

## How BISECT Uses It

BISECT uses ReCom-style ensembles as an evaluation and comparison layer:

```text
valid starting plan -> many valid sampled plans -> distributional comparison
```

This complements deterministic construction. A BISECT plan may be produced by a
constructor or solver, then compared against an ensemble to understand whether
its compactness, partisan outcome, or other metric is typical or outlying.

## Picture 0: Proposal Pair To Chain Diagnostics

The opening figure shows one ReCom step as a visible sampling decision. The
chain first chooses adjacent districts, then merges them and samples a spanning
tree over the merged region. Tree edges become candidate cuts, but only balanced
cuts can replace the district pair. A proposal with no balanced cut is a
different event from an accepted move, and both need to appear in diagnostics.

The bottom panels connect the local step to the ensemble claim. Acceptance
rates, R-hat, ESS, and seed transcripts are not decoration; they are the
evidence that lets BISECT compare a plan against the sampled distribution
without pretending the finite ensemble proves convergence or optimality.

## Picture 1: One ReCom Step

![One ReCom step](assets/recom-step.svg)

The key move is local but plan-scale: merge two adjacent districts, draw a
uniform spanning tree of the merged region, enumerate balanced tree cuts, and
select one balanced edge. If no balanced cut exists, the implementation can
resample or try another pair according to the configured chain variant.

## Picture 2: Ensemble Diagnostics

![ReCom diagnostics](assets/recom-diagnostics.svg)

One ReCom step is easy to draw, but the algorithm becomes useful through many
steps and often multiple chains. The diagnostic view records whether chains are
moving, whether cut fractions are stable, and whether summary metrics have
enough effective sample size to support the comparison being made.

## Step-By-Step Mechanics

1. Start from a valid district assignment.
2. Choose adjacent districts.
3. Merge their units into one connected subgraph.
4. Sample a random spanning tree, using Wilson-style machinery in the native
   ensemble path.
5. Enumerate balanced cuts of the tree.
6. Accept a replacement pair when the cut preserves the declared tolerance.
7. Record step diagnostics such as cut fraction, population deviation, and
   acceptance.

## Reading The Output

The chain transcript should let a reviewer distinguish "the step was rejected,"
"the pair had no balanced cut," "the chain moved but the summary did not mix,"
and "the ensemble comparison was run with too few accepted samples." Those are
different stories, and they should not collapse into a single sample count.

## Worked Diagnostics

| Chain | Proposed steps | Accepted steps | Acceptance | Warning |
|---:|---:|---:|---:|---|
| 0 | 1,000 | 611 | 0.611 |  |
| 1 | 1,000 | 94 | 0.094 | low movement |
| 2 | 1,000 | 587 | 0.587 |  |

The ensemble is not automatically bad because one chain moves slowly, but the
report has to make that visible. A comparison that uses only the pooled final
sample count can hide exactly the behavior a reviewer needs to inspect.

## Diagnostic Claims

| Evidence | Supports | Does not prove |
|---|---|---|
| acceptance rate | chain is moving under this proposal | correct stationary distribution |
| ESS | summary has enough effective draws for this metric | all summaries are reliable |
| R-hat | chains agree on this scalar summary | legal validity or optimality |
| seed transcript | fixed-seed reproducibility | seed-independent conclusion |

## What The Output Needs To Explain

The ensemble output needs chain seeds, per-chain step records, acceptance
behavior, cut-fraction summaries, R-hat/ESS diagnostics when available, and
enough seed derivation detail to make fixed-seed runs reproducible.

Example output fields:

```json
{
  "sampler": "recom",
  "chains": 3,
  "steps_per_chain": 1000,
  "acceptance": [0.611, 0.094, 0.587],
  "diagnostics": { "r_hat_edge_cut": 1.03, "ess_edge_cut": 420.0 }
}
```

## Claim Boundary

ReCom characterizes a sampling distribution. It does not prove optimality, and
finite ensembles do not prove full plan-space convergence. Diagnostics such as
R-hat and ESS are evidence about the sampled summaries, not universal guarantees
about every legal or political claim.

## References In This Repo

- Crate: `bisect-ensemble`
- Concept guide: `docs/concepts/ensemble-methods.md`
- Core files: `crates/bisect-ensemble/src/recom.rs`, `crates/bisect-ensemble/src/chain.rs`
- Tests: `crates/bisect-ensemble/tests/ensemble_l1.rs`, `crates/bisect-ensemble/tests/ensemble_l2.rs`
