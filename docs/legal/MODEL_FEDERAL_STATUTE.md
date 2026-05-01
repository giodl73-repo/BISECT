# Model Federal Statute: Districting Integrity Act

**Status:** v0.1 draft, 2026-05-01 (post 5-role review)
**Audience:** congressional staff (Senate Judiciary, House Administration), reform-organization legal teams, academic election-law specialists.
**Companion docs:** `STATUTE_RATIONALE.md` (policy memo), `STATUTE_ONE_PAGER.md` (90-second version), `STATUTE_REVIEW_NOTES.md` (open tensions + Option B architectural alternative), `FAIRNESS_DOCTRINE.md` (state-court companion).

This document proposes statutory text mandating that each State draw its congressional districts using a deterministic, partisan-blind algorithm (recursive bisection) and publish reproducibility artifacts that allow any person to verify the map's compliance byte-identically.

The architecture is deliberately analogous to **2 U.S.C. § 2a** (the Huntington-Hill apportionment statute): Congress prescribes the method; states execute; verification is mechanical.

---

## Changes from v0 → v0.1 (in response to 5-role review)

The v0 draft was reviewed by five stakeholder roles (constitutional law professor, Senate Judiciary staff drafter, VRA § 2 plaintiffs' counsel, state election director, hostile constitutional challenger). v0.1 incorporates the consensus must-fix items:

**Constitutional hardening:**
- Short title renamed to "Districting Integrity Act" (political register; "Reproducibility" reads as scientific-computing jargon outside the technical audience). Long title preserved.
- New § 101(f) and § 109(d) — dual constitutional authority under Article I § 4 AND § 5 of the Fourteenth Amendment, so the bill survives if a future Court narrows Elections Clause scope (*Moore v. Harper* dissent vote count is not gone).
- New § 101(g) — explicit Murphy v. NCAA / Printz rebuttal in findings, citing *Foster v. Love* (1997) and *Cook v. Gralike* (2001).
- § 104(b)(4)-(5) — tolerance schedule and edge-weight specification moved into statutory text. Director of Census now sets only the seed and binary SHA-256, narrowing the § 107 delegation to defuse non-delegation / *West Virginia v. EPA* attack.
- § 109(a) — fixed: § 104(e) is now non-severable from § 104. Striking the prohibited-inputs rule does not leave a partisan-optimization-mandate behind.

**VRA § 2 strengthening:**
- § 106(b) — "may modify" → "shall modify" where *Gingles* + *Senate Factors* compel.
- New § 106(b)(2) — failure to modify creates no presumption of § 2 compliance; the algorithm's output is not a defense.
- § 106(c) — restructured around *Allen v. Milligan*'s narrow-tailoring framework rather than a bare *Gingles* recital, to soften *Miller v. Johnson* race-predominance exposure.
- § 106(d) — presumption limited to *Shaw*-type race-predominance challenges; expressly excludes § 2 effect claims.
- New § 106(e) — coalition-district claims expressly preserved.
- § 106(f) — population deviation under § 104(c)(3) may exceed 0.5% when § 2 requires.

**Operational must-haves (state director's list):**
- § 105(a) — tiered publication deadline (180 days for ≤8 districts; 270 days otherwise).
- New § 105(e) — 12-year data-preservation mandate for input files (needed for byte-identity challenges to actually function).
- New § 107(c)(6) — late-publication backstop: prior cycle's parameter table applies if Director misses the 18-month deadline.
- New § 107(c)(7) — TIGER adjacency errata mechanism.
- New § 107(d) — reference-implementation defect process: bug in the federal binary is a complete safe-harbor defense.
- New § 107(e)(1)(C) — 30-day pre-suit cure period for non-VRA technical defects.
- § 107(e) standing — rewritten per *Hunt v. Washington Apple* three-part test.
- § 107(f) — venue clause invokes 28 U.S.C. § 2284 three-judge district court.
- § 107(g) — asymmetric fee-shifting (only § 2-protective plaintiffs recover).
- § 107(h) — criminal penalty for § 104(e) violations.
- New § 108(d) — automatic standing for § 108(c) interim-plan triggering.
- § 108(e) — census-release timing relief; clock pause for late parameter table; auto-extension keyed to state primary calendars.

**Drafting craft fixes:**
- § 102 — defines `edge-cut`, `level of recursion`, `non-trivial geographic boundary`.
- § 102(1) — decennial census reference locked to parameter-table identifier.
- § 105(d) — byte-identity rule restructured as two artifacts: `algorithm_output.json` (must be byte-identical to verifier) + `final_assignments.json` (whose diff against the former must equal the tracts enumerated in the § 106(c) justification).
- *Gingles* third-prong language fixed (no more shorthand "white bloc voting").
- Cites *Branch v. Smith* (2003) added in drafting notes.

**State-law collision (new section):**
- New § 110 — state-constitutional integrity rules (town/county integrity, "compact and contiguous"). Bounded "integrity deviation" parallel to § 106 with bright-line cap.

The v0.1 draft does **not** restructure as conditional spending under *South Dakota v. Dole* (Option B in `STATUTE_REVIEW_NOTES.md`). That architectural alternative remains available if reform-coalition feedback indicates the mandate posture won't reach 60 votes. See `STATUTE_REVIEW_NOTES.md` § 2 for the trigger conditions.

---

## Drafting conventions

This is a model bill, not enacted law. Section numbers (`§ 101`, `§ 102`, ...) are placeholders for future Title 2 codification (proposed slot: **2 U.S.C. §§ 2d–2j**, immediately after § 2c). Defined terms appear in `SMALL CAPS` on first use within a section. Cross-references use `§ X(y)(z)` form.

---

## Title

**Districting Integrity Act of [Year]**
*(Short title. Long title: "Reproducible Congressional Districting Act of [Year].")*

## § 101. Findings and purpose

Congress finds that —

(a) Article I, section 4 of the Constitution authorizes Congress to prescribe the manner of holding elections for the House of Representatives, and Congress has previously exercised this power to require single-member districts (2 U.S.C. § 2c) and substantial population equality among districts (consistent with *Wesberry v. Sanders*, 376 U.S. 1 (1964); *Branch v. Smith*, 538 U.S. 254 (2003));

(b) In *Rucho v. Common Cause*, 588 U.S. 684 (2019), the Supreme Court held that partisan-gerrymandering claims are nonjusticiable in federal court because federal courts lack a "judicially manageable standard," but expressly left to Congress and the States the question of how to constrain partisan gerrymandering through positive law;

(c) Apportionment of Representatives among the States is governed by 2 U.S.C. § 2a, which mandates the equal-proportions method (Huntington-Hill); this statute has resolved disputes over apportionment by specifying a single deterministic algorithm whose output any person can verify;

(d) Modern computational geography permits a similar deterministic specification for the drawing of district lines within a State, in a manner that is partisan-blind, race-blind in its core operation, and byte-reproducible from public inputs;

(e) A reproducibility-based standard produces an objective baseline against which Voting Rights Act § 2 deviations and other constitutionally required adjustments may be evaluated, without empowering Congress or any federal agency to draw the lines themselves;

(f) Congress acts in this Act under both (i) Article I, section 4 of the Constitution and (ii) section 5 of the Fourteenth Amendment, exercising its enforcement authority under *Reynolds v. Sims*, 377 U.S. 533 (1964), to ensure substantial equality of voting power among congressional districts; either authority alone is independently sufficient to support every operative provision of this Act;

(g) The mandates in this Act are direct constraints on the Manner of holding elections within Article I, section 4, and are consistent with *Foster v. Love*, 522 U.S. 67 (1997), *Cook v. Gralike*, 531 U.S. 510 (2001), and *Smiley v. Holm*, 285 U.S. 355 (1932); they do not constitute commandeering of state officers under *Printz v. United States*, 521 U.S. 898 (1997), or *Murphy v. National Collegiate Athletic Ass'n*, 138 S. Ct. 1461 (2018), because the Elections Clause expressly authorizes federal alteration of State regulations governing the Manner of congressional elections, and the Tenth Amendment does not constrain powers the Constitution affirmatively assigns to Congress.

The purpose of this Act is to require each State to draw its congressional districts using a deterministic algorithm, applied to public inputs, in a manner that any person may verify, while preserving the State's role as the line-drawing authority and protecting Voting Rights Act § 2 compliance.

## § 102. Definitions

As used in this Act —

(1) **TRACT** means a census tract as defined by the Bureau of the Census in the decennial census release identified in the parameter table under § 107(c)(4), identified by its 11-character GEOID.

(2) **TRACT POPULATION** means the resident population of a TRACT as enumerated by the Bureau of the Census in that decennial census, with no post-release adjustment except as a numbered correction issued under § 107(c)(7).

(3) **TRACT ADJACENCY GRAPH** means the undirected graph whose vertices are the TRACTS within a State and whose edges connect TRACTS that share a NON-TRIVIAL GEOGRAPHIC BOUNDARY, as further specified in subsection (10).

(4) **PARTITION** means an assignment of every TRACT in a State to exactly one congressional district.

(5) **CONTIGUOUS** means, with respect to a district, that the TRACTS assigned to that district form a connected subgraph of the TRACT ADJACENCY GRAPH.

(6) **POPULATION DEVIATION** means, with respect to a district, the absolute value of the difference between the district's total TRACT POPULATION and the State's average district population, expressed as a percentage of the average district population.

(7) **EDGE-CUT** means, for a given PARTITION, the sum of the edge weights (as specified in § 104(b)(5)) of every edge of the TRACT ADJACENCY GRAPH whose endpoints are assigned to different districts. The algorithm under § 104 minimizes the EDGE-CUT subject to the population-balance constraint at each LEVEL OF RECURSION.

(8) **LEVEL OF RECURSION** means, in the recursive bisection method specified in § 104, the depth at which the current subgraph is being partitioned. Level 1 is the State as a whole; level 2 partitions each level-1 part; etc. Total levels equal `ceiling(log2(N))` where N is the number of districts apportioned to the State under 2 U.S.C. § 2a.

(9) **REFERENCE IMPLEMENTATION** means the open-source software designated under § 107 that implements the algorithm specified in § 104 and produces the artifacts specified in § 105.

(10) **NON-TRIVIAL GEOGRAPHIC BOUNDARY** means a shared boundary between two TRACTS in the Census Bureau TIGER/Line shapefiles for the relevant decennial census whose length is non-zero. Point-contact adjacencies (vertex-only intersections), water-only boundaries, and sliver-polygon artifacts are excluded except as provided by a TIGER errata correction under § 107(c)(7).

(11) **REPRODUCIBILITY ARTIFACTS** means the files specified in § 105(b).

(12) **ALGORITHM OUTPUT** means the unmodified PARTITION produced by executing the algorithm in § 104 against the inputs in § 104(b), prior to any modification under § 106 or § 110.

## § 103. Mandate

Each State shall draw its congressional districts in accordance with this Act. A district plan that does not conform to this Act is invalid for use in any election for the House of Representatives, except as provided in § 106 (Voting Rights Act compliance), § 108 (transitional provisions), or § 110 (state-constitutional integrity).

## § 104. Method

(a) **ALGORITHM**. Each State shall draw its districts using recursive graph bisection on the TRACT ADJACENCY GRAPH, applied with the parameters specified in subsection (b).

(b) **PARAMETERS**. The recursive bisection shall be performed as follows —

   (1) the input graph is the State's complete TRACT ADJACENCY GRAPH as defined in § 102(3);

   (2) vertex weights are the TRACT POPULATIONS;

   (3) the algorithm computes the PARTITION by recursive bisection: at each LEVEL OF RECURSION, the current subgraph is partitioned into two parts approximately equal in total vertex weight, minimizing the EDGE-CUT, subject to subsection (b)(4); the recursion terminates when the partition count equals the number of congressional districts apportioned to the State under 2 U.S.C. § 2a;

   (4) **POPULATION-BALANCE TOLERANCE SCHEDULE.** At each LEVEL OF RECURSION, the imbalance between the two parts (the lighter part's vertex weight as a fraction of the heavier part's weight) shall not exceed:

   | Level | Tolerance |
   |---|---|
   | 1 | 0.005 |
   | 2 | 0.005 |
   | 3 | 0.010 |
   | 4 | 0.015 |
   | 5+ | 0.020 |

   These values are statutory; the Director of Census may not modify them by parameter-table publication;

   (5) **EDGE-WEIGHT SPECIFICATION.** Edge weights are the geographic-boundary lengths in meters as recorded in the TIGER/Line shapefiles for the relevant decennial census, rounded to the nearest integer. This specification is statutory; the Director of Census may not modify it;

   (6) **METHOD ANCHOR.** "Recursive graph bisection" and "edge-cut minimization" as used in this section refer to the multilevel graph partitioning method as implemented in the METIS library (Karypis & Kumar 1998, ACM J. Experimental Algorithmics) and substantially equivalent open-source successors. The REFERENCE IMPLEMENTATION designated under § 107(a) shall implement this method using a deterministic minimization (fixed seed; identical inputs produce identical outputs); approximation heuristics within METIS are permitted only insofar as they do not introduce non-determinism;

   (7) the random seed is fixed at the value specified in the parameter table under § 107(c)(1) (the parameter table SHALL specify a single seed across all States; the algorithm SHALL be deterministic given the seed and inputs).

(c) **OUTPUT REQUIREMENTS**. The PARTITION produced by subsection (a) shall —

   (1) assign every TRACT in the State to exactly one district;

   (2) produce CONTIGUOUS districts;

   (3) produce districts each of whose POPULATION DEVIATION does not exceed 0.5 percent, except as authorized by § 106(f) (greater deviation when § 2 of the VRA requires) or § 110(c) (greater deviation when state-constitutional integrity rule applies).

(d) **NO DISCRETION**. Subject to § 106 (VRA), § 110 (state-constitutional integrity), and § 107(d) (reference-implementation defects), the State's role under this section is to execute the algorithm faithfully and publish the result.

(e) **PROHIBITED INPUTS** *(non-severable from § 104(a)–(d) per § 109(a))*. The State shall not provide as input to the algorithm any data other than the inputs specified in subsection (b). It is unlawful for the State, in performing the act required by this section, to use as algorithm input any voter registration data, past election results, partisan affiliation data, racial composition data, or any other data not enumerated in subsection (b). This prohibition is integral to the algorithm specification and is not severable; if this subsection is held invalid, subsection (a) is also invalid.

## § 105. Reproducibility artifacts

(a) **PUBLICATION REQUIREMENT**. Each State shall publish the REPRODUCIBILITY ARTIFACTS to a publicly accessible internet location within —

   (1) 180 days after the public release of the decennial census tract-level population data, for States with 8 or fewer congressional districts; or

   (2) 270 days after that release, for States with 9 or more congressional districts.

(b) **REQUIRED ARTIFACTS**. The REPRODUCIBILITY ARTIFACTS are —

   (1) `algorithm_output.json`: a JSON object mapping every TRACT GEOID in the State to its district assignment AS PRODUCED by the algorithm under § 104, BEFORE any modification under § 106 or § 110; this file must be byte-identical to the output produced by independent execution of the REFERENCE IMPLEMENTATION on the same inputs;

   (2) `final_assignments.json`: a JSON object mapping every TRACT GEOID to its district assignment AFTER any § 106 / § 110 modifications; conforms to the schema published by the REFERENCE IMPLEMENTATION;

   (3) `manifest.json`: a JSON object recording the State, year, decennial census release identifier, REFERENCE IMPLEMENTATION version, parameter table identifier, random seed, SHA-256 of every input file, SHA-256 of `algorithm_output.json`, SHA-256 of `final_assignments.json`, the diff between the two assignment files, and the publication timestamp;

   (4) any § 106 deviation justification documents required under § 106(c);

   (5) any § 110 integrity-deviation justification documents required under § 110(d);

   (6) SHA-256 hashes recorded in `manifest.json` shall be computed using the standard SHA-256 algorithm (FIPS PUB 180-4).

(c) **VERIFICATION RIGHT**. Any person may, at any time after publication —

   (1) download the REPRODUCIBILITY ARTIFACTS;

   (2) execute the REFERENCE IMPLEMENTATION on the same inputs; and

   (3) compare the resulting `algorithm_output.json` to the State's published file under subsection (b)(1).

(d) **BYTE-IDENTICAL REQUIREMENT.** The State's published `algorithm_output.json` (subsection (b)(1)) shall be byte-identical to the result of the verification described in subsection (c)(2)–(3). The State's published `final_assignments.json` (subsection (b)(2)) shall differ from `algorithm_output.json` only in tract assignments enumerated in the § 106(c) and § 110(d) justifications. A failure of byte-identity in `algorithm_output.json`, or a diff in `final_assignments.json` not justified by § 106 or § 110, creates a rebuttable presumption that the State's plan is invalid under § 103.

(e) **DATA PRESERVATION**. Each State shall preserve, for not less than 12 years from publication, all input files (TIGER shapefiles, decennial census tract-population data) used to produce the REPRODUCIBILITY ARTIFACTS, with their SHA-256s as recorded in `manifest.json`. Failure to preserve creates an adverse inference in any § 107(e) action that the State did not in fact use the inputs claimed.

## § 106. Voting Rights Act compliance

(a) **VRA § 2 PRIORITY**. Nothing in this Act diminishes any State's obligation under section 2 of the Voting Rights Act of 1965 (52 U.S.C. § 10301), as interpreted by the Supreme Court in *Allen v. Milligan*, 599 U.S. 1 (2023), and successor authorities.

(b) **DEVIATION FROM ALGORITHM OUTPUT**.

   (1) Where the *Gingles* preconditions (*Thornburg v. Gingles*, 478 U.S. 30 (1986)) are met and the *Senate Factors* totality of circumstances indicates that section 2 requires an additional opportunity district, the State **shall** modify specific districts as necessary to bring the plan into compliance with section 2.

   (2) Failure to modify under paragraph (1) creates **no presumption** that section 2 has been satisfied. The ALGORITHM OUTPUT, standing alone, is not evidence of section 2 compliance and is not a defense to a section 2 claim. A State that fails to modify when paragraph (1) requires modification is liable under section 2 of the Voting Rights Act on the same terms as a State that draws a non-compliant plan by any other method.

(c) **NARROW-TAILORING JUSTIFICATION**. For each district modified under subsection (b), the State shall publish, as part of the REPRODUCIBILITY ARTIFACTS under § 105(b)(4), a written justification meeting the *Allen v. Milligan* narrow-tailoring framework, containing —

   (1) the district number(s) modified;

   (2) the *Gingles* analysis supporting the modification: (A) numerosity of a sufficiently large and geographically compact minority group; (B) political cohesion of that group; (C) majority bloc voting sufficient usually to defeat the minority's preferred candidate;

   (3) the specific TRACTS reassigned, before-and-after their district assignments;

   (4) the resulting demographic composition of the modified district(s);

   (5) a narrow-tailoring statement: that the modification is no broader than necessary to remedy the section 2 violation, and does not subordinate traditional districting principles further than section 2 compels;

   (6) an evidentiary record sufficient to permit *de novo* judicial review under § 107(e), including expert reports on racially polarized voting and a numerosity demonstration.

(d) **REBUTTABLE PRESUMPTION (Shaw-only)**. A modification under subsection (b) is presumptively valid against a *Shaw v. Reno* / *Miller v. Johnson* race-predominance challenge if the State publishes the justification required by subsection (c). The presumption may be rebutted in litigation by a preponderance-of-the-evidence showing that the modification exceeds what is necessary under *Allen v. Milligan*'s narrow-tailoring requirement.

   This presumption applies **only** to *Shaw*-type challenges. It does not apply to, and does not affect the burdens in, section 2 effect-prong claims, which proceed under the standard *Gingles*-and-*Senate-Factors* framework with burdens unmodified by this Act.

(e) **COALITION-DISTRICT CLAIMS PRESERVED**. Nothing in this section forecloses any section 2 theory recognized by binding circuit authority at the time of suit, including but not limited to coalition-district claims (*LULAC v. Perry*, 548 U.S. 399 (2006), as interpreted in the relevant circuit). Subsection (b)'s reference to "the *Gingles* preconditions" is illustrative, not exclusive of other section 2 theories.

(f) **POPULATION DEVIATION ACCOMMODATION**. Where compliance with section 2 of the Voting Rights Act under subsection (b)(1) requires a district to exceed the 0.5 percent POPULATION DEVIATION cap in § 104(c)(3), the State may exceed the cap to the minimum extent necessary to maintain political cohesion or geographic compactness in the section-2-protected community, with written justification under subsection (c).

(g) **PUBLIC HEARINGS**. State public hearings on congressional districting, where conducted, shall be repurposed toward gathering evidence relevant to subsection (c)(2) (racially polarized voting, *Senate Factors* totality of circumstances). Communities-of-interest input is not algorithm input under § 104(e); States may, but are not required to, treat it as supplementary documentation under § 105(b)(4).

## § 107. Reference implementation; parameter table; enforcement

(a) **DESIGNATION**. The Director of the Census, in consultation with the Director of the National Institute of Standards and Technology and the expert panel described in subsection (b), shall designate an open-source REFERENCE IMPLEMENTATION of the algorithm specified in § 104.

(b) **EXPERT PANEL**. The expert panel shall consist of —

   (1) one expert in graph partitioning algorithms;

   (2) one expert in election geography or political science with subject-matter expertise in redistricting;

   (3) one expert in software reproducibility or scientific computing.

   Members shall be selected by the Director of NIST from a pool of nominees solicited from the National Academies of Sciences, Engineering, and Medicine.

(c) **PARAMETER TABLE**. The Director of the Census shall publish, no later than 18 months before each decennial census tract release, the parameter table, specifying —

   (1) the random seed (a single value applied to all States);

   (2) the version of the REFERENCE IMPLEMENTATION designated under subsection (a) and the SHA-256 of its compiled binary for each supported platform;

   (3) the supported-platform list;

   (4) the decennial census release identifier referenced in § 102(1)–(2);

   (5) the SHA-256 of the TIGER/Line shapefile dataset referenced in § 102(3) and § 102(10);

   (6) **LATE-PUBLICATION BACKSTOP**. If the Director fails to publish the parameter table by the deadline in this subsection, the prior cycle's parameter table applies, with a single technical refresh authorized for items (2) and (4)–(5) to reflect the new census cycle. The State publication clock under § 105(a) is tolled for any period during which a current parameter table is not in force;

   (7) **TIGER ADJACENCY ERRATA**. The Director of the Census, on petition by an affected State, may publish numbered corrections to the TRACT ADJACENCY GRAPH defined in § 102(3) and § 102(10) to address documented TIGER/Line errors (waterfront-tract gaps, sliver-polygon artifacts, point-contact ambiguities). Errata shall be published not later than 60 days from petition; the REFERENCE IMPLEMENTATION shall consume them in subsequent runs.

(d) **REFERENCE-IMPLEMENTATION DEFECT PROCESS**. If a defect in the designated REFERENCE IMPLEMENTATION is identified after publication, the Director of the Census shall publish a numbered patch (analogous to FIPS errata) within 60 days of confirmation. A State that runs the published REFERENCE IMPLEMENTATION at the version designated under subsection (c)(2) at the time of execution has a complete safe-harbor defense to any § 107(e) action arising from output differences attributable to the defect; the State's remedy is to re-run the patched binary in the next published-version cycle.

(e) **PRIVATE RIGHT OF ACTION**.

   (1) **STANDING**. The following persons may bring a civil action to enforce this Act —

   (A) any candidate for the House of Representatives in the State;

   (B) any citizen of the State, but only on a showing of concrete injury beyond bare statutory violation, consistent with *TransUnion LLC v. Ramirez*, 594 U.S. 413 (2021); informational injury under § 105(c) qualifies as concrete injury for purposes of subparagraph (e)(1)(D)(i) below;

   (C) any organization whose members include persons described in subparagraph (A) or (B), provided that the organization satisfies the three-part test of *Hunt v. Washington State Apple Advertising Comm'n*, 432 U.S. 333 (1977): (i) at least one member would have standing in their own right; (ii) the interests asserted are germane to the organization's purpose; and (iii) neither the claim nor the relief requires participation of individual members;

   (D) **CAUSES OF ACTION**. A plaintiff with standing under (A)–(C) may bring an action —

   (i) to compel publication of the REPRODUCIBILITY ARTIFACTS under § 105;

   (ii) to challenge byte-identity failures under § 105(d);

   (iii) to challenge a VRA § 2 deviation justification under § 106(d) (*Shaw*-type challenges only) or to enforce the State's affirmative obligation under § 106(b)(1) (failure-to-modify claims);

   (iv) to challenge any input violation under § 104(e); or

   (v) to challenge a § 110 state-constitutional-integrity deviation.

(2) **PRE-SUIT CURE**. For actions under subparagraphs (D)(i), (D)(ii), and (D)(iv) — but **not** (D)(iii) (VRA-related) or (D)(v) (state-constitutional-integrity) — the plaintiff shall serve the State Attorney General with written notice of the alleged defect 30 days before filing. The State may cure within 30 days; cured defects extinguish the cause of action. This subparagraph does not apply to actions seeking interim-plan relief under § 108(c).

(f) **VENUE; THREE-JUDGE COURT**. A civil action under subsection (e) shall be filed in the United States District Court for the district in which the State capital is located. Such actions raise questions of constitutional apportionment within the meaning of 28 U.S.C. § 2284(a) and shall be heard by a three-judge district court, with direct appeal to the Supreme Court under 28 U.S.C. § 1253.

(g) **ASYMMETRIC FEE-SHIFTING**.

   (1) A prevailing plaintiff in an action under subsection (e)(1)(D)(i), (ii), (iii) (failure-to-modify claims under § 106(b)(1)), (iv), or (v) may recover reasonable attorneys' fees, in the manner provided by 42 U.S.C. § 1988.

   (2) A plaintiff in an action under subsection (e)(1)(D)(iii) challenging a § 106 deviation as exceeding what § 2 requires (a *Shaw*-type challenge) bears their own costs and attorneys' fees, absent a finding of bad faith on the part of the State. This asymmetry is intended to discourage *Shaw*-troll litigation against minority-protective deviations.

(h) **CRIMINAL PENALTY FOR INPUT VIOLATIONS**. Any State officer or employee who knowingly causes data prohibited by § 104(e) (voter-registration data, election results, partisan-affiliation data, racial-composition data, or any other non-enumerated data) to be used as input to the algorithm under § 104 shall be guilty of a misdemeanor and subject to a fine of not more than $10,000, imprisonment of not more than one year, or both. Mens rea is "knowingly"; negligent input contamination is not criminal but remains civilly actionable under subsection (e)(1)(D)(iv).

## § 108. Transitional provisions

(a) **EFFECTIVE DATE**. This Act takes effect upon enactment. The substantive obligations under §§ 103–106 and § 110 first apply to district plans drawn following the next decennial census after enactment.

(b) **NO RETROACTIVE INVALIDATION**. District plans in effect for the House election cycle ending immediately before the next decennial census after enactment shall remain valid for that cycle without regard to this Act.

(c) **COURT-ORDERED INTERIM PLANS**. Where a State fails to publish REPRODUCIBILITY ARTIFACTS conforming to this Act by the deadline in § 105(a), and a court orders an interim plan under existing equitable authority, the court shall order the ALGORITHM OUTPUT (the result described in § 105(c)(2)–(3)), modified only as necessary to comply with § 106 and § 110, as the interim plan.

(d) **STANDING TO INVOKE INTERIM-PLAN RELIEF**. The Attorney General of the United States, the State Attorney General, any candidate for the House of Representatives in the State, or any qualified elector of the State may petition for interim-plan relief under subsection (c) upon a showing that the § 105(a) deadline has passed without publication of conforming artifacts. Standing is conferred by this subsection without further showing of injury.

(e) **CENSUS-RELEASE TIMING RELIEF**.

   (1) The publication clock under § 105(a) is tolled for any period during which a current parameter table is not in force under § 107(c)(6).

   (2) If the deadline under § 105(a)(1) or (2) would fall within 90 days of the State's primary candidate-filing deadline for the House election following the next decennial census, the deadline is automatically extended to the later of (A) the original § 105(a) deadline or (B) the date 90 days before that filing deadline.

## § 109. Severability

If any provision of this Act, or the application of any provision to any person or circumstance, is held invalid, the remainder of this Act and the application of its provisions to other persons or circumstances shall not be affected, except as provided in this section.

(a) **§ 104(e) NON-SEVERABLE FROM § 104.** The PROHIBITED INPUTS rule in § 104(e) is integral to the algorithm specification in § 104 and is not severable. If § 104(e) is held invalid, § 104(a)–(d) is also invalid. (This is intentional: the bill's constitutional theory depends on partisan-blindness; an algorithm specification without partisan-blindness is not the bill Congress passed.)

(b) **VRA § 2 SEVERABILITY**. If § 106 is held invalid in any application, the State remains obligated to comply with section 2 of the Voting Rights Act independent of this Act, and the ALGORITHM OUTPUT applies as the baseline plan.

(c) **PRIVATE RIGHT OF ACTION SEVERABILITY**. If § 107(e) is held invalid, the United States, acting through the Attorney General, retains authority to enforce this Act.

(d) **DUAL CONSTITUTIONAL AUTHORITY**. If any provision of this Act is held to exceed Congress's authority under Article I, section 4 of the Constitution, that provision is reenacted as of the date of original enactment under section 5 of the Fourteenth Amendment to the same effect, and shall be construed to apply only to the extent necessary to enforce the substantial equality of voting power among congressional districts under *Reynolds v. Sims*, 377 U.S. 533 (1964), and successor authorities.

## § 110. State-constitutional integrity rules

(a) **ACKNOWLEDGMENT**. Many State constitutions require, for congressional districting, the preservation of political-subdivision boundaries (counties, towns, cities) "to the extent consistent with population equality" or analogous formulations. Congress acknowledges these requirements as legitimate State interests.

(b) **BOUNDED INTEGRITY DEVIATION**. Where the ALGORITHM OUTPUT splits a political subdivision in a manner that violates a State-constitutional integrity rule applicable to congressional districts, the State may modify specific TRACT assignments to preserve the subdivision boundary, subject to the limits in subsection (c).

(c) **CAP**. Modifications under subsection (b) shall not, in the aggregate, change the assignment of more than 1.5 percent of the State's TRACTS from their ALGORITHM OUTPUT assignment. (This cap is calibrated to permit small-county-line preservation without enabling swing-district reconstruction.)

(d) **JUSTIFICATION**. For each modification under subsection (b), the State shall publish, as part of the REPRODUCIBILITY ARTIFACTS under § 105(b)(5), a written justification containing —

   (1) the State-constitutional provision invoked, with citation;

   (2) the specific subdivision(s) preserved;

   (3) the specific TRACTS reassigned, before-and-after;

   (4) the cumulative tract count modified under this section as a percentage of the State's total tract count;

   (5) a statement that the modification does not exceed the minimum necessary to comply with the State-constitutional rule.

(e) **NO PARTISAN OR RACIAL JUSTIFICATION**. Modifications under this section may be justified only by a State-constitutional rule expressly preserving political-subdivision integrity. Modifications justified on partisan, racial, or community-of-interest grounds are unauthorized and unlawful under § 104(e).

(f) **SUPREMACY-CLAUSE RECOGNITION**. To the extent a State-constitutional integrity rule requires modifications exceeding the subsection (c) cap, this Act preempts the State-constitutional rule under the Supremacy Clause as applied to congressional districting only.

---

## Drafting notes (not part of the bill)

These notes flag intended meaning and anticipate likely interpretive disputes; they would not appear in enacted law.

### Anti-commandeering posture (Murphy v. NCAA)

The most serious facial-challenge vector for this bill is the Tenth Amendment. § 101(g) directly engages the doctrine, citing *Foster v. Love* (1997) and *Cook v. Gralike* (2001) as the modern Court's affirmation that Article I § 4 is an independent grant of authority not constrained by general anti-commandeering principles. The defense is: this is not commandeering of a sovereign function, because the act being commandeered is the *Manner of holding congressional elections*, which Article I § 4 affirmatively assigns to Congress's regulation. *Murphy v. NCAA* (2018) and *Printz v. United States* (1997) involved general legislative powers; the Elections Clause is a specific grant.

A more conservative architectural alternative — restructuring as conditional spending under *South Dakota v. Dole* (1987), so federal funding is the inducement — is described in `STATUTE_REVIEW_NOTES.md` Option B. We have not adopted that posture in v0.1 because (a) the *Foster*/*Cook*/*Smiley* line of authority is robust, and (b) the political optics of "compliance is conditional on federal funds" reads as soft, not as a serious mandate.

### Non-delegation (West Virginia v. EPA)

The v0 draft delegated the tolerance schedule and edge-weight specification to the Director of Census. v0.1 moves both into statutory text (§ 104(b)(4) and § 104(b)(5)). The Director now sets only the seed and the binary SHA-256. This is a narrow ministerial delegation, not a major-questions delegation, and survives *West Virginia v. EPA* (2022) and *Loper Bright* (2024) review.

### *Allen v. Milligan* tension

We acknowledge the tension between (a) this Act's race-blind algorithmic baseline and (b) *Milligan*'s rejection of Alabama's race-neutral-baseline argument. *Milligan* held that race-neutral simulations are not the controlling test for § 2; § 2 is about disparate effect on minority opportunity, not about whether a race-neutral process produced the map. The model bill is consistent with *Milligan* because § 106 expressly preserves § 2 with full force, including the duty to deviate from the race-blind baseline whenever § 2 requires it. The Act does not impose race-blindness as a ceiling; it imposes algorithmic determinism as a floor and lets § 2 push the floor where § 2 demands.

This reading depends on § 106(b) being mandatory ("shall"), not permissive ("may"). v0 was permissive; v0.1 is mandatory. This change resolves the *Milligan* tension and addresses the core concern raised in `STATUTE_REVIEW_NOTES.md` § 1.

### Race-predominance (Miller v. Johnson) exposure

§ 106(c) requires the State to publish a written justification for each § 106 modification. Critics argue this "manufactures" the evidentiary record that *Miller v. Johnson* (1995) and *Cooper v. Harris* (2017) treat as proof race "predominated." v0.1 mitigates by:

- Restructuring § 106(c) around the *Allen v. Milligan* narrow-tailoring framework (subsection (c)(5)–(6)), not a bare *Gingles* recital;
- Limiting the § 106(d) presumption to *Shaw*-type challenges only;
- Using asymmetric fee-shifting (§ 107(g)) to discourage *Shaw*-troll litigation against compliant States.

The residual exposure remains real and is documented in `STATUTE_REVIEW_NOTES.md` § 1. A more aggressive fix would be to make the justification an in-camera filing rather than a public publication, but this would compromise the bill's transparency theory.

### *Department of Commerce v. Montana* citation

We have removed the v0 reliance on *Montana* (1992) for the proposition that Congress can compel state line-drawing methods. *Montana* held that the equal-proportions method was constitutionally tolerable against a State's substantive challenge; it did not address Congress's affirmative authority to compel state methods. The cleaner authority is the *Smiley*/*Foster*/*Cook* line, plus *Branch v. Smith* (2003) for "manner includes line-drawing."

### Severability of § 104(e)

§ 109(a) makes § 104(e) non-severable from § 104. The reasoning: if § 104(e) (prohibited inputs) is struck down but § 104(a)–(d) (the algorithm) survives, the bill mandates a federally-prescribed algorithm that States can lawfully feed partisan registration data into — i.e., it mandates partisan-optimized maps. Severability cannot save the bill's constitutional theory. The honest answer is that § 104(e) and § 104(a) stand or fall together.

### Standing under TransUnion

§ 107(e)(1)(B) restricts citizen standing to "concrete injury beyond bare statutory violation," tracking *TransUnion v. Ramirez* (2021) directly. Informational injury under § 105(c) is recognized as concrete for verification-related claims (subparagraph (D)(i)) — but not for substantive claims (D)(ii)–(v), where the plaintiff must show downstream harm (e.g., vote dilution, candidate burden). This addresses the standing-defense vector flagged in the hostile-challenger review.

### Three-judge court (28 U.S.C. § 2284)

§ 107(f) expressly invokes the three-judge district court mechanism. This is consistent with the historic three-judge-court treatment of congressional-redistricting challenges (see *Branch v. Smith*, 538 U.S. 254 (2003)). Direct appeal to SCOTUS shortens the litigation timeline and produces uniform national rulings, which is the right posture for a federal-mandate statute.

### State-constitutional integrity (§ 110)

§ 110 is new in v0.1. It addresses the state-director's concern that state-constitutional integrity rules (town/county integrity) collide with § 104(d)'s "no discretion." The bounded-integrity-deviation approach (1.5% cap) parallels the § 106 deviation structure. The 1.5% cap is calibrated as: enough to preserve small counties (typical: <0.5% of state tracts) but not enough to reconstruct swing districts (which would require ~5% per the staff-drafter analysis). § 110(c) is the bright-line rule that resolves the state-constitutional collision without opening the door to integrity-pretext gerrymandering.

### Open drafting questions (preserved from v0)

1. **State legislative districts.** This bill addresses only congressional districts. Extending to state legislative districts requires either separate state legislation or a 14th-Amendment-grounded federal bill; out of scope here.

2. **Block-group resolution.** § 102(1) defines TRACT. For high-district-count states, block-group resolution may be necessary to achieve 0.5% population deviation; § 102 should be expanded in a future amendment to permit block-group inputs at the state's option.

3. **Senate / DC / territories.** Out of scope — congressional districting only.

---

## See also

- `docs/legal/STATUTE_RATIONALE.md` — policy memo
- `docs/legal/STATUTE_ONE_PAGER.md` — 90-second version
- `docs/legal/STATUTE_REVIEW_NOTES.md` — open tensions; Option B (conditional-spending) architectural alternative
- `docs/legal/FAIRNESS_DOCTRINE.md` — state-court companion strategy
- `docs/legal/CALLAIS_REFERENCE.md` — race-conscious-redistricting grounding
- `docs/file-formats/manifests.md` — schema underlying § 105(b)(3)
- `docs/REDIST_CLI.md` — the user-facing surface that satisfies § 107(a)
