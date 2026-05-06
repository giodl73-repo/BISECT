# Review: three-layer-compositor.md
**Reviewer**: DATUM (peer-reviewer)
**Date**: 2026-05-05

---

## What's accurate

The guide is honest about the research status of layer combinations: it clearly distinguishes "studied combinations" from "available in the CLI but have not yet been evaluated systematically." This is exactly the kind of scope qualification DATUM requires. The note that `geographic` is "the only weight mode that has been validated across all 50 states and all three census years" is a model of appropriate scope limitation. The statement that "cross-layer combinations involving VRA weights require careful legal review under the post-Callais disentanglement requirement" correctly names the evidentiary constraint without overstating it. The claim that the content-derived seed is "publicly verifiable" is accurate and is the kind of reproducibility anchor that makes results falsifiable.

---

## P1 — Required fixes

**Section "Layer 3 — Search", `convergence` description**: The guide states T=600 "satisfies the Districting Integrity Act (DIA) statutory stopping criterion." The DIA is referenced without citation, without a year, and without any indication of which jurisdictions have adopted it. Readers reviewing this guide for a specific litigation context cannot verify this claim. If the DIA is a model statute or a specific enacted law in one or more states, that must be stated explicitly. If it is a proposed standard, it must be labeled as such. A court submitting document based on this guide would face immediate challenge if "DIA" cannot be resolved to a specific statute.

**Section "Layer 1 — Structure", `prime-factor` row**: The claim that "prime-factor structure is the geographic completion of the Huntington-Hill apportionment" invokes a conceptual framing without evidence. The guide asserts this as a fact in a description table. Whether prime-factor bisection is meaningfully "completing" Huntington-Hill in a formal mathematical sense, or whether this is an analogy, is not established here. A researcher citing this claim needs the argument, not the assertion. The conceptual relationship should be stated as "analogous to" or should reference the paper (B.11) that defends the claim formally.

**Section "Orthogonality" table**: The table lists `ratio-optimal | geographic | bisection-ensemble` as Paper H.1, but BisectionEnsemble is described in the search layer as a combination of METIS topology with ReCom sampling. It is not clear whether this combination has been empirically evaluated or is only theoretically composed. The table presents it alongside empirical results from B.1-B.14 without flagging that H.1 may be a planned or partially evaluated paper.

**Section "Layer 2 — Weights", `proportional` row**: The claim that ncon=2 vertex weights "simultaneously" constrain population and partisan vote balance gives the impression that both constraints are equally enforced at every bisection. In fact, METIS prioritises the tighter constraint when conflicts arise (as stated correctly for AreaSection but not here). This asymmetry is material for anyone using ProportionalSection results in a legal argument about proportionality.

---

## P2 — Suggested improvements

The "Short version" paragraph correctly summarises the three layers but does not mention that not all layer combinations have been validated. Adding a sentence such as "Not all combinations have been empirically evaluated; see the Orthogonality table for studied combinations" would set expectations immediately.

The `compact-polsby` structure row deserves a note that this mode runs a per-node Polsby-Popper computation at each bisection level, which is computationally more expensive than other structure modes. Runtime expectations would help practitioners plan.

---

## Score: 3/4

The guide is well-structured and appropriately scoped on most claims. The DIA citation gap is the most serious issue — a guide used in legal proceedings cannot reference a statute by acronym without a full citation. The Huntington-Hill analogy and the proportional constraint asymmetry are secondary but should be clarified before court use.
