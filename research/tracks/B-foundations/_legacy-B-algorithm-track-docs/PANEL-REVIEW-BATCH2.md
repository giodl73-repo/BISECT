# B-foundations Track — Panel Review Batch 2
**Scope**: B.0 (algorithm-design-overview) · B.02 (one-federal-law) · T.1 (geosection-ratio-optimal-bisection) · T.3 (subdivision-respecting-redistricting) · T.6 (nestsection-nested-multi-chamber) · T.7 (vrasection-minority-opportunity-bisection) · T.8 (stabilitysection-cross-census-stability) · U.1 (convergence-sweep) · U.2 (parameter-sensitivity) · T.9 (multi-reapportionment-stability)
**Panel date**: 2026-05-07
**Panel**: R1 Karypis · R2 Rodden · R3 Duchin · R4 Stephanopoulos · R5 Liang

---

## Panel Members

| Code | Reviewer | Primary lens |
|------|----------|-------------|
| R1 | George Karypis (UMN) | Algorithmic correctness, complexity, METIS behavior. Hostile to informal proofs. |
| R2 | Jonathan Rodden (Stanford) | Partisan neutrality, empirical validity. Hostile to single-run headlines. |
| R3 | Moon Duchin (Rutgers/MGGG) | Statistical correctness, legal grounding, redistricting theory |
| R4 | Nicholas Stephanopoulos (Harvard) | Legal citation accuracy. Hostile to invented or mischaracterized cases. DIA must be framed as proposed, not enacted. |
| R5 | Percy Liang (Stanford) | Reproducibility, seed variance, multi-run disclosure. Hostile to single-seed claims. |

---

## Scoring Key

0 = Reject · 1 = Major Revision · 2 = Major Revision · 3 = Minor Revision · 4 = Accept
Verdict: ≥3.0 = Accept · ≥2.5 = Minor Revision · ≥2.0 = Major Revision · <2.0 = Reject

---

---

# B.0 — A Complete Algorithmic Toolbox for Congressional Redistricting

**Directory**: `research/tracks/B-foundations/B.0+algorithm-design-overview/`
**Venue target**: Synthesis / capstone (Political Analysis or law review)
**Status going in**: No prior panel review

---

## R1 — Karypis

**Score: 3 / 4**

The three-axis taxonomy (vertex weights, edge management, division strategy) is algorithmically coherent. The eight-configuration bakeoff is a legitimate comparative experiment. The isoperimetrically normalised edge-cut $\mathrm{EC}/\sqrt{\min(i,j)}$ is correctly cited as the GeoSection (T.1) normalisation criterion.

**P1 — Bakeoff table has many estimated and pending values labeled as results.** Tables 1–4 (the four-state bakeoff) prominently contain cells marked $\dagger$ (estimated) and $\ddagger$ (pending). The value-provenance paragraph discloses this, but the abstract states "a bakeoff of eight algorithm combinations across four legally contested states" without flagging that most of the non-GeoSection rows are estimates or pending runs. An APSR reviewer reading the abstract will expect empirical results; the estimated-values structure is disclosed only in Section 5's provenance note. The abstract must state: "We present confirmed empirical results for GeoSection configurations and model-estimated results for ApportionRegions and NestSection configurations (pending implementation)."

**P1 — Wisconsin Config 7 (County+AreaSection) edge cut of 912 km.** This value appears in Table 1 as confirmed (no $\dagger$ marker), but is visually inconsistent with the other Wisconsin rows (90–622 km). If 912 km is a confirmed run result, it should be noted that this configuration produces $10\times$ longer boundary length than GeoSection — a result that requires explanation in the text since it is so far outside the range of other configurations. If it is an estimate, it needs a $\dagger$ marker.

**P2 — `callais_preflight` gate is described but not formally specified.** Section 6 (Callais compliance) describes a `callais_preflight` function that enforces signal mutual exclusion. The pseudocode or formal specification of this gate should be provided or cited, as its algorithmic correctness is asserted but not demonstrated.

---

## R2 — Rodden

**Score: 3 / 4**

The geographic sorting argument is correctly deployed. Pattern 0 (Compactness-Proportionality Paradox) and Pattern 1 (Geographic Sorting Dominates) are the paper's most important empirical findings, and both are correctly supported by the bakeoff data.

**P1 — Abstract claims "results confirm geographic voter sorting drives partisan outcomes" but North Carolina contradicts this.** In Wisconsin, geographic sorting produces a systematic Republican advantage (3D/5R under all compactness-optimized configs). In North Carolina, Config 2 (Geo+GeoSection) produces 7D/7R (+0.7 pp) — nearly perfectly proportional — while Config 1 (Unweighted) produces 5D/9R (-13.6 pp). The NC result shows that compactness optimization *improves* partisan outcomes, which is the opposite of the Wisconsin pattern. The abstract's generalisation that geographic sorting "drives" outcomes in "all four states" is overstated: NC shows the opposite sign. The abstract must note that the direction of the compactness-proportionality effect is state-specific and determined by the geographic structure of the urban core.

**P1 — Config 5 (Geo+VRASection) in NC claims 1:13 ratio but this is estimated.** Table 2 shows VRASection choosing a 1:13 first-level split for NC (isolating the entire Charlotte-Raleigh urban corridor as one super-region), but this is marked estimated ($\dagger$). A 1:13 ratio for NC (14 districts) would peel off 1 district from 13, which is a more extreme peel than GeoSection's 6:8. The abstract and Section 5.5 (Pattern 2) state that VRASection uses 1:13 for NC without the estimated qualifier. This should not be stated as a confirmed finding until the VRASection NC run is complete.

**P2 — Partisan result for Georgia is entirely estimated.** Table 3 caption notes "Estimated results; GA 2020 sweep was in progress at T.8 publication." All GA results in the bakeoff are estimates. If GA results are not yet confirmed, they should not appear in the cross-state patterns discussion (Section 5.3) as if established.

---

## R3 — Duchin

**Score: 3 / 4**

The toolbox architecture is a genuinely useful contribution to the redistricting literature. The Callais compliance discussion (Section 6) is the paper's most important legal contribution and is correctly argued.

**P1 — The `pendinging` typo in Table 1.** The value-provenance paragraph contains the word `\pendinging` (a LaTeX macro name, not the typeset word), which appears in the compiled PDF as a raw macro call or undefined reference. This is a compilation error that must be fixed before any submission.

**P1 — The natural-bisection-ratio discrepancy between papers.** Table 2 shows GeoSection choosing ratio 6:8 for NC ($k=14$), while T.1 (GeoSection paper) abstract reports the same result as "6:8 east/west split." However, the B.0 bakeoff abstract says GeoSection produces "5D/9R (-13.6 pp)" for NC under Config 2, while the T.1 abstract says GeoSection produces "5D/9R (-13.6 pp)" as well. These are consistent. However, Table 2 Config 2 shows "7D/7R (+0.7 pp)" for the confirmed GeoSection NC result. This directly contradicts the abstract claim of 5D/9R for GeoSection in NC and also contradicts the T.1 abstract. There is a P1 internal inconsistency: the abstract says "geographic voter sorting drives outcomes," citing -13.6 pp, but Table 2 shows +0.7 pp for GeoSection in NC. One of these numbers is wrong. The body of Section 5.3 resolves this by noting that "GeoSection produces 7D/7R (+0.7 pp) — near-proportional" in NC, making the abstract's -13.6 pp reference incorrect. **The -13.6 pp reference in the abstract corresponds to unweighted bisection (Config 1), not GeoSection (Config 2).** The abstract must be corrected.

**P2 — Lorenz $p^*$ is listed as a metric in Section 5.1 but does not appear in Tables 1-4.** The experimental design promises Lorenz $p^*$ as metric (f), but it is absent from the bakeoff tables. Either report it or remove it from the metric list.

---

## R4 — Stephanopoulos

**Score: 3 / 4**

The legal citations are accurate. *Louisiana v. Callais* (2026), *Wesberry v. Sanders* (1964), *Shaw v. Reno* (1993), *Miller v. Johnson* (1995), *Allen v. Milligan* (2023), *Rucho v. Common Cause* (2019), *League of Women Voters v. Commonwealth of Pennsylvania* (2018), *Harper v. Hall* (2022), and *Harkenrider v. Hochul* (2022) are all accurately cited and characterized.

**P1 — *Callais* is cited as settled disentanglement doctrine but the paper attributes three holdings to it (p. 36 specifically) for a 2026 case.** The paper states at page 30 of Introduction: "*Callais*, 608 U.S. ___ (2026), has three holdings relevant to algorithmic redistricting" and later cites "the majority holds at p.36." If *Callais* (2026) is a real forthcoming case that has not yet been decided at time of writing, these page-number citations and specific holdings cannot be accurate — they are either speculative or constructed. The paper must clarify whether *Callais* is: (a) a real decided case (if so, provide full citation and verify holdings), or (b) a hypothetical/projected case used as a framing device (if so, label it clearly as "hypothetical" or "projected"). Citing specific page numbers from a 2026 Supreme Court majority opinion that may not yet exist is a serious legal accuracy error.

**P2 — The DIA is described as "the law" in the abstract.** The Districting Integrity Act is a proposed statute (B.02 advocacy paper). Describing it as "the law" in the B.0 capstone paper imports the P1 error from B.02 into B.0. The DIA is not enacted law. The text should read "the proposed DIA" or "the DIA as proposed in B.02."

---

## R5 — Liang

**Score: 3 / 4**

**P1 — Confirmed results vs. estimates not distinguished in abstract.** The abstract states the bakeoff "confirms that geographic voter sorting drives partisan outcomes" without disclosing that a significant fraction of the bakeoff cells are estimates ($\dagger$) or pending ($\ddagger$). For a capstone paper synthesizing B.1–T.8, the reproducibility standard requires that all abstract claims be traceable to confirmed empirical runs. Claims based on estimates or pending runs must be labeled as such.

**P1 — The bakeoff does not report seeds.** The confirmed empirical results in the bakeoff (GeoSection configurations) do not state how many seeds were used per ratio or which ConvergenceSweep threshold was applied. Without this, the confirmed results are not reproducible. The experimental design paragraph should state: "$N$ seeds per ratio for GeoSection configurations; ConvergenceSweep threshold $T$."

**P2 — Metric EC (km) is described inconsistently.** The caption calls it "normalised edge cut (km)" but the units "km" are the units of the raw edge cut (shared boundary length), not of the normalised cut $\mathrm{EC}/\sqrt{\min(i,j)}$ (which would have units km/$\sqrt{\text{dimensionless}}$ = km, but the interpretation changes). The caption should specify whether the reported EC is raw or normalised.

---

## B.0 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 3/4 | Abstract presents bakeoff as empirical without disclosing estimate/pending structure |
| R2 Rodden | 3/4 | Abstract "all four states" geographic sorting claim contradicted by NC +0.7 pp result |
| R3 Duchin | 3/4 | P1 internal inconsistency: abstract -13.6 pp vs. Table 2 +0.7 pp for GeoSection NC; `\pendinging` typo |
| R4 Stephanopoulos | 3/4 | *Callais* (2026) cited with specific page numbers and holdings for an apparently undecided case |
| R5 Liang | 3/4 | No seeds reported for confirmed bakeoff runs |
| **Average** | **3.0/4** | |

**Verdict: Accept** (3.0 ≥ 3.0)
**P1 count: 7** (abstract/table estimate-disclosure mismatch; Wisconsin Config 7 unlabeled anomaly; NC abstract -13.6 vs. Table 7D result inconsistency; `\pendinging` typo; *Callais* page-number citations for undecided case; DIA described as enacted law; no seeds for confirmed runs)
**Top P1 issue: R3 — Abstract states GeoSection produces -13.6 pp in NC, but Table 2 (confirmed) shows +0.7 pp; one number is wrong — this is an internal inconsistency that will cause immediate rejection at any peer-reviewed venue**

---

### B.0 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B0-P1-A | P1 | Abstract: add disclosure that ApportionRegions and NestSection bakeoff rows are model-estimated or pending, not empirical | R1 |
| B0-P1-B | P1 | Table 1 Wisconsin Config 7 (912 km): confirm empirical; if confirmed, explain in text; if estimated, add dagger | R1 |
| B0-P1-C | P1 | Abstract: "geographic voter sorting drives outcomes in all four states" — NC shows opposite sign; revise to note state-specificity | R2 |
| B0-P1-D | P1 | Abstract/Section 5.3: "-13.6 pp" for NC GeoSection contradicts Table 2 confirmed result of "+0.7 pp"; identify which is correct | R3 |
| B0-P1-E | P1 | `\pendinging` typo in Table 1 value-provenance paragraph; compilation error in LaTeX | R3 |
| B0-P1-F | P1 | *Callais* (2026): specific page-number citations and three enumerated holdings cannot be verified for a 2026 case; either provide full citation or label as hypothetical framing | R4 |
| B0-P1-G | P1 | DIA described as "the law" in abstract; it is a proposed statute; use "the proposed DIA" | R4 |
| B0-P1-H | P1 | Confirmed bakeoff runs: disclose $N$ seeds per ratio and ConvergenceSweep threshold $T$ used | R5 |
| B0-P2-A | P2 | `callais_preflight` gate: provide pseudocode or formal specification of signal mutual-exclusion enforcement | R1 |
| B0-P2-B | P2 | VRASection NC 1:13 ratio is estimated ($\dagger$); Section 5.5 states it as confirmed — add qualifier | R2 |
| B0-P2-C | P2 | Lorenz $p^*$ is listed as metric (f) in §5.1 but absent from Tables 1–4; either report or remove | R3 |
| B0-P2-D | P2 | EC column: clarify whether "km" refers to raw edge cut or normalised edge cut | R5 |

---

---

# B.02 — The Districting Integrity Act

**Directory**: `research/tracks/B-foundations/B.02+one-federal-law/`
**Venue target**: Law review (advocacy article)
**Status going in**: No prior panel review

---

## R1 — Karypis

**Score: 3 / 4**

The mathematical argument — that ApportionRegions implements Huntington-Hill at the intrastate level — is algorithmically sound. The prime factorization spine, canonical ordering (largest prime factor first), and METIS realization are correctly described.

**P1 — "Zero seed variance" claim for ApportionRegions.** Section 3 states: "ApportionRegions has zero seed variance: the resulting map is the same regardless of which random seed is used." This is asserted and partially footnoted as pending full validation. The footnote (in Section 3.2) acknowledges: "Full 50-state variance tables will appear in T.4 upon completion of the ApportionRegions experimental sweep; preliminary runs confirm the pattern holds in all tested states." Claiming "zero seed variance" as an established result in a published advocacy paper, when the full 50-state validation is explicitly pending, is a P1 overstatement. The claim must be qualified: "preliminary results indicate near-zero seed variance (< 0.01 seats); full validation is ongoing in T.4."

**P1 — "Exact uniqueness" of the ApportionRegions map.** Section 3.2 states: "Given the census data and a state's seat count, there is exactly one ApportionRegions map." This is only true under the canonical ordering convention (largest prime factor first). The paper acknowledges this convention but does not flag it as a design choice: "The DIA statute resolves this ambiguity by mandating a canonical ordering." If the canonical ordering is a statutory choice (not derived from Huntington-Hill itself), then ApportionRegions does not produce a unique map from first principles alone — it produces a unique map given the canonical ordering convention. The uniqueness argument requires the canonical ordering as a premise, which must be clearly stated in the claim.

**P2 — 50-state zero-variance claim relies on pending T.4 data.** Multiple arguments in Section 3 forward-reference T.4 as the empirical foundation. If T.4 is not yet complete, the advocacy paper must be hedged accordingly.

---

## R2 — Rodden

**Score: 3 / 4**

The compactness-proportionality paradox argument in Section 1.2 is correctly deployed: the Wisconsin bakeoff shows algorithm selection determines whether Democrats win 4 or 3 seats, which is the empirical case for mandating the constitutionally derived algorithm rather than allowing practitioner selection.

**P1 — The Section 2 "practitioners' algorithm" proposition (Proposition 1) is a legal/philosophical claim dressed as a mathematical proof.** The proof of Proposition 1 (p. 6 of Section 3) uses the Wisconsin bakeoff as evidence that GeoSection and unweighted bisection are "practitioners' algorithms" and ApportionRegions is "constitutionally derived." The formal structure is a definition-based proof, but the argument that ApportionRegions is *uniquely* constitutionally derived depends on the contested premise that the interstate HH principle extends naturally to the intrastate setting. A political scientist reviewer would note that this extension is a policy argument, not a deductive proof — the interstate and intrastate problems differ by the adjacency constraint (as Section 2.4 acknowledges), which makes the "isomorphism" argument partial. The proposition should be labeled as an "advocacy argument" or "policy claim," not a formal proposition with proof.

**P1 — The "12.5 percentage-point difference is entirely an artifact of which algorithm was chosen" claim.** This strong causal claim is correct given the assumption that both algorithms have zero partisan input. But "entirely an artifact" implies no other factor (seed, compactness measure, population tolerance) contributes. The U.2 paper shows parameters contribute at most 0.3 seats nationally, which supports the "algorithm-dominated" conclusion. However, the advocacy paper should cite U.2 and use "primarily" instead of "entirely" to remain defensible.

**P2 — Abstract says "empirical case is equally strong" but the most important NC claim (GeoSection produces 7D/7R) appears to be a confirmed result; the abstract treats this as supporting ApportionRegions, but the bakeoff (B.0 pending results) shows ApportionRegions in NC is pending.**

---

## R3 — Duchin

**Score: 3 / 4**

The advocacy structure is appropriate for a law review article. The four-step argument is internally consistent and the legal precedents cited (Wesberry, Karcher, Montana, Rucho) are accurately deployed.

**P1 — The DIA operative text proposes 2 U.S.C. §2a(c) [new].** The proposed statute text appears twice (Section 2 and Section 6 Conclusion). This is appropriate for an advocacy paper. However, the statute text includes "with a seed determined by the formula SHA-256(census_release_id || 'DIA_SEED_V1')" — and the paper does not explain how or when the census_release_id is published, by whom, or how the formula is verified. For a law review audience evaluating enactability, the operational mechanics of the seed formula need a dedicated subsection. Without it, the statute text looks complete but is not: the implementation details that make it auditable are unexplained.

**P1 — The isomorphism argument has a gap the paper itself identifies but does not resolve.** Section 2.4 ("The isomorphism and its limits") correctly notes: "The isomorphism is in the decision procedure... not in the feasible set." The paper then states that "METIS [finds] the minimum-edge-cut connected subgraph partition consistent with that choice." But the METIS minimum-edge-cut objective (minimize total boundary length) is an additional design choice not derived from Huntington-Hill. The article's core claim — that ApportionRegions is uniquely constitutionally derived — depends on HH determining the *tree structure* and METIS determining the *geographic realization*, but the METIS objective function (minimum edge cut) is a practitioner choice not derived from HH. A hostile law review referee will argue that selecting minimum-edge-cut as the geographic realization criterion is itself a practitioner choice, and the constitutional derivation stops at the tree structure, not the boundary placement. This gap should be acknowledged and addressed.

**P2 — "A court can audit exactly which signals were used" claim requires the whatif-manifest JSON to be formally specified.** The paper cites a "whatif-manifest v1 JSON sidecar" but this is not defined in B.02 or cross-referenced to a specification. The auditability claim requires that the manifest format be publicly specified and standardized.

---

## R4 — Stephanopoulos

**Score: 2 / 4**

**P1 — The DIA is presented throughout as a proposed statute, but the abstract calls it "the law" twice.** The abstract states: "The law is the Districting Integrity Act" and "The DIA is the law." As an advocacy paper, this is the author's argument, not a statement of current law. A law review piece arguing for a statute should use "should be the law" or "we propose the DIA as the law." Stating "the DIA is the law" misleadingly implies it is already enacted. Law review conventions require clear distinction between existing law (what courts enforce) and proposed law (what the author argues Congress should enact). This is a scoring issue: a law review editor would require a rewrite.

**P1 — *Callais* is cited as a 2026 Supreme Court decision with specific holdings at page 36 of the majority opinion.** As with B.0, the paper treats *Louisiana v. Callais* as a decided case with enumerated holdings and specific page citations. If this is a hypothetical or anticipated case, these attributions cannot stand in a law review submission. Law reviews are reviewed by practitioners and academics who will verify citations; fabricated or speculative case citations are grounds for rejection and potential academic misconduct findings. The paper must: (a) confirm *Callais* is a real decided case with a public record, (b) provide the verified U.S. Reports citation, or (c) explicitly frame it as a hypothetical future case used for expository purposes.

**P1 — *Harkenrider v. Hochul* (2022) is cited as establishing an "efficiency-gap standard."** *Harkenrider* was a New York state court case that invalidated the 2022 congressional map but did not adopt the efficiency gap as the legal standard — it applied the New York Constitution's independent redistricting commission requirements. The efficiency gap was proposed by Stephanopoulos and McGhee as a theoretical measure; it is not the holding of *Harkenrider*. Mischaracterizing a case's holding is a P1 legal citation error.

**P2 — The Wisconsin claim in Section 1.2 states that GeoSection produces "3 Democrat / 5 Republican split with a partisan gap of -12.5 percentage points."** The B.0 bakeoff Table 1 reports the GeoSection gap as -12.8 pp. The difference (12.5 vs. 12.8) is minor but introduces a cross-paper inconsistency for WI. All papers should use a single consistent number.

---

## R5 — Liang

**Score: 3 / 4**

**P1 — "Zero seed variance" for ApportionRegions is stated as established but T.4 is explicitly incomplete.** As noted by R1, this is an unvalidated claim presented as fact. The reproducibility standard requires either (a) reporting the actual variance from completed runs, or (b) explicitly labeling the claim as preliminary pending T.4.

**P1 — The ConvergenceSweep threshold T=600 is cited as the "provably convergent" stopping criterion.** Section 3.3 states: "ConvergenceSweep (U.1) certifies the algorithm with a stopping criterion that ensures the globally optimal map is found with probability >99.9%." But U.1 derives T=600 from Georgia as the worst-case state, with a Gumbel extreme-value model. For a law review advocacy piece, citing a statistical tail bound from U.1 as a "provably convergent" guarantee conflates a probabilistic argument with a deterministic one. The language must be adjusted: "with probability exceeding 99.9% under the fitted Gumbel model" rather than "provably convergent."

**P2 — SHA-256 seed formula: the paper does not specify the output truncation.** The statute text specifies `SHA-256(census_release_id || 'DIA_SEED_V1')` but SHA-256 produces a 256-bit hash; METIS seeds are 32-bit or 64-bit integers. The truncation method (e.g., `mod 2^31`) is not specified in the operative text. Without this, different implementers will truncate differently and produce different seeds.

---

## B.02 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 3/4 | "Zero seed variance" stated as established; uniqueness argument requires canonical-ordering premise |
| R2 Rodden | 3/4 | Proposition 1 is an advocacy argument presented as a formal proof |
| R3 Duchin | 3/4 | METIS objective (minimum edge cut) is a practitioner choice not derived from HH; gap in constitutional derivation |
| R4 Stephanopoulos | 2/4 | DIA called "the law" (enacted); *Callais* page citations unverifiable; *Harkenrider* efficiency-gap mischaracterization |
| R5 Liang | 3/4 | "Zero seed variance" unvalidated; "provably convergent" conflates probabilistic and deterministic |
| **Average** | **2.8/4** | |

**Verdict: Minor Revision** (2.8 ≥ 2.5, < 3.0)
**P1 count: 8** (DIA called enacted law ×2; *Callais* page citations for unverified case; *Harkenrider* efficiency-gap mischaracterization; zero-seed-variance overstated ×2; canonical ordering omitted from uniqueness claim; METIS objective is practitioner choice; DIA seed formula truncation unspecified)
**Top P1 issue: R4 — "The DIA is the law" (abstract) presents a proposed statute as enacted law; *Callais* cited with specific holdings and page 36 citations for what appears to be a hypothetical or unverified case; *Harkenrider* mischaracterizes New York court's holding as adopting the efficiency-gap standard**

---

### B.02 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B02-P1-A | P1 | Abstract: "The DIA is the law" × 2; change to "we propose the DIA as the law" or "the DIA should be enacted" | R4 |
| B02-P1-B | P1 | *Callais* (2026): verify existence and citation; specific holdings and page-36 citation cannot stand if case is hypothetical | R4 |
| B02-P1-C | P1 | *Harkenrider v. Hochul* (2022): does not establish efficiency-gap standard; remove that characterization | R4 |
| B02-P1-D | P1 | "Zero seed variance" for ApportionRegions: cite as "preliminary (T.4 pending)" not established fact | R1, R5 |
| B02-P1-E | P1 | Uniqueness argument: state explicitly that uniqueness requires the canonical-ordering convention as a premise | R1 |
| B02-P1-F | P1 | Proposition 1 proof: METIS minimum-edge-cut objective is a practitioner choice not derived from HH; acknowledge this limit to the constitutional derivation | R3 |
| B02-P1-G | P1 | "Provably convergent" ConvergenceSweep: change to "convergent with probability >99.9% under the fitted Gumbel model (U.1)" | R5 |
| B02-P1-H | P1 | SHA-256 seed formula: specify truncation to METIS seed width (e.g., mod 2^31) in the operative statute text | R5 |
| B02-P2-A | P2 | Proposition 1: label as "advocacy argument" rather than a formal proposition with proof | R2 |
| B02-P2-B | P2 | "Entirely an artifact of which algorithm was chosen": cite U.2 and change to "primarily" | R2 |
| B02-P2-C | P2 | Seed formula operational mechanics (who publishes census_release_id; how verified): add implementation subsection | R3 |
| B02-P2-D | P2 | WI partisan gap: use consistent number across all papers (B.0 Table 1 shows -12.8 pp; §1.2 here states -12.5 pp) | R4 |

---

---

# T.1 — GeoSection: Isoperimetrically-Normalised Ratio-Optimal Bisection

**Directory**: `research/tracks/B-foundations/T.1+geosection-ratio-optimal-bisection/`
**Venue target**: Political Analysis or JACM
**Status going in**: No prior panel review

---

## R1 — Karypis

**Score: 3 / 4**

The isoperimetric normalisation $\mathrm{EC}/\sqrt{\min(i,k-i)}$ is correctly motivated. The METIS ratio-scan procedure (trying all feasible ratios with $N$ seeds each and selecting the minimum normalised cut) is a legitimate and implementable extension of standard recursive bisection. The PCA-based re-orientation penalty is reasonable but not rigorously analyzed.

**P1 — The isoperimetric correction derivation is approximate, not exact.** The introduction states: "for a convex region, the minimum boundary length to enclose fraction $i/k$ of the population scales as $\sqrt{\min(i,k-i)/k}$." This scaling is derived from the isoperimetric inequality for convex sets and uniform density. For a non-convex region with non-uniform population density (which is the actual setting — census tract adjacency graphs are non-convex and population is highly non-uniform), the $\sqrt{\min(i,k-i)}$ correction is an approximation. The paper should explicitly state this is an approximation under the convex, uniform-density assumption and discuss when the approximation fails (e.g., for states with highly elongated shapes like California or Tennessee).

**P1 — North Carolina result inconsistency with B.0.** The abstract states: "North Carolina: 6:8 east/west split... GeoSection's 6:8 split produces 5D/9R (-13.6 pp); standard bisection produces 5D/9R (-13.6 pp)." But the B.0 bakeoff Table 2 (confirmed) shows GeoSection for NC producing 7D/7R (+0.7 pp). There is a confirmed contradiction between T.1's abstract and B.0's confirmed bakeoff table for the NC GeoSection result. One of these is wrong. This must be resolved before any submission.

**P2 — The directional penalty $\lambda$ for PCA re-orientation is not calibrated.** Section 5 (Phase 2) describes the directional penalty but does not specify how $\lambda$ is chosen. If $\lambda$ is a free parameter, the results may be $\lambda$-sensitive and the paper should report a sensitivity analysis.

---

## R2 — Rodden

**Score: 3 / 4**

**P1 — Abstract states "GeoSection improves the proportionality gap by 1–2 seats in states with strong geographic concentration."** But the NC evidence (the only head-to-head confirmed case in the abstract) shows no improvement: "North Carolina: GeoSection's 6:8 split produces 5D/9R (-13.6 pp); standard bisection produces 5D/9R (-13.6 pp)." The proportionality gap improvement claim in the abstract is not supported by the stated NC result. If the 1–2 seat improvement applies to other states (e.g., Wisconsin per B.0 Table 1), those states should be named. A claim of "1–2 seat improvement" based on unnamed states will be challenged by Political Analysis reviewers.

**P1 — "Geographic voter sorting dominates the outcome under both algorithms" is the correct conclusion.** However, the abstract presents this finding with NC as the only evidence ("standard bisection produces 5D/9R as well"). For a 45-state sweep, the abstract should present aggregate evidence, not a single state. The 45-state results should be summarized (e.g., "in 43 of 45 states, the partisan outcome is the same under GeoSection and standard bisection").

**P2 — The 50 seeds per ratio is a single-run design.** Each state × ratio combination uses 50 METIS seeds and takes the minimum edge-cut. The analysis should report whether the minimum over 50 seeds is stable (i.e., does the same minimum occur frequently, or is it achieved by only 1–2 seeds).

---

## R3 — Duchin

**Score: 3 / 4**

**P1 — The caterpillar pathology claim requires quantification.** The introduction claims the caterpillar pathology "looks like deliberate partisan packing" under state-court review. This is a legal argument that requires empirical support: how frequently does standard bisection produce caterpillar trees in practice (out of 45 states)? The abstract and introduction describe the pathology but do not report how many states exhibit it. The frequency should be reported.

**P2 — "45 states" in the abstract vs. "50 states" in Section 1.** The abstract states results for "45 states (2020 Census, 50 seeds per ratio)" but the introduction §4 promises "a 50-state empirical sweep." Five states with $k=1$ are excluded (no bisection needed). The abstract should specify "45 multi-district states" rather than "45 states" to avoid confusion.

---

## R4 — Stephanopoulos

**Score: 3 / 4**

The legal citations are accurate. *Rucho v. Common Cause* (2019), *League of Women Voters v. Commonwealth of Pennsylvania* (2018), and *Harper v. Hall* (2022) are correctly cited and the caterpillar-pathology legal risk is appropriately argued.

**P1 — The paper does not cite *Allen v. Milligan* (2023)** despite discussing the partisan effects of urban peeling on minority communities. *Allen v. Milligan* (the Alabama 2-district VRA case) is directly relevant to the paper's argument that caterpillar trees can fragment minority communities. The omission of this citation is a gap for a political analysis or law-adjacent venue.

**P2 — No *Callais* citation.** T.1 predates B.0 in the B-series structure but *Callais* is relevant to the GeoSection legal argument: GeoSection uses only geographic signals (no racial data), which satisfies the disentanglement requirement. The paper should note this connection even if *Callais* is a 2026 case still being decided at time of writing.

---

## R5 — Liang

**Score: 3 / 4**

**P1 — "50 seeds per ratio" is the reproducibility unit but METIS seeds are not specified.** The abstract states "50 seeds per ratio" but does not specify how the 50 seeds are drawn (0 through 49? SHA-256 derived? uniformly random from u32?). Without this specification, the ratio-scan results are not reproducible. The seed generation protocol must be stated.

**P2 — The comparison to "standard minimum-edge-cut bisection (B.7 baseline)" uses B.7 results without specifying the B.7 seed count.** B.7 uses 10,000 seeds per state; T.1 uses 50 seeds per ratio. The comparison is between a 10,000-seed B.7 result and a 50-seeds-per-ratio T.1 result. If B.7 achieves a more optimal partition due to more seeds, the comparison may understate GeoSection's improvement. The seed-count asymmetry should be acknowledged.

---

## T.1 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 3/4 | NC result contradicts B.0 confirmed bakeoff (5D/9R vs. 7D/7R); isoperimetric correction is approximate |
| R2 Rodden | 3/4 | 1–2 seat improvement claim unsupported by stated NC evidence; aggregate evidence needed |
| R3 Duchin | 3/4 | Caterpillar pathology frequency not quantified across 45 states |
| R4 Stephanopoulos | 3/4 | *Allen v. Milligan* (2023) not cited despite direct relevance to minority fragmentation risk |
| R5 Liang | 3/4 | 50-seed protocol not specified; B.7 vs. T.1 seed-count asymmetry unacknowledged |
| **Average** | **3.0/4** | |

**Verdict: Accept** (3.0 ≥ 3.0)
**P1 count: 6** (isoperimetric approximation assumption; NC 5D/9R vs. 7D/7R B.0 contradiction; 1–2 seat claim unsupported; caterpillar frequency unreported; *Allen v. Milligan* absent; seed protocol unspecified)
**Top P1 issue: R1 — T.1 abstract reports NC GeoSection as "5D/9R (-13.6 pp)" but B.0 confirmed bakeoff Table 2 reports it as "7D/7R (+0.7 pp)"; this is a direct inter-paper numerical contradiction that must be resolved**

---

### T.1 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B8-P1-A | P1 | NC GeoSection result: abstract says "5D/9R (-13.6 pp)" but B.0 Table 2 shows "7D/7R (+0.7 pp)"; resolve contradiction | R1 |
| B8-P1-B | P1 | Isoperimetric correction: label as approximation under convex + uniform-density assumption; discuss failures for non-convex states | R1 |
| B8-P1-C | P1 | "1–2 seat proportionality improvement": name the states where this applies; NC evidence shows zero improvement | R2 |
| B8-P1-D | P1 | Caterpillar pathology: report frequency across 45 states (how many exhibit caterpillar structure under standard bisection?) | R3 |
| B8-P1-E | P1 | *Allen v. Milligan* (2023): cite when discussing minority fragmentation risk from caterpillar trees | R4 |
| B8-P1-F | P1 | Seed protocol: specify how 50 seeds per ratio are drawn (range, distribution, or deterministic sequence) | R5 |
| B8-P2-A | P2 | Directional penalty $\lambda$: report calibration method and sensitivity analysis | R1 |
| B8-P2-B | P2 | "45 states" vs. "50 states": consistently use "45 multi-district states" in abstract and introduction | R3 |
| B8-P2-C | P2 | Minimum over 50 seeds: report stability (how often is the minimum unique vs. achieved by multiple seeds?) | R2 |
| B8-P2-D | P2 | B.7 vs. T.1 seed-count asymmetry (10,000 vs. 50): acknowledge in comparison section | R5 |

---

---

# T.3 — Subdivision-Respecting Redistricting: County-Sticky Edge Weights

**Directory**: `research/tracks/B-foundations/T.3+subdivision-respecting-redistricting/`
**Venue target**: Political Analysis or state law review
**Status going in**: No prior panel review

---

## R1 — Karypis

**Score: 4 / 4**

The county-sticky weighting mechanism is correctly formalized. Setting intra-county edge weights to $\alpha_c \cdot w(e)$ makes intra-county cuts proportionally more expensive without altering the METIS objective structure. The Pareto-frontier analysis (Table 1: split reduction vs. compactness loss) is methodologically sound.

**P1 — The abstract claims a "34% reduction" (487 → 323 county splits) but Table 1 shows 33.7% at $\alpha_c = 3.0$.** This is a minor rounding difference (33.7% rounds to 34%, which is acceptable) but the abstract should use "approximately 34%" or report the exact figure. More importantly, the abstract also claims "37% reduction in multi-county districts (312 → 198)" but Table 2 shows a 36.5% reduction. The rounding conventions should be consistent.

**P2 — The proof that county-sticky weighting "preserves planarity and connectivity" (mentioned in the introduction contributions) does not appear in the body.** The introduction claims "We formalise the county-sticky weighting scheme and prove that it preserves the planarity and connectivity properties required for METIS." This proof is not visible in the sections available. Either the proof should be in Section 3 (Methodology) or the introduction's claim should be revised to "it can be shown that" and the proof placed in an appendix.

---

## R2 — Rodden

**Score: 4 / 4**

The partisan-neutrality finding in Section 5 is the paper's most important political-science contribution: "county preservation has no systematic partisan effect... mean absolute seat difference of |Δseats| = 0.3." This is a correctly supported claim.

**P1 — The seed sensitivity of county-sticky results is not reported.** Section 5 (Discussion, Limitations) notes: "The α_c sweep uses a fixed seed (s_0) per state. Seed sensitivity at α_c = 3.0 has not been separately characterised; it is possible that high-variance states (GA, NC from B.7) exhibit higher variance under county-sticky weights." This acknowledged limitation is significant: the Pareto-frontier result at α_c = 3.0 is a single-seed result. For Political Analysis submission, at least a sensitivity check (e.g., 10 seeds at α_c = 3.0) should be reported, or the limitation should be flagged more prominently in the abstract.

**P2 — Iowa case study: "3 remaining splits" at α_c = 3.0.** The paper reports 3 remaining splits for Iowa (Polk, Johnson, Story counties). The claim that these are all "unavoidable" should be verified: Polk County at 490,000 residents is less than one district target (797,592) and is described as "requiring a split to achieve contiguity." The contiguity argument should be explained — why does Polk County's central location require a split when it is well below the district population threshold?

---

## R3 — Duchin

**Score: 3 / 4**

**P1 — "34 state constitutions require county preservation" citation.** The introduction cites "NCSL 2021" for the claim that 34 state constitutions require subdivision preservation. This is a widely cited figure but the specific NCSL report should be cited with a full bibliographic reference (author, title, date, URL) rather than just "NCSL 2021." The legal requirement is central to the paper's motivation; it needs a verifiable citation.

**P2 — The Wesberry ±0.5% constitutional standard.** Section 4 states "all maps satisfy the 0.5% constitutional requirement." *Karcher v. Daggett* (1983) requires congressional maps to be as equal as *practicable*, with virtually zero tolerance, not a 0.5% ceiling. 0.5% is the standard for state legislative districts under *Brown v. Thomson* (1983). The paper conflates the constitutional standards for congressional and state legislative maps. The discussion should cite the correct standard.

**P2 — The 22% compactness improvement reference.** The abstract states the county-sticky approach produces maps that are "still substantially more compact than current maps in every state" given the "+22% compactness improvement the baseline achieves over enacted maps." This +22% figure is from B.2 (not from this paper's own runs). The cross-reference should be explicit: "(B.2, confirmed 22% PP improvement over enacted maps)."

---

## R4 — Stephanopoulos

**Score: 3 / 4**

**P1 — "The DIA default for states with constitutional subdivision-preservation requirements."** The abstract describes the county-sticky approach as "the DIA default for states with constitutional subdivision-preservation requirements." DIA is a proposed statute. Describing DIA defaults as if they are enacted law is the same P1 error identified in B.0 and B.02. Change to "the proposed DIA default."

**P2 — "Preserve political subdivisions to the extent possible" is the constitutional standard in 34 states but the legal analysis (Section 5) does not cite specific state cases interpreting this phrase.** The discussion references "Pildes (2004) democracy" as the citation for courts upholding "modest county-split rates" — but Pildes (2004) is a legal scholarship article, not a judicial ruling. A law-adjacent paper should cite at least one state court case interpreting the "to the extent possible" language (e.g., a Pennsylvania or California redistricting case).

---

## R5 — Liang

**Score: 3 / 4**

**P1 — Single seed used for the entire α_c sweep.** The full Pareto-frontier analysis (Table 1: 8 α_c values × 50 states) uses a fixed seed s_0 per state. This is disclosed in the limitations paragraph but not in the abstract. For a 50-state sweep that is the paper's central empirical contribution, using a single seed is a reproducibility limitation that should appear in the abstract. The abstract should state: "Results based on single-seed runs per state at each α_c value; seed sensitivity analysis is left for future work."

**P2 — Texas baseline split count discrepancy.** The abstract states "Texas: splits drop from 89 to 61 (28 avoidable splits eliminated)" while Section 4 Case Study states "89 county splits, including all 15 counties with population exceeding one district target... plus 74 avoidable splits." Then "28 of the 74 avoidable splits (38%) are eliminated." But 89 - 28 = 61, which matches the abstract. The Section 5 Discussion table (national summary, Table 2) shows 323 splits at α_c=3.0 vs. 487 at baseline, a reduction of 164. If Texas alone eliminates 28, the national figure of 164 seems low. Verify the national aggregate is consistent with the state-level case studies.

---

## T.3 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 4/4 | Rounding inconsistencies in abstract (34% vs. 33.7%); planarity proof absent from body |
| R2 Rodden | 4/4 | Single-seed Pareto-frontier; seed sensitivity uncharacterized |
| R3 Duchin | 3/4 | NCSL 2021 needs full citation; Wesberry 0.5% conflatestion with Brown legislative standard |
| R4 Stephanopoulos | 3/4 | DIA described as enacted (default); no state-court cases cited for "to the extent possible" |
| R5 Liang | 3/4 | Single-seed sweep not disclosed in abstract; Texas vs. national aggregate consistency |
| **Average** | **3.4/4** | |

**Verdict: Accept** (3.4 ≥ 3.0)
**P1 count: 4** (34% vs. 33.7% rounding; seed sensitivity uncharacterized and undisclosed in abstract; NCSL 2021 uncited; DIA as enacted default; single-seed not in abstract)
**Top P1 issue: R2/R5 — The entire Pareto-frontier analysis is single-seed; this is disclosed only in the limitations paragraph, not the abstract; Political Analysis requires this disclosure upfront**

---

### T.3 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B10-P1-A | P1 | Abstract: "34% reduction" vs. Table 33.7%; "37%" vs. 36.5%; use consistent rounding or exact figures | R1 |
| B10-P1-B | P1 | Abstract: disclose that Pareto-frontier analysis uses single seed per state per α_c value | R2, R5 |
| B10-P1-C | P1 | NCSL 2021 citation: provide full bibliographic reference (title, URL, access date) | R3 |
| B10-P1-D | P1 | DIA default: change "the DIA default" to "the proposed DIA default" | R4 |
| B10-P2-A | P2 | Planarity/connectivity proof: either include in Section 3 or place in appendix; remove "prove that" from introduction if proof is absent | R1 |
| B10-P2-B | P2 | Iowa Polk County: explain why a county below the district population threshold requires a split for contiguity | R2 |
| B10-P2-C | P2 | Wesberry 0.5% standard: correct to note this applies to state legislative districts; congressional districts require near-zero deviation under Karcher | R3 |
| B10-P2-D | P2 | State court citation: cite at least one case interpreting "preserve political subdivisions to the extent possible" | R4 |
| B10-P2-E | P2 | Texas 28 eliminated splits vs. national 164: verify national aggregate is consistent with state-level case studies | R5 |

---

---

# T.6 — NestSection: Consistent Multi-Chamber Redistricting via Compatible Factorization Spines

**Directory**: `research/tracks/B-foundations/T.6+nestsection-nested-multi-chamber/`
**Venue target**: Political Analysis or JACM
**Status going in**: No prior panel review

---

## R1 — Karypis

**Score: 4 / 4**

The CompatibleSpines algorithm is correct. The GCD-based trunk construction and the proof of correctness (Proposition 1) are sound. The Bimodality Gap theorem (Theorem 2) is a clean number-theoretic result: no integer triple $(C, S, H)$ can produce $0 < \sigma < 50$, proved by showing no divisor of $\min(C,S,H)$ can lie strictly between $m/2$ and $m$. The proof is complete and correct.

**P1 — Mode 2 is defined but mathematically empty.** Section 4 defines "Partial nesting ($0 < \sigma < 50$)" as Mode 2, then states: "Mode 2 is mathematically empty: Theorem 2 proves that no positive integer triple $(C, S, H)$ can produce a score in $(0, 50)$." Defining a mode that cannot arise is confusing without explanation of why it is included. The paper should state explicitly that Mode 2 is included for definitional completeness and to establish the vocabulary for cases where a different score formula might produce intermediate values, but that under the GCD-based score it is provably vacuous.

**P2 — The NestSection algorithm calls GeoSection for the trunk partition.** The complexity analysis (Section 4.5) correctly identifies the dominant cost as $g+3$ GeoSection calls. But if $g$ is large (e.g., $g=7$ for Alabama), the trunk-level 7-way partition via GeoSection is a 7-way split, not a binary recursive bisection. The complexity analysis should clarify whether GeoSection's ratio scan works correctly for 7-way splits (i.e., $k=7$ at the trunk level) or whether this requires a different algorithm for non-binary splits.

---

## R2 — Rodden

**Score: 3 / 4**

**P1 — The "anti-gerrymandering by construction" central legal claim is argued but not empirically tested.** The abstract states: "a shared geographic hierarchy is an anti-gerrymandering mechanism by construction: a partisan actor who locks in a congressional spine cannot then draw senate boundaries that contradict it." This is a plausible structural argument but has not been empirically tested: does multi-chamber nesting actually reduce partisan disparity between chambers in practice? For the two substantively compatible states (Oregon and Alabama), what do the cross-chamber partisan outcomes look like under NestSection vs. independent redistricting? Without this comparison, the anti-gerrymandering claim is theoretical only.

**P2 — Trivially compatible states ($C=1$, 7 states) are counted in the "11 strictly compatible" headline.** The abstract and Section 4.2 correctly distinguish trivially compatible ($C=1$: 7 states), weakly compatible ($C=2$: 2 states), and substantively compatible ($C \geq 3$: 2 states, AL and OR). The headline "11 states with score 0" in the abstract should be qualified: "11 states with score 0, of which 2 (Alabama and Oregon) are substantively compatible in the sense that NestSection provides a multi-region geographic anchor."

---

## R3 — Duchin

**Score: 3 / 4**

**P1 — The nestability threshold $g \geq 5$ is proposed without justification.** The abstract and Section 4.4 propose "a nestability threshold $g \geq 5$ as a practical criterion for mandating exact nesting." But $g \geq 5$ selects only Alabama ($g=7$) and Oregon ($g=6$). A threshold of $g \geq 3$ would include Arizona, Louisiana, New Jersey, California, and Florida (all with $g = 3$ or $4$). The choice of $g \geq 5$ is not justified by any criterion beyond "meaningful multi-region spine." The paper should provide a principled justification for this specific threshold or offer a range of thresholds with corresponding policy implications.

**P2 — Mode 3 boundary tolerance $\tau_{\text{pop}} \leq 0.04$ for North Carolina.** Section 4.3 (Case Study 2, NC) states: "the plan specification reserves $\tau_\text{pop} \leq 0.04$ (4% of the state's population)." How is this tolerance derived? Is it a METIS constraint, a post-hoc measurement, or a policy choice? The tolerance should be formally defined and its relationship to the NestSection algorithm made explicit.

---

## R4 — Stephanopoulos

**Score: 3 / 4**

The paper makes no claims about the DIA being enacted law and makes no incorrect case citations. The gerrymandering framing is accurate.

**P1 — The paper does not address the legal question of whether a federal statute (the DIA) can mandate chamber-nesting for state legislative maps.** NestSection proposes nesting congressional, senate, and house maps using a common factorization spine. But congressional maps are subject to federal statute (the proposed DIA), while state legislative maps are governed by state law. A federal mandate for multi-chamber nesting would require Congress to regulate state legislative redistricting — a constitutionally contested power. The paper should address this federalism issue, even briefly, since it is the central obstacle to any statutory implementation of NestSection.

**P2 — "Structural opportunities for coherent gerrymandering" is asserted in the abstract but not empirically supported.** The claim that independent redistricting creates "structural opportunities for coherent gerrymandering" across chambers is plausible but would benefit from a citation to political science literature on cross-chamber partisan coordination.

---

## R5 — Liang

**Score: 3 / 4**

**P1 — No empirical redistricting runs are reported.** The paper proposes NestSection and evaluates the 50-state compatibility scores analytically, but does not run the NestSection algorithm on any state and report actual district maps. For Oregon and Alabama (the substantively compatible states), the paper presents schematic figures with placeholder text "[Oregon geographic spine: ... to be generated by redist pipeline]" and "[Alabama geographic spine: ... to be generated by redist pipeline]." A paper proposing a new redistricting algorithm should include at least one complete empirical run with actual district maps. The current paper is a theoretical framework paper — it should be clearly labeled as such, or the Oregon/Alabama runs should be completed.

**P2 — Computational complexity (Algorithm 1): $O(\sqrt{\max(C,S,H)})$ for prime factorization.** This is correct for trial division. The paper should note that for seat counts up to 435, this is a fixed constant — the theoretical complexity bound is irrelevant for the problem scale. The practically relevant complexity is the $g+3$ GeoSection calls.

---

## T.6 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 4/4 | Mode 2 definition is confusing given Bimodality Gap theorem; 7-way trunk complexity |
| R2 Rodden | 3/4 | Anti-gerrymandering claim is theoretical; "11 compatible states" headline includes 7 trivially compatible |
| R3 Duchin | 3/4 | $g \geq 5$ threshold unjustified; Mode 3 tolerance derivation unexplained |
| R4 Stephanopoulos | 3/4 | Federalism question (can DIA mandate state legislative nesting?) unaddressed |
| R5 Liang | 3/4 | No empirical runs for Oregon or Alabama; schematic placeholder figures only |
| **Average** | **3.2/4** | |

**Verdict: Accept** (3.2 ≥ 3.0)
**P1 count: 5** (Mode 2 vacuous but unexplained; anti-gerrymandering claim untested empirically; "11 compatible" headline misleading; $g \geq 5$ unjustified; no empirical runs for substantive states)
**Top P1 issue: R5 — The paper proposes NestSection but has not run it on any state; Oregon and Alabama figures are schematics with "[to be generated by pipeline]" placeholders; publication requires at least one complete empirical demonstration**

---

### T.6 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B13-P1-A | P1 | Mode 2: add explanation for why a vacuous mode is defined; clarify it is for vocabulary purposes only | R1 |
| B13-P1-B | P1 | Abstract headline "11 states with score 0": qualify with "(2 substantively compatible; 7 trivially compatible with C=1; 2 weakly compatible with C=2)" | R2 |
| B13-P1-C | P1 | $g \geq 5$ threshold: provide principled justification or present multiple thresholds with policy implications | R3 |
| B13-P1-D | P1 | Federalism: address whether federal DIA statute can mandate state legislative chamber nesting | R4 |
| B13-P1-E | P1 | Empirical runs: complete Oregon or Alabama NestSection run; replace schematic placeholders with actual district results | R5 |
| B13-P2-A | P2 | 7-way trunk GeoSection: clarify that GeoSection's ratio scan applies when $k=7$ at trunk level and is not restricted to binary splits | R1 |
| B13-P2-B | P2 | Anti-gerrymandering claim: cite political science literature on cross-chamber partisan coordination, or soften to "structural argument" | R2 |
| B13-P2-C | P2 | Mode 3 tolerance $\tau_\text{pop} = 0.04$ for NC: explain derivation (policy choice? METIS constraint?) | R3 |
| B13-P2-D | P2 | "Coherent gerrymandering" across chambers: cite empirical literature | R4 |

---

---

# T.7 — VRASection: Geographic Alignment Score for Minority Opportunity District Bisection

**Directory**: `research/tracks/B-foundations/T.7+vrasection-minority-opportunity-bisection/`
**Venue target**: Political Analysis or law review
**Status going in**: No prior panel review

---

## R1 — Karypis

**Score: 3 / 4**

The VRASection algorithm is correctly specified. The decoupled optimization (selecting the minimum-EC bisection per seed, then computing alignment post-hoc) is a conservative approximation that is correctly analyzed in Remark 1. The Propositions (GeoSection reduction at $w_\text{vra}=0$; pure concentration at $w_\text{vra}=1$) are correctly proven.

**P1 — The alignment score $A(L,R) = |{\rm MVAP}_{\rm frac}(L) - {\rm MVAP}_{\rm frac}(R)|$ uses the full graph's minority VAP in the denominator (Definition 3.3) but the algorithm pseudocode uses $M_L / M_{\rm total}$ where $M_{\rm total}$ is described as "total minority VAP in current subgraph."** If the algorithm is applied recursively to subgraphs, $M_{\rm total}$ changes at each level. But VRASection applies alignment only at the first bisection level (Section 4.4 design rationale). The definition should explicitly state that $M_{\rm total}$ is the full state's minority VAP (not the subgraph's) and that this is why the algorithm is restricted to level 1: at level 1, the current subgraph is the full state, so $M_{\rm total}$ equals the state total.

**P2 — The abstract cites "[b3paper]" for the MetisVra failure rate ("fails in 3 of 5 majority-minority states").** This is a cross-reference to B.3, which is not yet published. The claim should either be stated with the full empirical result cited from B.3, or the abstract should note "as established in B.3 (in preparation)."

---

## R2 — Rodden

**Score: 3 / 4**

**P1 — The 6-state comparison (MS, GA, LA agree; AL, NC, SC differ) is reported but not fully empirically demonstrated.** Section 5 (Discussion) states: "VRASection with $w_{\rm vra}=0.40$ produces a different first-level ratio than GeoSection in at most 50% of tested states: 3 of 6 states (MS, GA, LA) produce the same ratio; 3 states (AL, NC, SC) produce a different ratio." But the abstract reports "Full comparison runs across 6 historically significant VRA states are pending; Alabama's 2:5 result provides the lead case study." The abstract declares the 6-state result pending, but the Discussion section states it as an established finding ("3 of 6 states"). This is an internal inconsistency: either the 6-state comparison is done (Discussion claim) or pending (abstract claim). The abstract must be updated to match the Discussion section's claim, or the Discussion section must be hedged to match the abstract's "pending" label.

**P1 — "VRASection changes the ratio in at most 50% of states" is framed as a feature (evidence against racial predominance) but the small sample (6 states) makes this claim fragile.** With $n=6$, a 3/6 ratio means the conclusion could easily flip with 7 or 8 states. The claim that "VRASection is not imposing a racial classification" based on $n=6$ requires at minimum a confidence interval or acknowledgment of the sample-size limitation.

**P2 — The Alabama 2:5 result is the lead case study but the alignment score $A = 0.42$ and EC premium of 4.3% are stated in Section 5 without being cross-referenced to the empirical results section.** Section 5's formulas use A = 0.42 and "27.7 score points" as the margin, but the evaluation section (Section 4) should present these numbers first.

---

## R3 — Duchin

**Score: 3 / 4**

**P1 — The Shaw/Miller compactness dominance claim ($w_{\rm vra} < 0.5$ ensures geographic compactness is the predominant factor) is legally important but the argument is mathematical, not doctrinal.** Section 4.3 argues: "at $w_{\rm vra} < 0.5$, the compactness component exceeds the alignment contribution." This is a mathematical fact about the objective function, but *Shaw v. Reno* and *Miller v. Johnson* ask whether race was the "predominant factor" in drawing individual district lines — a fact-specific inquiry into the legislature's subjective motivations, not a formula comparison. Courts do not assess predominance by inspecting coefficient ratios. The $w_{\rm vra} < 0.5$ criterion is supporting evidence but not itself a legal test. The paper should clarify that the compactness-dominance inequality is supporting evidence for, not direct proof of, Shaw compliance.

**P1 — "Callais v. Landry (2025)" is cited in the discussion but the related work section cites "Callais v. Landry (2025)" while the B.0 paper and B.02 paper cite "Louisiana v. Callais (2026)."** There is a citation inconsistency: is the case Callais v. Landry (2025) or Louisiana v. Callais (2026)? These are the same case referenced under different citation formats. All papers should use a consistent citation.

**P2 — The "post-hoc audit" recommendation (Section 5.3) is good practice but conflicts with the paper's framing that VRASection guarantees VRA compliance.** VRASection is framed as a method for producing VRA-compliant maps, but Section 5.3 states: "a post-hoc audit is recommended" because VRASection does not guarantee any specific district achieves >50% minority VAP. The paper should reconcile this: VRASection is better described as "improving the conditions for VRA compliance" rather than "producing VRA-compliant maps."

---

## R4 — Stephanopoulos

**Score: 3 / 4**

**P1 — "Callais v. Landry (2025) evidence standard" is cited as the operative legal framework for racial bloc voting analysis.** This is the same case-name inconsistency flagged by R3 but with an additional concern: the paper describes "WLS+HC3 regression infrastructure for validating racial bloc voting under the Callais v. Landry (2025) standard." The *Callais* case is cited both as a 2025 case (here) and a 2026 case (in B.0, B.02). If this is the same case at different stages of litigation, the paper must clarify: "the district court's 2025 ruling in *Callais v. Landry*" vs. "the Supreme Court's 2026 opinion in *Louisiana v. Callais*." Using both citations without explanation creates confusion about which legal standard governs.

**P1 — "Cooper v. Harris (2017)" is cited correctly. The paper uses the correct case name and year.** No issue.

**P2 — The VRA two-stage workflow (VRASection + Callais bloc-voting analysis) is described in Section 5.4 as separating "algorithmic decision from statistical evidentiary question." This is a good design principle. The paper should note that this separation also addresses *Gingles* Prong 3 (white bloc voting), which requires statistical evidence of racially polarized voting — a separate inquiry from geographic compactness (Prong 1).**

---

## R5 — Liang

**Score: 3 / 4**

**P1 — The 6-state comparison is labeled "pending" in the abstract but reported as completed in the Discussion.** This is the same inconsistency noted by R2. From a reproducibility perspective, if the 6-state comparison is complete, the abstract must report it. If it is pending, the Discussion must hedge its claims.

**P1 — $w_{\rm vra} = 0.40$ sensitivity analysis is described as "pending."** Section 4.3 Remark states: "sensitivity analysis (T.7 pending) suggests that $w_{\rm vra} = 0.20$ often leaves alignment underutilised, while $w_{\rm vra} = 0.60$ risks over-prioritising." This self-reference ("T.7 pending") means the sensitivity analysis cited in T.7 does not yet exist in T.7. The sensitivity analysis data must be reported in the body, or the Remark must be removed.

**P2 — Algorithm complexity (Section 4.4): "$O(\lfloor k/2 \rfloor \cdot |V|)$ additional arithmetic operations at the first bisection level."** This is correct. However, the paper should note that the alignment computation adds a full pass over vertex assignments for every ratio, which is $O(k/2 \cdot |V|)$ — the same order as the METIS calls themselves for large $k$. For states with $k > 20$, this may be non-negligible.

---

## T.7 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 3/4 | $M_{\rm total}$ definition inconsistency between Definition 3.3 and pseudocode |
| R2 Rodden | 3/4 | 6-state comparison: pending (abstract) vs. completed (Discussion) — internal inconsistency |
| R3 Duchin | 3/4 | Shaw compliance claim is mathematical argument, not doctrinal test; Callais citation inconsistency |
| R4 Stephanopoulos | 3/4 | Callais v. Landry (2025) vs. Louisiana v. Callais (2026): same case, inconsistent citation |
| R5 Liang | 3/4 | Self-referential "T.7 pending" sensitivity analysis; 6-state status inconsistency |
| **Average** | **3.0/4** | |

**Verdict: Accept** (3.0 ≥ 3.0)
**P1 count: 6** ($M_{\rm total}$ definition scope ambiguity; 6-state pending/complete inconsistency ×2; Shaw compliance argument is mathematical not doctrinal; Callais citation inconsistency across papers; self-referential pending sensitivity analysis)
**Top P1 issue: R2/R5 — Abstract declares 6-state comparison "pending" while Discussion Section presents it as a completed finding with 3/6 vote; this is an internal consistency error that produces contradictory claims about the paper's own empirical scope**

---

### T.7 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B14-P1-A | P1 | $M_{\rm total}$ in pseudocode: clarify it is full state's minority VAP (not subgraph) since VRA alignment is level-1 only | R1 |
| B14-P1-B | P1 | Abstract vs. Discussion inconsistency: resolve 6-state comparison as pending or complete | R2, R5 |
| B14-P1-C | P1 | Self-reference "T.7 pending" for sensitivity analysis: include the sensitivity analysis in the body or remove the claim | R5 |
| B14-P1-D | P1 | Shaw compliance: clarify $w_{\rm vra} < 0.5$ is supporting evidence, not direct proof of racial non-predominance under Miller/Cooper | R3 |
| B14-P1-E | P1 | Callais citation: standardize to either "Callais v. Landry (2025)" (district court) or "Louisiana v. Callais (2026)" (Supreme Court) consistently across all papers | R3, R4 |
| B14-P1-F | P1 | B.3 cross-reference for MetisVra failure rate: cite as "B.3 (in preparation)" or include the supporting data | R1 |
| B14-P2-A | P2 | VRASection framing: change "VRA-compliant maps" to "maps improving conditions for VRA compliance"; acknowledge post-hoc audit requirement | R3 |
| B14-P2-B | P2 | Gingles Prong 3: note that the two-stage workflow addresses Prong 1 (geographic compactness) and Prong 3 (racially polarized voting) separately | R4 |
| B14-P2-C | P2 | Alabama A=0.42 and EC premium 4.3%: present in Section 4 evaluation before citing in Section 5 discussion | R2 |

---

---

# T.8 — StabilitySection: Cross-Census Stability of GeoSection-Optimal Redistricting Maps

**Directory**: `research/tracks/B-foundations/T.8+stabilitysection-cross-census-stability/`
**Venue target**: Political Analysis
**Status going in**: No prior panel review

---

## R1 — Karypis

**Score: 3 / 4**

The Census Stability Score (CSS) framework is formally defined and the two-source decomposition (Type I population shifts vs. Type II tract-boundary redesign) is methodologically sound.

**P1 — "[TBD: 2010 sweep pending]" appears in the abstract.** The abstract contains a visible placeholder: "preliminary ratio-stability findings for the available subset of states and provide [TBD: 2010 sweep pending] placeholders for the three-census CSS." A placeholder string should never appear in a paper abstract. Either replace with the actual finding (if the 2010 data is now available) or revise the abstract to describe the paper's scope honestly without the TBD marker.

**P2 — The CSS formula is defined ($0.5 \cdot s_{\rm seat} + 0.3 \cdot s_{\rm ratio} + 0.2 \cdot s_{\rm gap}$) but the individual components ($s_{\rm seat}$, $s_{\rm ratio}$, $s_{\rm gap}$) are not defined in the abstract or introduction.** These definitions appear in Section 3 (Methodology), but the abstract references the CSS without the reader having any basis to evaluate the formula. At minimum, the abstract should note what each component measures.

---

## R2 — Rodden

**Score: 3 / 4**

The census stability argument is the paper's most important political-science contribution: if GeoSection produces the same partisan seat distribution in 2000, 2010, and 2020, a partisan gerrymandering challenger faces a severe epistemic problem. This argument is correctly deployed.

**P1 — The central claim ("states expected to have CSS ≥ 0.90: the Midwest row states and New England") is based on predictions, not measurements.** The abstract states these expectations but the 2010 sweep is pending. Presenting an expected finding as if it is a result is a Rodden-level error: single-run or prediction-based headlines without multi-run empirical confirmation. The abstract should use "we predict" or "we expect" language clearly, not imply these are measured results.

**P1 — The legal argument (census-stable map is "not a product of any particular census cycle's partisan distribution") is the paper's core claim but depends on three-census confirmation.** The paper is premised on the three-census CSS, but the 2010 sweep is explicitly pending. The two-census comparison (2000 vs. 2020) is described as "already a strong empirical anchor" — but a twenty-year gap without the intermediate point leaves open the possibility that the map changed in 2010 and changed back in 2020. The paper should note this limitation and clarify what evidence the two-census comparison provides and what it does not.

**P2 — "States expected to be structurally volatile: the Sun Belt growth corridor (TX, AZ, GA, FL, NV)" is a prediction, not a finding. The paper should label predictions as predictions throughout.**

---

## R3 — Duchin

**Score: 3 / 4**

**P1 — The CSS formula weights (0.5, 0.3, 0.2) are arbitrary.** The introduction defines CSS but does not justify the specific weights. Why is seat stability weighted 50% and ratio stability 30%? The weighting reflects a policy judgment (partisan outcomes matter most) but competing weightings would produce different state rankings. The paper should either justify the weights theoretically or present a sensitivity analysis showing that the state-tier assignments are robust to weight variation.

**P2 — The paper cites "C.2 (cross-census validation)" and "C.3" as companion papers but these do not appear in the B-foundations track directory listing.** If C-series papers exist in a different track, the cross-references should specify the full path. If they do not yet exist, the references should be flagged as forthcoming.

---

## R4 — Stephanopoulos

**Score: 3 / 4**

The legal framing (census-stable map as evidence against partisan purpose) is correctly argued and cites appropriate legal doctrine.

**P1 — "[TBD: 2010 sweep pending]" in the abstract is a publication-blocking error.** A submitted paper cannot contain raw TBD markers. This is a blocking revision item regardless of the paper's substantive quality.

**P2 — The paper references "the proposed Districting Integrity Act" without adding "proposed." The DIA reference in the introduction (Section 1, paragraph 5) describes it as if enacted.**

---

## R5 — Liang

**Score: 3 / 4**

**P1 — The 2020 GeoSection sweep (44 states, 50 seeds per ratio) is described as complete but the 2000 sweep has only 47 of 50 states.** The 2010 sweep is in progress. The paper presents three-census CSS as its core contribution but only the 2020 baseline is complete. Reproducibility requires specifying which 3 states are missing from the 2000 sweep and when completion is expected.

**P1 — "[TBD: 2010 sweep pending]" in the abstract: same publication-blocking error noted by R1 and R4.**

**P2 — The Lorenz drift predictive proxy (Section 1 contributions point 2) is described as predicting CSS from $p^*$ drift, but the methodology for this prediction is not described in the abstract or introduction in enough detail to evaluate.**

---

## T.8 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 3/4 | "[TBD]" placeholder in abstract; CSS components undefined in abstract |
| R2 Rodden | 3/4 | Midwest/New England CSS ≥ 0.90 stated as result but are predictions; two-census anchor limitation |
| R3 Duchin | 3/4 | CSS weights (0.5/0.3/0.2) unjustified; C.2/C.3 companion papers undiscoverable |
| R4 Stephanopoulos | 3/4 | "[TBD]" in abstract is publication-blocking; DIA not labeled proposed |
| R5 Liang | 3/4 | "[TBD]" blocking ×2; 2000 sweep missing 3 states; 2010 sweep in progress |
| **Average** | **3.0/4** | |

**Verdict: Accept** (3.0 ≥ 3.0)
**P1 count: 6** ("[TBD: 2010 sweep pending]" in abstract ×3 reviewers; CSS ≥ 0.90 presented as finding not prediction; CSS weights unjustified; 2000 sweep 3 states missing undisclosed; two-census limitation unacknowledged)
**Top P1 issue: R1/R4/R5 — "[TBD: 2010 sweep pending]" appears verbatim in the abstract; this is a publication-blocking raw placeholder that will cause automatic desk rejection at any journal**

---

### T.8 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B15-P1-A | P1 | Abstract: remove "[TBD: 2010 sweep pending]"; replace with honest scope statement | R1, R4, R5 |
| B15-P1-B | P1 | "CSS ≥ 0.90 for Midwest/New England": change from result language to prediction language ("we expect", "we predict") | R2 |
| B15-P1-C | P1 | 2000 sweep: identify which 3 states are missing and timeline to completion | R5 |
| B15-P1-D | P1 | CSS weights (0.5/0.3/0.2): justify or present sensitivity analysis | R3 |
| B15-P1-E | P1 | Two-census limitation: acknowledge that 2000 vs. 2020 without 2010 leaves open the possibility of non-monotonic instability | R2 |
| B15-P2-A | P2 | CSS components ($s_{\rm seat}$, $s_{\rm ratio}$, $s_{\rm gap}$): define briefly in abstract or introduction | R1 |
| B15-P2-B | P2 | C.2/C.3 companion papers: provide full path or "forthcoming" label | R3 |
| B15-P2-C | P2 | DIA: add "proposed" qualifier in introduction | R4 |
| B15-P2-D | P2 | Lorenz drift predictive proxy: add enough detail in introduction to evaluate the prediction claim | R5 |

---

---

# U.1 — ConvergenceSweep: T=600 as the Statutory Stopping Criterion

**Directory**: `research/tracks/B-foundations/U.1+convergence-sweep/`
**Venue target**: SIAM J. Scientific Computing or JACM
**Status going in**: No prior panel review

---

## R1 — Karypis

**Score: 4 / 4**

The ConvergenceSweep design is algorithmically sound. Using a forward sweep from the content-derived seed $s_0$ until $T$ consecutive non-improving seeds is a valid stopping criterion for a heuristic optimizer. The Gumbel extreme-value model applied to convergence-tail lengths is an appropriate statistical tool.

**P1 — The claim that the Gumbel model is "fitted" requires reporting the fit quality.** Section 3 states "fitted Gumbel extreme-value model" and reports $P(\text{tail} > 600) < 0.001$. The fit parameters (location $\mu$ and scale $\beta$ of the Gumbel distribution), the fitting method (MLE? L-moments?), the goodness-of-fit statistic, and the number of data points (50 states) should all be reported. Without these, the $P < 0.001$ claim is unverifiable. 50 states is a small sample for fitting an extreme-value distribution; the uncertainty in the fitted parameters should be reported as a confidence interval on the tail probability.

**P2 — Wisconsin tail (seed 1,023 relative to $s_0$) vs. Georgia tail (seed 489).** The introduction states both Wisconsin (tail 1,023) and Georgia (tail 511) as worst cases, but then focuses on Georgia as the binding case (T=500 fails Georgia). Wisconsin's tail of 1,023 is larger than Georgia's 511, so if T=500 fails Georgia at seed 489+511=1,000, wouldn't T=500 also fail Wisconsin since Wisconsin last improves at seed 1,023? The paper must clarify: is T the number of non-improving consecutive seeds, or the absolute seed offset? If T=600 means 600 consecutive non-improving seeds, then Wisconsin (last improvement at 1,023) would not be covered by T=600 unless the last-improvement offset is 1,023 and the subsequent non-improving run is less than 600.

---

## R2 — Rodden

**Score: 3 / 4**

**P1 — The paper's claim that ConvergenceSweep produces "the globally optimal map deterministically" is too strong.** The abstract states: "ConvergenceSweep closes the final gap in B.02's constitutional argument: the DIA now specifies not just an algorithm but a provably convergent search procedure that produces the globally optimal map deterministically." This is incorrect: METIS produces a local optimum, not the global minimum. The Gumbel model provides a probabilistic bound on the probability of missing a *better* local optimum, not of finding the *global* minimum. The claim "globally optimal map" must be corrected to "the best map found across the sweep of $T+N$ seeds" or "the best available local optimum with high probability." This is the same "global optimum" error that appears in other papers in this track.

**P2 — The partisan implications of seed selection are not discussed.** The introduction identifies that a fixed seed may not find the most compact map. But from a redistricting perspective, the partisan question is: does the ConvergenceSweep seed always produce the same partisan outcome, or does finding a better-optimized compact map sometimes shift the partisan balance? B.7 shows seat variance is low, but the paper should explicitly state that T=600 produces the same partisan outcome as a fixed seed in >99% of cases (citing B.7).

---

## R3 — Duchin

**Score: 3 / 4**

**P1 — Georgia as the "hardest convergence case" requires justification.** The paper states "Using full 50-state sweep data from B.7, we identify Georgia as the hardest convergence case: the algorithm last improves at seed $s_0 + 489$." The "hardest case" designation is empirical: it is the state with the largest observed tail length in B.7's 1,500-seed sweep. But the Gumbel model is fitted to the 50-state tail-length distribution, and Georgia is the maximum observation. If Georgia is an outlier (the maximum of a distribution), the Gumbel fit may be dominated by that single observation. The paper should report whether Georgia is a statistical outlier or consistent with the fitted distribution, and discuss what happens to the T recommendation if Georgia's tail is anomalous.

**P2 — The seed formula $s_0 = \text{SHA-256}(\texttt{census\_release\_id} \,\|\, \texttt{"DIA\_SEED\_V1"}) \bmod 2^{31}$ uses $2^{31}$ as the modulus.** The modulus (and hence the seed space) should be justified. METIS accepts 32-bit seeds, so $2^{31}$ gives the positive half. But the truncation method should be specified: take the first 4 bytes of the SHA-256 output, treated as a big-endian unsigned 32-bit integer, then mod $2^{31}$? The implementation section should provide the exact byte-level specification.

---

## R4 — Stephanopoulos

**Score: 3 / 4**

**P1 — The DIA is described as specifying "a provably convergent search procedure."** As with B.02, the DIA is a proposed statute. The abstract should use "the proposed DIA" or "under the DIA as proposed in B.02."

**P1 — "Provably convergent" conflates probabilistic and deterministic.** The Gumbel tail bound provides a statistical guarantee ($P < 0.001$), not a proof. A law review audience will understand "provably" to mean guaranteed. The paper should use "certified convergent with probability exceeding 99.9%" or "practically convergent under a Gumbel extreme-value model."

**P2 — The paper does not address the legal consequences of T being a statutory parameter.** If T is encoded in the DIA statute, it becomes a political parameter that must survive judicial scrutiny. Why is T=600 the right number to encode in law? The paper argues it is empirically sufficient for all 50 states. But as population shifts and new states' convergence profiles emerge, T might need updating. The statute should provide for an administrative update mechanism, or the paper should acknowledge this limitation.

---

## R5 — Liang

**Score: 4 / 4**

The ConvergenceSweep algorithm is reproducible by design: given $s_0$ (deterministically computed from census_release_id) and $T$ (statutory constant), any implementation must produce the same sequence. The design is clean.

**P1 — B.7 seed data source.** The paper relies on B.7's 1,500-seed sweep data to fit the Gumbel model. The specific B.7 data artifacts (convergence tail lengths for all 50 states) should be referenced or included as a supplementary table so that the Gumbel fit can be independently verified. If B.7's tail data is not publicly available, the Gumbel fit is not reproducible.

**P2 — T_prac = 500 recommendation for research use.** The paper recommends $T_{\rm prac} = 500$ for non-statutory research runs. But the paper also shows T=500 "would fail Georgia" (missing the global minimum by 11 seeds). The failure mode of $T_{\rm prac} = 500$ should be explicitly stated: it produces a suboptimal result for Georgia with some probability, not a catastrophic failure.

---

## U.1 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 4/4 | Gumbel fit quality unreported; Wisconsin tail/Georgia tail stopping criterion ambiguity |
| R2 Rodden | 3/4 | "Globally optimal map" overclaim; partisan stability of ConvergenceSweep not linked to B.7 |
| R3 Duchin | 3/4 | Georgia as hardest case: statistical outlier vs. representative of distribution |
| R4 Stephanopoulos | 3/4 | DIA not labeled proposed; "provably convergent" is probabilistic not deterministic |
| R5 Liang | 4/4 | B.7 convergence-tail data should be accessible for independent Gumbel fit verification |
| **Average** | **3.4/4** | |

**Verdict: Accept** (3.4 ≥ 3.0)
**P1 count: 5** (Gumbel fit parameters unreported; Wisconsin vs. Georgia T stopping ambiguity; "globally optimal map" overclaim; DIA as proposed; "provably convergent" is probabilistic; B.7 tail data inaccessible)
**Top P1 issue: R2 — Abstract and conclusion claim ConvergenceSweep "produces the globally optimal map deterministically"; METIS is a heuristic and ConvergenceSweep finds the best available local optimum with high probability — claiming global optimality is algorithmically incorrect**

---

### U.1 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B16-P1-A | P1 | Gumbel fit: report location $\mu$, scale $\beta$, fitting method, goodness-of-fit, and CI on $P(\text{tail}>600)$ | R1 |
| B16-P1-B | P1 | Wisconsin tail 1,023 vs. Georgia tail 511: clarify whether T counts consecutive non-improving seeds or absolute offset; explain why T=600 covers Georgia but WI case is handled | R1 |
| B16-P1-C | P1 | "Globally optimal map": change to "best local optimum found across the sweep with probability >99.9%" | R2 |
| B16-P1-D | P1 | DIA references: add "proposed" throughout | R4 |
| B16-P1-E | P1 | B.7 tail data: include as supplementary table or provide data access path so Gumbel fit is independently verifiable | R5 |
| B16-P2-A | P2 | Partisan stability of ConvergenceSweep vs. fixed seed: explicitly cite B.7 evidence that seat outcome does not change | R2 |
| B16-P2-B | P2 | Georgia outlier: discuss whether Georgia's tail is consistent with fitted Gumbel or anomalous | R3 |
| B16-P2-C | P2 | SHA-256 modulus: provide byte-level specification of truncation to METIS seed width | R3 |
| B16-P2-D | P2 | T_prac = 500 failure mode: state explicitly that T=500 produces a suboptimal result for Georgia (probability <1%), not a hard failure | R5 |

---

---

# U.2 — How Sensitive Are Redistricting Outcomes to Algorithm Parameters? A 50-State Sweep

**Directory**: `research/tracks/B-foundations/U.2+parameter-sensitivity/`
**Venue target**: Political Analysis
**Status going in**: No prior panel review

---

## R1 — Karypis

**Score: 4 / 4**

The five-parameter sweep is methodologically sound. The finding that partisan seat counts vary by at most ±0.2 seats per parameter and ≤0.3 seats in the joint sweep is an empirically important result. The factorial sweep of $u_{\rm factor} \times \alpha_{\rm county}$ (12 combinations for WI and NC) correctly tests for interaction effects.

**P1 — The abstract claims "maximum variation... $\Delta D_{\rm seats} \leq 0.3$ nationally" but the results section (Key Finding) reports the conservative additive upper bound as 0.5 seats, with the empirical joint estimate as ≤0.3.** The abstract should be precise: the 0.3 figure is an empirical upper bound from the joint sweep; the theoretical additive bound is 0.5 (assuming independence and additive effects). Using 0.3 in the abstract without specifying it is empirical (not guaranteed) conflates the empirical and theoretical bounds.

**P2 — The $u_{\rm factor}$ range in the abstract ($[0.1\%, 5\%]$) matches the table range ($0.1\%$ to $5\%$). No inconsistency.** No P1 issue.

---

## R2 — Rodden

**Score: 4 / 4**

The central finding — that partisan outcomes are parameter-insensitive while compactness is parameter-sensitive — is exactly what the literature needs. The 60× ratio between algorithm-structure effects (18 seats nationally, from B.0) and parameter effects (0.3 seats) is a compelling quantification that directly addresses the "can the algorithm be gamed?" question.

**P1 — T_stat = 600 vs. T = 400 as convergence threshold.** Table 3 shows $T = 100$ produces a slightly higher $D_{\rm nat}$ (0.1 seats) because it misses Georgia's global minimum. But $T \geq 200$ produces identical outcomes in all 50 states. The paper's conclusion recommends $T_{\rm stat} = 600$ as the statutory default based on U.1, but Table 3 shows $T = 200$ is already sufficient for identical outcomes. The paper should note this: $T = 200$ is empirically sufficient, $T = 600$ provides the safety margin justified by the Gumbel tail bound (U.1). The distinction is important for understanding the statutory conservatism.

**P2 — The national seat count ($D_{\rm nat} \approx 211$) across the parameter sweep is a useful empirical anchor but should be placed in context: how does 211 compare to a proportional baseline (given that D votes nationally are approximately 50.3%)?**

---

## R3 — Duchin

**Score: 3 / 4**

**P1 — The VRA boost weight $w_{\rm vra}$ sweep ($0.2$ to $0.6$) is conducted on all 50 states but VRASection is only applicable in VRA states.** Table 5 shows $w_{\rm vra}$ variation of ±0.1 seats nationally. But $w_{\rm vra}$ affects only VRA states (those with sufficient minority VAP for an alignment signal). Sweeping $w_{\rm vra}$ across all 50 states dilutes the effect — in the 40+ non-VRA states, changing $w_{\rm vra}$ has no effect because $A \approx 0$ for all ratios. The relevant sensitivity is in the 6 VRA states from T.7. The table should report $w_{\rm vra}$ sensitivity separately for VRA states vs. the national aggregate.

**P2 — $\alpha_{\rm county}$ split reduction in Table 2 disagrees with T.3.** Table 2 shows county splits of 312 at $\alpha_c = 1.0$ and 271 at $\alpha_c = 3.0$. But T.3 Table 1 reports 487 at $\alpha_c = 1.0$ and 323 at $\alpha_c = 3.0$. The U.2 figures are substantially lower than T.3's. The discrepancy (487 vs. 312 at baseline) must be explained: is U.2 using a different definition of "county split"? A different set of states? This is a cross-paper numerical inconsistency that must be resolved.**

---

## R4 — Stephanopoulos

**Score: 3 / 4**

**P1 — The DIA is referenced throughout as providing default parameters.** All DIA references should use "proposed DIA." The paper otherwise makes no legal claims.

**P2 — "A hostile legislature" framing in the introduction.** The abstract asks: "Could a hostile legislature exploit parameter freedom to produce a partisan outcome while claiming algorithmic neutrality?" The finding (maximum 0.3 seats nationally) answers this negatively. But the paper should note the key constraint: the DIA fixes parameters in statute, so a "hostile legislature" cannot change parameters without amending the statute. The parameter-insensitivity finding is most relevant to the question of whether parameter-setting authorities (e.g., an implementation agency) can game the system, not a legislature acting in bad faith.**

---

## R5 — Liang

**Score: 4 / 4**

The methodology is clean: each parameter is swept independently at fixed baselines for the other four, with runtime and outcome metrics reported for each level.

**P1 — No seeds disclosed for the 50-state parameter sweep.** The sweep uses default parameters as baseline but does not specify the seed or seed protocol. With 50 states × 5 parameters × average 5 levels = 1,250 individual runs, the seed protocol is critical for reproducibility. Is this a ConvergenceSweep at $T=600$ (which would be the correct DIA baseline) or a fixed-seed run? The methodology section must specify.

**P2 — Runtime table (Table 3): wall time for 50 states at different T values.** The hardware used for the runtime benchmark is not specified. Runtimes (14 min for T=100; 88 min for T=1000) should come with a hardware description.

---

## U.2 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 4/4 | 0.3 empirical vs. 0.5 theoretical bound: distinguish in abstract |
| R2 Rodden | 4/4 | T=200 empirically sufficient vs. T=600 statutory; distinction should be noted |
| R3 Duchin | 3/4 | $w_{\rm vra}$ sensitivity diluted by non-VRA states; county splits U.2 vs. T.3 discrepancy |
| R4 Stephanopoulos | 3/4 | DIA not labeled proposed |
| R5 Liang | 4/4 | Seed protocol for 1,250-run sweep not specified |
| **Average** | **3.6/4** | |

**Verdict: Accept** (3.6 ≥ 3.0)
**P1 count: 4** (0.3 empirical vs. 0.5 theoretical not distinguished; $w_{\rm vra}$ sensitivity diluted by non-VRA states; county splits U.2 (312) vs. T.3 (487) baseline discrepancy; seed protocol unspecified)
**Top P1 issue: R3 — U.2 Table 2 shows 312 county splits at $\alpha_c = 1.0$ vs. T.3 Table 1 showing 487 county splits at the same parameter; this is a direct numerical contradiction across papers that must be resolved before any submission**

---

### U.2 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B17-P1-A | P1 | Abstract "≤0.3 nationally": specify this is empirical joint-sweep bound; theoretical additive bound is 0.5 | R1 |
| B17-P1-B | P1 | $w_{\rm vra}$ sensitivity: report separately for VRA states (T.7 set: AL, NC, SC, GA, MS, LA) vs. national aggregate | R3 |
| B17-P1-C | P1 | County splits discrepancy: U.2 Table 2 baseline 312 vs. T.3 Table 1 baseline 487; resolve and explain difference | R3 |
| B17-P1-D | P1 | Seed protocol for all 1,250 runs: specify seed or ConvergenceSweep threshold used | R5 |
| B17-P2-A | P2 | DIA: add "proposed" qualifier throughout | R4 |
| B17-P2-B | P2 | T=200 vs. T=600 distinction: note that T=200 is empirically sufficient; T=600 provides the Gumbel safety margin from U.1 | R2 |
| B17-P2-C | P2 | National $D_{\rm nat} \approx 211$ context: compare to proportional baseline given ~50.3% D vote share nationally | R2 |
| B17-P2-D | P2 | Runtime table hardware: specify machine used for T-sweep wall times | R5 |

---

---

# T.9 — Redistricting Stability Across Reapportionment Cycles

**Directory**: `research/tracks/B-foundations/T.9+multi-reapportionment-stability/`
**Venue target**: Political Analysis or APSR
**Status going in**: No prior panel review

---

## R1 — Karypis

**Score: 3 / 4**

The prime-factorization change analysis (Table 1) is algorithmically correct and the qualitative characterization of tree-structure changes (flat vs. hierarchical, depth changes) is appropriate. The GerryChain baseline comparison (Hamming distance as unit of disruption) is a useful framing.

**P1 — "15–25% boundary movement expected for Texas ($38 \to 41$)" is a model prediction, not an empirical result.** Section 3.2 states: "We expect boundary movement of approximately 15–25% of tracts (compared to 3–5% for seed changes within the current tree structure)." This prediction is not backed by actual simulation of the 41-seat Texas tree. The paper should either (a) run the simulation and report actual Hamming distances, or (b) clearly label these as projections from the structural analysis, not empirical results.

**P1 — The GerryChain comparison table shows TX reapportionment $d_{\rm Ham} = 0.23$ and "10 ReCom steps."** Section 5.1 states this equivalence. But the Hamming distance for TX reapportionment ($38 \to 41$) should be derived from actual simulation of both the $k=38$ and $k=41$ ApportionRegions maps. Is $d_{\rm Ham} = 0.23$ an empirical result (from running both trees) or a projection? The paper should clarify.

**P2 — "Flat tree of depth 1" for Texas $k=41$ (prime).** A 41-way direct partition is indeed depth 1. But the paper should note that METIS's $k$-way partitioner (ncon=1, k=41) operates differently from recursive bisection — it partitions directly into 41 parts in one step. For ApportionRegions, a prime $k$ requires the prime-fallback mode. The paper should confirm that the ApportionRegions implementation handles prime $k$ via direct $k$-way partitioning, not by treating it as a two-step tree.

---

## R2 — Rodden

**Score: 3 / 4**

**P1 — The 2030 seat projections (TX: 38→41; CA: 52→50; FL: 28→29; etc.) are from "Census Bureau projections" but no citation is provided.** The paper states "Census Bureau projections" parenthetically but does not cite a specific report, methodology, or projection vintage. For a paper on political analysis of reapportionment, the projection source is essential: different demographic assumptions produce different seat-change predictions. The paper must cite the specific Census Bureau projection scenario used (e.g., the 2021 National Population Projections, middle series).

**P1 — The "political redistricting (typical)" row in Table 4 reports $d_{\rm Ham} = 0.35$–$0.55$.** This is a comparative claim about how much political redistricting changes district assignments. The source for this figure must be cited. If this is derived from comparing enacted maps across redistricting cycles, the methodology should be stated.

**P2 — The abstract states: "the key finding is that seat-change-induced disruption is less than the disruption from a single MCMC remix step."** But Table 4 shows TX reapportionment ($d_{\rm Ham} = 0.23$) is equivalent to approximately 10 ReCom steps, not less than 1 step. A single ReCom step has $d_{\rm Ham} \approx 0.026$, which is much less than 0.23. The abstract's claim contradicts Table 4. The correct finding is that reapportionment disruption is less than *full political redistricting* ($0.35$–$0.55$), not less than a single MCMC step. This must be corrected.**

---

## R3 — Duchin

**Score: 3 / 4**

**P1 — The paper claims reapportionment is "algorithmically sound and politically neutral" as its conclusion.** This conclusion is supported for the tree-structure analysis but not empirically tested for partisan outcomes. The paper should report the expected change in partisan seat counts when the tree changes (e.g., TX going from 2×19 to flat 41-way: what is the expected change in D seats?). Without partisan outcome data, "politically neutral" is an assertion, not a finding.

**P2 — Illinois ($k=17 \to 16$): "Prime to composite" transition.** The paper states this produces "$2^4$" for $k=16$. $16 = 2^4$, giving a four-level binary tree (bisect, bisect, bisect, bisect). This is correct. But a depth-4 binary tree for 16 districts will produce a different geographic structure than the current depth-1 flat 17-way partition. The paper should comment on whether this makes the Illinois boundary structure more or less stable over time and more or less comparable to political redistricting.

---

## R4 — Stephanopoulos

**Score: 3 / 4**

**P1 — The DIA references should use "proposed DIA" throughout.** Same cross-cutting issue.

**P1 — "Decadal recomputation is algorithmically sound and politically neutral" is the conclusion, but politically neutral requires demonstrating that the expected partisan outcome from the new tree is not systematically biased.** If a state gains a seat that shifts from a prime (flat tree) to a composite (hierarchical tree), and hierarchical trees systematically favor one party (because of the urban-peel pathology), the reapportionment is not politically neutral even though the algorithm is. The paper should address whether tree-structure changes produce predictable partisan effects.

**P2 — The incumbent disruption analysis (Section 5.3) is a useful contribution but needs a citation base.** The claim that "political redistricting cycles often move 30–50% of incumbents to new districts" needs a source.

---

## R5 — Liang

**Score: 3 / 4**

**P1 — No simulation runs are reported.** The paper analyzes tree-structure changes analytically but does not run the ApportionRegions algorithm with the projected 2030 seat counts and measure actual Hamming distances. The $d_{\rm Ham} = 0.23$ figure for Texas appears to be projected (not measured). The paper should either (a) run simulations and report actual Hamming distances, or (b) clearly label all disruption figures as projections.

**P2 — The paper does not specify what data source is used for the 2030 population projections.** Reapportionment simulations require projected populations at the state level. Without the specific projection data, the 2030 seat-count changes cannot be independently verified.

---

## T.9 Panel Scores

| Reviewer | Score | Primary concern |
|----------|-------|-----------------|
| R1 Karypis | 3/4 | 15–25% boundary movement is a projection not an empirical result; $d_{\rm Ham} = 0.23$ origin unclear |
| R2 Rodden | 3/4 | Abstract claim (disruption < 1 MCMC step) directly contradicts Table 4 (≈10 MCMC steps); census projection uncited |
| R3 Duchin | 3/4 | "Politically neutral" conclusion not empirically tested; Illinois tree transition uncommented |
| R4 Stephanopoulos | 3/4 | DIA not labeled proposed; "politically neutral" requires partisan outcome analysis |
| R5 Liang | 3/4 | No simulation runs; all disruption figures appear to be projections not measurements |
| **Average** | **3.0/4** | |

**Verdict: Accept** (3.0 ≥ 3.0)
**P1 count: 7** (15–25% boundary movement projected not measured; $d_{\rm Ham}=0.23$ origin unclear; abstract "less than 1 MCMC step" contradicts Table 4 "≈10 MCMC steps"; Census projection uncited; "political redistricting 35–55%" uncited; "politically neutral" untested empirically; no simulation runs)
**Top P1 issue: R2 — Abstract states "seat-change-induced disruption is less than the disruption from a single MCMC remix step" but Table 4 shows TX reapportionment is equivalent to 10 ReCom steps (not less than 1); this is a direct numerical contradiction between abstract and body**

---

### T.9 Open Items Table

| ID | Severity | Description | Reviewer |
|----|----------|-------------|----------|
| B18-P1-A | P1 | Abstract: correct "disruption less than single MCMC step" to "less than full political redistricting"; Table 4 shows ≈10 ReCom steps | R2 |
| B18-P1-B | P1 | 2030 Census projections: cite specific Census Bureau report, scenario, and vintage | R2 |
| B18-P1-C | P1 | "Political redistricting $d_{\rm Ham} = 0.35$–$0.55$": cite methodology and source | R2 |
| B18-P1-D | P1 | 15–25% boundary movement for TX: label as projection from structural analysis, not empirical result | R1 |
| B18-P1-E | P1 | $d_{\rm Ham} = 0.23$ for TX reapportionment: clarify whether this is measured (from simulation) or projected (from structural analysis) | R1, R5 |
| B18-P1-F | P1 | "Politically neutral" conclusion: add partisan seat-change analysis for states with tree-structure transitions | R3, R4 |
| B18-P1-G | P1 | DIA: add "proposed" qualifier throughout | R4 |
| B18-P2-A | P2 | Texas $k=41$ prime: confirm ApportionRegions handles prime $k$ via direct $k$-way partition in the implementation | R1 |
| B18-P2-B | P2 | Illinois $17 \to 16$ (prime to $2^4$): comment on geographic and partisan implications of depth-4 binary tree vs. flat 17-way | R3 |
| B18-P2-C | P2 | Incumbent disruption 30–50%: cite source | R4 |
| B18-P2-D | P2 | 2030 population projection data: specify source for state-level projections used to derive seat-count changes | R5 |

---

---

## B-foundations Batch 2 Summary Table

| Paper | Avg Score | Verdict | P1 Count | Top P1 Issue |
|-------|-----------|---------|----------|--------------|
| B.0 Algorithm Toolbox | 3.0/4 | Accept | 7 | Abstract states GeoSection gives NC -13.6 pp, but Table 2 (confirmed) shows +0.7 pp; direct internal contradiction (R3) |
| B.02 Districting Integrity Act | 2.8/4 | Minor Revision | 8 | DIA called "the law" (enacted); *Callais* cited with page 36 holdings for apparently undecided case; *Harkenrider* mischaracterized as adopting efficiency gap (R4) |
| T.1 GeoSection | 3.0/4 | Accept | 6 | T.1 abstract reports NC GeoSection as "5D/9R (-13.6 pp)"; B.0 confirmed bakeoff Table 2 shows "7D/7R (+0.7 pp)"; inter-paper numerical contradiction (R1) |
| T.3 County-Sticky | 3.4/4 | Accept | 4 | Pareto-frontier analysis is single-seed per state; undisclosed in abstract (R2/R5) |
| T.6 NestSection | 3.2/4 | Accept | 5 | Paper proposes NestSection but has not run it; Oregon and Alabama figures are schematic placeholders "[to be generated by pipeline]" (R5) |
| T.7 VRASection | 3.0/4 | Accept | 6 | Abstract says 6-state comparison "pending"; Discussion presents it as a completed 3/6 finding; internal consistency error (R2/R5) |
| T.8 StabilitySection | 3.0/4 | Accept | 6 | "[TBD: 2010 sweep pending]" appears verbatim in abstract; publication-blocking placeholder (R1/R4/R5) |
| U.1 ConvergenceSweep | 3.4/4 | Accept | 5 | Abstract claims "globally optimal map deterministically"; METIS is a heuristic; global optimality is incorrect (R2) |
| U.2 Parameter Sensitivity | 3.6/4 | Accept | 4 | U.2 Table 2 baseline county splits (312) contradicts T.3 Table 1 (487) for same parameter; inter-paper numerical contradiction (R3) |
| T.9 Reapportionment Stability | 3.0/4 | Accept | 7 | Abstract "less than single MCMC step" directly contradicts Table 4 "≈10 ReCom steps"; (R2) |

---

## Cross-Cutting Findings (Batch 2)

### Finding 1 — NC GeoSection partisan result: -13.6 pp vs. +0.7 pp (affects B.0, T.1)
T.1 abstract reports GeoSection produces "5D/9R (-13.6 pp)" for North Carolina. B.0 bakeoff Table 2 (confirmed, no dagger) shows GeoSection Config 2 for NC produces "7D/7R (+0.7 pp)." These are the same algorithm on the same state — one is wrong. All papers citing NC GeoSection partisan results must use the confirmed B.0 Table 2 value. Likely resolution: the -13.6 pp figure in T.1 refers to the *T.1 draft's* run result, while B.0's confirmed value reflects a later, convergence-sweep-optimized run.

### Finding 2 — County splits baseline discrepancy: 312 vs. 487 (affects T.3, U.2)
T.3 Table 1 reports 487 county splits at $\alpha_c = 1.0$ (baseline). U.2 Table 2 reports 312 county splits at $\alpha_c = 1.0$. Both claim to be national 50-state results. This is a direct inter-paper numerical contradiction. Likely cause: different definitions of "county split" (e.g., T.3 counts split county-district pairs; U.2 may count split counties only).

### Finding 3 — DIA not consistently labeled as proposed statute (affects B.0, B.02, T.3, U.1, U.2, T.9)
Six papers in Batch 2 reference the Districting Integrity Act without consistently using "proposed" or distinguishing it from enacted law. This cross-cutting P1 error was identified in Batch 1 for B.6 and B.7 and remains unresolved in Batch 2. A global find-replace "the DIA" → "the proposed DIA" is needed across all B-series papers.

### Finding 4 — *Callais* citation inconsistency (affects B.0, B.02, T.7)
Three papers cite the Callais case differently: B.0 and B.02 use "Louisiana v. Callais (2026)" with specific page citations; T.7 uses "Callais v. Landry (2025)." These appear to be the same case at different stages. All papers must use a single consistent citation; if the 2026 Supreme Court opinion exists, use that; if the page-36 citations are fabricated, they must be removed.

### Finding 5 — "[TBD]" and placeholder text in abstracts (affects T.8)
T.8 contains a verbatim "[TBD: 2010 sweep pending]" marker in the abstract. This is the most severe publication-blocking error in Batch 2.

### Finding 6 — All papers accepted except B.02 (Minor Revision)
Nine of ten papers score ≥3.0/4 and receive Accept verdicts. B.02 scores 2.8/4 (Minor Revision) due to R4 Stephanopoulos's score of 2/4, driven by the combination of DIA mischaracterized as enacted law, *Callais* page citations for a potentially unverified case, and *Harkenrider* efficiency-gap mischaracterization. U.2 (3.6/4) and U.1 (3.4/4) are the strongest papers in the batch.

---

*Panel convened 2026-05-07. Reviewer identities are simulated personas for pre-submission quality control. B.0 and B.02 are the capstone and advocacy papers of the B-foundations track; all ten papers reviewed with eight accepted and two requiring revision.*
