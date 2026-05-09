# Review Synthesis — H.3: Resolution-Aware Redistricting
**Round**: 1
**Date**: 2026-05-09
**Reviewers**: Karypis, Duchin, Liang, Rodden, Stephanopoulos
**Scores**: 2/4, 3/4, 2/4, 2/4, 2/4
**Mean**: 2.2/4 — **Major Revision Required**

---

## Program Context

H.3 completes the H-track by making geographic resolution a first-class pipeline parameter. H.0 (PercentileSweep) and H.1 (BisectionEnsemble) focus on the search dimension; H.2 (redist-ensemble) addresses throughput; H.3 addresses spatial granularity. The track arc requires H.3 to be credible on both the algorithmic side (GEOID derivation, multi-scale Markov chains) and the legal side (VRA implications, litigation-admissibility of resolution choices).

---

## Consensus Strengths

1. **GEOID prefix insight is correct and clean.** All reviewers note that formalising the Census FIPS hierarchy into `derive_partition` is the paper's strongest contribution. Theorem 1 is well-structured.
2. **Manifest audit extension is practically valuable.** Recording `plan_resolution`, `n_units`, and `fine_to_coarse_formula` enables independent verification; Liang and Stephanopoulos both flag this as the most immediately deployable contribution.
3. **Option B's data-free property is a genuine engineering virtue.** Karypis and Duchin accept this as correct; the GEOID derivation reduces operational dependencies for practitioners.
4. **Theorem 2 (county adjacency criterion) is correctly stated** even if its proof has a gap (see K-P1-B below).

---

## Blocking Issues (P1)

### P1-A — 27% autocorrelation reduction: headline claim not supported
**Reviewers**: Duchin, Liang (both P1), Karypis (P2 — noted as "requires stronger validation")

The paper's headline empirical result — "approximately 27% reduction in lag-100 autocorrelation" — is footnoted as "Estimated value from a single 2000-step run, seed $s=42$." With 8 parallel chains at 2000 total steps, each chain has only 250 steps; lag-100 autocorrelation from 250 steps has extremely high variance and cannot be reliably estimated. This figure cannot appear in the abstract or the main claim without multi-run support.

**Required action**: Either (a) extend to ≥10,000 steps per chain, report mean ± SD across chains, and provide a 95% CI on the lag-100 reduction; or (b) move the 27% figure to a preliminary observation in the empirical section, remove it from the abstract, and replace the abstract with "Option B reduces lag-100 autocorrelation (preliminary observation; full empirical validation in Phase 2)."

### P1-B — Option B stationary distribution: claimed vs. actual
**Reviewer**: Duchin (P1)

Section 3.3 states that the coarse-level accept/reject is "the same approximation as standard ReCom." The paper does not state whether Option B samples the same distribution as single-scale tract ReCom. If the distributions differ, practitioners choosing Option B for litigation need to know the distributional gap before using it.

**Required action**: Add a paragraph in Section 3 stating explicitly whether (i) Option B is claimed to sample the same stationary distribution as single-scale tract ReCom, (ii) the distributions are known to differ (state how), or (iii) the question is open. If (iii), state it as a conjecture and recommend empirical comparison before using Option B in litigation.

### P1-C — GEOID prefix: year-invariance not established
**Reviewer**: Karypis (P1)

The paper claims the GEOID prefix property holds as "an invariant of the Census Bureau FIPS encoding" without restricting to within-year use. Tract GEOIDs can change between census years due to splits/merges; cross-year GEOID derivations may fail.

**Required action**: Restrict the claim to within-year GEOID derivation (same census year for tract and county files), or cite the formal Census specification guaranteeing year-invariant prefix structure. Add this as a precondition in Definition 1.

### P1-D — Theorem 2 forward direction: geometric gap
**Reviewer**: Karypis (P1)

The proof of Theorem 2 relies on "any county boundary segment is covered by a sequence of tract boundary segments" without proof or citation. This is geometrically plausible given TIGER's tiling property but is not stated as an axiom or cited.

**Required action**: Add a one-sentence citation to TIGER/Line documentation stating that tracts tile their county without gaps or overlaps, or prove the tiling property from the TIGER shapefile specification.

### P1-E — Partisan neutrality of resolution choice
**Reviewer**: Rodden (P1)

The paper does not address whether county-level coarsening can introduce partisan bias in polarised states (e.g., a state where counties are nearly monolithically partisan). An actor choosing Option B in such a state may be making a choice with systematic partisan effects not visible in compactness metrics.

**Required action**: Add a paragraph in Section 5 (or the legal section 5b) stating that: (i) resolution choice should be disclosed and justified in litigation; (ii) Option B in states with heavily polarised county structure may not be partisan-neutral; and (iii) partisan outcome sensitivity should be verified before choosing Option B.

### P1-F — Partisan outcomes: no comparison between resolution options
**Reviewer**: Rodden (P1)

The empirical section measures autocorrelation but not partisan-outcome distributions. Without a comparison of D-seat share between Option B and single-scale tract ReCom on the TX example, the paper cannot claim resolution choice is partisan-neutral.

**Required action**: Add to the TX empirical section: mean D-seat share and partisan efficiency gap for (a) single-scale tract ReCom and (b) Option B multi-scale. Report whether the difference is statistically distinguishable at the available sample size.

### P1-G — VRA boundary precision
**Reviewer**: Stephanopoulos (P1)

Section 5b notes that BG-level plans affect VRA minority VAP calculations but does not quantify the boundary precision difference. For near-threshold districts, the choice of resolution can be determinative.

**Required action**: Provide an order-of-magnitude estimate of boundary placement precision at BG vs. tract level (e.g., "the BG boundary resolution of ~100m vs. tract boundary resolution of ~500m means district boundaries in urban minority concentrations can differ by [X] to [Y] metres"). Discuss when this precision difference is likely to change whether a district satisfies the Gingles compactness prong.

### P1-H — Resolution standard for litigation
**Reviewer**: Stephanopoulos (P1)

The paper does not recommend a selection criterion when multiple resolution levels are plausible. Courts need legal guidance.

**Required action**: Add one recommendation: "When multiple resolution levels are technically feasible, practitioners should select the finest level at which population equality is achievable within the statutory tolerance, and document the selection before analysis begins. If the enacted plan was drawn at a specific resolution, matching that resolution is the legally defensible default."

---

## P2 Items (Important but not blocking)

| ID | Issue | Reviewer |
|---|---|---|
| P2-A | Report variance across chains for ρ₁₀₀ in Table 1 (SD or 95% CI) | Liang |
| P2-B | Reproducibility check: run same config twice, verify manifests identical | Liang |
| P2-C | Report measured (not estimated) overhead for `build_county_coarsening` with hardware spec | Liang |
| P2-D | Confirm GEOID field is guaranteed in adjacency files; state as precondition | Karypis |
| P2-E | Complexity: state O(|E_T| log |E_T|) worst case and O(|E_T|) expected separately | Karypis |
| P2-F | State formal conditions on county population equality for 3× coarse tolerance | Duchin |
| P2-G | Strengthen county-preservation warning: move before Option B is invoked, not after | Rodden |
| P2-H | Add pre-registration recommendation: resolution fixed and disclosed before analysis | Rodden |
| P2-I | Address adversarial context: what if opposing experts use different resolutions? | Stephanopoulos |
| P2-J | Address manifest/litigation-hold when resolution changes mid-analysis | Stephanopoulos |

---

## Score Trajectory to Accept

Karypis (2→3/4): Addressing K-P1-A, K-P1-B, K-P1-C (complexity), and P2-D/E gets to 3/4.
Duchin (3→3.5/4): P1-B (stationary distribution) is the binding constraint; P1-A already addressed via Duchin's route.
Liang (2→3/4): P1-A + P2-A (variance reporting) are binding. Manifest reproducibility (P2-B) is a +0.5.
Rodden (2→3/4): P1-E/F (partisan neutrality and outcome comparison) are binding.
Stephanopoulos (2→3/4): P1-G/H (VRA precision and resolution standard) are binding.

**Estimated Round 2 score if all P1 items resolved**: 3.0/4 (conditional accept).

---

## Revision Priority

1. P1-A (autocorrelation claim) — requires either extended runs or downgrade to preliminary
2. P1-F (partisan outcome comparison TX) — short empirical extension
3. P1-B (stationary distribution statement)
4. P1-E (partisan neutrality disclosure)
5. P1-C, P1-D (GEOID year-invariance, Theorem 2 gap) — prose-only fixes
6. P1-G, P1-H (VRA precision, resolution standard) — prose-only fixes
7. P2 items — batch
