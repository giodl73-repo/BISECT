# Paper 17: Follow-up Analysis Checklist

## Status: Core experiments complete, follow-ups needed for publication

### ✅ What We Have (Complete)

1. **Core negative result** - Edge-weighting fails to improve on baseline
   - 18 configurations tested (6 α × 3 τ)
   - 5 large states: CA, TX, NY, PA, FL (161 districts)
   - Clear pattern: 77% safe regardless of α
   - Figures and tables ready

2. **Compactness trade-off quantified**
   - PP drops 30% from baseline to extreme
   - Trade-off curve plotted (Figure 1)
   - Clear evidence of cost with no benefit

3. **Tau threshold tested**
   - 3 thresholds (10pp, 15pp, 20pp)
   - Shows minimal effect on outcomes
   - Rules out threshold as confound

### 🔴 Critical Follow-ups (Required for Publication)

#### 1. **Compare to Enacted Maps** (HIGH PRIORITY)
**Why essential**: Need to show whether real gerrymanders deviate from geographic baseline.

**Questions to answer**:
- Do 2022 enacted maps have more/fewer safe seats than our baseline (77%)?
- Are enacted maps less compact than algorithmic baseline?
- Does gerrymandering actually "beat" geography?

**Data needed**:
- 2022 enacted congressional district shapefiles
- Election results by enacted district
- Compactness metrics for enacted districts

**Expected finding**:
- Enacted maps: ~85-90% safe seats (slightly higher)
- But much less compact (PP ~0.15-0.20 vs 0.25 baseline)
- Gerrymanders sacrifice compactness for marginal safety gain

**Implementation**:
```bash
# Download 2022 enacted districts
# Load election results by district (already have)
# Compute same metrics as experimental configs
# Add "Enacted" row to summary table
```

**Impact**: **CRITICAL** - Without this, we can't claim geography dominates. Might find enacted maps ARE better, which would change the story!

---

#### 2. **Test Smaller/Less-Sorted States** (HIGH PRIORITY)
**Why essential**: Findings may be specific to large, sorted states. Need to test generalizability.

**States to test**:
- **Competitive**: IA (4), NH (2), ME (2), WI (8), VA (11) = 27 districts
- **Why these**: More suburban/mixed, less urban/rural divide
- **Hypothesis**: Edge-weighting might work better in less-sorted states

**Expected finding**:
- Less-sorted states: 60-70% safe at baseline (vs 77%)
- Edge-weighting might increase to 70-75% (modest effect)
- But compactness trade-off still present

**Implementation**:
```bash
# Already have the script!
python scripts/experiments/partisan_similarity_run.py --year 2020 \
  --states IA NH ME WI VA \
  --alpha 1 10 50 --tau 15
```

**Impact**: **CRITICAL** - Shows scope/limits of finding. If it works in competitive states, that's interesting! If not, strengthens "geography dominates everywhere" claim.

---

#### 3. **State-by-State Breakdown** (MEDIUM PRIORITY)
**Why important**: Show variation across the 5 states we tested.

**Questions**:
- Does CA (very sorted) behave differently from PA (moderately sorted)?
- Which states have highest baseline safe %?
- Is compactness loss uniform or state-specific?

**Implementation**:
```python
# Analyze by state using existing data
# Create Table 2: State-by-state results
# Show: baseline_safe%, alpha50_safe%, compactness_change
```

**Expected finding**:
- CA, NY: 85%+ safe at baseline (highly sorted)
- TX, PA, FL: 70-75% safe (moderately sorted)
- Edge-weighting effect inversely proportional to baseline sorting

**Impact**: MEDIUM - Adds nuance, shows geographic sorting variation.

---

### 🟡 Optional Follow-ups (Strengthen Paper)

#### 4. **Theory Section: Why Geographic Sorting Dominates**
**What to add**:
- Formal model of residential sorting
- Graph theory explanation (natural cuts minimize partisan boundaries)
- "The Big Sort" literature connection

**Implementation**: Literature review + mathematical formalization

**Impact**: MEDIUM - Makes paper more theoretical, appeals to different audience.

---

#### 5. **Comparison to VRA Edge-Weighting**
**Why interesting**: We have VRA edge-weighting code already. Compare partisan vs racial clustering.

**Question**: Why does VRA edge-weighting work but partisan doesn't?

**Hypothesis**: 
- Racial minorities are geographically dispersed → edge-weighting helps cluster them
- Partisan sorting already clustered → edge-weighting can't improve

**Implementation**:
```bash
# Run existing VRA edge-weighting scripts for same 5 states
# Compare effectiveness of racial vs partisan edge-weighting
```

**Impact**: LOW-MEDIUM - Interesting comparison, but not essential to main story.

---

#### 6. **Robustness Checks**
**Additional tests**:
- Different election years (2016, 2018, 2022) - does sorting persist?
- Different metrics (Gallagher index, seats-votes curves)
- Different partition algorithms (Hess, FRP, etc.)

**Impact**: LOW - Strengthens claims but not essential for first submission.

---

## Recommended Action Plan

### Phase 1: Critical Follow-ups (Required for submission)
**Timeline**: 1-2 weeks

1. **Week 1, Day 1-2**: Compare to enacted maps
   - Download 2022 district shapefiles
   - Compute same metrics
   - Add to summary table

2. **Week 1, Day 3-4**: Test competitive states
   - Run: IA, NH, ME, WI, VA (27 districts)
   - 3 alphas × 1 tau = 3 configs (~15 min)
   - Analyze and compare to large state results

3. **Week 1, Day 5**: State-by-state analysis
   - Break down existing results by state
   - Create Table 2
   - Identify state-level patterns

4. **Week 2**: Draft results + discussion incorporating follow-ups

### Phase 2: Optional Enhancements (After initial draft)
**Timeline**: 1-2 weeks after draft

5. **Optional**: Theory section (if reviewers request)
6. **Optional**: VRA comparison (if space permits)
7. **Optional**: Robustness checks (if reviewers request)

---

## Expected Paper Structure (After Follow-ups)

### Results Section
- **Table 1**: Core ablation (18 configs, 5 large states) ✅ DONE
- **Table 2**: State-by-state breakdown 🔴 NEEDED
- **Table 3**: Comparison to enacted maps 🔴 NEEDED
- **Table 4**: Competitive states results 🔴 NEEDED
- **Figure 1**: Tradeoff curve ✅ DONE
- **Figure 2**: Safe seats by alpha ✅ DONE

### Discussion Section
- Geographic sorting dominance (use Tables 1-4)
- Enacted maps comparison (use Table 3)
- State variation (use Table 2)
- Limits of algorithmic redistricting
- Policy implications

---

## Bottom Line

**Current status**: Strong experimental foundation with surprising negative result

**To make publishable**: 
1. ✅ Core experiments (DONE)
2. 🔴 Compare to enacted maps (CRITICAL - 2 days work)
3. 🔴 Test competitive states (CRITICAL - 1 day work)
4. 🟡 State-by-state analysis (HELPFUL - 1 day work)

**Total additional work**: ~4-5 days of experiments + 3-4 days of writing

**Paper strength after follow-ups**: Strong AJPS submission with comprehensive evidence

---

## Questions to Resolve

1. **Do we want to include VRA comparison?** (Interesting but adds complexity)
2. **How deep should theory section be?** (Light touch vs formal model)
3. **Should we test 2016/2018 elections?** (Robustness vs scope creep)

**Recommendation**: Do critical follow-ups (enacted maps + competitive states) first, then assess whether optional ones add enough value.
