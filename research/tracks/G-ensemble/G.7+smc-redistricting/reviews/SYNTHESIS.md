# Review Synthesis — G.7: Sequential Monte Carlo for Redistricting
**Round**: 1 | **Score**: 3.2/4 | **Verdict**: Conditional Accept

## Panel Scores
Karypis 3/4, Rodden 3/4, Duchin 4/4, Stephanopoulos 3/4, Liang 3/4

## Program Context
G.7 is the theoretical foundation for G.14's recommendation to use SMC for calibrated court-facing ensemble inference. With Phase 2 now complete (TX k=38 and CA k=52 validated; SmcPercentile compositor implemented), G.7 has matured from first draft to a substantively complete paper. Duchin gives 4/4 — the strongest single-reviewer score for any G-track paper.

## Consensus Strengths
- The ESS degradation model (Proposition: ESS ≥ exp(-k/10) × N) is the paper's theoretical anchor. Phase 2 empirical results confirm the prediction (TX ESS = 4,200 observed vs. ≥ 2,232 predicted; CA ESS = 12,500 observed vs. ≥ 9,119 predicted).
- Kahan summation for weight normalisation is a rigorous engineering choice that demonstrates the paper's attention to numerical precision.
- The SmcPercentile integration completes the connection to the `bisect` CLI — practitioners can now use SMC plan selection as a concrete workflow step.
- The six-method comparison table (G.7 §3) positioning SMC in the redistricting toolkit is the clearest exposition of when to use which method in the entire program.

## P1 Items
- **P1-A** (Karypis, Liang): The ESS degradation proposition is stated as a heuristic ("expected" ESS ≥ exp(-k/10) × N) without proof. Either prove this bound formally or label it explicitly as an empirical regularity derived from Phase 2 data and demote from Proposition to Empirical Observation.
- **P1-B** (Rodden): The court-admissibility claim (§7: "SMC percentile estimates are suitable for use in redistricting expert witness testimony") needs supporting argument. What makes an SMC estimate court-admissible that a ReCom estimate is not? The paper implies it's the calibration certificates (ESS, importance weights) — make this explicit with a reference to the evidentiary standards literature.
- **P1-C** (Stephanopoulos): The SmcPercentile selection step (selecting the plan at rank ⌊p × ESS⌋ in importance-weighted order) should be defined as a formal algorithm, not just a CLI flag description. Provide an algorithmic pseudocode block.

## P2 Items
- Add a table comparing SMC diagnostic output to ReCom diagnostic output side-by-side — practitioners need to know what they get additionally from SMC. (Liang)
- The connection between the importance weight distribution and the Gallagher-Maasoumi concentration measure is theoretically interesting — mention it. (Duchin)
- Cross-reference G.14 §5.1 which cites G.7 as its primary recommendation. (Karypis)

## Round 2 Path
Address P1-A (ESS proposition status) and P1-C (algorithm pseudocode) — both are one-paragraph fixes. P1-B (court admissibility) requires one careful paragraph citing evidentiary standards. Target R2 score ≥ 3.6/4.
