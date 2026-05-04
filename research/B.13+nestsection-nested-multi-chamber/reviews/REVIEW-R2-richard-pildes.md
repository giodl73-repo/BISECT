# Review: NestSection — Richard Pildes (Round 2)

**Reviewer**: Richard Pildes (NYU School of Law)
**Expertise**: Constitutional law, election law, separation of powers, redistricting doctrine, Voting Rights Act
**Round**: 2
**Score**: 2.5/4 (major revision, reduced concerns)
**Recommendation**: Major Revision — Conditional Upgrade to Accept

---

## Response to Round 1 Concerns

The revision takes a meaningful step on the Elections Clause issue, which was the most serious concern I raised in Round 1. The new Elections Clause paragraph in §5.1 explicitly scopes the mandate to state legislative maps. This is a substantive improvement. However, three of my four major concerns remain substantially open, and two of them — the Arizona/Indiana analog and the judicial enforceability gap — are requirements for publication in a law review audience.

### M1: Elections Clause — Partially addressed

The new Elections Clause paragraph is the most important addition in this revision. The paper now states clearly that it does not advocate for a federal mandate constraining congressional maps under Art. I, §4. It correctly identifies the state constitutional law framework as the governing authority for the proposed mandate (state compactness, contiguity, and political subdivision requirements). The distinction between (a) voluntary adoption by a commission, (b) state-legislated requirement for state maps, and (c) a federal mandate binding congressional maps is now present, with (c) explicitly disclaimed.

What still requires development: the paragraph states that states "voluntarily adopting NestSection for their state legislative chambers" face "no federal constitutional barrier." This is likely correct for state legislative maps drawn by state commissions, but the analysis is conclusory. The paper should cite the relevant line of doctrine: *Reynolds v. Sims*, 377 U.S. 533 (1964), as the constitutional baseline for state legislative apportionment (one-person-one-vote with somewhat more flexibility than *Wesberry*), and note that a spine-sharing requirement that maintains population equality within each chamber would survive *Reynolds* scrutiny because it imposes a geometric constraint compatible with equal population.

Additionally, the paper should acknowledge what happens when a state commission draws the congressional map and a different body draws state legislative maps under a NestSection mandate. If the congressional map is drawn without NestSection, the "pre-existing congressional spine" that state legislative commissions would rely on may not conform to the GeoSection algorithm at all. The paper implicitly assumes the congressional map is drawn first and serves as a fixed input, but does not state this assumption or analyze what happens when it fails.

### M2: Article I §2 hook — Resolved by scoping

By explicitly disclaiming the federal mandate scenario, the paper sidesteps the Art. I §2 question I raised. The Art. I §2 hook referenced in the plan metadata no longer appears in §5.1 without support. The state constitutional law framing is a more defensible basis than any federal hook, and the paper's pivot to that framing is the right move. I consider M2 resolved.

### M3: Arizona/Indiana analog — Not adequately addressed

This remains my primary concern. The paper still mentions Arizona and Indiana in one sentence: "Some statutes require state house districts to be subsets of senate districts (e.g., Arizona, Indiana)." Round 1 required: (a) describe Arizona's requirement and its constitutional/statutory/administrative status; (b) explain whether the NestSection spine requirement is stronger or weaker than Arizona's; (c) use Arizona as the strongest evidence that courts accept cross-chamber nesting.

None of these three sub-requirements are in the revision. For a Harvard Journal on Legislation audience, the Arizona house-in-senate precedent is the single most important legal anchor for the paper's proposal. Arizona's independent redistricting commission draws state legislative maps under Art. IV, Part 2, §1(14) of the Arizona Constitution, which requires legislative districts to be drawn so that house districts are "within a senate district" (Prop. 106, adopted 2000). This provision has been litigated (Harris v. Arizona Independent Redistricting Commission, 579 U.S. 872 (2016)) and upheld. The NestSection proposal extends this two-chamber nesting to a three-chamber nesting by adding the congressional tier. Developing this connection would take two paragraphs and would transform the legal argument from "potentially permissible" to "grounded in existing practice."

**Required**: Add a paragraph developing the Arizona/Indiana analog, addressing (a)–(c) as specified in Round 1. This is not optional for a law review publication.

### M4: Judicial enforceability — Not addressed

The paper contains no discussion of how a court would enforce a NestSection mandate. The Round 1 required: (a) a court-appointed expert computing the CompatibleSpines spine; (b) a standard for comparing the submitted map to the algorithmic spine; (c) a remedy if the map fails the test.

The paper's statutory design proposal (§5.1, final paragraphs) identifies a $g \geq 5$ threshold and a $g \geq 2$ best-effort tier, but does not describe the enforcement mechanism. For Oregon and Alabama, the spine is deterministic: given the state's census graph, CompatibleSpines$(C, S, H)$ returns a unique (up to ordering convention) trunk. A legislature challenging a NestSection mandate would need to know: what is the legal standard for compliance? Is a map NestSection-compliant if it produces districts whose boundaries match the algorithmic spine within some geographic tolerance? Who computes the algorithmic spine — the commission, the legislature, or a court-appointed expert?

These questions are not difficult to answer, but they must be answered somewhere in the paper. A one-paragraph sketch would suffice: "A NestSection compliance challenge would proceed as follows: (1) the algorithmic spine for the state is computed deterministically from the census data and is the same regardless of which party runs the computation; (2) the submitted plan is evaluated by whether each chamber's districts respect the trunk boundaries, measured by the boundary-zone fraction tau; (3) a plan with tau exceeding the statutory threshold is non-compliant and subject to judicial remedy. The objective, computable nature of the standard is its primary legal advantage over subjective compactness requirements."

**Required**: Add a paragraph (can be brief) addressing the enforcement mechanism for the statutory proposal.

---

## Additional Issues

### Gerrymander resistance claim (new concern)

The revised §5.1 still contains the claim that NestSection "reduces the feasible plan space available for partisan optimization." This is asserted as a structural fact, but as my colleague Duchin has noted, no ensemble comparison supports it. For a law review audience, an unsupported structural claim about anti-gerrymandering effect is exactly the kind of claim that opposing expert witnesses will attack in litigation. The paper should explicitly state that this is a conjecture awaiting empirical validation, not a demonstrated finding.

### Proposed threshold coverage (carried forward)

I raised in Round 1 that a mandate covering only 2 states ($g \geq 5$: Oregon and Alabama) is a narrow starting point for statutory design. The revision explains this narrowness honestly — it reflects the actual structure of US apportionments — but the paper could strengthen the best-effort tier ($g \geq 2$, 22 states) as the more practically significant statutory proposal. At 22 states, the best-effort mandate becomes a plausible legislative agenda item, not a curiosity.

### The 30–50% variance reduction estimate

Section 5.3 still contains "NestSection reduces partisan variance by approximately 30–50% in moderate-compatibility states." This estimate has no basis in the paper. It should be removed or explicitly labeled as a working hypothesis. For a law review, this kind of ungrounded quantitative claim is a liability.

---

## Assessment

The Elections Clause scoping is a genuine improvement that partially addresses my most serious concern. The paper is now on the right side of the federal constitutional line for its core proposal. But the Arizona/Indiana analog — the strongest legal foundation for the proposal — remains underdeveloped, and the judicial enforceability gap is still present. Both are standard requirements for a statutory design paper in a law review. I am upgrading my score from 2 to 2.5 to reflect the improvement, but I cannot recommend acceptance until M3 (Arizona/Indiana) and M4 (enforceability) are addressed. These two revisions are achievable in a targeted second revision without restructuring the paper.

**Score: 2.5/4 — Major revision, conditional upgrade to accept upon addressing M3 and M4.**
