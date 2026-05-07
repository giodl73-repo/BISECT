# Review Synthesis — Round 2
**Paper**: H.1: BisectionEnsemble
**Round**: 2
**Date**: 2026-05-05
**Reviewers**: Karypis, Rodden, Duchin, Stephanopoulos, Liang

---

## Verdict Summary

| Reviewer | Round 1 | Round 2 | Score | Key Remaining Concern |
|---|---|---|---|---|
| Karypis | Major Revision | Minor Revision | 3.0/4 | Theorem statement/proof mismatch; "bipartition failure" undefined |
| Rodden | Minor Revision | Minor Revision | 3.0/4 | GA/PA factual error in §4.4; MultiSeedMETIS absent |
| Duchin | Major Revision | Minor Revision | 3.0/4 | GerryChain config unspecified; citation preference |
| Stephanopoulos | Minor Revision | Accept w/ Minor Revisions | 3.5/4 | Article I §2 language; GerryChain config |
| Liang | Major Revision | Major Revision | 2.5/4 | GA/PA factual error; single-run results undisclosed; hardware spec |

**Average score**: 3.0/4 — meets the ≥3.0 target.

**Consensus**: 4/5 reviewers move to Minor Revision or Accept. 1/5 (Liang) maintains Major Revision on reproducibility grounds. The GA/PA factual error is flagged independently by Rodden and Liang and must be fixed before resubmission.

---

## What Was Successfully Addressed

- **TX causal account** (Duchin P1): Fully resolved. The geometric-constraint framing, pair-reselection distinction, and throughput-not-structure characterization are all correct and accepted.
- **Theorem 1 proof** (Karypis P1): Substantially improved. The three-part structure (existence, positive probability, finite expected time) is now explicit.
- **Rucho citation** (Stephanopoulos P0): Fully resolved. The state-court framework (*LWV v. Commonwealth*, *Common Cause v. Lewis*, *Harper v. Hall*) replaces the inverted Rucho citation.
- **Data citation** (Liang/Rodden P0): Fully resolved. VEST replaces CCES; interpolation method specified.
- **Ergodicity claim** (Duchin/Karypis P1): Fully resolved. "To the extent the local chain mixes" hedging is accepted by both reviewers.
- **METIS seed protocol** (Stephanopoulos P1): Fully resolved. CSPRNG-before-compute protocol with pre-run logging.
- **Percentile sensitivity reframe** (Stephanopoulos/Rodden P1): Fully resolved. Prospective sweep commitment replaces retrospective NC stability claim.
- **Locality qualification** (Duchin S-5): Fully resolved. Depth-indexed locality with root-node caveat.
- **Scope of "no failure" claim** (C-3): Fully resolved across Abstract, Introduction, and Conclusion.
- **Wilson's caveat** (Karypis C-2): Fully resolved in Proposition 2 proof.
- **$m=0$ edge case** (Karypis S-6): Fully resolved in Definition 1.
- **Ensemble logging** (Stephanopoulos S-7): Fully resolved with --log-ensemble flag.

---

## Must Fix Before Publication (All Reviewers Agree)

### F-1: GA/PA Factual Error in Section 4.4
**Flagged by**: Rodden, Liang
**Severity**: Must fix — factual error
**Action**: Line 144 reads "Partisan outcomes are stable across percentile levels $p$ for NC, GA, PA (zero seat change)." GA and PA data are not in the paper. Either (a) remove "GA, PA" and restrict stability claim to NC, or (b) add GA/PA data to Table 3. Option (a) is lower effort.

---

## Should Fix Before Publication

### F-2: Theorem 1 Statement/Proof Mismatch
**Flagged by**: Karypis
**Severity**: Precision — affects credibility
**Action**: Theorem 1 statement says "$O(|V_H|)$ steps in expectation" but the proof establishes "$O(\log(1/\delta))$ resamples with probability $\geq 1-\delta$." Either update the theorem statement to match the proof's probabilistic formulation, or add a sentence converting the bound to an expected-steps form.

### F-3: GerryChain Configuration Not Specified
**Flagged by**: Duchin, Stephanopoulos, Liang
**Severity**: Reproducibility — makes Table 1 unchallengeable-but-also-unverifiable
**Action**: Add a footnote or appendix paragraph specifying GerryChain version (pip-installed tag or commit), tolerance $\varepsilon$, ReCom variant (reversible/non-reversible), initial plan, and random seed for the TX experiment. This is low-effort and eliminates a legal-evidence vulnerability.

### F-4: Hardware/Software Specification for Table 4
**Flagged by**: Liang
**Severity**: Reproducibility
**Action**: Add one sentence to Section 4.1 specifying CPU model, clock speed, Rust compiler version, and METIS version.

### F-5: "Bipartition Failure" Not Formally Defined
**Flagged by**: Karypis, Duchin
**Severity**: Minor precision
**Action**: Add a one-sentence definition in Section 1 or Section 3: "We say a ReCom step fails if no balanced cut edge exists in the sampled spanning tree; a chain stalls if the acceptance rate falls below a threshold $\alpha$."

---

## Deferred to Future Work (Accepted by Reviewers)

- **Multi-run statistics** (C-5): Add explicit statement in Section 4.1 that all Tables 2-4 are single-run observations and variance is deferred to future work.
- **MultiSeedMETIS baseline** (S-2): Accept as future work with a discussion paragraph in Section 4.2 noting what the absence implies.
- **Broader state sample** (S-4): Accept as future work.

---

## Revision Checklist

### Must Do
- [ ] F-1: Remove GA/PA from Section 4.4 line 144 (or add data)

### Should Do
- [ ] F-2: Align Theorem 1 statement with proof's probabilistic bound
- [ ] F-3: Add GerryChain configuration footnote for Table 1
- [ ] F-4: Add hardware/software spec sentence to Section 4.1
- [ ] F-5: Add formal definition of "bipartition failure" in Section 1

### Acknowledge in Paper
- [ ] Add sentence in Section 4.1 that Tables 2-4 are single-run observations; distribution across runs is future work
- [ ] Add paragraph in Section 4.2 noting MultiSeedMETIS comparison is deferred to future work

---

## Path to Acceptance

The paper has addressed all P1 items and most P0 items successfully. The remaining blockers are: (1) the GA/PA factual error (must fix, low effort), (2) Theorem 1 statement precision (should fix, low effort), and (3) GerryChain configuration disclosure (should fix, low effort). With these three fixes and the two "acknowledge" notes, the paper should reach acceptance with 4/5 reviewers recommending Accept or Minor Revision, and Liang moving from Major Revision to Minor Revision once the single-run limitation is explicitly disclosed and the factual error is corrected.
