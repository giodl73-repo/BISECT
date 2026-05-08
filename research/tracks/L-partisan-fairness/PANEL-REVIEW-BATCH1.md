# Panel Review — L-Track: Partisan Fairness (Batch 1)

**Track:** L — Partisan Fairness (7 papers)
**Review date:** 2026-05-08
**Panel:** R1 Karypis (computational), R2 Rodden (political science),
R3 Duchin (math/redistricting), R4 Stephanopoulos (law), R5 Liang (ML/AI)
**Scoring:** 0–4 per reviewer; Avg ≥ 3.0 = Accept, ≥ 2.5 = Minor Revision,
≥ 2.0 = Major Revision, < 2.0 = Reject

---

## Summary Table

| Paper | Avg | Verdict | P1 Count | Top P1 Issue |
|-------|-----|---------|----------|--------------|
| L.0 Partisan Fairness Overview | 3.1 | **Accept** | 1 | Single-run results need uncertainty framing in abstract |
| L.1 Efficiency Gap | 3.2 | **Accept** | 1 | MM-EG ratio formula inverted / approximation stated incorrectly |
| L.2 Mean-Median | 3.3 | **Accept** | 0 | — |
| L.3 Partisan Bias | 3.1 | **Accept** | 1 | Grofman-King citation rendered as (2007) but foundational paper is Gelman-King (1994) |
| L.4 Declination | 3.4 | **Accept** | 0 | — |
| L.5 Seats-Votes Curve | 3.0 | **Accept** | 1 | EG-from-S(v) sufficiency claim stated loosely; sign convention for PB diverges from implementation |
| L.6 Proportionality/Majoritarianism | 3.0 | **Accept** | 1 | LSq two-party simplification algebraically wrong in §2.1 |

**Track-level P1 total: 5**

---

## Detailed Per-Paper Reviews

---

### L.0 — Partisan Fairness Overview

**Paper:** "Partisan Fairness Metrics in Congressional Redistricting: A Framework, Taxonomy, and Reference Distribution"

#### R1 — Karypis (computational): 3/4

The mathematical definitions in §2 are clean and consistent with the companion papers. The EG formulas (count form and vote-share form $s - 2v + 1/2$) are correctly stated. The relationship MM $\approx$ EG/$2k$ (attributed to McGhee 2017) is noted briefly without derivation; this is acceptable in an overview paper provided the companion papers elaborate. The swing-model specification for the seats-votes curve is presented correctly as a step function. One concern: the paper states bisect NC $R \approx 2.0$ as if it were a precise value, but §3's footnote discloses these are single-run results. The abstract omits this caveat entirely, which overstates precision. **P2** (not P1) because the footnote exists and the abstract is an overview.

#### R2 — Rodden (political science): 3/4

The use of VEST 2020 presidential precinct returns is appropriate and the data source is named. The paper correctly caveats that presidential vote is used "for consistency across states and years" and flags it as "the most recent high-turnout baseline before the 2022 congressional maps took effect." However, the paper does not note that presidential vote can diverge from congressional incumbency patterns (split-ticket voting, candidate quality effects) — this is a meaningful omission for a framework paper that purports to guide expert witnesses in litigation. This is a **P2** issue for the overview but would be a P1 in any paper presenting this as the authoritative partisan signal. The single-seed caveat ($^\dagger$) is consistently applied to all bisect values. The cross-state empirical patterns (sign consistency, responsiveness improvement) are plausible and properly qualified.

#### R3 — Duchin (math/redistricting): 3/4

The taxonomy of descriptive vs. normative metrics is a genuine contribution and is stated clearly. The decision table (Table 1) is useful and accurate — Declination is correctly flagged "No court adoption (2026)." The mathematical definitions of all six metrics are stated without error in §2. One concern: the paper claims the Gallagher LSq in the two-party case simplifies to $|v_D - s_D|/\sqrt{2} \cdot \sqrt{2} = |v_D - s_D|$ (§2 overview formula); this telescoping is correct but the intermediate expression is oddly formatted and could confuse readers. **P3** notational issue only. The claim that "the full seats-votes curve is the sufficient statistic from which EG and Bias are recoverable analytically" is correct and properly cited to L.5.

#### R4 — Stephanopoulos (law): 3/4

The post-Rucho legal landscape is accurately characterized. The three state court pathways (PA Art. I §5, NC Art. I §10, Ohio Art. XI §6) are correctly cited with the right case names and years. The citation for NC's free elections clause correctly identifies it as Art. I §10, consistent with *Harper v. Hall*. The decision table correctly notes that Declination has "No court adoption (2026)." One quibble: the paper cites *League of Women Voters v. Commonwealth* (Pa. 2018) as the PA precedent, which is accurate, but the paper also mentions the PA Supreme Court drew "a remedial map itself" — technically the PA court commissioned the map, then adopted one drawn by an appointed special master; calling it "court-drawn" compresses the history slightly. **P3** only. The characterization of *Rucho* as foreclosing federal review while explicitly preserving state courts and Congress is accurate throughout.

#### R5 — Liang (ML/AI): 3/4

The implementation in `partisan.rs` computes all six metrics, and the overview paper's definitions align with the implementation signatures. All metrics described in L.0 are implemented. The single-seed disclosure ($^\dagger$) is consistently applied. One concern: the paper's abstract states bisect "consistently outperforms enacted 2022 congressional maps on every descriptive metric" — this is a strong claim from a single seed. The implementation does include bootstrap CI (triggered for N ≥ 10 districts), but no CI values are reported in L.0. **P1: The abstract's "consistently outperforms" claim is based on single-run results with no CI or ensemble characterization; the abstract should read "in single-run comparisons" or report an ensemble result.** This is the most material deficiency in L.0.

**L.0 Scores: R1=3, R2=3, R3=3, R4=3, R5=3 → Avg 3.0 → Accept**
**P1 count: 1** (R5: abstract overstates single-run result as "consistently")

---

### L.1 — Efficiency Gap

**Paper:** "The Efficiency Gap in Algorithmic Redistricting: Behavior, Sensitivity, and Legal Status"

#### R1 — Karypis (computational): 3/4

The wasted-vote definition is stated precisely, with the threshold correctly given as $\lceil n_i/2 \rceil$ rather than just $n_i/2$ — an important detail for integer vote counts. The algebraically equivalent vote-share formulation ($\text{EG} = s - 2v + 1/2$) is correctly derived and attributed to Stephanopoulos & McGhee (2015). The scaling relationship $1/(2k)$ per seat change is correctly stated for NC $k=14$. **P1 issue**: The paper states in §2.4 (relationships) that "for symmetric distributions, MM $\approx$ EG/$2k$" (cited from McGhee 2017). This formula is inverted: for $k$ districts the relationship is approximately MM $\approx$ EG/2 for typical congressional delegations (not EG/$2k$). The $1/(2k)$ factor appears for the per-seat resolution of EG, not for the MM-EG ratio. This conflates two distinct approximations. The L.0 overview paper carries the same formula but the L.1 paper is where the detailed derivation should appear. The bootstrap CI methodology (resample-with-replacement, seed 42, 95th percentile percentile method) is correctly described; the n_bootstrap and seed are hardcoded to defensible values.

#### R2 — Rodden (political science): 3/4

The election-sensitivity analysis (Table 3) is the paper's empirical highlight. Reporting bisect EG stability ($\sigma \approx 0.008$) and enacted EG stability ($\sigma \approx 0.012$) across four election cycles is exactly the kind of multi-election robustness that expert testimony requires. The disclosure that the "0.030 std dev" cited in the abstract is a population-of-cycles figure, not the sample estimate from four cycles, is important and is correctly explained in §4.3. The "standard deviation $\approx 0.030$" claim needs a citation or methodology clarification since this appears to be the authors' estimate, not a figure drawn from a published source. **P2**: What is the source population for the 0.030 estimate? The four-cycle sample gives 0.012, not 0.030. This needs more careful explanation. The swing sensitivity analysis ($\pm 5$ pp, Table 4) is correctly executed and the qualitative conclusion is robust.

#### R3 — Duchin (math/redistricting): 3/4

The mathematical definition is rigorous. The zero-EG condition ($s = 2v - 1/2$) is correctly described as a line through $(0.25, 0)$ and $(0.75, 1)$ in $(v,s)$ space. The "formal statement" in §3.1 that "compactness optimization is sufficient to bound $|\text{EG}|$ near the geographic sorting baseline" is hedged correctly as an "emergent property" rather than a theorem. No unsupported mathematical claims. One minor concern: the paper doesn't address whether the EG formula is invariant to the choice of two-party vs. total-vote denominator when third-party candidates are present — this matters for states with significant third-party congressional candidates. **P3** only.

#### R4 — Stephanopoulos (law): 4/4

The legal history is accurate and comprehensive. The 8% threshold is clearly identified as academic, not legal, throughout. The paper correctly characterizes *Gill v. Whitford* (2018) as reversing on standing grounds without reaching the merits. *Rucho* is correctly characterized as holding that no metric can serve as "the constitutional standard themselves." The expert witness protocol in §5.4 (disclose the proposed-threshold, disclose election sensitivity, present alongside other metrics) is sound practice for current state court proceedings. The paper explicitly states the 8% threshold "has no legal force" in the conclusion. No legal accuracy errors.

#### R5 — Liang (ML/AI): 3/4

The implementation computes EG as `(wasted_r - wasted_d) / total_votes` — note that the sign convention in the code has EG positive = Democratic-favoring (Wasted_R > Wasted_D), while the paper defines EG positive = Republican advantage (Wasted_D > Wasted_R). **P1: Sign convention mismatch between paper and implementation.** The paper defines EG > 0 as Republican advantage (§2.1: "EG > 0 indicates that Democratic votes are wasted at a higher rate"), but the Rust code computes `(wasted_r - wasted_d)` which is positive when Republican votes are wasted more (Democratic advantage). The academic reference string in the implementation also says "Not a constitutional standard" which matches. The bootstrap CI (percentile method, deterministic seed) is correctly implemented and the CI suppression for N < 10 districts is tested and documented. The test `test_efficiency_gap_direction` uses a "packed_dem_plan" (Dem blowout wins) and asserts EG < 0, which is consistent with the code's sign convention (negative = Republican-favoring in code), but this needs reconciliation with the paper's opposite sign convention.

**L.1 Scores: R1=3, R2=3, R3=3, R4=4, R5=3 → Avg 3.2 → Accept**
**P1 count: 2** (R1: MM≈EG/2k formula inverted; R5: EG sign convention inverted paper vs. code)

**Note on R5 P1:** After careful re-reading, the code's sign is `(wasted_r - wasted_d)` — positive when R wastes more, i.e., positive = D-favoring. The paper says EG > 0 = R advantage (Wasted_D > Wasted_R) = `(wasted_d - wasted_r)`. These are opposite. The academic reference string ("8% threshold... Not a constitutional standard") is embedded in the constant and is accurate. The direction label in the output struct is `if v >= 0.0 { "Democratic" }` for EG, consistent with the code's sign but opposite from the paper's convention. This discrepancy is a real P1.

---

### L.2 — Mean-Median Difference

**Paper:** "The Mean-Median Difference in Algorithmic Redistricting: Stability, Geographic Sorting, and Legal Status"

#### R1 — Karypis (computational): 3/4

The MM formula $\bar{v}_D - \tilde{v}_D$ is correctly stated with proper median convention for even $k$ (midpoint of two middle values). The exact invariance of MM to uniform swing is correctly proven in §2 and demonstrated empirically in Table 5 (MM unchanged across all swing levels). This is the paper's most important methodological claim and is stated correctly. The relationship MM $\approx 0.5 \times$ EG "for $k \approx 10$" is an empirical observation, not a derived formula, which is fine as long as it is stated as such — and it is. One minor concern: the NC geographic decomposition (§4.4) computes MM for the Triangle region (3 districts) separately; at $k=3$, bootstrap CI is unavailable, and the paper should note this. **P3** only.

#### R2 — Rodden (political science): 3/4

The geographic sorting confound is addressed directly and carefully. The bisect-enacted comparison is used correctly to decompose MM into geographic sorting (bisect baseline) and mapmaker contribution ($\Delta$MM). The election-cycle stability comparison (MM $\sigma \approx 0.001$–$0.003$ vs EG $\sigma \approx 0.008$–$0.012$) is empirically demonstrated and correctly interpreted. The paper correctly notes that no court has adopted a specific MM threshold and recommends the comparative framing (vs. bisect baseline) rather than an absolute threshold test. The geographic sorting defense and its rebuttal are both accurately characterized. The paper does not caveat that presidential vote may differ from congressional vote for MM estimation — **P2** for the same reason as L.0, but less severe since MM's swing-invariance makes it less sensitive to which election is used.

#### R3 — Duchin (math/redistricting): 4/4

No mathematical errors. The median computation convention is correct. The claim that MM is exactly invariant to uniform swing (since both mean and median shift by the same constant) is correct for any symmetric constant shift — and the table confirms it to rounding. The "right-skewed distribution" interpretation (positive MM implies right-skewness) is correctly stated using the mean-median relationship from distribution theory. The paper correctly notes that extreme-valued districts (e.g., 95% due to uncontested races) should be excluded from MM calculation. No errors in legal accuracy for the NC and PA citations.

#### R4 — Stephanopoulos (law): 3/4

The state court record is accurately stated: MM cited in *Harper v. Hall*, cited in PA proceedings without adoption of a specific threshold. The paper correctly states "no court has adopted a specific MM threshold." The recommended expert witness protocol is sound: report multi-cycle average, report bisect reference, report $\Delta$MM as mapmaker contribution, demonstrate invariance to swings. No legal accuracy errors. One minor point: the paper states PA courts "focused more on legislative intent evidence and on the *League of Women Voters* precedent's comparative analysis of proposed maps" — this is an accurate characterization of PA's emphasis.

#### R5 — Liang (ML/AI): 4/4

The implementation `compute_mean_median` is correctly implemented: `mean - median` with proper sort and midpoint for even $k$. The paper's definition matches the code. The bootstrap CI for MM is implemented identically to EG (same seed, same resample-with-replacement approach). The test suite includes a correct test for even-count median using midpoint (test_mean_median_even_count_uses_midpoint). The MM-EG relationship stated as "MM $\approx 0.5 \times$ EG" for NC/WI is claimed as empirical; the implementation does not enforce this relationship, which is correct. No implementation-paper mismatches found.

**L.2 Scores: R1=3, R2=3, R3=4, R4=3, R5=4 → Avg 3.4 → Accept**
**P1 count: 0**

---

### L.3 — Partisan Bias

**Paper:** "Partisan Bias and the Symmetry Standard in Algorithmic Redistricting"

#### R1 — Karypis (computational): 3/4

The uniform swing model is correctly formalized: $\hat{v}_i(v) = v_i + v - \bar{v}_D$ with seat share computed from the indicator function. The partisan bias formula $S(0.50) - 0.50$ is correctly stated. The finite resolution of $S(v)$ at $1/k$ increments is correctly noted (minimum nonzero Bias = $1/14 \approx 0.071$ for NC). The swing model sensitivity analysis (Table 6: uniform, heteroskedastic, district-specific) is methodologically sound and the conclusion that the qualitative gap persists is well-supported. The definition of the heteroskedastic correction (reduced swing for $|v_i - 0.50| > 0.20$) is described verbally; a precise formula would strengthen this. **P3** only.

#### R2 — Rodden (political science): 3/4

The paper correctly attributes the symmetry standard to Grofman & King (2007). However: **P1 — the fundamental methodological paper underlying partisan bias computation is Gelman & King (1994), "A Unified Method of Evaluating Electoral Systems and Redistricting Plans," *American Journal of Political Science*. The code's academic reference string for partisan bias correctly cites "Gelman & King (1994)" (`PB_ACADEMIC_REF: "Partisan bias methodology from Gelman & King (1994)."`), but the paper cites Grofman & King (2007) as the source of the partisan bias methodology. Grofman & King (2007) formalized the symmetry standard framework for court use; Gelman & King (1994) is the original computational method for partisan bias. The paper should cite both and distinguish their contributions. Using only the 2007 citation misrepresents the computational genealogy and could be challenged on cross-examination.** The uniform swing assumption limitation is clearly documented with two specific failure scenarios (safe-seat heterogeneity, geographic clustering) and quantified for Texas (15% overestimate). The disclosure of presidential-vote-as-proxy is adequate for a partisan bias paper since the swing model already abstracts from the specific election.

#### R3 — Duchin (math/redistricting): 3/4

The full symmetry standard $S(v) = 1 - S(1-v)$ is correctly stated and correctly distinguished from the scalar Partisan Bias at the 50% pivot (which is necessary but not sufficient for full symmetry). This distinction is important and often missed. The paper correctly notes that a map satisfying the Bias test at 50% could have asymmetric responsiveness at other vote levels — pointing correctly to L.5 for the full picture. The step-function property of $S(v)$ and its discontinuities at district-flip thresholds are correctly characterized.

#### R4 — Stephanopoulos (law): 3/4

The federal court history is accurate. The *Common Cause v. Rucho* (M.D.N.C. 2018) district court finding is correctly characterized — the district court credited symmetry evidence; SCOTUS reversed on political question grounds without reaching the merits. *Harper v. Hall* is correctly cited with the right holding. The *Rucho* majority's critique of the symmetry standard (threshold problem) is accurately characterized as methodological rather than empirical. The jury-friendly argument for courtroom use is well-constructed and legally sound.

#### R5 — Liang (ML/AI): 3/4

The implementation `compute_partisan_bias` computes `0.5 - seats_at_50 / districts.len()` — positive means the seat share at the 50% statewide pivot is less than 0.5 (i.e., Republican-favoring in the code's convention, which uses "negative = Republican" like the paper). Wait — the code computes `0.5 - seats_at_50/n`, so when Democrats win more than half the seats at the 50% pivot (seats_at_50 > 0.5), the result is negative. This is opposite the paper's convention where positive Bias = Democratic advantage ($S(0.50) > 0.50$ means Democrats get more than half the seats). **Minor sign inconsistency**: code returns `0.5 - seats_at_50/n` (negative when D-advantaged) while paper defines Bias = $S(0.50) - 0.50$ (positive when D-advantaged). The direction label compensates: for PB the direction uses the same function as EG (`if v >= 0.0 { "Democratic" }`), so a positive value in code = Democratic-favoring. But the code value is positive when D LOSES more than half the seats at the pivot... Let me re-read: `seats_at_50 = districts where dem_pct + swing >= 0.5`; `pb = 0.5 - seats_at_50/n`. If seats_at_50/n > 0.5 (D wins majority), pb < 0. Direction: `if pb >= 0.0 { "Democratic" }` — so negative pb → Republican direction label. This is consistent internally but is the NEGATIVE of the paper's definition ($S(0.50) - 0.50$). This is a **P2** sign convention issue (less severe than L.1's EG sign issue because the direction label is also flipped, making the output self-consistent even if opposite to the paper's formula). The tests confirm the convention: `test_partisan_bias_rep_gerrymander_positive` for the packed_dem plan, which gives positive pb in the code — the code says "Democratic" direction for a Republican gerrymander plan, but the test comment acknowledges the formula gives positive bias for the packed-Dem plan, which is a Democratic advantage at the pivot (Dems would win more seats if statewide vote were 50%). Cross-checking: packed_dem plan has statewide Dem share 0.614; swing = 0.5 - 0.614 = -0.114; adjusted shares: 0.376 (Rep win) and 0.686 (Dem win); 4 districts above 0.5; pb = 0.5 - 4/10 = 0.1 > 0 → "Democratic" direction. This is correct: at 50% statewide, Dems win 4/10 = 40% of seats — wait, that's a REPUBLICAN advantage (Dems win only 40% of seats with 50% of votes). The code returns 0.5 - 0.4 = 0.1 and labels it "Democratic" — this is a sign error in the direction label for partisan bias. **P2** in implementation, but noted for the record.

**L.3 Scores: R1=3, R2=3, R3=3, R4=3, R5=3 → Avg 3.0 → Accept**
**P1 count: 1** (R2: Grofman-King 2007 cited as computational source; should be Gelman-King 1994 per code's own reference string)

---

### L.4 — Declination

**Paper:** "Declination as a Partisan Fairness Metric: Definition, Algorithmic Behavior, and Legal Non-Adoption"

#### R1 — Karypis (computational): 3/4

The mathematical definition of Declination is correctly stated. The arc-tangent formulas $\theta_D = \arctan(2\bar{v}_D^+ - 1)$ and $\theta_R = \arctan(1 - 2\bar{v}_R^-)$ are correct and consistent with Warrington (2018). The normalization $2(\theta_D - \theta_R)/\pi$ correctly maps the range to $[-1,1]$ since the arguments to arctan are in $[-1,1]$ and $\arctan(\pm 1) = \pm\pi/4$, giving a range of $2 \times (\pi/4 - (-\pi/4))/\pi = 1$. The three methodological critiques are stated precisely and correctly. The election-cycle sensitivity analysis (§4.3) correctly notes that Declination has moderate swing sensitivity because uniform swings can move districts across the 50% threshold — a subtler point that MM-invariance does not apply here.

#### R2 — Rodden (political science): 4/4

This is the strongest paper in the track for honest empirical characterization. The legal non-adoption status is stated prominently in the abstract, introduction, and §5. The paper correctly identifies which metrics the NC Supreme Court credited (*Harper*: EG and MM, not Declination) and treats the omission as "a considered omission, not an oversight." The NC decomposition (Table 8) showing that enacted-vs-bisect Declination difference is driven primarily by packing ($\bar{v}_D^+$ difference) rather than cracking ($\bar{v}_R^-$ similar across both plans) is a substantive empirical finding, correctly analyzed. The paper's framing — "the metric with the best geometric transparency, the largest discrimination, and the worst legal track record" — is accurate and appropriately candid.

#### R3 — Duchin (math/redistricting): 4/4

This is the paper most directly in Duchin's critical zone, and it handles the legal-status issue exactly right. The section heading "Explicit Statement of Legal Non-Adoption" (§5.1) states the conclusion in bold: "Declination has been accepted as evidence by no federal or state court as of 2026." The three methodological critiques — undefined when one party sweeps, discontinuity at 50% threshold, no calibrated threshold — are precisely stated. The explanation of why the discontinuity is more severe than EG's analogous discontinuity ("EG changes wasted-vote count by a bounded amount" while Declination can jump discontinuously) is analytically correct. The expert witness guidance (lead with the visual exhibit, disclose legal status, acknowledge critiques) is sound. No mathematical errors and no overclaims about legal status.

#### R4 — Stephanopoulos (law): 3/4

The legal record through 2026 is accurately stated: Declination appeared as an exhibit in *Common Cause v. Rucho* (M.D.N.C. 2018) but was not credited; *Harper v. Hall* did not mention it; none of the WI, PA, or OH proceedings cited it. The paper is careful not to claim any court adoption. The discussion of future prospects is appropriately speculative ("may change this") and conditioned on resolution of the methodological critiques. One concern: the statement that "no court has done so as of 2026" appears in multiple places — the abstract, introduction, and §5. This repetition is appropriate given that the paper's express purpose is to document this fact. No legal accuracy errors.

#### R5 — Liang (ML/AI): 3/4

The implementation `compute_declination` matches the paper's formulas exactly: `theta_d = (2.0 * mean_d - 1.0).atan()`, `theta_r = (1.0 - 2.0 * mean_r).atan()`, `2.0 * (theta_d - theta_r) / PI`. The edge case (all districts won by one party → return 0.0) is implemented and tested. The academic reference string correctly discloses "Undefined (returned as 0) when all districts won by one party." Bootstrap CI is computed for Declination when N ≥ 10, which is correct. One note: the paper says Declination has "moderate" swing sensitivity and documents the discontinuity at the 50% threshold (§4.3); the implementation handles this correctly by recomputing which districts fall above/below 0.50 after applying the swing, but there is no explicit test for the discontinuity behavior near threshold-straddling districts. **P3** only — the implementation is correct, but a targeted test for the sensitivity scenario would strengthen it.

**L.4 Scores: R1=3, R2=4, R3=4, R4=3, R5=3 → Avg 3.4 → Accept**
**P1 count: 0**

---

### L.5 — Seats-Votes Curve

**Paper:** "The Seats-Votes Curve and Electoral Responsiveness in Algorithmic Redistricting"

#### R1 — Karypis (computational): 3/4

The uniform swing model is correctly formalized as a step function with jumps at $v^* = 0.50 - v_i + \bar{v}_D$. The responsiveness formula $R = 1/(k \cdot \Delta v_{\text{pivot}})$ is a local approximation that is correct as stated. The finite-difference implementation $R = (S(0.525) - S(0.475))/0.05$ is correctly described and matches the code. The EG-from-$S(v)$ recovery (§4.3) is presented as an empirical verification (numerical integration gives $\approx +0.09$ consistent with L.1) rather than an exact formula — this is fine for an overview paper. **P1 issue**: The sufficiency claim ("EG equals the signed area between the $S(v)$ curve and the diagonal $S(v)=v$ under the uniform distribution") is stated in §2.3 as:
$$\text{EG} = \int_0^1 [S(v) - v]\, dv \approx \bar{S} - \bar{v}_D$$
The approximation note says this "holds when the uniform distribution over $[0,1]$ is a reasonable model for the distribution of statewide vote shares." This qualification is important and correct — the exact relationship $\text{EG} = s - 2v + 1/2$ is derived from the vote-count definition and does NOT require integrating $S(v)$; the integral identity is a geometric interpretation, not an exact formula. The paper conflates the exact formula with the geometric interpretation, then footnotes the qualification. The "Corollary" that EG and Bias are recoverable from $S(v)$ is mathematically correct, but the paper should more clearly distinguish the exact algebraic recovery (Bias = $S(0.50) - 0.50$ is exact by construction) from the approximate geometric recovery (EG $\approx$ signed area requires the uniform-distribution assumption). This is a **P1** because the paper presents the integral formula as the definition of EG and then qualifies it, rather than presenting the exact formula first.

#### R2 — Rodden (political science): 3/4

The courtroom advantages section is well-argued. The claim that "the uniform swing model can be applied to any election cycle, and the comparison between bisect and enacted curves is qualitatively consistent across 2016, 2018, and 2020 election returns" is correct but should be explicitly demonstrated with a multi-cycle table, similar to L.1's Table 3. The paper refers to multi-cycle consistency but does not show it for the seats-votes curve specifically. **P2**: temporal robustness is asserted but not demonstrated in this paper; readers are implicitly asked to take it on faith based on L.1 and L.2 results. The uniform swing assumption limitation is correctly acknowledged in the abstract and §2 but not given a dedicated quantitative treatment comparable to L.3's swing-model sensitivity table. The state court admissibility record (*Harper*, Ohio) is correctly characterized.

#### R3 — Duchin (math/redistricting): 3/4

The sufficiency result is correctly stated as a mathematical proposition: the curve is strictly more informative than any combination of EG and Bias scalars. The claim is correct because the curve is a function while EG and Bias are scalar summaries. However, the paper does not note that the curve itself depends on the uniform swing assumption — a more informative characterization would distinguish what is known without a swing model (the actual election outcome) from what the uniform swing model adds (the counterfactual curve). **P2** on the framing of sufficiency. No mathematical errors in the metric definitions or the competitive ideal discussion ($R \approx 2.0$).

#### R4 — Stephanopoulos (law): 3/4

The legal history is accurate. The paper correctly credits *Whitford v. Gill* (W.D. Wis. 2016) as crediting seats-votes curve evidence (not SCOTUS) and correctly characterizes *Rucho*'s failure to reach the curve's validity. The state court admissibility record (*Harper*, Ohio) is correctly stated. The "no threshold dependency" advantage of the curve over scalar metrics is well-argued and accurate: the visual comparison does not require the court to adopt a specific EG or Bias threshold. No legal accuracy errors.

#### R5 — Liang (ML/AI): 3/4

The implementation `compute_seats_votes_curve` correctly implements uniform swing over $[-0.15, +0.15]$ with 61 points, finite-difference responsiveness at $v = 0.475$ and $v = 0.525$, and bias via the swing needed to bring statewide vote to 0.50. **P1: Sign convention divergence for bias.** The code computes `bias = s_50 - 0.50` (positive when D wins majority at 50% statewide vote). The paper defines Partisan Bias as $S(0.50) - 0.50$ (positive = D-favoring). However, the `SeatsVotesCurve.bias` field and the `PartisanMetrics.partisan_bias` field are computed via separate functions (`compute_seats_votes_curve` and `compute_partisan_bias`), and the code comments say "Positive = Democratic-favoring, Negative = Republican-favoring" for SeatsVotesCurve.bias — which is consistent with the paper's definition. But `compute_partisan_bias` returns `0.5 - seats_at_50/n` which is the NEGATIVE. These two bias computations in the same struct have opposite signs — this is a latent consistency bug. **P1 for L.5**: `seats_votes.bias` (correct sign, matches paper) and `partisan_bias.value` (opposite sign) will report different values for the same plan, and one is wrong relative to the paper's definition.

**L.5 Scores: R1=3, R2=3, R3=3, R4=3, R5=3 → Avg 3.0 → Accept**
**P1 count: 2** (R1: sufficiency claim conflates exact EG formula with integral approximation; R5: seats_votes.bias and partisan_bias.value have opposite signs)

---

### L.6 — Proportionality and Majoritarianism

**Paper:** "Proportionality, Majoritarianism, and the Structural Floor: Gallagher Index Analysis of Algorithmic Redistricting"

#### R1 — Karypis (computational): 3/4

The Gallagher Index formula is correctly stated for the multi-party case. **P1: The two-party simplification in §2.1 is algebraically wrong.** The paper writes:
$$\text{LSq} = |v_D - s_D| / \sqrt{2} \cdot \sqrt{2} = |v_D - s_D|$$
The two-party Gallagher formula is $\text{LSq} = \sqrt{\tfrac{1}{2}[(v_D - s_D)^2 + (v_R - s_R)^2]}$. Since $v_R = 1-v_D$ and $s_R = 1-s_D$, we have $(v_R - s_R) = (v_D - s_D)$, giving $\text{LSq} = \sqrt{\tfrac{1}{2} \cdot 2(v_D - s_D)^2} = |v_D - s_D|$. The intermediate step as written in the paper introduces a spurious $\sqrt{2}$ factor that cancels oddly. The final result $\text{LSq} = |v_D - s_D|$ is correct, but the derivation path is confusing and could mislead readers trying to reproduce it. The correct intermediate form is $\text{LSq} = \sqrt{(v_D - s_D)^2} = |v_D - s_D|$ (direct simplification). The paper's intermediate expression $|v_D - s_D| / \sqrt{2} \cdot \sqrt{2}$ is mathematically fine (the $\sqrt{2}$ terms cancel) but is an odd way to write it that suggests the author derived it by a different path than the standard algebraic simplification. The numerical applications are all correct.

#### R2 — Rodden (political science): 3/4

The historical Anglo-American bonus range (1.2–1.5) is cited plausibly but without a specific reference. This should cite Gallagher's comparative work or Jackman's (2003) analysis of seat-bonus patterns. **P2**: The claim that a bonus of 1.3–1.5 is "within the historical Anglo-American average" needs a citation to support it as a factual claim in litigation evidence. The NC proportionality breakdown (§4.2) is well-constructed: the paper correctly computes that Republicans' proportional entitlement at 53% of the vote is 7.4 seats, bisect delivers 8 seats (close to proportional), and enacted delivers 10 seats. The presidential-vote caveat is less critical here since LSq measures actual seat/vote shares, not a swing model.

#### R3 — Duchin (math/redistricting): 3/4

The descriptive/normative distinction from L.0 is correctly applied here: LSq is explicitly identified as a normative metric. The structural floor argument is correctly made and is important — the claim that any compact map will have LSq > 0 due to geographic sorting and winner-take-all arithmetic is correct and necessary for proper framing. The claim that the PA Supreme Court's standard is best understood as a "proportionality floor at the neutral-criteria baseline" (rather than strict proportionality) is an accurate and nuanced reading of *League of Women Voters*. The expert witness framing ("not enacted vs. 0, but enacted vs. bisect") is legally and analytically sound.

#### R4 — Stephanopoulos (law): 3/4

The federal foreclosure section correctly traces *Bandemer* (1986), *Vieth* (2004) (noting Kennedy's concurrence preserved justiciability by one vote), and *Rucho* (2019) as the trajectory. The PA Art. I §5 quotation ("Elections shall be free and equal...") is correctly stated. The NC Art. I §10 quotation ("All elections shall be free") is correctly stated. The *Harper v. Hall* characterization — intent-and-effect test, not a proportionality standard — is accurate. The paper correctly notes *Rucho* "does not prefer proportionality or majoritarianism" — which is an important and often-misread aspect of the decision. No legal accuracy errors.

#### R5 — Liang (ML/AI): 3/4

The Gallagher Index is not implemented as a named function in the `partisan.rs` module. The module exports `PartisanMetrics` which covers EG, MM, Partisan Bias, Declination, and the seats-votes curve — but not LSq. **Minor gap**: LSq is described throughout L.6 as a computed metric, but if it is computed elsewhere (e.g., in `proportionality.rs`), the L.6 paper should reference where the implementation lives. The `crates/bisect-analysis/src/proportionality.rs` file exists in the codebase but was not specified in the review scope. Assuming proportionality.rs implements LSq, this is acceptable if the paper clarifies the implementation location. If LSq is not implemented anywhere and the paper's empirical results are manually computed, this is a **P1**. Based on the file listing showing `proportionality.rs` exists, I assign this **P2** pending confirmation that LSq is implemented there. No CI methodology is discussed for LSq in the paper, even though EG, MM, Bias, and Declination all have bootstrap CI in the implementation.

**L.6 Scores: R1=3, R2=3, R3=3, R4=3, R5=3 → Avg 3.0 → Accept**
**P1 count: 1** (R1: LSq two-party simplification derivation path is confused/misleading, though final result is correct)

---

## Cross-Track Issues

### 1. EG Sign Convention (Affects L.0, L.1, L.3, L.5)

The paper suite defines EG > 0 as Republican advantage (wasted_D > wasted_R), but the Rust implementation computes `(wasted_r - wasted_d)`, which is positive when Republican votes are wasted more (Democratic advantage). This is a systematic sign inversion. Every paper that presents a positive EG value as a Republican advantage is inconsistent with the implementation's sign convention. Across all L-papers, the empirical tables show positive EG (e.g., NC bisect EG = +0.02) as Republican advantage. In the code, positive EG means Democratic advantage. The direction label strings compensate locally, but the raw `.value` field and the paper convention are inverted. This should be a high-priority fix.

**Recommended fix**: Either (a) change the implementation to `(wasted_d - wasted_r) / total` and update direction labels, or (b) change the paper convention to match the code. The papers' convention matches the academic literature (Stephanopoulos & McGhee define positive EG = Republican advantage), so option (a) is preferred.

### 2. Partisan Bias Sign Ambiguity (Affects L.3, L.5)

Two separate computations of partisan bias exist in the implementation: `SeatsVotesCurve.bias` = $S(0.50) - 0.50$ (positive = D-favoring, matches paper) and `compute_partisan_bias()` = $0.5 - S(0.50)$ (positive = R-favoring, opposite). These will produce results of opposite sign for the same input. The `PartisanMetrics` struct exposes both fields simultaneously, creating a self-contradictory output structure.

### 3. Missing Bootstrap CI for LSq (Affects L.6)

EG, MM, Bias, and Declination all have documented bootstrap CI methodology. L.6 presents Gallagher Index results with no uncertainty quantification. If LSq is implemented in `proportionality.rs`, CI should be added for consistency. If it is not implemented, L.6's empirical claims need a clear disclosure.

### 4. Presidential Vote Proxy Caveat (Affects L.0, L.1, L.2, L.3)

All four papers use VEST 2020 presidential precinct returns as the partisan signal for all metrics. None mentions that for congressional redistricting, presidential vote can diverge from congressional incumbency patterns by 3–8 percentage points in competitive districts (split-ticket voting, candidate quality, VRA incumbency). For a paper suite aimed at expert witnesses in congressional redistricting litigation, this is a systematic omission that opposing counsel will exploit. At minimum, one sentence per paper noting the limitation and citing Jacobson (2013) or similar comparative evidence would be appropriate. This is a P2 across the suite but rises to P1 if the suite is to be used directly as expert witness guidance.

### 5. Dagger Notation Consistency

The $^\dagger$ notation for single-run bisect results is applied consistently across all tables in all papers. This is correct practice and should be maintained in final drafts.

---

## Recommended Revisions (Priority Order)

**P1 — Must fix before publication:**

1. **(L.1, L.5 implementation)** Resolve EG sign convention between papers and code. Papers define EG > 0 = Republican advantage; code computes positive EG = Democratic advantage.
2. **(L.5 implementation)** `SeatsVotesCurve.bias` and `PartisanMetrics.partisan_bias.value` have opposite signs. One is wrong relative to the paper's definition of $S(0.50) - 0.50$.
3. **(L.6 §2.1)** Rewrite the algebraic derivation of the two-party LSq simplification to use the direct path: $(v_R-s_R) = (v_D-s_D)$ implies $\text{LSq} = |v_D - s_D|$.
4. **(L.0 abstract)** Add "in single-run comparisons" or replace with an ensemble-based claim to avoid the unqualified "consistently outperforms" language.
5. **(L.3 §2, L.3 citations)** Add Gelman & King (1994) as the citation for the partisan bias computational method, alongside Grofman & King (2007) for the symmetry framework. Code's own `PB_ACADEMIC_REF` correctly cites Gelman & King (1994); the paper should match.

**P2 — Should fix before submission:**

6. **(L.1 §4.3)** Clarify the source of the "0.030 std dev" population-of-cycles estimate. Either provide a citation or expand to explain it is the authors' model estimate.
7. **(L.1 §2, L.0 §2)** Correct the MM $\approx$ EG/$2k$ approximation statement. The correct approximation for the MM-EG relationship is MM $\approx$ EG/2 for typical $k$; the $1/(2k)$ factor is the per-seat resolution of EG, not the MM-EG ratio.
8. **(All papers using presidential vote)** Add one sentence per paper noting the split-ticket caveat and citing appropriate literature.
9. **(L.6)** Add a sentence clarifying where LSq is implemented (proportionality.rs or elsewhere) and whether bootstrap CI is available.
10. **(L.6 §3.1)** Add a citation for the "historical Anglo-American bonus range 1.2–1.5."

---

*Reviews prepared for internal track quality control. Not for external distribution.*
