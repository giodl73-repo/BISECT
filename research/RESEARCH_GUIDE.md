# Research Process Guide - Redistricting Project

**Purpose**: How to run the AI-simulated expert review process for redistricting research papers.

**Applies to**: All papers derived from development waves in this project.

---

## The Review Process

### Overview

Each paper undergoes a multi-round AI-simulated expert review process designed to anticipate real reviewer concerns and strengthen the submission before venue submission.

### Process Steps

```
1. REVISION PLAN CREATION
   |-- Paper summary and key contributions
   |-- Target venue analysis
   |-- Expert panel selection (5-7 reviewers per paper)
   |-- Expected review themes by expert
   |-- Pre-submission checklist
   +-- Revision timeline

2. EXPERT REVIEW SIMULATION (Round 1)
   |-- Each expert reviews from their domain perspective
   |-- Scores: 1/4 (Reject) to 4/4 (Strong Accept)
   |-- Major issues (blocking) identified
   |-- Minor issues noted
   +-- Specific questions for authors

3. REVISION IMPLEMENTATION
   |-- Address major issues (required)
   |-- Address minor issues (recommended)
   |-- Update paper sections
   |-- Add missing experiments/data
   +-- Strengthen related work

4. EXPERT REVIEW SIMULATION (Round 2+)
   |-- Re-review with revisions
   |-- Updated scores
   |-- Verification of fixes
   +-- New issues if any

5. SUBMISSION READINESS
   |-- All reviewers at Weak Accept (2/4) or higher
   |-- Average score >= 2.5/4
   |-- All quality gates passed
   +-- Reproducibility artifacts ready
```

### Quality Gates

Each paper must pass these gates before submission:

| Gate | Criteria |
|------|----------|
| **Statistical Rigor** | Confidence intervals, effect sizes, significance tests |
| **Reproducibility** | Code, data, environment documented |
| **Technical Completeness** | All major reviewer concerns addressed |
| **Writing Quality** | Within page limit, claims supported, limitations stated |
| **Internal Review** | Co-author approval, final polish |

### Expert Selection Criteria

1. **Domain Expertise**: Each panel includes 5-7 experts with relevant backgrounds
2. **Cross-Disciplinary**: Mix of graph algorithms, political science, law, GIS, optimization
3. **Complementary Perspectives**: Different reviewers focus on different aspects
4. **Realistic Simulation**: Experts chosen who would plausibly review at target venue

Select experts from the global [REVIEWER-DATABASE.md](REVIEWER-DATABASE.md).

### Scoring Rubric

| Score | Label | Meaning |
|-------|-------|---------|
| 1/4 | Reject | Major flaws, not suitable for venue |
| 2/4 | Weak Accept | Acceptable with revisions |
| 3/4 | Accept | Good contribution, minor issues |
| 4/4 | Strong Accept | Excellent, must accept |

**Target for Submission**: Average >= 2.5/4, no scores below 2/4

---

## Special Considerations for Redistricting Research

### Cross-Disciplinary Nature
Papers in this domain must satisfy multiple audiences:
- **CS/Algorithms**: Graph partitioning quality, computational complexity, scalability
- **Political Science**: Electoral implications, partisan effects, representation
- **Law**: Constitutional compliance, Voting Rights Act, judicial standards
- **GIS**: Spatial accuracy, census data integrity, geographic validity

### Common Reviewer Concerns

#### From Algorithm Experts (Karypis, Çatalyürek, Hendrickson)
1. **Partitioning Quality**: How does METIS performance compare to theoretical bounds?
2. **Scalability**: How does the algorithm scale with graph size?
3. **Parameter Sensitivity**: How do edge weights, imbalance tolerance affect results?
4. **Optimization Trade-offs**: Compactness vs population balance vs contiguity

#### From Political Scientists (Rodden, Chen, Duchin, Stephanopoulos)
1. **Partisan Neutrality**: Does the algorithm favor one party?
2. **Geographic Clustering**: How does urban/rural divide affect outcomes?
3. **Compactness Metrics**: Are metrics politically meaningful?
4. **Comparison to Baselines**: How does this compare to existing plans?

#### From Legal Experts (Pildes, Gerken)
1. **One-Person-One-Vote**: Does ±0.5% meet constitutional requirements?
2. **Voting Rights Act**: Are minority opportunities protected?
3. **Justiciability**: Could this be used as a legal standard?
4. **Precedent Alignment**: Does this align with Supreme Court doctrine?

#### From GIS Experts (Goodchild, Yuan)
1. **Census Data Quality**: How are tract boundaries handled?
2. **Adjacency Detection**: How are shared boundaries identified?
3. **Topology Validation**: Are contiguity constraints enforced?
4. **Resolution Effects**: How does tract size affect results?

#### From Optimization Experts (Phillips, Cook)
1. **Approximation Quality**: What are the optimality bounds?
2. **Constraint Satisfaction**: How are hard constraints enforced?
3. **Heuristic Justification**: Why METIS over other approaches?
4. **Solution Space**: How diverse are the solutions?

---

## Reproducing the Process

To apply this review process to a new paper:

### 1. Create Revision Plan

```markdown
# Revision Plan: [Paper Title]

## 1. Paper Summary
- Core contributions (table format)
- Key results
- Target venues with fit analysis

## 2. Expert Review Panel
- 5-7 experts selected from REVIEWER-DATABASE.md
- Profile each expert's background
- Identify their likely review focus

## 3. Expected Review Themes
- Matrix: themes x experts (priority levels)
- Detailed critique predictions
- Pre-emptive actions for each theme

## 4. Pre-Submission Checklist
- Categorized tasks with status, owner, deadline
- Statistical validation items
- Reproducibility requirements
- Writing quality items

## 5. Timeline
- Revision cycle
- Day-by-day breakdown
- Milestone checkpoints

## 6. Quality Gates
- Gate criteria with thresholds
- Go/no-go decision points
```

### 2. Run Expert Review Simulation

For each expert, generate a review containing:
- Overall assessment and score (1-4)
- Major issues (blocking, with priority)
- Minor issues
- Specific recommendations
- Questions for authors

### 3. Implement Revisions

- Address all major issues
- Update paper sections
- Add missing experiments
- Strengthen weak areas

### 4. Re-run Review Simulation

- Generate Round 2 reviews
- Verify improvements
- Identify remaining issues
- Repeat until target score achieved

### 5. Verify Submission Readiness

- All quality gates passed
- Average score >= 2.5/4
- No reviewer below 2/4
- Reproducibility artifacts ready

---

## Paper-Wave Mapping

Papers in this project are derived from development waves:

| Wave | Focus | Paper Type | Recommended Panel |
|------|-------|-----------|-------------------|
| 01 | Core Algorithm | Algorithms/CS | Karypis, Çatalyürek, Hendrickson, Phillips, Chen |
| 02 | Pipeline Infrastructure | Systems/Software Engineering | Çatalyürek, Goodchild, Yuan, Phillips |
| 03 | Quality/Skills | Software Engineering | (Internal documentation, no paper) |
| 04 | Testing Foundation | Software Engineering | (Internal documentation, no paper) |
| 05 | Pipeline Optimization | Systems/Performance | Çatalyürek, Goodchild, Phillips |
| 06 | Analysis Comparison | Political Science | Chen, Rodden, Duchin, Stephanopoulos, Pildes |
| 07 | Data Architecture | GIS/Systems | Goodchild, Yuan, Çatalyürek |
| 08 | Wave Manager | Software Engineering | (Internal tooling, no paper) |
| 09 | API Migration | Software Engineering | (Infrastructure, no paper) |

**Publishable waves**: 1, 2, 5, 6, 7 (algorithm, data, analysis)
**Internal waves**: 3, 4, 8, 9 (tooling, testing, infrastructure)

---

*Last updated: February 2026*
