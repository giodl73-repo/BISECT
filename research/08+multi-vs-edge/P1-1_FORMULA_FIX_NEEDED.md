# P1-1: Formula Fix Required in Paper

**Date**: 2026-02-08
**Issue**: Paper documents INCORRECT formula, but code now implements CORRECT formula

---

## Status

✅ **Code is CORRECT**: `run_multi_constraint_experiments_FIXED.py` uses right formula
❌ **Paper is WRONG**: `sections/02_background.tex` lines 38-40 have incorrect equation

---

## The Paper's Incorrect Formula

**Location**: `sections/02_background.tex`, lines 38-40

**Current (INCORRECT)**:
```latex
t_i^{\text{min}} = \begin{cases}
0.60 \cdot \frac{M_{\text{total}}}{k} & \text{if } i \in \{\text{MM districts}\} \\
\frac{M_{\text{total}} - N \cdot 0.60 \cdot \frac{M_{\text{total}}}{k}}{k - N} & \text{otherwise}
\end{cases}
```

**Why this is wrong**:
- For Alabama: `t_i = 0.60 * (1,852,928 / 7) = 158,822` (absolute VAP, not fraction!)
- If normalized: `t_i = 158,822 / 1,852,928 = 0.0857` → only **22%** minority within-district
- We want **60%**, not 22%!

---

## The Correct Formula

**CORRECT formula** (what the code now implements):
```latex
t_i^{\text{min}} = \begin{cases}
\frac{0.60 \cdot (P_{\text{total}} / k)}{M_{\text{total}}} & \text{if } i \in \{\text{MM districts}\} \\
\frac{M_{\text{total}} - N \cdot [0.60 \cdot (P_{\text{total}} / k)]}{(k - N) \cdot M_{\text{total}}} & \text{otherwise}
\end{cases}
```

Or equivalently (simpler form):
```latex
t_i^{\text{min}} = \begin{cases}
\frac{0.60}{k \cdot m_{\text{state}}} & \text{if } i \in \{\text{MM districts}\} \\
\frac{1 - N \cdot t_{\text{MM}}^{\text{min}}}{k - N} & \text{otherwise}
\end{cases}
```

where $m_{\text{state}} = M_{\text{total}} / P_{\text{total}}$ is the state-wide minority fraction.

**Why this is correct**:
- For Alabama: `t_i = 0.60 / (7 * 0.369) = 0.2324` → **60%** minority within-district ✓
- Fractions sum to 1.0 ✓
- Produces desired concentration ✓

---

## Required Fix

**File**: `research/gerry-multi-vs-edge/sections/02_background.tex`
**Lines**: 38-42

**Replace with**:
```latex
\textbf{Target Weights:} To create MM districts, we specify per-partition target weights $\mathbf{t}_i = (t_i^{\text{pop}}, t_i^{\text{min}})$ where:
\begin{equation}
t_i^{\text{pop}} = \frac{1}{k}, \quad
t_i^{\text{min}} = \begin{cases}
\frac{c_{\text{MM}} \cdot (P_{\text{total}} / k)}{M_{\text{total}}} & \text{if } i \in \{\text{MM districts}\} \\
\frac{M_{\text{total}} - N \cdot [c_{\text{MM}} \cdot (P_{\text{total}} / k)]}{(k - N) \cdot M_{\text{total}}} & \text{otherwise}
\end{cases}
\end{equation}

where $N$ is the target number of MM districts, $M_{\text{total}}$ is total minority VAP, $P_{\text{total}}$ is total population, and $c_{\text{MM}} = 0.60$ is the desired minority concentration in MM districts. Equivalently, using the state-wide minority fraction $m_{\text{state}} = M_{\text{total}} / P_{\text{total}}$:
\begin{equation}
t_i^{\text{min}} = \begin{cases}
\frac{c_{\text{MM}}}{k \cdot m_{\text{state}}} & \text{if } i \in \{\text{MM districts}\} \\
\frac{1 - N \cdot t_{\text{MM}}^{\text{min}}}{k - N} & \text{otherwise}
\end{cases}
\end{equation}

This ensures MM districts have $c_{\text{MM}}$ minority concentration while other districts share the remaining minority population proportionally.
```

---

## Verification

Test with Alabama (k=7, target=2 MM, 36.9% minority):

**Incorrect formula** (current paper):
```
t_i = 0.60 * (1,852,928 / 7) / 1,852,928 = 0.0857
Within-district: 22% ✗
```

**Correct formula** (fixed code):
```
t_i = 0.60 / (7 * 0.369) = 0.2324
Within-district: 60% ✓
```

---

## Timeline

Part of P1-1 revision - must fix before paper resubmission.

**Estimate**: 30 minutes to update LaTeX equation and add explanation.

---

## Related Files

- ✅ `run_multi_constraint_experiments_FIXED.py` - Code is correct
- ✅ `CORRECT_IMPLEMENTATION.py` - Reference implementation
- ❌ `sections/02_background.tex` - Needs equation fix (lines 38-42)
- ⚠️ `sections/03_theory.tex` - May reference the incorrect formula, check after fixing
