# B.16 ConvergenceSweep — R1 Revision Plan

**Round**: 1 → 2
**Date**: 2026-05-07
**Paper**: B.16 ConvergenceSweep: T=600 Statutory Seed Formula

All P1 items are ordered first. No new empirical runs are required — all fixes are
text corrections, additions of content from existing B.7 data, or statutory drafting
additions. Items are numbered for tracking.

---

## P1 Items (Blocking — Do First)

### P1-1: Fix EC_norm definition inconsistency (Duchin + Karypis)

**File**: `sections/02-algorithm.tex`

**Problem**: Definition 2.2 defines EC_norm(Pi) using a recursive bisection tree with
denominator `sqrt(min(i, k_l - i))` at each level l. Algorithm 1 applies this to full
k-way partitions produced by a direct METIS k-way call, where no recursive tree exists
and the level-index `i` is undefined.

**Fix**:
1. Add a subsection or clarifying paragraph immediately before Algorithm 1 specifying
   the METIS call mode used in ConvergenceSweep: either recursive bisection or direct
   k-way. Choose one and be explicit.
2. If direct k-way: replace the level-indexed EC_norm formula in Definition 2.2 with
   EC_norm(Pi) = EC(Pi) / sqrt(k/2), and remove the level-indexed formulation entirely.
3. If recursive bisection: make explicit in Algorithm 1 that each METIS call is a
   bisector, and clarify that tau comparisons are always at the bisection level.
4. Add a remark after Definition 2.2 noting that convergence tails tau are only
   comparable across states when the same call mode (and thus the same normalisation)
   is used throughout the B.7 sweep.

### P1-2: Fix Theorem 1 exponent error (Duchin + Karypis)

**File**: `sections/03-t-sufficiency.tex`

**Problem**: Theorem 1 states the number of distinct minimum-edge-cut k-partitions
is O(n^{2k}), but the argument — a bisection tree with k-1 internal nodes each
contributing O(n^2) choices — gives O(n^{2(k-1)}). Additionally, the Euler-formula
bound of O(n^2) cuts applies to s-t cuts, not balanced bisection cuts.

**Fix** (choose one of two approaches):

Option A (correct the proof):
- Change the theorem statement from O(n^{2k}) to O(n^{2(k-1)}).
- Replace the Euler-formula argument for O(n^2) bisection cuts with a reference to the
  planar separator theorem: balanced bisections of a planar graph have separators of
  size O(sqrt(n)) by Lipton-Tarjan (1979), and the number of such separators is
  polynomial in n.
- Update the proof sketch to reflect O(n^{2(k-1)}).

Option B (simplify the theorem — recommended):
- Replace Theorem 1 with a Proposition: "The METIS seed space is finite because
  each distinct local optimum is a distinct partition of a finite vertex set; therefore
  ConvergenceSweep terminates in finite time for any fixed k and G." Remove the
  O(n^{2k}) bound entirely.
- This weaker statement is all the paper needs. Option B avoids introducing further
  errors; if Option A is preferred, have the corrected proof reviewed before resubmission.

### P1-3: State Gumbel i.i.d. assumption explicitly and add KS test (Duchin)

**File**: `sections/03-t-sufficiency.tex`

**Problem**: The Gumbel model is presented as a consequence of extreme-value theory
("the natural extreme-value model"), but the 50 convergence tails come from 50
structurally heterogeneous graphs — not i.i.d. samples from the same distribution.

**Fix**:
1. Before or after the Gumbel fit results, add a paragraph explicitly stating:
   "We treat the 50 observed tails as approximately exchangeable draws from an
   underlying distribution of congressional graph convergence tails. This is an
   empirical assumption — not a consequence of extreme-value theory — that requires
   future US congressional graphs to resemble the 50 current state structures."
2. Add a KS goodness-of-fit test: compute D = sup|F_n(x) - F_Gumbel(x)| and report
   the KS p-value. If p > 0.05, report "the Gumbel family is not rejected at alpha=0.05."
3. Add a sentence clarifying that the statutory recommendation rests primarily on the
   empirical 89-seed margin (Georgia's observed tail = 511; T_stat = 600 = 511 + 89),
   with the Gumbel tail bound P(tau > 600) < 0.001 as supporting evidence only.

### P1-4: Populate Table 1 j* column for all 50 states (Rodden)

**File**: `sections/03-t-sufficiency.tex`

**Problem**: The j* column (last improving seed index) in Table 1 contains em-dashes
for 48 of 50 states. Only Georgia (j*=489) and Wisconsin (j*=1023) are populated.

**Fix**:
1. Extract j* for all 50 states from the B.7 sweep log (runs/ or the B.7 deposition
   JSONL). Each state's JSONL log records EC_norm at every seed; j* is the index of
   the last seed where EC_norm strictly decreased.
2. Populate Table 1 j* column completely.
3. Add a column T_needed = j* + 1 (minimum T to certify the state), so the Georgia
   T_needed = 490 entry is directly readable alongside the T_stat = 600 recommendation.
   If Wisconsin's j*=1023 implies T_needed > T_stat, add a footnote explaining whether
   this reflects a different run configuration or a figure to be corrected.

### P1-5: Add Georgia partisan case study (Rodden)

**File**: `sections/03-t-sufficiency.tex` (new subsection) or `sections/01-introduction.tex`

**Problem**: The paper shows T=500 fails Georgia geometrically but does not report
whether the certification gap is politically consequential.

**Fix**:
1. From the B.7 Georgia sweep data, identify:
   - T=500 termination plan (seed at halt): EC_norm value and D seat count.
   - T=600 certified optimal plan (minimum EC_norm seed): EC_norm value and D seat count.
2. Add a case study table or paragraph in Section 3 reporting these values.
3. If D seat count is identical: state explicitly — "the T=600 certification matters for
   geometric purity and legal reproducibility, not for partisan outcome, in the 2020
   Georgia instance."
4. If D seat count differs: quantify the difference and note its significance.

### P1-6: Correct "approximately one standard deviation" arithmetic error (Rodden)

**File**: `sections/03-t-sufficiency.tex`

**Problem**: For Gumbel(mu, sigma_hat=150), the standard deviation is
sigma * pi/sqrt(6) ≈ 1.2825 * 150 ≈ 192 seeds. The 89-seed margin is
89 / 192 ≈ 0.46 standard deviations, not 1.0.

**Fix**:
1. Search for "one standard deviation" in `sections/03-t-sufficiency.tex`.
2. Replace with "approximately 0.46 standard deviations" (or "less than half a
   standard deviation") above the empirical worst case.
3. If the text conflates the Gumbel scale parameter sigma with the standard deviation,
   add a parenthetical: "the standard deviation of Gumbel(mu, sigma=150) is
   sigma * pi/sqrt(6) ≈ 192 seeds; the 89-seed margin is approximately 0.46 SD."
4. The underlying recommendation (T_stat=600) is unchanged by this correction.

### P1-7: Qualify METIS determinism claim for parallel builds (Liang)

**File**: `sections/02-algorithm.tex` (Proposition 1 / Proposition 2.1 and proof)

**Problem**: Proposition 1 claims ConvergenceSweep is deterministic "regardless of
hardware, operating system, or parallelisation strategy." METIS's OpenMP-parallel
variants use non-deterministic work-stealing scheduling. A verifier running
multi-threaded METIS could obtain a different plan and legitimately claim the
statutory run is non-reproducible.

**Fix**:
1. Add a qualifier to Proposition 1's statement: "when METIS is invoked with
   METIS_OPTION_NTHREADS=1 (single-threaded mode)."
2. In the proof, add: "METIS's OpenMP-parallel variants use a non-deterministic
   work-stealing scheduler; the statutory build therefore sets METIS_OPTION_NTHREADS=1
   at initialisation, ensuring thread-count-independent determinism."
3. In `sections/05-implementation.tex`, show the flag in the statutory invocation
   example (Section 5.2): either as a CLI argument or as a note that the `redist`
   binary sets METIS_OPTION_NTHREADS=1 automatically in statutory mode.

### P1-8: Add T_stat elevation mechanism to Section 5.4 (Stephanopoulos)

**File**: `sections/05-implementation.tex` (Section 5.4, clause (C))

**Problem**: The conclusion states "the statute provides for an administrative process
to raise T_stat without full legislative amendment," but Section 5.4 clause (C)
contains no such mechanism.

**Fix**:
Add a sub-clause to clause (C) in the proposed statutory text:

> "(C)(iii) Administrative adjustment of threshold. — The Election Assistance Commission
> may raise the minimum convergence threshold specified in clause (ii) by rulemaking
> pursuant to 5 U.S.C. chapter 5, subject to a 60-day public comment period, if
> empirical evidence from a subsequent decennial census sweep demonstrates that the
> then-current threshold is insufficient to certify any State's congressional
> apportionment plan. No rulemaking under this clause may lower the threshold below
> the value established by law. Until any such rulemaking takes effect, States whose
> convergence tails exceed the then-current threshold shall use the maximum seed-index
> plan found within twice the statutory threshold as an interim certified plan, subject
> to judicial review."

Adjust the exact language to match DIA drafting conventions in other B-series papers,
but the structural elements (EAC authority, public comment, floor, interim provision)
must all be present.

### P1-9: Bind version string to census cycle (Stephanopoulos)

**Files**: `sections/04-sha256-seed.tex` (Section 4.3) and
`sections/05-implementation.tex` (Section 5.4)

**Problem**: Section 4.3 notes a future DIA amendment would use "DIA_SEED_V2" but
does not specify whether the version string is pinned to the census cycle or the statute
version. A 2032 challenge to a 2021 V1 map could argue V2 should have applied.

**Fix**:
1. In Section 4.3, add: "The version string is bound to the census cycle, not to the
   current statute version. The version string in effect on the date of the Census
   Bureau's public release of the redistricting dataset identifier governs the entire
   redistricting cycle, including any subsequent court-ordered redistricting. A
   subsequent DIA amendment adopting DIA_SEED_V2 applies only to the next decennial
   census release."
2. In Section 5.4, add a clause: "(D) Version binding. — The version string used in
   the seed formula specified in clause (A) shall be the version in effect on the date
   of the Census Bureau's first public release of the redistricting dataset identifier
   for the applicable decennial census, and shall not be changed for any redistricting
   conducted pursuant to that census, including court-ordered redistricting."

### P1-10: Resolve "24-hour statutory window" inconsistency (Stephanopoulos)

**File**: `sections/02-algorithm.tex` or `sections/05-implementation.tex` (Section 2.4)

**Problem**: Section 2.4 compares the Texas sweep runtime (~8.1 minutes) to "the
statutory 24-hour window," but standard redistricting statutes specify a 30-day
submission deadline. The 24-hour sub-requirement does not appear in Section 5.4.

**Fix** (choose one):

Option A (add the 24-hour sub-requirement to the statute):
- Add to Section 5.4 clause (B): "The redistricting authority shall complete the
  convergence sweep and certify the seed-neutral plan within 24 hours of the official
  release of the census redistricting dataset identifier."
- Make the intent explicit in Section 2.4's prose.

Option B (remove the 24-hour comparison — recommended):
- In Section 2.4, replace "well within the statutory 24-hour window" with
  "well within the 30-day statutory submission deadline."
- Remove any other references to a 24-hour computation sub-requirement.

Use Option A only if the DIA's drafting history supports a 24-hour computation deadline.

---

## P2 Items (Important — Do After P1)

### P2-1: Add T_stat vs. T_prac computational cost column (Karypis)

**File**: `sections/03-t-sufficiency.tex` (Table 5)

Add a column to Table 5 for the five worst-case states showing seed count and
wall-clock runtime at T=500 versus T=600. This allows practitioners to quantify
the cost of choosing T_stat over T_prac for research runs.

### P2-2: Add block-level scaling argument (Karypis)

**File**: `sections/06-conclusion.tex` (open questions)

In the "block-level resolution" open question, add: "METIS complexity is O(m log n),
so a 100x increase in n (from ~15K tract-level to ~1.5M block-level) increases
per-seed time by approximately 100x and may increase the number of distinct local
optima. An explicit empirical certification at block-level resolution is the subject
of future work."

### P2-3: State characteristics as predictors of tail length (Rodden)

**File**: `sections/03-t-sufficiency.tex` (Table 1 or footnote)

Add an informal regression note identifying which state properties predict longer
convergence tails. Key candidates: number of districts k, tract count n, fraction of
population in top-5 MSAs. Even a qualitative observation ("states with k >= 10 and
a dominant urban core — Georgia, Wisconsin, Florida — have tau > 300") is useful for
practitioners flagging hard cases before the 2030 sweep.

### P2-4: Acknowledge compactness-proportionality tradeoff (Rodden)

**File**: `sections/06-conclusion.tex` or `sections/01-introduction.tex`

Add: "ConvergenceSweep produces the minimum EC_norm plan — the most compact plan in
the METIS bisection family. In states with concentrated urban Democratic populations,
minimum edge cut tends to pack Democratic votes into fewer districts, as established
in B.0. The DIA accepts this tradeoff in exchange for algorithmic determinism;
redistricting authorities seeking proportionality should consult the
proportional-weights configuration established in B.12."

### P2-5: Clarify Table 5 measured vs. estimated runtimes (Liang)

**File**: `sections/05-implementation.tex` (Table 5 caption and body)

If the Georgia sweep (1,111 seeds) was actually run: change the caption from
"estimated" to "measured." If times are derived from a measured per-seed T(M):
add a footnote reporting the measured T(M) value (hardware spec, single-threaded,
2020 census tract graph) and label the total as "derived: j_stop * T(M)."

### P2-6: Provide Cargo.lock URL (Liang)

**File**: `sections/05-implementation.tex` (Section 5.1)

Add: "The exact Cargo.lock used for the 50-state B.7 sweep is available at [URL] at
commit [SHA]. Independent verifiers should build from this commit to reproduce Table 1."
If not yet public: "The Cargo.lock will be published at [URL] upon acceptance."

### P2-7: Specify SHA-256 concatenation order (Duchin)

**Files**: `sections/04-sha256-seed.tex` (Section 4.3) and
`sections/05-implementation.tex` (Section 5.4 statutory text)

Specify explicitly: "The SHA-256 input is the UTF-8 encoding of census_release_id,
followed immediately by the UTF-8 encoding of the ASCII string 'DIA_SEED_V1', with
no separator byte." (Or whichever order the implementation uses.) Add the same
specification to the statutory text to prevent implementation divergence.

### P2-8: Add opponent-claim legal procedure to Certificate of Seed Neutrality (Stephanopoulos)

**File**: `sections/04-sha256-seed.tex` (Section 4.4)

Add: "In the event that a party to redistricting litigation presents an alternative
plan with a lower EC_norm value, the dispute shall be adjudicated by the three-judge
district court as a factual matter, with the burden on the challenging party to
demonstrate that their sweep used the statutory seed formula, the statutory build
(METIS_OPTION_NTHREADS=1, pinned Cargo.lock), and a convergence threshold of at least
T_stat=600. A plan produced by a non-statutory sweep configuration shall not be deemed
a certified alternative."

---

## P3 Items (Minor — If Time Allows)

### P3-1: Clarify ConvergenceSweep behavior with zero-seed-variance structures (Liang)

**File**: `sections/05-implementation.tex` (Section 5.2 or note after statutory invocation)

Add: "When --structure prime-factor (ApportionRegions) is combined with --search
convergence, and the structure has zero seed variance (as established in B.11),
ConvergenceSweep terminates at the first seed because the second seed cannot improve
on the first. The sweep log records j*=0 and tau=T_stat (all non-improving seeds).
This is correct behavior; practitioners should not interpret early termination as
a malfunction."

### P3-2: Add explicit null hypothesis for T=600 adequacy (Duchin)

**File**: `sections/03-t-sufficiency.tex`

Add before or after the Gumbel fit: "The null hypothesis underlying the T_stat=600
recommendation is: the 50 state congressional graphs in the 2020 B.7 dataset represent
the full range of convergence difficulty for US House redistricting; no congressional
graph arising from the 2030 decennial census will produce a convergence tail exceeding
511 seeds. The Gumbel tail bound P(tau > 600) < 0.001 provides quantitative evidence
against the complement, and the 89-seed empirical margin provides the primary statutory
justification."
