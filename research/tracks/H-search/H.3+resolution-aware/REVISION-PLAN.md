# Revision Plan — H.3: Resolution-Aware Redistricting
**Round 1 → Round 2**
**Source**: SYNTHESIS.md (2026-05-09)
**Target score**: ≥3.0/4 (conditional accept)

## Score Summary

| Reviewer | R1 Score | Verdict |
|---|---|---|
| Karypis | 2/4 | Major Revision |
| Duchin | 3/4 | Minor Revision |
| Liang | 2/4 | Major Revision |
| Rodden | 2/4 | Major Revision |
| Stephanopoulos | 2/4 | Major Revision |
| **Mean** | **2.2/4** | **Major Revision** |

---

## P1 Items (All must be addressed before resubmission)

### P1-A — 27% autocorrelation claim
**Reviewers**: Duchin, Liang, Karypis
**Action**: Extend TX Option B experiment to ≥10,000 steps per chain (8 parallel chains). Report mean lag-100 autocorrelation ± SD across chains. Provide 95% CI on the reduction percentage. If runs are not feasible, remove the 27% figure from the abstract and headline; replace with "preliminary single-run observation; multi-run validation in Phase 2."

### P1-B — Option B stationary distribution
**Reviewer**: Duchin
**Action**: Add paragraph in §3 explicitly stating whether Option B samples the same stationary distribution as single-scale tract ReCom. If open, state as conjecture. The paragraph should also cross-reference the TX partisan comparison (P1-F) as indirect evidence.

### P1-C — GEOID year-invariance
**Reviewer**: Karypis
**Action**: Add precondition to Definition 1: "Assumes within-year GEOID derivation — tract and county files from the same census year." Add footnote citing the Census FIPS encoding specification. Do not claim year-invariance without documentation.

### P1-D — Theorem 2 tiling assumption
**Reviewer**: Karypis
**Action**: Add a one-sentence citation or lemma: "By construction of TIGER/Line shapefiles, census tracts within a county tile the county exactly — no gaps, no overlaps (Census Bureau, 2020 TIGER Technical Documentation §3.2)." This makes the forward direction of Theorem 2 complete.

### P1-E — Partisan neutrality of resolution choice
**Reviewer**: Rodden
**Action**: Add §5.3 "Resolution Choice and Partisan Neutrality" (or integrate into §5b). State: (i) resolution choice must be disclosed in litigation; (ii) Option B county coarsening in states with polarised county structures may not be partisan-neutral; (iii) practitioners must verify partisan sensitivity before choosing Option B in contested redistricting.

### P1-F — Partisan outcome comparison
**Reviewer**: Rodden
**Action**: Add to §5 Table 1 or an adjacent table: mean D-seat share and efficiency gap for (a) single-scale tract ReCom and (b) Option B multi-scale, both on TX ($k=38$). Derived from the same ensemble runs used for the autocorrelation comparison. Report whether the distributions are statistically distinguishable.

### P1-G — VRA boundary precision
**Reviewer**: Stephanopoulos
**Action**: In §5b, add: "Block-group boundary resolution is approximately 100m in urbanised areas vs. ~500m for tracts (TIGER 2020 tolerances). For near-threshold minority VAP districts, this precision difference can shift estimated VAP by 0.1–0.5 percentage points at district edges, which is legally significant at the 50% threshold. Practitioners in covered jurisdictions running near-threshold minority districts should use block-group or block resolution."

### P1-H — Resolution selection standard
**Reviewer**: Stephanopoulos
**Action**: Add §5.4 "Selecting Resolution for Legal Proceedings" (or integrate into §5b conclusion): "When multiple resolution levels are technically feasible, select the finest level at which population equality is achievable within the statutory tolerance. If the enacted plan was drawn at a specific resolution, matching that resolution is the legally defensible default. Document the selection before analysis begins and disclose it in any expert report."

---

## P2 Items

| ID | Action | Section |
|---|---|---|
| P2-A | Report SD or 95% CI across chains for ρ₁₀₀ in Table 1 | §5 |
| P2-B | Run same TX config twice from same seed; verify manifests identical; report as §5.2 reproducibility check | §5.2 |
| P2-C | Replace overhead estimate with measured result: report hardware, measured wall time for `build_county_coarsening`, compute % overhead | §5.1 |
| P2-D | Confirm GEOID field is guaranteed present in `{state}_adjacency_{year}.pkl`; state as precondition in §2 | §2 |
| P2-E | Complexity theorem: "O(\|E_T\| log \|E_T\|) worst case via sort-and-unique; O(\|E_T\|) expected with hash-set deduplication" | §2 |
| P2-F | Add corollary: "The 3× coarse tolerance heuristic provides balance recovery when county populations are within 2× of ideal district population; for states with highly variable county populations, the factor should be increased" | §3 |
| P2-G | Move county-preservation warning to §3.3 preamble (before Option B is selected), not §5b | §3.3 |
| P2-H | Add §5.4 or §5b bullet: "Resolution must be fixed and documented before ensemble analysis begins; any change must be disclosed in expert reports" | §5b |
| P2-I | Add §5b paragraph: "When opposing experts use different resolutions, courts should request disclosure of selection criteria; the finest level supporting population equality is the legally defensible default" | §5b |
| P2-J | Add §5b paragraph: "If resolution changes mid-analysis, both manifests should be preserved; the final plan manifest controls for audit purposes; resolution changes must be documented in the audit trail" | §5b |

---

## Round 2 Path to Conditional Accept

**Critical path**: P1-A → P1-F (both require the extended TX empirical run).
Run 8 chains × 10,000 steps for TX at both single-scale and Option B. This produces:
- Reliable lag-100 autocorrelation with CI (P1-A, P2-A)
- Partisan outcome comparison (P1-F)

All other P1 items are prose-only. Batch fix P1-B through P1-H in one editing pass.
Batch fix P2 items in a second editing pass.

**Estimated revision time**: 1–2 weeks (dominated by the extended TX empirical run).
