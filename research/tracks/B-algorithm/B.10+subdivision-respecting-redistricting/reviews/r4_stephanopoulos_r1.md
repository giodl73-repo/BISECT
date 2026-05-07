---
reviewer: Nicholas Stephanopoulos
round: 1
score: 3
date: 2026-05-05
---

## Summary

B.10 addresses the legally significant challenge of county preservation in redistricting: 34 state constitutions require it, and existing algorithmic approaches either ignore it or make the problem infeasible. The county-sticky weighting approach is a principled and practically useful solution. From a legal perspective, the paper's central contribution is the operationalisation of "preserve to the extent possible" as a calibrated soft constraint, with the calibration (alpha_c = 3.0) determined empirically rather than by legislative fiat. The legal framing is generally sound, though it requires more careful treatment of the hierarchy of redistricting criteria.

## Strengths

- **The "to the extent possible" operationalisation is legally defensible.** Section 5.2 frames county-sticky weighting as an algorithmic implementation of the constitutional phrase "preserve political subdivisions to the extent possible." The paper correctly notes that courts have upheld county preservation as valid when justified by population-equality requirements, and the alpha_c = 3.0 default achieves maximum county preservation consistent with population balance. This is the correct legal structure.
- **The parameter transparency argument.** The paper notes that alpha_c = 3.0 is "published in the TIGER adjacency graph metadata, making the weighting fully reproducible and auditable." This transparency argument is essential for the legal defensibility of the DIA approach: any party can verify that the county-sticky weights were applied correctly.
- **The Iowa case study shows the algorithm tracks legislative intent.** The reduction from 12 to 3 county splits in Iowa, closely following the traditional four-region geography, demonstrates that the algorithm produces maps that match the kind of subdivision-preserving outcomes that human mapmakers and state legislatures have historically aimed for.

## Weaknesses / P1 Items (Required Fixes)

- **The hierarchy of redistricting criteria is not correctly represented.** Section 1 lists the criterion hierarchy as: (1) equal population, (2) VRA compliance, (3) contiguity, (4) compactness, (5) subdivision preservation. But the paper applies county-sticky weighting at the same time as compactness weighting — meaning criteria (4) and (5) are treated simultaneously rather than hierarchically. Under strict hierarchy, the algorithm should first satisfy population balance and VRA, then maximise compactness, then maximise county preservation subject to no compactness loss. The county-sticky approach blends criteria (4) and (5) via the alpha_c tradeoff. The paper must acknowledge that the simultaneous weighting deviates from a strict hierarchical application of the criteria, and argue why the simultaneous approach is legal under states that specify a strict priority ordering.
- **The "34-state constitutional requirement" claim needs a citation and enumeration.** The paper states "county preservation is a constitutionally required criterion in 34 state redistricting frameworks" and cites NCSL 2021. But it does not list the 34 states or distinguish between constitutional provisions (mandatory) and statutory provisions (directory). Courts have treated these differently: mandatory provisions require the criterion to be respected; directory provisions allow departure when other criteria demand it. The paper should provide the list of 34 states and note which have mandatory vs. directory provisions, as this affects whether alpha_c = 3.0 is the appropriate default for all 34.
- **The population deviation increase (0.03 pp) is not tested for significance.** Table 2 reports that the maximum population deviation increases from 0.41% to 0.44% at alpha_c = 3.0. The paper says this "increase is negligible and all maps satisfy the 0.5% constitutional requirement." But 0.44% is already 88% of the way to the 0.5% constitutional limit. Is the 0.03 pp increase consistent across all 50 states, or does it represent an average that hides individual states approaching the 0.5% limit? Report the maximum deviation for individual states at alpha_c = 3.0 to verify that no state exceeds the constitutional threshold.

## P2 Items (Suggestions)

- **Add a table listing the 34 states with county preservation requirements.** This would make the paper directly usable by state legislative staff and redistricting commission members. The table should note whether the requirement is constitutional or statutory, and whether it applies to congressional, state legislative, or both types of maps.
- **Discuss the interaction between county preservation and VRA compliance.** States with significant minority population concentrations in specific counties may face tension between county preservation and majority-minority district creation. A paragraph acknowledging this interaction and noting that the DIA's VRA compliance layer (B.14) takes precedence over county preservation would complete the legal picture.

## Score: 3 — Minor Revision

The criterion hierarchy issue (P1.1) is the most substantive concern — the paper needs to explain why simultaneous compactness + county weighting is legally appropriate when constitutions specify a hierarchical priority. The 34-state enumeration (P1.2) and population deviation verification (P1.3) are documentation and data gaps.
