# Incumbent Pairing Under Algorithmic Redistricting

**Series**: I.1
**Status**: Accepted 3.5/4
**Target**: American Political Science Review

## Algorithm / Subject

Analysis of incumbent pairing — the placement of two sitting incumbents in the same new congressional district — as an outcome of redistricting. Incumbent pairing forces one incumbent to retire or seek a different seat, reducing the incumbency advantage for at least one party in the affected district. Enacted maps routinely minimize pairing to protect all incumbents; algorithmic maps are indifferent to incumbent locations. The paper quantifies the fraction of incumbent pairs placed in the same district under \textsc{bisect} vs. enacted 2022 congressional maps across NC, WI, TX, and FL, and compares both to the analytical expectation under a random redistricting process given each state's geographic structure.

## Key Claims

1. \textsc{Bisect} algorithmic maps produce incumbent pairing rates statistically indistinguishable from the analytical expectation under random redistricting for all four states (NC, WI, TX, FL): the fraction of incumbents paired does not differ significantly from what geographic chance would produce given the tract-level assignment structure of the algorithm and the geographic distribution of incumbent residences.
2. Enacted 2022 congressional maps produce significantly lower pairing rates than the \textsc{bisect} baseline in all four states, consistent with explicit use of incumbent-address data to draw boundaries that place incumbents in separate districts — a practice that requires access to partisan and demographic data that the \textsc{bisect} algorithm does not receive.
3. Pairing rate differences between bisect and enacted maps are not attributable to geographic constraints (which affect both maps equally) but to deliberate mapmaker choices about where to draw district boundaries relative to incumbent addresses, establishing that enacted pairing protection is a redistricting choice, not a geographic necessity.

## Layer

Empirical

## Empirical Targets

- States: NC ($k=14$), WI ($k=8$), TX ($k=38$), FL ($k=28$)
- Incumbent identification: 2022 congressional incumbents by district, census tract of home address
- Metric: Fraction of incumbent pairs $(i, j)$ assigned to the same \textsc{bisect} district / same enacted district
- Baseline: analytical expectation $E[\text{pairs}]$ under uniformly random tract-to-district assignment with fixed district count $k$
- Compare: bisect pairing rate vs. enacted pairing rate vs. random baseline

## Test Invariants

- L0: Pairing rate $\in [0, 1]$; sum of paired incumbents $\leq k$ (at most $k/2$ pairings for $k$ districts)
- L1: On a synthetic state where all incumbents live in non-adjacent tracts, pairing rate = 0 under both bisect and enacted
- L2: NC bisect pairing rate $\in$ 95\% confidence interval of analytical random expectation $E[\text{pairs}] = k(k-1)/(n(n-1))$ adjusted for geographic adjacency

## Legal / Practitioner Value

Incumbent pairing is contested in every redistricting cycle: legislators cite the need to avoid pairing as a reason to depart from other redistricting criteria. Courts have recognized incumbent protection as a permissible factor (see I.4 for legal analysis) but have not required it. Expert witnesses in gerrymandering cases can use the pairing rate comparison to demonstrate that enacted maps provide incumbent protection that (a) is not a geographic necessity and (b) requires the mapmaker to use incumbency data that the algorithm doesn't access. The demonstration that bisect matches the random baseline — not the enacted map's lower pairing rate — establishes that enacted pairing protection is a deliberate redistricting choice with partisan implications (incumbents are predominantly from one party in asymmetrically gerrymandered states).

## Section Structure

§1 Introduction, §2 Mathematical Framework: Pairing Rate and Analytical Baseline, §3 Behavior Under Algorithmic Redistricting, §4 Empirical Results, §5 Legal Landscape (Incumbency as Permissible Factor), §6 Conclusion
