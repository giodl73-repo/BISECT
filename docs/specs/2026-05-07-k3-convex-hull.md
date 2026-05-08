# Convex Hull Ratio in Algorithmic Redistricting

**Series**: K.3
**Status**: Accepted 3.5/4
**Target**: Election Law Journal

## Algorithm / Subject

Empirical study of the Convex Hull Ratio (CH = Area / Area(convex hull)) as a compactness metric in redistricting. Covers the geometric definition and computation via Graham scan or Andrew's monotone chain, the ability of CH to detect "tentacle" or "octopus arm" districts that PP and Reock miss (because PP/Reock are insensitive to non-convex appendages that do not increase perimeter relative to area substantially), and an analysis of CH values across bisect structure algorithms for NC, WI, and TX under 2020 census data. Includes a visual explanation of the convex hull comparison appropriate for court presentation.

## Key Claims

1. Convex Hull Ratio detects "tentacle" districts that PP and Reock fail to flag: synthetic tentacle districts with PP $\approx 0.15$ and Reock $\approx 0.35$ score CH $< 0.60$, demonstrating CH's sensitivity to non-convex appendages.
2. All four bisect structure algorithms (standard-bisect, prime-factor, ratio-optimal, moving-knife) produce CH $> 0.85$ on all NC 2020 congressional districts, confirming that bisect's recursive partitioning does not generate tentacle geometries by construction.
3. CH is the most legally intuitive compactness metric for courtroom presentation: judges and juries can visually verify the convex hull overlay without mathematical training, making CH effective for per se visual-irregularity tests under Shaw v. Reno.

## Layer

Standalone

## Empirical Targets

- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), 2020 census
- Compare across: standard-bisect, prime-factor, ratio-optimal, moving-knife structure algorithms
- Metrics: district-level CH distribution (min, median, mean, max), state-level mean CH, CH–PP divergence cases (low PP but high CH, or low CH but moderate PP), synthetic tentacle detection test

## Test Invariants

- L0: CH of a convex polygon is exactly 1.0; CH is always in $(0, 1]$; CH of any polygon is $\geq$ PP (a convex hull is at least as large as the bounding circle only if circular, but CH $\geq$ PP for elongated shapes — verify on square and rectangle); CH is monotone under convex hull enlargement
- L1: on a cross-shaped synthetic district (a $+$ shape with four arms), CH $< 0.70$ and PP $> 0.10$ (non-trivial PP but low CH — the case tentacle detection targets); convex hull area $\geq$ district area for all test geometries
- L2: all NC 2020 bisect districts have CH $> 0.85$; the NC enacted 2020 map has at least one district with CH $< 0.85$ (tentacle detection in the comparison map)

## Legal / Practitioner Value

Convex hull ratio has been implicitly referenced in redistricting case law since Shaw v. Reno (1993), where the Court described districts that are "bizarrely shaped" or have "tentacle-like appendages." The convex hull provides a rigorous operationalisation of that legal test: if a district's area is substantially less than its convex hull, it has appendages or concavities that cannot be explained by population distribution alone. Expert witnesses can show the convex hull overlay on a map, making CH one of the most effective metrics for per se visual-irregularity arguments. Ohio's redistricting criteria and Pennsylvania's League of Women Voters standard both implicitly use a convex hull test. This paper gives CH values for bisect plans and documents that bisect's recursive structure avoids tentacle geometries by design.

## Section Structure

§1 Introduction, §2 Mathematical Definition (Convex Hull and CH Formula), §3 Computational Implementation (Graham Scan), §4 Tentacle Detection: Cases Where CH Diverges from PP and Reock, §5 Empirical Comparison Across Structure Algorithms, §6 Visual Explanation for Court Presentation, §7 Legal Usage and Case Survey, §8 Conclusion
