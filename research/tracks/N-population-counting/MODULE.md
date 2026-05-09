# Track N — Population Counting and the Balance Metric

**Theme**: "One person, one vote" requires equal *population* across districts — but whose population? Total population, citizen voting-age population, registered voters, and eligible voters all produce different district maps. This track addresses the most legally contested input assumption in algorithmic redistricting.

**Core question per paper**: If we change the population count used in the balance metric, how does the bisect plan change, and what are the legal implications?

## Track Chain

N.0 (overview: who counts?) → N.1 (prison) → N.2 (college) → N.3 (noncitizens)
                              → N.4 (military) → N.5 (total vs. citizen VAP empirical comparison)

N.1–N.4 are independent population edge cases. N.5 synthesizes them into an empirical comparison.

## Papers

| Paper | Title | Population Adjustment | Legal Hook |
|-------|-------|----------------------|-----------|
| N.0 | The Balance Metric: Who Counts in Congressional Redistricting? | Overview | Reynolds v. Sims; Evenwel |
| N.1 | Prison Gerrymandering: Rural Inflation and Algorithmic Correction | Exclude incarcerated, count at home | Dept. of Commerce v. New York |
| N.2 | College Students and the Campus-vs.-Home Counting Problem | Count at home address | Equal Protection |
| N.3 | Noncitizen Populations and Citizen VAP Redistricting | Citizen VAP balance metric | Evenwel v. Abbott (2016) |
| N.4 | Military and Overseas Populations in Redistricting | Home of record rule | UOCAVA; SCRA |
| N.5 | Total Population vs. Citizen VAP: A 50-State Empirical Comparison | Both, compared | All of the above |

## Contracts

Every N.1–N.4 paper must:
- Quantify the size of the adjustment in each of the 50 states (where data allows)
- Show the bisect plan under the adjusted vs. unadjusted metric for NC/WI/TX
- Report compactness and partisan outcome change vs. total-population baseline
- Cite the controlling legal authority for the population definition question

## Key Legal Anchors

- *Reynolds v. Sims* (1964): population equality required
- *Evenwel v. Abbott* (2016): total population permissible; citizen VAP left open
- *Dept. of Commerce v. New York* (2019): census citizenship question struck down
- *Prison gerrymandering*: over 40 states have adopted prison count reform since 2020

## Algorithm Note

The bisect pipeline's `--balance-metric` flag controls the population vector used for balancing. The default is `total_population` (Census P.L. 94-171 P1 table). N-track papers test `citizen_vap`, `total_vap`, and adjusted counts.
