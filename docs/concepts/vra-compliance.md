# VRA Compliance and Majority-Minority Districts

## Short version

The Voting Rights Act (Section 2) requires states to draw majority-minority districts where minority populations are geographically concentrated. The `metis-vra` partition mode encodes this signal into the **graph edge weights**: edges between high-minority tracts are weighted 10× higher, making METIS reluctant to cut through minority clusters. Population balance is enforced by standard single-constraint METIS (1D vertex weights, ufactor=5 → ±0.5%). No multi-constraint optimization; no sacrifice of population balance.

---

## The algorithm (Paper D.0)

**Standard edge-weighted mode** (`metis-vra` differs in one place only):

```
vertex_weights = population per tract        ← SAME (1D, population only)
edge_weights   = shared boundary length      ← MODIFIED (see below)
METIS          = single-constraint (ufactor=5)  ← SAME
```

**VRA edge weight formula:**

```
w(u, v) = α  if minority(u) ≥ τ AND minority(v) ≥ τ
w(u, v) = 1  otherwise
```

Where:
- `τ = 0.40` (40% minority threshold — Paper D.0 optimal)
- `α = 10 × (1 − 0.7 × minority_fraction)` capped at minimum 3×
  - Low-minority states (Alabama, 40%): α ≈ 8.5×
  - High-diversity states (California, 63%): α ≈ 5.6×
  - The taper prevents near-uniform weighting from disrupting balance

**Why this works:** METIS minimizes weighted edge cuts. Heavy minority-minority edges mean METIS avoids cutting through minority clusters — they stay together as districts. Population balance is guaranteed by the same ufactor=5 constraint used everywhere.

## Population balance guarantee

Constitutional one-person-one-vote requires ±0.5% population balance. The edge-weighting approach maintains this because:

1. **1D vertex weights** — METIS has a single population constraint, optimized directly
2. **ufactor=5** — allows maximum 0.5% imbalance per split
3. **No multi-constraint** — adding minority VAP as a second vertex weight dimension would allow METIS to trade population balance for minority concentration (this was the wrong approach, abandoned after testing)

All 50 states in the V4 run pass ±0.5% balance. Worst: California (52D) at 0.38%.

## Results (V4 run, 2020 Census)

| State | Minority % | Target MM | Achieved MM | Status |
|-------|-----------|-----------|-------------|--------|
| Georgia | 42.4% | 5 | 6 | ✓ Exceeds |
| Mississippi | 46.1% | 2 | 2 | ✓ Meets |
| Louisiana | 41.6% | 2 | 1 | Partial |
| Alabama | 36.9% | 2 | 0 | Geographic limit |
| South Carolina | 35.1% | 3 | 0 | Geographic limit |

**Paper D.1 finding confirmed:** States with ≥42% minority population achieve full VRA compliance. Below ~37%, geographic concentration prevents MM districts regardless of algorithm.

## Code variables — what they mean

| Variable | Meaning | Never means |
|----------|---------|-------------|
| `vra_mode=True` | "This is a VRA run" (for logging + MM analysis) | "Use 2D vertex weights" |
| `vra_target_weights=None` | No target tree → `multi_constraint=False` in METIS | |
| `multi_constraint` | Derived from `vra_target_tree is not None` | Derived from `vra_mode` |
| `vertex_weights` | Always 1D (population) in all modes | 2D in VRA mode |

## What NOT to do

**Do NOT use multi-constraint vertex weights for VRA:**
```python
# WRONG — sacrifices population balance (49-69% deviation observed)
vertex_weights = np.column_stack([population, minority_vap])  # 2D
```

**Do NOT conflate `vra_mode` with `multi_constraint`:**
```python
# WRONG — propagates old incorrect assumption
multi_constraint=vra_mode  # removed from recursive_bisection.py
```

## Files

- `src/apportionment/partition/recursive_bisection.py` — `vra_mode` flag, `multi_constraint` derived from `vra_target_tree`
- `scripts/pipeline/run_state_redistricting.py` — VRA edge weight construction, adaptive boost formula
- `src/apportionment/partition/vra_utils.py` — `analyze_mm_districts()`, demographic loading
- Papers: D.0 (methodology), D.1 (42% threshold), D.3 (compactness tradeoff)
