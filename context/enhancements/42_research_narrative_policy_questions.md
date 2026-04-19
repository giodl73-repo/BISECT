# Enhancement 42: Research Narrative and Policy Questions

**Status**: Proposed
**Priority**: Critical
**Created**: January 17, 2026
**Commits**: [ad542e9](https://github.com/giodl_microsoft/redistricting/commit/ad542e9efb96bd2ee745b6ab6e77f2ee721b1789)
**Size**: M - 1,191 lines changed (6 files)

## Problem Statement

The project has strong technical implementation but lacks a clear research narrative answering: **What problem are we solving and why does it matter?**

**Current State:**
- Three papers exist demonstrating algorithmic redistricting works
- Technical excellence: METIS integration, edge-weighted optimization, multi-year pipeline
- Comprehensive metrics: compactness, demographics, political analysis
- Baseline comparison to enacted districts (Enhancement 11)

**Missing:**
- Clear research question or hypothesis
- Policy implications and recommendations
- Contribution beyond "we built a working system"
- Audience: Who needs this research and why?

## Core Research Questions to Answer

### 1. Primary Question: Should Algorithmic Redistricting Replace Human Processes?

**Sub-questions:**
- How much more compact are algorithmic districts vs. enacted districts? (quantify improvement)
- Does edge-weighting provide meaningful gains over unweighted partitioning?
- Are gains consistent across all states or only gerrymandered ones?
- What's the trade-off between compactness and traditional redistricting principles (county boundaries, communities of interest)?

**Testable Hypothesis:**
> "Algorithmic redistricting using edge-weighted graph bisection produces districts that are 30-50% more compact than human-drawn districts while maintaining equivalent population balance and contiguity, suggesting it could serve as a neutral baseline for redistricting reform."

### 2. Secondary Question: Can We Detect Gerrymandering?

**Sub-questions:**
- Which states show largest deviation from algorithmic baseline?
- Does deviation correlate with partisan control during redistricting?
- Can compactness serve as a legal/objective gerrymandering metric?

**Testable Hypothesis:**
> "States with single-party control during redistricting exhibit 40-60% lower compactness than algorithmic districts, while competitive/commission-drawn states show only 10-20% deviation."

### 3. Methodological Question: Does Edge-Weighting Matter?

**Sub-questions:**
- How much improvement does edge-weighting provide over unweighted bisection?
- Is the 2-4 hour computational cost justified by results?
- When does edge-weighting provide largest gains?

**Testable Hypothesis:**
> "Edge-weighted recursive bisection improves district compactness by 15-25% over unweighted partitioning, with largest gains in urban states with irregular tract boundaries."

## Target Audiences and Their Questions

### 1. Academic Researchers (Political Science, Operations Research)
**Their Question:** "What's the contribution?"
- **Answer:** Novel application of graph bisection to redistricting with edge-weighting innovation
- **Contribution:** Scalable algorithmic baseline for comparing redistricting proposals
- **Novelty:** First implementation across all 50 states with comprehensive metrics

### 2. Redistricting Reformers / Good Government Groups
**Their Question:** "Can we use this?"
- **Answer:** Provides objective, reproducible baseline for evaluating fairness
- **Use Case:** Challenge gerrymandered maps in court using compactness deviation
- **Practical Tool:** Public dashboard for citizens to explore alternative districts

### 3. State Legislators / Redistricting Commissions
**Their Question:** "Should we adopt algorithmic redistricting?"
- **Answer:** Shows feasibility and quantifies improvements over current practice
- **Trade-offs:** Highly compact but may split communities/counties
- **Recommendation:** Use as starting point, then adjust for local considerations

### 4. Judges / Legal Community
**Their Question:** "Is compactness a valid legal standard?"
- **Answer:** Provides objective, measurable criterion for evaluating maps
- **Precedent:** Many state constitutions require compactness (but don't define it)
- **Evidence:** Quantifies deviation from neutral baseline

## Proposed Paper Organization

### Paper 1: "Algorithmic Redistricting as a Neutral Baseline"
**Research Question:** Can we create objectively compact districts?
**Contribution:** Methodology, implementation, baseline results
**Audience:** Academic (methodology)

### Paper 2 (or 03): "Evaluating Enacted Congressional Districts Against Algorithmic Baselines"
**Research Question:** How much deviation exists between enacted and algorithmic districts?
**Contribution:** Systematic 50-state comparison, gerrymandering detection
**Audience:** Policy / Legal

### Paper 3: "Edge-Weighted Graph Bisection for Redistricting"
**Research Question:** Does boundary-aware optimization improve district compactness?
**Contribution:** Novel edge-weighting approach, comparative analysis
**Audience:** Academic (technical)

## Implementation Tasks

### Phase 1: Define Research Questions (This Enhancement)
- [ ] Draft clear research questions for each paper
- [ ] Identify testable hypotheses
- [ ] Define success criteria (what would convince skeptics?)
- [ ] Map existing data/results to questions

### Phase 2: Gap Analysis
- [ ] What data do we have? (baseline comparison, compactness metrics, political analysis)
- [ ] What's missing? (VRA analysis, county splitting, COI impacts)
- [ ] What additional analysis is needed?

### Phase 3: Paper Reorganization
- [ ] Review existing papers (01, 02, 03)
- [ ] Restructure around clear research questions
- [ ] Add "Policy Implications" sections
- [ ] Add "Limitations" sections (what can't this solve?)

### Phase 4: Documentation Updates
- [ ] Update README.md with clear problem statement
- [ ] Add "Research Questions" to ARCHITECTURE.md
- [ ] Create RESEARCH_AGENDA.md documenting open questions
- [ ] Update CONTRIBUTING.md for academic contributors

### Phase 5: Narrative Development
- [ ] Draft 1-page executive summary (for non-technical audiences)
- [ ] Create "elevator pitch" (30 seconds explaining the research)
- [ ] Develop talking points for presentations
- [ ] Prepare FAQ for common questions

## Key Questions to Answer

**For Users:**
1. What problem does this solve? → Gerrymandering / lack of objective redistricting
2. Who should use this? → Reformers, researchers, commissions, citizens
3. What should they do with it? → Compare to enacted maps, detect manipulation, inform policy

**For Researchers:**
1. What's novel? → Scalable implementation, edge-weighting, comprehensive evaluation
2. What's the contribution? → Neutral baseline methodology for redistricting evaluation
3. What's next? → Block-level data, VRA compliance, COI integration

**For Policymakers:**
1. Should we adopt this? → Not necessarily as-is, but as a starting point
2. What are the trade-offs? → Compactness vs. communities/counties/politics
3. What's the path to implementation? → Pilot in reform-friendly states, refine

## Success Criteria

- [ ] Every paper has a clear research question in abstract/introduction
- [ ] README.md explains "the problem" in first paragraph
- [ ] Can explain project in 30 seconds to non-expert
- [ ] Policy recommendations section exists in at least one paper
- [ ] "Limitations and Future Work" sections added to papers
- [ ] Presentations/talks lead with research question, not technical details

## Related Enhancements

- [Enhancement 11: Baseline Comparison](11_baseline_comparison.md) - Provides data for comparison research question
- [Enhancement 12: Edge-Weighted Analysis](12_edge_weighted_analysis.md) - Answers edge-weighting research question
- [Enhancement 41: Public Distribution](41_public_distribution.md) - Makes research accessible to broader audience
- [Enhancement 44: Real-World Constraints](44_real_world_constraints.md) - Addresses practical adoption questions
- [Enhancement 45: Baseline Data Organization](45_baseline_data_organization.md) - Organizes evidence for claims

## Notes

**User Feedback (January 17, 2026):**
> "we have gathered baseline data for papers\03_ but i take your point it isnt organized enough"

This suggests:
- Data exists but needs better organization (Enhancement 45)
- Papers exist but may need narrative restructuring
- Technical work is done; storytelling needs improvement

**Key Insight:**
The project has solved the *technical* problem (algorithmic redistricting works). Now solve the *narrative* problem (why does anyone care?).
