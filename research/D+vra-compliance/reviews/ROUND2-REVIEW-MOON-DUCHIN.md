# Round 2 Review: Voting Rights Act Compliance Through Edge-Weighted Graph Partitioning

**Reviewer**: Moon Duchin (Rutgers University)
**Round**: 2
**Date**: February 7, 2026

---

## Overall Assessment

The revision makes significant progress on legal framing and empirical validation. The enacted plan comparison (Table 3) and constitutional analysis (Section 2.3) address critical gaps from Round 1. However, the paper remains mathematically shallow—edge-cut is still the primary compactness metric, ensemble analysis is absent, and the optimization landscape is unexamined. This is a solid empirical study but lacks the mathematical rigor for premier computational venues.

For political science publication, this is now acceptable. For mathematical or computational journals, more work is needed.

**Score**: 3/4 (Accept with minor revisions)

**Change from Round 1**: 3.0 → 3.0 (unchanged)

*Note: Score unchanged because Round 1 was already "accept with major revisions" for legal issues (now addressed), but mathematical issues (which I emphasized) remain unaddressed.*

---

## Improvements from Round 1

### I1: Legal Framing is Much Better (P1.1, P1.3, P1.4)
The reframing as "demographic viability" and addition of constitutional analysis substantially improve legal accuracy. Section 2.3's narrow tailoring argument (+4% edge cut, minimal weight factors) is well-reasoned.

The *Allen v. Milligan* integration demonstrates practical relevance—algorithmic methods can achieve court-mandated outcomes.

### I2: Enacted Plan Comparison Provides Baseline (P1.2)
Table 3 is the critical missing piece. Showing algorithmic plans outperform enacted plans (4 MM vs. 3 MM districts) validates practical significance.

However, compactness comparison is still edge-cut focused. More on this below.

---

## Continuing Mathematical Deficiencies

### MD1: Edge-Cut Remains Primary Compactness Metric
Table 3 includes Polsby-Popper averages (good!), but:
- **Edge-cut is still emphasized throughout** (every results table reports edge cut)
- **PP is relegated to comparison table** rather than primary metric
- **No Reock, convex hull, or other geometric metrics**
- **No per-district distributions** (boxplots)—only averages mask variation

**Why this matters**: Edge-cut is graph-theoretic, not geometric. From a metric geometry perspective, you're optimizing the wrong quantity. PP should be primary, edge-cut secondary (if reported at all).

**Evidence of the problem**: You claim Alabama edge-weighting "maintains compactness" with -1.4% edge cut change. But Table 3 shows PP: 0.34 (algorithmic) vs. 0.31 (enacted)—your plans are **less compact** geometrically despite "better" edge cut. These metrics tell different stories.

**Recommendation**: Flip emphasis—make PP primary throughout, relegate edge-cut to appendix. Add Reock for robustness.

### MD2: No Ensemble Analysis (P3.1)
**This was my top request in Round 1 and remains unaddressed**. Single plans without distributional context are mathematically unsatisfying. Questions unanswered:
- In a neutral ReCom ensemble for Alabama, what % of plans achieve 2 MM districts?
- Is your edge-weighted plan typical or a rare outlier?
- What does the Pareto frontier (VRA vs. compactness) look like?

**Why this matters**: If 40% of neutral ensemble plans achieve 2 MM in Alabama, your "breakthrough" is just finding a typical region of plan-space. If only 1% achieve it, the breakthrough is real.

**Feasibility**: MGGG has ReCom code publicly available. Generating 100K Alabama plans would take ~1 day compute time.

**Impact on score**: Without ensemble context, I cannot assess whether edge-weighting finds unusual plans or standard ones. This is a fundamental gap for mathematical evaluation.

### MD3: Optimization Landscape Unexamined (M4 from Round 1)
Still treating METIS as black box. Questions unanswered:
- **Local vs global optima**: How do you know these are near-optimal?
- **Multiple optima**: Are there many plans with similar edge cuts?
- **Stability**: Run METIS with 10 different seeds—how much do results vary?
- **Pareto frontier**: Plot edge cut vs. MM count for all weight factors—are solutions Pareto-optimal?

**Recommendation**: Add "Optimization Analysis" subsection showing sensitivity to METIS parameters and multiple runs.

---

## Remaining Issues

### RI1: Spatial Clustering Still Unquantified (P2.4)
You claim "geographic clustering determines success" but provide no spatial statistics. This is easy to fix:
- **Compute Moran's I** for minority tract distribution (one number per state)
- **Test correlation**: Moran's I vs. VRA success
- **Takes ~1 day** with spatial statistics software

### RI2: District-Level Variation Hidden (P2.2)
Table 3 reports PP averages: Alabama 0.34, Georgia 0.28. But what about individual districts?
- Are some districts highly non-compact (PP < 0.10) while others are compact?
- Does creating MM districts sacrifice compactness in those districts specifically?

**Recommendation**: Boxplots showing per-district PP distributions, or at minimum report (min, 25th, median, 75th, max).

### RI3: Resolution Dependence Not Discussed (m2 from Round 1)
Census tracts are arbitrary units. Would block-level data (10× resolution) change results? One paragraph acknowledging this limitation would suffice.

---

## Minor Technical Issues

1. **Section 4.2.2 (Alabama), line 8**: "Edge cut = 276" without context. Is this good/bad? Compare to typical compact 7-district plans.

2. **Figure missing**: Pareto frontier (VRA compliance vs. compactness) for different weight factors. This is the most important figure for understanding tradeoffs.

3. **Table 3**: PP scores seem low (0.28-0.34). For context, circle has PP = 1.0, typical compact districts achieve PP = 0.3-0.5. Are these scores good? Add comparison to "benchmark compact districts."

4. **Contiguity verification**: Did you verify all districts are contiguous? With high edge weights, METIS might produce disconnected components.

---

## Mathematical Significance

From a mathematical perspective, this paper demonstrates that **heuristic edge-weighting improves VRA outcomes** in an empirical sense. This is useful but not deep:

**What's established**:
- Edge-weighting beats multi-constraint on 5 test cases (40% → 80% success)
- Minimal compactness cost (+4% edge cut, ~3% PP decrease)
- Outperforms enacted plans (4 MM vs. 3 MM)

**What's not established**:
- Why edge-weighting works (theoretical understanding)
- When it works (beyond 5 states)
- Whether solutions are near-optimal or just "good enough"
- Whether results are robust to parameter changes, seeds, resolution

For *Discrete & Computational Geometry* or *Operations Research*, you'd need theoretical analysis. For *Political Analysis* or *APSR*, empirical demonstration suffices.

---

## Recommendation

**Accept for publication in political science venues** (*APSR*, *Political Analysis*, *Election Law Journal*) after minor revisions:
- Add spatial clustering statistics (Moran's I)
- Add district-level PP distributions (boxplots)
- Flip emphasis to PP over edge-cut

**Not recommended for mathematical venues** (*DCG*, *SICOMP*, *Mathematical Programming*) without ensemble analysis, optimization landscape analysis, and theoretical justification.

**Conditional strong accept**: If authors add ensemble analysis (P3.1) comparing edge-weighted plans to ReCom distributions, I'd raise score to 3.5/4 or 4/4. This is the key missing piece for mathematical rigor.

---

**Summary**: Solid empirical work with legal grounding, now suitable for political science publication. Mathematically shallow—lacks ensemble context, optimization analysis, and geometric compactness focus. For political science: accept. For mathematics: needs more work.
