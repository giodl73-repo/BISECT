# Review Synthesis — B.10 Subdivision-Respecting Redistricting

**Round**: 1
**Avg score**: 1.5 / 4
**Reviewers**: R-50 (Ullman, 1.0), R-1 (Liang, 1.0), R-37 (Polikarpova, 2.0), R-44 (Steinhardt, 1.5), R-30 (Zhang, 2.0)
**Decision**: Major revision required. Not accepted at current form.

---

## Consensus

The paper presents a genuinely useful and well-motivated contribution: operationalizing
subdivision preservation as a tunable METIS edge-weight signal. All five reviewers
agree the mechanism is clean, the legal grounding is sound, and the 50-state scope
is commendable. However, all five identify the same two fatal flaws:

1. **The Monotonicity Proposition is incorrect for a heuristic solver** (R-37, R-50)
2. **The single-seed evaluation makes all quantitative claims unreliable** (all reviewers)

---

## P1 Items — Must Fix Before Acceptance

### P1-A: Fix or restrict the Monotonicity Proposition
The current proof establishes that the OPTIMAL min-cut partition has weakly fewer
county splits as α increases. METIS is not an optimal solver — it is a multilevel
heuristic. The proposition as stated is a correctness error.

**Resolution options (choose one):**
- Restrict the proposition to the exact weighted min-cut problem (remove METIS
  from the scope of the claim). Add a sentence: "We conjecture this property
  extends empirically to METIS; §4 provides 50-state evidence."
- Replace the formal proposition with an empirical monotonicity result: run 25 seeds
  per state at α∈{0,1,2,3,5}, compute mean splits(α), plot with CI. Show the
  curve is weakly decreasing with high probability.

### P1-B: Run multi-seed evaluation (≥25 seeds per state per α)
All five reviewers flag the single-seed limitation. The 45% split reduction and
2.5× EC cost are the paper's headline numbers — they must have variance estimates.

**Minimum requirement:**
- Run 25 seeds per state per α for α∈{0,5} (already planned)
- Report mean ± std for county splits and EC
- Run α sweep (α∈{0,1,2,5,10}) with 10 seeds each for at least 5 focal states
  (TX, GA, NC, PA, CA) to produce the Pareto frontier figure

### P1-C: Explain missing "--" entries in Table 1
R-30 flags this as a critical reproducibility issue. The "--" values in the EC
column (roughly half the table) have no explanation. This must be resolved:
- If EC was not computed for those states, say so and fill in from the sweep data
- If runs failed, report the failure rate and diagnose

**Note:** From our own sweep data, ALL states ran successfully — the "--" entries
simply mean EC was only spot-checked for focal states. Fill in the full EC column
from the b10_sweep.csv output.

### P1-D: Restrict or remove the partisan neutrality claim
R-44 flags that the GA result at α=0 (4D seats) contradicts B.7's converged result
(7D/7R) for the same state. R-1 notes the partisan section cannot be supported
from single-seed data.

**Resolution:**
- Explicitly label §4.3 as "Pilot Evidence" with a clear caveat that these are
  single-seed results
- OR run 25-seed ensembles for the 5 focal states and report the mean seat change
  with error bars
- Remove the "politically neutral" claim from the abstract and conclusion until
  the multi-seed analysis is available

---

## P2 Items — Important but Not Blockers

### P2-A: Add Pareto frontier figure
The "forthcoming" figure referenced in §5.1 is the paper's central empirical claim
(the α-sweep Pareto frontier). R-1 considers this a precondition for publication.
Run the b10_sweep.ps1 with α∈{0,0.5,1,2,5,10,20} for at least 10 focal states
and produce Figure 1.

### P2-B: Justify α=5 as a default recommendation
R-50, R-1, R-30 all note the α=5 recommendation is unjustified. After producing
the Pareto frontier, locate the knee of the splits(α) vs. EC(α) curve and
justify α=5 (or a different value) based on it.

### P2-C: Fix the Florida citation in §1
The cited Florida statute (prohibits drawing to favor/disfavor a party) is about
partisan gerrymandering, not subdivision preservation. The Florida subdivision
preservation criterion is in a different clause: "districts shall, where
feasible, utilize existing political and geographical boundaries."

### P2-D: Stability analysis for deep hierarchy composition
R-37 notes that stacking county+MCD+place weightings multiplies intra-VTD edge
weights by ∏(1+α_ℓ). For α_ℓ=5 at each of 3 levels, this is 216×. Characterize
the amplification and its effect on population balance.

---

## P3 Items — Nice to Have

- **Comparison to enacted maps**: How many county splits does the actual 118th
  Congress plan have? Are our α=5 plans closer to it than α=0?
- **county(·) domain clarification**: State that all census tracts in the
  adjacency graph have a unique county assignment (no tract straddles counties).
- **Wall-clock timing**: Report the overhead of SubdivisionWeighter vs. total
  METIS time for CA (largest state) to quantify "negligible."
- **Binary "same" threshold**: Define what "D seat change = 0" means — ±0 or
  some rounding threshold — for partisan table.

---

## Path to Acceptance

The revision is substantial but not conceptually difficult:

1. Fill Table 1 from existing sweep data (1 hour)
2. Run 25-seed ensembles for 5 focal states at α∈{0,5} (~2 hours compute)
3. Run α sweep {0,0.5,1,2,5,10,20} for focal states (~4 hours compute)
4. Fix Monotonicity Proposition (scope to exact min-cut, add empirical note)
5. Produce Pareto frontier figure
6. Revise §4.3 partisan section with ensemble results or pilot caveat
7. Fix Florida citation

Estimated revision effort: ~1 day compute + 1 day writing.
Expected score after revision: 2.5–3.0 (accept with minor revisions).
