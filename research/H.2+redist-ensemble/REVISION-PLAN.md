# Revision Plan — H.2: redist-ensemble
**Round**: 2 (complete)
**Date revised**: 2026-05-06
**Reviews received**: Karypis, Rodden, Duchin, Stephanopoulos, Liang (Round 1)
**Round 2 status**: All Critical (R01–R07) and Moderate (R08–R13) fixes applied.
All Minor (R14–R18) fixes applied. Phase 2 dependencies noted as deferred.
PDF compiled clean: 25 pages, 0 LaTeX errors.

---

## Issue Triage

Issues are classified by severity and deduplicated across reviewers.

| ID | Reviewer(s) | Severity | Issue | Round 2 Status |
|----|------------|----------|-------|----------------|
| R01 | Karypis, Duchin | **CRITICAL** | Texas/CA claim misrepresents what pair reselection achieves vs. GerryChain | FIXED — §4.5, §5.3, §7.1 reframed; TX/CA as combinatorial, not language-specific |
| R02 | Karypis, Duchin | **CRITICAL** | Pair reselection modifies the Markov kernel — stationarity preservation not established | FIXED — stationarity conjecture paragraph added §3.3; deferred to Phase 2 |
| R03 | Karypis | **MAJOR** | Abstract conflates Wilson's cover-time bound with Aldous's planar specialization | FIXED — abstract and §1.2 explicitly attribute planar bound to Aldous |
| R04 | Karypis | **MAJOR** | Planarity of census-tract subgraphs not verified — $O(m \log m)$ bound depends on it | FIXED — planarity paragraph added §3.2 (TIGER non-crossing polygons; tri-point caveat) |
| R05 | Liang | **MAJOR** | SHA-256 seed encoding underspecified — integer-to-string collision risk, SmallRng truncation not stated | FIXED — §3.4 gives precise 32-byte encoding; §4.3 specifies SmallRng seeding from first 16 bytes |
| R06 | Liang | **MAJOR** | Phase 2 benchmark commitment is not falsifiable (no hardware spec, no acceptance threshold) | FIXED — §5.4 specifies hardware, GC version, acceptance criterion (30K steps/sec), 95% CI |
| R07 | Rodden | **MAJOR** | R-hat convergence conflates marginal-statistic convergence with plan-space convergence | FIXED — paragraph added §6.1; §1.1 revised to "marginal distributions of summary statistics" |
| R08 | Rodden | **MODERATE** | Hamming proxy ($\phi$) claim of "exactness" is wrong — two plans can share $\phi$ and differ | FIXED — §6.3 removes "exact"; replaced with "computationally efficient running diagnostic" |
| R09 | Stephanopoulos | **MODERATE** | Legal framing conflates AEA replication improvement with court-admissibility improvement | FIXED — §7.2 distinguishes AEA replication (genuine legal-adjacent) from throughput improvement |
| R10 | Karypis | **MODERATE** | $13\times$ overhead factor is ad hoc; sensitivity range never given | FIXED — §5.2 adds overhead sensitivity table (6.5×, 13×, 26× scenarios) |
| R11 | Rodden | **MODERATE** | GerryChain baseline hardware/version not fully specified for replication | FIXED — §5.1 adds CPU model, GC version, Python version, NumPy, starting plan, warm-up policy |
| R12 | Liang | **MODERATE** | SmallRng BigCrush claim may be incorrect (should be PractRand) | FIXED — §4.3 replaces BigCrush with PractRand; BigCrush status flagged as unverified |
| R13 | Duchin | **MODERATE** | CA "highest per-step cost" claim contradicts $O((n/k)\log(n/k))$ formula | FIXED — §5.3 corrects to PA having largest merged region; CA corrected; deferred to Phase 2 |
| R14 | Duchin | **MINOR** | Hamming autocorrelation definition uses $d_H(\sigma_t, \sigma_0)$ — nonstandard | FIXED — §6.3 redefines Ham(k) as plan-to-plan $d_H(\sigma_t, \sigma_{t+k})$ correlation |
| R15 | Stephanopoulos | **MINOR** | State-court case law citations missing (LWV, Harper, Harkenrider) | FIXED — §1.1 adds LWV v. PA (2018), Harper v. Hall (2022), Harkenrider v. Hochul (2022) |
| R16 | Liang | **MINOR** | Abstract missing dagger notation on throughput estimates | FIXED — abstract adds $\dagger$ to 50,000 steps/sec and 2,300× speedup |
| R17 | Rodden | **MINOR** | ESS bulk formula defines $\hat{\rho}_k^+$ without inline definition | FIXED — §6.2 adds inline definition referencing rank-normalization in §6.1 |
| R18 | Liang | **MINOR** | Serde output schema unversioned | FIXED — §4.4 adds ensemble\_output\_version: "1.0" as versioned constant |

---

## Required Changes (Critical and Major)

### R01 + R02: Texas/CA Claim and Stationarity of Pair Reselection

**The problem**: Table 1 marks GerryChain "failure" for TX/CA and implies redist-ensemble "handles" these cases. The conclusion states pair reselection enables "TX and CA runs that Python GerryChain cannot complete from cold start." This is incorrect: GerryChain with pair reselection in Python would also handle TX/CA from cold start; the advantage is throughput, not a structural difference in what the algorithm can do.

Additionally, pair reselection modifies the Markov kernel: pairs with low balanced-bipartition feasibility are systematically abandoned more often, which is not part of the standard ReCom specification and may alter the stationary distribution.

**Required revisions**:

1. **Section 4.5 (Texas and Bipartition Failures)**: Revise to clearly state that the bipartition failure is a combinatorial property of the plan space, not a Python limitation. Reframe Rust's advantage as: "pair reselection at microsecond speed rather than second speed reduces warm-up wall time without altering the algorithm's combinatorial behavior." Remove any language suggesting Rust fixes something Python cannot fix.

2. **Table 1**: Revise the TX/CA footnote. GerryChain's cold-start failure on TX/CA is documented for GerryChain *without* pair reselection. The comparison should be explicit: "GerryChain (no pair reselection) vs. redist-ensemble (with pair reselection)." If GerryChain with pair reselection runs TX successfully (even slowly), this must be acknowledged.

3. **Section 7.1 (Conclusion)**: Remove the claim that pair reselection enables "TX and CA runs that Python GerryChain cannot complete from cold start." Replace with: "Rust's throughput advantage reduces the wall time for the pair-reselection warm-up phase from [estimated seconds in Python] to [estimated microseconds]."

4. **Stationarity of pair reselection**: Add a paragraph in Section 3.2 or Section 4.5 that explicitly addresses whether the pair-reselection variant has the same stationary distribution as the original ReCom chain. Options:
   - **Conservative**: State that pair reselection constitutes a modification of the ReCom kernel and that stationarity preservation is a conjecture to be verified empirically in Phase 2 (by comparing empirical plan-space distributions to G.1 results).
   - **Stronger**: Provide a formal argument. One such argument: pair reselection is equivalent to a Metropolis rejection step that discards infeasible pair proposals, which preserves detailed balance if the pair-selection probability is symmetric and proposals are rejected uniformly. However, this argument requires verification against the specific pair-selection distribution in Algorithm 2.

Recommendation: Use the conservative option for Round 2. Cite the planned empirical comparison (Section 7.3) as the evidence that will settle this.

---

### R03 + R04: Wilson's Complexity — Abstract and Planarity

**The problem**: The abstract states "Wilson's algorithm achieves $O(|V|\log|V|)$ expected time on planar graphs" without mentioning cover time. Section 3.2 is correct but Sections 1.2 and the abstract undo this clarity.

**Required revisions**:

1. **Abstract (line 91)**: Change to: "Wilson's algorithm runs in expected time equal to the cover time of a random walk on the subgraph; for planar graphs, this is $O(|V|\log|V|)$~\citep{aldous1991random}."

2. **Section 1.2**: Replace "Wilson's algorithm runs in $O(\tau_{\text{cover}})$ expected time, where $\tau_{\text{cover}}$ is the cover time... For planar graphs... this is $O(|V|\log|V|)$" with a version that clearly makes the Aldous citation the source of the planar bound, not Wilson's original paper.

3. **Planarity claim**: Add a sentence in Section 3.2 or Section 4.2 stating that census-tract adjacency subgraphs used in practice are planar (by construction from TIGER shapefiles, which define non-crossing polygons). If known counter-examples exist (tri-point artifacts, state-line boundaries), acknowledge that these affect at most a small fraction of steps and bound the impact.

---

### R05: SHA-256 Seed Encoding Specification

**The problem**: The seed formula $\text{seed}_i = \text{SHA256}(\text{"ENSEMBLE\_CHAIN\_"} \| i \| \text{"\_"} \| \text{base\_seed})$ does not specify how $i$ and base\_seed are encoded (decimal string? fixed-width binary?). This creates collision ambiguity (e.g., chain 1 + seed "00" vs. chain 10 + seed "0" produce the same hash input if formatting is not controlled). The mapping from 256-bit SHA-256 output to SmallRng's 128-bit state is also unspecified.

**Required revisions**:

1. **Section 3.4**: Replace the informal concatenation with a precise encoding. Recommended: encode $i$ and base\_seed as 8-byte big-endian unsigned integers, separated by a fixed-length separator or by position. The hash input is then the 32-byte string: `b"ENSEMBLE_CHAIN_" || i.to_be_bytes() || b"_" || base_seed.to_be_bytes()`.

2. **Section 4.3**: State explicitly that SmallRng is seeded from the first 16 bytes (128 bits) of the SHA-256 output, interpreted as two little-endian u64 values (matching xoshiro128++'s native state format), or alternatively that the SHA-256 output is fed to a SeedableRng::from_seed call with the first 16 bytes. Specify the Rust `rand` crate version to pin the behavior.

---

### R06: Falsifiable Phase 2 Benchmark Commitment

**The problem**: Section 5.4 commits to criterion.rs benchmarks but does not specify hardware, comparison protocol, or acceptance criterion. A commitment that cannot fail is not a commitment.

**Required revisions**:

1. **Section 5.4**: Add the following specification:
   - **Hardware**: Report the CPU model, clock speed, L2/L3 cache, and OS used for the benchmark run.
   - **GerryChain comparison**: Specify Python version, GerryChain version, NumPy version, starting plan (cold start from a specific valid plan, defined by census-year FIPS state code), and measurement method (wall-clock for 1,000 steps after a 50-step warm-up).
   - **Acceptance criterion**: The Phase 2 report will confirm the estimate if criterion.rs measurement exceeds 30,000 steps/sec for NC (i.e., within 40% of the 50,000 target). If the measurement falls below 30,000 steps/sec, the speedup estimate will be revised downward and the abstract and Table 1 will be corrected.
   - **Variance reporting**: Report the 95% confidence interval from criterion.rs, not just the point estimate.

---

### R07: R-hat Convergence Semantics for Redistricting

**The problem**: The paper presents R-hat < 1.05 as indicating convergence of the ensemble chain. But R-hat convergence on a scalar summary statistic (seat count, compactness) is not the same as convergence of the underlying plan distribution.

**Required revisions**:

1. **Section 6.1**: Add a paragraph acknowledging that R-hat diagnostics applied to summary statistics certify convergence of the marginal distribution of those statistics, not convergence of the full plan-space distribution. A chain could have low R-hat on Democratic seat share while being stuck in a topologically restricted region of plan space. Recommend citing the discussion in Autry et al. (2021) on multiscale mixing, which addresses this point directly.

2. **Section 1.1**: Revise "give courts and litigants justified confidence in the ensemble's stationarity" to "give researchers and practitioners justified confidence that the marginal distributions of summary statistics have converged across chains."

---

## Recommended Changes (Moderate)

### R08: Hamming Proxy Exactness Claim

In Section 6.3, replace "This proxy is exact in the sense that $\phi$ is the same summary statistic minimized by METIS" with a precise statement of what the proxy achieves: "$\phi(\sigma)$ is used as a computationally efficient summary for tracking chain progress; it is not a collision-free plan identifier, but its correlation with plan-to-plan Hamming distance makes it a useful running diagnostic." Remove the word "exact."

### R09: Legal Framing — AEA Replication vs. Court Admissibility

In Sections 1.1, 7.2, and 7.3, distinguish two separate contributions of redist-ensemble:
- **AEA replication improvement**: Removing the Python GerryChain dependency enables full reproducibility from the Rust binary. This is a genuine and legally-adjacent contribution.
- **Throughput improvement**: Enabling longer runs and more chains is valuable for practitioners and for internal methodology quality, but does not alter a court's admissibility determination, which depends on methodological acceptance, not run length.

### R10: Overhead Factor Sensitivity

In Section 5.2, add a brief sensitivity table: "If the actual overhead is $26\times$ (rather than $13\times$), the estimate falls to 25,000 steps/sec, giving a 1,200× speedup. If overhead is $6.5\times$, the estimate rises to 100,000 steps/sec, giving a 4,700× speedup." This frames the estimate's uncertainty without undermining the conclusion.

### R11: GerryChain Baseline Reproducibility

Add to Section 5.1: CPU model, Python version, GerryChain version, starting plan type (cold start from seeded random plan?), and whether measurements were taken with or without the NumPy sparse adjacency backend. Specify that no warm-up steps were excluded from the 1,000-step count.

### R12: SmallRng and BigCrush Claim

In Section 4.3, replace "passes the BigCrush statistical test suite" with "passes the PractRand statistical test suite" (xoshiro128++ passes PractRand up to large byte counts; its BigCrush status depends on the specific variant and is not established in the rand crate documentation). If BigCrush results are available for xoshiro128++, cite them explicitly.

### R13: CA Per-Step Cost Claim

In Section 5.3, revise "California ($k=52$) presents the highest per-step cost due to its large $n$ and high $k$" to correctly reflect that per-step cost is $O((n/k)\log(n/k))$. For CA: $m \approx 2 \times 8057 / 52 \approx 310$. For PA: $m \approx 2 \times 3218 / 17 \approx 379$. PA has a larger merged region than CA despite lower $n$. The claim should be verified and corrected or qualified.

---

## Minor Changes

| ID | Location | Fix |
|----|----------|-----|
| R14 | Section 6.3 | Clarify Hamming autocorrelation: define whether $d_H(\sigma_t, \sigma_0)$ uses a fixed reference plan or whether Ham(k) uses plan-to-plan distances $d_H(\sigma_t, \sigma_{t+k})$ |
| R15 | Section 1.1 | Add case citations: League of Women Voters v. Pennsylvania (2018), Harper v. Hall (2022/2023), Harkenrider v. Hochul (2022) to support the state-court acceptance claim |
| R16 | Abstract | Add dagger notation or "(estimated)" to the 50,000 steps/sec and 2,300× speedup figures |
| R17 | Section 6.2 | Define $\hat{\rho}_k^+$ inline or cross-reference Section 6.1 where rank normalization is defined |
| R18 | Section 4.5 | State serde output schema version as a versioned constant (e.g., `ensemble_output_version: "1.0"`) |

---

## Phase 2 Dependencies

Several issues in this revision plan cannot be fully resolved by textual revision — they require Phase 2 implementation or empirical results:

1. **Stationarity of pair reselection (R02)**: Final resolution requires the empirical G-track replication (Section 7.3), comparing redist-ensemble and GerryChain ensemble distributions on the six G-track states.
2. **Throughput validation (R06)**: criterion.rs benchmarks are required before the 50,000 steps/sec figure can be stated as measured.
3. **CA per-step cost (R13)**: Requires running the benchmark for all six states to verify the per-step cost ordering.

These items should be noted as "deferred to Phase 2" in the revised paper, with the existing dagger notation extended to cover all figures that depend on these results.

---

## Priority Order for Revision

1. R01/R02 (Texas/stationarity) — most likely to be raised by a hostile expert in litigation context
2. R03/R04 (Wilson complexity abstract) — straightforward textual fix, high visibility
3. R05 (SHA-256 encoding) — low effort, high reproducibility value
4. R06 (benchmark falsifiability) — requires one additional paragraph in Section 5.4
5. R07 (R-hat semantics) — requires one paragraph in Section 6.1 and one sentence edit in Section 1.1
6. R08–R13 (moderate) — batch fix in one editing pass
7. R14–R18 (minor) — batch fix in one editing pass
