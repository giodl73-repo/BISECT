---
name: review-claims
description: Verify a specific quantitative claim against current pipeline data. Uses DATUM (evidence) and SCALE (statistics). Run this before publishing any headline number.
user_invocable: true
---

# Claim Verification

Verify a specific quantitative claim against actual pipeline outputs. The most dangerous moment in research is the gap between "this was true when we ran the experiment" and "this is true now." This skill closes that gap.

## Input

The user states a claim to verify. Examples:
- "+56% compactness improvement over unweighted baseline"
- "mean Polsby-Popper 0.367 for 2020 Census"
- "37 of 50 states beat enacted maps on compactness"
- "Alabama achieves 2 majority-minority districts in V4"
- "2020 results are 10% better than 2010"

## Steps

### 1. Parse the claim

Extract:
- **Metric**: what is being measured (PP, Reock, MM count, deviation %)
- **Value**: the claimed number
- **Comparison**: what it's compared against (baseline, enacted, prior year)
- **Scope**: which states/years/versions

### 2. Load the data

Based on scope, load from:
- `outputs/V3/2020/states/*/data/district_summary.csv` — PP, Reock, population
- `outputs/V4/2020/states/*/data/vra_analysis.json` — MM districts
- `outputs/data/2020/partitioner_comparison/partitioner_comparison_2020.csv` — baseline comparison
- `outputs/data/2020/partisan_metrics/partisan_metrics_2020_algorithmic.csv` — efficiency gap

### 3. DATUM check — does the evidence exist?

- Is the data source the correct one for this claim? (PL 94-171 not ACS for population)
- Is the comparison baseline correctly constructed?
- What was the pipeline version/date when this claim was last verified?
- Can the number be reproduced from current outputs?

### 4. SCALE check — is the number statistically sound?

**For percentage improvements:**
- What is the confidence interval? (If not reported, compute bootstrap CI)
- Is the improvement consistent across states or driven by outliers?
- What is the distribution? (Report mean AND median — are they close?)

**For counts (e.g., "37 of 50 states"):**
- Verify the exact count from current data
- Has it changed since the claim was written?

**For cross-year comparisons:**
- Are the same 50 states compared (some states change district counts)?
- Are the PP scores weighted by district count or state?

### 5. Report

```
CLAIM: "{exact claim text}"
SOURCE: {paper or document where this appears}

DATA CHECK:
  Source file:     {path to data used}
  Current value:   {computed from current outputs}
  Claimed value:   {from paper}
  Status:          MATCH / DRIFT / CANNOT VERIFY

STATISTICAL SOUNDNESS:
  Sample:          {N states, N districts}
  Mean:            {value}
  Median:          {value}
  95% CI:          [{low}, {high}]
  Outlier states:  {states most responsible for the result}

VERDICT:
  PUBLISH-READY   — claim matches current data and is statistically sound
  UPDATE NEEDED   — claim is directionally right but number has drifted
  WITHDRAW        — claim cannot be supported by current data
  REFRAME         — claim is true but misleadingly stated

RECOMMENDED TEXT: "{exact rewrite if needed}"
```

## Key Rules

- **Run on current pipeline outputs.** A claim verified 6 months ago against old data is not verified.
- **The +56% claim uses V3 vs. unweighted baseline.** If no unweighted run exists in outputs, load from `partitioner_comparison_2020.csv`.
- **The 0.367 mean PP is from V3/2020.** V4 (VRA) will have lower PP — different run, different claim.
- **SCALE requires uncertainty.** If the paper states "0.367" without bounds, the verdict is REFRAME, not PUBLISH-READY, until bounds are added.
- **37 of 50 states beating enacted** is verified from `partitioner_comparison_2020.csv` not from V3 district_summary directly.
- **Mississippi and Louisiana VRA claims need updating.** Current V4 shows 1 MM each, not 2. Any paper claiming 2 for these states needs UPDATE NEEDED verdict.
