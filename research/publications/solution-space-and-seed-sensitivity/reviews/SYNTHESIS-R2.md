# Synthesis Round 2
**Paper**: Solution Space of Minimum-Edge-Cut Redistricting
**Date**: 2026-05-01
**Reviewers**: R-50 (Ullman), R-1 (Liang), R-37 (Polikarpova), R-44 (Steinhardt), R-30 (Zhang)

---

## Scores

| Reviewer | R1 Score | R2 Score | Change |
|----------|---------|---------|--------|
| R-50 Ullman | 2.5 | 3.0 | +0.5 |
| R-1 Liang | 2.5 | 3.5 | +1.0 |
| R-37 Polikarpova | 3.0 | 3.5 | +0.5 |
| R-44 Steinhardt | 3.0 | 3.5 | +0.5 |
| R-30 Zhang | 3.0 | 3.5 | +0.5 |
| **Average** | **2.8** | **3.4** | **+0.6** |

**Verdict**: ACCEPT with minor revisions. Average 3.4/4 clears the 3.0 acceptance threshold. All P1 items from Round 1 are resolved or substantially addressed.

---

## What Changed (Round 1 → Round 2)

All 5 P1 items resolved:
- **P1-A**: Cheeger formula corrected (λ₂×n/4 not λ₂×W_total/4)
- **P1-B**: Proposition 3.1 proved (4-line proof, Proposition + Corollary structure)
- **P1-C**: Election data cited (VEST/Fekrazad DOI 10.7910/DVN/Z8TSH3)
- **P1-D**: ε in parameter table (statutory, not post-hoc)
- **P1-E**: Deep convergence for 8 states, false floors documented (GA/TN/WI)

Major new contributions since Round 1:
- Fiedler computation fixed (scipy ARPACK → correct λ₂); bound found non-certifying in practice (λ₂ = 7-46m, bound/achieved ≈ 2%)
- Exact TIGER perimeters via pyproj EPSG:5070 (PP now in [0,1])
- GMPP exact convergence analysis (WI/PA/NC)
- Content-derived seed recommendation (SHA256-derived)
- Complete conclusion section
- National seat totals (229D/206R vs enacted 222D/213R)

---

## Remaining Issues (prioritized)

### Must Fix Before Submission (P1 for this round)

**P1-I: PA evaluation table stale** (Ullman N-1, Liang R-1, Zhang R-1 — 3 reviewers)
Table 2 shows 100-seed PA data; Introduction and Conclusion cite 1,100-seed results. Inconsistent. Update Table 2 to 1,100-seed data: last improvement seed 181, EC=2,441km, 8D/9R.

**P1-II: WI GMPP not certified** (Liang R-3, Zhang R-2 — 2 reviewers)
WI GMPP last improved at seed 930 with 69-seed tail. The claim "both MEC and GMPP agree on 3D for WI" should be qualified: "provisionally agree; GMPP convergence not yet certified by the 300-seed standard."

### Should Fix (P2 for this round)

**P2-I: Definition 3.3 practical annotation** (Polikarpova R-1)
Add note: "In practice for US census-tract graphs, the ratio (1−δ) is not achievable; use the empirical tail criterion (§3.2) for practical certification."

**P2-II: CompactBisect release binary** (Liang R-2)
CompactBisect results still from debug binary. Run with release binary and update §4 table.

**P2-III: ε sensitivity analysis** (Steinhardt R-1)
Show that for ε ∈ {0.01, 0.02, 0.05, 0.10}, the selected partisan outcome is stable for all focal states.

**P2-IV: MEC primacy justification** (Steinhardt R-2)
Explicitly state why MEC is primary criterion over GMPP: "MEC has no interpretive degrees of freedom; GMPP requires a shape metric choice." Currently implied but not stated.

**P2-V: Complexity claim O(n²)** (Ullman N-3)
Change "O(n²) Lanczos" → "O(n×k) ARPACK steps, milliseconds for census-tract graphs."

**P2-VI: AK/HI seat citation** (Zhang R-3)
Add footnote: "AK 2020: Don Young (R) won 54.1%; HI: Kai Kahele and Ed Case (D) both won."

### Nice to Have (P3)

- Algorithm correctness argument for the false-floor phenomenon (theoretical explanation of why 200 seeds is insufficient for states with high Fiedler-ratio graphs)
- Explicitly connect Fiedler ratio (3.06× for WI) to convergence speed (GMPP last improved seed 930, EC last improved seed 891) — empirical validation of the prediction

---

## What's Strong (Do Not Change)

1. **False floor documentation** — the GA/TN/WI arcs are the paper's most important empirical contribution
2. **Honest Fiedler disclosure** — non-certifying bounds stated clearly; theoretical value acknowledged
3. **Content-derived seed** — cleanest defense against seed-choice attack
4. **National seat totals** — politically salient finding stated precisely
5. **Three-conclusion structure** (seed irrelevance / geometric sorting / edge-cut primacy) — clean and legally useful
6. **Proportionality gap as transparency metric** — correct framing for post-Rucho world

---

## Stage Transition

Current stage: **recheck**  
After P1-I and P1-II are fixed: advance to **ready**  
Venue recommendation: **Political Analysis** (primary) — interdisciplinary, accepts computational social science papers, appropriate for both the algorithmic and legal contributions.

Estimated remaining work: 2-3 hours (table update + WI caveat + P2 items).
