---
title: "Data Infrastructure for 2030: When Tracts Are No Longer Adequate"
series: Q.3
status: Planned
date: 2026-05-09
track: Q-forward-2030
---

## Claims
1. Applying F.3's resolution rule (k/n > 0.05 requires block-group resolution) to projected 2030 seat counts: 43 states (vs. 39 in 2020) will require block-group resolution for their state house chambers. 4 additional states will cross the threshold due to seat gains.
2. Block-group adjacency graphs for all 50 states need to be pre-built. These are not available as standard Census Bureau products — they must be constructed from TIGER/Line shapefiles by the bisect `bisect fetch --resolution block-group` command.
3. The storage and computation requirements for 2030 are estimated: block-group adjacency nationally is ~5× larger than tract adjacency (~1.2M vs ~240K units). Computation time scales approximately linearly (estimated ~90 minutes for full 50-state sweep at block-group resolution on 8 cores).
4. Pre-building and validating block-group adjacency graphs by 2029 reduces the 2031 redistricting window requirement from 6 months to 2 months (the remainder is plan review and VRA verification, not computation).
