# Section 203 Language Minority Requirements and Algorithmic Redistricting

**Series**: D.7
**Status**: Accepted 3.5/4
**Target**: Michigan Law Review

## Algorithm / Subject

Legal and empirical analysis of Section 203 of the Voting Rights Act, which requires jurisdictions with large language-minority populations to provide translated election materials (ballots, voter registration forms, polling place notices) in the relevant minority language. The paper examines which jurisdictions are covered by Section 203 (determined by the Census Bureau's five-year American Community Survey data on limited-English-proficient citizens), the coverage determinations for 2020–2032, and the interaction with algorithmic redistricting: whether district boundaries drawn by \textsc{bisect} affect which jurisdictions cross the Section 203 coverage thresholds, and how redistricting can be designed to ensure that communities with Section 203 coverage rights are not split across multiple districts in ways that impede their access to language assistance.

## Key Claims

1. Section 203 coverage determinations are made at the county or county-equivalent level based on whether the jurisdiction has: (a) $\geq 10{,}000$ voting-age citizens in the language minority group, or (b) $\geq 5\%$ of voting-age citizens in the language minority group, \textit{and} the literacy rate in English among the group is below the national average — coverage triggers mandatory language assistance that is a federal legal obligation independent of redistricting outcomes.
2. \textsc{Bisect} district boundaries that split Section 203-covered counties create implementation challenges: county election offices must provide language assistance county-wide, but federal and state election administrators must coordinate language assistance across district boundaries when a county is split between multiple congressional districts — the paper documents how \textsc{bisect}'s adjacency-based bisection typically preserves county boundaries more than enacted maps in states with significant county-splitting.
3. VRA Section 203 coverage and VRA Section 2 minority district requirements (analyzed in D.0–D.5) interact: communities with both Section 2 protection (sufficient Gingles prongs to require a majority-minority district) and Section 203 coverage (requiring translated materials) receive compounded protections that algorithmic redistricting must preserve in its VRA mode — the paper documents how the \textsc{bisect} VRA mode handles both requirements simultaneously.

## Layer

Legal

## Empirical Targets

- Section 203 coverage jurisdictions: 2023 Census Bureau coverage determination (applicable through 2032)
- States with significant coverage: TX (Spanish, Vietnamese, Chinese), CA (Spanish, Chinese, Filipino, Korean, Vietnamese), NY (Spanish, Chinese, Korean), FL (Spanish, Haitian Creole)
- Data: 2020 ACS 5-year estimates for limited-English-proficient citizens by county and language group
- Empirical: compare county-splitting rates in bisect vs. enacted maps for Section 203-covered counties in TX and CA
- VRA interaction: identify counties with both Section 2 and Section 203 coverage; document \textsc{bisect} VRA mode treatment

## Test Invariants

- L0: Section 203 coverage thresholds are computed correctly from ACS data for all covered jurisdictions
- L1: On a synthetic county with exactly the threshold population, coverage determination triggers correctly
- L2: TX Section 203 coverage jurisdictions are correctly identified and match Census Bureau 2023 determination

## Legal / Practitioner Value

Section 203 is often overlooked in redistricting analysis, which focuses on Section 2 (majority-minority districts) and Section 5 (preclearance, now largely dormant after \textit{Shelby County v.\ Holder}). But Section 203 creates independent legal obligations that affect every redistricting plan — if a district boundary splits a Section 203-covered county, the election administration obligations run to the county regardless of district boundaries. For expert witnesses and commission staff, D.7 provides the reference for identifying which jurisdictions have Section 203 obligations, how those obligations interact with redistricting, and whether algorithmic redistricting minimizes or exacerbates Section 203 implementation challenges through its county-splitting pattern.

## Section Structure

§1 Introduction: Section 203 in the VRA Framework, §2 Coverage Determination: Statutory Formula and Census Data, §3 2023 Coverage Determination: Which Jurisdictions Are Covered, §4 Section 203 and Redistricting: County-Splitting Implications, §5 Interaction with Section 2 Majority-Minority Requirements, §6 Bisect VRA Mode: Handling Section 203 and Section 2 Simultaneously, §7 Conclusion
