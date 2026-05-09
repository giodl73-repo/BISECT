# Track Q — 2030 Forward Analysis

**Theme**: All 150 redistricting scenarios in the program are backward-looking (2000/2010/2020). Track Q is the program's forward-looking arm: what will 2030 redistricting look like given demographic trends, and how should redistricting infrastructure be prepared now?

**Core question per paper**: Given demographic projections and algorithmic requirements, what does the 2030 redistricting cycle demand?

## Track Chain

Q.0 (overview: why 2030 matters now) → Q.1 (demographic projections) → Q.2 (Sun Belt/Rust Belt shifts)
                                      → Q.3 (data infrastructure) → Q.4 (reapportionment projections)

## Papers

| Paper | Title | Horizon | Key Deliverable |
|-------|-------|---------|----------------|
| Q.0 | Planning for 2030: Why Redistricting Infrastructure Needs to Begin Now | 2030 | Framework |
| Q.1 | Demographic Projections for 2030 Congressional Redistricting | 2030 | State-level population projection table |
| Q.2 | Sun Belt Growth and Rust Belt Decline: Redistricting Implications | 2030 | Seat gain/loss projections; compactness implications |
| Q.3 | Data Infrastructure for 2030: When Tracts Are No Longer Adequate | 2030 | Block-group necessity analysis by state |
| Q.4 | 2030 Reapportionment Projections and Algorithmic Stability | 2030 | Huntington-Hill projection using Census Bureau population projections |

## Contracts

Every Q.1–Q.4 paper must:
- Use Census Bureau 2017–2060 population projections (vintage 2020 series) as the primary data source
- Report a confidence interval on each projection (Census Bureau provides low/high variant)
- Show the 2030 bisect pipeline requirement (resolution level, estimated run time)
- Cross-reference the current 2020 result as baseline

## Data Sources

- **Q.1, Q.2**: Census Bureau 2020 Population Projections (state-level, vintage 2020)
- **Q.3**: F.3's resolution rule: k/n > 0.05 → block-group required. Apply to projected 2030 seat counts.
- **Q.4**: Huntington-Hill with projected 2030 state populations from Q.1

## Time Sensitivity

2030 Census data will be released April 2031. Redistricting must complete by late 2031 in most states. The bisect pipeline's data infrastructure (block-group adjacency graphs for 39 states; block adjacency for 3) should be pre-built and validated by 2029. Q.3 identifies exactly which states need which resolution level so that data preparation can begin now.

## Track Thesis

The algorithmic redistricting infrastructure for 2030 must be built during 2026–2029 — *before* the Census data arrives — to be ready when needed. This requires knowing now which states will need block-group resolution (Q.3), which states will gain or lose seats (Q.4), and what the demographic landscape will look like (Q.1/Q.2). The track converts the program from a retrospective analysis into a prospective tool.
