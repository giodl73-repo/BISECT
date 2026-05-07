# Track F — State Legislative Redistricting

**Module**: track-F
**Theme**: Does the algorithm generalize from congressional redistricting to state legislative chambers — including single-chamber, bicameral, and high-k chambers (WA 98, TX 150, NH 400)?
**Papers**: 7
**Author**: Giovanni Della Libera
**Created**: 2026-05-07

---

## Tracks

### Track state-legislative

**Theme**: Can minimum-edge-cut recursive bisection scale from 435 congressional districts to state legislative chambers with varying k, bicameral constraints, and state-specific redistricting criteria?

**Chain**: `F.0+state-legislative-overview` → `F.1+single-chamber-all-50` → `F.2+bicameral-nesting` → `F.3+multi-resolution-high-k` → `F.4+state-criteria-variation` → `F.5+compactness-legislative-vs-congressional` → `F.6+vra-state-legislative`

**Arc**: F.0 frames the state legislative extension program. F.1 applies the algorithm to all 50 state house chambers in a single sweep, establishing baseline performance. F.2 introduces NestSection for bicameral redistricting (senate = 2× house spine compatibility), quantifying the compactness cost of the nesting constraint. F.3 addresses high-k chambers (WA 98, TX 150, NH 400) via multi-resolution approaches, demonstrating scalability. F.4 documents state-by-state variation in redistricting criteria (contiguity, compactness, population equality, county splits) and how the algorithm adapts. F.5 compares compactness outcomes between state legislative and congressional maps under the same algorithm. F.6 extends VRA compliance analysis to state legislative chambers.

---

## Papers

| Paper | Tracks | Primary Number | Status | Venue |
|-------|--------|----------------|--------|-------|
| F.0+state-legislative-overview | state-legislative | overview: all 50 states × state legislative chambers | draft | internal |
| F.1+single-chamber-all-50 | state-legislative | single-chamber state house results: all 50 states | draft | State Politics |
| F.2+bicameral-nesting | state-legislative | compactness cost of senate = 2× house nesting constraint | draft | State Politics |
| F.3+multi-resolution-high-k | state-legislative | multi-resolution approach for k = 98, 150, 400 | draft | State Politics |
| F.4+state-criteria-variation | state-legislative | state-by-state criteria variation and algorithm adaptation | draft | State Politics |
| F.5+compactness-legislative-vs-congressional | state-legislative | compactness comparison: legislative vs. congressional | draft | Pol. Analysis |
| F.6+vra-state-legislative | state-legislative | VRA compliance for state legislative chambers | draft | Yale LJ |

---

## Module Arc

Track F extends Track B's congressional redistricting algorithm to the full landscape of US legislative redistricting — 50 state house chambers, bicameral systems, and high-k chambers up to 400 districts. The key scientific question is whether the compactness and stability properties established for 435 congressional districts generalize when k varies by two orders of magnitude. The key legal question (F.6) is whether VRA compliance at the congressional level (Track D) transfers to state legislative chambers, which have different Section 2 and Section 5 (pre-Shelby) obligations. Track F depends on B.13 (NestSection), B.14 (VRASection), and D.0–D.3 for the VRA methodology. Track F results are relevant to A.5 (policy brief) because state legislatures are the primary redistricting actors for both congressional and state maps.
