# Track G — Ensemble Methods and Comparison: Panel Revision Plan

**Track**: G-ensemble
**Based on**: REVIEW_PANEL.md (2026-05-07)
**Papers**: 15 (G.0–G.14)
**Revision cycles estimated**: 3 (ensemble-baseline); 1 (G.12 use-case); 0 (G.6, G.8–G.11, G.13–G.14 accepted)

---

## Revision Priority Order

**Tier 1 — Blockers** (must complete before any sub-track proceeds to submission):
G.1 (ESS correction, TX/CA data), G.4 (ESS table, statutory formula, G.1 certification section)

**Tier 2 — Foundation**:
G.7 Phase 2 (SMC empirics, SmcPercentile), G.0 (R-hat threshold cascade, NC PP cascade), G.5 (abstract bound correction)

**Tier 3 — Cross-track and completeness**:
G.2 (cascade from G.1), G.3 (metric framing, cascade), G.12 (use-case clarification), G.14 (three targeted additions)

**Tier 4 — Minor revisions to accepted papers**:
G.6 (Phase 2 multi-run stats), G.8 (TX run optional), G.9 (unbiasedness precision), G.10 (cost claim correction), G.11 (speedup calculation correction), G.13 (D.5 cross-reference)

---

## Tier 1 — Blockers

### G.1: GerryChain Congressional Comparison — R3 Required

**Responsible**: Author + P4 (GerryChain methodology) + P7 (election law)
**Estimated scope**: 2–3 weeks

#### Action G.1-A: ESS-based uncertainty correction (D3 — not resolved in R2)
- Remove π-hat ± 1.645 × sqrt(π-hat(1−π-hat)/N) wherever N is nominal chain length
- Replace with ESS-corrected formula using ESS from G.4 Table 1 (corrected values after G.4-B)
- For NC (Herschlag chain, N=24,518): ESS ≈ 1,703 (corrected). All NC uncertainty intervals are 3.6× wider than currently reported
- Remove all "negligible sampling error" statements. Replace with: "Sampling uncertainty is non-negligible: ESS ≈ 1,703 for NC (Herschlag 2020 single chain), giving a 90% CI of approximately ±3.5 percentile points for the Democratic seat percentile claim."
- This directly addresses Duchin R2 condition for score to rise from 2.6/4 to 3.0/4

#### Action G.1-B: NC baseline mismatch (H1 — not addressed in R2)
- Add paragraph to Section 4: note explicitly that Herschlag 2020 uses 2016 presidential returns while AR plan is evaluated under 2020 presidential returns
- Add robustness check: report Democratic seat percentile for AR plan under 2016 baseline

#### Action G.1-C: VRA analysis for MM districts (H3 — not addressed in R2)
- Add Section 8.1: "Minority Representation and VRA Compliance"
- Report MM district percentile for NC (71st) and GA (62nd)
- Note that NC at 71st percentile means slightly more concentrated minority districts than average — potential Section 2 packing concern
- Cross-reference G.13 and D.0–D.3

#### Action G.1-D: Allen v. Milligan 2023 update (H4 — not addressed in R2)
- Add one sentence to Section 8 acknowledging the 2023 Allen v. Milligan decision

#### Action G.1-E: TX/CA ensemble data completion
- Complete GerryChain ReCom runs for TX (k=38) and CA (k=52)
- Replace all "(pending)" placeholders and "< 5th percentile (lit)" placeholders with actual results

#### Action G.1-F: Ensemble size stability analysis
- Run ensembles at 1,000, 5,000, and 10,000 steps for NC and WI
- Report percentile results at each size in a supplementary table

**Blocker status**: G.1-A and G.1-E are blockers for the sub-track.

---

### G.4: Ensemble Diagnostics — R2 Required

**Responsible**: Author + P1 (mixing theory) + P2 (convergence diagnostics)
**Estimated scope**: 1–2 weeks

#### Action G.4-A: Standardise R-hat threshold to 1.1 (Blocking issue B1)
- Change all instances of R-hat threshold 1.05 in G.0 (body text) to 1.1
- Confirm G.4 already uses 1.1 throughout
- Add Vehtari et al. (2021) justification footnote to G.4

#### Action G.4-B: Resolve Hamming reference plan specification (Blocking issue B2)
- Preferred: switch to scalar-statistic autocorrelation. Replace Hamming autocorrelation with ACF(f(π_t)) where f(π_t) = mean Polsby-Popper of plan π_t

#### Action G.4-C: Correct ESS table (High-priority H1)
- Recompute all ESS values using ESS = N(1−ρ₁)/(1+ρ₁)
- For NC: ρ₁ = 0.87 → ESS = 10,000 × 0.13/1.87 = 695. Current table reports 769. Correct to 695
- Apply same check to all state rows
- Corrected NC ESS (695) must be used in G.1-A's uncertainty calculations

#### Action G.4-D: Reconcile statutory minimum formula and table (High-priority H2)
- Determine correct formula. Recommended: n_min(k) = max(10,000, 1,000k) giving TX = 38,000 and CA = 52,000
- The formula and table must agree exactly

#### Action G.4-E: Add Section 5.3 certifying G.1 source ensembles (High-priority H3)
- Add "Certification of G.1 Source Ensembles" as Section 5.3
- Herschlag 2020 (NC, N=24,518, single chain): R-hat cannot be computed. ESS ≈ 695 (corrected). Verdict: ESS certified for 5th-percentile claims; R-hat uncertifiable; insufficient for 1st-percentile tail certification
- DeFord 2021 (WI/GA/PA, N=50,000): if ρ₁ similar to measured values, ESS ≈ 4,000–7,500. Certification conditional on actual ρ₁ values

#### Action G.4-F: Add 99th-percentile ESS minimum (High-priority H4)
- Add to Section 3.3: "For 99th-percentile tail estimates with precision δ=0.01: ESS_min = 1/(4 × 0.01 × 0.99 × 0.01²) ≈ 2,525. For 1st/99th-percentile litigation claims, we recommend chain lengths of 20,000–100,000 steps for the states studied."

---

## Tier 2 — Foundation

### G.7: SMC Redistricting — Phase 2 Required Before R1

**Estimated scope**: 4–8 weeks

#### Action G.7-A: Complete WI and TX empirical results
- Run SMC for WI (k=8) and TX (k=38) at the same compute budget used by G.6/G.8–G.12
- Report EC, PP, partisan stability, and wall-clock runtime for NC, WI, TX

#### Action G.7-B: Implement SmcPercentile compositor
- Implement SeedCompositor::SmcPercentile in redist-cli as described in Phase 2 note

#### Action G.7-C: Validate against R redist::redist_smc()
- Run R redist::redist_smc() for NC (k=14) and compare weighted ensemble distribution to Rust implementation

#### Action G.7-D: Qualify the "only SMC" framing
- Replace "only SMC provides a statistically valid answer without requiring Markov chain mixing" with: "SMC provides an asymptotically exact calibration without mixing assumptions; Forest ReCom (G.9) and Merge-Split (G.10) provide approximate calibration via expected detailed balance — a genuine, if weaker, guarantee."

---

### G.0: Ensemble Methodology Framework — R2 Required

**Prerequisites**: G.1-A (ESS correction) and G.4-A (R-hat standardisation) must be complete first.

- G.0-A: Update NC PP percentile from "65th–75th" to "68th" following G.1 R2 correction
- G.0-B: Standardise R-hat threshold to 1.1; add Vehtari et al. (2021) justification footnote
- G.0-C: Clarify AR plan uniqueness: uniqueness holds for ConvergenceSweep at T=600 from a specified seed, not for METIS in general
- G.0-D: Add cross-reference to G.14 decision tree in Section 6

---

### G.5: Convergence and Mixing Analysis — R2 Required

- G.5-A: Correct abstract mixing time bound from O(n² log n) to O(n³ log n)
- G.5-B: Add "local mixing vs. global mixing" distinction paragraph to Section 3
- G.5-C: Add cross-reference to G.14 decision tree in Section 6

---

## Tier 3 — Cross-Track and Completeness

### G.2: Partisan Outcome Distributions — R2 Required
*(Prerequisite: G.1-A must be complete first)*

- G.2-A: Cascade ESS uncertainty correction using ESS-corrected uncertainty intervals
- G.2-B: Apply same DeFord 2021 attribution fix used in G.1 R2
- G.2-C: Develop algorithm bakeoff cross-reference (Section 5)

---

### G.3: Compactness Distribution Position — R2 Required

- G.3-A: Open with explicit EC vs. PP metric explanation: G.1 reports EC percentile (0.1–0.2nd); G.3 reports PP percentile (61st–72nd). METIS minimises EC, so both are correct and the distinction matters
- G.3-B: Add skewness statistic to Section 2 for the PP distribution
- G.3-C: Strengthen legal defensibility argument with case citations

---

### G.12: Short-Burst with Calibrated Chains — R1 Preparation Required

- G.12-A: Reframe use case around "ensemble coverage near compact plans" — the contribution is characterising the distribution of plans near the compactness frontier, distinct from pure optimisation (G.6) and from global sampling (G.9/G.10)
- G.12-B: Connect per-burst acceptance rate to G.4's ESS framework
- G.12-C: Add cross-reference to G.14's "compactness + distributional awareness" recommendation

---

### G.14: Practitioner Comparison — Three Targeted Additions

- G.14-A: Add G.7 contingency footnote to SMC recommendation in decision matrix
- G.14-B: Add VraRecom prerequisite note to Q1: VraRecom requires a VRA-compliant initial plan from --structure ratio-optimal-vra
- G.14-C: Handle VRA × large-k intersection: for TX k=38 with 7 protected districts, route to VraRecom with increased step count (≥ 2× Multiscale-recommended)

---

## Tier 4 — Minor Revisions to Accepted Papers

- **G.6-OPT**: Add multi-run statistics (10 seeds on NC, WI, TX); clarify ShortBurst vs. BisectionEnsemble comparison — optional, would raise score from 3.0/4 to ~3.4/4
- **G.8-OPT**: Add TX (k=38) Flip results — optional
- **G.9-A**: Clarify that Wilson's algorithm samples exact USTs, making the ratio c_fwd/c_rev exact on each draw (not only in expectation)
- **G.10-A**: Verify O(m²) claim for Forest ReCom. If Forest ReCom uses Wilson's UST, cost is O(m) expected. Correct the cost comparison
- **G.11-A**: Correct speedup claim to use merger subgraph sizes: TX merger ≈ 277 tracts vs. 10 county nodes ≈ 28× speedup (from the correct quantity)
- **G.13-A**: Add D.5 cross-reference in Section 2.1: the protected_districts list is populated from a Gingles bloc-voting analysis as described in D.5

---

## Submission Sequencing

**Ready for submission now**: G.6, G.8, G.9, G.10, G.11, G.13 (accepted, Tier 4 minor revisions only); G.14 (accepted, three targeted additions then submit)

**Ready after Tier 1 completion (4–8 weeks)**: G.1 (after G.1-A through G.1-F), G.4 (after G.4-A through G.4-F)

**Ready after Tier 2 completion (8–16 weeks)**: G.0 (requires G.1 and G.4 first), G.5

**Ready after Tier 3 completion (12–20 weeks)**: G.2, G.3 (require G.1 first)

**Requires Phase 2 implementation (16–24 weeks)**: G.7 (Phase 2), G.12 (use-case clarification first)

---

## Summary Checklist

| Paper | Tier | Actions | Prerequisite | Estimate |
|-------|------|---------|--------------|----------|
| G.1 | 1 | G.1-A, B, C, D, E, F | — | 2–3 weeks |
| G.4 | 1 | G.4-A, B, C, D, E, F | — | 1–2 weeks |
| G.7 | 2 | G.7-A, B, C, D | — | 4–8 weeks |
| G.0 | 2 | G.0-A, B, C, D | G.1, G.4 | 1 week |
| G.5 | 2 | G.5-A, B, C | — | 2–3 days |
| G.2 | 3 | G.2-A, B, C | G.1 | 1 week |
| G.3 | 3 | G.3-A, B, C | G.1 | 1 week |
| G.12 | 3 | G.12-A, B, C | — | 1–2 weeks |
| G.14 | 3 | G.14-A, B, C | — | 3–5 days |
| G.6 | 4 | G.6-OPT | — | Optional |
| G.9 | 4 | G.9-A | — | 1 day |
| G.10 | 4 | G.10-A | — | 1–2 days |
| G.11 | 4 | G.11-A | — | 1 day |
| G.13 | 4 | G.13-A | D.5 stable | 1 day |
