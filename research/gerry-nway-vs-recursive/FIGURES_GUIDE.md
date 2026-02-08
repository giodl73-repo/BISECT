# Figures Integration Guide

**Date Created**: February 8, 2026
**Paper**: N-way vs Recursive Bisection for VRA-Compliant Redistricting
**Location**: `research/gerry-nway-vs-recursive/figures/`

---

## Generated Figures

All 6 figures successfully generated from 1,760 redistricting runs (880 n-way + 880 recursive across 44 multi-district states).

### Figure 1: Overall Success Rates
**File**: `fig1_overall_success_rates.png` (144KB)
**Content**: Bar chart comparing overall VRA compliance success rates
- N-way: 47.5% (418/880)
- Recursive: 48.3% (425/880)
- Difference: 0.8 percentage points (not statistically significant, p=0.634)

**Placement**: Section 5 (Results), after Table 1
**Caption**:
```latex
\begin{figure}[h]
\centering
\includegraphics[width=0.7\textwidth]{figures/fig1_overall_success_rates.png}
\caption{Overall VRA compliance success rates for n-way and recursive bisection methods across 44 multi-district states with 20 parameter configurations each (880 runs per method). The difference of 0.8 percentage points is not statistically significant (paired t-test, $p=0.634$, Cohen's $d=-0.018$).}
\label{fig:overall_success}
\end{figure}
```

---

### Figure 2: Parameter Sensitivity Heatmaps
**File**: `fig2_parameter_sensitivity.png` (187KB)
**Content**: Side-by-side heatmaps showing success rate by weight factor (α) and minority threshold (τ)
- N-way best: α=50, τ=0.40 (52.3% success)
- Recursive best: α=100, τ=0.50 (56.8% success)
- Shows different parameter landscapes for each method

**Placement**: Section 5 (Results), after parameter sensitivity discussion
**Caption**:
```latex
\begin{figure*}[t]
\centering
\includegraphics[width=\textwidth]{figures/fig2_parameter_sensitivity.png}
\caption{Parameter sensitivity analysis showing success rates across weight factors ($\alpha$) and minority thresholds ($\tau$) for both methods. N-way (left) achieves optimal performance at $\alpha=50, \tau=0.40$ (52.3\%), while recursive (right) peaks at $\alpha=100, \tau=0.50$ (56.8\%). Blue boxes indicate best configurations. Color scale: green (higher success) to red (lower success).}
\label{fig:parameter_sensitivity}
\end{figure*}
```

---

### Figure 3: State-by-State Comparison
**File**: `fig3_state_comparison.png` (193KB)
**Content**: Horizontal bar chart showing advantage margins for top 10 states each direction
- N-way advantages: Virginia (+35%), Missouri (+20%), Wisconsin (+20%)
- Recursive advantages: Connecticut (+45%), Louisiana (+25%), Georgia (+15%)
- 70.4% of states show ties (within ±5 points)

**Placement**: Section 5 (Results), after state-specific results discussion
**Caption**:
```latex
\begin{figure*}[t]
\centering
\includegraphics[width=\textwidth]{figures/fig3_state_comparison.png}
\caption{State-specific method advantages showing top 10 states favoring each method. Positive values indicate n-way advantage (blue); negative values indicate recursive advantage (orange). Most states (31/44) show ties within ±5 percentage points. Notable patterns: n-way excels in states with 8-12 districts and concentrated urban minorities, while recursive performs better in states with 4-7 districts and dispersed minorities.}
\label{fig:state_comparison}
\end{figure*}
```

---

### Figure 4: Runtime Distribution
**File**: `fig4_runtime_comparison.png` (163KB)
**Content**: Box plots and histograms comparing runtime distributions
- N-way: 3.68s average, 2.05s std dev
- Recursive: 11.33s average, 7.54s std dev
- N-way is 67.5% faster

**Placement**: Section 5 (Results), after runtime analysis
**Caption**:
```latex
\begin{figure*}[t]
\centering
\includegraphics[width=\textwidth]{figures/fig4_runtime_comparison.png}
\caption{Runtime distribution comparison. Left: box plots showing median, quartiles, and outliers. N-way (blue) has tighter distribution and lower median (3.68s) compared to recursive (orange, 11.33s). Right: histograms showing frequency distributions. N-way exhibits 67.5\% speed advantage, crucial for ensemble generation and interactive applications.}
\label{fig:runtime_distribution}
\end{figure*}
```

---

### Figure 5: Best Configurations
**File**: `fig5_best_configs.png` (173KB)
**Content**: Top 5 parameter configurations for each method
- N-way top 5: α=50/τ=0.40 leads at 52.3%
- Recursive top 5: α=100/τ=0.50 leads at 56.8%
- Shows 4.5 percentage point ceiling difference

**Placement**: Section 5 (Results), after parameter optimization results
**Caption**:
```latex
\begin{figure*}[t]
\centering
\includegraphics[width=\textwidth]{figures/fig5_best_configs.png}
\caption{Top 5 parameter configurations ranked by success rate for each method. N-way (left) achieves 52.3\% peak with $\alpha=50, \tau=0.40$ (gold border). Recursive (right) reaches 56.8\% with $\alpha=100, \tau=0.50$ (gold border). The 4.5 percentage point gap represents recursive's higher optimization ceiling, though requiring more careful parameter tuning.}
\label{fig:best_configs}
\end{figure*}
```

---

### Figure 6: Speed-Quality Trade-off
**File**: `fig6_speed_quality_tradeoff.png` (264KB)
**Content**: Scatter plot showing success rate vs runtime for all 20 configurations
- Each point: one configuration averaged across 44 states
- N-way (circles): clustered at faster runtime
- Recursive (squares): wider spread, higher ceiling
- Annotations for best configurations

**Placement**: Section 6 (Discussion), illustrating speed-quality trade-off
**Caption**:
```latex
\begin{figure*}[t]
\centering
\includegraphics[width=\textwidth]{figures/fig6_speed_quality_tradeoff.png}
\caption{Speed-quality trade-off visualization. Each point represents one parameter configuration's average performance across 44 states. N-way configurations (circles, blue) cluster at faster runtimes with moderate success rates. Recursive configurations (squares, orange) spread wider with higher peak but also higher variance. Trade-off: n-way offers 67.5\% speed advantage for only 0.8 point average performance sacrifice, but recursive achieves 4.5 point higher ceiling (56.8\% vs 52.3\%) with careful tuning.}
\label{fig:speed_quality_tradeoff}
\end{figure*}
```

---

## Integration Instructions

### Step 1: Add graphicx Package
In `main.tex` preamble:
```latex
\usepackage{graphicx}
```

### Step 2: Insert Figures in Sections

**Section 5.1 (Overall Performance)**: Add Figure 1 after Table 1

**Section 5.2 (Parameter Sensitivity)**: Add Figure 2 after Tables 2-3

**Section 5.3 (State-Specific Results)**: Add Figure 3 after Table 5

**Section 5.4 (Runtime Analysis)**: Add Figure 4 after Tables 6-7

**Section 5.5 (Best Configurations)**: Add Figure 5 after Table 4

**Section 6 (Discussion)**: Add Figure 6 in speed-quality trade-off subsection

### Step 3: Update Cross-References

Throughout text, reference figures:
- "as shown in Figure~\ref{fig:overall_success}"
- "Figure~\ref{fig:parameter_sensitivity} reveals..."
- "state-specific advantages (Figure~\ref{fig:state_comparison})"

### Step 4: Recompile

```bash
cd research/gerry-nway-vs-recursive
pdflatex main.tex
bibtex main
pdflatex main.tex
pdflatex main.tex
```

---

## Figure Quality

All figures generated at 300 DPI for publication quality:
- **Resolution**: 300 DPI (suitable for print)
- **Format**: PNG (widely compatible)
- **Color**: Colorblind-friendly palette
- **Typography**: Bold labels, clear annotations
- **File sizes**: 144KB-264KB (reasonable for LaTeX compilation)

---

## Design Choices

### Color Palette
- **N-way**: Blue (#1f77b4) - consistent across all figures
- **Recursive**: Orange (#ff7f0e) - consistent across all figures
- **Colorblind-safe**: Seaborn colorblind palette

### Typography
- **Font size**: 9-13pt (readable in two-column format)
- **Font weight**: Bold for labels, regular for annotations
- **Axis labels**: Bold with units

### Layout
- **Single-column figures**: 0.7\textwidth (Figures 1)
- **Two-column figures**: \textwidth (Figures 2-6)
- **Consistent spacing**: tight_layout() applied

### Statistical Annotations
- **P-values**: Included where relevant (Figure 1)
- **Effect sizes**: Cohen's d noted (Figure 1)
- **Best configurations**: Highlighted with gold borders (Figures 2, 5)
- **Quantitative labels**: All bars/points labeled with values

---

## Data Sources

All figures generated from:
1. `results/50_states_ablation_test.csv` - N-way runs (1001 rows)
2. `results/50_states_recursive_ablation.csv` - Recursive runs (1001 rows)

Generated by: `create_visualizations.py`

---

## Next Steps

1. ✅ Generated all 6 figures
2. ⏸ Insert figures into LaTeX sections
3. ⏸ Update figure references in text
4. ⏸ Recompile paper with figures
5. ⏸ Review figure placement and sizing
6. ⏸ Adjust captions if needed for journal submission

---

## Regeneration

To regenerate figures (if data changes):
```bash
cd research/gerry-nway-vs-recursive
python create_visualizations.py
```

All figures will be overwritten in `figures/` directory.

---

*For questions: giovanni@dellarec.com*
