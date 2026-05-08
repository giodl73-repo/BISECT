# G-Track Panel Review — Batch 2

**Date**: 2026-05-07
**Reviewer panel**:
- R1: Karypis (graph algorithms / METIS)
- R2: Rodden (political science)
- R3: Duchin (math / redistricting)
- R4: Stephanopoulos (law)
- R5: Liang (ML / AI systems)

**Papers reviewed**: G.2, G.4, G.7, G.10, G.11, G.14, G.15

**Scale**: 0 = reject, 1 = major revision, 2 = minor revision, 3 = accept, 4 = strong accept

**Verdict thresholds**: Accept (avg ≥ 3.0) | Minor Revision (avg ≥ 2.5) | Major Revision (avg ≥ 2.0) | Reject (avg < 2.0)

---

## G.2 — Partisan Outcome Distributions

**Title**: Are Algorithmic Maps Partisan? Situating ApportionRegions Within the ReCom Ensemble

### Summary

G.2 situates the ApportionRegions (AR) algorithm's outputs within ReCom ensemble distributions for six states, arguing that AR is not systematically partisan. The paper introduces the "proportionality corridor" metric and shows AR falls inside it for five of six states, with the Georgia deviation attributed to geographic sorting rather than algorithmic bias.

### Reviewer Scores

| Reviewer | Score | Key concern |
|----------|-------|-------------|
| R1 (Karypis) | 3 | Ensemble construction is technically sound; good use of dagger notation for TX/CA estimates |
| R2 (Rodden) | 3 | Correctly applies Rodden concentration effect; Georgia analysis is rigorous; robustness to electoral baseline is a genuine strength |
| R3 (Duchin) | 2 | Section 04 (proportionality corridor) not present in section list but referenced in abstract; no section file found — corridor definition section appears missing from the manuscript |
| R4 (Stephanopoulos) | 3 | Legal framing (step 1–4 argument) is tight; Rucho citation is standard; Callais macro defined but not used in body text |
| R5 (Liang) | 2 | TX/CA percentiles are literature-bound estimates presented without derivation; single-state Georgia claim rests on one AR run; no multi-seed evidence shown |

**Average score**: 2.6

### P1 Blockers

1. **Missing section file**: `sections/04-proportionality-corridor.tex` is referenced in `main.tex` (`\input{sections/04-proportionality-corridor}`) but no file with that content appears in the directory listing. The manuscript compiles only with an incomplete section or a placeholder. The corridor definition and computation must be present to support the abstract claim "we characterise the proportionality corridor."

### P2 Items

1. The Georgia "one fewer Democratic seat" claim is a single-run AR result. Add a footnote with dagger notation and a brief multi-seed sensitivity note (even two seeds would suffice).
2. The `\callais` macro is defined in `main.tex` but does not appear to be used anywhere in the body — either use it or remove the definition.
3. Electoral baseline robustness ("stable to within 5 percentage points") is described but not shown in any table. Add a two-row table for the 2016 and 2018 baselines alongside 2020.
4. The conclusion claims AR "falls inside the corridor for NC, WI, and MN" but MN is not in the primary six-state table — clarify where the MN data comes from.

### P3 Items

- No `redist` binary name occurrences found in this paper (clean). No wrong CLI flag found. No invented court citations.

**Verdict**: Minor Revision (avg = 2.6)

---

## G.4 — Ensemble Diagnostics Paper

**Title**: Certifying MCMC Convergence for Redistricting Ensembles: R-hat, ESS, and Hamming Diagnostics

### Summary

G.4 formalises three MCMC convergence diagnostics (R-hat, ESS, Hamming autocorrelation) for redistricting ensembles, derives state-specific thresholds from ten parallel ReCom chains across six states, and proposes a statutory evidentiary standard. The paper argues ConvergenceSweep is preferable for plan generation while MCMC diagnostics remain irreplaceable for plan audit.

### Reviewer Scores

| Reviewer | Score | Key concern |
|----------|-------|-------------|
| R1 (Karypis) | 3 | R-hat derivation is textbook-correct with proper Vehtari 2021 update; Hamming canonicalisation caveat (Hungarian vs greedy) is honest; ESS via Geyer's monotone sequence estimator is appropriate |
| R2 (Rodden) | 3 | The two-task framework (generation vs evaluation) is the key conceptual contribution and is stated clearly; avoids over-claiming about what convergence implies |
| R3 (Duchin) | 2 | The claim "ESS > 500 requires at minimum 10,000 steps for all six study states" is read from Table 5.2 but the table shows ESS values at 10,000 steps, not the minimum steps to reach ESS 500 — this is a logical gap that must be addressed |
| R4 (Stephanopoulos) | 3 | LWV v. PA citation is correct (landmark state-court case). Rucho v. Common Cause citation is correct. Districting Integrity Act is a model statute, clearly labeled as such. |
| R5 (Liang) | 2 | The abstract CLI command uses `redist analyze` (old binary name — should be `bisect analyze`); same in section 01 introduction and section 08 implementation. The reported diagnostics (Table 5.1) are from a single run per state — need variance estimates or multi-run confirmation for statutory-evidentiary claims. |

**Average score**: 2.6

### P1 Blockers

1. **Binary name `redist` throughout**: Abstract (`\texttt{redist analyze --types ensemble-diagnostics}`), section 01 (`\texttt{redist} Rust binary`, `redist analyze --types ensemble-diagnostics`), section 08 (`redist analyze`, `\texttt{redist-analysis}` crate, `\texttt{redist-core}`, `\texttt{redist-analysis}::ensemble_diagnostics`) — all must be updated to `bisect`. The rename from `redist` to `bisect` was completed in Phase 1; papers must reflect this. **This is a systemic error throughout the paper.**

2. **Statistical gap in Section 5.3**: The paper states "ESS > 500 requires at minimum 10,000 steps for all six study states" but Table 5.1 shows ESS at 10,000 steps, not the step count at which ESS first crosses 500. Either: (a) add a column showing "steps to ESS > 500" derived from the checkpoint data, or (b) rewrite the claim to accurately reflect what the table shows ("at 10,000 steps, ESS > 500 for all states").

### P2 Items

1. The Abstract states "certified 5,000-step minimum recommended for all states" but the statutory section recommends 10,000 steps as the minimum for states with k ≤ 20. This is a contradiction — one of these must change.
2. The `\texttt{redist analyze}` command in section 08 is missing the `--chains` and `--steps` flags shown in the code block above it — the two presentations are inconsistent.
3. The ESS reported in the abstract ("approximately 300–850") matches the table but the ESS(D) column shows values up to 1,765. The abstract only mentions PP-ESS; clarify scope.
4. R-hat for "D-seats" is mentioned but never formally defined — add the rank-normalised variant in section 02.

### P3 Items

- `--weights geographic` flag not seen; no invented court citations.

**Verdict**: Minor Revision (avg = 2.6)

---

## G.7 — SMC Redistricting

**Title**: Sequential Monte Carlo for Calibrated Redistricting Ensembles

### Summary

G.7 implements Sequential Monte Carlo (SMC) for redistricting as the `redist-smc` Rust crate, derives the importance-weighting formula and connectivity invariant from first principles, and proves three propositions establishing weight correctness, connectivity preservation, and ESS degradation. The paper argues SMC is the only method providing statistically valid percentile claims without Markov chain mixing assumptions.

### Reviewer Scores

| Reviewer | Score | Key concern |
|----------|-------|-------------|
| R1 (Karypis) | 3 | Kahan summation implementation is technically sound; Rayon parallelism design is correct (particles are independent until ESS check); SHA-256 domain separation justification is proper; Wilson UST complexity claim is accurate |
| R2 (Rodden) | 3 | The calibration vs. mixing argument is the correct framing; the Fifield 2020 empirical citation is appropriate; the inference scope limitation (percentile claims require calibrated distribution) is properly restated |
| R3 (Duchin) | 3 | Proposition 1 (weight correctness) and Proposition 2 (connectivity invariant) are rigorously stated and proved; the water-boundary caveat in G.11 is appropriately cross-referenced |
| R4 (Stephanopoulos) | 3 | Court-facing framing is careful — "statistically valid answer" is not overclaimed; no invented case citations |
| R5 (Liang) | 1 | The abstract and implementation section prominently reference `redist-smc` crate and `redist ensemble --method smc` CLI command — both use the old `redist` binary name; must be `bisect-smc` / `bisect ensemble --method smc`. Additionally, the NC L2 test uses only N=200 particles with a single seed; all quantitative claims from this test are explicitly flagged as "should not be over-interpreted" which is appropriate, but the 48-test L1 suite claims are not reproducible without a test manifest. |

**Average score**: 2.6

### P1 Blockers

1. **Binary/crate name `redist`**: Abstract (`\texttt{redist-smc}` Rust crate, `\texttt{redist ensemble --method smc}`), section 01 (`\texttt{redist-smc}`, `\texttt{bisect ensemble --method smc}` — note: section 01 already uses `bisect` in the CLI example at line 47, but abstract uses `redist ensemble`), section 05 (`\texttt{redist-smc}` L2 test), section 06 (`\texttt{redist-smc}`, `\texttt{redist::redist\_smc()}`). The inconsistency is glaring: section 01 correctly uses `bisect ensemble --method smc` in the CLI example but the abstract uses `redist ensemble --method smc`. Standardise to `bisect-smc` / `bisect ensemble`.

### P2 Items

1. Proposition 3 (ESS degradation model) is referenced in the abstract but the proposition text is not visible in the sections reviewed. Confirm it is in `sections/02-algorithm.tex` and properly proved.
2. The comparison table (Section 3, Table 1) lists six methods but section 01 says "five distinct search and sampling methods" — the count is inconsistent. Resolve (Flip makes six).
3. The N=200 L2 test produces ESS 140–180. This is appropriate for a development test but should not be extrapolated to production guidance (N=5,000 is the recommendation). Add a sentence separating development-test results from production parameters.
4. The resampling rule `τ = 0.5` (resample when ESS drops below N/2) is stated as "default" but never justified. Add a one-sentence justification or cite Del Moral 2004.

### P3 Items

- No invented court citations. No `--weights geographic` issue in this paper.

**Verdict**: Minor Revision (avg = 2.6)

---

## G.10 — Merge-Split MCMC

**Title**: Merge-Split MCMC: Explicit Reversibility via Two-Tree Acceptance Ratio

### Summary

G.10 presents the Merge-Split chain as the preferred middle ground between standard ReCom (unknown stationary distribution) and Forest ReCom (exact but O(m²)). The paper derives the acceptance ratio from first principles — forward cuts in the numerator — and provides empirical results on NC (k=14) and WI (k=8) showing ~60% acceptance rate and distributional agreement with Forest ReCom.

### Reviewer Scores

| Reviewer | Score | Key concern |
|----------|-------|-------------|
| R1 (Karypis) | 4 | Algorithm box is rigorous; SHA-256 seed derivation is correctly specified; the pair-count assumption (|P(π)| ≈ |P(π')|) is honestly bounded; per-step cost proof is clean; Theorem 3.3 (approximate detailed balance) proof sketch is adequate |
| R2 (Rodden) | 3 | Partisan seat distributions on NC and WI at 5 independent seeds — the ±3% standard error acknowledgment is appropriate; 4pp method difference within sampling variation is correctly interpreted |
| R3 (Duchin) | 3 | The detailed balance proof (Theorem 3.3) is labeled "proof sketch" and defers to Janson 2022 Proposition 2 — this is honest; the approximation in E[cuts_fwd/cuts_rev] ≈ 1 is the key claim and needs more than "approximately identically distributed" — but acceptable for a conference paper |
| R4 (Stephanopoulos) | 3 | No legal citations; appropriate scope — does not over-claim about legal applications |
| R5 (Liang) | 2 | The empirical results on NC and WI are from 5 seeds (seeds 42–46) at T=1,000 steps — acknowledged as "likely too short for full plan-space convergence." The conclusion calls MergeSplit "recommended for production" based on these 1,000-step results; this is inconsistent — production runs should use ≥5,000 steps (per G.4). The one mention of `redist-cli` in section 01 introduction (line 59) uses old binary name. |

**Average score**: 3.0

### P1 Blockers

None.

### P2 Items

1. **Binary name**: Section 01 (`\texttt{redist-cli}`) — should be `bisect-cli` or just `bisect`. One occurrence; easy fix.
2. The conclusion states MergeSplit is "recommended for production redistricting ensembles" but all empirical evidence uses T=1,000 steps. Add a note that production use requires ≥5,000 steps per G.4 certified minimums for NC and WI.
3. Theorem 3.3 "proof sketch" defers to Janson 2022 Proposition 2 but the janson2022 citation in the abstract claims `\citep{janson2022}` — verify this is the correct paper/proposition number, as Janson 2022 is a preprint on random graph partitioning, not specifically on the two-tree MH correction.
4. The Remark on pair-count assumption says the correction factor is bounded "within 10% of 1" for NC. This should be stated more carefully — it is the ratio of pair counts, not the correction to the MH acceptance probability (which is already min(1, α)).

### P3 Items

- No invented court citations. No `--weights geographic` issue.

**Verdict**: Accept (avg = 3.0)

---

## G.11 — Multi-Scale MCMC

**Title**: Multi-scale MCMC: Hierarchical Mixing for Large-k Redistricting

### Summary

G.11 implements the multi-scale MCMC approach (coarse county-level + fine tract-level ReCom) as `MultiScaleChain` in `redist-ensemble`, exposed via `--search multiscale`. The paper derives county adjacency from the tract graph via GEOID prefix matching, proves a connectivity invariant, and demonstrates 43% lag-100 autocorrelation reduction for Texas (k=38) at 2,000 steps across 5 independent seeds.

### Reviewer Scores

| Reviewer | Score | Key concern |
|----------|-------|-------------|
| R1 (Karypis) | 3 | County adjacency derivation (Proposition 3.1) is proved correctly; GEOID prefix matching is exact for TIGER/Line products; algorithm box is complete; rebalance subroutine is well-specified with the 200-iteration hard limit; coarse tolerance factor of 3 is justified inline |
| R2 (Rodden) | 3 | The 5-seed validation with bootstrapped standard errors and paired t-test (p<0.01) for the 43% ACF reduction is solid; the Autry et al. speedup comparison caveat (different state, different statistic) is honest |
| R3 (Duchin) | 3 | Remark 3.8 (stationarity) honestly states the distribution is not characterised; the water-boundary caveat in Proposition 3.1 remark is important and properly scoped; validity proof (Proposition 4.1) handles the coarse→fine connectivity correctly via BFS check |
| R4 (Stephanopoulos) | 3 | No legal citations; appropriate scope — paper does not claim legal utility beyond "faster mixing" |
| R5 (Liang) | 2 | `redist-cli` appears in the footnote to Table 2 (runtime footnote) and in section 01 and section 06 conclusion — all use old binary name. The abstract reports "16%–43%" ACF reduction range but conflates two different experiments at different T values (T=1000 from G.15 cross-comparison vs. T=2000 measured here); the reported range should be presented as two separate results, not a single range. |

**Average score**: 2.8

### P1 Blockers

None.

### P2 Items

1. **Binary name throughout**: `redist-ensemble`, `redist-cli`, `--search multiscale in the redist-cli` (section 01), `redist-cli release build` (footnote), `redist-cli` (section 06 conclusion) — all should use `bisect`. This is the most common issue in this paper.
2. **Abstract ACF range conflation**: The abstract states "16%–43% relative to standard ReCom" but these come from two different experiments (T=1000 from G.15 vs. T=2000 measured here). The abstract must attribute these separately: "43% at T=2000 (this paper); 16% at T=1000 (G.15 cross-comparison)."
3. Proposition 4.1 (Fine Plan Validity) notes that connectivity of fine districts after a coarse move requires BFS check — but the algorithm box (Algorithm 3.1) does not explicitly include a BFS check in the rebalance subroutine. Ensure the algorithm and the proof are consistent.
4. The coarse tolerance factor of 3 rationale (inline in section 03) is a paragraph but would be clearer as a named Remark to make it discoverable.

### P3 Items

- No invented court citations. No `--weights geographic` flag issue found in this paper.

**Verdict**: Minor Revision (avg = 2.8)

---

## G.14 — Ensemble Comparison

**Title**: A Practitioner's Comparison of Redistricting Ensemble Algorithms

### Summary

G.14 systematically compares all eight implemented ensemble and search algorithms (Short-Burst, SMC, Flip, Forest ReCom, Merge-Split, Multi-scale, ShortBurstForest, VRA-Aware ReCom) on NC, WI, and TX across four metrics, distilling results into a four-question practitioner decision tree.

### Reviewer Scores

| Reviewer | Score | Key concern |
|----------|-------|-------------|
| R1 (Karypis) | 3 | All eight algorithms are correctly characterised algorithmically; the Forest ReCom ~8% overhead claim is plausible; the Multi-scale coarse speedup arithmetic (29% per-step reduction) is correct |
| R2 (Rodden) | 3 | Partisan seat stability (5 seeds for NC) is appropriate scope; the 100% stability of Short-Burst is correctly interpreted as concentration artifact; the distributional hierarchy is well-organised |
| R3 (Duchin) | 2 | All results in Table 4.3 (autocorrelation by state) are single-run, seed 42 — but the caption says "single-run, seed 42" so this is disclosed. The problem is that the decision tree in section 05 makes definitive algorithm recommendations ("use Forest ReCom for calibrated distribution") based on single-run evidence. The tree should carry single-run caveats where it uses ACF numbers to rank methods. |
| R4 (Stephanopoulos) | 3 | The `--weights geographic` CLI examples in section 05 should be `--weights-override geographic` — this is a systematic flag error. Rucho and state court references are standard. |
| R5 (Liang) | 2 | `redist-ensemble`, `redist` CLI, `redist::redist_smc()` references throughout. The abstract's acknowledgment that "all results are single-run (seed 42) and should be treated as illustrative" is correct and appropriate — but the decision tree in section 05 then drops this caveat and presents rankings as definitive. |

**Average score**: 2.6

### P1 Blockers

1. **`--weights geographic` flag error**: Section 05 (decision framework, lines 255 and 258) uses `--weights geographic` in two CLI example snippets. The correct flag is `--weights-override geographic`. This is a CLI documentation error that could mislead practitioners.

### P2 Items

1. **Binary name throughout**: `redist-ensemble`, `redist` CLI, `redist::redist_smc()` (section 05 decision framework), `redist` platform (section 06 conclusion) — all should use `bisect`.
2. The decision tree section (section 05) makes ranking claims based on Table 4.3 ACF values (single-run) without repeating the single-run caveat stated in the abstract. Add a brief reminder at the start of section 05.
3. The VRA-Aware section notes it "reduces to standard Forest ReCom when VRA constraint is not active" — but the table shows VRA-Aware with ACF(100) = 0.49 vs Forest ReCom 0.48 for TX, attributed to "implementation details of adaptive boost initialisation." This should be investigated or the table footnoted more explicitly.
4. Section 06 conclusion references future algorithms (CVD and BFS Growth as "in development") — per G.15, these are now implemented. Update the future work section.
5. The SMC runtime uses N=1,000 particles (Table 4.4) but the recommended production N for NC is N=5,000 (per G.7). The 22.8s NC runtime is for a subproduction particle count; note this.

### P3 Items

- No invented court citations.

**Verdict**: Minor Revision (avg = 2.6)

---

## G.15 — Comprehensive Comparison

**Title**: A Comprehensive Comparison of Redistricting Structure and Search Algorithms

### Summary

G.15 extends G.14 to the full 15-algorithm portfolio across both structure and search layers, adding five states (including FL and NH), six structure algorithms, and three new search algorithms (Parallel Tempering, SMC-Percentile, Adaptive Multi-scale). The paper provides a six-question decision framework and establishes ILP as the certified baseline for NH.

### Reviewer Scores

| Reviewer | Score | Key concern |
|----------|-------|-------------|
| R1 (Karypis) | 3 | ILP certified-optimal result for NH is correctly described (zero integrality gap); SA improvement correctly references METIS initialisation; BFS Growth O(N log N) claim is accurate; the structure hierarchy (EC and PP orderings) is rigorous and consistent across states |
| R2 (Rodden) | 3 | Dagger notation is applied systematically (FL, NH, new algorithms); G.14 baselines carried forward without re-running is appropriate; partisan stability of PT (5/5, 100%) is correctly interpreted as near-compact concentration |
| R3 (Duchin) | 3 | Single-run caveat is in the abstract with explicit dagger notation; the conclusion correctly states "PT vs AMS for TX ACF is inconclusive from single-run estimates and requires multi-seed validation" — this is proper epistemic humility; the six-question framework is internally consistent |
| R4 (Stephanopoulos) | 2 | Section 06 decision framework at line 246 states "All flags are for the `redist` CLI" — should be `bisect`. More substantively, the `redist fetch` command reference in section 03 (CVD-Geo centroid data paragraph) is wrong CLI name. Section 08 conclusion lists crate paths using `redist-core`, `redist-ilp`, `redist-ensemble`, `redist-cli` throughout — all wrong. The paper is a practitioner reference; wrong CLI names directly harm usability. |
| R5 (Liang) | 3 | The abstract's dagger notation and single-run disclosure are thorough. The AMS vs PT comparison correctly acknowledges uncertainty ("inconclusive from single-run estimates"). The G.14 baseline carry-forward is methodologically sound. Section 05 calibration table correctly marks all new rows with daggers. |

**Average score**: 2.8

### P1 Blockers

None (the single-run caveat is properly disclosed throughout).

### P2 Items

1. **Binary/crate names throughout**: `redist` ensemble platform (section 01), `redist` data pipeline (section 03), `redist fetch` (section 03), `redist` CLI (section 06 line 246), `redist` algorithm portfolio (section 08), `redist-core`, `redist-ilp`, `redist-ensemble`, `redist-cli` (section 08 implementation reference), `redist` platform (section 08 conclusion). All must be updated to `bisect` equivalents. This is the primary revision needed.
2. **Section 01 introduction** references "15 distinct algorithm identities" but then says "six structure algorithms and twelve search algorithms, totalling 15." Six plus twelve is eighteen, not fifteen. The count is wrong — clarify which algorithms are shared or overlap to arrive at 15.
3. The SA runtime claim ("12× slower than METIS for NC") is derived from Table 3.1 (25.3s / 2.1s = 12.0×) — this is correct and appropriately calculated. However, the "10× the structure-layer compute budget for a 50-state sweep" needs a brief derivation (it is not 12× because of state-size variation). Add a footnote.
4. B.25 future work paragraph mentions "spectral bisection based on the Fiedler vector" — per recent work B.25 is the Moving-Knife algorithm, not spectral. Update this reference.
5. Section 02 setup references `--weights geographic` at line 62 (partial context: "and `--weights geographic`) updated to the ApportionRegions composite") — verify whether this uses the correct `--weights-override geographic` flag or the stale `--weights geographic` form.

### P3 Items

- No invented court citations. The `--weights geographic` in section 02 needs verification (P2 item 5 above).

**Verdict**: Minor Revision (avg = 2.8)

---

## Summary Table

| Paper | Title (short) | R1 | R2 | R3 | R4 | R5 | Avg | P1 count | Verdict |
|-------|--------------|----|----|----|----|----|----|----------|---------|
| G.2 | Partisan Outcome Distributions | 3 | 3 | 2 | 3 | 2 | 2.6 | 1 | Minor Revision |
| G.4 | Ensemble Diagnostics (R-hat/ESS) | 3 | 3 | 2 | 3 | 2 | 2.6 | 2 | Minor Revision |
| G.7 | SMC for Redistricting | 3 | 3 | 3 | 3 | 1 | 2.6 | 1 | Minor Revision |
| G.10 | Merge-Split MCMC | 4 | 3 | 3 | 3 | 2 | 3.0 | 0 | **Accept** |
| G.11 | Multi-Scale MCMC | 3 | 3 | 3 | 3 | 2 | 2.8 | 0 | Minor Revision |
| G.14 | Ensemble Algorithm Comparison | 3 | 3 | 2 | 3 | 2 | 2.6 | 1 | Minor Revision |
| G.15 | Comprehensive Comparison (15 algo) | 3 | 3 | 3 | 2 | 3 | 2.8 | 0 | Minor Revision |

---

## Cross-Cutting Issues (All Papers)

### Issue 1: Binary Name Rename (`redist` → `bisect`) — Systemic P2/P3

All papers except G.2 contain occurrences of the old `redist` binary/crate name. The rename was completed in Phase 1. The affected papers and patterns:

| Paper | Occurrences | Worst instance |
|-------|-------------|----------------|
| G.4 | 6+ | Abstract CLI command `redist analyze` |
| G.7 | 6+ | Abstract `redist-smc` crate name; conflicting `bisect` in section 01 body |
| G.10 | 1 | Section 01 `redist-cli` |
| G.11 | 5+ | Throughout including footnote |
| G.14 | 6+ | Throughout including conclusion |
| G.15 | 10+ | Section 08 implementation reference lists |

**Recommendation**: Run a global search-replace on all G-series `.tex` files replacing `redist-cli` with `bisect-cli`, `redist-ensemble` with `bisect-ensemble`, `redist-core` with `bisect-core`, `redist-smc` with `bisect-smc`, `redist-ilp` with `bisect-ilp`, `redist analyze` with `bisect analyze`, `redist state` with `bisect state`, and `redist fetch` with `bisect fetch`. The R package `redist` (external) should retain its original name.

### Issue 2: `--weights geographic` vs `--weights-override geographic` — P1 in G.14

G.14 section 05 uses `--weights geographic` twice. This is a P1 blocker because G.14 is the primary practitioner reference for algorithm selection, and the wrong flag will cause CLI errors for users following the examples.

G.15 section 02 may also contain this error (partial context suggests it). Verify and correct.

### Issue 3: Single-Run Results Disclosure

All papers appropriately disclose single-run status either in the abstract (G.14, G.15) or in the caveat sections. The primary concern is that decision-tree sections in G.14 and G.15 drop the caveat when presenting rankings. Both papers should add a brief reminder at the start of their decision-framework sections.

### Issue 4: G.2 Missing Section File

G.2's `sections/04-proportionality-corridor.tex` is referenced in `main.tex` but not found in the directory listing. This is the most serious structural issue in the batch — the section referenced in the abstract may be absent or empty.

---

## Recommended Revision Priority

1. **G.4**: Fix P1.1 (binary name in abstract/implementation) + P1.2 (ESS minimum steps claim). Then update the abstract-to-statutory contradiction.
2. **G.2**: Locate or write `sections/04-proportionality-corridor.tex`. Verify `\callais` macro usage.
3. **G.14**: Fix `--weights geographic` to `--weights-override geographic` (P1). Then binary names.
4. **G.7**: Fix binary name inconsistency (abstract vs section 01). Complete Proposition 3 verification.
5. **G.11**: Fix abstract ACF range conflation. Binary names.
6. **G.15**: Binary names throughout. Fix the algorithm count (6+12≠15). Update B.25 future work paragraph.
7. **G.10**: Accepted — minor binary name fix only.
