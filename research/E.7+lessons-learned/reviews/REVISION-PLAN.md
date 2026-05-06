# Revision Plan — E.7 Six Systems, One Constraint: Design Lessons from Algorithmic Alternative Representation
Round 1 avg: 3.0/4

## Score Summary

| Reviewer | Score | Verdict |
|----------|-------|---------|
| Karypis  | 3/4   | Minor Revision |
| Rodden   | 3/4   | Minor Revision |
| Duchin   | 3/4   | Minor Revision |
| Stephanopoulos | 3/4 | Minor Revision |
| Liang    | 3/4   | Minor Revision |
| **Average** | **3.0/4** | **Minor Revision** |

**Gate Status**: PASSED (avg 3.0 >= 2.5, no score < 2.0)

---

## P1 — Required Fixes

### P1.1 — Resolve the 0.015 vs. 0.019 exchange rate inconsistency (Karypis, Liang)
**Issue**: Section 6.1 derives 0.019 from E.5 parametric results ($0.073 / 3.9$) and then adopts 0.015 from the "E.0 cross-system average" without explaining the difference or why 0.015 is "conservative." These two numbers are used interchangeably throughout the paper but measure different things.
**Fix**: Either (a) adopt a single canonical exchange rate with a documented derivation and report it throughout, or (b) maintain both and clearly label them: "within-system exchange rate (E.5-derived): 0.019 PP/pp" and "cross-system exchange rate (E.0 synthesis): 0.015 PP/pp," using each in contexts where it is appropriate. Explain the difference: within-system measures the slope of the E.5 optimization trajectory; cross-system measures the average slope between the DIA baseline and the alternative system points in the Pareto table.
**Target**: sections/06-lesson5.tex, sections/07-synthesis.tex (and all other uses)

### P1.2 — Document the 20% irreducibility bound calculation (Liang, Karypis)
**Issue**: The claim that 20% of the sorting gap is irreducible is derived from "the fraction of Democratic votes in census tracts with >70% Democratic composition," but the calculation is not provided.
**Fix**: Add a derivation or a specific citation to E.1 (the section and table where this is computed). Report: (a) the fraction of national Democratic votes in tracts with >70% Democratic composition, (b) how this fraction translates to the 20% irreducibility bound, and (c) why 70% is the threshold used. If the 70% threshold is arbitrary, note sensitivity to threshold choice (e.g., what changes if 60% or 80% is used?).
**Target**: sections/03-lesson2.tex (Section 3.3)

### P1.3 — Qualify the concavity claim to acknowledge insufficient data points (Liang)
**Issue**: "The relationship between district size and proportionality improvement is concave" is stated as a fact but is based on only three data points (1, 3, 5 members: 0%, 40%, 65% gap reduction).
**Fix**: Revise to: "The three available data points (1, 3, and 5 members) are consistent with a concave relationship: the marginal gap reduction is approximately 40% for the first additional two members and approximately 24% for the next two, suggesting diminishing returns. With only three data points, the functional form cannot be confirmed, but the pattern is sufficient to support the policy conclusion that the Fair Representation Act's 3--5 member specification captures most of the available improvement."
**Target**: sections/03-lesson2.tex (Section 3.4)

### P1.4 — Qualify the "not an artifact of the algorithm" claim (Karypis)
**Issue**: Section 6.3 claims the steep Pareto frontier "is not an artifact of the specific algorithm" but provides no comparative evidence from other algorithms.
**Fix**: Either (a) cite published work showing similar exchange rates for other edge-cut algorithms (spectral, flow-based) or for random-walk ensemble methods (GerryChain), or (b) qualify the claim to: "The steep frontier is consistent with the geographic sorting hypothesis, which predicts the same qualitative result regardless of algorithm: any algorithm that draws compact districts in areas with dense Democratic concentrations will waste Democratic votes. Whether the specific exchange rate (0.015--0.019 PP/pp) is algorithm-specific remains to be tested."
**Target**: sections/06-lesson5.tex (Section 6.3)

### P1.5 — Disaggregate the 4.2 pp national sorting gap by geography type (Rodden)
**Issue**: The national 4.2 pp Republican seat bonus is reported as a uniform figure, but the sorting gap is highly heterogeneous across states. The policy implications of Lessons 1 and 2 differ between low-sorting and high-sorting states.
**Fix**: Add a paragraph or small table in Section 2.1 or 2.2 showing the distribution of state-level sorting gaps: mean and SD across the 50 states, and the values for high-sorting states (IL, NY, PA, MI) vs. low-sorting states (IA, WI, AZ). Note that the lessons about multi-member district benefits apply most strongly to high-sorting states and have weaker implications for low-sorting states.
**Target**: sections/02-lesson1.tex (The Rodden Mechanism subsection)

---

## P2 — Suggested Improvements

### P2.1 — Add the E.4 null result to the abstract (Rodden)
**Issue**: The E.4 finding that partisan-similarity clustering doesn't change aggregate seat shares ($+4.0$ pp vs. $+4.2$ pp) is the paper's most compelling evidence and should be in the abstract.
**Suggestion**: Add one sentence to the abstract: "Notably, partisan-similarity clustering produces 95% safe seats but leaves the aggregate Republican seat bonus nearly unchanged ($+4.0$ pp vs. $+4.2$ pp at baseline), confirming that the sorting gap is structural rather than a drawing artifact."
**Target**: main.tex abstract

### P2.2 — Move the coda ("Value of Algorithmic Counterfactuals") to the introduction (Duchin)
**Issue**: The coda identifies a key methodological contribution --- algorithmic implementation of constitutionally unavailable systems as measurement instruments --- but this contribution is most useful as framing, not as a closing observation.
**Suggestion**: Move the substance of the coda (2--3 sentences) to the introduction's "Why Algorithmic Comparison Enables Design Learning" subsection.
**Target**: sections/08-conclusion.tex, sections/01-introduction.tex

### P2.3 — Reframe the conditional normative structure of the multi-member recommendation (Stephanopoulos)
**Issue**: The conclusion claims neutrality ("we do not advocate") while the synthesis section's evidence-marshaling is effectively an argument for multi-member districts. This inconsistency weakens the paper's credibility with sceptical readers.
**Suggestion**: Add explicit conditional framing: "IF proportionality and minority representation are weighted as important by the polity (a normative premise this paper does not adjudicate), THEN multi-member districts are the structural reform most supported by the E-track evidence. The empirical foundation is what the paper establishes; the normative weighting is what readers must supply."
**Target**: sections/07-synthesis.tex (Case for Multi-Member Districts subsection), sections/08-conclusion.tex

### P2.4 — Acknowledge the endogeneity of voter geography under alternative systems (Rodden)
**Issue**: All simulations hold voter geography constant, but voter geography may change under different electoral systems (residential sorting patterns may respond to electoral incentives).
**Suggestion**: Add 2 sentences in the Limitations section acknowledging that simulations treat voter geography as exogenous and noting that residential sorting patterns may be partly endogenous to the single-member district system.
**Target**: sections/08-conclusion.tex (Limitations and Future Research)
