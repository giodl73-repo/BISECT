# Track E — Experimental Extensions: Panel Review

**Track**: E-experimental
**Module**: track-E
**Papers reviewed**: 8 (E.0–E.7)
**Panel date**: 2026-05-07

---

## Panel Composition

| # | Reviewer | Affiliation | Primary Expertise |
|---|----------|-------------|-------------------|
| P1 | Arend Lijphart | UC San Diego | Comparative electoral systems, proportional representation, multi-member districts |
| P2 | John Carey | Dartmouth | Electoral system design, legislative studies |
| P3 | Jonathan Rodden | Stanford | Political geography, geographic constraints on electoral systems |
| P4 | Nicholas Stephanopoulos | Harvard Law | Election law, normative redistricting theory |
| P5 | Sona Nadenichek Golder | Penn State | International redistricting, cross-national comparisons |
| P6 | Bryan D. Jones | UT Austin | Political economy, public choice, electoral system effects |
| P7 | Joshua Tucker | NYU | Comparative politics methodology, counterfactual analysis |

---

## Module Overview

Track E is the counterfactual track — it answers the question "what if?" by applying the algorithm to alternative rule systems. Its scientific value is twofold: (1) the alternative results illuminate what is geometric necessity vs. political choice in the current single-member system; (2) they extend the algorithm's relevance beyond US congressional redistricting to international audiences and proportional representation debates.

Track E is the most speculative track in the portfolio. Papers E.0 and E.7 are accepted (both 4.0/4) and provide strong synthesis bookends. However, papers E.1–E.6 are all in draft stage, and some have significant methodological gaps that must be resolved before submission. The panel's primary concern is that the speculative nature of the track is not consistently signaled — papers E.2 and E.5 in particular make claims that could be read as prescriptive rather than exploratory.

**Module score estimate**: 6.8 / 10 (E.0/E.7 strong anchors; E.1–E.6 require revision)

---

## Paper-by-Paper Assessments

---

### E.0 — Experimental Extensions Overview
**Score**: 4.0/4 (accepted per PAPERS.md)
**Venue**: Internal overview
**Status**: Accepted

The strongest framing paper in Track E. The overview correctly positions the experimental track as: "these papers do not represent policy recommendations — they are computational explorations designed to quantify the geometric and political constraints embedded in current electoral rules."

**Residual concern (PP3)**: E.0 was accepted before E.1–E.5's draft status became clear. If any E.1–E.5 papers fail peer review or are substantially revised, E.0's summary of their "findings" becomes stale. E.0 should be updated to reflect the actual status of companion papers before final distribution.

**Cross-track note**: E.0 references B.0's bakeoff as the baseline for all counterfactual comparisons. B.0 is accepted (4.0/4), so this dependency is solid.

---

### E.1 — Multi-Member Districts and Proportional Representation
**Score**: Unscored (draft; evaluation section empty)
**Venue**: Electoral Studies
**Status**: Draft; blocking item

**Assessment**: The paper's framing (what proportionality gain is achievable by converting single-member districts to multi-member districts using the bisection algorithm?) is well-motivated. The geographic analysis — showing that the bisection framework can be extended to k-member districts by adjusting the population denominator — is technically sound.

**Blocking item (PP1) — Empty evaluation section**:
The paper's evaluation section is marked "TODO: Add electoral formula comparison results." The paper promises to compare D'Hondt and Saint-Laguë seat allocation formulas and report their effects on proportionality. This section does not exist. The paper cannot be submitted or cited as having evaluated proportionality until this section is written.

**Electoral formula gap (PP1)**:
The D'Hondt formula systematically over-represents larger parties; Saint-Laguë is more proportional. The paper must evaluate at least both formulas and report which one is used for its headline proportionality figure. The current draft cites only D'Hondt in the methodology but does not compare it to Saint-Laguë. For Electoral Studies, this comparison is required.

**Effort to resolve**: 2–4 weeks (run Saint-Laguë comparisons, write evaluation section)

---

### E.2 — Direct County Representation
**Score**: Unscored (draft; methodological gaps)
**Venue**: State Politics and Policy Quarterly
**Status**: Draft; methodological concerns

**Assessment**: The geographic analysis of county-based redistricting is interesting and well-motivated by the county-level politics literature. The paper correctly identifies that direct county representation would require multi-member districts for large counties.

**Population equality problem (PP1)**:
The paper's abstract claims "comparable population equality" for the county-representation scheme, but the analysis shows a 25% CV (coefficient of variation) in district population. A 25% CV is not "comparable" to the ±0.5% precision of the base algorithm — it is 50× worse. This claim is materially misleading and must be corrected. The abstract should read: "population equality is substantially degraded relative to algorithmic redistricting (25% CV vs. ±0.5%)."

**Estimation vs. redistricting (PP2)**:
The headline MMD claim (197–201 multi-member districts nationally) is based on estimation from county population distributions, not from actually running the redistricting algorithm. This must be flagged explicitly in the abstract and results section.

**Speculative signaling (PP2)**:
The paper's conclusion reads as a policy recommendation ("county representation would better reflect local interests"). Track E is an exploratory track — this framing should be changed to "this analysis suggests that county representation is geometrically feasible in some states but involves substantial population equality tradeoffs."

---

### E.3 — National Redistricting Without State Boundaries
**Score**: Unscored (draft; two factual issues)
**Venue**: Comparative Political Studies
**Status**: Draft; factual inconsistencies

**Assessment**: The "geometric cost of federalism" framing — quantifying how much compactness is lost by requiring state-boundary respect — is a genuinely original contribution. No prior paper has systematically measured this cost across all 50 states.

**Factual inconsistency (PP1) — Beta discrepancy**:
The paper reports 176 cross-state districts in the introduction and 180 in the results section. One of these is incorrect. The actual number must be verified from the redistricting output and used consistently throughout.

**Compactness metric scoping (PP2)**:
The "geometric cost of federalism" claim uses Polsby-Popper as the compactness metric. However, PP measures district compactness, not the "cost" of state-boundary constraints on the national redistricting problem. The paper should either: (a) use a different metric that directly quantifies cross-state connectivity loss; or (b) clearly state that the PP comparison is a proxy for the federalism cost and note its limitations as a direct measure.

---

### E.4 — Partisan Similarity Districts: Algorithmic Safe Seats
**Score**: Unscored (draft; paper not written beyond outline)
**Venue**: AJPS
**Status**: Outline only; not written

**Assessment**: The concept — using the bisection algorithm to create geographically contiguous districts whose residents have similar partisan preferences — is interesting as a counterfactual. If gerrymandering creates artificial safe seats, what would organic safe seats look like?

**Scoping concern (PP1)**:
The outline does not clearly distinguish E.4 from E.5. E.4 is described as "algorithmic safe seats" (geographic clustering of similar voters) and E.5 as "partisan fairness through algorithmic districting" (explicit partisan optimization). These descriptions suggest substantial methodological overlap. Before writing begins, the authors must clearly define what E.4 does that E.5 does not.

**Not yet written**: Cannot be reviewed in substantive depth until a manuscript exists.

---

### E.5 — Partisan Fairness Through Algorithmic Districting
**Score**: Unscored (draft; results section absent)
**Venue**: Electoral Studies
**Status**: Draft; results section absent

**Assessment**: The paper's framing creates significant risk: it appears to advocate for explicitly partisan redistricting optimization, which contradicts the program's core "neutral algorithm" claim.

**EG-zero mischaracterization (PP1)**:
The paper's introduction states "efficiency gap = 0 means partisan fairness." This is incorrect. Efficiency gap = 0 is one measure of partisan symmetry but does not equal partisan fairness more broadly (it does not address proportionality, voter equality, or other fairness dimensions). More critically: C.5 shows that algorithmic maps already produce efficiency gap ≈ -3.2% (not 0) due to geographic sorting. The paper's goal of EG = 0 requires explicit partisan optimization, which contradicts the program's neutrality claim.

**Missing results section (PP1)**:
The paper has no results section. The introduction and methodology are written but the empirical analysis is absent.

**Gerrymandering-by-another-name concern (PP2)**:
Any algorithm that explicitly targets EG = 0 is optimizing for a partisan outcome, which is precisely what the program argues algorithmic redistricting avoids. The paper must directly address this tension in its framing: it is exploring whether partisan-targeted optimization can be made transparent and rule-bound (a legitimate counterfactual), not whether the program should adopt partisan optimization.

---

### E.6 — International Applications
**Score**: Unscored (draft; results partially missing)
**Venue**: Electoral Studies
**Status**: Partial draft

**Assessment**: The cross-national application is genuinely novel — no paper in the redistricting literature applies the METIS bisection framework to non-US electoral systems. The UK, Canada, and Australia parliamentary system applications are well-motivated.

**Missing results (PP1)**:
The UK, Canada, and New Zealand results are absent from the paper. The introduction describes them as completed; the results section has placeholders. These must be completed before review.

**STV/D'Hondt approximation (PP2)**:
The paper uses D'Hondt as the seat allocation formula for multi-member proportional systems. The STV (Single Transferable Vote) approximation is mentioned in a TODO comment. For Ireland (the paper's primary PR example), STV is the actual formula used — using D'Hondt misrepresents the system being compared to. This must be elevated from a TODO to an explicit methodological choice with justification or replacement.

**Minority representation framework (PP2)**:
The paper compares district compactness but does not address minority representation differences between the US single-member and international multi-member systems. For international venues, this omission will draw reviewer comment.

---

### E.7 — Lessons Learned from Six Alternative Systems
**Score**: 4.0/4 (accepted per PAPERS.md)
**Venue**: Internal synthesis
**Status**: Accepted

**Assessment**: The second-strongest paper in Track E. The synthesis correctly identifies the track's key lessons:
1. Geographic sorting is a structural constraint that any electoral system must accommodate
2. Multi-member districts improve proportionality but require different compactness metrics
3. The "geometric cost of federalism" is measurable and non-trivial
4. Electoral system design tradeoffs are geographic as much as political

**Dependency note (PP3)**:
E.7 cites E.1–E.6 throughout and draws conclusions from their findings. If any E.1–E.6 papers change their empirical results through revision, E.7's synthesis conclusions must be updated. E.7 was accepted before E.1–E.6 were reviewed — the synthesis conclusions rest on unreviewed research.

Three P2 items remain in E.7's own revision plan (per review files): a more explicit acknowledgment that conclusions from unreviewed companion papers are preliminary; a note on the D'Hondt vs. Saint-Laguë sensitivity; and a cross-reference to G.14's practitioner comparison for the "when to use each approach" framing.

---

## Cross-Track Issues

### Issue X1 — Asymmetric Completion Risk
E.0 and E.7 (the synthesis bookends) are accepted. E.1–E.6 (the empirical papers synthesized) are all in draft with blocking items. If E.1, E.5, or E.6 fail peer review or require major revisions, E.0 and E.7 cite non-existent or substantially changed findings.

**Required**: Add "contingent on companion paper acceptance" footnotes to E.0 and E.7 for all citations of E.1–E.6 findings.

### Issue X2 — D'Hondt vs. Saint-Laguë Consistency (E.1, E.5, E.6)
Three papers use D'Hondt for their electoral formula comparisons, but none justifies this choice over Saint-Laguë. Electoral Studies reviewers will ask why. A track-level policy decision should be made: either use both formulas consistently across all three papers (reporting both D'Hondt and Saint-Laguë results) or explicitly justify the D'Hondt choice in each paper's methodology section.

### Issue X3 — E.4/E.5 Separation Risk
E.4 (partisan similarity districts) and E.5 (partisan fairness through algorithmic districting) potentially overlap substantially. Before writing E.4 (currently outline only), the authors must clearly delineate: (a) E.4's geographic clustering approach vs. E.5's explicit optimization approach; (b) what each paper uniquely contributes that the other does not. This should be documented in a "Scope" section added to E.4's outline.

### Issue X4 — Speculative-Nature Signaling (E.2, E.5)
E.2's abstract and conclusion read as policy-prescriptive rather than exploratory. E.5's EG = 0 goal contradicts the program's neutrality claims. Both papers must add explicit "this is a counterfactual exploration, not a policy recommendation" language in their abstracts and conclusions.

### Issue X5 — E.7 Cross-Citations Not Verifiable
E.7 cites E.1's proportionality results and E.4's partisan similarity findings. E.1's evaluation section doesn't exist (PP1). E.4 is outline only. E.7's synthesis conclusions for these two papers rest on data that hasn't been generated or written up.

---

## Module Score

**Score: 6.8 / 10**
**Tier: Conditionally adequate** — strong synthesis bookends (E.0, E.7) but weak empirical core (E.1–E.6 all in draft with blocking items)

| Sub-track | Papers | Status | Score estimate |
|-----------|--------|--------|----------------|
| Alternatives (E.1–E.6) | 6 | All draft; 4 have PP1 items | 6.0 / 10 |
| Synthesis (E.0, E.7) | 2 | Both accepted (4.0/4) | 9.0 / 10 |
| **Module** | **8** | Mixed | **6.8 / 10** |

---

## Priority Items

### PP1 — Blocking

**PP1-E1 — E.1 empty evaluation section**
Electoral formula comparison results (D'Hondt vs. Saint-Laguë) are marked TODO. Paper cannot be submitted or cited until written.

**PP1-E2 — E.2 "comparable equality" mischaracterization**
25% CV is not comparable to ±0.5%. Abstract must be corrected to accurately report the population equality cost of county-representation.

**PP1-E3 — E.3 beta discrepancy**
176 vs. 180 cross-state districts — one is incorrect. Must verify and make consistent throughout.

**PP1-E5 — E.5 EG = 0 mischaracterization and missing results**
EG = 0 ≠ partisan fairness (and contradicts the program's neutrality claims). Results section must be written.

**PP1-E6 — E.6 missing results**
UK, Canada, NZ results are absent despite being described as complete in the introduction.

### PP2 — Important

**PP2-E2 — E.2 estimation-vs-redistricting disclosure**
197–201 MMD claim is estimation, not actual redistricting. Must be disclosed in abstract.

**PP2-E5 — E.5 gerrymandering framing**
Explicit EG = 0 optimization contradicts the program's neutrality claims. Must directly address this tension.

**PP2-E6 — E.6 STV/D'Hondt approximation elevation from TODO**

### PP3 — Further strengthening

**PP3-E0 — E.0 status updates for companion papers**

**PP3-E7 — E.7 three pending P2 items**

*Panel convened 2026-05-07. Track E — eight papers, two sub-tracks: alternatives and synthesis.*
