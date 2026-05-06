# REVISION PLAN — C.9: Adoption Case Studies
**Round 1 Panel Scores:** Karypis 3/4 | Rodden 3/4 | Duchin 3/4 | Stephanopoulos 3/4 | Liang 3/4
**Average:** 3.0/4
**Status:** Conditional accept — but Stephanopoulos's "major revisions" rating requires substantive legal corrections

## Critical Issues (Must Fix Before R2)

### C1 — Harper v. Hall Reversal [Stephanopoulos, primary; Rodden secondary]
This is the most serious legal error in the paper. The 2023 North Carolina Supreme Court reversal of *Harper v. Hall* (after reconstitution of the court) is not a peripheral procedural note — it means the court-order pathway illustrated by *Harper* is currently *not available* in North Carolina. Using *Harper* as the case study for the court-order pathway without disclosing this reversal is legally misleading.
**Action:** Revise the North Carolina case study (Section 4) to:
1. Acknowledge the 2023 *Harper* reversal explicitly, with the legal explanation (court reconstitution changed the majority; the reconstituted court found no judicially manageable standard for partisan gerrymandering under the NC Constitution).
2. Reframe the case study as "the *Harper* model" — what the court-order pathway looks like when state courts maintain partisan gerrymandering jurisdiction — applicable to states like Pennsylvania (*LWV v. Commonwealth*, 2018), Michigan (*LWVMI v. Benson*, 2022), and Wisconsin (*Clarke v. Wisconsin Elections Commission*, 2024).
3. Note that the *Harper* fact pattern establishes the institutional template even if North Carolina itself no longer has active court-order jurisdiction.

### C2 — Allen v. Milligan VRA District Count [Stephanopoulos, primary]
The paper describes the court-order pathway for North Carolina as applying VRASection weights "to protect the Black-majority district in the northeastern part of the state." But *Allen v. Milligan* (2023) required Alabama to draw a *second* majority-Black district — not just protect an existing one. Any court-order pathway for a state with an existing Section 2 violation (like Alabama or Louisiana) must address how VRASection generates the *required number* of majority-minority districts, not just protects existing ones.
**Action:** Revise Section 4 to address the VRA compliance question for the court-order pathway: does VRASection produce the number of majority-minority districts required by Section 2 for the state in question? Add a paragraph explaining that the *Allen* requirement (multiple minority-opportunity districts where the demographic distribution supports them) is satisfied by VRASection's optimization over minority community alignment scores across the state.

### C3 — Reproducibility: Seed Specification [Duchin, primary]
The paper claims that algorithmic maps are "reproducible" because "the same inputs produce the same result." But reproducibility depends on fixing the random seed, and the paper does not specify whether the adopted map corresponds to a fixed-seed run or a ConvergenceSweep. A ConvergenceSweep may produce different results on different hardware due to METIS floating-point non-determinism.
**Action:** Add a subsection on "canonical run specification" that explains: (a) which seed is used for the adopted map (the best-of-ConvergenceSweep seed, recorded in the deposition log), (b) how the seed is recorded and can be used to reproduce the output, and (c) why cross-platform reproducibility is guaranteed (or what caveats apply).

## Moderate Issues (Should Fix Before R2)

### M1 — California Compactness Claim Evidence [Karypis]
Section 3.2 asserts that "Polsby-Popper and Reock compactness scores for California algorithmic districts exceed the CRC's enacted map on average" without providing data.
**Action:** Either (a) provide the comparison data (PP and Reock scores for both algorithmic and CRC enacted maps), or (b) remove the claim and replace with "compactness scores for California algorithmic districts are computed using redist analyze; comparison to CRC enacted maps is available upon request."

### M2 — Harris v. AIRC Cautionary Note [Rodden]
The AIRC's community-of-interest adjustment authority has historically been a source of legal controversy (*Harris v. AIRC*, 2016).
**Action:** Add a paragraph acknowledging that community-of-interest adjustments have been litigated as partisan manipulation (*Harris v. AIRC*) and that the two guardrails proposed (geographic justification, documented factual finding) are designed to address the specific defect alleged in *Harris*.

### M3 — Effects-Based State Constitutional Challenges [Stephanopoulos]
The paper's claim that algorithmic maps are legally stable because they eliminate intent-based challenges overlooks effects-based challenges under some state constitutional frameworks (e.g., Pennsylvania's free and equal elections clause).
**Action:** Add a paragraph acknowledging that state-court effects-based challenges remain possible even for algorithmically drawn maps, and that the response is the partisan symmetry finding from C.8 (algorithmic maps have near-zero efficiency gap and mean-median difference).

### M4 — Moore v. Harper Citation [Stephanopoulos]
The paper omits *Moore v. Harper* (2023), which rejected the independent-state-legislature theory and is relevant to the Elections Clause analysis for the commission pathway.
**Action:** Add *Moore v. Harper* to the Arizona AIRC section with a sentence: "The Court's rejection of the independent-state-legislature theory in *Moore v. Harper* (2023) reinforces *AIRC*'s holding that state legislatures may delegate redistricting authority to independent commissions."

## Minor Issues (Optional for R2)

- Address the contracting/conflict-of-interest question for technical expertise (Liang)
- Specify algorithm parameter choices for each case study jurisdiction (Karypis)
- Address California proportionality under geographic sorting constraints (Rodden)
- Add specific candidate states for early adoption beyond AZ, CA, NC (Liang)
- Clarify CVAP vs. total population for VRA threshold, consistent with D.5 (Karypis)

## Target R2 Score

The *Harper* reversal (C1) is the blocking issue — it must be corrected for the paper to be credible in legal circles. With C1 corrected and C2-C3 addressed, the score should move from 3.0/4 to ~3.3/4.
Stephanopoulos's "major revisions" will likely become "minor revisions" once C1 is addressed.
