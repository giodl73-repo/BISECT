# Review 5 (Round 2): Yiting Liang (Computational Social Science, Reproducibility)
**Paper**: H.1: BisectionEnsemble
**Round**: 2
**Recommendation**: Major Revision (Reproducibility)

---

## Response to Round 1 Concerns

### C-6: Partisan Data Citation — Addressed (Satisfactory)

The Kuriwaki (CCES) citation has been replaced with the Voting and Election Science Team (VEST) at Harvard Dataverse, and the interpolation method (areal weighting) is now specified. This is the correct fix. The citation is now accurate and the interpolation method is disclosed.

### C-5: Multi-Run Statistics — Not Addressed

The revision plan listed this as P2 (high effort). Tables 2, 3, and 4 still present single-run point estimates. The WI one-seat shift at $p=0.5$ in Table 3 is still a single-observation result with no indication of variance. The revised text in Section 4.4 now explicitly acknowledges that "three states are insufficient to establish partisan neutrality as a general property," but does not address the within-state single-run issue. A single-run WI result showing 3D/5R is not evidence of anything — it could be one outcome in a distribution that produces 2D/6R 80% of the time. For a stochastic algorithm paper, this remains the most significant empirical weakness.

I acknowledge the revision plan flagged this as high-effort and it may not be feasible before the revision deadline. However, the paper should at minimum add a note in the experimental setup stating that all Tables 2-4 results are single-run observations and that variance across runs is deferred to future work.

### C-4: GerryChain Reproducibility — Not Addressed

The GerryChain comparison in Table 1 still specifies no version, no tolerance $\varepsilon$, no ReCom variant, no initial plan, no random seed. The revision plan listed this as P2 (medium effort). A configuration footnote would be low-effort and would substantially improve reproducibility.

### Runtime Hardware — Not Addressed

Table 4 still does not specify CPU model, clock speed, Rust compiler version, or METIS version. This was a minor point in Round 1 that has not been addressed. A single sentence in Section 4.1 (e.g., "Runtimes are measured on an AMD Ryzen 9 5950X (16 cores), Rust 1.75, METIS 5.1.0") would suffice.

## New Issues Introduced by Revision

### Factual Error: GA and PA in Section 4.4

The revised Section 4.4 states: "Partisan outcomes are stable across percentile levels $p$ for NC, GA, PA (zero seat change)." GA and PA do not appear in any table in the paper. This sentence was apparently drafted in anticipation of a broader evaluation (S-4 from the revision plan) that was not executed. This is a factual error that must be corrected before publication: either remove the GA/PA references or add the data.

### VEST Citation Format

The `vest2020` bibliography entry uses `howpublished = {Harvard Dataverse}` without a DOI or URL that leads to the specific dataset. The VEST 2020 dataset has a specific DOI at Harvard Dataverse. The citation should specify the dataset DOI to be reproducible. The current entry states only a URL to the broader dataverse collection, not the specific 2020 dataset.

## Overall Assessment

The citation correction is the only reproducibility issue that has been substantively addressed. The three major reproducibility gaps from Round 1 (single-run results, GerryChain configuration, hardware specification) remain open. The factual error in Section 4.4 (GA/PA references without data) is a new problem introduced by the revision. I maintain my Major Revision recommendation until the GA/PA error is corrected and the single-run limitation is explicitly disclosed.

**Score**: 2.5/4
