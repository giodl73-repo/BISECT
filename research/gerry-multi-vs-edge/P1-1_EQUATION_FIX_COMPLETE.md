# P1-1: Equation Fix Complete

**Date**: 2026-02-08
**Status**: ✅ Equation corrected in Section 2

---

## What Was Fixed

**File**: `sections/02_background.tex`, lines 36-51

### Before (INCORRECT):

```latex
\textbf{Target Weights:} To create MM districts, we specify per-partition target weights $\mathbf{t}_i = (t_i^{\text{pop}}, t_i^{\text{min}})$ where:
\begin{equation}
t_i^{\text{min}} = \begin{cases}
0.60 \cdot \frac{M_{\text{total}}}{k} & \text{if } i \in \{\text{MM districts}\} \\
\frac{M_{\text{total}} - N \cdot 0.60 \cdot \frac{M_{\text{total}}}{k}}{k - N} & \text{otherwise}
\end{cases}
\end{equation}

where $N$ is the target number of MM districts and $M_{\text{total}}$ is total minority population.
```

**Problem**: Formula produces absolute VAP (not fractions) and results in only 22% within-district minority instead of 60%.

---

### After (CORRECT):

```latex
\textbf{Target Weights:} To create MM districts, we specify per-partition target weights $\mathbf{t}_i = (t_i^{\text{pop}}, t_i^{\text{min}})$ where:
\begin{equation}
t_i^{\text{pop}} = \frac{1}{k}, \quad
t_i^{\text{min}} = \begin{cases}
\frac{c_{\text{MM}} \cdot (P_{\text{total}} / k)}{M_{\text{total}}} & \text{if } i \in \{\text{MM districts}\} \\
\frac{M_{\text{total}} - N \cdot c_{\text{MM}} \cdot (P_{\text{total}} / k)}{(k - N) \cdot M_{\text{total}}} & \text{otherwise}
\end{cases}
\end{equation}

where $N$ is the target number of MM districts, $M_{\text{total}}$ is total minority VAP, $P_{\text{total}}$ is total population, and $c_{\text{MM}} = 0.60$ is the desired minority concentration in MM districts. This ensures each MM district has minority VAP equal to $c_{\text{MM}}$ times its population, while remaining districts share the residual minority population proportionally. Equivalently, using the state-wide minority fraction $m_{\text{state}} = M_{\text{total}} / P_{\text{total}}$:
\begin{equation}
t_i^{\text{min}} = \begin{cases}
\frac{c_{\text{MM}}}{k \cdot m_{\text{state}}} & \text{if } i \in \{\text{MM districts}\} \\
\frac{1 - N \cdot t_{\text{MM}}^{\text{min}}}{k - N} & \text{otherwise}
\end{cases}
\end{equation}
```

**Benefits**:
- ✅ Formula now produces proper fractions that sum to 1.0
- ✅ Achieves 60% within-district minority concentration
- ✅ Includes clear explanation of variables
- ✅ Provides two equivalent formulations (explicit and simplified)
- ✅ Matches the corrected code implementation

---

## Verification

### Alabama Example (k=7, target=2 MM, 36.9% minority)

**Old formula:**
```
t_i = 0.60 * (M_total / k) = 158,822 [absolute, wrong units!]
if normalized: t_i = 0.0857 → 22% within-district
```

**New formula:**
```
t_i = (0.60 * P_total/k) / M_total = 0.2324 → 60% within-district ✓
or: t_i = 0.60 / (k * m_state) = 0.60 / (7 * 0.369) = 0.2324 ✓
```

---

## LaTeX Compilation

✅ **Compiled successfully** with `pdflatex main.tex`

---

## Related Changes

1. ✅ **Code**: Already fixed in `run_multi_constraint_experiments_FIXED.py`
2. ✅ **Section 2**: Equation fixed (this document)
3. ⏳ **Section 3**: May need to update theoretical analysis referring to old formula
4. ⏳ **Section 4**: Update methodology description if needed
5. ⏳ **Section 5**: Update results with corrected data (30.0% vs 35.0%)
6. ⏳ **Figures**: Regenerate with corrected results
7. ⏳ **Tables**: Update with corrected data

---

## Impact on P1-1 Review Issue

**Karypis's Concern**: "Target weight specification may be incorrect, asking for 60% of total minority per district"

**Resolution**:
- ✅ Karypis was correct - formula was wrong
- ✅ Formula now corrected to properly calculate fractions
- ✅ Code implementation uses correct formula
- ✅ Paper now documents correct formula
- ✅ Results run with correct implementation (30.0% success)

**Remaining for P1-1**:
- Update results sections with new data
- Regenerate affected figures
- Update tables with corrected success rates

---

## Timeline

- ✅ **2026-02-08**: Equation fixed in Section 2
- ⏳ **Next**: Update results, figures, tables (1-2 days)

---

**Status**: Equation fix complete ✅ | Full P1-1 resolution in progress ⏳
