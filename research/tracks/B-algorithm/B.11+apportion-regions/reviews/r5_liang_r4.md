# Review: B.11 ApportionRegions — Round 4
## Reviewer: Percy Liang (Stanford University)
**Expertise**: Reproducibility, evaluation methodology, algorithmic foundations

---

### Summary

I have followed this paper across four rounds. The Round 4 revisions address my two primary outstanding concerns: reproducibility parameters and the constitutional language. The Reproducibility paragraph in §3 is well-structured and provides the necessary information. The constitutional language revision is a material improvement over "literally zero degrees of freedom." I recommend acceptance.

### Strengths

1. **Reproducibility paragraph is complete and specific.** SHA-256 seed derivation formula with the exact input string, METIS 5.2, ufactor=5, niter=100, ncuts=1, static linking via `redist-metis` crate v0.2 — this is the full parameter vector. A researcher could reproduce the results with this information plus the repository. This resolves my Round 3 primary concern.

2. **Constitutional language is now epistemically appropriate.** "No discretionary choices remain once the census data and seat count are fixed" is an empirical claim supported by the 480-run seed-invariance experiment. It does not overclaim formal mathematical uniqueness. The paper now correctly presents the seed-invariance result as empirical evidence for a practical claim, not a theoretical guarantee.

3. **Herschlag et al. citation resolves the sourcing issue.** The ensemble percentile claims are now tied to a specific publication. This is sufficient for reproducibility purposes — readers can verify whether the percentile claims are consistent with the Herschlag et al. distributions.

### Weaknesses

1. **"Plausible estimates" qualifier is now internally inconsistent.** With Herschlag et al. cited, calling these "plausible estimates consistent with published GerryChain results" is inaccurate — they should be described as "values consistent with the distributions reported in \citet{herschlag2020quantifying}" or simply attributed directly. The qualifier undersells the evidence now that a specific citation exists.

2. **Graph checksum for reproducibility still absent.** I asked for this in Round 3: to verify that two researchers are partitioning the same graph, the paper should report a checksum of the census-tract adjacency graph for at least NC and GA. The SHA-256 seed derivation alone does not help if the input graph differs (e.g., different TIGER vintage, different adjacency definition). This is a P2 item for journal submission.

3. **Reproducibility package still not released.** The paragraph says "source code and build manifest at the project repository" but gives no URL or DOI. For a paper making reproducibility claims, this should be a concrete pointer. A GitHub URL or Zenodo DOI would suffice.

### Questions for Authors

1. Are the 480 runs (10 seeds × 50 states + 20 seeds × 7 focal states = 480) all producing byte-identical `.adj.bin` adjacency files? The seed-invariance is reported for partisan outcomes; what about the full tract assignments?

### Suggestions

- **P1**: Drop "plausible estimates" qualifier; replace with direct attribution to Herschlag et al.
- **P2**: Add graph checksums (SHA-256 of adjacency list + population vector) for NC and GA to the Reproducibility paragraph.
- **P2**: Add a concrete URL or DOI for the repository in the Reproducibility paragraph.
- **P3**: Clarify whether seed-invariance holds for byte-level tract assignments or only for partisan seat counts.

### Verdict

[X] Accept with Minor Revisions

**Rationale**: The Round 4 revisions resolve my R3 P1 items on reproducibility and constitutional language. The remaining items (graph checksum, repository URL, "plausible estimates" removal) are P1/P2 copy-editing level fixes that do not require a new review round.

**Score: 3.5 / 4.0** (up from 3.0)
