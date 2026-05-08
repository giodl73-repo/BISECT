# Seats-Votes Curve and Responsiveness in Congressional Redistricting
**Series**: L.5
**Status**: Accepted 3.5/4
**Target**: Political Analysis

## Algorithm / Subject
Analysis of the seats-votes curve $S(v)$ and its summary statistics — responsiveness and partisan bias — as tools for evaluating congressional redistricting plans. The seats-votes curve maps statewide Democratic two-party vote share $v \in [0,1]$ to the expected Democratic seat share $S(v)$, estimated via uniform swing from observed district vote shares. Responsiveness is formally defined as the slope $R = dS/dv\big|_{v=0.50}$; a map with $R = 2.0$ is "competitively ideal" (a 1-point vote-share gain translates into a 2-point seat-share gain near the median), while $R < 1.5$ indicates suppressed competition (many safe seats insulate the seat outcome from vote swings). The seats-votes curve integrates EG (the area between the curve and the symmetric ideal), partisan bias ($S(0.50) - 0.50$), and responsiveness ($dS/dv$ at $v=0.50$) into a single picture of a map's partisan properties, enabling a unified visual display for expert testimony.

## Key Claims
1. Bisect algorithmic maps have responsiveness $R \approx 2.0$ near $v = 0.50$ on NC ($k=14$) and WI ($k=8$) — close to the competitive ideal — because compactness-driven bisection produces a roughly uniform distribution of district vote shares without systematic concentration in safe-seat territory.
2. The enacted NC 2022 congressional map has responsiveness $R \approx 1.3$ at $v = 0.50$, reflecting the high proportion of safe Republican seats created by packing Democratic voters; this suppressed responsiveness means Republican incumbents are insulated from vote swings that would produce seat changes in a competitively drawn map.
3. The seats-votes curve integrates EG, partisan bias, and responsiveness into a single analytically coherent framework: EG equals the signed area between the estimated $S(v)$ curve and the $S(v) = v$ diagonal (integrated over $v \in [0,1]$ under the uniform distribution), and partisan bias equals $S(0.50) - 0.50$, so the full curve is the sufficient statistic from which both scalar metrics can be recovered — making it the preferred exhibit when courts want a single comprehensive display.

## Layer
Legal

## Empirical Targets
- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), plus FL ($k=28$) for partisan analysis
- Data: VEST 2020 presidential precinct returns mapped to 2020 census tracts
- Compare: bisect algorithmic maps vs. enacted 2022 congressional maps

## Legal / Practitioner Value
The seats-votes curve is the most complete single-exhibit summary of a map's partisan properties for courts. Judicial audiences can read it as a graph: the horizontal axis is statewide vote share, the vertical axis is seat share, and a fair map has the curve passing through (0.50, 0.50) with a slope near 2.0. Enacted NC's curve passes well below (0.50, 0.50) and has a shallow slope, visually communicating both bias and suppressed competition without requiring the court to understand wasted-vote algebra. The comparison to bisect's curve — which passes through (0.50, 0.50) and has slope $\approx 2.0$ — provides a neutral reference. In state court proceedings post-Rucho (NC Harper v. Hall, OH League of Women Voters), the seats-votes curve has been admitted as expert evidence; this paper provides the methodological foundation and documents the uniform swing limitation (the curve is a first-order approximation that underestimates seat changes in wave elections).

## Section Structure
§1 Introduction, §2 Mathematical Definition and Properties, §3 Behavior Under Algorithmic Redistricting, §4 Empirical Results, §5 Legal Landscape, §6 Conclusion
