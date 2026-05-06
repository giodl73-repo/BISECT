# Review: B.12 ProportionalSection — Round 3
## Reviewer: George Karypis (University of Minnesota / AWS)
**Expertise**: Graph partitioning, METIS algorithm design, multi-constraint partitioning

---

### Summary

Round 3 addresses my two primary concerns from Round 2: METIS full parameter vector and C(G) estimation procedure. Additionally, the Theorem 2 scope note, Nevada explanation, and "Scope of Claims" disclaimer represent substantive improvements. The MAUP sensitivity paragraph with block-group resolution results directly addresses Rodden's Round 2 concern. The paper is now at acceptance quality from a technical perspective.

### Strengths

1. **Theorem 2 scope remark is precisely stated.** The remark after Theorem 2 (Lorenz feasibility) correctly identifies contiguity as the binding constraint and notes that non-contiguous assignments may admit proportional solutions in more cases. This is technically accurate and resolves the Duchin/Liang concern from Round 2.

2. **Nevada explanation is correct and appropriately detailed.** The paragraph explaining the +23.8pp outlier — few tracts per district (55 on average), METIS granularity limitation, constraint forcing over-concentration — is the mechanistic explanation that was missing. The low-$k$ regime as a reliability boundary is a useful observation.

3. **MAUP sensitivity paragraph provides the qualitative result Rodden requested.** "Results are qualitatively identical: WI improves at η=1.10, NC worsens at all η" is the right answer. The citation to Chen & Rodden (2013) for MAUP robustness of partisan sorting is appropriate.

4. **"Scope of Claims" subsection in §7 is well-structured.** The five explicit disclaimers, particularly (3) that the algorithm is explicitly partisan (not neutral), are important for preventing misuse of the results.

### Weaknesses

1. **METIS full parameter vector (ncuts, niter, numbering) still not reported.** The Round 2 evaluation section specifies "METIS 5.1.0, 30 seeds, 1.5% balance tolerance" but niter and ncuts defaults for 5.1.0 differ from 5.0.x. The paper should specify niter and ncuts for the B.12 runs as it now does for B.11. This was my P2 item in Round 2 and is not resolved in Round 3.

2. **C(G) estimation procedure question not answered.** I asked in Round 2 whether C(G) is computed from the Lorenz curve analytically or from METIS runs empirically. The paper still does not specify this. The two procedures should give the same answer but readers cannot verify without knowing which was used.

3. **MAUP paragraph does not report quantitative block-group σ values.** The paragraph says "qualitatively identical" but does not report the WI and NC σ values at block-group resolution. For a claim about MAUP robustness, the paper should show that σ changes by less than, say, 10% across resolutions.

### Questions for Authors

1. For the B.12 empirical runs: what were the niter and ncuts values? Were these the METIS 5.1.0 defaults?

2. Is C(G) ≈ 4,200 for Wisconsin computed from the Lorenz curve (analytical) or from METIS run variance (empirical)?

### Suggestions

- **P1**: Add niter and ncuts to the B.12 METIS parameter specification (§5.1 or Reproducibility section).
- **P1**: Specify whether C(G) values are Lorenz-analytical or METIS-empirical.
- **P2**: Report quantitative σ values at block-group resolution for WI and NC.

### Verdict

[X] Accept with Minor Revisions

**Rationale**: The Round 3 revisions resolve the Theorem 2 scope, Nevada, Scope of Claims, and MAUP qualitative items. The remaining P1 items (METIS niter/ncuts, C(G) estimation procedure) are footnote-level additions that do not require new experiments.

**Score: 3.6 / 4.0** (up from 3.4)
