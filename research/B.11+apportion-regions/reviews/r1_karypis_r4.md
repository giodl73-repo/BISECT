# Review: B.11 ApportionRegions — Round 4
## Reviewer: George Karypis (University of Minnesota / AWS)
**Expertise**: Graph partitioning, METIS algorithm design, multilevel methods

---

### Summary

Round 4 addresses my two primary outstanding concerns from Round 3: METIS parameter reporting and rebalancing non-perturbation verification. Both are now resolved at an acceptable level for publication. The reproducibility paragraph in §3 is well-formed and provides the necessary information for independent replication. The contiguity preservation sentence in §4 is appropriately placed and factually correct. The constitutional language revision is a material improvement.

### Strengths

1. **Reproducibility paragraph is correct and complete.** SHA-256 seed derivation with the specific census release ID string, ufactor=5, niter=100, ncuts=1 — this is exactly the parameter vector needed. The `redist-metis` crate version and static linking note are important for bit-exact reproduction. This resolves my Round 3 primary concern.

2. **Contiguity preservation sentence is factually accurate.** The claim "no boundary swap changes the district assignment of any tract whose removal would create a disconnected district" is the correct articulation of contiguity-preserving rebalancing. It answers the question I raised in Round 3.

3. **Constitutional language revision is improved.** "No discretionary choices remain once the census data and seat count are fixed" is a defensible claim that doesn't overreach METIS's formal guarantees.

### Weaknesses

1. **Reproducibility paragraph says ncuts=1 but earlier text says "ncuts" is not reported.** Minor internal inconsistency: §3.2 (AR Algorithm) refers to METIS 5.x without the version, while the new Reproducibility paragraph specifies METIS 5.2. The version specification in the reproducibility paragraph supersedes the vaguer earlier reference; consider updating §3.2 to say "METIS 5.2" for consistency.

2. **Swap counts for non-NC/GA prime-top-level states still not reported.** I accept that this is now a P2 item — the paper's empirical scope is NC and GA, and it acknowledges the rebalancing issue for other states. For a journal submission this is fine, but reviewers at *SISC* or *JCO* may ask for the full rebalancing table. I note it here but do not require it for acceptance.

3. **"plausible estimates" framing for GerryChain percentiles persists.** With the Herschlag et al. citation now in place, the "plausible estimates" qualifier is weaker than it should be. Herschlag et al. (2020) is a published NC ensemble study; the paper should cite it as a source, not as a consistency check. Suggest: "consistent with the NC-14 ensemble distribution reported in \citet{herschlag2020quantifying}" rather than "plausible estimates consistent with published GerryChain results."

### Questions for Authors

1. Does the SHA-256 seed derivation produce the same value on all platforms (i.e., is the hash input byte-identical regardless of endianness)? This is a detail that matters for true cross-platform reproducibility.

### Suggestions

- **P2**: Update §3.2 to specify "METIS 5.2" consistently with the Reproducibility paragraph.
- **P2**: Revise the GerryChain section to drop "plausible estimates" framing since Herschlag et al. is now cited as the source.
- **P3**: Consider reporting rebalancing swap counts for all prime-top-level states in an appendix.

### Verdict

[X] Accept with Minor Revisions

**Rationale**: The Round 4 revisions resolve my Round 3 P1 items. The METIS parameter reporting is complete and correct; the contiguity preservation claim is verified. The remaining items are P2/P3 and do not require another review round.

**Score: 3.6 / 4.0** (up from 3.3)
