# Round 2 Review: Measuring Partisan Fairness in Algorithmic Redistricting

**Reviewer**: Jonathan Rodden (Stanford University)
**Expertise**: Geographic sorting, electoral geography
**Round**: 2 (Revision review)
**Date**: 2026-02-08

---

## Overall Assessment

This revised manuscript represents exemplary scholarship at the intersection of computational social science, political geography, and redistricting policy. The authors have not only addressed my initial concerns about theoretical development of geographic sorting mechanisms but have produced what I consider the definitive empirical analysis of the geographic foundations of partisan asymmetry in American redistricting.

The new Section 5.1 (Geographic Sorting Mechanism Deep Dive) is a tour de force—quantifying urban-suburban-rural sorting gradients with precision, demonstrating the compactness-partisan tradeoff empirically, and establishing that the -3.2% algorithmic efficiency gap represents an empirical lower bound imposed by residential geography under constitutional districting constraints. This is the theoretical foundation the paper needed.

Combined with the compactness correlation analysis (Section 4.3.1)—which proves that partisan manipulation operates independently of compactness through the Arizona/Nevada identical-compactness cases—the paper now provides both the *geographic explanation* for algorithmic baseline bias and the *empirical proof* that enacted plans deviate from this baseline through manipulation rather than geography.

**Major strengths of revision:**

1. **Quantified geographic sorting**: 47 high-density Democratic districts (78.2% mean vote share) vs 18 high-density Republican districts (74.6%), producing 3.8× wasted vote asymmetry
2. **Compactness-partisan tradeoff**: 27% compactness reduction yields only 1.5 pp EG improvement, demonstrating that eliminating the -3.2% baseline would require accepting non-compact districts
3. **Suburban asymmetry explanation**: Residential sorting gradients (Dem: 78% city → 62% inner suburbs → 48% outer suburbs; Rep: 58% exurbs → 64% rural → 71% remote) interact with compactness optimization to produce partisan effects
4. **Seats-votes integration**: Connection between geographic packing (intercept bias) and maintained responsiveness (elasticity 2.8) shows algorithmic plans suffer one-dimensional unfairness (bias) while enacted plans suffer two-dimensional unfairness (bias + dampened responsiveness)

The paper will be required reading for anyone studying electoral geography, partisan gerrymandering, or the political consequences of residential sorting.

## Scoring

**Score**: 4.0/4 (Strong Accept)

**Score Justification**: All concerns addressed. Geographic mechanisms fully developed. Empirical rigor exceptional. Theoretical contributions substantial. This sets the standard for computational redistricting research.

---

## Detailed Assessment

### 1. Geographic Sorting Mechanism Deep Dive (NEW Section 5.1)

This section transforms the paper from descriptive analysis (algorithmic plans show -3.2% EG) to explanatory science (here's *why* they show -3.2% EG, and why this represents a lower bound).

**Quantification of urban concentration** (lines 11-18): The finding that 47 districts where Democrats win >70% account for 38% of Democratic votes but only 24% of Democratic seats immediately reveals the packing problem. The 3.8× wasted vote asymmetry (Dem: 4.2 million surplus votes; Rep: 1.1 million) provides the empirical foundation for the negative EG.

The critical insight: "Urban Democratic concentration is not a choice of algorithmic design but an unavoidable consequence of geographic compactness constraints: when major cities (Philadelphia, Milwaukee, Detroit, Phoenix, Atlanta) contain 65-80% Democratic voters, any compact district centered on these cities will produce Democratic supermajorities."

This sentence encapsulates the geographic determinism thesis that underlies the entire paper.

**Suburban asymmetry explanation** (lines 22-32): The residential sorting gradients—Democratic support dropping precipitously from cores to suburbs (78% → 62% → 48%), Republican support showing gentler slopes (58% → 64% → 71%)—explain why Republicans achieve more efficient vote allocation even under neutral algorithms.

The geometric mechanism is clearly stated: "Compact districting minimizes border length, which tends to group geographically proximate voters. Urban cores are geographically small but densely populated, forcing compact districts to capture entire cities plus suburbs in single units. Rural areas are geographically large but sparsely populated, allowing compact districts to aggregate dispersed Republican voters across multiple counties without creating Republican supermajorities."

**Minor suggestion**: Consider adding a visual (map or schematic) showing a prototypical city-suburban-rural transect with vote share gradients overlaid. This would make the mechanism immediately intuitive to readers unfamiliar with electoral geography. Something like: Philadelphia city center (75% Dem) → inner suburbs (60% Dem) → outer suburbs (48% Dem) → exurbs (55% Rep) → rural (68% Rep), with district boundaries superimposed showing how compactness constraints force packing.

**Compactness-partisan tradeoff** (lines 34-43): The empirical demonstration—testing edge weight variations from 1.0 (maximum compactness) to 0.5 (relaxed)—shows that:
- Maximum compactness (1.0): EG = -3.4%, Polsby-Popper = 0.33
- Relaxed compactness (0.5): EG = -1.9%, Polsby-Popper = 0.24 (-27% compactness)

The 1.5 pp EG improvement from accepting 27% compactness loss establishes the tradeoff gradient. Extrapolating: achieving zero EG would likely require ~50% compactness loss, producing districts courts would reject as gerrymandered on compactness grounds.

This finding has profound legal implications. Courts often assume "neutral" redistricting produces zero partisan bias. Your analysis proves this is geometrically impossible under compactness constraints—the best achievable is -3.2% EG (modest Democratic advantage from unavoidable urban packing).

**Connection to seats-votes analysis** (lines 45-52): Tying geographic sorting to the seats-votes curves (intercept bias + maintained responsiveness) integrates Sections 4.6.2 and 5.1. The key insight: "Urban packing creates the intercept bias (Democrats need only 47.2% of votes to win 50% of seats) but maintains high responsiveness (small vote shifts produce seat changes because many suburban districts remain competitive)."

This explains why algorithmic plans have *bias* (intercept) but not *entrenchment* (dampened responsiveness)—a crucial distinction for evaluating democratic fairness.

### 2. Compactness Correlation Analysis (NEW Section 4.3.1)

This section provides the smoking gun evidence that enacted plans' partisan bias cannot be explained by compactness differences from algorithmic plans.

**The critical finding** (Table, lines 105-107): Arizona and Nevada enacted plans have *identical* Polsby-Popper scores to algorithmic plans (0.28 and 0.25 respectively), yet show 6.6 pp and 8.4 pp higher efficiency gaps. This proves partisan manipulation operates independently of compactness—mapmakers achieved similar geometric compactness while introducing partisan advantage through strategic boundary placement.

The scatter plot analysis (lines 112-114) reinforces this: within enacted plans, there is no correlation between compactness and efficiency gap (r=0.12, p=0.68). States with high compactness (Wisconsin: 0.34) show similar EG (+7.2%) to states with low compactness (Texas: 0.24, +6.2%).

**Implication for gerrymandering detection**: This finding refutes the common defense: "Our maps are reasonably compact, so any partisan bias must reflect unavoidable geography." Your analysis shows this defense fails empirically—similar compactness can produce dramatically different partisan outcomes depending on boundary placement choices.

**Minor suggestion**: Consider adding one or two specific examples of how identical compactness with different EG occurs. For instance, Arizona: both algorithmic and enacted plans have Polsby-Popper 0.28, but enacted plan cracks Tucson differently, distributing urban Democrats across multiple suburban districts to dilute their influence. A district-level comparison (even just 2-3 districts) would make the mechanism concrete.

### 3. Multiple Metrics Comparison (NEW Section 4.8)

While this section primarily addresses Stephanopoulos's and McDonald's concerns about metric robustness, it also has geographic implications. The convergence across metrics—especially the connection between efficiency gap (-3.2%), partisan bias at 50% (+2.0 pp), and declination (-4.2°)—demonstrates that multiple independent measures all detect the same underlying geographic sorting pattern.

The mean-median difference analysis is particularly insightful: algorithmic plans show +0.8 pp (modest packing), enacted plans show +4.1 pp (severe packing). Yet enacted EG is +5.1% (Republican advantage) despite severe Democratic packing. This reveals enacted plans use *both* packing *and* cracking—packing Democrats into fewer districts while cracking remaining Democrats across suburban districts.

### 4. Seats-Votes Full Treatment (NEW Section 4.6.2)

The historical comparison (2000: +1.2 pp bias → 2010: +3.8 pp → 2020: +6.0 pp) documents escalating partisan manipulation over three redistricting cycles. By contrast, algorithmic plans show stable bias across decades (-2.9% → -3.1% → -3.2%), confirming that negative algorithmic EG reflects durable geographic patterns rather than decade-specific artifacts.

The bias vs responsiveness decomposition clarifies two distinct fairness dimensions. Algorithmic plans have intercept bias (+2.0 pp Democratic advantage at 50% vote parity) but high responsiveness (elasticity 2.8). Enacted plans have larger opposite-direction bias (-6.0 pp Republican advantage) *plus* dampened responsiveness (elasticity 2.1).

This double penalty—bias favoring one party *and* reduced electoral competition—represents the most pernicious form of gerrymandering.

---

## Theoretical Contributions

Beyond addressing my specific concerns, the revision makes three broader theoretical contributions to electoral geography:

### 1. Empirical Lower Bound on Partisan Bias

The -3.2% algorithmic efficiency gap establishes an empirical lower bound on partisan bias achievable under three constraints:
- Compact, contiguous districts (Polsby-Popper > 0.30)
- Equal population (±0.5% tolerance)
- No access to partisan data

This baseline has implications beyond redistricting. Any single-member district electoral system in a country with urbanized partisan sorting will exhibit similar bias. The finding generalizes beyond the U.S. context—Canada, UK, Australia all exhibit center-left urban concentration that produces similar packing effects.

**Future research direction**: Cross-national comparison of algorithmic redistricting in other democracies with geographic partisan sorting. Does UK Labour face similar packing in London/Manchester? Does Canadian NDP pack in Toronto/Vancouver?

### 2. Compactness-Proportionality Tradeoff

The compactness-partisan tradeoff (27% compactness loss → 1.5 pp EG improvement) quantifies a fundamental tension in democratic representation: should we prioritize *geographic communities* (compact districts) or *proportional outcomes* (seat share matching vote share)?

This connects to broader debates in democratic theory. Proportional representation systems (party-list, mixed-member) achieve proportionality by abandoning geographic representation. Single-member district systems prioritize geography but sacrifice proportionality. Your finding shows this tradeoff is not just institutional design but geometric necessity—compact districts with urban-rural sorting *cannot* achieve proportionality.

**Policy implication**: States adopting algorithmic redistricting should anticipate and explain to voters that modest Democratic advantage (-3.2% EG nationally, varying by state) reflects geography, not algorithm bias. Achieving perfect proportionality would require non-compact districts or abandoning single-member districts entirely.

### 3. Manipulation Detection via Geographic Baselines

The combination of algorithmic baselines + compactness analysis provides a principled framework for detecting partisan manipulation:

**Step 1**: Generate algorithmic baseline EG for state given its population geography
**Step 2**: Measure enacted plan EG and compactness
**Step 3**: Compare:
- If enacted EG ≈ algorithmic EG (within ~2 pp) → geography explains partisan patterns
- If enacted EG >> algorithmic EG *and* enacted compactness < algorithmic compactness → likely manipulation
- If enacted EG >> algorithmic EG *and* enacted compactness ≈ algorithmic compactness → **definitive manipulation** (Arizona/Nevada cases)

This framework could be operationalized for courts or redistricting commissions evaluating proposed maps.

---

## Remaining Questions and Future Work

While the revision is comprehensive, a few questions remain that could motivate future research:

### 1. Regional Variation in Geographic Sorting

You note that Rust Belt states (PA, WI, MI) show largest enacted-algorithmic differences (10 pp), while other regions show smaller gaps (7-9 pp). What explains this variation? Possible hypotheses:

- **REDMAP targeting**: Republican REDMAP initiative (2010) specifically targeted Rust Belt state legislatures for redistricting control
- **Urban structure**: Rust Belt cities (Detroit, Milwaukee, Pittsburgh) are highly monocentric with sharp urban-suburban gradients, enabling aggressive cracking
- **Competitive balance**: States with near-50% partisan balance offer more opportunities for manipulation than landslide states

A deeper analysis of regional variation mechanisms would be valuable. (You mention this as deferred P2.4 work—I agree this is medium-priority but would strengthen the paper if included.)

### 2. City Structure and Partisan Sorting

The residential sorting gradient analysis (urban → suburban → rural) implicitly assumes monocentric cities. But many Sunbelt cities (Phoenix, Houston, Atlanta) are polycentric with multiple urban cores. Does polycentric structure affect the compactness-partisan tradeoff?

Hypothesis: Monocentric cities (Detroit, Philadelphia) produce sharper sorting gradients and more severe packing than polycentric cities (Phoenix), potentially explaining why Arizona shows smaller algorithmic baseline EG (-1.9%) than Pennsylvania (-2.8%).

### 3. Temporal Evolution of Sorting

Your algorithmic baselines are stable across decades (-2.9% → -3.1% → -3.2%), but this masks potential underlying changes in sorting patterns. Are suburban areas becoming *more* Democratic (education polarization) or remaining stable? Has gentrification reduced urban Democratic concentration in some cities?

A deeper temporal analysis of sorting evolution—ideally tracking individual cities or metropolitan areas across decades—would enrich understanding of the -3.2% baseline's stability.

---

## Minor Suggestions for Polish

### 1. Visual Enhancement

The geographic sorting section (5.1) would benefit enormously from visuals:
- **Figure A**: Map showing vote share gradients from city center to rural areas for 2-3 prototypical states (e.g., Pennsylvania, Arizona, Texas representing different urban structures)
- **Figure B**: Schematic diagram illustrating how compactness constraints interact with population density gradients to produce packing

### 2. Case Study Deepening

Consider adding a brief case study (1-2 paragraphs) for Pennsylvania showing:
- Philadelphia city (75% Dem, 1.5M people, 50 sq mi) → compact district must include suburbs → produces 70%+ Dem supermajority
- Versus: Rural Pennsylvania (65% Rep, dispersed across 20,000 sq mi) → compact district aggregates multiple counties → produces 58% Rep moderate majority

This concretizes the abstract geometric argument.

### 3. Terminology Consistency

You use "residential sorting," "geographic sorting," and "partisan sorting" somewhat interchangeably. While meanings are clear from context, consider defining your preferred term early (I recommend "residential partisan sorting" as most precise) and using consistently.

---

## Recommendation

**Strong Accept**. This paper makes foundational contributions to electoral geography and redistricting science:

1. **Quantifies geographic sorting mechanisms**: First precise measurement of urban concentration, wasted vote asymmetry, and suburban gradients producing partisan effects
2. **Establishes empirical lower bound**: -3.2% EG represents minimum achievable bias under compactness + equal population + partisan blindness
3. **Proves manipulation detection**: Identical compactness with different EG (Arizona/Nevada) provides definitive evidence of strategic boundary placement
4. **Integrates partisan fairness literature**: Connects EG, proportionality, seats-votes, compactness in unified framework

The revision transforms the paper from a strong empirical study into a landmark contribution that will shape understanding of geographic foundations of partisan asymmetry in democracies with urbanized political geography.

I enthusiastically recommend acceptance for publication in the American Political Science Review.

---

## Score Changes from Round 1

**Round 1 Score**: 3.5/4 (Accept - moderate revisions required)
**Round 2 Score**: 4.0/4 (Strong Accept)

**Reasons for score increase**:
- Geographic sorting mechanisms fully developed (Section 5.1): quantified urban concentration, demonstrated compactness tradeoff, explained suburban asymmetry
- Compactness correlation analysis (Section 4.3.1): definitive proof that manipulation operates independently of compactness
- Theoretical contributions elevated from empirical description to explanatory science
- All P2.1 concerns (my primary issue) comprehensively addressed
- Paper now provides foundation for future electoral geography research
