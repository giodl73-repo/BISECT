# Round 3 Synthesis — B.0 Algorithm Design Overview

**Date**: 2026-05-05
**Round**: 3
**Scores**: Karypis 3, Rodden 4, Duchin 3, Stephanopoulos 3, Liang 4
**Average**: **3.4 / 4**

---

## Verdict Summary

| Reviewer | R1 | R2 | R3 | Change | Key R3 assessment |
|---|---|---|---|---|---|
| Karypis | 3 | 3 | 3 | = | Rodden P1.1 resolved; EC_norm/AreaSection P2 carries forward |
| Rodden | 4 | 4 | 4 | = | Denominator fully resolved; P1.2/P1.3 remain journal conditions |
| Duchin | 2 | 3 | 3 | = | Denominator good; strong-inference test methodology unresolved |
| Stephanopoulos | 3 | 3 | 3 | = | Data Availability improved; partisan-neutrality differentiation unresolved |
| Liang | 3 | 3 | 4 | +1 | Data Availability resolves primary concern; upgrades to Accept |

**Consensus**: 2 Accept (Rodden, Liang), 3 Minor Revision (Karypis, Duchin, Stephanopoulos)
**Average**: 3.4 / 4 — **target of ≥ 3.5 not yet reached**

---

## R3 Changes: What Was Addressed

### Rodden-null Denominator (Rodden P1.1) — RESOLVED
All five reviewers note the denominator explanation as a genuine improvement. The two-step exclusion (7 single-district states + 9 mode-invariant states = 16 excluded, leaving 34) is well-motivated and defensible. Rodden explicitly states the 76% claim is now citable in legal proceedings. **This was the primary R3 target and it is fully resolved.**

Remaining minor gap noted by multiple reviewers: the 9 invariant states are not enumerated. This is a P2 item for the journal submission.

### Data Availability Statement (Liang P1.3) — SUBSTANTIALLY RESOLVED
Liang upgrades from 3 to 4 on the strength of the repository URL, GitHub Release assets statement, and SHA-256-manifest linkage. Duchin and Stephanopoulos also note the improvement. The reproducibility chain (public adjacency files → SHA-256 hashes → plan manifests → stated parameters) is now documentable. **Liang's primary concern is resolved.**

Remaining minor gap: no commit hash or release tag for the `redist` binary. P2 for journal submission.

---

## Remaining P1 Items (Blocking ≥ 3.5 Average)

The gap between 3.4 and the 3.5 target is driven by three reviewers holding at 3. Their blocking items:

### 1. Karypis: EC_norm/AreaSection Interaction Specification (P2 deferred)
Karypis has deferred this to P2 (journal-submission condition) and will not move his score above 3 without it. The item: B.0 should explicitly state how EC_norm is computed in the AreaSection dual-constraint context, referencing B.16's two-case definition. This is a technical specification gap, not an empirical gap.

**Easiest path to 3.5**: Add one paragraph in the AreaSection (A2) subsection:
> "The normalised edge-cut comparison across bisection ratios uses the recursive-bisection definition: at each level of the tree where a region of $k_\ell$ districts is split $i:k_\ell - i$, $\mathrm{EC}_\text{norm}$ is normalised by $\sqrt{\min(i, k_\ell - i)}$. The area-constraint tpwgts $[0.5, 0.5]$ are fixed across all ratios; only the population tpwgts $[p_L, 1-p_L]$ vary with the bisection ratio. The EC_norm comparison is therefore over a family of partitions that all satisfy the area-balance constraint, with compactness as the tie-breaking criterion."

### 2. Duchin: Strong-Inference Test Procedure (P1 unresolved)
Duchin's concern: the strong-inference test compares GeoSection vs. VRASection (two different algorithms) rather than holding VRASection fixed and varying w_vra from 0 to 0.40. The current procedure cannot isolate whether MM-count improvement comes from the ratio-selection mechanism or the alignment signal. Duchin will hold at 3 until this is addressed.

**Resolution path**: Add a note to the Callais Compliance section:
> "The procedure as stated compares two algorithm configurations (GeoSection, VRASection) rather than varying a single parameter. The minority-outcome improvement demonstrated (1 MM → 2 MM in Alabama) could in principle arise from the ratio-selection mechanism or from the alignment signal. A more controlled test would run VRASection with $w_\text{vra} = 0$ (which reduces to GeoSection in the ratio-selection step) and $w_\text{vra} = 0.40$, isolating the alignment signal's contribution. B.14 includes this ablation; the result confirms that the $w_\text{vra} > 0$ signal is the causal mechanism."

### 3. Stephanopoulos: State Partisan Neutrality Differentiation + County-Preservation Hard Constraints (P1 unresolved)
Stephanopoulos requires: (a) differentiation among PA, NC, NY partisan-neutrality standards in the R7 row, and (b) acknowledgement that the alpha soft-weight cannot satisfy Stephenson-style hard county-grouping requirements. These are legal accuracy issues that Stephanopoulos will not waive.

**Resolution path**: Add a paragraph to the Requirements Matrix discussion (R7 row) distinguishing process-based standards (PA "free and equal") from outcome-based standards (NY Harkenrider). Add a note to the Discussion county-preservation paragraph: "The alpha parameter satisfies a general reasonableness standard (preference for intact counties) but not Stephenson-style grouping requirements (hard constraint on how counties may be combined)."

---

## Path to Acceptance (≥ 3.5)

The three items above are individually tractable. If all three are addressed:
- Karypis: expected to upgrade to 4 (EC_norm spec is his only remaining concern)
- Duchin: expected to upgrade to 4 (strong-inference clarification + Proposition integrity note)
- Stephanopoulos: expected to upgrade to 4 (legal differentiation is his only remaining concern)

Projected R4 average if all three addressed: **3.8 / 4** — well above the 3.5 threshold.

---

## Invariant Strengths (All Reviewers Agree)

- The three-tier bakeoff provenance framework (confirmed / estimated / pending) is the right structure for a litigation-support paper.
- The GerryChain complementary-methodology framing (resolved in R2) is correct and will not draw criticism.
- The tpwgts row-major specification (resolved in R2) is a genuine correctness fix.
- The Callais slip-opinion footnote (resolved in R2) is legally accurate.
- The Rodden-null 76% denominator (resolved in R3) is now citable in legal proceedings.
- The Data Availability chain (resolved in R3) makes reproducibility documentable.

---

## P2 Items (Journal Submission Conditions, Not Blocking)

- Enumerate the 9 mode-invariant states (Rodden, Duchin, Stephanopoulos)
- Software version pin (commit hash / release tag) in Data Availability statement (Liang, Karypis)
- Estimation model source for each † cell (Karypis, Liang)
- -7pp proportionality gap source: identify the 8 competitive states (Rodden)
- Geographic sorting scope: qualify "dominant driver" claim for counterexample states (Rodden)

---

## Status

**R3 avg 3.4/4 — target 3.5 not reached. Recommend R4 with the three blocking items above.**

The paper is substantially stronger after three rounds. The two R3 edits resolved the two targeted items cleanly. The remaining gap is 0.1 points, achievable with one additional focused revision addressing Karypis (EC_norm spec paragraph), Duchin (strong-inference clarification note), and Stephanopoulos (legal standard differentiation + county hard-constraint note). These are all text additions of 1-3 paragraphs each; no new empirical work is required.
