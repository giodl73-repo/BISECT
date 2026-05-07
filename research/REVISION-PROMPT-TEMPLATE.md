# Revision Plan Template - Redistricting Research

Use this template to create revision plans for papers before expert review simulation.

---

## Paper Information

**Title**: [Paper title]
**Wave**: [Wave number and name]
**Target Venue**: [Conference/journal]
**Authors**: [Author list]
**Date**: [Current date]

---

## 1. Paper Summary

### Core Contributions

| # | Contribution | Type | Novel? | Evidence |
|---|-------------|------|--------|----------|
| 1 | [Brief description] | [Algorithm/System/Analysis/Metric] | [Yes/No/Partial] | [Where proven] |
| 2 | | | | |
| 3 | | | | |

### Key Results

- **Main finding 1**: [Quantified result with metrics]
- **Main finding 2**: [Quantified result with metrics]
- **Main finding 3**: [Quantified result with metrics]

### Target Venues

| Venue | Fit (1-5) | Reasoning | Deadline |
|-------|-----------|-----------|----------|
| [Conference 1] | 5 | [Why perfect fit] | [Date] |
| [Conference 2] | 4 | [Why good fit] | [Date] |
| [Conference 3] | 3 | [Why acceptable fit] | [Date] |

---

## 2. Expert Review Panel

Select 5-7 reviewers from [REVIEWER-DATABASE.md](REVIEWER-DATABASE.md).

### Reviewer 1: [Name]
- **Affiliation**: [University/Lab]
- **Expertise**: [Domain areas]
- **Key Question**: [Their signature concern]
- **Why Selected**: [Relevance to this paper]
- **Expected Focus**: [What they'll scrutinize]

### Reviewer 2: [Name]
[Same structure]

[Continue for 5-7 reviewers]

---

## 3. Expected Review Themes

### Theme Priority Matrix

| Theme | Rev1 | Rev2 | Rev3 | Rev4 | Rev5 | Priority |
|-------|------|------|------|------|------|----------|
| Algorithm quality | H | H | M | L | M | **Critical** |
| Political neutrality | L | H | H | H | M | **Critical** |
| Scalability | H | M | L | L | H | **High** |
| Constitutional compliance | L | M | M | H | L | **High** |
| GIS data quality | M | L | L | M | H | **Medium** |
| Compactness metrics | M | H | H | M | L | **High** |
| Related work | M | M | M | M | M | **Medium** |
| Reproducibility | H | L | M | L | H | **Critical** |

*H=High priority for reviewer, M=Medium, L=Low*

### Detailed Critique Predictions

#### Theme: Algorithm Quality
**Likely concerns**:
- [Specific technical question reviewer will ask]
- [Performance comparison they'll want]
- [Theoretical guarantee they'll seek]

**Pre-emptive actions**:
- [ ] [Action to address before submission]
- [ ] [Action to address before submission]

#### Theme: Political Neutrality
**Likely concerns**:
- [Partisan bias question]
- [Geographic clustering concern]
- [Baseline comparison request]

**Pre-emptive actions**:
- [ ] [Action to address before submission]
- [ ] [Action to address before submission]

[Continue for each critical/high theme]

---

## 4. Pre-Submission Checklist

### Statistical Validation
- [ ] Confidence intervals computed for all key metrics
- [ ] Effect sizes reported (not just p-values)
- [ ] Multiple hypothesis correction applied where needed
- [ ] Outliers identified and explained
- [ ] Sensitivity analysis performed

### Reproducibility Requirements
- [ ] Code repository publicly available
- [ ] Data sources documented with download links
- [ ] Environment specifications provided (Python versions, dependencies)
- [ ] README with step-by-step reproduction instructions
- [ ] Runtime estimates provided
- [ ] All figures reproducible from code

### Technical Completeness
- [ ] All claims supported by evidence in paper
- [ ] Limitations section included
- [ ] Failure cases discussed
- [ ] Parameter choices justified
- [ ] Baseline comparisons included
- [ ] Ablation studies conducted

### Writing Quality
- [ ] Within page limit (including references)
- [ ] Abstract conveys all key contributions
- [ ] Introduction motivates problem clearly
- [ ] Related work comprehensive and fair
- [ ] Figures/tables have descriptive captions
- [ ] No jargon without definition
- [ ] Proofread for typos and grammar

### Domain-Specific (Redistricting)
- [ ] Constitutional compliance discussed
- [ ] Partisan effects analyzed
- [ ] Geographic considerations explained
- [ ] Census data quality validated
- [ ] Comparison to real/existing plans

---

## 5. Timeline

### Week 1: Core Revisions
- **Day 1-2**: Address algorithm concerns
- **Day 3-4**: Political analysis and neutrality evidence
- **Day 5-6**: Related work expansion
- **Day 7**: Writing polish

### Week 2: Validation & Polish
- **Day 8-9**: Statistical validation
- **Day 10-11**: Reproducibility artifacts
- **Day 12**: Internal review
- **Day 13**: Final edits
- **Day 14**: Expert review simulation (Round 1)

### Milestones
- [ ] **Day 7 checkpoint**: All major concerns addressed
- [ ] **Day 11 checkpoint**: Reproducibility verified
- [ ] **Day 14 checkpoint**: Ready for expert review

---

## 6. Quality Gates

### Gate 1: Technical Soundness
**Criteria**:
- Algorithm correctness verified
- Performance claims substantiated
- Edge cases tested
- Limitations documented

**Threshold**: All criteria met
**Status**: [GO/NO-GO]

### Gate 2: Political Neutrality
**Criteria**:
- Partisan effects analyzed
- Geographic bias examined
- Comparison to baselines included
- Neutrality claim supported

**Threshold**: All criteria met
**Status**: [GO/NO-GO]

### Gate 3: Reproducibility
**Criteria**:
- Code available
- Data accessible
- Instructions complete
- Environment documented

**Threshold**: All criteria met
**Status**: [GO/NO-GO]

### Gate 4: Writing Quality
**Criteria**:
- Within page limit
- Clear and concise
- No typos/grammar errors
- Figures publication-ready

**Threshold**: All criteria met
**Status**: [GO/NO-GO]

### Gate 5: Cross-Disciplinary Rigor
**Criteria**:
- CS concerns addressed
- Political science concerns addressed
- Legal concerns addressed
- GIS concerns addressed

**Threshold**: 4/4 domains satisfied
**Status**: [GO/NO-GO]

---

## 7. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| [Concern that could block acceptance] | [H/M/L] | [H/M/L] | [What to do] |
| Algorithm complexity concerns | M | H | Add complexity analysis section |
| Insufficient baselines | H | H | Add comparison to 3+ existing methods |
| Partisan bias appearance | M | H | Add sensitivity analysis across states |
| [Add more] | | | |

---

## 8. Post-Review Actions

After Round 1 expert review simulation:

1. **Categorize feedback**:
   - Blocking issues (must fix)
   - Major issues (should fix)
   - Minor issues (nice to fix)
   - Out of scope (defer to future work)

2. **Prioritize revisions**:
   - Address all blocking issues
   - Address major issues by priority
   - Address minor issues if time permits

3. **Update paper**:
   - Revise sections based on feedback
   - Add experiments/analysis as needed
   - Strengthen weak areas

4. **Re-run expert review**:
   - Generate Round 2 reviews
   - Verify improvements
   - Repeat until target score >= 2.5/4

---

*Template version: 1.0*
*Last updated: February 2026*
