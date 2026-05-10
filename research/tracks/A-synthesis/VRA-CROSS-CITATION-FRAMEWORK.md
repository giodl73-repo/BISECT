# VRA Cross-Citation Framework
**Cross-track document** — addresses B2.2 board item.
**Last updated**: 2026-05-09

---

## The Four VRA Layers

VRA compliance in this program is addressed at four distinct levels.
Each layer feeds the next; cross-citations must follow this dependency chain.

| Layer | Paper(s) | What it addresses | Key result |
|-------|----------|-------------------|------------|
| **Structure** | T.7 (VRASection) | Algorithm for generating majority-minority districts | 42% minority VAP threshold enables district formation |
| **Empirical** | D.0–D.3 | Threshold analysis, compactness tradeoff, n-way comparison | 42% threshold (r=0.78, p<1e-08); +69 district surplus |
| **State Legislative** | F.6 | VRASection at state house scale | 5/5 seed stable, same 42% threshold |
| **Ensemble** | G.13 (VRA-aware ReCom) | Ensemble chains preserving majority-minority districts | Accepted 3.8/4; `--search vra-recom` |

---

## Required Cross-Citations

### F.6 → D.1 (42% threshold)

F.6 must cite D.1 for the 42% threshold it applies at state house scale.
Required text in F.6 §3:

> The 42\% threshold is derived from congressional-scale analysis in
> D.1~\citep{dellaLibera2026vraThreshold} (r=0.78 across 43 states,
> p$<$1e-08); we apply the same threshold at state house scale under the
> scale-invariance assumption in Section~3.1.

**Status**: [ADD TO F.6 §3 methodology section]

### G.13 → D.5 (Bloc Voting Methodology)

G.13 cites D.0 as the complementary structure-layer tool. It should also cite D.5's
bloc voting methodology as the empirical basis for VRA chain certification.
Required text in G.13 §2:

> The chain's VRA preservation criterion is calibrated using the bloc voting
> methodology of D.5~\citep{dellaLibera2026blocVoting}, which provides the
> empirical basis for identifying majority-minority districts that satisfy all
> three Gingles preconditions.

**Status**: [ADD TO G.13 §2 related work section]

### D.3 → T.7 (VRASection algorithm)

D.3 (compactness-VRA tradeoff) studies the tradeoff empirically but should
cite T.7 as the source of the algorithm that enforces the VRA constraint.
Required text in D.3 §2:

> The VRA constraint in our Pareto frontier analysis is implemented via
> VRASection~\citep{dellaLibera2026vraSection} (Paper T.7), which enforces
> minimum minority VAP thresholds within the recursive bisection tree.

**Status**: CHECK — may already be in D.3 §2.

### T.7 → D.0 (empirical validation)

T.7 (VRASection algorithm) should cite D.0 as its empirical validation.
D.0 provides the 50-state empirical evidence that VRASection achieves
majority-minority district formation at the constitutional threshold.

**Status**: CHECK — may already be in T.7.

---

## Post-Callais Cross-Citations

All VRA papers must acknowledge *Louisiana v. Callais* (2026).

| Paper | Required text |
|-------|---------------|
| D.1 | "Callais does not alter the prong 1 geographic feasibility standard; its primary impact is on prong 3 racial-partisan disentanglement." ✓ Already added |
| D.3 | Same Callais note. ✓ Already added |
| D.5 | Implements the post-Callais WLS+HC3+Holm methodology. ✓ Already present |
| F.6 | Add: "At state house scale, the Callais disentanglement requirement applies identically to the congressional analysis in D.5." |
| G.13 | Add: "The VRA-aware chain produces plans that satisfy the geographic compactness prong; Callais disentanglement of racial/partisan bloc voting is addressed in D.5." |
| T.7 | Add one-sentence Callais reference in §5 (legal implications). |

---

## Action Items Summary

Priority order (based on board B2.2):

1. **F.6 §3**: Add 42% threshold citation to D.1 with scale-invariance language
2. **G.13 §2**: Add D.5 bloc voting citation
3. **F.6**: Add Callais sentence
4. **G.13**: Add Callais sentence  
5. **T.7**: Add Callais sentence in legal section
6. Verify D.3 → T.7 and T.7 → D.0 cross-citations exist
