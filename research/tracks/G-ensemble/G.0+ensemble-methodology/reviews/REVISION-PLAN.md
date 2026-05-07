# Revision Plan — G.0: Ensemble Methodology
**Round 1 → Round 2**

## Scores

| Reviewer | Score | Recommendation |
|---|---|---|
| Karypis | 3/4 | Accept minor revisions |
| Rodden | 3/4 | Accept moderate revisions |
| Duchin | 2/4 | Major revision |
| Stephanopoulos | 3/4 | Accept minor-to-moderate revisions |
| Liang | 3/4 | Accept revisions |
| **Mean** | **2.8/4** | |

## Duchin-Blocking Issues (must resolve before G.1–G.3 proceed)

### B1. ReCom stationary distribution claim (Section 2.2)
**Issue**: Paper states ReCom samples from "stationary distribution of valid plans" and Table 1 lists "Stationary dist. known: Implicit." The correct characterisation is "unknown/approximately uniform."
**Fix**: Change Table 1 entry for ReCom from "Implicit" to "Approximate (spanning-tree proportional)". Add a paragraph in Section 2.2 explicitly noting that standard ReCom has an unknown stationary distribution and that the percentile interpretation is therefore approximate. Reference DeFord 2021 Section 3 for the formal statement.

### B2. Rhat threshold inconsistency (Sections 4.1 and 4 table)
**Issue**: Section 4.1 body states $\hat{R} < 1.05$; the table at end of Section 4 states $\hat{R} < 1.1$. G.4 uses 1.1 throughout.
**Fix**: Standardise to $\hat{R} < 1.1$ everywhere in G.0. Add a footnote noting that Vehtari 2021 recommends 1.01 for continuous parameters but that 1.1 is the appropriate threshold for redistricting's discrete integer outcomes. Update the G.0 table to match G.4.

### B3. CS-to-ensemble bridge section (Section 5)
**Issue**: Paper claims ESS and the CS tail T=600 are "parallel measures of robustness to additional sampling." This is a false analogy: ESS measures estimator variance over draws from a distribution; T=600 measures non-improving seeds in a deterministic search.
**Fix**: Rewrite Section 5 to make the following distinction explicit: ESS and T=600 are INDEPENDENT certificates for DIFFERENT goals. Remove the phrase "both T and ESS measure the robustness of a result to additional sampling." Replace with: "ESS certifies that the ensemble distribution estimate is stable; T=600 certifies that the deterministic search has found the best available plan. These certificates are not comparable numerically but together provide complete coverage of the two-method framework."

## High-Priority Revisions

### H1. Percentile notation conflict
**Issue**: $\pi_f(P^*)$ conflicts with standard MCMC notation ($\pi$ = stationary distribution).
**Fix**: Change notation to $q_f(P^*)$ throughout G.0 and cascade to G.1–G.3.

### H2. Rhat citation (Section 4)
**Issue**: Vehtari 2021 formula cited to Gelman 1992.
**Fix**: Add Vehtari et al. (2021) "Rank-normalization, folding, and localization" as primary citation for the formula in Section 4.1. Retain Gelman 1992 as the original reference.

### H3. ESS threshold calibration (Section 4.2)
**Issue**: ESS minimum stated as 400; Liang shows correct calibration gives 526 for the stated precision. G.4 uses 500.
**Fix**: Change ESS threshold to 500 throughout G.0, consistent with G.4. Add the derivation from $1/(4\alpha(1-\alpha)\delta^2)$.

### H4. Hamming reference plan (Section 4.3)
**Issue**: Reference plan $P_{\rm ref}$ in Eq. 4 is underspecified.
**Fix**: State that G.0 uses $P_{\rm ref} = P_0$ (initial plan) and that G.4 Section 4 specifies this in detail. Add a cross-reference.

## Moderate-Priority Revisions

### M1. Table 1 compactness weight column
**Issue**: All ensemble methods listed as "Compactness weight: None," which is incorrect for Metropolized Forest ReCom.
**Fix**: Add a footnote to Table 1: "Standard implementations; Metropolized Forest ReCom (Autry 2021) can incorporate compactness weighting at higher computational cost."

### M2. Rodden effect in framework
**Issue**: Paper does not acknowledge that ensemble-median outcomes are not normatively neutral in sorted states.
**Fix**: Add one paragraph in Section 6.1 noting that "the ensemble median represents what geography determines, not what fairness requires. In geographically sorted states, the ensemble median itself reflects Republican over-representation relative to vote share. The AR plan's near-median position means it replicates geographic inevitability, not partisan neutrality." Cross-reference G.2 Section 4 for the proportionality corridor analysis.

### M3. Legal accuracy (Section 6.1)
**Issue**: Paper implies courts have broadly relied on ensemble evidence; post-Rucho federal courts cannot.
**Fix**: Add a sentence: "Federal courts may not use ensemble evidence for partisan gerrymandering claims after Rucho. State constitutional claims in PA (LWV v. PA), NC (Harper v. Hall), and other states remain viable routes."

### M4. "Exactly one AR plan" claim precision
**Issue**: METIS finds a local optimum, not a global one. "Exactly one AR plan" requires precision.
**Fix**: Add: "There is exactly one AR plan produced by the CS algorithm at $T = 600$ from the SHA-256-derived seed sequence and the specified census release. This plan is a local optimum under METIS, reached deterministically from the canonical seed."

## Low-Priority Revisions

### L1. \est marker definition
**Issue**: The \est marker is used in G.1 but not defined in G.0 (the framework paper).
**Fix**: Add a definition box at the end of Section 3: "\est: direct ensemble comparison. \lit: literature-bound estimate. All percentile claims not marked \est carry wider uncertainty."

## Propagation Notes

- B1 propagates to G.1, G.2, G.3: all ensemble percentage claims should be understood as approximate given unknown ReCom stationary distribution.
- B2 propagates to G.4: G.4 should confirm it uses 1.1.
- H1 propagates to G.1, G.2, G.3: notation change from $\pi_f$ to $q_f$.
- H3 propagates to G.4: confirm ESS threshold is 500, not 400.
