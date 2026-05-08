# Prison Gerrymandering and Algorithmic Redistricting

**Series**: D.6
**Status**: Accepted 3.5/4
**Target**: Yale Law Journal

## Algorithm / Subject

Legal and empirical analysis of prison gerrymandering — the practice of counting incarcerated people at the location of their prison rather than their home community in the decennial Census, which inflates the population count (and therefore the representation and federal funding allocation) of rural legislative districts containing prisons while deflating the count for urban and suburban communities from which most incarcerated people come. The paper examines the constitutional and legal framework for prison gerrymandering, the states that have enacted population count corrections (California, New York, Maryland, and others), the interaction with the \textsc{bisect} algorithmic redistricting pipeline (which uses Census data as its population input), and whether the \textsc{bisect-data} preprocessing step should implement prison population adjustments.

## Key Claims

1. Prison gerrymandering systematically distorts the population base used for redistricting: rural districts containing large prisons receive inflated Census counts, reducing the number of ordinary residents per representative in those districts, while urban districts from which incarcerated people disproportionately come are under-counted — a pattern that has racial and partisan implications because incarcerated populations are disproportionately Black and Hispanic, and rural prison districts tend to vote Republican.
2. At least 12 states have enacted legislation to reallocate prison populations to their pre-incarceration home addresses for redistricting purposes as of 2026, and the Census Bureau's 2020 Detailed Demographic and Housing Characteristics (DHC) file provides the prison-facility-level data needed to implement these adjustments — the \textsc{bisect-data} pipeline can incorporate these adjustments using available Census data before running the redistricting algorithm.
3. \textsc{Bisect} maps computed from unadjusted Census data in states with large rural prisons will systematically over-represent rural areas relative to their actual resident voting-age population; applying prison population adjustments before running \textsc{bisect} corrects this distortion without requiring any change to the algorithm itself.

## Layer

Legal / Data

## Empirical Targets

- States with large prison populations: TX, CA, NY, FL, PA, OH
- Data: 2020 Census group quarters (prison/jail population) by facility, by county
- Compare: district populations under unadjusted Census data vs. prison-adjusted Census data
- Identify: which \textsc{bisect} districts change substantially when prison adjustment is applied
- Benchmark: states with enacted prison population adjustment laws (CA, NY, MD) — use their official adjusted datasets to validate \textsc{bisect-data}'s adjustment implementation

## Test Invariants

- L0: Adjusted total population = unadjusted total population (prison adjustment reallocates, does not add/subtract people)
- L0: Adjusted prison-facility population $\geq 0$ for all facilities
- L1: On a synthetic 2-county example with one prison (500 inmates) in county A (all from county B), adjustment transfers all 500 from county A to county B
- L2: California adjusted population matches CA's official redistricting-adjusted population within 0.1\%

## Legal / Practitioner Value

Prison gerrymandering is a growing legal and political issue with documented VRA implications. The Supreme Court in \textit{Evenwel v.\ Abbott}\footnote{\textit{Evenwel v.\ Abbott}, 578 U.S. 54 (2016)} held that states may use total population (including non-citizens and non-voters) as the apportionment base, but did not address the prison gerrymandering question directly. States retain discretion to adjust their population base for state redistricting. Expert witnesses in state redistricting proceedings need to understand whether prison adjustment was applied, whether \textsc{bisect} can handle adjusted data, and what the redistricting implications of the adjustment are. This paper provides the complete framework.

## Section Structure

§1 Introduction: What Prison Gerrymandering Is and Why It Matters, §2 Constitutional and Legal Framework, §3 \textit{Evenwel v.\ Abbott} and Population Base Discretion, §4 State Enactments: Survey of Prison Adjustment Laws (2010–2026), §5 Implementation in Bisect-Data: Using Census DHC File, §6 Empirical Impact on Bisect Maps: TX, CA, NY Case Studies, §7 Conclusion
