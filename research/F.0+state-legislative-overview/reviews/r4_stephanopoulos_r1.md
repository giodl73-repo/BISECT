# Review 4 — Nicholas Stephanopoulos
**Paper**: F.0: Algorithmic State Legislative Redistricting — A Research Program
**Round**: R1
**Score**: 3/4

## Summary

F.0 provides a useful framing for the state legislative research track. My review focuses on the legal accuracy of the overview's characterisations, particularly the distinction between Wesberry and Reynolds, and the implicit claims about how algorithmic maps compare to legally permissible enacted maps.

## Strengths

The legal framing at the beginning of Section 5.2 is accurate and clearly stated: state legislative plans are governed by Reynolds v. Sims (1964), which requires "substantial equality" rather than near-mathematical precision, with courts accepting deviations up to ±5%. The paper correctly notes that the algorithmic maps achieve ±0.5% — tighter than legally required. This is a genuine selling point for the research program and is appropriately highlighted.

The overview correctly notes that the Wesberry standard applies to congressional redistricting and Reynolds to state legislative redistricting — a distinction that many algorithmic redistricting papers elide.

## Concerns

**C1 — The ±5% figure for Reynolds is underspecified.** Courts have accepted deviations up to ±5% for state legislative plans, but this is not a categorical rule — it is the outer boundary established by the Supreme Court's approval of particular plans. The citation to Karcher v. Daggett (1983) in the overview is misleading: Karcher addresses the Wesberry standard for congressional districts, not the Reynolds standard for state legislative districts. The correct citations for the ±5% tolerance in state legislative redistricting would be Mahan v. Howell (1973) (approving 16.4% deviation for Virginia legislative districts) and Brown v. Thomson (1983) (approving deviations below 10% absent systematic discrimination). The Karcher citation should be removed or replaced.

**C2 — Nesting constitutional requirement overstated.** Section 4.2 states that "nesting requirement is constitutionally mandated (in many states, senate districts must be exact unions of house districts or of county units)." However, only California has a constitution that explicitly requires senate districts to consist of two complete, contiguous assembly districts. Other states (Iowa, for example) have statutory rather than constitutional nesting requirements, and the requirements are typically "where practicable" rather than absolute. The characterisation "constitutionally mandated" is too strong for most of the 42 compatible states.

**C3 — California's constitutional requirement is misstated.** The paper's description of California's nesting requirement ("each Senate district shall consist of two complete, contiguous Assembly districts") is cited from the F.2 introduction. California's current constitutional provision (after Prop 11, 2008) no longer requires this exact relationship — the Citizens Redistricting Commission draws house and senate maps independently, subject only to the priority ordering of criteria. The old two-district rule was eliminated as part of the commission reform. This error, if it appears in F.0 and F.2, needs correction.

**C4 — VRA compliance described as "addressed in F.6" but not previewed.** Given that VRA is the most consequential legal constraint on redistricting maps, F.0's overview of key findings (Section 5) should include at least a one-paragraph preview of what F.6 finds at state legislative scale. The overview previews compactness, balance, runtime, and nesting, but omits VRA — the legal constraint most likely to make the research program relevant to practitioners.

## Recommendation

Accept with revisions. The legal overview is broadly accurate but C1 (Karcher citation error) and C3 (California constitution error) are specific factual mistakes that require correction.
