---
reviewer: George Karypis
round: 1
score: 3
date: 2026-05-06
---

## Review: PercentileSweep — Statutory Choice of Legal Posture in Algorithmic Redistricting

### Strengths

The algorithmic formalization is largely well-executed. Definition 1 is precise: the index-based seed walk $s_i = \lfloor\text{SHA-256}(\texttt{census\_id} \,\|\, i)\rfloor \bmod 2^{31}$ is clearly stated, the sorting criterion is unambiguous (ascending $\ECnorm$, lowest cut first), and the rank selection rule $r = \lfloor p \times T \rfloor$ is a standard empirical quantile with no off-by-one ambiguity for the special cases $p \in \{0.0, 0.25, 0.5, 0.75, 1.0\}$ with $T = 101$. The pseudocode in Algorithm 1 matches the definition faithfully. The paper correctly notes that PS is always-$T$ whereas ConvergenceSweep terminates early, and correctly identifies that the independence of the $T$ seeds enables parallelism — this is a genuine implementation advantage worth having in the formal record.

The relationship to ConvergenceSweep is drawn cleanly. The three differences in §3.3 (stopping rule, return criterion, seed independence) are accurate and sufficient. The paper does not overclaim that PS is computationally superior for $p = 0.0$; it correctly notes that CS is more efficient for that special case. This shows appropriate precision.

The bisection-space vs. ReCom-ensemble distinction (§3.5) is the paper's most important algorithmic contribution for this audience, and it is correct: plans in the $T$-seed PS distribution share the same bisection-tree topology determined by the prime factorisation of $k$. The leaf-level METIS seed varies; the tree structure does not. This is a crucial point that affects how the "median" claim is interpreted and is stated correctly.

### P1 Items (must resolve before acceptance)

**P1-A: Seed chain vs. independent seeds — notation conflict with B.16.**
The paper states in §2.1 that CS advances "one seed at a time" from $s_0$, implying a sequential walk $s_0, s_0+1, s_0+2, \ldots$ in B.16. But §3.2 of the present paper uses independent per-index hashes: $s_i = \lfloor\text{SHA-256}(\texttt{census\_id} \,\|\, i)\rfloor$. These are not the same seed sequence. Is the B.16 seed walk an increment walk ($s_0 + i$) or a re-hash at each index? If it is the former, then PS's seed construction is a design departure from CS — not merely a different stopping rule — and this architectural difference must be stated explicitly. If B.16 also uses re-hashing, the description in §2.1 is misleading. The paper cannot claim PS is "a direct generalisation of CS" while changing the seed-generation mechanism without acknowledging this change.

**P1-B: Floor in the SHA-256 application is undefined.**
The definition writes $\lfloor\text{SHA-256}(\texttt{census\_id} \,\|\, i)\rfloor$. SHA-256 outputs 256 bits, not a real number. The floor operation is meaningless unless a byte-to-integer convention is stated (e.g., big-endian truncation to 64 bits before mod $2^{31}$). The NIST reference is cited but does not specify this truncation. Either write $s_i = \text{SHA-256}_{\text{int}}(\cdot) \bmod 2^{31}$ with a brief note that the hash digest is interpreted as a big-endian unsigned integer, or adopt the notation used in FIPS 180-4 explicitly. As written, the formula is not reproducible.

**P1-C: $T = 101$ is too small to make "0.5-seat bound" a strong claim.**
With $T = 101$ plans, the bisection distribution is sparsely sampled. The CV-below-2% finding from B.7 was derived from a 10,000-seed sweep per state. Sampling 101 draws from a distribution with CV = 2% gives a standard error of approximately $2\% / \sqrt{101} \approx 0.2\%$ on the mean — but the extremes of the 101-draw sample are not well-characterized. In particular, $p = 1.0$ (rank 100 out of 101) is the maximum of 101 draws, which could sit at an unusual tail location relative to the true distribution. The text acknowledges $T$ sensitivity as a future work item but makes the "0.5-seat bound is not violated for any state" claim as if $T = 101$ is sufficient. The justification requires at minimum a comparison between the $p = 1.0$ plan at $T = 101$ vs. $T = 601$ to verify that the highest-rank plan is not an outlier. This is a gap between the empirical claim and the evidence presented.

### P2 Items (recommended improvements)

**P2-A:** The METIS imbalance factor $u_{\text{factor}} = 1\%$ used in experiments differs from the canonical congressional tolerance of $\pm 0.5\%$ stated in other series papers. Confirm or explain the discrepancy.

**P2-B:** The CV figure in §4.3 references "B.7 CV $< 2\%$" but the actual spread data in Table 2 shows $\Delta\ECnorm$ of 0.004–0.007. Provide the implied CV figures directly from Table 2 to allow the reader to verify the claim without external reference.

**P2-C:** Algorithm 1, line 4: the call $\ar{}(s_i)$ does not specify whether the seed is passed to the top-level bisection call or to every nested METIS call in the recursive tree. This matters for reproducibility; B.11 should be cross-referenced for the seeding protocol.

### Score

**3 / 4** — Accept with revisions. The core algorithm is sound and the formalisation is largely correct, but the seed-chain notation conflict (P1-A) and the floor-of-hash ambiguity (P1-B) would prevent reproduction by an independent implementer. These are fixable in one revision.
