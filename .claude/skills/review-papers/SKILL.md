---
name: review-papers
description: Review a research paper using all 8 cartographic roles. Checks claims against actual pipeline data, flags undefended assertions, verifies paper-data alignment, and catches drift between research findings and current results.
user_invocable: true
---

# Research Paper Review

Run all 8 cartographic roles against a research paper. Each role interrogates the paper from its domain. The goal is not to reject findings — it is to ensure every claim is defensible against the actual V4/V3 pipeline outputs and the current state of the code.

The most common failure mode: a paper claims X based on an experiment that used different code or data than the current pipeline. DATUM catches this. SCALE catches inflated claims. MERIDIAN catches algorithm misrepresentation.

## Input

User specifies which paper to review. Examples:
- `research/D.0+vra-compliance/main.tex`
- `research/B.2+edge-weighted-bisection/main.tex`
- A specific claim: "check the +56% compactness improvement claim"

## Steps

### 1. Load the paper

Read `main.tex` and all `sections/*.tex`. Identify:
- The abstract's core claims (usually 3-5 numbered findings)
- The methodology section's algorithm description
- All quantitative results in the results section
- The conclusion's policy recommendations

### 2. Load the current data

Load the relevant pipeline output CSVs for cross-checking:
- `outputs/V3/2020/states/*/data/district_summary.csv` (compactness, population)
- `outputs/V4/2020/states/*/data/vra_analysis.json` (VRA compliance)
- `outputs/data/2020/partitioner_comparison/partitioner_comparison_2020.csv` (baseline comparison)

### 3. Run each role

**MERIDIAN — Algorithm claims:**
- Does the paper's description of recursive bisection match `src/apportionment/partition/recursive_bisection.py`?
- Are the METIS parameters (ufactor, nparts, -contig) correctly described?
- Does the edge weight formula in the paper match `run_state_redistricting.py`?
- For VRA papers: does the paper describe edge-weighting, not multi-constraint vertex weights?

**BOUNDARY — Legal claims:**
- Does the paper correctly state the ±0.5% constitutional standard?
- Are VRA Section 2 Gingles preconditions correctly applied?
- Are the covered states and their targets accurate?

**CONTOUR — Data claims:**
- Does the paper use PL 94-171 or ACS data? Are they correctly distinguished?
- Are GEOID formats correct? Are tract boundary changes across decades handled?
- Are minority percentages from VAP or total population — and does the paper say which?

**SCALE — Statistical claims:**
- Is the headline improvement (e.g., +56%) reported with uncertainty bounds?
- Are cross-decade comparisons accounting for tract boundary changes (MAUP)?
- Are state-level results aggregated correctly (weighted by districts vs. unweighted)?

**PRECINCT — Political claims:**
- Does the paper address the partisan effect of compact districts?
- Are comparison baselines from the same political era?
- Does the neutrality claim acknowledge geographic sorting as a confounder?

**DATUM — Evidence claims:**
- Can every quantitative result in the paper be reproduced from the methods section alone?
- Are negative results reported — states that DON'T achieve VRA targets?
- Is the scope of claims matched to the scope of evidence?

**COMMONS — Community claims:**
- Does the paper note which communities are split by the compactness optimization?
- For VRA papers: are the states that fail (Alabama, South Carolina) explained with geographic context?
- Is the policy recommendation qualified for states where the algorithm underperforms?

**SURVEY — Implementation claims:**
- Are runtime estimates accurate (current pipeline: ~1hr/year for 50 states)?
- Are the data requirements honestly stated (55GB Census files)?
- Is the output format described as court-ready when it isn't?

### 4. Paper-data alignment check

For each quantitative claim in the paper, cross-check against current output:

```
| Paper Claim | Paper Value | Current Data | Status |
|-------------|-------------|--------------|--------|
| National mean PP 2020 | 0.367 | 0.367 (V3) | MATCH |
| Alabama MM districts | 2 (edge-weighting) | 2 (V4) | MATCH |
| Louisiana MM districts | 2 | 1 | DRIFT — update paper |
| Georgia MM districts | 5 | 7 | DRIFT — paper understates |
```

### 5. Summary report

```
PAPER: {paper name}
LAST DATA RUN: V4 2020 / V3 2020/2010/2000

ROLE FINDINGS:
  MERIDIAN: {N findings — algorithm description accuracy}
  BOUNDARY: {N findings — legal standard accuracy}
  CONTOUR:  {N findings — data provenance accuracy}
  SCALE:    {N findings — statistical validity}
  PRECINCT: {N findings — political claims}
  DATUM:    {N findings — evidence sufficiency}
  COMMONS:  {N findings — community impact accuracy}
  SURVEY:   {N findings — implementation claims}

PAPER-DATA DRIFT:
  {N claims match current data}
  {N claims need updating}
  {N claims cannot be verified from current outputs}

RECOMMENDED ACTIONS:
1. {specific text change with line number}
2. ...
```

## Key Rules

- **Drift is not a failure.** When a paper was written and current results don't match, that's information — the paper needs updating, not rejection.
- **DATUM is the gating role.** If a claim cannot be reproduced from the methods, it cannot be published regardless of what other roles say.
- **SCALE requires uncertainty.** A point estimate without confidence bounds is a conjecture, not a finding.
- **Check the VRA invariants.** Papers D.0-D.3 specifically: edge-weighting approach (not multi-constraint), adaptive boost formula, 42% threshold.
- **Mississippi and Louisiana currently show 1 MM district each, not 2.** Papers predicting 2 need to note this as a boundary condition finding.
