# Review 4 — Nicholas Stephanopoulos
**Paper**: F.1: Algorithmic State House Redistricting — A 50-State Empirical Study
**Round**: R1
**Score**: 3/4

## Summary

F.1 is the primary empirical paper of the F track. For legal purposes — this paper will likely be cited in redistricting litigation as a baseline comparison — the key questions are: are the legal standards correctly characterised, are the comparisons to enacted maps legally meaningful, and does the paper make claims that go beyond what the evidence supports?

## Strengths

The legal framing is broadly accurate. The paper correctly distinguishes Wesberry (congressional, near-mathematical equality) from Reynolds (state legislative, "substantial equality," ±5% tolerance). The statement that "the algorithm achieves a tighter standard than legally required, confirming that the algorithm's population-balance constraints are conservative" is precise and defensible.

The county-split comparison is legally meaningful: county splits are explicitly prohibited or minimised in many state redistricting statutes, so showing 18% fewer splits than enacted maps is directly relevant to state-law compliance arguments.

## Concerns

**C1 — Wesberry standard misapplied to state legislative results.** The paper states that "all 50 algorithmic maps achieve population balance within ±0.5% of the ideal district population. This is the standard applied to congressional redistricting under Wesberry v. Sanders (1964)." This framing applies a congressional standard to state legislative results. While achieving ±0.5% for state legislative maps is more than adequate legally, characterising it as satisfying "the Wesberry standard" conflates two different legal regimes. State legislative maps are not subject to Wesberry — they are subject to Reynolds. The paper should consistently distinguish the applicable standard.

**C2 — "Statistically significant" comparison to enacted maps.** Section 6.2 reports that the algorithmic maps achieve better population balance than enacted maps by a paired t-test (t=3.8, p<0.001). However, the legal question is not whether algorithmic maps are statistically significantly better balanced — it is whether enacted maps exceed the Reynolds ±5% tolerance. The paper reports that "12 (32%) have maximum deviations in the 0.5%--5.0% range permitted by Reynolds." No enacted map exceeds 5%. The appropriate legal conclusion is that all 38 enacted maps are constitutionally adequate on population balance, and the algorithmic maps happen to achieve a tighter standard. Presenting the t-test as a primary result overstates the legal significance of the difference.

**C3 — Comparison map availability.** Section 7 acknowledges that 12 states "had not yet finalized their maps or did not release machine-readable shapefiles." This is a significant caveat: 12 of 50 states (24%) are missing from the enacted-map comparison. If the missing states are non-randomly distributed — e.g., if they are states that have not finalized maps because of litigation — the comparison sample may be systematically biased. The paper should discuss whether the 12 missing states are known to have enacted gerrymanders.

**C4 — "Historically aggressive" gerrymandering characterisation.** Section 6.3 describes Pennsylvania, Wisconsin, Ohio, and North Carolina as "historically aggressive partisan gerrymanders." This is accurate as a historical matter and is supported by the cited literature (McGhee, Stephanopoulos, Chen). However, given that this paper is intended to be used in litigation, the characterisation should cite to court opinions as well as academic work, to avoid the impression that this is solely a contested academic characterisation rather than a judicially recognised fact.

## Recommendation

Accept with minor revisions. The legal framing is broadly accurate. C1 (Wesberry misapplication) is a specific error that requires correction. C4 (gerrymandering characterisation) would benefit from court citations to strengthen the paper's litigation utility.
