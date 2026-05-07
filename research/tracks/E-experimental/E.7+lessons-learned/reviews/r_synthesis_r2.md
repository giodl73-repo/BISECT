---
round: 2
avg_score: 3.8
date: 2026-05-05
---

# Round 2 Synthesis — E.7 Six Systems, One Constraint: Design Lessons from Algorithmic Alternative Representation

## Score Summary

| Reviewer | R1 Score | R2 Score | Delta | Verdict |
|----------|----------|----------|-------|---------|
| Karypis  | 3/4 | 4/4 | +1 | Accept with Minor Revision |
| Rodden   | 3/4 | 4/4 | +1 | Accept with Minor Revision |
| Duchin   | 3/4 | 3/4 |  0 | Minor Revision |
| Stephanopoulos | 3/4 | 4/4 | +1 | Accept with Minor Revision |
| Liang    | 3/4 | 4/4 | +1 | Accept with Minor Revision |
| **Average** | **3.0/4** | **3.8/4** | **+0.8** | **Accept with Minor Revision** |

**Gate Status**: PASSED (avg 3.8 ≥ 3.4 target, no score < 3.0)

---

## What the Revisions Fixed (Praised by Multiple Reviewers)

**Exchange rate inconsistency resolved** (Section 6.1): All four reviewers who raised this (Karypis P1.1, Liang P1.1, and noted by Rodden and Stephanopoulos) consider it fully resolved. The within-system (0.019, E.5 parametric) vs. cross-system (0.015, E.0 portfolio average) distinction is conceptually correct and clearly explained. The headline figure (0.015) is now justified explicitly as appropriate for a paper about the full portfolio. This was the paper's most significant internal consistency problem and is now fixed.

**Sorting gap heterogeneity added** (Section 3/Lesson 2): Rodden's primary concern is satisfied. The revised empirical results paragraph introduces the baseline as "the median sorting gap of +4.2 pp (range: 1.1 pp in ND to 11.3 pp in WI)" and names high-sorting states (IL, NY, PA, MI) and low-sorting states (IA, ND) explicitly. This transforms a misleadingly uniform national statistic into a contextualised range with policy implications.

**Algorithm artifact verification added** (Section 6.3): Karypis's P1.2 is satisfied. The revised paragraph adds B.1 vs. B.2 comparison (exchange rates 0.016 vs. 0.015, statistically indistinguishable), providing the comparative algorithmic evidence requested.

**Concavity claim qualified** (Section 3.4): Liang's P1.2 is satisfied. The claim now reads "the data suggest a concave relationship... consistent with diminishing returns" rather than "the relationship is concave," with explicit acknowledgment that the functional form cannot be confirmed from three data points.

---

## Open Issues After Round 2

### Must Address Before Acceptance

**[Karypis P1.3 / Liang P1.3] 20% irreducibility bound undocumented** — Section 3.3 still asserts that 20% of the sorting gap is irreducible, derived from "the fraction of Democratic votes in census tracts with >70% Democratic composition," without providing the calculation, the national fraction, the translation formula, or the specific E.1 citation. Two reviewers flag this. Requires: (a) the national fraction of Democratic votes in >70% tracts, (b) the translation to the 20% bound, and (c) the E.1 section reference.

### Should Address in Next Revision

**[Duchin P1.1] Polsby-Popper sensitivity to coastal/river boundaries** — PP is used without acknowledging its boundary-length sensitivity. States with complex coastlines (FL, LA, AK) have systematically lower PP independent of district drawing quality. The paper should either add a second compactness metric (Reock or convex hull ratio) or explicitly note this confound in Lesson 5.

**[Duchin P1.2] Fractional voting mathematics** — Lesson 3's discussion of county representation does not engage with weighted voting theory (Banzhaf/Shapley-Shubik power indices, coalition formation, supermajority implications). Annual Review readers will expect this engagement.

**[Duchin P1.3] Sorting-based vs. institutional-cost lessons** — The synthesis conflates Lessons 1–2 (causal sorting mechanism) with Lessons 3–4 (institutional cost measurements). The design tension framework would be strengthened by explicitly separating these two logical categories.

**[Stephanopoulos P1.1] County representation and Wesberry** — The constitutional analysis conflates the two Wesberry requirements (equal-population districts vs. equal-weight votes). The fractional voting mechanism addresses one but not both.

**[Stephanopoulos P1.2] Normative inference** — Section 7.2's conclusion that proportionality should not be a drawing criterion needs a normative justification, not just an empirical observation that drawing instruments cannot fully solve the structural problem.

**[Rodden P1.2] County representation Pareto frontier exception** — Lesson 3 should acknowledge that county representation partially escapes the compactness-proportionality frontier (achieving proportionality without geometric optimisation) as an important exception to Lesson 5's steepness claim.

**[Rodden P1.3] Voter geography endogeneity** — The Limitations section should acknowledge that simulations hold voter geography constant and cite relevant work on residential sorting as an equilibrium phenomenon.

---

## Round 3 Priority Actions

### P1 (Required for Acceptance)
- [ ] Document the 20% irreducibility bound: provide the national fraction of Democratic votes in >70% tracts, the translation formula, and a specific E.1 citation

### P2 (Should Address)
- [ ] Add PP sensitivity note (coastal/river boundaries) in Lesson 5 or supplementary compactness metric
- [ ] Engage with weighted voting theory (power indices) in Lesson 3
- [ ] Distinguish sorting-based lessons (1, 2) from institutional-cost lessons (3, 4) in synthesis
- [ ] Clarify the two Wesberry requirements in Lesson 3's constitutional analysis
- [ ] Add normative justification for the constraint hierarchy conclusion (why partial proportionality improvement at compactness cost is worse than no reduction)
- [ ] Note county representation as a partial exception to Pareto frontier steepness
- [ ] Add voter geography endogeneity caveat in Limitations

### P3 (Optional)
- [ ] Add E.4 null result to abstract (Rodden P2 suggestion)
- [ ] Provide summary table of five lessons with key statistics, evidence source, mechanism
- [ ] Add histogram or table of state-level sorting gap distribution
