# Track G — Ensemble Methods and Comparison: Panel Review

**Track**: G-ensemble
**Papers reviewed**: G.0–G.14 (15 papers)
**Panel date**: 2026-05-07
**Module arc**: Establish where the bisection plan sits in the full space of valid redistricting plans, certify that the ensemble comparison is statistically trustworthy, and provide practitioners with a principled algorithm selection framework.

---

## Panel Composition

| ID | Role | Expertise | Primary assignments |
|----|------|-----------|---------------------|
| P1 | MCMC / Statistics — mixing theory | Spectral gaps, conductance, Poincaré inequalities, canonical paths | G.4, G.5, G.9, G.10 |
| P2 | MCMC / Statistics — convergence diagnostics | R-hat, ESS, PSRF, Gelman-Rubin, rank-normalized diagnostics | G.0, G.4, G.5, G.12 |
| P3 | MCMC / Statistics — Markov chains for redistricting | ReCom, Forest ReCom, SMC, short-burst | G.7, G.9, G.10, G.11 |
| P4 | Political science methods — GerryChain comparison | Ensemble construction, plan comparison, DeFord/Herschlag methodology | G.1, G.2, G.3, G.14 |
| P5 | Political science methods — redistricting ensembles | Partisan distributions, proportionality, outlier analysis | G.0, G.1, G.2, G.3 |
| P6 | Computation / Systems | Performance benchmarking, Rust, Rayon, parallel implementation | G.7, G.11, G.12 |
| P7 | Election law | Rucho, Allen v. Milligan, Daubert, VRA Section 2, litigation use | G.1, G.13, G.14 |

---

## Module Score Summary

| Paper | Sub-track | Status | Score |
|-------|-----------|--------|-------|
| G.0 | ensemble-baseline | draft | unscored |
| G.1 | ensemble-baseline | draft | 2.6/4 (R1); conditional 3.0/4 (Duchin R2) |
| G.2 | ensemble-baseline | draft | unscored |
| G.3 | ensemble-baseline | draft | unscored |
| G.4 | ensemble-baseline | draft | 3.0/4 (R1) |
| G.5 | ensemble-baseline | draft | unscored |
| G.6 | mcmc-algorithms | ready | 3.0/4 |
| G.7 | mcmc-algorithms | draft | unscored (first draft) |
| G.8 | mcmc-algorithms | ready | 3.4/4 |
| G.9 | mcmc-algorithms | ready | 3.4/4 |
| G.10 | mcmc-algorithms | ready | 3.2/4 |
| G.11 | mcmc-algorithms | ready | 3.0/4 |
| G.12 | mcmc-algorithms | draft | unscored (spec accepted) |
| G.13 | mcmc-algorithms | ready | 3.8/4 |
| G.14 | practitioner-synthesis | ready | 4.0/4 (highest in full program) |

**Sub-track scores**: ensemble-baseline (G.0–G.5): draft; MCMC-algorithms (G.6–G.13): accepted mean 3.3/4; practitioner-synthesis (G.14): 4.0/4.

---

## Sub-Track 1: Ensemble Baseline (G.0–G.5)

**Panel flag PP1**: All six ensemble-baseline papers are at draft stage while the MCMC-algorithms papers are mostly accepted. This is architecturally inverted — the foundation papers are less developed than the papers that depend on them. G.1 and G.4 are the load-bearing members; elevate them to highest revision priority.

### G.0 — Ensemble Methodology Framework
**Status**: draft | **Venue**: Political Analysis

The paper correctly distinguishes ensemble characterisation from algorithmic plan selection. The statutory implications section (Section 6) is the paper's most original contribution — the bridge to B.16's ConvergenceSweep as a deterministic alternative.

**Gaps**: R-hat threshold inconsistency — body text uses 1.05 while G.4 uses 1.1. All G-series papers must standardise to R-hat < 1.1. The "exactly one AR plan" claim requires precision: uniqueness holds for ConvergenceSweep at T=600 from a specified seed, not for METIS in general. NC PP percentile estimate "65th–75th" needs to be tightened to "68th" following G.1's R2 correction.

### G.1 — GerryChain Congressional Comparison
**Status**: draft | **R1 mean**: 2.6/4 | **Venue**: Political Analysis

This is the key empirical paper in Track G. Primary finding — AR plan at 0.1–0.2nd percentile of edge-cut compactness for WI/GA/PA, at the 50th percentile for NC — is the central empirical claim of the entire G-track. Everything downstream depends on these numbers being correct and replicable.

**NC PP inconsistency**: RESOLVED in R2. AR plan PP corrected from 0.412 to 0.337 (z=0.61, consistent with 68th empirical percentile).

**REMAINING HIGH-PRIORITY GAP — D3 (ESS-based uncertainty, not resolved)**:
Section 3.4 continues to use the formula π-hat ± 1.645 × sqrt(π-hat(1−π-hat)/N) with N as nominal chain length and claims "negligible sampling error." For NC, Herschlag chain ESS is approximately 1,703 out of 24,518. The actual 90% CI is approximately 3.6× wider than reported. Under Daubert's "known error rate" standard, this is a litigation vulnerability. Must be corrected before the paper can be used in court proceedings.

**REMAINING HIGH-PRIORITY GAP — H1 (NC baseline mismatch, not addressed)**:
Herschlag 2020 ensemble uses 2016 presidential vote returns; the AR plan is evaluated under 2020 presidential returns. A robustness paragraph is required.

**REMAINING HIGH-PRIORITY GAP — H3 (VRA section, not addressed)**:
The MM district percentile results (71st for NC, 62nd for GA) have VRA implications not discussed. Paper needs Section 8.1 on minority representation and VRA compliance, cross-referencing G.13 and D.0–D.3.

**TX/CA data gap**: TX and CA remain placeholders. These two states (38 and 52 seats) must be completed before submission.

### G.2 — Partisan Outcome Distributions
**Status**: draft | **Venue**: AJPS

Main finding — AR is within one seat of the ensemble median in 5 of 6 states, with GA at the 38th percentile of Democratic seats — is politically important. The "proportionality corridor" concept is a useful addition to redistricting vocabulary.

**Dependency**: imports ensemble statistics from G.1. D3 ESS correction in G.1 must cascade to G.2's uncertainty intervals.

### G.3 — Compactness Distribution Position
**Status**: draft | **Venue**: Political Analysis

**Key gap**: G.1's primary metric is edge-cut (EC); G.3's is Polsby-Popper (PP). The AR algorithm minimises EC, so the AR plan is at the extreme of the EC distribution (0.1–0.2nd percentile in G.1) but only above-average in PP (61st–72nd percentile in G.3). Both papers are correct, but the tension needs explicit treatment in G.3's introduction.

The corrected NC PP score (0.337) from G.1 R2 is correctly reflected in Table 1.

### G.4 — Ensemble Diagnostics (R-hat, ESS, Hamming)
**Status**: draft | **R1 mean**: 3.0/4 | **Venue**: Statistics

The strongest paper in the ensemble-baseline sub-track. Formalises three complementary convergence diagnostics.

**BLOCKING ISSUE B1**: G.4 uses R-hat < 1.1; G.0 body text uses 1.05. All G-series papers must standardise to 1.1.

**BLOCKING ISSUE B2**: Hamming autocorrelation reference plan underspecified. Recommended: switch to scalar-statistic formulation — use ACF(f(π_t)) where f(π_t) = mean Polsby-Popper.

**HIGH-PRIORITY H1 (ESS table inconsistency)**: NC row reports ρ₁ = 0.87 but ESS = N(1−ρ₁)/(1+ρ₁) = 10,000 × 0.13/1.87 = 695, not 769 as reported. Verify all ESS values against their ρ₁ and formula.

**HIGH-PRIORITY H2 (statutory minimum formula vs. table)**: The formula n_min(k) = max(10,000, 500k) gives TX = 19,000 but the table recommends 25,000. The formula and table must agree.

**HIGH-PRIORITY H3 (G.1 ensemble certification)**: G.4 does not certify the specific ensembles used in G.1. A new Section 5.3 "Certification of G.1 Source Ensembles" is needed.

**HIGH-PRIORITY H4 (ESS minimum at 99th percentile)**: For litigation use of 1st/99th-percentile claims, chain lengths of 20,000–100,000 steps are required.

### G.5 — Convergence and Mixing Analysis
**Status**: draft | **Venue**: Statistics

**CRITICAL**: Abstract states mixing time is O(n² log n); proof gives O(n³ log n). Abstract must be corrected. The O(n² log n) conjecture may be stated as a remark.

The theory-practice ratio of 50,000–100,000× (Table 1) is correctly explained: concentration of the stationary distribution near compact plans means the chain reaches the compact sub-region quickly, far below the worst-case bound.

---

## Sub-Track 2: MCMC Algorithms (G.6–G.13)

**Panel verdict**: Seven of eight papers are accepted (G.6, G.8–G.11, G.13–G.14); G.12 has spec accepted. Mean score 3.3/4. The contributions are: (1) production Rust implementation with SHA-256 audit chains; (2) empirical comparison under unified methodology (NC/WI/TX benchmark at fixed compute budget); (3) the decision tree synthesis in G.14.

### G.6 — Short-Burst for Minimum-Edge-Cut Redistricting
**Score**: 3.0/4 | **Venue**: NeurIPS | **Status**: Accepted

PP improvement of 15–22% over single-seed METIS is single-run. Adding multi-run results with confidence intervals would raise score to ~3.4/4. Comparison with BisectionEnsemble performance envelopes should be addressed.

### G.7 — SMC for Calibrated Redistricting Ensembles
**Score**: unscored | **Venue**: Statistics | **Status**: First draft

G.7 makes the strongest distributional correctness claim in the MCMC-algorithms sub-track: SMC recovers the true uniform distribution without Markov chain mixing assumptions. If sustained, this makes G.7 the foundational paper for calibrated inference.

**Concern**: validated on synthetic graphs and NC only. All other MCMC papers report NC, WI, TX. Phase 2 (SmcPercentile compositor, validation against R redist::redist_smc()) is listed as future work. Until Phase 2 complete, G.7 cannot support G.14's recommendation that SMC is the correct method for court submissions.

**Framing overstatement**: "only SMC provides a statistically valid answer without requiring Markov chain mixing to be demonstrated" overstates the advantage relative to G.9 (Forest ReCom) and G.10 (Merge-Split). Qualify as: "SMC provides an asymptotically exact calibration without mixing assumptions; Forest ReCom and Merge-Split provide approximate calibration via expected detailed balance — a genuine, if weaker, guarantee."

### G.8 — Flip Proposals for Local Sensitivity Analysis
**Score**: 3.4/4 | **Venue**: NeurIPS | **Status**: Accepted

Excellent use-case scoping: Flip is explicitly described as inappropriate for full ensemble sampling but ideal for local sensitivity analysis. EC stability and partisan stability as sensitivity metrics are well-defined. Audit chain (visited_count and selected_plan_rank in manifest) is the right auditability mechanism.

### G.9 — Forest ReCom: Reversible Recombination
**Score**: 3.4/4 | **Venue**: NeurIPS | **Status**: Accepted

Chi-squared verification on the 4-node synthetic graph is the paper's strongest contribution — provides ground-truth empirical test that standard ReCom fails and Forest ReCom passes.

**Clarification needed**: The ratio c_fwd/c_rev "is an unbiased estimator of the true transition probability ratio." This requires precision: is the unbiasedness exact (if Wilson's UST is exact) or approximate?

### G.10 — Merge-Split MCMC: Explicit Reversibility
**Score**: 3.2/4 | **Venue**: NeurIPS | **Status**: Accepted

**Cost comparison concern**: Abstract claims Merge-Split is O(m log m) per step and Forest ReCom is O(m²). If Forest ReCom uses Wilson's UST algorithm, its cost is O(m) expected, not O(m²). The per-step cost difference may be smaller than claimed.

### G.11 — Multi-scale MCMC for Large-k Redistricting
**Score**: 3.0/4 | **Venue**: ICML | **Status**: Accepted

The GEOID-prefix derivation of county adjacency from tract adjacency is an elegant engineering contribution. 40–50% reduction in lag-100 autocorrelation of edge-cut statistic for TX (k=38) at comparable wall-clock time.

**Speedup claim needs correction**: Abstract claims county steps are 20× cheaper. The speedup is approximately 277/10 ≈ 28× at the merger level (derived correctly from merger subgraphs, not full graph sizes). Abstract should present calculation from merger sizes.

### G.12 — Short-Burst with Calibrated Chains
**Score**: unscored | **Venue**: NeurIPS | **Status**: Spec accepted

**Use-case motivation concern**: Standard ShortBurst achieves comparable or slightly better final edge-cut than the calibrated variants because its higher per-burst acceptance rate explores more plans per burst. The ShortBurstForest middle ground needs a clearer use case not already covered by G.6 or G.9.

The answer: SBF produces better ensemble coverage in the compact sub-region because each accepted step is an approximately correct sample. This use case — characterising the distribution of plans near the compactness frontier for litigation support — is distinct from pure optimisation and from global sampling. Should be the paper's lead claim.

### G.13 — VRA-Aware MCMC
**Score**: 3.8/4 | **Venue**: NeurIPS | **Status**: Accepted

The strongest paper in the MCMC-algorithms sub-track. The three-way step accounting identity (steps_accepted + vra_rejections + mh_rejections = steps_taken) is an elegant and complete auditability mechanism. VRA hard-rejection correctness proof is mathematically sound.

**Cross-track gap (D.5 cross-reference missing)**: G.13 correctly cites VRASection (D.0) as the complementary structure-layer tool. However, the paper does not reference D.5 (Gingles bloc-voting methodology), which determines which communities meet the three-part Gingles test. Without D.5, the legal basis for the protected_districts list in the audit record is undocumented.

---

## Sub-Track 3: Practitioner Synthesis (G.14)

### G.14 — A Practitioner's Comparison of Redistricting Ensemble Algorithms
**Score**: 4.0/4 (highest in full program) | **Venue**: JASA | **Status**: Accepted

G.14 is exceptional. The four-question decision tree (Q1: VRA compliance required? Q2: k ≥ 15? Q3: Calibrated distribution required? Q4: Pure compactness optimisation?) correctly partitions the use-case space. The decision tree follows from the algorithms' structural properties established in G.6–G.13, not solely from single-run empirical results.

The single-run honesty (all results are single-run; illustrative rather than definitive) strengthens rather than weakens the paper.

**Three targeted additions needed**:
1. **VraRecom prerequisite note**: VraRecom requires a VRA-compliant initial plan from VRASection (--structure ratio-optimal-vra)
2. **VRA × large-k intersection**: For TX k=38 with VRA requirements, both constraints apply; Q1 and Q2 routing must handle the intersection
3. **G.7 contingency footnote**: The SMC recommendation is contingent on G.7 Phase 2 completion. Until then, MergeSplit (G.10) is the fallback

---

## Cross-Track Issues

### PP1 — CRITICAL: Inverted Foundation Architecture (ensemble-baseline sub-track all draft)
The ensemble-baseline sub-track (G.0–G.5) is entirely at draft stage, while MCMC-algorithms papers (G.6–G.13) are mostly accepted.

**Panel recommendation**: Elevate G.1 (ESS correction, TX/CA data completion) and G.4 (ESS table, statutory formula reconciliation) to highest revision priority. These two papers are the load-bearing members. G.0, G.2, G.3, and G.5 can proceed in parallel once G.1 and G.4 are resolved.

### PP2 — G.7 (SMC, first draft) Underlies G.14's Strongest Recommendation
G.14 at 4.0/4 recommends SMC for court submissions. G.7 is the program's most incomplete algorithm paper — first draft, no review score, NC only.

**Panel recommendation**: Either (a) fast-track G.7 through R1 review with NC/WI/TX empirical results and Phase 2 completion; or (b) add explicit contingency footnote in G.14 that SMC is contingent on G.7 Phase 2 completion and MergeSplit (G.10) is the fallback until then.

### PP3 — G.1 Stability Under Ensemble Size
The 0.1–0.2nd percentile results are all from 1,000-step ensembles. G.5's mixing time analysis suggests NC chains at 1,000 steps are running just under the mixing threshold. Stability must be demonstrated: report results at 1,000, 5,000, and 10,000 steps for NC and WI.

### PP4 — VRA Cross-Track Consistency (G.13 ↔ D.5)
G.13 integrates correctly with VRASection (D.0). However, it does not reference D.5 (Gingles bloc-voting methodology), which determines which communities must have majority-minority districts preserved. Without D.5, the protected_districts list has no documented legal basis.

### PP5 — Distributional Claim Hierarchy Inconsistency (G.9 vs. G.10)
The program presents: SMC (exact) > Forest ReCom (expected detailed balance) > Merge-Split (approximate). However, Merge-Split's ratio valid_cuts_fwd / valid_cuts_rev may equal Q(π' → π) / Q(π → π') exactly under uniform cut selection — a stronger guarantee than Forest ReCom's expected detailed balance. The hierarchy may be incorrect at the G.9 vs. G.10 level.

---

## Review Files Found

| Paper | reviews/ directory | R1 files | R2 files | Revision plan |
|-------|-------------------|----------|----------|---------------|
| G.0 | Yes | 5 | 5 | Yes |
| G.1 | Yes | 5 | 5 | Yes |
| G.2 | Yes | 5 | 5 | Yes |
| G.3 | Yes | 5 | 5 | Yes |
| G.4 | Yes | 5 | 0 | Yes |
| G.5 | Yes | 5 | 5 | Yes |
| G.6–G.14 | No | — | — | — |

No _panel.yaml files found in any G-track paper directory.

*Panel convened 2026-05-07. Track G — 15 papers across three sub-tracks: ensemble-baseline, mcmc-algorithms, practitioner-synthesis.*
