# Review 5 — Reviewer: Christina Liang (Computational Social Science / Causal Inference)
**Paper:** B.18 — Redistricting Stability Across Reapportionment Cycles
**Round:** 1
**Score:** 3/4

## Summary

A technically sound paper on an interesting structural property of the ApportionRegions algorithm. The prime factorization analysis is the paper's main contribution, and it is correct and well-presented. The main weakness is the simulation methodology — specifically, the use of single-seed runs and the absence of confidence intervals on the Hamming distance estimates.

## Strengths

The factorization analysis is original and well-executed. The connection between k's prime factorization and the bisection tree depth is a structural property of the AR algorithm that has not been explicitly analyzed in prior work. The predictability of this structure — derivable years in advance from Census Bureau population projections — is an important practical observation.

The simulation design is appropriately scoped. Holding geography fixed and varying only seat count is exactly the right way to isolate the tree-structure effect from the population-redistribution effect. Acknowledging that the actual 2030 disruption will be larger (due to population redistribution) is honest and appropriate.

The three-type taxonomy of tree changes (flat-prime, hierarchical-composite, prime-to-composite, composite-to-prime) provides a useful framework for predicting disruption severity before running any simulation.

## Weaknesses and Concerns

The simulation results (Table 3) report single Hamming distance values for each state with no uncertainty quantification. A Hamming distance of 0.23 for Texas is a point estimate from a single pair of random seeds (one seed for the k=38 run, one for the k=41 run). This estimate has substantial variance: a different seed pair could produce a Hamming distance anywhere from roughly 0.15 to 0.35, depending on which specific 38-district and 41-district plans are generated.

The paper should run multiple seed pairs for each state and report mean ± standard deviation Hamming distances, or at minimum the range across 5-10 seed pairs. Without this, the claim that "Texas: d_Ham = 0.23" is not a stable empirical finding — it is a single data point from a stochastic process.

The comparison of Texas reapportionment disruption to political redistricting disruption (35–55% of tracts) lacks citations. Where does the 35–55% figure come from? This is a major substantive claim (algorithmic reapportionment is less disruptive than political redistricting), and it needs empirical support. The paper should cite either a systematic study of tract-level changes across political redistricting cycles or provide its own calculation using the 2000→2010 and 2010→2020 political redistricting changes for states like Ohio, North Carolina, and Texas.

The "9 ReCom steps" comparison (Texas reapportionment ≈ 9 ReCom steps in disruption) is computed as 0.23 / 0.026 = 8.8. But this arithmetic uses the expected ReCom step size (1/k ≈ 1/38), which is a rough approximation. The paper should either defend this approximation (e.g., by showing that the distribution of ReCom step sizes for k=38 has small variance) or use a more robust comparison.

## Minor Issues

- The open question section (7.3) is engaging but too long relative to the paper's core contribution. The prime frequency discussion in particular would fit better as a footnote.
- The abbreviations `k₂₀` (2020 seat count) and `k₃₀` (2030 projected seat count) are used in Table 3 but not defined anywhere before the table. Define them explicitly.
- The Huntington-Hill method is correctly cited but the paper does not show the Huntington-Hill calculation for any of the projected seat counts. Including the HH apportionment calculation for Texas (current population × projected growth rate) would strengthen the projection claim.

## Recommendation

Accept with minor revisions. Report mean ± standard deviation Hamming distances from multiple seed pairs (not just single-seed point estimates), and add citations for the 35–55% political redistricting disruption claim.
