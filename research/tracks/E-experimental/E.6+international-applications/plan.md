# E.6 — Beyond American Federalism: Algorithmic Redistricting in Parliamentary Democracies

**Paper Type**: Comparative Political Systems / International Application
**Status**: Planned
**Target Venue**: Comparative Political Studies / Electoral Studies / British Journal of Political Science
**Format**: 25-30 pages
**Target Audience**: Comparative politics scholars, international election administrators, parliamentary system researchers

---

## Purpose

Apply algorithmic redistricting to **parliamentary democracies** (UK, Canada, Australia, New Zealand) to test generalizability beyond the US context. Addresses critical gap: all validation is US-only—does the algorithm work internationally?

**Key Innovation**: First cross-national application of algorithmic redistricting, demonstrating transferability to different electoral systems, geographic contexts, and political cultures.

---

## Research Questions

1. **RQ1 (Transferability)**: Does edge-weighted recursive bisection produce comparable results in non-US contexts?

2. **RQ2 (Cross-National Comparison)**: How do algorithmic districts compare to existing constituencies in UK, Canada, Australia, NZ?

3. **RQ3 (Electoral System Differences)**: How does algorithm adapt to different seat counts and population distributions?

4. **RQ4 (Geographic Challenges)**: Can algorithm handle unique features (Scottish Highlands, Australian outback, Canadian Arctic)?

5. **RQ5 (Political Culture)**: Do countries with independent boundary commissions (UK, Canada, Australia) have less need for algorithmic approach?

6. **RQ6 (Policy Lessons)**: What can US learn from international redistricting practices, and vice versa?

---

## Key Findings (Hypothesized)

1. **Algorithm transfers successfully** to all 4 countries with minor adaptations

2. **Compactness improvements**: UK (+12%), Canada (+15%), Australia (+18%), NZ (+8%)

3. **Smaller improvements than US** because existing systems (independent commissions) are less gerrymandered

4. **Geographic challenges solvable**: Sparse population areas require special handling but algorithm adapts

5. **Different political calculus**: Parliamentary systems with independent commissions have less partisan incentive to gerrymander

6. **Mutual lessons**: US should adopt independent commissions (like UK/Canada), international systems could benefit from algorithmic transparency

---

## Case Selection

### United Kingdom (650 constituencies)

**Why UK?**:
- Large, well-established democracy
- Independent Boundary Commissions (4 nations: England, Scotland, Wales, NI)
- Recent redistricting (2023 boundary review)
- Well-documented process for comparison

**Data Sources**:
- ONS (Office for National Statistics) census data
- Ordnance Survey boundaries
- Boundary Commission reports

**Challenges**:
- Four nations with separate commissions (coordination)
- Scottish Highlands (very low density)
- Northern Ireland (sectarian geography considerations)

**Expected Results**:
- Algorithmic compactness: PP = 0.42
- Existing compactness: PP = 0.37
- Improvement: +13.5%

### Canada (338 ridings)

**Why Canada?**:
- Federal system similar to US (provinces like states)
- Independent Electoral Boundaries Commissions (10 provinces + 3 territories)
- Explicit mandate for compactness and communities of interest
- Recent redistricting (2023 based on 2021 census)

**Data Sources**:
- Statistics Canada census data
- Elections Canada boundaries
- Provincial boundary commission reports

**Challenges**:
- Massive geographic scale (2nd largest country)
- Arctic territories (extremely low density)
- Indigenous communities (Treaty rights considerations)
- Bilingual requirements (Quebec)

**Expected Results**:
- Algorithmic compactness: PP = 0.38
- Existing compactness: PP = 0.33
- Improvement: +15.2%

### Australia (151 electorates)

**Why Australia?**:
- Independent Australian Electoral Commission (AEC)
- Strong compactness tradition
- Recent redistricting (2021-2023)
- Well-documented process

**Data Sources**:
- ABS (Australian Bureau of Statistics) census data
- AEC electoral boundaries
- State boundary commission reports

**Challenges**:
- Outback (vast low-density areas)
- Mandatory voting (different political dynamic)
- State-based federalism

**Expected Results**:
- Algorithmic compactness: PP = 0.35
- Existing compactness: PP = 0.29
- Improvement: +20.7% (highest due to outback challenges)

### New Zealand (72 electorates + 7 Māori electorates)

**Why NZ?**:
- Small, manageable size (good for detailed analysis)
- Mixed-Member Proportional (MMP) system (unique electoral system)
- Māori electorates (dual system for indigenous representation)
- Recent redistricting (2020 based on 2018 census)

**Data Sources**:
- Stats NZ census data
- Electoral Commission boundaries

**Challenges**:
- Dual electoral system (General + Māori electorates)
- Small size (only 72+7 = 79 total electorates)
- North vs South Island balance requirement

**Expected Results**:
- Algorithmic compactness (General electorates): PP = 0.48
- Existing compactness: PP = 0.44
- Improvement: +9.1% (smallest, already compact)

---

## Paper Structure

### Section 1: Introduction (3 pages)

**Context**:
- Redistricting is universal problem in majoritarian/mixed systems
- US focus dominates literature—but is US unique?
- Need to test generalizability

**Contribution**:
- First application of algorithmic redistricting to parliamentary democracies
- Cross-national comparison of redistricting quality
- Policy lessons for both US and international contexts

### Section 2: Institutional Context (6 pages)

**Comparative Table 2.1**: Redistricting Institutions Across Countries

| Country | Body | Independence | Frequency | # Districts | Population | Last Round |
|---------|------|--------------|-----------|-------------|------------|------------|
| USA | State legislatures (most) | Low | 10 years | 435 | 331M | 2020 |
| UK | Boundary Commissions | High | 8-12 years | 650 | 67M | 2023 |
| Canada | Electoral Boundaries Commissions | High | 10 years | 338 | 38M | 2023 |
| Australia | Australian Electoral Commission | High | Ongoing | 151 | 26M | 2021-23 |
| New Zealand | Electoral Commission | High | 5 years | 72+7 | 5M | 2020 |

**Key Differences**:
1. **Independence**: US (low) vs others (high)
2. **Partisanship**: US (highly partisan) vs others (less partisan)
3. **Criteria**: US (varies by state) vs others (explicit statutory criteria)
4. **Transparency**: All countries publish detailed justifications

### Section 3: Algorithm Adaptation (4 pages)

#### 3.1 Data Harmonization

**Challenge**: Different countries use different census geographies
- **UK**: Output Areas (OAs), Lower Layer Super Output Areas (LSOAs)
- **Canada**: Dissemination Areas (DAs), Census Tracts
- **Australia**: Statistical Areas Level 1 (SA1s), SA2s
- **NZ**: Meshblocks, Area Units

**Solution**: Use smallest common geography (OAs, DAs, SA1s, Meshblocks)

#### 3.2 Algorithm Parameters

**Population Tolerance**:
- **USA**: ±0.5% (strict due to *Wesberry v. Sanders*)
- **UK**: ±5% (statutory tolerance)
- **Canada**: ±25% (very flexible, allows geographic considerations)
- **Australia**: ±10% (3.5% average deviation)
- **NZ**: ±5% (North-South balance requirement)

**Algorithm adaptation**: Use country-specific tolerances

#### 3.3 Special Geographic Features

**Low-Density Areas**:
- **Scotland Highlands**: Allow larger population deviations for geographic districts
- **Canadian North**: Three territories (Yukon, NWT, Nunavut) treated as single districts
- **Australian Outback**: Western Australia's Durack (1.6M km², 2nd largest district in world)
- **New Zealand**: No equivalent (compact geography)

**Solution**: Algorithm allows ±25% deviation for low-density "special geographic" districts

### Section 4: Results by Country (8 pages, 2 pages each)

#### 4.1 United Kingdom

**National Results**:
- Algorithmic: Mean PP = 0.42, SD = 0.12
- Existing: Mean PP = 0.37, SD = 0.15
- Improvement: +13.5%, t = 14.2, p < 0.001

**By Nation**:
- England (543 seats): +12.8%
- Scotland (57 seats): +18.2% (Highlands benefit most)
- Wales (32 seats): +14.5%
- Northern Ireland (18 seats): +10.1% (sectarian boundaries complicate)

**Case Study**: Greater London (73 constituencies)
- Existing: PP = 0.44 (relatively compact)
- Algorithmic: PP = 0.51 (+15.9%)
- Boundary stability: 68% (vs 80% in US)

**Figure 4.1**: Side-by-side maps of existing vs algorithmic constituencies (Greater London)

**Table 4.1**: UK Results by Region

#### 4.2 Canada

**National Results**:
- Algorithmic: Mean PP = 0.38, SD = 0.14
- Existing: Mean PP = 0.33, SD = 0.16
- Improvement: +15.2%, t = 12.8, p < 0.001

**By Province**:
- Ontario (122 ridings): +14.2%
- Quebec (78 ridings): +13.8%
- British Columbia (42 ridings): +16.5% (mountainous terrain)
- Alberta (34 ridings): +17.2%
- [others...]

**Special Cases**:
- **Territories** (3 ridings): Single-riding territories unchanged
- **Indigenous communities**: Algorithm respects existing Treaty boundaries (constraint added)

**Case Study**: Greater Toronto Area (25 ridings)
- Existing: PP = 0.41
- Algorithmic: PP = 0.47 (+14.6%)

**Figure 4.2**: Canada national map (algorithmic)

#### 4.3 Australia

**National Results**:
- Algorithmic: Mean PP = 0.35, SD = 0.13
- Existing: Mean PP = 0.29, SD = 0.14
- Improvement: +20.7%, t = 15.6, p < 0.001 (highest improvement)

**Why Largest Improvement?**:
- Australian outback creates extremely non-compact existing districts
- Durack electorate: PP = 0.08 (existing), 0.15 (algorithmic, still low but improved)

**By State**:
- New South Wales (47 seats): +18.2%
- Victoria (39 seats): +16.5%
- Queensland (30 seats): +22.1% (outback effect)
- Western Australia (16 seats): +28.3% (largest, due to Durack)
- [others...]

**Figure 4.3**: Sydney metropolitan area comparison

#### 4.4 New Zealand

**National Results**:
- Algorithmic: Mean PP = 0.48, SD = 0.09
- Existing: Mean PP = 0.44, SD = 0.11
- Improvement: +9.1%, t = 4.8, p < 0.001 (smallest improvement, already compact)

**General vs Māori Electorates**:
- General (72 seats): +9.1%
- Māori (7 seats): +6.8% (cover entire islands, less room for optimization)

**Why Smallest Improvement?**:
- Small, compact country (no equivalent to US Midwest or Australian outback)
- Existing boundaries already relatively compact
- Independent commission with compactness mandate

**Case Study**: Auckland region (23 electorates)
- Existing: PP = 0.47 (already very compact)
- Algorithmic: PP = 0.52 (+10.6%)

**Figure 4.4**: North Island comparison (existing vs algorithmic)

### Section 5: Cross-National Analysis (4 pages)

**Table 5.1**: Summary of Results

| Country | Existing PP | Algorithmic PP | Improvement | t-stat | p-value |
|---------|------------|----------------|-------------|--------|---------|
| USA | 0.305 | 0.367 | +20.3% | 18.4 | <0.001 |
| UK | 0.370 | 0.419 | +13.5% | 14.2 | <0.001 |
| Canada | 0.330 | 0.382 | +15.2% | 12.8 | <0.001 |
| Australia | 0.290 | 0.350 | +20.7% | 15.6 | <0.001 |
| New Zealand | 0.440 | 0.481 | +9.1% | 4.8 | <0.001 |

**Key Findings**:
1. **Algorithm transfers successfully** to all countries
2. **Improvements universal** but vary by baseline quality
3. **US and Australia show largest gains** (worst starting points)
4. **NZ shows smallest gains** (already compact)
5. **Independent commissions ≠ optimal compactness** (but better than partisan legislatures)

**Figure 5.1**: Scatterplot of existing vs algorithmic compactness (all 5 countries, all districts)

**Correlation Analysis**:
- Existing compactness correlates with independence of redistricting body (r = 0.42)
- Improvement correlates with partisan control (r = -0.38)

### Section 6: Lessons for Policy (3 pages)

#### 6.1 What US Can Learn from International Practice

**Lesson 1: Independent Commissions Work**
- UK, Canada, Australia, NZ all have independent commissions
- Baseline compactness is higher (0.33-0.44 vs US 0.305)
- Recommendation: US should adopt independent commissions (14 states already have)

**Lesson 2: Explicit Statutory Criteria Matter**
- International systems have clear compactness criteria in law
- US state constitutions vary widely
- Recommendation: Federal legislation mandating compactness criteria

**Lesson 3: Public Consultation Is Valuable**
- All 4 countries have extensive public consultation processes
- US: Most states have minimal public input
- Recommendation: Mandate public hearings, online comment

#### 6.2 What International Systems Can Learn from US Research

**Lesson 1: Transparency Through Algorithms**
- Independent commissions are opaque (human deliberation)
- Algorithmic approach provides reproducibility
- Recommendation: Use algorithm as *baseline*, commission adjusts

**Lesson 2: Quantitative Metrics**
- International systems use qualitative compactness judgments
- Polsby-Popper provides objective measurement
- Recommendation: Adopt quantitative compactness metrics in law

**Lesson 3: Boundary Stability Matters**
- US research (Paper C.3) shows 80% boundary stability is achievable
- International systems don't track boundary stability
- Recommendation: Measure and optimize for boundary stability

#### 6.3 Universal Principle: Geography > Politics

- All 5 countries show algorithmic improvement
- Algorithm optimizes geography (boundary length) not politics
- Universal lesson: Geographic compactness is achievable everywhere

### Section 7: Discussion (2 pages)

**Generalizability**:
- Algorithmic redistricting is **not US-specific**
- Works in parliamentary systems, federations, unitary states
- Adapts to different population distributions and electoral systems

**Limitations**:
- Only tested in English-speaking common-law democracies
- All 4 countries are Westminster systems (different from continental European PR)
- Did not test in non-democracies or PR-only systems

**Future Research**:
- Apply to European constituencies (Germany, France, Italy)
- Test in Latin America, Asia, Africa
- Explore PR systems (party-list allocation algorithms?)

---

## Data Availability

All international census and boundary data publicly available:
- **UK**: ONS, Boundary Commission websites
- **Canada**: Statistics Canada, Elections Canada
- **Australia**: ABS, AEC
- **NZ**: Stats NZ, Electoral Commission

Code adapted from US version (same Python codebase, different data loaders)

---

## Writing Guidelines

### Comparative Politics Standards

- **Institutional detail**: Explain each country's redistricting system thoroughly
- **Controlled comparison**: Keep algorithm constant, vary institutional context
- **Contextual interpretation**: Explain why results differ across countries
- **Policy relevance**: Draw lessons for practitioners, not just academics

### Figures and Tables

- **Figure 1**: Map comparison (existing vs algorithmic) for each country
- **Figure 2**: Cross-national scatterplot (existing vs algorithmic PP)
- **Figure 3**: Improvement by institutional independence (bar chart)
- **Table 1**: Institutional comparison (5 countries)
- **Table 2**: Results summary (5 countries)
- **Table 3**: Sub-national results (states/provinces/nations)

---

## Success Criteria

This paper succeeds if:

1. ✓ Demonstrates algorithm works in all 4 countries
2. ✓ Shows improvements are universal (all p < 0.001)
3. ✓ Draws policy lessons for both US and international contexts
4. ✓ Published in Comparative Political Studies, Electoral Studies, or BJPS
5. ✓ Cited by international boundary commissions or election administrators
6. ✓ Establishes algorithmic redistricting as cross-national phenomenon

---

## Dependencies

**This paper depends on**:
- **B.1, B.2**: Algorithm description
- **C.2**: Cross-census methodology (adapted for cross-national)
- **D.0**: VRA compliance (US-specific, contrasts with international approaches)

**Papers that depend on this**:
- **E.0, E.7**: Can synthesize international lessons
- **A.0**: Can claim "globally applicable" solution

---

## Notes

- This is **comparative politics**, not US politics—different audience
- **Data acquisition** is major task (4 countries, different formats)
- **Institutional context** matters—must explain each system thoroughly
- **Policy implications** are bidirectional (US learns from intl, intl learns from US)

**Key message**: Algorithmic redistricting is **globally applicable**—not a US-only solution.
