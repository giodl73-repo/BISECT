# C.6 — Public Perceptions of Algorithmic Redistricting: A Survey Experiment

**Paper Type**: Survey Experimental Study
**Status**: Planned
**Target Venue**: American Journal of Political Science (AJPS) / Public Opinion Quarterly / Political Behavior
**Format**: 25-30 pages + supplementary materials
**Target Audience**: Political scientists, public opinion researchers, policymakers

---

## Purpose

Conduct a **large-scale survey experiment** to measure public perceptions of algorithmic vs human-drawn redistricting maps. This addresses the critical gap: all validation has been technical/statistical, but do ordinary citizens perceive algorithmic maps as fairer?

**Key Innovation**: First systematic survey evidence on public acceptance of algorithmic redistricting, testing perceptions across partisan lines and geographic contexts.

---

## Research Questions

1. **RQ1 (Perceived Fairness)**: Do respondents perceive algorithmic maps as fairer than enacted maps when shown side-by-side?

2. **RQ2 (Partisan Differences)**: Do perceptions vary by respondent partisanship (Democrats vs Republicans vs Independents)?

3. **RQ3 (Geographic Context)**: Do perceptions differ in gerrymandered vs non-gerrymandered states?

4. **RQ4 (Transparency Effects)**: Does explaining how the algorithm works increase perceived fairness?

5. **RQ5 (Trust in Process)**: Do respondents trust algorithmic vs legislative vs commission redistricting processes?

6. **RQ6 (Gerrymandering Detection)**: Can ordinary citizens identify gerrymandered districts without being told?

---

## Key Hypotheses

**H1**: Respondents will rate algorithmic maps as fairer than enacted maps (main effect)

**H2**: Effect will be stronger among Independents than strong partisans (partisanship moderates)

**H3**: Effect will be stronger in gerrymandered states (IL, MD, NC) than compact states (IA, CO)

**H4**: Transparency treatment (explaining algorithm) will increase perceived fairness by 10-15 percentage points

**H5**: Trust in algorithmic process will exceed trust in legislative process but be lower than independent commissions

**H6**: Respondents can identify gerrymandered districts at above-chance levels (>50%) when shown visually

---

## Study Design

### Overview

**Design**: 2×2×2 factorial survey experiment with representative national sample

**Factors**:
1. **Map Type**: Algorithmic vs Enacted (between-subjects)
2. **Transparency**: Explanation vs No explanation (between-subjects)
3. **State Context**: Gerrymandered vs Non-gerrymandered (within-subjects, 2 states each)

**Sample**: N=2,400 (300 per condition × 8 conditions)

**Platform**: Lucid Theorem or Prolific (high-quality online panels with quota sampling)

**Duration**: 15-20 minutes per respondent

**Incentive**: $4.50 per completion (standard rate)

### Experimental Conditions

**Condition 1 (Control)**: Shown enacted map, no explanation
**Condition 2**: Shown enacted map, with transparency explanation
**Condition 3**: Shown algorithmic map, no explanation
**Condition 4**: Shown algorithmic map, with transparency explanation

**Within-subjects variation**: All respondents see 4 states (2 gerrymandered, 2 non-gerrymandered)

**Gerrymandered states**: Illinois, Maryland, North Carolina, Louisiana (randomly select 2)
**Non-gerrymandered states**: Iowa, Colorado, Washington, Michigan (randomly select 2)

### Sample Composition

**Quotas** (to match US adult population):
- 50% female, 50% male
- Age: 18-29 (22%), 30-44 (25%), 45-64 (34%), 65+ (19%)
- Race: White (63%), Black (12%), Hispanic (17%), Asian (6%), Other (2%)
- Party ID: Democrat (32%), Republican (30%), Independent (30%), Other (8%)
- Education: < High school (10%), High school (28%), Some college (30%), College+ (32%)
- Region: Northeast (18%), Midwest (21%), South (38%), West (23%)

**Power analysis**: N=2,400 detects effect size d=0.18 with 80% power, α=0.05

---

## Survey Instrument

### Part 1: Pre-Treatment (5 minutes)

**Demographics**:
- Age, gender, race/ethnicity, education, income
- State of residence
- Urban/suburban/rural classification

**Political Variables**:
- Party identification (7-point scale: Strong D to Strong R)
- Political interest (4-point scale)
- 2020 presidential vote (Biden/Trump/Other/Didn't vote)
- Political knowledge (3 questions: Speaker of House, # of Senators, gerrymandering definition)

**Baseline Attitudes**:
- Trust in government (5-point scale)
- Perceived fairness of current redistricting in your state (0-100 scale)
- Support for redistricting reform (5-point scale)

### Part 2: Treatment (8 minutes)

**Transparency Manipulation** (Conditions 2 and 4 only):

*"Algorithmic redistricting uses computer programs to draw district boundaries automatically based on objective criteria like population equality and geographic compactness. The algorithm uses census data and geographic boundaries but does NOT use any information about voters' political party, race, or voting history. The algorithm is open-source, meaning anyone can inspect the code and verify the results. The same algorithm applied to the same data always produces the same districts."*

**Map Presentation** (4 states, order randomized):

For each state, show:
- Map of congressional districts (colored by district, labeled with numbers)
- State name and district count
- No indication of whether algorithmic or enacted

**Visual Design**:
- High-resolution maps (1200×800 pixels)
- Consistent color palette across conditions
- District labels clearly visible
- Alaska/Hawaii insets for national context

### Part 3: Outcome Measures (5 minutes)

**For each state** (asked after viewing map):

**Perceived Fairness** (primary outcome):
*"How fair do you think these district boundaries are?"*
- Scale: 0 (Extremely unfair) to 100 (Extremely fair)

**Perceived Compactness**:
*"How compact are these districts? Compact districts have simple shapes and short boundaries."*
- Scale: 0 (Not at all compact) to 100 (Extremely compact)

**Suspected Gerrymandering**:
*"Do you think these districts were drawn to favor one political party?"*
- Options: Yes, to favor Democrats / Yes, to favor Republicans / No, they seem fair / Not sure

**Trust in Process**:
*"How much would you trust a redistricting process that produced these districts?"*
- Scale: 0 (No trust at all) to 100 (Complete trust)

**After all 4 states**:

**Process Preference**:
*"Who should draw congressional district boundaries?"*
- Options: State legislature / Independent commission / Computer algorithm / Courts / Federal government / Not sure

**Willingness to Support**:
*"Would you support your state using computer algorithms to draw congressional districts?"*
- Scale: 1 (Strongly oppose) to 7 (Strongly support)

**Open-Ended**:
*"In your own words, what makes congressional districts fair or unfair?"*
- Text box (optional, max 500 characters)

### Part 4: Manipulation Check & Debriefing (2 minutes)

**Manipulation checks**:
- *"Did you receive an explanation of how algorithmic redistricting works?"* (Yes/No)
- *"Were the maps you saw drawn by humans or computers?"* (Humans/Computers/Not sure)

**Attention checks** (embedded throughout):
- *"Please select 'Strongly agree' for this question"* (instructional manipulation check)
- *"What state did you just view a map for?"* (memory check)

**Debriefing**:
- Explain study purpose
- Reveal which maps were algorithmic vs enacted
- Provide resources on redistricting reform

---

## Analysis Plan

### Primary Analyses

**Analysis 1: Main Effect of Map Type (H1)**
- DV: Perceived fairness (0-100)
- IV: Map type (Algorithmic vs Enacted)
- Method: Linear mixed-effects model with random intercepts for respondents (repeated measures)
- Controls: State fixed effects, respondent demographics
- Expected result: β = +8-12 points for algorithmic maps, p < 0.001

**Analysis 2: Partisan Moderation (H2)**
- Interaction: Map type × Partisan strength (7-point scale)
- Method: Linear mixed-effects model with interaction term
- Expected result: Effect stronger for independents (β = +15) than strong partisans (β = +5)

**Analysis 3: Geographic Context (H3)**
- Interaction: Map type × State gerrymandering status
- Method: Linear mixed-effects model with three-way interaction
- Expected result: Larger treatment effect in gerrymandered states (β = +18) vs non-gerrymandered (β = +6)

**Analysis 4: Transparency Effect (H4)**
- Interaction: Map type × Transparency treatment
- Method: Between-subjects ANOVA
- Expected result: Transparency increases algorithmic fairness perceptions by 10-15 points

**Analysis 5: Process Trust (H5)**
- DV: Trust in redistricting process (0-100)
- IV: Map type (Algorithmic vs Enacted)
- Method: Same as Analysis 1
- Expected result: Algorithmic trust > Legislative trust, but < Commission trust (from separate question)

**Analysis 6: Gerrymandering Detection (H6)**
- DV: Correctly identified gerrymandered maps (binary)
- Method: Logistic regression
- Expected result: Accuracy > 50% for obvious gerrymanders (IL, MD), near-chance for compact states

### Secondary Analyses

**Heterogeneous Effects**:
- By education (college vs non-college)
- By political knowledge (high vs low)
- By age (<45 vs 45+)
- By urban/rural residence

**Mediation Analysis**:
- Test whether perceived compactness mediates fairness judgments
- Method: Causal mediation analysis (Imai et al.)

**Qualitative Coding**:
- Code open-ended responses for themes (fairness criteria, concerns about algorithms, etc.)
- Report top 10 themes with example quotes

---

## Expected Results

### Table 1: Treatment Effects on Perceived Fairness

| Condition | Mean Fairness (0-100) | SE | N |
|-----------|----------------------|-----|---|
| Enacted, No Explanation | 42.3 | 1.2 | 600 |
| Enacted, With Explanation | 43.8 | 1.2 | 600 |
| Algorithmic, No Explanation | 54.7 | 1.2 | 600 |
| Algorithmic, With Explanation | 63.2 | 1.1 | 600 |

**Key findings**:
- Main effect of algorithmic: +12.4 points (p < 0.001)
- Transparency effect for algorithmic: +8.5 points (p < 0.001)
- No transparency effect for enacted: +1.5 points (p = 0.34)

### Figure 1: Partisan Differences in Perceived Fairness

**Expected pattern**:
- Strong Democrats: Enacted 38, Algorithmic 48 (+10 points)
- Independents: Enacted 42, Algorithmic 60 (+18 points)
- Strong Republicans: Enacted 40, Algorithmic 52 (+12 points)

**Interpretation**: Independents are most responsive to algorithmic fairness

### Figure 2: Effects by State Gerrymandering Status

**Expected pattern**:
- Illinois (gerrymandered): Enacted 28, Algorithmic 52 (+24 points)
- Iowa (non-gerrymandered): Enacted 58, Algorithmic 64 (+6 points)

**Interpretation**: Largest gains where enacted maps are worst

---

## Discussion

### Substantive Contributions

1. **First survey evidence** on public perceptions of algorithmic redistricting
2. **Algorithmic maps are perceived as fairer** even by partisan respondents
3. **Transparency matters**: Explaining the algorithm increases trust
4. **Context-dependent**: Biggest effects in gerrymandered states
5. **Partisan symmetry**: Both Democrats and Republicans prefer algorithmic maps to status quo

### Practical Implications

1. **Public support exists**: Majority of respondents support algorithmic redistricting
2. **Education is key**: Transparency treatment significantly boosts acceptance
3. **Bipartisan appeal**: Not a partisan issue—both parties benefit from fairness
4. **Replaces intuition with data**: Citizens can *perceive* fairness, validating technical metrics

### Limitations

1. **Hypothetical maps**: Respondents don't live in these districts, may not care deeply
2. **Online sample**: May not fully represent offline population (though quotas help)
3. **Single exposure**: Real voters would experience districts over years, not minutes
4. **Limited context**: Didn't show election outcomes, demographic composition
5. **Desirability bias**: Respondents may say "fair" because it sounds good

### Future Research

1. **Longitudinal study**: Track perceptions over time as algorithm is debated
2. **Elite cues**: How do partisan endorsements affect support?
3. **Mechanism testing**: What exactly makes algorithmic maps seem fairer? (compactness, transparency, neutrality)
4. **Comparative**: Test algorithmic vs independent commission maps

---

## IRB and Ethics

### Human Subjects Protection

**IRB Approval**: Required before data collection
**Risk Level**: Minimal risk (survey about political perceptions)
**Informed Consent**: Obtained before survey begins
**Debriefing**: Explain study purpose, reveal deception (map types not labeled)
**Data Privacy**: No personally identifiable information collected, IP addresses anonymized

### Preregistration

**Platform**: OSF (Open Science Framework) or AsPredicted
**Timing**: Before data collection begins
**Contents**: Hypotheses, sample size, analysis plan, stopping rules

**Commitment**: Preregister to avoid HARKing (Hypothesizing After Results are Known)

---

## Budget

| Item | Cost |
|------|------|
| Respondent incentives (2,400 × $4.50) | $10,800 |
| Platform fees (Lucid Theorem, 30% markup) | $3,240 |
| Pilot testing (N=100) | $540 |
| Graphic design (professional maps) | $2,000 |
| IRB application fees | $500 |
| Research assistant (data cleaning, coding) | $2,000 |
| **Total** | **$19,080** |

**Funding sources**: NSF, Russell Sage Foundation, Arnold Ventures, Democracy Fund

---

## Timeline

| Phase | Duration | Activities |
|-------|----------|----------|
| Design | 2 weeks | Finalize instrument, create maps |
| IRB | 4 weeks | Submit protocol, revisions, approval |
| Preregistration | 1 week | Write and post preregistration |
| Pilot | 1 week | N=100 pilot, check manipulations |
| Main study | 2 weeks | Full data collection (N=2,400) |
| Cleaning | 1 week | Data cleaning, attention checks |
| Analysis | 3 weeks | Run models, create figures/tables |
| Writing | 4 weeks | Draft paper |
| Revision | 2 weeks | Internal feedback, revisions |
| **Total** | **20 weeks** | **~5 months** |

---

## Writing Guidelines

### Survey Experiment Standards

- **CONSORT diagram**: Flow chart of participant exclusions
- **Balance table**: Show randomization worked (demographics by condition)
- **Manipulation checks**: Report success rates (expect >90%)
- **Robustness checks**: Alternative specifications, subgroup analyses
- **Preregistration transparency**: Note any deviations from preregistered plan

### Figures and Tables

- **Figure 1**: Coefficient plot showing treatment effects by subgroup
- **Figure 2**: Interaction plots (Map type × Partisanship, Map type × State context)
- **Figure 3**: Distribution of fairness ratings (histogram by condition)
- **Table 1**: Descriptive statistics and balance check
- **Table 2**: Main regression results (4-5 models)
- **Table 3**: Heterogeneous effects by demographics

---

## Success Criteria

This paper succeeds if:

1. ✓ Finds significant positive effect of algorithmic maps on perceived fairness (p < 0.001, d > 0.30)
2. ✓ Effect holds across partisan lines (no interaction with partisanship or small interaction)
3. ✓ Transparency treatment significantly boosts acceptance (+10 points)
4. ✓ Published in AJPS, POQ, or Political Behavior (top public opinion journals)
5. ✓ Cited as evidence of "public support" for algorithmic redistricting in policy debates
6. ✓ Informs advocacy campaigns (FairVote, Common Cause use findings)

---

## Dependencies

**This paper depends on**:
- **B.1, B.2**: Algorithmic maps to show respondents
- **Existing enacted maps**: For comparison condition
- **A.3 (visualization)**: Map design standards

**Papers that depend on this**:
- **A.0 (synthesis)**: Can claim "public support" for algorithmic approach
- **A.5 (policy brief)**: Can cite survey evidence in advocacy materials

---

## Notes

- This is **not a technical validation**—it's about *perceptions* and *political feasibility*
- **Public opinion matters** for adoption—even perfect algorithm fails if public doesn't trust it
- **Transparency treatment** is key intervention—education increases acceptance
- **Bipartisan findings** are critical for policy—both parties must see benefits
- **Survey experiment** is gold standard for causal inference on public opinion

**Key message**: Algorithmic redistricting isn't just technically superior—**the public perceives it as fairer**.
