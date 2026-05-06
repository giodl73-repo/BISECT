# REVISION PLAN — H.0 PercentileSweep (Round 1)

**Paper**: PercentileSweep: Statutory Choice of Legal Posture in Algorithmic Redistricting
**Round**: 1
**Date**: 2026-05-06
**Average score**: 2.75 / 4

## Score Summary

| Reviewer | Score | Disposition |
|---|---|---|
| Karypis | 3/4 | Accept with revisions |
| Rodden | 3/4 | Accept with revisions |
| Duchin | 2/4 | Major revisions required |
| Stephanopoulos | 3/4 | Accept with revisions |
| Liang | 3/4 | Accept with revisions |
| **Average** | **2.75 / 4** | **Major revisions** |

Duchin is the blocking reviewer. The paper cannot proceed until P1-A (statute box bisection/ensemble conflation) is resolved, as it represents a legal risk embedded directly in the policy output.

---

## P1 Checklist (must resolve before Round 2)

### From Karypis

- [ ] **K-P1-A**: Clarify whether PercentileSweep uses independent per-index re-hashes ($s_i = \lfloor\text{SHA-256}(\texttt{census\_id} \,\|\, i)\rfloor$) or increment walks from $s_0$. If these differ from B.16's seed walk, state the departure explicitly and explain why the change is made. Do not claim PS is "a direct generalisation of CS" if the seed-generation mechanism has changed.

- [ ] **K-P1-B**: Fix the floor-of-hash ambiguity. SHA-256 outputs 256 bits, not a real number. Specify the byte-to-integer convention (e.g., big-endian unsigned integer interpretation, then take the low 31 bits via mod $2^{31}$). The formula must be unambiguous to an independent implementer.

- [ ] **K-P1-C**: Add a $T$-sensitivity check for the $p = 1.0$ plan in GA and NC using $T \geq 601$. Either confirm that the partisan outcome is identical to $p = 0.0$ at $T = 601$ (strengthening the claim), or qualify the 0.5-seat bound as conditional on $T = 101$.

### From Rodden

- [ ] **R-P1-A**: Add a paragraph (§4 or §5) acknowledging the geographic-sorting concern. Clarify that insensitivity across percentiles does not imply neutrality across algorithm families. Cite Rodden (2019) or equivalent urban clustering literature. Note that 223D/209R may reflect compact-algorithm structural bias, not a neutral benchmark.

- [ ] **R-P1-B**: Run actual PS sweeps for TX ($k = 38$) and CA ($k = 52$). Remove the interpolation footnote from Table 1. Present actual results or explicitly remove TX and CA from the main partisan insensitivity table and restrict the insensitivity claim to the four states with full data.

- [ ] **R-P1-C**: Add a bootstrap or jackknife confidence interval for the $p = 1.0$ partisan outcome for at least the two highest-CV states (GA and NC). Point estimates alone are insufficient for a quantitative claim entering a legal record.

### From Duchin (blocking)

- [ ] **D-P1-A** [BLOCKING]: Add a qualification sentence to the Posture 2 statute box (§5.2) stating explicitly that the plan is "representative of the bisection family, not of the full space of valid redistricting plans." The current statute box will be misread by courts and legislators as a full-ensemble representativeness claim. This is the highest-priority fix in the revision.

- [ ] **D-P1-B**: Restructure §5.1 to lead with the insensitivity argument as the response to the over-optimising objection. The "compactness is not zero-sum" argument is secondary and should follow. The current order presents a circular argument before the actual substantive response.

- [ ] **D-P1-C**: Add a sentence in §5.1 noting that G.1 ensemble percentile estimates are point estimates without confidence bounds, and that the "99.9% of valid plans" adversarial bar is contingent on G.1's sampling adequacy. Do not weaken the argument — just qualify the contingency.

- [ ] **D-P1-D**: Add a paragraph to §5 (either as a new subsection or appended to §5.4) acknowledging that legal posture analysis is restricted to compactness, population balance, and contiguity. Note that additional state-level redistricting criteria (county line preservation, community of interest, VRA) may impose constraints that interact with percentile selection. Cross-reference the VRA compliance work (Callais evidence layer).

### From Stephanopoulos

- [ ] **S-P1-A**: Qualify the Karcher analogy in §5.1. State explicitly that the compactness-Karcher analogy is an advocacy argument by structural parallel, not an established federal holding. Statute box language should say "under the Karcher framework by analogy" rather than implying direct precedential support.

- [ ] **S-P1-B**: Add a paragraph acknowledging the structural-bias challenge route: a challenger may argue the algorithm family (not the percentile choice) bakes in the partisan outcome. Distinguish this from the over-optimising objection. Note that H.0 defends against the latter; the former is defended by B.0 and B.17 (cross-reference explicitly).

- [ ] **S-P1-C**: Either (i) state that VRA compliance is evaluated independently of percentile choice with a brief justification, or (ii) add a fourth posture (VRA-constrained compactness) to §5. For Georgia specifically, note that the 6D/8R outcome at any percentile should be assessed against VRA requirements for majority-minority districts.

### From Liang

- [ ] **L-P1-A**: Run $T$-sensitivity check (see K-P1-C above — these are the same gap identified independently). Confirmed as shared P1.

- [ ] **L-P1-B** [same as R-P1-C cross-reference]: Reconcile the abstract/introduction claim of "at most 0.5 seats" with the §4.4 finding of "0 seats" observed variation. Either update the abstract to state 0 seats (conditional on TX/CA actual sweeps), or explain that 0.5 seats is a theoretical bound for the interpolated states and state this explicitly.

- [ ] **L-P1-C** [same as R-P1-B cross-reference]: Run actual TX and CA PS sweeps. Confirmed as shared P1.

---

## Consolidated P1 Work Items (de-duplicated)

| ID | Item | Effort | Priority |
|---|---|---|---|
| W1 | Run TX and CA actual PS sweeps (T=101, p in {0.0,0.25,0.5,0.75,1.0}) | Low (~7 min compute) | High |
| W2 | Run T-sensitivity check for GA and NC at T=601, p=1.0 | Low (~1 min compute) | High |
| W3 | Fix statute box §5.2: add bisection-family qualification sentence | Trivial | Blocking |
| W4 | Reconcile abstract "0.5 seats" vs §4.4 "0 seats" | Trivial | High |
| W5 | Fix SHA-256 floor ambiguity: specify byte-to-integer convention | Trivial | High |
| W6 | Clarify seed-chain vs independent-hash design vs B.16 | Low (prose) | High |
| W7 | Add geographic-sorting acknowledgment + Rodden citation | Low (prose) | High |
| W8 | Add bootstrap CI for p=1.0 partisan outcome (GA, NC) | Medium | Medium |
| W9 | Add ensemble-percentile-as-point-estimate qualification to §5.1 | Trivial | Medium |
| W10 | Restructure §5.1: lead with insensitivity argument vs over-optimising | Trivial | Medium |
| W11 | Qualify Karcher analogy as advocacy-by-parallel, not precedent | Low (prose) | Medium |
| W12 | Add structural-bias challenge route + B.0/B.17 cross-reference | Low (prose) | Medium |
| W13 | Acknowledge VRA interaction with percentile selection (GA specifically) | Low (prose) | Medium |
| W14 | Add PP compactness at p=1.0 to Table 2 | Trivial | Low |

---

## P2 Items (recommended for Round 2 if time permits)

- Karypis P2-A: Confirm METIS imbalance factor 1% vs ±0.5% population tolerance
- Karypis P2-B: Add implied CV figures from Table 2 directly
- Karypis P2-C: Clarify seed-passing protocol (top-level vs all levels) in Algorithm 1
- Rodden P2-A: Cite Rodden (2019) *Why Cities Lose* or equivalent
- Rodden P2-B: Add MA and OK to extend state sample to non-swing-state extremes
- Rodden P2-C: Explain 0.5-seat vs 0-seat bound discrepancy (resolved by W4)
- Duchin P2-A: Expand TargetedSweep dismissal to state mixing-time requirements
- Duchin P2-B: Engage Duchin & Tenner (2020) discrete geometry framework in §3.5
- Duchin P2-C: Define "geographic convergence" in one sentence
- Stephanopoulos P2-A: Note that ensemble-definition specification problem is symmetric to PS
- Stephanopoulos P2-C: Define "Districting Administrator" or use a placeholder
- Liang P2-A: Appendix with realized seed values for one state
- Liang P2-B: Table 2: add PP at p=1.0 (resolved by Karypis P2-B above)
- Liang P2-C: Elevate T-sensitivity future work; note it should precede the claim
- Liang P2-D: Note ecological inference limitation of presidential-vote partisan measure

---

## Round 2 Target

Resolve all W1–W13 items. Aim for average score >= 3.25 / 4. Duchin's score is the binding constraint: D-P1-A (W3) and D-P1-D (W13) must both be resolved before resubmission to that reviewer.
