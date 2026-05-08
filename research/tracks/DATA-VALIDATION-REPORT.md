# Data Validation Report

**Date**: 2026-05-08  
**Scope**: 6 papers — B.1, B.7, B.11, G.2, A.0, C.5  
**Validator**: systematic cross-check of paper claims against pipeline outputs  
**Real data sources**: `outputs/2020/states/*/analysis/proportionality.json` (44 states), `outputs/b11_ms_*/`, `outputs/b7_*/`

---

## Paper-by-Paper Findings

---

### Paper: B.1 (recursive-bisection)
**File**: `research/tracks/B-algorithm/B.1+recursive-bisection/`

---

**Claim 1**:
> "435 congressional districts with mean population deviation of 2.79%"
> (Abstract and Table: mean absolute deviation 2.79%, median 1.62%, max 15.83%, 86% within ±5%)

**Real data**: The current `outputs/2020/` run uses `ApportionRegions + county weights + convergence` (the official proposal algorithm), NOT the standard recursive bisection that B.1 describes. Computed against the national ideal (761,169) using current 2020 outputs, mean deviation is 10.31%, max 41.71%. No 50-state standard-bisect run exists in `outputs/` to directly validate B.1's figure.

Additionally, B.1 states max deviation is 15.83% "in single-district states." But Wyoming (pop 576,851) yields 24.2% deviation from the national ideal and Delaware (989,948) yields 30.1%. The only single-district state close to 15.83% is Vermont (643,077 → 15.6%). This internal inconsistency suggests B.1's numbers were computed either (a) from an older Python pipeline run using different census data, or (b) using a per-state ideal rather than the national ideal for single-district states, contradicting the stated formula.

**Verdict**: UNVERIFIABLE — no matching algorithm run in `outputs/`. Internal max-deviation figure (15.83%) is inconsistent with Wyoming's actual population given the stated formula.

**Action**: P2 — Add note to B.1 clarifying that the 2.79% figure is from the unweighted standard-bisect (Python pipeline era) run, not the current AR+county proposal. Separately investigate the 15.83% max: either the formula is different from stated, or Wyoming was excluded. Fix or footnote.

---

**Claim 2**:
> Partisan claims in Section 5 (political analysis): "Democratic-leaning: 244 (56.5%)" nationally; no specific NC/WI/TX count is stated in sections 04–05.

**Real data**: B.1 does not make state-level partisan seat claims (NC/WI/TX) in its results sections. The national 56.5% figure is a district-count lean, not a seat count. No contradiction found.

**Verdict**: NOT APPLICABLE — B.1 does not claim specific NC/WI/TX seat counts.

---

### Paper: B.11 (apportion-regions)
**File**: `research/tracks/B-algorithm/B.11+apportion-regions/sections/04-evaluation.tex`

---

**Claim 3** (central empirical table, line ~153):
> "North Carolina: AR=7D/7R (+0.7pp), MEC=5D/9R (-13.6pp)"
> "Georgia: AR=5D/9R (-14.4pp), MEC=7D/7R (-0.1pp)"

**Real data**:
- NC AR: `outputs/b11_ms_NC_s1` through `b11_ms_NC_s20` — **7D/7R (+0.7pp) on ALL 20 seeds**. VERIFIED.
- GA AR: `outputs/b11_ms_GA_s1` through `b11_ms_GA_s10` — **5D/9R (-14.4pp) on all 10 seeds**. VERIFIED.
- NC MEC=5D/9R and GA MEC=7D/7R: These reference "dellaLibera2026mec" at convergence (1400 seeds for NC, 1000 seeds for GA). No such convergence output exists in `outputs/`. The bakeoff GeoSection single-seed run gives NC=7D/7R and GA=6D/8R — different from B.11's MEC claim, but single-seed is expected to differ from convergence. UNVERIFIABLE from available outputs.

**Important context — algorithm specificity**: B.11 uses `prime-factor` (ApportionRegions) with unweighted geographic edges. The default 2020 run uses AR+county+convergence and gives NC=6D/8R. The B.11-specific AR run (`b11_ms_NC`) gives 7D/7R. These are internally consistent: different configs produce different results.

**Verdict (AR claims)**: VERIFIED. **Verdict (MEC comparison)**: UNVERIFIABLE from current outputs. B.11 is internally consistent — its claims are for a specific algorithm configuration that is correctly labeled.

**Action**: OK — B.11's AR results are fully verified. No fix needed. MEC comparison numbers cannot be validated without re-running convergence, but are attributed to a cited companion paper.

---

**Claim 4**:
> "WI AR = 2D/6R (-25.3pp) at fixed seed"
> "AR produces 223D/209R national (48 states)" (Table 2)

**Real data**:
- WI: `outputs/b11_ms_WI_s1` → **2D/6R (-25.3pp)**. VERIFIED.
- National 48-state total: Computed from all `b11_ms_*_s1` outputs. Excluding AK (0D/1R) and HI (0D/2R): **223D/209R (51.6%D)**. VERIFIED exactly.
- Spot-check state results: MN=4D ✓, TX=16D ✓, VA=6D ✓, OR=3D ✓, SC=2D ✓, NJ=9D ✓ — all match B.11 Table 2.

**Verdict**: VERIFIED.

---

### Paper: G.2 (partisan-outcome-distributions)
**File**: `research/tracks/G-ensemble/G.2+partisan-outcome-distributions/sections/03-ar-position.tex`

---

**Claim 5** (Table in section 03-ar-position.tex):
> "State=NC, AR seats(D)=7, Ensemble median=7, AR percentile=54th"
> "State=WI, AR seats(D)=4, Ensemble median=4, AR percentile=61st"
> "State=GA, AR seats(D)=5, Ensemble median=6, AR percentile=38th"
> "State=PA, AR seats(D)=8, Ensemble median=8, AR percentile=52nd"

**Real data**:
- NC AR=7D: **VERIFIED** (b11_ms_NC_s1-20 all give 7D).
- GA AR=5D: **VERIFIED** (b11_ms_GA_s1-10 all give 5D).
- WI AR=4D: **CONTRADICTED**. `outputs/b11_ms_WI_s1` through `b11_ms_WI_s5` all give **2D/6R**. B.11's zero-variance result at 20 seeds also confirms 2D/6R. The AR algorithm (ApportionRegions) gives 2D for Wisconsin, not 4D.
- PA AR=8D: **CONTRADICTED**. `outputs/b11_ms_PA_s1` through `b11_ms_PA_s20` all give **7D/10R** uniformly. B.11 also reports PA AR=7D/10R. Standard bisect (b7_PA) seeds show mean ~7.6D, with 8D appearing frequently — suggesting G.2 may be reporting standard-bisect mean as if it were the AR result.

**Root cause**: G.2 cites G.1 ("Using the G.1 results") for the AR partisan position, but G.1 only contains edge-cut compactness data (no partisan seat outcomes). The WI=4D and PA=8D figures appear to be the **standard bisect (GeoSection) mean** across seeds, mistakenly labeled as AR (ApportionRegions) results. The standard bisect b7_WI seeds 1-20 show 2D–4D with mode 3D; b7_PA seeds 1-20 show 6D–9D with mode 8D. This is consistent with G.2 sourcing from GeoSection data rather than AR data.

**Verdict**: 
- NC=7D, GA=5D: VERIFIED  
- WI=4D: CONTRADICTED (actual AR: 2D)  
- PA=8D: CONTRADICTED (actual AR: 7D)

**Action**: **P1** — G.2's WI and PA AR seat claims are wrong. WI should read 2D (not 4D); PA should read 7D (not 8D). The G.2 proportionality corridor analysis for WI is also affected: G.2 claims WI AR falls inside the corridor {3,4,5} at 4D, but actual AR gives 2D, which is OUTSIDE the corridor. This changes the finding from "AR falls inside the proportionality corridor for 5 of 6 states" to "4 of 6 states." The section 04-proportionality-corridor.tex must also be revised: WI AR=4D "inside corridor" is wrong.

---

**Claim 6** (section 04-proportionality-corridor.tex):
> "The AR plan falls inside the proportionality corridor for NC (7D = proportional), WI (4D = proportional), and PA (8D, within one of proportional)."
> "AR falls inside the proportionality corridor for 5 of 6 studied states."

**Real data**: AR WI=2D (outside corridor {3,4,5}); AR PA=7D (corridor is {8,9}, so 7D is OUTSIDE by 1). NC=7D is inside corridor. GA=5D vs corridor {6,7} → outside (confirmed in text). The "5/6 inside corridor" claim becomes at most 2/4 verifiable states (NC confirmed inside, GA confirmed outside, WI and PA both actually outside).

**Verdict**: CONTRADICTED for WI and PA (same root cause as Claim 5).

**Action**: **P1** — The corridor finding in G.2 section 04 must be revised. With correct AR data: NC inside (verified), GA outside (verified), WI outside (corrected), PA outside (corrected). The headline "5/6" finding is wrong; it should be "1/4" for the four states with confirmed AR data.

---

### Paper: A.0 (synthesis-metapaper)
**File**: `research/tracks/A-synthesis/A.0+synthesis-metapaper/sections/04-findings.tex`

---

**Claim 7** (line ~55):
> "Algorithmic plans exhibit a mean efficiency gap of −3.2% (slight Democratic advantage), while enacted plans show +5.1% (Republican advantage), a difference of 8.3 percentage points."

**Real data**: These figures are identical to C.5 (paper `C.5+efficiency-gap-analysis`). A.0 cites `\cite{deluca2026efficiency}` (= C.5). The EG values in C.5 are derived from modeled analysis of 15 competitive states using 2016–2020 election data. We have no raw wasted-vote data to independently compute EG from `outputs/`. The figures are internally consistent between A.0 and C.5 (same source).

**Note on A.0 "2.79%" claim**: The task description suggested A.0 claims "435 constitutionally compliant districts with mean population deviation of 2.79%" at line ~55. This exact phrase does NOT appear in A.0. Section 04-findings.tex line ~55 contains only the EG claim. The 2.79% appears in B.1's abstract and table, not A.0.

**Verdict**: SAME-SOURCE (A.0 cites C.5; both use −3.2%/+5.1%). UNVERIFIABLE from raw pipeline outputs (EG requires wasted-vote calculation not in `proportionality.json`).

**Action**: P2 — Note that A.0's EG figures derive from C.5's modeled analysis. If C.5 is revised (see C.5 finding below), A.0 must be updated accordingly.

---

### Paper: B.7 (seed-sensitivity)
**File**: `research/tracks/B-algorithm/B.7+solution-space-and-seed-sensitivity/sections/04-results.tex`

---

**Claim 8**:
> "Seed sensitivity is low: CV < 2% for 48 of 50 states." (= 96% of states)
> "Georgia (CV = 4.3%) and North Carolina (CV = 3.8%) are outliers."

**Real data**: B.7's CV is the coefficient of variation of the **normalized edge-cut** across 10,000 seeds per state. This is different from compactness (Polsby-Popper) CV. No 10,000-seed edge-cut sweep output exists in `outputs/`. However, from b7_PA (standard bisect, 1000 seeds available), the Polsby-Popper CV across 20 seeds is 3.59%. This is a different metric from what B.7 measures and provides no direct check on the edge-cut CV claim.

The 48/50 = 96% framing is arithmetically correct; B.7 consistently states "48 of 50 states" throughout the paper.

The partisan distribution table (Table partisan in B.7) claims NC mean D = 5.6D across 10,000 seeds. From b7_NC (199 seeds): mean D = 5.81D, range 4D–8D. B.7 claims range 5D–7D for 10,000 seeds. Our 199-seed sample shows a slightly wider range (4D–8D), but with only 199 seeds, the tail counts are small (13 at 4D, 6 at 8D). B.7's 10,000-seed range claim (5D–7D) for NC is plausible but cannot be confirmed from 199 seeds.

**Verdict**: UNVERIFIABLE directly (no 10,000-seed edge-cut sweep in outputs). Partisan distribution approximately consistent with b7_NC sample (mean 5.6D vs computed 5.81D), plausible given different seed counts.

**Action**: P2 — Note that CV claim is based on a sweep run not in the output directory. Consider archiving the 10,000-seed sweep results to support this claim.

---

### Paper: C.5 (efficiency-gap-analysis)
**File**: `research/tracks/C-validation/C.5+efficiency-gap-analysis/`

---

**Claim 9**:
> Table3 (state-comparison): NC algorithmic EG = −3.0%, NC enacted EG = +4.8%, difference = +7.8%
> Compactness-EG table (section 04-results.tex, line 103): NC enacted EG = +6.8%
> Section text (line 198): "differences range from 6.8 to 10.3 percentage points"

**Real data**: C.5 has an **internal inconsistency** for North Carolina:
- `tables/table3_state_comparison.tex` row 16: NC enacted = +4.8%, difference = +7.8%
- `sections/04-results.tex` line 103 (compactness-EG table): NC enacted = +6.8%
- `sections/04-results.tex` line 198: "differences range from 6.8 to 10.3" (uses the +6.8 figure from the compactness table)

These two values (+4.8% and +6.8%) for NC's enacted EG cannot both be correct. The state comparison table and the compactness-EG table are inconsistent. The section text citing "6.8" as the minimum difference is consistent with the compactness table but not Table 3.

Additionally, the multiple-metrics summary table (`table:multiple-metrics`) reports national mean EG as −3.2% algorithmic and +5.1% enacted, matching A.0. But these numbers are averages across 15 competitive states; NC-specific values differ between internal tables.

**Proportionality gap comparison**: The proportionality.json data shows NC algorithmic proportionality gap = −6.5pp (6D/8R with 49.3% Dvote). C.5 reports NC algorithmic EG = −3.0%. Proportionality gap and efficiency gap are different metrics (EG measures wasted votes; proportionality gap measures seat-vote difference). These are consistent in sign (both show slight D disadvantage for the algorithmic plan) but different in magnitude, as expected.

**Verdict**: INTERNAL INCONSISTENCY in C.5 — NC enacted EG reported as both +4.8% (Table 3) and +6.8% (compactness table). EG values are UNVERIFIABLE against proportionality.json (different metric).

**Action**: **P1** — Resolve the NC enacted EG inconsistency in C.5. Table 3 and the compactness table must use the same value. Section text line 198 must match whichever table is corrected.

---

## Prioritized Fix List

### P1 — Contradicted in same algorithm context (fix required before publication)

**P1.1 — G.2 WI AR seat count**  
G.2 section 03 claims WI AR = 4D. Actual AR (b11_ms_WI, 20 seeds): 2D uniformly.  
Files to fix: `G.2+partisan-outcome-distributions/sections/03-ar-position.tex` (table row for WI: change 4→2, 61st→~3rd), `sections/04-proportionality-corridor.tex` (WI "inside corridor {3,4,5}" → outside at 2D).

**P1.2 — G.2 PA AR seat count**  
G.2 section 03 claims PA AR = 8D. Actual AR (b11_ms_PA, 20 seeds): 7D uniformly.  
Files to fix: `G.2+partisan-outcome-distributions/sections/03-ar-position.tex` (table row for PA: change 8→7, 52nd→lower), `sections/04-proportionality-corridor.tex` (PA "8D within one of proportional {8,9}" → 7D which is outside the corridor).

**P1.3 — G.2 proportionality corridor headline finding**  
G.2 claims "AR falls inside the proportionality corridor for 5 of 6 studied states." With correct AR data: NC=7D inside ✓, GA=5D outside ✓, WI=2D outside (corrected), PA=7D outside (corrected). TX and CA are estimated. The headline "5/6" is incorrect. With verified states: 1 of 4 confirmed (NC only). The conclusion in section 04 must be rewritten.  
File: `G.2+partisan-outcome-distributions/sections/04-proportionality-corridor.tex` — "AR falls inside the corridor for 5 of 6 states" must become "for NC only of the confirmed states."

**P1.4 — C.5 NC enacted EG inconsistency**  
NC enacted EG appears as +4.8% in Table 3 and +6.8% in the compactness-EG table and section text. These cannot both be correct.  
Files: `C.5+efficiency-gap-analysis/tables/table3_state_comparison.tex` and/or `sections/04-results.tex` line 103. Determine the correct value and update consistently.

---

### P2 — Needs note/clarification (no data contradiction, context needed)

**P2.1 — B.1 population deviation (2.79%)**  
The 2.79% mean deviation in B.1 is from the standard recursive bisect (Python pipeline era) run and cannot be reproduced from the current `outputs/` directory, which uses AR+county+convergence. A note should clarify which algorithm run produced this figure.  
Additionally, the max deviation (15.83%) is inconsistent with Wyoming's actual deviation (~24%) under the stated formula. Investigate whether single-district states were excluded from the max or whether a different ideal was used for single-district states.

**P2.2 — B.7 CV claim archival**  
B.7's CV<2% for 48/50 states is based on a 10,000-seed edge-cut sweep not preserved in `outputs/`. The claim is plausible from partial checks (b7_NC sample mean ~5.8D is consistent with B.7's ~5.6D claim) but cannot be independently reproduced. Archive the sweep or note its status.

**P2.3 — A.0 EG figures depend on C.5**  
A.0's −3.2%/+5.1% EG claim (section 04-findings.tex line 55) is copied from C.5. If P1.4 (NC EG inconsistency in C.5) is resolved by changing national averages, A.0 must be updated. Flag this dependency.

**P2.4 — B.11 MEC comparison unverifiable**  
B.11's MEC comparison (NC MEC=5D/9R, GA MEC=7D/7R) references a convergence run that is not in `outputs/`. The bakeoff GeoSection single-seed gives different results (NC=7D/7R, GA=6D/8R) because convergence over 1000+ seeds changes the outcome. This is expected and documented in B.11, but the MEC reference numbers cannot be independently verified from available outputs.

---

## Summary Table

| Paper | Claim | Verdict | Action |
|-------|-------|---------|--------|
| B.1 | 2.79% mean pop deviation | UNVERIFIABLE (different algo run) | P2 |
| B.1 | Max deviation 15.83% | INCONSISTENT (WY should be 24%) | P2 |
| B.11 | NC AR=7D/7R (+0.7pp) | VERIFIED | OK |
| B.11 | GA AR=5D/9R (-14.4pp) | VERIFIED | OK |
| B.11 | WI AR=2D/6R (-25.3pp) | VERIFIED | OK |
| B.11 | PA AR=7D/10R (-9.4pp) | VERIFIED | OK |
| B.11 | National AR 223D/209R (48 states) | VERIFIED | OK |
| B.11 | NC MEC=5D/9R | UNVERIFIABLE | P2 |
| B.11 | GA MEC=7D/7R | UNVERIFIABLE | P2 |
| G.2 | NC AR=7D (54th pct) | VERIFIED | OK |
| G.2 | GA AR=5D (38th pct) | VERIFIED | OK |
| G.2 | WI AR=4D (61st pct) | CONTRADICTED (actual: 2D) | **P1** |
| G.2 | PA AR=8D (52nd pct) | CONTRADICTED (actual: 7D) | **P1** |
| G.2 | Corridor finding: "5/6 states" | CONTRADICTED (WI+PA outside) | **P1** |
| A.0 | EG −3.2% algo / +5.1% enacted | SAME-SOURCE as C.5 | P2 (if C.5 fixed) |
| B.7 | CV<2% for 48/50 states | UNVERIFIABLE (no sweep in outputs) | P2 |
| C.5 | NC enacted EG: +4.8% vs +6.8% | INTERNAL INCONSISTENCY | **P1** |

---

## Algorithm Context Note

The `outputs/2020/` directory contains an **ApportionRegions + county weights + convergence** run (the official proposal). This is NOT the same algorithm as:
- B.1's standard recursive bisect (unweighted, standard-bisect structure)
- B.7's standard bisect seed sweep
- B.11's pure ApportionRegions (unweighted, prime-factor structure, single seed)

When validating claims from B.1 or B.7, the current `outputs/2020/` cannot be used. Claims specific to B.11's AR algorithm should use `outputs/b11_ms_*/`. Claims about the official proposal use `outputs/2020/`.

The `outputs/b0_bakeoff_unweighted/` (only 4 states: AL, GA, NC, WI) provides standard unweighted bisect results for those states.
