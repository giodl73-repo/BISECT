---
reviewer: Percy Liang
round: 3
score: 4
date: 2026-05-05
---

## Response to Revision

My Round 2 score was 4 (Accept with Minor Revision). My single remaining concern from Round 2 was the undocumented 20% irreducibility bound (P1.3). This round that item has been addressed.

**P1.3 — 20% irreducibility bound (addressed, satisfactory).** The footnote in Section 3.3 now provides:
- Reference state: WI, sorting gap 11.3 pp
- Minimum achievable gap: 2.3 pp (Herschlag et al. 2020, NC ensemble lower bound, scaled by WI/NC sorting ratio)
- Arithmetic: 2.3/11.3 ≈ 20%
- Interpretation: the remaining 80% is structurally addressable via algorithm design

From a statistical perspective, the derivation is honest about its approximation nature (the phrase "scaled by WI/NC sorting ratio" acknowledges that this is an extrapolation). The Herschlag et al. citation gives the baseline measurement a literature source; the scaling step is an author-added approximation that is clearly labeled as such rather than implied. This is the appropriate level of documentation for a synthesis paper: not a primary empirical result, but a documented bound with a calculable derivation and a supporting citation. Satisfied.

**Remaining items.** The linearity test for the Pareto frontier and the Appendix A data (my P2 items from Round 2) remain deferred. I continue to regard these as appropriate for the Appendix of the final submission rather than as barriers to publication.

## Summary Assessment

All P1 items across all three rounds are now satisfied. The paper is statistically honest in its claims, appropriately qualified in its extrapolations, and consistent in its use of figures across sections. The irreducibility bound documentation closes the last gap I had flagged. Ready for publication.

## Score: 4 — Accept
