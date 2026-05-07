---
reviewer: George Karypis
round: 2
score: 4
date: 2026-05-05
---

## Review: PercentileSweep — Statutory Choice of Legal Posture in Algorithmic Redistricting (Round 2)

### Response to Round 1 Concerns

**K-P1-A (seed chain vs. independent seeds)**: Partially addressed. Section §3.2 now explicitly states that PS uses per-index re-hashes rather than sequential increment walks, and the advantages of this design (parallelism, independent verifiability) are stated clearly. The phrase "direct generalisation" in §2.4 is still present ("PS is a direct generalisation of CS"), but the seed-generation difference is now explained in §3.2, so the contradiction is resolved in substance even if not by removing the phrase. Acceptable.

**K-P1-B (floor-of-hash ambiguity)**: Resolved cleanly. The added sentence in §3.2 — "The SHA-256 output is truncated to the least-significant 31 bits: bytes 0–3 of the big-endian digest are interpreted as an unsigned 32-bit integer, then masked to $2^{31} - 1$ (i.e., the high bit is cleared)" — is precise and independently reproducible. This is exactly what was needed. The truncation convention is now unambiguous.

**K-P1-C ($T = 101$ sensitivity check for GA and NC)**: Not resolved. The paper still does not include the $T = 601$ sensitivity check for the $p = 1.0$ plan in GA and NC. The §4.4 note acknowledges that TX and CA sweeps are deferred to H.2, but GA and NC are presented as fully swept states, and the tail behavior of the $p = 1.0$ plan at $T = 601$ remains unchecked. This is now the sole unresolved P1 item from my Round 1 review. I am willing to accept conditioned on the $T$ sensitivity being addressed in H.2 and a forward-reference being added here.

### New Observations

The revised §4.2 note on TX and CA is a significant improvement. The explicit labeling of these entries as extrapolated B.11 data rather than confirmed PS sweeps corrects a material weakness in Round 1. The four-state confirmed insensitivity claim is now cleanly separated from the two-state conjecture.

The SHA-256 truncation specification (§3.2) is now implementation-grade. This was the most important reproducibility fix in the paper and it has been done correctly.

### Remaining Concerns

**P1 (carried from Round 1, conditional)**: The $T = 601$ sensitivity check for GA (CV = 4.3%) and NC (CV = 3.8%) remains unrun. I accept that the four-state claim is robust with high probability given B.7's characterisation of the tail, but the paper should add a forward reference: "A $T$-sensitivity check for GA and NC at $T = 601$ is included in the H.2 companion study." This single sentence resolves my remaining objection.

**P2 (new)**: The revised §4 heading "Partisan Insensitivity: Seat Variation Summary" (§4.4) is clear but the subsection label `\label{subsec:bound}` is referenced in `main.tex` but the old abstract text still contains "vary by at most 0.5 seats across $p$" in §1.3 (the Introduction, §1.3). The abstract in `main.tex` was updated; the introduction body (§1.3, line 58--59) retains "vary by at most 0.5 seats." This should be reconciled.

### Score

**4 / 4** — Accept. The two core reproducibility concerns (seed chain, hash truncation) are resolved. The $T$-sensitivity gap can be closed by a single forward-reference sentence. The paper is now implementation-grade and the insensitivity claim is correctly scoped.
