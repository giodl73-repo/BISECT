# REVISION-PLAN — D.4 Legal Implementation
**Paper**: Adopting Algorithmic Congressional Redistricting: Legal Pathways, Constitutional Constraints, and Model Legislation
**Round 1 Review Date**: 2026-05-05
**Round 2 Review Date**: 2026-05-05
**Target**: Avg ≥ 2.5/4 | R1 Achieved: 3.0/4 | R2 Achieved: 4.0/4 (all five reviewers: 4/4 Accept)

---

## Round 1 Score Summary

| Reviewer | Lens | Score | Verdict |
|---|---|---|---|
| Karypis | Technical accuracy / algorithm description | 3/4 | Accept w/ minor revisions |
| Rodden | Partisan implications / electoral geography | 3/4 | Accept w/ minor revisions |
| Duchin | Mathematics / algorithm-law interface | 3/4 | Accept w/ minor revisions |
| Stephanopoulos | Election law (lead) | 3/4 | Accept w/ minor revisions |
| Liang | Reproducibility / statute implementability | 3/4 | Accept w/ minor revisions |

**Panel Average**: 3.0 / 4.0 — Target met. All reviewers recommend acceptance with minor revisions.

---

## Round 2 Score Summary

| Reviewer | Lens | R2 Score | Verdict |
|---|---|---|---|
| Karypis | Technical accuracy / algorithm description | 4/4 | Accept |
| Rodden | Partisan implications / electoral geography | 4/4 | Accept |
| Duchin | Mathematics / algorithm-law interface | 4/4 | Accept |
| Stephanopoulos | Election law (lead) | 4/4 | Accept |
| Liang | Reproducibility / statute implementability | 4/4 | Accept |

**Round 2 Panel Average**: 4.0 / 4.0 — Target (≥ 3.4/4) exceeded. All five reviewers accept unconditionally.

### P1 Issues Resolved (Round 2)

| Issue | Status |
|---|---|
| P1-A: VRA mode not in statute (CRITICAL) | Resolved — §2(e) VRA Adjustment added |
| P1-B: Compactness metric mismatch (CRITICAL) | Resolved — Drafting Notes paragraph added |
| P2-B: Gundy nondelegation risk | Resolved — paragraph added in §6.1 |
| P2-C: Miller v. Johnson predominant-factor test | Resolved — paragraph added in §6.4 |

### Residual Notes (Non-blocking, for final revision)
- Karypis: Open-source specification (version pins, license type) in §2(a)(1)(E)
- Rodden: Geographic sorting footnote 2 → main text
- Rodden: 69-district disaggregation by state partisan control
- Duchin: 2% deviation tail and Karcher compliance
- Stephanopoulos: *Arizona State Legislature v. AIRC* engagement in §2
- Stephanopoulos: Gingles prongs 2/3 Bureau authority in §2(e)
- Stephanopoulos: Geographic sorting footnote → main text
- Liang: Recertification provision for §2(b)
- Liang: Adjustment standing and conflict resolution in §2(d)(1)
- Liang: Reproducibility environment specification in §2(b)
- Liang: At-large states exemption (Alaska, Delaware, Vermont, etc.)

---

## Cross-Reviewer Issue Synthesis

Issues appearing in 2+ reviews are prioritized. Stephanopoulos is the lead reviewer for this paper.

### Priority 1 — Model Statute Deficiencies (Multiple Reviewers, Liang Lead)

**P1-A: VRA mode not addressed in the model statute** [Liang, Stephanopoulos, Karypis — CRITICAL]
The model statute (Section 2) contains no provision addressing the VRA mode of the algorithm. Section 2(a)(1)(B) prohibits partisan, voting-history, and electoral-preference data but does not address minority population data. Without explicit statutory authorization, the Bureau of the Census cannot certify an algorithm with a VRA mode, even though Sections 5.1-5.4 of the paper argue at length for the VRA mode's compliance advantages. This is an internal inconsistency between the legal analysis and the model statute: the paper argues that the VRA mode is legally sound and practically necessary, but the statute as drafted does not authorize it.

Required addition: A new subsection in Section 2 of the statute authorizing the Bureau to certify an algorithm with a "VRA compliance mode" that may use minority population data from Census P.L. 94-171 redistricting data, specifying: (a) the data authorized (census minority population counts only); (b) that this mode shall be applied where preliminary analysis indicates Gingles prong 1 is potentially satisfied for an identified minority community; (c) documentation requirements for VRA mode parameters; and (d) that VRA mode use constitutes the safe harbor described in Section 5.3 of the paper.

**P1-B: Compactness metric mismatch between statute and algorithm** [Duchin, Karypis — CRITICAL]
Section 2(a)(1)(C)(iii) defines compactness as minimizing "the ratio of district boundary perimeter to district area (maximize compactness)" — the Polsby-Popper formulation. But METIS optimizes edge cut in a graph (minimizing inter-tract boundary crossings), not perimeter-to-area ratio in continuous space. These are related but distinct objectives. A district that is compact by METIS edge-cut may not be compact by Polsby-Popper, and vice versa. This discrepancy creates a litigation vulnerability: a challenger could argue that the certified algorithm does not satisfy the statute's compactness requirement as stated.

Options for revision: (a) Revise the statute to define compactness as graph edge-cut minimization, with a note that this approximates but is not identical to Polsby-Popper; or (b) retain Polsby-Popper as the statutory standard but require the Bureau's algorithm certification to include a post-partitioning compactness verification step that confirms Polsby-Popper scores meet the statutory threshold; or (c) specify that the Bureau may choose the compactness metric as part of the certification process, provided the chosen metric is documented and consistently applied. Option (a) is simplest; option (c) provides the most flexibility.

**P1-C: No recertification procedure for subsequent census cycles** [Liang]
Section 2(b) requires certification "not later than 180 days after the date of enactment" but is silent on recertification. As algorithmic methods improve between census cycles, the Bureau should have authority to recertify a better algorithm without statutory amendment. Add: "The Bureau may recertify the redistricting algorithm after each decennial census to incorporate improvements in algorithmic methods, provided that the recertified algorithm satisfies the requirements of subsection (a)(1) and is published and validated under the procedures of subsection (b) not later than 365 days before the census data is scheduled to be delivered to the President under section 141(b) of title 13."

**P1-D: Adjustment standing and conflict resolution** [Liang]
Section 2(d)(1) permits "any person" to propose adjustments without restriction on standing or volume. The statute should specify: (a) who has standing to propose adjustments (governments, registered organizations, individuals — with signature or organizational verification requirements); (b) a procedure for prioritizing and aggregating conflicting proposals; (c) a maximum number of adjustments the Bureau is required to evaluate per state (or a resource constraint mechanism); and (d) a conflict-resolution procedure when two accepted adjustments are geographically incompatible.

**P1-E: Reproducibility environment specification** [Liang]
Section 2(b) requires code publication but does not specify reproducibility environment. The statute should require: (a) version-pinned software dependencies to be published alongside the source code; (b) serialized adjacency graph files for each state to be published (so the algorithm can be reproduced from a fixed intermediate representation rather than from raw census data, avoiding floating-point indeterminism); (c) a public validation run that demonstrates identical output from the published code and data.

**P1-F: At-large states exemption** [Duchin, Karypis]
States with a single congressional district (currently: Alaska, Delaware, Montana, North Dakota, South Dakota, Vermont, Wyoming — subject to change after each census) require no redistricting. The model statute should explicitly address these states: they should be exempt from the algorithm production requirement but still subject to population-equality monitoring. Add a subsection: "States entitled to one Representative under the apportionment statement are exempt from subsections (b) through (d) of this section. The single congressional district for such a State shall encompass the entire State."

### Priority 2 — Constitutional Analysis Issues (Stephanopoulos Lead)

**P2-A: Arizona State Legislature v. Arizona Independent Redistricting Commission engagement** [Stephanopoulos]
The paper relies heavily on Smiley v. Holm for the proposition that the Elections Clause grants Congress authority to mandate redistricting method. Arizona State Legislature v. AIRC (2015) substantially elaborated Elections Clause interpretation, holding that "Legislature" in the Elections Clause can encompass citizen initiatives. This ruling extended Elections Clause legitimacy to non-legislative redistricting bodies, which is relevant to the paper's state pathway analysis. But the paper does not discuss whether Arizona State Legislature affects the scope of Smiley's force for congressional mandates. Specifically: if the Elections Clause's "Legislature thereof" in the first sentence can encompass commissions, does this imply that the second sentence's congressional override power is broader or narrower than Smiley suggested? The paper should engage Arizona State Legislature directly, explaining why its holding supports rather than complicates the federal mandate argument.

**P2-B: Post-Gundy nondelegation risk** [Stephanopoulos]
The paper's nondelegation analysis is adequate for the pre-2019 doctrinal landscape but does not engage the signals in Gundy v. United States (2019) and subsequent opinions from Justices Gorsuch, Thomas, Alito, and Barrett that suggest the Court may tighten the intelligible-principle requirement. The paper should (a) acknowledge this risk explicitly; (b) argue why the algorithmic redistricting mandate would satisfy a heightened intelligible-principle standard — specifically, that the statute's mathematical specification of required algorithm properties (not a delegation of political judgment but a specification of technical constraints) is constitutionally different from the broad delegations challenged in industrial and financial regulation contexts; and (c) note that even under the most demanding nondelegation reading, a statute that tells the Bureau "compute districts using these mathematical criteria" is far more specific than any contested delegation that has reached the Court.

**P2-C: Miller v. Johnson predominant-factor analysis for VRA mode** [Stephanopoulos, Duchin]
The paper's Shaw v. Reno analysis (Section 6.4) correctly argues that compact algorithmic districts have a non-racial explanation. But Shaw doctrine has evolved through Miller v. Johnson (1995) and Bethune-Hill v. Virginia State Board of Elections (2017) to focus on the "predominant factor" test — whether racial considerations predominated over traditional redistricting principles — rather than just district shape bizarreness. The paper should engage Miller v. Johnson directly: under the predominant-factor test, would the VRA mode's edge-weighting cause race to predominate over compactness as the algorithm's governing principle? The answer may be no (because edge-weighting modifies the compactness optimization rather than replacing it), but this needs to be argued, not assumed. The paper should specify how a court would evaluate predominance when an algorithm uses demographic data to modify (rather than override) its compactness objective.

**P2-D: Smiley's scope relative to the specific mandate** [Stephanopoulos]
Smiley v. Holm (1932) held that Congress may prescribe not just that states draw districts but how they draw them, and specifically sustained a requirement for single-member districts. The paper uses Smiley to support congressional authority to mandate a specific algorithmic method. However, there is a meaningful difference between mandating the constituency structure (single-member districts, which has a direct constitutional basis in the concept of "representatives chosen by the People of the several States") and mandating the specific mathematical algorithm by which those constituency boundaries are drawn. Courts have not extended Smiley's principle to the level of algorithm specification. The paper should acknowledge this gap and make the more specific argument: congressional authority to mandate redistricting criteria (population equality, compactness) is well-established post-Wesberry and Karcher, and mandating the algorithm is simply specifying the mechanism by which those criteria are implemented — which is squarely within Congress's power to determine "how" the criteria are met.

### Priority 3 — Partisan Implications (Rodden Lead)

**P3-A: Geographic sorting and partisan tilt must enter the main text** [Rodden]
The paper's footnote 2 acknowledges that algorithmic maps produce a Democratic advantage from urban-rural geographic sorting. This is a central political and legal objection to compactness-only redistricting and should not be in a footnote. The main text — specifically the Introduction's "Problem Algorithmic Redistricting Solves" subsection — should acknowledge this effect and explain why it is legally and normatively distinguishable from gerrymandering: (a) it reflects geographic sorting, not algorithmic design; (b) it exists in commission-drawn and court-ordered maps as well; and (c) VRA compliance analysis is unaffected because Section 2 addresses minority voter dilution, not partisan seat distributions. Moving this discussion to the main text also strengthens the paper's credibility — it demonstrates that the author is aware of the strongest objection to the proposal.

**P3-B: 69-majority-minority-district claim requires disaggregation** [Rodden]
The claim that the algorithm "produces 69 more majority-minority districts than enacted plans" appears in Section 6.3 and in the model statute's findings. This aggregate number combines states where Republican gerrymanders have reduced majority-minority districts below what compactness produces (Allen v. Milligan pattern) and states where Democratic gerrymanders have over-packed minority communities (reducing competitive influence). These are legally and politically different situations. The paper should provide at minimum a two-way disaggregation: (a) states where the algorithm produces more majority-minority districts than enacted plans (and by how much), and (b) states where it produces fewer. If the net +69 is entirely or substantially driven by Republican-controlled states where packing was used to minimize majority-minority districts, this should be stated.

**P3-C: Political feasibility of federal legislation** [Rodden]
The paper establishes constitutional authority for federal legislation but treats this as near-sufficient for federal adoption. The political obstacles — 60-vote Senate threshold, unified opposition from the majority party, incumbent protection incentives — are not discussed. The paper should include a brief realistic assessment of which federal legislative pathway is most immediately viable: is a standalone mandate (the model statute as drafted) achievable, or would it need to be attached to broader electoral reform legislation? Is the federal pathway primarily valuable as a precedent and reference point for state action, or as a near-term legislative strategy?

### Priority 4 — Technical Details (Karypis / Duchin)

**P4-A: Compactness metric consistency throughout paper** [Duchin, Karypis]
Independent of the statute revision (P1-B), the paper uses "compactness" inconsistently: sometimes referring to graph edge-cut, sometimes to Polsby-Popper, sometimes to generic "boundary perimeter to area" formulations. Standardize the terminology throughout the paper to reflect what the algorithm actually optimizes (graph edge-cut minimization) and acknowledge where this differs from the legal standard (perimeter-to-area ratio).

**P4-B: 2% of districts exceeding ±0.5% deviation** [Duchin]
The claim that the algorithm achieves ±0.5% for "98% of districts" leaves 2% unexplained. The paper should address: (a) whether the 2% tail reflects small states, geographic constraints, or systematic algorithmic patterns; (b) whether Karcher v. Daggett's "as nearly as practicable" standard is satisfied for these districts; and (c) whether the model statute's ±0.5% tolerance is a hard cap (no exceptions) or a target (with the Bureau authorized to exceed it in specified circumstances). Currently the statute reads as a hard cap, which conflicts with the empirical 98% figure.

**P4-C: Algorithm failure handling** [Karypis]
The model statute does not address what happens if the algorithm fails to produce a valid output for a state (convergence failure, non-contiguous districts, edge cases). A fallback provision is needed specifying that if the algorithm fails, the Bureau must notify Congress and the affected state within 5 days, and the state's existing district boundaries remain in effect pending rerun or legislative action.

**P4-D: Census Bureau VRA prong 2/3 analysis authority** [Karypis]
The VRA safe harbor provision requires that districts "satisfy the Gingles preconditions for identified minority communities." Prongs 2 and 3 of Gingles (political cohesion and majority bloc voting) require access to election data — which the redistricting algorithm does not use. The Bureau would need to conduct a separate empirical analysis (using election returns and ecological inference methods) to determine whether Gingles prongs 2 and 3 are satisfied for any given community. The statute should authorize this separate analysis and specify what data the Bureau may use for this purpose (election results, exit poll data, ecological inference estimates).

---

## Revision Actions by Section

| Section | Actions Required | Priority |
|---|---|---|
| 1. Introduction | Move footnote 2 (geographic sorting) to main text (P3-A) | P3 |
| 1.2 Problem Subsection | None required | — |
| 2. Constitutional Foundations | Add Arizona State Legislature engagement (P2-A); strengthen Smiley scope argument (P2-D) | P2 |
| 3. Federal Pathway | Add post-Gundy nondelegation risk acknowledgment and response (P2-B); add political feasibility assessment (P3-C) | P2, P3 |
| 4. State Pathways | Minor clarification of Type IV states' variation | P3 |
| 5. VRA | Engage Miller v. Johnson predominant-factor test (P2-C); disaggregate 69-district claim (P3-B) | P2, P3 |
| 6. Challenges | Add Miller v. Johnson directly to Shaw analysis (P2-C); address VRA-produces-too-few disaggregation (P3-B) | P2, P3 |
| 7. Model Statute — Definitions | Revise compactness definition (P1-B); add at-large exemption (P1-F) | P1 |
| 7. Model Statute — Section 2(b) | Add recertification provision (P1-C); add reproducibility environment specification (P1-E) | P1 |
| 7. Model Statute — Section 2(d) | Add standing restriction and conflict resolution (P1-D) | P1 |
| 7. Model Statute — New Section | Add VRA mode authorization provision (P1-A) | P1 — CRITICAL |
| 7. Model Statute — Findings | Condition VRA finding on VRA mode application (P1-A); address compactness metric (P1-B) | P1 |
| 7. Drafting Notes | Add algorithm failure fallback (P4-C); add Gingles prong 2/3 Bureau authority (P4-D) | P4 |
| Throughout | Standardize compactness terminology (P4-A); address 2% deviation tail (P4-B) | P4 |

---

## Key Doctrinal Checks Summary

| Claim | Status | Action Required |
|---|---|---|
| Smiley v. Holm supports federal mandate | Correct but scope understated | Engage Arizona State Legislature; strengthen scope argument (P2-A, P2-D) |
| Nondelegation: delegation is to Bureau, not algorithm | Correct | Acknowledge Gundy risk; strengthen under heightened standard (P2-B) |
| Model statute has intelligible principle | Yes — mathematical specification | Strengthen under post-Gundy reading |
| VRA safe harbor correctly interprets Callais | Structurally correct | Specify evidentiary mechanics under Miller v. Johnson (P2-C) |
| Algorithm produces more majority-minority districts | Empirically supported | Disaggregate by state political context (P3-B) |
| Shaw v. Reno: compact districts not racially motivated | Correct under bizarre-shape test | Engage Miller v. Johnson predominant-factor test (P2-C) |
| Model statute in U.S. Code style | Yes — correct formatting | Add VRA mode provision; add recertification; add at-large exemption (P1-A, P1-C, P1-F) |
| Compactness definition consistent with METIS | No — mismatch | Revise statute and paper terminology (P1-B, P4-A) — MATERIAL DEFICIENCY |

---

## Issues Not Requiring Revision

- Cook v. Gralike citation — correctly supports Elections Clause "how" authority.
- Huntington-Hill analogy — well-developed, no revision needed.
- Three-pathway structure (federal, state, court-ordered) — comprehensive and well-organized.
- Court-ordered remedy analysis — timely and persuasive; no revision needed.
- Model statute's three-section structure — legally coherent; no structural revision needed beyond the specific additions above.
- Judicial review provision's enumerated grounds — correctly limits to administrable standards.

---

## Estimated Revision Burden

- Priority 1 (statute deficiencies): Moderate-to-heavy — requires drafting new statutory text for VRA mode provision (P1-A, most significant), adjustment standing procedure (P1-D), and at-large exemption (P1-F). P1-B requires careful coordination between statute text and algorithm description.
- Priority 2 (constitutional analysis): Moderate — requires 2-4 paragraphs engaging Arizona State Legislature, Gundy, and Miller v. Johnson. No new research required.
- Priority 3 (partisan implications): Light — moving footnote 2 to main text (P3-A) and adding political feasibility paragraph (P3-C); moderate for 69-district disaggregation if data can be pulled from the companion technical paper.
- Priority 4 (technical details): Light — terminology standardization, algorithm failure provision, Gingles prong 2/3 analysis authority.

Total estimated effort: **Medium-to-heavy** — most effort is in the model statute revision, specifically the VRA mode provision (P1-A). No new legal research is required; the VRA analysis already in Section 5 provides the doctrinal foundation.
