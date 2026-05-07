# Track E — Experimental Extensions

**Module**: track-E
**Theme**: If we relax the single-member district constraint, what alternative electoral architectures does algorithmic redistricting enable, and what do they reveal about the geometric cost of current electoral rules?
**Papers**: 8
**Author**: Giovanni Della Libera
**Created**: 2026-05-07

---

## Tracks

### Track alternatives

**Theme**: What electoral architectures emerge when the bisection algorithm is applied to multi-member districts, county boundaries, national geography, and partisan optimization objectives?

**Chain**: `E.1+multi-member-districts` → `E.2+county-representation` → `E.3+national-redistricting` → `E.4+partisan-similarity-districts` → `E.5+party-based-allocation` → `E.6+international-applications`

**Arc**: E.1 introduces multi-member districts and proportional representation as a bisection variant, quantifying the proportionality gain. E.2 explores direct county representation — using county boundaries as the atomic unit — and the compactness and population-equality tradeoffs. E.3 computes national redistricting without state boundaries, quantifying the geometric cost of federalism (how much compactness is lost by requiring state-boundary respect). E.4 creates partisan similarity districts — algorithmic safe seats — and measures their effect on polarization metrics. E.5 explores partisan fairness through direct algorithmic optimization, revealing the tension between partisan targets and geographic feasibility. E.6 applies the methodology to international electoral systems.

### Track synthesis

**Theme**: What general lessons emerge from the experimental track about the limits and possibilities of algorithmic redistricting beyond the single-member-district framework?

**Chain**: `E.0+experimental-overview` → `E.7+lessons-learned`

**Arc**: E.0 frames the experimental program and its relationship to the base algorithm. E.7 synthesizes lessons learned across six alternative systems — what works, what doesn't, and what the experiments reveal about which features of the current system are geometric necessities vs. political choices.

---

## Papers

| Paper | Tracks | Primary Number | Status | Venue |
|-------|--------|----------------|--------|-------|
| E.0+experimental-overview | synthesis | overview of 8 experimental extensions | draft | internal |
| E.1+multi-member-districts | alternatives | proportionality gain from multi-member variant | draft | Electoral Studies |
| E.2+county-representation | alternatives | compactness and pop-equality cost of county-unit districting | draft | State Politics |
| E.3+national-redistricting | alternatives | geometric cost of federalism (compactness loss from state bounds) | draft | Comparative Pol. Studies |
| E.4+partisan-similarity-districts | alternatives | polarization effect of algorithmic safe seats | draft | AJPS |
| E.5+party-based-allocation | alternatives | partisan fairness vs. geographic feasibility tension | draft | Electoral Studies |
| E.6+international-applications | alternatives | cross-system application results | draft | Electoral Studies |
| E.7+lessons-learned | synthesis | lessons from 6 alternative systems | draft | internal |

---

## Module Arc

Track E is the counterfactual track — it answers the question "what if?" by applying the algorithm to alternative rule systems. Its scientific value is twofold: (1) the alternative results illuminate what is geometric necessity vs. political choice in the current single-member system, and (2) they extend the algorithm's relevance beyond US congressional redistricting to international audiences and proportional representation debates. Track E is the most speculative track in the portfolio — papers here are exploratory rather than confirmatory — and the panel should assess whether the counterfactual findings are presented with appropriate epistemic humility. Track E has no mandatory prerequisites in Tracks B–D, but papers should cite B.3 (single-objective vs. multi-constraint) when discussing the optimization logic of alternatives.
