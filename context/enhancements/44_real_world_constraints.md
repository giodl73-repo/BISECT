# E44: Real-World Redistricting Constraints

**Status**: Proposed
**Priority**: Medium
**Created**: January 17, 2026
**Commits**: [ad542e9](https://github.com/giodl_microsoft/redistricting/commit/ad542e9efb96bd2ee745b6ab6e77f2ee721b1789)
**Size**: M - 1,191 lines changed (6 files)

## Problem Statement

Current algorithmic redistricting optimizes for compactness and population balance but ignores real-world legal and practical constraints that govern actual redistricting.

**Current Implementation:**
- ✅ Population balance (±0.5%)
- ✅ Contiguity (all districts connected)
- ✅ Compactness (edge-weighted METIS)
- ❌ Voting Rights Act (VRA) compliance
- ❌ Communities of Interest (COI)
- ❌ County/municipality preservation
- ❌ Incumbent protection (partisan consideration)

**Limitation:**
Districts may be mathematically optimal but legally problematic or practically unacceptable.

## Real-World Constraints

### 1. Voting Rights Act (VRA) Compliance

**Legal Requirement:**
- Section 2: Prohibits racial vote dilution
- Requires "majority-minority" districts where possible
- Must not "pack" or "crack" minority communities

**Current Gap:**
- No demographic consideration in partitioning
- No tracking of minority representation
- No analysis of whether algorithmic districts improve/worsen minority representation

**Example:**
- Georgia: Should create 2-3 majority-Black districts
- Texas: Should create Hispanic-majority districts
- Current algorithm: Ignores race entirely (by design, but may violate VRA)

**Implementation Needs:**
- Load demographic data (race/ethnicity by tract)
- Compute minority voting-age population (MVAP) per district
- Flag districts below VRA thresholds
- (Advanced) Add VRA constraint to METIS optimization

### 2. Communities of Interest (COI)

**Definition:**
Groups with shared policy interests (ethnic, economic, geographic, social)

**Examples:**
- Agricultural regions (rural farming communities)
- Urban cores (city centers with shared transit/services)
- Tribal lands (Native American reservations)
- College towns (university-dominated areas)
- Industrial zones (manufacturing/mining regions)

**Current Gap:**
- Algorithm treats all tracts as interchangeable
- May split natural communities (e.g., divide a city across 4 districts)

**Implementation Challenges:**
- COI data is subjective (no official boundaries)
- Would require manual/community input
- Hard to quantify

**Possible Approach:**
- Define COI regions manually (shapefile/GeoJSON)
- Add penalty for crossing COI boundaries in METIS edge weights
- Report COI split percentages in analysis

### 3. County and Municipality Boundaries

**State Constitutional Requirements:**
Many states require minimizing county/city splits:
- Iowa: Minimize county splits (strict)
- Florida: Respect county boundaries
- Virginia: Preserve counties where possible

**Current Gap:**
- Algorithm ignores county boundaries
- May create districts spanning 10+ counties (visually messy, administratively complex)

**Implementation:**
- Load county boundaries
- Add edge penalty for crossing county lines
- Report: "District X spans 7 counties" in analysis
- (Advanced) Add hard constraint: "Never split counties" (Iowa-style)

**Trade-off:**
- Respecting counties may reduce compactness
- Conflict: Compactness vs. traditional boundaries

### 4. Incumbent Protection / Partisan Considerations

**Reality:**
- Real redistricting is highly partisan
- Parties try to protect incumbents or disadvantage opponents
- Algorithms ignore politics (by design)

**Current Gap:**
- No tracking of incumbent home locations
- No partisan analysis during partitioning (only post-hoc analysis)

**Ethical Question:**
Should algorithms consider partisan balance?
- **Pro:** "Fairness" could mean competitive districts (45-55% swing)
- **Con:** Districting should be non-partisan (compactness only)

**Possible Enhancement:**
- Load incumbent addresses, track which districts contain incumbents
- Report: "5 Republican incumbents, 2 Democratic incumbents in same district" (indicates pairing)
- (Controversial) Add partisan "fairness" constraint

**Note:** This is controversial and may undermine neutrality argument.

## Implementation Approach

### Phase 1: Data Collection
- [ ] Download VRA-relevant demographics (Census Bureau race/ethnicity data)
- [ ] Obtain county boundaries (TIGER/Line shapefiles)
- [ ] Define initial COI regions (major cities, tribal lands)
- [ ] (Optional) Collect incumbent addresses

### Phase 2: Analysis Integration
- [ ] Compute MVAP (Minority Voting-Age Population) per district
- [ ] Count county splits per district
- [ ] Measure COI fragmentation
- [ ] Add constraint violation reports to dashboard

### Phase 3: Constraint Enforcement (Advanced)
- [ ] Modify METIS edge weights to penalize county crossing
- [ ] Add COI boundary penalty
- [ ] (Research) Explore VRA-aware partitioning algorithms

### Phase 4: Comparative Analysis
- [ ] Compare algorithmic vs. enacted districts on VRA compliance
- [ ] Compare county splits (algorithmic vs. enacted)
- [ ] Generate "constraint violation score" for each approach

## Expected Findings

**Hypothetical Results:**
- "Algorithmic districts split 30% fewer counties than enacted districts"
- "Enacted districts create 12 majority-minority districts; algorithmic creates 8 (VRA compliance issue)"
- "67% of COIs remain intact under algorithmic redistricting vs. 45% under enacted districts"

## Use Cases

### Legal Defense
- Argue algorithmic districts satisfy traditional redistricting criteria (compactness, county preservation)
- Identify where VRA compliance requires deviation from pure compactness

### Policy Guidance
- Show trade-offs: "Respecting county boundaries reduces compactness by 8%"
- Guide reformers on which constraints matter most

### Academic Research
- Quantify tension between compactness and other principles
- Study whether VRA compliance is compatible with maximum compactness

## Success Criteria

- [ ] VRA analysis complete: Minority representation computed for all districts
- [ ] County split analysis: Count and visualize county boundaries crossed
- [ ] COI analysis: At least 10 major COIs defined and fragmentation measured
- [ ] Dashboard updated: Show constraint violations prominently
- [ ] Paper section: "Legal and Practical Constraints" written

## Files to Create

- `scripts/vra/analyze_minority_representation.py`
- `scripts/constraints/compute_county_splits.py`
- `scripts/constraints/analyze_coi_fragmentation.py`
- `scripts/constraints/generate_constraint_report.py`
- `data/constraints/county_boundaries.geojson`
- `data/constraints/communities_of_interest.geojson`
- `artifacts/papers/05_constraints_analysis/` (optional new paper)

## Related Enhancements

- [E42: Research Narrative](42_research_narrative_policy_questions.md) - Provides policy context
- [E11: Baseline Comparison](11_baseline_comparison.md) - Compare VRA compliance vs. enacted

## Notes

**Trade-offs:**
- Pure compactness algorithm is simpler, more defensible (neutral)
- Adding constraints makes it more practical but less "pure"
- Question: Should algorithm be theoretical ideal or practical tool?

**Legal Risk:**
- Ignoring VRA may make algorithmic districts legally problematic
- Courts have struck down maps for VRA violations
- Must address this for real-world adoption

**Recommendation:**
- Start with analysis only (measure violations, don't enforce yet)
- Phase 2: Add soft constraints (penalties, not hard blocks)
- Phase 3: Offer "VRA-compliant mode" as variant

**User's Research Decision:**
- Is this a "theoretical ideal" project or "practical tool" project?
- Theoretical → ignore constraints, focus on pure compactness
- Practical → must address VRA, counties, COI
