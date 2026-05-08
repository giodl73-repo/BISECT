# E-Track Panel Review — Batch 1
**Date**: 2026-05-07
**Reviewer Panel**: R1 Karypis (algorithms), R2 Rodden (political science), R3 Duchin (math/redistricting), R4 Stephanopoulos (law), R5 Liang (ML/reproducibility)
**Scale**: 0–4 per reviewer; Average ≥ 3.0 = Accept, ≥ 2.5 = Minor Revision, ≥ 2.0 = Major Revision, < 2.0 = Reject

---

## E.0 — Alternative Representation Systems: A Design Space Analysis

### Reviewer Scores

| Reviewer | Score | Primary Concern |
|----------|-------|-----------------|
| R1 Karypis | 3.0 | Pareto scoring rubric uses ordinal 1–3 scale without uncertainty bounds; correlation claim (r=−0.82) reported on 6 observations and grossly underpowered |
| R2 Rodden | 3.5 | Accurately attributes geographic sorting as binding constraint; synthesises my own work fairly; headline claim "40% gap reduction for 3-member districts" is a forward reference with no data here |
| R3 Duchin | 3.0 | Pareto table scores treated as cardinal rather than ordinal; bootstrapped CI [0.011, 0.021] on exchange rate claim appears in E.0 without derivation; references E.0's own data as if previously validated |
| R4 Stephanopoulos | 3.5 | Legal citations accurate (*Rucho*, *Moore*, *Allen*, *Wesberry*, *Reynolds*, *Gingles*, Fair Representation Act); 2 U.S.C. §2c correctly cited; no invented cases |
| R5 Liang | 2.5 | All quantitative claims are forward references to companion papers; E.0 itself presents no empirical analysis but states exchange rates, compactness figures, and gap reductions as settled |

**Average: 3.1 → Accept (Minor Revision Recommended)**

### Priority Issues

**P1 — Blockers**
- None.

**P2 — Should Fix**
- P2-1: The Pareto table treats 1–3 ordinal scores as if continuous (reports r=−0.82 on 6 data points). Either drop the correlation claim or add a caveat that it is illustrative only.
- P2-2: Bootstrap CI [0.011, 0.021] on the compactness–proportionality exchange rate appears in the abstract and synthesis without the derivation being available in this paper. Add forward reference with explicit caveat that the estimate is from E.5.
- P2-3: The "40%" and "65%" gap reduction figures for 3-member and 5-member districts are stated as established facts; clarify these are reported from E.1 (paper not yet reviewed).

**P3 — Optional**
- P3-1: Abstract mentions "District-Independent Algorithm (DIA)" but CLI binary is called `bisect`. Internal audience knows this; external reviewers may flag the terminology drift.
- P3-2: Footnote on manipulation resistance scoring buried mid-table; recommend moving to prose.

---

## E.2 — Direct County Representation

### Reviewer Scores

| Reviewer | Score | Primary Concern |
|----------|-------|-----------------|
| R1 Karypis | 2.5 | No algorithmic novelty; Huntington-Hill apportionment is a standard method applied without modification. Methodology section describes the algorithm correctly but contributes nothing new algorithmically. Resolution adequacy issues (county-internal redistricting feasibility) not analysed. |
| R2 Rodden | 3.0 | VRA majority-minority count estimate (197–201 districts) uses proportional allocation, not actual algorithmic redistricting. Section 07-vra-analysis.tex explicitly acknowledges this but buries it in a justification paragraph. Headline claim "+129–133 MM districts" is stated in abstract without this caveat. |
| R3 Duchin | 2.5 | The VRA district count estimate is not from a rigorous algorithmic run; it is proportional approximation. Claiming "nearly threefold increase" as a headline finding when the number is an estimate—not a measured result—is methodologically misleading. No ensemble analysis; single analytical framework. |
| R4 Stephanopoulos | 3.0 | All major citations accurate (*Baker v. Carr*, *Reynolds v. Sims*, *Wesberry*, *Thornburg v. Gingles*). Constitutional analysis is sound: correctly identifies that strict county representation would require amendment under *Wesberry*. Note: "2.0× ideal threshold" requires constitutional amendment claim is correct. |
| R5 Liang | 2.0 | Results section covers only 11 of 25 qualifying counties for partisan analysis. Full county partisan data is acknowledged as missing. VRA district count (197–201) is an estimate, not reproduced output. Single analytical run with no seed variation. Section 4 explicitly notes "we lack complete partisan data for all counties." |

**Average: 2.6 → Minor Revision**

### Priority Issues

**P1 — Blockers**
- P1-1: Abstract states "197–201 majority-minority districts" as a finding, but this is a proportional estimate, not a measured algorithmic output. The abstract must qualify this claim (e.g., "we estimate" or "proportional analysis suggests"). The current framing implies a precision that the methodology does not support.

**P2 — Should Fix**
- P2-1: Table 3 (partisan lean) covers only 11 of 25 counties with complete data; extrapolation to "+15–25 Democratic seat advantage" is unreliable. Either complete the data collection or narrow the claim.
- P2-2: Section on "Comparison to Other Redistricting Reforms" critiques algorithmic redistricting as creating "arbitrary constructs" without any empirical comparison to the proposed county-based approach's actual performance on compactness or equality.
- P2-3: The paper does not demonstrate county-internal VRA redistricting is feasible; it only asserts it would be. This is the key missing step.

**P3 — Optional**
- P3-1: Binary referenced as `redist` in one footnote (Section 03-methodology.tex, VRA weighting parameters paragraph); should be `bisect`.
- P3-2: The argument that "bipartisan fragmentation" justifies the reform would be strengthened by showing that commission-drawn maps also fragment counties (acknowledged in discussion but not quantified).

---

## E.3 — National Redistricting Without State Boundaries

### Reviewer Scores

| Reviewer | Score | Primary Concern |
|----------|-------|-----------------|
| R1 Karypis | 3.5 | Methodology is algorithmically rigorous: METIS configured correctly, ablation over β ∈ {0, 0.5, 1.0, 2.0, 5.0, 10.0, 50.0, 100.0, 300.0} is thorough. The non-monotonic boundary-crossing result (peak at β=1.0) is properly flagged as unexpected and mechanistically explained. Population balance verified separately from compactness. |
| R2 Rodden | 3.5 | Central finding (state boundaries improve compactness) is counterintuitive and well-supported. Correct to note that proportionality does not improve under national redistricting even without state constraints. Comparability note in abstract/Section 4 about the different PP baseline (0.461 national experiment vs. 0.361 B.2 per-state mean) is critical and appropriately flagged. |
| R3 Duchin | 3.0 | Single seed (42) for all runs; no ensemble analysis. For a paper whose main claim rests on a single optimisation outcome (0.265 vs. 0.461), seed sensitivity is important. Five seeds would substantially strengthen confidence. Ablation is extensive but all runs use same seed. |
| R4 Stephanopoulos | 3.5 | Constitutional analysis is accurate: Art. I §2 quoted correctly; *Wesberry* cited appropriately; the framing as "constitutionally unavailable counterfactual" is legally precise. The fractional representation discussion correctly flags the amendment requirement. No invented cases. |
| R5 Liang | 3.0 | Fixed seed 42 throughout. Ablation over β values is commendable but not a substitute for multi-seed testing of the primary result. Code is described sufficiently for replication (METIS, ufactor=5, 100 iterations, fixed seed). GeoPandas library referenced. No variance reported on the 42.5% compactness figure. |

**Average: 3.3 → Accept**

### Priority Issues

**P1 — Blockers**
- None.

**P2 — Should Fix**
- P2-1: The headline result (42.5% less compact under national optimisation) uses a single seed. Add at minimum 3–5 seeds for the primary national run and report variance; a ±2% range would not change the conclusion but would make it defensible.
- P2-2: The comparability note on baseline PP (0.461 vs. B.2's 0.361) is in the abstract Paragraph Note but easy to miss. Move to a dedicated Methodology subsection.

**P3 — Optional**
- P3-1: The discussion of fractional representation could note that the 2020 Equal Census Act proposals used different mechanisms; this would contextualize the theoretical discussion.
- P3-2: The ablation table reports β values to 300 but the theoretical justification for choosing those specific values is not stated.

---

## E.5 — Party-Based Seat Allocation (Overlapping Party Districts)

### Reviewer Scores

| Reviewer | Score | Primary Concern |
|----------|-------|-----------------|
| R1 Karypis | 2.0 | The algorithm description is internally inconsistent. Section 02 states edge weights use "inverse distance between tract centroids" and α=0.7, β=0.3 objective weights, but this is inconsistent with the METIS min-cut framework used throughout the rest of the paper series. The algorithm pseudocode (Algorithm 1) cites karypis1998metis but the objective function specified does not match METIS's documented cost function. |
| R2 Rodden | 2.0 | Results cover only Ohio (15 seats) and California (52 seats), both cherry-picked as "test cases" without rationale. No 50-state results. The "100% RepCoverage" claim is guaranteed by construction, not an empirical finding. The 2020 presidential vote is used as the proportional allocation input for congressional seats—this conflates executive and legislative vote shares, a basic methodological error. |
| R3 Duchin | 1.5 | Two test cases only; no ensemble. The "perfect proportionality" claim is tautological—proportionality is built in by construction before redistricting begins, not achieved by the algorithm. The compactness analysis defers all 50-state results to "future work." The main algorithmic contribution (overlapping districts) has no legal precedent analysis. |
| R4 Stephanopoulos | 2.5 | The paper correctly identifies 2 U.S.C. §2c as the single-member mandate. The constitutional analysis is underspecified: the paper states overlapping districts are "constitutionally available" but does not cite *Thornburg v. Gingles* or any case addressing whether at-large multi-party districts satisfy VRA obligations. The efficiency gap citation (Stephanopoulos 2015) is correct. |
| R5 Liang | 1.5 | Results are for two states only. The party-weighted population balance table (Table 1) shows 0.06% SD—suspiciously perfect, with no explanation of how this is achieved given METIS's stochastic nature. Full 50-state implementation described as "future work." No reproducibility package, no seed reported. |

**Average: 1.9 → Reject**

### Priority Issues

**P1 — Blockers**
- P1-1: The objective function in the methodology (α=0.7 EdgeCut + β=0.3 PopulationImbalance with inverse-distance weights) contradicts the METIS framework cited. Either correct the algorithm description to match the actual implementation or correct the implementation description.
- P1-2: The 2020 presidential vote is used for proportional seat allocation in a paper about congressional redistricting. Presidential and congressional vote shares diverge substantially in many states. This is not merely a simplification—it produces systematically wrong seat allocations (California's 52 congressional seats are not comparable to Biden's 63.4% presidential share).
- P1-3: The paper presents only 2 of 50 states as results, labelled as "test cases," but the introduction and abstract claim a complete 50-state system is demonstrated. This overstates the empirical coverage by 25×.

**P2 — Should Fix**
- P2-1: The VRA implications of overlapping district maps are unaddressed. If every voter is in both a Democratic and a Republican district, VRA majority-minority analysis must account for which map governs. This is not a minor omission.
- P2-2: The paper has no limitations section beyond a brief paragraph at the end of the discussion. A system with as many novel claims as this one requires a substantive limitations analysis.

**P3 — Optional**
- P3-1: The ballot design discussion is speculative without reference to any existing multi-member or overlapping district precedent in U.S. or comparative elections law.

---

## E.6 — International Applications

### Reviewer Scores

| Reviewer | Score | Primary Concern |
|----------|-------|-----------------|
| R1 Karypis | 2.5 | Germany prototype uses 4,680 Gemeinden with 15.7 units/Wahlkreis, which violates the paper's own stated 20-unit threshold. 6 non-contiguous districts in Germany result (2%). The Ireland run uses 106 GADM county units for 43 constituencies = 2.5 units/constituency, far below any feasibility threshold. These are acknowledged but their effect on the claimed results is not bounded. |
| R2 Rodden | 2.0 | Abstract claims "10–22% compactness improvements over enacted boundaries in all six countries" but the Results section explicitly states that UK, Canada, and New Zealand results are "pending." The abstract's central quantitative claim is not supported by the results as written. This is the most serious issue in the paper. |
| R3 Duchin | 2.0 | The Gallagher index results for Ireland and Malta (claimed < 2.0) are described as "pending" in the Results section. The abstract's claim that geographic optimization "does not impair proportionality" with "Gallagher index < 2.0" is not yet established empirically. The STV → D'Hondt approximation is acknowledged but not validated. |
| R4 Stephanopoulos | 2.5 | The institutional descriptions (Boundary Commissions, Electoral Commission Ireland, Wahlkreiskommission) are accurate. The legal analysis of each country's balance standard is correct and documented. No invented citations. |
| R5 Liang | 1.5 | The methodology section references `redist` CLI v0.1.0 as the implementation (not `bisect`). Results for 3 of 6 countries are prototype-only. No Gallagher index computed. Enacted boundary comparisons "pending." Population data for Germany uses "uniform distribution" as a placeholder. This paper is not yet ready for review in its current empirical state. |

**Average: 2.1 → Major Revision**

### Priority Issues

**P1 — Blockers**
- P1-1: The abstract claims "10–22% compactness improvements over enacted boundaries in all six countries." The Results section delivers data for 3 countries (Ireland, Malta, Germany) in prototype form only, with enacted boundary comparisons explicitly listed as "pending" for all countries. The abstract must be revised to reflect what is actually reported.
- P1-2: The methodology references `redist` as the CLI tool (Section 03, "the `redist` platform version 0.1.0"); the binary has been renamed `bisect`. Update all references.

**P2 — Should Fix**
- P2-1: Ireland prototype uses 2.5 units/constituency, far below the 20-unit threshold the F.3 companion paper establishes. The compactness result (PP = 0.203, range 0.022–0.399) is primarily a function of the coarse data rather than the algorithm. Acknowledge that the Ireland result is a feasibility demonstration, not a compactness measurement.
- P2-2: The STV → D'Hondt proportionality approximation is used to claim Gallagher < 2.0 for Ireland and Malta, but the actual Gallagher computation is listed as "pending." The Discussion section's claim that "geographic optimization does not impair proportionality" is stated as a finding but is not yet supported by the current Results.
- P2-3: Germany: 6 non-contiguous districts (2%) are attributed to GADM artifacts. This matters for the compactness comparison; non-contiguous districts cannot be fairly compared on PP to enacted contiguous districts.

**P3 — Optional**
- P3-1: The RPLAN format description is detailed but the actual format specification is not published. The paper would benefit from a link to the format spec or an appendix defining the 11-character GEOID encoding.

---

## E.7 — Lessons Learned from the E-Track Experiments

### Reviewer Scores

| Reviewer | Score | Primary Concern |
|----------|-------|-----------------|
| R1 Karypis | 3.0 | Algorithmic framing of redistricting as a design space problem is sound. The "no free lunch" reference (Wolpert 1997) is appropriate. The 20% irreducibility bound derivation (footnote in Section 3) is appropriately hedged as estimated, with the scaling caveat noted. |
| R2 Rodden | 3.5 | This is the best paper in the track for engaging with the empirical political science literature accurately. The distinction between within-system exchange rate (0.019 from E.5) and cross-system rate (0.015 from E.0) is carefully explained and appropriate. The use of 2016–2022 average House results (rather than a single year) for proportionality computation is methodologically sound. |
| R3 Duchin | 3.0 | The paper targets Annual Review of Political Science and achieves that standard. The 5-lesson structure is clear. The Lesson 4 claim that "national redistricting gains 12% mean Polsby-Popper" contradicts Lesson 4's own text, which states "42.5% compactness penalty"—the lesson says crossing state lines *hurts* compactness, but the intro to Lesson 4 header says "gains 12%." This appears to be an editing error. |
| R4 Stephanopoulos | 3.5 | Excellent legal framing. The constitutional taxonomy (what requires statute vs. amendment) is accurate and well-grounded. *Moore v. Harper* (2023), *Allen v. Milligan* (2023), *Wesberry*, *Reynolds* all correctly cited and applied. |
| R5 Liang | 3.0 | Forward references to companion papers are managed appropriately; E.7 correctly attributes specific numerical results to E.1–E.5 rather than re-running them. The single-seed limitation is noted. The 5-seed sensitivity check mentioned for E.1 case studies is consistent with F.1 practices. |

**Average: 3.2 → Accept**

### Priority Issues

**P1 — Blockers**
- P1-1: Lesson 4 internal contradiction. Section 05-lesson4.tex opens: "national redistricting gains 12% mean Polsby-Popper but loses 40% of county preservation" — but E.3's actual finding is that national redistricting produces 42.5% *less* compact districts. The "gains 12%" language is inconsistent with the E.3 finding. (The synthesis section correctly states "42.5% compactness penalty.") Fix the Lesson 4 opening statement.

**P2 — Should Fix**
- P2-1: The 20% irreducibility bound (footnote in Section 3) cites Herschlag (2020) "NC ensemble lower bound, scaled by the WI/NC sorting ratio." This is an approximation that should be stated more explicitly as such, particularly since the lower bound is for a different state and a different metric.
- P2-2: The paper says E.1 finds "18% more effective minority representation" in 3-member districts but E.1 is not yet reviewed and this figure cannot be checked here.

**P3 — Optional**
- P3-1: The "coda" on algorithmic counterfactuals is well-written but could be a subsection of the conclusion rather than a separate section.

---

## Batch Summary

| Paper | R1 | R2 | R3 | R4 | R5 | Avg | Verdict | P1 Count | Top P1 Issue |
|-------|----|----|----|----|-----|-----|---------|----------|--------------|
| E.0 Overview | 3.0 | 3.5 | 3.0 | 3.5 | 2.5 | **3.1** | Accept | 0 | — |
| E.2 County Rep | 2.5 | 3.0 | 2.5 | 3.0 | 2.0 | **2.6** | Minor Revision | 1 | Abstract overstates VRA MM district count as a measured result |
| E.3 National | 3.5 | 3.5 | 3.0 | 3.5 | 3.0 | **3.3** | Accept | 0 | — |
| E.5 Party-Based | 2.0 | 2.0 | 1.5 | 2.5 | 1.5 | **1.9** | **Reject** | 3 | Algorithm description contradicts METIS framework; presidential vote misused for congressional seat allocation |
| E.6 International | 2.5 | 2.0 | 2.0 | 2.5 | 1.5 | **2.1** | Major Revision | 2 | Abstract claims results for 6 countries; paper delivers prototype data for 3 |
| E.7 Lessons | 3.0 | 3.5 | 3.0 | 3.5 | 3.0 | **3.2** | Accept | 1 | Lesson 4 states national redistricting "gains 12% PP" but E.3 finds 42.5% compactness penalty |

### Cross-Paper Notes
- E.5 is the only Reject in this batch. The paper's algorithmic claims are internally inconsistent and the empirical results cover 2 of 50 states. Recommend a complete rewrite with: (1) corrected algorithm description, (2) actual congressional vote shares (not presidential), (3) 50-state results, (4) VRA analysis.
- E.6 is pending completion of UK, Canada, and New Zealand runs. Major Revision is appropriate; the paper is worth completing but cannot be published in current form with undelivered abstract claims.
- E.0 and E.7 are the strongest papers in the track. Both are well-suited for their target venues (design-space analysis and Annual Review respectively).
