# Hostile Panel Review — D.6, D.7, D.8
**Track**: D — VRA and Legal Frameworks
**Date**: 2026-05-08
**Panel**: Karypis (R1), Rodden (R2), Duchin (R3), Stephanopoulos (R4), Liang (R5)
**Post-write P1 fixes applied**: Yes

---

## Summary Scorecard

| Paper | R1 | R2 | R3 | R4 | R5 | Avg | Verdict | P1 count | Top P1 issue |
|-------|----|----|----|----|-----|-----|---------|----------|--------------|
| D.6 Prison Gerrymandering | 2 | 2 | 3 | 3 | 1 | 2.2 | Major Revision | 4 | `--adjust-prison-population` flag does not exist in codebase |
| D.7 Section 203 | 2 | 2 | 3 | 3 | 1 | 2.2 | Major Revision | 4 | County split counts are single-seed assertions without ensemble support |
| D.8 Post-Shelby | 3 | 3 | 3 | 3 | 2 | 2.8 | Minor Revision | 2 | "bisect uses no partisan data" claim requires verification |

---

## D.6 — Prison Gerrymandering and Algorithmic Redistricting

### Per-Reviewer Scores and Notes

---

**R1 — Karypis (computational): 2 / 4**

The paper makes a clean methodological claim: prison adjustment is a preprocessing step in `bisect-data`, triggered by `--adjust-prison-population` on `bisect fetch`. This is exactly the kind of claim I need to be able to verify. The adjustment algorithm (Steps 1–3: identify GQ tracts, subtract, reallocate to home county via BJS NPS data) is described with enough precision to be principled. The two L0 invariants (population conservation, non-negativity) are correct and appropriate.

However, the home-county reallocation falls apart on inspection. Step 3 states that BJS NPS data provides "home-county distribution for states that participate in county-level reporting" and falls back to "state-level distribution weighted by county population" for others. This fallback is a significant methodological concession: for states without county-level NPS data, the allocation is purely proportional to county population, which is not an adjustment to the prisoner's home address at all — it is a uniform redistribution. The paper does not disclose which states use the precise county-level allocation versus the population-weighted fallback, nor does it quantify the error introduced by the fallback. This is a pipeline opacity problem.

The California validation claim (matches CRC within 0.1% at county level) is specific and reassuring. But the paper provides no analogous validation for North Carolina or Texas. The NC case study asserts that reallocated individuals go "primarily to Mecklenburg County, Durham County, and Forsyth County" — sourced to "North Carolina Department of Adult Correction administrative data" — but it is unclear whether this specific administrative data is actually used in the bisect pipeline or is cited only for illustrative purposes. If the pipeline uses BJS NPS data (as stated in Step 3) but the illustrative example uses NCDAC data, there is an undisclosed mismatch.

**P1: Specify which states use county-level NPS data vs. population-weighted fallback. Quantify fallback error or bound it.**
**P1: Clarify whether NCDAC data is used in the NC pipeline run or only as narrative illustration.**
**P2: Add validation table for NC and TX analogous to the CA 0.1% validation.**
**P3: GQ Type codes (101, 201) should be verified against the 2020 DHC-A technical documentation; the paper cites them without a direct footnote to the DHC-A codebook.**

---

**R2 — Rodden (political science): 2 / 4**

The prison gerrymandering literature is well-established (Wagner, Sawyer; see also Sugie and Lens 2017; King et al. 2021 for the political science angle — none of these appear in the references). The paper's empirical claims are modest and directionally reasonable, but the single-seed design ($^\dagger$ notation is used consistently and correctly, which I credit) makes it impossible to know whether the partisan shift attributions are robust.

The NC result says the district shifts from R+2.1 to R+1.8 — a 0.3 pp change from an 1,800-person reallocation into a 745,000-person district. This is arithmetically coherent. But the paper attributes this shift "entirely to the population reallocation." With a single seed, we do not know whether rerunning bisect with different seeds would converge to the same district boundary configuration. The reallocation changes the population balance input, which changes where bisect cuts. But bisect's cut location can also vary from seed to seed for reasons unrelated to the adjustment. Without a seed sweep, the causal attribution is overclaimed.

The 2020 presidential vote proxy (VEST data) is appropriate and standard. The 58% Black or Hispanic incarcerated population figure is cited to BJS 2021 (NCJ 302776) — this is a legitimate source. However, the paper conflates "Black or Hispanic" throughout (abstract, intro, conclusion) with the VRA category "minority." In VRA Section 2 analysis, Black and Hispanic communities are distinct for Gingles purposes — a combined 58% figure is not the appropriate framing for VRA analysis even if it is appropriate for general description.

The Texas result (130,000 state prisoners reallocated) is the most impactful claim. The claim that Harris County absorbs "approximately 48,000" from this reallocation implies a ~37% share of the Texas prison population originating from Harris County. That figure is plausible but unsourced — the paper should cite the specific NPS county-of-origin data supporting this allocation.

**P1: Overclaimed causal attribution with single-seed runs. Add caveat or seed sweep for the key partisan shift claims.**
**P1: The "48,000 to Harris County" figure needs a source citation (specific BJS NPS data).**
**P2: Disaggregate Black and Hispanic in VRA analysis sections — the combined "minority" framing is inappropriate for Gingles.**
**P3: Cite political science literature on prison gerrymandering's electoral effects (Sugie & Lens; King et al.).**

---

**R3 — Duchin (math/redistricting): 3 / 4**

The GQ population handling is the methodological center of the paper, and it is handled correctly. The paper correctly identifies GQ Types 101 (Federal) and 201 (State correctional facilities), correctly notes that the DHC-A provides facility-level demographic breakdown, and correctly formulates the two invariants (conservation, non-negativity). The population conservation invariant is especially important: a naive implementation that subtracts but does not add back equally would violate conservation, and the paper makes clear this is checked.

The community integrity framing in the results section is well-calibrated. The paper appropriately uses the adjusted MVAP concept for Gingles threshold analysis (Section 5, bullet 3 under "VRA Implications") — this is legally correct and methodologically clear. The distinction between adjustment state (use adjusted MVAP) and non-adjustment state (use unadjusted) is correctly drawn.

The Evenwel characterization in Section 2 is accurate: "states may use total population as the apportionment base" (may, not must), and prison adjustment changes location not base. This is the correct legal reading and the distinction with CVAP-equalization is correctly drawn.

Minor issue: the paper defines GQ Type 101 as "Federal correctional facilities" and Type 201 as "State correctional facilities." Local jails (GQ Type 301) are not discussed. The prison gerrymandering literature (and the Prison Policy Initiative data) often includes local jail populations. The paper should clarify whether the bisect adjustment covers jail populations (Type 301) or only prisons (Types 101 and 201), and why.

**P2: Clarify whether GQ Type 301 (local jails) is included or excluded from the adjustment, and justify the choice.**
**P3: The non-negativity invariant proof sketch ("prisons do not contain more people than the tract total") is correct but could be tightened — a tract could theoretically have multiple GQ types where the sum exceeds expectations; add a note that the check applies to the net post-subtraction total.**

---

**R4 — Stephanopoulos (law): 3 / 4**

The legal citations are handled carefully and the Evenwel characterization is accurate. I want to address each citation directly.

**Evenwel v. Abbott (578 U.S. 54, 2016)**: The paper correctly states the citation, correctly characterizes the holding as "states may use total population" (permission, not mandate), and correctly notes the case did not address prison gerrymandering. The paper's reading of Evenwel — that prison adjustment changes the location of counting within total population rather than the population base itself — is legally sound and consistent with how adjustment-state statutes have been read post-Evenwel. This is the paper's strongest legal contribution. Score: correct.

**Thornburg v. Gingles (478 U.S. 30, 1986)**: Citation correct. The three-precondition framework is accurately described elsewhere in the D-track papers. In this paper, it is mentioned in passing for the "majority-minority threshold" concept. Correct.

**Reynolds v. Sims (377 U.S. 533, 1964)**: Citation correct. The invocation is appropriate — the paper situates Evenwel within the one-person-one-vote doctrine. Correct.

**State statutory citations** (Table 1): These are high-risk for legal papers. I cannot independently verify all twelve state statutes in this review, but I can check the two I know well. California AB 420, codified at Cal. Elec. Code § 21003 (2011) — correct. New York N.Y. Legis. Law § 83-m (2010) — correct. Colorado Colo. Rev. Stat. § 2-1-901 — plausible, though this section number is slightly unusual for Colorado's redistricting chapter; the more commonly cited provision is § 2-1-901(2). I flag this as P2, not P1, because the section number cited is within the correct chapter.

**Scope table (Table 2)**: The claim that Colorado is the only state whose adjustment law applies to congressional redistricting is significant. This is likely correct based on the statutes as written, but the paper's own footnote wisely notes "Expert witnesses must verify current law." The paper correctly flags this uncertainty. Score: appropriate caution displayed.

One genuine legal concern: the paper's discussion of "VRA Implications for Court Filings" (Section 5.3) states that "the Gingles threshold analysis...should use the adjusted MVAP" in adjustment states. This is the paper's normative legal argument, not a holding of any court. No case has directly ruled that adjustment-state Gingles analysis must use adjusted MVAP. The paper presents this as a clear legal requirement when it is actually a reasonable but untested legal argument. A hedged framing ("should be argued to use" or "the authors contend that") would be more accurate.

**P2: Colorado § 2-1-901 citation — verify whether the congressional applicability is in subsection (2) or a different subsection; cite the full provision.**
**P2: The "adjusted MVAP for Gingles" claim (Section 5.3) is a legal argument, not settled law. Reframe as such.**
**P3: Add citation to an adjustment-state case or DOJ guidance supporting the adjusted-MVAP-for-Gingles argument, if any exists.**

---

**R5 — Liang (ML/AI/systems): 1 / 4**

This paper describes a pipeline feature — `--adjust-prison-population` on `bisect fetch` — that I cannot find in the codebase. The CLAUDE.md for this project lists the core bisect commands: `bisect fetch --year 2020 --workers 8`. There is no `--adjust-prison-population` flag documented in the CLI reference (REDIST_CLI.md), and the CLAUDE.md description of bisect-data does not mention prison adjustment. This is a fatal reproducibility issue: the paper's central methodological claim (bisect implements prison adjustment) rests on a pipeline flag that does not appear to exist.

Compounding this: the California validation claim ("matches CRC within 0.1% at county level") requires running the described adjustment pipeline against known CRC data. If the flag does not exist, this validation cannot have been performed. The validation claim is therefore also unverifiable.

The two L0 invariants described (population conservation, non-negativity) are described as being "validated as L0 tests in the bisect test suite." Running `cargo test -p bisect-cli` does not surface tests by these names; the test suite documentation in CLAUDE.md does not mention prison adjustment tests.

The NPS data integration is described generically. No specific NPS dataset vintage (year, file name, BJS report number) is given for the county-level home data. The paper cites BJS 2021 (NCJ 302776) for the 58% demographic figure, but NPS county-of-origin data is a different product (NPS-8B or similar). This dataset is not cited.

**P1 (CRITICAL): Verify that `--adjust-prison-population` flag exists in bisect-cli. If not, this entire paper describes a planned but unimplemented feature. The paper must either (a) implement the feature, (b) reframe as a design specification, or (c) retract the empirical results.**
**P1: Cite specific NPS dataset (file name, BJS report, vintage year) used for county-of-origin home allocation.**
**P1: Document the L0 test names for prison adjustment invariants so they can be independently verified.**
**P2: The CA validation methodology needs a methods subsection — which CRC data file, which census year, how the 0.1% threshold was computed.**

---

### D.6 Consolidated P1/P2/P3 Issues

**P1 (must fix before acceptance):**
1. `--adjust-prison-population` flag: verify existence in codebase or reframe paper as design specification (R5)
2. NC/TX source data: specify which states use county-level NPS vs. population-weighted fallback (R1)
3. NCDAC data vs. BJS NPS: clarify whether NCDAC data is used in the pipeline or only for narrative illustration (R1)
4. Harris County 48,000 figure: provide citation to specific BJS NPS county-of-origin data (R2)

**P2 (revise before submission):**
1. Add NC and TX validation analogous to CA 0.1% comparison (R1)
2. Add seed sweep or explicit caveat on causal attribution of partisan shifts (R2)
3. Disaggregate Black and Hispanic in VRA sections (R2)
4. Clarify GQ Type 301 (local jails) scope (R3)
5. Colorado § 2-1-901 subsection verification (R4)
6. Reframe "adjusted MVAP for Gingles" as legal argument, not settled law (R4)
7. CA validation methodology subsection (R5)

**P3 (polish):**
1. DHC-A codebook footnote for GQ Type codes (R1)
2. Non-negativity invariant note on multiple GQ types per tract (R3)
3. Cite political science prison gerrymandering literature (R2)
4. Add citation for adjusted-MVAP-for-Gingles argument (R4)
5. Document L0 test names for prison adjustment invariants (R5)

---

## D.7 — Section 203 Language Minorities in Algorithmic Redistricting

### Per-Reviewer Scores and Notes

---

**R1 — Karypis (computational): 2 / 4**

The county-split methodology is clean and the formula is well-stated. The community integrity metric is appropriately defined with a clear mathematical formulation. The two-mode comparison (geographic vs. county-weight) is well-motivated.

However, the methodology section does not specify how the "Section 203 covered county" list was obtained. The paper cites 88 Fed. Reg. 34,396 (2023) correctly, but it does not document the process by which the 54 Spanish-covered Texas counties were identified and joined to the census tract data. Was this done by manual lookup of the Federal Register determination? By parsing the Federal Register data file? By downloading DOJ's published covered-jurisdictions list? The county split metric's validity depends entirely on having the correct coverage list, and the provenance of that list is undocumented.

The Texas claim (17 enacted splits, 15 geographic, 11 county-weight) is specific but single-seed. The California claim (22 enacted, 21 geographic, 20 county-weight) is so small that the difference between modes is barely detectable and may be within seed noise. The paper should explicitly compare these figures to a lower bound — given population arithmetic, what is the minimum possible number of county splits for California's 52-district plan given the county populations? If that minimum is already 18-19, the "9% reduction" headline is trivial.

**P1: Document the provenance of the Section 203 covered county list (how were the 54 TX Spanish counties identified and loaded?).**
**P2: Compute the population-arithmetic minimum county split count for California to contextualize the 9% improvement claim.**
**P3: The community integrity metric formula uses notation d(t) = d with d ranging over all districts — this is slightly ambiguous (d is used for both the function output and the summation variable). Use distinct notation.**

---

**R2 — Rodden (political science): 2 / 4**

The paper's empirical strategy is to compare single-seed bisect plans to enacted maps on county split counts. The comparison to enacted maps is appropriate — enacted maps are the legally relevant baseline — but the comparison methodology has problems.

First, the paper does not account for the population-arithmetic floor on county splits. Texas has 254 counties and 38 districts; by pigeonhole, at least 38-254 relationship forces splits in densely populated counties regardless of map design. The "35% reduction" claim is measured against the enacted map, not against the arithmetic minimum. If the enacted map has 17 splits but the arithmetic minimum is 11, bisect's result of 11 means it achieved the theoretical minimum — a very strong result that should be stated explicitly. If the minimum is 8, the result is modest. Without the baseline, the headline is uninterpretable.

Second, the Vietnamese community integrity claim for Harris County (0.73 vs. 0.45 enacted) involves 11 tracts. Defining a community as "Vietnamese-majority tracts >= 25% Vietnamese-American population" in Harris County yields whatever number of tracts passes that threshold. The threshold (25%) is both low (it means 3-in-4 residents are not Vietnamese) and undocumented in the methodology section. A higher threshold (40%, 50%) might yield a different set of tracts and a different result.

The claim that bisect uses no partisan data is stated implicitly (by describing bisect's inputs as Census demographics, adjacency, and population) but never verified. If bisect were exposed to election geometry indirectly through the geographic adjacency structure, the "neutral" characterization would be partially compromised.

**P1: Compute the arithmetic minimum county split count for TX and CA and present it alongside the results.**
**P1: Define and justify the 25% Vietnamese-American threshold for the Santa Clara community definition.**
**P2: Add confidence intervals or seed sweep on the community integrity scores (0.73 and 0.45 are single-seed estimates).**
**P3: Cite political science literature on language minority redistricting (Tucker 2009 is in the references but not engaged analytically).**

---

**R3 — Duchin (math/redistricting): 3 / 4**

The legal framework for Section 203 is accurately described. The statutory text (52 U.S.C. § 10503(b)) is correctly quoted: the two-prong test (group size AND English literacy rate below national average) is stated correctly, and the language-specificity of coverage (a county may be covered for Spanish but not Vietnamese) is correctly explained. The 2023 Federal Register citation (88 Fed. Reg. 34,396, May 30, 2023) is correct and verifiable.

The Section 203 vs. Section 2 legal hierarchy is correctly stated: Section 203 is an administrative obligation on the county election office, not a constraint on district lines. The paper correctly notes that a plan splitting a Section 203 community does not violate Section 203 as long as the county provides language assistance county-wide. The footnote on Alabama Legislative Black Caucus v. Alabama (575 U.S. 254, 2015) is correctly cited and the holding correctly described (racial predominance requires strict scrutiny but VRA-required districts satisfy it).

The dispersed vs. concentrated community distinction (Section 3.4) is analytically sharp and underappreciated in the redistricting literature. The paper correctly observes that geographic compactness optimization only helps concentrated communities; dispersed communities need explicit VRA-mode targeting.

One concern: the paper's coverage formula description (Section 2.1) states the two prongs are "(1) group size [5% of VAC or 10,000 VAC] AND (2) English literacy below national rate." This is accurate for the statutory text. However, the paper does not address the third prong that exists in some circumstances: the "historically unserved" language category that triggers coverage without an English literacy rate comparison for certain Native American and Alaska Native languages. The paper gestures at Alaska Native coverage in Table 2 without explaining why Alaska is covered under what appears to be a different mechanism.

**P2: Address the alternative coverage trigger for certain Native American/Alaska Native languages where English literacy comparison does not apply in the same way.**
**P3: The community integrity formula uses max_d, which is the mode. In a community split between three districts with 3/5/3 tracts respectively, the score is 5/11. This is reasonable but the paper should note that the metric is mode-based and a community split into many small pieces could score misleadingly high if one piece happens to be largest.**

---

**R4 — Stephanopoulos (law): 3 / 4**

Section 203 citations are verified.

**52 U.S.C. § 10503**: Correct. Section 203 is at this location in the VRA as recodified in 2014. Correct.

**88 Fed. Reg. 34,396 (May 30, 2023)**: This is the DOJ Section 203 covered jurisdictions determination. The date (May 30, 2023) and volume/page citation are correct. The paper states this covers "331 jurisdictions in 31 states" — the Federal Register determination actually covers approximately 326 jurisdictions (the number fluctuates as township coverage changes); "331" may be a rounding artifact or slightly off. This is a P2, not P1, because the precise count does not affect the legal analysis.

**VRARA 2006 (Pub. L. No. 109-246, 120 Stat. 577)**: Correct public law and stat citation. Section 203 was reauthorized as part of this act. Correct.

**Thornburg v. Gingles (478 U.S. 30, 1986)**: Correct.

**Alabama Legislative Black Caucus v. Alabama (575 U.S. 254, 2015)**: Correct citation and holding description.

**Chinatown Voter Education Alliance v. Ravitz (2009 WL 6445161, S.D.N.Y. 2009)**: This citation is used to support the proposition that "private plaintiffs may also bring Section 203 enforcement actions." The Westlaw citation is correct format, and the case is a real S.D.N.Y. Section 203 enforcement action. Correct.

One legal concern: the paper asserts a legal hierarchy in Section 5.3 — "Section 2's requirements are legally paramount" over Section 203 community integrity. This is correct as a general matter (Section 2 creates rights, Section 203 creates administrative obligations), but the paper could be more precise: Section 2 does not automatically require majority-minority districts wherever Gingles preconditions are met — it prohibits maps that dilute. There is space between "prohibits dilution" and "requires a majority-minority district." The paper's framing that Section 2 requirements always trump Section 203 is mostly correct but slightly overbroad.

**P2: Verify exact jurisdiction count in 88 Fed. Reg. 34,396 (326 vs. 331).**
**P2: Refine the Section 2 "legally paramount" framing — distinguish between Section 2 as prohibition (not automatically requiring majority-minority districts) and Section 203 as administrative obligation.**
**P3: The paper discusses enforcement (DOJ civil actions under § 10308) but does not cite the leading Section 203 case on translation adequacy — add one for completeness.**

---

**R5 — Liang (ML/AI/systems): 1 / 4**

The paper describes `--weights-override county` mode as the mechanism for minimizing county splits. This mode is documented in CLAUDE.md and the REDIST_CLI.md reference, so unlike D.6's phantom flag, this is a real pipeline feature. I credit this.

However, the empirical claims are based on comparing single-seed bisect runs to enacted 2022 congressional maps. The enacted maps are fixed (they are the actual enacted plans). The bisect runs are single-seed. The claim that bisect county-weight produces "11 covered county splits" vs. "17 enacted" is presented as a factual comparison, but we do not know whether a different bisect seed would produce 10 or 14. For an effect size this small (17 vs. 11), seed variance could plausibly span the difference.

The Section 203 covered county list integration with bisect is not described. The paper does not explain how the pipeline knows which counties are Section 203 covered. This is presumably done in analysis post-processing (not in the bisect algorithm itself), but the analysis pipeline is not described. The community integrity metric computation — which requires identifying the "Vietnamese-majority tracts >= 25% Vietnamese-American population" in Santa Clara County — requires ACS data mapping, which is not part of the core bisect pipeline. How was this mapping done? Is there code for it?

The 54 Spanish-covered TX counties claim (from a total of 67 covered TX jurisdictions) implies that 13 jurisdictions are townships or non-county equivalents. The paper does not explain this distinction or document how the 54-county subset was identified.

**P1: Describe how Section 203 coverage data is integrated into the analysis pipeline (what data file, what code, how coverage status is joined to census tracts).**
**P1: Provide seed sweep or sensitivity analysis for the core county split results (17 vs. 11 TX, 22 vs. 20 CA).**
**P2: Specify the ACS dataset vintage and variable used for Vietnamese-American tract identification (which ACS table, which year, what threshold computation).**
**P2: Explain the 67 jurisdictions → 54 counties decomposition for Texas Section 203 coverage.**

---

### D.7 Consolidated P1/P2/P3 Issues

**P1 (must fix before acceptance):**
1. Document provenance of Section 203 covered county list and pipeline integration (R1, R5)
2. Compute arithmetic minimum county split count for TX and CA — "35% reduction" headline is uninterpretable without it (R1, R2)
3. Define and justify 25% Vietnamese-American threshold for the Santa Clara community (R2)
4. Provide seed sweep or sensitivity analysis on county split counts (R5)

**P2 (revise before submission):**
1. Verify jurisdiction count in 88 Fed. Reg. 34,396 (326 vs. 331) (R4)
2. Refine Section 2 "legally paramount" framing (R4)
3. Address alternative coverage trigger for certain Native American/Alaska Native languages (R3)
4. Add confidence on community integrity scores via seed sweep (R2)
5. Specify ACS dataset for Vietnamese-American tract identification (R5)
6. Explain 67 jurisdictions → 54 counties TX decomposition (R5)
7. CA population-arithmetic minimum split count (R1)

**P3 (polish):**
1. Community integrity metric notation (d vs. d) disambiguation (R1)
2. Note mode-based nature of community integrity metric and edge cases (R3)
3. Section 2 distinction: prohibition vs. affirmative requirement (R4)
4. Leading Section 203 adequacy case citation (R4)
5. Engage Tucker 2009 analytically in the body (R2)

---

## D.8 — The Post-Shelby VRA Landscape

### Per-Reviewer Scores and Notes

---

**R1 — Karypis (computational): 3 / 4**

D.8 is primarily a legal survey paper with methodological claims about bisect as a "transparency substitute." The computational claims are more modest than D.6 or D.7, which makes them easier to evaluate.

The core computational claim: bisect produces a "full SHA audit chain" for every run, enabling independent verification. This is documented in the CLAUDE.md (`bisect label-verify <label>`) and is a real pipeline feature. The reproducibility claim is thus grounded. I credit this.

The "neutral baseline" argument — that bisect uses Census demographics, geographic adjacency, and population counts but not election results or voter registration data — is stated but not demonstrated. If bisect's adjacency graph is derived from geographic units whose boundaries correlate with election returns (as census tracts and counties almost always do), then the "partisan neutral" characterization is technically true but analytically misleading. The paper should acknowledge this limitation.

The paper is light on quantitative content. The enforcement comparison table (Table 1) is a qualitative summary. No bisect results are reported for specific states. This is appropriate for a legal survey paper, but the paper's claim to provide a "quantification" of minority vote dilution (in the legal section) is not backed by a worked example showing how such quantification would proceed.

**P2: Add a worked example showing the neutral baseline gap computation for one state (e.g., Alabama post-Allen v. Milligan) — BVAP under bisect vs. BVAP under enacted plan, and the quantitative gap.**
**P3: Acknowledge that geographic adjacency structure correlates with partisanship even without using election data directly — the "partisan neutral" characterization needs a note on this limitation.**

---

**R2 — Rodden (political science): 3 / 4**

The post-Shelby landscape description is accurate and current. The enforcement asymmetry table is well-constructed and the characterization of the Section 5 vs. Section 2 structural difference is correct. The paper correctly identifies the key empirical fact: the 2012 Texas congressional plan was not replaced until after the 2014 and 2016 elections (though a footnote clarifying the specific court timeline would help), and Alabama's 2021 plan was not remedied until after the 2022 election.

The Allen v. Milligan analysis is the paper's strongest section. The case description is accurate: 5-4 vote, Chief Justice Roberts in the majority, Gingles reaffirmed, Alabama required to draw a second majority-Black opportunity district. The paper correctly notes that the Gingles compactness analysis is the dimension where algorithmic tools add value — and this is exactly right. The district court's finding that a second opportunity district was "reasonably configured" was fundamentally a geographic and demographic claim of the kind bisect can test systematically.

The Brnovich characterization (vote-denial not vote-dilution, Gingles survival intact) is accurate and well-explained. The paper's footnote citing Brnovich at 594 U.S. at 680 n.19 for the proposition that vote-denial and vote-dilution are analyzed differently is precise and verifiable.

The "algorithmic redistricting as transparency substitute" argument is the paper's central normative contribution. It is argued clearly and with appropriate modesty — the paper explicitly acknowledges that bisect "cannot restore Section 5 preclearance." The three use cases (plaintiff reference plan, defendant neutral map defense, court-appointed special master) are well-delineated and practically useful.

One concern: the paper asserts that bisect is "resistant to the opposing argument that it was drawn to prove a point" because it is algorithmic and reproducible. This is an important claim but it assumes that the choice of bisect's parameters (weights mode, structure flag, convergence threshold) are not themselves choices that could be made strategically. An adversarial expert could argue that choosing vra-aligned weights rather than geographic weights was itself a strategic choice to produce more majority-minority districts. The paper should address how bisect parameter choices are documented and justified.

**P2: Address the adversarial parameter-choice argument — how do you demonstrate that bisect mode selection was not itself strategic?**
**P3: Add footnote on the Texas timeline: when was the 2012 plan replaced, and by court order or by the legislature?**

---

**R3 — Duchin (math/redistricting): 3 / 4**

The VRA legal framework is described correctly and with appropriate nuance. The Section 4(b) coverage formula description is accurate (1964/1968/1972 test-or-device plus below-50% turnout). The nine fully covered states are listed correctly. The Beer v. United States (425 U.S. 130, 1976) non-retrogression standard citation is correct.

The paper's key intellectual contribution — framing bisect as a "transparency substitute" for Section 5's documentation and deterrence functions — is legally coherent and methodologically useful. The explicit acknowledgment that bisect cannot replicate Section 5's ex ante legal requirement is important and prevents overclaiming.

The state VRA analogs section is accurate. The CVRA (Cal. Elec. Code §§ 14025-14032, 2001) and WVRA (Wash. Rev. Code § 29A.92, 2018) descriptions are correct. The NYVRA (S51001-A, 2022, with preclearance-like mechanism) description is accurate and the claim that it is "the most comprehensive state-level VRA analog" is defensible.

One concern: the Gingles three-part test description (Section 3.1) correctly states the three preconditions but then says "if all three preconditions are met, the court examines the totality of circumstances to determine whether the challenged plan results in minority vote dilution." This is accurate but the paper does not note that the preconditions themselves contain an implicit geographic analysis (the numerosity-and-compactness prong requires that the minority group could constitute a majority in a reasonably configured district). The bisect VRA mode is most useful in demonstrating this prong — the paper should make this connection explicit.

**P2: Make explicit the connection between Gingles prong 1 (numerosity and compactness) and bisect VRA mode — the mode directly tests whether a majority-minority district is achievable in a reasonably configured district.**
**P3: The Covington v. North Carolina citation (316 F.R.D. 117, M.D.N.C. 2016) and the Supreme Court per curiam (581 U.S. 1, 2017) are both cited in the references — make sure both appear in the text, not just one.**

---

**R4 — Stephanopoulos (law): 3 / 4**

This is D.8's strongest ground. I verify each required citation.

**Shelby County v. Holder (570 U.S. 529, 2013)**: Correct. The paper cites the holding accurately — Section 4(b) coverage formula invalidated; Section 5 not struck but unenforceable without a valid formula. The Roberts majority's equal sovereignty rationale is correctly described. The Ginsburg dissent is correctly characterized: Section 5's deterrent success was being evaluated against itself. The paper's quotation from the majority ("The Fifteenth Amendment is not designed to punish for the past; its purpose is to ensure a better future") with citation to 570 U.S. 529, 544 (2013) — correct page. The equal sovereignty doctrine citations (554-56) are correct.

**Allen v. Milligan (599 U.S. 1, 2023)**: Correct. 5-4 vote confirmed. Chief Justice Roberts in the majority confirmed. Kavanaugh concurrence noted. The holding — Gingles means what it says; using race to comply with VRA does not automatically violate Equal Protection; strict scrutiny triggered by exceeding VRA requirements, not complying with them — is accurately described.

**Brnovich v. Democratic National Committee (594 U.S. 647, 2021)**: Correct. The paper accurately describes Brnovich as limited to vote-denial claims. The citation to 594 U.S. at 680 n.19 for the vote-denial/vote-dilution distinction is specific and appears accurate (the majority opinion does address the two categories in that vicinity of the opinion). Kagan dissent correctly described.

**Abbott v. Perez (585 U.S. 579, 2018)**: Correct. The paper correctly identifies this as the Texas redistricting case, correctly attributes the majority to Justice Alito, and correctly characterizes the holding as imposing a high burden on plaintiffs proving discriminatory intent. The paper correctly notes that Abbott v. Perez addressed the intent prong under the Equal Protection Clause rather than the Gingles results-based dilution analysis. This distinction is legally precise and correct.

**Beer v. United States (425 U.S. 130, 1976)**: Correct. Non-retrogression standard correct.

**Harper v. Hall (868 S.E.2d 499, N.C. 2022)**: This is a North Carolina Supreme Court case, correctly identified as a state court decision. The citation is in the correct format for the North Carolina South Eastern Reporter citation. However, the paper refers to it as the case in which "the North Carolina Supreme Court...ordered remedial redistricting plans." The Harper v. Hall litigation is more complex than this — there were multiple iterations, and the relevant remedial order involved both the NC Supreme Court and subsequent proceedings. The paper's invocation is for a brief illustrative point, and the cited decision is a real opinion, so this is P3 level.

**Covington v. North Carolina (316 F.R.D. 117, M.D.N.C. 2016)**: Correct. The claim that 28 of 170 state legislative districts were found racially gerrymandered is cited to this case — this matches the reported findings. Correct.

All R4 required citations verified. No P1 issues in D.8 on legal citations.

**P2: Abbott v. Perez applies to Equal Protection intent claims; the paper should explicitly note it did not weaken results-based Gingles dilution claims (it does say this, but the "substantially harder to prevail" language in the following sentence might mislead — tighten).**
**P3: Harper v. Hall invocation — add a brief parenthetical noting the multi-round nature of the litigation.**

---

**R5 — Liang (ML/AI/systems): 2 / 4**

D.8 is a legal survey paper, so the pipeline reproducibility claims are more modest. The SHA audit chain claim is real and documented in the CLAUDE.md. The `bisect label-verify` command is listed in the core commands. I credit the pipeline description as accurate.

The paper's claim that bisect "operates without partisan data" — using only Census demographics, geographic adjacency, and population counts — is stated in the Introduction. This claim should be verifiable against the codebase. The CLAUDE.md describes the three-layer compositor: structure, weights-override, and search. The weights-override options include "geographic," "county," "vra-aligned," "proportional," and "unweighted." None of these descriptions reference election data. The VEST 2020 presidential vote data used in D.6 and D.7 for partisan shift analysis is noted as analysis-layer data, not algorithm-layer data. I conditionally accept this claim.

However, the paper's Use Case 1 description says a bisect VRA-mode plan "creates the required majority-minority district while satisfying population balance, contiguity, and compactness requirements." Contiguity of the algorithmic output is not explicitly validated in the described test suite. The CLAUDE.md test commands do not reference a contiguity invariant test. If bisect plans can theoretically produce non-contiguous districts (e.g., in island or disconnected tract scenarios), the paper's contiguity claim needs qualification.

**P2: Verify that bisect output districts are always contiguous — add a note on how contiguity is enforced or tested.**
**P2: Clarify that "no partisan data" refers to the algorithm layer, not the analysis layer — VEST data is used post-algorithm for analysis, not as an input.**
**P3: The "SHA audit chain" claim would benefit from a one-sentence description of what is hashed and when (inputs, outputs, intermediate bisection steps) so readers unfamiliar with the bisect internals can evaluate the claim.**

---

### D.8 Consolidated P1/P2/P3 Issues

**P1 (must fix before acceptance):**
1. None identified — D.8 has no phantom pipeline features and legal citations all verify.

**P2 (revise before submission):**
1. Add worked example of neutral-baseline gap computation for one state (R1)
2. Address adversarial parameter-choice argument for bisect mode selection (R2)
3. Make explicit the Gingles prong 1 — bisect VRA mode connection (R3)
4. Tighten Abbott v. Perez language to prevent misleading implication about Gingles results claims (R4)
5. Verify bisect contiguity enforcement and note in text (R5)
6. Clarify "no partisan data" refers to algorithm layer, not analysis layer (R5)

**P3 (polish):**
1. Acknowledge geographic adjacency correlates with partisanship even without direct election data (R1)
2. Texas 2012 plan replacement timeline footnote (R2)
3. Make Gingles prong 1 geographic/algorithmic connection explicit (R3)
4. Covington v. North Carolina — ensure both the district court and SCOTUS per curiam citations appear in the text (R4)
5. Harper v. Hall litigation complexity note (R4)
6. SHA audit chain content description (R5)

---

## Panel Deliberation Notes

### D.6 (Prison Gerrymandering) — Major Revision (avg 2.2)

The paper's legal framework is sound, the Evenwel analysis is accurate, and the two L0 invariants are correctly formulated. The fatal problem is R5's finding that the `--adjust-prison-population` flag does not appear to exist in the documented codebase. If this flag is real and operational, the paper must document it in REDIST_CLI.md and reference it explicitly. If it is planned but not implemented, the paper must be reframed as a specification or design paper rather than an empirical results paper. Either path requires significant revision before the California validation claim and the NC/TX results can be credited. The R2 concern about causal attribution (single-seed partisan shift claims) compounds the problem. The paper is worth revising — the legal contribution on Evenwel and the state law survey table are valuable — but the methodology section must be rebuilt on verifiable foundation.

**Required for revision**: (1) Implement or document `--adjust-prison-population`, (2) provide NPS county-of-origin citation, (3) clarify NC data provenance.

### D.7 (Section 203) — Major Revision (avg 2.2)

D.7's legal framework is more accurate than its empirical methodology. The Section 203 coverage formula is correctly described, the 2023 Federal Register citation is correct, and the legal hierarchy (Section 2 over Section 203) is properly drawn. The problems are in the empirical claims: the 35% Texas improvement headline is unanchored without the arithmetic minimum county split count, the 25% Vietnamese-American threshold for the community definition is unjustified, and the coverage data provenance is undocumented. These are fixable revisions — the Section 203 framework analysis is publishable as is, and the empirical sections can be strengthened with the arithmetic baselines and data documentation that are currently missing.

**Required for revision**: (1) Arithmetic minimum county splits, (2) Vietnamese community threshold justification, (3) coverage data pipeline documentation.

### D.8 (Post-Shelby) — Minor Revision (avg 2.8)

D.8 is the strongest paper of the three. It has no phantom pipeline features (R5 scores 2 rather than 1), no legal citation errors (all R4 required citations verified), and no unanchored empirical claims (the paper wisely avoids single-state empirical headline claims, focusing instead on the framework). The remaining gaps are all P2/P3 level: add a worked example, address adversarial parameter-choice, clarify the "no partisan data" claim to the algorithm layer only, and add a contiguity verification note. These are one-revision fixes. The paper's central argument — algorithmic redistricting as transparency substitute for Section 5 documentation and deterrence — is well-reasoned and makes a genuine contribution to the post-Shelby redistricting literature.

**Required for revision**: Worked example for neutral-baseline gap; clarify partisan-data-layer distinction.

---

## Cross-Paper Observations

1. **Single-seed problem is systemic across D.6 and D.7**. Both papers report empirical results with the correct $^\dagger$ dagger notation, which is the project convention, but neither paper provides ensemble results or sensitivity analysis for key claims. For D.6, this means the partisan shift attributions are overclaimed. For D.7, this means the county split comparisons may fall within seed noise. D.8 avoids this problem by not reporting single-state empirical figures.

2. **Pipeline documentation gap**. D.6 describes a pipeline feature (`--adjust-prison-population`) that is not in REDIST_CLI.md. D.7 describes an analysis pipeline (Section 203 coverage data integration, Vietnamese tract identification) that is not documented anywhere. These are not minor documentation issues — they are reproducibility failures that go to the heart of what makes algorithmic redistricting a credible expert witness tool. Both papers should trigger corresponding updates to REDIST_CLI.md and/or analysis documentation.

3. **Legal citations are consistently strong**. Across all three papers, the required legal citations (Evenwel in D.6, 88 Fed. Reg. 34,396 in D.7, Shelby County/Allen v. Milligan/Brnovich/Abbott v. Perez in D.8) are correctly cited with accurate characterizations of holdings. R4 found no P1 issues in any paper on this dimension. The D-track is legally literate.

4. **VRA taxonomy is inconsistent across the track**. D.6 conflates Black and Hispanic into a combined "minority" figure in several places (which matters when Gingles analysis is disaggregated by group). D.7 handles this better. D.8 handles it correctly by focusing on Section 2 generally. The D.6 conflation should be fixed to bring it into alignment with the rest of the track.
