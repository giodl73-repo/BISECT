---
name: review-vra
description: Deep audit of VRA compliance — code, results, and paper claims. Uses BOUNDARY (legal), MERIDIAN (algorithm), COMMONS (community), CONTOUR (data). Catches the exact class of bugs found today: vra_mode conflation, multi_constraint misuse, adaptive boost misconfiguration.
user_invocable: true
---

# VRA Compliance Review

Comprehensive audit of the VRA edge-weighting implementation. The VRA code is the most complex and drift-prone part of the pipeline. This skill runs four focused roles against code, outputs, and paper claims simultaneously.

## Input

Specify scope:
- `code` — audit `run_state_redistricting.py` and `recursive_bisection.py`
- `results` — audit V4 output data against paper D.0/D.1 predictions
- `papers` — audit D-track papers (D.0, D.1, D.2, D.3)
- `all` — full audit (default)

## Steps

### 1. Load the VRA code

Read:
- `scripts/pipeline/run_state_redistricting.py` — lines 183-325 (the VRA block)
- `src/apportionment/partition/recursive_bisection.py` — `__init__`, `_split_node`, worker function
- `src/apportionment/partition/vra_utils.py` — `create_vra_vertex_weights`, `analyze_mm_districts`
- `docs/concepts/vra-compliance.md` — the spec

### 2. MERIDIAN review — algorithm invariants

Check every invariant listed in `.roles/meridian.md` domains section:

**Invariant 1:** `vra_mode = True` throughout a VRA run
```python
# Correct: set True at start, never cleared
vra_mode = True
...
edge_weights = vra_edge_weights
# vra_mode stays True — do NOT set to False here
```

**Invariant 2:** `multi_constraint` derived from `vra_target_tree`, never from `vra_mode`
```python
# Correct
multi_constraint=(self.vra_target_tree is not None)
# WRONG
multi_constraint=vra_mode  # would be True, enabling 2D METIS
```

**Invariant 3:** Population calculations use `vertex_weights.ndim`, not `vra_mode`
```python
# Correct
if vertex_weights.ndim == 2:
    pop = vertex_weights[i, 0]
else:
    pop = vertex_weights[i]
# WRONG
if vra_mode:
    pop = vertex_weights[i, 0]  # 1D array → IndexError
```

**Invariant 4:** Adaptive boost formula
```python
# Correct
minority_frac = is_minority.mean()
MINORITY_WEIGHT = max(3.0, 10.0 * (1.0 - 0.7 * minority_frac))
# Check: Alabama (~40% minority tracts) → ~8.5x; California (~63%) → ~5.6x; floor 3x
```

**Invariant 5:** `vra_analysis.pkl` written when `vra_mode=True`
```python
# Must execute after partition, while vra_mode is still True
if vra_mode:
    vra_analysis = vra_utils.analyze_mm_districts(tracts_with_demo, assignments_array)
    pickle.dump(vra_analysis, open(vra_file, 'wb'))
```

**Invariant 6:** 1D vertex weights passed to RecursiveBisection
```python
# VRA mode must NOT set vertex_weights to 2D
# vertex_weights should remain graph_data['vertex_weights'] throughout
```

For each invariant: PASS / FAIL / WARNING with file:line evidence.

### 3. BOUNDARY review — constitutional invariants

**Population balance:**
- All 50 states in V4 must be within ±0.5% population deviation
- Load `outputs/V4/2020/states/*/data/final_assignments.pkl` and compute
- Flag any state exceeding 0.5%

**VRA legal targets:**
Cross-reference against known Section 2 litigation targets:
```
Alabama:        2 MM districts (Allen v. Milligan)
Georgia:        5 MM districts (historical)
Louisiana:      2 MM districts (litigation)
Mississippi:    2 MM districts (standard)
South Carolina: 3 MM districts (litigation target)
```

Report achieved vs. target with geographic explanation for gaps.

### 4. COMMONS review — community outcomes

For each VRA covered state, check:
- Does the best-achieving district actually cross the 50% threshold?
- Are the MM districts in the right geographic regions (Black Belt for Alabama, Atlanta + southwest Georgia)?
- Does the compactness-VRA tradeoff result in fragmented communities?

Load `outputs/V4/2020/states/*/data/vra_analysis.json` for per-district minority percentages.

### 5. CONTOUR review — data integrity

- Is minority percentage computed from VAP or total population? (Should be noted in paper)
- Are 2020 demographics from PL 94-171 or ACS? (PL 94-171 is correct for redistricting)
- Is the 40% threshold applied to tract pct_minority from the demographics CSV?

### 6. Summary

```
VRA CODE AUDIT:
  Invariant 1 (vra_mode stays True):     PASS / FAIL
  Invariant 2 (multi_constraint source):  PASS / FAIL
  Invariant 3 (ndim-based indexing):     PASS / FAIL
  Invariant 4 (adaptive boost formula):  PASS / FAIL
  Invariant 5 (vra_analysis.pkl saved):  PASS / FAIL
  Invariant 6 (1D vertex weights):       PASS / FAIL

VRA RESULTS (V4 2020):
  Population balance: N/44 states pass ±0.5%
  MM districts achieved vs. targets:
    Alabama:        N/2 (gap: geographic — Black Belt tract concentration)
    Georgia:        N/5 ✓
    Louisiana:      N/2
    Mississippi:    N/2 ✓
    South Carolina: N/3 (gap: geographic — dispersed minority population)

PAPER ALIGNMENT (D-track):
  {N claims match current results}
  {N claims need updating}
```

## Key Rules

- **Any FAIL on invariants 1-6 is a critical bug.** These are the exact categories that caused today's failures.
- **California (63% minority) and Texas (52% minority) are sentinel states.** If they fail population balance, the adaptive boost formula has drifted.
- **Alabama 2 MM is the breakthrough result.** Paper D.0 says edge-weighting achieves this; the V4 run must confirm it. If not, document why.
- **`vra_analysis.pkl` not existing = analysis step was skipped.** This means `vra_mode` was False when the analysis block ran.
