# Review: B.11 ApportionRegions — Round 4
## Reviewer: Moon Duchin (Tufts University / MGGG)
**Expertise**: Geometric topology, redistricting mathematics, ensemble methods

---

### Summary

I appreciate the care taken in Round 4. The three revisions I most wanted — constitutional language correction, Herschlag citation, and contiguity verification — are all present. The paper is now making claims at the right epistemic level. I move to conditional acceptance.

### Strengths

1. **Constitutional language is now correct.** "No discretionary choices remain once the census data and seat count are fixed" is defensible. Critically, the paper now distinguishes this from "zero degrees of freedom" (which would require a formal uniqueness proof for min-$k$-way cuts that does not exist). The Tie-breaking paragraph in §5.4 correctly notes that uniqueness for $k > 2$ is an open problem. These two pieces fit together correctly now.

2. **Herschlag et al. (2020) is the right citation for NC-14 ensemble statistics.** This is the published, peer-reviewed NC ensemble study. Replacing the Duchin & Walch (2019) arXiv citation with Herschlag et al. (2020) is the correct move both scientifically and in terms of citational credit.

3. **Contiguity preservation sentence is a material addition.** Rebalancing algorithms that preserve contiguity are not guaranteed by the greedy boundary-swap description alone; the explicit verification claim makes this a stronger methodological commitment.

### Weaknesses

1. **"Plausible estimates" qualifier should be dropped now that Herschlag et al. is cited.** The paper still says "we note the following plausible estimates consistent with published GerryChain results." With Herschlag et al. cited, these are not estimates — they are claims sourced from a published study. The "plausible estimates" framing is now underselling the evidence. The qualifier should be replaced with "consistent with the NC-14 ensemble distributions reported in \citet{herschlag2020quantifying}."

2. **Boundary-swap selection criterion still underspecified.** The rebalancing algorithm description says "identifies census tracts $t$ adjacent to at least two districts such that reassigning $t$..." but does not specify what happens on ties (multiple tracts with equal deviation reduction). This was a P2 item from Round 3 that Round 4 does not resolve.

3. **Seed invariance caveat for byte-level assignments still needs integration.** The Limitations section correctly notes that seed invariance may not extend to exact boundary positions of tracts near district edges. But §5.1 Step 3 says "no discretionary choices remain" without cross-referencing this caveat. A parenthetical "(subject to the boundary-position caveat in §5.4)" would close this gap.

### Questions for Authors

1. Do the 70th/55th/75th percentile claims come directly from Table 2 of Herschlag et al. (2020), or are they the authors' estimates of where AR would fall in that distribution? If the latter, the paper should say so explicitly.

### Suggestions

- **P1**: Drop "plausible estimates" qualifier — replace with direct attribution to Herschlag et al.
- **P2**: Add cross-reference from Step 3 to the boundary-position caveat in §5.4.
- **P2**: Specify tie-breaking criterion in the rebalancing algorithm description.

### Verdict

[X] Accept with Minor Revisions

**Rationale**: The three Round 3 P1 items are resolved. The remaining items are P1/P2 at the copy-editing level, not requiring new experiments or structural revision. I am prepared to recommend acceptance conditional on dropping the "plausible estimates" qualifier.

**Score: 3.5 / 4.0** (up from 3.0)
