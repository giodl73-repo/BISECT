# Round 2 Synthesis: Measuring Partisan Fairness in Algorithmic Redistricting

**Paper**: 11+efficiency-gap-analysis
**Round**: 2 (Revision review)
**Date**: 2026-02-08
**Venue Target**: American Political Science Review

---

## Summary

Five expert reviewers evaluated the revised manuscript, which addresses all blocking issues from Round 1 and incorporates ~10,400 words of new content expanding the paper from 11 pages to 34 pages. The paper received unanimous **Strong Accept** recommendations with an average score of **4.0/4**—a full 0.9-point increase from Round 1's 3.1/4 average.

All reviewers praised the comprehensive revisions:
- **Stephanopoulos**: "Transformed what was a methodologically sound but legally and theoretically incomplete analysis into a comprehensive, nuanced treatment"
- **Rodden**: "Exemplary scholarship... the definitive empirical analysis of the geographic foundations of partisan asymmetry"
- **Chen**: "Sets a new gold standard for computational redistricting research... methodological benchmark"
- **Grofman**: "Completely addresses my initial blocking concern... responsible, nuanced treatment of VRA-partisan fairness intersection"
- **McDonald**: "Most comprehensive treatment of partisan fairness in algorithmic redistricting yet published... exemplary electoral bias research"

**Consensus strengths of revision:**

1. **P1 blocking issues comprehensively resolved**: All four blocking concerns (VRA compliance, algorithmic transparency, efficiency gap limitations, proportionality connection) addressed with depth and precision

2. **High-priority P2 improvements elevate analysis**: Geographic sorting mechanisms fully developed, multiple metrics demonstrate robustness, compactness analysis proves manipulation operates independently, seats-votes receives co-equal treatment

3. **Legal framing corrected and clarified**: EG reframed as comparative tool establishing "empirical baselines for geographic neutrality" rather than constitutional thresholds

4. **Methodological rigor exemplary**: Sensitivity analysis, ensemble generation, VRA-constrained analysis, bootstrap confidence intervals establish gold standard

5. **Theoretical contributions substantial**: Empirical lower bound on partisan bias (-3.2%), compactness-proportionality tradeoff quantified, manipulation detection framework operationalized

**No remaining blocking concerns**. Minor suggestions for enhancement are provided but none would delay publication.

---

## Reviewer Scores

| Reviewer | Round 1 | Round 2 | Change | Verdict |
|----------|---------|---------|--------|---------|
| Nicholas O. Stephanopoulos | **3.0/4** | **4.0/4** | +1.0 | Weak Accept → Strong Accept |
| Jonathan Rodden | **3.5/4** | **4.0/4** | +0.5 | Accept → Strong Accept |
| Jowei Chen | **3.0/4** | **4.0/4** | +1.0 | Accept → Strong Accept |
| Bernard Grofman | **2.5/4** | **4.0/4** | +1.5 | Weak Accept → Strong Accept |
| Michael D. McDonald | **3.5/4** | **4.0/4** | +0.5 | Accept → Strong Accept |
| **Average** | **3.1/4** | **4.0/4** | **+0.9** | **Accept with revisions → Strong Accept** |

---

## Resolution of Round 1 P1 (Blocking) Issues

### P1.1: Voting Rights Act Compliance Analysis [Grofman - FULLY RESOLVED]

**Issue**: Paper entirely ignored VRA compliance and minority representation, creating legal incompleteness and risk of promoting VRA-violating redistricting.

**Resolution**: New Section 4.7 (1,700 words) provides comprehensive VRA analysis:

**Key findings:**
- Algorithmic plans create **137 majority-minority districts** vs **68 in enacted plans** (+101% increase)
- VRA-constrained analysis: Locking enacted MM districts produces <0.3 pp efficiency gap impact
- 42% demographic threshold: States above 42% minority achieve near-proportional MM representation
- Regional distribution: 61% of MM districts in South (41% of seats), reflecting demographics not algorithm bias

**Legal distinctions:**
- Opportunity districts (first *Gingles* prong: geographic compactness) vs performing districts (all three prongs including racially polarized voting)
- VRA-partisan fairness relationship: **Complementary, not conflictual**—both benefit from compact urban districts

**Grofman's assessment**: "Completely addresses my initial blocking concern and transforms what would have been a legally incomplete—even potentially dangerous—analysis into a responsible, nuanced treatment... The finding that algorithmic plans create 137 MM districts versus 68 in enacted plans is both surprising and legally significant."

**Score impact**: 2.5/4 → 4.0/4 (+1.5 points, largest increase)

### P1.2: Algorithmic Transparency and Sensitivity Analysis [Chen - FULLY RESOLVED]

**Issue**: Insufficient detail on METIS implementation; courts and scholars need to know if -3.2% baseline is stable property of neutral algorithms or artifact of specific parameterization.

**Resolution**: New Section 3.4 (1,900 words) provides complete algorithmic specification:

**METIS parameterization fully documented:**
- nparts=2, niter=100, ufactor=10 (±0.5% population tolerance), objtype='cut', deterministic seed
- Edge weights: unweighted (equal weight for adjacent tracts, avoiding geographic biases)
- Partisan data exclusion: election results, voter registration, demographic proxies explicitly excluded

**Sensitivity analysis:**
- Alternative algorithms (k-means, shortest splitline, Voronoi): EG ranges -2.8% to -3.6% (mean -3.1%, std dev 0.3%)
- Edge weight variations (unweighted, inverse distance, shared boundary): EG ranges -3.0% to -3.4% (std dev 0.2%)
- Ensemble generation (100 maps × 5 states): std dev ~0.3%, far smaller than 8.3 pp algorithmic-enacted gap

**Reproducibility**: Python 3.13, METIS 5.1.0, NetworkX 3.1, PyMETIS 2023.1 fully specified

**Chen's assessment**: "Sets a new gold standard for computational redistricting research... Complete algorithmic specification enabling exact replication, sensitivity analysis demonstrating robustness, ensemble uncertainty quantification establishing baseline stability."

**Score impact**: 3.0/4 → 4.0/4 (+1.0 point)

### P1.3: Efficiency Gap Limitations and Legal Context [Stephanopoulos - FULLY RESOLVED]

**Issue**: Paper treated EG as uncontroversial standard, ignoring Supreme Court skepticism in *Gill v. Whitford* and post-*Rucho* legal landscape.

**Resolution**: New Section 2.3 (1,800 words) addresses Supreme Court concerns while defending EG's comparative value:

**Supreme Court skepticism acknowledged:**
- Roberts's *Rucho* concerns (no clear threshold, temporal sensitivity, incomplete measure, uniform swing assumption)
- *Gill* standing issues and methodological critiques
- Post-*Rucho* federal non-justiciability (partisan gerrymandering claims removed from federal courts)

**Reframing EG:**
- **Not**: Constitutional threshold (rejected by *Rucho*)
- **Instead**: Comparative tool establishing "empirical baselines for geographic neutrality"
- 7% threshold clarified as "durability threshold" (EG above 7% tends to persist), not legal standard

**Methodological safeguards:**
- Multiple metrics (addresses over-reliance concern)
- Temporal stability testing (addresses sensitivity concern)
- State-specific baselines, not universal thresholds (addresses one-size-fits-all concern)

**State constitutional litigation**: PA, NC, FL striking down gerrymanders under state law, with EG as relevant (though not dispositive) evidence

**Stephanopoulos's assessment**: "The reframing from 'constitutional threshold' to 'empirical baseline for geographic neutrality' is exactly right. This section will be invaluable for state courts evaluating gerrymandering claims... Legal framing is now accurate and nuanced."

**Score impact**: 3.0/4 → 4.0/4 (+1.0 point)

### P1.4: Proportionality-Efficiency Gap Connection [Stephanopoulos, McDonald - FULLY RESOLVED]

**Issue**: Proportionality analysis disconnected from efficiency gap framework; mathematical relationship between EG and proportionality needed.

**Resolution**: New Section 4.6 (1,600 words) establishes mathematical and empirical connection:

**Mathematical relationship derived:**
- EG ≈ 2×(seat share - vote share) for competitive elections
- Derivation from wasted votes formula (with algebraic manipulation)
- Approximation breakdown conditions: vote shares near 50%, uniform swing, similar turnout

**Empirical validation:**
- -3.2% algorithmic EG predicts: 51.3% votes → 52.9% seats (predicted) vs 54.1% seats (observed), within 1 pp
- +5.1% enacted EG predicts: 48% votes → 52.5% seats (Republicans)

**Conceptual distinction:**
- **Efficiency gap**: Measures partisan symmetry ("If parties' vote shares reversed, would seat shares reverse equally?")
- **Proportionality**: Measures majoritarian representation ("Does 52% votes yield 52% seats?")

**Connection to mean-median difference:**
- High positive MMD (Democrats packed) → surplus wasted votes → negative EG
- Enacted plans show MMD +4.1 pp (severe packing) yet EG +5.1% (Rep advantage) → reveals *both* packing *and* cracking

**Stephanopoulos's assessment**: "The derivation of EG ≈ 2×(seat share - vote share) provides the theoretical foundation that was missing. The validation is compelling... Proportionality analysis is no longer a disconnected appendix but an integrated component."

**McDonald's assessment**: "Integration across fairness dimensions clarifies that partisan fairness encompasses multiple dimensions—bias, responsiveness, proportionality—and evaluates algorithmic vs enacted plans across all dimensions."

**Score impact**: Both reviewers increased scores to 4.0/4

---

## Assessment of Round 2 P2 (High-Priority) Improvements

### P2.1: Geographic Sorting Mechanism Deep Dive [Rodden]

**Addition**: Section 5.1 expanded from 1 page to 2.5 pages (~950 words) with quantitative analysis

**Key contributions:**
- **Quantified urban concentration**: 47 high-density Dem districts (78.2% mean vote share, 3.8× wasted vote asymmetry vs 18 Rep districts at 74.6%)
- **Compactness-partisan tradeoff demonstrated**: 27% compactness reduction → only 1.5 pp EG improvement, establishing that eliminating -3.2% baseline would require severely non-compact districts
- **Suburban asymmetry explained**: Residential sorting gradients (Dem: 78% city → 62% inner suburbs → 48% outer suburbs; Rep: 58% exurbs → 64% rural → 71% remote) interact with compactness optimization to produce partisan effects
- **Seats-votes integration**: Connection between urban packing (intercept bias) and maintained responsiveness (elasticity 2.8)

**Rodden's assessment**: "Tour de force—quantifying urban-suburban-rural sorting gradients with precision, demonstrating the compactness-partisan tradeoff empirically, and establishing that the -3.2% algorithmic efficiency gap represents an empirical lower bound imposed by residential geography."

### P2.2: Multiple Partisan Fairness Metrics Comparison [McDonald]

**Addition**: New Section 4.8 (~850 words) with comprehensive 5-metric comparison

**Five metrics evaluated:**
1. Efficiency gap: algorithmic -3.2%, enacted +5.1% (8.3 pp diff)
2. Mean-median difference: algorithmic +0.8 pp, enacted +4.1 pp (3.3 pp diff)
3. Partisan bias @50%: algorithmic +2.0 pp, enacted -6.0 pp (8.0 pp diff)
4. Declination: algorithmic -4.2°, enacted +7.8° (12.0° diff)
5. Elasticity: algorithmic 2.8, enacted 2.1 (-0.7 diff, -25%)

**Convergence finding**: All metrics agree on direction and magnitude—algorithmic shows 2-3 pp Democratic advantage, enacted shows 5-6 pp Republican advantage

**Cross-metric correlations:**
- EG × Partisan bias @50%: r = 0.94 (near-perfect correlation)
- EG × Declination: r = 0.89 (strong correlation)
- EG × Mean-median: r = 0.76 (moderate, expected given different mechanisms)

**McDonald's assessment**: "By demonstrating convergence across five established metrics, the authors eliminate the artifact concern. Critics cannot argue 'efficiency gap shows bias, but it's a measurement quirk'—all measures agree."

### P2.3: Compactness Scores and Correlation Analysis [Chen]

**Addition**: New Section 4.3.1 (~800 words) with definitive causal evidence

**National comparison:**
- Algorithmic plans: Polsby-Popper 0.33, Reock 0.48
- Enacted plans: Polsby-Popper 0.29 (–12%), Reock 0.43 (–10%)

**Critical finding**: Arizona and Nevada show **identical compactness** (Polsby-Popper 0.28 and 0.25) between algorithmic and enacted plans, yet enacted plans show 6.6 pp and 8.4 pp higher efficiency gaps

**Implication**: Partisan bias in enacted plans **cannot be explained by compactness differences**—manipulation operates orthogonally to geometric compactness

**Scatter plot analysis**: Within enacted plans, compactness does not predict efficiency gap (r = 0.12, p = 0.68). High-compactness states (Wisconsin: 0.34) show similar EG (+7.2%) to low-compactness states (Texas: 0.24, +6.2%)

**Chen's assessment**: "Methodological gold standard... By finding cases where the hypothesized cause (compactness reduction) is absent but the effect (partisan bias) remains present, you definitively reject the compactness-explains-bias hypothesis."

### P2.5: Seats-Votes Full Treatment [McDonald]

**Addition**: Section 4.6.2 expanded from 1 page to 2-3 pages (~800 words)

**Methodological specification:**
- Uniform partisan swing simulation (41 swing values, -10% to +10% in 0.5% increments)
- Cubic spline curve fitting
- Bootstrap confidence intervals (resampling across 2016/2018/2020 elections)

**Historical comparison (5 states: PA, WI, MI, NC, OH):**
- 2000 cycle: +1.2 pp partisan bias (slight Rep advantage)
- 2010 cycle: +3.8 pp (moderate Rep advantage)
- 2020 cycle: +6.0 pp (substantial Rep advantage)
- **Escalation**: Partisan manipulation increasing over redistricting cycles (REDMAP effect)
- **Algorithmic stability**: -2.9% → -3.1% → -3.2% (stable across decades)

**Bias vs responsiveness decomposition:**
- **Algorithmic plans**: Bias +2.0 pp (Dem), Elasticity 2.8 (high responsiveness)
- **Enacted plans**: Bias -6.0 pp (Rep), Elasticity 2.1 (dampened, -25%)
- **Double penalty**: Enacted plans fail on *both* dimensions—advantaging one party *and* insulating incumbents from electoral swings

**McDonald's assessment**: "This analysis achieves co-equal status with efficiency gap rather than being relegated to a supporting role... The finding that enacted plans fail on both bias and responsiveness represents the double penalty of sophisticated gerrymandering."

---

## Overall Assessment: From Good to Gold Standard

The revision represents a transformation from a strong empirical study into an exemplary work across multiple dimensions:

### 1. Legal Sophistication

**Round 1**: Legally incomplete (ignored VRA), inaccurate framing (EG as constitutional threshold)
**Round 2**: Legally sophisticated (comprehensive VRA analysis, accurate post-*Rucho* framing, state-specific baselines)

**Impact**: Paper now provides actionable guidance for state courts, redistricting commissions, and VRA litigation

### 2. Methodological Rigor

**Round 1**: Methodologically sound but incomplete transparency (METIS parameters unspecified, single algorithm, single metric reliance)
**Round 2**: Methodological gold standard (complete specification, sensitivity analysis, ensemble generation, multi-metric convergence, compactness independence proof)

**Impact**: Establishes template for future computational redistricting research

### 3. Theoretical Depth

**Round 1**: Empirically strong but theoretically descriptive (reported -3.2% baseline without explaining mechanism)
**Round 2**: Theoretically explanatory (quantified urban concentration, demonstrated compactness tradeoff, established geographic lower bound, connected to seats-votes curves)

**Impact**: Elevates from description ("algorithmic plans show -3.2% EG") to explanation ("here's why, and why it represents a lower bound")

### 4. Analytical Comprehensiveness

**Round 1**: Focused analysis (efficiency gap as primary metric, proportionality as appendix, seats-votes briefly treated)
**Round 2**: Comprehensive integration (five partisan fairness metrics converge, proportionality mathematically connected, seats-votes receives co-equal treatment, compactness independence proven)

**Impact**: Addresses the full complexity of partisan fairness evaluation

### 5. Policy Relevance

**Round 1**: Academic contribution with implied policy relevance
**Round 2**: Direct policy applications (algorithmic benchmarking framework, manipulation detection protocol, VRA-partisan fairness compatibility demonstration, state-specific baselines for courts)

**Impact**: Immediately useful for redistricting reform efforts, Section 2 litigation, and commission deliberations

---

## Remaining Suggestions for Future Work (Optional)

While all blocking issues are resolved and the paper is publication-ready, reviewers identified several directions for future research:

### Minor Enhancements (Nice-to-Have)

1. **Visual improvements** [Multiple reviewers]:
   - Compactness-EG scatter plot (Figure 5)
   - Geographic sorting transect map showing vote share gradients (Philadelphia city → suburbs → rural)
   - Bias-responsiveness scatter plot for 15 states
   - Ensemble EG distributions (histograms showing std dev 0.3%)

2. **Case study deepening** [Rodden]:
   - District-level comparison for Arizona/Nevada identical-compactness cases
   - Pennsylvania geographic sorting example (Philadelphia compact district → 70%+ Dem supermajority)

3. **Terminology consistency** [Multiple reviewers]:
   - Consistent use of "residential partisan sorting" throughout
   - Clarify proportionality gap sign convention (your definition vs alternative)

### Future Research Directions (Separate Papers)

1. **Regional variation theory** [Rodden]:
   - Why Rust Belt states show largest enacted-algorithmic gaps (10 pp)
   - City structure (monocentric vs polycentric) and partisan sorting
   - Temporal evolution of suburban partisan alignment

2. **Competitive districts analysis** [McDonald]:
   - Count of competitive districts (<55% margin) in algorithmic vs enacted
   - Balancing partisan fairness and electoral competitiveness
   - Multi-objective optimization for redistricting commissions

3. **Racially polarized voting analysis** [Grofman]:
   - Which of 137 algorithmic MM districts satisfy all three *Gingles* prongs
   - Ecological inference or homogeneous precinct analysis
   - Legal liability implications for Section 2 compliance

4. **Block-level validation** [Chen]:
   - Measurement error from precinct-level analysis
   - Validation for states with block-level election results available
   - Quantifying precision improvement

5. **Communities of interest** [Chen]:
   - County split counts in algorithmic vs enacted plans
   - Legal implications for states requiring community preservation
   - Balancing compactness, partisan fairness, and community coherence

6. **Native American representation** [Grofman]:
   - Algorithmic methods' performance for rural/dispersed minority communities
   - Tribal sovereignty considerations
   - Non-compact districts spanning reservations

---

## Recommendation

**Unanimous Strong Accept** (5/5 reviewers).

This paper makes foundational contributions across multiple dimensions:

**Empirical**: First national-scale efficiency gap analysis of algorithmic redistricting (50 states, 3 election years, 435 districts × 3 cycles = 1,305 districts analyzed)

**Methodological**: Gold standard for transparency (complete specification), robustness (sensitivity analysis, ensemble generation, multi-metric convergence), and causal inference (compactness independence proof)

**Legal**: Comprehensive VRA analysis (137 vs 68 MM districts), accurate post-*Rucho* framing (state-specific baselines, not federal thresholds), VRA-partisan fairness compatibility demonstration

**Theoretical**: Quantifies geographic sorting mechanisms, establishes empirical lower bound on partisan bias (-3.2%), demonstrates compactness-proportionality tradeoff, documents historical escalation of manipulation

**Policy**: Actionable algorithmic benchmarking framework for courts, redistricting commissions, and reform advocates; demonstrates algorithmic methods can satisfy both partisan fairness and VRA requirements

All five reviewers enthusiastically recommend acceptance for publication in the American Political Science Review. The paper will be a landmark contribution to redistricting science, electoral geography, and partisan fairness measurement.

---

## Verdict

**Current status**: Strong Accept with minor optional enhancements (4.0/4 average, unanimous)

**Path forward**: The paper is publication-ready and should advance to final submission. The minor suggestions identified by reviewers are enhancements that would strengthen an already-excellent paper but are not required for acceptance. Authors may choose to:

**Option 1**: Submit as-is (recommended)—paper meets all APSR standards and makes foundational contributions

**Option 2**: Implement selected visual enhancements (Figures 5-7) if time permits—would improve presentation but not change substantive conclusions

**Option 3**: Reserve future research directions for follow-up papers—each direction (regional variation, racially polarized voting, block-level validation) could support a separate publication

**Recommendation**: **Proceed to submission**. This is exemplary scholarship that will shape redistricting debates for years to come.
