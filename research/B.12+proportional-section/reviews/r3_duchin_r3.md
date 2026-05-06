# Review: B.12 ProportionalSection — Round 3
## Reviewer: Moon Duchin (Tufts University / MGGG)
**Expertise**: Geometric topology, redistricting mathematics, Lorenz curves, ensemble methods

---

### Summary

Round 3 addresses my Round 2 P1 concern (Theorem 2 scope). The remark after Theorem 2 is technically accurate: contiguity is the binding constraint. The "Scope of Claims" subsection in §7 is the right addition and is well-scoped. I am prepared to recommend acceptance. The Nevada explanation is mechanistically correct and fills the most puzzling empirical gap.

### Strengths

1. **Theorem 2 scope remark is technically precise.** "Non-contiguous assignments may admit proportional solutions in more cases; contiguity is the binding constraint." This is correct. The theorem itself remains stated as a biconditional for the non-contiguous case ("without contiguity constraint"), and the remark clarifies the scope. This is the right structure.

2. **Scope of Claims subsection is appropriately positioned.** Items (1)-(5) correctly identify the limits of what the paper demonstrates. Item (3) — the algorithm is explicitly partisan, not neutral — is the most important disclaimer and deserves its prominent position. I particularly appreciate the distinction between demonstrating feasibility (what the paper does) and demonstrating desirability (what it does not).

3. **Nevada paragraph is mechanistically correct.** The low-k granularity argument is the right explanation. The paper correctly notes that this is a general pattern (small k states are less reliable) rather than an isolated anomaly. This turns an embarrassing outlier into a useful finding.

4. **MAUP result strengthens the proportionality paradox claim.** If the paradox holds at both tract and block-group resolution, it is a genuine geographic phenomenon, not a measurement artifact. The Chen & Rodden citation is appropriate given their prior work on partisan sorting.

### Weaknesses

1. **Theorem 2 remark places the scope note after the proof, not in the theorem statement.** Mathematically, this means the theorem's "if and only if" quantifier applies to the non-contiguous case by default, with the contiguity caveat appearing as a remark. A cleaner structure would label the theorem as "Lorenz feasibility (non-contiguous)" in the heading and state the contiguity limitation in the theorem body itself. The current structure is acceptable but slightly awkward.

2. **B.9 baseline verification column still absent.** I asked in Round 2 for the B.9 column in Table 1 to be verified against B.9 paper figures. The paper still says "B.9 column is the geography-only baseline" without citing where those values come from. This is a P2 item.

3. **Confidence intervals for Table 1 not reported.** Liang raised this in Round 2. 30-seed runs — are results deterministic? If zero inter-seed variance, the paper should say so explicitly. If there is variance, CIs are needed. This remains unaddressed.

### Questions for Authors

1. Are the Table 1 entries (B.9 baseline, η=1.05, 1.10, 1.20) deterministic across all 30 seeds, or are they averages? If deterministic, please state this.

2. Does the WI improvement at η=1.10 persist at block-group resolution, and at what magnitude?

### Suggestions

- **P1**: Clarify whether Table 1 results are deterministic or averaged across 30 seeds. If deterministic, state "all 30 seeds produce identical outcomes." If averaged, report SD.
- **P2**: Add a citation for the B.9 baseline values in Table 1.
- **P3**: Relabel Theorem 2 heading as "Lorenz feasibility (non-contiguous)" for clarity.

### Verdict

[X] Accept with Minor Revisions

**Rationale**: The Theorem 2 scope remark resolves my Round 2 P1 concern. The Scope of Claims section and Nevada explanation are the right additions. The remaining items are P1 (seed determinism clarification — one sentence) and P2/P3 that can be addressed at the journal stage.

**Score: 3.6 / 4.0** (up from 3.2)
