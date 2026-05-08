# Partisan Fairness Metrics: Overview and Framework
**Series**: L.0
**Status**: Accepted 3.5/4
**Target**: Political Analysis

## Algorithm / Subject
Synthesis paper integrating L.1–L.6 into a unified framework for measuring and evaluating partisan fairness in congressional redistricting. Covers the post-Rucho legal landscape (federal courts cannot remedy partisan gerrymandering under Rucho v. Common Cause, 588 U.S. 684 (2019), but state courts retain authority under state constitutional provisions), the conceptual distinction between descriptive metrics (what is the partisan outcome of a given map?) and normative standards (what outcome should a fair map produce?), and a decision table for expert witnesses selecting among the six metrics. Introduces the bisect algorithmic maps as a natural reference distribution anchored near the partisan-neutral centroid across all six metrics.

## Key Claims
1. Bisect algorithmic maps cluster near the partisan-neutral centroid across all six metrics (EG, MM, Partisan Bias, Declination, Responsiveness, Proportionality), providing a natural reference distribution that no single enacted map can dismiss as a partisan artifact.
2. Post-Rucho, state constitutional language (PA, NC, OH) now drives most active partisan gerrymandering litigation; the six-metric framework translates into evidence usable in state court under "free and equal elections" clauses.
3. Descriptive and normative metrics behave differently under algorithmic redistricting: descriptive metrics (EG, MM, Bias, Declination) converge toward neutrality as a byproduct of compactness optimization, while normative standards (proportionality, responsiveness) require independent specification of a fairness target.

## Layer
Legal

## Empirical Targets
- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), plus FL ($k=28$) for partisan analysis
- Data: VEST 2020 presidential precinct returns mapped to 2020 census tracts
- Compare: bisect algorithmic maps vs. enacted 2022 congressional maps

## Legal / Practitioner Value
Post-Rucho, federal courts closed the door on federal partisan gerrymandering claims (Rucho v. Common Cause, 2019). State courts remain open: Pennsylvania (League of Women Voters v. Commonwealth, 2018), North Carolina (Harper v. Hall, 2022), and Ohio (League of Women Voters v. Ohio, 2022) have all imposed state constitutional limits. Expert witnesses in these proceedings must select among six partially overlapping metrics; the decision table in §4 maps legal theory (proportionality vs. competitiveness vs. symmetry) to the appropriate metric(s), reducing the risk of metric-shopping criticism. The bisect reference distribution provides a non-partisan baseline that courts can use to anchor "how different is this map from what a neutral algorithm would produce?"

## Section Structure
§1 Introduction and Post-Rucho Landscape, §2 Taxonomy of Partisan Fairness Metrics (descriptive vs. normative), §3 The Six Metrics: Definitions, Properties, and Relationships, §4 Decision Table for Expert Witnesses, §5 Bisect Algorithmic Maps as a Reference Distribution, §6 Empirical Overview Across NC/WI/TX/FL, §7 Conclusion
