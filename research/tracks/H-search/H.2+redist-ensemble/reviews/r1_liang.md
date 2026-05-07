# Review — H.2: redist-ensemble
**Reviewer**: Percy Liang (Reproducibility, Benchmarking, and Evaluation Standards in ML/CS)
**Round**: 1
**Score**: 3.5 / 5

---

## Summary

The paper proposes a high-performance Rust implementation of the ReCom redistricting Markov chain, motivated by the throughput bottleneck in GerryChain. The contribution is clear and the technical execution is competent. My concerns are with the Phase 2 benchmark commitments, which are underspecified as a reproducibility contract, and with the SHA-256 seed derivation scheme, which has a gap in the specification that could produce non-deterministic behavior across environments.

---

## Phase 2 Benchmark Commitments: Specificity

Section 5.4 commits to validating throughput estimates using criterion.rs. The list of benchmarks is:

1. Wilson UST: time to generate one UST as a function of $m$
2. Balance check: time for the full edge enumeration pass
3. Full step: end-to-end ReCom step time
4. Chain throughput: steps/sec for a 1,000-step chain vs. GerryChain

This list is a reasonable starting point, but as a reproducibility commitment it is underspecified in several ways.

**Missing: hardware specification.** The GerryChain baseline measurements in Table 1 (47 seconds for 1,000 NC steps) were run on "a standard development workstation." This phrase is not a reproducible hardware specification. For criterion.rs benchmarks to be comparable across environments, the paper needs to commit to a specific hardware configuration, or at minimum to reporting the hardware used. The 50,000 steps/sec estimate will look very different on a 2019 laptop vs. a 2024 server with 32 cores.

**Missing: GerryChain comparison protocol.** The Phase 2 commitment says the chain throughput benchmark will be "compared directly against GerryChain wall-clock time." But the comparison protocol is not specified. Same machine? Same starting plan? Same seed (insofar as GerryChain supports seeding)? GerryChain's Python overhead is not just algorithmic — it includes interpreter startup, graph-object construction, and NumPy overhead. A fair comparison should isolate the algorithmic contribution (Wilson's walk) from the implementation language overhead. The paper should specify whether the comparison controls for starting plan, warm-up steps, and measurement method (wall clock vs. process time).

**Missing: variance and confidence intervals.** criterion.rs produces confidence intervals over multiple runs. The Phase 2 commitment should state the minimum number of iterations (criterion.rs default is at least 100 samples for micro-benchmarks), the warm-up duration, and how the reported throughput figure will be derived (median? mean? lower bound of 95% CI?). Without this, the Phase 2 results will not be reproducible from the commitment alone.

**Missing: what constitutes validation.** The paper commits to "validating the 50,000 steps/sec target." But it does not specify what threshold of deviation would falsify the estimate. If the Phase 2 benchmark yields 40,000 steps/sec, does the estimate pass or fail? If it yields 8,000 steps/sec, what revision is triggered? A reproducibility commitment that cannot fail is not a commitment.

I recommend adding a subsection that frames the Phase 2 benchmark protocol as a falsifiable experiment: state the target (≥ 50,000 steps/sec for NC), the acceptable range, and what the paper will report if the target is not met.

---

## SHA-256 Seed Derivation: Specification Gap

The per-chain seed derivation (Section 3.4) is:

$$\text{seed}_i = \text{SHA256}(\text{"ENSEMBLE\_CHAIN\_"} \| i \| \text{"\_"} \| \text{base\_seed})$$

The property this construction is intended to provide — that the seed for chain $i$ does not depend on the total number of chains — is valuable and correctly designed. The concatenation of a chain-index-labeled prefix with a shared base seed produces independent seeds while preserving reproducibility.

However, the specification has a critical underspecification: how is $i$ encoded in the hash input? If $i$ is encoded as a decimal string (e.g., "3"), then chain 3 with base seed 42 and chain 13 with base seed 4 would produce the same hash input ("ENSEMBLE\_CHAIN\_3\_4" vs. "ENSEMBLE\_CHAIN\_13\_4" — these differ, good). But chain 3 with base seed 42 and chain 3 with base seed 4 would not collide ("ENSEMBLE\_CHAIN\_3\_42" vs. "ENSEMBLE\_CHAIN\_3\_4"). So string encoding avoids collisions in practice.

The deeper problem is with large chain indices. If $i$ can be, say, 1,000, and base\_seed is 0, the hash input is "ENSEMBLE\_CHAIN\_1000\_0". If $i = 100$ and base\_seed is 0, the input is "ENSEMBLE\_CHAIN\_100\_0". These do not collide. But note: if base\_seed is decimal-formatted, "ENSEMBLE\_CHAIN\_1\_00" (chain 1, seed "00") and "ENSEMBLE\_CHAIN\_10\_0" (chain 10, seed "0") produce the same string if the base\_seed is allowed to have leading zeros. The paper should specify that the base\_seed is formatted as a decimal integer without leading zeros (or use a separator that cannot appear in the integer representation, such as a null byte or a fixed-width big-endian encoding).

The safer specification is to use fixed-width encoding (e.g., 8-byte big-endian for $i$ and 8-byte big-endian for base\_seed), which eliminates all collision ambiguity. This is also what the Rust standard library would produce with `hash.update(&i.to_be_bytes())`. I recommend specifying the encoding explicitly.

**Additional concern**: The paper specifies that `SmallRng` is seeded from the SHA-256-derived seed (Section 4.3), but does not say how the 256-bit SHA-256 output is mapped to SmallRng's 128-bit state. The `rand` crate's SmallRng takes a 64-bit or 128-bit seed. If the first 128 bits of the SHA-256 hash are used, this should be stated. If a different truncation is used, it should be specified. Reproducibility across Rust compiler versions requires that the mapping from SHA-256 output to SmallRng state be pinned.

---

## On the GerryChain Baseline

The GerryChain measurements in Table 1 are labeled as "direct empirical measurements from the GerryChain Python package (version 0.3) on a standard development workstation." GerryChain 0.3 is the correct version to cite. However, GerryChain's performance can vary substantially depending on the Python version, NumPy version, and whether the adjacency graph uses the optimized sparse-matrix backend. The paper should report the Python version, NumPy version, and any relevant configuration options used to produce these measurements. Without this, the baseline cannot be replicated by another researcher trying to confirm the measurements.

Additionally, the paper does not specify the starting plan for the GerryChain baseline measurements. GerryChain's throughput is not constant across steps: the first few steps from a random start can be slower due to cache-cold graph traversal. Were the measurements taken for steps 1–1000 from a cold start, or from a warmed-up chain?

---

## On the Criterion.rs Granularity Breakdown

The four-level benchmark plan (Wilson UST, balance check, full step, chain throughput) is appropriate for understanding the overhead breakdown. The expected result — that the Wilson walk dominates over the balance check — should be verifiable from the per-component benchmarks. I recommend that the paper commit to reporting the overhead breakdown as a fraction of full-step time, not just as absolute numbers. This would allow readers to assess whether the $13\times$ overhead factor in Section 5.2 is borne primarily by subgraph construction, the Wilson walk, or serde output.

---

## Minor Points

- The abstract states 50,000 steps/sec and then immediately states "an estimated 2,300× speedup." Both figures should be dagger-marked in the abstract to match the notation used in the paper body, or a inline "(estimated)" should be added. Currently, the abstract reads these figures as if measured.
- Section 4.3 states that SmallRng "passes the BigCrush statistical test suite." To my knowledge, BigCrush is TestU01's test suite, and xoshiro128++ passes PractRand but has known (minor) failures on some BigCrush subtests depending on the specific generator variant. The paper should verify this claim or soften it to "passes PractRand."
- The serde output format (Section 4.5) is described as "newline-delimited JSON" but the field names are not specified. For reproducibility, the schema should be versioned. A schema version field in the output JSON would allow future changes without breaking downstream consumers.

---

## Recommendation

Minor revision. The Phase 2 benchmark commitments need to be specified as a falsifiable protocol with hardware, comparison method, and acceptance criterion. The SHA-256 seed encoding must be precisely specified to guarantee cross-environment reproducibility. These are fixable issues that do not undermine the paper's core contribution.
