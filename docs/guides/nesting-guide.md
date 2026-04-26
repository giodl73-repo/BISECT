# Multi-Chamber Nesting Guide

How to draw house and senate districts that nest correctly, which states require nesting, and how `redist` validates it.

---

## What is nesting?

In many states, senate districts must be composed of whole house districts — no house district may be split across two senate districts. This is called **nesting** or **alignment**.

Washington's example:
- 98 house districts, 49 senate districts
- Each senate district = exactly 2 house districts
- No house district may be in more than one senate district

This creates a clean hierarchy: voters in one senate district are always represented by exactly 2 house members from adjacent, non-overlapping territories.

---

## States with nesting requirements

| State | Requirement | Ratio |
|-------|-------------|-------|
| Washington | WA Const. Art. II §6 — senate districts composed of whole house districts | 2:1 (fixed) |
| New York | NY Const. Art. III §5 — senate districts composed of whole assembly districts | Variable |
| Illinois | IL Const. Art. IV §2 — legislative districts composed of whole representative districts | Variable |
| Maryland | MD Const. Art. III §2 — senatorial districts contain whole delegate districts | Variable |
| Minnesota | MN Const. Art. IV §3 — senate districts composed of house districts | Variable |
| Oregon | OR Const. Art. IV §6 — senate districts composed of whole house districts | 2:1 (fixed) |
| Wisconsin | WI Const. Art. IV §4 — senate districts composed of whole assembly districts | 3:1 (fixed) |

**Fixed ratio** states: the senate district always contains exactly N whole house districts (WA, OR: 2; WI: 3).

**Variable ratio** states: the ratio can vary between senate districts as long as every house district is wholly inside one senate district.

---

## Drawing nested districts with `redist suite`

```bash
redist suite --state WA --year 2020 --version WA_Plans \
  --name wa_commission_v1 \
  --house-districts 98 \
  --senate-districts 49 \
  --nest senate-in-house \
  --seed 42
```

What happens:
1. `redist` draws 98 house districts first (using METIS on census tracts)
2. It builds a new adjacency graph where **nodes are house districts** (not tracts)
3. METIS runs again on this house-district graph with k=49
4. Each senate district = 2 whole house districts — nesting is guaranteed by construction
5. `redist` validates the nesting and exits with an error if any violation is detected

### For variable-ratio states

```bash
# Illinois: variable ratio, must specify explicitly
redist suite --state IL --year 2020 --version IL_Plans \
  --name il_commission_v1 \
  --house-districts 118 \
  --senate-districts 59 \
  --nest senate-in-house \
  --nest-ratio 2:1 \    # Illinois is variable, must specify
  --seed 42
```

For variable-ratio states, `--nest-ratio` is required. `redist` exits with an error if you omit it:

```
ERROR: IL nesting ratio is variable by statute. Specify --nest-ratio N:M (e.g. --nest-ratio 2:1).
```

---

## How the nesting algorithm works

### Step 1: Draw house districts

METIS partitions census tracts into 98 districts using the standard algorithm. This step is identical to drawing congressional districts.

### Step 2: Build house-district adjacency graph

Two house districts are adjacent if any of their constituent tracts share a boundary. This creates a new graph where each node is a house district.

**Primary-component rule**: If a house district is noncontiguous (which should not happen — `redist` enforces contiguity), only the largest connected component of that district's tracts is used to determine adjacency. Secondary components are excluded from the senate adjacency graph.

### Step 3: Draw senate districts on the house-district graph

METIS runs on the house-district adjacency graph with k=49. Each "node" in the senate partition corresponds to 2 house districts.

This guarantees nesting by construction — senate districts are defined entirely in terms of house districts, never in terms of individual tracts.

---

## Validating nesting

After drawing, `redist suite validate` checks that nesting holds:

```bash
redist suite validate --name wa_commission_v1 --version WA_Plans --year 2020
```

Output:
```json
{
  "nesting": {
    "mode": "senate-in-house",
    "valid": true,
    "violations": [],
    "senate_to_house_map": {
      "1": [1, 2],
      "2": [3, 4],
      ...
      "49": [97, 98]
    }
  }
}
```

If a violation is detected:
```json
{
  "nesting": {
    "valid": false,
    "violations": [
      {
        "senate_district": 3,
        "house_districts_contained": [5, 6, 7],
        "message": "Senate district 3 contains 3 house districts (expected 2)"
      }
    ]
  }
}
```

`redist suite validate` exits with code 4 (nesting violation bit) if any violation is found.

---

## The senate-to-house map

The `senate_to_house_map` in the suite manifest is the official record of which house districts are in each senate district. This is:
- Useful for verifying nesting in the commission report
- Required by some state constitutions to be published alongside the plan
- The basis for voter communication ("You are in Senate District 7, which includes House Districts 13 and 14")

---

## What if my house plan has a noncontiguous district?

If any house district fails contiguity — even with `--allow-noncontiguous` — `redist suite` will refuse to draw senate districts:

```
ERROR: Senate nesting requires all house districts to be contiguous.
House district 7 has 2 connected components.
Fix the house plan before attempting nesting. Do not use --allow-noncontiguous with --nest.
```

Noncontiguous house districts make the senate adjacency graph undefined — the algorithm can't determine which senate district a disconnected tract "belongs to." Fix contiguity first.

---

## Exporting a suite

```bash
redist export --suite wa_commission_v1 --year 2020 --version WA_Plans \
  --format geojson shapefile rplan \
  --out exports/wa_commission_v1/
```

Produces:
```
exports/wa_commission_v1/
  suite.json                          suite manifest with senate-to-house map
  wa_commission_v1_congressional.rplan
  wa_commission_v1_house.rplan
  wa_commission_v1_senate.rplan
```

Each `.rplan` file is independently valid and can be validated separately. The `suite.json` binds them together with the nesting map.

---

## Common questions

**Q: Can I draw senate districts first and then house?**
No. The nesting algorithm requires house districts to exist first — senate districts are defined as groups of house districts. Drawing senate first and trying to fit house inside them is mathematically harder and often impossible to do exactly.

**Q: What if the state requires a 2:1 ratio but the district count doesn't divide evenly?**
It doesn't apply to Washington (98/49 = 2 exactly). States like Minnesota (134 house / 67 senate = 2 exactly) and Wisconsin (99 house / 33 senate = 3 exactly) have counts designed to allow exact nesting. States with variable ratios (New York, Illinois) handle non-divisible cases by allowing some senate districts to contain 2 house districts and others to contain 3.

**Q: Does `redist` support three-level nesting (congressional → senate → house)?**
Not yet. The current `redist suite` supports congressional + house + senate as three independent plans. True three-level nesting (where senate districts must also nest inside congressional districts) requires additional constraint logic that is not yet implemented.
