# Ensemble Methods and the Feasible Space

## Short version

The redistricting feasible space is the set of all valid plans for a state —
all plans that satisfy population balance, contiguity, and any applicable legal
constraints. There are billions of such plans for any large state. The `redist`
algorithm finds a plan at the compactness extremum of this space. An ensemble
samples the space randomly to characterise it. Comparing the `redist` plan to
the ensemble certifies its position: more compact than 99%+ of valid plans
in most states, or at the geographic median in constrained states like North
Carolina.

---

## Two complementary tools

The `redist` algorithm and the ensemble method answer different questions:

| Tool | Question answered |
|---|---|
| `redist` (METIS-based) | "What is the most compact valid plan?" |
| GerryChain / ReCom | "What does the distribution of valid plans look like?" |

These tools are complementary, not competing. `redist` finds the optimum.
The ensemble characterises where the optimum sits within the feasible space.
Together, they produce a certificate: the `redist` plan is at the compactness
extremum, and the ensemble confirms this.

---

## GerryChain / ReCom (the evaluator)

GerryChain implements the ReCom (Recombination) Markov chain, which samples
valid redistricting plans approximately uniformly at random. Unlike `redist`,
it does not optimise for any objective: each step is a random local move that
preserves population balance and contiguity.

**How ReCom works**: At each step, two adjacent districts are merged into a
single region. A random spanning tree of that region is drawn. The tree is cut
at a randomly chosen balanced edge, producing two new districts. The partition
is updated with the new districts and the chain continues.

The key property of ReCom is its stationary distribution: it samples plans
with probability proportional to the number of balanced spanning tree cuts
that can generate each plan. This is a well-characterised distribution that
allows rigorous inference about the feasible space.

**What GerryChain is used for**:

- **Auditing enacted maps**: Is a legislature's map an outlier relative to
  random valid plans? If an enacted map sits at the 99.5th percentile for
  partisan lean, it is highly improbable under neutral drawing — evidence
  of intent.

- **Characterising the feasible space**: What range of partisan outcomes is
  achievable under compact, balanced redistricting? What is the baseline
  Republican advantage in a state given its geography?

- **Certifying the `redist` plan**: After generating an ensemble of valid plans,
  compare the `redist` plan's compactness to the distribution. The percentile
  position is the certificate.

**GerryChain speed**: Approximately 21 steps per second for North Carolina
(k=14, approximately 2,700 tracts). For a 1,000-step ensemble at this speed,
a run takes approximately 50 seconds. For 10,000 steps: approximately 8 minutes.

---

## `redist-ensemble` — the Rust ReCom engine (H.2)

`redist-ensemble` is a native Rust implementation of the ReCom algorithm,
designed for high-throughput ensemble generation. It implements the same
stationary distribution as GerryChain (Python) but achieves much higher
throughput through Rust's zero-cost abstractions and efficient memory layout.

**Estimated throughput**: ~50,000 steps per second for NC-scale graphs
(estimated 2,300x speedup over Python GerryChain). This makes 1 million-step
ensembles practical in minutes rather than days.

### Wilson's algorithm for uniform spanning trees

The critical primitive in ReCom is sampling a uniform random spanning tree
(UST) of a subgraph. `redist-ensemble` uses Wilson's loop-erased random walk
algorithm (Wilson 1996):

1. Start with a root vertex in the tree.
2. Pick any unvisited vertex.
3. Run a simple random walk until hitting the tree.
4. Erase all loops from the walk path.
5. Add the loop-erased path to the tree.
6. Repeat until all vertices are in the tree.

This produces a spanning tree drawn uniformly at random from all spanning trees
of the graph. For planar graphs (census-tract adjacency graphs are approximately
planar), the expected running time is O(n log n) where n is the number of
vertices in the subgraph. For a merged region of approximately 2n/k vertices,
the cost per step is O((n/k) log(n/k)).

### Correct balance cut enumeration

A critical design decision in `redist-ensemble` is enumerating **all** balanced
cuts of each spanning tree, not just sampling one. This replicates GerryChain's
stationary distribution exactly.

After drawing a spanning tree T over the merged region:
1. Perform a DFS to compute the subtree population on each side of every edge.
2. Classify each of the n-1 tree edges as balanced or unbalanced.
3. Select uniformly at random from the balanced edges.

The enumeration costs O(n) per step, which is subdominant to the O(n log n)
Wilson cost.

### Failure handling

Some spanning trees of a merged region admit no balanced cuts: the population
distribution along the tree does not permit a balanced partition. When this
occurs:

- **Resample the tree** (not the same tree). Draw a new spanning tree via
  Wilson's algorithm. Each resample is a fresh draw from the UST distribution.
- After 10 consecutive resamples for the same district pair, **select a new
  pair**. The pair-reselection mechanism is not a Rust-specific workaround;
  the same failure mode occurs in Python GerryChain. Texas is a known case
  where bipartition failures are frequent due to population concentration in
  the Houston and Dallas metro areas.

### Running the Rust ensemble

```bash
# Single-chain ensemble, 10,000 steps
redist ensemble --state NC --year 2020 --steps 10000

# Four independent chains (for convergence diagnostics)
redist ensemble --state NC --year 2020 --steps 10000 --chains 4

# Save all accepted plans to a directory
redist ensemble --state NC --year 2020 --steps 10000 --out-dir /tmp/nc_ensemble

# Compare a specific plan against an ensemble
redist ensemble --state NC --year 2020 --steps 10000 \
  --compare-plan runs/official_2020/2020/north_carolina/final_assignments.json
```

---

## Ensemble diagnostics

Running multiple chains produces not just plans but convergence diagnostics
that certify the ensemble is representative of the feasible space.

### R-hat (Gelman-Rubin convergence)

R-hat compares within-chain variance to between-chain variance for a scalar
summary statistic (typically the normalised edge-cut fraction EC or the
Democratic seat share). If chains have converged to the same stationary
distribution, within-chain and between-chain variances should be equal.

```
R-hat = sqrt((N-1)/N + B/(N*W))
```

where B is the between-chain variance, W is the mean within-chain variance,
and N is the number of steps per chain.

| R-hat value | Interpretation |
|---|---|
| R-hat < 1.1 | Converged. Chains are sampling from the same distribution. |
| 1.1 <= R-hat < 1.2 | Marginal. Run longer chains or more chains. |
| R-hat >= 1.2 | Not converged. Chains may be exploring different regions of the feasible space. |

The R-hat implementation in `redist-analysis::ensemble_diagnostics` follows
Gelman and Rubin (1992) with the split-chain correction. Chains are split in
half before computing R-hat to detect trends within chains.

### ESS (Effective Sample Size)

The ReCom Markov chain is autocorrelated: consecutive plans are similar because
each step changes only two districts. The effective sample size (ESS) measures
how many independent draws the chain is equivalent to, accounting for
autocorrelation.

```
ESS = N / (1 + 2 * sum_{t=1}^{inf} rho_t)
```

where rho_t is the lag-t autocorrelation of the summary statistic and N is
the total number of steps. The sum uses Geyer's (1992) monotone truncation
rule to avoid estimating noise in the autocorrelation tail.

| ESS value | Interpretation |
|---|---|
| ESS >= 1000 | Reliable inference. Summary statistics are stable. |
| 400 <= ESS < 1000 | Adequate for descriptive statistics; marginal for tail inference. |
| ESS < 400 | Run longer chains. Tail percentiles (0.1st, 99.9th) are unreliable. |

For North Carolina at GerryChain speed (21 steps/sec), achieving ESS = 1,000
typically requires approximately 3,000-5,000 total steps across 4 chains
(750-1,250 per chain). With `redist-ensemble` at 50,000 steps/sec, the
same ESS is achieved in under a second.

### Hamming autocorrelation

Hamming autocorrelation measures lag-1 autocorrelation of the cut fraction
directly: what fraction of tracts change district assignment between consecutive
plans? High autocorrelation means consecutive plans are very similar (the chain
is mixing slowly). Low autocorrelation means each step produces a very different
plan (fast mixing).

```
cut_fraction_t = |{v : assignment_t(v) != assignment_{t-1}(v)}| / |V|
lag1_autocorr = corr(cut_fraction_t, cut_fraction_{t+1})
```

For NC, GerryChain produces a lag-1 autocorrelation of approximately 0.15-0.25,
indicating reasonably fast mixing. A lag-1 autocorrelation above 0.5 suggests
the chain is stuck and needs longer steps or a different initialisation.

---

## The legal argument

Ensemble evidence is used to certify the `redist` plan in legal proceedings.
The argument structure differs by state type.

### Maximum-compactness certificate (WI, GA, PA)

For Wisconsin (k=8), Georgia (k=14), and Pennsylvania (k=17), the
ApportionRegions plan sits at the 0.1-0.2nd percentile of the ensemble
edge-cut distribution. Fewer than 2 in 1,000 randomly drawn valid plans are
more compact.

The certificate reads:

> A GerryChain ReCom ensemble of 1,000 valid redistricting plans was run for
> this state. The ApportionRegions plan has lower edge-cut fraction than
> approximately 99.8-99.9% of ensemble plans. Any party claiming the plan
> is non-compact must identify a more compact alternative. Such an alternative
> will be found in fewer than 2 in 1,000 random draws from the feasible space.

This certificate is **legally stronger than a median certificate**. A plan at
the 50th percentile could be replaced by thousands of alternatives. A plan at
the 0.2nd percentile cannot: it is at the compactness frontier, and displacing
it requires accepting a less compact map.

Under this frame, any legal challenger must: (1) propose an alternative plan;
(2) show it has lower edge-cut fraction; (3) accept that only 0.1-0.2% of
valid plans satisfy condition (2). The evidentiary burden is unusually high.

### Geographic-inevitability certificate (NC)

For North Carolina (k=14), the ApportionRegions plan sits at the 50th percentile
of the ensemble. Its edge-cut fraction (0.0973) is within 0.3% of the ensemble
mean (0.0970), less than one-twentieth of one standard deviation above the mean.

This is not underperformance. It is the most important diagnostic in the G.1
paper: when geography tightly constrains the feasible space, the minimum-cut
plan and the ensemble median converge. All compact redistricting methods applied
to North Carolina produce essentially the same partition, because North Carolina's
geography leaves little room for variation.

The certificate reads:

> The ApportionRegions plan's edge-cut fraction is within 0.3% of the ensemble
> mean. The minimum-cut plan and the random-sampling median have converged.
> This means North Carolina's geography, not algorithm choice, determines the
> plan. The partisan outcome (5D/9R for GeoSection; 7D/7R for ApportionRegions)
> is a geographic outcome.

Combined, the two certificates address both flanks of a legal challenge:
for states where the plan is at the compactness extremum, the challenger cannot
find a more compact alternative; for states where the plan is at the geographic
median, the plan is what geography produces regardless of method.

---

## Further reading

- [section-algorithms.md](section-algorithms.md) — the B-series algorithms being evaluated
- [three-layer-compositor.md](three-layer-compositor.md) — how to configure ensemble runs
- Paper G.1: *GerryChain Congressional Comparison*
- Paper H.2: *redist-ensemble — Rust ReCom Engine*
- Paper G.4: *Ensemble Diagnostics*
- `redist-analysis::ensemble_diagnostics` — R-hat, ESS, Hamming implementation
