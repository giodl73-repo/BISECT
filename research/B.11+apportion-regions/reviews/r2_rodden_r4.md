# Review: B.11 ApportionRegions — Round 4
## Reviewer: Jonathan Rodden (Stanford University)
**Expertise**: Political geography, urban-rural sorting, redistricting and representation

---

### Summary

The Round 4 revisions address the constitutional language and reproducibility items I flagged in Round 3. This paper has made steady, credible progress across four rounds. The core empirical contribution — the NC/GA divergence under identical factorization — remains compelling. The constitutional argument is now more carefully scoped. I am prepared to recommend acceptance.

### Strengths

1. **Constitutional reframing is defensible.** Step 3 of the constitutional argument now reads "No discretionary choices remain once the census data and seat count are fixed" rather than "zero degrees of freedom." This is the correct level of precision. The seed-invariance result supports this claim for the tested states without overclaiming formal mathematical uniqueness.

2. **Herschlag et al. citation resolves the GerryChain sourcing issue.** The 70th/55th/75th percentile claims are now tied to a specific published ensemble study rather than a general reference. This is the appropriate treatment.

3. **Rebalancing contiguity claim is now explicit.** The added sentence verifying that no boundary swap disconnects a district is an important methodological commitment. It specifies the rebalancing algorithm's constraint.

### Weaknesses

1. **WI multi-seed vs. seed-invariance tension unresolved.** The paper still says AR is "functionally seedless" in §5.1 but then notes in §4.4 that WI requires "a multi-seed AR protocol" for the minority representation concern. This is not a contradiction — it applies to the single-seed WI AR result ($-25.3$\,pp) — but a careful reader will note the tension. A sentence clarifying that WI's seed-invariance holds (same outcome across all seeds) but that the single canonical outcome is a local optimum that multi-seed MEC escapes would resolve this.

2. **Reapportionment disruption study still deferred.** I raised this in Round 3. It remains deferred to future work. This is acceptable for acceptance but the paper would be stronger with even a simple one-state illustration of the recomputation depth proposition.

### Questions for Authors

1. For Wisconsin, is the 2D/6R outcome truly seed-invariant (same on all 20 seeds), or does it show variance? The paper reports zero variance in Table 4 but also notes WI needs a multi-seed protocol — this appears contradictory.

### Suggestions

- **P2**: Add one sentence clarifying that WI's seed-invariance holds (same outcome on all 20 seeds) but the single canonical outcome is a local optimum that MEC escapes via seed diversity — these are compatible properties.
- **P3**: Add a one-paragraph illustration of the recomputation depth proposition for a single 2010→2020 transition.

### Verdict

[X] Accept with Minor Revisions

**Rationale**: The Round 4 revisions adequately address my Round 3 P1 items. The constitutional argument reframing and the Herschlag et al. citation are the two changes I most wanted. The WI tension is a P2 item that can be resolved without a new review round.

**Score: 3.6 / 4.0** (up from 3.2)
