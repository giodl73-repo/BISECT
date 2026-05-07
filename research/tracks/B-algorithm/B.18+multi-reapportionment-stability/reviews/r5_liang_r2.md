# Review 5 — Reviewer: Christina Liang (Computational Social Science / Causal Inference)
**Paper:** B.18 — Redistricting Stability Across Reapportionment Cycles
**Round:** 2
**Score:** 3/4

## Summary

The revision addresses both of my principal concerns: the seed variance is now quantified (Section 4.4), and the 35–55% political redistricting disruption claim now has a footnote attributing it to the authors' analysis of 2000→2010 and 2010→2020 redistricting changes. I maintain my score at 3/4. The simulation methodology is now honestly stated, and the remaining issues are minor.

## Addressed Issues

The seed variance section (Section 4.4) directly answers my primary concern about single-seed Hamming distances. The finding that median variance is ±0.8 percentage points across T=600 seeds, with the qualitative ordering stable across seeds, is the key result. The paper now explicitly identifies seed=0 as the reporting point and provides the variance bound. This is the multi-seed distribution I requested, presented in a compact form.

The citation for the 35–55% political redistricting disruption claim now attributes it to the authors' own analysis of redistricting changes in Texas, Ohio, and North Carolina across 2000→2010 and 2010→2020 cycles. This is better than the uncited assertion in Round 1, though the methodology for this computation is only summarized in a footnote. I would prefer a supplementary analysis with more detail, but I accept the footnote as adequate for the paper's purposes.

The abbreviations k₂₀ and k₃₀ are now defined at first use.

The Huntington-Hill calculation for Texas (showing how 38→41 is derived from Census Bureau state-level projections) is now present in a footnote, which is the minimum I requested.

## Remaining Concerns

The ReCom step size comparison remains algebraic (0.23/0.026 ≈ 9 steps). The paper does not validate that the expected step size of 1/k=0.026 is a tight approximation for k=38 Texas. A brief justification of why the expected step size is appropriate (e.g., showing that the standard deviation of the step size distribution is small relative to 0.026) would strengthen this claim. I do not require the full empirical ReCom measurement that Duchin requested, but a variance estimate for the ReCom step size at k=38 would be valuable.

The California analysis at 52→51 (the more likely projection under some models) is still not present. The paper addresses 52→50 and its ±1 sensitivity, but 51 is within the range of current apportionment projections and has a different factorization (51=3×17, three-level tree) that could produce different structural analysis from 50=2×5². This is a minor concern since the paper correctly notes that both 50 and 51 are composite, but the d_Ham estimate at 51 would be useful.

The prime frequency discussion (Section 7.3) is appropriately shortened, as I suggested.

## Minor Issues

- The open questions section is now appropriately concise.
- The 9-step ReCom comparison should note the variance of the comparison — the actual number of steps could be 7-11 depending on the step size distribution.

## Recommendation

Accept. The seed variance quantification and the political redistricting disruption citation are the critical improvements. The simulation methodology is now honestly stated and the core claims are adequately supported.
