# County and Municipal Preservation Guide

How `redist` measures political subdivision splits, why they matter legally, and what the standards are by state.

---

## Why preservation matters

Most state constitutions require redistricting plans to avoid splitting counties and municipalities. The specific language varies dramatically by state, and the legal consequences of splitting depend on the exact wording of your state's constitution.

`redist` computes split counts and reports them against your state's specific constitutional standard — not a generic score.

---

## Running the analysis

```bash
redist analyze --label wa_house_v1 --year 2020 --version WA_Plans --types splits
```

For municipal splits, you also need to download the Census geographic relationship files first:

```bash
redist fetch --type geography --year 2020 --states WA
redist analyze --label wa_house_v1 --types splits
```

---

## Reading the output

```json
{
  "analyzer": "splits",
  "counties": {
    "total": 39,
    "split": 4,
    "preservation_score": 0.897,
    "legal_standard": "WA Const. Art. II §43 — counties shall be preserved where possible",
    "compliance_assessment": "4 splits present; no binding numerical limit under WMCA case law",
    "disclaimer": "Legal compliance determination requires counsel",
    "split_list": [
      {
        "county_fips": "53033",
        "county_name": "King",
        "districts_containing": [1, 7, 9],
        "split_severity": 3
      }
    ]
  },
  "municipalities": {
    "available": true,
    "total": 281,
    "split": 12,
    "preservation_score": 0.957,
    "split_list": [...]
  }
}
```

### Key fields

- **`preservation_score`**: `(total - split) / total`. 1.0 = no splits; 0.0 = all split.
- **`legal_standard`**: The specific constitutional language for this state.
- **`compliance_assessment`**: What the split count means under that state's law.
- **`split_severity`**: How many districts contain tracts from this county/municipality. Severity 2 = split across 2 districts; severity 3 = split across 3, etc.

---

## State-specific standards

### Washington

**Constitutional language**: WA Const. Art. II §43 — "counties shall be preserved where possible"

**Case law**: *Washington Majority Coalition v. Gregoire* (WMCA) — The Washington Supreme Court does not impose a hard numerical limit on county splits. "Where possible" is contextual — a large county like King that spans multiple districts is expected to be split; a small rural county that is split when it didn't need to be is a stronger violation.

**What to look for**: Which counties are split? Are they large urban counties (expected) or small rural counties (potentially problematic)? Are they split across 2 districts (minimal) or 5 (severe)?

### California

**Constitutional language**: CA Const. Art. XXI §2(d) — "shall minimize the division of any county"

**Standard**: California has a stricter standard than Washington. The Independent Redistricting Commission uses a two-part test: (1) minimize the number of county splits, and (2) minimize the population of any county fragment that falls in a different district.

`redist` reports both: split count and the population of split county fragments.

### Texas

**Congressional districts**: No state constitutional preservation requirement for congressional maps (federal law governs). Texas has a statutory preference for county preservation in state legislative maps.

**State legislative districts**: Tex. Gov't Code §812.005 — preserve county and precinct lines "to the extent possible."

### Colorado

**Constitutional language**: CO Const. Art. V §47 — "political subdivisions shall be preserved ... a county shall not be divided unless absolutely necessary."

**Standard**: "Absolutely necessary" is a higher bar than Washington's "where possible." Colorado courts have invalidated maps that split counties without demonstrating necessity.

### North Carolina, Georgia, Virginia

These states have had significant redistricting litigation. `redist` reports split counts but does not determine whether splits were "necessary" — that is a legal determination requiring facts about whether the split was avoidable.

### States without a splits requirement

Some states have no constitutional county preservation requirement (e.g., Rhode Island). For these states, `redist` reports split counts without a legal standard field.

---

## Municipal splits

Municipal splits are computed from the Census Bureau's Place-to-Tract relationship files. A city is "split" if its census-designated place (CDP) spans more than one district.

**Important**: Census-designated places include unincorporated areas that may not be legally considered "municipalities" under your state's constitution. Verify which entities your state's constitution covers before relying on `redist`'s municipal split count.

**Data availability**: Municipal split data requires `redist fetch --type geography`. If this data is unavailable and your state's constitution requires municipal preservation, `redist analyze --types splits` exits with an error rather than silently omitting municipal splits.

---

## Comparing your plan to the enacted map

```bash
redist compare --plan-a wa_house_v1 --enacted --year 2020 --version WA_Plans
```

The comparison output shows:
```
COUNTY SPLITS
  Your plan     Splits: 4
  Enacted       Splits: 7
  Difference: -3 splits (your plan preserves more counties)
```

Courts and commissions often evaluate plans relative to each other and to the enacted map, not against an absolute standard. A plan with 4 splits is easier to defend than one with 7 when the enacted map has 7.

---

## Minimizing splits in your plans

`redist` does not optimize for split minimization directly — it optimizes for compactness (edge-weighted mode) or population balance. To reduce splits, try:

1. **More seeds**: Run several plans with different seeds and pick the one with fewest splits.
2. **Edge weights**: In `--partition-mode edge-weighted`, edges along county lines get high weight (long shared boundary), so METIS tends to avoid cutting them.
3. **Manual review**: After the algorithmic run, inspect `splits.json` for split counties that could be reassigned to a single district without violating population balance.

Future versions of `redist` may support a county-preservation constraint mode that penalizes splits directly in the METIS objective.

---

## Severity scoring

Not all splits are equal. `redist` reports `split_severity` — the number of districts containing tracts from a split county:

| Severity | Meaning | Legal exposure |
|----------|---------|----------------|
| 2 | County in 2 districts | Minimal — often unavoidable for large counties |
| 3 | County in 3 districts | Moderate — requires explanation |
| 4+ | County in 4+ districts | Severe — likely legally vulnerable |

King County (Seattle area) at severity 3 in a 10-district congressional map is expected — King County has 2.3 million people in a state of 7.7 million, so it will necessarily span multiple districts. A rural county of 50,000 people at severity 2 is harder to justify.
