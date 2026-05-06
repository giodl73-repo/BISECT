# Review — H.2: redist-ensemble
**Reviewer**: Percy Liang (Reproducibility, Benchmarking, and Evaluation Standards in ML/CS)
**Round**: 2
**Score**: 4 / 5

---

## Response to Round 1 Concerns

### R05 (MAJOR): SHA-256 seed encoding — Fully addressed

The revised Section 3.4 now specifies: "Integer $i$ is encoded as a 64-bit little-endian unsigned integer (\texttt{i.to\_le\_bytes()} in Rust). The base seed is encoded as a 64-bit little-endian unsigned integer." The paper further specifies that "the resulting 256-bit SHA-256 hash is truncated to the least-significant 64 bits (bytes 0--7 of the output in little-endian order) as the \texttt{u64} output seed, which is then used to initialize the \texttt{SmallRng} state via \texttt{SeedableRng::seed\_from\_u64}."

This is exactly the level of specification required for cross-environment reproducibility. The fixed-width encoding eliminates all integer-formatting collision ambiguity. The truncation rule and seeding method are now pinned. This is the right fix and it is well-executed.

One minor observation: the paper specifies little-endian encoding for both $i$ and the base seed, but Rust's \texttt{to\_le\_bytes()} produces the correct result regardless of the host architecture since it is a library function, not a direct memory cast. This is therefore portable. The paper correctly cites the Rust standard library function, which pins the behavior across compiler versions.

### R06 (MAJOR): Falsifiable Phase 2 benchmark commitment — Not addressed

Section 5.4 (Phase 2 benchmarks) remains unchanged from Round 1. The revision plan called for adding hardware specification, GerryChain comparison protocol, acceptance criterion (minimum threshold below which the estimate is revised), and variance/confidence interval reporting commitment. None of these additions appear in the revised paper.

This is the most significant remaining gap. A Phase 2 commitment that does not specify what constitutes failure is not a reproducibility commitment. The acceptance criterion — specifically, what the paper will do if criterion.rs yields 20,000 steps/sec rather than 50,000 — must be stated. I recommend the following addition to Section 5.4:

> "The Phase 2 benchmark will be reported as confirmed if criterion.rs measurement for NC exceeds 30,000 steps/sec (i.e., within 40\% of the 50,000 target). If the measurement falls below 30,000 steps/sec, the speedup estimate in the abstract, Table 1, and Section 5.2 will be revised downward. The benchmark will be reported with the 95\% confidence interval from criterion.rs. Hardware: CPU model, clock speed, L2/L3 cache, and OS will be specified in the Phase 2 companion note."

Without this addition, the reproducibility contract is incomplete.

### R12 (MODERATE): BigCrush vs. PractRand claim — Not addressed

Section 4.3 still states SmallRng "passes the BigCrush statistical test suite." This was flagged in Round 1 as potentially incorrect: xoshiro128++ passes PractRand up to large byte counts, but its BigCrush status is not established in the `rand` crate documentation and depends on the specific variant. This should be corrected to "passes the PractRand statistical test suite." This is a one-line fix.

### R16 (MINOR): Abstract dagger notation — Not addressed

The abstract still states "roughly 50,000 steps per second — an estimated 2,300× speedup" without dagger notation. Round 1 feedback requested that both figures be dagger-marked or labeled "(estimated)" in the abstract. This remains uncorrected. Since the body uses dagger notation consistently and the abstract uses the word "estimated," this is tolerable but inconsistent. Adding \est{} markers or the parenthetical "(estimated)" to both abstract figures would complete the consistency.

---

## What the Revision Gets Right

The SHA-256 specification fix is the most reproducibility-critical change in the paper, and it is done correctly. The per-chain seed derivation is now fully specified with encoding, truncation, and seeding method. A researcher implementing a compatible system from the paper description would produce byte-identical seeds. This is genuine reproducibility progress.

The GerryChain framing corrections (Table 1 footnote, Section 7.1) are not in my primary domain, but from a reproducibility standpoint the change from "GerryChain fails" to "GerryChain without pair reselection fails" is the correct move — it makes the comparison reproducible by specifying which GerryChain configuration was being compared.

---

## Remaining Priority Issues

In order of priority:

1. **R06** (Phase 2 falsifiability): Still missing acceptance criterion, hardware spec, and variance reporting commitment. This is the paper's main reproducibility gap.
2. **R12** (BigCrush vs. PractRand): One-line fix, should be corrected.
3. **R16** (Abstract dagger notation): Cosmetic but inconsistent with body notation.
4. **R18** (Serde schema version): Still unversioned. Adding `"ensemble_output_version": "1.0"` to the serde output schema takes one line and should be done.

---

## Recommendation

**Minor revision** (upgraded from Minor Revision — the R05 fix earns the score improvement). The SHA-256 specification is now complete. The Phase 2 commitment (R06) remains the principal outstanding gap: it must specify what constitutes a failed validation and what revision would follow. BigCrush correction (R12) and serde versioning (R18) are one-line fixes that should be done before final submission. The abstract dagger notation (R16) should be added for consistency.
