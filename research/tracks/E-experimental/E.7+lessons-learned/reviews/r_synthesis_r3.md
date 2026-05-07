---
round: 3
avg_score: 4.0
date: 2026-05-05
---

# Round 3 Synthesis — E.7 Six Systems, One Constraint: Design Lessons from Algorithmic Alternative Representation

## Score Summary

| Reviewer | R1 Score | R2 Score | R3 Score | Delta R2→R3 | Verdict |
|----------|----------|----------|----------|-------------|---------|
| Karypis  | 3/4 | 4/4 | 4/4 | 0 | Accept |
| Rodden   | 3/4 | 4/4 | 4/4 | 0 | Accept |
| Duchin   | 3/4 | 3/4 | 4/4 | +1 | Accept with Minor Revision |
| Stephanopoulos | 3/4 | 4/4 | 4/4 | 0 | Accept |
| Liang    | 3/4 | 4/4 | 4/4 | 0 | Accept |
| **Average** | **3.0/4** | **3.8/4** | **4.0/4** | **+0.2** | **Accept** |

**Gate Status**: PASSED (avg 4.0 ≥ 3.8 target; Duchin upgraded to 4)

---

## What the Round 3 Revision Fixed

**20% irreducibility bound documented** (Section 3.3 footnote): The single P1 item shared by Karypis and Liang — and the only Must-Address item from the Round 2 synthesis — is now resolved. The footnote provides the reference state (WI, 11.3 pp sorting gap), the minimum achievable gap (2.3 pp from Herschlag et al. 2020 NC ensemble lower bound, scaled by WI/NC sorting ratio), and the arithmetic derivation (2.3/11.3 ≈ 20%). All three reviewers who engaged with this item (Karypis, Liang, Duchin) consider it satisfactory. Duchin notes a minor methodological concern about the "sorting ratio" scaling being undefined, but accepts the overall documentation as sufficient for an Annual Review synthesis paper.

**Duchin upgrade (3→4)**: Duchin's upgrade reflects a recalibration rather than new content addressing her three items (PP sensitivity, fractional voting theory, lessons conflation). Her reasoning: after the irreducibility bound is documented, the paper has no remaining undocumented quantitative claims; her three items are supplementary concerns appropriate for a response letter and final manuscript revision rather than another review round. This is the correct judgment for an Annual Review synthesis paper.

---

## Status of All Open Items

### Resolved in Round 3
- [x] **20% irreducibility bound documentation** (Karypis P1.3, Liang P1.3) — Herschlag et al. footnote in Section 3.3

### Remaining Open (Carry to Final Manuscript)

**Duchin items (addressable in response letter + copy-edit)**:
- [ ] PP sensitivity to coastal/river boundaries — add a footnote in Lesson 5 acknowledging systematic PP understatement for FL, LA, AK
- [ ] Fractional voting mathematics — add a sentence pointing to the Banzhaf/Shapley-Shubik literature in Lesson 3
- [ ] Sorting-based vs. institutional-cost lessons — add a clarifying sentence in Section 7 distinguishing Lessons 1–2 (causal mechanism) from Lessons 3–4 (institutional cost)

**Stephanopoulos item**:
- [ ] Normative justification for constraint hierarchy conclusion — add sentence distinguishing empirical observation from normative recommendation in Section 7.2

**Rodden item**:
- [ ] County representation Pareto frontier exception in Lesson 3
- [ ] Voter geography endogeneity caveat in Limitations

**Liang item**:
- [ ] Appendix A continuous metrics table

---

## Final Disposition

**STATUS: ACCEPTED**

All five reviewers have reached a score of 4/4. The paper has resolved every P1 item across three rounds of review:
1. Exchange rate inconsistency (0.015 vs. 0.019) — resolved Round 2
2. Algorithm artifact claim evidence — resolved Round 2
3. Sorting gap heterogeneity — resolved Round 2
4. Concavity claim qualification — resolved Round 2
5. 20% irreducibility bound documentation — resolved Round 3

The remaining items are P2/P3 and appropriate for final manuscript preparation. The paper is ready for submission to *Annual Review of Political Science*.
