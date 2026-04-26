# Partisan Metrics Guide

How `redist` measures partisan fairness — what each metric means, how to read the output, and what thresholds matter legally.

---

## Why measure partisan fairness?

Redistricting plans affect which party wins seats. A plan can be geographically compact and legally balanced while still systematically advantaging one party. Courts, commissions, and the public increasingly expect quantitative evidence that a plan is not a partisan gerrymander.

`redist` computes three widely-used metrics, each with 95% bootstrap confidence intervals. None of these metrics is a legal standard by itself — they are analytical tools that practitioners and experts use to evaluate fairness claims.

---

## Running the analysis

```bash
redist analyze --label wa_house_v1 --year 2020 --version WA_Plans --types partisan
```

This produces `analysis/partisan.json`. For state legislative chambers, `redist` uses 2020 presidential election results as a proxy (see Limitations below). To use state legislative election data, supply it directly:

```bash
redist analyze --label wa_house_v1 --types partisan \
  --election-file data/custom/wa_2022_house_by_tract.csv
```

---

## The three metrics

### Efficiency Gap (EG)

**What it measures**: The difference in "wasted votes" between the two parties. In every district, one party wins and the other loses. The winning party "wastes" every vote beyond what was needed to win (the margin). The losing party wastes every vote it cast. The efficiency gap measures whether one party consistently wastes more votes than the other — a sign the map is structured to crack or pack that party's voters.

**Formula**:
```
EG = (Wasted_Party_A - Wasted_Party_B) / Total_votes
```

**Range**: −1.0 to +1.0. Positive values favor one party; negative favor the other.

**Reading the output**:
```json
{
  "efficiency_gap": {
    "value": 0.031,
    "direction": "Democratic",
    "ci_95_low": 0.011,
    "ci_95_high": 0.051,
    "academic_reference": "8% threshold from Stephanopoulos & McGhee (2015). SCOTUS declined to adopt in Gill v. Whitford (2018). Not a constitutional standard."
  }
}
```

An EG of +0.031 means Democratic voters waste 3.1% more of their votes than Republican voters across all districts — a small advantage for Republicans.

**The 8% threshold**: Stephanopoulos and McGhee (2015) proposed ±8% as a presumptive gerrymander threshold. The Supreme Court in *Gill v. Whitford* (2018) explicitly did not adopt this. `redist` reports it as an academic reference, not a legal standard.

---

### Mean-Median Difference (MM)

**What it measures**: The gap between a party's average vote share across districts and its median vote share. In a symmetric map, the mean and median should be close. A large gap means the party wins some districts by huge margins (packing) while losing many narrowly (cracking).

**Formula**:
```
MM = mean(district_dem_vote_share) - median(district_dem_vote_share)
```

**Range**: −1.0 to +1.0. Positive = Democratic advantage; negative = Republican advantage.

**Reading the output**:
```json
{
  "mean_median": {
    "value": -0.023,
    "direction": "Republican",
    "ci_95_low": -0.041,
    "ci_95_high": -0.005
  }
}
```

MM of −0.023 means Democrats' median district vote share is 2.3 points higher than their mean — suggesting their votes are somewhat packed into strong districts.

**No widely accepted threshold** exists for MM. Values above ±7% are sometimes flagged in academic literature but have no legal standing.

---

### Partisan Bias (PB)

**What it measures**: How many seats a party would win if it received exactly 50% of the statewide vote. In a neutral map, a party with 50% of votes should win roughly 50% of seats. Partisan bias measures the deviation from this baseline.

**Formula**: Estimate via uniform swing — find the vote shift S where a party reaches exactly 50% statewide, then compute `actual_seat_share - 0.5` at that shift.

**Range**: −0.5 to +0.5. Positive = Democratic advantage at 50% vote.

**Reading the output**:
```json
{
  "partisan_bias": {
    "value": 0.018,
    "direction": "Democratic",
    "ci_95_low": -0.002,
    "ci_95_high": 0.038
  }
}
```

PB of 0.018 means that if Democrats won exactly 50% of the statewide vote, they would be expected to win about 51.8% of seats — a modest structural advantage.

---

## Confidence intervals

All three metrics come with 95% bootstrap confidence intervals computed from 1,000 resamples. When the CI spans zero (e.g., `ci_95_low: -0.002, ci_95_high: 0.038`), the metric is not statistically distinguishable from zero — don't over-interpret it.

**Important**: Bootstrap CI is not computed for chambers with fewer than 10 districts (VT, WY, AK, ND, SD, MT, DE). The output shows `ci_available: false` with an explanation.

---

## Statewide summary

The output also includes overall vote and seat shares:

```json
{
  "statewide": {
    "dem_vote_share": 0.583,
    "dem_seat_share": 0.600,
    "total_votes": 3284823
  }
}
```

If `dem_vote_share` ≈ `dem_seat_share`, the plan translates votes to seats proportionally. A large gap is not automatically a gerrymander (geographic clustering also causes gaps) but warrants explanation.

---

## Competitive districts

```json
{
  "districts": [
    {
      "district": 7,
      "dem_pct": 0.512,
      "margin": 0.024,
      "is_competitive": true,
      "is_uncontested": false
    }
  ]
}
```

Districts within ±5% (`dem_pct` between 0.45 and 0.55) are flagged as competitive. Uncontested races (`rep_votes = 0` or `dem_votes = 0`) are flagged separately — they show as 100%/0% but don't reflect genuine partisan preferences.

---

## Limitations

**Presidential proxy**: When state legislative election data is not available, `redist` uses presidential results. Presidential elections have stronger partisan swings than legislative races, especially in urban areas. When presidential data is used for a non-congressional chamber, `partisan.json` includes:

```json
{
  "methodology_warning": "Presidential election results used as proxy for state legislative district partisanship. Presidential coattail effects may systematically inflate urban Democratic margins. Provide state legislative election data via --election-file for more accurate analysis."
}
```

**Historical election data**: Partisan metrics measure how a plan performs against one specific election. A plan that looks fair for 2020 presidential results might look different for 2016 or 2022 legislative results. Courts have sometimes required analysis across multiple election cycles.

**Geographic clustering**: Some partisan asymmetry is inherent to geography. Democratic voters are more geographically concentrated in cities, which produces mechanical disadvantages in single-member district systems regardless of how districts are drawn. Metrics that don't account for this (EG and MM don't; PB partially does) may overstate partisan bias.

---

## What to say about these metrics

The commission report auto-generates appropriate framing, but when explaining these metrics to a non-technical audience:

- "These metrics measure whether one party's voters tend to waste more votes than the other party's voters. All three metrics show values close to zero with confidence intervals that include zero, suggesting the plan does not systematically disadvantage either party."

Or, if there is significant asymmetry:

- "The efficiency gap of 0.11 is above the academic threshold of 0.08. This means Republican voters cast approximately 11% more wasted votes than Democratic voters under this plan. The confidence interval (0.08, 0.14) does not include zero. The commission should consider whether this asymmetry is explained by geographic clustering or by the plan's structure."
